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
  const rpc = entry.resources_per_case || (entry.resources ? { core: entry.resources, catalog: entry.catalog } : null);
  const rpcKeys = rpc ? Object.keys(rpc).filter(k => k !== 'catalog' && k !== 'nproc' && k !== 'k6OffsetSeconds') : [];
  if (rpcKeys.length) {
    html += UI.Section('System Resources');
    html += `<div id="res-charts-container"></div>`;
    root.innerHTML = html;
    initResourceCharts(entry.resources_per_case || { core: entry.resources, catalog: entry.catalog }, entry);
  } else {
    root.innerHTML = html;
  }
}

function initResourceCharts(rpc, entry) {
  const SKIP_KEYS = new Set(['catalog', 'nproc', 'k6OffsetSeconds']);
  const rpcEntries = Object.entries(rpc).filter(([k]) => !SKIP_KEYS.has(k));
  if (!rpcEntries.length) return;

  const CASE_GAP = 5;
  const caseMaxTs = rpcEntries.map(([, res]) => {
    const pts = [...(res.system?.cpu||[]), ...Object.values(res.processes||{}).flatMap(p=>p.cpu||[])];
    return pts.length ? Math.max(...pts.map(p=>p[0]??0)) : 0;
  });
  const caseOffsets = [];
  let runningOffset = 0;
  for (let i = 0; i < rpcEntries.length; i++) {
    caseOffsets.push(runningOffset);
    runningOffset += caseMaxTs[i] + CASE_GAP;
  }
  const totalXMax = runningOffset - CASE_GAP + 5;

  const shiftPts = (pts, off) => pts.map(p => ({x:(p[0]??0)+off, y:p[1]}));
  const gap = (off, maxT) => [{x: off + maxT + 0.01, y: null}];

  const k6off = rpcEntries.find(([k]) => k === 'core')?.[1]?.k6OffsetSeconds || 0;
  const coreOff = caseOffsets[rpcEntries.findIndex(([k]) => k === 'core')] || 0;

  const mergePhases = [];
  rpcEntries.forEach(([caseName,], idx) => {
    const chartOff = caseOffsets[idx];
    if (caseName === 'core' && entry) {
      for (const p of (entry.phases || []).filter(p => !p.tunnel && !p.resourceCase && !p.name?.includes('(8k-q4)'))) {
        mergePhases.push({name: p.name, scenarios: p.scenarios||[], start: (p.start||0)+k6off+chartOff, end: (p.end||0)+k6off+chartOff});
      }
    } else if (caseName !== 'core') {
      const short = caseName.replace(/ \(.*?\)$/, '').replace(/_8000qps$/, ' 8k').replace(/_multihost/, ' mh').replace(/_/g,' ');
      mergePhases.push({name: short, scenarios: [caseName], start: chartOff, end: chartOff + caseMaxTs[idx]});
    }
  });

  const PHASE_COLORS = [
    'rgba(77,166,255,.08)','rgba(52,211,153,.08)','rgba(245,166,35,.08)',
    'rgba(239,83,80,.08)','rgba(167,139,250,.08)','rgba(244,114,182,.08)',
    'rgba(56,189,248,.08)','rgba(251,191,36,.08)',
    'rgba(132,204,22,.08)','rgba(168,85,247,.08)',
  ];
  const allAnnotations = {};
  mergePhases.forEach((p, i) => {
    const col = PHASE_COLORS[i % PHASE_COLORS.length];
    const nsc = (p.scenarios||[]).length;
    const label = nsc ? `${p.name} (${nsc})` : p.name;
    const yAdj = (i % 2 === 0) ? 10 : 24;
    allAnnotations[`phase${i}`] = {
      type:'box', xMin:p.start, xMax:p.end, backgroundColor:col, borderWidth:0,
      label:{display:true, content:label, color:'rgba(180,200,220,0.75)', font:{size:8,weight:'bold'}, position:{x:'center',y:'start'}, yAdjust:yAdj},
    };
    allAnnotations[`phaseLineS${i}`] = {type:'line', xMin:p.start, xMax:p.start, borderColor:'rgba(74,90,109,.3)', borderWidth:1, borderDash:[3,3]};
    allAnnotations[`phaseLineE${i}`] = {type:'line', xMin:p.end,   xMax:p.end,   borderColor:'rgba(74,90,109,.3)', borderWidth:1, borderDash:[3,3]};
  });

  const CASE_BG = ['rgba(77,166,255,.06)','rgba(52,211,153,.06)','rgba(245,166,35,.06)','rgba(167,139,250,.06)',
                   'rgba(244,114,182,.06)','rgba(56,189,248,.06)','rgba(251,191,36,.06)','rgba(129,140,248,.06)'];
  rpcEntries.forEach(([caseName,], idx) => {
    const xStart = caseOffsets[idx], xEnd = xStart + caseMaxTs[idx];
    if (caseName !== 'core') {
      allAnnotations[`caseBg${idx}`] = {
        type:'box', xMin:xStart, xMax:xEnd, backgroundColor:CASE_BG[idx%CASE_BG.length], borderWidth:0,
        label:{display:true, content:caseName.replace(/ \(.*?\)$/,'').replace(/_/g,' '), color:'rgba(180,200,220,0.55)', font:{size:11,weight:'bold'}, position:{x:'center',y:'start'}, yAdjust:4},
      };
    }
    if (idx > 0) {
      allAnnotations[`caseSep${idx}`] = {type:'line', xMin:xStart, xMax:xStart, borderColor:'rgba(255,255,255,0.2)', borderWidth:1.5, borderDash:[5,3]};
    }
  });

  if (entry) {
    (entry.phases||[])
      .filter(p => p.resourceCase || (p.name?.includes('(8k-q4)') && !p.tunnel && (p.start||0) > 0))
      .map(p => p.resourceCase ? p : {...p, resourceCase: 'core'})
      .forEach((p, fi) => {
      const chartOff = p.resourceCase === 'core' ? coreOff : 0;
      const shortName = p.name.replace(/ \([^)]*\)$/i, '').trim();
      const pIdx = mergePhases.length + fi;
      const col = PHASE_COLORS[pIdx % PHASE_COLORS.length];
      const yAdj = (pIdx % 2 === 0) ? 10 : 24;
      const nsc = (p.scenarios||[]).length;
      const label = nsc ? `${shortName} (${nsc})` : shortName;
      const xMin = (p.start||0)+chartOff, xMax = (p.end||0)+chartOff;
      allAnnotations[`frpOverlay${fi}`] = {
        type:'box', xMin, xMax, backgroundColor:col, borderWidth:0,
        label:{display:true, content:label, color:'rgba(180,200,220,0.75)', font:{size:8,weight:'bold'}, position:{x:'center',y:'start'}, yAdjust:yAdj},
      };
      allAnnotations[`frpOverlayLineS${fi}`] = {type:'line', xMin, xMax:xMin, borderColor:'rgba(74,90,109,.3)', borderWidth:1, borderDash:[3,3]};
      allAnnotations[`frpOverlayLineE${fi}`] = {type:'line', xMin:xMax, xMax, borderColor:'rgba(74,90,109,.3)', borderWidth:1, borderDash:[3,3]};
    });
  }

  const CO = {
    ...Charts.DEFAULT_OPTS, showLine: true,
    plugins: {...Charts.DEFAULT_OPTS.plugins, annotation: {annotations: allAnnotations}, tooltip: {mode:'index', intersect:false}},
    scales: {
      x: {type:'linear', title:{display:true, text:'seconds', color:'#6b7d93'}, ticks:{color:'#6b7d93',font:{size:9}}, grid:{color:'rgba(30,42,58,.5)'}, max: totalXMax},
      y: {...Charts.DEFAULT_OPTS.scales.y},
    },
  };
  const withYTitle = (text) => ({scales: {...CO.scales, y: {...CO.scales.y, title:{display:true, text, color:'#6b7d93'}}}});
  const linePts = (pts, color, label, extra={}) => ({label, data:pts, borderColor:color, backgroundColor:'transparent', tension:.3, pointRadius:0, borderWidth:1.5, ...extra});

  const container = document.getElementById('res-charts-container');
  const addChart = (id, title) => {
    container.insertAdjacentHTML('beforeend', `<div class="row full"><div class="panel tall"><h3>${title}</h3><div class="chart-area"><canvas id="${id}"></canvas></div><div id="${id}-leg" class="leg-wrap"></div></div></div>`);
  };

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
    {key:'cpu_usr',    label:'usr',          color:'#4da6ff'},
    {key:'cpu_sys',    label:'sys',          color:'#34d399'},
    {key:'cpu_soft',   label:'soft(net irq)',color:'#ef5350'},
    {key:'cpu_irq',    label:'irq',          color:'#f5a623'},
    {key:'cpu_iowait', label:'iowait',       color:'#a78bfa'},
    {key:'cpu_steal',  label:'steal',        color:'#4a5a6d'},
  ];
  const TOP_PALETTE = COLORS.palette;

  const cpuByKey={}, memByKey={}, vmsByKey={}, csVolByKey={}, csInvolByKey={};
  const sysCpuMerged=[], sysBdMerged={};
  const tcpEstab=[], tcpTimewait=[], tcpListen=[], tcpCloseWait=[], tcpFinWait1=[], tcpFinWait2=[], tcpSynSent=[], tcpSynRecv=[], tcpLastAck=[];
  const diskRdKbs=[], diskWrKbs=[], diskRdIops=[], diskWrIops=[];
  const intrPts=[], ctxSwPts=[];
  const udpRxErr=[], udpBufErr=[];
  const m8kRxKbs=[], m8kTxKbs=[], m8kRxPkts=[], m8kTxPkts=[], m8kDropIn=[], m8kDropOut=[];
  const m8kLoad1=[], m8kLoad5=[], m8kLoad15=[], m8kSwap=[], m8kMemMb=[];
  const merged8kTopCpu={}, merged8kTopRss={};
  const merged8kPerCore=[];
  let hasCpu=false, hasMem=false, hasVms=false, hasCs=false;
  let hasTcp=false, hasDisk=false, hasIntr=false, hasUdp=false;
  let hasNet=false, hasNetPkts=false, hasLoad=false;
  let hasTopCpu=false, hasTopRss=false, hasPerCore=false;

  rpcEntries.forEach(([, caseRes], idx) => {
    const off = caseOffsets[idx], maxT = caseMaxTs[idx];
    const cProc = caseRes.processes || {};
    const cSys  = caseRes.system || {};
    const g_ = (arr) => [...arr, ...gap(off, maxT)];

    for (const s of PSERIES) {
      const p = cProc[s.key];
      if (!cpuByKey[s.key])     cpuByKey[s.key]     = [];
      if (!memByKey[s.key])     memByKey[s.key]     = [];
      if (!vmsByKey[s.key])     vmsByKey[s.key]     = [];
      if (!csVolByKey[s.key])   csVolByKey[s.key]   = [];
      if (!csInvolByKey[s.key]) csInvolByKey[s.key] = [];
      if (p?.cpu?.length)    { cpuByKey[s.key].push(...g_(shiftPts(p.cpu, off)));         hasCpu=true; }
      if (p?.rss?.length)    { memByKey[s.key].push(...g_(shiftPts(p.rss, off)));         hasMem=true; }
      if (p?.vms?.length)    { vmsByKey[s.key].push(...g_(shiftPts(p.vms, off)));         hasVms=true; }
      if (p?.cswch?.length)  { csVolByKey[s.key].push(...g_(shiftPts(p.cswch, off)));    hasCs=true; }
      if (p?.nvcswch?.length){ csInvolByKey[s.key].push(...g_(shiftPts(p.nvcswch, off)));hasCs=true; }
    }

    if (cSys.cpu?.length) sysCpuMerged.push(...g_(shiftPts(cSys.cpu, off)));
    for (const bd of CPU_BREAKDOWN) {
      if (cSys[bd.key]?.length) {
        if (!sysBdMerged[bd.key]) sysBdMerged[bd.key]=[];
        sysBdMerged[bd.key].push(...g_(shiftPts(cSys[bd.key], off)));
      }
    }

    const estabPts = cSys.tcp_ESTABLISHED || cSys.tcp_estab    || [];
    const twaitPts = cSys.tcp_TIME_WAIT   || cSys.tcp_timewait || [];
    if (estabPts.length) { tcpEstab.push(...g_(shiftPts(estabPts, off)));    hasTcp=true; }
    if (twaitPts.length) { tcpTimewait.push(...g_(shiftPts(twaitPts, off))); hasTcp=true; }
    if (cSys.tcp_LISTEN?.length)     tcpListen.push(...g_(shiftPts(cSys.tcp_LISTEN, off)));
    if (cSys.tcp_CLOSE_WAIT?.length) tcpCloseWait.push(...g_(shiftPts(cSys.tcp_CLOSE_WAIT, off)));
    if (cSys.tcp_FIN_WAIT1?.length)  tcpFinWait1.push(...g_(shiftPts(cSys.tcp_FIN_WAIT1, off)));
    if (cSys.tcp_FIN_WAIT2?.length)  tcpFinWait2.push(...g_(shiftPts(cSys.tcp_FIN_WAIT2, off)));
    if (cSys.tcp_SYN_SENT?.length)   tcpSynSent.push(...g_(shiftPts(cSys.tcp_SYN_SENT, off)));
    if (cSys.tcp_SYN_RECV?.length)   tcpSynRecv.push(...g_(shiftPts(cSys.tcp_SYN_RECV, off)));
    if (cSys.tcp_LAST_ACK?.length)   tcpLastAck.push(...g_(shiftPts(cSys.tcp_LAST_ACK, off)));

    if (cSys.disk_read_kbs?.length)   { diskRdKbs.push(...g_(shiftPts(cSys.disk_read_kbs, off)));   hasDisk=true; }
    if (cSys.disk_write_kbs?.length)  { diskWrKbs.push(...g_(shiftPts(cSys.disk_write_kbs, off)));  hasDisk=true; }
    if (cSys.disk_read_iops?.length)  diskRdIops.push(...g_(shiftPts(cSys.disk_read_iops, off)));
    if (cSys.disk_write_iops?.length) diskWrIops.push(...g_(shiftPts(cSys.disk_write_iops, off)));

    if (cSys.interrupts?.length)   { intrPts.push(...g_(shiftPts(cSys.interrupts, off)));  hasIntr=true; }
    if (cSys.ctx_switches?.length) ctxSwPts.push(...g_(shiftPts(cSys.ctx_switches, off)));

    if (cSys.udp_rx_err?.length)  { udpRxErr.push(...g_(shiftPts(cSys.udp_rx_err, off)));  hasUdp=true; }
    if (cSys.udp_buf_err?.length) udpBufErr.push(...g_(shiftPts(cSys.udp_buf_err, off)));

    if (cSys.net_rx_kbs?.length) { m8kRxKbs.push(...g_(shiftPts(cSys.net_rx_kbs, off)));  hasNet=true; }
    if (cSys.net_tx_kbs?.length) { m8kTxKbs.push(...g_(shiftPts(cSys.net_tx_kbs, off)));  hasNet=true; }
    if (cSys.net_rx_pkts?.length){ m8kRxPkts.push(...g_(shiftPts(cSys.net_rx_pkts, off))); hasNetPkts=true; }
    if (cSys.net_tx_pkts?.length){ m8kTxPkts.push(...g_(shiftPts(cSys.net_tx_pkts, off))); hasNetPkts=true; }
    if (cSys.net_drop_in?.length)  m8kDropIn.push(...g_(shiftPts(cSys.net_drop_in, off)));
    if (cSys.net_drop_out?.length) m8kDropOut.push(...g_(shiftPts(cSys.net_drop_out, off)));

    if (cSys.loadavg_1?.length) { m8kLoad1.push(...g_(shiftPts(cSys.loadavg_1, off)));  hasLoad=true; }
    if (cSys.loadavg_5?.length)  m8kLoad5.push(...g_(shiftPts(cSys.loadavg_5, off)));
    if (cSys.loadavg_15?.length) m8kLoad15.push(...g_(shiftPts(cSys.loadavg_15, off)));
    if (cSys.swap_used_mb?.length) m8kSwap.push(...g_(shiftPts(cSys.swap_used_mb, off)));
    if (cSys.mem_used_mb?.length)  { m8kMemMb.push(...g_(shiftPts(cSys.mem_used_mb, off))); hasLoad=true; }

    for (const [name, pts] of Object.entries(caseRes.top_cpu || {})) {
      if (!merged8kTopCpu[name]) merged8kTopCpu[name]=[];
      merged8kTopCpu[name].push(...pts.map(p=>({x:(p[0]||0)+off, y:p[1]})));
      hasTopCpu=true;
    }
    for (const [name, pts] of Object.entries(caseRes.top_rss || {})) {
      if (!merged8kTopRss[name]) merged8kTopRss[name]=[];
      merged8kTopRss[name].push(...pts.map(p=>({x:(p[0]||0)+off, y:p[1]})));
      hasTopRss=true;
    }

    if (Array.isArray(cSys.cpu_per_core) && cSys.cpu_per_core.length) {
      const firstEntry = cSys.cpu_per_core[0];
      if (Array.isArray(firstEntry) && Array.isArray(firstEntry[1])) {
        // Format: [[t, [v0,v1,...]], ...] — expand per core
        cSys.cpu_per_core.forEach(([t, cores]) => {
          cores.forEach((v, ci) => {
            if (!merged8kPerCore[ci]) merged8kPerCore[ci]=[];
            merged8kPerCore[ci].push({x:(t||0)+off, y:v});
          });
        });
        if (cSys.cpu_per_core.length) {
          const lastT = (cSys.cpu_per_core[cSys.cpu_per_core.length-1][0]||0)+off;
          merged8kPerCore.forEach(s => s.push({x:lastT+0.01, y:null}));
        }
      } else {
        // Format: [[t,v],...] per core (after parse fix)
        cSys.cpu_per_core.forEach((pts, ci) => {
          if (!merged8kPerCore[ci]) merged8kPerCore[ci]=[];
          merged8kPerCore[ci].push(...shiftPts(pts, off), ...gap(off, maxT));
        });
      }
      hasPerCore = merged8kPerCore.length > 0;
    }
  });

  // CPU
  {
    addChart('res_cpu', 'CPU Usage (% of machine)');
    const ds=[];
    for (const s of PSERIES) {
      if (!cpuByKey[s.key]?.length) continue;
      ds.push({label:s.label, data:cpuByKey[s.key], borderColor:s.color, backgroundColor:s.color+'28', fill:true, tension:.3, pointRadius:0, borderWidth:1.5});
    }
    for (const bd of CPU_BREAKDOWN) {
      if (sysBdMerged[bd.key]?.length)
        ds.push({label:bd.label, data:sysBdMerged[bd.key], borderColor:bd.color, backgroundColor:'transparent', fill:false, tension:.3, pointRadius:0, borderWidth:1, borderDash:[3,2], hidden:true});
    }
    if (sysCpuMerged.length)
      ds.push({label:'System total', data:sysCpuMerged, borderColor:'#ffffff', backgroundColor:'transparent', fill:false, tension:.3, pointRadius:0, borderWidth:1, borderDash:[4,3]});
    Charts.create('res_cpu', {type:'line', data:{datasets:ds}, options:{...CO, scales:{...CO.scales, y:{...CO.scales.y, stacked:false, min:0, max:100, title:{display:true,text:'% of machine',color:'#6b7d93'}}}}});
  }

  // Per-core CPU
  if (hasPerCore) {
    addChart('res_percore', 'Per-core CPU (%)');
    const CORE_COLORS=['#4da6ff','#34d399','#f5a623','#ef5350','#a78bfa','#f472b6','#38bdf8','#fbbf24','#818cf8','#fb923c','#22d3ee','#a3e635','#e879f9','#94a3b8','#fb7185','#4ade80'];
    const ds = merged8kPerCore.map((pts,i)=>({label:`Core ${i}`, data:pts, borderColor:CORE_COLORS[i%CORE_COLORS.length], backgroundColor:'transparent', tension:.3, pointRadius:0, borderWidth:1}));
    Charts.create('res_percore', {type:'line', data:{datasets:ds}, options:{...CO, scales:{...CO.scales, y:{...CO.scales.y, min:0, max:100, title:{display:true,text:'%',color:'#6b7d93'}}}}});
  }

  // Memory RSS
  if (hasMem) {
    addChart('res_mem', 'Memory RSS (MB)');
    const ds=[];
    for (const s of PSERIES) { if (memByKey[s.key]?.length) ds.push(linePts(memByKey[s.key], s.color, s.label)); }
    Charts.create('res_mem', {type:'line', data:{datasets:ds}, options:{...CO, ...withYTitle('MB')}});
  }

  // Load Avg + Memory + Swap (dual Y-axis)
  if (hasLoad) {
    addChart('res_load', 'Load Average + Memory + Swap');
    const ds=[];
    if (m8kLoad1.length)  ds.push({...linePts(m8kLoad1,  '#4da6ff', 'load 1m'),               yAxisID:'yLoad'});
    if (m8kLoad5.length)  ds.push({...linePts(m8kLoad5,  '#f5a623', 'load 5m'),               yAxisID:'yLoad'});
    if (m8kLoad15.length) ds.push({...linePts(m8kLoad15, '#a78bfa', 'load 15m', {borderDash:[4,2]}), yAxisID:'yLoad'});
    if (m8kMemMb.length)  ds.push({...linePts(m8kMemMb,  '#34d399', 'mem MB',   {borderDash:[3,1]}), yAxisID:'yMem'});
    if (m8kSwap.length)   ds.push({...linePts(m8kSwap,   '#ef5350', 'swap MB',  {borderDash:[2,2]}), yAxisID:'yMem'});
    const loadOpts = {
      ...CO,
      scales: {
        ...CO.scales,
        yLoad: {position:'left',  title:{display:true, text:'load', color:'#6b7d93'}, ticks:{color:'#6b7d93',font:{size:9}}, grid:{color:'rgba(30,42,58,.5)'}},
        yMem:  {position:'right', title:{display:true, text:'MB',   color:'#6b7d93'}, ticks:{color:'#6b7d93',font:{size:9}}, grid:{drawOnChartArea:false}},
      },
    };
    Charts.create('res_load', {type:'line', data:{datasets:ds}, options:loadOpts});
  }

  // Network throughput
  if (hasNet) {
    addChart('res_net', 'Network Throughput (KB/s)');
    const ds=[];
    if (m8kRxKbs.length) ds.push(linePts(m8kRxKbs, '#34d399', 'RX KB/s'));
    if (m8kTxKbs.length) ds.push(linePts(m8kTxKbs, '#4da6ff', 'TX KB/s'));
    Charts.create('res_net', {type:'line', data:{datasets:ds}, options:{...CO, ...withYTitle('KB/s')}});
  }

  // Network packets + drops
  if (hasNetPkts) {
    addChart('res_netpkts', 'Network Packets/s + Drops');
    const ds=[];
    if (m8kRxPkts.length)  ds.push(linePts(m8kRxPkts,  '#34d399', 'RX pkts/s'));
    if (m8kTxPkts.length)  ds.push(linePts(m8kTxPkts,  '#4da6ff', 'TX pkts/s'));
    if (m8kDropIn.length)  ds.push(linePts(m8kDropIn,  '#ef5350', 'drop_in/s',  {borderDash:[3,2]}));
    if (m8kDropOut.length) ds.push(linePts(m8kDropOut, '#f5a623', 'drop_out/s', {borderDash:[3,2]}));
    Charts.create('res_netpkts', {type:'line', data:{datasets:ds}, options:{...CO, ...withYTitle('/s')}});
  }

  // TCP connections
  if (hasTcp) {
    addChart('res_tcp', 'TCP Connections');
    const ds=[];
    if (tcpEstab.length)     ds.push(linePts(tcpEstab,     '#4da6ff', 'ESTABLISHED'));
    if (tcpTimewait.length)  ds.push(linePts(tcpTimewait,  '#f5a623', 'TIME_WAIT'));
    if (tcpListen.length)    ds.push(linePts(tcpListen,    '#34d399', 'LISTEN',    {borderDash:[4,2], borderWidth:1}));
    if (tcpCloseWait.length) ds.push(linePts(tcpCloseWait, '#ef5350', 'CLOSE_WAIT',{borderDash:[3,2], borderWidth:1}));
    if (tcpFinWait1.length)  ds.push(linePts(tcpFinWait1,  '#a78bfa', 'FIN_WAIT1', {borderDash:[3,2], borderWidth:1}));
    if (tcpFinWait2.length)  ds.push(linePts(tcpFinWait2,  '#f472b6', 'FIN_WAIT2', {borderDash:[3,2], borderWidth:1}));
    if (tcpSynSent.length)   ds.push(linePts(tcpSynSent,   '#38bdf8', 'SYN_SENT',  {borderDash:[2,2], borderWidth:1}));
    if (tcpSynRecv.length)   ds.push(linePts(tcpSynRecv,   '#fbbf24', 'SYN_RECV',  {borderDash:[2,2], borderWidth:1}));
    if (tcpLastAck.length)   ds.push(linePts(tcpLastAck,   '#4a5a6d', 'LAST_ACK',  {borderDash:[2,2], borderWidth:1}));
    Charts.create('res_tcp', {type:'line', data:{datasets:ds}, options:{...CO, ...withYTitle('connections')}});
  }

  // VMS (exclude 'other' — its VMS reflects max across all system processes, not meaningful)
  if (hasVms) {
    addChart('res_vms', 'Virtual Memory Size (MB)');
    const ds=[];
    for (const s of PSERIES) { if (s.key !== 'other' && vmsByKey[s.key]?.length) ds.push(linePts(vmsByKey[s.key], s.color, s.label)); }
    Charts.create('res_vms', {type:'line', data:{datasets:ds}, options:{...CO, ...withYTitle('MB')}});
  }

  // Disk I/O
  if (hasDisk) {
    addChart('res_disk', 'Disk I/O (KB/s)');
    const ds=[];
    if (diskRdKbs.length)  ds.push(linePts(diskRdKbs,  '#34d399', 'read KB/s'));
    if (diskWrKbs.length)  ds.push(linePts(diskWrKbs,  '#4da6ff', 'write KB/s'));
    if (diskRdIops.length) ds.push(linePts(diskRdIops, '#a78bfa', 'read IOPS',  {borderDash:[3,2], borderWidth:1}));
    if (diskWrIops.length) ds.push(linePts(diskWrIops, '#f5a623', 'write IOPS', {borderDash:[3,2], borderWidth:1}));
    Charts.create('res_disk', {type:'line', data:{datasets:ds}, options:{...CO, ...withYTitle('KB/s')}});
  }

  // Interrupts + ctx switches
  if (hasIntr) {
    addChart('res_intr', 'Interrupts / s');
    const ds=[];
    if (intrPts.length)  ds.push(linePts(intrPts,  '#f472b6', 'interrupts/s'));
    if (ctxSwPts.length) ds.push(linePts(ctxSwPts, '#38bdf8', 'ctx_switches/s', {borderDash:[3,2], borderWidth:1}));
    Charts.create('res_intr', {type:'line', data:{datasets:ds}, options:{...CO, ...withYTitle('/s')}});
  }

  // Context switches per process
  if (hasCs) {
    addChart('res_ctxsw', 'Context Switches / s');
    const ds=[];
    for (const s of PSERIES) {
      if (csVolByKey[s.key]?.length)   ds.push(linePts(csVolByKey[s.key],   s.color, `${s.label} vol`));
      if (csInvolByKey[s.key]?.length) ds.push(linePts(csInvolByKey[s.key], s.color, `${s.label} invol`, {borderDash:[3,2], borderWidth:1}));
    }
    Charts.create('res_ctxsw', {type:'line', data:{datasets:ds}, options:{...CO, ...withYTitle('/s')}});
  }

  // UDP errors
  if (hasUdp) {
    addChart('res_udp', 'UDP Errors / s');
    const ds=[];
    if (udpRxErr.length)  ds.push(linePts(udpRxErr,  '#ef5350', 'InErrors/s'));
    if (udpBufErr.length) ds.push(linePts(udpBufErr, '#f5a623', 'RcvbufErr/s', {borderDash:[3,2]}));
    Charts.create('res_udp', {type:'line', data:{datasets:ds}, options:{...CO, ...withYTitle('/s')}});
  }

  // Top 10 CPU
  if (hasTopCpu) {
    addChart('res_topcpu', 'Top 10 CPU (% of machine)');
    const ds = Object.entries(merged8kTopCpu).map(([name, pts], i) => ({
      label:name, data:pts, borderColor:TOP_PALETTE[i%TOP_PALETTE.length], backgroundColor:'transparent', tension:.3, pointRadius:0, borderWidth:1.5,
    }));
    Charts.create('res_topcpu', {type:'line', data:{datasets:ds}, options:{...CO, ...withYTitle('% of machine')}});
  }

  // Top 10 RSS
  if (hasTopRss) {
    addChart('res_toprss', 'Top 10 RSS (MB)');
    const ds = Object.entries(merged8kTopRss).map(([name, pts], i) => ({
      label:name, data:pts, borderColor:TOP_PALETTE[i%TOP_PALETTE.length], backgroundColor:'transparent', tension:.3, pointRadius:0, borderWidth:1.5,
    }));
    Charts.create('res_toprss', {type:'line', data:{datasets:ds}, options:{...CO, ...withYTitle('MB')}});
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
