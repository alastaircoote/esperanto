use std::convert::{TryFrom, TryInto};

use esperanto_engine_shared::{
    errors::{JSContextError, JSConversionError, JSValueError},
    traits::{HasContext, JSValue, TryIntoJSValue},
};
use javascriptcore_sys::{
    JSObjectSetProperty, JSValueIsObject, JSValueMakeBoolean, JSValueMakeNumber, JSValueProtect,
    JSValueToBoolean, JSValueToNumber, JSValueUnprotect, OpaqueJSValue,
};

use crate::{jscore_context::JSCoreContext, jscore_string::JSCoreString};

// #[derive(Debug, Copy, Clone)]
// enum RawRef {
//     Value(*const OpaqueJSValue),
//     Object(*mut OpaqueJSValue),
// }

// impl RawRef {
//     fn value(&self) -> *const OpaqueJSValue {
//         match self {
//             RawRef::Value(v) => *v,
//             RawRef::Object(o) => *o as *const OpaqueJSValue,
//         }
//     }
//     fn object(&self) -> Result<*mut OpaqueJSValue, JSValueError> {}
// }
#[derive(Debug)]
pub struct JSCoreValue {
    pub(crate) raw_ref: *const OpaqueJSValue,
    pub(crate) context: JSCoreContext,
}

impl JSCoreValue {
    pub fn from_raw(raw_ref: *const OpaqueJSValue, in_context: &JSCoreContext) -> Self {
        JSCoreValue {
            raw_ref,
            context: in_context.clone(),
        }
    }
}

impl Clone for JSCoreValue {
    fn clone(&self) -> Self {
        unsafe { JSValueProtect(self.context.raw_ref, self.raw_ref) };
        JSCoreValue {
            raw_ref: self.raw_ref,
            context: self.context.clone(),
        }
    }
}

impl Drop for JSCoreValue {
    fn drop(&mut self) {
        unsafe { JSValueUnprotect(self.context.raw_ref, self.raw_ref) };
    }
}

impl HasContext for JSCoreValue {
    type Context = JSCoreContext;
}

impl JSValue for JSCoreValue {
    type Context = JSCoreContext;

    fn set_property(&self, name: &str, value: &Self) -> Result<(), JSValueError> {
        if !unsafe { JSValueIsObject(self.context.raw_ref, self.raw_ref) } {
            return Err(JSValueError::IsNotAnObject);
        }

        let name_js_str = JSCoreString::try_from(name)?;

        let mut exception = std::ptr::null();
        unsafe {
            JSObjectSetProperty(
                self.context.raw_ref,
                self.raw_ref as *mut OpaqueJSValue,
                name_js_str.raw_ref,
                value.raw_ref,
                0,
                &mut exception,
            )
        }

        self.context.check_exception(&exception)?;
        Ok(())
    }
}

impl TryFrom<JSCoreValue> for f64 {
    type Error = JSContextError;

    fn try_from(value: JSCoreValue) -> Result<Self, Self::Error> {
        let mut exception = std::ptr::null();
        let num = unsafe { JSValueToNumber(value.context.raw_ref, value.raw_ref, &mut exception) };
        value.context.check_exception(&exception)?;
        if num.is_nan() {
            Err(JSConversionError::CouldNotConvertToNumber.into())
        } else {
            Ok(num)
        }
    }
}

impl TryFrom<JSCoreValue> for String {
    type Error = JSContextError;

    fn try_from(value: JSCoreValue) -> Result<Self, Self::Error> {
        let js_str = JSCoreString::try_from(value)?;
        Ok(js_str.try_into()?)
    }
}

impl TryFrom<JSCoreValue> for bool {
    type Error = JSContextError;

    fn try_from(value: JSCoreValue) -> Result<Self, Self::Error> {
        let bool_val = unsafe { JSValueToBoolean(value.context.raw_ref, value.raw_ref) };
        Ok(bool_val)
    }
}

impl TryIntoJSValue<JSCoreValue> for f64 {
    fn try_into_jsvalue(self, context: &JSCoreContext) -> Result<JSCoreValue, JSConversionError> {
        let raw_ref = unsafe { JSValueMakeNumber(context.raw_ref, self) };
        Ok(JSCoreValue {
            raw_ref,
            context: context.clone(),
        })
    }
}

impl TryIntoJSValue<JSCoreValue> for bool {
    fn try_into_jsvalue(self, context: &JSCoreContext) -> Result<JSCoreValue, JSConversionError> {
        let raw_ref = unsafe { JSValueMakeBoolean(context.raw_ref, self) };
        Ok(JSCoreValue {
            raw_ref,
            context: context.clone(),
        })
    }
}

impl TryIntoJSValue<JSCoreValue> for &str {
    fn try_into_jsvalue(self, context: &JSCoreContext) -> Result<JSCoreValue, JSConversionError> {
        let js_str = JSCoreString::try_from(self)?;
        js_str.try_into_jsvalue(&context)
    }
}

impl TryIntoJSValue<JSCoreValue> for String {
    fn try_into_jsvalue(self, context: &JSCoreContext) -> Result<JSCoreValue, JSConversionError> {
        let js_str = JSCoreString::try_from(self.as_str())?;
        js_str.try_into_jsvalue(&context)
    }
}

impl TryIntoJSValue<JSCoreValue> for *const OpaqueJSValue {
    fn try_into_jsvalue(self, context: &JSCoreContext) -> Result<JSCoreValue, JSConversionError> {
        Ok(JSCoreValue {
            raw_ref: self,
            context: context.clone(),
        })
    }
}

impl TryIntoJSValue<JSCoreValue> for *mut OpaqueJSValue {
    fn try_into_jsvalue(self, context: &JSCoreContext) -> Result<JSCoreValue, JSConversionError> {
        Ok(JSCoreValue {
            raw_ref: self,
            context: context.clone(),
        })
    }
}
