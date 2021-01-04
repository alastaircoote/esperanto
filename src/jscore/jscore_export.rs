use javascriptcore_sys::{JSClassDefinition, JSStaticFunction};
use thiserror::Error;

use crate::errors::ImplementationError;

#[derive(Debug, Error)]
pub enum JSExportError {
    #[error("You did not supply enough arguments")]
    NotEnoughArguments,
    #[error(transparent)]
    NativeSideErrorOccurred(Box<dyn std::error::Error>),
}

impl ImplementationError for JSExportError {}

pub trait JSExport: 'static {
    fn get_definition() -> JSClassDefinition;
}
pub trait HasJSExportDefinition {}

#[macro_export]
macro_rules! js_export_definition {
    // Root call:
    ($js_class_name: expr => $struct_ident: ident, {
        static_functions: {
            $($js_func_name: expr => $export_name:ident |$args_ident:ident| $body:expr),*
        }
    }) => {
        impl esperanto::private::jscore_export::JSExport for $struct_ident {

            fn get_definition() -> javascriptcore_sys::JSClassDefinition {
                use javascriptcore_sys::{OpaqueJSValue, OpaqueJSContext,JSStaticFunction, JSClassDefinition};
                use esperanto::private::jscore_context::{JSCoreContextPrivate, JSCoreContext};
                use esperanto::private::jscore_value::{JSCoreValue};
                use esperanto::traits::JSValue;
                use std::iter::Extend;

                $(
                    esperanto::js_export_definition!(c_method_call, $export_name, $args_ident, $body);
                )*

                let static_defs = [
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

                JSClassDefinition {
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
                    finalize: None,
                    getProperty: None,
                    getPropertyNames: None,
                    hasInstance: None,
                    initialize: None,
                    parentClass: std::ptr::null_mut(),
                    staticValues: std::ptr::null_mut()
                }
            }
        }
        fn whaaa() {
            // mod c_method_calls {
                use javascriptcore_sys::{OpaqueJSValue, OpaqueJSContext,JSStaticFunction, JSClassDefinition};
                use esperanto::private::jscore_context::{JSCoreContextPrivate, JSCoreContext};
                use esperanto::private::jscore_value::{JSCoreValue};
                use esperanto::traits::JSValue;
                use std::iter::Extend;

                $(
                    esperanto::js_export_definition!(c_method_call, $export_name, $args_ident, $body);
                )*

                let static_defs = [
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

                let definition = JSClassDefinition {
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
                    finalize: None,
                    getProperty: None,
                    getPropertyNames: None,
                    hasInstance: None,
                    initialize: None,
                    parentClass: std::ptr::null_mut(),
                    staticValues: std::ptr::null_mut()
                };
            // }
        }
    };

    // Output JS function call:
    (c_method_call, $export_name: ident, $args_ident: ident, $body: expr) => {
        unsafe extern "C" fn $export_name(
            ctx: *const OpaqueJSContext,
            function: *mut OpaqueJSValue,
            this_object: *mut OpaqueJSValue,
            arg_count: usize,
            arguments: *const *const OpaqueJSValue,
            exception: *mut *const OpaqueJSValue
        ) -> *const OpaqueJSValue {


            let result = (|| -> esperanto::EsperantoResult<JSCoreValue> {
                let ctx = JSCoreContext::borrow_from_raw(ctx)?;

                let mut $args_ident = std::slice::from_raw_parts(arguments, arg_count)
                    .iter()
                    .map(|v| { JSCoreValue::try_from_js(*v, ctx)});

                let r: esperanto::EsperantoResult<_> = $body;

                r.map(|v| {JSCoreValue::try_from_js(v, ctx)})?

            })();

            match result {
                Ok(r) => r.raw_ref.as_const(),
                Err(err) => {

                    let err_str = err.to_string();
                    // since this could have even failed at the context-fetching stage
                    // we'll do this part manually rather than use our Rust API
                    let c_string = std::ffi::CString::new(err_str);

                    // again, we're all out of places to put failures at this point, so if for whatever
                    // reason we're not able to encode the error as a UTF8 string, we need to
                    // use a fallback.
                    const FALLBACK:&[u8] = b"An error occurred but could not get the details\0";

                    let ptr = match c_string {
                        Ok(s) => s.as_ptr(),
                        Err(_) => FALLBACK.as_ptr() as *const i8
                    };

                    let js_str = javascriptcore_sys::JSStringCreateWithUTF8CString(ptr);
                    let js_val = javascriptcore_sys::JSValueMakeString(ctx, js_str);
                    javascriptcore_sys::JSStringRelease(js_str);

                    let mut create_exception: *const OpaqueJSValue = std::ptr::null_mut();
                    let err = javascriptcore_sys::JSObjectMakeError(ctx,1,vec![js_val].as_ptr(), &mut create_exception);
                    javascriptcore_sys::JSValueUnprotect(ctx, js_val);

                    if create_exception.is_null() == false {
                        // The error creation process threw an exception, so we'll have to just
                        // pass that back into the JS environment without knowing what happened
                        *exception = create_exception
                    } else {
                        *exception = err;
                        javascriptcore_sys::JSValueUnprotect(ctx, err);
                    }

                    let undef = javascriptcore_sys::JSValueMakeUndefined(ctx);
                    javascriptcore_sys::JSValueUnprotect(ctx,undef);
                    undef
                }
            }




        }
    };
    // PARSE JS ARGS
    // If this method uses &self then we need to prepend this_object to our collection of
    // JSValues:
    (parse_args, $ctx:ident, $arguments:ident, $arg_count:ident, $this_object:ident, $self_expr: expr) => {
        std::iter::once(
            JSCoreValue::try_from($this_object, &$ctx)
        )
        .chain(esperanto::js_export_definition!(parse_args, $ctx, $arguments, $arg_count, $this_object))
    };
    // If not, we just process the arguments array as-is:
    (parse_args, $ctx:ident, $arguments:ident, $arg_count:ident, $this_object:ident) => {
        std::slice::from_raw_parts($arguments, $arg_count)
        .iter().map(|v| { JSCoreValue::try_from(*v, &$ctx)})
    };

    (repeat_args, $target:ident, $args:ident, 1) => {
        $target($args.next())
    };
    (repeat_args, $target:ident, $args:ident, 2) => {
        $target($args.next(), $args.next())
    }
}

// macro_rules! test_macro {
//     (a, $($item:expr),*) => {
//         test_macro!(b,$($item),*);
//     };
//     (b, $item1:expr, $item2:expr) => {
//         println!("{},{}", $item1, $item2);
//     }
// }

// test_macro!(a, "one", "two");

#[macro_export]
macro_rules! js_export_test {
    ($js_class_name: expr => $rust_ident: ident, {
        $($js_name: expr => $export_name: ident; $fn_name:ident($num_args:expr$(; $is_self:expr),*)),+
    }) => {
        fn __get_js_export_definition() {
            use esperanto::private::jscore_context::{JSCoreContextPrivate, JSCoreContext};
            use esperanto::private::jscore_value::JSCoreValue;
            use javascriptcore_sys::{OpaqueJSValue, OpaqueJSContext};
            use esperanto::traits::JSValue;



            // fn with_args()

            $(
                // The C functions that JavaScript core will call
                unsafe extern "C" fn $export_name(
                    ctx: *const OpaqueJSContext,
                    function: *mut OpaqueJSValue,
                    this_object: *mut OpaqueJSValue,
                    arg_count: usize,
                    arguments: *const *const OpaqueJSValue,
                    exception: *mut *const OpaqueJSValue,
                ) {
                    let result = (|| -> esperanto::EsperantoResult<()> {
                        let ctx = JSCoreContext::wrap_existing_raw(ctx)?;
                        if (arg_count < $num_args) {
                            return Err(esperanto::errors::JSExportError::NotEnoughArguments {
                                expected: $num_args,
                                actual: arg_count,
                            }
                            .into());
                        }
                        // let i = std::iter::Iterator::new();
                        let args_raw = std::slice::from_raw_parts(arguments, arg_count).to_vec();
                        let args_mapped = args_raw.iter().map(|v| {
                            JSCoreValue::try_from(*v, &ctx);
                        });
                        let hm = Hmm {};
                        $rust_ident::$fn_name(&hm);
                        args_mapped.next();
                        Ok(())
                    })();
                }
            );+
        }
    };
}
// trait Whattt {
//     fn what() {
//         unsafe extern "C" fn test() {}
//         javascriptcore_sys::JSStaticFunction {
//             name: std::ptr::null_mut(),
//             attributes: 0,
//             // callAsFunction: Some(unsafe extern "C" fn() {

//             // })
//         }
//     }
// }
