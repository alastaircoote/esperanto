use super::JSValue;

pub trait CreateJSValue<'runtime, 'context, T>: JSValue<'runtime, 'context> {
    fn create(from: T, in_context: &Self::Context) -> &'runtime Self;
}
