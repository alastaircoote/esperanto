use crate::ref_count::free_value;
use crate::{
    qjs_classids::{get_class_id, QJSClassType},
    qjs_context::QJSContext,
    qjs_runtime::QJSRuntime,
};
use esperanto_shared::util::closures::{FunctionInvocation, FunctionInvocationContext};
use libquickjs_sys::{
    JSClassDef, JSClassID, JSContext, JSValue as QJSRawValue, JS_GetOpaque, JS_IsRegisteredClass,
    JS_NewCFunctionData, JS_NewClass, JS_NewObjectClass, JS_SetOpaque,
};
use std::rc::Rc;

unsafe extern "C" fn run_native_closure(
    _ctx: *mut JSContext,
    this_val: QJSRawValue,
    argc: i32,
    argv: *mut QJSRawValue,
    // Magic is for getters and setters (and maybe constructors?), we haven't
    // had a need to use it yet
    _magic: i32,
    data: *mut QJSRawValue,
) -> QJSRawValue {
    // QuickJS uses different "class IDs" for storing opaque data. We have specified one for
    // this closure context:
    let class_id = get_class_id(QJSClassType::ClosureContext);

    // Now we grab the raw pointer from the JSValue:
    let closure_raw = JS_GetOpaque(*data, class_id) as *mut FunctionInvocation<QJSContext>;

    // Then we inflate that back into the box we previously stored:
    let closure = Box::from_raw(closure_raw);

    // Now we actually run the closure and grab the result:
    let evaluation_result = closure(FunctionInvocationContext {
        _this_val: this_val,
        number_of_arguments: argc as usize,
        arguments: argv,
    });

    // Then convert the storage back into a raw pointer. This ensures that Rust doesn't free
    // the memory for the closure (since we might want to run it again). The memory is eventually
    // freed in the finalizer.
    Box::into_raw(closure);

    // *finally* we return the actual result.
    evaluation_result.unwrap().raw
}

// This function is called when the QuickJS garbage collector cleans up the variable.
// At this point we free the memory associated with the closure.
unsafe extern "C" fn finalize_closure_context(
    _: *mut libquickjs_sys::JSRuntime,
    value: libquickjs_sys::JSValue,
) {
    let class_id = get_class_id(QJSClassType::ClosureContext);
    let closure_raw = JS_GetOpaque(value, class_id) as *mut FunctionInvocation<QJSContext>;

    // This creates the closure context then, because it isn't returned anywhere, immediately frees the memory
    // associated with it
    let _ = Box::from_raw(closure_raw);
}

// This is more for debugging than anything else, but we need to give our closure context
// holder a name, so we store that here.
const CLOSURE_CONTEXT_CLASS_NAME: &'static [u8] = b"ClosureContext\0";

// In order to make sure everything gets garbage collected correctly we need to register
// our class with the runtime, providing the class finalizer. This function makes sure
// that has happened:
unsafe fn ensure_closure_context_class_registered(runtime: &QJSRuntime) -> JSClassID {
    let class_id = get_class_id(QJSClassType::ClosureContext);
    if JS_IsRegisteredClass(runtime.raw, class_id) > 0 {
        return class_id;
    }
    JS_NewClass(
        runtime.raw,
        class_id,
        &JSClassDef {
            class_name: CLOSURE_CONTEXT_CLASS_NAME.as_ptr() as *const i8,
            // call is for constructing a new instance of a class, and we don't want
            // to ever allow that on the JS side
            call: None,
            finalizer: Some(finalize_closure_context),
            gc_mark: None,
            exotic: std::ptr::null_mut(),
        },
    );
    class_id
}

pub fn wrap_closure(
    closure: FunctionInvocation<QJSContext>,
    in_context: &Rc<QJSContext>,
) -> QJSRawValue {
    let class_id = unsafe { ensure_closure_context_class_registered(&in_context.runtime) };

    // Must double box this value (it's already a box). Not entirely clear why but I think it's because into_raw breaks down the outer box we
    // still need to be keeping one in the interior
    let raw_pointer = Box::into_raw(Box::new(closure));

    // Create the hidden JSValue that will store our pointer
    let mut pointer_container = unsafe { JS_NewObjectClass(in_context.raw, class_id as i32) };

    // Store the pointer in it:
    unsafe { JS_SetOpaque(pointer_container, raw_pointer as *mut std::ffi::c_void) };

    // Then wrap it in a JavaScript function
    let result = unsafe {
        JS_NewCFunctionData(
            in_context.raw,
            Some(run_native_closure),
            1,
            0,
            1,
            &mut pointer_container,
        )
    };

    // Since the function is now holding a reference to the hidden pointer container we want to free our reference
    // to it, otherwise it never gets garbage collected
    unsafe { free_value(in_context.raw, pointer_container) };

    // And then return the function
    result
}
