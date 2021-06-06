use quickjs_android_suitable_sys::{JSRuntime as QuickJSRuntime, JS_FreeRuntime, JS_NewRuntime};

use crate::shared::runtime::{JSRuntimeError, JSRuntimeInternal};

pub type QuickJSRuntimeInternal = *mut QuickJSRuntime;

impl JSRuntimeInternal for QuickJSRuntimeInternal {
    fn new() -> Result<Self, JSRuntimeError> {
        let runtime = unsafe { JS_NewRuntime() };
        if runtime.is_null() {
            return Err(JSRuntimeError::CouldNotCreateRuntime);
        }
        Ok(runtime)
    }

    fn release(self) {
        println!("FREE RUNTIME");
        unsafe { JS_FreeRuntime(self) }
    }
}
