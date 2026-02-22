/// gRPC server for CI integration tests.
///
/// Exposes two services on the same port (default 50051):
///   - grpc.health.v1.Health/Check   → always SERVING
///   - grpc_echo.v1.EchoService/Echo → echoes request headers + body back
///
/// EchoResponse mirrors grpc-echo.semior.dev:
///   message EchoResponse {
///     map<string, string> headers  = 1;
///     string body                  = 2;
///     string remote_addr           = 3;
///   }
use std::collections::HashMap;
use std::net::SocketAddr;
use tonic::{Request, Response, Status, transport::Server};
use tonic_health::{server::health_reporter, ServingStatus};

// ─── Protobuf types (hand-rolled with prost) ─────────────────────────────────

#[derive(Clone, prost::Message)]
struct EchoRequest {
    #[prost(string, tag = "1")]
    pub ping: String,
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

// ─── EchoService implementation ──────────────────────────────────────────────

#[derive(Debug, Default)]
struct EchoService;

#[tonic::async_trait]
impl grpc_echo_server_trait::EchoServiceServer for EchoService {
    async fn echo(
        &self,
        request: Request<EchoRequest>,
    ) -> Result<Response<EchoResponse>, Status> {
        let remote = request
            .remote_addr()
            .map(|a| a.to_string())
            .unwrap_or_default();

        let mut headers = HashMap::new();
        for kv in request.metadata().iter() {
            match kv {
                tonic::metadata::KeyAndValueRef::Ascii(key, val) => {
                    if let Ok(v) = val.to_str() {
                        headers.insert(key.as_str().to_string(), v.to_string());
                    }
                }
                tonic::metadata::KeyAndValueRef::Binary(_, _) => {}
            }
        }

        let ping = request.into_inner().ping;

        let resp = EchoResponse {
            headers,
            body: ping,
            remote_addr: remote,
        };
        Ok(Response::new(resp))
    }
}

// ─── Tonic service descriptor (hand-rolled, no codegen) ──────────────────────

mod grpc_echo_server_trait {
    use super::{EchoRequest, EchoResponse};
    use tonic::{Request, Response, Status};

    #[tonic::async_trait]
    pub trait EchoServiceServer: Send + Sync + 'static {
        async fn echo(
            &self,
            request: Request<EchoRequest>,
        ) -> Result<Response<EchoResponse>, Status>;
    }

    pub struct EchoServiceServerImpl<T: EchoServiceServer> {
        inner: std::sync::Arc<T>,
    }

    impl<T: EchoServiceServer> EchoServiceServerImpl<T> {
        pub fn new(inner: T) -> Self {
            Self { inner: std::sync::Arc::new(inner) }
        }
    }

    impl<T: EchoServiceServer> tonic::codegen::Service<tonic::codegen::http::Request<tonic::body::BoxBody>>
        for EchoServiceServerImpl<T>
    {
        type Response = tonic::codegen::http::Response<tonic::body::BoxBody>;
        type Error = std::convert::Infallible;
        type Future = std::pin::Pin<
            Box<dyn std::future::Future<Output = Result<Self::Response, Self::Error>> + Send>,
        >;

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
                        let codec = tonic::codec::ProstCodec::<EchoResponse, EchoRequest>::default();
                        let mut grpc = tonic::server::Grpc::new(codec);
                        struct Handler<T>(std::sync::Arc<T>);
                        #[tonic::async_trait]
                        impl<T: EchoServiceServer> tonic::server::UnaryService<EchoRequest> for Handler<T> {
                            type Response = EchoResponse;
                            type Future = std::pin::Pin<Box<dyn std::future::Future<Output = Result<Response<Self::Response>, Status>> + Send>>;
                            fn call(&mut self, req: Request<EchoRequest>) -> Self::Future {
                                let inner = self.0.clone();
                                Box::pin(async move { inner.echo(req).await })
                            }
                        }
                        let resp = grpc.unary(Handler(inner), req).await;
                        Ok(resp)
                    }
                    _ => {
                        Ok(tonic::codegen::http::Response::builder()
                            .status(tonic::codegen::http::StatusCode::from_u16(404).unwrap())
                            .body(tonic::body::empty_body())
                            .unwrap())
                    }
                }
            })
        }
    }

    impl<T: EchoServiceServer> Clone for EchoServiceServerImpl<T> {
        fn clone(&self) -> Self {
            Self { inner: self.inner.clone() }
        }
    }

    impl<T: EchoServiceServer> tonic::server::NamedService for EchoServiceServerImpl<T> {
        const NAME: &'static str = "grpc_echo.v1.EchoService";
    }
}

// ─── main ────────────────────────────────────────────────────────────────────

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let port: u16 = std::env::args()
        .nth(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(50051);

    let addr: SocketAddr = format!("127.0.0.1:{}", port).parse()?;

    let (mut reporter, health_service) = health_reporter();
    reporter.set_service_status("", ServingStatus::Serving).await;

    let echo_service = grpc_echo_server_trait::EchoServiceServerImpl::new(EchoService);

    eprintln!("gRPC echo+health server listening on {}", addr);

    Server::builder()
        .add_service(health_service)
        .add_service(echo_service)
        .serve(addr)
        .await?;

    Ok(())
}
