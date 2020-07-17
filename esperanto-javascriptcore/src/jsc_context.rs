use crate::jsc_sharedcontextref::JSCSharedContextRef;
use crate::jsc_string::JSCString;
use crate::{jsc_error::JSErrorFromJSC, jsc_object::JSCObject, jsc_value::JSCValue};
use esperanto_shared::errors::{JSContextError, JSError};
use esperanto_shared::traits::JSContext;
use javascriptcore_sys::{
    JSEvaluateScript, JSGlobalContextCreate, JSGlobalContextRetain, JSValueRef,
};
// use slotmap::{DefaultKey, SecondaryMap, SlotMap};
use std::rc::Rc;

pub struct JSCContext {
    pub(crate) context: Rc<JSCSharedContextRef>,
    // This is really messy but slotmap requires that values implement Copy, which we can't
    // do because JSValueRef isn't copy-safe. So instead we use a SecondaryMap which CAN
    // store non-Copy items to store our actual values.
    // value_initial_store: SlotMap<DefaultKey, ()>,
    // value_actual_store: SecondaryMap<DefaultKey, JSCValue>,
}

impl JSContext for JSCContext {
    type ValueType = JSCValue;
    type ObjectType = JSCObject;

    fn new() -> Result<Self, JSContextError> {
        let ctx = unsafe { JSGlobalContextCreate(std::ptr::null_mut()) };
        if ctx.is_null() {
            return Err(JSContextError::CouldNotCreateContext);
        }
        let retained_ctx = unsafe { JSGlobalContextRetain(ctx) };
        if retained_ctx.is_null() {
            return Err(JSContextError::CouldNotCreateContext);
        }

        Ok(JSCContext {
            // jsc_ref: retained_ctx,
            context: Rc::new(JSCSharedContextRef {
                jsc_ref: retained_ctx,
            }),
            // value_initial_store: SlotMap::new(),
            // value_actual_store: SecondaryMap::new(),
        })
    }

    fn evaluate(&self, script: &str) -> Result<JSCValue, JSContextError> {
        let script_jsstring = JSCString::from_string(script)?;

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

        JSError::check_jsc_value_ref(exception_ptr, &self.context)?;

        Ok(JSCValue::from_value_ref(return_value, &self.context))
    }
    // type StoreKey = DefaultKey;
    // fn store_value(&mut self, value: Self::ValueType) -> Self::StoreKey {
    //     let key = self.value_initial_store.insert(());
    //     self.value_actual_store.insert(key, value);
    //     key
    // }

    // fn get_value_ref(&self, key: Self::StoreKey) -> Result<&Self::ValueType, JSEnvError> {
    //     return self
    //         .value_actual_store
    //         .get(key)
    //         .ok_or(JSEnvError::ValueNoLongerExists);
    // }
    // fn pull_value(&mut self, key: Self::StoreKey) -> Result<Self::ValueType, JSEnvError> {
    //     let value = self
    //         .value_actual_store
    //         .remove(key)
    //         .ok_or(JSEnvError::ValueNoLongerExists)?;
    //     self.value_initial_store.remove(key);
    //     Ok(value)
    // }
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
