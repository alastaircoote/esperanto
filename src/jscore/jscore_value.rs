use super::{
    jscore_context::{JSContext, JSCoreContext},
    jscore_string::JSCoreString,
    jscore_value_rawref::JSCoreValueRawRef,
};
use crate::{
    shared::{external_api::value::Value, traits::tryas::TryIntoJS},
    EsperantoError, EsperantoResult,
};
use javascriptcore_sys::{
    JSObjectGetProperty, JSObjectMakeError, JSValueMakeUndefined, JSValueProtect,
};
use std::convert::{TryFrom, TryInto};
use thiserror::Error;

pub struct JSValue<'r, 'c, 'v> {
    pub(super) raw_ref: JSCoreValueRawRef<'v>,
    pub(super) context: &'c JSContext<'r, 'c>,
}

pub type JSCoreValue<'r, 'c, 'v> = JSValue<'r, 'c, 'v>;

impl<'r, 'c, 'v> Value<'r, 'c, 'v> for JSCoreValue<'r, 'c, 'v> {
    type Context = JSCoreContext<'r, 'c>;
    fn undefined(in_context: &'c Self::Context) -> EsperantoResult<Self> {
        let raw = unsafe { JSValueMakeUndefined(in_context.raw_ref) };
        Ok(JSValue {
            raw_ref: raw.try_into()?,
            context: in_context,
        })
    }

    fn new_error(error_message: &str, in_context: &'c Self::Context) -> EsperantoResult<Self> {
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

pub trait JSCoreValuePrivate<'r, 'c, 'v> {
    fn try_from_js<V>(
        val: V,
        in_context: &'c JSCoreContext<'r, 'c>,
    ) -> EsperantoResult<JSCoreValue<'r, 'c, 'v>>
    where
        V: TryIntoJS<'r, 'c, 'v>,
    {
        val.try_into_js(in_context)
    }
}

impl<'r, 'c, 'v> JSCoreValuePrivate<'r, 'c, 'v> for JSCoreValue<'r, 'c, 'v> {}

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
