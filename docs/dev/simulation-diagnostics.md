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

Active identities and coverage take priority over completed history. Producers can call
`ObservationContext::declare_summary` with a bounded responsibility purpose. Completed fit,
shooting, study and receiver-readiness responsibilities publish compact current/cumulative
facts through this generic contract. They remain completed; their diagnostic copies retain
no native handle, scientific payload or cancellation ownership. The hub never interprets
workflow names to select them. A relationship always names its
exact operation: if its parent has been evicted, the parent is absent rather than
replaced by another root. Retained readers can still inspect their copied operation.
Read generation/reset and detail-loss fields before interpreting a recent delta;
metric publication alone does not imply a complete event history.

Each process publishes one atomic JSON checkpoint under `PSE_DIAGNOSTICS_DIR`, or
`$XDG_STATE_HOME/pse/diagnostics`, or `$HOME/.local/state/pse/diagnostics`. The process locator
contains its instance identity rather than PID alone. Version **4** includes coherent
producer records alongside summary purpose, eligibility deadlines and explicit
admission/expiry/pressure losses. It retains the actual
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

A durable-study wait is associated with the primary admitted for that dispatch.
If it drains or is known to have changed and no terminal inventory exists, execution
returns `StudyExecutionUnavailable` with the study identity. Inspect the retained
incomplete records and use explicit recovery; this is not a scientific terminal.
Unknown/transient owner observations continue waiting. An owner mismatch is not
proof of drain and does not release any resource. Normal cancellation still waits
for actual issued work and durable terminal inventory.

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
runs once per second and is limited to 1 MiB. Essential active identity/coverage is reserved
first, then compact responsibility summaries, then optional richer active/history and ordinary
completed detail. Omitted operations and compacted history remain explicit. These limits affect diagnostics only.
`PSE_DIAGNOSTICS=off` selects absent collection at the first process registry initialization.
Engine `ObservationPolicy::Off` separately leaves operator metrics uncollected.

Completed summary eligibility defaults to 60 seconds, independently of the recent-window
setting. `PSE_DIAGNOSTICS_SUMMARY_SECONDS=0` opts out; invalid settings report an optional
writer failure and refuse summary retention. Hub bytes and operation count bound admission.
Active admission can displace oldest terminal summaries; `summaries.refused`, `expired` and
`pressure_lost` report distinct losses. Expiry is applied on catalogue, read, snapshot,
admission and retained-byte observations, and by ordinary publication. An idle hub can retain
bounded charged bytes until its next interaction/drop; no reaper or physical expiry-latency
promise is implied. Copied external readers remain charged and become stale at the semantic
deadline. Passive checkpoint inspection applies that same deadline. Writer failure neither
extends retention nor makes science unready.

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
| `modeling.declaration_environments` | Actual declaration-environment constructions. Empty scalar member-index checks avoid this work; required enclosing declaration checks remain. |
| `modeling.member_expansions`, `modeling.member_memo_hits` | Actual source-member expansion and empty-probing completed root-memo reuse on an unchanged operation-local frontier. A hit does not certify global graph closure. |
| `compiler.bound_structures` | Full bound row/instance expansion, including original structural validation of invalid bounds. Valid warm lookup avoids expansion while retaining fresh domain/value checks. |
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
with actual eligible value or derivative demand rather than assuming amortization.

This opt-in `compiled-evaluator` feature uses the fixed Linux x86-64 scalar C++ profile
for eligible Value, selected First/Second partials, weighted Hessian and directional/action
arithmetic, with independent worker scratch and owned executable lifetime. Providers,
branch selection and ordered source/domain/derivative-eligibility guards remain with their
host owners. Unsupported regions retain their explicit supported route; First-only demands
keep unused Second products cold. It selects neither a SIMD/batched route nor a host-native
CPU target, enables neither fast math nor FP contraction, and does not interrupt an
individual compiled call; cancellation is checked at its surrounding boundaries. These
producer facts do not establish universal evaluator benefit or whole-model qualification.
The selected analytic/interpreted/compiled and assembled acceptance is recorded in Plan43b/43g.

For an explicitly selected Python/native compiled-evaluator build, use the repository
checkout environment and the feature forwarded through the runtime to the mathematical owner:

```bash
scripts/pse-env --resource-class compile --native=compiler,solver,klu,isolation,uno,petsc -- \
  .venv/bin/maturin develop --uv --profile dev --locked \
  --features force-validate,native-solvers,compiled-evaluator
```

Ordinary `py-sync` builds do not select this optional mathematical feature. Select
`EvaluatorCompiledScalar(module_bytes=...)` in the preparation compiler's optimization
settings with a finite prepaid module allowance. A successful build only makes the route
available: actual compilation/evaluation counters identify its use by an eligible request.

## Artifact publication work

`artifact.metadata.attach` measures each bounded transaction attaching already sealed artifact file metadata. `canonical.artifact.metadata.groups_acknowledged` and `canonical.artifact.metadata.rows_acknowledged` count only groups and rows whose exact ordered key inventory was received and validated. File seals remain independent and differently shaped IPC data stays in separate files. A lost ACK remains unsettled and cannot authorize activation; these counters are neither a complete query count nor evidence of elapsed improvement.

## Typed inspection and reader ownership

For explicitly admitted compact and historical Plan39 simulation cases, the collector offers
`just case-measure <fresh-output> --smoke --audit-diagnostics --case <id>`.
This opt-in route runs one complete case observation with the original assertions,
retains typed stage/live/terminal inspection documents under each case's `diagnostics/`,
and captures a bounded CPU/timeline window when an operation remains active long enough
to inspect. It supports only the explicitly wired selectors in `scripts/case_measure.py`;
case sizes and identities remain declared in `.config/process-cases.json`.
Add `--audit-deadline-seconds 60` to request cooperative stop after that observation
interval. This is not a hard execution timeout: observation continues until the actual
owner exits, retaining native joins, cleanup and the stop reason separately.
The [short pipeline review](../design_review/reviews/design_review_pipeline-performance-optimization_2026-10-17.md)
describes compact cases and their interpretation limits; [39d D4](../plans/39d-qualification-and-plan38-cutover.md#d4--development-context-prerequisites-and-once-only-simulation-audit)
retains the original campaign's scope.
The selected reference store must be started inside its existing reference allocation
owner and drained before that owner exits.

Managed readiness carries the actual receiver's diagnostic identity and exact checkpoint,
under its exact registered receiver state, database and registration nonce. The actual readiness consumer retains a compact
copied handoff, so a short receiver can drain before the observer first polls. A checkpoint
may still be pending, disabled or have an optional writer failure; none of those states
changes scientific readiness. `diagnostics.read_checkpoint(path, expected=identity)` validates
the decoded image, and `diagnostics.inspect_image(image, request, checkpoint=path)` inspects
that same image. Copy the decoded image rather than reopening its path after validation.
A cached association can inspect the last published prefix after process exit without `/proc`;
it is not proof of final science, callback delivery or native drain, and never stop authority.

Managed observation selects the registered receiver and rotates its bounded active-operation
subset across successive reads. Bounded discovery continues catalogue pagination between
reads, and terminal navigation includes completed related owners within its existing bound.
A single window does not establish coverage of every lane or completed owner; catalogue
partiality, evicted records and unavailable receiver suffixes remain explicit.

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
        version=2,
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
            version=2,
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

Implicit derivative observation separates the actual `implicit.derivatives.jet`, `.extraction`,
`.first_rhs`, `.second_rhs`, `.contraction`, `.verification` and `.expansion` work under the
existing implicit owner. Each performed stage reports calls, completion/failure and elapsed time;
opaque factor/solve calls are checked before and after. Cached products do not fabricate repeated
construction stages. Parent and child times overlap, so these values are not additive partitions.

## Explicit numerical diagnostic operations

Dense `diagnose`, native `diagnose_linear`/`diagnose_jacobian` and finite `diagnose_samples` share
one caller clock across declaration, preparation, shared-product waiting, numerical admission,
analysis and initial response export. Python starts it before preparation using the unchanged
`SolveSettings.controls.time_limit`; sample controls can narrow that same allowance. Rust callers
create a `workflow::DiagnosticOperation` before preparation and pass its driver to preparation and
the operation to analysis. Reusing the operation across samples does not renew its deadline.
Leaving a shared preparation wait does not cancel another caller's producer. Owned numerical work
is cancelled and joined on expiry; opaque library calls may return later, but cannot publish a late
successful result. Term-combination analysis has the same scoped cancellation and admission.

Each endpoint accepts an optional typed `origin`, normally `result.diagnostic_origin(step=...)`.
This copies the original run, durable attempt, native attempt, diagnostic operation and retained
failure without reevaluation. Whole-run lookup includes authored refusals; a selected accepted step
does not inherit a later step's failure. Diagnostic results and generated Arrow exports retain that
context (`origin_json` in version 2 relations). A standalone diagnostic keeps `origin=None`.
Origin links, raw native products and partial trajectories do not grant scientific permission;
joined availability follows each product's governing original completion.

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
Checkpoint-attached CPU capture requires the selected operation to be active in its published checkpoint.
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
If a successful callchain decode produces no frame tokens, the same bounded decoder checks
sampled leaf instruction pointers. Useful leaf names establish that narrower symbol capability;
they do not establish complete unwinding. The original perf recording and Samply profile retain
their independent evidence and limitations.
Kernel sampling loss remains unknown unless quantified by the adapter.

For short commands, `pse.diagnostics.launch_capture(LaunchCaptureRequest(...))` returns
separate `LaunchedCommand` and `CaptureSession` owners. Perf starts disabled and must acknowledge
enablement before a fresh exec gate releases the scientific executable, so imports and model
preparation can be sampled. Arguments are a literal argv vector; `cwd` is explicit. If profiler
startup fails or exceeds `startup_timeout`, the command still starts and the capture reports its
independent failure. Stopping or dropping the capture never stops the command. Explicit command
`cancel()`/`close()` and command-owner drop stop its owned process group; `wait()` drains and reaps it.

```bash
just diagnostics capture-launch --output /tmp/pse-launch-new --duration 10 -- .venv/bin/python run_case.py
```

The CLI preserves the command exit/signal outcome; a capture failure makes an otherwise successful
command return failure. A launched `CaptureDocument` version 2 initially has `operation=None` and
an exact `launch_process`. Bounded discovery can add actual published `related_operations`, clock
and thread facts later; it cannot invent operation-exclusive CPU or preceding history. The derived
capture view uses a nonnullable process/operation `subject` and nullable operation/launch columns.

`CaptureDocument` keeps requested versus observed duration, targets and clock uncertainty,
output prefixes, symbol evidence, open intervals, losses and errors separate from scientific
outcomes. Output exhaustion, refused events, source loss and unknown clocks remain visible.
Neither a completed capture nor a readable file establishes numerical correctness or a root
cause. Tokio Console, allocation/native-heap and off-CPU/wait integrations are not supported
by this capture surface; their suggested next observations remain optional and unavailable,
not zero work. Current selected acceptance belongs to the Plan 43 Outcomes.


## Coherent source records and selected detail

Performing owners have independent producer IDs and prepared metric populations. Read a
producer's whole `current` record and bounded `recent` records rather than joining independent
field histories into iterations. Records retain source generation/sequence, family-sequence
availability, timestamp, phase, coordinates/norm and typed fields/availability. Current
native progress is also available from `RunHandle.current_progress()` with history zero;
its separately read issued head/loss makes stale or refused current retention visible.
Durable append sequence is a different identity.

`runtime.observations().collect(DetailRequest(...))` explicitly selects finite families,
window, stride and memory/output allowances before constructing work in its synchronous
context. Native families include `nonlinear_iteration.v1`, `root_evaluation.v1`,
`root_raw_map.v1`, `conic_iteration.v1`, `linear_search.v1`, `branch_and_bound.v1`,
`dynamic_output.v1`, `dynamic_adjoint.v1`, `dynamic_native_return.v1` and
`native_owner_return.v1`; only supported actual owners emit them. Workflow integration and
evaluator census/demand families retain their own semantics. Missing families stay incomplete.
`boundary_thread_cpu=True` selects actual synchronous callback-thread CPU, never async,
child-thread or exclusive CPU. No thread clock is enabled by ordinary collection.

The entered Python construction scope must not cross an await. After starting work, call
`collection.leave()` before yielding; constructed native/async owners retain the selection.
Keep the collection alive until work joins, then `finish()` (or normal context exit) joins
only its writer. `stop()`/exception exit retains incomplete coverage. Dropping a collection
never cancels science and releases Python execution while joining its writer.
The first closing reason is retained: later `finish()` or deadline expiry during drain
cannot relabel an earlier stop as a complete interval.

`read_detail` returns the versioned coverage/prefix; `detail_records` pages original whole
records with a byte cursor. Complete requires every requested family to be observed and
source-issued heads, offered records, queue/output acknowledgments and final drain to agree.
Loss, refusal, interruption, an unknown family or hard-death prefix is incomplete. Selected
sampling is labelled sampled, even when all chosen samples were flushed.

For managed studies, supply a separate `managed_budget` before starting the recorded study.
It is granted once and divided across declared attempts including retries; the local
allowance is not duplicated. Queued children retain the original same-host deadline.
`StudyHandle.diagnostic_selection()` reads bounded current assigned attempt references,
actual receiving PID/operation and each child's independent coverage. A local capture
never proves whole-study completeness; historical retry files are retained without being
substituted for current attempts.
`StudyReport.diagnostic_selection()` uses the report's retained canonical study locator
for the same readback; an ephemeral in-process report returns no managed selection.

`RunResult.invocation_evidence` references existing attempt/work/settings evidence and
separates qualification completion, scientific terminal completion, response readiness,
requested-product availability and later related access. Inclusive intervals overlap.
Version 2 starts its wall clock when the invocation recorder is established inside the
entered activity owner, before native dispatch; earlier preparation is excluded. Its
`native.worker_execution` interval is actual inclusive worker time, not exclusive CPU.
Version-1 supervisor-only documents do not provide this expanded comparison coverage.
Selected fitting/shooting integration records expose `forward_direction_columns` as a
physical request-layout count, with explicit NotRequested availability for other products;
it is not an executed native-vector count. Standalone trajectories retain `forward_axes`.
The native-process harness retains every invocation, including warmups/failures, separately
from Criterion statistics; measurement membership is unavailable when the harness does not
establish it. These records do not introduce performance thresholds or provenance proof.

## Selected numerical interpretation and retained evidence

A whole coherent record identifies one producer event. Read `family_sequence`, generation,
phase, coordinates, norm, availability and history coverage before comparing events.
KLU reports actual issued/completed setup, symbolic, factor/refactor and solve work. Selected
PETSc logging has one prepared producer population per native event and a common checkpoint
ordinal; event time is completed inclusive time, not exclusive wall or accepted iterations.
A zero-history request can retain current progress. Disabled/refused/absent observations are
unavailable rather than measured zeros.

`InspectionRequest` version2 optionally accepts `NumericalHistoryPolicy`. Thresholds are
explicit caller choices; interpretation requires a sufficient compatible record suffix.
The CLI accepts `just diagnostics inspect ... --numerical-policy <json>` with the same generated
policy. Restoration, tiny directions, barrier changes, original worst members, inertia and
raw recycle ratios produce source-linked facts, hypotheses or limitations. An initial zero
placeholder is not a tiny step; a scaled search direction is not accepted physical motion.
Coalesced endpoints and opaque intervals cannot establish stagnation or waiting.

`RunResult.invocation_evidence()` preserves actual products, starts, attempts, work and
independently observed qualification/terminal/response endpoints. `explain_invocation()` and
`compare_invocation(other)` consume those documents without executing either model. Their
optional generated `ComparisonPolicy` bounds inspected source items and returned statements.
Comparisons distinguish more work from descriptive cost per unit and retain changed requests,
failures, retries, coverage and overlapping inclusive clocks. They do not rank incompatible
products or establish a causal performance verdict from timing alone.

`ModelingPackage.diagnostic_source_map(case_id, settings, maximum_bytes=..., time_limit=...)`
reads a bounded static source/product manifest. It retains document fields and repeated
occurrence positions, aliases, gathers, fixed coordinates, shared arithmetic contributors and
scales. Static eligible branch alternatives do not say which branch executed. Unavailable
native pullbacks remain explicit. One caller clock covers Python declaration/preparation, artifact waits and mapping/copying;
zero bounds refuse before preparation, and a narrower enclosing clock cannot be renewed.
The operation prepares source maps but performs no residual,
property or derivative evaluation. Explicit full bounded singular-mode diagnostics separately
perform new numerical work and publish operator residual/normalization qualification; they do
not certify global feasibility or optimum.

`diagnostics.detail_arrow(records, offset=..., limit=..., maximum_bytes=..., memory_bytes=...)`,
`diagnostics.capture_arrow(document, memory_bytes=...)` and `RunResult.invocation_arrow()`
project retained evidence into registry-declared Arrow schemas. Rust supplies the same views
and a DataFusion `MemTable` over selected detail. Escaped buffers retain their finite allocation
lease. Count values remain UInt64; exact time is seconds plus nanoseconds, unavailable is null
with a reason/category, and missing prefixes/gaps remain partial. Repeated source identities
refuse a derived view rather than silently deduplicate raw records. Invocation/capture JSON
preserves full source meaning alongside convenient columns. No store writes or hidden scientific
evaluation are needed, and optional export failure leaves the source and scientific result intact.

## Optional trace queries and profiler selection

`just diagnostic-tools` explicitly installs the pinned optional Trace Processor. It never runs
implicitly during capture or query. `just trace-query --help` describes the finite query adapter;
select `just trace-query query <trace> <timeline|numerical|coverage> --output <fresh-directory>`.
Raw trace/capture documents remain the authority even when import, query, output or timeout
fails. Timeline slices are inclusive and may overlap. Coverage SQL reports importer errors/loss;
an empty result is not proof that the source collected every event.

The writer preserves actual MONOTONIC timestamps and emits MONOTONIC/BOOTTIME relationships
with the measured sampling bracket before events, refreshed at source checkpoints. It does
not relabel one clock as another. Source records use typed protobuf annotations; Trace Processor
exposes them as `debug.*` arguments. Its SQL signed representation can show a large unsigned
value as negative: use exact `display_value` for unsigned identities, or the direct UInt64 Arrow
view for numerical analysis. Unknown source-clock alignment remains unknown.

Use existing owned perf CPU capture and optional Samply rendering for a concrete hot-stack
question; retain raw samples, loss, unwind/symbol and process/thread scope. User cycles and
instructions do not imply raw/ftrace/uncore permission or prove bandwidth saturation. Heaptrack
and Callgrind are eligible isolated experiments; a launch-injected profiler tied to the simulation
is not an independently stoppable capture. BPF/off-CPU, Coz, Tracy, Tokio Console, SUNProfiler,
Caliper and uProf require their actual question, build, permission and lifetime; no compulsory
profiler stack or permanent daemon is installed. Compiler remarks/llvm-mca apply to a confirmed
arithmetic region, not the whole workflow. Retain conservative current ISA/BLAS/allocator/build
profiles unless independent scientific qualification supports a concrete change.
