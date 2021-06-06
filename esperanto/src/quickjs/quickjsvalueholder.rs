// use std::{marker::PhantomData, ops::Deref};

// use quickjs_android_suitable_sys::JSValue;

// #[derive(Debug, Copy, Clone)]
// pub struct QuickJSValueHolder<'c> {
//     value: JSValue,
//     _lifetime: PhantomData<&'c ()>,
// }

// impl<'c> From<JSValue> for QuickJSValueHolder<'c> {
//     fn from(value: JSValue) -> Self {
//         QuickJSValueHolder {
//             value,
//             _lifetime: PhantomData,
//         }
//     }
// }

// // impl<'c> From<QuickJSValueHolder<'c>> for JSValue {
// //     fn from(holder: QuickJSValueHolder<'c>) -> Self {
// //         holder.value
// //     }
// // }

// impl Deref for QuickJSValueHolder<'_> {
//     type Target = JSValue;

//     fn deref(&self) -> &Self::Target {
//         &self.value
//     }
// }
