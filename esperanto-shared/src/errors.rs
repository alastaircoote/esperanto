use crate::enums::JSType;
use std::{ffi::NulError, fmt::Display, str::Utf8Error};
use thiserror::*;
/// JSError is a representation of an Error within JavaScript. It's also kind of a method of last resort:
/// it could be implemented with the various JSValue and JSObject methods needed, but those methods can
/// return a JSError themselves. To avoid getting in some kind of infinite loop, JSError constructors are
/// implemented at the library-specific level.
#[derive(Error, Debug, PartialEq, Clone)]
pub struct JSError {
    pub name: String,
    pub message: String,
}

impl Display for JSError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.name, self.message)
    }
}

impl JSError {
    pub fn unknown() -> Self {
        JSError {
            name: "UnknownError".to_string(),
            message: "An error occurred but it couldn't be decoded from the JS environment"
                .to_string(),
        }
    }

    //     pub fn from_value<Value: JSValue>(val: Value) -> Self {
    //         // The JSError is intended as last resort when something fails, so if we encounter
    //         // an error while creating it then we'll just create an "unknown" error. Otherwise
    //         // we'll be chasing circles forever.

    //         // Casting this to a generic error because we don't actually care about the error
    //         // types, since we disregard them anyway.
    //         let to_obj = val.to_object() as Result<Value::ObjectType, dyn std::error::Error>;

    //         let (name, message) = to_obj
    //             .and_then(|obj| {
    //                 let name_prop = obj.get_property("name")?;
    //                 let message_prop = obj.get_property("message")?;

    //                 return Ok((name_prop.to_string()?, message_prop.to_string()?));
    //             })
    //             .unwrap_or((
    //                 "UnknownError",
    //                 "An error occurred but it couldn't be decoded from the JS environment",
    //             ));

    //         return JSError {
    //             message: (*message).to_string(),
    //             name: (*name).to_string(),
    //         };
    //     }
}

#[derive(Error, Debug)]
pub enum JSConversionError {
    #[error("The string you provided could not be converted into a JS-suitable one")]
    CouldNotConvertStringToSuitableFormat,
    #[error("The string was longer than the OS can accomodate (please don't ever do this..)")]
    StringWasTooLong,
    #[error("Could not convert between types")]
    ConversionFailed,
    #[error("Converting the provided string to a C-compatible string failed.")]
    ConversionToCStringFailed(#[from] NulError),
    #[error("Converting the C string to a native string failed due to a UTF8 encoding error")]
    ConversionFromCStringFailed(#[from] Utf8Error),
    #[error("The result of this conversion was not a number")]
    ResultIsNotANumber,
    #[error("An unknown error ocurred")]
    UnknownError,
}

#[derive(Error, Debug, PartialEq, Clone)]
pub enum JSEnvError {
    #[error("You are attempting to use the JS environment on the wrong thread. Please use the worker or JSValueWrapper.")]
    UsingWrongThread,
    #[error("The string you provided could not be parsed as valid JavaScript")]
    CouldNotParseScript,
    // #[error("An error occurred while converting values: {}", .0)]
    // ConversionError(JSConversionError),
    #[error(
        "This value has already been garbage collected. This should never happen, report a bug"
    )]
    ValueNoLongerExists,
    #[error("You provided the wrong type for this operation.")]
    IncorrectTypeForThisOperation(JSType, JSType),
    #[error("The JavaScript runtime threw an error")]
    JSErrorEncountered(JSError),
    #[error("Cannot encode the text provided into the correct format")]
    TextEncodingFailed,
    #[error("An unknown error occurred")]
    UnknownInternalError,
}
#[derive(Error, Debug)]
pub enum JSEvaluationError {
    #[error("Expected {} arguments, received {}", .expected, .actual)]
    NotEnoughArguments { expected: usize, actual: usize },
    #[error("This is not a function")]
    IsNotAFunction,
    #[error("This is not an object")]
    IsNotAnObject,
}

#[derive(Error, Debug)]
pub enum JSContextError {
    #[error("An error occurred inside the JavaScript environment")]
    JavaScriptErrorOccurred(#[from] JSError),
    #[error("Failed to convert the provided values")]
    ConversionError(#[from] JSConversionError),
    #[error("Creation of the JS context failed")]
    CouldNotCreateContext,
    #[error("The JS context has already been destroyed")]
    ContextAlreadyDestroyed,
    #[error("An evaluation error occurred")]
    EvaluationError(#[from] JSEvaluationError),
    #[error("This runtime does not support the operation you attempted")]
    NotSupported,
}

impl From<NulError> for JSContextError {
    fn from(e: NulError) -> Self {
        return JSConversionError::ConversionToCStringFailed(e).into();
    }
}

impl From<Utf8Error> for JSContextError {
    fn from(e: Utf8Error) -> Self {
        return JSConversionError::ConversionFromCStringFailed(e).into();
    }
}

impl From<JSConversionError> for JSError {
    fn from(err: JSConversionError) -> Self {
        JSError {
            name: "ConversionError".to_string(),
            message: format!("Could not convert value, reason: {}", err),
        }
    }
}
