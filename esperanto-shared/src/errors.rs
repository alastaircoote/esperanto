use crate::enums::JSType;
use crate::traits::JSObject;
use std::convert::{TryFrom, TryInto};

#[derive(Debug, PartialEq, Clone)]
pub struct JSError {
    pub name: String,
    pub message: String,
}

impl JSError {
    pub fn from<Object: JSObject>(val: Object) -> Result<Self, JSEnvError>
    where
        Object::ValueType: TryInto<String>,
    {
        let message: String = val
            .get_property("message")?
            .try_into()
            .map_err(|_| JSConversionError::ConversionFailed)?;

        let name: String = val
            .get_property("name")?
            .try_into()
            .map_err(|_| JSConversionError::ConversionFailed)?;

        return Ok(JSError { message, name });
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum JSConversionError {
    CouldNotConvertStringToSuitableFormat,
    StringWasTooLong,
    ConversionFailed,
}

#[derive(Debug, PartialEq, Clone)]
pub enum JSEnvError {
    UsingWrongThread,
    CouldNotParseScript,
    ConversionError(JSConversionError),
    ValueNoLongerExists,
    IncorrectTypeForThisOperation(JSType, JSType),
    JSErrorEncountered(JSError),
    TextEncodingFailed,
}

// impl Into<JSEnvError> for JSConversionError {
//     fn into(self) -> JSEnvError {
//         return JSEnvError::ConversionError(self);
//     }
// }

impl From<JSConversionError> for JSEnvError {
    fn from(val: JSConversionError) -> Self {
        return JSEnvError::ConversionError(val);
    }
}
