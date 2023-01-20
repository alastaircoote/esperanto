use std::ffi::CString;

use super::{context_error::JSContextError, evaluate_metadata::EvaluateMetadata};
use crate::shared::{context::JSContextInternal, errors::EsperantoResult};
use crate::shared::{engine_impl::JSContextInternalImpl, errors::EsperantoError};
use crate::shared::{runtime::JSRuntime, value::JSValueRef};

#[derive(Debug)]
pub struct JSContext<'c> {
    pub(crate) internal: JSContextInternalImpl,
    stored_runtime: Option<JSRuntime<'c>>,
    // _lifetime: PhantomData<&'c ()>,
}

impl<'c> JSContext<'c> {
    pub fn new() -> EsperantoResult<Self> {
        let runtime = JSRuntime::new()?;
        let raw = JSContextInternalImpl::new_in_runtime(runtime.internal)?;

        Ok(JSContext {
            internal: raw,
            stored_runtime: Some(runtime),
            // _lifetime: PhantomData,
        })
    }

    pub fn new_in_runtime<'r>(in_runtime: &'r JSRuntime<'r>) -> EsperantoResult<Self>
    where
        'r: 'c,
    {
        let raw = JSContextInternalImpl::new_in_runtime(in_runtime.internal)?;

        Ok(JSContext {
            internal: raw,
            stored_runtime: None,
            // _lifetime: PhantomData,
        })
    }

    pub fn evaluate<'new>(
        &'c self,
        script: &str,
        metadata: Option<&EvaluateMetadata>,
    ) -> Result<JSValueRef<'new>, EsperantoError>
    where
        'c: 'new,
    {
        let len = script.len();
        let cstr = CString::new(script).map_err(|_| JSContextError::CouldNotParseScript)?;

        self.internal.evaluate(cstr, len, metadata).map(|internal| {
            let val = JSValueRef::wrap_internal(internal, self);
            val
        })
    }

    // pub fn retain_value<'i>(
    //     &self,
    //     value: &'i JSValueRef<'i>,
    // ) -> Result<Retain<JSValueRef>, EsperantoError> {
    //     let extended_lifetime = value.extend_lifetime(&self)?;
    //     Ok(Retain::new(extended_lifetime))
    // }

    pub fn garbage_collect(&self) {
        self.internal.garbage_collect()
    }

    pub fn global_object<'new>(&'c self) -> JSValueRef<'new>
    where
        'c: 'new,
    {
        let raw = self.internal.get_globalobject();
        JSValueRef::wrap_internal(raw, self)
    }

    pub fn throw_error(&self, err: EsperantoError) -> EsperantoResult<()> {
        let error = JSValueRef::new_error("EsperantoError", &err.to_string(), &self)?;
        self.internal.throw_error(error.internal);
        Ok(())
    }
}

impl Drop for JSContext<'_> {
    fn drop(&mut self) {
        if let Some(_) = &self.stored_runtime {
            // this is really just here to stop a compiler error about stored_runtime
            // never being used.
        }
        self.internal.release();
    }
}

impl<'c, T> From<T> for JSContext<'c>
where
    JSContextInternalImpl: From<T>,
{
    fn from(val: T) -> Self {
        let internal = JSContextInternalImpl::from(val);
        JSContext {
            internal,
            stored_runtime: None,
        }
    }
}
