use std::{convert::TryFrom, ffi::CString};

use super::js_result;
use super::JSValueInternal;
use crate::shared::errors::JavaScriptError;
use crate::shared::{engine_impl::JSValueInternalImpl, errors::EsperantoError};
use crate::{
    shared::errors::{ConversionError, EsperantoResult},
    JSContext, JSValue,
};
use std::convert::TryInto;

pub trait TryJSValueFrom<'c, T>: Sized {
    fn try_new_value_from(value: T, in_context: &'c JSContext) -> EsperantoResult<Self>;
}

pub trait JSValueFrom<'c, T>: Sized {
    fn new_value_from(value: T, in_context: &'c JSContext) -> Self;
}

pub trait TryFromJSValue: Sized {
    fn try_from(value: &JSValue) -> EsperantoResult<Self>;
}

// impl<T> TryFrom<&JSValue<'_>> for T {}

// impl<T: TryFromJSValue> TryInto<T> for &JSValue {}

impl<'c, Source, Target> TryJSValueFrom<'c, Source> for Target
where
    Target: JSValueFrom<'c, Source>,
{
    fn try_new_value_from(value: Source, in_context: &'c JSContext) -> EsperantoResult<Self> {
        Ok(Self::new_value_from(value, in_context))
    }
}

// pub trait TryIntoJSContext<T>: Sized {
//     fn try_into(&self, in_context: &JSContext) -> EsperantoResult<T>;
// }

// impl<S, T> TryIntoJSContext<T> for S
// where
//     T: TryFromInJSContext<S>,
// {
//     fn try_into(&self, in_context: &JSContext) -> EsperantoResult<T> {
//         T::from_in_context(self, in_context)
//     }
// }

// String

impl TryFrom<&JSValue<'_>> for String {
    type Error = EsperantoError;

    fn try_from(value: &JSValue<'_>) -> Result<Self, Self::Error> {
        let cstring = value.internal.as_cstring(value.context.internal)?;
        let str = cstring
            .to_str()
            .map_err(|e| ConversionError::CouldNotConvertFromJSString(e))?;

        Ok(str.to_string())
    }
}

impl<'c> TryJSValueFrom<'c, String> for JSValue<'c> {
    fn try_new_value_from(value: String, in_context: &'c JSContext) -> EsperantoResult<Self> {
        let cstring =
            CString::new(value).map_err(|e| ConversionError::CouldNotConvertToJSString(e))?;

        let ptr = JSValueInternalImpl::from_cstring(&cstring, in_context.internal);
        let val = Self::wrap_internal(ptr, in_context);
        Ok(val)
    }
}

// f64

impl<'c> TryJSValueFrom<'c, f64> for JSValue<'c> {
    fn try_new_value_from(value: f64, in_context: &'c JSContext) -> EsperantoResult<Self> {
        let ptr = JSValueInternalImpl::from_number(value, in_context.internal)?;
        let val = Self::wrap_internal(ptr, in_context);
        Ok(val)
    }
}

impl TryFrom<&JSValue<'_>> for f64 {
    type Error = EsperantoError;

    fn try_from(value: &JSValue<'_>) -> Result<Self, Self::Error> {
        let n = value.internal.as_number(value.context.internal)?;
        Ok(n)
    }
}

// i32

impl<'c> TryJSValueFrom<'c, i32> for JSValue<'c> {
    fn try_new_value_from(value: i32, in_context: &'c JSContext) -> EsperantoResult<Self> {
        Self::try_new_value_from(value as f64, in_context)
    }
}

impl TryFrom<&JSValue<'_>> for i32 {
    type Error = EsperantoError;

    fn try_from(value: &JSValue<'_>) -> Result<Self, Self::Error> {
        let n = f64::try_from(value)?;
        Ok(n as i32)
    }
}

// bool

impl<'c> TryJSValueFrom<'c, bool> for JSValue<'c> {
    fn try_new_value_from(value: bool, in_context: &'c JSContext) -> EsperantoResult<Self> {
        let ptr = JSValueInternalImpl::from_bool(value, in_context.internal)?;
        let val = Self::wrap_internal(ptr, in_context);
        Ok(val)
    }
}

impl<'c> TryFrom<&JSValue<'c>> for bool {
    type Error = EsperantoError;

    fn try_from(value: &JSValue<'c>) -> Result<Self, Self::Error> {
        value.internal.as_bool(value.context.internal)
    }
}

// Error

impl<'c> JSValueFrom<'c, EsperantoError> for JSValue<'c> {
    fn new_value_from(value: EsperantoError, context: &'c JSContext<'c>) -> Self {
        let internal = match value {
            // Is this error already a JavaScriptError? If so we can recreate the original name and
            // message properties.
            EsperantoError::JavaScriptError(err) => {
                let error_name =
                    CString::new(err.name).unwrap_or(CString::new("NativeError").unwrap());
                let error_message = CString::new(err.message)
                    .unwrap_or(CString::new("[could not decode message").unwrap());
                JSValueInternalImpl::new_error(error_name, error_message, context.internal)
            }
            // If not then we'll make up "NativeError" as the name and pass through the message:
            _ => {
                let error_name = CString::new("NativeError").unwrap();
                let error_message = CString::new(value.to_string())
                    .unwrap_or(CString::new("[could not decode message").unwrap());
                JSValueInternalImpl::new_error(error_name, error_message, context.internal)
            }
        };
        Self::wrap_internal(internal, context)
    }
}

impl<'c> TryFrom<JSValue<'c>> for JavaScriptError {
    type Error = EsperantoError;

    fn try_from(value: JSValue<'c>) -> Result<Self, Self::Error> {
        if value.is_error() == false {
            return Err(ConversionError::JSValueWasNotAnError.into());
        }

        let name: String = value.get_property("name", js_result::convert)?;
        let msg: String = value.get_property("message", js_result::convert)?;

        return Ok(JavaScriptError::new(name, msg));
    }
}

impl<'c> TryFrom<JSValue<'c>> for EsperantoError {
    type Error = EsperantoError;

    fn try_from(value: JSValue<'c>) -> Result<Self, Self::Error> {
        let js_err = JavaScriptError::try_from(value);

        match js_err {
            Ok(js_err) => Ok(EsperantoError::JavaScriptError(js_err)),
            Err(native_error) => Err(native_error),
        }
    }
}

// Native

// impl<'c, T> TryJSValueFrom<'c, T> for JSValueRef<'c>
// where
//     T: JSExportClass,
// {
//     fn try_new_value_from(value: T, in_context: &'c JSContext) -> EsperantoResult<Self> {
//         let ptr = JSValueInternalImpl::from_native_class(value, &in_context.internal)?;
//         let val = JSValueRef::wrap_internal(ptr, in_context);
//         Ok(val)
//     }
// }

#[cfg(test)]
mod test {
    use crate::{
        shared::{context::JSContextError, errors::JavaScriptError},
        EsperantoError, JSContext, JSValue, TryJSValueFrom,
    };
    use std::convert::{TryFrom, TryInto};

    macro_rules! check_eval {
        ($type:ty, $script: expr, $value: expr) => {
            let ctx = JSContext::new().unwrap();
            let value = ctx.evaluate($script, None).unwrap();
            let converted: $type = (&value).try_into().unwrap();
            assert_eq!(converted, $value);
        };
    }

    macro_rules! check_comparison {
        ($value: expr, $js_string_match: expr) => {
            let ctx = JSContext::new().unwrap();
            let converted = JSValue::try_new_value_from($value, &ctx).unwrap();
            ctx.global_object()
                .set_property("testValue", &converted)
                .unwrap();
            let is_match: bool = (&ctx
                .evaluate(concat!("testValue === ", $js_string_match), None)
                .unwrap())
                .try_into()
                .unwrap();
            assert_eq!(is_match, true);
        };
    }

    #[test]
    fn converts_to_bool() {
        check_eval!(bool, "true", true);
        check_eval!(bool, "false", false);
    }

    #[test]
    fn converts_from_bool() {
        check_comparison!(true, "true");
        check_comparison!(false, "false");
    }

    #[test]
    fn converts_to_f64() {
        check_eval!(f64, "1234", 1234.0);
        check_eval!(f64, "12.34", 12.34);
    }

    #[test]
    fn converts_from_f64() {
        check_comparison!(12.34, "12.34");
        check_comparison!(1234, "1234");
    }

    #[test]
    fn converts_to_string() {
        check_eval!(String, "'hello'", "hello");
    }

    #[test]
    fn converts_from_string() {
        check_comparison!("hello".to_string(), "'hello'");
    }

    #[test]
    fn converts_from_js_error() {
        let ctx = JSContext::new().unwrap();

        let err_jsval = ctx.evaluate("new Error('test value')", None).unwrap();
        let err = JavaScriptError::try_from(err_jsval).unwrap();
        assert_eq!(err.name, "Error");
        assert_eq!(err.message, "test value");
    }

    #[test]
    fn converts_from_custom_js_error() {
        let ctx = JSContext::new().unwrap();

        let err_jsval = ctx
            .evaluate(
                "
            class CustomError extends Error {
                constructor(msg) {
                    super(msg);
                    this.name = 'CustomError';
                }
            }
            new CustomError('test value')
        ",
                None,
            )
            .unwrap();
        let err = JavaScriptError::try_from(err_jsval).unwrap();
        assert_eq!(err.name, "CustomError");
        assert_eq!(err.message, "test value");
    }

    #[test]
    fn converts_from_native_error() {
        let ctx = JSContext::new().unwrap();

        let expected_string = JSContextError::CouldNotCreateContext.to_string();

        let converted = JSValue::try_new_value_from(
            EsperantoError::ContextError(JSContextError::CouldNotCreateContext),
            &ctx,
        )
        .unwrap();

        let err = JavaScriptError::try_from(converted).unwrap();

        assert_eq!(err.name, "NativeError");
        assert_eq!(err.message, expected_string);
    }
}
