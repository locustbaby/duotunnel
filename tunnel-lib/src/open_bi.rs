use crate::inflight::{begin_inflight, InflightCounter, InflightGuard};
use crate::overload::{maybe_slow_path, OverloadLimits};
use anyhow::anyhow;
use quinn::{Connection, RecvStream, SendStream};
use std::sync::atomic::Ordering;
use std::time::{Duration, Instant};

#[derive(Debug)]
pub enum OpenBiOutcome {
    Ok,
    Timeout,
    ConnectionError,
}

pub struct OpenedStream {
    pub send: SendStream,
    pub recv: RecvStream,
    /// RAII guard holding the per-connection inflight slot.  Keep it
    /// alive for the full lifetime of the stream (through `relay_*`)
    /// so `pick_least_inflight` sees accurate load.
    pub inflight: InflightGuard,
}

/// Open a bidirectional QUIC stream with overload protection.
///
/// Flow:
/// 1. `maybe_slow_path(inflight, limits)` — yield/backoff if the
///    connection is near its `max_concurrent_streams` cap.
/// 2. `begin_inflight(inflight)` — take the slot reservation.
/// 3. `timeout(open_bi)` — bounded wait for a stream.
/// 4. `on_wait_done(elapsed, outcome)` — a zero-cost hook the caller
///    can use to observe wait time / emit metrics.  Always invoked.
pub async fn open_bi_guarded<F>(
    conn: &Connection,
    inflight: &InflightCounter,
    limits: &OverloadLimits,
    stream_timeout: Duration,
    on_wait_done: F,
) -> anyhow::Result<OpenedStream>
where
    F: FnOnce(Duration, OpenBiOutcome),
{
    let counter = inflight.clone();
    maybe_slow_path(move || counter.load(Ordering::Relaxed), limits).await;
    let guard = begin_inflight(inflight);
    let started = Instant::now();
    let result = tokio::time::timeout(stream_timeout, conn.open_bi()).await;
    let elapsed = started.elapsed();
    match result {
        Ok(Ok((send, recv))) => {
            on_wait_done(elapsed, OpenBiOutcome::Ok);
            Ok(OpenedStream {
                send,
                recv,
                inflight: guard,
            })
        }
        Ok(Err(e)) => {
            on_wait_done(elapsed, OpenBiOutcome::ConnectionError);
            Err(e.into())
        }
        Err(_) => {
            on_wait_done(elapsed, OpenBiOutcome::Timeout);
            Err(anyhow!("open_bi timed out after {:?}", stream_timeout))
        }
    }
}
