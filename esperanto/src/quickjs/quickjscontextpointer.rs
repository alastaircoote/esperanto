use quickjs_android_suitable_sys::JSContext;
use std::ops::Deref;

/// QuickJS doesn't have the ability to retain and release contexts
/// like JavaScriptCore does so we have to wrap our raw pointers in
/// this struct in order to retain info on whether this should be
/// freed when dropped
#[derive(Debug, Clone, Copy)]
pub struct QuickJSContextPointer {
    ptr: *mut JSContext,
    pub(super) free_on_drop: bool,
}

impl QuickJSContextPointer {
    pub(crate) fn wrap(ptr: *mut JSContext, free_on_drop: bool) -> Self {
        QuickJSContextPointer { ptr, free_on_drop }
    }
}

impl<'c> PartialEq for QuickJSContextPointer {
    fn eq(&self, other: &Self) -> bool {
        self.ptr == other.ptr
    }
}

impl<'c> Deref for QuickJSContextPointer {
    type Target = *mut JSContext;

    fn deref(&self) -> &Self::Target {
        &self.ptr
    }
}
