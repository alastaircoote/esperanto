use crate::errors::JSContextError;
use crate::traits::JSValue;
use std::{os::raw::c_char, rc::Rc};
pub trait JSContext: Sized + 'static {
    type ValueType: JSValue<ContextType = Self> + 'static;
    fn evaluate(self: &Rc<Self>, script: &str) -> Result<Self::ValueType, JSContextError>;
    // Our FFI interfaces use C strings so it's a waste to convert to a Rust string then immediately
    // back again, so we'll include a convenience method for using C strings directly
    fn evaluate_cstring(
        self: &Rc<Self>,
        script: *const c_char,
    ) -> Result<Self::ValueType, JSContextError>;
    fn new() -> Result<Rc<Self>, JSContextError>;
}

pub trait RawBackedJSContext: JSContext {
    type RawValueType;
}
