use std::{any::TypeId, cell::RefCell, collections::HashMap, pin::Pin};

use javascriptcore_sys::{
    JSClassCreate, JSClassRelease, JSContextGroupCreate, JSContextGroupRelease, OpaqueJSClass,
    OpaqueJSContextGroup,
};

use crate::{
    shared::external_api::runtime::JSRuntimeError, shared::external_api::runtime::Runtime,
    EsperantoResult,
};

use super::{jscore_context::JSCoreContext, jscore_export::JSCoreExport};
use crate::jscontext::Context;
pub struct JSRuntime<'r> {
    pub(super) raw_ref: &'r OpaqueJSContextGroup,
    pub(super) class_defs: RefCell<HashMap<TypeId, *mut OpaqueJSClass>>,
}

impl<'c> Drop for JSCoreRuntime<'c> {
    fn drop(&mut self) {
        unsafe { JSContextGroupRelease(self.raw_ref) }
    }
}

pub type JSCoreRuntime<'r> = JSRuntime<'r>;

impl<'r> JSCoreRuntime<'r> {
    pub(super) fn get_class_def<T: JSCoreExport + 'static>(
        &self,
    ) -> EsperantoResult<&mut OpaqueJSClass> {
        let type_id = TypeId::of::<T>();

        if let Some(existing) = self.class_defs.borrow().get(&type_id) {
            return unsafe { Ok((*existing).as_mut().unwrap()) };
        }
        let def = T::get_definition();
        let class = unsafe { JSClassCreate(def) };
        match unsafe { class.as_mut() } {
            Some(r) => Ok(r),
            None => Err(JSRuntimeError::CouldNotCreateClass.into()),
        }
    }
}

impl<'r> Runtime<'r> for JSCoreRuntime<'r> {
    type Context = JSCoreContext<'r>;

    fn new() -> EsperantoResult<Self> {
        let raw_ref = unsafe { JSContextGroupCreate() };
        match unsafe { raw_ref.as_ref() } {
            Some(r) => Ok(JSRuntime {
                raw_ref: r,
                class_defs: RefCell::new(HashMap::new()),
            }),
            None => Err(JSRuntimeError::CouldNotCreateRuntime.into()),
        }
    }
}
