use crate::{jscontext::JSContext, jsvalue::JSValue, EsperantoResult};

pub trait TryAsRef<T> {
    fn try_as_ref(&self) -> EsperantoResult<&T>;
}

pub trait TryAs<T> {
    fn try_as(&self) -> EsperantoResult<T>;
}

pub trait TryIntoJS<'c> {
    fn try_into_js(self, in_context: &'c JSContext) -> EsperantoResult<JSValue<'c>>;
}
