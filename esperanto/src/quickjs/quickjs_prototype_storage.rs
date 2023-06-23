// use std::{
//     any::TypeId,
//     collections::HashMap,
//     convert::TryInto,
//     ffi::{c_void, CString},
//     slice,
// };

// use quickjs_android_suitable_sys::{
//     JSCFunctionEnum_JS_CFUNC_constructor_or_func, JSCFunctionEnum_JS_CFUNC_generic,
//     JSContext as QuickJSContext, JSRuntime as QuickJSRuntime, JSValue as QuickJSValue,
//     JS_DupValue__, JS_GetClassProto, JS_GetOpaque, JS_GetRuntime, JS_GetRuntimeOpaque, JS_GetTag__,
//     JS_IsEqual__, JS_NewCFunction2, JS_NewClass, JS_NewClassID, JS_NewObjectClass,
//     JS_SetClassProto, JS_SetConstructor, JS_SetConstructorBit, JS_SetRuntimeOpaque, JS_NULL__,
//     JS_TAG_OBJECT, JS_TAG_UNDEFINED, JS_UNDEFINED__,
// };

// use super::{
//     quickjscontextpointer::QuickJSContextPointer, quickjsexport::QuickJSExportExtensions,
//     quickjsruntime::QuickJSRuntimeInternal,
// };
// use crate::{
//     shared::{
//         errors::{EsperantoResult, JSExportError},
//         runtime::JSRuntimeError,
//         value::JSValueInternal,
//     },
//     JSContext, JSExportClass, JSRuntime, JSValue,
// };

// type JSClassIDStorage<'a> = HashMap<TypeId, u32>;

// pub(super) fn get_classid_storage<'a>(
//     runtime: QuickJSRuntimeInternal,
// ) -> EsperantoResult<&'a mut JSClassIDStorage<'a>> {
//     // The storage is embedded in the runtime's opaque storage so let's grab it:
//     let storage_ptr = unsafe { JS_GetRuntimeOpaque(runtime) } as *mut JSClassIDStorage;

//     // This shouldn't ever fail because we attach storage when we create a runtime. But it's still
//     // theoretically possible so we check if the raw pointer actually can be converted:
//     unsafe { storage_ptr.as_mut() }.ok_or(JSRuntimeError::FailedToRetrievePrivateContext.into())
// }

// pub(super) fn attach_classid_storage(runtime: QuickJSRuntimeInternal) {
//     // When we create a new runtime we make a new ID store for later use. We need to Box<>
//     // it up in order to store it within QuickJS's opaque storage.

//     let new_storage = JSClassIDStorage::new();
//     let boxed = Box::new(new_storage);
//     let raw_ref = Box::into_raw(boxed);
//     unsafe { JS_SetRuntimeOpaque(runtime, raw_ref as *mut c_void) };
// }

// pub(super) fn drop_classid_storage(runtime: QuickJSRuntimeInternal) {
//     // Once our runtime has been dropped we need to manually drop the ID storage
//     // since it lives inside a raw pointer:
//     let storage_ptr = unsafe { JS_GetRuntimeOpaque(runtime) } as *mut JSClassIDStorage;
//     unsafe { JS_SetRuntimeOpaque(runtime, std::ptr::null_mut()) };
//     let boxed = unsafe { Box::from_raw(storage_ptr) };

//     // Not necessary but let's be explicit about why we're doing this:
//     drop(boxed);
// }

// pub(super) fn get_class_id<T: JSExportClass + 'static>(
//     runtime: *mut QuickJSRuntime,
//     storage: &mut JSClassIDStorage,
// ) -> EsperantoResult<u32> {
//     let addr = TypeId::of::<T>();

//     // QuickJS uses u32 class "IDs" to ensure there aren't ever any
//     // collisions between two different classes. First check: do we
//     // already have a class ID for this class?

//     if let Some(class_id) = storage.get(&addr) {
//         // If yes just directly return it:
//         return Ok(*class_id);
//     }

//     // If not then we need to ask QuickJS to generate a new, unique
//     // ID:
//     let mut class_id: u32 = 0;
//     unsafe { JS_NewClassID(&mut class_id) };

//     // Then create a class from our definition using that ID:
//     let definition = T::class_def()?;
//     unsafe { JS_NewClass(runtime, class_id, &definition) };

//     // Finally we save this ID so that we can use it later
//     storage.insert(addr, class_id);

//     return Ok(class_id);
// }

// /**
//  * This gets called whenever a native class is invoked either as a function (e.g. myClass())
//  * or as a constructor (e.g. new MyClass())
//  */
// fn custom_class_call<'c, T: JSExportClass>(
//     ctx: &'c JSContext<'c>,
//     new_target: QuickJSValue,
//     argc: i32,
//     argv: *mut QuickJSValue,
// ) -> EsperantoResult<QuickJSValue> {
//     // First off we need to check whether this invoke is as a constructor or as
//     // a function. We detect that by seeing if new_target is an object or not.

//     let target_is_constructor = match unsafe { JS_GetTag__(new_target) } {
//         JS_TAG_OBJECT => true,
//         JS_TAG_UNDEFINED => false,
//         _ => {
//             // It's possible that QuickJS might do something entirely unexpected here
//             // so if that's the case let's just exit.
//             return Err(JSExportError::UnexpectedBehaviour.into());
//         }
//     };

//     // Now we convert out raw value pointers into a Vec<JSValueRef>

//     let argc_size: usize = argc
//         .try_into()
//         .map_err(|err| JSExportError::CouldNotConvertArgumentNumber(err))?;

//     let args: Vec<JSValue<'c>> = unsafe { slice::from_raw_parts(argv, argc_size) }
//         .iter()
//         .map(|raw| JSValue::wrap_internal(*raw, ctx))
//         .collect();

//     let result = match (
//         target_is_constructor,
//         T::CALL_AS_CONSTRUCTOR,
//         T::CALL_AS_FUNCTION,
//     ) {
//         (true, Some(constructor), _) => {
//             // This was called as a constructor and we have a constructor. Expected behaviour.
//             (constructor.func)(&args, &ctx)
//         }
//         (true, None, _) => {
//             // This called as a constructor but we don't have one. Unexpected behaviour.
//             Err(
//                 JSExportError::ConstructorCalledOnNonConstructableClass(T::CLASS_NAME.to_string())
//                     .into(),
//             )
//         }
//         (false, _, Some(call_as_function)) => (call_as_function.func)(&args, &ctx),
//         (false, _, None) => {
//             // This called as a constructor but we don't have one. Unexpected behaviour.
//             Err(JSExportError::CalledNonFunctionClass(T::CLASS_NAME.to_string()).into())
//         }
//     };
//     return result.map(|val| val.internal.retain(ctx.internal));
// }

// /**
//  * This is just a tiny wrapper around custom_class_call that means we can use the ? operator there
//  * (since it returns a Result) and then unwrap that for a correct return to QuickJS.
//  */
// pub(super) unsafe extern "C" fn custom_class_call_extern<T: JSExportClass>(
//     ctx: *mut QuickJSContext,
//     func_obj: QuickJSValue,
//     new_target: QuickJSValue,
//     argc: i32,
//     argv: *mut QuickJSValue,
//     flags: i32,
// ) -> QuickJSValue {
//     println!("FLAGS? {}", flags);
//     let context_ptr = QuickJSContextPointer::wrap(ctx, false);
//     let runtime_ptr = JS_GetRuntime(ctx);
//     let runtime = JSRuntime::from_raw(runtime_ptr, false);
//     let context = JSContext::from_raw_storing_runtime(context_ptr, runtime);
//     custom_class_call::<T>(&context, new_target, argc, argv).unwrap_or_else(|e| {
//         let err_val = JSValue::try_new_from(e, &context).unwrap();

//         // We need to do an extra retain here as QuickJS releases the error when passed in. If we don't
//         // have an extra retain it'll immediately free the value then there will be no error and the
//         // JS_UNDEFINED__ value below will actually be returned
//         let retained = err_val.retain();

//         // let error = match e {
//         //     // If it's already a JavaScriptError then let's pass it through directly
//         //     crate::EsperantoError::JavaScriptError(jserr) => {
//         //         JSValue::new_error(&jserr.name, &jserr.message, &context)
//         //     }
//         //     // Otherwise we wrap the whole thing. Might want to revisit this to see how useful
//         //     // the string output of deeply nested errors actually are.
//         //     _ => JSValue::new_error("EsperantoError", &e.to_string(), &context),
//         // }
//         // .unwrap();

//         context_ptr.throw_error(retained.internal);
//         // Once we've thrown an error the return value never actually gets used but we should still
//         // return one anyway, so let's return undefined.
//         JS_UNDEFINED__
//     })
// }

// fn create_base_prototype_function<T: JSExportClass>(
//     name_as_cstring: &CString,
//     in_context: *mut QuickJSContext,
// ) -> EsperantoResult<QuickJSValue> {
//     let proto_arg_length: i32 = match T::CALL_AS_FUNCTION {
//         Some(f) => f.num_args,
//         _ => 0,
//     };

//     let proto = unsafe {
//         JS_NewCFunction2(
//             in_context,
//             None, //Some(custom_class_call_extern::<T>),
//             name_as_cstring.as_ptr(),
//             proto_arg_length,
//             JSCFunctionEnum_JS_CFUNC_generic,
//             0,
//         )
//     };

//     // We need to retain this. Why? Not sure. But it fails if we don't.
//     unsafe { JS_DupValue__(in_context, proto) };
//     return Ok(proto);
// }

// /**
//  * no matter whether our JSExportClass has a constructor or not we still need to
//  * define one because the JS code is always able to call the class as a constructor.
//  * the custom_class_call_extern code will check for the presence of a constructor.
//  */
// fn create_constructor<T: JSExportClass>(
//     name_as_cstring: &CString,
//     prototype: QuickJSValue,
//     in_context: *mut QuickJSContext,
// ) {
//     let ctx = QuickJSContextPointer::wrap(in_context, false);

//     let constructor_arg_length: i32 = match T::CALL_AS_CONSTRUCTOR {
//         Some(f) => f.num_args,
//         _ => 0,
//     };

//     let constructor = unsafe {
//         JS_NewCFunction2(
//             in_context,
//             None, //Some(custom_class_call_extern::<T>),
//             name_as_cstring.as_ptr(),
//             constructor_arg_length,
//             JSCFunctionEnum_JS_CFUNC_constructor_or_func,
//             0,
//         )
//     };
//     unsafe { JS_SetConstructor(in_context, constructor, prototype) }

//     // since this constructor is now attached to the prototype we don't need to hold our
//     // own reference to it, so we release.
//     constructor.release(ctx);
// }

// pub(super) fn get_or_create_class_prototype<T: JSExportClass>(
//     class_id: u32,
//     in_context: *mut QuickJSContext,
// ) -> EsperantoResult<QuickJSValue> {
//     let ctx = QuickJSContextPointer::wrap(in_context, false);

//     // Haven't been able to replicate it but in theory QuickJS could garbage collect the prototype
//     // at some point, so let's check here whether it actually still exists. If not we'll just create
//     // it again.
//     let existing_proto = unsafe { JS_GetClassProto(in_context, class_id) };
//     if unsafe { JS_IsEqual__(existing_proto, JS_NULL__) == 0 } {
//         // it isn't undefined so we assume it is the prototype. Could add extra checks but don't
//         // think they're necessary?
//         return Ok(existing_proto);
//     }

//     let proto = unsafe { JS_NewObjectClass(*ctx, class_id as _) };
//     unsafe { JS_SetConstructorBit(in_context, proto, 1) };
//     let name_as_cstring = CString::new(T::CLASS_NAME)?;

//     // let proto = create_base_prototype_function::<T>(&name_as_cstring, in_context)?;

//     // create_constructor::<T>(&name_as_cstring, proto, in_context);

//     // unsafe { JS_SetClassProto(*ctx, class_id, proto) };

//     return Ok(proto);
// }

// fn finalize_class_instance<T: JSExportClass>(
//     runtime: *mut QuickJSRuntime,
//     value: QuickJSValue,
// ) -> EsperantoResult<()> {
//     let storage = get_classid_storage(runtime)?;

//     let class_id = get_class_id::<T>(runtime, storage)?;
//     let ptr = unsafe { JS_GetOpaque(value, class_id) as *mut T };

//     let obj = unsafe { Box::from_raw(ptr) };
//     drop(obj);
//     Ok(())
// }

// pub(super) unsafe extern "C" fn finalize_class_instance_extern<T: JSExportClass>(
//     runtime: *mut QuickJSRuntime,
//     value: QuickJSValue,
// ) {
//     finalize_class_instance::<T>(runtime, value).unwrap();
// }
