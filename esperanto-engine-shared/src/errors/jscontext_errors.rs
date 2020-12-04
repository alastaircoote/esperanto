use thiserror::Error;

use super::{JSConversionError, JSRuntimeError};
#[derive(Error, Debug)]
pub enum JSContextError {
    #[error("Could not create the JS runtime")]
    RuntimeErrorOccurred(#[from] JSRuntimeError),

    #[error("An error occurred inside the JS environment: {}", .0)]
    JSErrorOccurred(String),

    #[error("A conversion error occurred: {}", .0)]
    ConversionError(#[from] JSConversionError),
}
