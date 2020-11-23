use crate::errors::JSContextError;
use crate::traits::JSValue;
use std::{hash::Hash, os::raw::c_char, rc::Rc};

pub trait JSContext<'context>: Sized {
    type ValueType: JSValue<'context, ContextType = Self>;

    // Both JSC and QuickJS allow you to share JSValues between context, provided they have the same
    // JSRuntime (QJS) or ContextGroup (JSC). But not all engines will necessarily support this, so this
    // type alias allows us to specify at what level things can be shared. For another engine this might
    // link straight back to Self.
    type ValueShareTarget: Hash;

    fn evaluate(self: &Rc<Self>, script: &str) -> Result<Self::ValueType, JSContextError>;
    // Our FFI interfaces use C strings so it's a waste to convert to a Rust string then immediately
    // back again, so we'll include a convenience method for using C strings directly
    fn evaluate_cstring(
        self: &Rc<Self>,
        script: *const c_char,
    ) -> Result<Self::ValueType, JSContextError>;
    fn new() -> Result<Rc<Self>, JSContextError>;
    fn compile_string<'a>(
        self: &Rc<Self>,
        script: *const std::os::raw::c_char,
    ) -> Result<&'a [u8], JSContextError>;
    fn eval_compiled(self: &Rc<Self>, binary: &[u8]) -> Result<Self::ValueType, JSContextError>;
    fn get_value_share_target(&self) -> &Self::ValueShareTarget;
}
