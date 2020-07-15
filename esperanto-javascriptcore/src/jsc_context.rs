use crate::jsc_globalcontext::JSCGlobalContext;
use crate::jsc_string::JSCString;
use crate::{jsc_object::JSCObject, jsc_value::JSCValue};
use esperanto_shared::errors::{JSConversionError, JSEnvError, JSError};
use esperanto_shared::traits::JSContext;
use javascriptcore_sys::{
    JSEvaluateScript, JSGlobalContextCreate, JSGlobalContextRetain, JSStringCreateWithUTF8CString,
    JSValueGetType, JSValueRef,
};
use slotmap::{DefaultKey, SecondaryMap, SlotMap};
use std::ffi::CString;
use std::rc::Rc;

pub struct JSCContext {
    pub(crate) context: Rc<JSCGlobalContext>,
    // This is really messy but slotmap requires that values implement Copy, which we can't
    // do because JSValueRef isn't copy-safe. So instead we use a SecondaryMap which CAN
    // store non-Copy items to store our actual values.
    value_initial_store: SlotMap<DefaultKey, ()>,
    value_actual_store: SecondaryMap<DefaultKey, JSCValue>,
}

impl JSContext for JSCContext {
    type ValueType = JSCValue;
    type ObjectType = JSCObject;

    fn new() -> Self {
        unsafe {
            let ctx = JSGlobalContextCreate(std::ptr::null_mut());
            let retained_ctx = JSGlobalContextRetain(ctx);

            JSCContext {
                // jsc_ref: retained_ctx,
                context: Rc::new(JSCGlobalContext {
                    jsc_ref: retained_ctx,
                }),
                value_initial_store: SlotMap::new(),
                value_actual_store: SecondaryMap::new(),
            }
        }
    }

    fn evaluate(&self, script: &str) -> Result<JSCValue, JSEnvError> {
        let script_jsstring = JSCString::from_string(script)?;
        // let script_c_string = CString::new(script).map_err(|_| JSEnvError::CouldNotParseScript)?;

        // let script_js_string = unsafe { JSStringCreateWithUTF8CString(script_c_string.as_ptr()) };

        let mut exception_ptr: JSValueRef = std::ptr::null_mut();

        let return_value = unsafe {
            JSEvaluateScript(
                self.context.jsc_ref,
                script_jsstring.jsc_ref,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                0,
                &mut exception_ptr,
            )
        };

        if exception_ptr.is_null() == false {
            let error_val = JSCObject::new(exception_ptr, self.context.clone());

            return Err(JSEnvError::JSErrorEncountered(JSError::from(error_val)?));
        }

        Ok(JSCValue::new(return_value, self.context.clone()))
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
    use esperanto_shared::trait_tests::jscontext_tests;

    #[test]
    fn it_evaluates_correct_code() {
        jscontext_tests::it_evaluates_correct_code::<JSCContext>();
    }

    #[test]
    fn it_throws_exceptions_on_invalid_code() {
        jscontext_tests::it_throws_exceptions_on_invalid_code::<JSCContext>();
    }
}
