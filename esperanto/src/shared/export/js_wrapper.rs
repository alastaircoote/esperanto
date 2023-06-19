use std::marker::PhantomData;

use lazy_static::__Deref;

use crate::{shared::value::JSValueInternal, EsperantoResult, JSExportClass, JSValue, Retain};
use std::ops::Deref;

#[derive(Debug)]
pub struct Js<'c, T: JSExportClass> {
    value: Retain<JSValue<'c>>,
    _phantom: PhantomData<T>,
    // reference: &'c T,
}

impl<'c, T> Js<'c, T>
where
    T: JSExportClass,
{
    pub(crate) fn new(wrapping: Retain<JSValue<'c>>) -> EsperantoResult<Self> {
        // This is weird. But Deref can't fail, so we make sure we can successfully
        // grab the reference when we first create the wrapper and assume that subsequent
        // attempts to access it will also work. Maybe something to revisit down the line.

        let created: Js<T> = Js {
            value: wrapping,
            _phantom: PhantomData, // reference,
        };

        _ = created.get_native_ref()?;

        Ok(created)
    }

    fn get_native_ref(&self) -> EsperantoResult<&T> {
        self.value
            .internal
            .get_native_ref(self.value.context.internal)
    }

    pub fn get_value(instance: &Self) -> &JSValue<'c> {
        &instance.value
    }
}

impl<'c, T> Deref for Js<'c, T>
where
    T: JSExportClass,
{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.get_native_ref().unwrap()
    }
}

impl<'c, T> AsRef<T> for Js<'c, T>
where
    T: JSExportClass,
{
    fn as_ref(&self) -> &T {
        self.deref()
    }
}