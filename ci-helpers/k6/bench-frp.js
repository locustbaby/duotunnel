import http from 'k6/http';
import exec from 'k6/execution';
import { check } from 'k6';
import { Counter } from 'k6/metrics';
import { textSummary } from 'https://jslib.k6.io/k6-summary/0.0.2/index.js';
import {
  FRP_CASES,
  FRP_PHASES,
  buildCounters,
  buildScenarios,
  buildThresholds,
  buildSummaryOutput,
} from './catalog.js';

function parseEnvPositiveInt(v) {
  if (v === undefined || v === null || v === '') return null;
  const n = Number.parseInt(String(v), 10);
  if (!Number.isFinite(n) || n < 1) return null;
  return n;
}

function parseCaseParamMap(raw) {
  const out = new Map();
  const text = String(raw || '').trim();
  if (!text) return out;
  for (const seg of text.split(';')) {
    const item = seg.trim();
    if (!item) continue;
    const eq = item.indexOf('=');
    if (eq <= 0) continue;
    const name = item.slice(0, eq).trim();
    const body = item.slice(eq + 1).trim();
    if (!name || !body) continue;
    const parts = body.split(',').map((v) => v.trim());
    const rate = parseEnvPositiveInt(parts[0] || '');
    const preAllocatedVUs = parseEnvPositiveInt(parts[1] || '');
    const maxVUs = parseEnvPositiveInt(parts[2] || '');
    if (rate === null && preAllocatedVUs === null && maxVUs === null) continue;
    out.set(name, { rate, preAllocatedVUs, maxVUs });
  }
  return out;
}

function applyCaseScenarioOverrides(cases) {
  const overrides = parseCaseParamMap(__ENV.K6_CASE_PARAMS);
  if (overrides.size === 0) return cases;
  return cases.map((c) => {
    const o = overrides.get(c.name);
    if (!o) return c;
    const scenario = { ...(c.scenario || {}) };
    if (o.rate !== null) scenario.rate = o.rate;
    if (o.preAllocatedVUs !== null) scenario.preAllocatedVUs = o.preAllocatedVUs;
    if (o.maxVUs !== null) scenario.maxVUs = o.maxVUs;
    const next = { ...c, scenario };
    if (o.rate !== null) next.targetRate = o.rate;
    return next;
  });
}

const ACTIVE_FRP_CASES = applyCaseScenarioOverrides(FRP_CASES);
const { reqCounters, errCounters } = buildCounters(ACTIVE_FRP_CASES, Counter);

export const options = {
  discardResponseBodies: true,
  scenarios: buildScenarios(ACTIVE_FRP_CASES),

  noConnectionReuse: false,

  thresholds: buildThresholds(ACTIVE_FRP_CASES),
};

function track(ok) {
  const sn = exec.scenario.name;
  reqCounters[sn].add(1);
  if (!ok) errCounters[sn].add(1);
}

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


export function ingressMultihost() {
  const sn = exec.scenario.name;
  const id = `${__VU}-${__ITER}`;
  const n = ((__VU - 1) % 50) + 1;
  const host = `echo-${String(n).padStart(2, '0')}.local`;
  const res = http.get(`http://${host}:18090/?id=${id}&host=${host}`, {
    timeout: '10s',
    tags: { name: sn },
    responseType: 'text',
  });
  const ok = check(res, {
    'frp ingress multihost 200': (r) => r.status === 200,
    'frp ingress multihost host echo': (r) => r.body && r.body.includes(host),
    'frp ingress multihost id echo': (r) => r.body && r.body.includes(id),
  });
  track(ok);
}

export function handleSummary(data) {
  const output = buildSummaryOutput(data, ACTIVE_FRP_CASES, FRP_PHASES, { tunnel: 'frp' });

  return {
    stdout: textSummary(data, { indent: ' ', enableColors: false }),
    '/tmp/bench-results-frp.json': JSON.stringify(output, null, 2),
  };
}
