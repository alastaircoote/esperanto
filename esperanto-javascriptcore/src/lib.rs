#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

extern crate esperanto_traits;
extern crate fragile;
use esperanto_traits::js_traits::{JSEnvError, JSRuntime, JSValue};
use fragile::Sticky;
use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::ffi::{CStr, CString};
use std::hash::Hasher;
use std::sync::{Arc, Mutex};

struct JSCRuntime {
    jsc_ref: *mut OpaqueJSContext,
}

struct JSCValue {
    jsc_ref: Sticky<JSValueRef>,
    runtime: Arc<JSCRuntime>,
}

impl JSCValue {
    fn get_ref(&self) -> Result<&JSValueRef, JSEnvError> {
        self.jsc_ref
            .try_get()
            .map_err(|_| JSEnvError::UsingWrongThread)
    }
}

// fn do_thing(item: JSValueRef) {
//     let h = DefaultHasher::new();
//     std::ptr::hash(item, &mut h);
//     h.finish()
// }

impl JSValue for JSCValue {
    fn to_string<'a>(&self) -> Result<&'a str, JSEnvError> {
        let jsc_ref = self.get_ref()?;
        let string_ref = JSValueToStringCopy(self.runtime.jsc_ref, jsc_ref, std::ptr::null_mut());
        let string_length = JSStringGetLength(string_ref);

        let mut bytes: Vec<u8> = vec![0; string_length];

        JSStringGetUTF8CString(string_ref, bytes, string_length);

        let cstr =
            CStr::from_bytes_with_nul(bytes).map_err(|_| JSEnvError::CouldNotConvertString)?;

        cstr.to_str().map_err(|_| JSEnvError::CouldNotConvertString);
    }
}

impl JSCRuntime {
    fn wrap_jsvalueref(&self, val_ref: JSValueRef, in_runtime: Arc<Self>) -> JSCValue {
        JSCValue {
            jsc_ref: Sticky::new(val_ref),
            runtime: in_runtime,
        }
    }
}

impl JSRuntime for JSCRuntime {
    type ValueType = JSCValue;

    fn new() -> Self {
        unsafe {
            let ctx = JSGlobalContextCreate(std::ptr::null_mut());
            let retained_ctx = JSGlobalContextRetain(ctx);

            JSCRuntime {
                jsc_ref: retained_ctx,
            }
        }
    }

    fn evaluate(&self, script: &str) -> Result<JSCValue, JSEnvError> {
        let script_c_string = CString::new(script).map_err(|_| JSEnvError::CouldNotParseScript)?;
        unsafe {
            let script_js_string = JSStringCreateWithUTF8CString(script_c_string.as_ptr());
            let return_value = JSEvaluateScript(
                self.jsc_ref,
                script_js_string,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                0,
                std::ptr::null_mut(),
            );

            Ok(self.wrap_jsvalueref(return_value))
        }
    }
}

impl Drop for JSCRuntime {
    fn drop(&mut self) {
        unsafe { JSGlobalContextRelease(self.jsc_ref) }
    }
}
