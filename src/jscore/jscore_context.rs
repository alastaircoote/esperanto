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
    shared::{
        external_api::context::{Context, EvaluateMetadata, JSContextError},
        traits::tryas::TryIntoJS,
    },
    EsperantoError, EsperantoResult,
};

use super::{
    jscore_ctx_global_data::JSCoreContextGlobalData,
    jscore_export::JSExport,
    jscore_runtime::{JSCoreRuntime, JSRuntime},
    jscore_string::JSCoreString,
    jscore_value::{JSCoreValue, JSValue},
};

pub struct JSContext<'c> {
    pub(super) raw_ref: &'c mut OpaqueJSContext,
    pub(super) runtime: &'c JSRuntime<'c>,
}

pub type JSCoreContext<'c> = JSContext<'c>;

impl<'c> Context<'c> for JSCoreContext<'c> {
    type Runtime = JSCoreRuntime<'c>;
    type Value = JSCoreValue<'c>;
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
        let result = check_jscore_exception!(&self, exception => {
            unsafe {JSEvaluateScript(
                self.raw_ref,
                script_jscore_string.raw_ref,
                std::ptr::null_mut(),
                source_url.map(|s| s.raw_ref).unwrap_or(std::ptr::null_mut()),
                meta.map(|m| m.line_number).unwrap_or(0),
                exception,
            )
        }
        })?;
        if unsafe { JSValueIsObject(self.raw_ref, result) } {
            (result as *mut OpaqueJSValue).try_into_js(&self)
        } else {
            result.try_into_js(&self)
        }
    }
}

struct DummyJSExport {}

impl JSExport for DummyJSExport {
    fn get_definition<'a>() -> &'a JSClassDefinition {
        const default: JSClassDefinition = JSClassDefinition {
            version: 1,
            className: "EmptyGlobalScope\0".as_ptr() as *const std::os::raw::c_char,
            attributes: 0,
            staticFunctions: std::ptr::null_mut(),
            callAsConstructor: None,
            callAsFunction: None,
            convertToType: None,
            deleteProperty: None,
            hasProperty: None,
            setProperty: None,
            finalize: None,
            getProperty: None,
            getPropertyNames: None,
            hasInstance: None,
            initialize: None,
            parentClass: std::ptr::null_mut(),
            staticValues: std::ptr::null_mut(),
        };
        &default
    }
}

impl<'c> JSCoreContext<'c> {
    pub(super) fn new_with_global_object<G: JSExport>(
        runtime: &'c JSRuntime<'c>,
        global_object: G,
    ) -> EsperantoResult<Pin<Box<JSContext<'c>>>> {
        let class_def = runtime.get_class_def::<G>()?;

        let raw_ref = unsafe { JSGlobalContextCreateInGroup(runtime.raw_ref, class_def) };

        match unsafe { raw_ref.as_mut() } {
            Some(r) => {
                let context = Box::pin(JSContext {
                    raw_ref: r,
                    runtime,
                });

                let js_global_obj = unsafe { JSContextGetGlobalObject(context.raw_ref) };
                G::store_private(global_object, js_global_obj, &context)?;
                Ok(context)
            }
            None => Err(JSContextError::CouldNotCreateContext.into()),
        }
    }

    pub(super) fn new(runtime: &'c JSRuntime<'c>) -> EsperantoResult<Pin<Box<JSContext<'c>>>> {
        let dummy = DummyJSExport {};
        Self::new_with_global_object(runtime, dummy)
    }
}

pub trait JSCoreContextPrivate<'c> {
    // fn new_in_context_group<G: JSExport>(
    //     runtime: &'c JSRuntime<'c>,
    //     global_object: Option<G>,
    // ) -> EsperantoResult<Pin<Box<JSContext<'c>>>> {
    //     let class = match global_object {
    //         Some(c) => c as *mut OpaqueJSClass,
    //         None => {
    //             let def = JSClassDefinition::default();
    //             unsafe { JSClassCreate(&def) }
    //         }
    //     };

    //     let raw_ref = unsafe { JSGlobalContextCreateInGroup(runtime.raw_ref, class) };

    //     match unsafe { raw_ref.as_mut() } {
    //         Some(r) => {
    //             let context = Box::pin(JSContext {
    //                 raw_ref: r,
    //                 runtime,
    //             });

    //             JSCoreContextGlobalData::attach_to_context(&context);
    //             Ok(context)
    //         }
    //         None => Err(JSContextError::CouldNotCreateContext.into()),
    //     }
    // }

    // fn borrow_from_raw<'a>(raw_ref: *const OpaqueJSContext) -> EsperantoResult<&'a JSContext<'a>> {
    //     let data = JSCoreContextGlobalData::get_from_raw(raw_ref)?;
    //     unsafe { data.ctx_ptr.as_ref() }
    //         .ok_or(JSContextError::CouldNotRetrieveFromNativePointer.into())
    // }
}

impl<'c> JSCoreContextPrivate<'c> for JSCoreContext<'c> {}

impl Drop for JSCoreContext<'_> {
    fn drop(&mut self) {
        unsafe { JSGlobalContextRelease(self.raw_ref) }
    }
}

impl<'a> From<&'a JSCoreContext<'a>> for *const OpaqueJSContext {
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

    use crate::{
        jscore::{jscore_ctx_global_data::JSCoreContextGlobalData, jscore_export::JSExport},
        jsruntime::Runtime,
    };

    use super::{DummyJSExport, JSContext, JSCoreContextPrivate, JSRuntime};

    #[test]
    fn creates_in_context_group() {
        let runtime = JSRuntime::new().unwrap();
        JSContext::new(&runtime).unwrap();
    }

    #[test]
    fn stores_context_ref_in_global_scope() {
        let runtime = JSRuntime::new().unwrap();
        let context = JSContext::new(&runtime).unwrap();
        JSCoreContextGlobalData::get_from_raw(context.raw_ref).unwrap();
    }

    #[test]
    fn fetches_context_in_c_callback() {
        static mut CODE_RAN: bool = false;

        unsafe extern "C" fn test_callback(
            ctx: *const OpaqueJSContext,
            _: *mut OpaqueJSValue,
            _: *mut OpaqueJSValue,
            _: usize,
            _: *const *const OpaqueJSValue,
            _: *mut *const OpaqueJSValue,
        ) -> *const OpaqueJSValue {
            // This is where the test would actually fail.
            DummyJSExport::get_context_from_raw(ctx).unwrap();
            CODE_RAN = true;
            JSValueMakeUndefined(ctx)
        }

        let runtime = JSRuntime::new().unwrap();
        fn create<'a>(rt: &'a JSRuntime<'a>) -> Box<JSContext<'a>> {
            let name = CString::new("test").unwrap();
            let static_funcs = vec![
                JSStaticFunction {
                    name: name.as_ptr(),
                    callAsFunction: Some(test_callback),
                    attributes: 0,
                },
                JSStaticFunction {
                    name: std::ptr::null_mut(),
                    callAsFunction: None,
                    attributes: 0,
                },
            ];
            let mut jsd = JSClassDefinition::default();
            jsd.staticFunctions = static_funcs.as_ptr();
            let class = unsafe { JSClassCreate(&jsd) };
            let ctx_raw = unsafe { JSGlobalContextCreateInGroup(rt.raw_ref, class) };
            unsafe {
                Box::new(JSContext {
                    raw_ref: ctx_raw.as_mut().unwrap(),
                    runtime: &rt,
                })
            }
        };

        let manual_context = create(&runtime);

        JSCoreContextGlobalData::attach_to_context(&manual_context);

        let script = CString::new("test()").unwrap();
        let js_script = unsafe { JSStringCreateWithUTF8CString(script.as_ptr()) };
        unsafe {
            JSEvaluateScript(
                manual_context.raw_ref,
                js_script,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                0,
                std::ptr::null_mut(),
            );
            assert_eq!(CODE_RAN, true);
        };
    }
}
