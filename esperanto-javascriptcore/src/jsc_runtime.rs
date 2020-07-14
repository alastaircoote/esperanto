use crate::jsc_globalcontext::JSCGlobalContext;
use crate::jsc_value::JSCValue;
use esperanto_traits::js_traits::{JSEnvError, JSRuntime};
use javascriptcore_sys::{
    JSEvaluateScript, JSGlobalContextCreate, JSGlobalContextRetain, JSStringCreateWithUTF8CString,
};
use slotmap::{DefaultKey, SecondaryMap, SlotMap};
use std::ffi::CString;
use std::rc::Rc;

pub struct JSCRuntime {
    // jsc_ref: *mut OpaqueJSContext,
    context: Rc<JSCGlobalContext>,
    // This is really messy but slotmap requires that values implement Copy, which we can't
    // do because JSValueRef isn't copy-safe. So instead we use a SecondaryMap which CAN
    // store non-Copy items to store our actual values.
    value_initial_store: SlotMap<DefaultKey, ()>,
    value_actual_store: SecondaryMap<DefaultKey, JSCValue>,
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

            Ok(JSCValue::new(return_value, self.context.clone()).into())
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

#[cfg(test)]
mod test {

    use super::*;
    use std::convert::TryInto;

    #[test]
    fn it_works() {
        let runtime = JSCRuntime::new();
        let val: JSCValue = runtime.evaluate("String(1+2)").unwrap();
        let str: &str = val.try_into().unwrap();
        assert_eq!(str, "3")
    }
}
