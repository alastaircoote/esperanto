use javascriptcore_sys::{
    JSContextGetGlobalObject, JSContextGetGroup, JSEvaluateScript, JSGarbageCollect,
    JSGlobalContextCreateInGroup, JSGlobalContextRelease, OpaqueJSValue,
};

use crate::shared::context::{EvaluateMetadata, JSContextError, JSContextInternal};
use crate::shared::value::JSValueInternal;

use super::{
    jscorecontextpointer::JSCoreContextPointer, jscoreruntime::JSCoreRuntimeInternal,
    jscorestring::JSCoreString, jscorevalue::JSCoreValueInternal,
};

use crate::shared::as_ptr::AsRawMutPtr;

pub(crate) type JSCoreContextInternal = JSCoreContextPointer;

impl JSContextInternal for JSCoreContextInternal {
    type RuntimeType = JSCoreRuntimeInternal;
    type ValueType = JSCoreValueInternal;
    fn new_in_runtime(runtime: Self::RuntimeType) -> Result<Self, JSContextError> {
        let ptr = unsafe { JSGlobalContextCreateInGroup(runtime, std::ptr::null_mut()) };

        if ptr.is_null() {
            return Err(JSContextError::CouldNotCreateContext);
        }

        Ok(ptr.into())
    }

    fn evaluate(
        self,
        script: std::ffi::CString,
        _: usize,
        metadata: Option<&EvaluateMetadata>,
    ) -> Result<Self::ValueType, crate::shared::errors::EsperantoError> {
        let mut script_jsstring = JSCoreString::from(&script);
        let mut filename_jsstring = metadata.map(|m| JSCoreString::from(&m.filename));

        let line_number = match metadata {
            Some(m) => m.line_number,
            None => 1,
        };

        let result: *const OpaqueJSValue = check_jscore_exception!(self, exception => {
            unsafe {
                JSEvaluateScript(
                    *self,
                    script_jsstring.as_mut_raw_ptr(),
                    std::ptr::null_mut(),
                    filename_jsstring.as_mut_raw_ptr(),
                    line_number,
                    exception,
                )
            }
        })?;

        // let hmm = JSCoreString::from_retained_ptr(script_jsstring.raw);
        // drop(hmm);

        let wrapped = JSCoreValueInternal::from(result);

        Ok(wrapped.retain(self.into()))
        // result.retain();
        // Ok(result)
    }

    // fn retain(&self) -> Self {
    //     unsafe { JSGlobalContextRetain(*self) }
    // }

    fn release(self) {
        if let JSCoreContextPointer::GlobalContext(g) = self {
            unsafe { JSGlobalContextRelease(g) }
        }
    }

    fn get_runtime(self) -> Self::RuntimeType {
        unsafe { JSContextGetGroup(*self) }
    }

    fn garbage_collect(self) {
        unsafe {
            JSGarbageCollect(*self);
            // JSSynchronousGarbageCollectForDebugging(*self);
        }
    }

    fn get_globalobject(self) -> Self::ValueType {
        unsafe { JSContextGetGlobalObject(*self) }.into()
    }
}

// #[link(name = "JavaScriptCore", kind = "framework")]
// extern "C" {
//     fn JSSynchronousGarbageCollectForDebugging(ctx: JSContextRef) -> ();
// }

#[cfg(test)]
mod test {
    use std::convert::TryInto;

    use javascriptcore_sys::{OpaqueJSContext, OpaqueJSValue};

    use crate::{JSContext, JSValue};

    #[link(name = "JavaScriptCore", kind = "framework")]
    extern "C" {
        fn JSGetMemoryUsageStatistics(ctx: *const OpaqueJSContext) -> *mut OpaqueJSValue;
    }

    // These are some sanity check tests to make sure we're retaining/releasing like we're
    // supposed to.

    fn get_protected_object_count(ctx: &JSContext) -> i32 {
        let hmm = unsafe { JSGetMemoryUsageStatistics(*ctx.internal) };
        let val = JSValue::wrap_internal(hmm.into(), &ctx);
        let num_js = val.get_property("protectedObjectCount").unwrap();
        return (&num_js).try_into().unwrap();
    }

    #[test]
    fn retains_and_releases_evals() {
        let ctx = JSContext::new().unwrap();
        let result = ctx.evaluate("({hello: 'there'})", None).unwrap();
        let retained = get_protected_object_count(&ctx);
        drop(result);
        assert_eq!(get_protected_object_count(&ctx) - retained, -1);
    }

    #[test]
    fn retains_and_releases_functions() {
        let ctx = JSContext::new().unwrap();
        let func = JSValue::new_function("return {}", vec![], &ctx).unwrap();
        let retained = get_protected_object_count(&ctx);
        drop(func);
        assert_eq!(get_protected_object_count(&ctx) - retained, -1);
    }
}
