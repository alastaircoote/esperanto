use crate::{
    jsc_error::JSErrorFromJSC, jsc_globalcontext::JSCGlobalContext, jsc_string::JSCString,
};
use esperanto_shared::errors::{JSContextError, JSConversionError, JSError, JSEvaluationError};
use esperanto_shared::traits::{JSContext, JSValue};
use javascriptcore_sys::{
    JSObjectCallAsFunction, JSObjectGetProperty, JSObjectIsFunction, JSObjectRef, JSValueIsObject,
    JSValueMakeBoolean, JSValueMakeNumber, JSValueProtect, JSValueRef, JSValueToBoolean,
    JSValueToNumber, JSValueToObject, JSValueUnprotect, OpaqueJSValue,
};
use std::rc::Rc;

#[derive(Debug)]
pub struct JSCValue {
    pub(crate) raw_ref: JSValueRef,
    pub(crate) object_raw_ref: Option<JSObjectRef>,
    pub(crate) context: Rc<JSCGlobalContext>,
}

impl Drop for JSCValue {
    fn drop(&mut self) {
        unsafe {
            JSValueUnprotect(self.context.raw_ref, self.raw_ref);
        }
    }
}

impl JSValue for JSCValue {
    type ContextType = JSCGlobalContext;
    type RawType = JSValueRef;
    fn as_string(&self) -> Result<String, JSContextError> {
        let jsc = JSCString::from_js_value(self)?;
        Ok(jsc.to_string()?)
    }

    fn as_number(&self) -> Result<f64, JSContextError> {
        // As best I've been able to tell JSValueToNumber never actually creates an exception.
        // instead the returned value is NaN.

        // Will leave this here in the hopes we'll be able to find something that triggers an exception
        // in the future and test for it
        let exception: *mut JSValueRef = std::ptr::null_mut();

        let val = unsafe { JSValueToNumber(self.context.raw_ref, self.raw_ref, exception) };

        if val.is_nan() {
            Err(JSConversionError::ResultIsNotANumber.into())
        } else {
            Ok(val)
        }
    }
    fn from_number(
        number: f64,
        in_context: &Rc<Self::ContextType>,
    ) -> Result<Self, JSContextError> {
        let raw = unsafe { JSValueMakeNumber(in_context.raw_ref, number) };
        Ok(JSCValue {
            raw_ref: raw,
            object_raw_ref: None,
            context: in_context.clone(),
        })
    }

    fn from_one_arg_closure<
        I: esperanto_shared::traits::FromJSValue<Self> + 'static,
        O: esperanto_shared::traits::ToJSValue<Self> + 'static,
        F: Fn(I) -> Result<O, JSContextError> + 'static,
    >(
        closure: F,
        in_context: &Rc<Self::ContextType>,
    ) -> Result<Self, JSContextError> {
        todo!()
    }

    fn from_two_arg_closure<
        I1: esperanto_shared::traits::FromJSValue<Self> + 'static,
        I2: esperanto_shared::traits::FromJSValue<Self> + 'static,
        O: esperanto_shared::traits::ToJSValue<Self> + 'static,
        F: Fn(I1, I2) -> Result<O, JSContextError> + 'static,
    >(
        closure: F,
        in_context: &Rc<Self::ContextType>,
    ) -> Result<Self, JSContextError> {
        todo!()
    }
    fn call(&self) -> Result<Self, JSContextError> {
        self.call_bound(Vec::new(), self)
    }
    fn call_with_arguments(&self, arguments: Vec<&Self>) -> Result<Self, JSContextError> {
        self.call_bound(arguments, self)
    }
    fn call_bound(&self, arguments: Vec<&Self>, bound_to: &Self) -> Result<Self, JSContextError> {
        let obj_ref = self
            .object_raw_ref
            .ok_or(JSEvaluationError::IsNotAFunction)?;

        let arguments_raw = arguments
            .iter()
            .map(|a| a.raw_ref)
            .collect::<Vec<Self::RawType>>();

        let mut exception_ptr: JSValueRef = std::ptr::null_mut();

        let result = unsafe {
            JSObjectCallAsFunction(
                self.context.raw_ref,
                obj_ref,
                bound_to.raw_ref as *mut OpaqueJSValue,
                arguments.len(),
                arguments_raw.as_ptr(),
                &mut exception_ptr,
            )
        };
        JSError::check_jsc_value_ref(exception_ptr, &self.context)?;
        Self::from_raw(result, &self.context)
    }

    fn from_raw(
        raw: Self::RawType,
        in_context: &Rc<Self::ContextType>,
    ) -> Result<Self, JSContextError> {
        unsafe { JSValueProtect(in_context.raw_ref, raw) };

        // JavaScriptCore has two different "value" types: JSValueRef and JSObjectRef. It half makes sense: JSValueRef is
        // a const because it represents values, e.g. the number type, which can't ever be changed internally. JSObjectRef
        // is mutable, and you can change the properties of an object. But at the same time, numbers are objects: they have
        // methods like toFixed() that you can call. So we're wrapping both types in one here (another reason to do so is that
        // QuickJS makes no such distincton)

        let object_ref = match unsafe { JSValueIsObject(in_context.raw_ref, raw) } {
            true => {
                let mut exception_ptr: JSValueRef = std::ptr::null_mut();
                let obj = unsafe { JSValueToObject(in_context.raw_ref, raw, &mut exception_ptr) };
                JSError::check_jsc_value_ref(exception_ptr, in_context)?;
                Some(obj)
            }
            false => None,
        };
        Ok(JSCValue {
            raw_ref: raw,
            object_raw_ref: object_ref,
            context: in_context.clone(),
        })
    }
    fn as_bool(&self) -> Result<bool, JSContextError> {
        unsafe { Ok(JSValueToBoolean(self.context.raw_ref, self.raw_ref)) }
    }
    fn from_bool(bool: bool, in_context: &Rc<Self::ContextType>) -> Result<Self, JSContextError> {
        let raw = unsafe { JSValueMakeBoolean(in_context.raw_ref, bool) };
        Self::from_raw(raw, in_context)
    }
    fn get_property(&self, name: &str) -> Result<Self, JSContextError> {
        let obj_ref = self
            .object_raw_ref
            .ok_or(JSEvaluationError::IsNotAnObject)?;
        let name_jscstring = JSCString::from_string(name)?;

        let mut exception_ptr: JSValueRef = std::ptr::null_mut();

        let prop_val = unsafe {
            JSObjectGetProperty(
                self.context.raw_ref,
                obj_ref,
                name_jscstring.raw_ref,
                &mut exception_ptr,
            )
        };

        JSError::check_jsc_value_ref(exception_ptr, &self.context)?;

        JSCValue::from_raw(prop_val, &self.context)
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

    #[test]
    fn can_call_functions() {
        jsvalue_tests::can_call_functions::<JSCValue>()
    }

    #[test]
    fn converts_from_boolean() {
        jsvalue_tests::converts_from_boolean::<JSCValue>()
    }

    #[test]
    fn converts_to_boolean() {
        jsvalue_tests::converts_to_boolean::<JSCValue>()
    }

    #[test]
    fn can_get_properties() {
        jsvalue_tests::can_get_properties::<JSCValue>()
    }
}
