import http from 'k6/http';
import grpc from 'k6/net/grpc';
import ws from 'k6/ws';
import exec from 'k6/execution';
import { check } from 'k6';
import { Counter } from 'k6/metrics';
import { textSummary } from 'https://jslib.k6.io/k6-summary/0.0.2/index.js';
import {
  DUOTUNNEL_CASES,
  DUOTUNNEL_PHASES,
  buildCounters,
  buildScenarios,
  buildThresholds,
  buildSummaryOutput,
  enrichCaseDefs,
} from './catalog.js';

function activeProfile() {
  const p = (__ENV.BENCH_PROFILE || 'full').toLowerCase();
  if (p === '8k' || p === 'core' || p === 'full') return p;
  return 'full';
}

function filterCases(cases, profile) {
  if (profile === '8k') return cases.filter((c) => c.name.includes('_8000qps'));
  if (profile === 'core') return cases.filter((c) => !c.name.includes('_8000qps'));
  return cases;
}

function filterByCase(cases) {
  const name = (__ENV.BENCH_CASE || '').trim();
  if (!name) return cases;
  return cases.filter((c) => c.name === name);
}

function parseEnvPositiveInt(v) {
  if (v === undefined || v === null || v === '') return null;
  const n = Number.parseInt(String(v), 10);
  if (!Number.isFinite(n) || n < 1) return null;
  return n;
}

function apply8kScenarioOverrides(cases) {
  const rate = parseEnvPositiveInt(__ENV.K6_8K_RATE);
  const preAllocatedVUs = parseEnvPositiveInt(__ENV.K6_8K_PREALLOCATED_VUS);
  const maxVUs = parseEnvPositiveInt(__ENV.K6_8K_MAX_VUS);
  if (rate === null && preAllocatedVUs === null && maxVUs === null) return cases;

  return cases.map((c) => {
    if (!c.name.includes('_8000qps')) return c;
    const scenario = { ...(c.scenario || {}) };
    if (rate !== null) scenario.rate = rate;
    if (preAllocatedVUs !== null) scenario.preAllocatedVUs = preAllocatedVUs;
    if (maxVUs !== null) scenario.maxVUs = maxVUs;
    return { ...c, scenario };
  });
}

// Apply K6_CORE_STRESS_RATE override to stress-category cases in the core profile.
// Scales both constant-arrival-rate `rate` and the final target of ramping stages.
// Does not affect 8k cases (those are handled by apply8kScenarioOverrides).
function applyCoreStressRateOverride(cases) {
  const rate = parseEnvPositiveInt(__ENV.K6_CORE_STRESS_RATE);
  if (rate === null) return cases;

  return cases.map((c) => {
    if (c.category !== 'stress' || c.name.includes('_8000qps')) return c;
    const scenario = { ...(c.scenario || {}) };
    if (scenario.executor === 'constant-arrival-rate') {
      scenario.rate = rate;
    } else if (Array.isArray(scenario.stages) && scenario.stages.length > 0) {
      // Scale the final stage target; keep intermediate ramp proportional
      const originalPeak = scenario.stages[scenario.stages.length - 1].target;
      scenario.stages = scenario.stages.map((st, i) => {
        if (i === scenario.stages.length - 1) return { ...st, target: rate };
        // Scale intermediate targets proportionally, minimum 1
        const scaled = Math.max(1, Math.round((st.target / originalPeak) * rate));
        return { ...st, target: scaled };
      });
    }
    return { ...c, scenario, targetRate: rate };
  });
}

function parseSeconds(v) {
  if (v === undefined || v === null) return 0;
  if (typeof v === 'number') return v;
  const s = String(v).trim();
  if (s.endsWith('s')) return Number(s.slice(0, -1)) || 0;
  return Number(s) || 0;
}

function formatSeconds(v) {
  const n = Math.max(0, Number(v) || 0);
  const s = Number.isInteger(n) ? String(n) : String(Number(n.toFixed(3)));
  return `${s}s`;
}

function normalizeCaseStartTimes(cases) {
  const cloned = cases.map((c) => ({ ...c, scenario: { ...(c.scenario || {}) } }));
  if (!cloned.length) return { cases: cloned, offset: 0 };
  const minStart = Math.min(...cloned.map((c) => parseSeconds(c.scenario.startTime || 0)));
  if (minStart <= 0) return { cases: cloned, offset: 0 };
  for (const c of cloned) {
    const shifted = parseSeconds(c.scenario.startTime || 0) - minStart;
    c.scenario.startTime = formatSeconds(shifted);
  }
  return { cases: cloned, offset: minStart };
}

function filterPhases(phases, activeCases) {
  const active = new Set(activeCases.map((c) => c.name));
  return phases.filter((p) => (p.scenarios || []).some((s) => active.has(s)));
}

const BENCH_PROFILE = activeProfile();
const FILTERED_CASES = apply8kScenarioOverrides(
  applyCoreStressRateOverride(filterByCase(filterCases(DUOTUNNEL_CASES, BENCH_PROFILE)))
);
const { cases: NORMALIZED_CASES, offset: ACTIVE_OFFSET } = normalizeCaseStartTimes(FILTERED_CASES);
const ACTIVE_PHASES = filterPhases(DUOTUNNEL_PHASES, FILTERED_CASES).map((p) => ({
  ...p,
  start: Math.max(0, (p.start || 0) - ACTIVE_OFFSET),
  end: Math.max(0, (p.end || 0) - ACTIVE_OFFSET),
}));
// Enrich cases with derived display metadata (timeRange, thresholdSpec, phase).
const ACTIVE_CASES = enrichCaseDefs(NORMALIZED_CASES, ACTIVE_PHASES);

const { reqCounters, errCounters } = buildCounters(ACTIVE_CASES, Counter);

const PAYLOAD_1K = 'x'.repeat(1024);
const PAYLOAD_10K = 'x'.repeat(10240);
const PAYLOAD_100K = 'x'.repeat(102400);

export const options = {
  discardResponseBodies: true,
  scenarios: buildScenarios(ACTIVE_CASES),

  noConnectionReuse: false,

  thresholds: buildThresholds(ACTIVE_CASES),
};

function track(ok) {
  const sn = exec.scenario.name;
  reqCounters[sn].add(1);
  if (!ok) errCounters[sn].add(1);
}

export function ingressHttpGet() {
  const sn = exec.scenario.name;
  const id = `${__VU}-${__ITER}`;
  const res = http.get(`http://echo.local:8080/?id=${id}`, {
    timeout: '10s',
    tags: { name: sn },
    responseType: 'text',
  });
  const ok = check(res, {
    'ingress GET 200': (r) => r.status === 200,
    'ingress GET id echo': (r) => r.body && r.body.includes(id),
  });
  track(ok);
}


const MULTIHOST_COUNT = 50;

export function ingressMultihost() {
  const sn = exec.scenario.name;
  const id = `${__VU}-${__ITER}`;
  const n = ((__VU - 1) % MULTIHOST_COUNT) + 1;
  const host = `echo-${String(n).padStart(2, '0')}.local`;
  const res = http.get(`http://${host}:8080/?id=${id}&host=${host}`, {
    timeout: '10s',
    tags: { name: sn },
    responseType: 'text',
  });
  const ok = check(res, {
    'ingress multihost 200': (r) => r.status === 200,
    'ingress multihost host echo': (r) => r.body && r.body.includes(host),
    'ingress multihost id echo': (r) => r.body && r.body.includes(id),
  });
  track(ok);
}

export function egressMultihost() {
  const sn = exec.scenario.name;
  const id = `${__VU}-${__ITER}`;
  const n = ((__VU - 1) % MULTIHOST_COUNT) + 1;
  const host = `echo-${String(n).padStart(2, '0')}.local`;
  const res = http.get(`http://${host}:8082/?id=${id}&host=${host}`, {
    timeout: '10s',
    tags: { name: sn },
    responseType: 'text',
  });
  const ok = check(res, {
    'egress multihost 200': (r) => r.status === 200,
    'egress multihost host echo': (r) => r.body && r.body.includes(host),
    'egress multihost id echo': (r) => r.body && r.body.includes(id),
  });
  track(ok);
}

export function ingressHttpPost() {
  const sn = exec.scenario.name;
  const id = `${__VU}-${__ITER}`;
  const res = http.post(
    'http://echo.local:8080/',
    JSON.stringify({ bench: 'ingress-post', id: id }),
    { headers: { 'Content-Type': 'application/json' }, timeout: '10s', tags: { name: sn }, responseType: 'text' },
  );
  const ok = check(res, {
    'ingress POST 200': (r) => r.status === 200,
    'ingress POST body': (r) => r.body && r.body.includes(id),
  });
  track(ok);
}

export function egressHttpGet() {
  const sn = exec.scenario.name;
  const id = `${__VU}-${__ITER}`;
  const res = http.get(`http://echo.local:8082/?id=${id}`, {
    timeout: '10s',
    tags: { name: sn },
    responseType: 'text',
  });
  const ok = check(res, {
    'egress GET 200': (r) => r.status === 200,
    'egress GET id echo': (r) => r.body && r.body.includes(id),
  });
  track(ok);
}

export function egressHttpPost() {
  const sn = exec.scenario.name;
  const id = `${__VU}-${__ITER}`;
  const res = http.post(
    'http://echo.local:8082/',
    JSON.stringify({ bench: 'egress-post', id: id }),
    {
      headers: { 'Content-Type': 'application/json' },
      timeout: '10s',
      tags: { name: sn },
      responseType: 'text',
    },
  );
  const ok = check(res, {
    'egress POST 200': (r) => r.status === 200,
    'egress POST body': (r) => r.body && r.body.includes(id),
  });
  track(ok);
}

export function ingressPost1K() {
  const sn = exec.scenario.name;
  const res = http.post(
    'http://echo.local:8080/',
    PAYLOAD_1K,
    {
      headers: { 'Content-Type': 'application/octet-stream' },
      timeout: '10s',
      tags: { name: sn },
      responseType: 'text',
    },
  );
  const ok = check(res, {
    'ingress 1K 200': (r) => r.status === 200,
    'ingress 1K size': (r) => r.body && r.body.length >= 1024,
  });
  track(ok);
}

export function ingressPost10K() {
  const sn = exec.scenario.name;
  const res = http.post(
    'http://echo.local:8080/',
    PAYLOAD_10K,
    {
      headers: { 'Content-Type': 'application/octet-stream' },
      timeout: '10s',
      tags: { name: sn },
      responseType: 'text',
    },
  );
  const ok = check(res, {
    'ingress 10K 200': (r) => r.status === 200,
    'ingress 10K size': (r) => r.body && r.body.length >= 10240,
  });
  track(ok);
}

export function ingressPost100K() {
  const sn = exec.scenario.name;
  const res = http.post(
    'http://echo.local:8080/',
    PAYLOAD_100K,
    {
      headers: { 'Content-Type': 'application/octet-stream' },
      timeout: '10s',
      tags: { name: sn },
      responseType: 'text',
    },
  );
  const ok = check(res, {
    'ingress 100K 200': (r) => r.status === 200,
    'ingress 100K size': (r) => r.body && r.body.length >= 102400,
  });
  track(ok);
}

export function egressPost10K() {
  const sn = exec.scenario.name;
  const res = http.post(
    'http://echo.local:8082/',
    PAYLOAD_10K,
    {
      headers: { 'Content-Type': 'application/octet-stream' },
      timeout: '10s',
      tags: { name: sn },
      responseType: 'text',
    },
  );
  const ok = check(res, {
    'egress 10K 200': (r) => r.status === 200,
    'egress 10K size': (r) => r.body && r.body.length >= 10240,
  });
  track(ok);
}

export function wsIngress() {
  const sn = exec.scenario.name;
  ws.connect('ws://ws.local:8080', { tags: { name: sn } }, function (socket) {
    let counted = false;

    socket.setTimeout(function () {
      if (!counted) { reqCounters[sn].add(1); errCounters[sn].add(1); counted = true; }
      socket.close();
    }, 5000);

    socket.on('open', function () {
      socket.send('k6-bench-ping');
    });

    socket.on('message', function (msg) {
      const ok = check(msg, {
        'ws echo matches': (d) => d === 'k6-bench-ping',
      });
      if (!counted) { reqCounters[sn].add(1); if (!ok) errCounters[sn].add(1); counted = true; }
      socket.close();
    });

    socket.on('error', function () {
      if (!counted) { reqCounters[sn].add(1); errCounters[sn].add(1); counted = true; }
    });
  });
}

export function wsMultiMsg() {
  const sn = exec.scenario.name;
  const msgCount = 20;
  let received = 0;
  let counted = false;

  ws.connect('ws://ws.local:8080', { tags: { name: sn } }, function (socket) {
    socket.setTimeout(function () {
      if (!counted) { reqCounters[sn].add(1); errCounters[sn].add(1); counted = true; }
      socket.close();
    }, 10000);

    socket.on('open', function () {
      for (let i = 0; i < msgCount; i++) {
        socket.send(`burst-${__VU}-${__ITER}-${i}`);
      }
    });

    socket.on('message', function (msg) {
      received++;
      check(msg, {
        'ws burst echo': (d) => d.startsWith(`burst-${__VU}-${__ITER}-`),
      });
      if (received >= msgCount) {
        if (!counted) { reqCounters[sn].add(1); counted = true; }
        socket.close();
      }
    });

    socket.on('error', function () {
      if (!counted) { reqCounters[sn].add(1); errCounters[sn].add(1); counted = true; }
    });
  });
}

const grpcHealthClient = new grpc.Client();
grpcHealthClient.load(['../proto'], 'health.proto');

const grpcEchoClient = new grpc.Client();
grpcEchoClient.load(['../proto'], 'grpc_echo.proto');

const grpcLargeClient = new grpc.Client();
grpcLargeClient.load(['../proto'], 'grpc_echo.proto');

const grpcHighQpsClient = new grpc.Client();
grpcHighQpsClient.load(['../proto'], 'grpc_echo.proto');


export function grpcHealthIngress() {
  const sn = exec.scenario.name;
  if (__ITER === 0) grpcHealthClient.connect('grpc.local:8080', { plaintext: true, timeout: '5s' });
  const resp = grpcHealthClient.invoke('grpc.health.v1.Health/Check', { service: '' }, { tags: { name: sn } });
  const ok = resp && resp.status === grpc.StatusOK;
  reqCounters[sn].add(1);
  if (!ok) {
    errCounters[sn].add(1);
    try { grpcHealthClient.close(); } catch (_) {}
    grpcHealthClient.connect('grpc.local:8080', { plaintext: true, timeout: '5s' });
  }
  check(resp, { 'grpc health OK': (r) => r && r.status === grpc.StatusOK });
}

export function grpcEchoIngress() {
  const sn = exec.scenario.name;
  const id = `${__VU}-${__ITER}`;
  if (__ITER === 0) grpcEchoClient.connect('grpc.local:8080', { plaintext: true, timeout: '5s' });
  const resp = grpcEchoClient.invoke('grpc_echo.v1.EchoService/Echo', { ping: id }, { tags: { name: sn } });
  const ok = resp && resp.status === grpc.StatusOK;
  reqCounters[sn].add(1);
  if (!ok) {
    errCounters[sn].add(1);
    try { grpcEchoClient.close(); } catch (_) {}
    grpcEchoClient.connect('grpc.local:8080', { plaintext: true, timeout: '5s' });
  }
  check(resp, {
    'grpc echo OK': (r) => r && r.status === grpc.StatusOK,
    'grpc echo body': (r) => r && r.message && r.message.body === id,
  });
}

export function grpcLargePayload() {
  const sn = exec.scenario.name;
  if (__ITER === 0) grpcLargeClient.connect('grpc.local:8080', { plaintext: true, timeout: '5s' });
  const resp = grpcLargeClient.invoke('grpc_echo.v1.EchoService/Echo', { ping: PAYLOAD_10K }, { tags: { name: sn } });
  const ok = resp && resp.status === grpc.StatusOK;
  reqCounters[sn].add(1);
  if (!ok) {
    errCounters[sn].add(1);
    try { grpcLargeClient.close(); } catch (_) {}
    grpcLargeClient.connect('grpc.local:8080', { plaintext: true, timeout: '5s' });
  }
  check(resp, {
    'grpc 10K OK': (r) => r && r.status === grpc.StatusOK,
    'grpc 10K size': (r) => r && r.message && r.message.body.length >= 10240,
  });
}

export function grpcHighQps() {
  const sn = exec.scenario.name;
  const id = `${__VU}-${__ITER}`;
  if (__ITER === 0) grpcHighQpsClient.connect('grpc.local:8080', { plaintext: true, timeout: '5s' });
  const resp = grpcHighQpsClient.invoke('grpc_echo.v1.EchoService/Echo', { ping: id }, { tags: { name: sn } });
  const ok = resp && resp.status === grpc.StatusOK;
  reqCounters[sn].add(1);
  if (!ok) {
    errCounters[sn].add(1);
    try { grpcHighQpsClient.close(); } catch (_) {}
    grpcHighQpsClient.connect('grpc.local:8080', { plaintext: true, timeout: '5s' });
  }
  check(resp, {
    'grpc high qps OK': (r) => r && r.status === grpc.StatusOK,
    'grpc high qps body': (r) => r && r.message && r.message.body === id,
  });
}

export function bidirectional() {
  const sn = exec.scenario.name;
  const id = `${__VU}-${__ITER}`;
  const inRes = http.get(`http://echo.local:8080/?id=${id}`, {
    timeout: '10s',
    tags: { name: sn },
    responseType: 'text',
  });
  const egRes = http.get(`http://echo.local:8082/?id=${id}`, {
    timeout: '10s',
    tags: { name: sn },
    responseType: 'text',
  });
  const ok1 = check(inRes, {
    'bidir ingress 200': (r) => r.status === 200,
    'bidir ingress echo': (r) => r.body && r.body.includes(id),
  });
  const ok2 = check(egRes, {
    'bidir egress 200': (r) => r.status === 200,
    'bidir egress echo': (r) => r.body && r.body.includes(id),
  });
  reqCounters[sn].add(1);
  if (!ok1 || !ok2) errCounters[sn].add(1);
}

export function handleSummary(data) {
  const output = buildSummaryOutput(data, ACTIVE_CASES, ACTIVE_PHASES);

  return {
    stdout: textSummary(data, { indent: ' ', enableColors: false }),
    '/tmp/bench-results.json': JSON.stringify(output, null, 2),
  };
}
