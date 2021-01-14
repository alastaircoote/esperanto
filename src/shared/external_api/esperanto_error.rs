use std::fmt::Debug;
use thiserror::Error;

use crate::errors::JSExportError;

use super::{
    context::JSContextError, conversion_error::JSConversionError, runtime::JSRuntimeError,
    value::JSValueError,
};

pub trait EngineError: std::error::Error + 'static {}

#[derive(Error, Debug)]
pub enum EsperantoError {
    #[error(transparent)]
    RuntimeError(#[from] JSRuntimeError),

    #[error(transparent)]
    ContextError(#[from] JSContextError),

    #[error(transparent)]
    ValueError(#[from] JSValueError),

    #[error(transparent)]
    ExportError(#[from] JSExportError),

    #[error(transparent)]
    ExternalError(Box<dyn std::error::Error>),

    #[error(transparent)]
    EngineError(Box<dyn std::error::Error>),

    #[error("An error occurred inside the JS environment: {}", .0)]
    JSErrorOccurred(String),

    #[error(transparent)]
    ConversionError(#[from] JSConversionError),
}

impl EsperantoError {
    pub fn external<Err: std::error::Error + 'static>(error: Err) -> Self {
        EsperantoError::ExternalError(Box::new(error))
    }
}

impl<IE: EngineError> From<IE> for EsperantoError {
    fn from(val: IE) -> Self {
        EsperantoError::EngineError(Box::new(val))
    }
}
