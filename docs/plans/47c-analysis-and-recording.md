---
title: "Plan47c: Analysis and recording"
status: complete
date: 2026-10-18
adrs: []
review_sources: [docs/design_review/reviews/design_review_pipeline-diagnostics-and-review-artifacts_2026-10-18.md]
doc_role: plan
doc_owner: docs/plans/47-pipeline-diagnostics-and-review-artifacts.md
doc_topics: [design, diagnostics, numerical-execution, performance]
---

# Plan47c: Analysis and recording

## Target and foundation

This companion turns selected observations into source-linked answers about preparation,
evaluation, numerical work, workflows and end-to-end execution. It supplies shared analysis and
recording capabilities to [offline publication](47d-offline-review-pack-publication.md) and
[CLI review](47e-cli-review-workflows.md). The [coordinator](47-pipeline-diagnostics-and-review-artifacts.md)
owns accepted RC01–RC03, finding dispositions and combined completion. This document contributes
to F01/F03; it does not create another finding ledger.

**Interface-checked:** the reviewed production foundation already has pure Rust operation and
invocation interpretation, versioned numerical-history/comparison policies, typed statement
strength and next observations; coherent detail pages; registry-declared Arrow projections;
selected detail with construction-time acknowledgement and separate managed grants; and finite
CPU/timeline capture. `scripts/diagnostic_trace.py` runs three declared queries but does not
retain the exact query/result context as a structured product. Current inclusive timeline
summaries are useful localization, not causal reconstruction. The
[analysis/publication inquiry](../design_review/evidence/pipeline-diagnostics-and-review-artifacts-2026-10-18/analysis-and-publication-capabilities.md)
records pinned library contracts and limits. The principal review §5 owns the originating
catalogues and §10 the evidence limits. Source foundations inspected for this plan include
`pse-runtime`'s `inspection_comparison`, `capture/evidence` and `capture/detail`,
`pse-diagnostics`' `inspection/performance`, and the trace adapter/SQL.

The target reuses those responsibilities. Rust owns numerical interpretation over immutable
admitted inputs; SQL owns relational selection/aggregation, and Python owns thin command and
rendering composition. Neither SQL nor Python defines a second numerical-progress policy.
[47a](47a-frozen-evidence-and-review-context.md) supplies A02 recorded anchors, A03 descriptive context, A04 actual related
inventory and A05 immutable member ingress; [47b](47b-native-diagnostics-and-causal-attribution.md)
supplies qualified owner facts and mappings. Analysis does not reconstruct an execution
environment, read mutable native state, query a store implicitly or run science to fill a gap.

The intended improvement is less bespoke investigation and fewer repeated decisions about
identity, populations and coverage. No elapsed speedup or observer-overhead bound is claimed.

<a id="c01"></a>

## C01 — Admit immutable analysis inputs and reuse typed projections

Accept selected immutable checkpoints, invocation documents, original detail pages and producer
declarations, capture documents, source maps and descriptive run context. A live consumer first
copies its selected inputs; a frozen consumer obtains them through 47a's explicit resolver and
recorded anchor. Both call the same interpretation owner. Live freshness and frozen source age
are distinct presentation facts; neither consumer changes the recorded clock/population.

Also admit selected existing engine Contract/Diagnostic observations: bounded plan/rule reports,
actual physical-operator metrics and operation completion/lifetime/drop/abandonment coverage.
Consume their actual retained report/trace owners and expose a copied export projection where
needed; do not add engine instrumentation simply to make a pack. `ObservationPolicy::Off`,
Contract instrumentation without plan text, and Diagnostic bounded plan capture remain distinct.
No policy is enabled implicitly, and absent engine observations are unavailable rather than zero.

Carry source identity, requested interval, actual covered interval, clock scope/uncertainty,
family/producer generation, original versus native coordinates, units/scales, selection,
sampling/loss, policy version and omitted inputs through derived outputs. Query eligibility and
analysis completeness depend on the particular question's required inputs, not on every possible
library observation being present. Unknown, disabled, inapplicable, unavailable, nonfinite and
zero remain distinguishable. Unsupported document/policy versions refuse interpretation without
modifying the original member.

Reuse existing registry projections rather than another event schema. Join source fields by the
whole event key `(operation, producer, generation, event_sequence)` and field ordinal; preserve
family sequence. Produce one logical event population before grouping field-expanded rows.
Duplicate identities refuse the derived view; raw evidence remains readable. A timestamp,
iteration number, family name or durable append sequence alone is not an event join key.
Retain the predecessor needed for counter deltas, detect resets/generation changes and missing
prefixes, and preserve empty events. Source/context joins must not multiply shared arithmetic
contributors or charge one factor to each substituted right-hand side.

Use existing Arrow/DataFusion filtering, projection, joins, grouping and ordered windows at the
existing query boundary. Explicitly choose ordering/frame semantics where duplicate timestamps
occur; filtering away a required predecessor changes the answer. Select before copying/hydrating
large source payloads. Output streaming is not a bound on blocking join/sort/window state or all
allocations: apply the existing admitted pool/cancellation boundary, inspect actual physical
plans for substantial queries and surface finite refusal independently of science. No persistent
diagnostic store, universal query language or new analytics engine is required.

**Targeted acceptance:** interleaved producers with equal timestamps/iteration values; duplicate
event identities; empty events; multiple fields per event; predecessor crossing the selected
window; reset/overflow/missing prefixes; nonfinite and unavailable categories; exact UInt64
values above 2^53; cancellation/allocation refusal. Assert known counts from independently
constructed source events, not expected results generated by the same projection.

<a id="c02"></a>

## C02 — Implement the ten cross-owner questions

Deliver named, bounded analysis operations over C01 inputs. Extend the current pure interpretation
and comparison owners rather than building an all-purpose causal controller. Every response
contains source-linked facts, hypotheses, limitations and optional discriminating next
observations. A next observation is advice; opening a response never performs it. Existing
selected numerical thresholds remain explicit interpretation policy, not convergence decisions.

| Question | Inputs and delivered answer | Required limitation / next observation |
| --- | --- | --- |
| Did an edit rebuild stable preparation or bind values? | Exact preparation/parent operations, construction/memo/lookup/rebind counters, product census and cold/repeat context; report observed construction versus binding | Invocation v2 excludes earlier preparation. Missing parent work is unknown, not zero; changed scientific/provider support can legitimately require preparation. |
| Was there more evaluator work or broader derivative demand? | Demand, completion/failure, actual evaluations, hit/projection/miss causes and product order/action identity; distinguish requested calls from executed work and order upgrades | Derivative construction is separate. Current provider effects and shared contributors cannot be erased by a cache hit or duplicated per authored alias. |
| Did each actual evaluation become more costly? | Matched completed count/duration populations under compatible product/input/start/reuse context; descriptive cost-per-completed-call comparison | Inclusive duration can include implicit work/contention. Observe nested work or CPU stacks before claiming intrinsic evaluator regression; unlike populations receive no normalized winner. |
| Is root/recycle progress dominated by trials, rejection or physical error? | Compatible coherent accepted/trial event suffix, raw-map values and actual physical error ratios/assessment where supplied | Raw ratios are not a contraction proof; a native search direction is not accepted physical displacement. Unknown executed physical pullback stays unknown. |
| Is dynamic work numerical difficulty or reconstruction? | Separate forward, sensitivity, quadrature, backward and replay work; setup causes, accepted/rejected steps and checkpoint usage, actual segment role | Requested directions/outputs are not executed internal vectors. 47b's local-error contributions are consumed only if qualified; they do not certify global trajectory error. |
| Which fitting/study dependency limits readiness? | Exact experiment/occurrence/attempt relationships, queue/admission/worker/drain intervals, requested responses and endpoint inventory | Nesting/overlap does not establish critical path. Retained current-attempt prefixes are not historical closure; request explicit frozen membership from 47a/47d. |
| Did factor recovery cause extra work? | Parent factor/path, issued/completed/failure/refactor/fallback/reorder/scaling/perturbation and substitution facts | Outer calls do not count inner retry/refinement. Matrix scope remains scaled/permuted/possibly perturbed; missing inner hooks prompt 47b's inquiry or optional profiling. |
| What does end-to-end waiting mean? | Actual operation relationships across preparation, admission/queue, supervisor join, worker/native, qualification, terminal, response and later access | Join wait can overlap native work; shared-flight waiters are not duplicate construction. Wall minus process CPU is not wait time; scheduler/off-CPU evidence is needed for a wait-cause claim. |
| Are completed runs comparable? | Selected products, original physical context, starts/reuse, effective options/queried defaults, work populations, coverage and actual clocks | Preserve existing comparison refusals and literal changed-context explanations. Missing metadata does not prove equality; unlike scientific outputs are not ranked. |
| Can a finding be followed to physical model terms? | Exact event/field through available native transform, original variable/row/occurrence/source, units/scales and original assessment | Aliases, eliminated/fixed variables, Schur/KKT/cone transforms need their actual mapping. No universal dual/stationarity pullback; fresh diagnostic evaluation is a separate explicit operation. |

Each operation specifies its minimum source set and reports a bounded answer when only part is
available. Missing a family does not prevent unrelated well-supported facts, but cannot support
an absence claim. Expose omitted source items/statements and the actual inspected scope so a
finite response cannot masquerade as exhaustive review. Keep source references resolvable through
47a/47d rather than storing another mutable work ledger.

Use selected engine observations in the applicable preparation, cost, waiting and result-access
questions to distinguish planning/operator/materialization work. Preserve operator/session and
attempt populations, native metric units and completion/drop scope; a partially consumed stream
or shared recorder lifetime cannot be presented as a complete invocation or exclusive CPU total.
Targeted fixtures cover Off/Contract/Diagnostic, bounded versus absent plan text, early stream
abandonment and a missing operator population alongside independently available numerical facts.

**Targeted acceptance:** a manufactured more-work/same-cost pair, same-work/changed-inclusive-cost
pair, unchanged-count incomplete pair, incompatible product/start pair, preparation-before-invocation
case, shared work with two waiters, trial versus accepted checkpoint, replay versus forward
segment, factor-with-many-substitutions and unmappable physical coordinate. Check both a supported
answer and an explicit limitation/next observation for each question. No numerical solve is needed
for these immutable-input tests.

<a id="c03"></a>

## C03 — Compose selected recording without changing its authority

Make the existing selected detail and capture mechanisms consumable as coherent recording inputs
for the new analysis/pack/CLI journey. Selection contains exact families, complete or sampled
requirement, stride, interval and finite memory/output allowances. A cooperating construction
scope acknowledges selection before work is built; an attach after construction cannot recover
prior detail or pretend the existing owner inherited a new selection. Keep ordinary hub current
and cumulative facts plus bounded recent history; do not enlarge default retention universally.

Preserve `collect`/leave/finish/stop distinctions. Entered synchronous Python construction scopes
must leave before awaiting. Constructed async/native owners keep the selection, writer completion
joins only the writer, and collection stop/drop does not cancel science. First close reason wins;
normal drain cannot turn interruption into complete. A complete declaration requires eligible
source activation/coverage, issued/offered/acknowledged populations and actual final drain to agree.
An acknowledged applicable family with no issued events can legitimately be empty; distinguish
that case from unrequested, unsupported or unobserved coverage rather than requiring a fake event.
Sampling is sampled even when every selected record is flushed. Unknown families, refusal,
source/queue/output loss, hard-death prefixes and missing child scopes stay explicit.

Managed scope gets its separate grant once, divided across declared attempts/retries without
copying the local allowance. Preserve actual receiving process/operation and same-host deadline;
local completeness does not imply managed-child completeness. Current references and historical
retry evidence remain distinct; 47a/47d close selected frozen membership instead of silently
expanding a current prefix. Native-child clocks retain their own scope. Boundary thread CPU is
optional at real synchronous owner boundaries, never inferred across async/child work.

Recording copies prepared facts nonblocking into finite queues; no export/query/file scan,
additional residual/derivative evaluation or native getter occurs on an external reader or
inside a callback merely for convenience. Apply selection before payload copying; output,
queue/live-memory and duration limits are separate. Preserve source records/coverage on optional
query/render failure. Any adapter changes migrate actual API/CLI/pack consumers together and
remove replaced ad hoc recording glue only once replacement targeted tests pass.

**Targeted acceptance:** selection before versus after construction; unsupported family; sampled
versus complete; explicit stop followed by finish; loss/output refusal and writer failure;
managed grant/retry division; native child with independent coverage; science continuing after
recording closes. Short actual-work controls are needed only for changed collection mechanics,
with ordinary selected scientific accuracy retained.

<a id="c04"></a>

## C04 — Publish bounded query results with their exact meaning

Extend the existing declared Trace Processor query adapter and shared analysis exports to retain
input member/source identity, exact query text/version and parameters, selected interval and
clock scope, output schema/units, tool version actually used, execution status, omissions and
readable output/error prefixes. These are interpretation context, not an environment attestation.
A successful empty table means only the executed query found no rows under its scope; nonzero
exit, timeout, output cap, cancellation and malformed results cannot become empty success.
Retain private partial outputs/errors on failure and expose their availability to 47d; failed
query output is never marked complete. A pack can still complete its declared review scope with
that failure explicit when the query is optional under 47d. Preserve bounded subprocess stop/join and explicit tool
installation; querying cannot auto-install or start a service.

Keep the three existing timeline/numerical/coverage questions usable, replacing their bare-output
composition with contextualized results. Completed timeline inclusive sums keep their existing
meaning. Add requested-window selection with explicit half-open interval overlap and clipping;
retain original endpoints and distinguish clipped duration. Instants and open intervals are
separate populations, not silently zero-duration completed work. Clock-unaligned inputs cannot
be positioned on one combined timeline merely by matching numeric timestamps.

Inclusive sums, clipped union coverage, exclusive ownership time, sampled CPU and a longest
observed dependency chain are different outputs. Critical-path claims require supplied actual
dependency/wakeup/flow edges and covered boundaries; temporal nesting alone is insufficient.
Trace-local `upid`/`utid` are scoped to their capture, not global process identities. In Perfetto
58.2, `SPAN_JOIN` requires nonoverlapping intervals per integer partition: nested spans violate
that precondition and can silently produce wrong results. Choose the simplest correct pinned
interval composition; qualify exact intersection-macro preconditions before adoption, rather
than assuming current-main syntax applies to 58.2.

**Targeted acceptance:** overlapping parent/child spans, adjacent half-open spans, clipping,
instants/open spans, duplicate PID/TID across captures, uncertain clock alignment, missing edges,
empty success versus failed output, missing binary, timeout and output truncation. Fixtures with
hand-known overlapping union/inclusive totals must expose double-counting. No profiler or native
simulation is required for adapter failure controls.

<a id="c05"></a>

## C05 — Resolve question-led optional profiling and interchange

The reviewed CPU adapter records host `perf` user-space `cpu-clock:u` at 99 Hz, DWARF allowance
32768, inherited task scope and a monotonic clock; Samply conversion retains raw input/logs.
Those settings define sampled user CPU, not exact operation CPU, kernel cost or blocking.
Successful conversion does not establish remote symbol completeness. Use raw capture and explicit
resolved/unresolved/unwind/loss/leaf-only coverage as the authority.

These inquiries supply coordinator-owned I06 (profiler, interval and symbol capabilities);
resource/export feasibility contributes to I07. Physical mapping limitations in C02 consume I05,
for which 47a/47b supply the relevant scientific/native mapping facts. The coordinator owns investigation state
and disposition; the controls below describe this companion's contribution, not a second ledger.
They are bounded decisions, not a mandatory tool installation programme:

| Inquiry | Existing evidence and smallest settling control | Adoption / no-adoption, dependents and reopen trigger |
| --- | --- | --- |
| Can current CPU profiles answer the selected hot-stack question offline? | Establish actual host perf/Samply versions; take a short controlled hot-stack capture only if needed, then open relocated converted/raw members without network symbol access; inspect unresolved/unwind/loss coverage | Adopt the existing adapter/render route when the claimed stack population survives. Otherwise publish raw evidence with explicit symbol limitation and avoid resolved-stack claims. 47d/47e consume this availability. Reopen when a concrete unresolved hot stack changes the optimization decision. |
| Is an additional allocation, off-CPU or native profiler warranted? | Begin with C02's remaining hypothesis. Assess one fitting existing tool: heap/allocation capture for churn/retention; off-CPU/scheduler evidence for blocking; native profiler for opaque library work. Heaptrack, Callgrind, BPF/Tokio Console, SUNProfiler, Caliper and uProf are discovery candidates, not qualified adapters | Adopt only a safe owner/launch/permission/stop route that distinguishes the actual question with short controls and explicit overhead/coverage. No useful available route means no adoption and a retained limitation. No component waits for every candidate. Reopen on a still-decision-relevant cause these inputs cannot distinguish. |
| Would a Perfetto derivative materially help the recipient? | Use pinned 58.2 export contracts: reloadable version-coupled `perfetto`, static-table `arrow_tar` which is not reloadable, SQLite including views but excluding runtime-created tables; exercise only the chosen selected-table scope | Adopt one optional derivative if it enables a real offline query without losing required context; otherwise retain raw trace plus contextual query tables. Do not export all tables or all formats by default. 47d carries optional availability. Reopen for a named recipient query unsupported by the initial files. |
| Does selected analysis fit ordinary admitted resources? | Inspect actual physical plan and retained state for a representative selected large detail/managed/retry input; test the smallest case exposing a blocking join/window, unnecessary repeated scan or retained source payload | Remove avoidable repetition/materialization at the existing owner; use admitted streaming/selective tables or explicit bounded refusal. Measure only if the unresolved cost can change the route. No invented benchmark SLA. Reopen when an actual selected pack cannot be analyzed usefully within its admitted allowance. |

No-adoption preserves the core Rust/Arrow/ordinary-file route and must be recorded with its exact
remaining limitation. An inquiry may not quietly promise causal insight it failed to establish.
Any new tool/dependency/hook includes its lifecycle, version fit and targeted integration checks;
no generic profiler name establishes local permissions or performance.

<a id="c06"></a>

## C06 — Integrate shared analysis products and retire duplicate glue

Expose C01–C04 results through shared typed documents/tables consumed by 47d's pack publisher and
47e's thin CLI. Render bounded readable Markdown/local HTML from those same results, with source
links, actual question/window/policy and facts/hypotheses/limitations visible. Human prose is a
view, not an independently editable numerical authority. Fully local assets and relative member
links must work without a CDN, hosted viewer or symbol server.

Original JSON/JSONL and raw trace/profile members stay authoritative. Preserve readable-prefix
and source/end/loss metadata. Use one useful typed selected table format by default: IPC for the
existing typed result/view route, Parquet only where selected scale/compression/query use warrants
it; do not duplicate every member into both formats. CSV is optional readable output accompanied
by schema/category/unit meaning. UInt64 stays exact in typed tables and string/tag form in browser
data; unavailable/nonfinite categories do not collapse into a generic null or JavaScript Number.
An IPC file needs its completed footer; a truncated file is not a valid stream prefix. Recoverable
JSONL frames and IPC stream messages have their own boundary and must not be conflated.

Move existing invocation explanation/comparison and numerical-history consumers onto the shared
immutable route incrementally; preserve current policies/refusals. Reconcile generic Uno wording
with actual accepted/trial producer checkpoints, without inventing a generic iteration counter.
Replace caller-authored duplicate joining/classification and uncontextualized query-result glue in
the migrated workflow; retain raw reader/projection APIs as real composable capabilities, not
compatibility shims. Delete genuinely replaced mechanisms, callers and tests together once targeted
replacement tests pass. A future diagnostic family extends its producer declaration and shared
analysis input, without requiring another Python numerical classifier or a pack writer controller.

**Targeted acceptance:** identical supplied input/policy produces identical semantic statements
through live-copy and frozen consumers; source links resolve after relocation; renderer remains
usable offline; UInt64 round-trip above 2^53; unknown/nonfinite categories; incomplete JSONL tail,
truncated IPC file versus valid IPC stream; optional render/query failure preserves raw evidence.
47f owns the assembled CLI/pack journey, cross-context selection and once-only scope-end checks.

## Work packages

| Package | Delivered capability | Prerequisites | State |
| --- | --- | --- | --- |
| C01 | Immutable analysis inputs, exact event joins and reused admitted typed projections | Working 47a recorded-anchor/member-resolution slice for frozen consumers; existing producer/projection contracts | done |
| C02 | Ten bounded source-linked cross-owner answers and honest missing-input/causal limits | C01; descriptive context/source maps from 47a; existing facts first, qualified 47b additions only for dependent answers | done |
| C03 | Existing selected recording integrated with analysis/pack inputs, actual ACK/drain/managed coverage preserved | Existing collection and ownership contracts; coordinator's accepted rule route where integration changes it | done |
| C04 | Contextual query-result products and correct selected-window interval analysis | C01; supplied capture clocks/identities; pinned query contracts | done |
| C05 | Recorded adoption/no-adoption decisions for useful optional profiles/exports and material execution-fit doubts | C02/C04 question or actual size concern; selected tools only if needed | done |
| C06 | Shared analysis rendering and migrated 47d/47e consumers with replaced glue removed | Working C01–C04 interfaces; C05 only for adopted optional output; cooperating 47d/47e consumers | done |

C03 can progress independently of frozen-reader implementation. C02 can deliver existing-fact
answers before native candidate adoption; unavailable candidates remain explicit rather than
blocking unrelated answers. C04 is not a prerequisite for every Rust interpretation. C05 does
not impose profiler availability on core delivery. C06 integrates available slices as their
actual consumers migrate; completing a slice does not close the broader package.

## Verification and handoff

**Tested (2026-10-18), baseline zero failures:** affected runtime/math/engine unit targets
used `scripts/select.py unit` with the explicit package/filter and resolved force-validation
for Arrow-consuming closures. `pse-runtime review_analysis` passed 24 tests;
`pse-math selected_` passed 32; `pse-engine copied_observation` passed 2. The current-tree
`review_query | capture::evidence::evidence_tests | frozen_inspection |
capture_builtin_clock_relationship_precedes_typed_source_events` selection passed 17 tests.
This covers the ten named questions, event/field populations and limits, actual selected
execution timing, copied engine policy/lifetime, query context and full UInt64 fidelity.
Detail/recording controls passed 23 tests; six managed grant/division/expiry/readback controls
passed. No broad all-library qualification is inferred from these selected targets.

**Tested:** `scripts.python_tests --unit-only` selected
`selected_native_detail_current_history_zero_and_invocation_endpoints` and
`diagnostics_opt_out_preserves_native_work_in_fresh_processes`: 3 passed, including complete
and output-refused detail, zero retained ordinary history, unchanged scientific work under
opt-out and passive interpretation. The isolated current-worker
`--managed-primary-route` test
`test_managed_selected_detail_retains_actual_worker_owners_and_complete_prefixes` passed
(1 test, both usable study points, actual child owners and independent complete prefixes).
Fast native dev correctness build/stubs and the canonical worker build passed.

**Tested:** the corrected short CLI root/dynamic journeys in 47e's Outcome establish actual
preconstruction propagation and offline consumers. No numerical tolerance was changed.
Three maintained SQL files also executed successfully with installed Trace Processor 58.2:
a hand-known nested-span fixture gives original inclusive 14,000 ns, clipped inclusive
6,000 ns and union 4,000 ns over [4,000,8,000); numerical and coverage queries returned
well-formed empty tables for that fixture. The separate Python fixture covers instants/open
intervals and the shared contextual failure categories. These are correctness observations,
not simulation timing or observer-overhead measurements.

<a id="outcome"></a>
## Outcome (recorded after implementation)

The [completion audit](#completion-audit--2026-10-18) qualifies the implementation claims
below. Earlier commands/results retain their original tested scope; they do not establish
complete delivery of the reopened packages. The Work packages table owns current progress.

### What was built

**Implemented:** one pure Rust analysis owner accepts immutable supplied facts, gives all ten
named answers or one selected answer, bounds inspected sources/statements, validates exact
references and separates facts, hypotheses, limitations and next observations. Whole event
identity and field ordinal prevent expanded-field double counting; duplicate current/recent
copies collapse while conflicting identities refuse. Existing comparison/numerical policy
remains authoritative. Missing physical pullbacks remain unknown; explicit original-source
numerical diagnostics are separate work, delivered by 47e.

**Implemented:** selected actual evaluator timing separates successful/failed executed calls
from cache/projection hits, with explicit inclusive scope and overflow. Passive engine export
copies retained policy, plans, operator metrics and completion/drop state without enabling
instrumentation. Registry Arrow views retain leases, cancellation and exact UInt64/categories.
FrozenEvidenceSet is v2. Shared AnalysisRequest/AnalysisDocument v1 drives Python/CLI and
pack Markdown/HTML; neither renderer nor SQL introduces another numerical classifier.

**Implemented:** shared SourceRecordingRequest/acknowledgement v1 installs selected detail
before cooperating construction. Detail headers/documents/pages are v2; replaced v1 output
is refused. ACK publication is exclusive and atomic; capability admission, actual source
activation and final coverage remain separate. Applicable zero-issued work can complete only
with activation; unsupported/unobserved/refused families, losses and missing managed scopes
remain explicit. The benchmark Audit and recurring case runner consume this same protocol;
replaced ad hoc source-collector composition is deleted. Managed grants remain divided once
with the original expiry, and collector stop never cancels science.

**Implemented:** contextual declared queries retain exact SQL/revision, selection/clock,
actual tool version, schema/units, status and readable failure prefixes under one caller
clock. Empty success, unavailable, failure, timeout, cancellation, output cap and malformed
schema differ. Window queries preserve original/clipped inclusive and union populations;
canonical unsigned text companions prevent SQL signed/double wrapping. Typed table exports
use registry IPC streams. The trace helper delegates to this shared product.

**Interface-checked / bounded decision:** I06 retains existing perf/Samply plus explicit
symbol/unwind/loss limits. Samply 0.13.1's examined import path ignores its accepted
presymbolication setting, so no symbol-sidecar correction/derivative is adopted. No additional
profiler/platform/format is required. Reopen a specific tool only for an unresolved concrete
question after existing owner facts and CPU/timeline observations. The supporting inquiry
records the actual releases and no-adoption triggers. I07 larger/skewed/overhead scope remains
with 47f; no universal no-overhead or lossless-buffer claim is made.

### A mistake made and corrected

The first actual IDAS case requested Diffsol's `dynamic_native_return.v1`, producing an
honest unobserved family despite successful science. Six explicit IDAS manifest selectors
now request the actual segment-summary/output families; Audit capabilities and a focused
selector regression migrated together. The performing backend was unchanged.

A 16 MiB source-memory root selection also exposed finite burst loss despite spare output
space. Its original incomplete history and offline report are preserved. One explicit
128 MiB source-memory control retained all 278 events without changing ordinary defaults or
science. A separate finite 16 MiB projection refusal was preserved; a 64 MiB passive projection
completed without rerunning science. Memory, output extent and coverage remain distinct.

### Deviations from the plan, deliberate

Existing native/engine owners, registry views and pure interpretation were extended; no new
analytics engine, diagnostic store or mandatory tool stack was needed. Nonoverlapping span
join assumptions are avoided through ordered running-max/union SQL. Existing profiling limits
remain selected instead of adopting extra formats or repairing Samply. Exact benchmark winners,
long simulations, all-context qualification and final static checks remain outside this slice.

## Completion audit — 2026-10-18

Historical assessment at audit time. Current package states and the
[corrective Outcome](47-pipeline-diagnostics-and-review-artifacts.md#corrective-outcome--2026-10-18)
supersede the remaining-work statements in this section.


**Interface-checked:** independent production-source inspection and coordinator integration
review against the package targets. The [coordinator's completion audit](47-pipeline-diagnostics-and-review-artifacts.md#implementation-completeness-audit--2026-10-18)
owns CA01–CA12, their source basis, priority and closure conditions. This assessment explains
package-table corrections; it is not another finding-disposition ledger. No product repair or
47f qualification was performed. Historical focused passes above remain valid under their
stated conditions; a passing local fixture is not complete consumer integration.

| Package | Audit assessment | Basis / remaining boundary |
| --- | --- | --- |
| C01 | Partial | CA01–CA03/CA06–CA07: context identity, independent-clock windows, admitted/cancellable preparation, map ingress and performed-engine supply remain incomplete. |
| C02 | Partial | CA02/CA04–CA06: known native populations/latest gauges, actual canonical inventories and physical attribution are incomplete; explicit windows can misselect child records. |
| C03 | Aligned | Actual preconstruction ACK/activation, first closure, finite losses, managed grant division and drain preserve scientific independence. |
| C04 | Aligned | Exact contextual query status/prefixes, bounded subprocess join, unsigned transport and interval union/original/clipped populations have implementation paths. CA10 concerns pack IPC admission. |
| C05 | Aligned | The conditional profiler inquiry has a source-backed no-adoption/support-limit outcome. Larger/skewed measurements remain with I07/47f. |
| C06 | Integration incomplete | Local shared rendering, typed projection and consumer migration work. CA04–CA07 and CA12 block the composed target; no independent classifier defect or duplicate renderer is proposed. |

Next execution closes the cited supplying gaps and focused acceptance before composed completion
is claimed. Existing aligned mechanisms and the distinct live/scientific boundaries are retained;
missing broad qualification is not itself an implementation gap.


## Completeness repair — 2026-10-18

**Implemented:** Analysis v2 owns cancellable preparation, borrowed event indexes, duplicate comparisons,
working state and retained output under finite admission. Source-clock selections remain
independent across processes. Exact canonical inventory run/attempt associations and explicit
inventory indices retain unresolved links and gaps. Actual physical-admission sessions provide
bounded copied policy/plan/metric receipts on success, error and dropped futures. Exact retained
run context associates those original dependency-session receipts without relabeling their
operations/clocks or counting flattened mirrors twice.

Source recording request/ACK and launch-result envelopes are v2 (DetailRequest and launch
request remain v1). The optional separately admitted review handoff publishes original joined or
pre-handle facts; absence retains no extra model context. Its selection, original expiry,
byte/member bounds, incomplete state and finalized receipt remain separate from detail/science.
Python retention preserves an entered construction scope. Supplied handoff readback is contained,
relocatable and bounded before aggregate decoding.

The focused runtime and engine/admission selections exercised these boundaries, including
moderate borrowed-source populations, allocation/cancellation refusal, actual performed-session
policies, source-map findings and canonical inventory association. Public Python/CLI acceptance
is recorded by the coordinator's [corrective Outcome](47-pipeline-diagnostics-and-review-artifacts.md#corrective-outcome--2026-10-18). Broad observer/export fit remains 47f.
