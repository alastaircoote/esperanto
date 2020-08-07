#[cfg(feature = "javascriptcore")]
pub use esperanto_javascriptcore::{JSCGlobalContext as Context, JSCValue as Value};

#[cfg(feature = "quickjs")]
pub use esperanto_quickjs::{QJSContext as Context, QJSValue as Value};
