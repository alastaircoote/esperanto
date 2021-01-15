use std::convert::{TryFrom, TryInto};

use javascriptcore_sys::{JSValueMakeString, JSValueMakeUndefined, JSValueProtect, OpaqueJSValue};

use crate::{jsvalue::Value, shared::traits::tryas::TryIntoJS, EsperantoResult};

use super::{jscore_context::JSCoreContext, jscore_string::JSCoreString, jscore_value::JSValue};

impl<'r, 'c, 'v> TryIntoJS<'r, 'c, 'v> for *const OpaqueJSValue {
    fn try_into_js(
        self,
        in_context: &'c JSCoreContext<'r, 'c>,
    ) -> EsperantoResult<JSValue<'r, 'c, 'v>> {
        unsafe { JSValueProtect(in_context.raw_ref, self) };
        Ok(JSValue {
            raw_ref: self.try_into()?,
            context: in_context,
        })
    }
}

impl<'r, 'c, 'v> TryIntoJS<'r, 'c, 'v> for *mut OpaqueJSValue {
    fn try_into_js(
        self,
        in_context: &'c JSCoreContext<'r, 'c>,
    ) -> EsperantoResult<JSValue<'r, 'c, 'v>> {
        unsafe { JSValueProtect(in_context.raw_ref, self) };
        Ok(JSValue {
            raw_ref: self.try_into()?,
            context: in_context,
        })
    }
}

impl<'r, 'c, 'v> TryIntoJS<'r, 'c, 'v> for JSCoreString {
    fn try_into_js(
        self,
        in_context: &'c JSCoreContext<'r, 'c>,
    ) -> EsperantoResult<JSValue<'r, 'c, 'v>> {
        let raw = unsafe { JSValueMakeString(in_context.raw_ref, self.raw_ref) };
        Ok(JSValue {
            raw_ref: raw.try_into()?,
            context: in_context,
        })
    }
}

impl<'r, 'c, 'v> TryIntoJS<'r, 'c, 'v> for String {
    fn try_into_js(
        self,
        in_context: &'c JSCoreContext<'r, 'c>,
    ) -> EsperantoResult<JSValue<'r, 'c, 'v>> {
        let st = JSCoreString::try_from(&self)?;
        st.try_into_js(in_context)
    }
}

impl<'r, 'c, 'v> TryIntoJS<'r, 'c, 'v> for &str {
    fn try_into_js(
        self,
        in_context: &'c JSCoreContext<'r, 'c>,
    ) -> EsperantoResult<JSValue<'r, 'c, 'v>> {
        let st = JSCoreString::try_from(self)?;
        st.try_into_js(in_context)
    }
}

impl<'r, 'c, 'v> TryIntoJS<'r, 'c, 'v> for () {
    fn try_into_js(
        self,
        in_context: &'c JSCoreContext<'r, 'c>,
    ) -> EsperantoResult<JSValue<'r, 'c, 'v>> {
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
