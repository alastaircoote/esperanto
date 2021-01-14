// use esperanto::{js_export_method, js_export_test, private::jscore_context::JSCoreContext};
use std::{any::TypeId, convert::TryInto, fmt::Display};
use std::{convert::TryFrom, error::Error};
// use Hmm::what::wat::hmm;
use esperanto::{convert::TryFromJSValueRef, jsruntime::*, jsvalue::Value};
use esperanto::{jscontext::*, private::jscore_export::JSExportError};
// use thiserror::Error;
use esperanto::EsperantoError;

#[derive(Debug)]
struct TestError {}

impl Display for TestError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("An error occurred"))
    }
}

impl Error for TestError {}

struct Hmm {}

impl Hmm {
    fn what(strr: &String, num: f64) -> String {
        let mut i = 0.0;
        let mut st: String = "".to_string();
        while i < num {
            i = i + 1.0;
            st.push_str(&strr)
        }
        st
    }
}

esperanto::js_export_definition!("Hmm" => Hmm, {
    static_functions: {
        "testFunction2" => test_f |args| {
            let s = String::try_from(&args
                .next()
                .ok_or(JSExportError::NotEnoughArguments)??)?;

            let f = f64::try_from_js_ref(&args.next().ok_or(JSExportError::NotEnoughArguments)??)?;
            Ok(Hmm::what(&s,f))
        },
        "testString" => test_str |args| {
            Ok("test_string")
        },
        "throwError" => test_err |args| {
            Result::<(),_>::Err(EsperantoError::external(TestError {}))
            // Err()
        }
    }
    // "testFunction" => func(what, 1),
    // "testFunction" => func(what, 1, selfa)
});

#[test]
fn can_call_static_function_on_global_object() {
    let runtime = JSRuntime::new().unwrap();
    let global = Hmm {};
    let ctx = runtime.create_context_with_global_object(global).unwrap();
    let result = ctx.evaluate("testString()", None).unwrap();
    let as_str = String::try_from(&result).unwrap();
    assert_eq!(as_str, "test_string")
}

#[test]
fn function_can_throw_error() {
    let runtime = JSRuntime::new().unwrap();
    let global = Hmm {};
    let ctx = runtime.create_context_with_global_object(global).unwrap();
    let result = ctx
        .evaluate(
            "var err; try { throwError() } catch (ex) {err = ex}; err",
            None,
        )
        .unwrap();
    let msg = result.get_property("message").unwrap();
    let as_str = String::try_from(&result).unwrap();
    assert_eq!(as_str, "test_string")
}
