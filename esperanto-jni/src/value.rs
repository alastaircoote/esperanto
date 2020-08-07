
use jni::{objects::JClass, sys::{jlong, jstring, jdouble}, JNIEnv};
use crate::implementation::Value;
use esperanto_shared::traits::JSValue;

#[no_mangle]
pub extern "system" fn Java_org_esperanto_esperanto_JSValuePrivate_asString(
    env: JNIEnv,
    _class: JClass,
    val_ptr: jlong,
) -> jstring {
    let value = unsafe { Box::from_raw(val_ptr as *mut Value) };
    let str = value.as_string().unwrap();
    let j_str_ptr = env.new_string(str).unwrap();
    j_str_ptr.into_inner()
}

#[no_mangle]
pub extern "system" fn Java_org_esperanto_esperanto_JSValuePrivate_asNumber(
    _env: JNIEnv,
    _class: JClass,
    val_ptr: jlong,
) -> jdouble {
    let value = unsafe { Box::from_raw(val_ptr as *mut Value) };
    let num = value.as_number().unwrap();
    num
}