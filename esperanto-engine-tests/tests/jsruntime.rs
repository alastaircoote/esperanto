use esperanto_engine_javascriptcore::JSCoreRuntime;
use esperanto_engine_shared::traits::{JSRuntime, RuntimeCreatesContext};
use test_impl::test_impl;

#[test_impl(JSRuntime(JSCoreRuntime))]
#[test]
fn wat() {
    let r = JSRuntime::new().unwrap();
    r.create_context().unwrap();
}
