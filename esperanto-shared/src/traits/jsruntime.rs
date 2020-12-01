use std::rc::Rc;

use crate::errors::JSContextError;

use super::JSContext;

pub trait JSRuntime<'runtime>: Sized {
    type ContextType: JSContext<'runtime>;
    fn new() -> Result<Self, JSContextError>;
    fn create_context(&self) -> Result<Self::ContextType, JSContextError>;
}

#[cfg(test)]
mod test {
    use std::{convert::TryInto, marker::PhantomData};

    use crate::{
        errors::JSContextError,
        traits::{JSContext, JSValue},
    };

    use super::JSRuntime;

    struct TestValue<'runtime> {
        _dummy: &'runtime PhantomData<()>,
    }

    impl<'runtime> TryInto<f64> for TestValue<'runtime> {
        type Error = JSContextError;

        fn try_into(self) -> Result<f64, Self::Error> {
            todo!()
        }
    }

    impl<'runtime> JSValue<'runtime> for TestValue<'runtime> {
        type ContextType = TestContext<'runtime>;

        type RawType = ();

        fn as_string(&self) -> Result<&String, crate::errors::JSContextError> {
            todo!()
        }

        fn as_number(&self) -> Result<&f64, crate::errors::JSContextError> {
            todo!()
        }

        fn as_bool(&self) -> Result<&bool, crate::errors::JSContextError> {
            todo!()
        }

        fn from_number(
            number: f64,
            in_context: &Self::ContextType,
        ) -> Result<&Self, crate::errors::JSContextError> {
            todo!()
        }

        fn from_bool(
            bool: bool,
            in_context: &Self::ContextType,
        ) -> Result<&Self, crate::errors::JSContextError> {
            todo!()
        }

        fn from_string<'c>(
            str: &str,
            in_context: &'c Self::ContextType,
        ) -> Result<&'c Self, crate::errors::JSContextError> {
            todo!()
        }

        fn call_bound(
            &self,
            arguments: Vec<&Self>,
            bound_to: &Self,
        ) -> Result<Self, crate::errors::JSContextError> {
            todo!()
        }

        fn get_property(&self, name: &str) -> Result<&Self, crate::errors::JSContextError> {
            todo!()
        }

        fn from_raw(
            raw: Self::RawType,
            in_context: &Self::ContextType,
        ) -> Result<Self, crate::errors::JSContextError> {
            todo!()
        }

        fn create_function<'c>(
            in_context: &'c Self::ContextType,
            arg_names: Vec<&str>,
            body: &str,
        ) -> Result<&'c Self, crate::errors::JSContextError> {
            todo!()
        }
    }

    struct TestContext<'runtime> {
        runtime: &'runtime TestRuntime,
    }

    impl<'runtime> JSContext<'runtime> for TestContext<'runtime> {
        type ValueType = TestValue<'runtime>;

        type RuntimeType = TestRuntime;

        fn evaluate<'a>(
            &'runtime self,
            script: &'a str,
        ) -> Result<&Self::ValueType, crate::errors::JSContextError> {
            todo!()
        }

        fn evaluate_cstring(
            &self,
            script: *const std::os::raw::c_char,
        ) -> Result<Self::ValueType, crate::errors::JSContextError> {
            todo!()
        }

        fn new() -> Result<Self, crate::errors::JSContextError> {
            todo!()
        }

        fn new_in_runtime(
            runtime: &'runtime Self::RuntimeType,
        ) -> Result<Self, crate::errors::JSContextError> {
            todo!()
        }

        fn compile_string<'a>(
            &self,
            script: *const std::os::raw::c_char,
        ) -> Result<&'a [u8], crate::errors::JSContextError> {
            todo!()
        }

        fn eval_compiled(
            &self,
            binary: &[u8],
        ) -> Result<&Self::ValueType, crate::errors::JSContextError> {
            todo!()
        }
    }

    struct TestRuntime {
        // _dummy: &'runtime PhantomData<()>,
    }

    impl<'runtime> JSRuntime<'runtime> for TestRuntime {
        fn new() -> Result<Self, crate::errors::JSContextError> {
            Ok(TestRuntime {
                // _dummy: &PhantomData,
            })
        }

        type ContextType = TestContext<'runtime>;

        fn create_context(&self) -> Result<Self::ContextType, JSContextError> {
            todo!()
        }
    }

    #[test]
    fn test() {
        let run = TestRuntime::new().unwrap();

        let ctx = run.create_context().unwrap();
        let ctx2 = run.create_context().unwrap();
        drop(run);
        ctx.evaluate("test").unwrap();
    }
}
