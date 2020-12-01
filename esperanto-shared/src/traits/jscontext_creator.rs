use crate::errors::JSContextError;

use super::JSContext;

pub trait JSContextCreator<'runtime> {
    type ContextType: JSContext<'runtime>;
    fn create_context<ContextType>(&self) -> Result<ContextType, JSContextError>;
}
