use javascriptcore_sys::{JSGlobalContextRelease, OpaqueJSContext};

pub struct JSCGlobalContext {
    pub(crate) jsc_ref: *mut OpaqueJSContext,
}

impl Drop for JSCGlobalContext {
    fn drop(&mut self) {
        unsafe { JSGlobalContextRelease(self.jsc_ref) }
    }
}
