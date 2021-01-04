use std::{any::TypeId, cell::RefCell, collections::HashMap, pin::Pin};

use javascriptcore_sys::{
    JSClassCreate, JSClassDefinition, JSContextGroupCreate, JSGlobalContextCreateInGroup,
    OpaqueJSClass, OpaqueJSContextGroup,
};

use crate::{shared::traits::jsruntime::JSRuntimeError, traits::JSRuntime, EsperantoResult};

use super::{
    jscore_context::{JSCoreContext, JSCoreContextPrivate},
    jscore_export::JSExport,
};
pub struct JSCoreRuntime<'r> {
    pub(super) raw_ref: &'r OpaqueJSContextGroup,
    pub(super) class_defs: RefCell<HashMap<TypeId, &'r OpaqueJSClass>>,
}

impl<'r> JSCoreRuntime<'r> {
    fn get_class_def<T: JSExport + 'static>(&self) -> EsperantoResult<&'r OpaqueJSClass> {
        let type_id = TypeId::of::<T>();
        if let Some(existing) = self.class_defs.borrow().get(&type_id) {
            return Ok(existing);
        }
        let def = T::get_definition();
        let class = unsafe { JSClassCreate(&def) };
        match unsafe { class.as_ref() } {
            Some(r) => Ok(r),
            None => Err(JSRuntimeError::CouldNotCreateClass.into()),
        }
    }

    pub fn create_context_with_global_object<G: JSExport>(
        &'r self,
    ) -> EsperantoResult<Pin<Box<JSCoreContext>>> {
        let def = self.get_class_def::<G>()?;
        JSCoreContext::new_in_context_group(self, Some(def))
    }
}

impl<'r> JSRuntime<'r> for JSCoreRuntime<'r> {
    type Context = JSCoreContext<'r>;
    fn new() -> EsperantoResult<Self> {
        let raw_ref = unsafe { JSContextGroupCreate() };
        match unsafe { raw_ref.as_ref() } {
            Some(r) => Ok(JSCoreRuntime {
                raw_ref: r,
                class_defs: RefCell::new(HashMap::new()),
            }),
            None => Err(JSRuntimeError::CouldNotCreateRuntime.into()),
        }
    }

    fn create_context(&'r self) -> EsperantoResult<Pin<Box<Self::Context>>> {
        JSCoreContext::new_in_context_group(self, None)
    }
}

// impl<'r, 'c> JSRuntimeHasContext<'r, 'c> for JSCoreRuntime<'r>
// where
//     'r: 'c,
// {
//     type Context = JSCoreContext<'c>;

//     fn create_context(&'r self) -> crate::errors::EsperantoResult<Pin<Box<Self::Context>>> {
//         JSCoreContext::new_in_context_group(self)
//     }
// }
