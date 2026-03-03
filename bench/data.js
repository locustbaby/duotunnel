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
    },
    {
      "timestamp": "2026-03-03T09:35:44.572Z",
      "commit": {
        "id": "0d51f189516bbc95b1870fc6f1f800a9caf6909b",
        "message": "ci: harden CI \u2014 SHA pins, timeouts, sysstat sampling",
        "url": "https://github.com/locustbaby/duotunnel/commit/0d51f189516bbc95b1870fc6f1f800a9caf6909b"
      },
      "scenarios": [
        {
          "name": "ingress_http_get",
          "protocol": "HTTP",
          "direction": "ingress",
          "category": "basic",
          "p50": 0.64,
          "p95": 1.36,
          "p99": null,
          "err": 0,
          "rps": 6.3,
          "requests": 725
        },
        {
          "name": "ingress_http_post",
          "protocol": "HTTP",
          "direction": "ingress",
          "category": "basic",
          "p50": 0.6,
          "p95": 1.3,
          "p99": null,
          "err": 0,
          "rps": 3.91,
          "requests": 450
        },
        {
          "name": "egress_http_get",
          "protocol": "HTTP",
          "direction": "egress",
          "category": "basic",
          "p50": 0.51,
          "p95": 1.06,
          "p99": null,
          "err": 0,
          "rps": 7.82,
          "requests": 899
        },
        {
          "name": "egress_http_post",
          "protocol": "HTTP",
          "direction": "egress",
          "category": "basic",
          "p50": 0.5,
          "p95": 0.64,
          "p99": null,
          "err": 0,
          "rps": 6.19,
          "requests": 712
        },
        {
          "name": "bidir_mixed",
          "protocol": "HTTP",
          "direction": "bidir",
          "category": "basic",
          "p50": 1,
          "p95": 2,
          "p99": null,
          "err": 0,
          "rps": 2.28,
          "requests": 262
        },
        {
          "name": "ingress_post_1k",
          "protocol": "HTTP",
          "direction": "ingress",
          "category": "body_size",
          "p50": 0.69,
          "p95": 1.99,
          "p99": null,
          "err": 0,
          "rps": 3.92,
          "requests": 451
        },
        {
          "name": "ingress_post_10k",
          "protocol": "HTTP",
          "direction": "ingress",
          "category": "body_size",
          "p50": 1.11,
          "p95": 2.81,
          "p99": null,
          "err": 0,
          "rps": 2.62,
          "requests": 301
        },
        {
          "name": "ingress_post_100k",
          "protocol": "HTTP",
          "direction": "ingress",
          "category": "body_size",
          "p50": 2.94,
          "p95": 43,
          "p99": null,
          "err": 0,
          "rps": 1.3,
          "requests": 150
        },
        {
          "name": "egress_post_10k",
          "protocol": "HTTP",
          "direction": "egress",
          "category": "body_size",
          "p50": 1.09,
          "p95": 2.59,
          "p99": null,
          "err": 0,
          "rps": 2.61,
          "requests": 300
        },
        {
          "name": "grpc_health_ingress",
          "protocol": "gRPC",
          "direction": "ingress",
          "category": "basic",
          "p50": 2,
          "p95": 43,
          "p99": null,
          "err": 0,
          "rps": 2.61,
          "requests": 300
        },
        {
          "name": "grpc_echo_ingress",
          "protocol": "gRPC",
          "direction": "ingress",
          "category": "basic",
          "p50": 2,
          "p95": 43,
          "p99": null,
          "err": 0,
          "rps": 2.61,
          "requests": 300
        },
        {
          "name": "grpc_large_payload",
          "protocol": "gRPC",
          "direction": "ingress",
          "category": "body_size",
          "p50": 2,
          "p95": 43,
          "p99": null,
          "err": 0,
          "rps": 1.96,
          "requests": 226
        },
        {
          "name": "grpc_high_qps",
          "protocol": "gRPC",
          "direction": "ingress",
          "category": "stress",
          "p50": 2,
          "p95": 42,
          "p99": null,
          "err": 0,
          "rps": 11.3,
          "requests": 1300
        },
        {
          "name": "ws_ingress",
          "protocol": "WS",
          "direction": "ingress",
          "category": "basic",
          "p50": 1,
          "p95": 1,
          "p99": null,
          "err": 0,
          "rps": 1.31,
          "requests": 151
        },
        {
          "name": "ws_multi_msg",
          "protocol": "WS",
          "direction": "ingress",
          "category": "basic",
          "p50": 44,
          "p95": 45,
          "p99": null,
          "err": 0,
          "rps": 0.66,
          "requests": 76
        },
        {
          "name": "ingress_1000qps",
          "protocol": "HTTP",
          "direction": "ingress",
          "category": "stress",
          "p50": 0.7,
          "p95": 1.2,
          "p99": null,
          "err": 0,
          "rps": 130.4,
          "requests": 15000
        },
        {
          "name": "egress_1000qps",
          "protocol": "HTTP",
          "direction": "egress",
          "category": "stress",
          "p50": 0.63,
          "p95": 1.05,
          "p99": null,
          "err": 0,
          "rps": 130.4,
          "requests": 15001
        },
        {
          "name": "ingress_2000qps",
          "protocol": "HTTP",
          "direction": "ingress",
          "category": "stress",
          "p50": 1.39,
          "p95": 8.31,
          "p99": null,
          "err": 0,
          "rps": 260.2,
          "requests": 29932
        },
        {
          "name": "egress_2000qps",
          "protocol": "HTTP",
          "direction": "egress",
          "category": "stress",
          "p50": 1.28,
          "p95": 7.99,
          "p99": null,
          "err": 0,
          "rps": 260.27,
          "requests": 29940
        },
        {
          "name": "ingress_3000qps",
          "protocol": "HTTP",
          "direction": "ingress",
          "category": "stress",
          "p50": 22.44,
          "p95": 59.63,
          "p99": null,
          "err": 0,
          "rps": 346.97,
          "requests": 39914
        },
        {
          "name": "egress_3000qps",
          "protocol": "HTTP",
          "direction": "egress",
          "category": "stress",
          "p50": 22.19,
          "p95": 61.58,
          "p99": null,
          "err": 0.54,
          "rps": 339.9,
          "requests": 39100
        }
      ],
      "summary": {
        "totalRPS": 1525.54,
        "totalErr": 0.12,
        "totalRequests": 175490
      },
      "scenarioConfig": {
        "getFullExecutionRequirements": {},
        "getSortedConfigs": {},
        "unmarshalJSON": {},
        "validate": {}
      },
      "resources": {
        "server": {
          "cpu": [
            {
              "t": 0.0,
              "v": 1.0
            },
            {
              "t": 4.0,
              "v": 1.5
            },
            {
              "t": 8.0,
              "v": 4.5
            },
            {
              "t": 12.0,
              "v": 7.5
            },
            {
              "t": 16.0,
              "v": 9.5
            },
            {
              "t": 20.0,
              "v": 10.0
            },
            {
              "t": 24.0,
              "v": 11.0
            },
            {
              "t": 28.0,
              "v": 10.5
            },
            {
              "t": 32.0,
              "v": 10.5
            },
            {
              "t": 36.0,
              "v": 10.0
            },
            {
              "t": 40.0,
              "v": 7.5
            },
            {
              "t": 44.0,
              "v": 5.0
            },
            {
              "t": 48.0,
              "v": 1.5
            },
            {
              "t": 52.0,
              "v": 0.5
            },
            {
              "t": 56.0,
              "v": 0.0
            },
            {
              "t": 60.0,
              "v": 3.0
            },
            {
              "t": 64.0,
              "v": 5.0
            },
            {
              "t": 68.0,
              "v": 9.0
            },
            {
              "t": 72.0,
              "v": 11.5
            },
            {
              "t": 76.0,
              "v": 14.5
            },
            {
              "t": 80.0,
              "v": 14.5
            },
            {
              "t": 84.0,
              "v": 14.0
            },
            {
              "t": 88.0,
              "v": 14.0
            },
            {
              "t": 92.0,
              "v": 10.5
            },
            {
              "t": 96.0,
              "v": 7.0
            },
            {
              "t": 100.0,
              "v": 0.0
            },
            {
              "t": 104.0,
              "v": 0.0
            },
            {
              "t": 108.0,
              "v": 0.0
            },
            {
              "t": 112.0,
              "v": 0.0
            },
            {
              "t": 116.0,
              "v": 0.0
            },
            {
              "t": 120.0,
              "v": 51.0
            },
            {
              "t": 124.0,
              "v": 63.0
            },
            {
              "t": 128.0,
              "v": 62.0
            },
            {
              "t": 132.0,
              "v": 62.0
            },
            {
              "t": 136.0,
              "v": 63.0
            },
            {
              "t": 140.0,
              "v": 62.5
            },
            {
              "t": 144.0,
              "v": 62.0
            },
            {
              "t": 148.0,
              "v": 43.0
            },
            {
              "t": 152.0,
              "v": 0.0
            },
            {
              "t": 156.0,
              "v": 0.0
            },
            {
              "t": 160.0,
              "v": 76.12
            },
            {
              "t": 164.0,
              "v": 91.0
            },
            {
              "t": 168.0,
              "v": 92.0
            },
            {
              "t": 172.0,
              "v": 90.0
            },
            {
              "t": 176.0,
              "v": 90.5
            },
            {
              "t": 180.0,
              "v": 92.5
            },
            {
              "t": 184.0,
              "v": 91.0
            },
            {
              "t": 188.0,
              "v": 62.5
            },
            {
              "t": 192.0,
              "v": 0.0
            },
            {
              "t": 196.0,
              "v": 0.0
            },
            {
              "t": 200.0,
              "v": 66.5
            },
            {
              "t": 204.0,
              "v": 80.0
            },
            {
              "t": 208.0,
              "v": 82.5
            },
            {
              "t": 212.0,
              "v": 80.5
            },
            {
              "t": 216.0,
              "v": 80.0
            },
            {
              "t": 220.0,
              "v": 70.0
            },
            {
              "t": 224.0,
              "v": 75.0
            }
          ],
          "rss": [
            {
              "t": 2.0,
              "v": 34.6
            },
            {
              "t": 6.0,
              "v": 34.6
            },
            {
              "t": 10.0,
              "v": 35.5
            },
            {
              "t": 14.0,
              "v": 35.5
            },
            {
              "t": 18.0,
              "v": 35.5
            },
            {
              "t": 22.0,
              "v": 35.6
            },
            {
              "t": 26.0,
              "v": 35.2
            },
            {
              "t": 30.0,
              "v": 34.4
            },
            {
              "t": 34.0,
              "v": 34.4
            },
            {
              "t": 38.0,
              "v": 34.1
            },
            {
              "t": 42.0,
              "v": 34.9
            },
            {
              "t": 46.0,
              "v": 35.0
            },
            {
              "t": 50.0,
              "v": 35.0
            },
            {
              "t": 54.0,
              "v": 35.0
            },
            {
              "t": 58.0,
              "v": 35.0
            },
            {
              "t": 62.0,
              "v": 34.9
            },
            {
              "t": 66.0,
              "v": 34.6
            },
            {
              "t": 70.0,
              "v": 34.7
            },
            {
              "t": 74.0,
              "v": 34.8
            },
            {
              "t": 78.0,
              "v": 34.7
            },
            {
              "t": 82.0,
              "v": 34.2
            },
            {
              "t": 86.0,
              "v": 34.1
            },
            {
              "t": 90.0,
              "v": 33.8
            },
            {
              "t": 94.0,
              "v": 33.8
            },
            {
              "t": 98.0,
              "v": 33.8
            },
            {
              "t": 102.0,
              "v": 33.8
            },
            {
              "t": 106.0,
              "v": 33.8
            },
            {
              "t": 110.0,
              "v": 33.8
            },
            {
              "t": 114.0,
              "v": 33.8
            },
            {
              "t": 118.0,
              "v": 33.8
            },
            {
              "t": 122.0,
              "v": 32.8
            },
            {
              "t": 126.0,
              "v": 32.8
            },
            {
              "t": 130.0,
              "v": 33.1
            },
            {
              "t": 134.0,
              "v": 33.3
            },
            {
              "t": 138.0,
              "v": 33.2
            },
            {
              "t": 142.0,
              "v": 33.2
            },
            {
              "t": 146.0,
              "v": 33.3
            },
            {
              "t": 150.0,
              "v": 33.0
            },
            {
              "t": 154.0,
              "v": 33.0
            },
            {
              "t": 158.0,
              "v": 33.0
            },
            {
              "t": 162.0,
              "v": 38.2
            },
            {
              "t": 166.0,
              "v": 38.2
            },
            {
              "t": 170.0,
              "v": 37.9
            },
            {
              "t": 174.0,
              "v": 38.0
            },
            {
              "t": 178.0,
              "v": 38.4
            },
            {
              "t": 182.0,
              "v": 37.6
            },
            {
              "t": 186.0,
              "v": 37.0
            },
            {
              "t": 190.0,
              "v": 36.9
            },
            {
              "t": 194.0,
              "v": 36.9
            },
            {
              "t": 198.0,
              "v": 36.9
            },
            {
              "t": 202.0,
              "v": 42.2
            },
            {
              "t": 206.0,
              "v": 43.2
            },
            {
              "t": 210.0,
              "v": 43.0
            },
            {
              "t": 214.0,
              "v": 41.3
            },
            {
              "t": 218.0,
              "v": 50.1
            },
            {
              "t": 222.0,
              "v": 50.7
            },
            {
              "t": 226.0,
              "v": 49.5
            }
          ]
        },
        "client": {
          "cpu": [
            {
              "t": 0.0,
              "v": 1.0
            },
            {
              "t": 4.0,
              "v": 2.0
            },
            {
              "t": 8.0,
              "v": 4.5
            },
            {
              "t": 12.0,
              "v": 6.0
            },
            {
              "t": 16.0,
              "v": 8.5
            },
            {
              "t": 20.0,
              "v": 9.5
            },
            {
              "t": 24.0,
              "v": 10.5
            },
            {
              "t": 28.0,
              "v": 10.0
            },
            {
              "t": 32.0,
              "v": 10.0
            },
            {
              "t": 36.0,
              "v": 9.0
            },
            {
              "t": 40.0,
              "v": 7.5
            },
            {
              "t": 44.0,
              "v": 4.0
            },
            {
              "t": 48.0,
              "v": 1.5
            },
            {
              "t": 52.0,
              "v": 0.0
            },
            {
              "t": 56.0,
              "v": 0.0
            },
            {
              "t": 60.0,
              "v": 3.0
            },
            {
              "t": 64.0,
              "v": 5.0
            },
            {
              "t": 68.0,
              "v": 8.5
            },
            {
              "t": 72.0,
              "v": 11.5
            },
            {
              "t": 76.0,
              "v": 13.5
            },
            {
              "t": 80.0,
              "v": 13.5
            },
            {
              "t": 84.0,
              "v": 14.0
            },
            {
              "t": 88.0,
              "v": 13.0
            },
            {
              "t": 92.0,
              "v": 10.5
            },
            {
              "t": 96.0,
              "v": 6.5
            },
            {
              "t": 100.0,
              "v": 0.0
            },
            {
              "t": 104.0,
              "v": 0.0
            },
            {
              "t": 108.0,
              "v": 0.0
            },
            {
              "t": 112.0,
              "v": 0.0
            },
            {
              "t": 116.0,
              "v": 0.0
            },
            {
              "t": 120.0,
              "v": 50.5
            },
            {
              "t": 124.0,
              "v": 62.0
            },
            {
              "t": 128.0,
              "v": 62.0
            },
            {
              "t": 132.0,
              "v": 61.5
            },
            {
              "t": 136.0,
              "v": 62.5
            },
            {
              "t": 140.0,
              "v": 62.5
            },
            {
              "t": 144.0,
              "v": 62.0
            },
            {
              "t": 148.0,
              "v": 42.0
            },
            {
              "t": 152.0,
              "v": 0.0
            },
            {
              "t": 156.0,
              "v": 0.0
            },
            {
              "t": 160.0,
              "v": 73.63
            },
            {
              "t": 164.0,
              "v": 89.5
            },
            {
              "t": 168.0,
              "v": 90.5
            },
            {
              "t": 172.0,
              "v": 87.5
            },
            {
              "t": 176.0,
              "v": 88.5
            },
            {
              "t": 180.0,
              "v": 89.5
            },
            {
              "t": 184.0,
              "v": 88.5
            },
            {
              "t": 188.0,
              "v": 61.5
            },
            {
              "t": 192.0,
              "v": 0.0
            },
            {
              "t": 196.0,
              "v": 0.0
            },
            {
              "t": 200.0,
              "v": 60.5
            },
            {
              "t": 204.0,
              "v": 72.5
            },
            {
              "t": 208.0,
              "v": 74.5
            },
            {
              "t": 212.0,
              "v": 72.5
            },
            {
              "t": 216.0,
              "v": 73.0
            },
            {
              "t": 220.0,
              "v": 63.5
            },
            {
              "t": 224.0,
              "v": 69.5
            }
          ],
          "rss": [
            {
              "t": 2.0,
              "v": 29.5
            },
            {
              "t": 6.0,
              "v": 29.5
            },
            {
              "t": 10.0,
              "v": 30.0
            },
            {
              "t": 14.0,
              "v": 32.0
            },
            {
              "t": 18.0,
              "v": 32.2
            },
            {
              "t": 22.0,
              "v": 32.2
            },
            {
              "t": 26.0,
              "v": 31.6
            },
            {
              "t": 30.0,
              "v": 31.5
            },
            {
              "t": 34.0,
              "v": 31.3
            },
            {
              "t": 38.0,
              "v": 30.7
            },
            {
              "t": 42.0,
              "v": 30.6
            },
            {
              "t": 46.0,
              "v": 30.3
            },
            {
              "t": 50.0,
              "v": 30.3
            },
            {
              "t": 54.0,
              "v": 30.3
            },
            {
              "t": 58.0,
              "v": 30.3
            },
            {
              "t": 62.0,
              "v": 30.9
            },
            {
              "t": 66.0,
              "v": 31.0
            },
            {
              "t": 70.0,
              "v": 29.9
            },
            {
              "t": 74.0,
              "v": 32.7
            },
            {
              "t": 78.0,
              "v": 32.7
            },
            {
              "t": 82.0,
              "v": 33.7
            },
            {
              "t": 86.0,
              "v": 33.1
            },
            {
              "t": 90.0,
              "v": 33.5
            },
            {
              "t": 94.0,
              "v": 32.4
            },
            {
              "t": 98.0,
              "v": 32.4
            },
            {
              "t": 102.0,
              "v": 32.4
            },
            {
              "t": 106.0,
              "v": 32.4
            },
            {
              "t": 110.0,
              "v": 32.4
            },
            {
              "t": 114.0,
              "v": 32.4
            },
            {
              "t": 118.0,
              "v": 32.4
            },
            {
              "t": 122.0,
              "v": 31.8
            },
            {
              "t": 126.0,
              "v": 30.6
            },
            {
              "t": 130.0,
              "v": 30.7
            },
            {
              "t": 134.0,
              "v": 30.5
            },
            {
              "t": 138.0,
              "v": 30.9
            },
            {
              "t": 142.0,
              "v": 30.3
            },
            {
              "t": 146.0,
              "v": 30.4
            },
            {
              "t": 150.0,
              "v": 30.3
            },
            {
              "t": 154.0,
              "v": 30.3
            },
            {
              "t": 158.0,
              "v": 30.3
            },
            {
              "t": 162.0,
              "v": 31.3
            },
            {
              "t": 166.0,
              "v": 32.0
            },
            {
              "t": 170.0,
              "v": 31.4
            },
            {
              "t": 174.0,
              "v": 31.4
            },
            {
              "t": 178.0,
              "v": 31.8
            },
            {
              "t": 182.0,
              "v": 32.0
            },
            {
              "t": 186.0,
              "v": 32.1
            },
            {
              "t": 190.0,
              "v": 31.8
            },
            {
              "t": 194.0,
              "v": 31.8
            },
            {
              "t": 198.0,
              "v": 31.8
            },
            {
              "t": 202.0,
              "v": 37.0
            },
            {
              "t": 206.0,
              "v": 36.3
            },
            {
              "t": 210.0,
              "v": 36.2
            },
            {
              "t": 214.0,
              "v": 38.6
            },
            {
              "t": 218.0,
              "v": 37.3
            },
            {
              "t": 222.0,
              "v": 39.2
            },
            {
              "t": 226.0,
              "v": 40.1
            }
          ]
        },
        "system": {
          "cpu": [
            {
              "t": 0.0,
              "v": 36.3
            },
            {
              "t": 2.0,
              "v": 26.1
            },
            {
              "t": 4.0,
              "v": 28.3
            },
            {
              "t": 6.0,
              "v": 30.8
            },
            {
              "t": 8.0,
              "v": 33.3
            },
            {
              "t": 10.0,
              "v": 33.1
            },
            {
              "t": 12.0,
              "v": 16.1
            },
            {
              "t": 14.0,
              "v": 10.0
            },
            {
              "t": 16.0,
              "v": 10.2
            },
            {
              "t": 18.0,
              "v": 8.7
            },
            {
              "t": 20.0,
              "v": 7.0
            },
            {
              "t": 22.0,
              "v": 5.2
            },
            {
              "t": 24.0,
              "v": 1.6
            },
            {
              "t": 26.0,
              "v": 1.5
            },
            {
              "t": 28.0,
              "v": 0.8
            },
            {
              "t": 30.0,
              "v": 2.7
            },
            {
              "t": 32.0,
              "v": 4.7
            },
            {
              "t": 34.0,
              "v": 9.4
            },
            {
              "t": 36.0,
              "v": 10.7
            },
            {
              "t": 38.0,
              "v": 13.5
            },
            {
              "t": 40.0,
              "v": 14.2
            },
            {
              "t": 42.0,
              "v": 13.8
            },
            {
              "t": 44.0,
              "v": 13.7
            },
            {
              "t": 46.0,
              "v": 10.2
            },
            {
              "t": 48.0,
              "v": 5.9
            },
            {
              "t": 50.0,
              "v": 0.2
            },
            {
              "t": 52.0,
              "v": 0.1
            },
            {
              "t": 54.0,
              "v": 0.2
            },
            {
              "t": 56.0,
              "v": 0.1
            },
            {
              "t": 58.0,
              "v": 1.4
            },
            {
              "t": 60.0,
              "v": 48.9
            },
            {
              "t": 62.0,
              "v": 61.5
            },
            {
              "t": 64.0,
              "v": 61.5
            },
            {
              "t": 66.0,
              "v": 61.9
            },
            {
              "t": 68.0,
              "v": 61.3
            },
            {
              "t": 70.0,
              "v": 62.0
            },
            {
              "t": 72.0,
              "v": 62.5
            },
            {
              "t": 74.0,
              "v": 42.3
            },
            {
              "t": 76.0,
              "v": 0.4
            },
            {
              "t": 78.0,
              "v": 0.2
            },
            {
              "t": 80.0,
              "v": 77.4
            },
            {
              "t": 82.0,
              "v": 95.2
            },
            {
              "t": 84.0,
              "v": 95.2
            },
            {
              "t": 86.0,
              "v": 95.1
            },
            {
              "t": 88.0,
              "v": 95.8
            },
            {
              "t": 90.0,
              "v": 95.2
            },
            {
              "t": 92.0,
              "v": 95.5
            },
            {
              "t": 94.0,
              "v": 65.2
            },
            {
              "t": 96.0,
              "v": 0.2
            },
            {
              "t": 98.0,
              "v": 18.3
            },
            {
              "t": 100.0,
              "v": 85.3
            },
            {
              "t": 102.0,
              "v": 99.8
            },
            {
              "t": 104.0,
              "v": 99.8
            },
            {
              "t": 106.0,
              "v": 99.8
            },
            {
              "t": 108.0,
              "v": 100.0
            },
            {
              "t": 110.0,
              "v": 100.0
            },
            {
              "t": 112.0,
              "v": 100.0
            }
          ]
        }
      }
    },
    {
      "timestamp": "2026-03-03T09:45:42.203Z",
      "commit": {
        "id": "09b9cdbf048447e5cd142076a0156fe971db6ca5",
        "message": "ci: harden CI \u2014 SHA pins, timeouts, sysstat sampling",
        "url": "https://github.com/locustbaby/duotunnel/commit/09b9cdbf048447e5cd142076a0156fe971db6ca5"
      },
      "scenarios": [
        {
          "name": "ingress_http_get",
          "protocol": "HTTP",
          "direction": "ingress",
          "category": "basic",
          "p50": 0.58,
          "p95": 1.21,
          "p99": null,
          "err": 0,
          "rps": 6.29,
          "requests": 724
        },
        {
          "name": "ingress_http_post",
          "protocol": "HTTP",
          "direction": "ingress",
          "category": "basic",
          "p50": 0.61,
          "p95": 1.24,
          "p99": null,
          "err": 0,
          "rps": 3.9,
          "requests": 449
        },
        {
          "name": "egress_http_get",
          "protocol": "HTTP",
          "direction": "egress",
          "category": "basic",
          "p50": 0.49,
          "p95": 0.97,
          "p99": null,
          "err": 0,
          "rps": 7.82,
          "requests": 900
        },
        {
          "name": "egress_http_post",
          "protocol": "HTTP",
          "direction": "egress",
          "category": "basic",
          "p50": 0.48,
          "p95": 0.58,
          "p99": null,
          "err": 0,
          "rps": 6.19,
          "requests": 712
        },
        {
          "name": "bidir_mixed",
          "protocol": "HTTP",
          "direction": "bidir",
          "category": "basic",
          "p50": 1,
          "p95": 2,
          "p99": null,
          "err": 0,
          "rps": 2.28,
          "requests": 262
        },
        {
          "name": "ingress_post_1k",
          "protocol": "HTTP",
          "direction": "ingress",
          "category": "body_size",
          "p50": 0.63,
          "p95": 1.97,
          "p99": null,
          "err": 0,
          "rps": 3.92,
          "requests": 451
        },
        {
          "name": "ingress_post_10k",
          "protocol": "HTTP",
          "direction": "ingress",
          "category": "body_size",
          "p50": 1.07,
          "p95": 2.41,
          "p99": null,
          "err": 0,
          "rps": 2.62,
          "requests": 301
        },
        {
          "name": "ingress_post_100k",
          "protocol": "HTTP",
          "direction": "ingress",
          "category": "body_size",
          "p50": 3.05,
          "p95": 42.78,
          "p99": null,
          "err": 0,
          "rps": 1.31,
          "requests": 151
        },
        {
          "name": "egress_post_10k",
          "protocol": "HTTP",
          "direction": "egress",
          "category": "body_size",
          "p50": 0.99,
          "p95": 41.17,
          "p99": null,
          "err": 0,
          "rps": 2.61,
          "requests": 300
        },
        {
          "name": "grpc_health_ingress",
          "protocol": "gRPC",
          "direction": "ingress",
          "category": "basic",
          "p50": 2,
          "p95": 42,
          "p99": null,
          "err": 0,
          "rps": 2.61,
          "requests": 300
        },
        {
          "name": "grpc_echo_ingress",
          "protocol": "gRPC",
          "direction": "ingress",
          "category": "basic",
          "p50": 2,
          "p95": 43,
          "p99": null,
          "err": 0,
          "rps": 2.62,
          "requests": 301
        },
        {
          "name": "grpc_large_payload",
          "protocol": "gRPC",
          "direction": "ingress",
          "category": "body_size",
          "p50": 2,
          "p95": 43,
          "p99": null,
          "err": 0,
          "rps": 1.96,
          "requests": 225
        },
        {
          "name": "grpc_high_qps",
          "protocol": "gRPC",
          "direction": "ingress",
          "category": "stress",
          "p50": 2,
          "p95": 42,
          "p99": null,
          "err": 0,
          "rps": 11.29,
          "requests": 1299
        },
        {
          "name": "ws_ingress",
          "protocol": "WS",
          "direction": "ingress",
          "category": "basic",
          "p50": 1,
          "p95": 1,
          "p99": null,
          "err": 0,
          "rps": 1.31,
          "requests": 151
        },
        {
          "name": "ws_multi_msg",
          "protocol": "WS",
          "direction": "ingress",
          "category": "basic",
          "p50": 43,
          "p95": 45,
          "p99": null,
          "err": 0,
          "rps": 0.66,
          "requests": 76
        },
        {
          "name": "ingress_1000qps",
          "protocol": "HTTP",
          "direction": "ingress",
          "category": "stress",
          "p50": 0.7,
          "p95": 1.19,
          "p99": null,
          "err": 0,
          "rps": 130.38,
          "requests": 15001
        },
        {
          "name": "egress_1000qps",
          "protocol": "HTTP",
          "direction": "egress",
          "category": "stress",
          "p50": 0.63,
          "p95": 1.06,
          "p99": null,
          "err": 0,
          "rps": 130.38,
          "requests": 15001
        },
        {
          "name": "ingress_2000qps",
          "protocol": "HTTP",
          "direction": "ingress",
          "category": "stress",
          "p50": 1.33,
          "p95": 6.8,
          "p99": null,
          "err": 0,
          "rps": 260.2,
          "requests": 29938
        },
        {
          "name": "egress_2000qps",
          "protocol": "HTTP",
          "direction": "egress",
          "category": "stress",
          "p50": 1.21,
          "p95": 6.68,
          "p99": null,
          "err": 0,
          "rps": 260.42,
          "requests": 29963
        },
        {
          "name": "ingress_3000qps",
          "protocol": "HTTP",
          "direction": "ingress",
          "category": "stress",
          "p50": 21.97,
          "p95": 68.48,
          "p99": null,
          "err": 0,
          "rps": 339.33,
          "requests": 39043
        },
        {
          "name": "egress_3000qps",
          "protocol": "HTTP",
          "direction": "egress",
          "category": "stress",
          "p50": 21.36,
          "p95": 68.82,
          "p99": null,
          "err": 0.93,
          "rps": 336.7,
          "requests": 38740
        }
      ],
      "summary": {
        "totalRPS": 1514.78,
        "totalErr": 0.21,
        "totalRequests": 174288
      },
      "scenarioConfig": {
        "getFullExecutionRequirements": {},
        "getSortedConfigs": {},
        "unmarshalJSON": {},
        "validate": {}
      },
      "resources": {
        "server": {
          "cpu": [
            {
              "t": 0.0,
              "v": 1.0
            },
            {
              "t": 4.0,
              "v": 2.0
            },
            {
              "t": 8.0,
              "v": 4.0
            },
            {
              "t": 12.0,
              "v": 7.5
            },
            {
              "t": 16.0,
              "v": 9.5
            },
            {
              "t": 20.0,
              "v": 10.0
            },
            {
              "t": 24.0,
              "v": 10.0
            },
            {
              "t": 28.0,
              "v": 10.5
            },
            {
              "t": 32.0,
              "v": 10.5
            },
            {
              "t": 36.0,
              "v": 9.5
            },
            {
              "t": 40.0,
              "v": 7.5
            },
            {
              "t": 44.0,
              "v": 4.5
            },
            {
              "t": 48.0,
              "v": 2.0
            },
            {
              "t": 52.0,
              "v": 0.0
            },
            {
              "t": 56.0,
              "v": 0.0
            },
            {
              "t": 60.0,
              "v": 3.0
            },
            {
              "t": 64.0,
              "v": 5.0
            },
            {
              "t": 68.0,
              "v": 8.5
            },
            {
              "t": 72.0,
              "v": 11.5
            },
            {
              "t": 76.0,
              "v": 13.5
            },
            {
              "t": 80.0,
              "v": 15.0
            },
            {
              "t": 84.0,
              "v": 14.5
            },
            {
              "t": 88.0,
              "v": 13.0
            },
            {
              "t": 92.0,
              "v": 11.0
            },
            {
              "t": 96.0,
              "v": 6.0
            },
            {
              "t": 100.0,
              "v": 0.0
            },
            {
              "t": 104.0,
              "v": 0.0
            },
            {
              "t": 108.0,
              "v": 0.0
            },
            {
              "t": 112.0,
              "v": 0.0
            },
            {
              "t": 116.0,
              "v": 0.0
            },
            {
              "t": 120.0,
              "v": 54.0
            },
            {
              "t": 124.0,
              "v": 63.0
            },
            {
              "t": 128.0,
              "v": 63.0
            },
            {
              "t": 132.0,
              "v": 63.0
            },
            {
              "t": 136.0,
              "v": 62.0
            },
            {
              "t": 140.0,
              "v": 62.5
            },
            {
              "t": 144.0,
              "v": 62.5
            },
            {
              "t": 148.0,
              "v": 39.5
            },
            {
              "t": 152.0,
              "v": 0.0
            },
            {
              "t": 156.0,
              "v": 0.0
            },
            {
              "t": 160.0,
              "v": 82.0
            },
            {
              "t": 164.0,
              "v": 92.0
            },
            {
              "t": 168.0,
              "v": 92.5
            },
            {
              "t": 172.0,
              "v": 92.0
            },
            {
              "t": 176.0,
              "v": 93.5
            },
            {
              "t": 180.0,
              "v": 92.5
            },
            {
              "t": 184.0,
              "v": 92.0
            },
            {
              "t": 188.0,
              "v": 59.5
            },
            {
              "t": 192.0,
              "v": 0.0
            },
            {
              "t": 196.0,
              "v": 0.0
            },
            {
              "t": 200.0,
              "v": 73.13
            },
            {
              "t": 204.0,
              "v": 81.5
            },
            {
              "t": 208.0,
              "v": 82.0
            },
            {
              "t": 212.0,
              "v": 79.5
            },
            {
              "t": 216.0,
              "v": 75.5
            },
            {
              "t": 220.0,
              "v": 68.5
            },
            {
              "t": 224.0,
              "v": 68.16
            }
          ],
          "rss": [
            {
              "t": 2.0,
              "v": 34.3
            },
            {
              "t": 6.0,
              "v": 34.3
            },
            {
              "t": 10.0,
              "v": 35.3
            },
            {
              "t": 14.0,
              "v": 35.3
            },
            {
              "t": 18.0,
              "v": 35.3
            },
            {
              "t": 22.0,
              "v": 34.7
            },
            {
              "t": 26.0,
              "v": 34.6
            },
            {
              "t": 30.0,
              "v": 34.5
            },
            {
              "t": 34.0,
              "v": 34.5
            },
            {
              "t": 38.0,
              "v": 33.9
            },
            {
              "t": 42.0,
              "v": 33.9
            },
            {
              "t": 46.0,
              "v": 33.9
            },
            {
              "t": 50.0,
              "v": 33.9
            },
            {
              "t": 54.0,
              "v": 33.9
            },
            {
              "t": 58.0,
              "v": 33.9
            },
            {
              "t": 62.0,
              "v": 33.8
            },
            {
              "t": 66.0,
              "v": 34.5
            },
            {
              "t": 70.0,
              "v": 34.7
            },
            {
              "t": 74.0,
              "v": 34.8
            },
            {
              "t": 78.0,
              "v": 34.8
            },
            {
              "t": 82.0,
              "v": 35.0
            },
            {
              "t": 86.0,
              "v": 34.4
            },
            {
              "t": 90.0,
              "v": 34.5
            },
            {
              "t": 94.0,
              "v": 34.0
            },
            {
              "t": 98.0,
              "v": 34.2
            },
            {
              "t": 102.0,
              "v": 34.2
            },
            {
              "t": 106.0,
              "v": 34.2
            },
            {
              "t": 110.0,
              "v": 34.2
            },
            {
              "t": 114.0,
              "v": 34.2
            },
            {
              "t": 118.0,
              "v": 34.2
            },
            {
              "t": 122.0,
              "v": 32.3
            },
            {
              "t": 126.0,
              "v": 32.7
            },
            {
              "t": 130.0,
              "v": 33.4
            },
            {
              "t": 134.0,
              "v": 33.0
            },
            {
              "t": 138.0,
              "v": 33.1
            },
            {
              "t": 142.0,
              "v": 32.9
            },
            {
              "t": 146.0,
              "v": 33.3
            },
            {
              "t": 150.0,
              "v": 33.3
            },
            {
              "t": 154.0,
              "v": 33.3
            },
            {
              "t": 158.0,
              "v": 33.3
            },
            {
              "t": 162.0,
              "v": 35.1
            },
            {
              "t": 166.0,
              "v": 37.9
            },
            {
              "t": 170.0,
              "v": 37.5
            },
            {
              "t": 174.0,
              "v": 37.6
            },
            {
              "t": 178.0,
              "v": 37.5
            },
            {
              "t": 182.0,
              "v": 37.1
            },
            {
              "t": 186.0,
              "v": 39.6
            },
            {
              "t": 190.0,
              "v": 38.9
            },
            {
              "t": 194.0,
              "v": 38.9
            },
            {
              "t": 198.0,
              "v": 38.9
            },
            {
              "t": 202.0,
              "v": 41.9
            },
            {
              "t": 206.0,
              "v": 43.7
            },
            {
              "t": 210.0,
              "v": 42.2
            },
            {
              "t": 214.0,
              "v": 43.9
            },
            {
              "t": 218.0,
              "v": 48.8
            },
            {
              "t": 222.0,
              "v": 45.8
            },
            {
              "t": 226.0,
              "v": 45.7
            }
          ]
        },
        "client": {
          "cpu": [
            {
              "t": 0.0,
              "v": 0.5
            },
            {
              "t": 4.0,
              "v": 2.0
            },
            {
              "t": 8.0,
              "v": 4.0
            },
            {
              "t": 12.0,
              "v": 6.0
            },
            {
              "t": 16.0,
              "v": 8.5
            },
            {
              "t": 20.0,
              "v": 9.5
            },
            {
              "t": 24.0,
              "v": 10.0
            },
            {
              "t": 28.0,
              "v": 10.0
            },
            {
              "t": 32.0,
              "v": 10.0
            },
            {
              "t": 36.0,
              "v": 9.5
            },
            {
              "t": 40.0,
              "v": 7.0
            },
            {
              "t": 44.0,
              "v": 4.0
            },
            {
              "t": 48.0,
              "v": 1.0
            },
            {
              "t": 52.0,
              "v": 0.5
            },
            {
              "t": 56.0,
              "v": 0.0
            },
            {
              "t": 60.0,
              "v": 3.5
            },
            {
              "t": 64.0,
              "v": 4.5
            },
            {
              "t": 68.0,
              "v": 8.0
            },
            {
              "t": 72.0,
              "v": 11.0
            },
            {
              "t": 76.0,
              "v": 13.0
            },
            {
              "t": 80.0,
              "v": 14.0
            },
            {
              "t": 84.0,
              "v": 14.0
            },
            {
              "t": 88.0,
              "v": 12.5
            },
            {
              "t": 92.0,
              "v": 11.0
            },
            {
              "t": 96.0,
              "v": 5.5
            },
            {
              "t": 100.0,
              "v": 0.0
            },
            {
              "t": 104.0,
              "v": 0.0
            },
            {
              "t": 108.0,
              "v": 0.0
            },
            {
              "t": 112.0,
              "v": 0.0
            },
            {
              "t": 116.0,
              "v": 0.0
            },
            {
              "t": 120.0,
              "v": 54.0
            },
            {
              "t": 124.0,
              "v": 62.5
            },
            {
              "t": 128.0,
              "v": 61.5
            },
            {
              "t": 132.0,
              "v": 63.0
            },
            {
              "t": 136.0,
              "v": 62.5
            },
            {
              "t": 140.0,
              "v": 62.5
            },
            {
              "t": 144.0,
              "v": 62.0
            },
            {
              "t": 148.0,
              "v": 39.5
            },
            {
              "t": 152.0,
              "v": 0.0
            },
            {
              "t": 156.0,
              "v": 0.0
            },
            {
              "t": 160.0,
              "v": 80.5
            },
            {
              "t": 164.0,
              "v": 90.0
            },
            {
              "t": 168.0,
              "v": 91.5
            },
            {
              "t": 172.0,
              "v": 89.5
            },
            {
              "t": 176.0,
              "v": 91.0
            },
            {
              "t": 180.0,
              "v": 92.0
            },
            {
              "t": 184.0,
              "v": 90.5
            },
            {
              "t": 188.0,
              "v": 58.5
            },
            {
              "t": 192.0,
              "v": 0.0
            },
            {
              "t": 196.0,
              "v": 0.0
            },
            {
              "t": 200.0,
              "v": 66.67
            },
            {
              "t": 204.0,
              "v": 74.5
            },
            {
              "t": 208.0,
              "v": 75.0
            },
            {
              "t": 212.0,
              "v": 74.0
            },
            {
              "t": 216.0,
              "v": 66.0
            },
            {
              "t": 220.0,
              "v": 61.5
            },
            {
              "t": 224.0,
              "v": 61.19
            }
          ],
          "rss": [
            {
              "t": 2.0,
              "v": 29.6
            },
            {
              "t": 6.0,
              "v": 29.6
            },
            {
              "t": 10.0,
              "v": 30.1
            },
            {
              "t": 14.0,
              "v": 30.1
            },
            {
              "t": 18.0,
              "v": 30.1
            },
            {
              "t": 22.0,
              "v": 29.8
            },
            {
              "t": 26.0,
              "v": 29.8
            },
            {
              "t": 30.0,
              "v": 29.3
            },
            {
              "t": 34.0,
              "v": 29.3
            },
            {
              "t": 38.0,
              "v": 29.3
            },
            {
              "t": 42.0,
              "v": 29.0
            },
            {
              "t": 46.0,
              "v": 29.0
            },
            {
              "t": 50.0,
              "v": 29.0
            },
            {
              "t": 54.0,
              "v": 29.5
            },
            {
              "t": 58.0,
              "v": 29.5
            },
            {
              "t": 62.0,
              "v": 29.0
            },
            {
              "t": 66.0,
              "v": 28.9
            },
            {
              "t": 70.0,
              "v": 28.7
            },
            {
              "t": 74.0,
              "v": 29.5
            },
            {
              "t": 78.0,
              "v": 31.6
            },
            {
              "t": 82.0,
              "v": 30.9
            },
            {
              "t": 86.0,
              "v": 30.7
            },
            {
              "t": 90.0,
              "v": 30.2
            },
            {
              "t": 94.0,
              "v": 30.1
            },
            {
              "t": 98.0,
              "v": 30.4
            },
            {
              "t": 102.0,
              "v": 30.4
            },
            {
              "t": 106.0,
              "v": 30.4
            },
            {
              "t": 110.0,
              "v": 30.4
            },
            {
              "t": 114.0,
              "v": 30.4
            },
            {
              "t": 118.0,
              "v": 30.4
            },
            {
              "t": 122.0,
              "v": 27.7
            },
            {
              "t": 126.0,
              "v": 28.3
            },
            {
              "t": 130.0,
              "v": 28.3
            },
            {
              "t": 134.0,
              "v": 28.7
            },
            {
              "t": 138.0,
              "v": 28.8
            },
            {
              "t": 142.0,
              "v": 28.5
            },
            {
              "t": 146.0,
              "v": 28.5
            },
            {
              "t": 150.0,
              "v": 28.6
            },
            {
              "t": 154.0,
              "v": 28.6
            },
            {
              "t": 158.0,
              "v": 28.6
            },
            {
              "t": 162.0,
              "v": 31.0
            },
            {
              "t": 166.0,
              "v": 32.2
            },
            {
              "t": 170.0,
              "v": 31.8
            },
            {
              "t": 174.0,
              "v": 31.7
            },
            {
              "t": 178.0,
              "v": 31.8
            },
            {
              "t": 182.0,
              "v": 31.2
            },
            {
              "t": 186.0,
              "v": 31.4
            },
            {
              "t": 190.0,
              "v": 31.5
            },
            {
              "t": 194.0,
              "v": 31.5
            },
            {
              "t": 198.0,
              "v": 31.5
            },
            {
              "t": 202.0,
              "v": 33.6
            },
            {
              "t": 206.0,
              "v": 36.8
            },
            {
              "t": 210.0,
              "v": 36.8
            },
            {
              "t": 214.0,
              "v": 37.0
            },
            {
              "t": 218.0,
              "v": 39.0
            },
            {
              "t": 222.0,
              "v": 41.0
            },
            {
              "t": 226.0,
              "v": 40.4
            }
          ]
        },
        "system": {
          "cpu": [
            {
              "t": 0.0,
              "v": 12.1
            },
            {
              "t": 2.0,
              "v": 1.8
            },
            {
              "t": 4.0,
              "v": 3.5
            },
            {
              "t": 6.0,
              "v": 6.2
            },
            {
              "t": 8.0,
              "v": 8.9
            },
            {
              "t": 10.0,
              "v": 10.2
            },
            {
              "t": 12.0,
              "v": 9.5
            },
            {
              "t": 14.0,
              "v": 10.8
            },
            {
              "t": 16.0,
              "v": 10.1
            },
            {
              "t": 18.0,
              "v": 10.3
            },
            {
              "t": 20.0,
              "v": 6.7
            },
            {
              "t": 22.0,
              "v": 3.9
            },
            {
              "t": 24.0,
              "v": 1.5
            },
            {
              "t": 26.0,
              "v": 0.6
            },
            {
              "t": 28.0,
              "v": 0.9
            },
            {
              "t": 30.0,
              "v": 3.8
            },
            {
              "t": 32.0,
              "v": 4.6
            },
            {
              "t": 34.0,
              "v": 8.0
            },
            {
              "t": 36.0,
              "v": 10.8
            },
            {
              "t": 38.0,
              "v": 13.4
            },
            {
              "t": 40.0,
              "v": 13.3
            },
            {
              "t": 42.0,
              "v": 13.6
            },
            {
              "t": 44.0,
              "v": 12.4
            },
            {
              "t": 46.0,
              "v": 11.0
            },
            {
              "t": 48.0,
              "v": 5.5
            },
            {
              "t": 50.0,
              "v": 0.2
            },
            {
              "t": 52.0,
              "v": 0.1
            },
            {
              "t": 54.0,
              "v": 1.0
            },
            {
              "t": 56.0,
              "v": 0.4
            },
            {
              "t": 58.0,
              "v": 0.2
            },
            {
              "t": 60.0,
              "v": 56.9
            },
            {
              "t": 62.0,
              "v": 64.9
            },
            {
              "t": 64.0,
              "v": 65.3
            },
            {
              "t": 66.0,
              "v": 64.6
            },
            {
              "t": 68.0,
              "v": 65.2
            },
            {
              "t": 70.0,
              "v": 64.8
            },
            {
              "t": 72.0,
              "v": 64.9
            },
            {
              "t": 74.0,
              "v": 41.4
            },
            {
              "t": 76.0,
              "v": 0.1
            },
            {
              "t": 78.0,
              "v": 0.4
            },
            {
              "t": 80.0,
              "v": 82.7
            },
            {
              "t": 82.0,
              "v": 95.5
            },
            {
              "t": 84.0,
              "v": 94.5
            },
            {
              "t": 86.0,
              "v": 95.3
            },
            {
              "t": 88.0,
              "v": 94.7
            },
            {
              "t": 90.0,
              "v": 94.9
            },
            {
              "t": 92.0,
              "v": 94.9
            },
            {
              "t": 94.0,
              "v": 59.2
            },
            {
              "t": 96.0,
              "v": 1.1
            },
            {
              "t": 98.0,
              "v": 0.5
            },
            {
              "t": 100.0,
              "v": 86.7
            },
            {
              "t": 102.0,
              "v": 99.5
            },
            {
              "t": 104.0,
              "v": 99.9
            },
            {
              "t": 106.0,
              "v": 100.0
            },
            {
              "t": 108.0,
              "v": 100.0
            },
            {
              "t": 110.0,
              "v": 99.9
            },
            {
              "t": 112.0,
              "v": 100.0
            }
          ]
        }
      }
    },
    {
      "timestamp": "2026-03-03T09:53:35.429Z",
      "commit": {
        "id": "9af9b96135ed54ed2701dc16b39479e0fb903804",
        "message": "ci: harden CI \u2014 SHA pins, timeouts, sysstat sampling",
        "url": "https://github.com/locustbaby/duotunnel/commit/9af9b96135ed54ed2701dc16b39479e0fb903804"
      },
      "scenarios": [
        {
          "name": "ingress_http_get",
          "protocol": "HTTP",
          "direction": "ingress",
          "category": "basic",
          "p50": 0.67,
          "p95": 1.43,
          "p99": null,
          "err": 0,
          "rps": 6.3,
          "requests": 725
        },
        {
          "name": "ingress_http_post",
          "protocol": "HTTP",
          "direction": "ingress",
          "category": "basic",
          "p50": 0.68,
          "p95": 1.49,
          "p99": null,
          "err": 0,
          "rps": 3.91,
          "requests": 450
        },
        {
          "name": "egress_http_get",
          "protocol": "HTTP",
          "direction": "egress",
          "category": "basic",
          "p50": 0.56,
          "p95": 1.11,
          "p99": null,
          "err": 0,
          "rps": 7.82,
          "requests": 900
        },
        {
          "name": "egress_http_post",
          "protocol": "HTTP",
          "direction": "egress",
          "category": "basic",
          "p50": 0.54,
          "p95": 0.77,
          "p99": null,
          "err": 0,
          "rps": 6.19,
          "requests": 712
        },
        {
          "name": "bidir_mixed",
          "protocol": "HTTP",
          "direction": "bidir",
          "category": "basic",
          "p50": 2,
          "p95": 2,
          "p99": null,
          "err": 0,
          "rps": 2.28,
          "requests": 262
        },
        {
          "name": "ingress_post_1k",
          "protocol": "HTTP",
          "direction": "ingress",
          "category": "body_size",
          "p50": 0.82,
          "p95": 2.23,
          "p99": null,
          "err": 0,
          "rps": 3.92,
          "requests": 451
        },
        {
          "name": "ingress_post_10k",
          "protocol": "HTTP",
          "direction": "ingress",
          "category": "body_size",
          "p50": 1.19,
          "p95": 2.47,
          "p99": null,
          "err": 0,
          "rps": 2.62,
          "requests": 301
        },
        {
          "name": "ingress_post_100k",
          "protocol": "HTTP",
          "direction": "ingress",
          "category": "body_size",
          "p50": 3.34,
          "p95": 43.05,
          "p99": null,
          "err": 0,
          "rps": 1.31,
          "requests": 151
        },
        {
          "name": "egress_post_10k",
          "protocol": "HTTP",
          "direction": "egress",
          "category": "body_size",
          "p50": 1.03,
          "p95": 2.44,
          "p99": null,
          "err": 0,
          "rps": 2.62,
          "requests": 301
        },
        {
          "name": "grpc_health_ingress",
          "protocol": "gRPC",
          "direction": "ingress",
          "category": "basic",
          "p50": 2,
          "p95": 43,
          "p99": null,
          "err": 0,
          "rps": 2.61,
          "requests": 300
        },
        {
          "name": "grpc_echo_ingress",
          "protocol": "gRPC",
          "direction": "ingress",
          "category": "basic",
          "p50": 2,
          "p95": 43,
          "p99": null,
          "err": 0,
          "rps": 2.61,
          "requests": 300
        },
        {
          "name": "grpc_large_payload",
          "protocol": "gRPC",
          "direction": "ingress",
          "category": "body_size",
          "p50": 2,
          "p95": 43,
          "p99": null,
          "err": 0,
          "rps": 1.96,
          "requests": 226
        },
        {
          "name": "grpc_high_qps",
          "protocol": "gRPC",
          "direction": "ingress",
          "category": "stress",
          "p50": 2,
          "p95": 43,
          "p99": null,
          "err": 0,
          "rps": 11.3,
          "requests": 1300
        },
        {
          "name": "ws_ingress",
          "protocol": "WS",
          "direction": "ingress",
          "category": "basic",
          "p50": 1,
          "p95": 1,
          "p99": null,
          "err": 0,
          "rps": 1.31,
          "requests": 151
        },
        {
          "name": "ws_multi_msg",
          "protocol": "WS",
          "direction": "ingress",
          "category": "basic",
          "p50": 44,
          "p95": 45.25,
          "p99": null,
          "err": 0,
          "rps": 0.66,
          "requests": 76
        },
        {
          "name": "ingress_1000qps",
          "protocol": "HTTP",
          "direction": "ingress",
          "category": "stress",
          "p50": 0.71,
          "p95": 1.29,
          "p99": null,
          "err": 0,
          "rps": 130.38,
          "requests": 15001
        },
        {
          "name": "egress_1000qps",
          "protocol": "HTTP",
          "direction": "egress",
          "category": "stress",
          "p50": 0.64,
          "p95": 1.15,
          "p99": null,
          "err": 0,
          "rps": 130.37,
          "requests": 15000
        },
        {
          "name": "ingress_2000qps",
          "protocol": "HTTP",
          "direction": "ingress",
          "category": "stress",
          "p50": 1.44,
          "p95": 9.04,
          "p99": null,
          "err": 0,
          "rps": 259.98,
          "requests": 29913
        },
        {
          "name": "egress_2000qps",
          "protocol": "HTTP",
          "direction": "egress",
          "category": "stress",
          "p50": 1.32,
          "p95": 8.44,
          "p99": null,
          "err": 0,
          "rps": 259.82,
          "requests": 29894
        },
        {
          "name": "ingress_3000qps",
          "protocol": "HTTP",
          "direction": "ingress",
          "category": "stress",
          "p50": 38.17,
          "p95": 84.78,
          "p99": null,
          "err": 0,
          "rps": 323.09,
          "requests": 37174
        },
        {
          "name": "egress_3000qps",
          "protocol": "HTTP",
          "direction": "egress",
          "category": "stress",
          "p50": 36.67,
          "p95": 83.35,
          "p99": null,
          "err": 5.93,
          "rps": 325.71,
          "requests": 37475
        }
      ],
      "summary": {
        "totalRPS": 1486.76,
        "totalErr": 1.3,
        "totalRequests": 171063
      },
      "scenarioConfig": {
        "getFullExecutionRequirements": {},
        "getSortedConfigs": {},
        "unmarshalJSON": {},
        "validate": {}
      },
      "resources": {
        "server": {
          "cpu": [
            {
              "t": 0.0,
              "v": 0.5
            },
            {
              "t": 4.0,
              "v": 2.0
            },
            {
              "t": 8.0,
              "v": 5.0
            },
            {
              "t": 12.0,
              "v": 9.0
            },
            {
              "t": 16.0,
              "v": 10.0
            },
            {
              "t": 20.0,
              "v": 10.5
            },
            {
              "t": 24.0,
              "v": 11.0
            },
            {
              "t": 28.0,
              "v": 11.5
            },
            {
              "t": 32.0,
              "v": 11.0
            },
            {
              "t": 36.0,
              "v": 11.5
            },
            {
              "t": 40.0,
              "v": 9.0
            },
            {
              "t": 44.0,
              "v": 5.0
            },
            {
              "t": 48.0,
              "v": 2.5
            },
            {
              "t": 52.0,
              "v": 0.0
            },
            {
              "t": 56.0,
              "v": 0.0
            },
            {
              "t": 60.0,
              "v": 4.0
            },
            {
              "t": 64.0,
              "v": 5.5
            },
            {
              "t": 68.0,
              "v": 10.0
            },
            {
              "t": 72.0,
              "v": 13.0
            },
            {
              "t": 76.0,
              "v": 16.0
            },
            {
              "t": 80.0,
              "v": 17.0
            },
            {
              "t": 84.0,
              "v": 16.5
            },
            {
              "t": 88.0,
              "v": 15.5
            },
            {
              "t": 92.0,
              "v": 13.5
            },
            {
              "t": 96.0,
              "v": 7.0
            },
            {
              "t": 100.0,
              "v": 0.0
            },
            {
              "t": 104.0,
              "v": 0.0
            },
            {
              "t": 108.0,
              "v": 0.0
            },
            {
              "t": 112.0,
              "v": 0.0
            },
            {
              "t": 116.0,
              "v": 0.5
            },
            {
              "t": 120.0,
              "v": 57.0
            },
            {
              "t": 124.0,
              "v": 65.5
            },
            {
              "t": 128.0,
              "v": 65.5
            },
            {
              "t": 132.0,
              "v": 65.0
            },
            {
              "t": 136.0,
              "v": 65.0
            },
            {
              "t": 140.0,
              "v": 65.5
            },
            {
              "t": 144.0,
              "v": 65.0
            },
            {
              "t": 148.0,
              "v": 41.0
            },
            {
              "t": 152.0,
              "v": 0.0
            },
            {
              "t": 156.0,
              "v": 0.0
            },
            {
              "t": 160.0,
              "v": 79.5
            },
            {
              "t": 164.0,
              "v": 93.0
            },
            {
              "t": 168.0,
              "v": 91.5
            },
            {
              "t": 172.0,
              "v": 91.0
            },
            {
              "t": 176.0,
              "v": 90.5
            },
            {
              "t": 180.0,
              "v": 90.5
            },
            {
              "t": 184.0,
              "v": 92.0
            },
            {
              "t": 188.0,
              "v": 59.5
            },
            {
              "t": 192.0,
              "v": 0.0
            },
            {
              "t": 196.0,
              "v": 0.0
            },
            {
              "t": 200.0,
              "v": 67.5
            },
            {
              "t": 204.0,
              "v": 79.5
            },
            {
              "t": 208.0,
              "v": 79.5
            },
            {
              "t": 212.0,
              "v": 80.5
            },
            {
              "t": 216.0,
              "v": 72.5
            },
            {
              "t": 220.0,
              "v": 69.0
            },
            {
              "t": 224.0,
              "v": 76.62
            }
          ],
          "rss": [
            {
              "t": 2.0,
              "v": 34.5
            },
            {
              "t": 6.0,
              "v": 34.5
            },
            {
              "t": 10.0,
              "v": 34.9
            },
            {
              "t": 14.0,
              "v": 35.1
            },
            {
              "t": 18.0,
              "v": 35.3
            },
            {
              "t": 22.0,
              "v": 34.9
            },
            {
              "t": 26.0,
              "v": 34.9
            },
            {
              "t": 30.0,
              "v": 34.6
            },
            {
              "t": 34.0,
              "v": 34.8
            },
            {
              "t": 38.0,
              "v": 34.2
            },
            {
              "t": 42.0,
              "v": 34.3
            },
            {
              "t": 46.0,
              "v": 34.1
            },
            {
              "t": 50.0,
              "v": 34.1
            },
            {
              "t": 54.0,
              "v": 34.1
            },
            {
              "t": 58.0,
              "v": 34.1
            },
            {
              "t": 62.0,
              "v": 34.1
            },
            {
              "t": 66.0,
              "v": 34.0
            },
            {
              "t": 70.0,
              "v": 33.9
            },
            {
              "t": 74.0,
              "v": 34.0
            },
            {
              "t": 78.0,
              "v": 34.3
            },
            {
              "t": 82.0,
              "v": 34.0
            },
            {
              "t": 86.0,
              "v": 33.7
            },
            {
              "t": 90.0,
              "v": 33.9
            },
            {
              "t": 94.0,
              "v": 33.7
            },
            {
              "t": 98.0,
              "v": 33.7
            },
            {
              "t": 102.0,
              "v": 33.7
            },
            {
              "t": 106.0,
              "v": 33.7
            },
            {
              "t": 110.0,
              "v": 33.7
            },
            {
              "t": 114.0,
              "v": 33.7
            },
            {
              "t": 118.0,
              "v": 33.7
            },
            {
              "t": 122.0,
              "v": 33.7
            },
            {
              "t": 126.0,
              "v": 33.4
            },
            {
              "t": 130.0,
              "v": 33.6
            },
            {
              "t": 134.0,
              "v": 33.7
            },
            {
              "t": 138.0,
              "v": 36.1
            },
            {
              "t": 142.0,
              "v": 35.6
            },
            {
              "t": 146.0,
              "v": 36.0
            },
            {
              "t": 150.0,
              "v": 35.4
            },
            {
              "t": 154.0,
              "v": 35.4
            },
            {
              "t": 158.0,
              "v": 35.4
            },
            {
              "t": 162.0,
              "v": 35.8
            },
            {
              "t": 166.0,
              "v": 35.8
            },
            {
              "t": 170.0,
              "v": 37.3
            },
            {
              "t": 174.0,
              "v": 37.5
            },
            {
              "t": 178.0,
              "v": 36.7
            },
            {
              "t": 182.0,
              "v": 37.4
            },
            {
              "t": 186.0,
              "v": 38.4
            },
            {
              "t": 190.0,
              "v": 36.4
            },
            {
              "t": 194.0,
              "v": 36.4
            },
            {
              "t": 198.0,
              "v": 36.4
            },
            {
              "t": 202.0,
              "v": 42.2
            },
            {
              "t": 206.0,
              "v": 45.8
            },
            {
              "t": 210.0,
              "v": 49.9
            },
            {
              "t": 214.0,
              "v": 48.4
            },
            {
              "t": 218.0,
              "v": 46.6
            },
            {
              "t": 222.0,
              "v": 48.5
            },
            {
              "t": 226.0,
              "v": 49.1
            }
          ]
        },
        "client": {
          "cpu": [
            {
              "t": 0.0,
              "v": 1.0
            },
            {
              "t": 4.0,
              "v": 2.0
            },
            {
              "t": 8.0,
              "v": 4.0
            },
            {
              "t": 12.0,
              "v": 7.5
            },
            {
              "t": 16.0,
              "v": 8.5
            },
            {
              "t": 20.0,
              "v": 10.5
            },
            {
              "t": 24.0,
              "v": 10.0
            },
            {
              "t": 28.0,
              "v": 11.0
            },
            {
              "t": 32.0,
              "v": 11.0
            },
            {
              "t": 36.0,
              "v": 11.0
            },
            {
              "t": 40.0,
              "v": 8.0
            },
            {
              "t": 44.0,
              "v": 4.5
            },
            {
              "t": 48.0,
              "v": 1.5
            },
            {
              "t": 52.0,
              "v": 0.0
            },
            {
              "t": 56.0,
              "v": 0.0
            },
            {
              "t": 60.0,
              "v": 4.5
            },
            {
              "t": 64.0,
              "v": 5.0
            },
            {
              "t": 68.0,
              "v": 10.0
            },
            {
              "t": 72.0,
              "v": 12.0
            },
            {
              "t": 76.0,
              "v": 16.0
            },
            {
              "t": 80.0,
              "v": 16.0
            },
            {
              "t": 84.0,
              "v": 15.5
            },
            {
              "t": 88.0,
              "v": 14.5
            },
            {
              "t": 92.0,
              "v": 12.5
            },
            {
              "t": 96.0,
              "v": 7.0
            },
            {
              "t": 100.0,
              "v": 0.0
            },
            {
              "t": 104.0,
              "v": 0.0
            },
            {
              "t": 108.0,
              "v": 0.0
            },
            {
              "t": 112.0,
              "v": 0.0
            },
            {
              "t": 116.0,
              "v": 0.0
            },
            {
              "t": 120.0,
              "v": 56.5
            },
            {
              "t": 124.0,
              "v": 65.0
            },
            {
              "t": 128.0,
              "v": 64.5
            },
            {
              "t": 132.0,
              "v": 65.0
            },
            {
              "t": 136.0,
              "v": 64.0
            },
            {
              "t": 140.0,
              "v": 65.0
            },
            {
              "t": 144.0,
              "v": 64.0
            },
            {
              "t": 148.0,
              "v": 40.5
            },
            {
              "t": 152.0,
              "v": 0.0
            },
            {
              "t": 156.0,
              "v": 0.0
            },
            {
              "t": 160.0,
              "v": 77.0
            },
            {
              "t": 164.0,
              "v": 90.0
            },
            {
              "t": 168.0,
              "v": 89.5
            },
            {
              "t": 172.0,
              "v": 88.0
            },
            {
              "t": 176.0,
              "v": 87.5
            },
            {
              "t": 180.0,
              "v": 88.0
            },
            {
              "t": 184.0,
              "v": 88.5
            },
            {
              "t": 188.0,
              "v": 57.0
            },
            {
              "t": 192.0,
              "v": 0.0
            },
            {
              "t": 196.0,
              "v": 0.0
            },
            {
              "t": 200.0,
              "v": 61.0
            },
            {
              "t": 204.0,
              "v": 71.0
            },
            {
              "t": 208.0,
              "v": 73.0
            },
            {
              "t": 212.0,
              "v": 74.0
            },
            {
              "t": 216.0,
              "v": 64.5
            },
            {
              "t": 220.0,
              "v": 65.0
            },
            {
              "t": 224.0,
              "v": 72.64
            }
          ],
          "rss": [
            {
              "t": 2.0,
              "v": 29.7
            },
            {
              "t": 6.0,
              "v": 29.7
            },
            {
              "t": 10.0,
              "v": 30.4
            },
            {
              "t": 14.0,
              "v": 32.4
            },
            {
              "t": 18.0,
              "v": 32.4
            },
            {
              "t": 22.0,
              "v": 32.5
            },
            {
              "t": 26.0,
              "v": 32.5
            },
            {
              "t": 30.0,
              "v": 31.7
            },
            {
              "t": 34.0,
              "v": 31.3
            },
            {
              "t": 38.0,
              "v": 31.4
            },
            {
              "t": 42.0,
              "v": 31.4
            },
            {
              "t": 46.0,
              "v": 31.9
            },
            {
              "t": 50.0,
              "v": 31.9
            },
            {
              "t": 54.0,
              "v": 31.9
            },
            {
              "t": 58.0,
              "v": 31.9
            },
            {
              "t": 62.0,
              "v": 31.9
            },
            {
              "t": 66.0,
              "v": 32.1
            },
            {
              "t": 70.0,
              "v": 32.0
            },
            {
              "t": 74.0,
              "v": 31.4
            },
            {
              "t": 78.0,
              "v": 31.7
            },
            {
              "t": 82.0,
              "v": 31.1
            },
            {
              "t": 86.0,
              "v": 30.9
            },
            {
              "t": 90.0,
              "v": 29.9
            },
            {
              "t": 94.0,
              "v": 29.9
            },
            {
              "t": 98.0,
              "v": 29.9
            },
            {
              "t": 102.0,
              "v": 29.9
            },
            {
              "t": 106.0,
              "v": 29.9
            },
            {
              "t": 110.0,
              "v": 29.9
            },
            {
              "t": 114.0,
              "v": 29.9
            },
            {
              "t": 118.0,
              "v": 29.9
            },
            {
              "t": 122.0,
              "v": 28.4
            },
            {
              "t": 126.0,
              "v": 28.9
            },
            {
              "t": 130.0,
              "v": 28.7
            },
            {
              "t": 134.0,
              "v": 28.9
            },
            {
              "t": 138.0,
              "v": 28.9
            },
            {
              "t": 142.0,
              "v": 28.9
            },
            {
              "t": 146.0,
              "v": 28.7
            },
            {
              "t": 150.0,
              "v": 28.4
            },
            {
              "t": 154.0,
              "v": 28.4
            },
            {
              "t": 158.0,
              "v": 28.4
            },
            {
              "t": 162.0,
              "v": 29.7
            },
            {
              "t": 166.0,
              "v": 30.1
            },
            {
              "t": 170.0,
              "v": 29.8
            },
            {
              "t": 174.0,
              "v": 29.8
            },
            {
              "t": 178.0,
              "v": 30.1
            },
            {
              "t": 182.0,
              "v": 30.4
            },
            {
              "t": 186.0,
              "v": 30.5
            },
            {
              "t": 190.0,
              "v": 30.8
            },
            {
              "t": 194.0,
              "v": 30.8
            },
            {
              "t": 198.0,
              "v": 30.8
            },
            {
              "t": 202.0,
              "v": 36.4
            },
            {
              "t": 206.0,
              "v": 37.2
            },
            {
              "t": 210.0,
              "v": 36.4
            },
            {
              "t": 214.0,
              "v": 40.5
            },
            {
              "t": 218.0,
              "v": 40.6
            },
            {
              "t": 222.0,
              "v": 39.4
            },
            {
              "t": 226.0,
              "v": 40.6
            }
          ]
        },
        "system": {
          "cpu": [
            {
              "t": 0.0,
              "v": 12.3
            },
            {
              "t": 2.0,
              "v": 2.0
            },
            {
              "t": 4.0,
              "v": 5.2
            },
            {
              "t": 6.0,
              "v": 7.6
            },
            {
              "t": 8.0,
              "v": 10.6
            },
            {
              "t": 10.0,
              "v": 10.8
            },
            {
              "t": 12.0,
              "v": 11.9
            },
            {
              "t": 14.0,
              "v": 10.6
            },
            {
              "t": 16.0,
              "v": 12.1
            },
            {
              "t": 18.0,
              "v": 10.7
            },
            {
              "t": 20.0,
              "v": 8.1
            },
            {
              "t": 22.0,
              "v": 5.2
            },
            {
              "t": 24.0,
              "v": 2.1
            },
            {
              "t": 26.0,
              "v": 0.9
            },
            {
              "t": 28.0,
              "v": 0.2
            },
            {
              "t": 30.0,
              "v": 3.5
            },
            {
              "t": 32.0,
              "v": 6.1
            },
            {
              "t": 34.0,
              "v": 10.6
            },
            {
              "t": 36.0,
              "v": 12.2
            },
            {
              "t": 38.0,
              "v": 16.1
            },
            {
              "t": 40.0,
              "v": 17.3
            },
            {
              "t": 42.0,
              "v": 15.0
            },
            {
              "t": 44.0,
              "v": 15.6
            },
            {
              "t": 46.0,
              "v": 12.1
            },
            {
              "t": 48.0,
              "v": 6.8
            },
            {
              "t": 50.0,
              "v": 0.2
            },
            {
              "t": 52.0,
              "v": 0.2
            },
            {
              "t": 54.0,
              "v": 0.1
            },
            {
              "t": 56.0,
              "v": 0.4
            },
            {
              "t": 58.0,
              "v": 0.1
            },
            {
              "t": 60.0,
              "v": 57.2
            },
            {
              "t": 62.0,
              "v": 66.7
            },
            {
              "t": 64.0,
              "v": 66.3
            },
            {
              "t": 66.0,
              "v": 64.9
            },
            {
              "t": 68.0,
              "v": 66.4
            },
            {
              "t": 70.0,
              "v": 65.8
            },
            {
              "t": 72.0,
              "v": 64.8
            },
            {
              "t": 74.0,
              "v": 41.4
            },
            {
              "t": 76.0,
              "v": 0.8
            },
            {
              "t": 78.0,
              "v": 0.1
            },
            {
              "t": 80.0,
              "v": 83.0
            },
            {
              "t": 82.0,
              "v": 95.1
            },
            {
              "t": 84.0,
              "v": 95.2
            },
            {
              "t": 86.0,
              "v": 95.4
            },
            {
              "t": 88.0,
              "v": 95.3
            },
            {
              "t": 90.0,
              "v": 95.1
            },
            {
              "t": 92.0,
              "v": 95.1
            },
            {
              "t": 94.0,
              "v": 60.0
            },
            {
              "t": 96.0,
              "v": 10.6
            },
            {
              "t": 98.0,
              "v": 11.9
            },
            {
              "t": 100.0,
              "v": 88.2
            },
            {
              "t": 102.0,
              "v": 100.0
            },
            {
              "t": 104.0,
              "v": 100.0
            },
            {
              "t": 106.0,
              "v": 100.0
            },
            {
              "t": 108.0,
              "v": 100.0
            },
            {
              "t": 110.0,
              "v": 99.9
            },
            {
              "t": 112.0,
              "v": 99.9
            }
          ]
        }
      }
    },
    {
      "timestamp": "2026-03-03T10:10:48.070Z",
      "commit": {
        "id": "b809e20b7cab57802b77aa20918f63aef9fec3f1",
        "message": "ci: harden CI \u2014 SHA pins, timeouts, sysstat sampling",
        "url": "https://github.com/locustbaby/duotunnel/commit/b809e20b7cab57802b77aa20918f63aef9fec3f1"
      },
      "scenarios": [
        {
          "name": "ingress_http_get",
          "protocol": "HTTP",
          "direction": "ingress",
          "category": "basic",
          "p50": 0.68,
          "p95": 1.3,
          "p99": null,
          "err": 0,
          "rps": 6.29,
          "requests": 724
        },
        {
          "name": "ingress_http_post",
          "protocol": "HTTP",
          "direction": "ingress",
          "category": "basic",
          "p50": 0.65,
          "p95": 1.33,
          "p99": null,
          "err": 0,
          "rps": 3.9,
          "requests": 449
        },
        {
          "name": "egress_http_get",
          "protocol": "HTTP",
          "direction": "egress",
          "category": "basic",
          "p50": 0.53,
          "p95": 1.01,
          "p99": null,
          "err": 0,
          "rps": 7.82,
          "requests": 900
        },
        {
          "name": "egress_http_post",
          "protocol": "HTTP",
          "direction": "egress",
          "category": "basic",
          "p50": 0.5,
          "p95": 0.67,
          "p99": null,
          "err": 0,
          "rps": 6.19,
          "requests": 712
        },
        {
          "name": "bidir_mixed",
          "protocol": "HTTP",
          "direction": "bidir",
          "category": "basic",
          "p50": 1,
          "p95": 2,
          "p99": null,
          "err": 0,
          "rps": 2.28,
          "requests": 262
        },
        {
          "name": "ingress_post_1k",
          "protocol": "HTTP",
          "direction": "ingress",
          "category": "body_size",
          "p50": 0.74,
          "p95": 2.2,
          "p99": null,
          "err": 0,
          "rps": 3.92,
          "requests": 451
        },
        {
          "name": "ingress_post_10k",
          "protocol": "HTTP",
          "direction": "ingress",
          "category": "body_size",
          "p50": 1.17,
          "p95": 2.52,
          "p99": null,
          "err": 0,
          "rps": 2.62,
          "requests": 301
        },
        {
          "name": "ingress_post_100k",
          "protocol": "HTTP",
          "direction": "ingress",
          "category": "body_size",
          "p50": 2.95,
          "p95": 42.89,
          "p99": null,
          "err": 0,
          "rps": 1.3,
          "requests": 150
        },
        {
          "name": "egress_post_10k",
          "protocol": "HTTP",
          "direction": "egress",
          "category": "body_size",
          "p50": 1.16,
          "p95": 2.3,
          "p99": null,
          "err": 0,
          "rps": 2.62,
          "requests": 301
        },
        {
          "name": "grpc_health_ingress",
          "protocol": "gRPC",
          "direction": "ingress",
          "category": "basic",
          "p50": 2,
          "p95": 43,
          "p99": null,
          "err": 0,
          "rps": 2.62,
          "requests": 301
        },
        {
          "name": "grpc_echo_ingress",
          "protocol": "gRPC",
          "direction": "ingress",
          "category": "basic",
          "p50": 2,
          "p95": 43,
          "p99": null,
          "err": 0,
          "rps": 2.62,
          "requests": 301
        },
        {
          "name": "grpc_large_payload",
          "protocol": "gRPC",
          "direction": "ingress",
          "category": "body_size",
          "p50": 2,
          "p95": 43,
          "p99": null,
          "err": 0,
          "rps": 1.96,
          "requests": 225
        },
        {
          "name": "grpc_high_qps",
          "protocol": "gRPC",
          "direction": "ingress",
          "category": "stress",
          "p50": 2,
          "p95": 43,
          "p99": null,
          "err": 0,
          "rps": 11.3,
          "requests": 1300
        },
        {
          "name": "ws_ingress",
          "protocol": "WS",
          "direction": "ingress",
          "category": "basic",
          "p50": 0,
          "p95": 1,
          "p99": null,
          "err": 0,
          "rps": 1.31,
          "requests": 151
        },
        {
          "name": "ws_multi_msg",
          "protocol": "WS",
          "direction": "ingress",
          "category": "basic",
          "p50": 44,
          "p95": 45,
          "p99": null,
          "err": 0,
          "rps": 0.66,
          "requests": 76
        },
        {
          "name": "ingress_1000qps",
          "protocol": "HTTP",
          "direction": "ingress",
          "category": "stress",
          "p50": 0.71,
          "p95": 1.25,
          "p99": null,
          "err": 0,
          "rps": 130.4,
          "requests": 15000
        },
        {
          "name": "egress_1000qps",
          "protocol": "HTTP",
          "direction": "egress",
          "category": "stress",
          "p50": 0.63,
          "p95": 1.11,
          "p99": null,
          "err": 0,
          "rps": 130.41,
          "requests": 15001
        },
        {
          "name": "ingress_2000qps",
          "protocol": "HTTP",
          "direction": "ingress",
          "category": "stress",
          "p50": 1.37,
          "p95": 8.18,
          "p99": null,
          "err": 0,
          "rps": 260.41,
          "requests": 29956
        },
        {
          "name": "egress_2000qps",
          "protocol": "HTTP",
          "direction": "egress",
          "category": "stress",
          "p50": 1.25,
          "p95": 7.85,
          "p99": null,
          "err": 0,
          "rps": 260.39,
          "requests": 29953
        },
        {
          "name": "ingress_3000qps",
          "protocol": "HTTP",
          "direction": "ingress",
          "category": "stress",
          "p50": 36.88,
          "p95": 82.14,
          "p99": null,
          "err": 0,
          "rps": 329.01,
          "requests": 37847
        },
        {
          "name": "egress_3000qps",
          "protocol": "HTTP",
          "direction": "egress",
          "category": "stress",
          "p50": 36.45,
          "p95": 81.24,
          "p99": null,
          "err": 4.18,
          "rps": 330.97,
          "requests": 38073
        }
      ],
      "summary": {
        "totalRPS": 1498.99,
        "totalErr": 0.92,
        "totalRequests": 172434
      },
      "scenarioConfig": {
        "getFullExecutionRequirements": {},
        "getSortedConfigs": {},
        "unmarshalJSON": {},
        "validate": {}
      },
      "resources": {
        "server": {
          "cpu": [
            {
              "t": 0.0,
              "v": 1.0
            },
            {
              "t": 4.0,
              "v": 2.5
            },
            {
              "t": 8.0,
              "v": 5.0
            },
            {
              "t": 12.0,
              "v": 8.0
            },
            {
              "t": 16.0,
              "v": 10.5
            },
            {
              "t": 20.0,
              "v": 10.0
            },
            {
              "t": 24.0,
              "v": 11.0
            },
            {
              "t": 28.0,
              "v": 11.0
            },
            {
              "t": 32.0,
              "v": 10.5
            },
            {
              "t": 36.0,
              "v": 10.0
            },
            {
              "t": 40.0,
              "v": 8.0
            },
            {
              "t": 44.0,
              "v": 4.5
            },
            {
              "t": 48.0,
              "v": 2.0
            },
            {
              "t": 52.0,
              "v": 0.0
            },
            {
              "t": 56.0,
              "v": 0.0
            },
            {
              "t": 60.0,
              "v": 3.5
            },
            {
              "t": 64.0,
              "v": 5.5
            },
            {
              "t": 68.0,
              "v": 9.0
            },
            {
              "t": 72.0,
              "v": 12.0
            },
            {
              "t": 76.0,
              "v": 15.0
            },
            {
              "t": 80.0,
              "v": 15.0
            },
            {
              "t": 84.0,
              "v": 15.0
            },
            {
              "t": 88.0,
              "v": 14.5
            },
            {
              "t": 92.0,
              "v": 12.0
            },
            {
              "t": 96.0,
              "v": 6.0
            },
            {
              "t": 100.0,
              "v": 0.0
            },
            {
              "t": 104.0,
              "v": 0.0
            },
            {
              "t": 108.0,
              "v": 0.0
            },
            {
              "t": 112.0,
              "v": 0.0
            },
            {
              "t": 116.0,
              "v": 0.0
            },
            {
              "t": 120.0,
              "v": 56.0
            },
            {
              "t": 124.0,
              "v": 63.0
            },
            {
              "t": 128.0,
              "v": 63.5
            },
            {
              "t": 132.0,
              "v": 63.0
            },
            {
              "t": 136.0,
              "v": 63.0
            },
            {
              "t": 140.0,
              "v": 64.0
            },
            {
              "t": 144.0,
              "v": 63.5
            },
            {
              "t": 148.0,
              "v": 38.5
            },
            {
              "t": 152.0,
              "v": 0.0
            },
            {
              "t": 156.0,
              "v": 0.0
            },
            {
              "t": 160.0,
              "v": 80.5
            },
            {
              "t": 164.0,
              "v": 92.5
            },
            {
              "t": 168.0,
              "v": 91.04
            },
            {
              "t": 172.0,
              "v": 92.0
            },
            {
              "t": 176.0,
              "v": 91.5
            },
            {
              "t": 180.0,
              "v": 92.5
            },
            {
              "t": 184.0,
              "v": 91.0
            },
            {
              "t": 188.0,
              "v": 56.0
            },
            {
              "t": 192.0,
              "v": 0.0
            },
            {
              "t": 196.0,
              "v": 0.0
            },
            {
              "t": 200.0,
              "v": 57.0
            },
            {
              "t": 204.0,
              "v": 64.5
            },
            {
              "t": 208.0,
              "v": 80.0
            },
            {
              "t": 212.0,
              "v": 82.5
            },
            {
              "t": 216.0,
              "v": 79.0
            },
            {
              "t": 220.0,
              "v": 81.5
            },
            {
              "t": 224.0,
              "v": 76.0
            }
          ],
          "rss": [
            {
              "t": 2.0,
              "v": 34.5
            },
            {
              "t": 6.0,
              "v": 34.5
            },
            {
              "t": 10.0,
              "v": 35.0
            },
            {
              "t": 14.0,
              "v": 35.4
            },
            {
              "t": 18.0,
              "v": 35.1
            },
            {
              "t": 22.0,
              "v": 35.0
            },
            {
              "t": 26.0,
              "v": 34.6
            },
            {
              "t": 30.0,
              "v": 34.7
            },
            {
              "t": 34.0,
              "v": 34.4
            },
            {
              "t": 38.0,
              "v": 34.6
            },
            {
              "t": 42.0,
              "v": 34.5
            },
            {
              "t": 46.0,
              "v": 34.5
            },
            {
              "t": 50.0,
              "v": 34.5
            },
            {
              "t": 54.0,
              "v": 34.5
            },
            {
              "t": 58.0,
              "v": 34.5
            },
            {
              "t": 62.0,
              "v": 34.4
            },
            {
              "t": 66.0,
              "v": 34.4
            },
            {
              "t": 70.0,
              "v": 34.2
            },
            {
              "t": 74.0,
              "v": 34.5
            },
            {
              "t": 78.0,
              "v": 34.5
            },
            {
              "t": 82.0,
              "v": 34.3
            },
            {
              "t": 86.0,
              "v": 34.3
            },
            {
              "t": 90.0,
              "v": 34.1
            },
            {
              "t": 94.0,
              "v": 34.0
            },
            {
              "t": 98.0,
              "v": 33.7
            },
            {
              "t": 102.0,
              "v": 33.7
            },
            {
              "t": 106.0,
              "v": 33.7
            },
            {
              "t": 110.0,
              "v": 33.7
            },
            {
              "t": 114.0,
              "v": 33.7
            },
            {
              "t": 118.0,
              "v": 33.7
            },
            {
              "t": 122.0,
              "v": 33.0
            },
            {
              "t": 126.0,
              "v": 33.5
            },
            {
              "t": 130.0,
              "v": 33.3
            },
            {
              "t": 134.0,
              "v": 34.0
            },
            {
              "t": 138.0,
              "v": 33.8
            },
            {
              "t": 142.0,
              "v": 33.8
            },
            {
              "t": 146.0,
              "v": 33.5
            },
            {
              "t": 150.0,
              "v": 33.5
            },
            {
              "t": 154.0,
              "v": 33.5
            },
            {
              "t": 158.0,
              "v": 33.5
            },
            {
              "t": 162.0,
              "v": 38.0
            },
            {
              "t": 166.0,
              "v": 37.8
            },
            {
              "t": 170.0,
              "v": 37.5
            },
            {
              "t": 174.0,
              "v": 37.2
            },
            {
              "t": 178.0,
              "v": 37.0
            },
            {
              "t": 182.0,
              "v": 36.4
            },
            {
              "t": 186.0,
              "v": 35.6
            },
            {
              "t": 190.0,
              "v": 36.2
            },
            {
              "t": 194.0,
              "v": 36.2
            },
            {
              "t": 198.0,
              "v": 36.2
            },
            {
              "t": 202.0,
              "v": 45.9
            },
            {
              "t": 206.0,
              "v": 45.7
            },
            {
              "t": 210.0,
              "v": 50.3
            },
            {
              "t": 214.0,
              "v": 50.9
            },
            {
              "t": 218.0,
              "v": 50.8
            },
            {
              "t": 222.0,
              "v": 51.0
            },
            {
              "t": 226.0,
              "v": 52.5
            }
          ]
        },
        "client": {
          "cpu": [
            {
              "t": 0.0,
              "v": 1.0
            },
            {
              "t": 4.0,
              "v": 2.0
            },
            {
              "t": 8.0,
              "v": 4.5
            },
            {
              "t": 12.0,
              "v": 7.0
            },
            {
              "t": 16.0,
              "v": 9.0
            },
            {
              "t": 20.0,
              "v": 9.5
            },
            {
              "t": 24.0,
              "v": 10.0
            },
            {
              "t": 28.0,
              "v": 10.0
            },
            {
              "t": 32.0,
              "v": 10.0
            },
            {
              "t": 36.0,
              "v": 10.0
            },
            {
              "t": 40.0,
              "v": 6.5
            },
            {
              "t": 44.0,
              "v": 4.0
            },
            {
              "t": 48.0,
              "v": 1.5
            },
            {
              "t": 52.0,
              "v": 0.5
            },
            {
              "t": 56.0,
              "v": 0.0
            },
            {
              "t": 60.0,
              "v": 3.0
            },
            {
              "t": 64.0,
              "v": 5.5
            },
            {
              "t": 68.0,
              "v": 9.0
            },
            {
              "t": 72.0,
              "v": 11.5
            },
            {
              "t": 76.0,
              "v": 14.0
            },
            {
              "t": 80.0,
              "v": 14.5
            },
            {
              "t": 84.0,
              "v": 14.0
            },
            {
              "t": 88.0,
              "v": 13.5
            },
            {
              "t": 92.0,
              "v": 11.5
            },
            {
              "t": 96.0,
              "v": 5.5
            },
            {
              "t": 100.0,
              "v": 0.0
            },
            {
              "t": 104.0,
              "v": 0.0
            },
            {
              "t": 108.0,
              "v": 0.0
            },
            {
              "t": 112.0,
              "v": 0.0
            },
            {
              "t": 116.0,
              "v": 0.0
            },
            {
              "t": 120.0,
              "v": 54.5
            },
            {
              "t": 124.0,
              "v": 62.5
            },
            {
              "t": 128.0,
              "v": 62.0
            },
            {
              "t": 132.0,
              "v": 62.5
            },
            {
              "t": 136.0,
              "v": 62.0
            },
            {
              "t": 140.0,
              "v": 62.5
            },
            {
              "t": 144.0,
              "v": 62.5
            },
            {
              "t": 148.0,
              "v": 39.0
            },
            {
              "t": 152.0,
              "v": 0.0
            },
            {
              "t": 156.0,
              "v": 0.0
            },
            {
              "t": 160.0,
              "v": 77.5
            },
            {
              "t": 164.0,
              "v": 90.5
            },
            {
              "t": 168.0,
              "v": 88.56
            },
            {
              "t": 172.0,
              "v": 89.5
            },
            {
              "t": 176.0,
              "v": 88.5
            },
            {
              "t": 180.0,
              "v": 89.0
            },
            {
              "t": 184.0,
              "v": 88.5
            },
            {
              "t": 188.0,
              "v": 54.5
            },
            {
              "t": 192.0,
              "v": 0.0
            },
            {
              "t": 196.0,
              "v": 0.0
            },
            {
              "t": 200.0,
              "v": 50.5
            },
            {
              "t": 204.0,
              "v": 57.0
            },
            {
              "t": 208.0,
              "v": 71.5
            },
            {
              "t": 212.0,
              "v": 73.0
            },
            {
              "t": 216.0,
              "v": 71.0
            },
            {
              "t": 220.0,
              "v": 73.0
            },
            {
              "t": 224.0,
              "v": 70.0
            }
          ],
          "rss": [
            {
              "t": 2.0,
              "v": 29.6
            },
            {
              "t": 6.0,
              "v": 29.6
            },
            {
              "t": 10.0,
              "v": 30.1
            },
            {
              "t": 14.0,
              "v": 30.1
            },
            {
              "t": 18.0,
              "v": 32.1
            },
            {
              "t": 22.0,
              "v": 32.1
            },
            {
              "t": 26.0,
              "v": 32.1
            },
            {
              "t": 30.0,
              "v": 31.9
            },
            {
              "t": 34.0,
              "v": 31.7
            },
            {
              "t": 38.0,
              "v": 31.7
            },
            {
              "t": 42.0,
              "v": 31.7
            },
            {
              "t": 46.0,
              "v": 31.4
            },
            {
              "t": 50.0,
              "v": 31.4
            },
            {
              "t": 54.0,
              "v": 31.4
            },
            {
              "t": 58.0,
              "v": 32.1
            },
            {
              "t": 62.0,
              "v": 31.6
            },
            {
              "t": 66.0,
              "v": 30.7
            },
            {
              "t": 70.0,
              "v": 30.7
            },
            {
              "t": 74.0,
              "v": 30.7
            },
            {
              "t": 78.0,
              "v": 30.8
            },
            {
              "t": 82.0,
              "v": 30.5
            },
            {
              "t": 86.0,
              "v": 32.9
            },
            {
              "t": 90.0,
              "v": 32.3
            },
            {
              "t": 94.0,
              "v": 31.1
            },
            {
              "t": 98.0,
              "v": 30.8
            },
            {
              "t": 102.0,
              "v": 30.8
            },
            {
              "t": 106.0,
              "v": 30.8
            },
            {
              "t": 110.0,
              "v": 30.8
            },
            {
              "t": 114.0,
              "v": 30.8
            },
            {
              "t": 118.0,
              "v": 30.8
            },
            {
              "t": 122.0,
              "v": 29.7
            },
            {
              "t": 126.0,
              "v": 30.0
            },
            {
              "t": 130.0,
              "v": 30.2
            },
            {
              "t": 134.0,
              "v": 30.0
            },
            {
              "t": 138.0,
              "v": 29.9
            },
            {
              "t": 142.0,
              "v": 30.2
            },
            {
              "t": 146.0,
              "v": 30.3
            },
            {
              "t": 150.0,
              "v": 30.1
            },
            {
              "t": 154.0,
              "v": 30.1
            },
            {
              "t": 158.0,
              "v": 30.1
            },
            {
              "t": 162.0,
              "v": 30.9
            },
            {
              "t": 166.0,
              "v": 32.0
            },
            {
              "t": 170.0,
              "v": 31.5
            },
            {
              "t": 174.0,
              "v": 31.1
            },
            {
              "t": 178.0,
              "v": 31.3
            },
            {
              "t": 182.0,
              "v": 31.7
            },
            {
              "t": 186.0,
              "v": 31.4
            },
            {
              "t": 190.0,
              "v": 31.7
            },
            {
              "t": 194.0,
              "v": 31.7
            },
            {
              "t": 198.0,
              "v": 31.7
            },
            {
              "t": 202.0,
              "v": 36.4
            },
            {
              "t": 206.0,
              "v": 40.3
            },
            {
              "t": 210.0,
              "v": 41.9
            },
            {
              "t": 214.0,
              "v": 42.2
            },
            {
              "t": 218.0,
              "v": 40.0
            },
            {
              "t": 222.0,
              "v": 41.5
            },
            {
              "t": 226.0,
              "v": 41.1
            }
          ]
        },
        "system": {
          "cpu": [
            {
              "t": 0.0,
              "v": 12.9
            },
            {
              "t": 2.0,
              "v": 1.9
            },
            {
              "t": 4.0,
              "v": 4.8
            },
            {
              "t": 6.0,
              "v": 8.5
            },
            {
              "t": 8.0,
              "v": 10.0
            },
            {
              "t": 10.0,
              "v": 11.1
            },
            {
              "t": 12.0,
              "v": 11.2
            },
            {
              "t": 14.0,
              "v": 12.9
            },
            {
              "t": 16.0,
              "v": 12.0
            },
            {
              "t": 18.0,
              "v": 10.6
            },
            {
              "t": 20.0,
              "v": 7.7
            },
            {
              "t": 22.0,
              "v": 3.9
            },
            {
              "t": 24.0,
              "v": 1.6
            },
            {
              "t": 26.0,
              "v": 0.4
            },
            {
              "t": 28.0,
              "v": 1.1
            },
            {
              "t": 30.0,
              "v": 3.0
            },
            {
              "t": 32.0,
              "v": 6.0
            },
            {
              "t": 34.0,
              "v": 9.0
            },
            {
              "t": 36.0,
              "v": 12.2
            },
            {
              "t": 38.0,
              "v": 13.6
            },
            {
              "t": 40.0,
              "v": 14.9
            },
            {
              "t": 42.0,
              "v": 14.2
            },
            {
              "t": 44.0,
              "v": 13.9
            },
            {
              "t": 46.0,
              "v": 10.9
            },
            {
              "t": 48.0,
              "v": 6.6
            },
            {
              "t": 50.0,
              "v": 0.2
            },
            {
              "t": 52.0,
              "v": 0.1
            },
            {
              "t": 54.0,
              "v": 0.1
            },
            {
              "t": 56.0,
              "v": 0.5
            },
            {
              "t": 58.0,
              "v": 0.4
            },
            {
              "t": 60.0,
              "v": 57.0
            },
            {
              "t": 62.0,
              "v": 66.0
            },
            {
              "t": 64.0,
              "v": 66.1
            },
            {
              "t": 66.0,
              "v": 65.5
            },
            {
              "t": 68.0,
              "v": 66.5
            },
            {
              "t": 70.0,
              "v": 66.7
            },
            {
              "t": 72.0,
              "v": 65.7
            },
            {
              "t": 74.0,
              "v": 42.2
            },
            {
              "t": 76.0,
              "v": 1.0
            },
            {
              "t": 78.0,
              "v": 0.2
            },
            {
              "t": 80.0,
              "v": 83.3
            },
            {
              "t": 82.0,
              "v": 95.0
            },
            {
              "t": 84.0,
              "v": 95.5
            },
            {
              "t": 86.0,
              "v": 95.5
            },
            {
              "t": 88.0,
              "v": 95.4
            },
            {
              "t": 90.0,
              "v": 95.3
            },
            {
              "t": 92.0,
              "v": 95.3
            },
            {
              "t": 94.0,
              "v": 59.7
            },
            {
              "t": 96.0,
              "v": 0.2
            },
            {
              "t": 98.0,
              "v": 1.5
            },
            {
              "t": 100.0,
              "v": 90.8
            },
            {
              "t": 102.0,
              "v": 100.0
            },
            {
              "t": 104.0,
              "v": 100.0
            },
            {
              "t": 106.0,
              "v": 99.9
            },
            {
              "t": 108.0,
              "v": 100.0
            },
            {
              "t": 110.0,
              "v": 99.9
            },
            {
              "t": 112.0,
              "v": 100.0
            }
          ]
        }
      }
    },
    {
      "timestamp": "2026-03-03T10:28:22.340Z",
      "commit": {
        "id": "87a4536236c374cf848c6e527398e60ddbc5871e",
        "message": "ci: harden CI \u2014 SHA pins, timeouts, sysstat sampling",
        "url": "https://github.com/locustbaby/duotunnel/commit/87a4536236c374cf848c6e527398e60ddbc5871e"
      },
      "scenarios": [
        {
          "name": "ingress_http_get",
          "protocol": "HTTP",
          "direction": "ingress",
          "category": "basic",
          "p50": 0.75,
          "p95": 1.52,
          "p99": null,
          "err": 0,
          "rps": 6.3,
          "requests": 725
        },
        {
          "name": "ingress_http_post",
          "protocol": "HTTP",
          "direction": "ingress",
          "category": "basic",
          "p50": 0.7,
          "p95": 1.61,
          "p99": null,
          "err": 0,
          "rps": 3.91,
          "requests": 450
        },
        {
          "name": "egress_http_get",
          "protocol": "HTTP",
          "direction": "egress",
          "category": "basic",
          "p50": 0.59,
          "p95": 1.23,
          "p99": null,
          "err": 0,
          "rps": 7.82,
          "requests": 900
        },
        {
          "name": "egress_http_post",
          "protocol": "HTTP",
          "direction": "egress",
          "category": "basic",
          "p50": 0.57,
          "p95": 0.78,
          "p99": null,
          "err": 0,
          "rps": 6.19,
          "requests": 712
        },
        {
          "name": "bidir_mixed",
          "protocol": "HTTP",
          "direction": "bidir",
          "category": "basic",
          "p50": 1,
          "p95": 2,
          "p99": null,
          "err": 0,
          "rps": 2.28,
          "requests": 262
        },
        {
          "name": "ingress_post_1k",
          "protocol": "HTTP",
          "direction": "ingress",
          "category": "body_size",
          "p50": 0.73,
          "p95": 1.92,
          "p99": null,
          "err": 0,
          "rps": 3.92,
          "requests": 451
        },
        {
          "name": "ingress_post_10k",
          "protocol": "HTTP",
          "direction": "ingress",
          "category": "body_size",
          "p50": 1.15,
          "p95": 2.94,
          "p99": null,
          "err": 0,
          "rps": 2.62,
          "requests": 301
        },
        {
          "name": "ingress_post_100k",
          "protocol": "HTTP",
          "direction": "ingress",
          "category": "body_size",
          "p50": 3,
          "p95": 43.09,
          "p99": null,
          "err": 0,
          "rps": 1.31,
          "requests": 151
        },
        {
          "name": "egress_post_10k",
          "protocol": "HTTP",
          "direction": "egress",
          "category": "body_size",
          "p50": 1.04,
          "p95": 2.3,
          "p99": null,
          "err": 0,
          "rps": 2.62,
          "requests": 301
        },
        {
          "name": "grpc_health_ingress",
          "protocol": "gRPC",
          "direction": "ingress",
          "category": "basic",
          "p50": 2,
          "p95": 43,
          "p99": null,
          "err": 0,
          "rps": 2.62,
          "requests": 301
        },
        {
          "name": "grpc_echo_ingress",
          "protocol": "gRPC",
          "direction": "ingress",
          "category": "basic",
          "p50": 2,
          "p95": 42,
          "p99": null,
          "err": 0,
          "rps": 2.62,
          "requests": 301
        },
        {
          "name": "grpc_large_payload",
          "protocol": "gRPC",
          "direction": "ingress",
          "category": "body_size",
          "p50": 2,
          "p95": 43,
          "p99": null,
          "err": 0,
          "rps": 1.96,
          "requests": 226
        },
        {
          "name": "grpc_high_qps",
          "protocol": "gRPC",
          "direction": "ingress",
          "category": "stress",
          "p50": 2,
          "p95": 43,
          "p99": null,
          "err": 0,
          "rps": 11.29,
          "requests": 1299
        },
        {
          "name": "ws_ingress",
          "protocol": "WS",
          "direction": "ingress",
          "category": "basic",
          "p50": 1,
          "p95": 1,
          "p99": null,
          "err": 0,
          "rps": 1.3,
          "requests": 150
        },
        {
          "name": "ws_multi_msg",
          "protocol": "WS",
          "direction": "ingress",
          "category": "basic",
          "p50": 44,
          "p95": 46,
          "p99": null,
          "err": 0,
          "rps": 0.65,
          "requests": 75
        },
        {
          "name": "ingress_1000qps",
          "protocol": "HTTP",
          "direction": "ingress",
          "category": "stress",
          "p50": 0.72,
          "p95": 1.28,
          "p99": null,
          "err": 0,
          "rps": 130.36,
          "requests": 15000
        },
        {
          "name": "egress_1000qps",
          "protocol": "HTTP",
          "direction": "egress",
          "category": "stress",
          "p50": 0.65,
          "p95": 1.14,
          "p99": null,
          "err": 0,
          "rps": 130.36,
          "requests": 15000
        },
        {
          "name": "ingress_2000qps",
          "protocol": "HTTP",
          "direction": "ingress",
          "category": "stress",
          "p50": 1.53,
          "p95": 10.17,
          "p99": null,
          "err": 0,
          "rps": 258.75,
          "requests": 29774
        },
        {
          "name": "egress_2000qps",
          "protocol": "HTTP",
          "direction": "egress",
          "category": "stress",
          "p50": 1.41,
          "p95": 10.3,
          "p99": null,
          "err": 0,
          "rps": 259.02,
          "requests": 29805
        },
        {
          "name": "ingress_3000qps",
          "protocol": "HTTP",
          "direction": "ingress",
          "category": "stress",
          "p50": 35.41,
          "p95": 82.55,
          "p99": null,
          "err": 0,
          "rps": 323.67,
          "requests": 37244
        },
        {
          "name": "egress_3000qps",
          "protocol": "HTTP",
          "direction": "egress",
          "category": "stress",
          "p50": 34.18,
          "p95": 78.34,
          "p99": null,
          "err": 3.42,
          "rps": 326.04,
          "requests": 37517
        }
      ],
      "summary": {
        "totalRPS": 1485.61,
        "totalErr": 0.75,
        "totalRequests": 170945
      },
      "scenarioConfig": {
        "getFullExecutionRequirements": {},
        "getSortedConfigs": {},
        "unmarshalJSON": {},
        "validate": {}
      },
      "resources": {
        "server": {
          "cpu": [
            {
              "t": 0.0,
              "v": 0.5
            },
            {
              "t": 4.0,
              "v": 2.5
            },
            {
              "t": 8.0,
              "v": 5.0
            },
            {
              "t": 12.0,
              "v": 9.0
            },
            {
              "t": 16.0,
              "v": 11.0
            },
            {
              "t": 20.0,
              "v": 11.0
            },
            {
              "t": 24.0,
              "v": 12.0
            },
            {
              "t": 28.0,
              "v": 12.0
            },
            {
              "t": 32.0,
              "v": 11.5
            },
            {
              "t": 36.0,
              "v": 10.5
            },
            {
              "t": 40.0,
              "v": 8.0
            },
            {
              "t": 44.0,
              "v": 5.5
            },
            {
              "t": 48.0,
              "v": 2.0
            },
            {
              "t": 52.0,
              "v": 0.0
            },
            {
              "t": 56.0,
              "v": 0.0
            },
            {
              "t": 60.0,
              "v": 3.5
            },
            {
              "t": 64.0,
              "v": 5.5
            },
            {
              "t": 68.0,
              "v": 9.0
            },
            {
              "t": 72.0,
              "v": 12.5
            },
            {
              "t": 76.0,
              "v": 15.5
            },
            {
              "t": 80.0,
              "v": 15.0
            },
            {
              "t": 84.0,
              "v": 15.5
            },
            {
              "t": 88.0,
              "v": 15.0
            },
            {
              "t": 92.0,
              "v": 13.0
            },
            {
              "t": 96.0,
              "v": 6.0
            },
            {
              "t": 100.0,
              "v": 0.0
            },
            {
              "t": 104.0,
              "v": 0.0
            },
            {
              "t": 108.0,
              "v": 0.0
            },
            {
              "t": 112.0,
              "v": 0.0
            },
            {
              "t": 116.0,
              "v": 0.0
            },
            {
              "t": 120.0,
              "v": 54.0
            },
            {
              "t": 124.0,
              "v": 64.5
            },
            {
              "t": 128.0,
              "v": 63.5
            },
            {
              "t": 132.0,
              "v": 64.0
            },
            {
              "t": 136.0,
              "v": 64.0
            },
            {
              "t": 140.0,
              "v": 64.0
            },
            {
              "t": 144.0,
              "v": 64.0
            },
            {
              "t": 148.0,
              "v": 42.5
            },
            {
              "t": 152.0,
              "v": 0.0
            },
            {
              "t": 156.0,
              "v": 0.0
            },
            {
              "t": 160.0,
              "v": 76.0
            },
            {
              "t": 164.0,
              "v": 91.5
            },
            {
              "t": 168.0,
              "v": 90.0
            },
            {
              "t": 172.0,
              "v": 92.5
            },
            {
              "t": 176.0,
              "v": 90.5
            },
            {
              "t": 180.0,
              "v": 90.5
            },
            {
              "t": 184.0,
              "v": 86.07
            },
            {
              "t": 188.0,
              "v": 43.5
            },
            {
              "t": 192.0,
              "v": 0.0
            },
            {
              "t": 196.0,
              "v": 0.0
            },
            {
              "t": 200.0,
              "v": 67.0
            },
            {
              "t": 204.0,
              "v": 79.0
            },
            {
              "t": 208.0,
              "v": 74.0
            },
            {
              "t": 212.0,
              "v": 71.0
            },
            {
              "t": 216.0,
              "v": 77.0
            },
            {
              "t": 220.0,
              "v": 82.0
            },
            {
              "t": 224.0,
              "v": 75.5
            }
          ],
          "rss": [
            {
              "t": 2.0,
              "v": 34.3
            },
            {
              "t": 6.0,
              "v": 34.3
            },
            {
              "t": 10.0,
              "v": 34.8
            },
            {
              "t": 14.0,
              "v": 35.2
            },
            {
              "t": 18.0,
              "v": 35.2
            },
            {
              "t": 22.0,
              "v": 35.2
            },
            {
              "t": 26.0,
              "v": 35.3
            },
            {
              "t": 30.0,
              "v": 34.7
            },
            {
              "t": 34.0,
              "v": 34.5
            },
            {
              "t": 38.0,
              "v": 34.2
            },
            {
              "t": 42.0,
              "v": 33.3
            },
            {
              "t": 46.0,
              "v": 33.3
            },
            {
              "t": 50.0,
              "v": 33.3
            },
            {
              "t": 54.0,
              "v": 33.3
            },
            {
              "t": 58.0,
              "v": 33.3
            },
            {
              "t": 62.0,
              "v": 33.6
            },
            {
              "t": 66.0,
              "v": 34.2
            },
            {
              "t": 70.0,
              "v": 34.7
            },
            {
              "t": 74.0,
              "v": 34.8
            },
            {
              "t": 78.0,
              "v": 35.0
            },
            {
              "t": 82.0,
              "v": 34.8
            },
            {
              "t": 86.0,
              "v": 36.0
            },
            {
              "t": 90.0,
              "v": 35.1
            },
            {
              "t": 94.0,
              "v": 35.2
            },
            {
              "t": 98.0,
              "v": 35.2
            },
            {
              "t": 102.0,
              "v": 35.2
            },
            {
              "t": 106.0,
              "v": 35.2
            },
            {
              "t": 110.0,
              "v": 35.2
            },
            {
              "t": 114.0,
              "v": 35.2
            },
            {
              "t": 118.0,
              "v": 35.2
            },
            {
              "t": 122.0,
              "v": 35.0
            },
            {
              "t": 126.0,
              "v": 37.2
            },
            {
              "t": 130.0,
              "v": 36.8
            },
            {
              "t": 134.0,
              "v": 37.0
            },
            {
              "t": 138.0,
              "v": 37.0
            },
            {
              "t": 142.0,
              "v": 36.8
            },
            {
              "t": 146.0,
              "v": 36.1
            },
            {
              "t": 150.0,
              "v": 36.5
            },
            {
              "t": 154.0,
              "v": 36.5
            },
            {
              "t": 158.0,
              "v": 36.5
            },
            {
              "t": 162.0,
              "v": 37.5
            },
            {
              "t": 166.0,
              "v": 38.0
            },
            {
              "t": 170.0,
              "v": 37.7
            },
            {
              "t": 174.0,
              "v": 38.2
            },
            {
              "t": 178.0,
              "v": 37.0
            },
            {
              "t": 182.0,
              "v": 36.5
            },
            {
              "t": 186.0,
              "v": 37.7
            },
            {
              "t": 190.0,
              "v": 38.3
            },
            {
              "t": 194.0,
              "v": 38.3
            },
            {
              "t": 198.0,
              "v": 38.3
            },
            {
              "t": 202.0,
              "v": 42.6
            },
            {
              "t": 206.0,
              "v": 45.9
            },
            {
              "t": 210.0,
              "v": 47.8
            },
            {
              "t": 214.0,
              "v": 46.8
            },
            {
              "t": 218.0,
              "v": 48.3
            },
            {
              "t": 222.0,
              "v": 53.4
            },
            {
              "t": 226.0,
              "v": 48.4
            }
          ],
          "read_kbs": [
            {
              "t": 0.0,
              "v": 0.0
            },
            {
              "t": 2.0,
              "v": 0.0
            },
            {
              "t": 4.0,
              "v": 0.0
            },
            {
              "t": 6.0,
              "v": 0.0
            },
            {
              "t": 8.0,
              "v": 0.0
            },
            {
              "t": 10.0,
              "v": 0.0
            },
            {
              "t": 12.0,
              "v": 0.0
            },
            {
              "t": 14.0,
              "v": 0.0
            },
            {
              "t": 16.0,
              "v": 0.0
            },
            {
              "t": 18.0,
              "v": 0.0
            },
            {
              "t": 20.0,
              "v": 0.0
            },
            {
              "t": 22.0,
              "v": 0.0
            },
            {
              "t": 24.0,
              "v": 0.0
            },
            {
              "t": 26.0,
              "v": 0.0
            },
            {
              "t": 28.0,
              "v": 0.0
            },
            {
              "t": 30.0,
              "v": 0.0
            },
            {
              "t": 32.0,
              "v": 0.0
            },
            {
              "t": 34.0,
              "v": 0.0
            },
            {
              "t": 36.0,
              "v": 0.0
            },
            {
              "t": 38.0,
              "v": 0.0
            },
            {
              "t": 40.0,
              "v": 0.0
            },
            {
              "t": 42.0,
              "v": 0.0
            },
            {
              "t": 44.0,
              "v": 0.0
            },
            {
              "t": 46.0,
              "v": 0.0
            },
            {
              "t": 48.0,
              "v": 0.0
            },
            {
              "t": 50.0,
              "v": 0.0
            },
            {
              "t": 52.0,
              "v": 0.0
            },
            {
              "t": 54.0,
              "v": 0.0
            },
            {
              "t": 56.0,
              "v": 0.0
            },
            {
              "t": 58.0,
              "v": 0.0
            },
            {
              "t": 60.0,
              "v": 0.0
            },
            {
              "t": 62.0,
              "v": 0.0
            },
            {
              "t": 64.0,
              "v": 0.0
            },
            {
              "t": 66.0,
              "v": 0.0
            },
            {
              "t": 68.0,
              "v": 0.0
            },
            {
              "t": 70.0,
              "v": 0.0
            },
            {
              "t": 72.0,
              "v": 0.0
            },
            {
              "t": 74.0,
              "v": 0.0
            },
            {
              "t": 76.0,
              "v": 0.0
            },
            {
              "t": 78.0,
              "v": 0.0
            },
            {
              "t": 80.0,
              "v": 0.0
            },
            {
              "t": 82.0,
              "v": 0.0
            },
            {
              "t": 84.0,
              "v": 0.0
            },
            {
              "t": 86.0,
              "v": 0.0
            },
            {
              "t": 88.0,
              "v": 0.0
            },
            {
              "t": 90.0,
              "v": 0.0
            },
            {
              "t": 92.0,
              "v": 0.0
            },
            {
              "t": 94.0,
              "v": 0.0
            },
            {
              "t": 96.0,
              "v": 0.0
            },
            {
              "t": 98.0,
              "v": 0.0
            },
            {
              "t": 100.0,
              "v": 0.0
            },
            {
              "t": 102.0,
              "v": 0.0
            },
            {
              "t": 104.0,
              "v": 0.0
            },
            {
              "t": 106.0,
              "v": 0.0
            },
            {
              "t": 108.0,
              "v": 0.0
            },
            {
              "t": 110.0,
              "v": 0.0
            },
            {
              "t": 112.0,
              "v": 0.0
            }
          ],
          "write_kbs": [
            {
              "t": 0.0,
              "v": 2.0
            },
            {
              "t": 2.0,
              "v": 8.0
            },
            {
              "t": 4.0,
              "v": 18.0
            },
            {
              "t": 6.0,
              "v": 34.0
            },
            {
              "t": 8.0,
              "v": 48.0
            },
            {
              "t": 10.0,
              "v": 58.0
            },
            {
              "t": 12.0,
              "v": 64.0
            },
            {
              "t": 14.0,
              "v": 62.0
            },
            {
              "t": 16.0,
              "v": 64.0
            },
            {
              "t": 18.0,
              "v": 62.0
            },
            {
              "t": 20.0,
              "v": 50.0
            },
            {
              "t": 22.0,
              "v": 40.0
            },
            {
              "t": 24.0,
              "v": 16.0
            },
            {
              "t": 26.0,
              "v": 2.0
            },
            {
              "t": 28.0,
              "v": 0.0
            },
            {
              "t": 30.0,
              "v": 12.0
            },
            {
              "t": 32.0,
              "v": 22.0
            },
            {
              "t": 34.0,
              "v": 26.0
            },
            {
              "t": 36.0,
              "v": 34.0
            },
            {
              "t": 38.0,
              "v": 38.0
            },
            {
              "t": 40.0,
              "v": 38.0
            },
            {
              "t": 42.0,
              "v": 40.0
            },
            {
              "t": 44.0,
              "v": 34.0
            },
            {
              "t": 46.0,
              "v": 24.0
            },
            {
              "t": 48.0,
              "v": 12.0
            },
            {
              "t": 50.0,
              "v": 0.0
            },
            {
              "t": 52.0,
              "v": 0.0
            },
            {
              "t": 54.0,
              "v": 0.0
            },
            {
              "t": 56.0,
              "v": 0.0
            },
            {
              "t": 58.0,
              "v": 0.0
            },
            {
              "t": 60.0,
              "v": 454.0
            },
            {
              "t": 62.0,
              "v": 544.0
            },
            {
              "t": 64.0,
              "v": 542.0
            },
            {
              "t": 66.0,
              "v": 542.0
            },
            {
              "t": 68.0,
              "v": 542.0
            },
            {
              "t": 70.0,
              "v": 542.0
            },
            {
              "t": 72.0,
              "v": 542.0
            },
            {
              "t": 74.0,
              "v": 358.0
            },
            {
              "t": 76.0,
              "v": 0.0
            },
            {
              "t": 78.0,
              "v": 0.0
            },
            {
              "t": 80.0,
              "v": 900.0
            },
            {
              "t": 82.0,
              "v": 1082.0
            },
            {
              "t": 84.0,
              "v": 1080.0
            },
            {
              "t": 86.0,
              "v": 1082.0
            },
            {
              "t": 88.0,
              "v": 1070.7
            },
            {
              "t": 90.0,
              "v": 1086.0
            },
            {
              "t": 92.0,
              "v": 1084.0
            },
            {
              "t": 94.0,
              "v": 686.0
            },
            {
              "t": 96.0,
              "v": 0.0
            },
            {
              "t": 98.0,
              "v": 0.0
            },
            {
              "t": 100.0,
              "v": 1202.0
            },
            {
              "t": 102.0,
              "v": 1318.0
            },
            {
              "t": 104.0,
              "v": 1276.0
            },
            {
              "t": 106.0,
              "v": 1228.0
            },
            {
              "t": 108.0,
              "v": 1326.0
            },
            {
              "t": 110.0,
              "v": 1452.0
            },
            {
              "t": 112.0,
              "v": 1284.0
            }
          ]
        },
        "client": {
          "cpu": [
            {
              "t": 0.0,
              "v": 1.0
            },
            {
              "t": 4.0,
              "v": 2.0
            },
            {
              "t": 8.0,
              "v": 4.5
            },
            {
              "t": 12.0,
              "v": 7.0
            },
            {
              "t": 16.0,
              "v": 9.5
            },
            {
              "t": 20.0,
              "v": 10.5
            },
            {
              "t": 24.0,
              "v": 11.5
            },
            {
              "t": 28.0,
              "v": 11.5
            },
            {
              "t": 32.0,
              "v": 10.0
            },
            {
              "t": 36.0,
              "v": 10.0
            },
            {
              "t": 40.0,
              "v": 8.0
            },
            {
              "t": 44.0,
              "v": 4.5
            },
            {
              "t": 48.0,
              "v": 1.5
            },
            {
              "t": 52.0,
              "v": 0.0
            },
            {
              "t": 56.0,
              "v": 0.0
            },
            {
              "t": 60.0,
              "v": 3.5
            },
            {
              "t": 64.0,
              "v": 5.5
            },
            {
              "t": 68.0,
              "v": 9.0
            },
            {
              "t": 72.0,
              "v": 11.0
            },
            {
              "t": 76.0,
              "v": 15.0
            },
            {
              "t": 80.0,
              "v": 14.5
            },
            {
              "t": 84.0,
              "v": 14.5
            },
            {
              "t": 88.0,
              "v": 14.5
            },
            {
              "t": 92.0,
              "v": 12.0
            },
            {
              "t": 96.0,
              "v": 6.5
            },
            {
              "t": 100.0,
              "v": 0.0
            },
            {
              "t": 104.0,
              "v": 0.0
            },
            {
              "t": 108.0,
              "v": 0.0
            },
            {
              "t": 112.0,
              "v": 0.0
            },
            {
              "t": 116.0,
              "v": 0.0
            },
            {
              "t": 120.0,
              "v": 53.0
            },
            {
              "t": 124.0,
              "v": 63.5
            },
            {
              "t": 128.0,
              "v": 63.0
            },
            {
              "t": 132.0,
              "v": 63.0
            },
            {
              "t": 136.0,
              "v": 63.5
            },
            {
              "t": 140.0,
              "v": 63.5
            },
            {
              "t": 144.0,
              "v": 63.5
            },
            {
              "t": 148.0,
              "v": 42.0
            },
            {
              "t": 152.0,
              "v": 0.0
            },
            {
              "t": 156.0,
              "v": 0.0
            },
            {
              "t": 160.0,
              "v": 74.0
            },
            {
              "t": 164.0,
              "v": 89.0
            },
            {
              "t": 168.0,
              "v": 88.5
            },
            {
              "t": 172.0,
              "v": 88.5
            },
            {
              "t": 176.0,
              "v": 88.5
            },
            {
              "t": 180.0,
              "v": 88.5
            },
            {
              "t": 184.0,
              "v": 83.08
            },
            {
              "t": 188.0,
              "v": 41.5
            },
            {
              "t": 192.0,
              "v": 0.0
            },
            {
              "t": 196.0,
              "v": 0.0
            },
            {
              "t": 200.0,
              "v": 61.0
            },
            {
              "t": 204.0,
              "v": 71.0
            },
            {
              "t": 208.0,
              "v": 67.5
            },
            {
              "t": 212.0,
              "v": 64.0
            },
            {
              "t": 216.0,
              "v": 71.5
            },
            {
              "t": 220.0,
              "v": 75.0
            },
            {
              "t": 224.0,
              "v": 67.5
            }
          ],
          "rss": [
            {
              "t": 2.0,
              "v": 29.6
            },
            {
              "t": 6.0,
              "v": 29.6
            },
            {
              "t": 10.0,
              "v": 30.2
            },
            {
              "t": 14.0,
              "v": 32.3
            },
            {
              "t": 18.0,
              "v": 32.3
            },
            {
              "t": 22.0,
              "v": 32.3
            },
            {
              "t": 26.0,
              "v": 32.1
            },
            {
              "t": 30.0,
              "v": 32.3
            },
            {
              "t": 34.0,
              "v": 31.7
            },
            {
              "t": 38.0,
              "v": 31.3
            },
            {
              "t": 42.0,
              "v": 31.3
            },
            {
              "t": 46.0,
              "v": 31.4
            },
            {
              "t": 50.0,
              "v": 31.4
            },
            {
              "t": 54.0,
              "v": 31.4
            },
            {
              "t": 58.0,
              "v": 32.4
            },
            {
              "t": 62.0,
              "v": 32.3
            },
            {
              "t": 66.0,
              "v": 32.3
            },
            {
              "t": 70.0,
              "v": 31.1
            },
            {
              "t": 74.0,
              "v": 32.8
            },
            {
              "t": 78.0,
              "v": 33.2
            },
            {
              "t": 82.0,
              "v": 33.3
            },
            {
              "t": 86.0,
              "v": 33.3
            },
            {
              "t": 90.0,
              "v": 32.6
            },
            {
              "t": 94.0,
              "v": 32.7
            },
            {
              "t": 98.0,
              "v": 31.9
            },
            {
              "t": 102.0,
              "v": 31.9
            },
            {
              "t": 106.0,
              "v": 31.9
            },
            {
              "t": 110.0,
              "v": 31.9
            },
            {
              "t": 114.0,
              "v": 31.9
            },
            {
              "t": 118.0,
              "v": 31.9
            },
            {
              "t": 122.0,
              "v": 30.3
            },
            {
              "t": 126.0,
              "v": 30.4
            },
            {
              "t": 130.0,
              "v": 30.5
            },
            {
              "t": 134.0,
              "v": 29.9
            },
            {
              "t": 138.0,
              "v": 30.0
            },
            {
              "t": 142.0,
              "v": 30.3
            },
            {
              "t": 146.0,
              "v": 30.1
            },
            {
              "t": 150.0,
              "v": 30.0
            },
            {
              "t": 154.0,
              "v": 30.0
            },
            {
              "t": 158.0,
              "v": 30.0
            },
            {
              "t": 162.0,
              "v": 30.9
            },
            {
              "t": 166.0,
              "v": 31.6
            },
            {
              "t": 170.0,
              "v": 31.8
            },
            {
              "t": 174.0,
              "v": 31.3
            },
            {
              "t": 178.0,
              "v": 31.1
            },
            {
              "t": 182.0,
              "v": 31.0
            },
            {
              "t": 186.0,
              "v": 31.5
            },
            {
              "t": 190.0,
              "v": 32.0
            },
            {
              "t": 194.0,
              "v": 32.0
            },
            {
              "t": 198.0,
              "v": 32.0
            },
            {
              "t": 202.0,
              "v": 35.7
            },
            {
              "t": 206.0,
              "v": 36.6
            },
            {
              "t": 210.0,
              "v": 37.5
            },
            {
              "t": 214.0,
              "v": 38.1
            },
            {
              "t": 218.0,
              "v": 38.7
            },
            {
              "t": 222.0,
              "v": 39.1
            },
            {
              "t": 226.0,
              "v": 38.2
            }
          ],
          "read_kbs": [
            {
              "t": 0.0,
              "v": 0.0
            },
            {
              "t": 2.0,
              "v": 0.0
            },
            {
              "t": 4.0,
              "v": 0.0
            },
            {
              "t": 6.0,
              "v": 0.0
            },
            {
              "t": 8.0,
              "v": 0.0
            },
            {
              "t": 10.0,
              "v": 0.0
            },
            {
              "t": 12.0,
              "v": 0.0
            },
            {
              "t": 14.0,
              "v": 0.0
            },
            {
              "t": 16.0,
              "v": 0.0
            },
            {
              "t": 18.0,
              "v": 0.0
            },
            {
              "t": 20.0,
              "v": 0.0
            },
            {
              "t": 22.0,
              "v": 0.0
            },
            {
              "t": 24.0,
              "v": 0.0
            },
            {
              "t": 26.0,
              "v": 0.0
            },
            {
              "t": 28.0,
              "v": 0.0
            },
            {
              "t": 30.0,
              "v": 0.0
            },
            {
              "t": 32.0,
              "v": 0.0
            },
            {
              "t": 34.0,
              "v": 0.0
            },
            {
              "t": 36.0,
              "v": 0.0
            },
            {
              "t": 38.0,
              "v": 0.0
            },
            {
              "t": 40.0,
              "v": 0.0
            },
            {
              "t": 42.0,
              "v": 0.0
            },
            {
              "t": 44.0,
              "v": 0.0
            },
            {
              "t": 46.0,
              "v": 0.0
            },
            {
              "t": 48.0,
              "v": 0.0
            },
            {
              "t": 50.0,
              "v": 0.0
            },
            {
              "t": 52.0,
              "v": 0.0
            },
            {
              "t": 54.0,
              "v": 0.0
            },
            {
              "t": 56.0,
              "v": 0.0
            },
            {
              "t": 58.0,
              "v": 0.0
            },
            {
              "t": 60.0,
              "v": 0.0
            },
            {
              "t": 62.0,
              "v": 0.0
            },
            {
              "t": 64.0,
              "v": 0.0
            },
            {
              "t": 66.0,
              "v": 0.0
            },
            {
              "t": 68.0,
              "v": 0.0
            },
            {
              "t": 70.0,
              "v": 0.0
            },
            {
              "t": 72.0,
              "v": 0.0
            },
            {
              "t": 74.0,
              "v": 0.0
            },
            {
              "t": 76.0,
              "v": 0.0
            },
            {
              "t": 78.0,
              "v": 0.0
            },
            {
              "t": 80.0,
              "v": 0.0
            },
            {
              "t": 82.0,
              "v": 0.0
            },
            {
              "t": 84.0,
              "v": 0.0
            },
            {
              "t": 86.0,
              "v": 0.0
            },
            {
              "t": 88.0,
              "v": 0.0
            },
            {
              "t": 90.0,
              "v": 0.0
            },
            {
              "t": 92.0,
              "v": 0.0
            },
            {
              "t": 94.0,
              "v": 0.0
            },
            {
              "t": 96.0,
              "v": 0.0
            },
            {
              "t": 98.0,
              "v": 0.0
            },
            {
              "t": 100.0,
              "v": 0.0
            },
            {
              "t": 102.0,
              "v": 0.0
            },
            {
              "t": 104.0,
              "v": 0.0
            },
            {
              "t": 106.0,
              "v": 0.0
            },
            {
              "t": 108.0,
              "v": 0.0
            },
            {
              "t": 110.0,
              "v": 0.0
            },
            {
              "t": 112.0,
              "v": 0.0
            }
          ],
          "write_kbs": [
            {
              "t": 0.0,
              "v": 4.0
            },
            {
              "t": 2.0,
              "v": 16.0
            },
            {
              "t": 4.0,
              "v": 28.0
            },
            {
              "t": 6.0,
              "v": 42.0
            },
            {
              "t": 8.0,
              "v": 48.0
            },
            {
              "t": 10.0,
              "v": 52.0
            },
            {
              "t": 12.0,
              "v": 54.0
            },
            {
              "t": 14.0,
              "v": 56.0
            },
            {
              "t": 16.0,
              "v": 54.0
            },
            {
              "t": 18.0,
              "v": 50.0
            },
            {
              "t": 20.0,
              "v": 30.0
            },
            {
              "t": 22.0,
              "v": 8.0
            },
            {
              "t": 24.0,
              "v": 0.0
            },
            {
              "t": 26.0,
              "v": 0.0
            },
            {
              "t": 28.0,
              "v": 0.0
            },
            {
              "t": 30.0,
              "v": 20.0
            },
            {
              "t": 32.0,
              "v": 32.0
            },
            {
              "t": 34.0,
              "v": 42.0
            },
            {
              "t": 36.0,
              "v": 58.0
            },
            {
              "t": 38.0,
              "v": 68.0
            },
            {
              "t": 40.0,
              "v": 72.0
            },
            {
              "t": 42.0,
              "v": 70.0
            },
            {
              "t": 44.0,
              "v": 64.0
            },
            {
              "t": 46.0,
              "v": 46.0
            },
            {
              "t": 48.0,
              "v": 26.0
            },
            {
              "t": 50.0,
              "v": 0.0
            },
            {
              "t": 52.0,
              "v": 0.0
            },
            {
              "t": 54.0,
              "v": 0.0
            },
            {
              "t": 56.0,
              "v": 0.0
            },
            {
              "t": 58.0,
              "v": 0.0
            },
            {
              "t": 60.0,
              "v": 330.0
            },
            {
              "t": 62.0,
              "v": 396.0
            },
            {
              "t": 64.0,
              "v": 392.0
            },
            {
              "t": 66.0,
              "v": 392.0
            },
            {
              "t": 68.0,
              "v": 392.0
            },
            {
              "t": 70.0,
              "v": 394.0
            },
            {
              "t": 72.0,
              "v": 392.0
            },
            {
              "t": 74.0,
              "v": 260.0
            },
            {
              "t": 76.0,
              "v": 0.0
            },
            {
              "t": 78.0,
              "v": 0.0
            },
            {
              "t": 80.0,
              "v": 648.0
            },
            {
              "t": 82.0,
              "v": 784.0
            },
            {
              "t": 84.0,
              "v": 782.0
            },
            {
              "t": 86.0,
              "v": 782.0
            },
            {
              "t": 88.0,
              "v": 776.1
            },
            {
              "t": 90.0,
              "v": 786.0
            },
            {
              "t": 92.0,
              "v": 786.0
            },
            {
              "t": 94.0,
              "v": 498.0
            },
            {
              "t": 96.0,
              "v": 0.0
            },
            {
              "t": 98.0,
              "v": 0.0
            },
            {
              "t": 100.0,
              "v": 868.0
            },
            {
              "t": 102.0,
              "v": 942.0
            },
            {
              "t": 104.0,
              "v": 912.0
            },
            {
              "t": 106.0,
              "v": 910.0
            },
            {
              "t": 108.0,
              "v": 992.0
            },
            {
              "t": 110.0,
              "v": 1074.0
            },
            {
              "t": 112.0,
              "v": 998.0
            }
          ]
        },
        "system": {
          "cpu": [
            {
              "t": 0.0,
              "v": 12.1
            },
            {
              "t": 2.0,
              "v": 2.0
            },
            {
              "t": 4.0,
              "v": 4.7
            },
            {
              "t": 6.0,
              "v": 8.7
            },
            {
              "t": 8.0,
              "v": 10.3
            },
            {
              "t": 10.0,
              "v": 11.4
            },
            {
              "t": 12.0,
              "v": 11.6
            },
            {
              "t": 14.0,
              "v": 12.3
            },
            {
              "t": 16.0,
              "v": 11.0
            },
            {
              "t": 18.0,
              "v": 10.3
            },
            {
              "t": 20.0,
              "v": 7.0
            },
            {
              "t": 22.0,
              "v": 4.8
            },
            {
              "t": 24.0,
              "v": 2.0
            },
            {
              "t": 26.0,
              "v": 0.5
            },
            {
              "t": 28.0,
              "v": 0.4
            },
            {
              "t": 30.0,
              "v": 2.8
            },
            {
              "t": 32.0,
              "v": 6.7
            },
            {
              "t": 34.0,
              "v": 8.5
            },
            {
              "t": 36.0,
              "v": 11.9
            },
            {
              "t": 38.0,
              "v": 14.5
            },
            {
              "t": 40.0,
              "v": 14.5
            },
            {
              "t": 42.0,
              "v": 15.7
            },
            {
              "t": 44.0,
              "v": 14.3
            },
            {
              "t": 46.0,
              "v": 12.2
            },
            {
              "t": 48.0,
              "v": 5.8
            },
            {
              "t": 50.0,
              "v": 0.1
            },
            {
              "t": 52.0,
              "v": 1.1
            },
            {
              "t": 54.0,
              "v": 0.2
            },
            {
              "t": 56.0,
              "v": 0.1
            },
            {
              "t": 58.0,
              "v": 0.4
            },
            {
              "t": 60.0,
              "v": 54.8
            },
            {
              "t": 62.0,
              "v": 67.0
            },
            {
              "t": 64.0,
              "v": 67.1
            },
            {
              "t": 66.0,
              "v": 66.5
            },
            {
              "t": 68.0,
              "v": 66.4
            },
            {
              "t": 70.0,
              "v": 66.7
            },
            {
              "t": 72.0,
              "v": 67.1
            },
            {
              "t": 74.0,
              "v": 43.9
            },
            {
              "t": 76.0,
              "v": 0.1
            },
            {
              "t": 78.0,
              "v": 0.2
            },
            {
              "t": 80.0,
              "v": 80.1
            },
            {
              "t": 82.0,
              "v": 95.5
            },
            {
              "t": 84.0,
              "v": 95.5
            },
            {
              "t": 86.0,
              "v": 95.7
            },
            {
              "t": 88.0,
              "v": 95.7
            },
            {
              "t": 90.0,
              "v": 95.4
            },
            {
              "t": 92.0,
              "v": 96.3
            },
            {
              "t": 94.0,
              "v": 74.1
            },
            {
              "t": 96.0,
              "v": 3.4
            },
            {
              "t": 98.0,
              "v": 0.4
            },
            {
              "t": 100.0,
              "v": 84.0
            },
            {
              "t": 102.0,
              "v": 100.0
            },
            {
              "t": 104.0,
              "v": 100.0
            },
            {
              "t": 106.0,
              "v": 100.0
            },
            {
              "t": 108.0,
              "v": 100.0
            },
            {
              "t": 110.0,
              "v": 100.0
            },
            {
              "t": 112.0,
              "v": 100.0
            }
          ]
        },
        "network": {
          "rx_kbs": [
            {
              "t": 0.0,
              "v": 10.3
            },
            {
              "t": 2.0,
              "v": 1.9
            },
            {
              "t": 4.0,
              "v": 0.3
            },
            {
              "t": 6.0,
              "v": 0.5
            },
            {
              "t": 8.0,
              "v": 2.2
            },
            {
              "t": 10.0,
              "v": 0.1
            },
            {
              "t": 12.0,
              "v": 0.1
            },
            {
              "t": 14.0,
              "v": 2.0
            },
            {
              "t": 16.0,
              "v": 0.1
            },
            {
              "t": 18.0,
              "v": 0.2
            },
            {
              "t": 20.0,
              "v": 2.1
            },
            {
              "t": 22.0,
              "v": 0.2
            },
            {
              "t": 24.0,
              "v": 0.1
            },
            {
              "t": 26.0,
              "v": 1.9
            },
            {
              "t": 28.0,
              "v": 0.1
            },
            {
              "t": 30.0,
              "v": 0.3
            },
            {
              "t": 32.0,
              "v": 2.1
            },
            {
              "t": 34.0,
              "v": 0.4
            },
            {
              "t": 36.0,
              "v": 0.2
            },
            {
              "t": 38.0,
              "v": 2.2
            },
            {
              "t": 40.0,
              "v": 0.1
            },
            {
              "t": 42.0,
              "v": 0.1
            },
            {
              "t": 44.0,
              "v": 4.5
            },
            {
              "t": 46.0,
              "v": 0.1
            },
            {
              "t": 48.0,
              "v": 0.1
            },
            {
              "t": 50.0,
              "v": 2.3
            },
            {
              "t": 52.0,
              "v": 0.2
            },
            {
              "t": 54.0,
              "v": 0.1
            },
            {
              "t": 56.0,
              "v": 2.0
            },
            {
              "t": 58.0,
              "v": 0.1
            },
            {
              "t": 60.0,
              "v": 0.3
            },
            {
              "t": 62.0,
              "v": 1.4
            },
            {
              "t": 64.0,
              "v": 0.7
            },
            {
              "t": 66.0,
              "v": 0.5
            },
            {
              "t": 68.0,
              "v": 0.5
            },
            {
              "t": 70.0,
              "v": 2.0
            },
            {
              "t": 72.0,
              "v": 0.2
            },
            {
              "t": 74.0,
              "v": 0.1
            },
            {
              "t": 76.0,
              "v": 2.0
            },
            {
              "t": 78.0,
              "v": 0.1
            },
            {
              "t": 80.0,
              "v": 0.8
            },
            {
              "t": 82.0,
              "v": 2.0
            },
            {
              "t": 84.0,
              "v": 0.1
            },
            {
              "t": 86.0,
              "v": 0.1
            },
            {
              "t": 88.0,
              "v": 1.6
            },
            {
              "t": 90.0,
              "v": 0.9
            },
            {
              "t": 92.0,
              "v": 0.1
            },
            {
              "t": 94.0,
              "v": 0.1
            },
            {
              "t": 96.0,
              "v": 2.0
            },
            {
              "t": 98.0,
              "v": 0.1
            },
            {
              "t": 100.0,
              "v": 0.1
            },
            {
              "t": 102.0,
              "v": 2.0
            },
            {
              "t": 104.0,
              "v": 2.6
            },
            {
              "t": 106.0,
              "v": 0.1
            },
            {
              "t": 108.0,
              "v": 1.8
            },
            {
              "t": 110.0,
              "v": 1.4
            },
            {
              "t": 112.0,
              "v": 0.9
            }
          ],
          "tx_kbs": [
            {
              "t": 0.0,
              "v": 45.3
            },
            {
              "t": 2.0,
              "v": 5.8
            },
            {
              "t": 4.0,
              "v": 2.0
            },
            {
              "t": 6.0,
              "v": 2.1
            },
            {
              "t": 8.0,
              "v": 7.4
            },
            {
              "t": 10.0,
              "v": 1.8
            },
            {
              "t": 12.0,
              "v": 1.7
            },
            {
              "t": 14.0,
              "v": 5.9
            },
            {
              "t": 16.0,
              "v": 1.8
            },
            {
              "t": 18.0,
              "v": 1.9
            },
            {
              "t": 20.0,
              "v": 6.2
            },
            {
              "t": 22.0,
              "v": 1.9
            },
            {
              "t": 24.0,
              "v": 1.8
            },
            {
              "t": 26.0,
              "v": 5.9
            },
            {
              "t": 28.0,
              "v": 1.8
            },
            {
              "t": 30.0,
              "v": 9.1
            },
            {
              "t": 32.0,
              "v": 6.2
            },
            {
              "t": 34.0,
              "v": 2.2
            },
            {
              "t": 36.0,
              "v": 2.0
            },
            {
              "t": 38.0,
              "v": 7.1
            },
            {
              "t": 40.0,
              "v": 1.9
            },
            {
              "t": 42.0,
              "v": 1.8
            },
            {
              "t": 44.0,
              "v": 7.5
            },
            {
              "t": 46.0,
              "v": 1.8
            },
            {
              "t": 48.0,
              "v": 1.9
            },
            {
              "t": 50.0,
              "v": 6.6
            },
            {
              "t": 52.0,
              "v": 2.0
            },
            {
              "t": 54.0,
              "v": 1.9
            },
            {
              "t": 56.0,
              "v": 6.0
            },
            {
              "t": 58.0,
              "v": 1.9
            },
            {
              "t": 60.0,
              "v": 7.9
            },
            {
              "t": 62.0,
              "v": 2.5
            },
            {
              "t": 64.0,
              "v": 5.6
            },
            {
              "t": 66.0,
              "v": 2.3
            },
            {
              "t": 68.0,
              "v": 3.5
            },
            {
              "t": 70.0,
              "v": 6.2
            },
            {
              "t": 72.0,
              "v": 2.1
            },
            {
              "t": 74.0,
              "v": 1.9
            },
            {
              "t": 76.0,
              "v": 6.1
            },
            {
              "t": 78.0,
              "v": 1.9
            },
            {
              "t": 80.0,
              "v": 3.3
            },
            {
              "t": 82.0,
              "v": 6.2
            },
            {
              "t": 84.0,
              "v": 1.9
            },
            {
              "t": 86.0,
              "v": 1.9
            },
            {
              "t": 88.0,
              "v": 3.5
            },
            {
              "t": 90.0,
              "v": 12.6
            },
            {
              "t": 92.0,
              "v": 1.9
            },
            {
              "t": 94.0,
              "v": 1.9
            },
            {
              "t": 96.0,
              "v": 6.1
            },
            {
              "t": 98.0,
              "v": 2.0
            },
            {
              "t": 100.0,
              "v": 2.0
            },
            {
              "t": 102.0,
              "v": 6.2
            },
            {
              "t": 104.0,
              "v": 3.4
            },
            {
              "t": 106.0,
              "v": 2.0
            },
            {
              "t": 108.0,
              "v": 11.1
            },
            {
              "t": 110.0,
              "v": 21.8
            },
            {
              "t": 112.0,
              "v": 30.7
            }
          ]
        }
      }
    },
    {
      "timestamp": "2026-03-03T10:40:47.088Z",
      "commit": {
        "id": "aea8477cf286b2d84ee55a8f964711c5b1eb55e5",
        "message": "ci: harden CI \u2014 SHA pins, timeouts, sysstat sampling",
        "url": "https://github.com/locustbaby/duotunnel/commit/aea8477cf286b2d84ee55a8f964711c5b1eb55e5"
      },
      "scenarios": [
        {
          "name": "ingress_http_get",
          "protocol": "HTTP",
          "direction": "ingress",
          "category": "basic",
          "p50": 0.6,
          "p95": 1.23,
          "p99": null,
          "err": 0,
          "rps": 6.3,
          "requests": 725
        },
        {
          "name": "ingress_http_post",
          "protocol": "HTTP",
          "direction": "ingress",
          "category": "basic",
          "p50": 0.59,
          "p95": 1.2,
          "p99": null,
          "err": 0,
          "rps": 3.91,
          "requests": 450
        },
        {
          "name": "egress_http_get",
          "protocol": "HTTP",
          "direction": "egress",
          "category": "basic",
          "p50": 0.5,
          "p95": 0.94,
          "p99": null,
          "err": 0,
          "rps": 7.82,
          "requests": 900
        },
        {
          "name": "egress_http_post",
          "protocol": "HTTP",
          "direction": "egress",
          "category": "basic",
          "p50": 0.48,
          "p95": 0.59,
          "p99": null,
          "err": 0,
          "rps": 6.19,
          "requests": 712
        },
        {
          "name": "bidir_mixed",
          "protocol": "HTTP",
          "direction": "bidir",
          "category": "basic",
          "p50": 1,
          "p95": 2,
          "p99": null,
          "err": 0,
          "rps": 2.28,
          "requests": 262
        },
        {
          "name": "ingress_post_1k",
          "protocol": "HTTP",
          "direction": "ingress",
          "category": "body_size",
          "p50": 0.66,
          "p95": 2.01,
          "p99": null,
          "err": 0,
          "rps": 3.91,
          "requests": 450
        },
        {
          "name": "ingress_post_10k",
          "protocol": "HTTP",
          "direction": "ingress",
          "category": "body_size",
          "p50": 1.13,
          "p95": 2.56,
          "p99": null,
          "err": 0,
          "rps": 2.62,
          "requests": 301
        },
        {
          "name": "ingress_post_100k",
          "protocol": "HTTP",
          "direction": "ingress",
          "category": "body_size",
          "p50": 2.98,
          "p95": 43.04,
          "p99": null,
          "err": 0,
          "rps": 1.31,
          "requests": 151
        },
        {
          "name": "egress_post_10k",
          "protocol": "HTTP",
          "direction": "egress",
          "category": "body_size",
          "p50": 1.09,
          "p95": 41.08,
          "p99": null,
          "err": 0,
          "rps": 2.61,
          "requests": 300
        },
        {
          "name": "grpc_health_ingress",
          "protocol": "gRPC",
          "direction": "ingress",
          "category": "basic",
          "p50": 2,
          "p95": 42,
          "p99": null,
          "err": 0,
          "rps": 2.61,
          "requests": 300
        },
        {
          "name": "grpc_echo_ingress",
          "protocol": "gRPC",
          "direction": "ingress",
          "category": "basic",
          "p50": 2,
          "p95": 42,
          "p99": null,
          "err": 0,
          "rps": 2.61,
          "requests": 300
        },
        {
          "name": "grpc_large_payload",
          "protocol": "gRPC",
          "direction": "ingress",
          "category": "body_size",
          "p50": 2,
          "p95": 43,
          "p99": null,
          "err": 0,
          "rps": 1.96,
          "requests": 225
        },
        {
          "name": "grpc_high_qps",
          "protocol": "gRPC",
          "direction": "ingress",
          "category": "stress",
          "p50": 2,
          "p95": 42,
          "p99": null,
          "err": 0,
          "rps": 11.3,
          "requests": 1300
        },
        {
          "name": "ws_ingress",
          "protocol": "WS",
          "direction": "ingress",
          "category": "basic",
          "p50": 1,
          "p95": 1,
          "p99": null,
          "err": 0,
          "rps": 1.31,
          "requests": 151
        },
        {
          "name": "ws_multi_msg",
          "protocol": "WS",
          "direction": "ingress",
          "category": "basic",
          "p50": 44,
          "p95": 45,
          "p99": null,
          "err": 0,
          "rps": 0.65,
          "requests": 75
        },
        {
          "name": "ingress_1000qps",
          "protocol": "HTTP",
          "direction": "ingress",
          "category": "stress",
          "p50": 0.7,
          "p95": 1.14,
          "p99": null,
          "err": 0,
          "rps": 130.4,
          "requests": 15001
        },
        {
          "name": "egress_1000qps",
          "protocol": "HTTP",
          "direction": "egress",
          "category": "stress",
          "p50": 0.62,
          "p95": 1.04,
          "p99": null,
          "err": 0,
          "rps": 130.4,
          "requests": 15001
        },
        {
          "name": "ingress_2000qps",
          "protocol": "HTTP",
          "direction": "ingress",
          "category": "stress",
          "p50": 1.29,
          "p95": 6.7,
          "p99": null,
          "err": 0,
          "rps": 260.08,
          "requests": 29918
        },
        {
          "name": "egress_2000qps",
          "protocol": "HTTP",
          "direction": "egress",
          "category": "stress",
          "p50": 1.18,
          "p95": 6.54,
          "p99": null,
          "err": 0,
          "rps": 260.56,
          "requests": 29973
        },
        {
          "name": "ingress_3000qps",
          "protocol": "HTTP",
          "direction": "ingress",
          "category": "stress",
          "p50": 31.62,
          "p95": 68.63,
          "p99": null,
          "err": 0,
          "rps": 337.85,
          "requests": 38865
        },
        {
          "name": "egress_3000qps",
          "protocol": "HTTP",
          "direction": "egress",
          "category": "stress",
          "p50": 32.21,
          "p95": 71.35,
          "p99": null,
          "err": 2.41,
          "rps": 335.37,
          "requests": 38579
        }
      ],
      "summary": {
        "totalRPS": 1512.06,
        "totalErr": 0.53,
        "totalRequests": 173939
      },
      "phases": [
        {
          "name": "Basic",
          "start": 0,
          "end": 23,
          "scenarios": [
            "ingress_http_get",
            "ingress_http_post",
            "egress_http_get",
            "egress_http_post",
            "ws_ingress",
            "grpc_health_ingress",
            "grpc_echo_ingress",
            "bidir_mixed"
          ]
        },
        {
          "name": "Body/Payload",
          "start": 30,
          "end": 49,
          "scenarios": [
            "ingress_post_1k",
            "ingress_post_10k",
            "ingress_post_100k",
            "egress_post_10k",
            "ws_multi_msg",
            "grpc_large_payload",
            "grpc_high_qps"
          ]
        },
        {
          "name": "1K QPS",
          "start": 60,
          "end": 75,
          "scenarios": [
            "ingress_1000qps",
            "egress_1000qps"
          ]
        },
        {
          "name": "2K QPS",
          "start": 80,
          "end": 95,
          "scenarios": [
            "ingress_2000qps",
            "egress_2000qps"
          ]
        },
        {
          "name": "3K QPS",
          "start": 100,
          "end": 115,
          "scenarios": [
            "ingress_3000qps",
            "egress_3000qps"
          ]
        }
      ],
      "resources": {
        "server": {
          "cpu": [
            {
              "t": 0.0,
              "v": 1.0
            },
            {
              "t": 4.0,
              "v": 1.5
            },
            {
              "t": 8.0,
              "v": 4.5
            },
            {
              "t": 12.0,
              "v": 7.5
            },
            {
              "t": 16.0,
              "v": 10.5
            },
            {
              "t": 20.0,
              "v": 9.0
            },
            {
              "t": 24.0,
              "v": 10.5
            },
            {
              "t": 28.0,
              "v": 10.5
            },
            {
              "t": 32.0,
              "v": 10.0
            },
            {
              "t": 36.0,
              "v": 10.0
            },
            {
              "t": 40.0,
              "v": 8.0
            },
            {
              "t": 44.0,
              "v": 5.0
            },
            {
              "t": 48.0,
              "v": 2.0
            },
            {
              "t": 52.0,
              "v": 0.0
            },
            {
              "t": 56.0,
              "v": 0.0
            },
            {
              "t": 60.0,
              "v": 3.0
            },
            {
              "t": 64.0,
              "v": 5.0
            },
            {
              "t": 68.0,
              "v": 8.0
            },
            {
              "t": 72.0,
              "v": 11.5
            },
            {
              "t": 76.0,
              "v": 13.0
            },
            {
              "t": 80.0,
              "v": 14.5
            },
            {
              "t": 84.0,
              "v": 14.5
            },
            {
              "t": 88.0,
              "v": 14.0
            },
            {
              "t": 92.0,
              "v": 11.5
            },
            {
              "t": 96.0,
              "v": 7.0
            },
            {
              "t": 100.0,
              "v": 0.0
            },
            {
              "t": 104.0,
              "v": 0.0
            },
            {
              "t": 108.0,
              "v": 0.0
            },
            {
              "t": 112.0,
              "v": 0.0
            },
            {
              "t": 116.0,
              "v": 0.0
            },
            {
              "t": 120.0,
              "v": 49.0
            },
            {
              "t": 124.0,
              "v": 61.5
            },
            {
              "t": 128.0,
              "v": 62.5
            },
            {
              "t": 132.0,
              "v": 62.0
            },
            {
              "t": 136.0,
              "v": 62.0
            },
            {
              "t": 140.0,
              "v": 62.0
            },
            {
              "t": 144.0,
              "v": 61.5
            },
            {
              "t": 148.0,
              "v": 44.0
            },
            {
              "t": 152.0,
              "v": 0.0
            },
            {
              "t": 156.0,
              "v": 0.0
            },
            {
              "t": 160.0,
              "v": 73.0
            },
            {
              "t": 164.0,
              "v": 92.5
            },
            {
              "t": 168.0,
              "v": 93.0
            },
            {
              "t": 172.0,
              "v": 91.54
            },
            {
              "t": 176.0,
              "v": 94.0
            },
            {
              "t": 180.0,
              "v": 92.5
            },
            {
              "t": 184.0,
              "v": 92.0
            },
            {
              "t": 188.0,
              "v": 63.5
            },
            {
              "t": 192.0,
              "v": 0.0
            },
            {
              "t": 196.0,
              "v": 0.0
            },
            {
              "t": 200.0,
              "v": 65.0
            },
            {
              "t": 204.0,
              "v": 73.0
            },
            {
              "t": 208.0,
              "v": 67.0
            },
            {
              "t": 212.0,
              "v": 70.5
            },
            {
              "t": 216.0,
              "v": 77.5
            },
            {
              "t": 220.0,
              "v": 79.5
            },
            {
              "t": 224.0,
              "v": 80.5
            }
          ],
          "rss": [
            {
              "t": 2.0,
              "v": 36.3
            },
            {
              "t": 6.0,
              "v": 36.3
            },
            {
              "t": 10.0,
              "v": 36.8
            },
            {
              "t": 14.0,
              "v": 37.3
            },
            {
              "t": 18.0,
              "v": 37.3
            },
            {
              "t": 22.0,
              "v": 37.1
            },
            {
              "t": 26.0,
              "v": 37.1
            },
            {
              "t": 30.0,
              "v": 36.8
            },
            {
              "t": 34.0,
              "v": 36.8
            },
            {
              "t": 38.0,
              "v": 36.8
            },
            {
              "t": 42.0,
              "v": 36.6
            },
            {
              "t": 46.0,
              "v": 36.6
            },
            {
              "t": 50.0,
              "v": 36.6
            },
            {
              "t": 54.0,
              "v": 36.0
            },
            {
              "t": 58.0,
              "v": 36.0
            },
            {
              "t": 62.0,
              "v": 35.9
            },
            {
              "t": 66.0,
              "v": 36.3
            },
            {
              "t": 70.0,
              "v": 36.4
            },
            {
              "t": 74.0,
              "v": 36.1
            },
            {
              "t": 78.0,
              "v": 36.1
            },
            {
              "t": 82.0,
              "v": 36.2
            },
            {
              "t": 86.0,
              "v": 36.1
            },
            {
              "t": 90.0,
              "v": 36.1
            },
            {
              "t": 94.0,
              "v": 36.3
            },
            {
              "t": 98.0,
              "v": 36.3
            },
            {
              "t": 102.0,
              "v": 36.3
            },
            {
              "t": 106.0,
              "v": 36.3
            },
            {
              "t": 110.0,
              "v": 36.3
            },
            {
              "t": 114.0,
              "v": 36.3
            },
            {
              "t": 118.0,
              "v": 36.3
            },
            {
              "t": 122.0,
              "v": 35.6
            },
            {
              "t": 126.0,
              "v": 34.9
            },
            {
              "t": 130.0,
              "v": 35.3
            },
            {
              "t": 134.0,
              "v": 36.0
            },
            {
              "t": 138.0,
              "v": 35.7
            },
            {
              "t": 142.0,
              "v": 35.4
            },
            {
              "t": 146.0,
              "v": 35.3
            },
            {
              "t": 150.0,
              "v": 35.6
            },
            {
              "t": 154.0,
              "v": 35.6
            },
            {
              "t": 158.0,
              "v": 35.6
            },
            {
              "t": 162.0,
              "v": 38.0
            },
            {
              "t": 166.0,
              "v": 38.1
            },
            {
              "t": 170.0,
              "v": 38.2
            },
            {
              "t": 174.0,
              "v": 37.9
            },
            {
              "t": 178.0,
              "v": 38.0
            },
            {
              "t": 182.0,
              "v": 37.4
            },
            {
              "t": 186.0,
              "v": 38.8
            },
            {
              "t": 190.0,
              "v": 39.4
            },
            {
              "t": 194.0,
              "v": 39.4
            },
            {
              "t": 198.0,
              "v": 39.4
            },
            {
              "t": 202.0,
              "v": 45.3
            },
            {
              "t": 206.0,
              "v": 47.7
            },
            {
              "t": 210.0,
              "v": 47.5
            },
            {
              "t": 214.0,
              "v": 50.9
            },
            {
              "t": 218.0,
              "v": 54.8
            },
            {
              "t": 222.0,
              "v": 54.3
            },
            {
              "t": 226.0,
              "v": 53.5
            }
          ],
          "read_kbs": [
            {
              "t": 0.0,
              "v": 0.0
            },
            {
              "t": 2.0,
              "v": 0.0
            },
            {
              "t": 4.0,
              "v": 0.0
            },
            {
              "t": 6.0,
              "v": 0.0
            },
            {
              "t": 8.0,
              "v": 0.0
            },
            {
              "t": 10.0,
              "v": 0.0
            },
            {
              "t": 12.0,
              "v": 0.0
            },
            {
              "t": 14.0,
              "v": 0.0
            },
            {
              "t": 16.0,
              "v": 0.0
            },
            {
              "t": 18.0,
              "v": 0.0
            },
            {
              "t": 20.0,
              "v": 0.0
            },
            {
              "t": 22.0,
              "v": 0.0
            },
            {
              "t": 24.0,
              "v": 0.0
            },
            {
              "t": 26.0,
              "v": 0.0
            },
            {
              "t": 28.0,
              "v": 0.0
            },
            {
              "t": 30.0,
              "v": 0.0
            },
            {
              "t": 32.0,
              "v": 0.0
            },
            {
              "t": 34.0,
              "v": 0.0
            },
            {
              "t": 36.0,
              "v": 0.0
            },
            {
              "t": 38.0,
              "v": 0.0
            },
            {
              "t": 40.0,
              "v": 0.0
            },
            {
              "t": 42.0,
              "v": 0.0
            },
            {
              "t": 44.0,
              "v": 0.0
            },
            {
              "t": 46.0,
              "v": 0.0
            },
            {
              "t": 48.0,
              "v": 0.0
            },
            {
              "t": 50.0,
              "v": 0.0
            },
            {
              "t": 52.0,
              "v": 0.0
            },
            {
              "t": 54.0,
              "v": 0.0
            },
            {
              "t": 56.0,
              "v": 0.0
            },
            {
              "t": 58.0,
              "v": 0.0
            },
            {
              "t": 60.0,
              "v": 0.0
            },
            {
              "t": 62.0,
              "v": 0.0
            },
            {
              "t": 64.0,
              "v": 0.0
            },
            {
              "t": 66.0,
              "v": 0.0
            },
            {
              "t": 68.0,
              "v": 0.0
            },
            {
              "t": 70.0,
              "v": 0.0
            },
            {
              "t": 72.0,
              "v": 0.0
            },
            {
              "t": 74.0,
              "v": 0.0
            },
            {
              "t": 76.0,
              "v": 0.0
            },
            {
              "t": 78.0,
              "v": 0.0
            },
            {
              "t": 80.0,
              "v": 0.0
            },
            {
              "t": 82.0,
              "v": 0.0
            },
            {
              "t": 84.0,
              "v": 0.0
            },
            {
              "t": 86.0,
              "v": 0.0
            },
            {
              "t": 88.0,
              "v": 0.0
            },
            {
              "t": 90.0,
              "v": 0.0
            },
            {
              "t": 92.0,
              "v": 0.0
            },
            {
              "t": 94.0,
              "v": 0.0
            },
            {
              "t": 96.0,
              "v": 0.0
            },
            {
              "t": 98.0,
              "v": 0.0
            },
            {
              "t": 100.0,
              "v": 0.0
            },
            {
              "t": 102.0,
              "v": 0.0
            },
            {
              "t": 104.0,
              "v": 0.0
            },
            {
              "t": 106.0,
              "v": 0.0
            },
            {
              "t": 108.0,
              "v": 0.0
            },
            {
              "t": 110.0,
              "v": 0.0
            },
            {
              "t": 112.0,
              "v": 0.0
            }
          ],
          "write_kbs": [
            {
              "t": 0.0,
              "v": 2.0
            },
            {
              "t": 2.0,
              "v": 8.0
            },
            {
              "t": 4.0,
              "v": 18.0
            },
            {
              "t": 6.0,
              "v": 32.0
            },
            {
              "t": 8.0,
              "v": 48.0
            },
            {
              "t": 10.0,
              "v": 58.0
            },
            {
              "t": 12.0,
              "v": 62.0
            },
            {
              "t": 14.0,
              "v": 64.0
            },
            {
              "t": 16.0,
              "v": 62.0
            },
            {
              "t": 18.0,
              "v": 62.0
            },
            {
              "t": 20.0,
              "v": 52.0
            },
            {
              "t": 22.0,
              "v": 40.0
            },
            {
              "t": 24.0,
              "v": 18.0
            },
            {
              "t": 26.0,
              "v": 2.0
            },
            {
              "t": 28.0,
              "v": 0.0
            },
            {
              "t": 30.0,
              "v": 12.0
            },
            {
              "t": 32.0,
              "v": 20.0
            },
            {
              "t": 34.0,
              "v": 26.0
            },
            {
              "t": 36.0,
              "v": 34.0
            },
            {
              "t": 38.0,
              "v": 38.0
            },
            {
              "t": 40.0,
              "v": 38.0
            },
            {
              "t": 42.0,
              "v": 40.0
            },
            {
              "t": 44.0,
              "v": 36.0
            },
            {
              "t": 46.0,
              "v": 24.0
            },
            {
              "t": 48.0,
              "v": 12.0
            },
            {
              "t": 50.0,
              "v": 0.0
            },
            {
              "t": 52.0,
              "v": 0.0
            },
            {
              "t": 54.0,
              "v": 0.0
            },
            {
              "t": 56.0,
              "v": 0.0
            },
            {
              "t": 58.0,
              "v": 0.0
            },
            {
              "t": 60.0,
              "v": 432.0
            },
            {
              "t": 62.0,
              "v": 542.0
            },
            {
              "t": 64.0,
              "v": 540.0
            },
            {
              "t": 66.0,
              "v": 544.0
            },
            {
              "t": 68.0,
              "v": 542.0
            },
            {
              "t": 70.0,
              "v": 542.0
            },
            {
              "t": 72.0,
              "v": 542.0
            },
            {
              "t": 74.0,
              "v": 382.0
            },
            {
              "t": 76.0,
              "v": 0.0
            },
            {
              "t": 78.0,
              "v": 0.0
            },
            {
              "t": 80.0,
              "v": 856.0
            },
            {
              "t": 82.0,
              "v": 1084.0
            },
            {
              "t": 84.0,
              "v": 1084.0
            },
            {
              "t": 86.0,
              "v": 1084.0
            },
            {
              "t": 88.0,
              "v": 1084.0
            },
            {
              "t": 90.0,
              "v": 1086.0
            },
            {
              "t": 92.0,
              "v": 1078.0
            },
            {
              "t": 94.0,
              "v": 762.0
            },
            {
              "t": 96.0,
              "v": 0.0
            },
            {
              "t": 98.0,
              "v": 0.0
            },
            {
              "t": 100.0,
              "v": 1182.1
            },
            {
              "t": 102.0,
              "v": 1310.0
            },
            {
              "t": 104.0,
              "v": 1226.0
            },
            {
              "t": 106.0,
              "v": 1284.0
            },
            {
              "t": 108.0,
              "v": 1388.0
            },
            {
              "t": 110.0,
              "v": 1434.0
            },
            {
              "t": 112.0,
              "v": 1446.0
            }
          ]
        },
        "client": {
          "cpu": [
            {
              "t": 0.0,
              "v": 1.0
            },
            {
              "t": 4.0,
              "v": 2.0
            },
            {
              "t": 8.0,
              "v": 4.0
            },
            {
              "t": 12.0,
              "v": 6.5
            },
            {
              "t": 16.0,
              "v": 8.5
            },
            {
              "t": 20.0,
              "v": 9.0
            },
            {
              "t": 24.0,
              "v": 10.0
            },
            {
              "t": 28.0,
              "v": 9.0
            },
            {
              "t": 32.0,
              "v": 10.0
            },
            {
              "t": 36.0,
              "v": 9.0
            },
            {
              "t": 40.0,
              "v": 7.0
            },
            {
              "t": 44.0,
              "v": 4.0
            },
            {
              "t": 48.0,
              "v": 2.0
            },
            {
              "t": 52.0,
              "v": 0.5
            },
            {
              "t": 56.0,
              "v": 0.0
            },
            {
              "t": 60.0,
              "v": 2.5
            },
            {
              "t": 64.0,
              "v": 5.5
            },
            {
              "t": 68.0,
              "v": 7.5
            },
            {
              "t": 72.0,
              "v": 10.5
            },
            {
              "t": 76.0,
              "v": 13.5
            },
            {
              "t": 80.0,
              "v": 13.5
            },
            {
              "t": 84.0,
              "v": 14.0
            },
            {
              "t": 88.0,
              "v": 13.0
            },
            {
              "t": 92.0,
              "v": 11.0
            },
            {
              "t": 96.0,
              "v": 6.5
            },
            {
              "t": 100.0,
              "v": 0.0
            },
            {
              "t": 104.0,
              "v": 0.0
            },
            {
              "t": 108.0,
              "v": 0.0
            },
            {
              "t": 112.0,
              "v": 0.0
            },
            {
              "t": 116.0,
              "v": 0.0
            },
            {
              "t": 120.0,
              "v": 48.5
            },
            {
              "t": 124.0,
              "v": 61.0
            },
            {
              "t": 128.0,
              "v": 61.5
            },
            {
              "t": 132.0,
              "v": 61.5
            },
            {
              "t": 136.0,
              "v": 62.0
            },
            {
              "t": 140.0,
              "v": 61.5
            },
            {
              "t": 144.0,
              "v": 61.5
            },
            {
              "t": 148.0,
              "v": 43.5
            },
            {
              "t": 152.0,
              "v": 0.0
            },
            {
              "t": 156.0,
              "v": 0.0
            },
            {
              "t": 160.0,
              "v": 71.0
            },
            {
              "t": 164.0,
              "v": 90.0
            },
            {
              "t": 168.0,
              "v": 91.5
            },
            {
              "t": 172.0,
              "v": 89.55
            },
            {
              "t": 176.0,
              "v": 91.0
            },
            {
              "t": 180.0,
              "v": 91.0
            },
            {
              "t": 184.0,
              "v": 91.0
            },
            {
              "t": 188.0,
              "v": 62.5
            },
            {
              "t": 192.0,
              "v": 0.0
            },
            {
              "t": 196.0,
              "v": 0.0
            },
            {
              "t": 200.0,
              "v": 58.5
            },
            {
              "t": 204.0,
              "v": 66.0
            },
            {
              "t": 208.0,
              "v": 61.0
            },
            {
              "t": 212.0,
              "v": 64.5
            },
            {
              "t": 216.0,
              "v": 70.0
            },
            {
              "t": 220.0,
              "v": 72.0
            },
            {
              "t": 224.0,
              "v": 74.5
            }
          ],
          "rss": [
            {
              "t": 2.0,
              "v": 29.5
            },
            {
              "t": 6.0,
              "v": 29.5
            },
            {
              "t": 10.0,
              "v": 32.2
            },
            {
              "t": 14.0,
              "v": 32.2
            },
            {
              "t": 18.0,
              "v": 32.2
            },
            {
              "t": 22.0,
              "v": 32.2
            },
            {
              "t": 26.0,
              "v": 32.0
            },
            {
              "t": 30.0,
              "v": 32.2
            },
            {
              "t": 34.0,
              "v": 32.1
            },
            {
              "t": 38.0,
              "v": 31.5
            },
            {
              "t": 42.0,
              "v": 31.5
            },
            {
              "t": 46.0,
              "v": 31.5
            },
            {
              "t": 50.0,
              "v": 32.3
            },
            {
              "t": 54.0,
              "v": 32.3
            },
            {
              "t": 58.0,
              "v": 32.3
            },
            {
              "t": 62.0,
              "v": 32.3
            },
            {
              "t": 66.0,
              "v": 32.3
            },
            {
              "t": 70.0,
              "v": 32.2
            },
            {
              "t": 74.0,
              "v": 31.2
            },
            {
              "t": 78.0,
              "v": 31.6
            },
            {
              "t": 82.0,
              "v": 30.9
            },
            {
              "t": 86.0,
              "v": 30.3
            },
            {
              "t": 90.0,
              "v": 32.2
            },
            {
              "t": 94.0,
              "v": 32.3
            },
            {
              "t": 98.0,
              "v": 31.9
            },
            {
              "t": 102.0,
              "v": 31.9
            },
            {
              "t": 106.0,
              "v": 31.9
            },
            {
              "t": 110.0,
              "v": 31.9
            },
            {
              "t": 114.0,
              "v": 31.9
            },
            {
              "t": 118.0,
              "v": 31.9
            },
            {
              "t": 122.0,
              "v": 31.2
            },
            {
              "t": 126.0,
              "v": 30.2
            },
            {
              "t": 130.0,
              "v": 30.1
            },
            {
              "t": 134.0,
              "v": 30.1
            },
            {
              "t": 138.0,
              "v": 29.9
            },
            {
              "t": 142.0,
              "v": 29.9
            },
            {
              "t": 146.0,
              "v": 30.0
            },
            {
              "t": 150.0,
              "v": 30.0
            },
            {
              "t": 154.0,
              "v": 30.0
            },
            {
              "t": 158.0,
              "v": 30.0
            },
            {
              "t": 162.0,
              "v": 30.6
            },
            {
              "t": 166.0,
              "v": 30.8
            },
            {
              "t": 170.0,
              "v": 30.8
            },
            {
              "t": 174.0,
              "v": 30.7
            },
            {
              "t": 178.0,
              "v": 31.0
            },
            {
              "t": 182.0,
              "v": 30.9
            },
            {
              "t": 186.0,
              "v": 31.1
            },
            {
              "t": 190.0,
              "v": 31.6
            },
            {
              "t": 194.0,
              "v": 31.6
            },
            {
              "t": 198.0,
              "v": 31.6
            },
            {
              "t": 202.0,
              "v": 35.7
            },
            {
              "t": 206.0,
              "v": 36.3
            },
            {
              "t": 210.0,
              "v": 35.8
            },
            {
              "t": 214.0,
              "v": 35.7
            },
            {
              "t": 218.0,
              "v": 42.0
            },
            {
              "t": 222.0,
              "v": 41.9
            },
            {
              "t": 226.0,
              "v": 42.4
            }
          ],
          "read_kbs": [
            {
              "t": 0.0,
              "v": 0.0
            },
            {
              "t": 2.0,
              "v": 0.0
            },
            {
              "t": 4.0,
              "v": 0.0
            },
            {
              "t": 6.0,
              "v": 0.0
            },
            {
              "t": 8.0,
              "v": 0.0
            },
            {
              "t": 10.0,
              "v": 0.0
            },
            {
              "t": 12.0,
              "v": 0.0
            },
            {
              "t": 14.0,
              "v": 0.0
            },
            {
              "t": 16.0,
              "v": 0.0
            },
            {
              "t": 18.0,
              "v": 0.0
            },
            {
              "t": 20.0,
              "v": 0.0
            },
            {
              "t": 22.0,
              "v": 0.0
            },
            {
              "t": 24.0,
              "v": 0.0
            },
            {
              "t": 26.0,
              "v": 0.0
            },
            {
              "t": 28.0,
              "v": 0.0
            },
            {
              "t": 30.0,
              "v": 0.0
            },
            {
              "t": 32.0,
              "v": 0.0
            },
            {
              "t": 34.0,
              "v": 0.0
            },
            {
              "t": 36.0,
              "v": 0.0
            },
            {
              "t": 38.0,
              "v": 0.0
            },
            {
              "t": 40.0,
              "v": 0.0
            },
            {
              "t": 42.0,
              "v": 0.0
            },
            {
              "t": 44.0,
              "v": 0.0
            },
            {
              "t": 46.0,
              "v": 0.0
            },
            {
              "t": 48.0,
              "v": 0.0
            },
            {
              "t": 50.0,
              "v": 0.0
            },
            {
              "t": 52.0,
              "v": 0.0
            },
            {
              "t": 54.0,
              "v": 0.0
            },
            {
              "t": 56.0,
              "v": 0.0
            },
            {
              "t": 58.0,
              "v": 0.0
            },
            {
              "t": 60.0,
              "v": 0.0
            },
            {
              "t": 62.0,
              "v": 0.0
            },
            {
              "t": 64.0,
              "v": 0.0
            },
            {
              "t": 66.0,
              "v": 0.0
            },
            {
              "t": 68.0,
              "v": 0.0
            },
            {
              "t": 70.0,
              "v": 0.0
            },
            {
              "t": 72.0,
              "v": 0.0
            },
            {
              "t": 74.0,
              "v": 0.0
            },
            {
              "t": 76.0,
              "v": 0.0
            },
            {
              "t": 78.0,
              "v": 0.0
            },
            {
              "t": 80.0,
              "v": 0.0
            },
            {
              "t": 82.0,
              "v": 0.0
            },
            {
              "t": 84.0,
              "v": 0.0
            },
            {
              "t": 86.0,
              "v": 0.0
            },
            {
              "t": 88.0,
              "v": 0.0
            },
            {
              "t": 90.0,
              "v": 0.0
            },
            {
              "t": 92.0,
              "v": 0.0
            },
            {
              "t": 94.0,
              "v": 0.0
            },
            {
              "t": 96.0,
              "v": 0.0
            },
            {
              "t": 98.0,
              "v": 0.0
            },
            {
              "t": 100.0,
              "v": 0.0
            },
            {
              "t": 102.0,
              "v": 0.0
            },
            {
              "t": 104.0,
              "v": 0.0
            },
            {
              "t": 106.0,
              "v": 0.0
            },
            {
              "t": 108.0,
              "v": 0.0
            },
            {
              "t": 110.0,
              "v": 0.0
            },
            {
              "t": 112.0,
              "v": 0.0
            }
          ],
          "write_kbs": [
            {
              "t": 0.0,
              "v": 4.0
            },
            {
              "t": 2.0,
              "v": 14.0
            },
            {
              "t": 4.0,
              "v": 30.0
            },
            {
              "t": 6.0,
              "v": 40.0
            },
            {
              "t": 8.0,
              "v": 48.0
            },
            {
              "t": 10.0,
              "v": 52.0
            },
            {
              "t": 12.0,
              "v": 54.0
            },
            {
              "t": 14.0,
              "v": 54.0
            },
            {
              "t": 16.0,
              "v": 56.0
            },
            {
              "t": 18.0,
              "v": 50.0
            },
            {
              "t": 20.0,
              "v": 30.0
            },
            {
              "t": 22.0,
              "v": 10.0
            },
            {
              "t": 24.0,
              "v": 0.0
            },
            {
              "t": 26.0,
              "v": 0.0
            },
            {
              "t": 28.0,
              "v": 0.0
            },
            {
              "t": 30.0,
              "v": 22.0
            },
            {
              "t": 32.0,
              "v": 28.0
            },
            {
              "t": 34.0,
              "v": 42.0
            },
            {
              "t": 36.0,
              "v": 56.0
            },
            {
              "t": 38.0,
              "v": 68.0
            },
            {
              "t": 40.0,
              "v": 72.0
            },
            {
              "t": 42.0,
              "v": 72.0
            },
            {
              "t": 44.0,
              "v": 64.0
            },
            {
              "t": 46.0,
              "v": 46.0
            },
            {
              "t": 48.0,
              "v": 28.0
            },
            {
              "t": 50.0,
              "v": 0.0
            },
            {
              "t": 52.0,
              "v": 0.0
            },
            {
              "t": 54.0,
              "v": 0.0
            },
            {
              "t": 56.0,
              "v": 0.0
            },
            {
              "t": 58.0,
              "v": 0.0
            },
            {
              "t": 60.0,
              "v": 314.0
            },
            {
              "t": 62.0,
              "v": 394.0
            },
            {
              "t": 64.0,
              "v": 392.0
            },
            {
              "t": 66.0,
              "v": 392.0
            },
            {
              "t": 68.0,
              "v": 392.0
            },
            {
              "t": 70.0,
              "v": 394.0
            },
            {
              "t": 72.0,
              "v": 392.0
            },
            {
              "t": 74.0,
              "v": 278.0
            },
            {
              "t": 76.0,
              "v": 0.0
            },
            {
              "t": 78.0,
              "v": 0.0
            },
            {
              "t": 80.0,
              "v": 616.0
            },
            {
              "t": 82.0,
              "v": 784.0
            },
            {
              "t": 84.0,
              "v": 786.0
            },
            {
              "t": 86.0,
              "v": 786.0
            },
            {
              "t": 88.0,
              "v": 784.0
            },
            {
              "t": 90.0,
              "v": 788.0
            },
            {
              "t": 92.0,
              "v": 776.0
            },
            {
              "t": 94.0,
              "v": 554.0
            },
            {
              "t": 96.0,
              "v": 0.0
            },
            {
              "t": 98.0,
              "v": 0.0
            },
            {
              "t": 100.0,
              "v": 867.7
            },
            {
              "t": 102.0,
              "v": 948.0
            },
            {
              "t": 104.0,
              "v": 912.0
            },
            {
              "t": 106.0,
              "v": 938.0
            },
            {
              "t": 108.0,
              "v": 1030.0
            },
            {
              "t": 110.0,
              "v": 1078.0
            },
            {
              "t": 112.0,
              "v": 1120.0
            }
          ]
        },
        "system": {
          "cpu": [
            {
              "t": 0.0,
              "v": 14.0
            },
            {
              "t": 2.0,
              "v": 3.3
            },
            {
              "t": 4.0,
              "v": 3.6
            },
            {
              "t": 6.0,
              "v": 6.3
            },
            {
              "t": 8.0,
              "v": 11.2
            },
            {
              "t": 10.0,
              "v": 10.0
            },
            {
              "t": 12.0,
              "v": 9.6
            },
            {
              "t": 14.0,
              "v": 9.9
            },
            {
              "t": 16.0,
              "v": 9.2
            },
            {
              "t": 18.0,
              "v": 9.5
            },
            {
              "t": 20.0,
              "v": 7.5
            },
            {
              "t": 22.0,
              "v": 4.6
            },
            {
              "t": 24.0,
              "v": 1.8
            },
            {
              "t": 26.0,
              "v": 0.6
            },
            {
              "t": 28.0,
              "v": 0.5
            },
            {
              "t": 30.0,
              "v": 3.8
            },
            {
              "t": 32.0,
              "v": 4.9
            },
            {
              "t": 34.0,
              "v": 8.4
            },
            {
              "t": 36.0,
              "v": 10.0
            },
            {
              "t": 38.0,
              "v": 12.8
            },
            {
              "t": 40.0,
              "v": 12.9
            },
            {
              "t": 42.0,
              "v": 14.2
            },
            {
              "t": 44.0,
              "v": 13.5
            },
            {
              "t": 46.0,
              "v": 9.9
            },
            {
              "t": 48.0,
              "v": 6.2
            },
            {
              "t": 50.0,
              "v": 0.5
            },
            {
              "t": 52.0,
              "v": 0.4
            },
            {
              "t": 54.0,
              "v": 0.1
            },
            {
              "t": 56.0,
              "v": 0.2
            },
            {
              "t": 58.0,
              "v": 1.0
            },
            {
              "t": 60.0,
              "v": 48.4
            },
            {
              "t": 62.0,
              "v": 61.5
            },
            {
              "t": 64.0,
              "v": 61.1
            },
            {
              "t": 66.0,
              "v": 61.6
            },
            {
              "t": 68.0,
              "v": 63.0
            },
            {
              "t": 70.0,
              "v": 61.7
            },
            {
              "t": 72.0,
              "v": 61.5
            },
            {
              "t": 74.0,
              "v": 43.0
            },
            {
              "t": 76.0,
              "v": 0.1
            },
            {
              "t": 78.0,
              "v": 0.2
            },
            {
              "t": 80.0,
              "v": 75.1
            },
            {
              "t": 82.0,
              "v": 95.0
            },
            {
              "t": 84.0,
              "v": 94.3
            },
            {
              "t": 86.0,
              "v": 94.0
            },
            {
              "t": 88.0,
              "v": 94.3
            },
            {
              "t": 90.0,
              "v": 94.7
            },
            {
              "t": 92.0,
              "v": 94.3
            },
            {
              "t": 94.0,
              "v": 66.6
            },
            {
              "t": 96.0,
              "v": 0.2
            },
            {
              "t": 98.0,
              "v": 0.1
            },
            {
              "t": 100.0,
              "v": 79.5
            },
            {
              "t": 102.0,
              "v": 99.6
            },
            {
              "t": 104.0,
              "v": 100.0
            },
            {
              "t": 106.0,
              "v": 100.0
            },
            {
              "t": 108.0,
              "v": 100.0
            },
            {
              "t": 110.0,
              "v": 100.0
            },
            {
              "t": 112.0,
              "v": 100.0
            }
          ]
        },
        "k6OffsetSeconds": 0,
        "network": {
          "rx_kbs": [
            {
              "t": 0.0,
              "v": 9.0
            },
            {
              "t": 2.0,
              "v": 1.5
            },
            {
              "t": 4.0,
              "v": 2.2
            },
            {
              "t": 6.0,
              "v": 0.1
            },
            {
              "t": 8.0,
              "v": 0.1
            },
            {
              "t": 10.0,
              "v": 2.2
            },
            {
              "t": 12.0,
              "v": 0.1
            },
            {
              "t": 14.0,
              "v": 0.1
            },
            {
              "t": 16.0,
              "v": 1.9
            },
            {
              "t": 18.0,
              "v": 0.3
            },
            {
              "t": 20.0,
              "v": 0.4
            },
            {
              "t": 22.0,
              "v": 2.0
            },
            {
              "t": 24.0,
              "v": 0.1
            },
            {
              "t": 26.0,
              "v": 0.1
            },
            {
              "t": 28.0,
              "v": 2.0
            },
            {
              "t": 30.0,
              "v": 0.1
            },
            {
              "t": 32.0,
              "v": 0.1
            },
            {
              "t": 34.0,
              "v": 2.0
            },
            {
              "t": 36.0,
              "v": 0.1
            },
            {
              "t": 38.0,
              "v": 0.1
            },
            {
              "t": 40.0,
              "v": 2.2
            },
            {
              "t": 42.0,
              "v": 0.1
            },
            {
              "t": 44.0,
              "v": 0.1
            },
            {
              "t": 46.0,
              "v": 2.0
            },
            {
              "t": 48.0,
              "v": 0.1
            },
            {
              "t": 50.0,
              "v": 0.3
            },
            {
              "t": 52.0,
              "v": 2.0
            },
            {
              "t": 54.0,
              "v": 0.1
            },
            {
              "t": 56.0,
              "v": 0.1
            },
            {
              "t": 58.0,
              "v": 4.5
            },
            {
              "t": 60.0,
              "v": 0.1
            },
            {
              "t": 62.0,
              "v": 0.1
            },
            {
              "t": 64.0,
              "v": 2.2
            },
            {
              "t": 66.0,
              "v": 0.1
            },
            {
              "t": 68.0,
              "v": 0.1
            },
            {
              "t": 70.0,
              "v": 2.1
            },
            {
              "t": 72.0,
              "v": 0.2
            },
            {
              "t": 74.0,
              "v": 0.1
            },
            {
              "t": 76.0,
              "v": 1.9
            },
            {
              "t": 78.0,
              "v": 0.3
            },
            {
              "t": 80.0,
              "v": 0.3
            },
            {
              "t": 82.0,
              "v": 2.0
            },
            {
              "t": 84.0,
              "v": 0.1
            },
            {
              "t": 86.0,
              "v": 0.1
            },
            {
              "t": 88.0,
              "v": 1.9
            },
            {
              "t": 90.0,
              "v": 0.1
            },
            {
              "t": 92.0,
              "v": 0.1
            },
            {
              "t": 94.0,
              "v": 2.1
            },
            {
              "t": 96.0,
              "v": 0.1
            },
            {
              "t": 98.0,
              "v": 0.1
            },
            {
              "t": 100.0,
              "v": 2.3
            },
            {
              "t": 102.0,
              "v": 0.1
            },
            {
              "t": 104.0,
              "v": 0.1
            },
            {
              "t": 106.0,
              "v": 1.9
            },
            {
              "t": 108.0,
              "v": 0.6
            },
            {
              "t": 110.0,
              "v": 0.7
            },
            {
              "t": 112.0,
              "v": 2.7
            }
          ],
          "tx_kbs": [
            {
              "t": 0.0,
              "v": 27.3
            },
            {
              "t": 2.0,
              "v": 16.9
            },
            {
              "t": 4.0,
              "v": 6.3
            },
            {
              "t": 6.0,
              "v": 1.7
            },
            {
              "t": 8.0,
              "v": 1.7
            },
            {
              "t": 10.0,
              "v": 17.7
            },
            {
              "t": 12.0,
              "v": 1.8
            },
            {
              "t": 14.0,
              "v": 1.7
            },
            {
              "t": 16.0,
              "v": 5.9
            },
            {
              "t": 18.0,
              "v": 1.9
            },
            {
              "t": 20.0,
              "v": 3.3
            },
            {
              "t": 22.0,
              "v": 6.0
            },
            {
              "t": 24.0,
              "v": 1.8
            },
            {
              "t": 26.0,
              "v": 1.8
            },
            {
              "t": 28.0,
              "v": 6.0
            },
            {
              "t": 30.0,
              "v": 1.8
            },
            {
              "t": 32.0,
              "v": 1.8
            },
            {
              "t": 34.0,
              "v": 6.0
            },
            {
              "t": 36.0,
              "v": 1.8
            },
            {
              "t": 38.0,
              "v": 1.9
            },
            {
              "t": 40.0,
              "v": 13.1
            },
            {
              "t": 42.0,
              "v": 1.8
            },
            {
              "t": 44.0,
              "v": 1.9
            },
            {
              "t": 46.0,
              "v": 6.0
            },
            {
              "t": 48.0,
              "v": 1.9
            },
            {
              "t": 50.0,
              "v": 2.9
            },
            {
              "t": 52.0,
              "v": 6.1
            },
            {
              "t": 54.0,
              "v": 1.9
            },
            {
              "t": 56.0,
              "v": 1.9
            },
            {
              "t": 58.0,
              "v": 7.4
            },
            {
              "t": 60.0,
              "v": 1.9
            },
            {
              "t": 62.0,
              "v": 1.9
            },
            {
              "t": 64.0,
              "v": 6.5
            },
            {
              "t": 66.0,
              "v": 1.9
            },
            {
              "t": 68.0,
              "v": 1.9
            },
            {
              "t": 70.0,
              "v": 12.1
            },
            {
              "t": 72.0,
              "v": 2.1
            },
            {
              "t": 74.0,
              "v": 1.9
            },
            {
              "t": 76.0,
              "v": 6.1
            },
            {
              "t": 78.0,
              "v": 2.1
            },
            {
              "t": 80.0,
              "v": 3.5
            },
            {
              "t": 82.0,
              "v": 6.2
            },
            {
              "t": 84.0,
              "v": 2.0
            },
            {
              "t": 86.0,
              "v": 1.9
            },
            {
              "t": 88.0,
              "v": 6.1
            },
            {
              "t": 90.0,
              "v": 2.1
            },
            {
              "t": 92.0,
              "v": 1.9
            },
            {
              "t": 94.0,
              "v": 6.4
            },
            {
              "t": 96.0,
              "v": 2.0
            },
            {
              "t": 98.0,
              "v": 2.0
            },
            {
              "t": 100.0,
              "v": 13.7
            },
            {
              "t": 102.0,
              "v": 2.0
            },
            {
              "t": 104.0,
              "v": 2.0
            },
            {
              "t": 106.0,
              "v": 6.2
            },
            {
              "t": 108.0,
              "v": 13.3
            },
            {
              "t": 110.0,
              "v": 18.5
            },
            {
              "t": 112.0,
              "v": 25.7
            }
          ]
        }
      }
    },
    {
      "timestamp": "2026-03-03T12:27:03.929Z",
      "commit": {
        "id": "517f35d7ddb0b2d6b1cb92f854fe4cc4b8843993",
        "message": "ci: harden CI \u2014 SHA pins, timeouts, sysstat sampling",
        "url": "https://github.com/locustbaby/duotunnel/commit/517f35d7ddb0b2d6b1cb92f854fe4cc4b8843993"
      },
      "scenarios": [
        {
          "name": "ingress_http_get",
          "protocol": "HTTP",
          "direction": "ingress",
          "category": "basic",
          "p50": 0.59,
          "p95": 1.12,
          "p99": null,
          "err": 0,
          "rps": 6.3,
          "requests": 725
        },
        {
          "name": "ingress_http_post",
          "protocol": "HTTP",
          "direction": "ingress",
          "category": "basic",
          "p50": 0.59,
          "p95": 1.23,
          "p99": null,
          "err": 0,
          "rps": 3.9,
          "requests": 449
        },
        {
          "name": "egress_http_get",
          "protocol": "HTTP",
          "direction": "egress",
          "category": "basic",
          "p50": 0.5,
          "p95": 0.91,
          "p99": null,
          "err": 0,
          "rps": 7.81,
          "requests": 899
        },
        {
          "name": "egress_http_post",
          "protocol": "HTTP",
          "direction": "egress",
          "category": "basic",
          "p50": 0.48,
          "p95": 0.6,
          "p99": null,
          "err": 0,
          "rps": 6.19,
          "requests": 712
        },
        {
          "name": "bidir_mixed",
          "protocol": "HTTP",
          "direction": "bidir",
          "category": "basic",
          "p50": 2,
          "p95": 2,
          "p99": null,
          "err": 0,
          "rps": 2.28,
          "requests": 262
        },
        {
          "name": "ingress_post_1k",
          "protocol": "HTTP",
          "direction": "ingress",
          "category": "body_size",
          "p50": 0.65,
          "p95": 1.95,
          "p99": null,
          "err": 0,
          "rps": 3.92,
          "requests": 451
        },
        {
          "name": "ingress_post_10k",
          "protocol": "HTTP",
          "direction": "ingress",
          "category": "body_size",
          "p50": 1.12,
          "p95": 2.54,
          "p99": null,
          "err": 0,
          "rps": 2.62,
          "requests": 301
        },
        {
          "name": "ingress_post_100k",
          "protocol": "HTTP",
          "direction": "ingress",
          "category": "body_size",
          "p50": 3.07,
          "p95": 42.96,
          "p99": null,
          "err": 0,
          "rps": 1.31,
          "requests": 151
        },
        {
          "name": "egress_post_10k",
          "protocol": "HTTP",
          "direction": "egress",
          "category": "body_size",
          "p50": 0.99,
          "p95": 2.27,
          "p99": null,
          "err": 0,
          "rps": 2.62,
          "requests": 301
        },
        {
          "name": "grpc_health_ingress",
          "protocol": "gRPC",
          "direction": "ingress",
          "category": "basic",
          "p50": 2,
          "p95": 43,
          "p99": null,
          "err": 0,
          "rps": 2.62,
          "requests": 301
        },
        {
          "name": "grpc_echo_ingress",
          "protocol": "gRPC",
          "direction": "ingress",
          "category": "basic",
          "p50": 2,
          "p95": 42,
          "p99": null,
          "err": 0,
          "rps": 2.62,
          "requests": 301
        },
        {
          "name": "grpc_large_payload",
          "protocol": "gRPC",
          "direction": "ingress",
          "category": "body_size",
          "p50": 2,
          "p95": 43,
          "p99": null,
          "err": 0,
          "rps": 1.96,
          "requests": 226
        },
        {
          "name": "grpc_high_qps",
          "protocol": "gRPC",
          "direction": "ingress",
          "category": "stress",
          "p50": 2,
          "p95": 42,
          "p99": null,
          "err": 0,
          "rps": 11.3,
          "requests": 1300
        },
        {
          "name": "ws_ingress",
          "protocol": "WS",
          "direction": "ingress",
          "category": "basic",
          "p50": 1,
          "p95": 1,
          "p99": null,
          "err": 0,
          "rps": 1.31,
          "requests": 151
        },
        {
          "name": "ws_multi_msg",
          "protocol": "WS",
          "direction": "ingress",
          "category": "basic",
          "p50": 44,
          "p95": 45.25,
          "p99": null,
          "err": 0,
          "rps": 0.66,
          "requests": 76
        },
        {
          "name": "ingress_1000qps",
          "protocol": "HTTP",
          "direction": "ingress",
          "category": "stress",
          "p50": 0.71,
          "p95": 1.18,
          "p99": null,
          "err": 0,
          "rps": 130.38,
          "requests": 15001
        },
        {
          "name": "egress_1000qps",
          "protocol": "HTTP",
          "direction": "egress",
          "category": "stress",
          "p50": 0.64,
          "p95": 1.06,
          "p99": null,
          "err": 0,
          "rps": 130.38,
          "requests": 15001
        },
        {
          "name": "ingress_2000qps",
          "protocol": "HTTP",
          "direction": "ingress",
          "category": "stress",
          "p50": 1.39,
          "p95": 8.35,
          "p99": null,
          "err": 0,
          "rps": 259.93,
          "requests": 29906
        },
        {
          "name": "egress_2000qps",
          "protocol": "HTTP",
          "direction": "egress",
          "category": "stress",
          "p50": 1.27,
          "p95": 8.1,
          "p99": null,
          "err": 0,
          "rps": 259.84,
          "requests": 29895
        },
        {
          "name": "ingress_3000qps",
          "protocol": "HTTP",
          "direction": "ingress",
          "category": "stress",
          "p50": 28.99,
          "p95": 75.89,
          "p99": null,
          "err": 0,
          "rps": 329.46,
          "requests": 37905
        },
        {
          "name": "egress_3000qps",
          "protocol": "HTTP",
          "direction": "egress",
          "category": "stress",
          "p50": 27.84,
          "p95": 75.25,
          "p99": null,
          "err": 2.08,
          "rps": 327.72,
          "requests": 37705
        }
      ],
      "summary": {
        "totalRPS": 1495.13,
        "totalErr": 0.46,
        "totalRequests": 172019
      },
      "phases": [
        {
          "name": "Basic",
          "start": 0,
          "end": 23,
          "scenarios": [
            "ingress_http_get",
            "ingress_http_post",
            "egress_http_get",
            "egress_http_post",
            "ws_ingress",
            "grpc_health_ingress",
            "grpc_echo_ingress",
            "bidir_mixed"
          ]
        },
        {
          "name": "Body/Payload",
          "start": 30,
          "end": 49,
          "scenarios": [
            "ingress_post_1k",
            "ingress_post_10k",
            "ingress_post_100k",
            "egress_post_10k",
            "ws_multi_msg",
            "grpc_large_payload",
            "grpc_high_qps"
          ]
        },
        {
          "name": "1K QPS",
          "start": 60,
          "end": 75,
          "scenarios": [
            "ingress_1000qps",
            "egress_1000qps"
          ]
        },
        {
          "name": "2K QPS",
          "start": 80,
          "end": 95,
          "scenarios": [
            "ingress_2000qps",
            "egress_2000qps"
          ]
        },
        {
          "name": "3K QPS",
          "start": 100,
          "end": 115,
          "scenarios": [
            "ingress_3000qps",
            "egress_3000qps"
          ]
        }
      ],
      "resources": {
        "server": {
          "cpu": [
            {
              "t": 2.0,
              "v": 1.0
            },
            {
              "t": 6.0,
              "v": 2.0
            },
            {
              "t": 10.0,
              "v": 4.5
            },
            {
              "t": 14.0,
              "v": 7.5
            },
            {
              "t": 18.0,
              "v": 10.0
            },
            {
              "t": 22.0,
              "v": 10.0
            },
            {
              "t": 26.0,
              "v": 10.5
            },
            {
              "t": 30.0,
              "v": 10.0
            },
            {
              "t": 34.0,
              "v": 10.5
            },
            {
              "t": 38.0,
              "v": 10.0
            },
            {
              "t": 42.0,
              "v": 7.5
            },
            {
              "t": 46.0,
              "v": 4.5
            },
            {
              "t": 50.0,
              "v": 2.0
            },
            {
              "t": 54.0,
              "v": 0.0
            },
            {
              "t": 58.0,
              "v": 0.0
            },
            {
              "t": 62.0,
              "v": 3.0
            },
            {
              "t": 66.0,
              "v": 5.0
            },
            {
              "t": 70.0,
              "v": 8.5
            },
            {
              "t": 74.0,
              "v": 11.5
            },
            {
              "t": 78.0,
              "v": 14.0
            },
            {
              "t": 82.0,
              "v": 14.5
            },
            {
              "t": 86.0,
              "v": 14.0
            },
            {
              "t": 90.0,
              "v": 13.5
            },
            {
              "t": 94.0,
              "v": 11.0
            },
            {
              "t": 98.0,
              "v": 6.5
            },
            {
              "t": 102.0,
              "v": 0.0
            },
            {
              "t": 106.0,
              "v": 0.0
            },
            {
              "t": 110.0,
              "v": 0.0
            },
            {
              "t": 114.0,
              "v": 0.0
            },
            {
              "t": 118.0,
              "v": 0.0
            },
            {
              "t": 122.0,
              "v": 53.0
            },
            {
              "t": 126.0,
              "v": 62.5
            },
            {
              "t": 130.0,
              "v": 62.5
            },
            {
              "t": 134.0,
              "v": 63.0
            },
            {
              "t": 138.0,
              "v": 63.0
            },
            {
              "t": 142.0,
              "v": 63.5
            },
            {
              "t": 146.0,
              "v": 62.5
            },
            {
              "t": 150.0,
              "v": 41.0
            },
            {
              "t": 154.0,
              "v": 0.0
            },
            {
              "t": 158.0,
              "v": 0.0
            },
            {
              "t": 162.0,
              "v": 77.5
            },
            {
              "t": 166.0,
              "v": 90.5
            },
            {
              "t": 170.0,
              "v": 91.0
            },
            {
              "t": 174.0,
              "v": 90.0
            },
            {
              "t": 178.0,
              "v": 89.5
            },
            {
              "t": 182.0,
              "v": 90.0
            },
            {
              "t": 186.0,
              "v": 90.5
            },
            {
              "t": 190.0,
              "v": 59.0
            },
            {
              "t": 194.0,
              "v": 0.0
            },
            {
              "t": 198.0,
              "v": 0.0
            },
            {
              "t": 202.0,
              "v": 67.66
            },
            {
              "t": 206.0,
              "v": 81.0
            },
            {
              "t": 210.0,
              "v": 80.5
            },
            {
              "t": 214.0,
              "v": 80.5
            },
            {
              "t": 218.0,
              "v": 79.0
            },
            {
              "t": 222.0,
              "v": 66.5
            },
            {
              "t": 226.0,
              "v": 66.5
            }
          ],
          "rss": [
            {
              "t": 4.0,
              "v": 34.4
            },
            {
              "t": 8.0,
              "v": 34.4
            },
            {
              "t": 12.0,
              "v": 34.9
            },
            {
              "t": 16.0,
              "v": 35.2
            },
            {
              "t": 20.0,
              "v": 35.1
            },
            {
              "t": 24.0,
              "v": 35.0
            },
            {
              "t": 28.0,
              "v": 34.8
            },
            {
              "t": 32.0,
              "v": 34.0
            },
            {
              "t": 36.0,
              "v": 34.0
            },
            {
              "t": 40.0,
              "v": 34.3
            },
            {
              "t": 44.0,
              "v": 34.3
            },
            {
              "t": 48.0,
              "v": 34.3
            },
            {
              "t": 52.0,
              "v": 34.3
            },
            {
              "t": 56.0,
              "v": 34.3
            },
            {
              "t": 60.0,
              "v": 34.3
            },
            {
              "t": 64.0,
              "v": 34.3
            },
            {
              "t": 68.0,
              "v": 34.6
            },
            {
              "t": 72.0,
              "v": 34.3
            },
            {
              "t": 76.0,
              "v": 33.8
            },
            {
              "t": 80.0,
              "v": 33.9
            },
            {
              "t": 84.0,
              "v": 34.3
            },
            {
              "t": 88.0,
              "v": 34.0
            },
            {
              "t": 92.0,
              "v": 33.9
            },
            {
              "t": 96.0,
              "v": 34.1
            },
            {
              "t": 100.0,
              "v": 33.6
            },
            {
              "t": 104.0,
              "v": 33.6
            },
            {
              "t": 108.0,
              "v": 33.6
            },
            {
              "t": 112.0,
              "v": 33.6
            },
            {
              "t": 116.0,
              "v": 33.6
            },
            {
              "t": 120.0,
              "v": 33.6
            },
            {
              "t": 124.0,
              "v": 32.8
            },
            {
              "t": 128.0,
              "v": 32.5
            },
            {
              "t": 132.0,
              "v": 33.2
            },
            {
              "t": 136.0,
              "v": 33.7
            },
            {
              "t": 140.0,
              "v": 33.3
            },
            {
              "t": 144.0,
              "v": 32.6
            },
            {
              "t": 148.0,
              "v": 32.9
            },
            {
              "t": 152.0,
              "v": 33.2
            },
            {
              "t": 156.0,
              "v": 33.2
            },
            {
              "t": 160.0,
              "v": 33.2
            },
            {
              "t": 164.0,
              "v": 37.2
            },
            {
              "t": 168.0,
              "v": 37.8
            },
            {
              "t": 172.0,
              "v": 38.1
            },
            {
              "t": 176.0,
              "v": 37.2
            },
            {
              "t": 180.0,
              "v": 37.8
            },
            {
              "t": 184.0,
              "v": 37.8
            },
            {
              "t": 188.0,
              "v": 37.7
            },
            {
              "t": 192.0,
              "v": 37.5
            },
            {
              "t": 196.0,
              "v": 37.5
            },
            {
              "t": 200.0,
              "v": 37.5
            },
            {
              "t": 204.0,
              "v": 41.0
            },
            {
              "t": 208.0,
              "v": 42.6
            },
            {
              "t": 212.0,
              "v": 43.6
            },
            {
              "t": 216.0,
              "v": 48.0
            },
            {
              "t": 220.0,
              "v": 49.5
            },
            {
              "t": 224.0,
              "v": 48.5
            },
            {
              "t": 228.0,
              "v": 51.2
            }
          ],
          "read_kbs": [
            {
              "t": 2.0,
              "v": 0.0
            },
            {
              "t": 4.0,
              "v": 0.0
            },
            {
              "t": 6.0,
              "v": 0.0
            },
            {
              "t": 8.0,
              "v": 0.0
            },
            {
              "t": 10.0,
              "v": 0.0
            },
            {
              "t": 12.0,
              "v": 0.0
            },
            {
              "t": 14.0,
              "v": 0.0
            },
            {
              "t": 16.0,
              "v": 0.0
            },
            {
              "t": 18.0,
              "v": 0.0
            },
            {
              "t": 20.0,
              "v": 0.0
            },
            {
              "t": 22.0,
              "v": 0.0
            },
            {
              "t": 24.0,
              "v": 0.0
            },
            {
              "t": 26.0,
              "v": 0.0
            },
            {
              "t": 28.0,
              "v": 0.0
            },
            {
              "t": 30.0,
              "v": 0.0
            },
            {
              "t": 32.0,
              "v": 0.0
            },
            {
              "t": 34.0,
              "v": 0.0
            },
            {
              "t": 36.0,
              "v": 0.0
            },
            {
              "t": 38.0,
              "v": 0.0
            },
            {
              "t": 40.0,
              "v": 0.0
            },
            {
              "t": 42.0,
              "v": 0.0
            },
            {
              "t": 44.0,
              "v": 0.0
            },
            {
              "t": 46.0,
              "v": 0.0
            },
            {
              "t": 48.0,
              "v": 0.0
            },
            {
              "t": 50.0,
              "v": 0.0
            },
            {
              "t": 52.0,
              "v": 0.0
            },
            {
              "t": 54.0,
              "v": 0.0
            },
            {
              "t": 56.0,
              "v": 0.0
            },
            {
              "t": 58.0,
              "v": 0.0
            },
            {
              "t": 60.0,
              "v": 0.0
            },
            {
              "t": 62.0,
              "v": 0.0
            },
            {
              "t": 64.0,
              "v": 0.0
            },
            {
              "t": 66.0,
              "v": 0.0
            },
            {
              "t": 68.0,
              "v": 0.0
            },
            {
              "t": 70.0,
              "v": 0.0
            },
            {
              "t": 72.0,
              "v": 0.0
            },
            {
              "t": 74.0,
              "v": 0.0
            },
            {
              "t": 76.0,
              "v": 0.0
            },
            {
              "t": 78.0,
              "v": 0.0
            },
            {
              "t": 80.0,
              "v": 0.0
            },
            {
              "t": 82.0,
              "v": 0.0
            },
            {
              "t": 84.0,
              "v": 0.0
            },
            {
              "t": 86.0,
              "v": 0.0
            },
            {
              "t": 88.0,
              "v": 0.0
            },
            {
              "t": 90.0,
              "v": 0.0
            },
            {
              "t": 92.0,
              "v": 0.0
            },
            {
              "t": 94.0,
              "v": 0.0
            },
            {
              "t": 96.0,
              "v": 0.0
            },
            {
              "t": 98.0,
              "v": 0.0
            },
            {
              "t": 100.0,
              "v": 0.0
            },
            {
              "t": 102.0,
              "v": 0.0
            },
            {
              "t": 104.0,
              "v": 0.0
            },
            {
              "t": 106.0,
              "v": 0.0
            },
            {
              "t": 108.0,
              "v": 0.0
            },
            {
              "t": 110.0,
              "v": 0.0
            },
            {
              "t": 112.0,
              "v": 0.0
            },
            {
              "t": 114.0,
              "v": 0.0
            }
          ],
          "write_kbs": [
            {
              "t": 2.0,
              "v": 2.0
            },
            {
              "t": 4.0,
              "v": 8.0
            },
            {
              "t": 6.0,
              "v": 20.0
            },
            {
              "t": 8.0,
              "v": 32.0
            },
            {
              "t": 10.0,
              "v": 50.0
            },
            {
              "t": 12.0,
              "v": 58.0
            },
            {
              "t": 14.0,
              "v": 62.0
            },
            {
              "t": 16.0,
              "v": 64.0
            },
            {
              "t": 18.0,
              "v": 62.0
            },
            {
              "t": 20.0,
              "v": 62.0
            },
            {
              "t": 22.0,
              "v": 52.0
            },
            {
              "t": 24.0,
              "v": 38.0
            },
            {
              "t": 26.0,
              "v": 16.0
            },
            {
              "t": 28.0,
              "v": 2.0
            },
            {
              "t": 30.0,
              "v": 0.0
            },
            {
              "t": 32.0,
              "v": 12.0
            },
            {
              "t": 34.0,
              "v": 22.0
            },
            {
              "t": 36.0,
              "v": 26.0
            },
            {
              "t": 38.0,
              "v": 34.0
            },
            {
              "t": 40.0,
              "v": 38.0
            },
            {
              "t": 42.0,
              "v": 38.0
            },
            {
              "t": 44.0,
              "v": 40.0
            },
            {
              "t": 46.0,
              "v": 34.0
            },
            {
              "t": 48.0,
              "v": 24.0
            },
            {
              "t": 50.0,
              "v": 12.0
            },
            {
              "t": 52.0,
              "v": 0.0
            },
            {
              "t": 54.0,
              "v": 0.0
            },
            {
              "t": 56.0,
              "v": 0.0
            },
            {
              "t": 58.0,
              "v": 0.0
            },
            {
              "t": 60.0,
              "v": 0.0
            },
            {
              "t": 62.0,
              "v": 460.0
            },
            {
              "t": 64.0,
              "v": 544.0
            },
            {
              "t": 66.0,
              "v": 542.0
            },
            {
              "t": 68.0,
              "v": 544.0
            },
            {
              "t": 70.0,
              "v": 540.0
            },
            {
              "t": 72.0,
              "v": 544.0
            },
            {
              "t": 74.0,
              "v": 542.0
            },
            {
              "t": 76.0,
              "v": 350.0
            },
            {
              "t": 78.0,
              "v": 0.0
            },
            {
              "t": 80.0,
              "v": 0.0
            },
            {
              "t": 82.0,
              "v": 910.0
            },
            {
              "t": 84.0,
              "v": 1084.0
            },
            {
              "t": 86.0,
              "v": 1084.0
            },
            {
              "t": 88.0,
              "v": 1078.0
            },
            {
              "t": 90.0,
              "v": 1084.0
            },
            {
              "t": 92.0,
              "v": 1080.0
            },
            {
              "t": 94.0,
              "v": 1084.0
            },
            {
              "t": 96.0,
              "v": 700.0
            },
            {
              "t": 98.0,
              "v": 0.0
            },
            {
              "t": 100.0,
              "v": 0.0
            },
            {
              "t": 102.0,
              "v": 1244.0
            },
            {
              "t": 104.0,
              "v": 1446.0
            },
            {
              "t": 106.0,
              "v": 1438.8
            },
            {
              "t": 108.0,
              "v": 1430.0
            },
            {
              "t": 110.0,
              "v": 1424.0
            },
            {
              "t": 112.0,
              "v": 1132.0
            },
            {
              "t": 114.0,
              "v": 1178.0
            }
          ]
        },
        "client": {
          "cpu": [
            {
              "t": 2.0,
              "v": 1.0
            },
            {
              "t": 6.0,
              "v": 2.5
            },
            {
              "t": 10.0,
              "v": 4.0
            },
            {
              "t": 14.0,
              "v": 6.5
            },
            {
              "t": 18.0,
              "v": 8.5
            },
            {
              "t": 22.0,
              "v": 10.0
            },
            {
              "t": 26.0,
              "v": 9.5
            },
            {
              "t": 30.0,
              "v": 10.0
            },
            {
              "t": 34.0,
              "v": 10.0
            },
            {
              "t": 38.0,
              "v": 10.0
            },
            {
              "t": 42.0,
              "v": 7.0
            },
            {
              "t": 46.0,
              "v": 4.0
            },
            {
              "t": 50.0,
              "v": 1.0
            },
            {
              "t": 54.0,
              "v": 0.5
            },
            {
              "t": 58.0,
              "v": 0.0
            },
            {
              "t": 62.0,
              "v": 3.0
            },
            {
              "t": 66.0,
              "v": 5.0
            },
            {
              "t": 70.0,
              "v": 8.5
            },
            {
              "t": 74.0,
              "v": 10.5
            },
            {
              "t": 78.0,
              "v": 13.5
            },
            {
              "t": 82.0,
              "v": 13.5
            },
            {
              "t": 86.0,
              "v": 14.0
            },
            {
              "t": 90.0,
              "v": 13.0
            },
            {
              "t": 94.0,
              "v": 10.5
            },
            {
              "t": 98.0,
              "v": 6.0
            },
            {
              "t": 102.0,
              "v": 0.0
            },
            {
              "t": 106.0,
              "v": 0.0
            },
            {
              "t": 110.0,
              "v": 0.0
            },
            {
              "t": 114.0,
              "v": 0.0
            },
            {
              "t": 118.0,
              "v": 0.0
            },
            {
              "t": 122.0,
              "v": 53.0
            },
            {
              "t": 126.0,
              "v": 62.5
            },
            {
              "t": 130.0,
              "v": 62.5
            },
            {
              "t": 134.0,
              "v": 62.5
            },
            {
              "t": 138.0,
              "v": 62.5
            },
            {
              "t": 142.0,
              "v": 62.5
            },
            {
              "t": 146.0,
              "v": 63.0
            },
            {
              "t": 150.0,
              "v": 41.0
            },
            {
              "t": 154.0,
              "v": 0.0
            },
            {
              "t": 158.0,
              "v": 0.0
            },
            {
              "t": 162.0,
              "v": 75.5
            },
            {
              "t": 166.0,
              "v": 88.5
            },
            {
              "t": 170.0,
              "v": 90.0
            },
            {
              "t": 174.0,
              "v": 88.0
            },
            {
              "t": 178.0,
              "v": 89.0
            },
            {
              "t": 182.0,
              "v": 87.5
            },
            {
              "t": 186.0,
              "v": 88.5
            },
            {
              "t": 190.0,
              "v": 57.5
            },
            {
              "t": 194.0,
              "v": 0.0
            },
            {
              "t": 198.0,
              "v": 0.0
            },
            {
              "t": 202.0,
              "v": 62.19
            },
            {
              "t": 206.0,
              "v": 73.0
            },
            {
              "t": 210.0,
              "v": 72.0
            },
            {
              "t": 214.0,
              "v": 72.5
            },
            {
              "t": 218.0,
              "v": 73.0
            },
            {
              "t": 222.0,
              "v": 60.5
            },
            {
              "t": 226.0,
              "v": 62.5
            }
          ],
          "rss": [
            {
              "t": 4.0,
              "v": 29.6
            },
            {
              "t": 8.0,
              "v": 29.6
            },
            {
              "t": 12.0,
              "v": 32.3
            },
            {
              "t": 16.0,
              "v": 32.4
            },
            {
              "t": 20.0,
              "v": 32.4
            },
            {
              "t": 24.0,
              "v": 32.4
            },
            {
              "t": 28.0,
              "v": 32.3
            },
            {
              "t": 32.0,
              "v": 31.6
            },
            {
              "t": 36.0,
              "v": 31.6
            },
            {
              "t": 40.0,
              "v": 31.2
            },
            {
              "t": 44.0,
              "v": 31.0
            },
            {
              "t": 48.0,
              "v": 30.8
            },
            {
              "t": 52.0,
              "v": 31.5
            },
            {
              "t": 56.0,
              "v": 31.5
            },
            {
              "t": 60.0,
              "v": 31.5
            },
            {
              "t": 64.0,
              "v": 31.2
            },
            {
              "t": 68.0,
              "v": 31.2
            },
            {
              "t": 72.0,
              "v": 31.2
            },
            {
              "t": 76.0,
              "v": 31.3
            },
            {
              "t": 80.0,
              "v": 31.0
            },
            {
              "t": 84.0,
              "v": 31.0
            },
            {
              "t": 88.0,
              "v": 31.2
            },
            {
              "t": 92.0,
              "v": 30.4
            },
            {
              "t": 96.0,
              "v": 29.8
            },
            {
              "t": 100.0,
              "v": 29.8
            },
            {
              "t": 104.0,
              "v": 29.8
            },
            {
              "t": 108.0,
              "v": 29.8
            },
            {
              "t": 112.0,
              "v": 29.8
            },
            {
              "t": 116.0,
              "v": 29.8
            },
            {
              "t": 120.0,
              "v": 29.8
            },
            {
              "t": 124.0,
              "v": 30.6
            },
            {
              "t": 128.0,
              "v": 30.1
            },
            {
              "t": 132.0,
              "v": 30.7
            },
            {
              "t": 136.0,
              "v": 30.8
            },
            {
              "t": 140.0,
              "v": 30.6
            },
            {
              "t": 144.0,
              "v": 30.8
            },
            {
              "t": 148.0,
              "v": 30.7
            },
            {
              "t": 152.0,
              "v": 30.8
            },
            {
              "t": 156.0,
              "v": 30.8
            },
            {
              "t": 160.0,
              "v": 30.8
            },
            {
              "t": 164.0,
              "v": 31.5
            },
            {
              "t": 168.0,
              "v": 32.3
            },
            {
              "t": 172.0,
              "v": 32.2
            },
            {
              "t": 176.0,
              "v": 31.8
            },
            {
              "t": 180.0,
              "v": 31.6
            },
            {
              "t": 184.0,
              "v": 32.7
            },
            {
              "t": 188.0,
              "v": 32.2
            },
            {
              "t": 192.0,
              "v": 32.3
            },
            {
              "t": 196.0,
              "v": 32.3
            },
            {
              "t": 200.0,
              "v": 32.3
            },
            {
              "t": 204.0,
              "v": 34.3
            },
            {
              "t": 208.0,
              "v": 37.7
            },
            {
              "t": 212.0,
              "v": 38.4
            },
            {
              "t": 216.0,
              "v": 44.2
            },
            {
              "t": 220.0,
              "v": 45.1
            },
            {
              "t": 224.0,
              "v": 44.7
            },
            {
              "t": 228.0,
              "v": 43.0
            }
          ],
          "read_kbs": [
            {
              "t": 2.0,
              "v": 0.0
            },
            {
              "t": 4.0,
              "v": 0.0
            },
            {
              "t": 6.0,
              "v": 0.0
            },
            {
              "t": 8.0,
              "v": 0.0
            },
            {
              "t": 10.0,
              "v": 0.0
            },
            {
              "t": 12.0,
              "v": 0.0
            },
            {
              "t": 14.0,
              "v": 0.0
            },
            {
              "t": 16.0,
              "v": 0.0
            },
            {
              "t": 18.0,
              "v": 0.0
            },
            {
              "t": 20.0,
              "v": 0.0
            },
            {
              "t": 22.0,
              "v": 0.0
            },
            {
              "t": 24.0,
              "v": 0.0
            },
            {
              "t": 26.0,
              "v": 0.0
            },
            {
              "t": 28.0,
              "v": 0.0
            },
            {
              "t": 30.0,
              "v": 0.0
            },
            {
              "t": 32.0,
              "v": 0.0
            },
            {
              "t": 34.0,
              "v": 0.0
            },
            {
              "t": 36.0,
              "v": 0.0
            },
            {
              "t": 38.0,
              "v": 0.0
            },
            {
              "t": 40.0,
              "v": 0.0
            },
            {
              "t": 42.0,
              "v": 0.0
            },
            {
              "t": 44.0,
              "v": 0.0
            },
            {
              "t": 46.0,
              "v": 0.0
            },
            {
              "t": 48.0,
              "v": 0.0
            },
            {
              "t": 50.0,
              "v": 0.0
            },
            {
              "t": 52.0,
              "v": 0.0
            },
            {
              "t": 54.0,
              "v": 0.0
            },
            {
              "t": 56.0,
              "v": 0.0
            },
            {
              "t": 58.0,
              "v": 0.0
            },
            {
              "t": 60.0,
              "v": 0.0
            },
            {
              "t": 62.0,
              "v": 0.0
            },
            {
              "t": 64.0,
              "v": 0.0
            },
            {
              "t": 66.0,
              "v": 0.0
            },
            {
              "t": 68.0,
              "v": 0.0
            },
            {
              "t": 70.0,
              "v": 0.0
            },
            {
              "t": 72.0,
              "v": 0.0
            },
            {
              "t": 74.0,
              "v": 0.0
            },
            {
              "t": 76.0,
              "v": 0.0
            },
            {
              "t": 78.0,
              "v": 0.0
            },
            {
              "t": 80.0,
              "v": 0.0
            },
            {
              "t": 82.0,
              "v": 0.0
            },
            {
              "t": 84.0,
              "v": 0.0
            },
            {
              "t": 86.0,
              "v": 0.0
            },
            {
              "t": 88.0,
              "v": 0.0
            },
            {
              "t": 90.0,
              "v": 0.0
            },
            {
              "t": 92.0,
              "v": 0.0
            },
            {
              "t": 94.0,
              "v": 0.0
            },
            {
              "t": 96.0,
              "v": 0.0
            },
            {
              "t": 98.0,
              "v": 0.0
            },
            {
              "t": 100.0,
              "v": 0.0
            },
            {
              "t": 102.0,
              "v": 0.0
            },
            {
              "t": 104.0,
              "v": 0.0
            },
            {
              "t": 106.0,
              "v": 0.0
            },
            {
              "t": 108.0,
              "v": 0.0
            },
            {
              "t": 110.0,
              "v": 0.0
            },
            {
              "t": 112.0,
              "v": 0.0
            },
            {
              "t": 114.0,
              "v": 0.0
            }
          ],
          "write_kbs": [
            {
              "t": 2.0,
              "v": 4.0
            },
            {
              "t": 4.0,
              "v": 16.0
            },
            {
              "t": 6.0,
              "v": 30.0
            },
            {
              "t": 8.0,
              "v": 40.0
            },
            {
              "t": 10.0,
              "v": 50.0
            },
            {
              "t": 12.0,
              "v": 52.0
            },
            {
              "t": 14.0,
              "v": 54.0
            },
            {
              "t": 16.0,
              "v": 54.0
            },
            {
              "t": 18.0,
              "v": 56.0
            },
            {
              "t": 20.0,
              "v": 48.0
            },
            {
              "t": 22.0,
              "v": 30.0
            },
            {
              "t": 24.0,
              "v": 8.0
            },
            {
              "t": 26.0,
              "v": 0.0
            },
            {
              "t": 28.0,
              "v": 0.0
            },
            {
              "t": 30.0,
              "v": 0.0
            },
            {
              "t": 32.0,
              "v": 22.0
            },
            {
              "t": 34.0,
              "v": 30.0
            },
            {
              "t": 36.0,
              "v": 44.0
            },
            {
              "t": 38.0,
              "v": 56.0
            },
            {
              "t": 40.0,
              "v": 70.0
            },
            {
              "t": 42.0,
              "v": 70.0
            },
            {
              "t": 44.0,
              "v": 72.0
            },
            {
              "t": 46.0,
              "v": 64.0
            },
            {
              "t": 48.0,
              "v": 44.0
            },
            {
              "t": 50.0,
              "v": 28.0
            },
            {
              "t": 52.0,
              "v": 0.0
            },
            {
              "t": 54.0,
              "v": 0.0
            },
            {
              "t": 56.0,
              "v": 0.0
            },
            {
              "t": 58.0,
              "v": 0.0
            },
            {
              "t": 60.0,
              "v": 0.0
            },
            {
              "t": 62.0,
              "v": 334.0
            },
            {
              "t": 64.0,
              "v": 394.0
            },
            {
              "t": 66.0,
              "v": 392.0
            },
            {
              "t": 68.0,
              "v": 394.0
            },
            {
              "t": 70.0,
              "v": 392.0
            },
            {
              "t": 72.0,
              "v": 394.0
            },
            {
              "t": 74.0,
              "v": 392.0
            },
            {
              "t": 76.0,
              "v": 254.0
            },
            {
              "t": 78.0,
              "v": 0.0
            },
            {
              "t": 80.0,
              "v": 0.0
            },
            {
              "t": 82.0,
              "v": 658.0
            },
            {
              "t": 84.0,
              "v": 784.0
            },
            {
              "t": 86.0,
              "v": 786.0
            },
            {
              "t": 88.0,
              "v": 782.0
            },
            {
              "t": 90.0,
              "v": 786.0
            },
            {
              "t": 92.0,
              "v": 784.0
            },
            {
              "t": 94.0,
              "v": 784.0
            },
            {
              "t": 96.0,
              "v": 508.0
            },
            {
              "t": 98.0,
              "v": 0.0
            },
            {
              "t": 100.0,
              "v": 0.0
            },
            {
              "t": 102.0,
              "v": 892.0
            },
            {
              "t": 104.0,
              "v": 1038.0
            },
            {
              "t": 106.0,
              "v": 1028.9
            },
            {
              "t": 108.0,
              "v": 1034.0
            },
            {
              "t": 110.0,
              "v": 1010.0
            },
            {
              "t": 112.0,
              "v": 882.0
            },
            {
              "t": 114.0,
              "v": 940.0
            }
          ]
        },
        "system": {
          "cpu": [
            {
              "t": 2.0,
              "v": 13.7
            },
            {
              "t": 4.0,
              "v": 2.1
            },
            {
              "t": 6.0,
              "v": 4.3
            },
            {
              "t": 8.0,
              "v": 6.9
            },
            {
              "t": 10.0,
              "v": 9.8
            },
            {
              "t": 12.0,
              "v": 9.7
            },
            {
              "t": 14.0,
              "v": 10.5
            },
            {
              "t": 16.0,
              "v": 10.0
            },
            {
              "t": 18.0,
              "v": 10.6
            },
            {
              "t": 20.0,
              "v": 9.4
            },
            {
              "t": 22.0,
              "v": 7.2
            },
            {
              "t": 24.0,
              "v": 4.6
            },
            {
              "t": 26.0,
              "v": 1.9
            },
            {
              "t": 28.0,
              "v": 0.8
            },
            {
              "t": 30.0,
              "v": 0.1
            },
            {
              "t": 32.0,
              "v": 3.0
            },
            {
              "t": 34.0,
              "v": 5.9
            },
            {
              "t": 36.0,
              "v": 9.8
            },
            {
              "t": 38.0,
              "v": 10.5
            },
            {
              "t": 40.0,
              "v": 13.8
            },
            {
              "t": 42.0,
              "v": 14.2
            },
            {
              "t": 44.0,
              "v": 12.6
            },
            {
              "t": 46.0,
              "v": 12.9
            },
            {
              "t": 48.0,
              "v": 10.2
            },
            {
              "t": 50.0,
              "v": 5.7
            },
            {
              "t": 52.0,
              "v": 0.5
            },
            {
              "t": 54.0,
              "v": 0.4
            },
            {
              "t": 56.0,
              "v": 0.2
            },
            {
              "t": 58.0,
              "v": 0.2
            },
            {
              "t": 60.0,
              "v": 0.2
            },
            {
              "t": 62.0,
              "v": 50.8
            },
            {
              "t": 64.0,
              "v": 60.5
            },
            {
              "t": 66.0,
              "v": 60.5
            },
            {
              "t": 68.0,
              "v": 60.8
            },
            {
              "t": 70.0,
              "v": 60.1
            },
            {
              "t": 72.0,
              "v": 61.0
            },
            {
              "t": 74.0,
              "v": 61.6
            },
            {
              "t": 76.0,
              "v": 38.3
            },
            {
              "t": 78.0,
              "v": 0.2
            },
            {
              "t": 80.0,
              "v": 0.6
            },
            {
              "t": 82.0,
              "v": 81.5
            },
            {
              "t": 84.0,
              "v": 95.5
            },
            {
              "t": 86.0,
              "v": 95.6
            },
            {
              "t": 88.0,
              "v": 95.7
            },
            {
              "t": 90.0,
              "v": 95.7
            },
            {
              "t": 92.0,
              "v": 95.8
            },
            {
              "t": 94.0,
              "v": 95.5
            },
            {
              "t": 96.0,
              "v": 61.7
            },
            {
              "t": 98.0,
              "v": 0.1
            },
            {
              "t": 100.0,
              "v": 0.2
            },
            {
              "t": 102.0,
              "v": 85.0
            },
            {
              "t": 104.0,
              "v": 99.8
            },
            {
              "t": 106.0,
              "v": 99.9
            },
            {
              "t": 108.0,
              "v": 100.0
            },
            {
              "t": 110.0,
              "v": 100.0
            },
            {
              "t": 112.0,
              "v": 100.0
            },
            {
              "t": 114.0,
              "v": 100.0
            }
          ]
        },
        "k6OffsetSeconds": 0,
        "network": {
          "rx_kbs": [
            {
              "t": 2.0,
              "v": 13.3
            },
            {
              "t": 4.0,
              "v": 1.9
            },
            {
              "t": 6.0,
              "v": 2.6
            },
            {
              "t": 8.0,
              "v": 0.2
            },
            {
              "t": 10.0,
              "v": 2.1
            },
            {
              "t": 12.0,
              "v": 0.1
            },
            {
              "t": 14.0,
              "v": 0.1
            },
            {
              "t": 16.0,
              "v": 2.0
            },
            {
              "t": 18.0,
              "v": 0.4
            },
            {
              "t": 20.0,
              "v": 0.1
            },
            {
              "t": 22.0,
              "v": 2.4
            },
            {
              "t": 24.0,
              "v": 0.2
            },
            {
              "t": 26.0,
              "v": 0.1
            },
            {
              "t": 28.0,
              "v": 2.0
            },
            {
              "t": 30.0,
              "v": 0.1
            },
            {
              "t": 32.0,
              "v": 0.1
            },
            {
              "t": 34.0,
              "v": 1.9
            },
            {
              "t": 36.0,
              "v": 0.1
            },
            {
              "t": 38.0,
              "v": 0.1
            },
            {
              "t": 40.0,
              "v": 1.9
            },
            {
              "t": 42.0,
              "v": 0.1
            },
            {
              "t": 44.0,
              "v": 0.2
            },
            {
              "t": 46.0,
              "v": 2.0
            },
            {
              "t": 48.0,
              "v": 0.3
            },
            {
              "t": 50.0,
              "v": 0.1
            },
            {
              "t": 52.0,
              "v": 2.1
            },
            {
              "t": 54.0,
              "v": 0.1
            },
            {
              "t": 56.0,
              "v": 0.1
            },
            {
              "t": 58.0,
              "v": 1.9
            },
            {
              "t": 60.0,
              "v": 0.1
            },
            {
              "t": 62.0,
              "v": 0.2
            },
            {
              "t": 64.0,
              "v": 1.9
            },
            {
              "t": 66.0,
              "v": 2.6
            },
            {
              "t": 68.0,
              "v": 0.1
            },
            {
              "t": 70.0,
              "v": 2.1
            },
            {
              "t": 72.0,
              "v": 0.1
            },
            {
              "t": 74.0,
              "v": 0.2
            },
            {
              "t": 76.0,
              "v": 2.0
            },
            {
              "t": 78.0,
              "v": 0.3
            },
            {
              "t": 80.0,
              "v": 0.1
            },
            {
              "t": 82.0,
              "v": 2.4
            },
            {
              "t": 84.0,
              "v": 0.1
            },
            {
              "t": 86.0,
              "v": 0.1
            },
            {
              "t": 88.0,
              "v": 1.9
            },
            {
              "t": 90.0,
              "v": 0.1
            },
            {
              "t": 92.0,
              "v": 0.2
            },
            {
              "t": 94.0,
              "v": 1.9
            },
            {
              "t": 96.0,
              "v": 0.2
            },
            {
              "t": 98.0,
              "v": 0.1
            },
            {
              "t": 100.0,
              "v": 1.9
            },
            {
              "t": 102.0,
              "v": 0.3
            },
            {
              "t": 104.0,
              "v": 0.1
            },
            {
              "t": 106.0,
              "v": 1.9
            },
            {
              "t": 108.0,
              "v": 0.3
            },
            {
              "t": 110.0,
              "v": 0.1
            },
            {
              "t": 112.0,
              "v": 2.4
            },
            {
              "t": 114.0,
              "v": 1.2
            }
          ],
          "tx_kbs": [
            {
              "t": 2.0,
              "v": 32.6
            },
            {
              "t": 4.0,
              "v": 5.8
            },
            {
              "t": 6.0,
              "v": 3.1
            },
            {
              "t": 8.0,
              "v": 1.8
            },
            {
              "t": 10.0,
              "v": 6.3
            },
            {
              "t": 12.0,
              "v": 1.8
            },
            {
              "t": 14.0,
              "v": 1.7
            },
            {
              "t": 16.0,
              "v": 6.0
            },
            {
              "t": 18.0,
              "v": 11.0
            },
            {
              "t": 20.0,
              "v": 1.7
            },
            {
              "t": 22.0,
              "v": 7.6
            },
            {
              "t": 24.0,
              "v": 1.8
            },
            {
              "t": 26.0,
              "v": 1.8
            },
            {
              "t": 28.0,
              "v": 6.0
            },
            {
              "t": 30.0,
              "v": 1.8
            },
            {
              "t": 32.0,
              "v": 1.8
            },
            {
              "t": 34.0,
              "v": 6.0
            },
            {
              "t": 36.0,
              "v": 1.8
            },
            {
              "t": 38.0,
              "v": 1.9
            },
            {
              "t": 40.0,
              "v": 6.0
            },
            {
              "t": 42.0,
              "v": 1.8
            },
            {
              "t": 44.0,
              "v": 1.9
            },
            {
              "t": 46.0,
              "v": 6.0
            },
            {
              "t": 48.0,
              "v": 8.4
            },
            {
              "t": 50.0,
              "v": 1.9
            },
            {
              "t": 52.0,
              "v": 7.1
            },
            {
              "t": 54.0,
              "v": 1.9
            },
            {
              "t": 56.0,
              "v": 1.9
            },
            {
              "t": 58.0,
              "v": 6.0
            },
            {
              "t": 60.0,
              "v": 1.9
            },
            {
              "t": 62.0,
              "v": 2.0
            },
            {
              "t": 64.0,
              "v": 6.1
            },
            {
              "t": 66.0,
              "v": 3.3
            },
            {
              "t": 68.0,
              "v": 2.0
            },
            {
              "t": 70.0,
              "v": 6.5
            },
            {
              "t": 72.0,
              "v": 1.9
            },
            {
              "t": 74.0,
              "v": 2.1
            },
            {
              "t": 76.0,
              "v": 6.1
            },
            {
              "t": 78.0,
              "v": 7.8
            },
            {
              "t": 80.0,
              "v": 1.9
            },
            {
              "t": 82.0,
              "v": 7.9
            },
            {
              "t": 84.0,
              "v": 2.0
            },
            {
              "t": 86.0,
              "v": 1.9
            },
            {
              "t": 88.0,
              "v": 6.1
            },
            {
              "t": 90.0,
              "v": 1.9
            },
            {
              "t": 92.0,
              "v": 2.1
            },
            {
              "t": 94.0,
              "v": 6.1
            },
            {
              "t": 96.0,
              "v": 2.2
            },
            {
              "t": 98.0,
              "v": 2.0
            },
            {
              "t": 100.0,
              "v": 6.1
            },
            {
              "t": 102.0,
              "v": 3.0
            },
            {
              "t": 104.0,
              "v": 2.0
            },
            {
              "t": 106.0,
              "v": 6.2
            },
            {
              "t": 108.0,
              "v": 8.8
            },
            {
              "t": 110.0,
              "v": 2.0
            },
            {
              "t": 112.0,
              "v": 22.2
            },
            {
              "t": 114.0,
              "v": 28.7
            }
          ]
        }
      }
    }
  ]
};
