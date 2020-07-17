use crate::{
    jsc_object::JSCObject, jsc_sharedcontextref::JSCSharedContextRef, jsc_string::JSCString,
};
use esperanto_shared::errors::{JSContextError, JSConversionError};
use esperanto_shared::traits::JSValue;
use javascriptcore_sys::{
    JSStringGetLength, JSStringGetUTF8CString, JSValueProtect, JSValueRef, JSValueToNumber,
    JSValueToStringCopy, JSValueUnprotect,
};
use std::convert::TryFrom;
use std::convert::TryInto;
use std::ffi::CStr;
use std::rc::Rc;

#[derive(Debug)]
pub struct JSCValue {
    pub(crate) jsc_ref: JSValueRef,
    pub(crate) context: Rc<JSCSharedContextRef>,
}

impl JSCValue {
    pub fn from_value_ref(v_ref: JSValueRef, in_context: &Rc<JSCSharedContextRef>) -> Self {
        unsafe { JSValueProtect(in_context.jsc_ref, v_ref) };
        JSCValue {
            jsc_ref: v_ref,
            context: in_context.clone(),
        }
    }
}

impl Drop for JSCValue {
    fn drop(&mut self) {
        unsafe {
            JSValueUnprotect(self.context.jsc_ref, self.jsc_ref);
        }
    }
}

impl TryFrom<JSCValue> for &str {
    type Error = JSConversionError;
    fn try_from(value: JSCValue) -> Result<Self, Self::Error> {
        unsafe {
            let string_ref =
                JSValueToStringCopy(value.context.jsc_ref, value.jsc_ref, std::ptr::null_mut());
            let string_length = JSStringGetLength(string_ref);

            // If we're on a 32 bit archiecture this could theoretically get too big. It really,
            // really shouldn't ever happen though.

            let string_length_usize: usize = string_length
                .try_into()
                .map_err(|_| JSConversionError::StringWasTooLong)?;

            let mut bytes: Vec<i8> = vec![0; string_length_usize];

            JSStringGetUTF8CString(string_ref, bytes.as_mut_ptr(), string_length);

            let cstr = CStr::from_ptr(bytes.as_ptr());
            cstr.to_str()
                .map_err(|_| JSConversionError::CouldNotConvertStringToSuitableFormat)
        }
    }
}

impl TryFrom<JSCValue> for String {
    type Error = JSConversionError;
    fn try_from(value: JSCValue) -> Result<Self, Self::Error> {
        let str: &str = value.try_into()?;
        Ok(str.to_string())
    }
}

impl TryFrom<JSCValue> for f64 {
    type Error = JSConversionError;
    fn try_from(value: JSCValue) -> Result<Self, Self::Error> {
        // As best I've been able to tell JSValueToNumber never actually creates an exception.
        // instead the returned value is NaN.

        // Will leave this here in the hopes we'll be able to find something that triggers an exception
        // in the future and test for it
        let exception: *mut JSValueRef = std::ptr::null_mut();

        let val = unsafe { JSValueToNumber(value.context.jsc_ref, value.jsc_ref, exception) };

        if val.is_nan() {
            Err(JSConversionError::ConversionFailed)
        } else {
            Ok(val)
        }
    }
}

impl JSValue for JSCValue {
    type ObjectType = JSCObject;
    fn to_string(&self) -> Result<String, JSConversionError> {
        let mut exception_ptr: JSValueRef = std::ptr::null_mut();
        let str_ptr =
            unsafe { JSValueToStringCopy(self.context.jsc_ref, self.jsc_ref, &mut exception_ptr) };

        let jsc_string = JSCString::from_ptr(str_ptr);
        jsc_string.to_string()
    }

    fn to_object(&self) -> Result<Self::ObjectType, JSContextError> {
        JSCObject::from_value_ref(self.jsc_ref, &self.context)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::jsc_context::JSCContext;
    use esperanto_shared::traits::JSContext;
    #[test]
    fn converts_to_number() {
        let runtime = JSCContext::new().unwrap();
        let value: JSCValue = runtime.evaluate("3.5").unwrap();
        let f: f64 = value.try_into().unwrap();
        assert_eq!(f, 3.5);
    }

    #[test]
    fn converts_to_string() {
        let runtime = JSCContext::new().unwrap();
        let value: JSCValue = runtime.evaluate("'hello'").unwrap();
        let f: &str = value.try_into().unwrap();
        assert_eq!(f, "hello");
    }
}
