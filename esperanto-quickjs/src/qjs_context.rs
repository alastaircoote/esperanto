use crate::qjs_error::QJSError;
use crate::qjs_runtime::QJSRuntime;
use crate::qjs_value::QJSValue;
use esperanto_shared::errors::{JSContextError, JSConversionError, JSError};
use esperanto_shared::traits::{JSContext, JSValue};
use quickjs_android_suitable_sys::{
    js_free, JSContext as QJSRawContext, JS_Eval, JS_EvalFunction, JS_FreeContext, JS_FreeValue__,
    JS_NewContext, JS_ReadObject, JS_WriteObject, JS_EVAL_FLAG_COMPILE_ONLY, JS_EVAL_TYPE_GLOBAL,
    JS_READ_OBJ_BYTECODE, JS_WRITE_OBJ_BYTECODE,
};
use std::ffi::{c_void, CStr, CString};
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
    type ValueShareTarget = QJSRuntime;
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

    fn compile_string<'a>(
        self: &Rc<Self>,
        script: *const std::os::raw::c_char,
    ) -> Result<&'a [u8], JSContextError> {
        let fin = CString::new("file.js").unwrap();

        let cstr_len = unsafe { CStr::from_ptr(script).to_str()?.len() };

        let compile_js_val = unsafe {
            JS_Eval(
                self.raw,
                script,
                cstr_len,
                fin.as_ptr(),
                JS_EVAL_TYPE_GLOBAL as i32 + JS_EVAL_FLAG_COMPILE_ONLY as i32,
            )
        };

        JSError::check_for_exception(compile_js_val, &self)?;

        let mut byte_length: usize = 0;

        let stored_obj: *mut u8 = unsafe {
            JS_WriteObject(
                self.raw,
                &mut byte_length,
                compile_js_val,
                JS_WRITE_OBJ_BYTECODE as i32,
            )
        };

        unsafe { JS_FreeValue__(self.raw, compile_js_val) };

        let byte_array = unsafe { std::slice::from_raw_parts(stored_obj, byte_length) };

        // let mut copy = Vec::with_capacity(byte_length);
        // copy.resize(byte_length, 0);
        // copy.copy_from_slice(byte_array);

        // unsafe { js_free(self.raw, stored_obj as *mut c_void) };

        Ok(byte_array)
    }

    fn eval_compiled(self: &Rc<Self>, binary: &[u8]) -> Result<QJSValue, JSContextError> {
        let ptr = binary.as_ptr();
        let val =
            unsafe { JS_ReadObject(self.raw, ptr, binary.len(), JS_READ_OBJ_BYTECODE as i32) };

        let result = unsafe { JS_EvalFunction(self.raw, val) };

        QJSValue::from_raw(result, self)
    }

    fn get_value_share_target(&self) -> &Self::ValueShareTarget {
        &self.runtime
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

    #[test]
    fn it_compiles_code() {
        let ctx = QJSContext::new().unwrap();
        let script = "var test = 12345;";
        let c_string_scr = CString::new(script).unwrap();
        let byte_array = ctx.compile_string(c_string_scr.as_ptr()).unwrap();

        let new_ctx = QJSContext::new().unwrap();
        new_ctx.eval_compiled(byte_array).unwrap();
        let get_val = new_ctx.evaluate("test").unwrap();
        assert_eq!(get_val.as_number().unwrap(), 12345.0);
    }
}
