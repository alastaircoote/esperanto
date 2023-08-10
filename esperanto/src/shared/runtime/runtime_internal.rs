use super::runtime_error::JSRuntimeError;

pub(crate) trait JSRuntimeInternal: Sized + Eq {
    fn new() -> Result<Self, JSRuntimeError>;
    // fn retain(self) -> Self;
    fn release(&mut self);
}
