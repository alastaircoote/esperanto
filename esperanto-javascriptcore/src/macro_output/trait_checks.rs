use proc_macro2::{Span, TokenStream};
use quote::{quote, quote_spanned};

fn jscontext_ident() -> TokenStream {
    quote! {esperanto_javascriptcore::JSCGlobalContext}
}

pub fn jsvalue_ident() -> TokenStream {
    quote! {esperanto_javascriptcore::JSCValue}
}

// pub fn to_value_trait_check(typ: TokenStream, span: Span) -> TokenStream {
//     let ctx = jsvalue_ident();
//     let varname = format_ident!("_{}__ToJSValueCheck", ident);
//     quote_spanned! {span=>
//         struct _AssertSync where #typ: esperanto_shared::traits::ToJSValue<#ctx>;
//     }
// }

// pub fn from_value_trait_check(typ: TokenStream, span: Span) -> TokenStream {
//     let ctx = jsvalue_ident();
//     quote_spanned! {span=>
//         struct _AssertSync where #typ: esperanto_shared::traits::FromJSValue<#ctx>;
//     }
// }
