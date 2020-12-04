use std::convert::{TryFrom, TryInto};

use crate::{
    jscore_runtime::JSCoreRuntime, jscore_string::JSCoreString, jscore_value::JSCoreValue,
};
use esperanto_engine_shared::errors::JSContextError;
use esperanto_engine_shared::traits::{JSContext, TryIntoJSValue};
use javascriptcore_sys::{
    JSContextGetGlobalObject, JSEvaluateScript, JSGlobalContextRelease, JSGlobalContextRetain,
    JSValueProtect, JSValueToStringCopy, OpaqueJSContext, OpaqueJSValue,
};

#[derive(Debug)]
pub struct JSCoreContext {
    pub(crate) raw_ref: *mut OpaqueJSContext,
    runtime: JSCoreRuntime,
}

impl JSCoreContext {
    pub(crate) fn check_exception(
        &self,
        exception: &*const OpaqueJSValue,
    ) -> Result<(), JSContextError> {
        if exception.is_null() {
            return Ok(());
        }
        let raw_ref =
            unsafe { JSValueToStringCopy(self.raw_ref, *exception, std::ptr::null_mut()) };
        let js_string: JSCoreString = raw_ref.into();
        Err(JSContextError::JSErrorOccurred(js_string.try_into()?))
    }
}

impl Clone for JSCoreContext {
    fn clone(&self) -> Self {
        JSCoreContext {
            raw_ref: unsafe { JSGlobalContextRetain(self.raw_ref) },
            runtime: self.runtime.clone(),
        }
    }
}

impl Drop for JSCoreContext {
    fn drop(&mut self) {
        unsafe { JSGlobalContextRelease(self.raw_ref) }
    }
}

impl JSContext for JSCoreContext {
    type Runtime = JSCoreRuntime;
    type Value = JSCoreValue;

    fn evaluate<'a>(
        &self,
        script: &'a str,
        source: Option<esperanto_engine_shared::metadata::JSScriptSource>,
    ) -> Result<Self::Value, JSContextError> {
        let script_string = JSCoreString::try_from(script)?;

        // Slight oddity here: we need to store the url variable outside
        // if the if statement below, because if we don't the variable will be
        // immediately dropped and the JSString will be released before it gets
        // used. By keeping it outside we ensure the variable will still be
        // valid when we run the eval call.

        let url: JSCoreString;
        let mut url_ptr = std::ptr::null_mut();
        let mut line_number = 0;

        if let Some(source) = source {
            url = JSCoreString::try_from(source.script_url)?;
            url_ptr = url.raw_ref;
            line_number = source.line_number;
        }
        let mut exception = std::ptr::null();
        let result = unsafe {
            JSEvaluateScript(
                self.raw_ref,
                script_string.raw_ref,
                std::ptr::null_mut(),
                url_ptr,
                line_number,
                &mut exception,
            )
        };
        self.check_exception(&exception)?;
        Ok(JSCoreValue::from_raw(result, &self))
    }

    fn new_in_runtime(runtime: &Self::Runtime) -> Result<Self, JSContextError> {
        let raw_ref = runtime.new_raw_context();
        Ok(JSCoreContext {
            raw_ref,
            runtime: runtime.clone(),
        })
    }

    fn global_object(&self) -> Result<Self::Value, JSContextError> {
        let obj_ref = unsafe { JSContextGetGlobalObject(self.raw_ref) };
        unsafe { JSValueProtect(self.raw_ref, obj_ref) };
        Ok(obj_ref.try_into_jsvalue(&self)?)
    }
}
