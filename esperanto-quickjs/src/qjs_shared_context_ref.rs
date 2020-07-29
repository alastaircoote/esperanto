use crate::qjs_runtime::QJSRuntime;
use libquickjs_sys::{JSContext as QJSContextRef, JS_FreeContext};
use std::rc::Rc;

#[derive(Debug)]
pub struct SharedQJSContextRef {
    pub(crate) qjs_ref: *mut QJSContextRef,
    pub(crate) runtime_ref: Rc<QJSRuntime>,
}

// impl std::fmt::Debug for SharedQJSContextRef {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         f.debug_struct("QJSValue")
//             .field("runtime_ref", self.runtime_ref)
//             .field("qjs_ref", self.qjs_ref.ptr)
//             .finish()
//     }
// }

impl SharedQJSContextRef {
    pub fn new(qjs_ref: *mut QJSContextRef, runtime_ref: Rc<QJSRuntime>) -> Self {
        SharedQJSContextRef {
            qjs_ref,
            runtime_ref,
        }
    }
}

impl Drop for SharedQJSContextRef {
    fn drop(&mut self) {
        unsafe { JS_FreeContext(self.qjs_ref) }
    }
}
