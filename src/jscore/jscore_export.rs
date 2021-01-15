use javascriptcore_sys::{
    JSClassDefinition, JSContextGetGlobalObject, JSObjectCallAsFunctionCallback,
    JSObjectGetPrivate, JSObjectMakeError, JSObjectSetPrivate, JSStaticFunction,
    JSStringCreateWithUTF8CString, JSStringRelease, JSValueMakeString, JSValueMakeUndefined,
    JSValueUnprotect, OpaqueJSContext, OpaqueJSValue,
};
use thiserror::Error;

use crate::{
    shared::external_api::{context::JSContextError, esperanto_error::EngineError},
    EsperantoError, EsperantoResult,
};

use super::jscore_context::JSCoreContext;

#[derive(Debug, Error)]
pub enum JSExportError {
    #[error("You did not supply enough arguments")]
    NotEnoughArguments,
    #[error(transparent)]
    NativeSideErrorOccurred(Box<dyn std::error::Error>),
    #[error("Could not retrieve data from private storage")]
    CouldNotRetrievePrivateData,
    #[error("Could not save private data in this object")]
    CouldNotSavePrivateData,
}

impl EngineError for JSExportError {}

pub trait JSCoreExport: Sized + 'static {
    fn get_definition<'a>() -> &'a JSClassDefinition;
    fn store_private<T>(
        obj: T,
        in_object: *mut OpaqueJSValue,
        with_context: &JSCoreContext,
    ) -> EsperantoResult<()> {
        let boxed_up = Box::new(obj);
        let raw_ptr = Box::into_raw(boxed_up) as *mut std::ffi::c_void;
        let ctx_raw = with_context as *const JSCoreContext as *mut std::ffi::c_void;
        let array = Box::into_raw(Box::new([raw_ptr, ctx_raw]));
        let saved = unsafe { JSObjectSetPrivate(in_object, array as *mut std::ffi::c_void) };
        if saved {
            Ok(())
        } else {
            Err(JSExportError::CouldNotSavePrivateData.into())
        }
    }

    fn clear_private(in_object: *mut OpaqueJSValue) {
        let private = unsafe { JSObjectGetPrivate(in_object) } as *mut [*mut std::ffi::c_void; 2];
        if let Some(r) = unsafe { private.as_ref() } {
            let obj_ref = r[0] as *mut Self;
            unsafe { Box::from_raw(obj_ref) };
        } else {
            // this function is used in finalizers where we have no ability to
            // return an error, so open question here about what we'd do. It shouldn't
            // ever happen, but..?!??
        }
    }

    fn get_context_from_object<'c>(
        object: *mut OpaqueJSValue,
    ) -> EsperantoResult<&'c JSCoreContext<'c>> {
        let private = unsafe { JSObjectGetPrivate(object) } as *mut [*mut std::ffi::c_void; 2];
        let ctx_raw = match unsafe { private.as_ref() } {
            Some(r) => r[1],
            None => return Err(JSExportError::CouldNotSavePrivateData.into()),
        };
        unsafe { (ctx_raw as *const JSCoreContext).as_ref() }
            .ok_or(JSExportError::CouldNotRetrievePrivateData.into())
    }

    fn get_context_from_raw<'c>(
        ctx: *const OpaqueJSContext,
    ) -> EsperantoResult<&'c JSCoreContext<'c>> {
        let global_obj = unsafe { JSContextGetGlobalObject(ctx) };
        Self::get_context_from_object(global_obj)
    }

    unsafe fn manually_map_error(
        err: EsperantoError,
        ctx: *const OpaqueJSContext,
    ) -> (*const OpaqueJSValue, *const OpaqueJSValue) {
        let err_str = err.to_string();
        // since this could have even failed at the context-fetching stage
        // we'll do this part manually rather than use our Rust API
        let c_string = std::ffi::CString::new(err_str);

        // again, we're all out of places to put failures at this point, so if for whatever
        // reason we're not able to encode the error as a UTF8 string, we need to
        // use a fallback.
        const FALLBACK: &[u8] = b"An error occurred but could not get the details\0";

        let js_str = match c_string {
            Ok(s) => JSStringCreateWithUTF8CString(s.as_ptr()),
            Err(_) => JSStringCreateWithUTF8CString(FALLBACK.as_ptr() as *const i8),
        };

        let js_val = JSValueMakeString(ctx, js_str);
        let js_vals = [js_val, js_val];
        JSStringRelease(js_str);

        let mut create_exception: *const OpaqueJSValue = std::ptr::null_mut();
        let err = JSObjectMakeError(ctx, 2, js_vals.as_ptr(), &mut create_exception);
        JSValueUnprotect(ctx, js_val);

        let undef = JSValueMakeUndefined(ctx);
        JSValueUnprotect(ctx, undef);

        if create_exception.is_null() == false {
            // The error creation process threw an exception, so we'll have to just
            // pass that back into the JS environment without knowing what happened
            return (create_exception, undef);
        } else {
            return (err, undef);
        }
    }
}

#[macro_export]
macro_rules! js_export_definition {
    // Root call:
    ($js_class_name: expr => $struct_ident: ident, {
        static_functions: {
            $($js_func_name: expr => $export_name:ident |$args_ident:ident| $body:expr),*
        }
    }) => {
        impl esperanto::JSExport for $struct_ident {

            fn get_definition<'a>() -> &'a esperanto::private::javascriptcore_sys::JSClassDefinition {
                use esperanto::private::javascriptcore_sys::{OpaqueJSValue, OpaqueJSContext,JSStaticFunction, JSStaticValue, JSClassDefinition};
                use esperanto::private::jscore_context::{JSContext};
                use esperanto::private::jscore_value::{JSCoreValuePrivate};
                use esperanto::jsvalue::*;

                use std::iter::Extend;

                $(
                    esperanto::js_export_definition!(c_method_call,$struct_ident, $export_name, $args_ident, $body);
                )*

                const static_defs: &'static [JSStaticFunction] = &[
                    $(
                        JSStaticFunction {
                            name: concat!($js_func_name,"\0").as_ptr() as *const std::os::raw::c_char,
                            attributes: 0,
                            callAsFunction: Some($export_name)
                        },
                    )*
                    JSStaticFunction {
                        name: std::ptr::null_mut(),
                        attributes: 0,
                        callAsFunction: None
                    }
                ];

                const val: &'static [JSStaticValue] = &[JSStaticValue {
                    name: b"testValue\0".as_ptr() as *const std::os::raw::c_char,
                    attributes: 0,
                    getProperty: None,
                    setProperty:None
                }, JSStaticValue {
                    name: std::ptr::null_mut(),
                    attributes: 0,
                    getProperty: None,
                    setProperty:None
                }];

                unsafe extern "C" fn finalize(value: *mut OpaqueJSValue) {
                    $struct_ident::clear_private(value);
                }

                const def:JSClassDefinition = JSClassDefinition {
                    version: 1,
                    className: concat!($js_class_name,"\0").as_ptr() as *const std::os::raw::c_char,
                    attributes: 0,
                    staticFunctions: static_defs.as_ptr(),
                    callAsConstructor: None,
                    callAsFunction: None,
                    convertToType: None,
                    deleteProperty: None,
                    hasProperty: None,
                    setProperty: None,
                    finalize: Some(finalize),
                    getProperty: None,
                    getPropertyNames: None,
                    hasInstance: None,
                    initialize: None,
                    parentClass: std::ptr::null_mut(),
                    staticValues: val.as_ptr()
                };

                &def

            }
        }

    };

    // Output JS function call:
    (c_method_call,$struct_ident:ident, $export_name: ident, $args_ident: ident, $body: expr) => {
        unsafe extern "C" fn $export_name(
            ctx: *const OpaqueJSContext,
            function: *mut OpaqueJSValue,
            this_object: *mut OpaqueJSValue,
            arg_count: usize,
            arguments: *const *const OpaqueJSValue,
            exception: *mut *const OpaqueJSValue
        ) -> *const OpaqueJSValue {

            let result = (|| -> esperanto::EsperantoResult<JSValue> {
                let ctx = $struct_ident::get_context_from_raw(ctx)?;

                let mut $args_ident = std::slice::from_raw_parts(arguments, arg_count)
                    .iter()
                    .map(|v| { JSValue::try_from_js(*v, ctx)});

                let r: esperanto::EsperantoResult<_> = $body;

                r.map(|v| {JSValue::try_from_js(v, ctx)})?

            })();

            match result {
                Ok(r) => r.into(),
                Err(err) => {

                    let (created_exception, undef) = $struct_ident::manually_map_error(err, ctx);

                    // use esperanto::private::javascriptcore_sys::{
                    //     JSStringCreateWithUTF8CString,JSValueMakeString,JSStringRelease,
                    //     JSObjectMakeError,JSValueUnprotect,JSValueMakeUndefined
                    // };

                    // let err_str = err.to_string();
                    // // since this could have even failed at the context-fetching stage
                    // // we'll do this part manually rather than use our Rust API
                    // let c_string = std::ffi::CString::new(err_str);

                    // // again, we're all out of places to put failures at this point, so if for whatever
                    // // reason we're not able to encode the error as a UTF8 string, we need to
                    // // use a fallback.
                    // const FALLBACK:&[u8] = b"An error occurred but could not get the details\0";

                    // let ptr = match c_string {
                    //     Ok(s) => s.as_ptr(),
                    //     Err(_) => FALLBACK.as_ptr() as *const i8
                    // };

                    // let js_str = JSStringCreateWithUTF8CString(ptr);
                    // let js_val = JSValueMakeString(ctx, js_str);
                    // JSStringRelease(js_str);

                    // let mut create_exception: *const OpaqueJSValue = std::ptr::null_mut();
                    // let err = JSObjectMakeError(ctx,1,vec![js_val].as_ptr(), &mut create_exception);
                    // JSValueUnprotect(ctx, js_val);

                    *exception = created_exception;

                    undef
                }
            }

        }
    };
}
