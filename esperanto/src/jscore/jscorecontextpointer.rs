use std::ops::Deref;

use javascriptcore_sys::OpaqueJSContext;

#[derive(Debug, Clone, Copy)]
pub enum JSCoreContextPointer {
    GlobalContext(*mut OpaqueJSContext),
    Context(*const OpaqueJSContext),
}

impl<'c> PartialEq for JSCoreContextPointer {
    fn eq(&self, other: &Self) -> bool {
        self.deref() == other.deref()
    }
}

impl Deref for JSCoreContextPointer {
    type Target = *const OpaqueJSContext;

    fn deref(&self) -> &Self::Target {
        match self {
            JSCoreContextPointer::GlobalContext(g) => unsafe { std::mem::transmute(g) },
            JSCoreContextPointer::Context(c) => c,
        }
    }
}

impl From<*mut OpaqueJSContext> for JSCoreContextPointer {
    fn from(val: *mut OpaqueJSContext) -> Self {
        JSCoreContextPointer::GlobalContext(val)
    }
}

impl From<*const OpaqueJSContext> for JSCoreContextPointer {
    fn from(val: *const OpaqueJSContext) -> Self {
        JSCoreContextPointer::Context(val)
    }
}
