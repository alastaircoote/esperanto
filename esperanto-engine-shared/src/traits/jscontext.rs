use super::{jsruntime::RuntimeCreatesContext, CreateJSValue, JSRuntime, JSValue};
use crate::{
    errors::JSContextError, metadata::JSScriptSource,
    util::jscontext_with_runtime::JSContextWithRuntime,
};

/// This is the most important part of an Esperanto implementation, as it's the
/// place where you actually run your code.
///
/// # Lifetimes:
/// - `'runtime: the lifetime of the [JSRuntime](crate::traits::JSRuntime) that created
///   this context.
/// - `'context': the lifetime for this context. If the context was created with `JSContext::new`
///   both lifetimes will be the same.
pub trait JSContext<'runtime, 'context>: 'static
where
    Self: Sized,
{
    type Runtime: RuntimeCreatesContext<'runtime, 'context, Context = Self>;
    type Value: JSValue<'runtime, 'context, Context = Self> + CreateJSValue<'runtime, 'context, f64>;

    /// When you only want to create one context you don't really care about the
    /// [JSRuntime](crate::traits::JSRuntime), so this method gives you a shortcut. However,
    /// if you're intending on running multiple contexts simultaneously you might want to look
    /// into the advantages and disadvantages of sharing a runtime.
    fn new() -> Result<Self, JSContextError> {
        // todo!()
        let runtime = Self::Runtime::new()?;
        let ctx = Self::new_in_runtime(&runtime)?;
        let bundled = JSContextWithRuntime {
            context: ctx,
            runtime,
        };
        Ok(ctx)
    }

    fn new_in_runtime(runtime: &'runtime Self::Runtime) -> Result<Self, JSContextError>;

    /// Run some JavaScript. Probably the reason you're here.
    ///
    /// # Arguments
    /// - `script`: the JavaScript string you want to evaluate.
    /// - `source`: an optional argument to provide the source of this script.
    ///
    /// # Returns
    ///
    /// Either an error if evaluation failed or a reference to the JSValue that
    /// resulted from this script. Is given a lifetime of the runtime rather than
    /// the context because JSValues can be shared between contexts, provided they
    /// were created by the same runtime.
    fn evaluate<'a>(
        &'context self,
        script: &'a str,
        source: Option<JSScriptSource>,
    ) -> Result<&'runtime Self::Value, JSContextError>;
}
