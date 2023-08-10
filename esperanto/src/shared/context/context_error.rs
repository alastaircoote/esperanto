use thiserror::Error;

/// Errors that occur at the context level
#[derive(Debug, Error, Eq, PartialEq)]
pub enum JSContextError {
    #[error("Could not create a context")]
    CouldNotCreateContext,
    #[error("Could not parse script provided")]
    CouldNotParseScript,
    #[error("Cannot retain a JSValue from a different JSContext")]
    RetainingWithWrongContext,
    #[error("Could not get internal context representation")]
    CouldNotGetInternalRepresentation,
}
