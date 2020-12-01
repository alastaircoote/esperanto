use std::convert::TryInto;

use super::{
    jsconvertible::{FromJSValue, ToJSValue},
    JSContext,
};
use crate::errors::JSContextError;
pub trait JSValue<'runtime>
where
    Self: Sized,

    Self: TryInto<f64, Error = JSContextError>,
{
    type ContextType: JSContext<'runtime, ValueType = Self>;
    type RawType: Copy;
    fn as_string(&self) -> Result<&String, JSContextError>;
    fn as_number(&self) -> Result<&f64, JSContextError>;
    fn as_bool(&self) -> Result<&bool, JSContextError>;
    fn from_number(number: f64, in_context: &Self::ContextType) -> Result<&Self, JSContextError>;
    fn from_bool(bool: bool, in_context: &Self::ContextType) -> Result<&Self, JSContextError>;
    fn from_string<'c>(
        str: &str,
        in_context: &'c Self::ContextType,
    ) -> Result<&'c Self, JSContextError>;
    // fn from_one_arg_closure<
    //     I: FromJSValue<'runtime, Self>,
    //     O: ToJSValue<'runtime, Self>,
    //     F: Fn(I) -> Result<O, JSContextError>,
    // >(
    //     closure: F,
    //     in_context: &Self::ContextType,
    // ) -> Result<&Self, JSContextError>;

    // fn undefined(in_context: &Self::ContextType) -> Result<Self, JSContextError>;

    // fn from_two_arg_closure<
    //     I1: FromJSValue<'runtime, Self>,
    //     I2: FromJSValue<'runtime, Self>,
    //     O: ToJSValue<'runtime, Self>,
    //     F: Fn(I1, I2) -> Result<O, JSContextError>,
    // >(
    //     closure: F,
    //     in_context: &Self::ContextType,
    // ) -> Result<&Self, JSContextError>;
    fn call(&self) -> Result<Self, JSContextError> {
        self.call_bound(Vec::new(), self)
    }
    fn call_with_arguments(&self, arguments: Vec<&Self>) -> Result<Self, JSContextError> {
        self.call_bound(arguments, self)
    }
    fn call_bound(&self, arguments: Vec<&Self>, bound_to: &Self) -> Result<Self, JSContextError>;

    fn get_property(&self, name: &str) -> Result<&Self, JSContextError>;
    // fn call_property(&self, name: &str) -> Result<Self, JSContextError> {
    //     self.call_property_with_arguments(name, Vec::new())
    // }
    // fn call_property_with_arguments(
    //     &self,
    //     name: &str,
    //     arguments: Vec<&Self>,
    // ) -> Result<Self, JSContextError>;

    fn from_raw(raw: Self::RawType, in_context: &Self::ContextType)
        -> Result<Self, JSContextError>;

    fn create_function<'c>(
        in_context: &'c Self::ContextType,
        arg_names: Vec<&str>,
        body: &str,
    ) -> Result<&'c Self, JSContextError>;

    // fn wrapping_native<NativeType>(
    //     native_object: NativeType,
    //     in_context: &Rc<Self::ContextType>,
    // ) -> Result<Self, JSContextError>;
}
