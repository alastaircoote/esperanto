// use javascriptcore_sys::JSClassDefinition;

// pub type JSCoreClassDefinition = javascriptcore_sys::JSClassDefinition;

// #[macro_export]
// macro_rules! js_export_function {
//     ($name: ident ($context: ident, $values: ident) $body:block) => {
//         unsafe extern "C" fn $name(
//             ctx: *const javascriptcore_sys::OpaqueJSContext,
//             function: *mut javascriptcore_sys::OpaqueJSValue,
//             this_object: *mut javascriptcore_sys::OpaqueJSValue,
//             argc: usize,
//             argv: *const *const javascriptcore_sys::OpaqueJSValue,
//             exception: *mut *const javascriptcore_sys::OpaqueJSValue,
//         ) -> *const javascriptcore_sys::OpaqueJSValue {
//             use $crate::{EsperantoError, JSContext, JSValueFrom, JSValueRef};

//             let context: JSContext = JSContext::from(ctx);
//             let as_ref = &context;

//             let mut values_vec = Vec::<JSValueRef>::with_capacity(argc);

//             std::slice::from_raw_parts(argv, argc)
//                 .iter()
//                 .for_each(|raw| {
//                     values_vec.push(JSValueRef::from((*raw, &context)));
//                 });

//             fn inner<'a>(
//                 $context: &'a JSContext,
//                 $values: Vec<JSValueRef>,
//             ) -> Result<JSValueRef<'a>, EsperantoError> {
//                 $body
//             }
//             let result = match inner(&context, values_vec) {
//                 Ok(success_jsval) => return success_jsval.into(),
//                 Err(error) => {
//                     let error_val: JSValueRef = JSValueRef::new_value_from(error, &context).into();
//                     error_val.into()
//                 }
//             };
//             drop(context);
//             result
//         }
//     };
// }

// #[macro_export]
// macro_rules! js_export_class {
//     ($class_ident: ident, $class_export_name: literal,{
//         call_as_function: $call_as_function:expr
//         // static_functions: {$($static_fn_export_name: literal : $static_fn_ident:path),*}
//     }) => {
//         impl $crate::JSExportClass for $class_ident {
//             const CLASS_DEFINITION: $crate::private::JSClassDefinition =
//                 $crate::private::JSClassDefinition {
//                     className: concat!($class_export_name, "\0").as_ptr() as *const i8,
//                     attributes: 0,
//                     callAsConstructor: None,
//                     callAsFunction: $call_as_function,
//                     convertToType: None,
//                     deleteProperty: None,
//                     finalize: None,
//                     initialize: None,
//                     getProperty: None,
//                     getPropertyNames: None,
//                     hasInstance: None,
//                     hasProperty: None,
//                     parentClass: std::ptr::null_mut(),
//                     setProperty: None,
//                     staticFunctions: std::ptr::null_mut(),
//                     staticValues: std::ptr::null_mut(),
//                     version: 1,
//                 };

//             const PROTOTYPE_DEFINITION: Option<$crate::private::JSClassDefinition> = None;
//         }
//     };
// }
