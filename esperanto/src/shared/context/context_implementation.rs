use std::ffi::{c_void, CString};

use super::{context_error::JSContextError, evaluate_metadata::EvaluateMetadata};
use crate::shared::runtime::JSRuntimeInternal;
use crate::shared::{errors::EsperantoError, value::JSValueInternal};
use crate::EsperantoResult;

pub(crate) trait JSContextImplementation: Sized {
    type RuntimeType: JSRuntimeInternal;
    type ValueType: JSValueInternal;
    fn new_in_runtime(runtime: &Self::RuntimeType) -> Result<Self, JSContextError>;
    fn evaluate(
        self,
        script: CString,
        script_size: usize,
        metadata: Option<&EvaluateMetadata>,
    ) -> Result<Self::ValueType, EsperantoError>;
    fn release(self);
    // fn get_runtime(self) -> Self::RuntimeType;
    fn get_globalobject(self) -> Self::ValueType;
    fn garbage_collect(self);

    fn get_private_data(self) -> EsperantoResult<*mut c_void>;
    fn set_private_data(self, data: *mut c_void) -> EsperantoResult<()>;
}
