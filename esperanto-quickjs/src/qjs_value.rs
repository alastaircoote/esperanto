use crate::{qjs_context::QJSContext, qjs_function::wrap_closure};
use esperanto_shared::errors::{JSContextError, JSConversionError};
use esperanto_shared::traits::{FromJSValue, JSValue, ToJSValue};
use esperanto_shared::util::closures::{wrap_one_argument_closure, wrap_two_argument_closure};
use quickjs_android_suitable_sys::{
    JSValue as QJSRawValue, JS_Call, JS_Eval, JS_FreeCString, JS_FreeValue__, JS_GetPropertyStr,
    JS_NewBool__, JS_NewFloat64__, JS_ToBool, JS_ToCStringLen2, JS_ToFloat64,
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
            // .field("raw", unsafe { &self.raw.u.ptr })
            .finish()
    }
}

impl JSValue for QJSValue {
    type ContextType = QJSContext;
    type RawType = QJSRawValue;
    fn as_string(&self) -> Result<String, JSContextError> {
        // let qjs_string = JS_ToString(self.context_ref.qjs_ref, self.qjs_ref);
        let c_str_ptr = unsafe { JS_ToCStringLen2(self.context.raw, &mut 0, self.raw, 0) };
        let c_str = unsafe { CStr::from_ptr(c_str_ptr) };
        let string = c_str.to_str()?.to_string();
        unsafe { JS_FreeCString(self.context.raw, c_str_ptr) };
        Ok(string)
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

    fn from_number(
        number: f64,
        in_context: &Rc<Self::ContextType>,
    ) -> Result<Self, JSContextError> {
        let val = unsafe { JS_NewFloat64__(in_context.raw, number) };
        Self::from_raw(val, in_context)
    }

    fn from_one_arg_closure<
        I: FromJSValue<Self> + 'static,
        O: ToJSValue<Self> + 'static,
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
        I1: FromJSValue<Self> + 'static,
        I2: FromJSValue<Self> + 'static,
        O: ToJSValue<Self> + 'static,
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

    fn from_raw(
        raw: Self::RawType,
        in_context: &Rc<Self::ContextType>,
    ) -> Result<Self, JSContextError> {
        // unsafe { dup_value(in_context.qjs_ref, value_ref) };
        // let ref_c = unsafe { get_ref_count(in_context.qjs_ref, value_ref) };

        // Unlike JSC QuickJS starts with this value having a refcount of 1, so we don't need to up
        // it when we create a reference

        Ok(QJSValue {
            raw,
            context: in_context.clone(),
        })
    }

    fn from_bool(bool_val: bool, in_context: &Rc<QJSContext>) -> Result<Self, JSContextError> {
        let val = unsafe { JS_NewBool__(in_context.raw, if bool_val { 1 } else { 0 }) };
        Self::from_raw(val, &in_context)
    }

    fn as_bool(&self) -> Result<bool, JSContextError> {
        let val = unsafe { JS_ToBool(self.context.raw, self.raw) };
        Ok(val == 1)
    }

    fn get_property(&self, name: &str) -> Result<Self, JSContextError> {
        let name_cstring = CString::new(name)?;
        let property_ref =
            unsafe { JS_GetPropertyStr(self.context.raw, self.raw, name_cstring.as_ptr()) };

        Self::from_raw(property_ref, &self.context)
    }

    fn create_function(
        in_context: &Rc<Self::ContextType>,
        arg_names: Vec<&str>,
        body: &str,
    ) -> Result<Self, JSContextError> {
        // I might be wrong but I can't find a way to define a function from a string in QuickJS,
        // so we just use JS_Eval to do it. This is not good, as it might allow someone the ability
        // to immediately execute code if they're creative with {}s. Going to leave for now but
        // should revisit in the future. Maybe look at the struct that gets created to try to
        // manually construct one ourselves?

        let combined_arg_names = arg_names.join(",");
        let function_complete = format!("(function({}) {{ {} }})", combined_arg_names, body);
        let function_length = function_complete.len();
        let function_c_string = CString::new(function_complete)?;
        let result = unsafe {
            JS_Eval(
                in_context.raw,
                function_c_string.as_ptr(),
                function_length as _,
                CString::new("script.js")?.as_ptr(),
                0,
            )
        };
        Self::from_raw(result, in_context)
    }
}

impl QJSValue {
    // pub fn exception(in_context: &Rc<QJSContext>) -> Result<Self, JSContextError> {
    //     let val = EXCEPTION_RAW;
    //     Self::from_raw(val, in_context)
    // }
}

// pub(crate) const EXCEPTION_RAW: QJSRawValue = QJSRawValue {
//     tag: JS_TAG_EXCEPTION as i64,
//     u: JSValueUnion { int32: 0 },
// };

impl Drop for QJSValue {
    fn drop(&mut self) {
        unsafe { JS_FreeValue__(self.context.raw, self.raw) }
    }
}

// macro_rules! foo {
//     ($v:ident) => {
//         fn what<V>(v: V)
//         where
//             : JSValue,
//         {
//             v.asdasd
//         }
//     };
// }

// foo!(QJSValue);

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
    fn converts_from_boolean() {
        jsvalue_tests::converts_from_boolean::<QJSValue>()
    }

    #[test]
    fn converts_to_boolean() {
        jsvalue_tests::converts_to_boolean::<QJSValue>()
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

    #[test]
    fn can_get_properties() {
        jsvalue_tests::can_get_properties::<QJSValue>()
    }

    #[test]
    fn can_create_function() {
        jsvalue_tests::can_create_function::<QJSValue>();
    }
}
