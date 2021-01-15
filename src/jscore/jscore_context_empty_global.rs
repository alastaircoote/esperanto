use super::jscore_export::JSCoreExport;
use javascriptcore_sys::{JSClassDefinition, OpaqueJSValue};

/// Even if we don't want to use a custom global scope we do need *some* kind of
/// struct in there. In that case we'll just use this empty scope placeholder.
pub struct EmptyGlobalScope {}

unsafe extern "C" fn dummy_export_finalize(val: *mut OpaqueJSValue) {
    EmptyGlobalScope::clear_private(val);
}

impl JSCoreExport for EmptyGlobalScope {
    fn get_definition<'a>() -> &'a JSClassDefinition {
        const DEFAULT: JSClassDefinition = JSClassDefinition {
            version: 1,
            className: "EsperantoGlobalObject\0".as_ptr() as *const std::os::raw::c_char,
            attributes: 0,
            staticFunctions: std::ptr::null_mut(),
            callAsConstructor: None,
            callAsFunction: None,
            convertToType: None,
            deleteProperty: None,
            hasProperty: None,
            setProperty: None,
            finalize: Some(dummy_export_finalize),
            getProperty: None,
            getPropertyNames: None,
            hasInstance: None,
            initialize: None,
            parentClass: std::ptr::null_mut(),
            staticValues: std::ptr::null_mut(),
        };
        &DEFAULT
    }
}
