use std::{convert::TryInto, ffi::CString, slice};

use quickjs_android_suitable_sys::{
    JSClassCall, JSClassDef, JSContext as QuickJSContext, JSRuntime as QuickJSRuntime,
    JSValue as QuickJSValue, JS_FreeValue__, JS_GetClassProto, JS_GetRuntime, JS_NewClass,
    JS_NewClassID, JS_NewObject, JS_NewObjectClass, JS_NewObjectProtoClass, JS_SetClassProto,
    JS_SetConstructorBit, JS_CALL_FLAG_CONSTRUCTOR,
};

use crate::{
    export::{
        get_stored_prototype_info, store_prototype_info, JSClassFunction, JSExportStoredClassInfo,
    },
    quickjs::quickjscontextpointer::QuickJSContextPointer,
    shared::{
        errors::{ConversionError, EsperantoResult, JSExportError, JavaScriptError},
        value::JSValueInternal,
    },
    EsperantoError, JSContext, JSExportClass, JSRuntime, JSValue, Retain,
};

pub(crate) type QuickJSClassID = u32;

fn get_new_class_id() -> u32 {
    let mut prototype_class_id: u32 = 0;
    unsafe { JS_NewClassID(&mut prototype_class_id) };
    return prototype_class_id;
}

pub(super) trait QuickJSExportExtensions: JSExportClass + Sized {
    fn get_or_create_class_prototype(
        in_context: *mut QuickJSContext,
        runtime: *mut QuickJSRuntime,
    ) -> EsperantoResult<JSExportStoredClassInfo> {
        if let Some(existing) = get_stored_prototype_info::<Self>(runtime)? {
            return Ok(existing);
        }

        let name_cstring = CString::new(Self::CLASS_NAME)?;

        let call: JSClassCall;
        if Self::CALL_AS_CONSTRUCTOR.is_some() || Self::CALL_AS_FUNCTION.is_some() {
            call = Some(class_prototype_call::<Self>);
        } else {
            call = None;
        }

        let prototype_def = JSClassDef {
            class_name: name_cstring.as_ptr(),
            call,
            finalizer: Some(finalize_prototype::<Self>),
            gc_mark: None,
            exotic: std::ptr::null_mut(),
        };

        let prototype_class_id = get_new_class_id();

        // Then create a class from our definition using that ID:
        if unsafe { JS_NewClass(runtime, prototype_class_id, &prototype_def) } != 0 {
            return Err(JSExportError::UnexpectedBehaviour.into());
        }

        // we (seemingly?) need to make sure our prototype has its own prototype set to Object. Grabbed
        // the const value for JS_CLASS_OBJECT from here:
        // https://github.com/bellard/quickjs/blob/2788d71e823b522b178db3b3660ce93689534e6d/quickjs.c#L120

        const JS_CLASS_OBJECT: u32 = 1;
        let obj_proto = unsafe { JS_GetClassProto(in_context, JS_CLASS_OBJECT) };
        unsafe { JS_SetClassProto(in_context, prototype_class_id, obj_proto) };

        // Quirk in QuickJS API here? It defines JSClassId as u32 but NewObjectClass wants an i32
        let asu32: i32 = prototype_class_id
            .try_into()
            .map_err(|_| JSExportError::UnexpectedBehaviour)?;

        let prototype = unsafe { JS_NewObjectClass(in_context, asu32) };

        if Self::CALL_AS_CONSTRUCTOR.is_some() {
            unsafe { JS_SetConstructorBit(in_context, prototype, 1) };
        }

        let instance_def = JSClassDef {
            class_name: name_cstring.as_ptr(),
            call: None,
            finalizer: Some(finalize_instance::<Self>),
            gc_mark: None,
            exotic: std::ptr::null_mut(),
        };

        let instance_class_id = get_new_class_id();
        if unsafe { JS_NewClass(runtime, instance_class_id, &instance_def) } != 0 {
            return Err(JSExportError::UnexpectedBehaviour.into());
        }

        store_prototype_info::<Self>(prototype, instance_class_id, runtime)
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
    value: QuickJSValue,
) {
    println!("finalize proto");
}

pub(super) unsafe extern "C" fn finalize_instance<T: JSExportClass>(
    runtime: *mut QuickJSRuntime,
    value: QuickJSValue,
) {
    println!("finalize instance");
}
