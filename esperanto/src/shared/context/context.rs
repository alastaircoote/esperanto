use std::ffi::CString;

use super::{context_error::JSContextError, evaluate_metadata::EvaluateMetadata};
use crate::shared::engine_impl::JSContextInternalImpl;
use crate::shared::value::ValueResult;
use crate::shared::{context::JSContextInternal, errors::EsperantoResult};
use crate::shared::{runtime::JSRuntime, value::JSValue};
use crate::Retain;

/**
 * This is the environment in which we actually evaluate JavaScript. It can belong to a
 * JSRuntime (in which case you can transfer JSValues between contexts) or live in its own
 * runtime.
 */
#[derive(Debug)]
pub struct JSContext<'c> {
    // The engine-specific implementation of JSContext
    pub(crate) internal: JSContextInternalImpl,

    // It's very common to want to spin up a JSContext without caring about the JSRuntime
    // it lives inside (e.g. using JSContext::new()). In that instance we do still have to
    // create a runtime but rather than having the user be concerned about it we store it
    // alongside the context with the same lifetime as the context.
    stored_runtime: Option<JSRuntime<'c>>,
}

impl<'c> JSContext<'c> {
    // Convert our internal implementation into an Esperanto wrapper
    pub(crate) fn from_raw(
        raw: JSContextInternalImpl,
        with_runtime: Option<JSRuntime<'c>>,
    ) -> Self {
        return JSContext {
            internal: raw,
            stored_runtime: with_runtime,
        };
    }

    // Create a new JSContext in its own runtime
    pub fn new() -> EsperantoResult<Self> {
        let runtime = JSRuntime::new()?;
        let raw = JSContextInternalImpl::new_in_runtime(runtime.internal)?;

        Ok(Self::from_raw(raw, Some(runtime)))
    }

    // Create a JSContext in an existing runtime. Context lifetime is specified as being shorter
    // or equivalent to runtime lifetime.
    pub fn new_in_runtime<'r>(in_runtime: &'r JSRuntime<'r>) -> EsperantoResult<Self>
    where
        'r: 'c,
    {
        let raw = JSContextInternalImpl::new_in_runtime(in_runtime.internal)?;

        Ok(Self::from_raw(raw, None))
    }

    // Actually execute some code. Optionally you can provide file metadata for the script you're evaluating,
    // this will appear in stack traces, dev tools and so on.
    pub fn evaluate(
        &'c self,
        script: &str,
        metadata: Option<&EvaluateMetadata>,
    ) -> ValueResult<'c> {
        let len = script.len();
        let cstr = CString::new(script).map_err(|_| JSContextError::CouldNotParseScript)?;

        self.internal.evaluate(cstr, len, metadata).map(|internal| {
            let val = JSValue::wrap_internal(internal, self);
            Retain::new(val, true)
        })
    }

    /// Manually run garbage collection. Isn't public because not all implementations expose a public way
    /// to force garbage collection, but it is useful when testing
    #[doc(hidden)]
    pub fn garbage_collect(&self) {
        self.internal.garbage_collect()
    }

    /// Grab the global object of this JSContext. Useful when you want to add properties to it to make
    /// globally accessible in user-run code. TBD: should we instead maybe use a closure here and name it
    /// with_global_object or something like that? Retaining a reference to the global object doesn't feel
    /// like something we actually want people to do.
    pub fn global_object(&'c self) -> Retain<JSValue<'c>> {
        let raw = self.internal.get_globalobject();
        Retain::new(JSValue::wrap_internal(raw, self), true)
    }

    // Commenting out ability to throw an error because JSCore doesn't actually allow us to do this:

    // pub fn throw_error(&self, err: EsperantoError) -> EsperantoResult<()> {
    //     let error = JSValue::new_error("EsperantoError", &err.to_string(), &self)?;
    //     self.internal.throw_error(error.internal);
    //     Ok(())
    // }
}

impl Drop for JSContext<'_> {
    fn drop(&mut self) {
        if let Some(runtime) = &self.stored_runtime {
            // this is really just here to stop a compiler error about stored_runtime
            // never being used, it would be automatically dropped when the context
            // is dropped anyway.
            drop(runtime)
        }

        // The context is retained when created. We don't mess around with Retain<JSContext> as our
        // lifetimes cover the necessity for that. We still need to drop the retain when the context
        // itself is dropped though:
        self.internal.release();
    }
}

// impl<'c, T> From<T> for JSContext<'c>
// where
//     JSContextInternalImpl: From<T>,
// {
//     fn from(val: T) -> Self {
//         let internal = JSContextInternalImpl::from(val);
//         JSContext {
//             internal,
//             stored_runtime: None,
//         }
//     }
// }
