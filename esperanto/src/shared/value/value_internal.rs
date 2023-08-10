use std::ffi::{CStr, CString};

use crate::shared::context::JSContextInternal;
use crate::shared::errors::{EsperantoError, EsperantoResult, JavaScriptError};
use crate::JSExportClass;

pub(crate) trait JSValueInternal: Sized + Copy {
    type ContextType: JSContextInternal + Copy;

    fn as_cstring(self, ctx: Self::ContextType) -> EsperantoResult<CString>;
    fn from_cstring(value: &CString, ctx: Self::ContextType) -> Self;
    fn is_string(self, ctx: Self::ContextType) -> bool;

    fn as_number(self, ctx: Self::ContextType) -> EsperantoResult<f64>;
    fn from_number(number: f64, ctx: Self::ContextType) -> EsperantoResult<Self>;

    fn as_bool(self, ctx: Self::ContextType) -> EsperantoResult<bool>;
    fn from_bool(bool: bool, ctx: Self::ContextType) -> EsperantoResult<Self>;

    fn new_error(name: CString, message: CString, ctx: Self::ContextType) -> Self;
    fn is_error(self, ctx: Self::ContextType) -> EsperantoResult<bool>;

    fn to_js_error(self, ctx: Self::ContextType) -> EsperantoResult<JavaScriptError> {
        const NAME_PROP_STR: &[u8] = b"name\0";
        const MESSAGE_PROP_STR: &[u8] = b"message\0";

        let name =
            unsafe { self.get_property(ctx, CStr::from_ptr(NAME_PROP_STR.as_ptr() as *const i8))? };
        let message = unsafe {
            self.get_property(ctx, CStr::from_ptr(MESSAGE_PROP_STR.as_ptr() as *const i8))?
        };

        let name_str = name.as_cstring(ctx)?;
        let message_str = message.as_cstring(ctx)?;

        name.release(ctx);
        message.release(ctx);

        Ok(JavaScriptError::new(
            name_str.to_string_lossy().into_owned(),
            message_str.to_string_lossy().into_owned(),
        ))
    }

    fn undefined(ctx: Self::ContextType) -> Self;

    fn native_prototype_for<T: JSExportClass>(ctx: Self::ContextType) -> EsperantoResult<Self>;

    fn constructor_for<T: JSExportClass>(ctx: Self::ContextType) -> EsperantoResult<Self>;

    fn from_native_class<T: JSExportClass>(
        instance: T,
        ctx: Self::ContextType,
    ) -> EsperantoResult<Self>;

    fn get_native_ref<'a, T: JSExportClass>(self, ctx: Self::ContextType)
        -> EsperantoResult<&'a T>;

    fn release(self, ctx: Self::ContextType);

    #[must_use]
    fn retain(self, ctx: Self::ContextType) -> Self;

    fn set_property(
        self,
        ctx: Self::ContextType,
        name: &CString,
        new_value: Self,
    ) -> Result<(), EsperantoError>;

    fn get_property(self, ctx: Self::ContextType, name: &CStr) -> Result<Self, EsperantoError>;

    fn delete_property(self, ctx: Self::ContextType, name: &CStr) -> EsperantoResult<bool>;

    fn new_function(
        function_text: &CString,
        argument_names: &Vec<CString>,
        ctx: Self::ContextType,
    ) -> EsperantoResult<Self>;

    fn call_as_function(
        self,
        arguments: Vec<Self>,
        bound_to: Option<Self>,
        ctx: Self::ContextType,
    ) -> EsperantoResult<Self>;

    fn call_as_constructor(
        self,
        arguments: Vec<Self>,
        ctx: Self::ContextType,
    ) -> EsperantoResult<Self>;

    fn equals(self, other: Self, ctx: Self::ContextType) -> bool;
    fn is_instanceof(self, target: Self, ctx: Self::ContextType) -> EsperantoResult<bool>;
    fn is_object(self, ctx: Self::ContextType) -> bool;
}
