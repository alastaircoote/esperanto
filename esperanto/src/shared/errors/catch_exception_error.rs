use std::ffi::NulError;

use thiserror::Error;

use super::EsperantoError;
#[derive(Debug, Error, Eq, PartialEq)]
pub enum CatchExceptionError {
    #[error("Couldn't create the strings used to get error details. This should never happen!")]
    CouldNotCreateIdentifierString(#[from] NulError),

    #[error("Could not get the details of the error that occurred ({})", .0)]
    CouldNotGetDetailsOfError(#[from] EsperantoError),

    #[error("An unknown problem occurred when trying to get details of an error")]
    UnknownExceptionOccurred,

    #[error("This JavaScript value is not an error")]
    IsNotAnError,
}
