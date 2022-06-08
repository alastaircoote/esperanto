use std::{collections::HashMap, ffi::c_void};

use quickjs_android_suitable_sys::{
    JSCFunctionEnum_JS_CFUNC_constructor, JSContext as QuickJSContext, JSValue as QuickJSValue,
    JS_GetClassProto, JS_GetPropertyStr, JS_GetRuntime, JS_GetRuntimeOpaque, JS_IsEqual__,
    JS_NewCFunction2, JS_NewClass, JS_NewClassID, JS_NewObject, JS_NewObjectProtoClass,
    JS_SetClassProto, JS_SetConstructor, JS_SetRuntimeOpaque, JS_EXCEPTION__, JS_UNDEFINED__,
};

use super::{
    quickjscontextpointer::QuickJSContextPointer, quickjsexport::QuickJSExportExtensions,
    quickjsruntime::QuickJSRuntimeInternal,
};
use crate::{
    export::JSExportMetadata,
    shared::{errors::EsperantoResult, runtime::JSRuntimeError, value::JSValueInternal},
    JSExportClass,
};

type JSClassIDStorage<'a> = HashMap<&'a JSExportMetadata, u32>;

fn get_classid_storage<'a>(
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

fn get_class_id<T: JSExportClass>(
    ctx: QuickJSContextPointer,
    storage: &mut JSClassIDStorage,
) -> EsperantoResult<u32> {
    // QuickJS uses u32 class "IDs" to ensure there aren't ever any
    // collisions between two different classes. First check: do we
    // already have a class ID for this class?

    let runtime = unsafe { JS_GetRuntime(*ctx) };

    if let Some(class_id) = storage.get(&T::METADATA) {
        // If yes just directly return it:
        return Ok(*class_id);
    }

    // If not then we need to ask QuickJS to generate a new, unique
    // ID:
    let mut class_id: u32 = 0;
    unsafe { JS_NewClassID(&mut class_id) };

    // Then create a class from our definition using that ID:
    let definition = T::class_def();
    check_quickjs_exception!(ctx => {
        unsafe { JS_NewClass(runtime, class_id, &definition) };
    })?;

    // Finally we save this ID so that we can use it later
    storage.insert(&T::METADATA, class_id);

    return Ok(class_id);
}

const STR_PROTOTYPE: *const i8 = b"prototype\0" as *const u8 as _;
const STR_CONSTRUCTOR: *const i8 = b"constructor\0" as *const u8 as _;

unsafe extern "C" fn custom_class_constructor<T: JSExportClass>(
    ctx: *mut QuickJSContext,
    new_target: QuickJSValue,
    argc: i32,
    argv: *mut QuickJSValue,
) -> QuickJSValue {
    let runtime = JS_GetRuntime(ctx);

    // This gets called whenever e.g. "new NativeClass()" gets called in JS code. It's a C function so
    // we can't return a Result<> and use ? operators, instead we do some and_then chaining to get
    // things going:

    get_classid_storage(runtime)
        .and_then(|storage| get_class_id::<T>(ctx.into(), storage))
        .and_then(|class_id| {
            // examples in the QuickJS codebase tell us "using new_target to get the prototype is
            // necessary when the class is extended", so, OK then!
            // https://github.com/bellard/quickjs/blob/b5e62895c619d4ffc75c9d822c8d85f1ece77e5b/examples/point.c#L60
            let proto = JS_GetPropertyStr(ctx, new_target, STR_PROTOTYPE);

            // Then we create a new object with this prototype:
            let new_object = JS_NewObjectProtoClass(ctx, proto, class_id);

            // Ensure that we release the prototype or we'll get a memory leak
            proto.release(ctx.into());

            // Then finally return our new JS object
            Ok(new_object)
        })
        // Might want to work out how to provide an actual useful error here:
        .unwrap_or(JS_EXCEPTION__)
}

pub(super) fn get_class_constructor<T: JSExportClass>(
    in_context: *mut QuickJSContext,
) -> EsperantoResult<QuickJSValue> {
    let runtime = unsafe { JS_GetRuntime(in_context) };
    let storage = get_classid_storage(runtime)?;
    let ctx: QuickJSContextPointer = in_context.into();

    // We only create these constructors once, reusing if we already have one. So let's
    // check if we already have one:

    match storage.get(&T::METADATA).and_then(|class_id| unsafe {
        // Haven't been able to replicate it but in theory QuickJS could garbage collect the prototype
        // at some point, so let's check here whether it actually still exists. If not we'll just create
        // it again.
        let proto = JS_GetClassProto(in_context, *class_id);
        if JS_IsEqual__(proto, JS_UNDEFINED__) != 1 {
            return None;
        }
        Some(proto)
    }) {
        Some(proto) => {
            // QuickJS actually makes it a little difficult to get a reference to the constructor
            // (there's JS_SetConstructor but no JS_GetConstructor!) so we have to manually grab
            // it by name:
            let constructor = unsafe { JS_GetPropertyStr(in_context, proto, STR_CONSTRUCTOR) };
            proto.release(ctx);
            return Ok(constructor);
        }
        None => {
            // Get the ID for the class we're looking to create:
            let proto_class_id = get_class_id::<T>(ctx, storage)?;

            // First we create our constructor function that calls the generic function
            // defined above here

            let constructor = unsafe {
                JS_NewCFunction2(
                    *ctx,
                    Some(custom_class_constructor::<T>),
                    T::METADATA.class_name as *const i8,
                    1,
                    JSCFunctionEnum_JS_CFUNC_constructor,
                    0,
                )
            };

            // Create an 'empty' object that we'll throw all of this onto:
            let proto = unsafe { JS_NewObject(*ctx) };

            unsafe { JS_SetConstructor(*ctx, constructor, proto) }
            unsafe { JS_SetClassProto(*ctx, proto_class_id, proto) };

            return Ok(constructor);
        }
    };
}

pub(super) unsafe extern "C" fn delete_stored_prototype<T: JSExportClass>(
    runtime: *mut quickjs_android_suitable_sys::JSRuntime,
    value: QuickJSValue,
) {
    println!("DELETE VALUE {:?}", value);

    // let storage = match get_prototype_storage(runtime) {
    //     Ok(store) => store,
    //     Err(err) => {
    //         panic!("Could not get prototype storage: {}", err)
    //     }
    // };

    // let stored_value = {
    //     storage
    //         .in_use_prototypes
    //         .iter()
    //         .find_map(|(key, iter_value)| match JS_IsEqual__(value, *iter_value) {
    //             1 => Some(*key),
    //             _ => None,
    //         })
    // };
    // // .find(|(_, iter_value)| JS_IsEqual__(value, **iter_value) == 1);

    // match stored_value {
    //     None => {
    //         println!("Tried to remove a prototype when it isn't stored. This should never happen.");
    //     }
    //     Some(key) => {
    //         storage.in_use_prototypes.remove(&key);
    //     },
    // };
}
