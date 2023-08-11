use std::marker::PhantomData;

use crate::shared::util::StoredOrReferenced;
use crate::shared::value::JSValueInternal;
use crate::JSRuntime;
use crate::{shared::engine_impl::ActiveJSContextImplementation, EsperantoResult};

use super::{JSContextError, JSContextImplementation};

type StoredOrReferencedRuntime<'r> = StoredOrReferenced<'r, JSRuntime<'r>>;

#[derive(Debug, Eq, PartialEq)]
pub(super) struct JSContextInternals<'r, 'c> {
    // The engine-specific implementation of JSContext
    pub(super) implementation: ActiveJSContextImplementation,
    pub(super) runtime: StoredOrReferencedRuntime<'r>,
    _lifetime: &'c PhantomData<()>,
}

impl<'r, 'c> JSContextInternals<'r, 'c> {
    pub(super) fn create_and_store(
        implementation: ActiveJSContextImplementation,
        runtime: StoredOrReferencedRuntime<'r>,
    ) -> Box<Self> {
        let boxed = Box::new(JSContextInternals {
            implementation,
            runtime,
            _lifetime: &PhantomData,
        });

        let raw: *const Self = boxed.as_ref();
        implementation.set_private_data(raw as _);
        boxed
    }

    pub(super) fn get_from_global_object(
        implementation: ActiveJSContextImplementation,
    ) -> EsperantoResult<&'c Self> {
        let raw = implementation.get_private_data()? as *const Self;
        unsafe { raw.as_ref() }.ok_or(JSContextError::CouldNotGetInternalRepresentation.into())
    }
}

impl Drop for JSContextInternals<'_, '_> {
    fn drop(&mut self) {
        self.implementation.release()
    }
}
