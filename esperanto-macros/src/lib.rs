use std::error::Error;

use proc_macro::{Span, TokenStream};
use quote::{format_ident, quote, quote_spanned};
use syn::{
    self, parse::Parse, parse::ParseStream, parse_macro_input, spanned::Spanned, Attribute,
    AttributeArgs, DeriveInput, FnArg, Ident, ImplItem, ImplItemMethod, Item, ItemFn, ItemImpl,
    ItemStruct, ItemTrait, Meta, MetaList, MetaNameValue,
};

#[proc_macro_attribute]
pub fn js_export(attr: TokenStream, input: TokenStream) -> TokenStream {
    let parsed_input = match syn::parse::<Item>(input) {
        Ok(Item::Trait(t)) => t,
        _ => panic!("Attribute can only be used on traits"),
    };

    // let mut parsed_input = syn::parse::<ItemTrait>(input).unwrap();

    // if parsed_input.trait_ != None {
    //     panic!("JSExport can only be used on a struct's own impl (for now)")
    // }

    // // the impl definition can include types and all kinds of things. We only
    // // care about methods, so let's get them:
    // let methods = parsed_input
    //     .items
    //     .iter()
    //     .filter_map(|item| {
    //         if let ImplItem::Method(method) = item {
    //             return Some(method);
    //         }
    //         return None;
    //     })
    //     .collect::<Vec<&ImplItemMethod>>();

    // // methods.first().unwrap().attrs
    // parsed_input.items.remove(0);

    TokenStream::from(quote! {
        #parsed_input
    })

    // let parsed_input = match syn::parse::<ItemImpl>(input) {
    //     Ok(result) => result,
    //     Err(err) => {
    //         return syn::Error::from(err).to_compile_error().into();
    //         // return syn::Error::new(Span::call_site().into(), "wat")
    //         //     .to_compile_error()
    //         //     .into();
    //     }
    // };

    // // let target_impl = match parsed_input {
    // //     // Item::Fn(f) => js_export_function(attr, f),
    // //     // Item::Struct(s) => js_export_class(attr, s),
    // //     Item::Impl(i) => i,
    // //     _ => {
    // //         return make_compile_error(
    // //             parsed_input,
    // //             "JSExport can only be used on an impl for now.",
    // //         )
    // //     }
    // // };

    // TokenStream::from(quote! {
    //     #parsed_input
    // })

    // let ast: syn::ItemFn = syn::parse(input).unwrap();
    // let name = ast.sig.ident.clone();

    // // let name = ast.ident;
    // // println!("{}", name);

    // // // println!("{}", ast.ident);
    // // let gen = quote! {
    // //     #ast
    // //     // impl Huh {
    // //     //     fn new() -> Self {
    // //     //         Huh {}
    // //     //     }
    // //     // }
    // // };
    // // gen.into()
    // let varname = format_ident!("{}2", name);
    // let gen = quote! {
    //     #ast

    //     fn #varname() {}
    // };

    // gen.into()
}
