use std::marker::PhantomData;

use crate::shared::engine_impl::JSRuntimeInternalImpl;
use crate::shared::runtime::runtime_implementation::JSRuntimeImplementation;

use super::runtime_error::JSRuntimeError;

#[derive(Debug, PartialEq, Eq)]
pub struct JSRuntime<'r> {
    implementation: JSRuntimeInternalImpl,
    _lifetime: PhantomData<&'r ()>,
}

impl<'r> JSRuntime<'r> {
    pub fn new() -> Result<Self, JSRuntimeError> {
        let new_runtime = JSRuntimeInternalImpl::new()?;
        // let retained = new_runtime.retain();
        Ok(JSRuntime {
            implementation: new_runtime,
            _lifetime: PhantomData,
        })
    }

    pub(crate) fn implementation(&self) -> &JSRuntimeInternalImpl {
        &self.implementation
    }
}

impl Drop for JSRuntime<'_> {
    fn drop(&mut self) {
        self.implementation.release()
    }
}

#[cfg(test)]
mod test {
    use super::JSRuntime;

    #[test]
    fn create_runtime_successfully() {
        JSRuntime::new().unwrap();
    }
}
