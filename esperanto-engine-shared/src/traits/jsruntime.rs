use crate::errors::JSRuntimeError;

use super::JSContext;

/// JSRuntime is the 'core' of any use of the JavaScript engine, containing
/// the garage collector and so on. A JSRuntime can create multiple
/// [`JSContext`](crate::traits::JSContext)s. Any JSValues created in these
/// contexts can be shared, allowing you to easily pass data between contexts.
///
/// However, there are downsides to sharing runtimes between contexts. They must
/// all run on the same thread, so if you want to run multiple intense operations
/// simultaneously the contexts will have to wait for each other to complete.
pub trait JSRuntime: Clone {
    type Context: JSContext<Runtime = Self>;
    /// Creates a new JSRuntime. Can also return an error if the engine implementation
    /// is not able to create a runtime for whatever reason.
    fn new() -> Result<Self, JSRuntimeError>;
}
