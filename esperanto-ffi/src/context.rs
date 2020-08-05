use crate::{
    implementation::{Context, Value},
    util::with_ptr,
};
use esperanto_shared::traits::JSContext;
use std::{os::raw::c_char, rc::Rc};

#[no_mangle]
pub extern "C" fn jscontext_new() -> *mut Rc<Context> {
    let ctx = Context::new().unwrap();
    Box::into_raw(Box::new(ctx))
}

#[no_mangle]
pub extern "C" fn jscontext_evaluate(
    ctx_ptr: *mut Rc<Context>,
    script_ptr: *const c_char,
) -> *mut Value {
    let result = with_ptr(ctx_ptr, |ctx| ctx.evaluate_cstring(script_ptr).unwrap());
    Box::into_raw(Box::new(result))
}

#[no_mangle]
pub extern "C" fn jscontext_free(ctx_ptr: *mut Rc<Context>) {
    unsafe { Box::from_raw(ctx_ptr) };
}
