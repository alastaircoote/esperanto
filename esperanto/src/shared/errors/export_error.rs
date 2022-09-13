use std::{ffi::NulError, num::TryFromIntError};

use thiserror::Error;

use super::EsperantoError;
#[derive(Debug, Error, Eq, PartialEq)]
pub enum JSExportError {
    #[error("Class constructor `${0}` cannot be invoked without 'new'")]
    ConstructorCalledOnNonConstructableClass(String),

    #[error("Could not process the argument number value provided by the runtime")]
    CouldNotConvertArgumentNumber(TryFromIntError),

    #[error("Could not get a reference to the underlying native object")]
    CouldNotGetNativeObject,

    #[error("Something happened that we're not expecting at all")]
    UnexpectedBehaviour,
}
