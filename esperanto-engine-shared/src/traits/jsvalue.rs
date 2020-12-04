use std::convert::TryInto;

use crate::errors::{JSConversionError, JSValueError};

use super::JSContext;

pub trait JSValue
where
    Self: Clone,
    Self: TryInto<f64>,
    Self: TryInto<bool>,
    Self: TryInto<String>,
    Self: TryJSValueFrom<f64>,
    Self: TryJSValueFrom<bool>,
    Self: TryJSValueFrom<String>,
{
    type Context: JSContext<Value = Self>;

    fn set_property(&self, name: &str, value: &Self) -> Result<(), JSValueError>;
}

pub trait HasContext: Sized {
    type Context: JSContext<Value = Self>;
}

pub trait TryIntoJSValue<T: HasContext> {
    fn try_into_jsvalue(self, context: &T::Context) -> Result<T, JSConversionError>;
}

pub trait TryJSValueFrom<T>: HasContext {
    fn try_from(value: T, context: &Self::Context) -> Result<Self, JSConversionError>;
}

impl<JSV, T> TryJSValueFrom<T> for JSV
where
    T: TryIntoJSValue<JSV>,
    Self: HasContext,
{
    fn try_from(value: T, context: &Self::Context) -> Result<Self, JSConversionError> {
        value.try_into_jsvalue(context)
    }
}
