use super::{JSContext, JSValue};
use crate::errors::JSEnvError;
pub trait JSObject
where
    Self: Sized,
{
    type ValueType: JSValue + 'static;
    // fn to_string<'a>(&self) -> Result<&'a str, JSEnvError>;
    fn get_property(&self, name: &str) -> Result<Self::ValueType, JSEnvError>;
}
