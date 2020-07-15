use crate::errors::JSEnvError;
use crate::JSValue;
use std::convert::TryFrom;
pub trait JSContext {
    type ValueType: JSValue + 'static;
    type StoreKey: Sync + Send + Copy + Clone;
    fn evaluate<O: TryFrom<Self::ValueType>>(&self, script: &str) -> Result<O, JSEnvError>;
    fn new() -> Self;
    fn store_value(&mut self, value: Self::ValueType) -> Self::StoreKey;
    fn get_value_ref(&self, key: Self::StoreKey) -> Result<&Self::ValueType, JSEnvError>;
    fn pull_value(&mut self, key: Self::StoreKey) -> Result<Self::ValueType, JSEnvError>;
}
