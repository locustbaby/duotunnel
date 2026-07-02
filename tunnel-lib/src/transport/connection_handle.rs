use crate::{
    open_bi_guarded, send_routing_info, InflightSlotId, InflightTable, OpenBiOutcome, OpenedStream,
    OverloadLimits, ProxyError, RoutingInfo,
};
use bytes::Bytes;
use quinn::Connection;
use std::sync::Arc;
use std::time::Duration;

pub type OpenWaitObserver = Box<dyn Fn(Duration, OpenBiOutcome) + Send + 'static>;

pub struct OpenStreamRequest {
    pub routing_info: RoutingInfo,
    pub initial_bytes: Option<Bytes>,
    pub overload_limits: OverloadLimits,
    pub stream_timeout: Duration,
    pub on_wait_done: Option<OpenWaitObserver>,
}

pub struct ConnectionHandle {
    conn: Connection,
    inflight_table: Arc<InflightTable>,
    slot_id: InflightSlotId,
    shard_id: usize,
    stream_semaphore: Arc<tokio::sync::Semaphore>,
}

impl ConnectionHandle {
    pub fn spawn(
        conn: Connection,
        inflight_table: Arc<InflightTable>,
        slot_id: InflightSlotId,
        shard_id: usize,
        max_concurrent_streams: u32,
    ) -> Arc<Self> {
        let stream_semaphore =
            Arc::new(tokio::sync::Semaphore::new(max_concurrent_streams as usize));

        Arc::new(Self {
            conn,
            inflight_table,
            slot_id,
            shard_id,
            stream_semaphore,
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

    pub fn inflight_table(&self) -> &Arc<InflightTable> {
        &self.inflight_table
    }

    pub fn slot_id(&self) -> InflightSlotId {
        self.slot_id
    }

    pub async fn open_stream(
        &self,
        request: OpenStreamRequest,
    ) -> Result<OpenedStream, ProxyError> {
        let permit = self
            .stream_semaphore
            .clone()
            .try_acquire_owned()
            .map_err(|_| {
                ProxyError::quic_open_rejected_overloaded("stream concurrency limit reached")
            })?;

        let mut wait_observer = request.on_wait_done;
        let mut opened = open_bi_guarded(
            &self.conn,
            &self.inflight_table,
            self.slot_id,
            &request.overload_limits,
            request.stream_timeout,
            Some(permit),
            move |elapsed, outcome| {
                if let Some(observer) = wait_observer.take() {
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
        self.conn
            .send_datagram(payload)
            .map_err(|error| anyhow::anyhow!(error.to_string()))
    }
}
