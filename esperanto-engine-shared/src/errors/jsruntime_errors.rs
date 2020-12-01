use thiserror::Error;

#[derive(Error, Debug)]
pub enum JSRuntimeError {
    #[error("Could not create the JS runtime")]
    CouldNotCreateRuntime,
    #[error("Runtime was not able to create a context")]
    CouldNotCreateContext,
}
