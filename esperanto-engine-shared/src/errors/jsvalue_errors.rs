use thiserror::Error;

use super::{JSContextError, JSConversionError};
#[derive(Error, Debug)]
pub enum JSValueError {
    #[error("This operation requires an object, but this item is a value")]
    IsNotAnObject,

    #[error("An error occurred while trying to convert between native and JS contexts: {}", .0)]
    ConversionError(#[from] JSConversionError),

    #[error("An error occurred outside this JSValue: {}", .0)]
    ContextError(#[from] JSContextError),
}
