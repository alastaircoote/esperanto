use crate::{jsvalue::*, EsperantoError};

pub trait TryFromJSValueRef: Sized {
    fn try_from_js_ref(value: &JSValue) -> Result<Self, EsperantoError>;
}
