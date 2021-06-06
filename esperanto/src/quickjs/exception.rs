macro_rules! check_quickjs_exception {
    ($ctx:expr => $stmt:expr) => {{
        let result = $stmt;
        let exception = unsafe { quickjs_android_suitable_sys::JS_GetException(*$ctx) };

        if unsafe { quickjs_android_suitable_sys::JS_IsError(*$ctx, exception) } == 0 {
            Ok(result)
        } else {
            let js_error_convert =
                crate::shared::value::JSValueInternal::to_js_error(exception, $ctx);

            crate::shared::value::JSValueInternal::release(exception, $ctx);

            match js_error_convert {
                Ok(is_error) => Err(crate::EsperantoError::JavaScriptError(is_error)),
                Err(conv_error) => Err(conv_error),
            }
        }
    }};
}
