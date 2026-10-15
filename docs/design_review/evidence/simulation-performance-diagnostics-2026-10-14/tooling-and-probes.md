---
title: Profiling tools, collection alternatives and bounded probes
doc_role: evidence
doc_owner: docs/design_review/reviews/design_review_simulation-performance-diagnostics_2026-10-14.md
doc_retention: while-dependent
---

# Profiling tools, collection alternatives and bounded probes

Supporting evidence for the [principal review](../../reviews/design_review_simulation-performance-diagnostics_2026-10-14.md). This is a scoped capability comparison and record of two small workstation probes, not simulation qualification or a second design verdict. Inspected baseline: `a08f5fa091db1840fc95b46c4d4f849ce4c5de62` and the existing working-tree changes on 2026-10-14 EDT. Recommendations are **Proposed**; library contracts below are **Interface-checked** unless a probe is named.

## Different instruments answer different questions

| Mechanism | Useful observation | What it does not establish | Recommended role |
| --- | --- | --- | --- |
| Existing `tracing` spans/events and owned counters | Operation boundaries, explicit causal links, work requests, cache decisions, elapsed intervals and owner-published progress | A span's lifetime is not CPU time; missing spans do not mean no work; parentage alone does not explain numerical convergence | Compose existing mechanisms into ordinary bounded inspection. Keep domain interpretation in the simulator, not the subscriber. |
| `tracing-subscriber` layers with local structured output | Filtering, composition, lifecycle events and a machine-readable diagnostic stream | A globally installed subscriber cannot be assumed in an embedded Rust/Python application; dropping a writer can lose an unfinished prefix | Composition roots select optional sinks. Core operation state must remain inspectable without taking over the caller's global subscriber. |
| Chrome trace export / Perfetto timelines | Visual phase overlap, threads, counters and flows; time-range analysis of a completed readable capture | No automatic propagation of simulator identities or useful cause labels; an export format does not supply live history or cross-process clock semantics | Optional timeline projection of the same observation contract. Do not create a separate instrumentation authority. |
| `tracing-flame` | Folded stacks from entered/exited instrumented spans; flamegraphs or ordered flamecharts | Not independent CPU sampling; uninstrumented native routines remain opaque, and collapsed stacks lose temporal order | Optional view when semantic span coverage is adequate; distinguish span occupancy from sampled CPU. |
| `perf` userspace sampling and counters | What an owned process/thread executes during the capture; sampled call paths when unwinding and symbols work; selected time-range samples | Does not recover time before attachment, prove waits, or explain why a symbolic computation was requested | On-demand deeper diagnosis. Preserve target process identity, capture interval, event, stack/symbol coverage and losses. |
| Samply | Existing native CPU profiler and interactive profile representation, with Linux perf-event collection | Does not add semantic operation links or reconstruct earlier convergence history | Eligible alternative presentation/collection tool. Exact late-attach and native-symbol behavior needs a bounded integration check before selection. |
| Tokio Console | Live Tokio task states and busy/scheduled/idle times, wake/poll activity and instrumented resource observations | Task idle time is not automatically a known resource wait; arbitrary native worker threads and foreign solver internals are not thereby instrumented | Optional async scheduling investigation, not the universal simulation observation layer. |
| Perfetto system data sources / Linux scheduling and pressure observations | Scheduler activity, process/thread relationships, CPU scheduling and resource pressure | System-wide causality is not granted by adjacency in a timeline; stronger data sources can need privileges absent from a userspace-only capture | Escalate when ordinary process/operation facts leave a material contention question. No global machine reconfiguration is required by this review. |
| DataFusion native operator metrics and instrumentation | Actual operator/partition work, optimizer phases, output cardinality, scan/spill and other available native facts | Summed partition/operator times are not request wall latency; metrics finalized on drop do not supply a live snapshot by themselves | Preserve native units, partition and executed-plan identity; expose bounded live snapshots only where the exact API supports them. |
| SurrealDB query statistics and plans | Native statement execution time; exact-version `EXPLAIN` / `EXPLAIN FULL` access paths and executed cardinality | Client elapsed time is not server execution time; analyzing a query executes it and is not passive observation of a prior invocation | Capture available statistics from the actual invocation. Use separate read-only explain probes only for a stated question, never silently rerun writes. |

The simplest useful target is an owned operation observation with active phase and cumulative snapshots, bounded recent history, and optional library-owned export/sampling tools. A mandatory collector service, universal tracing framework, or dashboard is not justified by the sole-operator requirement. Conversely, console text alone cannot support late inspection, interval analysis and independent consumers reliably.

## Exact source and version boundaries

- The checkout declares `tracing = 0.1.44`, `tracing-subscriber = 0.3.23`, DataFusion `55.1.0`, `datafusion-tracing = 55.0.0` and `instrumented-object-store = 55.0.0` in [Cargo.toml](../../../../Cargo.toml). The datafusion-tracing skill's reviewed metrics and lifecycle briefs were read alongside current `pse-engine::session::{assurance,observation,trace}`. `MetricsRecorder::drop` is completion-oriented. Current engine observations already retain node/partition/native unit and explicit truncated snapshots.
- Current `tracing` documentation distinguishes parent/child inclusion from `follows_from` causal relationships and warns against holding an entered-span guard across `await`: [span relationships](https://docs.rs/tracing/0.1.44/tracing/span/index.html), [Span](https://docs.rs/tracing/0.1.44/tracing/span/struct.Span.html). These are trace semantics, not a ready-made cross-process protocol or proof of a critical path.
- The candidate `tracing-chrome` documentation examined is **0.7.2**, not a checkout dependency. It supplies a subscriber layer and a writer guard: [crate documentation](https://docs.rs/tracing-chrome/0.7.2/tracing_chrome/). A final-file writer needs an explicit live-readable/flush strategy before it can satisfy inspection during a run.
- The candidate `tracing-flame` documentation examined is **0.2.0**, not a checkout dependency: [crate documentation](https://docs.rs/tracing-flame/0.2.0/tracing_flame/). Buffer flush and flamegraph/flamechart differences matter. Neither exporter is selected or installed by this review.
- Context7 resolved Tokio Console to its official repository. Its [README](https://github.com/tokio-rs/console) requires Tokio tracing instrumentation, `tokio_unstable` and enabled runtime targets. [Task documentation](https://github.com/tokio-rs/console/tree/main/tokio-console) distinguishes running, idle, scheduled and done. Current checkout configuration does not establish that this optional instrumented build exists. A diagnostic build is a deliberate choice, not a routine rebuild prerequisite.
- Perfetto's [SQL analysis](https://perfetto.dev/docs/analysis/perfetto-sql-getting-started), [CPU scheduling](https://perfetto.dev/docs/data-sources/cpu-scheduling) and [Linux collection](https://perfetto.dev/docs/getting-started/linux-cookbook) describe different collection and analysis capabilities. Chrome-format import eligibility does not establish full fidelity for every asynchronous link or incomplete interval; verify a shared-work/multi-process example before selecting an exporter.
- Context7 resolved Samply to [the upstream repository](https://github.com/mstange/samply). It is an eligible profiler, not an inspected pinned dependency here. No Samply installation, attach, Rust symbolization or UI check ran.
- Linux [perf security](https://github.com/torvalds/linux/blob/master/Documentation/admin-guide/perf-security.rst), [perf record](https://github.com/torvalds/linux/blob/master/tools/perf/Documentation/perf-record.txt) and [perf tips](https://github.com/torvalds/linux/blob/master/tools/perf/Documentation/tips.txt) are primary references. The distinction between process-scoped userspace collection and kernel/system-wide collection is material; do not infer all capabilities from one successful event.
- Exact vendored SurrealDB `Query::with_stats`, `WithStats<IndexedResults>::take` and `Stats::execution_time` were inspected in `vendor/surrealdb/src/method/query.rs`. The ordinary operations crate has no `.with_stats()` consumer in the bounded source search. Its capability tests already exercise executed `EXPLAIN FULL`. Current online documentation spans multiple query-engine generations; transfer only semantics confirmed in the vendored 3.3 source and existing tests.

`Cargo.toml` already has a `profiling` profile with full debug information, distinct from ordinary development's line tables and dependency debug stripping. This is a useful preservation constraint. Usable stack resolution across Rust, Python, native solver libraries and stripped dependencies is nevertheless an integration question, not established by the profile's name.

## P01 — Small launched-process counter probe

**Tested, bounded tool capability only:**

```text
PSE_TEST_RESOURCE_STATE=/tmp/pse-friction-review-resources-uMrtOxGw \
scripts/pse-env --resource-class light -- \
perf stat -e task-clock -- /usr/bin/true
```

Exit **0**. The tool reported a userspace task-clock measurement. This establishes access to that event on one owned launched process, not useful simulation stacks, kernel profiling, hardware-counter availability or an overhead bound. The reported duration is not a simulator performance measurement.

The same read-only host inspection found `/usr/bin/perf`, `perf version 7.0.14`, `perf_event_paranoid=2` and `kptr_restrict=1`. No sysctl, file capability or host permission was changed.

## P02 — Late attachment to an owned short-lived control

**Tested collection; symbolized call-path capability unresolved.** A scratch Python controller launched one Python busy-loop child for approximately 1.5 seconds, attached this sampler, awaited its own child, then interrupted and joined its own sampler:

```text
perf record -q -e cpu-clock:u -F 49 --call-graph dwarf \
  -o <owned-scratch>/perf.data -p <owned-child-pid>
perf script -i <owned-scratch>/perf.data
```

The child exited **0**. The sampler's raw exit status was **-2**, the explicitly requested SIGINT; this is not relabelled as an ordinary successful exit. Decoding exited **0** and yielded 138 text lines containing sample timestamps, but the inspected output contained **no resolved instruction or call-chain frames**. The event attributes requested IP, callchain, user registers and user stack; `perf version --build-options` advertised DWARF unwind support. These facts do not settle why useful frames were absent.

Raw artifacts remain in `/tmp/pse-diagnostics-profile-probe-6mma0auv/` (`result.json`, `record.log`, `perf.data`, `stacks.txt`, `decode.log`). All probe processes were joined. No simulation, worker, storage service, existing workload or repository-tracked code was started, stopped or modified by this probe.

This is a decision-relevant negative control: a process can be sampled and its output decoded without delivering the useful profile the operator asked for. A future profile command must report sample, frame, symbol and target coverage separately and retain a clear partial/unavailable result. A real Rust-plus-native control is needed before claiming that it can localize the stalled simulation.

## Remaining settling checks

1. Verify useful Rust/native call paths and process/thread targeting on one bounded representative executable; do not resume the ten-sample cold campaign to test the profiler.
2. Verify a live-readable trace export with an open long interval, a shared producer with two waiters and an interrupted writer. Do not assume a valid final trace file is readable during execution.
3. Check optional Tokio diagnostics on the exact enabled feature/build profile only if scheduling remains a material unknown; distinguish its native-thread blind spots.
4. Pair process CPU deltas with owner-published phase state before classifying active computation versus waiting. A stack sample or task idle total alone cannot establish a deadlock.
5. Preserve the original trace/summary semantic distinctions when producing a timeline, flamegraph or machine-readable report. Profile tools do not own convergence, validity or reuse policy.
