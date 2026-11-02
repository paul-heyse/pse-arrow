---
title: "Plan64h: Reference coverage and qualification"
status: draft
date: 2026-11-02
doc_role: plan
doc_owner: docs/plans/64-process-simulator-target.md
doc_topics: [current-work, design, validation]
review_sources: [docs/design_review/reviews/design_review_process_simulator_target_2026-11-02.md]
adrs: []
---

# Reference coverage and qualification

## Purpose and claim boundary

**Proposed:** establish P0's finite required reference population, Packet D's complete matched comparisons and P7's assembled acceptance. The [coordinator](64-process-simulator-target.md) owns combined scope, rule decisions and finding disposition; this companion owns H01–H07 and actual qualification accounting. Authoring baseline is `1acd8c83ca8f9e6f6e9b5e9c9cbf98e299d574b8`. Core 3.4, efficiency heuristics 1.0, ProcessSimulator 1.5 and test-assessment 1.2 apply.

RC01 was accepted on 2026-11-02: the target is expanded released process-modeling behavior under finite configuration and case mappings. It is not a promise to reproduce Python APIs, class hierarchies, solver iteration sequences or every companion deployment mechanism. Keep supplemental whole-distribution applications such as MatOpt separately identified; a process-only claim cannot silently become whole-release equivalence. Native MHE and broader ecosystem cases have their own extension evidence rather than fabricated IDAES 2.13 provenance.

Use the [scope inventory](../process-simulator-design/delivery/idaes-scope-and-capability-coverage.md), [notebook register](../process-simulator-design/reference/notebook-families.md), [twelve domain witnesses](../process-simulator-design/reference/acceptance-workflows.md) and [qualification contract](../process-simulator-design/delivery/equivalence-and-qualification.md). Their counts describe different populations. A witness tests a mechanism; a notebook family organizes source; neither is the denominator for a released configuration/behavior claim.

## Foundation assessment and reference custody

**Interface-checked:** native candidate exchanges and isolated reference drivers exist under `tests/reference/oracles`; authored conformance lives in `workflow/modeling/conformance.rs`. They are suitable owners for scoped scientific comparison, not proof that this expanded population has executed. The reviewed scientific/runtime source is unchanged by review publication. Historical receipts retain their original source, tool, support and evidence limits.

The package records core IDAES 2.13.0 at `84532d7f99a9f976b05170948147afdd69cfa5f6`, modern examples at `eb64e38ac429b7e3ae4f8db071f8d8cad2eef675`, and compatibility material at `8f1ae7a3ca358625128a78722bc2fdb75753a441`. H01 verifies the exact selected artifacts and reference environment before admitting them as executable references. The examples package's own version/dependency declaration does not prove compatibility with 2.13.0. Do not silently move core, examples or data revisions to obtain a passing run.

Recursively resolve helper modules, coefficient files, datasets, trained weights, serialized initialization and external prerequisites. Distinguish scientific formulation authority, parameter source, matched reference and independent validation. Preserve ordinary versus fixture/oracle eligibility through candidate production and readback; reference permissions cannot turn oracle-only inputs into ordinary production content.

Reuse existing case/source declarations, conformance/reference exchange and retained outcomes for exact correspondence and evidence. Extend missing fields at those owners only when a concrete case cannot be represented. Do not create a second scientific-result database, permanent receipt ledger or generic qualification framework.

## Package state

| ID | Status | Prerequisites | Completion |
| --- | --- | --- | --- |
| H01 | planned | Accepted RC01 scope and current case/register sources | Exact reference repositories/environment and finite required population are established, with every source category explicitly dispositioned |
| H02 | planned | H01 first-case slice; ordinary-data and formulation selection from 64b/64c | Matched recycle/exchanger reference contracts and independent candidate/comparison controls are ready before integrated execution |
| H03 | planned | H02; complete native A–C journeys and consumed F01/F02/F03 contracts | Packet D establishes complete outputs, intended failure correspondence, original physical checks and exact retained readback |
| H04 | planned | H01 population; selected 64d–64g scientific capabilities | Configuration, property, unit, workflow and extension acceptance closes each delivered family at its actual scope |
| H05 | planned | Selected H03/H04 outcomes; original execution and persistence owners | Relevant assembled native/operational journeys, interruption and deployment support claims are qualified without duplicating other plans' status |
| H06 | planned | Selected quality-equivalent cases and a concrete benefit/capacity question | Representative cold/reuse/structural/study/trajectory measurements answer the claim with original failures and conditions retained |
| H07 | planned | All required population rows and consumed owner obligations | Scope-end checks, final claim/evidence reconciliation and enduring publication complete with zero unresolved required variants |

## H01 — Freeze the denominator and resolve prerequisites

Classify the released core families, supported categorical configurations, shipped source examples/tests, modern documented notebooks, held/archive cases, developer/dummy material and supplemental applications. Register the scientific model once and its genuinely distinct configurations/study variants separately; duplicate wrappers do not increase coverage. Record required, deliberately excluded-with-reason and extension scope at the coordinator's accepted boundary. An unavailable dependency is a blocker for a required case, not permission to remove it from the denominator.

For each selected family enumerate relevant dynamic/steady/storage choices, material/energy/momentum balances, ports/directions, property/reaction compatibility, phase/state/flow basis, thermal/pressure assumptions, numerical discretization and design/operating specifications. Record forbidden combinations as refusal obligations. Continuous domains need stated envelopes and motivated boundary/interior checks; finite samples do not prove arbitrary-domain correctness.

The investigation output is the exact source/configuration/case inventory, reference-environment compatibility decisions, input closure and required outputs/evidence for each mandatory row. It supplies concrete downstream choices to 64b/64d–64g. Unknown parameterized variants or missing helper/data artifacts block that row's execution readiness; they do not block unrelated inventoried cases. Keep the existing held/archive disposition history and explain any change through the coordinator.

## H02/H03 — Packet D matched independent references

Fix the ideal benzene/toluene recycle and BT_PR/relational-density exchanger scientific contracts independently. Retain the exchanger's unknown-area/pressure-extrapolation permissions and scoped costing meaning. Rating and design use explicit corresponding fixed/free roles; substitution of another property model is improved/different science, not matched-physics parity.

For every mandatory first-case variant state exact native/reference declarations/data, component/phase identities, bases/datums, conditions, initialization/branch intent, requested outputs and comparison allowances before relying on results. Reconstruct the intended failure far enough to reach its initialization stage; setup failure cannot satisfy it. Native/reference algorithms can differ while retaining the same scientific problem and output correspondence.

Candidate exchange preserves Rust-produced values and exact source/operation identity. The isolated reference environment independently constructs and solves the matched problem; it must not calculate expected values through native production transformations. Compare every mapped required output and completeness, not one sentinel scalar. Keep independently derived analytic and original material/energy checks alongside differential comparison: two implementations can share a bad datum or coefficient.

Use observable-specific dimensional absolute/relative tolerances and near-zero rules. Compare original physical feasibility separately from scaled solver tolerance and native termination. Exercise a deliberately wrong output, an omitted required output, species/field reordering and an incompatible datum or formulation. The comparison must fail for the intended reason. Missing solver, wrong IDAES version or missing required reference data fails a selected run; unattempted scope remains unattempted.

D can develop once input science and mapping settle; its final acceptance consumes working native journeys. It need not wait for every later P2–P6 family. H03 supplies first-delivery evidence and leaves the broader population visibly open.

## H04 — Close configurations and scientific workflows

Grow acceptance from property/reaction values and derivative chains to configured units, coupled flowsheets, engineering studies and custom extension workflows. Include original balances, envelopes, conservation, phase/root behavior, initialization, coverage and requested derivative/dual/statistical meaning. Stable equilibrium, local optimum and globally certified results have different evidence requirements.

For dynamics agree on physical input profiles, initial conditions, events/resets, requested interval, output grid and reconstruction. Compare event timing separately from trajectory values; do not interpolate across missing segments or unresolved discontinuities. Independently account for inventory change from integrated original flux and event transfers. Use spatial/time refinement where the numerical approximation claim requires it. A reference trajectory is not an exact continuum solution.

For inference/control/surrogate/grid cases consume 64g's precise statistical, physical-time, artifact and discrete-capability contracts. Validate known confounding, failed samples, withheld products and constrained/infeasible cases as applicable. Custom-property, custom-reaction and custom-unit authoring remain required native user workflows; successful prebuilt models alone do not establish extensibility.

## H05/H06 — Assembled operation and measured benefit

Plans28/32/40/53/54/58 and 58h retain their independent status, packages and evidence. Identify the exact prerequisite slice consumed: protected publication/readback and durable liveness, lifecycle ownership, scientific diagnostics, independent assurance/fixtures, build environment, and native cutover/deployment qualification. Link the original owner and applicability evidence rather than copying its current status. A passing Plan64 case cannot close unrelated remaining obligations.

Exercise selected end-to-end author/edit/prepare/execute/inspect/readback journeys with original scientific assessments. Where the claim includes recorded execution, qualify interruption, lost acknowledgement, cancellation, retained-result scope and actual drain through existing owners. DA-062's interval semantics and DA-063's unrelated capture/liveness concern retain their original disposition; expose applicable correction or qualification as a prerequisite, without blanket bound relaxation or weakened capture completeness.

Before persistent/shared-deployment claims, consume supported historical migration, backup/restore and recovery evidence from the existing deployment owner. Do not infer those guarantees from regenerable development stores. Local scientific acceptance and deployment acceptance remain separate claims.

H06 begins only for an identified benefit or capacity decision. Separate first preparation, value-only reuse, structural edits, recycle robustness, sparse coupled models, case populations and trajectories. Record hardware, placement/thread policy, scientific inputs, tolerances, initialization, exact native/reference versions and workload size. Compare quality-equivalent results, completion/failure populations and timing distributions. Structural work removed is not itself measured speedup; do not introduce a universal threshold or cost-accounting system.

## Verification and final completion

**Proposed verification:** implementation packets use targeted compile and owner-local tests. Integrated scientific/reference/operational checks run after the corresponding functional scope is assembled, not after every edit. H07 selects scope-end `just hygiene`, governance/docs, relevant native/powerset/manual checks and performance campaigns from actual changes, with original commands, modes, zero baseline, results and exclusions. No command is a substitute for its scientific acceptance assertions.

Final completion requires every mandatory capability/configuration/case row to have accepted evidence, no unresolved required variant, zero failures in the selected acceptance population, complete required output coverage and public scope matching actual support. Preserve unavailable, failed and unattempted outcomes; do not shrink scope to improve a percentage. Publish enduring contracts through their established architecture/decision owners and retire replaced paths and completed planning material under the repository lifecycle policy.

**Outcome:** Proposed only. No builds, tests, solver/reference runs or measurements were performed in authoring this plan. H01's finite population and first-case input/reference investigation are the next ready work once the coordinator's scope/decision prerequisites are available; H03–H07 remain unqualified obligations.
