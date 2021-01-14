use crate::{
    convert::TryFromJSValueRef, shared::external_api::conversion_error::JSConversionError,
};
use std::convert::{TryFrom, TryInto};

use javascriptcore_sys::{JSValueToNumber, OpaqueJSValue};

use crate::EsperantoError;

use super::{jscore_string::JSCoreString, jscore_value::JSValue};

impl TryFromJSValueRef for f64 {
    fn try_from_js_ref(value: &JSValue) -> Result<Self, EsperantoError> {
        let num = check_jscore_exception!(value.context, exception =>
            unsafe {JSValueToNumber(value.context.raw_ref, value.raw_ref.as_const(), exception)}
        )?;
        if num.is_nan() {
            return Err(JSConversionError::IsNotANumber.into());
        }
        Ok(num)
    }
}

impl TryFromJSValueRef for &str {
    fn try_from_js_ref(value: &JSValue) -> Result<Self, EsperantoError> {
        let jscore_string: JSCoreString = value.try_into()?;
        Self::try_from(&jscore_string)
    }
}

impl TryFrom<&JSValue<'_>> for String {
    type Error = EsperantoError;

    fn try_from(value: &JSValue<'_>) -> Result<Self, Self::Error> {
        let st = &JSCoreString::try_from(value)?;
        st.try_into()
    }
}

impl TryFrom<&JSValue<'_>> for &str {
    type Error = EsperantoError;

    fn try_from(value: &JSValue<'_>) -> Result<Self, Self::Error> {
        let st = &JSCoreString::try_from(value)?;
        st.try_into()
    }
}

impl<'c> From<JSValue<'c>> for *const OpaqueJSValue {
    fn from(val: JSValue<'c>) -> Self {
        val.raw_ref.as_const()
    }
}
