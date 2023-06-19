use crate::{JSValue, Retain};

pub trait HasJSValue {
    fn get_value<'a>(&'a self) -> &'a JSValue<'a>;
}

impl<'c> HasJSValue for &JSValue<'c> {
    fn get_value<'a>(&'a self) -> &'a JSValue<'a> {
        self
    }
}
