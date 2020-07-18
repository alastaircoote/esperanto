use esperanto_shared::errors::JSConversionError;
use javascriptcore_sys::{
    JSStringCreateWithUTF8CString, JSStringGetLength, JSStringGetUTF8CString, OpaqueJSString,
};
use std::{
    ffi::{CStr, CString},
    os::raw::c_char,
};

pub struct JSCString {
    pub(crate) jsc_ref: *mut OpaqueJSString,
}

impl JSCString {
    pub fn from_string(string: &str) -> Result<Self, JSConversionError> {
        let script_c_string =
            CString::new(string).map_err(|e| JSConversionError::ConversionToCStringFailed(e))?;

        let string_ptr = unsafe { JSStringCreateWithUTF8CString(script_c_string.as_ptr()) };

        Ok(JSCString {
            jsc_ref: string_ptr,
        })
    }

    pub fn from_ptr(ptr: *mut OpaqueJSString) -> Self {
        JSCString { jsc_ref: ptr }
    }

    pub fn to_string(&self) -> Result<String, JSConversionError> {
        let len = unsafe { JSStringGetLength(self.jsc_ref) };
        // len + 1 because JSStringGetLength doesn't include a terminating null byte
        let mut buffer: Vec<c_char> = vec![0; len + 1];
        unsafe { JSStringGetUTF8CString(self.jsc_ref, buffer.as_mut_ptr(), len) };

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
