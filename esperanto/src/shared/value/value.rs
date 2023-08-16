use std::{ffi::CString, fmt::Display};

use crate::{
    export::Js,
    shared::{
        context::JSContext,
        errors::{ConversionError, EsperantoResult},
    },
    JSExportClass, Retain, TryJSValueFrom,
};

use crate::shared::engine_impl::JSValueInternalImpl;

use super::{value_implementation::JSValueImplementation, TryConvertJSValue};

#[derive(Debug, Eq)]
pub struct JSValue<'r, 'c> {
    pub(crate) internal: JSValueInternalImpl,
    pub(crate) context: &'c JSContext<'r, 'c>,
}

impl PartialEq for JSValue<'_, '_> {
    fn eq(&self, other: &Self) -> bool {
        self.internal
            .equals(other.internal, self.context.implementation())
    }
}

pub(crate) type ValueResult<'r, 'c> = EsperantoResult<Retain<JSValue<'r, 'c>>>;

impl Display for JSValue<'_, '_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.internal.as_cstring(self.context.implementation()) {
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

impl<'r, 'c> JSValue<'r, 'c>
where
    'r: 'c,
{
    pub fn set_property(&self, name: &str, value: &Self) -> EsperantoResult<()> {
        let name_cstring =
            CString::new(name).map_err(|e| ConversionError::CouldNotConvertToJSString(e))?;

        JSValueInternalImpl::set_property(
            self.internal,
            self.context.implementation(),
            &name_cstring,
            value.internal,
        )
    }
    pub fn get_property(&self, name: &str) -> ValueResult<'r, 'c> {
        let name_cstring =
            CString::new(name).map_err(|e| ConversionError::CouldNotConvertToJSString(e))?;

        let val = self
            .internal
            .get_property(self.context.implementation(), &name_cstring)
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
            .delete_property(self.context.implementation(), &name_cstring)
    }

    pub(crate) fn wrap_internal(
        val: JSValueInternalImpl,
        new_context: &'c JSContext<'r, 'c>,
    ) -> JSValue<'r, 'c> {
        JSValue {
            internal: val,
            context: new_context,
            // _ctx_lifetime: PhantomData,
        }
    }

    pub fn prototype_for<T>(in_context: &'c JSContext<'r, 'c>) -> ValueResult<'r, 'c>
    where
        T: JSExportClass,
    {
        let runtime = in_context.get_runtime();
        let val = JSValueInternalImpl::native_prototype_for::<T>(
            in_context.implementation(),
            runtime.implementation(),
        )?;

        Ok(Retain::wrap(Self::wrap_internal(val, in_context)))
    }

    pub fn constructor_for<T>(in_context: &'c JSContext<'r, 'c>) -> ValueResult<'r, 'c>
    where
        T: JSExportClass,
    {
        Self::prototype_for::<T>(in_context)?.get_property("constructor")
    }

    pub fn new_wrapped_native<T>(
        instance: T,
        in_context: &'c JSContext<'r, 'c>,
    ) -> EsperantoResult<Retain<JSValue<'r, 'c>>>
    where
        T: JSExportClass,
    {
        let runtime = in_context.get_runtime();
        let ptr = JSValueInternalImpl::from_native_class(
            instance,
            in_context.implementation(),
            runtime.implementation(),
        )?;
        let val = JSValue::wrap_internal(ptr, in_context);
        Ok(Retain::wrap(val))
    }

    pub fn as_native<T: JSExportClass>(&self) -> EsperantoResult<Js<'r, 'c, T>> {
        // self.internal.get_native_ref(self.context.internal)
        let retained = self.retain();
        Js::new(retained)
    }

    pub fn new_function(
        body: &str,
        arguments: Vec<&str>,
        in_context: &'c JSContext<'r, 'c>,
    ) -> ValueResult<'r, 'c> {
        let body_cstr =
            CString::new(body).map_err(|e| ConversionError::CouldNotConvertToJSString(e))?;

        let arguments_cstr: Vec<CString> = arguments
            .iter()
            .map(|v| CString::new(*v))
            .collect::<Result<Vec<CString>, _>>()?;

        let raw = JSValueInternalImpl::new_function(
            &body_cstr,
            &arguments_cstr,
            in_context.implementation(),
        )?;

        Ok(Retain::wrap(Self::wrap_internal(raw, in_context)))
    }

    pub fn new_error(
        name: &str,
        message: &str,
        in_context: &'c JSContext<'r, 'c>,
    ) -> EsperantoResult<Self> {
        let name_cstring = CString::new(name)?;
        let message_cstring = CString::new(message)?;

        let created = JSValueInternalImpl::new_error(
            name_cstring,
            message_cstring,
            in_context.implementation(),
        );

        Ok(Self::wrap_internal(created, in_context))
    }

    pub fn call_as_function(&self, arguments: Vec<&Self>) -> ValueResult<'r, 'c> {
        return self.call_as_function_bound(arguments, None);
    }

    pub fn call_as_function_bound(
        &self,
        arguments: Vec<&Self>,
        bind_to: Option<&Self>,
    ) -> ValueResult<'r, 'c> {
        let internal_vec = arguments.iter().map(|a| a.internal).collect();

        let internal_result = self.internal.call_as_function(
            internal_vec,
            bind_to.map(|b| b.internal),
            self.context.implementation(),
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
            .call_as_constructor(internal_vec, self.context.implementation())?;

        Ok(Retain::wrap(Self::wrap_internal(
            internal_result,
            self.context,
        )))
    }

    // Seems kind of silly for undefined to be a retain since it never gets garbage collected
    // but it's actually difficult for us to provide anything different. Will have to think on it.
    pub fn undefined(in_context: &'c JSContext<'r, 'c>) -> Retain<Self> {
        Retain::wrap(Self::wrap_internal(
            JSValueInternalImpl::undefined(in_context.implementation()),
            in_context,
        ))
    }

    pub fn is_instance_of(&self, other: &Self) -> EsperantoResult<bool> {
        self.internal
            .is_instanceof(other.internal, self.context.implementation())
    }

    pub fn is_string(&self) -> bool {
        self.internal.is_string(self.context.implementation())
    }

    pub fn is_error(&self) -> EsperantoResult<bool> {
        self.internal.is_error(self.context.implementation())
    }

    pub fn try_convert<'a, T>(&self) -> EsperantoResult<T>
    where
        T: TryConvertJSValue<'r, 'c>,
        'c: 'a,
    {
        T::try_from_jsvalue(&self)
    }

    pub fn try_new_from<T>(value: T, in_context: &'c JSContext<'r, 'c>) -> ValueResult<'r, 'c>
    where
        T: TryJSValueFrom<'r, 'c>,
    {
        return T::try_jsvalue_from(value, in_context);
    }

    pub fn retain(&self) -> Retain<Self> {
        let new_retained = self.internal.retain(self.context.implementation());
        return Retain::wrap(Self::wrap_internal(new_retained, self.context));
    }

    pub fn transfer_to_context<'n>(
        &self,
        new_context: &'n JSContext<'r, 'n>,
    ) -> Retain<JSValue<'r, 'n>>
    where
        'r: 'n,
    {
        let new_retain = self.internal.retain(new_context.implementation());
        Retain::wrap(JSValue::<'r, 'n>::wrap_internal(new_retain, new_context))
    }

    // pub fn transfer<'a, 'o>(
    //     value: &'a JSValue<'r, 'o>,
    //     to_context: &'c JSContext<'r, 'c>,
    // ) -> Retain<Self> {
    //     let new_retain = value.internal.retain(to_context.implementation());
    //     Retain::wrap(JSValue::wrap_internal(new_retain, &to_context))
    // }
}

impl<'r, 'c, RawValueType> From<(RawValueType, &'c JSContext<'r, 'c>)> for JSValue<'r, 'c>
where
    RawValueType: Into<JSValueInternalImpl>,
    'c: 'r,
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
