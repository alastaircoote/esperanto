use crate::jsc_globalcontext::JSCGlobalContext;
use crate::jsc_value::JSCValue;
use esperanto_traits::errors::{JSConversionError, JSEnvError};
use esperanto_traits::JSContext;
use javascriptcore_sys::{
    JSEvaluateScript, JSGlobalContextCreate, JSGlobalContextRetain, JSStringCreateWithUTF8CString,
    JSValueGetType, JSValueRef,
};
use slotmap::{DefaultKey, SecondaryMap, SlotMap};
use std::convert::TryFrom;
use std::convert::TryInto;
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

    fn evaluate<O: TryFrom<JSCValue>>(&self, script: &str) -> Result<O, JSEnvError> {
        let script_c_string = CString::new(script).map_err(|_| JSEnvError::CouldNotParseScript)?;
        unsafe {
            let script_js_string = JSStringCreateWithUTF8CString(script_c_string.as_ptr());

            let mut exception_ptr: JSValueRef = std::ptr::null_mut();

            let return_value = JSEvaluateScript(
                self.context.jsc_ref,
                script_js_string,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                0,
                &mut exception_ptr,
            );

            if exception_ptr.is_null() == false {
                let error_val = JSCValue::new(exception_ptr, self.context.clone());

                match error_val.try_into() {
                    Ok(error_str) => {
                        let _: &str = error_str; // need this so compiler knows the type
                        return Err(JSEnvError::JSErrorEncountered(error_str.to_string()));
                    }
                    Err(_) => {
                        return Err(JSEnvError::JSErrorEncountered(
                            "(could not read error message)".to_string(),
                        ))
                    }
                }
            }

            let jsvalue = JSCValue::new(return_value, self.context.clone());

            O::try_from(jsvalue)
                .map_err(|_| JSEnvError::ConversionError(JSConversionError::ConversionFailed))
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
    fn it_evaluates_correct_code() {
        let runtime = JSCContext::new();
        let val: JSCValue = runtime.evaluate("1+2").unwrap();
        let str: f64 = val.try_into().unwrap();
        assert_eq!(str, 3.0)
    }

    #[test]
    fn it_throws_exceptions_on_invalid_code() {
        let runtime = JSCContext::new();
        match runtime.evaluate::<JSCValue>("]") {
            Ok(_) => panic!("This call should not succeed"),
            Err(err) => {
                assert_eq!(
                    err,
                    JSEnvError::JSErrorEncountered("SyntaxError: Unexpected token ']'".to_string())
                );
            }
        }
    }
}
