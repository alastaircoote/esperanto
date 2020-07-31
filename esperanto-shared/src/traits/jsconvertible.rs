use super::JSValue;
use crate::errors::JSContextError;
use std::rc::Rc;

pub trait ToJSValue<ValueType: JSValue> {
    fn to_js_value(
        &self,
        in_context: &Rc<ValueType::ContextType>,
    ) -> Result<ValueType, JSContextError>;
}

pub trait FromJSValue<ValueType: JSValue>
where
    Self: Sized,
{
    fn from_js_value(val: ValueType) -> Result<Self, JSContextError>;
}

impl<ValueType: JSValue> ToJSValue<ValueType> for f64 {
    fn to_js_value(
        &self,
        in_context: &Rc<ValueType::ContextType>,
    ) -> Result<ValueType, JSContextError> {
        Ok(ValueType::from_number(*self, in_context))
    }
}

impl<ValueType: JSValue> FromJSValue<ValueType> for f64 {
    fn from_js_value(val: ValueType) -> Result<Self, JSContextError> {
        Ok(val.as_number()?)
    }
}
