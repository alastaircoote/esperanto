use crate::{jscore_runtime::JSCoreRuntime, jscore_value::JSCoreValue};
use esperanto_engine_shared::errors::JSContextError;
use esperanto_engine_shared::traits::JSContext;
use javascriptcore_sys::OpaqueJSContext;

pub struct JSCoreContext<'runtime, 'context> {
    raw_ref: *mut OpaqueJSContext,
    runtime: &'runtime JSCoreRuntime,
    global_object: &'context str,
}

impl<'runtime, 'context> JSCoreContext<'runtime, 'context> {
    pub(crate) fn wrapping_raw_ref(
        raw_ref: *mut OpaqueJSContext,
        in_runtime: &'runtime JSCoreRuntime,
    ) -> Result<Self, JSContextError> {
        Ok(JSCoreContext {
            raw_ref,
            runtime: in_runtime,
            global_object: "hello",
        })
    }
}

impl<'runtime, 'context> JSContext<'runtime, 'context> for JSCoreContext<'runtime, 'context>
where
    'runtime: 'context,
{
    type Runtime = JSCoreRuntime;

    type Value = JSCoreValue<'runtime, 'context>;

    fn evaluate<'a>(
        &'context self,
        script: &'a str,
        source: Option<esperanto_engine_shared::metadata::JSScriptSource>,
    ) -> Result<&'runtime Self::Value, JSContextError> {
        todo!()
    }
}
