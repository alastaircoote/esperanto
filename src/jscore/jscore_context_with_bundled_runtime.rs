// use crate::{shared::traits::jsruntime::JSRuntimeHasContext, traits::JSRuntime, EsperantoResult};

// use super::{jscore_context::JSCoreContext, jscore_runtime::JSCoreRuntime};

// struct JSCoreContextWithBundledRuntime<'c> {
//     context: JSCoreContext<'c>,
//     runtime: JSCoreRuntime<'c>,
// }

// impl<'c> JSCoreContextWithBundledRuntime<'c> {
//     fn new() -> EsperantoResult<Self> {
//         let rt = JSCoreRuntime::new()?;
//         let ctx = rt.create_context()?;
//         Ok(JSCoreContextWithBundledRuntime {
//             runtime: rt,
//             context: ctx,
//         })
//     }
// }
