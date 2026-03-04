import http from 'k6/http';
import exec from 'k6/execution';
import { check } from 'k6';
import { Counter, Trend } from 'k6/metrics';
import { textSummary } from 'https://jslib.k6.io/k6-summary/0.0.2/index.js';

const trends = {};
const reqCounters = {};
const errCounters = {};

const allScenarios = ['ingress_3000qps', 'ingress_3000qps_nokl'];
for (const s of allScenarios) {
  trends[s] = new Trend(`t_${s}`, true);
  reqCounters[s] = new Counter(`c_reqs_${s}`);
  errCounters[s] = new Counter(`c_err_${s}`);
}

export const options = {
  scenarios: {
    ingress_3000qps: {
      executor: 'constant-arrival-rate',
      exec: 'ingressHttpGetKeepalive',
      rate: 3000,
      timeUnit: '1s',
      duration: '20s',
      startTime: '0s',
      preAllocatedVUs: 50,
      maxVUs: 500,
    },
    ingress_3000qps_nokl: {
      executor: 'constant-arrival-rate',
      exec: 'ingressHttpGetNokl',
      rate: 3000,
      timeUnit: '1s',
      duration: '20s',
      startTime: '25s',
      preAllocatedVUs: 50,
      maxVUs: 500,
    },
  },

  noConnectionReuse: false,

  thresholds: {
    'http_req_failed{scenario:ingress_3000qps}':      ['rate<0.20'],
    'http_req_failed{scenario:ingress_3000qps_nokl}': ['rate<0.20'],
  },
};

function track(res, ok) {
  const sn = exec.scenario.name;
  trends[sn].add(res.timings.duration);
  reqCounters[sn].add(1);
  if (!ok) errCounters[sn].add(1);
}

export function ingressHttpGetKeepalive() {
  const id = `${__VU}-${__ITER}`;
  const res = http.get(`http://echo.local:18090/?id=${id}`, {
    timeout: '10s',
    tags: { name: 'frp_ingress_kl' },
  });
  const ok = check(res, {
    'frp ingress kl 200': (r) => r.status === 200,
    'frp ingress kl echo': (r) => r.body && r.body.includes(id),
  });
  track(res, ok);
}

export function ingressHttpGetNokl() {
  const id = `${__VU}-${__ITER}`;
  const res = http.get(`http://echo.local:18090/?id=${id}`, {
    timeout: '10s',
    headers: { Connection: 'close' },
    tags: { name: 'frp_ingress_nokl' },
  });
  const ok = check(res, {
    'frp ingress nokl 200': (r) => r.status === 200,
    'frp ingress nokl echo': (r) => r.body && r.body.includes(id),
  });
  track(res, ok);
}

export function handleSummary(data) {
  const m = data.metrics;

  function r2(v) {
    return Math.round(v * 100) / 100;
  }

  const scenarioMeta = {
    ingress_3000qps:      { protocol: 'HTTP', direction: 'ingress', category: 'stress', duration: 20 },
    ingress_3000qps_nokl: { protocol: 'HTTP', direction: 'ingress', category: 'stress', duration: 20 },
  };

  const phases = [
    {name:'3K KL (frp)',    start:0,  end:20, scenarios:['ingress_3000qps']},
    {name:'3K no-KL (frp)', start:25, end:45, scenarios:['ingress_3000qps_nokl']},
  ];

  const scenarios = [];
  let totalRequests = 0;
  let totalErrors = 0;

  for (const [name, meta] of Object.entries(scenarioMeta)) {
    const trend = m[`t_${name}`];
    if (!trend) continue;

    const v = trend.values;
    const p50 = r2(v.med);
    const p95 = r2(v['p(95)']);
    const p99 = r2(v['p(99)']);

    const reqMetric = m[`c_reqs_${name}`];
    const errMetric = m[`c_err_${name}`];

    const requests = reqMetric ? reqMetric.values.count : 0;
    const errors = errMetric ? errMetric.values.count : 0;
    const rps = meta.duration ? r2(requests / meta.duration) : 0;
    const err = requests > 0 ? r2(errors / requests * 100) : 0;

    totalRequests += requests;
    totalErrors += errors;

    scenarios.push({
      name,
      protocol: meta.protocol,
      direction: meta.direction,
      category: meta.category,
      p50, p95, p99, err, rps, requests,
    });
  }

  const totalErr = totalRequests > 0 ? r2(totalErrors / totalRequests * 100) : 0;
  const totalRPS = r2(scenarios.reduce((s, sc) => s + (sc.rps || 0), 0));

  const output = {
    timestamp: new Date().toISOString(),
    commit: __ENV.GITHUB_SHA || 'local',
    tunnel: 'frp',
    scenarios,
    summary: { totalRPS, totalErr, totalRequests },
    phases,
  };

  return {
    stdout: textSummary(data, { indent: ' ', enableColors: false }),
    '/tmp/bench-results-frp.json': JSON.stringify(output, null, 2),
  };
}
