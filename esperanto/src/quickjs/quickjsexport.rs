use std::{convert::TryInto, ffi::CString, slice};

use quickjs_android_suitable_sys::{
    JSClassCall, JSClassDef, JSContext as QuickJSContext, JSRuntime as QuickJSRuntime,
    JSValue as QuickJSValue, JS_GetClassProto, JS_GetOpaque, JS_GetRuntime, JS_NewClass,
    JS_NewObjectClass, JS_SetClassProto, JS_SetConstructorBit, JS_CALL_FLAG_CONSTRUCTOR,
};

use crate::{
    export::{JSClassFunction, JSExportPrivateData},
    quickjs::{
        quickjs_class_storage::get_existing_class_id, quickjscontextpointer::QuickJSContextPointer,
    },
    shared::{
        errors::{EsperantoResult, JSExportError, JavaScriptError},
        value::JSValueInternal,
    },
    EsperantoError, JSContext, JSExportClass, JSRuntime, JSValue, Retain,
};

use super::quickjs_class_storage::clear_class;

pub(crate) type QuickJSClassID = u32;

pub(super) trait QuickJSExportExtensions: JSExportClass + Sized {
    fn create_prototype_class(
        context: *mut QuickJSContext,
        prototype_class_id: u32,
    ) -> EsperantoResult<()> {
        let runtime = unsafe { JS_GetRuntime(context) };
        let name_cstring = CString::new(Self::CLASS_NAME)?;

        let call: JSClassCall;
        if Self::CALL_AS_CONSTRUCTOR.is_some() || Self::CALL_AS_FUNCTION.is_some() {
            call = Some(class_prototype_call::<Self>);
        } else {
            call = None;
        }

        let definition = JSClassDef {
            class_name: name_cstring.as_ptr(),
            call,
            finalizer: Some(finalize_prototype::<Self>),
            gc_mark: None,
            exotic: std::ptr::null_mut(),
        };

        if unsafe { JS_NewClass(runtime, prototype_class_id, &definition) } != 0 {
            return Err(JSExportError::UnexpectedBehaviour.into());
        }

        // we (seemingly?) need to make sure our prototype has its own prototype set to Object.

        let object = {
            // Grabbed the const value for JS_CLASS_OBJECT from here:
            // https://github.com/bellard/quickjs/blob/2788d71e823b522b178db3b3660ce93689534e6d/quickjs.c#L120

            const JS_CLASS_OBJECT: u32 = 1;
            unsafe { JS_GetClassProto(context, JS_CLASS_OBJECT) }
        };

        unsafe { JS_SetClassProto(context, prototype_class_id, object) };

        Ok(())
    }

    fn create_instance_class(
        runtime: *mut QuickJSRuntime,
        instance_class_id: u32,
    ) -> EsperantoResult<()> {
        let name_cstring = CString::new(Self::CLASS_NAME)?;
        let instance_def = JSClassDef {
            class_name: name_cstring.as_ptr(),
            call: None,
            finalizer: Some(finalize_instance::<Self>),
            gc_mark: None,
            exotic: std::ptr::null_mut(),
        };

        if unsafe { JS_NewClass(runtime, instance_class_id, &instance_def) } != 0 {
            return Err(JSExportError::UnexpectedBehaviour.into());
        }
        Ok(())
    }

    fn create_prototype(context: *mut QuickJSContext, prototype_class_id: u32) -> QuickJSValue {
        // weird quirk in the QuickJS API: created class IDs are u32, JS_NewObjectClass requires
        // i32. Assume it's just an oversight in the header.

        let prototype = unsafe { JS_NewObjectClass(context, prototype_class_id as _) };

        if Self::CALL_AS_CONSTRUCTOR.is_some() {
            unsafe { JS_SetConstructorBit(context, prototype, 1) };
        }

        prototype
    }
}

impl<T> QuickJSExportExtensions for T where T: JSExportClass {}

unsafe extern "C" fn class_prototype_call<T: JSExportClass>(
    ctx: *mut QuickJSContext,
    func_obj: QuickJSValue,
    new_target: QuickJSValue,
    argc: i32,
    argv: *mut QuickJSValue,
    flags: i32,
) -> QuickJSValue {
    let called_as_constructor =
        flags & JS_CALL_FLAG_CONSTRUCTOR as i32 == JS_CALL_FLAG_CONSTRUCTOR as i32;
    let runtime_ptr = JS_GetRuntime(ctx);
    let context_ptr = QuickJSContextPointer::wrap(ctx, false);
    let runtime = JSRuntime::from_raw(runtime_ptr, false);
    let context = JSContext::from_raw_storing_runtime(context_ptr, runtime);

    let args: Vec<JSValue> = slice::from_raw_parts(argv, argc.try_into().unwrap())
        .iter()
        .map(|raw| JSValue::wrap_internal(*raw, &context))
        .collect();

    let execution_target: Option<JSClassFunction>;
    let error: EsperantoError;
    if called_as_constructor {
        execution_target = T::CALL_AS_CONSTRUCTOR;
        error = JSExportError::ConstructorCalledOnNonConstructableClass(T::CLASS_NAME).into();
    } else {
        execution_target = T::CALL_AS_FUNCTION;
        // don't use one of our internal error types here because we want to mirror what
        // JSC provides, which is a TypeError
        error = JavaScriptError::new(
            "TypeError".to_owned(),
            format!(
                "Class constructor {} cannot be invoked without 'new'",
                T::CLASS_NAME
            )
            .to_owned(),
        )
        .into();
    };

    let execution_result: EsperantoResult<Retain<JSValue>>;

    if let Some(to_execute) = execution_target {
        execution_result = (to_execute.func)(&args, &context);
    } else {
        execution_result = Err(error.into())
    }
    //Class constructor Test cannot be invoked without 'new'
    let val = execution_result.unwrap_or_else(|err| {
        let error_jsval = JSValue::try_new_from(err, &context).unwrap();
        context_ptr.throw_error(error_jsval.internal);
        JSValue::undefined(&context)
    });

    val.internal.retain(context_ptr)
}

pub(super) unsafe extern "C" fn finalize_prototype<T: JSExportClass>(
    runtime: *mut QuickJSRuntime,
    _: QuickJSValue,
) {
    clear_class::<T>(runtime);
}

pub(super) unsafe extern "C" fn finalize_instance<T: JSExportClass>(
    runtime: *mut QuickJSRuntime,
    value: QuickJSValue,
) {
    let class_id = get_existing_class_id::<T>(runtime).unwrap().unwrap();
    let storage = unsafe { JS_GetOpaque(value, class_id) };
    JSExportPrivateData::<T>::drop(storage);
}
