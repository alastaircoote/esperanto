use super::{JSRuntime, JSValue};
use crate::{errors::JSContextError, metadata::JSScriptSource};

/// This is the most important part of an Esperanto implementation, as it's the
/// place where you actually run your code.
pub trait JSContext: Clone {
    type Runtime: JSRuntime<Context = Self>;
    type Value: JSValue<Context = Self>;

    /// When you only want to create one context you don't really care about the
    /// [JSRuntime](crate::traits::JSRuntime), so this method gives you a shortcut. However,
    /// if you're intending on running multiple contexts simultaneously you might want to look
    /// into the advantages and disadvantages of sharing a runtime.
    fn new() -> Result<Self, JSContextError> {
        let runtime = Self::Runtime::new()?;
        let ctx = Self::new_in_runtime(&runtime)?;
        Ok(ctx)
    }

    fn new_in_runtime(runtime: &Self::Runtime) -> Result<Self, JSContextError>;

    fn global_object(&self) -> Result<Self::Value, JSContextError>;

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
        &self,
        script: &'a str,
        source: Option<JSScriptSource>,
    ) -> Result<Self::Value, JSContextError>;
}
