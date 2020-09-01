use proc_macro::TokenStream;
use quote::quote;
use syn::{self, DeriveInput};

#[proc_macro_attribute]
pub fn js_export(attr: TokenStream, input: TokenStream) -> TokenStream {
    let ast: DeriveInput = syn::parse(input).unwrap();
    let name = ast.ident;

    // println!("{}", ast.ident);
    let gen = quote! {
        impl #name {
            fn new() -> Self {

            }
        }
    };
    gen.into()
}
