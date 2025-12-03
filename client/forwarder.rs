use anyhow::Result;
use std::sync::Arc;
use crate::types::ClientState;
use crate::http_forwarder::forward_http_request;
use crate::wss_forwarder::forward_wss_request;
use crate::grpc_forwarder::forward_grpc_request;

pub type ForwardResult = Result<Vec<u8>>;

#[derive(Clone)]
pub struct Forwarder {
    state: Arc<ClientState>,
}

impl Forwarder {
    pub fn new(state: Arc<ClientState>) -> Self {
        Self { state }
    }

    pub async fn forward(
        &self,
        protocol_type: &str,
        request_bytes: &[u8],
        target_uri: &str,
        is_ssl: bool,
    ) -> ForwardResult {
        let client = self.state.egress_pool.client();
        
        match protocol_type {
            "http" => {
                forward_http_request(
                    &client,
                    request_bytes.as_ref(),
                    target_uri,
                    is_ssl,
                ).await
            }
            "wss" => {
                forward_wss_request(
                    request_bytes.as_ref(),
                    target_uri,
                    is_ssl,
                ).await
            }
            "grpc" => {
                forward_grpc_request(
                    request_bytes.as_ref(),
                    target_uri,
                    is_ssl,
                ).await
            }
            _ => {
                Err(anyhow::anyhow!("Unsupported protocol type: {}", protocol_type))
            }
        }
    }
}

