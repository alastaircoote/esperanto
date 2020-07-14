#[derive(Debug, PartialEq, Copy, Clone)]
pub enum JSType {
    String,
    Number,
    Object,
    Array,
}

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
pub trait JSValue {
    // fn to_string<'a>(&self) -> Result<&'a str, JSEnvError>;
}

pub trait JSRuntime {
    type ValueType: JSValue + 'static;
    type StoreKey: Sync + Send + Copy + Clone;
    fn evaluate<O: From<Self::ValueType>>(&self, script: &str) -> Result<O, JSEnvError>;
    fn new() -> Self;
    fn store_value(&mut self, value: Self::ValueType) -> Self::StoreKey;
    fn get_value_ref(&self, key: Self::StoreKey) -> Result<&Self::ValueType, JSEnvError>;
    fn pull_value(&mut self, key: Self::StoreKey) -> Result<Self::ValueType, JSEnvError>;
}
