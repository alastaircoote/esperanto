use crate::test_util::ConstructableJSValue;
use esperanto_traits::js_traits::{JSEnvError, JSValue};

pub struct EmptyJSValue {}

impl JSValue for EmptyJSValue {
    fn to_string<'a>(&self) -> Result<&'a str, JSEnvError> {
        return Ok("THIS IS EMPTY");
    }
}

impl ConstructableJSValue for EmptyJSValue {
    fn new() -> Self {
        return EmptyJSValue {};
    }
}
