use std::{
    ffi::CStr,
    ffi::{CString, NulError},
    marker::PhantomData,
    rc::Rc,
};

use esperanto_shared::{
    errors::JSContextError, traits::JSValue, util::jsclass_builder::JSClassBuilder,
    util::jsclass_builder::JSClassBuilderOutput,
};
use javascriptcore_sys::{
    JSClassCreate, JSClassDefinition, JSObjectMake, JSObjectRef, JSObjectSetPrototype,
    JSStaticFunction, JSStaticValue, JSValueMakeNumber, JSValueRef,
};

use crate::{JSCGlobalContext, JSCValue};

unsafe extern "C" fn js_class_call_as_constructor(
    ctx: *const javascriptcore_sys::OpaqueJSContext,
    constructor: *mut javascriptcore_sys::OpaqueJSValue,
    argc: usize,
    argv: *const *const javascriptcore_sys::OpaqueJSValue,
    exception: *mut *const javascriptcore_sys::OpaqueJSValue,
) -> *mut javascriptcore_sys::OpaqueJSValue {
    let raw_obj = JSObjectMake(ctx, std::ptr::null_mut(), std::ptr::null_mut());
    JSObjectSetPrototype(ctx, raw_obj, constructor);
    return raw_obj;
    // std::ptr::null_mut()
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

unsafe extern "C" fn js_func_call(
    ctx: javascriptcore_sys::JSContextRef,
    function: JSObjectRef,
    thisObject: JSObjectRef,
    argumentCount: usize,
    arguments: *const JSValueRef,
    exception: *mut JSValueRef,
) -> *const javascriptcore_sys::OpaqueJSValue {
    // JSCValue::from_number(number, in_context)
    JSValueMakeNumber(ctx, 5.5)
    // std::ptr::null_mut()
}

impl<NativeObject> JSClassBuilderOutput<JSCGlobalContext>
    for JSClassBuilder<JSCGlobalContext, NativeObject>
{
    fn build_class(self, in_context: &Rc<JSCGlobalContext>) -> Result<JSCValue, JSContextError> {
        // Maybe there's a better Rust-y way of doing this, but if we declare the CString inside
        // the map call below the memory allocated to the string is immediately released, before
        // we run JSClassCreate, so there are no methods defined. If we keep them in a separate
        // vec they're kept around until this function completes.
        let method_names_as_cstrings = self
            .methods
            .keys()
            .map(|k| CString::new(*k))
            .collect::<Result<Vec<CString>, _>>()?;

        // Same thing for the class name:
        let class_name = CString::new(self.name)?;

        let method_definitions: Vec<JSStaticFunction> = method_names_as_cstrings
            .iter()
            .map(|name| JSStaticFunction {
                name: name.as_ptr(),
                attributes: 0,
                callAsFunction: Some(js_func_call),
            })
            .chain(
                // Since we aren't passing the array length to JavaScriptCore it requires
                // some kind of signifier that the array has ended. Much like a null-terminated string,
                // it requires a JSStaticFunction with a NULL name:
                vec![JSStaticFunction {
                    name: std::ptr::null_mut(),
                    attributes: 0,
                    callAsFunction: None,
                }],
            )
            .collect();

        let definition = JSClassDefinition {
            version: 1,
            className: class_name.as_ptr(),
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
            staticFunctions: method_definitions.as_ptr(),
        };

        // std::mem::forget(funcs);

        let class = unsafe { JSClassCreate(&definition) };
        let js_value = unsafe { JSObjectMake(in_context.raw_ref, class, std::ptr::null_mut()) };

        let pinned = Box::pin(self);

        JSCValue::from_raw_object_ref(js_value, in_context)
    }
}

#[cfg(test)]
mod test {
    use std::rc::Rc;

    use esperanto_shared::{
        errors::JSContextError, traits::JSContext, traits::JSValue, traits::ToJSValue,
        util::jsclass_builder::JSClassBuilder, util::jsclass_builder::JSClassBuilderOutput,
    };
    use javascriptcore_sys::OpaqueJSValue;

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

        println!("obj? {}", build.as_string().unwrap());
        let test_func = JSCValue::create_function(
            &ctx,
            vec!["arg"],
            "\
            return (new arg()).getValue()
        ",
        )
        .unwrap();

        let result = test_func.call_with_arguments(vec![&build]).unwrap();
        println!("done? {}", result.as_string().unwrap())
    }
}
