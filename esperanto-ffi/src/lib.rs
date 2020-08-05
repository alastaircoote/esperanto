pub mod context;
pub mod implementation;
pub mod shared;
mod util;
pub mod value;
// pub use implementation::Value;

// This is really dumb but cbindgen doesn't work with aliases like we're doing with
// JSCValue and QJSValue. But because the FFI side doesn't actually need to know
// anything about these structs we'll create a dummy struct, just so the header works.
pub struct Value {}
