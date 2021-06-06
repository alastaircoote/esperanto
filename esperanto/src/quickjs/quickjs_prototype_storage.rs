use std::{
    collections::HashMap,
    ffi::{c_void, CStr, CString},
};

use quickjs_android_suitable_sys::{
    JSCFunctionEnum_JS_CFUNC_generic, JSContext as QuickJSContext, JSValue as QuickJSValue,
    JS_ATOM_Symbol_toPrimitive, JS_DupValue__, JS_GetRuntime, JS_GetRuntimeOpaque, JS_IsEqual__,
    JS_NewCFunction2, JS_NewClass, JS_NewClassID, JS_NewObjectProtoClass, JS_NewString,
    JS_SetConstructorBit, JS_SetPropertyInternal, JS_SetRuntimeOpaque,
};

use super::{quickjscontextpointer::QuickJSContextPointer, quickjsexport::QuickJSExportExtensions};
use crate::{
    export::JSExportMetadata,
    shared::{context::JSContextInternal, errors::EsperantoResult, value::JSValueInternal},
    JSExportClass,
};

type PrototypeIdentifier = (JSExportMetadata, *mut QuickJSContext);

struct JSContextPrototypeStorage {
    class_ids: HashMap<JSExportMetadata, u32>,
    in_use_prototypes: HashMap<PrototypeIdentifier, QuickJSValue>,
}

pub(super) fn get_or_create_prototype<T: JSExportClass>(
    in_context: *mut QuickJSContext,
) -> EsperantoResult<QuickJSValue> {
    let identifier: PrototypeIdentifier = (T::METADATA, in_context);
    let runtime = unsafe { JS_GetRuntime(in_context) };

    let storage_ptr = unsafe { JS_GetRuntimeOpaque(runtime) } as *mut JSContextPrototypeStorage;

    let storage = match unsafe { storage_ptr.as_mut() } {
        Some(r) => r,
        None => {
            let new_storage = JSContextPrototypeStorage {
                class_ids: HashMap::new(),
                in_use_prototypes: HashMap::new(),
            };
            let boxed = Box::new(new_storage);
            let raw_ref = Box::into_raw(boxed);
            unsafe { JS_SetRuntimeOpaque(runtime, raw_ref as *mut c_void) };
            // we know this pointer isn't null because we just created it, so unwrap
            unsafe { raw_ref.as_mut().unwrap() }
        }
    };

    let ctx: QuickJSContextPointer = in_context.into();

    if let Some(existing_prototype) = storage.in_use_prototypes.get(&identifier) {
        println!("RETURN EXISTING PROTOTYPE");
        return unsafe { Ok(JS_DupValue__(in_context, *existing_prototype)) };
    }

    println!("MAKE NEW PROTOTYPE: {:?} {:?}", identifier.0, identifier.1);
    let global = ctx.get_globalobject();
    let object = global.get_property(ctx, &CString::new("Object").unwrap())?;

    let class_id = match storage.class_ids.get(&T::METADATA) {
        Some(class_id) => *class_id,
        None => {
            let runtime = unsafe { JS_GetRuntime(*ctx) };
            let proto_def = T::prototype_def();
            let proto_class_id = unsafe { JS_NewClassID(&mut 0) };
            check_quickjs_exception!(ctx => {
                unsafe { JS_NewClass(runtime, proto_class_id, &proto_def) };
            })?;
            storage.class_ids.insert(T::METADATA, proto_class_id);
            proto_class_id
        }
    };

    let proto = unsafe { JS_NewObjectProtoClass(*ctx, object, class_id) };

    const TO_STRING_NAME: &[u8] = b"toString";
    let to_string = unsafe {
        JS_NewCFunction2(
            *ctx,
            Some(return_class_tostring::<T>),
            TO_STRING_NAME.as_ptr() as *const i8,
            8,
            JSCFunctionEnum_JS_CFUNC_generic,
            0,
        )
    };

    object.release(ctx);
    global.release(ctx);

    unsafe { JS_SetConstructorBit(*ctx, proto, 1) };

    // set property releases the value it's given so no need to release this
    // let fff =
    //     QuickJSValue::new_function(&CString::new("return 'hello'").unwrap(), &Vec::new(), ctx)
    //         .unwrap();

    unsafe { JS_SetPropertyInternal(*ctx, proto, JS_ATOM_Symbol_toPrimitive, to_string, 0) };
    // to_string.release(ctx);
    storage.in_use_prototypes.insert(identifier, proto);
    Ok(proto)
}

unsafe extern "C" fn return_class_tostring<T: JSExportClass>(
    ctx: *mut quickjs_android_suitable_sys::JSContext,
    _this: QuickJSValue,
    _argc: i32,
    _argv: *mut QuickJSValue,
) -> QuickJSValue {
    let cstr = CStr::from_ptr(T::METADATA.class_name as *const i8)
        .to_str()
        .unwrap();
    let hm = format!("class {} {{}}", cstr);
    let cstring = CString::new(hm).unwrap();

    JS_NewString(ctx, cstring.as_ptr())
}

pub(super) unsafe extern "C" fn delete_stored_prototype<T: JSExportClass>(
    runtime: *mut quickjs_android_suitable_sys::JSRuntime,
    value: QuickJSValue,
) {
    println!("DELETE PROTOTYPE");
    let storage_ptr = JS_GetRuntimeOpaque(runtime) as *mut JSContextPrototypeStorage;
    let store = match storage_ptr.as_mut() {
        Some(exists) => exists,
        _ => {
            panic!("Tried to remove a prototype when it isn't stored. This should never happen.")
        }
    };

    let stored_value = {
        store
            .in_use_prototypes
            .iter()
            .find_map(|(key, iter_value)| match JS_IsEqual__(value, *iter_value) {
                1 => Some(*key),
                _ => None,
            })
    };
    // .find(|(_, iter_value)| JS_IsEqual__(value, **iter_value) == 1);

    match stored_value {
        None => {
            panic!("Tried to remove a prototype when it isn't stored. This should never happen.");
        }
        Some(key) => store.in_use_prototypes.remove(&key),
    };
}
