mod shared;

#[cfg(feature = "javascriptcore")]
mod jscore;

#[cfg(feature = "quickjs")]
mod quickjs;

pub use shared::context::JSContext;
pub use shared::errors::EsperantoError;
pub use shared::export::JSExportClass;
pub use shared::runtime::JSRuntime;
pub use shared::value::{AsJSValueRef, JSValue, JSValueFrom, TryJSValueFrom};

pub mod errors {
    pub use super::shared::errors::CatchExceptionError;
    pub use super::shared::value::JSValueError;
}

pub mod export {
    pub use super::shared::export::*;
}

pub mod private {
    #[cfg(feature = "javascriptcore")]
    pub use javascriptcore_sys::JSClassDefinition;

    #[cfg(feature = "quickjs")]
    pub use quickjs_android_suitable_sys::JSClassDef as JSClassDefinition;
}
