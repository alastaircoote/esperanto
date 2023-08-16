use std::{any::TypeId, cell::RefCell, collections::HashMap, ops::Deref};

use javascriptcore_sys::{JSContextGroupCreate, JSContextGroupRelease, OpaqueJSContextGroup};

use crate::shared::runtime::{JSRuntimeError, JSRuntimeImplementation};

use super::jscore_class_storage::JSClassStorage;

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct JSCoreRuntimeInternal {
    pub(super) raw: *const OpaqueJSContextGroup,
    pub(super) class_storage: RefCell<HashMap<TypeId, JSClassStorage>>,
}

// pub type JSCoreRuntimeInternal = *const OpaqueJSContextGroup;

impl JSRuntimeImplementation for JSCoreRuntimeInternal {
    fn new() -> Result<Self, JSRuntimeError> {
        let raw = unsafe { JSContextGroupCreate() };
        if raw.is_null() {
            return Err(JSRuntimeError::CouldNotCreateRuntime);
        }

        Ok(JSCoreRuntimeInternal {
            raw,
            class_storage: RefCell::new(HashMap::new()),
        })
    }

    fn release(&mut self) {
        unsafe { JSContextGroupRelease(self.raw) }
        #[cfg(debug_assertions)]
        {
            assert_eq!(self.class_storage.borrow().len(), 0)
        }
    }
}

impl Deref for JSCoreRuntimeInternal {
    type Target = *const OpaqueJSContextGroup;

    fn deref(&self) -> &Self::Target {
        &self.raw
    }
}
