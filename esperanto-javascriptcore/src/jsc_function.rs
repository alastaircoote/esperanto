use crate::jsc_globalcontext::JSCGlobalContext;
use esperanto_shared::util::closures::{FunctionInvocationContext, FunctionToInvoke};
use javascriptcore_sys::{
    JSClassCreate, JSClassDefinition, JSClassRef, JSObjectGetPrivate, JSObjectMake,
    JSValueIsObjectOfClass, JSValueRef, OpaqueJSContext, OpaqueJSValue,
};
use std::{ffi::c_void, rc::Rc};

const NATIVE_FUNCTION_CLASS_NAME: &'static [u8] = b"NativeFunction\0";

unsafe extern "C" fn invoke_function(
    ctx: *const OpaqueJSContext,
    _function: *mut OpaqueJSValue,
    this_object: *mut OpaqueJSValue,
    argument_count: usize,
    arguments: *const *const OpaqueJSValue,
    _exception: *mut *const OpaqueJSValue,
) -> *const OpaqueJSValue {
    let class_ref = NATIVE_FUNCTION_CLASS.unwrap();

    // It's possible for user code to re-bind the function by using xxx.bind(). So we need
    // to make sure we're actually bound to a native functon wrapper before we do anything else.
    if JSValueIsObjectOfClass(ctx, this_object, class_ref) == false {
        panic!("oh no")
    }

    let pointer = JSObjectGetPrivate(this_object) as *mut FunctionToInvoke<JSCGlobalContext>;

    let unwrapped_function = Box::from_raw(pointer);

    let output = unwrapped_function(FunctionInvocationContext {
        _this_val: this_object,
        number_of_arguments: argument_count,
        arguments: arguments,
    })
    .unwrap();

    Box::into_raw(unwrapped_function);

    output.raw_ref.get_jsvalue()
}

unsafe extern "C" fn finalize_native_function(val: *mut OpaqueJSValue) {
    let pointer = JSObjectGetPrivate(val) as *mut FunctionToInvoke<JSCGlobalContext>;
    let _ = Box::from_raw(pointer);
}

const NATIVE_FUNCTION_CLASS_DEFINITION: JSClassDefinition = JSClassDefinition {
    version: 1,
    attributes: 0,
    className: NATIVE_FUNCTION_CLASS_NAME.as_ptr() as *const i8,
    parentClass: std::ptr::null_mut(),
    staticValues: std::ptr::null_mut(),
    staticFunctions: std::ptr::null_mut(),
    initialize: None,
    finalize: Some(finalize_native_function),
    hasProperty: None,
    getProperty: None,
    setProperty: None,
    deleteProperty: None,
    getPropertyNames: None,
    callAsFunction: Some(invoke_function),
    callAsConstructor: None,
    hasInstance: None,
    convertToType: None,
};

static mut NATIVE_FUNCTION_CLASS: Option<JSClassRef> = None;

pub fn wrap_closure(
    closure: FunctionToInvoke<JSCGlobalContext>,
    in_context: &Rc<JSCGlobalContext>,
) -> JSValueRef {
    let class_ref = unsafe {
        match NATIVE_FUNCTION_CLASS {
            Some(class) => class,
            None => {
                let created_definition = JSClassCreate(&NATIVE_FUNCTION_CLASS_DEFINITION);
                NATIVE_FUNCTION_CLASS = Some(created_definition);
                created_definition
            }
        }
    };

    let raw_ptr = Box::into_raw(Box::new(closure));
    unsafe { JSObjectMake(in_context.raw_ref, class_ref, raw_ptr as *mut c_void) }
}
