use crate::qjs_runtime::QJSRuntime;
use crate::qjs_value::QJSValue;
// use crate::qjs_value::QJSValue;
use crate::qjs_shared_context_ref::SharedQJSContextRef;
use esperanto_shared::cstr;
use esperanto_shared::errors::{JSEnvError, JSError};
use esperanto_shared::traits::JSContext;
use libquickjs_sys::{
    JS_Eval, JS_GetException, JS_NewContext, JS_EVAL_TYPE_GLOBAL, JS_TAG_EXCEPTION,
};
use slotmap::{DefaultKey, SecondaryMap, SlotMap};
use std::ffi::CString;
use std::rc::Rc;

pub struct QJSContext {
    // runtime: Rc<QJSRuntime>,
    context_ref: Rc<SharedQJSContextRef>,
    // pub(crate) qjs_ref: *mut QJSContextRef,
    // This is really messy but slotmap requires that values implement Copy, which we can't
    // do because JSValueRef isn't copy-safe. So instead we use a SecondaryMap which CAN
    // store non-Copy items to store our actual values.
    value_initial_store: SlotMap<DefaultKey, ()>,
    value_actual_store: SecondaryMap<DefaultKey, QJSValue>,
}

const MESSAGE_CSTR: *const i8 = cstr!("message");
const NAME_CSTR: *const i8 = cstr!("name");

impl JSContext for QJSContext {
    type ValueType = QJSValue;
    type ObjectType = QJSValue;
    type StoreKey = DefaultKey;
    fn new() -> Self {
        // need to add support for shared runtimes
        let runtime = Rc::new(QJSRuntime::new());
        let qjs_ref = unsafe { JS_NewContext(runtime.qjs_ref) };
        if qjs_ref.is_null() {
            println!("seems bad.")
        }
        QJSContext {
            // runtime,
            context_ref: Rc::new(SharedQJSContextRef::new(qjs_ref, runtime)),
            value_initial_store: SlotMap::new(),
            value_actual_store: SecondaryMap::new(),
        }
    }

    fn evaluate(&self, script: &str) -> Result<QJSValue, JSEnvError> {
        let script_as_c_string =
            CString::new(script).map_err(|_| JSEnvError::CouldNotParseScript)?;

        let fin = CString::new("file.js").unwrap();

        let result = unsafe {
            JS_Eval(
                self.context_ref.qjs_ref,
                script_as_c_string.as_ptr(),
                script.len() as _,
                fin.as_ptr(),
                JS_EVAL_TYPE_GLOBAL as i32,
            )
        };

        if result.tag != JS_TAG_EXCEPTION as i64 {
            return Ok(QJSValue::new(result, self.context_ref.clone()));
        }

        let exception = unsafe {
            QJSValue::new(
                JS_GetException(self.context_ref.qjs_ref),
                self.context_ref.clone(),
            )
        };

        let error = JSError::from(exception)?;
        Err(JSEnvError::JSErrorEncountered(error))
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

#[cfg(test)]
mod test {
    use super::*;
    use esperanto_shared::trait_tests::jscontext_tests;

    #[test]
    fn it_evaluates_correct_code() {
        jscontext_tests::it_evaluates_correct_code::<QJSContext>();
    }

    #[test]
    fn it_throws_exceptions_on_invalid_code() {
        jscontext_tests::it_throws_exceptions_on_invalid_code::<QJSContext>();
    }
}
