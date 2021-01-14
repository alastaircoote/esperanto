use std::pin::Pin;

use crate::EsperantoResult;

use super::context::Context;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum JSRuntimeError {
    #[error("Could not create the JS runtime")]
    CouldNotCreateRuntime,
    #[error("Could not create class")]
    CouldNotCreateClass,
}

// pub(crate) trait RuntimeHasContext<'r> {
//     type Context: Context<'r>;
// }

pub trait Runtime<'r>: /*RuntimeHasContext<'r> +*/ Sized {
    type Context: Context<'r>;
    fn new() -> EsperantoResult<Self>;
    fn create_context(&'r self) -> EsperantoResult<<Self::Context as Context>::SelfInstanceType>;
}

// pub trait JSRuntimeHasContext<'r, 'c>: JSRuntime<'r> {
// type Context: JSContext<'c>;

// }
