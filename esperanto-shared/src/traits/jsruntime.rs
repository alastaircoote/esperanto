use std::rc::Rc;

use crate::errors::JSContextError;

use super::JSContext;

pub trait JSRuntime {
    type ContextType: JSContext;
    fn new() -> Result<Rc<Self>, JSContextError>;
    fn create_context(&self) -> Result<Self::ContextType, JSContextError>;
}
