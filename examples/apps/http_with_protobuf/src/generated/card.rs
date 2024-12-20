// This file is @generated by prost-build.
/// The service entity
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CardWire {
    #[prost(string, tag = "1")]
    pub id: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub owner_name: ::prost::alloc::string::String,
    #[prost(string, tag = "3")]
    pub card_id: ::prost::alloc::string::String,
    #[prost(message, optional, tag = "4")]
    pub created_at: ::core::option::Option<::prost_wkt_types::Timestamp>,
    #[prost(message, optional, tag = "5")]
    pub updated_at: ::core::option::Option<::prost_wkt_types::Timestamp>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CreateCardRequest {
    #[prost(string, tag = "1")]
    pub owner_name: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub card_id: ::prost::alloc::string::String,
    #[prost(bool, tag = "3")]
    pub debug: bool,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CreateCardResponse {
    #[prost(message, optional, tag = "1")]
    pub card: ::core::option::Option<CardWire>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetCardRequest {
    #[prost(string, tag = "1")]
    pub id: ::prost::alloc::string::String,
    #[prost(bool, tag = "2")]
    pub debug: bool,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetCardResponse {
    #[prost(message, optional, tag = "1")]
    pub card: ::core::option::Option<CardWire>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct UpdateCardRequest {
    #[prost(string, tag = "1")]
    pub id: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub owner_name: ::prost::alloc::string::String,
    #[prost(string, tag = "3")]
    pub card_id: ::prost::alloc::string::String,
    #[prost(bool, tag = "4")]
    pub debug: bool,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct UpdateCardResponse {
    #[prost(message, optional, tag = "1")]
    pub card: ::core::option::Option<CardWire>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DeleteCardRequest {
    #[prost(string, tag = "1")]
    pub id: ::prost::alloc::string::String,
    #[prost(bool, tag = "2")]
    pub debug: bool,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DeleteCardResponse {
    #[prost(message, optional, tag = "1")]
    pub card: ::core::option::Option<CardWire>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum CardType {
    Unspecified = 0,
    Credit = 1,
    Debit = 2,
}
impl CardType {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            Self::Unspecified => "CARD_TYPE_UNSPECIFIED",
            Self::Credit => "CARD_TYPE_CREDIT",
            Self::Debit => "CARD_TYPE_DEBIT",
        }
    }
    /// Creates an enum from field names used in the ProtoBuf definition.
    pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
        match value {
            "CARD_TYPE_UNSPECIFIED" => Some(Self::Unspecified),
            "CARD_TYPE_CREDIT" => Some(Self::Credit),
            "CARD_TYPE_DEBIT" => Some(Self::Debit),
            _ => None,
        }
    }
}
/// Generated client implementations.
pub mod card_service_client {
    #![allow(
        unused_variables,
        dead_code,
        missing_docs,
        clippy::wildcard_imports,
        clippy::let_unit_value,
    )]
    use tonic::codegen::*;
    use tonic::codegen::http::Uri;
    /// The service definition
    #[derive(Debug, Clone)]
    pub struct CardServiceClient<T> {
        inner: tonic::client::Grpc<T>,
    }
    impl CardServiceClient<tonic::transport::Channel> {
        /// Attempt to create a new client by connecting to a given endpoint.
        pub async fn connect<D>(dst: D) -> Result<Self, tonic::transport::Error>
        where
            D: TryInto<tonic::transport::Endpoint>,
            D::Error: Into<StdError>,
        {
            let conn = tonic::transport::Endpoint::new(dst)?.connect().await?;
            Ok(Self::new(conn))
        }
    }
    impl<T> CardServiceClient<T>
    where
        T: tonic::client::GrpcService<tonic::body::BoxBody>,
        T::Error: Into<StdError>,
        T::ResponseBody: Body<Data = Bytes> + std::marker::Send + 'static,
        <T::ResponseBody as Body>::Error: Into<StdError> + std::marker::Send,
    {
        pub fn new(inner: T) -> Self {
            let inner = tonic::client::Grpc::new(inner);
            Self { inner }
        }
        pub fn with_origin(inner: T, origin: Uri) -> Self {
            let inner = tonic::client::Grpc::with_origin(inner, origin);
            Self { inner }
        }
        pub fn with_interceptor<F>(
            inner: T,
            interceptor: F,
        ) -> CardServiceClient<InterceptedService<T, F>>
        where
            F: tonic::service::Interceptor,
            T::ResponseBody: Default,
            T: tonic::codegen::Service<
                http::Request<tonic::body::BoxBody>,
                Response = http::Response<
                    <T as tonic::client::GrpcService<tonic::body::BoxBody>>::ResponseBody,
                >,
            >,
            <T as tonic::codegen::Service<
                http::Request<tonic::body::BoxBody>,
            >>::Error: Into<StdError> + std::marker::Send + std::marker::Sync,
        {
            CardServiceClient::new(InterceptedService::new(inner, interceptor))
        }
        /// Compress requests with the given encoding.
        ///
        /// This requires the server to support it otherwise it might respond with an
        /// error.
        #[must_use]
        pub fn send_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.inner = self.inner.send_compressed(encoding);
            self
        }
        /// Enable decompressing responses.
        #[must_use]
        pub fn accept_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.inner = self.inner.accept_compressed(encoding);
            self
        }
        /// Limits the maximum size of a decoded message.
        ///
        /// Default: `4MB`
        #[must_use]
        pub fn max_decoding_message_size(mut self, limit: usize) -> Self {
            self.inner = self.inner.max_decoding_message_size(limit);
            self
        }
        /// Limits the maximum size of an encoded message.
        ///
        /// Default: `usize::MAX`
        #[must_use]
        pub fn max_encoding_message_size(mut self, limit: usize) -> Self {
            self.inner = self.inner.max_encoding_message_size(limit);
            self
        }
        pub async fn create_card(
            &mut self,
            request: impl tonic::IntoRequest<super::CreateCardRequest>,
        ) -> std::result::Result<
            tonic::Response<super::CreateCardResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::unknown(
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/card.CardService/CreateCard",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("card.CardService", "CreateCard"));
            self.inner.unary(req, path, codec).await
        }
        pub async fn get_card(
            &mut self,
            request: impl tonic::IntoRequest<super::GetCardRequest>,
        ) -> std::result::Result<
            tonic::Response<super::GetCardResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::unknown(
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/card.CardService/GetCard");
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new("card.CardService", "GetCard"));
            self.inner.unary(req, path, codec).await
        }
        pub async fn update_card(
            &mut self,
            request: impl tonic::IntoRequest<super::UpdateCardRequest>,
        ) -> std::result::Result<
            tonic::Response<super::UpdateCardResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::unknown(
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/card.CardService/UpdateCard",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("card.CardService", "UpdateCard"));
            self.inner.unary(req, path, codec).await
        }
        pub async fn delete_card(
            &mut self,
            request: impl tonic::IntoRequest<super::DeleteCardRequest>,
        ) -> std::result::Result<
            tonic::Response<super::DeleteCardResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::unknown(
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/card.CardService/DeleteCard",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("card.CardService", "DeleteCard"));
            self.inner.unary(req, path, codec).await
        }
    }
}
/// Generated server implementations.
pub mod card_service_server {
    #![allow(
        unused_variables,
        dead_code,
        missing_docs,
        clippy::wildcard_imports,
        clippy::let_unit_value,
    )]
    use tonic::codegen::*;
    /// Generated trait containing gRPC methods that should be implemented for use with CardServiceServer.
    #[async_trait]
    pub trait CardService: std::marker::Send + std::marker::Sync + 'static {
        async fn create_card(
            &self,
            request: tonic::Request<super::CreateCardRequest>,
        ) -> std::result::Result<
            tonic::Response<super::CreateCardResponse>,
            tonic::Status,
        >;
        async fn get_card(
            &self,
            request: tonic::Request<super::GetCardRequest>,
        ) -> std::result::Result<tonic::Response<super::GetCardResponse>, tonic::Status>;
        async fn update_card(
            &self,
            request: tonic::Request<super::UpdateCardRequest>,
        ) -> std::result::Result<
            tonic::Response<super::UpdateCardResponse>,
            tonic::Status,
        >;
        async fn delete_card(
            &self,
            request: tonic::Request<super::DeleteCardRequest>,
        ) -> std::result::Result<
            tonic::Response<super::DeleteCardResponse>,
            tonic::Status,
        >;
    }
    /// The service definition
    #[derive(Debug)]
    pub struct CardServiceServer<T> {
        inner: Arc<T>,
        accept_compression_encodings: EnabledCompressionEncodings,
        send_compression_encodings: EnabledCompressionEncodings,
        max_decoding_message_size: Option<usize>,
        max_encoding_message_size: Option<usize>,
    }
    impl<T> CardServiceServer<T> {
        pub fn new(inner: T) -> Self {
            Self::from_arc(Arc::new(inner))
        }
        pub fn from_arc(inner: Arc<T>) -> Self {
            Self {
                inner,
                accept_compression_encodings: Default::default(),
                send_compression_encodings: Default::default(),
                max_decoding_message_size: None,
                max_encoding_message_size: None,
            }
        }
        pub fn with_interceptor<F>(
            inner: T,
            interceptor: F,
        ) -> InterceptedService<Self, F>
        where
            F: tonic::service::Interceptor,
        {
            InterceptedService::new(Self::new(inner), interceptor)
        }
        /// Enable decompressing requests with the given encoding.
        #[must_use]
        pub fn accept_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.accept_compression_encodings.enable(encoding);
            self
        }
        /// Compress responses with the given encoding, if the client supports it.
        #[must_use]
        pub fn send_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.send_compression_encodings.enable(encoding);
            self
        }
        /// Limits the maximum size of a decoded message.
        ///
        /// Default: `4MB`
        #[must_use]
        pub fn max_decoding_message_size(mut self, limit: usize) -> Self {
            self.max_decoding_message_size = Some(limit);
            self
        }
        /// Limits the maximum size of an encoded message.
        ///
        /// Default: `usize::MAX`
        #[must_use]
        pub fn max_encoding_message_size(mut self, limit: usize) -> Self {
            self.max_encoding_message_size = Some(limit);
            self
        }
    }
    impl<T, B> tonic::codegen::Service<http::Request<B>> for CardServiceServer<T>
    where
        T: CardService,
        B: Body + std::marker::Send + 'static,
        B::Error: Into<StdError> + std::marker::Send + 'static,
    {
        type Response = http::Response<tonic::body::BoxBody>;
        type Error = std::convert::Infallible;
        type Future = BoxFuture<Self::Response, Self::Error>;
        fn poll_ready(
            &mut self,
            _cx: &mut Context<'_>,
        ) -> Poll<std::result::Result<(), Self::Error>> {
            Poll::Ready(Ok(()))
        }
        fn call(&mut self, req: http::Request<B>) -> Self::Future {
            match req.uri().path() {
                "/card.CardService/CreateCard" => {
                    #[allow(non_camel_case_types)]
                    struct CreateCardSvc<T: CardService>(pub Arc<T>);
                    impl<
                        T: CardService,
                    > tonic::server::UnaryService<super::CreateCardRequest>
                    for CreateCardSvc<T> {
                        type Response = super::CreateCardResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::CreateCardRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as CardService>::create_card(&inner, request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let method = CreateCardSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/card.CardService/GetCard" => {
                    #[allow(non_camel_case_types)]
                    struct GetCardSvc<T: CardService>(pub Arc<T>);
                    impl<
                        T: CardService,
                    > tonic::server::UnaryService<super::GetCardRequest>
                    for GetCardSvc<T> {
                        type Response = super::GetCardResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::GetCardRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as CardService>::get_card(&inner, request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let method = GetCardSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/card.CardService/UpdateCard" => {
                    #[allow(non_camel_case_types)]
                    struct UpdateCardSvc<T: CardService>(pub Arc<T>);
                    impl<
                        T: CardService,
                    > tonic::server::UnaryService<super::UpdateCardRequest>
                    for UpdateCardSvc<T> {
                        type Response = super::UpdateCardResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::UpdateCardRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as CardService>::update_card(&inner, request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let method = UpdateCardSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/card.CardService/DeleteCard" => {
                    #[allow(non_camel_case_types)]
                    struct DeleteCardSvc<T: CardService>(pub Arc<T>);
                    impl<
                        T: CardService,
                    > tonic::server::UnaryService<super::DeleteCardRequest>
                    for DeleteCardSvc<T> {
                        type Response = super::DeleteCardResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::DeleteCardRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as CardService>::delete_card(&inner, request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let method = DeleteCardSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                _ => {
                    Box::pin(async move {
                        let mut response = http::Response::new(empty_body());
                        let headers = response.headers_mut();
                        headers
                            .insert(
                                tonic::Status::GRPC_STATUS,
                                (tonic::Code::Unimplemented as i32).into(),
                            );
                        headers
                            .insert(
                                http::header::CONTENT_TYPE,
                                tonic::metadata::GRPC_CONTENT_TYPE,
                            );
                        Ok(response)
                    })
                }
            }
        }
    }
    impl<T> Clone for CardServiceServer<T> {
        fn clone(&self) -> Self {
            let inner = self.inner.clone();
            Self {
                inner,
                accept_compression_encodings: self.accept_compression_encodings,
                send_compression_encodings: self.send_compression_encodings,
                max_decoding_message_size: self.max_decoding_message_size,
                max_encoding_message_size: self.max_encoding_message_size,
            }
        }
    }
    /// Generated gRPC service name
    pub const SERVICE_NAME: &str = "card.CardService";
    impl<T> tonic::server::NamedService for CardServiceServer<T> {
        const NAME: &'static str = SERVICE_NAME;
    }
}
