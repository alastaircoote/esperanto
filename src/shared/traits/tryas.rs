use crate::{jscontext::JSContext, jsvalue::JSValue, EsperantoResult};

pub trait TryAsRef<T> {
    fn try_as_ref(&self) -> EsperantoResult<&T>;
}

pub trait TryAs<T> {
    fn try_as(&self) -> EsperantoResult<T>;
}

pub trait TryIntoJS<'r, 'c, 'v> {
    fn try_into_js(self, in_context: &'c JSContext<'r, 'c>)
        -> EsperantoResult<JSValue<'r, 'c, 'v>>;
}
