use super::{
    jsconvertible::{FromJSValue, ToJSValue},
    JSContext,
};
use crate::errors::JSContextError;
use std::rc::Rc;
pub trait JSValue
where
    Self: Sized + 'static,
{
    type ContextType: JSContext<ValueType = Self> + 'static;
    type RawType: Copy;
    fn as_string(&self) -> Result<String, JSContextError>;
    fn to_object(self) -> Result<<Self::ContextType as JSContext>::ObjectType, JSContextError>;
    fn as_number(&self) -> Result<f64, JSContextError>;
    fn from_number(number: f64, in_context: &Rc<Self::ContextType>) -> Self;
    fn from_one_arg_closure<
        I: FromJSValue<Self> + 'static,
        O: ToJSValue<Self> + 'static,
        F: Fn(I) -> Result<O, JSContextError> + 'static,
    >(
        closure: F,
        in_context: &Rc<Self::ContextType>,
    ) -> Self;

    fn from_two_arg_closure<
        I1: FromJSValue<Self> + 'static,
        I2: FromJSValue<Self> + 'static,
        O: ToJSValue<Self> + 'static,
        F: Fn(I1, I2) -> Result<O, JSContextError> + 'static,
    >(
        closure: F,
        in_context: &Rc<Self::ContextType>,
    ) -> Self;
    fn call(&self) -> Self;
    fn call_with_arguments(&self, arguments: Vec<&Self>) -> Self;
    fn call_bound(&self, arguments: Vec<&Self>, bound_to: &Self) -> Self;
    // fn from_bool(bool: bool, in_context: &Self::ContextType) -> Self;
    // fn get_property(&self, name: &str) -> Result<Self, JSEnvError>;

    fn from_raw(raw: Self::RawType, in_context: &Rc<Self::ContextType>) -> Self;
}
