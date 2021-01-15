macro_rules! check_jscore_exception {
    ($ctx:expr, $exception:ident => $stmt:expr) => {{
        let mut exception_val: *const javascriptcore_sys::OpaqueJSValue = std::ptr::null();
        let $exception = &mut exception_val;
        let out = $stmt;
        if exception_val.is_null() {
            Ok(out)
        } else {
            let message_js_core_string = JSCoreString::try_from("message")?;
            let message_raw_ref = unsafe {
                javascriptcore_sys::JSObjectGetProperty(
                    $ctx.raw_ref,
                    exception_val as *mut javascriptcore_sys::OpaqueJSValue,
                    message_js_core_string.raw_ref,
                    std::ptr::null_mut(),
                )
            };

            let raw_ref = unsafe {
                javascriptcore_sys::JSValueToStringCopy(
                    $ctx.raw_ref,
                    message_raw_ref,
                    std::ptr::null_mut(),
                )
            };
            let js_string: crate::jscore::jscore_string::JSCoreString = raw_ref.into();

            Err(crate::EsperantoError::JSErrorOccurred(
                match String::try_from(&js_string) {
                    Ok(s) => s,
                    Err(_) => "(could not get string from error)".to_string(),
                },
            ))
        }
    }};
}
