use std::ffi::CString;

use javascriptcore_sys::{OpaqueJSContext, OpaqueJSValue};

use super::{jscorecontextpointer::JSCoreContextPointer, jscorevalue::JSCoreValueInternal};
use crate::shared::errors::{CatchExceptionError, EsperantoError};
use crate::shared::value::JSValueInternal;

/// Converts a JSValue error into our Rust equivalent. Note that this function assumes that the
/// value exists and that it's an error. You probably don't want to invoke this function directly,
/// you want to use the [check_jscore_exception!](check_jscore_exception) macro instead.
// pub(crate) fn catch_exception(
//     exception_value: *const OpaqueJSValue,
//     ctx: *const OpaqueJSContext,
// ) -> Result<EsperantoError, CatchExceptionError> {
//     let context = JSCoreContextPointer::from(ctx);
//     let exception = JSCoreValueInternal::from(exception_value);

//     let name_cstring = CString::new("name")?;
//     let name_jsvalue = exception.get_property(context, &name_cstring)?;
//     let name_value = name_jsvalue.as_cstring(context)?;

//     let msg_cstring = CString::new("message")?;
//     let msg_jsvalue = exception.get_property(context, &msg_cstring)?;
//     let msg_value = msg_jsvalue.as_cstring(context)?;

//     Ok(EsperantoError::JavaScriptError {
//         name: name_value.to_string_lossy().to_string(),
//         message: msg_value.to_string_lossy().to_string(),
//     })
// }

macro_rules! check_jscore_exception {
    ($ctx:expr, $exception:ident => $stmt:expr) => {{
        let mut exception_val: *const javascriptcore_sys::OpaqueJSValue = std::ptr::null();
        let $exception = &mut exception_val;
        let out = $stmt;
        if exception_val.is_null() {
            Ok(out)
        } else {
            let val: crate::jscore::jscorevaluepointer::JSCoreValuePointer = exception_val.into();

            let js_error_convert = crate::shared::value::JSValueInternal::to_js_error(val, $ctx);
            crate::shared::value::JSValueInternal::release(val, $ctx);

            match js_error_convert {
                Ok(is_error) => Err(crate::EsperantoError::JavaScriptError(is_error)),
                Err(conv_error) => Err(conv_error),
            }
        }
    }};
}
