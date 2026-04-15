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
      item.addEventListener('click', (e) => {
        if (e.metaKey || e.ctrlKey) {
          chart.data.datasets[i].hidden = !chart.data.datasets[i].hidden;
        } else {
          const isSolo = chart.data.datasets.every((d, j) => j === i ? !d.hidden : d.hidden);
          chart.data.datasets.forEach((d, j) => { d.hidden = isSolo ? false : j !== i; });
        }
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

  const allCases = Object.values(entry.phases || {})
    .flatMap(p => Object.entries(p.cases || {}).map(([name, c]) => ({name, ...c})));

  let html = UI.Breadcrumb(s7);
  html += `<div class="cards">
    ${UI.Card('Total RPS', Utils.fmt(sum.totalRPS), Utils.delta(sum.totalRPS||0, prevSum.totalRPS, false), 'req/s')}
    ${UI.Card('Error Rate', (sum.totalErr??0)+'%', Utils.delta(sum.totalErr||0, prevSum.totalErr, true))}
    ${UI.Card('Total Requests', Utils.fmt(sum.totalRequests), Utils.delta(sum.totalRequests||0, prevSum.totalRequests, false))}
    ${UI.Card('Scenarios', allCases.length, `<span class="d d-flat">${prev ? 'vs ' + Utils.sha7(prev) : 'first run'}</span>`)}
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
  const maxP95 = Math.max(...allCases.map(c => (c.perf||{}).p95 || 0), 1);
  const prevCaseMap = {};
  if (prev) {
    Object.values(prev.phases || {}).forEach(p =>
      Object.entries(p.cases || {}).forEach(([name, c]) => {
        prevCaseMap[(c.tunnel||'duotunnel')+':'+name] = c;
      })
    );
  }

  html += UI.Section('Scenario Results');
  let tableRows = [];
  const base = ['Scenario', 'Proto', 'Dir', 'Time', 'Target', 'Threshold', 'p50', 'p95', 'Latency', 'RPS', 'Reqs', 'Err %'];
  const headers = prev ? [...base, 'Δ p50', 'Δ p95', 'Δ RPS'] : base;

  categoryDefs.forEach(cat => {
    const group = allCases.filter(c => c.category === cat.id);
    if (!group.length) return;
    tableRows.push({cells:[{content: `<div style="font-size:10px;font-weight:700;color:var(--text3);text-transform:uppercase">${cat.label}</div><div class="cat-desc">${cat.description||''}</div>`, attribs: `colspan="${headers.length}"`}], className:'no-hover'});
    group.forEach(c => {
      const perf = c.perf || {};
      const pc = prevCaseMap[(c.tunnel||'duotunnel')+':'+c.name];
      const pp = pc ? (pc.perf || {}) : null;
      const cells = [
        `${scenarioLabel(c)}${c.tunnel==='frp'?' <span class="pill pill-ws">frp</span>':''}`,
        Utils.pill(c.protocol), c.direction, Utils.fmtTimeRange(perf.timeRange),
        perf.targetRate ? Utils.fmt(perf.targetRate)+' rps' : '—', perf.thresholdSpec || '—',
        perf.p50+' ms', perf.p95+' ms', Utils.latencyBar(perf.p95, maxP95, c.protocol)+perf.p95+' ms',
        Utils.fmt(perf.rps), Utils.fmt(perf.requests), `<span class="${perf.err>0?'c-bad':'c-dim'}">${perf.err}%</span>`
      ];
      if (prev) {
        cells.push(pp ? Utils.deltaVal(perf.p50, pp.p50, true) : '—');
        cells.push(pp ? Utils.deltaVal(perf.p95, pp.p95, true) : '—');
        cells.push(pp ? Utils.deltaVal(perf.rps, pp.rps, false) : '—');
      }
      tableRows.push({cells});
    });
  });
  html += UI.Panel(`Detailed Metrics — ${s7}`, UI.Table(headers, tableRows));

  const casesWithRes = allCases.filter(c => c.resources);
  if (casesWithRes.length) {
    html += UI.Section('System Resources');
    html += `<div id="res-charts-container"></div>`;
    root.innerHTML = html;
    buildResourceCharts(casesWithRes);
  } else {
    root.innerHTML = html;
  }
}

const PSERIES = [
  {key:'server',    label:'Server',    color:'#4da6ff'},
  {key:'client',    label:'Client',    color:'#34d399'},
  {key:'ctld',      label:'Control',   color:'#22d3ee'},
  {key:'http_echo', label:'HTTP Echo', color:'#a78bfa'},
  {key:'ws_echo',   label:'WS Echo',   color:'#f472b6'},
  {key:'grpc_echo', label:'gRPC Echo', color:'#fb923c'},
  {key:'k6',        label:'k6',        color:'#f5a623'},
  {key:'frps',      label:'frps',      color:'#38bdf8'},
  {key:'frpc',      label:'frpc',      color:'#a3e635'},
  {key:'other',     label:'Other',     color:'#4a5a6d'},
];
const CPU_BREAKDOWN = [
  {key:'cpu_usr',    label:'usr',           color:'#4da6ff'},
  {key:'cpu_sys',    label:'sys',           color:'#34d399'},
  {key:'cpu_soft',   label:'soft(net irq)', color:'#ef5350'},
  {key:'cpu_irq',    label:'irq',           color:'#f5a623'},
  {key:'cpu_iowait', label:'iowait',        color:'#a78bfa'},
  {key:'cpu_steal',  label:'steal',         color:'#4a5a6d'},
];

let _chartIdSeq = 0;
function buildResourceCharts(cases) {
  const pfx = 'rc' + (++_chartIdSeq) + '_';
  const CASE_COLORS = ['#4da6ff','#34d399','#f5a623','#ef5350','#a78bfa','#f472b6','#38bdf8','#fbbf24','#a3e635','#fb923c'];
  const GAP = 5;

  // compute per-case x offset: each case starts after the previous case's max t + GAP
  const offsets = [];
  let cursor = 0;
  for (const c of cases) {
    offsets.push(cursor);
    const cSys = c.resources.system || {};
    const cProc = c.resources.processes || {};
    const allPts = [...(cSys.cpu||[]), ...Object.values(cProc).flatMap(p => p.cpu||[])];
    const maxT = allPts.length ? Math.max(...allPts.map(p => p[0]||0)) : 0;
    cursor += maxT + GAP;
  }

  const pts = (arr, off) => (arr||[]).map(p => ({x: (p[0]||0) + off, y: p[1]}));

  // per-PSERIES datasets keyed by series key, value = array of {label, data, color}
  const cpuByKey={}, memByKey={}, vmsByKey={}, csVolByKey={}, csInvolByKey={};
  let hasCpu=false, hasMem=false, hasVms=false, hasCs=false;
  // system-level datasets
  const sysCpuDs=[], sysBdDs={};
  const tcpEstabDs=[], tcpTwDs=[], tcpOtherDs=[];
  let hasTcp=false;
  const diskDs=[];
  let hasDisk=false;
  const intrDs=[], ctxDs=[];
  let hasIntr=false;
  const udpDs=[];
  let hasUdp=false;
  const netDs=[], netPktDs=[];
  let hasNet=false, hasNetPkts=false;
  const loadDs=[], memMbDs=[];
  let hasLoad=false;
  const topCpuDs=[], topRssDs=[];
  let hasTopCpu=false, hasTopRss=false;
  // per-core: keyed by core index, value = array of points across all cases
  const perCoreByIdx={};
  let hasPerCore=false;

  for (const s of PSERIES) { cpuByKey[s.key]=[]; memByKey[s.key]=[]; vmsByKey[s.key]=[]; csVolByKey[s.key]=[]; csInvolByKey[s.key]=[]; }

  cases.forEach((c, ci) => {
    const off = offsets[ci];
    const color = CASE_COLORS[ci % CASE_COLORS.length];
    const caseName = c.label || c.name;
    const res = c.resources;
    const cProc = res.processes || {};
    const cSys  = res.system || {};
    const g = (arr) => [...pts(arr, off), {x: off + (arr.length ? (arr[arr.length-1][0]||0) : 0) + 0.01, y: null}];

    for (const s of PSERIES) {
      const p = cProc[s.key];
      if (p?.cpu?.length)     { cpuByKey[s.key].push({label:`${caseName}`, data:g(p.cpu), color}); hasCpu=true; }
      if (p?.rss?.length)     { memByKey[s.key].push({label:`${caseName}`, data:g(p.rss), color}); hasMem=true; }
      if (p?.vms?.length)     { vmsByKey[s.key].push({label:`${caseName}`, data:g(p.vms), color}); hasVms=true; }
      if (p?.cswch?.length)   { csVolByKey[s.key].push({label:`${caseName}`,   data:g(p.cswch),   color}); hasCs=true; }
      if (p?.nvcswch?.length) { csInvolByKey[s.key].push({label:`${caseName}`, data:g(p.nvcswch), color}); hasCs=true; }
    }

    if (cSys.cpu?.length) sysCpuDs.push({label:`${caseName}`, data:g(cSys.cpu), color});
    for (const bd of CPU_BREAKDOWN) {
      if (cSys[bd.key]?.length) {
        if (!sysBdDs[bd.key]) sysBdDs[bd.key]=[];
        sysBdDs[bd.key].push({label:`${caseName}`, data:g(cSys[bd.key]), color});
      }
    }

    const estab = cSys.tcp_ESTABLISHED || cSys.tcp_estab || [];
    const tw    = cSys.tcp_TIME_WAIT   || cSys.tcp_timewait || [];
    if (estab.length) { tcpEstabDs.push({label:`${caseName}`, data:g(estab), color}); hasTcp=true; }
    if (tw.length)    { tcpTwDs.push({label:`${caseName}`,    data:g(tw),    color}); hasTcp=true; }
    for (const [k, lbl] of [['tcp_LISTEN','LISTEN'],['tcp_CLOSE_WAIT','CLOSE_WAIT'],['tcp_FIN_WAIT1','FIN_WAIT1'],['tcp_FIN_WAIT2','FIN_WAIT2'],['tcp_SYN_SENT','SYN_SENT'],['tcp_SYN_RECV','SYN_RECV'],['tcp_LAST_ACK','LAST_ACK']]) {
      if (cSys[k]?.length) tcpOtherDs.push({label:`${caseName} ${lbl}`, data:g(cSys[k]), color});
    }

    if (cSys.disk_read_kbs?.length)   { diskDs.push({label:`${caseName} read KB/s`,  data:g(cSys.disk_read_kbs),   color, axis:'yDKbs'}); hasDisk=true; }
    if (cSys.disk_write_kbs?.length)  { diskDs.push({label:`${caseName} write KB/s`, data:g(cSys.disk_write_kbs),  color, axis:'yDKbs'}); hasDisk=true; }
    if (cSys.disk_read_iops?.length)   diskDs.push({label:`${caseName} read IOPS`,   data:g(cSys.disk_read_iops),  color, axis:'yDIops', dash:[3,2]});
    if (cSys.disk_write_iops?.length)  diskDs.push({label:`${caseName} write IOPS`,  data:g(cSys.disk_write_iops), color, axis:'yDIops', dash:[3,2]});

    if (cSys.interrupts?.length)   { intrDs.push({label:`${caseName}`, data:g(cSys.interrupts),   color, axis:'yIntr'}); hasIntr=true; }
    if (cSys.ctx_switches?.length)   ctxDs.push({label:`${caseName}`,  data:g(cSys.ctx_switches), color, axis:'yCtx', dash:[3,2]});

    if (cSys.udp_rx_err?.length)  { udpDs.push({label:`${caseName} InErrors/s`,   data:g(cSys.udp_rx_err),  color}); hasUdp=true; }
    if (cSys.udp_buf_err?.length)   udpDs.push({label:`${caseName} RcvbufErr/s`,  data:g(cSys.udp_buf_err), color, dash:[3,2]});

    if (cSys.net_rx_kbs?.length)  { netDs.push({label:`${caseName} RX KB/s`, data:g(cSys.net_rx_kbs),  color}); hasNet=true; }
    if (cSys.net_tx_kbs?.length)  { netDs.push({label:`${caseName} TX KB/s`, data:g(cSys.net_tx_kbs),  color}); hasNet=true; }
    if (cSys.net_rx_pkts?.length) { netPktDs.push({label:`${caseName} RX pkts/s`, data:g(cSys.net_rx_pkts), color}); hasNetPkts=true; }
    if (cSys.net_tx_pkts?.length) { netPktDs.push({label:`${caseName} TX pkts/s`, data:g(cSys.net_tx_pkts), color}); hasNetPkts=true; }
    if (cSys.net_drop_in?.length)   netPktDs.push({label:`${caseName} drop_in/s`,  data:g(cSys.net_drop_in),  color, dash:[3,2]});
    if (cSys.net_drop_out?.length)  netPktDs.push({label:`${caseName} drop_out/s`, data:g(cSys.net_drop_out), color, dash:[3,2]});

    if (cSys.loadavg_1?.length)   { loadDs.push({label:`${caseName} load 1m`,  data:g(cSys.loadavg_1),  color}); hasLoad=true; }
    if (cSys.loadavg_5?.length)     loadDs.push({label:`${caseName} load 5m`,  data:g(cSys.loadavg_5),  color, dash:[4,2]});
    if (cSys.loadavg_15?.length)    loadDs.push({label:`${caseName} load 15m`, data:g(cSys.loadavg_15), color, dash:[4,2]});
    if (cSys.mem_used_mb?.length) { memMbDs.push({label:`${caseName} mem MB`,  data:g(cSys.mem_used_mb), color}); hasLoad=true; }
    if (cSys.swap_used_mb?.length)  memMbDs.push({label:`${caseName} swap MB`, data:g(cSys.swap_used_mb), color, dash:[2,2]});

    for (const [name, arr] of Object.entries(res.top_cpu || {})) {
      topCpuDs.push({label:`${caseName} ${name}`, data:g(arr), color}); hasTopCpu=true;
    }
    for (const [name, arr] of Object.entries(res.top_rss || {})) {
      topRssDs.push({label:`${caseName} ${name}`, data:g(arr), color}); hasTopRss=true;
    }

    const perCoreRaw = cSys.cpu_per_core;
    if (Array.isArray(perCoreRaw) && perCoreRaw.length) {
      const first = perCoreRaw[0];
      if (Array.isArray(first) && Array.isArray(first[0])) {
        perCoreRaw.forEach((arr, ki) => {
          if (!perCoreByIdx[ki]) perCoreByIdx[ki] = {label:`Core ${ki}`, data:[], color: CASE_COLORS[ki % CASE_COLORS.length]};
          perCoreByIdx[ki].data.push(...g(arr));
        });
      } else {
        const byCi = {};
        perCoreRaw.forEach(([t, cores]) => cores.forEach((v, ki) => {
          if (!byCi[ki]) byCi[ki]=[];
          byCi[ki].push([t, v]);
        }));
        Object.entries(byCi).forEach(([ki, arr]) => {
          if (!perCoreByIdx[ki]) perCoreByIdx[ki] = {label:`Core ${ki}`, data:[], color: CASE_COLORS[ki % CASE_COLORS.length]};
          perCoreByIdx[ki].data.push(...g(arr));
        });
      }
      hasPerCore = true;
    }
  });

  const CO = {
    ...Charts.DEFAULT_OPTS, showLine: true,
    plugins: {...Charts.DEFAULT_OPTS.plugins, tooltip: {mode:'index', intersect:false}},
    scales: {
      x: {type:'linear', title:{display:true, text:'seconds', color:'#6b7d93'}, ticks:{color:'#6b7d93',font:{size:9}}, grid:{color:'rgba(30,42,58,.5)'}},
      y: {...Charts.DEFAULT_OPTS.scales.y},
    },
  };
  const withY = (text) => ({scales: {...CO.scales, y: {...CO.scales.y, title:{display:true, text, color:'#6b7d93'}}}});
  const mkDs = (d, extra={}) => ({label:d.label, data:d.data, borderColor:d.color, backgroundColor:'transparent', tension:.3, pointRadius:0, borderWidth:1.5, ...(d.dash ? {borderDash:d.dash} : {}), ...extra});

  const container = document.getElementById('res-charts-container');
  const addChart = (id, title) => {
    container.insertAdjacentHTML('beforeend', `<div class="row full"><div class="panel tall"><h3>${title}</h3><div class="chart-area"><canvas id="${id}"></canvas></div><div id="${id}-leg" class="leg-wrap"></div></div></div>`);
  };

  {
    addChart(pfx+'cpu', 'CPU Usage (% of machine)');
    const ds=[];
    for (const s of PSERIES) {
      cpuByKey[s.key].forEach(d => ds.push({...mkDs(d), backgroundColor:d.color+'28', fill:true}));
    }
    for (const bd of CPU_BREAKDOWN) {
      (sysBdDs[bd.key]||[]).forEach(d => ds.push({...mkDs(d), borderDash:[3,2], borderWidth:1, hidden:true}));
    }
    sysCpuDs.forEach(d => ds.push({...mkDs(d), borderDash:[4,3], borderWidth:1}));
    Charts.create(pfx+'cpu', {type:'line', data:{datasets:ds}, options:{...CO, scales:{...CO.scales, y:{...CO.scales.y, stacked:false, min:0, max:100, title:{display:true,text:'% of machine',color:'#6b7d93'}}}}});
  }

  if (hasPerCore) {
    addChart(pfx+'percore', 'Per-core CPU (%)');
    const ds = Object.values(perCoreByIdx).map(d => mkDs(d));
    Charts.create(pfx+'percore', {type:'line', data:{datasets:ds}, options:{...CO, scales:{...CO.scales, y:{...CO.scales.y, min:0, max:100, title:{display:true,text:'%',color:'#6b7d93'}}}}});
  }

  if (hasMem) {
    addChart(pfx+'mem', 'Memory RSS (MB)');
    const ds=[];
    for (const s of PSERIES) { if (s.key !== 'other') memByKey[s.key].forEach(d => ds.push(mkDs(d))); }
    Charts.create(pfx+'mem', {type:'line', data:{datasets:ds}, options:{...CO, ...withY('MB')}});
  }

  if (hasLoad) {
    addChart(pfx+'load', 'Load Average + Memory + Swap');
    const ds = [...loadDs.map(d=>({...mkDs(d),yAxisID:'yLoad'})), ...memMbDs.map(d=>({...mkDs(d),yAxisID:'yMem'}))];
    Charts.create(pfx+'load', {type:'line', data:{datasets:ds}, options:{...CO, scales:{...CO.scales, yLoad:{position:'left',title:{display:true,text:'load',color:'#6b7d93'},ticks:{color:'#6b7d93',font:{size:9}},grid:{color:'rgba(30,42,58,.5)'}}, yMem:{position:'right',title:{display:true,text:'MB',color:'#6b7d93'},ticks:{color:'#6b7d93',font:{size:9}},grid:{drawOnChartArea:false}}}}});
  }

  if (hasNet) {
    addChart(pfx+'net', 'Network Throughput (KB/s)');
    Charts.create(pfx+'net', {type:'line', data:{datasets:netDs.map(mkDs)}, options:{...CO, ...withY('KB/s')}});
  }

  if (hasNetPkts) {
    addChart(pfx+'netpkts', 'Network Packets/s + Drops');
    Charts.create(pfx+'netpkts', {type:'line', data:{datasets:netPktDs.map(mkDs)}, options:{...CO, ...withY('/s')}});
  }

  if (hasTcp) {
    addChart(pfx+'tcp', 'TCP Connections');
    const ds = [...tcpEstabDs.map(d=>({...mkDs(d),yAxisID:'yConn'})), ...tcpTwDs.map(d=>({...mkDs(d),yAxisID:'yTW'})), ...tcpOtherDs.map(d=>({...mkDs(d),borderDash:[3,2],borderWidth:1,yAxisID:'yConn'}))];
    Charts.create(pfx+'tcp', {type:'line', data:{datasets:ds}, options:{...CO, scales:{...CO.scales, yConn:{position:'left',title:{display:true,text:'conns',color:'#6b7d93'},ticks:{color:'#6b7d93',font:{size:9}},grid:{color:'rgba(30,42,58,.5)'}}, yTW:{position:'right',title:{display:true,text:'TIME_WAIT',color:'#6b7d93'},ticks:{color:'#6b7d93',font:{size:9}},grid:{drawOnChartArea:false}}}}});
  }

  if (hasVms) {
    addChart(pfx+'vms', 'Virtual Memory Size (MB)');
    const ds=[];
    for (const s of PSERIES) { if (s.key !== 'other') vmsByKey[s.key].forEach(d => ds.push(mkDs(d))); }
    Charts.create(pfx+'vms', {type:'line', data:{datasets:ds}, options:{...CO, ...withY('MB')}});
  }

  if (hasDisk) {
    addChart(pfx+'disk', 'Disk I/O');
    Charts.create(pfx+'disk', {type:'line', data:{datasets:diskDs.map(d=>({...mkDs(d),yAxisID:d.axis}))}, options:{...CO, scales:{...CO.scales, yDKbs:{position:'left',title:{display:true,text:'KB/s',color:'#6b7d93'},ticks:{color:'#6b7d93',font:{size:9}},grid:{color:'rgba(30,42,58,.5)'}}, yDIops:{position:'right',title:{display:true,text:'IOPS',color:'#6b7d93'},ticks:{color:'#6b7d93',font:{size:9}},grid:{drawOnChartArea:false}}}}});
  }

  if (hasIntr) {
    addChart(pfx+'intr', 'Interrupts + Context Switches / s');
    const ds = [...intrDs.map(d=>({...mkDs(d),yAxisID:d.axis})), ...ctxDs.map(d=>({...mkDs(d),yAxisID:d.axis}))];
    Charts.create(pfx+'intr', {type:'line', data:{datasets:ds}, options:{...CO, scales:{...CO.scales, yIntr:{position:'left',title:{display:true,text:'interrupts/s',color:'#6b7d93'},ticks:{color:'#6b7d93',font:{size:9}},grid:{color:'rgba(30,42,58,.5)'}}, yCtx:{position:'right',title:{display:true,text:'ctx_switches/s',color:'#6b7d93'},ticks:{color:'#6b7d93',font:{size:9}},grid:{drawOnChartArea:false}}}}});
  }

  if (hasCs) {
    addChart(pfx+'ctxsw', 'Context Switches / s');
    const ds=[];
    for (const s of PSERIES) {
      csVolByKey[s.key].forEach(d => ds.push(mkDs(d)));
      csInvolByKey[s.key].forEach(d => ds.push({...mkDs(d), borderDash:[3,2], borderWidth:1}));
    }
    Charts.create(pfx+'ctxsw', {type:'line', data:{datasets:ds}, options:{...CO, ...withY('/s')}});
  }

  if (hasUdp) {
    addChart(pfx+'udp', 'UDP Errors / s');
    Charts.create(pfx+'udp', {type:'line', data:{datasets:udpDs.map(mkDs)}, options:{...CO, ...withY('/s')}});
  }

  if (hasTopCpu) {
    addChart(pfx+'topcpu', 'Top CPU (% of machine)');
    Charts.create(pfx+'topcpu', {type:'line', data:{datasets:topCpuDs.map(mkDs)}, options:{...CO, ...withY('% of machine')}});
  }

  if (hasTopRss) {
    addChart(pfx+'toprss', 'Top RSS (MB)');
    Charts.create(pfx+'toprss', {type:'line', data:{datasets:topRssDs.map(mkDs)}, options:{...CO, ...withY('MB')}});
  }
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
