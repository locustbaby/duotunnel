const SCHEMA = JSON.parse(open('../schema.json'));
const DEFAULTS = JSON.parse(open('./cases/defaults.json'));
const BASIC = JSON.parse(open('./cases/basic.json'));
const BODY_SIZE = JSON.parse(open('./cases/body_size.json'));
const STRESS = JSON.parse(open('./cases/stress.json'));
const FRP = JSON.parse(open('./cases/frp.json'));

export const CATEGORY_DEFS = SCHEMA.categories;

function deepMerge(target, source) {
  const output = Object.assign({}, target);
  if (source && typeof source === 'object') {
    Object.keys(source).forEach(key => {
      if (source[key] && typeof source[key] === 'object' && !Array.isArray(source[key])) {
        output[key] = deepMerge(target[key] || {}, source[key]);
      } else {
        output[key] = source[key];
      }
    });
  }
  return output;
}

function processCases(input) {
  return (input.cases || []).map(c => deepMerge(DEFAULTS, c));
}

export const ALL_CASES = [
  ...processCases(BASIC),
  ...processCases(BODY_SIZE),
  ...processCases(STRESS),
  ...processCases(FRP),
];

// --- Helpers ---

function parseSeconds(v) {
  if (v === undefined || v === null) return 0;
  if (typeof v === 'number') return v;
  const s = String(v).trim();
  if (s.endsWith('s')) return Number(s.slice(0, -1)) || 0;
  return Number(s) || 0;
}

function buildThresholdSpec(c) {
  const parts = [];
  const dur = (c.thresholds && c.thresholds.duration) || 'p(95)<60000';
  parts.push(dur.replace('p(95)<', 'p95<').replace('p(99)<', 'p99<'));
  if (c.thresholds && c.thresholds.httpReqFailed) parts.push(`err ${c.thresholds.httpReqFailed}`);
  if (c.thresholds && c.thresholds.errCounter) parts.push(`errs ${c.thresholds.errCounter}`);
  return parts.join(', ');
}

export function enrichCaseDefs(caseDefs) {
  return caseDefs.map((c) => {
    const startSec = parseSeconds(c.scenario && c.scenario.startTime);
    const endSec = startSec + (c.durationSec || parseSeconds(c.scenario && c.scenario.duration));
    return {
      ...c,
      timeRange: { startSec, endSec },
      thresholdSpec: buildThresholdSpec(c),
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

function metricForProtocol(protocol) {
  if (protocol === 'gRPC') return 'grpc_req_duration';
  if (protocol === 'WS') return 'ws_session_duration';
  return 'http_req_duration';
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
  const needle = `name:${name}`;
  for (const [k, v] of Object.entries(metrics)) {
    if (k.startsWith(`${base}{`) && k.includes(needle)) return v;
  }
  return null;
}

function resolveCategories(caseDefs) {
  const seen = new Set();
  const cats = [];
  for (const c of caseDefs) {
    if (c.category && !seen.has(c.category)) {
      seen.add(c.category);
      const def = CATEGORY_DEFS.find(d => d.id === c.category);
      cats.push(def || { id: c.category, label: c.category });
    }
  }
  return cats;
}

export function buildSummaryOutput(data, caseDefs, extras = {}) {
  const metrics = data.metrics;
  const casesOut = {};
  let totalRPS = 0;
  let totalRequests = 0;
  let totalErrors = 0;

  const tunnel = extras.tunnel || 'duotunnel';

  for (const c of caseDefs) {
    const metric = c.metric || metricForProtocol(c.protocol);
    const trend = metricByName(metrics, metric, c.name);
    if (!trend || !trend.values) continue;

    const values = trend.values;
    const reqMetric = metrics[`c_reqs_${c.name}`];
    const errMetric = metrics[`c_err_${c.name}`];
    const requests = reqMetric?.values?.count || 0;
    const errors = errMetric?.values?.count || 0;
    const rps = c.durationSec ? Math.round((requests / c.durationSec) * 100) / 100 : 0;
    const err = requests > 0 ? Math.round((errors / requests) * 10000) / 100 : 0;

    totalRequests += requests;
    totalErrors += errors;
    if (c.includeInTotalRps) totalRPS += rps;

    casesOut[c.name] = {
      label: c.label,
      protocol: c.protocol,
      direction: c.direction,
      category: c.category,
      tunnel,
      perf: {
        rps, err, requests,
        p50: Math.round((values.med || 0) * 100) / 100,
        p95: Math.round((values['p(95)'] || 0) * 100) / 100,
        targetRate: c.targetRate != null ? c.targetRate : (c.scenario?.rate || null),
        thresholdSpec: c.thresholdSpec,
        includeInTotalRps: !!c.includeInTotalRps,
        payloadBytes: c.payloadBytes || null,
        timeRange: c.timeRange,
      },
      resources: null,
    };
  }

  const { tunnel: _t, ...restExtras } = extras;
  return {
    timestamp: new Date().toISOString(),
    cases: casesOut,
    summary: {
      totalRPS: Math.round(totalRPS * 100) / 100,
      totalErr: totalRequests > 0 ? Math.round((totalErrors / totalRequests) * 10000) / 100 : 0,
      totalRequests
    },
    catalog: { categories: resolveCategories(caseDefs) },
    ...restExtras
  };
}
