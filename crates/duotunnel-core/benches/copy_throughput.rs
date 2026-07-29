use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use duotunnel_core::engine::copy::{copy_buffered_then_finish, copy_buffered_then_shutdown};
use std::sync::Arc;
use tokio::io::{repeat, sink, AsyncReadExt};

async fn run_copy_bench(size: u64, buf_size: usize) {
    let reader = repeat(0).take(size);
    let writer = sink();
    let _ = copy_buffered_then_shutdown(reader, writer, buf_size).await;
}

async fn run_quic_finish_bench(
    client_conn: quinn::Connection,
    server_conn: quinn::Connection,
    size: u64,
    buf_size: usize,
) {
    let reader = repeat(0).take(size);
    let send = client_conn.open_uni().await.unwrap();

    let drain_task = tokio::spawn(async move {
        let mut recv = server_conn.accept_uni().await.unwrap();
        while let Ok(Some(chunk)) = recv.read_chunk(128 * 1024, true).await {
            let _ = chunk;
        }
    });

    let _ = copy_buffered_then_finish(reader, send, buf_size).await;
    let _ = drain_task.await;
}

fn bench_copy_throughput(c: &mut Criterion) {
    let rt = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(4)
        .enable_all()
        .build()
        .unwrap();

    let data_size = 1024 * 1024 * 5; // 5 MiB
    let buf_sizes = [8 * 1024, 64 * 1024]; // 8 KiB, 64 KiB

    let mut group = c.benchmark_group("copy_buffered");
    for &buf_size in &buf_sizes {
        group.bench_with_input(
            BenchmarkId::new(format!("shutdown_buf_{}", buf_size), data_size),
            &buf_size,
            |b, &buf_size| {
                b.to_async(&rt).iter(|| run_copy_bench(data_size, buf_size));
            },
        );
    }

    // Set up a single long-lived QUIC connection pair with a huge stream limit and disabled idle timeout!
    let (client_conn, server_conn, _ce, _se) = rt.block_on(async {
        let (certs, key) = duotunnel_core::infra::pki::generate_self_signed_cert().unwrap();
        let mut server_crypto = rustls::ServerConfig::builder()
            .with_no_client_auth()
            .with_single_cert(certs.clone(), key)
            .unwrap();
        server_crypto.alpn_protocols = vec![b"tunnel-quic/v1".to_vec()];
        let mut server_config = quinn::ServerConfig::with_crypto(Arc::new(
            quinn::crypto::rustls::QuicServerConfig::try_from(server_crypto).unwrap(),
        ));

        let mut transport_config = quinn::TransportConfig::default();
        transport_config.max_concurrent_uni_streams(1_000_000u32.into());
        transport_config.max_idle_timeout(None); // Disable idle timeouts entirely to prevent starvations during heavy benchmarks
        transport_config.stream_receive_window(quinn::VarInt::from_u64(1024 * 1024 * 10).unwrap());
        transport_config.receive_window(quinn::VarInt::from_u64(1024 * 1024 * 100).unwrap());
        transport_config.send_window(1024 * 1024 * 100);

        let transport_config = Arc::new(transport_config);
        server_config.transport_config(transport_config.clone());

        let server_endpoint =
            quinn::Endpoint::server(server_config, "127.0.0.1:0".parse().unwrap()).unwrap();
        let server_addr = server_endpoint.local_addr().unwrap();

        let mut root_store = rustls::RootCertStore::empty();
        for cert in certs {
            root_store.add(cert).unwrap();
        }
        let mut client_crypto = rustls::ClientConfig::builder()
            .with_root_certificates(root_store)
            .with_no_client_auth();
        client_crypto.alpn_protocols = vec![b"tunnel-quic/v1".to_vec()];
        let mut client_config = quinn::ClientConfig::new(Arc::new(
            quinn::crypto::rustls::QuicClientConfig::try_from(client_crypto).unwrap(),
        ));
        client_config.transport_config(transport_config);

        let client_endpoint = quinn::Endpoint::client("127.0.0.1:0".parse().unwrap()).unwrap();

        let connecting = client_endpoint
            .connect_with(client_config, server_addr, "localhost")
            .unwrap();

        let (c, s) = tokio::join!(async { connecting.await.unwrap() }, async {
            server_endpoint.accept().await.unwrap().await.unwrap()
        });
        (c, s, client_endpoint, server_endpoint)
    });

    for &buf_size in &buf_sizes {
        let c_conn = client_conn.clone();
        let s_conn = server_conn.clone();
        group.bench_with_input(
            BenchmarkId::new(format!("quic_finish_buf_{}", buf_size), data_size),
            &buf_size,
            |b, &buf_size| {
                let cc = c_conn.clone();
                let sc = s_conn.clone();
                b.to_async(&rt)
                    .iter(|| run_quic_finish_bench(cc.clone(), sc.clone(), data_size, buf_size));
            },
        );
    }

    group.finish();

    client_conn.close(0u32.into(), b"done");
    server_conn.close(0u32.into(), b"done");
}

criterion_group!(benches, bench_copy_throughput);
criterion_main!(benches);
