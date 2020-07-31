use crate::{
    qjs_context::QJSContext,
    qjs_function::{wrap_one_argument_closure, wrap_two_argument_closure},
    ref_count::free_value,
};
use esperanto_shared::errors::{JSContextError, JSConversionError};
use esperanto_shared::traits::{FromJSValue, JSContext, JSObject, JSValue, ToJSValue};
use libquickjs_sys::{
    JSValue as QJSRawValue, JSValueUnion, JS_Call, JS_FreeCString, JS_GetPropertyStr,
    JS_ToCStringLen2, JS_ToFloat64, JS_TAG_BOOL, JS_TAG_EXCEPTION, JS_TAG_FLOAT64,
};
use std::ffi::{CStr, CString};
use std::rc::Rc;

pub struct QJSValue {
    pub(crate) raw: QJSRawValue,
    context: Rc<QJSContext>,
}

// Since QJSRawValue is a C struct it doesn't automatically derive Debug, so we need to
// manually implement debug here.
impl std::fmt::Debug for QJSValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("QJSValue")
            .field("context", &self.context)
            .field("raw", unsafe { &self.raw.u.ptr })
            .finish()
    }
}

impl JSValue for QJSValue {
    type ContextType = QJSContext;
    fn as_string(&self) -> Result<String, JSContextError> {
        // let qjs_string = JS_ToString(self.context_ref.qjs_ref, self.qjs_ref);
        let c_str_ptr = unsafe { JS_ToCStringLen2(self.context.raw, &mut 0, self.raw, 0) };
        let c_str = unsafe { CStr::from_ptr(c_str_ptr) };
        let string = c_str.to_str()?.to_string();
        unsafe { JS_FreeCString(self.context.raw, c_str_ptr) };
        Ok(string)
    }
    fn to_object(
        self,
    ) -> Result<
        <Self::ContextType as JSContext>::ObjectType,
        esperanto_shared::errors::JSContextError,
    > {
        Ok(self)
    }
    fn as_number(&self) -> Result<f64, JSContextError> {
        let mut result = f64::NAN;
        let return_code = unsafe { JS_ToFloat64(self.context.raw, &mut result, self.raw) };
        if return_code != 0 {
            // I've never seen it return anything other than 0, so if it does that seems notable.
            return Err(JSConversionError::UnknownError.into());
        }
        if result == f64::NAN {
            Err(JSConversionError::ResultIsNotANumber.into())
        } else {
            Ok(result)
        }
    }

    fn from_number(number: f64, in_context: &Rc<Self::ContextType>) -> Self {
        let val = QJSRawValue {
            tag: JS_TAG_FLOAT64 as i64,
            u: JSValueUnion { float64: number },
        };
        Self::from_raw(val, in_context)
    }

    fn from_one_arg_closure<
        I: FromJSValue<Self> + 'static,
        O: ToJSValue<Self> + 'static,
        F: Fn(I) -> O + 'static,
    >(
        closure: F,
        in_context: &Rc<Self::ContextType>,
    ) -> Self {
        let internal_val = wrap_one_argument_closure(closure, in_context);
        Self::from_raw(internal_val, in_context)
    }

    fn from_two_arg_closure<
        I1: FromJSValue<Self> + 'static,
        I2: FromJSValue<Self> + 'static,
        O: ToJSValue<Self> + 'static,
        F: Fn(I1, I2) -> O + 'static,
    >(
        closure: F,
        in_context: &Rc<Self::ContextType>,
    ) -> Self {
        let internal_val = wrap_two_argument_closure(closure, in_context);
        Self::from_raw(internal_val, in_context)
    }

    fn call(&self) -> Self {
        self.call_bound(Vec::new(), self)
    }
    fn call_with_arguments(&self, arguments: Vec<&Self>) -> Self {
        self.call_bound(arguments, self)
    }
    fn call_bound(&self, arguments: Vec<&Self>, bound_to: &Self) -> Self {
        let return_val = unsafe {
            JS_Call(
                self.context.raw,
                self.raw,
                bound_to.raw,
                arguments.len() as i32,
                arguments
                    .iter()
                    .map(|a| a.raw)
                    .collect::<Vec<QJSRawValue>>()
                    .as_mut_ptr(),
            )
        };

        QJSValue::from_raw(return_val, &self.context)
    }
}

// QuickJS doesn't really make a distinction between values and objects like JSC does
// so we just do it all on the same struct
impl JSObject for QJSValue {
    type ValueType = Self;
    fn get_property(&self, name: &str) -> Result<Self, JSContextError> {
        let name_cstring = CString::new(name)?;
        let property_ref =
            unsafe { JS_GetPropertyStr(self.context.raw, self.raw, name_cstring.as_ptr()) };

        Ok(Self::from_raw(property_ref, &self.context))
    }
}

impl QJSValue {
    pub(crate) fn from_raw(raw: QJSRawValue, in_context: &Rc<QJSContext>) -> Self {
        // unsafe { dup_value(in_context.qjs_ref, value_ref) };
        // let ref_c = unsafe { get_ref_count(in_context.qjs_ref, value_ref) };

        // Unlike JSC QuickJS starts with this value having a refcount of 1, so we don't need to up
        // it when we create a reference

        QJSValue {
            raw,
            context: in_context.clone(),
        }
    }

    fn from_bool(bool_val: bool, in_context: &Rc<QJSContext>) -> Self {
        let val = QJSRawValue {
            tag: JS_TAG_BOOL as i64,
            u: JSValueUnion {
                int32: if bool_val { 1 } else { 0 },
            },
        };
        Self::from_raw(val, &in_context)
    }

    pub fn exception(in_context: &Rc<QJSContext>) -> Self {
        let val = EXCEPTION_RAW;
        Self::from_raw(val, in_context)
    }
}

pub(crate) const EXCEPTION_RAW: QJSRawValue = QJSRawValue {
    tag: JS_TAG_EXCEPTION as i64,
    u: JSValueUnion { int32: 0 },
};

impl Drop for QJSValue {
    fn drop(&mut self) {
        unsafe { free_value(self.context.raw, self.raw) }
    }
}

#[cfg(test)]
mod test {
    use super::QJSValue;
    use esperanto_shared::trait_tests::jsvalue_tests;
    #[test]
    fn converts_to_number() {
        jsvalue_tests::converts_to_number::<QJSValue>()
    }

    #[test]
    fn converts_to_string() {
        jsvalue_tests::converts_to_string::<QJSValue>()
    }

    #[test]
    fn converts_from_number() {
        jsvalue_tests::converts_from_number::<QJSValue>()
    }

    #[test]
    fn can_call_functions() {
        jsvalue_tests::can_call_functions::<QJSValue>()
    }

    #[test]
    fn can_wrap_rust_closure_with_one_argument() {
        jsvalue_tests::can_wrap_rust_closure_with_one_argument::<QJSValue>();
    }

    #[test]
    fn can_wrap_rust_closure_with_two_arguments() {
        jsvalue_tests::can_wrap_rust_closure_with_two_arguments::<QJSValue>();
    }
}
