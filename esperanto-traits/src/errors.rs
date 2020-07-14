use crate::JSType;
#[derive(Debug, PartialEq, Copy, Clone)]
pub enum JSConversionError {
    CouldNotConvertStringToSuitableFormat,
    StringWasTooLong,
    ConversionFailed,
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum JSEnvError {
    UsingWrongThread,
    CouldNotParseScript,
    ConversionError(JSConversionError),
    ValueNoLongerExists,
    IncorrectTypeForThisOperation(JSType, JSType),
}
