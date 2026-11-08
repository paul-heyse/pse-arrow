---
title: "Plan70: Test assurance architecture"
status: in-progress
date: 2026-11-08
adrs: []
doc_role: plan
doc_owner: docs/plans/70-test-assurance-architecture.md
doc_topics: [current-work, design, validation, preparation]
review_sources:
  - docs/design_review/reviews/design_review_test-assurance-target_2026-11-08.md
  - docs/plan_improvement_reviews/plan70-repository-test-efficiency-2026-11-08.md
scenario_sources: [docs/plans/70-test-assurance-architecture.md]
---

# Test assurance architecture

## Target and baseline

**Proposed.** Realize the November review's owner-local test architecture with explicit
composition at the boundaries that carry selection, physical effects and evidence. Cargo owns
compiler eligibility and its unit graph. Nextest owns effective case selection and per-run
dispatch. Fixture and resource owners admit and settle their effects. Cases and independent
oracles own scientific expectations. The original workflow owner reports selection, verdict,
process exit and settlement separately. Focused feedback should demand only the work and state
the claim requires, while preserving the full guarantees of real integrations and required science.

**Interface-checked at `622261d8f5cab55159e7543b7df1762f78ca22f4`.** The
[design review](../design_review/reviews/design_review_test-assurance-target_2026-11-08.md)
published at `efc1a7cb4`, including its scoped audit reconciliation, is the design input.
The current baseline includes the accepted SurrealDB 4.0.0 fresh-default cutover and
Plan69c's guarded analyzer candidate and scoped binary-membership and image-leasing evidence.
[E01](70e-preparation-and-verification-scope.md#e01) receives the analyzer result as advisory input
and retains its scientific fixture associations and conservative widening. Core 3.4, heuristics
1.0, test-assessment 1.2 and the process-simulator profile 1.5 guide the target.
No new architecture rule change is proposed. [ADR-0257](../adr/0257-rebuild-canonical-storage-on-surrealdb-v4.md) ·
[Plan68 selection evidence](68e-surrealdb-v4-fresh-generation-and-acceptance.md#e03--select-open-writes-and-release)

The foundation assessment uses Plan69's delivered ordinary graph admission, pinned managed
nextest capture, native result pairing and release. Plan69c now supplies binary membership,
helper selection and image leases with scoped receiving evidence. [E02](70e-preparation-and-verification-scope.md#e02)
and [E03](70e-preparation-and-verification-scope.md#e03) use that foundation to narrow downstream
verification. Plan68's v4 SurrealKit receiving and Plan67's native borrower evidence support using
existing private service owners. Representative production-executor cleanup remains B03 work.
The current process supervisor
authenticates group drain but its optional stderr reader is Park-only. B01 must qualify bounded
two-stream capture before it becomes a common consumer contract. The ordinary Proto exchange's
independent physical values and bounded wire model are reusable. Legacy candidate allowances
and current-generated predecessor bytes require independent witnesses. [Plan69c capture](69c-ordinary-testing-and-managed-results.md#analyzer-and-capture-boundaries) ·
[Plan68d receiving](68d-surrealdb-v4-developer-and-test-receiving.md) · [Plan67c](67c-surrealkit-and-rust-test-consolidation.md) ·
[Plan66l](66l-required-population-and-assembled-acceptance.md)

Choose small changes at those owners over a new scheduler, generic fixture framework, oracle
wire, or custody ledger. Some end-to-end cases require real provider, process or physical-store
work. Selective reuse can remove hashing, capture and preparation only within the original
validity and trust scope. An independently receiving owner authenticates what it consumes.
The planned benefits are structural. Quantified savings require measurement on a named complete
workload.

**Interface-checked at `dca0dcf8f`.** The [repository-wide efficiency review](../plan_improvement_reviews/plan70-repository-test-efficiency-2026-11-08.md),
published at `55664cd05`, extends the planned work across shared compiler admission,
test inputs, conformance, benchmarks and developer verification. The repeated-work
and input-custody sites remain present. The v4 installed-schema receiving case and
[Plan69c's grouped fixture preparation](69c-ordinary-testing-and-managed-results.md#c06--reduce-repeated-fixture-backed-preparation)
keep their scoped evidence. Plan70 improves
the consumed compiler and fixture contracts without taking over the Plan69
managed-result lifecycle or Plan66's scientific denominator. Retain the existing
compiler reuse policy under [E05](70e-preparation-and-verification-scope.md#e05).
Treat cache recovery, actual execution and evidence from an original run as
separate operations. The efficiency review's source census is context for the
proposals. Actual selection and measured benefits require their own evidence.

## Ownership and execution order

| Companion | Scope | What it supplies |
|---|---|---|
| [70a](70a-selection-and-execution.md) | R01 | Exact native runner and compiler proof, selected requirements, actual shared-effect scope and specialist routes |
| [70b](70b-fixtures-and-process-ownership.md) | R02 and R04 process capture | Original process and deadline contract, representative borrower cleanup, authentic settlement and reached-stage controls |
| [70c](70c-test-support-and-contract-evidence.md) | R03 and R05 | Local support, typed controls, independent predecessor and final-apply fault evidence, pruning and governance |
| [70d](70d-scientific-references.md) | R04 | Independent expectations, actual environment and solver provenance, and bounded reference response handling |
| [70e](70e-preparation-and-verification-scope.md) | R06 | Demand-selected source closure, authenticated group and page verification, and proportionate graph acquisition |
| [70f](70f-conformance-and-benchmark-execution.md) | I05, I06, I14 | Accounted conformance evidence, run-local source reuse, and truthful smoke and prepared benchmark routes |
| [70g](70g-developer-verification.md) | I07, I08, I15, I16 | Direct codegen verification, consumer-scoped hosted setup, supplier preparation decision and scope-end composition |

Start A01, B01, C01/C02 and C04's pure controls, D01/D02, and E01/E02/E04 on their
existing inputs as capacity permits. E02 uses the delivered Plan69c membership and lease
mechanisms and must verify their applicability to each proposed group handoff.
A03 can map contention early. Its enforcement consumes A02's selected requirements and its
receiving checks consume B02's drain contract. B01 settles the capture slice of B02, while
deadline composition can advance independently. B03/B04, C03 and D03 need the relevant
B02 contract before their physical receiving checks. A04 migrates each route when its exact
input exists, including D02, E01 and the qualification deadline boundary. E03 depends on
E02's authenticated handoff and the actual binary membership for that handoff. This order
does not require a companion to finish before an independent package can start.

Prioritize A01's exact compiler comparison, D04's verified input custody, F03's
benchmark reach and F01's active report pressure. A01 supplies the shared comparison
before E03 migrates its relevant receivers and supplies E05's graph basis.
D04 and F02 consume E01's selected source contract for integration. Their local
input and admission checks can advance on the existing paths. B04 and B05 consume
B02 for real service capture and worker settlement respectively.
F01 and F03 can advance independently of the reference work. The G packages
can start on their existing generator and tooling inputs, with G04 supplying Q01's
qualification composition. If a conditional inquiry fails, retain its current
conforming mechanism and record what would reopen the decision. Required fixes
and their evidence still need an executable route.

The respective plan owner records the deciding evidence for each inquiry. A failed feasibility
case leaves the consuming slice explicitly gated with its unmet contract. It does not excuse
a requirement or prevent independent fixes. Keep the selected configured ordinary host capacity,
force-validation, pinned IDAES boundary and authentic managed release. Any newly discovered
rule impact goes through maintainer decision and the appropriate decision route before dependent
scope adopts it.

Plan66l L01/L03 owns the exact configured scientific population and assembled scientific
acceptance. Plan69c owns ordinary admission, exact failed rerun, managed result custody and
binary membership/capture. Link their exact source, case or identity evidence where consumed.
The new work tests its changed interfaces, retaining Plan69's live verdict, rerun and settlement
receiving controls. Plan53 is historical. Plan68 retains its outstanding platform qualification
and historical custody. [B04](70b-fixtures-and-process-ownership.md#b04) uses the selected v4
fixture and retains the SurrealKit, settlement and artifact-identity obligations.
Plan66l L04 owns simulator-wide complete-operation benefit claims. Plan70 verifies
the changed conformance and benchmark paths and links matching measurements where
needed. Plan69e retains its full-generation decision for guarded previews.

[70a's execution checkpoint](70a-selection-and-execution.md#execution-checkpoint)
records the active selection and execution scope and its remaining receiving checks.

## Verification scenarios

- <a id="s01"></a>**S01 — Selected route.** Add or rename a case, remove a prerequisite or Arrow
  validation leaf, substitute a runner, and present exit zero with incomplete evidence. Observe
  actual selection, readiness, verdict and settlement. A pure case stays effect-free.
- <a id="s02"></a>**S02 — Native effects.** Overlap two invocations on the same effect and with a
  private sibling. Cancel during original work and at the deadline boundary. Check real drain,
  retained custody when unresolved, representative borrower cleanup and a wrong setup failure.
- <a id="s03"></a>**S03 — Independent evidence.** Perturb a result and inflate the candidate allowance.
  Change units, release, tool identity or input correspondence. Exercise a child that overflows then
  exits zero and oversized/truncated data. Preserve the expected physical scope and original drain.
- <a id="s04"></a>**S04 — Contract evidence.** Challenge original predecessor scope, committed final
  apply with lost acknowledgement, absence and failed readback, wrong diagnostic code and wrong
  compile failure. Compare pruned results against unpruned data with multiplicity and limit ordering.
- <a id="s05"></a>**S05 — Focused work.** Select one family, one group and one retained page. Alter
  related and unrelated inputs, including a helper nextest opens before name filtering. Verify
  bounded acquisition and unchanged authority. Compare membership-only Cargo work with resolved
  graph governance.
- <a id="s06"></a>**S06 — Conformance and benchmark scope.** Exercise many small fixtures
  under a high report cap, out-of-order completion and pressure. Compare retained
  outcomes and pool release. Run local smoke with no canonical receipt, and the named
  canonical and solver cases only with the corresponding prepared owner. Observe the
  selected population, sampled phases and original cleanup.
- <a id="s07"></a>**S07 — Developer verification.** Exercise stale, untracked and
  invalid generated output together with an independent mismatch. Confirm cold hosted
  paths use only selected setup and retain their original outcomes. Compare the
  combined scope-end leaves and their failure attribution.

## Packages

| ID | Status | Prerequisites | Completion |
|---|---|---|---|
| [Q01](#q01) | in-progress | Functional packages integrated, G02 hosted evidence | Assembled changed-scope qualification and a reconciled finding disposition for each required claim. |
| [Q02](#q02) | in-progress | Replacement review, then Q01 for retirement | Enduring contracts and operator documentation are owned, replaced paths are absent, and closure distinguishes tested scope and any measured benefit. |

### <a id="q01"></a>Q01 — Assemble assurance after functional scope

Use targeted compile and owner unit controls during implementation. At scope end run selected
pure and Arrow/doctest journeys, ordinary and exact rerun receiving checks, required specialist
routes, real service and process boundaries, independent reference cases, conformance and
benchmark controls, and changed developer verification. Use
[G04's verified composition](70g-developer-verification.md#g04) for hygiene and governance
at the same source and context, with relevant manual docs, native and feature/powerset checks.
Use each standalone route where applicable evidence differs. Record command, selected
mode/conditions, actual scope and results against zero failures. Link matched Plan69 receiving
evidence. Record Plan66's configured scientific acceptance separately at its source owner.
Measure only where a conditional decision or quantitative claim requires it.

Local receiving checks are recorded in their owning packages. The current lock and the
execution baseline both expose one `family-check` finding against zero. The normal
compiler closure reaches Tokio through Protovalidate Buffa and ConnectRPC. The full
dependency policy and that diagnostic remain in force. G02's hosted cold setup still needs
evidence from the changed workflow. Plan66l's configured scientific population retains its
separate acceptance and schedule.

### <a id="q02"></a>Q02 — Reconcile ownership, evidence and retirement

Check each named consumer moved to the selected contract and obsolete support, routes and fixtures
are removed with their replacements. Move enduring contract and operating guidance to the
appropriate owners. Record actual verification, outcome, meaningful corrections and deliberate
deviations using the repository evidence vocabulary. Preserve historical findings and evidence
lineage when retiring work. Reconcile the DTA and improvement dispositions with package
evidence. Record the outcome of each conditional inquiry and its revisit condition where
the current mechanism was retained.

The owning packages record the scoped replacements and the decisions to retain export
preflight, borrowed inventory authentication and the existing compiler cache policy.
The [qualification guide](../dev/validation-assessment.md) and
[environment guide](../dev/agent-environment.md) own the operator routes. Retirement awaits
Q01's remaining qualification evidence.

## Finding dispositions

This is Plan70's disposition owner for its scope. The review's exact preservation and retirement
reasons are retained below. A resolved preservation row carries its scoped evidence and supplies
no new qualification. The external rows are explicit dependencies on Plan66l's own status and
evidence. Companion package tables own progress.

| Finding reference | Scenario reference | Disposition | Decision/work owner | Evidence or revisit trigger |
|---|---|---|---|---|
| [DTA-001](../design_review/test-audit/oracles-and-acceptance.md#dta-001) | — | superseded | — | Withdrawn in its original scope. R04 addresses the different candidate allowance finding, DTA-061. |
| [DTA-002](../design_review/test-audit/oracles-and-acceptance.md#dta-002) | — | resolved — scoped preservation | — | Keep the selected independent phase brackets and wrong-interior or missing-slack controls. |
| [DTA-003](../design_review/test-audit/oracles-and-acceptance.md#dta-003) | [S03](#s03) | open — external evidence | [Plan66l L01/L03](66l-required-population-and-assembled-acceptance.md#packages) | Match the simultaneous controller's exact case, configuration and result at Plan66l. Regeneration evidence remains separate. |
| [DTA-004](../design_review/test-audit/oracles-and-acceptance.md#dta-004) | — | resolved — scoped preservation | — | Keep the independent HiGHS full-status mapping control. |
| [DTA-005](../design_review/test-audit/oracles-and-acceptance.md#dta-005) | — | resolved — scoped preservation | — | Keep the independent PR jet check and its named derivative scope. |
| [DTA-006](../design_review/test-audit/oracles-and-acceptance.md#dta-006) | — | resolved — scoped preservation | — | Keep the real mixture input mutation and typed refusal controls. |
| [DTA-007](../design_review/test-audit/oracles-and-acceptance.md#dta-007) | — | resolved — scoped preservation | — | Keep the scoped finite-value, shape and time/extent checks. |
| [DTA-008](../design_review/test-audit/doubles-and-coupling.md#dta-008) | — | superseded | — | Withdrawn. Static scheduling controls and actual contextual readiness serve different claims. |
| [DTA-009](../design_review/test-audit/doubles-and-coupling.md#dta-009) | [S03](#s03) | resolved | [C01](70c-test-support-and-contract-evidence.md#c01) | Typed compiler observations passed the cold, warm and changed-input controls against fresh semantics. |
| [DTA-010](../design_review/test-audit/doubles-and-coupling.md#dta-010) | [S03](#s03) | resolved | [C01](70c-test-support-and-contract-evidence.md#c01) | The math and native residual controls passed with shared preparation, explicit inputs and independent numerical expectations. |
| [DTA-011](../design_review/test-audit/doubles-and-coupling.md#dta-011) | [S02](#s02) | resolved | [B03](70b-fixtures-and-process-ownership.md#b03) | The resumed private-service controls verified caller drop, sibling protection, and undrained refusal through production connection ownership. |
| [DTA-012](../design_review/test-audit/evidence-routes-and-execution.md#dta-012) | — | superseded | — | Original Python verdict path retired. Preserve managed nonpass and original drain/result separation through R01 and Plan69c. |
| [DTA-013](../design_review/test-audit/evidence-routes-and-execution.md#dta-013) | — | superseded | — | Original inactive Python control population retired. R01 checks current route correspondence under DTA-050. |
| [DTA-014](../design_review/test-audit/evidence-routes-and-execution.md#dta-014) | — | superseded | — | Original optional CLI outcome retired. Retain current declared native operator-evidence cases at their actual scope. |
| [DTA-015](../design_review/test-audit/evidence-routes-and-execution.md#dta-015) | [S01](#s01) | resolved | [A02](70a-selection-and-execution.md#a02) | The managed selection and live target-owner controls distinguished case identity, prerequisites, original verdict and settlement. |
| [DTA-016](../design_review/test-audit/evidence-routes-and-execution.md#dta-016) | [S01](#s01) | resolved | [A02](70a-selection-and-execution.md#a02) | Focused host controls refused renamed required cases and stale fixture declarations before preparation. |
| [DTA-017](../design_review/test-audit/evidence-routes-and-execution.md#dta-017) | [S01](#s01) | resolved | [A01](70a-selection-and-execution.md#a01) | Pinned Cargo and rustdoc distinguished an omitted validation leaf from ordinary library checking. The selected Arrow-consuming examples passed with validation. |
| [DTA-018](../design_review/test-audit/execution-efficiency.md#dta-018) | [S02](#s02) | resolved | [B03](70b-fixtures-and-process-ownership.md#b03) | Narrow refusals run with local owners and the real logical borrower controls retain their service boundary. B02 verified repeated-analysis deadline fit. |
| [DTA-019](../design_review/test-audit/evidence-routes-and-execution.md#dta-019) | — | resolved — scoped preservation | — | Keep the scoped unused-helper closure and current dev-only testkit boundary. |
| [DTA-020](../design_review/test-audit/oracles-and-acceptance.md#dta-020) | — | resolved — scoped preservation | — | Keep the named nonlinear curvature and reopened covariance cases in their native fixture scopes. |
| [DTA-021](../design_review/test-audit/oracles-and-acceptance.md#dta-021) | — | resolved — scoped preservation | — | Keep the named countercurrent and unequal-pressure physical controls. |
| [DTA-022](../design_review/test-audit/test-execution-system.md#dta-022) | — | superseded | — | Independent Python runner interpretation retired. Current native contract divergence remains DTA-063. |
| [DTA-023](../design_review/test-audit/test-execution-system.md#dta-023) | [S01](#s01) | resolved | [A04](70a-selection-and-execution.md#a04) | Named parity, reference, Miri, arbitrary and qualification routes exercised their intended prerequisites and actual owners. |
| [DTA-024](../design_review/test-audit/test-execution-system.md#dta-024) | [S01](#s01) | resolved | [A01](70a-selection-and-execution.md#a01) | The native correctness controls verified the configured runner pin and refusal for a wrong or changed executable. |
| [DTA-025](../design_review/test-audit/evidence-routes-and-execution.md#dta-025) | [S01](#s01) | resolved | [A02](70a-selection-and-execution.md#a02) | Setup refusal, the deliberate live assertion failure, and original physical settlement were observed separately. |
| [DTA-026](../design_review/test-audit/test-execution-system.md#dta-026) | — | superseded | — | The Python reciprocal tooling cause retired. Preserve lineage in the broad audit's DA-037 redirect. |
| [DTA-027](../design_review/test-audit/doubles-and-coupling.md#dta-027) | [S05](#s05) | resolved | [E01](70e-preparation-and-verification-scope.md#e01) | The shared catalog feeds the selected transitive package and fixture closure. Ordinary consumers passed without a prebuilt capsule. C01 checks dependency and lineage changes. |
| [DTA-028](../design_review/test-audit/doubles-and-coupling.md#dta-028) | [S03](#s03) | resolved | [C01](70c-test-support-and-contract-evidence.md#c01) | The selected modeling and host controls distinguished the original typed refusal and its receiving boundary. |
| [DTA-029](../design_review/test-audit/execution-efficiency.md#dta-029) | — | resolved — scoped preservation | — | Scenario feature variants retired. Keep actual service and maintenance capability distinctions. |
| [DTA-030](../design_review/test-audit/execution-efficiency.md#dta-030) | [S05](#s05) | resolved | [E01](70e-preparation-and-verification-scope.md#e01) | Selected family capture and its changed, missing and ambiguous input controls passed. Unrelated bodies stay outside the materialized closure. |
| [DTA-031](../design_review/test-audit/execution-efficiency.md#dta-031) | — | superseded | — | The Python runtime fixture dependency for capability-only checks retired. |
| [DTA-032](../design_review/test-audit/execution-efficiency.md#dta-032) | — | resolved — scoped preservation | — | Retain compatible compiler-graph reuse at its Plan67d scope. Remaining variants are DTA-053. |
| [DTA-033](../design_review/test-audit/execution-efficiency.md#dta-033) | — | superseded | — | The Python extension preparation path retired. |
| [DTA-034](../design_review/test-audit/execution-efficiency.md#dta-034) | — | resolved — scoped preservation | — | Keep one managed source build/list per context. DTA-039 covers repeated verification of the resulting capture. |
| [DTA-035](../design_review/test-audit/execution-efficiency.md#dta-035) | — | resolved — scoped preservation | — | Keep the sampled removal of scientific package embedding. Generated descriptors establish a different contract. |
| [DTA-036](../design_review/test-audit/execution-efficiency.md#dta-036) | [S05](#s05) | resolved | [E01](70e-preparation-and-verification-scope.md#e01) | Passive affected preview passed the shared, single-family and conservative widening controls. |
| [DTA-037](../design_review/test-audit/execution-efficiency.md#dta-037) | — | resolved — scoped preservation | — | Keep one aggregate codegen comparison with separate focused routes. |
| [DTA-038](../design_review/test-audit/execution-efficiency.md#dta-038) | — | superseded | — | The Python invocation-state rewrite path retired. |
| [DTA-039](../design_review/test-audit/execution-efficiency.md#dta-039) | [S05](#s05) | resolved | [E03](70e-preparation-and-verification-scope.md#e03) | A page shares one verification traversal. Group controls refused changed consumed images and preserved ordinary launch and exact rerun. Borrowed launch retains whole-inventory authentication. |
| [DTA-040](../design_review/test-audit/execution-efficiency.md#dta-040) | — | superseded | — | Preserve the earlier rejected blanket aggregate-reuse claim. |
| [DTA-041](../design_review/test-audit/execution-efficiency.md#dta-041) | — | superseded | — | Preserve the withdrawal of the generic recovery machinery diagnosis. |
| [DTA-042](../design_review/test-audit/execution-efficiency.md#dta-042) | — | superseded | — | Earlier Python cleanup joins retired. Target-owned settlement remains DTA-060. |
| [DTA-043](../design_review/test-audit/execution-efficiency.md#dta-043) | — | superseded | — | Earlier shared Python service premise retired. Keep isolated logical databases within the native borrowed endpoint. |
| [DTA-044](../design_review/test-audit/execution-efficiency.md#dta-044) | — | superseded | — | Former Python canonical barrier retired. Preserve actual effect exclusion under DTA-046. |
| [DTA-045](../design_review/test-audit/execution-efficiency.md#dta-045) | — | resolved — scoped preservation | — | Earlier broad serialization was narrowed. Keep pure controls outside protection for real effects under R01. |
| [DTA-046](../design_review/test-audit/execution-efficiency.md#dta-046) | [S02](#s02) | resolved | [A03](70a-selection-and-execution.md#a03) | Distinct private services overlapped, the actual listener owner refused collision, and cancellation without a terminal witness retained original custody. |
| [DTA-047](../design_review/test-audit/execution-efficiency.md#dta-047) | — | superseded | — | Earlier generic reclamation path retired. |
| [DTA-048](../design_review/test-audit/execution-efficiency.md#dta-048) | — | superseded | — | Earlier growing Python checkpoint maps retired. |
| [DTA-049](../design_review/test-audit/oracles-and-acceptance.md#dta-049) | [S03](#s03) | open — external evidence | [Plan66l L01/L03](66l-required-population-and-assembled-acceptance.md#packages) | Match required configured seed population and assembled scientific acceptance at Plan66l. A04 and D01 own route and oracle quality. |
| [DTA-050](../design_review/test-audit/evidence-routes-and-execution.md#dta-050) | [S01](#s01) | resolved | [A04](70a-selection-and-execution.md#a04) | Selected reference, Miri, arbitrary, source and qualification cases passed through their performing owners. A02 owns the marked-helper selection. |
| [DTA-051](../design_review/test-audit/doubles-and-coupling.md#dta-051) | — | superseded | — | The demonstrated Python orphan population retired. Retain currently used Plan14 controls. |
| [DTA-052](../design_review/test-audit/execution-efficiency.md#dta-052) | [S02](#s02) | resolved | [B03](70b-fixtures-and-process-ownership.md#b03) | The deliberate live assertion failure released eligible images after verified drain. The host receiver refused release with incomplete original settlement evidence. |
| [DTA-053](../design_review/test-audit/execution-efficiency.md#dta-053) | [S05](#s05) | resolved | [E05](70e-preparation-and-verification-scope.md#e05) | The exact timestamp control passed on the narrower validated graph. Four empty binary harnesses and one empty doctest harness retired. Ordinary helper launch and exact rerun passed. |
| [DTA-054](../design_review/test-audit/oracles-and-acceptance.md#dta-054) | [S03](#s03) | resolved | [D01](70d-scientific-references.md#d01) | Eight generator compare routes validated retained inputs and provenance without regeneration. Scoped endpoint and mixer expectations use their actual independent evidence. |
| [DTA-055](../design_review/test-audit/doubles-and-coupling.md#dta-055) | [S02](#s02) | resolved | [B03](70b-fixtures-and-process-ownership.md#b03) | Production connection ownership replaced the cleanup executor substitution and the actual borrower and protection controls passed. |
| [DTA-056](../design_review/test-audit/oracles-and-acceptance.md#dta-056) | [S04](#s04) | resolved | [C04](70c-test-support-and-contract-evidence.md#c04) | Seeded identity and the engine and native complete-row comparisons passed, including the wrong-pruning oracle and original drain. |
| [DTA-057](../design_review/test-audit/evidence-routes-and-execution.md#dta-057) | [S02](#s02) | resolved | [B04](70b-fixtures-and-process-ownership.md#b04) | The original exec gate, preparation lock and child interruption witnesses passed at their performing owners. |
| [DTA-058](../design_review/test-audit/oracles-and-acceptance.md#dta-058) | [S04](#s04) | resolved | [C04](70c-test-support-and-contract-evidence.md#c04) | The selected taxonomy, leaf and delegated diagnostics, and locked compile-fail and neighboring examples passed. A04 owns parity route wording. |
| [DTA-059](../design_review/test-audit/doubles-and-coupling.md#dta-059) | [S02](#s02) | resolved | [B03](70b-fixtures-and-process-ownership.md#b03) | Shared authenticated fixture inputs served the live service cases. C01's exact dependency overlay control passed. |
| [DTA-060](../design_review/test-audit/evidence-routes-and-execution.md#dta-060) | [S02](#s02) | resolved | [B02](70b-fixtures-and-process-ownership.md#b02) | The work-cutoff reserve and both live consumers passed. Their effective runner cutoff leaves the original work and settlement windows available. |
| [DTA-061](../design_review/test-audit/oracles-and-acceptance.md#dta-061) | [S03](#s03) | resolved | [D01](70d-scientific-references.md#d01) | The actual covariance and sensitivity exchanges accepted valid witnesses and refused wrong values paired with inflated candidate allowances. |
| [DTA-062](../design_review/test-audit/oracles-and-acceptance.md#dta-062) | [S03](#s03) | resolved | [D02](70d-scientific-references.md#d02) | All four native parity exchanges authenticated their performing tools. The locked solver controls covered indirect initialization and alternative or noisy executables. |
| [DTA-063](../design_review/test-audit/test-execution-system.md#dta-063) | [S01](#s01) | resolved | [A01](70a-selection-and-execution.md#a01) | Generic correctness and managed discovery use the same pin interpretation at their own launch boundaries. |
| [DTA-064](../design_review/test-audit/test-execution-system.md#dta-064) | — | superseded | — | Old surviving-route dependency on Python launch tooling retired. Verify current route adequacy under DTA-050. |
| [DTA-065](../design_review/test-audit/oracles-and-acceptance.md#dta-065) | [S04](#s04) | resolved | [C03](70c-test-support-and-contract-evidence.md#c03) | Independent v2/v3 specification witnesses established the supported compatibility and retarget refusal. |
| [DTA-066](../design_review/test-audit/oracles-and-acceptance.md#dta-066) | [S04](#s04) | resolved | [C03](70c-test-support-and-contract-evidence.md#c03) | Unit recovery branches passed. The native final activation survived acknowledgement loss and reopened with the exact original receipt and verified physical settlement. |
| [DTA-067](../design_review/test-audit/oracles-and-acceptance.md#dta-067) | [S02](#s02) | resolved | [B04](70b-fixtures-and-process-ownership.md#b04) | Live SurrealKit distinguished the intended seed and setup failures from a wrong endpoint. C01 verified typed local refusal. |
| [DTA-068](../design_review/test-audit/doubles-and-coupling.md#dta-068) | [S02](#s02) | resolved | [B04](70b-fixtures-and-process-ownership.md#b04) | Both native preparation cancellation controls passed, preserving publication until successful retry. |
| [DTA-069](../design_review/test-audit/doubles-and-coupling.md#dta-069) | [S02](#s02) | resolved | [B04](70b-fixtures-and-process-ownership.md#b04) | The named real-manager cgroup, SIGKILL and exec-gate controls passed with complete settlement. |
| [DTA-070](../design_review/test-audit/doubles-and-coupling.md#dta-070) | — | superseded | — | Python orchestration and supervisor test mechanisms retired. Retained reference tools serve distinct claims. |
| [DTA-071](../design_review/test-audit/execution-efficiency.md#dta-071) | [S05](#s05) | resolved | [E04](70e-preparation-and-verification-scope.md#e04) | Membership and stray controls passed on locked membership-only metadata. Dependency policies retain their resolved graph. |
| [DTA-072](../design_review/test-audit/execution-efficiency.md#dta-072) | [S03](#s03) | resolved | [D03](70d-scientific-references.md#d03) | Both streams overflowed with an original zero exit and complete settlement. The managed receiver and independent producer refused excessive response extent. |

## Improvement dispositions

This table owns the Plan70 dispositions from the efficiency review. The packages own
execution state and the evidence for each decision. Investigations are planned work
whose findings determine whether the candidate is adopted.

| Improvement | Disposition | Work owner | Required evidence or trigger |
|---|---|---|---|
| [I01](../plan_improvement_reviews/plan70-repository-test-efficiency-2026-11-08.md#i01) | resolved | [A01](70a-selection-and-execution.md#a01) | The reference walk agreed on pure, Arrow and workspace root decisions. The workspace comparison records reduced edge visits as structural evidence. |
| [I02](../plan_improvement_reviews/plan70-repository-test-efficiency-2026-11-08.md#i02) | resolved | [A01](70a-selection-and-execution.md#a01) | Complete unit multiset comparison serves the migrated receivers. The prepared xtask receiver accepted permutation and refused multiplicity and context substitution. |
| [I03](../plan_improvement_reviews/plan70-repository-test-efficiency-2026-11-08.md#i03) | resolved | [E04](70e-preparation-and-verification-scope.md#e04) | Member-scoped hasher tests reject missing and competing owners. Full graph policies retain their evidence. |
| [I04](../plan_improvement_reviews/plan70-repository-test-efficiency-2026-11-08.md#i04) | resolved | [C01](70c-test-support-and-contract-evidence.md#c01) | The typed fixture and both query-memory cases passed under explicit Arrow validation, including pressure, cancellation, recovery and release. |
| [I05](../plan_improvement_reviews/plan70-repository-test-efficiency-2026-11-08.md#i05) | resolved | [F01](70f-conformance-and-benchmark-execution.md#f01) | Demand-accounted capacity, ordering, linked cancellation and release passed. The existing export preflight preserves publication order and refusal precedence. |
| [I06](../plan_improvement_reviews/plan70-repository-test-efficiency-2026-11-08.md#i06) | resolved | [F02](70f-conformance-and-benchmark-execution.md#f02) | Shared exact physical document ownership and separate semantic admissions passed. The native control preserved six IPC relations and the original unknown-fixture refusal after drain. |
| [I07](../plan_improvement_reviews/plan70-repository-test-efficiency-2026-11-08.md#i07) | resolved | [G01](70g-developer-verification.md#g01) | Fresh direct comparison, grouped hosted projections and registry reuse passed their focused checks. Per-tree Git tracking retains independent pathspec attribution. |
| [I08](../plan_improvement_reviews/plan70-repository-test-efficiency-2026-11-08.md#i08) | open — hosted evidence | [G02](70g-developer-verification.md#g02) | Local cold tool and stale-lock controls passed. The changed hosted jobs still need cold-run qualification. |
| [I09](../plan_improvement_reviews/plan70-repository-test-efficiency-2026-11-08.md#i09) | resolved | [B05](70b-fixtures-and-process-ownership.md#b05) | Observed phases drive ordering. The retained real-clock control passed with active dispatch and original settlement. |
| [I10](../plan_improvement_reviews/plan70-repository-test-efficiency-2026-11-08.md#i10) | resolved | [E05](70e-preparation-and-verification-scope.md#e05) | Pinned runner and rustdoc confirmed the empty harnesses. The focused timestamp graph and the helper's live launch and exact rerun passed. |
| [I11](../plan_improvement_reviews/plan70-repository-test-efficiency-2026-11-08.md#i11) | resolved | [D04](70d-scientific-references.md#d04) | The fresh binary producer and receiving controls passed, including transient substitution and a single property decode. |
| [I12](../plan_improvement_reviews/plan70-repository-test-efficiency-2026-11-08.md#i12) | resolved | [B04](70b-fixtures-and-process-ownership.md#b04) | The pinned live suite proved exact two-case negative selection and bounded report admission. The full 19-case positive selection remains. |
| [I13](../plan_improvement_reviews/plan70-repository-test-efficiency-2026-11-08.md#i13) | resolved | [E04](70e-preparation-and-verification-scope.md#e04) | The five targeted governance suites passed, including relocated path rendering and checkout refusal. |
| [I14](../plan_improvement_reviews/plan70-repository-test-efficiency-2026-11-08.md#i14) | resolved | [F03](70f-conformance-and-benchmark-execution.md#f03) | Local and prepared smoke passed at their named scopes. Equivalent phase, admission and cleanup observations support K4's immutable source reuse across fresh scientific work. |
| [I15](../plan_improvement_reviews/plan70-repository-test-efficiency-2026-11-08.md#i15) | resolved | [G03](70g-developer-verification.md#g03) | The isolated locked stock build can be reused. Supplier equality and fresh decoder outcomes passed, with cold and warm stock work measured separately. |
| [I16](../plan_improvement_reviews/plan70-repository-test-efficiency-2026-11-08.md#i16) | resolved | [G04](70g-developer-verification.md#g04) | The combined route preserves independent generator, family and governance outcomes with one occurrence of each shared check. |
