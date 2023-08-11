use std::ffi::CString;
use std::marker::PhantomData;

use super::{context_error::JSContextError, evaluate_metadata::EvaluateMetadata};
use crate::shared::engine_impl::ActiveJSContextImplementation;
use crate::shared::util::StoredOrReferenced;
use crate::shared::value::ValueResult;
use crate::shared::{context::JSContextImplementation, errors::EsperantoResult};
use crate::shared::{runtime::JSRuntime, value::JSValue};
use crate::Retain;

type StoredOrReferencedRuntime<'r> = StoredOrReferenced<'r, JSRuntime<'r>>;

/**
 * This is the environment in which we actually evaluate JavaScript. It can belong to a
 * JSRuntime (in which case you can transfer JSValues between contexts) or live in its own
 * runtime.
 */
#[derive(Debug, Eq, PartialEq)]
pub struct JSContext<'r, 'c> {
    // The engine-specific implementation of JSContext
    pub(super) implementation: ActiveJSContextImplementation,
    pub(super) runtime: StoredOrReferencedRuntime<'r>,
    _lifetime: &'c PhantomData<()>,
}

impl<'r, 'c> JSContext<'r, 'c>
where
    'r: 'c,
{
    fn create_and_store_in_implementation(
        runtime: StoredOrReferencedRuntime<'r>,
    ) -> EsperantoResult<Box<Self>> {
        let implementation = ActiveJSContextImplementation::new_in_runtime(&runtime.internal)?;
        let ctx = JSContext {
            implementation,
            runtime: runtime.into(),
            _lifetime: &PhantomData,
        };

        let boxed_context = Box::new(ctx);
        let ptr_to_box: *const JSContext = boxed_context.as_ref();
        implementation.set_private_data(ptr_to_box as _)?;
        Ok(boxed_context)
    }

    /// Create a new JSContext in its own runtime
    pub fn new() -> EsperantoResult<Box<Self>> {
        let runtime = Box::new(JSRuntime::new()?);
        Self::create_and_store_in_implementation(runtime.into())
    }

    /// Create a JSContext in an existing runtime.
    /// # Arguments
    /// * `in_runtime`: A reference to the runtime we want to create a JSContext in.
    pub fn new_in_runtime(in_runtime: &'r JSRuntime<'r>) -> EsperantoResult<Box<Self>>
    where
        'r: 'c,
    {
        Self::create_and_store_in_implementation(in_runtime.into())
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
        self.implementation
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
        &self.runtime
    }
}

/// Internal functions

impl<'r, 'c> JSContext<'r, 'c>
where
    'r: 'c,
{
    pub(crate) fn borrow_from_implementation(
        ptr: ActiveJSContextImplementation,
    ) -> EsperantoResult<&'c Self> {
        let raw = ptr.get_private_data()? as *const Self;
        unsafe { raw.as_ref() }.ok_or(JSContextError::CouldNotGetInternalRepresentation.into())
    }
}

impl Drop for JSContext<'_, '_> {
    fn drop(&mut self) {
        self.implementation().release()
    }
}
