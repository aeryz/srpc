extern crate proc_macro;

use proc_macro::TokenStream;
use syn::{parse_macro_input};
use quote::quote;

#[proc_macro_attribute]
pub fn service(_attrs: TokenStream, input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as syn::ItemImpl);
    let self_ident = &input.self_ty;
    let match_arms = input.items.iter().map(|item| {
        if let syn::ImplItem::Method(item_method) = item {
            assert!(item_method.attrs.is_empty());
            let method_args = &item_method.sig.inputs;
            let method_ident  = &item_method.sig.ident;

            let param_names = method_args.iter().map(|param| {
                if let syn::FnArg::Typed(param) = param {
                    // Get identifier of the parameter
                    &param.pat
                } else {
                    // TODO: Cool error
                    panic!("nopee");
                }
            });

            let return_type = if let syn::ReturnType::Type(_, ret_type) = &item_method.sig.output {
                Some(ret_type)
            } else {
                None
            };

            if method_args.is_empty() && return_type.is_none() {
                quote! {
                    stringify!(#method_ident) => {
                        #self_ident::#method_ident();
                        String::new()
                    }
                }
            } else if method_args.is_empty() && return_type.is_some() {
                quote! {
                    stringify!(#method_ident) => {
                        serde_json::to_string(&#self_ident::#method_ident())?
                    }
                }
            } else if !method_args.is_empty() && return_type.is_none() {
                let param_names_clone = param_names.clone();
                quote! {
                    stringify!(#method_ident) => {
                        #[derive(serde::Deserialize)]
                        struct Anon { #method_args }; // TODO: Reference problem
                        let Anon { #(#param_names,)* } = serde_json::from_str(&args)?;
                        #self_ident::#method_ident(#(#param_names_clone,)*);
                        String::new()
                    }
                }
            } else {
                let param_names_clone = param_names.clone();
                quote! {
                    stringify!(#method_ident) => {
                        #[derive(serde::Deserialize)]
                        struct Anon { #method_args }; // TODO: Reference problem
                        let Anon { #(#param_names,)* } = serde_json::from_str(&args)?;
                        serde_json::to_string(&#self_ident::#method_ident(#(#param_names_clone,)*))?
                    }
                }
            }
        } else {
            panic!("Items other than function are not supported right now.");
        }
    });
    let q = quote! {
        #input
        impl Service for #self_ident {
            fn call(&self, fn_name: String, args: String) -> srpc::Result<String> {
                let ret_str = match fn_name.as_str() {
                    #(#match_arms,)*
                    _ => return Err(String::new().into())
                };

                Ok(ret_str)
            }

            fn get_route(&self) -> &'static str { return self.route }
        }
    };

    TokenStream::from(q)
}