// use crate::EsperantoResult;

// use super::jsruntime::Runtime;
// use thiserror::Error;

// #[derive(Error, Debug)]
// pub enum JSContextError {
//     #[error("Could not create context, reason unknown")]
//     CouldNotCreateContext,

//     #[error("Could not retrieve context from native pointer")]
//     CouldNotRetrieveFromNativePointer,
// }

// pub trait Context<'c> {
//     type Runtime: Runtime<'c>;
//     type Value: Value<'c, Context = Self>;

//     // fn evaluate(&self, script: String) -> EsperantoResult<Self::Value>;
// }
