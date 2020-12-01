mod dummy;
mod feature_imports;
mod js_export_attribute;
mod js_export_method;
mod js_export_options;
mod js_typechecks;

use dummy::set_dummy_without_js_attrs;
use js_export_method::JSExportMethod;
use proc_macro::TokenStream;
use proc_macro_error::{abort_call_site, proc_macro_error};
use quote::{format_ident, quote, quote_spanned};
use syn::{self, Item, TraitItem, TypeParamBound};

#[proc_macro_attribute]
#[proc_macro_error]
pub fn js_export(_: TokenStream, input: TokenStream) -> TokenStream {
    let w = input.iter();
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

    let typechecks: Vec<proc_macro2::TokenStream> = exports
        .iter()
        .map(|t| t.get_typecheck_token_stream(&parsed_input.ident))
        .collect();

    let hash_check_ident = format_ident!("_{}_hash_check", parsed_input.ident);
    let base_ident = parsed_input.ident.clone();
    let trait_span = parsed_input.ident.span();

    // In order to be able to match instances of our JSExported trait and JSValues we have to use
    // hash values. So the trait must inherit Hash. Let's check for that:
    let hash_check = quote_spanned! {trait_span=>
        use #base_ident as #hash_check_ident;
        fn _assert_hash_supertrait<#base_ident: #hash_check_ident>() {
            fn requires_hash<T: Hash>() {}
            let _ = requires_hash::<#base_ident>;
        }
    };

    TokenStream::from(quote! {
        #hash_check
        #parsed_input
        // trait #hash_check_ident: #base_ident where Self: std::hash::Hash {}
        #(#typechecks)*
    })
}
