use crate::worker::WorkerError;
use crate::Worker;
use esperanto_traits::errors::JSConversionError;
use esperanto_traits::JSContext;
use std::convert::TryFrom;
use std::sync::Arc;
pub struct JSValueWrapper<Runtime: JSContext + 'static> {
    worker: Arc<Worker<Runtime>>,
    js_value_key: Runtime::StoreKey,
}

impl<Runtime: JSContext + 'static> JSValueWrapper<Runtime> {
    pub async fn try_into<O: 'static + Send + Default + TryFrom<Runtime::ValueType>>(
        &self,
    ) -> Result<O, WorkerError> {
        let key = self.js_value_key;
        let value = self
            .worker
            .enqueue(move |r| {
                let jsvalue = r.pull_value(key).unwrap();
                O::try_from(jsvalue).map_err(|_| JSConversionError::ConversionFailed)
            })
            .await?;

        value.map_err(|e| WorkerError::ConversionError(e))
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
    use crate::test_util::{DummyJSContext, DummyJSValue};

    #[tokio::test]
    async fn it_wraps_successfully<'a>() {
        let worker = Worker::<DummyJSContext>::new().await.unwrap();

        let key = worker
            .enqueue(move |r| {
                let js_val = DummyJSValue::new("this is a string");
                r.store_value(js_val)
            })
            .await
            .unwrap();

        let wrapper = JSValueWrapper::new(key, Arc::new(worker));

        let result: &str = wrapper.try_into().await.unwrap();
        assert_eq!(result, "this is a string");
    }
}
