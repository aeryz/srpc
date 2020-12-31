extern crate proc_macro;

use {proc_macro::TokenStream, quote::quote, syn::parse_macro_input};

/// Generate RPC calls.
/// # Example
/// ```no_run
/// trait Service {
///     async fn foo(data: i32) -> i32;
/// }
/// ```
/// # Expansion
/// ```no_run
/// struct Service;
///
/// impl Service {
///     async fn foo(client: &srpc::client::Client, data: i32) -> srpc::Result<i32> {
///         // Small trick to make serde work
///         #[derive(serde::Serialize)]
///         struct Args { data: i32 };
///
///         let response = client.call(
///             srpc::json_rpc::Request::new(
///                 String::from(stringify!(#method_ident)),
///                 serde_json::to_value(Args { #(#param_names,)* }).unwrap(),
///                 None /* Id is handled in client.call */
///             )).await;
///
///         if response.error.is_some() {
///             Err(response.error.unwrap().into())
///         } else {
///             if response.result.is_some() {
///                 Ok(serde_json::from_value(response.result.unwrap())?)
///             } else {
///                 Err(srpc::json_rpc::Error::new(
///                         srpc::json_rpc::ErrorKind::InternalError,
///                         None).into())
///             }
///         }
///     }
/// }
/// ```
#[proc_macro_attribute]
pub fn client(_attrs: TokenStream, input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as syn::ItemTrait);

    let self_ident = &input.ident;
    let methods = input.items.iter().map(|item| {
        if let syn::TraitItem::Method(item_method) = item {
            let mut is_notif = false;
            if !item_method.attrs.is_empty() {
                for attr in &item_method.attrs {
                    if let Some(segment) = attr.path.segments.first() {
                        if segment.ident == "notification" && attr.path.segments.len() == 1 {
                            is_notif = true;
                            break;
                        }
                    }
                }
            }
            let method_args = &item_method.sig.inputs;
            let method_ident = &item_method.sig.ident;

            // There is no point to have 'self' in an RPC method and for simplicity's sake,
            // it is ignored.
            let param_names = method_args.iter().map(|param| {
                if let syn::FnArg::Typed(param) = param {
                    &param.pat
                } else {
                    panic!("Using 'self' in an RPC call is not allowed for now.");
                }
            });

            let mut return_type = None;
            if let syn::ReturnType::Type(_, ret_type) = &item_method.sig.output {
                if let syn::Type::Tuple(tuple) = ret_type.as_ref() {
                    if !tuple.elems.is_empty() {
                        return_type = Some(ret_type);
                    }
                } else {
                    return_type = Some(ret_type);
                }
            }

            if is_notif && return_type.is_some() {
                panic!("Notification functions should return ()");
            }

            if method_args.is_empty() && return_type.is_none() {
                quote! {
                    async fn #method_ident(client: &srpc::client::Client)
                        -> srpc::Result<()> {

                        let request = srpc::json_rpc::Request::new(
                            String::from(stringify!(#method_ident)),
                            serde_json::Value::Null,
                            None /* Id is handled in "client.call()" */
                        );

                        if #is_notif {
                            let _ = client.notify(request).await?;
                        } else {
                            let _ = client.call(request).await?;
                        }

                        Ok(())
                    }
                }
            } else if method_args.is_empty() && return_type.is_some() {
                let ret_type = return_type.unwrap();
                quote! {
                    async fn #method_ident(client: &srpc::client::Client)
                        -> srpc::Result<#ret_type> {
                        let response = client.call(srpc::json_rpc::Request::new(
                                String::from(stringify!(#method_ident)),
                                serde_json::Value::Null,
                                None
                            )).await?;

                        if response.error.is_some() {
                            Err(response.error.unwrap().into())
                        } else {
                            if response.result.is_some() {
                                Ok(serde_json::from_value(response.result.unwrap())?)
                            } else {
                                Err(srpc::json_rpc::Error::new(
                                        srpc::json_rpc::ErrorKind::InternalError,
                                        None).into())
                            }
                        }
                    }
                }
            } else if !method_args.is_empty() && return_type.is_none() {
                quote! {
                    async fn #method_ident(client: &srpc::client::Client, #method_args)
                        -> srpc::Result<()> {

                        #[derive(serde::Serialize)]
                        struct Args { #method_args }

                        let request = srpc::json_rpc::Request::new(
                            String::from(stringify!(#method_ident)),
                            serde_json::to_value(Args { #(#param_names,)* }).unwrap(),
                            None
                        );

                        if #is_notif {
                            let _ = client.notify(request).await?;
                        } else {
                            let _ = client.call(request).await?;
                        }

                        Ok(())
                    }
                }
            } else {
                let ret_type = return_type.unwrap();
                quote! {
                    async fn #method_ident(client: &srpc::client::Client, #method_args)
                        -> srpc::Result<#ret_type> {

                        #[derive(serde::Serialize)]
                        struct Args { #method_args }

                        let response = client.call(
                            srpc::json_rpc::Request::new(
                                String::from(stringify!(#method_ident)),
                                serde_json::to_value(Args { #(#param_names,)* }).unwrap(),
                                None
                            )).await?;

                        if response.error.is_some() {
                            Err(response.error.unwrap().into())
                        } else {
                            if response.result.is_some() {
                                Ok(serde_json::from_value(response.result.unwrap())?)
                            } else {
                                Err(srpc::json_rpc::Error::new(
                                        srpc::json_rpc::ErrorKind::InternalError,
                                        None).into())
                            }
                        }
                    }
                }
            }
        } else {
            panic!("Only methods are allowed in an srpc client.");
        }
    });

    TokenStream::from(quote! {
        struct #self_ident;
        impl #self_ident {
            #(#methods)*
        }
    })
}

/// Generates an RPC service. Note that RPC methods must be defined as
/// 'async' for now.
///
/// # Example
/// ```no_run
/// struct Service;
///
/// #[srpc::service]
/// impl StrService {
///     async fn contains(data: String, elem: String) -> bool {
///         data.contains(&elem)
///     }
/// }
///
/// ```
///
/// # Expansion
///```no_run
///struct StrService;                                                                            
///impl StrService {
///    async fn contains(data: String, elem: String) -> bool {
///        data.contains(&elem)
///    }
///}
///impl StrService {
///    async fn call(fn_name: String, args: serde_json::Value) -> srpc::Result<serde_json::Value> {
///        Ok(match fn_name.as_str() {
///            "contains" => {
///                struct Args {
///                    data: String,
///                    elem: String,
///                }
///                let Args { data, elem } = serde_json::from_value(args)?;
///                serde_json::to_value(&StrService::contains(data, elem).await)?
///             }
///             _ => return Err(String::new().into()),
///        })
///    }
///     
///    fn caller(
///        fn_name: String,
///        args: serde_json::Value,
///  ) -> std::pin::Pin<Box<dyn std::future::Future<Output = srpc::Result<serde_json::Value>> + Send>>
///
///    {
///        Box::pin(Self::call(fn_name, args))
///    }
///}
///```
#[proc_macro_attribute]
pub fn service(_attrs: TokenStream, input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as syn::ItemImpl);
    let self_ident = &input.self_ty;
    let match_arms = input.items.iter().map(|item| {
        if let syn::ImplItem::Method(item_method) = item {
            let method_args = &item_method.sig.inputs;
            let method_ident = &item_method.sig.ident;
            let method_block = &item_method.block;

            // We only get Typed parameters and ignore 'self' because there is no
            // point to have 'self' in the parameters and it also breaks the code.
            let param_names = method_args.iter().map(|param| {
                if let syn::FnArg::Typed(param) = param {
                    // Get the identifier of the parameter
                    if let syn::Pat::Ident(ref param_ident) = *param.pat {
                        if param_ident.ident == "self" || param_ident.ident == "context" {
                            None
                        } else {
                            Some(&param.pat)
                        }
                    }
                    else {
                        panic!("Unexpected ident");
                    }
                } else {
                    panic!("'self' can only be used in format 'self: Arc<Self>' in an RPC call is not allowed for now.");
                }
            }).filter(|param| param.is_some());

            let args = method_args.iter().map(|param| {
                if let syn::FnArg::Typed(param) = param {
                    // Get the identifier of the parameter
                    if let syn::Pat::Ident(ref param_ident) = *param.pat {
                        if param_ident.ident == "self" || param_ident.ident == "context" {
                            None
                        } else {
                            Some(param)
                        }
                    }
                    else {
                        panic!("Unexpected ident");
                    }
                } else {
                    panic!("'self' can only be used in format 'self: Arc<Self>' in an RPC call is not allowed for now.");
                }
            }).filter(|param| param.is_some());

            let mut return_type = None;
            if let syn::ReturnType::Type(_, ret_type) = &item_method.sig.output {
                if let syn::Type::Tuple(tuple) = ret_type.as_ref() {
                    if !tuple.elems.is_empty() {
                        return_type = Some(ret_type);
                    }
                } else {
                    return_type = Some(ret_type);
                }
            }

            // Generating the match arms
            if method_args.is_empty() && return_type.is_none() {
                quote! {
                    stringify!(#method_ident) => {
                        
                        async move { #method_block }.await;
                        
                        serde_json::Value::Null
                    }
                }
            } else if method_args.is_empty() && return_type.is_some() {
                quote! {
                    stringify!(#method_ident) => {
                        
                        serde_json::to_value(async move {
                            #method_block
                        }.await).unwrap()
                    }
                }
            } else if !method_args.is_empty() && return_type.is_none() {
                quote! {
                    stringify!(#method_ident) => {
                        #[derive(serde::Deserialize)]
                        struct Args { #(#args,)* };
                        let Args { #(#param_names,)* } = match serde_json::from_value(args) {
                            Ok(args) => args,
                            Err(e) => return Err(srpc::json_rpc::Error::new(
                                                srpc::json_rpc::ErrorKind::InvalidParams,
                                                Some(serde_json::to_value(e.to_string()).unwrap()))),
                        };
                        
                        async move {
                            #method_block  
                        }.await;
                        
                        serde_json::Value::Null
                    }
                }
            } else {
                quote! {
                    stringify!(#method_ident) => {
                        #[derive(serde::Deserialize)]
                        struct Args { #(#args,)* };
                        let Args { #(#param_names,)* } = match serde_json::from_value(args) {
                            Ok(args) => args,
                            Err(e) => return Err(srpc::json_rpc::Error::new(
                                                srpc::json_rpc::ErrorKind::InvalidParams,
                                                Some(serde_json::to_value(e.to_string()).unwrap())))
                        };
                        
                        serde_json::to_value(async move {
                            #method_block
                        }.await).unwrap()
                    }
                }
            }
        } else {
            panic!("Items other than function are not supported right now.");
        }
    });
    let q = quote! {
        #input
        impl #self_ident {
            async fn call(self: Arc<Self>,
                          context: Arc<srpc::server::Context>,
                          fn_name: String,
                          args: serde_json::Value)
                -> std::result::Result<serde_json::Value, srpc::json_rpc::Error> {

                Ok(match fn_name.as_str() {
                    #(#match_arms,)*
                    _ => return Err(srpc::json_rpc::Error::new(srpc::json_rpc::ErrorKind::MethodNotFound, None)),
                })
            }

            fn caller(self: Arc<Self>,
                      context: Arc<srpc::server::Context>,
                      fn_name: String,
                      args: serde_json::Value)
                -> std::pin::Pin<Box<dyn std::future::Future<Output = std::result::Result<serde_json::Value, srpc::json_rpc::Error>> + Send>> {

                Box::pin(self.call(context, fn_name, args))

            }
        }
    };

    TokenStream::from(q)
}
