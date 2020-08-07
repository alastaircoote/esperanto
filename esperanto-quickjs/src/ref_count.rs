// use libquickjs_sys::{JSContext, JSRefCountHeader, JSValue, __JS_FreeValue};

// // stole this from here: https://github.com/theduke/quickjs-rs/blob/96220c78d1e3c7011376ac59dc01ef705ce9acf0/src/bindings.rs

// /// Free a JSValue.
// /// This function is the equivalent of JS_FreeValue from quickjs, which can not
// /// be used due to being `static inline`.
// pub(crate) unsafe fn free_value(context: *mut JSContext, value: JSValue) {
//     // All tags < 0 are garbage collected and need to be freed.
//     if value.tag < 0 {
//         // This transmute is OK since if tag < 0, the union will be a refcount
//         // pointer.
//         let ptr = value.u.ptr as *mut JSRefCountHeader;
//         let pref: &mut JSRefCountHeader = &mut *ptr;
//         pref.ref_count -= 1;
//         if pref.ref_count <= 0 {
//             __JS_FreeValue(context, value);
//         }
//     }
// }

// pub(crate) unsafe fn dup_value(value: JSValue) {
//     // All tags < 0 are garbage collected and need to be freed.
//     if value.tag < 0 {
//         // This transmute is OK since if tag < 0, the union will be a refcount
//         // pointer.
//         let ptr = value.u.ptr as *mut JSRefCountHeader;
//         let pref: &mut JSRefCountHeader = &mut *ptr;
//         pref.ref_count += 1;
//     }
// }

// pub(crate) unsafe fn get_ref_count(value: JSValue) -> i32 {
//     if value.tag >= 0 {
//         return -1;
//     }

//     let ptr = value.u.ptr as *mut JSRefCountHeader;
//     let pref: &mut JSRefCountHeader = &mut *ptr;
//     return pref.ref_count;
// }
