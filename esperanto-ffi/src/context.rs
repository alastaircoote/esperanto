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

#[repr(C)]
pub struct CompiledCode {
    bytes: *mut u8,
    len: usize,
}

#[no_mangle]
pub extern "C" fn jscontext_compile_string(
    ctx_ptr: *mut Rc<Context>,
    script_ptr: *const c_char,
) -> *mut CompiledCode {
    let result = with_ptr(ctx_ptr, |ctx| {
        let mut result = Vec::from(ctx.compile_string(script_ptr).unwrap());
        let r = CompiledCode {
            bytes: result.as_mut_ptr(),
            len: result.len(),
        };
        // patch up later
        std::mem::forget(result);
        r
    });
    Box::into_raw(Box::new(result))
    // result
}

#[no_mangle]
pub extern "C" fn jscontext_eval_compiled(
    ctx_ptr: *mut Rc<Context>,
    compiled: *mut CompiledCode,
) -> *const Value {
    let result = with_ptr(ctx_ptr, |ctx| {
        let vec = unsafe { std::slice::from_raw_parts((*compiled).bytes, (*compiled).len) };

        let result = ctx.eval_compiled(vec).unwrap();
        result
    });
    Box::into_raw(Box::new(result))
    // result
}
