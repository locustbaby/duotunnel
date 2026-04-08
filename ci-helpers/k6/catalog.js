export const CATEGORY_DEFS = [
  {
    id: 'basic',
    label: 'Basic',
    description: 'Ramping rate with mixed protocols for baseline latency.',
  },
  {
    id: 'body_size',
    label: 'Body Size',
    description: 'Fixed rate with larger payloads to observe payload scaling.',
  },
  {
    id: 'stress',
    label: 'Stress',
    description: 'High-QPS stress stages for ingress and egress under pressure.',
  },
];

export const DUOTUNNEL_CASES = [
  {
    name: 'ingress_http_get',
    exec: 'ingressHttpGet',
    protocol: 'HTTP',
    direction: 'ingress',
    category: 'basic',
    durationSec: 25,
    includeInTotalRps: false,
    thresholds: { httpReqFailed: 'rate<0.05' },
    scenario: {
      executor: 'ramping-arrival-rate',
      startRate: 10,
      timeUnit: '1s',
      stages: [
        { target: 40, duration: '5s' },
        { target: 40, duration: '20s' },
      ],
      preAllocatedVUs: 5,
      maxVUs: 30,
    },
  },
  {
    name: 'ingress_http_post',
    exec: 'ingressHttpPost',
    protocol: 'HTTP',
    direction: 'ingress',
    category: 'basic',
    durationSec: 25,
    includeInTotalRps: false,
    thresholds: { httpReqFailed: 'rate<0.05' },
    scenario: {
      executor: 'ramping-arrival-rate',
      startRate: 5,
      timeUnit: '1s',
      startTime: '2s',
      stages: [
        { target: 25, duration: '5s' },
        { target: 25, duration: '20s' },
      ],
      preAllocatedVUs: 5,
      maxVUs: 20,
    },
  },
  {
    name: 'egress_http_get',
    exec: 'egressHttpGet',
    protocol: 'HTTP',
    direction: 'egress',
    category: 'basic',
    durationSec: 25,
    includeInTotalRps: false,
    thresholds: { httpReqFailed: 'rate<0.05' },
    scenario: {
      executor: 'ramping-arrival-rate',
      startRate: 10,
      timeUnit: '1s',
      startTime: '4s',
      stages: [
        { target: 50, duration: '5s' },
        { target: 50, duration: '20s' },
      ],
      preAllocatedVUs: 5,
      maxVUs: 40,
    },
  },
  {
    name: 'egress_http_post',
    exec: 'egressHttpPost',
    protocol: 'HTTP',
    direction: 'egress',
    category: 'basic',
    durationSec: 25,
    includeInTotalRps: false,
    thresholds: { httpReqFailed: 'rate<0.05' },
    scenario: {
      executor: 'ramping-arrival-rate',
      startRate: 5,
      timeUnit: '1s',
      startTime: '6s',
      stages: [
        { target: 40, duration: '5s' },
        { target: 40, duration: '20s' },
      ],
      preAllocatedVUs: 5,
      maxVUs: 30,
    },
  },
  {
    name: 'ws_ingress',
    exec: 'wsIngress',
    protocol: 'WS',
    direction: 'ingress',
    category: 'basic',
    durationSec: 20,
    includeInTotalRps: false,
    thresholds: { errCounter: 'count<50' },
    scenario: {
      executor: 'constant-arrival-rate',
      rate: 10,
      timeUnit: '1s',
      duration: '20s',
      startTime: '3s',
      preAllocatedVUs: 5,
      maxVUs: 20,
    },
  },
  {
    name: 'grpc_health_ingress',
    exec: 'grpcHealthIngress',
    protocol: 'gRPC',
    direction: 'ingress',
    category: 'basic',
    durationSec: 20,
    includeInTotalRps: false,
    scenario: {
      executor: 'constant-arrival-rate',
      rate: 20,
      timeUnit: '1s',
      duration: '20s',
      startTime: '5s',
      preAllocatedVUs: 3,
      maxVUs: 15,
    },
  },
  {
    name: 'grpc_echo_ingress',
    exec: 'grpcEchoIngress',
    protocol: 'gRPC',
    direction: 'ingress',
    category: 'basic',
    durationSec: 20,
    includeInTotalRps: false,
    scenario: {
      executor: 'constant-arrival-rate',
      rate: 20,
      timeUnit: '1s',
      duration: '20s',
      startTime: '7s',
      preAllocatedVUs: 3,
      maxVUs: 15,
    },
  },
  {
    name: 'bidir_mixed',
    exec: 'bidirectional',
    protocol: 'HTTP',
    direction: 'bidir',
    category: 'basic',
    durationSec: 20,
    includeInTotalRps: false,
    thresholds: { httpReqFailed: 'rate<0.05' },
    scenario: {
      executor: 'ramping-arrival-rate',
      startRate: 5,
      timeUnit: '1s',
      startTime: '8s',
      stages: [
        { target: 20, duration: '5s' },
        { target: 20, duration: '15s' },
      ],
      preAllocatedVUs: 5,
      maxVUs: 20,
    },
  },
  {
    name: 'ingress_post_1k',
    exec: 'ingressPost1K',
    protocol: 'HTTP',
    direction: 'ingress',
    category: 'body_size',
    durationSec: 20,
    includeInTotalRps: false,
    thresholds: { httpReqFailed: 'rate<0.05' },
    scenario: {
      executor: 'constant-arrival-rate',
      rate: 30,
      timeUnit: '1s',
      duration: '20s',
      startTime: '35s',
      preAllocatedVUs: 5,
      maxVUs: 20,
    },
  },
  {
    name: 'ingress_post_10k',
    exec: 'ingressPost10K',
    protocol: 'HTTP',
    direction: 'ingress',
    category: 'body_size',
    durationSec: 20,
    includeInTotalRps: false,
    thresholds: { httpReqFailed: 'rate<0.05' },
    scenario: {
      executor: 'constant-arrival-rate',
      rate: 20,
      timeUnit: '1s',
      duration: '20s',
      startTime: '35s',
      preAllocatedVUs: 5,
      maxVUs: 20,
    },
  },
  {
    name: 'ingress_post_100k',
    exec: 'ingressPost100K',
    protocol: 'HTTP',
    direction: 'ingress',
    category: 'body_size',
    durationSec: 20,
    includeInTotalRps: false,
    thresholds: { httpReqFailed: 'rate<0.05' },
    scenario: {
      executor: 'constant-arrival-rate',
      rate: 10,
      timeUnit: '1s',
      duration: '20s',
      startTime: '35s',
      preAllocatedVUs: 5,
      maxVUs: 20,
    },
  },
  {
    name: 'egress_post_10k',
    exec: 'egressPost10K',
    protocol: 'HTTP',
    direction: 'egress',
    category: 'body_size',
    durationSec: 20,
    includeInTotalRps: false,
    thresholds: { httpReqFailed: 'rate<0.05' },
    scenario: {
      executor: 'constant-arrival-rate',
      rate: 20,
      timeUnit: '1s',
      duration: '20s',
      startTime: '37s',
      preAllocatedVUs: 5,
      maxVUs: 20,
    },
  },
  {
    name: 'ws_multi_msg',
    exec: 'wsMultiMsg',
    protocol: 'WS',
    direction: 'ingress',
    category: 'basic',
    durationSec: 20,
    includeInTotalRps: false,
    thresholds: { errCounter: 'count<50' },
    scenario: {
      executor: 'constant-arrival-rate',
      rate: 5,
      timeUnit: '1s',
      duration: '20s',
      startTime: '36s',
      preAllocatedVUs: 5,
      maxVUs: 20,
    },
  },
  {
    name: 'grpc_large_payload',
    exec: 'grpcLargePayload',
    protocol: 'gRPC',
    direction: 'ingress',
    category: 'body_size',
    durationSec: 20,
    includeInTotalRps: false,
    scenario: {
      executor: 'constant-arrival-rate',
      rate: 15,
      timeUnit: '1s',
      duration: '20s',
      startTime: '38s',
      preAllocatedVUs: 3,
      maxVUs: 15,
    },
  },
  {
    name: 'grpc_high_qps',
    exec: 'grpcHighQps',
    protocol: 'gRPC',
    direction: 'ingress',
    category: 'stress',
    durationSec: 20,
    includeInTotalRps: true,
    scenario: {
      executor: 'ramping-arrival-rate',
      startRate: 20,
      timeUnit: '1s',
      startTime: '39s',
      stages: [
        { target: 100, duration: '5s' },
        { target: 100, duration: '15s' },
      ],
      preAllocatedVUs: 5,
      maxVUs: 30,
    },
  },
  {
    name: 'ingress_3000qps',
    exec: 'ingressHttpGet',
    protocol: 'HTTP',
    direction: 'ingress',
    category: 'stress',
    durationSec: 20,
    includeInTotalRps: true,
    scenario: {
      executor: 'constant-arrival-rate',
      rate: 3000,
      timeUnit: '1s',
      duration: '20s',
      startTime: '65s',
      preAllocatedVUs: 150,
      maxVUs: 300,
    },
  },
  {
    name: 'egress_3000qps',
    exec: 'egressHttpGet',
    protocol: 'HTTP',
    direction: 'egress',
    category: 'stress',
    durationSec: 20,
    includeInTotalRps: true,
    scenario: {
      executor: 'constant-arrival-rate',
      rate: 3000,
      timeUnit: '1s',
      duration: '20s',
      startTime: '90s',
      preAllocatedVUs: 150,
      maxVUs: 300,
    },
  },
  {
    name: 'ingress_multihost',
    exec: 'ingressMultihost',
    protocol: 'HTTP',
    direction: 'ingress',
    category: 'stress',
    durationSec: 20,
    includeInTotalRps: true,
    scenario: {
      executor: 'constant-arrival-rate',
      rate: 3000,
      timeUnit: '1s',
      duration: '20s',
      startTime: '115s',
      preAllocatedVUs: 200,
      maxVUs: 350,
    },
  },
  {
    name: 'egress_multihost',
    exec: 'egressMultihost',
    protocol: 'HTTP',
    direction: 'egress',
    category: 'stress',
    durationSec: 20,
    includeInTotalRps: true,
    scenario: {
      executor: 'constant-arrival-rate',
      rate: 3000,
      timeUnit: '1s',
      duration: '20s',
      startTime: '140s',
      preAllocatedVUs: 200,
      maxVUs: 350,
    },
  },
  {
    name: 'ingress_8000qps',
    exec: 'ingressHttpGet',
    protocol: 'HTTP',
    direction: 'ingress',
    category: 'stress',
    durationSec: 20,
    includeInTotalRps: true,
    scenario: {
      executor: 'constant-arrival-rate',
      rate: 8000,
      timeUnit: '1s',
      duration: '20s',
      startTime: '165s',
      preAllocatedVUs: 700,
      maxVUs: 900,
    },
  },
  {
    name: 'egress_8000qps',
    exec: 'egressHttpGet',
    protocol: 'HTTP',
    direction: 'egress',
    category: 'stress',
    durationSec: 20,
    includeInTotalRps: true,
    scenario: {
      executor: 'constant-arrival-rate',
      rate: 8000,
      timeUnit: '1s',
      duration: '20s',
      startTime: '190s',
      preAllocatedVUs: 700,
      maxVUs: 900,
    },
  },
  {
    name: 'ingress_multihost_8000qps',
    exec: 'ingressMultihost',
    protocol: 'HTTP',
    direction: 'ingress',
    category: 'stress',
    durationSec: 20,
    includeInTotalRps: true,
    scenario: {
      executor: 'constant-arrival-rate',
      rate: 8000,
      timeUnit: '1s',
      duration: '20s',
      startTime: '215s',
      preAllocatedVUs: 800,
      maxVUs: 1000,
    },
  },
  {
    name: 'egress_multihost_8000qps',
    exec: 'egressMultihost',
    protocol: 'HTTP',
    direction: 'egress',
    category: 'stress',
    durationSec: 20,
    includeInTotalRps: true,
    scenario: {
      executor: 'constant-arrival-rate',
      rate: 8000,
      timeUnit: '1s',
      duration: '20s',
      startTime: '240s',
      preAllocatedVUs: 800,
      maxVUs: 1000,
    },
  },
];

export const DUOTUNNEL_PHASES = [
  {
    name: 'Basic',
    start: 0,
    end: 29,
    scenarios: [
      'ingress_http_get',
      'ingress_http_post',
      'egress_http_get',
      'egress_http_post',
      'ws_ingress',
      'grpc_health_ingress',
      'grpc_echo_ingress',
      'bidir_mixed',
    ],
  },
  {
    name: 'Body/Payload',
    start: 35,
    end: 59,
    scenarios: [
      'ingress_post_1k',
      'ingress_post_10k',
      'ingress_post_100k',
      'egress_post_10k',
      'ws_multi_msg',
      'grpc_large_payload',
      'grpc_high_qps',
    ],
  },
  { name: 'Ingress 3K', start: 65, end: 85, scenarios: ['ingress_3000qps'] },
  { name: 'Egress 3K', start: 90, end: 110, scenarios: ['egress_3000qps'] },
  { name: 'Ingress multihost 3K', start: 115, end: 135, scenarios: ['ingress_multihost'] },
  { name: 'Egress multihost 3K', start: 140, end: 160, scenarios: ['egress_multihost'] },
  { name: 'Ingress 8K', start: 165, end: 185, scenarios: ['ingress_8000qps'] },
  { name: 'Egress 8K', start: 190, end: 210, scenarios: ['egress_8000qps'] },
  { name: 'Ingress multihost 8K', start: 215, end: 235, scenarios: ['ingress_multihost_8000qps'] },
  { name: 'Egress multihost 8K', start: 240, end: 260, scenarios: ['egress_multihost_8000qps'] },
];

export const FRP_CASES = [
  {
    name: 'ingress_3000qps',
    exec: 'ingressHttpGetKeepalive',
    protocol: 'HTTP',
    direction: 'ingress',
    category: 'stress',
    durationSec: 20,
    includeInTotalRps: true,
    thresholds: { httpReqFailed: 'rate<0.20' },
    scenario: {
      executor: 'constant-arrival-rate',
      rate: 3000,
      timeUnit: '1s',
      duration: '20s',
      startTime: '0s',
      preAllocatedVUs: 50,
      maxVUs: 500,
    },
  },
  {
    name: 'ingress_multihost',
    exec: 'ingressMultihost',
    protocol: 'HTTP',
    direction: 'ingress',
    category: 'stress',
    durationSec: 20,
    includeInTotalRps: true,
    thresholds: { httpReqFailed: 'rate<0.20' },
    scenario: {
      executor: 'constant-arrival-rate',
      rate: 3000,
      timeUnit: '1s',
      duration: '20s',
      startTime: '25s',
      preAllocatedVUs: 50,
      maxVUs: 500,
    },
  },
  {
    name: 'ingress_multihost_8000qps',
    exec: 'ingressMultihost',
    protocol: 'HTTP',
    direction: 'ingress',
    category: 'stress',
    durationSec: 20,
    includeInTotalRps: true,
    thresholds: { httpReqFailed: 'rate<0.20' },
    scenario: {
      executor: 'constant-arrival-rate',
      rate: 8000,
      timeUnit: '1s',
      duration: '20s',
      startTime: '50s',
      preAllocatedVUs: 100,
      maxVUs: 500,
    },
  },
];

export const FRP_PHASES = [
  { name: '3K KL (frp)', start: 0, end: 20, scenarios: ['ingress_3000qps'] },
  { name: '3K Multihost (frp)', start: 25, end: 45, scenarios: ['ingress_multihost'] },
  { name: '8K Multihost (frp)', start: 50, end: 70, scenarios: ['ingress_multihost_8000qps'] },
];

function round2(v) {
  return Math.round(v * 100) / 100;
}

function metricForProtocol(protocol) {
  if (protocol === 'gRPC') return 'grpc_req_duration';
  if (protocol === 'WS') return 'ws_session_duration';
  return 'http_req_duration';
}

export function buildScenarios(caseDefs) {
  const out = {};
  for (const c of caseDefs) {
    out[c.name] = { ...c.scenario, exec: c.exec };
  }
  return out;
}

export function buildThresholds(caseDefs) {
  const thresholds = {};
  for (const c of caseDefs) {
    const metric = c.metric || metricForProtocol(c.protocol);
    const durationThreshold = c.thresholds && c.thresholds.duration ? c.thresholds.duration : 'p(95)<60000';
    thresholds[`${metric}{name:${c.name}}`] = [durationThreshold];
    if (c.thresholds && c.thresholds.httpReqFailed) {
      thresholds[`http_req_failed{scenario:${c.name}}`] = [c.thresholds.httpReqFailed];
    }
    if (c.thresholds && c.thresholds.errCounter) {
      thresholds[`c_err_${c.name}`] = [c.thresholds.errCounter];
    }
  }
  return thresholds;
}

export function buildCounters(caseDefs, Counter) {
  const reqCounters = {};
  const errCounters = {};
  for (const c of caseDefs) {
    reqCounters[c.name] = new Counter(`c_reqs_${c.name}`);
    errCounters[c.name] = new Counter(`c_err_${c.name}`);
  }
  return { reqCounters, errCounters };
}

function metricByName(metrics, base, name) {
  const direct = metrics[`${base}{name:${name}}`];
  if (direct) return direct;
  const needle = `name:${name}`;
  for (const [k, v] of Object.entries(metrics)) {
    if (k.startsWith(`${base}{`) && k.includes(needle)) return v;
  }
  return null;
}

function resolveCategories(caseDefs) {
  const seen = new Set();
  const defs = [];
  const catalogById = {};
  for (const c of CATEGORY_DEFS) catalogById[c.id] = c;
  for (const c of caseDefs) {
    if (seen.has(c.category)) continue;
    seen.add(c.category);
    if (catalogById[c.category]) {
      defs.push(catalogById[c.category]);
    } else {
      defs.push({ id: c.category, label: c.category, description: '' });
    }
  }
  return defs;
}

export function buildSummaryOutput(data, caseDefs, phases, extras = {}) {
  const metrics = data.metrics || {};
  const scenarios = [];
  let totalRPS = 0;
  let totalRequests = 0;
  let totalErrors = 0;

  for (const c of caseDefs) {
    const metric = c.metric || metricForProtocol(c.protocol);
    const trend = metricByName(metrics, metric, c.name);
    if (!trend || !trend.values) continue;

    const values = trend.values;
    const p50 = round2(values.med || 0);
    const p95 = round2(values['p(95)'] || 0);
    const p99 = round2(values['p(99)'] || 0);

    const reqMetric = metrics[`c_reqs_${c.name}`];
    const errMetric = metrics[`c_err_${c.name}`];

    const requests = reqMetric && reqMetric.values ? reqMetric.values.count || 0 : 0;
    const errors = errMetric && errMetric.values ? errMetric.values.count || 0 : 0;
    const rps = c.durationSec ? round2(requests / c.durationSec) : 0;
    const err = requests > 0 ? round2((errors / requests) * 100) : 0;

    totalRequests += requests;
    totalErrors += errors;
    if (c.includeInTotalRps) totalRPS += rps;

    scenarios.push({
      name: c.name,
      protocol: c.protocol,
      direction: c.direction,
      category: c.category,
      includeInTotalRps: !!c.includeInTotalRps,
      p50,
      p95,
      p99,
      err,
      rps,
      requests,
    });
  }

  const totalErr = totalRequests > 0 ? round2((totalErrors / totalRequests) * 100) : 0;

  return {
    timestamp: new Date().toISOString(),
    commit: __ENV.GITHUB_SHA || 'local',
    scenarios,
    summary: {
      totalRPS: round2(totalRPS),
      totalErr,
      totalRequests,
    },
    phases,
    catalog: {
      categories: resolveCategories(caseDefs),
      caseOrder: caseDefs.map((c) => c.name),
    },
    ...extras,
  };
}
