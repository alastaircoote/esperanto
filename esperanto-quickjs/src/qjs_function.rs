use crate::ref_count::free_value;
use crate::{
    qjs_classids::{get_class_id, QJSClassType},
    qjs_context::QJSContext,
    qjs_runtime::QJSRuntime,
    qjs_shared_context_ref::SharedQJSContextRef,
    qjs_value::{QJSValue, EXCEPTION_RAW},
};
use esperanto_shared::{
    errors::JSError,
    traits::{FromJSValue, ToJSValue},
};
use libquickjs_sys::{
    JSClassDef, JSClassID, JSContext, JSValue, JS_DefinePropertyValue, JS_GetOpaque,
    JS_GetPropertyUint32, JS_IsRegisteredClass, JS_NewCFunctionData, JS_NewClass,
    JS_NewObjectClass, JS_SetOpaque,
};
use std::{ffi::CString, rc::Rc};

struct FunctionInvocationContext {
    this_val: JSValue,
    number_of_arguments: i32,
    arguments: *mut JSValue,
    context: *mut JSContext,
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
        this_val,
        number_of_arguments: argc,
        arguments: argv,
        context: ctx,
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
    if JS_IsRegisteredClass(runtime.qjs_ref, class_id) > 0 {
        return class_id;
    }
    JS_NewClass(
        runtime.qjs_ref,
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
    dyn Fn(&[libquickjs_sys::JSValue], &Rc<SharedQJSContextRef>) -> Result<QJSValue, JSError>;

pub fn wrap_one_argument_closure<Input, Output, ClosureType>(
    closure: ClosureType,
    in_context: &Rc<SharedQJSContextRef>,
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
            });
        }

        let argument_converted = Input::from_js_value(QJSValue::new(arguments[0], in_context))?;
        let output_value = closure(argument_converted);
        Ok(output_value.to_js_value(&in_context)?)
    });

    create_function_for_wrapper(wrapper, in_context)
}

pub fn wrap_two_argument_closure<Input1, Input2, Output, ClosureType>(
    closure: ClosureType,
    in_context: &Rc<SharedQJSContextRef>,
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
            });
        }

        let argument_one_converted =
            Input1::from_js_value(QJSValue::new(arguments[0], in_context))?;
        let argument_two_converted =
            Input2::from_js_value(QJSValue::new(arguments[1], in_context))?;

        let output_value = closure(argument_one_converted, argument_two_converted);
        Ok(output_value.to_js_value(&in_context)?)
    });

    create_function_for_wrapper(wrapper, in_context)
}

fn create_function_for_wrapper(
    wrapper: Box<ClosureWrapper>,
    in_context: &Rc<SharedQJSContextRef>,
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
    let class_id = unsafe { ensure_closure_context_class_registered(&in_context.runtime_ref) };

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

        let result = || {
            if closure_context.number_of_arguments < 1 {
                // more than 1 is OK by the standard JS operates: we just ignore the extras.
                return Err(JSError {
                    name: "ArgumentError".to_string(),
                    message: format!(
                        "Expected 1 argument, got {}",
                        closure_context.number_of_arguments
                    ),
                });
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
            Err(js_error) => QJSValue::exception(&context).qjs_ref,
            Ok(jsvalue) => jsvalue.qjs_ref,
        }
    };

    let boxed: Box<FunctionInvokeReceiver> = Box::new(closure_wrapper);
    let raw_pointer = Box::into_raw(Box::new(boxed));

    let mut data_container = unsafe { JS_NewObjectClass(in_context.qjs_ref, class_id as i32) };

    unsafe { JS_SetOpaque(data_container, raw_pointer as *mut std::ffi::c_void) };

    let result = unsafe {
        JS_NewCFunctionData(
            in_context.qjs_ref,
            Some(run_native_closure),
            1,
            0,
            1,
            &mut data_container,
        )
    };
    unsafe { free_value(in_context.qjs_ref, data_container) };
    result
}

pub fn create_function_one_argument<I, O, F: (Fn(I) -> O) + 'static>(
    closure: F,
    in_context: &Rc<SharedQJSContextRef>,
) -> JSValue
where
    I: FromJSValue<QJSValue> + 'static,
    O: ToJSValue<QJSValue> + 'static,
{
    // The way QuickJS allows this is quite confusing at first. It lets you create a function that wraps
    // a C function and attach "data" to it, which is essential for us because the C function has to be
    // static and we want to be able to dynamically dispatch Rust closures.

    // BUT we can't attach the rust closure itself as "data", the data must be a JSValue. The good news is that
    // we can attach "opaque data" to a JSValue, so we can still do what we want, just with a layer of misdirection.

    // BUT we can't just arbitrarily attach opaque data. Instead we need to create a "JSClassID", then register
    // that class with the context, then create an instance of that class, THEN attach opaque data to the class specifically.
    // Phew.

    // SO. First of all we grab our class ID (and also ensure it's been registered correctly)
    let class_id = unsafe { ensure_closure_context_class_registered(&in_context.runtime_ref) };

    // We use a weak reference here, because if we don't we'll have a loop: the JSContext internals will
    // hold onto the closure wrapper, and the closure wrapper will hold onto the externals of the JSContext.
    let context_weak = Rc::downgrade(in_context);

    // Then we box the closure we've been sent so that we can turn it into a raw pointer.
    let boxed_closure = Box::new(closure);

    let closure_wrapper = move |closure_context: FunctionInvocationContext| -> JSValue {
        let context = match context_weak.upgrade() {
            Some(val) => val,
            None => {
                // This shouldn't really happen since it shouldn't be possible to to execute
                // a function after the runtime has been destroyed. But all the same, we need to
                // unwrap the weak reference and should handle this error
                return EXCEPTION_RAW;
                // return Err(JSError {
                //     name: "LifetimeError".to_string(),
                //     message: "This JavaScript context has been destroyed".to_string()
                // })
            }
        };

        let result = || {
            if closure_context.number_of_arguments < 1 {
                // more than 1 is OK by the standard JS operates: we just ignore the extras.
                return Err(JSError {
                    name: "ArgumentError".to_string(),
                    message: format!(
                        "Expected 1 argument, got {}",
                        closure_context.number_of_arguments
                    ),
                });
            }

            let arguments_slice = unsafe {
                std::slice::from_raw_parts(
                    closure_context.arguments,
                    closure_context.number_of_arguments as usize,
                )
            };

            let mut arguments_as_values = arguments_slice
                .iter()
                .map(|raw_val| QJSValue::new(*raw_val, &context))
                .collect::<Vec<QJSValue>>();

            // let argument_value =
            //     unsafe { QJSValue::new(*closure_context.arguments.offset(0), &context) };
            let argument_converted = I::from_js_value(arguments_as_values.remove(0)).unwrap();

            let output = boxed_closure(argument_converted);

            let output_as_jsvalue = output.to_js_value(&context).unwrap();

            Ok(output_as_jsvalue)
        };

        match result() {
            Err(js_error) => QJSValue::exception(&context).qjs_ref,
            Ok(jsvalue) => jsvalue.qjs_ref,
        }
    };

    let boxed: Box<FunctionInvokeReceiver> = Box::new(closure_wrapper);
    let raw_pointer = Box::into_raw(Box::new(boxed));

    let mut data_container = unsafe { JS_NewObjectClass(in_context.qjs_ref, class_id as i32) };

    unsafe { JS_SetOpaque(data_container, raw_pointer as *mut std::ffi::c_void) };

    let result = unsafe {
        JS_NewCFunctionData(
            in_context.qjs_ref,
            Some(run_native_closure),
            1,
            0,
            1,
            &mut data_container,
        )
    };
    unsafe { free_value(in_context.qjs_ref, data_container) };
    result
}

// #[cfg(test)]
// mod test {
//     use super::*;
//     use crate::{qjs_context::QJSContext, ref_count::get_ref_count};
//     use esperanto_shared::traits::{JSContext, JSValue};
//     use libquickjs_sys::{
//         JS_DefinePropertyValue, JS_DeleteProperty, JS_FreeAtom, JS_GetGlobalObject, JS_NewAtom,
//         JS_RunGC,
//     };

//     #[test]
//     fn test_a_closure() {
//         let ctx = QJSContext::new().unwrap();
//         let (func, data) = create_function_one_argument(|f: f64| f * 2.0, &ctx.context_ref);

//         let global = unsafe { JS_GetGlobalObject(ctx.context_ref.qjs_ref) };
//         let name = unsafe {
//             JS_NewAtom(
//                 ctx.context_ref.qjs_ref,
//                 std::ffi::CString::new("testFunction").unwrap().as_ptr(),
//             )
//         };
//         // unsafe { JS_DefinePropertyValue(ctx.context_ref.qjs_ref, global, name, func, 0) };

//         // let eval_test = ctx.evaluate("testFunction(2,1)").unwrap();
//         // // let eval_test2 = ctx.evaluate("testFunction(2,1)").unwrap();

//         // let result_number = eval_test.as_number().unwrap();
//         // assert_eq!(result_number, 4.0);
//         unsafe {
//             let success = JS_DeleteProperty(ctx.context_ref.qjs_ref, global, name, 0);
//             println!("Success? {}", success);
//             free_value(ctx.context_ref.qjs_ref, func);
//             free_value(ctx.context_ref.qjs_ref, global);
//             JS_FreeAtom(ctx.context_ref.qjs_ref, name);
//             JS_RunGC(ctx.context_ref.runtime_ref.qjs_ref);
//         }

//         let refs = unsafe { get_ref_count(func) };
//         let data_ref = unsafe { get_ref_count(data) };
//         // let val_refs = unsafe { get_ref_count(eval_test.qjs_ref) };
//         println!("{} {}", refs, data_ref)
//     }
// }

// #[cfg(test)]
// mod test {
//     use libquickjs_sys::{
//         JSCFunction, JSCFunctionData, JSClassID, JSValue, JSValueUnion, JS_DefinePropertyValueStr,
//         JS_GetGlobalObject, JS_GetOpaque, JS_NewCFunction2, JS_NewCFunctionData, JS_NewClassID,
//         JS_NewObjectClass, JS_SetOpaque, JS_TAG_BOOL,
//     };

//     use crate::{
//         qjs_context::QJSContext, qjs_shared_context_ref::SharedQJSContextRef, qjs_value::QJSValue,
//     };
//     use esperanto_shared::traits::{JSContext, JSValue as JSValueTrait};
//     use std::rc::Rc;

//     trait JSConvertible<ValueType: JSValueTrait> {
//         fn as_js_value(&self, in_context: &ValueType::ContextType) -> ValueType;
//     }

//     impl<ValueType: JSValueTrait> JSConvertible<ValueType> for f64 {
//         fn as_js_value(&self, in_context: &ValueType::ContextType) -> ValueType {
//             ValueType::from_number(*self, in_context)
//         }
//     }

//     struct TestClosureContext<ValueType: JSValueTrait> {
//         ctx: Rc<ValueType::ContextType>,
//         closure: Box<dyn Fn(ValueType, &ValueType::ContextType) -> ValueType>,
//     }

//     static mut test: JSClassID = 0;

//     unsafe extern "C" fn what(
//         ctx: *mut libquickjs_sys::JSContext,
//         this_val: libquickjs_sys::JSValue,
//         argc: i32,
//         argv: *mut libquickjs_sys::JSValue,
//         magic: i32,
//         data: *mut libquickjs_sys::JSValue,
//     ) -> libquickjs_sys::JSValue {
//         let d = JS_GetOpaque(*data, test) as *mut TestClosureContext<QJSValue>;

//         let wat = Box::from_raw(d);
//         // let thing = *d;
//         // let wat = d.as_ref().unwrap();
//         let val = QJSValue::new(this_val, &wat.ctx.context_ref);
//         // wat.closure.
//         let result = (wat.closure)(val, &(*d).ctx);
//         return result.qjs_ref;
//     }
//     #[test]
//     fn fiddling_around() {
//         unsafe { JS_NewClassID(&mut test) };
//         let ctx = QJSContext::new().unwrap();
//         let mut obj = unsafe { JS_NewObjectClass(ctx.context_ref.qjs_ref, test as i32) };

//         let rc = Rc::new(ctx);

//         let test_clos = TestClosureContext::<QJSValue> {
//             ctx: rc.clone(),
//             closure: Box::new(|_, ctx| return QJSValue::from_number(1.2, ctx)),
//         };

//         let boxed = Box::new(test_clos);
//         let raw_ptr = Box::into_raw(boxed);

//         unsafe { JS_SetOpaque(obj, raw_ptr as *mut std::ffi::c_void) };

//         let func =
//             unsafe { JS_NewCFunctionData(rc.context_ref.qjs_ref, Some(what), 1, 0, 1, &mut obj) };

//         let global = unsafe { JS_GetGlobalObject(rc.context_ref.qjs_ref) };
//         unsafe {
//             JS_DefinePropertyValueStr(
//                 rc.context_ref.qjs_ref,
//                 global,
//                 std::ffi::CString::new("testFunction").unwrap().as_ptr(),
//                 func,
//                 0,
//             )
//         };

//         rc.evaluate("testFunction.bind('sdfsdf')(1,2)").unwrap();

//         // println!("{}", id)
//         // let hmm: JSCFunctionData = Some(what);

//         // JS_NewCFunction2();
//     }
// }
