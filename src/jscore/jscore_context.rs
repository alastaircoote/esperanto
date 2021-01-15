use std::{
    convert::{TryFrom, TryInto},
    pin::Pin,
};

use javascriptcore_sys::{
    JSClassCreate, JSClassDefinition, JSContextGetGlobalObject, JSEvaluateScript,
    JSGlobalContextCreateInGroup, JSGlobalContextRelease, JSObjectGetPrivate, JSObjectSetPrivate,
    JSValueIsObject, OpaqueJSClass, OpaqueJSContext, OpaqueJSValue,
};

use crate::{
    jsruntime::Runtime,
    shared::{
        external_api::context::{Context, EvaluateMetadata, JSContextError},
        traits::tryas::TryIntoJS,
    },
    EsperantoError, EsperantoResult,
};

use super::{
    jscore_context_empty_global::EmptyGlobalScope,
    jscore_context_runtime_store::JSCoreContextRuntimeStore,
    jscore_export::JSCoreExport,
    jscore_runtime::{JSCoreRuntime, JSRuntime},
    jscore_string::JSCoreString,
    jscore_value::{JSCoreValue, JSValue},
};

pub struct JSContext<'r, 'c> {
    pub(super) raw_ref: &'c mut OpaqueJSContext,
    pub(super) runtime: JSCoreContextRuntimeStore<'r>,
}

pub type JSCoreContext<'r, 'c> = JSContext<'r, 'c>;

impl<'r, 'c, 'v> Context<'r, 'c, 'v> for JSCoreContext<'r, 'c>
where
    'r: 'c,
{
    type Runtime = JSCoreRuntime<'r>;
    type Value = JSCoreValue<'r, 'c, 'v>;
    type SelfInstanceType = Pin<Box<Self>>;

    fn evaluate(
        &'c self,
        script: &str,
        meta: Option<EvaluateMetadata>,
    ) -> EsperantoResult<Self::Value> {
        let script_jscore_string: JSCoreString = script.try_into()?;

        let source_url = match &meta {
            Some(meta) => Some(JSCoreString::try_from(meta.file_name)?),
            None => None,
        };
        let result = check_jscore_exception!(&self, exception =>
            unsafe { JSEvaluateScript(
                self.raw_ref,
                script_jscore_string.raw_ref,
                std::ptr::null_mut(),
                source_url.map(|s| s.raw_ref).unwrap_or(std::ptr::null_mut()),
                meta.map(|m| m.line_number).unwrap_or(0),
                exception,
            )
        }
        )?;
        if unsafe { JSValueIsObject(self.raw_ref, result) } {
            (result as *mut OpaqueJSValue).try_into_js(self)
        } else {
            result.try_into_js(&self)
        }
    }

    fn new(runtime: Option<&'r Self::Runtime>) -> EsperantoResult<Self::SelfInstanceType> {
        Self::new_with_global(runtime, EmptyGlobalScope {})
    }

    fn new_with_global<G: crate::JSExport>(
        runtime: Option<&'r Self::Runtime>,
        global_object: G,
    ) -> EsperantoResult<Self::SelfInstanceType> {
        let rt_ref = match runtime {
            Some(rt) => JSCoreContextRuntimeStore::External(rt),
            None => {
                let new_runtime = JSCoreRuntime::new()?;
                let boxed = Box::new(new_runtime);
                let boxed_ref = Box::into_raw(boxed);
                JSCoreContextRuntimeStore::SelfContained(boxed_ref)
            }
        };

        let raw_ref = {
            let class_def = rt_ref.get_class_def::<G>()?;
            unsafe { JSGlobalContextCreateInGroup(rt_ref.raw_ref, class_def) }
        };

        match unsafe { raw_ref.as_mut() } {
            Some(r) => {
                let context = Box::pin(JSContext {
                    raw_ref: r,
                    runtime: rt_ref,
                });

                let js_global_obj = unsafe { JSContextGetGlobalObject(context.raw_ref) };
                G::store_private(global_object, js_global_obj, &context)?;
                Ok(context)
            }
            None => Err(JSContextError::CouldNotCreateContext.into()),
        }
    }
}

impl Drop for JSCoreContext<'_, '_> {
    fn drop(&mut self) {
        unsafe { JSGlobalContextRelease(self.raw_ref) }
    }
}

impl From<&JSCoreContext<'_, '_>> for *const OpaqueJSContext {
    fn from(ctx: &JSCoreContext) -> Self {
        ctx.raw_ref as *const OpaqueJSContext
    }
}

#[cfg(test)]
mod test {
    use std::ffi::CString;

    use javascriptcore_sys::{
        JSClassCreate, JSClassDefinition, JSContextGetGlobalObject, JSContextGroupCreate,
        JSEvaluateScript, JSGlobalContextCreate, JSGlobalContextCreateInGroup, JSStaticFunction,
        JSStringCreateWithUTF8CString, JSValueMakeUndefined, OpaqueJSContext, OpaqueJSValue,
    };

    use crate::{jscontext::Context, jscore::jscore_export::JSCoreExport, jsruntime::Runtime};

    use super::{JSContext, JSRuntime};

    #[test]
    fn creates_in_context_group() {
        let runtime = JSRuntime::new().unwrap();
        JSContext::new(Some(&runtime)).unwrap();
    }

    #[test]
    fn creates_isolated() {
        JSContext::new(None).unwrap();
    }

    // #[test]
    // fn fetches_context_in_c_callback() {
    //     static mut CODE_RAN: bool = false;

    //     unsafe extern "C" fn test_callback(
    //         ctx: *const OpaqueJSContext,
    //         _: *mut OpaqueJSValue,
    //         _: *mut OpaqueJSValue,
    //         _: usize,
    //         _: *const *const OpaqueJSValue,
    //         _: *mut *const OpaqueJSValue,
    //     ) -> *const OpaqueJSValue {
    //         // This is where the test would actually fail.
    //         DummyJSExport::get_context_from_raw(ctx).unwrap();
    //         CODE_RAN = true;
    //         JSValueMakeUndefined(ctx)
    //     }

    //     let runtime = JSRuntime::new().unwrap();
    //     fn create<'a>(rt: &'a JSRuntime<'a>) -> Box<JSContext<'a>> {
    //         let name = CString::new("test").unwrap();
    //         let static_funcs = vec![
    //             JSStaticFunction {
    //                 name: name.as_ptr(),
    //                 callAsFunction: Some(test_callback),
    //                 attributes: 0,
    //             },
    //             JSStaticFunction {
    //                 name: std::ptr::null_mut(),
    //                 callAsFunction: None,
    //                 attributes: 0,
    //             },
    //         ];
    //         let mut jsd = JSClassDefinition::default();
    //         jsd.staticFunctions = static_funcs.as_ptr();
    //         let class = unsafe { JSClassCreate(&jsd) };
    //         let ctx_raw = unsafe { JSGlobalContextCreateInGroup(rt.raw_ref, class) };
    //         unsafe {
    //             Box::new(JSContext {
    //                 raw_ref: ctx_raw.as_mut().unwrap(),
    //                 runtime: &rt,
    //             })
    //         }
    //     };

    //     let manual_context = create(&runtime);

    //     JSCoreContextGlobalData::attach_to_context(&manual_context);

    //     let script = CString::new("test()").unwrap();
    //     let js_script = unsafe { JSStringCreateWithUTF8CString(script.as_ptr()) };
    //     unsafe {
    //         JSEvaluateScript(
    //             manual_context.raw_ref,
    //             js_script,
    //             std::ptr::null_mut(),
    //             std::ptr::null_mut(),
    //             0,
    //             std::ptr::null_mut(),
    //         );
    //         assert_eq!(CODE_RAN, true);
    //     };
    // }
}
