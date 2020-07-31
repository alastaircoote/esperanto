use crate::errors::JSContextError;
use crate::traits::JSObject;
use crate::traits::JSValue;
use std::rc::Rc;
pub trait JSContext: Sized + 'static {
    type ValueType: JSValue<ContextType = Self> + 'static;
    type ObjectType: JSObject + 'static;
    fn evaluate(self: &Rc<Self>, script: &str) -> Result<Self::ValueType, JSContextError>;
    fn new() -> Result<Rc<Self>, JSContextError>;
}
