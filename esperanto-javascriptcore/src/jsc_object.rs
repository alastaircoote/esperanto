use crate::{
    jsc_error::JSErrorFromJSC, jsc_sharedcontextref::JSCSharedContextRef, jsc_string::JSCString,
    jsc_value::JSCValue,
};
use esperanto_shared::errors::{JSContextError, JSError};
use esperanto_shared::traits::JSObject;
use javascriptcore_sys::{
    JSObjectGetProperty, JSObjectRef, JSValueProtect, JSValueRef, JSValueToObject, JSValueUnprotect,
};
use std::convert::TryFrom;
use std::rc::Rc;

#[derive(Debug)]
pub struct JSCObject {
    pub(crate) jsc_ref: JSObjectRef,
    context: Rc<JSCSharedContextRef>,
}

impl JSCObject {
    // pub fn from_obj_ref(value_ref: JSObjectRef, in_context: Rc<JSCSharedContextRef>) -> Self {
    //     unsafe { JSValueProtect(in_context.jsc_ref, value_ref) };
    //     JSCObject {
    //         jsc_ref: value_ref,
    //         context: in_context,
    //     }
    // }

    pub fn from_value_ref(
        value_ref: JSValueRef,
        in_context: &Rc<JSCSharedContextRef>,
    ) -> Result<Self, JSContextError> {
        let mut exception_ptr: JSValueRef = std::ptr::null_mut();
        let obj_ref = unsafe { JSValueToObject(in_context.jsc_ref, value_ref, &mut exception_ptr) };

        JSError::check_jsc_value_ref(exception_ptr, &in_context)?;

        unsafe { JSValueProtect(in_context.jsc_ref, value_ref) };

        Ok(JSCObject {
            jsc_ref: obj_ref,
            context: in_context.clone(),
        })
    }
}

impl TryFrom<JSCValue> for JSCObject {
    type Error = JSContextError;
    fn try_from(value: JSCValue) -> Result<JSCObject, Self::Error> {
        JSCObject::from_value_ref(value.jsc_ref, &value.context)
    }
}

impl Drop for JSCObject {
    fn drop(&mut self) {
        unsafe { JSValueUnprotect(self.context.jsc_ref, self.jsc_ref) }
    }
}

impl JSObject for JSCObject {
    type ValueType = JSCValue;
    fn get_property(&self, name: &str) -> Result<Self::ValueType, JSContextError> {
        let name_jscstring = JSCString::from_string(name)?;

        let mut exception_ptr: JSValueRef = std::ptr::null_mut();

        let prop_val = unsafe {
            JSObjectGetProperty(
                self.context.jsc_ref,
                self.jsc_ref,
                name_jscstring.jsc_ref,
                &mut exception_ptr,
            )
        };

        JSError::check_jsc_value_ref(exception_ptr, &self.context)?;

        Ok(JSCValue::from_value_ref(prop_val, &self.context))
    }
}

#[cfg(test)]
mod test {

    use super::*;
    use crate::jsc_globalcontext::JSCGlobalContext;
    use esperanto_shared::traits::{JSContext, JSValue};
    use std::convert::TryInto;

    #[test]
    fn can_create_from_object() {
        let ctx = JSCGlobalContext::new().unwrap();
        let result = ctx.evaluate("({})").unwrap();
        let _: JSCObject = result.try_into().unwrap();
    }

    #[test]
    fn throws_when_not_given_object() {
        let ctx = JSCGlobalContext::new().unwrap();
        let result = ctx.evaluate("undefined").unwrap();
        let conversion_result = result.to_object().unwrap_err();

        match conversion_result {
            JSContextError::JavaScriptErrorOccurred(err) => {
                assert_eq!(err.name, "TypeError");
            }
            _ => panic!("Unexpected error type"),
        }
    }
}
