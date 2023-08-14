use super::value::{HasJSValue, JSValueImplementation};
use crate::JSValue;
use std::ops::Deref;

pub trait Retainable {
    // fn retain(&self) -> Self;
    fn release(&mut self);
    // fn deref(&self) -> Self;
}

#[derive(Debug)]
pub struct Retain<T: Retainable> {
    pub(crate) retained_value: T,
}

impl<T: Retainable> Drop for Retain<T> {
    fn drop(&mut self) {
        T::release(&mut self.retained_value)
    }
}

impl<T: Retainable> Retain<T> {
    pub fn wrap(retained_value: T) -> Self {
        Retain { retained_value }
    }

    pub fn value(&self) -> &T {
        &self.retained_value
    }
}

impl Retainable for JSValue<'_, '_> {
    // fn retain(&self) -> Self {
    //     let new_retained_value = self.internal.retain(self.context.internal);
    //     return JSValue::wrap_internal(new_retained_value, self.context);
    // }

    fn release(&mut self) {
        self.internal.release(self.context.implementation());
    }

    // fn deref(&self) -> Self {
    //     JSValue::wrap_internal(self.internal, self.context)
    // }
}

impl<T: Retainable> Deref for Retain<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.retained_value
    }
}

impl<T> AsRef<T> for Retain<T>
where
    <Retain<T> as Deref>::Target: AsRef<T>,
    T: Retainable,
{
    fn as_ref(&self) -> &T {
        self.deref().as_ref()
    }
}

// Slightly unclear on this one though I'm probably overthinking it. One retain
// isn't actually equal to another (they're separate retains) but when we're doing
// this comparison 99% of the time (100% of the time?) we're actually trying to
// check the equality of the underlying JSValues so it makes sense for this convenience
// method to exist.
impl<T> PartialEq for Retain<T>
where
    T: Retainable,
    // T: AsRef<T>,
    T: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.retained_value == other.retained_value
    }
}

impl<'r, 'c> HasJSValue<'r, 'c> for Retain<JSValue<'r, 'c>> {
    fn get_value(&'c self) -> &'c JSValue<'r, 'c> {
        &self.retained_value
    }
}
