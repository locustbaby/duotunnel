import http from 'k6/http';
import grpc from 'k6/net/grpc';
import ws from 'k6/ws';
import exec from 'k6/execution';
import { check } from 'k6';
import { Counter } from 'k6/metrics';
import { textSummary } from 'https://jslib.k6.io/k6-summary/0.0.2/index.js';

const reqCounters = {};
const errCounters = {};

const allScenarios = [
  'ingress_http_get', 'ingress_http_post',
  'egress_http_get', 'egress_http_post',
  'bidir_mixed',
  'ingress_post_1k', 'ingress_post_10k', 'ingress_post_100k', 'egress_post_10k',
  'grpc_health_ingress', 'grpc_echo_ingress', 'grpc_large_payload', 'grpc_high_qps',
  'ws_ingress', 'ws_multi_msg',
  'ingress_3000qps', 'egress_3000qps',
  'ingress_3000qps_nokl', 'egress_3000qps_nokl',
  'ingress_8000qps', 'egress_8000qps',
];
for (const s of allScenarios) {
  reqCounters[s] = new Counter(`c_reqs_${s}`);
  errCounters[s] = new Counter(`c_err_${s}`);
}

const durationThresholds = {};
for (const s of allScenarios) {
  let metric = 'http_req_duration';
  if (s.startsWith('grpc_')) metric = 'grpc_req_duration';
  if (s.startsWith('ws_')) metric = 'ws_session_duration';
  durationThresholds[`${metric}{name:${s}}`] = ['p(95)<60000'];
}

const PAYLOAD_1K = 'x'.repeat(1024);
const PAYLOAD_10K = 'x'.repeat(10240);
const PAYLOAD_100K = 'x'.repeat(102400);

export const options = {
  discardResponseBodies: true,
  scenarios: {
    ingress_http_get: {
      executor: 'ramping-arrival-rate',
      exec: 'ingressHttpGet',
      startRate: 10,
      timeUnit: '1s',
      stages: [
        { target: 40, duration: '5s' },
        { target: 40, duration: '20s' },
      ],
      preAllocatedVUs: 5,
      maxVUs: 30,
    },
    ingress_http_post: {
      executor: 'ramping-arrival-rate',
      exec: 'ingressHttpPost',
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
    egress_http_get: {
      executor: 'ramping-arrival-rate',
      exec: 'egressHttpGet',
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
    egress_http_post: {
      executor: 'ramping-arrival-rate',
      exec: 'egressHttpPost',
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
    ws_ingress: {
      executor: 'constant-arrival-rate',
      exec: 'wsIngress',
      rate: 10,
      timeUnit: '1s',
      duration: '20s',
      startTime: '3s',
      preAllocatedVUs: 5,
      maxVUs: 20,
    },
    grpc_health_ingress: {
      executor: 'constant-arrival-rate',
      exec: 'grpcHealthIngress',
      rate: 20,
      timeUnit: '1s',
      duration: '20s',
      startTime: '5s',
      preAllocatedVUs: 3,
      maxVUs: 15,
    },
    grpc_echo_ingress: {
      executor: 'constant-arrival-rate',
      exec: 'grpcEchoIngress',
      rate: 20,
      timeUnit: '1s',
      duration: '20s',
      startTime: '7s',
      preAllocatedVUs: 3,
      maxVUs: 15,
    },
    bidir_mixed: {
      executor: 'ramping-arrival-rate',
      exec: 'bidirectional',
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

    ingress_post_1k: {
      executor: 'constant-arrival-rate',
      exec: 'ingressPost1K',
      rate: 30,
      timeUnit: '1s',
      duration: '20s',
      startTime: '35s',
      preAllocatedVUs: 5,
      maxVUs: 20,
    },
    ingress_post_10k: {
      executor: 'constant-arrival-rate',
      exec: 'ingressPost10K',
      rate: 20,
      timeUnit: '1s',
      duration: '20s',
      startTime: '35s',
      preAllocatedVUs: 5,
      maxVUs: 20,
    },
    ingress_post_100k: {
      executor: 'constant-arrival-rate',
      exec: 'ingressPost100K',
      rate: 10,
      timeUnit: '1s',
      duration: '20s',
      startTime: '35s',
      preAllocatedVUs: 5,
      maxVUs: 20,
    },
    egress_post_10k: {
      executor: 'constant-arrival-rate',
      exec: 'egressPost10K',
      rate: 20,
      timeUnit: '1s',
      duration: '20s',
      startTime: '37s',
      preAllocatedVUs: 5,
      maxVUs: 20,
    },
    ws_multi_msg: {
      executor: 'constant-arrival-rate',
      exec: 'wsMultiMsg',
      rate: 5,
      timeUnit: '1s',
      duration: '20s',
      startTime: '36s',
      preAllocatedVUs: 5,
      maxVUs: 20,
    },
    grpc_large_payload: {
      executor: 'constant-arrival-rate',
      exec: 'grpcLargePayload',
      rate: 15,
      timeUnit: '1s',
      duration: '20s',
      startTime: '38s',
      preAllocatedVUs: 3,
      maxVUs: 15,
    },
    grpc_high_qps: {
      executor: 'ramping-arrival-rate',
      exec: 'grpcHighQps',
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

    ingress_3000qps: {
      executor: 'constant-arrival-rate',
      exec: 'ingressHttpGet',
      rate: 3000,
      timeUnit: '1s',
      duration: '20s',
      startTime: '65s',
      preAllocatedVUs: 50,
      maxVUs: 500,
    },
    ingress_3000qps_nokl: {
      executor: 'constant-arrival-rate',
      exec: 'ingressHttpGetNoKeepalive',
      rate: 3000,
      timeUnit: '1s',
      duration: '20s',
      startTime: '90s',
      preAllocatedVUs: 50,
      maxVUs: 500,
    },
    egress_3000qps: {
      executor: 'constant-arrival-rate',
      exec: 'egressHttpGet',
      rate: 3000,
      timeUnit: '1s',
      duration: '20s',
      startTime: '115s',
      preAllocatedVUs: 50,
      maxVUs: 500,
    },
    egress_3000qps_nokl: {
      executor: 'constant-arrival-rate',
      exec: 'egressHttpGetNoKeepalive',
      rate: 3000,
      timeUnit: '1s',
      duration: '20s',
      startTime: '140s',
      preAllocatedVUs: 50,
      maxVUs: 500,
    },
    ingress_8000qps: {
      executor: 'constant-arrival-rate',
      exec: 'ingressHttpGet',
      rate: 8000,
      timeUnit: '1s',
      duration: '20s',
      startTime: '165s',
      preAllocatedVUs: 100,
      maxVUs: 500,
    },
    egress_8000qps: {
      executor: 'constant-arrival-rate',
      exec: 'egressHttpGet',
      rate: 8000,
      timeUnit: '1s',
      duration: '20s',
      startTime: '190s',
      preAllocatedVUs: 100,
      maxVUs: 500,
    },
  },

  noConnectionReuse: false,

  thresholds: {
    ...durationThresholds,
    'http_req_failed{scenario:ingress_http_get}': ['rate<0.05'],
    'http_req_failed{scenario:ingress_http_post}': ['rate<0.05'],
    'http_req_failed{scenario:egress_http_get}': ['rate<0.05'],
    'http_req_failed{scenario:egress_http_post}': ['rate<0.05'],
    'http_req_failed{scenario:bidir_mixed}': ['rate<0.05'],
    'http_req_failed{scenario:ingress_post_1k}': ['rate<0.05'],
    'http_req_failed{scenario:ingress_post_10k}': ['rate<0.05'],
    'http_req_failed{scenario:ingress_post_100k}': ['rate<0.05'],
    'http_req_failed{scenario:egress_post_10k}': ['rate<0.05'],
    c_err_ws_ingress: ['count<50'],
    c_err_ws_multi_msg: ['count<50'],
  },
};

function track(ok) {
  const sn = exec.scenario.name;
  reqCounters[sn].add(1);
  if (!ok) errCounters[sn].add(1);
}

export function ingressHttpGet() {
  const sn = exec.scenario.name;
  const withBody = !(sn.includes('_3000qps') || sn.includes('_8000qps'));
  const id = `${__VU}-${__ITER}`;
  const res = http.get(`http://echo.local:8080/?id=${id}`, {
    timeout: '10s',
    tags: { name: sn },
    responseType: withBody ? 'text' : 'none',
  });
  const checks = { 'ingress GET 200': (r) => r.status === 200 };
  if (withBody) checks['ingress GET echo'] = (r) => r.body && r.body.includes(id);
  const ok = check(res, checks);
  track(ok);
}

export function ingressHttpGetNoKeepalive() {
  const sn = exec.scenario.name;
  const id = `${__VU}-${__ITER}`;
  const res = http.get(`http://echo.local:8080/?id=${id}`, {
    timeout: '10s',
    headers: { Connection: 'close' },
    tags: { name: sn },
  });
  const ok = check(res, {
    'ingress GET nokl 200': (r) => r.status === 200,
  });
  track(ok);
}

export function egressHttpGetNoKeepalive() {
  const sn = exec.scenario.name;
  const id = `${__VU}-${__ITER}`;
  const res = http.get(`http://127.0.0.1:8082/?id=${id}`, {
    headers: { Host: 'echo.local', Connection: 'close' },
    timeout: '10s',
    tags: { name: sn },
  });
  const ok = check(res, {
    'egress GET nokl 200': (r) => r.status === 200,
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
  const withBody = !(sn.includes('_3000qps') || sn.includes('_8000qps'));
  const id = `${__VU}-${__ITER}`;
  const res = http.get(`http://127.0.0.1:8082/?id=${id}`, {
    headers: { Host: 'echo.local' },
    timeout: '10s',
    tags: { name: sn },
    responseType: withBody ? 'text' : 'none',
  });
  const checks = { 'egress GET 200': (r) => r.status === 200 };
  if (withBody) checks['egress GET echo'] = (r) => r.body && r.body.includes(id);
  const ok = check(res, checks);
  track(ok);
}

export function egressHttpPost() {
  const sn = exec.scenario.name;
  const id = `${__VU}-${__ITER}`;
  const res = http.post(
    'http://127.0.0.1:8082/',
    JSON.stringify({ bench: 'egress-post', id: id }),
    {
      headers: { Host: 'echo.local', 'Content-Type': 'application/json' },
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
    'http://127.0.0.1:8082/',
    PAYLOAD_10K,
    {
      headers: { Host: 'echo.local', 'Content-Type': 'application/octet-stream' },
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
  check(resp, { 'grpc high qps OK': (r) => r && r.status === grpc.StatusOK });
}

export function bidirectional() {
  const sn = exec.scenario.name;
  const id = `${__VU}-${__ITER}`;
  const inRes = http.get(`http://echo.local:8080/?id=${id}`, {
    timeout: '10s',
    tags: { name: sn },
    responseType: 'text',
  });
  const egRes = http.get(`http://127.0.0.1:8082/?id=${id}`, {
    headers: { Host: 'echo.local' },
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
  const m = data.metrics;

  function r2(v) {
    return Math.round(v * 100) / 100;
  }

  function metricByName(base, name) {
    const direct = m[`${base}{name:${name}}`];
    if (direct) return direct;
    const needle = `name:${name}`;
    for (const [k, v] of Object.entries(m)) {
      if (k.startsWith(`${base}{`) && k.includes(needle)) return v;
    }
    return null;
  }

  const scenarioMeta = {
    ingress_http_get:    { protocol: 'HTTP', direction: 'ingress', category: 'basic',     duration: 25, metric: 'http_req_duration' },
    ingress_http_post:   { protocol: 'HTTP', direction: 'ingress', category: 'basic',     duration: 25, metric: 'http_req_duration' },
    egress_http_get:     { protocol: 'HTTP', direction: 'egress',  category: 'basic',     duration: 25, metric: 'http_req_duration' },
    egress_http_post:    { protocol: 'HTTP', direction: 'egress',  category: 'basic',     duration: 25, metric: 'http_req_duration' },
    bidir_mixed:         { protocol: 'HTTP', direction: 'bidir',   category: 'basic',     duration: 20, metric: 'http_req_duration' },
    ingress_post_1k:     { protocol: 'HTTP', direction: 'ingress', category: 'body_size', duration: 20, metric: 'http_req_duration' },
    ingress_post_10k:    { protocol: 'HTTP', direction: 'ingress', category: 'body_size', duration: 20, metric: 'http_req_duration' },
    ingress_post_100k:   { protocol: 'HTTP', direction: 'ingress', category: 'body_size', duration: 20, metric: 'http_req_duration' },
    egress_post_10k:     { protocol: 'HTTP', direction: 'egress',  category: 'body_size', duration: 20, metric: 'http_req_duration' },
    grpc_health_ingress: { protocol: 'gRPC', direction: 'ingress', category: 'basic',     duration: 20, metric: 'grpc_req_duration' },
    grpc_echo_ingress:   { protocol: 'gRPC', direction: 'ingress', category: 'basic',     duration: 20, metric: 'grpc_req_duration' },
    grpc_large_payload:  { protocol: 'gRPC', direction: 'ingress', category: 'body_size', duration: 20, metric: 'grpc_req_duration' },
    grpc_high_qps:       { protocol: 'gRPC', direction: 'ingress', category: 'stress',    duration: 20, metric: 'grpc_req_duration' },
    ws_ingress:          { protocol: 'WS',   direction: 'ingress', category: 'basic',     duration: 20, metric: 'ws_session_duration' },
    ws_multi_msg:        { protocol: 'WS',   direction: 'ingress', category: 'basic',     duration: 20, metric: 'ws_session_duration' },
    ingress_3000qps:      { protocol: 'HTTP', direction: 'ingress', category: 'stress',   duration: 20, metric: 'http_req_duration' },
    egress_3000qps:       { protocol: 'HTTP', direction: 'egress',  category: 'stress',   duration: 20, metric: 'http_req_duration' },
    ingress_3000qps_nokl: { protocol: 'HTTP', direction: 'ingress', category: 'stress',   duration: 20, metric: 'http_req_duration' },
    egress_3000qps_nokl:  { protocol: 'HTTP', direction: 'egress',  category: 'stress',   duration: 20, metric: 'http_req_duration' },
    ingress_8000qps:      { protocol: 'HTTP', direction: 'ingress', category: 'stress',   duration: 20, metric: 'http_req_duration' },
    egress_8000qps:       { protocol: 'HTTP', direction: 'egress',  category: 'stress',   duration: 20, metric: 'http_req_duration' },
  };

  const phases = [
    {name:'Basic', start:0, end:29, scenarios:['ingress_http_get','ingress_http_post','egress_http_get','egress_http_post','ws_ingress','grpc_health_ingress','grpc_echo_ingress','bidir_mixed']},
    {name:'Body/Payload', start:35, end:59, scenarios:['ingress_post_1k','ingress_post_10k','ingress_post_100k','egress_post_10k','ws_multi_msg','grpc_large_payload','grpc_high_qps']},
    {name:'Ingress 3K', start:65, end:85, scenarios:['ingress_3000qps']},
    {name:'Ingress no-KL', start:90, end:110, scenarios:['ingress_3000qps_nokl']},
    {name:'Egress 3K', start:115, end:135, scenarios:['egress_3000qps']},
    {name:'Egress no-KL', start:140, end:160, scenarios:['egress_3000qps_nokl']},
    {name:'Ingress 8K', start:165, end:185, scenarios:['ingress_8000qps']},
    {name:'Egress 8K', start:190, end:210, scenarios:['egress_8000qps']},
  ];

  const scenarios = [];
  let totalRPS = 0;
  let totalRequests = 0;
  let totalErrors = 0;

  for (const [name, meta] of Object.entries(scenarioMeta)) {
    const trend = metricByName(meta.metric, name);
    if (!trend) continue;

    const v = trend.values;
    const p50 = r2(v.med);
    const p95 = r2(v['p(95)']);
    const p99 = r2(v['p(99)']);

    const reqMetric = m[`c_reqs_${name}`];
    const errMetric = m[`c_err_${name}`];

    const requests = reqMetric ? reqMetric.values.count : 0;
    const errors = errMetric ? errMetric.values.count : 0;
    const rps = meta.duration ? r2(requests / meta.duration) : r2(reqMetric ? reqMetric.values.rate : 0);
    const err = requests > 0 ? r2(errors / requests * 100) : 0;

    totalRequests += requests;
    totalErrors += errors;
    if (meta.category === 'stress') totalRPS += rps;

    scenarios.push({
      name,
      protocol: meta.protocol,
      direction: meta.direction,
      category: meta.category,
      p50, p95, p99, err, rps, requests,
    });
  }

  const totalErr = totalRequests > 0 ? r2(totalErrors / totalRequests * 100) : 0;

  const output = {
    timestamp: new Date().toISOString(),
    commit: __ENV.GITHUB_SHA || 'local',
    scenarios,
    summary: {
      totalRPS: r2(totalRPS),
      totalErr,
      totalRequests,
    },
    phases,
  };

  return {
    stdout: textSummary(data, { indent: ' ', enableColors: false }),
    '/tmp/bench-results.json': JSON.stringify(output, null, 2),
  };
}
