use crate::{jsc_globalcontext::JSCGlobalContext, jsc_string::JSCString, jsc_value::JSCValue};
use esperanto_shared::errors::JSEnvError;
use esperanto_shared::traits::JSObject;
use javascriptcore_sys::{
    JSObjectGetProperty, JSObjectRef, JSValueProtect, JSValueRef, JSValueUnprotect,
};
use std::rc::Rc;

pub struct JSCObject {
    jsc_ref: JSObjectRef,
    context: Rc<JSCGlobalContext>,
}

impl JSCObject {
    pub fn new(value_ref: JSObjectRef, in_context: Rc<JSCGlobalContext>) -> Self {
        unsafe { JSValueProtect(in_context.jsc_ref, value_ref) };
        JSCObject {
            jsc_ref: value_ref,
            context: in_context,
        }
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

        Ok(JSCValue::new(prop_val, self.context.clone()))
    }
}
