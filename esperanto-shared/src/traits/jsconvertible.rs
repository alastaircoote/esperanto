use super::{JSContext, JSValue};
use crate::errors::JSError;
use std::rc::Rc;

pub trait ToJSValue<ValueType: JSValue> {
    fn to_js_value(
        &self,
        in_context: &<ValueType::ContextType as JSContext>::SharedRef,
    ) -> Result<ValueType, JSError>;
}

pub trait FromJSValue<ValueType: JSValue>
where
    Self: Sized,
{
    fn from_js_value(val: ValueType) -> Result<Self, JSError>;
}

impl<ValueType: JSValue> ToJSValue<ValueType> for f64 {
    fn to_js_value(
        &self,
        in_context: &<ValueType::ContextType as JSContext>::SharedRef,
    ) -> Result<ValueType, JSError> {
        Ok(ValueType::from_number(self, in_context))
    }
}

impl<ValueType: JSValue> FromJSValue<ValueType> for f64 {
    fn from_js_value(val: ValueType) -> Result<Self, JSError> {
        Ok(val.as_number()?)
    }
}
