use crate::app::{ClientApp, LocalProxyMap};
use anyhow::Result;
use http_body_util::BodyExt;
use quinn::{RecvStream, SendStream};
use std::sync::Arc;
use std::time::Instant;
use tracing::{error, info, warn};
use tunnel_lib::proxy::core::{FlowDirection, StreamFlow};
use tunnel_lib::proxy::core::Protocol;
use tunnel_lib::proxy::tcp::UpstreamScheme;
use tunnel_lib::recv_routing_info;

fn http_error_response(
    status: hyper::StatusCode,
    body: &'static str,
) -> hyper::Response<http_body_util::combinators::UnsyncBoxBody<bytes::Bytes, std::io::Error>> {
    hyper::Response::builder()
        .status(status)
        .body(
            http_body_util::Full::new(bytes::Bytes::from(body))
                .map_err(|_| unreachable!())
                .boxed_unsync(),
        )
        .unwrap()
}

async fn write_http_response(
    send: &mut SendStream,
    response: hyper::Response<impl hyper::body::Body<Data = bytes::Bytes, Error = impl Into<Box<dyn std::error::Error + Send + Sync>> + 'static> + Send>,
) -> Result<()> {
    let (parts, body) = response.into_parts();
    let head = tunnel_lib::HttpResponseHead::from_parts(&parts);
    tunnel_lib::send_http_response_head(send, &head).await?;
    tunnel_lib::send_http_body(send, body).await?;
    send.finish()?;
    Ok(())
}

async fn handle_h1_request_stream(
    mut send: SendStream,
    mut recv: RecvStream,
    routing_info: tunnel_lib::RoutingInfo,
    proxy_map: Arc<LocalProxyMap>,
) -> Result<()> {
    let t0 = Instant::now();
    let head = match tunnel_lib::recv_http_request_head(&mut recv).await {
        Ok(h) => h,
        Err(err) => {
            warn!(proxy_name = %routing_info.proxy_name, error = %err, "client H1 recv_request_head failed");
            return Err(err);
        }
    };
    let method = head.method.clone();
    let uri = head.uri.clone();
    let t_head = t0.elapsed().as_millis();
    let upstream_addr = proxy_map
        .get_local_address(&routing_info.proxy_name)
        .ok_or_else(|| anyhow::anyhow!("no upstream for proxy_name: {}", routing_info.proxy_name))?;
    let (scheme, connect_addr, _tls_host) = UpstreamScheme::from_address(&upstream_addr);
    let flow = tunnel_lib::HttpFlow::new(
        tunnel_lib::FixedHttpFlowResolver::new(connect_addr.clone())
            .with_route_key(routing_info.proxy_name.clone())
            .with_selected_endpoint(connect_addr.clone()),
    );
    let parts = head.into_parts()?;
    let request_body = tunnel_lib::recv_http_body(recv);
    let request = hyper::Request::from_parts(parts, request_body);
    let resolved = flow
        .resolve_target_for_request(0, &request, Protocol::H1)
        .await?
        .expect("fixed resolver always resolves");
    let target_host = resolved
        .context
        .target_host
        .clone()
        .unwrap_or(connect_addr.clone());
    let (mut parts, body) = request.into_parts();
    tunnel_lib::rewrite_request_upstream(
        &mut parts,
        if scheme.requires_tls() { "https" } else { "http" },
        &target_host,
    );
    let upstream_req = hyper::Request::from_parts(parts, body);
    let t_upstream_start = t0.elapsed().as_millis();
    let response = match proxy_map.https_client.request(upstream_req).await {
        Ok(resp) => {
            let t_upstream = t0.elapsed().as_millis();
            tracing::trace!(
                proxy_name = %routing_info.proxy_name,
                method = %method,
                uri = %uri,
                head_ms = t_head,
                upstream_ms = t_upstream - t_upstream_start,
                "client H1 upstream ok"
            );
            let (mut parts, body) = resp.into_parts();
            if let Err(err) = flow.filter_response_parts(&resolved, &mut parts).await {
                error!(error = %err, "client H1 response filter error");
            }
            hyper::Response::from_parts(parts, body.map_err(std::io::Error::other).boxed_unsync())
        }
        Err(err) => {
            error!(proxy_name = %routing_info.proxy_name, method = %method, uri = %uri, elapsed_ms = %t0.elapsed().as_millis(), error = %err, "client H1 upstream request failed");
            http_error_response(hyper::StatusCode::BAD_GATEWAY, "Bad Gateway")
        }
    };
    if let Err(err) = write_http_response(&mut send, response).await {
        warn!(proxy_name = %routing_info.proxy_name, method = %method, uri = %uri, elapsed_ms = %t0.elapsed().as_millis(), error = %err, "client H1 write_response failed");
        return Err(err);
    }
    tracing::trace!(
        proxy_name = %routing_info.proxy_name,
        method = %method,
        uri = %uri,
        total_ms = %t0.elapsed().as_millis(),
        "client H1 stream done"
    );
    Ok(())
}

pub async fn handle_work_stream(
    send: SendStream,
    mut recv: RecvStream,
    proxy_map: Arc<LocalProxyMap>,
    tcp_params: tunnel_lib::TcpParams,
) -> Result<()> {
    let routing_info = match recv_routing_info(&mut recv).await {
        Ok(r) => r,
        Err(err) => {
            warn!(error = %err, "client recv_routing_info failed");
            return Err(err);
        }
    };
    info!(
        proxy_name = % routing_info.proxy_name, protocol = ? routing_info.protocol,
        "received work stream"
    );
    if routing_info.protocol == Protocol::H1 {
        return handle_h1_request_stream(send, recv, routing_info, proxy_map).await;
    }
    let app = ClientApp::new(proxy_map, tcp_params);
    let engine = StreamFlow::new(app);
    engine
        .run_stream(
            send,
            recv,
            "0.0.0.0:0".parse().unwrap(),
            FlowDirection::Ingress,
            Some(routing_info),
        )
        .await
}
