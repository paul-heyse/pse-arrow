---
title: Simulation pipeline efficiency and effectiveness
status: in-progress
date: 2026-10-15
adrs: [ADR-0190, ADR-0191]
review_sources:
  - docs/design_review/reviews/design_review_simulation-pipeline-efficiency-and-effectiveness_2026-10-15.md
scenario_sources:
  - docs/design_review/reviews/design_review_simulation-pipeline-efficiency-and-effectiveness_2026-10-15.md
doc_owner: docs/plans/41-simulation-pipeline-efficiency-and-effectiveness.md
doc_topics: [current-work, design, preparation, libraries, validation]
---

# Plan 41: Simulation pipeline efficiency and effectiveness

## Purpose, basis and boundary

This standalone series turns the [simulation-pipeline review](../design_review/reviews/design_review_simulation-pipeline-efficiency-and-effectiveness_2026-10-15.md) and its [supporting inquiries](../design_review/evidence/simulation-pipeline-efficiency-and-effectiveness-2026-10-15/README.md) into corrective work and bounded numerical/protocol investigations. It covers F01–F14, O01–O08, RC01–RC04 and S01–S10. The target serves interactive edit/re-solve, studies, recycles, dynamics, fitting and extension from the same authored models, with responsibilities testable using their actual dependencies.

**Interface-checked basis:** the review's source/interface assessment at `a08f5fa091db1840fc95b46c4d4f849ce4c5de62` plus the inspected concurrent working tree, with focused authoring reads of the affected owners. **Proposed:** the targets, packages and future acceptance in this series. Authoring supplies no new product test, numerical qualification or measured speedup. The standard remains Core 3.4, Efficient Architecture 1.0, Process Simulator 1.5 and the pse-arrow binding. Relevant architectural owners are blueprint §0/§0.2/§0.5, §7/§14, §15–§21 and §23/§24.

Existing plans, their native statuses and the original review remain unchanged. This series owns the disposition of its newly adopted review items; AUD39 observations remain historical evidence at their existing owner. It neither supersedes another plan nor imports the rest of its qualification campaign. After this work is implemented, 41f F6/F7 reconcile relevant existing documents through their proper owners. Eliminated environment/provenance proof is not deferred or reinstated. Scientific source/result identity, original physical assessment, Arrow correctness validation and actual cancellation/drain remain necessary.

## Documents and combined target

| Document | Responsibility |
| --- | --- |
| [41a: Source, specialization and selected preparation](41a-source-specialization-and-selected-preparation.md) | F01–F05: use maintained access paths, share immutable premises and prepare actual selected demand without stale mutable overlays. |
| [41b: Cancellation, shared work and admission](41b-cancellation-shared-work-and-admission.md) | F06–F08/O07: hierarchical stopping, waiter isolation, finite shared production, cancellable compiler serialization and justified idle-pressure cooperation. |
| [41c: Studies, response artifacts and result resources](41c-studies-response-artifacts-and-result-resources.md) | F09–F11/O02: honest local/durable sources, family-qualified response evidence, exact metadata ownership and sound protocol grouping. |
| [41d: Native storage and workflow worker reuse](41d-native-storage-and-workflow-worker-reuse.md) | F12–F13/O05/O06: complete sparse layouts, method-shaped allowances, reusable owner-affine workers and qualified routing/presolve alternatives. |
| [41e: Derivative reuse and evaluator capabilities](41e-derivative-reuse-and-evaluator-capabilities.md) | F14/O01/O03/O04: completed exact curvature, same-point capabilities, selected derivative actions and explicit evaluator qualification. |
| [41f: Causal diagnostics, qualification and closeout](41f-causal-diagnostics-qualification-and-closeout.md) | O08: incremental observations, assembled acceptance, scope-end checks and later documentation reconciliation. |

Keep one routed solve pipeline. Authored definitions and physical context own scientific meaning; compiler/math owners supply immutable preparation; native libraries own iteration, globalization, integration and factors; workflow owners compose finite operations; independent original assessment owns usability. Persistence and graph representation do not certify numerical meaning. Existing multi-output/CSE, achieved-demand caches, factor/reset/update APIs, native joins, compatible block batching and original completion are foundations, not missing features to recreate. Frozen inner starts/branches remain frozen in recycle maps; retained workers must not introduce hidden continuation. E3 also owns the supporting library inquiry's unchanged-sample elimination/symbolic reuse and qualified multi-RHS eligibility questions.

Extend useful existing owners rather than introducing a generic model wrapper, parallel cache hierarchy, scheduler or numerical policy interpreter. A new unit/property remains an authored extension; a solver change stays at its capability adapter; transport changes stay at the storage boundary. Public local-study and response contracts are the consequential interface changes. Other corrections principally change internal access, allocation and lifetime realization.

## Rule decisions

The decisions below record individual operator confirmation. Pending is not acceptance. Rejection preserves the current rule and requires adapting dependent work, recording the remaining target limitation. Accepting a bounded inquiry does not pre-authorize an unspecified contract expansion it may discover.

| Rule impact | Decision and date | Recommended target and amendment route | Dependent work / retained-rule consequence |
| --- | --- | --- | --- |
| [RC01](../design_review/reviews/design_review_simulation-pipeline-efficiency-and-effectiveness_2026-10-15.md#rc01) | Accepted by operator — 2026-10-15 | Accept genuine runtime-local admitted model/physical ownership alongside portable recorded study descriptors, common scientific/occurrence policy and explicit durable publication. Before C1 migration, amend blueprint §0.5/§19.3/§21/§21.1 through the decision/design route; a changed Python boundary needs the required proposed ADR before implementation. Version affected persisted documents deliberately, without a compatibility execution path. | C1 and its public local/durable acceptance. Rejecting the boundary leaves F09 unresolved; storage initialization or fabricated receipts are not substitutes. |
| [RC02](../design_review/reviews/design_review_simulation-pipeline-efficiency-and-effectiveness_2026-10-15.md#rc02) | Accepted by operator — 2026-10-15 | Accept family-qualified dispatch using existing Trajectory and LocalAnalysis promises, preserving scalar validity admission. Retain current certification contracts. C0/C2 may identify a missing stronger transient/persistent promise, but must return its concrete schema/validity impact for a separate decision before reliance; do not authorize that extension by inference. | C2's dispatcher correction can use existing kinds without a new artifact authority. Stronger certification is unavailable unless separately established and amended at blueprint §19.2/§20.4 with the required decision route. |
| [RC03](../design_review/reviews/design_review_simulation-pipeline-efficiency-and-effectiveness_2026-10-15.md#rc03) | Accepted by operator — 2026-10-15 | Accept bounded qualification of an explicit compiled/batched evaluator target. Adoption requires exact-release fit for floating-point/no-FMA/ISA, cancellation, provider errors, executable/scratch lifetime and useful repeated demand. If it fits, install the supported-profile/feature amendment to blueprint §25 before adoption; if a scientific guarantee must change, return a new concrete decision instead. No global enablement or tolerance change. | E4 inquiry can proceed under current rules; production adoption requires its positive decision and G0 amendment. Interpreter corrections and F01–F14 remain independent if the candidate is refused. |
| [RC04](../design_review/reviews/design_review_simulation-pipeline-efficiency-and-effectiveness_2026-10-15.md#rc04) | Accepted by operator — 2026-10-15 | Accept bounded investigation of native original-reaction provenance versus an explicitly primal-only tightening route. Retain current disabled execution/full-reaction eligibility and useful infeasibility prechecks while investigating. A concrete primal-only output contract or new full-reaction capability must return for decision and blueprint §15.3–§15.5/§18 amendment before activation. | D4 must settle and record a route, including a justified no-adoption result. No silent FBBT enablement, fabricated reactions or unqualified dual/KKT outputs. |

No automatic rule change is assigned to indexed access, overlays, selected preparation, cancellation, admission, metadata transfer, sparse storage, neutral worker reuse or completed curvature caching. Shared-producer policy is explicit but never renews original caller clocks or inserts deadlines into scientific identity. Same-point retention does not relax ADR-0186's fresh-IVP rule for changed trials. Additional concrete impacts discovered during execution return through G0 before dependent changes.

## Coordination packages

| Packet | Prerequisite | Delivery and acceptance | State |
| --- | --- | --- | --- |
| G0 | Concrete operator decisions in the rule table | Record decisions/date/routes; install required accepted amendments before their dependent production changes. Keep conditional inquiries under current rules until their actual adoption decision. Preserve immutable ADRs; ordinary refactors/dependency changes receive no invented ADR. | Completed — accepted RC01–RC04 target routes installed; ADR-0190/0191 remain proposed under the decision-PR rule |
| G1 | Companion delivery and investigation outcomes | Reconcile combined API/lifetime/invalidation/resource contracts, actual consumers and deletion scope. Settle every bounded inquiry or obtain an explicit scope change; expose remaining required failures before F2 selects assembled acceptance. This is integration readiness, not another status ledger. | Completed — current generated/native Python and reference TOML consumers migrated; all investigation choices and source/lifetime contracts handed to assembled acceptance |

## Finding dispositions

| Finding reference | Scenario reference | Disposition | Decision / work owner | Evidence or revisit trigger |
| --- | --- | --- | --- | --- |
| [F01](../design_review/reviews/design_review_simulation-pipeline-efficiency-and-effectiveness_2026-10-15.md#f01) | S01/S10 | resolved | 41a A1 | **Tested — targeted correction accepted 2026-10-15:** [Outcome](41a-source-specialization-and-selected-preparation.md#outcome--2026-10-15). Indexed import candidates match completeness, precedence and declaration ordering while unrelated roots are not examined. Full-series qualification remains at 41f. |
| [F02](../design_review/reviews/design_review_simulation-pipeline-efficiency-and-effectiveness_2026-10-15.md#f02) | S01/S10 | resolved | 41a A2 | **Tested — targeted correction accepted 2026-10-15:** [Outcome](41a-source-specialization-and-selected-preparation.md#outcome--2026-10-15). Stable lexical bases are shared; argument/value/reader/refinement/synthetic overlays remain fresh and extension-local. Full-series qualification remains at 41f. |
| [F03](../design_review/reviews/design_review_simulation-pipeline-efficiency-and-effectiveness_2026-10-15.md#f03) | S01/S10 | resolved | 41a A3 | **Tested — targeted correction accepted 2026-10-15:** [Outcome](41a-source-specialization-and-selected-preparation.md#outcome--2026-10-15). Function overrides borrow unrelated package state, use one lookup authority and preserve evolving lowered functions. Full-series qualification remains at 41f. |
| [F04](../design_review/reviews/design_review_simulation-pipeline-efficiency-and-effectiveness_2026-10-15.md#f04) | S01 | resolved | 41a A4 | **Tested — targeted correction accepted 2026-10-15:** [Outcome](41a-source-specialization-and-selected-preparation.md#outcome--2026-10-15). Selected preparation avoids unnecessary broad construction and duplicate observation structure without losing coupled provider support. Full-series qualification remains at 41f. |
| [F05](../design_review/reviews/design_review_simulation-pipeline-efficiency-and-effectiveness_2026-10-15.md#f05) | S01 | resolved | 41a A5 | **Tested — targeted correction accepted 2026-10-15:** [Outcome](41a-source-specialization-and-selected-preparation.md#outcome--2026-10-15). Projection lookup is shared once; expression-local bindings, order and validity remain exact. Full-series qualification remains at 41f. |
| [F06](../design_review/reviews/design_review_simulation-pipeline-efficiency-and-effectiveness_2026-10-15.md#f06) | S02/S09 | resolved | 41b B1 | **Tested — targeted correction accepted 2026-10-15:** [Outcome](41b-cancellation-shared-work-and-admission.md#outcome--2026-10-15). Parent cancellation propagates, child/sibling isolation holds, study frontier bookkeeping completes once and native work really drains. Full-series qualification remains at 41f. |
| [F07](../design_review/reviews/design_review_simulation-pipeline-efficiency-and-effectiveness_2026-10-15.md#f07) | S02 | resolved | 41b B2 | **Tested — targeted correction accepted 2026-10-15:** [Outcome](41b-cancellation-shared-work-and-admission.md#outcome--2026-10-15). Unequal-deadline waiters remain isolated, producer policy is finite and last-waiter retirement joins issued work. Full-series qualification remains at 41f. |
| [F08](../design_review/reviews/design_review_simulation-pipeline-efficiency-and-effectiveness_2026-10-15.md#f08) | S02 | resolved | 41b B3 | **Tested — targeted correction accepted 2026-10-15:** [Outcome](41b-cancellation-shared-work-and-admission.md#outcome--2026-10-15). Cancellable compiler admission precedes scarce worker dispatch; atomic head/use and publication remain coherent. Full-series qualification remains at 41f. |
| [F09](../design_review/reviews/design_review_simulation-pipeline-efficiency-and-effectiveness_2026-10-15.md#f09) | S03/S10 | resolved | G0/RC01; 41c C1/C5 | **Tested — targeted correction accepted 2026-10-15:** [41c Outcome](41c-studies-response-artifacts-and-result-resources.md#outcome--2026-10-15). Public no-store physical-study ownership, explicit genuine recording/refusal, lazy attribution and retained source/result lifetimes passed. Full 41f qualification remains open. |
| [F10](../design_review/reviews/design_review_simulation-pipeline-efficiency-and-effectiveness_2026-10-15.md#f10) | S05 | resolved | 41c C2/C5 | **Tested — targeted correction accepted 2026-10-15:** [41c Outcome](41c-studies-response-artifacts-and-result-resources.md#outcome--2026-10-15). Root/optimal/transient family dispatch, valid/withheld premises and retained evidence/coordinate controls passed without stronger certification. Full 41f qualification remains open. |
| [F11](../design_review/reviews/design_review_simulation-pipeline-efficiency-and-effectiveness_2026-10-15.md#f11) | S04 | resolved | 41c C3/C5 | **Tested — targeted correction accepted 2026-10-15:** [41c Outcome](41c-studies-response-artifacts-and-result-resources.md#outcome--2026-10-15). Nonempty multipage/file-open reads passed with the original observer pool/compiler leases; SDK suffix, retained aliases, exact cleanup and actual managed reads passed. Full 41f qualification remains open. |
| [F12](../design_review/reviews/design_review_simulation-pipeline-efficiency-and-effectiveness_2026-10-15.md#f12) | S06 | resolved | 41d D0/D1/D6 | **Tested — targeted correction accepted 2026-10-16:** [Outcome](41d-native-storage-and-workflow-worker-reuse.md#outcome--2026-10-16). Complete sparse/all-mode layout, selected storage, three-method support refusal and actual backward fill controls passed; full-series qualification remains at 41f. |
| [F13](../design_review/reviews/design_review_simulation-pipeline-efficiency-and-effectiveness_2026-10-15.md#f13) | S07/S08 | resolved | 41d D2/D6 | **Tested — targeted correction accepted 2026-10-16:** [Outcome](41d-native-storage-and-workflow-worker-reuse.md#outcome--2026-10-16). Both shooting integrators and six-period Ipopt/POUNCE horizon controls passed with retained workers, fresh changed-IVPs and real owner teardown; full-series qualification remains at 41f. |
| [F14](../design_review/reviews/design_review_simulation-pipeline-efficiency-and-effectiveness_2026-10-15.md#f14) | S07 | resolved | 41e E1/E5 | **Tested — targeted correction accepted 2026-10-16:** [Outcome](41e-derivative-reuse-and-evaluator-capabilities.md#outcome--2026-10-16). Exact transient curvature/outer-weight reuse, changed-point, failure/retry, cancellation and original-rank controls passed; full-series qualification remains at 41f. |

F2–F5 supply assembled evidence where packages interact. Scheduled does not mean implemented, and a companion's completion alone cannot close a finding with unmet combined obligations.

## Opportunity routes and source reconciliation

| Opportunity | Owning package | Decision and completion boundary |
| --- | --- | --- |
| O01 same-point integration capability | 41e E2 | **Interface-checked supported no-adoption:** retain achieved-demand replay and completed products; [E2 decision](41e-derivative-reuse-and-evaluator-capabilities.md#investigation-decisions--2026-10-15) owns current-consumer and native-history limits. |
| O02 durable protocol grouping | 41c C4 | **Tested adoption:** bounded already-sealed file metadata grouping and exact table-completion ACK/schema controls passed; [41c Outcome](41c-studies-response-artifacts-and-result-resources.md#outcome--2026-10-15) owns limits. |
| O03 derivative actions | 41e E3 | **Tested adoption:** owner-local sample symbolic preparation with fresh numerical factors; existing directional/multi-RHS and full unsupported second-jet demands retained. [41e Outcome](41e-derivative-reuse-and-evaluator-capabilities.md#outcome--2026-10-16). |
| O04 compiled/batched evaluation | 41e E4 / RC03 | **Tested explicit scalar adoption:** optional Value compilation under unchanged scientific/FP contract; interpreter default and no SIMD/JIT/batching adoption. [41e Outcome](41e-derivative-reuse-and-evaluator-capabilities.md#outcome--2026-10-16). |
| O05 affine route/native study lifetime | 41d D3 | **Tested adoption:** actual healthy independent study/common-run consumers use existing neutral owner pool. Retain existing affine routing with bounded source/control rationale. [41d Outcome](41d-native-storage-and-workflow-worker-reuse.md#outcome--2026-10-16). |
| O06 presolve provenance/primal-only | 41d D4 / RC04 | **Interface-checked supported no-adoption:** pinned fork lacks full tightening-reaction provenance and no distinct demanded primal-only contract is selected. Retain restrictions/prechecks. [D4 decision](41d-native-storage-and-workflow-worker-reuse.md#investigation-decisions--2026-10-15). |
| O07 idle-retention pressure | 41b B4 after 41c C3 | **Tested supported no-adoption:** original 256 MiB pool and two 64 MiB compiler leases passed the real 65-block read; no new idle eviction is justified. [41b Outcome](41b-cancellation-shared-work-and-admission.md#remaining-scope-outcome--2026-10-15) owns evidence and revisit limits. |
| O08 causal observations | 41f F0/F1 | **Tested adoption:** selected producer work counts and copied Rust/Python/CLI views, active-root partiality, reset/loss and fresh-process opt-out; actual managed interruption/read/cleanup controls passed. No extra scientific evaluations or universal attribution guarantee. 41f owns assembled evidence. |

An inquiry finishes with evidence, a decision, its rationale and the resulting implementation or explicit no-adoption boundary. A proposal to investigate again is not completion. A required finding remains open if its corrective contract is still unmet. New unsupported scientific guarantees require an explicit operator decision rather than a quiet scope reduction.

The principal review reconciles local inquiry IDs; retain that mapping rather than creating competing findings. Preparation P-F1/P-F2/P-F3 map to F01/F02/F03, P-F5 to F04, P-F4 to F05, and P-F6–P-F8 to F06–F08. Numerical N1 maps to F12, N2's worker portion to F13 and its optional window waves to D5; its preliminary wording about trials is constrained by the principal to independent anchored windows within one trial. N3 feeds O01; the fresh IA-11 source finding is the narrower mandatory F14, not a checkpoint requirement. IA-01 covers F01–F05, IA-02–IA-08 cover F06–F11/O02, IA-09/10/11 cover F12/13/14. The library matrix's unused candidates are addressed by O01/O03–O06; availability is not adoption. Workflow/resource inquiry sections map to C1–C4. AUD39-01→A1; AUD39-02→A2–A5; AUD39-03→B1–B3; AUD39-04→C1; AUD39-05→C4; AUD39-06→C2; AUD39-07→C3/B4.

Preserve contrary evidence and limits: the cold case never reached a numerical RunHandle; 234 sweep joins were not 1,000-point completion; managed execution passed sixteen outcomes then failed before persisted numerical checks; prior fitting/shooting controls were scoped successes. Whole-operation query counts do not isolate publication cost. Two compiler leases were real, safe refusals were not physical success or host OOM, and a report is not a native checkpoint. No speedup or global numerical optimum follows from this plan.

## Prerequisites and execution sequence

1. Record rule decisions and begin only authorized production scope. A0, B0, C0, D0, E0 and F0 can resolve their bounded foundations independently. Install G0 amendments before the particular changes that consume them, not as a blanket barrier to unrelated within-contract corrections.
2. Implement B1's cancellation and simultaneous study bookkeeping before long dependent simulations. B2/B3 establish shared-producer and workspace admission behavior before concurrent preparation claims. F0 supplies observation meanings; F1 producer slices integrate with the owning packages rather than waiting until all corrections are complete.
3. A1–A5 remove established preparation amplification. C1 follows RC01's installed boundary; C2 can use existing artifact promises; C3 follows its exact request/retention analysis. These independent responsibilities do not require a universal lifecycle rewrite.
4. D0's complete layout and method selection precede D1 accounting. D2 neutral-worker retention and E1 completed curvature reuse are independent of E2 checkpoint policy or D5 window parallelism. Rebinding cancellation/diagnostics and changed-IVP reset are prerequisites to claiming reuse.
5. C4/E2–E4/D3–D5 settle their own opportunity decisions, with the relevant factual diagnostic slices available. B4 follows implemented C3 and evidence of remaining pressure. Qualified optional adoption includes actual consumers, tests and deletion in its package; unselected mechanisms do not become hidden backlogs.
6. A6/B5/C5/D6/E5 complete their consumer and targeted closure obligations. G1 checks combined readiness; F2 selects assembled cases; F3/F4 qualify composed scientific and ownership behavior; F5 performs scope-end checks; F6/F7 transfer enduring meaning, then reconcile relevant older plans and lifecycle references after implementation.

Settled semantics are not implemented prerequisites. Logical independence does not grant simultaneous ownership of shared runtime/compiler/native files, declarations or generators. The coordinator owns shared decisions, manifests and integration; bounded executors own explicit surfaces and preserve concurrent edits. Worktrees are only for genuine concurrent editing, then integrated and removed. No new scheduler or capacity cap is required.

## Scenario coverage and verification

| Principal scenario | Required route |
| --- | --- |
| [S01](../design_review/reviews/design_review_simulation-pipeline-efficiency-and-effectiveness_2026-10-15.md#s01) edits and preparation | A1–A6: retained/fresh comparisons, import membership/order, mutable overlays, selected work and unrelated-context growth. |
| [S02](../design_review/reviews/design_review_simulation-pipeline-efficiency-and-effectiveness_2026-10-15.md#s02) cancellation and sharing | B1–B3/B5: parent/child/sibling, unequal waiter clocks in both orders, queued admission, last-waiter cancel and real drain. |
| [S03](../design_review/reviews/design_review_simulation-pipeline-efficiency-and-effectiveness_2026-10-15.md#s03) study source ownership | C1/C5: public Rust/Python local and durable, explicit/lazy, occurrence order and actual immutable source lifetime. |
| [S04](../design_review/reviews/design_review_simulation-pipeline-efficiency-and-effectiveness_2026-10-15.md#s04) durable publication and read | C3/C4: nonempty managed read, pages/file-open, exact metadata ownership, multiple IPC schemas and exact ACK inventory. |
| [S05](../design_review/reviews/design_review_simulation-pipeline-efficiency-and-effectiveness_2026-10-15.md#s05) response families | C2/C5: valid/withheld root/optimal/transient, partial/events and independently retained artifact/graph meaning. |
| [S06](../design_review/reviews/design_review_simulation-pipeline-efficiency-and-effectiveness_2026-10-15.md#s06) sparse/native methods | D0/D1/D6: diagonal/banded/dense, mode-union/transpose/second-order support, direct/Krylov/preconditioner storage. |
| [S07](../design_review/reviews/design_review_simulation-pipeline-efficiency-and-effectiveness_2026-10-15.md#s07) point and demand changes | E1–E3/D2: same/changed point, objective versus scientific weights, derivative upgrades, replays and fresh-IVP invalidation. |
| [S08](../design_review/reviews/design_review_simulation-pipeline-efficiency-and-effectiveness_2026-10-15.md#s08) windows and horizon | D2/D5: compatible construction reuse, causal starts/feedback, owner thread, failure/reset and optional admitted waves. |
| [S09](../design_review/reviews/design_review_simulation-pipeline-efficiency-and-effectiveness_2026-10-15.md#s09) failure and scientific policy | B/C/D/E targeted controls plus F3: nonconvergent recycle/IC, changed scaling/branch/profile and unsupported capability keep original specification and truthful outcomes. |
| [S10](../design_review/reviews/design_review_simulation-pipeline-efficiency-and-effectiveness_2026-10-15.md#s10) extension and local dependencies | Actual dependency-local tests and focused design readback: new unit/property extends authored declarations, adapter/transport replacement stays local, no canonical store for pure policy. |

**Proposed verification:** during implementation compile touched packages and run targeted controls, regenerating declarations when necessary. Pure ownership/selection/policy controls use no unrelated store/native service. Actual Arrow-consuming correctness closures explicitly activate force-validation. Retained/fresh comparison is supplemented by independently specified mathematical and ownership controls where both paths could share the defect. Original physical checks and exact ACK/drain protections survive.

After functional scope and selected decisions are complete, 41f selects assembled local/durable/managed/native/Python journeys and runs hygiene/manual checks once, repairing scoped failures. Use counters and complete operations to explain removed work, with no statistical benchmark or hard latency gate. One complete representative simulation is justified only by a remaining decision-relevant question; no automatic eleven-case replay or generalized unsupported solver campaign is added. Report commands, modes, scopes, zero-failure baseline, exclusions and evidence maturity. Documentation-only changes require no product requalification.

## Current checkpoint

All A–E delivery and bounded investigation choices are complete. G0/G1 and F0–F4
have their selected acceptance: current generated/native Python contracts, the original
sparse and reuse controls, assembled S01–S10 ordinary journeys, real managed continuation/
cancellation, a 65-block read with two compiler leases during occupied three-point native
work, primary death preserving incomplete science, and public Python result reopening.
Initial failures were corrected at their actual consumer/oracle boundaries; prior evidence
keeps its original scope. Owned workers/stores stopped through their registered routes.

All six original hygiene failures pass repaired reruns; full workspace Clippy passes
default, no-default and linked all-feature/all-target modes. Generated equivalence, family,
Rust docs and 25 governance tests passed. Feature checks completed 268/393 combinations
before ENOSPC at 269; collector and host release also could not write. The operator freed
space with cargo clean. Normal admission reaping settled allocation
`d2abc339fe874ed8b9b66b46bcfd8bf3`; only the 125 unfinished combinations are resuming after
matching the prior successful selection. Passing source evidence is retained; the source
is unchanged. F5 is in progress, then F6/F7 final Outcomes, links and retirement. ADR-0190/
0191 stay proposed; no whole-product, older campaign, speedup or environment-proof claim.

## Authoring verification

**Tested — documentation only, 2026-10-15, zero-failure target:** `scripts/pse-env --docs --resource-class light -- just docs` published 471 chapters with scoped search. It reported an existing unclosed `<state>` HTML-tag warning in the supporting library inquiry; that evidence document remains unchanged. A docs-environment read-only link/coverage check across the seven new plans found 66 valid local file/fragment targets, fourteen F identifiers, eight O identifiers, four RC identifiers, ten S identifiers, 42 Planned package rows and eight declared native bindings, with zero scoped errors.

**Tested — metadata check, zero-error target not met globally:** `scripts/pse-env --docs --resource-class light -- python3 -m scripts.document_lifecycle validate` reports the previously documented single error: `docs/plans/39b-deployment-assurance-and-preparation.md::execution-packets` does not select exactly one native table. All seven new plans and their eight bindings validate without an added error. The existing plan is not changed to manufacture a clean global result.

**Interface-checked:** an independent assembled readback checked principal/supporting obligations, pending rule decisions, ownership, prerequisites and preservation. Corrections incorporated distinguish D0 design from D2 implementation, reuse assembled acceptance rather than repeating it at F5, explicitly route unchanged-sample elimination and POUNCE multi-RHS eligibility through E3, and preserve frozen recycle-map starts/branches. No product/scientific acceptance or measured speedup follows.

**Tested — tooling, zero command-failure target:** the initial default-registry `just turn-end` could not start because the legacy resource registry requires explicit fenced migration recovery. No migration/cleanup was attempted. Selecting the already provisioned `/tmp/pse-friction-review-resources-uMrtOxGw` report registry, `PSE_TEST_RESOURCE_STATE=/tmp/pse-friction-review-resources-uMrtOxGw scripts/pse-env --resource-class light -- just ready` passed skill synchronization and doctor; the same selection with `just turn-end` passed ADR indexing and formatting. Its advisory Ruff step still reports pre-existing diagnostics in `python/pse/diagnostics.py`; a passing formatting bundle is not strict lint acceptance. No product check, test, simulation or service was run for this authoring scope. Navigation/current-search selection and native bindings register the new series without updating existing plan bodies/statuses.

## A/B execution documentation closeout — 2026-10-15

**Tested — documentation:** `scripts/pse-env --docs --resource-class light -- just docs`
(with the selected resource registry used by the Outcomes) published 471 chapters and
scoped search. The existing unclosed `<state>` warning in the supporting library inquiry
remains. Published A/B Outcome and verification fragment targets were inspected.

**Tested — metadata, zero-error target not met globally:** `scripts/pse-env --docs
--resource-class light -- python3 -m scripts.document_lifecycle validate` reports only the
existing Plan39b `execution-packets` selector error recorded at authoring; the new series
and its native scope/status bindings add no error. No old plan was changed to hide it.

**Tested — turn-end:** `PSE_TEST_RESOURCE_STATE=/tmp/pse-friction-review-resources-uMrtOxGw
scripts/pse-env --resource-class light -- just turn-end` passed ADR indexing and formatting,
exit zero. Its advisory safe-fix Ruff step still reports 42 diagnostics in the existing
`python/pse/diagnostics.py`; this is not strict lint acceptance. Full hygiene/manual product
qualification remains at 41f. No product check was rerun after documentation-only closeout.

All delegated work is integrated in the existing main checkout. Only that checkout is
registered; no worktree was created or remains to merge/delete for this delivery. Owned
build/test processes have ended, and no service or managed worker was started. Unrelated
concurrent work and all original review/older-plan bodies remain preserved.

## B/C execution closeout — 2026-10-15

**Tested — targeted scope:** 41b B4/B5 and 41c C0–C5 completed with their named controls and exclusions at the companion Outcomes. The final managed Rust continuation and Python repeated-occurrence/result journeys both passed; registered service stop completed, and no owned worker/service or exclusive allocation remains. Integrated worktrees were removed. `just turn-end` passed indexing/formatting; its final advisory Ruff output contains 42 existing diagnostics, so no strict hygiene or full-series qualification is claimed. The two older numerical failures and the limits of crashed-primary observation remain explicit at 41c. Full D/E delivery, 41f acceptance and subsequent older-plan reconciliation remain open.


## Outcome (recorded after implementation)

### What was built

**Implemented:** A–E deliver all fourteen corrective findings and settle all eight opportunity
routes. Indexed import access, borrowed immutable inference/function premises and selected
preparation remove established broad/repeated construction. Cancellation propagates downward,
shared callers keep independent original clocks, producer policy is finite and compiler
admission precedes active dispatch through actual join. Public Rust/Python studies own genuine
local source/physical context, with explicit exact-source recording for durable use. Response
artifacts use their existing family-qualified meanings. Result reads retain exact selected/page/
SDK ownership and bounded already-sealed metadata grouping validates complete ACK inventory.
Native layouts/accounting cover complete support and selected methods. Shooting/horizon and
independent-run owners reuse neutral workers while changed IVPs reset. Completed exact fit
curvature and sample symbolic preparation reuse valid products with fresh numerical factors.
Optional scalar Value compilation is explicit; the interpreter remains the default.

**Implemented:** F0/F1 extend existing diagnostics with actual causal work, retained active
roots, explicit missing-parent/reset/loss meaning and copied readers. The exact admitted primary
association distinguishes operational unavailability from durable scientific completion.
Pending original correction preserves declared recovery and the original budget instead of
fabricating a capability veto. Replaced broad constructions/duplicate paths and migrated
historical wire consumers are removed; no compatibility execution path is introduced.

**Tested/Measured — work counts only, 2026-10-16, zero-failure baseline:**
[41f's Outcome](41f-causal-diagnostics-qualification-and-closeout.md#outcome-recorded-after-implementation)
records actual commands, initial failures, successful affected reruns, independent science,
interruption and enclosing cleanup. Its S01–S10 selection covers the changed local/native/
managed/Python compositions. All 148 selected ordinary obligations, four managed Rust controls
and selected public Python/diagnostic obligations are covered after repairs; these are composite
results, not a fabricated fresh all-green run or a sum of overlapping tests. Earlier package
Outcomes retain their exact scope. The 65-block managed read keeps the original observer pool
and two genuine compiler leases during occupied three-point native work, then verifies distinct
retained numerical results. Actual primary death leaves durable science incomplete. Enabled/
disabled diagnostics preserve scientific call counts and outcomes. No numerical superiority,
elapsed speedup or whole-product scientific qualification follows.

**Interface-checked supported no-adoption:** achieved-demand replay rather than a new eager
checkpoint frontier; existing affine routing rather than another backend; current full-reaction
restrictions rather than primal-only presolve/FBBT activation; serial windows/trials rather than
window waves; no idle-pressure eviction after correcting required metadata ownership. O03/O05
adopt owner-local symbolic/native worker reuse, O02 bounded metadata grouping and O04 optional
scalar compilation. Unselected candidates are settled outcomes, not deferred implementation.
RC01–RC04 routes are applied within their accepted bounds; ADR-0190/0191 remain proposed under
the existing decision-PR rule. Enduring meanings belong to blueprint §14.3/§14.5, §19.2–§21.5,
§23.1/§24.2/§25 and the simulation diagnostics guide.

### A mistake made and corrected

Public consumer tests found a separately maintained native request-header version and policy
constructors missed by declaration migration. They now consume the required current selector
and v7 request; scientific replay/lazy variants v2 and study definition v11 are regenerated
from their owners. The edit/root assertion originally imposed an unrelated tolerance; the
corrected independent oracle uses its frozen original row budget and permits either valid
root. The managed read initially requested global release while intentionally live study
readers still existed; it now checks this read's exact reservation recovery after SDK drain.
A loss fixture now creates actual detail before asserting dropped detail. These repairs and
scope-end Clippy/Python corrections retain production scientific and ownership contracts.

### Deviations from the plan, deliberate

Small decisive managed death/read cases supplement the original sixteen-lane cancellation
control; the test-only fixture guard admits one through the unchanged profile's lane bound.
No capacity bump, scheduling policy, extra scientific diagnostic evaluation or profiler rerun
was needed. Package and assembled checks are not the old Plan39 eleven-case campaign or full
Plan40e acceptance. Their remaining work stays independently owned. Statistical benchmarks,
hard timing criteria and eliminated environment/provenance proof have no deferred obligation.
Unexercised models, arbitrary solver combinations, release/distribution and other platforms
are outside this affected-scope evidence. Scope-end checks/document retirement remain in
progress until the final closeout record below is filled.
