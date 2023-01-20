#[cfg(test)]
mod value_tests {
    use std::convert::TryFrom;

    use esperanto::errors::JSValueError;
    use esperanto::export::JSExportMetadata;
    use esperanto::{EsperantoError, JSContext, JSExportClass, JSValueRef, TryJSValueFrom};

    #[test]
    fn sets_properties() {
        let ctx = JSContext::new().unwrap();
        let value_to_set = JSValueRef::try_new_value_from(123, &ctx).unwrap();

        let obj = ctx
            .evaluate("var testObject = {}; testObject", None)
            .unwrap();

        obj.set_property("testValue", &value_to_set).unwrap();

        let result = ctx.evaluate("testObject.testValue", None).unwrap();
        assert_eq!(i32::try_from(&result).unwrap(), 123);
    }

    #[test]
    fn errors_when_cannot_set_property() {
        let ctx = JSContext::new().unwrap();
        let value_to_set = JSValueRef::try_new_value_from(123, &ctx).unwrap();

        let obj = ctx
            .evaluate("var testObject = true; testObject", None)
            .unwrap();

        let err = obj.set_property("testValue", &value_to_set).unwrap_err();

        if let EsperantoError::ValueError(v) = err {
            assert_eq!(v, JSValueError::IsNotAnObject);
        } else {
            panic!("Unexpected error type")
        }
    }

    #[test]
    fn gets_properties() {
        let ctx = JSContext::new().unwrap();

        let obj = ctx
            .evaluate("var obj = {testValue: 456}; obj", None)
            .unwrap();

        let value = obj.get_property("testValue").unwrap();

        assert_eq!(i32::try_from(&value).unwrap(), 456);
    }

    #[test]
    fn calls_function() {
        let ctx = JSContext::new().unwrap();
        let func = ctx
            .evaluate("(function(one,two) { return one + two })", None)
            .unwrap();
        let arg_one = JSValueRef::try_new_value_from(1200, &ctx).unwrap();
        let arg_two = JSValueRef::try_new_value_from(34, &ctx).unwrap();
        let result = func.call_as_function(vec![&arg_one, &arg_two]).unwrap();
        assert_eq!(i32::try_from(&result).unwrap(), 1234);
    }

    #[test]
    fn errors_when_calling_non_function() {
        let ctx = JSContext::new().unwrap();
        let not_func = JSValueRef::undefined(&ctx);
        let result = not_func.call_as_function(vec![]).unwrap_err();
        match result {
            EsperantoError::JavaScriptError(err) => {
                assert_eq!(err.name, "TypeError")
            }
            _ => {
                panic!("Unexpected error: {}", result)
            }
        }
    }

    #[test]
    fn calls_function_bound() {
        let ctx = JSContext::new().unwrap();
        let bound_obj = ctx.evaluate("({testValue: 4567})", None).unwrap();
        let func = ctx
            .evaluate("(function() { return this.testValue })", None)
            .unwrap();
        let result = func
            .call_as_function_bound(vec![], Some(&bound_obj))
            .unwrap();
        assert_eq!(i32::try_from(&result).unwrap(), 4567);
    }

    #[test]
    fn creates_function() {
        let ctx = JSContext::new().unwrap();
        let body = "return one * two";
        let func = JSValueRef::new_function(body, vec!["one", "two"], &ctx).unwrap();

        let arg_one = JSValueRef::try_new_value_from(5, &ctx).unwrap();
        let arg_two = JSValueRef::try_new_value_from(25, &ctx).unwrap();

        let arguments = vec![&arg_one, &arg_two];
        let result = func.call_as_function(arguments).unwrap();
        assert_eq!(i32::try_from(&result).unwrap(), 125);
    }

    #[test]
    fn calls_constructor() {
        let ctx = JSContext::new().unwrap();
        let class = ctx.evaluate("(class TestClass {})", None).unwrap();
        let instance = class.call_as_constructor(vec![]).unwrap();
        let checked_value = instance.get_property("constructor").unwrap();

        assert_eq!(checked_value, class);
    }

    #[test]
    fn check_equality_of_ints() {
        let ctx = JSContext::new().unwrap();
        let first_num = ctx.evaluate("1234", None).unwrap();
        let second_num = ctx.evaluate("1234", None).unwrap();
        let third_num = ctx.evaluate("2345", None).unwrap();
        assert_eq!(first_num, second_num);
        assert_ne!(first_num, third_num);
    }

    #[test]
    fn check_equality_of_floats() {
        let ctx = JSContext::new().unwrap();
        let first_num = ctx.evaluate("1234.56", None).unwrap();
        let second_num = ctx.evaluate("1234.56", None).unwrap();
        let third_num = ctx.evaluate("2345.67", None).unwrap();
        assert_eq!(first_num, second_num);
        assert_ne!(first_num, third_num);
    }

    #[test]
    fn check_equality_of_strings() {
        let ctx = JSContext::new().unwrap();
        let first_string = ctx.evaluate("'hello there'", None).unwrap();
        let second_string = ctx.evaluate("'hello there'", None).unwrap();
        let third_string = ctx.evaluate("'there, hello'", None).unwrap();
        assert_eq!(first_string, second_string);
        assert_ne!(first_string, third_string);
    }

    #[test]
    fn check_equality_of_values() {
        let ctx = JSContext::new().unwrap();
        ctx.evaluate("var testValue = {}", None).unwrap();
        let first_value = ctx.evaluate("testValue", None).unwrap();
        let second_value = ctx.evaluate("testValue", None).unwrap();
        let third_value = ctx.evaluate("({})", None).unwrap();
        assert_eq!(first_value, second_value);
        assert_ne!(first_value, third_value);
    }

    #[test]
    fn is_instance_of_works() {
        let ctx = JSContext::new().unwrap();
        let class = ctx
            .evaluate("class NotThisClass{}; class Test {}; Test", None)
            .unwrap();

        let instance = ctx.evaluate("new Test()", None).unwrap();
        let is_instance = instance.is_instance_of(&class).unwrap();
        assert_eq!(is_instance, true);

        let other_instance = ctx.evaluate("new NotThisClass()", None).unwrap();
        let other_is_instance = other_instance.is_instance_of(&class).unwrap();
        assert_eq!(other_is_instance, false)
    }

    #[test]
    fn wraps_native_objects() {
        struct TestStruct {
            num_value: u32,
        }

        impl JSExportClass for TestStruct {
            const METADATA: esperanto::export::JSExportMetadata<'static> = JSExportMetadata {
                class_name: "TestStruct",
                attributes: None,
                call_as_constructor: None,
            };
        }

        let str = TestStruct { num_value: 12345 };
        let ctx = JSContext::new().unwrap();

        let wrapped = JSValueRef::wrap_native(str, &ctx).unwrap();

        let as_ref = wrapped.get_native::<TestStruct>(&ctx).unwrap();
        assert_eq!(as_ref.num_value, 12345);
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
            const METADATA: esperanto::export::JSExportMetadata<'static> = JSExportMetadata {
                class_name: "TestStruct",
                attributes: None,
                call_as_constructor: None,
            };
        }

        let str = TestStruct {};
        let ctx = JSContext::new().unwrap();

        let wrapped = JSValueRef::wrap_native(str, &ctx).unwrap();
        drop(wrapped);
        ctx.garbage_collect();

        unsafe { assert_eq!(IS_DESTROYED, true) };
    }
}
