use crate::traits::{JSContext, JSValue};
use crate::{errors::JSContextError, traits::JSRuntime};
use std::{convert::TryInto, fmt::Debug};

// pub fn it_evaluates_correct_code<'a, Context: JSContext<'a>>(ctx: &'a Context) {
//     let result = ctx.evaluate("1+2").unwrap();
//     // drop(ctx);
//     let number: f64 = result.try_into().unwrap();
//     assert_eq!(number, 3.0)
// }

// pub fn it_throws_exceptions_on_invalid_code<'a, Context: JSContext<'a>>()
// where
//     Context::ValueType: Debug,
// {
//     let runtime = Context::new().unwrap();

//     let result = runtime.evaluate("]").unwrap_err();
//     match result {
//         JSContextError::JavaScriptErrorOccurred(js_error) => {
//             assert_eq!(js_error.name, "SyntaxError")
//         }
//         _ => panic!("Should have returned JSError"),
//     }
// }
