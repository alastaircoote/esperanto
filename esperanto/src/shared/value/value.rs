use std::{ffi::CString, fmt::Display};

use crate::{
    shared::{
        context::JSContext,
        errors::{ConversionError, EsperantoResult},
    },
    JSExportClass,
};

use crate::shared::engine_impl::JSValueInternalImpl;

use super::value_internal::JSValueInternal;

#[derive(Debug)]
pub struct JSValueRef<'c> {
    pub(crate) internal: JSValueInternalImpl,
    pub(crate) context: &'c JSContext<'c>,
    // _lifetime: PhantomData<&'v ()>,
    // _ctx_lifetime: PhantomData<&'c ()>,
}

impl PartialEq for JSValueRef<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.internal.equals(other.internal, self.context.internal)
    }
}

impl Eq for JSValueRef<'_> {}

impl Display for JSValueRef<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.internal.as_cstring(self.context.internal) {
            Ok(cstring) => {
                let str = cstring.to_string_lossy();
                write!(f, "{}", str)
            }
            Err(err) => {
                write!(f, "[could not represent JSValue: {}]", err)
            }
        }
    }
}

impl<'c> JSValueRef<'c> {
    pub fn set_property(&self, name: &str, value: &Self) -> EsperantoResult<()> {
        let name_cstring =
            CString::new(name).map_err(|e| ConversionError::CouldNotConvertToJSString(e))?;

        JSValueInternalImpl::set_property(
            self.internal,
            self.context.internal,
            &name_cstring,
            value.internal,
        )
    }
    pub fn get_property(&self, name: &str) -> EsperantoResult<Self> {
        let name_cstring =
            CString::new(name).map_err(|e| ConversionError::CouldNotConvertToJSString(e))?;

        self.internal
            .get_property(self.context.internal, &name_cstring)
            .map(|p| JSValueRef {
                internal: p,
                context: self.context,
            })
    }

    pub fn delete_property(&self, name: &str) -> EsperantoResult<()> {
        let name_cstring =
            CString::new(name).map_err(|e| ConversionError::CouldNotConvertToJSString(e))?;

        self.internal
            .delete_property(self.context.internal, &name_cstring)
    }

    pub(crate) fn wrap_internal(
        val: JSValueInternalImpl,
        new_context: &'c JSContext<'c>,
    ) -> JSValueRef<'c> {
        JSValueRef {
            internal: val,
            context: new_context,
            // _ctx_lifetime: PhantomData,
        }
    }

    // pub(crate) fn extend_lifetime<'new>(
    //     &self,
    //     new_context: &'new JSContext<'new>,
    // ) -> Result<JSValueRef<'new>, JSValueError> {
    //     if new_context.internal != self.context.internal {
    //         return Err(JSValueError::CannotUpgradeWithDifferentContext);
    //     }
    //     Ok(JSValueRef {
    //         internal: self.internal,
    //         context: new_context,
    //         // _ctx_lifetime: PhantomData,
    //     })
    // }

    pub fn prototype_for<T>(in_context: &'c JSContext<'c>) -> EsperantoResult<JSValueRef<'c>>
    where
        T: JSExportClass,
    {
        let ptr = JSValueInternalImpl::native_prototype_for::<T>(in_context.internal)?;
        let val = JSValueRef::wrap_internal(ptr, in_context);

        Ok(val)
    }

    pub fn constructor_for<T>(in_context: &'c JSContext<'c>) -> EsperantoResult<JSValueRef<'c>>
    where
        T: JSExportClass,
    {
        return Self::prototype_for::<T>(in_context)?.get_property("constructor");
    }

    pub fn wrap_native<T>(
        instance: T,
        in_context: &'c JSContext<'c>,
    ) -> EsperantoResult<JSValueRef<'c>>
    where
        T: JSExportClass,
    {
        let ptr = JSValueInternalImpl::from_native_class(instance, in_context.internal)?;
        let val = JSValueRef::wrap_internal(ptr, in_context);
        Ok(val)
    }

    pub fn get_native<T: JSExportClass>(
        &self,
        in_context: &'c JSContext<'c>,
    ) -> EsperantoResult<&T> {
        self.internal.get_native_ref(in_context.internal)
    }

    pub fn new_function(
        body: &str,
        arguments: Vec<&str>,
        in_context: &'c JSContext<'c>,
    ) -> EsperantoResult<Self> {
        let body_cstr =
            CString::new(body).map_err(|e| ConversionError::CouldNotConvertToJSString(e))?;

        let arguments_cstr: Vec<CString> = arguments
            .iter()
            .map(|v| CString::new(*v))
            .collect::<Result<Vec<CString>, _>>()?;

        let raw =
            JSValueInternalImpl::new_function(&body_cstr, &arguments_cstr, in_context.internal)?;

        Ok(Self::wrap_internal(raw, in_context))
    }

    pub fn new_error(
        name: &str,
        message: &str,
        in_context: &'c JSContext<'c>,
    ) -> EsperantoResult<Self> {
        let name_cstring = CString::new(name)?;
        let message_cstring = CString::new(message)?;

        let created =
            JSValueInternalImpl::new_error(name_cstring, message_cstring, in_context.internal);

        Ok(Self::wrap_internal(created, in_context))
    }

    pub fn call_as_function(&self, arguments: Vec<&Self>) -> EsperantoResult<Self> {
        return self.call_as_function_bound(arguments, None);
    }

    pub fn call_as_function_bound(
        &self,
        arguments: Vec<&Self>,
        bind_to: Option<&Self>,
    ) -> EsperantoResult<Self> {
        let internal_vec = arguments.iter().map(|a| a.internal).collect();

        let internal_result = self.internal.call_as_function(
            internal_vec,
            bind_to.map(|b| b.internal),
            self.context.internal,
        )?;

        Ok(Self::wrap_internal(internal_result, self.context))
    }

    pub fn call_as_constructor(&self, arguments: Vec<&Self>) -> EsperantoResult<Self> {
        let internal_vec = arguments.iter().map(|a| a.internal).collect();

        let internal_result = self
            .internal
            .call_as_constructor(internal_vec, self.context.internal)?;

        Ok(Self::wrap_internal(internal_result, self.context))
    }

    pub fn undefined(in_context: &'c JSContext) -> Self {
        Self::wrap_internal(
            JSValueInternalImpl::undefined(in_context.internal),
            in_context,
        )
    }

    pub fn is_instance_of(&self, other: &Self) -> EsperantoResult<bool> {
        self.internal
            .is_instanceof(other.internal, self.context.internal)
    }

    pub fn test_norelease(self) -> JSValueInternalImpl {
        let retained = self.internal.retain(self.context.internal);
        retained
    }

    pub fn is_string(&self) -> bool {
        self.internal.is_string(self.context.internal)
    }

    pub fn is_error(&self) -> bool {
        self.internal.is_error(self.context.internal)
    }
}

impl Drop for JSValueRef<'_> {
    fn drop(&mut self) {
        self.internal.release(self.context.internal)
    }
}

// impl<'c> Clone for JSValueRef<'c> {
//     fn clone(&self) -> Self {
//         JSValueRef {
//             context: &self.context,
//             internal: self.internal.retain(&self.context.internal),
//         }
//     }
// }

// impl<'c> Retainable for JSValueRef<'c> {
//     fn retain(&self) -> Self {
//         JSValueRef {
//             internal: self.internal.retain(&self.context.internal),
//             context: self.context,
//             // _ctx_lifetime: PhantomData,
//         }
//     }

//     fn release(&mut self) {
//         self.internal.release(&self.context.internal);
//     }
// }

impl<'c, RawValueType> From<(RawValueType, &'c JSContext<'c>)> for JSValueRef<'c>
where
    RawValueType: Into<JSValueInternalImpl>,
{
    fn from(val: (RawValueType, &'c JSContext)) -> Self {
        JSValueRef {
            internal: val.0.into(),
            context: val.1,
        }
    }
}
