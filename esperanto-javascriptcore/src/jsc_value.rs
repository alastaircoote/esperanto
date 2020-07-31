use crate::{jsc_globalcontext::JSCGlobalContext, jsc_object::JSCObject, jsc_string::JSCString};
use esperanto_shared::errors::{JSContextError, JSConversionError};
use esperanto_shared::traits::{JSContext, JSValue};
use javascriptcore_sys::{
    JSValueMakeNumber, JSValueProtect, JSValueRef, JSValueToNumber, JSValueToStringCopy,
    JSValueUnprotect,
};
use std::rc::Rc;

#[derive(Debug)]
pub struct JSCValue {
    pub(crate) jsc_ref: JSValueRef,
    pub(crate) context: Rc<JSCGlobalContext>,
}

impl JSCValue {
    pub fn from_value_ref(v_ref: JSValueRef, in_context: &Rc<JSCGlobalContext>) -> Self {
        unsafe { JSValueProtect(in_context.raw_ref, v_ref) };
        JSCValue {
            jsc_ref: v_ref,
            context: in_context.clone(),
        }
    }
}

impl Drop for JSCValue {
    fn drop(&mut self) {
        unsafe {
            JSValueUnprotect(self.context.raw_ref, self.jsc_ref);
        }
    }
}

impl JSValue for JSCValue {
    type ContextType = JSCGlobalContext;
    fn as_string(&self) -> Result<String, JSConversionError> {
        let mut exception_ptr: JSValueRef = std::ptr::null_mut();
        let str_ptr =
            unsafe { JSValueToStringCopy(self.context.raw_ref, self.jsc_ref, &mut exception_ptr) };

        let jsc_string = JSCString::from_ptr(str_ptr);
        jsc_string.to_string()
    }

    fn to_object(self) -> Result<<Self::ContextType as JSContext>::ObjectType, JSContextError> {
        JSCObject::from_value_ref(self.jsc_ref, &self.context)
    }
    fn as_number(&self) -> Result<f64, JSConversionError> {
        // As best I've been able to tell JSValueToNumber never actually creates an exception.
        // instead the returned value is NaN.

        // Will leave this here in the hopes we'll be able to find something that triggers an exception
        // in the future and test for it
        let exception: *mut JSValueRef = std::ptr::null_mut();

        let val = unsafe { JSValueToNumber(self.context.raw_ref, self.jsc_ref, exception) };

        if val.is_nan() {
            Err(JSConversionError::ResultIsNotANumber)
        } else {
            Ok(val)
        }
    }
    fn from_number(number: f64, in_context: &Rc<Self::ContextType>) -> Self {
        let raw = unsafe { JSValueMakeNumber(in_context.raw_ref, number) };
        JSCValue::from_value_ref(raw, in_context)
    }
    fn from_one_arg_closure<
        I: esperanto_shared::traits::FromJSValue<Self> + 'static,
        O: esperanto_shared::traits::ToJSValue<Self> + 'static,
        F: Fn(I) -> O + 'static,
    >(
        closure: F,
        in_context: &Rc<Self::ContextType>,
    ) -> Self {
        todo!()
    }
    fn from_two_arg_closure<
        I1: esperanto_shared::traits::FromJSValue<Self> + 'static,
        I2: esperanto_shared::traits::FromJSValue<Self> + 'static,
        O: esperanto_shared::traits::ToJSValue<Self> + 'static,
        F: Fn(I1, I2) -> O + 'static,
    >(
        closure: F,
        in_context: &Rc<Self::ContextType>,
    ) -> Self {
        todo!()
    }
    fn call(&self) -> Self {
        todo!()
    }
    fn call_with_arguments(&self, arguments: Vec<&Self>) -> Self {
        todo!()
    }
    fn call_bound(&self, arguments: Vec<&Self>, bound_to: &Self) -> Self {
        todo!()
    }
}

#[cfg(test)]
mod test {
    use super::JSCValue;
    use esperanto_shared::trait_tests::jsvalue_tests;
    #[test]
    fn converts_to_number() {
        jsvalue_tests::converts_to_number::<JSCValue>()
    }

    #[test]
    fn converts_to_string() {
        jsvalue_tests::converts_to_string::<JSCValue>()
    }

    #[test]
    fn converts_from_number() {
        jsvalue_tests::converts_from_number::<JSCValue>()
    }
}
