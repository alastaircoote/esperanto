use std::{
    convert::TryFrom,
    ffi::{CStr, CString},
    os::raw::c_char,
};

use esperanto_engine_shared::{
    errors::{JSContextError, JSConversionError},
    traits::TryIntoJSValue,
};
use javascriptcore_sys::{
    JSStringCreateWithUTF8CString, JSStringGetMaximumUTF8CStringSize, JSStringGetUTF8CString,
    JSValueMakeString, JSValueToStringCopy, OpaqueJSString,
};

use crate::{JSCoreContext, JSCoreValue};

pub(crate) struct JSCoreString {
    pub raw_ref: *mut OpaqueJSString,
}

impl TryFrom<&str> for JSCoreString {
    type Error = JSConversionError;
    fn try_from(str: &str) -> Result<Self, Self::Error> {
        let c_string = CString::new(str).map_err(|_| JSConversionError::TextEncodingFailed)?;
        let raw_ref = unsafe { JSStringCreateWithUTF8CString(c_string.as_ptr()) };

        Ok(JSCoreString { raw_ref })
    }
}

impl From<*mut OpaqueJSString> for JSCoreString {
    fn from(raw_ref: *mut OpaqueJSString) -> Self {
        JSCoreString { raw_ref }
    }
}

impl TryFrom<JSCoreValue> for JSCoreString {
    type Error = JSConversionError;

    fn try_from(value: JSCoreValue) -> Result<Self, Self::Error> {
        let mut exception = std::ptr::null();
        let raw_ref =
            unsafe { JSValueToStringCopy(value.context.raw_ref, value.raw_ref, &mut exception) };
        value
            .context
            .check_exception(&exception)
            .map_err(|err| match err {
                JSContextError::JSErrorOccurred(err_str) => {
                    JSConversionError::JSErrorOccurred(err_str)
                }
                _ => JSConversionError::UnknownError(
                    "An unexpected JSContext error occurred".to_string(),
                ),
            })?;
        Ok(raw_ref.into())
    }
}

impl TryFrom<JSCoreString> for String {
    type Error = JSConversionError;

    fn try_from(jsc: JSCoreString) -> Result<Self, Self::Error> {
        let len = unsafe { JSStringGetMaximumUTF8CStringSize(jsc.raw_ref) };
        let mut buffer: Vec<c_char> = vec![0; len];
        unsafe { JSStringGetUTF8CString(jsc.raw_ref, buffer.as_mut_ptr(), len) };

        let str = unsafe { CStr::from_ptr(buffer.as_ptr()) };
        Ok(str
            .to_str()
            .map_err(|_| JSConversionError::TextDecodingFailed)?
            .to_string())
    }
}

impl TryIntoJSValue<JSCoreValue> for JSCoreString {
    fn try_into_jsvalue(self, context: &JSCoreContext) -> Result<JSCoreValue, JSConversionError> {
        let raw_ref = unsafe { JSValueMakeString(context.raw_ref, self.raw_ref) };
        Ok(JSCoreValue {
            raw_ref,
            context: context.clone(),
        })
    }
}
