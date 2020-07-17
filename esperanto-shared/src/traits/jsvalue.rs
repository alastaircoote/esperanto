use super::JSObject;
use crate::errors::{JSContextError, JSConversionError};
pub trait JSValue
where
    Self: Sized,
{
    // type ContextType: JSContext + 'static;
    type ObjectType: JSObject + 'static;
    fn to_string(&self) -> Result<String, JSConversionError>;
    fn to_object(&self) -> Result<Self::ObjectType, JSContextError>;
    // fn get_property(&self, name: &str) -> Result<Self, JSEnvError>;
}
