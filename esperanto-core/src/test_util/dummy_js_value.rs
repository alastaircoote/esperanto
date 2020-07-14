use esperanto_traits::errors::{JSConversionError, JSEnvError};
use esperanto_traits::{JSRuntime, JSValue};
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

impl JSValue for DummyJSValue {
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

pub struct DummyJSRuntime {
    value_store: Vec<DummyJSValue>,
}

impl JSRuntime for DummyJSRuntime {
    type ValueType = DummyJSValue;
    type StoreKey = usize;
    fn evaluate<O: From<Self::ValueType>>(&self, _: &str) -> Result<O, JSEnvError> {
        Ok(DummyJSValue::new(Box::new(())).into())
    }

    fn new() -> Self {
        DummyJSRuntime {
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
