window.BENCH_DETAIL['756e8d1'] = {
  "timestamp": "2026-04-09T09:57:56.729Z",
  "commit": {
    "id": "756e8d113a09a0c9cbb7acc7a3e07b30a0b6b46f",
    "message": "fix: HEAD 502 and no-keepalive stall in H2 sender",
    "url": "https://github.com/locustbaby/duotunnel/commit/756e8d113a09a0c9cbb7acc7a3e07b30a0b6b46f"
  },
  "scenarios": [
    {
      "name": "ingress_http_get",
      "protocol": "HTTP",
      "direction": "ingress",
      "category": "basic",
      "includeInTotalRps": false,
      "p50": 0.47,
      "p95": 0.89,
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
      "p50": 0.48,
      "p95": 0.82,
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
      "p50": 0.45,
      "p95": 0.91,
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
      "p50": 0.45,
      "p95": 0.55,
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
      "p50": 1.6,
      "p95": 2.23,
      "p99": 0,
      "err": 0,
      "rps": 10.05,
      "requests": 201
    },
    {
      "name": "grpc_health_ingress",
      "protocol": "gRPC",
      "direction": "ingress",
      "category": "basic",
      "includeInTotalRps": false,
      "p50": 0.92,
      "p95": 1.45,
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
      "p50": 0.92,
      "p95": 1.64,
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
      "p50": 0.42,
      "p95": 0.55,
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
      "p50": 0.53,
      "p95": 1.88,
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
      "p50": 0.99,
      "p95": 2.11,
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
      "p50": 2.49,
      "p95": 3.35,
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
      "p50": 0.94,
      "p95": 2.05,
      "p99": 0,
      "err": 0,
      "rps": 20.05,
      "requests": 401
    },
    {
      "name": "ws_multi_msg",
      "protocol": "WS",
      "direction": "ingress",
      "category": "basic",
      "includeInTotalRps": false,
      "p50": 43.96,
      "p95": 45.78,
      "p99": 0,
      "err": 0,
      "rps": 5.05,
      "requests": 101
    },
    {
      "name": "grpc_large_payload",
      "protocol": "gRPC",
      "direction": "ingress",
      "category": "body_size",
      "includeInTotalRps": false,
      "p50": 0.94,
      "p95": 2.61,
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
      "p50": 0.7,
      "p95": 1.49,
      "p99": 0,
      "err": 0,
      "rps": 90,
      "requests": 1800
    },
    {
      "name": "ingress_3000qps",
      "protocol": "HTTP",
      "direction": "ingress",
      "category": "stress",
      "includeInTotalRps": true,
      "p50": 0.48,
      "p95": 1.85,
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
      "p50": 0.48,
      "p95": 1.82,
      "p99": 0,
      "err": 0,
      "rps": 3000,
      "requests": 60000
    },
    {
      "name": "ingress_multihost",
      "protocol": "HTTP",
      "direction": "ingress",
      "category": "stress",
      "includeInTotalRps": true,
      "p50": 0.5,
      "p95": 2.08,
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
      "p50": 0.5,
      "p95": 2.15,
      "p99": 0,
      "err": 0,
      "rps": 3000.05,
      "requests": 60001
    },
    {
      "name": "ingress_8000qps (8k-q4)",
      "protocol": "HTTP",
      "direction": "ingress",
      "category": "stress",
      "includeInTotalRps": true,
      "p50": 31.97,
      "p95": 104.75,
      "p99": 0,
      "err": 0,
      "rps": 7337.15,
      "requests": 146743
    },
    {
      "name": "egress_8000qps (8k-q4)",
      "protocol": "HTTP",
      "direction": "egress",
      "category": "stress",
      "includeInTotalRps": true,
      "p50": 12.3,
      "p95": 99.19,
      "p99": 0,
      "err": 0,
      "rps": 7386.9,
      "requests": 147738
    },
    {
      "name": "ingress_multihost_8000qps (8k-q4)",
      "protocol": "HTTP",
      "direction": "ingress",
      "category": "stress",
      "includeInTotalRps": true,
      "p50": 10.16,
      "p95": 35.12,
      "p99": 0,
      "err": 1.05,
      "rps": 7210.95,
      "requests": 144219
    },
    {
      "name": "egress_multihost_8000qps (8k-q4)",
      "protocol": "HTTP",
      "direction": "egress",
      "category": "stress",
      "includeInTotalRps": true,
      "p50": 36.03,
      "p95": 134.85,
      "p99": 0,
      "err": 0.01,
      "rps": 7187.05,
      "requests": 143741
    },
    {
      "name": "ingress_3000qps",
      "protocol": "HTTP",
      "direction": "ingress",
      "category": "stress",
      "includeInTotalRps": true,
      "p50": 1.74,
      "p95": 53.37,
      "p99": 0,
      "err": 0,
      "rps": 2925.8,
      "requests": 58516,
      "tunnel": "frp"
    },
    {
      "name": "ingress_multihost",
      "protocol": "HTTP",
      "direction": "ingress",
      "category": "stress",
      "includeInTotalRps": true,
      "p50": 0.55,
      "p95": 2.6,
      "p99": 0,
      "err": 0,
      "rps": 2998.65,
      "requests": 59973,
      "tunnel": "frp"
    },
    {
      "name": "ingress_multihost_8000qps",
      "protocol": "HTTP",
      "direction": "ingress",
      "category": "stress",
      "includeInTotalRps": true,
      "p50": 30.13,
      "p95": 94.53,
      "p99": 0,
      "err": 0,
      "rps": 6351.45,
      "requests": 127029,
      "tunnel": "frp"
    }
  ],
  "summary": {
    "totalRPS": 41212.2,
    "totalErr": 0.18,
    "totalRequests": 831175
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
  "run_url": "https://github.com/locustbaby/duotunnel/actions/runs/24183846731",
  "resources": {
    "processes": {
      "server": {
        "cpu": [
          {
            "t": 76.1,
            "v": 0.09
          },
          {
            "t": 79.7,
            "v": 0.76
          },
          {
            "t": 83.4,
            "v": 1.23
          },
          {
            "t": 87.1,
            "v": 1.62
          },
          {
            "t": 90.8,
            "v": 1.55
          },
          {
            "t": 94.5,
            "v": 1.56
          },
          {
            "t": 98.2,
            "v": 1.49
          },
          {
            "t": 101.8,
            "v": 0.88
          },
          {
            "t": 105.5,
            "v": 0.21
          },
          {
            "t": 109.1,
            "v": 0.21
          },
          {
            "t": 112.7,
            "v": 1.03
          },
          {
            "t": 116.4,
            "v": 1.64
          },
          {
            "t": 120.1,
            "v": 2.1
          },
          {
            "t": 123.8,
            "v": 2.07
          },
          {
            "t": 127.6,
            "v": 2.08
          },
          {
            "t": 131.3,
            "v": 1.55
          },
          {
            "t": 135.0,
            "v": 0.27
          },
          {
            "t": 138.6,
            "v": 1.66
          },
          {
            "t": 142.8,
            "v": 12.01
          },
          {
            "t": 147.2,
            "v": 11.66
          },
          {
            "t": 151.5,
            "v": 11.48
          },
          {
            "t": 155.9,
            "v": 11.67
          },
          {
            "t": 160.3,
            "v": 5.13
          },
          {
            "t": 163.9,
            "v": 3.8
          },
          {
            "t": 168.3,
            "v": 14.13
          },
          {
            "t": 172.7,
            "v": 14.13
          },
          {
            "t": 177.1,
            "v": 14.42
          },
          {
            "t": 181.4,
            "v": 14.12
          },
          {
            "t": 185.8,
            "v": 4.51
          },
          {
            "t": 189.4,
            "v": 5.24
          },
          {
            "t": 193.8,
            "v": 11.39
          },
          {
            "t": 198.2,
            "v": 11.43
          },
          {
            "t": 202.7,
            "v": 11.37
          },
          {
            "t": 207.1,
            "v": 11.65
          },
          {
            "t": 211.3,
            "v": 1.91
          },
          {
            "t": 215.0,
            "v": 8.54
          },
          {
            "t": 219.5,
            "v": 13.78
          },
          {
            "t": 223.9,
            "v": 13.8
          },
          {
            "t": 228.4,
            "v": 14.05
          },
          {
            "t": 232.8,
            "v": 13.61
          }
        ],
        "rss": [
          {
            "t": 3.6,
            "v": 14.5
          },
          {
            "t": 10.4,
            "v": 14.5
          },
          {
            "t": 15.6,
            "v": 14.5
          },
          {
            "t": 20.6,
            "v": 14.5
          },
          {
            "t": 25.4,
            "v": 14.5
          },
          {
            "t": 29.0,
            "v": 14.5
          },
          {
            "t": 33.6,
            "v": 14.5
          },
          {
            "t": 38.2,
            "v": 14.5
          },
          {
            "t": 42.8,
            "v": 14.5
          },
          {
            "t": 47.3,
            "v": 14.5
          },
          {
            "t": 51.0,
            "v": 14.5
          },
          {
            "t": 55.0,
            "v": 14.5
          },
          {
            "t": 62.3,
            "v": 14.5
          },
          {
            "t": 70.5,
            "v": 14.5
          },
          {
            "t": 76.1,
            "v": 14.7
          },
          {
            "t": 79.7,
            "v": 15.8
          },
          {
            "t": 83.4,
            "v": 16.4
          },
          {
            "t": 87.1,
            "v": 16.7
          },
          {
            "t": 90.8,
            "v": 16.9
          },
          {
            "t": 94.5,
            "v": 16.8
          },
          {
            "t": 98.2,
            "v": 16.7
          },
          {
            "t": 101.8,
            "v": 16.7
          },
          {
            "t": 105.5,
            "v": 16.8
          },
          {
            "t": 109.1,
            "v": 18.1
          },
          {
            "t": 112.7,
            "v": 19.7
          },
          {
            "t": 116.4,
            "v": 20.7
          },
          {
            "t": 120.1,
            "v": 21.0
          },
          {
            "t": 123.8,
            "v": 21.3
          },
          {
            "t": 127.6,
            "v": 21.5
          },
          {
            "t": 131.3,
            "v": 21.6
          },
          {
            "t": 135.0,
            "v": 21.6
          },
          {
            "t": 138.6,
            "v": 22.8
          },
          {
            "t": 142.8,
            "v": 22.7
          },
          {
            "t": 147.2,
            "v": 22.6
          },
          {
            "t": 151.5,
            "v": 22.5
          },
          {
            "t": 155.9,
            "v": 22.5
          },
          {
            "t": 160.3,
            "v": 22.5
          },
          {
            "t": 163.9,
            "v": 24.4
          },
          {
            "t": 168.3,
            "v": 25.1
          },
          {
            "t": 172.7,
            "v": 25.1
          },
          {
            "t": 177.1,
            "v": 25.0
          },
          {
            "t": 181.4,
            "v": 25.2
          },
          {
            "t": 185.8,
            "v": 25.1
          },
          {
            "t": 189.4,
            "v": 28.2
          },
          {
            "t": 193.8,
            "v": 28.3
          },
          {
            "t": 198.2,
            "v": 28.3
          },
          {
            "t": 202.7,
            "v": 28.3
          },
          {
            "t": 207.1,
            "v": 28.4
          },
          {
            "t": 211.3,
            "v": 28.4
          },
          {
            "t": 215.0,
            "v": 29.7
          },
          {
            "t": 219.5,
            "v": 29.8
          },
          {
            "t": 223.9,
            "v": 29.0
          },
          {
            "t": 228.4,
            "v": 29.1
          },
          {
            "t": 232.8,
            "v": 28.7
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
            "v": 2.0
          },
          {
            "t": 74.0,
            "v": 2.0
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
            "t": 76.1,
            "v": 0.04
          },
          {
            "t": 79.7,
            "v": 0.76
          },
          {
            "t": 83.4,
            "v": 1.29
          },
          {
            "t": 87.1,
            "v": 1.56
          },
          {
            "t": 90.8,
            "v": 1.49
          },
          {
            "t": 94.5,
            "v": 1.42
          },
          {
            "t": 98.2,
            "v": 1.49
          },
          {
            "t": 101.8,
            "v": 0.75
          },
          {
            "t": 105.5,
            "v": 0.14
          },
          {
            "t": 109.1,
            "v": 0.21
          },
          {
            "t": 112.7,
            "v": 1.03
          },
          {
            "t": 116.4,
            "v": 1.64
          },
          {
            "t": 120.1,
            "v": 1.97
          },
          {
            "t": 123.8,
            "v": 1.94
          },
          {
            "t": 127.6,
            "v": 1.95
          },
          {
            "t": 131.3,
            "v": 1.42
          },
          {
            "t": 135.0,
            "v": 0.27
          },
          {
            "t": 138.6,
            "v": 1.93
          },
          {
            "t": 142.8,
            "v": 14.55
          },
          {
            "t": 147.2,
            "v": 14.28
          },
          {
            "t": 151.5,
            "v": 14.05
          },
          {
            "t": 155.9,
            "v": 14.13
          },
          {
            "t": 160.3,
            "v": 6.17
          },
          {
            "t": 163.9,
            "v": 3.32
          },
          {
            "t": 168.3,
            "v": 11.4
          },
          {
            "t": 172.7,
            "v": 11.52
          },
          {
            "t": 177.1,
            "v": 11.51
          },
          {
            "t": 181.4,
            "v": 11.43
          },
          {
            "t": 185.8,
            "v": 3.6
          },
          {
            "t": 189.4,
            "v": 6.28
          },
          {
            "t": 193.8,
            "v": 13.78
          },
          {
            "t": 198.2,
            "v": 13.97
          },
          {
            "t": 202.7,
            "v": 13.84
          },
          {
            "t": 207.1,
            "v": 14.14
          },
          {
            "t": 211.3,
            "v": 2.26
          },
          {
            "t": 215.0,
            "v": 7.18
          },
          {
            "t": 219.5,
            "v": 11.12
          },
          {
            "t": 223.9,
            "v": 11.21
          },
          {
            "t": 228.4,
            "v": 11.35
          },
          {
            "t": 232.8,
            "v": 10.97
          }
        ],
        "rss": [
          {
            "t": 3.6,
            "v": 11.0
          },
          {
            "t": 10.4,
            "v": 11.0
          },
          {
            "t": 15.6,
            "v": 11.0
          },
          {
            "t": 20.6,
            "v": 11.0
          },
          {
            "t": 25.4,
            "v": 11.0
          },
          {
            "t": 29.0,
            "v": 11.0
          },
          {
            "t": 33.6,
            "v": 11.0
          },
          {
            "t": 38.2,
            "v": 11.0
          },
          {
            "t": 42.8,
            "v": 11.0
          },
          {
            "t": 47.3,
            "v": 11.0
          },
          {
            "t": 51.0,
            "v": 11.0
          },
          {
            "t": 55.0,
            "v": 11.0
          },
          {
            "t": 62.3,
            "v": 11.0
          },
          {
            "t": 70.5,
            "v": 11.0
          },
          {
            "t": 76.1,
            "v": 11.2
          },
          {
            "t": 79.7,
            "v": 12.2
          },
          {
            "t": 83.4,
            "v": 12.7
          },
          {
            "t": 87.1,
            "v": 13.0
          },
          {
            "t": 90.8,
            "v": 13.1
          },
          {
            "t": 94.5,
            "v": 13.3
          },
          {
            "t": 98.2,
            "v": 13.3
          },
          {
            "t": 101.8,
            "v": 13.3
          },
          {
            "t": 105.5,
            "v": 13.3
          },
          {
            "t": 109.1,
            "v": 15.7
          },
          {
            "t": 112.7,
            "v": 17.7
          },
          {
            "t": 116.4,
            "v": 17.9
          },
          {
            "t": 120.1,
            "v": 18.2
          },
          {
            "t": 123.8,
            "v": 18.1
          },
          {
            "t": 127.6,
            "v": 18.6
          },
          {
            "t": 131.3,
            "v": 18.0
          },
          {
            "t": 135.0,
            "v": 18.0
          },
          {
            "t": 138.6,
            "v": 18.7
          },
          {
            "t": 142.8,
            "v": 18.3
          },
          {
            "t": 147.2,
            "v": 18.5
          },
          {
            "t": 151.5,
            "v": 18.1
          },
          {
            "t": 155.9,
            "v": 18.3
          },
          {
            "t": 160.3,
            "v": 18.3
          },
          {
            "t": 163.9,
            "v": 19.7
          },
          {
            "t": 168.3,
            "v": 19.7
          },
          {
            "t": 172.7,
            "v": 19.7
          },
          {
            "t": 177.1,
            "v": 19.6
          },
          {
            "t": 181.4,
            "v": 19.6
          },
          {
            "t": 185.8,
            "v": 19.6
          },
          {
            "t": 189.4,
            "v": 22.3
          },
          {
            "t": 193.8,
            "v": 22.9
          },
          {
            "t": 198.2,
            "v": 22.4
          },
          {
            "t": 202.7,
            "v": 22.8
          },
          {
            "t": 207.1,
            "v": 22.4
          },
          {
            "t": 211.3,
            "v": 22.6
          },
          {
            "t": 215.0,
            "v": 23.0
          },
          {
            "t": 219.5,
            "v": 23.0
          },
          {
            "t": 223.9,
            "v": 22.9
          },
          {
            "t": 228.4,
            "v": 22.5
          },
          {
            "t": 232.8,
            "v": 22.4
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
            "v": 4.0
          },
          {
            "t": 78.0,
            "v": 6.0
          },
          {
            "t": 80.0,
            "v": 8.0
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
            "v": 4.0
          },
          {
            "t": 94.0,
            "v": 6.0
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
            "v": 4.0
          },
          {
            "t": 122.0,
            "v": 2.0
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
            "v": 1.0
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
            "v": 12.0
          },
          {
            "t": 80.0,
            "v": 13.0
          },
          {
            "t": 82.0,
            "v": 12.5
          },
          {
            "t": 84.0,
            "v": 11.5
          },
          {
            "t": 86.0,
            "v": 13.0
          },
          {
            "t": 88.0,
            "v": 13.5
          },
          {
            "t": 90.0,
            "v": 12.0
          },
          {
            "t": 92.0,
            "v": 14.5
          },
          {
            "t": 94.0,
            "v": 12.0
          },
          {
            "t": 96.0,
            "v": 6.0
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
            "v": 8.0
          },
          {
            "t": 110.0,
            "v": 7.0
          },
          {
            "t": 112.0,
            "v": 8.5
          },
          {
            "t": 114.0,
            "v": 6.0
          },
          {
            "t": 116.0,
            "v": 7.5
          },
          {
            "t": 118.0,
            "v": 8.5
          },
          {
            "t": 120.0,
            "v": 6.5
          },
          {
            "t": 122.0,
            "v": 6.5
          },
          {
            "t": 124.0,
            "v": 7.0
          },
          {
            "t": 126.0,
            "v": 7.0
          },
          {
            "t": 128.0,
            "v": 7.5
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
            "v": 49.5
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
            "v": 84.0
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
            "v": 1.0
          },
          {
            "t": 78.0,
            "v": 2.0
          },
          {
            "t": 80.0,
            "v": 0.0
          },
          {
            "t": 82.0,
            "v": 1.5
          },
          {
            "t": 84.0,
            "v": 0.5
          },
          {
            "t": 86.0,
            "v": 0.5
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
            "v": 0.5
          },
          {
            "t": 94.0,
            "v": 0.5
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
            "v": 0.0
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
            "v": 0.5
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
            "v": 0.5
          },
          {
            "t": 126.0,
            "v": 0.0
          },
          {
            "t": 128.0,
            "v": 3.5
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
            "v": 4.0
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
            "v": 1.0
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
            "t": 3.6,
            "v": 2.55
          },
          {
            "t": 10.4,
            "v": 4.77
          },
          {
            "t": 15.6,
            "v": 4.41
          },
          {
            "t": 20.6,
            "v": 4.81
          },
          {
            "t": 25.4,
            "v": 0.42
          },
          {
            "t": 29.0,
            "v": 2.55
          },
          {
            "t": 33.6,
            "v": 3.86
          },
          {
            "t": 38.2,
            "v": 3.79
          },
          {
            "t": 42.8,
            "v": 3.82
          },
          {
            "t": 47.3,
            "v": 3.12
          },
          {
            "t": 55.0,
            "v": 5.96
          },
          {
            "t": 62.3,
            "v": 6.05
          },
          {
            "t": 70.5,
            "v": 6.48
          },
          {
            "t": 76.1,
            "v": 0.04
          },
          {
            "t": 79.7,
            "v": 0.07
          },
          {
            "t": 83.4,
            "v": 0.2
          },
          {
            "t": 87.1,
            "v": 0.27
          },
          {
            "t": 90.8,
            "v": 0.27
          },
          {
            "t": 94.5,
            "v": 0.27
          },
          {
            "t": 98.2,
            "v": 0.27
          },
          {
            "t": 101.8,
            "v": 0.2
          },
          {
            "t": 105.5,
            "v": 0.07
          },
          {
            "t": 112.7,
            "v": 0.27
          },
          {
            "t": 116.4,
            "v": 0.27
          },
          {
            "t": 120.1,
            "v": 0.27
          },
          {
            "t": 123.8,
            "v": 0.27
          },
          {
            "t": 127.6,
            "v": 0.27
          },
          {
            "t": 138.6,
            "v": 0.48
          },
          {
            "t": 142.8,
            "v": 3.43
          },
          {
            "t": 147.2,
            "v": 3.31
          },
          {
            "t": 151.5,
            "v": 3.25
          },
          {
            "t": 155.9,
            "v": 3.37
          },
          {
            "t": 160.3,
            "v": 1.56
          },
          {
            "t": 163.9,
            "v": 0.83
          },
          {
            "t": 168.3,
            "v": 3.31
          },
          {
            "t": 172.7,
            "v": 3.29
          },
          {
            "t": 177.1,
            "v": 3.3
          },
          {
            "t": 181.4,
            "v": 3.32
          },
          {
            "t": 185.8,
            "v": 1.2
          },
          {
            "t": 189.4,
            "v": 1.38
          },
          {
            "t": 193.8,
            "v": 3.3
          },
          {
            "t": 198.2,
            "v": 3.28
          },
          {
            "t": 202.7,
            "v": 3.31
          },
          {
            "t": 207.1,
            "v": 3.28
          },
          {
            "t": 211.3,
            "v": 0.71
          },
          {
            "t": 215.0,
            "v": 1.91
          },
          {
            "t": 219.5,
            "v": 3.27
          },
          {
            "t": 223.9,
            "v": 3.21
          },
          {
            "t": 228.4,
            "v": 3.37
          },
          {
            "t": 232.8,
            "v": 3.26
          }
        ],
        "rss": [
          {
            "t": 3.6,
            "v": 5.3
          },
          {
            "t": 10.4,
            "v": 11.6
          },
          {
            "t": 15.6,
            "v": 11.7
          },
          {
            "t": 20.6,
            "v": 12.5
          },
          {
            "t": 25.4,
            "v": 12.4
          },
          {
            "t": 29.0,
            "v": 12.4
          },
          {
            "t": 33.6,
            "v": 12.4
          },
          {
            "t": 38.2,
            "v": 12.4
          },
          {
            "t": 42.8,
            "v": 12.4
          },
          {
            "t": 47.3,
            "v": 12.4
          },
          {
            "t": 51.0,
            "v": 12.4
          },
          {
            "t": 55.0,
            "v": 12.7
          },
          {
            "t": 62.3,
            "v": 15.9
          },
          {
            "t": 70.5,
            "v": 23.3
          },
          {
            "t": 76.1,
            "v": 20.2
          },
          {
            "t": 79.7,
            "v": 20.2
          },
          {
            "t": 83.4,
            "v": 20.2
          },
          {
            "t": 87.1,
            "v": 20.2
          },
          {
            "t": 90.8,
            "v": 20.2
          },
          {
            "t": 94.5,
            "v": 20.2
          },
          {
            "t": 98.2,
            "v": 20.2
          },
          {
            "t": 101.8,
            "v": 20.2
          },
          {
            "t": 105.5,
            "v": 20.2
          },
          {
            "t": 109.1,
            "v": 20.3
          },
          {
            "t": 112.7,
            "v": 20.3
          },
          {
            "t": 116.4,
            "v": 20.3
          },
          {
            "t": 120.1,
            "v": 20.3
          },
          {
            "t": 123.8,
            "v": 20.3
          },
          {
            "t": 127.6,
            "v": 20.3
          },
          {
            "t": 131.3,
            "v": 20.3
          },
          {
            "t": 135.0,
            "v": 20.3
          },
          {
            "t": 138.6,
            "v": 20.3
          },
          {
            "t": 142.8,
            "v": 20.3
          },
          {
            "t": 147.2,
            "v": 20.1
          },
          {
            "t": 151.5,
            "v": 20.1
          },
          {
            "t": 155.9,
            "v": 20.1
          },
          {
            "t": 160.3,
            "v": 20.1
          },
          {
            "t": 163.9,
            "v": 20.1
          },
          {
            "t": 168.3,
            "v": 20.1
          },
          {
            "t": 172.7,
            "v": 20.1
          },
          {
            "t": 177.1,
            "v": 20.1
          },
          {
            "t": 181.4,
            "v": 20.1
          },
          {
            "t": 185.8,
            "v": 20.1
          },
          {
            "t": 189.4,
            "v": 20.1
          },
          {
            "t": 193.8,
            "v": 20.1
          },
          {
            "t": 198.2,
            "v": 20.1
          },
          {
            "t": 202.7,
            "v": 20.1
          },
          {
            "t": 207.1,
            "v": 20.1
          },
          {
            "t": 211.3,
            "v": 20.1
          },
          {
            "t": 215.0,
            "v": 20.1
          },
          {
            "t": 219.5,
            "v": 20.1
          },
          {
            "t": 223.9,
            "v": 20.1
          },
          {
            "t": 228.4,
            "v": 20.1
          },
          {
            "t": 232.8,
            "v": 20.1
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
            "v": 54.0
          },
          {
            "t": 4.0,
            "v": 288.6
          },
          {
            "t": 6.0,
            "v": 278.5
          },
          {
            "t": 8.0,
            "v": 282.5
          },
          {
            "t": 10.0,
            "v": 122.0
          },
          {
            "t": 12.0,
            "v": 222.0
          },
          {
            "t": 14.0,
            "v": 73.5
          },
          {
            "t": 16.0,
            "v": 176.0
          },
          {
            "t": 18.0,
            "v": 94.0
          },
          {
            "t": 20.0,
            "v": 323.5
          },
          {
            "t": 22.0,
            "v": 81.0
          },
          {
            "t": 24.0,
            "v": 0.0
          },
          {
            "t": 26.0,
            "v": 8.0
          },
          {
            "t": 28.0,
            "v": 0.0
          },
          {
            "t": 30.0,
            "v": 0.5
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
            "v": 0.5
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
            "v": 17.0
          },
          {
            "t": 54.0,
            "v": 11.4
          },
          {
            "t": 56.0,
            "v": 31.0
          },
          {
            "t": 58.0,
            "v": 85.0
          },
          {
            "t": 60.0,
            "v": 62.2
          },
          {
            "t": 62.0,
            "v": 131.0
          },
          {
            "t": 64.0,
            "v": 155.2
          },
          {
            "t": 66.0,
            "v": 193.5
          },
          {
            "t": 68.0,
            "v": 235.5
          },
          {
            "t": 70.0,
            "v": 194.5
          },
          {
            "t": 72.0,
            "v": 53.0
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
            "v": 1.0
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
            "v": 19.0
          },
          {
            "t": 140.0,
            "v": 0.5
          },
          {
            "t": 142.0,
            "v": 3.0
          },
          {
            "t": 144.0,
            "v": 1.5
          },
          {
            "t": 146.0,
            "v": 6.5
          },
          {
            "t": 148.0,
            "v": 0.5
          },
          {
            "t": 150.0,
            "v": 5.0
          },
          {
            "t": 152.0,
            "v": 8.5
          },
          {
            "t": 154.0,
            "v": 4.5
          },
          {
            "t": 156.0,
            "v": 13.0
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
            "v": 7.0
          },
          {
            "t": 164.0,
            "v": 6.5
          },
          {
            "t": 166.0,
            "v": 5.5
          },
          {
            "t": 168.0,
            "v": 0.5
          },
          {
            "t": 170.0,
            "v": 1.5
          },
          {
            "t": 172.0,
            "v": 7.5
          },
          {
            "t": 174.0,
            "v": 0.0
          },
          {
            "t": 176.0,
            "v": 3.0
          },
          {
            "t": 178.0,
            "v": 21.0
          },
          {
            "t": 180.0,
            "v": 5.0
          },
          {
            "t": 182.0,
            "v": 8.5
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
            "v": 17.5
          },
          {
            "t": 192.0,
            "v": 3.0
          },
          {
            "t": 194.0,
            "v": 6.0
          },
          {
            "t": 196.0,
            "v": 2.0
          },
          {
            "t": 198.0,
            "v": 9.5
          },
          {
            "t": 200.0,
            "v": 0.0
          },
          {
            "t": 202.0,
            "v": 3.5
          },
          {
            "t": 204.0,
            "v": 13.0
          },
          {
            "t": 206.0,
            "v": 1.5
          },
          {
            "t": 208.0,
            "v": 8.5
          },
          {
            "t": 210.0,
            "v": 0.0
          },
          {
            "t": 212.0,
            "v": 0.5
          },
          {
            "t": 214.0,
            "v": 4.5
          },
          {
            "t": 216.0,
            "v": 0.5
          },
          {
            "t": 218.0,
            "v": 2.0
          },
          {
            "t": 220.0,
            "v": 13.5
          },
          {
            "t": 222.0,
            "v": 1.5
          },
          {
            "t": 224.0,
            "v": 12.5
          },
          {
            "t": 226.0,
            "v": 20.5
          },
          {
            "t": 228.0,
            "v": 7.0
          },
          {
            "t": 230.0,
            "v": 5.0
          },
          {
            "t": 232.0,
            "v": 5.5
          }
        ],
        "nvcswch": [
          {
            "t": 0.0,
            "v": 0.0
          },
          {
            "t": 2.0,
            "v": 5.5
          },
          {
            "t": 4.0,
            "v": 111.4
          },
          {
            "t": 6.0,
            "v": 130.5
          },
          {
            "t": 8.0,
            "v": 95.5
          },
          {
            "t": 10.0,
            "v": 70.5
          },
          {
            "t": 12.0,
            "v": 111.0
          },
          {
            "t": 14.0,
            "v": 16.5
          },
          {
            "t": 16.0,
            "v": 42.5
          },
          {
            "t": 18.0,
            "v": 47.5
          },
          {
            "t": 20.0,
            "v": 189.5
          },
          {
            "t": 22.0,
            "v": 19.5
          },
          {
            "t": 24.0,
            "v": 0.0
          },
          {
            "t": 26.0,
            "v": 0.5
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
            "v": 0.5
          },
          {
            "t": 56.0,
            "v": 0.5
          },
          {
            "t": 58.0,
            "v": 2.0
          },
          {
            "t": 60.0,
            "v": 1.5
          },
          {
            "t": 62.0,
            "v": 60.0
          },
          {
            "t": 64.0,
            "v": 27.9
          },
          {
            "t": 66.0,
            "v": 53.5
          },
          {
            "t": 68.0,
            "v": 63.0
          },
          {
            "t": 70.0,
            "v": 115.0
          },
          {
            "t": 72.0,
            "v": 18.0
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
            "v": 3.5
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
            "v": 4.5
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
            "v": 1.5
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
            "v": 2.5
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
            "v": 10.5
          },
          {
            "t": 180.0,
            "v": 0.5
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
            "v": 1.0
          },
          {
            "t": 192.0,
            "v": 0.0
          },
          {
            "t": 194.0,
            "v": 0.5
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
            "v": 0.5
          },
          {
            "t": 214.0,
            "v": 0.5
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
            "v": 6.0
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
            "v": 1.5
          },
          {
            "t": 228.0,
            "v": 3.5
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
      "ws_echo": {
        "cpu": [
          {
            "t": 83.4,
            "v": 0.07
          },
          {
            "t": 87.1,
            "v": 0.07
          },
          {
            "t": 90.8,
            "v": 0.07
          },
          {
            "t": 94.5,
            "v": 0.07
          },
          {
            "t": 112.7,
            "v": 0.07
          },
          {
            "t": 123.8,
            "v": 0.13
          },
          {
            "t": 127.6,
            "v": 0.07
          }
        ],
        "rss": [
          {
            "t": 3.6,
            "v": 2.8
          },
          {
            "t": 10.4,
            "v": 2.8
          },
          {
            "t": 15.6,
            "v": 2.8
          },
          {
            "t": 20.6,
            "v": 2.8
          },
          {
            "t": 25.4,
            "v": 2.8
          },
          {
            "t": 29.0,
            "v": 2.8
          },
          {
            "t": 33.6,
            "v": 2.8
          },
          {
            "t": 38.2,
            "v": 2.8
          },
          {
            "t": 42.8,
            "v": 2.8
          },
          {
            "t": 47.3,
            "v": 2.8
          },
          {
            "t": 51.0,
            "v": 2.8
          },
          {
            "t": 55.0,
            "v": 2.8
          },
          {
            "t": 62.3,
            "v": 2.8
          },
          {
            "t": 70.5,
            "v": 2.8
          },
          {
            "t": 76.1,
            "v": 2.8
          },
          {
            "t": 79.7,
            "v": 3.1
          },
          {
            "t": 83.4,
            "v": 3.1
          },
          {
            "t": 87.1,
            "v": 3.1
          },
          {
            "t": 90.8,
            "v": 3.1
          },
          {
            "t": 94.5,
            "v": 3.1
          },
          {
            "t": 98.2,
            "v": 3.1
          },
          {
            "t": 101.8,
            "v": 3.1
          },
          {
            "t": 105.5,
            "v": 3.1
          },
          {
            "t": 109.1,
            "v": 3.1
          },
          {
            "t": 112.7,
            "v": 3.1
          },
          {
            "t": 116.4,
            "v": 3.1
          },
          {
            "t": 120.1,
            "v": 3.1
          },
          {
            "t": 123.8,
            "v": 3.1
          },
          {
            "t": 127.6,
            "v": 3.1
          },
          {
            "t": 131.3,
            "v": 3.1
          },
          {
            "t": 135.0,
            "v": 3.1
          },
          {
            "t": 138.6,
            "v": 3.1
          },
          {
            "t": 142.8,
            "v": 3.1
          },
          {
            "t": 147.2,
            "v": 3.1
          },
          {
            "t": 151.5,
            "v": 3.1
          },
          {
            "t": 155.9,
            "v": 3.1
          },
          {
            "t": 160.3,
            "v": 3.1
          },
          {
            "t": 163.9,
            "v": 3.1
          },
          {
            "t": 168.3,
            "v": 3.1
          },
          {
            "t": 172.7,
            "v": 3.1
          },
          {
            "t": 177.1,
            "v": 3.1
          },
          {
            "t": 181.4,
            "v": 3.1
          },
          {
            "t": 185.8,
            "v": 3.1
          },
          {
            "t": 189.4,
            "v": 3.1
          },
          {
            "t": 193.8,
            "v": 3.1
          },
          {
            "t": 198.2,
            "v": 3.1
          },
          {
            "t": 202.7,
            "v": 3.1
          },
          {
            "t": 207.1,
            "v": 3.1
          },
          {
            "t": 211.3,
            "v": 3.1
          },
          {
            "t": 215.0,
            "v": 3.1
          },
          {
            "t": 219.5,
            "v": 3.1
          },
          {
            "t": 223.9,
            "v": 3.1
          },
          {
            "t": 228.4,
            "v": 3.1
          },
          {
            "t": 232.8,
            "v": 3.1
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
            "v": 6.5
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
            "v": 4.0
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
            "v": 4.5
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
            "v": 2.5
          },
          {
            "t": 78.0,
            "v": 4.5
          },
          {
            "t": 80.0,
            "v": 1.0
          },
          {
            "t": 82.0,
            "v": 6.5
          },
          {
            "t": 84.0,
            "v": 4.0
          },
          {
            "t": 86.0,
            "v": 7.0
          },
          {
            "t": 88.0,
            "v": 5.0
          },
          {
            "t": 90.0,
            "v": 3.5
          },
          {
            "t": 92.0,
            "v": 3.0
          },
          {
            "t": 94.0,
            "v": 6.5
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
            "v": 0.0
          },
          {
            "t": 110.0,
            "v": 3.0
          },
          {
            "t": 112.0,
            "v": 2.0
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
            "v": 1.5
          },
          {
            "t": 122.0,
            "v": 4.0
          },
          {
            "t": 124.0,
            "v": 1.0
          },
          {
            "t": 126.0,
            "v": 4.5
          },
          {
            "t": 128.0,
            "v": 1.5
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
            "t": 83.4,
            "v": 0.14
          },
          {
            "t": 87.1,
            "v": 0.14
          },
          {
            "t": 90.8,
            "v": 0.14
          },
          {
            "t": 94.5,
            "v": 0.14
          },
          {
            "t": 98.2,
            "v": 0.14
          },
          {
            "t": 112.7,
            "v": 0.07
          },
          {
            "t": 116.4,
            "v": 0.27
          },
          {
            "t": 120.1,
            "v": 0.41
          },
          {
            "t": 123.8,
            "v": 0.4
          },
          {
            "t": 127.6,
            "v": 0.47
          },
          {
            "t": 131.3,
            "v": 0.4
          },
          {
            "t": 135.0,
            "v": 0.07
          }
        ],
        "rss": [
          {
            "t": 3.6,
            "v": 3.4
          },
          {
            "t": 10.4,
            "v": 3.4
          },
          {
            "t": 15.6,
            "v": 3.4
          },
          {
            "t": 20.6,
            "v": 3.4
          },
          {
            "t": 25.4,
            "v": 3.4
          },
          {
            "t": 29.0,
            "v": 3.4
          },
          {
            "t": 33.6,
            "v": 3.4
          },
          {
            "t": 38.2,
            "v": 3.4
          },
          {
            "t": 42.8,
            "v": 3.4
          },
          {
            "t": 47.3,
            "v": 3.4
          },
          {
            "t": 51.0,
            "v": 3.4
          },
          {
            "t": 55.0,
            "v": 3.4
          },
          {
            "t": 62.3,
            "v": 3.4
          },
          {
            "t": 70.5,
            "v": 3.4
          },
          {
            "t": 76.1,
            "v": 3.4
          },
          {
            "t": 79.7,
            "v": 4.0
          },
          {
            "t": 83.4,
            "v": 4.1
          },
          {
            "t": 87.1,
            "v": 4.1
          },
          {
            "t": 90.8,
            "v": 4.1
          },
          {
            "t": 94.5,
            "v": 4.1
          },
          {
            "t": 98.2,
            "v": 4.2
          },
          {
            "t": 101.8,
            "v": 4.2
          },
          {
            "t": 105.5,
            "v": 4.2
          },
          {
            "t": 109.1,
            "v": 4.2
          },
          {
            "t": 112.7,
            "v": 4.2
          },
          {
            "t": 116.4,
            "v": 4.3
          },
          {
            "t": 120.1,
            "v": 4.3
          },
          {
            "t": 123.8,
            "v": 4.3
          },
          {
            "t": 127.6,
            "v": 4.4
          },
          {
            "t": 131.3,
            "v": 4.4
          },
          {
            "t": 135.0,
            "v": 4.4
          },
          {
            "t": 138.6,
            "v": 4.4
          },
          {
            "t": 142.8,
            "v": 4.4
          },
          {
            "t": 147.2,
            "v": 4.4
          },
          {
            "t": 151.5,
            "v": 4.4
          },
          {
            "t": 155.9,
            "v": 4.4
          },
          {
            "t": 160.3,
            "v": 4.4
          },
          {
            "t": 163.9,
            "v": 4.4
          },
          {
            "t": 168.3,
            "v": 4.4
          },
          {
            "t": 172.7,
            "v": 4.4
          },
          {
            "t": 177.1,
            "v": 4.4
          },
          {
            "t": 181.4,
            "v": 4.4
          },
          {
            "t": 185.8,
            "v": 4.4
          },
          {
            "t": 189.4,
            "v": 4.4
          },
          {
            "t": 193.8,
            "v": 4.4
          },
          {
            "t": 198.2,
            "v": 4.4
          },
          {
            "t": 202.7,
            "v": 4.4
          },
          {
            "t": 207.1,
            "v": 4.4
          },
          {
            "t": 211.3,
            "v": 4.4
          },
          {
            "t": 215.0,
            "v": 4.4
          },
          {
            "t": 219.5,
            "v": 4.4
          },
          {
            "t": 223.9,
            "v": 4.4
          },
          {
            "t": 228.4,
            "v": 4.4
          },
          {
            "t": 232.8,
            "v": 4.4
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
            "t": 10.4,
            "v": 19.29
          },
          {
            "t": 15.6,
            "v": 22.72
          },
          {
            "t": 20.6,
            "v": 21.4
          },
          {
            "t": 25.4,
            "v": 0.1
          },
          {
            "t": 29.0,
            "v": 19.5
          },
          {
            "t": 33.6,
            "v": 22.32
          },
          {
            "t": 38.2,
            "v": 21.38
          },
          {
            "t": 42.8,
            "v": 21.38
          },
          {
            "t": 47.3,
            "v": 15.24
          },
          {
            "t": 51.0,
            "v": 0.07
          },
          {
            "t": 55.0,
            "v": 59.38
          },
          {
            "t": 62.3,
            "v": 43.17
          },
          {
            "t": 79.7,
            "v": 1.11
          },
          {
            "t": 83.4,
            "v": 2.32
          },
          {
            "t": 87.1,
            "v": 2.57
          },
          {
            "t": 90.8,
            "v": 2.7
          },
          {
            "t": 94.5,
            "v": 2.71
          },
          {
            "t": 98.2,
            "v": 2.72
          },
          {
            "t": 101.8,
            "v": 3.12
          },
          {
            "t": 105.5,
            "v": 0.41
          },
          {
            "t": 109.1,
            "v": 0.34
          },
          {
            "t": 112.7,
            "v": 1.58
          },
          {
            "t": 116.4,
            "v": 2.45
          },
          {
            "t": 120.1,
            "v": 4.48
          },
          {
            "t": 123.8,
            "v": 2.94
          },
          {
            "t": 127.6,
            "v": 2.96
          },
          {
            "t": 131.3,
            "v": 2.09
          },
          {
            "t": 135.0,
            "v": 0.34
          },
          {
            "t": 138.6,
            "v": 6.7
          },
          {
            "t": 142.8,
            "v": 23.9
          },
          {
            "t": 147.2,
            "v": 23.2
          },
          {
            "t": 151.5,
            "v": 21.3
          },
          {
            "t": 155.9,
            "v": 23.39
          },
          {
            "t": 160.3,
            "v": 9.16
          },
          {
            "t": 163.9,
            "v": 7.89
          },
          {
            "t": 168.3,
            "v": 22.68
          },
          {
            "t": 172.7,
            "v": 22.98
          },
          {
            "t": 177.1,
            "v": 23.36
          },
          {
            "t": 181.4,
            "v": 23.27
          },
          {
            "t": 185.8,
            "v": 6.96
          },
          {
            "t": 189.4,
            "v": 11.25
          },
          {
            "t": 193.8,
            "v": 24.94
          },
          {
            "t": 198.2,
            "v": 24.72
          },
          {
            "t": 202.7,
            "v": 24.65
          },
          {
            "t": 207.1,
            "v": 25.05
          },
          {
            "t": 211.3,
            "v": 3.99
          },
          {
            "t": 215.0,
            "v": 17.36
          },
          {
            "t": 219.5,
            "v": 24.63
          },
          {
            "t": 223.9,
            "v": 25.24
          },
          {
            "t": 228.4,
            "v": 24.89
          },
          {
            "t": 232.8,
            "v": 24.24
          }
        ],
        "rss": [
          {
            "t": 3.6,
            "v": 146.9
          },
          {
            "t": 10.4,
            "v": 210.5
          },
          {
            "t": 15.6,
            "v": 215.3
          },
          {
            "t": 20.6,
            "v": 234.8
          },
          {
            "t": 25.4,
            "v": 234.8
          },
          {
            "t": 29.0,
            "v": 224.8
          },
          {
            "t": 33.6,
            "v": 234.4
          },
          {
            "t": 38.2,
            "v": 235.5
          },
          {
            "t": 42.8,
            "v": 235.6
          },
          {
            "t": 47.3,
            "v": 243.5
          },
          {
            "t": 51.0,
            "v": 243.5
          },
          {
            "t": 55.0,
            "v": 372.5
          },
          {
            "t": 62.3,
            "v": 493.8
          },
          {
            "t": 76.1,
            "v": 371.7
          },
          {
            "t": 79.7,
            "v": 372.1
          },
          {
            "t": 83.4,
            "v": 372.2
          },
          {
            "t": 87.1,
            "v": 375.1
          },
          {
            "t": 90.8,
            "v": 384.1
          },
          {
            "t": 94.5,
            "v": 408.6
          },
          {
            "t": 98.2,
            "v": 433.4
          },
          {
            "t": 101.8,
            "v": 435.7
          },
          {
            "t": 105.5,
            "v": 435.7
          },
          {
            "t": 109.1,
            "v": 435.8
          },
          {
            "t": 112.7,
            "v": 436.5
          },
          {
            "t": 116.4,
            "v": 437.4
          },
          {
            "t": 120.1,
            "v": 439.7
          },
          {
            "t": 123.8,
            "v": 439.7
          },
          {
            "t": 127.6,
            "v": 439.8
          },
          {
            "t": 131.3,
            "v": 439.9
          },
          {
            "t": 135.0,
            "v": 439.9
          },
          {
            "t": 138.6,
            "v": 454.5
          },
          {
            "t": 142.8,
            "v": 494.2
          },
          {
            "t": 147.2,
            "v": 509.6
          },
          {
            "t": 151.5,
            "v": 509.9
          },
          {
            "t": 155.9,
            "v": 518.9
          },
          {
            "t": 160.3,
            "v": 519.1
          },
          {
            "t": 163.9,
            "v": 524.2
          },
          {
            "t": 168.3,
            "v": 526.4
          },
          {
            "t": 172.7,
            "v": 526.4
          },
          {
            "t": 177.1,
            "v": 526.4
          },
          {
            "t": 181.4,
            "v": 540.6
          },
          {
            "t": 185.8,
            "v": 540.7
          },
          {
            "t": 189.4,
            "v": 540.8
          },
          {
            "t": 193.8,
            "v": 561.9
          },
          {
            "t": 198.2,
            "v": 562.0
          },
          {
            "t": 202.7,
            "v": 562.1
          },
          {
            "t": 207.1,
            "v": 562.1
          },
          {
            "t": 211.3,
            "v": 562.1
          },
          {
            "t": 215.0,
            "v": 562.2
          },
          {
            "t": 219.5,
            "v": 564.8
          },
          {
            "t": 223.9,
            "v": 579.0
          },
          {
            "t": 228.4,
            "v": 564.0
          },
          {
            "t": 232.8,
            "v": 564.1
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
            "v": 19.5
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
            "v": 42.0
          },
          {
            "t": 74.0,
            "v": 86.0
          },
          {
            "t": 76.0,
            "v": 147.5
          },
          {
            "t": 78.0,
            "v": 244.0
          },
          {
            "t": 80.0,
            "v": 295.0
          },
          {
            "t": 82.0,
            "v": 357.5
          },
          {
            "t": 84.0,
            "v": 397.5
          },
          {
            "t": 86.0,
            "v": 365.5
          },
          {
            "t": 88.0,
            "v": 390.0
          },
          {
            "t": 90.0,
            "v": 274.0
          },
          {
            "t": 92.0,
            "v": 372.5
          },
          {
            "t": 94.0,
            "v": 321.5
          },
          {
            "t": 96.0,
            "v": 383.0
          },
          {
            "t": 98.0,
            "v": 268.0
          },
          {
            "t": 100.0,
            "v": 126.5
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
            "v": 98.0
          },
          {
            "t": 110.0,
            "v": 174.0
          },
          {
            "t": 112.0,
            "v": 218.0
          },
          {
            "t": 114.0,
            "v": 311.0
          },
          {
            "t": 116.0,
            "v": 383.0
          },
          {
            "t": 118.0,
            "v": 380.0
          },
          {
            "t": 120.0,
            "v": 328.0
          },
          {
            "t": 122.0,
            "v": 420.5
          },
          {
            "t": 124.0,
            "v": 236.5
          },
          {
            "t": 126.0,
            "v": 327.0
          },
          {
            "t": 128.0,
            "v": 325.5
          },
          {
            "t": 130.0,
            "v": 281.0
          },
          {
            "t": 132.0,
            "v": 81.0
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
            "v": 1114.5
          },
          {
            "t": 140.0,
            "v": 2201.5
          },
          {
            "t": 142.0,
            "v": 2143.5
          },
          {
            "t": 144.0,
            "v": 2171.5
          },
          {
            "t": 146.0,
            "v": 2170.0
          },
          {
            "t": 148.0,
            "v": 0.0
          },
          {
            "t": 150.0,
            "v": 1880.0
          },
          {
            "t": 152.0,
            "v": 1982.5
          },
          {
            "t": 154.0,
            "v": 2276.5
          },
          {
            "t": 156.0,
            "v": 1815.0
          },
          {
            "t": 158.0,
            "v": 847.0
          },
          {
            "t": 160.0,
            "v": 49.5
          },
          {
            "t": 162.0,
            "v": 240.5
          },
          {
            "t": 164.0,
            "v": 1995.5
          },
          {
            "t": 166.0,
            "v": 2355.0
          },
          {
            "t": 168.0,
            "v": 1557.0
          },
          {
            "t": 170.0,
            "v": 1847.0
          },
          {
            "t": 172.0,
            "v": 2033.0
          },
          {
            "t": 174.0,
            "v": 2318.0
          },
          {
            "t": 176.0,
            "v": 2168.0
          },
          {
            "t": 178.0,
            "v": 1952.5
          },
          {
            "t": 180.0,
            "v": 2234.3
          },
          {
            "t": 182.0,
            "v": 1659.0
          },
          {
            "t": 184.0,
            "v": 36.0
          },
          {
            "t": 186.0,
            "v": 24.5
          },
          {
            "t": 188.0,
            "v": 1325.0
          },
          {
            "t": 190.0,
            "v": 1828.5
          },
          {
            "t": 192.0,
            "v": 2299.5
          },
          {
            "t": 194.0,
            "v": 1767.5
          },
          {
            "t": 196.0,
            "v": 2223.0
          },
          {
            "t": 198.0,
            "v": 1868.5
          },
          {
            "t": 200.0,
            "v": 2462.0
          },
          {
            "t": 202.0,
            "v": 2143.0
          },
          {
            "t": 204.0,
            "v": 2154.5
          },
          {
            "t": 206.0,
            "v": 1571.0
          },
          {
            "t": 208.0,
            "v": 558.5
          },
          {
            "t": 210.0,
            "v": 26.0
          },
          {
            "t": 212.0,
            "v": 369.0
          },
          {
            "t": 214.0,
            "v": 2075.0
          },
          {
            "t": 216.0,
            "v": 2111.5
          },
          {
            "t": 218.0,
            "v": 2257.5
          },
          {
            "t": 220.0,
            "v": 1691.0
          },
          {
            "t": 222.0,
            "v": 2427.5
          },
          {
            "t": 224.0,
            "v": 1737.5
          },
          {
            "t": 226.0,
            "v": 2266.0
          },
          {
            "t": 228.0,
            "v": 1932.5
          },
          {
            "t": 230.0,
            "v": 2185.5
          }
        ],
        "nvcswch": [
          {
            "t": 2.0,
            "v": 1.5
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
            "v": 79.0
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
            "v": 4.5
          },
          {
            "t": 80.0,
            "v": 12.5
          },
          {
            "t": 82.0,
            "v": 14.5
          },
          {
            "t": 84.0,
            "v": 20.0
          },
          {
            "t": 86.0,
            "v": 8.0
          },
          {
            "t": 88.0,
            "v": 11.5
          },
          {
            "t": 90.0,
            "v": 11.5
          },
          {
            "t": 92.0,
            "v": 10.5
          },
          {
            "t": 94.0,
            "v": 12.0
          },
          {
            "t": 96.0,
            "v": 8.5
          },
          {
            "t": 98.0,
            "v": 18.0
          },
          {
            "t": 100.0,
            "v": 1.0
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
            "v": 5.5
          },
          {
            "t": 110.0,
            "v": 7.5
          },
          {
            "t": 112.0,
            "v": 21.5
          },
          {
            "t": 114.0,
            "v": 7.5
          },
          {
            "t": 116.0,
            "v": 19.0
          },
          {
            "t": 118.0,
            "v": 19.5
          },
          {
            "t": 120.0,
            "v": 41.0
          },
          {
            "t": 122.0,
            "v": 15.0
          },
          {
            "t": 124.0,
            "v": 9.0
          },
          {
            "t": 126.0,
            "v": 15.5
          },
          {
            "t": 128.0,
            "v": 17.0
          },
          {
            "t": 130.0,
            "v": 3.0
          },
          {
            "t": 132.0,
            "v": 2.0
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
            "v": 206.0
          },
          {
            "t": 140.0,
            "v": 182.0
          },
          {
            "t": 142.0,
            "v": 257.0
          },
          {
            "t": 144.0,
            "v": 405.0
          },
          {
            "t": 146.0,
            "v": 177.5
          },
          {
            "t": 148.0,
            "v": 0.0
          },
          {
            "t": 150.0,
            "v": 226.0
          },
          {
            "t": 152.0,
            "v": 468.0
          },
          {
            "t": 154.0,
            "v": 113.5
          },
          {
            "t": 156.0,
            "v": 474.0
          },
          {
            "t": 158.0,
            "v": 51.0
          },
          {
            "t": 160.0,
            "v": 0.0
          },
          {
            "t": 162.0,
            "v": 34.0
          },
          {
            "t": 164.0,
            "v": 407.0
          },
          {
            "t": 166.0,
            "v": 162.0
          },
          {
            "t": 168.0,
            "v": 249.5
          },
          {
            "t": 170.0,
            "v": 124.0
          },
          {
            "t": 172.0,
            "v": 254.5
          },
          {
            "t": 174.0,
            "v": 230.5
          },
          {
            "t": 176.0,
            "v": 365.0
          },
          {
            "t": 178.0,
            "v": 257.0
          },
          {
            "t": 180.0,
            "v": 267.7
          },
          {
            "t": 182.0,
            "v": 348.5
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
            "v": 155.5
          },
          {
            "t": 190.0,
            "v": 335.0
          },
          {
            "t": 192.0,
            "v": 279.0
          },
          {
            "t": 194.0,
            "v": 516.5
          },
          {
            "t": 196.0,
            "v": 114.0
          },
          {
            "t": 198.0,
            "v": 390.0
          },
          {
            "t": 200.0,
            "v": 189.0
          },
          {
            "t": 202.0,
            "v": 445.0
          },
          {
            "t": 204.0,
            "v": 241.0
          },
          {
            "t": 206.0,
            "v": 173.0
          },
          {
            "t": 208.0,
            "v": 116.5
          },
          {
            "t": 210.0,
            "v": 0.0
          },
          {
            "t": 212.0,
            "v": 29.0
          },
          {
            "t": 214.0,
            "v": 431.0
          },
          {
            "t": 216.0,
            "v": 311.5
          },
          {
            "t": 218.0,
            "v": 395.5
          },
          {
            "t": 220.0,
            "v": 418.5
          },
          {
            "t": 222.0,
            "v": 176.0
          },
          {
            "t": 224.0,
            "v": 366.0
          },
          {
            "t": 226.0,
            "v": 303.0
          },
          {
            "t": 228.0,
            "v": 516.5
          },
          {
            "t": 230.0,
            "v": 240.5
          }
        ]
      },
      "frps": {
        "cpu": [
          {
            "t": 10.4,
            "v": 19.59
          },
          {
            "t": 15.6,
            "v": 21.45
          },
          {
            "t": 20.6,
            "v": 21.6
          },
          {
            "t": 29.0,
            "v": 15.44
          },
          {
            "t": 33.6,
            "v": 19.78
          },
          {
            "t": 38.2,
            "v": 19.27
          },
          {
            "t": 42.8,
            "v": 19.41
          },
          {
            "t": 47.3,
            "v": 14.47
          },
          {
            "t": 55.0,
            "v": 31.98
          },
          {
            "t": 62.3,
            "v": 27.79
          }
        ],
        "rss": [
          {
            "t": 3.6,
            "v": 34.2
          },
          {
            "t": 10.4,
            "v": 46.2
          },
          {
            "t": 15.6,
            "v": 45.8
          },
          {
            "t": 20.6,
            "v": 51.2
          },
          {
            "t": 25.4,
            "v": 51.2
          },
          {
            "t": 29.0,
            "v": 44.9
          },
          {
            "t": 33.6,
            "v": 41.9
          },
          {
            "t": 38.2,
            "v": 41.6
          },
          {
            "t": 42.8,
            "v": 41.5
          },
          {
            "t": 47.3,
            "v": 41.5
          },
          {
            "t": 51.0,
            "v": 41.5
          },
          {
            "t": 55.0,
            "v": 56.3
          },
          {
            "t": 62.3,
            "v": 77.5
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
            "v": 8.0
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
            "v": 25.4
          },
          {
            "t": 2.0,
            "v": 2512.0
          },
          {
            "t": 4.0,
            "v": 784.1
          },
          {
            "t": 6.0,
            "v": 343.5
          },
          {
            "t": 8.0,
            "v": 1274.0
          },
          {
            "t": 10.0,
            "v": 1938.5
          },
          {
            "t": 12.0,
            "v": 1604.5
          },
          {
            "t": 14.0,
            "v": 2679.0
          },
          {
            "t": 16.0,
            "v": 1700.5
          },
          {
            "t": 18.0,
            "v": 2524.5
          },
          {
            "t": 20.0,
            "v": 1266.5
          },
          {
            "t": 22.0,
            "v": 364.0
          },
          {
            "t": 24.0,
            "v": 40.0
          },
          {
            "t": 26.0,
            "v": 993.0
          },
          {
            "t": 28.0,
            "v": 2899.5
          },
          {
            "t": 30.0,
            "v": 1363.5
          },
          {
            "t": 32.0,
            "v": 2822.5
          },
          {
            "t": 34.0,
            "v": 2180.0
          },
          {
            "t": 36.0,
            "v": 3181.0
          },
          {
            "t": 38.0,
            "v": 2077.5
          },
          {
            "t": 40.0,
            "v": 2134.5
          },
          {
            "t": 42.0,
            "v": 1744.5
          },
          {
            "t": 44.0,
            "v": 2510.5
          },
          {
            "t": 46.0,
            "v": 2061.5
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
            "v": 245.5
          },
          {
            "t": 54.0,
            "v": 389.6
          },
          {
            "t": 56.0,
            "v": 489.0
          },
          {
            "t": 58.0,
            "v": 460.5
          },
          {
            "t": 60.0,
            "v": 473.6
          },
          {
            "t": 62.0,
            "v": 379.5
          },
          {
            "t": 64.0,
            "v": 301.5
          },
          {
            "t": 66.0,
            "v": 290.5
          },
          {
            "t": 68.0,
            "v": 244.0
          },
          {
            "t": 70.0,
            "v": 277.5
          }
        ],
        "nvcswch": [
          {
            "t": 0.0,
            "v": 3.5
          },
          {
            "t": 2.0,
            "v": 269.0
          },
          {
            "t": 4.0,
            "v": 318.4
          },
          {
            "t": 6.0,
            "v": 227.5
          },
          {
            "t": 8.0,
            "v": 388.0
          },
          {
            "t": 10.0,
            "v": 431.5
          },
          {
            "t": 12.0,
            "v": 477.5
          },
          {
            "t": 14.0,
            "v": 367.0
          },
          {
            "t": 16.0,
            "v": 486.5
          },
          {
            "t": 18.0,
            "v": 309.0
          },
          {
            "t": 20.0,
            "v": 456.5
          },
          {
            "t": 22.0,
            "v": 101.5
          },
          {
            "t": 24.0,
            "v": 0.0
          },
          {
            "t": 26.0,
            "v": 49.8
          },
          {
            "t": 28.0,
            "v": 398.5
          },
          {
            "t": 30.0,
            "v": 320.0
          },
          {
            "t": 32.0,
            "v": 268.0
          },
          {
            "t": 34.0,
            "v": 573.5
          },
          {
            "t": 36.0,
            "v": 169.0
          },
          {
            "t": 38.0,
            "v": 516.5
          },
          {
            "t": 40.0,
            "v": 305.5
          },
          {
            "t": 42.0,
            "v": 354.5
          },
          {
            "t": 44.0,
            "v": 422.0
          },
          {
            "t": 46.0,
            "v": 206.5
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
            "v": 128.5
          },
          {
            "t": 54.0,
            "v": 201.5
          },
          {
            "t": 56.0,
            "v": 331.0
          },
          {
            "t": 58.0,
            "v": 297.0
          },
          {
            "t": 60.0,
            "v": 376.6
          },
          {
            "t": 62.0,
            "v": 236.5
          },
          {
            "t": 64.0,
            "v": 286.6
          },
          {
            "t": 66.0,
            "v": 251.5
          },
          {
            "t": 68.0,
            "v": 261.0
          },
          {
            "t": 70.0,
            "v": 304.5
          }
        ]
      },
      "frpc": {
        "cpu": [
          {
            "t": 10.4,
            "v": 17.94
          },
          {
            "t": 15.6,
            "v": 16.16
          },
          {
            "t": 20.6,
            "v": 16.99
          },
          {
            "t": 29.0,
            "v": 10.34
          },
          {
            "t": 33.6,
            "v": 12.12
          },
          {
            "t": 38.2,
            "v": 11.86
          },
          {
            "t": 42.8,
            "v": 12.11
          },
          {
            "t": 47.3,
            "v": 8.33
          },
          {
            "t": 55.0,
            "v": 18.06
          },
          {
            "t": 62.3,
            "v": 17.57
          }
        ],
        "rss": [
          {
            "t": 3.6,
            "v": 34.0
          },
          {
            "t": 10.4,
            "v": 48.3
          },
          {
            "t": 15.6,
            "v": 31.5
          },
          {
            "t": 20.6,
            "v": 44.4
          },
          {
            "t": 25.4,
            "v": 40.9
          },
          {
            "t": 29.0,
            "v": 37.7
          },
          {
            "t": 33.6,
            "v": 37.7
          },
          {
            "t": 38.2,
            "v": 37.7
          },
          {
            "t": 42.8,
            "v": 37.7
          },
          {
            "t": 47.3,
            "v": 37.7
          },
          {
            "t": 51.0,
            "v": 37.7
          },
          {
            "t": 55.0,
            "v": 53.6
          },
          {
            "t": 62.3,
            "v": 104.4
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
            "v": 6.5
          },
          {
            "t": 2.0,
            "v": 2071.0
          },
          {
            "t": 4.0,
            "v": 557.7
          },
          {
            "t": 6.0,
            "v": 375.5
          },
          {
            "t": 8.0,
            "v": 562.5
          },
          {
            "t": 10.0,
            "v": 1700.0
          },
          {
            "t": 12.0,
            "v": 1134.0
          },
          {
            "t": 14.0,
            "v": 2599.5
          },
          {
            "t": 16.0,
            "v": 1123.0
          },
          {
            "t": 18.0,
            "v": 2417.0
          },
          {
            "t": 20.0,
            "v": 1200.5
          },
          {
            "t": 22.0,
            "v": 187.0
          },
          {
            "t": 24.0,
            "v": 2.0
          },
          {
            "t": 26.0,
            "v": 8.5
          },
          {
            "t": 28.0,
            "v": 1589.0
          },
          {
            "t": 30.0,
            "v": 2204.0
          },
          {
            "t": 32.0,
            "v": 1750.5
          },
          {
            "t": 34.0,
            "v": 2295.5
          },
          {
            "t": 36.0,
            "v": 2847.5
          },
          {
            "t": 38.0,
            "v": 2332.5
          },
          {
            "t": 40.0,
            "v": 2181.0
          },
          {
            "t": 42.0,
            "v": 2212.0
          },
          {
            "t": 44.0,
            "v": 555.5
          },
          {
            "t": 46.0,
            "v": 1120.0
          },
          {
            "t": 48.0,
            "v": 0.5
          },
          {
            "t": 50.0,
            "v": 0.0
          },
          {
            "t": 52.0,
            "v": 899.0
          },
          {
            "t": 54.0,
            "v": 1177.6
          },
          {
            "t": 56.0,
            "v": 669.0
          },
          {
            "t": 58.0,
            "v": 787.5
          },
          {
            "t": 60.0,
            "v": 1005.0
          },
          {
            "t": 62.0,
            "v": 598.0
          },
          {
            "t": 64.0,
            "v": 35.3
          },
          {
            "t": 66.0,
            "v": 294.0
          },
          {
            "t": 68.0,
            "v": 137.5
          },
          {
            "t": 70.0,
            "v": 594.0
          }
        ],
        "nvcswch": [
          {
            "t": 0.0,
            "v": 5.5
          },
          {
            "t": 2.0,
            "v": 252.0
          },
          {
            "t": 4.0,
            "v": 257.7
          },
          {
            "t": 6.0,
            "v": 209.5
          },
          {
            "t": 8.0,
            "v": 444.5
          },
          {
            "t": 10.0,
            "v": 291.0
          },
          {
            "t": 12.0,
            "v": 241.5
          },
          {
            "t": 14.0,
            "v": 293.5
          },
          {
            "t": 16.0,
            "v": 358.0
          },
          {
            "t": 18.0,
            "v": 247.5
          },
          {
            "t": 20.0,
            "v": 526.5
          },
          {
            "t": 22.0,
            "v": 116.5
          },
          {
            "t": 24.0,
            "v": 0.5
          },
          {
            "t": 26.0,
            "v": 5.0
          },
          {
            "t": 28.0,
            "v": 304.5
          },
          {
            "t": 30.0,
            "v": 408.0
          },
          {
            "t": 32.0,
            "v": 198.0
          },
          {
            "t": 34.0,
            "v": 553.0
          },
          {
            "t": 36.0,
            "v": 176.5
          },
          {
            "t": 38.0,
            "v": 498.5
          },
          {
            "t": 40.0,
            "v": 282.0
          },
          {
            "t": 42.0,
            "v": 360.0
          },
          {
            "t": 44.0,
            "v": 127.0
          },
          {
            "t": 46.0,
            "v": 98.5
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
            "v": 246.5
          },
          {
            "t": 54.0,
            "v": 299.5
          },
          {
            "t": 56.0,
            "v": 127.5
          },
          {
            "t": 58.0,
            "v": 216.0
          },
          {
            "t": 60.0,
            "v": 232.8
          },
          {
            "t": 62.0,
            "v": 199.5
          },
          {
            "t": 64.0,
            "v": 19.4
          },
          {
            "t": 66.0,
            "v": 200.0
          },
          {
            "t": 68.0,
            "v": 75.5
          },
          {
            "t": 70.0,
            "v": 376.5
          }
        ]
      },
      "other": {
        "cpu": [
          {
            "t": 3.6,
            "v": 3.79
          },
          {
            "t": 10.4,
            "v": 18.41
          },
          {
            "t": 15.6,
            "v": 2.06
          },
          {
            "t": 20.6,
            "v": 7.93
          },
          {
            "t": 25.4,
            "v": 3.13
          },
          {
            "t": 29.0,
            "v": 1.93
          },
          {
            "t": 33.6,
            "v": 1.93
          },
          {
            "t": 38.2,
            "v": 2.33
          },
          {
            "t": 42.8,
            "v": 1.64
          },
          {
            "t": 47.3,
            "v": 2.36
          },
          {
            "t": 51.0,
            "v": 1.69
          },
          {
            "t": 55.0,
            "v": 2.45
          },
          {
            "t": 62.3,
            "v": 1.81
          },
          {
            "t": 70.5,
            "v": 1.63
          },
          {
            "t": 76.1,
            "v": 2.17
          },
          {
            "t": 79.7,
            "v": 1.73
          },
          {
            "t": 83.4,
            "v": 1.77
          },
          {
            "t": 87.1,
            "v": 1.56
          },
          {
            "t": 90.8,
            "v": 1.76
          },
          {
            "t": 94.5,
            "v": 1.9
          },
          {
            "t": 98.2,
            "v": 1.56
          },
          {
            "t": 101.8,
            "v": 1.56
          },
          {
            "t": 105.5,
            "v": 1.72
          },
          {
            "t": 109.1,
            "v": 1.93
          },
          {
            "t": 112.7,
            "v": 1.72
          },
          {
            "t": 116.4,
            "v": 1.64
          },
          {
            "t": 120.1,
            "v": 1.7
          },
          {
            "t": 123.8,
            "v": 1.87
          },
          {
            "t": 127.6,
            "v": 1.61
          },
          {
            "t": 131.3,
            "v": 1.42
          },
          {
            "t": 135.0,
            "v": 1.97
          },
          {
            "t": 138.6,
            "v": 1.59
          },
          {
            "t": 142.8,
            "v": 1.95
          },
          {
            "t": 147.2,
            "v": 1.89
          },
          {
            "t": 151.5,
            "v": 2.06
          },
          {
            "t": 155.9,
            "v": 1.77
          },
          {
            "t": 160.3,
            "v": 1.84
          },
          {
            "t": 163.9,
            "v": 2.01
          },
          {
            "t": 168.3,
            "v": 2.45
          },
          {
            "t": 172.7,
            "v": 1.93
          },
          {
            "t": 177.1,
            "v": 1.94
          },
          {
            "t": 181.4,
            "v": 2.17
          },
          {
            "t": 185.8,
            "v": 1.48
          },
          {
            "t": 189.4,
            "v": 2.0
          },
          {
            "t": 193.8,
            "v": 1.94
          },
          {
            "t": 198.2,
            "v": 1.92
          },
          {
            "t": 202.7,
            "v": 2.02
          },
          {
            "t": 207.1,
            "v": 1.7
          },
          {
            "t": 211.3,
            "v": 1.61
          },
          {
            "t": 215.0,
            "v": 3.28
          },
          {
            "t": 219.5,
            "v": 3.21
          },
          {
            "t": 223.9,
            "v": 1.92
          },
          {
            "t": 228.4,
            "v": 2.53
          },
          {
            "t": 232.8,
            "v": 1.97
          }
        ],
        "rss": [
          {
            "t": 3.6,
            "v": 128.9
          },
          {
            "t": 10.4,
            "v": 129.7
          },
          {
            "t": 15.6,
            "v": 132.1
          },
          {
            "t": 20.6,
            "v": 131.0
          },
          {
            "t": 25.4,
            "v": 131.4
          },
          {
            "t": 29.0,
            "v": 131.8
          },
          {
            "t": 33.6,
            "v": 132.9
          },
          {
            "t": 38.2,
            "v": 133.1
          },
          {
            "t": 42.8,
            "v": 133.1
          },
          {
            "t": 47.3,
            "v": 133.1
          },
          {
            "t": 51.0,
            "v": 133.2
          },
          {
            "t": 55.0,
            "v": 133.1
          },
          {
            "t": 62.3,
            "v": 133.8
          },
          {
            "t": 70.5,
            "v": 133.8
          },
          {
            "t": 76.1,
            "v": 136.8
          },
          {
            "t": 79.7,
            "v": 136.8
          },
          {
            "t": 83.4,
            "v": 136.8
          },
          {
            "t": 87.1,
            "v": 134.5
          },
          {
            "t": 90.8,
            "v": 134.5
          },
          {
            "t": 94.5,
            "v": 134.6
          },
          {
            "t": 98.2,
            "v": 134.6
          },
          {
            "t": 101.8,
            "v": 134.6
          },
          {
            "t": 105.5,
            "v": 134.6
          },
          {
            "t": 109.1,
            "v": 134.6
          },
          {
            "t": 112.7,
            "v": 134.6
          },
          {
            "t": 116.4,
            "v": 134.6
          },
          {
            "t": 120.1,
            "v": 134.6
          },
          {
            "t": 123.8,
            "v": 134.6
          },
          {
            "t": 127.6,
            "v": 134.6
          },
          {
            "t": 131.3,
            "v": 134.6
          },
          {
            "t": 135.0,
            "v": 134.6
          },
          {
            "t": 138.6,
            "v": 134.6
          },
          {
            "t": 142.8,
            "v": 134.6
          },
          {
            "t": 147.2,
            "v": 134.6
          },
          {
            "t": 151.5,
            "v": 134.6
          },
          {
            "t": 155.9,
            "v": 134.7
          },
          {
            "t": 160.3,
            "v": 134.7
          },
          {
            "t": 163.9,
            "v": 134.7
          },
          {
            "t": 168.3,
            "v": 134.7
          },
          {
            "t": 172.7,
            "v": 134.7
          },
          {
            "t": 177.1,
            "v": 134.7
          },
          {
            "t": 181.4,
            "v": 134.7
          },
          {
            "t": 185.8,
            "v": 134.7
          },
          {
            "t": 189.4,
            "v": 134.7
          },
          {
            "t": 193.8,
            "v": 134.7
          },
          {
            "t": 198.2,
            "v": 134.7
          },
          {
            "t": 202.7,
            "v": 134.7
          },
          {
            "t": 207.1,
            "v": 134.7
          },
          {
            "t": 211.3,
            "v": 134.9
          },
          {
            "t": 215.0,
            "v": 134.9
          },
          {
            "t": 219.5,
            "v": 134.9
          },
          {
            "t": 223.9,
            "v": 134.9
          },
          {
            "t": 228.4,
            "v": 135.3
          },
          {
            "t": 232.8,
            "v": 135.2
          }
        ],
        "read_kbs": [
          {
            "t": 0.0,
            "v": 136.5
          },
          {
            "t": 2.0,
            "v": -158.0
          },
          {
            "t": 4.0,
            "v": -158.0
          },
          {
            "t": 6.0,
            "v": -158.0
          },
          {
            "t": 8.0,
            "v": -158.0
          },
          {
            "t": 10.0,
            "v": -158.0
          },
          {
            "t": 12.0,
            "v": -158.0
          },
          {
            "t": 14.0,
            "v": -158.0
          },
          {
            "t": 16.0,
            "v": -158.0
          },
          {
            "t": 18.0,
            "v": -158.0
          },
          {
            "t": 20.0,
            "v": -158.0
          },
          {
            "t": 22.0,
            "v": -158.0
          },
          {
            "t": 24.0,
            "v": -158.0
          },
          {
            "t": 26.0,
            "v": -158.0
          },
          {
            "t": 28.0,
            "v": -158.0
          },
          {
            "t": 30.0,
            "v": -158.0
          },
          {
            "t": 32.0,
            "v": -158.0
          },
          {
            "t": 34.0,
            "v": -158.0
          },
          {
            "t": 36.0,
            "v": -158.0
          },
          {
            "t": 38.0,
            "v": -158.0
          },
          {
            "t": 40.0,
            "v": -158.0
          },
          {
            "t": 42.0,
            "v": -158.0
          },
          {
            "t": 44.0,
            "v": -158.0
          },
          {
            "t": 46.0,
            "v": -158.0
          },
          {
            "t": 48.0,
            "v": -158.0
          },
          {
            "t": 50.0,
            "v": -158.0
          },
          {
            "t": 52.0,
            "v": -158.0
          },
          {
            "t": 54.0,
            "v": -158.0
          },
          {
            "t": 56.0,
            "v": -158.0
          },
          {
            "t": 58.0,
            "v": -158.0
          },
          {
            "t": 60.0,
            "v": -158.0
          },
          {
            "t": 62.0,
            "v": -158.0
          },
          {
            "t": 64.0,
            "v": -158.0
          },
          {
            "t": 66.0,
            "v": -158.0
          },
          {
            "t": 68.0,
            "v": -158.0
          },
          {
            "t": 70.0,
            "v": -158.0
          },
          {
            "t": 72.0,
            "v": -158.0
          },
          {
            "t": 74.0,
            "v": -158.0
          },
          {
            "t": 76.0,
            "v": -158.0
          },
          {
            "t": 78.0,
            "v": -158.0
          },
          {
            "t": 80.0,
            "v": -158.0
          },
          {
            "t": 82.0,
            "v": -158.0
          },
          {
            "t": 84.0,
            "v": -158.0
          },
          {
            "t": 86.0,
            "v": -158.0
          },
          {
            "t": 88.0,
            "v": -158.0
          },
          {
            "t": 90.0,
            "v": -158.0
          },
          {
            "t": 92.0,
            "v": -158.0
          },
          {
            "t": 94.0,
            "v": -158.0
          },
          {
            "t": 96.0,
            "v": -158.0
          },
          {
            "t": 98.0,
            "v": -158.0
          },
          {
            "t": 100.0,
            "v": -158.0
          },
          {
            "t": 102.0,
            "v": -158.0
          },
          {
            "t": 104.0,
            "v": -158.0
          },
          {
            "t": 106.0,
            "v": -158.0
          },
          {
            "t": 108.0,
            "v": -158.0
          },
          {
            "t": 110.0,
            "v": -158.0
          },
          {
            "t": 112.0,
            "v": -158.0
          },
          {
            "t": 114.0,
            "v": -158.0
          },
          {
            "t": 116.0,
            "v": -158.0
          },
          {
            "t": 118.0,
            "v": -158.0
          },
          {
            "t": 120.0,
            "v": -158.0
          },
          {
            "t": 122.0,
            "v": -158.0
          },
          {
            "t": 124.0,
            "v": -158.0
          },
          {
            "t": 126.0,
            "v": -158.0
          },
          {
            "t": 128.0,
            "v": -158.0
          },
          {
            "t": 130.0,
            "v": -158.0
          },
          {
            "t": 132.0,
            "v": -158.0
          },
          {
            "t": 134.0,
            "v": -158.0
          },
          {
            "t": 136.0,
            "v": -158.0
          },
          {
            "t": 138.0,
            "v": -158.0
          },
          {
            "t": 140.0,
            "v": -158.0
          },
          {
            "t": 142.0,
            "v": -158.0
          },
          {
            "t": 144.0,
            "v": -158.0
          },
          {
            "t": 146.0,
            "v": -158.0
          },
          {
            "t": 148.0,
            "v": -158.0
          },
          {
            "t": 150.0,
            "v": -158.0
          },
          {
            "t": 152.0,
            "v": -158.0
          },
          {
            "t": 154.0,
            "v": -158.0
          },
          {
            "t": 156.0,
            "v": -158.0
          },
          {
            "t": 158.0,
            "v": -158.0
          },
          {
            "t": 160.0,
            "v": -158.0
          },
          {
            "t": 162.0,
            "v": -158.0
          },
          {
            "t": 164.0,
            "v": -158.0
          },
          {
            "t": 166.0,
            "v": -158.0
          },
          {
            "t": 168.0,
            "v": -158.0
          },
          {
            "t": 170.0,
            "v": -158.0
          },
          {
            "t": 172.0,
            "v": -158.0
          },
          {
            "t": 174.0,
            "v": -158.0
          },
          {
            "t": 176.0,
            "v": -158.0
          },
          {
            "t": 178.0,
            "v": -158.0
          },
          {
            "t": 180.0,
            "v": -158.0
          },
          {
            "t": 182.0,
            "v": -158.0
          },
          {
            "t": 184.0,
            "v": -158.0
          },
          {
            "t": 186.0,
            "v": -158.0
          },
          {
            "t": 188.0,
            "v": -158.0
          },
          {
            "t": 190.0,
            "v": -158.0
          },
          {
            "t": 192.0,
            "v": -158.0
          },
          {
            "t": 194.0,
            "v": -158.0
          },
          {
            "t": 196.0,
            "v": -158.0
          },
          {
            "t": 198.0,
            "v": -158.0
          },
          {
            "t": 200.0,
            "v": -158.0
          },
          {
            "t": 202.0,
            "v": -158.0
          },
          {
            "t": 204.0,
            "v": -158.0
          },
          {
            "t": 206.0,
            "v": -158.0
          },
          {
            "t": 208.0,
            "v": -158.0
          },
          {
            "t": 210.0,
            "v": -159.0
          },
          {
            "t": 212.0,
            "v": -159.0
          },
          {
            "t": 214.0,
            "v": -159.0
          },
          {
            "t": 216.0,
            "v": -159.0
          },
          {
            "t": 218.0,
            "v": -159.0
          },
          {
            "t": 220.0,
            "v": -159.0
          },
          {
            "t": 222.0,
            "v": -158.0
          },
          {
            "t": 224.0,
            "v": -158.0
          },
          {
            "t": 226.0,
            "v": -158.0
          },
          {
            "t": 228.0,
            "v": -148.0
          },
          {
            "t": 230.0,
            "v": -148.0
          },
          {
            "t": 232.0,
            "v": -148.0
          }
        ],
        "write_kbs": [
          {
            "t": 0.0,
            "v": 38683.8
          },
          {
            "t": 2.0,
            "v": -146.0
          },
          {
            "t": 4.0,
            "v": -132.1
          },
          {
            "t": 6.0,
            "v": -140.0
          },
          {
            "t": 8.0,
            "v": -144.0
          },
          {
            "t": 10.0,
            "v": -148.0
          },
          {
            "t": 12.0,
            "v": -142.0
          },
          {
            "t": 14.0,
            "v": -138.0
          },
          {
            "t": 16.0,
            "v": -132.0
          },
          {
            "t": 18.0,
            "v": -130.0
          },
          {
            "t": 20.0,
            "v": -140.0
          },
          {
            "t": 22.0,
            "v": -142.0
          },
          {
            "t": 24.0,
            "v": -140.0
          },
          {
            "t": 26.0,
            "v": -138.0
          },
          {
            "t": 28.0,
            "v": -142.0
          },
          {
            "t": 30.0,
            "v": -142.0
          },
          {
            "t": 32.0,
            "v": -138.0
          },
          {
            "t": 34.0,
            "v": -138.0
          },
          {
            "t": 36.0,
            "v": -146.0
          },
          {
            "t": 38.0,
            "v": -138.0
          },
          {
            "t": 40.0,
            "v": -142.0
          },
          {
            "t": 42.0,
            "v": -142.0
          },
          {
            "t": 44.0,
            "v": -138.0
          },
          {
            "t": 46.0,
            "v": -142.0
          },
          {
            "t": 48.0,
            "v": -130.0
          },
          {
            "t": 50.0,
            "v": -132.0
          },
          {
            "t": 52.0,
            "v": -138.0
          },
          {
            "t": 54.0,
            "v": -140.1
          },
          {
            "t": 56.0,
            "v": -142.0
          },
          {
            "t": 58.0,
            "v": -144.0
          },
          {
            "t": 60.0,
            "v": -136.0
          },
          {
            "t": 62.0,
            "v": -138.0
          },
          {
            "t": 64.0,
            "v": -144.0
          },
          {
            "t": 66.0,
            "v": -144.1
          },
          {
            "t": 68.0,
            "v": -144.0
          },
          {
            "t": 70.0,
            "v": -132.0
          },
          {
            "t": 72.0,
            "v": -88.0
          },
          {
            "t": 74.0,
            "v": -138.0
          },
          {
            "t": 76.0,
            "v": -128.0
          },
          {
            "t": 78.0,
            "v": -132.0
          },
          {
            "t": 80.0,
            "v": -130.0
          },
          {
            "t": 82.0,
            "v": -136.0
          },
          {
            "t": 84.0,
            "v": -128.0
          },
          {
            "t": 86.0,
            "v": -130.0
          },
          {
            "t": 88.0,
            "v": -134.0
          },
          {
            "t": 90.0,
            "v": -134.0
          },
          {
            "t": 92.0,
            "v": -136.0
          },
          {
            "t": 94.0,
            "v": -132.0
          },
          {
            "t": 96.0,
            "v": -136.0
          },
          {
            "t": 98.0,
            "v": -134.0
          },
          {
            "t": 100.0,
            "v": -136.0
          },
          {
            "t": 102.0,
            "v": -132.0
          },
          {
            "t": 104.0,
            "v": -134.0
          },
          {
            "t": 106.0,
            "v": -138.0
          },
          {
            "t": 108.0,
            "v": -132.0
          },
          {
            "t": 110.0,
            "v": -132.0
          },
          {
            "t": 112.0,
            "v": -132.0
          },
          {
            "t": 114.0,
            "v": -136.0
          },
          {
            "t": 116.0,
            "v": -126.0
          },
          {
            "t": 118.0,
            "v": -136.0
          },
          {
            "t": 120.0,
            "v": -128.0
          },
          {
            "t": 122.0,
            "v": -128.0
          },
          {
            "t": 124.0,
            "v": -134.0
          },
          {
            "t": 126.0,
            "v": -142.0
          },
          {
            "t": 128.0,
            "v": -130.0
          },
          {
            "t": 130.0,
            "v": -134.0
          },
          {
            "t": 132.0,
            "v": -134.0
          },
          {
            "t": 134.0,
            "v": -134.0
          },
          {
            "t": 136.0,
            "v": -136.0
          },
          {
            "t": 138.0,
            "v": -132.0
          },
          {
            "t": 140.0,
            "v": -132.0
          },
          {
            "t": 142.0,
            "v": -138.0
          },
          {
            "t": 144.0,
            "v": -132.0
          },
          {
            "t": 146.0,
            "v": -136.0
          },
          {
            "t": 148.0,
            "v": -134.0
          },
          {
            "t": 150.0,
            "v": -134.0
          },
          {
            "t": 152.0,
            "v": -130.0
          },
          {
            "t": 154.0,
            "v": -136.0
          },
          {
            "t": 156.0,
            "v": -132.0
          },
          {
            "t": 158.0,
            "v": -126.0
          },
          {
            "t": 160.0,
            "v": -130.0
          },
          {
            "t": 162.0,
            "v": -134.0
          },
          {
            "t": 164.0,
            "v": -132.0
          },
          {
            "t": 166.0,
            "v": -134.0
          },
          {
            "t": 168.0,
            "v": -132.0
          },
          {
            "t": 170.0,
            "v": -136.0
          },
          {
            "t": 172.0,
            "v": -132.1
          },
          {
            "t": 174.0,
            "v": -134.0
          },
          {
            "t": 176.0,
            "v": -134.0
          },
          {
            "t": 178.0,
            "v": -136.0
          },
          {
            "t": 180.0,
            "v": -134.0
          },
          {
            "t": 182.0,
            "v": -130.0
          },
          {
            "t": 184.0,
            "v": -134.0
          },
          {
            "t": 186.0,
            "v": -134.0
          },
          {
            "t": 188.0,
            "v": -130.0
          },
          {
            "t": 190.0,
            "v": -130.0
          },
          {
            "t": 192.0,
            "v": -128.0
          },
          {
            "t": 194.0,
            "v": -138.0
          },
          {
            "t": 196.0,
            "v": -136.0
          },
          {
            "t": 198.0,
            "v": -132.0
          },
          {
            "t": 200.0,
            "v": -134.0
          },
          {
            "t": 202.0,
            "v": -136.0
          },
          {
            "t": 204.0,
            "v": -134.0
          },
          {
            "t": 206.0,
            "v": -132.0
          },
          {
            "t": 208.0,
            "v": -132.0
          },
          {
            "t": 210.0,
            "v": -137.0
          },
          {
            "t": 212.0,
            "v": -131.0
          },
          {
            "t": 214.0,
            "v": -135.0
          },
          {
            "t": 216.0,
            "v": -135.0
          },
          {
            "t": 218.0,
            "v": -137.0
          },
          {
            "t": 220.0,
            "v": -131.0
          },
          {
            "t": 222.0,
            "v": -132.0
          },
          {
            "t": 224.0,
            "v": -124.0
          },
          {
            "t": 226.0,
            "v": -136.0
          },
          {
            "t": 228.0,
            "v": -116.0
          },
          {
            "t": 230.0,
            "v": -128.0
          },
          {
            "t": 232.0,
            "v": -80.0
          }
        ],
        "cswch": [
          {
            "t": 0.0,
            "v": 1398.1
          },
          {
            "t": 2.0,
            "v": 475.0
          },
          {
            "t": 4.0,
            "v": 1281.7
          },
          {
            "t": 6.0,
            "v": 1065.0
          },
          {
            "t": 8.0,
            "v": 699.0
          },
          {
            "t": 10.0,
            "v": 1270.0
          },
          {
            "t": 12.0,
            "v": 1190.5
          },
          {
            "t": 14.0,
            "v": 565.0
          },
          {
            "t": 16.0,
            "v": 1481.0
          },
          {
            "t": 18.0,
            "v": 595.0
          },
          {
            "t": 20.0,
            "v": 1079.5
          },
          {
            "t": 22.0,
            "v": 1089.5
          },
          {
            "t": 24.0,
            "v": 590.5
          },
          {
            "t": 26.0,
            "v": 851.3
          },
          {
            "t": 28.0,
            "v": 921.0
          },
          {
            "t": 30.0,
            "v": 1235.0
          },
          {
            "t": 32.0,
            "v": 463.5
          },
          {
            "t": 34.0,
            "v": 1573.5
          },
          {
            "t": 36.0,
            "v": 224.5
          },
          {
            "t": 38.0,
            "v": 1500.0
          },
          {
            "t": 40.0,
            "v": 624.5
          },
          {
            "t": 42.0,
            "v": 1068.0
          },
          {
            "t": 44.0,
            "v": 1061.0
          },
          {
            "t": 46.0,
            "v": 978.5
          },
          {
            "t": 48.0,
            "v": 836.0
          },
          {
            "t": 50.0,
            "v": 857.5
          },
          {
            "t": 52.0,
            "v": 807.5
          },
          {
            "t": 54.0,
            "v": 742.8
          },
          {
            "t": 56.0,
            "v": 1150.0
          },
          {
            "t": 58.0,
            "v": 1188.0
          },
          {
            "t": 60.0,
            "v": 627.9
          },
          {
            "t": 62.0,
            "v": 1235.5
          },
          {
            "t": 64.0,
            "v": 1230.9
          },
          {
            "t": 66.0,
            "v": 1233.0
          },
          {
            "t": 68.0,
            "v": 939.5
          },
          {
            "t": 70.0,
            "v": 1193.5
          },
          {
            "t": 72.0,
            "v": 1371.5
          },
          {
            "t": 74.0,
            "v": 123.5
          },
          {
            "t": 76.0,
            "v": 1355.5
          },
          {
            "t": 78.0,
            "v": 376.0
          },
          {
            "t": 80.0,
            "v": 1158.5
          },
          {
            "t": 82.0,
            "v": 660.5
          },
          {
            "t": 84.0,
            "v": 916.0
          },
          {
            "t": 86.0,
            "v": 896.5
          },
          {
            "t": 88.0,
            "v": 688.5
          },
          {
            "t": 90.0,
            "v": 1126.5
          },
          {
            "t": 92.0,
            "v": 446.0
          },
          {
            "t": 94.0,
            "v": 1357.0
          },
          {
            "t": 96.0,
            "v": 173.5
          },
          {
            "t": 98.0,
            "v": 1433.5
          },
          {
            "t": 100.0,
            "v": 264.0
          },
          {
            "t": 102.0,
            "v": 1205.0
          },
          {
            "t": 104.0,
            "v": 541.0
          },
          {
            "t": 106.0,
            "v": 1243.5
          },
          {
            "t": 108.0,
            "v": 872.5
          },
          {
            "t": 110.0,
            "v": 628.5
          },
          {
            "t": 112.0,
            "v": 1148.5
          },
          {
            "t": 114.0,
            "v": 390.5
          },
          {
            "t": 116.0,
            "v": 1433.0
          },
          {
            "t": 118.0,
            "v": 147.0
          },
          {
            "t": 120.0,
            "v": 1471.0
          },
          {
            "t": 122.0,
            "v": 290.5
          },
          {
            "t": 124.0,
            "v": 1275.5
          },
          {
            "t": 126.0,
            "v": 528.5
          },
          {
            "t": 128.0,
            "v": 1060.0
          },
          {
            "t": 130.0,
            "v": 736.0
          },
          {
            "t": 132.0,
            "v": 775.0
          },
          {
            "t": 134.0,
            "v": 937.5
          },
          {
            "t": 136.0,
            "v": 458.0
          },
          {
            "t": 138.0,
            "v": 1224.0
          },
          {
            "t": 140.0,
            "v": 649.5
          },
          {
            "t": 142.0,
            "v": 1050.0
          },
          {
            "t": 144.0,
            "v": 942.5
          },
          {
            "t": 146.0,
            "v": 775.5
          },
          {
            "t": 148.0,
            "v": 1272.0
          },
          {
            "t": 150.0,
            "v": 504.5
          },
          {
            "t": 152.0,
            "v": 1475.5
          },
          {
            "t": 154.0,
            "v": 213.5
          },
          {
            "t": 156.0,
            "v": 1595.0
          },
          {
            "t": 158.0,
            "v": 243.0
          },
          {
            "t": 160.0,
            "v": 1350.0
          },
          {
            "t": 162.0,
            "v": 246.0
          },
          {
            "t": 164.0,
            "v": 1587.5
          },
          {
            "t": 166.0,
            "v": 592.0
          },
          {
            "t": 168.0,
            "v": 1416.0
          },
          {
            "t": 170.0,
            "v": 559.0
          },
          {
            "t": 172.0,
            "v": 1133.0
          },
          {
            "t": 174.0,
            "v": 846.0
          },
          {
            "t": 176.0,
            "v": 908.5
          },
          {
            "t": 178.0,
            "v": 1091.5
          },
          {
            "t": 180.0,
            "v": 587.1
          },
          {
            "t": 182.0,
            "v": 1386.0
          },
          {
            "t": 184.0,
            "v": 301.0
          },
          {
            "t": 186.0,
            "v": 1115.0
          },
          {
            "t": 188.0,
            "v": 594.0
          },
          {
            "t": 190.0,
            "v": 1399.5
          },
          {
            "t": 192.0,
            "v": 302.0
          },
          {
            "t": 194.0,
            "v": 1573.0
          },
          {
            "t": 196.0,
            "v": 221.5
          },
          {
            "t": 198.0,
            "v": 1472.0
          },
          {
            "t": 200.0,
            "v": 774.5
          },
          {
            "t": 202.0,
            "v": 1142.0
          },
          {
            "t": 204.0,
            "v": 864.0
          },
          {
            "t": 206.0,
            "v": 820.5
          },
          {
            "t": 208.0,
            "v": 987.5
          },
          {
            "t": 210.0,
            "v": 701.0
          },
          {
            "t": 212.0,
            "v": 814.0
          },
          {
            "t": 214.0,
            "v": 883.0
          },
          {
            "t": 216.0,
            "v": 1175.0
          },
          {
            "t": 218.0,
            "v": 531.0
          },
          {
            "t": 220.0,
            "v": 1486.5
          },
          {
            "t": 222.0,
            "v": 236.5
          },
          {
            "t": 224.0,
            "v": 1546.5
          },
          {
            "t": 226.0,
            "v": 696.5
          },
          {
            "t": 228.0,
            "v": 1372.5
          },
          {
            "t": 230.0,
            "v": 690.0
          },
          {
            "t": 232.0,
            "v": 1030.0
          }
        ],
        "nvcswch": [
          {
            "t": 0.0,
            "v": 23.9
          },
          {
            "t": 2.0,
            "v": 102.0
          },
          {
            "t": 4.0,
            "v": 221.9
          },
          {
            "t": 6.0,
            "v": 109.0
          },
          {
            "t": 8.0,
            "v": 68.5
          },
          {
            "t": 10.0,
            "v": 211.5
          },
          {
            "t": 12.0,
            "v": 172.5
          },
          {
            "t": 14.0,
            "v": 96.5
          },
          {
            "t": 16.0,
            "v": 227.5
          },
          {
            "t": 18.0,
            "v": 105.5
          },
          {
            "t": 20.0,
            "v": 160.0
          },
          {
            "t": 22.0,
            "v": 59.0
          },
          {
            "t": 24.0,
            "v": 9.0
          },
          {
            "t": 26.0,
            "v": 13.4
          },
          {
            "t": 28.0,
            "v": 171.0
          },
          {
            "t": 30.0,
            "v": 244.5
          },
          {
            "t": 32.0,
            "v": 115.0
          },
          {
            "t": 34.0,
            "v": 300.0
          },
          {
            "t": 36.0,
            "v": 67.0
          },
          {
            "t": 38.0,
            "v": 224.0
          },
          {
            "t": 40.0,
            "v": 129.5
          },
          {
            "t": 42.0,
            "v": 185.5
          },
          {
            "t": 44.0,
            "v": 162.0
          },
          {
            "t": 46.0,
            "v": 148.0
          },
          {
            "t": 48.0,
            "v": 8.0
          },
          {
            "t": 50.0,
            "v": 14.5
          },
          {
            "t": 52.0,
            "v": 77.0
          },
          {
            "t": 54.0,
            "v": 111.5
          },
          {
            "t": 56.0,
            "v": 134.5
          },
          {
            "t": 58.0,
            "v": 166.5
          },
          {
            "t": 60.0,
            "v": 41.3
          },
          {
            "t": 62.0,
            "v": 126.5
          },
          {
            "t": 64.0,
            "v": 78.6
          },
          {
            "t": 66.0,
            "v": 168.0
          },
          {
            "t": 68.0,
            "v": 72.0
          },
          {
            "t": 70.0,
            "v": 112.0
          },
          {
            "t": 72.0,
            "v": 93.0
          },
          {
            "t": 74.0,
            "v": 0.5
          },
          {
            "t": 76.0,
            "v": 10.0
          },
          {
            "t": 78.0,
            "v": 3.5
          },
          {
            "t": 80.0,
            "v": 22.5
          },
          {
            "t": 82.0,
            "v": 16.0
          },
          {
            "t": 84.0,
            "v": 15.0
          },
          {
            "t": 86.0,
            "v": 17.0
          },
          {
            "t": 88.0,
            "v": 14.0
          },
          {
            "t": 90.0,
            "v": 19.5
          },
          {
            "t": 92.0,
            "v": 6.5
          },
          {
            "t": 94.0,
            "v": 25.0
          },
          {
            "t": 96.0,
            "v": 3.0
          },
          {
            "t": 98.0,
            "v": 22.0
          },
          {
            "t": 100.0,
            "v": 2.5
          },
          {
            "t": 102.0,
            "v": 8.5
          },
          {
            "t": 104.0,
            "v": 7.5
          },
          {
            "t": 106.0,
            "v": 120.0
          },
          {
            "t": 108.0,
            "v": 16.5
          },
          {
            "t": 110.0,
            "v": 10.5
          },
          {
            "t": 112.0,
            "v": 16.5
          },
          {
            "t": 114.0,
            "v": 10.0
          },
          {
            "t": 116.0,
            "v": 33.5
          },
          {
            "t": 118.0,
            "v": 1.0
          },
          {
            "t": 120.0,
            "v": 28.5
          },
          {
            "t": 122.0,
            "v": 10.0
          },
          {
            "t": 124.0,
            "v": 34.5
          },
          {
            "t": 126.0,
            "v": 15.0
          },
          {
            "t": 128.0,
            "v": 24.0
          },
          {
            "t": 130.0,
            "v": 12.0
          },
          {
            "t": 132.0,
            "v": 8.5
          },
          {
            "t": 134.0,
            "v": 13.5
          },
          {
            "t": 136.0,
            "v": 4.5
          },
          {
            "t": 138.0,
            "v": 104.0
          },
          {
            "t": 140.0,
            "v": 71.0
          },
          {
            "t": 142.0,
            "v": 108.5
          },
          {
            "t": 144.0,
            "v": 97.5
          },
          {
            "t": 146.0,
            "v": 80.0
          },
          {
            "t": 148.0,
            "v": 125.5
          },
          {
            "t": 150.0,
            "v": 75.0
          },
          {
            "t": 152.0,
            "v": 148.5
          },
          {
            "t": 154.0,
            "v": 38.0
          },
          {
            "t": 156.0,
            "v": 179.5
          },
          {
            "t": 158.0,
            "v": 30.5
          },
          {
            "t": 160.0,
            "v": 16.5
          },
          {
            "t": 162.0,
            "v": 34.0
          },
          {
            "t": 164.0,
            "v": 186.5
          },
          {
            "t": 166.0,
            "v": 121.0
          },
          {
            "t": 168.0,
            "v": 149.5
          },
          {
            "t": 170.0,
            "v": 62.5
          },
          {
            "t": 172.0,
            "v": 149.0
          },
          {
            "t": 174.0,
            "v": 97.0
          },
          {
            "t": 176.0,
            "v": 105.5
          },
          {
            "t": 178.0,
            "v": 91.5
          },
          {
            "t": 180.0,
            "v": 56.7
          },
          {
            "t": 182.0,
            "v": 144.0
          },
          {
            "t": 184.0,
            "v": 6.0
          },
          {
            "t": 186.0,
            "v": 14.5
          },
          {
            "t": 188.0,
            "v": 64.5
          },
          {
            "t": 190.0,
            "v": 136.5
          },
          {
            "t": 192.0,
            "v": 36.0
          },
          {
            "t": 194.0,
            "v": 188.0
          },
          {
            "t": 196.0,
            "v": 27.5
          },
          {
            "t": 198.0,
            "v": 159.0
          },
          {
            "t": 200.0,
            "v": 86.5
          },
          {
            "t": 202.0,
            "v": 149.0
          },
          {
            "t": 204.0,
            "v": 98.5
          },
          {
            "t": 206.0,
            "v": 89.0
          },
          {
            "t": 208.0,
            "v": 57.5
          },
          {
            "t": 210.0,
            "v": 14.0
          },
          {
            "t": 212.0,
            "v": 27.5
          },
          {
            "t": 214.0,
            "v": 140.0
          },
          {
            "t": 216.0,
            "v": 182.5
          },
          {
            "t": 218.0,
            "v": 143.0
          },
          {
            "t": 220.0,
            "v": 199.5
          },
          {
            "t": 222.0,
            "v": 40.0
          },
          {
            "t": 224.0,
            "v": 187.5
          },
          {
            "t": 226.0,
            "v": 134.0
          },
          {
            "t": 228.0,
            "v": 152.0
          },
          {
            "t": 230.0,
            "v": 78.0
          },
          {
            "t": 232.0,
            "v": 76.5
          }
        ]
      }
    },
    "system": {
      "cpu": [
        {
          "t": 0.0,
          "v": 37.2
        },
        {
          "t": 2.0,
          "v": 47.8
        },
        {
          "t": 4.0,
          "v": 91.1
        },
        {
          "t": 6.0,
          "v": 98.6
        },
        {
          "t": 8.0,
          "v": 93.3
        },
        {
          "t": 10.0,
          "v": 79.9
        },
        {
          "t": 12.0,
          "v": 85.0
        },
        {
          "t": 14.0,
          "v": 67.0
        },
        {
          "t": 16.0,
          "v": 87.0
        },
        {
          "t": 18.0,
          "v": 68.7
        },
        {
          "t": 20.0,
          "v": 90.5
        },
        {
          "t": 22.0,
          "v": 37.5
        },
        {
          "t": 24.0,
          "v": 9.5
        },
        {
          "t": 26.0,
          "v": 28.5
        },
        {
          "t": 28.0,
          "v": 68.0
        },
        {
          "t": 30.0,
          "v": 77.2
        },
        {
          "t": 32.0,
          "v": 58.9
        },
        {
          "t": 34.0,
          "v": 80.9
        },
        {
          "t": 36.0,
          "v": 53.2
        },
        {
          "t": 38.0,
          "v": 79.0
        },
        {
          "t": 40.0,
          "v": 64.3
        },
        {
          "t": 42.0,
          "v": 71.4
        },
        {
          "t": 44.0,
          "v": 72.7
        },
        {
          "t": 46.0,
          "v": 47.4
        },
        {
          "t": 48.0,
          "v": 15.4
        },
        {
          "t": 50.0,
          "v": 14.7
        },
        {
          "t": 52.0,
          "v": 78.9
        },
        {
          "t": 54.0,
          "v": 93.8
        },
        {
          "t": 56.0,
          "v": 97.0
        },
        {
          "t": 58.0,
          "v": 98.4
        },
        {
          "t": 60.0,
          "v": 95.5
        },
        {
          "t": 62.0,
          "v": 98.4
        },
        {
          "t": 64.0,
          "v": 98.7
        },
        {
          "t": 66.0,
          "v": 99.0
        },
        {
          "t": 68.0,
          "v": 99.0
        },
        {
          "t": 70.0,
          "v": 99.0
        },
        {
          "t": 72.0,
          "v": 82.4
        },
        {
          "t": 74.0,
          "v": 2.0
        },
        {
          "t": 76.0,
          "v": 24.7
        },
        {
          "t": 78.0,
          "v": 6.4
        },
        {
          "t": 80.0,
          "v": 23.8
        },
        {
          "t": 82.0,
          "v": 13.2
        },
        {
          "t": 84.0,
          "v": 20.7
        },
        {
          "t": 86.0,
          "v": 18.5
        },
        {
          "t": 88.0,
          "v": 17.0
        },
        {
          "t": 90.0,
          "v": 23.1
        },
        {
          "t": 92.0,
          "v": 11.5
        },
        {
          "t": 94.0,
          "v": 28.0
        },
        {
          "t": 96.0,
          "v": 6.1
        },
        {
          "t": 98.0,
          "v": 31.5
        },
        {
          "t": 100.0,
          "v": 4.4
        },
        {
          "t": 102.0,
          "v": 22.7
        },
        {
          "t": 104.0,
          "v": 8.3
        },
        {
          "t": 106.0,
          "v": 17.0
        },
        {
          "t": 108.0,
          "v": 15.2
        },
        {
          "t": 110.0,
          "v": 14.0
        },
        {
          "t": 112.0,
          "v": 23.3
        },
        {
          "t": 114.0,
          "v": 10.2
        },
        {
          "t": 116.0,
          "v": 29.6
        },
        {
          "t": 118.0,
          "v": 7.2
        },
        {
          "t": 120.0,
          "v": 34.6
        },
        {
          "t": 122.0,
          "v": 8.3
        },
        {
          "t": 124.0,
          "v": 29.3
        },
        {
          "t": 126.0,
          "v": 12.8
        },
        {
          "t": 128.0,
          "v": 23.6
        },
        {
          "t": 130.0,
          "v": 14.6
        },
        {
          "t": 132.0,
          "v": 15.4
        },
        {
          "t": 134.0,
          "v": 15.8
        },
        {
          "t": 136.0,
          "v": 8.5
        },
        {
          "t": 138.0,
          "v": 51.9
        },
        {
          "t": 140.0,
          "v": 64.4
        },
        {
          "t": 142.0,
          "v": 69.1
        },
        {
          "t": 144.0,
          "v": 69.1
        },
        {
          "t": 146.0,
          "v": 63.6
        },
        {
          "t": 148.0,
          "v": 72.1
        },
        {
          "t": 150.0,
          "v": 61.3
        },
        {
          "t": 152.0,
          "v": 76.6
        },
        {
          "t": 154.0,
          "v": 56.3
        },
        {
          "t": 156.0,
          "v": 78.6
        },
        {
          "t": 158.0,
          "v": 22.2
        },
        {
          "t": 160.0,
          "v": 24.0
        },
        {
          "t": 162.0,
          "v": 7.7
        },
        {
          "t": 164.0,
          "v": 78.2
        },
        {
          "t": 166.0,
          "v": 58.1
        },
        {
          "t": 168.0,
          "v": 75.5
        },
        {
          "t": 170.0,
          "v": 62.7
        },
        {
          "t": 172.0,
          "v": 69.7
        },
        {
          "t": 174.0,
          "v": 66.3
        },
        {
          "t": 176.0,
          "v": 67.0
        },
        {
          "t": 178.0,
          "v": 71.0
        },
        {
          "t": 180.0,
          "v": 62.1
        },
        {
          "t": 182.0,
          "v": 69.7
        },
        {
          "t": 184.0,
          "v": 3.4
        },
        {
          "t": 186.0,
          "v": 20.9
        },
        {
          "t": 188.0,
          "v": 38.7
        },
        {
          "t": 190.0,
          "v": 76.6
        },
        {
          "t": 192.0,
          "v": 59.0
        },
        {
          "t": 194.0,
          "v": 78.6
        },
        {
          "t": 196.0,
          "v": 58.0
        },
        {
          "t": 198.0,
          "v": 77.5
        },
        {
          "t": 200.0,
          "v": 61.5
        },
        {
          "t": 202.0,
          "v": 72.2
        },
        {
          "t": 204.0,
          "v": 68.8
        },
        {
          "t": 206.0,
          "v": 67.5
        },
        {
          "t": 208.0,
          "v": 39.3
        },
        {
          "t": 210.0,
          "v": 11.5
        },
        {
          "t": 212.0,
          "v": 20.8
        },
        {
          "t": 214.0,
          "v": 69.0
        },
        {
          "t": 216.0,
          "v": 71.8
        },
        {
          "t": 218.0,
          "v": 64.2
        },
        {
          "t": 220.0,
          "v": 78.3
        },
        {
          "t": 222.0,
          "v": 59.5
        },
        {
          "t": 224.0,
          "v": 78.9
        },
        {
          "t": 226.0,
          "v": 62.4
        },
        {
          "t": 228.0,
          "v": 75.8
        },
        {
          "t": 230.0,
          "v": 66.0
        },
        {
          "t": 232.0,
          "v": 68.4
        }
      ],
      "cpu_usr": [
        {
          "t": 0.0,
          "v": 14.2
        },
        {
          "t": 2.0,
          "v": 25.7
        },
        {
          "t": 4.0,
          "v": 52.7
        },
        {
          "t": 6.0,
          "v": 65.1
        },
        {
          "t": 8.0,
          "v": 64.7
        },
        {
          "t": 10.0,
          "v": 40.0
        },
        {
          "t": 12.0,
          "v": 44.4
        },
        {
          "t": 14.0,
          "v": 36.7
        },
        {
          "t": 16.0,
          "v": 43.4
        },
        {
          "t": 18.0,
          "v": 36.8
        },
        {
          "t": 20.0,
          "v": 54.6
        },
        {
          "t": 22.0,
          "v": 16.6
        },
        {
          "t": 24.0,
          "v": 1.9
        },
        {
          "t": 26.0,
          "v": 10.1
        },
        {
          "t": 28.0,
          "v": 32.6
        },
        {
          "t": 30.0,
          "v": 37.5
        },
        {
          "t": 32.0,
          "v": 30.4
        },
        {
          "t": 34.0,
          "v": 36.6
        },
        {
          "t": 36.0,
          "v": 27.7
        },
        {
          "t": 38.0,
          "v": 36.4
        },
        {
          "t": 40.0,
          "v": 32.1
        },
        {
          "t": 42.0,
          "v": 33.4
        },
        {
          "t": 44.0,
          "v": 33.9
        },
        {
          "t": 46.0,
          "v": 21.8
        },
        {
          "t": 48.0,
          "v": 3.3
        },
        {
          "t": 50.0,
          "v": 2.8
        },
        {
          "t": 52.0,
          "v": 49.9
        },
        {
          "t": 54.0,
          "v": 63.1
        },
        {
          "t": 56.0,
          "v": 65.3
        },
        {
          "t": 58.0,
          "v": 63.7
        },
        {
          "t": 60.0,
          "v": 67.9
        },
        {
          "t": 62.0,
          "v": 64.5
        },
        {
          "t": 64.0,
          "v": 62.0
        },
        {
          "t": 66.0,
          "v": 61.8
        },
        {
          "t": 68.0,
          "v": 66.3
        },
        {
          "t": 70.0,
          "v": 63.0
        },
        {
          "t": 72.0,
          "v": 53.2
        },
        {
          "t": 74.0,
          "v": 0.5
        },
        {
          "t": 76.0,
          "v": 5.2
        },
        {
          "t": 78.0,
          "v": 1.7
        },
        {
          "t": 80.0,
          "v": 5.4
        },
        {
          "t": 82.0,
          "v": 3.9
        },
        {
          "t": 84.0,
          "v": 5.2
        },
        {
          "t": 86.0,
          "v": 4.8
        },
        {
          "t": 88.0,
          "v": 5.0
        },
        {
          "t": 90.0,
          "v": 5.9
        },
        {
          "t": 92.0,
          "v": 3.4
        },
        {
          "t": 94.0,
          "v": 6.7
        },
        {
          "t": 96.0,
          "v": 2.5
        },
        {
          "t": 98.0,
          "v": 9.5
        },
        {
          "t": 100.0,
          "v": 1.2
        },
        {
          "t": 102.0,
          "v": 5.1
        },
        {
          "t": 104.0,
          "v": 1.8
        },
        {
          "t": 106.0,
          "v": 3.3
        },
        {
          "t": 108.0,
          "v": 3.7
        },
        {
          "t": 110.0,
          "v": 3.3
        },
        {
          "t": 112.0,
          "v": 6.0
        },
        {
          "t": 114.0,
          "v": 3.1
        },
        {
          "t": 116.0,
          "v": 7.9
        },
        {
          "t": 118.0,
          "v": 3.4
        },
        {
          "t": 120.0,
          "v": 11.2
        },
        {
          "t": 122.0,
          "v": 3.5
        },
        {
          "t": 124.0,
          "v": 7.6
        },
        {
          "t": 126.0,
          "v": 4.8
        },
        {
          "t": 128.0,
          "v": 6.1
        },
        {
          "t": 130.0,
          "v": 3.8
        },
        {
          "t": 132.0,
          "v": 3.4
        },
        {
          "t": 134.0,
          "v": 3.0
        },
        {
          "t": 136.0,
          "v": 1.8
        },
        {
          "t": 138.0,
          "v": 23.6
        },
        {
          "t": 140.0,
          "v": 33.4
        },
        {
          "t": 142.0,
          "v": 33.4
        },
        {
          "t": 144.0,
          "v": 33.1
        },
        {
          "t": 146.0,
          "v": 30.3
        },
        {
          "t": 148.0,
          "v": 34.5
        },
        {
          "t": 150.0,
          "v": 32.3
        },
        {
          "t": 152.0,
          "v": 35.5
        },
        {
          "t": 154.0,
          "v": 30.9
        },
        {
          "t": 156.0,
          "v": 36.5
        },
        {
          "t": 158.0,
          "v": 11.0
        },
        {
          "t": 160.0,
          "v": 4.8
        },
        {
          "t": 162.0,
          "v": 3.4
        },
        {
          "t": 164.0,
          "v": 36.2
        },
        {
          "t": 166.0,
          "v": 30.5
        },
        {
          "t": 168.0,
          "v": 36.5
        },
        {
          "t": 170.0,
          "v": 32.7
        },
        {
          "t": 172.0,
          "v": 32.7
        },
        {
          "t": 174.0,
          "v": 32.0
        },
        {
          "t": 176.0,
          "v": 33.6
        },
        {
          "t": 178.0,
          "v": 35.0
        },
        {
          "t": 180.0,
          "v": 32.7
        },
        {
          "t": 182.0,
          "v": 32.5
        },
        {
          "t": 184.0,
          "v": 0.5
        },
        {
          "t": 186.0,
          "v": 3.9
        },
        {
          "t": 188.0,
          "v": 17.8
        },
        {
          "t": 190.0,
          "v": 35.2
        },
        {
          "t": 192.0,
          "v": 32.6
        },
        {
          "t": 194.0,
          "v": 35.4
        },
        {
          "t": 196.0,
          "v": 32.5
        },
        {
          "t": 198.0,
          "v": 35.4
        },
        {
          "t": 200.0,
          "v": 29.9
        },
        {
          "t": 202.0,
          "v": 33.8
        },
        {
          "t": 204.0,
          "v": 34.7
        },
        {
          "t": 206.0,
          "v": 34.0
        },
        {
          "t": 208.0,
          "v": 17.3
        },
        {
          "t": 210.0,
          "v": 2.9
        },
        {
          "t": 212.0,
          "v": 6.8
        },
        {
          "t": 214.0,
          "v": 37.9
        },
        {
          "t": 216.0,
          "v": 34.2
        },
        {
          "t": 218.0,
          "v": 34.9
        },
        {
          "t": 220.0,
          "v": 37.3
        },
        {
          "t": 222.0,
          "v": 33.6
        },
        {
          "t": 224.0,
          "v": 37.7
        },
        {
          "t": 226.0,
          "v": 32.9
        },
        {
          "t": 228.0,
          "v": 35.8
        },
        {
          "t": 230.0,
          "v": 33.6
        },
        {
          "t": 232.0,
          "v": 34.7
        }
      ],
      "cpu_sys": [
        {
          "t": 0.0,
          "v": 21.3
        },
        {
          "t": 2.0,
          "v": 17.9
        },
        {
          "t": 4.0,
          "v": 32.7
        },
        {
          "t": 6.0,
          "v": 27.0
        },
        {
          "t": 8.0,
          "v": 21.6
        },
        {
          "t": 10.0,
          "v": 33.9
        },
        {
          "t": 12.0,
          "v": 33.9
        },
        {
          "t": 14.0,
          "v": 24.3
        },
        {
          "t": 16.0,
          "v": 37.7
        },
        {
          "t": 18.0,
          "v": 26.1
        },
        {
          "t": 20.0,
          "v": 29.5
        },
        {
          "t": 22.0,
          "v": 18.7
        },
        {
          "t": 24.0,
          "v": 7.7
        },
        {
          "t": 26.0,
          "v": 16.7
        },
        {
          "t": 28.0,
          "v": 29.4
        },
        {
          "t": 30.0,
          "v": 33.7
        },
        {
          "t": 32.0,
          "v": 22.1
        },
        {
          "t": 34.0,
          "v": 38.7
        },
        {
          "t": 36.0,
          "v": 18.1
        },
        {
          "t": 38.0,
          "v": 36.8
        },
        {
          "t": 40.0,
          "v": 26.0
        },
        {
          "t": 42.0,
          "v": 32.0
        },
        {
          "t": 44.0,
          "v": 33.1
        },
        {
          "t": 46.0,
          "v": 21.2
        },
        {
          "t": 48.0,
          "v": 12.0
        },
        {
          "t": 50.0,
          "v": 12.0
        },
        {
          "t": 52.0,
          "v": 23.8
        },
        {
          "t": 54.0,
          "v": 24.3
        },
        {
          "t": 56.0,
          "v": 26.2
        },
        {
          "t": 58.0,
          "v": 27.7
        },
        {
          "t": 60.0,
          "v": 20.9
        },
        {
          "t": 62.0,
          "v": 26.8
        },
        {
          "t": 64.0,
          "v": 28.3
        },
        {
          "t": 66.0,
          "v": 30.1
        },
        {
          "t": 68.0,
          "v": 24.2
        },
        {
          "t": 70.0,
          "v": 27.6
        },
        {
          "t": 72.0,
          "v": 26.5
        },
        {
          "t": 74.0,
          "v": 1.5
        },
        {
          "t": 76.0,
          "v": 19.5
        },
        {
          "t": 78.0,
          "v": 4.5
        },
        {
          "t": 80.0,
          "v": 18.0
        },
        {
          "t": 82.0,
          "v": 8.9
        },
        {
          "t": 84.0,
          "v": 15.0
        },
        {
          "t": 86.0,
          "v": 13.2
        },
        {
          "t": 88.0,
          "v": 11.6
        },
        {
          "t": 90.0,
          "v": 16.6
        },
        {
          "t": 92.0,
          "v": 7.5
        },
        {
          "t": 94.0,
          "v": 20.9
        },
        {
          "t": 96.0,
          "v": 3.5
        },
        {
          "t": 98.0,
          "v": 21.4
        },
        {
          "t": 100.0,
          "v": 3.1
        },
        {
          "t": 102.0,
          "v": 17.5
        },
        {
          "t": 104.0,
          "v": 6.6
        },
        {
          "t": 106.0,
          "v": 13.8
        },
        {
          "t": 108.0,
          "v": 11.4
        },
        {
          "t": 110.0,
          "v": 10.4
        },
        {
          "t": 112.0,
          "v": 17.1
        },
        {
          "t": 114.0,
          "v": 6.7
        },
        {
          "t": 116.0,
          "v": 21.1
        },
        {
          "t": 118.0,
          "v": 3.5
        },
        {
          "t": 120.0,
          "v": 23.0
        },
        {
          "t": 122.0,
          "v": 4.6
        },
        {
          "t": 124.0,
          "v": 21.1
        },
        {
          "t": 126.0,
          "v": 7.5
        },
        {
          "t": 128.0,
          "v": 17.1
        },
        {
          "t": 130.0,
          "v": 10.6
        },
        {
          "t": 132.0,
          "v": 11.8
        },
        {
          "t": 134.0,
          "v": 12.8
        },
        {
          "t": 136.0,
          "v": 6.8
        },
        {
          "t": 138.0,
          "v": 25.8
        },
        {
          "t": 140.0,
          "v": 26.5
        },
        {
          "t": 142.0,
          "v": 31.1
        },
        {
          "t": 144.0,
          "v": 31.5
        },
        {
          "t": 146.0,
          "v": 28.2
        },
        {
          "t": 148.0,
          "v": 33.2
        },
        {
          "t": 150.0,
          "v": 24.0
        },
        {
          "t": 152.0,
          "v": 36.6
        },
        {
          "t": 154.0,
          "v": 19.8
        },
        {
          "t": 156.0,
          "v": 38.5
        },
        {
          "t": 158.0,
          "v": 9.4
        },
        {
          "t": 160.0,
          "v": 19.1
        },
        {
          "t": 162.0,
          "v": 3.7
        },
        {
          "t": 164.0,
          "v": 38.4
        },
        {
          "t": 166.0,
          "v": 22.5
        },
        {
          "t": 168.0,
          "v": 35.3
        },
        {
          "t": 170.0,
          "v": 25.9
        },
        {
          "t": 172.0,
          "v": 32.5
        },
        {
          "t": 174.0,
          "v": 29.9
        },
        {
          "t": 176.0,
          "v": 28.9
        },
        {
          "t": 178.0,
          "v": 31.7
        },
        {
          "t": 180.0,
          "v": 24.8
        },
        {
          "t": 182.0,
          "v": 33.5
        },
        {
          "t": 184.0,
          "v": 2.9
        },
        {
          "t": 186.0,
          "v": 17.0
        },
        {
          "t": 188.0,
          "v": 18.1
        },
        {
          "t": 190.0,
          "v": 37.5
        },
        {
          "t": 192.0,
          "v": 21.9
        },
        {
          "t": 194.0,
          "v": 38.7
        },
        {
          "t": 196.0,
          "v": 20.3
        },
        {
          "t": 198.0,
          "v": 38.2
        },
        {
          "t": 200.0,
          "v": 26.8
        },
        {
          "t": 202.0,
          "v": 33.7
        },
        {
          "t": 204.0,
          "v": 29.6
        },
        {
          "t": 206.0,
          "v": 29.0
        },
        {
          "t": 208.0,
          "v": 20.2
        },
        {
          "t": 210.0,
          "v": 8.6
        },
        {
          "t": 212.0,
          "v": 13.5
        },
        {
          "t": 214.0,
          "v": 26.6
        },
        {
          "t": 216.0,
          "v": 33.0
        },
        {
          "t": 218.0,
          "v": 24.5
        },
        {
          "t": 220.0,
          "v": 36.9
        },
        {
          "t": 222.0,
          "v": 20.9
        },
        {
          "t": 224.0,
          "v": 37.4
        },
        {
          "t": 226.0,
          "v": 24.3
        },
        {
          "t": 228.0,
          "v": 35.1
        },
        {
          "t": 230.0,
          "v": 26.9
        },
        {
          "t": 232.0,
          "v": 29.8
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
          "v": 0.2
        },
        {
          "t": 2.0,
          "v": 4.2
        },
        {
          "t": 4.0,
          "v": 5.7
        },
        {
          "t": 6.0,
          "v": 6.6
        },
        {
          "t": 8.0,
          "v": 7.0
        },
        {
          "t": 10.0,
          "v": 6.0
        },
        {
          "t": 12.0,
          "v": 6.8
        },
        {
          "t": 14.0,
          "v": 6.0
        },
        {
          "t": 16.0,
          "v": 5.7
        },
        {
          "t": 18.0,
          "v": 5.7
        },
        {
          "t": 20.0,
          "v": 6.3
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
          "v": 6.1
        },
        {
          "t": 30.0,
          "v": 6.0
        },
        {
          "t": 32.0,
          "v": 6.5
        },
        {
          "t": 34.0,
          "v": 5.6
        },
        {
          "t": 36.0,
          "v": 7.3
        },
        {
          "t": 38.0,
          "v": 5.8
        },
        {
          "t": 40.0,
          "v": 6.3
        },
        {
          "t": 42.0,
          "v": 6.0
        },
        {
          "t": 44.0,
          "v": 5.7
        },
        {
          "t": 46.0,
          "v": 4.4
        },
        {
          "t": 48.0,
          "v": 0.1
        },
        {
          "t": 50.0,
          "v": 0.0
        },
        {
          "t": 52.0,
          "v": 5.3
        },
        {
          "t": 54.0,
          "v": 6.4
        },
        {
          "t": 56.0,
          "v": 5.4
        },
        {
          "t": 58.0,
          "v": 6.9
        },
        {
          "t": 60.0,
          "v": 6.8
        },
        {
          "t": 62.0,
          "v": 7.2
        },
        {
          "t": 64.0,
          "v": 8.4
        },
        {
          "t": 66.0,
          "v": 7.1
        },
        {
          "t": 68.0,
          "v": 8.4
        },
        {
          "t": 70.0,
          "v": 8.4
        },
        {
          "t": 72.0,
          "v": 2.8
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
          "v": 0.3
        },
        {
          "t": 80.0,
          "v": 0.4
        },
        {
          "t": 82.0,
          "v": 0.4
        },
        {
          "t": 84.0,
          "v": 0.5
        },
        {
          "t": 86.0,
          "v": 0.5
        },
        {
          "t": 88.0,
          "v": 0.4
        },
        {
          "t": 90.0,
          "v": 0.5
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
          "v": 0.1
        },
        {
          "t": 98.0,
          "v": 0.6
        },
        {
          "t": 100.0,
          "v": 0.1
        },
        {
          "t": 102.0,
          "v": 0.1
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
          "v": 0.2
        },
        {
          "t": 112.0,
          "v": 0.2
        },
        {
          "t": 114.0,
          "v": 0.4
        },
        {
          "t": 116.0,
          "v": 0.7
        },
        {
          "t": 118.0,
          "v": 0.4
        },
        {
          "t": 120.0,
          "v": 0.5
        },
        {
          "t": 122.0,
          "v": 0.3
        },
        {
          "t": 124.0,
          "v": 0.5
        },
        {
          "t": 126.0,
          "v": 0.5
        },
        {
          "t": 128.0,
          "v": 0.4
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
          "v": 0.0
        },
        {
          "t": 136.0,
          "v": 0.0
        },
        {
          "t": 138.0,
          "v": 2.5
        },
        {
          "t": 140.0,
          "v": 4.5
        },
        {
          "t": 142.0,
          "v": 4.7
        },
        {
          "t": 144.0,
          "v": 4.5
        },
        {
          "t": 146.0,
          "v": 5.1
        },
        {
          "t": 148.0,
          "v": 4.3
        },
        {
          "t": 150.0,
          "v": 5.0
        },
        {
          "t": 152.0,
          "v": 4.6
        },
        {
          "t": 154.0,
          "v": 5.5
        },
        {
          "t": 156.0,
          "v": 3.7
        },
        {
          "t": 158.0,
          "v": 1.7
        },
        {
          "t": 160.0,
          "v": 0.0
        },
        {
          "t": 162.0,
          "v": 0.6
        },
        {
          "t": 164.0,
          "v": 3.6
        },
        {
          "t": 166.0,
          "v": 5.1
        },
        {
          "t": 168.0,
          "v": 3.7
        },
        {
          "t": 170.0,
          "v": 4.0
        },
        {
          "t": 172.0,
          "v": 4.5
        },
        {
          "t": 174.0,
          "v": 4.4
        },
        {
          "t": 176.0,
          "v": 4.6
        },
        {
          "t": 178.0,
          "v": 4.3
        },
        {
          "t": 180.0,
          "v": 4.5
        },
        {
          "t": 182.0,
          "v": 3.7
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
          "v": 2.9
        },
        {
          "t": 190.0,
          "v": 4.0
        },
        {
          "t": 192.0,
          "v": 4.6
        },
        {
          "t": 194.0,
          "v": 4.5
        },
        {
          "t": 196.0,
          "v": 5.2
        },
        {
          "t": 198.0,
          "v": 4.0
        },
        {
          "t": 200.0,
          "v": 4.8
        },
        {
          "t": 202.0,
          "v": 4.6
        },
        {
          "t": 204.0,
          "v": 4.5
        },
        {
          "t": 206.0,
          "v": 4.5
        },
        {
          "t": 208.0,
          "v": 1.8
        },
        {
          "t": 210.0,
          "v": 0.0
        },
        {
          "t": 212.0,
          "v": 0.5
        },
        {
          "t": 214.0,
          "v": 4.5
        },
        {
          "t": 216.0,
          "v": 4.7
        },
        {
          "t": 218.0,
          "v": 4.8
        },
        {
          "t": 220.0,
          "v": 4.1
        },
        {
          "t": 222.0,
          "v": 5.0
        },
        {
          "t": 224.0,
          "v": 3.9
        },
        {
          "t": 226.0,
          "v": 5.2
        },
        {
          "t": 228.0,
          "v": 4.9
        },
        {
          "t": 230.0,
          "v": 5.5
        },
        {
          "t": 232.0,
          "v": 3.9
        }
      ],
      "cpu_iowait": [
        {
          "t": 0.0,
          "v": 1.4
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
          "v": 0.3
        },
        {
          "t": 18.0,
          "v": 0.1
        },
        {
          "t": 20.0,
          "v": 0.0
        },
        {
          "t": 22.0,
          "v": 0.4
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
      "mem_total_mb": 15994
    },
    "network": {
      "rx_kbs": [
        {
          "t": 0.0,
          "v": 6236.5
        },
        {
          "t": 2.0,
          "v": 5.5
        },
        {
          "t": 4.0,
          "v": 1.4
        },
        {
          "t": 6.0,
          "v": 0.6
        },
        {
          "t": 8.0,
          "v": 0.1
        },
        {
          "t": 10.0,
          "v": 0.1
        },
        {
          "t": 12.0,
          "v": 1.9
        },
        {
          "t": 14.0,
          "v": 0.4
        },
        {
          "t": 16.0,
          "v": 0.1
        },
        {
          "t": 18.0,
          "v": 1.9
        },
        {
          "t": 20.0,
          "v": 0.1
        },
        {
          "t": 22.0,
          "v": 0.1
        },
        {
          "t": 24.0,
          "v": 1.9
        },
        {
          "t": 26.0,
          "v": 0.1
        },
        {
          "t": 28.0,
          "v": 0.1
        },
        {
          "t": 30.0,
          "v": 3.9
        },
        {
          "t": 32.0,
          "v": 0.7
        },
        {
          "t": 34.0,
          "v": 0.1
        },
        {
          "t": 36.0,
          "v": 0.1
        },
        {
          "t": 38.0,
          "v": 2.3
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
          "v": 1.7
        },
        {
          "t": 46.0,
          "v": 0.8
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
          "v": 1.5
        },
        {
          "t": 60.0,
          "v": 0.7
        },
        {
          "t": 62.0,
          "v": 0.2
        },
        {
          "t": 64.0,
          "v": 0.3
        },
        {
          "t": 66.0,
          "v": 1.3
        },
        {
          "t": 68.0,
          "v": 0.6
        },
        {
          "t": 70.0,
          "v": 0.1
        },
        {
          "t": 72.0,
          "v": 21.5
        },
        {
          "t": 74.0,
          "v": 2.2
        },
        {
          "t": 76.0,
          "v": 0.2
        },
        {
          "t": 78.0,
          "v": 0.1
        },
        {
          "t": 80.0,
          "v": 1.4
        },
        {
          "t": 82.0,
          "v": 0.7
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
          "v": 2.1
        },
        {
          "t": 90.0,
          "v": 2.7
        },
        {
          "t": 92.0,
          "v": 0.1
        },
        {
          "t": 94.0,
          "v": 2.0
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
          "v": 2.2
        },
        {
          "t": 102.0,
          "v": 0.1
        },
        {
          "t": 104.0,
          "v": 0.3
        },
        {
          "t": 106.0,
          "v": 1.6
        },
        {
          "t": 108.0,
          "v": 0.9
        },
        {
          "t": 110.0,
          "v": 0.1
        },
        {
          "t": 112.0,
          "v": 0.1
        },
        {
          "t": 114.0,
          "v": 2.0
        },
        {
          "t": 116.0,
          "v": 0.1
        },
        {
          "t": 118.0,
          "v": 0.1
        },
        {
          "t": 120.0,
          "v": 2.0
        },
        {
          "t": 122.0,
          "v": 0.1
        },
        {
          "t": 124.0,
          "v": 0.1
        },
        {
          "t": 126.0,
          "v": 1.9
        },
        {
          "t": 128.0,
          "v": 0.1
        },
        {
          "t": 130.0,
          "v": 0.1
        },
        {
          "t": 132.0,
          "v": 2.0
        },
        {
          "t": 134.0,
          "v": 0.3
        },
        {
          "t": 136.0,
          "v": 0.1
        },
        {
          "t": 138.0,
          "v": 2.2
        },
        {
          "t": 140.0,
          "v": 0.1
        },
        {
          "t": 142.0,
          "v": 0.1
        },
        {
          "t": 144.0,
          "v": 1.9
        },
        {
          "t": 146.0,
          "v": 0.1
        },
        {
          "t": 148.0,
          "v": 0.1
        },
        {
          "t": 150.0,
          "v": 3.9
        },
        {
          "t": 152.0,
          "v": 0.7
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
          "v": 0.1
        },
        {
          "t": 162.0,
          "v": 0.1
        },
        {
          "t": 164.0,
          "v": 2.4
        },
        {
          "t": 166.0,
          "v": 0.3
        },
        {
          "t": 168.0,
          "v": 0.4
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
          "v": 1.9
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
          "v": 2.0
        },
        {
          "t": 184.0,
          "v": 0.1
        },
        {
          "t": 186.0,
          "v": 0.1
        },
        {
          "t": 188.0,
          "v": 2.1
        },
        {
          "t": 190.0,
          "v": 0.1
        },
        {
          "t": 192.0,
          "v": 0.1
        },
        {
          "t": 194.0,
          "v": 2.1
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
          "v": 1.9
        },
        {
          "t": 202.0,
          "v": 0.1
        },
        {
          "t": 204.0,
          "v": 0.1
        },
        {
          "t": 206.0,
          "v": 1.9
        },
        {
          "t": 208.0,
          "v": 0.1
        },
        {
          "t": 210.0,
          "v": 2.6
        },
        {
          "t": 212.0,
          "v": 2.0
        },
        {
          "t": 214.0,
          "v": 0.1
        },
        {
          "t": 216.0,
          "v": 0.4
        },
        {
          "t": 218.0,
          "v": 1.9
        },
        {
          "t": 220.0,
          "v": 4.9
        },
        {
          "t": 222.0,
          "v": 2.9
        },
        {
          "t": 224.0,
          "v": 2.3
        },
        {
          "t": 226.0,
          "v": 0.3
        },
        {
          "t": 228.0,
          "v": 0.4
        },
        {
          "t": 230.0,
          "v": 1.9
        },
        {
          "t": 232.0,
          "v": 2.4
        }
      ],
      "tx_kbs": [
        {
          "t": 0.0,
          "v": 66.1
        },
        {
          "t": 2.0,
          "v": 2.2
        },
        {
          "t": 4.0,
          "v": 1.1
        },
        {
          "t": 6.0,
          "v": 3.8
        },
        {
          "t": 8.0,
          "v": 0.5
        },
        {
          "t": 10.0,
          "v": 0.5
        },
        {
          "t": 12.0,
          "v": 4.3
        },
        {
          "t": 14.0,
          "v": 12.5
        },
        {
          "t": 16.0,
          "v": 0.5
        },
        {
          "t": 18.0,
          "v": 4.4
        },
        {
          "t": 20.0,
          "v": 0.5
        },
        {
          "t": 22.0,
          "v": 0.5
        },
        {
          "t": 24.0,
          "v": 4.3
        },
        {
          "t": 26.0,
          "v": 0.5
        },
        {
          "t": 28.0,
          "v": 0.6
        },
        {
          "t": 30.0,
          "v": 2.6
        },
        {
          "t": 32.0,
          "v": 3.9
        },
        {
          "t": 34.0,
          "v": 0.5
        },
        {
          "t": 36.0,
          "v": 0.5
        },
        {
          "t": 38.0,
          "v": 5.8
        },
        {
          "t": 40.0,
          "v": 0.5
        },
        {
          "t": 42.0,
          "v": 0.5
        },
        {
          "t": 44.0,
          "v": 17.0
        },
        {
          "t": 46.0,
          "v": 4.0
        },
        {
          "t": 48.0,
          "v": 2.0
        },
        {
          "t": 50.0,
          "v": 0.5
        },
        {
          "t": 52.0,
          "v": 4.4
        },
        {
          "t": 54.0,
          "v": 0.5
        },
        {
          "t": 56.0,
          "v": 0.5
        },
        {
          "t": 58.0,
          "v": 1.3
        },
        {
          "t": 60.0,
          "v": 3.9
        },
        {
          "t": 62.0,
          "v": 0.7
        },
        {
          "t": 64.0,
          "v": 0.9
        },
        {
          "t": 66.0,
          "v": 1.1
        },
        {
          "t": 68.0,
          "v": 3.8
        },
        {
          "t": 70.0,
          "v": 0.5
        },
        {
          "t": 72.0,
          "v": 38.1
        },
        {
          "t": 74.0,
          "v": 23.4
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
          "v": 2.2
        },
        {
          "t": 82.0,
          "v": 4.9
        },
        {
          "t": 84.0,
          "v": 1.6
        },
        {
          "t": 86.0,
          "v": 1.6
        },
        {
          "t": 88.0,
          "v": 6.5
        },
        {
          "t": 90.0,
          "v": 3.1
        },
        {
          "t": 92.0,
          "v": 1.6
        },
        {
          "t": 94.0,
          "v": 5.5
        },
        {
          "t": 96.0,
          "v": 1.6
        },
        {
          "t": 98.0,
          "v": 1.6
        },
        {
          "t": 100.0,
          "v": 5.8
        },
        {
          "t": 102.0,
          "v": 1.7
        },
        {
          "t": 104.0,
          "v": 7.0
        },
        {
          "t": 106.0,
          "v": 2.4
        },
        {
          "t": 108.0,
          "v": 6.5
        },
        {
          "t": 110.0,
          "v": 1.7
        },
        {
          "t": 112.0,
          "v": 1.7
        },
        {
          "t": 114.0,
          "v": 5.6
        },
        {
          "t": 116.0,
          "v": 1.7
        },
        {
          "t": 118.0,
          "v": 1.8
        },
        {
          "t": 120.0,
          "v": 5.6
        },
        {
          "t": 122.0,
          "v": 1.8
        },
        {
          "t": 124.0,
          "v": 1.7
        },
        {
          "t": 126.0,
          "v": 5.6
        },
        {
          "t": 128.0,
          "v": 1.7
        },
        {
          "t": 130.0,
          "v": 1.7
        },
        {
          "t": 132.0,
          "v": 5.6
        },
        {
          "t": 134.0,
          "v": 8.1
        },
        {
          "t": 136.0,
          "v": 1.8
        },
        {
          "t": 138.0,
          "v": 6.9
        },
        {
          "t": 140.0,
          "v": 1.8
        },
        {
          "t": 142.0,
          "v": 1.8
        },
        {
          "t": 144.0,
          "v": 5.6
        },
        {
          "t": 146.0,
          "v": 1.8
        },
        {
          "t": 148.0,
          "v": 1.8
        },
        {
          "t": 150.0,
          "v": 3.8
        },
        {
          "t": 152.0,
          "v": 5.1
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
          "v": 1.8
        },
        {
          "t": 162.0,
          "v": 1.9
        },
        {
          "t": 164.0,
          "v": 11.3
        },
        {
          "t": 166.0,
          "v": 2.0
        },
        {
          "t": 168.0,
          "v": 3.4
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
          "v": 5.6
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
          "v": 5.7
        },
        {
          "t": 184.0,
          "v": 1.8
        },
        {
          "t": 186.0,
          "v": 1.8
        },
        {
          "t": 188.0,
          "v": 6.7
        },
        {
          "t": 190.0,
          "v": 1.8
        },
        {
          "t": 192.0,
          "v": 1.8
        },
        {
          "t": 194.0,
          "v": 11.8
        },
        {
          "t": 196.0,
          "v": 1.8
        },
        {
          "t": 198.0,
          "v": 1.9
        },
        {
          "t": 200.0,
          "v": 5.7
        },
        {
          "t": 202.0,
          "v": 1.8
        },
        {
          "t": 204.0,
          "v": 1.8
        },
        {
          "t": 206.0,
          "v": 5.7
        },
        {
          "t": 208.0,
          "v": 1.8
        },
        {
          "t": 210.0,
          "v": 3.2
        },
        {
          "t": 212.0,
          "v": 5.7
        },
        {
          "t": 214.0,
          "v": 1.8
        },
        {
          "t": 216.0,
          "v": 12.7
        },
        {
          "t": 218.0,
          "v": 5.7
        },
        {
          "t": 220.0,
          "v": 3.3
        },
        {
          "t": 222.0,
          "v": 59.9
        },
        {
          "t": 224.0,
          "v": 11.3
        },
        {
          "t": 226.0,
          "v": 2.0
        },
        {
          "t": 228.0,
          "v": 3.4
        },
        {
          "t": 230.0,
          "v": 5.7
        },
        {
          "t": 232.0,
          "v": 3.4
        }
      ]
    },
    "paging": {
      "majflt": [
        {
          "t": 0.0,
          "v": 92104.5
        },
        {
          "t": 2.0,
          "v": 16837.0
        },
        {
          "t": 4.0,
          "v": 54210.0
        },
        {
          "t": 6.0,
          "v": 38073.5
        },
        {
          "t": 8.0,
          "v": 18757.71
        },
        {
          "t": 10.0,
          "v": 58126.0
        },
        {
          "t": 12.0,
          "v": 50774.5
        },
        {
          "t": 14.0,
          "v": 20647.0
        },
        {
          "t": 16.0,
          "v": 66337.0
        },
        {
          "t": 18.0,
          "v": 27067.5
        },
        {
          "t": 20.0,
          "v": 45338.5
        },
        {
          "t": 22.0,
          "v": 67101.0
        },
        {
          "t": 24.0,
          "v": 41356.5
        },
        {
          "t": 26.0,
          "v": 61830.5
        },
        {
          "t": 28.0,
          "v": 42915.5
        },
        {
          "t": 30.0,
          "v": 59107.5
        },
        {
          "t": 32.0,
          "v": 20143.5
        },
        {
          "t": 34.0,
          "v": 76700.0
        },
        {
          "t": 36.0,
          "v": 5929.5
        },
        {
          "t": 38.0,
          "v": 73977.5
        },
        {
          "t": 40.0,
          "v": 27228.0
        },
        {
          "t": 42.0,
          "v": 51662.0
        },
        {
          "t": 44.0,
          "v": 50714.0
        },
        {
          "t": 46.0,
          "v": 43313.0
        },
        {
          "t": 48.0,
          "v": 59564.5
        },
        {
          "t": 50.0,
          "v": 65326.0
        },
        {
          "t": 52.0,
          "v": 38544.0
        },
        {
          "t": 54.0,
          "v": 26965.5
        },
        {
          "t": 56.0,
          "v": 40011.5
        },
        {
          "t": 58.0,
          "v": 37628.5
        },
        {
          "t": 60.0,
          "v": 9843.5
        },
        {
          "t": 62.0,
          "v": 36090.55
        },
        {
          "t": 64.0,
          "v": 34492.0
        },
        {
          "t": 66.0,
          "v": 38698.5
        },
        {
          "t": 68.0,
          "v": 17082.0
        },
        {
          "t": 70.0,
          "v": 33464.0
        },
        {
          "t": 72.0,
          "v": 170209.5
        },
        {
          "t": 74.0,
          "v": 2642.5
        },
        {
          "t": 76.0,
          "v": 100340.0
        },
        {
          "t": 78.0,
          "v": 22232.5
        },
        {
          "t": 80.0,
          "v": 80140.0
        },
        {
          "t": 82.0,
          "v": 41550.5
        },
        {
          "t": 84.0,
          "v": 61078.0
        },
        {
          "t": 86.0,
          "v": 59975.0
        },
        {
          "t": 88.0,
          "v": 43357.5
        },
        {
          "t": 90.0,
          "v": 76959.0
        },
        {
          "t": 92.0,
          "v": 24605.0
        },
        {
          "t": 94.0,
          "v": 96730.5
        },
        {
          "t": 96.0,
          "v": 5591.5
        },
        {
          "t": 98.0,
          "v": 100433.0
        },
        {
          "t": 100.0,
          "v": 14655.5
        },
        {
          "t": 102.0,
          "v": 85897.5
        },
        {
          "t": 104.0,
          "v": 38522.0
        },
        {
          "t": 106.0,
          "v": 64693.0
        },
        {
          "t": 108.0,
          "v": 60439.0
        },
        {
          "t": 110.0,
          "v": 42609.0
        },
        {
          "t": 112.0,
          "v": 81591.5
        },
        {
          "t": 114.0,
          "v": 21818.5
        },
        {
          "t": 116.0,
          "v": 101663.5
        },
        {
          "t": 118.0,
          "v": 3268.5
        },
        {
          "t": 120.0,
          "v": 102549.5
        },
        {
          "t": 122.0,
          "v": 15933.5
        },
        {
          "t": 124.0,
          "v": 89839.5
        },
        {
          "t": 126.0,
          "v": 33035.0
        },
        {
          "t": 128.0,
          "v": 73504.5
        },
        {
          "t": 130.0,
          "v": 50044.0
        },
        {
          "t": 132.0,
          "v": 55434.5
        },
        {
          "t": 134.0,
          "v": 73060.5
        },
        {
          "t": 136.0,
          "v": 32180.5
        },
        {
          "t": 138.0,
          "v": 70101.5
        },
        {
          "t": 140.0,
          "v": 32486.0
        },
        {
          "t": 142.0,
          "v": 54263.5
        },
        {
          "t": 144.0,
          "v": 49120.5
        },
        {
          "t": 146.0,
          "v": 38260.2
        },
        {
          "t": 148.0,
          "v": 65900.0
        },
        {
          "t": 150.0,
          "v": 23947.5
        },
        {
          "t": 152.0,
          "v": 78237.0
        },
        {
          "t": 154.0,
          "v": 7481.0
        },
        {
          "t": 156.0,
          "v": 88841.5
        },
        {
          "t": 158.0,
          "v": 8954.0
        },
        {
          "t": 160.0,
          "v": 104264.5
        },
        {
          "t": 162.0,
          "v": 9578.5
        },
        {
          "t": 164.0,
          "v": 87078.5
        },
        {
          "t": 166.0,
          "v": 8800.5
        },
        {
          "t": 168.0,
          "v": 75945.5
        },
        {
          "t": 170.0,
          "v": 26416.0
        },
        {
          "t": 172.0,
          "v": 59560.5
        },
        {
          "t": 174.0,
          "v": 43278.5
        },
        {
          "t": 176.0,
          "v": 45428.5
        },
        {
          "t": 178.0,
          "v": 57333.5
        },
        {
          "t": 180.0,
          "v": 29384.0
        },
        {
          "t": 182.0,
          "v": 73290.5
        },
        {
          "t": 184.0,
          "v": 18064.0
        },
        {
          "t": 186.0,
          "v": 86919.0
        },
        {
          "t": 188.0,
          "v": 28718.0
        },
        {
          "t": 190.0,
          "v": 76016.5
        },
        {
          "t": 192.0,
          "v": 12263.5
        },
        {
          "t": 194.0,
          "v": 85950.5
        },
        {
          "t": 196.0,
          "v": 7522.5
        },
        {
          "t": 198.0,
          "v": 79451.5
        },
        {
          "t": 200.0,
          "v": 28722.0
        },
        {
          "t": 202.0,
          "v": 60811.5
        },
        {
          "t": 204.0,
          "v": 41879.5
        },
        {
          "t": 206.0,
          "v": 42303.5
        },
        {
          "t": 208.0,
          "v": 62524.0
        },
        {
          "t": 210.0,
          "v": 50090.0
        },
        {
          "t": 212.0,
          "v": 58837.0
        },
        {
          "t": 214.0,
          "v": 45331.0
        },
        {
          "t": 216.0,
          "v": 61121.0
        },
        {
          "t": 218.0,
          "v": 25751.0
        },
        {
          "t": 220.0,
          "v": 79163.0
        },
        {
          "t": 222.0,
          "v": 9595.5
        },
        {
          "t": 224.0,
          "v": 85121.5
        },
        {
          "t": 226.0,
          "v": 14300.0
        },
        {
          "t": 228.0,
          "v": 73131.5
        },
        {
          "t": 230.0,
          "v": 33167.5
        },
        {
          "t": 232.0,
          "v": 128924.0
        }
      ],
      "minflt": [
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
          "v": 20
        },
        {
          "t": 4,
          "v": 133
        },
        {
          "t": 6,
          "v": 526
        },
        {
          "t": 8,
          "v": 465
        },
        {
          "t": 10,
          "v": 513
        },
        {
          "t": 12,
          "v": 593
        },
        {
          "t": 14,
          "v": 467
        },
        {
          "t": 16,
          "v": 589
        },
        {
          "t": 18,
          "v": 467
        },
        {
          "t": 20,
          "v": 467
        },
        {
          "t": 22,
          "v": 893
        },
        {
          "t": 24,
          "v": 487
        },
        {
          "t": 26,
          "v": 487
        },
        {
          "t": 28,
          "v": 687
        },
        {
          "t": 30,
          "v": 687
        },
        {
          "t": 32,
          "v": 692
        },
        {
          "t": 34,
          "v": 691
        },
        {
          "t": 37,
          "v": 691
        },
        {
          "t": 39,
          "v": 693
        },
        {
          "t": 41,
          "v": 695
        },
        {
          "t": 43,
          "v": 695
        },
        {
          "t": 45,
          "v": 695
        },
        {
          "t": 47,
          "v": 696
        },
        {
          "t": 49,
          "v": 695
        },
        {
          "t": 51,
          "v": 695
        },
        {
          "t": 53,
          "v": 695
        },
        {
          "t": 55,
          "v": 1279
        },
        {
          "t": 57,
          "v": 1477
        },
        {
          "t": 59,
          "v": 1617
        },
        {
          "t": 61,
          "v": 1837
        },
        {
          "t": 63,
          "v": 2082
        },
        {
          "t": 65,
          "v": 2552
        },
        {
          "t": 67,
          "v": 2127
        },
        {
          "t": 69,
          "v": 2342
        },
        {
          "t": 71,
          "v": 2253
        },
        {
          "t": 73,
          "v": 2137
        },
        {
          "t": 75,
          "v": 26
        },
        {
          "t": 77,
          "v": 34
        },
        {
          "t": 79,
          "v": 46
        },
        {
          "t": 81,
          "v": 66
        },
        {
          "t": 83,
          "v": 90
        },
        {
          "t": 85,
          "v": 94
        },
        {
          "t": 87,
          "v": 94
        },
        {
          "t": 89,
          "v": 95
        },
        {
          "t": 91,
          "v": 94
        },
        {
          "t": 93,
          "v": 94
        },
        {
          "t": 95,
          "v": 94
        },
        {
          "t": 97,
          "v": 94
        },
        {
          "t": 99,
          "v": 94
        },
        {
          "t": 101,
          "v": 94
        },
        {
          "t": 103,
          "v": 94
        },
        {
          "t": 105,
          "v": 94
        },
        {
          "t": 107,
          "v": 94
        },
        {
          "t": 109,
          "v": 94
        },
        {
          "t": 111,
          "v": 127
        },
        {
          "t": 113,
          "v": 145
        },
        {
          "t": 115,
          "v": 156
        },
        {
          "t": 117,
          "v": 155
        },
        {
          "t": 119,
          "v": 155
        },
        {
          "t": 121,
          "v": 155
        },
        {
          "t": 123,
          "v": 155
        },
        {
          "t": 125,
          "v": 155
        },
        {
          "t": 127,
          "v": 155
        },
        {
          "t": 129,
          "v": 153
        },
        {
          "t": 131,
          "v": 151
        },
        {
          "t": 133,
          "v": 151
        },
        {
          "t": 135,
          "v": 151
        },
        {
          "t": 137,
          "v": 151
        },
        {
          "t": 139,
          "v": 151
        },
        {
          "t": 141,
          "v": 463
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
          "v": 448
        },
        {
          "t": 165,
          "v": 751
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
          "v": 1037
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
          "v": 1253
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
        },
        {
          "t": 235,
          "v": 8
        }
      ],
      "timewait": [
        {
          "t": 0,
          "v": 16
        },
        {
          "t": 2,
          "v": 17
        },
        {
          "t": 4,
          "v": 263
        },
        {
          "t": 6,
          "v": 2872
        },
        {
          "t": 8,
          "v": 7107
        },
        {
          "t": 10,
          "v": 9818
        },
        {
          "t": 12,
          "v": 10426
        },
        {
          "t": 14,
          "v": 12183
        },
        {
          "t": 16,
          "v": 12506
        },
        {
          "t": 18,
          "v": 13614
        },
        {
          "t": 20,
          "v": 13890
        },
        {
          "t": 22,
          "v": 15146
        },
        {
          "t": 24,
          "v": 15867
        },
        {
          "t": 26,
          "v": 15867
        },
        {
          "t": 28,
          "v": 15862
        },
        {
          "t": 30,
          "v": 15862
        },
        {
          "t": 32,
          "v": 15862
        },
        {
          "t": 34,
          "v": 15862
        },
        {
          "t": 37,
          "v": 15862
        },
        {
          "t": 39,
          "v": 15862
        },
        {
          "t": 41,
          "v": 15861
        },
        {
          "t": 43,
          "v": 15861
        },
        {
          "t": 45,
          "v": 15861
        },
        {
          "t": 47,
          "v": 15861
        },
        {
          "t": 49,
          "v": 15861
        },
        {
          "t": 51,
          "v": 15861
        },
        {
          "t": 53,
          "v": 15861
        },
        {
          "t": 55,
          "v": 15813
        },
        {
          "t": 57,
          "v": 15814
        },
        {
          "t": 59,
          "v": 15879
        },
        {
          "t": 61,
          "v": 16209
        },
        {
          "t": 63,
          "v": 16714
        },
        {
          "t": 65,
          "v": 17198
        },
        {
          "t": 67,
          "v": 17277
        },
        {
          "t": 69,
          "v": 18678
        },
        {
          "t": 71,
          "v": 16381
        },
        {
          "t": 73,
          "v": 17541
        },
        {
          "t": 75,
          "v": 17144
        },
        {
          "t": 77,
          "v": 17144
        },
        {
          "t": 79,
          "v": 16040
        },
        {
          "t": 81,
          "v": 16078
        },
        {
          "t": 83,
          "v": 13500
        },
        {
          "t": 85,
          "v": 13541
        },
        {
          "t": 87,
          "v": 13418
        },
        {
          "t": 89,
          "v": 13461
        },
        {
          "t": 91,
          "v": 13503
        },
        {
          "t": 93,
          "v": 13545
        },
        {
          "t": 95,
          "v": 13589
        },
        {
          "t": 97,
          "v": 13629
        },
        {
          "t": 99,
          "v": 13635
        },
        {
          "t": 101,
          "v": 13635
        },
        {
          "t": 103,
          "v": 13635
        },
        {
          "t": 105,
          "v": 13635
        },
        {
          "t": 107,
          "v": 13635
        },
        {
          "t": 109,
          "v": 13635
        },
        {
          "t": 111,
          "v": 13644
        },
        {
          "t": 113,
          "v": 13664
        },
        {
          "t": 115,
          "v": 13686
        },
        {
          "t": 117,
          "v": 13706
        },
        {
          "t": 119,
          "v": 13727
        },
        {
          "t": 121,
          "v": 13591
        },
        {
          "t": 123,
          "v": 13612
        },
        {
          "t": 125,
          "v": 12368
        },
        {
          "t": 127,
          "v": 12388
        },
        {
          "t": 129,
          "v": 8087
        },
        {
          "t": 131,
          "v": 8102
        },
        {
          "t": 133,
          "v": 3119
        },
        {
          "t": 135,
          "v": 3119
        },
        {
          "t": 137,
          "v": 630
        },
        {
          "t": 139,
          "v": 630
        },
        {
          "t": 141,
          "v": 626
        },
        {
          "t": 143,
          "v": 628
        },
        {
          "t": 145,
          "v": 566
        },
        {
          "t": 147,
          "v": 572
        },
        {
          "t": 149,
          "v": 516
        },
        {
          "t": 151,
          "v": 533
        },
        {
          "t": 153,
          "v": 453
        },
        {
          "t": 155,
          "v": 479
        },
        {
          "t": 157,
          "v": 412
        },
        {
          "t": 159,
          "v": 453
        },
        {
          "t": 161,
          "v": 441
        },
        {
          "t": 163,
          "v": 448
        },
        {
          "t": 165,
          "v": 477
        },
        {
          "t": 167,
          "v": 526
        },
        {
          "t": 169,
          "v": 543
        },
        {
          "t": 171,
          "v": 545
        },
        {
          "t": 173,
          "v": 529
        },
        {
          "t": 175,
          "v": 572
        },
        {
          "t": 177,
          "v": 562
        },
        {
          "t": 179,
          "v": 616
        },
        {
          "t": 181,
          "v": 571
        },
        {
          "t": 183,
          "v": 602
        },
        {
          "t": 185,
          "v": 587
        },
        {
          "t": 187,
          "v": 587
        },
        {
          "t": 189,
          "v": 587
        },
        {
          "t": 191,
          "v": 552
        },
        {
          "t": 193,
          "v": 642
        },
        {
          "t": 195,
          "v": 637
        },
        {
          "t": 197,
          "v": 672
        },
        {
          "t": 199,
          "v": 679
        },
        {
          "t": 201,
          "v": 769
        },
        {
          "t": 203,
          "v": 726
        },
        {
          "t": 205,
          "v": 773
        },
        {
          "t": 207,
          "v": 754
        },
        {
          "t": 209,
          "v": 791
        },
        {
          "t": 211,
          "v": 752
        },
        {
          "t": 213,
          "v": 752
        },
        {
          "t": 215,
          "v": 710
        },
        {
          "t": 217,
          "v": 736
        },
        {
          "t": 219,
          "v": 730
        },
        {
          "t": 221,
          "v": 777
        },
        {
          "t": 223,
          "v": 739
        },
        {
          "t": 225,
          "v": 751
        },
        {
          "t": 227,
          "v": 736
        },
        {
          "t": 229,
          "v": 805
        },
        {
          "t": 231,
          "v": 826
        },
        {
          "t": 233,
          "v": 869
        },
        {
          "t": 235,
          "v": 1468
        }
      ],
      "closed": [
        {
          "t": 0,
          "v": 16
        },
        {
          "t": 2,
          "v": 17
        },
        {
          "t": 4,
          "v": 263
        },
        {
          "t": 6,
          "v": 2872
        },
        {
          "t": 8,
          "v": 7107
        },
        {
          "t": 10,
          "v": 9818
        },
        {
          "t": 12,
          "v": 10426
        },
        {
          "t": 14,
          "v": 12183
        },
        {
          "t": 16,
          "v": 12507
        },
        {
          "t": 18,
          "v": 13614
        },
        {
          "t": 20,
          "v": 13890
        },
        {
          "t": 22,
          "v": 15146
        },
        {
          "t": 24,
          "v": 15867
        },
        {
          "t": 26,
          "v": 15867
        },
        {
          "t": 28,
          "v": 15862
        },
        {
          "t": 30,
          "v": 15862
        },
        {
          "t": 32,
          "v": 15862
        },
        {
          "t": 34,
          "v": 15862
        },
        {
          "t": 37,
          "v": 15862
        },
        {
          "t": 39,
          "v": 15862
        },
        {
          "t": 41,
          "v": 15861
        },
        {
          "t": 43,
          "v": 15861
        },
        {
          "t": 45,
          "v": 15861
        },
        {
          "t": 47,
          "v": 15861
        },
        {
          "t": 49,
          "v": 15861
        },
        {
          "t": 51,
          "v": 15861
        },
        {
          "t": 53,
          "v": 15861
        },
        {
          "t": 55,
          "v": 15813
        },
        {
          "t": 57,
          "v": 15814
        },
        {
          "t": 59,
          "v": 15879
        },
        {
          "t": 61,
          "v": 16209
        },
        {
          "t": 63,
          "v": 16714
        },
        {
          "t": 65,
          "v": 17199
        },
        {
          "t": 67,
          "v": 17277
        },
        {
          "t": 69,
          "v": 18678
        },
        {
          "t": 71,
          "v": 16381
        },
        {
          "t": 73,
          "v": 17541
        },
        {
          "t": 75,
          "v": 17144
        },
        {
          "t": 77,
          "v": 17144
        },
        {
          "t": 79,
          "v": 16040
        },
        {
          "t": 81,
          "v": 16078
        },
        {
          "t": 83,
          "v": 13500
        },
        {
          "t": 85,
          "v": 13541
        },
        {
          "t": 87,
          "v": 13418
        },
        {
          "t": 89,
          "v": 13461
        },
        {
          "t": 91,
          "v": 13503
        },
        {
          "t": 93,
          "v": 13545
        },
        {
          "t": 95,
          "v": 13589
        },
        {
          "t": 97,
          "v": 13629
        },
        {
          "t": 99,
          "v": 13635
        },
        {
          "t": 101,
          "v": 13635
        },
        {
          "t": 103,
          "v": 13635
        },
        {
          "t": 105,
          "v": 13635
        },
        {
          "t": 107,
          "v": 13635
        },
        {
          "t": 109,
          "v": 13635
        },
        {
          "t": 111,
          "v": 13644
        },
        {
          "t": 113,
          "v": 13664
        },
        {
          "t": 115,
          "v": 13686
        },
        {
          "t": 117,
          "v": 13706
        },
        {
          "t": 119,
          "v": 13727
        },
        {
          "t": 121,
          "v": 13591
        },
        {
          "t": 123,
          "v": 13612
        },
        {
          "t": 125,
          "v": 12368
        },
        {
          "t": 127,
          "v": 12388
        },
        {
          "t": 129,
          "v": 8087
        },
        {
          "t": 131,
          "v": 8102
        },
        {
          "t": 133,
          "v": 3119
        },
        {
          "t": 135,
          "v": 3119
        },
        {
          "t": 137,
          "v": 630
        },
        {
          "t": 139,
          "v": 630
        },
        {
          "t": 141,
          "v": 626
        },
        {
          "t": 143,
          "v": 628
        },
        {
          "t": 145,
          "v": 566
        },
        {
          "t": 147,
          "v": 572
        },
        {
          "t": 149,
          "v": 516
        },
        {
          "t": 151,
          "v": 533
        },
        {
          "t": 153,
          "v": 453
        },
        {
          "t": 155,
          "v": 479
        },
        {
          "t": 157,
          "v": 412
        },
        {
          "t": 159,
          "v": 453
        },
        {
          "t": 161,
          "v": 441
        },
        {
          "t": 163,
          "v": 448
        },
        {
          "t": 165,
          "v": 477
        },
        {
          "t": 167,
          "v": 526
        },
        {
          "t": 169,
          "v": 543
        },
        {
          "t": 171,
          "v": 545
        },
        {
          "t": 173,
          "v": 529
        },
        {
          "t": 175,
          "v": 572
        },
        {
          "t": 177,
          "v": 562
        },
        {
          "t": 179,
          "v": 616
        },
        {
          "t": 181,
          "v": 571
        },
        {
          "t": 183,
          "v": 602
        },
        {
          "t": 185,
          "v": 587
        },
        {
          "t": 187,
          "v": 587
        },
        {
          "t": 189,
          "v": 587
        },
        {
          "t": 191,
          "v": 552
        },
        {
          "t": 193,
          "v": 642
        },
        {
          "t": 195,
          "v": 637
        },
        {
          "t": 197,
          "v": 672
        },
        {
          "t": 199,
          "v": 679
        },
        {
          "t": 201,
          "v": 769
        },
        {
          "t": 203,
          "v": 726
        },
        {
          "t": 205,
          "v": 773
        },
        {
          "t": 207,
          "v": 754
        },
        {
          "t": 209,
          "v": 791
        },
        {
          "t": 211,
          "v": 752
        },
        {
          "t": 213,
          "v": 752
        },
        {
          "t": 215,
          "v": 710
        },
        {
          "t": 217,
          "v": 736
        },
        {
          "t": 219,
          "v": 730
        },
        {
          "t": 221,
          "v": 777
        },
        {
          "t": 223,
          "v": 739
        },
        {
          "t": 225,
          "v": 751
        },
        {
          "t": 227,
          "v": 736
        },
        {
          "t": 229,
          "v": 805
        },
        {
          "t": 231,
          "v": 826
        },
        {
          "t": 233,
          "v": 869
        },
        {
          "t": 235,
          "v": 1468
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
          "t": 19,
          "v": 0.0
        },
        {
          "t": 21,
          "v": 0.0
        },
        {
          "t": 23,
          "v": 0.0
        },
        {
          "t": 25,
          "v": 0.0
        },
        {
          "t": 27,
          "v": 0.0
        },
        {
          "t": 29,
          "v": 0.0
        },
        {
          "t": 31,
          "v": 0.0
        },
        {
          "t": 33,
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
          "t": 144,
          "v": 0.0
        },
        {
          "t": 146,
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
          "t": 19,
          "v": 0.0
        },
        {
          "t": 21,
          "v": 0.0
        },
        {
          "t": 23,
          "v": 0.0
        },
        {
          "t": 25,
          "v": 0.0
        },
        {
          "t": 27,
          "v": 0.0
        },
        {
          "t": 29,
          "v": 0.0
        },
        {
          "t": 31,
          "v": 0.0
        },
        {
          "t": 33,
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
          "t": 144,
          "v": 0.0
        },
        {
          "t": 146,
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
          "v": 0.0
        },
        {
          "t": 2,
          "v": 0.18
        },
        {
          "t": 4,
          "v": 0.15
        },
        {
          "t": 6,
          "v": 0.12
        },
        {
          "t": 8,
          "v": 0.1
        },
        {
          "t": 10,
          "v": 0.08
        },
        {
          "t": 12,
          "v": 0.06
        },
        {
          "t": 14,
          "v": 0.05
        },
        {
          "t": 16,
          "v": 0.04
        },
        {
          "t": 19,
          "v": 0.03
        },
        {
          "t": 21,
          "v": 0.03
        },
        {
          "t": 23,
          "v": 0.02
        },
        {
          "t": 25,
          "v": 0.01
        },
        {
          "t": 27,
          "v": 0.01
        },
        {
          "t": 29,
          "v": 0.01
        },
        {
          "t": 31,
          "v": 0.01
        },
        {
          "t": 33,
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
          "t": 144,
          "v": 0.0
        },
        {
          "t": 146,
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
    "trace_server": "https://locustbaby.github.io/duotunnel/bench/traces/756e8d1-24183846731-server.bin.gz",
    "trace_client": "https://locustbaby.github.io/duotunnel/bench/traces/756e8d1-24183846731-client.bin.gz"
  }
};
