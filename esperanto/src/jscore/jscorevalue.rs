use std::{
    convert::TryInto,
    ffi::{c_void, CStr, CString},
    ops::DerefMut,
};

use javascriptcore_sys::{
    JSObjectCallAsConstructor, JSObjectCallAsFunction, JSObjectDeleteProperty, JSObjectGetPrivate,
    JSObjectGetProperty, JSObjectMake, JSObjectMakeError, JSObjectMakeFunction, JSObjectSetPrivate,
    JSObjectSetProperty, JSObjectSetPrototype, JSValueIsInstanceOfConstructor, JSValueIsObject,
    JSValueIsStrictEqual, JSValueIsString, JSValueMakeBoolean, JSValueMakeNumber,
    JSValueMakeString, JSValueMakeUndefined, JSValueProtect, JSValueToBoolean, JSValueToNumber,
    JSValueToStringCopy, JSValueUnprotect, OpaqueJSContext, OpaqueJSString, OpaqueJSValue,
};

use crate::{
    export::JSExportPrivateData,
    shared::{
        context::JSContextImplementation,
        errors::EsperantoResult,
        value::{JSValueError, JSValueImplementation},
    },
    JSExportClass,
};

use crate::shared::as_ptr::AsRawMutPtr;

use super::{
    jscore_class_storage::JSClassStorage, jscorestring::JSCoreString,
    jscorevaluepointer::JSCoreValuePointer,
};

pub(crate) type JSCoreValueInternal = JSCoreValuePointer;

// static CONSTRUCTOR_STRING: &CStr = unsafe { CStr::from_bytes_with_nul_unchecked(b"constructor\0") };

impl JSValueImplementation for JSCoreValueInternal {
    type ContextType = *mut OpaqueJSContext;

    fn retain(self, ctx: Self::ContextType) -> Self {
        unsafe { JSValueProtect(ctx, self.as_value()) }
        self.clone()
    }

    fn release(self, ctx: Self::ContextType) {
        unsafe { JSValueUnprotect(ctx, self.as_value()) };
    }

    fn as_cstring(self, ctx: Self::ContextType) -> EsperantoResult<CString> {
        let ptr = check_jscore_exception!(ctx, exception => {
            unsafe { JSValueToStringCopy(ctx, self.as_value(), exception) }
        })?;

        let jsc_string = JSCoreString::from_retained_ptr(ptr);
        Ok(jsc_string.try_into()?)
    }

    fn from_cstring(value: &CString, ctx: Self::ContextType) -> Self {
        let mut js_string = JSCoreString::from(value);
        let ptr = unsafe { JSValueMakeString(ctx, js_string.as_mut_raw_ptr()) };
        ptr.into()
    }

    fn as_number(self, ctx: Self::ContextType) -> EsperantoResult<f64> {
        check_jscore_exception!(ctx, exception => {
            unsafe { JSValueToNumber(ctx, self.as_value(), exception) }
        })
    }

    fn from_number(number: f64, ctx: Self::ContextType) -> EsperantoResult<Self> {
        let ptr = unsafe { JSValueMakeNumber(ctx, number) };
        Ok(ptr.into())
    }

    fn from_bool(bool: bool, ctx: Self::ContextType) -> EsperantoResult<Self> {
        let ptr = unsafe { JSValueMakeBoolean(ctx, bool) };
        Ok(ptr.into())
    }

    fn as_bool(self, ctx: Self::ContextType) -> EsperantoResult<bool> {
        Ok(unsafe { JSValueToBoolean(ctx, self.as_value()) })
    }

    fn set_property(
        self,
        ctx: Self::ContextType,
        name: &CString,
        new_value: Self,
    ) -> EsperantoResult<()> {
        let mut name_jsstring = JSCoreString::from(name);
        check_jscore_exception!(ctx, exception => {
            unsafe {JSObjectSetProperty(ctx, self.try_as_object(ctx)?, name_jsstring.as_mut_raw_ptr(), new_value.as_value(), 0, exception)}
        })
    }

    fn get_property(self, ctx: Self::ContextType, name: &CStr) -> EsperantoResult<Self> {
        let mut name_jsstring = JSCoreString::from(name);
        let result = check_jscore_exception!(ctx, exception => {
            unsafe {
                JSObjectGetProperty(
                    ctx,
                    self.try_as_object(ctx)?,
                    name_jsstring.as_mut_raw_ptr(),
                    exception,
                )
            }
        })?;

        Ok(result.into())
    }

    fn from_native_class<T: JSExportClass>(
        instance: T,
        ctx: Self::ContextType,
        runtime: &<Self::ContextType as JSContextImplementation>::RuntimeType,
    ) -> EsperantoResult<Self> {
        let private_data = JSExportPrivateData::from_instance(instance);
        let class = JSClassStorage::get_or_create::<T>(ctx, runtime)?;

        // The "create rule"
        // https://developer.apple.com/library/archive/documentation/CoreFoundation/Conceptual/CFMemoryMgmt/Concepts/Ownership.html#//apple_ref/doc/uid/20001148-103029
        // states that a method with "Create" in is already owned (i.e. retained) but it looks like "Make"
        // isn't because this object does not appear to be retained. Which honestly doesn't make a lot of sense
        // to me but oh well

        let raw = unsafe { JSObjectMake(ctx, class.instance_class, private_data) };

        // So let's retain it:

        unsafe { JSValueProtect(ctx, raw) }
        unsafe { JSObjectSetPrototype(ctx, raw, class.prototype) }

        return Ok(JSCoreValuePointer::Object(raw));
    }

    fn new_error(name: CString, message: CString, ctx: Self::ContextType) -> Self {
        let message_val = JSCoreValuePointer::from_cstring(&message, ctx);
        let args = [message_val.as_value()];
        let create_result: Result<*mut OpaqueJSValue, _> = check_jscore_exception!(ctx, exception => {
            unsafe { JSObjectMakeError(ctx, 1, args.as_ptr(), exception) }
        });
        let ptr: JSCoreValuePointer = create_result
            .expect("Could not create a JavaScript error")
            .into();

        if let Ok(name_property) = CString::new("name") {
            let name_val = JSCoreValuePointer::from_cstring(&name, ctx.into());
            let try_set_name = ptr.set_property(ctx, &name_property, name_val);
            if let Err(_) = try_set_name {
                // need to add some logging or whatever here. Not a critical failure but all the
                // same, it shouldn't ever fail
            }
        }

        ptr
    }

    fn new_function(
        function_text: &CString,
        argument_names: &Vec<CString>,
        ctx: Self::ContextType,
    ) -> EsperantoResult<Self> {
        let mut body_jsstring = JSCoreString::from(function_text);
        let argc = argument_names.len();

        let mut argument_names_jsstring: Vec<JSCoreString> = argument_names
            .iter()
            .map(|cstr| JSCoreString::from(cstr))
            .collect();

        let argument_names_ptrs: Vec<&mut OpaqueJSString> = argument_names_jsstring
            .iter_mut()
            .map(|jsc| jsc.deref_mut())
            .collect();

        let func = check_jscore_exception!(ctx, exception => {
            unsafe {
                JSObjectMakeFunction(
                    ctx,
                    // JSC does let you make named functions but it doesn't appear to actually
                    // work (I get a SyntaxError), so just skipping over it entirely for now.
                    std::ptr::null_mut(),
                    argc as u32,
                    argument_names_ptrs.as_ptr() as *const *mut OpaqueJSString,
                    body_jsstring.as_mut_raw_ptr(),
                    // we don't add source URL info for functions because there isn't really
                    // a situation where that's going to make sense (unlike eval)
                    std::ptr::null_mut(),
                    1,
                    exception,
                )
            }
        })?;

        let as_internal: JSCoreValuePointer = func.into();

        // JSC 'create rule' does not apply to functions with "Make" in them, so we need to retain:
        Ok(as_internal.retain(ctx))
    }

    fn call_as_function(
        self,
        arguments: Vec<Self>,
        bound_to: Option<Self>,
        ctx: Self::ContextType,
    ) -> EsperantoResult<Self> {
        let argc = arguments.len();

        let arg_ptrs: Vec<*const OpaqueJSValue> = arguments.iter().map(|a| a.as_value()).collect();

        let bound_to_ptr = bound_to.map_or(Ok(std::ptr::null_mut()), |b| b.try_as_object(ctx))?;

        let raw = check_jscore_exception!(ctx, exception => {
            unsafe {
                JSObjectCallAsFunction(
                    ctx,
                    self.try_as_object(ctx)?,
                    bound_to_ptr,
                    argc,
                    arg_ptrs.as_ptr(),
                    exception,
                )
            }
        })?;

        Ok(raw.into())
    }

    fn call_as_constructor(
        self,
        arguments: Vec<Self>,
        ctx: Self::ContextType,
    ) -> EsperantoResult<Self> {
        let argc = arguments.len();
        let arg_ptrs: Vec<*const OpaqueJSValue> = arguments.iter().map(|a| a.as_value()).collect();

        let raw = check_jscore_exception!(ctx, exception => {
            unsafe {
                JSObjectCallAsConstructor(
                    ctx,
                    self.try_as_object(ctx)?,
                    argc,
                    arg_ptrs.as_ptr(),
                    exception,
                )
            }
        })?;

        Ok(raw.into())
    }

    fn is_string(self, ctx: Self::ContextType) -> bool {
        unsafe { JSValueIsString(ctx, self.as_value()) }
    }

    fn undefined(ctx: Self::ContextType) -> Self {
        unsafe { JSValueMakeUndefined(ctx) }.into()
    }

    fn native_prototype_for<'r: 'c, 'c, T: JSExportClass>(
        ctx: Self::ContextType,
        runtime: &<Self::ContextType as JSContextImplementation>::RuntimeType,
    ) -> EsperantoResult<Self> {
        let class = JSClassStorage::get_or_create::<T>(ctx, runtime)?;
        let value = class.prototype;
        Ok(value.into())
    }

    // fn constructor_for<T: JSExportClass>(wrapped_ctx: &JSContext) -> EsperantoResult<Self> {
    //     let prototype = Self::native_prototype_for::<T>(wrapped_ctx)?;
    //     let constructor =
    //         prototype.get_property(wrapped_ctx.implementation(), CONSTRUCTOR_STRING)?;
    //     Ok(constructor)
    // }

    fn equals(self, other: Self, ctx: Self::ContextType) -> bool {
        unsafe { JSValueIsStrictEqual(ctx, self.as_value(), other.as_value()) }
    }

    fn is_instanceof(self, target: Self, ctx: Self::ContextType) -> EsperantoResult<bool> {
        check_jscore_exception!(ctx, exception => {
            unsafe { JSValueIsInstanceOfConstructor(ctx, self.as_value(), target.try_as_object(ctx)?, exception) }
        })
    }

    fn get_native_ref<'a, T: JSExportClass>(
        self,
        ctx: Self::ContextType,
    ) -> EsperantoResult<&'a T> {
        let ptr = unsafe { JSObjectGetPrivate(self.try_as_object(ctx)?) };
        JSExportPrivateData::data_from_ptr(ptr)
    }

    fn delete_property(self, ctx: Self::ContextType, name: &CStr) -> EsperantoResult<bool> {
        let mut name_jsstring = JSCoreString::from(name);
        check_jscore_exception!(ctx, exception => {
            unsafe { JSObjectDeleteProperty(ctx, self.try_as_object(ctx)?, name_jsstring.as_mut(), exception)}
        })
    }

    fn is_object(self, ctx: Self::ContextType) -> bool {
        unsafe { JSValueIsObject(ctx, self.as_value()) }
    }

    fn is_error(self, ctx: Self::ContextType) -> EsperantoResult<bool> {
        let error_name = CString::new("Error")?;
        let error_type = ctx.get_globalobject().get_property(ctx, &error_name)?;
        self.is_instanceof(error_type, ctx)
    }

    fn get_private_data(self, ctx: Self::ContextType) -> EsperantoResult<*mut c_void> {
        let as_obj = self.try_as_object(ctx)?;
        Ok(unsafe { JSObjectGetPrivate(as_obj) })
    }

    fn set_private_data(self, ctx: Self::ContextType, data: *mut c_void) -> EsperantoResult<()> {
        let as_obj = self.try_as_object(ctx)?;
        if unsafe { JSObjectSetPrivate(as_obj, data) } == false {
            return Err(JSValueError::CouldNotStorePrivateData.into());
        }
        Ok(())
    }
}

// impl TryFromInJSContext<i32> for JSCoreValuePointer {
//     fn from_in_context(value: &i32, in_context: &JSCoreContextPointer) -> EsperantoResult<Self> {
//         let ptr = unsafe { JSValueMakeNumber(in_context.into(), *value as f64) };
//         Ok(ptr.into())
//     }
// }

// impl TryFromInJSContext<CString> for JSCoreValuePointer {
//     fn from_in_context(
//         value: &CString,
//         in_context: &JSCoreContextPointer,
//     ) -> EsperantoResult<Self> {
//         let js_string = JSCoreString::from(value);
//         let ptr = unsafe { JSValueMakeString(in_context.into(), js_string.as_ptr()) };
//         Ok(ptr.into())
//     }
// }

// impl TryFromInJSContext<JSCoreValuePointer> for i32 {
//     fn from_in_context(
//         value: &JSCoreValuePointer,
//         in_context: &JSCoreContextPointer,
//     ) -> EsperantoResult<Self> {
//         let num: f64 = check_jscore_exception!(in_context.into(), exception => {
//             unsafe { JSValueToNumber(in_context.into(), value.as_value(), exception) }
//         })?;
//         Ok(num as i32)
//     }
// }
