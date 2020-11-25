use super::{
    jsconvertible::{FromJSValue, ToJSValue},
    JSContext,
};
use crate::errors::JSContextError;
pub trait JSValue
where
    Self: Sized + 'static,
{
    type ContextType: JSContext<ValueType = Self> + 'static;
    type RawType: Copy;
    fn as_string(&self) -> Result<String, JSContextError>;
    fn as_number(&self) -> Result<f64, JSContextError>;
    fn as_bool(&self) -> Result<bool, JSContextError>;
    fn from_number(number: f64, in_context: &Self::ContextType) -> Result<Self, JSContextError>;
    fn from_bool(bool: bool, in_context: &Self::ContextType) -> Result<Self, JSContextError>;
    fn from_string(str: &str, in_context: &Self::ContextType) -> Result<Self, JSContextError>;
    fn from_one_arg_closure<
        I: FromJSValue<Self>,
        O: ToJSValue<Self> + 'static,
        F: Fn(I) -> Result<O, JSContextError> + 'static,
    >(
        closure: F,
        in_context: &Self::ContextType,
    ) -> Result<Self, JSContextError>;

    fn undefined(in_context: &Self::ContextType) -> Result<Self, JSContextError>;

    fn from_two_arg_closure<
        I1: FromJSValue<Self> + 'static,
        I2: FromJSValue<Self> + 'static,
        O: ToJSValue<Self> + 'static,
        F: Fn(I1, I2) -> Result<O, JSContextError> + 'static,
    >(
        closure: F,
        in_context: &Self::ContextType,
    ) -> Result<Self, JSContextError>;
    fn call(&self) -> Result<Self, JSContextError> {
        self.call_bound(Vec::new(), self)
    }
    fn call_with_arguments(&self, arguments: Vec<&Self>) -> Result<Self, JSContextError> {
        self.call_bound(arguments, self)
    }
    fn call_bound(&self, arguments: Vec<&Self>, bound_to: &Self) -> Result<Self, JSContextError>;

    fn get_property(&self, name: &str) -> Result<Self, JSContextError>;
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

    fn create_function(
        in_context: &Self::ContextType,
        arg_names: Vec<&str>,
        body: &str,
    ) -> Result<Self, JSContextError>;

    // fn wrapping_native<NativeType>(
    //     native_object: NativeType,
    //     in_context: &Rc<Self::ContextType>,
    // ) -> Result<Self, JSContextError>;
}
