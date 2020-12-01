// use std::marker::PhantomData;

// use esperanto_engine_shared::{
//     metadata::JSScriptSource,
//     traits::{JSContext, JSRuntime, JSValue, RuntimeCreatesContext},
// };

// pub struct DummyJSValue<'runtime, 'context> {
//     _rt: &'runtime PhantomData<()>,
//     _cx: &'context PhantomData<()>,
// }

// pub struct DummyJSContext<'runtime, 'context> {
//     _rt: &'runtime PhantomData<()>,
//     _cx: &'context PhantomData<()>,
// }

// pub struct DummyJSRuntime<'runtime> {
//     _rt: &'runtime PhantomData<()>,
// }

// impl<'runtime, 'context> JSValue<'runtime, 'context> for DummyJSValue<'runtime, 'context> {
//     type Context = DummyJSContext<'runtime, 'context>;
// }

// impl<'runtime, 'context> JSContext<'runtime, 'context> for DummyJSContext<'runtime, 'context> {
//     type Runtime = DummyJSRuntime<'runtime>;

//     type Value = DummyJSValue<'runtime, 'context>;

//     fn evaluate<'a>(
//         &'context self,
//         script: &'a str,
//         source: Option<JSScriptSource>,
//     ) -> Result<&'runtime Self::Value, esperanto_engine_shared::errors::JSContextError> {
//         todo!()
//     }

//     fn test_take_value(&self, value: &'runtime Self::Value) {
//         todo!()
//     }
// }

// impl<'runtime> JSRuntime<'runtime> for DummyJSRuntime<'runtime> {
//     fn new() -> Result<Self, esperanto_engine_shared::errors::JSRuntimeError> {
//         todo!()
//     }
// }

// impl<'runtime, 'context> RuntimeCreatesContext<'runtime, 'context> for DummyJSRuntime<'runtime> {
//     type Context = DummyJSContext<'runtime, 'context>;

//     fn create_context(
//         &self,
//     ) -> Result<Self::Context, esperanto_engine_shared::errors::JSRuntimeError> {
//         todo!()
//     }
// }
