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
    pub(crate) qjs_ref: *mut QJSRuntimeRef,
}

unsafe extern "C" fn finalize(
    runtime: *mut libquickjs_sys::JSRuntime,
    value: libquickjs_sys::JSValue,
) {
    println!("Finalizzze");
    let data = JS_GetOpaque(value, get_class_id(QJSClassType::ClosureContext));
    let boxx = Box::from_raw(data);
}

impl QJSRuntime {
    pub fn new() -> Self {
        ensure_class_ids_created();

        let qjs_ref = unsafe { JS_NewRuntime() };
        // unsafe {
        //     JS_NewClass(
        //         qjs_ref,
        //         get_class_id(QJSClassType::ClosureContext),
        //         &JSClassDef {
        //             class_name: std::ffi::CString::new("Woah").unwrap().as_ptr(),
        //             call: None,
        //             finalizer: Some(finalize),
        //             gc_mark: None,
        //             exotic: std::ptr::null_mut(),
        //         },
        //     );
        // }
        QJSRuntime { qjs_ref }
    }
}

impl Drop for QJSRuntime {
    fn drop(&mut self) {
        unsafe { JS_FreeRuntime(self.qjs_ref) };
    }
}
