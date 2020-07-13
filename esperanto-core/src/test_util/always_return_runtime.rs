use esperanto_traits::js_traits::{JSEnvError, JSRuntime, JSValue};

pub trait ConstructableJSValue: JSValue {
    fn new() -> Self;
}

pub struct AlwaysReturnRuntime<ReturnValue: ConstructableJSValue> {
    dummy: std::marker::PhantomData<ReturnValue>,
}

impl<ReturnValue: ConstructableJSValue + Send + Sync + 'static> JSRuntime
    for AlwaysReturnRuntime<ReturnValue>
{
    type ValueType = ReturnValue;
    fn evaluate(&self, _: &str) -> Result<Self::ValueType, JSEnvError> {
        return Ok(ReturnValue::new());
    }
    fn new() -> Self {
        return AlwaysReturnRuntime::<ReturnValue> {
            dummy: std::marker::PhantomData::default(),
        };
    }
}
