use crate::enums::JSType;
#[derive(Debug, Clone)]
struct JSError {
    message: String,
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
    JSErrorEncountered(String),
}
