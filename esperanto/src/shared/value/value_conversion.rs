use std::ffi::CString;

use super::value::ValueResult;
use super::JSValueInternal;
use crate::shared::errors::JavaScriptError;
use crate::shared::{engine_impl::JSValueInternalImpl, errors::EsperantoError};
use crate::{
    shared::errors::{ConversionError, EsperantoResult},
    JSContext, JSValue, Retain,
};

pub trait TryJSValueFrom<'c>: Sized {
    fn try_jsvalue_from(value: Self, in_context: &'c JSContext) -> ValueResult<'c>;
}

pub trait JSValueFrom<'c>: Sized {
    fn jsvalue_from(value: Self, in_context: &'c JSContext<'c>) -> Retain<JSValue<'c>>;
}

pub trait TryConvertJSValue: Sized {
    fn try_from_jsvalue(value: &JSValue) -> EsperantoResult<Self>;
}

// impl<T> TryFrom<&JSValue<'_>> for T {}

// impl<T: TryFromJSValue> TryInto<T> for &JSValue {}

impl<'c, Target> TryJSValueFrom<'c> for Target
where
    Target: JSValueFrom<'c>,
{
    fn try_jsvalue_from(
        value: Target,
        in_context: &'c JSContext,
    ) -> EsperantoResult<Retain<JSValue<'c>>> {
        Ok(Self::jsvalue_from(value, in_context))
    }
}

// String

impl TryConvertJSValue for String {
    fn try_from_jsvalue(value: &JSValue<'_>) -> EsperantoResult<Self> {
        let cstring = value.internal.as_cstring(value.context.internal)?;
        let str = cstring
            .to_str()
            .map_err(|e| ConversionError::CouldNotConvertFromJSString(e))?;

        Ok(str.to_string())
    }
}

impl<'c> TryJSValueFrom<'c> for &str {
    fn try_jsvalue_from(value: &str, in_context: &'c JSContext) -> ValueResult<'c> {
        let cstring =
            CString::new(value).map_err(|e| ConversionError::CouldNotConvertToJSString(e))?;

        let ptr = JSValueInternalImpl::from_cstring(&cstring, in_context.internal);
        let val = JSValue::wrap_internal(ptr, in_context);
        Ok(Retain::new(val, true))
    }
}

// f64

impl<'c> TryJSValueFrom<'c> for f64 {
    fn try_jsvalue_from(value: f64, in_context: &'c JSContext) -> ValueResult<'c> {
        let ptr = JSValueInternalImpl::from_number(value, in_context.internal)?;
        let val = JSValue::wrap_internal(ptr, in_context);
        Ok(Retain::new(val, true))
    }
}

impl TryConvertJSValue for f64 {
    fn try_from_jsvalue(value: &JSValue<'_>) -> EsperantoResult<Self> {
        let n = value.internal.as_number(value.context.internal)?;
        Ok(n)
    }
}

// i32

impl<'c> TryJSValueFrom<'c> for i32 {
    fn try_jsvalue_from(value: i32, in_context: &'c JSContext) -> ValueResult<'c> {
        f64::try_jsvalue_from(value as f64, in_context)
    }
}

impl TryConvertJSValue for i32 {
    fn try_from_jsvalue(value: &JSValue<'_>) -> EsperantoResult<Self> {
        let n: f64 = value.try_convert()?;
        Ok(n as i32)
    }
}

// bool

impl<'c> TryJSValueFrom<'c> for bool {
    fn try_jsvalue_from(value: bool, in_context: &'c JSContext) -> ValueResult<'c> {
        let ptr = JSValueInternalImpl::from_bool(value, in_context.internal)?;
        let val = JSValue::wrap_internal(ptr, in_context);
        Ok(Retain::new(val, true))
    }
}

impl TryConvertJSValue for bool {
    fn try_from_jsvalue(value: &JSValue<'_>) -> EsperantoResult<Self> {
        value.internal.as_bool(value.context.internal)
    }
}

// Error

impl<'c> JSValueFrom<'c> for EsperantoError {
    fn jsvalue_from(value: EsperantoError, context: &'c JSContext<'c>) -> Retain<JSValue<'c>> {
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
        let val = JSValue::wrap_internal(internal, context);
        Retain::new(val, true)
    }
}

impl TryConvertJSValue for JavaScriptError {
    fn try_from_jsvalue(value: &JSValue<'_>) -> EsperantoResult<Self> {
        if value.is_error() == false {
            return Err(ConversionError::JSValueWasNotAnError.into());
        }

        let name: String = value.get_property("name")?.try_convert()?;
        let msg: String = value.get_property("message")?.try_convert()?;

        return Ok(JavaScriptError::new(name, msg));
    }
}

impl TryConvertJSValue for EsperantoError {
    fn try_from_jsvalue(value: &JSValue<'_>) -> EsperantoResult<Self> {
        let js_err: Result<JavaScriptError, EsperantoError> = value.try_convert();

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
        shared::{context::JSContextError, errors::JavaScriptError, value::TryConvertJSValue},
        EsperantoError, JSContext, JSValue,
    };

    macro_rules! check_eval {
        ($type:ty, $script: expr, $value: expr) => {
            let ctx = JSContext::new().unwrap();
            let value = ctx.evaluate($script, None).unwrap();
            let converted: $type = value.try_convert().unwrap();
            assert_eq!(converted, $value);
        };
    }

    macro_rules! check_comparison {
        ($value: expr, $js_string_match: expr) => {
            let ctx = JSContext::new().unwrap();
            let converted = JSValue::try_new_from($value, &ctx).unwrap();
            ctx.global_object()
                .set_property("testValue", &converted)
                .unwrap();
            let is_match: bool = (&ctx
                .evaluate(concat!("testValue === ", $js_string_match), None)
                .unwrap())
                .try_convert()
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
        check_comparison!("hello", "'hello'");
    }

    #[test]
    fn converts_from_js_error() {
        let ctx = JSContext::new().unwrap();

        let err_jsval = ctx.evaluate("new Error('test value')", None).unwrap();
        let err = JavaScriptError::try_from_jsvalue(&err_jsval).unwrap();
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
        let err = JavaScriptError::try_from_jsvalue(&err_jsval).unwrap();
        assert_eq!(err.name, "CustomError");
        assert_eq!(err.message, "test value");
    }

    #[test]
    fn converts_from_native_error() {
        let ctx = JSContext::new().unwrap();

        let expected_string = JSContextError::CouldNotCreateContext.to_string();

        let converted = JSValue::try_new_from(
            EsperantoError::ContextError(JSContextError::CouldNotCreateContext),
            &ctx,
        )
        .unwrap();

        let err = JavaScriptError::try_from_jsvalue(&converted).unwrap();

        assert_eq!(err.name, "NativeError");
        assert_eq!(err.message, expected_string);
    }
}
