use std::pin::Pin;

use javascriptcore_sys::{
    JSClassCreate, JSClassDefinition, JSGlobalContextCreateInGroup, JSGlobalContextRelease,
    OpaqueJSClass, OpaqueJSContext,
};

use crate::{
    shared::traits::jscontext::{JSContext, JSContextError},
    EsperantoResult,
};

use super::{
    jscore_ctx_global_data::JSCoreContextGlobalData, jscore_runtime::JSCoreRuntime,
    jscore_value::JSCoreValue,
};

pub struct JSCoreContext<'c> {
    pub(super) raw_ref: &'c mut OpaqueJSContext,
    runtime: &'c JSCoreRuntime<'c>,
}

pub trait JSCoreContextPrivate<'c> {
    fn new_in_context_group(
        runtime: &'c JSCoreRuntime<'c>,
        global_object: Option<&OpaqueJSClass>,
    ) -> EsperantoResult<Pin<Box<JSCoreContext<'c>>>> {
        // Need to turn this into the customisable global scope
        let def = JSClassDefinition::default();
        let class = unsafe { JSClassCreate(&def) };
        let raw_ref = unsafe { JSGlobalContextCreateInGroup(runtime.raw_ref, class) };

        match unsafe { raw_ref.as_mut() } {
            Some(r) => {
                let context = Box::pin(JSCoreContext {
                    raw_ref: r,
                    runtime,
                });

                JSCoreContextGlobalData::attach_to_context(&context);
                Ok(context)
            }
            None => Err(JSContextError::CouldNotCreateContext.into()),
        }
    }

    fn borrow_from_raw<'a>(
        raw_ref: *const OpaqueJSContext,
    ) -> EsperantoResult<&'a JSCoreContext<'a>> {
        let data = JSCoreContextGlobalData::get_from_raw(raw_ref)?;
        unsafe { data.ctx_ptr.as_ref() }
            .ok_or(JSContextError::CouldNotRetrieveFromNativePointer.into())
    }
}

impl<'c> JSCoreContextPrivate<'c> for JSCoreContext<'c> {}

impl Drop for JSCoreContext<'_> {
    fn drop(&mut self) {
        unsafe { JSGlobalContextRelease(self.raw_ref) }
    }
}

impl<'c> JSContext<'c> for JSCoreContext<'c> {
    type Runtime = JSCoreRuntime<'c>;
    type Value = JSCoreValue<'c>;
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

    use crate::{jscore::jscore_ctx_global_data::JSCoreContextGlobalData, traits::JSRuntime};

    use super::{JSCoreContext, JSCoreContextPrivate, JSCoreRuntime};

    #[test]
    fn creates_in_context_group() {
        let runtime = JSCoreRuntime::new().unwrap();
        JSCoreContext::new_in_context_group(&runtime, None).unwrap();
    }

    #[test]
    fn stores_context_ref_in_global_scope() {
        let runtime = JSCoreRuntime::new().unwrap();
        let context = JSCoreContext::new_in_context_group(&runtime, None).unwrap();
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
            JSCoreContext::borrow_from_raw(ctx).unwrap();
            CODE_RAN = true;
            JSValueMakeUndefined(ctx)
        }

        let runtime = JSCoreRuntime::new().unwrap();
        fn create<'a>(rt: &'a JSCoreRuntime<'a>) -> Box<JSCoreContext<'a>> {
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
                Box::new(JSCoreContext {
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
