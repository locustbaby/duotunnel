use std::collections::HashMap;
use std::net::SocketAddr;
use std::pin::Pin;
use tokio_stream::{wrappers::ReceiverStream, StreamExt};
use tonic::{transport::Server, Request, Response, Status, Streaming};
use tonic_health::{server::health_reporter, ServingStatus};

pub mod grpc_echo {
    tonic::include_proto!("grpc_echo.v1");
}

use grpc_echo::echo_service_server::{EchoService as EchoServiceTrait, EchoServiceServer};
use grpc_echo::{EchoRequest, EchoResponse, StreamEchoRequest};

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
impl EchoServiceTrait for EchoService {
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

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let port: u16 = std::env::args()
        .nth(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(50051);

    let addr: SocketAddr = format!("127.0.0.1:{}", port).parse()?;

    let (reporter, health_service) = health_reporter();
    reporter
        .set_service_status("", ServingStatus::Serving)
        .await;

    eprintln!("gRPC echo+health server listening on {}", addr);

    Server::builder()
        .add_service(health_service)
        .add_service(EchoServiceServer::new(EchoService))
        .serve(addr)
        .await?;

    Ok(())
}
