#[derive(Debug, PartialEq, Copy, Clone)]
pub enum JSEnvError {
    UsingWrongThread,
    CouldNotParseScript,
    CouldNotConvertString,
}

pub trait JSValue {
    fn to_string<'a>(&self) -> Result<&'a str, JSEnvError>;
}

pub trait JSRuntime {
    type ValueType: JSValue + Send + Sync + 'static;
    fn evaluate(&self, script: &str) -> Result<Self::ValueType, JSEnvError>;
    fn new() -> Self;
}
