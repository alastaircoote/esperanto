use std::convert::{TryFrom, TryInto};

use javascriptcore_sys::{JSValueMakeString, JSValueProtect, OpaqueJSValue};

use crate::{
    errors::EsperantoError, shared::traits::tryas::TryIntoJS, traits::JSValue, EsperantoResult,
};

use super::{
    jscore_context::JSCoreContext, jscore_string::JSCoreString, jscore_value::JSCoreValue,
};

impl<'c> TryIntoJS<'c, JSCoreValue<'c>> for *const OpaqueJSValue {
    fn try_into_js(self, in_context: &'c JSCoreContext<'c>) -> EsperantoResult<JSCoreValue<'c>> {
        unsafe { JSValueProtect(in_context.raw_ref, self) };
        Ok(JSCoreValue {
            raw_ref: self.try_into()?,
            context: in_context,
        })
    }
}

impl<'c> TryIntoJS<'c, JSCoreValue<'c>> for *mut OpaqueJSValue {
    fn try_into_js(self, in_context: &'c JSCoreContext<'c>) -> EsperantoResult<JSCoreValue<'c>> {
        unsafe { JSValueProtect(in_context.raw_ref, self) };
        Ok(JSCoreValue {
            raw_ref: self.try_into()?,
            context: in_context,
        })
    }
}

impl<'c> TryIntoJS<'c, JSCoreValue<'c>> for JSCoreString {
    fn try_into_js(self, in_context: &'c JSCoreContext<'c>) -> EsperantoResult<JSCoreValue<'c>> {
        let raw = unsafe { JSValueMakeString(in_context.raw_ref, self.raw_ref) };
        Ok(JSCoreValue {
            raw_ref: raw.try_into()?,
            context: in_context,
        })
    }
}

impl<'c> TryIntoJS<'c, JSCoreValue<'c>> for String {
    fn try_into_js(self, in_context: &'c JSCoreContext<'c>) -> EsperantoResult<JSCoreValue<'c>> {
        let st = JSCoreString::try_from(&self)?;
        JSCoreValue::try_from_js(st, in_context)
    }
}

impl<'c> TryIntoJS<'c, JSCoreValue<'c>> for &str {
    fn try_into_js(self, in_context: &'c JSCoreContext<'c>) -> EsperantoResult<JSCoreValue<'c>> {
        let st = JSCoreString::try_from(self)?;
        JSCoreValue::try_from_js(st, in_context)
    }
}

// impl<'c, E> TryIntoJS<'c, JSCoreValue<'c>> for E
// where
//     E: std::error::Error,
// {
//     fn try_into_js(self, in_context: &'c Value::Context) -> EsperantoResult<JSCoreValue<'c>> {
//         todo!()
//     }
// }
