use crate::{
    qjs_classids::{ensure_class_ids_created, get_class_id, QJSClassType},
    ref_count::free_value,
};
use libquickjs_sys::{
    JSClassDef, JSClassExoticMethods, JSClassID, JSRuntime as QJSRuntimeRef, JS_FreeRuntime,
    JS_GetOpaque, JS_NewClass, JS_NewClassID, JS_NewRuntime,
};

// struct JSClassIDs {
//     rust_function: JSClassID,
// }

// impl JSClassIDs {
//     fn new() -> Self {
//         let mut i: u32 = 0;
//         let rust_function_class_id = unsafe { JS_NewClassID(&mut i) };
//         JSClassIDs {
//             rust_function: rust_function_class_id,
//         }
//     }
// }

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
