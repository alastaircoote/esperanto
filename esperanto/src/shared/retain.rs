use super::value::JSValueInternal;
use crate::JSValue;
use std::ops::Deref;

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
    pub fn new(retaining: T, already_retained: bool) -> Self {
        let retained = match already_retained {
            false => T::retain(&retaining),
            true => retaining,
        };
        Retain {
            retained_value: retained,
        }
    }

    pub fn value(&self) -> &T {
        &self.retained_value
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

// impl<'c, T: Retainable> TryInto<T> for Retain<JSValue<'c>>
// where
//     T: TryFrom<JSValue<'c>>,
// {
//     type Error = EsperantoError;

//     fn try_into(self) -> Result<Retain<T>, Self::Error> {
//         todo!()
//     }
// }

// impl<'c, T> TryFromJSValue for Retain<JSValue<'c>>
// where
//     T: Retainable,
//     T: TryFromJSValue,
// {
//     fn try_from(value: &JSValue<'c>) -> EsperantoResult<Self> {
//         todo!()
//     }
// }

// impl<'a> From<Retain<JSValue<'a>>> for JSValue<'a> {
//     fn from(val: Retain<JSValue<'a>>) -> Self {
//         val.retained_value
//     }
// }

// impl<'c, T> TryInto<T> for Retain<JSValue<'c>> {
//     type Error;

//     fn try_into(self) -> Result<T, Self::Error> {
//         todo!()
//     }
// }
