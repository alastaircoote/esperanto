use crate::{JSCGlobalContext, JSCValue};
use esperanto_shared::traits::JSContext;
use std::{
    ffi::{CStr, CString},
    os::raw::c_char,
    rc::Rc,
};

#[no_mangle]
pub extern "C" fn context_new() -> *mut Box<Rc<JSCGlobalContext>> {
    let ctx = JSCGlobalContext::new().unwrap();
    Box::into_raw(Box::new(Box::new(ctx)))
}

#[no_mangle]
pub extern "C" fn context_evaluate(
    ctx_ptr: *mut Box<Rc<JSCGlobalContext>>,
    script_ptr: *const c_char,
) -> *mut JSCValue {
    let ctx = unsafe { Box::from_raw(ctx_ptr) };

    let script = unsafe { CStr::from_ptr(script_ptr) };
    let result = ctx.evaluate(script.to_str().unwrap()).unwrap();
    Box::into_raw(ctx);
    std::mem::forget(script);
    Box::into_raw(Box::new(result))
}

#[no_mangle]
pub extern "C" fn context_free(ctx_ptr: *mut Box<Rc<JSCGlobalContext>>) {
    println!("Should drop?");
    unsafe { Box::from_raw(ctx_ptr) };
}
