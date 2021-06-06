use std::fmt::{Display, Formatter};

use thiserror::Error;

/// JavaScriptError is just a small wrapper for JavaScript error objects. By extracting them
/// from the JS runtime we avoid lifetime, retain issues which makes error handling easier.
#[derive(Debug, PartialEq, Eq, Error)]
pub struct JavaScriptError {
    pub name: String,
    pub message: String,
}

impl JavaScriptError {
    pub(crate) fn new(name: String, message: String) -> Self {
        JavaScriptError { name, message }
    }
}

impl Display for JavaScriptError {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        write!(fmt, "{}: {}", self.name, self.message)
    }
}
