use crate::{
    open_bi_guarded, send_routing_info, InflightSlotId, InflightTable, OpenBiOutcome,
    OpenedStream, OverloadLimits, ProxyError, RoutingInfo,
};
use bytes::Bytes;
use quinn::Connection;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{mpsc, oneshot};

pub type OpenWaitObserver = Box<dyn Fn(Duration, OpenBiOutcome) + Send + 'static>;

pub struct OpenStreamRequest {
    pub routing_info: RoutingInfo,
    pub initial_bytes: Option<Bytes>,
    pub overload_limits: OverloadLimits,
    pub stream_timeout: Duration,
    pub on_wait_done: Option<OpenWaitObserver>,
}

enum ConnectionOp {
    OpenStream {
        request: OpenStreamRequest,
        reply: oneshot::Sender<Result<OpenedStream, ProxyError>>,
    },
    SendDatagram {
        payload: Bytes,
        reply: oneshot::Sender<anyhow::Result<()>>,
    },
}

pub struct ConnectionHandle {
    conn: Connection,
    inflight_table: Arc<InflightTable>,
    slot_id: InflightSlotId,
    shard_id: usize,
    tx: mpsc::Sender<ConnectionOp>,
}

impl ConnectionHandle {
    pub fn spawn(
        conn: Connection,
        inflight_table: Arc<InflightTable>,
        slot_id: InflightSlotId,
        shard_id: usize,
    ) -> Arc<Self> {
        let (tx, mut rx) = mpsc::channel(128);
        let actor_conn = conn.clone();
        let actor_table = inflight_table.clone();

        tokio::spawn(async move {
            while let Some(op) = rx.recv().await {
                match op {
                    ConnectionOp::OpenStream { request, reply } => {
                        let mut wait_observer = request.on_wait_done;
                        let result = match open_bi_guarded(
                            &actor_conn,
                            &actor_table,
                            slot_id,
                            &request.overload_limits,
                            request.stream_timeout,
                            move |elapsed, outcome| {
                                if let Some(observer) = wait_observer.take() {
                                    observer(elapsed, outcome);
                                }
                            },
                        )
                        .await
                        {
                            Ok(mut opened) => {
                                if let Err(error) =
                                    send_routing_info(&mut opened.send, &request.routing_info).await
                                {
                                    Err(ProxyError::quic_connection_lost(error.to_string()))
                                } else if let Some(initial_bytes) = request.initial_bytes {
                                    if let Err(error) = opened.send.write_all(&initial_bytes).await {
                                        Err(ProxyError::quic_connection_lost(error.to_string()))
                                    } else {
                                        Ok(opened)
                                    }
                                } else {
                                    Ok(opened)
                                }
                            }
                            Err(error) => Err(error),
                        };
                        let _ = reply.send(result);
                    }
                    ConnectionOp::SendDatagram { payload, reply } => {
                        let result = actor_conn
                            .send_datagram(payload)
                            .map_err(|error| anyhow::anyhow!(error.to_string()));
                        let _ = reply.send(result);
                    }
                }
            }
        });

        Arc::new(Self {
            conn,
            inflight_table,
            slot_id,
            shard_id,
            tx,
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

    pub async fn open_stream(&self, request: OpenStreamRequest) -> Result<OpenedStream, ProxyError> {
        let (reply_tx, reply_rx) = oneshot::channel();
        self.tx
            .send(ConnectionOp::OpenStream {
                request,
                reply: reply_tx,
            })
            .await
            .map_err(|_| ProxyError::quic_connection_lost("connection actor channel closed"))?;
        reply_rx
            .await
            .unwrap_or_else(|_| Err(ProxyError::quic_connection_lost("connection actor dropped")))
    }

    pub async fn send_datagram(&self, payload: Bytes) -> anyhow::Result<()> {
        let (reply_tx, reply_rx) = oneshot::channel();
        self.tx
            .send(ConnectionOp::SendDatagram {
                payload,
                reply: reply_tx,
            })
            .await
            .map_err(|_| anyhow::anyhow!("connection actor channel closed"))?;
        reply_rx
            .await
            .unwrap_or_else(|_| Err(anyhow::anyhow!("connection actor dropped")))
    }
}
