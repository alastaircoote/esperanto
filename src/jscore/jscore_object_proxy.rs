use std::{
    cell::{BorrowError, BorrowMutError, Cell, RefCell},
    convert::TryInto,
    ops::Deref,
};

use super::{
    jscore_context::JSCoreContext,
    jscore_export::JSCoreExport,
    jscore_value::{JSCoreValue, JSValue},
};
use crate::{
    jscontext::Context, shared::external_api::esperanto_error::EngineError, EsperantoResult,
};
use javascriptcore_sys::{JSObjectMake, OpaqueJSValue};
use thiserror::Error;

// enum JSObjectProxyStorage<'c, T> {
//     NotInJSContext(T),
//     InJSContext(JSValue<'c>, &'c T),
// }
#[derive(Debug, Error)]
enum JSObjectProxyError {
    #[error("Could not create a reference to the object you've provided")]
    CouldNotCreateReference,
    #[error("Could not borrow the JSValue reference")]
    CouldNotBorrowReference(#[from] BorrowMutError),
}

impl EngineError for JSObjectProxyError {}

pub struct Js<'c, T: JSCoreExport> {
    // storage: RefCell<JSObjectProxyStorage<'c, T>>,
    reference: &'c T,
    js_value: RefCell<Option<JSValue<'c>>>,
}

impl<'c, T: JSCoreExport> Deref for Js<'c, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.reference
    }
}

impl<'c, T: JSCoreExport> Js<'c, T> {
    pub fn new(wrapping_obj: T) -> EsperantoResult<Self> {
        let boxed = Box::new(wrapping_obj);
        let ptr = Box::into_raw(boxed);
        match unsafe { ptr.as_ref() } {
            Some(reference) => Ok(Js {
                reference,
                js_value: RefCell::new(None),
            }),
            None => Err(JSObjectProxyError::CouldNotCreateReference.into()),
        }
    }

    pub fn as_js(&self, in_context: &'c JSCoreContext<'c>) -> EsperantoResult<*mut OpaqueJSValue> {
        let mut borrowed = self
            .js_value
            .try_borrow_mut()
            .map_err(|e| JSObjectProxyError::from(e))?;

        match &mut *borrowed {
            Some(jsv) => jsv.raw_ref.as_mut(),
            None => {
                let class_def = in_context.runtime.get_class_def::<T>()?;
                let obj = unsafe {
                    JSObjectMake(
                        in_context.raw_ref,
                        class_def,
                        self.reference as *const T as *mut std::ffi::c_void,
                    )
                };
                let mut js_value = JSCoreValue {
                    raw_ref: obj.try_into()?,
                    context: in_context,
                };

                let raw_to_return = js_value.raw_ref.as_mut()?;

                *borrowed = Some(js_value);
                Ok(raw_to_return)
            }
        }
    }
}
