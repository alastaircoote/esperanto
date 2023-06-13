use std::ffi::{CString, NulError};

/// File metadata to use when evaluating code. This will appear in stack traces,
/// dev tools etc. if included
pub struct EvaluateMetadata {
    pub filename: CString,
    pub line_number: i32,
}

impl EvaluateMetadata {
    pub fn new(filename: &str, line_number: i32) -> Result<Self, NulError> {
        let cstr = CString::new(filename)?;
        Ok(EvaluateMetadata {
            filename: cstr,
            line_number,
        })
    }
}
