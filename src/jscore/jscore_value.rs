use std::convert::{TryFrom, TryInto};

use crate::{
    errors::{EsperantoError, JSConversionError},
    shared::traits::{jsvalue::JSValue, tryas::TryIntoJS},
    EsperantoResult,
};

pub struct JSCoreValue<'c> {
    pub raw_ref: JSCoreValueRawRef<'c>,
    pub(super) context: &'c JSCoreContext<'c>,
}

impl<'c> JSValue<'c> for JSCoreValue<'c> {
    type Context = JSCoreContext<'c>;

    fn undefined(in_context: &'c Self::Context) -> EsperantoResult<Self> {
        let raw = unsafe { JSValueMakeUndefined(in_context.raw_ref) };
        Ok(JSCoreValue {
            raw_ref: raw.try_into()?,
            context: in_context,
        })
    }

    fn new_error(error_message: &str, in_context: &'c Self::Context) -> EsperantoResult<Self> {
        let msg = JSCoreValue::try_from_js(error_message, in_context)?;
        let args = vec![msg.raw_ref.as_const()];
        let raw = check_jscore_exception!(in_context, exception =>
            unsafe { JSObjectMakeError(in_context.raw_ref, 1, args.as_ptr(), exception)}
        )?;
        Ok(JSCoreValue {
            raw_ref: raw.try_into()?,
            context: in_context,
        })
    }
}

// impl<'c> TryIntoJS<'c, JSCoreValue<'c>> for *const OpaqueJSValue {
//     fn try_into_js(self, in_context: &'c JSCoreContext) -> EsperantoResult<JSCoreValue<'c>> {
//         unsafe { JSValueProtect(in_context.raw_ref, self) };
//         Ok(JSCoreValue {
//             raw_ref: self.try_into()?,
//             context: in_context,
//         })
//     }
// }

// impl<'c> TryIntoJS<'c, JSCoreValue<'c>> for *mut OpaqueJSValue {
//     fn try_into_js(self, in_context: &'c JSCoreContext) -> EsperantoResult<JSCoreValue<'c>> {
//         unsafe { JSValueProtect(in_context.raw_ref, self) };
//         Ok(JSCoreValue {
//             raw_ref: self.try_into()?,
//             context: in_context,
//         })
//     }
// }

// impl<'c> TryIntoJS<'c, JSCoreValue<'c>> for JSCoreString {
//     fn try_into_js(self, in_context: &'c JSCoreContext) -> EsperantoResult<JSCoreValue<'c>> {
//         let raw_ref = unsafe { JSValueMakeString(in_context.raw_ref, self.raw_ref) };
//         Ok(JSCoreValue {
//             raw_ref: raw_ref.try_into()?,
//             context: in_context,
//         })
//     }
// }

impl TryFrom<&JSCoreValue<'_>> for f64 {
    type Error = EsperantoError;

    fn try_from(value: &JSCoreValue<'_>) -> Result<Self, Self::Error> {
        let num = check_jscore_exception!(value.context, exception =>
            unsafe {JSValueToNumber(value.context.raw_ref, value.raw_ref.as_const(), exception)}
        )?;
        if num.is_nan() {
            return Err(JSConversionError::IsNotANumber.into());
        }
        Ok(num)
    }
}

impl TryFrom<&JSCoreValue<'_>> for &str {
    type Error = EsperantoError;

    fn try_from(value: &JSCoreValue) -> Result<Self, Self::Error> {
        value.try_into()
    }
}

// impl<'a, T> TryFrom<&'a JSCoreValue<'a>> for Option<T>
// where
//     T: TryFrom<&'a JSCoreValue<'a>>,
// {
//     type Error = EsperantoError;

//     fn try_from(value: &'a JSCoreValue<'a>) -> Result<Self, Self::Error> {
//         todo!()
//     }
// }

// use std::convert::{TryFrom, TryInto};

// use javascriptcore_sys::{JSValueMakeUndefined, JSValueToNumber, OpaqueJSValue};

// use crate::{
//     errors::{EsperantoError, JSConversionError},
//     shared::traits::{jsconvertable::TryIntoJSValue, jsvalue::JSValue},
//     traits::TryAsRef,
//     EsperantoResult,
// };

use javascriptcore_sys::{
    JSObjectMakeError, JSValueMakeString, JSValueMakeUndefined, JSValueProtect, JSValueToNumber,
    OpaqueJSValue,
};
// use super::{
//     check_exception, jscore_context::JSCoreContext, jscore_string::JSCoreString,
//     jscore_value_rawref::JSCoreValueRawRef,
// };
use thiserror::Error;

use super::{
    jscore_context::JSCoreContext, jscore_string::JSCoreString,
    jscore_value_rawref::JSCoreValueRawRef,
};

#[derive(Error, Debug, PartialEq)]
pub(super) enum JSCoreValueError {
    #[error("Attempted to create a JSValue from a raw pointer but it was null")]
    RawReferenceWasNull,
}

impl From<JSCoreValueError> for EsperantoError {
    fn from(val: JSCoreValueError) -> Self {
        EsperantoError::implementation_error_from(val)
    }
}
