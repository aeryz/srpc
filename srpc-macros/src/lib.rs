extern crate proc_macro;

use proc_macro::TokenStream;
use syn::{braced, parse_macro_input, token::Token, Field, Ident, Result, Error};
use syn::Token;
use syn::parse::{Parse, ParseStream, ParseBuffer};
use syn::punctuated::Punctuated;
use syn::parse_quote::parse;
use std::sync::WaitTimeoutResult;
use syn::token::Impl;
use quote::quote;

#[proc_macro_attribute]
pub fn server_impl(_attrs: TokenStream, input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as syn::ItemImpl);
    let impl_ident = &input.self_ty;
    let items = input.items.iter().map(|item| {
        if let syn::ImplItem::Method(item_method) = item {
            assert!(item_method.attrs.is_empty());
            let inputs = &item_method.sig.inputs;
            let method_name  = &item_method.sig.ident;
            let block = &item_method.block;

            let param_names = inputs.iter().map(|inp| {
                if let syn::FnArg::Typed(pat_type) = inp {
                    &pat_type.pat
                } else {
                    panic!("nopee");
                }
            });

            if inputs.is_empty() {
                quote! { fn #method_name() -> String { serde_json::to_string(&#block).unwrap() } }
            } else {
                quote! {
                    fn #method_name(args: String) -> String {
                        #[derive(serde::Deserialize)]
                        struct Anon { #inputs };
                        let Anon { #(#param_names,)* } = serde_json::from_str(&args).unwrap();

                        serde_json::to_string(&#block).unwrap()
                    }
                }
            }
        } else {
            panic!("Items other than function are not supported right now.");
        }
    });
    let q = quote! {
        impl #impl_ident {
        #(#items)*
        }
    };

    TokenStream::from(q)
}