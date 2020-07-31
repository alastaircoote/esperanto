use crate::jsc_string::JSCString;
use crate::{jsc_error::JSErrorFromJSC, jsc_object::JSCObject, jsc_value::JSCValue};
use esperanto_shared::errors::{JSContextError, JSError};
use esperanto_shared::traits::JSContext;
use javascriptcore_sys::{
    JSContextGroupCreate, JSEvaluateScript, JSGlobalContextCreateInGroup, JSGlobalContextRelease,
    JSGlobalContextRetain, JSValueRef, OpaqueJSContext, OpaqueJSContextGroup,
};
// use slotmap::{DefaultKey, SecondaryMap, SlotMap};
use std::rc::Rc;

#[derive(Debug)]
pub struct JSCGlobalContext {
    pub(crate) raw_ref: *mut OpaqueJSContext,
    group_raw_ref: *const OpaqueJSContextGroup,
}

impl JSContext for JSCGlobalContext {
    type ValueType = JSCValue;
    type ObjectType = JSCObject;

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
            raw_ref: retained_ctx,
            group_raw_ref: group,
        }))
    }

    fn evaluate(self: &Rc<Self>, script: &str) -> Result<JSCValue, JSContextError> {
        let script_jsstring = JSCString::from_string(script)?;

        let mut exception_ptr: JSValueRef = std::ptr::null_mut();

        let return_value = unsafe {
            JSEvaluateScript(
                self.raw_ref,
                script_jsstring.raw_ref,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                0,
                &mut exception_ptr,
            )
        };

        JSError::check_jsc_value_ref(exception_ptr, &self)?;

        Ok(JSCValue::from_value_ref(return_value, self))
    }
}

impl Drop for JSCGlobalContext {
    fn drop(&mut self) {
        unsafe { JSGlobalContextRelease(self.raw_ref) }
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
