use std::{any::TypeId, ffi::CString};

use javascriptcore_sys::{
    JSClassCreate, JSClassDefinition, JSClassRelease, JSObjectGetPrivate, JSObjectMake,
    JSValueProtect, JSValueUnprotect, OpaqueJSClass, OpaqueJSContext, OpaqueJSValue,
};

use crate::{shared::errors::JSExportError, EsperantoResult, JSExportClass};

use super::{
    jscoreexport::{
        call_as_func_extern, constructor_extern, finalize_instance, finalize_prototype,
    },
    jscoreruntime::JSCoreRuntimeInternal,
};

// This is what we store in the HashMap inside our runtime
#[derive(Eq, PartialEq, Hash, Debug, Clone)]
pub(crate) struct JSClassStorage {
    // The actual JSValue for the class prototype. It is deliberately *not* retained
    // in this storage so that the JS runtime is able to garbage collect it when it stops
    // being used. We specify a finalizer in the prototype class definition to remove
    // prototypes from storage once GCed.
    pub(crate) prototype: *mut OpaqueJSValue,

    // The class we defined for individual instances of our custom class. We use this whenever
    // we wrap a native object to be forwarded into a JS context.
    pub(crate) instance_class: *mut OpaqueJSClass,

    // The class that defines the prototype. We don't actually use this anywhere except immediately
    // after it gets created but it seemed like it was worth keeping around in case any of this
    // implementation changes in the future
    pub(crate) prototype_class: *mut OpaqueJSClass,
}

// When we return our JSClassStorage we wrap it in this struct that also contains a pointer
// to the context. This is so that we can make sure we're managing the lifecycle of the prototype
// correctly: it needs to be released after use, and we need the context to call JSValueUnprotect()
pub(crate) struct JSClassStorageWithContext {
    storage: JSClassStorage,
    context: *mut OpaqueJSContext,
}

impl JSClassStorageWithContext {
    fn new(storage: &JSClassStorage, context: *mut OpaqueJSContext) -> Self {
        // We need to make sure we retain the prototype to avoid the (extremely
        // unlikely) scenario where the prototype is garbage collected and finalized
        // before it can be used.
        unsafe { JSValueProtect(context, storage.prototype) }
        JSClassStorageWithContext {
            storage: storage.clone(),
            context,
        }
    }
}

impl Drop for JSClassStorageWithContext {
    fn drop(&mut self) {
        unsafe { JSValueUnprotect(self.context, self.storage.prototype) }
    }
}

impl std::ops::Deref for JSClassStorageWithContext {
    type Target = JSClassStorage;

    fn deref(&self) -> &Self::Target {
        &self.storage
    }
}

impl JSClassStorage {
    pub(super) fn get_or_create<'a, T: JSExportClass>(
        ctx: *mut OpaqueJSContext,
        runtime: &JSCoreRuntimeInternal,
    ) -> EsperantoResult<JSClassStorageWithContext> {
        let type_id = TypeId::of::<T>();

        let mut storage_mut_ref = runtime.class_storage.borrow_mut();

        if let Some(existing) = storage_mut_ref.get(&type_id) {
            return Ok(JSClassStorageWithContext::new(existing, ctx));
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

        let runtime_ref: *const JSCoreRuntimeInternal = runtime;

        // We store the pointer to the runtime in the prototype private data because we need it
        // in the finalizer.
        let prototype = unsafe { JSObjectMake(ctx, prototype_class, runtime_ref as _) };

        let storage = JSClassStorage {
            prototype,
            instance_class,
            prototype_class,
        };
        storage_mut_ref.insert(type_id, storage.clone());
        return Ok(JSClassStorageWithContext::new(&storage, ctx));
    }

    pub(super) fn remove<T: JSExportClass>(prototype: *mut OpaqueJSValue) -> EsperantoResult<()> {
        let private = unsafe { JSObjectGetPrivate(prototype) } as *const JSCoreRuntimeInternal;
        let mut storage = unsafe { private.as_ref() }
            .ok_or(JSExportError::UnexpectedBehaviour)?
            .class_storage
            .borrow_mut();
        let stored = storage
            .remove(&TypeId::of::<T>())
            .ok_or(JSExportError::UnexpectedBehaviour)?;

        unsafe { JSClassRelease(stored.instance_class) };
        unsafe { JSClassRelease(stored.prototype_class) };
        Ok(())
    }
}
