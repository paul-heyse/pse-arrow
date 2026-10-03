---
title: "25k: Integrated qualification and closure"
status: in-progress
date: 2026-09-30
adrs: []
review_sources: [docs/design_review/reviews/design_review_codebase-domain-alignment_2026-09-30.md, docs/design_review/reviews/design_review_codebase-domain-alignment-follow-up_2026-09-30.md]
scenario_sources: [docs/design_review/reviews/design_review_codebase-domain-alignment_2026-09-30.md#s01, docs/design_review/reviews/design_review_codebase-domain-alignment-follow-up_2026-09-30.md#fs11]
---

# 25k: Integrated qualification and closure

## Purpose and ownership

This is the **only full qualification stage** of the [Plan 25 series](25-design-remediation.md).
Plans 25a–25j implement the target directly, use compilation and focused behavioral checks,
and delete replaced mechanisms as their callers move. They do not repeat this campaign at
section or document boundaries. This plan starts after all their functional packets are complete.
The later [25l functional follow-up](25l-flowsheet-compilation-and-solver-routing.md) supplies
the corrections required at the current review boundary before K3 resumes.

The coordinator owns finding dispositions. This document owns the assembled system's eventual
qualification evidence. A passed packet, an accepted ADR and a historical Plan 23 result are
different evidence from a qualified Plan 25 implementation. The execution checkpoints below
record current readiness; the Outcome will record the completed campaign and its limits.

## Decisions

- Qualify behavior, architectural fitness and performance separately. A numerical pass does
  not establish change locality; a design review does not establish numerical accuracy.
- Use existing recipe-owned environments and independent reference data. Keep explicit Arrow
  force-validation for correctness runs. Native campaigns use the repository's memory-capped
  recipes; preserve the limit and record thread, timeout and feature conditions.
- Test the final target only. Do not retain deleted execution paths as comparison oracles.
  Preserve source/reference observations with independent scientific authority, regenerating
  only when their actual inputs or contracts change.
- Performance acceptance includes reuse/allocation counts and resource bounds. Report timings
  as measurements under named conditions, not invented speedup thresholds. Historical results
  are contextual unless a reproducible comparable baseline is available.
- Keep one qualification result per selected scope. Repair failures and rerun the failed scope;
  repeat other scopes only when the repair could invalidate their results. Report such a result
  as a composite campaign, with the zero-failure target, rather than as an initially clean run.

## Integrated journeys

These journeys combine the existing review scenarios; they are not a new parallel scenario
registry. Each needs positive controls and the indicated refusals, with physical tolerances
and fixture inputs authored alongside the implementation that supplies the behavior.

| Journey | Consumed plans | Required evidence |
|---|---|---|
| Typed scientific extension, S01/S09 and FS01–FS03 | 25a/25b/25h/25j | Lawful reduced-coordinate model and typed overlay round-trip; wrong coordinate/basis/datum refused; unknown composition cannot prove conservation; complete empty composition remains distinguishable; mixed pair selections respect fit-group constraints |
| Connected steady and dynamic flowsheets, S02 and FS04/FS05 | 25a/25b/25c/25e | Indexed mixer → holdup reactor → separator; executable recycle with conditional unit solves; independent boundary and temporal closure; required initial conditions; undeclared/wrong event transfer refused |
| Implicit substitution and square sensitivity, S03/S04 and FS06/FS07 | 25d/25e | Two-root and restricted-root cases, honest exact/relaxed labels, C1 first-order versus C2 second-order demand, rank/branch failure, qualified root response without a dummy objective |
| Event endpoints, shooting and incumbents, S04 and FS05 | 25c/25e/25f | Event-defined success evaluates its actual endpoint obligations; fixed-horizon coverage remains incomplete after an early event; shooting consumes the declared route and common qualification; accepted limit incumbent remains explicitly non-optimal |
| Repeated and interrupted durable experiments, S04/S06/S10 and FS09 | 25e/25f/25g/25i | Same binding in distinct occurrences, return-path continuation, compatible seed/fallback decisions, cancellation/retry and lease recovery, per-occurrence idempotency and typed failures without a fabricated result |
| Durable evolution, S05/S11 and FS10 | 25f/25g/25j | Old artifact read with its recorded domain; directional consumer/writer refusal; read-only open; explicit migration and lineage; additive vocabulary change preserves catalog references; bounded orphan reconciliation |
| Edit/rebind/reuse, S07/S09 and FS08/FS11 | 25a/25h/25i/25j | Clean/incremental agreement, fresh diagnostics over shared mathematics, old result attribution retained, A/B/A binding reuse, referenced-context invalidation and repeated durable-worker preparation reuse |
| Resource and failure lifecycle, S06/S10 and FS11 | 25f/25i | Abandoned staged work retains admission until native completion; failed dispatch releases it; escaped products retain reservations through eviction; worker panic is visible and never silently rerun |

## Execution packets

| Packet | Prerequisite | Responsibility and acceptance | Status |
|---|---|---|---|
| <a id="k1"></a>K1 Functional readiness | All functional packets in 25a–25j | Confirm target consumers are migrated, replacement/deletion obligations are complete, required decision routes are satisfied and fixtures exist for the journeys above. Resolve remaining functional work in its owning plan before starting K2 | done |
| <a id="k2"></a>K2 Format and static qualification | K1 | Run the single series-wide formatting/lint pass and relevant Rust/Python compilation, generation, family, governance and documentation checks; repair to zero | done |
| <a id="k3"></a>K3 Behavioral and scientific qualification | K2; at the current review boundary, 25l L6 and refreshed static evidence for its changed source | Run the selected Rust, native, Python, conformance and compatibility journeys; independently assess physical closure, domains, original-space residuals, outcome truth and durable lifecycle | in progress |
| <a id="k4"></a>K4 Reuse and performance measurement | K3 | Measure cold/warm preparation, one-body edits, in-process/durable value studies, retention after eviction and worker admission; report counts, timing distributions, memory and all refusals/failures | planned |
| <a id="k5"></a>K5 Architectural assessment and closure | K3/K4 | Conduct one bounded target-design review of the assembled change, reconcile every finding with its evidence, update enduring owners and close only demonstrated scope | planned |

### Review boundary — full case compilation and solver execution

Execution checkpoint, 2026-10-02: the maintainer paused further qualification to assess
full flowsheet/case compilation, capability-driven mathematical execution and the design
issues exposed by integrated failures. The
[target review](../design_review/reviews/design_review_flowsheet-compilation-and-solver-routing_2026-10-02.md)
records the current implementation's Revise judgment and proposed direction. It identifies
consumer demand arriving after executable/support admission, incomplete composition of
preparation with contextual solver eligibility, and misleading public interpretation of
unavailable feasibility evidence/resource failure. It also preserves the existing physical,
structural, implicit-meaning, identity and lifecycle contracts.

K3 remains incomplete; K4 and closure remain pending. This requested review is broader in
its compilation/routing inquiry than the planned final K5 evidence assessment and does not
complete that packet. Its findings are now scheduled in the
[coordinator's existing disposition owner](25-design-remediation.md#full-case-compilation-and-solver-routing-follow-up)
through the [25l implementation plan](25l-flowsheet-compilation-and-solver-routing.md).
**Implemented/Tested, 2026-10-02:** [25l L0–L6 functional scope is complete](25l-flowsheet-compilation-and-solver-routing.md#outcome-recorded-after-implementation),
with selective support, contextual readiness, completion and generated/recorded boundaries,
124 focused Rust tests, five Python codec tests and final default/linked compilation.
Its four runtime preparation controls start no native attempt and use no storage.
Static refresh checkpoint, 2026-10-02: the requested `just hygiene` refresh is complete
after source repairs and regeneration of the route-decision invariant fixtures. Default,
no-default and linked native Clippy all pass with `--keep-going` and `-D warnings`;
`just lint-solver-contracts` covers the linked graph outside hygiene, and
`just fmt-rust-check` passes. The final checks meet the zero-failure target after iterative
repair. Targeted diagnostics, math assembly, native preparation and contextual routing
controls pass with explicit force-validation and one test thread; the five Python codec
controls also pass. The repairs preserve shared immutable facts, inline admitted metadata,
retained routing evidence and the distinct exact-Hessian inputs.

The next work here is the affected K3 scientific and integrated journeys, K4 measurements
and K5 assessment. Earlier K1/K2 results retain their historical conditions; this static
refresh and focused readiness do not qualify whole-case solutions or performance.
The selected dev-profile measurement choice remains in force.

The resumed campaign includes 25l's demand separation, pending class/capability evidence,
contextual method/build/representation controls and original failure-meaning matrix alongside
these existing journeys. K4 extends its current workload owners to measure actual selected
output and Value/First/Second support construction, reuse and resource lifetimes. This remains
one composite qualification campaign; 25l adds no second full gate.

### K1 — Readiness without a new governance framework

Execution checkpoint, 2026-10-02: the completed 25a–25j working tree is the preserved
baseline. Existing controls cover much of the required storage, response and lifetime
matrix. Readiness work adds actual connected process/recycle and mixed durable journeys,
and prepares measurement workloads before the comprehensive functional receipt. K4 uses
the existing dev profile, as selected by the maintainer. Native campaigns remain capped
at 120 GiB, with bounded test scheduling and one BLAS/OpenMP thread. Heavy campaigns and
timing runs are serialized. Proposed ADRs retain their decision-PR status; no commit,
push or publication is part of this execution.

Readiness is complete. Actual indexed conditional recycle and dynamic inventory controls,
mixed durable recovery/continuation/cancellation, and nonoptimal incumbent acceptance are
registered in their existing harnesses. The mixed durable journey corrected nullable retry
classification and premature refusal before explicitly permitted fallback acquisition in
their existing store/policy owners. The five K4 workloads now have untimed controls through
`just bench-case-smoke`; this mode creates no measurement receipt and does not relax
`just case-measure`'s comprehensive functional prerequisite. Measurement instrumentation
counts actual body admissions, value rebinds and Delta write attempts. The next work is the
format/static pass, then the comprehensive functional receipt and selected dev measurements.

Use the existing plan ledger, source inspection and packet evidence. Do not create a source seal,
mandatory symbol manifest or automated architecture score. An intentional retained component
must still have a current target responsibility; deleted-path tests are not historical artifacts
to preserve. Legitimate independent oracles and native-library adapters remain.

### K2/K3 — Recipe selection and reporting

Execution checkpoint, 2026-10-02: static repairs are complete. Registry-owned enum defaults,
generated tuple-key projection, explicit store fences and receipt groups, current Python/native
consumers and invariant fixtures now agree with the target. The feature recipes retain workspace
unification and the complete declared combinations while supplying native math paths and qualifying
member feature names. Native helper compile conditions follow their actual consumers.

The imported future-compatibility repair uses the published validator/derive releases through the
existing generated Delta override, with one root version pin and the workspace lock as build
authority. A provisional Cargo patch was rejected by governance and removed; no governance rule
or warning was suppressed. Imported metadata boundary controls preserve schema, length and Unicode
semantics. The remaining work is the full behavioral/scientific campaign, then dev measurements
and the bounded architectural verdict. Source inputs stay unchanged from the comprehensive receipt
through measurement; closure documentation follows afterward. Parity runs before the ordinary
linked extension is refreshed for that receipt.

The first complete assessment exposed scientific lowering, initialization ownership and
recorded-contract defects, as well as fixtures still expecting replaced contracts. Repairs
retain authored temporal occurrence positions and resolved axis ownership, actual dependency
closure through validity/evidence, datum-point subtraction before reference translation,
and the admitted physical types of static conditional branches. Block binding now retains
existing shared parents while charging its new products. Recorded contracts select exact
versions and provenance; fault-proxy shutdown awaits owned relays and backend completion.
The local operational store has completed its preserving V1–V7 transition without reset.
Targeted checks precede the repaired integrated campaign. The default Rust and doctest scopes
will inherit the existing native execution environment explicitly, including the Symbolica
licence, rather than relying on an interactive shell's direnv activation. K4 and the final
review remain pending successful qualification; the campaign will be reported as composite.

The full authored seed closure exposed additional campaign consumers beyond the earlier
focused roots. Their migration preserves nominal reduced quantities, directed transfer
owners, explicit indexed arguments and the original independent oracles. Concrete analyses
must declare narrow permission for selected records with unknown applicability; the evidence
remains unknown and extrapolation remains forbidden. Qualification also checks fixture
binding against the actual refined physical target and bounds actual sparse support
construction rather than refusing an otherwise lawful body solely for its support width.
Repeated PFR instances also exposed occurrence-specific validity targets in mathematical
identity. The ADR-0150 correction separates body-local checked members from each instance's
total attribution binding, retains complete ranges and physical context, and versions the
changed current frames without altering historical bytes. Typed diagnostic composition must
apply that binding before wrappers contribute independent source identities.

Positive Peng–Robinson flowsheets now explicitly select the original density relation in the
outer problem. ADR-0144 distinguishes this authored relational formulation from the retained
operational selector; an unproved nonlinear selector remains value-only and its derivative
refusal retains a typed capability cause. Intensive fixtures declare finite construction/support
allowances rather than changing global defaults. Original independent expectations remain.
The complete seed campaign and assembled receipt must pass before measurement begins.

Use the current `just --list` contract at execution time. The initial selection is `just fmt`,
`just ci-fast`, `just quality`, `just governance`, `just adr-lint`, and `just docs`, with
`just docs-test` for any publisher/citation changes. Avoid repeating aggregate checks already
covered by a successful enclosing recipe without a reason.

Refresh the linked extension through `just py-sync-native`; exercise `just native-test` and
`just native-python <output>` for the selected native and Python scopes, `just seed-conformance`
for the full authored seed/domain campaign, and `just parity` for the affected IDAES compatibility
journeys. Use filters to select actual tests, and record those filters and omitted scopes. Missing
required native or parity prerequisites are failures, not skips or passing exclusions.

The final campaign must include real storage and solver composition, in addition to the isolated
policy checks used during implementation. New cross-plan fixtures should be registered in the
existing harnesses by their functional owners. Update a recipe if the target changes its required
feature/environment selection; do not work around it with an unrecorded long command line.

The [25g handoff](25g-durable-contract-evolution.md#verification) supplies authored, unexecuted
storage controls in `pse-catalog/tests/native_artifact_migration.rs` and the operations migration/
retirement harnesses. K3 must exercise exact frozen-25f preservation, interrupted committed prefixes,
malformed history, reset rollback/lost acknowledgement, prior unresolved inventory, export expiry,
foreign overlapping intent protection and queued lease renewal, plus native migration lineage,
changed-map recovery refusal and actual restart/discovery/reclaim. The existing `just native-test`
workspace harness discovers these targets; `just db-test` selects the isolated operations journeys.
Do not count their earlier all-target compilation as execution.

The integrated 25h/25i/25j handoff adds the frozen `test-fixtures/plan25g/` operational
source and the appended V7 operational identity/frame transition. Focused preserving migration
controls have run against isolated PostgreSQL; K3 still owns actual restart, mixed-operation
and recovery composition. Historical digests retain their bytes and nullable frame provenance;
an unknown historical frame cannot become a current operational cache key.

Generated native-boundary fixtures under
`python/pse/tests/fixtures/generated-native-boundaries/` cover partial/cancelled outcomes,
pre-result refusal, failed attempts, incumbents and event-ended trajectories. Their isolated
Python decoding checks establish transport distinctions, not solver or storage outcomes. K3
must exercise those distinctions through the actual assembled workflows.

The 25i handoff supplies complete-context body/package reuse, nested escaped-allocation owners
and injected profile dispatch/panic controls. K4 must measure these mechanisms under actual
in-process and durable workloads; focused reuse counts are not timing or RSS evidence. K2
also owns the recorded imported `proc-macro-error2` future-compatibility advisory alongside
the zero source-warning target.

Each executed result names command, mode/features, fixture scope, conditions, failure count
against zero, and remaining exclusions. K2/K3 commands are a proposed selection, not claims
that any current command has run or passed.

### K4 — Measurements that distinguish the design

Count actual body admissions and prepared products: an unchanged body with a retained warm entry
and unchanged complete admission context must reuse its admitted product; changing one equation
invalidates its dependency closure; a span-only edit changes attribution without unnecessary
arithmetic rebuilding. Measure resident reuse separately from lawful recomputation after eviction
or denied retention. Compare an N-point durable study on one worker with equivalent in-process
execution, separating structural preparation from binding and
attempt-owned solver work. Count repeated occurrences as separate experiments even when they
share preparation.

Use the existing cache/case measurement recipes and add the necessary workload selectors to
their owners. Account for unique live allocations and escaped owners, not merely cache size or
RSS. A deterministic gated worker establishes permit lifetime; timing alone is insufficient.
Record pool limits, concurrency, cache state, source revision and force-validation mode. Performance
mode must not be substituted for correctness qualification.

### K5 — Final review and retention

Apply the selected core and process-simulator design standard to the changed boundaries. Check
ordinary scientific extension, mechanism substitution and isolated policy testing, alongside
physical/numerical gates. Judge the new target itself; neither historical review verdict proves it.

The coordinator changes a finding to resolved only when all its child obligations and required
evidence are complete. Preserve any actual residual gap explicitly rather than averaging it away.
Move enduring contracts and rationale into their architecture/ADR owners through the existing
routes; then retire completed plans and resolved reviews only when their readers are redirected.
No commit, push or publication is implied by this campaign description.

## Outcome (recorded after implementation)

### What was built

Not executed. Record actual qualification scope and evidence labels at closure.

### A mistake made and corrected

Record an actual correction from execution; do not invent one during planning.

### Deviations from the plan, deliberate

None recorded. Explain any later scope or decision change with its owning route.
