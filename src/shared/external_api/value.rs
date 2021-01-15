use super::context::Context;
use crate::EsperantoResult;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum JSValueError {
    #[error("The target JSValue is not an object and it needs to be")]
    IsNotAnObject,
}

pub trait Value<'r, 'c, 'v>: Sized {
    type Context: Context<'r, 'c, 'v, Value = Self>;

    fn undefined(in_context: &'c Self::Context) -> EsperantoResult<Self>;
    fn new_error(error_message: &str, in_context: &'c Self::Context) -> EsperantoResult<Self>;
    fn get_property(&self, name: &str) -> EsperantoResult<Self>;
}
