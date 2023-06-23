use std::marker::PhantomData;

use crate::shared::engine_impl::JSRuntimeInternalImpl;
use crate::shared::runtime::runtime_internal::JSRuntimeInternal;

use super::runtime_error::JSRuntimeError;

#[derive(Debug)]
pub struct JSRuntime<'r> {
    pub(crate) internal: JSRuntimeInternalImpl,
    _lifetime: PhantomData<&'r ()>,
    release_on_drop: bool,
}

impl<'r> JSRuntime<'r> {
    pub fn new() -> Result<Self, JSRuntimeError> {
        let new_runtime = JSRuntimeInternalImpl::new()?;
        // let retained = new_runtime.retain();
        Ok(JSRuntime {
            internal: new_runtime,
            _lifetime: PhantomData,
            release_on_drop: true,
        })
    }

    pub(crate) fn from_raw(raw: JSRuntimeInternalImpl, release_on_drop: bool) -> Self {
        JSRuntime {
            internal: raw,
            _lifetime: PhantomData,
            release_on_drop,
        }
    }
}

impl Drop for JSRuntime<'_> {
    fn drop(&mut self) {
        if self.release_on_drop {
            self.internal.release()
        }
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
