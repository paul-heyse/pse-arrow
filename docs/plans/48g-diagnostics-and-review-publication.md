---
title: Diagnostics and review publication
status: draft
date: 2026-10-20
adrs: []
doc_role: plan
doc_owner: docs/plans/48-execution-performance-and-numerical-efficiency.md
doc_topics: [design, diagnostics, numerical-execution, performance, documentation, validation]
review_sources:
  - docs/design_review/reviews/design_review_execution-performance-recommendations-audit_2026-10-20.md
---

# Diagnostics and review publication

## Purpose and ownership

This supporting plan proposes reliable current lifecycle state, cheaper coherent
ordinary observation, prepared native progress and bounded review publication.
The [Plan 48 coordinator](48-execution-performance-and-numerical-efficiency.md)
owns finding dispositions, inquiry outcomes and the
[rule decisions](48-execution-performance-and-numerical-efficiency.md#rule-decisions),
including the operator's 2026-10-20 acceptance of RC01–RC12. This document owns G packet descriptions
and packet progress only. Acceptance of those recommendations does not establish an
implemented observation cadence or choose an allocator or ring design.

The [audit's observation and representation conclusions](../design_review/reviews/design_review_execution-performance-recommendations-audit_2026-10-20.md#observation-representation-and-additional-improvements)
and [recommendation register](../design_review/evidence/execution-performance-recommendations-audit-2026-10-20/recommendation-register.md#r2-orchestration-resources-and-diagnostics)
supply AF16 and N02–N04. I34 and I35 remain bounded selection work at the coordinator.
All new behavior here is **Proposed**. The foundation assessment follows the selected
Core 3.4, Efficient Architecture 1.0 and applicable Process Simulator 1.5 criteria,
particularly AP-05/AP-07, DP-10/DP-19/DP-21 and PS-10/PS-12. It reuses the audit rather
than initiating another whole-system review.

The target removes diagnostic amplification without changing authored science,
result permission or the distinction between measured facts and unavailable evidence.
The physical target remains **0.001** in the selected engineering cases. Callback
control, native drain and result assessment keep their existing scientific owners.
Offline CSV publication is a separate review-tool operation; its improvement cannot
be reported as lower native solve latency.

## Actual baseline and affected foundations

`crates/pse-diagnostics/src/observation.rs` already separates current producer state
from bounded history and represents loss. It has prepared fact handles and finite
phase admission. However, `PhaseGuard::drop` retires a phase through `Node::update`,
the same try-lock path used for optional updates. A failed try-lock increments loss
but can leave an admitted phase active and occupying phase capacity. N02 concerns
that current-state lifetime, not an absence of loss reporting or evidence that science
was corrupted by dropped history.

`crates/pse-backend-native/src/callback.rs::State::evaluate` already owns pre-work
stop checks, exact work admission, panic containment, attempted/completed accounting,
prepared `CallbackFacts`, and selected detail gating. Its ordinary return still
publishes several shared count/time/phase updates and can repeat unavailable-CPU
status. The audit's AF16 correction addresses this residual work. It does not justify
removing a control boundary or treating every try-lock as a blocking solver lock.

`crates/pse-backend-native/src/kinsol.rs::residual` constructs String-keyed events,
formats stable variable/row identifiers and records source-associated scalar values
for the retained prefix. A function evaluation is distinct from an accepted native
iteration. These events also feed progress paths outside selected detail, so an
instrumentation-off switch based only on selected detail would discard valid current
or durable progress. N03 requires preparing stable identity and respecting every
actual consumer, including history capacity zero.

`scripts/pipeline_review.py::publish` builds a list of every extracted scalar row,
then writes `observations.csv` and records `len(rows)` in the index.
`scripts/tests/test_pipeline_review.py` is its existing focused test owner. The audit
reports an existing aggregate of 6,550,481 rows and 1,176,406,434 CSV bytes; these are
decoded output facts, not a new peak-memory measurement. N04 concerns the retained
aggregate list and coherent output publication. Decoding one large input may still
dominate working memory after that list is removed.

These source-inspected foundations are sufficient to plan the bounded replacements.
They do not establish a measured callback overhead, an allocator bottleneck, or
losslessness under an indefinitely stalled consumer. The enduring diagnostic and
offline meanings are owned by [blueprint §23.1](../authoritative_design/sections/operations-and-validation.md#section-23-1),
failure semantics by [blueprint §23.2](../authoritative_design/sections/operations-and-validation.md#section-23-2),
and native callback/resource boundaries by
[blueprint §18.2](../authoritative_design/sections/numerical-execution.md#section-18-2)
and [§18.8](../authoritative_design/sections/numerical-execution.md#section-18-8).
Frozen review and scientific completion remain distinct under
[blueprint §19.2](../authoritative_design/sections/workflows-and-results.md#section-19-2).

Plan authoring changed documentation only. No callback, phase lifecycle, event
format, allocator or publication code was changed. No build, test, formatter or
performance campaign was run. The cited audit and focused source inspection supply
the baseline, not product-closure evidence.

## Target design and alternatives

### Reliable admitted lifecycle, independently lossy evidence

Admission of an active phase creates a finite retirement obligation. The owner must
discharge it exactly once on normal drop, unwinding, thread exit and operation
termination even when optional observation/history cannot be retained. Current phase
capacity must become reusable and readers must not see a finished phase as active
merely because the history path was busy. A lost optional end record remains explicit
loss; it cannot prevent retirement or be invented later as an observed duration.

Use the smallest bounded mechanism that supplies this guarantee with the existing
phase owner. A reliable retirement slot/state can be separate from optional event
storage; a finite queue can work only if retirement cannot disappear when it fills.
Blocking callbacks indefinitely, making science fail under optional telemetry
pressure, or declaring a finite nonblocking ring universally lossless are unacceptable
alternatives. G01 settles lifecycle first. I34 may later select a ring for optional
events without making it the authority for whether an admitted phase is active.

This boundary accommodates another instrumented stage by supplying its phase owner
and optional observations, rather than reimplementing closure under each producer.
A thread-migrating async activity must have explicit ownership; thread-local state
cannot silently interpret migration or thread exit as scientific completion.

### Exact local control with coherent ordinary publication

Keep exact work admission, local attempted/completed counts, cancellation and panic
reconciliation on the real callback path. Publish an aggregate of already produced
facts at an explicit cadence and at final flush. Each visible snapshot must have a
coherent source, count/time endpoints, active/completed phase meaning and freshness
statement; readers must not combine a new count with an unrelated old denominator.
Counters must not be reconstructed from sampled history.

RC03 acceptance permits defining this ordinary freshness contract; it does not select
the cadence value. G02 must choose and document one supported publication rule and
its stale/partial/loss semantics before migrating consumers. Compare bounded local
accumulation with safe-return publication that reduces redundant update calls while
preserving existing freshness. Prefer the simpler option when coalescing buys no
material work removal. Preserve selected detailed boundary evidence separately.
No per-callback time target or fixed percentage overhead is a completion condition.

The compiler's `crates/pse-compiler/src/workspace.rs::observe_query_event` is another
G02 consumer: selected Salsa event variants format database keys and publish shared
observations. Aggregate their ordinary execute/validate/wait/discard facts using prepared
identities where useful, retaining optional per-query attribution and its availability/loss
meanings. Do not describe every Salsa variant as formatted or alter query execution,
cancellation, observer hooks or semantic dependencies to suppress telemetry. Focused
compiler-observation controls distinguish ordinary aggregate and selected query views,
unchanged cancellation/diagnostics and final counts across database replacement.

The pre-work stop check prevents prohibited work; the post-success stop check rejects
a result that became late while executing. Preserve both, the panic boundary, failed
attempt accounting and final original assessment. Flushing observation must not
revive success after cancellation, turn callback evaluations into accepted iterations,
or release native charges before actual drain. Invariant unavailable-CPU state can
be prepared/published once without pretending a sample was taken.

### Stable identity once, compact values per event

Prepare KINSOL's stable row/variable identifiers, retained scalar slots, role and
source metadata at their actual session/request validity boundary. Repeated callbacks
update compact values and exact ordinals. Human-readable String-keyed views are
materialized only for consumers that need them. A value/policy/request change binds
the right source and controls; compatible native reuse must not retain a prior
attempt's progress identity or consumer selection.

Inventory and migrate current progress, retained history, durable taps, selected
detail, result/native records and exported review consumers together. A public rich
projection may remain where it is the required format; it should consume prepared
identity rather than force callbacks to rebuild it. The alternative of gating rich
events solely on selected detail fails the history-zero current and durable cases.
Keeping the current rich representation may be reasonable for a genuinely cold
consumer boundary; the target removes repeated stable formatting in the inner path.

KINSOL's equation/Picard residual role, coordinate prefix omission, source pointer,
nonfinite availability, and evaluation-versus-iteration meaning survive. The work
does not change native stopping norms or the physical tolerance corrections owned by
the [native-controls plan](48d-native-controls-starts-and-recycle-policies.md).

### Instrument actual missing stages after lifecycle repair

Missing lowering, specialization and materialization boundaries deserve stage spans
where they answer a concrete attribution question. More events before reliable
retirement can increase loss and false active state. G04 therefore follows G01 and
uses G02/G03 volume/consumer decisions before adopting another buffer.

I34 compares direct prepared observation, compact local/coalesced records and finite
rings with explicit overload semantics. It must account for producer cardinality,
async migration, thread exit, terminal drain and retained bytes. An SPSC design is
eligible only where single producer/consumer ownership is actually true. Closed
phases and known final counts remain reliable even when optional history reports
overflow. No missing sample is interpolated into native CPU work or a critical path.

The [source/preparation plan](48a-source-checking-and-incremental-preparation.md),
[receiving-validation plan](48b-registry-and-receiving-validation.md) and
[mathematical-products plan](48c-mathematical-products-and-compiled-execution.md)
own the actual stage boundaries they change. They provide G04 meaningful ownership,
not a mandate for a receipt/span per tiny product. The
[dynamics/fitting plan](48e-dynamics-fitting-and-preconditioning.md) retains its
segment/reset and trial meanings. Any spawn selected by
[48f](48f-scheduling-resources-and-result-completion.md) must propagate observation,
invocation, canonical, activity and population context and join issued producers;
adding spans cannot repair missing execution context after the fact.

### Stream raw review facts through one coherent publication

Write each extracted scalar row to private temporary output as it is decoded, keeping
a running count. Preserve case/source identity, record line, JSON pointer, exact
scalar/null encoding and deterministic traversal order. JSON null, unavailable input,
parse failure and absent scalar remain different. Preserve distinct original samples;
duplicate projections are labeled as projections, not silently counted as independent
observations. Optional typed or partitioned analytical views can consume the raw facts
later; they do not replace or deduplicate the raw output in this packet.

The output/index/report set must agree on one completed generation. Use a private
staged output and the existing publication owner's coherent finalization convention,
or an explicit manifest/commit point that readers honor. Renaming CSV and index
individually is not by itself multi-file atomicity. On a writer/read error retain a
truthful incomplete/error outcome or the prior complete generation according to the
chosen owner; never advertise a partial new CSV with a completed count. All writers
close before final visibility. Do not impose the scientific durable store protocol
on this offline tool merely to obtain coherent local files.

The target working memory is the current decoded input plus bounded writer buffers
and the existing compact publication index, without an aggregate scalar-row list.
It is not a constant-space claim for arbitrary JSON input or every index member.
Retaining all rows is simpler for small inputs but does not fit the known large
aggregate. Changing CSV to another mandatory format would move the consumer contract
without being necessary to remove the identified list.

### Allocator selection follows removal of materialization

I35 investigates an allocator only after the relevant repeated formatting and
materialization are removed. Compare the current allocator with a technically
eligible alternative under matched representative operations, including retained
memory and cleanup. An observed allocation-heavy stack is a lead; debugger occupancy
and `cpu-clock:u` self share are not allocator-only cost or hardware cycle shares.
Ordinary observation enabled by default must not be mislabeled diagnostics-off.
No allocator, new dependency or environment-attestation mechanism is predetermined.

## Work packages

All deliveries are **Proposed**. Packet progress belongs in this table; findings,
inquiry selections and RC03 acceptance stay at the coordinator.

| Packet | Prerequisite | Delivery and acceptance | State |
| --- | --- | --- | --- |
| G01 — Reliable phase retirement | Existing phase admission/guard and terminal operation owner | Separate reliable admitted current-phase retirement from optional history (N02). Verify held-lock contention, overflow/refusal, guard unwind, thread exit, operation termination and phase-capacity reuse without false active state. Remove the lossy update route as the retirement authority and replace tests/fixtures that encode its incidental loss. | open |
| G02 — Coherent ordinary aggregates | G01 lifecycle contract; accepted RC03 and explicit freshness/final-flush rule | Coalesce ordinary callback and selected compiler/Salsa shared updates, retaining exact control and optional query attribution. Test cadence/zero/nesting/late stop/panic/loss/final flush plus unchanged compiler execution/cancellation and database replacement; delete redundant publication/formatting after consumers migrate. | open |
| G03 — Prepared compact native progress | Current/history/durable/detail consumer inventory; G02 publication meanings where shared | Prepare stable KINSOL IDs/slots and invariant unavailable status once; update compact scalar/ordinal values and project rich events at consumers (N03). Check history-zero current, durable taps, selected detail, prefix omissions, nonfinite values, evaluation roles and reused-session identity. Delete repeated stable String formatting and superseded event construction, preserving required projections. | open |
| G04 — Missing stages and finite buffers | G01 reliable retirement; G02/G03 volume/consumer contracts; I34 outcome; actual stage owners in 48a–48e | Add useful missing lowering/specialization/materialization boundaries; adopt finite rings only if ownership and overload/drain evidence justify them. Test stalled consumer, migration, overflow, thread exit, terminal drain and exact loss/freshness meanings. Delete replaced buffering/update paths after complete migration; do not claim lossless optional history. | open |
| G05 — Streaming aggregate publication | Existing raw scalar identity/output contract and chosen coherent local commit boundary | Stream temporary CSV with running count and publish coherent output/index/report (N04). Verify exact rows/pointers/nulls/counts, distinct samples and labeled duplicate projections, JSONL errors, write failure and interrupted finalization. Delete aggregate row-list accumulation and tests depending on that implementation; preserve raw format consumers. | open |
| G06 — Matched allocator inquiry | G02/G03/G05 and applicable 48a–48f materialization fixes on the measured path; I35 outcome | Compare current and technically eligible allocator behavior only for an observed remaining question. Record matched conditions, actual native/scientific work, memory/cleanup and descriptive variation. Adopt only a justified scoped change, or record no adoption and its observable trigger at the coordinator. No allocator dependency or platform expansion is assumed. | open |

G01 should land before increased span volume. G03 may prepare identities while G02
defines publication, but migration must consume the settled shared meanings. G05 is
independent offline work and lower product priority; it need not block callback work.
G06 waits only for fixes on its actual measured operation, rather than requiring all
unrelated packets to finish. Adoption involving native placement consumes F07/F08
resource/context boundaries; a diagnostic buffer never owns native teardown.

## Finite inquiries

The following descriptions explain the coordinator's inquiries. A no-adopt decision
is a valid result, with the reason and observable trigger recorded at that single
owner. Neither inquiry creates a new independent rule/status ledger.

### I34 — Which stage evidence needs which buffer?

The question is whether actual missing stage attribution after G01–G03 needs another
finite transport or can use the repaired prepared/coalesced owner. Compare that route
with a ring only for a defined producer/consumer population. Examine one normal
lowering/materialization path, a stalled consumer/overflow case and a producer exit
or migration case. Establish admitted lifecycle capacity, optional loss reporting,
drain, ownership and retained-byte bounds before choosing a design.

Stop once source ownership and those controls distinguish the smallest useful route;
do not run a platform-wide buffer campaign. Adopt only the selected stage/buffering
slice with explicit overload semantics, or retain the existing route and record no
adoption. Revisit on observed missing stage attribution, sustained optional loss
that prevents the selected question being answered, or a new producer topology that
invalidates the chosen route. Finite rings cannot promise losslessness for arbitrary
consumer stalls; reliable lifecycle remains independent in either outcome.

### I35 — Does a different allocator improve the remaining operation?

The question is whether allocation behavior still materially limits a named supported
operation after structural fixes. Compare the current allocator, a technically
eligible alternative and no allocator change. Use the same model/selection, warm or
cold state, ordinary/detail policy, native profile, compiler/build and capacity
conditions. Capture actual scientific work and accepted outcomes as well as elapsed
time, memory and teardown; fewer numerical iterations cannot be attributed to the
allocator. Treat inclusive time and independent process clocks according to their
owners, and distinguish child memory from parent RSS and reservations.

Begin with a short matched case. If a timing-dependent selection remains, use a
finite alternating set such as five paired observations with actual medians/ranges.
Stop when the remaining allocator contribution is absent, the alternative fails a
functional/resource contract, or the matched evidence distinguishes a useful scoped
choice. Adopt only with that evidence and affected targeted checks; otherwise retain
the current allocator and record no adoption. Revisit on a new supported operation
showing substantial residual allocation pressure or a relevant library/allocator
capability change. No exact percentage or latency threshold is introduced.

## Migration and acceptance

The migration scope is the actual diagnostic producer and all consumed current,
history, durable, selected-detail, frozen export, CLI/Python and review views affected
by each changed representation or cadence. Representative controls do not narrow that
inventory. Preserve exact source identities, clocks, units, omitted prefixes,
availability, reset boundaries and loss. A missing event is not zero work, an
evaluation is not a native iteration, and an offline pack completion is not native
scientific success. Keep optional capture failure separate from scientific failure.

Use existing prepared fact/producer and review publication owners. Once a replacement
is exercised by targeted tests and consumers have moved, delete replaced update,
formatting, buffering and aggregate-list paths, their obsolete callers, tests and
fixtures in the same change. No compatibility observer, second progress stream or
parallel publication path is retained just as evidence. A no-adopt inquiry adds none.

**Proposed verification:** `just unit-package pse-diagnostics observation` covers
the focused lifecycle/current/history controls; targeted callback/progress/KINSOL
tests in `pse-backend-native` cover exact work, both stop boundaries, panic and
native-role preservation; affected `pse-runtime` observation/worker/review filters
cover actual consumers. `scripts/tests/test_pipeline_review.py` receives streaming
equivalence and failure-finalization controls through the pinned script-test route.
Choose the precise filters with the implementation; these names are planned test
owners, not executed evidence. Arrow-consuming correctness closures retain explicit
force-validation and native tests use the native environment.

Use independently specified small records to compare CSV source/line/pointer/value
tuples and counts; deriving the expected output through the same new iterator would
miss a shared omission. Exercise many rows through a bounded writer without requiring
recreation of the full known aggregate. Any quantitative peak-memory claim requires
an actual measured operation, decoded-input scope and conditions. Lifecycle tests
must force contention and terminal loss, not only show an uncontended phase closes.

The [qualification/publication plan](48h-qualification-and-design-publication.md)
owns assembled native/worker/Python/frozen-review journeys, final hygiene and any
matched performance campaign needed for selection. Each actual check later reports
command, mode, scope and failure count against zero at its evidence owner. Focused
callback or script tests establish neither full simulator qualification nor a
performance benefit outside their measured operation.

## Outcome (recorded after implementation)

### What was built

**Proposed; not implemented.** Product execution and qualification remain unstarted.
Record the delivered behavior and its evidence here after implementation.

### A mistake made and corrected

**Proposed; not implemented.** Record an actual implementation mistake and its
correction when execution supplies that evidence.

### Deviations from the plan, deliberate

**Proposed; not implemented.** Record deliberate implementation deviations and their
reasons after execution. Finding, inquiry and rule state remains owned by the
coordinator.
