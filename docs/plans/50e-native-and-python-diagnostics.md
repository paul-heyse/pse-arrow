---
title: "Plan 50e: Native and Python diagnostic capabilities"
status: draft
date: 2026-10-22
adrs: []
review_sources: [docs/functional_reviews/codex-effectiveness-and-diagnostics-2026-10-22.md]
doc_role: plan
doc_owner: docs/plans/50-codex-effectiveness-and-diagnostics.md
doc_topics: [diagnostics, performance, numerical-execution, environment, validation]
---

# Plan 50e: Native and Python diagnostic capabilities

## Purpose and assessed foundation

Deliver O14–O17 and the deeper-counter/rr alternatives: useful mixed stacks, crashes, actual
scheduling classes, suspended tasks and allocating stacks. These answer questions beyond existing
operation snapshots and user CPU sampling. The [coordinator](50-codex-effectiveness-and-diagnostics.md)
owns state and feasibility outcomes. **Interface-checked:** `bb61ae3f5`; **Proposed:** this target.

Installed-tool checks from the source review found perf, Samply, GDB15.1, py-spy0.4.2 and
Heaptrack1.5; Python3.14.8 reports `PY_HAVE_PERF_TRAMPOLINE=1`. Memray/rr were not found on that
invocation's PATH. Tool presence is not attachment, unwinding, permission or coverage qualification.
The host routes cores through Apport, not an established `coredumpctl` service.

Current `CaptureSession` owns CPU/timeline collectors and leaves target work independent; its
documents have perf-specific symbols/loss semantics. Do not add unrelated active-action enum
variants under that promise. `pse-runtime::workflow::Runtime` is also not a Tokio executor owner:
Python owns its executor in `pse-py::inspection::runtime`, while Rust callers supply theirs.
Neither passive `Diagnostics` nor `OperationReader` should acquire hidden executor/control authority.

## Target contracts and integration

Provide action-specific requests/results alongside existing tooling commands. Share subject and
artifact helpers where their meanings agree, not a universal debugger framework. Inputs are
distinct: an exact live process/start locator; an independently owned diagnostic launch; a
cooperating actual executor handle; or explicitly supplied postmortem artifacts. An operation ID
alone is not attachment authority. Preserve optional operation attribution and unknown coverage.

Each action reports its actual effect: sampling, stopped-process inspection, cooperative task
re-polling, allocation interception or offline interpretation. Tool execution, artifact readability,
attachment/detach coverage and target scientific/command outcome are independent. Diagnostic
failure cannot cancel unrelated science or turn it into a scientific failure. An owned experimental
launch has normal command cancellation/drain responsibilities; an attached target retains its owner.

Retain raw data and useful versioned summaries with actual tool/options, identities, coverage,
loss and symbol references. The diagnostics command reads its action document directly, including
standalone attach/core/taskdump actions with no test invocation. C01/C03 follows associated
artifacts when a real invocation exists; existing review packs retain selected outputs. Do not
manufacture invocation IDs or copy arbitrary
original paths or executables during offline import. A core is not `ProfileBinary`: initially
retain its interpreted report/reference; copying cores requires an explicit selected member format
and size/lifetime handling in the existing pack owner.

The first interfaces use direct existing tools through the diagnostics command owner; public
Rust/Python documents use the current catalogue/generator. Host tool execution remains outside
the scientific runtime dependency graph. Consume Plan49g TOOL05's actual boundary, not completion
of all 49g, and keep product hooks limited to actual executor/source observations.

## Packages and feasibility decisions

<a id="E01"></a>

### E01 — Action-specific subjects, effects and retained output

Implement the common projection/identity/artifact slice and first offline/launch consumers.
Include the standalone action-document reader and optional genuine run links described above.
Request validation refuses changed process identity, arbitrary path following and unsupported
attachment modes. Preserve process start/clock facts and explicit missing operation association.
Inspection of an old pack never attaches to a live process merely because a PID/path was recorded.

Select actual CLI capabilities and exact tool versions through existing setup/dependency routes
when a tool needs installation. No unconditional suite of profilers is installed. Diagnostic
setup is explicit and separate from passive reads. Tests specify subject/effect/output meaning
without invoking a full numerical model; do not build a generic shell-command execution DSL.

<a id="E02"></a>

### E02 — Mixed-language stack and crash workflows

Resolve I03 with one small real Python→PyO3→Rust→native journey. Select CPython `-X perf` /
`PYTHONPERFSUPPORT` integrated with existing perf capture first. Use `-X perf_jit` and
`perf inject --jit` when actual unwinding needs that route. Keep maps/JIT unwind data available
through decoding. Qualify perf decoding and Samply import separately; useful retained decoded
stacks can be supported even if the viewer lacks some Python frames. Do not promise portable
symbolication from an unconsumed sidecar setting.

Use installed py-spy for Python-thread/GIL snapshots and explicitly selected native sampling.
Native mode pauses the target and cannot combine with nonblocking mode; Python-thread coverage
does not include every Rust-owned worker. Report covered/uncovered interpreter, released-GIL and
native-worker populations. If neither route yields useful mixed frames, this capability remains
unfinished; an unavailable report alone cannot close it.

For crashes, first support explicit diagnostic launches and supplied-core GDB batch backtraces,
with selected faulthandler output, matching executable/shared-object identity and last checkpoint.
Python traceback and current-thread C-stack populations are distinct and build/platform dependent.
Use the actual Apport route; absent/truncated/inaccessible cores remain absent. GDB output keeps
optimized-out frames and missing symbols explicit. No new signal handler is necessary.

Live GDB attachment is a separate selected mode requiring detach/cleanup qualification: capture
pre-attachment stop state, actual attach/detach and uncertainty. Never unconditionally SIGCONT
a previously stopped process or kill an attached target because the helper timed out. A command
launched under GDB is owned experimental execution and can be cancelled/drained by that owner.
Refusing unsafe live attachment does not block the useful offline/launch route.

Acceptance includes the real mixed journey, a tiny intentional native crash, unavailable core,
wrong executable identity, released-GIL worker gaps and helper interruption. Symbol preservation
is a concrete debugging need, not a reinstated all-role provenance campaign or routine rebuild.

<a id="E03"></a>

### E03 — Scheduling classes and evidence-supported wait causes

Resolve I04 using `perf sched record`/`timehist` and a short known CPU/sleep/contention control.
Permission for `cpu-clock:u` does not establish tracepoint permission. Identify an unavailable
host prerequisite precisely; BPF does not automatically bypass the same access restriction.
Do not substitute `/proc` sampling and call it equivalent scheduling attribution.

Preserve PID/TID/start identity, monotonic mapping, wake/state evidence, losses and unmatched
threads. Report running, scheduled-out and runnable delay. I/O, futex, lock or supervisor join
causes require actual owner-phase/syscall/lock evidence. Neither a sleeping stack nor wall-minus-
CPU identifies a lock owner or proves deadlock. Join existing native-thread scopes without
adding overlapping phase durations as exclusive elapsed time.

Deliver the available capture with useful summary and explicit unavailable states. A required
host permission awaiting operator action stays a precise prerequisite, not a passed feature.
No machine-wide tuning or performance threshold is implicit. Actual perturbation is recorded
when measured; a profiler-enabled run is not ordinary timing evidence.

<a id="E04"></a>

### E04 — Cooperative inspection of the actual async executor

Select pinned Tokio1.53.2 `Handle::dump()` for the pending-await question. Bind the action to
the actual Python executor or a caller-supplied Rust handle, with its population stated.
Offer a direct/cooperative diagnostic launch or explicit caller API; the library does not supply
arbitrary external runtime attachment. Do not create another executor or permanent control server.

Use a named diagnostic Cargo profile inheriting the existing profiling profile, an explicitly
selected `tokio/taskdump` feature and invocation-scoped `tokio_unstable` through the current
build-environment owner. Preserve its linker/rust flags and force-validation when performing
correctness checks. Profile directories isolate artifacts inside the normal target tree; never
override the pinned toolchain or manually export `CARGO_TARGET_DIR`. A diagnostic Python extension
stays distinct from the ordinary editable build and is selected/reported explicitly.

Resolve I05 with a pending-channel task, blocked worker and local-executor coverage case.
Dumping pauses/synchronizes workers and re-polls tasks. An in-runtime timeout may never run on
a wedged executor; an independent observer must retain the ability to report noncompletion and
end its own collection without claiming executor recovery. Unsupported LocalSet/other task
populations remain explicit.

Qualify cancellation/drop behavior before and after dump issuance. A stopped observer does not
mean a queued or in-progress target request drained. Retain target-side ownership and distinguish
pending, abort-requested, completed, acknowledged cancellation and unknown completion until the
actual owner settles it. Test cancellation before issuance, after issuance, a blocked worker and
observer exit; collector completion alone cannot release the target-side request lifetime.

Tokio Console is conditional for poll/scheduled/idle/wake/resource dynamics beyond task traces.
Its recording requires finalization adaptation before being called a durable offline report.
The supplied tooling overview's server Console is another target: the selected server omits
`tokio-console`, and environment-only activation warns without enabling it. A positive I11
decision uses a separately identified build of the same patched source, required Tokio tracing,
owned loopback listener and explicit supervisor restart. Never alter the normal server silently
or confuse server tasks, app tasks and native-solver threads.

<a id="E05"></a>

### E05 — Allocating-stack investigation

Select present Heaptrack/headless `heaptrack_print` for actual native executables. Resolve I06
with known temporary and retained allocations; show call hotspots, peak consumers and allocating
frames. Profile the executable, not Cargo. Retain symbol/library references and custom allocator/
pool omissions. These are not exact operation-private peaks, RSS or reservation accounting.

Assess Memray `--native` only for an actual Python-object/native allocation question not covered
by the native route. A positive decision selects/pins it deliberately and tests current Python
compatibility. Native report generation needs matching interpreter/libraries; raw files alone
do not promise portable symbolication. No production allocator change is selected for observability.

Accept useful native attribution and a concrete Memray adopt/no-adopt decision with unsupported
coverage explicit. Optional profiler refusal is not proof that the code has no allocation churn.

<a id="E06"></a>

### E06 — Deeper counters and reverse debugging disposition

Offer `perf stat` as a selected direct action after locating a hot path. Preserve actual workload,
supported events, running/multiplexing fractions and measured interval. No generic IPC/branch/cache
threshold is introduced. Counter observations test a concrete hypothesis, not a language ranking.

Resolve I10 for rr with a compact owned FFI failure process tree. Check host CPU/kernel/syscall
support and external shared-memory limitations. A useful route supplies record/replay and reverse
inspection with retained evidence; otherwise record unsupported/no-adopt. Its effectively
single-core recording is never parallel-performance evidence. No complex wrapper is required
when direct established commands satisfy the chosen consumer.

## Verification and sources

**Proposed:** small independent controls establish useful stacks, trace classes, identity checks,
safe detach, noncompletion, allocator population and capture finalization. Actual symbol/thread
coverage must accompany the real examples. G02 composes selected pipeline journeys after functional
delivery; static checks occur once at scope end. No full benchmark/scientific campaign is required.

Sources: existing `tooling/capture`, inspection/review-pack owners, native-operation supervision,
`pse-py/src/inspection/runtime.rs`, Cargo/build environment profiles and
[diagnostic guide](../dev/simulation-diagnostics.md). Primary mechanisms:
[Python perf](https://docs.python.org/3.14/howto/perf_profiling.html),
[faulthandler](https://docs.python.org/3.14/library/faulthandler.html),
[py-spy](https://github.com/benfred/py-spy),
[GDB](https://sourceware.org/gdb/current/onlinedocs/gdb.html/Backtrace.html),
[perf sched](https://github.com/torvalds/linux/blob/master/tools/perf/Documentation/perf-sched.txt),
[Tokio1.53.2 dump](https://docs.rs/tokio/1.53.2/tokio/runtime/struct.Handle.html#method.dump),
[Console](https://github.com/tokio-rs/console),
[server Console](https://github.com/surrealdb/surrealdb/blob/v3.3.4/surrealdb/server/src/telemetry/console/mod.rs),
[Heaptrack](https://github.com/KDE/heaptrack),
[Memray](https://bloomberg.github.io/memray/native_mode.html), [rr](https://rr-project.org/).
