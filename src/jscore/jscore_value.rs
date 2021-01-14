use std::convert::{TryFrom, TryInto};

use crate::{
    shared::{
        external_api::{conversion_error::JSConversionError, value::Value},
        traits::tryas::TryIntoJS,
    },
    EsperantoError, EsperantoResult,
};

pub struct JSValue<'c> {
    pub(super) raw_ref: JSCoreValueRawRef<'c>,
    pub(super) context: &'c JSContext<'c>,
}

pub type JSCoreValue<'c> = JSValue<'c>;

impl<'c> Value<'c> for JSCoreValue<'c> {
    type Context = JSCoreContext<'c>;
    fn undefined(in_context: &'c JSContext) -> EsperantoResult<Self> {
        let raw = unsafe { JSValueMakeUndefined(in_context.raw_ref) };
        Ok(JSValue {
            raw_ref: raw.try_into()?,
            context: in_context,
        })
    }

    fn new_error(error_message: &str, in_context: &'c JSContext) -> EsperantoResult<Self> {
        let msg = JSValue::try_from_js(error_message, in_context)?;
        let args = [msg.raw_ref.as_const()];
        let raw = check_jscore_exception!(in_context, exception =>
            unsafe { JSObjectMakeError(in_context.raw_ref, 1, args.as_ptr(), exception)}
        )?;
        Ok(JSValue {
            raw_ref: raw.try_into()?,
            context: in_context,
        })
    }

    fn get_property(&self, name: &str) -> EsperantoResult<Self> {
        let name_jscore = JSCoreString::try_from(name)?;
        let mut_raw = self.raw_ref.as_mut()?;
        let raw_val = check_jscore_exception!(self.context, exception => {
            unsafe { JSObjectGetProperty(self.context.raw_ref, mut_raw, name_jscore.raw_ref, exception) }
        })?;
        unsafe { JSValueProtect(self.context.raw_ref, raw_val) };
        raw_val.try_into_js(self.context)
    }
}

pub trait JSCoreValuePrivate<'c> {
    fn try_from_js<V>(val: V, in_context: &'c JSContext) -> EsperantoResult<JSCoreValue<'c>>
    where
        V: TryIntoJS<'c>,
    {
        val.try_into_js(in_context)
    }
}

impl<'c> JSCoreValuePrivate<'c> for JSCoreValue<'c> {}

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
    JSObjectGetProperty, JSObjectMakeError, JSValueMakeString, JSValueMakeUndefined,
    JSValueProtect, JSValueToNumber, OpaqueJSValue,
};
// use super::{
//     check_exception, jscore_context::JSCoreContext, jscore_string::JSCoreString,
//     jscore_value_rawref::JSCoreValueRawRef,
// };
use thiserror::Error;

use super::{
    jscore_context::{JSContext, JSCoreContext},
    jscore_string::{self, JSCoreString},
    jscore_value_rawref::JSCoreValueRawRef,
};

#[derive(Error, Debug, PartialEq)]
pub(super) enum JSCoreValueError {
    #[error("Attempted to create a JSValue from a raw pointer but it was null")]
    RawReferenceWasNull,
}

impl From<JSCoreValueError> for EsperantoError {
    fn from(val: JSCoreValueError) -> Self {
        EsperantoError::EngineError(Box::new(val))
    }
}
