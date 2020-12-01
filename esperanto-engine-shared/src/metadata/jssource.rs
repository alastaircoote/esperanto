/// Information on the source of a script that's being evaluated. Is used in debugging
/// contexts (i.e. developer tools) to give the inspecting user information to help
/// them debug.
pub struct JSScriptSource<'a> {
    /// The full URL of the script being evaluated
    pub script_url: &'a str,
    /// The line number represented by the first line of the script provided. If you're
    /// evaluating an entire file this should be 0.
    pub line_number: i32,
}
