// use std::ops::Deref;

// use crate::JSValueRef;

// pub trait Retainable {
//     fn retain(&self) -> Self;
//     fn release(&mut self);
// }

// #[derive(Debug)]
// pub struct Retain<T: Retainable> {
//     pub(crate) retained_value: T,
// }

// impl<T: Retainable> Drop for Retain<T> {
//     fn drop(&mut self) {
//         T::release(&mut self.retained_value)
//     }
// }

// impl<T: Retainable> Retain<T> {
//     pub fn new(retaining: T) -> Self {
//         let retained = T::retain(&retaining);
//         Retain {
//             retained_value: retained,
//         }
//     }

//     pub(crate) fn wrap_already_retained(retained: T) -> Self {
//         Retain {
//             retained_value: retained,
//         }
//     }

//     // pub fn get<'a>(from_retainer: Self) -> &'a T {
//     //     &from_retainer.inner_value
//     // }
// }

// impl<T: Retainable> Deref for Retain<T> {
//     type Target = T;

//     fn deref(&self) -> &Self::Target {
//         &self.retained_value
//     }
// }

// // impl<'a> From<Retain<JSValueRef<'a>>> for JSValueRef<'a> {
// //     fn from(val: Retain<JSValueRef<'a>>) -> Self {
// //         val.retained_value
// //     }
// // }
