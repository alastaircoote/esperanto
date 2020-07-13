use esperanto_traits::js_traits::{JSEnvError, JSRuntime, JSValue};
use std::any::Any;

pub struct DummyJSValue<'a> {
    underlying_value: &'a (dyn Any + Send + Sync),
}

impl<'a> DummyJSValue<'a> {
    pub fn new<A: Any + Send + Sync>(underlying_value: &'a A) -> Self {
        DummyJSValue {
            underlying_value: underlying_value,
        }
    }
}

impl<'a> JSValue for DummyJSValue<'a> {
    fn to_string<'b>(&self) -> Result<&'b str, JSEnvError> {
        // self.underlying_value.
        match self.underlying_value.downcast_ref::<&str>() {
            Some(str_val) => Ok(str_val),
            None => Ok("dummy string value"),
        }
    }
}

pub struct DummyJSRuntime {}

impl JSRuntime for DummyJSRuntime {
    type ValueType = DummyJSValue<'static>;
    fn evaluate(&self, _: &str) -> Result<Self::ValueType, JSEnvError> {
        Ok(DummyJSValue::new(&()))
    }
    fn new() -> Self {
        DummyJSRuntime {}
    }
}
