use std::num::TryFromIntError;

use thiserror::Error;

#[derive(Debug, Error, Eq, PartialEq)]
pub enum JSExportError {
    #[error("Class {0} does not have a constructor")]
    ConstructorCalledOnNonConstructableClass(&'static str),

    #[error("Class {0} cannot be called as a function")]
    CalledNonFunctionClass(&'static str),

    #[error("Could not process the argument number value provided by the runtime")]
    CouldNotConvertArgumentNumber(TryFromIntError),

    #[error("Could not get a reference to the underlying native object")]
    CouldNotGetNativeObject,

    #[error("Something happened that we're not expecting at all")]
    UnexpectedBehaviour,

    #[error("Expected native object to be {expected} but it was {actual}")]
    IncorrectNativeType {
        expected: &'static str,
        actual: &'static str,
    },
}
