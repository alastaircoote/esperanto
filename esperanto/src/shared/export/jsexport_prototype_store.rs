// use crate::{
//     shared::{
//         engine_impl::{JSRuntimeInternalImpl, JSValueInternalImpl},
//         errors::JSExportError,
//     },
//     EsperantoError, EsperantoResult, JSExportClass,
// };
// use javascriptcore_sys::JSClassRef;
// use lazy_static::lazy_static;
// use std::{any::TypeId, collections::HashMap, sync::RwLock};
// type JSClassIdentifierImpl = JSClassRef;

// /// Since we can't pass JSValues between runtimes we need to make separate
// /// prototypes per runtime. This struct lets us store in a hashmap accordingly:
// #[derive(Eq, PartialEq, Hash, Copy, Clone)]
// struct JSExportPrototypeStoreKey {
//     runtime: JSRuntimeInternalImpl,
//     type_id: TypeId,
// }

// #[derive(Eq, PartialEq, Hash, Copy, Clone)]
// pub(crate) struct JSExportStoredClassInfo {
//     pub(crate) prototype: JSValueInternalImpl,
//     pub(crate) instance_class: JSClassIdentifierImpl,
// }

// /// JavaScriptCore docs state that the API is multithread-safe:
// /// https://developer.apple.com/documentation/javascriptcore/jsvirtualmachine#
// ///
// /// And QuickJS discussion appears to suggest so (I think I saw something more
// /// authoriative at one point):
// /// https://www.freelists.org/post/quickjs-devel/Usage-of-QuickJS-in-multithreaded-environments,1
// ///
// /// so we're OK to impl Sync and Send here. But if we ever add a third JS engine
// /// we might need to think very hard about this.
// unsafe impl Sync for JSExportPrototypeStoreKey {}
// unsafe impl Send for JSExportPrototypeStoreKey {}
// unsafe impl Sync for JSExportStoredClassInfo {}
// unsafe impl Send for JSExportStoredClassInfo {}

// type PrototypeStore = RwLock<HashMap<JSExportPrototypeStoreKey, JSExportStoredClassInfo>>;

// lazy_static! {
//     static ref JS_PROTOTYPES: PrototypeStore = RwLock::new(HashMap::new());
// }

// pub(crate) fn get_stored_prototype_info<T: JSExportClass>(
//     for_runtime: JSRuntimeInternalImpl,
// ) -> EsperantoResult<Option<JSExportStoredClassInfo>> {
//     let key = JSExportPrototypeStoreKey {
//         runtime: for_runtime,
//         type_id: TypeId::of::<T>(),
//     };

//     JS_PROTOTYPES
//         .read()
//         .or(Err(EsperantoError::ExportError(
//             JSExportError::UnexpectedBehaviour,
//         )))
//         .map(|hashmap| hashmap.get(&key).map(|v| v.clone()))
// }

// pub(crate) fn store_prototype_info<T: JSExportClass>(
//     prototype: JSValueInternalImpl,
//     instance_class: JSClassIdentifierImpl,
//     for_runtime: JSRuntimeInternalImpl,
// ) -> EsperantoResult<JSExportStoredClassInfo> {
//     let key = JSExportPrototypeStoreKey {
//         runtime: for_runtime,
//         type_id: TypeId::of::<T>(),
//     };

//     let mut store = JS_PROTOTYPES.write().or(Err(EsperantoError::ExportError(
//         JSExportError::UnexpectedBehaviour,
//     )))?;

//     let info = JSExportStoredClassInfo {
//         prototype,
//         instance_class,
//     };

//     store.insert(key, info.clone());
//     Ok(info)
// }
