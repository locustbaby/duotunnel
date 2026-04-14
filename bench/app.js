/* ci-helpers/bench_ui/app.js */
(function(){

const root = document.getElementById('app');
const metaEl = document.getElementById('meta');
const entries = (window.BENCHMARK_DATA && window.BENCHMARK_DATA.entries) || [];

// --- Utilities ---
const Utils = {
  esc: (s) => { const d = document.createElement('div'); d.textContent = s; return d.innerHTML; },
  r2: (v) => Math.round(v * 100) / 100,
  fmt: (v) => v == null ? '—' : typeof v === 'number' ? v.toLocaleString(undefined, {maximumFractionDigits:1}) : v,
  sha7: (e) => { const c = e.commit; return typeof c === 'object' ? (c.id||'').substring(0,7) : (c||'').substring(0,7); },
  shaFull: (e) => { const c = e.commit; return typeof c === 'object' ? (c.id||'') : (c||''); },
  commitUrl: (e) => { const c = e.commit; return typeof c === 'object' ? (c.url||'') : ''; },
  commitMsg: (e) => { const c = e.commit; return typeof c === 'object' ? (c.message||'') : ''; },
  pill: (proto) => `<span class="pill pill-${proto.toLowerCase()}">${proto}</span>`,
  delta: (cur, pre, invert) => {
    if (pre == null || pre === 0) return '<span class="d d-flat">—</span>';
    const pct = Utils.r2((cur - pre) / pre * 100);
    const good = invert ? pct < 0 : pct > 0;
    const cls = Math.abs(pct) < 1 ? 'd-flat' : good ? 'd-up' : 'd-down';
    return `<span class="d ${cls}">${pct > 0 ? '+' : ''}${pct}%</span>`;
  },
  deltaVal: (cur, pre, invert) => {
    if (pre == null) return '—';
    const diff = Utils.r2(cur - pre);
    const good = invert ? diff < 0 : diff > 0;
    const cls = Math.abs(diff) < 0.01 ? 'c-dim' : good ? 'c-good' : 'c-bad';
    return `<span class="${cls}">${diff > 0 ? '+' : ''}${diff}</span>`;
  },
  latencyBar: (val, max, proto) => {
    const colors = {HTTP:'var(--blue)',gRPC:'var(--green)',WS:'var(--orange)'};
    const w = max > 0 ? Math.max(2, Math.round(val / max * 80)) : 0;
    return `<span class="bar" style="width:${w}px;background:${colors[proto]||'var(--text3)'}"></span>`;
  }
};

// --- UI Components ---
const UI = {
  Card: (label, value, deltaHtml, valueSuffix = '') => `
    <div class="card">
      <div class="lb">${label}</div>
      <div class="val">${value} ${valueSuffix ? `<span style="font-size:14px;font-weight:400;color:var(--text2)">${valueSuffix}</span>` : ''}</div>
      ${deltaHtml}
    </div>`,

  Panel: (title, content, options = {}) => `
    <div class="panel ${options.className || 'auto'}">
      ${title ? `<h3>${title}</h3>` : ''}
      ${content}
    </div>`,

  Table: (headers, rows, options = {}) => {
    let t = `<table ${options.id ? `id="${options.id}"` : ''}><thead><tr>`;
    headers.forEach(h => t += `<th>${h}</th>`);
    t += `</tr></thead><tbody>`;
    rows.forEach(r => {
      t += `<tr class="${r.className || ''}" ${r.attribs || ''}>`;
      r.cells.forEach(c => t += `<td ${c.attribs || ''}>${c.content !== undefined ? c.content : c}</td>`);
      t += `</tr>`;
    });
    t += `</tbody></table>`;
    return `<div class="tbl-wrap">${t}</div>`;
  },

  ChartPanel: (id, title, options = {}) => `
    <div class="row ${options.full ? 'full' : ''}">
      <div class="panel ${options.className || 'tall'}">
        <h3>${title}</h3>
        <div class="chart-area"><canvas id="${id}"></canvas></div>
        <div id="${id}-leg" class="leg-wrap"></div>
      </div>
    </div>`,

  Breadcrumb: (activeSha) => `
    <div class="breadcrumb">
      <a onclick="location.hash='overview';return false;" href="#overview">← All Runs</a> / ${activeSha}
    </div>`,

  Section: (title, content = '') => `
    <div class="sec">${title}</div>${content}`,

  LinkGroup: (links) => `
    <div style="font-size:13px;display:flex;gap:16px;flex-wrap:wrap">
      ${links.map(l => `<a href="${Utils.esc(l.url)}" target="_blank" style="${l.style || ''}">${l.label}</a>`).join('')}
    </div>`
};

// --- Chart Logic ---
const Charts = {
  live: [],
  destroyAll: () => { Charts.live.forEach(c => c.destroy()); Charts.live = []; },
  
  DEFAULT_OPTS: {
    responsive:true, maintainAspectRatio:false,
    plugins:{legend:{display:false}},
    scales:{
      x:{ticks:{color:'#6b7d93',font:{size:9}},grid:{color:'rgba(30,42,58,.5)'}},
      y:{ticks:{color:'#6b7d93'},grid:{color:'rgba(30,42,58,.5)'},beginAtZero:true},
    },
  },

  create: (id, cfg) => {
    const cv = document.getElementById(id);
    if (!cv) return;
    const c = new Chart(cv, cfg);
    Charts.live.push(c);
    Charts.buildLegend(c, id + '-leg');
    return c;
  },

  buildLegend: (chart, containerId) => {
    const container = document.getElementById(containerId);
    if (!container) return;
    container.innerHTML = '';
    const sync = () => {
      container.querySelectorAll('.leg-item').forEach((el, i) => { el.style.opacity = chart.data.datasets[i].hidden ? '0.3' : '1'; });
      chart.update();
    };
    chart.data.datasets.forEach((ds, i) => {
      const color = ds.borderColor || '#6b7d93';
      const isDash = ds.borderDash && ds.borderDash.length > 0;
      const item = document.createElement('span');
      item.className = 'leg-item';
      item.innerHTML = `<span class="leg-swatch" style="background:${isDash?'transparent':color};border:1.5px ${isDash?'dashed':'solid'} ${color};"></span><span class="leg-label">${ds.label||''}</span>`;
      item.addEventListener('click', () => {
        const isSolo = chart.data.datasets.every((d, j) => j === i ? !d.hidden : d.hidden);
        chart.data.datasets.forEach((d, j) => { d.hidden = isSolo ? false : j !== i; });
        sync();
      });
      container.appendChild(item);
    });
  }
};

// --- Domain Logic ---
const Data = {
  findEntry: (sha) => entries.find(e => Utils.shaFull(e) === sha || Utils.sha7(e) === sha),
  prevOf: (entry) => {
    const s = Utils.sha7(entry);
    const idx = entries.findIndex(e => Utils.sha7(e) === s);
    return idx > 0 ? entries[idx - 1] : null;
  },
  loadDetail: async (entry) => {
    const s7 = Utils.sha7(entry);
    try {
      const res = await fetch(`data/${s7}.json`);
      return res.ok ? await res.json() : entry;
    } catch(_) { return entry; }
  }
};

// --- Constants ---
const COLORS = {
  phases: ['rgba(77,166,255,.08)','rgba(52,211,153,.08)','rgba(245,166,35,.08)','rgba(239,83,80,.08)','rgba(167,139,250,.08)','rgba(244,114,182,.08)','rgba(56,189,248,.08)','rgba(251,191,36,.08)'],
  palette: ['#4da6ff','#34d399','#f5a623','#ef5350','#a78bfa','#f472b6','#38bdf8','#fbbf24','#818cf8','#fb923c','#22d3ee','#a3e635','#e879f9','#94a3b8'],
  cores: ['#4da6ff','#34d399','#f5a623','#ef5350','#a78bfa','#f472b6','#38bdf8','#fbbf24','#818cf8','#fb923c','#22d3ee','#a3e635','#e879f9','#94a3b8','#fb7185','#4ade80'],
};

// --- Main Views ---
function renderOverview() {
  const latest = entries[entries.length - 1];
  const prev = Data.prevOf(latest);
  const sum = latest.summary || {};
  const prevSum = prev ? (prev.summary || {}) : {};
  
  metaEl.innerHTML = `Latest: <a href="#${Utils.sha7(latest)}">${Utils.sha7(latest)}</a> ${Utils.esc(Utils.commitMsg(latest))} · ${new Date(latest.timestamp).toLocaleString()}`;

  let html = `<div class="cards">
    ${UI.Card('Total RPS', Utils.fmt(sum.totalRPS), Utils.delta(sum.totalRPS||0, prevSum.totalRPS, false), 'req/s')}
    ${UI.Card('Error Rate', (sum.totalErr??0)+'%', Utils.delta(sum.totalErr||0, prevSum.totalErr, true))}
    ${UI.Card('Total Requests', Utils.fmt(sum.totalRequests), Utils.delta(sum.totalRequests||0, prevSum.totalRequests, false))}
    ${UI.Card('Scenarios', (latest.scenarios||[]).length, `<span class="d d-flat">${entries.length} runs</span>`)}
  </div>`;

  const categoryDefs = categoryDefsFromEntries(entries);
  html += UI.Section('Latency Trends (p95 ms)');
  categoryDefs.forEach(cat => {
    html += UI.ChartPanel(trendCanvasId(cat.id), `${Utils.esc(cat.label)} — p95 latency`, {full: true});
  });

  html += UI.Section('All Runs');
  html += UI.Panel('Click a row to see details', '<div id="histTable"></div>');
  
  root.innerHTML = html;

  // Initialize Trends
  const lbl = entries.map(Utils.sha7);
  categoryDefs.forEach(cat => {
    const list = getCatScenarios(cat.id, entries);
    if (!list.length || entries.length < 2) return;
    const ds = list.map(({name, tunnel}, i) => ({
      label: name.replace(/_/g, ' '),
      data: entries.map(e => { const s = (e.scenarios||[]).find(x => x.name === name && (x.tunnel||'duotunnel') === tunnel); return s ? s.p95 : null; }),
      borderColor: COLORS.palette[i % COLORS.palette.length],
      backgroundColor: 'transparent',
      borderDash: tunnel === 'frp' ? [4,3] : [],
      tension: .3, pointRadius: 2, borderWidth: 1.5, spanGaps: true,
    }));
    Charts.create(trendCanvasId(cat.id), {type:'line', data:{labels:lbl, datasets:ds}, options:{
      ...Charts.DEFAULT_OPTS,
      onClick(evt, elems) { if (elems.length) location.hash = lbl[elems[0].index]; },
      scales:{...Charts.DEFAULT_OPTS.scales, y:{...Charts.DEFAULT_OPTS.scales.y, title:{display:true,text:'ms',color:'#6b7d93'}}},
    }});
  });

  // Initialize History Table
  const tableRows = entries.slice().reverse().map((e, i) => {
    const p = entries[entries.length - 2 - i];
    const s7 = Utils.sha7(e);
    let dR='—', dE='—';
    if (p) {
      const r=e.summary?.totalRPS??0, pr=p.summary?.totalRPS??0;
      const er=e.summary?.totalErr??0, per=p.summary?.totalErr??0;
      if (pr>0) { const d=Utils.r2(r-pr); dR=`<span class="${d>0?'c-good':d<0?'c-bad':'c-dim'}">${d>0?'+':''}${d}</span>`; }
      const dEv=Utils.r2(er-per); dE=`<span class="${dEv<0?'c-good':dEv>0?'c-bad':'c-dim'}">${dEv>0?'+':''}${dEv}%</span>`;
    }
    return {
      className: 'clickable' + (i === 0 ? ' row-active' : ''),
      attribs: `data-sha="${s7}"`,
      cells: [s7, new Date(e.timestamp).toLocaleString(undefined, {year:'numeric',month:'numeric',day:'numeric',hour:'2-digit',minute:'2-digit',hour12:false}), Utils.fmt(e.summary?.totalRPS??0), dR, (e.summary?.totalErr??0)+'%', dE, Utils.fmt(e.summary?.totalRequests??0), (e.scenarios||[]).length]
    };
  });
  document.getElementById('histTable').innerHTML = UI.Table(['Commit','Date','RPS','Δ','Err %','Δ','Reqs','Scenarios'], tableRows);
  document.getElementById('histTable').addEventListener('click', e => { const tr = e.target.closest('tr.clickable'); if (tr) location.hash = tr.dataset.sha; });
}

function renderDetail(entry) {
  const prev = Data.prevOf(entry);
  const s7 = Utils.sha7(entry);
  const sum = entry.summary || {};
  const prevSum = prev ? (prev.summary || {}) : {};
  const artifacts = entry.artifacts || {};
  const runUrl = entry.run_url || artifacts.run_url || '';

  metaEl.innerHTML = `<a href="${Utils.commitUrl(entry)}" target="_blank">${s7}</a> ${runUrl ? `<a href="${Utils.esc(runUrl)}" target="_blank" style="font-size:11px;color:var(--text2)">[CI run]</a>` : ''} ${Utils.esc(Utils.commitMsg(entry))}`;

  let html = UI.Breadcrumb(s7);
  html += `<div class="cards">
    ${UI.Card('Total RPS', Utils.fmt(sum.totalRPS), Utils.delta(sum.totalRPS||0, prevSum.totalRPS, false), 'req/s')}
    ${UI.Card('Error Rate', (sum.totalErr??0)+'%', Utils.delta(sum.totalErr||0, prevSum.totalErr, true))}
    ${UI.Card('Total Requests', Utils.fmt(sum.totalRequests), Utils.delta(sum.totalRequests||0, prevSum.totalRequests, false))}
    ${UI.Card('Scenarios', (entry.scenarios||[]).length, `<span class="d d-flat">${prev ? 'vs ' + Utils.sha7(prev) : 'first run'}</span>`)}
  </div>`;

  if (artifacts.trace_server || artifacts.trace_client || (artifacts.trace_cases||[]).length) {
    const links = [];
    if (artifacts.trace_server) links.push({label:'Server trace', url:artifacts.trace_server});
    if (artifacts.trace_client) links.push({label:'Client trace', url:artifacts.trace_client});
    (artifacts.trace_cases||[]).forEach(tc => {
      links.push({label:`${tc.case} (srv)`, url: `viewer/?trace=${encodeURIComponent(tc.server)}`});
      links.push({label:`${tc.case} (cli)`, url: `viewer/?trace=${encodeURIComponent(tc.client)}`});
    });
    html += UI.Panel('Dial9 Traces', UI.LinkGroup(links));
  }

  const categoryDefs = categoryDefsFromEntry(entry);
  const scenarios = entry.scenarios || [];
  const maxP95 = Math.max(...scenarios.map(s => s.p95 || 0), 1);
  const prevScMap = {};
  if (prev) (prev.scenarios||[]).forEach(s => { prevScMap[(s.tunnel||'duotunnel')+':'+s.name] = s; });

  html += UI.Section('Scenario Results');
  let tableRows = [];
  const groupColumns = (scenarios) => {
    const base = ['Scenario', 'Proto', 'Dir', 'Time', 'Target', 'Threshold', 'p50', 'p95', 'Latency', 'RPS', 'Reqs', 'Err %'];
    return prev ? [...base, 'Δ p50', 'Δ p95', 'Δ RPS'] : base;
  };
  const headers = groupColumns(scenarios);

  categoryDefs.forEach(cat => {
    const group = scenarios.filter(s => s.category === cat.id);
    if (!group.length) return;
    tableRows.push({cells:[{content: `<div style="font-size:10px;font-weight:700;color:var(--text3);text-transform:uppercase">${cat.label}</div><div class="cat-desc">${cat.description||''}</div>`, attribs: `colspan="${headers.length}"`}], className:'no-hover'});
    group.forEach(s => {
      const ps = prevScMap[(s.tunnel||'duotunnel')+':'+s.name];
      const cells = [
        `${scenarioLabel(s)}${s.tunnel==='frp'?' <span class="pill pill-ws">frp</span>':''}`,
        Utils.pill(s.protocol), s.direction, Utils.fmtTimeRange(s.timeRange),
        s.targetRate ? Utils.fmt(s.targetRate)+' rps' : '—', s.thresholdSpec || '—',
        s.p50+' ms', s.p95+' ms', Utils.latencyBar(s.p95, maxP95, s.protocol)+s.p95+' ms',
        Utils.fmt(s.rps), Utils.fmt(s.requests), `<span class="${s.err>0?'c-bad':'c-dim'}">${s.err}%</span>`
      ];
      if (prev) {
        cells.push(ps ? Utils.deltaVal(s.p50, ps.p50, true) : '—');
        cells.push(ps ? Utils.deltaVal(s.p95, ps.p95, true) : '—');
        cells.push(ps ? Utils.deltaVal(s.rps, ps.rps, false) : '—');
      }
      tableRows.push({cells});
    });
  });
  html += UI.Panel(`Detailed Metrics — ${s7}`, UI.Table(headers, tableRows));

  // Resources (Adaptive Container)
  if (rpcKeys.length) {
    html += UI.Section('System Resources');
    html += `<div id="res-charts-container"></div>`;
    root.innerHTML = html;
    initResourceCharts(entry.resources_per_case || { core: entry.resources, catalog: entry.catalog }, null);
  } else {
    root.innerHTML = html;
  }
}

function initResourceCharts(rpc, _) {
  const catalog = rpc.catalog || rpc.core?.catalog || { metrics: ['cpu', 'rss'], groups: ['server', 'client'] };
  const rpcEntries = Object.entries(rpc).filter(([k]) => k !== 'catalog' && k !== 'nproc' && k !== 'k6OffsetSeconds');
  
  const CASE_GAP = 5;
  let runningOffset = 0;
  const caseOffsets = rpcEntries.map(([, res]) => {
    const off = runningOffset;
    const pts = [...(res.system?.cpu||[]), ...Object.values(res.processes||{}).flatMap(p=>p.cpu||[])];
    const maxT = pts.length ? Math.max(...pts.map(p=>p[0]??0)) : 0;
    runningOffset += maxT + CASE_GAP;
    return {off, maxT};
  });

  const shift = (pts, off) => pts.map(p => ({x:(p[0]??0)+off, y:p[1]}));
  const merge = (key, subGroup) => {
    const all = [];
    rpcEntries.forEach(([, res], i) => {
      const pts = subGroup ? res.processes?.[subGroup]?.[key] : res.system?.[key];
      if (pts) { all.push(...shift(pts, caseOffsets[i].off), {x:caseOffsets[i].off+caseOffsets[i].maxT+0.01, y:null}); }
    });
    return all;
  };

  const metricDefs = catalog.metrics || [];
  const METRIC_LOOKUP = {};
  metricDefs.forEach(m => METRIC_LOOKUP[m.id || m] = m);

  metricDefs.forEach(mDef => {
    const mKey = mDef.id || mDef;
    const meta = METRIC_LOOKUP[mKey] || { label: mKey, unit: '' };
    const title = meta.label || meta.id || mKey;
    const unit = meta.unit || '';
    const datasets = [];
    
    const groupDefs = catalog.groups || [];
    const GROUP_LOOKUP = {};
    groupDefs.forEach(g => GROUP_LOOKUP[g.id || g] = g);

    // Process groups
    (catalog.groups || []).forEach((gObj, idx) => {
      const gId = gObj.id || gObj;
      const gLabel = gObj.label || gId;
      const data = merge(mKey, gId);
      if (data.length) {
        datasets.push({
          label: gLabel,
          data, borderColor: COLORS.palette[idx % COLORS.palette.length],
          backgroundColor: 'transparent', tension: .3, pointRadius: 0, borderWidth: 1.5
        });
      }
    });

    // System total
    const sysData = merge(mKey, null);
    if (sysData.length) {
      datasets.push({
        label: 'System Total',
        data: sysData, borderColor: '#ffffff', borderDash: [4,4],
        backgroundColor: 'transparent', tension: .3, pointRadius: 0, borderWidth: 1.5
      });
    }

    if (datasets.length) {
      // Create chart panel if it doesn't exist (adaptive UI)
      const chartId = 'res_chart_' + mKey;
      if (!document.getElementById(chartId)) {
          const container = document.getElementById('res-charts-container');
          if (container) container.innerHTML += UI.ChartPanel(chartId, title);
      }
      const common = { ...Charts.DEFAULT_OPTS, scales: { ...Charts.DEFAULT_OPTS.scales, x: { type: 'linear', title: {display:true, text:'seconds'} }, y: { ...Charts.DEFAULT_OPTS.scales.y, title: {display: !!unit, text: unit} } } };
      Charts.create(chartId, {type:'line', data:{datasets}, options:common});

    }
  });
}

// --- Helper Data Mappers ---
function getCatScenarios(catId, entryList) {
  const seen = new Set();
  const list = [];
  entryList.forEach(e => (e.scenarios||[]).forEach(s => {
    if (s.category !== catId || s.name === 'ws_multi_msg') return;
    const key = (s.tunnel||'duotunnel') + ':' + s.name;
    if (!seen.has(key)) { seen.add(key); list.push({name:s.name, tunnel:s.tunnel||'duotunnel'}); }
  }));
  return list.sort((a,b) => (a.tunnel===b.tunnel ? a.name.localeCompare(b.name) : (a.tunnel==='frp'?1:-1)));
}

function categoryDefsFromEntries(list) {
  const latest = list[list.length - 1];
  const catalog = (latest?.catalog?.categories || []).length ? latest.catalog.categories : [
    {id:'basic',label:'Basic',description:'Baseline latency at various rates.'},
    {id:'body_size',label:'Body Size',description:'Throughput with different payload sizes.'},
    {id:'stress',label:'Stress',description:'High concurrency stress testing.'}
  ];
  return catalog;
}
function categoryDefsFromEntry(e) { return categoryDefsFromEntries([e]); }
function trendCanvasId(id) { return 'trend_' + id.replace(/[^a-z0-9]/gi, '_'); }
function scenarioLabel(s) { return s.label || s.name.replace(/_/g, ' '); }
Utils.fmtTimeRange = (tr) => tr ? `${tr.startSec}–${tr.endSec}s` : '—';

function navigate() {
  const hash = location.hash.replace('#','');
  Charts.destroyAll();
  if (hash && hash !== 'overview') {
    const entry = Data.findEntry(hash);
    if (entry) {
      root.innerHTML = '<div class="empty">Loading...</div>';
      Data.loadDetail(entry).then(renderDetail);
      return;
    }
  }
  renderOverview();
}

window.addEventListener('hashchange', navigate);
navigate();

})();
