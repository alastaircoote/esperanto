#[cfg(test)]
mod context_tests {
    use esperanto_engine_javascriptcore::JSCoreContext;
    use esperanto_engine_shared::{errors::JSContextError, traits::JSContext};
    use test_impl::test_impl;

    #[test_impl(JSContext = JSCoreContext)]
    fn creates_successfully() {
        JSContext::new().unwrap();
    }

    #[test_impl(JSContext = JSCoreContext)]
    fn evaluates_code_successfully() {
        let ctx = JSContext::new().unwrap();
        ctx.evaluate("var one = 1; one", None).unwrap();
    }

    #[test_impl(JSContext = JSCoreContext)]
    fn catches_evaluation_errors() {
        let ctx = JSContext::new().unwrap();
        let err = ctx.evaluate("throw new Error('oh no')", None).unwrap_err();
        match err {
            JSContextError::JSErrorOccurred(js_text) => {
                assert_eq!(js_text, "Error: oh no");
            }
            _ => panic!("Unexpected error"),
        }
    }
}
