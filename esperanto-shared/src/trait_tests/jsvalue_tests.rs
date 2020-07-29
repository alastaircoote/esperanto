use crate::traits::{JSContext, JSValue};

pub fn converts_to_number<Value: JSValue>() {
    let runtime = Value::ContextType::new().unwrap();
    let value = runtime.evaluate("3.5").unwrap();
    let f = value.as_number().unwrap();
    assert_eq!(f, 3.5);
}

pub fn converts_to_string<Value: JSValue>() {
    let runtime = Value::ContextType::new().unwrap();
    let value = runtime.evaluate("'hello'").unwrap();
    let f = value.as_string().unwrap();
    assert_eq!(f, "hello");
}

pub fn can_call_functions<Value: JSValue>() {
    let ctx = Value::ContextType::new().unwrap();
    let value = ctx
        .evaluate("(function(one, two) { return (one + two) / 2 })")
        .unwrap();

    let arg_one = Value::from_number(&2.0, ctx.get_shared_ref());
    let arg_two = Value::from_number(&3.0, ctx.get_shared_ref());

    let number = value
        .call_with_arguments(vec![&arg_one, &arg_two])
        .as_number()
        .unwrap();
    assert_eq!(number, 2.5);
}

pub fn can_wrap_rust_closure_with_one_argument<Value: JSValue>() {
    let ctx = Value::ContextType::new().unwrap();
    let func = Value::from_one_arg_closure(|num: f64| num * 2.0, ctx.get_shared_ref());
    let arg = Value::from_number(&3.0, ctx.get_shared_ref());
    let result = func.call_with_arguments(vec![&arg]);
    let number = result.as_number().unwrap();
    assert_eq!(number, 6.0);
}

pub fn can_wrap_rust_closure_with_two_arguments<Value: JSValue>() {
    let ctx = Value::ContextType::new().unwrap();
    let func =
        Value::from_two_arg_closure(|num1: f64, num2: f64| num1 * num2, ctx.get_shared_ref());
    let arg1 = Value::from_number(&3.0, ctx.get_shared_ref());
    let arg2 = Value::from_number(&4.0, ctx.get_shared_ref());
    let result = func.call_with_arguments(vec![&arg1, &arg2]);
    let number = result.as_number().unwrap();
    assert_eq!(number, 12.0);
}
