use crate::JSValue;

pub trait HasJSValue<'r, 'c> {
    fn get_value(&'c self) -> &'c JSValue<'r, 'c>;
}

impl<'r, 'c> HasJSValue<'r, 'c> for &JSValue<'r, 'c> {
    fn get_value(&'c self) -> &'c JSValue<'r, 'c> {
        self
    }
}
