use std::{
    convert::TryFrom,
    ffi::{CStr, CString},
    marker::PhantomData,
    ops::{Deref, DerefMut},
    os::raw::c_char,
};

use javascriptcore_sys::{
    JSStringCreateWithUTF8CString, JSStringGetMaximumUTF8CStringSize, JSStringGetUTF8CString,
    JSStringRelease, OpaqueJSString,
};

use crate::shared::{as_ptr::AsRawMutPtr, errors::ConversionError};

/// JavaScriptCore has special handling for strings: all incoming and outgoing string values
/// have to come through OpaqueJSString before you can use them. JSCoreString wraps those
/// pointers to handle release/retain and conversion to Rust strings.
pub struct JSCoreString<'a> {
    pub(crate) raw: &'a mut OpaqueJSString,
}

impl<'a> JSCoreString<'a> {
    pub fn from_retained_ptr(ptr: *mut OpaqueJSString) -> Self {
        JSCoreString {
            raw: unsafe { ptr.as_mut().unwrap() },
        }
    }
}

impl AsRawMutPtr<OpaqueJSString> for JSCoreString<'_> {
    fn as_mut_raw_ptr(&mut self) -> *mut OpaqueJSString {
        self.raw
    }
}

impl<'a> TryFrom<&str> for JSCoreString<'a> {
    type Error = ConversionError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let cstring =
            CString::new(value).map_err(|e| ConversionError::CouldNotConvertToJSString(e))?;
        Ok((cstring.as_c_str()).into())
    }
}

impl<'a> From<&CStr> for JSCoreString<'a> {
    fn from(value: &CStr) -> Self {
        let ptr = unsafe { JSStringCreateWithUTF8CString(value.as_ptr()) };
        JSCoreString::from_retained_ptr(ptr)
    }
}

impl<'a> From<&CString> for JSCoreString<'a> {
    fn from(value: &CString) -> Self {
        let ptr = unsafe { JSStringCreateWithUTF8CString(value.as_ptr()) };
        JSCoreString::from_retained_ptr(ptr)
    }
}

impl<'a> TryFrom<JSCoreString<'a>> for CString {
    type Error = ConversionError;

    fn try_from(mut value: JSCoreString) -> Result<Self, Self::Error> {
        let value_ptr = value.as_mut_raw_ptr();
        let len = unsafe { JSStringGetMaximumUTF8CStringSize(value_ptr) };
        let mut buffer: Vec<c_char> = vec![0; len];
        unsafe { JSStringGetUTF8CString(value_ptr, buffer.as_mut_ptr(), len) };

        let str = unsafe { CStr::from_ptr(buffer.as_ptr()) };

        Ok(str.to_owned())
    }
}

impl Drop for JSCoreString<'_> {
    fn drop(&mut self) {
        unsafe { JSStringRelease(self.raw) }
    }
}

#[cfg(test)]
mod test {
    use std::{convert::TryFrom, ffi::CString};

    use javascriptcore_sys::JSStringGetLength;

    use super::JSCoreString;
    use crate::shared::as_ptr::AsRawMutPtr;

    #[test]
    fn can_create_string() {
        let mut str = JSCoreString::try_from("hello").unwrap();
        let len = unsafe { JSStringGetLength(str.as_mut_raw_ptr()) };
        assert_eq!(len, 5);
        let string = CString::try_from(str).unwrap();
        assert_eq!(string.to_str().unwrap(), "hello");
    }
}
