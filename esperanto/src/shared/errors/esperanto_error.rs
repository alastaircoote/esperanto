use std::ffi::NulError;

use crate::{
    shared::{
        context::JSContextError, errors::conversion_error::ConversionError,
        runtime::JSRuntimeError, value::JSValueError,
    },
    JSContext,
};
use thiserror::Error;

use super::{CatchExceptionError, JSExportError, JavaScriptError};

#[derive(Debug, Error, Eq, PartialEq)]
pub enum EsperantoError {
    #[error(transparent)]
    RuntimeError(#[from] JSRuntimeError),

    #[error(transparent)]
    ContextError(#[from] JSContextError),

    #[error(transparent)]
    ValueError(#[from] JSValueError),

    #[error(transparent)]
    JavaScriptError(#[from] JavaScriptError),

    #[error(transparent)]
    ConversionError(#[from] ConversionError),

    #[error(transparent)]
    CatchExceptionError(#[from] Box<CatchExceptionError>),

    #[error(transparent)]
    ExportError(#[from] JSExportError),
}

pub type EsperantoResult<T> = Result<T, EsperantoError>;

impl From<NulError> for EsperantoError {
    fn from(err: NulError) -> Self {
        return EsperantoError::ConversionError(ConversionError::CouldNotConvertToJSString(err));
    }
}
