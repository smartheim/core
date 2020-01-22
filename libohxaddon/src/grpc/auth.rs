#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RequestTokenMsg {
    /// A valid refresh token must be provided
    #[prost(string, tag = "1")]
    pub refresh_token: std::string::String,
    /// Provide all required scopes
    #[prost(string, repeated, tag = "2")]
    pub scope: ::std::vec::Vec<std::string::String>,
}
/// The response message containing the token
/// There is no user visible error string, because this RPC is purely for M2M communication where scopes are carefully selected.
/// The only reason for this to fail is, if the refresh_token has been revoked which requires manual user intervention anyway.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct AccessTokenReply {
    #[prost(bool, tag = "1")]
    pub success: bool,
    #[prost(string, tag = "2")]
    pub access_token: std::string::String,
}
#[doc = r" Generated server implementations."]
pub mod request_token_client {
    #![allow(unused_variables, dead_code, missing_docs)]
    use tonic::codegen::*;
    #[doc = " RequestToken is the only RPC that does not need to pass the JWT auth validation"]
    #[doc = " NEVER send this RPC via an unencrypted transport channel. Preferably check peer certificates or use certificate pinning."]
    #[doc = " The refresh_token will send in plain text otherwise!"]
    pub struct RequestTokenClient<T> {
        inner: tonic::client::Grpc<T>,
    }
    impl RequestTokenClient<tonic::transport::Channel> {
        #[doc = r" Attempt to create a new client by connecting to a given endpoint."]
        pub async fn connect<D>(dst: D) -> Result<Self, tonic::transport::Error>
        where
            D: std::convert::TryInto<tonic::transport::Endpoint>,
            D::Error: Into<StdError>,
        {
            let conn = tonic::transport::Endpoint::new(dst)?.connect().await?;
            Ok(Self::new(conn))
        }
    }
    impl<T> RequestTokenClient<T>
    where
        T: tonic::client::GrpcService<tonic::body::BoxBody>,
        T::ResponseBody: Body + HttpBody + Send + 'static,
        T::Error: Into<StdError>,
        <T::ResponseBody as HttpBody>::Error: Into<StdError> + Send,
    {
        pub fn new(inner: T) -> Self {
            let inner = tonic::client::Grpc::new(inner);
            Self { inner }
        }
        pub fn with_interceptor(inner: T, interceptor: impl Into<tonic::Interceptor>) -> Self {
            let inner = tonic::client::Grpc::with_interceptor(inner, interceptor);
            Self { inner }
        }
        #[doc = " Request an access token by providing the refresh token"]
        pub async fn request_token(
            &mut self,
            request: impl tonic::IntoRequest<super::RequestTokenMsg>,
        ) -> Result<tonic::Response<super::AccessTokenReply>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(tonic::Code::Unknown, format!("Service was not ready: {}", e.into()))
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/auth.RequestToken/RequestToken");
            self.inner.unary(request.into_request(), path, codec).await
        }
    }
    impl<T: Clone> Clone for RequestTokenClient<T> {
        fn clone(&self) -> Self {
            Self {
                inner: self.inner.clone(),
            }
        }
    }
}
#[doc = r" Generated server implementations."]
pub mod request_token_server {
    #![allow(unused_variables, dead_code, missing_docs)]
    use tonic::codegen::*;
    #[doc = "Generated trait containing gRPC methods that should be implemented for use with RequestTokenServer."]
    #[async_trait]
    pub trait RequestToken: Send + Sync + 'static {
        #[doc = " Request an access token by providing the refresh token"]
        async fn request_token(
            &self,
            request: tonic::Request<super::RequestTokenMsg>,
        ) -> Result<tonic::Response<super::AccessTokenReply>, tonic::Status>;
    }
    #[doc = " RequestToken is the only RPC that does not need to pass the JWT auth validation"]
    #[doc = " NEVER send this RPC via an unencrypted transport channel. Preferably check peer certificates or use certificate pinning."]
    #[doc = " The refresh_token will send in plain text otherwise!"]
    #[derive(Debug)]
    #[doc(hidden)]
    pub struct RequestTokenServer<T: RequestToken> {
        inner: _Inner<T>,
    }
    struct _Inner<T>(Arc<T>, Option<tonic::Interceptor>);
    impl<T: RequestToken> RequestTokenServer<T> {
        pub fn new(inner: T) -> Self {
            let inner = Arc::new(inner);
            let inner = _Inner(inner, None);
            Self { inner }
        }
        pub fn with_interceptor(inner: T, interceptor: impl Into<tonic::Interceptor>) -> Self {
            let inner = Arc::new(inner);
            let inner = _Inner(inner, Some(interceptor.into()));
            Self { inner }
        }
    }
    impl<T: RequestToken> Service<http::Request<HyperBody>> for RequestTokenServer<T> {
        type Response = http::Response<tonic::body::BoxBody>;
        type Error = Never;
        type Future = BoxFuture<Self::Response, Self::Error>;
        fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
            Poll::Ready(Ok(()))
        }
        fn call(&mut self, req: http::Request<HyperBody>) -> Self::Future {
            let inner = self.inner.clone();
            match req.uri().path() {
                "/auth.RequestToken/RequestToken" => {
                    struct RequestTokenSvc<T: RequestToken>(pub Arc<T>);
                    impl<T: RequestToken> tonic::server::UnaryService<super::RequestTokenMsg> for RequestTokenSvc<T> {
                        type Response = super::AccessTokenReply;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(&mut self, request: tonic::Request<super::RequestTokenMsg>) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { inner.request_token(request).await };
                            Box::pin(fut)
                        }
                    }
                    let inner = self.inner.clone();
                    let fut = async move {
                        let interceptor = inner.1.clone();
                        let inner = inner.0;
                        let method = RequestTokenSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = if let Some(interceptor) = interceptor {
                            tonic::server::Grpc::with_interceptor(codec, interceptor)
                        } else {
                            tonic::server::Grpc::new(codec)
                        };
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                },
                _ => Box::pin(async move {
                    Ok(http::Response::builder()
                        .status(200)
                        .header("grpc-status", "12")
                        .body(tonic::body::BoxBody::empty())
                        .unwrap())
                }),
            }
        }
    }
    impl<T: RequestToken> Clone for RequestTokenServer<T> {
        fn clone(&self) -> Self {
            let inner = self.inner.clone();
            Self { inner }
        }
    }
    impl<T: RequestToken> Clone for _Inner<T> {
        fn clone(&self) -> Self {
            Self(self.0.clone(), self.1.clone())
        }
    }
    impl<T: std::fmt::Debug> std::fmt::Debug for _Inner<T> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{:?}", self.0)
        }
    }
    impl<T: RequestToken> tonic::transport::NamedService for RequestTokenServer<T> {
        const NAME: &'static str = "auth.RequestToken";
    }
}
