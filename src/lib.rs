mod shared;

#[cfg(feature = "engine-javascriptcore-sys")]
#[macro_use]
mod jscore;

#[cfg(feature = "engine-javascriptcore-sys")]
pub mod jscontext {
    pub use crate::jscore::jscore_context::JSContext;
    pub use crate::shared::external_api::context::Context;
}
#[cfg(feature = "engine-javascriptcore-sys")]
pub mod jsruntime {
    pub use crate::jscore::jscore_runtime::JSRuntime;
    pub use crate::shared::external_api::runtime::Runtime;
}
#[cfg(feature = "engine-javascriptcore-sys")]
pub mod jsvalue {
    pub use crate::jscore::jscore_value::JSValue;
    pub use crate::shared::external_api::value::Value;
}

pub mod private {
    #[cfg(feature = "engine-javascriptcore-sys")]
    pub use crate::jscore::*;
    #[cfg(feature = "engine-javascriptcore-sys")]
    pub mod javascriptcore_sys {
        pub use javascriptcore_sys::*;
    }
}
#[cfg(feature = "engine-javascriptcore-sys")]
pub use crate::jscore::jscore_export::JSCoreExport as JSExport;

// pub mod traits {
//     // pub use crate::shared::traits::jscontext::Context;
//     // pub use crate::shared::traits::jsruntime::Runtime;
//     // pub use crate::shared::traits::jsvalue::Value;
//     pub use crate::shared::traits::tryas::{TryAs, TryAsRef};
// }
pub use crate::shared::external_api::esperanto_error::EsperantoError;
pub type EsperantoResult<T> = Result<T, EsperantoError>;
pub mod convert {
    pub use crate::shared::external_api::convert::*;
}

pub mod errors {
    pub use crate::shared::errors::*;
    pub use crate::shared::export::export_error::*;
}
