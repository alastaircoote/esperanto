use crate::errors::JSContextError;
use crate::traits::JSValue;
use std::{hash::Hash, os::raw::c_char, rc::Rc};

pub trait JSContext: Sized + Clone + 'static {
    type ValueType: JSValue<ContextType = Self> + 'static;

    fn evaluate(&self, script: &str) -> Result<Self::ValueType, JSContextError>;
    // Our FFI interfaces use C strings so it's a waste to convert to a Rust string then immediately
    // back again, so we'll include a convenience method for using C strings directly
    fn evaluate_cstring(&self, script: *const c_char) -> Result<Self::ValueType, JSContextError>;
    fn new() -> Result<Self, JSContextError>;
    fn compile_string<'a>(
        &self,
        script: *const std::os::raw::c_char,
    ) -> Result<&'a [u8], JSContextError>;
    fn eval_compiled(&self, binary: &[u8]) -> Result<Self::ValueType, JSContextError>;
}
