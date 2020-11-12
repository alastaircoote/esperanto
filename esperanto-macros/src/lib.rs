mod dummy;
mod feature_imports;
mod js_export_options;
mod js_export_attribute;
mod js_typechecks;
mod js_export_method;

use crate::feature_imports::jsvalue_ident;
use dummy::set_dummy_without_js_attrs;
use js_export_attribute::{JSExportAttributeParseResult, JSExportAttribute};
use js_export_method::JSExportMethod;
use js_typechecks::{JSTypeCheck, JSTypeCheckPosition, JSTypeCheckType};
use proc_macro::TokenStream;
use proc_macro2::{Span, TokenTree};
use proc_macro_error::{abort_call_site, emit_error, proc_macro_error, set_dummy};
use quote::{quote, quote_spanned, ToTokens};
use syn::{
    self, spanned::Spanned, FnArg, Item, ItemTrait, Pat, ReturnType, TraitItem, TraitItemMethod,
};

/// Examine a method and check to see if it has any js-specific attributes attached.
fn parse_method(method: &mut TraitItemMethod) -> Option<JSExportAttribute> {
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

    export_type
}

/// Add type checking code to the provided vec to provide useful compiler errors when
/// JSExport types haven't been implemented properly.
fn typecheck_method(method: &mut TraitItemMethod) -> Vec<proc_macro2::TokenStream> {
    // Because this macro is just parsing a syntax tree we can't actually inspect their types
    // or know anything other than their identifiers. If the types specified aren't valid (for
    // example, if inputs don't implement FromJSValue, outputs don't implement ToJSValue) the code
    // won't compile, but the errors will appear in odd places that wouldn't make their provenance
    // all that clear.
    //
    // Thankfully, the quote crate gives us a way around this: quote_spanned!. It allows us to insert
    // custom code and have the errors triggered by it attributed to a specific portion of source code.
    // So we inject some dummy code to do type checks and pin it to the type definitions in the method
    // signature. Got it? No, me neither.

    let value_ident = jsvalue_ident();

    method
        .sig
        .inputs
        .iter()
        .filter_map(|arg| {
            match arg {
                FnArg::Receiver(rec) => {
                    // This is the 'self' argument at the start of a lot of Rust functions.
                    if rec.reference == None {
                        // If this fn consumes the struct (i.e. fn(self) {}) then we can't export to JS, because JS
                        // doesn't have any concept of a function that does that. So... emit an error.
                        emit_error!(rec, "Cannot js_export a method that consumes the object")
                    }
                    None
                }

                FnArg::Typed(t) => {
                    // This is a normal argument
                    if let Pat::Ident(ident) = &*t.pat {
                        let hm = &t.ty;
                        return Some(quote_spanned!{t.span()=>
                            struct _AssertSync where #hm: esperanto_shared::traits::FromJSValue<#value_ident>;
                        });

                    } else {
                        // not sure! need to investigate
                        panic!("no")
                    }
                }
            }
        })
        
        .chain(match &method.sig.output {
            // Then we do the same for the return type, if there is one
            ReturnType::Type(_, typ) => {
                return vec![quote_spanned!{typ.span()=>
                    struct _AssertSync where #typ: esperanto_shared::traits::ToJSValue<#value_ident>;
                }];
            },
            _ => Vec::new(),
        })
    
        .collect()
}

#[proc_macro_attribute]
#[proc_macro_error]
pub fn js_export(_: TokenStream, input: TokenStream) -> TokenStream {
    let mut parsed_input = match syn::parse::<Item>(input) {
        Ok(Item::Trait(t)) => t,
        Ok(_) => {
            abort_call_site! {  "Attribute can only be used on traits."}
        }
        Err(err) => {
            abort_call_site! {  "Parse error occurred: {}", err}
        }
    };

    // Set a "dummy" version of this token stream with all the js-specific stuff stripped out,
    // to be used by the compiler when parsing fails:
    set_dummy_without_js_attrs(&parsed_input);

    let exports: Vec<JSExportMethod> = parsed_input
        .items
        .iter_mut()
        .filter_map(|item| match item {
            TraitItem::Method(method) => JSExportMethod::from_definition(method),
            _ => None,
        })
        .collect();

    let typechecks: Vec<proc_macro2::TokenStream> =
        exports.iter().map(|t| t.get_typecheck_token_stream(&parsed_input.ident))
        .collect();

    // panic!("{}",typechecks.len());

    TokenStream::from(quote! {
        #parsed_input
        #(#typechecks)*
    })
}
