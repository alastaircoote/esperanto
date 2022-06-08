#[cfg(test)]
mod test {
    use std::convert::TryInto;

    use esperanto::{js_export_class, JSContext};
    use esperanto::{JSValueRef, TryJSValueFrom};

    #[test]
    fn exports_sets_prototype() {
        struct TestStruct {}

        js_export_class! { TestStruct as "TestStruct" =>
            call_as_function: (ctx, _this_obj, _values) {
                Ok(JSValueRef::try_new_value_from(1234, ctx)?.into())
            },
        };

        let test = TestStruct {};
        let ctx = JSContext::new().unwrap();
        let wrapped = JSValueRef::wrap_native(test, &ctx).unwrap();
        let proto = JSValueRef::prototype_for::<TestStruct>(&ctx).unwrap();

        assert_eq!(wrapped.is_instance_of(&proto).unwrap(), true);
    }

    #[test]
    fn exports_call_as_function_successfully() {
        struct TestStruct {}

        js_export_class! { TestStruct as "TestStruct" =>
            call_as_function: (ctx, _this_obj, _values) {
                Ok(JSValueRef::try_new_value_from(1234, ctx)?.into())
            },
        };

        let test = TestStruct {};
        let ctx = JSContext::new().unwrap();
        let wrapped = JSValueRef::wrap_native(test, &ctx).unwrap();

        ctx.global_object()
            .set_property("testValue", &wrapped)
            .unwrap();

        let str = ctx.evaluate("new String(testValue)", None).unwrap();
        println!("{}", str.to_string());

        let result = ctx.evaluate("testValue()", None).unwrap();
        let num: i32 = result.try_into().unwrap();
        assert_eq!(num, 1234)
    }

    #[test]
    fn exports_calls_constructor_successfully() {
        struct TestStruct {}

        js_export_class! { TestStruct as "TestStruct" =>
            call_as_constructor: (_, _lifetime) {
                Ok(TestStruct {})
            },
        };

        let ctx = JSContext::new().unwrap();
        let wrapped = JSValueRef::prototype_for::<TestStruct>(&ctx).unwrap();
        ctx.global_object()
            .set_property("TestValue", &wrapped)
            .unwrap();

        let str = ctx.evaluate("TestValue.toString()", None).unwrap();
        println!("{}", str.to_string());

        let result = ctx.evaluate("new TestValue()", None).unwrap();

        println!("{}", result.to_string());
        println!(
            "are they equal? {}",
            ctx.evaluate("new TestValue() instanceof TestValue", None)
                .unwrap()
        );

        let is_instance = result.is_instance_of(&wrapped).unwrap();
        assert_eq!(is_instance, true)
    }

    #[test]
    fn reuses_prototypes() {
        struct TestStruct {}

        js_export_class! { TestStruct as "TestStruct" =>
            call_as_constructor: (_, _lifetime) {
                Ok(TestStruct {})
            },
        };

        let ctx = JSContext::new().unwrap();
        ctx.global_object()
            .set_property(
                "TestValue",
                &JSValueRef::prototype_for::<TestStruct>(&ctx).unwrap(),
            )
            .unwrap();

        ctx.global_object()
            .set_property(
                "TestValue2",
                &JSValueRef::prototype_for::<TestStruct>(&ctx).unwrap(),
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

        js_export_class! { TestStruct as "TestStruct" =>
            call_as_constructor: (_, _lifetime) {
                Ok(TestStruct {})
            },
        };

        let ctx = JSContext::new().unwrap();
        {
            ctx.global_object()
                .set_property(
                    "TestValue",
                    &JSValueRef::prototype_for::<TestStruct>(&ctx).unwrap(),
                )
                .unwrap();

            ctx.evaluate("new TestValue()", None).unwrap();
            ctx.global_object().delete_property("TestValue").unwrap();
            ctx.garbage_collect();
        }

        ctx.global_object()
            .set_property(
                "TestValue2",
                &JSValueRef::prototype_for::<TestStruct>(&ctx).unwrap(),
            )
            .unwrap();

        ctx.evaluate("new TestValue2()", None).unwrap();
    }

    #[test]
    fn stringy() {
        struct TestStruct {}

        js_export_class! { TestStruct as "TestStruct" =>

        };

        let ctx = JSContext::new().unwrap();
        let wrapped = JSValueRef::wrap_native(TestStruct {}, &ctx).unwrap();
        let to_str: String = wrapped.try_into().unwrap();
        println!("huh: {}", to_str)
    }
}
