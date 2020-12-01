use esperanto_engine_shared::traits::JSRuntime;
use esperanto_engine_shared::{
    errors::JSContextError, errors::JSRuntimeError, traits::RuntimeCreatesContext,
};
use javascriptcore_sys::{
    JSContextGroupCreate, JSGlobalContextCreate, JSGlobalContextCreateInGroup, OpaqueJSContext,
    OpaqueJSContextGroup,
};

use crate::jscore_context::JSCoreContext;

pub struct JSCoreRuntime {
    raw_ref: *const OpaqueJSContextGroup,
}

impl JSCoreRuntime {
    pub fn new_raw_context(&self) -> *mut OpaqueJSContext {
        unsafe { JSGlobalContextCreateInGroup(self.raw_ref, std::ptr::null_mut()) }
    }
}

impl<'runtime> JSRuntime<'runtime> for JSCoreRuntime {
    fn new() -> Result<Self, JSRuntimeError> {
        let raw_ref = unsafe { JSContextGroupCreate() };
        if raw_ref.is_null() {
            return Err(JSRuntimeError::CouldNotCreateRuntime);
        }
        Ok(JSCoreRuntime { raw_ref })
    }
}

impl<'runtime, 'context> RuntimeCreatesContext<'runtime, 'context> for JSCoreRuntime
where
    'runtime: 'context,
{
    type Context = JSCoreContext<'runtime, 'context>;

    fn create_context(&'runtime self) -> Result<Self::Context, JSContextError> {
        let raw_ref = unsafe { JSGlobalContextCreateInGroup(self.raw_ref, std::ptr::null_mut()) };
        let context = JSCoreContext::wrapping_raw_ref(raw_ref, self)?;
        Ok(context)
    }
}
