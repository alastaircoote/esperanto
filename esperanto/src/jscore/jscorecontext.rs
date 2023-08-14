use javascriptcore_sys::{
    JSClassCreate, JSClassDefinition, JSClassRelease, JSContextGetGlobalObject, JSEvaluateScript,
    JSGarbageCollect, JSGlobalContextCreateInGroup, JSGlobalContextRelease, OpaqueJSContext,
    OpaqueJSValue,
};

use crate::shared::context::{EvaluateMetadata, JSContextError, JSContextImplementation};
use crate::shared::value::JSValueImplementation;
use crate::EsperantoResult;

use super::{
    jscoreruntime::JSCoreRuntimeInternal, jscorestring::JSCoreString,
    jscorevalue::JSCoreValueInternal,
};

use crate::shared::as_ptr::AsRawMutPtr;

pub(crate) type JSCoreContextInternal = *mut OpaqueJSContext;

impl JSContextImplementation for JSCoreContextInternal {
    type RuntimeType = JSCoreRuntimeInternal;
    type ValueType = JSCoreValueInternal;
    fn new_in_runtime(runtime: &Self::RuntimeType) -> Result<Self, JSContextError> {
        // Eventually we'll want to provide custom global objects. But for now let's just use a default
        // but not *the* default because we can't store private data against that.

        let global_object_class_def = JSClassDefinition::default();
        let global_object_class = unsafe { JSClassCreate(&global_object_class_def) };

        let ptr = unsafe { JSGlobalContextCreateInGroup(runtime.raw, global_object_class) };

        unsafe { JSClassRelease(global_object_class) };

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
                    self,
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
        println!("RELEASE CONTEXT??");
        unsafe { JSGlobalContextRelease(self) }
    }

    // fn get_runtime(self) -> Self::RuntimeType {
    //     let storage =
    //         unsafe { JSObjectGetPrivate(self.get_globalobject().try_as_object(self).unwrap()) };

    //     JSCoreRuntimeInternal {
    //         raw: unsafe { JSContextGetGroup(self) },
    //         class_storage: JSCoreRuntimeStorage::Referenced(storage as _),
    //     }
    // }

    fn garbage_collect(self) {
        unsafe {
            // JSSynchronousGarbageCollectForDebugging(self);
            // JSSynchronousEdenCollectForDebugging(self);
            // JSReportExtraMemoryCost(self, 9999999999);
            JSGarbageCollect(self);
        }
    }

    fn get_globalobject(self) -> Self::ValueType {
        unsafe { JSContextGetGlobalObject(self) }.into()
    }

    fn get_private_data(self) -> EsperantoResult<*mut std::ffi::c_void> {
        // JSC doesn't have context-private data but it does have storage in the global object.
        // might need to think about what to do if we actually want to store something else there
        // some day.

        let global: Self::ValueType = unsafe { JSContextGetGlobalObject(self) }.into();
        global.get_private_data(self)
    }

    fn set_private_data(self, data: *mut std::ffi::c_void) -> EsperantoResult<()> {
        let global: Self::ValueType = unsafe { JSContextGetGlobalObject(self) }.into();
        global.set_private_data(self, data)
    }
}

// #[link(name = "JavaScriptCore", kind = "framework")]
// extern "C" {
//     fn JSSynchronousGarbageCollectForDebugging(ctx: JSContextRef) -> ();
//     fn JSSynchronousEdenCollectForDebugging(ctx: JSContextRef) -> ();
//     fn JSReportExtraMemoryCost(ctx: JSContextRef, size: usize) -> ();
// }

#[cfg(test)]
mod test {

    use javascriptcore_sys::{OpaqueJSContext, OpaqueJSValue};

    use crate::{JSContext, JSExportClass, JSValue};

    // use super::JSSynchronousGarbageCollectForDebugging;

    #[link(name = "JavaScriptCore", kind = "framework")]
    extern "C" {
        fn JSGetMemoryUsageStatistics(ctx: *const OpaqueJSContext) -> *mut OpaqueJSValue;
    }

    // These are some sanity check tests to make sure we're retaining/releasing like we're
    // supposed to.

    // #[test]
    // fn can_manually_garbage_collect() {
    //     let group = unsafe { JSContextGroupCreate() };

    //     let global_object_class_def = JSClassDefinition::default();
    //     let global_object_class = unsafe { JSClassCreate(&global_object_class_def) };
    //     // unsafe { JSClassRelease(global_object_class) };

    //     // let ctx = unsafe { JSGlobalContextCreate(std::ptr::null_mut()) };
    //     let ctx = unsafe { JSGlobalContextCreateInGroup(group, global_object_class) };

    //     let obj = unsafe { JSObjectMake(ctx, std::ptr::null_mut(), std::ptr::null_mut()) };
    //     unsafe { JSValueUnprotect(ctx, obj) };

    //     unsafe { JSSynchronousGarbageCollectForDebugging(ctx) };
    //     unsafe { JSObjectGetPrototype(ctx, obj) };

    //     unsafe { JSGlobalContextRelease(ctx) };
    //     unsafe { JSContextGroupRelease(group) };
    // }

    fn get_protected_object_count(ctx: &JSContext) -> i32 {
        let hmm = unsafe { JSGetMemoryUsageStatistics(ctx.implementation()) };
        let val = JSValue::wrap_internal(hmm.into(), &ctx);
        let json = ctx
            .global_object()
            .get_property("JSON")
            .unwrap()
            .get_property("stringify")
            .unwrap()
            .call_as_function(vec![
                &val,
                &JSValue::undefined(&ctx),
                &JSValue::try_new_from(2.0, &ctx).unwrap(),
            ])
            .unwrap();

        println!("{}", json.to_string());
        let num_js = val.get_property("protectedObjectCount").unwrap();
        return num_js.try_convert().unwrap();
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

    #[test]
    fn retains_and_releases_native_classes() {
        struct TestStruct {}

        impl JSExportClass for TestStruct {
            const CLASS_NAME: &'static str = "TestStruct";
        }

        let ctx = JSContext::new().unwrap();
        let start = get_protected_object_count(&ctx);
        let object = JSValue::new_wrapped_native(TestStruct {}, &ctx).unwrap();
        drop(object);
        assert_eq!(get_protected_object_count(&ctx), start);
    }
}
