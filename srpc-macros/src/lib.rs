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
                        let response: srpc::protocol::SrpcResponse<()> = serde_json::from_slice(&client.call(
                                srpc::protocol::SrpcRequest::new(
                                    "str-service", 
                                    stringify!(#method_ident), 
                                    ()
                                ))?)?;
                        let _ = srpc::utils::throw_if_error(response.status_code)?;
                        Ok(())
                    }
                }
            } else if method_args.is_empty() && return_type.is_some() {
                let ret_type = return_type.unwrap();
                quote! {
                    fn #method_ident(client: &mut srpc::client::Client) -> srpc::Result<#ret_type> {
                        let response: srpc::protocol::SrpcResponse<#ret_type> = serde_json::from_slice(&client.call(
                                    srpc::protocol::SrpcRequest::new(
                                        "str-service", 
                                        stringify!(#method_ident), 
                                        ()
                                    ))?)?;
                        let _ = srpc::utils::throw_if_error(response.status_code)?;
                        Ok(response.data)
                    }
                }
            } else if !method_args.is_empty() && return_type.is_none() {
                quote! {
                    fn #method_ident(client: &mut srpc::client::Client, #method_args) -> srpc::Result<()> {
                        #[derive(serde::Serialize)]
                        struct Args { #method_args }
                        let response: srpc::protocol::SrpcResponse<()> = serde_json::from_slice(&client.call(
                            srpc::protocol::SrpcRequest::new(
                                "str-service", 
                                stringify!(#method_ident), 
                                Args { #(#param_names,)* }
                            ))?)?;
                        let _ = srpc::utils::throw_if_error(response.status_code)?;
                        Ok(())
                    }
                }
            } else {
                let ret_type = return_type.unwrap();
                quote! {
                    fn #method_ident(client: &mut srpc::client::Client, #method_args) -> srpc::Result<#ret_type> {
                        #[derive(serde::Serialize)]
                        struct Args { #method_args }
                        let response: srpc::protocol::SrpcResponse<#ret_type> = serde_json::from_slice(&client.call(
                                srpc::protocol::SrpcRequest::new(
                                        "str-service", 
                                        stringify!(#method_ident), 
                                        Args { #(#param_names,)* })
                                )?)?;
                        let _ = srpc::utils::throw_if_error(response.status_code)?;
                        Ok(response.data)
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
                        serde_json::Value::Null
                    }
                }
            } else if method_args.is_empty() && return_type.is_some() {
                quote! {
                    stringify!(#method_ident) => {
                        serde_json::to_value(&#self_ident::#method_ident().await)?
                    }
                }
            } else if !method_args.is_empty() && return_type.is_none() {
                let param_names_clone = param_names.clone();
                quote! {
                    stringify!(#method_ident) => {
                        #[derive(serde::Deserialize)]
                        struct Args { #method_args }; // TODO: Reference problem
                        let Args { #(#param_names,)* } = serde_json::from_value(args)?;
                        #self_ident::#method_ident(#(#param_names_clone,)*).await;
                        serde_json::Value::Null
                    }
                }
            } else {
                let param_names_clone = param_names.clone();
                quote! {
                    stringify!(#method_ident) => {
                        #[derive(serde::Deserialize)]
                        struct Args { #method_args }; // TODO: Reference problem
                        let Args { #(#param_names,)* } = serde_json::from_value(args)?;
                        serde_json::to_value(&#self_ident::#method_ident(#(#param_names_clone,)*).await)?
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
            async fn call(fn_name: String, args: serde_json::Value) -> srpc::Result<serde_json::Value> {
                let ret_str = match fn_name.as_str() {
                    #(#match_arms,)*
                    _ => return Err(String::new().into())
                };

                Ok(ret_str)
            }
        }
    };

    TokenStream::from(q)
}
