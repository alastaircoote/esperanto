use std::ffi::CString;

use quickjs_android_suitable_sys::{
    JS_Eval, JS_FreeContext, JS_GetGlobalObject, JS_GetRuntime, JS_NewContext, JS_RunGC,
    JS_ThrowInternalError, JS_EVAL_TYPE_GLOBAL,
};

use super::quickjscontextpointer::QuickJSContextPointer;
use super::quickjsruntime::QuickJSRuntimeInternal;
use crate::shared::{
    context::{EvaluateMetadata, JSContextError, JSContextInternal},
    errors::EsperantoResult,
};

use super::quickjsvalue::QuickJSValueInternal;

// pub(crate) type QuickJSContextInternal = *mut QuickJSContext;
pub(crate) type QuickJSContextInternal = QuickJSContextPointer;

static PLACEHOLDER_FILENAME: &[u8] = b"<unknown>\0";

impl JSContextInternal for QuickJSContextInternal {
    type RuntimeType = QuickJSRuntimeInternal;
    type ValueType = QuickJSValueInternal;

    fn new_in_runtime(runtime: Self::RuntimeType) -> Result<Self, JSContextError> {
        let raw = unsafe { JS_NewContext(runtime) };
        match raw.is_null() {
            true => Err(JSContextError::CouldNotCreateContext),
            false => Ok(QuickJSContextPointer::wrap_retained(raw)),
        }
    }

    fn evaluate(
        self,
        script: CString,
        script_size: usize,
        metadata: Option<&EvaluateMetadata>,
    ) -> EsperantoResult<Self::ValueType> {
        // let line_number = metadata.map(|m| m.line_number).unwrap_or(0);
        let filename = metadata
            .map(|m| m.filename.as_ptr())
            .unwrap_or(PLACEHOLDER_FILENAME.as_ptr() as *const i8);

        check_quickjs_exception!(self => {
            unsafe {
                JS_Eval(
                    *self,
                    script.as_ptr(),
                    script_size,
                    filename,
                    JS_EVAL_TYPE_GLOBAL as i32,
                )
            }
        })
    }

    fn release(self) {
        if self.retained {
            unsafe { JS_FreeContext(*self) }
        }
    }

    fn get_runtime(self) -> Self::RuntimeType {
        unsafe { JS_GetRuntime(*self) }
    }

    fn garbage_collect(self) {
        unsafe { JS_RunGC(self.get_runtime()) }
    }

    fn get_globalobject(self) -> Self::ValueType {
        let obj = unsafe { JS_GetGlobalObject(*self) };
        obj.into()
    }

    fn throw_error(self, err: crate::EsperantoError) {
        let err_as_string = err.to_string();
        unsafe { JS_ThrowInternalError(*self, err_as_string.as_ptr() as _) };
    }
}
