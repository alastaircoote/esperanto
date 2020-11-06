use crate::{jsc_error::JSErrorFromJSC, jsc_value::JSCValue};
use esperanto_shared::errors::{JSConversionError, JSError};
use javascriptcore_sys::{
    JSStringCreateWithUTF8CString, JSStringGetLength, JSStringGetUTF8CString, JSStringRelease,
    JSStringRetain, JSValueRef, JSValueToStringCopy, OpaqueJSString,
};
use std::{
    ffi::{CStr, CString},
    os::raw::c_char,
};

pub struct JSCString {
    pub(crate) raw_ref: *mut OpaqueJSString,
}

impl Drop for JSCString {
    fn drop(&mut self) {
        unsafe { JSStringRelease(self.raw_ref) }
    }
}

impl JSCString {
    pub fn from_string(string: &str) -> Result<Self, JSConversionError> {
        let script_c_string =
            CString::new(string).map_err(|e| JSConversionError::ConversionToCStringFailed(e))?;

        Self::from_c_string(script_c_string.as_ptr())
    }

    pub fn from_c_string(c_string: *const c_char) -> Result<Self, JSConversionError> {
        let string_ptr = unsafe { JSStringCreateWithUTF8CString(c_string) };
        unsafe { JSStringRetain(string_ptr) };
        Ok(JSCString {
            raw_ref: string_ptr,
        })
    }

    pub fn from_js_value(val: &JSCValue) -> Result<Self, JSError> {
        let mut exception_ptr: JSValueRef = std::ptr::null_mut();
        let str_ptr = unsafe {
            JSValueToStringCopy(
                val.context.raw_ref,
                val.raw_ref.get_jsvalue(),
                &mut exception_ptr,
            )
        };

        JSError::check_jsc_value_ref(exception_ptr, &val.context)?;
        unsafe { JSStringRetain(str_ptr) };
        Ok(Self::from_ptr(str_ptr))
    }

    pub fn from_ptr(ptr: *mut OpaqueJSString) -> Self {
        JSCString { raw_ref: ptr }
    }

    pub fn to_string(&self) -> Result<String, JSConversionError> {
        let len = unsafe { JSStringGetLength(self.raw_ref) };
        // len + 1 because JSStringGetLength doesn't include a terminating null byte
        let mut buffer: Vec<c_char> = vec![0; len + 1];
        unsafe { JSStringGetUTF8CString(self.raw_ref, buffer.as_mut_ptr(), len) };

        let str = unsafe { CStr::from_ptr(buffer.as_ptr()) };
        Ok(str
            .to_str()
            .map_err(|e| JSConversionError::ConversionFromCStringFailed(e))?
            .to_string())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn it_creates_js_strings_successfully() {
        let s = JSCString::from_string("this is a test string").unwrap();
        let back_again = s.to_string().unwrap();
        assert_eq!(back_again, "this is a test string")
    }
}
