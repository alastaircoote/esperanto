use std::{any::TypeId, cell::RefCell, collections::HashMap, hash::Hash, iter::Map};

use super::{
    quickjs_prototype_storage::attach_classid_storage, quickjsexport::QuickJSExportExtensions,
};
use by_address::ByAddress;
use quickjs_android_suitable_sys::{
    JSRuntime as QuickJSRuntime, JS_FreeRuntime, JS_GetRuntimeOpaque, JS_NewClass, JS_NewClassID,
    JS_NewRuntime,
};

use crate::{
    quickjs::quickjs_prototype_storage::drop_classid_storage,
    shared::{
        errors::EsperantoResult,
        runtime::{JSRuntimeError, JSRuntimeInternal},
    },
    JSExportClass,
};

pub type QuickJSRuntimeInternal = *mut QuickJSRuntime;

type ClassIDStore = HashMap<TypeId, u32>;

// pub(super) fn get_class_id_for_type<T: JSExportClass>(
//     runtime: QuickJSRuntimeInternal,
// ) -> EsperantoResult<u32> {
//     let addr = ByAddress(&T::METADATA);

//     let class_store_raw = unsafe { JS_GetRuntimeOpaque(runtime) } as *mut ClassIDStore;

//     let class_store = match unsafe { class_store_raw.as_mut() } {
//         Some(store) => store,
//         None => return Err(JSRuntimeError::FailedToRetrievePrivateContext.into()),
//     };

//     if let Some(existing_class_id) = class_store.get(&addr) {
//         return Ok(*existing_class_id);
//     }

//     let mut cid: u32 = 0;
//     let definition = T::class_def();
//     let class_id = unsafe { JS_NewClassID(&mut cid) };
//     unsafe { JS_NewClass(runtime, class_id, &definition) };
//     class_store.insert(addr, class_id);
//     return Ok(class_id);
// }

impl JSRuntimeInternal for QuickJSRuntimeInternal {
    fn new() -> Result<Self, JSRuntimeError> {
        let runtime = unsafe { JS_NewRuntime() };
        if runtime.is_null() {
            return Err(JSRuntimeError::CouldNotCreateRuntime);
        }
        attach_classid_storage(runtime);
        Ok(runtime)
    }

    fn release(self) {
        println!("FREE RUNTIME");

        unsafe { JS_FreeRuntime(self) }
        drop_classid_storage(self);
    }
}
