use std::pin::Pin;

use super::jscontext::JSContext;
use crate::shared::errors::EsperantoResult;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum JSRuntimeError {
    #[error("Could not create the JS runtime")]
    CouldNotCreateRuntime,
    #[error("Could not create class")]
    CouldNotCreateClass,
}

pub trait JSRuntime<'r>: Sized {
    type Context: JSContext<'r>;
    fn new() -> EsperantoResult<Self>;
    fn create_context(&'r self) -> EsperantoResult<Pin<Box<Self::Context>>>;
}

// pub trait JSRuntimeHasContext<'r, 'c>: JSRuntime<'r> {
//     type Context: JSContext<'c>;

// }
