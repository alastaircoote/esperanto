use std::{
    convert::{TryFrom, TryInto},
    ffi::{CStr, CString},
    os::raw::c_char,
};

use javascriptcore_sys::{
    JSStringCreateWithUTF8CString, JSStringGetMaximumUTF8CStringSize, JSStringGetUTF8CString,
    JSStringIsEqualToUTF8CString, JSValueToStringCopy, OpaqueJSString,
};

use crate::{shared::external_api::conversion_error::JSConversionError, EsperantoError};

use super::jscore_value::JSValue;

pub(crate) struct JSCoreString {
    pub raw_ref: *mut OpaqueJSString,
}

impl PartialEq<JSCoreString> for CString {
    fn eq(&self, other: &JSCoreString) -> bool {
        unsafe { JSStringIsEqualToUTF8CString(other.raw_ref, self.as_ptr()) }
    }
}

impl TryFrom<&str> for JSCoreString {
    type Error = JSConversionError;
    fn try_from(str: &str) -> Result<Self, Self::Error> {
        let c_string =
            CString::new(str).map_err(|_| JSConversionError::FailedToConvertStringToJS)?;
        Ok((&c_string).into())
    }
}

impl From<&CString> for JSCoreString {
    fn from(c_string: &CString) -> Self {
        let raw_ref = unsafe { JSStringCreateWithUTF8CString(c_string.as_ptr()) };
        JSCoreString { raw_ref }
    }
}

impl TryFrom<&String> for JSCoreString {
    type Error = JSConversionError;

    fn try_from(value: &String) -> Result<Self, Self::Error> {
        JSCoreString::try_from(value.as_str())
    }
}

impl From<*mut OpaqueJSString> for JSCoreString {
    fn from(raw_ref: *mut OpaqueJSString) -> Self {
        JSCoreString { raw_ref }
    }
}

impl TryFrom<&JSCoreString> for String {
    type Error = EsperantoError;

    fn try_from(jsc: &JSCoreString) -> Result<Self, Self::Error> {
        let s: &str = jsc.try_into()?;
        Ok(s.to_string())
    }
}

impl TryFrom<&JSCoreString> for &str {
    type Error = EsperantoError;

    fn try_from(value: &JSCoreString) -> Result<Self, Self::Error> {
        let len = unsafe { JSStringGetMaximumUTF8CStringSize(value.raw_ref) };
        let mut buffer: Vec<c_char> = vec![0; len];
        unsafe { JSStringGetUTF8CString(value.raw_ref, buffer.as_mut_ptr(), len) };

        let str = unsafe { CStr::from_ptr(buffer.as_ptr()) };
        Ok(str
            .to_str()
            .map_err(|_| JSConversionError::FailedToConvertStringToNative)?)
    }
}

impl TryFrom<&JSValue<'_>> for JSCoreString {
    type Error = EsperantoError;

    fn try_from(value: &JSValue) -> Result<Self, Self::Error> {
        let str = check_jscore_exception!(&value.context, exception => {
            unsafe { JSValueToStringCopy(value.context.into(), value.raw_ref.as_const(), exception)}
        })?;
        Ok(str.into())
    }
}
