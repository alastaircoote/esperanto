use crate::{
    shared::value::JSValueImplementation, EsperantoResult, JSExportClass, JSValue, Retain,
};
use std::ops::Deref;

#[derive(Debug)]
pub struct Js<'r, 'c: 'r, T: JSExportClass> {
    value: Retain<JSValue<'r, 'c>>,
    reference: &'c T,
}

impl<'r, 'c, T> Js<'r, 'c, T>
where
    T: JSExportClass,
    'c: 'r,
{
    pub(crate) fn new(wrapping: Retain<JSValue<'r, 'c>>) -> EsperantoResult<Self> {
        let re = wrapping
            .internal
            .get_native_ref::<T>(wrapping.context.implementation())?;

        let created: Js<T> = Js {
            value: wrapping,
            reference: re,
        };

        Ok(created)
    }

    pub fn get_jsvalue(instance: &Self) -> &JSValue<'r, 'c> {
        &instance.value
    }
}

impl<'r, 'c, T> Deref for Js<'r, 'c, T>
where
    T: JSExportClass,
    'c: 'r,
{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.reference
    }
}

impl<'r, 'c, T> AsRef<T> for Js<'r, 'c, T>
where
    T: JSExportClass,
    'c: 'r,
{
    fn as_ref(&self) -> &T {
        self.deref()
    }
}
