use crate::qjs_runtime::QJSRuntime;
use esperanto_traits::JSValue;
use qjs_sys::{JSValue as QJSValueRef, JS_DupValue};
pub struct QJSValue {
    qjs_ref: QJSValueRef,
}

impl JSValue for QJSValue {
    pub fn new(value_ref: QJSValueRef, in_context: Rc<JSCGlobalContext>) -> Self {
        JS_DupValue(value_ref);
    }
}
