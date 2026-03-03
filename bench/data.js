window.BENCHMARK_DATA = {
  "lastUpdate": 1772518864864,
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
      }
    ]
  }
}