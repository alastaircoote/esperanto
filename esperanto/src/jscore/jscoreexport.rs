use std::{any::TypeId, collections::HashMap, ffi::CString, hash::Hash, slice, sync::RwLock};

use javascriptcore_sys::{
    JSClassCreate, JSClassDefinition, JSClassRelease, JSContextGetGlobalContext, JSContextGetGroup,
    JSContextGroupRetain, JSGlobalContextRetain, JSObjectGetPrivate, JSObjectMake,
    JSValueMakeUndefined, OpaqueJSClass, OpaqueJSContext, OpaqueJSValue,
};
// use javascriptcore_sys::JSClassDefinition;
use lazy_static::lazy_static;

use crate::{
    export::{JSClassFunction, JSExportPrivateData},
    jscore::jscorecontext::JSCoreContextInternal,
    jscore::jscoreruntime::JSCoreRuntimeInternal,
    jscore::jscorevaluepointer::JSCoreValuePointer,
    shared::{context::JSContextImplementation, errors::JSExportError, value::JSValueInternal},
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
/// so we're OK to impl Sync and Send here.
unsafe impl Sync for JSContextWrapper {}
unsafe impl Send for JSContextWrapper {}
unsafe impl Sync for JSClassStorage {}
unsafe impl Send for JSClassStorage {}

type ClassMap = RwLock<HashMap<(TypeId, JSContextWrapper), JSClassStorage>>;

pub(super) unsafe extern "C" fn finalize_instance<T: JSExportClass>(val: *mut OpaqueJSValue) {
    println!("finalize instance");
    let ptr = JSObjectGetPrivate(val);
    JSExportPrivateData::<T>::drop(ptr);
}

pub(super) unsafe extern "C" fn finalize_prototype<T: JSExportClass>(val: *mut OpaqueJSValue) {
    // The prototype is no longer in use so we should remove it from our class
    // storage.
    let private = unsafe { JSObjectGetPrivate(val) } as *const JSCoreRuntimeInternal;
    let mut storage = private.as_ref().unwrap().class_storage.borrow_mut();
    // let ctx = JSContext::borrow_from_implementation(private).unwrap();
    // let mut storage = ctx.get_runtime().internal.class_storage.borrow_mut();
    storage.remove(&TypeId::of::<T>());
}

unsafe fn execute_function<'r: 'c, 'c, T: JSExportClass, ReturnType>(
    ctx: *const OpaqueJSContext,
    argc: usize,
    argv: *const *const OpaqueJSValue,
    exception: *mut *const OpaqueJSValue,
    function: &Option<JSClassFunction>,
    transform: fn(&JSValue<'r, 'c>, *const OpaqueJSContext) -> EsperantoResult<ReturnType>,
    empty_result: fn(*const OpaqueJSContext) -> ReturnType,
) -> ReturnType {
    let global_context = unsafe { JSContextGetGlobalContext(ctx) };

    let context = JSContext::borrow_from_implementation(global_context).unwrap();

    let result: EsperantoResult<Retain<JSValue>>;

    if let Some(function) = function {
        let args: Vec<JSValue> = slice::from_raw_parts(argv, argc)
            .iter()
            .map(|raw| JSValue::wrap_internal(JSCoreValuePointer::Value(*raw), &context))
            .collect();

        let arg_refs: Vec<&JSValue> = args.iter().map(|a| a).collect();

        let func_result = (function.func)(arg_refs.as_slice(), &context);
        result = func_result
    } else {
        result = Err(JSExportError::ConstructorCalledOnNonConstructableClass(T::CLASS_NAME).into())
    }

    result
        .and_then(|val| transform(&val, ctx))
        .unwrap_or_else(|error| {
            let error_val = JSValue::try_new_from(error, &context).unwrap();
            exception.write(error_val.internal.as_value());
            return empty_result(ctx);
        })
}

pub(super) unsafe extern "C" fn call_as_func_extern<'r: 'c, 'c, T: JSExportClass>(
    ctx: *const OpaqueJSContext,
    _function: *mut OpaqueJSValue,
    _this_object: *mut OpaqueJSValue,
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

pub(super) unsafe extern "C" fn constructor_extern<'r: 'c, 'c, T: JSExportClass>(
    ctx: *const OpaqueJSContext,
    _constructor_val: *mut OpaqueJSValue,
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
            // return really. Can't return undefined as it's a value, not an object
            return JSObjectMake(ctx, std::ptr::null_mut(), std::ptr::null_mut());
        },
    )
}
