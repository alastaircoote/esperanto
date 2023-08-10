use std::{
    any::{Any, TypeId},
    borrow::BorrowMut,
    cell::RefCell,
    collections::HashMap,
    ffi::CString,
};

use javascriptcore_sys::{
    JSClassCreate, JSClassDefinition, JSClassRelease, JSContextGetGlobalObject, JSObjectGetPrivate,
    JSObjectMake, JSObjectSetPrivate, OpaqueJSClass, OpaqueJSValue,
};

use crate::{shared::errors::JSExportError, EsperantoResult, JSExportClass};

use super::{
    jscorecontext::JSCoreContextInternal,
    jscoreexport::{
        call_as_func_extern, constructor_extern, finalize_instance, finalize_prototype,
    },
};

#[derive(Eq, PartialEq, Hash, Copy, Clone, Debug)]
pub(crate) struct JSClassStorage {
    pub(crate) prototype: *mut OpaqueJSValue,
    pub(crate) instance_class: *mut OpaqueJSClass,
}

pub(super) type JSCoreClassStorage = HashMap<TypeId, JSClassStorage>;

pub(super) fn attach_storage(
    storage: *const RefCell<JSCoreClassStorage>,
    to_context: JSCoreContextInternal,
) {
    // store a reference to our runtime storage in the global object so we can easily grab it later
    let global_object = unsafe { JSContextGetGlobalObject(to_context) };
    unsafe { JSObjectSetPrivate(global_object, storage as _) };
}

pub(super) fn get_or_create_class_info<T: JSExportClass>(
    in_context: JSCoreContextInternal,
) -> EsperantoResult<JSClassStorage> {
    let type_id = TypeId::of::<T>();
    let global_object = unsafe { JSContextGetGlobalObject(in_context) };
    let storage_ptr =
        unsafe { JSObjectGetPrivate(global_object) } as *const RefCell<JSCoreClassStorage>;

    let mut storage_mut_ref = unsafe { storage_ptr.as_ref() }.unwrap().borrow_mut();

    if let Some(existing) = storage_mut_ref.get(&type_id) {
        return Ok(*existing);
    }

    // Otherwise we need to make a new definition. We actually need to create *two* definitions
    //
    // - the prototype definition
    // - the instance definition
    //
    // The prototype actually contains most of what we want: the functions that get called, the
    // properties, etc etc. The instance definition is really only needed to a) get the name right
    // and b) allow us to attach private data (JSC only lets you attach private data to objects
    // that use a custom class) and c) to finalize (and drop) instances
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

    // We store the pointer to the context in the prototype private data because we need it
    // in the finalizer.
    let prototype = unsafe { JSObjectMake(in_context, prototype_class, in_context as _) };

    // Now that we've created our prototype we can release the class: we only ever need one prototype per class
    // unsafe { JSClassRelease(prototype_class) };

    let storage = JSClassStorage {
        prototype,
        instance_class,
    };
    println!("proto: {:?} instance: {:?}", prototype, instance_class);
    storage_mut_ref.insert(type_id, storage);
    Ok(storage)
}
