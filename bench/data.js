window.BENCHMARK_DATA = {
  "lastUpdate": 1772520040640,
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
          "id": "bda37e0e4e865b02b9647b680f776ba2c57ec819",
          "message": "ci: grant contents:write to stress-test job for gh-pages push\n\nCo-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>",
          "timestamp": "2026-03-03T14:17:09+08:00",
          "tree_id": "7d2ef483c4a6859e09b775af2e508122a1386fe3",
          "url": "https://github.com/locustbaby/duotunnel/commit/bda37e0e4e865b02b9647b680f776ba2c57ec819"
        },
        "date": 1772518864433,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "ingress_http_get p50",
            "value": 0.77,
            "unit": "ms"
          },
          {
            "name": "ingress_http_get p99",
            "value": 0,
            "unit": "ms"
          },
          {
            "name": "ingress_http_post p50",
            "value": 0.7,
            "unit": "ms"
          },
          {
            "name": "ingress_http_post p99",
            "value": 0,
            "unit": "ms"
          },
          {
            "name": "egress_http_get p50",
            "value": 0.58,
            "unit": "ms"
          },
          {
            "name": "egress_http_get p99",
            "value": 0,
            "unit": "ms"
          },
          {
            "name": "egress_http_post p50",
            "value": 0.56,
            "unit": "ms"
          },
          {
            "name": "egress_http_post p99",
            "value": 0,
            "unit": "ms"
          },
          {
            "name": "grpc_health_ingress p50",
            "value": 1.54,
            "unit": "ms"
          },
          {
            "name": "grpc_health_ingress p99",
            "value": 0,
            "unit": "ms"
          },
          {
            "name": "grpc_echo_ingress p50",
            "value": 1.54,
            "unit": "ms"
          },
          {
            "name": "grpc_echo_ingress p99",
            "value": 0,
            "unit": "ms"
          },
          {
            "name": "ws_ingress p50",
            "value": 1,
            "unit": "ms"
          },
          {
            "name": "ws_ingress p99",
            "value": 0,
            "unit": "ms"
          },
          {
            "name": "ws_multi_msg p50",
            "value": 44,
            "unit": "ms"
          },
          {
            "name": "ws_multi_msg p99",
            "value": 0,
            "unit": "ms"
          },
          {
            "name": "total_http_rps",
            "value": 1444.4,
            "unit": "req/s"
          },
          {
            "name": "http_error_rate",
            "value": 1.74,
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
          "id": "8eeb20231a37329bb9b9dd5cf32d08eda40bbfea",
          "message": "ci: fix benchmark data — use custom Trend per scenario for reliable p50/p99\n\nk6 handleSummary sub-metrics with {scenario:xxx} tags don't include\npercentiles unless referenced by thresholds. Switch to explicit Trend\nmetrics per scenario using exec.scenario.name to route data, ensuring\nall phases (large body, gRPC, QPS tiers) report p50/p99 correctly.\n\nCo-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>",
          "timestamp": "2026-03-03T14:36:40+08:00",
          "tree_id": "d502a299fe7fa0ee30734597d76ae8656b1d38ac",
          "url": "https://github.com/locustbaby/duotunnel/commit/8eeb20231a37329bb9b9dd5cf32d08eda40bbfea"
        },
        "date": 1772520039793,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "ingress_http_get p50",
            "value": 0.73,
            "unit": "ms"
          },
          {
            "name": "ingress_http_get p99",
            "value": 0,
            "unit": "ms"
          },
          {
            "name": "ingress_http_post p50",
            "value": 0.69,
            "unit": "ms"
          },
          {
            "name": "ingress_http_post p99",
            "value": 0,
            "unit": "ms"
          },
          {
            "name": "egress_http_get p50",
            "value": 0.55,
            "unit": "ms"
          },
          {
            "name": "egress_http_get p99",
            "value": 0,
            "unit": "ms"
          },
          {
            "name": "egress_http_post p50",
            "value": 0.52,
            "unit": "ms"
          },
          {
            "name": "egress_http_post p99",
            "value": 0,
            "unit": "ms"
          },
          {
            "name": "bidir_mixed p50",
            "value": 1,
            "unit": "ms"
          },
          {
            "name": "bidir_mixed p99",
            "value": 0,
            "unit": "ms"
          },
          {
            "name": "ingress_post_1k p50",
            "value": 0.63,
            "unit": "ms"
          },
          {
            "name": "ingress_post_1k p99",
            "value": 0,
            "unit": "ms"
          },
          {
            "name": "ingress_post_10k p50",
            "value": 1.04,
            "unit": "ms"
          },
          {
            "name": "ingress_post_10k p99",
            "value": 0,
            "unit": "ms"
          },
          {
            "name": "ingress_post_100k p50",
            "value": 2.89,
            "unit": "ms"
          },
          {
            "name": "ingress_post_100k p99",
            "value": 0,
            "unit": "ms"
          },
          {
            "name": "egress_post_10k p50",
            "value": 1.05,
            "unit": "ms"
          },
          {
            "name": "egress_post_10k p99",
            "value": 0,
            "unit": "ms"
          },
          {
            "name": "grpc_health p50",
            "value": 2,
            "unit": "ms"
          },
          {
            "name": "grpc_health p99",
            "value": 0,
            "unit": "ms"
          },
          {
            "name": "grpc_echo p50",
            "value": 2,
            "unit": "ms"
          },
          {
            "name": "grpc_echo p99",
            "value": 0,
            "unit": "ms"
          },
          {
            "name": "grpc_large_payload p50",
            "value": 2,
            "unit": "ms"
          },
          {
            "name": "grpc_large_payload p99",
            "value": 0,
            "unit": "ms"
          },
          {
            "name": "grpc_high_qps p50",
            "value": 2,
            "unit": "ms"
          },
          {
            "name": "grpc_high_qps p99",
            "value": 0,
            "unit": "ms"
          },
          {
            "name": "ws_ingress p50",
            "value": 1,
            "unit": "ms"
          },
          {
            "name": "ws_ingress p99",
            "value": 0,
            "unit": "ms"
          },
          {
            "name": "ws_multi_msg p50",
            "value": 44,
            "unit": "ms"
          },
          {
            "name": "ws_multi_msg p99",
            "value": 0,
            "unit": "ms"
          },
          {
            "name": "ingress_1000qps p50",
            "value": 0.72,
            "unit": "ms"
          },
          {
            "name": "ingress_1000qps p99",
            "value": 0,
            "unit": "ms"
          },
          {
            "name": "egress_1000qps p50",
            "value": 0.65,
            "unit": "ms"
          },
          {
            "name": "egress_1000qps p99",
            "value": 0,
            "unit": "ms"
          },
          {
            "name": "ingress_2000qps p50",
            "value": 1.57,
            "unit": "ms"
          },
          {
            "name": "ingress_2000qps p99",
            "value": 0,
            "unit": "ms"
          },
          {
            "name": "egress_2000qps p50",
            "value": 1.45,
            "unit": "ms"
          },
          {
            "name": "egress_2000qps p99",
            "value": 0,
            "unit": "ms"
          },
          {
            "name": "ingress_3000qps p50",
            "value": 27.86,
            "unit": "ms"
          },
          {
            "name": "ingress_3000qps p99",
            "value": 0,
            "unit": "ms"
          },
          {
            "name": "egress_3000qps p50",
            "value": 25.98,
            "unit": "ms"
          },
          {
            "name": "egress_3000qps p99",
            "value": 0,
            "unit": "ms"
          },
          {
            "name": "total_http_rps",
            "value": 1461.2,
            "unit": "req/s"
          },
          {
            "name": "http_error_rate",
            "value": 0.73,
            "unit": "%"
          }
        ]
      }
    ]
  }
}