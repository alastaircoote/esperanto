use std::{ffi::CStr, os::raw::c_char};

#[no_mangle]
pub extern "C" fn free_string(value_ptr: *const c_char) {
    unsafe { CStr::from_ptr(value_ptr) };
}
