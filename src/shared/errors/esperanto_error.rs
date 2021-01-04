use std::fmt::Debug;
use thiserror::Error;

use crate::{
    errors::JSExportError,
    shared::traits::{jscontext::JSContextError, jsruntime::JSRuntimeError, jsvalue::JSValueError},
};

use super::JSConversionError;

pub trait ImplementationError: std::error::Error + 'static {}

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

    #[error("An implementation specific detail occurred: {}", .0)]
    ImplementationError(Box<dyn std::error::Error>),

    #[error("An error occurred inside the JS environment: {}", .0)]
    JSErrorOccurred(String),

    #[error(transparent)]
    ConversionError(#[from] JSConversionError),
}

impl EsperantoError {
    pub fn implementation_error_from<Err: std::error::Error + 'static>(error: Err) -> Self {
        EsperantoError::ImplementationError(Box::new(error))
    }
}

pub type EsperantoResult<T> = Result<T, EsperantoError>;

impl<IE: ImplementationError> From<IE> for EsperantoError {
    fn from(val: IE) -> Self {
        EsperantoError::ImplementationError(Box::new(val))
    }
}
