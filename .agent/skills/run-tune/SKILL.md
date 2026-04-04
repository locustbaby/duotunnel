---
name: run_tune
description: Verifies issues in module-tune docs, considers better solutions, and documents high-level optimization strategies in docs/OPTIMIZATIONS.md.
---

# Code Optimization Verification and Documentation (run_tune)

When assigned to verify and plan code optimizations (e.g., running the `run_tune` workflow), follow these step-by-step instructions:

## Step 1: Read the Module's Tuning Report
Use the `view_file` tool to read the specific `*-tune.md` report (such as `docs/tmp-tune/module-tune.md` or `docs/tmp-tune/server-tune.md`) that documents the identified bottlenecks. Get an understanding of what issues were flagged mapping to memory allocations, lock contention, or latency spikes.

## Step 2: Verify the Code Reality
Do not blindly trust the markdown report. Use `view_file` or `grep_search` to inspect the targeted source `.rs` files.
- Critically assess if the described issues genuinely exist.
- Verify if they actually reside on a hot path (e.g., within a high-frequency polling loop or per-connection setup).

## Step 3: Propose Advanced Solutions
Evaluate the fix suggested in the report. Is there a better architectural or idiomatic Rust alternative?
- **Locks & Contention**: E.g., replacing `DashMap` iteration with `arc_swap` (RCU), or using atomic integer counters.
- **Memory Allocations**: E.g., switching `Vec` collecting to zero-cost iterators, utilizing `ThreadLocal` storage, or adopting object pooling.
- **CPU Bottlenecks**: E.g., shifting dynamic processing (like TLS generation) into global singletons with cache (`moka` or `OnceCell`).

Determine the optimal, least invasive, but most performant high-level strategy.

## Step 4: Document Strategy in `docs/OPTIMIZATIONS.md`
Edit or append to `/Users/sexy/Documents/GitHub/tunnel/docs/OPTIMIZATIONS.md` with your finalized optimization strategies.
- **Strict Rule**: DO NOT write exact code details, line numbers, or variable names in this file. 
- **Goal**: Write purely structural and architectural optimization *concepts*. Focus on the "Before" vs "After" architecture design, the mental model, and *why* it solves the problem. 
- This document's role is to track the high-level thought process of how the system scaled.

Stop and wait for the user to confirm the optimization approach before implementing the code.
