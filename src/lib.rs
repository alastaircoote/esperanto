mod shared;

#[cfg(feature = "engine-javascriptcore-sys")]
#[macro_use]
mod jscore;

#[cfg(feature = "engine-javascriptcore-sys")]
mod impls {
    pub use crate::jscore::jscore_context::JSCoreContext as JSContext;
    pub use crate::jscore::jscore_runtime::JSCoreRuntime as JSRuntime;
    pub use crate::jscore::jscore_value::JSCoreValue as JSValue;
}

pub mod export {
    #[cfg(feature = "engine-javascriptcore-sys")]
    pub use crate::jscore::jscore_export::HasJSExportDefinition;
}

pub mod private {
    #[cfg(feature = "engine-javascriptcore-sys")]
    pub use crate::jscore::*;
}

pub mod traits {
    pub use crate::shared::traits::jscontext::JSContext;
    pub use crate::shared::traits::jsruntime::JSRuntime;
    pub use crate::shared::traits::jsvalue::JSValue;
    pub use crate::shared::traits::tryas::{TryAs, TryAsRef};
}

pub use crate::shared::errors::EsperantoResult;
pub mod errors {
    pub use crate::shared::errors::*;
    pub use crate::shared::export::export_error::*;
}
