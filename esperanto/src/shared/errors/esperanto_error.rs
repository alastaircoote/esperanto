use std::ffi::NulError;

use crate::shared::{
    context::JSContextError, errors::conversion_error::ConversionError, runtime::JSRuntimeError,
    value::JSValueError,
};
use thiserror::Error;

use super::{CatchExceptionError, JSExportError, JavaScriptError};

/// A wrapper for all our sub-error types. We return this from all the public functions
/// to let different types of error bubble up.
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

    // This has to be a Box<> because a CatchExceptionError can wrap an EsperantoError,
    // without a box we get a recursion error
    #[error(transparent)]
    CatchExceptionError(#[from] Box<CatchExceptionError>),

    #[error(transparent)]
    ExportError(#[from] JSExportError),
}

/// A shortcut we use all across the codebase to avoid having to import EsperantoError everywhere
pub type EsperantoResult<T> = Result<T, EsperantoError>;

impl From<NulError> for EsperantoError {
    fn from(err: NulError) -> Self {
        return EsperantoError::ConversionError(ConversionError::CouldNotConvertToJSString(err));
    }
}
