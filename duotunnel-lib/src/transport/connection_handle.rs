use crate::{
    open_bi_guarded, send_routing_info, ConnectionState, OpenBiOutcome, OpenedStream, ProxyError,
    RoutingInfo,
};
use bytes::Bytes;
use quinn::Connection;
use std::sync::Arc;
use std::time::Duration;

pub type OpenWaitObserver = fn(Duration, OpenBiOutcome);

pub struct OpenStreamRequest {
    pub routing_info: RoutingInfo,
    pub initial_bytes: Option<Bytes>,
    pub stream_timeout: Duration,
    pub on_wait_done: Option<OpenWaitObserver>,
}

pub struct ConnectionHandle {
    conn: Connection,
    connection_state: Arc<ConnectionState>,
    shard_id: usize,
    stream_semaphore: Arc<tokio::sync::Semaphore>,
    pending_semaphore: Arc<tokio::sync::Semaphore>,
    pending_limit: usize,
}

impl ConnectionHandle {
    pub fn spawn(
        conn: Connection,
        connection_state: Arc<ConnectionState>,
        shard_id: usize,
        max_concurrent_streams: u32,
        max_pending_streams: usize,
    ) -> Arc<Self> {
        let stream_limit = max_concurrent_streams.max(1) as usize;
        let stream_semaphore = Arc::new(tokio::sync::Semaphore::new(stream_limit));
        let pending_limit = max_pending_streams.max(1).min(stream_limit);
        let pending_semaphore = Arc::new(tokio::sync::Semaphore::new(pending_limit));

        Arc::new(Self {
            conn,
            connection_state,
            shard_id,
            stream_semaphore,
            pending_semaphore,
            pending_limit,
        })
    }

    pub fn stable_id(&self) -> usize {
        self.conn.stable_id()
    }

    pub fn shard_id(&self) -> usize {
        self.shard_id
    }

    pub fn close_reason(&self) -> Option<quinn::ConnectionError> {
        self.conn.close_reason()
    }

    pub fn is_selectable(&self) -> bool {
        self.connection_state.is_selectable()
    }

    pub fn has_stream_capacity(&self) -> bool {
        self.stream_semaphore.available_permits() > 0
    }

    pub fn connection_state(&self) -> &Arc<ConnectionState> {
        &self.connection_state
    }

    pub fn retire(&self) -> bool {
        self.connection_state.retire()
    }

    pub fn close(&self, code: u32, reason: &[u8]) {
        self.conn.close(code.into(), reason);
    }

    pub async fn open_stream(
        &self,
        request: OpenStreamRequest,
    ) -> Result<OpenedStream, ProxyError> {
        if !self.connection_state.is_selectable() {
            return Err(ProxyError::quic_connection_lost("connection retired"));
        }
        let permit = self
            .stream_semaphore
            .clone()
            .try_acquire_owned()
            .map_err(|_| {
                ProxyError::quic_open_rejected_overloaded("stream concurrency limit reached")
            })?;

        let wait_observer = request.on_wait_done;
        let mut opened = open_bi_guarded(
            &self.conn,
            &self.connection_state,
            request.stream_timeout,
            Some(permit),
            crate::transport::open_bi::PendingAdmission {
                semaphore: &self.pending_semaphore,
                limit: self.pending_limit,
            },
            move |elapsed, outcome| {
                if let Some(observer) = wait_observer {
                    observer(elapsed, outcome);
                }
            },
        )
        .await?;

        if let Err(error) = send_routing_info(&mut opened.send, &request.routing_info).await {
            return Err(ProxyError::quic_connection_lost(error.to_string()));
        }

        if let Some(initial_bytes) = request.initial_bytes {
            if let Err(error) = opened.send.write_all(&initial_bytes).await {
                return Err(ProxyError::quic_connection_lost(error.to_string()));
            }
        }

        Ok(opened)
    }

    pub async fn send_datagram(&self, payload: Bytes) -> anyhow::Result<()> {
        let _inflight = crate::begin_inflight(&self.connection_state)
            .ok_or_else(|| anyhow::anyhow!("connection retired"))?;
        self.conn
            .send_datagram(payload)
            .map_err(|error| anyhow::anyhow!(error.to_string()))
    }
}
