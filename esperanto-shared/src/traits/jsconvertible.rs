use super::JSValue;
use crate::errors::JSContextError;

pub trait ToJSValue<'group, ValueType: JSValue<'group>> {
    fn to_js_value(
        &'group self,
        in_context: &'group ValueType::ContextType,
    ) -> Result<&'group ValueType, JSContextError>;
}

pub trait FromJSValue<'group, ValueType: JSValue<'group>>
where
    Self: Sized,
{
    fn from_js_value(val: &'group ValueType) -> Result<&'group Self, JSContextError>;
}

impl<'group, ValueType: JSValue<'group>> ToJSValue<'group, ValueType> for ValueType {
    fn to_js_value(
        &'group self,
        _: &ValueType::ContextType,
    ) -> Result<&'group ValueType, JSContextError> {
        Ok(self)
    }
}

impl<'group, ValueType: JSValue<'group>> FromJSValue<'group, ValueType> for ValueType {
    fn from_js_value(val: &'group ValueType) -> Result<&'group Self, JSContextError> {
        Ok(val)
    }
}

impl<'group, ValueType: JSValue<'group>> ToJSValue<'group, ValueType> for f64 {
    fn to_js_value(
        &'group self,
        in_context: &'group ValueType::ContextType,
    ) -> Result<&ValueType, JSContextError> {
        ValueType::from_number(*self, in_context)
    }
}

impl<'group, ValueType: JSValue<'group>> FromJSValue<'group, ValueType> for f64 {
    fn from_js_value(val: &ValueType) -> Result<&Self, JSContextError> {
        val.as_number()
    }
}
