---
doc_provenance: authored
doc_retention: while-dependent
doc_owner: docs/design_review/reviews/design_review_post-plan41-simulation-efficiency-and-diagnostics_2026-10-16.md
title: Post-Plan41 simulation efficiency and diagnostics
date: 2026-10-16
tier: design
purpose: target
reviewer: Independent principal design reviewer
baseline: 6af92d1dee1ffa0b80dca71e03a63b489f2d940c
standard:
  core: "3.4"
  efficient-architecture: "1.0"
  process-simulator: "1.5"
audit_observation_owner: docs/plans/39d-qualification-and-plan38-cutover.md
---

# Post-Plan41 simulation efficiency and diagnostics

## 1. Integrated assessment and review boundary

**Overall decision: Revise. Architectural fitness: failed. Behavioral and scientific adequacy: not accepted for the combined reviewed route.** The retained scientific successes remain valid within their recorded conditions; they do not qualify the default result-analysis route, complete diagnostic coverage or the proposed corrections.

The simulator has useful architectural foundations: authored definitions govern preparation; declared execution can retain its admitted model; mathematical structure and changing values have separate downstream products; native libraries own numerical iteration; durable results have exact fenced acknowledgements; diagnostics remain observational. The next design should preserve these distinctions.

The material failures occur in their composition. Some consumers discard preparation already obtained, warm lookup constructs substantial structure before discovering a reusable product, homogeneous file packing also determines metadata submission boundaries, and generic diagnostic recency determines whether a completed scientific responsibility remains explainable. Separately, the result-analysis async composition has demonstrated excessive demand on the ordinary caller stack. These causes do not require one replacement architecture.

This is a **design-tier, target-purpose subsystem review** of preparation/reuse, result publication and analysis, diagnostic retention/export, and managed receiver discovery. The functional target is the Process Simulator 1.5 target: interactive value edits without structural rebuilding; studies, recycles, dynamics and fitting over shared model definitions; extensibility; and development using a responsibility’s actual dependencies.

The examined baseline is `main` at `6af92d1dee1ffa0b80dca71e03a63b489f2d940c`, plus the seven pending audit/harness/collector/documentation files identified by the coordinator. They contain no production corrections for AUD39-08–15. Supporting evidence is in [the evidence directory](../evidence/post-plan41-simulation-efficiency-and-diagnostics-2026-10-16/README.md). Source observations below are **Interface-checked**. Corrections and their benefits are **Proposed**.

The existing audit’s **Tested / Measured** claims retain its conditions: once-only development-profile execution, selected reference store, native solvers, explicit Arrow validation and original assertions. Against zero failures, the eleven original cases recorded six completed cases, three failures and two cooperative interruptions. The unchanged publication workload passed separately with a 64 MiB main-stack soft limit instead of 8 MiB. Those observations are not statistical performance qualification.

Decisive source was inspected independently, including modeling declaration/case/diagnostic preparation, scalar checking and selected-source resolution, bound-view lookup and attribution, result projection and canonical append, result analysis and the Python blocking bridge, diagnostic admission/link/export, fitting/shooting observation, and the collector. Supporting inquiries informed alternatives; their recommendations were not treated as conclusions.

No new tests or measurements were run. This review does not qualify every unit/property model, solver class, derivative chain, deployment, cleanup path or scientific campaign. Environment/provenance proofs and a broad benchmark campaign are outside scope.

## 2. Responsibilities and operational contracts

The important seams already exist. They should be completed and consumed rather than replaced by a universal operation registry.

| Responsibility | Owned meaning and consequential dependencies | Appropriate local verification |
|---|---|---|
| Modeling preparation | Source revision, actual instance, structural bindings, demand, physical/provider context and admitted model | Checker/compiler inputs; runtime integration where admission or retained ownership is exercised |
| Case resolution | Specifications, starts, fixed/free state, bounds, numerical policy and missing-input permission | Explicit case/model inputs; storage only when source or recording effects are tested |
| Numerical execution | Native capability selection, attempt-owned workspaces, cancellation, joins and original assessment | Admitted numerical problem and actual adapter |
| Result publication | Sealed payload identity, attempt fence, set ordering, immutable operation receipts and exact acknowledged inventory | Codec/group controls locally; actual-store tests for fencing, atomicity and settlement |
| Result analysis | Protected original evidence, response dispatch, artifact promotion, graph membership and activation | Bounded graph/interpretation controls, plus genuine protected-read/publication integration |
| Diagnostics | Producer-owned facts, relationships, cumulative/recent meanings, loss and terminal observability | Small hub/export inputs without scientific execution |
| Receiver discovery | Exact process instance and checkpoint association; live versus retained-prefix distinction | Synthetic catalogues and locator documents; managed execution for assembled coverage |

A model preparation is not an executed result. A mathematical view is not current source attribution. A sealed file is not acknowledged result membership. An artifact metadata group is not a run-result append group. A diagnostic checkpoint is neither a scientific completion nor permission to stop a service.

These distinctions govern behavior in the inspected code. Two operational contracts need strengthening: retaining useful responsibility summaries through terminal consumption, and making ordinary caller execution placement credible for result analysis. Generic mechanisms may implement those contracts, but should not decide their domain meaning independently.

### Physical semantics and well-posedness

This review changes no physical formulation. The following are preservation obligations, not newly qualified scientific claims.

| Element | Dimension/unit and basis | Convention/envelope | Authority and review boundary |
|---|---|---|---|
| Authored state, parameter and output quantities | Declared physical quantity contracts; no conversion inferred from a bare numeric value | Declared basis, reference state, bounds and applicability remain effective | Physical declarations/compiler products, blueprint §8–9; full catalogue not requalified |
| Fixed/free variables and starts | Physical ports retain their quantity identities | Specification, free unknown and initial guess remain distinct | `resolve_starts`, `resolve_prepared_case`, `bound_structure`; diagnostics intentionally permits missing free values |
| Dynamic samples and shooting controls | Authored time/control/state quantities and original coordinates | Horizon, scheduled changes, derivative demand and endpoint conditions remain explicit | Dynamics/shooting owners; retained audit exercises a selected IDAS trajectory only |
| Published evidence | Original physical rows, fields, coordinates and source identities | Withholding, partiality and scientific usability survive analysis/publication | Result/artifact/analysis owners; neither ACK nor graph edge establishes scientific validity |

Well-posedness must continue to derive from declared variable roles and equation structure before numerical execution. Topology, incidence and solve order remain different structures. The review inspected bound-state/domain admission and shooting completion interpretation, but did not independently trace every degree-of-freedom or structural-rejection path. **PS-G2 therefore remains unresolved for the surrounding scientific breadth**, rather than being passed from successful fixtures.

## 3. Representative journeys and execution

| Scenario | Required response and distinguishing observation |
|---|---|
| **S01 — Value edit and re-solve; instance/binding change** | Reuse admitted structure; bind changed values and refresh affected numerical facts. Preserve demand, bounds, physical eligibility and current attribution. |
| **S02 — Study, fitting or shooting; composition** | Reuse compatible preparation/workers, retain independent attempts, and explain trial/experiment/window work without unbounded histories. |
| **S03 — Difficult recycle or interrupted dynamics; policy/failure** | Preserve specification, declared convergence policy, truthful partiality and actual drain. Diagnosis must survive repetitive inner work. |
| **S04 — Add a unit/property model; ordinary domain extension** | Existing physical concepts extend declarations and their semantic owner. Common checks and workflow/publication policy should not be re-authored. |
| **S05 — Replace an integration mechanism** | A scheduler, tracing subscriber or storage implementation preserves consumed ownership, cancellation, acknowledgement and diagnostic meaning. Incompatible capabilities remain explicit. |
| **S06 — Grow or interrupt publication; lifecycle** | Group compatible metadata without retaining encoder-sized working sets, weakening live-prefix acknowledgement or restarting clocks. |
| **S07 — Test one responsibility** | Index checking, lookup semantics, retention priority and locator validation run without unrelated store/native startup. Real transaction and solver properties retain integration tests. |

The numerical route remains preparation → case binding → admitted native execution → independent original assessment → publication. Its important numerical constraints are:

| Stage | Formulation/derivatives/scaling | Capability and outcome obligations |
|---|---|---|
| Preparation/binding | Preserve guards, physical typing, discrete admission, derivative demand and supplied scaling | Structural changes may rebuild; changing starts alone must not redefine structure |
| Fit/shooting evaluation | Preserve declared derivative sources/order, accuracy and branch/start permissions | Libraries own numerical iteration; domain owners compose experiments/windows and retain accepted/rejected trial distinctions |
| Original assessment | Preserve residual, bound, domain, closure and trajectory/endpoint evidence | Solver status alone cannot make a solution |
| Publication/analysis | Introduces no new numerical method | Preserve original coordinates, units, withholding and exact coverage; transport success is not scientific success |

In `shooting.rs`, `original_candidate` combines native usability with fresh quality; completion evidence separately includes checks, closure requirements, endpoint and requested-sample coverage. This is a strength to preserve. It is not qualification of all integration methods or property derivatives.

The execution concern is qualitative and concrete. Stable work repeats before reuse; result-table boundaries induce crossings; recent inner attempts displace explanatory owners; nested async polling consumes a finite caller stack. No cost model, complete instrumentation inventory or capacity SLA is needed to identify these mechanisms.

## 4. Findings and positions on AUD39-08–15

### <a id="f01"></a>F01 — Result-analysis composition has an unfit ordinary caller-stack route

**Cause and evidence — Interface-checked; historical Tested/Measured:** `workflow/analyses.rs::Runtime::result_analysis` embeds protected result reads, dispatch, artifact promotion, artifact reads, graph construction and activation in nested async composition. Both default result audit cases abort on `thread 'main'` after simulation completion. The unchanged publication workload completes under the separately recorded larger main-stack condition.

The retained development binary’s large poll-prologue reservations support finite stack demand, but do not identify the exact faulting frame or prove that reported frame sizes add linearly. Inspected graph traversal is finite/iterative; the bounded search found no direct analysis/promotion self-recursion, which is narrower than proving all recursion absent.

Python is exposed through `blocking_on_with_interrupted_value`: it boxes a borrowed future and invokes `Runtime::block_on` inside `py.detach`. Tokio 1.53.2 polls that root on the calling thread; runtime worker-stack configuration does not resize it. The 100 ms timeout is cooperative and polls the inner future before its delay. Neither timeout nor `Box::pin` guarantees bounded nested poll frames. **No production Python crash has been observed.** [Tokio’s versioned contract](https://docs.rs/tokio/1.53.2/tokio/runtime/struct.Runtime.html#method.block_on) agrees with inspected pinned source.

**Consequence:** S05/S06 can terminate the process during supported result analysis despite completed science. AP-05/AP-07 and G9 fail at the caller boundary.

**Correction — Proposed:** first narrow async state/composition lifetimes and selectively separate large child futures/state. Compare an explicit execution boundary only if needed. Spawning requires owned `Send + 'static` inputs and adds cancellation, detach/abort/join, context and recovery obligations. A larger-stack thread relocates demand and introduces another lifecycle; it is not automatically the simplest remedy.

**Preserve:** borrowed-lifetime safety, protected evidence, original response meaning, artifact uncertainty/recovery keys, exact ACKs, graph oracles and actual drain.

**Verification:** run both original full cases on their ordinary stack with unchanged assertions and cleanup; exercise the Python caller boundary and interruption/recovery behavior. Heap placement alone is not acceptance evidence.

**AUD39-14:** upheld as the highest functional blocker; narrow exact frame attribution and Python failure claims as above.

### <a id="f02"></a>F02 — Some workflow consumers discard already-owned preparation

**Cause and evidence — Interface-checked:** the benchmark selects `.analysis` from `declared_execution`, then calls `prepare_analysis`. Declared execution has already prepared and retained the model. `resolve_case` extends demand with case paths and prepares again; complete demand in the basis legitimately makes a different query key.

Python solve preparation, declared studies and declared dynamics already retain their model. Python diagnose variants detach analysis and call `prepare_diagnostics`, which resolves again. Diagnostics deliberately uses `allow_missing_free=true`; `prepare_declared` requires an authored solve procedure and passes `false`.

**Consequence:** S01/S02 pays avoidable preparation in these compositions and the benchmark misrepresents the ordinary retained route. AP-03/AP-07 are violated in the affected consumers. The cold 266.38 s phase includes both preparation and start; it does not isolate the second preparation.

**Correction — Proposed:** consume retained declared preparation for the benchmark where its edit obligations permit. Give diagnostic composition a retained-model path preserving diagnostic applicability and missing-input policy. A domain function is sufficient; a generic cache is unnecessary.

**Preserve:** revision/instance/binding/context/demand checks, original edited-result assertions and legitimate differing experiment/horizon demands. Do not force diagnostic inspection through solve admission.

**Verification:** compare retained/detached compositions with identical scientific edits; show which preparation disappears and which dependencies still invalidate. Include diagnostics with missing free values and non-solve authored procedures.

**AUD39-08:** upheld and narrowed to affected consumers; reject universal duplicate-production-solve and whole-phase savings claims.

### <a id="f03"></a>F03 — Empty scalar index checking constructs an unused environment

**Cause and evidence — Interface-checked:** `expression.rs::check_member_indices` validates equal arity, constructs `declaration_environment`, then executes an empty zip when both index lists are empty. The environment enumerates named types and resolves ancestor parameters, coordinate arguments and function arguments.

**Consequence:** ordinary scalar references repeat irrelevant environment work as model size grows. AP-07 is violated by this exact unused-work path. Sampled frames support exercise of relevant code, not exclusive CPU attribution.

**Correction — Proposed:** return after successful empty arity validation, provided enclosing checked-package validity supplies the required premise. This is a narrow checker correction.

**Preserve:** nonempty sequential index-domain validation, error behavior, contextual/refined references, guards and unindexed `Type::Indexed` exports.

**Verification:** focused checker controls for scalar, contextual, refined, indexed and lowered-function cases; directly distinguish eliminated environment construction. No store or solver is required.

**AUD39-09:** upheld; reject a persistent package cache as necessary for this correction.

### <a id="f04"></a>F04 — Selected-source lookup repeats allocation and inherited/guarded expansion

**Cause and evidence — Interface-checked:** `canonical.rs::SourceIndex` allocates probing keys, clones candidate/base vectors and traverses guarded/base members; `selected_source::resolve_segments` repeats lookup across lexical ancestors and path candidates. The inspected index is operation-local. This is distinct from the removed root-wide import scan.

**Consequence:** S04 and larger inherited/guarded namespaces amplify source-admission work. Avoidable key/vector allocation is established; the best reuse boundary and overall timing contribution remain unresolved.

**Correction — Proposed:** start with borrowed-key/nested-map access and reduced copying. Consider completed root-result reuse only under an explicit unchanged frontier and probing context.

The current recursion cut returns `Absent` for an active key. Nested **positive** results can also depend on that active path. Therefore “cache positive results” is insufficient. Define both insertion and consultation context; a root result cannot be consulted indiscriminately inside recursive probes. Empty-context memoization reuses an observation of the present resolver, not an exact graph-closure certificate.

**Preserve:** direct precedence, candidate ordering/multiplicity, inherited/guarded deduplication, potential guard representatives, shadowing, protected absence and `Pending` acquisition.

**Verification:** compare ingress, local selection, delta and replay on cycles, diamonds, guarded members, pending acquisition and frontier changes. Borrowed-key-only improvement remains viable if memo semantics do not justify their machinery.

**AUD39-10:** upheld as a bounded execution concern; reject broad cross-run caching or SCC replacement without further evidence.

### <a id="f05"></a>F05 — File packing unnecessarily determines metadata submission, but batching must separate ownership stages

**Cause and evidence — Interface-checked:** `result_projection.rs::ResultAppendQueue` flushes a homogeneous file on table changes/finish and immediately appends its metadata. `canonical_execution::append_result_blocks` already accepts bounded items across files/result sets, enforces a common fence and ordered per-set ordinals, invokes `append_many`, and checks every exact receipt or settles each original operation.

**Consequence:** S02/S06 pays additional metadata crossings because a real file-schema boundary becomes a metadata-transaction boundary. The retained 100 builders/232 statements describe one whole completion owner; they do not quantify publication-only or avoidable time.

**Correction — Proposed:** keep homogeneous file packing, then queue bounded already-sealed **nonempty** metadata through the existing append owner. Empty declarations retain their separate opaque ACK initially. Artifact grouping has a different lifecycle and should remain separate.

A critical preservation constraint follows from current ownership: `flush` reserves `3 * MAX_FILE_BYTES` and passes that owner into `ResultPayload::from_sealed_owned`, although the sealed payload contains no resident bytes. Extending its lifetime across a metadata group could retain large per-file allowances and defeat the intended working-set improvement. Transfer to appropriately charged small metadata ownership only after encoding, sealing and actual file/transport drain; do not release credits while work remains live.

**Preserve:** exact file intent/payload binding, same-fence limits, per-set order, original clocks, every demanded empty/nonempty output and the full ACK inventory. Flush at genuine demanded acknowledgement/live-prefix boundaries; an existing `finish` cannot be delayed if a consumer needs its ACK.

**Verification:** interleaved sets/files, exact replay, reply loss, gap/conflict/deadline rejection, empties and interruption, plus bounded simultaneous encoder/metadata ownership under the original observer pool.

**AUD39-11:** upheld and narrowed. No unlike-schema IPC merge, cross-run transaction, new protocol or server replacement is selected.

### <a id="f06"></a>F06 — Generic diagnostic retention/export loses responsibility summaries

**Cause and evidence — Interface-checked; historical Measured:** diagnostic admission and metric reservation can trim completed nodes; relationships retain weak references. Runtime publication favors active then newest completed owners within a bounded image.

The inspected fit checkpoint links allocated `fit.execution` 85 from `math.job` 84 with nonempty kind and `available=false`. The source sets that availability from weak-reference upgrade: this is retention/lifetime loss, not exporter omission alone. Shooting instead links available completed `math.job` 60 without its payload, supporting a separate export-loss mechanism. Exact event ordering is unavailable because snapshots are not one catalogue cut.

Fit/shooting owners already propagate context and record trial/cache/upgrade/experiment/window facts. Missing propagation is not supported as the primary cause. Recent inner attempts dominate the retained images.

**Consequence:** S02/S03 completes science but loses the cumulative explanation of the responsibility. The operational model is insufficiently realized by retention/export policy: AP-04/AP-07 fail in this diagnostic scope. Explicit unknown/loss remains truthful, but does not supply useful diagnosis.

**Correction — Proposed:** retain selected producer-owned responsibility summaries through bounded terminal consumption, and give compact summaries export priority over repetitive completed attempts. Keep bounded exemplars/recent history and explicit loss. Define admission/refusal, byte bounds and release.

The producing composition declares summary purpose, eligibility and lifetime through a generic diagnostic contract. The hub must not infer them by parsing `fit.*`/`shooting.*` names or importing workflow types. A new workflow obtains the same retention semantics through its owned composition; no central workflow-name registry is required. Absent or unrecognized summary roles retain ordinary bounded behavior and explicit partiality rather than an invented retention guarantee. The concrete type/API remains open for later design.

**Preserve:** observation-only ownership; no scientific payload/cancellation leases; gauge versus cumulative semantics, resets, freshness, relationship identity and unknown work. Never reconstruct totals by summing overlapping parent/child or final/strategy reports.

**Verification:** inner-attempt-heavy fit/shooting controls preserve meaningful cumulative summaries under existing finite bounds, including refusal and release. Separate retention and export controls. A tracing exporter cannot recover facts discarded before export.

**AUD39-12:** upheld with the two distinct losses. Collector terminal traversal is already repaired; it does not close producer retention/export.

### <a id="f07"></a>F07 — Catalogue discovery and live-process lookup are insufficient for short-run terminal observation

**Cause and evidence — Interface-checked:** the collector now carries its pagination cursor and rotates operation reads. This repairs the old repeated-prefix starvation. Nevertheless receiver association still depends on catalogue history and obtaining `/proc` start ticks; that lookup precedes cached terminal inspection and can fail after successful drain.

**Consequence:** S02/S03 can miss a short receiver or fail to copy an already-associated last checkpoint. Scientific success and native drain remain separate; absent caller native joins do not describe external work.

**Correction — Proposed:** receiver readiness should expose an exact diagnostic locator tied to its real process/database identity. After a validated live association, terminal inspection may use the retained locator and embedded checkpoint identity without requiring the process to remain alive.

**Preserve:** process-instance identity, explicit partiality/staleness and last-published-prefix wording. A locator grants no service-stop, release or scientific authority.

**Verification:** short-lived receivers, historical catalogue growth, PID reuse/mismatch, missing checkpoint and post-drain prefix reads. Existing collector unit tests establish only their named behavior. The cursor/rotation repair was **not followed by another managed scientific replay**; the historical case therefore neither qualifies assembled observation after the repair nor disproves that repair. A subsequent managed replay must establish the corrected route’s actual coverage.

**AUD39-13:** uphold remaining handoff/terminal issues; recognize pagination/rotation repairs as delivered. Reject retrospective profiling coverage.

### <a id="f08"></a>F08 — Warm lookup pays structural and attribution reconstruction before reuse

**Cause and evidence — Interface-checked:** the correct owner is `workflow/modeling/views.rs::bound_case`. It calls `bound_structure` before cache lookup. That compiler method copies variables, applies overlays/domain admission, selects/clones rows and instances, and builds `CaseStructure`. `attribute_modeling_view` reserves and rebuilds the occurrence map before equality comparison.

**Consequence:** S01 repeats immutable construction even when the retained mathematical view is reused. AP-07 is violated in that physical route; latency materiality remains unmeasured.

**Correction — Proposed:** separate a complete compact lookup identity from miss-only expansion, consuming existing immutable prepared products. Keep fresh state/domain tightenings and value rebinding. Attribution needs an independently valid retained occurrence identity: equal mathematics does not prove equal current spans.

**Preserve:** fixed/discrete/bound semantics, ordering, complete context/demand/physical dependencies, revision spans and current permission/protection.

**Verification:** distinguish hit-path construction from miss work; compare value, bound/fixed-state, structural and span-only edits. No owner-only cache key. The duplicate observation construction on a miss was already removed.

**AUD39-15:** upheld with corrected source location and narrow mechanism. Reject attribution equality inferred from mathematical equality and any claimed 123-second saving.

## 5. Foundation and independent gate judgments

| Foundation | Verdict | Scoped basis |
|---|---|---|
| AP-01 Separation | **Satisfied** | Semantic preparation, numerical execution, publication and observation have identifiable owners; proposed corrections retain them. |
| AP-02 Stable contracts | **Unresolved** | Exact publication contracts are strong; a replacement execution boundary must settle borrowed inputs, cancellation and acknowledged recovery. |
| AP-03 Composition | **Violated** | Affected declared/diagnostic consumers discard retained preparation and reconstruct its interpretation path, F02. |
| AP-04 Domain model/authority | **Violated** | Scientific distinctions are substantially explicit, but generic retention/export does not realize responsibility-level diagnostic behavior, F06. |
| AP-05 Explicit constraints | **Violated** | Ordinary result analysis relies on an unenforced caller-stack premise, F01. |
| AP-06 Local reasoning/testability | **Satisfied in inspected scope** | Checker, lookup and diagnostic policies admit local controls; transaction/native guarantees legitimately need integration. No new contrary dependency was established. |
| AP-07 Execution fit | **Violated** | Demonstrated caller failure and source-established repeated work/crossings, F01–F03/F05/F08; diagnostic lifecycle mismatch, F06–F07. |

| Gate | Judgment | Evidence and limitation |
|---|---|---|
| G1 Authority | **Pass, inspected contracts** | No competing mutable model/result authority established; summaries remain producer-owned. |
| G2 Semantic fidelity | **Pass, inspected representations** | Missing/pending, empty/nonempty, ACK/seal and unavailable/zero distinctions are explicit. Proposed memo/batching must preserve them. |
| G3 Validity | **Unresolved** | Relevant admission/assessment exists; complete physical/provider/formulation validity was not traced. |
| G4 Hidden behavior | **Pass, inspected boundaries** | Preparation, effects and passive diagnostics have discoverable owners; inspection does not authorize science. |
| G5 Consistency/recovery | **Unresolved for combined route** | Exact append settlement is strong; aborting analysis and any replacement lifecycle require end-to-end recovery evidence. |
| G6 Transformation/reuse | **Unresolved for corrections** | No current unsound cache established. Proposed root memoization and compact keys require complete contextual equivalence. |
| G7 Truthful capability | **Fail for ordinary combined result route** | Default result-analysis support lacks a demonstrated usable execution route; separate successful conditions cannot qualify it. Diagnostic omissions remain explicitly unknown. |
| G8 Library leverage | **Pass, inspected alternatives** | No clear bespoke replacement for a fitting generic library capability established. Use existing append, Salsa and native solver owners. |
| G9 Architectural fitness | **Fail** | Individual violated foundations cannot be averaged against scientific successes. |
| PS-G1 Physical consistency | **Unresolved beyond preserved boundary contracts** | No new physical mismatch established; all property envelopes/balances were not requalified. |
| PS-G2 Well-posedness | **Unresolved** | Full structural admission/rejection breadth not inspected or executed. |
| PS-G3 Numerical integrity | **Unresolved beyond selected controls** | Shooting assessment separates native status and original evidence; retained selected tests do not qualify every derivative/domain/status path. |

## 6. Alternatives, relationships and remaining inquiry

The preferred direction is a set of small, compatible corrections under existing owners: retain preparation, skip unused work, improve lookup representation, separate file encoding from metadata submission, preserve responsibility summaries, and narrow large async composition.

Library alternatives support that direction:

- **Salsa 0.28.4:** useful for complete tracked dependencies and retained structural products; it cannot remove work performed before lookup. Another generic cache adds overlapping ownership.
- **Tokio/futures:** heap ownership and scheduling are different mechanisms. Spawning buys placement but adds lifecycle obligations; `spawn_blocking` is not cancellation-equivalent. Selective composition changes are the simplest first comparison.
- **Tracing/subscriber layers:** useful for generic context and supplied events. Existing typed observation owns scientific counters; wholesale telemetry replacement would not repair discarded facts.
- **Existing SurrealDB append grouping:** already supplies the consumed fenced receipt contract. Generic bulk insertion does not replace it.
- **Native numerical libraries:** retain iteration/globalization/factor ownership. Compiled scalar evaluation, sparse reuse and Krylov methods remain capability-specific alternatives, not conclusions from preparation/publication overhead.

The preferred F08 correction preserves the public complete view-key contract and separates lookup from expansion internally; it does not select a new scientific/view identity. The preferred F01 direction preserves borrowed caller ownership while narrowing async composition and state lifetimes. A changed identity contract or spawned/dedicated bridge remains an **unselected alternative**. If later selected, it must receive a concrete decision covering changed dependencies, ownership, cancellation, joining and recovery; it is not a current rule-impact recommendation.

F02/F03/F04/F08 share stable-work repetition, but have different dependencies and verification. F05 and F06 share the mistake of letting incidental physical ordering determine useful lifecycle units; their guarantees remain independent. F01 is a separate placement/composition failure.

Consequence priority starts with F01, then diagnostic explanation and repeated supported-work costs. Prerequisite order differs: metadata ownership transfer precedes cross-file batching; summary selection/retention precedes export priority; exact locator meaning precedes readiness wiring; complete lookup identity precedes miss-only construction. These are design dependencies, not an implementation schedule.

Remaining investigations should settle decisions, not manufacture proof machinery:

1. Locate the consequential poll-frame boundary and compare narrowing with explicit placement.
2. Determine which result `finish` calls represent demanded ACK/live-prefix boundaries.
3. Establish summary selection, refusal and terminal release within existing diagnostic bounds.
4. Settle probing-context memo semantics before introducing lookup reuse.
5. Establish hit-path construction costs before choosing additional retention.
6. Investigate `Evaluator::typed_result` repeated subtree inference with shape-sensitive controls. Source shows repeated inference; dominant quadratic cost is unmeasured. Preserve integer overflow/integrality, physical typing, optional/refined paths and binders.
7. Treat duplicate physical encoding and fresh protection separately: carrying owned bytes may help, but matching content does not remove current eligibility obligations.

No touched unit/property model receives new conformance or reference-validation status. Existing selected controls remain their own evidence; all correction verification is **not_run**.

## 7. Rule impacts, disposition and decision

Recommendations remain pending later plan creation and adoption. The review changes no rule, implementation status or historical audit result. **39d owns the audit observations. The new F01–F08 recommendations have no assigned implementation disposition owner yet.** A subsequently authorized plan should own adopted corrective work and link these stable findings.

### <a id="rc01"></a>RC01 — Diagnostic retention and export priority

**Rule/location:** blueprint §23.1, `operations-and-validation.md`: bounded history favors active owners over completed history. The current hub/export realizes completed eviction and active/newest priority.

**Proposed change:** provide a bounded tier for selected producer-owned responsibility summaries through declared terminal consumption, and give their compact representation a credible export route. Preserve active work, truthful completion, admission/refusal, loss reporting and a finite release point. Required by F06.

The summary’s publication lifetime is distinct from the scientific operation’s execution lifetime. A completed fit, shooting job or native attempt must remain completed. Keeping science falsely active solely to avoid completed-node eviction is not an acceptable realization.

**If kept:** copy compact producer-authored facts into an existing enclosing diagnostic/publication owner that is legitimately still active and can export them within its bounds. Keep the completed child’s actual outcome and distinguish the enclosing owner’s remaining publication work. If no such truthful owner/lifetime exists, retain explicit unavailable/partial diagnosis and leave the target unmet; increasing completed-attempt history does not satisfy it.

### <a id="rc02"></a>RC02 — Receiver readiness diagnostic locator

**Rule/location:** current `primary-receiver.json` registration/readiness contract, its writer/supervision consumers, and `case_audit.py::inspect_managed`; blueprint §23.1 supplies diagnostic identity semantics.

**Proposed change:** readiness carries an exact checkpoint/process locator bound to the registered receiver’s real identity. This supplies the preferred F07 short-run discovery route without catalogue-history dependence.

**If kept:** retain bounded discovery and validated cached terminal-prefix copying, with explicit incomplete association. Reliable early association cannot be claimed from eventual enumeration. The delivered cursor/rotation repair remains a valid bounded improvement; its assembled managed coverage has not been replayed.

The preferred F08 correction preserves the complete view-key contract; internal lookup/expansion improvements do not themselves require a rule change. The preferred F01 correction preserves borrowed caller ownership. Changed identity contracts and spawned/dedicated bridge ownership remain unselected alternatives discussed in section 6, rather than actionable rule impacts.

There is **no required publication-consistency rule change** for F05’s narrow use of the existing grouped append API. Blueprint §20’s demanded ACK/live-prefix flush rule remains a preservation constraint. Combining empty declarations, delaying real ACK demand or changing settlement would require a separately identified decision if later selected. F03 and borrowed-key-only F04 corrections likewise need no blanket ADR.

The combined decision remains **Revise**. The strongest evidence is the default caller failure alongside source-established unnecessary preparation, reconstruction and metadata boundaries. Scientific successes constrain corrections; they do not waive architectural failures. The next consequential decisions are the ordinary caller execution remedy and bounded responsibility-summary lifecycle. Their adoption authorizes neither implementation nor qualification by itself.
