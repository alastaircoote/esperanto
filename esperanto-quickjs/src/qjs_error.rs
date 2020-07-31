use crate::{qjs_context::QJSContext, qjs_value::QJSValue, ref_count::free_value};
use esperanto_shared::{errors::JSError, traits::JSValue};
use libquickjs_sys::{
    JSValue as QJSValueRef, JS_FreeCString, JS_GetException, JS_GetPropertyStr, JS_ToCStringLen2,
    JS_TAG_EXCEPTION, JS_TAG_STRING, JS_TAG_UNDEFINED,
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
    let exception = unsafe { QJSValue::from_raw(JS_GetException(context_ref.raw), context_ref) };
    if exception.raw.tag == JS_TAG_UNDEFINED as i64 {
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

    let name_ref = unsafe { JS_GetPropertyStr(context_ref.raw, exception.raw, name_str.as_ptr()) };
    let message_ref =
        unsafe { JS_GetPropertyStr(context_ref.raw, exception.raw, message_str.as_ptr()) };

    if name_ref.tag != JS_TAG_STRING as i64 || message_ref.tag != JS_TAG_STRING as i64 {
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
                free_value(context_ref.raw, name_ref);
                free_value(context_ref.raw, message_ref);
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
        if value_ref.tag != JS_TAG_EXCEPTION as i64 {
            return Ok(());
        }

        Err(best_effort_get_error(context_ref).unwrap_or(JSError::unknown()))
    }
}
