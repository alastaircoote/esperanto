use crate::feature_imports::jsvalue_ident;
use proc_macro2::Span;
use proc_macro2::TokenStream;
use proc_macro_error::emit_error;
use quote::quote_spanned;
use quote::{format_ident, ToTokens};
use syn::{spanned::Spanned, FnArg, Ident, Pat, PatType, ReturnType, TraitItemMethod, Type};

pub enum JSTypeCheckPosition {
    FnInput(syn::Ident),
    Return,
}
// Because this macro is just parsing a syntax tree we can't actually inspect their types
// or know anything other than their identifiers. If the types specified aren't valid (for
// example, if inputs don't implement FromJSValue, outputs don't implement ToJSValue) the code
// won't compile, but the errors will appear in odd places that wouldn't make their provenance
// all that clear.
// Thankfully, the quote crate gives us a way around this: quote_spanned!. It allows us to insert
// custom code and have the errors triggered by it attributed to a specific portion of source code.
// So we inject some dummy code to do type checks and pin it to the type definitions in the method
// signature. Got it? No, me neither. But anyway, we store them here:
pub struct JSTypeCheck {
    pub typ: Box<Type>,
    pub position: JSTypeCheckPosition,
    pub span: Span, // pub check_type: JSTypeCheckType,
}

fn get_underlying_type(typ: &Box<Type>) -> TokenStream {
    match &**typ {
        Type::Path(p) => p.into_token_stream(),
        Type::Reference(r) => r.elem.clone().into_token_stream(),
        _ => panic!("oh no"),
    }
}

impl JSTypeCheck {
    pub fn to_token_stream(&self, method_ident: &Ident, trait_ident: &Ident) -> TokenStream {
        let value_ident = jsvalue_ident();
        let ty = get_underlying_type(&self.typ);
        match &self.position {
            JSTypeCheckPosition::Return => {
                let check_ident = format_ident!("_{}{}return", trait_ident, method_ident);
                quote_spanned! {self.span=>
                    struct #check_ident where #ty: esperanto_shared::traits::ToJSValue<#value_ident>;
                }
            }
            JSTypeCheckPosition::FnInput(ident) => {
                if let Type::Path(_) = &*self.typ {
                    emit_error!(&self.typ, "All arguments must be references for now")
                }

                let check_ident = format_ident!("_{}{}{}", trait_ident, method_ident, ident);
                quote_spanned! {self.span=>
                    struct #check_ident where #ty: esperanto_shared::traits::FromJSValue<#value_ident>;
                }
            }
        }
    }

    pub fn from_method(method: &TraitItemMethod) -> Vec<Self> {
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
                        if let Pat::Ident(pat_ident) = &*t.pat {
                            return Some(JSTypeCheck {
                                typ: t.ty.clone(),
                                position: JSTypeCheckPosition::FnInput(pat_ident.ident.clone()),
                                span: arg.span(),
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
                ReturnType::Type(_, typ) => vec![JSTypeCheck {
                    typ: typ.clone(),
                    position: JSTypeCheckPosition::Return,
                    span: typ.span(),
                }],
                _ => Vec::new(),
            })
            .collect()
    }
}
