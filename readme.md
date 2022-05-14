# Esperanto

A Rust-powered JavaScript worker environment designed to be embedded in native mobile apps. It builds on work established in [SWWebView](https://github.com/gdnmobilelab/SWWebView) but seeks to be fully comprehensive and platform-independent.

**...at least, it will be at some point, right now it's little more than a proof of concept.**

## Some background

Native mobile apps are wonderful, slick and performant. But they're difficult to change on the fly when every new version requires App Store approval and rollout to all your users. Wouldn't it be great if you could use a sandboxed JavaScript environment to adjust app behaviours on the fly? Well, you already can if you want to triple down on JS: use React Native and construct the entire app in JS. But I'm interested in something that lives somewhere in the middle: a JS environment with no dependencies that can sit more seamlessly within native code and add as little bloat to your app as possible, to make it as easy as possible to bring together JS-focused and native-focused teams.

To that end, Esperanto can be built to run against two different JavaScript engines:

### JavaScriptCore

This is the JS engine bundled by Apple with iOS. If you enable the `javascriptcore` feature when building this library it will run against the system-provided JSC runtime. There's no option to build and embed JSC so this feature will only ever work if you're running on iOS or macOS.

### QuickJS

Android comes with a System WebView package that has the entire V8 JS runtime in it... but sadly it doesn't expose it in a usable way. So as an alternative Esperanto can compile and embed [QuickJS](https://bellard.org/quickjs/), a tiny (210KB) JS engine that still manages to be somewhat performant. It depends on my [quickjs-android-suitable-sys](https://github.com/alastaircoote/quickjs-android-suitable-sys) crate, which includes a few tweaks to make sure QuickJS will run on Android fine.

## Components

### JSContext

A context is the environment in which your code runs.

- `JSContext::new`
- `JSContext::evaluate`
- `JSContext::get_global_object`

### JSValue

`JSContext::evaluate` returns a `JSValue`. You can convert a JSValue into a number of native types (strings, numbers, etc) via `try_into`.

- `JSValue::call_as_function`
- `JSValue::new_function`
- `JSValue::call_as_constructor`
- `JSValue::get_property`
- `JSValue::set_property`
- `JSValue::is_instance_of`

### JSRuntime

Rarely of use but you can create multiple `JSContexts` that share an underlying `JSRuntime`. This allows you to share `JSValue`s between contexts (which is otherwise impossible).

- `JSRuntime::new`

### JSExportClass

A trait you can implement in Rust to allow you to pass a Rust struct in and out of JS contexts. Right now only two functionalities are implemented:

- `call_as_function`
- `call_as_constructor`

## Examples:

### Evaluate a string that returns a string:

```rust
let ctx = JSContext::new().unwrap();
let result = ctx.evaluate("['one','two'].join(', ')", None).unwrap();
let str = String::try_from(result).unwrap();
assert_eq!(str, "one, two");
```

### Create a function from a string and call it:

```rust
let ctx = JSContext::new().unwrap();
let body = "return one * two";
let func = JSValueRef::new_function(body, vec!["one", "two"], &ctx).unwrap();

let two = JSValueRef::try_new_value_from(2);
let three = JSValueRef::try_new_value_from(3);

let result = func.call_as_function(vec![two, three]);
let result_f64 = f64::try_from(result).unwrap();
assert_eq!(result_f64, 5)
```

### Wrap native Rust code and make it accessible in JS:

```rust
struct TestStruct {}

js_export_class! { TestStruct as "TestStruct" =>
    call_as_function: (ctx, _this_obj, _values) {
        Ok(JSValueRef::try_new_value_from(1234, ctx)?.into())
    },
};

let test = TestStruct {};
let ctx = JSContext::new().unwrap();
let wrapped = JSValueRef::wrap_native(test, &ctx).unwrap();
ctx.global_object()
    .set_property("testValue", &wrapped)
    .unwrap();

let result = ctx.evaluate("testValue()", None).unwrap();
let num: i32 = result.try_into().unwrap();
assert_eq!(num, 1234)
```
