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

const { reqCounters, errCounters } = buildCounters(FRP_CASES, Counter);

export const options = {
  discardResponseBodies: true,
  scenarios: buildScenarios(FRP_CASES),

  noConnectionReuse: false,

  thresholds: buildThresholds(FRP_CASES),
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
  const output = buildSummaryOutput(data, FRP_CASES, FRP_PHASES, { tunnel: 'frp' });

  return {
    stdout: textSummary(data, { indent: ' ', enableColors: false }),
    '/tmp/bench-results-frp.json': JSON.stringify(output, null, 2),
  };
}
