use crate::error::ProxyError;
use crate::inflight::{begin_inflight, InflightGuard, InflightSlotId, InflightTable};
use crate::timeout;
use futures_util::FutureExt;
use quinn::{Connection, RecvStream, SendStream};
use std::sync::Arc;
use std::time::{Duration, Instant};

#[derive(Debug)]
pub enum OpenBiOutcome {
    Ready,
    TimedOut,
    ConnectionLost,
    ConnectionClosing,
    TransportFatal,
}

pub struct OpenedStream {
    pub send: SendStream,
    pub recv: RecvStream,
    pub inflight: InflightGuard,
}

pub async fn open_bi_guarded<F>(
    conn: &Connection,
    inflight_table: &Arc<InflightTable>,
    slot_id: InflightSlotId,
    stream_timeout: Duration,
    on_wait_done: F,
) -> Result<OpenedStream, ProxyError>
where
    F: FnOnce(Duration, OpenBiOutcome),
{
    let guard = begin_inflight(inflight_table, slot_id);
    let immediate = conn.open_bi().now_or_never();
    match immediate {
        Some(Ok((send, recv))) => {
            return Ok(OpenedStream {
                send,
                recv,
                inflight: guard.promote(),
            });
        }
        Some(Err(error)) => {
            let (_, err) = classify_open_bi_error(error);
            return Err(err);
        }
        None => {}
    }

    let started = Instant::now();
    crate::infra::metrics::METRICS.stream_pending_queue_depth.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let result = timeout(stream_timeout, conn.open_bi()).await;
    crate::infra::metrics::METRICS.stream_pending_queue_depth.fetch_sub(1, std::sync::atomic::Ordering::Relaxed);
    let elapsed = started.elapsed();
    match result {
        Ok(Ok((send, recv))) => {
            on_wait_done(elapsed, OpenBiOutcome::Ready);
            Ok(OpenedStream {
                send,
                recv,
                inflight: guard.promote(),
            })
        }
        Ok(Err(error)) => {
            let (outcome, err) = classify_open_bi_error(error);
            on_wait_done(elapsed, outcome);
            Err(err)
        }
        Err(_) => {
            on_wait_done(elapsed, OpenBiOutcome::TimedOut);
            Err(ProxyError::quic_open_timed_out(stream_timeout))
        }
    }
}

fn classify_open_bi_error(error: quinn::ConnectionError) -> (OpenBiOutcome, ProxyError) {
    use quinn::ConnectionError;

    match error {
        ConnectionError::TimedOut
        | ConnectionError::Reset
        | ConnectionError::ConnectionClosed(_)
        | ConnectionError::ApplicationClosed(_) => (
            OpenBiOutcome::ConnectionLost,
            ProxyError::quic_connection_lost(error.to_string()),
        ),
        ConnectionError::LocallyClosed => (
            OpenBiOutcome::ConnectionClosing,
            ProxyError::quic_connection_lost(error.to_string()),
        ),
        ConnectionError::VersionMismatch
        | ConnectionError::TransportError(_)
        | ConnectionError::CidsExhausted => (
            OpenBiOutcome::TransportFatal,
            ProxyError::quic_connection_fatal(error.to_string()),
        ),
    }
}
