use super::JSContext;

pub trait JSValue<'runtime, 'context> {
    type Context: JSContext<'runtime, 'context>;
}

fn test<'a>() -> &'a String {
    let hello = "string".to_string();
    &hello
}
