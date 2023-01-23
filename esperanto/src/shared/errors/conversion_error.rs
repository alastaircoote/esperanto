use std::{ffi::NulError, str::Utf8Error, num::TryFromIntError};

use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum ConversionError {
    #[error("Could not convert this native string into a JS-compatible one")]
    CouldNotConvertToJSString(#[from] NulError),

    #[error("Could not parse this JS string into a native-suitable one")]
    CouldNotConvertFromJSString(#[from] Utf8Error),

    #[error("Expected to receive an error but value is something else")]
    JSValueWasNotAnError,

    #[error("Could not convert value into an integer")]
    CouldNotConvertToInteger(#[from] TryFromIntError)
}
