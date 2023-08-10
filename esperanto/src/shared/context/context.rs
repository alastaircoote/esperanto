use std::ffi::CString;
use std::marker::PhantomData;

use super::{context_error::JSContextError, evaluate_metadata::EvaluateMetadata};
use crate::shared::engine_impl::JSContextInternalImpl;
use crate::shared::value::ValueResult;
use crate::shared::{context::JSContextInternal, errors::EsperantoResult};
use crate::shared::{runtime::JSRuntime, value::JSValue};
use crate::Retain;

#[derive(Debug)]
enum JSRuntimeStore<'r> {
    StoredInternally(JSRuntime<'r>),
    Referenced(&'r JSRuntime<'r>),
}

/**
 * This is the environment in which we actually evaluate JavaScript. It can belong to a
 * JSRuntime (in which case you can transfer JSValues between contexts) or live in its own
 * runtime.
 */
#[derive(Debug)]
pub struct JSContext<'c> {
    // The engine-specific implementation of JSContext
    pub(crate) internal: JSContextInternalImpl,
    runtime: JSRuntimeStore<'c>,
}

impl<'r, 'c> JSContext<'c>
where
    'r: 'c,
{
    /// Convert our internal implementation into an Esperanto wrapper
    pub(crate) fn from_raw(
        raw: JSContextInternalImpl,
        with_runtime: &'r JSRuntime<'r>,
    ) -> JSContext<'c> {
        return JSContext {
            internal: raw,
            runtime: JSRuntimeStore::Referenced(with_runtime),
        };
    }

    pub(crate) fn from_raw_storing_runtime(
        raw: JSContextInternalImpl,
        with_runtime: JSRuntime<'c>,
    ) -> JSContext<'c> {
        JSContext {
            internal: raw,
            runtime: JSRuntimeStore::StoredInternally(with_runtime),
        }
    }
    /// Create a new JSContext in its own runtime
    pub fn new() -> EsperantoResult<Self> {
        let runtime = JSRuntime::new()?;
        let raw = JSContextInternalImpl::new_in_runtime(&runtime.internal)?;
        Ok(JSContext {
            internal: raw,
            runtime: JSRuntimeStore::StoredInternally(runtime),
        })
    }

    /// Create a JSContext in an existing runtime.
    /// # Arguments
    /// * `in_runtime`: A reference to the runtime we want to create a JSContext in.
    pub fn new_in_runtime(in_runtime: &'r JSRuntime<'r>) -> EsperantoResult<Self>
    where
        'r: 'c,
    {
        let raw = JSContextInternalImpl::new_in_runtime(&in_runtime.internal)?;
        Ok(JSContext {
            internal: raw,
            runtime: JSRuntimeStore::Referenced(in_runtime),
        })
    }

    /// Take a string, convert it into executable JavaScript, then execute it.
    ///
    /// # Arguments
    /// * `script`: The script you want to evaluate
    /// * `metadata`: Optional file metadata to be used during evaluation (used in stack traces
    ///               dev tools, etc)
    pub fn evaluate(
        &'c self,
        script: &str,
        metadata: Option<&EvaluateMetadata>,
    ) -> ValueResult<'c> {
        let len = script.len();
        let cstr = CString::new(script).map_err(|_| JSContextError::CouldNotParseScript)?;

        self.internal.evaluate(cstr, len, metadata).map(|internal| {
            let val = JSValue::wrap_internal(internal, self);
            Retain::wrap(val)
        })
    }

    /// Manually run garbage collection. Is hidden because not all implementations expose a public way
    /// to force garbage collection, but it is useful when testing
    #[doc(hidden)]
    pub fn garbage_collect(&self) {
        self.internal.garbage_collect()
    }

    /// Grab the global object of this JSContext. Useful when you want to add properties to it to make
    /// globally accessible in user-run code.
    //
    // TBD: should we instead maybe use a closure here and name it
    // with_global_object or something like that? Retaining a reference to the global object doesn't feel
    // like something we actually want people to do.
    pub fn global_object(&'c self) -> Retain<JSValue<'c>> {
        let raw = self.internal.get_globalobject();
        Retain::wrap(JSValue::wrap_internal(raw, self))
    }

    pub fn get_runtime(&'c self) -> &JSRuntime {
        match &self.runtime {
            JSRuntimeStore::StoredInternally(stored) => stored,
            JSRuntimeStore::Referenced(reference) => reference,
        }
    }
}

impl Drop for JSContext<'_> {
    fn drop(&mut self) {
        // The context is retained when created. We don't mess around with Retain<JSContext> as our
        // lifetimes cover the necessity for that. We still need to drop the retain when the context
        // itself is dropped though:
        self.internal.release();
    }
}
