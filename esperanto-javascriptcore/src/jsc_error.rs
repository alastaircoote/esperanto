use super::jsc_string::JSCString;
use crate::jsc_sharedcontextref::JSCSharedContextRef;
use esperanto_shared::errors::JSError;
use javascriptcore_sys::{JSObjectGetProperty, JSValueRef, JSValueToObject, JSValueToStringCopy};
use std::rc::Rc;

pub trait JSErrorFromJSC {
    fn check_jsc_value_ref(
        v: JSValueRef,
        in_context: &Rc<JSCSharedContextRef>,
    ) -> Result<(), JSError>;
}

impl JSErrorFromJSC for JSError {
    fn check_jsc_value_ref(
        v: JSValueRef,
        in_context: &Rc<JSCSharedContextRef>,
    ) -> Result<(), JSError> {
        if v.is_null() {
            return Ok(());
        }

        let maybe = || unsafe {
            let mut exception: JSValueRef = std::ptr::null_mut();

            let obj_ref = JSValueToObject(in_context.jsc_ref, v, &mut exception);
            if exception.is_null() == false {
                return None;
            }

            // Don't care what the errors are here, so we turn them into optionals:
            let name_property = JSCString::from_string("name").ok();
            let message_property = JSCString::from_string("message").ok();

            match (name_property, message_property) {
                (Some(name_exists), Some(message_exists)) => {
                    let name_ref = JSObjectGetProperty(
                        in_context.jsc_ref,
                        obj_ref,
                        name_exists.jsc_ref,
                        &mut exception,
                    );
                    if exception.is_null() == false {
                        return None;
                    }

                    let message_ref = JSObjectGetProperty(
                        in_context.jsc_ref,
                        obj_ref,
                        message_exists.jsc_ref,
                        &mut exception,
                    );
                    if exception.is_null() == false {
                        return None;
                    }

                    let name_string =
                        JSValueToStringCopy(in_context.jsc_ref, name_ref, &mut exception);
                    if exception.is_null() == false {
                        return None;
                    }

                    let msg_string =
                        JSValueToStringCopy(in_context.jsc_ref, message_ref, &mut exception);
                    if exception.is_null() == false {
                        return None;
                    }

                    let final_name = JSCString::from_ptr(name_string).to_string().ok();
                    let final_msg = JSCString::from_ptr(msg_string).to_string().ok();

                    match (final_name, final_msg) {
                        (Some(n), Some(m)) => Some(JSError {
                            name: n,
                            message: m,
                        }),
                        _ => None,
                    }
                }
                _ => return None,
            }
        };

        Err(maybe().unwrap_or(JSError::unknown()))
    }
}
