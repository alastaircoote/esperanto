mod shared;

#[cfg(feature = "javascriptcore")]
mod jscore;

#[cfg(feature = "quickjs")]
mod quickjs;

pub use shared::context::{EvaluateMetadata, JSContext};
pub use shared::errors::{EsperantoError, EsperantoResult};
pub use shared::export::JSExportClass;
pub use shared::retain::Retain;
pub use shared::runtime::JSRuntime;
pub use shared::value::{AsJSValueRef, JSValue, JSValueFrom, TryConvertJSValue, TryJSValueFrom};

pub mod errors {
    pub use super::shared::errors::*;
    pub use super::shared::value::JSValueError;
}

pub mod export {
    pub use super::shared::export::*;
}
