use std::ffi::CString;

use super::{context_error::JSContextError, evaluate_metadata::EvaluateMetadata};
use crate::shared::runtime::JSRuntimeInternal;
use crate::shared::{errors::EsperantoError, value::JSValueInternal};

pub(crate) trait JSContextInternal: Sized {
    type RuntimeType: JSRuntimeInternal;
    type ValueType: JSValueInternal;
    fn new_in_runtime(runtime: Self::RuntimeType) -> Result<Self, JSContextError>;
    fn evaluate(
        self,
        script: CString,
        script_size: usize,
        metadata: Option<&EvaluateMetadata>,
    ) -> Result<Self::ValueType, EsperantoError>;
    // fn retain(&self) -> Self;
    fn release(self);
    fn get_runtime(self) -> Self::RuntimeType;
    fn get_globalobject(self) -> Self::ValueType;
    fn garbage_collect(self);
}
