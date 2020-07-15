use crate::{jsc_globalcontext::JSCGlobalContext, jsc_string::JSCString, jsc_value::JSCValue};
use esperanto_shared::errors::{JSEnvError, JSError};
use esperanto_shared::traits::JSObject;
use javascriptcore_sys::{
    JSObjectGetProperty, JSObjectRef, JSValueProtect, JSValueRef, JSValueToObject, JSValueUnprotect,
};
use std::convert::{TryFrom, TryInto};
use std::rc::Rc;

pub struct JSCObject {
    jsc_ref: JSObjectRef,
    context: Rc<JSCGlobalContext>,
}

impl JSCObject {
    pub fn from_obj_ref(value_ref: JSObjectRef, in_context: Rc<JSCGlobalContext>) -> Self {
        unsafe { JSValueProtect(in_context.jsc_ref, value_ref) };
        JSCObject {
            jsc_ref: value_ref,
            context: in_context,
        }
    }

    pub fn from_value_ref(
        value_ref: JSValueRef,
        in_context: Rc<JSCGlobalContext>,
    ) -> Result<Self, JSEnvError> {
        let mut exception_ptr: JSValueRef = std::ptr::null_mut();
        let obj_ref = unsafe { JSValueToObject(in_context.jsc_ref, value_ref, &mut exception_ptr) };

        if exception_ptr.is_null() == false {
            // This is a JSObject so if we wanted as little code as possible we'd call this function again... but then
            // there's a chance we end up in an infinite loop. So let's not.

            let mut exception_to_the_exception_ref: JSValueRef = std::ptr::null_mut();

            // This exception SHOULD be an instance of an Error, and so a JSObject. But there's always the possibility
            // of an exception here too, so let's make sure we're checking that.
            let obj_ref = unsafe {
                JSValueToObject(
                    in_context.jsc_ref,
                    exception_ptr,
                    &mut exception_to_the_exception_ref,
                )
            };

            if exception_to_the_exception_ref.is_null() == false {
                // We're not going to go any further. Just throw an unknown error.
                return Err(JSEnvError::UnknownInternalError);
            }

            let obj = JSCObject::from_obj_ref(obj_ref, in_context);
            let js_error = JSError::from(obj)?;

            return Err(JSEnvError::JSErrorEncountered(js_error));
        }

        unsafe { JSValueProtect(in_context.jsc_ref, value_ref) };
        Ok(JSCObject {
            jsc_ref: obj_ref,
            context: in_context,
        })
    }
}

impl TryFrom<JSCValue> for JSCObject {
    type Error = JSEnvError;
    fn try_from(value: JSCValue) -> Result<JSCObject, Self::Error> {
        JSCObject::from_value_ref(value.jsc_ref, value.context.clone())
    }
}

impl Drop for JSCObject {
    fn drop(&mut self) {
        unsafe { JSValueUnprotect(self.context.jsc_ref, self.jsc_ref) }
    }
}

impl JSObject for JSCObject {
    type ValueType = JSCValue;
    fn get_property(&self, name: &str) -> Result<Self::ValueType, JSEnvError> {
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

        Ok(JSCValue::from_value_ref(prop_val, self.context.clone()))
    }
}

#[cfg(test)]
mod test {

    use super::*;
    use crate::jsc_context::JSCContext;
    use esperanto_shared::traits::JSContext;
    use std::convert::TryInto;

    #[test]
    fn can_create_from_object() {
        let ctx = JSCContext::new();
        let result = ctx.evaluate("({})").unwrap();
        let _: JSCObject = result.try_into().unwrap();
    }

    #[test]
    fn throws_when_not_given_object() {
        let ctx = JSCContext::new();
        let result = ctx.evaluate("undefined").unwrap();
        let conversion_result: Result<JSCObject, JSEnvError> = result.try_into();
        match conversion_result {
            Ok(_) => panic!("Conversion should not succeed"),
            Err(e) => match e {
                JSEnvError::JSErrorEncountered(err) => {
                    assert_eq!(err.name, "TypeError");
                }
                _ => panic!("Unexpected error type"),
            },
        }
    }
}
