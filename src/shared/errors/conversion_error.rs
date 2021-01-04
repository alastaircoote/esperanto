use thiserror::Error;

#[derive(Debug, Error)]
pub enum JSConversionError {
    #[error("Converting native string to JS string failed")]
    FailedToConvertStringToJS,

    #[error("Converting JS string to native failed")]
    FailedToConvertStringToNative,

    #[error("This value is not a number")]
    IsNotANumber,
}
