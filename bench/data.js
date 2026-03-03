window.BENCHMARK_DATA = {
  "lastUpdate": 1772526251652,
  "repoUrl": "https://github.com/locustbaby/duotunnel",
  "entries": {
    "Benchmark": [
      {
        "commit": {
          "author": {
            "email": "chuanfeng.liu@zilliz.com",
            "name": "Sheldon",
            "username": "locustbaby"
          },
          "committer": {
            "email": "chuanfeng.liu@zilliz.com",
            "name": "Sheldon",
            "username": "locustbaby"
          },
          "distinct": true,
          "id": "573ee9efc4c9ad29ab5b43cd264ab89d086185e0",
          "message": "ci: trigger retry\n\nCo-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>",
          "timestamp": "2026-03-03T15:03:47+08:00",
          "tree_id": "69e7b368294c21d5c9316597938aec3c14b3afc2",
          "url": "https://github.com/locustbaby/duotunnel/commit/573ee9efc4c9ad29ab5b43cd264ab89d086185e0"
        },
        "date": 1772521643995,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "ingress_http_get p50",
            "value": 0.58,
            "unit": "ms"
          },
          {
            "name": "ingress_http_get p95",
            "value": 1.02,
            "unit": "ms"
          },
          {
            "name": "ingress_http_post p50",
            "value": 0.59,
            "unit": "ms"
          },
          {
            "name": "ingress_http_post p95",
            "value": 1.21,
            "unit": "ms"
          },
          {
            "name": "egress_http_get p50",
            "value": 0.49,
            "unit": "ms"
          },
          {
            "name": "egress_http_get p95",
            "value": 0.96,
            "unit": "ms"
          },
          {
            "name": "egress_http_post p50",
            "value": 0.47,
            "unit": "ms"
          },
          {
            "name": "egress_http_post p95",
            "value": 0.57,
            "unit": "ms"
          },
          {
            "name": "bidir_mixed p50",
            "value": 1,
            "unit": "ms"
          },
          {
            "name": "bidir_mixed p95",
            "value": 2,
            "unit": "ms"
          },
          {
            "name": "ingress_post_1k p50",
            "value": 0.61,
            "unit": "ms"
          },
          {
            "name": "ingress_post_1k p95",
            "value": 1.93,
            "unit": "ms"
          },
          {
            "name": "ingress_post_10k p50",
            "value": 1.02,
            "unit": "ms"
          },
          {
            "name": "ingress_post_10k p95",
            "value": 2.42,
            "unit": "ms"
          },
          {
            "name": "ingress_post_100k p50",
            "value": 2.72,
            "unit": "ms"
          },
          {
            "name": "ingress_post_100k p95",
            "value": 42.94,
            "unit": "ms"
          },
          {
            "name": "egress_post_10k p50",
            "value": 0.89,
            "unit": "ms"
          },
          {
            "name": "egress_post_10k p95",
            "value": 41.08,
            "unit": "ms"
          },
          {
            "name": "grpc_health p50",
            "value": 2,
            "unit": "ms"
          },
          {
            "name": "grpc_health p95",
            "value": 42,
            "unit": "ms"
          },
          {
            "name": "grpc_echo p50",
            "value": 2,
            "unit": "ms"
          },
          {
            "name": "grpc_echo p95",
            "value": 43,
            "unit": "ms"
          },
          {
            "name": "grpc_large_payload p50",
            "value": 2,
            "unit": "ms"
          },
          {
            "name": "grpc_large_payload p95",
            "value": 43,
            "unit": "ms"
          },
          {
            "name": "grpc_high_qps p50",
            "value": 2,
            "unit": "ms"
          },
          {
            "name": "grpc_high_qps p95",
            "value": 42,
            "unit": "ms"
          },
          {
            "name": "ws_ingress p50",
            "value": 1,
            "unit": "ms"
          },
          {
            "name": "ws_ingress p95",
            "value": 1,
            "unit": "ms"
          },
          {
            "name": "ws_multi_msg p50",
            "value": 44,
            "unit": "ms"
          },
          {
            "name": "ws_multi_msg p95",
            "value": 45,
            "unit": "ms"
          },
          {
            "name": "ingress_1000qps p50",
            "value": 0.71,
            "unit": "ms"
          },
          {
            "name": "ingress_1000qps p95",
            "value": 1.57,
            "unit": "ms"
          },
          {
            "name": "egress_1000qps p50",
            "value": 0.63,
            "unit": "ms"
          },
          {
            "name": "egress_1000qps p95",
            "value": 1.44,
            "unit": "ms"
          },
          {
            "name": "ingress_2000qps p50",
            "value": 1.55,
            "unit": "ms"
          },
          {
            "name": "ingress_2000qps p95",
            "value": 16.59,
            "unit": "ms"
          },
          {
            "name": "egress_2000qps p50",
            "value": 1.43,
            "unit": "ms"
          },
          {
            "name": "egress_2000qps p95",
            "value": 16.31,
            "unit": "ms"
          },
          {
            "name": "ingress_3000qps p50",
            "value": 34.72,
            "unit": "ms"
          },
          {
            "name": "ingress_3000qps p95",
            "value": 91.69,
            "unit": "ms"
          },
          {
            "name": "egress_3000qps p50",
            "value": 33.23,
            "unit": "ms"
          },
          {
            "name": "egress_3000qps p95",
            "value": 88.12,
            "unit": "ms"
          },
          {
            "name": "total_http_rps",
            "value": 1448.6,
            "unit": "req/s"
          },
          {
            "name": "http_error_rate",
            "value": 1.45,
            "unit": "%"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "chuanfeng.liu@zilliz.com",
            "name": "Sheldon",
            "username": "locustbaby"
          },
          "committer": {
            "email": "chuanfeng.liu@zilliz.com",
            "name": "Sheldon",
            "username": "locustbaby"
          },
          "distinct": true,
          "id": "2feec17be003a0da6511c41fac15ac93f68260da",
          "message": "ci: add k6 stress test with benchmark tracking\n\nMulti-protocol stress test (HTTP/WS/gRPC) across ingress, egress, and\nbidirectional paths. Includes stepped QPS tiers, Prometheus metrics\noutput, gh-pages benchmark charts via github-action-benchmark, and\ngRPC echo server with health/reflection support.\n\nCo-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>",
          "timestamp": "2026-03-03T15:21:28+08:00",
          "tree_id": "69e7b368294c21d5c9316597938aec3c14b3afc2",
          "url": "https://github.com/locustbaby/duotunnel/commit/2feec17be003a0da6511c41fac15ac93f68260da"
        },
        "date": 1772523779571,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "ingress_http_get p50",
            "value": 0.65,
            "unit": "ms"
          },
          {
            "name": "ingress_http_get p95",
            "value": 1.31,
            "unit": "ms"
          },
          {
            "name": "ingress_http_post p50",
            "value": 0.61,
            "unit": "ms"
          },
          {
            "name": "ingress_http_post p95",
            "value": 1.24,
            "unit": "ms"
          },
          {
            "name": "egress_http_get p50",
            "value": 0.53,
            "unit": "ms"
          },
          {
            "name": "egress_http_get p95",
            "value": 1.03,
            "unit": "ms"
          },
          {
            "name": "egress_http_post p50",
            "value": 0.5,
            "unit": "ms"
          },
          {
            "name": "egress_http_post p95",
            "value": 0.72,
            "unit": "ms"
          },
          {
            "name": "bidir_mixed p50",
            "value": 2,
            "unit": "ms"
          },
          {
            "name": "bidir_mixed p95",
            "value": 2,
            "unit": "ms"
          },
          {
            "name": "ingress_post_1k p50",
            "value": 0.72,
            "unit": "ms"
          },
          {
            "name": "ingress_post_1k p95",
            "value": 2.23,
            "unit": "ms"
          },
          {
            "name": "ingress_post_10k p50",
            "value": 1.15,
            "unit": "ms"
          },
          {
            "name": "ingress_post_10k p95",
            "value": 2.32,
            "unit": "ms"
          },
          {
            "name": "ingress_post_100k p50",
            "value": 2.97,
            "unit": "ms"
          },
          {
            "name": "ingress_post_100k p95",
            "value": 43,
            "unit": "ms"
          },
          {
            "name": "egress_post_10k p50",
            "value": 1.1,
            "unit": "ms"
          },
          {
            "name": "egress_post_10k p95",
            "value": 2.47,
            "unit": "ms"
          },
          {
            "name": "grpc_health p50",
            "value": 2,
            "unit": "ms"
          },
          {
            "name": "grpc_health p95",
            "value": 43,
            "unit": "ms"
          },
          {
            "name": "grpc_echo p50",
            "value": 2,
            "unit": "ms"
          },
          {
            "name": "grpc_echo p95",
            "value": 43,
            "unit": "ms"
          },
          {
            "name": "grpc_large_payload p50",
            "value": 2,
            "unit": "ms"
          },
          {
            "name": "grpc_large_payload p95",
            "value": 43,
            "unit": "ms"
          },
          {
            "name": "grpc_high_qps p50",
            "value": 2,
            "unit": "ms"
          },
          {
            "name": "grpc_high_qps p95",
            "value": 42,
            "unit": "ms"
          },
          {
            "name": "ws_ingress p50",
            "value": 0,
            "unit": "ms"
          },
          {
            "name": "ws_ingress p95",
            "value": 1,
            "unit": "ms"
          },
          {
            "name": "ws_multi_msg p50",
            "value": 44,
            "unit": "ms"
          },
          {
            "name": "ws_multi_msg p95",
            "value": 46,
            "unit": "ms"
          },
          {
            "name": "ingress_1000qps p50",
            "value": 0.72,
            "unit": "ms"
          },
          {
            "name": "ingress_1000qps p95",
            "value": 2.44,
            "unit": "ms"
          },
          {
            "name": "egress_1000qps p50",
            "value": 0.65,
            "unit": "ms"
          },
          {
            "name": "egress_1000qps p95",
            "value": 2.25,
            "unit": "ms"
          },
          {
            "name": "ingress_2000qps p50",
            "value": 1.61,
            "unit": "ms"
          },
          {
            "name": "ingress_2000qps p95",
            "value": 16.16,
            "unit": "ms"
          },
          {
            "name": "egress_2000qps p50",
            "value": 1.48,
            "unit": "ms"
          },
          {
            "name": "egress_2000qps p95",
            "value": 16.39,
            "unit": "ms"
          },
          {
            "name": "ingress_3000qps p50",
            "value": 41.57,
            "unit": "ms"
          },
          {
            "name": "ingress_3000qps p95",
            "value": 90.24,
            "unit": "ms"
          },
          {
            "name": "egress_3000qps p50",
            "value": 40.05,
            "unit": "ms"
          },
          {
            "name": "egress_3000qps p95",
            "value": 88.76,
            "unit": "ms"
          },
          {
            "name": "total_http_rps",
            "value": 1409.2,
            "unit": "req/s"
          },
          {
            "name": "http_error_rate",
            "value": 1.33,
            "unit": "%"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "chuanfeng.liu@zilliz.com",
            "name": "Sheldon",
            "username": "locustbaby"
          },
          "committer": {
            "email": "chuanfeng.liu@zilliz.com",
            "name": "Sheldon",
            "username": "locustbaby"
          },
          "distinct": true,
          "id": "142664f7358a0938cf6e007d03eb5caeb09d5f58",
          "message": "ci: add timeout + auto-restart on warmup failure (#2)\n\n- timeout-minutes on all startup/warmup steps to fail fast\n- curl --max-time to prevent indefinite hangs\n- On warmup failure: dump logs, restart client → server, retry\n\nCloses #2\n\nCo-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>",
          "timestamp": "2026-03-03T15:54:35+08:00",
          "tree_id": "3f0039c393a50eebe5cd96d56247178fe6b488ad",
          "url": "https://github.com/locustbaby/duotunnel/commit/142664f7358a0938cf6e007d03eb5caeb09d5f58"
        },
        "date": 1772524707013,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "ingress_http_get p50",
            "value": 0.58,
            "unit": "ms"
          },
          {
            "name": "ingress_http_get p95",
            "value": 1.17,
            "unit": "ms"
          },
          {
            "name": "ingress_http_post p50",
            "value": 0.58,
            "unit": "ms"
          },
          {
            "name": "ingress_http_post p95",
            "value": 1.36,
            "unit": "ms"
          },
          {
            "name": "egress_http_get p50",
            "value": 0.49,
            "unit": "ms"
          },
          {
            "name": "egress_http_get p95",
            "value": 1.02,
            "unit": "ms"
          },
          {
            "name": "egress_http_post p50",
            "value": 0.47,
            "unit": "ms"
          },
          {
            "name": "egress_http_post p95",
            "value": 0.57,
            "unit": "ms"
          },
          {
            "name": "bidir_mixed p50",
            "value": 2,
            "unit": "ms"
          },
          {
            "name": "bidir_mixed p95",
            "value": 2,
            "unit": "ms"
          },
          {
            "name": "ingress_post_1k p50",
            "value": 0.65,
            "unit": "ms"
          },
          {
            "name": "ingress_post_1k p95",
            "value": 2.11,
            "unit": "ms"
          },
          {
            "name": "ingress_post_10k p50",
            "value": 1.05,
            "unit": "ms"
          },
          {
            "name": "ingress_post_10k p95",
            "value": 2.59,
            "unit": "ms"
          },
          {
            "name": "ingress_post_100k p50",
            "value": 2.79,
            "unit": "ms"
          },
          {
            "name": "ingress_post_100k p95",
            "value": 42.84,
            "unit": "ms"
          },
          {
            "name": "egress_post_10k p50",
            "value": 0.96,
            "unit": "ms"
          },
          {
            "name": "egress_post_10k p95",
            "value": 2.65,
            "unit": "ms"
          },
          {
            "name": "grpc_health p50",
            "value": 2,
            "unit": "ms"
          },
          {
            "name": "grpc_health p95",
            "value": 43,
            "unit": "ms"
          },
          {
            "name": "grpc_echo p50",
            "value": 2,
            "unit": "ms"
          },
          {
            "name": "grpc_echo p95",
            "value": 43,
            "unit": "ms"
          },
          {
            "name": "grpc_large_payload p50",
            "value": 2,
            "unit": "ms"
          },
          {
            "name": "grpc_large_payload p95",
            "value": 43,
            "unit": "ms"
          },
          {
            "name": "grpc_high_qps p50",
            "value": 1,
            "unit": "ms"
          },
          {
            "name": "grpc_high_qps p95",
            "value": 42,
            "unit": "ms"
          },
          {
            "name": "ws_ingress p50",
            "value": 1,
            "unit": "ms"
          },
          {
            "name": "ws_ingress p95",
            "value": 1,
            "unit": "ms"
          },
          {
            "name": "ws_multi_msg p50",
            "value": 44,
            "unit": "ms"
          },
          {
            "name": "ws_multi_msg p95",
            "value": 46,
            "unit": "ms"
          },
          {
            "name": "ingress_1000qps p50",
            "value": 0.71,
            "unit": "ms"
          },
          {
            "name": "ingress_1000qps p95",
            "value": 1.78,
            "unit": "ms"
          },
          {
            "name": "egress_1000qps p50",
            "value": 0.65,
            "unit": "ms"
          },
          {
            "name": "egress_1000qps p95",
            "value": 1.71,
            "unit": "ms"
          },
          {
            "name": "ingress_2000qps p50",
            "value": 1.52,
            "unit": "ms"
          },
          {
            "name": "ingress_2000qps p95",
            "value": 14.41,
            "unit": "ms"
          },
          {
            "name": "egress_2000qps p50",
            "value": 1.41,
            "unit": "ms"
          },
          {
            "name": "egress_2000qps p95",
            "value": 14.56,
            "unit": "ms"
          },
          {
            "name": "ingress_3000qps p50",
            "value": 40.81,
            "unit": "ms"
          },
          {
            "name": "ingress_3000qps p95",
            "value": 84.83,
            "unit": "ms"
          },
          {
            "name": "egress_3000qps p50",
            "value": 38.91,
            "unit": "ms"
          },
          {
            "name": "egress_3000qps p95",
            "value": 82.3,
            "unit": "ms"
          },
          {
            "name": "total_http_rps",
            "value": 1436.7,
            "unit": "req/s"
          },
          {
            "name": "http_error_rate",
            "value": 1.17,
            "unit": "%"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "chuanfeng.liu@zilliz.com",
            "name": "Sheldon",
            "username": "locustbaby"
          },
          "committer": {
            "email": "chuanfeng.liu@zilliz.com",
            "name": "Sheldon",
            "username": "locustbaby"
          },
          "distinct": true,
          "id": "00d42051f1188c5b413d0722004757fa35265d6d",
          "message": "ci: only trigger on business code changes\n\nSkip CI for docs, scripts, README, and other non-code changes.\nTriggers on: server/, client/, tunnel-lib/, ci-helpers/, Cargo.*,\nand the workflow file itself.\n\nCo-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>",
          "timestamp": "2026-03-03T15:57:27+08:00",
          "tree_id": "38fbfdc1aa5e2b8e618075011855e4452013834e",
          "url": "https://github.com/locustbaby/duotunnel/commit/00d42051f1188c5b413d0722004757fa35265d6d"
        },
        "date": 1772524903352,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "ingress_http_get p50",
            "value": 0.7,
            "unit": "ms"
          },
          {
            "name": "ingress_http_get p95",
            "value": 1.39,
            "unit": "ms"
          },
          {
            "name": "ingress_http_post p50",
            "value": 0.66,
            "unit": "ms"
          },
          {
            "name": "ingress_http_post p95",
            "value": 1.61,
            "unit": "ms"
          },
          {
            "name": "egress_http_get p50",
            "value": 0.54,
            "unit": "ms"
          },
          {
            "name": "egress_http_get p95",
            "value": 1.12,
            "unit": "ms"
          },
          {
            "name": "egress_http_post p50",
            "value": 0.55,
            "unit": "ms"
          },
          {
            "name": "egress_http_post p95",
            "value": 0.75,
            "unit": "ms"
          },
          {
            "name": "bidir_mixed p50",
            "value": 1,
            "unit": "ms"
          },
          {
            "name": "bidir_mixed p95",
            "value": 2,
            "unit": "ms"
          },
          {
            "name": "ingress_post_1k p50",
            "value": 0.72,
            "unit": "ms"
          },
          {
            "name": "ingress_post_1k p95",
            "value": 2.36,
            "unit": "ms"
          },
          {
            "name": "ingress_post_10k p50",
            "value": 1.17,
            "unit": "ms"
          },
          {
            "name": "ingress_post_10k p95",
            "value": 2.39,
            "unit": "ms"
          },
          {
            "name": "ingress_post_100k p50",
            "value": 2.96,
            "unit": "ms"
          },
          {
            "name": "ingress_post_100k p95",
            "value": 42.94,
            "unit": "ms"
          },
          {
            "name": "egress_post_10k p50",
            "value": 1.1,
            "unit": "ms"
          },
          {
            "name": "egress_post_10k p95",
            "value": 2.37,
            "unit": "ms"
          },
          {
            "name": "grpc_health p50",
            "value": 2,
            "unit": "ms"
          },
          {
            "name": "grpc_health p95",
            "value": 43,
            "unit": "ms"
          },
          {
            "name": "grpc_echo p50",
            "value": 2,
            "unit": "ms"
          },
          {
            "name": "grpc_echo p95",
            "value": 43,
            "unit": "ms"
          },
          {
            "name": "grpc_large_payload p50",
            "value": 2,
            "unit": "ms"
          },
          {
            "name": "grpc_large_payload p95",
            "value": 43,
            "unit": "ms"
          },
          {
            "name": "grpc_high_qps p50",
            "value": 2,
            "unit": "ms"
          },
          {
            "name": "grpc_high_qps p95",
            "value": 42,
            "unit": "ms"
          },
          {
            "name": "ws_ingress p50",
            "value": 0,
            "unit": "ms"
          },
          {
            "name": "ws_ingress p95",
            "value": 1,
            "unit": "ms"
          },
          {
            "name": "ws_multi_msg p50",
            "value": 44,
            "unit": "ms"
          },
          {
            "name": "ws_multi_msg p95",
            "value": 45,
            "unit": "ms"
          },
          {
            "name": "ingress_1000qps p50",
            "value": 0.72,
            "unit": "ms"
          },
          {
            "name": "ingress_1000qps p95",
            "value": 1.95,
            "unit": "ms"
          },
          {
            "name": "egress_1000qps p50",
            "value": 0.64,
            "unit": "ms"
          },
          {
            "name": "egress_1000qps p95",
            "value": 1.79,
            "unit": "ms"
          },
          {
            "name": "ingress_2000qps p50",
            "value": 1.58,
            "unit": "ms"
          },
          {
            "name": "ingress_2000qps p95",
            "value": 21.26,
            "unit": "ms"
          },
          {
            "name": "egress_2000qps p50",
            "value": 1.47,
            "unit": "ms"
          },
          {
            "name": "egress_2000qps p95",
            "value": 21.51,
            "unit": "ms"
          },
          {
            "name": "ingress_3000qps p50",
            "value": 46.63,
            "unit": "ms"
          },
          {
            "name": "ingress_3000qps p95",
            "value": 106.92,
            "unit": "ms"
          },
          {
            "name": "egress_3000qps p50",
            "value": 40.22,
            "unit": "ms"
          },
          {
            "name": "egress_3000qps p95",
            "value": 99.51,
            "unit": "ms"
          },
          {
            "name": "total_http_rps",
            "value": 1405.5,
            "unit": "req/s"
          },
          {
            "name": "http_error_rate",
            "value": 2.19,
            "unit": "%"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "chuanfeng.liu@zilliz.com",
            "name": "Sheldon",
            "username": "locustbaby"
          },
          "committer": {
            "email": "chuanfeng.liu@zilliz.com",
            "name": "Sheldon",
            "username": "locustbaby"
          },
          "distinct": true,
          "id": "26e9822fa7f4b22d6c0110910140614f5ba4c37a",
          "message": "ci: fix action pins — use full 40-char SHA\n\nGitHub Actions requires full commit SHA, not abbreviated.\n\nCo-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>",
          "timestamp": "2026-03-03T16:20:18+08:00",
          "tree_id": "dc09cf9e1bcb5e209b268c4a5e666d56f572a9c1",
          "url": "https://github.com/locustbaby/duotunnel/commit/26e9822fa7f4b22d6c0110910140614f5ba4c37a"
        },
        "date": 1772526251072,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "ingress_http_get p50",
            "value": 0.72,
            "unit": "ms"
          },
          {
            "name": "ingress_http_get p95",
            "value": 1.47,
            "unit": "ms"
          },
          {
            "name": "ingress_http_post p50",
            "value": 0.68,
            "unit": "ms"
          },
          {
            "name": "ingress_http_post p95",
            "value": 1.51,
            "unit": "ms"
          },
          {
            "name": "egress_http_get p50",
            "value": 0.55,
            "unit": "ms"
          },
          {
            "name": "egress_http_get p95",
            "value": 1.12,
            "unit": "ms"
          },
          {
            "name": "egress_http_post p50",
            "value": 0.54,
            "unit": "ms"
          },
          {
            "name": "egress_http_post p95",
            "value": 0.71,
            "unit": "ms"
          },
          {
            "name": "bidir_mixed p50",
            "value": 2,
            "unit": "ms"
          },
          {
            "name": "bidir_mixed p95",
            "value": 2,
            "unit": "ms"
          },
          {
            "name": "ingress_post_1k p50",
            "value": 0.7,
            "unit": "ms"
          },
          {
            "name": "ingress_post_1k p95",
            "value": 2.01,
            "unit": "ms"
          },
          {
            "name": "ingress_post_10k p50",
            "value": 1.14,
            "unit": "ms"
          },
          {
            "name": "ingress_post_10k p95",
            "value": 2.4,
            "unit": "ms"
          },
          {
            "name": "ingress_post_100k p50",
            "value": 2.97,
            "unit": "ms"
          },
          {
            "name": "ingress_post_100k p95",
            "value": 42.88,
            "unit": "ms"
          },
          {
            "name": "egress_post_10k p50",
            "value": 1.08,
            "unit": "ms"
          },
          {
            "name": "egress_post_10k p95",
            "value": 4.89,
            "unit": "ms"
          },
          {
            "name": "grpc_health p50",
            "value": 2,
            "unit": "ms"
          },
          {
            "name": "grpc_health p95",
            "value": 43,
            "unit": "ms"
          },
          {
            "name": "grpc_echo p50",
            "value": 2,
            "unit": "ms"
          },
          {
            "name": "grpc_echo p95",
            "value": 43,
            "unit": "ms"
          },
          {
            "name": "grpc_large_payload p50",
            "value": 2,
            "unit": "ms"
          },
          {
            "name": "grpc_large_payload p95",
            "value": 43,
            "unit": "ms"
          },
          {
            "name": "grpc_high_qps p50",
            "value": 2,
            "unit": "ms"
          },
          {
            "name": "grpc_high_qps p95",
            "value": 43,
            "unit": "ms"
          },
          {
            "name": "ws_ingress p50",
            "value": 0,
            "unit": "ms"
          },
          {
            "name": "ws_ingress p95",
            "value": 1,
            "unit": "ms"
          },
          {
            "name": "ws_multi_msg p50",
            "value": 44,
            "unit": "ms"
          },
          {
            "name": "ws_multi_msg p95",
            "value": 45,
            "unit": "ms"
          },
          {
            "name": "ingress_1000qps p50",
            "value": 0.71,
            "unit": "ms"
          },
          {
            "name": "ingress_1000qps p95",
            "value": 1.86,
            "unit": "ms"
          },
          {
            "name": "egress_1000qps p50",
            "value": 0.64,
            "unit": "ms"
          },
          {
            "name": "egress_1000qps p95",
            "value": 1.67,
            "unit": "ms"
          },
          {
            "name": "ingress_2000qps p50",
            "value": 1.68,
            "unit": "ms"
          },
          {
            "name": "ingress_2000qps p95",
            "value": 34.21,
            "unit": "ms"
          },
          {
            "name": "egress_2000qps p50",
            "value": 1.57,
            "unit": "ms"
          },
          {
            "name": "egress_2000qps p95",
            "value": 34.75,
            "unit": "ms"
          },
          {
            "name": "ingress_3000qps p50",
            "value": 39.68,
            "unit": "ms"
          },
          {
            "name": "ingress_3000qps p95",
            "value": 87.84,
            "unit": "ms"
          },
          {
            "name": "egress_3000qps p50",
            "value": 38.93,
            "unit": "ms"
          },
          {
            "name": "egress_3000qps p95",
            "value": 85.72,
            "unit": "ms"
          },
          {
            "name": "total_http_rps",
            "value": 1433.6,
            "unit": "req/s"
          },
          {
            "name": "http_error_rate",
            "value": 1.47,
            "unit": "%"
          }
        ]
      }
    ]
  }
}