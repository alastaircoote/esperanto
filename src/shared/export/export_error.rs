use thiserror::Error;

#[derive(Debug, Error)]
pub enum JSExportError {
    #[error("You didn't provide enough arguments when calling this function. Expected {}, received {}.", .expected, .actual)]
    NotEnoughArguments { expected: usize, actual: usize },
}
