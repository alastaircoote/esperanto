use super::context::Context;
use crate::EsperantoResult;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum JSRuntimeError {
    #[error("Could not create the JS runtime")]
    CouldNotCreateRuntime,
    #[error("Could not create class")]
    CouldNotCreateClass,
}

pub trait Runtime<'r, 'c, 'v>: /*RuntimeHasContext<'r> +*/ Sized {
    type Context: Context<'r, 'c,'v, Runtime = Self>;
    fn new() -> EsperantoResult<Self>;
}
