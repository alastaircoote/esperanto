use std::ffi::CString;

use super::value::ValueResult;
use super::JSValueImplementation;
use crate::shared::errors::JavaScriptError;
use crate::shared::{engine_impl::JSValueInternalImpl, errors::EsperantoError};
use crate::{
    shared::errors::{ConversionError, EsperantoResult},
    JSContext, JSValue, Retain,
};

pub trait TryJSValueFrom<'r, 'c>: Sized
where
    'r: 'c,
{
    fn try_jsvalue_from(value: Self, in_context: &'c JSContext<'r, 'c>) -> ValueResult<'r, 'c>;
}

pub trait JSValueFrom<'r: 'c, 'c>: Sized {
    fn jsvalue_from(value: Self, in_context: &'c JSContext<'r, 'c>) -> Retain<JSValue<'r, 'c>>;
}

pub trait TryConvertJSValue<'r: 'c, 'c>: Sized {
    fn try_from_jsvalue(value: &JSValue<'r, 'c>) -> EsperantoResult<Self>;
}

// Rather than repeat all these lifetime requirements a million times over
// we'll just use a macro

macro_rules! try_to_js_value {
    ($target_type:ty, ($value: ident, $in_context:ident) => $body:expr) => {
        impl<'r, 'c> TryJSValueFrom<'r, 'c> for $target_type
        where
            'r: 'c,
        {
            fn try_jsvalue_from(
                $value: $target_type,
                $in_context: &'c JSContext<'r, 'c>,
            ) -> ValueResult<'r, 'c> {
                $body
            }
        }
    };
}

macro_rules! try_from_js_value {
    ($target_type:ty, ($value: ident) => $body:expr) => {
        impl<'r: 'c, 'c> TryConvertJSValue<'r, 'c> for $target_type {
            fn try_from_jsvalue($value: &JSValue<'_, '_>) -> EsperantoResult<Self> {
                $body
            }
        }
    };
}

impl<'r, 'c, Target> TryJSValueFrom<'r, 'c> for Target
where
    Target: JSValueFrom<'r, 'c>,
    'r: 'c,
{
    fn try_jsvalue_from(
        value: Target,
        in_context: &'c JSContext<'r, 'c>,
    ) -> EsperantoResult<Retain<JSValue<'r, 'c>>> {
        Ok(Self::jsvalue_from(value, in_context))
    }
}

// String

try_to_js_value! {&str, (value, in_context) => {
    let cstring = CString::new(value).map_err(|e| ConversionError::CouldNotConvertToJSString(e))?;
    let ptr = JSValueInternalImpl::from_cstring(&cstring, in_context.implementation());
    let val = JSValue::wrap_internal(ptr, in_context);
    Ok(Retain::wrap(val))
}}

try_to_js_value! {String, (value, in_context) => {
    <&str>::try_jsvalue_from(value.as_str(), in_context)
}}

try_from_js_value! {String, (value) => {
    let cstring = value.internal.as_cstring(value.context.implementation())?;
    let str = cstring
        .to_str()
        .map_err(|e| ConversionError::CouldNotConvertFromJSString(e))?;

    Ok(str.to_owned())
}}

// f64

try_to_js_value! {f64, (value, in_context) => {
    let ptr = JSValueInternalImpl::from_number(value, in_context.implementation())?;
    let val = JSValue::wrap_internal(ptr, in_context);
    Ok(Retain::wrap(val))
}}

try_from_js_value! {f64, (value) => {
    let n = value.internal.as_number(value.context.implementation())?;
    Ok(n)
}}

// i32

try_to_js_value! {i32, (value, in_context) => {
    f64::try_jsvalue_from(value as f64, in_context)
}}

try_from_js_value! {i32, (value) => {
    let n: f64 = value.try_convert()?;
    Ok(n as i32)
}}

// bool

try_to_js_value! {bool, (value, in_context) => {
    let ptr = JSValueInternalImpl::from_bool(value, in_context.implementation())?;
    let val = JSValue::wrap_internal(ptr, in_context);
    Ok(Retain::wrap(val))
}}

try_from_js_value! {bool, (value) => {
    value.internal.as_bool(value.context.implementation())
}}

// Error

try_to_js_value! {EsperantoError, (value, in_context) => {
    let (name, message): (&str, String) = match &value {
        Self::RuntimeError(err) => ("RuntimeError", err.to_string()),
        Self::CatchExceptionError(err) => ("CatchExceptionError", err.to_string()),
        Self::ContextError(err) => ("ContextError", err.to_string()),
        Self::ConversionError(err) => ("ConversionError", err.to_string()),
        Self::ExportError(err) => ("ExportError", err.to_string()),
        Self::ValueError(err) => ("ValueError", err.to_string()),
        Self::JavaScriptError(err) => (&err.name, err.message.to_string()),
    };

    return Ok(Retain::wrap(JSValue::new_error(
        name, &message, in_context,
    )?));
}}

try_from_js_value! {JavaScriptError, (value) => {
    if value.is_error()? == false {
        return Err(ConversionError::JSValueWasNotAnError.into());
    }

    let name: String = value.get_property("name")?.try_convert()?;
    let msg: String = value.get_property("message")?.try_convert()?;

    return Ok(JavaScriptError::new(name, msg));
}}

// impl TryConvertJSValue<'_, '_> for EsperantoError {
//     fn try_from_jsvalue(value: &JSValue<'_, '_>) -> EsperantoResult<Self> {
//         let js_err: Result<JavaScriptError, EsperantoError> = value.try_convert();

//         match js_err {
//             Ok(js_err) => Ok(EsperantoError::JavaScriptError(js_err)),
//             Err(native_error) => Err(native_error),
//         }
//     }
// }

// // Native

// impl<'r, 'c, T> TryConvertJSValue<'r, 'c> for Js<'r, 'c, T>
// where
//     T: JSExportClass,
// {
//     fn try_from_jsvalue(value: &JSValue<'r, 'c>) -> EsperantoResult<Self> {
//         value.as_native()
//     }
// }

// impl<'r, 'c, T> TryJSValueFrom<'r, 'c> for Js<'r, 'c, T>
// where
//     T: JSExportClass,
// {
//     fn try_jsvalue_from(value: Self, _: &'c crate::JSContext) -> ValueResult<'r, 'c> {
//         Ok(Js::get_value(&value).retain())
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

        assert_eq!(err.name, "ContextError");
        assert_eq!(err.message, expected_string);
    }
}
