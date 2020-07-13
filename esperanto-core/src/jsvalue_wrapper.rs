use crate::worker::WorkerError;
use crate::Worker;
use esperanto_traits::js_traits::JSRuntime;
use esperanto_traits::js_traits::JSValue;
use std::sync::Arc;

pub struct JSValueWrapper<Runtime: JSRuntime + Send + 'static> {
    worker: Arc<Worker<Runtime>>,
    js_value: Arc<Runtime::ValueType>,
}

impl<Runtime: JSRuntime + Send + 'static> JSValueWrapper<Runtime> {
    pub async fn to_string<'b>(&self) -> Result<&'b str, WorkerError> {
        let jsvalue = self.js_value.clone();
        self.worker
            .enqueue(move |_| {
                jsvalue
                    .to_string()
                    .map_err(|e| WorkerError::InternalRuntimeError(e))
            })
            .await?
    }

    fn new(
        wrapping: Arc<Runtime::ValueType>,
        in_worker: Arc<Worker<Runtime>>,
    ) -> JSValueWrapper<Runtime> {
        return JSValueWrapper {
            worker: in_worker,
            js_value: wrapping,
        };
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::test_util::{DummyJSRuntime, DummyJSValue};

    #[tokio::test]
    async fn it_wraps_successfully<'a>() {
        let js_val = DummyJSValue::new(&"this is a string");
        let worker = Worker::<DummyJSRuntime>::new().await.unwrap();
        let wrapper = JSValueWrapper::new(Arc::new(js_val), Arc::new(worker));
        let result = wrapper.to_string().await.unwrap();
        assert_eq!(result, "this is a string");
    }
}
