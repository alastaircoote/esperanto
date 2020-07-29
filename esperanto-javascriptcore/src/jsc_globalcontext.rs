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

#[derive(Debug)]
pub struct JSCGlobalContext {
    pub(crate) context: Rc<JSCSharedContextRef>,
}

impl JSContext for JSCGlobalContext {
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

        Ok(JSCGlobalContext {
            // jsc_ref: retained_ctx,
            context: Rc::new(JSCSharedContextRef { jsc_ref: ctx }),
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
}

#[cfg(test)]
mod test {
    use super::*;
    use esperanto_shared::{
        trait_tests::jscontext_tests,
        traits::{JSObject, JSValue},
    };
    use javascriptcore_sys::{JSContextRef, JSGarbageCollect, JSValueUnprotect};

    #[test]
    fn it_evaluates_correct_code() {
        jscontext_tests::it_evaluates_correct_code::<JSCGlobalContext>();
    }

    #[test]
    fn it_throws_exceptions_on_invalid_code() {
        jscontext_tests::it_throws_exceptions_on_invalid_code::<JSCGlobalContext>();
    }

    #[link(name = "JavaScriptCore", kind = "framework")]
    extern "C" {
        fn JSSynchronousGarbageCollectForDebugging(_: JSContextRef) -> ();
    }

    // sync garbage collect thing doesn't seem to work, so never mind

    #[test]
    fn it_discards_values() {
        let context = JSCGlobalContext::new().unwrap();
        let val = context.evaluate("'hello'").unwrap();
        // let obj = val.to_object().unwrap();
        unsafe {
            JSValueUnprotect(context.context.jsc_ref, val.jsc_ref);
            context.evaluate("new Uint8Array(9999999)").unwrap();
            JSSynchronousGarbageCollectForDebugging(context.context.jsc_ref);
        }
        let s = val.as_string().unwrap();
        println!("{}", s)
    }
}
