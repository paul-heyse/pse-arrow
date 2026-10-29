---
title: Simulation diagnostics and explicit capture
status: active
doc_owner: docs/authoritative_design/sections/operations-and-validation.md
doc_topics: [environment, validation]
---

# Simulation diagnostics and explicit capture

Rust diagnostic readers copy observations from the actual performing owners. Use
`Runtime::observations()`, `RunHandle::observations()` and retained surrogate observations;
these readers cannot cancel work, keep a solver alive, admit a candidate or open provisional
scientific results. Blueprint §23.1 owns the contract. The current source surface includes
native inspection, bounded capture, frozen analysis and review export. Whole-system and H
qualification remain with [Plan58](../plans/58-rust-owned-system.md).

An operation identity exists before preparation. Read its phases, source references,
relationships, timestamps and coverage before interpreting a metric. Inclusive nested intervals
do not sum to exclusive elapsed or CPU time. Shared-flight waiters identify one producer;
waiter count is not repeated construction work. Stale checkpoints and opaque native intervals
alone do not establish a hang.

Each process publishes an atomic checkpoint selected by `PSE_DIAGNOSTICS_DIR`, then the XDG
state directory, then `$HOME/.local/state/pse/diagnostics`. `Diagnostics::checkpoint_path()`
returns its exact path; `pse_runtime::observation::read_checkpoint` reads an already published
image. The locator records process instance, actual PID/start ticks where available, executable,
clock mapping and bounded actual native thread scopes. PID alone is not attachment identity.
Publication and producer times differ; cross-process reads are not atomic. Missing alignment,
thread identity, detail or source coverage remains explicitly unavailable.

Current phases retire independently of optional detail publication. Safe-return callback facts
form coherent attempted/completed/time/failure tuples; in-flight work remains an active phase,
not a fabricated final tuple. Point history preserves observed timestamps, first/latest values,
coalescing, reset generations and losses. A predecessor outside a requested window is not an
inside-window sample. An unavailable/nonfinite sample never becomes zero.

## Preparation work and waiting

Compiler specialization, typed lowering, materialization, provider acquisition and preparation
retain separate owner phases. Request wait overlaps producer work; a supervisor join overlaps
native execution. Discovery reports resident metadata and eligibility without compiling,
transferring providers, refreshing cache recency or reopening portable bodies. A source path or
observed prepared product is not a current execution capability.

Counter resets, cache owner epochs and relation/source identities delimit comparisons. Current
and cumulative summaries can remain after completion without retaining scientific payloads or
cancellation handles. A relationship names the exact operation even if its parent has expired.

## Native work and reuse

Tokio runtime registration reports supported worker/task/injection-queue/busy observations;
local queues and unregistered executors remain outside coverage. Busy duration is elapsed poll
activity, not process CPU. Selected future monitors have class-wide nesting and wake/poll scopes;
their totals cannot be added into exclusive native time. Rayon reservation and pool observations
have their own meanings. No queue, completed future or dropped observer proves native drain.

Original process, thread and native attempt identities bound diagnostic populations. Continuation,
fitting, horizon and study observations retain their separate attempted/committed/failure facts.
Unavailable sensitivities, Hessians, responses or covariance do not invalidate independently
available quantities. Reuse observations explain the owner's actual admitted source cut; they
do not authorize seeds, transfer native handles or measure a compiler cache hit by inference.

## Artifact publication work

Publication, protection, file synchronization and serving are separate phases from numerical
execution. Canonical run, attempt, occurrence and manifest identities remain distinct. A copied
checkpoint never substitutes for an acknowledged terminal or payload custody. See
[the substrate guide](surreal-substrate.md#payload-placement-and-owner-recovery).

## Typed inspection and reader ownership

`pse_runtime::tooling::inspection` exposes `InspectionRequest`, `InspectionDocument`,
`list_processes`, `inspect_checkpoint`, `inspect_image` and snapshot inspection. Select an exact
operation/checkpoint and bounded interval. These return copied documents and explicit coverage;
process discovery is not authority to attach or stop a process. Retaining a document does not
retain an executor, numerical result or source-owner lifetime.

Scientific inspection belongs to `ModelingPackage` and the native Runtime/result interfaces,
including `source_tables`, `run_record`, `attempt_record`, `result_manifest` and result readers.
Keep exact source, run and attempt keys. Arrow arrays carry their allocation owner after the
reader is dropped. Diagnostic detail tables use their declared registry schemas, units and
native integer coordinates. A user-facing display may round a value; the typed table remains
authoritative. Explicit complete source export materializes the selected package inventory,
not the whole database.

## Standalone reads and the CLI

There is no Python diagnostics facade, `just diagnostics`, generic inspection CLI or active
debugger proxy. Rust callers use the typed APIs directly. Native process observation is
`just activity [UNIT]`; optional installed-service inspection is `just telemetry inspect`.
Help and passive reads do not build tools or initialize a scientific/canonical context.

Generated JSON schemas describe standalone native documents; bounded versioned decoding refuses
unknown/incompatible versions, malformed inputs and exceeded extents. `tooling::frozen_inspection`
reads supplied documents with aggregate bounds and cancellation. It does not discover additional
original paths, open a database or repair missing evidence. Missing supplied members remain missing.

## Explicit numerical diagnostic operations

The [native workflow guide](native-workflow.md#source-modeling-operations) describes structural,
Jacobian, linear and elastic diagnostic requests. These are explicit mathematical operations,
with actual provider and admission requirements. A local Jacobian finding is local to its
point/scaling; positive nonlinear elastic slack is not global infeasibility. Diagnostic candidate
points do not change fixed/parameter values or grant solve admission. Native original reports
retain their own availability and interpretation, independently of live observation detail.

## Explicit bounded capture

`pse_runtime::tooling::capture::start_capture(CaptureRequest)` starts an explicitly selected
session over an exact checkpoint/operation and a new exclusive output directory. The request
selects kind, duration and `CaptureBudget`. `CaptureSession` owns collectors/exporters only;
`request_stop`, `stop` and `wait` retain actual collector settlement. Dropping an observer does
not cancel scientific work. Passive inspection never creates a capture.

The resulting `CaptureDocument` separates target identity, collector outcome, retained files,
readable prefixes, clock alignment, thread coverage and losses. Readable output can be partial;
file creation or a successful decoder is not complete capture. CPU samples describe inclusive
actual processes/threads and may include concurrent operations. Attachment rechecks start
identity. Native perf permissions, missing tools, unavailable symbols and unsupported clocks
remain explicit. Truncation preserves the complete readable prefix and names omitted detail.

`launch_capture(LaunchCaptureRequest)` owns a literal command launch and its bounded collection.
It is distinct from attaching a passive reader. Original command exit, collector outcome and
owned child/group drain are separate. A collector failure cannot silently rewrite science;
cancellation of an owned launch still requires its explicit settlement contract.

## Coherent source records and selected detail

Source-recorded numerical events retain actual producers, versions, operation/attempt identities,
coordinates, reset epochs and source-family coverage. Complete and sampled requests have
different claims. Budget/expiry/pressure losses remain visible; no event or a curtailed page is
not evidence of zero numerical work. Native counters retain full UInt64 precision. Text/JSON
copies cannot redefine the typed relation's schema or units.

Original detail is optional evidence. Refusal, expiry or capture loss never changes solver
admission, result availability or native cancellation. Read detail within its declared coverage;
do not combine separately partial sources into a synthetic complete history.

## Selected numerical interpretation and retained evidence

`tooling::review_analysis::analyze` interprets explicitly supplied `FrozenEvidenceSet` members
under an `AnalysisRequest`. It retains exact event/source identities, question selection,
unknowns and limitations. It does not rerun the solver or call original native getters.
`inspection_comparison::explain_invocation` and `compare_invocations` explain supplied invocation
facts without substituting a later mutable run. Comparison conditions and unknown applicability
remain visible. Observational differences alone are not causal performance evidence.

## Optional trace queries and profiler selection

Install optional tools explicitly with `just diagnostic-tools trace-processor` or `rr`;
`.config/diagnostic-tools.toml` owns exact pins. Installation does no capture/query. Verification
requires actual pinned binary/archive provenance; it does not trust arbitrary existing tools.
Perfetto consumes finite exported traces independently of scientific execution.

`tooling::review_query::specification(&ReviewQueryRequest)` produces maintained timeline,
numerical or coverage SQL and its declared columns. It performs no I/O and installs/runs no tool.
The caller selects the supplied input member, source clock, half-open nanosecond interval,
finite timeout and output extent. The Python TraceProcessor execution wrapper is retired;
an external consumer must own actual execution, tool version, bounded output, cancellation,
join and schema validation. No generic SQL execution adapter is implied by this pure API.
Perfetto's signed integers cannot represent every native UInt64. Successful empty output,
unavailable tool, malformed output, timeout, cancellation and output-limit outcomes remain
distinct in the retained query documents.

### Escalating a selected telemetry observation

Ordinary telemetry exports coarse owner spans and selected metrics/logs. Rust export selects
exact `pse.owner.stage` metadata and the canonical request span, rather than a name prefix.
A suspicious coarse observation can motivate explicit native inspection/capture with the
actual target identity. Telemetry is not an operation catalogue or scientific permission.
Empty backend selections cannot distinguish nonexecution, sampling, loss or retention expiry.

### Optional mechanism fit, assessed 2026-10-24

Choose a profiler for the stated question: CPU sampling, allocation traffic, scheduling and
record/replay have different populations and interference. Native tools may be used directly
under an explicitly owned launch/attachment; the retired Python proxy supplies no safety or
settlement adapter. rr changes execution and is unsuitable for parallel timing claims.
Optional third-party Python profilers live in isolated tool environments, without a product SDK.

## Recorded analysis, cooperating recording and offline review

### Explicit active diagnostics

The former GDB/heaptrack/scheduler/Python-stack command proxy is retired. Direct external tools
retain their native launch/core/attach interfaces and require explicit owner authorization and
physical settlement. An operation ID supplies attribution, not attachment authority. Exact
PID/start identity, symbols, unsupported frames and shared-process interference bound a finding.
There is no claim of equivalent native active-debugger orchestration.

### Storage health and declared evidence links

Native canonical transport and store readers expose selected operational observations. Source
receipts and declared evidence links retain original identity/digest and interpretation; they
do not authorize arbitrary filesystem discovery, payload borrowing or cleanup. Service liveness,
SQL finality, schema admission and scientific success are separate. See the
[substrate guide](surreal-substrate.md) for the actual Rust owners.

### Cooperative inspection of the actual executor

`tooling::taskdump` accepts an explicit request through the caller's actual registered executor.
It requires the diagnostic-taskdump feature and its `tokio_unstable` build contract. The native
correctness adapter isolates that profile. There is no substitute Tokio runtime created just
to observe another. The collector waits outside the target executor; collector wait expiry does
not cancel Tokio's separately issued request. Read target state and observer expiry separately.
A returned/truncated task trace is diagnostic evidence, not native scientific drain.

### Preparation comparison and executable reproducer extraction

Review analysis compares supplied source/preparation/native records within original scopes.
A retained reference is descriptive unless its actual immutable member is supplied. Export
copies chosen members; it does not reconstruct a model, restore a provider, execute a reproducer
or fetch omitted source. Reproducer/campaign drivers retired with the Python controllers have
explicit H handoffs rather than an invented replacement CLI.

### Explicit preconstruction source recording

`capture::SourceRecordingRequest` selects versioned detail families and an exclusive
acknowledgement path. A cooperating Rust root consumes `PSE_DIAGNOSTIC_RECORD_REQUEST` through
`source_recording_from_environment` before constructing scientific work. It declares actual
supported families and finite local/managed grants; successful ordinary command exit is not a
source-selection acknowledgement. Complete/sampled requirements, stride, duration and losses
retain their own semantics.

`tooling::review_launch::record_launch` executes the literal cooperating request and records
independent command exit, timeout/cancellation, acknowledgement, bounded logs and physical group
drain. Source ACK and review handoff can be incomplete independently of command success.
Selected review context must be retained while the original caller owns source/run/attempt
facts; no later path lookup reconstructs it. `capture::read_review_handoff` reads only listed
finalized contained members with the frozen aggregate bound. Optional recording/handoff failure
never replaces the native scientific result.

### Contextual queries and typed tables

`review_pack::PackSession`, `PackReader` and `review_export` build/read a finite selected review
package from supplied members, explicit finalized files and typed tables. Index publication,
unavailable/failed members and prefix losses remain explicit. Protected buffers retain their
allocation lease. The package can be relocated without following original source references;
its source labels are descriptive, not current authority. Exported IPC is the declared stream
format with schema metadata, not an inferred table from arbitrary JSON/CSV.

### Explicit original-source numerical diagnostics

Use the original package/request/provider under its native numerical diagnostic owner.
Offline review does not grant original source or execution capabilities. Retained native
segment statistics preserve their reset scope and original joined getters; sums across
reinitializations do not automatically form a coherent cumulative population. Source checks,
solver convergence and diagnostic interpretation remain separate outcomes.

### Short external-review campaigns

A short diagnostic campaign must name its question, actual selected input, bounds, tool/source
conditions and observed failures/losses. The old case-measure/audit and review CLI drivers are
retired; H owns their required scenario handoffs. Native capture/review controls establish their
selected contracts only, not complete diagnostic coverage, whole-product qualification or a
performance improvement. Keep original failures and interrupted evidence alongside any later pass.
