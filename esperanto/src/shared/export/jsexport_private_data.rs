use std::{any::TypeId, ffi::c_void, os::macos::raw};

use crate::{shared::errors::JSExportError, EsperantoResult, JSExportClass};

// We use repr(C) here for not FFI reasons but because it guarantees the order of
// the fields in a struct. When we grab this from JSValue private storage we can't
// actually guarantee it's of type T (hence the methods around that). But if we make
// sure data (which is of variable length) is always the last field we can guarantee
// we're able to read the preceding attributes because they'll always appear in the
// same place in memory.
#[repr(C)]
pub(crate) struct JSExportPrivateData<T> {
    /// Used when creating error messages
    class_name: &'static str,
    type_id: TypeId,
    pub(crate) data: T,
}

impl<T: JSExportClass> JSExportPrivateData<T> {
    pub(crate) fn from_instance(instance: T) -> *mut c_void {
        let wrapped = JSExportPrivateData {
            class_name: T::CLASS_NAME,
            type_id: TypeId::of::<T>(),
            data: instance,
        };
        let boxed = Box::new(wrapped);
        Box::into_raw(boxed) as _
    }

    pub(crate) fn data_from_ptr<'a>(raw_pointer: *mut c_void) -> EsperantoResult<&'a T> {
        let as_ref = unsafe { (raw_pointer as *mut Self).as_ref() }
            .ok_or(JSExportError::CouldNotGetNativeObject(T::CLASS_NAME))?;

        if as_ref.type_id != TypeId::of::<T>() {
            return Err(JSExportError::CouldNotGetNativeObject(T::CLASS_NAME).into());
        }
        return Ok(&as_ref.data);
    }

    pub(crate) fn drop(raw_pointer: *mut c_void) {
        let wrapped = unsafe { Box::from_raw(raw_pointer as *mut Self) };
        if wrapped.type_id != TypeId::of::<T>() {
            // The finalize is triggered by the JS runtime and we have no way of returning an error. So
            // should we ever run into issues here (and we shouldn't!) we have no option but to panic:
            panic!(
                "Tried to finalize an object with the wrong class. Expected {}, got {}",
                T::CLASS_NAME,
                wrapped.class_name
            )
        }
        // This would happen automatically but let's be clear about what we're doing:
        drop(wrapped);
    }
}
