use crate::error::ProxyError;
use crate::inflight::{InflightCounter, InflightGuard, begin_inflight};
use quinn::{Connection, RecvStream, SendStream};
use std::time::{Duration, Instant};

#[derive(Debug)]
pub enum OpenBiOutcome {
    Ok,
    StreamLimit,
    ConnectionLost,
    ConnectionFatal,
}

pub struct OpenedStream {
    pub send: SendStream,
    pub recv: RecvStream,
    /// RAII guard holding the per-connection inflight slot.  Keep it
    /// alive for the full lifetime of the stream (through `relay_*`)
    /// so `pick_least_inflight` sees accurate load.
    pub inflight: InflightGuard,
}

/// Open a bidirectional QUIC stream with overload-safe inflight accounting.
///
/// Flow:
/// 1. `begin_inflight(inflight)` — take the slot reservation.
/// 2. `timeout(open_bi)` — bounded wait for a stream.
/// 3. `on_wait_done(elapsed, outcome)` — a zero-cost hook the caller
///    can use to observe wait time / emit metrics.  Always invoked.
///
/// The caller is responsible for invoking `crate::maybe_slow_path` before
/// this function when appropriate — doing it here would force the yield to
/// happen after any caller-side work (e.g. draining peeked bytes), which
/// adds a scheduler round-trip to the request's critical path under load.
pub async fn open_bi_guarded<F>(
    conn: &Connection,
    inflight: &InflightCounter,
    stream_timeout: Duration,
    on_wait_done: F,
) -> Result<OpenedStream, ProxyError>
where
    F: FnOnce(Duration, OpenBiOutcome),
{
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
            let (outcome, err) = classify_open_bi_error(e);
            on_wait_done(elapsed, outcome);
            Err(err)
        }
        Err(_) => {
            on_wait_done(elapsed, OpenBiOutcome::StreamLimit);
            Err(ProxyError::quic_stream_limit(stream_timeout))
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
        ConnectionError::VersionMismatch
        | ConnectionError::TransportError(_)
        | ConnectionError::LocallyClosed
        | ConnectionError::CidsExhausted => (
            OpenBiOutcome::ConnectionFatal,
            ProxyError::quic_connection_fatal(error.to_string()),
        ),
    }
}
