use std::convert::TryFrom;

use javascriptcore_sys::OpaqueJSValue;

use crate::{shared::external_api::value::JSValueError, EsperantoError, EsperantoResult};

use super::jscore_value::JSCoreValueError;

pub enum JSCoreValueRawRef<'a> {
    Value(&'a OpaqueJSValue),
    Object(&'a OpaqueJSValue),
}

impl<'a> JSCoreValueRawRef<'a> {
    pub fn as_const(&self) -> *const OpaqueJSValue {
        match self {
            JSCoreValueRawRef::Value(v) => *v as _,
            JSCoreValueRawRef::Object(o) => *o as _,
        }
    }
    pub fn as_mut(&self) -> EsperantoResult<*mut OpaqueJSValue> {
        match self {
            JSCoreValueRawRef::Object(o) => Ok(*o as *const OpaqueJSValue as _),
            _ => Err(JSValueError::IsNotAnObject.into()),
        }
    }
}

impl<'a> TryFrom<*const OpaqueJSValue> for JSCoreValueRawRef<'a> {
    type Error = EsperantoError;

    fn try_from(value: *const OpaqueJSValue) -> Result<Self, Self::Error> {
        unsafe { value.as_ref() }
            .map(|v| JSCoreValueRawRef::Value(v))
            .ok_or(JSCoreValueError::RawReferenceWasNull.into())
    }
}

impl<'a> TryFrom<*mut OpaqueJSValue> for JSCoreValueRawRef<'a> {
    type Error = EsperantoError;

    fn try_from(value: *mut OpaqueJSValue) -> Result<Self, Self::Error> {
        unsafe { value.as_mut() }
            .map(|v| JSCoreValueRawRef::Object(v))
            .ok_or(JSCoreValueError::RawReferenceWasNull.into())
    }
}
