#[cfg(test)]
mod test {
    use esperanto::export::{JSClassFunction, JSExportAttribute, JSExportMetadata};
    use esperanto::{js_result, JSValue};
    use esperanto::{JSContext, JSExportClass};
    use esperanto::{Retain, TryJSValueFrom};
    use phf::phf_ordered_map;
    use std::convert::{TryFrom, TryInto};

    #[test]
    fn exports_sets_constructor() {
        struct TestStruct {}

        impl JSExportClass for TestStruct {
            const METADATA: esperanto::export::JSExportMetadata = JSExportMetadata {
                class_name: "TestStruct",
                attributes: None,
                call_as_constructor: None,
                call_as_function: None,
            };
        }

        let test = TestStruct {};
        let ctx = JSContext::new().unwrap();
        let wrapped = JSValue::new_wrapped_native(test, &ctx).unwrap();
        let constructor: Retain<JSValue> =
            JSValue::constructor_for::<TestStruct, _>(&ctx, js_result::retain).unwrap();
        assert_eq!(wrapped.is_instance_of(&constructor).unwrap(), true);
    }

    #[test]
    fn exports_calls_constructor_successfully() {
        struct TestStruct {}

        impl JSExportClass for TestStruct {
            const METADATA: esperanto::export::JSExportMetadata = JSExportMetadata {
                class_name: "TestStruct",
                attributes: None,
                call_as_constructor: Some(JSClassFunction {
                    num_args: 1,
                    func: |args, ctx| {
                        let item = TestStruct {};
                        return JSValue::new_wrapped_native(item, ctx);
                    },
                }),
                call_as_function: None,
            };
        }

        let ctx = JSContext::new().unwrap();
        let wrapped = JSValue::constructor_for::<TestStruct, _>(&ctx, js_result::retain).unwrap();
        ctx.global_object()
            .set_property("TestValue", &wrapped)
            .unwrap();

        let result = ctx.evaluate("new TestValue()", None).unwrap();

        let is_instance = result.is_instance_of(&wrapped).unwrap();
        assert_eq!(is_instance, true)
    }

    #[test]
    fn exports_safely_fails_with_no_constructor() {
        struct TestStruct {}

        impl JSExportClass for TestStruct {
            const METADATA: esperanto::export::JSExportMetadata = JSExportMetadata {
                class_name: "TestStruct",
                attributes: None,
                call_as_constructor: None,
                call_as_function: None,
            };
        }

        let ctx = JSContext::new().unwrap();
        let wrapped = JSValue::constructor_for::<TestStruct>(&ctx).unwrap();
        ctx.global_object()
            .set_property("TestValue", &wrapped)
            .unwrap();

        let result = ctx.evaluate("new TestValue()", None);
        let err_result = result.unwrap_err();
        match err_result {
            esperanto::EsperantoError::JavaScriptError(err) => {
                assert_eq!(err.message, "Class TestStruct does not have a constuctor");
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
            const METADATA: esperanto::export::JSExportMetadata = JSExportMetadata {
                class_name: "TestStruct",
                attributes: None,
                call_as_constructor: Some(JSClassFunction {
                    num_args: 1,
                    func: |args, ctx| {
                        let num = f64::try_from(args[0])?;
                        let str = String::try_from(args[1])?;

                        let item = TestStruct {
                            value_one: num,
                            value_two: str,
                        };
                        return JSValueRef::new_wrapped_native(item, ctx);
                    },
                }),
                call_as_function: None,
            };
        }

        let ctx = JSContext::new().unwrap();

        let wrapped = JSValueRef::constructor_for::<TestStruct>(&ctx).unwrap();

        let f = JSValueRef::new_function(
            "return new TestStruct(123,'test')",
            vec!["TestStruct"],
            &ctx,
        )
        .unwrap();

        let result = f.call_as_function(vec![&wrapped]).unwrap();
        let as_ref: &TestStruct = result.get_native(&ctx).unwrap();

        assert_eq!(as_ref.value_one, 123 as f64);
        assert_eq!(as_ref.value_two, "test");
    }

    #[test]
    fn reuses_prototypes() {
        struct TestStruct {}

        impl JSExportClass for TestStruct {
            const METADATA: esperanto::export::JSExportMetadata = JSExportMetadata {
                class_name: "TestStruct",
                attributes: None,
                call_as_constructor: Some(JSClassFunction {
                    num_args: 0,
                    func: |_, ctx| {
                        let item = TestStruct {};
                        return JSValueRef::new_wrapped_native(item, &ctx);
                    },
                }),
                call_as_function: None,
            };
        }

        let ctx = JSContext::new().unwrap();
        ctx.global_object()
            .set_property(
                "TestValue",
                &JSValueRef::constructor_for::<TestStruct>(&ctx).unwrap(),
            )
            .unwrap();

        ctx.global_object()
            .set_property(
                "TestValue2",
                &JSValueRef::constructor_for::<TestStruct>(&ctx).unwrap(),
            )
            .unwrap();

        let first = ctx.evaluate("new TestValue()", None).unwrap();
        let second = ctx.evaluate("new TestValue2()", None).unwrap();
        let first_constructor = first.get_property("constructor").unwrap();
        let second_constructor = second.get_property("constructor").unwrap();
        assert_eq!(first_constructor, second_constructor)
    }

    #[test]
    fn works_across_garbage_collections() {
        struct TestStruct {}

        impl JSExportClass for TestStruct {
            const METADATA: esperanto::export::JSExportMetadata = JSExportMetadata {
                class_name: "TestStruct",
                attributes: None,
                call_as_constructor: Some(JSClassFunction {
                    num_args: 0,
                    func: |_, ctx| {
                        let item = TestStruct {};
                        return JSValueRef::new_wrapped_native(item, &ctx);
                    },
                }),
                call_as_function: None,
            };
        }

        let ctx = JSContext::new().unwrap();
        {
            ctx.global_object()
                .set_property(
                    "TestValue",
                    &JSValueRef::constructor_for::<TestStruct>(&ctx).unwrap(),
                )
                .unwrap();

            ctx.evaluate("new TestValue()", None).unwrap();
            ctx.global_object().delete_property("TestValue").unwrap();
            ctx.garbage_collect();
        }

        ctx.global_object()
            .set_property(
                "TestValue2",
                &JSValueRef::constructor_for::<TestStruct>(&ctx).unwrap(),
            )
            .unwrap();

        ctx.evaluate("new TestValue2()", None).unwrap();
    }

    #[test]
    fn calls_as_function() {
        struct TestStruct {
            val: f64,
        }

        impl JSExportClass for TestStruct {
            const METADATA: esperanto::export::JSExportMetadata = JSExportMetadata {
                class_name: "TestStruct",
                attributes: None,
                call_as_constructor: None,
                call_as_function: Some(JSClassFunction {
                    num_args: 2,
                    func: |args, ctx| {
                        let from_arg: f64 = args[0].try_into()?;
                        let from_arg_two: f64 = args[1].try_into()?;
                        let obj = TestStruct {
                            val: from_arg * from_arg_two,
                        };
                        return JSValueRef::new_wrapped_native(obj, &ctx);
                    },
                }),
            };
        }

        let ctx = JSContext::new().unwrap();
        let constructor = JSValueRef::prototype_for::<TestStruct>(&ctx).unwrap();
        ctx.global_object()
            .set_property("TestStruct", &constructor)
            .unwrap();

        let result = ctx.evaluate("TestStruct(10, 23)", None).unwrap();
        let as_struct: &TestStruct = result.get_native(&ctx).unwrap();
        assert_eq!(as_struct.val, 230.0);
    }

    #[test]
    fn exports_safely_fails_with_no_function() {
        struct TestStruct {}

        impl JSExportClass for TestStruct {
            const METADATA: esperanto::export::JSExportMetadata = JSExportMetadata {
                class_name: "TestStruct",
                attributes: None,
                call_as_constructor: None,
                call_as_function: None,
            };
        }

        let ctx = JSContext::new().unwrap();
        let wrapped = JSValueRef::constructor_for::<TestStruct>(&ctx).unwrap();
        ctx.global_object()
            .set_property("TestValue", &wrapped)
            .unwrap();

        let result = ctx.evaluate("TestValue()", None);
        let err_result = result.unwrap_err();
        match err_result {
            esperanto::EsperantoError::JavaScriptError(err) => {
                assert_eq!(
                    err.message,
                    "Class TestStruct cannot be called as a function"
                );
            }
            _ => panic!("Unexpected result"),
        }
    }

    #[ignore]
    #[test]
    fn export_attribute_property_getters_work() {
        struct TestStruct {}

        impl JSExportClass for TestStruct {
            const METADATA: esperanto::export::JSExportMetadata = JSExportMetadata {
                class_name: "TestStruct",
                attributes: Some(phf_ordered_map!(
                    "testAttribute" => JSExportAttribute::Property {
                        getter: &| ctx, this_obj | {
                            JSValueRef::try_new_value_from(123.0, &ctx)
                        },
                        setter: None
                    }
                )),
                call_as_constructor: None,
                call_as_function: None,
            };
        }

        let ctx = JSContext::new().unwrap();
        let wrapped = JSValueRef::constructor_for::<TestStruct>(&ctx).unwrap();
        ctx.global_object()
            .set_property("TestValue", &wrapped)
            .unwrap();

        let result = ctx.evaluate("TestValue.testAttribute", None).unwrap();
        assert!(JSValueRef::undefined(&ctx) != result);
        let number: f64 = result.try_into().unwrap();
        assert_eq!(number, 123.0);
    }
}
