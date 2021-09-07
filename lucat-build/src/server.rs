use super::{Attributes, Method, Service};
use super::{generate_doc_comment, generate_doc_comments, naive_snake_case};
use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{Ident, Lit, LitStr};

pub fn generate<T: Service>(
    service: &T,
    emit_package: bool,
    proto_path: &str,
    compile_well_known_types: bool,
    attributes: &Attributes,
) -> TokenStream {
    let methods = generate_methods_plain(service, proto_path, compile_well_known_types);

    let server_service = quote::format_ident!("{}Server", service.name());
    let server_trait = quote::format_ident!("{}", service.name());
    let server_mod = quote::format_ident!("{}_server", naive_snake_case(&service.name()));
    let generated_trait = generate_trait(
        service,
        proto_path,
        compile_well_known_types,
        server_trait.clone(),
    );
    let service_doc = generate_doc_comments(service.comment());
    let package = if emit_package { service.package() } else { "" };
    let path = format!(
        "{}{}{}",
        package,
        if package.is_empty() { "" } else { "." },
        service.identifier()
    );
    let transport = generate_transport(&server_service, &server_trait, &path);
    let mod_attributes = attributes.for_mod(package);
    let struct_attributes = attributes.for_struct(&path);

    let compression_enabled = cfg!(feature = "compression");

    let _compression_config_ty = if compression_enabled {
        quote! { EnabledCompressionEncodings }
    } else {
        quote! { () }
    };

    quote! {
        #(#mod_attributes)*
        pub mod #server_mod {
            #![allow(
                unused_variables,
                dead_code,
                missing_docs,
                clippy::let_unit_value,
            )]
            use lucat::generator::*;

            #generated_trait

            #service_doc
            #(#struct_attributes)*
            #[derive(Debug)]
            pub struct #server_service<T: #server_trait> {
                inner: _Inner<T>,
            }

            struct _Inner<T>(Arc<T>);

            impl<T: #server_trait> #server_service<T> {
                pub fn new(inner: T) -> Self {
                    let inner = Arc::new(inner);
                    let inner = _Inner(inner);
                    Self {
                        inner,
                    }
                }
            }

            impl<T: #server_trait> Service<lucat::Request<lucat::Body>> for #server_service<T> {
                type Response = lucat::Response<lucat::Body>;
                type Error = Never;
                type Future = BoxFuture<Self::Response, Self::Error>;

                fn call(&mut self, req: lucat::Request<lucat::Body>) -> Self::Future {
                    let inner = self.inner.clone();

                    #methods
                }
            }

            impl<T: #server_trait> Clone for #server_service<T> {
                fn clone(&self) -> Self {
                    let inner = self.inner.clone();
                    Self {
                        inner,
                    }
                }
            }

            impl<T: #server_trait> Clone for _Inner<T> {
                fn clone(&self) -> Self {
                    Self(self.0.clone())
                }
            }

            impl<T: std::fmt::Debug> std::fmt::Debug for _Inner<T> {
                fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                   write!(f, "{:?}", self.0)
                }
            }

            #transport
        }
    }
}

fn generate_trait<T: Service>(
    service: &T,
    proto_path: &str,
    compile_well_known_types: bool,
    server_trait: Ident,
) -> TokenStream {
    let methods = generate_trait_methods(service, proto_path, compile_well_known_types);
    let trait_doc = generate_doc_comment(&format!(
        "Generated trait containing gRPC methods that should be implemented for use with {}Server.",
        service.name()
    ));

    quote! {
        #trait_doc
        #[async_trait]
        pub trait #server_trait : Send + Sync + 'static {
            #methods
        }
    }
}

fn generate_trait_methods<T: Service>(
    service: &T,
    proto_path: &str,
    compile_well_known_types: bool,
) -> TokenStream {
    let mut stream = TokenStream::new();

    for method in service.methods() {
        let name = quote::format_ident!("{}", method.name());

        let (req_message, res_message) =
            method.request_response_name(proto_path, compile_well_known_types);

        let method_doc = generate_doc_comments(method.comment());

        let method = match (method.client_streaming(), method.server_streaming()) {
            (false, false) => {
                quote! {
                    #method_doc
                    async fn #name(&self, request: lucat::Request<#req_message>)
                        -> Result<lucat::Response<#res_message>, lucat::Status>;
                }
            }
            (true, false) => {
                quote! {
                    #method_doc
                    async fn #name(&self, request: lucat::Request<lucat::Streaming<#req_message>>)
                        -> Result<lucat::Response<#res_message>, lucat::Status>;
                }
            }
            (false, true) => {
                let stream = quote::format_ident!("{}Stream", method.identifier());
                let stream_doc = generate_doc_comment(&format!(
                    "Server streaming response type for the {} method.",
                    method.identifier()
                ));

                quote! {
                    #stream_doc
                    type #stream: futures_core::Stream<Item = Result<#res_message, lucat::Status>> + Send + Sync + 'static;

                    #method_doc
                    async fn #name(&self, request: lucat::Request<#req_message>)
                        -> Result<lucat::Response<Self::#stream>, lucat::Status>;
                }
            }
            (true, true) => {
                let stream = quote::format_ident!("{}Stream", method.identifier());
                let stream_doc = generate_doc_comment(&format!(
                    "Server streaming response type for the {} method.",
                    method.identifier()
                ));

                quote! {
                    #stream_doc
                    type #stream: futures_core::Stream<Item = Result<#res_message, lucat::Status>> + Send + Sync + 'static;

                    #method_doc
                    async fn #name(&self, request: lucat::Request<lucat::Streaming<#req_message>>)
                        -> Result<lucat::Response<Self::#stream>, lucat::Status>;
                }
            }
        };

        stream.extend(method);
    }

    stream
}

fn generate_transport(
    _server_service: &syn::Ident,
    _server_trait: &syn::Ident,
    _service_name: &str,
) -> TokenStream {
    TokenStream::new()
}

fn generate_methods_plain<T: Service>(
    service: &T,
    proto_path: &str,
    compile_well_known_types: bool,
) -> TokenStream {
    let mut stream = TokenStream::new();

    for method in service.methods() {
        let path = format!(
            "/",
        );
        let _method_path = Lit::Str(LitStr::new(&path, Span::call_site()));
        let ident = quote::format_ident!("{}", method.name());
        let server_trait = quote::format_ident!("{}", service.name());

        let method_stream = generate_unary(
            method,
            proto_path,
            compile_well_known_types,
            ident,
            server_trait,
        );

        let method = quote! {
            #method_stream
        };
        stream.extend(method);
    }

    stream
}

#[allow(dead_code)]
fn generate_methods<T: Service>(
    service: &T,
    proto_path: &str,
    compile_well_known_types: bool,
) -> TokenStream {
    let mut stream = TokenStream::new();

    for method in service.methods() {
        let path = format!(
            "/",
        );
        let method_path = Lit::Str(LitStr::new(&path, Span::call_site()));
        let ident = quote::format_ident!("{}", method.name());
        let server_trait = quote::format_ident!("{}", service.name());

        let method_stream = generate_unary(
            method,
            proto_path,
            compile_well_known_types,
            ident,
            server_trait,
        );

        let method = quote! {
            #method_path => {
                #method_stream
            }
        };
        stream.extend(method);
    }

    stream
}

fn generate_unary<T: Method>(
    method: &T,
    proto_path: &str,
    compile_well_known_types: bool,
    method_ident: Ident,
    server_trait: Ident,
) -> TokenStream {
    let codec_name = syn::parse_str::<syn::Path>(T::CODEC_PATH).unwrap();

    let service_ident = quote::format_ident!("{}", method.identifier());

    let (request, response) = method.request_response_name(proto_path, compile_well_known_types);

    quote! {
        #[allow(non_camel_case_types)]
        struct #service_ident<T: #server_trait >(pub Arc<T>);

        impl<T: #server_trait> lucat::server::UnaryService<#request> for #service_ident<T> {
            type Response = #response;
            type Future = BoxFuture<lucat::Response<Self::Response>, lucat::Status>;

            fn call(&mut self, request: lucat::Request<#request>) -> Self::Future {
                let inner = self.0.clone();
                let fut = async move {
                    (*inner).#method_ident(request).await
                };
                Box::pin(fut)
            }
        }

        let inner = self.inner.clone();
        let fut = async move {
            let inner = inner.0;
            let method = #service_ident(inner);
            let codec = #codec_name::default();

            let mut grpc = lucat::server::Rpc::new(codec);

            let res = grpc.unary(method, req).await;
            Ok(res)
        };

        Box::pin(fut)
    }
}
