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

// ---------------------------------------------------------------------------
// CaseDef schema (each field documented):
//
//   name            — unique scenario id (used as k6 scenario key and metric tag)
//   exec            — k6 exported function name
//   protocol        — 'HTTP' | 'WS' | 'gRPC'
//   direction       — 'ingress' | 'egress' | 'bidir'
//   category        — category id; must match a CATEGORY_DEFS entry
//   durationSec     — how long the scenario runs (used for RPS calculation)
//   includeInTotalRps — whether this case counts toward summary Total RPS
//   label           — short human-readable display name (auto-derived if omitted)
//   payloadBytes    — payload size in bytes (body_size cases only)
//   thresholds      — k6 threshold config:
//                       duration:       p(95) latency threshold string (default 'p(95)<60000')
//                       httpReqFailed:  error rate threshold (e.g. 'rate<0.05')
//                       errCounter:     absolute error count threshold (e.g. 'count<50')
//   scenario        — k6 scenario executor config (startTime, executor, rate/stages, VUs)
//
// Derived at runtime (by enrichCaseDefs):
//   timeRange       — { startSec, endSec } absolute seconds from start of k6 run
//   thresholdSpec   — human-readable threshold summary string for display
//   phase           — name of the PHASE this case belongs to (or null)
// ---------------------------------------------------------------------------

export const DUOTUNNEL_CASES = [
  {
    name: 'ingress_http_get',
    label: 'ingress GET ramp→40 rps',
    exec: 'ingressHttpGet',
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
      stages: [
        { target: 40, duration: '5s' },
        { target: 40, duration: '20s' },
      ],
      preAllocatedVUs: 30,
      maxVUs: 200,
    },
  },
  {
    name: 'ingress_http_post',
    label: 'ingress POST ramp→25 rps',
    exec: 'ingressHttpPost',
    protocol: 'HTTP',
    direction: 'ingress',
    category: 'basic',
    durationSec: 25,
    includeInTotalRps: false,
    thresholds: { httpReqFailed: 'rate<0.05' },
    scenario: {
      executor: 'ramping-arrival-rate',
      startRate: 3,
      timeUnit: '1s',
      startTime: '2s',
      stages: [
        { target: 25, duration: '5s' },
        { target: 25, duration: '20s' },
      ],
      preAllocatedVUs: 20,
      maxVUs: 150,
    },
  },
  {
    name: 'egress_http_get',
    label: 'egress GET ramp→50 rps',
    exec: 'egressHttpGet',
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
      startTime: '4s',
      stages: [
        { target: 50, duration: '5s' },
        { target: 50, duration: '20s' },
      ],
      preAllocatedVUs: 30,
      maxVUs: 250,
    },
  },
  {
    name: 'egress_http_post',
    label: 'egress POST ramp→40 rps',
    exec: 'egressHttpPost',
    protocol: 'HTTP',
    direction: 'egress',
    category: 'basic',
    durationSec: 25,
    includeInTotalRps: false,
    thresholds: { httpReqFailed: 'rate<0.05' },
    scenario: {
      executor: 'ramping-arrival-rate',
      startRate: 3,
      timeUnit: '1s',
      startTime: '6s',
      stages: [
        { target: 40, duration: '5s' },
        { target: 40, duration: '20s' },
      ],
      preAllocatedVUs: 20,
      maxVUs: 200,
    },
  },
  {
    name: 'ws_ingress',
    label: 'ingress WS echo 10 rps',
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
      preAllocatedVUs: 30,
      maxVUs: 150,
    },
  },
  {
    name: 'grpc_health_ingress',
    label: 'ingress gRPC health 20 rps',
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
      preAllocatedVUs: 20,
      maxVUs: 100,
    },
  },
  {
    name: 'grpc_echo_ingress',
    label: 'ingress gRPC echo 20 rps',
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
      preAllocatedVUs: 20,
      maxVUs: 100,
    },
  },
  {
    name: 'bidir_mixed',
    label: 'bidir HTTP ramp→20 rps',
    exec: 'bidirectional',
    protocol: 'HTTP',
    direction: 'bidir',
    category: 'basic',
    durationSec: 20,
    includeInTotalRps: false,
    thresholds: { httpReqFailed: 'rate<0.05' },
    scenario: {
      executor: 'ramping-arrival-rate',
      startRate: 3,
      timeUnit: '1s',
      startTime: '8s',
      stages: [
        { target: 20, duration: '5s' },
        { target: 20, duration: '15s' },
      ],
      preAllocatedVUs: 20,
      maxVUs: 150,
    },
  },
  {
    name: 'ingress_post_1k',
    label: 'ingress POST 1 KB @ 30 rps',
    exec: 'ingressPost1K',
    protocol: 'HTTP',
    direction: 'ingress',
    category: 'body_size',
    durationSec: 20,
    payloadBytes: 1024,
    includeInTotalRps: false,
    thresholds: { httpReqFailed: 'rate<0.05' },
    scenario: {
      executor: 'constant-arrival-rate',
      rate: 30,
      timeUnit: '1s',
      duration: '20s',
      startTime: '35s',
      preAllocatedVUs: 100,
      maxVUs: 300,
    },
  },
  {
    name: 'ingress_post_10k',
    label: 'ingress POST 10 KB @ 20 rps',
    exec: 'ingressPost10K',
    protocol: 'HTTP',
    direction: 'ingress',
    category: 'body_size',
    durationSec: 20,
    payloadBytes: 10240,
    includeInTotalRps: false,
    thresholds: { httpReqFailed: 'rate<0.05' },
    scenario: {
      executor: 'constant-arrival-rate',
      rate: 20,
      timeUnit: '1s',
      duration: '20s',
      startTime: '35s',
      preAllocatedVUs: 80,
      maxVUs: 250,
    },
  },
  {
    name: 'ingress_post_100k',
    label: 'ingress POST 100 KB @ 10 rps',
    exec: 'ingressPost100K',
    protocol: 'HTTP',
    direction: 'ingress',
    category: 'body_size',
    durationSec: 20,
    payloadBytes: 102400,
    includeInTotalRps: false,
    thresholds: { httpReqFailed: 'rate<0.05' },
    scenario: {
      executor: 'constant-arrival-rate',
      rate: 10,
      timeUnit: '1s',
      duration: '20s',
      startTime: '35s',
      preAllocatedVUs: 50,
      maxVUs: 200,
    },
  },
  {
    name: 'egress_post_10k',
    label: 'egress POST 10 KB @ 20 rps',
    exec: 'egressPost10K',
    protocol: 'HTTP',
    direction: 'egress',
    category: 'body_size',
    durationSec: 20,
    payloadBytes: 10240,
    includeInTotalRps: false,
    thresholds: { httpReqFailed: 'rate<0.05' },
    scenario: {
      executor: 'constant-arrival-rate',
      rate: 20,
      timeUnit: '1s',
      duration: '20s',
      startTime: '37s',
      preAllocatedVUs: 80,
      maxVUs: 250,
    },
  },
  {
    name: 'ws_multi_msg',
    label: 'ingress WS burst 20msg @ 5 rps',
    exec: 'wsMultiMsg',
    protocol: 'WS',
    direction: 'ingress',
    category: 'body_size',
    durationSec: 20,
    includeInTotalRps: false,
    thresholds: { errCounter: 'count<50' },
    scenario: {
      executor: 'constant-arrival-rate',
      rate: 5,
      timeUnit: '1s',
      duration: '20s',
      startTime: '36s',
      preAllocatedVUs: 30,
      maxVUs: 100,
    },
  },
  {
    name: 'grpc_large_payload',
    label: 'ingress gRPC 10 KB @ 15 rps',
    exec: 'grpcLargePayload',
    protocol: 'gRPC',
    direction: 'ingress',
    category: 'body_size',
    durationSec: 20,
    payloadBytes: 10240,
    includeInTotalRps: false,
    scenario: {
      executor: 'constant-arrival-rate',
      rate: 15,
      timeUnit: '1s',
      duration: '20s',
      startTime: '38s',
      preAllocatedVUs: 20,
      maxVUs: 80,
    },
  },
  {
    name: 'grpc_high_qps',
    label: 'ingress gRPC ramp→100 rps',
    exec: 'grpcHighQps',
    protocol: 'gRPC',
    direction: 'ingress',
    category: 'body_size',
    durationSec: 20,
    includeInTotalRps: true,
    scenario: {
      executor: 'ramping-arrival-rate',
      startRate: 10,
      timeUnit: '1s',
      startTime: '39s',
      stages: [
        { target: 100, duration: '5s' },
        { target: 100, duration: '15s' },
      ],
      preAllocatedVUs: 50,
      maxVUs: 200,
    },
  },
  {
    name: 'ingress_3000qps',
    label: 'ingress GET 3000 rps',
    exec: 'ingressHttpGet',
    protocol: 'HTTP',
    direction: 'ingress',
    category: 'stress',
    durationSec: 20,
    includeInTotalRps: true,
    thresholds: { httpReqFailed: 'rate<0.05' },
    scenario: {
      executor: 'constant-arrival-rate',
      rate: 3000,
      timeUnit: '1s',
      duration: '20s',
      startTime: '65s',
      preAllocatedVUs: 100,
      maxVUs: 500,
    },
  },
  {
    name: 'egress_3000qps',
    label: 'egress GET 3000 rps',
    exec: 'egressHttpGet',
    protocol: 'HTTP',
    direction: 'egress',
    category: 'stress',
    durationSec: 20,
    includeInTotalRps: true,
    thresholds: { httpReqFailed: 'rate<0.05' },
    scenario: {
      executor: 'constant-arrival-rate',
      rate: 3000,
      timeUnit: '1s',
      duration: '20s',
      startTime: '90s',
      preAllocatedVUs: 100,
      maxVUs: 500,
    },
  },
  {
    name: 'ingress_multihost',
    label: 'ingress multihost 3000 rps',
    exec: 'ingressMultihost',
    protocol: 'HTTP',
    direction: 'ingress',
    category: 'stress',
    durationSec: 20,
    includeInTotalRps: true,
    thresholds: { httpReqFailed: 'rate<0.05' },
    scenario: {
      executor: 'constant-arrival-rate',
      rate: 3000,
      timeUnit: '1s',
      duration: '20s',
      startTime: '115s',
      preAllocatedVUs: 100,
      maxVUs: 500,
    },
  },
  {
    name: 'egress_multihost',
    label: 'egress multihost 3000 rps',
    exec: 'egressMultihost',
    protocol: 'HTTP',
    direction: 'egress',
    category: 'stress',
    durationSec: 20,
    includeInTotalRps: true,
    thresholds: { httpReqFailed: 'rate<0.05' },
    scenario: {
      executor: 'constant-arrival-rate',
      rate: 3000,
      timeUnit: '1s',
      duration: '20s',
      startTime: '140s',
      preAllocatedVUs: 100,
      maxVUs: 500,
    },
  },
  {
    name: 'ingress_multihost_6000qps',
    label: 'ingress multihost 6000 rps',
    exec: 'ingressMultihost',
    protocol: 'HTTP',
    direction: 'ingress',
    category: 'stress',
    durationSec: 20,
    includeInTotalRps: true,
    thresholds: { httpReqFailed: 'rate<0.05' },
    scenario: {
      executor: 'constant-arrival-rate',
      rate: 6000,
      timeUnit: '1s',
      duration: '20s',
      startTime: '165s',
      preAllocatedVUs: 100,
      maxVUs: 2000,
    },
  },
  {
    name: 'egress_multihost_6000qps',
    label: 'egress multihost 6000 rps',
    exec: 'egressMultihost',
    protocol: 'HTTP',
    direction: 'egress',
    category: 'stress',
    durationSec: 20,
    includeInTotalRps: true,
    thresholds: { httpReqFailed: 'rate<0.05' },
    scenario: {
      executor: 'constant-arrival-rate',
      rate: 6000,
      timeUnit: '1s',
      duration: '20s',
      startTime: '190s',
      preAllocatedVUs: 100,
      maxVUs: 2000,
    },
  },
  {
    name: 'ingress_8000qps',
    label: 'ingress GET 8000 rps',
    exec: 'ingressHttpGet',
    protocol: 'HTTP',
    direction: 'ingress',
    category: 'stress',
    durationSec: 20,
    includeInTotalRps: true,
    thresholds: { httpReqFailed: 'rate<0.05' },
    scenario: {
      executor: 'constant-arrival-rate',
      rate: 8000,
      timeUnit: '1s',
      duration: '20s',
      startTime: '215s',
      preAllocatedVUs: 100,
      maxVUs: 3000,
    },
  },
  {
    name: 'egress_8000qps',
    label: 'egress GET 8000 rps',
    exec: 'egressHttpGet',
    protocol: 'HTTP',
    direction: 'egress',
    category: 'stress',
    durationSec: 20,
    includeInTotalRps: true,
    thresholds: { httpReqFailed: 'rate<0.05' },
    scenario: {
      executor: 'constant-arrival-rate',
      rate: 8000,
      timeUnit: '1s',
      duration: '20s',
      startTime: '240s',
      preAllocatedVUs: 100,
      maxVUs: 3000,
    },
  },
  {
    name: 'ingress_multihost_8000qps',
    label: 'ingress multihost 8000 rps',
    exec: 'ingressMultihost',
    protocol: 'HTTP',
    direction: 'ingress',
    category: 'stress',
    durationSec: 20,
    includeInTotalRps: true,
    thresholds: { httpReqFailed: 'rate<0.05' },
    scenario: {
      executor: 'constant-arrival-rate',
      rate: 8000,
      timeUnit: '1s',
      duration: '20s',
      startTime: '265s',
      preAllocatedVUs: 100,
      maxVUs: 3000,
    },
  },
  {
    name: 'egress_multihost_8000qps',
    label: 'egress multihost 8000 rps',
    exec: 'egressMultihost',
    protocol: 'HTTP',
    direction: 'egress',
    category: 'stress',
    durationSec: 20,
    includeInTotalRps: true,
    thresholds: { httpReqFailed: 'rate<0.05' },
    scenario: {
      executor: 'constant-arrival-rate',
      rate: 8000,
      timeUnit: '1s',
      duration: '20s',
      startTime: '290s',
      preAllocatedVUs: 100,
      maxVUs: 3000,
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
  { name: 'Ingress multihost 6K', start: 165, end: 185, scenarios: ['ingress_multihost_6000qps'] },
  { name: 'Egress multihost 6K', start: 190, end: 210, scenarios: ['egress_multihost_6000qps'] },
  { name: 'Ingress 8K', start: 215, end: 235, scenarios: ['ingress_8000qps'] },
  { name: 'Egress 8K', start: 240, end: 260, scenarios: ['egress_8000qps'] },
  { name: 'Ingress multihost 8K', start: 265, end: 285, scenarios: ['ingress_multihost_8000qps'] },
  { name: 'Egress multihost 8K', start: 290, end: 310, scenarios: ['egress_multihost_8000qps'] },
];

export const FRP_CASES = [
  {
    name: 'frp_ingress_3000qps',
    label: 'frp ingress GET 3000 rps',
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
      preAllocatedVUs: 100,
      maxVUs: 500,
    },
  },
  {
    name: 'frp_ingress_multihost',
    label: 'frp ingress multihost 3000 rps',
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
      preAllocatedVUs: 100,
      maxVUs: 500,
    },
  },
  {
    name: 'frp_ingress_multihost_6000qps',
    label: 'frp ingress multihost 6000 rps',
    exec: 'ingressMultihost',
    protocol: 'HTTP',
    direction: 'ingress',
    category: 'stress',
    durationSec: 20,
    includeInTotalRps: true,
    thresholds: { httpReqFailed: 'rate<0.20' },
    scenario: {
      executor: 'constant-arrival-rate',
      rate: 6000,
      timeUnit: '1s',
      duration: '20s',
      startTime: '50s',
      preAllocatedVUs: 100,
      maxVUs: 2000,
    },
  },
  {
    name: 'frp_ingress_multihost_8000qps',
    label: 'frp ingress multihost 8000 rps',
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
      startTime: '75s',
      preAllocatedVUs: 100,
      maxVUs: 3000,
    },
  },
];

export const FRP_PHASES = [
  { name: '3K KL (frp)', start: 0, end: 20, scenarios: ['frp_ingress_3000qps'] },
  { name: '3K Multihost (frp)', start: 25, end: 45, scenarios: ['frp_ingress_multihost'] },
  { name: '6K Multihost (frp)', start: 50, end: 70, scenarios: ['frp_ingress_multihost_6000qps'] },
  { name: '8K Multihost (frp)', start: 75, end: 95, scenarios: ['frp_ingress_multihost_8000qps'] },
];

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

function round2(v) {
  return Math.round(v * 100) / 100;
}

function metricForProtocol(protocol) {
  if (protocol === 'gRPC') return 'grpc_req_duration';
  if (protocol === 'WS') return 'ws_session_duration';
  return 'http_req_duration';
}

function parseSeconds(v) {
  if (v === undefined || v === null) return 0;
  if (typeof v === 'number') return v;
  const s = String(v).trim();
  if (s.endsWith('s')) return Number(s.slice(0, -1)) || 0;
  return Number(s) || 0;
}

// Build a human-readable threshold spec string for display.
function buildThresholdSpec(c) {
  const parts = [];
  const dur = (c.thresholds && c.thresholds.duration) || 'p(95)<60000';
  parts.push(dur.replace('p(95)<', 'p95<').replace('p(99)<', 'p99<'));
  if (c.thresholds && c.thresholds.httpReqFailed) parts.push(`err ${c.thresholds.httpReqFailed}`);
  if (c.thresholds && c.thresholds.errCounter) parts.push(`errs ${c.thresholds.errCounter}`);
  return parts.join(', ');
}

// Build a phase lookup map: case name -> phase name.
function buildPhaseMap(phases) {
  const map = {};
  for (const p of phases) {
    for (const s of (p.scenarios || [])) {
      map[s] = p.name;
    }
  }
  return map;
}

// Enrich caseDefs with derived fields: timeRange, thresholdSpec, phase.
// This is called once after filtering/normalising start times.
export function enrichCaseDefs(caseDefs, phases) {
  const phaseMap = buildPhaseMap(phases);
  return caseDefs.map((c) => {
    const startSec = parseSeconds(c.scenario && c.scenario.startTime);
    const endSec = startSec + (c.durationSec || 0);
    return {
      ...c,
      timeRange: { startSec, endSec },
      thresholdSpec: buildThresholdSpec(c),
      phase: phaseMap[c.name] || null,
    };
  });
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
      // identity
      name: c.name,
      protocol: c.protocol,
      direction: c.direction,
      category: c.category,
      phase: c.phase || null,
      // display metadata derived from CaseDef
      label: c.label || null,
      payloadBytes: c.payloadBytes || null,
      timeRange: c.timeRange || null,
      thresholdSpec: c.thresholdSpec || null,
      includeInTotalRps: !!c.includeInTotalRps,
      // configured target rate (null for ramping cases or when not overridden)
      targetRate: c.targetRate != null ? c.targetRate : (c.scenario?.rate || null),
      // results
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
