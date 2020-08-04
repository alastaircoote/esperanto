use super::util::with_ptr;
use crate::{JSCGlobalContext, JSCValue};
use esperanto_shared::traits::JSContext;
use std::{
    ffi::{CStr, CString},
    os::raw::c_char,
    rc::Rc,
};

#[no_mangle]
pub extern "C" fn context_new() -> *mut Rc<JSCGlobalContext> {
    let ctx = JSCGlobalContext::new().unwrap();
    Box::into_raw(Box::new(ctx))
}

#[no_mangle]
pub extern "C" fn context_evaluate(
    ctx_ptr: *mut Rc<JSCGlobalContext>,
    script_ptr: *const c_char,
) -> *mut JSCValue {
    let result = with_ptr(ctx_ptr, |ctx| ctx.evaluate_cstring(script_ptr).unwrap());
    Box::into_raw(Box::new(result))
}

#[no_mangle]
pub extern "C" fn context_free(ctx_ptr: *mut Rc<JSCGlobalContext>) {
    unsafe { Box::from_raw(ctx_ptr) };
}
