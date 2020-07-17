use crate::{errors::JSEnvError, traits::JSContext};
use std::{convert::TryInto, fmt::Debug};

pub fn can_create_from_object<Context: JSContext>()
where
    Context::ValueType: TryInto<Context::ObjectType>,
    <Context::ValueType as TryInto<Context::ObjectType>>::Error: Debug,
{
    let ctx = Context::new().unwrap();
    let value = ctx.evaluate("({})").unwrap();
    let _: Context::ObjectType = value.try_into().unwrap();
}

pub fn throws_when_not_given_object<Context: JSContext>()
where
    Context::ValueType: TryInto<Context::ObjectType, Error = JSEnvError>,
{
    let ctx = Context::new().unwrap();
    let result = ctx.evaluate("undefined").unwrap();
    let conversion_result: Result<Context::ObjectType, _> = result.try_into();
    match conversion_result {
        Ok(_) => panic!("Conversion should not succeed"),
        Err(e) => match e {
            JSEnvError::JSErrorEncountered(err) => {
                assert_eq!(err.name, "TypeError");
            }
            _ => panic!("Unexpected error type"),
        },
    }
}
