window.BENCH_DETAIL['59a87d5'] = {
  "timestamp": "2026-04-09T09:53:18.394Z",
  "commit": {
    "id": "59a87d58ba895cf337ea9e4dcf3063558b49d728",
    "message": "fix: HEAD 502 and no-keepalive stall in H2 sender",
    "url": "https://github.com/locustbaby/duotunnel/commit/59a87d58ba895cf337ea9e4dcf3063558b49d728"
  },
  "scenarios": [
    {
      "name": "ingress_http_get",
      "protocol": "HTTP",
      "direction": "ingress",
      "category": "basic",
      "includeInTotalRps": false,
      "p50": 0.63,
      "p95": 1.2,
      "p99": 0,
      "err": 0,
      "rps": 37,
      "requests": 925
    },
    {
      "name": "ingress_http_post",
      "protocol": "HTTP",
      "direction": "ingress",
      "category": "basic",
      "includeInTotalRps": false,
      "p50": 0.56,
      "p95": 1.15,
      "p99": 0,
      "err": 0,
      "rps": 23,
      "requests": 575
    },
    {
      "name": "egress_http_get",
      "protocol": "HTTP",
      "direction": "egress",
      "category": "basic",
      "includeInTotalRps": false,
      "p50": 0.55,
      "p95": 1.1,
      "p99": 0,
      "err": 0,
      "rps": 46,
      "requests": 1150
    },
    {
      "name": "egress_http_post",
      "protocol": "HTTP",
      "direction": "egress",
      "category": "basic",
      "includeInTotalRps": false,
      "p50": 0.56,
      "p95": 0.79,
      "p99": 0,
      "err": 0,
      "rps": 36.48,
      "requests": 912
    },
    {
      "name": "ws_ingress",
      "protocol": "WS",
      "direction": "ingress",
      "category": "basic",
      "includeInTotalRps": false,
      "p50": 2.06,
      "p95": 2.85,
      "p99": 0,
      "err": 0,
      "rps": 10,
      "requests": 200
    },
    {
      "name": "grpc_health_ingress",
      "protocol": "gRPC",
      "direction": "ingress",
      "category": "basic",
      "includeInTotalRps": false,
      "p50": 1.19,
      "p95": 1.85,
      "p99": 0,
      "err": 0,
      "rps": 20.05,
      "requests": 401
    },
    {
      "name": "grpc_echo_ingress",
      "protocol": "gRPC",
      "direction": "ingress",
      "category": "basic",
      "includeInTotalRps": false,
      "p50": 1.23,
      "p95": 1.9,
      "p99": 0,
      "err": 0,
      "rps": 20.05,
      "requests": 401
    },
    {
      "name": "bidir_mixed",
      "protocol": "HTTP",
      "direction": "bidir",
      "category": "basic",
      "includeInTotalRps": false,
      "p50": 0.53,
      "p95": 0.8,
      "p99": 0,
      "err": 0,
      "rps": 18.1,
      "requests": 362
    },
    {
      "name": "ingress_post_1k",
      "protocol": "HTTP",
      "direction": "ingress",
      "category": "body_size",
      "includeInTotalRps": false,
      "p50": 0.71,
      "p95": 1.94,
      "p99": 0,
      "err": 0,
      "rps": 30.05,
      "requests": 601
    },
    {
      "name": "ingress_post_10k",
      "protocol": "HTTP",
      "direction": "ingress",
      "category": "body_size",
      "includeInTotalRps": false,
      "p50": 1.2,
      "p95": 2.59,
      "p99": 0,
      "err": 0,
      "rps": 20,
      "requests": 400
    },
    {
      "name": "ingress_post_100k",
      "protocol": "HTTP",
      "direction": "ingress",
      "category": "body_size",
      "includeInTotalRps": false,
      "p50": 2.76,
      "p95": 3.69,
      "p99": 0,
      "err": 0,
      "rps": 10,
      "requests": 200
    },
    {
      "name": "egress_post_10k",
      "protocol": "HTTP",
      "direction": "egress",
      "category": "body_size",
      "includeInTotalRps": false,
      "p50": 1.13,
      "p95": 2.32,
      "p99": 0,
      "err": 0,
      "rps": 20,
      "requests": 400
    },
    {
      "name": "ws_multi_msg",
      "protocol": "WS",
      "direction": "ingress",
      "category": "basic",
      "includeInTotalRps": false,
      "p50": 44.37,
      "p95": 46.21,
      "p99": 0,
      "err": 0,
      "rps": 5,
      "requests": 100
    },
    {
      "name": "grpc_large_payload",
      "protocol": "gRPC",
      "direction": "ingress",
      "category": "body_size",
      "includeInTotalRps": false,
      "p50": 1.14,
      "p95": 2.89,
      "p99": 0,
      "err": 0,
      "rps": 15.05,
      "requests": 301
    },
    {
      "name": "grpc_high_qps",
      "protocol": "gRPC",
      "direction": "ingress",
      "category": "stress",
      "includeInTotalRps": true,
      "p50": 0.9,
      "p95": 1.77,
      "p99": 0,
      "err": 0,
      "rps": 89.95,
      "requests": 1799
    },
    {
      "name": "ingress_3000qps",
      "protocol": "HTTP",
      "direction": "ingress",
      "category": "stress",
      "includeInTotalRps": true,
      "p50": 0.55,
      "p95": 2.05,
      "p99": 0,
      "err": 0,
      "rps": 3000.05,
      "requests": 60001
    },
    {
      "name": "egress_3000qps",
      "protocol": "HTTP",
      "direction": "egress",
      "category": "stress",
      "includeInTotalRps": true,
      "p50": 0.54,
      "p95": 1.91,
      "p99": 0,
      "err": 0,
      "rps": 3000.05,
      "requests": 60001
    },
    {
      "name": "ingress_multihost",
      "protocol": "HTTP",
      "direction": "ingress",
      "category": "stress",
      "includeInTotalRps": true,
      "p50": 0.56,
      "p95": 2.3,
      "p99": 0,
      "err": 0,
      "rps": 3000.05,
      "requests": 60001
    },
    {
      "name": "egress_multihost",
      "protocol": "HTTP",
      "direction": "egress",
      "category": "stress",
      "includeInTotalRps": true,
      "p50": 0.58,
      "p95": 2.16,
      "p99": 0,
      "err": 0,
      "rps": 3000.1,
      "requests": 60002
    },
    {
      "name": "ingress_8000qps (8k-q4)",
      "protocol": "HTTP",
      "direction": "ingress",
      "category": "stress",
      "includeInTotalRps": true,
      "p50": 5.34,
      "p95": 83.26,
      "p99": 0,
      "err": 0,
      "rps": 7515.1,
      "requests": 150302
    },
    {
      "name": "egress_8000qps (8k-q4)",
      "protocol": "HTTP",
      "direction": "egress",
      "category": "stress",
      "includeInTotalRps": true,
      "p50": 5.53,
      "p95": 79.65,
      "p99": 0,
      "err": 0,
      "rps": 7556.95,
      "requests": 151139
    },
    {
      "name": "ingress_multihost_8000qps (8k-q4)",
      "protocol": "HTTP",
      "direction": "ingress",
      "category": "stress",
      "includeInTotalRps": true,
      "p50": 3.29,
      "p95": 35.83,
      "p99": 0,
      "err": 0.94,
      "rps": 7370.15,
      "requests": 147403
    },
    {
      "name": "egress_multihost_8000qps (8k-q4)",
      "protocol": "HTTP",
      "direction": "egress",
      "category": "stress",
      "includeInTotalRps": true,
      "p50": 6.35,
      "p95": 95.97,
      "p99": 0,
      "err": 0.01,
      "rps": 7454.15,
      "requests": 149083
    },
    {
      "name": "ingress_3000qps",
      "protocol": "HTTP",
      "direction": "ingress",
      "category": "stress",
      "includeInTotalRps": true,
      "p50": 1.4,
      "p95": 59.99,
      "p99": 0,
      "err": 0,
      "rps": 2901.7,
      "requests": 58034,
      "tunnel": "frp"
    },
    {
      "name": "ingress_multihost",
      "protocol": "HTTP",
      "direction": "ingress",
      "category": "stress",
      "includeInTotalRps": true,
      "p50": 0.71,
      "p95": 3.34,
      "p99": 0,
      "err": 0,
      "rps": 2999.4,
      "requests": 59988,
      "tunnel": "frp"
    },
    {
      "name": "ingress_multihost_8000qps",
      "protocol": "HTTP",
      "direction": "ingress",
      "category": "stress",
      "includeInTotalRps": true,
      "p50": 44.52,
      "p95": 114.33,
      "p99": 0,
      "err": 0,
      "rps": 5467,
      "requests": 109340,
      "tunnel": "frp"
    }
  ],
  "summary": {
    "totalRPS": 41986.55,
    "totalErr": 0.17,
    "totalRequests": 846659
  },
  "phases": [
    {
      "name": "Basic",
      "start": 0,
      "end": 29,
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
      "name": "3K KL (frp)",
      "start": 2,
      "end": 22,
      "scenarios": [
        "ingress_3000qps"
      ],
      "tunnel": "frp"
    },
    {
      "name": "3K Multihost (frp)",
      "start": 27,
      "end": 47,
      "scenarios": [
        "ingress_multihost"
      ],
      "tunnel": "frp"
    },
    {
      "name": "Body/Payload",
      "start": 35,
      "end": 59,
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
      "name": "8K Multihost (frp)",
      "start": 52,
      "end": 72,
      "scenarios": [
        "ingress_multihost_8000qps"
      ],
      "tunnel": "frp"
    },
    {
      "name": "Ingress 3K",
      "start": 65,
      "end": 85,
      "scenarios": [
        "ingress_3000qps"
      ]
    },
    {
      "name": "Egress 3K",
      "start": 90,
      "end": 110,
      "scenarios": [
        "egress_3000qps"
      ]
    },
    {
      "name": "Ingress multihost 3K",
      "start": 115,
      "end": 135,
      "scenarios": [
        "ingress_multihost"
      ]
    },
    {
      "name": "Egress multihost 3K",
      "start": 140,
      "end": 160,
      "scenarios": [
        "egress_multihost"
      ]
    },
    {
      "name": "Ingress 8K (8k-q4)",
      "start": 165.0,
      "end": 185.0,
      "scenarios": [
        "ingress_8000qps (8k-q4)"
      ]
    },
    {
      "name": "Egress 8K (8k-q4)",
      "start": 190.0,
      "end": 210.0,
      "scenarios": [
        "egress_8000qps (8k-q4)"
      ]
    },
    {
      "name": "Ingress multihost 8K (8k-q4)",
      "start": 215.0,
      "end": 235.0,
      "scenarios": [
        "ingress_multihost_8000qps (8k-q4)"
      ]
    },
    {
      "name": "Egress multihost 8K (8k-q4)",
      "start": 240.0,
      "end": 260.0,
      "scenarios": [
        "egress_multihost_8000qps (8k-q4)"
      ]
    }
  ],
  "catalog": {
    "categories": [
      {
        "id": "basic",
        "label": "Basic",
        "description": "Ramping rate with mixed protocols for baseline latency."
      },
      {
        "id": "body_size",
        "label": "Body Size",
        "description": "Fixed rate with larger payloads to observe payload scaling."
      },
      {
        "id": "stress",
        "label": "Stress",
        "description": "High-QPS stress stages for ingress and egress under pressure."
      }
    ],
    "caseOrder": [
      "ingress_http_get",
      "ingress_http_post",
      "egress_http_get",
      "egress_http_post",
      "ws_ingress",
      "grpc_health_ingress",
      "grpc_echo_ingress",
      "bidir_mixed",
      "ingress_post_1k",
      "ingress_post_10k",
      "ingress_post_100k",
      "egress_post_10k",
      "ws_multi_msg",
      "grpc_large_payload",
      "grpc_high_qps",
      "ingress_3000qps",
      "egress_3000qps",
      "ingress_multihost",
      "egress_multihost",
      "ingress_8000qps (8k-q4)",
      "egress_8000qps (8k-q4)",
      "ingress_multihost_8000qps (8k-q4)",
      "egress_multihost_8000qps (8k-q4)"
    ]
  },
  "resources": {
    "processes": {
      "server": {
        "cpu": [
          {
            "t": 58.9,
            "v": 0.04
          },
          {
            "t": 75.0,
            "v": 0.04
          },
          {
            "t": 78.7,
            "v": 0.62
          },
          {
            "t": 82.4,
            "v": 1.49
          },
          {
            "t": 86.1,
            "v": 1.81
          },
          {
            "t": 89.8,
            "v": 1.87
          },
          {
            "t": 93.6,
            "v": 1.87
          },
          {
            "t": 97.3,
            "v": 1.86
          },
          {
            "t": 101.1,
            "v": 1.33
          },
          {
            "t": 104.8,
            "v": 0.41
          },
          {
            "t": 108.4,
            "v": 0.07
          },
          {
            "t": 112.1,
            "v": 1.03
          },
          {
            "t": 115.8,
            "v": 1.89
          },
          {
            "t": 119.5,
            "v": 2.39
          },
          {
            "t": 123.3,
            "v": 2.45
          },
          {
            "t": 127.1,
            "v": 2.39
          },
          {
            "t": 130.9,
            "v": 1.97
          },
          {
            "t": 134.6,
            "v": 0.54
          },
          {
            "t": 138.3,
            "v": 0.48
          },
          {
            "t": 142.6,
            "v": 13.02
          },
          {
            "t": 147.2,
            "v": 12.08
          },
          {
            "t": 151.9,
            "v": 12.1
          },
          {
            "t": 156.4,
            "v": 12.16
          },
          {
            "t": 160.9,
            "v": 3.57
          },
          {
            "t": 164.5,
            "v": 7.2
          },
          {
            "t": 169.2,
            "v": 14.8
          },
          {
            "t": 173.8,
            "v": 14.74
          },
          {
            "t": 178.4,
            "v": 14.87
          },
          {
            "t": 183.0,
            "v": 14.45
          },
          {
            "t": 190.5,
            "v": 9.4
          },
          {
            "t": 195.2,
            "v": 12.17
          },
          {
            "t": 199.8,
            "v": 12.02
          },
          {
            "t": 204.4,
            "v": 11.91
          },
          {
            "t": 209.0,
            "v": 9.13
          },
          {
            "t": 216.7,
            "v": 14.66
          },
          {
            "t": 221.3,
            "v": 14.68
          },
          {
            "t": 226.0,
            "v": 14.41
          },
          {
            "t": 230.6,
            "v": 14.83
          }
        ],
        "rss": [
          {
            "t": 3.5,
            "v": 15.2
          },
          {
            "t": 8.3,
            "v": 15.2
          },
          {
            "t": 13.0,
            "v": 15.2
          },
          {
            "t": 20.6,
            "v": 15.2
          },
          {
            "t": 25.2,
            "v": 15.2
          },
          {
            "t": 28.9,
            "v": 15.2
          },
          {
            "t": 33.9,
            "v": 15.2
          },
          {
            "t": 38.8,
            "v": 15.2
          },
          {
            "t": 43.6,
            "v": 15.2
          },
          {
            "t": 48.4,
            "v": 15.2
          },
          {
            "t": 52.1,
            "v": 15.2
          },
          {
            "t": 58.9,
            "v": 15.2
          },
          {
            "t": 67.9,
            "v": 15.2
          },
          {
            "t": 75.0,
            "v": 15.3
          },
          {
            "t": 78.7,
            "v": 16.1
          },
          {
            "t": 82.4,
            "v": 16.9
          },
          {
            "t": 86.1,
            "v": 17.1
          },
          {
            "t": 89.8,
            "v": 17.3
          },
          {
            "t": 93.6,
            "v": 17.4
          },
          {
            "t": 97.3,
            "v": 17.6
          },
          {
            "t": 101.1,
            "v": 17.3
          },
          {
            "t": 104.8,
            "v": 17.3
          },
          {
            "t": 108.4,
            "v": 18.0
          },
          {
            "t": 112.1,
            "v": 19.8
          },
          {
            "t": 115.8,
            "v": 21.3
          },
          {
            "t": 119.5,
            "v": 21.2
          },
          {
            "t": 123.3,
            "v": 21.7
          },
          {
            "t": 127.1,
            "v": 21.6
          },
          {
            "t": 130.9,
            "v": 21.8
          },
          {
            "t": 134.6,
            "v": 21.8
          },
          {
            "t": 138.3,
            "v": 22.5
          },
          {
            "t": 142.6,
            "v": 22.6
          },
          {
            "t": 147.2,
            "v": 22.3
          },
          {
            "t": 151.9,
            "v": 22.1
          },
          {
            "t": 156.4,
            "v": 22.2
          },
          {
            "t": 160.9,
            "v": 22.2
          },
          {
            "t": 164.5,
            "v": 24.6
          },
          {
            "t": 169.2,
            "v": 25.0
          },
          {
            "t": 173.8,
            "v": 24.8
          },
          {
            "t": 178.4,
            "v": 24.8
          },
          {
            "t": 183.0,
            "v": 24.7
          },
          {
            "t": 186.9,
            "v": 24.7
          },
          {
            "t": 190.5,
            "v": 28.0
          },
          {
            "t": 195.2,
            "v": 28.1
          },
          {
            "t": 199.8,
            "v": 28.1
          },
          {
            "t": 204.4,
            "v": 28.1
          },
          {
            "t": 209.0,
            "v": 28.1
          },
          {
            "t": 212.7,
            "v": 28.1
          },
          {
            "t": 216.7,
            "v": 29.4
          },
          {
            "t": 221.3,
            "v": 29.1
          },
          {
            "t": 226.0,
            "v": 28.7
          },
          {
            "t": 230.6,
            "v": 28.4
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
          },
          {
            "t": 114.0,
            "v": 0.0
          },
          {
            "t": 116.0,
            "v": 0.0
          },
          {
            "t": 118.0,
            "v": 0.0
          },
          {
            "t": 120.0,
            "v": 0.0
          },
          {
            "t": 122.0,
            "v": 0.0
          },
          {
            "t": 124.0,
            "v": 0.0
          },
          {
            "t": 126.0,
            "v": 0.0
          },
          {
            "t": 128.0,
            "v": 0.0
          },
          {
            "t": 130.0,
            "v": 0.0
          },
          {
            "t": 132.0,
            "v": 0.0
          },
          {
            "t": 134.0,
            "v": 0.0
          },
          {
            "t": 136.0,
            "v": 0.0
          },
          {
            "t": 138.0,
            "v": 0.0
          },
          {
            "t": 140.0,
            "v": 0.0
          },
          {
            "t": 142.0,
            "v": 0.0
          },
          {
            "t": 144.0,
            "v": 0.0
          },
          {
            "t": 146.0,
            "v": 0.0
          },
          {
            "t": 148.0,
            "v": 0.0
          },
          {
            "t": 150.0,
            "v": 0.0
          },
          {
            "t": 152.0,
            "v": 0.0
          },
          {
            "t": 154.0,
            "v": 0.0
          },
          {
            "t": 156.0,
            "v": 0.0
          },
          {
            "t": 158.0,
            "v": 0.0
          },
          {
            "t": 160.0,
            "v": 0.0
          },
          {
            "t": 162.0,
            "v": 0.0
          },
          {
            "t": 164.0,
            "v": 0.0
          },
          {
            "t": 166.0,
            "v": 0.0
          },
          {
            "t": 168.0,
            "v": 0.0
          },
          {
            "t": 170.0,
            "v": 0.0
          },
          {
            "t": 172.0,
            "v": 0.0
          },
          {
            "t": 174.0,
            "v": 0.0
          },
          {
            "t": 176.0,
            "v": 0.0
          },
          {
            "t": 178.0,
            "v": 0.0
          },
          {
            "t": 180.0,
            "v": 0.0
          },
          {
            "t": 182.0,
            "v": 0.0
          },
          {
            "t": 184.0,
            "v": 0.0
          },
          {
            "t": 186.0,
            "v": 0.0
          },
          {
            "t": 188.0,
            "v": 0.0
          },
          {
            "t": 190.0,
            "v": 0.0
          },
          {
            "t": 192.0,
            "v": 0.0
          },
          {
            "t": 194.0,
            "v": 0.0
          },
          {
            "t": 196.0,
            "v": 0.0
          },
          {
            "t": 198.0,
            "v": 0.0
          },
          {
            "t": 200.0,
            "v": 0.0
          },
          {
            "t": 202.0,
            "v": 0.0
          },
          {
            "t": 204.0,
            "v": 0.0
          },
          {
            "t": 206.0,
            "v": 0.0
          },
          {
            "t": 208.0,
            "v": 0.0
          },
          {
            "t": 210.0,
            "v": 0.0
          },
          {
            "t": 212.0,
            "v": 0.0
          },
          {
            "t": 214.0,
            "v": 0.0
          },
          {
            "t": 216.0,
            "v": 0.0
          },
          {
            "t": 218.0,
            "v": 0.0
          },
          {
            "t": 220.0,
            "v": 0.0
          },
          {
            "t": 222.0,
            "v": 0.0
          },
          {
            "t": 224.0,
            "v": 0.0
          },
          {
            "t": 226.0,
            "v": 0.0
          },
          {
            "t": 228.0,
            "v": 0.0
          },
          {
            "t": 230.0,
            "v": 0.0
          }
        ],
        "write_kbs": [
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
            "v": 4.0
          },
          {
            "t": 74.0,
            "v": 0.0
          },
          {
            "t": 76.0,
            "v": 2.0
          },
          {
            "t": 78.0,
            "v": 2.0
          },
          {
            "t": 80.0,
            "v": 2.0
          },
          {
            "t": 82.0,
            "v": 4.0
          },
          {
            "t": 84.0,
            "v": 2.0
          },
          {
            "t": 86.0,
            "v": 2.0
          },
          {
            "t": 88.0,
            "v": 2.0
          },
          {
            "t": 90.0,
            "v": 2.0
          },
          {
            "t": 92.0,
            "v": 2.0
          },
          {
            "t": 94.0,
            "v": 2.0
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
            "v": 4.0
          },
          {
            "t": 110.0,
            "v": 2.0
          },
          {
            "t": 112.0,
            "v": 0.0
          },
          {
            "t": 114.0,
            "v": 2.0
          },
          {
            "t": 116.0,
            "v": 0.0
          },
          {
            "t": 118.0,
            "v": 2.0
          },
          {
            "t": 120.0,
            "v": 0.0
          },
          {
            "t": 122.0,
            "v": 2.0
          },
          {
            "t": 124.0,
            "v": 0.0
          },
          {
            "t": 126.0,
            "v": 2.0
          },
          {
            "t": 128.0,
            "v": 0.0
          },
          {
            "t": 130.0,
            "v": 0.0
          },
          {
            "t": 132.0,
            "v": 0.0
          },
          {
            "t": 134.0,
            "v": 0.0
          },
          {
            "t": 136.0,
            "v": 0.0
          },
          {
            "t": 138.0,
            "v": 16.0
          },
          {
            "t": 140.0,
            "v": 0.0
          },
          {
            "t": 142.0,
            "v": 0.0
          },
          {
            "t": 144.0,
            "v": 0.0
          },
          {
            "t": 146.0,
            "v": 0.0
          },
          {
            "t": 148.0,
            "v": 0.0
          },
          {
            "t": 150.0,
            "v": 0.0
          },
          {
            "t": 152.0,
            "v": 0.0
          },
          {
            "t": 154.0,
            "v": 0.0
          },
          {
            "t": 156.0,
            "v": 0.0
          },
          {
            "t": 158.0,
            "v": 0.0
          },
          {
            "t": 160.0,
            "v": 0.0
          },
          {
            "t": 162.0,
            "v": 16.0
          },
          {
            "t": 164.0,
            "v": 0.0
          },
          {
            "t": 166.0,
            "v": 0.0
          },
          {
            "t": 168.0,
            "v": 0.0
          },
          {
            "t": 170.0,
            "v": 0.0
          },
          {
            "t": 172.0,
            "v": 0.0
          },
          {
            "t": 174.0,
            "v": 0.0
          },
          {
            "t": 176.0,
            "v": 0.0
          },
          {
            "t": 178.0,
            "v": 0.0
          },
          {
            "t": 180.0,
            "v": 0.0
          },
          {
            "t": 182.0,
            "v": 0.0
          },
          {
            "t": 184.0,
            "v": 0.0
          },
          {
            "t": 186.0,
            "v": 0.0
          },
          {
            "t": 188.0,
            "v": 20.0
          },
          {
            "t": 190.0,
            "v": 0.0
          },
          {
            "t": 192.0,
            "v": 0.0
          },
          {
            "t": 194.0,
            "v": 0.0
          },
          {
            "t": 196.0,
            "v": 0.0
          },
          {
            "t": 198.0,
            "v": 0.0
          },
          {
            "t": 200.0,
            "v": 0.0
          },
          {
            "t": 202.0,
            "v": 0.0
          },
          {
            "t": 204.0,
            "v": 0.0
          },
          {
            "t": 206.0,
            "v": 0.0
          },
          {
            "t": 208.0,
            "v": 0.0
          },
          {
            "t": 210.0,
            "v": 0.0
          },
          {
            "t": 212.0,
            "v": 8.0
          },
          {
            "t": 214.0,
            "v": 0.0
          },
          {
            "t": 216.0,
            "v": 0.0
          },
          {
            "t": 218.0,
            "v": 0.0
          },
          {
            "t": 220.0,
            "v": 0.0
          },
          {
            "t": 222.0,
            "v": 0.0
          },
          {
            "t": 224.0,
            "v": 0.0
          },
          {
            "t": 226.0,
            "v": 0.0
          },
          {
            "t": 228.0,
            "v": 0.0
          },
          {
            "t": 230.0,
            "v": 0.0
          }
        ],
        "cswch": [
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
          },
          {
            "t": 114.0,
            "v": 0.0
          },
          {
            "t": 116.0,
            "v": 0.0
          },
          {
            "t": 118.0,
            "v": 0.0
          },
          {
            "t": 120.0,
            "v": 0.0
          },
          {
            "t": 122.0,
            "v": 0.0
          },
          {
            "t": 124.0,
            "v": 0.0
          },
          {
            "t": 126.0,
            "v": 0.0
          },
          {
            "t": 128.0,
            "v": 0.0
          },
          {
            "t": 130.0,
            "v": 0.0
          },
          {
            "t": 132.0,
            "v": 0.0
          },
          {
            "t": 134.0,
            "v": 0.0
          },
          {
            "t": 136.0,
            "v": 0.0
          },
          {
            "t": 138.0,
            "v": 0.0
          },
          {
            "t": 140.0,
            "v": 0.0
          },
          {
            "t": 142.0,
            "v": 0.0
          },
          {
            "t": 144.0,
            "v": 0.0
          },
          {
            "t": 146.0,
            "v": 0.0
          },
          {
            "t": 148.0,
            "v": 0.0
          },
          {
            "t": 150.0,
            "v": 0.0
          },
          {
            "t": 152.0,
            "v": 0.0
          },
          {
            "t": 154.0,
            "v": 0.0
          },
          {
            "t": 156.0,
            "v": 0.0
          },
          {
            "t": 158.0,
            "v": 0.0
          },
          {
            "t": 160.0,
            "v": 0.0
          },
          {
            "t": 162.0,
            "v": 0.0
          },
          {
            "t": 164.0,
            "v": 0.0
          },
          {
            "t": 166.0,
            "v": 0.0
          },
          {
            "t": 168.0,
            "v": 0.0
          },
          {
            "t": 170.0,
            "v": 0.0
          },
          {
            "t": 172.0,
            "v": 0.0
          },
          {
            "t": 174.0,
            "v": 0.0
          },
          {
            "t": 176.0,
            "v": 0.0
          },
          {
            "t": 178.0,
            "v": 0.0
          },
          {
            "t": 180.0,
            "v": 0.0
          },
          {
            "t": 182.0,
            "v": 0.0
          },
          {
            "t": 184.0,
            "v": 0.0
          },
          {
            "t": 186.0,
            "v": 0.0
          },
          {
            "t": 188.0,
            "v": 0.0
          },
          {
            "t": 190.0,
            "v": 0.0
          },
          {
            "t": 192.0,
            "v": 0.0
          },
          {
            "t": 194.0,
            "v": 0.0
          },
          {
            "t": 196.0,
            "v": 0.0
          },
          {
            "t": 198.0,
            "v": 0.0
          },
          {
            "t": 200.0,
            "v": 0.0
          },
          {
            "t": 202.0,
            "v": 0.0
          },
          {
            "t": 204.0,
            "v": 0.0
          },
          {
            "t": 206.0,
            "v": 0.0
          },
          {
            "t": 208.0,
            "v": 0.0
          },
          {
            "t": 210.0,
            "v": 0.0
          },
          {
            "t": 212.0,
            "v": 0.0
          },
          {
            "t": 214.0,
            "v": 0.0
          },
          {
            "t": 216.0,
            "v": 0.0
          },
          {
            "t": 218.0,
            "v": 0.0
          },
          {
            "t": 220.0,
            "v": 0.0
          },
          {
            "t": 222.0,
            "v": 0.0
          },
          {
            "t": 224.0,
            "v": 0.0
          },
          {
            "t": 226.0,
            "v": 0.0
          },
          {
            "t": 228.0,
            "v": 0.0
          },
          {
            "t": 230.0,
            "v": 0.0
          }
        ],
        "nvcswch": [
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
          },
          {
            "t": 114.0,
            "v": 0.0
          },
          {
            "t": 116.0,
            "v": 0.0
          },
          {
            "t": 118.0,
            "v": 0.0
          },
          {
            "t": 120.0,
            "v": 0.0
          },
          {
            "t": 122.0,
            "v": 0.0
          },
          {
            "t": 124.0,
            "v": 0.0
          },
          {
            "t": 126.0,
            "v": 0.0
          },
          {
            "t": 128.0,
            "v": 0.0
          },
          {
            "t": 130.0,
            "v": 0.0
          },
          {
            "t": 132.0,
            "v": 0.0
          },
          {
            "t": 134.0,
            "v": 0.0
          },
          {
            "t": 136.0,
            "v": 0.0
          },
          {
            "t": 138.0,
            "v": 0.0
          },
          {
            "t": 140.0,
            "v": 0.0
          },
          {
            "t": 142.0,
            "v": 0.0
          },
          {
            "t": 144.0,
            "v": 0.0
          },
          {
            "t": 146.0,
            "v": 0.0
          },
          {
            "t": 148.0,
            "v": 0.0
          },
          {
            "t": 150.0,
            "v": 0.0
          },
          {
            "t": 152.0,
            "v": 0.0
          },
          {
            "t": 154.0,
            "v": 0.0
          },
          {
            "t": 156.0,
            "v": 0.0
          },
          {
            "t": 158.0,
            "v": 0.0
          },
          {
            "t": 160.0,
            "v": 0.0
          },
          {
            "t": 162.0,
            "v": 0.0
          },
          {
            "t": 164.0,
            "v": 0.0
          },
          {
            "t": 166.0,
            "v": 0.0
          },
          {
            "t": 168.0,
            "v": 0.0
          },
          {
            "t": 170.0,
            "v": 0.0
          },
          {
            "t": 172.0,
            "v": 0.0
          },
          {
            "t": 174.0,
            "v": 0.0
          },
          {
            "t": 176.0,
            "v": 0.0
          },
          {
            "t": 178.0,
            "v": 0.0
          },
          {
            "t": 180.0,
            "v": 0.0
          },
          {
            "t": 182.0,
            "v": 0.0
          },
          {
            "t": 184.0,
            "v": 0.0
          },
          {
            "t": 186.0,
            "v": 0.0
          },
          {
            "t": 188.0,
            "v": 0.0
          },
          {
            "t": 190.0,
            "v": 0.0
          },
          {
            "t": 192.0,
            "v": 0.0
          },
          {
            "t": 194.0,
            "v": 0.0
          },
          {
            "t": 196.0,
            "v": 0.0
          },
          {
            "t": 198.0,
            "v": 0.0
          },
          {
            "t": 200.0,
            "v": 0.0
          },
          {
            "t": 202.0,
            "v": 0.0
          },
          {
            "t": 204.0,
            "v": 0.0
          },
          {
            "t": 206.0,
            "v": 0.0
          },
          {
            "t": 208.0,
            "v": 0.0
          },
          {
            "t": 210.0,
            "v": 0.0
          },
          {
            "t": 212.0,
            "v": 0.0
          },
          {
            "t": 214.0,
            "v": 0.0
          },
          {
            "t": 216.0,
            "v": 0.0
          },
          {
            "t": 218.0,
            "v": 0.0
          },
          {
            "t": 220.0,
            "v": 0.0
          },
          {
            "t": 222.0,
            "v": 0.0
          },
          {
            "t": 224.0,
            "v": 0.0
          },
          {
            "t": 226.0,
            "v": 0.0
          },
          {
            "t": 228.0,
            "v": 0.0
          },
          {
            "t": 230.0,
            "v": 0.0
          }
        ]
      },
      "client": {
        "cpu": [
          {
            "t": 75.0,
            "v": 0.07
          },
          {
            "t": 78.7,
            "v": 0.76
          },
          {
            "t": 82.4,
            "v": 1.35
          },
          {
            "t": 86.1,
            "v": 1.68
          },
          {
            "t": 89.8,
            "v": 1.8
          },
          {
            "t": 93.6,
            "v": 1.73
          },
          {
            "t": 97.3,
            "v": 1.73
          },
          {
            "t": 101.1,
            "v": 1.13
          },
          {
            "t": 104.8,
            "v": 0.27
          },
          {
            "t": 108.4,
            "v": 0.14
          },
          {
            "t": 112.1,
            "v": 1.03
          },
          {
            "t": 115.8,
            "v": 1.89
          },
          {
            "t": 119.5,
            "v": 2.26
          },
          {
            "t": 123.3,
            "v": 2.25
          },
          {
            "t": 127.1,
            "v": 2.39
          },
          {
            "t": 130.9,
            "v": 1.77
          },
          {
            "t": 134.6,
            "v": 0.4
          },
          {
            "t": 138.3,
            "v": 0.75
          },
          {
            "t": 142.6,
            "v": 15.6
          },
          {
            "t": 147.2,
            "v": 14.57
          },
          {
            "t": 151.9,
            "v": 14.59
          },
          {
            "t": 156.4,
            "v": 14.57
          },
          {
            "t": 160.9,
            "v": 4.24
          },
          {
            "t": 164.5,
            "v": 6.04
          },
          {
            "t": 169.2,
            "v": 11.75
          },
          {
            "t": 173.8,
            "v": 11.7
          },
          {
            "t": 178.4,
            "v": 11.81
          },
          {
            "t": 183.0,
            "v": 11.46
          },
          {
            "t": 190.5,
            "v": 11.32
          },
          {
            "t": 195.2,
            "v": 14.49
          },
          {
            "t": 199.8,
            "v": 14.22
          },
          {
            "t": 204.4,
            "v": 14.32
          },
          {
            "t": 209.0,
            "v": 10.76
          },
          {
            "t": 216.7,
            "v": 12.04
          },
          {
            "t": 221.3,
            "v": 11.92
          },
          {
            "t": 226.0,
            "v": 11.56
          },
          {
            "t": 230.6,
            "v": 11.89
          }
        ],
        "rss": [
          {
            "t": 3.5,
            "v": 11.1
          },
          {
            "t": 8.3,
            "v": 11.1
          },
          {
            "t": 13.0,
            "v": 11.1
          },
          {
            "t": 20.6,
            "v": 11.1
          },
          {
            "t": 25.2,
            "v": 11.1
          },
          {
            "t": 28.9,
            "v": 11.1
          },
          {
            "t": 33.9,
            "v": 11.1
          },
          {
            "t": 38.8,
            "v": 11.1
          },
          {
            "t": 43.6,
            "v": 11.1
          },
          {
            "t": 48.4,
            "v": 11.1
          },
          {
            "t": 52.1,
            "v": 11.1
          },
          {
            "t": 58.9,
            "v": 11.1
          },
          {
            "t": 67.9,
            "v": 11.1
          },
          {
            "t": 75.0,
            "v": 11.3
          },
          {
            "t": 78.7,
            "v": 12.3
          },
          {
            "t": 82.4,
            "v": 12.8
          },
          {
            "t": 86.1,
            "v": 13.1
          },
          {
            "t": 89.8,
            "v": 13.4
          },
          {
            "t": 93.6,
            "v": 13.5
          },
          {
            "t": 97.3,
            "v": 13.6
          },
          {
            "t": 101.1,
            "v": 13.6
          },
          {
            "t": 104.8,
            "v": 13.6
          },
          {
            "t": 108.4,
            "v": 14.9
          },
          {
            "t": 112.1,
            "v": 17.0
          },
          {
            "t": 115.8,
            "v": 18.1
          },
          {
            "t": 119.5,
            "v": 18.5
          },
          {
            "t": 123.3,
            "v": 18.5
          },
          {
            "t": 127.1,
            "v": 18.5
          },
          {
            "t": 130.9,
            "v": 18.0
          },
          {
            "t": 134.6,
            "v": 18.0
          },
          {
            "t": 138.3,
            "v": 18.8
          },
          {
            "t": 142.6,
            "v": 18.2
          },
          {
            "t": 147.2,
            "v": 18.4
          },
          {
            "t": 151.9,
            "v": 18.3
          },
          {
            "t": 156.4,
            "v": 18.0
          },
          {
            "t": 160.9,
            "v": 18.2
          },
          {
            "t": 164.5,
            "v": 19.8
          },
          {
            "t": 169.2,
            "v": 19.9
          },
          {
            "t": 173.8,
            "v": 19.9
          },
          {
            "t": 178.4,
            "v": 19.6
          },
          {
            "t": 183.0,
            "v": 19.6
          },
          {
            "t": 186.9,
            "v": 19.6
          },
          {
            "t": 190.5,
            "v": 22.5
          },
          {
            "t": 195.2,
            "v": 22.7
          },
          {
            "t": 199.8,
            "v": 22.6
          },
          {
            "t": 204.4,
            "v": 22.6
          },
          {
            "t": 209.0,
            "v": 22.4
          },
          {
            "t": 212.7,
            "v": 22.4
          },
          {
            "t": 216.7,
            "v": 22.9
          },
          {
            "t": 221.3,
            "v": 22.6
          },
          {
            "t": 226.0,
            "v": 22.1
          },
          {
            "t": 230.6,
            "v": 22.1
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
          },
          {
            "t": 114.0,
            "v": 0.0
          },
          {
            "t": 116.0,
            "v": 0.0
          },
          {
            "t": 118.0,
            "v": 0.0
          },
          {
            "t": 120.0,
            "v": 0.0
          },
          {
            "t": 122.0,
            "v": 0.0
          },
          {
            "t": 124.0,
            "v": 0.0
          },
          {
            "t": 126.0,
            "v": 0.0
          },
          {
            "t": 128.0,
            "v": 0.0
          },
          {
            "t": 130.0,
            "v": 0.0
          },
          {
            "t": 132.0,
            "v": 0.0
          },
          {
            "t": 134.0,
            "v": 0.0
          },
          {
            "t": 136.0,
            "v": 0.0
          },
          {
            "t": 138.0,
            "v": 0.0
          },
          {
            "t": 140.0,
            "v": 0.0
          },
          {
            "t": 142.0,
            "v": 0.0
          },
          {
            "t": 144.0,
            "v": 0.0
          },
          {
            "t": 146.0,
            "v": 0.0
          },
          {
            "t": 148.0,
            "v": 0.0
          },
          {
            "t": 150.0,
            "v": 0.0
          },
          {
            "t": 152.0,
            "v": 0.0
          },
          {
            "t": 154.0,
            "v": 0.0
          },
          {
            "t": 156.0,
            "v": 0.0
          },
          {
            "t": 158.0,
            "v": 0.0
          },
          {
            "t": 160.0,
            "v": 0.0
          },
          {
            "t": 162.0,
            "v": 0.0
          },
          {
            "t": 164.0,
            "v": 0.0
          },
          {
            "t": 166.0,
            "v": 0.0
          },
          {
            "t": 168.0,
            "v": 0.0
          },
          {
            "t": 170.0,
            "v": 0.0
          },
          {
            "t": 172.0,
            "v": 0.0
          },
          {
            "t": 174.0,
            "v": 0.0
          },
          {
            "t": 176.0,
            "v": 0.0
          },
          {
            "t": 178.0,
            "v": 0.0
          },
          {
            "t": 180.0,
            "v": 0.0
          },
          {
            "t": 182.0,
            "v": 0.0
          },
          {
            "t": 184.0,
            "v": 0.0
          },
          {
            "t": 186.0,
            "v": 0.0
          },
          {
            "t": 188.0,
            "v": 0.0
          },
          {
            "t": 190.0,
            "v": 0.0
          },
          {
            "t": 192.0,
            "v": 0.0
          },
          {
            "t": 194.0,
            "v": 0.0
          },
          {
            "t": 196.0,
            "v": 0.0
          },
          {
            "t": 198.0,
            "v": 0.0
          },
          {
            "t": 200.0,
            "v": 0.0
          },
          {
            "t": 202.0,
            "v": 0.0
          },
          {
            "t": 204.0,
            "v": 0.0
          },
          {
            "t": 206.0,
            "v": 0.0
          },
          {
            "t": 208.0,
            "v": 0.0
          },
          {
            "t": 210.0,
            "v": 0.0
          },
          {
            "t": 212.0,
            "v": 0.0
          },
          {
            "t": 214.0,
            "v": 0.0
          },
          {
            "t": 216.0,
            "v": 0.0
          },
          {
            "t": 218.0,
            "v": 0.0
          },
          {
            "t": 220.0,
            "v": 0.0
          },
          {
            "t": 222.0,
            "v": 0.0
          },
          {
            "t": 224.0,
            "v": 0.0
          },
          {
            "t": 226.0,
            "v": 0.0
          },
          {
            "t": 228.0,
            "v": 0.0
          },
          {
            "t": 230.0,
            "v": 0.0
          },
          {
            "t": 232.0,
            "v": 0.0
          }
        ],
        "write_kbs": [
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
            "v": 2.0
          },
          {
            "t": 74.0,
            "v": 0.0
          },
          {
            "t": 76.0,
            "v": 6.0
          },
          {
            "t": 78.0,
            "v": 6.0
          },
          {
            "t": 80.0,
            "v": 6.0
          },
          {
            "t": 82.0,
            "v": 6.0
          },
          {
            "t": 84.0,
            "v": 4.0
          },
          {
            "t": 86.0,
            "v": 6.0
          },
          {
            "t": 88.0,
            "v": 6.0
          },
          {
            "t": 90.0,
            "v": 6.0
          },
          {
            "t": 92.0,
            "v": 6.0
          },
          {
            "t": 94.0,
            "v": 4.0
          },
          {
            "t": 96.0,
            "v": 2.0
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
            "v": 4.0
          },
          {
            "t": 110.0,
            "v": 4.0
          },
          {
            "t": 112.0,
            "v": 4.0
          },
          {
            "t": 114.0,
            "v": 2.0
          },
          {
            "t": 116.0,
            "v": 4.0
          },
          {
            "t": 118.0,
            "v": 2.0
          },
          {
            "t": 120.0,
            "v": 2.0
          },
          {
            "t": 122.0,
            "v": 4.0
          },
          {
            "t": 124.0,
            "v": 2.0
          },
          {
            "t": 126.0,
            "v": 4.0
          },
          {
            "t": 128.0,
            "v": 2.0
          },
          {
            "t": 130.0,
            "v": 0.0
          },
          {
            "t": 132.0,
            "v": 0.0
          },
          {
            "t": 134.0,
            "v": 0.0
          },
          {
            "t": 136.0,
            "v": 0.0
          },
          {
            "t": 138.0,
            "v": 18.0
          },
          {
            "t": 140.0,
            "v": 0.0
          },
          {
            "t": 142.0,
            "v": 0.0
          },
          {
            "t": 144.0,
            "v": 0.0
          },
          {
            "t": 146.0,
            "v": 0.0
          },
          {
            "t": 148.0,
            "v": 0.0
          },
          {
            "t": 150.0,
            "v": 0.0
          },
          {
            "t": 152.0,
            "v": 0.0
          },
          {
            "t": 154.0,
            "v": 0.0
          },
          {
            "t": 156.0,
            "v": 0.0
          },
          {
            "t": 158.0,
            "v": 0.0
          },
          {
            "t": 160.0,
            "v": 0.0
          },
          {
            "t": 162.0,
            "v": 0.0
          },
          {
            "t": 164.0,
            "v": 0.0
          },
          {
            "t": 166.0,
            "v": 0.0
          },
          {
            "t": 168.0,
            "v": 0.0
          },
          {
            "t": 170.0,
            "v": 0.0
          },
          {
            "t": 172.0,
            "v": 0.0
          },
          {
            "t": 174.0,
            "v": 0.0
          },
          {
            "t": 176.0,
            "v": 0.0
          },
          {
            "t": 178.0,
            "v": 0.0
          },
          {
            "t": 180.0,
            "v": 0.0
          },
          {
            "t": 182.0,
            "v": 0.0
          },
          {
            "t": 184.0,
            "v": 0.0
          },
          {
            "t": 186.0,
            "v": 0.0
          },
          {
            "t": 188.0,
            "v": 28.0
          },
          {
            "t": 190.0,
            "v": 0.0
          },
          {
            "t": 192.0,
            "v": 0.0
          },
          {
            "t": 194.0,
            "v": 0.0
          },
          {
            "t": 196.0,
            "v": 0.0
          },
          {
            "t": 198.0,
            "v": 0.0
          },
          {
            "t": 200.0,
            "v": 0.0
          },
          {
            "t": 202.0,
            "v": 0.0
          },
          {
            "t": 204.0,
            "v": 0.0
          },
          {
            "t": 206.0,
            "v": 0.0
          },
          {
            "t": 208.0,
            "v": 0.0
          },
          {
            "t": 210.0,
            "v": 0.0
          },
          {
            "t": 212.0,
            "v": 0.0
          },
          {
            "t": 214.0,
            "v": 0.0
          },
          {
            "t": 216.0,
            "v": 0.0
          },
          {
            "t": 218.0,
            "v": 0.0
          },
          {
            "t": 220.0,
            "v": 0.0
          },
          {
            "t": 222.0,
            "v": 0.0
          },
          {
            "t": 224.0,
            "v": 0.0
          },
          {
            "t": 226.0,
            "v": 0.0
          },
          {
            "t": 228.0,
            "v": 0.0
          },
          {
            "t": 230.0,
            "v": 0.0
          },
          {
            "t": 232.0,
            "v": 0.0
          }
        ],
        "cswch": [
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
            "v": 1.5
          },
          {
            "t": 74.0,
            "v": 2.0
          },
          {
            "t": 76.0,
            "v": 8.5
          },
          {
            "t": 78.0,
            "v": 12.5
          },
          {
            "t": 80.0,
            "v": 14.0
          },
          {
            "t": 82.0,
            "v": 12.5
          },
          {
            "t": 84.0,
            "v": 13.0
          },
          {
            "t": 86.0,
            "v": 12.5
          },
          {
            "t": 88.0,
            "v": 12.0
          },
          {
            "t": 90.0,
            "v": 10.5
          },
          {
            "t": 92.0,
            "v": 13.0
          },
          {
            "t": 94.0,
            "v": 15.5
          },
          {
            "t": 96.0,
            "v": 4.5
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
            "v": 8.5
          },
          {
            "t": 110.0,
            "v": 9.0
          },
          {
            "t": 112.0,
            "v": 10.5
          },
          {
            "t": 114.0,
            "v": 8.0
          },
          {
            "t": 116.0,
            "v": 7.5
          },
          {
            "t": 118.0,
            "v": 5.5
          },
          {
            "t": 120.0,
            "v": 6.5
          },
          {
            "t": 122.0,
            "v": 10.0
          },
          {
            "t": 124.0,
            "v": 7.0
          },
          {
            "t": 126.0,
            "v": 8.5
          },
          {
            "t": 128.0,
            "v": 4.0
          },
          {
            "t": 130.0,
            "v": 0.0
          },
          {
            "t": 132.0,
            "v": 0.0
          },
          {
            "t": 134.0,
            "v": 0.0
          },
          {
            "t": 136.0,
            "v": 0.0
          },
          {
            "t": 138.0,
            "v": 42.5
          },
          {
            "t": 140.0,
            "v": 0.0
          },
          {
            "t": 142.0,
            "v": 0.0
          },
          {
            "t": 144.0,
            "v": 0.0
          },
          {
            "t": 146.0,
            "v": 0.0
          },
          {
            "t": 148.0,
            "v": 0.0
          },
          {
            "t": 150.0,
            "v": 0.0
          },
          {
            "t": 152.0,
            "v": 0.0
          },
          {
            "t": 154.0,
            "v": 0.0
          },
          {
            "t": 156.0,
            "v": 0.0
          },
          {
            "t": 158.0,
            "v": 0.0
          },
          {
            "t": 160.0,
            "v": 0.0
          },
          {
            "t": 162.0,
            "v": 0.0
          },
          {
            "t": 164.0,
            "v": 0.0
          },
          {
            "t": 166.0,
            "v": 0.0
          },
          {
            "t": 168.0,
            "v": 0.0
          },
          {
            "t": 170.0,
            "v": 0.0
          },
          {
            "t": 172.0,
            "v": 0.0
          },
          {
            "t": 174.0,
            "v": 0.0
          },
          {
            "t": 176.0,
            "v": 0.0
          },
          {
            "t": 178.0,
            "v": 0.0
          },
          {
            "t": 180.0,
            "v": 0.0
          },
          {
            "t": 182.0,
            "v": 0.0
          },
          {
            "t": 184.0,
            "v": 0.0
          },
          {
            "t": 186.0,
            "v": 0.0
          },
          {
            "t": 188.0,
            "v": 62.5
          },
          {
            "t": 190.0,
            "v": 0.0
          },
          {
            "t": 192.0,
            "v": 0.0
          },
          {
            "t": 194.0,
            "v": 0.0
          },
          {
            "t": 196.0,
            "v": 0.0
          },
          {
            "t": 198.0,
            "v": 0.0
          },
          {
            "t": 200.0,
            "v": 0.0
          },
          {
            "t": 202.0,
            "v": 0.0
          },
          {
            "t": 204.0,
            "v": 0.0
          },
          {
            "t": 206.0,
            "v": 0.0
          },
          {
            "t": 208.0,
            "v": 0.0
          },
          {
            "t": 210.0,
            "v": 0.0
          },
          {
            "t": 212.0,
            "v": 0.0
          },
          {
            "t": 214.0,
            "v": 0.0
          },
          {
            "t": 216.0,
            "v": 0.0
          },
          {
            "t": 218.0,
            "v": 0.0
          },
          {
            "t": 220.0,
            "v": 0.0
          },
          {
            "t": 222.0,
            "v": 0.0
          },
          {
            "t": 224.0,
            "v": 0.0
          },
          {
            "t": 226.0,
            "v": 0.0
          },
          {
            "t": 228.0,
            "v": 0.0
          },
          {
            "t": 230.0,
            "v": 0.0
          },
          {
            "t": 232.0,
            "v": 0.0
          }
        ],
        "nvcswch": [
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
            "v": 0.5
          },
          {
            "t": 78.0,
            "v": 2.5
          },
          {
            "t": 80.0,
            "v": 0.5
          },
          {
            "t": 82.0,
            "v": 0.0
          },
          {
            "t": 84.0,
            "v": 0.5
          },
          {
            "t": 86.0,
            "v": 0.0
          },
          {
            "t": 88.0,
            "v": 1.5
          },
          {
            "t": 90.0,
            "v": 1.0
          },
          {
            "t": 92.0,
            "v": 1.5
          },
          {
            "t": 94.0,
            "v": 2.5
          },
          {
            "t": 96.0,
            "v": 0.5
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
            "v": 0.5
          },
          {
            "t": 110.0,
            "v": 1.5
          },
          {
            "t": 112.0,
            "v": 0.0
          },
          {
            "t": 114.0,
            "v": 0.5
          },
          {
            "t": 116.0,
            "v": 0.5
          },
          {
            "t": 118.0,
            "v": 2.5
          },
          {
            "t": 120.0,
            "v": 1.0
          },
          {
            "t": 122.0,
            "v": 0.5
          },
          {
            "t": 124.0,
            "v": 0.5
          },
          {
            "t": 126.0,
            "v": 0.0
          },
          {
            "t": 128.0,
            "v": 1.0
          },
          {
            "t": 130.0,
            "v": 0.0
          },
          {
            "t": 132.0,
            "v": 0.0
          },
          {
            "t": 134.0,
            "v": 0.0
          },
          {
            "t": 136.0,
            "v": 0.0
          },
          {
            "t": 138.0,
            "v": 5.0
          },
          {
            "t": 140.0,
            "v": 0.0
          },
          {
            "t": 142.0,
            "v": 0.0
          },
          {
            "t": 144.0,
            "v": 0.0
          },
          {
            "t": 146.0,
            "v": 0.0
          },
          {
            "t": 148.0,
            "v": 0.0
          },
          {
            "t": 150.0,
            "v": 0.0
          },
          {
            "t": 152.0,
            "v": 0.0
          },
          {
            "t": 154.0,
            "v": 0.0
          },
          {
            "t": 156.0,
            "v": 0.0
          },
          {
            "t": 158.0,
            "v": 0.0
          },
          {
            "t": 160.0,
            "v": 0.0
          },
          {
            "t": 162.0,
            "v": 0.0
          },
          {
            "t": 164.0,
            "v": 0.0
          },
          {
            "t": 166.0,
            "v": 0.0
          },
          {
            "t": 168.0,
            "v": 0.0
          },
          {
            "t": 170.0,
            "v": 0.0
          },
          {
            "t": 172.0,
            "v": 0.0
          },
          {
            "t": 174.0,
            "v": 0.0
          },
          {
            "t": 176.0,
            "v": 0.0
          },
          {
            "t": 178.0,
            "v": 0.0
          },
          {
            "t": 180.0,
            "v": 0.0
          },
          {
            "t": 182.0,
            "v": 0.0
          },
          {
            "t": 184.0,
            "v": 0.0
          },
          {
            "t": 186.0,
            "v": 0.0
          },
          {
            "t": 188.0,
            "v": 4.5
          },
          {
            "t": 190.0,
            "v": 0.0
          },
          {
            "t": 192.0,
            "v": 0.0
          },
          {
            "t": 194.0,
            "v": 0.0
          },
          {
            "t": 196.0,
            "v": 0.0
          },
          {
            "t": 198.0,
            "v": 0.0
          },
          {
            "t": 200.0,
            "v": 0.0
          },
          {
            "t": 202.0,
            "v": 0.0
          },
          {
            "t": 204.0,
            "v": 0.0
          },
          {
            "t": 206.0,
            "v": 0.0
          },
          {
            "t": 208.0,
            "v": 0.0
          },
          {
            "t": 210.0,
            "v": 0.0
          },
          {
            "t": 212.0,
            "v": 0.0
          },
          {
            "t": 214.0,
            "v": 0.0
          },
          {
            "t": 216.0,
            "v": 0.0
          },
          {
            "t": 218.0,
            "v": 0.0
          },
          {
            "t": 220.0,
            "v": 0.0
          },
          {
            "t": 222.0,
            "v": 0.0
          },
          {
            "t": 224.0,
            "v": 0.0
          },
          {
            "t": 226.0,
            "v": 0.0
          },
          {
            "t": 228.0,
            "v": 0.0
          },
          {
            "t": 230.0,
            "v": 0.0
          },
          {
            "t": 232.0,
            "v": 0.0
          }
        ]
      },
      "http_echo": {
        "cpu": [
          {
            "t": 3.5,
            "v": 2.35
          },
          {
            "t": 8.3,
            "v": 4.35
          },
          {
            "t": 13.0,
            "v": 5.09
          },
          {
            "t": 20.6,
            "v": 4.66
          },
          {
            "t": 25.2,
            "v": 1.09
          },
          {
            "t": 28.9,
            "v": 2.6
          },
          {
            "t": 33.9,
            "v": 3.91
          },
          {
            "t": 38.8,
            "v": 4.06
          },
          {
            "t": 43.6,
            "v": 4.13
          },
          {
            "t": 48.4,
            "v": 2.49
          },
          {
            "t": 52.1,
            "v": 0.48
          },
          {
            "t": 58.9,
            "v": 7.0
          },
          {
            "t": 67.9,
            "v": 6.04
          },
          {
            "t": 75.0,
            "v": 2.43
          },
          {
            "t": 78.7,
            "v": 0.14
          },
          {
            "t": 82.4,
            "v": 0.14
          },
          {
            "t": 86.1,
            "v": 0.4
          },
          {
            "t": 89.8,
            "v": 0.27
          },
          {
            "t": 93.6,
            "v": 0.33
          },
          {
            "t": 97.3,
            "v": 0.33
          },
          {
            "t": 101.1,
            "v": 0.33
          },
          {
            "t": 104.8,
            "v": 0.07
          },
          {
            "t": 112.1,
            "v": 0.27
          },
          {
            "t": 115.8,
            "v": 0.34
          },
          {
            "t": 119.5,
            "v": 0.33
          },
          {
            "t": 123.3,
            "v": 0.26
          },
          {
            "t": 127.1,
            "v": 0.33
          },
          {
            "t": 130.9,
            "v": 0.13
          },
          {
            "t": 138.3,
            "v": 0.07
          },
          {
            "t": 142.6,
            "v": 3.67
          },
          {
            "t": 147.2,
            "v": 3.47
          },
          {
            "t": 151.9,
            "v": 3.46
          },
          {
            "t": 156.4,
            "v": 3.51
          },
          {
            "t": 160.9,
            "v": 1.17
          },
          {
            "t": 164.5,
            "v": 1.65
          },
          {
            "t": 169.2,
            "v": 3.47
          },
          {
            "t": 173.8,
            "v": 3.52
          },
          {
            "t": 178.4,
            "v": 3.5
          },
          {
            "t": 183.0,
            "v": 3.42
          },
          {
            "t": 190.5,
            "v": 2.54
          },
          {
            "t": 195.2,
            "v": 3.5
          },
          {
            "t": 199.8,
            "v": 3.43
          },
          {
            "t": 204.4,
            "v": 3.5
          },
          {
            "t": 209.0,
            "v": 2.66
          },
          {
            "t": 216.7,
            "v": 3.43
          },
          {
            "t": 221.3,
            "v": 3.47
          },
          {
            "t": 226.0,
            "v": 3.44
          },
          {
            "t": 230.6,
            "v": 3.54
          }
        ],
        "rss": [
          {
            "t": 3.5,
            "v": 5.5
          },
          {
            "t": 8.3,
            "v": 6.4
          },
          {
            "t": 13.0,
            "v": 9.4
          },
          {
            "t": 20.6,
            "v": 12.2
          },
          {
            "t": 25.2,
            "v": 12.2
          },
          {
            "t": 28.9,
            "v": 12.2
          },
          {
            "t": 33.9,
            "v": 12.2
          },
          {
            "t": 38.8,
            "v": 12.2
          },
          {
            "t": 43.6,
            "v": 12.2
          },
          {
            "t": 48.4,
            "v": 12.2
          },
          {
            "t": 52.1,
            "v": 12.3
          },
          {
            "t": 58.9,
            "v": 15.5
          },
          {
            "t": 67.9,
            "v": 22.8
          },
          {
            "t": 75.0,
            "v": 20.5
          },
          {
            "t": 78.7,
            "v": 20.5
          },
          {
            "t": 82.4,
            "v": 20.5
          },
          {
            "t": 86.1,
            "v": 20.5
          },
          {
            "t": 89.8,
            "v": 20.5
          },
          {
            "t": 93.6,
            "v": 20.5
          },
          {
            "t": 97.3,
            "v": 20.5
          },
          {
            "t": 101.1,
            "v": 20.5
          },
          {
            "t": 104.8,
            "v": 20.5
          },
          {
            "t": 108.4,
            "v": 20.5
          },
          {
            "t": 112.1,
            "v": 20.6
          },
          {
            "t": 115.8,
            "v": 20.6
          },
          {
            "t": 119.5,
            "v": 20.6
          },
          {
            "t": 123.3,
            "v": 20.6
          },
          {
            "t": 127.1,
            "v": 20.6
          },
          {
            "t": 130.9,
            "v": 20.6
          },
          {
            "t": 134.6,
            "v": 20.6
          },
          {
            "t": 138.3,
            "v": 20.6
          },
          {
            "t": 142.6,
            "v": 20.6
          },
          {
            "t": 147.2,
            "v": 20.6
          },
          {
            "t": 151.9,
            "v": 20.6
          },
          {
            "t": 156.4,
            "v": 20.6
          },
          {
            "t": 160.9,
            "v": 19.9
          },
          {
            "t": 164.5,
            "v": 19.9
          },
          {
            "t": 169.2,
            "v": 19.9
          },
          {
            "t": 173.8,
            "v": 19.9
          },
          {
            "t": 178.4,
            "v": 19.9
          },
          {
            "t": 183.0,
            "v": 19.9
          },
          {
            "t": 186.9,
            "v": 19.9
          },
          {
            "t": 190.5,
            "v": 19.9
          },
          {
            "t": 195.2,
            "v": 19.9
          },
          {
            "t": 199.8,
            "v": 19.9
          },
          {
            "t": 204.4,
            "v": 19.9
          },
          {
            "t": 209.0,
            "v": 19.9
          },
          {
            "t": 212.7,
            "v": 19.9
          },
          {
            "t": 216.7,
            "v": 19.9
          },
          {
            "t": 221.3,
            "v": 19.9
          },
          {
            "t": 226.0,
            "v": 19.9
          },
          {
            "t": 230.6,
            "v": 19.9
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
          },
          {
            "t": 114.0,
            "v": 0.0
          },
          {
            "t": 116.0,
            "v": 0.0
          },
          {
            "t": 118.0,
            "v": 0.0
          },
          {
            "t": 120.0,
            "v": 0.0
          },
          {
            "t": 122.0,
            "v": 0.0
          },
          {
            "t": 124.0,
            "v": 0.0
          },
          {
            "t": 126.0,
            "v": 0.0
          },
          {
            "t": 128.0,
            "v": 0.0
          },
          {
            "t": 130.0,
            "v": 0.0
          },
          {
            "t": 132.0,
            "v": 0.0
          },
          {
            "t": 134.0,
            "v": 0.0
          },
          {
            "t": 136.0,
            "v": 0.0
          },
          {
            "t": 138.0,
            "v": 0.0
          },
          {
            "t": 140.0,
            "v": 0.0
          },
          {
            "t": 142.0,
            "v": 0.0
          },
          {
            "t": 144.0,
            "v": 0.0
          },
          {
            "t": 146.0,
            "v": 0.0
          },
          {
            "t": 148.0,
            "v": 0.0
          },
          {
            "t": 150.0,
            "v": 0.0
          },
          {
            "t": 152.0,
            "v": 0.0
          },
          {
            "t": 154.0,
            "v": 0.0
          },
          {
            "t": 156.0,
            "v": 0.0
          },
          {
            "t": 158.0,
            "v": 0.0
          },
          {
            "t": 160.0,
            "v": 0.0
          },
          {
            "t": 162.0,
            "v": 0.0
          },
          {
            "t": 164.0,
            "v": 0.0
          },
          {
            "t": 166.0,
            "v": 0.0
          },
          {
            "t": 168.0,
            "v": 0.0
          },
          {
            "t": 170.0,
            "v": 0.0
          },
          {
            "t": 172.0,
            "v": 0.0
          },
          {
            "t": 174.0,
            "v": 0.0
          },
          {
            "t": 176.0,
            "v": 0.0
          },
          {
            "t": 178.0,
            "v": 0.0
          },
          {
            "t": 180.0,
            "v": 0.0
          },
          {
            "t": 182.0,
            "v": 0.0
          },
          {
            "t": 184.0,
            "v": 0.0
          },
          {
            "t": 186.0,
            "v": 0.0
          },
          {
            "t": 188.0,
            "v": 0.0
          },
          {
            "t": 190.0,
            "v": 0.0
          },
          {
            "t": 192.0,
            "v": 0.0
          },
          {
            "t": 194.0,
            "v": 0.0
          },
          {
            "t": 196.0,
            "v": 0.0
          },
          {
            "t": 198.0,
            "v": 0.0
          },
          {
            "t": 200.0,
            "v": 0.0
          },
          {
            "t": 202.0,
            "v": 0.0
          },
          {
            "t": 204.0,
            "v": 0.0
          },
          {
            "t": 206.0,
            "v": 0.0
          },
          {
            "t": 208.0,
            "v": 0.0
          },
          {
            "t": 210.0,
            "v": 0.0
          },
          {
            "t": 212.0,
            "v": 0.0
          },
          {
            "t": 214.0,
            "v": 0.0
          },
          {
            "t": 216.0,
            "v": 0.0
          },
          {
            "t": 218.0,
            "v": 0.0
          },
          {
            "t": 220.0,
            "v": 0.0
          },
          {
            "t": 222.0,
            "v": 0.0
          },
          {
            "t": 224.0,
            "v": 0.0
          },
          {
            "t": 226.0,
            "v": 0.0
          },
          {
            "t": 228.0,
            "v": 0.0
          },
          {
            "t": 230.0,
            "v": 0.0
          },
          {
            "t": 232.0,
            "v": 0.0
          }
        ],
        "write_kbs": [
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
          },
          {
            "t": 114.0,
            "v": 0.0
          },
          {
            "t": 116.0,
            "v": 0.0
          },
          {
            "t": 118.0,
            "v": 0.0
          },
          {
            "t": 120.0,
            "v": 0.0
          },
          {
            "t": 122.0,
            "v": 0.0
          },
          {
            "t": 124.0,
            "v": 0.0
          },
          {
            "t": 126.0,
            "v": 0.0
          },
          {
            "t": 128.0,
            "v": 0.0
          },
          {
            "t": 130.0,
            "v": 0.0
          },
          {
            "t": 132.0,
            "v": 0.0
          },
          {
            "t": 134.0,
            "v": 0.0
          },
          {
            "t": 136.0,
            "v": 0.0
          },
          {
            "t": 138.0,
            "v": 0.0
          },
          {
            "t": 140.0,
            "v": 0.0
          },
          {
            "t": 142.0,
            "v": 0.0
          },
          {
            "t": 144.0,
            "v": 0.0
          },
          {
            "t": 146.0,
            "v": 0.0
          },
          {
            "t": 148.0,
            "v": 0.0
          },
          {
            "t": 150.0,
            "v": 0.0
          },
          {
            "t": 152.0,
            "v": 0.0
          },
          {
            "t": 154.0,
            "v": 0.0
          },
          {
            "t": 156.0,
            "v": 0.0
          },
          {
            "t": 158.0,
            "v": 0.0
          },
          {
            "t": 160.0,
            "v": 0.0
          },
          {
            "t": 162.0,
            "v": 0.0
          },
          {
            "t": 164.0,
            "v": 0.0
          },
          {
            "t": 166.0,
            "v": 0.0
          },
          {
            "t": 168.0,
            "v": 0.0
          },
          {
            "t": 170.0,
            "v": 0.0
          },
          {
            "t": 172.0,
            "v": 0.0
          },
          {
            "t": 174.0,
            "v": 0.0
          },
          {
            "t": 176.0,
            "v": 0.0
          },
          {
            "t": 178.0,
            "v": 0.0
          },
          {
            "t": 180.0,
            "v": 0.0
          },
          {
            "t": 182.0,
            "v": 0.0
          },
          {
            "t": 184.0,
            "v": 0.0
          },
          {
            "t": 186.0,
            "v": 0.0
          },
          {
            "t": 188.0,
            "v": 0.0
          },
          {
            "t": 190.0,
            "v": 0.0
          },
          {
            "t": 192.0,
            "v": 0.0
          },
          {
            "t": 194.0,
            "v": 0.0
          },
          {
            "t": 196.0,
            "v": 0.0
          },
          {
            "t": 198.0,
            "v": 0.0
          },
          {
            "t": 200.0,
            "v": 0.0
          },
          {
            "t": 202.0,
            "v": 0.0
          },
          {
            "t": 204.0,
            "v": 0.0
          },
          {
            "t": 206.0,
            "v": 0.0
          },
          {
            "t": 208.0,
            "v": 0.0
          },
          {
            "t": 210.0,
            "v": 0.0
          },
          {
            "t": 212.0,
            "v": 0.0
          },
          {
            "t": 214.0,
            "v": 0.0
          },
          {
            "t": 216.0,
            "v": 0.0
          },
          {
            "t": 218.0,
            "v": 0.0
          },
          {
            "t": 220.0,
            "v": 0.0
          },
          {
            "t": 222.0,
            "v": 0.0
          },
          {
            "t": 224.0,
            "v": 0.0
          },
          {
            "t": 226.0,
            "v": 0.0
          },
          {
            "t": 228.0,
            "v": 0.0
          },
          {
            "t": 230.0,
            "v": 0.0
          },
          {
            "t": 232.0,
            "v": 0.0
          }
        ],
        "cswch": [
          {
            "t": 0.0,
            "v": 0.0
          },
          {
            "t": 2.0,
            "v": 68.5
          },
          {
            "t": 4.0,
            "v": 127.0
          },
          {
            "t": 6.0,
            "v": 75.0
          },
          {
            "t": 8.0,
            "v": 110.5
          },
          {
            "t": 10.0,
            "v": 108.5
          },
          {
            "t": 12.0,
            "v": 285.0
          },
          {
            "t": 14.0,
            "v": 294.0
          },
          {
            "t": 16.0,
            "v": 368.2
          },
          {
            "t": 18.0,
            "v": 210.5
          },
          {
            "t": 20.0,
            "v": 176.0
          },
          {
            "t": 22.0,
            "v": 86.0
          },
          {
            "t": 24.0,
            "v": 0.0
          },
          {
            "t": 26.0,
            "v": 7.5
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
            "v": 0.5
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
            "v": 29.0
          },
          {
            "t": 54.0,
            "v": 51.2
          },
          {
            "t": 56.0,
            "v": 173.5
          },
          {
            "t": 58.0,
            "v": 116.5
          },
          {
            "t": 60.0,
            "v": 216.5
          },
          {
            "t": 62.0,
            "v": 115.5
          },
          {
            "t": 64.0,
            "v": 172.6
          },
          {
            "t": 66.0,
            "v": 221.5
          },
          {
            "t": 68.0,
            "v": 175.5
          },
          {
            "t": 70.0,
            "v": 340.5
          },
          {
            "t": 72.0,
            "v": 38.5
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
            "v": 0.5
          },
          {
            "t": 80.0,
            "v": 0.5
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
            "v": 0.5
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
          },
          {
            "t": 116.0,
            "v": 0.0
          },
          {
            "t": 118.0,
            "v": 0.0
          },
          {
            "t": 120.0,
            "v": 0.0
          },
          {
            "t": 122.0,
            "v": 0.0
          },
          {
            "t": 124.0,
            "v": 0.0
          },
          {
            "t": 126.0,
            "v": 0.0
          },
          {
            "t": 128.0,
            "v": 0.0
          },
          {
            "t": 130.0,
            "v": 0.0
          },
          {
            "t": 132.0,
            "v": 0.0
          },
          {
            "t": 134.0,
            "v": 0.0
          },
          {
            "t": 136.0,
            "v": 0.0
          },
          {
            "t": 138.0,
            "v": 9.5
          },
          {
            "t": 140.0,
            "v": 1.5
          },
          {
            "t": 142.0,
            "v": 4.0
          },
          {
            "t": 144.0,
            "v": 2.5
          },
          {
            "t": 146.0,
            "v": 0.0
          },
          {
            "t": 148.0,
            "v": 3.0
          },
          {
            "t": 150.0,
            "v": 0.0
          },
          {
            "t": 152.0,
            "v": 4.0
          },
          {
            "t": 154.0,
            "v": 2.0
          },
          {
            "t": 156.0,
            "v": 17.5
          },
          {
            "t": 158.0,
            "v": 0.0
          },
          {
            "t": 160.0,
            "v": 0.0
          },
          {
            "t": 162.0,
            "v": 2.5
          },
          {
            "t": 164.0,
            "v": 6.5
          },
          {
            "t": 166.0,
            "v": 7.5
          },
          {
            "t": 168.0,
            "v": 1.0
          },
          {
            "t": 170.0,
            "v": 11.5
          },
          {
            "t": 172.0,
            "v": 3.5
          },
          {
            "t": 174.0,
            "v": 7.5
          },
          {
            "t": 176.0,
            "v": 3.5
          },
          {
            "t": 178.0,
            "v": 7.0
          },
          {
            "t": 180.0,
            "v": 0.0
          },
          {
            "t": 182.0,
            "v": 2.5
          },
          {
            "t": 184.0,
            "v": 0.0
          },
          {
            "t": 186.0,
            "v": 0.0
          },
          {
            "t": 188.0,
            "v": 6.0
          },
          {
            "t": 190.0,
            "v": 6.5
          },
          {
            "t": 192.0,
            "v": 1.5
          },
          {
            "t": 194.0,
            "v": 2.5
          },
          {
            "t": 196.0,
            "v": 14.0
          },
          {
            "t": 198.0,
            "v": 3.5
          },
          {
            "t": 200.0,
            "v": 7.5
          },
          {
            "t": 202.0,
            "v": 2.0
          },
          {
            "t": 204.0,
            "v": 2.0
          },
          {
            "t": 206.0,
            "v": 0.5
          },
          {
            "t": 208.0,
            "v": 0.0
          },
          {
            "t": 210.0,
            "v": 0.0
          },
          {
            "t": 212.0,
            "v": 1.0
          },
          {
            "t": 214.0,
            "v": 3.0
          },
          {
            "t": 216.0,
            "v": 6.0
          },
          {
            "t": 218.0,
            "v": 4.0
          },
          {
            "t": 220.0,
            "v": 4.0
          },
          {
            "t": 222.0,
            "v": 0.0
          },
          {
            "t": 224.0,
            "v": 3.0
          },
          {
            "t": 226.0,
            "v": 11.5
          },
          {
            "t": 228.0,
            "v": 2.0
          },
          {
            "t": 230.0,
            "v": 8.0
          },
          {
            "t": 232.0,
            "v": 1.0
          }
        ],
        "nvcswch": [
          {
            "t": 0.0,
            "v": 0.0
          },
          {
            "t": 2.0,
            "v": 15.0
          },
          {
            "t": 4.0,
            "v": 7.5
          },
          {
            "t": 6.0,
            "v": 5.0
          },
          {
            "t": 8.0,
            "v": 24.5
          },
          {
            "t": 10.0,
            "v": 16.0
          },
          {
            "t": 12.0,
            "v": 98.0
          },
          {
            "t": 14.0,
            "v": 164.5
          },
          {
            "t": 16.0,
            "v": 300.0
          },
          {
            "t": 18.0,
            "v": 107.0
          },
          {
            "t": 20.0,
            "v": 91.5
          },
          {
            "t": 22.0,
            "v": 43.5
          },
          {
            "t": 24.0,
            "v": 0.0
          },
          {
            "t": 26.0,
            "v": 1.0
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
            "v": 1.0
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
            "v": 2.5
          },
          {
            "t": 54.0,
            "v": 2.5
          },
          {
            "t": 56.0,
            "v": 55.5
          },
          {
            "t": 58.0,
            "v": 53.0
          },
          {
            "t": 60.0,
            "v": 89.5
          },
          {
            "t": 62.0,
            "v": 32.5
          },
          {
            "t": 64.0,
            "v": 50.2
          },
          {
            "t": 66.0,
            "v": 148.0
          },
          {
            "t": 68.0,
            "v": 70.5
          },
          {
            "t": 70.0,
            "v": 169.5
          },
          {
            "t": 72.0,
            "v": 5.5
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
          },
          {
            "t": 116.0,
            "v": 0.0
          },
          {
            "t": 118.0,
            "v": 0.0
          },
          {
            "t": 120.0,
            "v": 0.0
          },
          {
            "t": 122.0,
            "v": 0.0
          },
          {
            "t": 124.0,
            "v": 0.0
          },
          {
            "t": 126.0,
            "v": 0.0
          },
          {
            "t": 128.0,
            "v": 0.0
          },
          {
            "t": 130.0,
            "v": 0.0
          },
          {
            "t": 132.0,
            "v": 0.0
          },
          {
            "t": 134.0,
            "v": 0.0
          },
          {
            "t": 136.0,
            "v": 0.0
          },
          {
            "t": 138.0,
            "v": 1.0
          },
          {
            "t": 140.0,
            "v": 0.0
          },
          {
            "t": 142.0,
            "v": 0.0
          },
          {
            "t": 144.0,
            "v": 0.0
          },
          {
            "t": 146.0,
            "v": 0.0
          },
          {
            "t": 148.0,
            "v": 0.0
          },
          {
            "t": 150.0,
            "v": 0.0
          },
          {
            "t": 152.0,
            "v": 0.0
          },
          {
            "t": 154.0,
            "v": 2.0
          },
          {
            "t": 156.0,
            "v": 3.5
          },
          {
            "t": 158.0,
            "v": 0.0
          },
          {
            "t": 160.0,
            "v": 0.0
          },
          {
            "t": 162.0,
            "v": 0.5
          },
          {
            "t": 164.0,
            "v": 0.0
          },
          {
            "t": 166.0,
            "v": 0.0
          },
          {
            "t": 168.0,
            "v": 0.0
          },
          {
            "t": 170.0,
            "v": 5.0
          },
          {
            "t": 172.0,
            "v": 0.0
          },
          {
            "t": 174.0,
            "v": 0.0
          },
          {
            "t": 176.0,
            "v": 0.5
          },
          {
            "t": 178.0,
            "v": 0.5
          },
          {
            "t": 180.0,
            "v": 0.0
          },
          {
            "t": 182.0,
            "v": 0.0
          },
          {
            "t": 184.0,
            "v": 0.0
          },
          {
            "t": 186.0,
            "v": 0.0
          },
          {
            "t": 188.0,
            "v": 0.0
          },
          {
            "t": 190.0,
            "v": 0.0
          },
          {
            "t": 192.0,
            "v": 0.0
          },
          {
            "t": 194.0,
            "v": 0.0
          },
          {
            "t": 196.0,
            "v": 0.0
          },
          {
            "t": 198.0,
            "v": 0.0
          },
          {
            "t": 200.0,
            "v": 0.5
          },
          {
            "t": 202.0,
            "v": 0.5
          },
          {
            "t": 204.0,
            "v": 0.0
          },
          {
            "t": 206.0,
            "v": 0.0
          },
          {
            "t": 208.0,
            "v": 0.0
          },
          {
            "t": 210.0,
            "v": 0.0
          },
          {
            "t": 212.0,
            "v": 0.0
          },
          {
            "t": 214.0,
            "v": 0.0
          },
          {
            "t": 216.0,
            "v": 0.0
          },
          {
            "t": 218.0,
            "v": 0.0
          },
          {
            "t": 220.0,
            "v": 0.0
          },
          {
            "t": 222.0,
            "v": 0.0
          },
          {
            "t": 224.0,
            "v": 0.0
          },
          {
            "t": 226.0,
            "v": 7.5
          },
          {
            "t": 228.0,
            "v": 0.0
          },
          {
            "t": 230.0,
            "v": 0.0
          },
          {
            "t": 232.0,
            "v": 0.5
          }
        ]
      },
      "ws_echo": {
        "cpu": [
          {
            "t": 82.4,
            "v": 0.07
          },
          {
            "t": 86.1,
            "v": 0.07
          },
          {
            "t": 89.8,
            "v": 0.07
          },
          {
            "t": 93.6,
            "v": 0.13
          },
          {
            "t": 97.3,
            "v": 0.07
          },
          {
            "t": 115.8,
            "v": 0.07
          },
          {
            "t": 123.3,
            "v": 0.07
          },
          {
            "t": 127.1,
            "v": 0.07
          }
        ],
        "rss": [
          {
            "t": 3.5,
            "v": 2.9
          },
          {
            "t": 8.3,
            "v": 2.9
          },
          {
            "t": 13.0,
            "v": 2.9
          },
          {
            "t": 20.6,
            "v": 2.9
          },
          {
            "t": 25.2,
            "v": 2.9
          },
          {
            "t": 28.9,
            "v": 2.9
          },
          {
            "t": 33.9,
            "v": 2.9
          },
          {
            "t": 38.8,
            "v": 2.9
          },
          {
            "t": 43.6,
            "v": 2.9
          },
          {
            "t": 48.4,
            "v": 2.9
          },
          {
            "t": 52.1,
            "v": 2.9
          },
          {
            "t": 58.9,
            "v": 2.9
          },
          {
            "t": 67.9,
            "v": 2.9
          },
          {
            "t": 75.0,
            "v": 2.9
          },
          {
            "t": 78.7,
            "v": 3.2
          },
          {
            "t": 82.4,
            "v": 3.2
          },
          {
            "t": 86.1,
            "v": 3.2
          },
          {
            "t": 89.8,
            "v": 3.2
          },
          {
            "t": 93.6,
            "v": 3.2
          },
          {
            "t": 97.3,
            "v": 3.2
          },
          {
            "t": 101.1,
            "v": 3.2
          },
          {
            "t": 104.8,
            "v": 3.2
          },
          {
            "t": 108.4,
            "v": 3.2
          },
          {
            "t": 112.1,
            "v": 3.2
          },
          {
            "t": 115.8,
            "v": 3.2
          },
          {
            "t": 119.5,
            "v": 3.2
          },
          {
            "t": 123.3,
            "v": 3.2
          },
          {
            "t": 127.1,
            "v": 3.2
          },
          {
            "t": 130.9,
            "v": 3.2
          },
          {
            "t": 134.6,
            "v": 3.2
          },
          {
            "t": 138.3,
            "v": 3.2
          },
          {
            "t": 142.6,
            "v": 3.2
          },
          {
            "t": 147.2,
            "v": 3.2
          },
          {
            "t": 151.9,
            "v": 3.2
          },
          {
            "t": 156.4,
            "v": 3.2
          },
          {
            "t": 160.9,
            "v": 3.2
          },
          {
            "t": 164.5,
            "v": 3.2
          },
          {
            "t": 169.2,
            "v": 3.2
          },
          {
            "t": 173.8,
            "v": 3.2
          },
          {
            "t": 178.4,
            "v": 3.2
          },
          {
            "t": 183.0,
            "v": 3.2
          },
          {
            "t": 186.9,
            "v": 3.2
          },
          {
            "t": 190.5,
            "v": 3.2
          },
          {
            "t": 195.2,
            "v": 3.2
          },
          {
            "t": 199.8,
            "v": 3.2
          },
          {
            "t": 204.4,
            "v": 3.2
          },
          {
            "t": 209.0,
            "v": 3.2
          },
          {
            "t": 212.7,
            "v": 3.2
          },
          {
            "t": 216.7,
            "v": 3.2
          },
          {
            "t": 221.3,
            "v": 3.2
          },
          {
            "t": 226.0,
            "v": 3.2
          },
          {
            "t": 230.6,
            "v": 3.2
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
          },
          {
            "t": 114.0,
            "v": 0.0
          },
          {
            "t": 116.0,
            "v": 0.0
          },
          {
            "t": 118.0,
            "v": 0.0
          },
          {
            "t": 120.0,
            "v": 0.0
          },
          {
            "t": 122.0,
            "v": 0.0
          },
          {
            "t": 124.0,
            "v": 0.0
          },
          {
            "t": 126.0,
            "v": 0.0
          },
          {
            "t": 128.0,
            "v": 0.0
          },
          {
            "t": 130.0,
            "v": 0.0
          },
          {
            "t": 132.0,
            "v": 0.0
          },
          {
            "t": 134.0,
            "v": 0.0
          },
          {
            "t": 136.0,
            "v": 0.0
          },
          {
            "t": 138.0,
            "v": 0.0
          },
          {
            "t": 140.0,
            "v": 0.0
          },
          {
            "t": 142.0,
            "v": 0.0
          },
          {
            "t": 144.0,
            "v": 0.0
          },
          {
            "t": 146.0,
            "v": 0.0
          },
          {
            "t": 148.0,
            "v": 0.0
          },
          {
            "t": 150.0,
            "v": 0.0
          },
          {
            "t": 152.0,
            "v": 0.0
          },
          {
            "t": 154.0,
            "v": 0.0
          },
          {
            "t": 156.0,
            "v": 0.0
          },
          {
            "t": 158.0,
            "v": 0.0
          },
          {
            "t": 160.0,
            "v": 0.0
          },
          {
            "t": 162.0,
            "v": 0.0
          },
          {
            "t": 164.0,
            "v": 0.0
          },
          {
            "t": 166.0,
            "v": 0.0
          },
          {
            "t": 168.0,
            "v": 0.0
          },
          {
            "t": 170.0,
            "v": 0.0
          },
          {
            "t": 172.0,
            "v": 0.0
          },
          {
            "t": 174.0,
            "v": 0.0
          },
          {
            "t": 176.0,
            "v": 0.0
          },
          {
            "t": 178.0,
            "v": 0.0
          },
          {
            "t": 180.0,
            "v": 0.0
          },
          {
            "t": 182.0,
            "v": 0.0
          },
          {
            "t": 184.0,
            "v": 0.0
          },
          {
            "t": 186.0,
            "v": 0.0
          },
          {
            "t": 188.0,
            "v": 0.0
          },
          {
            "t": 190.0,
            "v": 0.0
          },
          {
            "t": 192.0,
            "v": 0.0
          },
          {
            "t": 194.0,
            "v": 0.0
          },
          {
            "t": 196.0,
            "v": 0.0
          },
          {
            "t": 198.0,
            "v": 0.0
          },
          {
            "t": 200.0,
            "v": 0.0
          },
          {
            "t": 202.0,
            "v": 0.0
          },
          {
            "t": 204.0,
            "v": 0.0
          },
          {
            "t": 206.0,
            "v": 0.0
          },
          {
            "t": 208.0,
            "v": 0.0
          },
          {
            "t": 210.0,
            "v": 0.0
          },
          {
            "t": 212.0,
            "v": 0.0
          },
          {
            "t": 214.0,
            "v": 0.0
          },
          {
            "t": 216.0,
            "v": 0.0
          },
          {
            "t": 218.0,
            "v": 0.0
          },
          {
            "t": 220.0,
            "v": 0.0
          },
          {
            "t": 222.0,
            "v": 0.0
          },
          {
            "t": 224.0,
            "v": 0.0
          },
          {
            "t": 226.0,
            "v": 0.0
          },
          {
            "t": 228.0,
            "v": 0.0
          },
          {
            "t": 230.0,
            "v": 0.0
          },
          {
            "t": 232.0,
            "v": 0.0
          }
        ],
        "write_kbs": [
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
            "v": 2.0
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
            "v": 2.0
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
            "v": 2.0
          },
          {
            "t": 112.0,
            "v": 2.0
          },
          {
            "t": 114.0,
            "v": 0.0
          },
          {
            "t": 116.0,
            "v": 0.0
          },
          {
            "t": 118.0,
            "v": 0.0
          },
          {
            "t": 120.0,
            "v": 0.0
          },
          {
            "t": 122.0,
            "v": 0.0
          },
          {
            "t": 124.0,
            "v": 0.0
          },
          {
            "t": 126.0,
            "v": 0.0
          },
          {
            "t": 128.0,
            "v": 0.0
          },
          {
            "t": 130.0,
            "v": 0.0
          },
          {
            "t": 132.0,
            "v": 0.0
          },
          {
            "t": 134.0,
            "v": 0.0
          },
          {
            "t": 136.0,
            "v": 0.0
          },
          {
            "t": 138.0,
            "v": 0.0
          },
          {
            "t": 140.0,
            "v": 0.0
          },
          {
            "t": 142.0,
            "v": 0.0
          },
          {
            "t": 144.0,
            "v": 0.0
          },
          {
            "t": 146.0,
            "v": 0.0
          },
          {
            "t": 148.0,
            "v": 0.0
          },
          {
            "t": 150.0,
            "v": 0.0
          },
          {
            "t": 152.0,
            "v": 0.0
          },
          {
            "t": 154.0,
            "v": 0.0
          },
          {
            "t": 156.0,
            "v": 0.0
          },
          {
            "t": 158.0,
            "v": 0.0
          },
          {
            "t": 160.0,
            "v": 0.0
          },
          {
            "t": 162.0,
            "v": 0.0
          },
          {
            "t": 164.0,
            "v": 0.0
          },
          {
            "t": 166.0,
            "v": 0.0
          },
          {
            "t": 168.0,
            "v": 0.0
          },
          {
            "t": 170.0,
            "v": 0.0
          },
          {
            "t": 172.0,
            "v": 0.0
          },
          {
            "t": 174.0,
            "v": 0.0
          },
          {
            "t": 176.0,
            "v": 0.0
          },
          {
            "t": 178.0,
            "v": 0.0
          },
          {
            "t": 180.0,
            "v": 0.0
          },
          {
            "t": 182.0,
            "v": 0.0
          },
          {
            "t": 184.0,
            "v": 0.0
          },
          {
            "t": 186.0,
            "v": 0.0
          },
          {
            "t": 188.0,
            "v": 0.0
          },
          {
            "t": 190.0,
            "v": 0.0
          },
          {
            "t": 192.0,
            "v": 0.0
          },
          {
            "t": 194.0,
            "v": 0.0
          },
          {
            "t": 196.0,
            "v": 0.0
          },
          {
            "t": 198.0,
            "v": 0.0
          },
          {
            "t": 200.0,
            "v": 0.0
          },
          {
            "t": 202.0,
            "v": 0.0
          },
          {
            "t": 204.0,
            "v": 0.0
          },
          {
            "t": 206.0,
            "v": 0.0
          },
          {
            "t": 208.0,
            "v": 0.0
          },
          {
            "t": 210.0,
            "v": 0.0
          },
          {
            "t": 212.0,
            "v": 0.0
          },
          {
            "t": 214.0,
            "v": 0.0
          },
          {
            "t": 216.0,
            "v": 0.0
          },
          {
            "t": 218.0,
            "v": 0.0
          },
          {
            "t": 220.0,
            "v": 0.0
          },
          {
            "t": 222.0,
            "v": 0.0
          },
          {
            "t": 224.0,
            "v": 0.0
          },
          {
            "t": 226.0,
            "v": 0.0
          },
          {
            "t": 228.0,
            "v": 0.0
          },
          {
            "t": 230.0,
            "v": 0.0
          },
          {
            "t": 232.0,
            "v": 0.0
          }
        ],
        "cswch": [
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
            "v": 7.0
          },
          {
            "t": 78.0,
            "v": 10.0
          },
          {
            "t": 80.0,
            "v": 10.0
          },
          {
            "t": 82.0,
            "v": 10.0
          },
          {
            "t": 84.0,
            "v": 10.0
          },
          {
            "t": 86.0,
            "v": 10.0
          },
          {
            "t": 88.0,
            "v": 10.0
          },
          {
            "t": 90.0,
            "v": 10.0
          },
          {
            "t": 92.0,
            "v": 10.0
          },
          {
            "t": 94.0,
            "v": 10.0
          },
          {
            "t": 96.0,
            "v": 3.0
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
            "v": 1.0
          },
          {
            "t": 110.0,
            "v": 5.0
          },
          {
            "t": 112.0,
            "v": 5.0
          },
          {
            "t": 114.0,
            "v": 5.0
          },
          {
            "t": 116.0,
            "v": 5.0
          },
          {
            "t": 118.0,
            "v": 5.0
          },
          {
            "t": 120.0,
            "v": 5.0
          },
          {
            "t": 122.0,
            "v": 5.0
          },
          {
            "t": 124.0,
            "v": 5.0
          },
          {
            "t": 126.0,
            "v": 5.0
          },
          {
            "t": 128.0,
            "v": 4.0
          },
          {
            "t": 130.0,
            "v": 0.0
          },
          {
            "t": 132.0,
            "v": 0.0
          },
          {
            "t": 134.0,
            "v": 0.0
          },
          {
            "t": 136.0,
            "v": 0.0
          },
          {
            "t": 138.0,
            "v": 0.0
          },
          {
            "t": 140.0,
            "v": 0.0
          },
          {
            "t": 142.0,
            "v": 0.0
          },
          {
            "t": 144.0,
            "v": 0.0
          },
          {
            "t": 146.0,
            "v": 0.0
          },
          {
            "t": 148.0,
            "v": 0.0
          },
          {
            "t": 150.0,
            "v": 0.0
          },
          {
            "t": 152.0,
            "v": 0.0
          },
          {
            "t": 154.0,
            "v": 0.0
          },
          {
            "t": 156.0,
            "v": 0.0
          },
          {
            "t": 158.0,
            "v": 0.0
          },
          {
            "t": 160.0,
            "v": 0.0
          },
          {
            "t": 162.0,
            "v": 0.0
          },
          {
            "t": 164.0,
            "v": 0.0
          },
          {
            "t": 166.0,
            "v": 0.0
          },
          {
            "t": 168.0,
            "v": 0.0
          },
          {
            "t": 170.0,
            "v": 0.0
          },
          {
            "t": 172.0,
            "v": 0.0
          },
          {
            "t": 174.0,
            "v": 0.0
          },
          {
            "t": 176.0,
            "v": 0.0
          },
          {
            "t": 178.0,
            "v": 0.0
          },
          {
            "t": 180.0,
            "v": 0.0
          },
          {
            "t": 182.0,
            "v": 0.0
          },
          {
            "t": 184.0,
            "v": 0.0
          },
          {
            "t": 186.0,
            "v": 0.0
          },
          {
            "t": 188.0,
            "v": 0.0
          },
          {
            "t": 190.0,
            "v": 0.0
          },
          {
            "t": 192.0,
            "v": 0.0
          },
          {
            "t": 194.0,
            "v": 0.0
          },
          {
            "t": 196.0,
            "v": 0.0
          },
          {
            "t": 198.0,
            "v": 0.0
          },
          {
            "t": 200.0,
            "v": 0.0
          },
          {
            "t": 202.0,
            "v": 0.0
          },
          {
            "t": 204.0,
            "v": 0.0
          },
          {
            "t": 206.0,
            "v": 0.0
          },
          {
            "t": 208.0,
            "v": 0.0
          },
          {
            "t": 210.0,
            "v": 0.0
          },
          {
            "t": 212.0,
            "v": 0.0
          },
          {
            "t": 214.0,
            "v": 0.0
          },
          {
            "t": 216.0,
            "v": 0.0
          },
          {
            "t": 218.0,
            "v": 0.0
          },
          {
            "t": 220.0,
            "v": 0.0
          },
          {
            "t": 222.0,
            "v": 0.0
          },
          {
            "t": 224.0,
            "v": 0.0
          },
          {
            "t": 226.0,
            "v": 0.0
          },
          {
            "t": 228.0,
            "v": 0.0
          },
          {
            "t": 230.0,
            "v": 0.0
          },
          {
            "t": 232.0,
            "v": 0.0
          }
        ],
        "nvcswch": [
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
            "v": 5.0
          },
          {
            "t": 80.0,
            "v": 7.0
          },
          {
            "t": 82.0,
            "v": 6.5
          },
          {
            "t": 84.0,
            "v": 8.5
          },
          {
            "t": 86.0,
            "v": 2.5
          },
          {
            "t": 88.0,
            "v": 4.5
          },
          {
            "t": 90.0,
            "v": 1.0
          },
          {
            "t": 92.0,
            "v": 9.5
          },
          {
            "t": 94.0,
            "v": 5.0
          },
          {
            "t": 96.0,
            "v": 0.5
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
            "v": 1.0
          },
          {
            "t": 110.0,
            "v": 2.5
          },
          {
            "t": 112.0,
            "v": 1.0
          },
          {
            "t": 114.0,
            "v": 3.0
          },
          {
            "t": 116.0,
            "v": 0.5
          },
          {
            "t": 118.0,
            "v": 2.5
          },
          {
            "t": 120.0,
            "v": 4.0
          },
          {
            "t": 122.0,
            "v": 3.0
          },
          {
            "t": 124.0,
            "v": 0.0
          },
          {
            "t": 126.0,
            "v": 0.0
          },
          {
            "t": 128.0,
            "v": 0.0
          },
          {
            "t": 130.0,
            "v": 0.0
          },
          {
            "t": 132.0,
            "v": 0.0
          },
          {
            "t": 134.0,
            "v": 0.0
          },
          {
            "t": 136.0,
            "v": 0.0
          },
          {
            "t": 138.0,
            "v": 0.0
          },
          {
            "t": 140.0,
            "v": 0.0
          },
          {
            "t": 142.0,
            "v": 0.0
          },
          {
            "t": 144.0,
            "v": 0.0
          },
          {
            "t": 146.0,
            "v": 0.0
          },
          {
            "t": 148.0,
            "v": 0.0
          },
          {
            "t": 150.0,
            "v": 0.0
          },
          {
            "t": 152.0,
            "v": 0.0
          },
          {
            "t": 154.0,
            "v": 0.0
          },
          {
            "t": 156.0,
            "v": 0.0
          },
          {
            "t": 158.0,
            "v": 0.0
          },
          {
            "t": 160.0,
            "v": 0.0
          },
          {
            "t": 162.0,
            "v": 0.0
          },
          {
            "t": 164.0,
            "v": 0.0
          },
          {
            "t": 166.0,
            "v": 0.0
          },
          {
            "t": 168.0,
            "v": 0.0
          },
          {
            "t": 170.0,
            "v": 0.0
          },
          {
            "t": 172.0,
            "v": 0.0
          },
          {
            "t": 174.0,
            "v": 0.0
          },
          {
            "t": 176.0,
            "v": 0.0
          },
          {
            "t": 178.0,
            "v": 0.0
          },
          {
            "t": 180.0,
            "v": 0.0
          },
          {
            "t": 182.0,
            "v": 0.0
          },
          {
            "t": 184.0,
            "v": 0.0
          },
          {
            "t": 186.0,
            "v": 0.0
          },
          {
            "t": 188.0,
            "v": 0.0
          },
          {
            "t": 190.0,
            "v": 0.0
          },
          {
            "t": 192.0,
            "v": 0.0
          },
          {
            "t": 194.0,
            "v": 0.0
          },
          {
            "t": 196.0,
            "v": 0.0
          },
          {
            "t": 198.0,
            "v": 0.0
          },
          {
            "t": 200.0,
            "v": 0.0
          },
          {
            "t": 202.0,
            "v": 0.0
          },
          {
            "t": 204.0,
            "v": 0.0
          },
          {
            "t": 206.0,
            "v": 0.0
          },
          {
            "t": 208.0,
            "v": 0.0
          },
          {
            "t": 210.0,
            "v": 0.0
          },
          {
            "t": 212.0,
            "v": 0.0
          },
          {
            "t": 214.0,
            "v": 0.0
          },
          {
            "t": 216.0,
            "v": 0.0
          },
          {
            "t": 218.0,
            "v": 0.0
          },
          {
            "t": 220.0,
            "v": 0.0
          },
          {
            "t": 222.0,
            "v": 0.0
          },
          {
            "t": 224.0,
            "v": 0.0
          },
          {
            "t": 226.0,
            "v": 0.0
          },
          {
            "t": 228.0,
            "v": 0.0
          },
          {
            "t": 230.0,
            "v": 0.0
          },
          {
            "t": 232.0,
            "v": 0.0
          }
        ]
      },
      "grpc_echo": {
        "cpu": [
          {
            "t": 82.4,
            "v": 0.14
          },
          {
            "t": 86.1,
            "v": 0.13
          },
          {
            "t": 89.8,
            "v": 0.2
          },
          {
            "t": 93.6,
            "v": 0.2
          },
          {
            "t": 97.3,
            "v": 0.2
          },
          {
            "t": 101.1,
            "v": 0.07
          },
          {
            "t": 115.8,
            "v": 0.34
          },
          {
            "t": 119.5,
            "v": 0.53
          },
          {
            "t": 123.3,
            "v": 0.53
          },
          {
            "t": 127.1,
            "v": 0.46
          },
          {
            "t": 130.9,
            "v": 0.53
          },
          {
            "t": 134.6,
            "v": 0.2
          }
        ],
        "rss": [
          {
            "t": 3.5,
            "v": 3.3
          },
          {
            "t": 8.3,
            "v": 3.3
          },
          {
            "t": 13.0,
            "v": 3.3
          },
          {
            "t": 20.6,
            "v": 3.3
          },
          {
            "t": 25.2,
            "v": 3.3
          },
          {
            "t": 28.9,
            "v": 3.3
          },
          {
            "t": 33.9,
            "v": 3.3
          },
          {
            "t": 38.8,
            "v": 3.3
          },
          {
            "t": 43.6,
            "v": 3.3
          },
          {
            "t": 48.4,
            "v": 3.3
          },
          {
            "t": 52.1,
            "v": 3.3
          },
          {
            "t": 58.9,
            "v": 3.3
          },
          {
            "t": 67.9,
            "v": 3.3
          },
          {
            "t": 75.0,
            "v": 3.3
          },
          {
            "t": 78.7,
            "v": 3.9
          },
          {
            "t": 82.4,
            "v": 4.0
          },
          {
            "t": 86.1,
            "v": 4.0
          },
          {
            "t": 89.8,
            "v": 4.0
          },
          {
            "t": 93.6,
            "v": 4.0
          },
          {
            "t": 97.3,
            "v": 4.0
          },
          {
            "t": 101.1,
            "v": 4.0
          },
          {
            "t": 104.8,
            "v": 4.0
          },
          {
            "t": 108.4,
            "v": 4.0
          },
          {
            "t": 112.1,
            "v": 4.1
          },
          {
            "t": 115.8,
            "v": 4.2
          },
          {
            "t": 119.5,
            "v": 4.2
          },
          {
            "t": 123.3,
            "v": 4.2
          },
          {
            "t": 127.1,
            "v": 4.2
          },
          {
            "t": 130.9,
            "v": 4.2
          },
          {
            "t": 134.6,
            "v": 4.2
          },
          {
            "t": 138.3,
            "v": 4.2
          },
          {
            "t": 142.6,
            "v": 4.2
          },
          {
            "t": 147.2,
            "v": 4.2
          },
          {
            "t": 151.9,
            "v": 4.2
          },
          {
            "t": 156.4,
            "v": 4.2
          },
          {
            "t": 160.9,
            "v": 4.2
          },
          {
            "t": 164.5,
            "v": 4.2
          },
          {
            "t": 169.2,
            "v": 4.2
          },
          {
            "t": 173.8,
            "v": 4.2
          },
          {
            "t": 178.4,
            "v": 4.2
          },
          {
            "t": 183.0,
            "v": 4.2
          },
          {
            "t": 186.9,
            "v": 4.2
          },
          {
            "t": 190.5,
            "v": 4.2
          },
          {
            "t": 195.2,
            "v": 4.2
          },
          {
            "t": 199.8,
            "v": 4.2
          },
          {
            "t": 204.4,
            "v": 4.2
          },
          {
            "t": 209.0,
            "v": 4.2
          },
          {
            "t": 212.7,
            "v": 4.2
          },
          {
            "t": 216.7,
            "v": 4.2
          },
          {
            "t": 221.3,
            "v": 4.2
          },
          {
            "t": 226.0,
            "v": 4.2
          },
          {
            "t": 230.6,
            "v": 4.2
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
          },
          {
            "t": 114.0,
            "v": 0.0
          },
          {
            "t": 116.0,
            "v": 0.0
          },
          {
            "t": 118.0,
            "v": 0.0
          },
          {
            "t": 120.0,
            "v": 0.0
          },
          {
            "t": 122.0,
            "v": 0.0
          },
          {
            "t": 124.0,
            "v": 0.0
          },
          {
            "t": 126.0,
            "v": 0.0
          },
          {
            "t": 128.0,
            "v": 0.0
          },
          {
            "t": 130.0,
            "v": 0.0
          },
          {
            "t": 132.0,
            "v": 0.0
          },
          {
            "t": 134.0,
            "v": 0.0
          },
          {
            "t": 136.0,
            "v": 0.0
          },
          {
            "t": 138.0,
            "v": 0.0
          },
          {
            "t": 140.0,
            "v": 0.0
          },
          {
            "t": 142.0,
            "v": 0.0
          },
          {
            "t": 144.0,
            "v": 0.0
          },
          {
            "t": 146.0,
            "v": 0.0
          },
          {
            "t": 148.0,
            "v": 0.0
          },
          {
            "t": 150.0,
            "v": 0.0
          },
          {
            "t": 152.0,
            "v": 0.0
          },
          {
            "t": 154.0,
            "v": 0.0
          },
          {
            "t": 156.0,
            "v": 0.0
          },
          {
            "t": 158.0,
            "v": 0.0
          },
          {
            "t": 160.0,
            "v": 0.0
          },
          {
            "t": 162.0,
            "v": 0.0
          },
          {
            "t": 164.0,
            "v": 0.0
          },
          {
            "t": 166.0,
            "v": 0.0
          },
          {
            "t": 168.0,
            "v": 0.0
          },
          {
            "t": 170.0,
            "v": 0.0
          },
          {
            "t": 172.0,
            "v": 0.0
          },
          {
            "t": 174.0,
            "v": 0.0
          },
          {
            "t": 176.0,
            "v": 0.0
          },
          {
            "t": 178.0,
            "v": 0.0
          },
          {
            "t": 180.0,
            "v": 0.0
          },
          {
            "t": 182.0,
            "v": 0.0
          },
          {
            "t": 184.0,
            "v": 0.0
          },
          {
            "t": 186.0,
            "v": 0.0
          },
          {
            "t": 188.0,
            "v": 0.0
          },
          {
            "t": 190.0,
            "v": 0.0
          },
          {
            "t": 192.0,
            "v": 0.0
          },
          {
            "t": 194.0,
            "v": 0.0
          },
          {
            "t": 196.0,
            "v": 0.0
          },
          {
            "t": 198.0,
            "v": 0.0
          },
          {
            "t": 200.0,
            "v": 0.0
          },
          {
            "t": 202.0,
            "v": 0.0
          },
          {
            "t": 204.0,
            "v": 0.0
          },
          {
            "t": 206.0,
            "v": 0.0
          },
          {
            "t": 208.0,
            "v": 0.0
          },
          {
            "t": 210.0,
            "v": 0.0
          },
          {
            "t": 212.0,
            "v": 0.0
          },
          {
            "t": 214.0,
            "v": 0.0
          },
          {
            "t": 216.0,
            "v": 0.0
          },
          {
            "t": 218.0,
            "v": 0.0
          },
          {
            "t": 220.0,
            "v": 0.0
          },
          {
            "t": 222.0,
            "v": 0.0
          },
          {
            "t": 224.0,
            "v": 0.0
          },
          {
            "t": 226.0,
            "v": 0.0
          },
          {
            "t": 228.0,
            "v": 0.0
          },
          {
            "t": 230.0,
            "v": 0.0
          },
          {
            "t": 232.0,
            "v": 0.0
          }
        ],
        "write_kbs": [
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
          },
          {
            "t": 114.0,
            "v": 0.0
          },
          {
            "t": 116.0,
            "v": 0.0
          },
          {
            "t": 118.0,
            "v": 0.0
          },
          {
            "t": 120.0,
            "v": 0.0
          },
          {
            "t": 122.0,
            "v": 0.0
          },
          {
            "t": 124.0,
            "v": 0.0
          },
          {
            "t": 126.0,
            "v": 0.0
          },
          {
            "t": 128.0,
            "v": 0.0
          },
          {
            "t": 130.0,
            "v": 0.0
          },
          {
            "t": 132.0,
            "v": 0.0
          },
          {
            "t": 134.0,
            "v": 0.0
          },
          {
            "t": 136.0,
            "v": 0.0
          },
          {
            "t": 138.0,
            "v": 0.0
          },
          {
            "t": 140.0,
            "v": 0.0
          },
          {
            "t": 142.0,
            "v": 0.0
          },
          {
            "t": 144.0,
            "v": 0.0
          },
          {
            "t": 146.0,
            "v": 0.0
          },
          {
            "t": 148.0,
            "v": 0.0
          },
          {
            "t": 150.0,
            "v": 0.0
          },
          {
            "t": 152.0,
            "v": 0.0
          },
          {
            "t": 154.0,
            "v": 0.0
          },
          {
            "t": 156.0,
            "v": 0.0
          },
          {
            "t": 158.0,
            "v": 0.0
          },
          {
            "t": 160.0,
            "v": 0.0
          },
          {
            "t": 162.0,
            "v": 0.0
          },
          {
            "t": 164.0,
            "v": 0.0
          },
          {
            "t": 166.0,
            "v": 0.0
          },
          {
            "t": 168.0,
            "v": 0.0
          },
          {
            "t": 170.0,
            "v": 0.0
          },
          {
            "t": 172.0,
            "v": 0.0
          },
          {
            "t": 174.0,
            "v": 0.0
          },
          {
            "t": 176.0,
            "v": 0.0
          },
          {
            "t": 178.0,
            "v": 0.0
          },
          {
            "t": 180.0,
            "v": 0.0
          },
          {
            "t": 182.0,
            "v": 0.0
          },
          {
            "t": 184.0,
            "v": 0.0
          },
          {
            "t": 186.0,
            "v": 0.0
          },
          {
            "t": 188.0,
            "v": 0.0
          },
          {
            "t": 190.0,
            "v": 0.0
          },
          {
            "t": 192.0,
            "v": 0.0
          },
          {
            "t": 194.0,
            "v": 0.0
          },
          {
            "t": 196.0,
            "v": 0.0
          },
          {
            "t": 198.0,
            "v": 0.0
          },
          {
            "t": 200.0,
            "v": 0.0
          },
          {
            "t": 202.0,
            "v": 0.0
          },
          {
            "t": 204.0,
            "v": 0.0
          },
          {
            "t": 206.0,
            "v": 0.0
          },
          {
            "t": 208.0,
            "v": 0.0
          },
          {
            "t": 210.0,
            "v": 0.0
          },
          {
            "t": 212.0,
            "v": 0.0
          },
          {
            "t": 214.0,
            "v": 0.0
          },
          {
            "t": 216.0,
            "v": 0.0
          },
          {
            "t": 218.0,
            "v": 0.0
          },
          {
            "t": 220.0,
            "v": 0.0
          },
          {
            "t": 222.0,
            "v": 0.0
          },
          {
            "t": 224.0,
            "v": 0.0
          },
          {
            "t": 226.0,
            "v": 0.0
          },
          {
            "t": 228.0,
            "v": 0.0
          },
          {
            "t": 230.0,
            "v": 0.0
          },
          {
            "t": 232.0,
            "v": 0.0
          }
        ],
        "cswch": [
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
            "v": 0.5
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
          },
          {
            "t": 116.0,
            "v": 0.0
          },
          {
            "t": 118.0,
            "v": 0.0
          },
          {
            "t": 120.0,
            "v": 0.0
          },
          {
            "t": 122.0,
            "v": 0.0
          },
          {
            "t": 124.0,
            "v": 0.0
          },
          {
            "t": 126.0,
            "v": 0.0
          },
          {
            "t": 128.0,
            "v": 0.0
          },
          {
            "t": 130.0,
            "v": 0.0
          },
          {
            "t": 132.0,
            "v": 0.0
          },
          {
            "t": 134.0,
            "v": 0.0
          },
          {
            "t": 136.0,
            "v": 0.0
          },
          {
            "t": 138.0,
            "v": 0.0
          },
          {
            "t": 140.0,
            "v": 0.0
          },
          {
            "t": 142.0,
            "v": 0.0
          },
          {
            "t": 144.0,
            "v": 0.0
          },
          {
            "t": 146.0,
            "v": 0.0
          },
          {
            "t": 148.0,
            "v": 0.0
          },
          {
            "t": 150.0,
            "v": 0.0
          },
          {
            "t": 152.0,
            "v": 0.0
          },
          {
            "t": 154.0,
            "v": 0.0
          },
          {
            "t": 156.0,
            "v": 0.0
          },
          {
            "t": 158.0,
            "v": 0.0
          },
          {
            "t": 160.0,
            "v": 0.0
          },
          {
            "t": 162.0,
            "v": 0.0
          },
          {
            "t": 164.0,
            "v": 0.0
          },
          {
            "t": 166.0,
            "v": 0.0
          },
          {
            "t": 168.0,
            "v": 0.0
          },
          {
            "t": 170.0,
            "v": 0.0
          },
          {
            "t": 172.0,
            "v": 0.0
          },
          {
            "t": 174.0,
            "v": 0.0
          },
          {
            "t": 176.0,
            "v": 0.0
          },
          {
            "t": 178.0,
            "v": 0.0
          },
          {
            "t": 180.0,
            "v": 0.0
          },
          {
            "t": 182.0,
            "v": 0.0
          },
          {
            "t": 184.0,
            "v": 0.0
          },
          {
            "t": 186.0,
            "v": 0.0
          },
          {
            "t": 188.0,
            "v": 0.0
          },
          {
            "t": 190.0,
            "v": 0.0
          },
          {
            "t": 192.0,
            "v": 0.0
          },
          {
            "t": 194.0,
            "v": 0.0
          },
          {
            "t": 196.0,
            "v": 0.0
          },
          {
            "t": 198.0,
            "v": 0.0
          },
          {
            "t": 200.0,
            "v": 0.0
          },
          {
            "t": 202.0,
            "v": 0.0
          },
          {
            "t": 204.0,
            "v": 0.0
          },
          {
            "t": 206.0,
            "v": 0.0
          },
          {
            "t": 208.0,
            "v": 0.0
          },
          {
            "t": 210.0,
            "v": 0.0
          },
          {
            "t": 212.0,
            "v": 0.0
          },
          {
            "t": 214.0,
            "v": 0.0
          },
          {
            "t": 216.0,
            "v": 0.0
          },
          {
            "t": 218.0,
            "v": 0.0
          },
          {
            "t": 220.0,
            "v": 0.0
          },
          {
            "t": 222.0,
            "v": 0.0
          },
          {
            "t": 224.0,
            "v": 0.0
          },
          {
            "t": 226.0,
            "v": 0.0
          },
          {
            "t": 228.0,
            "v": 0.0
          },
          {
            "t": 230.0,
            "v": 0.0
          },
          {
            "t": 232.0,
            "v": 0.0
          }
        ],
        "nvcswch": [
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
          },
          {
            "t": 114.0,
            "v": 0.0
          },
          {
            "t": 116.0,
            "v": 0.0
          },
          {
            "t": 118.0,
            "v": 0.0
          },
          {
            "t": 120.0,
            "v": 0.0
          },
          {
            "t": 122.0,
            "v": 0.0
          },
          {
            "t": 124.0,
            "v": 0.0
          },
          {
            "t": 126.0,
            "v": 0.0
          },
          {
            "t": 128.0,
            "v": 0.0
          },
          {
            "t": 130.0,
            "v": 0.0
          },
          {
            "t": 132.0,
            "v": 0.0
          },
          {
            "t": 134.0,
            "v": 0.0
          },
          {
            "t": 136.0,
            "v": 0.0
          },
          {
            "t": 138.0,
            "v": 0.0
          },
          {
            "t": 140.0,
            "v": 0.0
          },
          {
            "t": 142.0,
            "v": 0.0
          },
          {
            "t": 144.0,
            "v": 0.0
          },
          {
            "t": 146.0,
            "v": 0.0
          },
          {
            "t": 148.0,
            "v": 0.0
          },
          {
            "t": 150.0,
            "v": 0.0
          },
          {
            "t": 152.0,
            "v": 0.0
          },
          {
            "t": 154.0,
            "v": 0.0
          },
          {
            "t": 156.0,
            "v": 0.0
          },
          {
            "t": 158.0,
            "v": 0.0
          },
          {
            "t": 160.0,
            "v": 0.0
          },
          {
            "t": 162.0,
            "v": 0.0
          },
          {
            "t": 164.0,
            "v": 0.0
          },
          {
            "t": 166.0,
            "v": 0.0
          },
          {
            "t": 168.0,
            "v": 0.0
          },
          {
            "t": 170.0,
            "v": 0.0
          },
          {
            "t": 172.0,
            "v": 0.0
          },
          {
            "t": 174.0,
            "v": 0.0
          },
          {
            "t": 176.0,
            "v": 0.0
          },
          {
            "t": 178.0,
            "v": 0.0
          },
          {
            "t": 180.0,
            "v": 0.0
          },
          {
            "t": 182.0,
            "v": 0.0
          },
          {
            "t": 184.0,
            "v": 0.0
          },
          {
            "t": 186.0,
            "v": 0.0
          },
          {
            "t": 188.0,
            "v": 0.0
          },
          {
            "t": 190.0,
            "v": 0.0
          },
          {
            "t": 192.0,
            "v": 0.0
          },
          {
            "t": 194.0,
            "v": 0.0
          },
          {
            "t": 196.0,
            "v": 0.0
          },
          {
            "t": 198.0,
            "v": 0.0
          },
          {
            "t": 200.0,
            "v": 0.0
          },
          {
            "t": 202.0,
            "v": 0.0
          },
          {
            "t": 204.0,
            "v": 0.0
          },
          {
            "t": 206.0,
            "v": 0.0
          },
          {
            "t": 208.0,
            "v": 0.0
          },
          {
            "t": 210.0,
            "v": 0.0
          },
          {
            "t": 212.0,
            "v": 0.0
          },
          {
            "t": 214.0,
            "v": 0.0
          },
          {
            "t": 216.0,
            "v": 0.0
          },
          {
            "t": 218.0,
            "v": 0.0
          },
          {
            "t": 220.0,
            "v": 0.0
          },
          {
            "t": 222.0,
            "v": 0.0
          },
          {
            "t": 224.0,
            "v": 0.0
          },
          {
            "t": 226.0,
            "v": 0.0
          },
          {
            "t": 228.0,
            "v": 0.0
          },
          {
            "t": 230.0,
            "v": 0.0
          },
          {
            "t": 232.0,
            "v": 0.0
          }
        ]
      },
      "k6": {
        "cpu": [
          {
            "t": 8.3,
            "v": 23.0
          },
          {
            "t": 13.0,
            "v": 27.52
          },
          {
            "t": 20.6,
            "v": 17.93
          },
          {
            "t": 25.2,
            "v": 0.11
          },
          {
            "t": 28.9,
            "v": 20.63
          },
          {
            "t": 33.9,
            "v": 22.0
          },
          {
            "t": 38.8,
            "v": 23.34
          },
          {
            "t": 43.6,
            "v": 23.23
          },
          {
            "t": 48.4,
            "v": 10.53
          },
          {
            "t": 52.1,
            "v": 20.52
          },
          {
            "t": 58.9,
            "v": 46.77
          },
          {
            "t": 67.9,
            "v": 34.02
          },
          {
            "t": 78.7,
            "v": 1.17
          },
          {
            "t": 82.4,
            "v": 3.99
          },
          {
            "t": 86.1,
            "v": 3.09
          },
          {
            "t": 89.8,
            "v": 3.21
          },
          {
            "t": 93.6,
            "v": 3.13
          },
          {
            "t": 97.3,
            "v": 3.33
          },
          {
            "t": 101.1,
            "v": 2.4
          },
          {
            "t": 104.8,
            "v": 2.43
          },
          {
            "t": 108.4,
            "v": 0.21
          },
          {
            "t": 112.1,
            "v": 1.64
          },
          {
            "t": 115.8,
            "v": 2.77
          },
          {
            "t": 119.5,
            "v": 3.45
          },
          {
            "t": 123.3,
            "v": 4.96
          },
          {
            "t": 127.1,
            "v": 3.51
          },
          {
            "t": 130.9,
            "v": 2.76
          },
          {
            "t": 134.6,
            "v": 0.67
          },
          {
            "t": 138.3,
            "v": 5.01
          },
          {
            "t": 142.6,
            "v": 25.81
          },
          {
            "t": 147.2,
            "v": 23.35
          },
          {
            "t": 151.9,
            "v": 23.07
          },
          {
            "t": 156.4,
            "v": 24.26
          },
          {
            "t": 160.9,
            "v": 7.08
          },
          {
            "t": 164.5,
            "v": 14.27
          },
          {
            "t": 169.2,
            "v": 24.57
          },
          {
            "t": 173.8,
            "v": 24.49
          },
          {
            "t": 178.4,
            "v": 22.91
          },
          {
            "t": 183.0,
            "v": 22.71
          },
          {
            "t": 186.9,
            "v": 0.13
          },
          {
            "t": 190.5,
            "v": 20.65
          },
          {
            "t": 195.2,
            "v": 26.02
          },
          {
            "t": 199.8,
            "v": 26.07
          },
          {
            "t": 204.4,
            "v": 25.85
          },
          {
            "t": 209.0,
            "v": 19.02
          },
          {
            "t": 212.7,
            "v": 0.07
          },
          {
            "t": 216.7,
            "v": 28.0
          },
          {
            "t": 221.3,
            "v": 26.39
          },
          {
            "t": 226.0,
            "v": 25.76
          },
          {
            "t": 230.6,
            "v": 26.67
          }
        ],
        "rss": [
          {
            "t": 3.5,
            "v": 122.3
          },
          {
            "t": 8.3,
            "v": 137.3
          },
          {
            "t": 13.0,
            "v": 198.8
          },
          {
            "t": 20.6,
            "v": 227.1
          },
          {
            "t": 25.2,
            "v": 227.1
          },
          {
            "t": 28.9,
            "v": 225.3
          },
          {
            "t": 33.9,
            "v": 236.3
          },
          {
            "t": 38.8,
            "v": 244.5
          },
          {
            "t": 43.6,
            "v": 250.3
          },
          {
            "t": 48.4,
            "v": 244.0
          },
          {
            "t": 52.1,
            "v": 338.3
          },
          {
            "t": 58.9,
            "v": 515.8
          },
          {
            "t": 67.9,
            "v": 562.2
          },
          {
            "t": 75.0,
            "v": 352.8
          },
          {
            "t": 78.7,
            "v": 362.3
          },
          {
            "t": 82.4,
            "v": 374.9
          },
          {
            "t": 86.1,
            "v": 375.1
          },
          {
            "t": 89.8,
            "v": 376.5
          },
          {
            "t": 93.6,
            "v": 382.3
          },
          {
            "t": 97.3,
            "v": 398.1
          },
          {
            "t": 101.1,
            "v": 415.8
          },
          {
            "t": 104.8,
            "v": 421.3
          },
          {
            "t": 108.4,
            "v": 421.5
          },
          {
            "t": 112.1,
            "v": 421.6
          },
          {
            "t": 115.8,
            "v": 422.2
          },
          {
            "t": 119.5,
            "v": 423.3
          },
          {
            "t": 123.3,
            "v": 433.0
          },
          {
            "t": 127.1,
            "v": 433.0
          },
          {
            "t": 130.9,
            "v": 433.1
          },
          {
            "t": 134.6,
            "v": 433.1
          },
          {
            "t": 138.3,
            "v": 447.1
          },
          {
            "t": 142.6,
            "v": 507.9
          },
          {
            "t": 147.2,
            "v": 511.4
          },
          {
            "t": 151.9,
            "v": 513.2
          },
          {
            "t": 156.4,
            "v": 513.3
          },
          {
            "t": 160.9,
            "v": 513.4
          },
          {
            "t": 164.5,
            "v": 519.0
          },
          {
            "t": 169.2,
            "v": 525.2
          },
          {
            "t": 173.8,
            "v": 527.8
          },
          {
            "t": 178.4,
            "v": 527.9
          },
          {
            "t": 183.0,
            "v": 535.2
          },
          {
            "t": 186.9,
            "v": 535.2
          },
          {
            "t": 190.5,
            "v": 535.3
          },
          {
            "t": 195.2,
            "v": 550.0
          },
          {
            "t": 199.8,
            "v": 555.8
          },
          {
            "t": 204.4,
            "v": 555.9
          },
          {
            "t": 209.0,
            "v": 556.0
          },
          {
            "t": 212.7,
            "v": 556.0
          },
          {
            "t": 216.7,
            "v": 556.0
          },
          {
            "t": 221.3,
            "v": 561.9
          },
          {
            "t": 226.0,
            "v": 567.5
          },
          {
            "t": 230.6,
            "v": 574.2
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
          },
          {
            "t": 116.0,
            "v": 0.0
          },
          {
            "t": 118.0,
            "v": 0.0
          },
          {
            "t": 120.0,
            "v": 0.0
          },
          {
            "t": 122.0,
            "v": 0.0
          },
          {
            "t": 124.0,
            "v": 0.0
          },
          {
            "t": 126.0,
            "v": 0.0
          },
          {
            "t": 128.0,
            "v": 0.0
          },
          {
            "t": 130.0,
            "v": 0.0
          },
          {
            "t": 132.0,
            "v": 0.0
          },
          {
            "t": 134.0,
            "v": 0.0
          },
          {
            "t": 136.0,
            "v": 0.0
          },
          {
            "t": 138.0,
            "v": 0.0
          },
          {
            "t": 140.0,
            "v": 0.0
          },
          {
            "t": 142.0,
            "v": 0.0
          },
          {
            "t": 144.0,
            "v": 0.0
          },
          {
            "t": 146.0,
            "v": 0.0
          },
          {
            "t": 148.0,
            "v": 0.0
          },
          {
            "t": 150.0,
            "v": 0.0
          },
          {
            "t": 152.0,
            "v": 0.0
          },
          {
            "t": 154.0,
            "v": 0.0
          },
          {
            "t": 156.0,
            "v": 0.0
          },
          {
            "t": 158.0,
            "v": 0.0
          },
          {
            "t": 160.0,
            "v": 0.0
          },
          {
            "t": 162.0,
            "v": 0.0
          },
          {
            "t": 164.0,
            "v": 0.0
          },
          {
            "t": 166.0,
            "v": 0.0
          },
          {
            "t": 168.0,
            "v": 0.0
          },
          {
            "t": 170.0,
            "v": 0.0
          },
          {
            "t": 172.0,
            "v": 0.0
          },
          {
            "t": 174.0,
            "v": 0.0
          },
          {
            "t": 176.0,
            "v": 0.0
          },
          {
            "t": 178.0,
            "v": 0.0
          },
          {
            "t": 180.0,
            "v": 0.0
          },
          {
            "t": 182.0,
            "v": 0.0
          },
          {
            "t": 184.0,
            "v": 0.0
          },
          {
            "t": 186.0,
            "v": 0.0
          },
          {
            "t": 188.0,
            "v": 0.0
          },
          {
            "t": 190.0,
            "v": 0.0
          },
          {
            "t": 192.0,
            "v": 0.0
          },
          {
            "t": 194.0,
            "v": 0.0
          },
          {
            "t": 196.0,
            "v": 0.0
          },
          {
            "t": 198.0,
            "v": 0.0
          },
          {
            "t": 200.0,
            "v": 0.0
          },
          {
            "t": 202.0,
            "v": 0.0
          },
          {
            "t": 204.0,
            "v": 0.0
          },
          {
            "t": 206.0,
            "v": 0.0
          },
          {
            "t": 208.0,
            "v": 0.0
          },
          {
            "t": 210.0,
            "v": 0.0
          },
          {
            "t": 212.0,
            "v": 0.0
          },
          {
            "t": 214.0,
            "v": 0.0
          },
          {
            "t": 216.0,
            "v": 0.0
          },
          {
            "t": 218.0,
            "v": 0.0
          },
          {
            "t": 220.0,
            "v": 0.0
          },
          {
            "t": 222.0,
            "v": 0.0
          },
          {
            "t": 224.0,
            "v": 0.0
          },
          {
            "t": 226.0,
            "v": 0.0
          },
          {
            "t": 228.0,
            "v": 0.0
          },
          {
            "t": 230.0,
            "v": 0.0
          }
        ],
        "write_kbs": [
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
          },
          {
            "t": 116.0,
            "v": 0.0
          },
          {
            "t": 118.0,
            "v": 0.0
          },
          {
            "t": 120.0,
            "v": 0.0
          },
          {
            "t": 122.0,
            "v": 0.0
          },
          {
            "t": 124.0,
            "v": 0.0
          },
          {
            "t": 126.0,
            "v": 0.0
          },
          {
            "t": 128.0,
            "v": 0.0
          },
          {
            "t": 130.0,
            "v": 0.0
          },
          {
            "t": 132.0,
            "v": 0.0
          },
          {
            "t": 134.0,
            "v": 0.0
          },
          {
            "t": 136.0,
            "v": 0.0
          },
          {
            "t": 138.0,
            "v": 0.0
          },
          {
            "t": 140.0,
            "v": 0.0
          },
          {
            "t": 142.0,
            "v": 0.0
          },
          {
            "t": 144.0,
            "v": 0.0
          },
          {
            "t": 146.0,
            "v": 0.0
          },
          {
            "t": 148.0,
            "v": 0.0
          },
          {
            "t": 150.0,
            "v": 0.0
          },
          {
            "t": 152.0,
            "v": 0.0
          },
          {
            "t": 154.0,
            "v": 0.0
          },
          {
            "t": 156.0,
            "v": 0.0
          },
          {
            "t": 158.0,
            "v": 0.0
          },
          {
            "t": 160.0,
            "v": 0.0
          },
          {
            "t": 162.0,
            "v": 0.0
          },
          {
            "t": 164.0,
            "v": 0.0
          },
          {
            "t": 166.0,
            "v": 0.0
          },
          {
            "t": 168.0,
            "v": 0.0
          },
          {
            "t": 170.0,
            "v": 0.0
          },
          {
            "t": 172.0,
            "v": 0.0
          },
          {
            "t": 174.0,
            "v": 0.0
          },
          {
            "t": 176.0,
            "v": 0.0
          },
          {
            "t": 178.0,
            "v": 0.0
          },
          {
            "t": 180.0,
            "v": 0.0
          },
          {
            "t": 182.0,
            "v": 0.0
          },
          {
            "t": 184.0,
            "v": 0.0
          },
          {
            "t": 186.0,
            "v": 0.0
          },
          {
            "t": 188.0,
            "v": 0.0
          },
          {
            "t": 190.0,
            "v": 0.0
          },
          {
            "t": 192.0,
            "v": 0.0
          },
          {
            "t": 194.0,
            "v": 0.0
          },
          {
            "t": 196.0,
            "v": 0.0
          },
          {
            "t": 198.0,
            "v": 0.0
          },
          {
            "t": 200.0,
            "v": 0.0
          },
          {
            "t": 202.0,
            "v": 0.0
          },
          {
            "t": 204.0,
            "v": 0.0
          },
          {
            "t": 206.0,
            "v": 0.0
          },
          {
            "t": 208.0,
            "v": 0.0
          },
          {
            "t": 210.0,
            "v": 0.0
          },
          {
            "t": 212.0,
            "v": 0.0
          },
          {
            "t": 214.0,
            "v": 0.0
          },
          {
            "t": 216.0,
            "v": 0.0
          },
          {
            "t": 218.0,
            "v": 0.0
          },
          {
            "t": 220.0,
            "v": 0.0
          },
          {
            "t": 222.0,
            "v": 0.0
          },
          {
            "t": 224.0,
            "v": 0.0
          },
          {
            "t": 226.0,
            "v": 0.0
          },
          {
            "t": 228.0,
            "v": 0.0
          },
          {
            "t": 230.0,
            "v": 0.0
          }
        ],
        "cswch": [
          {
            "t": 2.0,
            "v": 229.5
          },
          {
            "t": 4.0,
            "v": 1209.0
          },
          {
            "t": 6.0,
            "v": 1800.0
          },
          {
            "t": 8.0,
            "v": 1181.5
          },
          {
            "t": 10.0,
            "v": 1328.0
          },
          {
            "t": 12.0,
            "v": 110.5
          },
          {
            "t": 14.0,
            "v": 132.5
          },
          {
            "t": 16.0,
            "v": 286.1
          },
          {
            "t": 18.0,
            "v": 762.5
          },
          {
            "t": 20.0,
            "v": 1311.0
          },
          {
            "t": 22.0,
            "v": 139.5
          },
          {
            "t": 24.0,
            "v": 32.5
          },
          {
            "t": 26.0,
            "v": 551.0
          },
          {
            "t": 28.0,
            "v": 1578.0
          },
          {
            "t": 30.0,
            "v": 277.5
          },
          {
            "t": 32.0,
            "v": 636.5
          },
          {
            "t": 34.0,
            "v": 1583.0
          },
          {
            "t": 36.0,
            "v": 1731.5
          },
          {
            "t": 38.0,
            "v": 1672.5
          },
          {
            "t": 40.0,
            "v": 1553.5
          },
          {
            "t": 42.0,
            "v": 2080.0
          },
          {
            "t": 44.0,
            "v": 1426.5
          },
          {
            "t": 46.0,
            "v": 1361.5
          },
          {
            "t": 48.0,
            "v": 28.5
          },
          {
            "t": 50.0,
            "v": 16.5
          },
          {
            "t": 52.0,
            "v": 64.5
          },
          {
            "t": 54.0,
            "v": 81.6
          },
          {
            "t": 56.0,
            "v": 0.5
          },
          {
            "t": 58.0,
            "v": 0.0
          },
          {
            "t": 60.0,
            "v": 0.5
          },
          {
            "t": 62.0,
            "v": 0.0
          },
          {
            "t": 64.0,
            "v": 4.5
          },
          {
            "t": 66.0,
            "v": 37.0
          },
          {
            "t": 68.0,
            "v": 49.5
          },
          {
            "t": 70.0,
            "v": 0.0
          },
          {
            "t": 72.0,
            "v": 14.5
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
          },
          {
            "t": 116.0,
            "v": 0.0
          },
          {
            "t": 118.0,
            "v": 0.0
          },
          {
            "t": 120.0,
            "v": 0.0
          },
          {
            "t": 122.0,
            "v": 0.0
          },
          {
            "t": 124.0,
            "v": 0.0
          },
          {
            "t": 126.0,
            "v": 0.0
          },
          {
            "t": 128.0,
            "v": 0.0
          },
          {
            "t": 130.0,
            "v": 0.0
          },
          {
            "t": 132.0,
            "v": 0.0
          },
          {
            "t": 134.0,
            "v": 0.0
          },
          {
            "t": 136.0,
            "v": 0.0
          },
          {
            "t": 138.0,
            "v": 0.0
          },
          {
            "t": 140.0,
            "v": 0.0
          },
          {
            "t": 142.0,
            "v": 0.0
          },
          {
            "t": 144.0,
            "v": 0.0
          },
          {
            "t": 146.0,
            "v": 0.0
          },
          {
            "t": 148.0,
            "v": 0.0
          },
          {
            "t": 150.0,
            "v": 0.0
          },
          {
            "t": 152.0,
            "v": 0.0
          },
          {
            "t": 154.0,
            "v": 0.0
          },
          {
            "t": 156.0,
            "v": 0.0
          },
          {
            "t": 158.0,
            "v": 0.0
          },
          {
            "t": 160.0,
            "v": 0.0
          },
          {
            "t": 162.0,
            "v": 0.0
          },
          {
            "t": 164.0,
            "v": 0.0
          },
          {
            "t": 166.0,
            "v": 0.0
          },
          {
            "t": 168.0,
            "v": 0.0
          },
          {
            "t": 170.0,
            "v": 0.0
          },
          {
            "t": 172.0,
            "v": 0.0
          },
          {
            "t": 174.0,
            "v": 0.0
          },
          {
            "t": 176.0,
            "v": 0.0
          },
          {
            "t": 178.0,
            "v": 0.0
          },
          {
            "t": 180.0,
            "v": 0.0
          },
          {
            "t": 182.0,
            "v": 0.0
          },
          {
            "t": 184.0,
            "v": 0.0
          },
          {
            "t": 186.0,
            "v": 0.0
          },
          {
            "t": 188.0,
            "v": 0.0
          },
          {
            "t": 190.0,
            "v": 0.0
          },
          {
            "t": 192.0,
            "v": 0.0
          },
          {
            "t": 194.0,
            "v": 0.0
          },
          {
            "t": 196.0,
            "v": 0.0
          },
          {
            "t": 198.0,
            "v": 0.0
          },
          {
            "t": 200.0,
            "v": 0.0
          },
          {
            "t": 202.0,
            "v": 0.0
          },
          {
            "t": 204.0,
            "v": 0.0
          },
          {
            "t": 206.0,
            "v": 0.0
          },
          {
            "t": 208.0,
            "v": 0.0
          },
          {
            "t": 210.0,
            "v": 0.0
          },
          {
            "t": 212.0,
            "v": 0.0
          },
          {
            "t": 214.0,
            "v": 0.0
          },
          {
            "t": 216.0,
            "v": 0.0
          },
          {
            "t": 218.0,
            "v": 0.0
          },
          {
            "t": 220.0,
            "v": 0.0
          },
          {
            "t": 222.0,
            "v": 0.0
          },
          {
            "t": 224.0,
            "v": 0.0
          },
          {
            "t": 226.0,
            "v": 0.0
          },
          {
            "t": 228.0,
            "v": 0.0
          },
          {
            "t": 230.0,
            "v": 0.0
          }
        ],
        "nvcswch": [
          {
            "t": 2.0,
            "v": 47.5
          },
          {
            "t": 4.0,
            "v": 456.0
          },
          {
            "t": 6.0,
            "v": 378.5
          },
          {
            "t": 8.0,
            "v": 358.0
          },
          {
            "t": 10.0,
            "v": 361.0
          },
          {
            "t": 12.0,
            "v": 55.5
          },
          {
            "t": 14.0,
            "v": 233.5
          },
          {
            "t": 16.0,
            "v": 415.9
          },
          {
            "t": 18.0,
            "v": 506.5
          },
          {
            "t": 20.0,
            "v": 559.0
          },
          {
            "t": 22.0,
            "v": 78.5
          },
          {
            "t": 24.0,
            "v": 0.0
          },
          {
            "t": 26.0,
            "v": 49.0
          },
          {
            "t": 28.0,
            "v": 541.0
          },
          {
            "t": 30.0,
            "v": 160.5
          },
          {
            "t": 32.0,
            "v": 78.5
          },
          {
            "t": 34.0,
            "v": 520.5
          },
          {
            "t": 36.0,
            "v": 440.0
          },
          {
            "t": 38.0,
            "v": 482.5
          },
          {
            "t": 40.0,
            "v": 382.0
          },
          {
            "t": 42.0,
            "v": 522.5
          },
          {
            "t": 44.0,
            "v": 725.5
          },
          {
            "t": 46.0,
            "v": 200.0
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
            "v": 812.5
          },
          {
            "t": 54.0,
            "v": 1526.4
          },
          {
            "t": 56.0,
            "v": 3.0
          },
          {
            "t": 58.0,
            "v": 0.0
          },
          {
            "t": 60.0,
            "v": 3.5
          },
          {
            "t": 62.0,
            "v": 0.0
          },
          {
            "t": 64.0,
            "v": 38.3
          },
          {
            "t": 66.0,
            "v": 451.5
          },
          {
            "t": 68.0,
            "v": 606.5
          },
          {
            "t": 70.0,
            "v": 0.0
          },
          {
            "t": 72.0,
            "v": 7.5
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
          },
          {
            "t": 116.0,
            "v": 0.0
          },
          {
            "t": 118.0,
            "v": 0.0
          },
          {
            "t": 120.0,
            "v": 0.0
          },
          {
            "t": 122.0,
            "v": 0.0
          },
          {
            "t": 124.0,
            "v": 0.0
          },
          {
            "t": 126.0,
            "v": 0.0
          },
          {
            "t": 128.0,
            "v": 0.0
          },
          {
            "t": 130.0,
            "v": 0.0
          },
          {
            "t": 132.0,
            "v": 0.0
          },
          {
            "t": 134.0,
            "v": 0.0
          },
          {
            "t": 136.0,
            "v": 0.0
          },
          {
            "t": 138.0,
            "v": 0.0
          },
          {
            "t": 140.0,
            "v": 0.0
          },
          {
            "t": 142.0,
            "v": 0.0
          },
          {
            "t": 144.0,
            "v": 0.0
          },
          {
            "t": 146.0,
            "v": 0.0
          },
          {
            "t": 148.0,
            "v": 0.0
          },
          {
            "t": 150.0,
            "v": 0.0
          },
          {
            "t": 152.0,
            "v": 0.0
          },
          {
            "t": 154.0,
            "v": 0.0
          },
          {
            "t": 156.0,
            "v": 0.0
          },
          {
            "t": 158.0,
            "v": 0.0
          },
          {
            "t": 160.0,
            "v": 0.0
          },
          {
            "t": 162.0,
            "v": 0.0
          },
          {
            "t": 164.0,
            "v": 0.0
          },
          {
            "t": 166.0,
            "v": 0.0
          },
          {
            "t": 168.0,
            "v": 0.0
          },
          {
            "t": 170.0,
            "v": 0.0
          },
          {
            "t": 172.0,
            "v": 0.0
          },
          {
            "t": 174.0,
            "v": 0.0
          },
          {
            "t": 176.0,
            "v": 0.0
          },
          {
            "t": 178.0,
            "v": 0.0
          },
          {
            "t": 180.0,
            "v": 0.0
          },
          {
            "t": 182.0,
            "v": 0.0
          },
          {
            "t": 184.0,
            "v": 0.0
          },
          {
            "t": 186.0,
            "v": 0.0
          },
          {
            "t": 188.0,
            "v": 0.0
          },
          {
            "t": 190.0,
            "v": 0.0
          },
          {
            "t": 192.0,
            "v": 0.0
          },
          {
            "t": 194.0,
            "v": 0.0
          },
          {
            "t": 196.0,
            "v": 0.0
          },
          {
            "t": 198.0,
            "v": 0.0
          },
          {
            "t": 200.0,
            "v": 0.0
          },
          {
            "t": 202.0,
            "v": 0.0
          },
          {
            "t": 204.0,
            "v": 0.0
          },
          {
            "t": 206.0,
            "v": 0.0
          },
          {
            "t": 208.0,
            "v": 0.0
          },
          {
            "t": 210.0,
            "v": 0.0
          },
          {
            "t": 212.0,
            "v": 0.0
          },
          {
            "t": 214.0,
            "v": 0.0
          },
          {
            "t": 216.0,
            "v": 0.0
          },
          {
            "t": 218.0,
            "v": 0.0
          },
          {
            "t": 220.0,
            "v": 0.0
          },
          {
            "t": 222.0,
            "v": 0.0
          },
          {
            "t": 224.0,
            "v": 0.0
          },
          {
            "t": 226.0,
            "v": 0.0
          },
          {
            "t": 228.0,
            "v": 0.0
          },
          {
            "t": 230.0,
            "v": 0.0
          }
        ]
      },
      "frps": {
        "cpu": [
          {
            "t": 8.3,
            "v": 20.86
          },
          {
            "t": 13.0,
            "v": 25.84
          },
          {
            "t": 20.6,
            "v": 18.95
          },
          {
            "t": 25.2,
            "v": 2.01
          },
          {
            "t": 28.9,
            "v": 16.18
          },
          {
            "t": 33.9,
            "v": 19.18
          },
          {
            "t": 38.8,
            "v": 20.31
          },
          {
            "t": 43.6,
            "v": 20.29
          },
          {
            "t": 48.4,
            "v": 10.47
          },
          {
            "t": 52.1,
            "v": 8.17
          },
          {
            "t": 58.9,
            "v": 32.98
          },
          {
            "t": 67.9,
            "v": 27.54
          }
        ],
        "rss": [
          {
            "t": 3.5,
            "v": 29.4
          },
          {
            "t": 8.3,
            "v": 31.4
          },
          {
            "t": 13.0,
            "v": 58.0
          },
          {
            "t": 20.6,
            "v": 52.3
          },
          {
            "t": 25.2,
            "v": 54.0
          },
          {
            "t": 28.9,
            "v": 47.2
          },
          {
            "t": 33.9,
            "v": 44.4
          },
          {
            "t": 38.8,
            "v": 44.2
          },
          {
            "t": 43.6,
            "v": 44.0
          },
          {
            "t": 48.4,
            "v": 44.0
          },
          {
            "t": 52.1,
            "v": 53.7
          },
          {
            "t": 58.9,
            "v": 103.8
          },
          {
            "t": 67.9,
            "v": 109.4
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
          }
        ],
        "write_kbs": [
          {
            "t": 0.0,
            "v": 7.9
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
          }
        ],
        "cswch": [
          {
            "t": 0.0,
            "v": 23.4
          },
          {
            "t": 2.0,
            "v": 1647.0
          },
          {
            "t": 4.0,
            "v": 2050.0
          },
          {
            "t": 6.0,
            "v": 1714.0
          },
          {
            "t": 8.0,
            "v": 1727.0
          },
          {
            "t": 10.0,
            "v": 0.5
          },
          {
            "t": 12.0,
            "v": 0.0
          },
          {
            "t": 14.0,
            "v": 134.0
          },
          {
            "t": 16.0,
            "v": 683.6
          },
          {
            "t": 18.0,
            "v": 783.5
          },
          {
            "t": 20.0,
            "v": 1344.5
          },
          {
            "t": 22.0,
            "v": 143.5
          },
          {
            "t": 24.0,
            "v": 0.0
          },
          {
            "t": 26.0,
            "v": 793.0
          },
          {
            "t": 28.0,
            "v": 2231.5
          },
          {
            "t": 30.0,
            "v": 1354.0
          },
          {
            "t": 32.0,
            "v": 1539.5
          },
          {
            "t": 34.0,
            "v": 2178.0
          },
          {
            "t": 36.0,
            "v": 2753.5
          },
          {
            "t": 38.0,
            "v": 2434.0
          },
          {
            "t": 40.0,
            "v": 2120.0
          },
          {
            "t": 42.0,
            "v": 3013.0
          },
          {
            "t": 44.0,
            "v": 1547.5
          },
          {
            "t": 46.0,
            "v": 1123.0
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
            "v": 516.0
          },
          {
            "t": 54.0,
            "v": 419.9
          },
          {
            "t": 56.0,
            "v": 290.0
          },
          {
            "t": 58.0,
            "v": 203.0
          },
          {
            "t": 60.0,
            "v": 533.0
          },
          {
            "t": 62.0,
            "v": 475.5
          },
          {
            "t": 64.0,
            "v": 446.8
          },
          {
            "t": 66.0,
            "v": 228.5
          },
          {
            "t": 68.0,
            "v": 361.5
          },
          {
            "t": 70.0,
            "v": 559.0
          }
        ],
        "nvcswch": [
          {
            "t": 0.0,
            "v": 3.5
          },
          {
            "t": 2.0,
            "v": 309.0
          },
          {
            "t": 4.0,
            "v": 594.5
          },
          {
            "t": 6.0,
            "v": 198.0
          },
          {
            "t": 8.0,
            "v": 404.5
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
            "v": 99.0
          },
          {
            "t": 16.0,
            "v": 479.6
          },
          {
            "t": 18.0,
            "v": 447.0
          },
          {
            "t": 20.0,
            "v": 321.5
          },
          {
            "t": 22.0,
            "v": 53.5
          },
          {
            "t": 24.0,
            "v": 0.0
          },
          {
            "t": 26.0,
            "v": 88.0
          },
          {
            "t": 28.0,
            "v": 462.5
          },
          {
            "t": 30.0,
            "v": 370.0
          },
          {
            "t": 32.0,
            "v": 155.5
          },
          {
            "t": 34.0,
            "v": 647.5
          },
          {
            "t": 36.0,
            "v": 380.0
          },
          {
            "t": 38.0,
            "v": 455.0
          },
          {
            "t": 40.0,
            "v": 462.5
          },
          {
            "t": 42.0,
            "v": 375.0
          },
          {
            "t": 44.0,
            "v": 457.5
          },
          {
            "t": 46.0,
            "v": 187.5
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
            "v": 264.0
          },
          {
            "t": 54.0,
            "v": 253.2
          },
          {
            "t": 56.0,
            "v": 338.5
          },
          {
            "t": 58.0,
            "v": 116.5
          },
          {
            "t": 60.0,
            "v": 383.0
          },
          {
            "t": 62.0,
            "v": 277.5
          },
          {
            "t": 64.0,
            "v": 387.1
          },
          {
            "t": 66.0,
            "v": 174.5
          },
          {
            "t": 68.0,
            "v": 319.0
          },
          {
            "t": 70.0,
            "v": 357.0
          }
        ]
      },
      "frpc": {
        "cpu": [
          {
            "t": 8.3,
            "v": 14.93
          },
          {
            "t": 13.0,
            "v": 24.47
          },
          {
            "t": 20.6,
            "v": 16.68
          },
          {
            "t": 25.2,
            "v": 0.16
          },
          {
            "t": 28.9,
            "v": 10.69
          },
          {
            "t": 33.9,
            "v": 11.77
          },
          {
            "t": 38.8,
            "v": 12.59
          },
          {
            "t": 43.6,
            "v": 12.54
          },
          {
            "t": 48.4,
            "v": 5.86
          },
          {
            "t": 52.1,
            "v": 5.97
          },
          {
            "t": 58.9,
            "v": 22.11
          },
          {
            "t": 67.9,
            "v": 19.03
          }
        ],
        "rss": [
          {
            "t": 3.5,
            "v": 27.0
          },
          {
            "t": 8.3,
            "v": 30.6
          },
          {
            "t": 13.0,
            "v": 54.7
          },
          {
            "t": 20.6,
            "v": 40.9
          },
          {
            "t": 25.2,
            "v": 37.0
          },
          {
            "t": 28.9,
            "v": 29.2
          },
          {
            "t": 33.9,
            "v": 29.4
          },
          {
            "t": 38.8,
            "v": 29.5
          },
          {
            "t": 43.6,
            "v": 31.0
          },
          {
            "t": 48.4,
            "v": 31.7
          },
          {
            "t": 52.1,
            "v": 44.3
          },
          {
            "t": 58.9,
            "v": 114.6
          },
          {
            "t": 67.9,
            "v": 100.3
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
          }
        ],
        "write_kbs": [
          {
            "t": 0.0,
            "v": 4.0
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
          }
        ],
        "cswch": [
          {
            "t": 0.0,
            "v": 4.5
          },
          {
            "t": 2.0,
            "v": 1984.0
          },
          {
            "t": 4.0,
            "v": 1614.0
          },
          {
            "t": 6.0,
            "v": 1262.5
          },
          {
            "t": 8.0,
            "v": 1319.5
          },
          {
            "t": 10.0,
            "v": 1829.5
          },
          {
            "t": 12.0,
            "v": 1155.5
          },
          {
            "t": 14.0,
            "v": 369.5
          },
          {
            "t": 16.0,
            "v": 355.7
          },
          {
            "t": 18.0,
            "v": 1477.0
          },
          {
            "t": 20.0,
            "v": 1578.0
          },
          {
            "t": 22.0,
            "v": 220.0
          },
          {
            "t": 24.0,
            "v": 4.5
          },
          {
            "t": 26.0,
            "v": 794.0
          },
          {
            "t": 28.0,
            "v": 2168.5
          },
          {
            "t": 30.0,
            "v": 1473.0
          },
          {
            "t": 32.0,
            "v": 982.0
          },
          {
            "t": 34.0,
            "v": 2208.5
          },
          {
            "t": 36.0,
            "v": 353.5
          },
          {
            "t": 38.0,
            "v": 1994.5
          },
          {
            "t": 40.0,
            "v": 2031.5
          },
          {
            "t": 42.0,
            "v": 151.5
          },
          {
            "t": 44.0,
            "v": 1923.0
          },
          {
            "t": 46.0,
            "v": 1822.5
          },
          {
            "t": 48.0,
            "v": 3.0
          },
          {
            "t": 50.0,
            "v": 0.0
          },
          {
            "t": 52.0,
            "v": 525.0
          },
          {
            "t": 54.0,
            "v": 549.2
          },
          {
            "t": 56.0,
            "v": 814.0
          },
          {
            "t": 58.0,
            "v": 650.0
          },
          {
            "t": 60.0,
            "v": 240.5
          },
          {
            "t": 62.0,
            "v": 326.0
          },
          {
            "t": 64.0,
            "v": 398.5
          },
          {
            "t": 66.0,
            "v": 1.5
          },
          {
            "t": 68.0,
            "v": 0.0
          },
          {
            "t": 70.0,
            "v": 0.0
          }
        ],
        "nvcswch": [
          {
            "t": 0.0,
            "v": 4.0
          },
          {
            "t": 2.0,
            "v": 389.5
          },
          {
            "t": 4.0,
            "v": 420.0
          },
          {
            "t": 6.0,
            "v": 169.5
          },
          {
            "t": 8.0,
            "v": 413.0
          },
          {
            "t": 10.0,
            "v": 290.0
          },
          {
            "t": 12.0,
            "v": 391.5
          },
          {
            "t": 14.0,
            "v": 153.5
          },
          {
            "t": 16.0,
            "v": 115.4
          },
          {
            "t": 18.0,
            "v": 467.0
          },
          {
            "t": 20.0,
            "v": 374.0
          },
          {
            "t": 22.0,
            "v": 108.5
          },
          {
            "t": 24.0,
            "v": 0.0
          },
          {
            "t": 26.0,
            "v": 72.5
          },
          {
            "t": 28.0,
            "v": 411.0
          },
          {
            "t": 30.0,
            "v": 345.5
          },
          {
            "t": 32.0,
            "v": 115.5
          },
          {
            "t": 34.0,
            "v": 603.5
          },
          {
            "t": 36.0,
            "v": 69.0
          },
          {
            "t": 38.0,
            "v": 426.5
          },
          {
            "t": 40.0,
            "v": 423.0
          },
          {
            "t": 42.0,
            "v": 16.5
          },
          {
            "t": 44.0,
            "v": 469.0
          },
          {
            "t": 46.0,
            "v": 239.5
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
            "v": 168.0
          },
          {
            "t": 54.0,
            "v": 120.9
          },
          {
            "t": 56.0,
            "v": 309.5
          },
          {
            "t": 58.0,
            "v": 293.5
          },
          {
            "t": 60.0,
            "v": 288.0
          },
          {
            "t": 62.0,
            "v": 143.0
          },
          {
            "t": 64.0,
            "v": 206.0
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
          }
        ]
      },
      "other": {
        "cpu": [
          {
            "t": 3.5,
            "v": 4.28
          },
          {
            "t": 8.3,
            "v": 1.68
          },
          {
            "t": 13.0,
            "v": 10.13
          },
          {
            "t": 20.6,
            "v": 10.8
          },
          {
            "t": 25.2,
            "v": 1.25
          },
          {
            "t": 28.9,
            "v": 3.63
          },
          {
            "t": 33.9,
            "v": 4.55
          },
          {
            "t": 38.8,
            "v": 2.36
          },
          {
            "t": 43.6,
            "v": 1.96
          },
          {
            "t": 48.4,
            "v": 1.97
          },
          {
            "t": 52.1,
            "v": 1.78
          },
          {
            "t": 58.9,
            "v": 2.33
          },
          {
            "t": 67.9,
            "v": 1.7
          },
          {
            "t": 75.0,
            "v": 2.04
          },
          {
            "t": 78.7,
            "v": 2.4
          },
          {
            "t": 82.4,
            "v": 1.56
          },
          {
            "t": 86.1,
            "v": 1.88
          },
          {
            "t": 89.8,
            "v": 1.8
          },
          {
            "t": 93.6,
            "v": 1.87
          },
          {
            "t": 97.3,
            "v": 1.46
          },
          {
            "t": 101.1,
            "v": 1.67
          },
          {
            "t": 104.8,
            "v": 1.96
          },
          {
            "t": 108.4,
            "v": 1.92
          },
          {
            "t": 112.1,
            "v": 1.92
          },
          {
            "t": 115.8,
            "v": 1.55
          },
          {
            "t": 119.5,
            "v": 1.79
          },
          {
            "t": 123.3,
            "v": 1.45
          },
          {
            "t": 127.1,
            "v": 1.72
          },
          {
            "t": 130.9,
            "v": 1.9
          },
          {
            "t": 134.6,
            "v": 1.75
          },
          {
            "t": 138.3,
            "v": 2.2
          },
          {
            "t": 142.6,
            "v": 2.29
          },
          {
            "t": 147.2,
            "v": 2.06
          },
          {
            "t": 151.9,
            "v": 1.89
          },
          {
            "t": 156.4,
            "v": 1.75
          },
          {
            "t": 160.9,
            "v": 1.78
          },
          {
            "t": 164.5,
            "v": 2.2
          },
          {
            "t": 169.2,
            "v": 2.46
          },
          {
            "t": 173.8,
            "v": 2.0
          },
          {
            "t": 178.4,
            "v": 1.86
          },
          {
            "t": 183.0,
            "v": 2.06
          },
          {
            "t": 186.9,
            "v": 1.68
          },
          {
            "t": 190.5,
            "v": 2.13
          },
          {
            "t": 195.2,
            "v": 2.05
          },
          {
            "t": 199.8,
            "v": 2.68
          },
          {
            "t": 204.4,
            "v": 1.8
          },
          {
            "t": 209.0,
            "v": 1.85
          },
          {
            "t": 212.7,
            "v": 1.82
          },
          {
            "t": 216.7,
            "v": 2.31
          },
          {
            "t": 221.3,
            "v": 1.95
          },
          {
            "t": 226.0,
            "v": 1.99
          },
          {
            "t": 230.6,
            "v": 2.13
          }
        ],
        "rss": [
          {
            "t": 3.5,
            "v": 129.3
          },
          {
            "t": 8.3,
            "v": 129.9
          },
          {
            "t": 13.0,
            "v": 130.5
          },
          {
            "t": 20.6,
            "v": 131.7
          },
          {
            "t": 25.2,
            "v": 132.2
          },
          {
            "t": 28.9,
            "v": 132.7
          },
          {
            "t": 33.9,
            "v": 133.2
          },
          {
            "t": 38.8,
            "v": 133.3
          },
          {
            "t": 43.6,
            "v": 133.3
          },
          {
            "t": 48.4,
            "v": 133.3
          },
          {
            "t": 52.1,
            "v": 133.4
          },
          {
            "t": 58.9,
            "v": 133.8
          },
          {
            "t": 67.9,
            "v": 133.9
          },
          {
            "t": 75.0,
            "v": 136.9
          },
          {
            "t": 78.7,
            "v": 136.9
          },
          {
            "t": 82.4,
            "v": 136.9
          },
          {
            "t": 86.1,
            "v": 136.9
          },
          {
            "t": 89.8,
            "v": 136.9
          },
          {
            "t": 93.6,
            "v": 136.9
          },
          {
            "t": 97.3,
            "v": 136.9
          },
          {
            "t": 101.1,
            "v": 136.9
          },
          {
            "t": 104.8,
            "v": 136.9
          },
          {
            "t": 108.4,
            "v": 136.9
          },
          {
            "t": 112.1,
            "v": 136.9
          },
          {
            "t": 115.8,
            "v": 136.9
          },
          {
            "t": 119.5,
            "v": 136.9
          },
          {
            "t": 123.3,
            "v": 136.9
          },
          {
            "t": 127.1,
            "v": 136.9
          },
          {
            "t": 130.9,
            "v": 136.9
          },
          {
            "t": 134.6,
            "v": 136.9
          },
          {
            "t": 138.3,
            "v": 136.9
          },
          {
            "t": 142.6,
            "v": 136.9
          },
          {
            "t": 147.2,
            "v": 136.9
          },
          {
            "t": 151.9,
            "v": 137.1
          },
          {
            "t": 156.4,
            "v": 137.1
          },
          {
            "t": 160.9,
            "v": 137.1
          },
          {
            "t": 164.5,
            "v": 137.1
          },
          {
            "t": 169.2,
            "v": 137.2
          },
          {
            "t": 173.8,
            "v": 137.2
          },
          {
            "t": 178.4,
            "v": 137.2
          },
          {
            "t": 183.0,
            "v": 137.2
          },
          {
            "t": 186.9,
            "v": 137.2
          },
          {
            "t": 190.5,
            "v": 137.2
          },
          {
            "t": 195.2,
            "v": 137.1
          },
          {
            "t": 199.8,
            "v": 137.1
          },
          {
            "t": 204.4,
            "v": 137.1
          },
          {
            "t": 209.0,
            "v": 137.1
          },
          {
            "t": 212.7,
            "v": 137.1
          },
          {
            "t": 216.7,
            "v": 137.1
          },
          {
            "t": 221.3,
            "v": 137.1
          },
          {
            "t": 226.0,
            "v": 137.2
          },
          {
            "t": 230.6,
            "v": 137.2
          }
        ],
        "read_kbs": [
          {
            "t": 0.0,
            "v": 92.7
          },
          {
            "t": 2.0,
            "v": -139.0
          },
          {
            "t": 4.0,
            "v": -139.0
          },
          {
            "t": 6.0,
            "v": -139.0
          },
          {
            "t": 8.0,
            "v": -139.0
          },
          {
            "t": 10.0,
            "v": -139.0
          },
          {
            "t": 12.0,
            "v": -139.0
          },
          {
            "t": 14.0,
            "v": -162.0
          },
          {
            "t": 16.0,
            "v": -162.0
          },
          {
            "t": 18.0,
            "v": -153.0
          },
          {
            "t": 20.0,
            "v": -153.0
          },
          {
            "t": 22.0,
            "v": -153.0
          },
          {
            "t": 24.0,
            "v": -153.0
          },
          {
            "t": 26.0,
            "v": -153.0
          },
          {
            "t": 28.0,
            "v": -153.0
          },
          {
            "t": 30.0,
            "v": -153.0
          },
          {
            "t": 32.0,
            "v": -153.0
          },
          {
            "t": 34.0,
            "v": -153.0
          },
          {
            "t": 36.0,
            "v": -153.0
          },
          {
            "t": 38.0,
            "v": -153.0
          },
          {
            "t": 40.0,
            "v": -153.0
          },
          {
            "t": 42.0,
            "v": -153.0
          },
          {
            "t": 44.0,
            "v": -153.0
          },
          {
            "t": 46.0,
            "v": -153.0
          },
          {
            "t": 48.0,
            "v": -153.0
          },
          {
            "t": 50.0,
            "v": -153.0
          },
          {
            "t": 52.0,
            "v": -153.0
          },
          {
            "t": 54.0,
            "v": -153.0
          },
          {
            "t": 56.0,
            "v": -153.0
          },
          {
            "t": 58.0,
            "v": -153.0
          },
          {
            "t": 60.0,
            "v": -153.0
          },
          {
            "t": 62.0,
            "v": -153.0
          },
          {
            "t": 64.0,
            "v": -153.0
          },
          {
            "t": 66.0,
            "v": -153.0
          },
          {
            "t": 68.0,
            "v": -153.0
          },
          {
            "t": 70.0,
            "v": -153.0
          },
          {
            "t": 72.0,
            "v": -153.0
          },
          {
            "t": 74.0,
            "v": -153.0
          },
          {
            "t": 76.0,
            "v": -153.0
          },
          {
            "t": 78.0,
            "v": -153.0
          },
          {
            "t": 80.0,
            "v": -153.0
          },
          {
            "t": 82.0,
            "v": -153.0
          },
          {
            "t": 84.0,
            "v": -153.0
          },
          {
            "t": 86.0,
            "v": -153.0
          },
          {
            "t": 88.0,
            "v": -153.0
          },
          {
            "t": 90.0,
            "v": -153.0
          },
          {
            "t": 92.0,
            "v": -153.0
          },
          {
            "t": 94.0,
            "v": -153.0
          },
          {
            "t": 96.0,
            "v": -153.0
          },
          {
            "t": 98.0,
            "v": -153.0
          },
          {
            "t": 100.0,
            "v": -153.0
          },
          {
            "t": 102.0,
            "v": -153.0
          },
          {
            "t": 104.0,
            "v": -153.0
          },
          {
            "t": 106.0,
            "v": -153.0
          },
          {
            "t": 108.0,
            "v": -153.0
          },
          {
            "t": 110.0,
            "v": -153.0
          },
          {
            "t": 112.0,
            "v": -153.0
          },
          {
            "t": 114.0,
            "v": -153.0
          },
          {
            "t": 116.0,
            "v": -153.0
          },
          {
            "t": 118.0,
            "v": -153.0
          },
          {
            "t": 120.0,
            "v": -153.0
          },
          {
            "t": 122.0,
            "v": -153.0
          },
          {
            "t": 124.0,
            "v": -153.0
          },
          {
            "t": 126.0,
            "v": -153.0
          },
          {
            "t": 128.0,
            "v": -153.0
          },
          {
            "t": 130.0,
            "v": -153.0
          },
          {
            "t": 132.0,
            "v": -153.0
          },
          {
            "t": 134.0,
            "v": -153.0
          },
          {
            "t": 136.0,
            "v": -153.0
          },
          {
            "t": 138.0,
            "v": -153.0
          },
          {
            "t": 140.0,
            "v": -153.0
          },
          {
            "t": 142.0,
            "v": -153.0
          },
          {
            "t": 144.0,
            "v": -153.0
          },
          {
            "t": 146.0,
            "v": -153.0
          },
          {
            "t": 148.0,
            "v": -153.0
          },
          {
            "t": 150.0,
            "v": -153.0
          },
          {
            "t": 152.0,
            "v": -153.0
          },
          {
            "t": 154.0,
            "v": -153.0
          },
          {
            "t": 156.0,
            "v": -153.0
          },
          {
            "t": 158.0,
            "v": -153.0
          },
          {
            "t": 160.0,
            "v": -153.0
          },
          {
            "t": 162.0,
            "v": -153.0
          },
          {
            "t": 164.0,
            "v": -153.0
          },
          {
            "t": 166.0,
            "v": -153.0
          },
          {
            "t": 168.0,
            "v": -153.0
          },
          {
            "t": 170.0,
            "v": -153.0
          },
          {
            "t": 172.0,
            "v": -153.0
          },
          {
            "t": 174.0,
            "v": -153.0
          },
          {
            "t": 176.0,
            "v": -153.0
          },
          {
            "t": 178.0,
            "v": -153.0
          },
          {
            "t": 180.0,
            "v": -153.0
          },
          {
            "t": 182.0,
            "v": -153.0
          },
          {
            "t": 184.0,
            "v": -153.0
          },
          {
            "t": 186.0,
            "v": -153.0
          },
          {
            "t": 188.0,
            "v": -153.0
          },
          {
            "t": 190.0,
            "v": -153.0
          },
          {
            "t": 192.0,
            "v": -153.0
          },
          {
            "t": 194.0,
            "v": -153.0
          },
          {
            "t": 196.0,
            "v": -153.0
          },
          {
            "t": 198.0,
            "v": -153.0
          },
          {
            "t": 200.0,
            "v": -153.0
          },
          {
            "t": 202.0,
            "v": -153.0
          },
          {
            "t": 204.0,
            "v": -153.0
          },
          {
            "t": 206.0,
            "v": -153.0
          },
          {
            "t": 208.0,
            "v": -153.0
          },
          {
            "t": 210.0,
            "v": -153.0
          },
          {
            "t": 212.0,
            "v": -153.0
          },
          {
            "t": 214.0,
            "v": -153.0
          },
          {
            "t": 216.0,
            "v": -153.0
          },
          {
            "t": 218.0,
            "v": -153.0
          },
          {
            "t": 220.0,
            "v": -153.0
          },
          {
            "t": 222.0,
            "v": -153.0
          },
          {
            "t": 224.0,
            "v": -153.0
          },
          {
            "t": 226.0,
            "v": -153.0
          },
          {
            "t": 228.0,
            "v": -153.0
          },
          {
            "t": 230.0,
            "v": -153.0
          },
          {
            "t": 232.0,
            "v": -153.0
          }
        ],
        "write_kbs": [
          {
            "t": 0.0,
            "v": 38510.5
          },
          {
            "t": 2.0,
            "v": -113.0
          },
          {
            "t": 4.0,
            "v": -117.0
          },
          {
            "t": 6.0,
            "v": -123.0
          },
          {
            "t": 8.0,
            "v": -121.0
          },
          {
            "t": 10.0,
            "v": -129.0
          },
          {
            "t": 12.0,
            "v": -129.0
          },
          {
            "t": 14.0,
            "v": -140.1
          },
          {
            "t": 16.0,
            "v": -146.0
          },
          {
            "t": 18.0,
            "v": -131.0
          },
          {
            "t": 20.0,
            "v": -145.0
          },
          {
            "t": 22.0,
            "v": -131.0
          },
          {
            "t": 24.0,
            "v": -133.0
          },
          {
            "t": 26.0,
            "v": -135.0
          },
          {
            "t": 28.0,
            "v": -143.0
          },
          {
            "t": 30.0,
            "v": -131.0
          },
          {
            "t": 32.0,
            "v": -139.0
          },
          {
            "t": 34.0,
            "v": -127.0
          },
          {
            "t": 36.0,
            "v": -139.0
          },
          {
            "t": 38.0,
            "v": -127.0
          },
          {
            "t": 40.0,
            "v": -131.0
          },
          {
            "t": 42.0,
            "v": -135.0
          },
          {
            "t": 44.0,
            "v": -133.0
          },
          {
            "t": 46.0,
            "v": -137.0
          },
          {
            "t": 48.0,
            "v": -137.0
          },
          {
            "t": 50.0,
            "v": -137.0
          },
          {
            "t": 52.0,
            "v": -137.0
          },
          {
            "t": 54.0,
            "v": -133.1
          },
          {
            "t": 56.0,
            "v": -139.0
          },
          {
            "t": 58.0,
            "v": -143.0
          },
          {
            "t": 60.0,
            "v": -129.1
          },
          {
            "t": 62.0,
            "v": -137.0
          },
          {
            "t": 64.0,
            "v": -131.0
          },
          {
            "t": 66.0,
            "v": -141.0
          },
          {
            "t": 68.0,
            "v": -137.0
          },
          {
            "t": 70.0,
            "v": -127.1
          },
          {
            "t": 72.0,
            "v": -81.0
          },
          {
            "t": 74.0,
            "v": -119.0
          },
          {
            "t": 76.0,
            "v": -123.0
          },
          {
            "t": 78.0,
            "v": -129.0
          },
          {
            "t": 80.0,
            "v": -135.0
          },
          {
            "t": 82.0,
            "v": -125.0
          },
          {
            "t": 84.0,
            "v": -133.0
          },
          {
            "t": 86.0,
            "v": -131.0
          },
          {
            "t": 88.0,
            "v": -133.0
          },
          {
            "t": 90.0,
            "v": -127.0
          },
          {
            "t": 92.0,
            "v": -129.0
          },
          {
            "t": 94.0,
            "v": -131.0
          },
          {
            "t": 96.0,
            "v": -127.0
          },
          {
            "t": 98.0,
            "v": -131.0
          },
          {
            "t": 100.0,
            "v": -127.0
          },
          {
            "t": 102.0,
            "v": -129.0
          },
          {
            "t": 104.0,
            "v": -125.0
          },
          {
            "t": 106.0,
            "v": -133.0
          },
          {
            "t": 108.0,
            "v": -119.0
          },
          {
            "t": 110.0,
            "v": -131.0
          },
          {
            "t": 112.0,
            "v": -127.0
          },
          {
            "t": 114.0,
            "v": -129.0
          },
          {
            "t": 116.0,
            "v": -129.0
          },
          {
            "t": 118.0,
            "v": -129.0
          },
          {
            "t": 120.0,
            "v": -131.0
          },
          {
            "t": 122.0,
            "v": -129.0
          },
          {
            "t": 124.0,
            "v": -129.0
          },
          {
            "t": 126.0,
            "v": -133.0
          },
          {
            "t": 128.0,
            "v": -129.0
          },
          {
            "t": 130.0,
            "v": -125.0
          },
          {
            "t": 132.0,
            "v": -133.0
          },
          {
            "t": 134.0,
            "v": -127.0
          },
          {
            "t": 136.0,
            "v": -127.0
          },
          {
            "t": 138.0,
            "v": -131.0
          },
          {
            "t": 140.0,
            "v": -121.0
          },
          {
            "t": 142.0,
            "v": -129.0
          },
          {
            "t": 144.0,
            "v": -131.0
          },
          {
            "t": 146.0,
            "v": -127.0
          },
          {
            "t": 148.0,
            "v": -131.0
          },
          {
            "t": 150.0,
            "v": -131.0
          },
          {
            "t": 152.0,
            "v": -127.0
          },
          {
            "t": 154.0,
            "v": -133.0
          },
          {
            "t": 156.0,
            "v": -127.0
          },
          {
            "t": 158.0,
            "v": -133.0
          },
          {
            "t": 160.0,
            "v": -125.0
          },
          {
            "t": 162.0,
            "v": -129.0
          },
          {
            "t": 164.0,
            "v": -123.0
          },
          {
            "t": 166.0,
            "v": -127.0
          },
          {
            "t": 168.0,
            "v": -131.0
          },
          {
            "t": 170.0,
            "v": -129.0
          },
          {
            "t": 172.0,
            "v": -131.0
          },
          {
            "t": 174.0,
            "v": -129.0
          },
          {
            "t": 176.0,
            "v": -121.0
          },
          {
            "t": 178.0,
            "v": -125.0
          },
          {
            "t": 180.0,
            "v": -131.0
          },
          {
            "t": 182.0,
            "v": -129.0
          },
          {
            "t": 184.0,
            "v": -131.0
          },
          {
            "t": 186.0,
            "v": -127.0
          },
          {
            "t": 188.0,
            "v": -131.0
          },
          {
            "t": 190.0,
            "v": -129.0
          },
          {
            "t": 192.0,
            "v": -129.1
          },
          {
            "t": 194.0,
            "v": -131.0
          },
          {
            "t": 196.0,
            "v": -129.0
          },
          {
            "t": 198.0,
            "v": -131.0
          },
          {
            "t": 200.0,
            "v": -125.0
          },
          {
            "t": 202.0,
            "v": -129.0
          },
          {
            "t": 204.0,
            "v": -129.0
          },
          {
            "t": 206.0,
            "v": -127.0
          },
          {
            "t": 208.0,
            "v": -131.0
          },
          {
            "t": 210.0,
            "v": -131.0
          },
          {
            "t": 212.0,
            "v": -121.0
          },
          {
            "t": 214.0,
            "v": -127.0
          },
          {
            "t": 216.0,
            "v": -129.0
          },
          {
            "t": 218.0,
            "v": -131.0
          },
          {
            "t": 220.0,
            "v": -129.0
          },
          {
            "t": 222.0,
            "v": -127.0
          },
          {
            "t": 224.0,
            "v": -131.0
          },
          {
            "t": 226.0,
            "v": -123.0
          },
          {
            "t": 228.0,
            "v": -129.0
          },
          {
            "t": 230.0,
            "v": -125.0
          },
          {
            "t": 232.0,
            "v": -87.0
          }
        ],
        "cswch": [
          {
            "t": 0.0,
            "v": 1258.7
          },
          {
            "t": 2.0,
            "v": 574.0
          },
          {
            "t": 4.0,
            "v": 1427.0
          },
          {
            "t": 6.0,
            "v": 326.5
          },
          {
            "t": 8.0,
            "v": 1315.5
          },
          {
            "t": 10.0,
            "v": 810.5
          },
          {
            "t": 12.0,
            "v": 769.0
          },
          {
            "t": 14.0,
            "v": 1730.5
          },
          {
            "t": 16.0,
            "v": 1495.6
          },
          {
            "t": 18.0,
            "v": 665.0
          },
          {
            "t": 20.0,
            "v": 1118.5
          },
          {
            "t": 22.0,
            "v": 919.0
          },
          {
            "t": 24.0,
            "v": 686.0
          },
          {
            "t": 26.0,
            "v": 773.0
          },
          {
            "t": 28.0,
            "v": 904.5
          },
          {
            "t": 30.0,
            "v": 1365.0
          },
          {
            "t": 32.0,
            "v": 212.0
          },
          {
            "t": 34.0,
            "v": 1538.0
          },
          {
            "t": 36.0,
            "v": 598.0
          },
          {
            "t": 38.0,
            "v": 1025.5
          },
          {
            "t": 40.0,
            "v": 1157.5
          },
          {
            "t": 42.0,
            "v": 372.5
          },
          {
            "t": 44.0,
            "v": 1471.5
          },
          {
            "t": 46.0,
            "v": 385.0
          },
          {
            "t": 48.0,
            "v": 1297.0
          },
          {
            "t": 50.0,
            "v": 138.0
          },
          {
            "t": 52.0,
            "v": 1186.5
          },
          {
            "t": 54.0,
            "v": 1139.9
          },
          {
            "t": 56.0,
            "v": 944.0
          },
          {
            "t": 58.0,
            "v": 1016.5
          },
          {
            "t": 60.0,
            "v": 1266.5
          },
          {
            "t": 62.0,
            "v": 1181.5
          },
          {
            "t": 64.0,
            "v": 1244.8
          },
          {
            "t": 66.0,
            "v": 843.0
          },
          {
            "t": 68.0,
            "v": 1261.0
          },
          {
            "t": 70.0,
            "v": 1219.0
          },
          {
            "t": 72.0,
            "v": 841.0
          },
          {
            "t": 74.0,
            "v": 880.5
          },
          {
            "t": 76.0,
            "v": 938.5
          },
          {
            "t": 78.0,
            "v": 1154.5
          },
          {
            "t": 80.0,
            "v": 375.0
          },
          {
            "t": 82.0,
            "v": 1346.0
          },
          {
            "t": 84.0,
            "v": 160.5
          },
          {
            "t": 86.0,
            "v": 1442.5
          },
          {
            "t": 88.0,
            "v": 279.0
          },
          {
            "t": 90.0,
            "v": 1275.5
          },
          {
            "t": 92.0,
            "v": 480.5
          },
          {
            "t": 94.0,
            "v": 1088.0
          },
          {
            "t": 96.0,
            "v": 645.5
          },
          {
            "t": 98.0,
            "v": 903.5
          },
          {
            "t": 100.0,
            "v": 818.0
          },
          {
            "t": 102.0,
            "v": 644.5
          },
          {
            "t": 104.0,
            "v": 1035.0
          },
          {
            "t": 106.0,
            "v": 363.0
          },
          {
            "t": 108.0,
            "v": 1310.5
          },
          {
            "t": 110.0,
            "v": 147.0
          },
          {
            "t": 112.0,
            "v": 1403.5
          },
          {
            "t": 114.0,
            "v": 319.5
          },
          {
            "t": 116.0,
            "v": 1259.5
          },
          {
            "t": 118.0,
            "v": 506.5
          },
          {
            "t": 120.0,
            "v": 1096.0
          },
          {
            "t": 122.0,
            "v": 681.0
          },
          {
            "t": 124.0,
            "v": 925.0
          },
          {
            "t": 126.0,
            "v": 839.5
          },
          {
            "t": 128.0,
            "v": 748.5
          },
          {
            "t": 130.0,
            "v": 976.0
          },
          {
            "t": 132.0,
            "v": 511.5
          },
          {
            "t": 134.0,
            "v": 1152.0
          },
          {
            "t": 136.0,
            "v": 597.0
          },
          {
            "t": 138.0,
            "v": 1328.0
          },
          {
            "t": 140.0,
            "v": 534.0
          },
          {
            "t": 142.0,
            "v": 1076.0
          },
          {
            "t": 144.0,
            "v": 1010.0
          },
          {
            "t": 146.0,
            "v": 683.0
          },
          {
            "t": 148.0,
            "v": 1296.5
          },
          {
            "t": 150.0,
            "v": 223.5
          },
          {
            "t": 152.0,
            "v": 1473.5
          },
          {
            "t": 154.0,
            "v": 363.0
          },
          {
            "t": 156.0,
            "v": 1230.5
          },
          {
            "t": 158.0,
            "v": 687.0
          },
          {
            "t": 160.0,
            "v": 947.0
          },
          {
            "t": 162.0,
            "v": 448.5
          },
          {
            "t": 164.0,
            "v": 1108.5
          },
          {
            "t": 166.0,
            "v": 895.0
          },
          {
            "t": 168.0,
            "v": 648.0
          },
          {
            "t": 170.0,
            "v": 1302.5
          },
          {
            "t": 172.0,
            "v": 254.5
          },
          {
            "t": 174.0,
            "v": 1471.0
          },
          {
            "t": 176.0,
            "v": 400.5
          },
          {
            "t": 178.0,
            "v": 1203.0
          },
          {
            "t": 180.0,
            "v": 756.0
          },
          {
            "t": 182.0,
            "v": 857.0
          },
          {
            "t": 184.0,
            "v": 690.0
          },
          {
            "t": 186.0,
            "v": 950.0
          },
          {
            "t": 188.0,
            "v": 494.5
          },
          {
            "t": 190.0,
            "v": 1113.5
          },
          {
            "t": 192.0,
            "v": 876.0
          },
          {
            "t": 194.0,
            "v": 716.0
          },
          {
            "t": 196.0,
            "v": 1646.0
          },
          {
            "t": 198.0,
            "v": 456.5
          },
          {
            "t": 200.0,
            "v": 1448.5
          },
          {
            "t": 202.0,
            "v": 369.5
          },
          {
            "t": 204.0,
            "v": 1186.0
          },
          {
            "t": 206.0,
            "v": 799.0
          },
          {
            "t": 208.0,
            "v": 861.0
          },
          {
            "t": 210.0,
            "v": 554.5
          },
          {
            "t": 212.0,
            "v": 1045.0
          },
          {
            "t": 214.0,
            "v": 573.0
          },
          {
            "t": 216.0,
            "v": 998.5
          },
          {
            "t": 218.0,
            "v": 1007.5
          },
          {
            "t": 220.0,
            "v": 561.5
          },
          {
            "t": 222.0,
            "v": 1466.5
          },
          {
            "t": 224.0,
            "v": 160.5
          },
          {
            "t": 226.0,
            "v": 1481.5
          },
          {
            "t": 228.0,
            "v": 457.5
          },
          {
            "t": 230.0,
            "v": 1118.0
          },
          {
            "t": 232.0,
            "v": 947.5
          }
        ],
        "nvcswch": [
          {
            "t": 0.0,
            "v": 16.5
          },
          {
            "t": 2.0,
            "v": 85.5
          },
          {
            "t": 4.0,
            "v": 269.0
          },
          {
            "t": 6.0,
            "v": 61.0
          },
          {
            "t": 8.0,
            "v": 245.0
          },
          {
            "t": 10.0,
            "v": 139.0
          },
          {
            "t": 12.0,
            "v": 115.5
          },
          {
            "t": 14.0,
            "v": 327.5
          },
          {
            "t": 16.0,
            "v": 317.4
          },
          {
            "t": 18.0,
            "v": 45.5
          },
          {
            "t": 20.0,
            "v": 202.0
          },
          {
            "t": 22.0,
            "v": 68.5
          },
          {
            "t": 24.0,
            "v": 5.5
          },
          {
            "t": 26.0,
            "v": 9.5
          },
          {
            "t": 28.0,
            "v": 182.0
          },
          {
            "t": 30.0,
            "v": 220.5
          },
          {
            "t": 32.0,
            "v": 64.0
          },
          {
            "t": 34.0,
            "v": 287.0
          },
          {
            "t": 36.0,
            "v": 134.5
          },
          {
            "t": 38.0,
            "v": 196.5
          },
          {
            "t": 40.0,
            "v": 195.5
          },
          {
            "t": 42.0,
            "v": 77.0
          },
          {
            "t": 44.0,
            "v": 230.0
          },
          {
            "t": 46.0,
            "v": 57.0
          },
          {
            "t": 48.0,
            "v": 17.0
          },
          {
            "t": 50.0,
            "v": 0.5
          },
          {
            "t": 52.0,
            "v": 157.0
          },
          {
            "t": 54.0,
            "v": 228.9
          },
          {
            "t": 56.0,
            "v": 79.0
          },
          {
            "t": 58.0,
            "v": 84.0
          },
          {
            "t": 60.0,
            "v": 99.5
          },
          {
            "t": 62.0,
            "v": 134.5
          },
          {
            "t": 64.0,
            "v": 112.9
          },
          {
            "t": 66.0,
            "v": 82.0
          },
          {
            "t": 68.0,
            "v": 135.5
          },
          {
            "t": 70.0,
            "v": 93.0
          },
          {
            "t": 72.0,
            "v": 28.5
          },
          {
            "t": 74.0,
            "v": 8.0
          },
          {
            "t": 76.0,
            "v": 120.5
          },
          {
            "t": 78.0,
            "v": 25.0
          },
          {
            "t": 80.0,
            "v": 2.0
          },
          {
            "t": 82.0,
            "v": 25.0
          },
          {
            "t": 84.0,
            "v": 2.5
          },
          {
            "t": 86.0,
            "v": 23.5
          },
          {
            "t": 88.0,
            "v": 4.5
          },
          {
            "t": 90.0,
            "v": 26.0
          },
          {
            "t": 92.0,
            "v": 8.5
          },
          {
            "t": 94.0,
            "v": 24.0
          },
          {
            "t": 96.0,
            "v": 10.5
          },
          {
            "t": 98.0,
            "v": 21.0
          },
          {
            "t": 100.0,
            "v": 15.0
          },
          {
            "t": 102.0,
            "v": 8.0
          },
          {
            "t": 104.0,
            "v": 15.0
          },
          {
            "t": 106.0,
            "v": 5.0
          },
          {
            "t": 108.0,
            "v": 19.0
          },
          {
            "t": 110.0,
            "v": 1.0
          },
          {
            "t": 112.0,
            "v": 22.0
          },
          {
            "t": 114.0,
            "v": 5.0
          },
          {
            "t": 116.0,
            "v": 28.5
          },
          {
            "t": 118.0,
            "v": 10.0
          },
          {
            "t": 120.0,
            "v": 27.5
          },
          {
            "t": 122.0,
            "v": 15.5
          },
          {
            "t": 124.0,
            "v": 19.5
          },
          {
            "t": 126.0,
            "v": 18.5
          },
          {
            "t": 128.0,
            "v": 14.5
          },
          {
            "t": 130.0,
            "v": 23.0
          },
          {
            "t": 132.0,
            "v": 4.5
          },
          {
            "t": 134.0,
            "v": 9.5
          },
          {
            "t": 136.0,
            "v": 111.0
          },
          {
            "t": 138.0,
            "v": 102.5
          },
          {
            "t": 140.0,
            "v": 91.5
          },
          {
            "t": 142.0,
            "v": 126.0
          },
          {
            "t": 144.0,
            "v": 114.5
          },
          {
            "t": 146.0,
            "v": 88.5
          },
          {
            "t": 148.0,
            "v": 157.5
          },
          {
            "t": 150.0,
            "v": 33.0
          },
          {
            "t": 152.0,
            "v": 157.0
          },
          {
            "t": 154.0,
            "v": 53.0
          },
          {
            "t": 156.0,
            "v": 142.0
          },
          {
            "t": 158.0,
            "v": 62.0
          },
          {
            "t": 160.0,
            "v": 17.5
          },
          {
            "t": 162.0,
            "v": 7.5
          },
          {
            "t": 164.0,
            "v": 109.0
          },
          {
            "t": 166.0,
            "v": 98.5
          },
          {
            "t": 168.0,
            "v": 82.0
          },
          {
            "t": 170.0,
            "v": 160.0
          },
          {
            "t": 172.0,
            "v": 31.0
          },
          {
            "t": 174.0,
            "v": 170.0
          },
          {
            "t": 176.0,
            "v": 55.5
          },
          {
            "t": 178.0,
            "v": 139.5
          },
          {
            "t": 180.0,
            "v": 73.5
          },
          {
            "t": 182.0,
            "v": 69.5
          },
          {
            "t": 184.0,
            "v": 7.0
          },
          {
            "t": 186.0,
            "v": 8.5
          },
          {
            "t": 188.0,
            "v": 14.0
          },
          {
            "t": 190.0,
            "v": 123.0
          },
          {
            "t": 192.0,
            "v": 107.5
          },
          {
            "t": 194.0,
            "v": 104.5
          },
          {
            "t": 196.0,
            "v": 270.0
          },
          {
            "t": 198.0,
            "v": 54.5
          },
          {
            "t": 200.0,
            "v": 165.0
          },
          {
            "t": 202.0,
            "v": 61.5
          },
          {
            "t": 204.0,
            "v": 126.5
          },
          {
            "t": 206.0,
            "v": 106.0
          },
          {
            "t": 208.0,
            "v": 13.5
          },
          {
            "t": 210.0,
            "v": 12.0
          },
          {
            "t": 212.0,
            "v": 62.0
          },
          {
            "t": 214.0,
            "v": 71.0
          },
          {
            "t": 216.0,
            "v": 122.5
          },
          {
            "t": 218.0,
            "v": 130.5
          },
          {
            "t": 220.0,
            "v": 80.0
          },
          {
            "t": 222.0,
            "v": 192.5
          },
          {
            "t": 224.0,
            "v": 25.5
          },
          {
            "t": 226.0,
            "v": 162.5
          },
          {
            "t": 228.0,
            "v": 51.0
          },
          {
            "t": 230.0,
            "v": 122.5
          },
          {
            "t": 232.0,
            "v": 139.5
          }
        ]
      }
    },
    "system": {
      "cpu": [
        {
          "t": 0.0,
          "v": 34.4
        },
        {
          "t": 2.0,
          "v": 56.1
        },
        {
          "t": 4.0,
          "v": 83.8
        },
        {
          "t": 6.0,
          "v": 63.4
        },
        {
          "t": 8.0,
          "v": 81.2
        },
        {
          "t": 10.0,
          "v": 74.2
        },
        {
          "t": 12.0,
          "v": 90.0
        },
        {
          "t": 14.0,
          "v": 98.0
        },
        {
          "t": 16.0,
          "v": 97.8
        },
        {
          "t": 18.0,
          "v": 85.4
        },
        {
          "t": 20.0,
          "v": 82.1
        },
        {
          "t": 22.0,
          "v": 32.1
        },
        {
          "t": 24.0,
          "v": 11.9
        },
        {
          "t": 26.0,
          "v": 27.9
        },
        {
          "t": 28.0,
          "v": 77.8
        },
        {
          "t": 30.0,
          "v": 82.4
        },
        {
          "t": 32.0,
          "v": 59.6
        },
        {
          "t": 34.0,
          "v": 83.3
        },
        {
          "t": 36.0,
          "v": 68.2
        },
        {
          "t": 38.0,
          "v": 74.7
        },
        {
          "t": 40.0,
          "v": 78.9
        },
        {
          "t": 42.0,
          "v": 63.4
        },
        {
          "t": 44.0,
          "v": 82.9
        },
        {
          "t": 46.0,
          "v": 45.5
        },
        {
          "t": 48.0,
          "v": 22.8
        },
        {
          "t": 50.0,
          "v": 1.8
        },
        {
          "t": 52.0,
          "v": 82.5
        },
        {
          "t": 54.0,
          "v": 98.5
        },
        {
          "t": 56.0,
          "v": 98.4
        },
        {
          "t": 58.0,
          "v": 98.4
        },
        {
          "t": 60.0,
          "v": 99.2
        },
        {
          "t": 62.0,
          "v": 98.8
        },
        {
          "t": 64.0,
          "v": 99.0
        },
        {
          "t": 66.0,
          "v": 98.7
        },
        {
          "t": 68.0,
          "v": 98.6
        },
        {
          "t": 70.0,
          "v": 99.1
        },
        {
          "t": 72.0,
          "v": 69.1
        },
        {
          "t": 74.0,
          "v": 14.9
        },
        {
          "t": 76.0,
          "v": 12.0
        },
        {
          "t": 78.0,
          "v": 23.1
        },
        {
          "t": 80.0,
          "v": 12.4
        },
        {
          "t": 82.0,
          "v": 30.3
        },
        {
          "t": 84.0,
          "v": 6.6
        },
        {
          "t": 86.0,
          "v": 32.2
        },
        {
          "t": 88.0,
          "v": 7.9
        },
        {
          "t": 90.0,
          "v": 29.4
        },
        {
          "t": 92.0,
          "v": 12.0
        },
        {
          "t": 94.0,
          "v": 25.7
        },
        {
          "t": 96.0,
          "v": 15.2
        },
        {
          "t": 98.0,
          "v": 20.4
        },
        {
          "t": 100.0,
          "v": 16.6
        },
        {
          "t": 102.0,
          "v": 12.8
        },
        {
          "t": 104.0,
          "v": 21.8
        },
        {
          "t": 106.0,
          "v": 5.9
        },
        {
          "t": 108.0,
          "v": 25.1
        },
        {
          "t": 110.0,
          "v": 4.8
        },
        {
          "t": 112.0,
          "v": 30.4
        },
        {
          "t": 114.0,
          "v": 9.3
        },
        {
          "t": 116.0,
          "v": 30.3
        },
        {
          "t": 118.0,
          "v": 13.8
        },
        {
          "t": 120.0,
          "v": 30.7
        },
        {
          "t": 122.0,
          "v": 17.1
        },
        {
          "t": 124.0,
          "v": 24.1
        },
        {
          "t": 126.0,
          "v": 21.1
        },
        {
          "t": 128.0,
          "v": 19.5
        },
        {
          "t": 130.0,
          "v": 21.4
        },
        {
          "t": 132.0,
          "v": 10.5
        },
        {
          "t": 134.0,
          "v": 20.8
        },
        {
          "t": 136.0,
          "v": 4.7
        },
        {
          "t": 138.0,
          "v": 59.1
        },
        {
          "t": 140.0,
          "v": 65.2
        },
        {
          "t": 142.0,
          "v": 73.7
        },
        {
          "t": 144.0,
          "v": 72.8
        },
        {
          "t": 146.0,
          "v": 64.9
        },
        {
          "t": 148.0,
          "v": 77.5
        },
        {
          "t": 150.0,
          "v": 60.1
        },
        {
          "t": 152.0,
          "v": 79.1
        },
        {
          "t": 154.0,
          "v": 64.0
        },
        {
          "t": 156.0,
          "v": 75.1
        },
        {
          "t": 158.0,
          "v": 29.1
        },
        {
          "t": 160.0,
          "v": 16.7
        },
        {
          "t": 162.0,
          "v": 16.3
        },
        {
          "t": 164.0,
          "v": 74.5
        },
        {
          "t": 166.0,
          "v": 71.8
        },
        {
          "t": 168.0,
          "v": 65.3
        },
        {
          "t": 170.0,
          "v": 77.3
        },
        {
          "t": 172.0,
          "v": 60.8
        },
        {
          "t": 174.0,
          "v": 79.3
        },
        {
          "t": 176.0,
          "v": 63.6
        },
        {
          "t": 178.0,
          "v": 75.5
        },
        {
          "t": 180.0,
          "v": 67.2
        },
        {
          "t": 182.0,
          "v": 61.9
        },
        {
          "t": 184.0,
          "v": 13.5
        },
        {
          "t": 186.0,
          "v": 16.6
        },
        {
          "t": 188.0,
          "v": 47.1
        },
        {
          "t": 190.0,
          "v": 75.4
        },
        {
          "t": 192.0,
          "v": 70.7
        },
        {
          "t": 194.0,
          "v": 69.2
        },
        {
          "t": 196.0,
          "v": 78.7
        },
        {
          "t": 198.0,
          "v": 63.1
        },
        {
          "t": 200.0,
          "v": 80.5
        },
        {
          "t": 202.0,
          "v": 63.8
        },
        {
          "t": 204.0,
          "v": 76.1
        },
        {
          "t": 206.0,
          "v": 71.0
        },
        {
          "t": 208.0,
          "v": 33.2
        },
        {
          "t": 210.0,
          "v": 10.7
        },
        {
          "t": 212.0,
          "v": 26.9
        },
        {
          "t": 214.0,
          "v": 68.0
        },
        {
          "t": 216.0,
          "v": 73.7
        },
        {
          "t": 218.0,
          "v": 73.7
        },
        {
          "t": 220.0,
          "v": 67.6
        },
        {
          "t": 222.0,
          "v": 77.9
        },
        {
          "t": 224.0,
          "v": 60.9
        },
        {
          "t": 226.0,
          "v": 80.6
        },
        {
          "t": 228.0,
          "v": 66.3
        },
        {
          "t": 230.0,
          "v": 74.8
        },
        {
          "t": 232.0,
          "v": 66.8
        }
      ],
      "cpu_usr": [
        {
          "t": 0.0,
          "v": 13.2
        },
        {
          "t": 2.0,
          "v": 30.7
        },
        {
          "t": 4.0,
          "v": 39.7
        },
        {
          "t": 6.0,
          "v": 36.7
        },
        {
          "t": 8.0,
          "v": 39.4
        },
        {
          "t": 10.0,
          "v": 39.5
        },
        {
          "t": 12.0,
          "v": 58.0
        },
        {
          "t": 14.0,
          "v": 55.5
        },
        {
          "t": 16.0,
          "v": 57.2
        },
        {
          "t": 18.0,
          "v": 53.4
        },
        {
          "t": 20.0,
          "v": 43.1
        },
        {
          "t": 22.0,
          "v": 13.0
        },
        {
          "t": 24.0,
          "v": 2.1
        },
        {
          "t": 26.0,
          "v": 10.8
        },
        {
          "t": 28.0,
          "v": 41.9
        },
        {
          "t": 30.0,
          "v": 41.6
        },
        {
          "t": 32.0,
          "v": 33.9
        },
        {
          "t": 34.0,
          "v": 41.4
        },
        {
          "t": 36.0,
          "v": 36.0
        },
        {
          "t": 38.0,
          "v": 37.8
        },
        {
          "t": 40.0,
          "v": 39.0
        },
        {
          "t": 42.0,
          "v": 34.8
        },
        {
          "t": 44.0,
          "v": 41.0
        },
        {
          "t": 46.0,
          "v": 25.0
        },
        {
          "t": 48.0,
          "v": 4.3
        },
        {
          "t": 50.0,
          "v": 0.4
        },
        {
          "t": 52.0,
          "v": 50.4
        },
        {
          "t": 54.0,
          "v": 63.5
        },
        {
          "t": 56.0,
          "v": 64.7
        },
        {
          "t": 58.0,
          "v": 64.8
        },
        {
          "t": 60.0,
          "v": 60.5
        },
        {
          "t": 62.0,
          "v": 61.5
        },
        {
          "t": 64.0,
          "v": 61.8
        },
        {
          "t": 66.0,
          "v": 64.8
        },
        {
          "t": 68.0,
          "v": 60.2
        },
        {
          "t": 70.0,
          "v": 61.5
        },
        {
          "t": 72.0,
          "v": 48.0
        },
        {
          "t": 74.0,
          "v": 3.2
        },
        {
          "t": 76.0,
          "v": 2.4
        },
        {
          "t": 78.0,
          "v": 5.3
        },
        {
          "t": 80.0,
          "v": 6.0
        },
        {
          "t": 82.0,
          "v": 7.7
        },
        {
          "t": 84.0,
          "v": 3.4
        },
        {
          "t": 86.0,
          "v": 8.1
        },
        {
          "t": 88.0,
          "v": 3.4
        },
        {
          "t": 90.0,
          "v": 7.8
        },
        {
          "t": 92.0,
          "v": 3.9
        },
        {
          "t": 94.0,
          "v": 7.4
        },
        {
          "t": 96.0,
          "v": 4.6
        },
        {
          "t": 98.0,
          "v": 5.5
        },
        {
          "t": 100.0,
          "v": 4.2
        },
        {
          "t": 102.0,
          "v": 3.2
        },
        {
          "t": 104.0,
          "v": 6.9
        },
        {
          "t": 106.0,
          "v": 0.9
        },
        {
          "t": 108.0,
          "v": 5.6
        },
        {
          "t": 110.0,
          "v": 2.3
        },
        {
          "t": 112.0,
          "v": 7.8
        },
        {
          "t": 114.0,
          "v": 4.4
        },
        {
          "t": 116.0,
          "v": 8.9
        },
        {
          "t": 118.0,
          "v": 5.1
        },
        {
          "t": 120.0,
          "v": 11.6
        },
        {
          "t": 122.0,
          "v": 5.9
        },
        {
          "t": 124.0,
          "v": 7.5
        },
        {
          "t": 126.0,
          "v": 6.9
        },
        {
          "t": 128.0,
          "v": 6.0
        },
        {
          "t": 130.0,
          "v": 5.8
        },
        {
          "t": 132.0,
          "v": 2.3
        },
        {
          "t": 134.0,
          "v": 4.3
        },
        {
          "t": 136.0,
          "v": 0.8
        },
        {
          "t": 138.0,
          "v": 25.5
        },
        {
          "t": 140.0,
          "v": 34.0
        },
        {
          "t": 142.0,
          "v": 36.0
        },
        {
          "t": 144.0,
          "v": 34.9
        },
        {
          "t": 146.0,
          "v": 30.7
        },
        {
          "t": 148.0,
          "v": 34.8
        },
        {
          "t": 150.0,
          "v": 32.7
        },
        {
          "t": 152.0,
          "v": 35.1
        },
        {
          "t": 154.0,
          "v": 34.9
        },
        {
          "t": 156.0,
          "v": 34.6
        },
        {
          "t": 158.0,
          "v": 11.9
        },
        {
          "t": 160.0,
          "v": 3.2
        },
        {
          "t": 162.0,
          "v": 6.1
        },
        {
          "t": 164.0,
          "v": 38.2
        },
        {
          "t": 166.0,
          "v": 37.9
        },
        {
          "t": 168.0,
          "v": 34.1
        },
        {
          "t": 170.0,
          "v": 39.1
        },
        {
          "t": 172.0,
          "v": 36.0
        },
        {
          "t": 174.0,
          "v": 38.2
        },
        {
          "t": 176.0,
          "v": 35.7
        },
        {
          "t": 178.0,
          "v": 38.5
        },
        {
          "t": 180.0,
          "v": 33.8
        },
        {
          "t": 182.0,
          "v": 31.7
        },
        {
          "t": 184.0,
          "v": 2.9
        },
        {
          "t": 186.0,
          "v": 3.0
        },
        {
          "t": 188.0,
          "v": 24.8
        },
        {
          "t": 190.0,
          "v": 37.1
        },
        {
          "t": 192.0,
          "v": 31.9
        },
        {
          "t": 194.0,
          "v": 34.7
        },
        {
          "t": 196.0,
          "v": 36.4
        },
        {
          "t": 198.0,
          "v": 34.7
        },
        {
          "t": 200.0,
          "v": 36.8
        },
        {
          "t": 202.0,
          "v": 34.7
        },
        {
          "t": 204.0,
          "v": 36.8
        },
        {
          "t": 206.0,
          "v": 35.6
        },
        {
          "t": 208.0,
          "v": 12.4
        },
        {
          "t": 210.0,
          "v": 2.0
        },
        {
          "t": 212.0,
          "v": 7.9
        },
        {
          "t": 214.0,
          "v": 36.4
        },
        {
          "t": 216.0,
          "v": 37.0
        },
        {
          "t": 218.0,
          "v": 35.6
        },
        {
          "t": 220.0,
          "v": 36.1
        },
        {
          "t": 222.0,
          "v": 33.1
        },
        {
          "t": 224.0,
          "v": 36.2
        },
        {
          "t": 226.0,
          "v": 37.8
        },
        {
          "t": 228.0,
          "v": 35.7
        },
        {
          "t": 230.0,
          "v": 36.2
        },
        {
          "t": 232.0,
          "v": 34.3
        }
      ],
      "cpu_sys": [
        {
          "t": 0.0,
          "v": 20.6
        },
        {
          "t": 2.0,
          "v": 21.2
        },
        {
          "t": 4.0,
          "v": 38.5
        },
        {
          "t": 6.0,
          "v": 20.3
        },
        {
          "t": 8.0,
          "v": 36.8
        },
        {
          "t": 10.0,
          "v": 28.8
        },
        {
          "t": 12.0,
          "v": 25.6
        },
        {
          "t": 14.0,
          "v": 36.0
        },
        {
          "t": 16.0,
          "v": 32.7
        },
        {
          "t": 18.0,
          "v": 24.4
        },
        {
          "t": 20.0,
          "v": 32.7
        },
        {
          "t": 22.0,
          "v": 17.3
        },
        {
          "t": 24.0,
          "v": 9.8
        },
        {
          "t": 26.0,
          "v": 15.4
        },
        {
          "t": 28.0,
          "v": 28.8
        },
        {
          "t": 30.0,
          "v": 35.8
        },
        {
          "t": 32.0,
          "v": 18.6
        },
        {
          "t": 34.0,
          "v": 37.1
        },
        {
          "t": 36.0,
          "v": 25.9
        },
        {
          "t": 38.0,
          "v": 32.0
        },
        {
          "t": 40.0,
          "v": 34.3
        },
        {
          "t": 42.0,
          "v": 22.2
        },
        {
          "t": 44.0,
          "v": 36.6
        },
        {
          "t": 46.0,
          "v": 16.4
        },
        {
          "t": 48.0,
          "v": 18.5
        },
        {
          "t": 50.0,
          "v": 1.4
        },
        {
          "t": 52.0,
          "v": 27.1
        },
        {
          "t": 54.0,
          "v": 28.1
        },
        {
          "t": 56.0,
          "v": 25.7
        },
        {
          "t": 58.0,
          "v": 25.7
        },
        {
          "t": 60.0,
          "v": 30.3
        },
        {
          "t": 62.0,
          "v": 28.9
        },
        {
          "t": 64.0,
          "v": 29.4
        },
        {
          "t": 66.0,
          "v": 24.5
        },
        {
          "t": 68.0,
          "v": 29.8
        },
        {
          "t": 70.0,
          "v": 29.9
        },
        {
          "t": 72.0,
          "v": 18.7
        },
        {
          "t": 74.0,
          "v": 11.7
        },
        {
          "t": 76.0,
          "v": 9.3
        },
        {
          "t": 78.0,
          "v": 17.4
        },
        {
          "t": 80.0,
          "v": 6.3
        },
        {
          "t": 82.0,
          "v": 22.1
        },
        {
          "t": 84.0,
          "v": 3.0
        },
        {
          "t": 86.0,
          "v": 23.3
        },
        {
          "t": 88.0,
          "v": 4.2
        },
        {
          "t": 90.0,
          "v": 21.3
        },
        {
          "t": 92.0,
          "v": 7.5
        },
        {
          "t": 94.0,
          "v": 17.9
        },
        {
          "t": 96.0,
          "v": 10.3
        },
        {
          "t": 98.0,
          "v": 14.4
        },
        {
          "t": 100.0,
          "v": 12.3
        },
        {
          "t": 102.0,
          "v": 9.7
        },
        {
          "t": 104.0,
          "v": 14.9
        },
        {
          "t": 106.0,
          "v": 5.0
        },
        {
          "t": 108.0,
          "v": 19.4
        },
        {
          "t": 110.0,
          "v": 2.2
        },
        {
          "t": 112.0,
          "v": 22.1
        },
        {
          "t": 114.0,
          "v": 4.5
        },
        {
          "t": 116.0,
          "v": 20.9
        },
        {
          "t": 118.0,
          "v": 8.3
        },
        {
          "t": 120.0,
          "v": 18.4
        },
        {
          "t": 122.0,
          "v": 10.7
        },
        {
          "t": 124.0,
          "v": 15.8
        },
        {
          "t": 126.0,
          "v": 13.7
        },
        {
          "t": 128.0,
          "v": 13.1
        },
        {
          "t": 130.0,
          "v": 15.4
        },
        {
          "t": 132.0,
          "v": 8.0
        },
        {
          "t": 134.0,
          "v": 16.4
        },
        {
          "t": 136.0,
          "v": 3.9
        },
        {
          "t": 138.0,
          "v": 30.7
        },
        {
          "t": 140.0,
          "v": 26.4
        },
        {
          "t": 142.0,
          "v": 33.2
        },
        {
          "t": 144.0,
          "v": 33.4
        },
        {
          "t": 146.0,
          "v": 29.4
        },
        {
          "t": 148.0,
          "v": 38.2
        },
        {
          "t": 150.0,
          "v": 22.5
        },
        {
          "t": 152.0,
          "v": 39.5
        },
        {
          "t": 154.0,
          "v": 24.7
        },
        {
          "t": 156.0,
          "v": 35.9
        },
        {
          "t": 158.0,
          "v": 15.9
        },
        {
          "t": 160.0,
          "v": 13.5
        },
        {
          "t": 162.0,
          "v": 9.7
        },
        {
          "t": 164.0,
          "v": 31.8
        },
        {
          "t": 166.0,
          "v": 30.1
        },
        {
          "t": 168.0,
          "v": 26.8
        },
        {
          "t": 170.0,
          "v": 34.3
        },
        {
          "t": 172.0,
          "v": 19.9
        },
        {
          "t": 174.0,
          "v": 37.1
        },
        {
          "t": 176.0,
          "v": 23.0
        },
        {
          "t": 178.0,
          "v": 32.7
        },
        {
          "t": 180.0,
          "v": 29.4
        },
        {
          "t": 182.0,
          "v": 26.3
        },
        {
          "t": 184.0,
          "v": 10.6
        },
        {
          "t": 186.0,
          "v": 13.5
        },
        {
          "t": 188.0,
          "v": 19.1
        },
        {
          "t": 190.0,
          "v": 33.9
        },
        {
          "t": 192.0,
          "v": 34.0
        },
        {
          "t": 194.0,
          "v": 29.4
        },
        {
          "t": 196.0,
          "v": 37.9
        },
        {
          "t": 198.0,
          "v": 22.9
        },
        {
          "t": 200.0,
          "v": 39.3
        },
        {
          "t": 202.0,
          "v": 24.3
        },
        {
          "t": 204.0,
          "v": 34.5
        },
        {
          "t": 206.0,
          "v": 30.8
        },
        {
          "t": 208.0,
          "v": 19.1
        },
        {
          "t": 210.0,
          "v": 8.7
        },
        {
          "t": 212.0,
          "v": 18.2
        },
        {
          "t": 214.0,
          "v": 26.9
        },
        {
          "t": 216.0,
          "v": 31.7
        },
        {
          "t": 218.0,
          "v": 33.2
        },
        {
          "t": 220.0,
          "v": 26.6
        },
        {
          "t": 222.0,
          "v": 40.4
        },
        {
          "t": 224.0,
          "v": 19.3
        },
        {
          "t": 226.0,
          "v": 38.6
        },
        {
          "t": 228.0,
          "v": 25.7
        },
        {
          "t": 230.0,
          "v": 34.0
        },
        {
          "t": 232.0,
          "v": 28.6
        }
      ],
      "cpu_irq": [
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
        },
        {
          "t": 114.0,
          "v": 0.0
        },
        {
          "t": 116.0,
          "v": 0.0
        },
        {
          "t": 118.0,
          "v": 0.0
        },
        {
          "t": 120.0,
          "v": 0.0
        },
        {
          "t": 122.0,
          "v": 0.0
        },
        {
          "t": 124.0,
          "v": 0.0
        },
        {
          "t": 126.0,
          "v": 0.0
        },
        {
          "t": 128.0,
          "v": 0.0
        },
        {
          "t": 130.0,
          "v": 0.0
        },
        {
          "t": 132.0,
          "v": 0.0
        },
        {
          "t": 134.0,
          "v": 0.0
        },
        {
          "t": 136.0,
          "v": 0.0
        },
        {
          "t": 138.0,
          "v": 0.0
        },
        {
          "t": 140.0,
          "v": 0.0
        },
        {
          "t": 142.0,
          "v": 0.0
        },
        {
          "t": 144.0,
          "v": 0.0
        },
        {
          "t": 146.0,
          "v": 0.0
        },
        {
          "t": 148.0,
          "v": 0.0
        },
        {
          "t": 150.0,
          "v": 0.0
        },
        {
          "t": 152.0,
          "v": 0.0
        },
        {
          "t": 154.0,
          "v": 0.0
        },
        {
          "t": 156.0,
          "v": 0.0
        },
        {
          "t": 158.0,
          "v": 0.0
        },
        {
          "t": 160.0,
          "v": 0.0
        },
        {
          "t": 162.0,
          "v": 0.0
        },
        {
          "t": 164.0,
          "v": 0.0
        },
        {
          "t": 166.0,
          "v": 0.0
        },
        {
          "t": 168.0,
          "v": 0.0
        },
        {
          "t": 170.0,
          "v": 0.0
        },
        {
          "t": 172.0,
          "v": 0.0
        },
        {
          "t": 174.0,
          "v": 0.0
        },
        {
          "t": 176.0,
          "v": 0.0
        },
        {
          "t": 178.0,
          "v": 0.0
        },
        {
          "t": 180.0,
          "v": 0.0
        },
        {
          "t": 182.0,
          "v": 0.0
        },
        {
          "t": 184.0,
          "v": 0.0
        },
        {
          "t": 186.0,
          "v": 0.0
        },
        {
          "t": 188.0,
          "v": 0.0
        },
        {
          "t": 190.0,
          "v": 0.0
        },
        {
          "t": 192.0,
          "v": 0.0
        },
        {
          "t": 194.0,
          "v": 0.0
        },
        {
          "t": 196.0,
          "v": 0.0
        },
        {
          "t": 198.0,
          "v": 0.0
        },
        {
          "t": 200.0,
          "v": 0.0
        },
        {
          "t": 202.0,
          "v": 0.0
        },
        {
          "t": 204.0,
          "v": 0.0
        },
        {
          "t": 206.0,
          "v": 0.0
        },
        {
          "t": 208.0,
          "v": 0.0
        },
        {
          "t": 210.0,
          "v": 0.0
        },
        {
          "t": 212.0,
          "v": 0.0
        },
        {
          "t": 214.0,
          "v": 0.0
        },
        {
          "t": 216.0,
          "v": 0.0
        },
        {
          "t": 218.0,
          "v": 0.0
        },
        {
          "t": 220.0,
          "v": 0.0
        },
        {
          "t": 222.0,
          "v": 0.0
        },
        {
          "t": 224.0,
          "v": 0.0
        },
        {
          "t": 226.0,
          "v": 0.0
        },
        {
          "t": 228.0,
          "v": 0.0
        },
        {
          "t": 230.0,
          "v": 0.0
        },
        {
          "t": 232.0,
          "v": 0.0
        }
      ],
      "cpu_soft": [
        {
          "t": 0.0,
          "v": 0.6
        },
        {
          "t": 2.0,
          "v": 4.2
        },
        {
          "t": 4.0,
          "v": 5.5
        },
        {
          "t": 6.0,
          "v": 6.4
        },
        {
          "t": 8.0,
          "v": 5.1
        },
        {
          "t": 10.0,
          "v": 5.9
        },
        {
          "t": 12.0,
          "v": 6.3
        },
        {
          "t": 14.0,
          "v": 6.5
        },
        {
          "t": 16.0,
          "v": 7.9
        },
        {
          "t": 18.0,
          "v": 7.6
        },
        {
          "t": 20.0,
          "v": 6.4
        },
        {
          "t": 22.0,
          "v": 1.8
        },
        {
          "t": 24.0,
          "v": 0.0
        },
        {
          "t": 26.0,
          "v": 1.6
        },
        {
          "t": 28.0,
          "v": 6.3
        },
        {
          "t": 30.0,
          "v": 5.0
        },
        {
          "t": 32.0,
          "v": 7.1
        },
        {
          "t": 34.0,
          "v": 4.8
        },
        {
          "t": 36.0,
          "v": 6.2
        },
        {
          "t": 38.0,
          "v": 5.0
        },
        {
          "t": 40.0,
          "v": 5.6
        },
        {
          "t": 42.0,
          "v": 6.5
        },
        {
          "t": 44.0,
          "v": 5.2
        },
        {
          "t": 46.0,
          "v": 4.2
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
          "v": 5.0
        },
        {
          "t": 54.0,
          "v": 7.0
        },
        {
          "t": 56.0,
          "v": 7.9
        },
        {
          "t": 58.0,
          "v": 7.9
        },
        {
          "t": 60.0,
          "v": 8.4
        },
        {
          "t": 62.0,
          "v": 8.3
        },
        {
          "t": 64.0,
          "v": 7.8
        },
        {
          "t": 66.0,
          "v": 9.5
        },
        {
          "t": 68.0,
          "v": 8.6
        },
        {
          "t": 70.0,
          "v": 7.7
        },
        {
          "t": 72.0,
          "v": 2.4
        },
        {
          "t": 74.0,
          "v": 0.0
        },
        {
          "t": 76.0,
          "v": 0.3
        },
        {
          "t": 78.0,
          "v": 0.4
        },
        {
          "t": 80.0,
          "v": 0.1
        },
        {
          "t": 82.0,
          "v": 0.5
        },
        {
          "t": 84.0,
          "v": 0.3
        },
        {
          "t": 86.0,
          "v": 0.8
        },
        {
          "t": 88.0,
          "v": 0.4
        },
        {
          "t": 90.0,
          "v": 0.3
        },
        {
          "t": 92.0,
          "v": 0.5
        },
        {
          "t": 94.0,
          "v": 0.4
        },
        {
          "t": 96.0,
          "v": 0.4
        },
        {
          "t": 98.0,
          "v": 0.5
        },
        {
          "t": 100.0,
          "v": 0.1
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
          "v": 0.1
        },
        {
          "t": 110.0,
          "v": 0.4
        },
        {
          "t": 112.0,
          "v": 0.5
        },
        {
          "t": 114.0,
          "v": 0.4
        },
        {
          "t": 116.0,
          "v": 0.5
        },
        {
          "t": 118.0,
          "v": 0.4
        },
        {
          "t": 120.0,
          "v": 0.7
        },
        {
          "t": 122.0,
          "v": 0.5
        },
        {
          "t": 124.0,
          "v": 0.8
        },
        {
          "t": 126.0,
          "v": 0.5
        },
        {
          "t": 128.0,
          "v": 0.5
        },
        {
          "t": 130.0,
          "v": 0.1
        },
        {
          "t": 132.0,
          "v": 0.2
        },
        {
          "t": 134.0,
          "v": 0.1
        },
        {
          "t": 136.0,
          "v": 0.0
        },
        {
          "t": 138.0,
          "v": 2.9
        },
        {
          "t": 140.0,
          "v": 4.8
        },
        {
          "t": 142.0,
          "v": 4.5
        },
        {
          "t": 144.0,
          "v": 4.5
        },
        {
          "t": 146.0,
          "v": 4.8
        },
        {
          "t": 148.0,
          "v": 4.5
        },
        {
          "t": 150.0,
          "v": 4.9
        },
        {
          "t": 152.0,
          "v": 4.5
        },
        {
          "t": 154.0,
          "v": 4.4
        },
        {
          "t": 156.0,
          "v": 4.6
        },
        {
          "t": 158.0,
          "v": 1.3
        },
        {
          "t": 160.0,
          "v": 0.0
        },
        {
          "t": 162.0,
          "v": 0.5
        },
        {
          "t": 164.0,
          "v": 4.4
        },
        {
          "t": 166.0,
          "v": 3.8
        },
        {
          "t": 168.0,
          "v": 4.4
        },
        {
          "t": 170.0,
          "v": 3.9
        },
        {
          "t": 172.0,
          "v": 4.9
        },
        {
          "t": 174.0,
          "v": 4.0
        },
        {
          "t": 176.0,
          "v": 4.9
        },
        {
          "t": 178.0,
          "v": 4.3
        },
        {
          "t": 180.0,
          "v": 4.0
        },
        {
          "t": 182.0,
          "v": 3.9
        },
        {
          "t": 184.0,
          "v": 0.0
        },
        {
          "t": 186.0,
          "v": 0.0
        },
        {
          "t": 188.0,
          "v": 3.2
        },
        {
          "t": 190.0,
          "v": 4.4
        },
        {
          "t": 192.0,
          "v": 4.8
        },
        {
          "t": 194.0,
          "v": 5.0
        },
        {
          "t": 196.0,
          "v": 4.4
        },
        {
          "t": 198.0,
          "v": 5.5
        },
        {
          "t": 200.0,
          "v": 4.3
        },
        {
          "t": 202.0,
          "v": 4.8
        },
        {
          "t": 204.0,
          "v": 4.8
        },
        {
          "t": 206.0,
          "v": 4.6
        },
        {
          "t": 208.0,
          "v": 1.7
        },
        {
          "t": 210.0,
          "v": 0.0
        },
        {
          "t": 212.0,
          "v": 0.8
        },
        {
          "t": 214.0,
          "v": 4.7
        },
        {
          "t": 216.0,
          "v": 4.9
        },
        {
          "t": 218.0,
          "v": 4.9
        },
        {
          "t": 220.0,
          "v": 4.9
        },
        {
          "t": 222.0,
          "v": 4.4
        },
        {
          "t": 224.0,
          "v": 5.4
        },
        {
          "t": 226.0,
          "v": 4.2
        },
        {
          "t": 228.0,
          "v": 4.9
        },
        {
          "t": 230.0,
          "v": 4.7
        },
        {
          "t": 232.0,
          "v": 3.9
        }
      ],
      "cpu_iowait": [
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
          "v": 0.9
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
        },
        {
          "t": 116.0,
          "v": 0.0
        },
        {
          "t": 118.0,
          "v": 0.0
        },
        {
          "t": 120.0,
          "v": 0.0
        },
        {
          "t": 122.0,
          "v": 0.0
        },
        {
          "t": 124.0,
          "v": 0.0
        },
        {
          "t": 126.0,
          "v": 0.0
        },
        {
          "t": 128.0,
          "v": 0.0
        },
        {
          "t": 130.0,
          "v": 0.0
        },
        {
          "t": 132.0,
          "v": 0.0
        },
        {
          "t": 134.0,
          "v": 0.0
        },
        {
          "t": 136.0,
          "v": 0.0
        },
        {
          "t": 138.0,
          "v": 0.0
        },
        {
          "t": 140.0,
          "v": 0.0
        },
        {
          "t": 142.0,
          "v": 0.0
        },
        {
          "t": 144.0,
          "v": 0.0
        },
        {
          "t": 146.0,
          "v": 0.0
        },
        {
          "t": 148.0,
          "v": 0.0
        },
        {
          "t": 150.0,
          "v": 0.0
        },
        {
          "t": 152.0,
          "v": 0.0
        },
        {
          "t": 154.0,
          "v": 0.0
        },
        {
          "t": 156.0,
          "v": 0.0
        },
        {
          "t": 158.0,
          "v": 0.0
        },
        {
          "t": 160.0,
          "v": 0.0
        },
        {
          "t": 162.0,
          "v": 0.0
        },
        {
          "t": 164.0,
          "v": 0.0
        },
        {
          "t": 166.0,
          "v": 0.0
        },
        {
          "t": 168.0,
          "v": 0.0
        },
        {
          "t": 170.0,
          "v": 0.0
        },
        {
          "t": 172.0,
          "v": 0.0
        },
        {
          "t": 174.0,
          "v": 0.0
        },
        {
          "t": 176.0,
          "v": 0.0
        },
        {
          "t": 178.0,
          "v": 0.0
        },
        {
          "t": 180.0,
          "v": 0.0
        },
        {
          "t": 182.0,
          "v": 0.0
        },
        {
          "t": 184.0,
          "v": 0.0
        },
        {
          "t": 186.0,
          "v": 0.0
        },
        {
          "t": 188.0,
          "v": 0.0
        },
        {
          "t": 190.0,
          "v": 0.0
        },
        {
          "t": 192.0,
          "v": 0.0
        },
        {
          "t": 194.0,
          "v": 0.0
        },
        {
          "t": 196.0,
          "v": 0.0
        },
        {
          "t": 198.0,
          "v": 0.0
        },
        {
          "t": 200.0,
          "v": 0.0
        },
        {
          "t": 202.0,
          "v": 0.0
        },
        {
          "t": 204.0,
          "v": 0.0
        },
        {
          "t": 206.0,
          "v": 0.0
        },
        {
          "t": 208.0,
          "v": 0.0
        },
        {
          "t": 210.0,
          "v": 0.0
        },
        {
          "t": 212.0,
          "v": 0.0
        },
        {
          "t": 214.0,
          "v": 0.0
        },
        {
          "t": 216.0,
          "v": 0.0
        },
        {
          "t": 218.0,
          "v": 0.0
        },
        {
          "t": 220.0,
          "v": 0.0
        },
        {
          "t": 222.0,
          "v": 0.0
        },
        {
          "t": 224.0,
          "v": 0.0
        },
        {
          "t": 226.0,
          "v": 0.0
        },
        {
          "t": 228.0,
          "v": 0.0
        },
        {
          "t": 230.0,
          "v": 0.0
        },
        {
          "t": 232.0,
          "v": 0.0
        }
      ],
      "cpu_steal": [
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
        },
        {
          "t": 114.0,
          "v": 0.0
        },
        {
          "t": 116.0,
          "v": 0.0
        },
        {
          "t": 118.0,
          "v": 0.0
        },
        {
          "t": 120.0,
          "v": 0.0
        },
        {
          "t": 122.0,
          "v": 0.0
        },
        {
          "t": 124.0,
          "v": 0.0
        },
        {
          "t": 126.0,
          "v": 0.0
        },
        {
          "t": 128.0,
          "v": 0.0
        },
        {
          "t": 130.0,
          "v": 0.0
        },
        {
          "t": 132.0,
          "v": 0.0
        },
        {
          "t": 134.0,
          "v": 0.0
        },
        {
          "t": 136.0,
          "v": 0.0
        },
        {
          "t": 138.0,
          "v": 0.0
        },
        {
          "t": 140.0,
          "v": 0.0
        },
        {
          "t": 142.0,
          "v": 0.0
        },
        {
          "t": 144.0,
          "v": 0.0
        },
        {
          "t": 146.0,
          "v": 0.0
        },
        {
          "t": 148.0,
          "v": 0.0
        },
        {
          "t": 150.0,
          "v": 0.0
        },
        {
          "t": 152.0,
          "v": 0.0
        },
        {
          "t": 154.0,
          "v": 0.0
        },
        {
          "t": 156.0,
          "v": 0.0
        },
        {
          "t": 158.0,
          "v": 0.0
        },
        {
          "t": 160.0,
          "v": 0.0
        },
        {
          "t": 162.0,
          "v": 0.0
        },
        {
          "t": 164.0,
          "v": 0.0
        },
        {
          "t": 166.0,
          "v": 0.0
        },
        {
          "t": 168.0,
          "v": 0.0
        },
        {
          "t": 170.0,
          "v": 0.0
        },
        {
          "t": 172.0,
          "v": 0.0
        },
        {
          "t": 174.0,
          "v": 0.0
        },
        {
          "t": 176.0,
          "v": 0.0
        },
        {
          "t": 178.0,
          "v": 0.0
        },
        {
          "t": 180.0,
          "v": 0.0
        },
        {
          "t": 182.0,
          "v": 0.0
        },
        {
          "t": 184.0,
          "v": 0.0
        },
        {
          "t": 186.0,
          "v": 0.0
        },
        {
          "t": 188.0,
          "v": 0.0
        },
        {
          "t": 190.0,
          "v": 0.0
        },
        {
          "t": 192.0,
          "v": 0.0
        },
        {
          "t": 194.0,
          "v": 0.0
        },
        {
          "t": 196.0,
          "v": 0.0
        },
        {
          "t": 198.0,
          "v": 0.0
        },
        {
          "t": 200.0,
          "v": 0.0
        },
        {
          "t": 202.0,
          "v": 0.0
        },
        {
          "t": 204.0,
          "v": 0.0
        },
        {
          "t": 206.0,
          "v": 0.0
        },
        {
          "t": 208.0,
          "v": 0.0
        },
        {
          "t": 210.0,
          "v": 0.0
        },
        {
          "t": 212.0,
          "v": 0.0
        },
        {
          "t": 214.0,
          "v": 0.0
        },
        {
          "t": 216.0,
          "v": 0.0
        },
        {
          "t": 218.0,
          "v": 0.0
        },
        {
          "t": 220.0,
          "v": 0.0
        },
        {
          "t": 222.0,
          "v": 0.0
        },
        {
          "t": 224.0,
          "v": 0.0
        },
        {
          "t": 226.0,
          "v": 0.0
        },
        {
          "t": 228.0,
          "v": 0.0
        },
        {
          "t": 230.0,
          "v": 0.0
        },
        {
          "t": 232.0,
          "v": 0.0
        }
      ]
    },
    "nproc": 4,
    "k6OffsetSeconds": 73,
    "machine": {
      "cpu_cores": 4,
      "mem_total_mb": 15990
    },
    "network": {
      "rx_kbs": [
        {
          "t": 0.0,
          "v": 6495.2
        },
        {
          "t": 2.0,
          "v": 8.6
        },
        {
          "t": 4.0,
          "v": 0.1
        },
        {
          "t": 6.0,
          "v": 0.1
        },
        {
          "t": 8.0,
          "v": 1.9
        },
        {
          "t": 10.0,
          "v": 3.5
        },
        {
          "t": 12.0,
          "v": 0.8
        },
        {
          "t": 14.0,
          "v": 1.9
        },
        {
          "t": 16.0,
          "v": 0.1
        },
        {
          "t": 18.0,
          "v": 0.1
        },
        {
          "t": 20.0,
          "v": 1.9
        },
        {
          "t": 22.0,
          "v": 0.1
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
          "v": 2.6
        },
        {
          "t": 30.0,
          "v": 0.1
        },
        {
          "t": 32.0,
          "v": 2.5
        },
        {
          "t": 34.0,
          "v": 0.2
        },
        {
          "t": 36.0,
          "v": 0.1
        },
        {
          "t": 38.0,
          "v": 1.9
        },
        {
          "t": 40.0,
          "v": 0.6
        },
        {
          "t": 42.0,
          "v": 0.3
        },
        {
          "t": 44.0,
          "v": 2.1
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
          "v": 1.9
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
          "v": 1.9
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
          "v": 1.9
        },
        {
          "t": 64.0,
          "v": 0.1
        },
        {
          "t": 66.0,
          "v": 0.1
        },
        {
          "t": 68.0,
          "v": 1.9
        },
        {
          "t": 70.0,
          "v": 0.5
        },
        {
          "t": 72.0,
          "v": 14.8
        },
        {
          "t": 74.0,
          "v": 2.0
        },
        {
          "t": 76.0,
          "v": 0.1
        },
        {
          "t": 78.0,
          "v": 0.1
        },
        {
          "t": 80.0,
          "v": 1.9
        },
        {
          "t": 82.0,
          "v": 0.1
        },
        {
          "t": 84.0,
          "v": 0.3
        },
        {
          "t": 86.0,
          "v": 3.5
        },
        {
          "t": 88.0,
          "v": 1.2
        },
        {
          "t": 90.0,
          "v": 0.1
        },
        {
          "t": 92.0,
          "v": 2.1
        },
        {
          "t": 94.0,
          "v": 0.1
        },
        {
          "t": 96.0,
          "v": 0.1
        },
        {
          "t": 98.0,
          "v": 1.9
        },
        {
          "t": 100.0,
          "v": 0.3
        },
        {
          "t": 102.0,
          "v": 0.3
        },
        {
          "t": 104.0,
          "v": 2.2
        },
        {
          "t": 106.0,
          "v": 0.1
        },
        {
          "t": 108.0,
          "v": 0.1
        },
        {
          "t": 110.0,
          "v": 1.9
        },
        {
          "t": 112.0,
          "v": 0.1
        },
        {
          "t": 114.0,
          "v": 0.1
        },
        {
          "t": 116.0,
          "v": 2.0
        },
        {
          "t": 118.0,
          "v": 0.1
        },
        {
          "t": 120.0,
          "v": 0.1
        },
        {
          "t": 122.0,
          "v": 1.9
        },
        {
          "t": 124.0,
          "v": 0.1
        },
        {
          "t": 126.0,
          "v": 0.1
        },
        {
          "t": 128.0,
          "v": 1.9
        },
        {
          "t": 130.0,
          "v": 0.3
        },
        {
          "t": 132.0,
          "v": 0.1
        },
        {
          "t": 134.0,
          "v": 2.2
        },
        {
          "t": 136.0,
          "v": 0.1
        },
        {
          "t": 138.0,
          "v": 0.1
        },
        {
          "t": 140.0,
          "v": 1.9
        },
        {
          "t": 142.0,
          "v": 0.1
        },
        {
          "t": 144.0,
          "v": 0.1
        },
        {
          "t": 146.0,
          "v": 1.9
        },
        {
          "t": 148.0,
          "v": 2.7
        },
        {
          "t": 150.0,
          "v": 0.1
        },
        {
          "t": 152.0,
          "v": 2.1
        },
        {
          "t": 154.0,
          "v": 0.1
        },
        {
          "t": 156.0,
          "v": 0.1
        },
        {
          "t": 158.0,
          "v": 1.9
        },
        {
          "t": 160.0,
          "v": 0.3
        },
        {
          "t": 162.0,
          "v": 0.3
        },
        {
          "t": 164.0,
          "v": 2.2
        },
        {
          "t": 166.0,
          "v": 0.1
        },
        {
          "t": 168.0,
          "v": 0.1
        },
        {
          "t": 170.0,
          "v": 1.9
        },
        {
          "t": 172.0,
          "v": 0.1
        },
        {
          "t": 174.0,
          "v": 0.1
        },
        {
          "t": 176.0,
          "v": 2.0
        },
        {
          "t": 178.0,
          "v": 0.1
        },
        {
          "t": 180.0,
          "v": 0.1
        },
        {
          "t": 182.0,
          "v": 1.9
        },
        {
          "t": 184.0,
          "v": 0.3
        },
        {
          "t": 186.0,
          "v": 0.1
        },
        {
          "t": 188.0,
          "v": 2.0
        },
        {
          "t": 190.0,
          "v": 0.3
        },
        {
          "t": 192.0,
          "v": 0.1
        },
        {
          "t": 194.0,
          "v": 2.0
        },
        {
          "t": 196.0,
          "v": 0.1
        },
        {
          "t": 198.0,
          "v": 0.1
        },
        {
          "t": 200.0,
          "v": 1.4
        },
        {
          "t": 202.0,
          "v": 0.7
        },
        {
          "t": 204.0,
          "v": 0.1
        },
        {
          "t": 206.0,
          "v": 0.1
        },
        {
          "t": 208.0,
          "v": 4.5
        },
        {
          "t": 210.0,
          "v": 0.1
        },
        {
          "t": 212.0,
          "v": 0.1
        },
        {
          "t": 214.0,
          "v": 2.1
        },
        {
          "t": 216.0,
          "v": 0.1
        },
        {
          "t": 218.0,
          "v": 0.1
        },
        {
          "t": 220.0,
          "v": 2.1
        },
        {
          "t": 222.0,
          "v": 0.3
        },
        {
          "t": 224.0,
          "v": 0.4
        },
        {
          "t": 226.0,
          "v": 1.9
        },
        {
          "t": 228.0,
          "v": 0.1
        },
        {
          "t": 230.0,
          "v": 0.1
        },
        {
          "t": 232.0,
          "v": 4.2
        }
      ],
      "tx_kbs": [
        {
          "t": 0.0,
          "v": 61.1
        },
        {
          "t": 2.0,
          "v": 10.2
        },
        {
          "t": 4.0,
          "v": 0.5
        },
        {
          "t": 6.0,
          "v": 0.5
        },
        {
          "t": 8.0,
          "v": 4.3
        },
        {
          "t": 10.0,
          "v": 1.0
        },
        {
          "t": 12.0,
          "v": 11.4
        },
        {
          "t": 14.0,
          "v": 4.4
        },
        {
          "t": 16.0,
          "v": 0.5
        },
        {
          "t": 18.0,
          "v": 0.5
        },
        {
          "t": 20.0,
          "v": 4.4
        },
        {
          "t": 22.0,
          "v": 0.5
        },
        {
          "t": 24.0,
          "v": 0.5
        },
        {
          "t": 26.0,
          "v": 4.6
        },
        {
          "t": 28.0,
          "v": 1.8
        },
        {
          "t": 30.0,
          "v": 0.5
        },
        {
          "t": 32.0,
          "v": 31.6
        },
        {
          "t": 34.0,
          "v": 1.7
        },
        {
          "t": 36.0,
          "v": 0.5
        },
        {
          "t": 38.0,
          "v": 4.4
        },
        {
          "t": 40.0,
          "v": 17.1
        },
        {
          "t": 42.0,
          "v": 0.7
        },
        {
          "t": 44.0,
          "v": 5.9
        },
        {
          "t": 46.0,
          "v": 0.5
        },
        {
          "t": 48.0,
          "v": 0.5
        },
        {
          "t": 50.0,
          "v": 4.3
        },
        {
          "t": 52.0,
          "v": 0.5
        },
        {
          "t": 54.0,
          "v": 0.7
        },
        {
          "t": 56.0,
          "v": 4.4
        },
        {
          "t": 58.0,
          "v": 0.7
        },
        {
          "t": 60.0,
          "v": 0.9
        },
        {
          "t": 62.0,
          "v": 4.5
        },
        {
          "t": 64.0,
          "v": 0.5
        },
        {
          "t": 66.0,
          "v": 0.5
        },
        {
          "t": 68.0,
          "v": 4.4
        },
        {
          "t": 70.0,
          "v": 19.2
        },
        {
          "t": 72.0,
          "v": 37.5
        },
        {
          "t": 74.0,
          "v": 5.5
        },
        {
          "t": 76.0,
          "v": 1.6
        },
        {
          "t": 78.0,
          "v": 1.6
        },
        {
          "t": 80.0,
          "v": 5.4
        },
        {
          "t": 82.0,
          "v": 1.6
        },
        {
          "t": 84.0,
          "v": 2.7
        },
        {
          "t": 86.0,
          "v": 6.3
        },
        {
          "t": 88.0,
          "v": 2.3
        },
        {
          "t": 90.0,
          "v": 1.6
        },
        {
          "t": 92.0,
          "v": 5.8
        },
        {
          "t": 94.0,
          "v": 1.6
        },
        {
          "t": 96.0,
          "v": 1.6
        },
        {
          "t": 98.0,
          "v": 5.5
        },
        {
          "t": 100.0,
          "v": 7.9
        },
        {
          "t": 102.0,
          "v": 1.9
        },
        {
          "t": 104.0,
          "v": 7.1
        },
        {
          "t": 106.0,
          "v": 1.6
        },
        {
          "t": 108.0,
          "v": 1.6
        },
        {
          "t": 110.0,
          "v": 5.5
        },
        {
          "t": 112.0,
          "v": 1.7
        },
        {
          "t": 114.0,
          "v": 1.7
        },
        {
          "t": 116.0,
          "v": 5.6
        },
        {
          "t": 118.0,
          "v": 1.8
        },
        {
          "t": 120.0,
          "v": 1.7
        },
        {
          "t": 122.0,
          "v": 5.6
        },
        {
          "t": 124.0,
          "v": 1.7
        },
        {
          "t": 126.0,
          "v": 1.7
        },
        {
          "t": 128.0,
          "v": 5.6
        },
        {
          "t": 130.0,
          "v": 7.8
        },
        {
          "t": 132.0,
          "v": 1.8
        },
        {
          "t": 134.0,
          "v": 6.8
        },
        {
          "t": 136.0,
          "v": 1.7
        },
        {
          "t": 138.0,
          "v": 1.8
        },
        {
          "t": 140.0,
          "v": 5.6
        },
        {
          "t": 142.0,
          "v": 1.8
        },
        {
          "t": 144.0,
          "v": 1.8
        },
        {
          "t": 146.0,
          "v": 5.6
        },
        {
          "t": 148.0,
          "v": 3.2
        },
        {
          "t": 150.0,
          "v": 1.8
        },
        {
          "t": 152.0,
          "v": 6.0
        },
        {
          "t": 154.0,
          "v": 1.8
        },
        {
          "t": 156.0,
          "v": 1.8
        },
        {
          "t": 158.0,
          "v": 5.6
        },
        {
          "t": 160.0,
          "v": 7.2
        },
        {
          "t": 162.0,
          "v": 2.1
        },
        {
          "t": 164.0,
          "v": 7.2
        },
        {
          "t": 166.0,
          "v": 1.8
        },
        {
          "t": 168.0,
          "v": 1.8
        },
        {
          "t": 170.0,
          "v": 5.6
        },
        {
          "t": 172.0,
          "v": 1.8
        },
        {
          "t": 174.0,
          "v": 1.8
        },
        {
          "t": 176.0,
          "v": 5.7
        },
        {
          "t": 178.0,
          "v": 1.8
        },
        {
          "t": 180.0,
          "v": 1.8
        },
        {
          "t": 182.0,
          "v": 5.6
        },
        {
          "t": 184.0,
          "v": 2.8
        },
        {
          "t": 186.0,
          "v": 1.8
        },
        {
          "t": 188.0,
          "v": 5.7
        },
        {
          "t": 190.0,
          "v": 8.3
        },
        {
          "t": 192.0,
          "v": 1.8
        },
        {
          "t": 194.0,
          "v": 5.7
        },
        {
          "t": 196.0,
          "v": 1.8
        },
        {
          "t": 198.0,
          "v": 1.8
        },
        {
          "t": 200.0,
          "v": 2.4
        },
        {
          "t": 202.0,
          "v": 5.1
        },
        {
          "t": 204.0,
          "v": 1.8
        },
        {
          "t": 206.0,
          "v": 1.8
        },
        {
          "t": 208.0,
          "v": 7.1
        },
        {
          "t": 210.0,
          "v": 1.8
        },
        {
          "t": 212.0,
          "v": 1.8
        },
        {
          "t": 214.0,
          "v": 6.0
        },
        {
          "t": 216.0,
          "v": 1.8
        },
        {
          "t": 218.0,
          "v": 1.8
        },
        {
          "t": 220.0,
          "v": 11.8
        },
        {
          "t": 222.0,
          "v": 2.0
        },
        {
          "t": 224.0,
          "v": 3.4
        },
        {
          "t": 226.0,
          "v": 5.7
        },
        {
          "t": 228.0,
          "v": 1.9
        },
        {
          "t": 230.0,
          "v": 1.8
        },
        {
          "t": 232.0,
          "v": 7.2
        }
      ]
    },
    "paging": {
      "majflt": [
        {
          "t": 0.0,
          "v": 85462.0
        },
        {
          "t": 2.0,
          "v": 18540.0
        },
        {
          "t": 4.0,
          "v": 64614.5
        },
        {
          "t": 6.0,
          "v": 9308.5
        },
        {
          "t": 8.0,
          "v": 58772.0
        },
        {
          "t": 10.0,
          "v": 36235.5
        },
        {
          "t": 12.0,
          "v": 27973.5
        },
        {
          "t": 14.0,
          "v": 40013.5
        },
        {
          "t": 16.0,
          "v": 42965.0
        },
        {
          "t": 18.0,
          "v": 21554.5
        },
        {
          "t": 20.0,
          "v": 47140.5
        },
        {
          "t": 22.0,
          "v": 57036.5
        },
        {
          "t": 24.0,
          "v": 47856.0
        },
        {
          "t": 26.0,
          "v": 51631.5
        },
        {
          "t": 28.0,
          "v": 36774.5
        },
        {
          "t": 30.0,
          "v": 61975.5
        },
        {
          "t": 32.0,
          "v": 5450.5
        },
        {
          "t": 34.0,
          "v": 69635.5
        },
        {
          "t": 36.0,
          "v": 25282.5
        },
        {
          "t": 38.0,
          "v": 44892.5
        },
        {
          "t": 40.0,
          "v": 53039.0
        },
        {
          "t": 42.0,
          "v": 15134.0
        },
        {
          "t": 44.0,
          "v": 69301.0
        },
        {
          "t": 46.0,
          "v": 16697.5
        },
        {
          "t": 48.0,
          "v": 93705.5
        },
        {
          "t": 50.0,
          "v": 4379.5
        },
        {
          "t": 52.0,
          "v": 52000.5
        },
        {
          "t": 54.0,
          "v": 36054.0
        },
        {
          "t": 56.0,
          "v": 16675.5
        },
        {
          "t": 58.0,
          "v": 18383.5
        },
        {
          "t": 60.0,
          "v": 35422.5
        },
        {
          "t": 62.0,
          "v": 33264.5
        },
        {
          "t": 64.0,
          "v": 35643.5
        },
        {
          "t": 66.0,
          "v": 7869.5
        },
        {
          "t": 68.0,
          "v": 38014.5
        },
        {
          "t": 70.0,
          "v": 34462.19
        },
        {
          "t": 72.0,
          "v": 132154.5
        },
        {
          "t": 74.0,
          "v": 59640.5
        },
        {
          "t": 76.0,
          "v": 37775.5
        },
        {
          "t": 78.0,
          "v": 78000.0
        },
        {
          "t": 80.0,
          "v": 19838.0
        },
        {
          "t": 82.0,
          "v": 94050.5
        },
        {
          "t": 84.0,
          "v": 3944.0
        },
        {
          "t": 86.0,
          "v": 98813.5
        },
        {
          "t": 88.0,
          "v": 11553.0
        },
        {
          "t": 90.0,
          "v": 85986.0
        },
        {
          "t": 92.0,
          "v": 26091.5
        },
        {
          "t": 94.0,
          "v": 74188.5
        },
        {
          "t": 96.0,
          "v": 39124.5
        },
        {
          "t": 98.0,
          "v": 60041.5
        },
        {
          "t": 100.0,
          "v": 55117.5
        },
        {
          "t": 102.0,
          "v": 43732.0
        },
        {
          "t": 104.0,
          "v": 74990.5
        },
        {
          "t": 106.0,
          "v": 23594.0
        },
        {
          "t": 108.0,
          "v": 95771.0
        },
        {
          "t": 110.0,
          "v": 4247.5
        },
        {
          "t": 112.0,
          "v": 100615.0
        },
        {
          "t": 114.0,
          "v": 16235.0
        },
        {
          "t": 116.0,
          "v": 86443.0
        },
        {
          "t": 118.0,
          "v": 29303.0
        },
        {
          "t": 120.0,
          "v": 74719.5
        },
        {
          "t": 122.0,
          "v": 41587.5
        },
        {
          "t": 124.0,
          "v": 61144.5
        },
        {
          "t": 126.0,
          "v": 53821.0
        },
        {
          "t": 128.0,
          "v": 48931.0
        },
        {
          "t": 130.0,
          "v": 65570.5
        },
        {
          "t": 132.0,
          "v": 34586.5
        },
        {
          "t": 134.0,
          "v": 85955.5
        },
        {
          "t": 136.0,
          "v": 14380.5
        },
        {
          "t": 138.0,
          "v": 77386.5
        },
        {
          "t": 140.0,
          "v": 23795.0
        },
        {
          "t": 142.0,
          "v": 52559.0
        },
        {
          "t": 144.0,
          "v": 48209.5
        },
        {
          "t": 146.0,
          "v": 32689.0
        },
        {
          "t": 148.0,
          "v": 68298.0
        },
        {
          "t": 150.0,
          "v": 7997.5
        },
        {
          "t": 152.0,
          "v": 77652.0
        },
        {
          "t": 154.0,
          "v": 15913.0
        },
        {
          "t": 156.0,
          "v": 62067.5
        },
        {
          "t": 158.0,
          "v": 38763.5
        },
        {
          "t": 160.0,
          "v": 70138.5
        },
        {
          "t": 162.0,
          "v": 32522.5
        },
        {
          "t": 164.0,
          "v": 55512.0
        },
        {
          "t": 166.0,
          "v": 45065.5
        },
        {
          "t": 168.0,
          "v": 32760.0
        },
        {
          "t": 170.0,
          "v": 68562.0
        },
        {
          "t": 172.0,
          "v": 8325.5
        },
        {
          "t": 174.0,
          "v": 77680.0
        },
        {
          "t": 176.0,
          "v": 15520.5
        },
        {
          "t": 178.0,
          "v": 60618.5
        },
        {
          "t": 180.0,
          "v": 38180.0
        },
        {
          "t": 182.0,
          "v": 48952.5
        },
        {
          "t": 184.0,
          "v": 52477.0
        },
        {
          "t": 186.0,
          "v": 71024.5
        },
        {
          "t": 188.0,
          "v": 30493.5
        },
        {
          "t": 190.0,
          "v": 55780.6
        },
        {
          "t": 192.0,
          "v": 44877.5
        },
        {
          "t": 194.0,
          "v": 34086.0
        },
        {
          "t": 196.0,
          "v": 65074.5
        },
        {
          "t": 198.0,
          "v": 9410.0
        },
        {
          "t": 200.0,
          "v": 76040.0
        },
        {
          "t": 202.0,
          "v": 14678.5
        },
        {
          "t": 204.0,
          "v": 60964.0
        },
        {
          "t": 206.0,
          "v": 38649.0
        },
        {
          "t": 208.0,
          "v": 60558.5
        },
        {
          "t": 210.0,
          "v": 39987.5
        },
        {
          "t": 212.0,
          "v": 72918.0
        },
        {
          "t": 214.0,
          "v": 26772.5
        },
        {
          "t": 216.0,
          "v": 48569.5
        },
        {
          "t": 218.0,
          "v": 51001.0
        },
        {
          "t": 220.0,
          "v": 24237.5
        },
        {
          "t": 222.0,
          "v": 76654.5
        },
        {
          "t": 224.0,
          "v": 3014.5
        },
        {
          "t": 226.0,
          "v": 78088.0
        },
        {
          "t": 228.0,
          "v": 20972.0
        },
        {
          "t": 230.0,
          "v": 55506.5
        },
        {
          "t": 232.0,
          "v": 119067.5
        }
      ],
      "minflt": [
        {
          "t": 0.0,
          "v": 0.5
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
        },
        {
          "t": 114.0,
          "v": 0.0
        },
        {
          "t": 116.0,
          "v": 0.0
        },
        {
          "t": 118.0,
          "v": 0.0
        },
        {
          "t": 120.0,
          "v": 0.0
        },
        {
          "t": 122.0,
          "v": 0.0
        },
        {
          "t": 124.0,
          "v": 0.0
        },
        {
          "t": 126.0,
          "v": 0.0
        },
        {
          "t": 128.0,
          "v": 0.0
        },
        {
          "t": 130.0,
          "v": 0.0
        },
        {
          "t": 132.0,
          "v": 0.0
        },
        {
          "t": 134.0,
          "v": 0.0
        },
        {
          "t": 136.0,
          "v": 0.0
        },
        {
          "t": 138.0,
          "v": 0.0
        },
        {
          "t": 140.0,
          "v": 0.0
        },
        {
          "t": 142.0,
          "v": 0.0
        },
        {
          "t": 144.0,
          "v": 0.0
        },
        {
          "t": 146.0,
          "v": 0.0
        },
        {
          "t": 148.0,
          "v": 0.0
        },
        {
          "t": 150.0,
          "v": 0.0
        },
        {
          "t": 152.0,
          "v": 0.0
        },
        {
          "t": 154.0,
          "v": 0.0
        },
        {
          "t": 156.0,
          "v": 0.0
        },
        {
          "t": 158.0,
          "v": 0.0
        },
        {
          "t": 160.0,
          "v": 0.0
        },
        {
          "t": 162.0,
          "v": 0.0
        },
        {
          "t": 164.0,
          "v": 0.0
        },
        {
          "t": 166.0,
          "v": 0.0
        },
        {
          "t": 168.0,
          "v": 0.0
        },
        {
          "t": 170.0,
          "v": 0.0
        },
        {
          "t": 172.0,
          "v": 0.0
        },
        {
          "t": 174.0,
          "v": 0.0
        },
        {
          "t": 176.0,
          "v": 0.0
        },
        {
          "t": 178.0,
          "v": 0.0
        },
        {
          "t": 180.0,
          "v": 0.0
        },
        {
          "t": 182.0,
          "v": 0.0
        },
        {
          "t": 184.0,
          "v": 0.0
        },
        {
          "t": 186.0,
          "v": 0.0
        },
        {
          "t": 188.0,
          "v": 0.0
        },
        {
          "t": 190.0,
          "v": 0.0
        },
        {
          "t": 192.0,
          "v": 0.0
        },
        {
          "t": 194.0,
          "v": 0.0
        },
        {
          "t": 196.0,
          "v": 0.0
        },
        {
          "t": 198.0,
          "v": 0.0
        },
        {
          "t": 200.0,
          "v": 0.0
        },
        {
          "t": 202.0,
          "v": 0.0
        },
        {
          "t": 204.0,
          "v": 0.0
        },
        {
          "t": 206.0,
          "v": 0.0
        },
        {
          "t": 208.0,
          "v": 0.0
        },
        {
          "t": 210.0,
          "v": 0.0
        },
        {
          "t": 212.0,
          "v": 0.0
        },
        {
          "t": 214.0,
          "v": 0.0
        },
        {
          "t": 216.0,
          "v": 0.0
        },
        {
          "t": 218.0,
          "v": 0.0
        },
        {
          "t": 220.0,
          "v": 0.0
        },
        {
          "t": 222.0,
          "v": 0.0
        },
        {
          "t": 224.0,
          "v": 0.0
        },
        {
          "t": 226.0,
          "v": 0.0
        },
        {
          "t": 228.0,
          "v": 0.0
        },
        {
          "t": 230.0,
          "v": 0.0
        },
        {
          "t": 232.0,
          "v": 0.0
        }
      ],
      "pgfree": [
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
        },
        {
          "t": 114.0,
          "v": 0.0
        },
        {
          "t": 116.0,
          "v": 0.0
        },
        {
          "t": 118.0,
          "v": 0.0
        },
        {
          "t": 120.0,
          "v": 0.0
        },
        {
          "t": 122.0,
          "v": 0.0
        },
        {
          "t": 124.0,
          "v": 0.0
        },
        {
          "t": 126.0,
          "v": 0.0
        },
        {
          "t": 128.0,
          "v": 0.0
        },
        {
          "t": 130.0,
          "v": 0.0
        },
        {
          "t": 132.0,
          "v": 0.0
        },
        {
          "t": 134.0,
          "v": 0.0
        },
        {
          "t": 136.0,
          "v": 0.0
        },
        {
          "t": 138.0,
          "v": 0.0
        },
        {
          "t": 140.0,
          "v": 0.0
        },
        {
          "t": 142.0,
          "v": 0.0
        },
        {
          "t": 144.0,
          "v": 0.0
        },
        {
          "t": 146.0,
          "v": 0.0
        },
        {
          "t": 148.0,
          "v": 0.0
        },
        {
          "t": 150.0,
          "v": 0.0
        },
        {
          "t": 152.0,
          "v": 0.0
        },
        {
          "t": 154.0,
          "v": 0.0
        },
        {
          "t": 156.0,
          "v": 0.0
        },
        {
          "t": 158.0,
          "v": 0.0
        },
        {
          "t": 160.0,
          "v": 0.0
        },
        {
          "t": 162.0,
          "v": 0.0
        },
        {
          "t": 164.0,
          "v": 0.0
        },
        {
          "t": 166.0,
          "v": 0.0
        },
        {
          "t": 168.0,
          "v": 0.0
        },
        {
          "t": 170.0,
          "v": 0.0
        },
        {
          "t": 172.0,
          "v": 0.0
        },
        {
          "t": 174.0,
          "v": 0.0
        },
        {
          "t": 176.0,
          "v": 0.0
        },
        {
          "t": 178.0,
          "v": 0.0
        },
        {
          "t": 180.0,
          "v": 0.0
        },
        {
          "t": 182.0,
          "v": 0.0
        },
        {
          "t": 184.0,
          "v": 0.0
        },
        {
          "t": 186.0,
          "v": 0.0
        },
        {
          "t": 188.0,
          "v": 0.0
        },
        {
          "t": 190.0,
          "v": 0.0
        },
        {
          "t": 192.0,
          "v": 0.0
        },
        {
          "t": 194.0,
          "v": 0.0
        },
        {
          "t": 196.0,
          "v": 0.0
        },
        {
          "t": 198.0,
          "v": 0.0
        },
        {
          "t": 200.0,
          "v": 0.0
        },
        {
          "t": 202.0,
          "v": 0.0
        },
        {
          "t": 204.0,
          "v": 0.0
        },
        {
          "t": 206.0,
          "v": 0.0
        },
        {
          "t": 208.0,
          "v": 0.0
        },
        {
          "t": 210.0,
          "v": 0.0
        },
        {
          "t": 212.0,
          "v": 0.0
        },
        {
          "t": 214.0,
          "v": 0.0
        },
        {
          "t": 216.0,
          "v": 0.0
        },
        {
          "t": 218.0,
          "v": 0.0
        },
        {
          "t": 220.0,
          "v": 0.0
        },
        {
          "t": 222.0,
          "v": 0.0
        },
        {
          "t": 224.0,
          "v": 0.0
        },
        {
          "t": 226.0,
          "v": 0.0
        },
        {
          "t": 228.0,
          "v": 0.0
        },
        {
          "t": 230.0,
          "v": 0.0
        },
        {
          "t": 232.0,
          "v": 0.0
        }
      ]
    },
    "tcp_conns": {
      "estab": [
        {
          "t": 0,
          "v": 17
        },
        {
          "t": 2,
          "v": 19
        },
        {
          "t": 4,
          "v": 150
        },
        {
          "t": 6,
          "v": 164
        },
        {
          "t": 8,
          "v": 176
        },
        {
          "t": 10,
          "v": 196
        },
        {
          "t": 12,
          "v": 239
        },
        {
          "t": 14,
          "v": 417
        },
        {
          "t": 16,
          "v": 604
        },
        {
          "t": 18,
          "v": 659
        },
        {
          "t": 20,
          "v": 515
        },
        {
          "t": 22,
          "v": 521
        },
        {
          "t": 24,
          "v": 515
        },
        {
          "t": 26,
          "v": 515
        },
        {
          "t": 28,
          "v": 727
        },
        {
          "t": 30,
          "v": 727
        },
        {
          "t": 32,
          "v": 727
        },
        {
          "t": 34,
          "v": 727
        },
        {
          "t": 36,
          "v": 731
        },
        {
          "t": 38,
          "v": 731
        },
        {
          "t": 40,
          "v": 731
        },
        {
          "t": 42,
          "v": 731
        },
        {
          "t": 44,
          "v": 731
        },
        {
          "t": 46,
          "v": 731
        },
        {
          "t": 48,
          "v": 731
        },
        {
          "t": 50,
          "v": 731
        },
        {
          "t": 52,
          "v": 731
        },
        {
          "t": 54,
          "v": 1465
        },
        {
          "t": 56,
          "v": 1792
        },
        {
          "t": 58,
          "v": 1960
        },
        {
          "t": 60,
          "v": 2349
        },
        {
          "t": 62,
          "v": 3037
        },
        {
          "t": 64,
          "v": 2420
        },
        {
          "t": 66,
          "v": 2242
        },
        {
          "t": 68,
          "v": 2184
        },
        {
          "t": 70,
          "v": 2144
        },
        {
          "t": 73,
          "v": 2170
        },
        {
          "t": 75,
          "v": 25
        },
        {
          "t": 77,
          "v": 35
        },
        {
          "t": 79,
          "v": 49
        },
        {
          "t": 81,
          "v": 65
        },
        {
          "t": 83,
          "v": 93
        },
        {
          "t": 85,
          "v": 93
        },
        {
          "t": 87,
          "v": 93
        },
        {
          "t": 89,
          "v": 93
        },
        {
          "t": 91,
          "v": 93
        },
        {
          "t": 93,
          "v": 93
        },
        {
          "t": 95,
          "v": 93
        },
        {
          "t": 97,
          "v": 93
        },
        {
          "t": 99,
          "v": 93
        },
        {
          "t": 101,
          "v": 93
        },
        {
          "t": 103,
          "v": 93
        },
        {
          "t": 105,
          "v": 93
        },
        {
          "t": 107,
          "v": 93
        },
        {
          "t": 109,
          "v": 92
        },
        {
          "t": 111,
          "v": 124
        },
        {
          "t": 113,
          "v": 140
        },
        {
          "t": 115,
          "v": 150
        },
        {
          "t": 117,
          "v": 150
        },
        {
          "t": 119,
          "v": 150
        },
        {
          "t": 121,
          "v": 150
        },
        {
          "t": 123,
          "v": 150
        },
        {
          "t": 125,
          "v": 150
        },
        {
          "t": 127,
          "v": 150
        },
        {
          "t": 129,
          "v": 150
        },
        {
          "t": 131,
          "v": 150
        },
        {
          "t": 133,
          "v": 150
        },
        {
          "t": 135,
          "v": 150
        },
        {
          "t": 137,
          "v": 149
        },
        {
          "t": 139,
          "v": 148
        },
        {
          "t": 141,
          "v": 462
        },
        {
          "t": 143,
          "v": 462
        },
        {
          "t": 145,
          "v": 462
        },
        {
          "t": 147,
          "v": 462
        },
        {
          "t": 149,
          "v": 462
        },
        {
          "t": 151,
          "v": 462
        },
        {
          "t": 153,
          "v": 462
        },
        {
          "t": 155,
          "v": 462
        },
        {
          "t": 157,
          "v": 462
        },
        {
          "t": 159,
          "v": 462
        },
        {
          "t": 161,
          "v": 462
        },
        {
          "t": 163,
          "v": 442
        },
        {
          "t": 165,
          "v": 747
        },
        {
          "t": 167,
          "v": 747
        },
        {
          "t": 169,
          "v": 747
        },
        {
          "t": 171,
          "v": 747
        },
        {
          "t": 173,
          "v": 747
        },
        {
          "t": 175,
          "v": 747
        },
        {
          "t": 177,
          "v": 747
        },
        {
          "t": 179,
          "v": 747
        },
        {
          "t": 181,
          "v": 747
        },
        {
          "t": 183,
          "v": 747
        },
        {
          "t": 185,
          "v": 747
        },
        {
          "t": 187,
          "v": 747
        },
        {
          "t": 189,
          "v": 1147
        },
        {
          "t": 191,
          "v": 1137
        },
        {
          "t": 193,
          "v": 1137
        },
        {
          "t": 195,
          "v": 1137
        },
        {
          "t": 197,
          "v": 1137
        },
        {
          "t": 199,
          "v": 1137
        },
        {
          "t": 201,
          "v": 1137
        },
        {
          "t": 203,
          "v": 1137
        },
        {
          "t": 205,
          "v": 1137
        },
        {
          "t": 207,
          "v": 1137
        },
        {
          "t": 209,
          "v": 1137
        },
        {
          "t": 211,
          "v": 1137
        },
        {
          "t": 213,
          "v": 1137
        },
        {
          "t": 215,
          "v": 1237
        },
        {
          "t": 217,
          "v": 1237
        },
        {
          "t": 219,
          "v": 1237
        },
        {
          "t": 221,
          "v": 1237
        },
        {
          "t": 223,
          "v": 1237
        },
        {
          "t": 225,
          "v": 1237
        },
        {
          "t": 227,
          "v": 1237
        },
        {
          "t": 229,
          "v": 1237
        },
        {
          "t": 231,
          "v": 1237
        },
        {
          "t": 233,
          "v": 1237
        }
      ],
      "timewait": [
        {
          "t": 0,
          "v": 18
        },
        {
          "t": 2,
          "v": 19
        },
        {
          "t": 4,
          "v": 406
        },
        {
          "t": 6,
          "v": 1347
        },
        {
          "t": 8,
          "v": 1894
        },
        {
          "t": 10,
          "v": 2820
        },
        {
          "t": 12,
          "v": 3555
        },
        {
          "t": 14,
          "v": 5730
        },
        {
          "t": 16,
          "v": 9188
        },
        {
          "t": 18,
          "v": 12146
        },
        {
          "t": 20,
          "v": 13472
        },
        {
          "t": 22,
          "v": 14319
        },
        {
          "t": 24,
          "v": 14821
        },
        {
          "t": 26,
          "v": 14821
        },
        {
          "t": 28,
          "v": 14809
        },
        {
          "t": 30,
          "v": 14809
        },
        {
          "t": 32,
          "v": 14809
        },
        {
          "t": 34,
          "v": 14809
        },
        {
          "t": 36,
          "v": 14809
        },
        {
          "t": 38,
          "v": 14809
        },
        {
          "t": 40,
          "v": 14809
        },
        {
          "t": 42,
          "v": 14809
        },
        {
          "t": 44,
          "v": 14809
        },
        {
          "t": 46,
          "v": 14809
        },
        {
          "t": 48,
          "v": 14809
        },
        {
          "t": 50,
          "v": 14809
        },
        {
          "t": 52,
          "v": 14808
        },
        {
          "t": 54,
          "v": 14762
        },
        {
          "t": 56,
          "v": 15105
        },
        {
          "t": 58,
          "v": 15888
        },
        {
          "t": 60,
          "v": 16655
        },
        {
          "t": 62,
          "v": 17641
        },
        {
          "t": 64,
          "v": 18987
        },
        {
          "t": 66,
          "v": 19868
        },
        {
          "t": 68,
          "v": 20237
        },
        {
          "t": 70,
          "v": 20856
        },
        {
          "t": 73,
          "v": 21069
        },
        {
          "t": 75,
          "v": 22185
        },
        {
          "t": 77,
          "v": 20773
        },
        {
          "t": 79,
          "v": 20814
        },
        {
          "t": 81,
          "v": 16999
        },
        {
          "t": 83,
          "v": 17038
        },
        {
          "t": 85,
          "v": 15514
        },
        {
          "t": 87,
          "v": 15557
        },
        {
          "t": 89,
          "v": 15600
        },
        {
          "t": 91,
          "v": 15642
        },
        {
          "t": 93,
          "v": 15684
        },
        {
          "t": 95,
          "v": 15726
        },
        {
          "t": 97,
          "v": 15767
        },
        {
          "t": 99,
          "v": 15769
        },
        {
          "t": 101,
          "v": 15769
        },
        {
          "t": 103,
          "v": 15769
        },
        {
          "t": 105,
          "v": 15769
        },
        {
          "t": 107,
          "v": 15769
        },
        {
          "t": 109,
          "v": 15769
        },
        {
          "t": 111,
          "v": 15778
        },
        {
          "t": 113,
          "v": 15798
        },
        {
          "t": 115,
          "v": 15819
        },
        {
          "t": 117,
          "v": 15569
        },
        {
          "t": 119,
          "v": 15589
        },
        {
          "t": 121,
          "v": 13947
        },
        {
          "t": 123,
          "v": 13969
        },
        {
          "t": 125,
          "v": 10239
        },
        {
          "t": 127,
          "v": 10259
        },
        {
          "t": 129,
          "v": 6528
        },
        {
          "t": 131,
          "v": 6539
        },
        {
          "t": 133,
          "v": 1675
        },
        {
          "t": 135,
          "v": 1675
        },
        {
          "t": 137,
          "v": 623
        },
        {
          "t": 139,
          "v": 624
        },
        {
          "t": 141,
          "v": 663
        },
        {
          "t": 143,
          "v": 578
        },
        {
          "t": 145,
          "v": 596
        },
        {
          "t": 147,
          "v": 530
        },
        {
          "t": 149,
          "v": 552
        },
        {
          "t": 151,
          "v": 464
        },
        {
          "t": 153,
          "v": 466
        },
        {
          "t": 155,
          "v": 405
        },
        {
          "t": 157,
          "v": 411
        },
        {
          "t": 159,
          "v": 396
        },
        {
          "t": 161,
          "v": 396
        },
        {
          "t": 163,
          "v": 406
        },
        {
          "t": 165,
          "v": 419
        },
        {
          "t": 167,
          "v": 449
        },
        {
          "t": 169,
          "v": 471
        },
        {
          "t": 171,
          "v": 518
        },
        {
          "t": 173,
          "v": 551
        },
        {
          "t": 175,
          "v": 513
        },
        {
          "t": 177,
          "v": 557
        },
        {
          "t": 179,
          "v": 527
        },
        {
          "t": 181,
          "v": 561
        },
        {
          "t": 183,
          "v": 532
        },
        {
          "t": 185,
          "v": 532
        },
        {
          "t": 187,
          "v": 489
        },
        {
          "t": 189,
          "v": 532
        },
        {
          "t": 191,
          "v": 504
        },
        {
          "t": 193,
          "v": 539
        },
        {
          "t": 195,
          "v": 561
        },
        {
          "t": 197,
          "v": 613
        },
        {
          "t": 199,
          "v": 622
        },
        {
          "t": 201,
          "v": 624
        },
        {
          "t": 203,
          "v": 647
        },
        {
          "t": 205,
          "v": 653
        },
        {
          "t": 207,
          "v": 628
        },
        {
          "t": 209,
          "v": 630
        },
        {
          "t": 211,
          "v": 608
        },
        {
          "t": 213,
          "v": 608
        },
        {
          "t": 215,
          "v": 590
        },
        {
          "t": 217,
          "v": 598
        },
        {
          "t": 219,
          "v": 575
        },
        {
          "t": 221,
          "v": 588
        },
        {
          "t": 223,
          "v": 606
        },
        {
          "t": 225,
          "v": 620
        },
        {
          "t": 227,
          "v": 661
        },
        {
          "t": 229,
          "v": 668
        },
        {
          "t": 231,
          "v": 608
        },
        {
          "t": 233,
          "v": 653
        }
      ],
      "closed": [
        {
          "t": 0,
          "v": 18
        },
        {
          "t": 2,
          "v": 19
        },
        {
          "t": 4,
          "v": 406
        },
        {
          "t": 6,
          "v": 1347
        },
        {
          "t": 8,
          "v": 1894
        },
        {
          "t": 10,
          "v": 2820
        },
        {
          "t": 12,
          "v": 3555
        },
        {
          "t": 14,
          "v": 5730
        },
        {
          "t": 16,
          "v": 9188
        },
        {
          "t": 18,
          "v": 12146
        },
        {
          "t": 20,
          "v": 13472
        },
        {
          "t": 22,
          "v": 14320
        },
        {
          "t": 24,
          "v": 14821
        },
        {
          "t": 26,
          "v": 14821
        },
        {
          "t": 28,
          "v": 14809
        },
        {
          "t": 30,
          "v": 14809
        },
        {
          "t": 32,
          "v": 14809
        },
        {
          "t": 34,
          "v": 14809
        },
        {
          "t": 36,
          "v": 14809
        },
        {
          "t": 38,
          "v": 14809
        },
        {
          "t": 40,
          "v": 14809
        },
        {
          "t": 42,
          "v": 14809
        },
        {
          "t": 44,
          "v": 14809
        },
        {
          "t": 46,
          "v": 14809
        },
        {
          "t": 48,
          "v": 14809
        },
        {
          "t": 50,
          "v": 14809
        },
        {
          "t": 52,
          "v": 14808
        },
        {
          "t": 54,
          "v": 14762
        },
        {
          "t": 56,
          "v": 15105
        },
        {
          "t": 58,
          "v": 15888
        },
        {
          "t": 60,
          "v": 16655
        },
        {
          "t": 62,
          "v": 17641
        },
        {
          "t": 64,
          "v": 18987
        },
        {
          "t": 66,
          "v": 19868
        },
        {
          "t": 68,
          "v": 20237
        },
        {
          "t": 70,
          "v": 20856
        },
        {
          "t": 73,
          "v": 21069
        },
        {
          "t": 75,
          "v": 22185
        },
        {
          "t": 77,
          "v": 20773
        },
        {
          "t": 79,
          "v": 20814
        },
        {
          "t": 81,
          "v": 16999
        },
        {
          "t": 83,
          "v": 17038
        },
        {
          "t": 85,
          "v": 15514
        },
        {
          "t": 87,
          "v": 15557
        },
        {
          "t": 89,
          "v": 15600
        },
        {
          "t": 91,
          "v": 15642
        },
        {
          "t": 93,
          "v": 15684
        },
        {
          "t": 95,
          "v": 15726
        },
        {
          "t": 97,
          "v": 15767
        },
        {
          "t": 99,
          "v": 15769
        },
        {
          "t": 101,
          "v": 15769
        },
        {
          "t": 103,
          "v": 15769
        },
        {
          "t": 105,
          "v": 15769
        },
        {
          "t": 107,
          "v": 15769
        },
        {
          "t": 109,
          "v": 15769
        },
        {
          "t": 111,
          "v": 15778
        },
        {
          "t": 113,
          "v": 15798
        },
        {
          "t": 115,
          "v": 15819
        },
        {
          "t": 117,
          "v": 15569
        },
        {
          "t": 119,
          "v": 15589
        },
        {
          "t": 121,
          "v": 13947
        },
        {
          "t": 123,
          "v": 13969
        },
        {
          "t": 125,
          "v": 10239
        },
        {
          "t": 127,
          "v": 10259
        },
        {
          "t": 129,
          "v": 6528
        },
        {
          "t": 131,
          "v": 6539
        },
        {
          "t": 133,
          "v": 1675
        },
        {
          "t": 135,
          "v": 1675
        },
        {
          "t": 137,
          "v": 623
        },
        {
          "t": 139,
          "v": 624
        },
        {
          "t": 141,
          "v": 663
        },
        {
          "t": 143,
          "v": 578
        },
        {
          "t": 145,
          "v": 596
        },
        {
          "t": 147,
          "v": 530
        },
        {
          "t": 149,
          "v": 552
        },
        {
          "t": 151,
          "v": 464
        },
        {
          "t": 153,
          "v": 466
        },
        {
          "t": 155,
          "v": 405
        },
        {
          "t": 157,
          "v": 411
        },
        {
          "t": 159,
          "v": 396
        },
        {
          "t": 161,
          "v": 396
        },
        {
          "t": 163,
          "v": 406
        },
        {
          "t": 165,
          "v": 419
        },
        {
          "t": 167,
          "v": 449
        },
        {
          "t": 169,
          "v": 471
        },
        {
          "t": 171,
          "v": 518
        },
        {
          "t": 173,
          "v": 551
        },
        {
          "t": 175,
          "v": 513
        },
        {
          "t": 177,
          "v": 557
        },
        {
          "t": 179,
          "v": 527
        },
        {
          "t": 181,
          "v": 561
        },
        {
          "t": 183,
          "v": 532
        },
        {
          "t": 185,
          "v": 532
        },
        {
          "t": 187,
          "v": 489
        },
        {
          "t": 189,
          "v": 532
        },
        {
          "t": 191,
          "v": 504
        },
        {
          "t": 193,
          "v": 539
        },
        {
          "t": 195,
          "v": 561
        },
        {
          "t": 197,
          "v": 613
        },
        {
          "t": 199,
          "v": 622
        },
        {
          "t": 201,
          "v": 624
        },
        {
          "t": 203,
          "v": 647
        },
        {
          "t": 205,
          "v": 653
        },
        {
          "t": 207,
          "v": 628
        },
        {
          "t": 209,
          "v": 630
        },
        {
          "t": 211,
          "v": 608
        },
        {
          "t": 213,
          "v": 608
        },
        {
          "t": 215,
          "v": 590
        },
        {
          "t": 217,
          "v": 598
        },
        {
          "t": 219,
          "v": 575
        },
        {
          "t": 221,
          "v": 588
        },
        {
          "t": 223,
          "v": 606
        },
        {
          "t": 225,
          "v": 620
        },
        {
          "t": 227,
          "v": 661
        },
        {
          "t": 229,
          "v": 668
        },
        {
          "t": 231,
          "v": 608
        },
        {
          "t": 233,
          "v": 653
        }
      ]
    },
    "psi": {
      "cpu": [
        {
          "t": 0,
          "v": 0.0
        },
        {
          "t": 2,
          "v": 0.0
        },
        {
          "t": 4,
          "v": 0.0
        },
        {
          "t": 6,
          "v": 0.0
        },
        {
          "t": 8,
          "v": 0.0
        },
        {
          "t": 10,
          "v": 0.0
        },
        {
          "t": 12,
          "v": 0.0
        },
        {
          "t": 14,
          "v": 0.0
        },
        {
          "t": 16,
          "v": 0.0
        },
        {
          "t": 18,
          "v": 0.0
        },
        {
          "t": 20,
          "v": 0.0
        },
        {
          "t": 22,
          "v": 0.0
        },
        {
          "t": 24,
          "v": 0.0
        },
        {
          "t": 26,
          "v": 0.0
        },
        {
          "t": 28,
          "v": 0.0
        },
        {
          "t": 30,
          "v": 0.0
        },
        {
          "t": 32,
          "v": 0.0
        },
        {
          "t": 35,
          "v": 0.0
        },
        {
          "t": 37,
          "v": 0.0
        },
        {
          "t": 39,
          "v": 0.0
        },
        {
          "t": 41,
          "v": 0.0
        },
        {
          "t": 43,
          "v": 0.0
        },
        {
          "t": 45,
          "v": 0.0
        },
        {
          "t": 47,
          "v": 0.0
        },
        {
          "t": 49,
          "v": 0.0
        },
        {
          "t": 51,
          "v": 0.0
        },
        {
          "t": 53,
          "v": 0.0
        },
        {
          "t": 55,
          "v": 0.0
        },
        {
          "t": 57,
          "v": 0.0
        },
        {
          "t": 59,
          "v": 0.0
        },
        {
          "t": 61,
          "v": 0.0
        },
        {
          "t": 63,
          "v": 0.0
        },
        {
          "t": 65,
          "v": 0.0
        },
        {
          "t": 67,
          "v": 0.0
        },
        {
          "t": 69,
          "v": 0.0
        },
        {
          "t": 71,
          "v": 0.0
        },
        {
          "t": 73,
          "v": 0.0
        },
        {
          "t": 75,
          "v": 0.0
        },
        {
          "t": 77,
          "v": 0.0
        },
        {
          "t": 79,
          "v": 0.0
        },
        {
          "t": 81,
          "v": 0.0
        },
        {
          "t": 83,
          "v": 0.0
        },
        {
          "t": 85,
          "v": 0.0
        },
        {
          "t": 87,
          "v": 0.0
        },
        {
          "t": 89,
          "v": 0.0
        },
        {
          "t": 91,
          "v": 0.0
        },
        {
          "t": 93,
          "v": 0.0
        },
        {
          "t": 95,
          "v": 0.0
        },
        {
          "t": 97,
          "v": 0.0
        },
        {
          "t": 99,
          "v": 0.0
        },
        {
          "t": 101,
          "v": 0.0
        },
        {
          "t": 103,
          "v": 0.0
        },
        {
          "t": 105,
          "v": 0.0
        },
        {
          "t": 107,
          "v": 0.0
        },
        {
          "t": 109,
          "v": 0.0
        },
        {
          "t": 111,
          "v": 0.0
        },
        {
          "t": 113,
          "v": 0.0
        },
        {
          "t": 115,
          "v": 0.0
        },
        {
          "t": 117,
          "v": 0.0
        },
        {
          "t": 119,
          "v": 0.0
        },
        {
          "t": 121,
          "v": 0.0
        },
        {
          "t": 123,
          "v": 0.0
        },
        {
          "t": 125,
          "v": 0.0
        },
        {
          "t": 127,
          "v": 0.0
        },
        {
          "t": 129,
          "v": 0.0
        },
        {
          "t": 131,
          "v": 0.0
        },
        {
          "t": 133,
          "v": 0.0
        },
        {
          "t": 135,
          "v": 0.0
        },
        {
          "t": 137,
          "v": 0.0
        },
        {
          "t": 139,
          "v": 0.0
        },
        {
          "t": 141,
          "v": 0.0
        },
        {
          "t": 143,
          "v": 0.0
        },
        {
          "t": 145,
          "v": 0.0
        },
        {
          "t": 148,
          "v": 0.0
        },
        {
          "t": 150,
          "v": 0.0
        },
        {
          "t": 152,
          "v": 0.0
        },
        {
          "t": 154,
          "v": 0.0
        },
        {
          "t": 156,
          "v": 0.0
        },
        {
          "t": 158,
          "v": 0.0
        },
        {
          "t": 160,
          "v": 0.0
        },
        {
          "t": 162,
          "v": 0.0
        },
        {
          "t": 164,
          "v": 0.0
        },
        {
          "t": 166,
          "v": 0.0
        },
        {
          "t": 168,
          "v": 0.0
        },
        {
          "t": 170,
          "v": 0.0
        },
        {
          "t": 172,
          "v": 0.0
        },
        {
          "t": 174,
          "v": 0.0
        },
        {
          "t": 176,
          "v": 0.0
        },
        {
          "t": 178,
          "v": 0.0
        },
        {
          "t": 180,
          "v": 0.0
        },
        {
          "t": 182,
          "v": 0.0
        },
        {
          "t": 184,
          "v": 0.0
        },
        {
          "t": 186,
          "v": 0.0
        },
        {
          "t": 188,
          "v": 0.0
        },
        {
          "t": 190,
          "v": 0.0
        },
        {
          "t": 192,
          "v": 0.0
        },
        {
          "t": 194,
          "v": 0.0
        },
        {
          "t": 196,
          "v": 0.0
        },
        {
          "t": 198,
          "v": 0.0
        },
        {
          "t": 200,
          "v": 0.0
        },
        {
          "t": 202,
          "v": 0.0
        },
        {
          "t": 204,
          "v": 0.0
        },
        {
          "t": 206,
          "v": 0.0
        },
        {
          "t": 208,
          "v": 0.0
        },
        {
          "t": 210,
          "v": 0.0
        },
        {
          "t": 212,
          "v": 0.0
        },
        {
          "t": 214,
          "v": 0.0
        },
        {
          "t": 216,
          "v": 0.0
        },
        {
          "t": 218,
          "v": 0.0
        },
        {
          "t": 220,
          "v": 0.0
        },
        {
          "t": 222,
          "v": 0.0
        },
        {
          "t": 224,
          "v": 0.0
        },
        {
          "t": 226,
          "v": 0.0
        },
        {
          "t": 228,
          "v": 0.0
        },
        {
          "t": 230,
          "v": 0.0
        },
        {
          "t": 232,
          "v": 0.0
        },
        {
          "t": 234,
          "v": 0.0
        }
      ],
      "mem": [
        {
          "t": 0,
          "v": 0.0
        },
        {
          "t": 2,
          "v": 0.0
        },
        {
          "t": 4,
          "v": 0.0
        },
        {
          "t": 6,
          "v": 0.0
        },
        {
          "t": 8,
          "v": 0.0
        },
        {
          "t": 10,
          "v": 0.0
        },
        {
          "t": 12,
          "v": 0.0
        },
        {
          "t": 14,
          "v": 0.0
        },
        {
          "t": 16,
          "v": 0.0
        },
        {
          "t": 18,
          "v": 0.0
        },
        {
          "t": 20,
          "v": 0.0
        },
        {
          "t": 22,
          "v": 0.0
        },
        {
          "t": 24,
          "v": 0.0
        },
        {
          "t": 26,
          "v": 0.0
        },
        {
          "t": 28,
          "v": 0.0
        },
        {
          "t": 30,
          "v": 0.0
        },
        {
          "t": 32,
          "v": 0.0
        },
        {
          "t": 35,
          "v": 0.0
        },
        {
          "t": 37,
          "v": 0.0
        },
        {
          "t": 39,
          "v": 0.0
        },
        {
          "t": 41,
          "v": 0.0
        },
        {
          "t": 43,
          "v": 0.0
        },
        {
          "t": 45,
          "v": 0.0
        },
        {
          "t": 47,
          "v": 0.0
        },
        {
          "t": 49,
          "v": 0.0
        },
        {
          "t": 51,
          "v": 0.0
        },
        {
          "t": 53,
          "v": 0.0
        },
        {
          "t": 55,
          "v": 0.0
        },
        {
          "t": 57,
          "v": 0.0
        },
        {
          "t": 59,
          "v": 0.0
        },
        {
          "t": 61,
          "v": 0.0
        },
        {
          "t": 63,
          "v": 0.0
        },
        {
          "t": 65,
          "v": 0.0
        },
        {
          "t": 67,
          "v": 0.0
        },
        {
          "t": 69,
          "v": 0.0
        },
        {
          "t": 71,
          "v": 0.0
        },
        {
          "t": 73,
          "v": 0.0
        },
        {
          "t": 75,
          "v": 0.0
        },
        {
          "t": 77,
          "v": 0.0
        },
        {
          "t": 79,
          "v": 0.0
        },
        {
          "t": 81,
          "v": 0.0
        },
        {
          "t": 83,
          "v": 0.0
        },
        {
          "t": 85,
          "v": 0.0
        },
        {
          "t": 87,
          "v": 0.0
        },
        {
          "t": 89,
          "v": 0.0
        },
        {
          "t": 91,
          "v": 0.0
        },
        {
          "t": 93,
          "v": 0.0
        },
        {
          "t": 95,
          "v": 0.0
        },
        {
          "t": 97,
          "v": 0.0
        },
        {
          "t": 99,
          "v": 0.0
        },
        {
          "t": 101,
          "v": 0.0
        },
        {
          "t": 103,
          "v": 0.0
        },
        {
          "t": 105,
          "v": 0.0
        },
        {
          "t": 107,
          "v": 0.0
        },
        {
          "t": 109,
          "v": 0.0
        },
        {
          "t": 111,
          "v": 0.0
        },
        {
          "t": 113,
          "v": 0.0
        },
        {
          "t": 115,
          "v": 0.0
        },
        {
          "t": 117,
          "v": 0.0
        },
        {
          "t": 119,
          "v": 0.0
        },
        {
          "t": 121,
          "v": 0.0
        },
        {
          "t": 123,
          "v": 0.0
        },
        {
          "t": 125,
          "v": 0.0
        },
        {
          "t": 127,
          "v": 0.0
        },
        {
          "t": 129,
          "v": 0.0
        },
        {
          "t": 131,
          "v": 0.0
        },
        {
          "t": 133,
          "v": 0.0
        },
        {
          "t": 135,
          "v": 0.0
        },
        {
          "t": 137,
          "v": 0.0
        },
        {
          "t": 139,
          "v": 0.0
        },
        {
          "t": 141,
          "v": 0.0
        },
        {
          "t": 143,
          "v": 0.0
        },
        {
          "t": 145,
          "v": 0.0
        },
        {
          "t": 148,
          "v": 0.0
        },
        {
          "t": 150,
          "v": 0.0
        },
        {
          "t": 152,
          "v": 0.0
        },
        {
          "t": 154,
          "v": 0.0
        },
        {
          "t": 156,
          "v": 0.0
        },
        {
          "t": 158,
          "v": 0.0
        },
        {
          "t": 160,
          "v": 0.0
        },
        {
          "t": 162,
          "v": 0.0
        },
        {
          "t": 164,
          "v": 0.0
        },
        {
          "t": 166,
          "v": 0.0
        },
        {
          "t": 168,
          "v": 0.0
        },
        {
          "t": 170,
          "v": 0.0
        },
        {
          "t": 172,
          "v": 0.0
        },
        {
          "t": 174,
          "v": 0.0
        },
        {
          "t": 176,
          "v": 0.0
        },
        {
          "t": 178,
          "v": 0.0
        },
        {
          "t": 180,
          "v": 0.0
        },
        {
          "t": 182,
          "v": 0.0
        },
        {
          "t": 184,
          "v": 0.0
        },
        {
          "t": 186,
          "v": 0.0
        },
        {
          "t": 188,
          "v": 0.0
        },
        {
          "t": 190,
          "v": 0.0
        },
        {
          "t": 192,
          "v": 0.0
        },
        {
          "t": 194,
          "v": 0.0
        },
        {
          "t": 196,
          "v": 0.0
        },
        {
          "t": 198,
          "v": 0.0
        },
        {
          "t": 200,
          "v": 0.0
        },
        {
          "t": 202,
          "v": 0.0
        },
        {
          "t": 204,
          "v": 0.0
        },
        {
          "t": 206,
          "v": 0.0
        },
        {
          "t": 208,
          "v": 0.0
        },
        {
          "t": 210,
          "v": 0.0
        },
        {
          "t": 212,
          "v": 0.0
        },
        {
          "t": 214,
          "v": 0.0
        },
        {
          "t": 216,
          "v": 0.0
        },
        {
          "t": 218,
          "v": 0.0
        },
        {
          "t": 220,
          "v": 0.0
        },
        {
          "t": 222,
          "v": 0.0
        },
        {
          "t": 224,
          "v": 0.0
        },
        {
          "t": 226,
          "v": 0.0
        },
        {
          "t": 228,
          "v": 0.0
        },
        {
          "t": 230,
          "v": 0.0
        },
        {
          "t": 232,
          "v": 0.0
        },
        {
          "t": 234,
          "v": 0.0
        }
      ],
      "io": [
        {
          "t": 0,
          "v": 6.14
        },
        {
          "t": 2,
          "v": 5.03
        },
        {
          "t": 4,
          "v": 4.12
        },
        {
          "t": 6,
          "v": 3.37
        },
        {
          "t": 8,
          "v": 2.76
        },
        {
          "t": 10,
          "v": 2.26
        },
        {
          "t": 12,
          "v": 1.85
        },
        {
          "t": 14,
          "v": 1.51
        },
        {
          "t": 16,
          "v": 1.24
        },
        {
          "t": 18,
          "v": 1.01
        },
        {
          "t": 20,
          "v": 0.83
        },
        {
          "t": 22,
          "v": 0.68
        },
        {
          "t": 24,
          "v": 0.55
        },
        {
          "t": 26,
          "v": 0.45
        },
        {
          "t": 28,
          "v": 0.37
        },
        {
          "t": 30,
          "v": 0.3
        },
        {
          "t": 32,
          "v": 0.25
        },
        {
          "t": 35,
          "v": 0.2
        },
        {
          "t": 37,
          "v": 0.16
        },
        {
          "t": 39,
          "v": 0.13
        },
        {
          "t": 41,
          "v": 0.11
        },
        {
          "t": 43,
          "v": 0.09
        },
        {
          "t": 45,
          "v": 0.07
        },
        {
          "t": 47,
          "v": 0.06
        },
        {
          "t": 49,
          "v": 0.04
        },
        {
          "t": 51,
          "v": 0.04
        },
        {
          "t": 53,
          "v": 0.03
        },
        {
          "t": 55,
          "v": 0.02
        },
        {
          "t": 57,
          "v": 0.02
        },
        {
          "t": 59,
          "v": 0.01
        },
        {
          "t": 61,
          "v": 0.01
        },
        {
          "t": 63,
          "v": 0.01
        },
        {
          "t": 65,
          "v": 0.0
        },
        {
          "t": 67,
          "v": 0.0
        },
        {
          "t": 69,
          "v": 0.0
        },
        {
          "t": 71,
          "v": 0.0
        },
        {
          "t": 73,
          "v": 0.0
        },
        {
          "t": 75,
          "v": 0.0
        },
        {
          "t": 77,
          "v": 0.0
        },
        {
          "t": 79,
          "v": 0.0
        },
        {
          "t": 81,
          "v": 0.0
        },
        {
          "t": 83,
          "v": 0.0
        },
        {
          "t": 85,
          "v": 0.0
        },
        {
          "t": 87,
          "v": 0.0
        },
        {
          "t": 89,
          "v": 0.0
        },
        {
          "t": 91,
          "v": 0.0
        },
        {
          "t": 93,
          "v": 0.0
        },
        {
          "t": 95,
          "v": 0.0
        },
        {
          "t": 97,
          "v": 0.0
        },
        {
          "t": 99,
          "v": 0.0
        },
        {
          "t": 101,
          "v": 0.0
        },
        {
          "t": 103,
          "v": 0.0
        },
        {
          "t": 105,
          "v": 0.0
        },
        {
          "t": 107,
          "v": 0.0
        },
        {
          "t": 109,
          "v": 0.0
        },
        {
          "t": 111,
          "v": 0.0
        },
        {
          "t": 113,
          "v": 0.0
        },
        {
          "t": 115,
          "v": 0.0
        },
        {
          "t": 117,
          "v": 0.0
        },
        {
          "t": 119,
          "v": 0.0
        },
        {
          "t": 121,
          "v": 0.0
        },
        {
          "t": 123,
          "v": 0.0
        },
        {
          "t": 125,
          "v": 0.0
        },
        {
          "t": 127,
          "v": 0.0
        },
        {
          "t": 129,
          "v": 0.0
        },
        {
          "t": 131,
          "v": 0.0
        },
        {
          "t": 133,
          "v": 0.0
        },
        {
          "t": 135,
          "v": 0.0
        },
        {
          "t": 137,
          "v": 0.0
        },
        {
          "t": 139,
          "v": 0.0
        },
        {
          "t": 141,
          "v": 0.0
        },
        {
          "t": 143,
          "v": 0.0
        },
        {
          "t": 145,
          "v": 0.0
        },
        {
          "t": 148,
          "v": 0.0
        },
        {
          "t": 150,
          "v": 0.0
        },
        {
          "t": 152,
          "v": 0.0
        },
        {
          "t": 154,
          "v": 0.0
        },
        {
          "t": 156,
          "v": 0.0
        },
        {
          "t": 158,
          "v": 0.0
        },
        {
          "t": 160,
          "v": 0.0
        },
        {
          "t": 162,
          "v": 0.0
        },
        {
          "t": 164,
          "v": 0.0
        },
        {
          "t": 166,
          "v": 0.0
        },
        {
          "t": 168,
          "v": 0.0
        },
        {
          "t": 170,
          "v": 0.0
        },
        {
          "t": 172,
          "v": 0.0
        },
        {
          "t": 174,
          "v": 0.0
        },
        {
          "t": 176,
          "v": 0.0
        },
        {
          "t": 178,
          "v": 0.0
        },
        {
          "t": 180,
          "v": 0.0
        },
        {
          "t": 182,
          "v": 0.0
        },
        {
          "t": 184,
          "v": 0.0
        },
        {
          "t": 186,
          "v": 0.0
        },
        {
          "t": 188,
          "v": 0.0
        },
        {
          "t": 190,
          "v": 0.0
        },
        {
          "t": 192,
          "v": 0.0
        },
        {
          "t": 194,
          "v": 0.0
        },
        {
          "t": 196,
          "v": 0.0
        },
        {
          "t": 198,
          "v": 0.0
        },
        {
          "t": 200,
          "v": 0.0
        },
        {
          "t": 202,
          "v": 0.0
        },
        {
          "t": 204,
          "v": 0.0
        },
        {
          "t": 206,
          "v": 0.0
        },
        {
          "t": 208,
          "v": 0.0
        },
        {
          "t": 210,
          "v": 0.0
        },
        {
          "t": 212,
          "v": 0.0
        },
        {
          "t": 214,
          "v": 0.0
        },
        {
          "t": 216,
          "v": 0.0
        },
        {
          "t": 218,
          "v": 0.0
        },
        {
          "t": 220,
          "v": 0.0
        },
        {
          "t": 222,
          "v": 0.0
        },
        {
          "t": 224,
          "v": 0.0
        },
        {
          "t": 226,
          "v": 0.0
        },
        {
          "t": 228,
          "v": 0.0
        },
        {
          "t": 230,
          "v": 0.0
        },
        {
          "t": 232,
          "v": 0.0
        },
        {
          "t": 234,
          "v": 0.0
        }
      ]
    }
  },
  "artifacts": {
    "trace_server": "https://locustbaby.github.io/duotunnel/bench/traces/59a87d5-24183635629-server.bin.gz",
    "trace_client": "https://locustbaby.github.io/duotunnel/bench/traces/59a87d5-24183635629-client.bin.gz"
  }
};
