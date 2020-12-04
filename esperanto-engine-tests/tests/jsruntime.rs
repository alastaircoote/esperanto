mod runtime_tests {
    use esperanto_engine_javascriptcore::JSCoreRuntime;
    use esperanto_engine_shared::traits::JSRuntime;
    use test_impl::test_impl;

    #[test_impl(JSRuntime = JSCoreRuntime)]
    fn runtime_creates_successfully() {
        JSRuntime::new().unwrap();
    }
}
