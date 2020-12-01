use std::{cell::RefCell, rc::Rc};

use javascriptcore_sys::{
    JSClassCreate, JSClassDefinition, JSObjectGetPrivate, JSObjectMake, JSObjectRef,
    JSObjectSetPrivate,
};

use crate::{JSCGlobalContext, JSCValue};

pub struct FinalizerChecker(Rc<RefCell<bool>>);

pub fn make_finalize_checker<'a>(
    in_context: &Rc<JSCGlobalContext>,
) -> (FinalizerChecker, JSCValue<'a>) {
    let mut def = JSClassDefinition::default();
    def.finalize = Some(finalize);
    let class_def = unsafe { JSClassCreate(&def) };
    let obj = unsafe { JSObjectMake(in_context.raw_ref, class_def, std::ptr::null_mut()) };

    let finalizer_tracker = Rc::new(RefCell::new(false));

    // let finalizer_check = Rc::new(FinalizerChecker {
    //     is_finalized: RefCell::new(false),
    //     js_value: JSCValue::from_raw_object_ref(obj, in_context).unwrap(),
    // });

    let boxed = Box::new(finalizer_tracker.clone());
    unsafe { JSObjectSetPrivate(obj, Box::into_raw(boxed) as *mut std::ffi::c_void) };

    (
        FinalizerChecker(finalizer_tracker),
        JSCValue::from_raw_object_ref(obj, in_context).unwrap(),
    )
}

impl FinalizerChecker {
    pub fn is_finalized(&self) -> bool {
        self.0.borrow().clone()
    }
}

unsafe extern "C" fn finalize(val: *mut javascriptcore_sys::OpaqueJSValue) {
    let raw = JSObjectGetPrivate(val) as *mut Rc<RefCell<bool>>;
    let boxed = Box::from_raw(raw);
    boxed.replace(true);
}
