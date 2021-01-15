use std::convert::{TryFrom, TryInto};

use javascriptcore_sys::{JSValueMakeString, JSValueMakeUndefined, JSValueProtect, OpaqueJSValue};

use crate::{jsvalue::Value, shared::traits::tryas::TryIntoJS, EsperantoResult};

use super::{jscore_context::JSCoreContext, jscore_string::JSCoreString, jscore_value::JSValue};

impl<'c> TryIntoJS<'c> for *const OpaqueJSValue {
    fn try_into_js(self, in_context: &'c JSCoreContext<'c>) -> EsperantoResult<JSValue<'c>> {
        unsafe { JSValueProtect(in_context.raw_ref, self) };
        Ok(JSValue {
            raw_ref: self.try_into()?,
            context: in_context,
        })
    }
}

impl<'c> TryIntoJS<'c> for *mut OpaqueJSValue {
    fn try_into_js(self, in_context: &'c JSCoreContext<'c>) -> EsperantoResult<JSValue<'c>> {
        unsafe { JSValueProtect(in_context.raw_ref, self) };
        Ok(JSValue {
            raw_ref: self.try_into()?,
            context: in_context,
        })
    }
}

impl<'c> TryIntoJS<'c> for JSCoreString {
    fn try_into_js(self, in_context: &'c JSCoreContext<'c>) -> EsperantoResult<JSValue<'c>> {
        let raw = unsafe { JSValueMakeString(in_context.raw_ref, self.raw_ref) };
        Ok(JSValue {
            raw_ref: raw.try_into()?,
            context: in_context,
        })
    }
}

impl<'c> TryIntoJS<'c> for String {
    fn try_into_js(self, in_context: &'c JSCoreContext<'c>) -> EsperantoResult<JSValue<'c>> {
        let st = JSCoreString::try_from(&self)?;
        st.try_into_js(in_context)
    }
}

impl<'c> TryIntoJS<'c> for &str {
    fn try_into_js(self, in_context: &'c JSCoreContext<'c>) -> EsperantoResult<JSValue<'c>> {
        let st = JSCoreString::try_from(self)?;
        st.try_into_js(in_context)
    }
}

impl<'c> TryIntoJS<'c> for () {
    fn try_into_js(self, in_context: &'c JSCoreContext<'c>) -> EsperantoResult<JSValue<'c>> {
        JSValue::undefined(in_context)
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
