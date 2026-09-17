// @generated
/// Generated server implementations.
pub mod model_material_service_server {
    #![allow(
        unused_variables,
        dead_code,
        missing_docs,
        clippy::wildcard_imports,
        clippy::let_unit_value,
    )]
    use tonic::codegen::*;
    /// Generated trait containing gRPC methods that should be implemented for use with ModelMaterialServiceServer.
    #[async_trait]
    pub trait ModelMaterialService: std::marker::Send + std::marker::Sync + 'static {
        ///
        async fn get_embedding_config(
            &self,
            request: tonic::Request<()>,
        ) -> std::result::Result<tonic::Response<super::EmbeddingConfig>, tonic::Status>;
        ///
        async fn list_embedding_needs(
            &self,
            request: tonic::Request<super::EmbeddingNeedsRequest>,
        ) -> std::result::Result<
            tonic::Response<super::EmbeddingNeedsResponse>,
            tonic::Status,
        >;
        ///
        async fn commit_embedding(
            &self,
            request: tonic::Request<super::CommitEmbeddingRequest>,
        ) -> std::result::Result<tonic::Response<()>, tonic::Status>;
        ///
        async fn commit_interpretation(
            &self,
            request: tonic::Request<super::CommitInterpretationRequest>,
        ) -> std::result::Result<
            tonic::Response<super::super::super::v1alpha1::DerivedRepresentation>,
            tonic::Status,
        >;
    }
    ///
    #[derive(Debug)]
    pub struct ModelMaterialServiceServer<T> {
        inner: Arc<T>,
        accept_compression_encodings: EnabledCompressionEncodings,
        send_compression_encodings: EnabledCompressionEncodings,
        max_decoding_message_size: Option<usize>,
        max_encoding_message_size: Option<usize>,
    }
    impl<T> ModelMaterialServiceServer<T> {
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
    impl<T, B> tonic::codegen::Service<http::Request<B>>
    for ModelMaterialServiceServer<T>
    where
        T: ModelMaterialService,
        B: Body + std::marker::Send + 'static,
        B::Error: Into<StdError> + std::marker::Send + 'static,
    {
        type Response = http::Response<tonic::body::Body>;
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
                "/nous.wave.kernel.v1alpha1.ModelMaterialService/GetEmbeddingConfig" => {
                    #[allow(non_camel_case_types)]
                    struct GetEmbeddingConfigSvc<T: ModelMaterialService>(pub Arc<T>);
                    impl<T: ModelMaterialService> tonic::server::UnaryService<()>
                    for GetEmbeddingConfigSvc<T> {
                        type Response = super::EmbeddingConfig;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(&mut self, request: tonic::Request<()>) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as ModelMaterialService>::get_embedding_config(
                                        &inner,
                                        request,
                                    )
                                    .await
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
                        let method = GetEmbeddingConfigSvc(inner);
                        let codec = tonic_prost::ProstCodec::default();
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
                "/nous.wave.kernel.v1alpha1.ModelMaterialService/ListEmbeddingNeeds" => {
                    #[allow(non_camel_case_types)]
                    struct ListEmbeddingNeedsSvc<T: ModelMaterialService>(pub Arc<T>);
                    impl<
                        T: ModelMaterialService,
                    > tonic::server::UnaryService<super::EmbeddingNeedsRequest>
                    for ListEmbeddingNeedsSvc<T> {
                        type Response = super::EmbeddingNeedsResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::EmbeddingNeedsRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as ModelMaterialService>::list_embedding_needs(
                                        &inner,
                                        request,
                                    )
                                    .await
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
                        let method = ListEmbeddingNeedsSvc(inner);
                        let codec = tonic_prost::ProstCodec::default();
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
                "/nous.wave.kernel.v1alpha1.ModelMaterialService/CommitEmbedding" => {
                    #[allow(non_camel_case_types)]
                    struct CommitEmbeddingSvc<T: ModelMaterialService>(pub Arc<T>);
                    impl<
                        T: ModelMaterialService,
                    > tonic::server::UnaryService<super::CommitEmbeddingRequest>
                    for CommitEmbeddingSvc<T> {
                        type Response = ();
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::CommitEmbeddingRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as ModelMaterialService>::commit_embedding(
                                        &inner,
                                        request,
                                    )
                                    .await
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
                        let method = CommitEmbeddingSvc(inner);
                        let codec = tonic_prost::ProstCodec::default();
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
                "/nous.wave.kernel.v1alpha1.ModelMaterialService/CommitInterpretation" => {
                    #[allow(non_camel_case_types)]
                    struct CommitInterpretationSvc<T: ModelMaterialService>(pub Arc<T>);
                    impl<
                        T: ModelMaterialService,
                    > tonic::server::UnaryService<super::CommitInterpretationRequest>
                    for CommitInterpretationSvc<T> {
                        type Response = super::super::super::v1alpha1::DerivedRepresentation;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::CommitInterpretationRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as ModelMaterialService>::commit_interpretation(
                                        &inner,
                                        request,
                                    )
                                    .await
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
                        let method = CommitInterpretationSvc(inner);
                        let codec = tonic_prost::ProstCodec::default();
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
                        let mut response = http::Response::new(
                            tonic::body::Body::default(),
                        );
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
    impl<T> Clone for ModelMaterialServiceServer<T> {
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
    pub const SERVICE_NAME: &str = "nous.wave.kernel.v1alpha1.ModelMaterialService";
    impl<T> tonic::server::NamedService for ModelMaterialServiceServer<T> {
        const NAME: &'static str = SERVICE_NAME;
    }
}
/// Generated server implementations.
pub mod runtime_store_service_server {
    #![allow(
        unused_variables,
        dead_code,
        missing_docs,
        clippy::wildcard_imports,
        clippy::let_unit_value,
    )]
    use tonic::codegen::*;
    /// Generated trait containing gRPC methods that should be implemented for use with RuntimeStoreServiceServer.
    #[async_trait]
    pub trait RuntimeStoreService: std::marker::Send + std::marker::Sync + 'static {
        ///
        async fn read_runtime(
            &self,
            request: tonic::Request<super::ReadRuntimeRequest>,
        ) -> std::result::Result<
            tonic::Response<super::ReadRuntimeResponse>,
            tonic::Status,
        >;
        ///
        async fn mutate_runtime(
            &self,
            request: tonic::Request<super::MutateRuntimeRequest>,
        ) -> std::result::Result<
            tonic::Response<super::MutateRuntimeResponse>,
            tonic::Status,
        >;
        ///
        async fn check_references(
            &self,
            request: tonic::Request<super::CheckReferencesRequest>,
        ) -> std::result::Result<
            tonic::Response<super::CheckReferencesResponse>,
            tonic::Status,
        >;
    }
    ///
    #[derive(Debug)]
    pub struct RuntimeStoreServiceServer<T> {
        inner: Arc<T>,
        accept_compression_encodings: EnabledCompressionEncodings,
        send_compression_encodings: EnabledCompressionEncodings,
        max_decoding_message_size: Option<usize>,
        max_encoding_message_size: Option<usize>,
    }
    impl<T> RuntimeStoreServiceServer<T> {
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
    impl<T, B> tonic::codegen::Service<http::Request<B>> for RuntimeStoreServiceServer<T>
    where
        T: RuntimeStoreService,
        B: Body + std::marker::Send + 'static,
        B::Error: Into<StdError> + std::marker::Send + 'static,
    {
        type Response = http::Response<tonic::body::Body>;
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
                "/nous.wave.kernel.v1alpha1.RuntimeStoreService/ReadRuntime" => {
                    #[allow(non_camel_case_types)]
                    struct ReadRuntimeSvc<T: RuntimeStoreService>(pub Arc<T>);
                    impl<
                        T: RuntimeStoreService,
                    > tonic::server::UnaryService<super::ReadRuntimeRequest>
                    for ReadRuntimeSvc<T> {
                        type Response = super::ReadRuntimeResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::ReadRuntimeRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as RuntimeStoreService>::read_runtime(&inner, request)
                                    .await
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
                        let method = ReadRuntimeSvc(inner);
                        let codec = tonic_prost::ProstCodec::default();
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
                "/nous.wave.kernel.v1alpha1.RuntimeStoreService/MutateRuntime" => {
                    #[allow(non_camel_case_types)]
                    struct MutateRuntimeSvc<T: RuntimeStoreService>(pub Arc<T>);
                    impl<
                        T: RuntimeStoreService,
                    > tonic::server::UnaryService<super::MutateRuntimeRequest>
                    for MutateRuntimeSvc<T> {
                        type Response = super::MutateRuntimeResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::MutateRuntimeRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as RuntimeStoreService>::mutate_runtime(&inner, request)
                                    .await
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
                        let method = MutateRuntimeSvc(inner);
                        let codec = tonic_prost::ProstCodec::default();
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
                "/nous.wave.kernel.v1alpha1.RuntimeStoreService/CheckReferences" => {
                    #[allow(non_camel_case_types)]
                    struct CheckReferencesSvc<T: RuntimeStoreService>(pub Arc<T>);
                    impl<
                        T: RuntimeStoreService,
                    > tonic::server::UnaryService<super::CheckReferencesRequest>
                    for CheckReferencesSvc<T> {
                        type Response = super::CheckReferencesResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::CheckReferencesRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as RuntimeStoreService>::check_references(
                                        &inner,
                                        request,
                                    )
                                    .await
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
                        let method = CheckReferencesSvc(inner);
                        let codec = tonic_prost::ProstCodec::default();
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
                        let mut response = http::Response::new(
                            tonic::body::Body::default(),
                        );
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
    impl<T> Clone for RuntimeStoreServiceServer<T> {
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
    pub const SERVICE_NAME: &str = "nous.wave.kernel.v1alpha1.RuntimeStoreService";
    impl<T> tonic::server::NamedService for RuntimeStoreServiceServer<T> {
        const NAME: &'static str = SERVICE_NAME;
    }
}
/// Generated server implementations.
pub mod artifact_stream_service_server {
    #![allow(
        unused_variables,
        dead_code,
        missing_docs,
        clippy::wildcard_imports,
        clippy::let_unit_value,
    )]
    use tonic::codegen::*;
    /// Generated trait containing gRPC methods that should be implemented for use with ArtifactStreamServiceServer.
    #[async_trait]
    pub trait ArtifactStreamService: std::marker::Send + std::marker::Sync + 'static {
        ///
        async fn upload_artifact(
            &self,
            request: tonic::Request<tonic::Streaming<super::UploadChunk>>,
        ) -> std::result::Result<
            tonic::Response<super::super::super::v1alpha1::Artifact>,
            tonic::Status,
        >;
        /// Server streaming response type for the DownloadArtifact method.
        type DownloadArtifactStream: tonic::codegen::tokio_stream::Stream<
                Item = std::result::Result<super::DownloadChunk, tonic::Status>,
            >
            + std::marker::Send
            + 'static;
        ///
        async fn download_artifact(
            &self,
            request: tonic::Request<super::DownloadRequest>,
        ) -> std::result::Result<
            tonic::Response<Self::DownloadArtifactStream>,
            tonic::Status,
        >;
    }
    ///
    #[derive(Debug)]
    pub struct ArtifactStreamServiceServer<T> {
        inner: Arc<T>,
        accept_compression_encodings: EnabledCompressionEncodings,
        send_compression_encodings: EnabledCompressionEncodings,
        max_decoding_message_size: Option<usize>,
        max_encoding_message_size: Option<usize>,
    }
    impl<T> ArtifactStreamServiceServer<T> {
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
    impl<T, B> tonic::codegen::Service<http::Request<B>>
    for ArtifactStreamServiceServer<T>
    where
        T: ArtifactStreamService,
        B: Body + std::marker::Send + 'static,
        B::Error: Into<StdError> + std::marker::Send + 'static,
    {
        type Response = http::Response<tonic::body::Body>;
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
                "/nous.wave.kernel.v1alpha1.ArtifactStreamService/UploadArtifact" => {
                    #[allow(non_camel_case_types)]
                    struct UploadArtifactSvc<T: ArtifactStreamService>(pub Arc<T>);
                    impl<
                        T: ArtifactStreamService,
                    > tonic::server::ClientStreamingService<super::UploadChunk>
                    for UploadArtifactSvc<T> {
                        type Response = super::super::super::v1alpha1::Artifact;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<tonic::Streaming<super::UploadChunk>>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as ArtifactStreamService>::upload_artifact(
                                        &inner,
                                        request,
                                    )
                                    .await
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
                        let method = UploadArtifactSvc(inner);
                        let codec = tonic_prost::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.client_streaming(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/nous.wave.kernel.v1alpha1.ArtifactStreamService/DownloadArtifact" => {
                    #[allow(non_camel_case_types)]
                    struct DownloadArtifactSvc<T: ArtifactStreamService>(pub Arc<T>);
                    impl<
                        T: ArtifactStreamService,
                    > tonic::server::ServerStreamingService<super::DownloadRequest>
                    for DownloadArtifactSvc<T> {
                        type Response = super::DownloadChunk;
                        type ResponseStream = T::DownloadArtifactStream;
                        type Future = BoxFuture<
                            tonic::Response<Self::ResponseStream>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::DownloadRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as ArtifactStreamService>::download_artifact(
                                        &inner,
                                        request,
                                    )
                                    .await
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
                        let method = DownloadArtifactSvc(inner);
                        let codec = tonic_prost::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.server_streaming(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                _ => {
                    Box::pin(async move {
                        let mut response = http::Response::new(
                            tonic::body::Body::default(),
                        );
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
    impl<T> Clone for ArtifactStreamServiceServer<T> {
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
    pub const SERVICE_NAME: &str = "nous.wave.kernel.v1alpha1.ArtifactStreamService";
    impl<T> tonic::server::NamedService for ArtifactStreamServiceServer<T> {
        const NAME: &'static str = SERVICE_NAME;
    }
}
/// Generated server implementations.
pub mod authority_service_server {
    #![allow(
        unused_variables,
        dead_code,
        missing_docs,
        clippy::wildcard_imports,
        clippy::let_unit_value,
    )]
    use tonic::codegen::*;
    /// Generated trait containing gRPC methods that should be implemented for use with AuthorityServiceServer.
    #[async_trait]
    pub trait AuthorityService: std::marker::Send + std::marker::Sync + 'static {
        ///
        async fn put_resource(
            &self,
            request: tonic::Request<super::super::super::v1alpha1::PutResourceRequest>,
        ) -> std::result::Result<
            tonic::Response<super::super::super::v1alpha1::ResourceDescriptor>,
            tonic::Status,
        >;
        ///
        async fn get_resource(
            &self,
            request: tonic::Request<super::super::super::v1alpha1::ObjectRequest>,
        ) -> std::result::Result<
            tonic::Response<super::super::super::v1alpha1::ResourceDescriptor>,
            tonic::Status,
        >;
        ///
        async fn list_resources(
            &self,
            request: tonic::Request<super::super::super::v1alpha1::ListRequest>,
        ) -> std::result::Result<
            tonic::Response<super::super::super::v1alpha1::ListResourcesResponse>,
            tonic::Status,
        >;
        ///
        async fn remove_resource(
            &self,
            request: tonic::Request<super::super::super::v1alpha1::ObjectRequest>,
        ) -> std::result::Result<tonic::Response<()>, tonic::Status>;
        ///
        async fn create_tag(
            &self,
            request: tonic::Request<super::super::super::v1alpha1::CreateTagRequest>,
        ) -> std::result::Result<
            tonic::Response<super::super::super::v1alpha1::Tag>,
            tonic::Status,
        >;
        ///
        async fn get_tag(
            &self,
            request: tonic::Request<super::super::super::v1alpha1::ObjectRequest>,
        ) -> std::result::Result<
            tonic::Response<super::super::super::v1alpha1::Tag>,
            tonic::Status,
        >;
        ///
        async fn list_tags(
            &self,
            request: tonic::Request<super::super::super::v1alpha1::ListRequest>,
        ) -> std::result::Result<
            tonic::Response<super::super::super::v1alpha1::ListTagsResponse>,
            tonic::Status,
        >;
        ///
        async fn create_anchor(
            &self,
            request: tonic::Request<super::super::super::v1alpha1::CreateAnchorRequest>,
        ) -> std::result::Result<
            tonic::Response<super::super::super::v1alpha1::Anchor>,
            tonic::Status,
        >;
        ///
        async fn get_anchor(
            &self,
            request: tonic::Request<super::super::super::v1alpha1::ObjectRequest>,
        ) -> std::result::Result<
            tonic::Response<super::super::super::v1alpha1::Anchor>,
            tonic::Status,
        >;
        ///
        async fn list_anchors(
            &self,
            request: tonic::Request<super::super::super::v1alpha1::ListRequest>,
        ) -> std::result::Result<
            tonic::Response<super::super::super::v1alpha1::ListAnchorsResponse>,
            tonic::Status,
        >;
        ///
        async fn create_association(
            &self,
            request: tonic::Request<
                super::super::super::v1alpha1::CreateAssociationRequest,
            >,
        ) -> std::result::Result<
            tonic::Response<super::super::super::v1alpha1::Association>,
            tonic::Status,
        >;
        ///
        async fn get_neighborhood(
            &self,
            request: tonic::Request<super::super::super::v1alpha1::NeighborhoodRequest>,
        ) -> std::result::Result<
            tonic::Response<super::super::super::v1alpha1::NeighborhoodResponse>,
            tonic::Status,
        >;
        ///
        async fn rebind_entity(
            &self,
            request: tonic::Request<super::super::super::v1alpha1::RebindEntityRequest>,
        ) -> std::result::Result<tonic::Response<()>, tonic::Status>;
        ///
        async fn get_status(
            &self,
            request: tonic::Request<()>,
        ) -> std::result::Result<
            tonic::Response<super::super::super::v1alpha1::SystemStatus>,
            tonic::Status,
        >;
        ///
        async fn get_projection_status(
            &self,
            request: tonic::Request<super::super::super::v1alpha1::SubjectRequest>,
        ) -> std::result::Result<
            tonic::Response<super::super::super::v1alpha1::ProjectionStatus>,
            tonic::Status,
        >;
        ///
        async fn consolidate_memory(
            &self,
            request: tonic::Request<
                super::super::super::v1alpha1::ConsolidateMemoryRequest,
            >,
        ) -> std::result::Result<
            tonic::Response<super::super::super::v1alpha1::Memory>,
            tonic::Status,
        >;
        ///
        async fn get_occurrence(
            &self,
            request: tonic::Request<super::super::super::v1alpha1::ObjectRequest>,
        ) -> std::result::Result<
            tonic::Response<super::super::super::v1alpha1::Occurrence>,
            tonic::Status,
        >;
        ///
        async fn get_source_region(
            &self,
            request: tonic::Request<super::super::super::v1alpha1::ObjectRequest>,
        ) -> std::result::Result<
            tonic::Response<super::super::super::v1alpha1::SourceRegion>,
            tonic::Status,
        >;
        ///
        async fn get_derived_representation(
            &self,
            request: tonic::Request<super::super::super::v1alpha1::ObjectRequest>,
        ) -> std::result::Result<
            tonic::Response<super::super::super::v1alpha1::DerivedRepresentation>,
            tonic::Status,
        >;
        ///
        async fn bind_identity(
            &self,
            request: tonic::Request<super::super::super::v1alpha1::BindIdentityRequest>,
        ) -> std::result::Result<
            tonic::Response<super::super::super::v1alpha1::IdentityBinding>,
            tonic::Status,
        >;
        ///
        async fn resolve_identity(
            &self,
            request: tonic::Request<
                super::super::super::v1alpha1::ResolveIdentityRequest,
            >,
        ) -> std::result::Result<
            tonic::Response<super::super::super::v1alpha1::ResolveIdentityResponse>,
            tonic::Status,
        >;
        ///
        async fn build_contribution_batch(
            &self,
            request: tonic::Request<super::super::super::v1alpha1::ProjectionRequest>,
        ) -> std::result::Result<
            tonic::Response<super::super::super::v1alpha1::Projection>,
            tonic::Status,
        >;
        ///
        async fn create_subject(
            &self,
            request: tonic::Request<super::super::super::v1alpha1::CreateSubjectRequest>,
        ) -> std::result::Result<
            tonic::Response<super::super::super::v1alpha1::Subject>,
            tonic::Status,
        >;
        ///
        async fn get_subject(
            &self,
            request: tonic::Request<super::super::super::v1alpha1::SubjectRequest>,
        ) -> std::result::Result<
            tonic::Response<super::super::super::v1alpha1::Subject>,
            tonic::Status,
        >;
        ///
        async fn list_subjects(
            &self,
            request: tonic::Request<super::super::super::v1alpha1::ListRequest>,
        ) -> std::result::Result<
            tonic::Response<super::super::super::v1alpha1::ListSubjectsResponse>,
            tonic::Status,
        >;
        ///
        async fn get_character_seed(
            &self,
            request: tonic::Request<super::super::super::v1alpha1::SubjectRequest>,
        ) -> std::result::Result<
            tonic::Response<super::super::super::v1alpha1::SeedRevision>,
            tonic::Status,
        >;
        ///
        async fn revise_character_seed(
            &self,
            request: tonic::Request<
                super::super::super::v1alpha1::ReviseCharacterSeedRequest,
            >,
        ) -> std::result::Result<
            tonic::Response<super::super::super::v1alpha1::SeedRevision>,
            tonic::Status,
        >;
        ///
        async fn open_session(
            &self,
            request: tonic::Request<super::super::super::v1alpha1::SubjectRequest>,
        ) -> std::result::Result<
            tonic::Response<super::super::super::v1alpha1::Session>,
            tonic::Status,
        >;
        ///
        async fn get_session(
            &self,
            request: tonic::Request<super::super::super::v1alpha1::ObjectRequest>,
        ) -> std::result::Result<
            tonic::Response<super::super::super::v1alpha1::Session>,
            tonic::Status,
        >;
        ///
        async fn list_sessions(
            &self,
            request: tonic::Request<super::super::super::v1alpha1::ListRequest>,
        ) -> std::result::Result<
            tonic::Response<super::super::super::v1alpha1::ListSessionsResponse>,
            tonic::Status,
        >;
        ///
        async fn close_session(
            &self,
            request: tonic::Request<super::super::super::v1alpha1::ObjectRequest>,
        ) -> std::result::Result<
            tonic::Response<super::super::super::v1alpha1::Session>,
            tonic::Status,
        >;
        ///
        async fn record_observation(
            &self,
            request: tonic::Request<super::super::super::v1alpha1::ObservationInput>,
        ) -> std::result::Result<
            tonic::Response<super::super::super::v1alpha1::AcceptedObservation>,
            tonic::Status,
        >;
        ///
        async fn query(
            &self,
            request: tonic::Request<super::KernelQueryRequest>,
        ) -> std::result::Result<
            tonic::Response<super::super::super::v1alpha1::QueryResponse>,
            tonic::Status,
        >;
        ///
        async fn report_use(
            &self,
            request: tonic::Request<super::super::super::v1alpha1::ReportUseRequest>,
        ) -> std::result::Result<tonic::Response<()>, tonic::Status>;
        ///
        async fn get_memory(
            &self,
            request: tonic::Request<super::super::super::v1alpha1::ObjectRequest>,
        ) -> std::result::Result<
            tonic::Response<super::super::super::v1alpha1::Memory>,
            tonic::Status,
        >;
        ///
        async fn list_memories(
            &self,
            request: tonic::Request<super::super::super::v1alpha1::ListRequest>,
        ) -> std::result::Result<
            tonic::Response<super::super::super::v1alpha1::ListMemoriesResponse>,
            tonic::Status,
        >;
        ///
        async fn get_memory_revision(
            &self,
            request: tonic::Request<super::super::super::v1alpha1::ObjectRequest>,
        ) -> std::result::Result<
            tonic::Response<super::super::super::v1alpha1::Memory>,
            tonic::Status,
        >;
        ///
        async fn list_memory_revisions(
            &self,
            request: tonic::Request<super::super::super::v1alpha1::MemoryHistoryRequest>,
        ) -> std::result::Result<
            tonic::Response<super::super::super::v1alpha1::ListMemoriesResponse>,
            tonic::Status,
        >;
        ///
        async fn form_memory(
            &self,
            request: tonic::Request<super::super::super::v1alpha1::FormMemoryRequest>,
        ) -> std::result::Result<
            tonic::Response<super::super::super::v1alpha1::Memory>,
            tonic::Status,
        >;
        ///
        async fn revise_memory(
            &self,
            request: tonic::Request<super::super::super::v1alpha1::ReviseMemoryRequest>,
        ) -> std::result::Result<
            tonic::Response<super::super::super::v1alpha1::Memory>,
            tonic::Status,
        >;
        ///
        async fn suppress_memory(
            &self,
            request: tonic::Request<super::super::super::v1alpha1::MemoryMutationRequest>,
        ) -> std::result::Result<
            tonic::Response<super::super::super::v1alpha1::Memory>,
            tonic::Status,
        >;
        ///
        async fn restore_memory(
            &self,
            request: tonic::Request<super::super::super::v1alpha1::MemoryMutationRequest>,
        ) -> std::result::Result<
            tonic::Response<super::super::super::v1alpha1::Memory>,
            tonic::Status,
        >;
        ///
        async fn purge_memory(
            &self,
            request: tonic::Request<super::super::super::v1alpha1::MemoryMutationRequest>,
        ) -> std::result::Result<tonic::Response<()>, tonic::Status>;
        ///
        async fn get_artifact(
            &self,
            request: tonic::Request<super::super::super::v1alpha1::ObjectRequest>,
        ) -> std::result::Result<
            tonic::Response<super::super::super::v1alpha1::Artifact>,
            tonic::Status,
        >;
        ///
        async fn list_artifacts(
            &self,
            request: tonic::Request<super::super::super::v1alpha1::ListRequest>,
        ) -> std::result::Result<
            tonic::Response<super::super::super::v1alpha1::ListArtifactsResponse>,
            tonic::Status,
        >;
        ///
        async fn materialize_evidence(
            &self,
            request: tonic::Request<super::super::super::v1alpha1::MaterializeRequest>,
        ) -> std::result::Result<
            tonic::Response<super::super::super::v1alpha1::MaterializedEvidence>,
            tonic::Status,
        >;
    }
    ///
    #[derive(Debug)]
    pub struct AuthorityServiceServer<T> {
        inner: Arc<T>,
        accept_compression_encodings: EnabledCompressionEncodings,
        send_compression_encodings: EnabledCompressionEncodings,
        max_decoding_message_size: Option<usize>,
        max_encoding_message_size: Option<usize>,
    }
    impl<T> AuthorityServiceServer<T> {
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
    impl<T, B> tonic::codegen::Service<http::Request<B>> for AuthorityServiceServer<T>
    where
        T: AuthorityService,
        B: Body + std::marker::Send + 'static,
        B::Error: Into<StdError> + std::marker::Send + 'static,
    {
        type Response = http::Response<tonic::body::Body>;
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
                "/nous.wave.kernel.v1alpha1.AuthorityService/PutResource" => {
                    #[allow(non_camel_case_types)]
                    struct PutResourceSvc<T: AuthorityService>(pub Arc<T>);
                    impl<
                        T: AuthorityService,
                    > tonic::server::UnaryService<
                        super::super::super::v1alpha1::PutResourceRequest,
                    > for PutResourceSvc<T> {
                        type Response = super::super::super::v1alpha1::ResourceDescriptor;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<
                                super::super::super::v1alpha1::PutResourceRequest,
                            >,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as AuthorityService>::put_resource(&inner, request).await
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
                        let method = PutResourceSvc(inner);
                        let codec = tonic_prost::ProstCodec::default();
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
                "/nous.wave.kernel.v1alpha1.AuthorityService/GetResource" => {
                    #[allow(non_camel_case_types)]
                    struct GetResourceSvc<T: AuthorityService>(pub Arc<T>);
                    impl<
                        T: AuthorityService,
                    > tonic::server::UnaryService<
                        super::super::super::v1alpha1::ObjectRequest,
                    > for GetResourceSvc<T> {
                        type Response = super::super::super::v1alpha1::ResourceDescriptor;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<
                                super::super::super::v1alpha1::ObjectRequest,
                            >,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as AuthorityService>::get_resource(&inner, request).await
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
                        let method = GetResourceSvc(inner);
                        let codec = tonic_prost::ProstCodec::default();
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
                "/nous.wave.kernel.v1alpha1.AuthorityService/ListResources" => {
                    #[allow(non_camel_case_types)]
                    struct ListResourcesSvc<T: AuthorityService>(pub Arc<T>);
                    impl<
                        T: AuthorityService,
                    > tonic::server::UnaryService<
                        super::super::super::v1alpha1::ListRequest,
                    > for ListResourcesSvc<T> {
                        type Response = super::super::super::v1alpha1::ListResourcesResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<
                                super::super::super::v1alpha1::ListRequest,
                            >,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as AuthorityService>::list_resources(&inner, request)
                                    .await
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
                        let method = ListResourcesSvc(inner);
                        let codec = tonic_prost::ProstCodec::default();
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
                "/nous.wave.kernel.v1alpha1.AuthorityService/RemoveResource" => {
                    #[allow(non_camel_case_types)]
                    struct RemoveResourceSvc<T: AuthorityService>(pub Arc<T>);
                    impl<
                        T: AuthorityService,
                    > tonic::server::UnaryService<
                        super::super::super::v1alpha1::ObjectRequest,
                    > for RemoveResourceSvc<T> {
                        type Response = ();
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<
                                super::super::super::v1alpha1::ObjectRequest,
                            >,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as AuthorityService>::remove_resource(&inner, request)
                                    .await
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
                        let method = RemoveResourceSvc(inner);
                        let codec = tonic_prost::ProstCodec::default();
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
                "/nous.wave.kernel.v1alpha1.AuthorityService/CreateTag" => {
                    #[allow(non_camel_case_types)]
                    struct CreateTagSvc<T: AuthorityService>(pub Arc<T>);
                    impl<
                        T: AuthorityService,
                    > tonic::server::UnaryService<
                        super::super::super::v1alpha1::CreateTagRequest,
                    > for CreateTagSvc<T> {
                        type Response = super::super::super::v1alpha1::Tag;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<
                                super::super::super::v1alpha1::CreateTagRequest,
                            >,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as AuthorityService>::create_tag(&inner, request).await
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
                        let method = CreateTagSvc(inner);
                        let codec = tonic_prost::ProstCodec::default();
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
                "/nous.wave.kernel.v1alpha1.AuthorityService/GetTag" => {
                    #[allow(non_camel_case_types)]
                    struct GetTagSvc<T: AuthorityService>(pub Arc<T>);
                    impl<
                        T: AuthorityService,
                    > tonic::server::UnaryService<
                        super::super::super::v1alpha1::ObjectRequest,
                    > for GetTagSvc<T> {
                        type Response = super::super::super::v1alpha1::Tag;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<
                                super::super::super::v1alpha1::ObjectRequest,
                            >,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as AuthorityService>::get_tag(&inner, request).await
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
                        let method = GetTagSvc(inner);
                        let codec = tonic_prost::ProstCodec::default();
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
                "/nous.wave.kernel.v1alpha1.AuthorityService/ListTags" => {
                    #[allow(non_camel_case_types)]
                    struct ListTagsSvc<T: AuthorityService>(pub Arc<T>);
                    impl<
                        T: AuthorityService,
                    > tonic::server::UnaryService<
                        super::super::super::v1alpha1::ListRequest,
                    > for ListTagsSvc<T> {
                        type Response = super::super::super::v1alpha1::ListTagsResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<
                                super::super::super::v1alpha1::ListRequest,
                            >,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as AuthorityService>::list_tags(&inner, request).await
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
                        let method = ListTagsSvc(inner);
                        let codec = tonic_prost::ProstCodec::default();
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
                "/nous.wave.kernel.v1alpha1.AuthorityService/CreateAnchor" => {
                    #[allow(non_camel_case_types)]
                    struct CreateAnchorSvc<T: AuthorityService>(pub Arc<T>);
                    impl<
                        T: AuthorityService,
                    > tonic::server::UnaryService<
                        super::super::super::v1alpha1::CreateAnchorRequest,
                    > for CreateAnchorSvc<T> {
                        type Response = super::super::super::v1alpha1::Anchor;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<
                                super::super::super::v1alpha1::CreateAnchorRequest,
                            >,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as AuthorityService>::create_anchor(&inner, request)
                                    .await
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
                        let method = CreateAnchorSvc(inner);
                        let codec = tonic_prost::ProstCodec::default();
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
                "/nous.wave.kernel.v1alpha1.AuthorityService/GetAnchor" => {
                    #[allow(non_camel_case_types)]
                    struct GetAnchorSvc<T: AuthorityService>(pub Arc<T>);
                    impl<
                        T: AuthorityService,
                    > tonic::server::UnaryService<
                        super::super::super::v1alpha1::ObjectRequest,
                    > for GetAnchorSvc<T> {
                        type Response = super::super::super::v1alpha1::Anchor;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<
                                super::super::super::v1alpha1::ObjectRequest,
                            >,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as AuthorityService>::get_anchor(&inner, request).await
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
                        let method = GetAnchorSvc(inner);
                        let codec = tonic_prost::ProstCodec::default();
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
                "/nous.wave.kernel.v1alpha1.AuthorityService/ListAnchors" => {
                    #[allow(non_camel_case_types)]
                    struct ListAnchorsSvc<T: AuthorityService>(pub Arc<T>);
                    impl<
                        T: AuthorityService,
                    > tonic::server::UnaryService<
                        super::super::super::v1alpha1::ListRequest,
                    > for ListAnchorsSvc<T> {
                        type Response = super::super::super::v1alpha1::ListAnchorsResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<
                                super::super::super::v1alpha1::ListRequest,
                            >,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as AuthorityService>::list_anchors(&inner, request).await
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
                        let method = ListAnchorsSvc(inner);
                        let codec = tonic_prost::ProstCodec::default();
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
                "/nous.wave.kernel.v1alpha1.AuthorityService/CreateAssociation" => {
                    #[allow(non_camel_case_types)]
                    struct CreateAssociationSvc<T: AuthorityService>(pub Arc<T>);
                    impl<
                        T: AuthorityService,
                    > tonic::server::UnaryService<
                        super::super::super::v1alpha1::CreateAssociationRequest,
                    > for CreateAssociationSvc<T> {
                        type Response = super::super::super::v1alpha1::Association;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<
                                super::super::super::v1alpha1::CreateAssociationRequest,
                            >,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as AuthorityService>::create_association(&inner, request)
                                    .await
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
                        let method = CreateAssociationSvc(inner);
                        let codec = tonic_prost::ProstCodec::default();
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
                "/nous.wave.kernel.v1alpha1.AuthorityService/GetNeighborhood" => {
                    #[allow(non_camel_case_types)]
                    struct GetNeighborhoodSvc<T: AuthorityService>(pub Arc<T>);
                    impl<
                        T: AuthorityService,
                    > tonic::server::UnaryService<
                        super::super::super::v1alpha1::NeighborhoodRequest,
                    > for GetNeighborhoodSvc<T> {
                        type Response = super::super::super::v1alpha1::NeighborhoodResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<
                                super::super::super::v1alpha1::NeighborhoodRequest,
                            >,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as AuthorityService>::get_neighborhood(&inner, request)
                                    .await
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
                        let method = GetNeighborhoodSvc(inner);
                        let codec = tonic_prost::ProstCodec::default();
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
                "/nous.wave.kernel.v1alpha1.AuthorityService/RebindEntity" => {
                    #[allow(non_camel_case_types)]
                    struct RebindEntitySvc<T: AuthorityService>(pub Arc<T>);
                    impl<
                        T: AuthorityService,
                    > tonic::server::UnaryService<
                        super::super::super::v1alpha1::RebindEntityRequest,
                    > for RebindEntitySvc<T> {
                        type Response = ();
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<
                                super::super::super::v1alpha1::RebindEntityRequest,
                            >,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as AuthorityService>::rebind_entity(&inner, request)
                                    .await
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
                        let method = RebindEntitySvc(inner);
                        let codec = tonic_prost::ProstCodec::default();
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
                "/nous.wave.kernel.v1alpha1.AuthorityService/GetStatus" => {
                    #[allow(non_camel_case_types)]
                    struct GetStatusSvc<T: AuthorityService>(pub Arc<T>);
                    impl<T: AuthorityService> tonic::server::UnaryService<()>
                    for GetStatusSvc<T> {
                        type Response = super::super::super::v1alpha1::SystemStatus;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(&mut self, request: tonic::Request<()>) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as AuthorityService>::get_status(&inner, request).await
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
                        let method = GetStatusSvc(inner);
                        let codec = tonic_prost::ProstCodec::default();
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
                "/nous.wave.kernel.v1alpha1.AuthorityService/GetProjectionStatus" => {
                    #[allow(non_camel_case_types)]
                    struct GetProjectionStatusSvc<T: AuthorityService>(pub Arc<T>);
                    impl<
                        T: AuthorityService,
                    > tonic::server::UnaryService<
                        super::super::super::v1alpha1::SubjectRequest,
                    > for GetProjectionStatusSvc<T> {
                        type Response = super::super::super::v1alpha1::ProjectionStatus;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<
                                super::super::super::v1alpha1::SubjectRequest,
                            >,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as AuthorityService>::get_projection_status(
                                        &inner,
                                        request,
                                    )
                                    .await
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
                        let method = GetProjectionStatusSvc(inner);
                        let codec = tonic_prost::ProstCodec::default();
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
                "/nous.wave.kernel.v1alpha1.AuthorityService/ConsolidateMemory" => {
                    #[allow(non_camel_case_types)]
                    struct ConsolidateMemorySvc<T: AuthorityService>(pub Arc<T>);
                    impl<
                        T: AuthorityService,
                    > tonic::server::UnaryService<
                        super::super::super::v1alpha1::ConsolidateMemoryRequest,
                    > for ConsolidateMemorySvc<T> {
                        type Response = super::super::super::v1alpha1::Memory;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<
                                super::super::super::v1alpha1::ConsolidateMemoryRequest,
                            >,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as AuthorityService>::consolidate_memory(&inner, request)
                                    .await
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
                        let method = ConsolidateMemorySvc(inner);
                        let codec = tonic_prost::ProstCodec::default();
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
                "/nous.wave.kernel.v1alpha1.AuthorityService/GetOccurrence" => {
                    #[allow(non_camel_case_types)]
                    struct GetOccurrenceSvc<T: AuthorityService>(pub Arc<T>);
                    impl<
                        T: AuthorityService,
                    > tonic::server::UnaryService<
                        super::super::super::v1alpha1::ObjectRequest,
                    > for GetOccurrenceSvc<T> {
                        type Response = super::super::super::v1alpha1::Occurrence;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<
                                super::super::super::v1alpha1::ObjectRequest,
                            >,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as AuthorityService>::get_occurrence(&inner, request)
                                    .await
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
                        let method = GetOccurrenceSvc(inner);
                        let codec = tonic_prost::ProstCodec::default();
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
                "/nous.wave.kernel.v1alpha1.AuthorityService/GetSourceRegion" => {
                    #[allow(non_camel_case_types)]
                    struct GetSourceRegionSvc<T: AuthorityService>(pub Arc<T>);
                    impl<
                        T: AuthorityService,
                    > tonic::server::UnaryService<
                        super::super::super::v1alpha1::ObjectRequest,
                    > for GetSourceRegionSvc<T> {
                        type Response = super::super::super::v1alpha1::SourceRegion;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<
                                super::super::super::v1alpha1::ObjectRequest,
                            >,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as AuthorityService>::get_source_region(&inner, request)
                                    .await
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
                        let method = GetSourceRegionSvc(inner);
                        let codec = tonic_prost::ProstCodec::default();
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
                "/nous.wave.kernel.v1alpha1.AuthorityService/GetDerivedRepresentation" => {
                    #[allow(non_camel_case_types)]
                    struct GetDerivedRepresentationSvc<T: AuthorityService>(pub Arc<T>);
                    impl<
                        T: AuthorityService,
                    > tonic::server::UnaryService<
                        super::super::super::v1alpha1::ObjectRequest,
                    > for GetDerivedRepresentationSvc<T> {
                        type Response = super::super::super::v1alpha1::DerivedRepresentation;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<
                                super::super::super::v1alpha1::ObjectRequest,
                            >,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as AuthorityService>::get_derived_representation(
                                        &inner,
                                        request,
                                    )
                                    .await
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
                        let method = GetDerivedRepresentationSvc(inner);
                        let codec = tonic_prost::ProstCodec::default();
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
                "/nous.wave.kernel.v1alpha1.AuthorityService/BindIdentity" => {
                    #[allow(non_camel_case_types)]
                    struct BindIdentitySvc<T: AuthorityService>(pub Arc<T>);
                    impl<
                        T: AuthorityService,
                    > tonic::server::UnaryService<
                        super::super::super::v1alpha1::BindIdentityRequest,
                    > for BindIdentitySvc<T> {
                        type Response = super::super::super::v1alpha1::IdentityBinding;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<
                                super::super::super::v1alpha1::BindIdentityRequest,
                            >,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as AuthorityService>::bind_identity(&inner, request)
                                    .await
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
                        let method = BindIdentitySvc(inner);
                        let codec = tonic_prost::ProstCodec::default();
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
                "/nous.wave.kernel.v1alpha1.AuthorityService/ResolveIdentity" => {
                    #[allow(non_camel_case_types)]
                    struct ResolveIdentitySvc<T: AuthorityService>(pub Arc<T>);
                    impl<
                        T: AuthorityService,
                    > tonic::server::UnaryService<
                        super::super::super::v1alpha1::ResolveIdentityRequest,
                    > for ResolveIdentitySvc<T> {
                        type Response = super::super::super::v1alpha1::ResolveIdentityResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<
                                super::super::super::v1alpha1::ResolveIdentityRequest,
                            >,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as AuthorityService>::resolve_identity(&inner, request)
                                    .await
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
                        let method = ResolveIdentitySvc(inner);
                        let codec = tonic_prost::ProstCodec::default();
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
                "/nous.wave.kernel.v1alpha1.AuthorityService/BuildContributionBatch" => {
                    #[allow(non_camel_case_types)]
                    struct BuildContributionBatchSvc<T: AuthorityService>(pub Arc<T>);
                    impl<
                        T: AuthorityService,
                    > tonic::server::UnaryService<
                        super::super::super::v1alpha1::ProjectionRequest,
                    > for BuildContributionBatchSvc<T> {
                        type Response = super::super::super::v1alpha1::Projection;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<
                                super::super::super::v1alpha1::ProjectionRequest,
                            >,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as AuthorityService>::build_contribution_batch(
                                        &inner,
                                        request,
                                    )
                                    .await
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
                        let method = BuildContributionBatchSvc(inner);
                        let codec = tonic_prost::ProstCodec::default();
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
                "/nous.wave.kernel.v1alpha1.AuthorityService/CreateSubject" => {
                    #[allow(non_camel_case_types)]
                    struct CreateSubjectSvc<T: AuthorityService>(pub Arc<T>);
                    impl<
                        T: AuthorityService,
                    > tonic::server::UnaryService<
                        super::super::super::v1alpha1::CreateSubjectRequest,
                    > for CreateSubjectSvc<T> {
                        type Response = super::super::super::v1alpha1::Subject;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<
                                super::super::super::v1alpha1::CreateSubjectRequest,
                            >,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as AuthorityService>::create_subject(&inner, request)
                                    .await
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
                        let method = CreateSubjectSvc(inner);
                        let codec = tonic_prost::ProstCodec::default();
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
                "/nous.wave.kernel.v1alpha1.AuthorityService/GetSubject" => {
                    #[allow(non_camel_case_types)]
                    struct GetSubjectSvc<T: AuthorityService>(pub Arc<T>);
                    impl<
                        T: AuthorityService,
                    > tonic::server::UnaryService<
                        super::super::super::v1alpha1::SubjectRequest,
                    > for GetSubjectSvc<T> {
                        type Response = super::super::super::v1alpha1::Subject;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<
                                super::super::super::v1alpha1::SubjectRequest,
                            >,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as AuthorityService>::get_subject(&inner, request).await
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
                        let method = GetSubjectSvc(inner);
                        let codec = tonic_prost::ProstCodec::default();
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
                "/nous.wave.kernel.v1alpha1.AuthorityService/ListSubjects" => {
                    #[allow(non_camel_case_types)]
                    struct ListSubjectsSvc<T: AuthorityService>(pub Arc<T>);
                    impl<
                        T: AuthorityService,
                    > tonic::server::UnaryService<
                        super::super::super::v1alpha1::ListRequest,
                    > for ListSubjectsSvc<T> {
                        type Response = super::super::super::v1alpha1::ListSubjectsResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<
                                super::super::super::v1alpha1::ListRequest,
                            >,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as AuthorityService>::list_subjects(&inner, request)
                                    .await
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
                        let method = ListSubjectsSvc(inner);
                        let codec = tonic_prost::ProstCodec::default();
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
                "/nous.wave.kernel.v1alpha1.AuthorityService/GetCharacterSeed" => {
                    #[allow(non_camel_case_types)]
                    struct GetCharacterSeedSvc<T: AuthorityService>(pub Arc<T>);
                    impl<
                        T: AuthorityService,
                    > tonic::server::UnaryService<
                        super::super::super::v1alpha1::SubjectRequest,
                    > for GetCharacterSeedSvc<T> {
                        type Response = super::super::super::v1alpha1::SeedRevision;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<
                                super::super::super::v1alpha1::SubjectRequest,
                            >,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as AuthorityService>::get_character_seed(&inner, request)
                                    .await
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
                        let method = GetCharacterSeedSvc(inner);
                        let codec = tonic_prost::ProstCodec::default();
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
                "/nous.wave.kernel.v1alpha1.AuthorityService/ReviseCharacterSeed" => {
                    #[allow(non_camel_case_types)]
                    struct ReviseCharacterSeedSvc<T: AuthorityService>(pub Arc<T>);
                    impl<
                        T: AuthorityService,
                    > tonic::server::UnaryService<
                        super::super::super::v1alpha1::ReviseCharacterSeedRequest,
                    > for ReviseCharacterSeedSvc<T> {
                        type Response = super::super::super::v1alpha1::SeedRevision;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<
                                super::super::super::v1alpha1::ReviseCharacterSeedRequest,
                            >,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as AuthorityService>::revise_character_seed(
                                        &inner,
                                        request,
                                    )
                                    .await
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
                        let method = ReviseCharacterSeedSvc(inner);
                        let codec = tonic_prost::ProstCodec::default();
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
                "/nous.wave.kernel.v1alpha1.AuthorityService/OpenSession" => {
                    #[allow(non_camel_case_types)]
                    struct OpenSessionSvc<T: AuthorityService>(pub Arc<T>);
                    impl<
                        T: AuthorityService,
                    > tonic::server::UnaryService<
                        super::super::super::v1alpha1::SubjectRequest,
                    > for OpenSessionSvc<T> {
                        type Response = super::super::super::v1alpha1::Session;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<
                                super::super::super::v1alpha1::SubjectRequest,
                            >,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as AuthorityService>::open_session(&inner, request).await
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
                        let method = OpenSessionSvc(inner);
                        let codec = tonic_prost::ProstCodec::default();
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
                "/nous.wave.kernel.v1alpha1.AuthorityService/GetSession" => {
                    #[allow(non_camel_case_types)]
                    struct GetSessionSvc<T: AuthorityService>(pub Arc<T>);
                    impl<
                        T: AuthorityService,
                    > tonic::server::UnaryService<
                        super::super::super::v1alpha1::ObjectRequest,
                    > for GetSessionSvc<T> {
                        type Response = super::super::super::v1alpha1::Session;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<
                                super::super::super::v1alpha1::ObjectRequest,
                            >,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as AuthorityService>::get_session(&inner, request).await
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
                        let method = GetSessionSvc(inner);
                        let codec = tonic_prost::ProstCodec::default();
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
                "/nous.wave.kernel.v1alpha1.AuthorityService/ListSessions" => {
                    #[allow(non_camel_case_types)]
                    struct ListSessionsSvc<T: AuthorityService>(pub Arc<T>);
                    impl<
                        T: AuthorityService,
                    > tonic::server::UnaryService<
                        super::super::super::v1alpha1::ListRequest,
                    > for ListSessionsSvc<T> {
                        type Response = super::super::super::v1alpha1::ListSessionsResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<
                                super::super::super::v1alpha1::ListRequest,
                            >,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as AuthorityService>::list_sessions(&inner, request)
                                    .await
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
                        let method = ListSessionsSvc(inner);
                        let codec = tonic_prost::ProstCodec::default();
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
                "/nous.wave.kernel.v1alpha1.AuthorityService/CloseSession" => {
                    #[allow(non_camel_case_types)]
                    struct CloseSessionSvc<T: AuthorityService>(pub Arc<T>);
                    impl<
                        T: AuthorityService,
                    > tonic::server::UnaryService<
                        super::super::super::v1alpha1::ObjectRequest,
                    > for CloseSessionSvc<T> {
                        type Response = super::super::super::v1alpha1::Session;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<
                                super::super::super::v1alpha1::ObjectRequest,
                            >,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as AuthorityService>::close_session(&inner, request)
                                    .await
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
                        let method = CloseSessionSvc(inner);
                        let codec = tonic_prost::ProstCodec::default();
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
                "/nous.wave.kernel.v1alpha1.AuthorityService/RecordObservation" => {
                    #[allow(non_camel_case_types)]
                    struct RecordObservationSvc<T: AuthorityService>(pub Arc<T>);
                    impl<
                        T: AuthorityService,
                    > tonic::server::UnaryService<
                        super::super::super::v1alpha1::ObservationInput,
                    > for RecordObservationSvc<T> {
                        type Response = super::super::super::v1alpha1::AcceptedObservation;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<
                                super::super::super::v1alpha1::ObservationInput,
                            >,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as AuthorityService>::record_observation(&inner, request)
                                    .await
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
                        let method = RecordObservationSvc(inner);
                        let codec = tonic_prost::ProstCodec::default();
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
                "/nous.wave.kernel.v1alpha1.AuthorityService/Query" => {
                    #[allow(non_camel_case_types)]
                    struct QuerySvc<T: AuthorityService>(pub Arc<T>);
                    impl<
                        T: AuthorityService,
                    > tonic::server::UnaryService<super::KernelQueryRequest>
                    for QuerySvc<T> {
                        type Response = super::super::super::v1alpha1::QueryResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::KernelQueryRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as AuthorityService>::query(&inner, request).await
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
                        let method = QuerySvc(inner);
                        let codec = tonic_prost::ProstCodec::default();
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
                "/nous.wave.kernel.v1alpha1.AuthorityService/ReportUse" => {
                    #[allow(non_camel_case_types)]
                    struct ReportUseSvc<T: AuthorityService>(pub Arc<T>);
                    impl<
                        T: AuthorityService,
                    > tonic::server::UnaryService<
                        super::super::super::v1alpha1::ReportUseRequest,
                    > for ReportUseSvc<T> {
                        type Response = ();
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<
                                super::super::super::v1alpha1::ReportUseRequest,
                            >,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as AuthorityService>::report_use(&inner, request).await
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
                        let method = ReportUseSvc(inner);
                        let codec = tonic_prost::ProstCodec::default();
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
                "/nous.wave.kernel.v1alpha1.AuthorityService/GetMemory" => {
                    #[allow(non_camel_case_types)]
                    struct GetMemorySvc<T: AuthorityService>(pub Arc<T>);
                    impl<
                        T: AuthorityService,
                    > tonic::server::UnaryService<
                        super::super::super::v1alpha1::ObjectRequest,
                    > for GetMemorySvc<T> {
                        type Response = super::super::super::v1alpha1::Memory;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<
                                super::super::super::v1alpha1::ObjectRequest,
                            >,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as AuthorityService>::get_memory(&inner, request).await
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
                        let method = GetMemorySvc(inner);
                        let codec = tonic_prost::ProstCodec::default();
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
                "/nous.wave.kernel.v1alpha1.AuthorityService/ListMemories" => {
                    #[allow(non_camel_case_types)]
                    struct ListMemoriesSvc<T: AuthorityService>(pub Arc<T>);
                    impl<
                        T: AuthorityService,
                    > tonic::server::UnaryService<
                        super::super::super::v1alpha1::ListRequest,
                    > for ListMemoriesSvc<T> {
                        type Response = super::super::super::v1alpha1::ListMemoriesResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<
                                super::super::super::v1alpha1::ListRequest,
                            >,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as AuthorityService>::list_memories(&inner, request)
                                    .await
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
                        let method = ListMemoriesSvc(inner);
                        let codec = tonic_prost::ProstCodec::default();
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
                "/nous.wave.kernel.v1alpha1.AuthorityService/GetMemoryRevision" => {
                    #[allow(non_camel_case_types)]
                    struct GetMemoryRevisionSvc<T: AuthorityService>(pub Arc<T>);
                    impl<
                        T: AuthorityService,
                    > tonic::server::UnaryService<
                        super::super::super::v1alpha1::ObjectRequest,
                    > for GetMemoryRevisionSvc<T> {
                        type Response = super::super::super::v1alpha1::Memory;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<
                                super::super::super::v1alpha1::ObjectRequest,
                            >,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as AuthorityService>::get_memory_revision(
                                        &inner,
                                        request,
                                    )
                                    .await
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
                        let method = GetMemoryRevisionSvc(inner);
                        let codec = tonic_prost::ProstCodec::default();
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
                "/nous.wave.kernel.v1alpha1.AuthorityService/ListMemoryRevisions" => {
                    #[allow(non_camel_case_types)]
                    struct ListMemoryRevisionsSvc<T: AuthorityService>(pub Arc<T>);
                    impl<
                        T: AuthorityService,
                    > tonic::server::UnaryService<
                        super::super::super::v1alpha1::MemoryHistoryRequest,
                    > for ListMemoryRevisionsSvc<T> {
                        type Response = super::super::super::v1alpha1::ListMemoriesResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<
                                super::super::super::v1alpha1::MemoryHistoryRequest,
                            >,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as AuthorityService>::list_memory_revisions(
                                        &inner,
                                        request,
                                    )
                                    .await
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
                        let method = ListMemoryRevisionsSvc(inner);
                        let codec = tonic_prost::ProstCodec::default();
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
                "/nous.wave.kernel.v1alpha1.AuthorityService/FormMemory" => {
                    #[allow(non_camel_case_types)]
                    struct FormMemorySvc<T: AuthorityService>(pub Arc<T>);
                    impl<
                        T: AuthorityService,
                    > tonic::server::UnaryService<
                        super::super::super::v1alpha1::FormMemoryRequest,
                    > for FormMemorySvc<T> {
                        type Response = super::super::super::v1alpha1::Memory;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<
                                super::super::super::v1alpha1::FormMemoryRequest,
                            >,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as AuthorityService>::form_memory(&inner, request).await
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
                        let method = FormMemorySvc(inner);
                        let codec = tonic_prost::ProstCodec::default();
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
                "/nous.wave.kernel.v1alpha1.AuthorityService/ReviseMemory" => {
                    #[allow(non_camel_case_types)]
                    struct ReviseMemorySvc<T: AuthorityService>(pub Arc<T>);
                    impl<
                        T: AuthorityService,
                    > tonic::server::UnaryService<
                        super::super::super::v1alpha1::ReviseMemoryRequest,
                    > for ReviseMemorySvc<T> {
                        type Response = super::super::super::v1alpha1::Memory;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<
                                super::super::super::v1alpha1::ReviseMemoryRequest,
                            >,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as AuthorityService>::revise_memory(&inner, request)
                                    .await
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
                        let method = ReviseMemorySvc(inner);
                        let codec = tonic_prost::ProstCodec::default();
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
                "/nous.wave.kernel.v1alpha1.AuthorityService/SuppressMemory" => {
                    #[allow(non_camel_case_types)]
                    struct SuppressMemorySvc<T: AuthorityService>(pub Arc<T>);
                    impl<
                        T: AuthorityService,
                    > tonic::server::UnaryService<
                        super::super::super::v1alpha1::MemoryMutationRequest,
                    > for SuppressMemorySvc<T> {
                        type Response = super::super::super::v1alpha1::Memory;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<
                                super::super::super::v1alpha1::MemoryMutationRequest,
                            >,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as AuthorityService>::suppress_memory(&inner, request)
                                    .await
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
                        let method = SuppressMemorySvc(inner);
                        let codec = tonic_prost::ProstCodec::default();
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
                "/nous.wave.kernel.v1alpha1.AuthorityService/RestoreMemory" => {
                    #[allow(non_camel_case_types)]
                    struct RestoreMemorySvc<T: AuthorityService>(pub Arc<T>);
                    impl<
                        T: AuthorityService,
                    > tonic::server::UnaryService<
                        super::super::super::v1alpha1::MemoryMutationRequest,
                    > for RestoreMemorySvc<T> {
                        type Response = super::super::super::v1alpha1::Memory;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<
                                super::super::super::v1alpha1::MemoryMutationRequest,
                            >,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as AuthorityService>::restore_memory(&inner, request)
                                    .await
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
                        let method = RestoreMemorySvc(inner);
                        let codec = tonic_prost::ProstCodec::default();
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
                "/nous.wave.kernel.v1alpha1.AuthorityService/PurgeMemory" => {
                    #[allow(non_camel_case_types)]
                    struct PurgeMemorySvc<T: AuthorityService>(pub Arc<T>);
                    impl<
                        T: AuthorityService,
                    > tonic::server::UnaryService<
                        super::super::super::v1alpha1::MemoryMutationRequest,
                    > for PurgeMemorySvc<T> {
                        type Response = ();
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<
                                super::super::super::v1alpha1::MemoryMutationRequest,
                            >,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as AuthorityService>::purge_memory(&inner, request).await
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
                        let method = PurgeMemorySvc(inner);
                        let codec = tonic_prost::ProstCodec::default();
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
                "/nous.wave.kernel.v1alpha1.AuthorityService/GetArtifact" => {
                    #[allow(non_camel_case_types)]
                    struct GetArtifactSvc<T: AuthorityService>(pub Arc<T>);
                    impl<
                        T: AuthorityService,
                    > tonic::server::UnaryService<
                        super::super::super::v1alpha1::ObjectRequest,
                    > for GetArtifactSvc<T> {
                        type Response = super::super::super::v1alpha1::Artifact;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<
                                super::super::super::v1alpha1::ObjectRequest,
                            >,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as AuthorityService>::get_artifact(&inner, request).await
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
                        let method = GetArtifactSvc(inner);
                        let codec = tonic_prost::ProstCodec::default();
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
                "/nous.wave.kernel.v1alpha1.AuthorityService/ListArtifacts" => {
                    #[allow(non_camel_case_types)]
                    struct ListArtifactsSvc<T: AuthorityService>(pub Arc<T>);
                    impl<
                        T: AuthorityService,
                    > tonic::server::UnaryService<
                        super::super::super::v1alpha1::ListRequest,
                    > for ListArtifactsSvc<T> {
                        type Response = super::super::super::v1alpha1::ListArtifactsResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<
                                super::super::super::v1alpha1::ListRequest,
                            >,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as AuthorityService>::list_artifacts(&inner, request)
                                    .await
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
                        let method = ListArtifactsSvc(inner);
                        let codec = tonic_prost::ProstCodec::default();
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
                "/nous.wave.kernel.v1alpha1.AuthorityService/MaterializeEvidence" => {
                    #[allow(non_camel_case_types)]
                    struct MaterializeEvidenceSvc<T: AuthorityService>(pub Arc<T>);
                    impl<
                        T: AuthorityService,
                    > tonic::server::UnaryService<
                        super::super::super::v1alpha1::MaterializeRequest,
                    > for MaterializeEvidenceSvc<T> {
                        type Response = super::super::super::v1alpha1::MaterializedEvidence;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<
                                super::super::super::v1alpha1::MaterializeRequest,
                            >,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as AuthorityService>::materialize_evidence(
                                        &inner,
                                        request,
                                    )
                                    .await
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
                        let method = MaterializeEvidenceSvc(inner);
                        let codec = tonic_prost::ProstCodec::default();
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
                        let mut response = http::Response::new(
                            tonic::body::Body::default(),
                        );
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
    impl<T> Clone for AuthorityServiceServer<T> {
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
    pub const SERVICE_NAME: &str = "nous.wave.kernel.v1alpha1.AuthorityService";
    impl<T> tonic::server::NamedService for AuthorityServiceServer<T> {
        const NAME: &'static str = SERVICE_NAME;
    }
}
