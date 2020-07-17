use javascriptcore_sys::{JSGlobalContextRelease, OpaqueJSContext};

#[derive(Debug)]
pub struct JSCSharedContextRef {
    pub(crate) jsc_ref: *mut OpaqueJSContext,
}

impl Drop for JSCSharedContextRef {
    fn drop(&mut self) {
        unsafe { JSGlobalContextRelease(self.jsc_ref) }
    }
}
