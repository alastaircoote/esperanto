use super::jscontext::JSContext;
use crate::errors::{JSContextError, JSRuntimeError};

/// JSRuntime is the 'core' of any use of the JavaScript engine, containing
/// the garage collector and so on. A JSRuntime can create multiple
/// [`JSContext`](crate::traits::JSContext)s. Any JSValues created in these
/// contexts can be shared, allowing you to easily pass data between contexts.
///
/// However, there are downsides to sharing runtimes between contexts. They must
/// all run on the same thread, so if you want to run multiple intense operations
/// simultaneously the contexts will have to wait for each other to complete.

/// If you want to actually create a context you'll need to use an additional trait
/// , [`RuntimeCreatesContext`](crate::traits::RuntimeCreatesContext).
///
/// # Lifetimes
/// - `'runtime` - the lifetime for this runtime. It'll apply to any contexts
///   and values created with this runtime.
pub trait JSRuntime<'runtime>: Sized {
    /// Creates a new JSRuntime. Can also return an error if the engine implementation
    /// is not able to create a runtime for whatever reason.
    fn new() -> Result<Self, JSRuntimeError>;
}

/// The point of having a JSRuntime! Because Rust doesn't support [generic associated
/// types yet](https://github.com/rust-lang/rust/issues/44265) we have to keep this in
/// a separate trait to `JSRuntime`. In the future we'll be able to merge it into one.
pub trait RuntimeCreatesContext<'runtime, 'context>: JSRuntime<'runtime> + 'static {
    /// This engine's [`JSContext`](crate::traits::JSContext) implementation. Has it's
    /// own context-specific lifetime.
    type Context: JSContext<'runtime, 'context, Runtime = Self>;

    /// Make a context so that you can actually run some JavaScript code. If the engine
    /// implementation can't make a context for whatever reason it'll return an error
    /// instead.
    fn create_context(&'runtime self) -> Result<Self::Context, JSContextError>;
}
