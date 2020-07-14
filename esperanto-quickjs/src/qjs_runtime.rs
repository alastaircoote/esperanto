use qjs_sys::{JSRuntime as QJSRuntimeRef, JS_FreeRuntime, JS_NewRuntime};

pub struct QJSRuntime {
    pub(crate) qjs_ref: *mut QJSRuntimeRef,
}

impl QJSRuntime {
    pub fn new() -> Self {
        let qjs_ref = unsafe { JS_NewRuntime() };
        QJSRuntime { qjs_ref }
    }
}

impl Drop for QJSRuntime {
    fn drop(&mut self) {
        unsafe { JS_FreeRuntime(self.qjs_ref) };
    }
}
