---
title: Operations and validation
status: current
---

# Operations and validation

This area covers how the running system reports what it did and why it failed, and how
the repository establishes that the implementation behaves as its contracts say.
`pse-diagnostics` owns the failure vocabulary; `pse-columnar::engine` classifies native
engine errors; `pse-engine::session` owns engine observation policy; runtime result
relations carry execution facts. Native tests live in the owning crates and `tests/`, isolated
reference exchanges under `tests/reference`, and benchmarks under `benches/`; `just --list` is the
command surface.

## 23. Observability and failure semantics

Observability explains execution; provenance explains derivation; results state
scientific meaning. They share identifiers but are never merged, and observing a
computation can never change its scientific or publication outcome
([ADR-0106](../../adr/0106-execution-vocabulary-discrete-and-global.md)).

### 23.1 Observability

> Decision: [ADR-0215](../../adr/0215-resolved-library-execution.md) — Proposed, maintainer-authorized Plan49 target; implementation/qualification remains at the owning plan.

Product detail and protected result-selection contracts precede internal tooling extraction. Rust owns conformance manifests/defaults and cooperating-source launch execution; native developer adapters consume those owners. Optional tracing/crate selection requires its own bounded outcome; existing observation authority remains until replacement qualifies.

> Decision: ADR-0192/0196 (proposed; accepted Plan47 RC01/RC02 target). Live inspection
> observes current time and known local publication. Frozen inspection consumes supplied
> immutable documents and a recorded endpoint/lookback or explicit source-clock interval;
> it never consults the reader clock, neighbouring files, a process or the scientific store.
> Related membership is explicit and bounded, with retained historical attempts distinct
> from current pointers. Missing members, capture loss, resets and unsupported clock
> alignment remain limitations, not reconstructed observations. Recorded freshness does not
> renew live eligibility. Versioned bounded ingress supplies the same pure interpretation
> used by live owners without reconstructing numerical or allocation authority. Descriptive
> context and offline completion do not establish executable replay or scientific success.

Frozen native ingress accepts versioned copied checkpoint, invocation, context,
membership and capture documents with finite input bounds. Saved invocation explanation
and comparison use the live owners' pure interpreter, validating exact step/product/native
coordinates before matching. Related operations resolve only from supplied members;
independent process clocks remain independent. A selected earlier interval filters retained
history and predecessor baselines, but does not reconstruct earlier current/cumulative state.
Live compact attempt payloads retain their allocation guards even through shared document
references; decoded facts do not recreate those guards or native execution authority.

Standalone initialization and declared-recycle strategies can explicitly select descriptive
context at their original join. Their actual run/operation, original source, submitted controls,
stages/attempts and retained native records remain distinct from a RunResult invocation.
Unrequested context is not copied; a selected copy refusal does not alter the scientific result.
Escaped context owns its finite byte allowance and retains no executable or native handle.
Selected dynamic context also copies the original joined native segment statistics, with
original run identity and segment/reset boundaries. These retained facts remain available when
the compact live metric catalogue reports curtailment; export performs no fresh native queries.

Selected review analysis consumes copied immutable inputs through one Rust interpreter. Its
versioned ten questions retain exact member/event/field references, actual populations, units,
source clocks, missing coverage and bounded omissions. Inclusive time is not exclusive CPU,
overlap is not a critical path, and missing physical pullbacks remain unavailable. Engine
Contract/Diagnostic observations are copied from the sessions that actually performed selected
physical admission, including failure/drop, without enabling instrumentation. Off remains an
explicit empty-policy fact. Copy extents are admitted before materialization; descriptive source bounds count only
selected original objects, provider ABI descriptors and physical streams, excluding parsed
or runtime inventories from the broader retained-owner estimate. Replay admission is
unchanged. Retained copies hold no
session/native handles. Counting physical IPC uses a finite sink before allocating the
copy; this allowance does not establish Arrow writer scratch or total private peak. Interpreted context headers agree with original run/step/source
identities; exact retained maps can explain original members, affine slots, occurrences and source
terms without inventing eliminated/native pullbacks. Analysis preparation and escaped results
share finite allocation ownership and cooperative cancellation. Independent process selections
retain their own intervals; parent-clock numbers never filter an unrelated clock population.

Selected source recording is explicitly accepted before construction. Applicable producer/family
activation is independent of issued work: acknowledged zero work may be complete; unsupported,
unobserved, sampled, refused and lost sources remain distinct. DetailRequest remains v1;
source-recording request/acknowledgement and launch-result envelopes are v2, selected detail
headers/documents/pages and supplied frozen sets are v2. Optional producer handoff is separately
admitted before construction and publishes only original joined facts or retained pre-handle
context under its selected byte/member/expiry limits. It has its own completion receipt and
retains no extra context when absent. Data-only handoff readback obeys frozen aggregate bounds. Earlier
output codecs are refused rather than interpreted through a compatibility path. A cooperating
command accepts a request file and publishes an actual operation/process acknowledgement;
CPU capture of arbitrary argv cannot establish source recording. Closing a recorder drains its
writer and never cancels science; command cancellation owns its exact child separately.

Offline review publication exclusively creates its destination, publishes an incomplete index,
finalizes selected members under finite allowances, and seals its index only after required
copies and any selected reports drain. Supplied non-scientific member publication uses the same
index, sealing and bounded readers without fabricating a scientific checkpoint, analysis or report.
Membership, explanatory context, capture availability and scientific
outcome are independent. Selected protected results use their actual reader/export owners and
release each borrower before opening the next reader. A shared composition serves CLI,
Runtime and single-result export: explicit supplied/live checkpoint and optional original handoff,
exact durable attempt/member/range/output predicates, typed diagnostic IPC, then sealing after
drain. Pure supplied publication opens no deployment; protected rows require an explicit existing
deployment. Analysis and pack request/selection/index/document envelopes are v2. Explanatory
closure follows selected questions/products and supported actual source pointers, independently
of publication success; unrelated optional missing facts do not block a narrow question. Diagnostic
IPC retains its registry/source contract separately from scientific lineage. Table source
comparison selects the requested original member before encoding it; unrelated attachments do
not enter that comparison. Finalized frozen ingress still has its own finite document allowance,
separate from the table projection pool; neither is a claim about total private process memory. Reports contain
bounded model/settings/table/profile orientation and clickable local source anchors. Reports and
local assets are ordinary offline files. Readers
follow only contained indexed member paths and readable prefixes, refusing identity/version/
format/path escapes without consulting live neighbours or original absolute locations.

Declared query products retain exact SQL/parameters, input/capture and clock/window, output
schema/units, actual tool version, status and partial/error prefixes. Half-open interval clipping,
inclusive sums and union coverage have distinct meanings. Missing tools and failed/truncated
queries never become empty success. Optional profiling remains question-led; stock Samply
perf-import portable presymbolication is not adopted because its accepted flag is unused on
that path. Additional platforms/derivatives require a concrete unanswered question. Explicit
numerical diagnostics use admitted original source and a caller deadline established before
preparation; passive inspection and source-following perform no new scientific evaluation.

Selected IDAS `dynamic_local_error.v1` reports state-only weighted local-error estimates
after eligible fresh successful native returns. Internal time/step/order and returned sample
time differ; algebraic suppression follows the native full-state-denominator WRMS convention.
Bounded contributors retain omissions and nonfinite/availability facts. Failed, repeated,
uninitialized and reset returns cannot reuse a stale estimate. Selection owns its optional
scratch vectors and private SUNDIALS context; it changes no tolerances or scientific masks.
POUNCE `native_factor.v2` and `native_refinement.v2` retain actual linear-call parentage and
bounded multipart group coordinates. Every part is required before treating a group as
complete. Factor snapshots describe the actual scaling/order/front/perturbation path; inertia
under static perturbation describes the perturbed matrix. Actual refinement/RHS work and
MC64 numeric retry/fallback deltas are narrower populations than all possible native inner
work. `native_linear_call.v1` supplies issued/completed nested calls with inclusive boundaries,
not exclusive CPU cost. Uno accepted and termination-trial checkpoint ordinals remain
distinct from generic iteration numbers. Optional capture failure changes no numerical outcome.


**Paired evidence.** Finite baseline/candidate comparison is implemented in the existing native
evidence-analysis owner (ADR-0225 remains a proposed decision record). Exact compatible owner
identity or supplied mapping governs correspondence; unmatched, ambiguous, mixed and unexecuted
subjects remain explicit. Source cuts, full denominators and independently selected sections
survive native, indexed, offline and canonical consumers. Measurement context incompatibility and
missing associations remain inconclusive rather than being repaired by aggregation. The headless
report consumes the same original document and never changes scientific or qualification outcomes.


**Shared diagnostic telemetry.** Maintainer-selected Plan52 OpenTelemetry RC01/RC02 add a
persistent native Collector and local OpenObserve projection alongside finite capture. The process
host composes providers, resource identity and shutdown once; constructing a Runtime never installs
a provider. Each native process retains its actual incarnation and separate instrumentation scopes. Child
processes have their own incarnation. Bounded W3C parent context
crosses explicit host/task/thread boundaries independently of scientific request documents, hashes,
operation handles and native parent locators. A shared ID is association, not a propagated edge.

SDK batching and standard Collector queues/retry own optional signal delivery. Ordinary selected
telemetry exports coarse owner stages and safe structured diagnostics, with metric populations
remaining defined at their performing owners. Export failure changes neither native admission nor
execution, drain, scientific assurance or publication. Long-lived native executable
hosts settle native owners before explicit provider shutdown. The private one-shot native lane
keeps its providers for process lifetime without synchronous network drain on scientific exit;
its final batches, or all spans from a short solve, may be absent. Native result, child join and
reap/deadline contracts remain unchanged. A successful flush/processed count does not
prove receipt; ingest acceptance, query visibility and crash survival remain separate claims.
Passive effective-configuration and pipeline-health reads start no service or scientific work and
retain unavailable, unsupported, lost, sampled, expired and unknown observations.

Owner-keyed metric instruments retain declared population, unit, monotonicity and histogram bucket
meaning through producer, schema and query. Gauge-rate and non-histogram percentile questions,
invalid buckets and unsupported native exemplars refuse before querying. Ordinary span export
selects the explicit `pse.owner.stage` marker and exact canonical request span; finite inner capture is
separate. Rust SDK 0.33.1 internal diagnostics expose first-drop observation during processor life
and a cumulative drop count at the SDK shutdown attempt, before its control-send/wait. That count
does not establish successful shutdown, queue drain, export or ingestion. Failed shutdown remains
independent; abrupt buffered loss is unknown, and capacity is not occupancy. The small delegating processor
observes SDK events without replacing its queue or mutating global meter configuration.

Rust serde types declare private host settings. The native host owns environment resolution;
immutable settings and native validation preserve that single selection. Responsibility-owned
schema publication does not require a scientific product import. Public effective
configuration is a separate redacted projection and exposes no private credential document.

Registered native executor owners supply library measurements to the existing typed process
checkpoints and optional OTel callbacks. Observers create no executor and retain no registration
token. Selected composition roots and fixed future classes define coverage; this is not a universal
executor census. Queue depth excludes worker-local queues, worker busy time is not process CPU,
and nested future poll/idle durations are not exclusive execution intervals. Executor exports are
gauges over the current registered population; selected-future counters are process cumulative.
Absent or refused observation sources are not zero work or failed scientific admission.

Copied `ExecutionContext` transfers observation, invocation recorder, tracing dispatcher/span,
W3C parent and available runtime handle across task/thread boundaries. It grants no execution,
connection, byte, population, deadline or cancellation authority. Continuations retain their
context; nested operations derive explicit children. Independent shared producers use a linked
relationship without borrowing the first waiter's recorder or scientific lifetime. Absent
attribution remains absent. An owner classifies completion; entering attribution alone cannot
report scientific success.

Owner-typed lifecycle boundaries feed the observation hub, operational counts, optional tracing
and bounded invocation recorder independently. Job, native-session and window-team populations
remain distinct: observing a session cannot increment native-job counts.
Sampling or absent subscribers does not remove owner state, and unavailable observation is not
zero work. Fine boundaries do not consume the recorder's 64 interval slots; only declared
intervals such as actual inclusive native-worker execution do, with its existing name bound.
These intervals are not exclusive CPU measurements.

Dedicated native jobs and sessions retain one OS-thread join owner through success, stop and
panic, including TLS/foreign destruction before the owner's joined notification. Caller-future
drop leaves the supervisor/join running. Neutral stop reasons carry cancellation/deadline intent
without depending on Tokio. Abandon-safe readiness races prefer an already-ready stop; issued
work drains the same retained future under its independent supervisor. Renewal uses the admitted
heartbeat with skipped missed ticks: durable execution preserves its immediate first renewal,
while a newly admitted study first renews after one heartbeat. Stop joins an in-flight renewal;
a renewal failure requests stop without proving scientific completion or releasing authority.
The neutral shared-flight and I/O-gate owners remain; the unused engine-specific flight wrapper
is removed. Sharing a producer never transfers a waiter's connection or allocation grant.

Native operational, preparation/reuse and canonical-query populations come from their performing
owners, including attempts/admissions/dispatch/join, shared producers/waiters, retained reuse and
actual statement outcomes. Version6 process checkpoints and cumulative SDK callbacks consume the
same sources. Rust-owned metric declarations generate the source-only query descriptors; host
build/assessment/stage and prepared pressure observations retain their original owners and clocks.
Dimensions are bounded categories, with invocation/operation identity in selected detail rather
than metric labels. Histogram boundaries, source epochs, missing observations and collection
coverage remain explicit. Sampling off does not disable owner metrics.

The optional Python FastMCP endpoint, global operation catalogue, generated capability CLI and
first-party frontend sessions are retired under proposed ADR-0233 and Plan58 G02. Direct Rust owners and
retained passive reports supply selected native operations and evidence routes; an eventual
agent adapter needs a concrete consumer and its own acceptance. The retained backend adapter owns signal-specific field
mapping, credentials, reusable asynchronous transport and service-generation fences. Cumulative
SQL points preserve original start/reset timestamps; rate/increase extrapolate, p95 interpolates
and threshold fractions require an actual declared bucket. Related exemplars establish pointers,
not exact population membership or causality. Query answers retain source/retrieval intervals,
lookback, limits, units and unavailable/partial states; live pagination is not a frozen snapshot.

Explicit publication copies finite selected answers and native references into the existing sealed
member-pack custody. Offline reads use only admitted copied members and digest-bound cursors,
without live backend queries or current-clock reinterpretation. Selected finite native capture
verifies target identity before and after capture, drains its own helper and never stops science.
Calling-process SDK state remains distinct from a recorded native target. Native documents are
preserved or refused as whole excessive detail; telemetry grants no scientific outcome, resource
release or cancellation authority.

Telemetry services have independently owned native user-service lifetimes and persistent state,
separate from scientific store/scratch and workload allocations. Explicit installation/setup/start/
stop/enablement select exact artifacts and configured generations. The operator selected login
startup and seven-day ordinary retention; finite exported evidence retains its existing custody.
Explicit reconfiguration stages and validates immutable generations before replacement, with
original-generation recovery and existing retention/credentials preserved. Collector internal metrics
and actual database OTLP metrics share the selected pipeline; database launch identity distinguishes
restart incarnations. Native OSS scheduled watches observe selected queue/export/storage conditions,
with local delivery/history and explicit missing/stale/disabled states. Reads neither start services
nor schedule intervention. Native results remain authoritative. The canonical client captures
request-local W3C context before spawning its performer and carries it across dispatcher and
transport tasks into authenticated gRPC metadata. Its narrow native-host span selection uses
the client-owned request target; response/decode lifetime and dispatch uncertainty retain their
original owners. Rust→client→server→Surrealism is the native propagation route;
historical Python/FastMCP and SDK-route qualification
retains its original scope in §24.2.
This continuity remains separate from metric delivery and service identity. Finite capture is retained;
its bounds, restoration and evidence consumers are not replaced by ordinary export. The selected
architecture does not claim exactly-once delivery or an atomic cross-signal source cut.

**Spans.** `tracing` spans named `pse.*` wrap the expensive boundaries:

| Owner | Spans |
|---|---|
| `pse-compiler::workspace` | `pse.case.definition_admission`, `pse.case.structural_analysis`, `pse.case.program_optimization`, `pse.case.sparse_plan` |
| `pse-runtime::math` | `pse.case.program_assembly`, `pse.case.function_assembly` |
| `pse-engine::session` | `pse.operation` around native query, round and command execution |

Fields known only at the end of a span are declared `tracing::field::Empty` at creation
and recorded later; `#[instrument]` uses `skip_all` or explicit `skip` with explicit
`fields`. Library crates install no subscriber; tests (`pse-testkit`) and benchmarks
install their own.

**Engine observation.** `pse-engine::session::assurance::ObservationPolicy` is `Off` by
default. `Contract` installs `datafusion-tracing` instrumentation once per session state
(physical operators and optimizer phases, target `pse_engine::execution`) and records
explicit operation completion, but no plan text: the instrumented state restores the
caller's query planner, so the library's planner span, which would record the complete
logical and physical plan text without bound, is installed under no policy
(`native_unit_contract_capture_records_no_plan_text`, `pse-testkit` units). `Diagnostic`
adds bounded plan text and rule names through its own charged recorder, never value
previews. Captured plan observations are attempt-local, bounded and charged to a
diagnostic reservation; they are diagnostics, not output authority.

**Execution facts as data.** Completed runs project typed relations rather than log
text: `runtime.computation_runs` (state, native termination, qualification, candidate
facts), `runtime.solve_metrics` (effective options and native metrics, with typed
unavailable reasons instead of invented values), `runtime.execution_statistics`
(planning, load and acceleration counts) and `runtime.diagnostics_findings`. Native
solver progress is exposed as events on the joined job (`RunHandle::progress`); a durable
attempt's progress events and incumbents are also stored in the operational store and read
back by `Runtime::progress` or queried under `pse_ops`
([§20.6](identity-and-publication.md#section-20-6)). Field
detail is in the [generated runtime reference](../../generated/relations/runtime.md);
result meaning is owned by [§19](workflows-and-results.md#section-19).

**Operation diagnostics.** Ordinary workflow operations establish an inspectable diagnostic
identity before expensive preparation/admission. Lightweight collection maintains current
activity, cumulative owner facts and bounded recent context; the default recent request is
60 seconds, shortened to actual observed duration. Explicit opt-out, loss, reset, curtailment,
staleness and interruption remain visible. Diagnostic identities reference scientific
model/run/attempt identities without changing their authority or cache keys.

The low observation vocabulary belongs to `pse-diagnostics`; runtime composes access and
publication, and numerical/context owners retain their meanings. Explicit propagation follows
async tasks, native threads, shared producers/waiters and actual managed/native-child
execution. Library code does not install a global subscriber. A reader retains copied
observations rather than scientific cancellation leases or native resources. Closing a
reader does not stop execution; native ownership ends only after actual drain/join.

Bounded history preserves useful active work and admitted compact responsibility summaries
before ordinary completed-attempt detail. The producing composition declares summary eligibility
and its owned facts through the generic observation contract; the hub does not classify workflow
names. Admission charges copied facts and indexing within the existing diagnostic allowance.
Execution completion remains truthful: terminal handoff retains observations only, never native
handles, scientific payloads or cancellation leases. Completed summaries have a default finite
60-second retention lifetime, independent of the recent window and external-reader acknowledgement.
Expiry, pressure-driven displacement, refusal, resets and unavailable detail remain explicit.
Independent readers may retain only their charged copied observations. Compact summaries have
an export route after essential active identity/coverage and before optional repetitive history;
the bounded image does not promise an unlimited complete operation tree.

The completed-summary deadline governs semantic eligibility. Catalogue, read, snapshot,
admission and retained-byte observations prune expired hub outputs. The ordinary publisher
also prunes on publication. An idle hub without a publisher can keep bounded charged copied
bytes until its next interaction or drop; no background reaper or physical expiry-latency
promise follows. Independently held copied readers stay charged and report stale summary
eligibility after the deadline. Transported snapshots reapply the same deadline.

Relationships keep exact
operation identities: an evicted parent is absent/partial, never rebound to another
root. Retained readers may still inspect their copied operation. Producer generations,
resets, saturation and dropped detail survive serialization and recent interpretation;
missing history does not supply a counter delta. Opt-out at process initialization
removes collection without changing requested scientific/native work.

Independent same-host readers consume bounded atomically published process checkpoints,
separate from scientific request/result IPC and closed-result admission. A checkpoint's
publication time is not the freshness of its native facts. Cross-process composition is not
atomic, and hard death preserves at most the last published prefix. Optional writer failure
cannot supply or revoke scientific permission. Cleanup follows exact diagnostic ownership
and publisher lifetime, never generic age/size deletion.

Managed receiver readiness carries its actual runtime diagnostic process identity, start locator
and checkpoint path inside the existing receiver/database registration. A locator may precede
the first checkpoint; opt-out or optional publication failure does not make science unready.
Independent inspection validates the expected identity against the same bounded image it reads.
After valid association, a retained prefix remains inspectable after process exit without a fresh
live-process lookup. This is the last published prefix, not proof of final science or native drain;
the locator grants no service-stop or allocation-release authority.

Numerical observations preserve native phase, activation, coordinate, unit, clock and reset
semantics. Owner-safe callbacks/returns publish already available facts; inspection performs
no residual/property/derivative evaluation, query replay or unsynchronized foreign getter.
Inclusive/parallel/shared work is not blindly summed; unknown is not zero and an unattributed
interval is not automatically waiting. A native checkpoint is not an accepted physical state.

Deep callback detail, expensive engine plans, exporters and sampling remain optional. Engine
`ObservationPolicy::Off` does not disable ordinary lightweight operation diagnostics or imply
that absent engine metrics are zero. Collection bounds are diagnostic settings, not numerical
work limits or new scientific/cache identities. No exact overhead/latency criterion or
identical trajectory guarantee is added. Environment/provenance proof campaigns remain
permanently eliminated. Plan 40 owns implementation and qualification of this accepted target.

**Producer-scoped coherent evidence.** > Decision: ADR-0192 (proposed; maintainer-authorized Plan43a target).

Performing owners declare operation-local producer instances and stable fact populations.
Prepared fact handles bind kind and unit before hot updates. Native boundary, coordinate
transport and original oracle observations remain distinct even when their demands match;
only the existing work owner charges science. Coupled numerical fields are published as one
bounded source-issued record, with producer/attempt, generation, sequence, actual event time,
family, phase and coordinate/norm meanings. Source sequences precede lossy fan-out and differ
from durable append sequences. Current compact records and bounded recent records have
independent retention; history zero does not remove admitted current progress. Point-coalesced
metric histories remain a separate view and never reconstruct an iteration tuple.

Plan48 RC03 / proposed ADR-0192 extend ordinary observation with one prepared
safe-return batch for coupled callback completion counts and elapsed facts. Each
batch is accepted or lost together at one source time; optional metrics may remain
stale or incomplete after loss, which remains explicit. Exact local work/admission,
pre-work and post-success stop checks and final native report counts/time remain
authoritative independently of optional observation. Missing intervals are never
invented from cumulative elapsed time. Current phase retirement does not depend on
batch publication. Compiler ordinary events use prepared populations bound to the
current operation; opaque per-query attribution is selected detail. Plan48g owns
implementation and qualification of this extension.

Selected detail declares event family, interval, sampled/complete requirement and finite
memory/output allowances. Successful queue drain is not complete source coverage; loss,
unsupported families, writer failure and unacknowledged tails remain explicit. Selection is
acknowledged by the actual producer before its interval. Later checkpoint attachment is
sampled and cannot recover an unrecorded prefix. Existing Progress, capture and artifact owners
carry the records without installing a subscriber, retaining unbounded native histories or
implicitly blocking a numerical callback. Optional telemetry failure cannot change science.

Invocation records reference actual strategy/work evidence and preserve distinct qualification,
terminal, response-readiness and requested-product outcomes. Later materialization/reopening
are related access operations, not invented endpoints at native return. Benchmarks retain one
record per executed invocation; Criterion statistics keep their actual separate sample population.
Observation effects are classified before any preparation identity is narrowed. Full effective
settings and resource/reporting consequences remain visible; no identical-deadline-trajectory or
zero-overhead claim follows.

> Decision: ADR-0192 (proposed; Plan44/RC01 implemented). The invocation recorder is established inside the entered activity owner before native dispatch, and that one context transfers through worker and supervisor. Version 2 uses wall seconds from that establishment, excluding earlier preparation; synchronous admission/refusal and nested invocation boundaries remain. Version-1 supervisor-only observations do not supply that expanded coverage. The native worker records its actual bounded inclusive execution interval; it is not exclusive CPU accounting. Selected historical invocation, comparison and projection acceptance retains its original scope in §24.2.

> Decision: ADR-0196 (proposed; Plan43e target). Retained source-event pages can be projected through the registry-declared `runtime.diagnostic_evidence_fields`, `runtime.diagnostic_invocations` and `runtime.diagnostic_captures` relations. Operation/producer/generation/event/field membership, exact time, units, typed values and explicit loss/availability survive projection. Existing source artifacts remain raw authority; finite analytical allocation leases remain attached to escaped Arrow buffers. Native DataFusion consumers use this same projection without scientific-store availability, implicit evaluation or a second retention authority. Repeated event identities refuse the derived view rather than silently deduplicate raw evidence.

Pure interpretation consumes copied coherent history or actual joined invocation documents. Selected numerical-history policies declare thresholds and require compatible phase, generation, sequence, coordinate and norm meaning. Statements distinguish facts, hypotheses and limitations and reference their source fields. Tiny native search directions are not accepted physical displacements; equal endpoints do not establish continuous stagnation. Invocation comparisons preserve requested products, original permission, failures, retries and inclusive clock scopes without initiating science or selecting an automatic performance winner.

Explicit static source-map inspection retains original document/field/occurrence locations, aliases, gathers, fixed/eliminated coordinates and arithmetic contributors. Shared arithmetic execution is represented once while all contributing sources remain attributable. A static branch alternative is eligibility, not evidence it executed. Unknown native pullbacks remain unavailable; passive inspection never reevaluates equations. One nonrenewable caller deadline covers declaration/preparation, artifact waits and mapping/copying; finite byte admission remains separate. Explicit full bounded singular-mode diagnostics qualify actual scaled-operator residuals and normalization; absent rectangular equations are absent, not zero residuals or a global certificate.

Optional Perfetto capture emits actual MONOTONIC/BOOTTIME relationships before events and retains its alignment sampling width. Typed record fields preserve source event membership; independently stoppable bounded query tools consume immutable traces without stopping science. Native KLU detail counts actual issued/completed owner operations; PETSc detail uses an independently owned handler and prepared event populations with inclusive completed-event scope. Accepted Plan45 RC08 retains separate actual symbolic/full-numeric/refactor/substitution observations for fixed-pattern qualification (§18); retained symbolic ownership never implies fresh numeric work or convergence. Unselected hooks do not claim measured zero. Operating routes and tool limitations live in the [diagnostic guide](../../dev/simulation-diagnostics.md).

> Decision: ADR-0192/0196 (proposed; Plan43 production-completeness correction). Explicit numerical diagnostics use one operation clock across preparation, admission and execution and retain optional originating invocation/attempt/failure attribution. Origin conveys no scientific permission. Joined scientific products follow their individual governing original completion; retained raw or partial data remains diagnostic. Launch CPU profiling composes a separately owned command with the existing exporter owner: enable acknowledgement precedes scientific exec, and capture failure/stop never cancels command work. CaptureDocument version 2 identifies the exact launch process with no fabricated selected operation; bounded discovery retains actual related operations. The capture view version 2 uses an explicit process/operation subject. DiagnosticOrigin is one typed optional run/attempt/step/native-attempt/operation/failure reference retained in the version 2 diagnostic relations; native `DiagnosticOperation` consumers share the original caller clock through preparation, term analysis, owned numerical work and initial export. Whole-run refusal and step-local availability remain independent. Changed generated documents/views migrate together. Implementation and current package status remain with Plan43.

### 23.2 Failure taxonomy

> Supplement: [ADR-0152](../../adr/0152-demand-driven-compilation-and-contextual-routing.md) (proposed; Plan 25l functional implementation complete).

An absent original-space feasibility assessment is an explicit refusal distinct from evaluated infeasibility. Completion-owned projections preserve cancellation, precise limits/resource exhaustion, numerical failures, validation causes and contradiction witnesses; adapter-owned native classification remains retained evidence. Registry/Rust declarations own new generated vocabulary and transport, and recorded contracts retain historical interpretation.

> Decision: [ADR-0106](../../adr/0106-execution-vocabulary-discrete-and-global.md) —
> `numerical` and `inconclusive` boundary classes, diagnostic severity and typed
> `ProblemError` variants (Plan 22 A1, implemented);
> [ADR-0104](../../adr/0104-discrete-constraint-forms-and-realizations.md) — the typed
> realization refusal `modeling.realization` (Plan 22 M3 and M4, implemented);
> [ADR-0103](../../adr/0103-variable-domain-facet.md) — the recorded bound tightening
> `modeling.domain.tightened` and the conflicting-bound refusal (Plan 22 M2a, implemented);
> [ADR-0164](../../adr/0164-unify-simulation-substrate.md) (proposed; authorized implementation) — canonical client, codec, completion and lifecycle failures retain typed causes and exact operation identity.

> Supplement: [ADR-0148](../../adr/0148-studies-diagnostics-and-admitted-bindings.md)
> (proposed; maintainer-authorized Plan 25f implementation).

A source-owned diagnostic interface supplies detailed code, closed rule/stage, affected
identities, revision-bound locations, typed observations and ordered causes. Lower owners
project their own variants; upper owners retain the typed cause without a central downcast
classifier or rendered-message interpretation. Request admission (`WorkflowError::Input`),
internal defects and typed source failures have separate contracts. Contextual owners enrich
otherwise unattributed locations recursively with the selected revision, preserving existing
source attribution.

`BoundaryDiagnostic` exists before any model, attempt result or table. Its codec distinguishes
missing evidence, finite values, physical quantities with complete operand contracts, and
explicit NaN/infinity observations. The finite physical input domain does not admit those
nonfinite observations as physical values. Detailed diagnostic identity is authoritative;
coarse `FailureClass`, boundary disposition, severity, scientific usability and retry are
separate purpose-specific projections. A coarse class never reconstructs the detailed code.
Closed numerical/qualification rules are validated at authored admission. Rust, generated
relations, durable termination documents and native reports retain the same structured envelope.
`DiagnosticReport.envelope()` exposes the complete document, including nested causes.

> Supplement: [ADR-0224](../../adr/0224-native-result-inspection.md) (proposed;
> maintainer-authorized Plan52l typed native decode/inspection extension).

The agent interface projects safe native codes, stages, schema-declared structural locations,
related/context populations and authentic issued recovery references through the existing report
bridge. Original source-bearing messages remain private; rendered exception text supplies no new
classification, path or retry permission. Explicit wrapping preserves the native cause. Host input,
selection, stale-interface, prerequisite and unsettled-action failures provide bounded remedies;
an interrupted edit with a retained preview points to its original reconciliation reader.

Initialization uses its declared scientific attempt policy and exhaustive native-stop
projections. Durable retry additionally reads operation and publication-effect state; a
transient infrastructure code alone cannot authorize repeating an unknown publication effect.
Scientific refusal diagnostics and available members survive operational failure or cancellation.

**Ownership.** `pse-diagnostics` declares one vocabulary: `FailureClass` (coarse class)
and `DiagnosticCode` (detailed code, each mapped to one class). The registry projects
both into generated enums, Arrow and native documents; nothing else declares a code.
Codes are dotted in data (`authoring.parse.missing_id`) and rendered in Rust path form
by miette (`authoring::parse::missing_id`).

**Invariants.**

- Every public `*Error` enum in `crates/*/src` derives `thiserror::Error` and implements
  `miette::Diagnostic`, either derived or through the shared
  `pse_diagnostics::impl_diagnostic!` projection that assigns a typed `DiagnosticCode`.
  `tests/governance/tests/error_taxonomy.rs` checks the implementations; the compiler
  checks that each code is a declared variant. No `anyhow` in library crates.
- Wrappers retain original causes and related failures. Aggregates keep every leaf;
  the first error is never the only one reported. Formatting an error into a string
  does not establish its class or erase an uncertain commit.
- Findings are relations; a rendered diagnostic is a projection of a finding, never its
  storage. Boundary diagnostics carry source identities, stage and observations
  (`pse-model::diagnostic`); native reports preserve that same structured content.
- A non-converged or infeasible solve is a typed result (native termination, candidate
  kind, qualification), not an error ([§18](numerical-execution.md#section-18)).
  Errors cover refusals, evaluation failures, failed native operations, resource limits,
  cancellation, infrastructure and internal failures.

**Classes.** The IDAES column names the counterpart used for behavior comparison
([§6.14](schema-and-relations.md#section-6-14)).

| Class | Detailed code families and examples | IDAES counterpart |
|---|---|---|
| `authoring.parse` | `authoring.parse.{syntax, unknown_key, missing_id, nonfinite_number, budget, document_io, ambiguous_unary_power, unresolved_target}` | `ConfigurationError` |
| `authoring.reference` | `authoring.reference.{contract, derived_write, rename_named, unknown_row_key}`, `authoring.pkg.{unresolved, version_conflict}` | `ConfigurationError` |
| `validation.invariant` | declared invariant or boundary refusal; `schema.*` registry and contract admission codes | `ConfigurationError`, `PropertyPackageError` |
| `compile.property` | unsupported or ambiguous property, incomplete provider | `PropertyNotSupportedError`, `PropertyPackageError` |
| `compile.math` | `compile.math.{unit_inconsistent, quantity_operation_unsupported, domain_violation_static}` | Pyomo `UnitsError` |
| `kernel.unbound_parameter` | a selected provider lacks its executable or parameter binding | `PropertyPackageError` |
| `capability.backend` | unsupported capability, an unlinked or ineligible backend, or numerical preparation | `PropertyPackageError` |
| `solve.evaluation_error` | nonfinite or rejected trial evaluation | Pyomo evaluation errors |
| `solve.solver_error` | a native method or numerical kernel failed; a failed injected inner solve; `numerical` and `inconclusive` boundary diagnostics | — |
| `runtime.cancelled` | a cancellation token fired | — |
| `runtime.timeout` | a wall-clock deadline was exhausted | — |
| `runtime.resource_limit` | reservation, size, work or memory limit exceeded | — |
| `runtime.infrastructure` | storage I/O, integrity failure, publication conflict or incompatibility; proposed ADR-0214 keeps typed conflict exhaustion, poisoned store and uncertain dispatched effects distinct | — |
| `config.invalid` | invalid engine or platform configuration | — |
| `internal.invariant` | a postcondition failed; a platform bug | `BurntToast` |
| `user.model` | an authored query or assertion failed | `UserModelError` |

The vocabulary also declares `compile.feature`, `compile.law`, `compile.discretization`,
`plan.initialization`, `solve.infeasible`, `solve.locally_infeasible`, `solve.unbounded`
and `solve.limit`, and the detailed codes `compile.math.cyclic_expression`,
`template.guard_undecidable`, `rule.float_key`, `rule.head_schema_mismatch`,
`schema.rule_float_key` and `schema.rule_stratification`. No current Rust error
produces them; solver terminations are result tags instead. They remain declared
vocabulary, not evidence of a supported failure path.

**Boundary classes.** `pse-model::diagnostic::BoundaryClass` is a purpose-specific
boundary disposition. Source-owned detailed codes retain their declared coarse `FailureClass`;
`disposition` supplies an exhaustive boundary projection. `numerical` is an algorithmic
failure without a model cause; `inconclusive` is an analysis that could not reach its conclusion.
Neither severity nor retry permission changes diagnostic identity.

**Severity.** Every boundary diagnostic also carries a `DiagnosticSeverity`: `error`
(the default), `warning` or `info`. Severity is independent of class, and a warning never
makes a model invalid. `runtime.modeling_findings` publishes class and severity as
separate columns. Each numerical-diagnostics rule
(`pse-runtime/src/workflow/modeling/diagnostics.rs`) has an explicit disposition:

| Findings | Class | Severity |
|---|---|---|
| Structural under- or overdetermination; missing variable value | `invalid_model` | `error` |
| Nonfinite variable value | `nonfinite` | `error` |
| Unused variable; variable only in inequalities; potential domain evaluation error | `invalid_model` | `info` |
| Variable outside a bound; large equation residual; parallel Jacobian rows or columns; numerical rank deficiency | `numerical` | `warning` |
| Variable near a bound, fixed at zero, or of large or small magnitude; extreme Jacobian entries, rows or columns; mismatched or cancelling equation terms | `numerical` | `info` |
| Failed term evaluation | `trial_rejected` | `warning` |
| Jacobian analysis that could not complete | `inconclusive` | `warning` |

Scaling, conditioning and near-bound findings are therefore warnings or information,
never an invalid model. An undeclared rule is an `internal` error.

**Modeling domain refusals.** `pse_modeling::ModelingError::Domain` is the typed refusal
of a declared variable domain ([§6.8](schema-and-relations.md#section-6-8)). Its boundary
rule is `modeling.domain`, with the variable's instance path, its domain, the analysis and
the reason as observations and its declaration and variable identities as sources. A
quantity kind that is not a count or indicator, a binary bound outside `[0, 1]`
(`ConflictingBound`) and bounds that admit no value of the domain are `invalid_model`
(`validation.invariant`); missing finite bounds and a free discrete variable that the
analysis cannot decide are `unsupported` (`capability.backend`). An integer bound that
admission tightened inward is not a refusal: `DomainTightening` is the finding
`modeling.domain.tightened`, class `invalid_model` with severity `info`, observing the
variable, its domain and the specified and tightened bounds.

**Modeling realization refusals.** `pse_modeling::ModelingError::Realization` is the typed
refusal of a constraint-form or disjunction lowering that the bound case cannot admit
([§19.7](workflows-and-results.md#section-19-7)). It names the authored form or disjunction
(declaration and path), the subject row or variable, the declared realization and one of four
reasons (`RealizationRefusal`): `InfiniteBound`, a hull or linear lowering that needs a finite
case bound on the named variable; `IncompleteInterval`, a derived big-M row whose interval is
not FBBT-complete; `UnboundedInterval`, a derived big-M row whose interval is not finite over
the case box; and `Nonlinear`, a nonlinear disjunct row under `hull` without an ε. Its code is
`capability.backend`, and its boundary rule is `modeling.realization`, class `unsupported`,
with the form, subject, realization and reason as observations and the declaration as
source. A malformed declaration (a misplaced form, a wrong type or argument, competing
realizations) is an ordinary checking refusal (`validation.invariant`).

**Native failures.** `pse-backend-native::ProblemError` classifies a refused request or an
attributable native failure by cause; each variant keeps its cause and structural
identities, and none is flattened into a string of unknown class. Workflow diagnostics
(`pse-runtime/src/workflow/diagnostics.rs`) derive the boundary class and a stable rule
from the variant:

| Variant | Meaning | Code | Boundary class (rule) |
|---|---|---|---|
| `Unavailable` | The explicitly selected backend is not linked; other linked backends are listed; no fallback | `capability.backend` | `unsupported` (`native.unavailable`), backend observed |
| `Unsupported` | No eligible route, an ineligible explicit selection, or an adapter that cannot represent the request | `capability.backend` | `unsupported` (`native.unsupported`) |
| `Reuse` | `RequireReuse` found retained state it cannot reuse: another backend's (`Foreign`), a different structure (`Structure`) or options the step does not set (`DroppedOptions`) | `capability.backend` | `incompatible` (`native.reuse`), backend, reason and the held backend or dropped keys observed |
| `Contract` | The model or request violates a declared contract | `compile.math` | `invalid_model` (`native.contract`) |
| `Structural` | Structural deficiency with overdetermined rows and underdetermined columns | `compile.math` | `invalid_model` (`native.structural`), rows and columns as sources |
| `Math` | Mathematical evaluation failure with its domain or provider cause | the cause's code | the cause's class |
| `Provider` | A registered provider failed outside an attributed expression | the cause's code | the cause's class |
| `Numerical` | A native method or numerical kernel failed; the native status is kept when one exists | `solve.solver_error` | `numerical` (`native.numerical`), native backend, code and status observed |
| `Limit` | A declared finite allowance (`Time`, `Work` or `Memory`) was exhausted | `runtime.timeout` for `Time`, otherwise `runtime.resource_limit` | `resource_limit` (`native.limit`), limit kind observed |
| `Cancelled` | Cooperative cancellation fired | `runtime.cancelled` | `cancelled` (`native.cancelled`) |
| `Internal` | An adapter postcondition or platform invariant failed | `internal.invariant` | `internal` (`native.internal`) |

A failed native call is classified from its mapped stop category
(`ProblemError::native`): cancelled → `Cancelled`; time limit → `Limit` (`Time`);
iteration, solution, objective and general limits → `Limit` (`Work`); resource
exhaustion → `Limit` (`Memory`); invalid or panic → `Internal`; every other category →
`Numerical` with the native status. A failed injected inner solve
(`pse-math::MathError::Native`) carries `solve.solver_error` and keeps its typed native
cause, which the workflow classifies together with the block identity.

**Canonical store failures.** `pse_operations::canonical::CanonicalError` retains typed protocol/client and codec causes, interpretation mismatch, reused operation identity, payload limits, quiesced admission, deadline expiry and missing statement completion. A transport success alone cannot establish absence, write success or a complete exported stream. Complete native statement results and application result shape are checked before those conclusions.

Generated guarded refusals have one declared code/class owner. Decoding first
requires native `is_thrown()` and then an exact known prefix; unknown codes, ordinary
schema errors and wrapped event failures stay fail-closed. Runtime policy consumes typed
refusal/effect information, distinguishes acknowledged study cancellation from uncertain
export, and records worker loss as infrastructure termination.

The server preserves structured unary gRPC errors in bounded binary metadata, separately
from its bounded human message. Native conflict details, rather than message text, permit
conflict retry. MX capacity, transaction age, queue exhaustion, poison and corrupt-store
failures retain distinct meanings; persistence failure may leave commit outcome unknown.
A dispatched refusal is no universal proof that scientific effects were absent.

Definite transaction conflicts retry the complete fresh guarded decision. Unknown acknowledgments settle the original immutable operation request; they cannot authorize new scientific effects. Initialization uses the existing longer bounded atomic-client timeout, while ordinary reads retain their shorter deadline. Reopening verifies interpretation and never installs schema. Codec domain/shape errors remain validation failures; workflow boundaries preserve resource, cancellation, incompatible and infrastructure distinctions. Messages do not supply lifecycle authority. A fixture's scientific execution policy remains authored with that fixture ([§6.10](schema-and-relations.md#section-6-10)).

**Native engine errors.** `pse-columnar::engine` classifies `DataFusionError` in one
place, by the plan's origin (`PlanOrigin`), never per call site:

| Native error | Class |
|---|---|
| `External` carrying a typed platform diagnostic | that diagnostic's declared code |
| `ResourcesExhausted` | `runtime.resource_limit` |
| `Configuration` | `config.invalid` |
| `Plan`, `SchemaError` from an authored analytics query | `user.model` |
| `Plan`, `SchemaError`, `NotImplemented` from numerical preparation | `capability.backend` |
| `Execution` inside a kernel adapter | `solve.evaluation_error` |
| other `Execution`, `ArrowError`, `IoError`, `ParquetError`, `ObjectStore` | `runtime.infrastructure` |
| `Context`, `Diagnostic`, `Shared`, `Collection` | traversed at every depth; one observation per leaf, retaining contexts and native diagnostics |
| anything else, including planning errors from platform-generated plans | `internal.invariant` |

`SessionConfig::set_str` and `Field::extension_type()` panic instead of returning errors
and are banned in `clippy.toml`; the typed configuration path and
`try_extension_type` are used instead.

## 24. Testing and qualification

Tests establish behavior; qualification is a separate, maintainer-requested activity
that runs the relevant checks once and records scope and conditions
([ADR-0092](../../adr/0092-ordinary-execution-evidence.md)). The failure baseline is
zero. A report names the command, mode and baseline. Architecture review and
design-change tracking are owned by
[§24.4](design-change-workflow.md#section-24-4).

### 24.1 Test layers

First-party development configuration, documentation metadata/lifecycle/site interpretation,
fixtures, governance, generation and evidence callers have native responsibility owners. Typed
fixtures use production admission and original resource owners. Strict YAML and Markdown parsing
serve the existing documentation policies through `serde-saphyr` and `pulldown-cmark`; parser
success does not decide lifecycle, current authority or publication. The source-only knowledge
constructor captures bounded bytes once, preserves native metadata and section/package identities,
and reports unknowns and omissions rather than inferring API use from dependency declarations.

Python-only product declarations, stubs, extension currency and wrapper tests retire with their
producers/consumers. Native registry/schema and complete regeneration equivalence remain required.
Independent scientific oracle exchanges are isolated from production. Retired measurement drivers
hand their exact scenarios, inputs, assertions and observation/custody requirements to H; historical
measurements retain only their original scope. Retirement neither runs those campaigns nor
establishes replacement performance.

> Decision: [ADR-0233](../../adr/0233-rust-owned-host-and-execution-boundaries.md) (proposed;
> maintainer-authorized Plan58 native cutover).

Native typed operations, protected result readers and passive reports are the retained access
routes. The Python MCP endpoint, its presentation-only contracts, generated Cyclopts frontend,
Codex refresh/reconnect helpers and first-party mcpls clients are deleted. Their dedicated tests and generated
outputs retire with those mechanisms. The [interface guide](../../dev/capability-interface.md)
names current native routes. No replacement server or mandatory CLI
follows. The existing structural migration retains its source-cut, preview, apply and
reconciliation owner; ordinary editor and shell workflows supply source navigation.

Rust scientific and canonical declarations retain their existing owners. Original request identity,
correlation, deadlines, cursor/issuer/selection binding and physical-context rules remain native
contracts. Observation close is distinct from scientific cancellation, durable completion and
physical release. Protected result custody survives another reader or publisher closing; native
and connection owners retain actual settlement and independent cleanup failures. Removing a
frontend does not qualify unexercised native consumers or the assembled product.

**Implemented (ADR-0231, maintainer-directed):** explicit native product and fixed
evidence-executable construction publish a completed immutable artifact generation through the
existing preparation receipt. The build/install owner establishes validity once and publishes the
typed descriptor only after completion. Failure before publication commits preserves the previous selection. If descriptor replacement
has an uncertain outcome, the constructor retains both possible issued generations rather than
deleting an image the descriptor may now select. Native image attachment consumers use the issued generation and check distinct requested compatibility;
that image path does not rescan source, compiler environments, Cargo records or artifact digests. Linting and
source edits affect later construction, not the validity of the existing generation. An attached
generation survives mutable installation-alias changes and newer publications until explicit
replacement drains its owner. Original cursors never attach silently to a replacement.

Native attachment retains its original environment/provider owner. Explicit image replacement
drains those native owners; source-only discovery never prepares prerequisites. Explicit stages
compare requested mode, profile and validation at construction. Historical frontend attachment and
two-generation evidence in §24.2 do not qualify replacement consumers. Original native owners
close admission and retain resources until actual settlement; observer loss grants no release.

> Decision: [ADR-0219](../../adr/0219-selected-development-evidence.md) (proposed; maintainer-authorized Plan51d target).

Selected authentic owner snapshots support supplied-only Arrow/DataFusion evidence questions
and explicit activated cuts over the independently disposable tooling index. Typed invocation,
assessment, diagnostic, host and measurement adapters retain authority/codec/identity, source
digest, declared inventory/context, coverage and actual links. Failed, unreported, skipped,
reused/transferred and command-failure populations remain distinct. Offline dispatch precedes
service/activity initialization and requires neither a product extension, implicit build nor
unrelated traversal; original snapshot/detail inspection remains usable without the optional
query executable.

Imports reuse the same facts and staged/activated generation owner. Repeated snapshots deduplicate;
changed versions require explicit history selection. Results describe their selected imported cut;
freshness is established only by supplied current observations, with stale/missing sources explicit.
Diagnostic questions preserve separate tool, target and scientific outcomes. Only declared links
associate actions and invocations; timestamps establish no association. Selected raw detail uses
original owner protection or a validated supplied copy, optionally bound to an expected digest;
index presence grants no custody. Released detail leaves frozen summaries usable.

Measurement populations preserve actual context and sample membership, genuine samples separately
from reported summaries, failed/censored samples, known capture costs and explicit unknowns.
Unmeasured smoke output is not a performance sample. Criterion-to-invocation membership remains
unavailable when its owner does not record it; these queries establish no causal defect, regression
or quantitative benefit. Storage-health collection and portable closure/release remain separately
owned capabilities. The [substrate guide](../../dev/surreal-substrate.md#research-versions-study-workbench-and-selected-evidence)
describes the supported selected-source commands and operational limits.

> Decision: [ADR-0216](../../adr/0216-project-and-evidence-foundations.md) (proposed; Plan51a evidence catalogue).

Imported evidence is a derived projection with original authority/codec/identity, snapshot and
coverage. Only complete reconciled import generations activate; incomplete source populations
remain explicitly incomplete. The index cannot compose terminals, release resources, grant raw
artifact lifetime or infer missing tests passed. Explicit rebuild/drop affects only the selected
tooling database. Original owner inspection remains independent of compiler/product/database
availability; metadata references are not current protection grants.

> Decision: [ADR-0160](../../adr/0160-centralize-testing-responsibility.md); [ADR-0170](../../adr/0170-scoped-arrow-validation.md)

| Layer | Location | What it establishes |
|---|---|---|
| Crate tests | `#[cfg(test)]` modules and `crates/*/tests` | the owning contract: identity, admission, compilation, physical providers, native adapters, workflow, publication and settlement; `pse-rules` owns independent integrity mechanism witnesses and catalog/enforcement controls |
| Governance | `tests/governance` | workspace invariants: sole hasher, dependency pins and floors, error taxonomy, crate registration, FFI unwind containment, no shadow structs, registry governance, MSRV, unsafe allowlist |
| Engine | `tests/engine` | provider hierarchy, plan codec, pushdown truthfulness against unpruned sources, relational expansion, unified sources |
| Conformance | `tests/conformance` | selected scientific acceptance journeys; native value/IEEE framing and Arrow admission properties remain in their owning crates |
| Lifecycle | `tests/lifecycle` | guarded canonical admission, interruption, exact retained reads and result memory budgets |
| Native application | `crates/pse-runtime/tests/native_*.rs` and workflow tests | actual typed scientific/local/durable callers, protected data, original completion and resource settlement |
| Developer tools | `xtask` and `pse-host` tests | native source/doc/governance/evidence owners, exact command/environment selection and authenticated independent operator effects |
| Independent references | native exchange tests and isolated `tests/reference/oracles` | the IDAES 2.13.0 environment and selected scientific reference comparisons (proposed [ADR-0097](../../adr/0097-modeling-scope-and-parity.md)) |

Structural algorithms are tested in their owning production crates; no empty structural
test package is retained. Scientific checks compare against analytic, exhaustive or
independently generated references; they establish the stated cases, not untested
formulations. Parity does not establish numerical IDAES equivalence
([relationship to IDAES](../../relationship-to-idaes.md)).

**Invariants.**

- Arrow `force_validate` is a feature, not a profile. Correctness checks/tests explicitly
  activate it for the actual selected Arrow-consuming normal/build/test-dev closure,
  including target and host dependencies. Command composition selects opt-in boundary-owner
  propagation and verifies the enabled Arrow leaf features; unknown or uncovered consuming
  closures refuse. Pure closures add no Arrow dependency or unrelated relation target.
  Workspace-unifier edges do not establish consumption, and never activate validation by
  default. User target, package and feature selections remain part of the closure.
  Workspace commands may use the relation flag when those targets are already selected.
- Tests own requested temporary files, private fresh databases and mutable runtime state.
  Native canonical fixtures select an explicit immutable prepared module and an already-built
  helper; they never borrow or replace an existing user store. Isolated oracle exchanges fail
  when their requested environment/reference is unavailable; no product Python environment is
  required. Prohibited-write checks use disposable known inputs, not concurrent Git polling.
- Generated trees equal a fresh regeneration through the complete nonmutating xtask
  comparison, including missing, extra and untracked outputs. Product tests do not repeat
  a weaker freshness check ([ADR-0160](../../adr/0160-centralize-testing-responsibility.md)).
- Native test selection belongs to Cargo/nextest; no exact-name acceptance manifest exists.
  Reports distinguish executed, unchanged-input reuse, reviewed transfer and not-run
  evidence. Feature campaigns may reuse individual completed passing combinations only from a
  complete frozen cargo-hack enumeration with identical literal arguments, tool/compiler identity,
  conservative consumed inputs, native context and actual force-validation coverage. Missing,
  failed or interrupted combinations prevent complete qualification; reused results are explicitly
  identified and never described as fresh execution. Current receipts bind conservative gate-specific consumed inputs and actual
  native/binary provenance; unknown scope captures all inputs. One terminal composer
  reconciles actual selection, failure, cancellation and missing evidence. Comprehensive
  qualification runs the shared native graph once, with separate feature-absence controls
  and separately requested independent-reference exchanges; ordinary effects follow requested
  fixtures and resource owners.
  Invocation resources associate that exact selection with an isolated mutable context
  before creation. Fixture Drop/exit records borrower/drain facts, never test success.
  Only reconciled terminal pass plus actual reader/native/cgroup drain and no protected
  reference permits automatic disposable cleanup. Failed/incomplete outcomes and unknown
  historical resources remain pinned until explicit release. Reference acquisition and
  receipt publication coordinate with reclamation, preserving original paths/digests.

> Decision: [ADR-0174](../../adr/0174-exact-resource-metadata-and-terminal-freeze.md)
> (proposed; maintainer-authorized Plan 36 target).

Test-resource metadata has one exact resource authority, full-identity indexes and
invocation-owned membership. Registration publishes durable Pending membership before
resource authority and refuses after terminal freeze. Stable exclusion follows invocation,
identity, resource order and the original deadline; it never spans external effects or
physical drain. The selected composer freezes authentic terminal identity and membership,
then projects exact resources resumably. A terminal result does not prove drain or dispose
an accepted late effect. Bounded maintenance follows active/unresolved paths. Versioned
cutover excludes legacy writers and durably publishes their refusal fence before v2
activation. Fenced-but-unactivated recovery validates the same staged imports before
activation, preserving uncertainty/evidence and never restoring writable legacy metadata. Plan 36e owns the
implementation and actual-file/race qualification of this amendment.

Explicit exact-invocation abandonment may separately close membership under that same
owner and exclusion, without fabricating a missing terminal. Pending publication uncertainty
remains protected; later registration refuses before effects. Disposition is resumable and
never grants automatic cleanup, overrides a frozen terminal or bypasses owner/descendant
drain, creation borrowers, references or unknown external effects. The selected composer
retains sole outcome authority. Plan50 C05 owns implementation and race/recovery qualification
of this maintainer-selected extension to proposed ADR-0174.

**Accepted target — Plan53 P00 (operator-confirmed 2026-10-23).** Managed correctness
commands will return zero if and only if the selected terminal composer establishes a
complete reconciled Pass. Raw child return (`workload_exit`), cancellation and cleanup
remain separately observable. A zero-exit child with selected but missing reports cannot
produce a zero managed correctness exit. Cleanup after terminal publication cannot rewrite
the frozen assurance verdict. This changes the managed command's assurance interface, not
`scripts/pse-env` signal/exit forwarding or read-only result-reader success semantics.
Plan53 C04 owns implementation and caller/self-test migration; publication establishes no
implemented exit behavior.

The same accepted target gives actual selected effect/resource requirements one semantic
owner across managed affected/native/setup routes. Tool catalogs remain selection
authority; homogeneous binary/module defaults and explicit exceptions must correspond to
actual selected cases and guard effect creation. Demonstrably pure controls carry no accidental
solver/canonical barriers. Genuine native thread, timing, service and remaining canonical
protection stays conservative. Per-run exclusion is not service-wide quiet. Plan53 C02/F01/F02
own implementation; this is neither a universal exact-test manifest nor permission to widen
canonical concurrency. The [Plan53 coordinator](../../plans/53-test-architecture-target.md)
owns current package and finding state.

**Preparation and retained test evidence — Plan53 D/E/F.** Source capture selects its
conservative declared scope before opening or hashing bytes; membership, absence, modes and
symlink targets remain inputs where consumed. A whole-source provenance request retains its
whole-source cut. Ordinary selected Rust execution consumes its retained actual build and
catalogue, including original tool/configuration inputs; generation aggregates invoke shared
owners once while their independent comparison routes remain available. Native preparation,
immutable artifact publication and independent reference environments retain their separate
construction/admission responsibilities. Published artifact validity is established at construction rather than
repeated by readers; fresh numerical-state checks retain their scientific responsibility. Declared
assessment-output placement and process SDK
selection are runtime observations rather than compiler inputs; unknown/compiler/provider
environment controls belong to subsequent construction or their explicit runtime owner.

Collected invocation selection and semantic requirements are immutable, digest-bound owner
data. Compact coordination retains mutable membership and references to that data and frozen
terminal detail. Uncollected staging is not an empty collected selection. Format conversion
excludes affected old writers/readers, preserves authenticated original metadata and terminal
identity, and publishes activation last behind a resumable refusal fence. Historical policy
does not acquire current effect permission merely through conversion.

Covered service and scratch access paths are derived discovery projections. Their membership
is protected before entry becomes authoritative; departure becomes authoritative before its
marker is removed. Actual service/registry binding and complete writer coverage are prerequisites
to an absence claim. Unresolved original associations remain protective unknowns. Scratch
candidate discovery is independent of retained evidence discovery and cannot itself authorize
release. Every effect, maintenance action and cleanup rechecks its actual authority and lifetime.
Physical fixture settlement shares one original outer deadline across native fixture and receiver
owners, attempts independent siblings, and retains each error and incomplete reader/owner;
logical close or terminal pass alone cannot establish physical drain.

Assessment version 6 atomically publishes a compact root after its immutable source and gate
detail is durable. Selected gate/failure readers open only requested detail; source detail and
faithful full export are explicit. Known sealed root/detail reads authenticate the exact bytes
and file identity under owner protection; full-archive inspection verifies the entire manifest.
Missing or corrupt requested detail is unavailable/refused, never empty or passing. Historical
original inspection grants no current automatic reuse. Evidence questions compute semantic
populations and comparability before independently bounded optional section pages, retaining
section completeness, continuation and original source-generation context.

Selected requirements also derive scheduling and pre-effect admission. Positive case namespaces
and fixture roles replace duplicate broad exclusion rules. Database-local assertions retain
database isolation; real service maintenance retains lifecycle exclusion and covered borrowers
across invocations. Timing/native/thread constraints remain where actually demanded. These
mechanisms establish no measured speedup or general permission to widen canonical capacity.
Current implementation, qualification and scientific limits remain at the Plan53 coordinator.

**Commands.** During implementation, compile touched packages and run targeted owning tests with
`just check-package <pkg>` and `just unit-package <pkg> <filter>`; declaration changes include
`just codegen`. Native correctness routes use Cargo/nextest with explicit selected Arrow validation.
`just native-test <module-receipt> ...` builds selected tests and runs them against a private fresh
native canonical fixture using that immutable module. `just docs-test` exercises the native
metadata/lifecycle/publisher owners without a scientific product environment. On request, relevant
native scientific/component/integration journeys, `just doctest`, `just governance-tests`,
`just setup-test`, `just codegen-check`, family checks and scope-end hygiene/manual publication
qualify their selected responsibilities. `just parity` runs explicit native candidate exchanges
with the isolated IDAES/Pyomo environment. Current command arguments and unsupported selections
belong to the [development guides](../../dev/README.md) and `just --list`; no extension build,
Python product import, wheel or generated Python API check is an acceptance prerequisite.

Storage health uses the versioned `storage-health-v1` observation document. It composes the
selected service/engine identity, bounded process/cgroup and host storage observations, exact
capture time, unavailable fields and approximate engine statistics. Service identity and cgroup
observation are verified independently of metric transport. An oversized, timed-out or invalid
metrics response retains its precise unavailable reason and the original response bound; it does
not by itself establish that the service is down. Owner/incarnation changes are rechecked before
combining observations. Engine memory is not RSS;
queue depth and retained versions alone establish no cause or durability guarantee. Finite owner
command references are inert suggestions, not repair authorization. Independent host inspection
works without the service. Explicitly supplied or imported diagnostic evidence can retain these
observations without fabricating invocation, process or scientific associations. The
[diagnostic guide](../../dev/simulation-diagnostics.md) owns operational usage.

**Development knowledge and source-backed changes.** Native source owners supply exact
contract-section, decision, plan-package, declaration, generator and generated-output identities.
Interpreted metadata retains lifecycle/site policy inputs; decision state, package state and
publication scope remain independent. A task packet consumes bounded captured native-reader
bytes and refuses stale rows. Historical Git/session rationale is attributable evidence, never
current authorization. Exact producer assertions preserve declaration–generator–specific-output
correspondence; two declarations sharing a generator do not inherit each other's outputs.
Catalogue registration, type definition and actual compiled/native API use retain distinct roles.
Dependency declarations alone establish neither API use nor feature activation.

The optional development-knowledge v2 projection reuses the existing private tooling database,
roles and original request lifetime. Closed subjects, role-specific contributors, native topics,
typed ENFORCED relations and exact Production assertions sit above immutable source versions.
Record references carry explicit existence, kind and same-generation constraints; optional
excerpt attribution is also checked on a required field. Only explicitly attributed excerpts
participate in a subject's optional text filter. Source-only exact queries work without the
projection; lexical discovery is explicit and semantic similarity remains unselected.

Import reconciles every normalized family before atomic activation, including replay of an
already active generation and acknowledgement-loss settlement. Exact CAS and issuer/query/cut
cursors prevent stale activation, pointer rewind or mixed pages. Empty query results retain the
complete provenance source cut. Current repository observations include policy contributors;
external evidence references retain original authority/codec/source/version/member identity and
grant no payload lifetime. Producer fingerprints remain unverified against the live registry,
and undeclared governing/qualification relationships remain unknown. Native indexed query/explain
adapters preserve finite original versioned requests and exact answers without import/service
startup or source fallback. Producer-declared governing sections and explicit source-bound test
markers add attributed contract/evidence links. Unexecuted selection is never executed support;
failed execution retains its status and establishes no positive qualification. Passive affected
preview and prior-evidence compatibility reuse original selection/source-stamp owners, with
unknown environments/populations visible and no automatic pruning. Knowledge-only reset leaves
original files and the independent evidence projection intact. Whole-tooling disposal is explicit.
The [environment](../../dev/agent-environment.md) and
[substrate](../../dev/surreal-substrate.md#disposable-development-knowledge) guides own commands.

> Decision: [ADR-0222](../../adr/0222-derived-development-knowledge.md)
> (proposed; maintainer-authorized Plan52a/52b target). Selected host/native checks establish
> these bounded tooling behaviors, not assembled Plan52 or scientific qualification.

Scientific reduction is a separate diagnostic request/result contract from original reproduction.
The first supported family retains an authored numerical evaluation-domain issue under explicit
code/stage/rule, source correspondence and necessary domain conditions. Candidate execution uses
the existing runtime, admission and cancellation owners. Parser errors, unrelated failures and
incomplete evidence do not establish preservation; search reports only its actual moves and bounds.
The original typed reproducer and `OriginalFailureReproduced` meaning are unchanged.

> Decision: [ADR-0223](../../adr/0223-separate-scientific-reduction.md) — Proposed,
> maintainer-authorized Plan52c target; implementation and qualification remain at C06.

### 24.2 Current qualification basis

The current native source cut implements the first-party callable Rust application and native
host/developer owners described by ADR-0233. The retired Python product and generated frontend
outputs supply no current acceptance requirement. Exact package state, commands, failures and
successful affected reruns remain at [58a](../../plans/58a-native-product-and-scientific-workflows.md),
[58f](../../plans/58f-development-and-assurance.md) and
[58g](../../plans/58g-boundary-retirement-and-optional-consumers.md).

**Tested, focused local Linux scope:** the native boundary acceptance callers exercise actual SCIP
with its scientific 1 MiB solver heap stop and managed two-Ipopt execution through a fresh canonical
service and external native workers. The SCIP case retains no candidate, unavailable feasibility,
nine null variable observations and exact durable assessments/findings. The selected-detail case
binds external process and exact attempt identity, positive observed/acknowledged samples and an
incomplete parent recording. Their names are
`public_durable_scip_memory_stop_retains_unavailable_feasibility_and_exact_rows` and
`public_managed_detail_keeps_external_owners_exact_attempts_and_complete_prefixes` in
`native_boundary_acceptance`; their prepared profile, commands and original repairs remain at 58g.
Selected native host, feature/field, source/documentation and actual ADR/corpus command checks
support only their exercised responsibilities; they are not a clean assembled product campaign.

[58h](../../plans/58h-qualification-and-publication.md) owns the remaining assembled/native and
durable-deployment qualification and the exact measurement-scenario handoffs. These campaigns
remain unqualified by this publication. No whole B–G closure, full feature powerset, full
solver-fleet/parity, distribution, other-platform result or measured benefit follows from source
retirement or the focused checks. Historical frontend evidence below is historical, with no
transfer of acceptance to deleted or changed paths.

The historical full qualification basis is Plan 22's Q1 of 2026-09-29, on the final state of
the solver, discrete-decision and operational-store work. The retired record states each
command and result:
[Plan 22 Outcome](https://github.com/paul-heyse/pse-arrow/blob/f57b71d56f6eb2c319c4340d6f26abc6a1dc5abc/docs/plans/22-solver-capabilities.md#outcome-recorded-after-implementation).

**Scope and conditions.** Local Linux, the pinned dated nightly (ADR-0122) and Python 3.14.7
(parity on 3.13), against a zero failure baseline:
- the default workspace (`just test`) and the full native feature graph with explicit
  force-validation (`just native-test`: native units, conformance acceptance and the
  invariant harness);
- the store, worker and publication journeys against the local PostgreSQL 18 server;
- the linked Python suite (unit, component and integration) on a freshly built extension;
- doctests, governance, generated equivalence and family pins;
- the strict Clippy modes, including the native Clippy of the linked solver contracts;
- formatting, Python quality and documentation checks;
- the IDAES parity comparisons on this machine (`just parity`: environment, enumerations,
  sIPOPT sensitivities, parmest covariance, DegeneracyHunter and a PETSc PID loop).

Native runs used single-threaded BLAS/OpenMP unless a test admits threads.

**Measurements.** The large-KKT linear-solver and Clarabel thread measurements are single
runs on a synthetic grid (`measurement_tests`, run on demand). They are local observations
that separate orders of magnitude, not performance guarantees. The earlier 33 measured
process cases (2026-09-25/26) were not rerun, and the Plan 22 process cases were not added.

**Exclusions.**
- No IDAES numerical parity beyond the exercised comparisons.
- No exhaustive feature powerset, release or distribution, remote CI, container or
  other-platform result.
- No performance campaign.

Analytic, exhaustive and reference checks establish the stated cases and do not establish
untested formulations. The upstream `proc-macro-error2` future-incompatibility notice
remains; this is not a warning-free toolchain claim.

Raw reports are local ignored outputs under `build/assessment/` and are not part of the
repository. The previous basis, the
[final qualification](https://github.com/paul-heyse/pse-arrow/blob/8950dd3d6ddb3aa7c78acc0db7d0601497b302a8/docs/plans/16-p14-p18-execution.md#verification)
of 2026-09-25/26, is historical.

A retired qualification is not transferred to changed paths. After a change, targeted
tests show the new behavior; comprehensive qualification runs again only when the
maintainer requests it.

Focused SurrealDB lifecycle/integration acceptance and its exact commands, conditions and
exclusions belong to the [Plan 37 Outcome](../../plans/37-surrealdb-lifecycle-and-integration.md#outcome-recorded-after-implementation).
That affected-scope evidence does not replace Plan 22's historical full basis, qualify
unchanged or unexercised scientific scope, or close other plans' incomplete campaigns.
Source integration and document reconciliation alone do not establish acceptance.

Focused simulation-pipeline acceptance of 2026-10-16 belongs to the
[Plan 41 Outcome](../../plans/41-simulation-pipeline-efficiency-and-effectiveness.md#outcome-recorded-after-implementation).
It composes selected S01–S10 local/native/managed/Python journeys, independent original
numerical assertions, causal work counts, cancellation/drain and nonempty retained result
reads. Initial failures and successful affected reruns remain explicit; this is not a fresh
whole-product campaign. The original 256 MiB observer and two compiler leases remain in
the managed read control. Primary loss yields operational unavailability with incomplete
science. This evidence neither replaces the historical full basis nor completes Plan39/40
or Plan28 campaigns. No elapsed-speedup, arbitrary solver configuration or other-platform
claim is made; environment/provenance proof is permanently eliminated.

Focused post-Plan41 correction acceptance belongs to the
[Plan42 Outcome](../../plans/42-post-plan41-simulation-efficiency-and-diagnostics.md#outcome-recorded-after-implementation).
Both original ordinary caller-stack result journeys, selected preparation/fit/shooting/managed
journeys, reader pressure/lifetime and installed analysis interruption/recovery pass with their
original scientific/resource assertions and registered cleanup. The scope-end workspace check,
repaired hygiene/three continuing Clippy modes, depth-two feature/no-default checks and governance
pass. Earlier failures retain their actual conditions. This supplies corrected selected evidence
to Plan39/40; their independent final campaigns and other unexercised breadth remain separate.


Completed selected Plan39/38 qualification of 2026-10-17 belongs to the
[Plan39d Outcome](../../plans/39d-qualification-and-plan38-cutover.md#outcome-recorded-after-implementation).
It composes the original full native selection with affected successful repairs, 272 ordinary
and nine managed installed-Python cases, both physical thousand-point variants, all eleven
once-only simulation positions, actual entered-pull SIGINT, scientific process-crash/offline
restore/rebuild and nine IDAES comparisons. Exact native/file/receiver drain and original
scientific assertions remain. All 28 final hygiene checks, default/no-default/linked-native
continuing Clippy and governance pass; current documentation/metadata and turn-end results
remain in that Outcome. Initial failures, interrupted runners and repaired reruns retain their
actual boundaries. This is composed selected acceptance, not a fresh monolithic whole-product
rerun, statistical speedup, exhaustive feature powerset or qualification of Plan40/28/32 scope.
Environment/provenance proof and repeated comparative benchmarking are permanently eliminated.

Selected Plan43 solver/diagnostic acceptance of 2026-10-17 belongs to the
[Plan43g Outcome](../../plans/43g-qualification-and-design-publication.md#outcome-recorded-after-implementation).
It records independent mathematical/product/guard controls, native conformance and
workflow/dynamic/fitting journeys, installed ordinary/managed Python consumers, coherent
inspection/query views, actual bounded Trace Processor consumption and resource drain.
Continuing default/no-default/linked-native Clippy, repaired hygiene leaves, governance,
native contract/data checks and the depth-two feature matrix retain their exact commands
and composite repair boundaries there. The optional compiled route has its separately
selected qualification. This is selected assembled acceptance, not a replacement of the
historical full basis, certification of the whole physical model library, a statistical
speedup or an all-platform campaign. At that checkpoint, an external-report metadata error blocked
full-book publication; authored-scope metadata/native tables/links/rendering were a
separate check. Older plans and formal proposed ADR statuses are not reassessed.
Environment/provenance proof and compulsory repeated benchmarking remain eliminated.

Selected Plan44 acceptance of 2026-10-17 belongs to the
[Plan44f Outcome](../../plans/44f-integrated-acceptance-and-follow-up.md#outcome).
The integrated scope includes 86 native-local scientific/resource/diagnostic controls,
eleven actual compiled-evaluator controls, one canonical exact-manifest reopen,
three managed publication/continuation/cancellation-and-drain controls, and four local
plus four durable public Python controls. Fast editable native correctness builds were
used; Arrow-consuming correctness closures explicitly activated force-validation.
The 38-case compact resweep, four matched output-density observations and four corrected
retained-result/publication observations retain their different populations and once-only
conditions in Plan44e. Scope-end static/publication results retain their exact commands
and composite repair boundaries in Plan44f; no full wheel, whole-product/parity rerun,
statistical speedup or environment/provenance attestation follows.

Conditional combined shooting quadrature products and shared immutable inner programs
remove tested unnecessary work while preserving independent original assessment,
mutable-owner isolation, accounting and the ordinary 0.001 engineering fraction.
Integral-only/preflight adjoints, lazy trial defaults, generic stateful-provider barriers,
conservative foreign bounds and explicit numerical profiles remain. Repeated KLU symbolic
setup is a confirmed deferred limitation, with a material-cost trigger and safe-owner
prerequisites in Plan44f; investigation completion does not resolve it. The supplied external
report's authorized title/front-matter/H1 repair permits full-book publication. Formal
proposed ADR status and older plan reconciliation remain separate.

Selected Plan45 qualification of 2026-10-18 belongs to the
[Plan45f Outcome](../../plans/45f-short-run-qualification-and-publication.md#outcome-recorded-after-implementation).
It composes typed/persistent source and historical preparation, inspected mathematical
programs, full/selected durable results, protected artifact/analysis transfer, bounded
physical groups, consumed adjoint products, corrected IDAS quadrature histories and
provider/recycle controls through selected Rust/native/public Python and actual managed
journeys. The retained evidence distinguishes original failures, corrected test setup,
not-run commands, subsequent passes and normal owned drain. Full continuing Clippy passed
default/no-default and linked-native workspace/all-target modes; the scoped 31 feature
combinations retain explicit warning-repair boundaries. These are affected-scope checks,
not renewed whole-product/parity or release-wheel qualification.

The four result-growth shapes are recorded-evidence fixtures; the unchanged complete
80-state/five-time publication case separately exercised scientific execution and original
result/locator/retirement assertions. Exact work-count qualification rejects lost or
refused observations; its isolated growth fixture joins the independent periodic publisher
before measurement while keeping collection and strict coverage checks enabled. Ordinary
physical accuracy remains 0.001 and independent gradient oracles remain separate. Inclusive
clocks and once-only contrasts do not establish statistical speedup or exclusive CPU cost.
The later Plan45 A06/E05 follow-up implements consumed physical reuse and private fixed-pattern
KLU symbolic retention. Its combined twelve-control native/edit journey passed with original
checks and actual current/historical owners; the [reopened 45f acceptance](../../plans/45f-short-run-qualification-and-publication.md#reopened-f05-acceptance-2026-10-18) records repaired zero-target scope-end checks, affected feature combinations and governance/generated equivalence.
Scientific-witness/direct implicit and unclassified dependency paths remain explicitly
complete-context, and generic mutable KLU keeps its original supported lifetime. R-54/R-53
are resolved and removed from the deferred register. Unsupported composition/JIT/projection/checkpoint alternatives, arbitrary large
relation atomic closure and oversized optional-cache retention are not claimed.

Selected Plan46 acceptance of 2026-10-18 belongs to the
[Plan46g Outcome](../../plans/46g-investigations-qualification-and-design-publication.md#outcome).
It covers effect-safe selected implicit products and compact mathematical execution,
primal-only root transformation with full original assessment, exact response ownership,
completion-driven conditional work and conservative child permission, pre-allocation dynamic
checkpoint limits/direct actions, incremental lexical source products, live capture/publication
and independently protected result payloads. Native and joined canonical/compiled controls retain
original numerical, cancellation/drain and publication assertions. The initial 97-test runtime
selection had one selected-start fixture failure; the affected 11-test rerun passed after an
explicit missing-start refusal control and the required authored start were supplied. That
composite result is not a second clean all-97 run. The Outcome owns all selected commands and
scope-end results, including failures and repairs.

Short KINSOL/FERAL/PETSc, DAE/checkpoint, fitting affinity and decoder comparisons preserve
ordinary 0.001 physical accuracy where selected and separately meaningful derivative/rank
checks. They select no universal solver or speed SLA. Stronger coupled matrix-free DAE
preconditioning, cross-sample batching/parallel protected decoding, strict opaque-fill producer
eligibility and general cost-aware experiment assignment retain explicit limits and triggers
with their current owners. Source-backed no-adoption decisions for inner teams, SIMD/JIT,
PGO and GPU do not assert absent benefit. This affected-scope acceptance does not replace
the historical full basis, rerun all parity/platforms or close other active campaigns.
Environment/provenance proof remains permanently eliminated.

Selected Plan47 diagnostic acceptance of 2026-10-19 belongs to the
[Plan47f Outcome](../../plans/47f-qualification-and-design-publication.md#outcome).
It composes original joined scientific owners, acknowledged CLI recording/handoff,
selected protected publication and relocated frozen readback. An actual admitted native
workflow was held pending while copied review ran and only its copied reader was cancelled;
the original workflow then joined correctly. Current managed inventory is checked against
actual stored run/attempt keys; historical retry, independent-clock and failure combinations
also use explicit copied controls. Optional original initialization/recycle context and joined
dynamic segment statistics remain independent of scientific success and compact retention.
Short off/ordinary/detail observations preserve original outputs and counter populations;
finite growth/skew controls establish selected behavior, not statistical overhead, total
private peak memory or a capacity SLA. The Outcome owns exact commands, zero-target repairs,
static/manual results and unexamined routes. Numerical defaults and practical accuracies
remain unchanged; this supplements the historical basis rather than requalifying every
scientific model, native library, platform or distribution.

Selected Plan48 execution-performance acceptance of 2026-10-20 belongs to the
[Plan48h Outcome](../../plans/48h-qualification-and-design-publication.md#outcome-recorded-after-implementation).
**Implemented and Tested, selected scope:** indexed source/preparation and typed receiving,
declaration-derived registry hydration, exact selected mathematical products, bounded scalar
modules/cache, native stagnation/refinement/start controls, fixed-profile IDAS value/forward
storage, independent fixed-trial windows, qualified fitting residual LM, and finite
compute/publication handoff compose with current original assessment and ownership. Their
contract owners are §§4/5/13/14/18/19/20/23; proposed ADR-0201–ADR-0212 retain their formal
status independently of implementation acceptance. Unsupported reset, backward/Second
storage and unknown producer bounds retain fresh execution or conservative coverage.

The selected native/Python journeys include actual fitting foreign populations and exact
pool balance after enclosing job drain, authored shooting, cancellation, publication failure,
ordinary-stack durable studies, current public strategy receiving, edits and relocated
review readback. Composite repaired selections retain their initial failures rather than
claiming a second whole-selection pass. Selected covariance/Parmest and sensitivity/sIPOPT
comparisons pass against the pinned IDAES reference. Continuing Clippy passes default,
no-default and full linked-native/canonical/compiled workspace/all-target modes with zero
warnings/errors. Ruff, Pyrefly, generated equivalence and governance pass after the
continuing hygiene bundle's reported repairs; an affected no-default Diffsol-only canary
also passes. The Outcome owns actual commands, counts, scope and exclusions.

**Measured, bounded conditions:** package controls record reduced selected-entry construction,
registry hydration, compiler children, redundant initialization and compatible native
allocation, with their actual samples and scientific checks. These establish the described
work removal, not a universal timing winner or statistical/industrial capacity claim.
Qualified first-principles work reduction may be deployed despite marginal timing differences;
unexpected composed costs prompt causal diagnosis. Practical physical accuracy remains 0.001;
native local tolerances do not themselves certify global physical-output error. No full
feature powerset, all-case campaign, new-platform or release/distribution qualification is
claimed. Environment/provenance proof remains permanently eliminated. Other plans retain
their independent scope and status.

Selected Plan49a canonical-store acceptance of 2026-10-21 belongs to the
[Plan49 store Outcome](https://github.com/paul-heyse/pse-arrow/blob/197616768b1b7abcbe67af2b5b8d42ab445b291b/docs/plans/49-library-native-remediation.md#outcome--plan49a-canonical-store).
**Implemented and Tested, selected scope:** source-built SurrealDB 3.3.4 with exact
SurrealMX 1.0.0, explicit SSI and synchronous persistent AOL, the single bounded gRPC path,
registry-generated physical lifecycle/refusals and native record/graph access operate through
one supervised authority. Fresh data replaces the retired store; no historical import or
old-AOL rollback is qualified. New-data restart, interrupted initialization, exact protected
history and closed backup/restore remain tested obligations. Proposed ADR-0214 retains its
formal status independently of implementation acceptance.

Selected controls include sixteen actual native owners, independent workers and continuation
after submitting-process exit, retained-client crash/reconnect without mutation replay,
Python result/study consumers, and entered pull/analysis SIGINT with original exception/cause
and physical drain. Native fsync EIO, process death, torn tails, interrupted snapshot
replacement, finite memory refusal and corrupt framed-log startup have distinct fault models.
Authenticated existing metrics expose held reader pins and drainage; anonymous metrics omit
storage gauges. The Outcome owns exact commands, zero-baseline failures/repairs and static
results. Composite affected reruns do not imply a fresh whole-suite pass. Default,
no-default and linked-native/canonical/maintenance all-target Clippy, the repaired hygiene
leaves, governance, 419 depth-two feature combinations, 33 no-default package checks and
final Rust documentation pass against zero warnings/errors. These static checks do not
supply unexercised scientific or performance qualification.

Necessary source-creator/reclaimer and commit-order guards remain, as do attributed ENFORCED
relations and high-degree fallback. Output-page bounds do not bound engine scans. No
hardware power-loss, statistical speedup, whole scientific-model library, new-platform or
release/distribution qualification is established. The historical full basis and independent
Plan28/32/40 campaigns, other Plan49 packages and their proposed decisions retain their scope.

Selected remaining Plan49a–49c acceptance of 2026-10-22 belongs to the
[remaining-scope Outcome](https://github.com/paul-heyse/pse-arrow/blob/197616768b1b7abcbe67af2b5b8d42ab445b291b/docs/plans/49-library-native-remediation.md#outcome--remaining-49a49c-2026-10-22).
**Implemented and Tested, selected scope:** the same Rust study policy runs read-only through
Surrealism inside bounded native readiness transactions, with complete output admission before
writes and immutable original intent settlement. External scope/apply is retired. Actual-server
controls cover full-u64 decisions, stale/fresh facts, notification/cursor/epoch fences, committed
policy refusal, uncertain cancellation on both sides of commit, trap/timeout/memory rollback and
subsequent recovery. A separate qualification artifact certifies sixteen actual guest hold
intervals throughout typed global-capacity refusal, including cache eviction and post-drain reuse.
Production keeps its strict 1s/64MiB/pool 4 manifest, global 16 controller bound and cache 4; the
5s/6s qualification-only artifact is absent from the final active deployment. Cold initialization
and synchronous host work are finite but not instantly preemptible. Public open refuses missing
module and tampered read-only backend; authenticated initialization/stop/drain/restart passed.

**Implemented and Tested, selected scope:** joined protected result pages and one selection-local
native predicate/pruning plan replace scalar locators, public legacy block paging and repeated
physical planning. Full file/batch admission, conservative IEEE/null pruning, residual-before-limit,
original coordinates, escaped Arrow ownership, protected retirement and durable reopen remain.
The current two-file public entered-pull SIGINT control preserves the original exception/cause,
retains response ownership while the owned server is paused, and releases the exact ticket and
results after joined pull and server resumption.
Direct resolved bodies, generated semantic handles, complete dependency indexes and query-owned
local fragments replace rendered reconstruction. Final root scientific obligations remain.
Salsa 0.28.5 exclusive nullable accounting plus shared-owner census supports finite 131,072-entry/
2GiB known-retention defaults; complete current reference coverage/backdating remains generation
zero. This accounting is not process RSS. PRE09 concludes no new configuration feature or Z3
binding for the bounded supplied-profile inquiry; RC23 remains unselected.

The Outcome names the 738 pure-library,51 selected native/scientific,15 readiness,3 module-capacity/
catalog and8 assembled Rust controls,52 Python contract controls and composite six-plus-one
public Python journeys, their actual commands and initial failures/repairs. Scope-end default,
no-default and linked-native all-target Clippy, repaired hygiene leaves, generator/family checks
and manual publication retain their stated scopes. The complete depth-two feature matrix
passes 420 combinations plus 34 no-default package checks with zero warnings/errors, including
post-repair tooling coverage. A bounded complete source/publication/
simulation/selection measurement records the current target with competing compiler activity;
it establishes no controlled speedup. These selected checks do not close whole 49d–49h, the
canceled native sweep, broader model-library/parity campaigns or independent plans. Proposed
ADR-0214/0215 remain formally proposed; older unknown creation effects remain unresolved.

Selected remaining Plan49d–49e acceptance of 2026-10-22 belongs to the
[math/runtime Outcome](https://github.com/paul-heyse/pse-arrow/blob/197616768b1b7abcbe67af2b5b8d42ab445b291b/docs/plans/49-library-native-remediation.md#outcome--remaining-49d49e-2026-10-22).
**Implemented and Tested, selected scope:** demanded library regions compose all five local
derivative products across provider, branch and intermediate boundaries. Interpreted/C++ controls
retain guards, canonical inputs and independent analytical/action checks. Actual library AST,
literal, Taylor-width and exported-storage admission replaces source-byte population estimates;
sparse stage descriptions charge reached occurrences. Constructor allowances and opaque foreign
envelopes remain. Tracked immutable package projections preserve cross-parent equality, A/B/A
reuse, canonical physical owners, source attribution and finite retention.

Shared run supervision preserves the original clock, downward cancellation, native stop flag,
partial horizons and actual destruction/join before release. Typed requests and outcomes,
runtime admission/deployment, parameter-valued Simulation/Fit points and compiler-owned policies
serve their Rust/Python consumers. Native map routes retain fresh sessions, authorized starts,
scaling and original residual qualification. Uno retains its declared SQP-LBFGS integration;
FERAL estimates supplement post-factor envelope enforcement. Retained surrogate workers join
their actual threads. Independent LM criteria do not replace physical stationarity or covariance
qualification. Complete LM eligibility and staged covariance withholding take explicit actual
facts, with Runtime-free policy tables and retained native scientific controls; prepared-problem
adapters retain their ordered typed refusals and original numerical assembly.
Conic/recycle/block-initialization strategies remain explicitly nondurable;
Shooting remains host-only.

The Outcome records final selected physical and public Python journeys, interpreted/C++,
compiler, runtime and native controls, failed runs and composite repaired selections. The final
six-test reference selection passes five sixteen-owner lifecycle/publication controls and isolated
single-owner primary death, followed by normal quiesce and drained shutdown. Four current installed
Python journeys supplement the earlier 51-case assembled selection after the final repairs.
Fresh/preinitialized-host controls do not authorize resizing a host's global pool. Known-byte
accounting and future layout are not process RSS; no controlled speedup, universal bit equality,
whole-model-library/parity, new-platform or distribution claim follows. Independent 49f–49h and
broader campaigns remain open, and ADR-0215 remains formally proposed.

Selected Plan51 engineering/research/evidence, lifecycle and portable-project acceptance of
2026-10-23 belongs to the [Plan51g Outcome](../../plans/51g-qualification-and-publication.md#verification-and-outcome--assembled-execution-2026-10-23).
On Linux with explicit force-validation, the 100-case Rust selection passed 73 cases at the
default 16 concurrent tests; 27 canonical fixture startups failed. The exact affected 27
binary/test instances then passed at four concurrent tests with unchanged deadlines/assertions.
The cause is unproven; this composes selected functional evidence and does not qualify startup
at 16 concurrent tests. Fourteen installed native-Python component journeys and four affected
frozen-context unit controls passed. The actual calibrated fit/publication/import/edit/downstream
study retains independent original numerical assertions, not database-only qualification.

The 322 selected host controls, affected final health controls and actual disposable application
backup/recovery exercise nonempty project/draft inventory, incomplete closure refusal, crash/reopen,
protected cells, fresh restored authority/credentials and semantic validation. Ten native metric
families were observed on the disposable service. The primary's metric response exceeded the finite
capture bound: service/cgroup identity remained observed and metrics unavailable with that precise
reason. No prefix parsing, larger bound or causal memory/performance inference follows.

Default, no-default, full linked-native workspace and explicit changed optional-tool Clippy graphs,
repaired Python/ADR hygiene, generator equivalence and governance pass at the conditions in the
Outcome. Final feature-matrix/publication state remains at that owner. This evidence does not replace
the historical full basis, qualify a fresh whole-product/parity/scientific-recovery/distribution
campaign, establish confidentiality or a speed/capacity benefit, or close Plan32/40/49 obligations.
Proposed ADR-0216–0220 formal decision status is unchanged.

Selected remaining Plan49 scientific-boundary, tooling and qualification acceptance of
2026-10-23 belongs to the [final Outcome](https://github.com/paul-heyse/pse-arrow/blob/197616768b1b7abcbe67af2b5b8d42ab445b291b/docs/plans/49-library-native-remediation.md#outcome--remaining-49f49h-2026-10-23).
The linked force-validated Rust selection initially passed 70 of 73 controls; two numeric
EXPLAIN-limit assertions and a dense-fixture operation identity failed. After the literal-cap
and fixture repairs, all three affected live controls plus six pure graph controls passed.
This composes the selected 73-case scope; it is not a clean rerun of the original command.
Actual generated maintenance covers protected retirement, 65-guard pagination, exact replay
across generation handoff and empty terminal completion. Graph acceptance covers all seven
neighborhood scopes, version-first operator transport and actual OWNER/VIEWER execution.

Thirty-two installed native-local Python controls pass keyed Arrow custody, generated
vocabulary/identity/version transport, conformance and independent covariance propagation.
The three affected scientific integration tests pass after a single-target revision fixture
repair. Three canonical component journeys pass by composition: actual source-command and
protected-selection journeys first, then the corrected reopen/fit-covariance journey. Twelve
public review workflow units and 43 host maintenance/probe-runner controls pass. These scopes
remain distinct; ephemeral tests do not establish canonical-store behavior.

An exact pinned-source test-only scanner observer passes under both root roles: numeric
budgets 65 and 1,001 return exactly 65 and 1,000+1 neighborhood index entries, while the
uncapped control scans 2,400. Native planner/index/scanner source equivalence is retained with
the probe; this does not establish whole deployed-binary equivalence. Study-anchor validation
is separately bounded. No MVCC-version, disk-I/O or latency bound follows. Original engine,
SSI, transport, sixteen-owner and Plan51 evidence retain their original conditions. No canceled
sweep, fresh whole-product/parity, exhaustive feature-matrix, release-wheel, new-platform,
throughput or RSS claim follows. Other plans' campaigns and formally proposed ADRs remain
independent. Default/no-default/full-linked-native Clippy, Pyrefly, repaired Ruff/license
checks, generated equivalence, governance, environment and book publication pass. All 99
depth-two feature combinations of the changed operations/tooling roots pass with explicit
validation; this is a scoped matrix, not the entire workspace powerset. Exact commands and
historical failures remain at the final Outcome.

Selected remaining Plan52 MCP/observability acceptance of 2026-10-25 belongs to
[52h's Outcome](../../plans/52h-qualification-and-publication.md#verification-and-outcome--remaining-mcp-scope-2026-10-25).
**Implemented; Tested at the named scopes:** exact same-cut native evidence comparison and
selected sections, actual continuous/finite Python/FastMCP→Rust→SDK→server→Surrealism sibling
parentage, per-call study claim custody, current-artifact/full-wire admission, independent
endpoint cleanup and representative actual-agent workflows have their native/host consumers.
The linked sixteen-case lifecycle/claim/evidence/child-drain selection passed with explicit
validation. Host and Python functional qualification is composed from the original full
selections and successful affected repairs; it is not an initially clean monolithic rerun.
Default/no-default and linked-native/all-target Clippy, repaired hygiene leaves, Ruff/Pyrefly,
generated equivalence and governance passed at the conditions in that Outcome. The completed
`CARGO_INCREMENTAL=0 just assessment --group features-powerset` passed all 505 depth-two and 34
no-default configurations, with zero failures and unchanged relevant Rust/compiler inputs.
The enclosing thirteen-document source drift is retained as broader provenance; original gate
receipts record no relevant source/environment changes. This is complete selected build-feature
qualification, separately from scientific testing. Final publication and historical failures
remain at the owning Outcome.

Actual Codex recovered current native schema paging and protected manifest continuation after
exactly one explicit selected-endpoint preparation. The deeply nested study relation's mandatory
context descriptors exceeded the readable inspector's 64 KiB bound before values were acquired;
its selected row cells remain unknown. The unsuccessful enclosing values assessor remains failed.
Typed refusal, bounded positive schema recovery and prior different-source wide-result acceptance
remain distinct. Telemetry query visibility and capture/restoration do not prove native success,
complete delivery, causality or physical drain. Owned original services/contexts settled separately.
C04 uses the accepted ordinary structured fallback; coverage-guided/sanitizer acceptance and the
historical intermittent decoder-crash cause remain unqualified. Conditional production CodeMode,
protocol tasks, Apps/UI and batching remain unselected for delivery; the maintainer retained the
ordinary baseline and deferred those additions under R-55. Inquiries do not deploy them. No new whole-product, full solver-fleet, parity, overhead/performance,
distribution or other-platform qualification is inferred. Other plans and proposed ADR decisions
retain their independent authority.

Selected Plan55 developer-interface acceptance of 2026-10-25 belongs to the
[Plan55f Outcome](../../plans/55f-qualification-and-publication.md#verification-and-outcome--2026-10-25).
Local Linux qualification composes 2,277 setup cases, 50 rebuilt public-native boundary unit
cases, selected force-validated Rust/model/result/knowledge tests and actual canonical MCP/CLI
journeys. It includes foreground edit custody, attributed indexed source/evidence inquiry,
model declaration/inspection/comparison, original retained investigation reopening and one real
protected nested LP outcome value. The actual value survives the original public-native and MCP
JSON paths, while deferred composite logical-type metadata keeps context/readable completeness
false and scientific usability unknown. This does not establish complete physical interpretation
or unexercised scientific correctness.

Default/no-default/linked-native Clippy, repaired hygiene leaves, generated/public-stub equivalence,
governance, native contract/data and documentation checks pass at the Outcome's conditions. The
initial failed static/test/controller runs and successful affected repairs remain distinct. The
original selected historical fixture was unavailable and preserved; an explicitly owned fixture
supplied the passing canonical journey and normal final stop/drain/retirement.

Guarded Codex app-server reload queued a global refresh, and a subsequent actual Codex attachment
used current model-exposed tools with correct checkout/interpreter and no restart requirement.
The root's already-active older attachment is a separate observation; fresh clients do not prove
in-place refresh. No automatic replay or hot reloading is established. Completed optional
experiment/query-packet inquiries retain existing direct readers and concrete-consumer deferrals.
No fresh exhaustive feature matrix, whole solver fleet/parity, distribution, other platform,
backend performance/causality or productivity/speed benefit was qualified. This selected evidence
does not replace the historical full-product basis. Proposed ADR-0224 remains proposed.

Selected Plan56a model/physical acceptance of 2026-10-25 belongs to its
[Verification and Outcome](../../plans/56a-model-meaning-and-physical-interpretation.md#verification-and-outcome--2026-10-25).
**Tested:** exact occurrence/binder admission, compiler mode/support/view products, affine inventory
and integral scaling, typed physical alias/Arrow boundaries, and original retained physical result
interpretation pass at the recorded local Linux conditions. The composed canonical scope passed
sixteen controls, then its repaired adverse-receipt control passed both missing and wrong-kind
branches. Actual correctness Python, MCP and derived CLI expose the same physical projection;
the stored-result integration preserves original context and governing permission separately from
unknown selected-product usability. Two public fitting journeys also passed. Failed attempts and
successful affected reruns retain distinct evidence in the Outcome.

Scope-end checks and repaired leaves passed against zero. The separate owned functional deployment
was quiesced, authenticated-drained and stopped with no surviving owned process or listener;
its database/receipts remain retained and the preexisting selected store was preserved. This is
selected model/physical qualification, not the remainder of Plan56, a fresh feature matrix,
full parity/solver-fleet, distribution, other-platform or measured performance evidence.
ADR-0226 remains proposed; RC01–RC06 are not adopted.

Selected Plan56b scientific-operation acceptance of 2026-10-25 belongs to its
[Verification and Outcome](../../plans/56b-scientific-operations-and-permission.md#verification-and-outcome--2026-10-25).
**Implemented/Tested:** original/product permission, complete profile endpoints, exhaustive kind
facts/tagged replay, explicit service/binding/issuer boundaries, configured native capability and
typed failure/effect remedies are substantially adopted through local, durable and public consumers.
The [56b completion audit](../../plans/56b-scientific-operations-and-permission.md#completion-audit--2026-10-27)
identified real operational producer constructors classified as invalid model. The authorized
repair now preserves typed poison, allocation and worker-join causes, with affected owner controls
passing. The earlier consumer controls retain their original scope. Local Linux
pure controls, actual native fit/profile/KINSOL, protected canonical hydration/replay and installed
correctness Python passed at the recorded scopes. Actual STDIO MCP and derived CLI agree with
Python on original failed permission while diagnostic check bytes remain readable. Initial Rust
and Python integration failures and exact affected reruns retain distinct evidence; no single
clean full selection is inferred.

Scope-end generation, compiled API stubs, default/no-default/supported native lint, governance
and repaired hygiene leaves passed against zero. The full feature powerset was accidentally
started, interrupted and authenticated-drained; its incomplete receipt remains, with no matrix
qualification claim. Four new functional/no-resident deployments were initialized and qualified,
then quiesced, authenticated-drained and stopped with databases/receipts retained. The preexisting
selected store was preserved. This acceptance covers the named B and necessary C/F slices, with the later audit-gap acceptance recorded separately, not the
remaining Plan56 scenarios, full parity/fleet, desktop-host attachment, distribution, other
platforms or measured performance. ADR-0227 remains proposed; RC01–RC06 are not adopted.

Selected Plan56c canonical/operational acceptance of 2026-10-26 belongs to its
[Verification and Outcome](../../plans/56c-canonical-contracts-and-effects.md#verification-and-outcome--2026-10-26).
**Implemented/Tested:** declarations, exact attributed settlement, v9 inline/sealed placement,
product native file authority and controlled fresh activation are integrated. The
[56c completion audit](../../plans/56c-canonical-contracts-and-effects.md#completion-audit--2026-10-27)
identified a lazy study association-prefix consumer (C01) and lost failed-readback cause in
source-stage acquisition (C02). Authorized repairs consume retained declared associations and
preserve both original causes as unresolved settlement under the original parent deadline.
The source transport and affected owned-v10 lazy/canonical controls pass; their exact conditions remain with the closure Outcome. Canonical 49/49,
final native runtime 83/83, affected installed Python/physical readers and actual STDIO MCP/CLI
passed at the recorded scopes. A private backup contained ten inline bodies and five sealed
instances; all 345 copied members were independently verified. Corruption refused before target
creation. SIGKILL after durable restore publication retained closed admission through three
refusals, then actual recovery, closed exact validation, explicit readmission and reopened reads
passed. Eight original tables and inspection/Completion/context facts matched exactly.

Original failed invocations, repaired affected checks and composed scope-end results remain
explicit in the Outcome. Shared-context backup now refuses before freezing, matching existing
private restore authority. Owned test/private/restored services are stopped with data and receipts
retained; the selected existing store and prior 56b evidence are preserved. This is selected
local Linux acceptance, not full-series D/E/F/G, shared-context restore, desktop-host attachment,
trace export, a fresh feature powerset, parity/fleet, distribution, other-platform or measured
performance qualification. ADR-0228 remains proposed.

Selected Plan56d/56e local acceptance of 2026-10-26 belongs to the
[56d Outcome](../../plans/56d-publication-and-workload-fit.md#verification-and-outcome--2026-10-26)
and [56e Outcome](../../plans/56e-product-deployment-and-resource-ownership.md#verification-and-outcome--2026-10-26).
**Implemented/Tested:** D01–D06 complete selected grouped publication, resumable population,
compact outcomes, coherent capture/provider, fused offline integrity and incremental exchange.
The actual public 10,000-point journey and four other capture/observer controls pass 5/5;
completed-parent/history controls pass 4/4, and installed correctness Python/CLI/actual STDIO MCP
passes 8/8. Initial native 78/82 plus corrected 4/4 remain composite evidence. The 100,000 ceiling
is unqualified. One-GiB sealed opaque bodies plus eight inline bytes survive SIGKILL/reopen,
backup/restore and explicit interrupted-copy resume; descriptors and bytes match original custody.
This establishes storage behavior, without numerical-success or comparative-speed claims.

**Implemented/Tested:** E01–E05 deliver responsibility-owned declarations, product/development
partition, upstream production profiles, product roots and frozen receiver closure, shared
observations/borrowers, original-owner orphan drain and truthful supplier records. The preserved
legacy v2 service was authenticated and drained by its original owner; only its unchanged dead
allocation row was reconciled. Data, frozen interpretation and receipts remain retained.
Actual current reference initialization, frozen helper/normal worker launch, workload, quiesce,
drain and cleanup passed with sixteen scientific lanes and the existing logical-CPU/profile
allocations. This establishes deployment/resource behavior, not sixteen numerical solve outcomes.
See [56e's final activation](../../plans/56e-product-deployment-and-resource-ownership.md#final-activation-outcome--2026-10-26).

**Implemented/Tested:** selected F slices deliver finite operation/default/effect/prerequisite
projections, keyed metric semantics, typed remedy consumers and instrument-correct questions,
explicit export stage and precise SDK loss observations. The
[completion audit](../../plans/56-design-principle-alignment.md#completion-audit--2026-10-27)
withdrew all-series closure. Authorized repairs now adopt the telemetry sampler/bounds and evidence
comparison enums/map keys through generated schemas and actual host consumers, honor explicit stage
compatibility and remove automatic CLI source scans. The affected contract/host controls and both
actual two-generation/reload journeys pass, with original session-family, enclosing allocation and
kernel-scope settlement separately established. Passing controls below retain their original scope. Controlled constructors publish immutable artifact
generations once. Supplied-only executable/MCP/CLI evidence needs no product extension or canonical
store. Ten actual native attachment/context/CLI/cancellation/EOF/health/capture controls pass;
mutable installation-alias replacement preserves both an attached generation and fresh attachment.
Three actual canonical project/knowledge/protected-result cases, two original-cursor/protection
adverse cases and three installed project/Python CLI journeys pass. The disposable canonical
fixture's original lifecycle reports all contexts and kernel scopes drained and its owned state
retired. Native image attachment avoids image/source/build-record revalidation. Automatic CLI admission
uses the loaded contract as described above.

[56f's closure Outcome](../../plans/56f-host-agent-and-observability-contracts.md#audit-gap-closure-outcome--2026-10-27)
and [56g's closure Outcome](../../plans/56g-qualification-and-publication.md#audit-gap-closure-outcome--2026-10-27)
retain source cuts, commands, failures, repairs and exclusions. Independent review found and
resolved strict deployment selection, interrupted-publication cleanup, concurrent-close/successor,
clock-wrapper destruction and inherited-charge oracle gaps. Current repair qualification includes
142/142 affected Python controls, 7/7 final operations/math controls, 2/2 owned-v10 lazy/intent
controls, three rebuilt canonical cases and both actual CA08 journeys. The root's original
controller exited zero with all contexts/kernel scopes drained and positive disposable-fixture
retirement. The old selected store and failed-run data remain preserved.

Plan56's final full hygiene attempt recorded six unsuccessful leaves; all affected leaves were repaired and
passed separately. Default/no-default Clippy, declared native contract/data static recipes,
seven-leaf governance, actual post-lint/post-clean native CLI and documentation publication pass
against zero at their recorded conditions. The native-data ENOSPC interruption, cold rerun after
the maintainer's Cargo clean and earlier stack/refusal/oracle failures remain distinct evidence.
No clean aggregate is inferred. All selected Plan56 packages are complete through composed local
qualification; formal ADR-0229/0230/0231 statuses remain proposed. No 505-case campaign, full solver
fleet/parity, global feature powerset, release/wheels, remote CI, other-platform or actual
desktop-host qualification, 100,000-point qualification or measured performance improvement
follows from these selected local Linux results.

Selected Plan57a–c implementation and functional acceptance on 2026-10-27 is recorded in
[57j's Outcome](../../plans/57j-qualification-and-publication.md#selected-a-c-outcome--2026-10-27).
**Implemented/Tested:** current protected IPC/import/export and original-column selection,
bounded cubic/exact-proof preparation, source/arithmetic identity, SUNDIALS 7.9.0, LU/terminal/gate
and compiler/capture supervision pass their selected local Linux routes: 153 Rust case executions
across the declared modes and five composed Python cases. Independent reconciliation confirms
matching product inputs/artifact digests and original enclosing lifetime settlement; inner
unobserved drain fields retain their distinct scope. The current native correctness extension
was explicitly rebuilt/admitted. The owned fresh v10 fixture was quiesced/stopped and its
allocation retired; the earlier v8 state and original failure evidence remain preserved.

The hygiene aggregate recorded seven unsuccessful leaves, repaired and passed separately;
governance passes all seven leaves, and default/no-default/native lint and rustdoc checks pass.
No clean original aggregate is inferred. The maintainer canceled the remaining feature campaign:
179 configurations passed, one was interrupted and 321 were not run out of the current 501;
the separate no-default sweep did not start. Its source remains unchanged, but complete coverage
is false. Full matrix/no-default validation is explicitly deferred to the end of Plan57. The
original cancellation outcomes remain, and only the authenticated empty campaign allocation was
released. No campaign remains active.

These are selected functional/local checks, not whole-Plan57, full feature, solver-fleet/parity,
distribution, other-platform or performance qualification. Broader CBO-001 alias-header and
unestablished tighter whole-Jacobian/opaque bounds remain open. General non-RNE coefficient
conversion retains its hold and exact/current-numerical proof hull; recorded-point replay is
not an arithmetic waiver. The IBEX exception and retained supplier triggers remain. Benefits
beyond implemented structural removal are unmeasured.

### 24.3 Benchmarks

> Decision: [ADR-0160](../../adr/0160-centralize-testing-responsibility.md)

Design-phase measurements run the declared cases and retain their actual samples, scientific
assertions, execution conditions and cleanup outcomes. A functional report may be referenced
for context; identical source, environment, native bytes or artifact provenance is not a
prerequisite. Differences are observations, not automatic invalidation. Actual command/test
failures, invalid samples and unsafe ownership or drain still fail. Existing runners own case
selection; no second test-name manifest or proof campaign is required. Comparative performance
claims still require a controlled comparison.

Criterion benchmarks live in `benches/` (`pse-benches`):
`native_cache`, `native_consolidation`, `native_process` (complete-process cases) and
`modeling_preparation` (specialization, lowering and derivative programs of the seed
PC-SAFT state at a stated number of components and of the Peng–Robinson density root
under inline and nested realizations; nothing is solved, and workloads are registered in
`.config/preparation-cases.json`).

| Command | Purpose |
|---|---|
| `just bench-smoke` | runs every benchmark once as a test; no timing |
| `just bench-cache` | native cache/reuse observations under the declared benchmark profile |
| `just check-native-process-bench` | compile the native process target; this does not execute or qualify a measurement campaign |
| `just bench-production` | production-equivalent native measurements without force-validation, in its declared isolated target directory, with outputs under the repository `build/` |

Measurement rules:

- There is no timing gate. A performance claim is labelled Measured and names the
  benchmark, profile, features, thread settings and machine conditions.
- Compilation is untimed setup; cold and warm costs refer to case preparation, rebuild
  and complete process operations, preserving Cargo and compiler caches.
- Compare like-for-like inputs. Report raw samples and uncertainty (mean and standard
  error, not a confidence interval unless computed), work counts and teardown.
- Pool reservations and process RSS are separate observations; neither attributes
  individual library allocations, and zero reservations after teardown does not mean
  the allocator returned pages to the operating system.
- A changed physical, numerical or lifecycle contract prevents a causal before/after
  speedup claim from older measurements.
