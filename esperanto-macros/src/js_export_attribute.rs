// use proc_macro_error::emit_error;
use syn::{Attribute, Meta};

use crate::js_export_options::{JSExportOptions, MetaParseResult};

enum JSExportAttributeName {
    JSFunction,
    JSGetter,
    JSSetter,
}

impl JSExportAttributeName {
    fn from_string(string_value: String) -> Option<Self> {
        if string_value == "js_function" {
            return Some(JSExportAttributeName::JSFunction);
        }
        if string_value == "js_getter" {
            return Some(JSExportAttributeName::JSGetter);
        }
        if string_value == "js_setter" {
            return Some(JSExportAttributeName::JSSetter);
        }
        None
    }
}

pub enum JSExportAttribute {
    Function(JSExportOptions),
    Getter(JSExportOptions),
    Setter(JSExportOptions),
}

pub enum JSExportAttributeParseResult {
    /// Successfully parsed this attribute. The export statement is attached.
    Parsed(JSExportAttribute),
    /// This was identified as a JS export attribute but could not be successfully parsed.
    CouldNotParse(&'static str),
    /// Some other attribute. Ignore it.
    NotAJSExportAttribute,
}

impl JSExportAttribute {
    pub fn from_tokens(attribute: &Attribute) -> JSExportAttributeParseResult {
        let ident_str = match attribute.path.get_ident() {
            Some(id) => id.to_string(),
            // Not really sure in what circumstance a path would not have an ident, so we won't
            // remove it.
            None => return JSExportAttributeParseResult::NotAJSExportAttribute,
        };

        let attribute_name = match JSExportAttributeName::from_string(ident_str) {
            Some(value) => value,
            None => return JSExportAttributeParseResult::NotAJSExportAttribute,
        };

        // An attribute can be option-less, e.g. #[js_function], in which case it's Meta::Path
        // or it can have options, e.g. #[js_function(name = 'func')], in which case it's Meta::List

        let meta = match attribute.parse_meta() {
            Ok(Meta::Path(_)) => None,
            Ok(Meta::List(list)) => Some(list.nested),
            _ => {
                return JSExportAttributeParseResult::CouldNotParse(
                    "Could not parse attribute format",
                );
            }
        };

        let options = match JSExportOptions::from_nested_meta(meta) {
            MetaParseResult::Success(o) => o,
            _ => return JSExportAttributeParseResult::CouldNotParse("Could not parse nested meta"),
        };

        JSExportAttributeParseResult::Parsed(match attribute_name {
            JSExportAttributeName::JSFunction => JSExportAttribute::Function(options),
            JSExportAttributeName::JSGetter => JSExportAttribute::Getter(options),
            JSExportAttributeName::JSSetter => JSExportAttribute::Setter(options),
        })
    }
}
