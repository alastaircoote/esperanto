use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum JSRuntimeError {
    #[error("Could not create a runtime")]
    CouldNotCreateRuntime,

    #[error("Failed to retrieve runtime private context")]
    FailedToRetrievePrivateContext
}
