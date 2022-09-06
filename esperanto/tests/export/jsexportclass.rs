#[cfg(test)]
mod test {
    use std::convert::{TryFrom, TryInto};

    use esperanto::export::{
        JSClassFunction, JSExportCall, JSExportMetadata, JSExportMetadataOptional,
    };
    use esperanto::{js_export_class, JSContext, JSExportClass};
    use esperanto::{JSValueRef, TryJSValueFrom};
    use quickjs_android_suitable_sys::JSValue;

    #[test]
    fn exports_sets_constructor() {
        struct TestStruct {}

        impl JSExportClass for TestStruct {
            const METADATA: esperanto::export::JSExportMetadata<'static> = JSExportMetadata {
                class_name: b"TestStruct\0" as _,
                attributes: None,
                call_as_constructor: None,
            };
        }

        let test = TestStruct {};
        let ctx = JSContext::new().unwrap();
        let wrapped = JSValueRef::wrap_native(test, &ctx).unwrap();
        let constructor = JSValueRef::constructor_for::<TestStruct>(&ctx).unwrap();

        assert_eq!(wrapped.is_instance_of(&constructor).unwrap(), true);
    }

    #[test]
    fn exports_calls_constructor_successfully() {
        struct TestStruct {}

        impl JSExportClass for TestStruct {
            const METADATA: esperanto::export::JSExportMetadata<'static> = JSExportMetadata {
                class_name: b"TestStruct\0" as _,
                attributes: None,
                call_as_constructor: Some(JSClassFunction {
                    num_args: 1,
                    func: &|_, ctx| {
                        let item = TestStruct {};
                        return JSValueRef::wrap_native(item, ctx);
                    },
                }),
            };
        }

        let ctx = JSContext::new().unwrap();
        let wrapped = JSValueRef::constructor_for::<TestStruct>(&ctx).unwrap();
        ctx.global_object()
            .set_property("TestValue", &wrapped)
            .unwrap();

        let result = ctx.evaluate("new TestValue()", None).unwrap();

        let is_instance = result.is_instance_of(&wrapped).unwrap();
        assert_eq!(is_instance, true)
    }

    #[test]
    fn constructor_arguments_are_passed() {
        struct TestStruct {
            value_one: f64,
            value_two: String,
        }

        impl JSExportClass for TestStruct {
            const METADATA: esperanto::export::JSExportMetadata<'static> = JSExportMetadata {
                class_name: b"TestStruct\0" as _,
                attributes: None,
                call_as_constructor: Some(JSClassFunction {
                    num_args: 1,
                    func: &|args, ctx| {
                        let num = f64::try_from(&args[0])?;
                        let str = String::try_from(&args[1])?;

                        let item = TestStruct {
                            value_one: num,
                            value_two: str,
                        };
                        return JSValueRef::wrap_native(item, ctx);
                    },
                }),
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
            const METADATA: esperanto::export::JSExportMetadata<'static> = JSExportMetadata {
                class_name: b"TestStruct\0" as _,
                attributes: None,
                call_as_constructor: None,
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
            const METADATA: esperanto::export::JSExportMetadata<'static> = JSExportMetadata {
                class_name: b"TestStruct\0" as _,
                attributes: None,
                call_as_constructor: None,
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
        struct TestStruct {}

        impl JSExportClass for TestStruct {
            const METADATA: esperanto::export::JSExportMetadata<'static> = JSExportMetadata {
                class_name: b"TestStruct\0" as _,
                attributes: None,
                call_as_constructor: None,
            };
        }

        let ctx = JSContext::new().unwrap();
        let constructor = JSValueRef::constructor_for::<TestStruct>(&ctx).unwrap();
        ctx.global_object()
            .set_property("TestStruct", &constructor)
            .unwrap();

        let result = ctx.evaluate("new TestStruct()", None).unwrap();
        println!("{}", result.to_string())
    }
}
