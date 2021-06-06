/// This is the giant switch for changing the underlying JS library we're using.
/// Other modules always go through this module rather than include engine-specific
/// stuff directly.

#[cfg(feature = "javascriptcore")]
pub(crate) use crate::jscore::*;

#[cfg(feature = "quickjs")]
pub(crate) use crate::quickjs::*;
