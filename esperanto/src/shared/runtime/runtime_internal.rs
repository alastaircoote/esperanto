use super::runtime_error::JSRuntimeError;

pub(crate) trait JSRuntimeInternal: Sized {
    fn new() -> Result<Self, JSRuntimeError>;
    // fn retain(self) -> Self;
    fn release(self);
    
}
