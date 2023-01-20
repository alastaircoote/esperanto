use std::ffi::CString;

use quickjs_android_suitable_sys::{JSClassDef, JSContext, JSValue};

use super::quickjs_prototype_storage::delete_stored_prototype_extern;
use crate::{
    shared::errors::{ConversionError, EsperantoResult},
    JSExportClass,
};

pub type QuickJSCallAsFunction =
    unsafe extern "C" fn(*mut JSContext, JSValue, JSValue, i32, *mut JSValue, i32) -> JSValue;
pub type QuickJSCallAsConstructor =
    unsafe extern "C" fn(*mut JSContext, JSValue, JSValue, i32, *mut JSValue, i32) -> JSValue;


pub(super) trait QuickJSExportExtensions: JSExportClass + Sized {
    // fn prototype_def() -> JSClassDef {
    //     JSClassDef {
    //         class_name: Self::METADATA.class_name as *const i8,
    //         call: Self::METADATA.optional.call_as_constructor,
    //         finalizer: Some(delete_stored_prototype::<Self>),
    //         gc_mark: None,
    //         exotic: std::ptr::null_mut(),
    //     }
    // }

    fn class_def() -> EsperantoResult<JSClassDef> {
        let name_cstring = CString::new(Self::METADATA.class_name)
            .map_err(|e| ConversionError::CouldNotConvertToJSString(e))?;
        Ok(JSClassDef {
            class_name: name_cstring.as_ptr(),
            call: None,
            // call: Some(test_call),
            finalizer: Some(delete_stored_prototype_extern::<Self>),
            gc_mark: None,
            exotic: std::ptr::null_mut(),
        })
    }
}

impl<T> QuickJSExportExtensions for T where T: JSExportClass {}

#[macro_export]
macro_rules! js_export_class_impl {
    (CREATE_VALUES_VEC: $argc:ident, $argv: ident, &$ctx:ident) => {{
        let mut values_vec = Vec::<JSValueRef>::with_capacity(($argc) as usize);

        std::slice::from_raw_parts($argv, $argc as usize)
            .iter()
            .for_each(|raw| {
                values_vec.push(JSValueRef::from((*raw, &$ctx)));
            });

        values_vec
    }};
    (CALL_AS_FUNCTION: $func_ident:ident, $ctx_ident: pat, $this_ident:pat, $values_ident: pat, $stmt: stmt) => {
        unsafe extern "C" fn $func_ident(
            ctx: *mut quickjs_android_suitable_sys::JSContext,
            _: quickjs_android_suitable_sys::JSValue,
            this_obj: quickjs_android_suitable_sys::JSValue,
            argc: i32,
            argv: *mut quickjs_android_suitable_sys::JSValue,
            _: i32,
        ) -> quickjs_android_suitable_sys::JSValue {
            use $crate::{EsperantoError, JSContext, JSValueFrom, JSValueRef};
            let context: JSContext = JSContext::from(ctx);

            let values_vec = $crate::js_export_class_impl!(CREATE_VALUES_VEC: argc, argv, &context);

            fn inner<'a>(
                $ctx_ident: &'a JSContext,
                $this_ident: JSValueRef,
                $values_ident: Vec<JSValueRef>,
            ) -> Result<JSValueRef<'a>, EsperantoError> {
                $stmt
            }
            let result = match inner(&context, JSValueRef::from((this_obj, &context)), values_vec) {
                Ok(success_jsval) => return success_jsval.into(),
                Err(error) => {
                    let error_val: JSValueRef = JSValueRef::new_value_from(error, &context).into();
                    error_val.into()
                }
            };

            result
        }
    };
    (CALL_AS_CONSTRUCTOR: $func_ident:ident, $struct_ident:ident, $ctx_ident: pat, $values_ident: pat, $stmt: stmt) => {
        unsafe extern "C" fn $func_ident(
            ctx: *mut quickjs_android_suitable_sys::JSContext,
            func_obj: quickjs_android_suitable_sys::JSValue,
            this_obj: quickjs_android_suitable_sys::JSValue,
            argc: i32,
            argv: *mut quickjs_android_suitable_sys::JSValue,
            flags: i32,
        ) -> quickjs_android_suitable_sys::JSValue {
            use $crate::{
                errors::JSValueError, EsperantoError, JSContext, JSValueFrom, JSValueRef,
            };
            let context: JSContext = JSContext::from(ctx);

            // let func_obj_jsv = JSValueRef::from((func_obj, &context));
            // let this_obj_jsv = JSValueRef::from((this_obj, &context));

            // As per the QuickJS comments, the this_obj is new.target when called as a constructor:
            // https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/new.target
            // We need to check whether the code used 'new' or not.
            // if (func_obj_jsv != this_obj_jsv
            // /*|| flags & quickjs_android_suitable_sys::JS_CALL_FLAG_CONSTRUCTOR as i32 != 0*/)
            // {
            //     let error_val = JSValueRef::new_value_from(
            //         JSValueError::MustUseNewWithConstuctor.into(),
            //         &context,
            //     );

            //     return error_val.into();
            // }

            let values_vec = $crate::js_export_class_impl!(CREATE_VALUES_VEC: argc, argv, &context);

            fn inner<'a>(
                $ctx_ident: &'a JSContext,
                $values_ident: Vec<JSValueRef>,
            ) -> Result<$struct_ident, EsperantoError> {
                $stmt
            }
            let result = match inner(&context, values_vec)
                .and_then(|success_struct| JSValueRef::wrap_native(success_struct, &context))
            {
                Ok(success) => success.test_norelease(),
                Err(error) => {
                    let error_val: JSValueRef = JSValueRef::new_value_from(error, &context).into();
                    error_val.into()
                }
            };
            // drop(func_obj_jsv);
            // drop(this_obj_jsv);
            // drop(context);
            result
        }
    };
}
