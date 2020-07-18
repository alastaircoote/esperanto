use crate::qjs_shared_context_ref::SharedQJSContextRef;
// use crate::ref_count::{dup_value, free_value, get_ref_count};
use crate::ref_count::free_value;
use esperanto_shared::errors::{JSContextError, JSConversionError};
use esperanto_shared::traits::{JSObject, JSValue};
use libquickjs_sys::{
    JSValue as QJSValueRef, JS_GetPropertyStr, JS_ToCStringLen2, JS_ToFloat64, JS_TAG_FLOAT64,
    JS_TAG_STRING,
};
use std::convert::{TryFrom, TryInto};
use std::ffi::{CStr, CString};
use std::rc::Rc;

pub struct QJSValue {
    qjs_ref: QJSValueRef,
    context_ref: Rc<SharedQJSContextRef>,
}

// For whatever reason Debug isn't implemented on QJSValueRef (but is on other QuickJS objects?)
// so we have to manually define Debug here.
impl std::fmt::Debug for QJSValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("QJSValue")
            .field("context_ref", &self.context_ref)
            .field("qjs_ref", unsafe { &self.qjs_ref.u.ptr })
            .finish()
    }
}

impl JSValue for QJSValue {
    type ObjectType = QJSValue;
    fn as_string(&self) -> Result<String, JSConversionError> {
        let c_str_ptr =
            unsafe { JS_ToCStringLen2(self.context_ref.qjs_ref, &mut 0, self.qjs_ref, 0) };
        let c_str = unsafe { CStr::from_ptr(c_str_ptr) };
        Ok(c_str.to_str()?.to_string())
    }
    fn to_object(self) -> Result<Self::ObjectType, esperanto_shared::errors::JSContextError> {
        Ok(self)
    }
}

// QuickJS doesn't really make a distinction between values and objects like JSC does
// so we just do it all on the same struct
impl JSObject for QJSValue {
    type ValueType = Self;
    fn get_property(&self, name: &str) -> Result<Self, JSContextError> {
        let name_cstring = CString::new(name)?;
        let property_ref = unsafe {
            JS_GetPropertyStr(
                self.context_ref.qjs_ref,
                self.qjs_ref,
                name_cstring.as_ptr(),
            )
        };

        Ok(Self::new(property_ref, &self.context_ref))
    }
}

impl QJSValue {
    pub(crate) fn new(value_ref: QJSValueRef, in_context: &Rc<SharedQJSContextRef>) -> Self {
        // unsafe { dup_value(in_context.qjs_ref, value_ref) };
        // let ref_c = unsafe { get_ref_count(in_context.qjs_ref, value_ref) };

        // Unlike JSC QuickJS starts with this value having a refcount of 1, so we don't need to up
        // it when we create a reference

        QJSValue {
            qjs_ref: value_ref,
            context_ref: in_context.clone(),
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

impl TryFrom<QJSValue> for String {
    type Error = JSConversionError;
    fn try_from(value: QJSValue) -> Result<Self, Self::Error> {
        let str: &str = value.try_into()?;
        Ok(str.to_string())
    }
}

impl TryFrom<QJSValue> for f64 {
    type Error = JSConversionError;
    fn try_from(value: QJSValue) -> Result<Self, Self::Error> {
        if value.qjs_ref.tag != JS_TAG_FLOAT64 as i64 {
            return Err(JSConversionError::ConversionFailed);
        }

        let mut precision = 0.0;
        let val = unsafe { JS_ToFloat64(value.context_ref.qjs_ref, &mut precision, value.qjs_ref) };
        Ok(val as f64 / precision)
    }
}

impl Drop for QJSValue {
    fn drop(&mut self) {
        unsafe { free_value(self.context_ref.qjs_ref, self.qjs_ref) }
    }
}
