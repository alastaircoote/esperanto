use crate::errors::JSEnvError;
use crate::JSValue;
pub trait JSRuntime {
    type ValueType: JSValue + 'static;
    type StoreKey: Sync + Send + Copy + Clone;
    fn evaluate<O: From<Self::ValueType>>(&self, script: &str) -> Result<O, JSEnvError>;
    fn new() -> Self;
    fn store_value(&mut self, value: Self::ValueType) -> Self::StoreKey;
    fn get_value_ref(&self, key: Self::StoreKey) -> Result<&Self::ValueType, JSEnvError>;
    fn pull_value(&mut self, key: Self::StoreKey) -> Result<Self::ValueType, JSEnvError>;
}
