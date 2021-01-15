use std::ops::Deref;

use super::jscore_runtime::JSCoreRuntime;

pub(super) enum JSCoreContextRuntimeStore<'c> {
    SelfContained(*mut JSCoreRuntime<'c>),
    External(&'c JSCoreRuntime<'c>),
}

impl<'c> Drop for JSCoreContextRuntimeStore<'c> {
    fn drop(&mut self) {
        if let JSCoreContextRuntimeStore::SelfContained(runtime_raw) = self {
            let runtime = unsafe { Box::from_raw(*runtime_raw) };
        }
    }
}

impl<'c> Deref for JSCoreContextRuntimeStore<'c> {
    type Target = JSCoreRuntime<'c>;

    fn deref(&self) -> &Self::Target {
        match self {
            JSCoreContextRuntimeStore::SelfContained(raw) => unsafe { raw.as_ref() }.unwrap(),
            JSCoreContextRuntimeStore::External(r) => r,
        }
    }
}
