use crate::EsperantoResult;

use super::{jscontext::JSContext, jsvalue::JSValue};

pub trait TryAsRef<T> {
    fn try_as_ref(&self) -> EsperantoResult<&T>;
}

pub trait TryAs<T> {
    fn try_as(&self) -> EsperantoResult<T>;
}

// pub trait

pub trait TryIntoJS<'c, Value: JSValue<'c>> {
    fn try_into_js(self, in_context: &'c Value::Context) -> EsperantoResult<Value>;
}

// pub trait IntoJS<'c, Value: JSValue<'c>> {
//     fn into_js(self, in_context: &'c Value::Context) -> Value;
// }

// pub trait

// impl<A, B> TryAs<B> for A
// where
//     A: TryAsRef<B>,
//     B: Copy,
// {
//     fn try_as(&self) -> B {
//         *self.try_as_ref()
//     }
// }
