use dial9_trace_format::decoder::Decoder;
use dial9_trace_format::types::FieldValueRef;
use flate2::read::GzDecoder;
use std::collections::HashMap;
use std::io::Read;

fn load(path: &str) -> Vec<u8> {
    let raw = if path.starts_with("http://") || path.starts_with("https://") {
        eprintln!("Downloading {path} ...");
        let out = std::process::Command::new("curl")
            .args(["-sL", path])
            .output()
            .expect("curl not found");
        if !out.status.success() {
            panic!("curl failed: {}", String::from_utf8_lossy(&out.stderr));
        }
        out.stdout
    } else {
        std::fs::read(path).unwrap_or_else(|e| panic!("read {path}: {e}"))
    };

    if path.ends_with(".gz") || raw.starts_with(b"\x1f\x8b") {
        eprintln!("Decompressing ...");
        let mut out = Vec::new();
        GzDecoder::new(raw.as_slice())
            .read_to_end(&mut out)
            .expect("gunzip failed");
        out
    } else {
        raw
    }
}

fn short(loc: &str) -> &str {
    loc.split('/').last().unwrap_or(loc)
}

fn main() {
    let path = std::env::args().nth(1).unwrap_or_else(|| {
        eprintln!("Usage: trace-parser <path-or-url.bin[.gz]>");
        std::process::exit(1);
    });

    let data = load(&path);
    eprintln!("Loaded {} bytes, parsing ...\n", data.len());

    // ── Pass 1: event counts ─────────────────────────────────────────────────
    let mut event_counts: HashMap<String, u64> = HashMap::new();
    {
        let mut dec = Decoder::new(&data).expect("invalid TRC header");
        let _ = dec.for_each_event(|ev| {
            *event_counts.entry(ev.name.to_string()).or_insert(0) += 1;
        });
    }

    println!("=== Event Counts ===");
    let mut counts: Vec<_> = event_counts.iter().collect();
    counts.sort_by(|a, b| b.1.cmp(a.1));
    for (name, count) in &counts {
        println!("  {:<40} {:>10}", name, count);
    }

    // ── Pass 2: poll / park analysis ─────────────────────────────────────────
    // PollStartEvent:    [0]worker_id [1]local_queue [2]task_id [3]spawn_loc(pooled)
    // PollEndEvent:      [0]worker_id   (no task_id — one active poll per worker)
    // WorkerParkEvent:   [0]worker_id [1]local_queue [2]cpu_time_ns
    // WorkerUnparkEvent: [0]worker_id [1]local_queue [2]sched_wait_ns

    let mut current_poll: HashMap<u64, (u64, String)> = HashMap::new(); // worker -> (ts, loc)
    let mut long_polls: Vec<(u64, u64, String)> = Vec::new();           // (dur_ns, worker, loc)
    let mut worker_poll_ns: HashMap<u64, u64> = HashMap::new();
    let mut worker_park_ns: HashMap<u64, u64> = HashMap::new();
    let mut park_starts: HashMap<u64, u64> = HashMap::new();
    let mut loc_poll_hist: HashMap<String, Vec<u64>> = HashMap::new();
    let mut n_workers = 0u64;

    let mut dec2 = Decoder::new(&data).expect("invalid TRC header");
    let _ = dec2.for_each_event(|ev| {
        let ts = ev.timestamp_ns.unwrap_or(0);
        let pool = ev.string_pool;

        match ev.name {
            "PollStartEvent" => {
                let mut worker_id = 0u64;
                let mut loc = String::new();
                for (i, f) in ev.fields.iter().enumerate() {
                    match (i, f) {
                        (0, FieldValueRef::Varint(v)) => worker_id = *v,
                        (_, FieldValueRef::PooledString(id)) => {
                            if let Some(s) = pool.get(*id) {
                                loc = s.to_string();
                            }
                        }
                        (_, FieldValueRef::String(s)) => loc = s.to_string(),
                        _ => {}
                    }
                }
                n_workers = n_workers.max(worker_id + 1);
                current_poll.insert(worker_id, (ts, loc));
            }
            "PollEndEvent" => {
                let mut worker_id = 0u64;
                if let Some(FieldValueRef::Varint(v)) = ev.fields.first() {
                    worker_id = *v;
                }
                if let Some((start, loc)) = current_poll.remove(&worker_id) {
                    let dur = ts.saturating_sub(start);
                    *worker_poll_ns.entry(worker_id).or_insert(0) += dur;
                    loc_poll_hist.entry(loc.clone()).or_default().push(dur);
                    if dur > 1_000_000 {
                        long_polls.push((dur, worker_id, loc));
                    }
                }
            }
            "WorkerParkEvent" => {
                let mut w = 0u64;
                if let Some(FieldValueRef::Varint(v)) = ev.fields.first() {
                    w = *v;
                }
                park_starts.insert(w, ts);
            }
            "WorkerUnparkEvent" => {
                let mut w = 0u64;
                if let Some(FieldValueRef::Varint(v)) = ev.fields.first() {
                    w = *v;
                }
                if let Some(s) = park_starts.remove(&w) {
                    *worker_park_ns.entry(w).or_insert(0) += ts.saturating_sub(s);
                }
            }
            _ => {}
        }
    });

    // ── Worker utilization ───────────────────────────────────────────────────
    // Estimate trace wall time: max of (poll + park) per worker
    let trace_ns = (0..n_workers)
        .map(|w| {
            worker_poll_ns.get(&w).copied().unwrap_or(0)
                + worker_park_ns.get(&w).copied().unwrap_or(0)
        })
        .max()
        .unwrap_or(1)
        .max(1);

    let total_poll_ns: u64 = worker_poll_ns.values().sum();
    let total_park_ns: u64 = worker_park_ns.values().sum();

    println!("\n=== Worker Utilization (wall ~{:.0}ms) ===", trace_ns as f64 / 1e6);
    for w in 0..n_workers {
        let poll = worker_poll_ns.get(&w).copied().unwrap_or(0);
        let park = worker_park_ns.get(&w).copied().unwrap_or(0);
        println!(
            "  Worker {}: poll={:6.0}ms ({:5.1}%)  park={:6.0}ms ({:5.1}%)",
            w,
            poll as f64 / 1e6, poll as f64 / trace_ns as f64 * 100.0,
            park as f64 / 1e6, park as f64 / trace_ns as f64 * 100.0,
        );
    }
    println!(
        "  Total:    poll={:6.0}ms              park={:6.0}ms",
        total_poll_ns as f64 / 1e6,
        total_park_ns as f64 / 1e6,
    );

    // ── Poll latency by spawn location ───────────────────────────────────────
    println!("\n=== Poll Latency by Spawn Location ===");
    println!(
        "  {:<48} {:>8} {:>8} {:>8} {:>10} {:>10}",
        "Location", "polls", "p50µs", "p99µs", "max_ms", "total_ms"
    );
    let mut loc_stats: Vec<_> = loc_poll_hist
        .iter()
        .map(|(loc, durs)| {
            let n = durs.len();
            let sum: u64 = durs.iter().sum();
            let mut s = durs.clone();
            s.sort_unstable();
            let p50 = s[(n as f64 * 0.50) as usize];
            let p99 = s[((n as f64 * 0.99) as usize).min(n - 1)];
            let max = *s.last().unwrap_or(&0);
            (loc, n, sum, p50, p99, max)
        })
        .collect();
    loc_stats.sort_by(|a, b| b.1.cmp(&a.1));
    for (loc, cnt, sum, p50, p99, max) in &loc_stats {
        println!(
            "  {:<48} {:>8} {:>8.0} {:>8.0} {:>10.2} {:>10.1}",
            short(loc), cnt,
            *p50 as f64 / 1e3,
            *p99 as f64 / 1e3,
            *max as f64 / 1e6,
            *sum as f64 / 1e6,
        );
    }

    // ── Long polls ───────────────────────────────────────────────────────────
    long_polls.sort_by(|a, b| b.0.cmp(&a.0));
    println!("\n=== Long Polls >1ms: {} total ===", long_polls.len());

    let mut by_loc: HashMap<String, (u64, u64)> = HashMap::new();
    for (dur, _, loc) in &long_polls {
        let e = by_loc.entry(loc.clone()).or_insert((0, 0));
        e.0 += 1;
        e.1 += dur;
    }

    println!("Top 20 longest:");
    for (dur, w, loc) in long_polls.iter().take(20) {
        println!("  {:>8.2}ms  w{}  {}", *dur as f64 / 1e6, w, short(loc));
    }

    println!("\nGrouped by spawn location:");
    let mut bv: Vec<_> = by_loc.iter().collect();
    bv.sort_by(|a, b| b.1.0.cmp(&a.1.0));
    for (loc, (cnt, total)) in &bv {
        println!(
            "  {:>6}x  avg={:>7.2}ms  total={:>7.0}ms  {}",
            cnt,
            (*total as f64 / *cnt as f64) / 1e6,
            *total as f64 / 1e6,
            short(loc),
        );
    }
}
