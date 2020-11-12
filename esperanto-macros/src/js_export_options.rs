use proc_macro_error::emit_error;
use syn::{punctuated::Punctuated, Lit, Meta, NestedMeta, Token};

pub struct JSExportAttributeOptions {
    override_name: Option<String>,
}

pub enum MetaParseResult {
    Failed,
    Success(JSExportAttributeOptions),
}

impl JSExportAttributeOptions {
    pub fn from_nested_meta(
        nested_meta: Option<Punctuated<NestedMeta, Token![,]>>,
    ) -> MetaParseResult {
        let mut options = JSExportAttributeOptions {
            override_name: None,
        };

        let meta = match nested_meta {
            Some(m) => m,
            _ => return MetaParseResult::Success(options),
        };

        let mut found_errors = false;

        // Go through each nested meta entry

        for meta in meta {
            match meta {
                // Check to see whether it's a name/value pair
                NestedMeta::Meta(Meta::NameValue(name_value)) => {
                    // Check that it has a path and a literal value
                    match (name_value.path.get_ident(), &name_value.lit) {
                        (Some(ident), Lit::Str(str)) => {
                            // Actually check the path to see if we recognise it,
                            // and if so, deal with it
                            if ident.to_string() == "name" {
                                options.override_name = Some(str.value())
                            } else {
                                emit_error! {ident, "Did not recognise option name" }
                                found_errors = true
                            }
                        }
                        _ => {
                            emit_error! {name_value, "Options should be specified as string name/value pairs"}
                            found_errors = true
                        }
                    };
                }
                _ => {
                    emit_error! {meta, "Options should be specified as string name/value pairs"};
                    found_errors = true
                }
            }
        }

        if found_errors {
            MetaParseResult::Failed
        } else {
            MetaParseResult::Success(options)
        }
    }
}
