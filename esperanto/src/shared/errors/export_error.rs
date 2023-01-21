use std::num::TryFromIntError;

use thiserror::Error;

#[derive(Debug, Error, Eq, PartialEq)]
pub enum JSExportError {
    #[error("Class {0} does not have a constuctor")]
    ConstructorCalledOnNonConstructableClass(String),

    #[error("Class {0} cannot be called as a function")]
    CalledNonFunctionClass(String),

    #[error("Could not process the argument number value provided by the runtime")]
    CouldNotConvertArgumentNumber(TryFromIntError),

    #[error("Could not get a reference to the underlying native object")]
    CouldNotGetNativeObject,

    #[error("Something happened that we're not expecting at all")]
    UnexpectedBehaviour,
}
