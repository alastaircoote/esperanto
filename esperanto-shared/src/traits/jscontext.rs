use crate::errors::JSEnvError;
use crate::traits::JSObject;
use crate::traits::JSValue;
pub trait JSContext {
    type ValueType: JSValue + 'static;
    type ObjectType: JSObject + 'static;
    type StoreKey: Sync + Send + Copy + Clone;
    fn evaluate(&self, script: &str) -> Result<Self::ValueType, JSEnvError>;
    fn new() -> Self;
    fn store_value(&mut self, value: Self::ValueType) -> Self::StoreKey;
    fn get_value_ref(&self, key: Self::StoreKey) -> Result<&Self::ValueType, JSEnvError>;
    fn pull_value(&mut self, key: Self::StoreKey) -> Result<Self::ValueType, JSEnvError>;
}
