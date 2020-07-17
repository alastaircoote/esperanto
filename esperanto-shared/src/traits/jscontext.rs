use crate::errors::JSContextError;
use crate::traits::JSObject;
use crate::traits::JSValue;
pub trait JSContext: Sized {
    type ValueType: JSValue + 'static;
    type ObjectType: JSObject + 'static;
    // type StoreKey: Sync + Send + Copy + Clone;
    fn evaluate(&self, script: &str) -> Result<Self::ValueType, JSContextError>;
    fn new() -> Result<Self, JSContextError>;
    // fn store_value(&mut self, value: Self::ValueType) -> Self::StoreKey;
    // fn get_value_ref(&self, key: Self::StoreKey) -> Result<&Self::ValueType, JSEnvError>;
    // fn pull_value(&mut self, key: Self::StoreKey) -> Result<Self::ValueType, JSEnvError>;
}
