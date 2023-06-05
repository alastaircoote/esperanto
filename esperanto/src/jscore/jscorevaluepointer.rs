use std::{convert::TryFrom, ops::Deref};

use javascriptcore_sys::{JSValueIsObject, OpaqueJSValue};

use crate::{
    shared::{errors::EsperantoResult, value::JSValueError},
    JSValue,
};

use super::{jscorecontextpointer::JSCoreContextPointer, jscorevalue};

#[derive(Debug, Clone, Copy)]
pub enum JSCoreValuePointer {
    Object(*mut OpaqueJSValue),
    Value(*const OpaqueJSValue),
}

impl JSCoreValuePointer {
    pub fn try_as_object(
        self,
        in_context: JSCoreContextPointer,
    ) -> EsperantoResult<*mut OpaqueJSValue> {
        match self {
            // This is messy but seems to be the only way to get Rust to allow us to (effectively) copy a mutable
            // reference.
            JSCoreValuePointer::Object(o) => Ok(o),
            JSCoreValuePointer::Value(v) => {
                let is_obj = unsafe { JSValueIsObject(*in_context, v) };
                if is_obj {
                    // This is gross but it works. Maybe worth revisiting what we're doing here
                    // at some point.
                    return Ok(v as _);
                }
                return Err(JSValueError::IsNotAnObject.into());
            }
        }
    }

    pub fn as_value(self) -> *const OpaqueJSValue {
        match self {
            JSCoreValuePointer::Object(o) => o,
            JSCoreValuePointer::Value(v) => v,
        }
    }
}

impl PartialEq for JSCoreValuePointer {
    fn eq(&self, other: &Self) -> bool {
        let const_self = self.as_value();
        let const_other = other.as_value();
        const_self == const_other
    }
}

impl From<*mut OpaqueJSValue> for JSCoreValuePointer {
    fn from(val: *mut OpaqueJSValue) -> Self {
        JSCoreValuePointer::Object(unsafe { val.as_mut().unwrap() })
    }
}

impl From<*const OpaqueJSValue> for JSCoreValuePointer {
    fn from(val: *const OpaqueJSValue) -> Self {
        JSCoreValuePointer::Value(unsafe { val.as_ref().unwrap() })
    }
}

impl From<JSValue<'_>> for *const OpaqueJSValue {
    fn from(val: JSValue<'_>) -> Self {
        val.internal.as_value()
    }
}
