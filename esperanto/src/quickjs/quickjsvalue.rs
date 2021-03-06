use std::ffi::{c_void, CStr, CString};

use quickjs_android_suitable_sys::{
    JSValue as QuickJSValue, JS_Call, JS_CallConstructor, JS_DupValue__, JS_FreeCString,
    JS_FreeValue__, JS_GetPropertyStr, JS_GetRuntime, JS_IsEqual__, JS_IsError, JS_IsInstanceOf,
    JS_IsObject__, JS_IsString__, JS_NewBool__, JS_NewClass, JS_NewClassID, JS_NewError,
    JS_NewFloat64__, JS_NewObjectProtoClass, JS_NewString, JS_SetOpaque, JS_SetPropertyStr,
    JS_ToBool, JS_ToCStringLen2, JS_ToFloat64, JS_UNDEFINED__,
};

use crate::{
    shared::{
        context::JSContextInternal,
        errors::CatchExceptionError,
        errors::EsperantoError,
        errors::EsperantoResult,
        value::{JSValueError, JSValueInternal},
    },
    JSValueRef,
};

use super::quickjs_prototype_storage::get_or_create_prototype;
use super::quickjscontextpointer::QuickJSContextPointer;
use super::quickjsexport::QuickJSExportExtensions;
pub type QuickJSValueInternal = QuickJSValue;

impl JSValueInternal for QuickJSValueInternal {
    type ContextType = QuickJSContextPointer;

    fn retain(self, ctx: Self::ContextType) -> Self {
        unsafe { JS_DupValue__(*ctx, self) }.into()
    }

    fn release(self, ctx: Self::ContextType) {
        unsafe { JS_FreeValue__(*ctx, self) }
    }

    fn as_cstring(self, ctx: Self::ContextType) -> EsperantoResult<std::ffi::CString> {
        let ptr = check_quickjs_exception!(ctx => {
            unsafe { JS_ToCStringLen2(*ctx, std::ptr::null_mut(), self, 0) }
        })?;

        let cstr = unsafe { CStr::from_ptr(ptr) };
        let cstring = cstr.to_owned();
        unsafe { JS_FreeCString(*ctx, ptr) };
        Ok(cstring)
    }

    fn from_cstring(value: &std::ffi::CString, ctx: Self::ContextType) -> Self {
        unsafe { JS_NewString(*ctx, value.as_ptr()) }.into()
    }

    fn as_number(self, ctx: Self::ContextType) -> EsperantoResult<f64> {
        let mut result = 0.0;
        let success = unsafe { JS_ToFloat64(*ctx, &mut result, self) };
        if success != 0 {
            return Err(JSValueError::IsNotANumber.into());
        }
        return Ok(result);
    }

    fn from_number(number: f64, ctx: Self::ContextType) -> EsperantoResult<Self> {
        Ok(unsafe { JS_NewFloat64__(*ctx, number) }.into())
    }

    fn as_bool(self, ctx: Self::ContextType) -> EsperantoResult<bool> {
        Ok(unsafe { JS_ToBool(*ctx, self) } == 1)
    }

    fn from_bool(
        bool: bool,
        ctx: Self::ContextType,
    ) -> crate::shared::errors::EsperantoResult<Self> {
        Ok(unsafe {
            JS_NewBool__(
                *ctx,
                match bool {
                    true => 1,
                    false => 0,
                },
            )
        }
        .into())
    }

    fn new_error(name: CString, message: CString, ctx: Self::ContextType) -> Self {
        let err = unsafe { JS_NewError(*ctx) };
        let name_jsv = Self::from_cstring(&name, ctx);
        let message_jsv = Self::from_cstring(&message, ctx);

        const NAME_PROP_STR: &[u8] = b"name\0";
        const MESSAGE_PROP_STR: &[u8] = b"message\0";

        let name_ident = unsafe { CStr::from_ptr(NAME_PROP_STR.as_ptr() as *const i8) }.to_owned();
        let message_ident =
            unsafe { CStr::from_ptr(MESSAGE_PROP_STR.as_ptr() as *const i8) }.to_owned();

        err.set_property(ctx.into(), &name_ident, name_jsv)
            .unwrap_or_else(|_| {});

        err.set_property(ctx.into(), &message_ident, message_jsv)
            .unwrap_or_else(|_| {});

        name_jsv.release(ctx);
        message_jsv.release(ctx);

        err
    }

    fn native_prototype_for<T: crate::JSExportClass>(
        ctx: Self::ContextType,
    ) -> EsperantoResult<Self> {
        get_or_create_prototype::<T>(*ctx)
    }

    fn from_native_class<T: crate::JSExportClass>(
        instance: T,
        ctx: Self::ContextType,
    ) -> EsperantoResult<Self> {
        let proto = Self::native_prototype_for::<T>(ctx)?;

        let runtime = unsafe { JS_GetRuntime(*ctx) };

        let mut cid: u32 = 0;
        let definition = T::class_def();
        let class_id = unsafe { JS_NewClassID(&mut cid) };
        unsafe { JS_NewClass(runtime, class_id, &definition) };
        let obj = unsafe { JS_NewObjectProtoClass(*ctx, proto, class_id) };

        proto.release(ctx);

        // proto.release(ctx);

        let boxed = Box::new(instance);

        unsafe { JS_SetOpaque(obj, Box::into_raw(boxed) as *mut c_void) }
        Ok(obj.into())
    }

    fn set_property(
        self,
        ctx: Self::ContextType,
        name: &std::ffi::CString,
        new_value: Self,
    ) -> Result<(), crate::EsperantoError> {
        if unsafe { JS_IsObject__(self) } == 0 {
            // QuickJS will happily return a TypeError if we don't do this first, but JavaScriptCore
            // will throw an object-specific error. So for the sake of consistency, we replicate that here.
            return Err(JSValueError::IsNotAnObject.into());
        }

        // according to comments in quickjs.c, SetPropertyInternal frees the value it's given, which isn't
        // something we want, so we'll retain first:
        let retained = new_value.retain(ctx);

        check_quickjs_exception!(ctx => {
            unsafe { JS_SetPropertyStr(*ctx, self, name.as_ptr(), retained) }
        })?;

        Ok(())
    }

    fn get_property(
        self,
        ctx: Self::ContextType,
        name: &std::ffi::CStr,
    ) -> Result<Self, crate::EsperantoError> {
        Ok(unsafe { JS_GetPropertyStr(*ctx, self, name.as_ptr()) }.into())
    }

    fn new_function(
        function_text: &CString,
        argument_names: &Vec<CString>,
        ctx: Self::ContextType,
    ) -> EsperantoResult<Self> {
        static FUNCTION_NAME: &[u8] = b"Function\0";
        let global_obj = ctx.get_globalobject();

        let func_constructor =
            unsafe { JS_GetPropertyStr(*ctx, global_obj, FUNCTION_NAME.as_ptr() as *const i8) };

        let mut construct_arguments: Vec<Self> = argument_names
            .iter()
            .map(|cstr| Self::from_cstring(cstr, ctx))
            .collect();

        construct_arguments.push(Self::from_cstring(function_text, ctx));

        let result = func_constructor.call_as_constructor(construct_arguments.clone(), ctx);
        global_obj.release(ctx);
        func_constructor.release(ctx);
        construct_arguments.iter().for_each(|str| str.release(ctx));
        result
    }

    fn call_as_function(
        self,
        arguments: Vec<Self>,
        bound_to: Option<Self>,
        ctx: Self::ContextType,
    ) -> EsperantoResult<Self> {
        let argc = arguments.len() as i32;
        let mut argv: Vec<QuickJSValue> = arguments.iter().map(|a| *a).collect();
        let bound = bound_to.unwrap_or(Self::undefined(ctx));
        check_quickjs_exception!(ctx => {
            unsafe { JS_Call(*ctx, self, bound, argc, argv.as_mut_ptr()) }
        })
    }

    fn call_as_constructor(
        self,
        arguments: Vec<Self>,
        ctx: Self::ContextType,
    ) -> EsperantoResult<Self> {
        let argc = arguments.len() as i32;
        let mut argv: Vec<QuickJSValue> = arguments.iter().map(|a| *a).collect();
        check_quickjs_exception!(ctx => {
            unsafe {JS_CallConstructor(*ctx, self, argc, argv.as_mut_ptr())}
        })
    }

    fn undefined(_: Self::ContextType) -> Self {
        unsafe { JS_UNDEFINED__ }
    }

    fn is_string(self, _: Self::ContextType) -> bool {
        unsafe { JS_IsString__(self) == 1 }
    }

    fn equals(self, other: Self, _: Self::ContextType) -> bool {
        unsafe { JS_IsEqual__(self, other) == 1 }
    }

    fn is_instanceof(self, target: Self, ctx: Self::ContextType) -> EsperantoResult<bool> {
        let result = check_quickjs_exception!(ctx => {
            unsafe {JS_IsInstanceOf(*ctx,self, target)}
        })?;
        if result != 0 && result != 1 {
            // we got a result we aren't expecting but no exception was thrown
            return Err(EsperantoError::CatchExceptionError(Box::new(
                CatchExceptionError::UnknownExceptionOccurred,
            )));
        }
        Ok(result == 1)
    }

    fn is_error(self, ctx: Self::ContextType) -> bool {
        unsafe { JS_IsError(*ctx, self) == 1 }
    }
}

impl<'c> From<JSValueRef<'c>> for QuickJSValue {
    fn from(val: JSValueRef<'c>) -> Self {
        val.internal
    }
}

#[cfg(test)]
mod test {
    use crate::JSContext;
    use quickjs_android_suitable_sys::*;
    #[test]
    fn wtf() {
        let ctx = JSContext::new().unwrap();
        let ctxptr = ctx.internal;

        // let obj = unsafe { JS_AtomToValue(*ctxptr, JS_ATOM_object) };

        let obj = ctx.evaluate("Object", None).unwrap().internal;
        let prop =
            unsafe { JS_GetPropertyInternal(*ctxptr, obj, JS_ATOM_Symbol_hasInstance, obj, 0) };

        let tag = unsafe { JS_GetTag__(prop) };

        unsafe { JS_FreeValue__(*ctxptr, prop) };
        println!("{}", tag)
    }
}
