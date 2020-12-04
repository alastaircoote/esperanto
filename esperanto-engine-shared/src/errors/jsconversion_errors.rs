use thiserror::Error;

#[derive(Error, Debug)]
pub enum JSConversionError {
    #[error("Could not convert this value to a number")]
    CouldNotConvertToNumber,

    #[error("Could not encode the text provided to a JS-suitable string")]
    TextEncodingFailed,

    #[error("Could not decode the JS value into a valid UTF8 string")]
    TextDecodingFailed,

    #[error("An error occurred inside the JS environment: {}", .0)]
    JSErrorOccurred(String),

    #[error("An unknown error occurred ({})", .0)]
    UnknownError(String),
}
