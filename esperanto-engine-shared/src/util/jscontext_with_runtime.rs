use std::ops::Deref;

use crate::traits::JSContext;

pub struct JSContextWithRuntime<'runtime, 'context, Context: JSContext<'runtime, 'context>> {
    context: Context,
    runtime: Context::Runtime,
}

impl<'runtime, 'context, Context: JSContext<'runtime, 'context>> Deref
    for JSContextWithRuntime<'runtime, 'context, Context>
{
    type Target = Context;

    fn deref(&self) -> &Self::Target {
        &self.context
    }
}
