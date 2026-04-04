---
name: Systematic Codebase Performance Analysis
description: A straightforward, step-by-step workflow for breaking down a repository into modules, performing file-by-file deep dives, and identifying specific performance or latency optimization opportunities.
---

# Systematic Codebase Performance Analysis

When asked to review a project for optimizations, do not jump straight to rewriting code. Follow this systematic, methodical workflow to deeply understand the architecture and discover hidden bottlenecks.

## Step 1: Project Module Discovery
Start by getting a bird's-eye view of the entire repository.
- Search for workspace definition files (e.g., `Cargo.toml`, `package.json`, `pom.xml`, etc.) or root project directories.
- Break the repository down into distinct logical **Modules** (e.g., core engine, client, server, shared library).
- State the function and responsibility of each module clearly before moving forward.

## Step 2: File-by-File Module Breakdown
Take the review one module at a time. For the selected module:
- Iterate through its critical source files or directories.
- Read the entire contents of the files using file-reading tools to understand the exact implementation details.

## Step 3: Deep Review for Performance & Latency
While reading each file, actively search for performance and latency anti-patterns.
- **Latency Spikes**: Look for redundant parsing, unnecessary decoding/encoding loops, or multiple passes over the same data.
- **Memory Allocation**: Hunt for inside-the-loop dynamic allocations, zero-filled array initializations, and lack of buffer pooling.
- **Concurrency Bottlenecks**: Identify heavy locks (`Mutex`, `RwLock`) in hot paths or lack of multithreading optimizations (e.g., missing kernel-level load balancing).

## Step 4: Document and Propose
- Once a module mapping and file-by-file review are complete, write down a structured analysis (like a markdown doc) listing what each file does and what specific optimizations are required.
- **Output Report Format**: The output report must strictly follow the format and structural layout demonstrated in `docs/tmp-tune/module-tune.md`. Use it as a template for documenting module sections, file analysis, issue identification, and optimization proposals.
- Wait for user feedback and confirmation before writing any code to implement the optimizations.
- Repeat the process for the next module.
