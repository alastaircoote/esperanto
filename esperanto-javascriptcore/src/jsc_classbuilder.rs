use std::{ffi::CStr, ffi::CString, marker::PhantomData, rc::Rc};

use esperanto_shared::{
    errors::JSContextError, traits::JSValue, util::jsclass_builder::JSClassBuilder,
    util::jsclass_builder::JSClassBuilderOutput,
};
use javascriptcore_sys::{
    JSClassCreate, JSClassDefinition, JSObjectRef, JSStaticValue, JSValueRef,
};

use crate::{jsc_class::JSCClass, JSCGlobalContext, JSCValue};

unsafe extern "C" fn js_class_call_as_constructor(
    ctx: *const javascriptcore_sys::OpaqueJSContext,
    constructor: *mut javascriptcore_sys::OpaqueJSValue,
    argc: usize,
    argv: *const *const javascriptcore_sys::OpaqueJSValue,
    exception: *mut *const javascriptcore_sys::OpaqueJSValue,
) -> *mut javascriptcore_sys::OpaqueJSValue {
    std::ptr::null_mut()
}

unsafe extern "C" fn js_class_finalize(class_instance: *mut javascriptcore_sys::OpaqueJSValue) {}

unsafe extern "C" fn js_obj_get(
    ctx: *const javascriptcore_sys::OpaqueJSContext,
    obj: *mut javascriptcore_sys::OpaqueJSValue,
    propertyName: *mut javascriptcore_sys::OpaqueJSString,
    exception: *mut *const javascriptcore_sys::OpaqueJSValue,
) -> *const javascriptcore_sys::OpaqueJSValue {
    std::ptr::null_mut()
}

unsafe extern "C" fn js_obj_set(
    ctx: *const javascriptcore_sys::OpaqueJSContext,
    obj: *mut javascriptcore_sys::OpaqueJSValue,
    property_name: *mut javascriptcore_sys::OpaqueJSString,
    new_vlue: *const javascriptcore_sys::OpaqueJSValue,
    exception: *mut *const javascriptcore_sys::OpaqueJSValue,
) -> bool {
    false
}

impl<NativeObject> JSClassBuilderOutput<JSCGlobalContext>
    for JSClassBuilder<JSCGlobalContext, NativeObject>
{
    fn build_class(self, in_context: &Rc<JSCGlobalContext>) -> Result<JSCClass, JSContextError> {
        let name = CString::new(self.name)?;
        let val = JSStaticValue {
            name: CString::new("test")?.as_ptr(),
            getProperty: Some(js_obj_get),
            setProperty: Some(js_obj_set),
            attributes: 0,
        };
        let definition = JSClassDefinition {
            version: 1,
            className: name.as_ptr(),
            attributes: 0,
            parentClass: std::ptr::null_mut(),
            initialize: None,
            callAsFunction: None,
            callAsConstructor: Some(js_class_call_as_constructor),
            finalize: Some(js_class_finalize),
            hasProperty: None,
            getProperty: None,
            setProperty: None,
            deleteProperty: None,
            getPropertyNames: None,
            hasInstance: None,
            convertToType: None,
            staticValues: std::ptr::null_mut(),
            staticFunctions: std::ptr::null_mut(),
        };

        let class = unsafe { JSClassCreate(&definition) };
        // JSCValue::from_raw_object_ref(class as JSObjectRef, in_context)
    }
}

#[cfg(test)]
mod test {
    use std::rc::Rc;

    use esperanto_shared::{
        errors::JSContextError, traits::JSContext, traits::JSValue, traits::ToJSValue,
        util::jsclass_builder::JSClassBuilder, util::jsclass_builder::JSClassBuilderOutput,
    };

    use crate::{JSCGlobalContext, JSCValue};

    struct Test {
        value: f64,
    }

    // impl ToJSValue<JSCValue> for Test {
    //     fn to_js_value(
    //         self,
    //         in_context: &std::rc::Rc<JSCGlobalContext>,
    //     ) -> Result<JSCValue, esperanto_shared::errors::JSContextError> {
    //         return JSCValue::from_bool(true, in_context);
    //     }
    // }

    impl Test {
        fn new(value: f64) -> Result<Self, JSContextError> {
            Ok(Test { value })
        }

        fn get_value(&self) -> Result<f64, JSContextError> {
            Ok(self.value)
        }
    }

    #[test]
    fn wtf() {
        let ctx = JSCGlobalContext::new().unwrap();
        let build = JSClassBuilder::<JSCGlobalContext, Test>::new("Test")
            .set_constructor_one_arg(Test::new)
            .add_zero_argument_method("getValue", Test::get_value)
            .build_class(&ctx)
            .unwrap();
    }
}
