use crate::implementation::Value;
use esperanto_shared::traits::JSValue;
use jni::{
    objects::{JClass, JString},
    sys::{jdouble, jlong, jstring},
    JNIEnv,
};
use log::debug;

// struct RevivedPointer<T> {
//     value: Box<T>
// }

// impl<T> RevivedPointer<T> {
//     fn from_ptr(ptr: i64) -> Self{
//         RevivedPointer {
//             value: unsafe {Box::from_raw(ptr as *mut T) }
//         }
//     }
// }

// impl<T> Drop for RevivedPointer<T> {
//     fn drop(&mut self) {
//         debug!("DROPPING!!");
//         std::mem::replace(&mut self.value, unsafe { Box::new(std::mem::MaybeUninit::uninit().assume_init()) });
//         std::mem::forget(self)
//     }

// }

#[no_mangle]
pub extern "system" fn Java_org_esperanto_esperanto_JSValuePrivate_asString(
    env: JNIEnv,
    _class: JClass,
    val_ptr: jlong,
) -> jstring {
    // let revived: RevivedPointer<Value> = RevivedPointer::from_ptr(val_ptr);
    let value = unsafe { Box::from_raw(val_ptr as *mut Value) };
    let str = value.as_string().unwrap();
    let j_str_ptr = env.new_string(str).unwrap();
    Box::into_raw(Box::new(value));
    j_str_ptr.into_inner()
}

#[no_mangle]
pub extern "system" fn Java_org_esperanto_esperanto_JSValuePrivate_asNumber(
    _env: JNIEnv,
    _class: JClass,
    val_ptr: jlong,
) -> jdouble {
    debug!("asNumber requested");
    let value = unsafe { Box::from_raw(val_ptr as *mut Value) };
    let num = value.as_number().unwrap();
    Box::into_raw(Box::new(value));
    num
}

#[no_mangle]
pub extern "system" fn Java_org_esperanto_esperanto_JSValuePrivate_getProperty(
    env: JNIEnv,
    _class: JClass,
    val_ptr: jlong,
    property_name_str: JString,
) -> jlong {
    debug!("getProperty requested");
    let property_name: String = env.get_string(property_name_str).unwrap().into();
    let value = unsafe { Box::from_raw(val_ptr as *mut Value) };
    let property = value.get_property(&property_name).unwrap();
    Box::into_raw(Box::new(value));
    Box::into_raw(Box::new(property)) as _
}

#[no_mangle]
pub extern "system" fn Java_org_esperanto_esperanto_JSValuePrivate_call(
    _env: JNIEnv,
    _class: JClass,
    val_ptr: jlong,
    bound_to: jlong,
) -> jlong {
    debug!("call requested");
    // unsafe {(val_ptr as *mut Value).as_ref()}.get_property("sdf");
    let value: &Value = to_ref(val_ptr).unwrap();
    // let value = unsafe {Box::from_raw(val_ptr as *mut Value) };
    let bound = unsafe { Box::from_raw(bound_to as *mut Value) };
    let result = value.call_bound(Vec::new(), &bound).unwrap();
    Box::into_raw(Box::new(value));
    Box::into_raw(Box::new(bound));
    // let result = revived.value.call_bound(Vec::new(), &revived_bound.value).unwrap();
    Box::into_raw(Box::new(result)) as _
}
#[derive(Debug)]
enum Errors {
    OhNo,
}

fn to_ref<'a, T>(ptr_long: jlong) -> Result<&'a T, Errors> {
    unsafe { (ptr_long as *mut T).as_ref().ok_or(Errors::OhNo) }
}
