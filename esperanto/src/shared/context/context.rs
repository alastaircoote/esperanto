use std::ffi::CString;
use std::marker::PhantomData;

use super::context_internals::JSContextInternals;
use super::{context_error::JSContextError, evaluate_metadata::EvaluateMetadata};
use crate::shared::engine_impl::ActiveJSContextImplementation;
use crate::shared::util::StoredOrReferenced;
use crate::shared::value::{JSValueInternal, ValueResult};
use crate::shared::{context::JSContextImplementation, errors::EsperantoResult};
use crate::shared::{runtime::JSRuntime, value::JSValue};
use crate::Retain;

/**
 * This is the environment in which we actually evaluate JavaScript. It can belong to a
 * JSRuntime (in which case you can transfer JSValues between contexts) or live in its own
 * runtime.
 */
#[derive(Debug, Eq, PartialEq)]
pub struct JSContext<'r, 'c> {
    internals: StoredOrReferenced<'c, JSContextInternals<'r, 'c>>,
}

impl<'r, 'c> JSContext<'r, 'c>
where
    'c: 'r,
{
    /// Create a new JSContext in its own runtime
    pub fn new() -> EsperantoResult<Self> {
        let runtime = Box::new(JSRuntime::new()?);
        let implementation = ActiveJSContextImplementation::new_in_runtime(&runtime.internal)?;
        let internals = JSContextInternals::create_and_store(implementation, runtime.into());

        Ok(JSContext {
            internals: internals.into(),
        })
    }

    /// Create a JSContext in an existing runtime.
    /// # Arguments
    /// * `in_runtime`: A reference to the runtime we want to create a JSContext in.
    pub fn new_in_runtime(in_runtime: &'r JSRuntime<'r>) -> EsperantoResult<Self>
    where
        'r: 'c,
    {
        let implementation = ActiveJSContextImplementation::new_in_runtime(&in_runtime.internal)?;
        let internals = JSContextInternals::create_and_store(implementation, in_runtime.into());

        Ok(JSContext {
            internals: internals.into(),
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
    ) -> ValueResult<'r, 'c> {
        let len = script.len();
        let cstr = CString::new(script).map_err(|_| JSContextError::CouldNotParseScript)?;

        self.implementation()
            .evaluate(cstr, len, metadata)
            .map(|internal| {
                let val = JSValue::wrap_internal(internal, self);
                Retain::wrap(val)
            })
    }

    pub(crate) fn implementation(&self) -> ActiveJSContextImplementation {
        self.internals.implementation
    }

    /// Manually run garbage collection. Is hidden because not all implementations expose a public way
    /// to force garbage collection, but it is useful when testing
    #[doc(hidden)]
    pub fn garbage_collect(&self) {
        self.implementation().garbage_collect()
    }

    /// Grab the global object of this JSContext. Useful when you want to add properties to it to make
    /// globally accessible in user-run code.
    //
    // TBD: should we instead maybe use a closure here and name it
    // with_global_object or something like that? Retaining a reference to the global object doesn't feel
    // like something we actually want people to do.
    pub fn global_object(&'c self) -> Retain<JSValue<'r, 'c>> {
        let raw = self.implementation().get_globalobject();
        Retain::wrap(JSValue::wrap_internal(raw, self))
    }

    pub fn get_runtime(&'c self) -> &JSRuntime {
        &self.internals.runtime
    }
}

/// Internal functions

impl<'r, 'c> JSContext<'r, 'c>
where
    'r: 'c,
{
    pub(crate) fn wrap_raw<'lifetime>(
        reference: ActiveJSContextImplementation,
    ) -> EsperantoResult<Self> {
        Ok(JSContext {
            internals: JSContextInternals::get_from_global_object(reference)?.into(),
        })
    }
}
