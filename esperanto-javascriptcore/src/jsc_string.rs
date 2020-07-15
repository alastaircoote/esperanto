use esperanto_shared::errors::JSEnvError;
use javascriptcore_sys::{JSStringCreateWithUTF8CString, OpaqueJSString};
use std::ffi::CString;

pub struct JSCString {
    pub(crate) jsc_ref: *mut OpaqueJSString,
}

impl JSCString {
    pub fn from_string(string: &str) -> Result<Self, JSEnvError> {
        let script_c_string = CString::new(string).map_err(|_| JSEnvError::TextEncodingFailed)?;

        let string_ptr = unsafe { JSStringCreateWithUTF8CString(script_c_string.as_ptr()) };

        Ok(JSCString {
            jsc_ref: string_ptr,
        })
    }
}
