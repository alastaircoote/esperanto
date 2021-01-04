macro_rules! check_jscore_exception {
    ($ctx:expr, $exception:ident => $stmt:expr) => {{
        // use std::convert::TryFrom;
        let mut exception_val: *const javascriptcore_sys::OpaqueJSValue = std::ptr::null();
        let $exception = &mut exception_val;
        let out = $stmt;
        if exception_val.is_null() {
            Ok(out)
        } else {
            let raw_ref = unsafe {
                javascriptcore_sys::JSValueToStringCopy(
                    $ctx.raw_ref,
                    exception_val,
                    std::ptr::null_mut(),
                )
            };
            let js_string: crate::jscore::jscore_string::JSCoreString = raw_ref.into();

            Err(crate::errors::EsperantoError::JSErrorOccurred(
                match String::try_from(&js_string) {
                    Ok(s) => s,
                    Err(_) => "(could not get string from error)".to_string(),
                },
            ))
        }
    }};
}
