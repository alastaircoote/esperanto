use crate::worker::WorkerError;
use crate::Worker;
use esperanto_traits::js_traits::JSRuntime;
use esperanto_traits::js_traits::JSValue;
use std::sync::Arc;

pub struct JSValueWrapper<Runtime: JSRuntime + 'static> {
    worker: Arc<Worker<Runtime>>,
    js_value_key: Runtime::StoreKey,
}

impl<Runtime: JSRuntime + 'static> JSValueWrapper<Runtime> {
    pub async fn to_string<'b>(&self) -> Result<&'b str, WorkerError> {
        let key = self.js_value_key;
        self.worker
            .enqueue(move |r| {
                let jsvalue = r.get_value_ref(key).unwrap();
                jsvalue
                    .to_string()
                    .map_err(|e| WorkerError::InternalRuntimeError(e))
            })
            .await?
    }

    pub fn new(
        from_key: Runtime::StoreKey,
        in_worker: Arc<Worker<Runtime>>,
    ) -> JSValueWrapper<Runtime> {
        return JSValueWrapper {
            worker: in_worker,
            js_value_key: from_key,
        };
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::test_util::{DummyJSRuntime, DummyJSValue};

    #[tokio::test]
    async fn it_wraps_successfully<'a>() {
        let worker = Worker::<DummyJSRuntime>::new().await.unwrap();

        let key = worker
            .enqueue(move |r| {
                let js_val = DummyJSValue::new("this is a string");
                r.store_value(js_val)
            })
            .await
            .unwrap();

        let wrapper = JSValueWrapper::new(key, Arc::new(worker));

        let result = wrapper.to_string().await.unwrap();
        assert_eq!(result, "this is a string");
    }
}
