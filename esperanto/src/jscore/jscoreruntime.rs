use javascriptcore_sys::{JSContextGroupCreate, JSContextGroupRelease, OpaqueJSContextGroup};

use crate::shared::runtime::{JSRuntimeError, JSRuntimeInternal};

pub type JSCoreRuntimeInternal = *const OpaqueJSContextGroup;

impl JSRuntimeInternal for JSCoreRuntimeInternal {
    fn new() -> Result<Self, JSRuntimeError> {
        let raw = unsafe { JSContextGroupCreate() };
        if raw.is_null() {
            return Err(JSRuntimeError::CouldNotCreateRuntime);
        }
        Ok(raw)
    }

    fn release(self) {
        unsafe { JSContextGroupRelease(self) }
    }
}
