// use async_trait::async_trait;
// use esperanto_core::js_trait::JSContext;
// use esperanto_core::jsvalue::JSValue;
// use esperanto_core::worker::Worker;
// use util::always_return_runtime;
// struct Urrgh {}
// struct UrrghJSValue {}

// impl JSValue for UrrghJSValue {
//     fn to_string(&self) -> String {
//         return "huh".to_string();
//     }
// }

// impl JSContext for Urrgh {
//     fn evaluate(&self, script: String) -> UrrghJSValue {
//         return UrrghJSValue {};
//     }
//     fn new() -> Self {
//         return Urrgh {};
//     }
//     type ValueType = UrrghJSValue;
// }

// #[tokio::test]
// async fn workers_run_on_the_right_thread() {
//     let worker = Worker::<Urrgh>::new().await.unwrap();
//     let result = worker
//         .enqueue(|_| return std::thread::current().id())
//         .await
//         .unwrap();
//     assert_eq!(result, worker.thread_id);
// }
