use anyhow::Result;
use quinn::{SendStream, RecvStream};
use tracing::{info, debug};
use std::sync::Arc;
use tunnel_lib::recv_routing_info;
use tunnel_lib::proxy::core::ProxyEngine;
use crate::app::{ClientApp, LocalProxyMap};

pub async fn handle_work_stream(
    send: SendStream, 
    mut recv: RecvStream,
    proxy_map: Arc<LocalProxyMap>,
) -> Result<()> {
    use hyper::server::conn::http2::Builder as H2Builder;
    use hyper::service::service_fn;
    use hyper::{Request, Response};
    use http_body_util::BodyExt;
    use hyper_util::rt::TokioIo;
    use tunnel_lib::QuinnStream;

    let routing_info = recv_routing_info(&mut recv).await?;

    info!(
        proxy_name = %routing_info.proxy_name,
        protocol = %routing_info.protocol,
        host = ?routing_info.host,
        src = format!("{}:{}", routing_info.src_addr, routing_info.src_port),
        "received work stream"
    );

    if routing_info.protocol == "h2" {
        let host = routing_info.host.clone().unwrap_or_else(|| routing_info.proxy_name.clone());
        let host_without_port = host.split(':').next().unwrap_or(&host);
        
        let upstream_addr = proxy_map.get_local_address(&routing_info.proxy_name, Some(&host), false)
            .ok_or_else(|| anyhow::anyhow!("no upstream for H2 request to {}", host))?;
        
        info!("H2 protocol detected, forwarding to upstream: {}", upstream_addr);
        
        let https_client = proxy_map.https_client.clone();
        
        let service = service_fn(move |req: Request<hyper::body::Incoming>| {
            let upstream_addr = upstream_addr.clone();
            let https_client = https_client.clone();
            async move {
                let (mut parts, body) = req.into_parts();
                
                let normalized_addr = upstream_addr
                    .replace("wss://", "https://")
                    .replace("ws://", "http://");
                let upstream_uri: hyper::Uri = normalized_addr.parse().unwrap();
                let mut uri_parts = parts.uri.clone().into_parts();
                uri_parts.scheme = upstream_uri.scheme().cloned();
                uri_parts.authority = upstream_uri.authority().cloned();
                if let Ok(new_uri) = hyper::Uri::from_parts(uri_parts) {
                    parts.uri = new_uri;
                }
                if let Some(auth) = upstream_uri.authority() {
                    if let Ok(hv) = auth.as_str().parse() {
                        parts.headers.insert(hyper::header::HOST, hv);
                    }
                }
                
                debug!("Client H2: forwarding {} {} to upstream", parts.method, parts.uri);
                
                let boxed_body = body.map_err(|e| std::io::Error::other(e)).boxed_unsync();
                let upstream_req = Request::from_parts(parts, boxed_body);
                
                match https_client.request(upstream_req).await {
                    Ok(resp) => {
                        let (parts, body) = resp.into_parts();
                        let boxed = body.map_err(|e| std::io::Error::other(e)).boxed_unsync();
                        Ok::<_, hyper::Error>(Response::from_parts(parts, boxed))
                    }
                    Err(e) => {
                        tracing::error!("Client H2: upstream request failed: {}", e);
                        Ok(Response::builder()
                            .status(502)
                            .body(http_body_util::Full::new(bytes::Bytes::from("Bad Gateway")).map_err(|_| unreachable!()).boxed_unsync())
                            .unwrap())
                    }
                }
            }
        });

        let quic_stream = QuinnStream { send, recv };
        let io = TokioIo::new(quic_stream);
        
        H2Builder::new(hyper_util::rt::TokioExecutor::new())
            .serve_connection(io, service)
            .await
            .map_err(|e| anyhow::anyhow!("Client H2 connection error: {}", e))?;
    } else {
        let app = ClientApp::new(proxy_map);
        let engine = ProxyEngine::new(app);

        let client_addr = format!("{}:{}", routing_info.src_addr, routing_info.src_port)
            .parse()
            .unwrap_or_else(|_| "0.0.0.0:0".parse().unwrap());

        engine.run_stream(send, recv, client_addr, Some(routing_info)).await?;
    }

    Ok(())
}
