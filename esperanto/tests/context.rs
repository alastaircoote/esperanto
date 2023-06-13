#[cfg(test)]
mod test {

    use esperanto::{EsperantoError, JSContext};

    #[test]
    fn creates_context_successfully() {
        JSContext::new().unwrap();
    }

    #[test]
    fn evaluates_code_successfully() {
        let ctx = JSContext::new().unwrap();
        let result = ctx.evaluate("['one','two'].join(', ')", None).unwrap();
        let str: String = result.try_convert().unwrap();
        assert_eq!(str, "one, two");
    }

    #[test]
    fn saved_values_withstand_garbage_collection() {
        // not a great test as I can't verify that garbage collection actually does anything right now...
        let ctx = JSContext::new().unwrap();
        let result = ctx.evaluate("'hello'", None).unwrap();
        ctx.garbage_collect();
        let str: String = result.try_convert().unwrap();
        assert_eq!(str, "hello");
    }

    #[test]
    fn catches_errors() {
        let ctx = JSContext::new().unwrap();
        let result = ctx.evaluate("throw new Error('woah')", None).unwrap_err();
        match result {
            EsperantoError::JavaScriptError(err) => {
                assert_eq!(err.name, "Error");
                assert_eq!(err.message, "woah")
            }
            _ => panic!("Unexpected error type"),
        }
    }

    #[test]
    fn catches_invalid_syntax() {
        let ctx = JSContext::new().unwrap();
        let result = ctx.evaluate("][", None).unwrap_err();
        match result {
            EsperantoError::JavaScriptError(err) => {
                assert_eq!(err.name, "SyntaxError");
            }
            _ => panic!("Unexpected error type"),
        }
    }
}
