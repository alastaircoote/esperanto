#[cfg(test)]
mod value_tests {

    use std::convert::TryInto;

    use esperanto_engine_javascriptcore::{JSCoreContext, JSCoreValue};
    use esperanto_engine_shared::{traits::JSContext, traits::JSValue, traits::TryJSValueFrom};
    use test_impl::test_impl;

    #[test_impl(
        (JSContext = JSCoreContext),
        (JSValue = JSCoreValue)
    )]
    fn from_f64() {
        let ctx = JSContext::new().unwrap();
        let val = JSValue::try_from(5.1, &ctx).unwrap();
        ctx.global_object()
            .unwrap()
            .set_property("test", &val)
            .unwrap();

        let result: bool = ctx
            .evaluate("test === 5.1", None)
            .unwrap()
            .try_into()
            .unwrap();

        assert_eq!(result, true)
    }

    #[test_impl(
        (JSContext = JSCoreContext),
        (JSValue = JSCoreValue)
    )]
    fn from_string() {
        let ctx = JSContext::new().unwrap();
        let val_str = JSValue::try_from("test_string".to_string(), &ctx).unwrap();

        let global = ctx.global_object().unwrap();
        global.set_property("val_str", &val_str).unwrap();

        let result: bool = ctx
            .evaluate("val_str === 'test_string'", None)
            .unwrap()
            .try_into()
            .unwrap();

        assert_eq!(result, true)
    }

    #[test_impl(
        (JSContext = JSCoreContext),
        (JSValue = JSCoreValue)
    )]
    fn from_bool() {
        let ctx = JSContext::new().unwrap();
        let val_true = JSValue::try_from(true, &ctx).unwrap();
        let val_false = JSValue::try_from(false, &ctx).unwrap();

        let global = ctx.global_object().unwrap();
        global.set_property("val_true", &val_true).unwrap();
        global.set_property("val_false", &val_false).unwrap();

        let result: String = ctx
            .evaluate("[val_true, val_false].join(',')", None)
            .unwrap()
            .try_into()
            .unwrap();

        assert_eq!(result, "true,false")
    }
}
