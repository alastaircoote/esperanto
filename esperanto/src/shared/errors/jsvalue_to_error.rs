// use crate::shared::errors::CatchExceptionError;
// use crate::shared::errors::JavaScriptError;
// use crate::EsperantoError;
// use crate::JSValueRef;

// pub(crate) fn jsvalue_to_error(exception: JSValueRef) -> Result<JavaScriptError, EsperantoError> {
//     let name_jsvalue = exception.get_property("name")?;
//     let message_jsvalue = exception.get_property("message")?;

//     let name_string = match name_jsvalue.is_string() {
//         // Some QuickJS errors appear to not have a name property, in that instance
//         // let's just return a default.
//         false => "Error".to_string(),
//         true => name_jsvalue.to_string(),
//     };

//     let message_string = message_jsvalue.to_string();

//     Ok(JavaScriptError::new(name_string, message_string))
// }
