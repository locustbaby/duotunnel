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

  const allCases = Object.entries(entry.cases || {})
    .map(([name, c]) => ({name, ...c}));

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
    Object.entries(prev.cases || {}).forEach(([name, c]) => {
      prevCaseMap[(c.tunnel||'duotunnel')+':'+name] = c;
    });
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

  const caseResources = allCases.filter(c => c.resources);
  if (caseResources.length) {
    html += UI.Section('System Resources');
    html += `<div id="res-charts-container"></div>`;
    root.innerHTML = html;
    if (caseResources.length === 1) {
      const c = caseResources[0];
      renderCaseResources(c.resources, c.label || c.name, (c.perf||{}).timeRange);
    } else {
      renderSerialCaseResources(caseResources);
    }
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
function buildResourceCharts(res, annotations, xMax, titlePrefix) {
  const prefix = titlePrefix ? titlePrefix + ' — ' : '';
  const pfx = 'rc' + (++_chartIdSeq) + '_';
  const cProc = res.processes || {};
  const cSys  = res.system || {};
  const pts = (arr) => (arr||[]).map(p => ({x: p[0], y: p[1]}));

  const CO = {
    ...Charts.DEFAULT_OPTS, showLine: true,
    plugins: {...Charts.DEFAULT_OPTS.plugins, annotation: {annotations: annotations || {}}, tooltip: {mode:'index', intersect:false}},
    scales: {
      x: {type:'linear', title:{display:true, text:'seconds', color:'#6b7d93'}, ticks:{color:'#6b7d93',font:{size:9}}, grid:{color:'rgba(30,42,58,.5)'}, ...(xMax != null ? {max: xMax} : {})},
      y: {...Charts.DEFAULT_OPTS.scales.y},
    },
  };
  const withY = (text) => ({scales: {...CO.scales, y: {...CO.scales.y, title:{display:true, text, color:'#6b7d93'}}}});
  const lp = (d, color, label, extra={}) => ({label, data:d, borderColor:color, backgroundColor:'transparent', tension:.3, pointRadius:0, borderWidth:1.5, ...extra});

  const container = document.getElementById('res-charts-container');
  const addChart = (id, title) => {
    container.insertAdjacentHTML('beforeend', `<div class="row full"><div class="panel tall"><h3>${title}</h3><div class="chart-area"><canvas id="${id}"></canvas></div><div id="${id}-leg" class="leg-wrap"></div></div></div>`);
  };

  const cpuByKey={}, memByKey={}, vmsByKey={}, csVolByKey={}, csInvolByKey={};
  let hasCpu=false, hasMem=false, hasVms=false, hasCs=false;

  for (const s of PSERIES) {
    const p = cProc[s.key];
    if (p?.cpu?.length)     { cpuByKey[s.key]     = pts(p.cpu);     hasCpu=true; }
    if (p?.rss?.length)     { memByKey[s.key]      = pts(p.rss);     hasMem=true; }
    if (p?.vms?.length)     { vmsByKey[s.key]      = pts(p.vms);     hasVms=true; }
    if (p?.cswch?.length)   { csVolByKey[s.key]    = pts(p.cswch);   hasCs=true; }
    if (p?.nvcswch?.length) { csInvolByKey[s.key]  = pts(p.nvcswch); hasCs=true; }
  }

  const sysCpu = pts(cSys.cpu);
  const sysBd = {};
  for (const bd of CPU_BREAKDOWN) { if (cSys[bd.key]?.length) sysBd[bd.key] = pts(cSys[bd.key]); }

  const estab    = pts(cSys.tcp_ESTABLISHED || cSys.tcp_estab    || []);
  const timewait = pts(cSys.tcp_TIME_WAIT   || cSys.tcp_timewait || []);
  const tcpListen    = pts(cSys.tcp_LISTEN);
  const tcpCW        = pts(cSys.tcp_CLOSE_WAIT);
  const tcpFW1       = pts(cSys.tcp_FIN_WAIT1);
  const tcpFW2       = pts(cSys.tcp_FIN_WAIT2);
  const tcpSS        = pts(cSys.tcp_SYN_SENT);
  const tcpSR        = pts(cSys.tcp_SYN_RECV);
  const tcpLA        = pts(cSys.tcp_LAST_ACK);
  const hasTcp = estab.length || timewait.length;

  const diskRdKbs  = pts(cSys.disk_read_kbs);
  const diskWrKbs  = pts(cSys.disk_write_kbs);
  const diskRdIops = pts(cSys.disk_read_iops);
  const diskWrIops = pts(cSys.disk_write_iops);
  const hasDisk = diskRdKbs.length || diskWrKbs.length;

  const intrPts  = pts(cSys.interrupts);
  const ctxSwPts = pts(cSys.ctx_switches);
  const hasIntr  = intrPts.length > 0;

  const udpRx  = pts(cSys.udp_rx_err);
  const udpBuf = pts(cSys.udp_buf_err);
  const hasUdp = udpRx.length || udpBuf.length;

  const rxKbs  = pts(cSys.net_rx_kbs);
  const txKbs  = pts(cSys.net_tx_kbs);
  const rxPkts = pts(cSys.net_rx_pkts);
  const txPkts = pts(cSys.net_tx_pkts);
  const dropIn  = pts(cSys.net_drop_in);
  const dropOut = pts(cSys.net_drop_out);
  const hasNet     = rxKbs.length || txKbs.length;
  const hasNetPkts = rxPkts.length || txPkts.length;

  const load1  = pts(cSys.loadavg_1);
  const load5  = pts(cSys.loadavg_5);
  const load15 = pts(cSys.loadavg_15);
  const swap   = pts(cSys.swap_used_mb);
  const memMb  = pts(cSys.mem_used_mb);
  const hasLoad = load1.length || memMb.length;

  const topCpu = res.top_cpu || {};
  const topRss = res.top_rss || {};
  const hasTopCpu = Object.keys(topCpu).length > 0;
  const hasTopRss = Object.keys(topRss).length > 0;

  const perCoreRaw = cSys.cpu_per_core;
  const perCoreSeries = [];
  if (Array.isArray(perCoreRaw) && perCoreRaw.length) {
    const first = perCoreRaw[0];
    if (Array.isArray(first) && Array.isArray(first[1])) {
      perCoreRaw.forEach(([t, cores]) => cores.forEach((v, ci) => {
        if (!perCoreSeries[ci]) perCoreSeries[ci]=[];
        perCoreSeries[ci].push({x:t, y:v});
      }));
    } else {
      perCoreRaw.forEach((arr, ci) => { perCoreSeries[ci] = pts(arr); });
    }
  }
  const hasPerCore = perCoreSeries.length > 0;

  {
    addChart(pfx+'cpu', prefix+'CPU Usage (% of machine)');
    const ds=[];
    for (const s of PSERIES) {
      if (!cpuByKey[s.key]?.length) continue;
      ds.push({label:s.label, data:cpuByKey[s.key], borderColor:s.color, backgroundColor:s.color+'28', fill:true, tension:.3, pointRadius:0, borderWidth:1.5});
    }
    for (const bd of CPU_BREAKDOWN) {
      if (sysBd[bd.key]?.length)
        ds.push({label:bd.label, data:sysBd[bd.key], borderColor:bd.color, backgroundColor:'transparent', fill:false, tension:.3, pointRadius:0, borderWidth:1, borderDash:[3,2], hidden:true});
    }
    if (sysCpu.length)
      ds.push({label:'System total', data:sysCpu, borderColor:'#ffffff', backgroundColor:'transparent', fill:false, tension:.3, pointRadius:0, borderWidth:1, borderDash:[4,3]});
    Charts.create(pfx+'cpu', {type:'line', data:{datasets:ds}, options:{...CO, scales:{...CO.scales, y:{...CO.scales.y, stacked:false, min:0, max:100, title:{display:true,text:'% of machine',color:'#6b7d93'}}}}});
  }

  if (hasPerCore) {
    addChart(pfx+'percore', prefix+'Per-core CPU (%)');
    const CC=['#4da6ff','#34d399','#f5a623','#ef5350','#a78bfa','#f472b6','#38bdf8','#fbbf24','#818cf8','#fb923c','#22d3ee','#a3e635','#e879f9','#94a3b8','#fb7185','#4ade80'];
    const ds = perCoreSeries.map((d,i)=>({label:`Core ${i}`, data:d, borderColor:CC[i%CC.length], backgroundColor:'transparent', tension:.3, pointRadius:0, borderWidth:1}));
    Charts.create(pfx+'percore', {type:'line', data:{datasets:ds}, options:{...CO, scales:{...CO.scales, y:{...CO.scales.y, min:0, max:100, title:{display:true,text:'%',color:'#6b7d93'}}}}});
  }

  if (hasMem) {
    addChart(pfx+'mem', prefix+'Memory RSS (MB)');
    const ds=[];
    for (const s of PSERIES) { if (s.key !== 'other' && memByKey[s.key]?.length) ds.push(lp(memByKey[s.key], s.color, s.label)); }
    Charts.create(pfx+'mem', {type:'line', data:{datasets:ds}, options:{...CO, ...withY('MB')}});
  }

  if (hasLoad) {
    addChart(pfx+'load', prefix+'Load Average + Memory + Swap');
    const ds=[];
    if (load1.length)  ds.push({...lp(load1,  '#4da6ff', 'load 1m'),               yAxisID:'yLoad'});
    if (load5.length)  ds.push({...lp(load5,  '#f5a623', 'load 5m'),               yAxisID:'yLoad'});
    if (load15.length) ds.push({...lp(load15, '#a78bfa', 'load 15m', {borderDash:[4,2]}), yAxisID:'yLoad'});
    if (memMb.length)  ds.push({...lp(memMb,  '#34d399', 'mem MB',   {borderDash:[3,1]}), yAxisID:'yMem'});
    if (swap.length)   ds.push({...lp(swap,   '#ef5350', 'swap MB',  {borderDash:[2,2]}), yAxisID:'yMem'});
    Charts.create(pfx+'load', {type:'line', data:{datasets:ds}, options:{...CO, scales:{...CO.scales, yLoad:{position:'left',title:{display:true,text:'load',color:'#6b7d93'},ticks:{color:'#6b7d93',font:{size:9}},grid:{color:'rgba(30,42,58,.5)'}}, yMem:{position:'right',title:{display:true,text:'MB',color:'#6b7d93'},ticks:{color:'#6b7d93',font:{size:9}},grid:{drawOnChartArea:false}}}}});
  }

  if (hasNet) {
    addChart(pfx+'net', prefix+'Network Throughput (KB/s)');
    const ds=[];
    if (rxKbs.length) ds.push(lp(rxKbs, '#34d399', 'RX KB/s'));
    if (txKbs.length) ds.push(lp(txKbs, '#4da6ff', 'TX KB/s'));
    Charts.create(pfx+'net', {type:'line', data:{datasets:ds}, options:{...CO, ...withY('KB/s')}});
  }

  if (hasNetPkts) {
    addChart(pfx+'netpkts', prefix+'Network Packets/s + Drops');
    const ds=[];
    if (rxPkts.length)  ds.push(lp(rxPkts,  '#34d399', 'RX pkts/s'));
    if (txPkts.length)  ds.push(lp(txPkts,  '#4da6ff', 'TX pkts/s'));
    if (dropIn.length)  ds.push(lp(dropIn,  '#ef5350', 'drop_in/s',  {borderDash:[3,2]}));
    if (dropOut.length) ds.push(lp(dropOut, '#f5a623', 'drop_out/s', {borderDash:[3,2]}));
    Charts.create(pfx+'netpkts', {type:'line', data:{datasets:ds}, options:{...CO, ...withY('/s')}});
  }

  if (hasTcp) {
    addChart(pfx+'tcp', prefix+'TCP Connections');
    const ds=[];
    const yL='yConn', yR='yTW';
    if (estab.length)     ds.push({...lp(estab,     '#4da6ff', 'ESTABLISHED'),                             yAxisID:yL});
    if (timewait.length)  ds.push({...lp(timewait,  '#f5a623', 'TIME_WAIT'),                               yAxisID:yR});
    if (tcpListen.length) ds.push({...lp(tcpListen, '#34d399', 'LISTEN',     {borderDash:[4,2],borderWidth:1}), yAxisID:yL});
    if (tcpCW.length)     ds.push({...lp(tcpCW,     '#ef5350', 'CLOSE_WAIT', {borderDash:[3,2],borderWidth:1}), yAxisID:yL});
    if (tcpFW1.length)    ds.push({...lp(tcpFW1,    '#a78bfa', 'FIN_WAIT1',  {borderDash:[3,2],borderWidth:1}), yAxisID:yL});
    if (tcpFW2.length)    ds.push({...lp(tcpFW2,    '#f472b6', 'FIN_WAIT2',  {borderDash:[3,2],borderWidth:1}), yAxisID:yL});
    if (tcpSS.length)     ds.push({...lp(tcpSS,     '#38bdf8', 'SYN_SENT',   {borderDash:[2,2],borderWidth:1}), yAxisID:yL});
    if (tcpSR.length)     ds.push({...lp(tcpSR,     '#fbbf24', 'SYN_RECV',   {borderDash:[2,2],borderWidth:1}), yAxisID:yL});
    if (tcpLA.length)     ds.push({...lp(tcpLA,     '#94a3b8', 'LAST_ACK',   {borderDash:[2,2],borderWidth:1}), yAxisID:yL});
    Charts.create(pfx+'tcp', {type:'line', data:{datasets:ds}, options:{...CO, scales:{...CO.scales, [yL]:{position:'left',title:{display:true,text:'conns',color:'#6b7d93'},ticks:{color:'#6b7d93',font:{size:9}},grid:{color:'rgba(30,42,58,.5)'}}, [yR]:{position:'right',title:{display:true,text:'TIME_WAIT',color:'#f5a623'},ticks:{color:'#f5a623',font:{size:9}},grid:{drawOnChartArea:false}}}}});
  }

  if (hasVms) {
    addChart(pfx+'vms', prefix+'Virtual Memory Size (MB)');
    const ds=[];
    for (const s of PSERIES) { if (s.key !== 'other' && vmsByKey[s.key]?.length) ds.push(lp(vmsByKey[s.key], s.color, s.label)); }
    Charts.create(pfx+'vms', {type:'line', data:{datasets:ds}, options:{...CO, ...withY('MB')}});
  }

  if (hasDisk) {
    addChart(pfx+'disk', prefix+'Disk I/O');
    const ds=[];
    if (diskRdKbs.length)  ds.push({...lp(diskRdKbs,  '#34d399', 'read KB/s'),                         yAxisID:'yDKbs'});
    if (diskWrKbs.length)  ds.push({...lp(diskWrKbs,  '#4da6ff', 'write KB/s'),                        yAxisID:'yDKbs'});
    if (diskRdIops.length) ds.push({...lp(diskRdIops, '#a78bfa', 'read IOPS',  {borderDash:[3,2],borderWidth:1}), yAxisID:'yDIops'});
    if (diskWrIops.length) ds.push({...lp(diskWrIops, '#f5a623', 'write IOPS', {borderDash:[3,2],borderWidth:1}), yAxisID:'yDIops'});
    Charts.create(pfx+'disk', {type:'line', data:{datasets:ds}, options:{...CO, scales:{...CO.scales, yDKbs:{position:'left',title:{display:true,text:'KB/s',color:'#6b7d93'},ticks:{color:'#6b7d93',font:{size:9}},grid:{color:'rgba(30,42,58,.5)'}}, yDIops:{position:'right',title:{display:true,text:'IOPS',color:'#6b7d93'},ticks:{color:'#6b7d93',font:{size:9}},grid:{drawOnChartArea:false}}}}});
  }

  if (hasIntr) {
    addChart(pfx+'intr', prefix+'Interrupts + Context Switches / s');
    const ds=[];
    if (intrPts.length)  ds.push({...lp(intrPts,  '#f472b6', 'interrupts/s'),                                yAxisID:'yIntr'});
    if (ctxSwPts.length) ds.push({...lp(ctxSwPts, '#38bdf8', 'ctx_switches/s', {borderDash:[3,2],borderWidth:1}), yAxisID:'yCtx'});
    Charts.create(pfx+'intr', {type:'line', data:{datasets:ds}, options:{...CO, scales:{...CO.scales, yIntr:{position:'left',title:{display:true,text:'interrupts/s',color:'#f472b6'},ticks:{color:'#f472b6',font:{size:9}},grid:{color:'rgba(30,42,58,.5)'}}, yCtx:{position:'right',title:{display:true,text:'ctx_switches/s',color:'#38bdf8'},ticks:{color:'#38bdf8',font:{size:9}},grid:{drawOnChartArea:false}}}}});
  }

  if (hasCs) {
    addChart(pfx+'ctxsw', prefix+'Context Switches / s');
    const ds=[];
    for (const s of PSERIES) {
      if (csVolByKey[s.key]?.length)   ds.push(lp(csVolByKey[s.key],   s.color, `${s.label} vol`));
      if (csInvolByKey[s.key]?.length) ds.push(lp(csInvolByKey[s.key], s.color, `${s.label} invol`, {borderDash:[3,2], borderWidth:1}));
    }
    Charts.create(pfx+'ctxsw', {type:'line', data:{datasets:ds}, options:{...CO, ...withY('/s')}});
  }

  if (hasUdp) {
    addChart(pfx+'udp', prefix+'UDP Errors / s');
    const ds=[];
    if (udpRx.length)  ds.push(lp(udpRx,  '#ef5350', 'InErrors/s'));
    if (udpBuf.length) ds.push(lp(udpBuf, '#f5a623', 'RcvbufErr/s', {borderDash:[3,2]}));
    Charts.create(pfx+'udp', {type:'line', data:{datasets:ds}, options:{...CO, ...withY('/s')}});
  }

  if (hasTopCpu) {
    addChart(pfx+'topcpu', prefix+'Top 10 CPU (% of machine)');
    const ds = Object.entries(topCpu).map(([name, arr], i) => ({
      label:name, data:pts(arr), borderColor:COLORS.palette[i%COLORS.palette.length], backgroundColor:'transparent', tension:.3, pointRadius:0, borderWidth:1.5,
    }));
    Charts.create(pfx+'topcpu', {type:'line', data:{datasets:ds}, options:{...CO, ...withY('% of machine')}});
  }

  if (hasTopRss) {
    addChart(pfx+'toprss', prefix+'Top 10 RSS (MB)');
    const ds = Object.entries(topRss).map(([name, arr], i) => ({
      label:name, data:pts(arr), borderColor:COLORS.palette[i%COLORS.palette.length], backgroundColor:'transparent', tension:.3, pointRadius:0, borderWidth:1.5,
    }));
    Charts.create(pfx+'toprss', {type:'line', data:{datasets:ds}, options:{...CO, ...withY('MB')}});
  }
}


function renderSerialCaseResources(cases) {
  const CASE_COLORS = ['rgba(77,166,255,.08)','rgba(52,211,153,.08)','rgba(245,166,35,.08)','rgba(239,83,80,.08)','rgba(167,139,250,.08)','rgba(244,114,182,.08)','rgba(56,189,248,.08)','rgba(251,191,36,.08)','rgba(132,204,22,.08)','rgba(168,85,247,.08)'];
  const shiftPts = (arr, off) => (arr || []).map(p => [p[0] + off, p[1]]);
  const shiftTopMap = (map, off) => {
    if (!map) return {};
    const out = {};
    for (const [k, arr] of Object.entries(map)) out[k] = shiftPts(arr, off);
    return out;
  };

  const merged = { system: {}, processes: {}, top_cpu: {}, top_rss: {} };
  const annotations = {};
  let offset = 0;

  cases.forEach((c, i) => {
    const res = c.resources;
    if (!res) return;
    const k6off = res.k6_offset || 0;
    const allPts = [...(res.system?.cpu || []), ...Object.values(res.processes || {}).flatMap(p => p.cpu || [])];
    const caseXMax = allPts.length ? Math.max(...allPts.map(p => p[0] || 0)) : 20;
    const shift = offset + k6off;

    for (const [key, arr] of Object.entries(res.system || {})) {
      if (!Array.isArray(arr)) continue;
      if (key === 'cpu_per_core') {
        if (!merged.system[key]) merged.system[key] = [];
        arr.forEach((coreSeries, ci) => {
          if (!merged.system[key][ci]) merged.system[key][ci] = [];
          merged.system[key][ci].push(...shiftPts(coreSeries, offset));
        });
      } else {
        if (!merged.system[key]) merged.system[key] = [];
        merged.system[key].push(...shiftPts(arr, offset));
      }
    }
    for (const [pkey, pval] of Object.entries(res.processes || {})) {
      if (!merged.processes[pkey]) merged.processes[pkey] = {};
      for (const [key, arr] of Object.entries(pval)) {
        if (!Array.isArray(arr)) continue;
        if (!merged.processes[pkey][key]) merged.processes[pkey][key] = [];
        merged.processes[pkey][key].push(...shiftPts(arr, offset));
      }
    }
    const topCpu = shiftTopMap(res.top_cpu, offset);
    const topRss = shiftTopMap(res.top_rss, offset);
    for (const [k, arr] of Object.entries(topCpu)) {
      if (!merged.top_cpu[k]) merged.top_cpu[k] = [];
      merged.top_cpu[k].push(...arr);
    }
    for (const [k, arr] of Object.entries(topRss)) {
      if (!merged.top_rss[k]) merged.top_rss[k] = [];
      merged.top_rss[k].push(...arr);
    }

    const tr = (c.perf || {}).timeRange || {};
    const xMin = (tr.startSec || 0) + shift;
    const xMax2 = (tr.endSec || caseXMax) + shift;
    const col = CASE_COLORS[i % CASE_COLORS.length];
    const label = c.label || c.name;
    const yAdj = (i % 2 === 0) ? 10 : 24;
    annotations[`case${i}`] = {type:'box', xMin, xMax:xMax2, backgroundColor:col, borderWidth:0, label:{display:true,content:label,color:'rgba(180,200,220,0.75)',font:{size:8,weight:'bold'},position:{x:'center',y:'start'},yAdjust:yAdj}};
    annotations[`caseL${i}`] = {type:'line', xMin, xMax:xMin, borderColor:'rgba(74,90,109,.4)', borderWidth:1, borderDash:[3,3]};

    offset += caseXMax;
  });

  const allMergedPts = [...(merged.system?.cpu || []), ...Object.values(merged.processes || {}).flatMap(p => p.cpu || [])];
  const totalXMax = allMergedPts.length ? Math.max(...allMergedPts.map(p => p[0] || 0)) + 5 : null;
  buildResourceCharts(merged, annotations, totalXMax, null);
}

function renderCaseResources(res, label, timeRange) {
  const k6off = res.k6_offset || 0;
  const annotations = {};
  if (timeRange) {
    annotations['active'] = {type:'box',
      xMin: (timeRange.startSec || 0) + k6off,
      xMax: (timeRange.endSec   || 0) + k6off,
      backgroundColor:'rgba(77,166,255,.10)', borderWidth:0,
      label:{display:true, content:'active', color:'rgba(180,200,220,0.75)', font:{size:8,weight:'bold'}, position:{x:'center',y:'start'}}};
  }
  const allPts = [...(res.system?.cpu||[]), ...Object.values(res.processes||{}).flatMap(p=>p.cpu||[])];
  const xMax = allPts.length ? Math.max(...allPts.map(p=>p[0]||0)) + 5 : null;
  buildResourceCharts(res, annotations, xMax, label);
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
  _chartIdSeq = 0;
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
