use thiserror::Error;

use super::JSRuntimeError;
#[derive(Error, Debug)]
pub enum JSContextError {
    #[error("Could not create the JS runtime")]
    RuntimeErrorOccurred(#[from] JSRuntimeError),
}
