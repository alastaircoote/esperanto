use crate::errors::JSEnvError;
use crate::traits::JSContext;
pub trait JSValue
where
    Self: Sized,
{
    // type ContextType: JSContext + 'static;
    // fn to_string<'a>(&self) -> Result<&'a str, JSEnvError>;
    // fn get_property(&self, name: &str) -> Result<Self, JSEnvError>;
}
