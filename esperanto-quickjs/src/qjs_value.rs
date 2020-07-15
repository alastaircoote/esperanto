use crate::qjs_shared_context_ref::SharedQJSContextRef;
// use crate::ref_count::{dup_value, free_value, get_ref_count};
use crate::ref_count::free_value;
use esperanto_shared::errors::JSConversionError;
use esperanto_shared::traits::JSValue;
use libquickjs_sys::{JSValue as QJSValueRef, JS_ToCStringLen2, JS_TAG_STRING};
use std::convert::TryFrom;
use std::rc::Rc;

pub struct QJSValue {
    qjs_ref: QJSValueRef,
    context_ref: Rc<SharedQJSContextRef>,
}

impl JSValue for QJSValue {}

impl QJSValue {
    pub(crate) fn new(value_ref: QJSValueRef, in_context: Rc<SharedQJSContextRef>) -> Self {
        // unsafe { dup_value(in_context.qjs_ref, value_ref) };
        // let ref_c = unsafe { get_ref_count(in_context.qjs_ref, value_ref) };

        // Unlike JSC QuickJS starts with this value having a refcount of 1, so we don't need to up
        // it when we create a reference

        QJSValue {
            qjs_ref: value_ref,
            context_ref: in_context,
        }
    }
}

impl TryFrom<QJSValue> for &str {
    type Error = JSConversionError;
    fn try_from(value: QJSValue) -> Result<Self, Self::Error> {
        if value.qjs_ref.tag != JS_TAG_STRING as i64 {
            return Err(JSConversionError::ConversionFailed);
        }

        let c_string_ptr = unsafe {
            JS_ToCStringLen2(
                value.context_ref.qjs_ref,
                std::ptr::null_mut(),
                value.qjs_ref,
                0,
            )
        };

        let cstr = unsafe { std::ffi::CStr::from_ptr(c_string_ptr) };
        cstr.to_str()
            .map_err(|_| JSConversionError::ConversionFailed)
    }
}

impl Drop for QJSValue {
    fn drop(&mut self) {
        unsafe { free_value(self.context_ref.qjs_ref, self.qjs_ref) }
    }
}
