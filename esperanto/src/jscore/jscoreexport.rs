use std::{any::TypeId, collections::HashMap, ffi::CString, hash::Hash, slice, sync::RwLock};

use javascriptcore_sys::{
    JSClassCreate, JSClassDefinition, JSContextGetGroup, JSObjectMake, JSValueMakeUndefined,
    OpaqueJSClass, OpaqueJSContext, OpaqueJSContextGroup, OpaqueJSValue,
};
// use javascriptcore_sys::JSClassDefinition;
use lazy_static::lazy_static;

use crate::{
    jscore::jscorevaluepointer::JSCoreValuePointer, shared::errors::JSExportError, EsperantoError,
    EsperantoResult, JSContext, JSExportClass, JSRuntime, JSValue, Retain,
};

/// We can't implement sync for OpaqueJSClass because it's foreign.
/// Instead we use this struct as a wrapper
#[derive(Eq, PartialEq, Hash, Copy, Clone)]
struct JSClassRefWrapper(*mut OpaqueJSClass);
#[derive(Eq, PartialEq, Hash, Copy, Clone)]
struct JSContextGroupWrapper(*const OpaqueJSContextGroup);

/// JavaScriptCore docs state that the API is thread-safe:
/// https://developer.apple.com/documentation/javascriptcore/jsvirtualmachine#
/// so we're OK to impl Sync and Send here. That said we're not using any
/// multi-threading right now anyway so this is kind of moot.
unsafe impl Sync for JSClassRefWrapper {}
unsafe impl Send for JSClassRefWrapper {}
unsafe impl Sync for JSContextGroupWrapper {}
unsafe impl Send for JSContextGroupWrapper {}

type TypeClassMap = HashMap<TypeId, JSClassRefWrapper>;
type RuntimeClassMap = RwLock<HashMap<JSContextGroupWrapper, TypeClassMap>>;

lazy_static! {
    static ref JS_CLASSES: RuntimeClassMap = RwLock::new(HashMap::new());
}

// JSClass handling in JSC is a little confusing. We're given JSClassRetain() and JSClassRelease()
// much like we have with JSValues but as best I can find there's no finalizer for the class itself,
// which means we never actually know when an OpaqueJSClass has been destroyed.
//
// For simplicity's sake we generate separate JSClasses for each runtime and release them when the
// runtime is dropped.

pub(super) fn get_jsclass_for<T: JSExportClass>(
    in_context_group: *const OpaqueJSContextGroup,
) -> EsperantoResult<*mut OpaqueJSClass> {
    let group_wrapper = JSContextGroupWrapper(in_context_group);
    let type_id = TypeId::of::<T>();

    if let Some(map) = JS_CLASSES
        .try_read()
        .map_err(|_| JSExportError::UnexpectedBehaviour)?
        .get(&group_wrapper)
    {
        if let Some(existing) = map.get(&type_id) {
            return Ok(existing.0);
        }
    }

    // No current class so we need to make it
    let class = make_jsclass_for::<T>()?;

    let mut writable = JS_CLASSES
        .try_write()
        .map_err(|_| JSExportError::UnexpectedBehaviour)?;

    // Then either get the existing map or make a new one
    let map = match writable.get_mut(&group_wrapper) {
        Some(map) => map,
        None => {
            let new_map = HashMap::new();
            writable.insert(group_wrapper, new_map);
            // we know we're safe to call unwrap() here because we literally just added it
            writable.get_mut(&group_wrapper).unwrap()
        }
    };

    map.insert(type_id, JSClassRefWrapper(class));
    return Ok(class);
}

fn make_jsclass_for<T: JSExportClass>() -> EsperantoResult<*mut OpaqueJSClass> {
    let name_as_c_string = CString::new(T::CLASS_NAME)?;
    let def = JSClassDefinition {
        version: 0,
        attributes: 1 << 1,
        className: name_as_c_string.as_ptr(),
        parentClass: std::ptr::null_mut(),
        staticValues: std::ptr::null_mut(),
        staticFunctions: std::ptr::null_mut(),
        initialize: None,
        finalize: None,
        hasProperty: None,
        getProperty: None,
        setProperty: None,
        deleteProperty: None,
        getPropertyNames: None,
        callAsFunction: Some(call_as_func_extern::<T>),
        callAsConstructor: Some(constructor_extern::<T>),
        // callAsConstructor: None,
        hasInstance: None,
        convertToType: None,
    };
    return Ok(unsafe { JSClassCreate(&def) });
}

unsafe extern "C" fn call_as_func_extern<T: JSExportClass>(
    ctx: *const OpaqueJSContext,
    function: *mut OpaqueJSValue,
    this_object: *mut OpaqueJSValue,
    argc: usize,
    argv: *const *const OpaqueJSValue,
    exception: *mut *const OpaqueJSValue,
) -> *const OpaqueJSValue {
    panic!("oh")
}

pub(super) unsafe extern "C" fn constructor_extern<T: JSExportClass>(
    ctx: *const OpaqueJSContext,
    constructor_val: *mut OpaqueJSValue,
    argc: usize,
    argv: *const *const OpaqueJSValue,
    exception: *mut *const OpaqueJSValue,
) -> *mut OpaqueJSValue {
    // panic!("oh2");
    let group = JSContextGetGroup(ctx);
    let runtime = JSRuntime::from_raw(group);
    let context = JSContext::from_raw(ctx.into(), &runtime);
    let result = constructor::<T>(&context, constructor_val, argc, argv);
    match result {
        Ok(result) => {
            return result;
        }
        Err(error) => {
            let error_val = JSValue::try_new_from(error, &context).unwrap();
            exception.write(error_val.internal.as_value());
            return JSObjectMake(ctx, std::ptr::null_mut(), std::ptr::null_mut());
        }
    }
}

fn constructor<'c, T: JSExportClass>(
    ctx: &'c JSContext<'c>,
    constructor: *mut OpaqueJSValue,
    argc: usize,
    argv: *const *const OpaqueJSValue,
) -> EsperantoResult<*mut OpaqueJSValue> {
    match T::CALL_AS_CONSTRUCTOR {
        None => Err(JSExportError::ConstructorCalledOnNonConstructableClass(
            T::CLASS_NAME.to_string(),
        )
        .into()),
        Some(constructor) => {
            let args: Vec<JSValue<'c>> = unsafe { slice::from_raw_parts(argv, argc) }
                .iter()
                .map(|raw| JSValue::wrap_internal(JSCoreValuePointer::Value(*raw), ctx))
                .collect();

            let func_result = (constructor.func)(&args, &ctx);
            return func_result?.internal.try_as_object(ctx.internal);
        }
    }
}
