use std::convert::TryFrom;

use crate::{
    shared::{errors::EsperantoResult, retain::Retain},
    EsperantoError, JSValue,
};

pub type JSResultProcessor<'c, T> = for<'a> fn(&'a JSValue<'c>) -> EsperantoResult<T>;

pub fn convert<'c, T: for<'a> TryFrom<&'a JSValue<'c>, Error = EsperantoError>>(
    value: &JSValue<'c>,
) -> EsperantoResult<T> {
    return T::try_from(value);
}

pub fn retain<'c>(value: &JSValue<'c>) -> EsperantoResult<Retain<JSValue<'c>>> {
    return Ok(Retain::new(value, false));
}
