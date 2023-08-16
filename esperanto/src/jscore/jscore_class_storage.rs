use std::{any::TypeId, ffi::CString};

use javascriptcore_sys::{
    JSClassCreate, JSClassDefinition, JSClassRelease, JSObjectGetPrivate, JSObjectMake,
    JSValueProtect, JSValueUnprotect, OpaqueJSClass, OpaqueJSContext, OpaqueJSValue,
};

use crate::{EsperantoResult, JSExportClass};

use super::{
    jscoreexport::{
        call_as_func_extern, constructor_extern, finalize_instance, finalize_prototype,
    },
    jscoreruntime::JSCoreRuntimeInternal,
};

#[derive(Eq, PartialEq, Hash, Debug, Clone)]
pub(crate) struct JSClassStorage {
    pub(crate) prototype: *mut OpaqueJSValue,
    pub(crate) instance_class: *mut OpaqueJSClass,
    pub(crate) prototype_class: *mut OpaqueJSClass,
}

pub(crate) struct JSClassStorageWithContext {
    storage: JSClassStorage,
    context: *mut OpaqueJSContext,
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
    // pub(super) fn get_prototype<'r: 'c, 'c>(
    //     &self,
    //     in_context: &'c JSContext<'r, 'c>,
    // ) -> JSValue<'r, 'c> {
    //     unsafe { JSValueProtect(in_context.implementation(), self.prototype) };
    //     JSValue::wrap_internal(JSCoreValuePointer::Value(self.prototype), in_context)
    // }

    pub(super) fn get<'a, T: JSExportClass>(
        // wrapped_ctx: &'a JSContext,
        ctx: *mut OpaqueJSContext,
        runtime: &JSCoreRuntimeInternal,
    ) -> EsperantoResult<JSClassStorageWithContext> {
        let type_id = TypeId::of::<T>();

        let mut storage_mut_ref = runtime.class_storage.borrow_mut();

        if let Some(existing) = storage_mut_ref.get(&type_id) {
            unsafe { JSValueProtect(ctx, existing.prototype) }
            return Ok(JSClassStorageWithContext {
                storage: existing.clone(),
                context: ctx,
            });
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
        return Ok(JSClassStorageWithContext {
            storage,
            context: ctx,
        });
    }

    pub(super) fn remove<T: JSExportClass>(prototype: *mut OpaqueJSValue) {
        let private = unsafe { JSObjectGetPrivate(prototype) } as *const JSCoreRuntimeInternal;
        let mut storage = unsafe { private.as_ref() }
            .expect("Could not get reference to runtime in finaliser")
            .class_storage
            .borrow_mut();
        let stored = storage
            .remove(&TypeId::of::<T>())
            .expect("Tried to finalise a class that isn't stored");
        unsafe { JSClassRelease(stored.instance_class) };
        unsafe { JSClassRelease(stored.prototype_class) };
    }
}
