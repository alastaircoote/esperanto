use crate::errors::JSContextError;
use crate::traits::JSContext;
use std::fmt::Debug;

pub fn it_evaluates_correct_code<Context: JSContext>()
where
    f64: std::convert::TryFrom<Context::ValueType>,
{
    let runtime = Context::new().unwrap();
    let _: Context::ValueType = runtime.evaluate("1+2").unwrap();
}

pub fn it_throws_exceptions_on_invalid_code<Context: JSContext>()
where
    Context::ValueType: Debug,
{
    let runtime = Context::new().unwrap();
    let result = runtime.evaluate("]").unwrap_err();
    match result {
        JSContextError::JavaScriptErrorOccurred(js_error) => {
            assert_eq!(js_error.name, "SyntaxError")
        }
        _ => panic!("Should have returned JSError"),
    }
}
