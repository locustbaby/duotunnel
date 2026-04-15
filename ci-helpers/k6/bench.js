import http from 'k6/http';
import grpc from 'k6/net/grpc';
import ws from 'k6/ws';
import exec from 'k6/execution';
import { check } from 'k6';
import { Counter } from 'k6/metrics';
import { textSummary } from 'https://jslib.k6.io/k6-summary/0.0.2/index.js';
import {
  ALL_CASES,
  buildCounters,
  buildScenarios,
  buildThresholds,
  buildSummaryOutput,
  enrichCaseDefs,
} from './catalog.js';

function activeProfile() {
  const p = (__ENV.BENCH_PROFILE || 'full').toLowerCase();
  if (['8k', 'core', 'full', 'frp', 'basic', 'body_size'].includes(p)) return p;
  return 'full';
}

function filterCases(cases, profile) {
  if (profile === '8k')        return cases.filter((c) => c.name.includes('_8000qps') && !c.name.startsWith('frp_'));
  if (profile === 'core')      return cases.filter((c) => !c.name.includes('_8000qps') && !c.name.startsWith('frp_'));
  if (profile === 'frp')       return cases.filter((c) => c.name.startsWith('frp_'));
  if (profile === 'basic')     return cases.filter((c) => c.category === 'basic');
  if (profile === 'body_size') return cases.filter((c) => c.category === 'body_size');
  return cases;
}

function filterByCase(cases) {
  const name = (__ENV.BENCH_CASE || '').trim();
  if (!name) return cases;
  return cases
    .filter((c) => c.name === name)
    .map((c) => ({ ...c, scenario: { ...(c.scenario || {}), startTime: '0s' } }));
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

const BENCH_PROFILE = activeProfile();
const ACTIVE_CASES = enrichCaseDefs(
  apply8kScenarioOverrides(
    applyCoreStressRateOverride(filterByCase(filterCases(ALL_CASES, BENCH_PROFILE)))
  )
);

const RESULTS_PATH = __ENV.BENCH_RESULT_PATH || '/tmp/bench-results.json';

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
  // DuoTunnel uses 8080, FRP uses 18090. Differentiate by scenario name.
  const port = sn.startsWith('frp_') ? 18090 : 8080;
  const res = http.get(`http://${host}:${port}/?id=${id}&host=${host}`, {
    timeout: '10s',
    tags: { name: sn },
    responseType: 'text',
  });

  const checkPrefix = sn.startsWith('frp_') ? 'frp ingress multihost' : 'ingress multihost';
  const ok = check(res, {
    [`${checkPrefix} 200`]: (r) => r.status === 200,
    [`${checkPrefix} host echo`]: (r) => r.body && r.body.includes(host),
    [`${checkPrefix} id echo`]: (r) => r.body && r.body.includes(id),
  });
  track(ok);
}

// FRP-specific Keepalive function
export function ingressHttpGetKeepalive() {
  const sn = exec.scenario.name;
  const id = `${__VU}-${__ITER}`;
  const res = http.get(`http://echo.local:18090/?id=${id}`, {
    timeout: '10s',
    tags: { name: sn },
    responseType: 'text',
  });
  const ok = check(res, {
    'frp ingress kl 200': (r) => r.status === 200,
    'frp ingress kl id echo': (r) => r.body && r.body.includes(id),
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
  const extras = {};
  if (ACTIVE_CASES.length > 0 && ACTIVE_CASES.every(c => c.name.startsWith('frp_'))) {
    extras.tunnel = 'frp';
  }

  const output = buildSummaryOutput(data, ACTIVE_CASES, extras);

  return {
    stdout: textSummary(data, { indent: ' ', enableColors: false }),
    [RESULTS_PATH]: JSON.stringify(output, null, 2),
  };
}
