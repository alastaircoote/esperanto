use crate::enums::JSType;
use crate::traits::{JSObject, JSValue};
use std::{
    convert::{TryFrom, TryInto},
    error::Error,
    ffi::NulError,
    fmt::Display,
};
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
    #[error("String conversion failed")]
    StringConversionFailed(Box<dyn std::error::Error>),
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
pub enum JSContextError {
    #[error("An error occurred inside the JavaScript environment")]
    JavaScriptErrorOccurred(#[from] JSError),
    #[error("Failed to convert the provided values")]
    ConversionError(#[from] JSConversionError),
    #[error("Creation of the JS context failed")]
    CouldNotCreateContext,
}
