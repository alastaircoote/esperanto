use super::util::with_ptr;
use crate::JSCValue;
use esperanto_shared::traits::JSValue;
use std::{ffi::CString, os::raw::c_char};

#[no_mangle]
pub extern "C" fn jsvalue_as_string(val_ptr: *mut JSCValue) -> *const c_char {
    with_ptr(val_ptr, |val| {
        let rust_string = val.as_string().unwrap();
        let cstring = CString::new(rust_string).unwrap();
        let ptr = cstring.as_ptr();
        std::mem::forget(cstring);
        ptr
    })
}

#[no_mangle]
pub extern "C" fn jsvalue_as_number(val_ptr: *mut JSCValue) -> f64 {
    with_ptr(val_ptr, |val| val.as_number().unwrap())
}

#[no_mangle]
pub extern "C" fn jsvalue_call(val_ptr: *mut JSCValue) -> *const JSCValue {
    with_ptr(val_ptr, |val| {
        let result = val.call().unwrap();
        Box::into_raw(Box::new(result))
    })
}

#[no_mangle]
pub extern "C" fn jsvalue_free(val_ptr: &mut JSCValue) {
    unsafe { Box::from_raw(val_ptr) };
}
