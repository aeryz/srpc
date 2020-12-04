extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::parse_macro_input;

fn parse_route(mut attrs: syn::AttributeArgs) -> syn::LitStr {
    if let syn::NestedMeta::Meta(syn::Meta::NameValue(syn::MetaNameValue {
        path,
        lit: syn::Lit::Str(route_name),
        ..
    })) = attrs.pop().unwrap()
    {
        if path.segments.len() != 1 || path.segments.first().unwrap().ident != "route" {
            panic!(
                "'route' attribute is expected only. eg/ #[srpc::service(route = \"cool_route\")]"
            );
        }
        return route_name;
    } else {
        panic!("'route' attribute is expected. eg/ #[srpc::service(route = \"cool_route\")]");
    };
}

#[proc_macro_attribute]
pub fn service(attrs: TokenStream, input: TokenStream) -> TokenStream {
    let attrs = parse_macro_input!(attrs as syn::AttributeArgs);
    let input = parse_macro_input!(input as syn::ItemStruct);

    if !input.fields.is_empty() {
        panic!("An srpc service struct should be a unit struct.");
    }

    if attrs.len() != 1 {
        panic!("Attribute 'route' should be defined. eg/ #[srpc::service(route = \"cool_route\")]");
    }

    let self_ident = &input.ident;

    let route_name = parse_route(attrs);

    TokenStream::from(quote! {
        struct #self_ident {
            route: &'static str
        }

        impl #self_ident {
            pub fn new() -> Self {
                Self { route: #route_name }
            }
        }
    })
}

#[proc_macro_attribute]
pub fn client(attrs: TokenStream, input: TokenStream) -> TokenStream {
    let attrs = parse_macro_input!(attrs as syn::AttributeArgs);
    let input = parse_macro_input!(input as syn::ItemTrait);

    if attrs.len() != 1 {
        panic!("Expected 'route' attribute only. eg/ #[srpc::client(route = \"cool_route\")]");
    }

    let route_name = parse_route(attrs);

    let self_ident = &input.ident;
    let methods = input.items.iter().map(|item| {
        if let syn::TraitItem::Method(item_method) = item {
            let method_args = &item_method.sig.inputs;
            let method_ident = &item_method.sig.ident;
            let param_names = method_args.iter().map(|param| {
                if let syn::FnArg::Typed(param) = param {
                    &param.pat
                } else {
                    panic!("Using 'self' in an RPC call is not allowed for now.");
                }
            });

            let return_type = if let syn::ReturnType::Type(_, ret_type) = &item_method.sig.output {
                Some(ret_type)
            } else {
                None
            };

            if method_args.is_empty() && return_type.is_none() {
                quote! {
                    fn #method_ident(client: &mut srpc::client::Client) -> srpc::Result<()> {
                        let _ = client.call("str-service", stringify!(#method_ident), String::new())?;
                        Ok(())
                    }
                }
            } else if method_args.is_empty() && return_type.is_some() {
                let ret_type = return_type.unwrap();
                quote! {
                    fn #method_ident(client: &mut srpc::client::Client) -> srpc::Result<#ret_type> {
                        Ok(serde_json::from_slice(&client.call("str-service", stringify!(#method_ident), String::new())?)?)
                    }
                }
            } else if !method_args.is_empty() && return_type.is_none() {
                quote! {
                    fn #method_ident(client: &mut srpc::client::Client, #method_args) -> srpc::Result<()> {
                        #[derive(serde::Serialize)]
                        struct Args { #method_args }
                        let _ = client.call("str-service", stringify!(#method_ident), serde_json::to_string(&Args { #(#param_names,)* })?)?;
                        Ok(())
                    }
                }
            } else {
                let ret_type = return_type.unwrap();
                quote! {
                    fn #method_ident(client: &mut srpc::client::Client, #method_args) -> srpc::Result<#ret_type> {
                        #[derive(serde::Serialize)]
                        struct Args { #method_args }
                        Ok(serde_json::from_slice(&client.call("str-service", stringify!(#method_ident), serde_json::to_string(&Args { #(#param_names,)* })?)?)?)
                    }
                }
            }
        } else {
            panic!("Only methods are allowed in an srpc client.");
        }
    });

    let q = quote! {
        struct #self_ident { route: &'static str }
        impl #self_ident {
            pub fn new() -> Self {
                Self { route: #route_name }
            }
            #(#methods)*
        }
    };

    TokenStream::from(q)
}

#[proc_macro_attribute]
pub fn service_impl(_attrs: TokenStream, input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as syn::ItemImpl);
    let self_ident = &input.self_ty;
    let match_arms = input.items.iter().map(|item| {
        if let syn::ImplItem::Method(item_method) = item {
            let method_args = &item_method.sig.inputs;
            let method_ident = &item_method.sig.ident;

            let param_names = method_args.iter().map(|param| {
                if let syn::FnArg::Typed(param) = param {
                    // Get identifier of the parameter
                    &param.pat
                } else {
                    panic!("Using 'self' in an RPC call is not allowed for now.");
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
        impl srpc::server::Service for #self_ident {
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
