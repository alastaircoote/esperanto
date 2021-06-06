use std::{ffi::CStr, ops::Deref};

use quickjs_android_suitable_sys::{JSContext as QuickJSContext, JS_FreeCString};

pub(crate) struct QuickJSString<'a> {
    underlying_cstr: &'a CStr,
    context: *mut QuickJSContext,
}

impl Deref for QuickJSString<'_> {
    type Target = CStr;

    fn deref(&self) -> &Self::Target {
        self.underlying_cstr
    }
}

impl Drop for QuickJSString<'_> {
    fn drop(&mut self) {
        unsafe { JS_FreeCString(self.context, self.underlying_cstr.as_ptr()) }
    }
}
