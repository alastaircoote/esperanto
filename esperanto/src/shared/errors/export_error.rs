use std::{ffi::NulError, num::TryFromIntError};

use thiserror::Error;

use super::EsperantoError;
#[derive(Debug, Error, Eq, PartialEq)]
pub enum JSExportError {
    #[error("This class can't be constructed but somehow the constructor has been called")]
    ConstructorCalledOnNonConstructableClass,

    #[error("Could not process the argument number value provided by the runtime")]
    CouldNotConvertArgumentNumber(TryFromIntError),

    #[error("Could not get a reference to the underlying native object")]
    CouldNotGetNativeObject,
}
