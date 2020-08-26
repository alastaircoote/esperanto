use crate::traits::{JSContext, JSValue};

pub fn converts_to_number<Value: JSValue>() {
    let ctx = Value::ContextType::new().unwrap();
    let value = ctx.evaluate("3.5").unwrap();
    let f = value.as_number().unwrap();
    assert_eq!(f, 3.5);
}

pub fn converts_to_string<Value: JSValue>() {
    let ctx = Value::ContextType::new().unwrap();
    let value = ctx.evaluate("'hello'").unwrap();
    let f = value.as_string().unwrap();
    assert_eq!(f, "hello");
}

pub fn converts_from_number<Value: JSValue>() {
    let ctx = Value::ContextType::new().unwrap();
    let value = Value::from_number(3.5, &ctx).unwrap();
    let f = value.as_number().unwrap();
    assert_eq!(f, 3.5);
}

pub fn converts_to_boolean<Value: JSValue>() {
    let ctx = Value::ContextType::new().unwrap();
    let value = ctx.evaluate("true").unwrap();
    let f = value.as_bool().unwrap();
    assert_eq!(f, true);
}

pub fn converts_from_boolean<Value: JSValue>() {
    let ctx = Value::ContextType::new().unwrap();
    let value = Value::from_bool(true, &ctx).unwrap();
    let f = value.as_bool().unwrap();
    assert_eq!(f, true);
}

pub fn converts_from_string<Value: JSValue>() {
    let ctx = Value::ContextType::new().unwrap();
    let value = Value::from_string("TEST_STRING", &ctx).unwrap();
    let back_to_str = value.as_string().unwrap();
    assert_eq!(back_to_str, "TEST_STRING");
}

pub fn can_call_functions<Value: JSValue>() {
    let ctx = Value::ContextType::new().unwrap();
    let value = ctx
        .evaluate("(function(one, two) { return (one + two) / 2 })")
        .unwrap();

    let arg_one = Value::from_number(2.0, &ctx).unwrap();
    let arg_two = Value::from_number(3.0, &ctx).unwrap();

    let number = value
        .call_with_arguments(vec![&arg_one, &arg_two])
        .unwrap()
        .as_number()
        .unwrap();
    assert_eq!(number, 2.5);
}

pub fn can_wrap_rust_closure_with_one_argument<Value: JSValue>() {
    let ctx = Value::ContextType::new().unwrap();
    let func = Value::from_one_arg_closure(|num: f64| Ok(num * 2.0), &ctx).unwrap();
    let arg = Value::from_number(3.0, &ctx).unwrap();
    let result = func.call_with_arguments(vec![&arg]).unwrap();
    let number = result.as_number().unwrap();
    assert_eq!(number, 6.0);
}

pub fn can_wrap_rust_closure_with_two_arguments<Value: JSValue>() {
    let ctx = Value::ContextType::new().unwrap();
    let func = Value::from_two_arg_closure(|num1: f64, num2: f64| Ok(num1 * num2), &ctx).unwrap();
    let arg1 = Value::from_number(3.0, &ctx).unwrap();
    let arg2 = Value::from_number(4.0, &ctx).unwrap();
    let result = func.call_with_arguments(vec![&arg1, &arg2]).unwrap();
    let number = result.as_number().unwrap();
    assert_eq!(number, 12.0);
}

pub fn can_get_properties<Value: JSValue>() {
    let ctx = Value::ContextType::new().unwrap();
    let obj = ctx.evaluate("({a: 'bcd'})").unwrap();
    let value = obj.get_property("a").unwrap().as_string().unwrap();
    assert_eq!(value, "bcd");
}

pub fn can_create_function<Value: JSValue>() {
    let ctx = Value::ContextType::new().unwrap();
    let func = Value::create_function(&ctx, vec!["one", "two"], "return one * two").unwrap();
    let arg1 = Value::from_number(3.0, &ctx).unwrap();
    let arg2 = Value::from_number(4.0, &ctx).unwrap();
    let result = func.call_with_arguments(vec![&arg1, &arg2]).unwrap();
    let result_number = result.as_number().unwrap();
    assert_eq!(result_number, 12.0);
}
