use std::{
    any::TypeId, collections::HashMap, convert::TryInto, ffi::CString, hash::Hash, slice,
    sync::RwLock,
};

use javascriptcore_sys::{
    JSClassCreate, JSClassDefinition, JSClassRelease, JSContextGetGlobalContext,
    JSContextGetGlobalObject, JSContextGetGroup, JSContextGroupRetain, JSGlobalContextRetain,
    JSObjectGetPrivate, JSObjectMake, JSObjectSetPrivate, JSValueMakeNull, JSValueMakeUndefined,
    JSValueProtect, JSValueUnprotect, OpaqueJSClass, OpaqueJSContext, OpaqueJSContextGroup,
    OpaqueJSValue,
};
// use javascriptcore_sys::JSClassDefinition;
use lazy_static::lazy_static;

use crate::{
    export::JSClassFunction,
    jscore::jscorevaluepointer::JSCoreValuePointer,
    shared::{errors::JSExportError, value::JSValueInternal},
    EsperantoError, EsperantoResult, JSContext, JSExportClass, JSRuntime, JSValue, Retain,
};

/// We can't implement sync for OpaqueJSClass because it's foreign.
/// Instead we use this struct as a wrapper
#[derive(Eq, PartialEq, Hash, Copy, Clone)]
struct JSContextWrapper(*mut OpaqueJSContext);
#[derive(Eq, PartialEq, Hash, Copy, Clone)]
pub(crate) struct JSClassStorage {
    pub(crate) prototype: *mut OpaqueJSValue,
    pub(crate) instance_class: *mut OpaqueJSClass,
}

/// JavaScriptCore docs state that the API is thread-safe:
/// https://developer.apple.com/documentation/javascriptcore/jsvirtualmachine#
/// so we're OK to impl Sync and Send here. That said we're not using any
/// multi-threading right now anyway so this is kind of moot.
unsafe impl Sync for JSContextWrapper {}
unsafe impl Send for JSContextWrapper {}
unsafe impl Sync for JSClassStorage {}
unsafe impl Send for JSClassStorage {}

type ClassMap = RwLock<HashMap<(TypeId, JSContextWrapper), JSClassStorage>>;

lazy_static! {
    static ref JS_CLASSES: ClassMap = RwLock::new(HashMap::new());
}

unsafe extern "C" fn finalize_instance<T: JSExportClass>(val: *mut OpaqueJSValue) {
    let ptr = JSObjectGetPrivate(val) as *mut T;
    let boxed = Box::from_raw(ptr);
    println!("DID YOU FINALIIIIZEEEE")
    // gets dropped here.
}

unsafe extern "C" fn finalize_prototype<T: JSExportClass>(val: *mut OpaqueJSValue) {
    println!("finalize proto!")
}

pub(super) fn get_class_for<T: JSExportClass>(
    in_context: *mut OpaqueJSContext,
) -> EsperantoResult<JSClassStorage> {
    let type_id = TypeId::of::<T>();
    let wrapped_context = JSContextWrapper(in_context);

    if let Some(already_created) = JS_CLASSES
        .try_read()
        .map_err(|_| JSExportError::UnexpectedBehaviour)?
        .get(&(type_id, wrapped_context))
    {
        return Ok(*already_created);
    }

    let name_as_c_string = CString::new(T::CLASS_NAME)?;

    let mut prototype_def = JSClassDefinition::default();
    prototype_def.className = name_as_c_string.as_ptr();

    let mut instance_def = prototype_def;

    if T::CALL_AS_CONSTRUCTOR.is_some() {
        prototype_def.callAsConstructor = Some(constructor_extern::<T>);
    }
    if T::CALL_AS_FUNCTION.is_some() {
        prototype_def.callAsFunction = Some(call_as_func_extern::<T>);
    }

    instance_def.finalize = Some(finalize_instance::<T>);
    prototype_def.finalize = Some(finalize_prototype::<T>);

    let prototype_class = unsafe { JSClassCreate(&prototype_def) };
    let instance_class = unsafe { JSClassCreate(&instance_def) };

    let prototype = unsafe { JSObjectMake(in_context, prototype_class, std::ptr::null_mut()) };
    unsafe { JSClassRelease(prototype_class) };
    // hmm
    // unsafe { JSValueProtect(in_context, prototype) }

    let storage = JSClassStorage {
        prototype,
        instance_class,
    };

    let mut writable = JS_CLASSES
        .try_write()
        .map_err(|_| JSExportError::UnexpectedBehaviour)?;

    writable.insert((type_id, wrapped_context), storage.clone());

    Ok(storage)
}

fn get_wrapped_context<'c>(ctx: &'c *const OpaqueJSContext) -> JSContext<'c> {
    let group = unsafe { JSContextGroupRetain(JSContextGetGroup(*ctx)) };
    let runtime = JSRuntime::from_raw(group);
    let global_ctx = unsafe { JSGlobalContextRetain(JSContextGetGlobalContext(*ctx)) };
    JSContext::from_raw_storing_runtime(global_ctx, runtime)
}

unsafe fn execute_function<T: JSExportClass, ReturnType>(
    ctx: *const OpaqueJSContext,
    argc: usize,
    argv: *const *const OpaqueJSValue,
    exception: *mut *const OpaqueJSValue,
    function: &Option<JSClassFunction>,
    transform: fn(&JSValue, *const OpaqueJSContext) -> EsperantoResult<ReturnType>,
    empty_result: fn(*const OpaqueJSContext) -> ReturnType,
) -> ReturnType {
    let context = get_wrapped_context(&ctx);
    let result: EsperantoResult<Retain<JSValue>>;

    if let Some(function) = function {
        let args: Vec<JSValue> = slice::from_raw_parts(argv, argc)
            .iter()
            .map(|raw| JSValue::wrap_internal(JSCoreValuePointer::Value(*raw), &context))
            .collect();
        let func_result = (function.func)(&args, &context);
        result = func_result
    } else {
        result = Err(JSExportError::ConstructorCalledOnNonConstructableClass(
            T::CLASS_NAME.to_string(),
        )
        .into())
    }

    let into_raw = result.and_then(|val| transform(&val, ctx));
    match into_raw {
        Ok(result) => result,
        Err(error) => {
            let error_val = JSValue::try_new_from(error, &context).unwrap();
            exception.write(error_val.internal.as_value());
            // return an empty object? never gets used because we're storing an exception, not sure what else to
            // return really
            return empty_result(ctx);
        }
    }
}

unsafe extern "C" fn call_as_func_extern<T: JSExportClass>(
    ctx: *const OpaqueJSContext,
    function: *mut OpaqueJSValue,
    this_object: *mut OpaqueJSValue,
    argc: usize,
    argv: *const *const OpaqueJSValue,
    exception: *mut *const OpaqueJSValue,
) -> *const OpaqueJSValue {
    execute_function::<T, *const OpaqueJSValue>(
        ctx,
        argc,
        argv,
        exception,
        &T::CALL_AS_FUNCTION,
        |val, _| Ok(val.internal.as_value()),
        |ctx| JSValueMakeUndefined(ctx),
    )
}

pub(super) unsafe extern "C" fn constructor_extern<T: JSExportClass>(
    ctx: *const OpaqueJSContext,
    constructor_val: *mut OpaqueJSValue,
    argc: usize,
    argv: *const *const OpaqueJSValue,
    exception: *mut *const OpaqueJSValue,
) -> *mut OpaqueJSValue {
    execute_function::<T, *mut OpaqueJSValue>(
        ctx,
        argc,
        argv,
        exception,
        &T::CALL_AS_CONSTRUCTOR,
        |val, ctx| val.internal.try_as_object(ctx),
        |ctx| {
            // return an empty object? never gets used because we're storing an exception, not sure what else to
            // return really
            return JSObjectMake(ctx, std::ptr::null_mut(), std::ptr::null_mut());
        },
    )
    // let context = get_wrapped_context(&ctx);
    // let constructor_result: EsperantoResult<Retain<JSValue>>;

    // if let Some(constructor) = T::CALL_AS_CONSTRUCTOR {
    //     let args: Vec<JSValue> = slice::from_raw_parts(argv, argc)
    //         .iter()
    //         .map(|raw| JSValue::wrap_internal(JSCoreValuePointer::Value(*raw), &context))
    //         .collect();
    //     let func_result = (constructor.func)(&args, &context);
    //     constructor_result = func_result
    // } else {
    //     constructor_result = Err(JSExportError::ConstructorCalledOnNonConstructableClass(
    //         T::CLASS_NAME.to_string(),
    //     )
    //     .into())
    // }

    // let into_raw = constructor_result.and_then(|val| val.internal.try_as_object(context.internal));
    // let raw = match into_raw {
    //     Ok(result) => result,
    //     Err(error) => {
    //         let error_val = JSValue::try_new_from(error, &context).unwrap();
    //         exception.write(error_val.internal.as_value());
    //         // return an empty object? never gets used because we're storing an exception, not sure what else to
    //         // return really
    //         return JSObjectMake(ctx, std::ptr::null_mut(), std::ptr::null_mut());
    //     }
    // };

    // raw
}
