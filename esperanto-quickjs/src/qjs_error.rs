use crate::qjs_context::QJSContext;
use esperanto_shared::errors::JSError;
use quickjs_android_suitable_sys::{
    JSValue as QJSValueRef, JS_FreeCString, JS_FreeValue__, JS_GetException, JS_GetPropertyStr,
    JS_IsException__, JS_IsString__, JS_IsUndefined__, JS_ToCStringLen2,
};
use std::{
    ffi::{CStr, CString},
    rc::Rc,
};

pub(crate) trait QJSError {
    fn check_for_exception(value_ref: QJSValueRef, context: &Rc<QJSContext>)
        -> Result<(), JSError>;
}

fn best_effort_get_error(context_ref: &Rc<QJSContext>) -> Option<JSError> {
    let exception = unsafe { JS_GetException(context_ref.raw) };

    if unsafe { JS_IsUndefined__(exception) } == 1 {
        return None;
    }

    let name_str: CString;
    let message_str: CString;

    match (CString::new("name"), CString::new("message")) {
        (Ok(name), Ok(message)) => {
            name_str = name;
            message_str = message;
        }
        _ => return None,
    }

    let name_ref = unsafe { JS_GetPropertyStr(context_ref.raw, exception, name_str.as_ptr()) };
    let message_ref =
        unsafe { JS_GetPropertyStr(context_ref.raw, exception, message_str.as_ptr()) };

    if unsafe { JS_IsString__(name_ref) == 0 || JS_IsString__(message_ref) == 0 } {
        return None;
    }

    let name_string = unsafe { JS_ToCStringLen2(context_ref.raw, &mut 0, name_ref, 0) };
    let message_string = unsafe { JS_ToCStringLen2(context_ref.raw, &mut 0, message_ref, 0) };

    if name_string.is_null() || message_string.is_null() {
        return None;
    }
    unsafe {
        match (
            CStr::from_ptr(name_string).to_str(),
            CStr::from_ptr(message_string).to_str(),
        ) {
            (Ok(final_name), Ok(final_message)) => {
                JS_FreeValue__(context_ref.raw, name_ref);
                JS_FreeValue__(context_ref.raw, message_ref);
                JS_FreeCString(context_ref.raw, name_string);
                JS_FreeCString(context_ref.raw, message_string);

                Some(JSError {
                    name: final_name.to_string(),
                    message: final_message.to_string(),
                })
            }
            _ => None,
        }
    }
}

impl QJSError for JSError {
    fn check_for_exception(
        value_ref: QJSValueRef,
        context_ref: &Rc<QJSContext>,
    ) -> Result<(), JSError> {
        if unsafe { JS_IsException__(value_ref) } == 0 {
            return Ok(());
        }

        Err(best_effort_get_error(context_ref).unwrap_or(JSError::unknown()))
    }
}
