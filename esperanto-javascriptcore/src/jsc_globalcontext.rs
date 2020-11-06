use crate::jsc_string::JSCString;
use crate::{jsc_error::JSErrorFromJSC, jsc_value::JSCValue};
use esperanto_shared::errors::{JSContextError, JSError};
use esperanto_shared::traits::{JSContext, JSValue};
use javascriptcore_sys::{
    JSContextGroupCreate, JSContextGroupRelease, JSEvaluateScript, JSGlobalContextCreateInGroup,
    JSGlobalContextRelease, JSGlobalContextRetain, JSValueRef, OpaqueJSContext,
    OpaqueJSContextGroup,
};
// use slotmap::{DefaultKey, SecondaryMap, SlotMap};
use std::{os::raw::c_char, rc::Rc};

#[derive(Debug)]
pub struct JSCGlobalContext {
    pub(crate) raw_ref: *mut OpaqueJSContext,
    group_raw_ref: *const OpaqueJSContextGroup,
}

impl JSCGlobalContext {
    fn evaluate_jscstring(self: &Rc<Self>, script: JSCString) -> Result<JSCValue, JSContextError> {
        let mut exception_ptr: JSValueRef = std::ptr::null_mut();

        let return_value = unsafe {
            JSEvaluateScript(
                self.raw_ref,
                script.raw_ref,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                0,
                &mut exception_ptr,
            )
        };

        JSError::check_jsc_value_ref(exception_ptr, &self)?;

        JSCValue::from_raw(return_value, self)
    }
}

impl JSContext for JSCGlobalContext {
    type ValueType = JSCValue;

    fn new() -> Result<Rc<Self>, JSContextError> {
        let group = unsafe { JSContextGroupCreate() };

        let ctx = unsafe { JSGlobalContextCreateInGroup(group, std::ptr::null_mut()) };
        if ctx.is_null() {
            return Err(JSContextError::CouldNotCreateContext);
        }
        let retained_ctx = unsafe { JSGlobalContextRetain(ctx) };
        if retained_ctx.is_null() {
            return Err(JSContextError::CouldNotCreateContext);
        }
        assert_eq!(retained_ctx, ctx);

        Ok(Rc::new(JSCGlobalContext {
            raw_ref: ctx,
            group_raw_ref: group,
        }))
    }

    fn evaluate(self: &Rc<Self>, script: &str) -> Result<JSCValue, JSContextError> {
        let script_jsstring = JSCString::from_string(script)?;
        self.evaluate_jscstring(script_jsstring)
    }

    fn evaluate_cstring(
        self: &Rc<Self>,
        script: *const c_char,
    ) -> Result<Self::ValueType, JSContextError> {
        let script_jsstring = JSCString::from_c_string(script)?;
        self.evaluate_jscstring(script_jsstring)
    }

    fn compile_string<'a>(self: &Rc<Self>, _: *const c_char) -> Result<&'a [u8], JSContextError> {
        Err(JSContextError::NotSupported)
    }

    fn eval_compiled(self: &Rc<Self>, _: &[u8]) -> Result<Self::ValueType, JSContextError> {
        Err(JSContextError::NotSupported)
    }
}

impl Drop for JSCGlobalContext {
    fn drop(&mut self) {
        unsafe { JSGlobalContextRelease(self.raw_ref) }
        unsafe { JSContextGroupRelease(self.group_raw_ref) }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use esperanto_shared::trait_tests::jscontext_tests;
    // use javascriptcore_sys::{JSContextRef, JSGarbageCollect, JSValueUnprotect};

    #[test]
    fn it_evaluates_correct_code() {
        jscontext_tests::it_evaluates_correct_code::<JSCGlobalContext>();
    }

    #[test]
    fn it_throws_exceptions_on_invalid_code() {
        jscontext_tests::it_throws_exceptions_on_invalid_code::<JSCGlobalContext>();
    }

    // #[link(name = "JavaScriptCore", kind = "framework")]
    // extern "C" {
    //     fn JSSynchronousGarbageCollectForDebugging(ctx: JSContextRef) -> ();
    // }

    // // sync garbage collect thing doesn't seem to work, so never mind

    // #[test]
    // fn it_discards_values() {
    //     let context = JSCGlobalContext::new().unwrap();
    //     let val = context.evaluate("var test = 'hello'; test").unwrap();
    //     // let obj = val.to_object().unwrap();
    //     unsafe {
    //         JSValueUnprotect(context.raw_ref, val.jsc_ref);
    //         // JSValueUnprotect(context.raw_ref, val.jsc_ref);
    //         context.evaluate("test = undefined").unwrap();
    //         JSSynchronousGarbageCollectForDebugging(context.raw_ref);
    //     }
    //     let s = val.as_string().unwrap();
    //     println!("{}", s)
    // }
}
