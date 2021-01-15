#[cfg(test)]
mod test {
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
            f.write_fmt(format_args!("This is a test error"))
        }
    }

    impl Error for TestError {}

    #[test]
    fn can_call_static_function_on_global_object() {
        struct TestStruct {}

        esperanto::js_export_definition!("TestStruct" => TestStruct, {
            static_functions: {
                "testString" => test_str |args| {
                    Ok("test_string")
                }
            }
        });

        let global = TestStruct {};
        let ctx = JSContext::new_with_global(None, global).unwrap();
        let result = ctx.evaluate("testString()", None).unwrap();
        drop(result);
        let as_str = String::try_from(&result).unwrap();
        assert_eq!(as_str, "test_string")
    }

    #[test]
    fn function_can_throw_error() {
        struct TestStruct {}

        esperanto::js_export_definition!("TestStruct" => TestStruct, {
            static_functions: {
                "throwError" => test_err |args| {
                    Result::<(),_>::Err(EsperantoError::external(TestError {}))
                }
            }
        });

        let global = TestStruct {};
        let ctx = JSContext::new_with_global(None, global).unwrap();
        let result = ctx
            .evaluate(
                "var err; try { throwError() } catch (ex) {err = ex}; err",
                None,
            )
            .unwrap();
        let as_str = String::try_from(&result).unwrap();
        assert_eq!(as_str, "Error: This is a test error")
    }

    #[test]
    fn drops_wrapped_object() {
        static mut WAS_DROPPED: bool = false;
        struct TestStruct {}

        impl Drop for TestStruct {
            fn drop(&mut self) {
                unsafe { WAS_DROPPED = true }
            }
        }

        esperanto::js_export_definition!("TestStruct" => TestStruct, {
            static_functions: {}
        });

        let global = TestStruct {};
        let ctx = JSContext::new_with_global(None, global).unwrap();
        drop(ctx);
        unsafe { assert_eq!(WAS_DROPPED, true) }
    }
}
