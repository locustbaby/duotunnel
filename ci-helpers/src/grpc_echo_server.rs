/// Minimal gRPC server for CI integration tests.
///
/// Exposes the standard gRPC Health Checking Protocol on port 50051 (or the
/// port given by the first CLI argument).  Clients can verify end-to-end
/// gRPC through the tunnel with:
///
///   grpcurl -plaintext <host>:<port> grpc.health.v1.Health/Check
///
/// The health service reports the root service ("") as SERVING.
use std::net::SocketAddr;
use tonic::transport::Server;
use tonic_health::{server::health_reporter, ServingStatus};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let port: u16 = std::env::args()
        .nth(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(50051);

    let addr: SocketAddr = format!("127.0.0.1:{}", port).parse()?;

    let (mut reporter, health_service) = health_reporter();
    // Empty string = root service; any Health/Check with service="" will return SERVING.
    reporter.set_service_status("", ServingStatus::Serving).await;

    eprintln!("gRPC health server listening on {}", addr);

    Server::builder()
        .add_service(health_service)
        .serve(addr)
        .await?;

    Ok(())
}
