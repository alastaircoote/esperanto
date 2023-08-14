macro_rules! check_jscore_exception {
    ($ctx:expr, $exception:ident => $stmt:expr) => {{
        let mut exception_val: *const javascriptcore_sys::OpaqueJSValue = std::ptr::null();
        let $exception = &mut exception_val;
        let out = $stmt;
        if exception_val.is_null() {
            Ok(out)
        } else {
            let val: crate::jscore::jscorevaluepointer::JSCoreValuePointer = exception_val.into();
            let js_error_convert =
                crate::shared::value::JSValueImplementation::to_js_error(val, $ctx);
            crate::shared::value::JSValueImplementation::release(val, $ctx);

            match js_error_convert {
                Ok(is_error) => Err(crate::EsperantoError::JavaScriptError(is_error)),
                Err(conv_error) => Err(conv_error),
            }
        }
    }};
}
