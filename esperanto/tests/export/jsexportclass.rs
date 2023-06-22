#[cfg(test)]
mod test {
    use esperanto::errors::{JSExportError, JavaScriptError};
    use esperanto::export::{JSClassFunction, JSExportAttribute, JSExportName, Js};
    use esperanto::{EsperantoError, JSRuntime, JSValue};
    use esperanto::{JSContext, JSExportClass};
    use phf::phf_ordered_map;
    use std::ops::Deref;

    #[test]
    fn exports_sets_prototype() {
        struct TestStruct {}

        impl JSExportClass for TestStruct {
            const CLASS_NAME: JSExportName = "TestStruct";
        }

        let test = TestStruct {};
        let ctx = JSContext::new().unwrap();
        let wrapped = JSValue::new_wrapped_native(test, &ctx).unwrap();
        let constructor = JSValue::prototype_for::<TestStruct>(&ctx).unwrap();
        assert_eq!(wrapped.is_instance_of(&constructor).unwrap(), true);
    }

    #[test]
    fn exports_calls_constructor_successfully() {
        struct TestStruct {}

        impl JSExportClass for TestStruct {
            const CLASS_NAME: JSExportName = "TestStruct";
            const CALL_AS_CONSTRUCTOR: Option<JSClassFunction> = Some(JSClassFunction {
                num_args: 1,
                func: |_, ctx| {
                    let item = TestStruct {};
                    // return JSValue::new_function("return hi", vec![], &ctx);
                    return JSValue::new_wrapped_native(item, ctx);
                },
            });
        }

        let ctx = JSContext::new().unwrap();
        let prototype = JSValue::prototype_for::<TestStruct>(&ctx).unwrap();
        ctx.global_object()
            .set_property("TestValue", &prototype)
            .unwrap();

        let result = ctx.evaluate("new TestValue()", None).unwrap();
        let is_instance = result.is_instance_of(&prototype).unwrap();
        assert_eq!(is_instance, true)
    }

    #[test]
    fn exports_safely_fails_with_no_constructor() {
        struct TestStruct {}

        impl JSExportClass for TestStruct {
            const CLASS_NAME: &'static str = "TestStruct";
        }

        let ctx = JSContext::new().unwrap();
        let wrapped = JSValue::prototype_for::<TestStruct>(&ctx).unwrap();
        ctx.global_object()
            .set_property("TestValue", &wrapped)
            .unwrap();

        let result = ctx.evaluate("new TestValue()", None);

        // assert_eq(result.is_err(), true);

        let err_result = result.unwrap_err();
        match err_result {
            esperanto::EsperantoError::JavaScriptError(err) => {
                // Can't easily inspect more than this because the JS engine implementation defines the specific
                // error message.
                assert_eq!(err.name, "TypeError");
            }
            _ => panic!("Unexpected result"),
        }
    }

    #[test]
    fn constructor_arguments_are_passed() {
        struct TestStruct {
            value_one: f64,
            value_two: String,
        }

        impl JSExportClass for TestStruct {
            const CLASS_NAME: &'static str = "TestStruct";
            const CALL_AS_CONSTRUCTOR: Option<JSClassFunction> = Some(JSClassFunction {
                num_args: 1,
                func: |args, ctx| {
                    let num: f64 = args[0].try_convert()?;
                    let str: String = args[1].try_convert()?;

                    let item = TestStruct {
                        value_one: num,
                        value_two: str,
                    };
                    return JSValue::new_wrapped_native(item, ctx);
                },
            });
        }

        let ctx = JSContext::new().unwrap();

        let wrapped = JSValue::prototype_for::<TestStruct>(&ctx).unwrap();

        let function = JSValue::new_function(
            "return new TestStruct(123,'test')",
            vec!["TestStruct"],
            &ctx,
        )
        .unwrap();

        let result = function.call_as_function(vec![&wrapped]).unwrap();
        let as_ref: Js<TestStruct> = result.as_native().unwrap();

        assert_eq!(as_ref.value_one, 123 as f64);
        assert_eq!(as_ref.value_two, "test");
    }

    #[test]
    fn throws_error_when_constructor_fails() {
        struct TestStruct {}

        impl JSExportClass for TestStruct {
            const CLASS_NAME: &'static str = "TestStruct";
            const CALL_AS_CONSTRUCTOR: Option<JSClassFunction> = Some(JSClassFunction {
                num_args: 0,
                func: |_, _| {
                    let js_err =
                        JavaScriptError::new("CustomError".into(), "This function failed".into());
                    return Err(esperanto::EsperantoError::JavaScriptError(js_err));
                },
            });
        }

        let ctx = JSContext::new().unwrap();

        let wrapped = JSValue::prototype_for::<TestStruct>(&ctx).unwrap();
        let result = wrapped.call_as_constructor(vec![]);

        let err = result.unwrap_err();
        match err {
            esperanto::EsperantoError::JavaScriptError(jserr) => {
                assert_eq!(jserr.name, "CustomError");
                assert_eq!(jserr.message, "This function failed")
            }
            _ => panic!("Unexpected error type returned"),
        }
    }

    #[test]
    fn reuses_prototypes() {
        struct TestStruct {}

        impl JSExportClass for TestStruct {
            const CLASS_NAME: &'static str = "TestStruct";
            const CALL_AS_CONSTRUCTOR: Option<JSClassFunction> = Some(JSClassFunction {
                num_args: 0,
                func: |_, ctx| {
                    let item = TestStruct {};
                    return JSValue::new_wrapped_native(item, &ctx);
                },
            });
        }

        let ctx = JSContext::new().unwrap();
        ctx.global_object()
            .set_property(
                "TestValue",
                &JSValue::prototype_for::<TestStruct>(&ctx).unwrap(),
            )
            .unwrap();

        ctx.global_object()
            .set_property(
                "TestValue2",
                &JSValue::prototype_for::<TestStruct>(&ctx).unwrap(),
            )
            .unwrap();

        let script = r#"
            const one = new TestValue();
            const two = new TestValue2();

            const one_prototype = Object.getPrototypeOf(one);
            const two_prototype = Object.getPrototypeOf(two);

            one_prototype === two_prototype;
        "#;

        let result = ctx.evaluate(script, None).unwrap();
        assert_eq!(result.try_convert::<bool>().unwrap(), true);
    }

    #[test]
    fn works_across_garbage_collections() {
        struct TestStruct {}

        impl JSExportClass for TestStruct {
            const CLASS_NAME: &'static str = "TestStruct";
            const CALL_AS_CONSTRUCTOR: Option<JSClassFunction> = Some(JSClassFunction {
                num_args: 0,
                func: |_, ctx| {
                    let item = TestStruct {};
                    return JSValue::new_wrapped_native(item, &ctx);
                },
            });
        }

        let ctx = JSContext::new().unwrap();
        {
            ctx.global_object()
                .set_property(
                    "TestValue",
                    &JSValue::constructor_for::<TestStruct>(&ctx).unwrap(),
                )
                .unwrap();

            ctx.evaluate("new TestValue()", None).unwrap();
            ctx.global_object().delete_property("TestValue").unwrap();
            ctx.garbage_collect();
        }

        ctx.global_object()
            .set_property(
                "TestValue2",
                &JSValue::constructor_for::<TestStruct>(&ctx).unwrap(),
            )
            .unwrap();

        ctx.evaluate("new TestValue2()", None).unwrap();
    }

    #[test]
    fn calls_as_function() {
        struct TestStruct {}

        impl JSExportClass for TestStruct {
            const CLASS_NAME: &'static str = "TestStruct";
            const CALL_AS_FUNCTION: Option<JSClassFunction> = Some(JSClassFunction {
                num_args: 2,
                func: |args, ctx| {
                    let obj = TestStruct {};
                    // return JSValue::try_new_from(123.0, &ctx);
                    return JSValue::new_wrapped_native(obj, &ctx);
                },
            });
        }

        let ctx = JSContext::new().unwrap();
        let constructor = JSValue::prototype_for::<TestStruct>(&ctx).unwrap();
        ctx.global_object()
            .set_property("TestStruct", &constructor)
            .unwrap();

        let result = ctx.evaluate("TestStruct(10, 23)", None).unwrap();
        result.as_native::<TestStruct>().unwrap();
    }

    #[test]
    fn function_args_are_passed() {
        struct TestStruct {
            val: f64,
        }

        impl JSExportClass for TestStruct {
            const CLASS_NAME: &'static str = "TestStruct";
            const CALL_AS_FUNCTION: Option<JSClassFunction> = Some(JSClassFunction {
                num_args: 2,
                func: |args, ctx| {
                    let from_arg: f64 = args[0].try_convert()?;
                    let from_arg_two: f64 = args[1].try_convert()?;
                    let obj = TestStruct {
                        val: from_arg * from_arg_two,
                    };
                    return JSValue::new_wrapped_native(obj, &ctx);
                },
            });
        }

        let ctx = JSContext::new().unwrap();
        let constructor = JSValue::prototype_for::<TestStruct>(&ctx).unwrap();
        ctx.global_object()
            .set_property("TestStruct", &constructor)
            .unwrap();

        let result = ctx.evaluate("TestStruct(10, 23)", None).unwrap();
        let as_struct: Js<TestStruct> = result.as_native().unwrap();
        assert_eq!(as_struct.val, 230.0);
    }
    #[test]
    fn exports_safely_fails_with_no_function() {
        struct TestStruct {}

        impl JSExportClass for TestStruct {
            const CLASS_NAME: &'static str = "TestStruct";
        }

        let ctx = JSContext::new().unwrap();
        let wrapped = JSValue::prototype_for::<TestStruct>(&ctx).unwrap();
        ctx.global_object()
            .set_property("TestValue", &wrapped)
            .unwrap();

        let result = ctx.evaluate("TestValue()", None);

        let err_result = result.unwrap_err();
        match err_result {
            esperanto::EsperantoError::JavaScriptError(err) => {
                assert_eq!(err.name, "TypeError");
            }
            _ => panic!("Unexpected result"),
        }
    }

    #[test]
    fn wraps_native_objects() {
        struct TestStruct {
            num_value: u32,
        }

        impl JSExportClass for TestStruct {
            const CLASS_NAME: &'static str = "TestStruct";
        }

        let str = TestStruct { num_value: 12345 };
        let ctx = JSContext::new().unwrap();

        let wrapped = JSValue::new_wrapped_native(str, &ctx).unwrap();

        let as_ref = wrapped.as_native::<TestStruct>().unwrap();
        assert_eq!(as_ref.num_value, 12345);
    }

    #[test]
    fn will_not_return_wrong_native_type() {
        struct TestStruct {}

        #[derive(Debug)]
        struct TestStruct2 {}

        impl JSExportClass for TestStruct {
            const CLASS_NAME: &'static str = "TestStruct";
        }

        impl JSExportClass for TestStruct2 {
            const CLASS_NAME: &'static str = "TestStruct2";
        }

        let str = TestStruct {};
        let ctx = JSContext::new().unwrap();

        let wrapped = JSValue::new_wrapped_native(str, &ctx).unwrap();

        let err = wrapped.as_native::<TestStruct2>().unwrap_err();
        assert_eq!(
            err,
            EsperantoError::ExportError(JSExportError::IncorrectNativeType {
                expected: "TestStruct2",
                actual: "TestStruct"
            })
        )
    }

    #[test]
    fn destroys_native_objects() {
        static mut IS_DESTROYED: bool = false;

        struct TestStruct {}

        impl Drop for TestStruct {
            fn drop(&mut self) {
                unsafe { IS_DESTROYED = true };
            }
        }

        impl JSExportClass for TestStruct {
            const CLASS_NAME: &'static str = "TestStruct";
        }

        // want this test to manually call the garbage collector but for whatever reason
        // manually calling GC on JSC doesn't seem to actually do anything. So instead
        // we rely on destroying the context to trigger the drop

        {
            let str = TestStruct {};
            let ctx = JSContext::new().unwrap();
            JSValue::new_wrapped_native(str, &ctx).unwrap();
        }
        unsafe { assert_eq!(IS_DESTROYED, true) };
    }

    #[ignore]
    #[test]
    fn export_attribute_property_getters_work() {
        struct TestStruct {}

        impl JSExportClass for TestStruct {
            const CLASS_NAME: &'static str = "TestStruct";
            const ATTRIBUTES: esperanto::export::JSExportAttributes = Some(phf_ordered_map!(
                "testAttribute" => JSExportAttribute::Property {
                    getter: &| ctx, this_obj | {
                        JSValue::try_new_from(123.0, &ctx)
                    },
                    setter: None
                }
            ));
        }

        let ctx = JSContext::new().unwrap();
        let wrapped = JSValue::constructor_for::<TestStruct>(&ctx).unwrap();
        ctx.global_object()
            .set_property("TestValue", &wrapped)
            .unwrap();

        let result = ctx.evaluate("TestValue.testAttribute", None).unwrap();
        assert!(JSValue::undefined(&ctx).value() != result.value());
        let number: f64 = result.try_convert().unwrap();
        assert_eq!(number, 123.0);
    }
}
