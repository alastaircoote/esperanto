use crate::qjs_value::QJSValue;
use esperanto_traits::errors::JSEnvError;
use esperanto_traits::JSRuntime;
use qjs_sys::{
    JSContext as QJSContextRef, JSRuntime as QJSRuntimeRef, JS_Eval, JS_FreeContext,
    JS_NewContextRaw, JS_NewRuntime,
};
use slotmap::{DefaultKey, SecondaryMap, SlotMap};
use std::ffi::CString;

struct QJSContext {
    qjs_ref: *mut QJSContextRef,
}

impl QJSContext {
    fn new() -> Self {
        let ref = JS_NewContextRaw();
    }
}

impl Drop for QJSContext {
    fn drop(&mut self) {
        unsafe { JS_FreeContext(self.qjs_ref) };
    }
}

struct QJSRuntime {
    qjs_ref: *mut QJSRuntimeRef,
    // This is really messy but slotmap requires that values implement Copy, which we can't
    // do because JSValueRef isn't copy-safe. So instead we use a SecondaryMap which CAN
    // store non-Copy items to store our actual values.
    value_initial_store: SlotMap<DefaultKey, ()>,
    value_actual_store: SecondaryMap<DefaultKey, QJSValue>,
}

impl JSRuntime for QJSRuntime {
    type ValueType = QJSValue;
    type StoreKey = DefaultKey;
    fn new() -> Self {
        let qjs_ref = unsafe { JS_NewRuntime() };
        QJSRuntime {
            qjs_ref,
            value_initial_store: SlotMap::new(),
            value_actual_store: SecondaryMap::new(),
        }
    }

    fn evaluate<O: From<QJSValue>>(&self, script: &str) -> Result<O, JSEnvError> {
        let script_as_c_string =
            CString::new(script).map_err(|_| JSEnvError::CouldNotParseScript)?;
        let bytes = script_as_c_string.as_bytes();

        unsafe { JS_Eval(self.qjs_ref, bytes, bytes.len(), std::ptr::null_mut(), 0) };
    }

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
