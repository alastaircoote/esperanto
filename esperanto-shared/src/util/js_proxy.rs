// use std::{collections::HashMap, ops::Deref};

// use crate::traits::JSContext;

// struct Js<ContextType: JSContext, T> {
//     wrapped_object: T,
//     jsvalues: HashMap<ContextType::ValueShareTarget, ContextType::ValueType>,
// }

// impl<ContextType: JSContext, T> Js<ContextType, T> {
//     pub fn new(x: T) -> Js<ContextType, T> {
//         Js {
//             wrapped_object: x,
//             jsvalues: HashMap::new(),
//         }
//     }

// }

// impl<ContextType: JSContext, T> Deref for Js<ContextType, T> {
//     type Target = T;

//     fn deref(&self) -> &T {
//         &self.wrapped_object
//     }
// }

// // fn hm() {
// //     let st = Js::<DummyJSContext>::new("hello");
// // }
