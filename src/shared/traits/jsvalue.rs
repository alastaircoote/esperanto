use super::{jscontext::JSContext, tryas::TryIntoJS};
use crate::EsperantoResult;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum JSValueError {
    #[error("The target JSValue is not an object and it needs to be")]
    IsNotAnObject,
}

pub trait JSValue<'c>: Sized {
    type Context: JSContext<'c, Value = Self>;

    fn undefined(in_context: &'c Self::Context) -> EsperantoResult<Self>;
    fn new_error(error_message: &str, in_context: &'c Self::Context) -> EsperantoResult<Self>;

    fn try_from_js<V: TryIntoJS<'c, Self>>(
        val: V,
        in_context: &'c Self::Context,
    ) -> EsperantoResult<Self> {
        val.try_into_js(in_context)
    }
}
