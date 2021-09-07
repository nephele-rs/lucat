use super::{Attributes, Method, Service};
use super::{generate_doc_comments, naive_snake_case};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

pub fn generate<T: Service>(
    service: &T,
    emit_package: bool,
    proto_path: &str,
    compile_well_known_types: bool,
    attributes: &Attributes,
) -> TokenStream {
    let service_ident = quote::format_ident!("{}Client", service.name());
    let client_mod = quote::format_ident!("{}_client", naive_snake_case(&service.name()));
    let methods = generate_methods(service, emit_package, proto_path, compile_well_known_types);

    let connect = generate_connect(&service_ident);
    let service_doc = generate_doc_comments(service.comment());

    let package = if emit_package { service.package() } else { "" };
    let path = format!(
        "{}{}{}",
        package,
        if package.is_empty() { "" } else { "." },
        service.identifier()
    );

    let mod_attributes = attributes.for_mod(package);
    let struct_attributes = attributes.for_struct(&path);

    quote! {
        #(#mod_attributes)*
        pub mod #client_mod {
            #![allow(
                unused_variables,
                dead_code,
                missing_docs,
                clippy::let_unit_value,
            )]

            #service_doc
            #(#struct_attributes)*
            pub struct #service_ident<T> {
                inner: lucat::controller::client::Rpc<T>,
            }

            #connect

            impl<T> #service_ident<T> 
            where
            T: lucat::SimpleInstantService + Sync + Send + 'static,
            {
                pub fn new(inner: T) -> Self {
                    let inner = lucat::controller::client::Rpc::new(inner);
                    Self { inner }
                }

                #methods
            }
        }
    }
}

#[cfg(feature = "transport")]
fn generate_connect(service_ident: &syn::Ident) -> TokenStream {
    quote! {
        impl #service_ident<lucat::transport::Endpoint> {
            pub async fn connect(dst: String) -> Result<Self, lucat::Error> {
                let ep = lucat::transport::Endpoint::connect(dst).await?;
                Ok(Self::new(ep))
            }
        }
    }
}

#[cfg(not(feature = "transport"))]
fn generate_connect(_service_ident: &syn::Ident) -> TokenStream {
    TokenStream::new()
}

fn generate_methods<T: Service>(
    service: &T,
    emit_package: bool,
    proto_path: &str,
    compile_well_known_types: bool,
) -> TokenStream {
    let mut stream = TokenStream::new();
    let package = if emit_package { service.package() } else { "" };

    for method in service.methods() {
        let path = format!(
            "/{}{}{}/{}",
            package,
            if package.is_empty() { "" } else { "." },
            service.identifier(),
            method.identifier()
        );

        stream.extend(generate_doc_comments(method.comment()));

        let method = match (method.client_streaming(), method.server_streaming()) {
            (false, false) => generate_unary(method, proto_path, compile_well_known_types, path),
            (false, true) => {
                generate_server_streaming(method, proto_path, compile_well_known_types, path)
            }
            (true, false) => {
                generate_client_streaming(method, proto_path, compile_well_known_types, path)
            }
            (true, true) => generate_streaming(method, proto_path, compile_well_known_types, path),
        };

        stream.extend(method);
    }

    stream
}

fn generate_unary<T: Method>(
    method: &T,
    proto_path: &str,
    compile_well_known_types: bool,
    _path: String,
) -> TokenStream {
    let _codec_name = syn::parse_str::<syn::Path>(T::CODEC_PATH).unwrap();
    let ident = format_ident!("{}", method.name());
    let (request, response) = method.request_response_name(proto_path, compile_well_known_types);

    quote! {
        pub async fn #ident(
            &mut self,
            request: lucat::Request<#request>,
        ) -> Result<lucat::Response<#response>, lucat::Status> {
            let codec = lucat::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/helloworld.Greeter/SayHello");
            self.inner.unary(request, path, codec).await
        }
    }
}

fn generate_server_streaming<T: Method>(
    method: &T,
    proto_path: &str,
    compile_well_known_types: bool,
    path: String,
) -> TokenStream {
    let codec_name = syn::parse_str::<syn::Path>(T::CODEC_PATH).unwrap();
    let ident = format_ident!("{}", method.name());

    let (request, response) = method.request_response_name(proto_path, compile_well_known_types);

    quote! {
        pub async fn #ident(
            &mut self,
            request: impl lucat::IntoRequest<#request>,
        ) -> Result<lucat::Response<lucat::codec::Streaming<#response>>, lucat::Status> {
            self.inner.ready().await.map_err(|e| {
                        lucat::Status::new(lucat::Code::Unknown, format!("Service was not ready: {}", e.into()))
            })?;
            let codec = #codec_name::default();
           let path = http::uri::PathAndQuery::from_static(#path);
           self.inner.server_streaming(request.into_request(), path, codec).await
        }
    }
}

fn generate_client_streaming<T: Method>(
    method: &T,
    proto_path: &str,
    compile_well_known_types: bool,
    path: String,
) -> TokenStream {
    let codec_name = syn::parse_str::<syn::Path>(T::CODEC_PATH).unwrap();
    let ident = format_ident!("{}", method.name());

    let (request, response) = method.request_response_name(proto_path, compile_well_known_types);

    quote! {
        pub async fn #ident(
            &mut self,
            request: impl lucat::IntoStreamingRequest<Message = #request>
        ) -> Result<lucat::Response<#response>, lucat::Status> {
            self.inner.ready().await.map_err(|e| {
                        lucat::Status::new(lucat::Code::Unknown, format!("Service was not ready: {}", e.into()))
            })?;
            let codec = #codec_name::default();
            let path = http::uri::PathAndQuery::from_static(#path);
            self.inner.client_streaming(request.into_streaming_request(), path, codec).await
        }
    }
}

fn generate_streaming<T: Method>(
    method: &T,
    proto_path: &str,
    compile_well_known_types: bool,
    path: String,
) -> TokenStream {
    let codec_name = syn::parse_str::<syn::Path>(T::CODEC_PATH).unwrap();
    let ident = format_ident!("{}", method.name());

    let (request, response) = method.request_response_name(proto_path, compile_well_known_types);

    quote! {
        pub async fn #ident(
            &mut self,
            request: impl lucat::IntoStreamingRequest<Message = #request>
        ) -> Result<lucat::Response<lucat::codec::Streaming<#response>>, lucat::Status> {
            self.inner.ready().await.map_err(|e| {
                        lucat::Status::new(lucat::Code::Unknown, format!("Service was not ready: {}", e.into()))
            })?;
            let codec = #codec_name::default();
           let path = http::uri::PathAndQuery::from_static(#path);
           self.inner.streaming(request.into_streaming_request(), path, codec).await
        }
    }
}
