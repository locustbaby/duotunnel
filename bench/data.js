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
    }
  ]
};
