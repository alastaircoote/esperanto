use proc_macro2::TokenStream;
use proc_macro_error::emit_error;
use quote::quote;
use syn::{Ident, TraitItemMethod};

use crate::{
    js_export_attribute::{JSExportAttribute, JSExportAttributeParseResult},
    js_typechecks::JSTypeCheck,
};

pub struct JSExportMethod {
    ident: Ident,
    pub attribute: JSExportAttribute,
    pub typechecks: Vec<JSTypeCheck>,
}

impl JSExportMethod {
    pub fn from_definition(method: &mut TraitItemMethod) -> Option<Self> {
        // Let's start with the assumption that this method has no export.
        let mut export_type: Option<JSExportAttribute> = None;

        // Then iterate over each of the attributes applied to this method. The retain function mutates
        // the attrs vec, removing any item where we return false
        method.attrs.retain(|a| {
        match JSExportAttribute::from_tokens(a) {
            JSExportAttributeParseResult::NotAJSExportAttribute => {
                // This attribute isn't anything to do with us, so pass it through untouched.
                return true
            },
            JSExportAttributeParseResult::CouldNotParse(reason) => {
                // This has been identified as a JSExport attribute but wasn't parsed successfully.
                // Emit an error and remove this attribute from the token stream so that other
                // parsing can continue
                emit_error!(a, reason);
                return false
            },
            JSExportAttributeParseResult::Parsed(export) => {
                // The result we want! We've parsed out an export statment successfully.
                if let Some(_) = export_type {
                    // But wait, do we already have an export statement set? If so then two statements have been
                    // applied to the same method, which isn't valid. Emit and error and remove the second
                    // attribute, effectively ignoring it.
                    emit_error! { a, "Can only apply one js_export attribute to a method, ignoring this one" };
                    return true
                }
                // Otherwise, great, we're good to go. Let's store the export so that it can be checked below
                // below.
                export_type = Some(export);

                // Then we remove the attribute from the token stream because our custom attributes don't *actually*
                // exist (e.g. Rust doesn't know what to do with a js_function attribute, only we do).
                return false
            }
        }
    });

        export_type.map(|attribute| JSExportMethod {
            attribute,
            typechecks: JSTypeCheck::from_method(&method),
            ident: method.sig.ident.clone(),
        })
    }

    pub fn get_typecheck_token_stream(&self, trait_ident: &Ident) -> TokenStream {
        let tokenstreams: Vec<TokenStream> = self
            .typechecks
            .iter()
            .map(|t| t.to_token_stream(&self.ident, &trait_ident))
            .collect();

        quote! {
            #(#tokenstreams)*
        }
    }
}
