window.BENCHMARK_DATA = {
  "entries": [
    {
      "timestamp": "2026-03-03T09:05:01.620Z",
      "commit": {
        "id": "4c733b708ea910013badbc9b49769b4b59ca3429",
        "message": "ci: harden CI \u2014 SHA pins, timeouts, sysstat sampling",
        "url": "https://github.com/locustbaby/duotunnel/commit/4c733b708ea910013badbc9b49769b4b59ca3429"
      },
      "scenarios": [
        {
          "name": "ingress_http_get",
          "protocol": "HTTP",
          "direction": "ingress",
          "category": "basic",
          "p50": 0.6,
          "p95": 1.19,
          "err": 0,
          "rps": 0,
          "requests": 0
        },
        {
          "name": "ingress_http_post",
          "protocol": "HTTP",
          "direction": "ingress",
          "category": "basic",
          "p50": 0.57,
          "p95": 1.3,
          "err": 0,
          "rps": 0,
          "requests": 0
        },
        {
          "name": "egress_http_get",
          "protocol": "HTTP",
          "direction": "egress",
          "category": "basic",
          "p50": 0.48,
          "p95": 0.99,
          "err": 0,
          "rps": 0,
          "requests": 0
        },
        {
          "name": "egress_http_post",
          "protocol": "HTTP",
          "direction": "egress",
          "category": "basic",
          "p50": 0.47,
          "p95": 0.57,
          "err": 0,
          "rps": 0,
          "requests": 0
        },
        {
          "name": "bidir_mixed",
          "protocol": "HTTP",
          "direction": "bidir",
          "category": "basic",
          "p50": 2,
          "p95": 2,
          "err": 0,
          "rps": 0,
          "requests": 0
        },
        {
          "name": "ingress_post_1k",
          "protocol": "HTTP",
          "direction": "ingress",
          "category": "body_size",
          "p50": 0.64,
          "p95": 1.95,
          "err": 0,
          "rps": 0,
          "requests": 0
        },
        {
          "name": "ingress_post_10k",
          "protocol": "HTTP",
          "direction": "ingress",
          "category": "body_size",
          "p50": 1.09,
          "p95": 2.46,
          "err": 0,
          "rps": 0,
          "requests": 0
        },
        {
          "name": "ingress_post_100k",
          "protocol": "HTTP",
          "direction": "ingress",
          "category": "body_size",
          "p50": 2.82,
          "p95": 42.56,
          "err": 0,
          "rps": 0,
          "requests": 0
        },
        {
          "name": "egress_post_10k",
          "protocol": "HTTP",
          "direction": "egress",
          "category": "body_size",
          "p50": 1.01,
          "p95": 2.18,
          "err": 0,
          "rps": 0,
          "requests": 0
        },
        {
          "name": "grpc_health",
          "protocol": "gRPC",
          "direction": "ingress",
          "category": "basic",
          "p50": 2,
          "p95": 42,
          "err": 0,
          "rps": 0,
          "requests": 0
        },
        {
          "name": "grpc_echo",
          "protocol": "gRPC",
          "direction": "ingress",
          "category": "basic",
          "p50": 2,
          "p95": 42,
          "err": 0,
          "rps": 0,
          "requests": 0
        },
        {
          "name": "grpc_large_payload",
          "protocol": "gRPC",
          "direction": "ingress",
          "category": "body_size",
          "p50": 2,
          "p95": 43,
          "err": 0,
          "rps": 0,
          "requests": 0
        },
        {
          "name": "grpc_high_qps",
          "protocol": "gRPC",
          "direction": "ingress",
          "category": "stress",
          "p50": 2,
          "p95": 42,
          "err": 0,
          "rps": 0,
          "requests": 0
        },
        {
          "name": "ws_ingress",
          "protocol": "WS",
          "direction": "ingress",
          "category": "basic",
          "p50": 0,
          "p95": 1,
          "err": 0,
          "rps": 0,
          "requests": 0
        },
        {
          "name": "ws_multi_msg",
          "protocol": "WS",
          "direction": "ingress",
          "category": "basic",
          "p50": 43,
          "p95": 45,
          "err": 0,
          "rps": 0,
          "requests": 0
        },
        {
          "name": "ingress_1000qps",
          "protocol": "HTTP",
          "direction": "ingress",
          "category": "stress",
          "p50": 0.71,
          "p95": 1.18,
          "err": 0,
          "rps": 0,
          "requests": 0
        },
        {
          "name": "egress_1000qps",
          "protocol": "HTTP",
          "direction": "egress",
          "category": "stress",
          "p50": 0.65,
          "p95": 1.06,
          "err": 0,
          "rps": 0,
          "requests": 0
        },
        {
          "name": "ingress_2000qps",
          "protocol": "HTTP",
          "direction": "ingress",
          "category": "stress",
          "p50": 1.33,
          "p95": 6.25,
          "err": 0,
          "rps": 0,
          "requests": 0
        },
        {
          "name": "egress_2000qps",
          "protocol": "HTTP",
          "direction": "egress",
          "category": "stress",
          "p50": 1.21,
          "p95": 5.85,
          "err": 0,
          "rps": 0,
          "requests": 0
        },
        {
          "name": "ingress_3000qps",
          "protocol": "HTTP",
          "direction": "ingress",
          "category": "stress",
          "p50": 19.19,
          "p95": 57.08,
          "err": 0,
          "rps": 0,
          "requests": 0
        },
        {
          "name": "egress_3000qps",
          "protocol": "HTTP",
          "direction": "egress",
          "category": "stress",
          "p50": 19.31,
          "p95": 54.37,
          "err": 0,
          "rps": 0,
          "requests": 0
        }
      ],
      "summary": {
        "totalRPS": 0,
        "totalErr": 0,
        "totalRequests": 0
      }
    }
  ]
};
