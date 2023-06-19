// // use std::intrinsics::transmute;

// use std::slice::Iter;

// use esperanto::{js_export_class, js_export_function, JSValueRef};

// use javascriptcore_sys::{JSClassDefinition, JSGlobalContextCreate, OpaqueJSContext};

// // // fn wat<'a>() -> &'a String {
// // //     let hm = "".to_string();
// // //     &hm
// // // }

// // // impl javascriptcore_sys::OpaqueJSContextGroup {}

// // trait Blerg {
// //     fn new() -> String;
// // }

// // struct What {
// //     // ptr: *mut String,
// // }

// // fn what() {
// //     let test = vec![1, 2];
// //     let wat = test.iter().map(|i| i * 2);
// //     let h = wat.into_iter();
// //     let hm = wat.collect();
// // }

// struct Thing {}

// impl Thing {
//     js_export_function! {
//         fn export_name<'c>(arr: Vec<JSValueRef<'c>>) -> JSValueRef<'c> {
//             return arr[0];
//         }
//     }
// }

// js_export_class!(Thing, "Thing", {
//     call_as_function: Some(Thing::export_name)
//     // static_functions: {
//     //     "testFunction" : Thing::export_name
//     // }
// });

// // #[test]
// // fn test() {
// //     let one = "one".to_string();
// //     let two = "two".to_string();

// //     let result = take_strings(&one, &two);

// //     let s = result.str_one;

// //     unsafe { println!("{}", *s) };
// // }
