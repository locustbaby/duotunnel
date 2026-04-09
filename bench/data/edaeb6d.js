window.BENCH_DETAIL['edaeb6d'] = {
  "timestamp": "2026-04-09T10:04:57.452Z",
  "commit": {
    "id": "edaeb6d11c8b1477e5b36a45322d392204fdaee3",
    "message": "ci: store dial9 traces in GHA artifacts instead of gh-pages",
    "url": "https://github.com/locustbaby/duotunnel/commit/edaeb6d11c8b1477e5b36a45322d392204fdaee3"
  },
  "scenarios": [
    {
      "name": "ingress_http_get",
      "protocol": "HTTP",
      "direction": "ingress",
      "category": "basic",
      "includeInTotalRps": false,
      "p50": 0.55,
      "p95": 1.23,
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
      "p50": 0.57,
      "p95": 1.18,
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
      "p50": 0.51,
      "p95": 1.04,
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
      "p50": 0.51,
      "p95": 0.74,
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
      "p50": 1.9,
      "p95": 2.89,
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
      "p50": 1.18,
      "p95": 1.91,
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
      "p50": 1.19,
      "p95": 1.92,
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
      "p50": 0.46,
      "p95": 0.64,
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
      "p50": 0.6,
      "p95": 1.92,
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
      "p50": 1.04,
      "p95": 2.34,
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
      "p50": 2.61,
      "p95": 3.83,
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
      "p50": 1.05,
      "p95": 2.21,
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
      "p50": 44.19,
      "p95": 45.46,
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
      "p50": 1.05,
      "p95": 2.62,
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
      "p50": 0.77,
      "p95": 1.67,
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
      "p50": 0.51,
      "p95": 1.83,
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
      "p50": 0.5,
      "p95": 1.74,
      "p99": 0,
      "err": 0,
      "rps": 3000.1,
      "requests": 60002
    },
    {
      "name": "ingress_multihost",
      "protocol": "HTTP",
      "direction": "ingress",
      "category": "stress",
      "includeInTotalRps": true,
      "p50": 0.52,
      "p95": 1.91,
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
      "p50": 0.54,
      "p95": 2.04,
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
      "p50": 42.09,
      "p95": 113.99,
      "p99": 0,
      "err": 0,
      "rps": 7020.25,
      "requests": 140405
    },
    {
      "name": "egress_8000qps (8k-q4)",
      "protocol": "HTTP",
      "direction": "egress",
      "category": "stress",
      "includeInTotalRps": true,
      "p50": 33.42,
      "p95": 116.78,
      "p99": 0,
      "err": 0,
      "rps": 7034.2,
      "requests": 140684
    },
    {
      "name": "ingress_multihost_8000qps (8k-q4)",
      "protocol": "HTTP",
      "direction": "ingress",
      "category": "stress",
      "includeInTotalRps": true,
      "p50": 12.97,
      "p95": 36.54,
      "p99": 0,
      "err": 1.16,
      "rps": 7040.1,
      "requests": 140802
    },
    {
      "name": "egress_multihost_8000qps (8k-q4)",
      "protocol": "HTTP",
      "direction": "egress",
      "category": "stress",
      "includeInTotalRps": true,
      "p50": 56.12,
      "p95": 140.04,
      "p99": 0,
      "err": 0.01,
      "rps": 7163.4,
      "requests": 143268
    },
    {
      "name": "ingress_3000qps",
      "protocol": "HTTP",
      "direction": "ingress",
      "category": "stress",
      "includeInTotalRps": true,
      "p50": 1.21,
      "p95": 33.56,
      "p99": 0,
      "err": 0,
      "rps": 2950.1,
      "requests": 59002,
      "tunnel": "frp"
    },
    {
      "name": "ingress_multihost",
      "protocol": "HTTP",
      "direction": "ingress",
      "category": "stress",
      "includeInTotalRps": true,
      "p50": 0.65,
      "p95": 3.21,
      "p99": 0,
      "err": 0,
      "rps": 2999.15,
      "requests": 59983,
      "tunnel": "frp"
    },
    {
      "name": "ingress_multihost_8000qps",
      "protocol": "HTTP",
      "direction": "ingress",
      "category": "stress",
      "includeInTotalRps": true,
      "p50": 40.71,
      "p95": 113.97,
      "p99": 0,
      "err": 0,
      "rps": 5622.7,
      "requests": 112454,
      "tunnel": "frp"
    }
  ],
  "summary": {
    "totalRPS": 40348.2,
    "totalErr": 0.2,
    "totalRequests": 813894
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
  "run_url": "https://github.com/locustbaby/duotunnel/actions/runs/24184124275",
  "resources": {
    "processes": {
      "server": {
        "cpu": [
          {
            "t": 77.3,
            "v": 0.26
          },
          {
            "t": 80.9,
            "v": 1.03
          },
          {
            "t": 84.6,
            "v": 1.48
          },
          {
            "t": 88.4,
            "v": 1.68
          },
          {
            "t": 92.1,
            "v": 1.81
          },
          {
            "t": 95.8,
            "v": 1.62
          },
          {
            "t": 99.5,
            "v": 1.54
          },
          {
            "t": 103.3,
            "v": 0.8
          },
          {
            "t": 106.9,
            "v": 0.14
          },
          {
            "t": 110.5,
            "v": 0.56
          },
          {
            "t": 114.2,
            "v": 1.36
          },
          {
            "t": 117.9,
            "v": 2.03
          },
          {
            "t": 121.6,
            "v": 2.2
          },
          {
            "t": 125.3,
            "v": 2.14
          },
          {
            "t": 129.1,
            "v": 2.21
          },
          {
            "t": 132.8,
            "v": 1.28
          },
          {
            "t": 140.0,
            "v": 6.86
          },
          {
            "t": 144.5,
            "v": 11.85
          },
          {
            "t": 149.0,
            "v": 11.94
          },
          {
            "t": 153.4,
            "v": 11.93
          },
          {
            "t": 157.9,
            "v": 12.02
          },
          {
            "t": 161.8,
            "v": 0.25
          },
          {
            "t": 165.5,
            "v": 10.34
          },
          {
            "t": 169.9,
            "v": 14.62
          },
          {
            "t": 174.5,
            "v": 14.69
          },
          {
            "t": 178.9,
            "v": 14.48
          },
          {
            "t": 183.4,
            "v": 13.5
          },
          {
            "t": 190.8,
            "v": 9.52
          },
          {
            "t": 195.3,
            "v": 12.07
          },
          {
            "t": 199.8,
            "v": 11.79
          },
          {
            "t": 204.3,
            "v": 11.75
          },
          {
            "t": 208.8,
            "v": 9.83
          },
          {
            "t": 216.2,
            "v": 12.91
          },
          {
            "t": 220.8,
            "v": 14.5
          },
          {
            "t": 225.3,
            "v": 14.46
          },
          {
            "t": 229.9,
            "v": 14.52
          }
        ],
        "rss": [
          {
            "t": 3.6,
            "v": 15.2
          },
          {
            "t": 8.6,
            "v": 15.2
          },
          {
            "t": 14.4,
            "v": 15.2
          },
          {
            "t": 20.1,
            "v": 15.2
          },
          {
            "t": 25.3,
            "v": 15.2
          },
          {
            "t": 28.9,
            "v": 15.2
          },
          {
            "t": 33.7,
            "v": 15.2
          },
          {
            "t": 38.4,
            "v": 15.2
          },
          {
            "t": 43.1,
            "v": 15.2
          },
          {
            "t": 47.8,
            "v": 15.2
          },
          {
            "t": 51.4,
            "v": 15.2
          },
          {
            "t": 56.2,
            "v": 15.2
          },
          {
            "t": 64.3,
            "v": 15.2
          },
          {
            "t": 73.4,
            "v": 15.3
          },
          {
            "t": 77.3,
            "v": 15.5
          },
          {
            "t": 80.9,
            "v": 16.6
          },
          {
            "t": 84.6,
            "v": 17.2
          },
          {
            "t": 88.4,
            "v": 17.5
          },
          {
            "t": 92.1,
            "v": 17.6
          },
          {
            "t": 95.8,
            "v": 17.5
          },
          {
            "t": 99.5,
            "v": 17.4
          },
          {
            "t": 103.3,
            "v": 17.5
          },
          {
            "t": 106.9,
            "v": 17.5
          },
          {
            "t": 110.5,
            "v": 19.2
          },
          {
            "t": 114.2,
            "v": 20.4
          },
          {
            "t": 117.9,
            "v": 21.4
          },
          {
            "t": 121.6,
            "v": 21.5
          },
          {
            "t": 125.3,
            "v": 21.5
          },
          {
            "t": 129.1,
            "v": 21.6
          },
          {
            "t": 132.8,
            "v": 21.1
          },
          {
            "t": 136.4,
            "v": 21.1
          },
          {
            "t": 140.0,
            "v": 22.2
          },
          {
            "t": 144.5,
            "v": 22.2
          },
          {
            "t": 149.0,
            "v": 22.3
          },
          {
            "t": 153.4,
            "v": 22.1
          },
          {
            "t": 157.9,
            "v": 22.1
          },
          {
            "t": 161.8,
            "v": 22.1
          },
          {
            "t": 165.5,
            "v": 24.1
          },
          {
            "t": 169.9,
            "v": 24.7
          },
          {
            "t": 174.5,
            "v": 24.7
          },
          {
            "t": 178.9,
            "v": 26.0
          },
          {
            "t": 183.4,
            "v": 25.3
          },
          {
            "t": 187.1,
            "v": 25.3
          },
          {
            "t": 190.8,
            "v": 27.9
          },
          {
            "t": 195.3,
            "v": 27.9
          },
          {
            "t": 199.8,
            "v": 27.9
          },
          {
            "t": 204.3,
            "v": 27.9
          },
          {
            "t": 208.8,
            "v": 27.9
          },
          {
            "t": 212.5,
            "v": 27.9
          },
          {
            "t": 216.2,
            "v": 29.4
          },
          {
            "t": 220.8,
            "v": 29.2
          },
          {
            "t": 225.3,
            "v": 28.6
          },
          {
            "t": 229.9,
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
      "client": {
        "cpu": [
          {
            "t": 73.4,
            "v": 0.03
          },
          {
            "t": 77.3,
            "v": 0.32
          },
          {
            "t": 80.9,
            "v": 1.03
          },
          {
            "t": 84.6,
            "v": 1.41
          },
          {
            "t": 88.4,
            "v": 1.61
          },
          {
            "t": 92.1,
            "v": 1.68
          },
          {
            "t": 95.8,
            "v": 1.55
          },
          {
            "t": 99.5,
            "v": 1.41
          },
          {
            "t": 103.3,
            "v": 0.6
          },
          {
            "t": 106.9,
            "v": 0.07
          },
          {
            "t": 110.5,
            "v": 0.62
          },
          {
            "t": 114.2,
            "v": 1.23
          },
          {
            "t": 117.9,
            "v": 1.97
          },
          {
            "t": 121.6,
            "v": 2.13
          },
          {
            "t": 125.3,
            "v": 2.01
          },
          {
            "t": 129.1,
            "v": 2.07
          },
          {
            "t": 132.8,
            "v": 1.14
          },
          {
            "t": 140.0,
            "v": 8.52
          },
          {
            "t": 144.5,
            "v": 14.61
          },
          {
            "t": 149.0,
            "v": 14.6
          },
          {
            "t": 153.4,
            "v": 14.67
          },
          {
            "t": 157.9,
            "v": 14.59
          },
          {
            "t": 161.8,
            "v": 0.25
          },
          {
            "t": 165.5,
            "v": 8.55
          },
          {
            "t": 169.9,
            "v": 12.15
          },
          {
            "t": 174.5,
            "v": 11.82
          },
          {
            "t": 178.9,
            "v": 11.81
          },
          {
            "t": 183.4,
            "v": 10.91
          },
          {
            "t": 190.8,
            "v": 11.66
          },
          {
            "t": 195.3,
            "v": 14.57
          },
          {
            "t": 199.8,
            "v": 14.5
          },
          {
            "t": 204.3,
            "v": 14.28
          },
          {
            "t": 208.8,
            "v": 11.88
          },
          {
            "t": 216.2,
            "v": 10.59
          },
          {
            "t": 220.8,
            "v": 11.74
          },
          {
            "t": 225.3,
            "v": 11.77
          },
          {
            "t": 229.9,
            "v": 11.68
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
            "t": 14.4,
            "v": 11.2
          },
          {
            "t": 20.1,
            "v": 11.2
          },
          {
            "t": 25.3,
            "v": 11.2
          },
          {
            "t": 28.9,
            "v": 11.2
          },
          {
            "t": 33.7,
            "v": 11.2
          },
          {
            "t": 38.4,
            "v": 11.2
          },
          {
            "t": 43.1,
            "v": 11.2
          },
          {
            "t": 47.8,
            "v": 11.2
          },
          {
            "t": 51.4,
            "v": 11.2
          },
          {
            "t": 56.2,
            "v": 11.2
          },
          {
            "t": 64.3,
            "v": 11.2
          },
          {
            "t": 73.4,
            "v": 11.2
          },
          {
            "t": 77.3,
            "v": 11.6
          },
          {
            "t": 80.9,
            "v": 12.7
          },
          {
            "t": 84.6,
            "v": 13.1
          },
          {
            "t": 88.4,
            "v": 13.3
          },
          {
            "t": 92.1,
            "v": 13.4
          },
          {
            "t": 95.8,
            "v": 13.5
          },
          {
            "t": 99.5,
            "v": 13.6
          },
          {
            "t": 103.3,
            "v": 13.6
          },
          {
            "t": 106.9,
            "v": 13.6
          },
          {
            "t": 110.5,
            "v": 16.7
          },
          {
            "t": 114.2,
            "v": 18.3
          },
          {
            "t": 117.9,
            "v": 18.8
          },
          {
            "t": 121.6,
            "v": 18.9
          },
          {
            "t": 125.3,
            "v": 18.8
          },
          {
            "t": 129.1,
            "v": 18.9
          },
          {
            "t": 132.8,
            "v": 18.9
          },
          {
            "t": 136.4,
            "v": 18.9
          },
          {
            "t": 140.0,
            "v": 18.4
          },
          {
            "t": 144.5,
            "v": 18.8
          },
          {
            "t": 149.0,
            "v": 19.0
          },
          {
            "t": 153.4,
            "v": 18.5
          },
          {
            "t": 157.9,
            "v": 18.4
          },
          {
            "t": 161.8,
            "v": 18.4
          },
          {
            "t": 165.5,
            "v": 20.1
          },
          {
            "t": 169.9,
            "v": 20.0
          },
          {
            "t": 174.5,
            "v": 20.0
          },
          {
            "t": 178.9,
            "v": 20.1
          },
          {
            "t": 183.4,
            "v": 19.9
          },
          {
            "t": 187.1,
            "v": 19.9
          },
          {
            "t": 190.8,
            "v": 23.0
          },
          {
            "t": 195.3,
            "v": 23.0
          },
          {
            "t": 199.8,
            "v": 22.5
          },
          {
            "t": 204.3,
            "v": 22.7
          },
          {
            "t": 208.8,
            "v": 22.8
          },
          {
            "t": 212.5,
            "v": 22.8
          },
          {
            "t": 216.2,
            "v": 23.3
          },
          {
            "t": 220.8,
            "v": 22.9
          },
          {
            "t": 225.3,
            "v": 22.9
          },
          {
            "t": 229.9,
            "v": 22.9
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
            "v": 12.5
          },
          {
            "t": 80.0,
            "v": 13.0
          },
          {
            "t": 82.0,
            "v": 14.5
          },
          {
            "t": 84.0,
            "v": 12.0
          },
          {
            "t": 86.0,
            "v": 13.5
          },
          {
            "t": 88.0,
            "v": 11.5
          },
          {
            "t": 90.0,
            "v": 13.5
          },
          {
            "t": 92.0,
            "v": 13.5
          },
          {
            "t": 94.0,
            "v": 11.5
          },
          {
            "t": 96.0,
            "v": 5.0
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
            "v": 10.0
          },
          {
            "t": 110.0,
            "v": 9.5
          },
          {
            "t": 112.0,
            "v": 13.0
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
            "v": 7.0
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
            "v": 6.5
          },
          {
            "t": 128.0,
            "v": 8.5
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
            "v": 64.0
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
            "v": 87.5
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
            "v": 1.5
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
            "v": 1.0
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
            "v": 0.5
          },
          {
            "t": 92.0,
            "v": 1.5
          },
          {
            "t": 94.0,
            "v": 0.5
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
            "v": 1.5
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
            "v": 0.5
          },
          {
            "t": 126.0,
            "v": 0.0
          },
          {
            "t": 128.0,
            "v": 0.5
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
            "v": 1.5
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
            "v": 7.5
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
            "v": 2.38
          },
          {
            "t": 8.6,
            "v": 4.46
          },
          {
            "t": 14.4,
            "v": 4.81
          },
          {
            "t": 20.1,
            "v": 4.34
          },
          {
            "t": 25.3,
            "v": 1.3
          },
          {
            "t": 28.9,
            "v": 2.4
          },
          {
            "t": 33.7,
            "v": 4.05
          },
          {
            "t": 38.4,
            "v": 4.01
          },
          {
            "t": 43.1,
            "v": 3.97
          },
          {
            "t": 47.8,
            "v": 3.03
          },
          {
            "t": 56.2,
            "v": 6.48
          },
          {
            "t": 64.3,
            "v": 6.1
          },
          {
            "t": 73.4,
            "v": 4.51
          },
          {
            "t": 77.3,
            "v": 0.06
          },
          {
            "t": 80.9,
            "v": 0.14
          },
          {
            "t": 84.6,
            "v": 0.2
          },
          {
            "t": 88.4,
            "v": 0.27
          },
          {
            "t": 92.1,
            "v": 0.34
          },
          {
            "t": 95.8,
            "v": 0.27
          },
          {
            "t": 99.5,
            "v": 0.34
          },
          {
            "t": 103.3,
            "v": 0.13
          },
          {
            "t": 110.5,
            "v": 0.21
          },
          {
            "t": 114.2,
            "v": 0.27
          },
          {
            "t": 117.9,
            "v": 0.27
          },
          {
            "t": 121.6,
            "v": 0.33
          },
          {
            "t": 125.3,
            "v": 0.2
          },
          {
            "t": 129.1,
            "v": 0.33
          },
          {
            "t": 140.0,
            "v": 1.87
          },
          {
            "t": 144.5,
            "v": 3.43
          },
          {
            "t": 149.0,
            "v": 3.44
          },
          {
            "t": 153.4,
            "v": 3.42
          },
          {
            "t": 157.9,
            "v": 3.47
          },
          {
            "t": 161.8,
            "v": 0.13
          },
          {
            "t": 165.5,
            "v": 2.27
          },
          {
            "t": 169.9,
            "v": 3.42
          },
          {
            "t": 174.5,
            "v": 3.42
          },
          {
            "t": 178.9,
            "v": 3.45
          },
          {
            "t": 183.4,
            "v": 3.22
          },
          {
            "t": 190.8,
            "v": 2.62
          },
          {
            "t": 195.3,
            "v": 3.39
          },
          {
            "t": 199.8,
            "v": 3.43
          },
          {
            "t": 204.3,
            "v": 3.42
          },
          {
            "t": 208.8,
            "v": 2.89
          },
          {
            "t": 216.2,
            "v": 2.91
          },
          {
            "t": 220.8,
            "v": 3.47
          },
          {
            "t": 225.3,
            "v": 3.41
          },
          {
            "t": 229.9,
            "v": 3.49
          },
          {
            "t": 234.5,
            "v": 2.35
          }
        ],
        "rss": [
          {
            "t": 3.6,
            "v": 5.9
          },
          {
            "t": 8.6,
            "v": 7.7
          },
          {
            "t": 14.4,
            "v": 10.6
          },
          {
            "t": 20.1,
            "v": 10.3
          },
          {
            "t": 25.3,
            "v": 11.3
          },
          {
            "t": 28.9,
            "v": 11.3
          },
          {
            "t": 33.7,
            "v": 11.3
          },
          {
            "t": 38.4,
            "v": 11.3
          },
          {
            "t": 43.1,
            "v": 11.3
          },
          {
            "t": 47.8,
            "v": 11.3
          },
          {
            "t": 51.4,
            "v": 11.3
          },
          {
            "t": 56.2,
            "v": 13.5
          },
          {
            "t": 64.3,
            "v": 17.4
          },
          {
            "t": 73.4,
            "v": 19.6
          },
          {
            "t": 77.3,
            "v": 19.6
          },
          {
            "t": 80.9,
            "v": 19.6
          },
          {
            "t": 84.6,
            "v": 19.6
          },
          {
            "t": 88.4,
            "v": 19.6
          },
          {
            "t": 92.1,
            "v": 19.6
          },
          {
            "t": 95.8,
            "v": 19.6
          },
          {
            "t": 99.5,
            "v": 19.6
          },
          {
            "t": 103.3,
            "v": 19.6
          },
          {
            "t": 106.9,
            "v": 19.6
          },
          {
            "t": 110.5,
            "v": 19.8
          },
          {
            "t": 114.2,
            "v": 19.8
          },
          {
            "t": 117.9,
            "v": 19.8
          },
          {
            "t": 121.6,
            "v": 19.8
          },
          {
            "t": 125.3,
            "v": 19.8
          },
          {
            "t": 129.1,
            "v": 19.8
          },
          {
            "t": 132.8,
            "v": 19.8
          },
          {
            "t": 136.4,
            "v": 19.8
          },
          {
            "t": 140.0,
            "v": 19.8
          },
          {
            "t": 144.5,
            "v": 19.6
          },
          {
            "t": 149.0,
            "v": 19.6
          },
          {
            "t": 153.4,
            "v": 19.6
          },
          {
            "t": 157.9,
            "v": 19.6
          },
          {
            "t": 161.8,
            "v": 19.6
          },
          {
            "t": 165.5,
            "v": 19.6
          },
          {
            "t": 169.9,
            "v": 19.6
          },
          {
            "t": 174.5,
            "v": 19.6
          },
          {
            "t": 178.9,
            "v": 19.7
          },
          {
            "t": 183.4,
            "v": 19.7
          },
          {
            "t": 187.1,
            "v": 19.7
          },
          {
            "t": 190.8,
            "v": 19.7
          },
          {
            "t": 195.3,
            "v": 19.7
          },
          {
            "t": 199.8,
            "v": 19.7
          },
          {
            "t": 204.3,
            "v": 19.7
          },
          {
            "t": 208.8,
            "v": 19.7
          },
          {
            "t": 212.5,
            "v": 19.7
          },
          {
            "t": 216.2,
            "v": 19.6
          },
          {
            "t": 220.8,
            "v": 19.6
          },
          {
            "t": 225.3,
            "v": 19.6
          },
          {
            "t": 229.9,
            "v": 19.6
          },
          {
            "t": 234.5,
            "v": 19.6
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
            "v": 92.5
          },
          {
            "t": 4.0,
            "v": 127.0
          },
          {
            "t": 6.0,
            "v": 80.5
          },
          {
            "t": 8.0,
            "v": 220.0
          },
          {
            "t": 10.0,
            "v": 325.5
          },
          {
            "t": 12.0,
            "v": 122.5
          },
          {
            "t": 14.0,
            "v": 261.0
          },
          {
            "t": 16.0,
            "v": 152.0
          },
          {
            "t": 18.0,
            "v": 82.5
          },
          {
            "t": 20.0,
            "v": 291.5
          },
          {
            "t": 22.0,
            "v": 74.5
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
            "v": 1.0
          },
          {
            "t": 30.0,
            "v": 0.0
          },
          {
            "t": 32.0,
            "v": 1.0
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
            "v": 31.0
          },
          {
            "t": 54.0,
            "v": 26.0
          },
          {
            "t": 56.0,
            "v": 104.0
          },
          {
            "t": 58.0,
            "v": 136.5
          },
          {
            "t": 60.0,
            "v": 141.3
          },
          {
            "t": 62.0,
            "v": 218.5
          },
          {
            "t": 64.0,
            "v": 182.5
          },
          {
            "t": 66.0,
            "v": 236.0
          },
          {
            "t": 68.0,
            "v": 265.7
          },
          {
            "t": 70.0,
            "v": 209.0
          },
          {
            "t": 72.0,
            "v": 94.0
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
            "v": 9.0
          },
          {
            "t": 140.0,
            "v": 10.0
          },
          {
            "t": 142.0,
            "v": 6.5
          },
          {
            "t": 144.0,
            "v": 4.5
          },
          {
            "t": 146.0,
            "v": 1.5
          },
          {
            "t": 148.0,
            "v": 7.0
          },
          {
            "t": 150.0,
            "v": 9.0
          },
          {
            "t": 152.0,
            "v": 1.0
          },
          {
            "t": 154.0,
            "v": 2.5
          },
          {
            "t": 156.0,
            "v": 1.0
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
            "v": 0.5
          },
          {
            "t": 166.0,
            "v": 6.0
          },
          {
            "t": 168.0,
            "v": 2.5
          },
          {
            "t": 170.0,
            "v": 0.0
          },
          {
            "t": 172.0,
            "v": 4.0
          },
          {
            "t": 174.0,
            "v": 3.5
          },
          {
            "t": 176.0,
            "v": 12.0
          },
          {
            "t": 178.0,
            "v": 17.0
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
            "v": 2.5
          },
          {
            "t": 190.0,
            "v": 9.5
          },
          {
            "t": 192.0,
            "v": 1.5
          },
          {
            "t": 194.0,
            "v": 6.5
          },
          {
            "t": 196.0,
            "v": 0.5
          },
          {
            "t": 198.0,
            "v": 0.5
          },
          {
            "t": 200.0,
            "v": 5.5
          },
          {
            "t": 202.0,
            "v": 4.0
          },
          {
            "t": 204.0,
            "v": 7.0
          },
          {
            "t": 206.0,
            "v": 3.5
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
            "v": 3.5
          },
          {
            "t": 216.0,
            "v": 4.0
          },
          {
            "t": 218.0,
            "v": 1.5
          },
          {
            "t": 220.0,
            "v": 4.5
          },
          {
            "t": 222.0,
            "v": 5.5
          },
          {
            "t": 224.0,
            "v": 5.5
          },
          {
            "t": 226.0,
            "v": 16.0
          },
          {
            "t": 228.0,
            "v": 0.5
          },
          {
            "t": 230.0,
            "v": 5.0
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
            "v": 24.5
          },
          {
            "t": 4.0,
            "v": 39.5
          },
          {
            "t": 6.0,
            "v": 23.5
          },
          {
            "t": 8.0,
            "v": 73.5
          },
          {
            "t": 10.0,
            "v": 170.0
          },
          {
            "t": 12.0,
            "v": 44.0
          },
          {
            "t": 14.0,
            "v": 86.5
          },
          {
            "t": 16.0,
            "v": 45.0
          },
          {
            "t": 18.0,
            "v": 15.5
          },
          {
            "t": 20.0,
            "v": 134.8
          },
          {
            "t": 22.0,
            "v": 45.5
          },
          {
            "t": 24.0,
            "v": 0.0
          },
          {
            "t": 26.0,
            "v": 9.0
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
            "v": 13.5
          },
          {
            "t": 58.0,
            "v": 32.0
          },
          {
            "t": 60.0,
            "v": 77.6
          },
          {
            "t": 62.0,
            "v": 88.5
          },
          {
            "t": 64.0,
            "v": 89.5
          },
          {
            "t": 66.0,
            "v": 91.5
          },
          {
            "t": 68.0,
            "v": 291.0
          },
          {
            "t": 70.0,
            "v": 126.0
          },
          {
            "t": 72.0,
            "v": 18.5
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
            "v": 1.0
          },
          {
            "t": 140.0,
            "v": 2.5
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
            "v": 2.0
          },
          {
            "t": 148.0,
            "v": 2.0
          },
          {
            "t": 150.0,
            "v": 0.5
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
            "v": 2.0
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
            "v": 0.5
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
            "v": 3.5
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
            "v": 0.5
          },
          {
            "t": 224.0,
            "v": 0.0
          },
          {
            "t": 226.0,
            "v": 4.5
          },
          {
            "t": 228.0,
            "v": 0.0
          },
          {
            "t": 230.0,
            "v": 0.5
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
            "t": 80.9,
            "v": 0.07
          },
          {
            "t": 84.6,
            "v": 0.07
          },
          {
            "t": 92.1,
            "v": 0.07
          },
          {
            "t": 95.8,
            "v": 0.13
          },
          {
            "t": 114.2,
            "v": 0.07
          },
          {
            "t": 121.6,
            "v": 0.07
          },
          {
            "t": 129.1,
            "v": 0.13
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
            "t": 14.4,
            "v": 2.7
          },
          {
            "t": 20.1,
            "v": 2.7
          },
          {
            "t": 25.3,
            "v": 2.7
          },
          {
            "t": 28.9,
            "v": 2.7
          },
          {
            "t": 33.7,
            "v": 2.7
          },
          {
            "t": 38.4,
            "v": 2.7
          },
          {
            "t": 43.1,
            "v": 2.7
          },
          {
            "t": 47.8,
            "v": 2.7
          },
          {
            "t": 51.4,
            "v": 2.7
          },
          {
            "t": 56.2,
            "v": 2.7
          },
          {
            "t": 64.3,
            "v": 2.7
          },
          {
            "t": 73.4,
            "v": 2.7
          },
          {
            "t": 77.3,
            "v": 3.0
          },
          {
            "t": 80.9,
            "v": 3.0
          },
          {
            "t": 84.6,
            "v": 3.0
          },
          {
            "t": 88.4,
            "v": 3.0
          },
          {
            "t": 92.1,
            "v": 3.0
          },
          {
            "t": 95.8,
            "v": 3.0
          },
          {
            "t": 99.5,
            "v": 3.0
          },
          {
            "t": 103.3,
            "v": 3.0
          },
          {
            "t": 106.9,
            "v": 3.0
          },
          {
            "t": 110.5,
            "v": 3.0
          },
          {
            "t": 114.2,
            "v": 3.0
          },
          {
            "t": 117.9,
            "v": 3.0
          },
          {
            "t": 121.6,
            "v": 3.0
          },
          {
            "t": 125.3,
            "v": 3.0
          },
          {
            "t": 129.1,
            "v": 3.0
          },
          {
            "t": 132.8,
            "v": 3.0
          },
          {
            "t": 136.4,
            "v": 3.0
          },
          {
            "t": 140.0,
            "v": 3.0
          },
          {
            "t": 144.5,
            "v": 3.0
          },
          {
            "t": 149.0,
            "v": 3.0
          },
          {
            "t": 153.4,
            "v": 3.0
          },
          {
            "t": 157.9,
            "v": 3.0
          },
          {
            "t": 161.8,
            "v": 3.0
          },
          {
            "t": 165.5,
            "v": 3.0
          },
          {
            "t": 169.9,
            "v": 3.0
          },
          {
            "t": 174.5,
            "v": 3.0
          },
          {
            "t": 178.9,
            "v": 3.0
          },
          {
            "t": 183.4,
            "v": 3.0
          },
          {
            "t": 187.1,
            "v": 3.0
          },
          {
            "t": 190.8,
            "v": 3.0
          },
          {
            "t": 195.3,
            "v": 3.0
          },
          {
            "t": 199.8,
            "v": 3.0
          },
          {
            "t": 204.3,
            "v": 3.0
          },
          {
            "t": 208.8,
            "v": 3.0
          },
          {
            "t": 212.5,
            "v": 3.0
          },
          {
            "t": 216.2,
            "v": 3.0
          },
          {
            "t": 220.8,
            "v": 3.0
          },
          {
            "t": 225.3,
            "v": 3.0
          },
          {
            "t": 229.9,
            "v": 3.0
          },
          {
            "t": 234.5,
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
            "v": 3.5
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
            "v": 5.0
          },
          {
            "t": 80.0,
            "v": 4.5
          },
          {
            "t": 82.0,
            "v": 3.5
          },
          {
            "t": 84.0,
            "v": 4.5
          },
          {
            "t": 86.0,
            "v": 5.5
          },
          {
            "t": 88.0,
            "v": 3.5
          },
          {
            "t": 90.0,
            "v": 7.0
          },
          {
            "t": 92.0,
            "v": 5.5
          },
          {
            "t": 94.0,
            "v": 5.0
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
            "v": 1.0
          },
          {
            "t": 110.0,
            "v": 1.0
          },
          {
            "t": 112.0,
            "v": 2.0
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
            "v": 3.0
          },
          {
            "t": 120.0,
            "v": 3.0
          },
          {
            "t": 122.0,
            "v": 5.5
          },
          {
            "t": 124.0,
            "v": 4.5
          },
          {
            "t": 126.0,
            "v": 3.0
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
            "t": 84.6,
            "v": 0.13
          },
          {
            "t": 88.4,
            "v": 0.2
          },
          {
            "t": 92.1,
            "v": 0.2
          },
          {
            "t": 95.8,
            "v": 0.13
          },
          {
            "t": 99.5,
            "v": 0.2
          },
          {
            "t": 114.2,
            "v": 0.14
          },
          {
            "t": 117.9,
            "v": 0.34
          },
          {
            "t": 121.6,
            "v": 0.47
          },
          {
            "t": 125.3,
            "v": 0.4
          },
          {
            "t": 129.1,
            "v": 0.47
          },
          {
            "t": 132.8,
            "v": 0.34
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
            "t": 14.4,
            "v": 3.3
          },
          {
            "t": 20.1,
            "v": 3.3
          },
          {
            "t": 25.3,
            "v": 3.3
          },
          {
            "t": 28.9,
            "v": 3.3
          },
          {
            "t": 33.7,
            "v": 3.3
          },
          {
            "t": 38.4,
            "v": 3.3
          },
          {
            "t": 43.1,
            "v": 3.3
          },
          {
            "t": 47.8,
            "v": 3.3
          },
          {
            "t": 51.4,
            "v": 3.3
          },
          {
            "t": 56.2,
            "v": 3.3
          },
          {
            "t": 64.3,
            "v": 3.3
          },
          {
            "t": 73.4,
            "v": 3.3
          },
          {
            "t": 77.3,
            "v": 3.3
          },
          {
            "t": 80.9,
            "v": 4.1
          },
          {
            "t": 84.6,
            "v": 4.1
          },
          {
            "t": 88.4,
            "v": 4.2
          },
          {
            "t": 92.1,
            "v": 4.2
          },
          {
            "t": 95.8,
            "v": 4.2
          },
          {
            "t": 99.5,
            "v": 4.2
          },
          {
            "t": 103.3,
            "v": 4.2
          },
          {
            "t": 106.9,
            "v": 4.2
          },
          {
            "t": 110.5,
            "v": 4.2
          },
          {
            "t": 114.2,
            "v": 4.3
          },
          {
            "t": 117.9,
            "v": 4.3
          },
          {
            "t": 121.6,
            "v": 4.4
          },
          {
            "t": 125.3,
            "v": 4.4
          },
          {
            "t": 129.1,
            "v": 4.4
          },
          {
            "t": 132.8,
            "v": 4.4
          },
          {
            "t": 136.4,
            "v": 4.4
          },
          {
            "t": 140.0,
            "v": 4.4
          },
          {
            "t": 144.5,
            "v": 4.4
          },
          {
            "t": 149.0,
            "v": 4.4
          },
          {
            "t": 153.4,
            "v": 4.4
          },
          {
            "t": 157.9,
            "v": 4.4
          },
          {
            "t": 161.8,
            "v": 4.4
          },
          {
            "t": 165.5,
            "v": 4.4
          },
          {
            "t": 169.9,
            "v": 4.4
          },
          {
            "t": 174.5,
            "v": 4.4
          },
          {
            "t": 178.9,
            "v": 4.4
          },
          {
            "t": 183.4,
            "v": 4.4
          },
          {
            "t": 187.1,
            "v": 4.4
          },
          {
            "t": 190.8,
            "v": 4.4
          },
          {
            "t": 195.3,
            "v": 4.4
          },
          {
            "t": 199.8,
            "v": 4.4
          },
          {
            "t": 204.3,
            "v": 4.4
          },
          {
            "t": 208.8,
            "v": 4.4
          },
          {
            "t": 212.5,
            "v": 4.4
          },
          {
            "t": 216.2,
            "v": 4.4
          },
          {
            "t": 220.8,
            "v": 4.4
          },
          {
            "t": 225.3,
            "v": 4.4
          },
          {
            "t": 229.9,
            "v": 4.4
          },
          {
            "t": 234.5,
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
            "v": 23.71
          },
          {
            "t": 14.4,
            "v": 20.54
          },
          {
            "t": 20.1,
            "v": 21.72
          },
          {
            "t": 25.3,
            "v": 0.05
          },
          {
            "t": 28.9,
            "v": 19.85
          },
          {
            "t": 33.7,
            "v": 22.93
          },
          {
            "t": 38.4,
            "v": 22.57
          },
          {
            "t": 43.1,
            "v": 22.36
          },
          {
            "t": 47.8,
            "v": 12.83
          },
          {
            "t": 51.4,
            "v": 0.07
          },
          {
            "t": 56.2,
            "v": 65.26
          },
          {
            "t": 64.3,
            "v": 36.55
          },
          {
            "t": 77.3,
            "v": 0.65
          },
          {
            "t": 80.9,
            "v": 1.85
          },
          {
            "t": 84.6,
            "v": 2.62
          },
          {
            "t": 88.4,
            "v": 2.89
          },
          {
            "t": 92.1,
            "v": 3.16
          },
          {
            "t": 95.8,
            "v": 3.03
          },
          {
            "t": 99.5,
            "v": 2.95
          },
          {
            "t": 103.3,
            "v": 1.34
          },
          {
            "t": 106.9,
            "v": 0.21
          },
          {
            "t": 110.5,
            "v": 2.57
          },
          {
            "t": 114.2,
            "v": 2.05
          },
          {
            "t": 117.9,
            "v": 3.05
          },
          {
            "t": 121.6,
            "v": 4.79
          },
          {
            "t": 125.3,
            "v": 3.08
          },
          {
            "t": 129.1,
            "v": 3.15
          },
          {
            "t": 132.8,
            "v": 1.55
          },
          {
            "t": 136.4,
            "v": 0.07
          },
          {
            "t": 140.0,
            "v": 17.74
          },
          {
            "t": 144.5,
            "v": 23.82
          },
          {
            "t": 149.0,
            "v": 21.59
          },
          {
            "t": 153.4,
            "v": 23.24
          },
          {
            "t": 157.9,
            "v": 21.52
          },
          {
            "t": 161.8,
            "v": 0.06
          },
          {
            "t": 165.5,
            "v": 18.68
          },
          {
            "t": 169.9,
            "v": 23.86
          },
          {
            "t": 174.5,
            "v": 23.97
          },
          {
            "t": 178.9,
            "v": 23.28
          },
          {
            "t": 183.4,
            "v": 18.36
          },
          {
            "t": 187.1,
            "v": 0.07
          },
          {
            "t": 190.8,
            "v": 24.15
          },
          {
            "t": 195.3,
            "v": 25.14
          },
          {
            "t": 199.8,
            "v": 25.01
          },
          {
            "t": 204.3,
            "v": 24.6
          },
          {
            "t": 208.8,
            "v": 17.1
          },
          {
            "t": 212.5,
            "v": 0.07
          },
          {
            "t": 216.2,
            "v": 26.29
          },
          {
            "t": 220.8,
            "v": 25.8
          },
          {
            "t": 225.3,
            "v": 25.73
          },
          {
            "t": 229.9,
            "v": 25.65
          }
        ],
        "rss": [
          {
            "t": 3.6,
            "v": 126.1
          },
          {
            "t": 8.6,
            "v": 152.2
          },
          {
            "t": 14.4,
            "v": 181.0
          },
          {
            "t": 20.1,
            "v": 196.3
          },
          {
            "t": 25.3,
            "v": 196.3
          },
          {
            "t": 28.9,
            "v": 190.3
          },
          {
            "t": 33.7,
            "v": 192.7
          },
          {
            "t": 38.4,
            "v": 201.4
          },
          {
            "t": 43.1,
            "v": 210.7
          },
          {
            "t": 47.8,
            "v": 210.8
          },
          {
            "t": 51.4,
            "v": 210.8
          },
          {
            "t": 56.2,
            "v": 414.1
          },
          {
            "t": 64.3,
            "v": 488.5
          },
          {
            "t": 73.4,
            "v": 392.1
          },
          {
            "t": 77.3,
            "v": 392.5
          },
          {
            "t": 80.9,
            "v": 392.8
          },
          {
            "t": 84.6,
            "v": 393.3
          },
          {
            "t": 88.4,
            "v": 398.9
          },
          {
            "t": 92.1,
            "v": 421.1
          },
          {
            "t": 95.8,
            "v": 447.3
          },
          {
            "t": 99.5,
            "v": 470.0
          },
          {
            "t": 103.3,
            "v": 480.0
          },
          {
            "t": 106.9,
            "v": 481.3
          },
          {
            "t": 110.5,
            "v": 487.5
          },
          {
            "t": 114.2,
            "v": 488.3
          },
          {
            "t": 117.9,
            "v": 489.5
          },
          {
            "t": 121.6,
            "v": 490.8
          },
          {
            "t": 125.3,
            "v": 490.5
          },
          {
            "t": 129.1,
            "v": 490.6
          },
          {
            "t": 132.8,
            "v": 490.6
          },
          {
            "t": 136.4,
            "v": 490.6
          },
          {
            "t": 140.0,
            "v": 492.1
          },
          {
            "t": 144.5,
            "v": 508.3
          },
          {
            "t": 149.0,
            "v": 512.2
          },
          {
            "t": 153.4,
            "v": 514.6
          },
          {
            "t": 157.9,
            "v": 521.9
          },
          {
            "t": 161.8,
            "v": 521.9
          },
          {
            "t": 165.5,
            "v": 527.5
          },
          {
            "t": 169.9,
            "v": 532.1
          },
          {
            "t": 174.5,
            "v": 532.2
          },
          {
            "t": 178.9,
            "v": 532.2
          },
          {
            "t": 183.4,
            "v": 539.8
          },
          {
            "t": 187.1,
            "v": 539.8
          },
          {
            "t": 190.8,
            "v": 546.9
          },
          {
            "t": 195.3,
            "v": 557.6
          },
          {
            "t": 199.8,
            "v": 557.7
          },
          {
            "t": 204.3,
            "v": 571.3
          },
          {
            "t": 208.8,
            "v": 571.4
          },
          {
            "t": 212.5,
            "v": 571.4
          },
          {
            "t": 216.2,
            "v": 571.4
          },
          {
            "t": 220.8,
            "v": 571.4
          },
          {
            "t": 225.3,
            "v": 573.6
          },
          {
            "t": 229.9,
            "v": 573.7
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
          },
          {
            "t": 232.0,
            "v": -1.0
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
          },
          {
            "t": 232.0,
            "v": -1.0
          }
        ],
        "cswch": [
          {
            "t": 2.0,
            "v": 17.5
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
            "v": 27.5
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
            "v": 2.0
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
            "v": 4.0
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
            "v": 0.5
          }
        ]
      },
      "frps": {
        "cpu": [
          {
            "t": 8.6,
            "v": 22.52
          },
          {
            "t": 14.4,
            "v": 20.41
          },
          {
            "t": 20.1,
            "v": 20.65
          },
          {
            "t": 25.3,
            "v": 1.11
          },
          {
            "t": 28.9,
            "v": 16.08
          },
          {
            "t": 33.7,
            "v": 20.09
          },
          {
            "t": 38.4,
            "v": 20.14
          },
          {
            "t": 43.1,
            "v": 19.82
          },
          {
            "t": 47.8,
            "v": 12.83
          },
          {
            "t": 56.2,
            "v": 37.51
          },
          {
            "t": 64.3,
            "v": 28.74
          }
        ],
        "rss": [
          {
            "t": 3.6,
            "v": 32.1
          },
          {
            "t": 8.6,
            "v": 32.9
          },
          {
            "t": 14.4,
            "v": 42.0
          },
          {
            "t": 20.1,
            "v": 46.6
          },
          {
            "t": 25.3,
            "v": 45.8
          },
          {
            "t": 28.9,
            "v": 41.6
          },
          {
            "t": 33.7,
            "v": 38.8
          },
          {
            "t": 38.4,
            "v": 38.6
          },
          {
            "t": 43.1,
            "v": 38.7
          },
          {
            "t": 47.8,
            "v": 38.7
          },
          {
            "t": 51.4,
            "v": 38.7
          },
          {
            "t": 56.2,
            "v": 74.3
          },
          {
            "t": 64.3,
            "v": 97.6
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
            "v": 27.2
          },
          {
            "t": 2.0,
            "v": 293.0
          },
          {
            "t": 4.0,
            "v": 1357.5
          },
          {
            "t": 6.0,
            "v": 2443.5
          },
          {
            "t": 8.0,
            "v": 913.0
          },
          {
            "t": 10.0,
            "v": 912.0
          },
          {
            "t": 12.0,
            "v": 2024.0
          },
          {
            "t": 14.0,
            "v": 1037.5
          },
          {
            "t": 16.0,
            "v": 1106.5
          },
          {
            "t": 18.0,
            "v": 1604.5
          },
          {
            "t": 20.0,
            "v": 784.1
          },
          {
            "t": 22.0,
            "v": 192.0
          },
          {
            "t": 24.0,
            "v": 0.0
          },
          {
            "t": 26.0,
            "v": 767.0
          },
          {
            "t": 28.0,
            "v": 2484.5
          },
          {
            "t": 30.0,
            "v": 1884.0
          },
          {
            "t": 32.0,
            "v": 2825.5
          },
          {
            "t": 34.0,
            "v": 2155.5
          },
          {
            "t": 36.0,
            "v": 2704.0
          },
          {
            "t": 38.0,
            "v": 2210.5
          },
          {
            "t": 40.0,
            "v": 1305.5
          },
          {
            "t": 42.0,
            "v": 1391.0
          },
          {
            "t": 44.0,
            "v": 1771.0
          },
          {
            "t": 46.0,
            "v": 2245.0
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
            "v": 309.0
          },
          {
            "t": 54.0,
            "v": 255.5
          },
          {
            "t": 56.0,
            "v": 0.0
          },
          {
            "t": 58.0,
            "v": 371.0
          },
          {
            "t": 60.0,
            "v": 262.7
          },
          {
            "t": 62.0,
            "v": 551.5
          },
          {
            "t": 64.0,
            "v": 19.5
          },
          {
            "t": 66.0,
            "v": 263.5
          },
          {
            "t": 68.0,
            "v": 6.0
          },
          {
            "t": 70.0,
            "v": 190.5
          }
        ],
        "nvcswch": [
          {
            "t": 0.0,
            "v": 5.0
          },
          {
            "t": 2.0,
            "v": 98.5
          },
          {
            "t": 4.0,
            "v": 385.0
          },
          {
            "t": 6.0,
            "v": 317.0
          },
          {
            "t": 8.0,
            "v": 243.5
          },
          {
            "t": 10.0,
            "v": 448.5
          },
          {
            "t": 12.0,
            "v": 619.0
          },
          {
            "t": 14.0,
            "v": 481.5
          },
          {
            "t": 16.0,
            "v": 313.5
          },
          {
            "t": 18.0,
            "v": 155.5
          },
          {
            "t": 20.0,
            "v": 322.4
          },
          {
            "t": 22.0,
            "v": 138.0
          },
          {
            "t": 24.0,
            "v": 0.0
          },
          {
            "t": 26.0,
            "v": 60.5
          },
          {
            "t": 28.0,
            "v": 434.5
          },
          {
            "t": 30.0,
            "v": 404.0
          },
          {
            "t": 32.0,
            "v": 334.0
          },
          {
            "t": 34.0,
            "v": 550.0
          },
          {
            "t": 36.0,
            "v": 221.0
          },
          {
            "t": 38.0,
            "v": 497.5
          },
          {
            "t": 40.0,
            "v": 206.0
          },
          {
            "t": 42.0,
            "v": 293.5
          },
          {
            "t": 44.0,
            "v": 401.0
          },
          {
            "t": 46.0,
            "v": 153.5
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
            "v": 142.0
          },
          {
            "t": 54.0,
            "v": 171.0
          },
          {
            "t": 56.0,
            "v": 0.0
          },
          {
            "t": 58.0,
            "v": 257.5
          },
          {
            "t": 60.0,
            "v": 274.1
          },
          {
            "t": 62.0,
            "v": 403.0
          },
          {
            "t": 64.0,
            "v": 2.5
          },
          {
            "t": 66.0,
            "v": 421.0
          },
          {
            "t": 68.0,
            "v": 1.5
          },
          {
            "t": 70.0,
            "v": 139.5
          }
        ]
      },
      "frpc": {
        "cpu": [
          {
            "t": 8.6,
            "v": 17.86
          },
          {
            "t": 14.4,
            "v": 17.4
          },
          {
            "t": 20.1,
            "v": 17.11
          },
          {
            "t": 28.9,
            "v": 10.61
          },
          {
            "t": 33.7,
            "v": 12.36
          },
          {
            "t": 38.4,
            "v": 12.55
          },
          {
            "t": 43.1,
            "v": 12.19
          },
          {
            "t": 47.8,
            "v": 7.5
          },
          {
            "t": 56.2,
            "v": 22.68
          },
          {
            "t": 64.3,
            "v": 20.72
          }
        ],
        "rss": [
          {
            "t": 3.6,
            "v": 26.9
          },
          {
            "t": 8.6,
            "v": 26.9
          },
          {
            "t": 14.4,
            "v": 46.2
          },
          {
            "t": 20.1,
            "v": 44.0
          },
          {
            "t": 25.3,
            "v": 44.0
          },
          {
            "t": 28.9,
            "v": 47.1
          },
          {
            "t": 33.7,
            "v": 48.7
          },
          {
            "t": 38.4,
            "v": 50.3
          },
          {
            "t": 43.1,
            "v": 50.4
          },
          {
            "t": 47.8,
            "v": 50.5
          },
          {
            "t": 51.4,
            "v": 50.5
          },
          {
            "t": 56.2,
            "v": 83.9
          },
          {
            "t": 64.3,
            "v": 126.1
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
            "v": 17.3
          },
          {
            "t": 2.0,
            "v": 1697.0
          },
          {
            "t": 4.0,
            "v": 1920.5
          },
          {
            "t": 6.0,
            "v": 1312.5
          },
          {
            "t": 8.0,
            "v": 740.5
          },
          {
            "t": 10.0,
            "v": 835.5
          },
          {
            "t": 12.0,
            "v": 1559.0
          },
          {
            "t": 14.0,
            "v": 563.0
          },
          {
            "t": 16.0,
            "v": 614.0
          },
          {
            "t": 18.0,
            "v": 1773.5
          },
          {
            "t": 20.0,
            "v": 735.8
          },
          {
            "t": 22.0,
            "v": 138.5
          },
          {
            "t": 24.0,
            "v": 0.5
          },
          {
            "t": 26.0,
            "v": 578.0
          },
          {
            "t": 28.0,
            "v": 2301.0
          },
          {
            "t": 30.0,
            "v": 2245.5
          },
          {
            "t": 32.0,
            "v": 1016.5
          },
          {
            "t": 34.0,
            "v": 1623.0
          },
          {
            "t": 36.0,
            "v": 335.5
          },
          {
            "t": 38.0,
            "v": 2459.5
          },
          {
            "t": 40.0,
            "v": 2412.5
          },
          {
            "t": 42.0,
            "v": 2444.5
          },
          {
            "t": 44.0,
            "v": 2213.5
          },
          {
            "t": 46.0,
            "v": 1737.5
          },
          {
            "t": 48.0,
            "v": 2.5
          },
          {
            "t": 50.0,
            "v": 0.0
          },
          {
            "t": 52.0,
            "v": 349.5
          },
          {
            "t": 54.0,
            "v": 783.5
          },
          {
            "t": 56.0,
            "v": 562.5
          },
          {
            "t": 58.0,
            "v": 519.5
          },
          {
            "t": 60.0,
            "v": 608.0
          },
          {
            "t": 62.0,
            "v": 631.5
          },
          {
            "t": 64.0,
            "v": 536.0
          },
          {
            "t": 66.0,
            "v": 716.0
          },
          {
            "t": 68.0,
            "v": 689.5
          },
          {
            "t": 70.0,
            "v": 767.5
          }
        ],
        "nvcswch": [
          {
            "t": 0.0,
            "v": 2.0
          },
          {
            "t": 2.0,
            "v": 287.0
          },
          {
            "t": 4.0,
            "v": 515.5
          },
          {
            "t": 6.0,
            "v": 201.0
          },
          {
            "t": 8.0,
            "v": 256.0
          },
          {
            "t": 10.0,
            "v": 299.5
          },
          {
            "t": 12.0,
            "v": 457.0
          },
          {
            "t": 14.0,
            "v": 245.5
          },
          {
            "t": 16.0,
            "v": 157.0
          },
          {
            "t": 18.0,
            "v": 170.5
          },
          {
            "t": 20.0,
            "v": 233.3
          },
          {
            "t": 22.0,
            "v": 156.5
          },
          {
            "t": 24.0,
            "v": 0.0
          },
          {
            "t": 26.0,
            "v": 73.0
          },
          {
            "t": 28.0,
            "v": 432.0
          },
          {
            "t": 30.0,
            "v": 491.5
          },
          {
            "t": 32.0,
            "v": 110.0
          },
          {
            "t": 34.0,
            "v": 418.5
          },
          {
            "t": 36.0,
            "v": 79.5
          },
          {
            "t": 38.0,
            "v": 531.0
          },
          {
            "t": 40.0,
            "v": 387.5
          },
          {
            "t": 42.0,
            "v": 358.5
          },
          {
            "t": 44.0,
            "v": 516.0
          },
          {
            "t": 46.0,
            "v": 140.0
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
            "v": 94.5
          },
          {
            "t": 54.0,
            "v": 221.5
          },
          {
            "t": 56.0,
            "v": 228.5
          },
          {
            "t": 58.0,
            "v": 204.0
          },
          {
            "t": 60.0,
            "v": 240.3
          },
          {
            "t": 62.0,
            "v": 307.0
          },
          {
            "t": 64.0,
            "v": 228.0
          },
          {
            "t": 66.0,
            "v": 377.5
          },
          {
            "t": 68.0,
            "v": 383.1
          },
          {
            "t": 70.0,
            "v": 269.5
          }
        ]
      },
      "other": {
        "cpu": [
          {
            "t": 3.6,
            "v": 3.85
          },
          {
            "t": 8.6,
            "v": 2.78
          },
          {
            "t": 14.4,
            "v": 20.5
          },
          {
            "t": 20.1,
            "v": 3.06
          },
          {
            "t": 25.3,
            "v": 6.84
          },
          {
            "t": 28.9,
            "v": 2.26
          },
          {
            "t": 33.7,
            "v": 2.26
          },
          {
            "t": 38.4,
            "v": 2.06
          },
          {
            "t": 43.1,
            "v": 2.07
          },
          {
            "t": 47.8,
            "v": 1.7
          },
          {
            "t": 51.4,
            "v": 1.66
          },
          {
            "t": 56.2,
            "v": 2.3
          },
          {
            "t": 64.3,
            "v": 2.01
          },
          {
            "t": 73.4,
            "v": 1.81
          },
          {
            "t": 77.3,
            "v": 2.07
          },
          {
            "t": 80.9,
            "v": 1.85
          },
          {
            "t": 84.6,
            "v": 1.81
          },
          {
            "t": 88.4,
            "v": 1.68
          },
          {
            "t": 92.1,
            "v": 1.75
          },
          {
            "t": 95.8,
            "v": 1.75
          },
          {
            "t": 99.5,
            "v": 1.74
          },
          {
            "t": 103.3,
            "v": 1.94
          },
          {
            "t": 106.9,
            "v": 1.72
          },
          {
            "t": 110.5,
            "v": 1.6
          },
          {
            "t": 114.2,
            "v": 1.91
          },
          {
            "t": 117.9,
            "v": 1.69
          },
          {
            "t": 121.6,
            "v": 1.73
          },
          {
            "t": 125.3,
            "v": 1.47
          },
          {
            "t": 129.1,
            "v": 1.67
          },
          {
            "t": 132.8,
            "v": 1.88
          },
          {
            "t": 136.4,
            "v": 2.14
          },
          {
            "t": 140.0,
            "v": 2.15
          },
          {
            "t": 144.5,
            "v": 1.85
          },
          {
            "t": 149.0,
            "v": 1.83
          },
          {
            "t": 153.4,
            "v": 1.85
          },
          {
            "t": 157.9,
            "v": 1.9
          },
          {
            "t": 161.8,
            "v": 1.66
          },
          {
            "t": 165.5,
            "v": 2.27
          },
          {
            "t": 169.9,
            "v": 1.96
          },
          {
            "t": 174.5,
            "v": 2.1
          },
          {
            "t": 178.9,
            "v": 2.0
          },
          {
            "t": 183.4,
            "v": 1.92
          },
          {
            "t": 187.1,
            "v": 1.59
          },
          {
            "t": 190.8,
            "v": 1.86
          },
          {
            "t": 195.3,
            "v": 2.34
          },
          {
            "t": 199.8,
            "v": 2.32
          },
          {
            "t": 204.3,
            "v": 1.93
          },
          {
            "t": 208.8,
            "v": 1.72
          },
          {
            "t": 212.5,
            "v": 1.58
          },
          {
            "t": 216.2,
            "v": 2.25
          },
          {
            "t": 220.8,
            "v": 2.04
          },
          {
            "t": 225.3,
            "v": 2.09
          },
          {
            "t": 229.9,
            "v": 2.13
          },
          {
            "t": 234.5,
            "v": 2.46
          }
        ],
        "rss": [
          {
            "t": 3.6,
            "v": 130.7
          },
          {
            "t": 8.6,
            "v": 130.8
          },
          {
            "t": 14.4,
            "v": 130.8
          },
          {
            "t": 20.1,
            "v": 130.8
          },
          {
            "t": 25.3,
            "v": 130.7
          },
          {
            "t": 28.9,
            "v": 130.7
          },
          {
            "t": 33.7,
            "v": 130.7
          },
          {
            "t": 38.4,
            "v": 130.7
          },
          {
            "t": 43.1,
            "v": 130.7
          },
          {
            "t": 47.8,
            "v": 130.7
          },
          {
            "t": 51.4,
            "v": 130.7
          },
          {
            "t": 56.2,
            "v": 130.7
          },
          {
            "t": 64.3,
            "v": 131.3
          },
          {
            "t": 73.4,
            "v": 134.3
          },
          {
            "t": 77.3,
            "v": 134.5
          },
          {
            "t": 80.9,
            "v": 134.5
          },
          {
            "t": 84.6,
            "v": 134.5
          },
          {
            "t": 88.4,
            "v": 134.5
          },
          {
            "t": 92.1,
            "v": 134.5
          },
          {
            "t": 95.8,
            "v": 134.4
          },
          {
            "t": 99.5,
            "v": 134.4
          },
          {
            "t": 103.3,
            "v": 134.4
          },
          {
            "t": 106.9,
            "v": 134.6
          },
          {
            "t": 110.5,
            "v": 134.6
          },
          {
            "t": 114.2,
            "v": 134.6
          },
          {
            "t": 117.9,
            "v": 134.6
          },
          {
            "t": 121.6,
            "v": 134.6
          },
          {
            "t": 125.3,
            "v": 134.6
          },
          {
            "t": 129.1,
            "v": 134.6
          },
          {
            "t": 132.8,
            "v": 134.6
          },
          {
            "t": 136.4,
            "v": 134.6
          },
          {
            "t": 140.0,
            "v": 135.1
          },
          {
            "t": 144.5,
            "v": 135.1
          },
          {
            "t": 149.0,
            "v": 135.1
          },
          {
            "t": 153.4,
            "v": 135.1
          },
          {
            "t": 157.9,
            "v": 135.1
          },
          {
            "t": 161.8,
            "v": 135.1
          },
          {
            "t": 165.5,
            "v": 135.1
          },
          {
            "t": 169.9,
            "v": 135.1
          },
          {
            "t": 174.5,
            "v": 135.1
          },
          {
            "t": 178.9,
            "v": 135.2
          },
          {
            "t": 183.4,
            "v": 135.2
          },
          {
            "t": 187.1,
            "v": 135.2
          },
          {
            "t": 190.8,
            "v": 135.2
          },
          {
            "t": 195.3,
            "v": 135.2
          },
          {
            "t": 199.8,
            "v": 135.2
          },
          {
            "t": 204.3,
            "v": 135.2
          },
          {
            "t": 208.8,
            "v": 135.2
          },
          {
            "t": 212.5,
            "v": 135.2
          },
          {
            "t": 216.2,
            "v": 135.2
          },
          {
            "t": 220.8,
            "v": 135.2
          },
          {
            "t": 225.3,
            "v": 135.2
          },
          {
            "t": 229.9,
            "v": 135.2
          },
          {
            "t": 234.5,
            "v": 135.4
          }
        ],
        "read_kbs": [
          {
            "t": 0.0,
            "v": 128.1
          },
          {
            "t": 2.0,
            "v": -165.0
          },
          {
            "t": 4.0,
            "v": -156.0
          },
          {
            "t": 6.0,
            "v": -156.0
          },
          {
            "t": 8.0,
            "v": -156.0
          },
          {
            "t": 10.0,
            "v": -156.0
          },
          {
            "t": 12.0,
            "v": -156.0
          },
          {
            "t": 14.0,
            "v": -156.0
          },
          {
            "t": 16.0,
            "v": -156.0
          },
          {
            "t": 18.0,
            "v": -156.0
          },
          {
            "t": 20.0,
            "v": -156.0
          },
          {
            "t": 22.0,
            "v": -156.0
          },
          {
            "t": 24.0,
            "v": -156.0
          },
          {
            "t": 26.0,
            "v": -156.0
          },
          {
            "t": 28.0,
            "v": -156.0
          },
          {
            "t": 30.0,
            "v": -156.0
          },
          {
            "t": 32.0,
            "v": -156.0
          },
          {
            "t": 34.0,
            "v": -156.0
          },
          {
            "t": 36.0,
            "v": -156.0
          },
          {
            "t": 38.0,
            "v": -156.0
          },
          {
            "t": 40.0,
            "v": -156.0
          },
          {
            "t": 42.0,
            "v": -156.0
          },
          {
            "t": 44.0,
            "v": -156.0
          },
          {
            "t": 46.0,
            "v": -156.0
          },
          {
            "t": 48.0,
            "v": -156.0
          },
          {
            "t": 50.0,
            "v": -156.0
          },
          {
            "t": 52.0,
            "v": -156.0
          },
          {
            "t": 54.0,
            "v": -156.0
          },
          {
            "t": 56.0,
            "v": -156.0
          },
          {
            "t": 58.0,
            "v": -156.0
          },
          {
            "t": 60.0,
            "v": -156.0
          },
          {
            "t": 62.0,
            "v": -156.0
          },
          {
            "t": 64.0,
            "v": -156.0
          },
          {
            "t": 66.0,
            "v": -156.0
          },
          {
            "t": 68.0,
            "v": -156.0
          },
          {
            "t": 70.0,
            "v": -156.0
          },
          {
            "t": 72.0,
            "v": -156.0
          },
          {
            "t": 74.0,
            "v": -156.0
          },
          {
            "t": 76.0,
            "v": -156.0
          },
          {
            "t": 78.0,
            "v": -156.0
          },
          {
            "t": 80.0,
            "v": -156.0
          },
          {
            "t": 82.0,
            "v": -156.0
          },
          {
            "t": 84.0,
            "v": -156.0
          },
          {
            "t": 86.0,
            "v": -156.0
          },
          {
            "t": 88.0,
            "v": -156.0
          },
          {
            "t": 90.0,
            "v": -156.0
          },
          {
            "t": 92.0,
            "v": -156.0
          },
          {
            "t": 94.0,
            "v": -156.0
          },
          {
            "t": 96.0,
            "v": -156.0
          },
          {
            "t": 98.0,
            "v": -156.0
          },
          {
            "t": 100.0,
            "v": -156.0
          },
          {
            "t": 102.0,
            "v": -156.0
          },
          {
            "t": 104.0,
            "v": -156.0
          },
          {
            "t": 106.0,
            "v": -156.0
          },
          {
            "t": 108.0,
            "v": -156.0
          },
          {
            "t": 110.0,
            "v": -156.0
          },
          {
            "t": 112.0,
            "v": -156.0
          },
          {
            "t": 114.0,
            "v": -156.0
          },
          {
            "t": 116.0,
            "v": -156.0
          },
          {
            "t": 118.0,
            "v": -156.0
          },
          {
            "t": 120.0,
            "v": -156.0
          },
          {
            "t": 122.0,
            "v": -156.0
          },
          {
            "t": 124.0,
            "v": -156.0
          },
          {
            "t": 126.0,
            "v": -156.0
          },
          {
            "t": 128.0,
            "v": -156.0
          },
          {
            "t": 130.0,
            "v": -156.0
          },
          {
            "t": 132.0,
            "v": -156.0
          },
          {
            "t": 134.0,
            "v": -156.0
          },
          {
            "t": 136.0,
            "v": -156.0
          },
          {
            "t": 138.0,
            "v": -156.0
          },
          {
            "t": 140.0,
            "v": -156.0
          },
          {
            "t": 142.0,
            "v": -156.0
          },
          {
            "t": 144.0,
            "v": -156.0
          },
          {
            "t": 146.0,
            "v": -156.0
          },
          {
            "t": 148.0,
            "v": -156.0
          },
          {
            "t": 150.0,
            "v": -156.0
          },
          {
            "t": 152.0,
            "v": -156.0
          },
          {
            "t": 154.0,
            "v": -156.0
          },
          {
            "t": 156.0,
            "v": -156.0
          },
          {
            "t": 158.0,
            "v": -156.0
          },
          {
            "t": 160.0,
            "v": -156.0
          },
          {
            "t": 162.0,
            "v": -156.0
          },
          {
            "t": 164.0,
            "v": -156.0
          },
          {
            "t": 166.0,
            "v": -156.0
          },
          {
            "t": 168.0,
            "v": -156.0
          },
          {
            "t": 170.0,
            "v": -156.0
          },
          {
            "t": 172.0,
            "v": -156.0
          },
          {
            "t": 174.0,
            "v": -156.0
          },
          {
            "t": 176.0,
            "v": -156.0
          },
          {
            "t": 178.0,
            "v": -156.0
          },
          {
            "t": 180.0,
            "v": -156.0
          },
          {
            "t": 182.0,
            "v": -156.0
          },
          {
            "t": 184.0,
            "v": -156.0
          },
          {
            "t": 186.0,
            "v": -156.0
          },
          {
            "t": 188.0,
            "v": -156.0
          },
          {
            "t": 190.0,
            "v": -156.0
          },
          {
            "t": 192.0,
            "v": -156.0
          },
          {
            "t": 194.0,
            "v": -156.0
          },
          {
            "t": 196.0,
            "v": -156.0
          },
          {
            "t": 198.0,
            "v": -156.0
          },
          {
            "t": 200.0,
            "v": -156.0
          },
          {
            "t": 202.0,
            "v": -156.0
          },
          {
            "t": 204.0,
            "v": -156.0
          },
          {
            "t": 206.0,
            "v": -156.0
          },
          {
            "t": 208.0,
            "v": -156.0
          },
          {
            "t": 210.0,
            "v": -156.0
          },
          {
            "t": 212.0,
            "v": -156.0
          },
          {
            "t": 214.0,
            "v": -156.0
          },
          {
            "t": 216.0,
            "v": -156.0
          },
          {
            "t": 218.0,
            "v": -156.0
          },
          {
            "t": 220.0,
            "v": -156.0
          },
          {
            "t": 222.0,
            "v": -156.0
          },
          {
            "t": 224.0,
            "v": -156.0
          },
          {
            "t": 226.0,
            "v": -156.0
          },
          {
            "t": 228.0,
            "v": -156.0
          },
          {
            "t": 230.0,
            "v": -156.0
          },
          {
            "t": 232.0,
            "v": -156.0
          }
        ],
        "write_kbs": [
          {
            "t": 0.0,
            "v": 38484.5
          },
          {
            "t": 2.0,
            "v": -149.0
          },
          {
            "t": 4.0,
            "v": -134.0
          },
          {
            "t": 6.0,
            "v": -138.0
          },
          {
            "t": 8.0,
            "v": -136.0
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
            "v": -134.0
          },
          {
            "t": 16.0,
            "v": -136.0
          },
          {
            "t": 18.0,
            "v": -142.0
          },
          {
            "t": 20.0,
            "v": -136.1
          },
          {
            "t": 22.0,
            "v": -136.0
          },
          {
            "t": 24.0,
            "v": -136.0
          },
          {
            "t": 26.0,
            "v": -144.0
          },
          {
            "t": 28.0,
            "v": -134.0
          },
          {
            "t": 30.0,
            "v": -136.0
          },
          {
            "t": 32.0,
            "v": -128.0
          },
          {
            "t": 34.0,
            "v": -136.0
          },
          {
            "t": 36.0,
            "v": -138.0
          },
          {
            "t": 38.0,
            "v": -140.0
          },
          {
            "t": 40.0,
            "v": -140.0
          },
          {
            "t": 42.0,
            "v": -142.0
          },
          {
            "t": 44.0,
            "v": -134.0
          },
          {
            "t": 46.0,
            "v": -134.0
          },
          {
            "t": 48.0,
            "v": -138.0
          },
          {
            "t": 50.0,
            "v": -142.0
          },
          {
            "t": 52.0,
            "v": -134.0
          },
          {
            "t": 54.0,
            "v": -144.0
          },
          {
            "t": 56.0,
            "v": -142.0
          },
          {
            "t": 58.0,
            "v": -140.1
          },
          {
            "t": 60.0,
            "v": -134.0
          },
          {
            "t": 62.0,
            "v": -136.0
          },
          {
            "t": 64.0,
            "v": -144.0
          },
          {
            "t": 66.0,
            "v": -134.0
          },
          {
            "t": 68.0,
            "v": -132.1
          },
          {
            "t": 70.0,
            "v": -130.0
          },
          {
            "t": 72.0,
            "v": -84.0
          },
          {
            "t": 74.0,
            "v": -136.0
          },
          {
            "t": 76.0,
            "v": -128.0
          },
          {
            "t": 78.0,
            "v": -140.0
          },
          {
            "t": 80.0,
            "v": -130.0
          },
          {
            "t": 82.0,
            "v": -126.0
          },
          {
            "t": 84.0,
            "v": -132.0
          },
          {
            "t": 86.0,
            "v": -136.0
          },
          {
            "t": 88.0,
            "v": -128.0
          },
          {
            "t": 90.0,
            "v": -138.0
          },
          {
            "t": 92.0,
            "v": -128.0
          },
          {
            "t": 94.0,
            "v": -132.0
          },
          {
            "t": 96.0,
            "v": -134.0
          },
          {
            "t": 98.0,
            "v": -134.0
          },
          {
            "t": 100.0,
            "v": -132.0
          },
          {
            "t": 102.0,
            "v": -120.0
          },
          {
            "t": 104.0,
            "v": -122.0
          },
          {
            "t": 106.0,
            "v": -134.0
          },
          {
            "t": 108.0,
            "v": -142.0
          },
          {
            "t": 110.0,
            "v": -120.0
          },
          {
            "t": 112.0,
            "v": -144.0
          },
          {
            "t": 114.0,
            "v": -126.0
          },
          {
            "t": 116.0,
            "v": -136.0
          },
          {
            "t": 118.0,
            "v": -130.0
          },
          {
            "t": 120.0,
            "v": -136.0
          },
          {
            "t": 122.0,
            "v": -134.0
          },
          {
            "t": 124.0,
            "v": -134.0
          },
          {
            "t": 126.0,
            "v": -126.0
          },
          {
            "t": 128.0,
            "v": -126.0
          },
          {
            "t": 130.0,
            "v": -134.0
          },
          {
            "t": 132.0,
            "v": -132.0
          },
          {
            "t": 134.0,
            "v": -132.0
          },
          {
            "t": 136.0,
            "v": -130.0
          },
          {
            "t": 138.0,
            "v": -126.0
          },
          {
            "t": 140.0,
            "v": -134.1
          },
          {
            "t": 142.0,
            "v": -132.0
          },
          {
            "t": 144.0,
            "v": -130.0
          },
          {
            "t": 146.0,
            "v": -136.0
          },
          {
            "t": 148.0,
            "v": -128.0
          },
          {
            "t": 150.0,
            "v": -132.0
          },
          {
            "t": 152.0,
            "v": -128.0
          },
          {
            "t": 154.0,
            "v": -136.0
          },
          {
            "t": 156.0,
            "v": -134.0
          },
          {
            "t": 158.0,
            "v": -126.0
          },
          {
            "t": 160.0,
            "v": -136.0
          },
          {
            "t": 162.0,
            "v": -134.0
          },
          {
            "t": 164.0,
            "v": -122.0
          },
          {
            "t": 166.0,
            "v": -138.0
          },
          {
            "t": 168.0,
            "v": -132.0
          },
          {
            "t": 170.0,
            "v": -122.0
          },
          {
            "t": 172.0,
            "v": -134.0
          },
          {
            "t": 174.0,
            "v": -130.0
          },
          {
            "t": 176.0,
            "v": -124.0
          },
          {
            "t": 178.0,
            "v": -138.0
          },
          {
            "t": 180.0,
            "v": -132.0
          },
          {
            "t": 182.0,
            "v": -132.0
          },
          {
            "t": 184.0,
            "v": -130.0
          },
          {
            "t": 186.0,
            "v": -132.0
          },
          {
            "t": 188.0,
            "v": -134.0
          },
          {
            "t": 190.0,
            "v": -132.0
          },
          {
            "t": 192.0,
            "v": -130.0
          },
          {
            "t": 194.0,
            "v": -128.0
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
            "v": -138.0
          },
          {
            "t": 202.0,
            "v": -126.0
          },
          {
            "t": 204.0,
            "v": -132.0
          },
          {
            "t": 206.0,
            "v": -134.0
          },
          {
            "t": 208.0,
            "v": -126.0
          },
          {
            "t": 210.0,
            "v": -128.0
          },
          {
            "t": 212.0,
            "v": -124.0
          },
          {
            "t": 214.0,
            "v": -136.0
          },
          {
            "t": 216.0,
            "v": -130.0
          },
          {
            "t": 218.0,
            "v": -132.0
          },
          {
            "t": 220.0,
            "v": -134.0
          },
          {
            "t": 222.0,
            "v": -124.0
          },
          {
            "t": 224.0,
            "v": -134.0
          },
          {
            "t": 226.0,
            "v": -128.0
          },
          {
            "t": 228.0,
            "v": -136.0
          },
          {
            "t": 230.0,
            "v": -130.0
          },
          {
            "t": 232.0,
            "v": -110.0
          }
        ],
        "cswch": [
          {
            "t": 0.0,
            "v": 1906.0
          },
          {
            "t": 2.0,
            "v": 510.5
          },
          {
            "t": 4.0,
            "v": 1466.0
          },
          {
            "t": 6.0,
            "v": 551.5
          },
          {
            "t": 8.0,
            "v": 1004.0
          },
          {
            "t": 10.0,
            "v": 1290.5
          },
          {
            "t": 12.0,
            "v": 499.0
          },
          {
            "t": 14.0,
            "v": 1443.0
          },
          {
            "t": 16.0,
            "v": 1504.5
          },
          {
            "t": 18.0,
            "v": 257.0
          },
          {
            "t": 20.0,
            "v": 1288.6
          },
          {
            "t": 22.0,
            "v": 937.0
          },
          {
            "t": 24.0,
            "v": 734.0
          },
          {
            "t": 26.0,
            "v": 803.0
          },
          {
            "t": 28.0,
            "v": 896.0
          },
          {
            "t": 30.0,
            "v": 1268.0
          },
          {
            "t": 32.0,
            "v": 393.5
          },
          {
            "t": 34.0,
            "v": 1502.0
          },
          {
            "t": 36.0,
            "v": 373.0
          },
          {
            "t": 38.0,
            "v": 1255.0
          },
          {
            "t": 40.0,
            "v": 845.5
          },
          {
            "t": 42.0,
            "v": 801.0
          },
          {
            "t": 44.0,
            "v": 1384.5
          },
          {
            "t": 46.0,
            "v": 261.5
          },
          {
            "t": 48.0,
            "v": 1160.5
          },
          {
            "t": 50.0,
            "v": 537.0
          },
          {
            "t": 52.0,
            "v": 1263.0
          },
          {
            "t": 54.0,
            "v": 430.0
          },
          {
            "t": 56.0,
            "v": 1110.5
          },
          {
            "t": 58.0,
            "v": 1238.5
          },
          {
            "t": 60.0,
            "v": 1299.1
          },
          {
            "t": 62.0,
            "v": 823.5
          },
          {
            "t": 64.0,
            "v": 1243.5
          },
          {
            "t": 66.0,
            "v": 1296.0
          },
          {
            "t": 68.0,
            "v": 1147.8
          },
          {
            "t": 70.0,
            "v": 1109.0
          },
          {
            "t": 72.0,
            "v": 837.5
          },
          {
            "t": 74.0,
            "v": 1346.5
          },
          {
            "t": 76.0,
            "v": 741.5
          },
          {
            "t": 78.0,
            "v": 782.0
          },
          {
            "t": 80.0,
            "v": 990.5
          },
          {
            "t": 82.0,
            "v": 561.0
          },
          {
            "t": 84.0,
            "v": 1233.0
          },
          {
            "t": 86.0,
            "v": 363.5
          },
          {
            "t": 88.0,
            "v": 1402.5
          },
          {
            "t": 90.0,
            "v": 134.0
          },
          {
            "t": 92.0,
            "v": 1434.5
          },
          {
            "t": 94.0,
            "v": 329.0
          },
          {
            "t": 96.0,
            "v": 1250.0
          },
          {
            "t": 98.0,
            "v": 508.0
          },
          {
            "t": 100.0,
            "v": 986.5
          },
          {
            "t": 102.0,
            "v": 710.5
          },
          {
            "t": 104.0,
            "v": 725.5
          },
          {
            "t": 106.0,
            "v": 964.0
          },
          {
            "t": 108.0,
            "v": 450.0
          },
          {
            "t": 110.0,
            "v": 1281.0
          },
          {
            "t": 112.0,
            "v": 200.5
          },
          {
            "t": 114.0,
            "v": 1432.5
          },
          {
            "t": 116.0,
            "v": 290.0
          },
          {
            "t": 118.0,
            "v": 1329.5
          },
          {
            "t": 120.0,
            "v": 486.5
          },
          {
            "t": 122.0,
            "v": 1099.0
          },
          {
            "t": 124.0,
            "v": 686.5
          },
          {
            "t": 126.0,
            "v": 891.0
          },
          {
            "t": 128.0,
            "v": 888.0
          },
          {
            "t": 130.0,
            "v": 662.5
          },
          {
            "t": 132.0,
            "v": 1061.0
          },
          {
            "t": 134.0,
            "v": 705.5
          },
          {
            "t": 136.0,
            "v": 1313.0
          },
          {
            "t": 138.0,
            "v": 135.5
          },
          {
            "t": 140.0,
            "v": 1559.5
          },
          {
            "t": 142.0,
            "v": 469.0
          },
          {
            "t": 144.0,
            "v": 1227.5
          },
          {
            "t": 146.0,
            "v": 759.0
          },
          {
            "t": 148.0,
            "v": 917.5
          },
          {
            "t": 150.0,
            "v": 1079.0
          },
          {
            "t": 152.0,
            "v": 548.5
          },
          {
            "t": 154.0,
            "v": 1412.0
          },
          {
            "t": 156.0,
            "v": 218.5
          },
          {
            "t": 158.0,
            "v": 1423.0
          },
          {
            "t": 160.0,
            "v": 263.5
          },
          {
            "t": 162.0,
            "v": 1183.5
          },
          {
            "t": 164.0,
            "v": 537.0
          },
          {
            "t": 166.0,
            "v": 1432.9
          },
          {
            "t": 168.0,
            "v": 191.0
          },
          {
            "t": 170.0,
            "v": 1525.5
          },
          {
            "t": 172.0,
            "v": 336.0
          },
          {
            "t": 174.0,
            "v": 1232.0
          },
          {
            "t": 176.0,
            "v": 765.0
          },
          {
            "t": 178.0,
            "v": 921.5
          },
          {
            "t": 180.0,
            "v": 1041.0
          },
          {
            "t": 182.0,
            "v": 608.5
          },
          {
            "t": 184.0,
            "v": 894.0
          },
          {
            "t": 186.0,
            "v": 781.5
          },
          {
            "t": 188.0,
            "v": 643.0
          },
          {
            "t": 190.0,
            "v": 1016.0
          },
          {
            "t": 192.0,
            "v": 975.0
          },
          {
            "t": 194.0,
            "v": 988.0
          },
          {
            "t": 196.0,
            "v": 1356.0
          },
          {
            "t": 198.0,
            "v": 330.5
          },
          {
            "t": 200.0,
            "v": 1621.0
          },
          {
            "t": 202.0,
            "v": 279.0
          },
          {
            "t": 204.0,
            "v": 1343.0
          },
          {
            "t": 206.0,
            "v": 663.0
          },
          {
            "t": 208.0,
            "v": 1030.5
          },
          {
            "t": 210.0,
            "v": 392.0
          },
          {
            "t": 212.0,
            "v": 1308.5
          },
          {
            "t": 214.0,
            "v": 227.0
          },
          {
            "t": 216.0,
            "v": 1408.0
          },
          {
            "t": 218.0,
            "v": 605.5
          },
          {
            "t": 220.0,
            "v": 1010.5
          },
          {
            "t": 222.0,
            "v": 983.5
          },
          {
            "t": 224.0,
            "v": 593.0
          },
          {
            "t": 226.0,
            "v": 1393.5
          },
          {
            "t": 228.0,
            "v": 230.5
          },
          {
            "t": 230.0,
            "v": 1468.0
          },
          {
            "t": 232.0,
            "v": 379.0
          }
        ],
        "nvcswch": [
          {
            "t": 0.0,
            "v": 88.2
          },
          {
            "t": 2.0,
            "v": 63.0
          },
          {
            "t": 4.0,
            "v": 235.5
          },
          {
            "t": 6.0,
            "v": 108.0
          },
          {
            "t": 8.0,
            "v": 175.0
          },
          {
            "t": 10.0,
            "v": 266.0
          },
          {
            "t": 12.0,
            "v": 101.5
          },
          {
            "t": 14.0,
            "v": 368.5
          },
          {
            "t": 16.0,
            "v": 236.5
          },
          {
            "t": 18.0,
            "v": 38.5
          },
          {
            "t": 20.0,
            "v": 185.1
          },
          {
            "t": 22.0,
            "v": 70.5
          },
          {
            "t": 24.0,
            "v": 9.0
          },
          {
            "t": 26.0,
            "v": 21.0
          },
          {
            "t": 28.0,
            "v": 150.5
          },
          {
            "t": 30.0,
            "v": 225.0
          },
          {
            "t": 32.0,
            "v": 109.5
          },
          {
            "t": 34.0,
            "v": 275.5
          },
          {
            "t": 36.0,
            "v": 107.5
          },
          {
            "t": 38.0,
            "v": 197.0
          },
          {
            "t": 40.0,
            "v": 140.5
          },
          {
            "t": 42.0,
            "v": 140.0
          },
          {
            "t": 44.0,
            "v": 239.0
          },
          {
            "t": 46.0,
            "v": 24.0
          },
          {
            "t": 48.0,
            "v": 15.0
          },
          {
            "t": 50.0,
            "v": 10.5
          },
          {
            "t": 52.0,
            "v": 99.0
          },
          {
            "t": 54.0,
            "v": 94.0
          },
          {
            "t": 56.0,
            "v": 178.5
          },
          {
            "t": 58.0,
            "v": 106.5
          },
          {
            "t": 60.0,
            "v": 120.9
          },
          {
            "t": 62.0,
            "v": 55.5
          },
          {
            "t": 64.0,
            "v": 138.0
          },
          {
            "t": 66.0,
            "v": 209.5
          },
          {
            "t": 68.0,
            "v": 137.3
          },
          {
            "t": 70.0,
            "v": 86.5
          },
          {
            "t": 72.0,
            "v": 27.0
          },
          {
            "t": 74.0,
            "v": 128.5
          },
          {
            "t": 76.0,
            "v": 12.0
          },
          {
            "t": 78.0,
            "v": 9.5
          },
          {
            "t": 80.0,
            "v": 29.0
          },
          {
            "t": 82.0,
            "v": 17.0
          },
          {
            "t": 84.0,
            "v": 24.5
          },
          {
            "t": 86.0,
            "v": 12.0
          },
          {
            "t": 88.0,
            "v": 32.5
          },
          {
            "t": 90.0,
            "v": 5.0
          },
          {
            "t": 92.0,
            "v": 33.0
          },
          {
            "t": 94.0,
            "v": 8.5
          },
          {
            "t": 96.0,
            "v": 25.5
          },
          {
            "t": 98.0,
            "v": 10.5
          },
          {
            "t": 100.0,
            "v": 15.0
          },
          {
            "t": 102.0,
            "v": 7.0
          },
          {
            "t": 104.0,
            "v": 9.5
          },
          {
            "t": 106.0,
            "v": 6.5
          },
          {
            "t": 108.0,
            "v": 11.5
          },
          {
            "t": 110.0,
            "v": 21.5
          },
          {
            "t": 112.0,
            "v": 14.0
          },
          {
            "t": 114.0,
            "v": 40.0
          },
          {
            "t": 116.0,
            "v": 16.0
          },
          {
            "t": 118.0,
            "v": 31.5
          },
          {
            "t": 120.0,
            "v": 14.5
          },
          {
            "t": 122.0,
            "v": 28.5
          },
          {
            "t": 124.0,
            "v": 17.0
          },
          {
            "t": 126.0,
            "v": 19.0
          },
          {
            "t": 128.0,
            "v": 19.0
          },
          {
            "t": 130.0,
            "v": 10.5
          },
          {
            "t": 132.0,
            "v": 10.5
          },
          {
            "t": 134.0,
            "v": 39.0
          },
          {
            "t": 136.0,
            "v": 23.5
          },
          {
            "t": 138.0,
            "v": 20.5
          },
          {
            "t": 140.0,
            "v": 188.0
          },
          {
            "t": 142.0,
            "v": 50.0
          },
          {
            "t": 144.0,
            "v": 124.5
          },
          {
            "t": 146.0,
            "v": 93.5
          },
          {
            "t": 148.0,
            "v": 100.5
          },
          {
            "t": 150.0,
            "v": 123.0
          },
          {
            "t": 152.0,
            "v": 76.5
          },
          {
            "t": 154.0,
            "v": 154.5
          },
          {
            "t": 156.0,
            "v": 38.5
          },
          {
            "t": 158.0,
            "v": 99.5
          },
          {
            "t": 160.0,
            "v": 5.0
          },
          {
            "t": 162.0,
            "v": 24.5
          },
          {
            "t": 164.0,
            "v": 58.5
          },
          {
            "t": 166.0,
            "v": 190.1
          },
          {
            "t": 168.0,
            "v": 66.0
          },
          {
            "t": 170.0,
            "v": 193.5
          },
          {
            "t": 172.0,
            "v": 64.5
          },
          {
            "t": 174.0,
            "v": 142.0
          },
          {
            "t": 176.0,
            "v": 113.5
          },
          {
            "t": 178.0,
            "v": 102.0
          },
          {
            "t": 180.0,
            "v": 117.5
          },
          {
            "t": 182.0,
            "v": 50.5
          },
          {
            "t": 184.0,
            "v": 15.5
          },
          {
            "t": 186.0,
            "v": 11.5
          },
          {
            "t": 188.0,
            "v": 27.5
          },
          {
            "t": 190.0,
            "v": 126.5
          },
          {
            "t": 192.0,
            "v": 128.5
          },
          {
            "t": 194.0,
            "v": 194.5
          },
          {
            "t": 196.0,
            "v": 174.0
          },
          {
            "t": 198.0,
            "v": 70.0
          },
          {
            "t": 200.0,
            "v": 184.0
          },
          {
            "t": 202.0,
            "v": 47.0
          },
          {
            "t": 204.0,
            "v": 154.0
          },
          {
            "t": 206.0,
            "v": 71.0
          },
          {
            "t": 208.0,
            "v": 33.5
          },
          {
            "t": 210.0,
            "v": 9.0
          },
          {
            "t": 212.0,
            "v": 44.5
          },
          {
            "t": 214.0,
            "v": 45.0
          },
          {
            "t": 216.0,
            "v": 161.5
          },
          {
            "t": 218.0,
            "v": 98.5
          },
          {
            "t": 220.0,
            "v": 115.0
          },
          {
            "t": 222.0,
            "v": 130.0
          },
          {
            "t": 224.0,
            "v": 94.5
          },
          {
            "t": 226.0,
            "v": 183.5
          },
          {
            "t": 228.0,
            "v": 42.5
          },
          {
            "t": 230.0,
            "v": 180.0
          },
          {
            "t": 232.0,
            "v": 42.5
          }
        ]
      }
    },
    "system": {
      "cpu": [
        {
          "t": 0.0,
          "v": 43.7
        },
        {
          "t": 2.0,
          "v": 51.9
        },
        {
          "t": 4.0,
          "v": 84.4
        },
        {
          "t": 6.0,
          "v": 66.9
        },
        {
          "t": 8.0,
          "v": 81.2
        },
        {
          "t": 10.0,
          "v": 93.2
        },
        {
          "t": 12.0,
          "v": 83.5
        },
        {
          "t": 14.0,
          "v": 93.6
        },
        {
          "t": 16.0,
          "v": 85.6
        },
        {
          "t": 18.0,
          "v": 65.5
        },
        {
          "t": 20.0,
          "v": 91.0
        },
        {
          "t": 22.0,
          "v": 42.6
        },
        {
          "t": 24.0,
          "v": 13.8
        },
        {
          "t": 26.0,
          "v": 25.5
        },
        {
          "t": 28.0,
          "v": 72.0
        },
        {
          "t": 30.0,
          "v": 80.6
        },
        {
          "t": 32.0,
          "v": 63.2
        },
        {
          "t": 34.0,
          "v": 81.9
        },
        {
          "t": 36.0,
          "v": 61.7
        },
        {
          "t": 38.0,
          "v": 77.6
        },
        {
          "t": 40.0,
          "v": 72.8
        },
        {
          "t": 42.0,
          "v": 68.5
        },
        {
          "t": 44.0,
          "v": 80.2
        },
        {
          "t": 46.0,
          "v": 42.7
        },
        {
          "t": 48.0,
          "v": 21.1
        },
        {
          "t": 50.0,
          "v": 8.2
        },
        {
          "t": 52.0,
          "v": 79.8
        },
        {
          "t": 54.0,
          "v": 93.9
        },
        {
          "t": 56.0,
          "v": 98.1
        },
        {
          "t": 58.0,
          "v": 98.6
        },
        {
          "t": 60.0,
          "v": 98.9
        },
        {
          "t": 62.0,
          "v": 98.6
        },
        {
          "t": 64.0,
          "v": 99.1
        },
        {
          "t": 66.0,
          "v": 98.6
        },
        {
          "t": 68.0,
          "v": 99.1
        },
        {
          "t": 70.0,
          "v": 98.7
        },
        {
          "t": 72.0,
          "v": 74.3
        },
        {
          "t": 74.0,
          "v": 20.1
        },
        {
          "t": 76.0,
          "v": 12.9
        },
        {
          "t": 78.0,
          "v": 15.7
        },
        {
          "t": 80.0,
          "v": 20.8
        },
        {
          "t": 82.0,
          "v": 13.6
        },
        {
          "t": 84.0,
          "v": 26.2
        },
        {
          "t": 86.0,
          "v": 10.0
        },
        {
          "t": 88.0,
          "v": 30.6
        },
        {
          "t": 90.0,
          "v": 6.0
        },
        {
          "t": 92.0,
          "v": 30.9
        },
        {
          "t": 94.0,
          "v": 8.7
        },
        {
          "t": 96.0,
          "v": 27.9
        },
        {
          "t": 98.0,
          "v": 13.0
        },
        {
          "t": 100.0,
          "v": 21.6
        },
        {
          "t": 102.0,
          "v": 13.1
        },
        {
          "t": 104.0,
          "v": 13.6
        },
        {
          "t": 106.0,
          "v": 17.0
        },
        {
          "t": 108.0,
          "v": 11.7
        },
        {
          "t": 110.0,
          "v": 25.7
        },
        {
          "t": 112.0,
          "v": 6.4
        },
        {
          "t": 114.0,
          "v": 30.8
        },
        {
          "t": 116.0,
          "v": 8.6
        },
        {
          "t": 118.0,
          "v": 31.0
        },
        {
          "t": 120.0,
          "v": 12.7
        },
        {
          "t": 122.0,
          "v": 29.2
        },
        {
          "t": 124.0,
          "v": 16.4
        },
        {
          "t": 126.0,
          "v": 22.7
        },
        {
          "t": 128.0,
          "v": 20.3
        },
        {
          "t": 130.0,
          "v": 15.5
        },
        {
          "t": 132.0,
          "v": 19.2
        },
        {
          "t": 134.0,
          "v": 6.9
        },
        {
          "t": 136.0,
          "v": 23.9
        },
        {
          "t": 138.0,
          "v": 32.2
        },
        {
          "t": 140.0,
          "v": 78.9
        },
        {
          "t": 142.0,
          "v": 61.2
        },
        {
          "t": 144.0,
          "v": 73.8
        },
        {
          "t": 146.0,
          "v": 64.9
        },
        {
          "t": 148.0,
          "v": 68.8
        },
        {
          "t": 150.0,
          "v": 72.3
        },
        {
          "t": 152.0,
          "v": 63.1
        },
        {
          "t": 154.0,
          "v": 77.3
        },
        {
          "t": 156.0,
          "v": 57.4
        },
        {
          "t": 158.0,
          "v": 45.6
        },
        {
          "t": 160.0,
          "v": 3.0
        },
        {
          "t": 162.0,
          "v": 26.0
        },
        {
          "t": 164.0,
          "v": 63.8
        },
        {
          "t": 166.0,
          "v": 77.7
        },
        {
          "t": 168.0,
          "v": 57.9
        },
        {
          "t": 170.0,
          "v": 77.2
        },
        {
          "t": 172.0,
          "v": 61.7
        },
        {
          "t": 174.0,
          "v": 75.0
        },
        {
          "t": 176.0,
          "v": 67.7
        },
        {
          "t": 178.0,
          "v": 69.2
        },
        {
          "t": 180.0,
          "v": 72.1
        },
        {
          "t": 182.0,
          "v": 58.7
        },
        {
          "t": 184.0,
          "v": 16.7
        },
        {
          "t": 186.0,
          "v": 13.3
        },
        {
          "t": 188.0,
          "v": 44.3
        },
        {
          "t": 190.0,
          "v": 71.1
        },
        {
          "t": 192.0,
          "v": 72.6
        },
        {
          "t": 194.0,
          "v": 66.0
        },
        {
          "t": 196.0,
          "v": 75.3
        },
        {
          "t": 198.0,
          "v": 59.1
        },
        {
          "t": 200.0,
          "v": 79.2
        },
        {
          "t": 202.0,
          "v": 59.3
        },
        {
          "t": 204.0,
          "v": 76.4
        },
        {
          "t": 206.0,
          "v": 66.8
        },
        {
          "t": 208.0,
          "v": 38.7
        },
        {
          "t": 210.0,
          "v": 6.5
        },
        {
          "t": 212.0,
          "v": 27.0
        },
        {
          "t": 214.0,
          "v": 61.5
        },
        {
          "t": 216.0,
          "v": 77.7
        },
        {
          "t": 218.0,
          "v": 67.8
        },
        {
          "t": 220.0,
          "v": 72.5
        },
        {
          "t": 222.0,
          "v": 73.2
        },
        {
          "t": 224.0,
          "v": 66.8
        },
        {
          "t": 226.0,
          "v": 79.3
        },
        {
          "t": 228.0,
          "v": 57.7
        },
        {
          "t": 230.0,
          "v": 80.0
        },
        {
          "t": 232.0,
          "v": 61.4
        }
      ],
      "cpu_usr": [
        {
          "t": 0.0,
          "v": 16.7
        },
        {
          "t": 2.0,
          "v": 29.8
        },
        {
          "t": 4.0,
          "v": 41.1
        },
        {
          "t": 6.0,
          "v": 35.6
        },
        {
          "t": 8.0,
          "v": 44.9
        },
        {
          "t": 10.0,
          "v": 55.1
        },
        {
          "t": 12.0,
          "v": 56.5
        },
        {
          "t": 14.0,
          "v": 58.7
        },
        {
          "t": 16.0,
          "v": 41.2
        },
        {
          "t": 18.0,
          "v": 37.7
        },
        {
          "t": 20.0,
          "v": 51.2
        },
        {
          "t": 22.0,
          "v": 22.3
        },
        {
          "t": 24.0,
          "v": 2.5
        },
        {
          "t": 26.0,
          "v": 9.3
        },
        {
          "t": 28.0,
          "v": 36.9
        },
        {
          "t": 30.0,
          "v": 40.2
        },
        {
          "t": 32.0,
          "v": 35.8
        },
        {
          "t": 34.0,
          "v": 37.7
        },
        {
          "t": 36.0,
          "v": 33.5
        },
        {
          "t": 38.0,
          "v": 37.0
        },
        {
          "t": 40.0,
          "v": 36.8
        },
        {
          "t": 42.0,
          "v": 34.5
        },
        {
          "t": 44.0,
          "v": 38.3
        },
        {
          "t": 46.0,
          "v": 22.2
        },
        {
          "t": 48.0,
          "v": 4.3
        },
        {
          "t": 50.0,
          "v": 1.8
        },
        {
          "t": 52.0,
          "v": 48.4
        },
        {
          "t": 54.0,
          "v": 66.4
        },
        {
          "t": 56.0,
          "v": 63.9
        },
        {
          "t": 58.0,
          "v": 62.5
        },
        {
          "t": 60.0,
          "v": 61.3
        },
        {
          "t": 62.0,
          "v": 67.3
        },
        {
          "t": 64.0,
          "v": 63.8
        },
        {
          "t": 66.0,
          "v": 60.0
        },
        {
          "t": 68.0,
          "v": 61.1
        },
        {
          "t": 70.0,
          "v": 62.6
        },
        {
          "t": 72.0,
          "v": 54.0
        },
        {
          "t": 74.0,
          "v": 4.5
        },
        {
          "t": 76.0,
          "v": 2.7
        },
        {
          "t": 78.0,
          "v": 3.6
        },
        {
          "t": 80.0,
          "v": 4.9
        },
        {
          "t": 82.0,
          "v": 4.0
        },
        {
          "t": 84.0,
          "v": 7.0
        },
        {
          "t": 86.0,
          "v": 3.2
        },
        {
          "t": 88.0,
          "v": 7.9
        },
        {
          "t": 90.0,
          "v": 2.6
        },
        {
          "t": 92.0,
          "v": 7.4
        },
        {
          "t": 94.0,
          "v": 3.2
        },
        {
          "t": 96.0,
          "v": 7.1
        },
        {
          "t": 98.0,
          "v": 4.2
        },
        {
          "t": 100.0,
          "v": 5.1
        },
        {
          "t": 102.0,
          "v": 2.9
        },
        {
          "t": 104.0,
          "v": 2.9
        },
        {
          "t": 106.0,
          "v": 2.9
        },
        {
          "t": 108.0,
          "v": 4.9
        },
        {
          "t": 110.0,
          "v": 5.8
        },
        {
          "t": 112.0,
          "v": 2.2
        },
        {
          "t": 114.0,
          "v": 8.2
        },
        {
          "t": 116.0,
          "v": 3.4
        },
        {
          "t": 118.0,
          "v": 8.9
        },
        {
          "t": 120.0,
          "v": 4.5
        },
        {
          "t": 122.0,
          "v": 9.8
        },
        {
          "t": 124.0,
          "v": 5.6
        },
        {
          "t": 126.0,
          "v": 6.6
        },
        {
          "t": 128.0,
          "v": 6.2
        },
        {
          "t": 130.0,
          "v": 4.1
        },
        {
          "t": 132.0,
          "v": 4.0
        },
        {
          "t": 134.0,
          "v": 1.4
        },
        {
          "t": 136.0,
          "v": 4.6
        },
        {
          "t": 138.0,
          "v": 19.6
        },
        {
          "t": 140.0,
          "v": 36.0
        },
        {
          "t": 142.0,
          "v": 32.9
        },
        {
          "t": 144.0,
          "v": 35.4
        },
        {
          "t": 146.0,
          "v": 31.3
        },
        {
          "t": 148.0,
          "v": 33.8
        },
        {
          "t": 150.0,
          "v": 36.3
        },
        {
          "t": 152.0,
          "v": 33.2
        },
        {
          "t": 154.0,
          "v": 35.9
        },
        {
          "t": 156.0,
          "v": 31.3
        },
        {
          "t": 158.0,
          "v": 16.5
        },
        {
          "t": 160.0,
          "v": 0.6
        },
        {
          "t": 162.0,
          "v": 7.3
        },
        {
          "t": 164.0,
          "v": 31.6
        },
        {
          "t": 166.0,
          "v": 35.4
        },
        {
          "t": 168.0,
          "v": 30.0
        },
        {
          "t": 170.0,
          "v": 31.5
        },
        {
          "t": 172.0,
          "v": 31.7
        },
        {
          "t": 174.0,
          "v": 35.4
        },
        {
          "t": 176.0,
          "v": 31.9
        },
        {
          "t": 178.0,
          "v": 31.6
        },
        {
          "t": 180.0,
          "v": 33.4
        },
        {
          "t": 182.0,
          "v": 26.0
        },
        {
          "t": 184.0,
          "v": 3.4
        },
        {
          "t": 186.0,
          "v": 2.5
        },
        {
          "t": 188.0,
          "v": 21.5
        },
        {
          "t": 190.0,
          "v": 33.9
        },
        {
          "t": 192.0,
          "v": 34.2
        },
        {
          "t": 194.0,
          "v": 33.3
        },
        {
          "t": 196.0,
          "v": 32.7
        },
        {
          "t": 198.0,
          "v": 32.7
        },
        {
          "t": 200.0,
          "v": 35.5
        },
        {
          "t": 202.0,
          "v": 32.2
        },
        {
          "t": 204.0,
          "v": 36.3
        },
        {
          "t": 206.0,
          "v": 33.5
        },
        {
          "t": 208.0,
          "v": 14.5
        },
        {
          "t": 210.0,
          "v": 1.5
        },
        {
          "t": 212.0,
          "v": 6.8
        },
        {
          "t": 214.0,
          "v": 36.0
        },
        {
          "t": 216.0,
          "v": 37.8
        },
        {
          "t": 218.0,
          "v": 37.0
        },
        {
          "t": 220.0,
          "v": 37.0
        },
        {
          "t": 222.0,
          "v": 36.8
        },
        {
          "t": 224.0,
          "v": 35.8
        },
        {
          "t": 226.0,
          "v": 38.5
        },
        {
          "t": 228.0,
          "v": 29.9
        },
        {
          "t": 230.0,
          "v": 39.0
        },
        {
          "t": 232.0,
          "v": 34.3
        }
      ],
      "cpu_sys": [
        {
          "t": 0.0,
          "v": 26.3
        },
        {
          "t": 2.0,
          "v": 17.6
        },
        {
          "t": 4.0,
          "v": 38.0
        },
        {
          "t": 6.0,
          "v": 25.0
        },
        {
          "t": 8.0,
          "v": 30.3
        },
        {
          "t": 10.0,
          "v": 31.8
        },
        {
          "t": 12.0,
          "v": 21.5
        },
        {
          "t": 14.0,
          "v": 28.8
        },
        {
          "t": 16.0,
          "v": 37.5
        },
        {
          "t": 18.0,
          "v": 21.3
        },
        {
          "t": 20.0,
          "v": 33.6
        },
        {
          "t": 22.0,
          "v": 18.0
        },
        {
          "t": 24.0,
          "v": 9.8
        },
        {
          "t": 26.0,
          "v": 15.1
        },
        {
          "t": 28.0,
          "v": 29.1
        },
        {
          "t": 30.0,
          "v": 35.4
        },
        {
          "t": 32.0,
          "v": 20.6
        },
        {
          "t": 34.0,
          "v": 39.2
        },
        {
          "t": 36.0,
          "v": 22.0
        },
        {
          "t": 38.0,
          "v": 35.6
        },
        {
          "t": 40.0,
          "v": 30.3
        },
        {
          "t": 42.0,
          "v": 28.3
        },
        {
          "t": 44.0,
          "v": 36.8
        },
        {
          "t": 46.0,
          "v": 15.8
        },
        {
          "t": 48.0,
          "v": 16.8
        },
        {
          "t": 50.0,
          "v": 6.4
        },
        {
          "t": 52.0,
          "v": 26.8
        },
        {
          "t": 54.0,
          "v": 20.8
        },
        {
          "t": 56.0,
          "v": 27.4
        },
        {
          "t": 58.0,
          "v": 28.7
        },
        {
          "t": 60.0,
          "v": 30.2
        },
        {
          "t": 62.0,
          "v": 23.1
        },
        {
          "t": 64.0,
          "v": 27.4
        },
        {
          "t": 66.0,
          "v": 30.7
        },
        {
          "t": 68.0,
          "v": 29.3
        },
        {
          "t": 70.0,
          "v": 28.2
        },
        {
          "t": 72.0,
          "v": 17.5
        },
        {
          "t": 74.0,
          "v": 15.5
        },
        {
          "t": 76.0,
          "v": 10.1
        },
        {
          "t": 78.0,
          "v": 12.1
        },
        {
          "t": 80.0,
          "v": 15.5
        },
        {
          "t": 82.0,
          "v": 9.2
        },
        {
          "t": 84.0,
          "v": 18.7
        },
        {
          "t": 86.0,
          "v": 6.2
        },
        {
          "t": 88.0,
          "v": 22.1
        },
        {
          "t": 90.0,
          "v": 3.0
        },
        {
          "t": 92.0,
          "v": 23.2
        },
        {
          "t": 94.0,
          "v": 4.8
        },
        {
          "t": 96.0,
          "v": 20.3
        },
        {
          "t": 98.0,
          "v": 8.4
        },
        {
          "t": 100.0,
          "v": 16.1
        },
        {
          "t": 102.0,
          "v": 10.2
        },
        {
          "t": 104.0,
          "v": 10.7
        },
        {
          "t": 106.0,
          "v": 14.1
        },
        {
          "t": 108.0,
          "v": 6.5
        },
        {
          "t": 110.0,
          "v": 19.7
        },
        {
          "t": 112.0,
          "v": 3.7
        },
        {
          "t": 114.0,
          "v": 22.2
        },
        {
          "t": 116.0,
          "v": 4.8
        },
        {
          "t": 118.0,
          "v": 21.5
        },
        {
          "t": 120.0,
          "v": 7.8
        },
        {
          "t": 122.0,
          "v": 18.6
        },
        {
          "t": 124.0,
          "v": 10.1
        },
        {
          "t": 126.0,
          "v": 15.4
        },
        {
          "t": 128.0,
          "v": 13.7
        },
        {
          "t": 130.0,
          "v": 10.8
        },
        {
          "t": 132.0,
          "v": 15.2
        },
        {
          "t": 134.0,
          "v": 5.5
        },
        {
          "t": 136.0,
          "v": 19.4
        },
        {
          "t": 138.0,
          "v": 10.0
        },
        {
          "t": 140.0,
          "v": 39.3
        },
        {
          "t": 142.0,
          "v": 23.9
        },
        {
          "t": 144.0,
          "v": 34.4
        },
        {
          "t": 146.0,
          "v": 29.0
        },
        {
          "t": 148.0,
          "v": 30.1
        },
        {
          "t": 150.0,
          "v": 31.8
        },
        {
          "t": 152.0,
          "v": 25.2
        },
        {
          "t": 154.0,
          "v": 37.0
        },
        {
          "t": 156.0,
          "v": 20.5
        },
        {
          "t": 158.0,
          "v": 27.2
        },
        {
          "t": 160.0,
          "v": 2.4
        },
        {
          "t": 162.0,
          "v": 18.3
        },
        {
          "t": 164.0,
          "v": 26.6
        },
        {
          "t": 166.0,
          "v": 38.3
        },
        {
          "t": 168.0,
          "v": 21.9
        },
        {
          "t": 170.0,
          "v": 41.3
        },
        {
          "t": 172.0,
          "v": 24.3
        },
        {
          "t": 174.0,
          "v": 34.3
        },
        {
          "t": 176.0,
          "v": 30.1
        },
        {
          "t": 178.0,
          "v": 31.7
        },
        {
          "t": 180.0,
          "v": 33.6
        },
        {
          "t": 182.0,
          "v": 27.5
        },
        {
          "t": 184.0,
          "v": 13.3
        },
        {
          "t": 186.0,
          "v": 10.8
        },
        {
          "t": 188.0,
          "v": 20.1
        },
        {
          "t": 190.0,
          "v": 33.0
        },
        {
          "t": 192.0,
          "v": 33.5
        },
        {
          "t": 194.0,
          "v": 27.8
        },
        {
          "t": 196.0,
          "v": 38.0
        },
        {
          "t": 198.0,
          "v": 21.8
        },
        {
          "t": 200.0,
          "v": 39.1
        },
        {
          "t": 202.0,
          "v": 22.5
        },
        {
          "t": 204.0,
          "v": 35.7
        },
        {
          "t": 206.0,
          "v": 28.5
        },
        {
          "t": 208.0,
          "v": 22.4
        },
        {
          "t": 210.0,
          "v": 5.0
        },
        {
          "t": 212.0,
          "v": 19.8
        },
        {
          "t": 214.0,
          "v": 20.8
        },
        {
          "t": 216.0,
          "v": 35.9
        },
        {
          "t": 218.0,
          "v": 26.2
        },
        {
          "t": 220.0,
          "v": 31.0
        },
        {
          "t": 222.0,
          "v": 32.1
        },
        {
          "t": 224.0,
          "v": 26.2
        },
        {
          "t": 226.0,
          "v": 36.7
        },
        {
          "t": 228.0,
          "v": 22.5
        },
        {
          "t": 230.0,
          "v": 37.0
        },
        {
          "t": 232.0,
          "v": 22.5
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
          "v": 0.5
        },
        {
          "t": 2.0,
          "v": 4.5
        },
        {
          "t": 4.0,
          "v": 5.3
        },
        {
          "t": 6.0,
          "v": 6.3
        },
        {
          "t": 8.0,
          "v": 6.1
        },
        {
          "t": 10.0,
          "v": 6.3
        },
        {
          "t": 12.0,
          "v": 5.5
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
          "v": 6.5
        },
        {
          "t": 20.0,
          "v": 6.2
        },
        {
          "t": 22.0,
          "v": 2.3
        },
        {
          "t": 24.0,
          "v": 0.1
        },
        {
          "t": 26.0,
          "v": 1.1
        },
        {
          "t": 28.0,
          "v": 5.9
        },
        {
          "t": 30.0,
          "v": 5.0
        },
        {
          "t": 32.0,
          "v": 6.8
        },
        {
          "t": 34.0,
          "v": 5.0
        },
        {
          "t": 36.0,
          "v": 6.1
        },
        {
          "t": 38.0,
          "v": 5.0
        },
        {
          "t": 40.0,
          "v": 5.8
        },
        {
          "t": 42.0,
          "v": 5.8
        },
        {
          "t": 44.0,
          "v": 5.1
        },
        {
          "t": 46.0,
          "v": 4.7
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
          "v": 4.6
        },
        {
          "t": 54.0,
          "v": 6.8
        },
        {
          "t": 56.0,
          "v": 6.8
        },
        {
          "t": 58.0,
          "v": 7.4
        },
        {
          "t": 60.0,
          "v": 7.4
        },
        {
          "t": 62.0,
          "v": 8.2
        },
        {
          "t": 64.0,
          "v": 7.9
        },
        {
          "t": 66.0,
          "v": 8.0
        },
        {
          "t": 68.0,
          "v": 8.7
        },
        {
          "t": 70.0,
          "v": 7.9
        },
        {
          "t": 72.0,
          "v": 2.8
        },
        {
          "t": 74.0,
          "v": 0.1
        },
        {
          "t": 76.0,
          "v": 0.1
        },
        {
          "t": 78.0,
          "v": 0.0
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
          "v": 0.5
        },
        {
          "t": 90.0,
          "v": 0.4
        },
        {
          "t": 92.0,
          "v": 0.3
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
          "v": 0.4
        },
        {
          "t": 100.0,
          "v": 0.4
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
          "v": 0.2
        },
        {
          "t": 110.0,
          "v": 0.3
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
          "v": 0.4
        },
        {
          "t": 118.0,
          "v": 0.7
        },
        {
          "t": 120.0,
          "v": 0.4
        },
        {
          "t": 122.0,
          "v": 0.8
        },
        {
          "t": 124.0,
          "v": 0.7
        },
        {
          "t": 126.0,
          "v": 0.7
        },
        {
          "t": 128.0,
          "v": 0.4
        },
        {
          "t": 130.0,
          "v": 0.5
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
          "v": 2.6
        },
        {
          "t": 140.0,
          "v": 3.6
        },
        {
          "t": 142.0,
          "v": 4.5
        },
        {
          "t": 144.0,
          "v": 4.1
        },
        {
          "t": 146.0,
          "v": 4.7
        },
        {
          "t": 148.0,
          "v": 4.8
        },
        {
          "t": 150.0,
          "v": 4.2
        },
        {
          "t": 152.0,
          "v": 4.7
        },
        {
          "t": 154.0,
          "v": 4.4
        },
        {
          "t": 156.0,
          "v": 5.6
        },
        {
          "t": 158.0,
          "v": 1.8
        },
        {
          "t": 160.0,
          "v": 0.0
        },
        {
          "t": 162.0,
          "v": 0.4
        },
        {
          "t": 164.0,
          "v": 5.6
        },
        {
          "t": 166.0,
          "v": 4.0
        },
        {
          "t": 168.0,
          "v": 5.9
        },
        {
          "t": 170.0,
          "v": 4.4
        },
        {
          "t": 172.0,
          "v": 5.7
        },
        {
          "t": 174.0,
          "v": 5.4
        },
        {
          "t": 176.0,
          "v": 5.8
        },
        {
          "t": 178.0,
          "v": 6.0
        },
        {
          "t": 180.0,
          "v": 5.1
        },
        {
          "t": 182.0,
          "v": 5.2
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
          "v": 2.7
        },
        {
          "t": 190.0,
          "v": 4.2
        },
        {
          "t": 192.0,
          "v": 4.8
        },
        {
          "t": 194.0,
          "v": 4.9
        },
        {
          "t": 196.0,
          "v": 4.7
        },
        {
          "t": 198.0,
          "v": 4.6
        },
        {
          "t": 200.0,
          "v": 4.6
        },
        {
          "t": 202.0,
          "v": 4.6
        },
        {
          "t": 204.0,
          "v": 4.4
        },
        {
          "t": 206.0,
          "v": 4.7
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
          "v": 0.4
        },
        {
          "t": 214.0,
          "v": 4.7
        },
        {
          "t": 216.0,
          "v": 4.0
        },
        {
          "t": 218.0,
          "v": 4.6
        },
        {
          "t": 220.0,
          "v": 4.5
        },
        {
          "t": 222.0,
          "v": 4.3
        },
        {
          "t": 224.0,
          "v": 4.8
        },
        {
          "t": 226.0,
          "v": 4.1
        },
        {
          "t": 228.0,
          "v": 5.3
        },
        {
          "t": 230.0,
          "v": 4.0
        },
        {
          "t": 232.0,
          "v": 4.7
        }
      ],
      "cpu_iowait": [
        {
          "t": 0.0,
          "v": 0.2
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
          "v": 1.3
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
      "mem_total_mb": 15990
    },
    "network": {
      "rx_kbs": [
        {
          "t": 0.0,
          "v": 6496.6
        },
        {
          "t": 2.0,
          "v": 6.6
        },
        {
          "t": 4.0,
          "v": 0.1
        },
        {
          "t": 6.0,
          "v": 1.9
        },
        {
          "t": 8.0,
          "v": 4.1
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
          "v": 0.1
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
          "v": 2.0
        },
        {
          "t": 26.0,
          "v": 2.6
        },
        {
          "t": 28.0,
          "v": 0.1
        },
        {
          "t": 30.0,
          "v": 2.5
        },
        {
          "t": 32.0,
          "v": 0.3
        },
        {
          "t": 34.0,
          "v": 0.1
        },
        {
          "t": 36.0,
          "v": 1.9
        },
        {
          "t": 38.0,
          "v": 0.7
        },
        {
          "t": 40.0,
          "v": 0.3
        },
        {
          "t": 42.0,
          "v": 2.1
        },
        {
          "t": 44.0,
          "v": 0.1
        },
        {
          "t": 46.0,
          "v": 0.1
        },
        {
          "t": 48.0,
          "v": 1.9
        },
        {
          "t": 50.0,
          "v": 0.1
        },
        {
          "t": 52.0,
          "v": 0.2
        },
        {
          "t": 54.0,
          "v": 2.0
        },
        {
          "t": 56.0,
          "v": 0.1
        },
        {
          "t": 58.0,
          "v": 0.1
        },
        {
          "t": 60.0,
          "v": 1.9
        },
        {
          "t": 62.0,
          "v": 0.1
        },
        {
          "t": 64.0,
          "v": 0.1
        },
        {
          "t": 66.0,
          "v": 1.9
        },
        {
          "t": 68.0,
          "v": 0.7
        },
        {
          "t": 70.0,
          "v": 0.1
        },
        {
          "t": 72.0,
          "v": 11.3
        },
        {
          "t": 74.0,
          "v": 1.0
        },
        {
          "t": 76.0,
          "v": 0.1
        },
        {
          "t": 78.0,
          "v": 1.9
        },
        {
          "t": 80.0,
          "v": 0.1
        },
        {
          "t": 82.0,
          "v": 0.3
        },
        {
          "t": 84.0,
          "v": 2.0
        },
        {
          "t": 86.0,
          "v": 2.7
        },
        {
          "t": 88.0,
          "v": 0.1
        },
        {
          "t": 90.0,
          "v": 2.1
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
          "v": 1.9
        },
        {
          "t": 98.0,
          "v": 0.4
        },
        {
          "t": 100.0,
          "v": 0.3
        },
        {
          "t": 102.0,
          "v": 2.2
        },
        {
          "t": 104.0,
          "v": 0.1
        },
        {
          "t": 106.0,
          "v": 0.1
        },
        {
          "t": 108.0,
          "v": 1.9
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
          "v": 1.9
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
          "v": 0.4
        },
        {
          "t": 130.0,
          "v": 0.1
        },
        {
          "t": 132.0,
          "v": 2.1
        },
        {
          "t": 134.0,
          "v": 0.1
        },
        {
          "t": 136.0,
          "v": 0.1
        },
        {
          "t": 138.0,
          "v": 1.9
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
          "v": 2.0
        },
        {
          "t": 146.0,
          "v": 2.6
        },
        {
          "t": 148.0,
          "v": 0.2
        },
        {
          "t": 150.0,
          "v": 2.1
        },
        {
          "t": 152.0,
          "v": 0.1
        },
        {
          "t": 154.0,
          "v": 0.1
        },
        {
          "t": 156.0,
          "v": 1.9
        },
        {
          "t": 158.0,
          "v": 0.3
        },
        {
          "t": 160.0,
          "v": 0.3
        },
        {
          "t": 162.0,
          "v": 2.2
        },
        {
          "t": 164.0,
          "v": 0.1
        },
        {
          "t": 166.0,
          "v": 0.1
        },
        {
          "t": 168.0,
          "v": 1.9
        },
        {
          "t": 170.0,
          "v": 0.1
        },
        {
          "t": 172.0,
          "v": 0.1
        },
        {
          "t": 174.0,
          "v": 2.0
        },
        {
          "t": 176.0,
          "v": 0.1
        },
        {
          "t": 178.0,
          "v": 0.1
        },
        {
          "t": 180.0,
          "v": 1.9
        },
        {
          "t": 182.0,
          "v": 0.3
        },
        {
          "t": 184.0,
          "v": 0.1
        },
        {
          "t": 186.0,
          "v": 1.9
        },
        {
          "t": 188.0,
          "v": 0.4
        },
        {
          "t": 190.0,
          "v": 0.1
        },
        {
          "t": 192.0,
          "v": 2.0
        },
        {
          "t": 194.0,
          "v": 0.1
        },
        {
          "t": 196.0,
          "v": 0.1
        },
        {
          "t": 198.0,
          "v": 1.9
        },
        {
          "t": 200.0,
          "v": 0.1
        },
        {
          "t": 202.0,
          "v": 0.1
        },
        {
          "t": 204.0,
          "v": 2.0
        },
        {
          "t": 206.0,
          "v": 2.6
        },
        {
          "t": 208.0,
          "v": 0.1
        },
        {
          "t": 210.0,
          "v": 2.1
        },
        {
          "t": 212.0,
          "v": 0.1
        },
        {
          "t": 214.0,
          "v": 0.1
        },
        {
          "t": 216.0,
          "v": 1.9
        },
        {
          "t": 218.0,
          "v": 0.3
        },
        {
          "t": 220.0,
          "v": 0.3
        },
        {
          "t": 222.0,
          "v": 2.2
        },
        {
          "t": 224.0,
          "v": 0.1
        },
        {
          "t": 226.0,
          "v": 0.1
        },
        {
          "t": 228.0,
          "v": 1.9
        },
        {
          "t": 230.0,
          "v": 0.1
        },
        {
          "t": 232.0,
          "v": 2.5
        }
      ],
      "tx_kbs": [
        {
          "t": 0.0,
          "v": 56.8
        },
        {
          "t": 2.0,
          "v": 6.6
        },
        {
          "t": 4.0,
          "v": 0.5
        },
        {
          "t": 6.0,
          "v": 4.3
        },
        {
          "t": 8.0,
          "v": 11.9
        },
        {
          "t": 10.0,
          "v": 0.5
        },
        {
          "t": 12.0,
          "v": 4.4
        },
        {
          "t": 14.0,
          "v": 0.5
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
          "v": 4.4
        },
        {
          "t": 26.0,
          "v": 1.9
        },
        {
          "t": 28.0,
          "v": 0.5
        },
        {
          "t": 30.0,
          "v": 31.6
        },
        {
          "t": 32.0,
          "v": 1.6
        },
        {
          "t": 34.0,
          "v": 0.5
        },
        {
          "t": 36.0,
          "v": 4.4
        },
        {
          "t": 38.0,
          "v": 19.4
        },
        {
          "t": 40.0,
          "v": 0.7
        },
        {
          "t": 42.0,
          "v": 5.9
        },
        {
          "t": 44.0,
          "v": 0.5
        },
        {
          "t": 46.0,
          "v": 0.6
        },
        {
          "t": 48.0,
          "v": 4.3
        },
        {
          "t": 50.0,
          "v": 0.5
        },
        {
          "t": 52.0,
          "v": 0.7
        },
        {
          "t": 54.0,
          "v": 4.5
        },
        {
          "t": 56.0,
          "v": 0.6
        },
        {
          "t": 58.0,
          "v": 0.6
        },
        {
          "t": 60.0,
          "v": 4.5
        },
        {
          "t": 62.0,
          "v": 0.6
        },
        {
          "t": 64.0,
          "v": 0.5
        },
        {
          "t": 66.0,
          "v": 4.4
        },
        {
          "t": 68.0,
          "v": 18.6
        },
        {
          "t": 70.0,
          "v": 0.6
        },
        {
          "t": 72.0,
          "v": 36.4
        },
        {
          "t": 74.0,
          "v": 5.7
        },
        {
          "t": 76.0,
          "v": 1.6
        },
        {
          "t": 78.0,
          "v": 5.4
        },
        {
          "t": 80.0,
          "v": 1.6
        },
        {
          "t": 82.0,
          "v": 2.7
        },
        {
          "t": 84.0,
          "v": 5.5
        },
        {
          "t": 86.0,
          "v": 3.1
        },
        {
          "t": 88.0,
          "v": 1.6
        },
        {
          "t": 90.0,
          "v": 5.8
        },
        {
          "t": 92.0,
          "v": 1.6
        },
        {
          "t": 94.0,
          "v": 1.6
        },
        {
          "t": 96.0,
          "v": 5.5
        },
        {
          "t": 98.0,
          "v": 12.1
        },
        {
          "t": 100.0,
          "v": 1.8
        },
        {
          "t": 102.0,
          "v": 7.1
        },
        {
          "t": 104.0,
          "v": 1.6
        },
        {
          "t": 106.0,
          "v": 1.6
        },
        {
          "t": 108.0,
          "v": 5.5
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
          "v": 1.7
        },
        {
          "t": 120.0,
          "v": 5.6
        },
        {
          "t": 122.0,
          "v": 1.7
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
          "v": 12.2
        },
        {
          "t": 130.0,
          "v": 1.8
        },
        {
          "t": 132.0,
          "v": 6.7
        },
        {
          "t": 134.0,
          "v": 1.7
        },
        {
          "t": 136.0,
          "v": 1.8
        },
        {
          "t": 138.0,
          "v": 5.6
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
          "v": 5.7
        },
        {
          "t": 146.0,
          "v": 3.2
        },
        {
          "t": 148.0,
          "v": 1.9
        },
        {
          "t": 150.0,
          "v": 6.0
        },
        {
          "t": 152.0,
          "v": 1.8
        },
        {
          "t": 154.0,
          "v": 1.8
        },
        {
          "t": 156.0,
          "v": 5.6
        },
        {
          "t": 158.0,
          "v": 10.8
        },
        {
          "t": 160.0,
          "v": 2.0
        },
        {
          "t": 162.0,
          "v": 7.3
        },
        {
          "t": 164.0,
          "v": 1.8
        },
        {
          "t": 166.0,
          "v": 1.8
        },
        {
          "t": 168.0,
          "v": 5.6
        },
        {
          "t": 170.0,
          "v": 1.8
        },
        {
          "t": 172.0,
          "v": 1.8
        },
        {
          "t": 174.0,
          "v": 5.7
        },
        {
          "t": 176.0,
          "v": 1.8
        },
        {
          "t": 178.0,
          "v": 1.8
        },
        {
          "t": 180.0,
          "v": 5.6
        },
        {
          "t": 182.0,
          "v": 2.8
        },
        {
          "t": 184.0,
          "v": 1.8
        },
        {
          "t": 186.0,
          "v": 5.6
        },
        {
          "t": 188.0,
          "v": 12.8
        },
        {
          "t": 190.0,
          "v": 1.8
        },
        {
          "t": 192.0,
          "v": 5.7
        },
        {
          "t": 194.0,
          "v": 1.8
        },
        {
          "t": 196.0,
          "v": 1.8
        },
        {
          "t": 198.0,
          "v": 5.7
        },
        {
          "t": 200.0,
          "v": 1.8
        },
        {
          "t": 202.0,
          "v": 1.8
        },
        {
          "t": 204.0,
          "v": 5.7
        },
        {
          "t": 206.0,
          "v": 3.3
        },
        {
          "t": 208.0,
          "v": 1.8
        },
        {
          "t": 210.0,
          "v": 6.0
        },
        {
          "t": 212.0,
          "v": 1.8
        },
        {
          "t": 214.0,
          "v": 1.8
        },
        {
          "t": 216.0,
          "v": 5.7
        },
        {
          "t": 218.0,
          "v": 11.5
        },
        {
          "t": 220.0,
          "v": 2.0
        },
        {
          "t": 222.0,
          "v": 7.2
        },
        {
          "t": 224.0,
          "v": 1.8
        },
        {
          "t": 226.0,
          "v": 1.9
        },
        {
          "t": 228.0,
          "v": 5.7
        },
        {
          "t": 230.0,
          "v": 1.8
        },
        {
          "t": 232.0,
          "v": 4.3
        }
      ]
    },
    "paging": {
      "majflt": [
        {
          "t": 0.0,
          "v": 90449.0
        },
        {
          "t": 2.0,
          "v": 15432.0
        },
        {
          "t": 4.0,
          "v": 66984.0
        },
        {
          "t": 6.0,
          "v": 21041.5
        },
        {
          "t": 8.0,
          "v": 43260.5
        },
        {
          "t": 10.0,
          "v": 55792.0
        },
        {
          "t": 12.0,
          "v": 17645.0
        },
        {
          "t": 14.0,
          "v": 40608.5
        },
        {
          "t": 16.0,
          "v": 68525.0
        },
        {
          "t": 18.0,
          "v": 4538.0
        },
        {
          "t": 20.0,
          "v": 53685.07
        },
        {
          "t": 22.0,
          "v": 51929.5
        },
        {
          "t": 24.0,
          "v": 45204.0
        },
        {
          "t": 26.0,
          "v": 53318.5
        },
        {
          "t": 28.0,
          "v": 41442.5
        },
        {
          "t": 30.0,
          "v": 58831.0
        },
        {
          "t": 32.0,
          "v": 14428.5
        },
        {
          "t": 34.0,
          "v": 73218.5
        },
        {
          "t": 36.0,
          "v": 13890.0
        },
        {
          "t": 38.0,
          "v": 58593.0
        },
        {
          "t": 40.0,
          "v": 40084.5
        },
        {
          "t": 42.0,
          "v": 35029.0
        },
        {
          "t": 44.0,
          "v": 64917.5
        },
        {
          "t": 46.0,
          "v": 14034.5
        },
        {
          "t": 48.0,
          "v": 84928.0
        },
        {
          "t": 50.0,
          "v": 36322.0
        },
        {
          "t": 52.0,
          "v": 60608.5
        },
        {
          "t": 54.0,
          "v": 7745.5
        },
        {
          "t": 56.0,
          "v": 32397.01
        },
        {
          "t": 58.0,
          "v": 36213.0
        },
        {
          "t": 60.0,
          "v": 38698.0
        },
        {
          "t": 62.0,
          "v": 10407.5
        },
        {
          "t": 64.0,
          "v": 34557.5
        },
        {
          "t": 66.0,
          "v": 35259.5
        },
        {
          "t": 68.0,
          "v": 30396.02
        },
        {
          "t": 70.0,
          "v": 27909.0
        },
        {
          "t": 72.0,
          "v": 116899.0
        },
        {
          "t": 74.0,
          "v": 71989.5
        },
        {
          "t": 76.0,
          "v": 47928.0
        },
        {
          "t": 78.0,
          "v": 50585.0
        },
        {
          "t": 80.0,
          "v": 64892.0
        },
        {
          "t": 82.0,
          "v": 34121.0
        },
        {
          "t": 84.0,
          "v": 82998.5
        },
        {
          "t": 86.0,
          "v": 17715.0
        },
        {
          "t": 88.0,
          "v": 97156.5
        },
        {
          "t": 90.0,
          "v": 1528.5
        },
        {
          "t": 92.0,
          "v": 96865.0
        },
        {
          "t": 94.0,
          "v": 16076.5
        },
        {
          "t": 96.0,
          "v": 82201.0
        },
        {
          "t": 98.0,
          "v": 30564.0
        },
        {
          "t": 100.0,
          "v": 67725.0
        },
        {
          "t": 102.0,
          "v": 48800.0
        },
        {
          "t": 104.0,
          "v": 50754.5
        },
        {
          "t": 106.0,
          "v": 70916.0
        },
        {
          "t": 108.0,
          "v": 27510.0
        },
        {
          "t": 110.0,
          "v": 93686.5
        },
        {
          "t": 112.0,
          "v": 8052.5
        },
        {
          "t": 114.0,
          "v": 99228.5
        },
        {
          "t": 116.0,
          "v": 13360.0
        },
        {
          "t": 118.0,
          "v": 89131.0
        },
        {
          "t": 120.0,
          "v": 27647.5
        },
        {
          "t": 122.0,
          "v": 74391.0
        },
        {
          "t": 124.0,
          "v": 42532.0
        },
        {
          "t": 126.0,
          "v": 59652.5
        },
        {
          "t": 128.0,
          "v": 57112.5
        },
        {
          "t": 130.0,
          "v": 43323.5
        },
        {
          "t": 132.0,
          "v": 77179.5
        },
        {
          "t": 134.0,
          "v": 23206.0
        },
        {
          "t": 136.0,
          "v": 98965.0
        },
        {
          "t": 138.0,
          "v": 2240.5
        },
        {
          "t": 140.0,
          "v": 81626.5
        },
        {
          "t": 142.0,
          "v": 17562.5
        },
        {
          "t": 144.0,
          "v": 62468.0
        },
        {
          "t": 146.0,
          "v": 37948.0
        },
        {
          "t": 148.0,
          "v": 44853.5
        },
        {
          "t": 150.0,
          "v": 55288.5
        },
        {
          "t": 152.0,
          "v": 26170.0
        },
        {
          "t": 154.0,
          "v": 74278.5
        },
        {
          "t": 156.0,
          "v": 6629.5
        },
        {
          "t": 158.0,
          "v": 94519.0
        },
        {
          "t": 160.0,
          "v": 14101.0
        },
        {
          "t": 162.0,
          "v": 87644.0
        },
        {
          "t": 164.0,
          "v": 25213.5
        },
        {
          "t": 166.0,
          "v": 74862.5
        },
        {
          "t": 168.0,
          "v": 5051.5
        },
        {
          "t": 170.0,
          "v": 82662.0
        },
        {
          "t": 172.0,
          "v": 13973.5
        },
        {
          "t": 174.0,
          "v": 64416.5
        },
        {
          "t": 176.0,
          "v": 37437.5
        },
        {
          "t": 178.0,
          "v": 46183.5
        },
        {
          "t": 180.0,
          "v": 54748.5
        },
        {
          "t": 182.0,
          "v": 33935.0
        },
        {
          "t": 184.0,
          "v": 68274.0
        },
        {
          "t": 186.0,
          "v": 57391.0
        },
        {
          "t": 188.0,
          "v": 44686.5
        },
        {
          "t": 190.0,
          "v": 50910.0
        },
        {
          "t": 192.0,
          "v": 49980.0
        },
        {
          "t": 194.0,
          "v": 30166.67
        },
        {
          "t": 196.0,
          "v": 70106.5
        },
        {
          "t": 198.0,
          "v": 12221.0
        },
        {
          "t": 200.0,
          "v": 79304.5
        },
        {
          "t": 202.0,
          "v": 10939.5
        },
        {
          "t": 204.0,
          "v": 69842.5
        },
        {
          "t": 206.0,
          "v": 31487.0
        },
        {
          "t": 208.0,
          "v": 77428.0
        },
        {
          "t": 210.0,
          "v": 24294.5
        },
        {
          "t": 212.0,
          "v": 95884.5
        },
        {
          "t": 214.0,
          "v": 7103.5
        },
        {
          "t": 216.0,
          "v": 73572.0
        },
        {
          "t": 218.0,
          "v": 29055.5
        },
        {
          "t": 220.0,
          "v": 51803.5
        },
        {
          "t": 222.0,
          "v": 51142.5
        },
        {
          "t": 224.0,
          "v": 28292.0
        },
        {
          "t": 226.0,
          "v": 74063.5
        },
        {
          "t": 228.0,
          "v": 7939.5
        },
        {
          "t": 230.0,
          "v": 78679.5
        },
        {
          "t": 232.0,
          "v": 85741.5
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
          "v": 166
        },
        {
          "t": 6,
          "v": 198
        },
        {
          "t": 8,
          "v": 192
        },
        {
          "t": 10,
          "v": 296
        },
        {
          "t": 12,
          "v": 277
        },
        {
          "t": 14,
          "v": 303
        },
        {
          "t": 16,
          "v": 325
        },
        {
          "t": 18,
          "v": 335
        },
        {
          "t": 20,
          "v": 357
        },
        {
          "t": 22,
          "v": 513
        },
        {
          "t": 24,
          "v": 353
        },
        {
          "t": 26,
          "v": 353
        },
        {
          "t": 28,
          "v": 553
        },
        {
          "t": 30,
          "v": 561
        },
        {
          "t": 32,
          "v": 561
        },
        {
          "t": 34,
          "v": 569
        },
        {
          "t": 36,
          "v": 569
        },
        {
          "t": 38,
          "v": 569
        },
        {
          "t": 40,
          "v": 569
        },
        {
          "t": 42,
          "v": 569
        },
        {
          "t": 44,
          "v": 569
        },
        {
          "t": 46,
          "v": 569
        },
        {
          "t": 48,
          "v": 569
        },
        {
          "t": 50,
          "v": 569
        },
        {
          "t": 52,
          "v": 569
        },
        {
          "t": 54,
          "v": 1262
        },
        {
          "t": 56,
          "v": 1402
        },
        {
          "t": 58,
          "v": 1674
        },
        {
          "t": 60,
          "v": 1926
        },
        {
          "t": 62,
          "v": 1984
        },
        {
          "t": 64,
          "v": 1958
        },
        {
          "t": 66,
          "v": 2505
        },
        {
          "t": 68,
          "v": 2140
        },
        {
          "t": 70,
          "v": 1999
        },
        {
          "t": 72,
          "v": 2227
        },
        {
          "t": 74,
          "v": 25
        },
        {
          "t": 76,
          "v": 33
        },
        {
          "t": 79,
          "v": 47
        },
        {
          "t": 81,
          "v": 65
        },
        {
          "t": 83,
          "v": 89
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
          "v": 92
        },
        {
          "t": 107,
          "v": 92
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
          "v": 154
        },
        {
          "t": 119,
          "v": 154
        },
        {
          "t": 121,
          "v": 154
        },
        {
          "t": 123,
          "v": 154
        },
        {
          "t": 125,
          "v": 154
        },
        {
          "t": 127,
          "v": 154
        },
        {
          "t": 129,
          "v": 154
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
          "v": 150
        },
        {
          "t": 139,
          "v": 149
        },
        {
          "t": 141,
          "v": 463
        },
        {
          "t": 143,
          "v": 463
        },
        {
          "t": 145,
          "v": 463
        },
        {
          "t": 147,
          "v": 463
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
          "v": 1055
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
          "v": 1238
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
        },
        {
          "t": 235,
          "v": 9
        }
      ],
      "timewait": [
        {
          "t": 0,
          "v": 24
        },
        {
          "t": 2,
          "v": 25
        },
        {
          "t": 4,
          "v": 690
        },
        {
          "t": 6,
          "v": 1724
        },
        {
          "t": 8,
          "v": 2278
        },
        {
          "t": 10,
          "v": 3691
        },
        {
          "t": 12,
          "v": 6009
        },
        {
          "t": 14,
          "v": 7134
        },
        {
          "t": 16,
          "v": 9623
        },
        {
          "t": 18,
          "v": 10555
        },
        {
          "t": 20,
          "v": 10849
        },
        {
          "t": 22,
          "v": 12721
        },
        {
          "t": 24,
          "v": 13502
        },
        {
          "t": 26,
          "v": 13502
        },
        {
          "t": 28,
          "v": 13480
        },
        {
          "t": 30,
          "v": 13478
        },
        {
          "t": 32,
          "v": 13478
        },
        {
          "t": 34,
          "v": 13478
        },
        {
          "t": 36,
          "v": 13477
        },
        {
          "t": 38,
          "v": 13477
        },
        {
          "t": 40,
          "v": 13477
        },
        {
          "t": 42,
          "v": 13477
        },
        {
          "t": 44,
          "v": 13477
        },
        {
          "t": 46,
          "v": 13477
        },
        {
          "t": 48,
          "v": 13477
        },
        {
          "t": 50,
          "v": 13477
        },
        {
          "t": 52,
          "v": 13477
        },
        {
          "t": 54,
          "v": 13425
        },
        {
          "t": 56,
          "v": 13462
        },
        {
          "t": 58,
          "v": 13885
        },
        {
          "t": 60,
          "v": 14706
        },
        {
          "t": 62,
          "v": 16202
        },
        {
          "t": 64,
          "v": 17367
        },
        {
          "t": 66,
          "v": 17477
        },
        {
          "t": 68,
          "v": 19197
        },
        {
          "t": 70,
          "v": 19859
        },
        {
          "t": 72,
          "v": 20776
        },
        {
          "t": 74,
          "v": 20127
        },
        {
          "t": 76,
          "v": 20127
        },
        {
          "t": 79,
          "v": 17992
        },
        {
          "t": 81,
          "v": 18033
        },
        {
          "t": 83,
          "v": 16775
        },
        {
          "t": 85,
          "v": 16817
        },
        {
          "t": 87,
          "v": 15639
        },
        {
          "t": 89,
          "v": 15683
        },
        {
          "t": 91,
          "v": 15725
        },
        {
          "t": 93,
          "v": 15766
        },
        {
          "t": 95,
          "v": 15808
        },
        {
          "t": 97,
          "v": 15853
        },
        {
          "t": 99,
          "v": 15857
        },
        {
          "t": 101,
          "v": 15857
        },
        {
          "t": 103,
          "v": 15857
        },
        {
          "t": 105,
          "v": 15857
        },
        {
          "t": 107,
          "v": 15857
        },
        {
          "t": 109,
          "v": 15857
        },
        {
          "t": 111,
          "v": 15865
        },
        {
          "t": 113,
          "v": 15886
        },
        {
          "t": 115,
          "v": 15903
        },
        {
          "t": 117,
          "v": 15925
        },
        {
          "t": 119,
          "v": 15451
        },
        {
          "t": 121,
          "v": 15471
        },
        {
          "t": 123,
          "v": 13222
        },
        {
          "t": 125,
          "v": 13243
        },
        {
          "t": 127,
          "v": 10097
        },
        {
          "t": 129,
          "v": 10117
        },
        {
          "t": 131,
          "v": 4541
        },
        {
          "t": 133,
          "v": 4541
        },
        {
          "t": 135,
          "v": 632
        },
        {
          "t": 137,
          "v": 632
        },
        {
          "t": 139,
          "v": 598
        },
        {
          "t": 141,
          "v": 616
        },
        {
          "t": 143,
          "v": 578
        },
        {
          "t": 145,
          "v": 606
        },
        {
          "t": 147,
          "v": 569
        },
        {
          "t": 149,
          "v": 641
        },
        {
          "t": 151,
          "v": 685
        },
        {
          "t": 153,
          "v": 597
        },
        {
          "t": 155,
          "v": 601
        },
        {
          "t": 157,
          "v": 532
        },
        {
          "t": 159,
          "v": 538
        },
        {
          "t": 161,
          "v": 497
        },
        {
          "t": 163,
          "v": 504
        },
        {
          "t": 165,
          "v": 525
        },
        {
          "t": 167,
          "v": 532
        },
        {
          "t": 169,
          "v": 566
        },
        {
          "t": 171,
          "v": 586
        },
        {
          "t": 173,
          "v": 593
        },
        {
          "t": 175,
          "v": 604
        },
        {
          "t": 177,
          "v": 561
        },
        {
          "t": 179,
          "v": 714
        },
        {
          "t": 181,
          "v": 731
        },
        {
          "t": 183,
          "v": 733
        },
        {
          "t": 185,
          "v": 693
        },
        {
          "t": 187,
          "v": 693
        },
        {
          "t": 189,
          "v": 656
        },
        {
          "t": 191,
          "v": 666
        },
        {
          "t": 193,
          "v": 702
        },
        {
          "t": 195,
          "v": 705
        },
        {
          "t": 197,
          "v": 724
        },
        {
          "t": 199,
          "v": 725
        },
        {
          "t": 201,
          "v": 721
        },
        {
          "t": 203,
          "v": 721
        },
        {
          "t": 205,
          "v": 669
        },
        {
          "t": 207,
          "v": 701
        },
        {
          "t": 209,
          "v": 612
        },
        {
          "t": 211,
          "v": 612
        },
        {
          "t": 213,
          "v": 573
        },
        {
          "t": 215,
          "v": 598
        },
        {
          "t": 217,
          "v": 605
        },
        {
          "t": 219,
          "v": 609
        },
        {
          "t": 221,
          "v": 608
        },
        {
          "t": 223,
          "v": 650
        },
        {
          "t": 225,
          "v": 638
        },
        {
          "t": 227,
          "v": 663
        },
        {
          "t": 229,
          "v": 671
        },
        {
          "t": 231,
          "v": 700
        },
        {
          "t": 233,
          "v": 656
        },
        {
          "t": 235,
          "v": 1273
        }
      ],
      "closed": [
        {
          "t": 0,
          "v": 24
        },
        {
          "t": 2,
          "v": 25
        },
        {
          "t": 4,
          "v": 690
        },
        {
          "t": 6,
          "v": 1724
        },
        {
          "t": 8,
          "v": 2278
        },
        {
          "t": 10,
          "v": 3691
        },
        {
          "t": 12,
          "v": 6009
        },
        {
          "t": 14,
          "v": 7134
        },
        {
          "t": 16,
          "v": 9623
        },
        {
          "t": 18,
          "v": 10555
        },
        {
          "t": 20,
          "v": 10849
        },
        {
          "t": 22,
          "v": 12721
        },
        {
          "t": 24,
          "v": 13502
        },
        {
          "t": 26,
          "v": 13502
        },
        {
          "t": 28,
          "v": 13480
        },
        {
          "t": 30,
          "v": 13478
        },
        {
          "t": 32,
          "v": 13478
        },
        {
          "t": 34,
          "v": 13478
        },
        {
          "t": 36,
          "v": 13477
        },
        {
          "t": 38,
          "v": 13477
        },
        {
          "t": 40,
          "v": 13477
        },
        {
          "t": 42,
          "v": 13477
        },
        {
          "t": 44,
          "v": 13477
        },
        {
          "t": 46,
          "v": 13477
        },
        {
          "t": 48,
          "v": 13477
        },
        {
          "t": 50,
          "v": 13477
        },
        {
          "t": 52,
          "v": 13477
        },
        {
          "t": 54,
          "v": 13425
        },
        {
          "t": 56,
          "v": 13462
        },
        {
          "t": 58,
          "v": 13885
        },
        {
          "t": 60,
          "v": 14706
        },
        {
          "t": 62,
          "v": 16202
        },
        {
          "t": 64,
          "v": 17367
        },
        {
          "t": 66,
          "v": 17477
        },
        {
          "t": 68,
          "v": 19197
        },
        {
          "t": 70,
          "v": 19859
        },
        {
          "t": 72,
          "v": 20778
        },
        {
          "t": 74,
          "v": 20127
        },
        {
          "t": 76,
          "v": 20127
        },
        {
          "t": 79,
          "v": 17992
        },
        {
          "t": 81,
          "v": 18033
        },
        {
          "t": 83,
          "v": 16775
        },
        {
          "t": 85,
          "v": 16817
        },
        {
          "t": 87,
          "v": 15639
        },
        {
          "t": 89,
          "v": 15683
        },
        {
          "t": 91,
          "v": 15725
        },
        {
          "t": 93,
          "v": 15766
        },
        {
          "t": 95,
          "v": 15808
        },
        {
          "t": 97,
          "v": 15853
        },
        {
          "t": 99,
          "v": 15857
        },
        {
          "t": 101,
          "v": 15857
        },
        {
          "t": 103,
          "v": 15857
        },
        {
          "t": 105,
          "v": 15857
        },
        {
          "t": 107,
          "v": 15857
        },
        {
          "t": 109,
          "v": 15857
        },
        {
          "t": 111,
          "v": 15865
        },
        {
          "t": 113,
          "v": 15886
        },
        {
          "t": 115,
          "v": 15903
        },
        {
          "t": 117,
          "v": 15925
        },
        {
          "t": 119,
          "v": 15451
        },
        {
          "t": 121,
          "v": 15471
        },
        {
          "t": 123,
          "v": 13222
        },
        {
          "t": 125,
          "v": 13243
        },
        {
          "t": 127,
          "v": 10097
        },
        {
          "t": 129,
          "v": 10117
        },
        {
          "t": 131,
          "v": 4541
        },
        {
          "t": 133,
          "v": 4541
        },
        {
          "t": 135,
          "v": 632
        },
        {
          "t": 137,
          "v": 632
        },
        {
          "t": 139,
          "v": 598
        },
        {
          "t": 141,
          "v": 616
        },
        {
          "t": 143,
          "v": 578
        },
        {
          "t": 145,
          "v": 606
        },
        {
          "t": 147,
          "v": 569
        },
        {
          "t": 149,
          "v": 641
        },
        {
          "t": 151,
          "v": 685
        },
        {
          "t": 153,
          "v": 597
        },
        {
          "t": 155,
          "v": 601
        },
        {
          "t": 157,
          "v": 532
        },
        {
          "t": 159,
          "v": 538
        },
        {
          "t": 161,
          "v": 497
        },
        {
          "t": 163,
          "v": 504
        },
        {
          "t": 165,
          "v": 525
        },
        {
          "t": 167,
          "v": 532
        },
        {
          "t": 169,
          "v": 566
        },
        {
          "t": 171,
          "v": 586
        },
        {
          "t": 173,
          "v": 593
        },
        {
          "t": 175,
          "v": 604
        },
        {
          "t": 177,
          "v": 561
        },
        {
          "t": 179,
          "v": 714
        },
        {
          "t": 181,
          "v": 731
        },
        {
          "t": 183,
          "v": 733
        },
        {
          "t": 185,
          "v": 693
        },
        {
          "t": 187,
          "v": 693
        },
        {
          "t": 189,
          "v": 659
        },
        {
          "t": 191,
          "v": 666
        },
        {
          "t": 193,
          "v": 702
        },
        {
          "t": 195,
          "v": 705
        },
        {
          "t": 197,
          "v": 724
        },
        {
          "t": 199,
          "v": 725
        },
        {
          "t": 201,
          "v": 721
        },
        {
          "t": 203,
          "v": 721
        },
        {
          "t": 205,
          "v": 669
        },
        {
          "t": 207,
          "v": 701
        },
        {
          "t": 209,
          "v": 612
        },
        {
          "t": 211,
          "v": 612
        },
        {
          "t": 213,
          "v": 573
        },
        {
          "t": 215,
          "v": 598
        },
        {
          "t": 217,
          "v": 605
        },
        {
          "t": 219,
          "v": 609
        },
        {
          "t": 221,
          "v": 608
        },
        {
          "t": 223,
          "v": 650
        },
        {
          "t": 225,
          "v": 638
        },
        {
          "t": 227,
          "v": 663
        },
        {
          "t": 229,
          "v": 671
        },
        {
          "t": 231,
          "v": 700
        },
        {
          "t": 233,
          "v": 656
        },
        {
          "t": 235,
          "v": 1273
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
          "v": 6.83
        },
        {
          "t": 2,
          "v": 5.59
        },
        {
          "t": 4,
          "v": 4.58
        },
        {
          "t": 6,
          "v": 3.75
        },
        {
          "t": 8,
          "v": 3.07
        },
        {
          "t": 10,
          "v": 2.51
        },
        {
          "t": 12,
          "v": 2.06
        },
        {
          "t": 14,
          "v": 1.68
        },
        {
          "t": 16,
          "v": 1.38
        },
        {
          "t": 18,
          "v": 1.13
        },
        {
          "t": 20,
          "v": 0.92
        },
        {
          "t": 22,
          "v": 0.75
        },
        {
          "t": 24,
          "v": 0.62
        },
        {
          "t": 26,
          "v": 0.5
        },
        {
          "t": 28,
          "v": 0.59
        },
        {
          "t": 30,
          "v": 0.48
        },
        {
          "t": 32,
          "v": 0.39
        },
        {
          "t": 34,
          "v": 0.32
        },
        {
          "t": 36,
          "v": 0.26
        },
        {
          "t": 38,
          "v": 0.21
        },
        {
          "t": 40,
          "v": 0.17
        },
        {
          "t": 42,
          "v": 0.14
        },
        {
          "t": 44,
          "v": 0.11
        },
        {
          "t": 46,
          "v": 0.09
        },
        {
          "t": 48,
          "v": 0.07
        },
        {
          "t": 50,
          "v": 0.06
        },
        {
          "t": 52,
          "v": 0.05
        },
        {
          "t": 54,
          "v": 0.04
        },
        {
          "t": 57,
          "v": 0.03
        },
        {
          "t": 59,
          "v": 0.02
        },
        {
          "t": 61,
          "v": 0.02
        },
        {
          "t": 63,
          "v": 0.01
        },
        {
          "t": 65,
          "v": 0.01
        },
        {
          "t": 67,
          "v": 0.01
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
    "trace_artifact_url": "https://github.com/locustbaby/duotunnel/actions/runs/24184124275/artifacts/6346970921"
  }
};
