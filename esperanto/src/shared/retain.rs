use std::ops::Deref;

use crate::JSValue;

use super::value::JSValueInternal;

pub trait Retainable {
    fn retain(&self) -> Self;
    fn release(&mut self);
    fn deref(&self) -> Self;
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
    pub fn new(retaining: &T, already_retained: bool) -> Self {
        let retained = match already_retained {
            false => T::retain(retaining),
            true => T::deref(retaining),
        };
        Retain {
            retained_value: retained,
        }
    }
}

impl<'c> Retainable for JSValue<'c> {
    fn retain(&self) -> Self {
        return JSValue::wrap_internal(self.internal.retain(self.context.internal), self.context);
    }

    fn release(&mut self) {
        self.internal.release(self.context.internal);
    }

    fn deref(&self) -> Self {
        JSValue::wrap_internal(self.internal, self.context)
    }
}

impl<T: Retainable> Deref for Retain<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.retained_value
    }
}

// // impl<'a> From<Retain<JSValueRef<'a>>> for JSValueRef<'a> {
// //     fn from(val: Retain<JSValueRef<'a>>) -> Self {
// //         val.retained_value
// //     }
// // }
