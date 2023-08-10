use std::{any::TypeId, cell::RefCell, collections::HashMap, ops::Deref};

use javascriptcore_sys::{JSContextGroupCreate, JSContextGroupRelease, OpaqueJSContextGroup};

use crate::shared::runtime::{JSRuntimeError, JSRuntimeInternal};

use super::jscore_class_storage::JSCoreClassStorage;

#[derive(Debug)]
pub(crate) enum JSCoreRuntimeStorage {
    Source(Box<RefCell<JSCoreClassStorage>>),
    Referenced(*const RefCell<JSCoreClassStorage>),
}

impl JSCoreRuntimeStorage {
    pub(super) fn as_ptr(&self) -> *const RefCell<JSCoreClassStorage> {
        match self {
            JSCoreRuntimeStorage::Source(refcell) => Box::as_ref(refcell),
            JSCoreRuntimeStorage::Referenced(r) => *r,
        }
    }
}

#[derive(Debug)]
pub(crate) struct JSCoreRuntimeInternal {
    pub(super) raw: *const OpaqueJSContextGroup,
    pub(super) class_storage: JSCoreRuntimeStorage,
}

// pub type JSCoreRuntimeInternal = *const OpaqueJSContextGroup;

impl JSRuntimeInternal for JSCoreRuntimeInternal {
    fn new() -> Result<Self, JSRuntimeError> {
        let raw = unsafe { JSContextGroupCreate() };
        if raw.is_null() {
            return Err(JSRuntimeError::CouldNotCreateRuntime);
        }

        Ok(JSCoreRuntimeInternal {
            raw,
            class_storage: JSCoreRuntimeStorage::Source(Box::new(RefCell::new(HashMap::new()))),
        })
    }

    fn release(&mut self) {
        unsafe { JSContextGroupRelease(self.raw) }
    }
}

impl Deref for JSCoreRuntimeInternal {
    type Target = *const OpaqueJSContextGroup;

    fn deref(&self) -> &Self::Target {
        &self.raw
    }
}
