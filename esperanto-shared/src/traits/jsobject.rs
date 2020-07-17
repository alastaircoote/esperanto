use super::JSValue;
use crate::errors::JSContextError;
pub trait JSObject {
    type ValueType: JSValue + 'static;
    // fn to_string<'a>(&self) -> Result<&'a str, JSEnvError>;
    fn get_property(&self, name: &str) -> Result<Self::ValueType, JSContextError>;
}
