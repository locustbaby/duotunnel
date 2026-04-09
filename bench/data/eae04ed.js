window.BENCH_DETAIL['eae04ed'] = {
  "timestamp": "2026-04-09T10:07:47.700Z",
  "commit": {
    "id": "eae04ed93e566effb9fa5d1c25606a39008e5a1c",
    "message": "ci: reduce dial9 trace artifact retention to 30d",
    "url": "https://github.com/locustbaby/duotunnel/commit/eae04ed93e566effb9fa5d1c25606a39008e5a1c"
  },
  "scenarios": [
    {
      "name": "ingress_http_get",
      "protocol": "HTTP",
      "direction": "ingress",
      "category": "basic",
      "includeInTotalRps": false,
      "p50": 0.6,
      "p95": 1.13,
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
      "p50": 0.6,
      "p95": 1.17,
      "p99": 0,
      "err": 0,
      "rps": 22.96,
      "requests": 574
    },
    {
      "name": "egress_http_get",
      "protocol": "HTTP",
      "direction": "egress",
      "category": "basic",
      "includeInTotalRps": false,
      "p50": 0.55,
      "p95": 0.98,
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
      "p50": 0.54,
      "p95": 0.75,
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
      "p50": 1.84,
      "p95": 2.58,
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
      "p50": 1.22,
      "p95": 1.9,
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
      "p50": 1.21,
      "p95": 1.88,
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
      "p50": 0.52,
      "p95": 0.75,
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
      "p50": 0.66,
      "p95": 1.98,
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
      "p50": 1.11,
      "p95": 2.67,
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
      "p50": 2.71,
      "p95": 3.62,
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
      "p50": 1.06,
      "p95": 2.48,
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
      "p50": 44.31,
      "p95": 46.08,
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
      "p50": 1.11,
      "p95": 2.88,
      "p99": 0,
      "err": 0,
      "rps": 15,
      "requests": 300
    },
    {
      "name": "grpc_high_qps",
      "protocol": "gRPC",
      "direction": "ingress",
      "category": "stress",
      "includeInTotalRps": true,
      "p50": 0.83,
      "p95": 1.72,
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
      "p50": 0.54,
      "p95": 2.05,
      "p99": 0,
      "err": 0,
      "rps": 3000,
      "requests": 60000
    },
    {
      "name": "egress_3000qps",
      "protocol": "HTTP",
      "direction": "egress",
      "category": "stress",
      "includeInTotalRps": true,
      "p50": 0.55,
      "p95": 2.09,
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
      "p50": 0.57,
      "p95": 2.21,
      "p99": 0,
      "err": 0,
      "rps": 2999.8,
      "requests": 59996
    },
    {
      "name": "egress_multihost",
      "protocol": "HTTP",
      "direction": "egress",
      "category": "stress",
      "includeInTotalRps": true,
      "p50": 0.55,
      "p95": 2.08,
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
      "p50": 40.13,
      "p95": 112.85,
      "p99": 0,
      "err": 0,
      "rps": 6902.15,
      "requests": 138043
    },
    {
      "name": "egress_8000qps (8k-q4)",
      "protocol": "HTTP",
      "direction": "egress",
      "category": "stress",
      "includeInTotalRps": true,
      "p50": 23.93,
      "p95": 104.04,
      "p99": 0,
      "err": 0,
      "rps": 7242.95,
      "requests": 144859
    },
    {
      "name": "ingress_multihost_8000qps (8k-q4)",
      "protocol": "HTTP",
      "direction": "ingress",
      "category": "stress",
      "includeInTotalRps": true,
      "p50": 10.54,
      "p95": 37.39,
      "p99": 0,
      "err": 1.11,
      "rps": 6983.7,
      "requests": 139674
    },
    {
      "name": "egress_multihost_8000qps (8k-q4)",
      "protocol": "HTTP",
      "direction": "egress",
      "category": "stress",
      "includeInTotalRps": true,
      "p50": 44.97,
      "p95": 117.88,
      "p99": 0,
      "err": 0.01,
      "rps": 7219.1,
      "requests": 144382
    },
    {
      "name": "ingress_3000qps",
      "protocol": "HTTP",
      "direction": "ingress",
      "category": "stress",
      "includeInTotalRps": true,
      "p50": 1.09,
      "p95": 37.34,
      "p99": 0,
      "err": 0,
      "rps": 2956.35,
      "requests": 59127,
      "tunnel": "frp"
    },
    {
      "name": "ingress_multihost",
      "protocol": "HTTP",
      "direction": "ingress",
      "category": "stress",
      "includeInTotalRps": true,
      "p50": 0.65,
      "p95": 3.1,
      "p99": 0,
      "err": 0,
      "rps": 3000.05,
      "requests": 60001,
      "tunnel": "frp"
    },
    {
      "name": "ingress_multihost_8000qps",
      "protocol": "HTTP",
      "direction": "ingress",
      "category": "stress",
      "includeInTotalRps": true,
      "p50": 44.62,
      "p95": 114.42,
      "p99": 0,
      "err": 0,
      "rps": 5615.4,
      "requests": 112308,
      "tunnel": "frp"
    }
  ],
  "summary": {
    "totalRPS": 40437.8,
    "totalErr": 0.19,
    "totalRequests": 815684
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
  "run_url": "https://github.com/locustbaby/duotunnel/actions/runs/24184246999",
  "resources": {
    "processes": {
      "server": {
        "cpu": [
          {
            "t": 20.0,
            "v": 0.04
          },
          {
            "t": 75.7,
            "v": 0.09
          },
          {
            "t": 79.3,
            "v": 0.84
          },
          {
            "t": 82.9,
            "v": 1.52
          },
          {
            "t": 86.5,
            "v": 1.85
          },
          {
            "t": 90.2,
            "v": 1.9
          },
          {
            "t": 93.9,
            "v": 1.83
          },
          {
            "t": 97.6,
            "v": 1.7
          },
          {
            "t": 101.3,
            "v": 1.14
          },
          {
            "t": 105.0,
            "v": 0.27
          },
          {
            "t": 108.5,
            "v": 0.21
          },
          {
            "t": 112.1,
            "v": 1.12
          },
          {
            "t": 115.7,
            "v": 1.86
          },
          {
            "t": 119.4,
            "v": 2.18
          },
          {
            "t": 123.2,
            "v": 2.54
          },
          {
            "t": 126.9,
            "v": 2.27
          },
          {
            "t": 130.6,
            "v": 1.83
          },
          {
            "t": 134.2,
            "v": 0.41
          },
          {
            "t": 137.8,
            "v": 0.21
          },
          {
            "t": 141.9,
            "v": 12.99
          },
          {
            "t": 146.4,
            "v": 12.09
          },
          {
            "t": 150.8,
            "v": 12.02
          },
          {
            "t": 155.2,
            "v": 11.91
          },
          {
            "t": 159.7,
            "v": 5.86
          },
          {
            "t": 163.3,
            "v": 3.52
          },
          {
            "t": 167.8,
            "v": 14.99
          },
          {
            "t": 172.3,
            "v": 14.59
          },
          {
            "t": 176.7,
            "v": 14.68
          },
          {
            "t": 181.2,
            "v": 14.73
          },
          {
            "t": 185.5,
            "v": 3.84
          },
          {
            "t": 189.1,
            "v": 5.62
          },
          {
            "t": 193.5,
            "v": 12.41
          },
          {
            "t": 198.0,
            "v": 11.97
          },
          {
            "t": 202.5,
            "v": 12.0
          },
          {
            "t": 207.0,
            "v": 12.01
          },
          {
            "t": 211.1,
            "v": 1.21
          },
          {
            "t": 214.7,
            "v": 9.76
          },
          {
            "t": 219.1,
            "v": 14.86
          },
          {
            "t": 223.6,
            "v": 14.73
          },
          {
            "t": 228.2,
            "v": 14.56
          },
          {
            "t": 232.7,
            "v": 13.82
          }
        ],
        "rss": [
          {
            "t": 3.6,
            "v": 15.1
          },
          {
            "t": 8.6,
            "v": 15.1
          },
          {
            "t": 13.6,
            "v": 15.1
          },
          {
            "t": 20.0,
            "v": 15.1
          },
          {
            "t": 24.7,
            "v": 15.1
          },
          {
            "t": 28.3,
            "v": 15.1
          },
          {
            "t": 33.0,
            "v": 15.1
          },
          {
            "t": 37.7,
            "v": 15.1
          },
          {
            "t": 42.4,
            "v": 15.1
          },
          {
            "t": 47.1,
            "v": 15.1
          },
          {
            "t": 50.7,
            "v": 15.1
          },
          {
            "t": 54.4,
            "v": 15.1
          },
          {
            "t": 62.0,
            "v": 15.1
          },
          {
            "t": 70.0,
            "v": 15.1
          },
          {
            "t": 75.7,
            "v": 15.3
          },
          {
            "t": 79.3,
            "v": 16.2
          },
          {
            "t": 82.9,
            "v": 16.7
          },
          {
            "t": 86.5,
            "v": 16.8
          },
          {
            "t": 90.2,
            "v": 17.1
          },
          {
            "t": 93.9,
            "v": 17.2
          },
          {
            "t": 97.6,
            "v": 17.2
          },
          {
            "t": 101.3,
            "v": 17.0
          },
          {
            "t": 105.0,
            "v": 17.0
          },
          {
            "t": 108.5,
            "v": 18.1
          },
          {
            "t": 112.1,
            "v": 19.9
          },
          {
            "t": 115.7,
            "v": 21.0
          },
          {
            "t": 119.4,
            "v": 21.5
          },
          {
            "t": 123.2,
            "v": 21.7
          },
          {
            "t": 126.9,
            "v": 21.6
          },
          {
            "t": 130.6,
            "v": 21.8
          },
          {
            "t": 134.2,
            "v": 21.3
          },
          {
            "t": 137.8,
            "v": 22.6
          },
          {
            "t": 141.9,
            "v": 22.5
          },
          {
            "t": 146.4,
            "v": 22.4
          },
          {
            "t": 150.8,
            "v": 22.2
          },
          {
            "t": 155.2,
            "v": 22.2
          },
          {
            "t": 159.7,
            "v": 22.2
          },
          {
            "t": 163.3,
            "v": 24.6
          },
          {
            "t": 167.8,
            "v": 25.1
          },
          {
            "t": 172.3,
            "v": 25.3
          },
          {
            "t": 176.7,
            "v": 25.2
          },
          {
            "t": 181.2,
            "v": 25.2
          },
          {
            "t": 185.5,
            "v": 25.3
          },
          {
            "t": 189.1,
            "v": 28.4
          },
          {
            "t": 193.5,
            "v": 28.4
          },
          {
            "t": 198.0,
            "v": 28.4
          },
          {
            "t": 202.5,
            "v": 28.4
          },
          {
            "t": 207.0,
            "v": 28.4
          },
          {
            "t": 211.1,
            "v": 28.4
          },
          {
            "t": 214.7,
            "v": 29.7
          },
          {
            "t": 219.1,
            "v": 30.0
          },
          {
            "t": 223.6,
            "v": 29.7
          },
          {
            "t": 228.2,
            "v": 29.3
          },
          {
            "t": 232.7,
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
            "v": 4.0
          },
          {
            "t": 82.0,
            "v": 2.0
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
            "t": 75.7,
            "v": 0.09
          },
          {
            "t": 79.3,
            "v": 0.84
          },
          {
            "t": 82.9,
            "v": 1.52
          },
          {
            "t": 86.5,
            "v": 1.64
          },
          {
            "t": 90.2,
            "v": 1.83
          },
          {
            "t": 93.9,
            "v": 1.76
          },
          {
            "t": 97.6,
            "v": 1.63
          },
          {
            "t": 101.3,
            "v": 1.01
          },
          {
            "t": 105.0,
            "v": 0.21
          },
          {
            "t": 108.5,
            "v": 0.14
          },
          {
            "t": 112.1,
            "v": 1.26
          },
          {
            "t": 115.7,
            "v": 1.66
          },
          {
            "t": 119.4,
            "v": 2.18
          },
          {
            "t": 123.2,
            "v": 2.34
          },
          {
            "t": 126.9,
            "v": 2.2
          },
          {
            "t": 130.6,
            "v": 1.63
          },
          {
            "t": 134.2,
            "v": 0.34
          },
          {
            "t": 137.8,
            "v": 0.35
          },
          {
            "t": 141.9,
            "v": 15.95
          },
          {
            "t": 146.4,
            "v": 14.79
          },
          {
            "t": 150.8,
            "v": 14.67
          },
          {
            "t": 155.2,
            "v": 14.51
          },
          {
            "t": 159.7,
            "v": 7.09
          },
          {
            "t": 163.3,
            "v": 3.04
          },
          {
            "t": 167.8,
            "v": 12.11
          },
          {
            "t": 172.3,
            "v": 11.83
          },
          {
            "t": 176.7,
            "v": 11.98
          },
          {
            "t": 181.2,
            "v": 11.92
          },
          {
            "t": 185.5,
            "v": 3.09
          },
          {
            "t": 189.1,
            "v": 6.74
          },
          {
            "t": 193.5,
            "v": 15.03
          },
          {
            "t": 198.0,
            "v": 14.41
          },
          {
            "t": 202.5,
            "v": 14.53
          },
          {
            "t": 207.0,
            "v": 14.5
          },
          {
            "t": 211.1,
            "v": 1.45
          },
          {
            "t": 214.7,
            "v": 8.02
          },
          {
            "t": 219.1,
            "v": 11.94
          },
          {
            "t": 223.6,
            "v": 11.84
          },
          {
            "t": 228.2,
            "v": 11.8
          },
          {
            "t": 232.7,
            "v": 11.1
          }
        ],
        "rss": [
          {
            "t": 3.6,
            "v": 11.2
          },
          {
            "t": 8.6,
            "v": 11.2
          },
          {
            "t": 13.6,
            "v": 11.2
          },
          {
            "t": 20.0,
            "v": 11.2
          },
          {
            "t": 24.7,
            "v": 11.2
          },
          {
            "t": 28.3,
            "v": 11.2
          },
          {
            "t": 33.0,
            "v": 11.2
          },
          {
            "t": 37.7,
            "v": 11.2
          },
          {
            "t": 42.4,
            "v": 11.2
          },
          {
            "t": 47.1,
            "v": 11.2
          },
          {
            "t": 50.7,
            "v": 11.2
          },
          {
            "t": 54.4,
            "v": 11.2
          },
          {
            "t": 62.0,
            "v": 11.2
          },
          {
            "t": 70.0,
            "v": 11.2
          },
          {
            "t": 75.7,
            "v": 11.5
          },
          {
            "t": 79.3,
            "v": 12.6
          },
          {
            "t": 82.9,
            "v": 13.1
          },
          {
            "t": 86.5,
            "v": 13.2
          },
          {
            "t": 90.2,
            "v": 13.4
          },
          {
            "t": 93.9,
            "v": 13.5
          },
          {
            "t": 97.6,
            "v": 13.6
          },
          {
            "t": 101.3,
            "v": 13.7
          },
          {
            "t": 105.0,
            "v": 13.7
          },
          {
            "t": 108.5,
            "v": 15.8
          },
          {
            "t": 112.1,
            "v": 17.4
          },
          {
            "t": 115.7,
            "v": 18.3
          },
          {
            "t": 119.4,
            "v": 18.5
          },
          {
            "t": 123.2,
            "v": 19.4
          },
          {
            "t": 126.9,
            "v": 18.7
          },
          {
            "t": 130.6,
            "v": 17.4
          },
          {
            "t": 134.2,
            "v": 17.4
          },
          {
            "t": 137.8,
            "v": 18.2
          },
          {
            "t": 141.9,
            "v": 18.4
          },
          {
            "t": 146.4,
            "v": 18.4
          },
          {
            "t": 150.8,
            "v": 18.3
          },
          {
            "t": 155.2,
            "v": 18.4
          },
          {
            "t": 159.7,
            "v": 18.5
          },
          {
            "t": 163.3,
            "v": 19.9
          },
          {
            "t": 167.8,
            "v": 19.8
          },
          {
            "t": 172.3,
            "v": 19.8
          },
          {
            "t": 176.7,
            "v": 19.7
          },
          {
            "t": 181.2,
            "v": 19.6
          },
          {
            "t": 185.5,
            "v": 19.7
          },
          {
            "t": 189.1,
            "v": 23.0
          },
          {
            "t": 193.5,
            "v": 22.9
          },
          {
            "t": 198.0,
            "v": 22.3
          },
          {
            "t": 202.5,
            "v": 22.6
          },
          {
            "t": 207.0,
            "v": 22.8
          },
          {
            "t": 211.1,
            "v": 22.8
          },
          {
            "t": 214.7,
            "v": 23.3
          },
          {
            "t": 219.1,
            "v": 23.2
          },
          {
            "t": 223.6,
            "v": 23.0
          },
          {
            "t": 228.2,
            "v": 22.7
          },
          {
            "t": 232.7,
            "v": 22.2
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
            "v": 6.0
          },
          {
            "t": 86.0,
            "v": 6.0
          },
          {
            "t": 88.0,
            "v": 4.0
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
            "v": 6.0
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
            "v": 6.0
          },
          {
            "t": 110.0,
            "v": 2.0
          },
          {
            "t": 112.0,
            "v": 4.0
          },
          {
            "t": 114.0,
            "v": 4.0
          },
          {
            "t": 116.0,
            "v": 2.0
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
            "v": 4.0
          },
          {
            "t": 126.0,
            "v": 2.0
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
            "v": 2.5
          },
          {
            "t": 74.0,
            "v": 1.5
          },
          {
            "t": 76.0,
            "v": 10.5
          },
          {
            "t": 78.0,
            "v": 12.0
          },
          {
            "t": 80.0,
            "v": 15.5
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
            "v": 13.0
          },
          {
            "t": 88.0,
            "v": 13.0
          },
          {
            "t": 90.0,
            "v": 13.5
          },
          {
            "t": 92.0,
            "v": 12.5
          },
          {
            "t": 94.0,
            "v": 11.0
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
            "v": 12.0
          },
          {
            "t": 110.0,
            "v": 8.0
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
            "v": 9.0
          },
          {
            "t": 118.0,
            "v": 9.0
          },
          {
            "t": 120.0,
            "v": 6.0
          },
          {
            "t": 122.0,
            "v": 7.5
          },
          {
            "t": 124.0,
            "v": 8.0
          },
          {
            "t": 126.0,
            "v": 6.5
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
            "v": 39.5
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
            "v": 61.5
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
            "v": 1.5
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
            "v": 0.5
          },
          {
            "t": 84.0,
            "v": 0.5
          },
          {
            "t": 86.0,
            "v": 1.0
          },
          {
            "t": 88.0,
            "v": 0.5
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
            "v": 1.5
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
            "v": 1.0
          },
          {
            "t": 110.0,
            "v": 0.5
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
            "v": 1.0
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
            "v": 1.0
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
            "v": 3.0
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
            "v": 5.5
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
            "v": 3.11
          },
          {
            "t": 8.6,
            "v": 4.53
          },
          {
            "t": 13.6,
            "v": 5.04
          },
          {
            "t": 20.0,
            "v": 4.33
          },
          {
            "t": 24.7,
            "v": 1.17
          },
          {
            "t": 28.3,
            "v": 2.36
          },
          {
            "t": 33.0,
            "v": 4.07
          },
          {
            "t": 37.7,
            "v": 4.01
          },
          {
            "t": 42.4,
            "v": 4.05
          },
          {
            "t": 47.1,
            "v": 3.27
          },
          {
            "t": 54.4,
            "v": 6.39
          },
          {
            "t": 62.0,
            "v": 6.44
          },
          {
            "t": 70.0,
            "v": 6.37
          },
          {
            "t": 79.3,
            "v": 0.14
          },
          {
            "t": 82.9,
            "v": 0.21
          },
          {
            "t": 86.5,
            "v": 0.27
          },
          {
            "t": 90.2,
            "v": 0.41
          },
          {
            "t": 93.9,
            "v": 0.27
          },
          {
            "t": 97.6,
            "v": 0.34
          },
          {
            "t": 101.3,
            "v": 0.27
          },
          {
            "t": 105.0,
            "v": 0.07
          },
          {
            "t": 108.5,
            "v": 0.07
          },
          {
            "t": 112.1,
            "v": 0.28
          },
          {
            "t": 115.7,
            "v": 0.35
          },
          {
            "t": 119.4,
            "v": 0.2
          },
          {
            "t": 123.2,
            "v": 0.4
          },
          {
            "t": 126.9,
            "v": 0.27
          },
          {
            "t": 130.6,
            "v": 0.07
          },
          {
            "t": 141.9,
            "v": 3.75
          },
          {
            "t": 146.4,
            "v": 3.54
          },
          {
            "t": 150.8,
            "v": 3.5
          },
          {
            "t": 155.2,
            "v": 3.44
          },
          {
            "t": 159.7,
            "v": 1.79
          },
          {
            "t": 163.3,
            "v": 0.83
          },
          {
            "t": 167.8,
            "v": 3.56
          },
          {
            "t": 172.3,
            "v": 3.54
          },
          {
            "t": 176.7,
            "v": 3.43
          },
          {
            "t": 181.2,
            "v": 3.54
          },
          {
            "t": 185.5,
            "v": 1.09
          },
          {
            "t": 189.1,
            "v": 1.54
          },
          {
            "t": 193.5,
            "v": 3.47
          },
          {
            "t": 198.0,
            "v": 3.44
          },
          {
            "t": 202.5,
            "v": 3.48
          },
          {
            "t": 207.0,
            "v": 3.37
          },
          {
            "t": 211.1,
            "v": 0.42
          },
          {
            "t": 214.7,
            "v": 2.3
          },
          {
            "t": 219.1,
            "v": 3.49
          },
          {
            "t": 223.6,
            "v": 3.56
          },
          {
            "t": 228.2,
            "v": 3.47
          },
          {
            "t": 232.7,
            "v": 3.27
          }
        ],
        "rss": [
          {
            "t": 3.6,
            "v": 6.0
          },
          {
            "t": 8.6,
            "v": 7.3
          },
          {
            "t": 13.6,
            "v": 10.0
          },
          {
            "t": 20.0,
            "v": 10.6
          },
          {
            "t": 24.7,
            "v": 11.0
          },
          {
            "t": 28.3,
            "v": 11.0
          },
          {
            "t": 33.0,
            "v": 11.0
          },
          {
            "t": 37.7,
            "v": 11.0
          },
          {
            "t": 42.4,
            "v": 11.0
          },
          {
            "t": 47.1,
            "v": 11.0
          },
          {
            "t": 50.7,
            "v": 11.0
          },
          {
            "t": 54.4,
            "v": 11.8
          },
          {
            "t": 62.0,
            "v": 22.0
          },
          {
            "t": 70.0,
            "v": 25.3
          },
          {
            "t": 75.7,
            "v": 22.5
          },
          {
            "t": 79.3,
            "v": 22.5
          },
          {
            "t": 82.9,
            "v": 22.5
          },
          {
            "t": 86.5,
            "v": 22.5
          },
          {
            "t": 90.2,
            "v": 22.5
          },
          {
            "t": 93.9,
            "v": 22.5
          },
          {
            "t": 97.6,
            "v": 22.5
          },
          {
            "t": 101.3,
            "v": 22.5
          },
          {
            "t": 105.0,
            "v": 22.5
          },
          {
            "t": 108.5,
            "v": 22.5
          },
          {
            "t": 112.1,
            "v": 22.5
          },
          {
            "t": 115.7,
            "v": 22.6
          },
          {
            "t": 119.4,
            "v": 22.6
          },
          {
            "t": 123.2,
            "v": 22.6
          },
          {
            "t": 126.9,
            "v": 22.6
          },
          {
            "t": 130.6,
            "v": 22.6
          },
          {
            "t": 134.2,
            "v": 22.6
          },
          {
            "t": 137.8,
            "v": 22.6
          },
          {
            "t": 141.9,
            "v": 22.4
          },
          {
            "t": 146.4,
            "v": 22.3
          },
          {
            "t": 150.8,
            "v": 22.2
          },
          {
            "t": 155.2,
            "v": 22.2
          },
          {
            "t": 159.7,
            "v": 22.2
          },
          {
            "t": 163.3,
            "v": 22.2
          },
          {
            "t": 167.8,
            "v": 22.2
          },
          {
            "t": 172.3,
            "v": 22.2
          },
          {
            "t": 176.7,
            "v": 22.2
          },
          {
            "t": 181.2,
            "v": 22.2
          },
          {
            "t": 185.5,
            "v": 22.2
          },
          {
            "t": 189.1,
            "v": 22.2
          },
          {
            "t": 193.5,
            "v": 22.2
          },
          {
            "t": 198.0,
            "v": 22.2
          },
          {
            "t": 202.5,
            "v": 22.2
          },
          {
            "t": 207.0,
            "v": 22.2
          },
          {
            "t": 211.1,
            "v": 22.2
          },
          {
            "t": 214.7,
            "v": 22.2
          },
          {
            "t": 219.1,
            "v": 22.2
          },
          {
            "t": 223.6,
            "v": 22.2
          },
          {
            "t": 228.2,
            "v": 22.2
          },
          {
            "t": 232.7,
            "v": 22.2
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
            "v": 92.0
          },
          {
            "t": 4.0,
            "v": 164.0
          },
          {
            "t": 6.0,
            "v": 109.5
          },
          {
            "t": 8.0,
            "v": 115.5
          },
          {
            "t": 10.0,
            "v": 145.5
          },
          {
            "t": 12.0,
            "v": 220.0
          },
          {
            "t": 14.0,
            "v": 364.0
          },
          {
            "t": 16.0,
            "v": 347.0
          },
          {
            "t": 18.0,
            "v": 119.0
          },
          {
            "t": 20.0,
            "v": 205.0
          },
          {
            "t": 22.0,
            "v": 30.5
          },
          {
            "t": 24.0,
            "v": 0.0
          },
          {
            "t": 26.0,
            "v": 11.0
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
            "v": 27.0
          },
          {
            "t": 54.0,
            "v": 35.5
          },
          {
            "t": 56.0,
            "v": 91.0
          },
          {
            "t": 58.0,
            "v": 182.5
          },
          {
            "t": 60.0,
            "v": 177.0
          },
          {
            "t": 62.0,
            "v": 210.0
          },
          {
            "t": 64.0,
            "v": 176.0
          },
          {
            "t": 66.0,
            "v": 223.5
          },
          {
            "t": 68.0,
            "v": 260.7
          },
          {
            "t": 70.0,
            "v": 161.5
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
            "v": 0.5
          },
          {
            "t": 78.0,
            "v": 0.0
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
            "v": 6.0
          },
          {
            "t": 140.0,
            "v": 1.5
          },
          {
            "t": 142.0,
            "v": 5.0
          },
          {
            "t": 144.0,
            "v": 10.5
          },
          {
            "t": 146.0,
            "v": 6.5
          },
          {
            "t": 148.0,
            "v": 1.0
          },
          {
            "t": 150.0,
            "v": 5.0
          },
          {
            "t": 152.0,
            "v": 5.5
          },
          {
            "t": 154.0,
            "v": 4.0
          },
          {
            "t": 156.0,
            "v": 14.0
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
            "v": 2.0
          },
          {
            "t": 164.0,
            "v": 13.5
          },
          {
            "t": 166.0,
            "v": 11.0
          },
          {
            "t": 168.0,
            "v": 4.0
          },
          {
            "t": 170.0,
            "v": 4.0
          },
          {
            "t": 172.0,
            "v": 6.0
          },
          {
            "t": 174.0,
            "v": 1.0
          },
          {
            "t": 176.0,
            "v": 11.0
          },
          {
            "t": 178.0,
            "v": 1.0
          },
          {
            "t": 180.0,
            "v": 4.5
          },
          {
            "t": 182.0,
            "v": 10.5
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
            "v": 21.0
          },
          {
            "t": 190.0,
            "v": 0.5
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
            "v": 4.5
          },
          {
            "t": 198.0,
            "v": 6.5
          },
          {
            "t": 200.0,
            "v": 4.0
          },
          {
            "t": 202.0,
            "v": 13.5
          },
          {
            "t": 204.0,
            "v": 0.0
          },
          {
            "t": 206.0,
            "v": 5.5
          },
          {
            "t": 208.0,
            "v": 0.5
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
            "v": 16.5
          },
          {
            "t": 216.0,
            "v": 4.5
          },
          {
            "t": 218.0,
            "v": 8.5
          },
          {
            "t": 220.0,
            "v": 2.0
          },
          {
            "t": 222.0,
            "v": 3.0
          },
          {
            "t": 224.0,
            "v": 4.5
          },
          {
            "t": 226.0,
            "v": 2.0
          },
          {
            "t": 228.0,
            "v": 6.5
          },
          {
            "t": 230.0,
            "v": 1.0
          },
          {
            "t": 232.0,
            "v": 4.5
          }
        ],
        "nvcswch": [
          {
            "t": 0.0,
            "v": 0.0
          },
          {
            "t": 2.0,
            "v": 19.5
          },
          {
            "t": 4.0,
            "v": 25.5
          },
          {
            "t": 6.0,
            "v": 41.5
          },
          {
            "t": 8.0,
            "v": 24.5
          },
          {
            "t": 10.0,
            "v": 30.5
          },
          {
            "t": 12.0,
            "v": 60.0
          },
          {
            "t": 14.0,
            "v": 134.5
          },
          {
            "t": 16.0,
            "v": 148.5
          },
          {
            "t": 18.0,
            "v": 38.0
          },
          {
            "t": 20.0,
            "v": 87.5
          },
          {
            "t": 22.0,
            "v": 67.0
          },
          {
            "t": 24.0,
            "v": 0.0
          },
          {
            "t": 26.0,
            "v": 3.5
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
            "v": 3.0
          },
          {
            "t": 54.0,
            "v": 2.0
          },
          {
            "t": 56.0,
            "v": 14.4
          },
          {
            "t": 58.0,
            "v": 55.5
          },
          {
            "t": 60.0,
            "v": 135.5
          },
          {
            "t": 62.0,
            "v": 63.0
          },
          {
            "t": 64.0,
            "v": 56.0
          },
          {
            "t": 66.0,
            "v": 79.0
          },
          {
            "t": 68.0,
            "v": 93.0
          },
          {
            "t": 70.0,
            "v": 110.0
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
            "v": 0.5
          },
          {
            "t": 150.0,
            "v": 0.5
          },
          {
            "t": 152.0,
            "v": 0.5
          },
          {
            "t": 154.0,
            "v": 1.0
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
            "v": 0.0
          },
          {
            "t": 164.0,
            "v": 1.5
          },
          {
            "t": 166.0,
            "v": 2.0
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
            "v": 5.5
          },
          {
            "t": 178.0,
            "v": 0.0
          },
          {
            "t": 180.0,
            "v": 3.0
          },
          {
            "t": 182.0,
            "v": 0.5
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
            "v": 2.5
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
            "v": 0.5
          },
          {
            "t": 200.0,
            "v": 0.0
          },
          {
            "t": 202.0,
            "v": 1.5
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
            "v": 1.5
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
            "t": 82.9,
            "v": 0.07
          },
          {
            "t": 86.5,
            "v": 0.14
          },
          {
            "t": 90.2,
            "v": 0.07
          },
          {
            "t": 93.9,
            "v": 0.07
          },
          {
            "t": 97.6,
            "v": 0.07
          },
          {
            "t": 119.4,
            "v": 0.07
          },
          {
            "t": 123.2,
            "v": 0.07
          },
          {
            "t": 126.9,
            "v": 0.07
          }
        ],
        "rss": [
          {
            "t": 3.6,
            "v": 2.7
          },
          {
            "t": 8.6,
            "v": 2.7
          },
          {
            "t": 13.6,
            "v": 2.7
          },
          {
            "t": 20.0,
            "v": 2.7
          },
          {
            "t": 24.7,
            "v": 2.7
          },
          {
            "t": 28.3,
            "v": 2.7
          },
          {
            "t": 33.0,
            "v": 2.7
          },
          {
            "t": 37.7,
            "v": 2.7
          },
          {
            "t": 42.4,
            "v": 2.7
          },
          {
            "t": 47.1,
            "v": 2.7
          },
          {
            "t": 50.7,
            "v": 2.7
          },
          {
            "t": 54.4,
            "v": 2.7
          },
          {
            "t": 62.0,
            "v": 2.7
          },
          {
            "t": 70.0,
            "v": 2.7
          },
          {
            "t": 75.7,
            "v": 2.7
          },
          {
            "t": 79.3,
            "v": 3.0
          },
          {
            "t": 82.9,
            "v": 3.0
          },
          {
            "t": 86.5,
            "v": 3.0
          },
          {
            "t": 90.2,
            "v": 3.0
          },
          {
            "t": 93.9,
            "v": 3.0
          },
          {
            "t": 97.6,
            "v": 3.0
          },
          {
            "t": 101.3,
            "v": 3.0
          },
          {
            "t": 105.0,
            "v": 3.0
          },
          {
            "t": 108.5,
            "v": 3.0
          },
          {
            "t": 112.1,
            "v": 3.0
          },
          {
            "t": 115.7,
            "v": 3.0
          },
          {
            "t": 119.4,
            "v": 3.0
          },
          {
            "t": 123.2,
            "v": 3.0
          },
          {
            "t": 126.9,
            "v": 3.0
          },
          {
            "t": 130.6,
            "v": 3.0
          },
          {
            "t": 134.2,
            "v": 3.0
          },
          {
            "t": 137.8,
            "v": 3.0
          },
          {
            "t": 141.9,
            "v": 3.0
          },
          {
            "t": 146.4,
            "v": 3.0
          },
          {
            "t": 150.8,
            "v": 3.0
          },
          {
            "t": 155.2,
            "v": 3.0
          },
          {
            "t": 159.7,
            "v": 3.0
          },
          {
            "t": 163.3,
            "v": 3.0
          },
          {
            "t": 167.8,
            "v": 3.0
          },
          {
            "t": 172.3,
            "v": 3.0
          },
          {
            "t": 176.7,
            "v": 3.0
          },
          {
            "t": 181.2,
            "v": 3.0
          },
          {
            "t": 185.5,
            "v": 3.0
          },
          {
            "t": 189.1,
            "v": 3.0
          },
          {
            "t": 193.5,
            "v": 3.0
          },
          {
            "t": 198.0,
            "v": 3.0
          },
          {
            "t": 202.5,
            "v": 3.0
          },
          {
            "t": 207.0,
            "v": 3.0
          },
          {
            "t": 211.1,
            "v": 3.0
          },
          {
            "t": 214.7,
            "v": 3.0
          },
          {
            "t": 219.1,
            "v": 3.0
          },
          {
            "t": 223.6,
            "v": 3.0
          },
          {
            "t": 228.2,
            "v": 3.0
          },
          {
            "t": 232.7,
            "v": 3.0
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
            "v": 2.0
          },
          {
            "t": 110.0,
            "v": 0.0
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
            "v": 8.5
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
            "v": 2.0
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
            "v": 3.0
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
            "v": 1.5
          },
          {
            "t": 80.0,
            "v": 6.0
          },
          {
            "t": 82.0,
            "v": 2.5
          },
          {
            "t": 84.0,
            "v": 5.5
          },
          {
            "t": 86.0,
            "v": 1.0
          },
          {
            "t": 88.0,
            "v": 3.0
          },
          {
            "t": 90.0,
            "v": 3.5
          },
          {
            "t": 92.0,
            "v": 7.5
          },
          {
            "t": 94.0,
            "v": 4.5
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
            "v": 1.5
          },
          {
            "t": 110.0,
            "v": 2.5
          },
          {
            "t": 112.0,
            "v": 3.0
          },
          {
            "t": 114.0,
            "v": 2.5
          },
          {
            "t": 116.0,
            "v": 1.0
          },
          {
            "t": 118.0,
            "v": 3.5
          },
          {
            "t": 120.0,
            "v": 3.0
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
            "v": 5.0
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
            "t": 82.9,
            "v": 0.14
          },
          {
            "t": 86.5,
            "v": 0.21
          },
          {
            "t": 90.2,
            "v": 0.14
          },
          {
            "t": 93.9,
            "v": 0.2
          },
          {
            "t": 97.6,
            "v": 0.2
          },
          {
            "t": 101.3,
            "v": 0.13
          },
          {
            "t": 115.7,
            "v": 0.28
          },
          {
            "t": 119.4,
            "v": 0.48
          },
          {
            "t": 123.2,
            "v": 0.47
          },
          {
            "t": 126.9,
            "v": 0.53
          },
          {
            "t": 130.6,
            "v": 0.41
          },
          {
            "t": 134.2,
            "v": 0.14
          }
        ],
        "rss": [
          {
            "t": 3.6,
            "v": 3.3
          },
          {
            "t": 8.6,
            "v": 3.3
          },
          {
            "t": 13.6,
            "v": 3.3
          },
          {
            "t": 20.0,
            "v": 3.3
          },
          {
            "t": 24.7,
            "v": 3.3
          },
          {
            "t": 28.3,
            "v": 3.3
          },
          {
            "t": 33.0,
            "v": 3.3
          },
          {
            "t": 37.7,
            "v": 3.3
          },
          {
            "t": 42.4,
            "v": 3.3
          },
          {
            "t": 47.1,
            "v": 3.3
          },
          {
            "t": 50.7,
            "v": 3.3
          },
          {
            "t": 54.4,
            "v": 3.3
          },
          {
            "t": 62.0,
            "v": 3.3
          },
          {
            "t": 70.0,
            "v": 3.3
          },
          {
            "t": 75.7,
            "v": 3.3
          },
          {
            "t": 79.3,
            "v": 4.0
          },
          {
            "t": 82.9,
            "v": 4.1
          },
          {
            "t": 86.5,
            "v": 4.1
          },
          {
            "t": 90.2,
            "v": 4.2
          },
          {
            "t": 93.9,
            "v": 4.2
          },
          {
            "t": 97.6,
            "v": 4.2
          },
          {
            "t": 101.3,
            "v": 4.2
          },
          {
            "t": 105.0,
            "v": 4.2
          },
          {
            "t": 108.5,
            "v": 4.2
          },
          {
            "t": 112.1,
            "v": 4.2
          },
          {
            "t": 115.7,
            "v": 4.3
          },
          {
            "t": 119.4,
            "v": 4.3
          },
          {
            "t": 123.2,
            "v": 4.4
          },
          {
            "t": 126.9,
            "v": 4.4
          },
          {
            "t": 130.6,
            "v": 4.4
          },
          {
            "t": 134.2,
            "v": 4.4
          },
          {
            "t": 137.8,
            "v": 4.4
          },
          {
            "t": 141.9,
            "v": 4.4
          },
          {
            "t": 146.4,
            "v": 4.4
          },
          {
            "t": 150.8,
            "v": 4.4
          },
          {
            "t": 155.2,
            "v": 4.4
          },
          {
            "t": 159.7,
            "v": 4.4
          },
          {
            "t": 163.3,
            "v": 4.4
          },
          {
            "t": 167.8,
            "v": 4.4
          },
          {
            "t": 172.3,
            "v": 4.4
          },
          {
            "t": 176.7,
            "v": 4.4
          },
          {
            "t": 181.2,
            "v": 4.4
          },
          {
            "t": 185.5,
            "v": 4.4
          },
          {
            "t": 189.1,
            "v": 4.4
          },
          {
            "t": 193.5,
            "v": 4.4
          },
          {
            "t": 198.0,
            "v": 4.4
          },
          {
            "t": 202.5,
            "v": 4.4
          },
          {
            "t": 207.0,
            "v": 4.4
          },
          {
            "t": 211.1,
            "v": 4.4
          },
          {
            "t": 214.7,
            "v": 4.4
          },
          {
            "t": 219.1,
            "v": 4.4
          },
          {
            "t": 223.6,
            "v": 4.4
          },
          {
            "t": 228.2,
            "v": 4.4
          },
          {
            "t": 232.7,
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
            "t": 8.6,
            "v": 22.43
          },
          {
            "t": 13.6,
            "v": 23.58
          },
          {
            "t": 20.0,
            "v": 20.02
          },
          {
            "t": 24.7,
            "v": 1.17
          },
          {
            "t": 28.3,
            "v": 18.43
          },
          {
            "t": 33.0,
            "v": 22.97
          },
          {
            "t": 37.7,
            "v": 22.75
          },
          {
            "t": 42.4,
            "v": 22.17
          },
          {
            "t": 47.1,
            "v": 15.43
          },
          {
            "t": 50.7,
            "v": 0.07
          },
          {
            "t": 54.4,
            "v": 62.11
          },
          {
            "t": 62.0,
            "v": 38.37
          },
          {
            "t": 79.3,
            "v": 1.39
          },
          {
            "t": 82.9,
            "v": 2.62
          },
          {
            "t": 86.5,
            "v": 3.15
          },
          {
            "t": 90.2,
            "v": 3.26
          },
          {
            "t": 93.9,
            "v": 3.25
          },
          {
            "t": 97.6,
            "v": 4.76
          },
          {
            "t": 101.3,
            "v": 2.14
          },
          {
            "t": 105.0,
            "v": 0.55
          },
          {
            "t": 108.5,
            "v": 0.35
          },
          {
            "t": 112.1,
            "v": 1.75
          },
          {
            "t": 115.7,
            "v": 2.69
          },
          {
            "t": 119.4,
            "v": 4.97
          },
          {
            "t": 123.2,
            "v": 3.61
          },
          {
            "t": 126.9,
            "v": 3.27
          },
          {
            "t": 130.6,
            "v": 2.58
          },
          {
            "t": 134.2,
            "v": 0.55
          },
          {
            "t": 137.8,
            "v": 4.56
          },
          {
            "t": 141.9,
            "v": 25.87
          },
          {
            "t": 146.4,
            "v": 22.72
          },
          {
            "t": 150.8,
            "v": 23.69
          },
          {
            "t": 155.2,
            "v": 24.0
          },
          {
            "t": 159.7,
            "v": 11.11
          },
          {
            "t": 163.3,
            "v": 6.98
          },
          {
            "t": 167.8,
            "v": 27.04
          },
          {
            "t": 172.3,
            "v": 24.27
          },
          {
            "t": 176.7,
            "v": 24.13
          },
          {
            "t": 181.2,
            "v": 22.71
          },
          {
            "t": 185.5,
            "v": 6.36
          },
          {
            "t": 189.1,
            "v": 14.33
          },
          {
            "t": 193.5,
            "v": 26.87
          },
          {
            "t": 198.0,
            "v": 25.67
          },
          {
            "t": 202.5,
            "v": 24.29
          },
          {
            "t": 207.0,
            "v": 26.11
          },
          {
            "t": 211.1,
            "v": 2.49
          },
          {
            "t": 214.7,
            "v": 18.4
          },
          {
            "t": 219.1,
            "v": 26.12
          },
          {
            "t": 223.6,
            "v": 26.45
          },
          {
            "t": 228.2,
            "v": 25.81
          }
        ],
        "rss": [
          {
            "t": 3.6,
            "v": 129.1
          },
          {
            "t": 8.6,
            "v": 141.7
          },
          {
            "t": 13.6,
            "v": 167.5
          },
          {
            "t": 20.0,
            "v": 195.4
          },
          {
            "t": 24.7,
            "v": 195.4
          },
          {
            "t": 28.3,
            "v": 192.9
          },
          {
            "t": 33.0,
            "v": 199.2
          },
          {
            "t": 37.7,
            "v": 205.1
          },
          {
            "t": 42.4,
            "v": 205.2
          },
          {
            "t": 47.1,
            "v": 213.7
          },
          {
            "t": 50.7,
            "v": 213.7
          },
          {
            "t": 54.4,
            "v": 362.0
          },
          {
            "t": 62.0,
            "v": 483.4
          },
          {
            "t": 75.7,
            "v": 366.2
          },
          {
            "t": 79.3,
            "v": 366.6
          },
          {
            "t": 82.9,
            "v": 366.7
          },
          {
            "t": 86.5,
            "v": 368.5
          },
          {
            "t": 90.2,
            "v": 374.5
          },
          {
            "t": 93.9,
            "v": 397.9
          },
          {
            "t": 97.6,
            "v": 420.9
          },
          {
            "t": 101.3,
            "v": 421.0
          },
          {
            "t": 105.0,
            "v": 421.0
          },
          {
            "t": 108.5,
            "v": 421.1
          },
          {
            "t": 112.1,
            "v": 421.6
          },
          {
            "t": 115.7,
            "v": 422.3
          },
          {
            "t": 119.4,
            "v": 434.5
          },
          {
            "t": 123.2,
            "v": 434.5
          },
          {
            "t": 126.9,
            "v": 434.6
          },
          {
            "t": 130.6,
            "v": 434.6
          },
          {
            "t": 134.2,
            "v": 434.7
          },
          {
            "t": 137.8,
            "v": 453.1
          },
          {
            "t": 141.9,
            "v": 504.3
          },
          {
            "t": 146.4,
            "v": 507.9
          },
          {
            "t": 150.8,
            "v": 510.9
          },
          {
            "t": 155.2,
            "v": 520.0
          },
          {
            "t": 159.7,
            "v": 520.0
          },
          {
            "t": 163.3,
            "v": 520.1
          },
          {
            "t": 167.8,
            "v": 525.7
          },
          {
            "t": 172.3,
            "v": 525.7
          },
          {
            "t": 176.7,
            "v": 527.8
          },
          {
            "t": 181.2,
            "v": 531.0
          },
          {
            "t": 185.5,
            "v": 534.5
          },
          {
            "t": 189.1,
            "v": 549.6
          },
          {
            "t": 193.5,
            "v": 555.3
          },
          {
            "t": 198.0,
            "v": 555.5
          },
          {
            "t": 202.5,
            "v": 555.5
          },
          {
            "t": 207.0,
            "v": 559.7
          },
          {
            "t": 211.1,
            "v": 559.8
          },
          {
            "t": 214.7,
            "v": 559.8
          },
          {
            "t": 219.1,
            "v": 559.8
          },
          {
            "t": 223.6,
            "v": 579.5
          },
          {
            "t": 228.2,
            "v": 579.5
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
            "v": 21.0
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
            "v": 30.0
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
            "v": 4.5
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
            "t": 8.6,
            "v": 21.14
          },
          {
            "t": 13.6,
            "v": 22.76
          },
          {
            "t": 20.0,
            "v": 19.24
          },
          {
            "t": 24.7,
            "v": 2.66
          },
          {
            "t": 28.3,
            "v": 14.54
          },
          {
            "t": 33.0,
            "v": 20.11
          },
          {
            "t": 37.7,
            "v": 19.95
          },
          {
            "t": 42.4,
            "v": 20.09
          },
          {
            "t": 47.1,
            "v": 14.78
          },
          {
            "t": 54.4,
            "v": 33.21
          },
          {
            "t": 62.0,
            "v": 30.24
          }
        ],
        "rss": [
          {
            "t": 3.6,
            "v": 33.1
          },
          {
            "t": 8.6,
            "v": 34.2
          },
          {
            "t": 13.6,
            "v": 41.1
          },
          {
            "t": 20.0,
            "v": 42.2
          },
          {
            "t": 24.7,
            "v": 44.9
          },
          {
            "t": 28.3,
            "v": 40.6
          },
          {
            "t": 33.0,
            "v": 39.5
          },
          {
            "t": 37.7,
            "v": 39.4
          },
          {
            "t": 42.4,
            "v": 39.3
          },
          {
            "t": 47.1,
            "v": 39.0
          },
          {
            "t": 50.7,
            "v": 39.0
          },
          {
            "t": 54.4,
            "v": 64.9
          },
          {
            "t": 62.0,
            "v": 104.2
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
            "v": 19.4
          },
          {
            "t": 2.0,
            "v": 1676.0
          },
          {
            "t": 4.0,
            "v": 1567.0
          },
          {
            "t": 6.0,
            "v": 2588.5
          },
          {
            "t": 8.0,
            "v": 698.5
          },
          {
            "t": 10.0,
            "v": 1742.0
          },
          {
            "t": 12.0,
            "v": 1748.0
          },
          {
            "t": 14.0,
            "v": 585.0
          },
          {
            "t": 16.0,
            "v": 650.5
          },
          {
            "t": 18.0,
            "v": 2188.0
          },
          {
            "t": 20.0,
            "v": 1516.0
          },
          {
            "t": 22.0,
            "v": 38.0
          },
          {
            "t": 24.0,
            "v": 0.0
          },
          {
            "t": 26.0,
            "v": 1274.5
          },
          {
            "t": 28.0,
            "v": 2147.5
          },
          {
            "t": 30.0,
            "v": 2482.5
          },
          {
            "t": 32.0,
            "v": 2600.0
          },
          {
            "t": 34.0,
            "v": 1944.0
          },
          {
            "t": 36.0,
            "v": 440.0
          },
          {
            "t": 38.0,
            "v": 1686.6
          },
          {
            "t": 40.0,
            "v": 2651.5
          },
          {
            "t": 42.0,
            "v": 1968.0
          },
          {
            "t": 44.0,
            "v": 2691.5
          },
          {
            "t": 46.0,
            "v": 1681.0
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
            "v": 551.5
          },
          {
            "t": 54.0,
            "v": 564.5
          },
          {
            "t": 56.0,
            "v": 382.6
          },
          {
            "t": 58.0,
            "v": 216.5
          },
          {
            "t": 60.0,
            "v": 254.0
          },
          {
            "t": 62.0,
            "v": 298.5
          },
          {
            "t": 64.0,
            "v": 361.0
          },
          {
            "t": 66.0,
            "v": 271.0
          },
          {
            "t": 68.0,
            "v": 306.5
          },
          {
            "t": 70.0,
            "v": 401.0
          }
        ],
        "nvcswch": [
          {
            "t": 0.0,
            "v": 7.5
          },
          {
            "t": 2.0,
            "v": 275.0
          },
          {
            "t": 4.0,
            "v": 532.5
          },
          {
            "t": 6.0,
            "v": 408.0
          },
          {
            "t": 8.0,
            "v": 204.5
          },
          {
            "t": 10.0,
            "v": 456.0
          },
          {
            "t": 12.0,
            "v": 589.5
          },
          {
            "t": 14.0,
            "v": 236.0
          },
          {
            "t": 16.0,
            "v": 275.5
          },
          {
            "t": 18.0,
            "v": 328.5
          },
          {
            "t": 20.0,
            "v": 457.0
          },
          {
            "t": 22.0,
            "v": 20.5
          },
          {
            "t": 24.0,
            "v": 0.0
          },
          {
            "t": 26.0,
            "v": 131.0
          },
          {
            "t": 28.0,
            "v": 631.5
          },
          {
            "t": 30.0,
            "v": 388.0
          },
          {
            "t": 32.0,
            "v": 546.0
          },
          {
            "t": 34.0,
            "v": 505.5
          },
          {
            "t": 36.0,
            "v": 131.5
          },
          {
            "t": 38.0,
            "v": 522.9
          },
          {
            "t": 40.0,
            "v": 294.0
          },
          {
            "t": 42.0,
            "v": 380.5
          },
          {
            "t": 44.0,
            "v": 458.5
          },
          {
            "t": 46.0,
            "v": 204.0
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
            "v": 266.0
          },
          {
            "t": 54.0,
            "v": 349.0
          },
          {
            "t": 56.0,
            "v": 251.7
          },
          {
            "t": 58.0,
            "v": 202.0
          },
          {
            "t": 60.0,
            "v": 238.5
          },
          {
            "t": 62.0,
            "v": 360.0
          },
          {
            "t": 64.0,
            "v": 261.5
          },
          {
            "t": 66.0,
            "v": 326.5
          },
          {
            "t": 68.0,
            "v": 197.0
          },
          {
            "t": 70.0,
            "v": 310.0
          }
        ]
      },
      "frpc": {
        "cpu": [
          {
            "t": 8.6,
            "v": 15.92
          },
          {
            "t": 13.6,
            "v": 19.35
          },
          {
            "t": 20.0,
            "v": 15.53
          },
          {
            "t": 24.7,
            "v": 1.7
          },
          {
            "t": 28.3,
            "v": 9.74
          },
          {
            "t": 33.0,
            "v": 12.49
          },
          {
            "t": 37.7,
            "v": 12.4
          },
          {
            "t": 42.4,
            "v": 12.47
          },
          {
            "t": 47.1,
            "v": 8.73
          },
          {
            "t": 54.4,
            "v": 19.56
          },
          {
            "t": 62.0,
            "v": 20.15
          }
        ],
        "rss": [
          {
            "t": 3.6,
            "v": 23.6
          },
          {
            "t": 8.6,
            "v": 27.4
          },
          {
            "t": 13.6,
            "v": 29.0
          },
          {
            "t": 20.0,
            "v": 28.5
          },
          {
            "t": 24.7,
            "v": 53.6
          },
          {
            "t": 28.3,
            "v": 57.5
          },
          {
            "t": 33.0,
            "v": 57.7
          },
          {
            "t": 37.7,
            "v": 57.7
          },
          {
            "t": 42.4,
            "v": 57.7
          },
          {
            "t": 47.1,
            "v": 57.7
          },
          {
            "t": 50.7,
            "v": 57.7
          },
          {
            "t": 54.4,
            "v": 69.6
          },
          {
            "t": 62.0,
            "v": 122.5
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
            "v": 7.0
          },
          {
            "t": 2.0,
            "v": 1749.0
          },
          {
            "t": 4.0,
            "v": 1483.0
          },
          {
            "t": 6.0,
            "v": 2499.0
          },
          {
            "t": 8.0,
            "v": 1358.0
          },
          {
            "t": 10.0,
            "v": 1497.5
          },
          {
            "t": 12.0,
            "v": 1755.5
          },
          {
            "t": 14.0,
            "v": 224.0
          },
          {
            "t": 16.0,
            "v": 423.5
          },
          {
            "t": 18.0,
            "v": 0.0
          },
          {
            "t": 20.0,
            "v": 1161.5
          },
          {
            "t": 22.0,
            "v": 51.0
          },
          {
            "t": 24.0,
            "v": 0.0
          },
          {
            "t": 26.0,
            "v": 1098.0
          },
          {
            "t": 28.0,
            "v": 2251.5
          },
          {
            "t": 30.0,
            "v": 2809.5
          },
          {
            "t": 32.0,
            "v": 2142.5
          },
          {
            "t": 34.0,
            "v": 1982.5
          },
          {
            "t": 36.0,
            "v": 2157.5
          },
          {
            "t": 38.0,
            "v": 1778.6
          },
          {
            "t": 40.0,
            "v": 2526.0
          },
          {
            "t": 42.0,
            "v": 1806.5
          },
          {
            "t": 44.0,
            "v": 1905.5
          },
          {
            "t": 46.0,
            "v": 1487.0
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
            "v": 970.5
          },
          {
            "t": 54.0,
            "v": 657.5
          },
          {
            "t": 56.0,
            "v": 474.1
          },
          {
            "t": 58.0,
            "v": 344.0
          },
          {
            "t": 60.0,
            "v": 259.0
          },
          {
            "t": 62.0,
            "v": 337.5
          },
          {
            "t": 64.0,
            "v": 234.0
          },
          {
            "t": 66.0,
            "v": 291.0
          },
          {
            "t": 68.0,
            "v": 556.2
          },
          {
            "t": 70.0,
            "v": 654.0
          }
        ],
        "nvcswch": [
          {
            "t": 0.0,
            "v": 4.5
          },
          {
            "t": 2.0,
            "v": 257.0
          },
          {
            "t": 4.0,
            "v": 485.5
          },
          {
            "t": 6.0,
            "v": 316.5
          },
          {
            "t": 8.0,
            "v": 275.5
          },
          {
            "t": 10.0,
            "v": 387.0
          },
          {
            "t": 12.0,
            "v": 530.0
          },
          {
            "t": 14.0,
            "v": 147.5
          },
          {
            "t": 16.0,
            "v": 192.5
          },
          {
            "t": 18.0,
            "v": 0.0
          },
          {
            "t": 20.0,
            "v": 395.0
          },
          {
            "t": 22.0,
            "v": 16.0
          },
          {
            "t": 24.0,
            "v": 0.0
          },
          {
            "t": 26.0,
            "v": 93.0
          },
          {
            "t": 28.0,
            "v": 566.0
          },
          {
            "t": 30.0,
            "v": 443.5
          },
          {
            "t": 32.0,
            "v": 320.5
          },
          {
            "t": 34.0,
            "v": 445.0
          },
          {
            "t": 36.0,
            "v": 174.0
          },
          {
            "t": 38.0,
            "v": 461.7
          },
          {
            "t": 40.0,
            "v": 276.0
          },
          {
            "t": 42.0,
            "v": 370.5
          },
          {
            "t": 44.0,
            "v": 290.5
          },
          {
            "t": 46.0,
            "v": 163.5
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
            "v": 276.5
          },
          {
            "t": 54.0,
            "v": 189.0
          },
          {
            "t": 56.0,
            "v": 229.8
          },
          {
            "t": 58.0,
            "v": 132.5
          },
          {
            "t": 60.0,
            "v": 203.5
          },
          {
            "t": 62.0,
            "v": 153.0
          },
          {
            "t": 64.0,
            "v": 90.5
          },
          {
            "t": 66.0,
            "v": 396.0
          },
          {
            "t": 68.0,
            "v": 311.4
          },
          {
            "t": 70.0,
            "v": 321.0
          }
        ]
      },
      "other": {
        "cpu": [
          {
            "t": 3.6,
            "v": 4.28
          },
          {
            "t": 8.6,
            "v": 1.79
          },
          {
            "t": 13.6,
            "v": 14.06
          },
          {
            "t": 20.0,
            "v": 10.22
          },
          {
            "t": 24.7,
            "v": 5.79
          },
          {
            "t": 28.3,
            "v": 2.5
          },
          {
            "t": 33.0,
            "v": 2.28
          },
          {
            "t": 37.7,
            "v": 2.37
          },
          {
            "t": 42.4,
            "v": 1.87
          },
          {
            "t": 47.1,
            "v": 1.71
          },
          {
            "t": 50.7,
            "v": 2.07
          },
          {
            "t": 54.4,
            "v": 2.49
          },
          {
            "t": 62.0,
            "v": 1.86
          },
          {
            "t": 70.0,
            "v": 1.69
          },
          {
            "t": 75.7,
            "v": 2.14
          },
          {
            "t": 79.3,
            "v": 1.67
          },
          {
            "t": 82.9,
            "v": 1.72
          },
          {
            "t": 86.5,
            "v": 1.71
          },
          {
            "t": 90.2,
            "v": 1.7
          },
          {
            "t": 93.9,
            "v": 1.76
          },
          {
            "t": 97.6,
            "v": 2.04
          },
          {
            "t": 101.3,
            "v": 2.21
          },
          {
            "t": 105.0,
            "v": 1.37
          },
          {
            "t": 108.5,
            "v": 1.9
          },
          {
            "t": 112.1,
            "v": 1.68
          },
          {
            "t": 115.7,
            "v": 1.73
          },
          {
            "t": 119.4,
            "v": 2.04
          },
          {
            "t": 123.2,
            "v": 1.47
          },
          {
            "t": 126.9,
            "v": 1.74
          },
          {
            "t": 130.6,
            "v": 1.7
          },
          {
            "t": 134.2,
            "v": 1.44
          },
          {
            "t": 137.8,
            "v": 1.61
          },
          {
            "t": 141.9,
            "v": 2.42
          },
          {
            "t": 146.4,
            "v": 1.69
          },
          {
            "t": 150.8,
            "v": 2.09
          },
          {
            "t": 155.2,
            "v": 2.03
          },
          {
            "t": 159.7,
            "v": 2.4
          },
          {
            "t": 163.3,
            "v": 1.8
          },
          {
            "t": 167.8,
            "v": 2.32
          },
          {
            "t": 172.3,
            "v": 1.77
          },
          {
            "t": 176.7,
            "v": 1.91
          },
          {
            "t": 181.2,
            "v": 2.25
          },
          {
            "t": 185.5,
            "v": 1.55
          },
          {
            "t": 189.1,
            "v": 2.04
          },
          {
            "t": 193.5,
            "v": 2.05
          },
          {
            "t": 198.0,
            "v": 2.0
          },
          {
            "t": 202.5,
            "v": 1.91
          },
          {
            "t": 207.0,
            "v": 1.94
          },
          {
            "t": 211.1,
            "v": 1.52
          },
          {
            "t": 214.7,
            "v": 2.16
          },
          {
            "t": 219.1,
            "v": 2.76
          },
          {
            "t": 223.6,
            "v": 1.83
          },
          {
            "t": 228.2,
            "v": 2.21
          },
          {
            "t": 232.7,
            "v": 1.83
          }
        ],
        "rss": [
          {
            "t": 3.6,
            "v": 128.5
          },
          {
            "t": 8.6,
            "v": 129.1
          },
          {
            "t": 13.6,
            "v": 131.6
          },
          {
            "t": 20.0,
            "v": 130.5
          },
          {
            "t": 24.7,
            "v": 131.0
          },
          {
            "t": 28.3,
            "v": 131.4
          },
          {
            "t": 33.0,
            "v": 132.4
          },
          {
            "t": 37.7,
            "v": 132.9
          },
          {
            "t": 42.4,
            "v": 132.9
          },
          {
            "t": 47.1,
            "v": 132.9
          },
          {
            "t": 50.7,
            "v": 132.9
          },
          {
            "t": 54.4,
            "v": 132.9
          },
          {
            "t": 62.0,
            "v": 133.6
          },
          {
            "t": 70.0,
            "v": 133.6
          },
          {
            "t": 75.7,
            "v": 135.1
          },
          {
            "t": 79.3,
            "v": 135.1
          },
          {
            "t": 82.9,
            "v": 135.1
          },
          {
            "t": 86.5,
            "v": 135.1
          },
          {
            "t": 90.2,
            "v": 135.1
          },
          {
            "t": 93.9,
            "v": 135.1
          },
          {
            "t": 97.6,
            "v": 133.5
          },
          {
            "t": 101.3,
            "v": 133.5
          },
          {
            "t": 105.0,
            "v": 133.5
          },
          {
            "t": 108.5,
            "v": 133.5
          },
          {
            "t": 112.1,
            "v": 133.6
          },
          {
            "t": 115.7,
            "v": 133.6
          },
          {
            "t": 119.4,
            "v": 133.6
          },
          {
            "t": 123.2,
            "v": 133.6
          },
          {
            "t": 126.9,
            "v": 133.6
          },
          {
            "t": 130.6,
            "v": 133.7
          },
          {
            "t": 134.2,
            "v": 133.7
          },
          {
            "t": 137.8,
            "v": 133.7
          },
          {
            "t": 141.9,
            "v": 134.1
          },
          {
            "t": 146.4,
            "v": 134.1
          },
          {
            "t": 150.8,
            "v": 134.1
          },
          {
            "t": 155.2,
            "v": 134.1
          },
          {
            "t": 159.7,
            "v": 134.0
          },
          {
            "t": 163.3,
            "v": 134.0
          },
          {
            "t": 167.8,
            "v": 134.0
          },
          {
            "t": 172.3,
            "v": 134.0
          },
          {
            "t": 176.7,
            "v": 134.0
          },
          {
            "t": 181.2,
            "v": 134.5
          },
          {
            "t": 185.5,
            "v": 134.5
          },
          {
            "t": 189.1,
            "v": 134.5
          },
          {
            "t": 193.5,
            "v": 134.5
          },
          {
            "t": 198.0,
            "v": 134.4
          },
          {
            "t": 202.5,
            "v": 134.4
          },
          {
            "t": 207.0,
            "v": 134.4
          },
          {
            "t": 211.1,
            "v": 134.5
          },
          {
            "t": 214.7,
            "v": 134.5
          },
          {
            "t": 219.1,
            "v": 134.5
          },
          {
            "t": 223.6,
            "v": 134.5
          },
          {
            "t": 228.2,
            "v": 134.5
          },
          {
            "t": 232.7,
            "v": 134.4
          }
        ],
        "read_kbs": [
          {
            "t": 0.0,
            "v": 84.8
          },
          {
            "t": 2.0,
            "v": -148.0
          },
          {
            "t": 4.0,
            "v": -148.0
          },
          {
            "t": 6.0,
            "v": -148.0
          },
          {
            "t": 8.0,
            "v": -148.0
          },
          {
            "t": 10.0,
            "v": -148.0
          },
          {
            "t": 12.0,
            "v": -148.0
          },
          {
            "t": 14.0,
            "v": -148.0
          },
          {
            "t": 16.0,
            "v": -148.0
          },
          {
            "t": 18.0,
            "v": -148.0
          },
          {
            "t": 20.0,
            "v": -148.0
          },
          {
            "t": 22.0,
            "v": -148.0
          },
          {
            "t": 24.0,
            "v": -148.0
          },
          {
            "t": 26.0,
            "v": -148.0
          },
          {
            "t": 28.0,
            "v": -148.0
          },
          {
            "t": 30.0,
            "v": -148.0
          },
          {
            "t": 32.0,
            "v": -148.0
          },
          {
            "t": 34.0,
            "v": -148.0
          },
          {
            "t": 36.0,
            "v": -148.0
          },
          {
            "t": 38.0,
            "v": -148.0
          },
          {
            "t": 40.0,
            "v": -148.0
          },
          {
            "t": 42.0,
            "v": -148.0
          },
          {
            "t": 44.0,
            "v": -148.0
          },
          {
            "t": 46.0,
            "v": -148.0
          },
          {
            "t": 48.0,
            "v": -148.0
          },
          {
            "t": 50.0,
            "v": -148.0
          },
          {
            "t": 52.0,
            "v": -148.0
          },
          {
            "t": 54.0,
            "v": -148.0
          },
          {
            "t": 56.0,
            "v": -148.0
          },
          {
            "t": 58.0,
            "v": -148.0
          },
          {
            "t": 60.0,
            "v": -148.0
          },
          {
            "t": 62.0,
            "v": -148.0
          },
          {
            "t": 64.0,
            "v": -148.0
          },
          {
            "t": 66.0,
            "v": -148.0
          },
          {
            "t": 68.0,
            "v": -148.0
          },
          {
            "t": 70.0,
            "v": -148.0
          },
          {
            "t": 72.0,
            "v": -148.0
          },
          {
            "t": 74.0,
            "v": -148.0
          },
          {
            "t": 76.0,
            "v": -148.0
          },
          {
            "t": 78.0,
            "v": -148.0
          },
          {
            "t": 80.0,
            "v": -148.0
          },
          {
            "t": 82.0,
            "v": -148.0
          },
          {
            "t": 84.0,
            "v": -148.0
          },
          {
            "t": 86.0,
            "v": -148.0
          },
          {
            "t": 88.0,
            "v": -148.0
          },
          {
            "t": 90.0,
            "v": -148.0
          },
          {
            "t": 92.0,
            "v": -148.0
          },
          {
            "t": 94.0,
            "v": -148.0
          },
          {
            "t": 96.0,
            "v": -148.0
          },
          {
            "t": 98.0,
            "v": -148.0
          },
          {
            "t": 100.0,
            "v": -148.0
          },
          {
            "t": 102.0,
            "v": -148.0
          },
          {
            "t": 104.0,
            "v": -148.0
          },
          {
            "t": 106.0,
            "v": -148.0
          },
          {
            "t": 108.0,
            "v": -148.0
          },
          {
            "t": 110.0,
            "v": -148.0
          },
          {
            "t": 112.0,
            "v": -148.0
          },
          {
            "t": 114.0,
            "v": -148.0
          },
          {
            "t": 116.0,
            "v": -148.0
          },
          {
            "t": 118.0,
            "v": -148.0
          },
          {
            "t": 120.0,
            "v": -148.0
          },
          {
            "t": 122.0,
            "v": -148.0
          },
          {
            "t": 124.0,
            "v": -148.0
          },
          {
            "t": 126.0,
            "v": -148.0
          },
          {
            "t": 128.0,
            "v": -148.0
          },
          {
            "t": 130.0,
            "v": -148.0
          },
          {
            "t": 132.0,
            "v": -148.0
          },
          {
            "t": 134.0,
            "v": -148.0
          },
          {
            "t": 136.0,
            "v": -148.0
          },
          {
            "t": 138.0,
            "v": -148.0
          },
          {
            "t": 140.0,
            "v": -148.0
          },
          {
            "t": 142.0,
            "v": -148.0
          },
          {
            "t": 144.0,
            "v": -148.0
          },
          {
            "t": 146.0,
            "v": -148.0
          },
          {
            "t": 148.0,
            "v": -148.0
          },
          {
            "t": 150.0,
            "v": -148.0
          },
          {
            "t": 152.0,
            "v": -148.0
          },
          {
            "t": 154.0,
            "v": -148.0
          },
          {
            "t": 156.0,
            "v": -148.0
          },
          {
            "t": 158.0,
            "v": -148.0
          },
          {
            "t": 160.0,
            "v": -148.0
          },
          {
            "t": 162.0,
            "v": -148.0
          },
          {
            "t": 164.0,
            "v": -148.0
          },
          {
            "t": 166.0,
            "v": -148.0
          },
          {
            "t": 168.0,
            "v": -148.0
          },
          {
            "t": 170.0,
            "v": -148.0
          },
          {
            "t": 172.0,
            "v": -148.0
          },
          {
            "t": 174.0,
            "v": -148.0
          },
          {
            "t": 176.0,
            "v": -148.0
          },
          {
            "t": 178.0,
            "v": -148.0
          },
          {
            "t": 180.0,
            "v": -148.0
          },
          {
            "t": 182.0,
            "v": -148.0
          },
          {
            "t": 184.0,
            "v": -148.0
          },
          {
            "t": 186.0,
            "v": -148.0
          },
          {
            "t": 188.0,
            "v": -148.0
          },
          {
            "t": 190.0,
            "v": -148.0
          },
          {
            "t": 192.0,
            "v": -148.0
          },
          {
            "t": 194.0,
            "v": -148.0
          },
          {
            "t": 196.0,
            "v": -148.0
          },
          {
            "t": 198.0,
            "v": -148.0
          },
          {
            "t": 200.0,
            "v": -148.0
          },
          {
            "t": 202.0,
            "v": -148.0
          },
          {
            "t": 204.0,
            "v": -148.0
          },
          {
            "t": 206.0,
            "v": -148.0
          },
          {
            "t": 208.0,
            "v": -148.0
          },
          {
            "t": 210.0,
            "v": -148.0
          },
          {
            "t": 212.0,
            "v": -148.0
          },
          {
            "t": 214.0,
            "v": -148.0
          },
          {
            "t": 216.0,
            "v": -148.0
          },
          {
            "t": 218.0,
            "v": -148.0
          },
          {
            "t": 220.0,
            "v": -148.0
          },
          {
            "t": 222.0,
            "v": -148.0
          },
          {
            "t": 224.0,
            "v": -148.0
          },
          {
            "t": 226.0,
            "v": -148.0
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
            "v": 38693.8
          },
          {
            "t": 2.0,
            "v": -138.0
          },
          {
            "t": 4.0,
            "v": -124.0
          },
          {
            "t": 6.0,
            "v": -130.0
          },
          {
            "t": 8.0,
            "v": -134.0
          },
          {
            "t": 10.0,
            "v": -132.0
          },
          {
            "t": 12.0,
            "v": -136.0
          },
          {
            "t": 14.0,
            "v": -126.0
          },
          {
            "t": 16.0,
            "v": -134.0
          },
          {
            "t": 18.0,
            "v": -132.0
          },
          {
            "t": 20.0,
            "v": -132.0
          },
          {
            "t": 22.0,
            "v": -132.0
          },
          {
            "t": 24.0,
            "v": -126.0
          },
          {
            "t": 26.0,
            "v": -134.0
          },
          {
            "t": 28.0,
            "v": -132.0
          },
          {
            "t": 30.0,
            "v": -130.0
          },
          {
            "t": 32.0,
            "v": -120.0
          },
          {
            "t": 34.0,
            "v": -122.0
          },
          {
            "t": 36.0,
            "v": -136.0
          },
          {
            "t": 38.0,
            "v": -136.0
          },
          {
            "t": 40.0,
            "v": -130.0
          },
          {
            "t": 42.0,
            "v": -134.0
          },
          {
            "t": 44.0,
            "v": -126.0
          },
          {
            "t": 46.0,
            "v": -130.0
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
            "v": -132.0
          },
          {
            "t": 54.0,
            "v": -128.1
          },
          {
            "t": 56.0,
            "v": -138.0
          },
          {
            "t": 58.0,
            "v": -134.0
          },
          {
            "t": 60.0,
            "v": -130.0
          },
          {
            "t": 62.0,
            "v": -122.0
          },
          {
            "t": 64.0,
            "v": -138.0
          },
          {
            "t": 66.0,
            "v": -130.0
          },
          {
            "t": 68.0,
            "v": -128.0
          },
          {
            "t": 70.0,
            "v": -122.0
          },
          {
            "t": 72.0,
            "v": -74.0
          },
          {
            "t": 74.0,
            "v": -126.0
          },
          {
            "t": 76.0,
            "v": -122.0
          },
          {
            "t": 78.0,
            "v": -122.0
          },
          {
            "t": 80.0,
            "v": -130.0
          },
          {
            "t": 82.0,
            "v": -124.0
          },
          {
            "t": 84.0,
            "v": -126.0
          },
          {
            "t": 86.0,
            "v": -126.0
          },
          {
            "t": 88.0,
            "v": -128.0
          },
          {
            "t": 90.0,
            "v": -120.0
          },
          {
            "t": 92.0,
            "v": -128.0
          },
          {
            "t": 94.0,
            "v": -118.0
          },
          {
            "t": 96.0,
            "v": -130.0
          },
          {
            "t": 98.0,
            "v": -126.0
          },
          {
            "t": 100.0,
            "v": -124.0
          },
          {
            "t": 102.0,
            "v": -124.0
          },
          {
            "t": 104.0,
            "v": -118.0
          },
          {
            "t": 106.0,
            "v": -124.0
          },
          {
            "t": 108.0,
            "v": -122.0
          },
          {
            "t": 110.0,
            "v": -124.0
          },
          {
            "t": 112.0,
            "v": -124.0
          },
          {
            "t": 114.0,
            "v": -126.0
          },
          {
            "t": 116.0,
            "v": -124.0
          },
          {
            "t": 118.0,
            "v": -124.0
          },
          {
            "t": 120.0,
            "v": -128.0
          },
          {
            "t": 122.0,
            "v": -124.0
          },
          {
            "t": 124.0,
            "v": -122.0
          },
          {
            "t": 126.0,
            "v": -124.0
          },
          {
            "t": 128.0,
            "v": -130.0
          },
          {
            "t": 130.0,
            "v": -122.0
          },
          {
            "t": 132.0,
            "v": -128.0
          },
          {
            "t": 134.0,
            "v": -120.0
          },
          {
            "t": 136.0,
            "v": -124.0
          },
          {
            "t": 138.0,
            "v": -124.0
          },
          {
            "t": 140.0,
            "v": -118.0
          },
          {
            "t": 142.0,
            "v": -124.1
          },
          {
            "t": 144.0,
            "v": -126.0
          },
          {
            "t": 146.0,
            "v": -126.0
          },
          {
            "t": 148.0,
            "v": -124.0
          },
          {
            "t": 150.0,
            "v": -126.0
          },
          {
            "t": 152.0,
            "v": -126.0
          },
          {
            "t": 154.0,
            "v": -124.0
          },
          {
            "t": 156.0,
            "v": -120.0
          },
          {
            "t": 158.0,
            "v": -124.0
          },
          {
            "t": 160.0,
            "v": -124.0
          },
          {
            "t": 162.0,
            "v": -128.0
          },
          {
            "t": 164.0,
            "v": -120.0
          },
          {
            "t": 166.0,
            "v": -126.0
          },
          {
            "t": 168.0,
            "v": -122.0
          },
          {
            "t": 170.0,
            "v": -128.0
          },
          {
            "t": 172.0,
            "v": -122.0
          },
          {
            "t": 174.0,
            "v": -124.0
          },
          {
            "t": 176.0,
            "v": -120.0
          },
          {
            "t": 178.0,
            "v": -122.0
          },
          {
            "t": 180.0,
            "v": -122.0
          },
          {
            "t": 182.0,
            "v": -126.0
          },
          {
            "t": 184.0,
            "v": -126.0
          },
          {
            "t": 186.0,
            "v": -122.0
          },
          {
            "t": 188.0,
            "v": -124.0
          },
          {
            "t": 190.0,
            "v": -124.0
          },
          {
            "t": 192.0,
            "v": -126.0
          },
          {
            "t": 194.0,
            "v": -130.0
          },
          {
            "t": 196.0,
            "v": -126.0
          },
          {
            "t": 198.0,
            "v": -124.0
          },
          {
            "t": 200.0,
            "v": -122.0
          },
          {
            "t": 202.0,
            "v": -122.0
          },
          {
            "t": 204.0,
            "v": -126.0
          },
          {
            "t": 206.0,
            "v": -122.0
          },
          {
            "t": 208.0,
            "v": -124.0
          },
          {
            "t": 210.0,
            "v": -124.0
          },
          {
            "t": 212.0,
            "v": -118.0
          },
          {
            "t": 214.0,
            "v": -126.0
          },
          {
            "t": 216.0,
            "v": -122.0
          },
          {
            "t": 218.0,
            "v": -126.0
          },
          {
            "t": 220.0,
            "v": -122.0
          },
          {
            "t": 222.0,
            "v": -124.0
          },
          {
            "t": 224.0,
            "v": -124.0
          },
          {
            "t": 226.0,
            "v": -126.0
          },
          {
            "t": 228.0,
            "v": -120.0
          },
          {
            "t": 230.0,
            "v": -124.0
          },
          {
            "t": 232.0,
            "v": -80.0
          }
        ],
        "cswch": [
          {
            "t": 0.0,
            "v": 1353.3
          },
          {
            "t": 2.0,
            "v": 485.5
          },
          {
            "t": 4.0,
            "v": 1408.0
          },
          {
            "t": 6.0,
            "v": 570.5
          },
          {
            "t": 8.0,
            "v": 1085.5
          },
          {
            "t": 10.0,
            "v": 1215.5
          },
          {
            "t": 12.0,
            "v": 503.5
          },
          {
            "t": 14.0,
            "v": 1295.0
          },
          {
            "t": 16.0,
            "v": 1192.0
          },
          {
            "t": 18.0,
            "v": 411.5
          },
          {
            "t": 20.0,
            "v": 1473.5
          },
          {
            "t": 22.0,
            "v": 580.5
          },
          {
            "t": 24.0,
            "v": 1130.0
          },
          {
            "t": 26.0,
            "v": 320.5
          },
          {
            "t": 28.0,
            "v": 1321.0
          },
          {
            "t": 30.0,
            "v": 760.0
          },
          {
            "t": 32.0,
            "v": 878.0
          },
          {
            "t": 34.0,
            "v": 1262.0
          },
          {
            "t": 36.0,
            "v": 333.0
          },
          {
            "t": 38.0,
            "v": 1811.0
          },
          {
            "t": 40.0,
            "v": 373.0
          },
          {
            "t": 42.0,
            "v": 1229.5
          },
          {
            "t": 44.0,
            "v": 808.0
          },
          {
            "t": 46.0,
            "v": 846.5
          },
          {
            "t": 48.0,
            "v": 583.0
          },
          {
            "t": 50.0,
            "v": 1082.0
          },
          {
            "t": 52.0,
            "v": 510.0
          },
          {
            "t": 54.0,
            "v": 981.0
          },
          {
            "t": 56.0,
            "v": 1232.4
          },
          {
            "t": 58.0,
            "v": 1179.0
          },
          {
            "t": 60.0,
            "v": 775.0
          },
          {
            "t": 62.0,
            "v": 1270.5
          },
          {
            "t": 64.0,
            "v": 1245.5
          },
          {
            "t": 66.0,
            "v": 1167.5
          },
          {
            "t": 68.0,
            "v": 820.5
          },
          {
            "t": 70.0,
            "v": 1279.0
          },
          {
            "t": 72.0,
            "v": 1137.5
          },
          {
            "t": 74.0,
            "v": 350.0
          },
          {
            "t": 76.0,
            "v": 1025.5
          },
          {
            "t": 78.0,
            "v": 725.5
          },
          {
            "t": 80.0,
            "v": 755.0
          },
          {
            "t": 82.0,
            "v": 996.0
          },
          {
            "t": 84.0,
            "v": 494.5
          },
          {
            "t": 86.0,
            "v": 1273.0
          },
          {
            "t": 88.0,
            "v": 249.0
          },
          {
            "t": 90.0,
            "v": 1396.5
          },
          {
            "t": 92.0,
            "v": 211.5
          },
          {
            "t": 94.0,
            "v": 1311.0
          },
          {
            "t": 96.0,
            "v": 436.5
          },
          {
            "t": 98.0,
            "v": 1428.5
          },
          {
            "t": 100.0,
            "v": 616.5
          },
          {
            "t": 102.0,
            "v": 780.5
          },
          {
            "t": 104.0,
            "v": 885.0
          },
          {
            "t": 106.0,
            "v": 459.0
          },
          {
            "t": 108.0,
            "v": 1253.5
          },
          {
            "t": 110.0,
            "v": 166.0
          },
          {
            "t": 112.0,
            "v": 1358.0
          },
          {
            "t": 114.0,
            "v": 352.0
          },
          {
            "t": 116.0,
            "v": 1164.5
          },
          {
            "t": 118.0,
            "v": 607.5
          },
          {
            "t": 120.0,
            "v": 960.5
          },
          {
            "t": 122.0,
            "v": 797.0
          },
          {
            "t": 124.0,
            "v": 780.5
          },
          {
            "t": 126.0,
            "v": 1026.5
          },
          {
            "t": 128.0,
            "v": 532.0
          },
          {
            "t": 130.0,
            "v": 1235.0
          },
          {
            "t": 132.0,
            "v": 228.5
          },
          {
            "t": 134.0,
            "v": 1233.0
          },
          {
            "t": 136.0,
            "v": 276.0
          },
          {
            "t": 138.0,
            "v": 1414.5
          },
          {
            "t": 140.0,
            "v": 192.5
          },
          {
            "t": 142.0,
            "v": 1494.5
          },
          {
            "t": 144.0,
            "v": 349.0
          },
          {
            "t": 146.0,
            "v": 1260.5
          },
          {
            "t": 148.0,
            "v": 697.0
          },
          {
            "t": 150.0,
            "v": 976.0
          },
          {
            "t": 152.0,
            "v": 932.0
          },
          {
            "t": 154.0,
            "v": 687.0
          },
          {
            "t": 156.0,
            "v": 1245.5
          },
          {
            "t": 158.0,
            "v": 685.5
          },
          {
            "t": 160.0,
            "v": 1042.0
          },
          {
            "t": 162.0,
            "v": 595.5
          },
          {
            "t": 164.0,
            "v": 1283.0
          },
          {
            "t": 166.0,
            "v": 330.5
          },
          {
            "t": 168.0,
            "v": 1458.5
          },
          {
            "t": 170.0,
            "v": 270.5
          },
          {
            "t": 172.0,
            "v": 1318.0
          },
          {
            "t": 174.0,
            "v": 586.6
          },
          {
            "t": 176.0,
            "v": 1009.0
          },
          {
            "t": 178.0,
            "v": 917.0
          },
          {
            "t": 180.0,
            "v": 733.0
          },
          {
            "t": 182.0,
            "v": 1123.0
          },
          {
            "t": 184.0,
            "v": 476.5
          },
          {
            "t": 186.0,
            "v": 869.0
          },
          {
            "t": 188.0,
            "v": 771.5
          },
          {
            "t": 190.0,
            "v": 1135.0
          },
          {
            "t": 192.0,
            "v": 488.0
          },
          {
            "t": 194.0,
            "v": 1419.0
          },
          {
            "t": 196.0,
            "v": 150.5
          },
          {
            "t": 198.0,
            "v": 1616.0
          },
          {
            "t": 200.0,
            "v": 371.5
          },
          {
            "t": 202.0,
            "v": 1187.0
          },
          {
            "t": 204.0,
            "v": 752.5
          },
          {
            "t": 206.0,
            "v": 882.0
          },
          {
            "t": 208.0,
            "v": 844.5
          },
          {
            "t": 210.0,
            "v": 822.0
          },
          {
            "t": 212.0,
            "v": 537.5
          },
          {
            "t": 214.0,
            "v": 1049.0
          },
          {
            "t": 216.0,
            "v": 845.0
          },
          {
            "t": 218.0,
            "v": 1096.5
          },
          {
            "t": 220.0,
            "v": 1209.0
          },
          {
            "t": 222.0,
            "v": 409.5
          },
          {
            "t": 224.0,
            "v": 1439.5
          },
          {
            "t": 226.0,
            "v": 186.0
          },
          {
            "t": 228.0,
            "v": 1416.0
          },
          {
            "t": 230.0,
            "v": 549.0
          },
          {
            "t": 232.0,
            "v": 1081.5
          }
        ],
        "nvcswch": [
          {
            "t": 0.0,
            "v": 23.4
          },
          {
            "t": 2.0,
            "v": 64.0
          },
          {
            "t": 4.0,
            "v": 254.5
          },
          {
            "t": 6.0,
            "v": 122.0
          },
          {
            "t": 8.0,
            "v": 177.5
          },
          {
            "t": 10.0,
            "v": 204.0
          },
          {
            "t": 12.0,
            "v": 123.5
          },
          {
            "t": 14.0,
            "v": 190.0
          },
          {
            "t": 16.0,
            "v": 194.5
          },
          {
            "t": 18.0,
            "v": 75.0
          },
          {
            "t": 20.0,
            "v": 251.5
          },
          {
            "t": 22.0,
            "v": 31.0
          },
          {
            "t": 24.0,
            "v": 16.0
          },
          {
            "t": 26.0,
            "v": 23.0
          },
          {
            "t": 28.0,
            "v": 218.0
          },
          {
            "t": 30.0,
            "v": 120.5
          },
          {
            "t": 32.0,
            "v": 172.0
          },
          {
            "t": 34.0,
            "v": 221.0
          },
          {
            "t": 36.0,
            "v": 69.5
          },
          {
            "t": 38.0,
            "v": 392.1
          },
          {
            "t": 40.0,
            "v": 88.0
          },
          {
            "t": 42.0,
            "v": 221.0
          },
          {
            "t": 44.0,
            "v": 126.5
          },
          {
            "t": 46.0,
            "v": 39.0
          },
          {
            "t": 48.0,
            "v": 4.5
          },
          {
            "t": 50.0,
            "v": 25.0
          },
          {
            "t": 52.0,
            "v": 52.5
          },
          {
            "t": 54.0,
            "v": 149.0
          },
          {
            "t": 56.0,
            "v": 112.5
          },
          {
            "t": 58.0,
            "v": 99.5
          },
          {
            "t": 60.0,
            "v": 33.5
          },
          {
            "t": 62.0,
            "v": 100.0
          },
          {
            "t": 64.0,
            "v": 108.5
          },
          {
            "t": 66.0,
            "v": 114.0
          },
          {
            "t": 68.0,
            "v": 58.7
          },
          {
            "t": 70.0,
            "v": 107.0
          },
          {
            "t": 72.0,
            "v": 24.0
          },
          {
            "t": 74.0,
            "v": 2.5
          },
          {
            "t": 76.0,
            "v": 14.0
          },
          {
            "t": 78.0,
            "v": 6.5
          },
          {
            "t": 80.0,
            "v": 11.5
          },
          {
            "t": 82.0,
            "v": 22.0
          },
          {
            "t": 84.0,
            "v": 8.5
          },
          {
            "t": 86.0,
            "v": 32.0
          },
          {
            "t": 88.0,
            "v": 5.5
          },
          {
            "t": 90.0,
            "v": 24.5
          },
          {
            "t": 92.0,
            "v": 8.0
          },
          {
            "t": 94.0,
            "v": 22.0
          },
          {
            "t": 96.0,
            "v": 13.5
          },
          {
            "t": 98.0,
            "v": 138.5
          },
          {
            "t": 100.0,
            "v": 4.0
          },
          {
            "t": 102.0,
            "v": 7.0
          },
          {
            "t": 104.0,
            "v": 11.5
          },
          {
            "t": 106.0,
            "v": 8.5
          },
          {
            "t": 108.0,
            "v": 16.0
          },
          {
            "t": 110.0,
            "v": 2.0
          },
          {
            "t": 112.0,
            "v": 28.0
          },
          {
            "t": 114.0,
            "v": 9.5
          },
          {
            "t": 116.0,
            "v": 20.5
          },
          {
            "t": 118.0,
            "v": 10.5
          },
          {
            "t": 120.0,
            "v": 18.0
          },
          {
            "t": 122.0,
            "v": 18.0
          },
          {
            "t": 124.0,
            "v": 18.5
          },
          {
            "t": 126.0,
            "v": 20.5
          },
          {
            "t": 128.0,
            "v": 9.5
          },
          {
            "t": 130.0,
            "v": 22.5
          },
          {
            "t": 132.0,
            "v": 5.0
          },
          {
            "t": 134.0,
            "v": 15.0
          },
          {
            "t": 136.0,
            "v": 6.0
          },
          {
            "t": 138.0,
            "v": 144.0
          },
          {
            "t": 140.0,
            "v": 31.0
          },
          {
            "t": 142.0,
            "v": 180.5
          },
          {
            "t": 144.0,
            "v": 58.5
          },
          {
            "t": 146.0,
            "v": 183.0
          },
          {
            "t": 148.0,
            "v": 75.5
          },
          {
            "t": 150.0,
            "v": 144.5
          },
          {
            "t": 152.0,
            "v": 109.0
          },
          {
            "t": 154.0,
            "v": 108.0
          },
          {
            "t": 156.0,
            "v": 144.0
          },
          {
            "t": 158.0,
            "v": 126.0
          },
          {
            "t": 160.0,
            "v": 19.5
          },
          {
            "t": 162.0,
            "v": 80.0
          },
          {
            "t": 164.0,
            "v": 150.5
          },
          {
            "t": 166.0,
            "v": 44.0
          },
          {
            "t": 168.0,
            "v": 177.5
          },
          {
            "t": 170.0,
            "v": 39.0
          },
          {
            "t": 172.0,
            "v": 161.5
          },
          {
            "t": 174.0,
            "v": 68.7
          },
          {
            "t": 176.0,
            "v": 129.0
          },
          {
            "t": 178.0,
            "v": 128.0
          },
          {
            "t": 180.0,
            "v": 101.5
          },
          {
            "t": 182.0,
            "v": 87.5
          },
          {
            "t": 184.0,
            "v": 8.0
          },
          {
            "t": 186.0,
            "v": 12.0
          },
          {
            "t": 188.0,
            "v": 83.5
          },
          {
            "t": 190.0,
            "v": 145.5
          },
          {
            "t": 192.0,
            "v": 64.5
          },
          {
            "t": 194.0,
            "v": 149.0
          },
          {
            "t": 196.0,
            "v": 16.0
          },
          {
            "t": 198.0,
            "v": 170.5
          },
          {
            "t": 200.0,
            "v": 41.0
          },
          {
            "t": 202.0,
            "v": 126.0
          },
          {
            "t": 204.0,
            "v": 64.5
          },
          {
            "t": 206.0,
            "v": 103.5
          },
          {
            "t": 208.0,
            "v": 32.5
          },
          {
            "t": 210.0,
            "v": 6.5
          },
          {
            "t": 212.0,
            "v": 16.5
          },
          {
            "t": 214.0,
            "v": 117.5
          },
          {
            "t": 216.0,
            "v": 94.0
          },
          {
            "t": 218.0,
            "v": 190.5
          },
          {
            "t": 220.0,
            "v": 129.0
          },
          {
            "t": 222.0,
            "v": 73.0
          },
          {
            "t": 224.0,
            "v": 159.0
          },
          {
            "t": 226.0,
            "v": 23.5
          },
          {
            "t": 228.0,
            "v": 160.0
          },
          {
            "t": 230.0,
            "v": 58.5
          },
          {
            "t": 232.0,
            "v": 67.0
          }
        ]
      }
    },
    "system": {
      "cpu": [
        {
          "t": 0.0,
          "v": 38.1
        },
        {
          "t": 2.0,
          "v": 64.1
        },
        {
          "t": 4.0,
          "v": 85.6
        },
        {
          "t": 6.0,
          "v": 70.6
        },
        {
          "t": 8.0,
          "v": 78.9
        },
        {
          "t": 10.0,
          "v": 81.0
        },
        {
          "t": 12.0,
          "v": 85.3
        },
        {
          "t": 14.0,
          "v": 94.2
        },
        {
          "t": 16.0,
          "v": 95.3
        },
        {
          "t": 18.0,
          "v": 75.1
        },
        {
          "t": 20.0,
          "v": 86.9
        },
        {
          "t": 22.0,
          "v": 23.9
        },
        {
          "t": 24.0,
          "v": 21.6
        },
        {
          "t": 26.0,
          "v": 25.3
        },
        {
          "t": 28.0,
          "v": 80.9
        },
        {
          "t": 30.0,
          "v": 71.0
        },
        {
          "t": 32.0,
          "v": 71.2
        },
        {
          "t": 34.0,
          "v": 80.4
        },
        {
          "t": 36.0,
          "v": 60.0
        },
        {
          "t": 38.0,
          "v": 83.1
        },
        {
          "t": 40.0,
          "v": 62.9
        },
        {
          "t": 42.0,
          "v": 77.8
        },
        {
          "t": 44.0,
          "v": 72.6
        },
        {
          "t": 46.0,
          "v": 45.1
        },
        {
          "t": 48.0,
          "v": 11.1
        },
        {
          "t": 50.0,
          "v": 19.2
        },
        {
          "t": 52.0,
          "v": 87.0
        },
        {
          "t": 54.0,
          "v": 97.3
        },
        {
          "t": 56.0,
          "v": 98.5
        },
        {
          "t": 58.0,
          "v": 99.4
        },
        {
          "t": 60.0,
          "v": 98.0
        },
        {
          "t": 62.0,
          "v": 99.1
        },
        {
          "t": 64.0,
          "v": 99.1
        },
        {
          "t": 66.0,
          "v": 99.0
        },
        {
          "t": 68.0,
          "v": 98.0
        },
        {
          "t": 70.0,
          "v": 99.1
        },
        {
          "t": 72.0,
          "v": 66.4
        },
        {
          "t": 74.0,
          "v": 8.6
        },
        {
          "t": 76.0,
          "v": 20.3
        },
        {
          "t": 78.0,
          "v": 13.9
        },
        {
          "t": 80.0,
          "v": 18.0
        },
        {
          "t": 82.0,
          "v": 22.5
        },
        {
          "t": 84.0,
          "v": 13.2
        },
        {
          "t": 86.0,
          "v": 28.3
        },
        {
          "t": 88.0,
          "v": 10.1
        },
        {
          "t": 90.0,
          "v": 31.6
        },
        {
          "t": 92.0,
          "v": 7.0
        },
        {
          "t": 94.0,
          "v": 30.0
        },
        {
          "t": 96.0,
          "v": 12.4
        },
        {
          "t": 98.0,
          "v": 27.7
        },
        {
          "t": 100.0,
          "v": 12.5
        },
        {
          "t": 102.0,
          "v": 15.7
        },
        {
          "t": 104.0,
          "v": 15.4
        },
        {
          "t": 106.0,
          "v": 8.0
        },
        {
          "t": 108.0,
          "v": 24.5
        },
        {
          "t": 110.0,
          "v": 4.9
        },
        {
          "t": 112.0,
          "v": 28.7
        },
        {
          "t": 114.0,
          "v": 9.9
        },
        {
          "t": 116.0,
          "v": 27.3
        },
        {
          "t": 118.0,
          "v": 18.8
        },
        {
          "t": 120.0,
          "v": 24.7
        },
        {
          "t": 122.0,
          "v": 20.5
        },
        {
          "t": 124.0,
          "v": 20.7
        },
        {
          "t": 126.0,
          "v": 23.7
        },
        {
          "t": 128.0,
          "v": 13.9
        },
        {
          "t": 130.0,
          "v": 25.6
        },
        {
          "t": 132.0,
          "v": 4.5
        },
        {
          "t": 134.0,
          "v": 22.8
        },
        {
          "t": 136.0,
          "v": 3.9
        },
        {
          "t": 138.0,
          "v": 68.7
        },
        {
          "t": 140.0,
          "v": 58.5
        },
        {
          "t": 142.0,
          "v": 79.8
        },
        {
          "t": 144.0,
          "v": 61.4
        },
        {
          "t": 146.0,
          "v": 75.6
        },
        {
          "t": 148.0,
          "v": 63.9
        },
        {
          "t": 150.0,
          "v": 71.5
        },
        {
          "t": 152.0,
          "v": 71.1
        },
        {
          "t": 154.0,
          "v": 66.5
        },
        {
          "t": 156.0,
          "v": 76.8
        },
        {
          "t": 158.0,
          "v": 16.6
        },
        {
          "t": 160.0,
          "v": 19.7
        },
        {
          "t": 162.0,
          "v": 23.5
        },
        {
          "t": 164.0,
          "v": 76.8
        },
        {
          "t": 166.0,
          "v": 61.6
        },
        {
          "t": 168.0,
          "v": 79.4
        },
        {
          "t": 170.0,
          "v": 60.3
        },
        {
          "t": 172.0,
          "v": 76.3
        },
        {
          "t": 174.0,
          "v": 64.4
        },
        {
          "t": 176.0,
          "v": 71.7
        },
        {
          "t": 178.0,
          "v": 68.0
        },
        {
          "t": 180.0,
          "v": 68.1
        },
        {
          "t": 182.0,
          "v": 56.6
        },
        {
          "t": 184.0,
          "v": 7.2
        },
        {
          "t": 186.0,
          "v": 16.0
        },
        {
          "t": 188.0,
          "v": 56.8
        },
        {
          "t": 190.0,
          "v": 73.9
        },
        {
          "t": 192.0,
          "v": 66.7
        },
        {
          "t": 194.0,
          "v": 79.6
        },
        {
          "t": 196.0,
          "v": 60.8
        },
        {
          "t": 198.0,
          "v": 80.2
        },
        {
          "t": 200.0,
          "v": 65.1
        },
        {
          "t": 202.0,
          "v": 76.1
        },
        {
          "t": 204.0,
          "v": 67.5
        },
        {
          "t": 206.0,
          "v": 72.3
        },
        {
          "t": 208.0,
          "v": 28.3
        },
        {
          "t": 210.0,
          "v": 13.9
        },
        {
          "t": 212.0,
          "v": 24.2
        },
        {
          "t": 214.0,
          "v": 73.2
        },
        {
          "t": 216.0,
          "v": 71.6
        },
        {
          "t": 218.0,
          "v": 67.7
        },
        {
          "t": 220.0,
          "v": 75.6
        },
        {
          "t": 222.0,
          "v": 65.2
        },
        {
          "t": 224.0,
          "v": 79.6
        },
        {
          "t": 226.0,
          "v": 61.3
        },
        {
          "t": 228.0,
          "v": 78.4
        },
        {
          "t": 230.0,
          "v": 66.8
        },
        {
          "t": 232.0,
          "v": 63.6
        }
      ],
      "cpu_usr": [
        {
          "t": 0.0,
          "v": 16.5
        },
        {
          "t": 2.0,
          "v": 37.4
        },
        {
          "t": 4.0,
          "v": 42.4
        },
        {
          "t": 6.0,
          "v": 38.3
        },
        {
          "t": 8.0,
          "v": 39.3
        },
        {
          "t": 10.0,
          "v": 40.7
        },
        {
          "t": 12.0,
          "v": 58.6
        },
        {
          "t": 14.0,
          "v": 56.1
        },
        {
          "t": 16.0,
          "v": 58.0
        },
        {
          "t": 18.0,
          "v": 45.9
        },
        {
          "t": 20.0,
          "v": 44.1
        },
        {
          "t": 22.0,
          "v": 13.2
        },
        {
          "t": 24.0,
          "v": 5.5
        },
        {
          "t": 26.0,
          "v": 13.1
        },
        {
          "t": 28.0,
          "v": 42.1
        },
        {
          "t": 30.0,
          "v": 38.6
        },
        {
          "t": 32.0,
          "v": 36.9
        },
        {
          "t": 34.0,
          "v": 42.0
        },
        {
          "t": 36.0,
          "v": 33.7
        },
        {
          "t": 38.0,
          "v": 41.0
        },
        {
          "t": 40.0,
          "v": 35.2
        },
        {
          "t": 42.0,
          "v": 38.6
        },
        {
          "t": 44.0,
          "v": 37.6
        },
        {
          "t": 46.0,
          "v": 19.7
        },
        {
          "t": 48.0,
          "v": 2.5
        },
        {
          "t": 50.0,
          "v": 3.7
        },
        {
          "t": 52.0,
          "v": 58.8
        },
        {
          "t": 54.0,
          "v": 64.7
        },
        {
          "t": 56.0,
          "v": 63.6
        },
        {
          "t": 58.0,
          "v": 63.2
        },
        {
          "t": 60.0,
          "v": 66.7
        },
        {
          "t": 62.0,
          "v": 61.8
        },
        {
          "t": 64.0,
          "v": 62.2
        },
        {
          "t": 66.0,
          "v": 59.9
        },
        {
          "t": 68.0,
          "v": 64.6
        },
        {
          "t": 70.0,
          "v": 62.1
        },
        {
          "t": 72.0,
          "v": 44.6
        },
        {
          "t": 74.0,
          "v": 3.9
        },
        {
          "t": 76.0,
          "v": 4.8
        },
        {
          "t": 78.0,
          "v": 3.6
        },
        {
          "t": 80.0,
          "v": 5.1
        },
        {
          "t": 82.0,
          "v": 6.0
        },
        {
          "t": 84.0,
          "v": 4.5
        },
        {
          "t": 86.0,
          "v": 7.8
        },
        {
          "t": 88.0,
          "v": 4.7
        },
        {
          "t": 90.0,
          "v": 8.4
        },
        {
          "t": 92.0,
          "v": 3.0
        },
        {
          "t": 94.0,
          "v": 7.9
        },
        {
          "t": 96.0,
          "v": 4.6
        },
        {
          "t": 98.0,
          "v": 9.0
        },
        {
          "t": 100.0,
          "v": 3.2
        },
        {
          "t": 102.0,
          "v": 4.0
        },
        {
          "t": 104.0,
          "v": 2.9
        },
        {
          "t": 106.0,
          "v": 1.8
        },
        {
          "t": 108.0,
          "v": 5.3
        },
        {
          "t": 110.0,
          "v": 2.2
        },
        {
          "t": 112.0,
          "v": 7.3
        },
        {
          "t": 114.0,
          "v": 4.2
        },
        {
          "t": 116.0,
          "v": 7.2
        },
        {
          "t": 118.0,
          "v": 8.3
        },
        {
          "t": 120.0,
          "v": 7.9
        },
        {
          "t": 122.0,
          "v": 6.9
        },
        {
          "t": 124.0,
          "v": 7.0
        },
        {
          "t": 126.0,
          "v": 6.7
        },
        {
          "t": 128.0,
          "v": 4.7
        },
        {
          "t": 130.0,
          "v": 6.5
        },
        {
          "t": 132.0,
          "v": 1.0
        },
        {
          "t": 134.0,
          "v": 4.3
        },
        {
          "t": 136.0,
          "v": 0.9
        },
        {
          "t": 138.0,
          "v": 32.0
        },
        {
          "t": 140.0,
          "v": 35.0
        },
        {
          "t": 142.0,
          "v": 37.4
        },
        {
          "t": 144.0,
          "v": 35.0
        },
        {
          "t": 146.0,
          "v": 37.3
        },
        {
          "t": 148.0,
          "v": 31.2
        },
        {
          "t": 150.0,
          "v": 36.5
        },
        {
          "t": 152.0,
          "v": 37.2
        },
        {
          "t": 154.0,
          "v": 34.6
        },
        {
          "t": 156.0,
          "v": 38.4
        },
        {
          "t": 158.0,
          "v": 7.2
        },
        {
          "t": 160.0,
          "v": 3.8
        },
        {
          "t": 162.0,
          "v": 9.0
        },
        {
          "t": 164.0,
          "v": 34.3
        },
        {
          "t": 166.0,
          "v": 34.1
        },
        {
          "t": 168.0,
          "v": 35.5
        },
        {
          "t": 170.0,
          "v": 33.3
        },
        {
          "t": 172.0,
          "v": 34.5
        },
        {
          "t": 174.0,
          "v": 32.1
        },
        {
          "t": 176.0,
          "v": 33.8
        },
        {
          "t": 178.0,
          "v": 30.2
        },
        {
          "t": 180.0,
          "v": 33.3
        },
        {
          "t": 182.0,
          "v": 24.5
        },
        {
          "t": 184.0,
          "v": 1.3
        },
        {
          "t": 186.0,
          "v": 3.3
        },
        {
          "t": 188.0,
          "v": 27.9
        },
        {
          "t": 190.0,
          "v": 32.5
        },
        {
          "t": 192.0,
          "v": 35.3
        },
        {
          "t": 194.0,
          "v": 36.7
        },
        {
          "t": 196.0,
          "v": 33.9
        },
        {
          "t": 198.0,
          "v": 36.3
        },
        {
          "t": 200.0,
          "v": 35.2
        },
        {
          "t": 202.0,
          "v": 36.9
        },
        {
          "t": 204.0,
          "v": 31.5
        },
        {
          "t": 206.0,
          "v": 36.7
        },
        {
          "t": 208.0,
          "v": 11.8
        },
        {
          "t": 210.0,
          "v": 2.6
        },
        {
          "t": 212.0,
          "v": 9.6
        },
        {
          "t": 214.0,
          "v": 36.6
        },
        {
          "t": 216.0,
          "v": 36.9
        },
        {
          "t": 218.0,
          "v": 32.7
        },
        {
          "t": 220.0,
          "v": 36.6
        },
        {
          "t": 222.0,
          "v": 38.2
        },
        {
          "t": 224.0,
          "v": 38.3
        },
        {
          "t": 226.0,
          "v": 36.1
        },
        {
          "t": 228.0,
          "v": 39.4
        },
        {
          "t": 230.0,
          "v": 37.3
        },
        {
          "t": 232.0,
          "v": 31.9
        }
      ],
      "cpu_sys": [
        {
          "t": 0.0,
          "v": 21.3
        },
        {
          "t": 2.0,
          "v": 21.0
        },
        {
          "t": 4.0,
          "v": 37.6
        },
        {
          "t": 6.0,
          "v": 25.8
        },
        {
          "t": 8.0,
          "v": 33.0
        },
        {
          "t": 10.0,
          "v": 34.2
        },
        {
          "t": 12.0,
          "v": 21.1
        },
        {
          "t": 14.0,
          "v": 32.0
        },
        {
          "t": 16.0,
          "v": 30.4
        },
        {
          "t": 18.0,
          "v": 22.2
        },
        {
          "t": 20.0,
          "v": 36.8
        },
        {
          "t": 22.0,
          "v": 10.1
        },
        {
          "t": 24.0,
          "v": 16.1
        },
        {
          "t": 26.0,
          "v": 9.5
        },
        {
          "t": 28.0,
          "v": 33.9
        },
        {
          "t": 30.0,
          "v": 27.5
        },
        {
          "t": 32.0,
          "v": 28.9
        },
        {
          "t": 34.0,
          "v": 33.6
        },
        {
          "t": 36.0,
          "v": 20.4
        },
        {
          "t": 38.0,
          "v": 37.4
        },
        {
          "t": 40.0,
          "v": 21.1
        },
        {
          "t": 42.0,
          "v": 34.4
        },
        {
          "t": 44.0,
          "v": 29.8
        },
        {
          "t": 46.0,
          "v": 21.7
        },
        {
          "t": 48.0,
          "v": 8.6
        },
        {
          "t": 50.0,
          "v": 15.6
        },
        {
          "t": 52.0,
          "v": 22.6
        },
        {
          "t": 54.0,
          "v": 26.7
        },
        {
          "t": 56.0,
          "v": 28.4
        },
        {
          "t": 58.0,
          "v": 28.9
        },
        {
          "t": 60.0,
          "v": 22.5
        },
        {
          "t": 62.0,
          "v": 29.5
        },
        {
          "t": 64.0,
          "v": 29.2
        },
        {
          "t": 66.0,
          "v": 30.5
        },
        {
          "t": 68.0,
          "v": 24.6
        },
        {
          "t": 70.0,
          "v": 29.5
        },
        {
          "t": 72.0,
          "v": 21.0
        },
        {
          "t": 74.0,
          "v": 4.5
        },
        {
          "t": 76.0,
          "v": 15.2
        },
        {
          "t": 78.0,
          "v": 10.0
        },
        {
          "t": 80.0,
          "v": 12.4
        },
        {
          "t": 82.0,
          "v": 16.2
        },
        {
          "t": 84.0,
          "v": 8.2
        },
        {
          "t": 86.0,
          "v": 20.0
        },
        {
          "t": 88.0,
          "v": 5.0
        },
        {
          "t": 90.0,
          "v": 22.7
        },
        {
          "t": 92.0,
          "v": 3.5
        },
        {
          "t": 94.0,
          "v": 21.4
        },
        {
          "t": 96.0,
          "v": 7.2
        },
        {
          "t": 98.0,
          "v": 18.0
        },
        {
          "t": 100.0,
          "v": 9.2
        },
        {
          "t": 102.0,
          "v": 11.7
        },
        {
          "t": 104.0,
          "v": 12.5
        },
        {
          "t": 106.0,
          "v": 6.3
        },
        {
          "t": 108.0,
          "v": 18.7
        },
        {
          "t": 110.0,
          "v": 2.7
        },
        {
          "t": 112.0,
          "v": 20.8
        },
        {
          "t": 114.0,
          "v": 5.2
        },
        {
          "t": 116.0,
          "v": 19.8
        },
        {
          "t": 118.0,
          "v": 10.0
        },
        {
          "t": 120.0,
          "v": 16.3
        },
        {
          "t": 122.0,
          "v": 12.9
        },
        {
          "t": 124.0,
          "v": 13.2
        },
        {
          "t": 126.0,
          "v": 16.3
        },
        {
          "t": 128.0,
          "v": 8.8
        },
        {
          "t": 130.0,
          "v": 18.8
        },
        {
          "t": 132.0,
          "v": 3.4
        },
        {
          "t": 134.0,
          "v": 18.5
        },
        {
          "t": 136.0,
          "v": 3.0
        },
        {
          "t": 138.0,
          "v": 34.0
        },
        {
          "t": 140.0,
          "v": 18.8
        },
        {
          "t": 142.0,
          "v": 38.5
        },
        {
          "t": 144.0,
          "v": 22.2
        },
        {
          "t": 146.0,
          "v": 34.5
        },
        {
          "t": 148.0,
          "v": 28.1
        },
        {
          "t": 150.0,
          "v": 30.9
        },
        {
          "t": 152.0,
          "v": 29.7
        },
        {
          "t": 154.0,
          "v": 27.1
        },
        {
          "t": 156.0,
          "v": 34.2
        },
        {
          "t": 158.0,
          "v": 8.4
        },
        {
          "t": 160.0,
          "v": 15.7
        },
        {
          "t": 162.0,
          "v": 13.2
        },
        {
          "t": 164.0,
          "v": 37.9
        },
        {
          "t": 166.0,
          "v": 22.3
        },
        {
          "t": 168.0,
          "v": 39.8
        },
        {
          "t": 170.0,
          "v": 21.6
        },
        {
          "t": 172.0,
          "v": 37.5
        },
        {
          "t": 174.0,
          "v": 27.4
        },
        {
          "t": 176.0,
          "v": 33.5
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
          "v": 29.3
        },
        {
          "t": 184.0,
          "v": 5.9
        },
        {
          "t": 186.0,
          "v": 12.7
        },
        {
          "t": 188.0,
          "v": 25.0
        },
        {
          "t": 190.0,
          "v": 36.8
        },
        {
          "t": 192.0,
          "v": 26.2
        },
        {
          "t": 194.0,
          "v": 38.9
        },
        {
          "t": 196.0,
          "v": 21.3
        },
        {
          "t": 198.0,
          "v": 39.8
        },
        {
          "t": 200.0,
          "v": 24.9
        },
        {
          "t": 202.0,
          "v": 34.9
        },
        {
          "t": 204.0,
          "v": 31.4
        },
        {
          "t": 206.0,
          "v": 31.0
        },
        {
          "t": 208.0,
          "v": 15.4
        },
        {
          "t": 210.0,
          "v": 11.2
        },
        {
          "t": 212.0,
          "v": 13.3
        },
        {
          "t": 214.0,
          "v": 32.0
        },
        {
          "t": 216.0,
          "v": 30.5
        },
        {
          "t": 218.0,
          "v": 30.3
        },
        {
          "t": 220.0,
          "v": 34.9
        },
        {
          "t": 222.0,
          "v": 22.4
        },
        {
          "t": 224.0,
          "v": 37.1
        },
        {
          "t": 226.0,
          "v": 21.0
        },
        {
          "t": 228.0,
          "v": 35.1
        },
        {
          "t": 230.0,
          "v": 25.0
        },
        {
          "t": 232.0,
          "v": 28.5
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
          "v": 5.8
        },
        {
          "t": 4.0,
          "v": 5.7
        },
        {
          "t": 6.0,
          "v": 6.5
        },
        {
          "t": 8.0,
          "v": 6.6
        },
        {
          "t": 10.0,
          "v": 6.2
        },
        {
          "t": 12.0,
          "v": 5.7
        },
        {
          "t": 14.0,
          "v": 6.0
        },
        {
          "t": 16.0,
          "v": 6.9
        },
        {
          "t": 18.0,
          "v": 7.0
        },
        {
          "t": 20.0,
          "v": 6.1
        },
        {
          "t": 22.0,
          "v": 0.5
        },
        {
          "t": 24.0,
          "v": 0.0
        },
        {
          "t": 26.0,
          "v": 2.2
        },
        {
          "t": 28.0,
          "v": 4.9
        },
        {
          "t": 30.0,
          "v": 4.8
        },
        {
          "t": 32.0,
          "v": 5.2
        },
        {
          "t": 34.0,
          "v": 4.8
        },
        {
          "t": 36.0,
          "v": 5.9
        },
        {
          "t": 38.0,
          "v": 4.8
        },
        {
          "t": 40.0,
          "v": 6.7
        },
        {
          "t": 42.0,
          "v": 4.9
        },
        {
          "t": 44.0,
          "v": 5.2
        },
        {
          "t": 46.0,
          "v": 3.7
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
          "v": 5.7
        },
        {
          "t": 54.0,
          "v": 5.9
        },
        {
          "t": 56.0,
          "v": 6.4
        },
        {
          "t": 58.0,
          "v": 7.3
        },
        {
          "t": 60.0,
          "v": 8.8
        },
        {
          "t": 62.0,
          "v": 7.8
        },
        {
          "t": 64.0,
          "v": 7.6
        },
        {
          "t": 66.0,
          "v": 8.5
        },
        {
          "t": 68.0,
          "v": 8.8
        },
        {
          "t": 70.0,
          "v": 7.5
        },
        {
          "t": 72.0,
          "v": 0.9
        },
        {
          "t": 74.0,
          "v": 0.1
        },
        {
          "t": 76.0,
          "v": 0.3
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
          "v": 0.3
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
          "v": 0.7
        },
        {
          "t": 96.0,
          "v": 0.5
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
          "v": 0.5
        },
        {
          "t": 110.0,
          "v": 0.0
        },
        {
          "t": 112.0,
          "v": 0.6
        },
        {
          "t": 114.0,
          "v": 0.5
        },
        {
          "t": 116.0,
          "v": 0.4
        },
        {
          "t": 118.0,
          "v": 0.5
        },
        {
          "t": 120.0,
          "v": 0.5
        },
        {
          "t": 122.0,
          "v": 0.7
        },
        {
          "t": 124.0,
          "v": 0.5
        },
        {
          "t": 126.0,
          "v": 0.7
        },
        {
          "t": 128.0,
          "v": 0.5
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
          "v": 2.8
        },
        {
          "t": 140.0,
          "v": 4.7
        },
        {
          "t": 142.0,
          "v": 4.0
        },
        {
          "t": 144.0,
          "v": 4.2
        },
        {
          "t": 146.0,
          "v": 3.9
        },
        {
          "t": 148.0,
          "v": 4.6
        },
        {
          "t": 150.0,
          "v": 4.1
        },
        {
          "t": 152.0,
          "v": 4.1
        },
        {
          "t": 154.0,
          "v": 4.8
        },
        {
          "t": 156.0,
          "v": 4.2
        },
        {
          "t": 158.0,
          "v": 1.0
        },
        {
          "t": 160.0,
          "v": 0.1
        },
        {
          "t": 162.0,
          "v": 1.3
        },
        {
          "t": 164.0,
          "v": 4.6
        },
        {
          "t": 166.0,
          "v": 5.2
        },
        {
          "t": 168.0,
          "v": 4.1
        },
        {
          "t": 170.0,
          "v": 5.3
        },
        {
          "t": 172.0,
          "v": 4.3
        },
        {
          "t": 174.0,
          "v": 4.9
        },
        {
          "t": 176.0,
          "v": 4.4
        },
        {
          "t": 178.0,
          "v": 5.1
        },
        {
          "t": 180.0,
          "v": 5.4
        },
        {
          "t": 182.0,
          "v": 2.8
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
          "v": 3.9
        },
        {
          "t": 190.0,
          "v": 4.6
        },
        {
          "t": 192.0,
          "v": 5.1
        },
        {
          "t": 194.0,
          "v": 4.0
        },
        {
          "t": 196.0,
          "v": 5.6
        },
        {
          "t": 198.0,
          "v": 4.1
        },
        {
          "t": 200.0,
          "v": 5.0
        },
        {
          "t": 202.0,
          "v": 4.3
        },
        {
          "t": 204.0,
          "v": 4.5
        },
        {
          "t": 206.0,
          "v": 4.6
        },
        {
          "t": 208.0,
          "v": 1.1
        },
        {
          "t": 210.0,
          "v": 0.0
        },
        {
          "t": 212.0,
          "v": 1.4
        },
        {
          "t": 214.0,
          "v": 4.5
        },
        {
          "t": 216.0,
          "v": 4.3
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
          "v": 4.7
        },
        {
          "t": 224.0,
          "v": 4.2
        },
        {
          "t": 226.0,
          "v": 4.2
        },
        {
          "t": 228.0,
          "v": 3.9
        },
        {
          "t": 230.0,
          "v": 4.5
        },
        {
          "t": 232.0,
          "v": 3.2
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
          "v": 0.1
        },
        {
          "t": 24.0,
          "v": 0.0
        },
        {
          "t": 26.0,
          "v": 0.4
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
          "v": 0.1
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
          "v": 0.1
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
    "k6OffsetSeconds": 72,
    "machine": {
      "cpu_cores": 4,
      "mem_total_mb": 15990
    },
    "network": {
      "rx_kbs": [
        {
          "t": 0.0,
          "v": 6224.6
        },
        {
          "t": 2.0,
          "v": 5.5
        },
        {
          "t": 4.0,
          "v": 1.9
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
          "v": 0.1
        },
        {
          "t": 20.0,
          "v": 0.1
        },
        {
          "t": 22.0,
          "v": 1.9
        },
        {
          "t": 24.0,
          "v": 0.1
        },
        {
          "t": 26.0,
          "v": 2.6
        },
        {
          "t": 28.0,
          "v": 1.9
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
          "v": 2.4
        },
        {
          "t": 36.0,
          "v": 0.0
        },
        {
          "t": 38.0,
          "v": 0.2
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
          "v": 0.1
        },
        {
          "t": 46.0,
          "v": 2.1
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
          "v": 1.9
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
          "v": 2.1
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
          "v": 2.4
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
          "v": 1.9
        },
        {
          "t": 72.0,
          "v": 18.3
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
          "v": 0.1
        },
        {
          "t": 82.0,
          "v": 1.9
        },
        {
          "t": 84.0,
          "v": 0.1
        },
        {
          "t": 86.0,
          "v": 2.6
        },
        {
          "t": 88.0,
          "v": 2.1
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
          "v": 2.5
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
          "v": 1.9
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
          "v": 2.1
        },
        {
          "t": 108.0,
          "v": 0.4
        },
        {
          "t": 110.0,
          "v": 0.1
        },
        {
          "t": 112.0,
          "v": 1.9
        },
        {
          "t": 114.0,
          "v": 0.1
        },
        {
          "t": 116.0,
          "v": 0.1
        },
        {
          "t": 118.0,
          "v": 1.9
        },
        {
          "t": 120.0,
          "v": 0.1
        },
        {
          "t": 122.0,
          "v": 0.1
        },
        {
          "t": 124.0,
          "v": 2.2
        },
        {
          "t": 126.0,
          "v": 0.1
        },
        {
          "t": 128.0,
          "v": 0.1
        },
        {
          "t": 130.0,
          "v": 1.9
        },
        {
          "t": 132.0,
          "v": 0.1
        },
        {
          "t": 134.0,
          "v": 0.1
        },
        {
          "t": 136.0,
          "v": 2.0
        },
        {
          "t": 138.0,
          "v": 0.4
        },
        {
          "t": 140.0,
          "v": 0.1
        },
        {
          "t": 142.0,
          "v": 1.9
        },
        {
          "t": 144.0,
          "v": 0.1
        },
        {
          "t": 146.0,
          "v": 2.6
        },
        {
          "t": 148.0,
          "v": 1.9
        },
        {
          "t": 150.0,
          "v": 0.1
        },
        {
          "t": 152.0,
          "v": 0.1
        },
        {
          "t": 154.0,
          "v": 2.4
        },
        {
          "t": 156.0,
          "v": 0.1
        },
        {
          "t": 158.0,
          "v": 0.1
        },
        {
          "t": 160.0,
          "v": 2.0
        },
        {
          "t": 162.0,
          "v": 0.1
        },
        {
          "t": 164.0,
          "v": 0.1
        },
        {
          "t": 166.0,
          "v": 2.1
        },
        {
          "t": 168.0,
          "v": 0.4
        },
        {
          "t": 170.0,
          "v": 0.1
        },
        {
          "t": 172.0,
          "v": 1.9
        },
        {
          "t": 174.0,
          "v": 0.1
        },
        {
          "t": 176.0,
          "v": 0.1
        },
        {
          "t": 178.0,
          "v": 1.9
        },
        {
          "t": 180.0,
          "v": 0.1
        },
        {
          "t": 182.0,
          "v": 0.1
        },
        {
          "t": 184.0,
          "v": 2.2
        },
        {
          "t": 186.0,
          "v": 0.1
        },
        {
          "t": 188.0,
          "v": 0.3
        },
        {
          "t": 190.0,
          "v": 1.9
        },
        {
          "t": 192.0,
          "v": 0.1
        },
        {
          "t": 194.0,
          "v": 0.1
        },
        {
          "t": 196.0,
          "v": 2.0
        },
        {
          "t": 198.0,
          "v": 0.1
        },
        {
          "t": 200.0,
          "v": 0.1
        },
        {
          "t": 202.0,
          "v": 1.9
        },
        {
          "t": 204.0,
          "v": 0.1
        },
        {
          "t": 206.0,
          "v": 2.5
        },
        {
          "t": 208.0,
          "v": 2.1
        },
        {
          "t": 210.0,
          "v": 0.1
        },
        {
          "t": 212.0,
          "v": 0.3
        },
        {
          "t": 214.0,
          "v": 2.4
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
          "v": 1.9
        },
        {
          "t": 222.0,
          "v": 0.1
        },
        {
          "t": 224.0,
          "v": 0.1
        },
        {
          "t": 226.0,
          "v": 2.1
        },
        {
          "t": 228.0,
          "v": 0.4
        },
        {
          "t": 230.0,
          "v": 0.1
        },
        {
          "t": 232.0,
          "v": 7.7
        }
      ],
      "tx_kbs": [
        {
          "t": 0.0,
          "v": 57.7
        },
        {
          "t": 2.0,
          "v": 2.2
        },
        {
          "t": 4.0,
          "v": 4.3
        },
        {
          "t": 6.0,
          "v": 0.5
        },
        {
          "t": 8.0,
          "v": 0.5
        },
        {
          "t": 10.0,
          "v": 15.9
        },
        {
          "t": 12.0,
          "v": 0.5
        },
        {
          "t": 14.0,
          "v": 0.5
        },
        {
          "t": 16.0,
          "v": 4.4
        },
        {
          "t": 18.0,
          "v": 0.5
        },
        {
          "t": 20.0,
          "v": 0.5
        },
        {
          "t": 22.0,
          "v": 4.4
        },
        {
          "t": 24.0,
          "v": 0.4
        },
        {
          "t": 26.0,
          "v": 2.0
        },
        {
          "t": 28.0,
          "v": 4.3
        },
        {
          "t": 30.0,
          "v": 0.5
        },
        {
          "t": 32.0,
          "v": 0.6
        },
        {
          "t": 34.0,
          "v": 23.9
        },
        {
          "t": 36.0,
          "v": 0.3
        },
        {
          "t": 38.0,
          "v": 1.8
        },
        {
          "t": 40.0,
          "v": 4.3
        },
        {
          "t": 42.0,
          "v": 0.6
        },
        {
          "t": 44.0,
          "v": 0.5
        },
        {
          "t": 46.0,
          "v": 4.5
        },
        {
          "t": 48.0,
          "v": 2.1
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
          "v": 4.5
        },
        {
          "t": 60.0,
          "v": 0.8
        },
        {
          "t": 62.0,
          "v": 0.7
        },
        {
          "t": 64.0,
          "v": 23.7
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
          "v": 4.4
        },
        {
          "t": 72.0,
          "v": 38.6
        },
        {
          "t": 74.0,
          "v": 1.5
        },
        {
          "t": 76.0,
          "v": 5.4
        },
        {
          "t": 78.0,
          "v": 1.7
        },
        {
          "t": 80.0,
          "v": 1.6
        },
        {
          "t": 82.0,
          "v": 5.5
        },
        {
          "t": 84.0,
          "v": 1.6
        },
        {
          "t": 86.0,
          "v": 3.0
        },
        {
          "t": 88.0,
          "v": 6.5
        },
        {
          "t": 90.0,
          "v": 1.6
        },
        {
          "t": 92.0,
          "v": 1.6
        },
        {
          "t": 94.0,
          "v": 21.1
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
          "v": 5.5
        },
        {
          "t": 102.0,
          "v": 1.7
        },
        {
          "t": 104.0,
          "v": 1.7
        },
        {
          "t": 106.0,
          "v": 5.7
        },
        {
          "t": 108.0,
          "v": 3.3
        },
        {
          "t": 110.0,
          "v": 1.7
        },
        {
          "t": 112.0,
          "v": 5.5
        },
        {
          "t": 114.0,
          "v": 1.7
        },
        {
          "t": 116.0,
          "v": 1.7
        },
        {
          "t": 118.0,
          "v": 5.5
        },
        {
          "t": 120.0,
          "v": 1.8
        },
        {
          "t": 122.0,
          "v": 1.7
        },
        {
          "t": 124.0,
          "v": 15.1
        },
        {
          "t": 126.0,
          "v": 1.7
        },
        {
          "t": 128.0,
          "v": 1.8
        },
        {
          "t": 130.0,
          "v": 5.6
        },
        {
          "t": 132.0,
          "v": 1.8
        },
        {
          "t": 134.0,
          "v": 1.7
        },
        {
          "t": 136.0,
          "v": 5.6
        },
        {
          "t": 138.0,
          "v": 3.0
        },
        {
          "t": 140.0,
          "v": 1.8
        },
        {
          "t": 142.0,
          "v": 5.6
        },
        {
          "t": 144.0,
          "v": 1.8
        },
        {
          "t": 146.0,
          "v": 3.2
        },
        {
          "t": 148.0,
          "v": 5.6
        },
        {
          "t": 150.0,
          "v": 1.8
        },
        {
          "t": 152.0,
          "v": 1.8
        },
        {
          "t": 154.0,
          "v": 16.4
        },
        {
          "t": 156.0,
          "v": 1.8
        },
        {
          "t": 158.0,
          "v": 1.8
        },
        {
          "t": 160.0,
          "v": 5.6
        },
        {
          "t": 162.0,
          "v": 1.9
        },
        {
          "t": 164.0,
          "v": 1.8
        },
        {
          "t": 166.0,
          "v": 5.8
        },
        {
          "t": 168.0,
          "v": 3.4
        },
        {
          "t": 170.0,
          "v": 1.8
        },
        {
          "t": 172.0,
          "v": 5.6
        },
        {
          "t": 174.0,
          "v": 1.8
        },
        {
          "t": 176.0,
          "v": 1.8
        },
        {
          "t": 178.0,
          "v": 5.6
        },
        {
          "t": 180.0,
          "v": 1.8
        },
        {
          "t": 182.0,
          "v": 1.8
        },
        {
          "t": 184.0,
          "v": 15.7
        },
        {
          "t": 186.0,
          "v": 1.8
        },
        {
          "t": 188.0,
          "v": 2.9
        },
        {
          "t": 190.0,
          "v": 5.7
        },
        {
          "t": 192.0,
          "v": 1.8
        },
        {
          "t": 194.0,
          "v": 1.8
        },
        {
          "t": 196.0,
          "v": 5.7
        },
        {
          "t": 198.0,
          "v": 1.9
        },
        {
          "t": 200.0,
          "v": 1.8
        },
        {
          "t": 202.0,
          "v": 5.7
        },
        {
          "t": 204.0,
          "v": 1.8
        },
        {
          "t": 206.0,
          "v": 3.2
        },
        {
          "t": 208.0,
          "v": 5.7
        },
        {
          "t": 210.0,
          "v": 1.8
        },
        {
          "t": 212.0,
          "v": 7.0
        },
        {
          "t": 214.0,
          "v": 17.6
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
          "v": 5.7
        },
        {
          "t": 222.0,
          "v": 1.8
        },
        {
          "t": 224.0,
          "v": 1.8
        },
        {
          "t": 226.0,
          "v": 5.8
        },
        {
          "t": 228.0,
          "v": 3.4
        },
        {
          "t": 230.0,
          "v": 1.8
        },
        {
          "t": 232.0,
          "v": 15.3
        }
      ]
    },
    "paging": {
      "majflt": [
        {
          "t": 0.0,
          "v": 86713.5
        },
        {
          "t": 2.0,
          "v": 14624.5
        },
        {
          "t": 4.0,
          "v": 63852.5
        },
        {
          "t": 6.0,
          "v": 22021.5
        },
        {
          "t": 8.0,
          "v": 46210.0
        },
        {
          "t": 10.0,
          "v": 53740.0
        },
        {
          "t": 12.0,
          "v": 14582.0
        },
        {
          "t": 14.0,
          "v": 55547.0
        },
        {
          "t": 16.0,
          "v": 48936.0
        },
        {
          "t": 18.0,
          "v": 9184.0
        },
        {
          "t": 20.0,
          "v": 64704.5
        },
        {
          "t": 22.0,
          "v": 32317.0
        },
        {
          "t": 24.0,
          "v": 79510.5
        },
        {
          "t": 26.0,
          "v": 15448.5
        },
        {
          "t": 28.0,
          "v": 60414.0
        },
        {
          "t": 30.0,
          "v": 33740.5
        },
        {
          "t": 32.0,
          "v": 37971.64
        },
        {
          "t": 34.0,
          "v": 57065.0
        },
        {
          "t": 36.0,
          "v": 12310.0
        },
        {
          "t": 38.0,
          "v": 69487.0
        },
        {
          "t": 40.0,
          "v": 14347.0
        },
        {
          "t": 42.0,
          "v": 57464.0
        },
        {
          "t": 44.0,
          "v": 36785.0
        },
        {
          "t": 46.0,
          "v": 54823.5
        },
        {
          "t": 48.0,
          "v": 40745.5
        },
        {
          "t": 50.0,
          "v": 79292.5
        },
        {
          "t": 52.0,
          "v": 19095.0
        },
        {
          "t": 54.0,
          "v": 33045.5
        },
        {
          "t": 56.0,
          "v": 38624.0
        },
        {
          "t": 58.0,
          "v": 31878.5
        },
        {
          "t": 60.0,
          "v": 5750.5
        },
        {
          "t": 62.0,
          "v": 37573.0
        },
        {
          "t": 64.0,
          "v": 39032.0
        },
        {
          "t": 66.0,
          "v": 41144.5
        },
        {
          "t": 68.0,
          "v": 11015.92
        },
        {
          "t": 70.0,
          "v": 41995.5
        },
        {
          "t": 72.0,
          "v": 147593.0
        },
        {
          "t": 74.0,
          "v": 21278.5
        },
        {
          "t": 76.0,
          "v": 73153.5
        },
        {
          "t": 78.0,
          "v": 44909.5
        },
        {
          "t": 80.0,
          "v": 49588.5
        },
        {
          "t": 82.0,
          "v": 64687.0
        },
        {
          "t": 84.0,
          "v": 29384.5
        },
        {
          "t": 86.0,
          "v": 82893.5
        },
        {
          "t": 88.0,
          "v": 11396.5
        },
        {
          "t": 90.0,
          "v": 92482.0
        },
        {
          "t": 92.0,
          "v": 6653.0
        },
        {
          "t": 94.0,
          "v": 88239.0
        },
        {
          "t": 96.0,
          "v": 24502.0
        },
        {
          "t": 98.0,
          "v": 70214.0
        },
        {
          "t": 100.0,
          "v": 40019.5
        },
        {
          "t": 102.0,
          "v": 55410.5
        },
        {
          "t": 104.0,
          "v": 63708.5
        },
        {
          "t": 106.0,
          "v": 31116.5
        },
        {
          "t": 108.0,
          "v": 89566.5
        },
        {
          "t": 110.0,
          "v": 6913.0
        },
        {
          "t": 112.0,
          "v": 95424.5
        },
        {
          "t": 114.0,
          "v": 18389.5
        },
        {
          "t": 116.0,
          "v": 81388.0
        },
        {
          "t": 118.0,
          "v": 35392.0
        },
        {
          "t": 120.0,
          "v": 64527.0
        },
        {
          "t": 122.0,
          "v": 50496.0
        },
        {
          "t": 124.0,
          "v": 50479.5
        },
        {
          "t": 126.0,
          "v": 66205.5
        },
        {
          "t": 128.0,
          "v": 31887.5
        },
        {
          "t": 130.0,
          "v": 83888.0
        },
        {
          "t": 132.0,
          "v": 13681.0
        },
        {
          "t": 134.0,
          "v": 95912.0
        },
        {
          "t": 136.0,
          "v": 14731.5
        },
        {
          "t": 138.0,
          "v": 82103.5
        },
        {
          "t": 140.0,
          "v": 4686.0
        },
        {
          "t": 142.0,
          "v": 78890.5
        },
        {
          "t": 144.0,
          "v": 14442.0
        },
        {
          "t": 146.0,
          "v": 63898.5
        },
        {
          "t": 148.0,
          "v": 32379.5
        },
        {
          "t": 150.0,
          "v": 49129.5
        },
        {
          "t": 152.0,
          "v": 48104.5
        },
        {
          "t": 154.0,
          "v": 32968.5
        },
        {
          "t": 156.0,
          "v": 64068.5
        },
        {
          "t": 158.0,
          "v": 19791.5
        },
        {
          "t": 160.0,
          "v": 80277.0
        },
        {
          "t": 162.0,
          "v": 30240.0
        },
        {
          "t": 164.0,
          "v": 67878.0
        },
        {
          "t": 166.0,
          "v": 11635.5
        },
        {
          "t": 168.0,
          "v": 77209.0
        },
        {
          "t": 170.0,
          "v": 10122.5
        },
        {
          "t": 172.0,
          "v": 69089.5
        },
        {
          "t": 174.0,
          "v": 29193.0
        },
        {
          "t": 176.0,
          "v": 51255.5
        },
        {
          "t": 178.0,
          "v": 46976.0
        },
        {
          "t": 180.0,
          "v": 35633.0
        },
        {
          "t": 182.0,
          "v": 61703.0
        },
        {
          "t": 184.0,
          "v": 30813.5
        },
        {
          "t": 186.0,
          "v": 66730.0
        },
        {
          "t": 188.0,
          "v": 38583.0
        },
        {
          "t": 190.0,
          "v": 59056.0
        },
        {
          "t": 192.0,
          "v": 22224.0
        },
        {
          "t": 194.0,
          "v": 77286.5
        },
        {
          "t": 196.0,
          "v": 2279.0
        },
        {
          "t": 198.0,
          "v": 80154.5
        },
        {
          "t": 200.0,
          "v": 17260.5
        },
        {
          "t": 202.0,
          "v": 59997.5
        },
        {
          "t": 204.0,
          "v": 37864.5
        },
        {
          "t": 206.0,
          "v": 43856.5
        },
        {
          "t": 208.0,
          "v": 55129.0
        },
        {
          "t": 210.0,
          "v": 59425.0
        },
        {
          "t": 212.0,
          "v": 40440.0
        },
        {
          "t": 214.0,
          "v": 52859.5
        },
        {
          "t": 216.0,
          "v": 44881.0
        },
        {
          "t": 218.0,
          "v": 37293.53
        },
        {
          "t": 220.0,
          "v": 63043.5
        },
        {
          "t": 222.0,
          "v": 17809.0
        },
        {
          "t": 224.0,
          "v": 77972.5
        },
        {
          "t": 226.0,
          "v": 5234.5
        },
        {
          "t": 228.0,
          "v": 73747.0
        },
        {
          "t": 230.0,
          "v": 25889.0
        },
        {
          "t": 232.0,
          "v": 138276.0
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
          "v": 18
        },
        {
          "t": 2,
          "v": 20
        },
        {
          "t": 4,
          "v": 139
        },
        {
          "t": 6,
          "v": 199
        },
        {
          "t": 8,
          "v": 255
        },
        {
          "t": 10,
          "v": 223
        },
        {
          "t": 12,
          "v": 219
        },
        {
          "t": 14,
          "v": 391
        },
        {
          "t": 16,
          "v": 403
        },
        {
          "t": 18,
          "v": 663
        },
        {
          "t": 20,
          "v": 371
        },
        {
          "t": 22,
          "v": 539
        },
        {
          "t": 24,
          "v": 379
        },
        {
          "t": 26,
          "v": 379
        },
        {
          "t": 28,
          "v": 579
        },
        {
          "t": 30,
          "v": 579
        },
        {
          "t": 32,
          "v": 579
        },
        {
          "t": 34,
          "v": 579
        },
        {
          "t": 36,
          "v": 579
        },
        {
          "t": 38,
          "v": 579
        },
        {
          "t": 40,
          "v": 579
        },
        {
          "t": 42,
          "v": 579
        },
        {
          "t": 44,
          "v": 579
        },
        {
          "t": 46,
          "v": 579
        },
        {
          "t": 48,
          "v": 579
        },
        {
          "t": 50,
          "v": 579
        },
        {
          "t": 52,
          "v": 979
        },
        {
          "t": 54,
          "v": 1213
        },
        {
          "t": 56,
          "v": 1539
        },
        {
          "t": 58,
          "v": 1820
        },
        {
          "t": 60,
          "v": 2042
        },
        {
          "t": 62,
          "v": 2032
        },
        {
          "t": 64,
          "v": 1990
        },
        {
          "t": 66,
          "v": 2395
        },
        {
          "t": 68,
          "v": 2609
        },
        {
          "t": 70,
          "v": 2317
        },
        {
          "t": 72,
          "v": 12
        },
        {
          "t": 74,
          "v": 25
        },
        {
          "t": 76,
          "v": 37
        },
        {
          "t": 78,
          "v": 55
        },
        {
          "t": 80,
          "v": 71
        },
        {
          "t": 82,
          "v": 93
        },
        {
          "t": 84,
          "v": 93
        },
        {
          "t": 86,
          "v": 93
        },
        {
          "t": 88,
          "v": 93
        },
        {
          "t": 90,
          "v": 93
        },
        {
          "t": 92,
          "v": 93
        },
        {
          "t": 94,
          "v": 93
        },
        {
          "t": 96,
          "v": 93
        },
        {
          "t": 98,
          "v": 93
        },
        {
          "t": 100,
          "v": 93
        },
        {
          "t": 102,
          "v": 93
        },
        {
          "t": 104,
          "v": 93
        },
        {
          "t": 106,
          "v": 93
        },
        {
          "t": 108,
          "v": 114
        },
        {
          "t": 110,
          "v": 132
        },
        {
          "t": 112,
          "v": 150
        },
        {
          "t": 114,
          "v": 154
        },
        {
          "t": 116,
          "v": 154
        },
        {
          "t": 118,
          "v": 154
        },
        {
          "t": 120,
          "v": 154
        },
        {
          "t": 122,
          "v": 154
        },
        {
          "t": 124,
          "v": 154
        },
        {
          "t": 126,
          "v": 154
        },
        {
          "t": 128,
          "v": 151
        },
        {
          "t": 130,
          "v": 150
        },
        {
          "t": 132,
          "v": 150
        },
        {
          "t": 134,
          "v": 150
        },
        {
          "t": 136,
          "v": 150
        },
        {
          "t": 138,
          "v": 464
        },
        {
          "t": 140,
          "v": 463
        },
        {
          "t": 142,
          "v": 462
        },
        {
          "t": 144,
          "v": 462
        },
        {
          "t": 146,
          "v": 462
        },
        {
          "t": 148,
          "v": 462
        },
        {
          "t": 150,
          "v": 462
        },
        {
          "t": 152,
          "v": 462
        },
        {
          "t": 154,
          "v": 462
        },
        {
          "t": 156,
          "v": 462
        },
        {
          "t": 158,
          "v": 462
        },
        {
          "t": 160,
          "v": 462
        },
        {
          "t": 162,
          "v": 441
        },
        {
          "t": 164,
          "v": 747
        },
        {
          "t": 166,
          "v": 747
        },
        {
          "t": 168,
          "v": 747
        },
        {
          "t": 170,
          "v": 747
        },
        {
          "t": 172,
          "v": 747
        },
        {
          "t": 174,
          "v": 747
        },
        {
          "t": 176,
          "v": 747
        },
        {
          "t": 178,
          "v": 747
        },
        {
          "t": 180,
          "v": 747
        },
        {
          "t": 182,
          "v": 747
        },
        {
          "t": 184,
          "v": 747
        },
        {
          "t": 186,
          "v": 747
        },
        {
          "t": 188,
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
          "v": 1245
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
          "v": 16
        },
        {
          "t": 2,
          "v": 17
        },
        {
          "t": 4,
          "v": 483
        },
        {
          "t": 6,
          "v": 1681
        },
        {
          "t": 8,
          "v": 2377
        },
        {
          "t": 10,
          "v": 3468
        },
        {
          "t": 12,
          "v": 4638
        },
        {
          "t": 14,
          "v": 5862
        },
        {
          "t": 16,
          "v": 8603
        },
        {
          "t": 18,
          "v": 11139
        },
        {
          "t": 20,
          "v": 12063
        },
        {
          "t": 22,
          "v": 13004
        },
        {
          "t": 24,
          "v": 13140
        },
        {
          "t": 26,
          "v": 13140
        },
        {
          "t": 28,
          "v": 13120
        },
        {
          "t": 30,
          "v": 13120
        },
        {
          "t": 32,
          "v": 13120
        },
        {
          "t": 34,
          "v": 13120
        },
        {
          "t": 36,
          "v": 13120
        },
        {
          "t": 38,
          "v": 13120
        },
        {
          "t": 40,
          "v": 13120
        },
        {
          "t": 42,
          "v": 13120
        },
        {
          "t": 44,
          "v": 13120
        },
        {
          "t": 46,
          "v": 13120
        },
        {
          "t": 48,
          "v": 13120
        },
        {
          "t": 50,
          "v": 13120
        },
        {
          "t": 52,
          "v": 13100
        },
        {
          "t": 54,
          "v": 13088
        },
        {
          "t": 56,
          "v": 13204
        },
        {
          "t": 58,
          "v": 13819
        },
        {
          "t": 60,
          "v": 14990
        },
        {
          "t": 62,
          "v": 16471
        },
        {
          "t": 64,
          "v": 17655
        },
        {
          "t": 66,
          "v": 19057
        },
        {
          "t": 68,
          "v": 18945
        },
        {
          "t": 70,
          "v": 20265
        },
        {
          "t": 72,
          "v": 21688
        },
        {
          "t": 74,
          "v": 21688
        },
        {
          "t": 76,
          "v": 19702
        },
        {
          "t": 78,
          "v": 19744
        },
        {
          "t": 80,
          "v": 17488
        },
        {
          "t": 82,
          "v": 17529
        },
        {
          "t": 84,
          "v": 16738
        },
        {
          "t": 86,
          "v": 16783
        },
        {
          "t": 88,
          "v": 16826
        },
        {
          "t": 90,
          "v": 16871
        },
        {
          "t": 92,
          "v": 16911
        },
        {
          "t": 94,
          "v": 16953
        },
        {
          "t": 96,
          "v": 16992
        },
        {
          "t": 98,
          "v": 16992
        },
        {
          "t": 100,
          "v": 16992
        },
        {
          "t": 102,
          "v": 16992
        },
        {
          "t": 104,
          "v": 16992
        },
        {
          "t": 106,
          "v": 16992
        },
        {
          "t": 108,
          "v": 16992
        },
        {
          "t": 110,
          "v": 17003
        },
        {
          "t": 112,
          "v": 17024
        },
        {
          "t": 114,
          "v": 17044
        },
        {
          "t": 116,
          "v": 17066
        },
        {
          "t": 118,
          "v": 16936
        },
        {
          "t": 120,
          "v": 16956
        },
        {
          "t": 122,
          "v": 14919
        },
        {
          "t": 124,
          "v": 14940
        },
        {
          "t": 126,
          "v": 11135
        },
        {
          "t": 128,
          "v": 11155
        },
        {
          "t": 130,
          "v": 5836
        },
        {
          "t": 132,
          "v": 5836
        },
        {
          "t": 134,
          "v": 635
        },
        {
          "t": 136,
          "v": 635
        },
        {
          "t": 138,
          "v": 650
        },
        {
          "t": 140,
          "v": 653
        },
        {
          "t": 142,
          "v": 570
        },
        {
          "t": 144,
          "v": 614
        },
        {
          "t": 146,
          "v": 549
        },
        {
          "t": 148,
          "v": 583
        },
        {
          "t": 150,
          "v": 508
        },
        {
          "t": 152,
          "v": 541
        },
        {
          "t": 154,
          "v": 455
        },
        {
          "t": 156,
          "v": 465
        },
        {
          "t": 158,
          "v": 480
        },
        {
          "t": 160,
          "v": 480
        },
        {
          "t": 162,
          "v": 491
        },
        {
          "t": 164,
          "v": 564
        },
        {
          "t": 166,
          "v": 576
        },
        {
          "t": 168,
          "v": 602
        },
        {
          "t": 170,
          "v": 631
        },
        {
          "t": 172,
          "v": 650
        },
        {
          "t": 174,
          "v": 651
        },
        {
          "t": 176,
          "v": 655
        },
        {
          "t": 178,
          "v": 647
        },
        {
          "t": 180,
          "v": 683
        },
        {
          "t": 182,
          "v": 691
        },
        {
          "t": 184,
          "v": 691
        },
        {
          "t": 186,
          "v": 645
        },
        {
          "t": 188,
          "v": 678
        },
        {
          "t": 191,
          "v": 752
        },
        {
          "t": 193,
          "v": 756
        },
        {
          "t": 195,
          "v": 771
        },
        {
          "t": 197,
          "v": 780
        },
        {
          "t": 199,
          "v": 751
        },
        {
          "t": 201,
          "v": 838
        },
        {
          "t": 203,
          "v": 850
        },
        {
          "t": 205,
          "v": 914
        },
        {
          "t": 207,
          "v": 863
        },
        {
          "t": 209,
          "v": 874
        },
        {
          "t": 211,
          "v": 824
        },
        {
          "t": 213,
          "v": 824
        },
        {
          "t": 215,
          "v": 781
        },
        {
          "t": 217,
          "v": 850
        },
        {
          "t": 219,
          "v": 906
        },
        {
          "t": 221,
          "v": 888
        },
        {
          "t": 223,
          "v": 904
        },
        {
          "t": 225,
          "v": 971
        },
        {
          "t": 227,
          "v": 980
        },
        {
          "t": 229,
          "v": 891
        },
        {
          "t": 231,
          "v": 895
        },
        {
          "t": 233,
          "v": 857
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
          "v": 483
        },
        {
          "t": 6,
          "v": 1681
        },
        {
          "t": 8,
          "v": 2377
        },
        {
          "t": 10,
          "v": 3468
        },
        {
          "t": 12,
          "v": 4638
        },
        {
          "t": 14,
          "v": 5862
        },
        {
          "t": 16,
          "v": 8603
        },
        {
          "t": 18,
          "v": 11139
        },
        {
          "t": 20,
          "v": 12063
        },
        {
          "t": 22,
          "v": 13004
        },
        {
          "t": 24,
          "v": 13140
        },
        {
          "t": 26,
          "v": 13140
        },
        {
          "t": 28,
          "v": 13120
        },
        {
          "t": 30,
          "v": 13120
        },
        {
          "t": 32,
          "v": 13120
        },
        {
          "t": 34,
          "v": 13120
        },
        {
          "t": 36,
          "v": 13120
        },
        {
          "t": 38,
          "v": 13120
        },
        {
          "t": 40,
          "v": 13120
        },
        {
          "t": 42,
          "v": 13120
        },
        {
          "t": 44,
          "v": 13120
        },
        {
          "t": 46,
          "v": 13120
        },
        {
          "t": 48,
          "v": 13120
        },
        {
          "t": 50,
          "v": 13120
        },
        {
          "t": 52,
          "v": 13100
        },
        {
          "t": 54,
          "v": 13088
        },
        {
          "t": 56,
          "v": 13204
        },
        {
          "t": 58,
          "v": 13819
        },
        {
          "t": 60,
          "v": 14990
        },
        {
          "t": 62,
          "v": 16471
        },
        {
          "t": 64,
          "v": 17655
        },
        {
          "t": 66,
          "v": 19057
        },
        {
          "t": 68,
          "v": 18945
        },
        {
          "t": 70,
          "v": 20265
        },
        {
          "t": 72,
          "v": 21688
        },
        {
          "t": 74,
          "v": 21688
        },
        {
          "t": 76,
          "v": 19702
        },
        {
          "t": 78,
          "v": 19744
        },
        {
          "t": 80,
          "v": 17488
        },
        {
          "t": 82,
          "v": 17529
        },
        {
          "t": 84,
          "v": 16738
        },
        {
          "t": 86,
          "v": 16783
        },
        {
          "t": 88,
          "v": 16826
        },
        {
          "t": 90,
          "v": 16871
        },
        {
          "t": 92,
          "v": 16911
        },
        {
          "t": 94,
          "v": 16953
        },
        {
          "t": 96,
          "v": 16992
        },
        {
          "t": 98,
          "v": 16992
        },
        {
          "t": 100,
          "v": 16992
        },
        {
          "t": 102,
          "v": 16992
        },
        {
          "t": 104,
          "v": 16992
        },
        {
          "t": 106,
          "v": 16992
        },
        {
          "t": 108,
          "v": 16992
        },
        {
          "t": 110,
          "v": 17003
        },
        {
          "t": 112,
          "v": 17024
        },
        {
          "t": 114,
          "v": 17044
        },
        {
          "t": 116,
          "v": 17066
        },
        {
          "t": 118,
          "v": 16936
        },
        {
          "t": 120,
          "v": 16956
        },
        {
          "t": 122,
          "v": 14919
        },
        {
          "t": 124,
          "v": 14940
        },
        {
          "t": 126,
          "v": 11135
        },
        {
          "t": 128,
          "v": 11155
        },
        {
          "t": 130,
          "v": 5836
        },
        {
          "t": 132,
          "v": 5836
        },
        {
          "t": 134,
          "v": 635
        },
        {
          "t": 136,
          "v": 635
        },
        {
          "t": 138,
          "v": 650
        },
        {
          "t": 140,
          "v": 653
        },
        {
          "t": 142,
          "v": 570
        },
        {
          "t": 144,
          "v": 614
        },
        {
          "t": 146,
          "v": 549
        },
        {
          "t": 148,
          "v": 583
        },
        {
          "t": 150,
          "v": 508
        },
        {
          "t": 152,
          "v": 541
        },
        {
          "t": 154,
          "v": 455
        },
        {
          "t": 156,
          "v": 465
        },
        {
          "t": 158,
          "v": 480
        },
        {
          "t": 160,
          "v": 480
        },
        {
          "t": 162,
          "v": 491
        },
        {
          "t": 164,
          "v": 564
        },
        {
          "t": 166,
          "v": 576
        },
        {
          "t": 168,
          "v": 602
        },
        {
          "t": 170,
          "v": 631
        },
        {
          "t": 172,
          "v": 650
        },
        {
          "t": 174,
          "v": 651
        },
        {
          "t": 176,
          "v": 655
        },
        {
          "t": 178,
          "v": 647
        },
        {
          "t": 180,
          "v": 683
        },
        {
          "t": 182,
          "v": 691
        },
        {
          "t": 184,
          "v": 691
        },
        {
          "t": 186,
          "v": 645
        },
        {
          "t": 188,
          "v": 678
        },
        {
          "t": 191,
          "v": 752
        },
        {
          "t": 193,
          "v": 756
        },
        {
          "t": 195,
          "v": 771
        },
        {
          "t": 197,
          "v": 780
        },
        {
          "t": 199,
          "v": 751
        },
        {
          "t": 201,
          "v": 838
        },
        {
          "t": 203,
          "v": 850
        },
        {
          "t": 205,
          "v": 914
        },
        {
          "t": 207,
          "v": 863
        },
        {
          "t": 209,
          "v": 874
        },
        {
          "t": 211,
          "v": 824
        },
        {
          "t": 213,
          "v": 824
        },
        {
          "t": 215,
          "v": 781
        },
        {
          "t": 217,
          "v": 850
        },
        {
          "t": 219,
          "v": 906
        },
        {
          "t": 221,
          "v": 888
        },
        {
          "t": 223,
          "v": 904
        },
        {
          "t": 225,
          "v": 971
        },
        {
          "t": 227,
          "v": 980
        },
        {
          "t": 229,
          "v": 891
        },
        {
          "t": 231,
          "v": 895
        },
        {
          "t": 233,
          "v": 857
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
          "t": 34,
          "v": 0.0
        },
        {
          "t": 36,
          "v": 0.0
        },
        {
          "t": 38,
          "v": 0.0
        },
        {
          "t": 40,
          "v": 0.0
        },
        {
          "t": 42,
          "v": 0.0
        },
        {
          "t": 44,
          "v": 0.0
        },
        {
          "t": 46,
          "v": 0.0
        },
        {
          "t": 48,
          "v": 0.0
        },
        {
          "t": 50,
          "v": 0.0
        },
        {
          "t": 52,
          "v": 0.0
        },
        {
          "t": 54,
          "v": 0.0
        },
        {
          "t": 56,
          "v": 0.0
        },
        {
          "t": 58,
          "v": 0.0
        },
        {
          "t": 60,
          "v": 0.0
        },
        {
          "t": 62,
          "v": 0.0
        },
        {
          "t": 64,
          "v": 0.0
        },
        {
          "t": 66,
          "v": 0.0
        },
        {
          "t": 68,
          "v": 0.0
        },
        {
          "t": 70,
          "v": 0.0
        },
        {
          "t": 72,
          "v": 0.0
        },
        {
          "t": 74,
          "v": 0.0
        },
        {
          "t": 76,
          "v": 0.0
        },
        {
          "t": 78,
          "v": 0.0
        },
        {
          "t": 80,
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
          "t": 147,
          "v": 0.0
        },
        {
          "t": 149,
          "v": 0.0
        },
        {
          "t": 151,
          "v": 0.0
        },
        {
          "t": 153,
          "v": 0.0
        },
        {
          "t": 155,
          "v": 0.0
        },
        {
          "t": 157,
          "v": 0.0
        },
        {
          "t": 159,
          "v": 0.0
        },
        {
          "t": 161,
          "v": 0.0
        },
        {
          "t": 163,
          "v": 0.0
        },
        {
          "t": 165,
          "v": 0.0
        },
        {
          "t": 167,
          "v": 0.0
        },
        {
          "t": 169,
          "v": 0.0
        },
        {
          "t": 171,
          "v": 0.0
        },
        {
          "t": 173,
          "v": 0.0
        },
        {
          "t": 175,
          "v": 0.0
        },
        {
          "t": 177,
          "v": 0.0
        },
        {
          "t": 179,
          "v": 0.0
        },
        {
          "t": 181,
          "v": 0.0
        },
        {
          "t": 183,
          "v": 0.0
        },
        {
          "t": 185,
          "v": 0.0
        },
        {
          "t": 187,
          "v": 0.0
        },
        {
          "t": 189,
          "v": 0.0
        },
        {
          "t": 191,
          "v": 0.0
        },
        {
          "t": 193,
          "v": 0.0
        },
        {
          "t": 195,
          "v": 0.0
        },
        {
          "t": 197,
          "v": 0.0
        },
        {
          "t": 199,
          "v": 0.0
        },
        {
          "t": 201,
          "v": 0.0
        },
        {
          "t": 203,
          "v": 0.0
        },
        {
          "t": 205,
          "v": 0.0
        },
        {
          "t": 207,
          "v": 0.0
        },
        {
          "t": 209,
          "v": 0.0
        },
        {
          "t": 211,
          "v": 0.0
        },
        {
          "t": 213,
          "v": 0.0
        },
        {
          "t": 215,
          "v": 0.0
        },
        {
          "t": 217,
          "v": 0.0
        },
        {
          "t": 219,
          "v": 0.0
        },
        {
          "t": 221,
          "v": 0.0
        },
        {
          "t": 223,
          "v": 0.0
        },
        {
          "t": 225,
          "v": 0.0
        },
        {
          "t": 227,
          "v": 0.0
        },
        {
          "t": 229,
          "v": 0.0
        },
        {
          "t": 231,
          "v": 0.0
        },
        {
          "t": 233,
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
          "t": 34,
          "v": 0.0
        },
        {
          "t": 36,
          "v": 0.0
        },
        {
          "t": 38,
          "v": 0.0
        },
        {
          "t": 40,
          "v": 0.0
        },
        {
          "t": 42,
          "v": 0.0
        },
        {
          "t": 44,
          "v": 0.0
        },
        {
          "t": 46,
          "v": 0.0
        },
        {
          "t": 48,
          "v": 0.0
        },
        {
          "t": 50,
          "v": 0.0
        },
        {
          "t": 52,
          "v": 0.0
        },
        {
          "t": 54,
          "v": 0.0
        },
        {
          "t": 56,
          "v": 0.0
        },
        {
          "t": 58,
          "v": 0.0
        },
        {
          "t": 60,
          "v": 0.0
        },
        {
          "t": 62,
          "v": 0.0
        },
        {
          "t": 64,
          "v": 0.0
        },
        {
          "t": 66,
          "v": 0.0
        },
        {
          "t": 68,
          "v": 0.0
        },
        {
          "t": 70,
          "v": 0.0
        },
        {
          "t": 72,
          "v": 0.0
        },
        {
          "t": 74,
          "v": 0.0
        },
        {
          "t": 76,
          "v": 0.0
        },
        {
          "t": 78,
          "v": 0.0
        },
        {
          "t": 80,
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
          "t": 147,
          "v": 0.0
        },
        {
          "t": 149,
          "v": 0.0
        },
        {
          "t": 151,
          "v": 0.0
        },
        {
          "t": 153,
          "v": 0.0
        },
        {
          "t": 155,
          "v": 0.0
        },
        {
          "t": 157,
          "v": 0.0
        },
        {
          "t": 159,
          "v": 0.0
        },
        {
          "t": 161,
          "v": 0.0
        },
        {
          "t": 163,
          "v": 0.0
        },
        {
          "t": 165,
          "v": 0.0
        },
        {
          "t": 167,
          "v": 0.0
        },
        {
          "t": 169,
          "v": 0.0
        },
        {
          "t": 171,
          "v": 0.0
        },
        {
          "t": 173,
          "v": 0.0
        },
        {
          "t": 175,
          "v": 0.0
        },
        {
          "t": 177,
          "v": 0.0
        },
        {
          "t": 179,
          "v": 0.0
        },
        {
          "t": 181,
          "v": 0.0
        },
        {
          "t": 183,
          "v": 0.0
        },
        {
          "t": 185,
          "v": 0.0
        },
        {
          "t": 187,
          "v": 0.0
        },
        {
          "t": 189,
          "v": 0.0
        },
        {
          "t": 191,
          "v": 0.0
        },
        {
          "t": 193,
          "v": 0.0
        },
        {
          "t": 195,
          "v": 0.0
        },
        {
          "t": 197,
          "v": 0.0
        },
        {
          "t": 199,
          "v": 0.0
        },
        {
          "t": 201,
          "v": 0.0
        },
        {
          "t": 203,
          "v": 0.0
        },
        {
          "t": 205,
          "v": 0.0
        },
        {
          "t": 207,
          "v": 0.0
        },
        {
          "t": 209,
          "v": 0.0
        },
        {
          "t": 211,
          "v": 0.0
        },
        {
          "t": 213,
          "v": 0.0
        },
        {
          "t": 215,
          "v": 0.0
        },
        {
          "t": 217,
          "v": 0.0
        },
        {
          "t": 219,
          "v": 0.0
        },
        {
          "t": 221,
          "v": 0.0
        },
        {
          "t": 223,
          "v": 0.0
        },
        {
          "t": 225,
          "v": 0.0
        },
        {
          "t": 227,
          "v": 0.0
        },
        {
          "t": 229,
          "v": 0.0
        },
        {
          "t": 231,
          "v": 0.0
        },
        {
          "t": 233,
          "v": 0.0
        }
      ],
      "io": [
        {
          "t": 0,
          "v": 0.36
        },
        {
          "t": 2,
          "v": 0.29
        },
        {
          "t": 4,
          "v": 0.24
        },
        {
          "t": 6,
          "v": 0.19
        },
        {
          "t": 8,
          "v": 0.16
        },
        {
          "t": 10,
          "v": 0.13
        },
        {
          "t": 12,
          "v": 0.1
        },
        {
          "t": 14,
          "v": 0.08
        },
        {
          "t": 16,
          "v": 0.07
        },
        {
          "t": 18,
          "v": 0.05
        },
        {
          "t": 20,
          "v": 0.04
        },
        {
          "t": 22,
          "v": 0.03
        },
        {
          "t": 24,
          "v": 0.03
        },
        {
          "t": 26,
          "v": 0.02
        },
        {
          "t": 28,
          "v": 0.02
        },
        {
          "t": 30,
          "v": 0.01
        },
        {
          "t": 32,
          "v": 0.01
        },
        {
          "t": 34,
          "v": 0.01
        },
        {
          "t": 36,
          "v": 0.0
        },
        {
          "t": 38,
          "v": 0.0
        },
        {
          "t": 40,
          "v": 0.0
        },
        {
          "t": 42,
          "v": 0.0
        },
        {
          "t": 44,
          "v": 0.0
        },
        {
          "t": 46,
          "v": 0.0
        },
        {
          "t": 48,
          "v": 0.0
        },
        {
          "t": 50,
          "v": 0.0
        },
        {
          "t": 52,
          "v": 0.0
        },
        {
          "t": 54,
          "v": 0.0
        },
        {
          "t": 56,
          "v": 0.0
        },
        {
          "t": 58,
          "v": 0.0
        },
        {
          "t": 60,
          "v": 0.0
        },
        {
          "t": 62,
          "v": 0.0
        },
        {
          "t": 64,
          "v": 0.0
        },
        {
          "t": 66,
          "v": 0.0
        },
        {
          "t": 68,
          "v": 0.0
        },
        {
          "t": 70,
          "v": 0.0
        },
        {
          "t": 72,
          "v": 0.0
        },
        {
          "t": 74,
          "v": 0.0
        },
        {
          "t": 76,
          "v": 0.0
        },
        {
          "t": 78,
          "v": 0.0
        },
        {
          "t": 80,
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
          "t": 147,
          "v": 0.0
        },
        {
          "t": 149,
          "v": 0.0
        },
        {
          "t": 151,
          "v": 0.0
        },
        {
          "t": 153,
          "v": 0.0
        },
        {
          "t": 155,
          "v": 0.0
        },
        {
          "t": 157,
          "v": 0.0
        },
        {
          "t": 159,
          "v": 0.0
        },
        {
          "t": 161,
          "v": 0.0
        },
        {
          "t": 163,
          "v": 0.0
        },
        {
          "t": 165,
          "v": 0.0
        },
        {
          "t": 167,
          "v": 0.0
        },
        {
          "t": 169,
          "v": 0.0
        },
        {
          "t": 171,
          "v": 0.0
        },
        {
          "t": 173,
          "v": 0.0
        },
        {
          "t": 175,
          "v": 0.0
        },
        {
          "t": 177,
          "v": 0.0
        },
        {
          "t": 179,
          "v": 0.0
        },
        {
          "t": 181,
          "v": 0.0
        },
        {
          "t": 183,
          "v": 0.0
        },
        {
          "t": 185,
          "v": 0.0
        },
        {
          "t": 187,
          "v": 0.0
        },
        {
          "t": 189,
          "v": 0.0
        },
        {
          "t": 191,
          "v": 0.0
        },
        {
          "t": 193,
          "v": 0.0
        },
        {
          "t": 195,
          "v": 0.0
        },
        {
          "t": 197,
          "v": 0.0
        },
        {
          "t": 199,
          "v": 0.0
        },
        {
          "t": 201,
          "v": 0.0
        },
        {
          "t": 203,
          "v": 0.0
        },
        {
          "t": 205,
          "v": 0.0
        },
        {
          "t": 207,
          "v": 0.0
        },
        {
          "t": 209,
          "v": 0.0
        },
        {
          "t": 211,
          "v": 0.0
        },
        {
          "t": 213,
          "v": 0.0
        },
        {
          "t": 215,
          "v": 0.0
        },
        {
          "t": 217,
          "v": 0.0
        },
        {
          "t": 219,
          "v": 0.0
        },
        {
          "t": 221,
          "v": 0.0
        },
        {
          "t": 223,
          "v": 0.0
        },
        {
          "t": 225,
          "v": 0.0
        },
        {
          "t": 227,
          "v": 0.0
        },
        {
          "t": 229,
          "v": 0.0
        },
        {
          "t": 231,
          "v": 0.0
        },
        {
          "t": 233,
          "v": 0.0
        }
      ]
    }
  },
  "artifacts": {
    "trace_artifact_url": "https://github.com/locustbaby/duotunnel/actions/runs/24184246999/artifacts/6347033039"
  }
};
