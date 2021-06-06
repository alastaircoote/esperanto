use std::ffi::{CString, NulError};

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
