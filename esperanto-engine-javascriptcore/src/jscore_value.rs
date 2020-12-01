use esperanto_engine_shared::traits::{CreateJSValue, JSValue};

use crate::jscore_context::JSCoreContext;

pub struct JSCoreValue<'runtime, 'context> {
    context: &'context JSCoreContext<'runtime, 'context>,
}

impl<'runtime, 'context> JSValue<'runtime, 'context> for JSCoreValue<'runtime, 'context> {
    type Context = JSCoreContext<'runtime, 'context>;
}

impl<'runtime, 'context> CreateJSValue<'runtime, 'context, f64>
    for JSCoreValue<'runtime, 'context>
{
    fn create(from: f64, in_context: &Self::Context) -> &'runtime Self {
        todo!()
    }
}
