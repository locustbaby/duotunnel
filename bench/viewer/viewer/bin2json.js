#!/usr/bin/env node
// bin2json.js — decode a dial9 .bin.gz trace into JSON shards for gh-pages hosting
//
// Usage: node bin2json.js <trace.bin.gz> <output-base>
//   Produces:
//     <output-base>-meta.json        (~1-2 MB, not gzipped)
//     <output-base>-events.json.gz   (~5-15 MB gzipped)
//     <output-base>-cpu.json.gz      (~1 MB gzipped)
//
// Events are serialized in streaming chunks to avoid V8 max-string-length
// on large traces (2M+ events).

'use strict';

const fs   = require('fs');
const path = require('path');
const zlib = require('zlib');

const { TraceDecoder } = require(path.join(__dirname, 'decode.js'));
global.TraceDecoder = TraceDecoder;
const { parseTrace } = require(path.join(__dirname, 'trace_parser.js'));

const [inputPath, outputBase] = process.argv.slice(2);
if (!inputPath || !outputBase) {
  console.error('Usage: node bin2json.js <trace.bin.gz> <output-base>');
  process.exit(1);
}

function mapToEntries(m) {
  return m instanceof Map ? [...m.entries()] : [];
}

// Serialize array to gzip file in chunks to avoid V8 max-string-length limit.
function writeGzippedJsonArray(arr, outPath) {
  const CHUNK = 50000;
  const gz = zlib.createGzip({ level: 6 });
  const out = fs.createWriteStream(outPath);
  gz.pipe(out);
  gz.write('[');
  for (let i = 0; i < arr.length; i += CHUNK) {
    const slice = arr.slice(i, i + CHUNK);
    const chunk = JSON.stringify(slice).slice(1, -1); // strip outer [ ]
    if (i > 0) gz.write(',');
    gz.write(chunk);
  }
  gz.write(']');
  gz.end();
  return new Promise((resolve, reject) => {
    out.on('finish', resolve);
    out.on('error', reject);
    gz.on('error', reject);
  });
}

async function main() {
  // Read + decompress
  const raw = fs.readFileSync(inputPath);
  const buf = raw[0] === 0x1f && raw[1] === 0x8b ? zlib.gunzipSync(raw) : raw;
  console.error(`Decoded ${(buf.length / 1024 / 1024).toFixed(1)} MB`);

  // Parse
  const ab = buf.buffer.slice(buf.byteOffset, buf.byteOffset + buf.byteLength);
  const trace = parseTrace(ab);
  console.error(`Parsed: ${trace.events.length} events, ${trace.cpuSamples.length} cpu samples`);

  // ── Shard 1: meta.json ──
  const meta = {
    magic:              'D9TF',
    version:            trace.version,
    truncated:          trace.truncated,
    hasCpuTime:         trace.hasCpuTime,
    hasSchedWait:       trace.hasSchedWait,
    hasTaskTracking:    trace.hasTaskTracking,
    eventCount:         trace.events.length,
    cpuSampleCount:     trace.cpuSamples.length,
    callframeSymbols:   mapToEntries(trace.callframeSymbols),
    spawnLocations:     mapToEntries(trace.spawnLocations),
    taskSpawnLocs:      mapToEntries(trace.taskSpawnLocs),
    taskSpawnTimes:     mapToEntries(trace.taskSpawnTimes),
    taskTerminateTimes: mapToEntries(trace.taskTerminateTimes),
    runtimeWorkers:     mapToEntries(trace.runtimeWorkers),
  };
  const metaPath = `${outputBase}-meta.json`;
  fs.writeFileSync(metaPath, JSON.stringify(meta));
  console.error(`Written ${metaPath} (${(fs.statSync(metaPath).size / 1024).toFixed(0)} KB)`);

  // ── Shard 2: events.json.gz ──
  const eventsPath = `${outputBase}-events.json.gz`;
  await writeGzippedJsonArray(trace.events, eventsPath);
  console.error(`Written ${eventsPath} (${(fs.statSync(eventsPath).size / 1024 / 1024).toFixed(1)} MB)`);

  // ── Shard 3: cpu.json.gz ──
  const cpuPath = `${outputBase}-cpu.json.gz`;
  await writeGzippedJsonArray(trace.cpuSamples, cpuPath);
  console.error(`Written ${cpuPath} (${(fs.statSync(cpuPath).size / 1024 / 1024).toFixed(1)} MB)`);

  console.error('Done.');
}

main().catch((err) => { console.error(err); process.exit(1); });
