use crate::{jsc_contextgroup::JSCContextGroup, jsc_string::JSCString};
use crate::{jsc_error::JSErrorFromJSC, jsc_value::JSCValue};
use esperanto_shared::traits::{JSContext, JSValue};
use esperanto_shared::{
    errors::{JSContextError, JSError},
    traits::JSRuntime,
};
use javascriptcore_sys::{
    JSClassCreate, JSClassDefinition, JSContextGetGlobalObject, JSContextGroupCreate,
    JSContextGroupRelease, JSEvaluateScript, JSGlobalContextCreateInGroup, JSGlobalContextRelease,
    JSGlobalContextRetain, JSObjectGetPrivate, JSObjectRef, JSObjectSetPrivate, JSValueRef,
    OpaqueJSContext, OpaqueJSContextGroup,
};
// use slotmap::{DefaultKey, SecondaryMap, SlotMap};
use std::{cell::Cell, fmt::Debug, hash::Hash, os::raw::c_char, rc::Rc};

#[derive(Debug)]
pub struct JSCGlobalContext<'group> {
    pub(crate) raw_ref: *mut OpaqueJSContext,
    pub(crate) group: &'group JSCContextGroup,
}

impl JSCGlobalContext<'_> {
    fn evaluate_jscstring(self: &Self, script: JSCString) -> Result<JSCValue, JSContextError> {
        let mut exception_ptr: JSValueRef = std::ptr::null_mut();

        let return_value = unsafe {
            JSEvaluateScript(
                self.raw_ref,
                script.raw_ref,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                0,
                &mut exception_ptr,
            )
        };

        JSError::check_jsc_value_ref(exception_ptr, self)?;

        JSCValue::from_raw(return_value, self)
    }
}

impl<'group> JSContext<'group> for JSCGlobalContext<'group> {
    type ValueType = JSCValue<'group>;
    type RuntimeType = JSCContextGroup;

    fn new() -> Result<Self, JSContextError> {
        let runtime = Self::RuntimeType::new()?;
        Self::new_in_runtime(&runtime)
    }

    fn new_in_runtime(runtime: &'group Self::RuntimeType) -> Result<Self, JSContextError> {
        runtime.create_context()
    }

    fn evaluate<'a>(&'group self, script: &'a str) -> Result<JSCValue, JSContextError> {
        let script_jsstring = JSCString::from_string(script)?;
        self.evaluate_jscstring(script_jsstring)
    }

    fn evaluate_cstring(&self, script: *const c_char) -> Result<Self::ValueType, JSContextError> {
        let script_jsstring = JSCString::from_c_string(script)?;
        self.evaluate_jscstring(script_jsstring)
    }

    fn compile_string<'a>(&self, _: *const c_char) -> Result<&'a [u8], JSContextError> {
        Err(JSContextError::NotSupported)
    }

    fn eval_compiled(&self, _: &[u8]) -> Result<Self::ValueType, JSContextError> {
        Err(JSContextError::NotSupported)
    }
}

impl Drop for JSCGlobalContext<'_> {
    fn drop(&mut self) {
        unsafe { JSGlobalContextRelease(self.raw_ref) }
    }
}

#[cfg(test)]
mod test {

    use std::ffi::CString;

    use crate::jsc_value::RawRef;

    use super::*;
    use esperanto_shared::trait_tests::jscontext_tests;
    use javascriptcore_sys::{
        JSClassCreate, JSClassDefinition, JSClassRef, JSClassRelease, JSClassRetain,
        JSContextGetGlobalObject, JSContextRef, JSGlobalContextCreate, JSGlobalContextRef,
        JSObjectCallAsFunction, JSObjectDeleteProperty, JSObjectGetPrivate, JSObjectMake,
        JSObjectMakeFunction, JSObjectMakeFunctionWithCallback, JSObjectRef, JSObjectSetPrivate,
        JSObjectSetProperty, JSObjectSetPropertyAtIndex, JSObjectSetPrototype,
        JSStringCreateWithUTF8CString, JSStringRelease, JSStringRetain, JSValueIsUndefined,
        JSValueMakeUndefined, JSValueProtect, JSValueToStringCopy, JSValueUnprotect, OpaqueJSClass,
        OpaqueJSValue,
    };
    // use javascriptcore_sys::{JSContextRef, JSGarbageCollect, JSValueUnprotect};

    #[test]
    fn it_evaluates_correct_code() {
        jscontext_tests::it_evaluates_correct_code::<JSCGlobalContext>();
    }

    #[test]
    fn it_throws_exceptions_on_invalid_code() {
        jscontext_tests::it_throws_exceptions_on_invalid_code::<JSCGlobalContext>();
    }

    // #[link(name = "JavaScriptCore", kind = "framework")]
    extern "C" {
        fn JSSynchronousGarbageCollectForDebugging(
            ctx: *const javascriptcore_sys::OpaqueJSContext,
        ) -> ();
    }

    // sync garbage collect thing doesn't seem to work, so never mind

    // static mut context: JSGlobalContextRef = std::ptr::null_mut();

    unsafe extern "C" fn init(
        ctx: *const javascriptcore_sys::OpaqueJSContext,
        val: *mut javascriptcore_sys::OpaqueJSValue,
    ) {
        println!("hello!")
    }

    unsafe extern "C" fn fin(val: *mut javascriptcore_sys::OpaqueJSValue) {
        println!("fin instance!");
        // JSObjectSetPrototype(context, val, std::ptr::null_mut())
    }

    unsafe extern "C" fn fin2(val: *mut javascriptcore_sys::OpaqueJSValue) {
        println!("fin class!")
    }

    unsafe extern "C" fn construct(
        ctx: *const javascriptcore_sys::OpaqueJSContext,
        constructor: *mut javascriptcore_sys::OpaqueJSValue,
        argc: usize,
        argv: *const *const javascriptcore_sys::OpaqueJSValue,
        exception: *mut *const javascriptcore_sys::OpaqueJSValue,
    ) -> *mut javascriptcore_sys::OpaqueJSValue {
        let type_def = JSClassDefinition {
            version: 1,
            attributes: 0,
            className: std::ptr::null_mut(),
            parentClass: std::ptr::null_mut(),
            staticValues: std::ptr::null_mut(),
            staticFunctions: std::ptr::null_mut(),
            initialize: None,
            finalize: Some(fin),
            hasProperty: None,
            getProperty: None,
            setProperty: None,
            deleteProperty: None,
            getPropertyNames: None,
            callAsFunction: None,
            callAsConstructor: None,
            hasInstance: None,
            convertToType: None,
        };

        let class = JSClassCreate(&type_def);
        println!("Construct");
        let obj = JSObjectMake(ctx, class, std::ptr::null_mut());
        JSObjectSetPrototype(ctx, obj, constructor);
        obj
    }

    unsafe extern "C" fn func_test(
        ctx: *const javascriptcore_sys::OpaqueJSContext,
        func: *mut javascriptcore_sys::OpaqueJSValue,
        this: *mut javascriptcore_sys::OpaqueJSValue,
        argc: usize,
        argv: *const *const javascriptcore_sys::OpaqueJSValue,
        exception: *mut *const javascriptcore_sys::OpaqueJSValue,
    ) -> *const javascriptcore_sys::OpaqueJSValue {
        let v = JSObjectGetPrivate(func);
        JSObjectMake(ctx, v as JSClassRef, std::ptr::null_mut())
    }

    #[test]
    fn it_discards_values() {
        let grp = unsafe { JSContextGroupCreate() };
        let context = unsafe { JSGlobalContextCreateInGroup(grp, std::ptr::null_mut()) };
        unsafe { JSContextGroupRelease(grp) };
        let script2 = CString::new("'123'").unwrap();
        let script_str2 = unsafe { JSStringCreateWithUTF8CString(script2.as_ptr()) };
        let mut exception_ptr: JSValueRef = std::ptr::null_mut();
        unsafe {
            JSEvaluateScript(
                context,
                script_str2,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                0,
                &mut exception_ptr,
            )
        };

        unsafe { JSGlobalContextRetain(context) };
        let global_obj = unsafe { JSContextGetGlobalObject(context) };

        let name = CString::new("TestClass").unwrap();

        let type_def = JSClassDefinition {
            version: 1,
            attributes: 0,
            className: name.as_ptr(),
            parentClass: std::ptr::null_mut(),
            staticValues: std::ptr::null_mut(),
            staticFunctions: std::ptr::null_mut(),
            initialize: None,
            finalize: Some(fin2),
            hasProperty: None,
            getProperty: None,
            setProperty: None,
            deleteProperty: None,
            getPropertyNames: None,
            callAsFunction: None,
            callAsConstructor: Some(construct),
            hasInstance: None,
            convertToType: None,
        };

        let class = unsafe { JSClassCreate(&type_def) };
        unsafe { JSClassRetain(class) };
        // let js_obj = unsafe { JSObjectMake(context, class, std::ptr::null_mut()) };

        let class2 = unsafe { JSClassCreate(&type_def) };
        let js_obj2 = unsafe { JSObjectMake(context, class2, std::ptr::null_mut()) };

        let type_def_getter = JSClassDefinition {
            version: 1,
            attributes: 0,
            className: std::ptr::null_mut(),
            parentClass: std::ptr::null_mut(),
            staticValues: std::ptr::null_mut(),
            staticFunctions: std::ptr::null_mut(),
            initialize: None,
            finalize: None,
            hasProperty: None,
            getProperty: None,
            setProperty: None,
            deleteProperty: None,
            getPropertyNames: None,
            callAsFunction: Some(func_test),
            callAsConstructor: None,
            hasInstance: None,
            convertToType: None,
        };

        let getter_class = unsafe { JSClassCreate(&type_def_getter) };

        let test_func = unsafe { JSObjectMake(context, getter_class, std::ptr::null_mut()) };

        // let test_func = unsafe {
        //     JSObjectMakeFunctionWithCallback(context, std::ptr::null_mut(), Some(func_test))
        // };

        unsafe { JSObjectSetPrivate(test_func, class as *mut std::ffi::c_void) };

        // let objs = vec![js_obj as JSValueRef, js_obj2 as JSValueRef];

        // unsafe { JSValueProtect(context, js_obj) };
        // unsafe { JSValueUnprotect(context, js_obj) };

        // unsafe { JSClassRelease(class) };

        let arg_name = CString::new("testClass").unwrap();
        let arg_str = unsafe { JSStringCreateWithUTF8CString(arg_name.as_ptr()) };

        let arg_name2 = CString::new("testClass2").unwrap();
        let arg_str2 = unsafe { JSStringCreateWithUTF8CString(arg_name2.as_ptr()) };

        let arg_names = vec![arg_str, arg_str2];

        let func_body = CString::new("var cl = testClass(); new cl()").unwrap();
        let func_body_str = unsafe { JSStringCreateWithUTF8CString(func_body.as_ptr()) };

        let func = unsafe {
            JSObjectMakeFunction(
                context,
                std::ptr::null_mut(),
                2,
                arg_names.as_ptr(),
                func_body_str,
                std::ptr::null_mut(),
                0,
                std::ptr::null_mut(),
            )
        };

        let vals = vec![test_func as JSValueRef, js_obj2];

        let mut exception_ptr: JSValueRef = std::ptr::null_mut();

        unsafe {
            JSObjectCallAsFunction(
                context,
                func,
                std::ptr::null_mut(),
                1,
                vals.as_ptr(),
                &mut exception_ptr,
            )
        };

        if exception_ptr.is_null() == false {
            // panic!("oh no");

            let str_ptr =
                unsafe { JSValueToStringCopy(context, exception_ptr, std::ptr::null_mut()) };
            let s = JSCString::from_ptr(str_ptr);
            let whaaa = s.to_string().unwrap();
            println!("{}", whaaa)
        }

        unsafe {
            JSSynchronousGarbageCollectForDebugging(context);
        }
        println!("RELEASE");
        unsafe { JSGlobalContextRelease(context) }
        println!("RELEASE GROUP");
        unsafe { JSContextGroupRelease(grp) }

        // let script2 = CString::new("'123'").unwrap();
        // let script_str2 = unsafe { JSStringCreateWithUTF8CString(script2.as_ptr()) };
        // let mut exception_ptr: JSValueRef = std::ptr::null_mut();
        // unsafe {
        //     JSEvaluateScript(
        //         context,
        //         script_str2,
        //         std::ptr::null_mut(),
        //         std::ptr::null_mut(),
        //         0,
        //         &mut exception_ptr,
        //     )
        // };

        // std::thread::sleep_ms(10000);
        // let s = val.as_string().unwrap();
        // println!("{}", s)
    }

    static mut is_finalized: bool = false;

    unsafe extern "C" fn test_class_finalizer(val: *mut javascriptcore_sys::OpaqueJSValue) {
        is_finalized = true
    }

    unsafe extern "C" fn test_generate_class_with_finalizer(
        ctx: *const javascriptcore_sys::OpaqueJSContext,
        func: *mut javascriptcore_sys::OpaqueJSValue,
        this: *mut javascriptcore_sys::OpaqueJSValue,
        argc: usize,
        argv: *const *const javascriptcore_sys::OpaqueJSValue,
        exception: *mut *const javascriptcore_sys::OpaqueJSValue,
    ) -> *const javascriptcore_sys::OpaqueJSValue {
        let test_class_def = JSClassDefinition {
            version: 1,
            attributes: 0,
            className: std::ptr::null_mut(),
            parentClass: std::ptr::null_mut(),
            staticValues: std::ptr::null_mut(),
            staticFunctions: std::ptr::null_mut(),
            initialize: None,
            finalize: Some(test_class_finalizer),
            hasProperty: None,
            getProperty: None,
            setProperty: None,
            deleteProperty: None,
            getPropertyNames: None,
            callAsFunction: None,
            callAsConstructor: None,
            hasInstance: None,
            convertToType: None,
        };

        let class = JSClassCreate(&test_class_def);

        JSObjectMake(ctx, class, std::ptr::null_mut())
    }
}
