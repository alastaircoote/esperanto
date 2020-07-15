use esperanto_shared::errors::{JSConversionError, JSEnvError};
use esperanto_shared::traits::{JSContext, JSObject, JSValue};
use std::any::Any;
use std::convert::TryFrom;

pub struct DummyJSValue {
    underlying_value: Box<dyn Any + Send + Sync>,
}

impl DummyJSValue {
    pub fn new<T: Any + Send + Sync>(obj: T) -> Self {
        DummyJSValue {
            underlying_value: Box::new(obj),
        }
    }
}

pub struct DummyJSObject {}

impl JSObject for DummyJSObject {
    type ValueType = DummyJSValue;

    fn get_property(&self, name: &str) -> Result<Self::ValueType, JSEnvError> {
        return Ok(DummyJSValue::new(()));
    }
}

impl JSValue for DummyJSValue {
    // fn get_property(&self, name: &str) -> Result<Self, JSEnvError> {
    //     return Ok(DummyJSValue::new(()));
    // }
    // fn to_string<'b>(&self) -> Result<&'b str, JSEnvError> {
    //     match self.underlying_value.downcast_ref::<&str>() {
    //         Some(str_val) => Ok(str_val),
    //         None => Ok("dummy string value"),
    //     }
    // }
}

impl TryFrom<DummyJSValue> for &str {
    type Error = JSConversionError;
    fn try_from(value: DummyJSValue) -> Result<Self, Self::Error> {
        match value.underlying_value.downcast_ref::<&str>() {
            Some(str_val) => Ok(str_val),
            None => Err(JSConversionError::ConversionFailed),
        }
    }
}

pub struct DummyJSContext {
    value_store: Vec<DummyJSValue>,
}

impl JSContext for DummyJSContext {
    type ValueType = DummyJSValue;
    type ObjectType = DummyJSObject;
    type StoreKey = usize;
    fn evaluate(&self, _: &str) -> Result<DummyJSValue, JSEnvError> {
        Ok(DummyJSValue::new(Box::new(())))
    }

    fn new() -> Self {
        DummyJSContext {
            value_store: Vec::new(),
        }
    }

    fn store_value(&mut self, value: Self::ValueType) -> Self::StoreKey {
        let pos = self.value_store.len();
        self.value_store.push(value);
        pos
    }
    fn get_value_ref(&self, key: Self::StoreKey) -> Result<&Self::ValueType, JSEnvError> {
        Ok(&self.value_store[key])
    }
    fn pull_value(&mut self, key: Self::StoreKey) -> Result<Self::ValueType, JSEnvError> {
        Ok(self.value_store.remove(key))
    }
}
