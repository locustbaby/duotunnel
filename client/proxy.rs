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
    tcp_params: tunnel_lib::TcpParams,
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

        let upstream_addr = proxy_map.get_local_address(&routing_info.proxy_name, Some(&host), false)
            .ok_or_else(|| anyhow::anyhow!("no upstream for H2 request to {}", host))?;

        info!("H2 protocol detected, forwarding to upstream: {}", upstream_addr);

        let service = service_fn(move |req: Request<hyper::body::Incoming>| {
            let upstream_addr = upstream_addr.clone();
            async move {
                let (mut parts, body) = req.into_parts();

                let normalized_addr = upstream_addr
                    .replace("wss://", "https://")
                    .replace("ws://", "http://");
                let normalized_addr = if normalized_addr.contains("://") {
                    normalized_addr
                } else {
                    format!("http://{}", normalized_addr)
                };
                let upstream_uri: hyper::Uri = match normalized_addr.parse() {
                    Ok(uri) => uri,
                    Err(e) => {
                        tracing::error!("invalid upstream URI '{}': {}", normalized_addr, e);
                        return Ok(Response::builder()
                            .status(502)
                            .body(http_body_util::Full::new(bytes::Bytes::from("Bad Gateway"))
                                .map_err(|_| unreachable!())
                                .boxed_unsync())
                            .unwrap());
                    }
                };

                let connect_host = upstream_uri.host().unwrap_or("127.0.0.1").to_string();
                let connect_port = upstream_uri.port_u16()
                    .unwrap_or(if upstream_uri.scheme_str() == Some("https") { 443 } else { 80 });
                let is_https = upstream_uri.scheme_str() == Some("https");

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

                debug!("Client H2: forwarding {} {} to upstream {}:{}", parts.method, parts.uri, connect_host, connect_port);

                let boxed_body = body.map_err(std::io::Error::other).boxed_unsync();
                let upstream_req = Request::from_parts(parts, boxed_body);

                // Connect to upstream using H2 prior knowledge (plaintext or TLS).
                // We bypass the pooled client and establish a direct H2 connection
                // so that gRPC servers (which require H2) are handled correctly.
                let connect_addr = format!("{connect_host}:{connect_port}");
                let tcp = match tokio::net::TcpStream::connect(&connect_addr).await {
                    Ok(s) => s,
                    Err(e) => {
                        tracing::error!("Client H2: TCP connect to {} failed: {}", connect_addr, e);
                        return Ok(Response::builder()
                            .status(502)
                            .body(http_body_util::Full::new(bytes::Bytes::from("Bad Gateway")).map_err(|_| unreachable!()).boxed_unsync())
                            .unwrap());
                    }
                };

                if is_https {
                    // TLS upstream: establish TLS + H2 (ALPN "h2") over the TCP socket.
                    use tokio_rustls::TlsConnector;
                    use rustls::ClientConfig as RustlsClientConfig;
                    use rustls::RootCertStore;

                    let mut root_store = RootCertStore::empty();
                    root_store.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());
                    let mut tls_cfg = RustlsClientConfig::builder()
                        .with_root_certificates(root_store)
                        .with_no_client_auth();
                    tls_cfg.alpn_protocols = vec![b"h2".to_vec()];

                    let connector = TlsConnector::from(Arc::new(tls_cfg));
                    let server_name = match rustls::pki_types::ServerName::try_from(connect_host.clone()) {
                        Ok(n) => n,
                        Err(e) => {
                            tracing::error!("Client H2: invalid server name '{}': {}", connect_host, e);
                            return Ok(Response::builder()
                                .status(502)
                                .body(http_body_util::Full::new(bytes::Bytes::from("Bad Gateway")).map_err(|_| unreachable!()).boxed_unsync())
                                .unwrap());
                        }
                    };
                    let tls_stream = match connector.connect(server_name, tcp).await {
                        Ok(s) => s,
                        Err(e) => {
                            tracing::error!("Client H2: TLS connect to {} failed: {}", connect_addr, e);
                            return Ok(Response::builder()
                                .status(502)
                                .body(http_body_util::Full::new(bytes::Bytes::from("Bad Gateway")).map_err(|_| unreachable!()).boxed_unsync())
                                .unwrap());
                        }
                    };
                    let io = TokioIo::new(tls_stream);
                    let (mut sender, conn) = match hyper::client::conn::http2::handshake(
                        hyper_util::rt::TokioExecutor::new(), io
                    ).await {
                        Ok(pair) => pair,
                        Err(e) => {
                            tracing::error!("Client H2: TLS H2 handshake to {} failed: {}", connect_addr, e);
                            return Ok(Response::builder()
                                .status(502)
                                .body(http_body_util::Full::new(bytes::Bytes::from("Bad Gateway")).map_err(|_| unreachable!()).boxed_unsync())
                                .unwrap());
                        }
                    };
                    tokio::spawn(async move {
                        if let Err(e) = conn.await {
                            debug!("Client H2: TLS upstream connection error: {}", e);
                        }
                    });
                    return match sender.send_request(upstream_req).await {
                        Ok(resp) => {
                            let (parts, body) = resp.into_parts();
                            let boxed = body.map_err(std::io::Error::other).boxed_unsync();
                            Ok(Response::from_parts(parts, boxed))
                        }
                        Err(e) => {
                            tracing::error!("Client H2: TLS upstream request failed: {}", e);
                            Ok(Response::builder()
                                .status(502)
                                .body(http_body_util::Full::new(bytes::Bytes::from("Bad Gateway")).map_err(|_| unreachable!()).boxed_unsync())
                                .unwrap())
                        }
                    };
                }

                let io = TokioIo::new(tcp);
                let (mut sender, conn) = match hyper::client::conn::http2::handshake(
                    hyper_util::rt::TokioExecutor::new(), io
                ).await {
                    Ok(pair) => pair,
                    Err(e) => {
                        tracing::error!("Client H2: H2 handshake to upstream {} failed: {}", connect_addr, e);
                        return Ok(Response::builder()
                            .status(502)
                            .body(http_body_util::Full::new(bytes::Bytes::from("Bad Gateway")).map_err(|_| unreachable!()).boxed_unsync())
                            .unwrap());
                    }
                };
                tokio::spawn(async move {
                    if let Err(e) = conn.await {
                        debug!("Client H2: upstream connection error: {}", e);
                    }
                });

                match sender.send_request(upstream_req).await {
                    Ok(resp) => {
                        let (parts, body) = resp.into_parts();
                        let boxed = body.map_err(std::io::Error::other).boxed_unsync();
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
        let app = ClientApp::new(proxy_map, tcp_params);
        let engine = ProxyEngine::new(app);

        let client_addr = format!("{}:{}", routing_info.src_addr, routing_info.src_port)
            .parse()
            .unwrap_or_else(|_| "0.0.0.0:0".parse().unwrap());

        engine.run_stream(send, recv, client_addr, Some(routing_info)).await?;
    }

    Ok(())
}
