use crate::EsperantoResult;

use super::runtime::Runtime;
use super::value::Value;
use crate::JSExport;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum JSContextError {
    #[error("Could not create context, reason unknown")]
    CouldNotCreateContext,

    #[error("Could not retrieve context from native pointer")]
    CouldNotRetrieveFromNativePointer,
}

pub struct EvaluateMetadata<'a> {
    pub(crate) file_name: &'a str,
    pub(crate) line_number: i32,
}

impl<'a> EvaluateMetadata<'a> {
    pub fn new(file_name: &'a str, line_number: i32) -> Self {
        EvaluateMetadata {
            file_name,
            line_number,
        }
    }
}

pub trait Context<'r, 'c, 'v>: Sized {
    type Runtime: Runtime<'r, 'c, 'v>;
    type Value: Value<'r, 'c, 'v, Context = Self>;
    type SelfInstanceType;

    fn evaluate(
        &'c self,
        script: &str,
        meta: Option<EvaluateMetadata>,
    ) -> EsperantoResult<Self::Value>;

    fn new(runtime: Option<&'r Self::Runtime>) -> EsperantoResult<Self::SelfInstanceType>;
    fn new_with_global<G: JSExport>(
        runtime: Option<&'r Self::Runtime>,
        global_object: G,
    ) -> EsperantoResult<Self::SelfInstanceType>;
}
