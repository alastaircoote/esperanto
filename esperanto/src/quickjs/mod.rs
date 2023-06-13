#[macro_use]
mod exception;
mod quickjscontext;
// mod quickjscstr;
#[macro_use]
mod quickjsexport;
mod quickjs_prototype_storage;
mod quickjscontextpointer;
mod quickjsruntime;
mod quickjsvalue;

pub(crate) use quickjscontext::QuickJSContextInternal as JSContextInternalImpl;
pub(crate) use quickjsruntime::QuickJSRuntimeInternal as JSRuntimeInternalImpl;
pub(crate) use quickjsvalue::QuickJSValueInternal as JSValueInternalImpl;

// pub mod export {
//     pub use super::quickjsexport::QuickJSCallAsConstructor as JSCallAsConstructorImpl;
//     pub use super::quickjsexport::QuickJSCallAsFunction as JSCallAsFunctionImpl;
// }
