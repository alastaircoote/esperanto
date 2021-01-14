use crate::{shared::external_api::context::JSContextError, EsperantoError, EsperantoResult};
use javascriptcore_sys::{
    JSContextGetGlobalObject, JSObjectGetPrivate, JSObjectSetPrivate, OpaqueJSContext,
    OpaqueJSContextGroup,
};
use std::ffi::c_void;
use thiserror::Error;

use super::jscore_context::JSContext;

#[derive(Error, Debug, PartialEq)]
pub(super) enum JSCoreContextGlobalDataError {
    #[error("Could not store data in the JSContext global object.")]
    CouldNotStorePrivateData,
    #[error("Could not retrieve context data from the global object")]
    CouldNotRetreivePrivateData,
}

impl From<JSCoreContextGlobalDataError> for EsperantoError {
    fn from(val: JSCoreContextGlobalDataError) -> Self {
        EsperantoError::external(val)
    }
}

pub(super) struct JSCoreContextGlobalData<'a> {
    pub ctx_ptr: *const JSContext<'a>,
}

impl JSCoreContextGlobalData<'_> {
    pub fn attach_to_context(context: &JSContext) {
        let ptr = context as *const JSContext;
        let data = JSCoreContextGlobalData { ctx_ptr: ptr };
        let boxed = Box::new(data);
        let global = unsafe { JSContextGetGlobalObject(context.raw_ref) };
        unsafe { JSObjectSetPrivate(global, Box::into_raw(boxed) as *mut std::ffi::c_void) };
    }

    pub fn get_from_raw<'a>(ctx: *const OpaqueJSContext) -> EsperantoResult<&'a Self> {
        let global = unsafe { JSContextGetGlobalObject(ctx) };
        let data_ptr = unsafe { JSObjectGetPrivate(global) } as *const JSCoreContextGlobalData;
        unsafe { data_ptr.as_ref() }.ok_or(JSContextError::CouldNotRetrieveFromNativePointer.into())
    }
}

#[cfg(test)]
mod test {
    // use javascriptcore_sys::{
    //     JSClassCreate, JSClassDefinition, JSContextGroupCreate, JSGlobalContextCreate,
    //     JSGlobalContextCreateInGroup,
    // };

    // use super::{JSCoreContextGlobalData, JSCoreContextGlobalDataError};

    // #[test]
    // fn stores_in_global_object_that_has_class() {
    //     let def = JSClassDefinition::default();
    //     let class = unsafe { JSClassCreate(&def) };
    //     let rt = unsafe { JSContextGroupCreate() };
    //     let ctx = unsafe { JSGlobalContextCreateInGroup(rt, class) };
    //     let data = JSCoreContextGlobalData {
    //         ctx_ptr: ctx,
    //         rt_ptr: rt,
    //     };
    //     data.attach_to_global_object(ctx).unwrap();
    // }

    // #[test]
    // fn fails_to_store_in_global_that_has_no_class() {
    //     // We shouldn't have to worry about this since we never use a null global object, but testing
    //     // this to ensure future behaviour never changes.
    //     let rt = unsafe { JSContextGroupCreate() };
    //     let ctx = unsafe { JSGlobalContextCreateInGroup(rt, std::ptr::null_mut()) };
    //     let data = JSCoreContextGlobalData {
    //         ctx_ptr: ctx,
    //         rt_ptr: rt,
    //     };
    //     let err = data.attach_to_global_object(ctx).unwrap_err();
    //     assert_eq!(err, JSCoreContextGlobalDataError::CouldNotStorePrivateData);
    // }

    // #[test]
    // fn retreives_stored_data() {
    //     let def = JSClassDefinition::default();
    //     let class = unsafe { JSClassCreate(&def) };
    //     let rt = unsafe { JSContextGroupCreate() };
    //     let ctx = unsafe { JSGlobalContextCreateInGroup(rt, class) };
    //     let data = JSCoreContextGlobalData {
    //         ctx_ptr: ctx,
    //         rt_ptr: rt,
    //     };
    //     data.attach_to_global_object(ctx).unwrap();

    //     JSCoreContextGlobalData::fetch_from_global_object(ctx).unwrap();
    // }
}
