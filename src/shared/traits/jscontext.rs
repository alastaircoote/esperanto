use crate::EsperantoResult;

use super::{jsruntime::JSRuntime, jsvalue::JSValue};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum JSContextError {
    #[error("Could not create context, reason unknown")]
    CouldNotCreateContext,

    #[error("Could not retrieve context from native pointer")]
    CouldNotRetrieveFromNativePointer,
}

pub trait JSContext<'c> {
    type Runtime: JSRuntime<'c>;
    type Value: JSValue<'c, Context = Self>;

    fn evaluate(&self, script: String) -> EsperantoResult<Self::Value>;
}
