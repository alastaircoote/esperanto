use std::{ffi::CString, fmt::Display};

use crate::{
    export::Js,
    shared::{
        context::JSContext,
        errors::{ConversionError, EsperantoResult},
        retain::Retainable,
    },
    JSExportClass, Retain, TryJSValueFrom,
};

use crate::shared::engine_impl::JSValueInternalImpl;

use super::{value_internal::JSValueInternal, TryConvertJSValue};

#[derive(Debug)]
pub struct JSValue<'c> {
    pub(crate) internal: JSValueInternalImpl,
    pub(crate) context: &'c JSContext<'c>,
}

impl PartialEq for JSValue<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.internal.equals(other.internal, self.context.internal)
    }
}

impl Eq for JSValue<'_> {}

pub(crate) type ValueResult<'c> = EsperantoResult<Retain<JSValue<'c>>>;

impl Display for JSValue<'_> {
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

impl<'c> JSValue<'c> {
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
    pub fn get_property(&self, name: &str) -> ValueResult<'c> {
        let name_cstring =
            CString::new(name).map_err(|e| ConversionError::CouldNotConvertToJSString(e))?;

        let val = self
            .internal
            .get_property(self.context.internal, &name_cstring)
            .map(|p| JSValue {
                internal: p,
                context: self.context,
            })?;

        return Ok(Retain::wrap(val));
    }

    pub fn delete_property(&self, name: &str) -> EsperantoResult<bool> {
        let name_cstring =
            CString::new(name).map_err(|e| ConversionError::CouldNotConvertToJSString(e))?;

        self.internal
            .delete_property(self.context.internal, &name_cstring)
    }

    pub(crate) fn wrap_internal(
        val: JSValueInternalImpl,
        new_context: &'c JSContext<'c>,
    ) -> JSValue<'c> {
        JSValue {
            internal: val,
            context: new_context,
            // _ctx_lifetime: PhantomData,
        }
    }

    pub fn prototype_for<T>(in_context: &'c JSContext<'c>) -> ValueResult
    where
        T: JSExportClass,
    {
        let ptr = JSValueInternalImpl::native_prototype_for::<T>(
            in_context.internal,
            in_context.get_runtime().internal,
        )?;
        let val = JSValue::wrap_internal(ptr, in_context);

        Ok(Retain::wrap(val))
    }

    pub fn constructor_for<T>(in_context: &'c JSContext<'c>) -> ValueResult
    where
        T: JSExportClass,
    {
        let ptr = JSValueInternalImpl::constructor_for::<T>(
            in_context.internal,
            in_context.get_runtime().internal,
        )?;
        let val = JSValue::wrap_internal(ptr, in_context);

        Ok(Retain::wrap(val))
    }

    pub fn new_wrapped_native<T>(
        instance: T,
        in_context: &'c JSContext<'c>,
    ) -> EsperantoResult<Retain<JSValue<'c>>>
    where
        T: JSExportClass,
    {
        let ptr = JSValueInternalImpl::from_native_class(
            instance,
            in_context.internal,
            in_context.get_runtime().internal,
        )?;
        let val = JSValue::wrap_internal(ptr, in_context);
        Ok(Retain::wrap(val))
    }

    pub fn as_native<T: JSExportClass>(&self) -> EsperantoResult<Js<'c, T>> {
        // self.internal.get_native_ref(self.context.internal)
        let retained = self.retain();
        Js::new(retained)
    }

    pub fn new_function(
        body: &str,
        arguments: Vec<&str>,
        in_context: &'c JSContext<'c>,
    ) -> ValueResult<'c> {
        let body_cstr =
            CString::new(body).map_err(|e| ConversionError::CouldNotConvertToJSString(e))?;

        let arguments_cstr: Vec<CString> = arguments
            .iter()
            .map(|v| CString::new(*v))
            .collect::<Result<Vec<CString>, _>>()?;

        let raw =
            JSValueInternalImpl::new_function(&body_cstr, &arguments_cstr, in_context.internal)?;

        Ok(Retain::wrap(Self::wrap_internal(raw, in_context)))
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

    pub fn call_as_function(&self, arguments: Vec<&Self>) -> ValueResult<'c> {
        return self.call_as_function_bound(arguments, None);
    }

    pub fn call_as_function_bound(
        &self,
        arguments: Vec<&Self>,
        bind_to: Option<&Self>,
    ) -> ValueResult<'c> {
        let internal_vec = arguments.iter().map(|a| a.internal).collect();

        let internal_result = self.internal.call_as_function(
            internal_vec,
            bind_to.map(|b| b.internal),
            self.context.internal,
        )?;

        Ok(Retain::wrap(Self::wrap_internal(
            internal_result,
            self.context,
        )))
    }

    pub fn call_as_constructor(&self, arguments: Vec<&Self>) -> ValueResult {
        let internal_vec = arguments.iter().map(|a| a.internal).collect();

        let internal_result = self
            .internal
            .call_as_constructor(internal_vec, self.context.internal)?;

        Ok(Retain::wrap(Self::wrap_internal(
            internal_result,
            self.context,
        )))
    }

    // Seems kind of silly for undefined to be a retain since it never gets garbage collected
    // but it's actually difficult for us to provide anything different. Will have to think on it.
    pub fn undefined(in_context: &'c JSContext) -> Retain<Self> {
        Retain::wrap(Self::wrap_internal(
            JSValueInternalImpl::undefined(in_context.internal),
            in_context,
        ))
    }

    pub fn is_instance_of(&self, other: &Self) -> EsperantoResult<bool> {
        self.internal
            .is_instanceof(other.internal, self.context.internal)
    }

    pub fn is_string(&self) -> bool {
        self.internal.is_string(self.context.internal)
    }

    pub fn is_error(&self) -> EsperantoResult<bool> {
        self.internal.is_error(self.context.internal)
    }

    pub fn try_convert<'a, T>(&self) -> EsperantoResult<T>
    where
        T: TryConvertJSValue<'c>,
        'c: 'a,
    {
        T::try_from_jsvalue(&self)
    }

    pub fn try_new_from<T>(value: T, in_context: &'c JSContext<'c>) -> ValueResult
    where
        T: TryJSValueFrom<'c>,
    {
        return T::try_jsvalue_from(value, in_context);
    }

    pub fn retain(&self) -> Retain<Self> {
        let new_retained = self.internal.retain(self.context.internal);
        return Retain::wrap(Self::wrap_internal(new_retained, self.context));
    }
}

impl<'c, RawValueType> From<(RawValueType, &'c JSContext<'c>)> for JSValue<'c>
where
    RawValueType: Into<JSValueInternalImpl>,
{
    fn from(val: (RawValueType, &'c JSContext)) -> Self {
        JSValue {
            internal: val.0.into(),
            context: val.1,
        }
    }
}

// impl<'c> AsRef<JSValue<'c>> for JSValue<'c> {
//     fn as_ref(&self) -> &JSValue<'c> {
//         return &self;
//     }
// }
