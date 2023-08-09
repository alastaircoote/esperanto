/**
 * Good morning. So what are we doing here? Much like JavaScriptCore, QuickJS requires
 * the use of "classes" when we want to use custom native code. Unlike JavaScriptCore
 * it has a very simple way of tracking these classes: a u32 class ID. We need to keep
 * track of which u32 maps to which JSExportClass type, hence this file.
 *
 * We take advantage of QuickJS providing opaque storage for runtimes (something JSC
 * doesn't provide, sadly) to store all this alongside the runtime itself.
 */
use std::{any::TypeId, collections::HashMap};

use quickjs_android_suitable_sys::{
    JS_GetRuntimeOpaque, JS_NewClassID, JS_SetClassProto, JS_SetRuntimeOpaque,
};

use crate::{
    quickjs::quickjscontext::QuickJSContextInternal,
    quickjs::quickjsexport::QuickJSExportExtensions,
    quickjs::quickjsruntime::QuickJSRuntimeInternal,
    shared::{context::JSContextInternal, errors::JSExportError},
    EsperantoResult, JSExportClass,
};

// A simple struct to wrap our class IDs. We need to create separate classes for both
// prototypes (where methods, constructors etc are defined) and instances (where we store
// the raw pointers to our Rust structs)
struct StoredClassIDs {
    instance: u32,
    // We actually don't ever refer back to the prototype ID after initial use but it feels
    // worth storing in case we do sometime in the future (store private data in the prototype?
    // we might!)
    _prototype: u32,
}

type ClassIDStorage = HashMap<TypeId, StoredClassIDs>;

fn get_new_class_id() -> u32 {
    let mut prototype_class_id: u32 = 0;
    unsafe { JS_NewClassID(&mut prototype_class_id) };
    return prototype_class_id;
}

fn create_ids<T: JSExportClass>(
    context: QuickJSContextInternal,
    runtime: QuickJSRuntimeInternal,
) -> EsperantoResult<StoredClassIDs> {
    let prototype_class_id = get_new_class_id();
    let instance_class_id = get_new_class_id();

    T::create_prototype_class(*context, prototype_class_id)?;
    T::create_instance_class(runtime, instance_class_id)?;

    let prototype = T::create_prototype(*context, prototype_class_id);

    // Rather than have to specify the prototype each time QuickJS lets us set a class
    // prototype, which it then automatically uses. This means we don't have to keep
    // track of prototype objects ourselves. Which is nice.
    unsafe { JS_SetClassProto(*context, instance_class_id, prototype) };

    Ok(StoredClassIDs {
        instance: instance_class_id,
        _prototype: prototype_class_id,
    })
}

pub(super) fn clear_class<T: JSExportClass>(runtime: QuickJSRuntimeInternal) {
    let storage_ptr = unsafe { JS_GetRuntimeOpaque(runtime) } as *mut ClassIDStorage;
    if storage_ptr.is_null() {
        // clear_class() being called implies we've *defined* a custom class. If we've defined
        // the class but it isn't stored anywhere we're in undefined behaviour territory. Not a lot
        // we can do here since clear_class is called by QuickJS itself. So we should panic:
        panic!("Tried to clear class but there's no storage present");
    }
    let mut boxed = unsafe { Box::from_raw(storage_ptr) };
    boxed.remove(&TypeId::of::<T>());
    if boxed.len() == 0 {
        // If our storage is now empty let's just clear opaque storage entirely. This
        // means we don't have to worry about specifically doing so when dropping a runtime.
        unsafe { JS_SetRuntimeOpaque(runtime, std::ptr::null_mut()) }
    } else {
        unsafe { JS_SetRuntimeOpaque(runtime, Box::into_raw(boxed) as _) }
    }
}

// We have a separate method that specifically does *not* create new class IDs because
// we need this functionality during the prototype finalizer. At that moment we don't
// have a reference to a JSContext to create a prototype, but we also don't want to:
// if a class doesn't already exist inside a finalizer we've got some really weird stuff
// going on.
pub(super) fn get_existing_class_id<T: JSExportClass>(
    runtime: QuickJSRuntimeInternal,
) -> EsperantoResult<Option<u32>> {
    let type_id = TypeId::of::<T>();
    let storage_ptr = unsafe { JS_GetRuntimeOpaque(runtime) } as *mut ClassIDStorage;
    if storage_ptr.is_null() == false {
        let storage_reg = unsafe {
            storage_ptr
                .as_ref()
                .ok_or(JSExportError::UnexpectedBehaviour)?
        };

        return Ok(storage_reg.get(&type_id).map(|s| s.instance));
    }
    Ok(None)
}

// We don't need to define our custom JS classses upfront so at any point we can call
// this method to either grab the existing class or define a new one on demand.
pub(super) fn get_or_create_class_id<T: JSExportClass>(
    context: QuickJSContextInternal,
) -> EsperantoResult<u32> {
    let runtime = context.get_runtime();

    if let Some(existing) = get_existing_class_id::<T>(runtime)? {
        return Ok(existing);
    }

    let type_id = TypeId::of::<T>();

    let ids = create_ids::<T>(context, runtime)?;
    let instance_id = ids.instance;
    let storage_ptr: *mut ClassIDStorage = unsafe { JS_GetRuntimeOpaque(runtime) as _ };

    // If this is the first time we're defining a custom class the storage won't currently
    // exist. No problem, if that's the case we'll make a new one:
    let mut storage = match storage_ptr.is_null() {
        false => unsafe { Box::from_raw(storage_ptr) },
        true => Box::new(HashMap::new()),
    };

    storage.insert(type_id, ids);

    // Convert the storage to a raw pointer with manual memory management and attach it to
    // our runtime:
    let back_to_raw = Box::into_raw(storage);
    unsafe { JS_SetRuntimeOpaque(runtime, back_to_raw as _) }

    return Ok(instance_id);
}
