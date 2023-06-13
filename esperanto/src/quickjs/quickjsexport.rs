use std::ffi::CString;

use quickjs_android_suitable_sys::JSClassDef;

use super::quickjs_prototype_storage::finalize_class_instance_extern;
use crate::{
    shared::errors::{ConversionError, EsperantoResult},
    JSExportClass,
};

pub(super) trait QuickJSExportExtensions: JSExportClass + Sized {
    fn class_def() -> EsperantoResult<JSClassDef> {
        let name_cstring = CString::new(Self::METADATA.class_name)
            .map_err(|e| ConversionError::CouldNotConvertToJSString(e))?;
        Ok(JSClassDef {
            class_name: name_cstring.as_ptr(),
            call: None,
            finalizer: Some(finalize_class_instance_extern::<Self>),
            gc_mark: None,
            exotic: std::ptr::null_mut(),
        })
    }
}

impl<T> QuickJSExportExtensions for T where T: JSExportClass {}
