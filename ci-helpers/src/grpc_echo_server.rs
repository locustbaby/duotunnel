use std::collections::HashMap;
use std::net::SocketAddr;
use std::pin::Pin;
use tokio_stream::{wrappers::ReceiverStream, StreamExt};
use tonic::{transport::Server, Request, Response, Status, Streaming};
use tonic_health::{server::health_reporter, ServingStatus};

#[derive(Clone, prost::Message)]
struct EchoRequest {
    #[prost(string, tag = "1")]
    pub ping: String,
}

#[derive(Clone, prost::Message)]
struct StreamEchoRequest {
    #[prost(string, tag = "1")]
    pub ping: String,
    #[prost(int32, tag = "2")]
    pub count: i32,
}

#[derive(Clone, prost::Message)]
struct EchoResponse {
    #[prost(map = "string, string", tag = "1")]
    pub headers: HashMap<String, String>,
    #[prost(string, tag = "2")]
    pub body: String,
    #[prost(string, tag = "3")]
    pub remote_addr: String,
}

#[derive(Debug, Default)]
struct EchoService;

fn extract_headers(metadata: &tonic::metadata::MetadataMap) -> HashMap<String, String> {
    let mut headers = HashMap::new();
    for kv in metadata.iter() {
        if let tonic::metadata::KeyAndValueRef::Ascii(key, val) = kv {
            if let Ok(v) = val.to_str() {
                headers.insert(key.as_str().to_string(), v.to_string());
            }
        }
    }
    headers
}

#[tonic::async_trait]
impl echo_service_trait::EchoServiceServer for EchoService {
    async fn echo(&self, request: Request<EchoRequest>) -> Result<Response<EchoResponse>, Status> {
        let remote = request
            .remote_addr()
            .map(|a| a.to_string())
            .unwrap_or_default();
        let headers = extract_headers(request.metadata());
        let ping = request.into_inner().ping;
        Ok(Response::new(EchoResponse {
            headers,
            body: ping,
            remote_addr: remote,
        }))
    }

    type ServerStreamEchoStream =
        Pin<Box<dyn futures_util::Stream<Item = Result<EchoResponse, Status>> + Send>>;

    async fn server_stream_echo(
        &self,
        request: Request<StreamEchoRequest>,
    ) -> Result<Response<Self::ServerStreamEchoStream>, Status> {
        let remote = request
            .remote_addr()
            .map(|a| a.to_string())
            .unwrap_or_default();
        let headers = extract_headers(request.metadata());
        let inner = request.into_inner();
        let count = inner.count.max(1) as usize;
        let ping = inner.ping;

        let (tx, rx) = tokio::sync::mpsc::channel(count.min(64));
        tokio::spawn(async move {
            for i in 0..count {
                let resp = EchoResponse {
                    headers: headers.clone(),
                    body: format!("{}:{}", ping, i),
                    remote_addr: remote.clone(),
                };
                if tx.send(Ok(resp)).await.is_err() {
                    break;
                }
            }
        });

        Ok(Response::new(Box::pin(ReceiverStream::new(rx))))
    }

    type BidiStreamEchoStream =
        Pin<Box<dyn futures_util::Stream<Item = Result<EchoResponse, Status>> + Send>>;

    async fn bidi_stream_echo(
        &self,
        request: Request<Streaming<EchoRequest>>,
    ) -> Result<Response<Self::BidiStreamEchoStream>, Status> {
        let remote = request
            .remote_addr()
            .map(|a| a.to_string())
            .unwrap_or_default();
        let headers = extract_headers(request.metadata());
        let mut stream = request.into_inner();

        let (tx, rx) = tokio::sync::mpsc::channel(64);
        tokio::spawn(async move {
            while let Some(Ok(msg)) = stream.next().await {
                let resp = EchoResponse {
                    headers: headers.clone(),
                    body: msg.ping,
                    remote_addr: remote.clone(),
                };
                if tx.send(Ok(resp)).await.is_err() {
                    break;
                }
            }
        });

        Ok(Response::new(Box::pin(ReceiverStream::new(rx))))
    }
}

mod echo_service_trait {
    use super::{EchoRequest, EchoResponse, StreamEchoRequest};
    use std::pin::Pin;
    use tonic::{Request, Response, Status, Streaming};

    #[tonic::async_trait]
    pub trait EchoServiceServer: Send + Sync + 'static {
        async fn echo(
            &self,
            request: Request<EchoRequest>,
        ) -> Result<Response<EchoResponse>, Status>;

        type ServerStreamEchoStream: futures_util::Stream<Item = Result<EchoResponse, Status>>
            + Send
            + 'static;

        async fn server_stream_echo(
            &self,
            request: Request<StreamEchoRequest>,
        ) -> Result<Response<Self::ServerStreamEchoStream>, Status>;

        type BidiStreamEchoStream: futures_util::Stream<Item = Result<EchoResponse, Status>>
            + Send
            + 'static;

        async fn bidi_stream_echo(
            &self,
            request: Request<Streaming<EchoRequest>>,
        ) -> Result<Response<Self::BidiStreamEchoStream>, Status>;
    }

    pub struct EchoServiceServerImpl<T: EchoServiceServer> {
        inner: std::sync::Arc<T>,
    }

    impl<T: EchoServiceServer> EchoServiceServerImpl<T> {
        pub fn new(inner: T) -> Self {
            Self {
                inner: std::sync::Arc::new(inner),
            }
        }
    }

    impl<T: EchoServiceServer> Clone for EchoServiceServerImpl<T> {
        fn clone(&self) -> Self {
            Self {
                inner: self.inner.clone(),
            }
        }
    }

    impl<T: EchoServiceServer> tonic::server::NamedService for EchoServiceServerImpl<T> {
        const NAME: &'static str = "grpc_echo.v1.EchoService";
    }

    impl<T: EchoServiceServer>
        tonic::codegen::Service<tonic::codegen::http::Request<tonic::body::BoxBody>>
        for EchoServiceServerImpl<T>
    {
        type Response = tonic::codegen::http::Response<tonic::body::BoxBody>;
        type Error = std::convert::Infallible;
        type Future =
            Pin<Box<dyn std::future::Future<Output = Result<Self::Response, Self::Error>> + Send>>;

        fn poll_ready(
            &mut self,
            _cx: &mut std::task::Context<'_>,
        ) -> std::task::Poll<Result<(), Self::Error>> {
            std::task::Poll::Ready(Ok(()))
        }

        fn call(
            &mut self,
            req: tonic::codegen::http::Request<tonic::body::BoxBody>,
        ) -> Self::Future {
            let inner = self.inner.clone();
            Box::pin(async move {
                let path = req.uri().path().to_string();
                match path.as_str() {
                    "/grpc_echo.v1.EchoService/Echo" => {
                        let codec =
                            tonic::codec::ProstCodec::<EchoResponse, EchoRequest>::default();
                        let mut grpc = tonic::server::Grpc::new(codec);
                        struct Handler<T>(std::sync::Arc<T>);
                        #[tonic::async_trait]
                        impl<T: EchoServiceServer> tonic::server::UnaryService<EchoRequest> for Handler<T> {
                            type Response = EchoResponse;
                            type Future = Pin<
                                Box<
                                    dyn std::future::Future<
                                            Output = Result<Response<Self::Response>, Status>,
                                        > + Send,
                                >,
                            >;
                            fn call(&mut self, req: Request<EchoRequest>) -> Self::Future {
                                let inner = self.0.clone();
                                Box::pin(async move { inner.echo(req).await })
                            }
                        }
                        let resp = grpc.unary(Handler(inner), req).await;
                        Ok(resp)
                    }
                    "/grpc_echo.v1.EchoService/ServerStreamEcho" => {
                        let codec =
                            tonic::codec::ProstCodec::<EchoResponse, StreamEchoRequest>::default();
                        let mut grpc = tonic::server::Grpc::new(codec);
                        struct Handler<T>(std::sync::Arc<T>);
                        #[tonic::async_trait]
                        impl<T: EchoServiceServer>
                            tonic::server::ServerStreamingService<StreamEchoRequest>
                            for Handler<T>
                        {
                            type Response = EchoResponse;
                            type ResponseStream = T::ServerStreamEchoStream;
                            type Future = Pin<
                                Box<
                                    dyn std::future::Future<
                                            Output = Result<Response<Self::ResponseStream>, Status>,
                                        > + Send,
                                >,
                            >;
                            fn call(&mut self, req: Request<StreamEchoRequest>) -> Self::Future {
                                let inner = self.0.clone();
                                Box::pin(async move { inner.server_stream_echo(req).await })
                            }
                        }
                        let resp = grpc.server_streaming(Handler(inner), req).await;
                        Ok(resp)
                    }
                    "/grpc_echo.v1.EchoService/BidiStreamEcho" => {
                        let codec =
                            tonic::codec::ProstCodec::<EchoResponse, EchoRequest>::default();
                        let mut grpc = tonic::server::Grpc::new(codec);
                        struct Handler<T>(std::sync::Arc<T>);
                        #[tonic::async_trait]
                        impl<T: EchoServiceServer> tonic::server::StreamingService<EchoRequest> for Handler<T> {
                            type Response = EchoResponse;
                            type ResponseStream = T::BidiStreamEchoStream;
                            type Future = Pin<
                                Box<
                                    dyn std::future::Future<
                                            Output = Result<Response<Self::ResponseStream>, Status>,
                                        > + Send,
                                >,
                            >;
                            fn call(
                                &mut self,
                                req: Request<Streaming<EchoRequest>>,
                            ) -> Self::Future {
                                let inner = self.0.clone();
                                Box::pin(async move { inner.bidi_stream_echo(req).await })
                            }
                        }
                        let resp = grpc.streaming(Handler(inner), req).await;
                        Ok(resp)
                    }
                    _ => Ok(tonic::codegen::http::Response::builder()
                        .status(tonic::codegen::http::StatusCode::from_u16(404).unwrap())
                        .body(tonic::body::empty_body())
                        .unwrap()),
                }
            })
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let port: u16 = std::env::args()
        .nth(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(50051);

    let addr: SocketAddr = format!("127.0.0.1:{}", port).parse()?;

    let (mut reporter, health_service) = health_reporter();
    reporter
        .set_service_status("", ServingStatus::Serving)
        .await;

    let echo_service = echo_service_trait::EchoServiceServerImpl::new(EchoService);

    eprintln!("gRPC echo+health server listening on {}", addr);

    Server::builder()
        .add_service(health_service)
        .add_service(echo_service)
        .serve(addr)
        .await?;

    Ok(())
}
