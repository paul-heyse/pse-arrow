---
title: "Plan 62d: Compiler-backed explanations"
status: proposed
date: 2026-11-01
adrs: []
review_sources: [docs/functional_reviews/code-intelligence-capability-expansion-2026-11-01.md]
doc_role: plan
doc_owner: docs/plans/62-code-intelligence-capability-expansion.md
doc_topics: [design, diagnostics, libraries]
---

# Plan 62d: Compiler-backed explanations

## Purpose and basis

**Proposed:** answer selected questions about captures, values retained across suspension,
initialization-dependent destruction and unwind cleanup using compiler-owned evidence. This
companion owns O05 and the compiler-backed portions of O04/O13 from the
[source review](../functional_reviews/code-intelligence-capability-expansion-2026-11-01.md#O05).
The [coordinator](62-code-intelligence-capability-expansion.md) owns shared decisions, opportunity
disposition, cross-plan acceptance and overall completion. This document's table owns D01–D04.

The planning baseline is `4b2f8df34`. The source review examined `5be967c10f3905732d73dfaa960fdad9aa9534b0`;
its library overviews include operator-owned dirty inputs that remain untouched. Core 3.4,
the selected efficient-architecture heuristics and the pse-arrow binding shape this proposal.
This is plan authoring, without a new diagnostic audit or runtime qualification. Plan60 supplies
no scope, dependency or completion authority here.

The operator confirmed a narrow pinned compiler producer after bounded feasibility where
original artifacts and analyzer facts cannot adequately answer a selected question. That
selection does not settle compiler callback accessibility, approve a new crate automatically,
or establish delivered compiler integration. T01 routes the consequential decision before D01;
any resulting architecture/ADR change follows the existing decision process before its dependent
implementation. No governing rule is changed by this plan.

## Existing foundations and target boundary

`crates/pse-host/src/compiler_investigation.rs` reads original diagnostics, related spans,
suggestions and source comparisons. Its `artifact.rs` owner already acquires explicitly selected
MIR, THIR, type-size and mono-item evidence, retains source/context and original files, and
supports bounded reads. `artifact::flags` applies native body/pass filters before capture.
`semantic/facts.rs` supplies prepared-world type/target/bound facts; the analyzer packages own
source-level captures and operation explanations. These remain useful independently of a
compiler-private producer.

The missing behavior is a structured answer that connects a selected source/body question to
the compiler phase, places, suspension sites or cleanup paths supporting it. Raw dumps can
already settle some investigations. D01 must distinguish those from questions requiring typed
compiler queries. It must not turn a general textual MIR parser into a claimed borrow checker.

Keep compiler integration under the existing compiler-investigation responsibility, using an
explicitly prepared, isolated producer only when needed. The placement choice is between
existing artifact acquisition and the narrow producer; it is not a new controller, persistent
service, test runner or compiler-pass registry. Native compiler state stays inside its compiler
lifetime. Project owned facts before that context disappears and retain original artifacts as
expandable evidence. The consumer must not depend on compiler pointers, session-local numeric
IDs or a pretty-printed MIR grammar.

T02 supplies the first consumed typed contract slice for D02/D03: qualified compiler source,
unit/body and phase identity; native locations and source correspondence; requested versus
established facts; and unavailable/unsupported/omitted distinctions. It extends the existing
fact/lineage owners and authored proto only where facts cross their actual helper boundary.
It does not require a new serializable type for every compiler implementation detail. Original
opaque evidence remains available, while structured consumers stop interpreting it independently.

A02 source capture is a useful input when a question begins at current or historical source.
It is not a blanket prerequisite: an original body/unit artifact can be inspected without a
current-source selection. Correspondence to current source is separately qualified; failed or
ambiguous correspondence must not invalidate an otherwise valid original-artifact observation
or silently attach it to the current semantic world.

## Meaning and execution choices

Every established answer names the compiler identity, target, effective compilation context,
selected body/unit, MIR phase and relevant original evidence. Distinguish compiler-analyzed
source, original captured bytes and any current-source comparison. A body identifier is local
to its qualified acquisition, not a durable global symbol. Macro-generated or synthetic bodies
can have incomplete or many-to-many source relationships; keep that uncertainty explicit.

Demand selects a body and question before expensive acquisition. Reuse one compatible compiler
context for the requested facts rather than running a full compiler per local or suspension
point. Keep early borrow-check facts separate from later lowered/optimized bodies, whose source
relationships may have disappeared. Reuse binds all consumed compiler/context/phase inputs;
changing any of them requires new evidence. No blanket cache or index is required.

The existing operation owner retains cancellation, child ownership, captured output and terminal
settlement. The producer returns owned facts and original failure causes through that owner.
A missing toolchain component, incompatible compiler, stolen MIR, type-check failure or
unsupported body shape remains explicit. Reads never install components, compile a producer,
start analysis or rerun failed evidence implicitly. Physical completion, compiler success,
capture completeness and semantic support remain separate outcomes.

### Concrete mechanisms and limits

The actual pin is `nightly-2026-09-29`, reporting rustc/rustdoc
`1.101.0-nightly (c1070d693 2026-09-28)`. The two MIR overviews target stable 1.99.1 and overlap;
treat them as one discovery family. D01 uses exact commit sources and a bounded compile check
to settle integration, rather than assuming that overview names imply callable downstream APIs.

- `TypeckResults::closure_min_captures_flattened`, `closure_min_captures` and
  `closure_kind_origins` support place-sensitive captures and their source causes.
  [Pinned type-check results](https://github.com/rust-lang/rust/blob/c1070d693/compiler/rustc_middle/src/ty/typeck_results.rs).
- `mir_coroutine_witnesses` constructs `CoroutineLayout` with `field_tys`, `variant_fields`,
  `variant_source_info` and `storage_conflicts`. Its lower helpers are restricted visibility;
  query/driver accessibility must be checked separately. Saved-state reasoning combines
  `MaybeLiveLocals`, `MaybeBorrowedLocals`, `MaybeRequiresStorage` and `MaybeStorageLive`.
  [Pinned coroutine layout](https://github.com/rust-lang/rust/blob/c1070d693/compiler/rustc_mir_transform/src/coroutine/layout.rs).
- `MoveData`, `MaybeInitializedPlaces`, `MaybeUninitializedPlaces` and drop-state effects
  describe initialization; `ElaborateDrops` makes runtime destruction explicit;
  `AbortUnwindingCalls` enforces ABI-sensitive unwind behavior. Drop validity/liveness and
  runtime cleanup are different questions. `run_analysis_to_runtime_passes` is public but
  compiler-private and phase-constrained; it is not a registration API for arbitrary passes.
  [Pinned orchestration](https://github.com/rust-lang/rust/blob/c1070d693/compiler/rustc_mir_transform/src/lib.rs).
- `rustc_borrowck::consumers::get_bodies_with_borrowck_facts` and `ConsumerOptions` expose
  analyzed bodies, loans and region context. Extraction coordinates type-checking roots and
  nested bodies, regenerates region IDs, costs compiler work and can fail after MIR stealing.
  `-Zpolonius` can override extraction options. Legacy facts are not Polonius Alpha.
  [Pinned consumer API](https://github.com/rust-lang/rust/blob/c1070d693/compiler/rustc_borrowck/src/consumers.rs).

The inspected pinned dataflow exports do not expose `precise_liveness`; do not schedule it as
an available prerequisite merely because newer nightly documentation describes it.
[Pinned exports](https://github.com/rust-lang/rust/blob/c1070d693/compiler/rustc_mir_dataflow/src/impls/mod.rs).
General dataflow is conservative and intraprocedural. `MaybeBorrowedLocals` is not loan checking;
use liveness, drop liveness and storage liveness are not interchangeable. Storage-conflict
absence does not guarantee physical overlap. Static retention/cleanup never proves actual
process resource release, executor behavior or a runtime observation.

## Package state

| Package | Status | Dependencies | Completion boundary |
| --- | --- | --- | --- |
| D01 | open | T01 decision route | Selected question set, exact-interface feasibility and placement settled; viable supported scope and unresolved dependent scope recorded. |
| D02 | open | D01 viable capture/suspension route; T02 first consumed typed slice | Capture/suspension answers consumed through existing investigation surfaces, with phase/context fidelity and revealing targeted tests; replaced interpretation deleted. |
| D03 | open | D01 viable cleanup route; T02 first consumed typed slice | Drop/unwind answers consumed through existing surfaces with original-artifact expansion and targeted normal/unwind controls; replaced interpretation deleted. |
| D04 | open | D01; concrete unanswered ownership question after diagnostics/artifact assessment | Conditional trigger evaluated explicitly; either supported extraction and consumers verified, or reasoned deferred/not-needed disposition recorded without claiming delivered borrow-fact functionality. |

### D01 — Settle selected questions, exact interfaces and placement

Start after T01 supplies the decision route and accepted scope for a pinned producer. Select
the minimal questions needed for D02/D03, with one revealing success/refusal scenario for each.
Compare prepared analyzer facts, existing native dump acquisition and a selected query/driver
adapter against those questions. Determine precisely which answers need structured acquisition.

For the producer candidate, check the actual pinned compiler components/linkage, callback stage,
query accessibility, type-checking-root selection, synthetic/nested-body handling and required
MIR before later queries consume it. Perform bounded compile/interface checks and a small
selected-body feasibility exercise during authorized execution; do not build a general provider
before those checks settle the seam. Coordinate any new crate or changed boundary meaning with
T01's ADR/design route before dependent changes.

Record the selected route per question, preparation and failure behavior, owned projection and
consumer seam. Where artifacts suffice, retain that route instead of introducing a driver for
uniformity. Where the narrow producer is viable, implement its explicit preparation/acquisition
seam under the existing owner with an interface control for incompatibility and cancellation.
Only questions justified by this exercise enter D02/D03 support.

D01's investigation can finish with a negative feasibility result, but that result does not
complete D02/D03. Record the affected supported scope as unresolved and obtain a coordinator
disposition or revised design before claiming the series complete. A failed probe is evidence
about that attempt; absence of an accessible helper is not evidence that all compiler routes fail.

### D02 — Explain captures and values retained across suspension

Consume the selected D01 route and T02 contract. Given a qualified closure/coroutine body or
source selection, return captured places/modes with available cause locations, and saved-local
types associated with selected suspension variants/sites. Relate source-level captures to
compiler state only through established correspondences. Keep source capture, captured value,
saved local and projected coroutine field distinct. Explain missing source names/spans without
inventing them. A `Send` explanation identifies relevant retained-type evidence and the actual
native predicate/diagnostic, rather than equating every retained type with a failed bound.

Migrate the compiler investigation provider, focused answer composition and presentation to
these typed facts, retaining guarded original artifact expansion. Share the consumed slice with
the analyzer capture owner so both can contribute without asserting identical guarantees.
Where a current consumer interprets raw capture/suspension sections, replace that interpretation
and delete its callers/tests once the typed route passes. Original retained artifacts and their
readers are preserved evidence, not obsolete mechanisms.

Targeted tests cover disjoint-field versus whole-aggregate capture; a non-Send value scoped
before an await versus retained across it; two awaits retaining different locals; and a
generated body with unavailable source correspondence. Compare selected output with independently
visible native compiler diagnostics/MIR and source intent, not only a snapshot generated by the
same projection. Verify incompatible contexts and unsupported variants remain unknown/refused.

### D03 — Explain destruction and unwind cleanup

Consume D01's phase-specific route and T02 contract. Given an original body and selected
operation/place, distinguish initialization state, lifetime validity for destruction and lowered
drop/cleanup control flow. Use native before/after artifacts when they settle the question;
structured query acquisition fills only the justified gaps. Preserve panic strategy, target
and enclosing/callee ABI facts where unwind behavior depends on them.

Migrate compiler explanation composition and presentation together with their provider. Identify
source-authored versus generated drop glue and shims where established. Keep both phase-specific
evidence addresses so a user can expand the actual branch/drop rather than receive an unsupported
summary. Delete superseded textual cleanup inference and its tests after callers migrate;
do not delete native artifact acquisition or introduce a compatibility interpretation path.

Targeted tests compare partial move followed by normal return versus a panicking call; a
conditional/open drop versus a definitely uninitialized value; and an `extern "C-unwind"`
boundary whose permitted unwind propagation differs from another ABI. Include a mismatch in
phase/compiler context and a generated drop with no unique source site. Expected destructor
order/behavior comes from independently authored scenario assertions and native evidence.
Runtime scenarios, where needed, execute through existing run/test ownership; static analysis
alone establishes no actual resource-release result.

### D04 — Acquire borrow facts only for a demonstrated unresolved question

The trigger is an accepted, concrete ownership/lifetime question that existing original
diagnostics, analyzer facts and selected MIR cannot answer adequately. Evaluate that trigger
explicitly even if no extraction is selected. Record the question and the missing evidence;
do not add broad loan/region acquisition merely to populate a category.

If triggered, check the exact consumer/callback lifecycle in D01's producer, choose the least
sufficient `ConsumerOptions`, and capture only owned qualified facts needed by the question.
Coordinate roots/nested bodies before query consumption; expose early/late locations, loan
identity and region relationships with their compiler context. Do not invent source intervals
for NLL regions or conflate separate analyses with a universal fixed point. Migrate the actual
question consumer and delete the narrower approximation it replaces, keeping diagnostics and
original facts available as original evidence.

Targeted tests exercise two-phase borrowing, a partial initialization/move, nested closure
constraints and a deliberate query-order/refusal case. Add higher-ranked/opaque obligations
only when required by the selected question; unsupported cases must not appear proved.
Test option/configuration interactions if legacy Polonius facts are consumed. If extraction is
not needed or not feasible, record the bounded reason, affected unsupported question and any
observable revisit trigger with the coordinator. Conditional disposition may close D04's
assessment obligation; it cannot be reported as implemented borrow-fact support.

## Verification and handoff

**Proposed verification:** each implemented slice uses `scripts/pse check <package> --all-targets`
and `scripts/pse test <package> --lib -E '<filterset>'` controls for the
new behavior. Compiler-specific controls must identify the exact pin, target, selected body,
phase and configuration. Keep Arrow force-validation where the actual closure requires it;
do not add unrelated product or scientific campaigns to compiler explanation tests.

The coordinator selects the assembled investigation journey and scope-end integrated/static
checks after functional scope, including relevant lineage/proto generation checks where the
authored boundary changes. D02/D03 can proceed independently after their actual prerequisite
slices land, while shared compiler/fact files require coordinated ownership. Focused passes do
not establish every compiler construct, platform or whole-system qualification. No quantitative
performance benefit is claimed; a measurement campaign requires a specific unresolved workload
question rather than being an automatic package gate.

**Interface-checked planning evidence:** the source review and exact source references support
the named candidate mechanisms and limits. **Not run during plan creation:** compiler probes,
builds, tests, services and product qualification. The next executable step is T01 followed by
D01; an investigation outcome is a design input until its required producer/consumers and tests
are implemented.

Before closure, record actual commands, mode, scope, zero-failure baseline and results in the
owning completion account. Transfer enduring contracts and operating guidance through their
normal owners, including the decision route where required; reconcile the coordinator's
opportunity dispositions and retire completed material under ADR-0096. No Outcome completion
claim exists at plan creation.
