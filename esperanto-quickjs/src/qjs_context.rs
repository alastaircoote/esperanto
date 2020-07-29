use crate::qjs_runtime::QJSRuntime;
use crate::qjs_value::QJSValue;
use crate::{qjs_error::QJSError, qjs_shared_context_ref::SharedQJSContextRef};
use esperanto_shared::errors::{JSContextError, JSConversionError, JSError};
use esperanto_shared::traits::JSContext;
use libquickjs_sys::{JS_Eval, JS_NewContext, JS_EVAL_TYPE_GLOBAL};
use std::ffi::CString;
use std::rc::Rc;

pub struct QJSContext {
    pub(crate) context_ref: Rc<SharedQJSContextRef>,
}

impl JSContext for QJSContext {
    type ValueType = QJSValue;
    type ObjectType = QJSValue;
    type SharedRef = Rc<SharedQJSContextRef>;
    fn new() -> Result<Self, JSContextError> {
        // need to add support for shared runtimes
        let runtime = Rc::new(QJSRuntime::new());
        let qjs_ref = unsafe { JS_NewContext(runtime.qjs_ref) };
        if qjs_ref.is_null() {
            return Err(JSContextError::CouldNotCreateContext);
        }
        Ok(QJSContext {
            context_ref: Rc::new(SharedQJSContextRef::new(qjs_ref, runtime)),
        })
    }

    fn evaluate(&self, script: &str) -> Result<QJSValue, JSContextError> {
        let script_as_c_string =
            CString::new(script).map_err(|e| JSConversionError::ConversionToCStringFailed(e))?;

        let fin = CString::new("file.js").unwrap();

        let result = unsafe {
            JS_Eval(
                self.context_ref.qjs_ref,
                script_as_c_string.as_ptr(),
                script.len() as _,
                fin.as_ptr(),
                JS_EVAL_TYPE_GLOBAL as i32,
            )
        };

        JSError::check_for_exception(result, &self.context_ref)?;

        Ok(QJSValue::new(result, &self.context_ref))
    }

    fn get_shared_ref(&self) -> &Self::SharedRef {
        &self.context_ref
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
