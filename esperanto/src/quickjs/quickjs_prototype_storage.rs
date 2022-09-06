use std::{any::TypeId, collections::HashMap, convert::TryInto, ffi::c_void, slice};

use by_address::ByAddress;
use quickjs_android_suitable_sys::{
    JSCFunctionEnum_JS_CFUNC_constructor, JSCFunctionEnum_JS_CFUNC_constructor_or_func,
    JSCFunctionEnum_JS_CFUNC_generic, JSContext as QuickJSContext, JSRuntime,
    JSValue as QuickJSValue, JS_DupValue__, JS_GetClassProto, JS_GetOpaque, JS_GetPropertyStr,
    JS_GetRuntime, JS_GetRuntimeOpaque, JS_GetTag__, JS_IsEqual__, JS_NewCFunction2, JS_NewClass,
    JS_NewClassID, JS_NewObject, JS_NewObjectProtoClass, JS_SetClassProto, JS_SetConstructor,
    JS_SetOpaque, JS_SetRuntimeOpaque, JS_EXCEPTION__, JS_NULL__, JS_UNDEFINED__,
};

use super::{
    quickjscontextpointer::QuickJSContextPointer, quickjsexport::QuickJSExportExtensions,
    quickjsruntime::QuickJSRuntimeInternal,
};
use crate::{
    export::{JSExportCall, JSExportMetadata},
    shared::{
        errors::{EsperantoResult, JSExportError},
        runtime::JSRuntimeError,
        value::JSValueInternal,
    },
    JSContext, JSExportClass, JSValueRef,
};

type JSClassIDStorage<'a> = HashMap<ByAddress<&'a JSExportMetadata<'a>>, u32>;

pub(super) fn get_classid_storage<'a>(
    runtime: QuickJSRuntimeInternal,
) -> EsperantoResult<&'a mut JSClassIDStorage<'a>> {
    // The storage is embedded in the runtime's opaque storage so let's grab it:
    let storage_ptr = unsafe { JS_GetRuntimeOpaque(runtime) } as *mut JSClassIDStorage;

    // This shouldn't ever fail because we attach storage when we create a runtime. But it's still
    // theoretically possible so we check if the raw pointer actually can be converted:
    unsafe { storage_ptr.as_mut() }.ok_or(JSRuntimeError::FailedToRetrievePrivateContext.into())
}

pub(super) fn attach_classid_storage(runtime: QuickJSRuntimeInternal) {
    // When we create a new runtime we make a new ID store for later use. We need to Box<>
    // it up in order to store it within QuickJS's opaque storage.

    let new_storage = JSClassIDStorage::new();
    let boxed = Box::new(new_storage);
    let raw_ref = Box::into_raw(boxed);
    unsafe { JS_SetRuntimeOpaque(runtime, raw_ref as *mut c_void) };
}

pub(super) fn drop_classid_storage(runtime: QuickJSRuntimeInternal) {
    // Once our runtime has been dropped we need to manually drop the ID storage
    // since it lives inside a raw pointer:
    let storage_ptr = unsafe { JS_GetRuntimeOpaque(runtime) } as *mut JSClassIDStorage;
    unsafe { JS_SetRuntimeOpaque(runtime, std::ptr::null_mut()) };
    let boxed = unsafe { Box::from_raw(storage_ptr) };

    // Not necessary but let's be explicit about why we're doing this:
    drop(boxed);
}

pub(super) fn get_class_id<T: JSExportClass>(
    runtime: *mut JSRuntime,
    storage: &mut JSClassIDStorage,
) -> EsperantoResult<u32> {
    let addr = ByAddress(&T::METADATA);

    // QuickJS uses u32 class "IDs" to ensure there aren't ever any
    // collisions between two different classes. First check: do we
    // already have a class ID for this class?

    if let Some(class_id) = storage.get(&addr) {
        // If yes just directly return it:
        return Ok(*class_id);
    }

    // If not then we need to ask QuickJS to generate a new, unique
    // ID:
    let mut class_id: u32 = 0;
    unsafe { JS_NewClassID(&mut class_id) };

    // Then create a class from our definition using that ID:
    let definition = T::class_def();
    unsafe { JS_NewClass(runtime, class_id, &definition) };

    // Finally we save this ID so that we can use it later
    storage.insert(addr, class_id);

    return Ok(class_id);
}

const STR_PROTOTYPE: *const i8 = b"prototype\0" as *const u8 as _;
const STR_CONSTRUCTOR: *const i8 = b"constructor\0" as *const u8 as _;

fn custom_class_constructor<T: JSExportClass>(
    ctx: *mut QuickJSContext,
    new_target: QuickJSValue,
    argc: i32,
    argv: *mut QuickJSValue,
) -> EsperantoResult<QuickJSValue> {
    // This gets called whenever e.g. "new NativeClass()" gets called in JS code.
    // let runtime = unsafe { JS_GetRuntime(ctx) };
    // let storage = get_classid_storage(runtime)?;

    let context: JSContext = ctx.into();

    // let class_id = get_class_id::<T>(runtime, storage)?;

    // // examples in the QuickJS codebase tell us "using new_target to get the prototype is
    // // necessary when the class is extended", so, OK then!
    // // https://github.com/bellard/quickjs/blob/b5e62895c619d4ffc75c9d822c8d85f1ece77e5b/examples/point.c#L60
    // let proto = unsafe { JS_GetPropertyStr(ctx, new_target, STR_PROTOTYPE) };

    // Then we create a new object with this prototype:
    // let new_object = unsafe { JS_NewObjectProtoClass(ctx, proto, class_id) };

    // Ensure that we release the prototype or we'll get a memory leak
    // proto.release(ctx.into());

    let constructor = match T::METADATA.call_as_constructor {
        Some(constructor) => constructor,
        _ => return Err(JSExportError::ConstructorCalledOnNonConstructableClass.into()),
    };

    let argc_size: usize = argc
        .try_into()
        .map_err(|err| JSExportError::CouldNotConvertArgumentNumber(err))?;
    let args = unsafe { slice::from_raw_parts(argv, argc_size) };

    let wrapped_args: Vec<JSValueRef> = args
        .iter()
        .map(|raw| JSValueRef::wrap_internal(unsafe { JS_DupValue__(ctx, *raw) }, &context))
        .collect();

    let item = (constructor.func)(&wrapped_args, &context)?;

    return Ok(item.internal.retain(context.internal));
}

unsafe extern "C" fn custom_class_call_extern<T: JSExportClass>(
    ctx: *mut QuickJSContext,
    new_target: QuickJSValue,
    argc: i32,
    argv: *mut QuickJSValue,
) -> QuickJSValue {
    let tag = unsafe { JS_GetTag__(new_target) };

    custom_class_constructor::<T>(ctx, new_target, argc, argv).unwrap_or(JS_EXCEPTION__)
}

pub(super) fn get_class_prototype<T: JSExportClass>(
    class_id: u32,
    in_context: *mut QuickJSContext,
) -> EsperantoResult<QuickJSValue> {
    let ctx: QuickJSContextPointer = in_context.into();

    // Haven't been able to replicate it but in theory QuickJS could garbage collect the prototype
    // at some point, so let's check here whether it actually still exists. If not we'll just create
    // it again.
    let existing_proto = unsafe { JS_GetClassProto(in_context, class_id) };
    if unsafe { JS_IsEqual__(existing_proto, JS_NULL__) == 0 } {
        // it isn't undefined so we assume it is the prototype. Could add extra checks but don't
        // think they're necessary?
        return Ok(existing_proto);
    }

    // If we can't find it, make it!
    // Create an 'empty' object that we'll throw all of this onto:
    let proto = unsafe { JS_NewObject(*ctx) };

    let proto_retained = unsafe {
        // Annoying thing: I don't know why this is required. You'd think
        // JS_NewObject would return a retained object. But if we don't do it
        // the runtime fails to free correctly.
        JS_DupValue__(in_context, proto)
    };

    // let cproto = match (
    //     T::METADATA.call_as_constructor,
    //     T::METADATA.call_as_function,
    // ) {
    //     (Some(_), Some(_)) => Some(JSCFunctionEnum_JS_CFUNC_constructor_or_func),
    //     (Some(_), None) => Some(JSCFunctionEnum_JS_CFUNC_constructor),
    //     (None, Some(_)) => Some(JSCFunctionEnum_JS_CFUNC_generic),
    //     (None, None) => None,
    // };

    if T::METADATA.call_as_constructor.is_some() {
        // First we create our constructor function that calls the generic function
        // defined above here
        let constructor = unsafe {
            JS_NewCFunction2(
                *ctx,
                Some(custom_class_call_extern::<T>),
                T::METADATA.class_name as *const i8,
                2,
                JSCFunctionEnum_JS_CFUNC_constructor,
                0,
            )
        };
        unsafe { JS_SetConstructor(*ctx, constructor, proto_retained) }
        // since this constructor is now attached to the prototype we don't need to hold our
        // own reference to it, so we release.
        constructor.release(ctx);
    }

    unsafe { JS_SetClassProto(*ctx, class_id, proto_retained) };

    return Ok(proto);
}

fn delete_stored_prototype<T: JSExportClass>(
    runtime: *mut JSRuntime,
    value: QuickJSValue,
) -> EsperantoResult<()> {
    println!("DELETE VALUE {:?}", value);
    let storage = get_classid_storage(runtime)?;

    let class_id = get_class_id::<T>(runtime, storage)?;
    let ptr = unsafe { JS_GetOpaque(value, class_id) as *mut T };

    let obj = unsafe { Box::from_raw(ptr) };
    drop(obj);
    Ok(())
}

pub(super) unsafe extern "C" fn delete_stored_prototype_extern<T: JSExportClass>(
    runtime: *mut JSRuntime,
    value: QuickJSValue,
) {
    delete_stored_prototype::<T>(runtime, value).unwrap();
}
