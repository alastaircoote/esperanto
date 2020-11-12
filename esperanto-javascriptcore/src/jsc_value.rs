use crate::{
    jsc_error::JSErrorFromJSC, jsc_function::wrap_closure, jsc_globalcontext::JSCGlobalContext,
    jsc_string::JSCString,
};
use esperanto_shared::errors::{JSContextError, JSConversionError, JSError, JSEvaluationError};
use esperanto_shared::{
    traits::JSValue,
    util::closures::{wrap_one_argument_closure, wrap_two_argument_closure},
};
use javascriptcore_sys::{
    JSClassRef, JSClassRelease, JSClassRetain, JSObjectCallAsFunction, JSObjectGetProperty,
    JSObjectMakeFunction, JSObjectRef, JSValueIsObject, JSValueMakeBoolean, JSValueMakeNumber,
    JSValueMakeString, JSValueProtect, JSValueRef, JSValueToBoolean, JSValueToNumber,
    JSValueToObject, JSValueUnprotect, OpaqueJSClass, OpaqueJSString, OpaqueJSValue,
};
use std::rc::Rc;

#[derive(Debug)]
pub enum RawRef {
    JSValue(JSValueRef),
    JSObject(JSObjectRef),
    JSClass(JSClassRef, JSObjectRef),
}

impl RawRef {
    pub fn get_jsvalue(&self) -> JSValueRef {
        match self {
            RawRef::JSValue(val) => *val,
            RawRef::JSObject(obj) => *obj,
            RawRef::JSClass(_, obj) => *obj,
        }
    }

    pub fn get_jsobject(&self) -> Option<JSObjectRef> {
        match self {
            RawRef::JSValue(_) => None,
            RawRef::JSObject(obj) => Some(*obj),
            RawRef::JSClass(_, obj) => Some(*obj),
        }
    }
}

#[derive(Debug)]
pub struct JSCValue {
    pub(crate) raw_ref: RawRef,
    // pub(crate) object_raw_ref: Option<JSObjectRef>,
    pub(crate) context: Rc<JSCGlobalContext>,
}

impl Drop for JSCValue {
    fn drop(&mut self) {
        match self.raw_ref {
            RawRef::JSValue(val) => unsafe { JSValueUnprotect(self.context.raw_ref, val) },
            RawRef::JSObject(obj) => unsafe {
                // Objects are unprotected the same way (I think?!)
                JSValueUnprotect(self.context.raw_ref, obj)
            },
            RawRef::JSClass(class, obj) => unsafe {
                JSValueUnprotect(self.context.raw_ref, obj);
                JSClassRelease(class);
            },
        }
    }
}

impl JSCValue {
    pub fn from_raw_object_ref(
        obj_ref: *mut OpaqueJSValue,
        in_context: &Rc<JSCGlobalContext>,
    ) -> Result<Self, JSContextError> {
        Ok(JSCValue {
            context: in_context.clone(),
            raw_ref: RawRef::JSObject(obj_ref),
        })
    }

    pub fn from_class_ref(
        class_ref: *mut OpaqueJSClass,
        class_obj_ref: *mut OpaqueJSValue,
        in_context: &Rc<JSCGlobalContext>,
    ) -> Result<Self, JSContextError> {
        unsafe { JSClassRetain(class_ref) };
        Ok(JSCValue {
            context: in_context.clone(),
            raw_ref: RawRef::JSClass(class_ref, class_obj_ref),
        })
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
        let raw_ref = match self.raw_ref {
            RawRef::JSValue(val) => val,
            _ => return Err(JSConversionError::ResultIsNotANumber.into()),
        };

        // As best I've been able to tell JSValueToNumber never actually creates an exception.
        // instead the returned value is NaN.

        // Will leave this here in the hopes we'll be able to find something that triggers an exception
        // in the future and test for it

        let exception: *mut JSValueRef = std::ptr::null_mut();

        let val = unsafe { JSValueToNumber(self.context.raw_ref, raw_ref, exception) };

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
        Self::from_raw(raw, in_context)
    }

    fn from_string(str: &str, in_context: &Rc<Self::ContextType>) -> Result<Self, JSContextError> {
        let jsc_string = JSCString::from_string(str)?;
        let val_ref = unsafe { JSValueMakeString(in_context.raw_ref, jsc_string.raw_ref) };
        Self::from_raw(val_ref, in_context)
    }

    fn from_one_arg_closure<
        I: esperanto_shared::traits::FromJSValue<Self> + 'static,
        O: esperanto_shared::traits::ToJSValue<Self> + 'static,
        F: Fn(I) -> Result<O, JSContextError> + 'static,
    >(
        closure: F,
        in_context: &Rc<Self::ContextType>,
    ) -> Result<Self, JSContextError> {
        let closure = wrap_one_argument_closure(closure, in_context);
        let raw = wrap_closure(closure, in_context);
        Self::from_raw(raw, in_context)
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
        let closure = wrap_two_argument_closure(closure, in_context);
        let raw = wrap_closure(closure, in_context);
        Self::from_raw(raw, in_context)
    }

    fn call_bound(&self, arguments: Vec<&Self>, bound_to: &Self) -> Result<Self, JSContextError> {
        let obj_ref = self
            .raw_ref
            .get_jsobject()
            .ok_or(JSEvaluationError::IsNotAFunction)?;

        let bound_obj = bound_to
            .raw_ref
            .get_jsobject()
            .ok_or(JSEvaluationError::IsNotAnObject)?;

        let arguments_raw = arguments
            .iter()
            .map(|a| a.raw_ref.get_jsvalue())
            .collect::<Vec<Self::RawType>>();

        let mut exception_ptr: JSValueRef = std::ptr::null_mut();

        let result = unsafe {
            JSObjectCallAsFunction(
                self.context.raw_ref,
                obj_ref,
                bound_obj,
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

        let raw_ref = match unsafe { JSValueIsObject(in_context.raw_ref, raw) } {
            true => {
                let mut exception_ptr: JSValueRef = std::ptr::null_mut();
                let obj = unsafe { JSValueToObject(in_context.raw_ref, raw, &mut exception_ptr) };
                JSError::check_jsc_value_ref(exception_ptr, in_context)?;
                RawRef::JSObject(obj)
            }
            false => RawRef::JSValue(raw),
        };
        Ok(JSCValue {
            raw_ref,
            context: in_context.clone(),
        })
    }
    fn as_bool(&self) -> Result<bool, JSContextError> {
        unsafe {
            Ok(JSValueToBoolean(
                self.context.raw_ref,
                self.raw_ref.get_jsvalue(),
            ))
        }
    }
    fn from_bool(bool: bool, in_context: &Rc<Self::ContextType>) -> Result<Self, JSContextError> {
        let raw = unsafe { JSValueMakeBoolean(in_context.raw_ref, bool) };
        Self::from_raw(raw, in_context)
    }
    fn get_property(&self, name: &str) -> Result<Self, JSContextError> {
        let obj_ref = self
            .raw_ref
            .get_jsobject()
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
    fn create_function(
        in_context: &Rc<Self::ContextType>,
        arg_names: Vec<&str>,
        body: &str,
    ) -> Result<Self, JSContextError> {
        // Originally I had this creating the JSCString and immediately grabbing the
        // raw reference, but when I did that JSCString::drop was called before
        // the function was created. So we're doing it separately to keep the reference
        // to the JSC strings.

        let args_as_jsc_strings = arg_names
            .iter()
            .map(|n| JSCString::from_string(n))
            .collect::<Result<Vec<JSCString>, _>>()?;

        let args_raw = args_as_jsc_strings
            .iter()
            .map(|s| s.raw_ref)
            .collect::<Vec<*mut OpaqueJSString>>();

        let mut exception_ptr: JSValueRef = std::ptr::null_mut();
        let raw = unsafe {
            JSObjectMakeFunction(
                in_context.raw_ref,
                std::ptr::null_mut(),
                args_as_jsc_strings.len() as u32,
                args_raw.as_ptr(),
                JSCString::from_string(body)?.raw_ref,
                std::ptr::null_mut(),
                0,
                &mut exception_ptr,
            )
        };

        JSError::check_jsc_value_ref(exception_ptr, in_context)?;

        Self::from_raw(raw, in_context)
    }

    // fn call_property_with_arguments(
    //     &self,
    //     name: &str,
    //     arguments: Vec<&Self>,
    // ) -> Result<Self, JSContextError> {
    //     todo!()
    // }
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
    fn converts_from_string() {
        jsvalue_tests::converts_from_string::<JSCValue>()
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

    #[test]
    fn can_wrap_rust_closure_with_one_argument() {
        jsvalue_tests::can_wrap_rust_closure_with_one_argument::<JSCValue>();
    }

    #[test]
    fn can_wrap_rust_closure_with_two_arguments() {
        jsvalue_tests::can_wrap_rust_closure_with_two_arguments::<JSCValue>();
    }

    #[test]
    fn can_create_function() {
        jsvalue_tests::can_create_function::<JSCValue>();
    }
}
