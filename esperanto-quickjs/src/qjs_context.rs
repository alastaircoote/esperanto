use crate::qjs_error::QJSError;
use crate::qjs_runtime::QJSRuntime;
use crate::qjs_value::QJSValue;
use esperanto_shared::errors::{JSContextError, JSConversionError, JSError};
use esperanto_shared::traits::{JSContext, JSValue};
use quickjs_android_suitable_sys::{
    JSContext as QJSRawContext, JS_Eval, JS_FreeContext, JS_NewContext, JS_EVAL_TYPE_GLOBAL,
};
use std::ffi::{CStr, CString};
use std::rc::Rc;
#[derive(Debug)]
pub struct QJSContext {
    pub(crate) raw: *mut QJSRawContext,
    pub(crate) runtime: Rc<QJSRuntime>,
}

impl Drop for QJSContext {
    fn drop(&mut self) {
        unsafe { JS_FreeContext(self.raw) }
    }
}

impl QJSContext {
    fn evaluate_cstring_with_len(
        self: &Rc<Self>,
        script: *const std::os::raw::c_char,
        script_len: usize,
    ) -> Result<QJSValue, JSContextError> {
        let fin = CString::new("file.js").unwrap();

        let result = unsafe {
            JS_Eval(
                self.raw,
                script,
                script_len,
                fin.as_ptr(),
                JS_EVAL_TYPE_GLOBAL as i32,
            )
        };

        JSError::check_for_exception(result, &self)?;

        QJSValue::from_raw(result, &self)
    }
}

impl JSContext for QJSContext {
    type ValueType = QJSValue;
    fn new() -> Result<Rc<Self>, JSContextError> {
        // need to add support for shared runtimes
        let runtime = Rc::new(QJSRuntime::new());
        let qjs_ref = unsafe { JS_NewContext(runtime.raw) };
        if qjs_ref.is_null() {
            return Err(JSContextError::CouldNotCreateContext);
        }
        Ok(Rc::new(QJSContext {
            raw: qjs_ref,
            runtime,
        }))
    }

    fn evaluate(self: &Rc<Self>, script: &str) -> Result<QJSValue, JSContextError> {
        let script_as_c_string =
            CString::new(script).map_err(|e| JSConversionError::ConversionToCStringFailed(e))?;

        self.evaluate_cstring_with_len(script_as_c_string.as_ptr(), script.len())
    }
    fn evaluate_cstring(
        self: &Rc<Self>,
        script: *const std::os::raw::c_char,
    ) -> Result<Self::ValueType, JSContextError> {
        let cstr_len = unsafe { CStr::from_ptr(script).to_str()?.len() };
        self.evaluate_cstring_with_len(script, cstr_len)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use esperanto_shared::trait_tests::jscontext_tests;

    #[test]
    fn it_evaluates_correct_code() {
        jscontext_tests::it_evaluates_correct_code::<QJSContext>();
    }

    #[test]
    fn it_throws_exceptions_on_invalid_code() {
        jscontext_tests::it_throws_exceptions_on_invalid_code::<QJSContext>();
    }
}
