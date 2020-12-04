use esperanto_engine_shared::errors::JSRuntimeError;
use esperanto_engine_shared::traits::JSRuntime;
use javascriptcore_sys::{
    JSContextGroupCreate, JSContextGroupRelease, JSContextGroupRetain,
    JSGlobalContextCreateInGroup, OpaqueJSContext, OpaqueJSContextGroup,
};

use crate::jscore_context::JSCoreContext;

#[derive(Debug)]
pub struct JSCoreRuntime {
    raw_ref: *const OpaqueJSContextGroup,
}

impl JSCoreRuntime {
    pub fn new_raw_context(&self) -> *mut OpaqueJSContext {
        unsafe { JSGlobalContextCreateInGroup(self.raw_ref, std::ptr::null_mut()) }
    }
}

impl Clone for JSCoreRuntime {
    fn clone(&self) -> Self {
        JSCoreRuntime {
            raw_ref: unsafe { JSContextGroupRetain(self.raw_ref) },
        }
    }
}

impl Drop for JSCoreRuntime {
    fn drop(&mut self) {
        unsafe { JSContextGroupRelease(self.raw_ref) };
    }
}

impl JSRuntime for JSCoreRuntime {
    type Context = JSCoreContext;
    fn new() -> Result<Self, JSRuntimeError> {
        let raw_ref = unsafe { JSContextGroupCreate() };
        if raw_ref.is_null() {
            return Err(JSRuntimeError::CouldNotCreateRuntime);
        }
        Ok(JSCoreRuntime { raw_ref })
    }
}
