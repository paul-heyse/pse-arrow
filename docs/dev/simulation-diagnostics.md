---
title: Simulation diagnostics and explicit capture
status: active
doc_owner: docs/authoritative_design/sections/operations-and-validation.md
doc_topics: [environment, validation]
---

# Simulation diagnostics and explicit capture

Ordinary runtime activity gets an operation identity before preparation. The Rust interface
is available through `Runtime::observations()`, `RunHandle::observations()` and retained
surrogate `observations()`. These readers copy diagnostics; they cannot cancel a run, keep a
native solver alive, qualify a candidate or open provisional scientific results. The owning
contract is blueprint §23.1; [Plan 40's checkpoint](../plans/40-simulation-performance-diagnostics.md#current-checkpoint)
records implementation and qualification limits. The interfaces below describe the implemented
source surface; they do not establish assembled or all-context qualification.

A runtime's hub lists operation summaries with pagination. An exact operation reader returns
`snapshot(Duration::from_secs(60))`, including active phases, current and cumulative metrics,
recent counter/duration aggregates, point history, relationships, source references and
independent coverage. Inspecting before a RunHandle exists uses the same hub. It does not
ask the preparation owner to do work.

Each process publishes one atomic JSON checkpoint under `PSE_DIAGNOSTICS_DIR`, or
`$XDG_STATE_HOME/pse/diagnostics`, or `$HOME/.local/state/pse/diagnostics`. The process locator
contains its instance identity rather than PID alone. Checkpoint version **2** adds the actual
OS PID, optional Linux process start ticks and executable path, a same-host `CLOCK_MONOTONIC`
mapping with explicit uncertainty, and bounded actual native-thread scopes. A thread scope
contains its operation, OS TID, optional thread start ticks and observed entry/return times;
it is not an inferred async-task schedule. Missing alignment or thread identity remains
unavailable. CPU attachment rechecks process start identity rather than trusting a reusable PID.
These locators support operation and attachment, not build or provenance attestation.
`Diagnostics::checkpoint_path()` names
the exact local file; `pse_runtime::observation::read_checkpoint(path)` reads an already
published image independently of the busy caller. Managed workers use their own process
identity and actual canonical run/attempt/occurrence references. Private native children
publish their actual operation locator and diagnostic parent. Publication time and producer
checkpoint time are separate; cross-process reads are not atomic.

Read phases first to locate observed activity, then its metrics and related producer. A
supervisor's join wait overlaps native execution. Shared-flight waiters link to one producer;
their waiting intervals are not additional construction or exclusive CPU time. A stale
checkpoint or opaque native interval alone does not establish a hang.

Context producers keep their own meanings. Continuation exposes attempted displacement,
rejection/subdivision and committed predecessor; fitting experiments have separate child
identities; horizons distinguish foreground control, prediction and next-sample background
work. Sensitivities, root responses, Hessians and covariance propagation report their own
availability and withholding causes. One unavailable quantity does not invalidate another,
and none of these observations grants scientific permission.

Point history keeps first/latest values and their actual timestamps per second, with a
coalesced observation count and generation/reset markers. It distinguishes changed from
unchanged native values over the retained interval. Unavailable and nonfinite samples stay
explicit. A predecessor can establish the last observation before the requested window;
it is not an observation inside that window. Coalescing, lost points and curtailed history
remain visible. Counter deltas spanning a request boundary are partial, not fabricated
uniform rates. Native checkpoints remain distinct from accepted or original physical states.

Collection defaults to a 60-second recent window, 128 metric slots, 256 detail events and
16 MiB of internal process diagnostic allowance, including bounded transport buffers and
one read-copy allowance. Caller-retained copies are caller memory. Publication normally
runs once per second and is limited to 1 MiB: active/recent work takes priority, compacted
history and omitted operations are explicit. These limits affect diagnostics only.
`PSE_DIAGNOSTICS=off` selects absent collection at the first process registry initialization.
Engine `ObservationPolicy::Off` separately leaves operator metrics uncollected.

Process RSS, lifetime resident high-water and CPU ticks have process scope. Shared pool
reserved bytes have shared-runtime scope. Neither supplies a private operation peak or
exclusive CPU time. Server query nanoseconds are supplied statement samples; they are not
client wall intervals. The fixed `canonical.query.latest_sample` text field contains an
atomic JSON `pse_operations::canonical::CanonicalQuerySample`: actual invocation, statement
index, status and optional exact server nanoseconds travel together. Recent history retains
first/latest samples with coalescing counts; it is not a complete query ledger. Fixed counters
retain received, successful/failed and timed/untimed statement counts. Request failure has
its own sample and counter, without invented statements. One shared origin reference leaves
room for actual run/attempt identities. Idle managed-worker discovery creates no child nodes;
real candidates own observations before preparation. Unknown values are not zero.

Last runtime registration requests final publication asynchronously. Explicit
`Diagnostics::release()` refuses while actual registrations remain, then joins the publisher
and removes only that process's exact checkpoint/pending files. Readers do not block release.
No age/size-based deletion is selected. Optional writer failure is available independently
through `writer_failure()` and does not change scientific permission. Hard death preserves
at most the last published prefix; normal scope abandonment records interruption.

## Preparation work and waiting

Preparation exposes work at its performing owner. Interpret these counters with the
operation's collection coverage and generation; missing metrics do not establish zero work.

| Metrics | Observed work |
| --- | --- |
| `modeling.import_candidates` | Declarations examined in requested root buckets, including repeated import groups. Unrelated root inventory is excluded. |
| `modeling.lexical_base_builds`, `modeling.lexical_base_bindings`, `modeling.type_overlays` | Actual immutable lexical bases and their bindings, then current occurrence frames. Evaluation and source inference have distinct bases where their meanings differ. |
| `math.selected_function_constructions`, `math.selected_function_bodies`, `math.selected_function_rows` | Selected plan constructions, distinct required complete bodies and selected rows. Coupled provider obligations can legitimately require broader support within a selected body. |
| `compiler.observation_structures` | Structures constructed for observation key requests. A miss carries that same structure into construction; a hit still needs its key request. |
| `compiler.projection_lookups`, `compiler.projection_lookup_entries` | Shared runtime/formal lookup construction and its entries for a grouped projection. Consumer-local normalization remains separate. |
| `study.parent_cancellation_handled` | Actual study frontier cancellation handling, independent of the child token already observing its parent. |

The `math.compiler_admission` phase is a wait for exclusive workspace authority before
active CPU/native dispatch. It still occupies the existing bounded population. A subsequent
`native.join.wait` can overlap execution and foreign teardown; neither interval is exclusive
CPU time. Shared-flight relationships distinguish one producer from its individual waiters.

`EngineSettings.math_preparation_limit_ms` selects the finite preparation omission and
frontier/completion producer allowance. An enclosing caller clock takes precedence and is carried through
selected compilation, artifact waits and value rebinds. A producer's clock has its own
origin; one waiter leaving does not stop another valid waiter. Expiry requests cooperative
stopping while issued native work stays owned through actual join. These preparation clocks
are separate from solver stopping controls and admission-only waiting; blueprint §14.3 owns
their contract. Counters establish work under the exercised demand, without implying an
elapsed speedup or diagnosing numerical convergence.

## Native work and reuse

Read these facts at their producing operation, with the same interval and generation.
They distinguish retained preparation from fresh numerical work; a reuse counter alone
neither establishes a speedup nor qualifies the resulting simulation.

| Metrics or phases | Interpretation |
| --- | --- |
| `idas.layout.nonzeros`, `idas.layout.bytes` | Complete all-mode state support, including required diagonals, and retained Rust layout capacity. These do not measure native factor fill. |
| `idas.linear.workspace.bytes`, `idas.linear.known_workspace_allowance.bytes`, `idas.linear.opaque_factor_allowance.bytes` | Selected forward workspace allowance, separated into shape-derived storage and conservative opaque direct-factor fill. Forward SPGMR/SPFGMR carries no KLU fill allowance. Allowances are not measured allocation or RSS. |
| `idas.adjoint.opaque_factor_allowance.bytes` | Backward KLU fill allowance, including the enlarged second-order problems and requested directions. A forward Krylov selection does not remove the backward KLU owner. |
| `dynamics.workflow_worker_constructions`; `fit.experiment_worker_constructions`, `fit.experiment_worker_reuses` | Successful workflow worker construction, and fitting worker construction/reuse entries at their respective owners. A fitting construction entry can still fail. These describe mathematical/provider ownership; fewer constructions can coexist with the same number of fresh solves. |
| `horizon.plant_worker_constructions`, `horizon.plant_worker_reuses` | Actual plant-worker acquisition on the staged owner thread, including initial evaluation and subsequent periods. Worker reuse preserves neither a prior IVP nor completion of the current period. |
| `dynamics.window_layout_constructions`, `dynamics.window_layout_reuses` | Lazy observed-quadrature layout construction and subsequent reuse. Plain anchored windows share the base layout without this transform. |
| `dynamics.window_ivp_initializations`, `dynamics.window_adjoint_initializations` | Window integration/adjoint requests entering initialization, including requests that later refuse or fail. Retained workers do not carry native IVP history across changed trials. Check actual `idas.forward`, `idas.forward_checkpointed` and `idas.backward` phases and their `.calls` counters for issued native work. |
| `fit.transient_curvature_integrations`, `fit.transient_curvature_completions`, `fit.transient_curvature_reuses` | Per-experiment curvature integration requests, successful publication of a complete point product, and reuse of that completed product. Repeated exact-point Hessian assembly can reuse curvature while applying different solver weights; it does not establish retained integration/checkpoint history. |
| `dynamics.sample_elimination.symbolic_constructions`, `dynamics.sample_elimination.symbolic_reuses`; `dynamics.sample_elimination.faer.factor` | Compatible canonical algebraic patterns reuse symbolic LU analysis. Each sample still constructs a fresh numeric factor; the factor phase's `.calls` includes failed attempts. Within that sample, multiple right-hand sides share its factor; compare `dynamics.sample_elimination.solve.calls`. An all-differential sample needs no algebraic factor. |
| `native.session.constructions`, `native.session.pool_checkouts`, `native.session.pool_returns` | New native owner threads, actual neutral-owner checkouts and completed returns after reset. A returned owner retains only what its adapter's neutral-reset contract permits; checkout does not establish reuse of numeric state, seeds or a previous task clock. |

For fitting, also compare `fit.new_trials`, `fit.derivative_upgrades` and
`fit.trial_cache_reuses`. A same-point demand upgrade can require fresh missing products;
an already completed curvature product is a separate fact. Failure or cancellation
cannot publish partial curvature as a completion. Counts describe the observed demands,
not every optimizer iteration or every scientific output.

The explicit compiled scalar evaluator adds the `math.native_scalar_compilation` phase,
`math.native_scalar_compilations`, `math.native_scalar_evaluations` and
`math.native_scalar_executable_bytes`. Compilation counts successful loaded modules;
evaluation counts native arithmetic calls, not independently accepted outputs. Executable
bytes describe the generated file, not mapped-memory residency. Compare compilation work
with actual repeated value demand rather than assuming amortization.

This opt-in `compiled-evaluator` feature uses the fixed Linux x86-64 scalar C++ profile
for Value arithmetic blocks, with independent worker scratch and owned executable lifetime.
Derivatives and provider calls remain interpreted. It selects neither a SIMD/batched route nor a host-native CPU
target, enables neither fast math nor FP contraction, and does not interrupt an
individual compiled call; cancellation is checked at its surrounding boundaries. These
producer facts do not establish all-context evaluator or assembled workflow qualification.

## Artifact publication work

`artifact.metadata.attach` measures each bounded transaction attaching already sealed artifact file metadata. `canonical.artifact.metadata.groups_acknowledged` and `canonical.artifact.metadata.rows_acknowledged` count only groups and rows whose exact ordered key inventory was received and validated. File seals remain independent and differently shaped IPC data stays in separate files. A lost ACK remains unsettled and cannot authorize activation; these counters are neither a complete query count nor evidence of elapsed improvement.

## Typed inspection and reader ownership

For the eleven selected Plan39 simulation cases, the existing collector offers
`just case-measure <fresh-output> --smoke --audit-diagnostics --case <id>`.
This opt-in route runs one complete case observation with the original assertions,
retains typed stage/live/terminal inspection documents under each case's `diagnostics/`,
and captures a bounded CPU/timeline window when an operation remains active long enough
to inspect. It supports only the explicitly wired cases listed in
[39d D4](../plans/39d-qualification-and-plan38-cutover.md#d4--development-context-prerequisites-and-once-only-simulation-audit).
The selected reference store must be started inside its existing reference allocation
owner and drained before that owner exits.

`case-operation.json` names the exact checkpoint and operation. An explicit
`diagnostics/stop.request` requests cooperative cancellation through the case's owners;
it is not evidence of native completion. Terminal outcomes keep fixture cleanup and
native/managed joins separate from scientific completion. Ctrl-C isolates the child
from the terminal signal, writes that request and waits for the owned child; it does
not forcefully kill native work. Production cancellation defects can therefore delay
drain and must remain visible in the audit. This route supplies design-phase evidence,
not statistical benchmarking or whole-plan qualification.

`pse_runtime::inspection` owns the explanation shared by Rust, Python and the CLI. Its
`InspectionRequest::new(operation)` selects the preceding **60 seconds**, includes directly
related operations and defaults to 16 related copies, with an effective ceiling of 32.
`InspectionDocument` carries the selected snapshot, related snapshots, omitted count,
transport state and typed statements. Each statement retains its strength (`Established`,
`Hypothesis` or `Limitation`), supporting evidence and next discriminating observations.
Displaying a hypothesis or a proposed capture does not execute it or establish a cause.

Requested and observed intervals differ. A thirty-second operation supplies at most its
thirty-second lifetime; a late read can have a missing suffix, coalesced boundary bucket,
expired history or stale source. Inspect `requested_interval`, `interval_start`, `interval_end`,
`interval_partial`, metric coverage and transport partiality. Missing work is never filled
with zeros, and counter subtraction cannot cross incompatible attempts or resets.

In Rust, retain the runtime's diagnostic facade before starting preparation and read its
bounded catalogue from an independent caller:

```rust
use pse_runtime::inspection::InspectionRequest;

let observations = runtime.observations().clone();
let page = observations.hub().list(0, 16);
if let Some(summary) = page.operations.first() {
    let document = observations.inspect(InspectionRequest::new(summary.id));
    // Render document.statements and their evidence without reinterpreting science.
}
```

`RunHandle::observations()` returns an optional exact `OperationReader`; its
`snapshot(Duration::from_secs(60))` and
`pse_runtime::inspection::inspect_snapshot(snapshot, request)` read copied facts. A retained
reader remains independent of the run handle and its cancellation lease. Terminal diagnostic
outcomes remain observations of their original owner, not scientific result permissions.

Python exposes the same ownership through `Runtime.observations()` and
`RunHandle.observations()`. `Diagnostics` retains only the diagnostic collection; an
`OperationObservations` retains only its exact operation read lease. Neither retains the
native runtime caller, cancellation owner or publisher registration. Obtain the facade before
blocking preparation and pass it to another thread when live inspection is needed. Closing a
runtime does not invalidate already retained diagnostic facts or authorize new scientific work.
Readers do not keep publication running or prevent explicit checkpoint release.

```python
from pse.contracts.documents import ClosedDuration, InspectionRequest

observations = runtime.observations()  # runtime is an existing caller
page = observations.list(offset=0, limit=16)
if page.operations:
    selected = page.operations[0].id  # select the actual desired operation
    reader = observations.reader(selected)
    if reader is not None:
        snapshot = reader.snapshot()  # preceding 60 seconds
        document = reader.inspect(window=15.0)  # this exact operation only
    document = observations.inspect(InspectionRequest(
        version=1,
        operation=selected,
        window=ClosedDuration(secs=60, nanos=0),
        include_related=True,
        related_limit=16,
    ))

run_reader = run_handle.observations()  # optional; run_handle already exists
if run_reader is not None:
    document = run_reader.inspect()
```

Use the collection's `inspect(request)` for related navigation. The exact Python operation
reader's `inspect(window=...)` selects only that operation. Python documents are generated
from Rust owners and pass through `pse.codec`; adapters do not define another JSON shape.

## Standalone reads and the CLI

Standalone discovery and inspection read published files without constructing a runtime,
starting a publisher, connecting to a scientific store or asking an occupied owner to work.
Rust provides `inspection::default_directory()`,
`inspection::list_processes(directory, DiagnosticListRequest)` and
`inspection::inspect_checkpoint(path, InspectionRequest)`. Python provides:

```python
from pse import diagnostics
from pse.contracts.documents import ClosedDuration, DiagnosticListRequest, InspectionRequest

page = diagnostics.list_processes(DiagnosticListRequest(after=None, limit=16))
for process in page.processes:
    if process.failure is not None:
        print(process.checkpoint, process.failure)
        continue
    for summary in process.operations:
        selected = diagnostics.operation_id(diagnostics.operation_text(summary.id))
        document = diagnostics.inspect_checkpoint(process.checkpoint, InspectionRequest(
            version=1,
            operation=selected,
            window=ClosedDuration(secs=60, nanos=0),
            include_related=True,
            related_limit=16,
        ))
        for statement in document.statements:
            print(statement.kind, statement.message, statement.evidence)
```

Discovery returns up to 16 process prefixes by default, capped at 32. `page.next` is an
exclusive checkpoint filename cursor for a subsequent `DiagnosticListRequest(after=page.next,
limit=16)`; it is not a durable catalogue generation. An unreadable prefix retains its path
and failure. A missing default directory yields an empty page without creating it. An exact
checkpoint read that fails raises an observation error; it does not imply numerical failure.

```bash
just diagnostics list
just diagnostics list --directory /path/to/diagnostics --limit 16 --json
just diagnostics list --after PREVIOUS_CHECKPOINT_FILENAME --limit 16
just diagnostics inspect "$checkpoint" "$operation"
just diagnostics inspect "$checkpoint" "$operation" --window 15 --related-limit 8 --json
```

Set `checkpoint` and `operation` to the exact values shown by `list`. The operation locator is
32 hexadecimal digits for the two process-instance words, followed by `:` and its unsigned
decimal sequence. `diagnostics.operation_id(text)` and `operation_text(id)` use the Rust
parser/formatter. This locator is not a scientific run identity. The default CLI view shows
the selected operation and intervals, active phases, metric summaries and partiality,
Rust-owned statements with evidence/next observations, and compact related-operation
navigation. It omits retained point/counter arrays from the human view; `--json` emits the full generated response
encoding. Its thin recipe uses the existing checkout/native environment and light workload
admission, without rebuilding an extension or opening a database.

## Explicit bounded capture

Inspection never starts capture implicitly. `pse_runtime::capture::start_capture(request)`
and `pse.diagnostics.capture(request)` start a separately owned optional session from an
exact published checkpoint and selected operation. `CaptureRequest` version 1 selects
`Timeline` or `Cpu`, an output directory that must not already exist, duration and budgets.
Defaults are **10 seconds**, **16 MiB** of exporter working allowance and **256 MiB** of retained
output allowance. Duration must be positive and no more than one hour; working allowance is
at least 4 MiB and output allowance is positive. External tool working sets are separate from
the in-process exporter allowance.

```python
from pse import diagnostics
from pse.contracts.documents import CaptureBudget, CaptureRequest, ClosedDuration
from pse.contracts.enums import CaptureKind

request = CaptureRequest(
    version=1,
    checkpoint=str(checkpoint),
    operation=selected,  # an actual discovered OperationId
    kind=CaptureKind.TIMELINE,  # select CaptureKind.CPU explicitly for sampling
    output="/tmp/pse-timeline-new",
    duration=ClosedDuration(secs=10, nanos=0),
    budget=CaptureBudget(memory_bytes=16 << 20, output_bytes=256 << 20),
)
with diagnostics.capture(request) as session:
    prefix = session.snapshot()  # capture status and flushed readable output prefixes
    captured = session.wait()  # releases Python execution while joining capture
```

```bash
just diagnostics capture "$checkpoint" "$operation" --kind timeline --output /tmp/pse-timeline-new
just diagnostics capture "$checkpoint" "$operation" --kind cpu --output /tmp/pse-cpu-new --duration 10
just diagnostics capture "$checkpoint" "$operation" --kind timeline --output /tmp/pse-small-new --memory-bytes 16777216 --output-bytes 268435456 --json
```

Use a new output directory for every session. Rust `CaptureSession` and Python
`CaptureSession` expose `snapshot()`, `request_stop()`, `wait()` and `stop()`. Stop and drop
request capture shutdown and join only owned exporters, samplers and tool children. Python
wait/stop release interpreter execution; an interpreter signal requests capture stop and
drains capture before escaping. They never cancel, join or restart the scientific operation,
take over its native handle, install a global subscriber or grant result access. Joining may
include bounded exporter flush, symbol decoding and profile conversion after the requested
sampling interval. Capture files remain available after session stop/drop.

`Timeline` writes standard Perfetto packets from owner-published operation/phase/thread times,
metrics, known related producers and waiter links. Rust callers may compose `session.layer()`
into their own tracing subscriber; the optional layer enqueues bounded records and does not
replace a global subscriber. Previously published owner timestamps can appear in the timeline,
but absent starts/ends or unknown cross-process clock alignment are not reconstructed. An open
interval stays open until its actual end is observed; capture stop is not an operation end.
Use each output's `readable_prefix_bytes` and `readable` fields when decoding a live or partial
artifact; file length alone does not establish a complete packet stream.

`Cpu` uses explicitly attached userspace `perf` sampling with DWARF call chains, then bounded
`perf script --no-inline` frame decoding and Samply save-only import. The bounded frame-name
check omits inline expansion; the raw recording remains available for deeper analysis.
CPU capture requires the selected operation to be active in its published checkpoint.
For a completed root with continuing work, select its active related operation; use `Timeline`
to read retained history. Sampling cannot recover CPU work that has already finished.
It does not launch a UI/server or
change host permissions. It can follow known published native-child/worker process links;
target identity is rechecked before attachment. Sampling begins at attachment: it cannot
recover the preceding minute's CPU samples. Process/task samples, including inherited work,
remain inclusive; published operation/thread locators do not make CPU exclusive to that
operation. Concurrent operations and partial thread attribution remain explicit. Check
`loss.sampling_inherited_tasks` and `loss.sampling_overlap_possible`: forked descendants
can also be recorded separately, so their samples must not be added as exclusive CPU time.
Inspect `symbols.checked`, resolved/unresolved frame counts and bounded names rather than treating
file creation or profile conversion as useful-frame evidence. Missing tools, permissions,
unwinding, symbols or target identity surface as independent capture failures/partiality.
Kernel sampling loss remains unknown unless quantified by the adapter.

`CaptureDocument` keeps requested versus observed duration, targets and clock uncertainty,
output prefixes, symbol evidence, open intervals, losses and errors separate from scientific
outcomes. Output exhaustion, refused events, source loss and unknown clocks remain visible.
Neither a completed capture nor a readable file establishes numerical correctness or a root
cause. Tokio Console, allocation/native-heap and off-CPU/wait integrations are not supported
by this capture surface; their suggested next observations remain optional and unavailable,
not zero work. Current acceptance and remaining qualification belong to Plan 40c/40e.
