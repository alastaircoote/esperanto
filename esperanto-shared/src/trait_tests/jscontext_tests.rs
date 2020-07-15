use crate::errors::{JSEnvError, JSError};
use crate::traits::JSContext;
use std::{convert::TryInto, fmt::Debug};

pub fn it_evaluates_correct_code<Context: JSContext>()
where
    f64: std::convert::TryFrom<Context::ValueType>,
{
    let runtime = Context::new();
    let _: Context::ValueType = runtime.evaluate("1+2").unwrap();
}

pub fn it_throws_exceptions_on_invalid_code<Context: JSContext>() {
    let runtime = Context::new();
    match runtime.evaluate("]") {
        Ok(_) => panic!("This call should not succeed"),
        Err(err) => match err {
            JSEnvError::JSErrorEncountered(err) => assert_eq!(err.name, "SyntaxError"),
            _ => panic!("Should have returned JSError"),
        },
    }
}
