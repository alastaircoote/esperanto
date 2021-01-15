use super::jscore_runtime::JSCoreRuntime;
use crate::jsruntime::Runtime;
use crate::EsperantoResult;
use std::ops::Deref;

/// It's very common that the user will want to use a single JSContext by itself
/// without having to worry about the JSRuntime. In order to do that we need to be
/// able to store a JSRuntime OR a reference *to* a JSRuntime in the same place.
pub(super) enum JSCoreContextRuntimeStore<'r> {
    SelfContained(JSCoreRuntime<'r>),
    External(&'r JSCoreRuntime<'r>),
}

impl JSCoreContextRuntimeStore<'_> {
    /// Create a new runtime to be used by one context internally
    pub(super) fn new_self_contained() -> EsperantoResult<Self> {
        let new_runtime = JSCoreRuntime::new()?;
        Ok(JSCoreContextRuntimeStore::SelfContained(new_runtime))
    }
}

/// Let's make our lives easier by letting us use this enum as a JSRuntime without
/// any extra work
impl<'r> Deref for JSCoreContextRuntimeStore<'r> {
    type Target = JSCoreRuntime<'r>;

    fn deref(&self) -> &Self::Target {
        match self {
            JSCoreContextRuntimeStore::SelfContained(raw) => raw,
            JSCoreContextRuntimeStore::External(r) => *r,
        }
    }
}
