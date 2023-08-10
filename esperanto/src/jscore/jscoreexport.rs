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
    jscore::jscorevaluepointer::JSCoreValuePointer,
    shared::{context::JSContextInternal, errors::JSExportError},
    EsperantoResult, JSContext, JSExportClass, JSRuntime, JSValue, Retain,
};

use super::JSContextInternalImpl;

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

lazy_static! {
    static ref JS_CLASSES: ClassMap = RwLock::new(HashMap::new());
}

pub(super) unsafe extern "C" fn finalize_instance<T: JSExportClass>(val: *mut OpaqueJSValue) {
    let ptr = JSObjectGetPrivate(val);
    JSExportPrivateData::<T>::drop(ptr);
}

pub(super) unsafe extern "C" fn finalize_prototype<T: JSExportClass>(val: *mut OpaqueJSValue) {
    // The prototype is no longer in use so we should remove it from our class
    // storage.

    let context = JSObjectGetPrivate(val) as *mut OpaqueJSContext;

    JS_CLASSES
        .write()
        .unwrap()
        .remove(&(TypeId::of::<T>(), JSContextWrapper(context)));
}

// pub(super) fn get_class_for<T: JSExportClass>(
//     in_context: *mut OpaqueJSContext,
// ) -> EsperantoResult<JSClassStorage> {
//     let type_id = TypeId::of::<T>();
//     let wrapped_context = JSContextWrapper(in_context);

//     if let Some(already_created) = JS_CLASSES
//         .read()
//         .map_err(|_| JSExportError::UnexpectedBehaviour)?
//         .get(&(type_id, wrapped_context))
//     {
//         // We've already created the class definition for this JSExportClass so let's
//         // just return it:
//         return Ok(*already_created);
//     }

//     // Otherwise we need to make a new definition. We actually need to create *two* definitions
//     //
//     // - the prototype definition
//     // - the instance definition
//     //
//     // The prototype actually contains most of what we want: the functions that get called, the
//     // properties, etc etc. The instance definition is really only needed to a) get the name right
//     // and b) allow us to attach private data (JSC only lets you attach private data to objects
//     // that use a custom class) and c) to finalize (and drop) instances
//     let name_as_c_string = CString::new(T::CLASS_NAME)?;

//     let mut prototype_def = JSClassDefinition::default();
//     prototype_def.className = name_as_c_string.as_ptr();

//     let mut instance_def = prototype_def;

//     if T::CALL_AS_CONSTRUCTOR.is_some() {
//         prototype_def.callAsConstructor = Some(constructor_extern::<T>);
//     }
//     if T::CALL_AS_FUNCTION.is_some() {
//         prototype_def.callAsFunction = Some(call_as_func_extern::<T>);
//     }

//     instance_def.finalize = Some(finalize_instance::<T>);
//     prototype_def.finalize = Some(finalize_prototype::<T>);

//     let prototype_class = unsafe { JSClassCreate(&prototype_def) };
//     let instance_class = unsafe { JSClassCreate(&instance_def) };

//     // We store the pointer to the context in the prototype private data because we need it
//     // in the finalizer.
//     let prototype = unsafe { JSObjectMake(in_context, prototype_class, in_context as _) };

//     // Now that we've created our prototype we can release the class: we only ever need one prototype per class
//     unsafe { JSClassRelease(prototype_class) };

//     let storage = JSClassStorage {
//         prototype,
//         instance_class,
//     };

//     let mut writable = JS_CLASSES
//         .write()
//         .map_err(|_| JSExportError::UnexpectedBehaviour)?;

//     writable.insert((type_id, wrapped_context), storage.clone());

//     Ok(storage)
// }

fn get_wrapped_context<'c>(ctx: &'c *mut OpaqueJSContext) -> JSContext<'c> {
    let ctx = JSCoreContextInternal::from(*ctx);
    let runtime = JSRuntime::from_raw(ctx.get_runtime(), false);
    JSContext::from_raw_storing_runtime(ctx, runtime)
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
    let global_context = unsafe { JSContextGetGlobalContext(ctx) };
    let context = get_wrapped_context(&global_context);
    let result: EsperantoResult<Retain<JSValue>>;

    if let Some(function) = function {
        let args: Vec<JSValue> = slice::from_raw_parts(argv, argc)
            .iter()
            .map(|raw| JSValue::wrap_internal(JSCoreValuePointer::Value(*raw), &context))
            .collect();
        let func_result = (function.func)(&args, &context);
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

pub(super) unsafe extern "C" fn call_as_func_extern<T: JSExportClass>(
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

pub(super) unsafe extern "C" fn constructor_extern<T: JSExportClass>(
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
