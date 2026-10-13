---
title: Plan38c native reuse and resources target review
date: 2026-10-13
tier: design
purpose: target
standard: Core 3.4; Heuristics for Efficient Architecture 1.0; Process Simulator 1.5
binding: pse-arrow
decision: Accept
evidence: Interface-checked
disposition_owner: docs/plans/38-efficiency-and-solver-target.md
---

# Plan38c native reuse and resources target review

## Decision and scope

<a id="slot-1"></a><a id="slot-12"></a>

**Accept at the target-document level.** The final proposed ADR-0185, architecture revision 162 amendments to blueprint §15.4/§17.6/§18.6/§18.8/§18.9/§21, and Plan38c execution checkpoint define a coherent bounded target. Behavioral/semantic adequacy and architectural fitness are satisfied at this evidence level. No target blocker remains after the request-completion/lane-retirement distinction was incorporated during review.

This accepts the proposed contracts and responsibility boundaries. It does not establish implemented neutral reuse, admit an unqualified native profile, activate the new default, select SPQR, close the original audit findings, or qualify numerical behavior. Assembled scientific, native, durable and performance acceptance remains Plan38g.

The independent reviewer examined `main` at `0a469337fc3a75d756e89405c0c6487e8e7d8b01` plus the uncommitted documents named above, including the final ADR-0185 and §18.8 completion amendments, on 2026-10-13. Criteria were the selected standard and its functional target. Existing ADRs, audit conclusions, accepted rule rows and maintainer defaults supplied context rather than acceptance criteria.

The relevant target is interactive edit/re-solve and independent studies that retain useful native preparation without accidentally inheriting numerical history, while preserving original-space acceptance and bounded deployment resources. The review covers independent-request reset, compatible native retention, explicit starts, resource/lane/child ownership, mutation failure, and the selected-method/rank decision boundary. It considers case-count growth, changing values and profiles, competing resident teams, cancellation and native failure. Whole-codebase architecture, new scientific controllers, storage protocols, physical-model qualification and quantitative performance are excluded.

## Ownership, contracts and semantic distinctions

<a id="slot-2"></a><a id="slot-3"></a>

The target preserves the useful separation between scientific meaning, native integration and physical execution placement.

| Responsibility | Owned contract and boundary | Relevant examined owner |
|---|---|---|
| Scientific request and acceptance | Authored problem class, accuracy, domain/branch obligations and start permission; original assessment remains independent of native success | Blueprint §17.6/§18.6 and Plan38c C1/C3 |
| Native retention and mutation | Exact compatibility; neutral versus numerical state; reset, refusal and invalidation; library-specific update and stop semantics | `pse-backend-native/src/execution.rs::Retained`, `execution/highs.rs`, `kinsol.rs`, `pounce/retained.rs` |
| Lane and deployment lifetime | Exclusive request use, resident charging, active/construction admission, pressure retirement, owner-thread destruction and join | `pse-runtime/src/math/staged.rs::NativeSession`, `serve`, `math/jobs.rs`; blueprint §18.8 |
| Method integration | Complete consumed capability through existing scientific builders and common policy/evidence; no adapter-only completion | Plan38c C3/I-C2/I-C3 |
| Numerical rank evidence | Named convention and original-coordinate interpretation; established SVD meaning remains separate | `pse-math/src/diagnostics.rs`; Plan38c C6/I-C4 |
| Public/generated reports | Project actual reset, resource, stopping and method facts without creating policy or inferring absent counters | Blueprint §21 |

The four retained-state categories are adequate for this boundary: structural/symbolic preparation, neutral workspace, numerical state, and value-sensitive heuristic policy. They explain why matching dimensions, a fresh numeric factor, or a retained wrapper alone cannot authorize independent reuse. Allocation retention grants neither a seed nor continuation. A lane owns physical residency; one request exclusively owns its active scientific execution and mutable request state.

The final completion contract resolves the important lifetime distinction. Request finish waits for request native work and callback borrowers to drain, unsafe state to reset or be destroyed, and request active/construction owners to release. Only accepted neutral state and its resident charges transfer to a parked service lane. Cancelled, fatal and uncertain requests transfer no healthy retained state. Lane retirement and service shutdown additionally require full native/thread-local destruction and owner-thread join before resident/population release. Existing ephemeral paths retain their full-join completion behavior until migrated.

This distinction is necessary because the examined `MathService::solve` currently opens and closes a one-step session before returning its report, and `SolveHandle::finish` documents full teardown and join. Keeping that literal guarantee for every future successful independent request would defeat persistent lanes; releasing lane residency at request finish would undercharge them.

Native mutation has a separate three-way contract: incompatible before mutation, successfully updated, or failed/uncertain after mutation. The last outcome destroys retained state before returning the original typed cause, regardless of `RequireReuse`. This directly addresses the examined HiGHS adapter’s `session.update(...).is_ok()` conversion and the generic retained-session seam. A compatibility refusal cannot stand in for an uncertain partially updated model.

The physical semantics preserved by this amendment are:

| Quantity/model element | Dimension/unit and basis | Convention and validity | Authority |
|---|---|---|---|
| Scalar interval and candidate | Authored variable coordinates and physical units; native normalization is an explicit transport | Admitted interval, continuity/evaluability and branch premises; candidate checked against original residual/domain obligations | Existing scientific builder and original assessment |
| Residual and stopping tolerance | Declared physical residual units and scaled/unscaled meaning | Native interval-width or step termination does not grant scientific permission | Existing numerical policy and original acceptance |
| Rank and singular-mode evidence | Dimensionless rank count over declared row/column coordinates | Method-specific threshold, scaling and ordering; QR evidence is not SVD singular-mode evidence | Existing SVD diagnostics or a separately selected rank tier |

No new unit, property, material-system or balance definition is introduced.

**Well-posedness:** the target retains existing structural/class admission and model identities before native execution. Numerical rank remains value-dependent evidence; it cannot silently replace structural well-posedness or reinterpret square-response tolerance. SPQR remains an investigation with explicit bounded-SVD support if it does not qualify.

## Distinguishing scenarios and execution

<a id="slot-4"></a><a id="slot-5"></a>

| Scenario | Expected target behavior | Boundary and evidence |
|---|---|---|
| S01 — Independent value edit after different predecessor histories | Reuse compatible neutral work; clear numerical setup, value-sensitive policy and request-owned advanced products; preserve explicit seed permission and fresh original assessment | C1/C2/I-C1; existing FERAL and KINSOL seams inspected, new equivalence controls Proposed |
| S02 — Changed pattern, bound classification, profile or runtime/provider context | Rebuild or issue an explicit reuse refusal; dimensions/nnz alone never establish compatibility | C1/C2; complete compatibility belongs to the native integration owner |
| S03 — Partial HiGHS update or failed temporary edit | Restore completely or invalidate; preserve typed mutation cause, including under forbidden rebuild | C5; current error-to-boolean seam independently inspected |
| S04 — Many independent requests under resident/active pressure | Prefer compatible idle lanes, bound residency and queues, release idle active capacity/exclusions, retire idle owners before starvation, retain charges until actual drain/join | C2/C4; current session loop already supplies an extension seam |
| S05 — Cancel, callback failure or late child completion | Drain actual work, invalidate unsafe state, reject late results, preserve original deadline/accounting and authenticated owner identity | C1/C4; child reconstruction/IPC remains Proposed |
| S06 — Replace a profile or add a scalar/rank method | Integration absorbs native details; scientific consumers retain explicit class, branch, acceptance and evidence contracts | C3/C6; no universal solver object or generic iteration implementation is introduced |

For numerical stages, the target preserves these consumed contracts:

| Stage | Formulation, derivatives and scaling | Class/capability | Termination and original check |
|---|---|---|---|
| Retained nonlinear/root/NLP solve | Existing authored guards, declared derivative order and model scaling; independent reset does not alter them | Selected class-specific native profile | Native status and reset facts remain distinct; each new candidate receives original assessment |
| Scalar bracket solve | Admitted continuity/domain/branch interval; no derivative claim is introduced; coordinate and tolerance transport remains explicit | Library-owned TOMS748 scalar bracketing | Fallible charged evaluations, explicit limit/error mapping and original residual acceptance |
| Rank diagnostic | Original row/column identities and declared scaling; method/version/threshold/ordering explicit | Established bounded SVD or separately selected qualified sparse tier | Evidence retains its own numerical meaning; no automatic solution permission or SVD-mode substitution |

Execution fit is credible at the proposed level. Persistent lanes extend the existing session request loop, rather than adding a competing native scheduler. Exact profile affinity and charged parked state avoid retaining every adapter pool indefinitely. Explicit FERAL width and ambient Rayon installation allow placement to reflect the admitted envelope; separately created and transitive teams remain charged. Thread-affine SUNDIALS ownership avoids assuming unqualified migration.

Store-free child lanes have a bounded architectural role for independently reconstructible native solves. They do not serialize arbitrary provider callbacks, acquire canonical worker authority or claim distributed decomposition of one solve. Actual reconstruction, IPC/backpressure, process failure and drain behavior still require implementation evidence.

## Foundations and gates

<a id="slot-6"></a>

All verdicts below concern the final **Proposed target**, informed by inspected interfaces. They are not current implementation qualification.

| Foundation | Verdict | Scoped argument |
|---|---|---|
| AP-01 — Separation of concerns | Satisfied | Scientific policy, native reset/mutation, lane residency and report projection have distinct owners. |
| AP-02 — Stable contracts | Satisfied | Compatibility, reset/refusal, mutation failure and method support are consumed contracts rather than incidental native handles. |
| AP-03 — Composition | Satisfied | Existing builders, policy/evidence and session execution are extended; no duplicated end-to-end solver route is selected. |
| AP-04 — Domain model and authority | Satisfied | Independent request, continuation, explicit start, neutral residency and numerical history remain distinct and govern the proposed operations. |
| AP-05 — Explicit structure | Satisfied | Ownership transitions, reset preconditions, mutation outcomes and retirement obligations are explicit. |
| AP-06 — Local reasoning/testability | Satisfied | Adapter reset/update and lane lifecycle controls can exercise their actual dependencies; a canonical store is not required for those responsibilities. |
| AP-07 — Execution fit | Satisfied | The target removes avoidable reconstruction/crossings while bounding resident pools, nested teams and failure lifetimes. No measured speed or capacity claim is accepted. |

| Gate | Verdict | Evidence or scope reason |
|---|---|---|
| G1 — Authority | Pass | Existing scientific owners govern permission; native and generated observations do not become policy. |
| G2 — Semantic fidelity | Pass | Allocation, history, seed, mutation failure, stopping and rank conventions remain distinct. |
| G3 — Validity | Pass | Full compatibility/reset and method premises precede reuse/selection; unsupported states rebuild or refuse. |
| G4 — Hidden behavior | Pass | Independent reset prevents implicit predecessor policy; source/provider oracles and expired controls cannot park unnoticed. |
| G5 — Consistency/recovery | Pass | Mutation failure invalidates before returning; finish, parked residency and retirement have separate release boundaries. |
| G6 — Transformation/reuse | Pass | Explicit numerical dependencies and independent original acceptance remain; QR cannot silently replace SVD meaning. |
| G7 — Capability truth | Pass | Default activation and profile claims require actual acceptance; unknown counters and unsupported methods remain explicit. |
| G8 — Library leverage | Pass | Libraries retain numerical iteration/globalization/factorization; narrow reset/width seams address integration gaps. |
| G9 — Architectural fitness | Pass | Applicable foundations are satisfied for S01–S06; implementation and quantitative claims remain open. |
| PS-G1 — Physical consistency | Not applicable to new physical definitions | No physical model is changed; original physical/domain/closure checks remain mandatory. |
| PS-G2 — Well-posedness | Pass for the amendment | Structural admission remains separate from numerical rank evidence; no new bypass is selected. |
| PS-G3 — Numerical integrity | Pass for the target contracts | Native status/interval termination cannot grant solution permission; seeds/history and each new-state assessment remain explicit. Runtime adequacy is unqualified. |

## Finding and its target resolution

<a id="slot-7"></a><a id="f01"></a>

**F01 — Request completion and persistent-lane retirement initially shared an incompatible teardown guarantee.**

- **Principles/gates/scenarios:** AP-02, AP-05, AP-07; DP-19/20/24; G2/G5/G9; S01/S04/S05.
- **Observed cause:** the initial amendment selected parked service-owned native state while retained §18.8 required every `SolveHandle::finish` to await native/thread-local destruction and owner-thread join. Current `math/solves.rs::MathService::solve` and `SolveHandle::finish` realize that ephemeral behavior.
- **Consequence:** literal preservation would eliminate cross-request retention; treating request completion as permission to release resident charges would undercharge live native state.
- **Correction:** distinguish request drain/reset and request-owner release from neutral residency transfer, lane retirement and service-shutdown join.
- **Final reviewed target:** ADR-0185 Outcome and blueprint §18.8 now state that distinction, including no healthy transfer from cancelled/fatal/uncertain requests and continued full join on unmigrated ephemeral paths.
- **Resolution boundary:** the target defect is corrected in the reviewed documents. Implementation migration and lifecycle verification remain open in Plan38c C2/C4; this review does not mark those mechanisms Tested.

No other blocking target finding was identified in the bounded inspection.

## Library fit and alternatives

<a id="slot-8"></a><a id="slot-9"></a>

Fresh affected objects remain the simplest safe fallback where neutral reset cannot be established. They preserve supported behavior without requiring every profile to achieve identical retention. Unqualified persistent wrappers are unsuitable because allocation retention does not erase numeric setup or value-sensitive policy.

The existing project POUNCE factory and role-specific FERAL leases are worth preserving. The inspected fork at `ff6386944421e8037069e521222ada9f22d457e3`, including FERAL 0.18.0, retains Auto/MC64 policy and a persistent pool. Narrow library-owned reset and explicit-width interfaces address those gaps without copying factorization or MC64. Restoring construction policy may require rebuilding symbolics when ordering escalated; the target explicitly permits that.

HiGHS bulk-vector updates are a proportionate first step. An optional uploaded-content witness earns additional machinery only if it avoids meaningful unchanged work and commits after all mutations succeed. Full model verification and original assessment remain, so this does not establish end-to-end work proportional only to changed entries.

The selected thin scalar-root route has a fitting library contract: [root1d 0.4.1 TOMS748 documentation](https://docs.rs/root1d/0.4.1/root1d/fn.toms748.html) provides fallible callbacks and bracket tolerance control. It also documents that the default iteration cap can return a candidate without an error. The binding must enable `maxiter_err(true)` or preserve equivalent typed iteration-limit evidence under the existing common stop contract. This is an implementation acceptance condition, not a new target finding. Context7 returned unrelated matches on two resolution attempts; exact-version library documentation supplied the fallback inspection. This does not establish a locked or compiled repository integration.

A separately named sparse-rank tier is preferable to pretending SPQR reproduces SVD singular values and modes. Concrete SPQR binding/ABI/index-width/memory qualification remains unselected. Bounded SVD with explicit limits remains a coherent supported outcome if the investigation does not qualify a sparse tier.

## Verification and evidence limits

<a id="slot-10"></a>

| Claim | Evidence | Result or remaining obligation |
|---|---|---|
| Existing session-loop, exclusive retention, conditional KINSOL setup and FERAL policy/pool seams | Interface-checked | Source inspected; no runtime neutrality or width behavior established |
| HiGHS mutation-error suppression and retained-session failure seam | Interface-checked | Diagnosis supports typed invalidation; correction and injected-failure controls remain Proposed |
| SVD threshold and singular-mode meaning | Interface-checked | `diagnostics.rs` independently inspected; no SPQR equivalence claimed |
| Final request-finish/lane-retirement contract | Proposed, reviewed | Target distinction sufficient; actual transfer/drain/join controls remain open |
| Scalar-root library seam | Interface-checked | Exact-version documentation inspected; integration, stopping and original-space controls remain Proposed |
| Default activation | Proposed | Every reachable profile must have accepted reset or fresh fallback, with persistent HiGHS failure handling integrated |
| Scientific/native/performance and assembled acceptance | Not established | Remains Plan38g |

No writes, builds, tests, probes, services, canonical-store access or measurements were performed. No unit/property model was changed, so no new model-conformance result is claimed. Existing tests and historical audit receipts were not promoted into current qualification.

## Rule impacts and disposition

<a id="slot-11"></a>

**Additional rule impacts: none.** The reviewed changes and F01 correction fit accepted RC06/07, S-RC03–07/11/12, ARC01, R38-01 and the explicit maintainer selections for narrow POUNCE/FERAL enhancements and conditional neutral `AllowRebuild` default.

The finish/retirement amendment is part of the accepted native-lifetime change. Remaining reconciliation of older attempt-lifetime or pool-width descriptions must preserve the concrete ADR-0185 target; it does not require a new product choice. No new crate, host partition, storage topology, HSL profile or universal solver language is recommended.

Plan38 owns finding dispositions; Plan38c owns implementation and investigation outcomes. F01’s target correction is incorporated, while C2/C4 implementation acceptance remains open. Concrete rank-binding selection retains its decision route before production implementation.

**Overall decision: Accept for the final bounded target documents, at Proposed/Interface-checked evidence.** The next acceptance boundaries are actual profile reset/fallback and mutation-failure controls, lane/resource/child lifetime behavior, and selected scientific consumers. Default activation and Plan38g qualification remain conditional on their own evidence.
