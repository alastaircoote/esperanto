#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

extern crate esperanto_traits;
use esperanto_traits::js_traits::{JSConversionError, JSEnvError, JSRuntime, JSValue};
use slotmap::{DefaultKey, SecondaryMap, SlotMap};
use std::ffi::{CStr, CString};
use std::rc::Rc;

struct JSCGlobalContext {
    jsc_ref: *mut OpaqueJSContext,
}

impl Drop for JSCGlobalContext {
    fn drop(&mut self) {
        unsafe { JSGlobalContextRelease(self.jsc_ref) }
    }
}

struct JSCRuntime {
    // jsc_ref: *mut OpaqueJSContext,
    context: Rc<JSCGlobalContext>,
    // This is really messy but slotmap requires that values implement Copy, which we can't
    // do because JSValueRef isn't copy-safe. So instead we use a SecondaryMap which CAN
    // store non-Copy items to store our actual values.
    value_initial_store: SlotMap<DefaultKey, ()>,
    value_actual_store: SecondaryMap<DefaultKey, JSCValue>,
}
struct JSCValue {
    jsc_ref: JSValueRef,
    context: Rc<JSCGlobalContext>,
}

impl JSValue for JSCValue {
    fn to_string<'b>(&self) -> Result<&'b str, JSEnvError> {
        unsafe {
            let string_ref =
                JSValueToStringCopy(self.context.jsc_ref, self.jsc_ref, std::ptr::null_mut());
            let string_length = JSStringGetLength(string_ref);

            if string_length > usize::MAX as u64 {
                // If we're on a 32 bit archiecture this could theoretically get too big. It really,
                // really shouldn't ever happen though.
                return Err(JSEnvError::ConversionError(
                    JSConversionError::StringWasTooLong,
                ));
            }

            let mut bytes: Vec<i8> = vec![0; string_length as usize];

            JSStringGetUTF8CString(string_ref, bytes.as_mut_ptr(), string_length);

            let cstr = CStr::from_ptr(bytes.as_ptr());
            cstr.to_str().map_err(|_| {
                JSEnvError::ConversionError(
                    JSConversionError::CouldNotConvertStringToSuitableFormat,
                )
            })
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
                // jsc_ref: retained_ctx,
                context: Rc::new(JSCGlobalContext {
                    jsc_ref: retained_ctx,
                }),
                value_initial_store: SlotMap::new(),
                value_actual_store: SecondaryMap::new(),
            }
        }
    }

    fn evaluate<O: From<JSCValue>>(&self, script: &str) -> Result<O, JSEnvError> {
        let script_c_string = CString::new(script).map_err(|_| JSEnvError::CouldNotParseScript)?;
        unsafe {
            let script_js_string = JSStringCreateWithUTF8CString(script_c_string.as_ptr());
            let return_value = JSEvaluateScript(
                self.context.jsc_ref,
                script_js_string,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                0,
                std::ptr::null_mut(),
            );

            JSValueProtect(self.context.jsc_ref, return_value);

            Ok(JSCValue {
                jsc_ref: return_value,
                context: self.context.clone(),
            }
            .into())
        }
    }
    type StoreKey = DefaultKey;
    fn store_value(&mut self, value: Self::ValueType) -> Self::StoreKey {
        let key = self.value_initial_store.insert(());
        self.value_actual_store.insert(key, value);
        key
    }

    fn get_value_ref(&self, key: Self::StoreKey) -> Result<&Self::ValueType, JSEnvError> {
        return self
            .value_actual_store
            .get(key)
            .ok_or(JSEnvError::ValueNoLongerExists);
    }
    fn pull_value(&mut self, key: Self::StoreKey) -> Result<Self::ValueType, JSEnvError> {
        let value = self
            .value_actual_store
            .remove(key)
            .ok_or(JSEnvError::ValueNoLongerExists)?;
        self.value_initial_store.remove(key);
        Ok(value)
    }
}

impl Drop for JSCValue {
    fn drop(&mut self) {
        unsafe {
            JSValueUnprotect(self.context.jsc_ref, self.jsc_ref);
        }
    }
}

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn it_works() {
        let runtime = JSCRuntime::new();
        let val: JSCValue = runtime.evaluate("String(1+2)").unwrap();
        let str = val.to_string().unwrap();
        assert_eq!(str, "3")
    }
}
