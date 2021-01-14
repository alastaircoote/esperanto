use std::{any::TypeId, cell::RefCell, collections::HashMap, pin::Pin};

use javascriptcore_sys::{
    JSClassCreate, JSClassRelease, JSContextGroupCreate, OpaqueJSClass, OpaqueJSContextGroup,
};

use crate::{
    shared::external_api::runtime::JSRuntimeError, shared::external_api::runtime::Runtime,
    EsperantoResult,
};

use super::{
    jscore_context::JSCoreContext, jscore_context::JSCoreContextPrivate, jscore_export::JSExport,
};
pub struct JSRuntime<'r> {
    pub(super) raw_ref: &'r OpaqueJSContextGroup,
    pub(super) class_defs: RefCell<HashMap<TypeId, *mut OpaqueJSClass>>,
}

pub type JSCoreRuntime<'r> = JSRuntime<'r>;

impl<'r> JSCoreRuntime<'r> {
    pub(super) fn get_class_def<T: JSExport + 'static>(
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

    pub fn create_context_with_global_object<G: JSExport>(
        &'r self,
        global_object: G,
    ) -> EsperantoResult<Pin<Box<JSCoreContext>>> {
        let ctx = JSCoreContext::new_with_global_object(&self, global_object)?;
        Ok(ctx)
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

    fn create_context(&'r self) -> EsperantoResult<Pin<Box<JSCoreContext>>> {
        JSCoreContext::new(&self)
    }
}
