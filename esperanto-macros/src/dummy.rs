use proc_macro_error::set_dummy;
use quote::quote;
use syn::{ItemTrait, TraitItem};

// Part of proc_macro_attribute is the ability to set a 'dummy' TokenStream that will
// be returned if an error is found. In order for this code to be parsed we need to
// strip out all the js_ attributes (since they don't actually exist). So this does that:
pub fn set_dummy_without_js_attrs(trait_def: &ItemTrait) {
    let mut cloned = trait_def.clone();

    cloned.items.iter_mut().for_each(|item| {
        let attrs = match item {
            TraitItem::Const(c) => &mut c.attrs,
            TraitItem::Macro(m) => &mut m.attrs,
            TraitItem::Method(m) => &mut m.attrs,
            TraitItem::Type(t) => &mut t.attrs,
            _ => return,
        };
        attrs.retain(|attr| {
            let ident = match attr.path.get_ident() {
                Some(i) => i.to_string(),
                _ => return true,
            };

            ident != "js_function" && ident != "js_getter" && ident != "js_setter"
        })
    });

    let stream = proc_macro2::TokenStream::from(quote! { #cloned });
    set_dummy(stream);
}
