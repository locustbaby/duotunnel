#!/usr/bin/env node
// bin2json.js — decode a dial9 .bin.gz trace into three JSON shards for gh-pages hosting
//
// Usage: node bin2json.js <trace.bin.gz> <output-base>
//   Produces:
//     <output-base>-meta.json        (~1-2 MB, not gzipped)
//     <output-base>-events.json.gz   (~5-10 MB gzipped)
//     <output-base>-cpu.json.gz      (~1 MB gzipped)

'use strict';

const fs   = require('fs');
const path = require('path');
const zlib = require('zlib');

// Load decoder from same directory
const { TraceDecoder } = require(path.join(__dirname, 'decode.js'));
global.TraceDecoder = TraceDecoder;
const { parseTrace } = require(path.join(__dirname, 'trace_parser.js'));

function usage() {
  console.error('Usage: node bin2json.js <trace.bin.gz> <output-base>');
  process.exit(1);
}

const [inputPath, outputBase] = process.argv.slice(2);
if (!inputPath || !outputBase) usage();

// Read + decompress input
const raw = fs.readFileSync(inputPath);
const buf = raw[0] === 0x1f && raw[1] === 0x8b ? zlib.gunzipSync(raw) : raw;

console.error(`Decoded ${(buf.length / 1024 / 1024).toFixed(1)} MB from ${inputPath}`);

// Parse
const ab = buf.buffer.slice(buf.byteOffset, buf.byteOffset + buf.byteLength);
const trace = parseTrace(ab);

console.error(`Parsed: ${trace.events.length} events, ${trace.cpuSamples.length} cpu samples`);

// Helper: Map → serializable array of [k, v] pairs
function mapToEntries(m) {
  if (!(m instanceof Map)) return [];
  return [...m.entries()];
}

// ── Shard 1: meta.json (no gzip, small, fetched first) ──
const meta = {
  magic:            'D9TF',
  version:          trace.version,
  truncated:        trace.truncated,
  hasCpuTime:       trace.hasCpuTime,
  hasSchedWait:     trace.hasSchedWait,
  hasTaskTracking:  trace.hasTaskTracking,
  eventCount:       trace.events.length,
  cpuSampleCount:   trace.cpuSamples.length,
  // Maps serialized as arrays of [key, value] pairs
  callframeSymbols: mapToEntries(trace.callframeSymbols),
  spawnLocations:   mapToEntries(trace.spawnLocations),
  taskSpawnLocs:    mapToEntries(trace.taskSpawnLocs),
  taskSpawnTimes:   mapToEntries(trace.taskSpawnTimes),
  taskTerminateTimes: mapToEntries(trace.taskTerminateTimes),
  runtimeWorkers:   mapToEntries(trace.runtimeWorkers),
};

const metaPath = `${outputBase}-meta.json`;
fs.writeFileSync(metaPath, JSON.stringify(meta));
console.error(`Written ${metaPath} (${(fs.statSync(metaPath).size / 1024).toFixed(0)} KB)`);

// ── Shard 2: events.json.gz ──
const eventsPath = `${outputBase}-events.json.gz`;
fs.writeFileSync(eventsPath, zlib.gzipSync(JSON.stringify(trace.events), { level: 6 }));
console.error(`Written ${eventsPath} (${(fs.statSync(eventsPath).size / 1024 / 1024).toFixed(1)} MB)`);

// ── Shard 3: cpu.json.gz ──
const cpuPath = `${outputBase}-cpu.json.gz`;
fs.writeFileSync(cpuPath, zlib.gzipSync(JSON.stringify(trace.cpuSamples), { level: 6 }));
console.error(`Written ${cpuPath} (${(fs.statSync(cpuPath).size / 1024 / 1024).toFixed(1)} MB)`);

console.error('Done.');
