use crate::ref_count::free_value;
use crate::{
    qjs_classids::{get_class_id, QJSClassType},
    qjs_context::QJSContext,
    qjs_runtime::QJSRuntime,
    qjs_value::{QJSValue, EXCEPTION_RAW},
};
use esperanto_shared::{
    errors::{JSContextError, JSError},
    traits::{FromJSValue, ToJSValue},
};
use libquickjs_sys::{
    JSClassDef, JSClassID, JSContext, JSValue, JS_GetOpaque, JS_IsRegisteredClass,
    JS_NewCFunctionData, JS_NewClass, JS_NewObjectClass, JS_SetOpaque,
};
use std::rc::Rc;

struct FunctionInvocationContext {
    _this_val: JSValue,
    number_of_arguments: i32,
    arguments: *mut JSValue,
    _context: *mut JSContext,
}

type FunctionInvokeReceiver = dyn Fn(FunctionInvocationContext) -> JSValue;

unsafe extern "C" fn run_native_closure(
    ctx: *mut JSContext,
    this_val: JSValue,
    argc: i32,
    argv: *mut JSValue,
    // Magic is for getters and setters (and maybe constructors?), we haven't
    // had a need to use it yet
    _magic: i32,
    data: *mut JSValue,
) -> JSValue {
    // QuickJS uses different "class IDs" for storing opaque data. We have specified one for
    // this closure context:
    let class_id = get_class_id(QJSClassType::ClosureContext);

    // Now we grab the raw pointer from the JSValue:
    let closure_raw = JS_GetOpaque(*data, class_id) as *mut Box<FunctionInvokeReceiver>;

    // Then we inflate that back into the box we previously stored:
    let closure = Box::from_raw(closure_raw);

    // Now we actually run the closure and grab the result:
    let evaluation_result = closure(FunctionInvocationContext {
        _this_val: this_val,
        number_of_arguments: argc,
        arguments: argv,
        _context: ctx,
    });

    // Then convert the storage back into a raw pointer. This ensures that Rust doesn't free
    // the memory for the closure (since we might want to run it again). The memory is finally
    // freed in the finalizer.
    Box::into_raw(closure);

    // *finally* we return the actual result.
    evaluation_result
}

// This function is called when the QuickJS garbage collector cleans up the variable.
// At this point we free the memory associated with the closure.
unsafe extern "C" fn finalize_closure_context(
    _: *mut libquickjs_sys::JSRuntime,
    value: libquickjs_sys::JSValue,
) {
    let class_id = get_class_id(QJSClassType::ClosureContext);
    let closure_raw = JS_GetOpaque(value, class_id) as *mut Box<FunctionInvokeReceiver>;

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

type ClosureWrapper =
    dyn Fn(&[libquickjs_sys::JSValue], &Rc<QJSContext>) -> Result<QJSValue, JSContextError>;

pub fn wrap_one_argument_closure<Input, Output, ClosureType>(
    closure: ClosureType,
    in_context: &Rc<QJSContext>,
) -> JSValue
where
    Input: FromJSValue<QJSValue> + 'static,
    Output: ToJSValue<QJSValue> + 'static,
    ClosureType: (Fn(Input) -> Output) + 'static,
{
    let wrapper: Box<ClosureWrapper> = Box::new(move |arguments, in_context| {
        if arguments.len() < 1 {
            // more than 1 is OK by the standard JS operates: we just ignore the extras.
            return Err(JSError {
                name: "ArgumentError".to_string(),
                message: format!("Expected 1 argument, got {}", arguments.len()),
            }
            .into());
        }

        let argument_converted =
            Input::from_js_value(QJSValue::from_raw(arguments[0], in_context))?;
        let output_value = closure(argument_converted);
        Ok(output_value.to_js_value(&in_context)?)
    });

    create_function_for_wrapper(wrapper, in_context)
}

pub fn wrap_two_argument_closure<Input1, Input2, Output, ClosureType>(
    closure: ClosureType,
    in_context: &Rc<QJSContext>,
) -> JSValue
where
    Input1: FromJSValue<QJSValue> + 'static,
    Input2: FromJSValue<QJSValue> + 'static,
    Output: ToJSValue<QJSValue> + 'static,
    ClosureType: (Fn(Input1, Input2) -> Output) + 'static,
{
    let wrapper: Box<ClosureWrapper> = Box::new(move |arguments, in_context| {
        if arguments.len() < 2 {
            return Err(JSError {
                name: "ArgumentError".to_string(),
                message: format!("Expected 2 arguments, got {}", arguments.len()),
            }
            .into());
        }

        let argument_one_converted =
            Input1::from_js_value(QJSValue::from_raw(arguments[0], in_context))?;
        let argument_two_converted =
            Input2::from_js_value(QJSValue::from_raw(arguments[1], in_context))?;

        let output_value = closure(argument_one_converted, argument_two_converted);
        Ok(output_value.to_js_value(&in_context)?)
    });

    create_function_for_wrapper(wrapper, in_context)
}

fn create_function_for_wrapper(
    wrapper: Box<ClosureWrapper>,
    in_context: &Rc<QJSContext>,
) -> JSValue {
    // The way QuickJS allows this is quite confusing at first. It lets you create a function that wraps
    // a C function and attach "data" to it, which is essential for us because the C function has to be
    // static and we want to be able to dynamically dispatch Rust closures.

    // BUT we can't attach the rust closure itself as "data", the data must be a JSValue. The good news is that
    // we can attach "opaque data" to a JSValue, so we can still do what we want, just with a layer of misdirection.

    // BUT we can't just arbitrarily attach opaque data. Instead we need to create a "JSClassID", then register
    // that class with the context, then create an instance of that class, THEN attach opaque data to the class specifically.
    // Phew.

    // SO. First of all we grab our class ID (and also ensure it's been registered correctly)
    let class_id = unsafe { ensure_closure_context_class_registered(&in_context.runtime) };

    // We use a weak reference here because if we don't we'll have a loop: the JSContext internals will
    // hold onto the closure wrapper, and the closure wrapper will hold onto the externals of the JSContext.
    let context_weak = Rc::downgrade(in_context);

    let closure_wrapper = move |closure_context: FunctionInvocationContext| -> JSValue {
        let context = match context_weak.upgrade() {
            Some(val) => val,
            None => {
                // This shouldn't really happen since it shouldn't be possible to to execute
                // a function after the runtime has been destroyed. But all the same, we need to
                // unwrap the weak reference. Since we can't get the context we can't properly throw
                // an error, but we can at least return the QuickJS exception constant.
                return EXCEPTION_RAW;
                // return Err(JSError {
                //     name: "LifetimeError".to_string(),
                //     message: "This JavaScript context has been destroyed".to_string()
                // })
            }
        };

        let result = || -> Result<QJSValue, JSContextError> {
            if closure_context.number_of_arguments < 1 {
                // more than 1 is OK by the standard JS operates: we just ignore the extras.
                return Err(JSError {
                    name: "ArgumentError".to_string(),
                    message: format!(
                        "Expected 1 argument, got {}",
                        closure_context.number_of_arguments
                    ),
                }
                .into());
            }

            let arguments_slice = unsafe {
                std::slice::from_raw_parts(
                    closure_context.arguments,
                    closure_context.number_of_arguments as usize,
                )
            };

            Ok(wrapper(arguments_slice, &context)?)
        };

        match result() {
            // TODO: better error handling, need to pass the actual error
            Err(_) => QJSValue::exception(&context).raw,
            Ok(jsvalue) => jsvalue.raw,
        }
    };

    let boxed: Box<FunctionInvokeReceiver> = Box::new(closure_wrapper);
    let raw_pointer = Box::into_raw(Box::new(boxed));

    let mut data_container = unsafe { JS_NewObjectClass(in_context.raw, class_id as i32) };

    unsafe { JS_SetOpaque(data_container, raw_pointer as *mut std::ffi::c_void) };

    let result = unsafe {
        JS_NewCFunctionData(
            in_context.raw,
            Some(run_native_closure),
            1,
            0,
            1,
            &mut data_container,
        )
    };
    unsafe { free_value(in_context.raw, data_container) };
    result
}
