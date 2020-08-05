use crate::util::with_ptr;

use crate::implementation::Value;
use esperanto_shared::traits::JSValue;
use std::{
    ffi::{CStr, CString},
    os::raw::c_char,
};

#[no_mangle]
pub extern "C" fn jsvalue_as_string(value_ptr: *mut Value) -> *const c_char {
    let cstring = with_ptr(value_ptr, |val| {
        let rust_str = val.as_string().unwrap();
        CString::new(rust_str).unwrap()
    });
    let cstring_ptr = cstring.as_ptr();
    std::mem::forget(cstring);
    cstring_ptr
}

#[no_mangle]
pub extern "C" fn jsvalue_free(value_ptr: *mut Value) {
    unsafe { Box::from_raw(value_ptr) };
}

#[no_mangle]
pub extern "C" fn jsvalue_call(value_ptr: *mut Value) -> *mut Value {
    with_ptr(value_ptr, |val| {
        let result = val.call().unwrap();
        Box::into_raw(Box::new(result))
    })
}

#[no_mangle]
pub extern "C" fn jsvalue_call_bound(value_ptr: *mut Value, bound_to: *mut Value) -> *mut Value {
    let result = with_ptr(value_ptr, |val| {
        with_ptr(bound_to, |b| val.call_bound(Vec::new(), b).unwrap())
    });
    Box::into_raw(Box::new(result))
}

#[no_mangle]
pub extern "C" fn jsvalue_get_property(value_ptr: *mut Value, name: *const c_char) -> *mut Value {
    let property_name = unsafe { CStr::from_ptr(name).to_str().unwrap() };
    with_ptr(value_ptr, |val| {
        let result = val.get_property(property_name).unwrap();
        Box::into_raw(Box::new(result))
    })
}

#[no_mangle]
pub extern "C" fn jsvalue_as_number(value_ptr: *mut Value) -> f64 {
    with_ptr(value_ptr, |val| val.as_number().unwrap())
}
