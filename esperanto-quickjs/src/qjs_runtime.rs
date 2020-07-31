use crate::qjs_classids::ensure_class_ids_created;
use libquickjs_sys::{JSRuntime as QJSRuntimeRef, JS_FreeRuntime, JS_NewRuntime};

#[derive(Debug)]
pub struct QJSRuntime {
    pub(crate) raw: *mut QJSRuntimeRef,
}

impl QJSRuntime {
    pub fn new() -> Self {
        ensure_class_ids_created();
        let raw = unsafe { JS_NewRuntime() };
        QJSRuntime { raw }
    }
}

impl Drop for QJSRuntime {
    fn drop(&mut self) {
        unsafe { JS_FreeRuntime(self.raw) };
    }
}
