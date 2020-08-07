use jni::objects::{JClass, JString};
use jni::{
    sys::jlong,
    JNIEnv,
};

use esperanto_shared::traits::JSContext;
use crate::implementation::Context;
use std::rc::Rc;
use android_logger::Config;
use log::Level;

#[no_mangle]
pub extern "system" fn Java_org_esperanto_esperanto_JSContextPrivate_new(
    _env: JNIEnv,
    _class: JClass,
) -> jlong {
    android_logger::init_once(
        Config::default().with_min_level(Level::Trace),
    );
    let ctx = Context::new().unwrap();
    let ptr = Box::into_raw(Box::new(ctx));
    ptr as _
}

#[no_mangle]
pub extern "system" fn Java_org_esperanto_esperanto_JSContextPrivate_evaluate(
    env: JNIEnv,
    _class: JClass,
    ctx_ptr: jlong,
    script_ptr: JString
) -> jlong {
    let ctx = unsafe { Box::from_raw(ctx_ptr as *mut Rc<Context>) };
  
    let script:String = env.get_string(script_ptr).unwrap().into();

    let value = ctx.evaluate(&script).unwrap();
    Box::into_raw(ctx);
    Box::into_raw(Box::new(value)) as _
}
