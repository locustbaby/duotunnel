window.BENCHMARK_DATA = {
  "lastUpdate": 1772527379038,
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
          "id": "d15137a60826601d4ebdf1c09c919e6328f61c53",
          "message": "ci: consolidate benchmark metrics — group by scenario type\n\nReduce from 44 flat metrics to ~17 grouped metrics:\n- HTTP basic: avg of GET/POST ingress+egress\n- gRPC/WS: avg across sub-scenarios\n- Body size: 1K/10K/100K p95\n- QPS tiers: 1000/2000/3000 with p50+p95+err%\n- Overall: total RPS + error rate\n\nCo-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>",
          "timestamp": "2026-03-03T16:39:10+08:00",
          "tree_id": "d58e9e41c18ab05fed60eeef7a435cfabc47f20e",
          "url": "https://github.com/locustbaby/duotunnel/commit/d15137a60826601d4ebdf1c09c919e6328f61c53"
        },
        "date": 1772527378747,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "HTTP basic p50",
            "value": 0.55,
            "unit": "ms"
          },
          {
            "name": "HTTP basic p95",
            "value": 0.9,
            "unit": "ms"
          },
          {
            "name": "gRPC p50",
            "value": 2,
            "unit": "ms"
          },
          {
            "name": "gRPC p95",
            "value": 42,
            "unit": "ms"
          },
          {
            "name": "WS p50",
            "value": 22,
            "unit": "ms"
          },
          {
            "name": "WS p95",
            "value": 23,
            "unit": "ms"
          },
          {
            "name": "body 1K p95",
            "value": 1.81,
            "unit": "ms"
          },
          {
            "name": "body 10K p95",
            "value": 2.46,
            "unit": "ms"
          },
          {
            "name": "body 100K p95",
            "value": 42.71,
            "unit": "ms"
          },
          {
            "name": "1000 QPS p50",
            "value": 0.66,
            "unit": "ms"
          },
          {
            "name": "1000 QPS p95",
            "value": 1.04,
            "unit": "ms"
          },
          {
            "name": "2000 QPS p50",
            "value": 1.08,
            "unit": "ms"
          },
          {
            "name": "2000 QPS p95",
            "value": 5.23,
            "unit": "ms"
          },
          {
            "name": "3000 QPS p50",
            "value": 13.19,
            "unit": "ms"
          },
          {
            "name": "3000 QPS p95",
            "value": 46.03,
            "unit": "ms"
          },
          {
            "name": "bidir p95",
            "value": 2,
            "unit": "ms"
          },
          {
            "name": "total RPS",
            "value": 1543.19,
            "unit": "req/s"
          },
          {
            "name": "total err",
            "value": 0,
            "unit": "%"
          }
        ]
      }
    ]
  }
}