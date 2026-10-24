---
title: "Plan 53c: Invocation, requirements and outcomes"
status: done
date: 2026-10-23
adrs: []
review_sources: [docs/design_review/reviews/design_review_test-architecture-target_2026-10-23.md]
scenario_sources: [docs/design_review/reviews/design_review_test-architecture-target_2026-10-23.md]
doc_role: plan
doc_owner: docs/plans/53-test-architecture-target.md
doc_topics: [current-work, design, validation, environment]
---

# Plan 53c: Invocation, requirements and outcomes

## Target and existing owners

**Proposed:** the requested selection, tool-produced selected catalog, preparation/effect requirements and reconciled outcome describe the same invocation. Implements [R04/R06](../design_review/reviews/design_review_test-architecture-target_2026-10-23.md#r04); progress and finding closure stay at the [coordinator](53-test-architecture-target.md#package-state-and-execution-route). Existing `select`, `native_tests`, `test_run`, `arrow_validation`, `validation` and pytest owners remain; introduce neither a universal runner nor an exact-name acceptance manifest (blueprint §24.1).

Interpretation is pure and consumed by performing owners. A composition root may orchestrate selection/build/effects/reporting, but lower owners do not import each other's performing semantics to reconstruct the request. Names below describe meaning, not required new serializable types or files.

## Packages

<a id="c01"></a>
### C01 — Normalize the invocation once

Resolve supported Cargo/nextest/pytest package, feature, target, filter and profile forms before effects. Account for aliases such as `-P`, effective `NEXTEST_PROFILE`, explicit/environment/default profile, custom configuration, default filters and the precedence of tool base configuration. Arguments after `--` stay opaque program arguments. Refuse unknown scope-affecting options before preparing/building or allocating resources; do not silently assume a safe default. Feed one normalized value to discovery, validation, report association and execution.

Use the actual nextest machine-readable list and pytest catalog as selection authorities. Preserve package/binary/test/filter-match facts and valid empty selection distinctly from uncollected scope. Keep tool incompatibility explicit. Targeted existing script/selection tests must show equivalent supported aliases/environment forms have matching selected/report identity, custom profiles/configs survive, opaque arguments are unchanged, and unsupported selection refuses before effects.

For the report overlay add direct exact `tomlkit==0.15.1` (already transitive in `uv.lock`), use `parse`, selected-profile `junit.path` update and `dumps`, and remove the regex rewriters in `rust_report_command` and `validation.native_report_config`. Preserve nested configuration, quoting and comments, and nextest's lower-priority tool base semantics; do not turn the overlay into an arbitrary overriding config. Concrete API evidence is the [0.15.1 source](https://raw.githubusercontent.com/python-poetry/tomlkit/0.15.1/tomlkit/api.py), carried from review R04/R05. During implementation confirm selected nextest compatibility through D01 and use the required library-documentation route for any new API questions. No library research campaign is needed for plan authoring.

<a id="c02"></a>
### C02 — Actual requirement and campaign correspondence

Attach fixture/effect/capacity and required-campaign-role facts once to actual selected catalog members. Prefer homogeneous binary/module defaults plus explicit exceptions; no magical name inference or hand-maintained universal exact-test manifest. Check stale/unmatched declarations and missing required roles. A campaign with a nonempty union must still fail when one required member is renamed, split or removed. Optional capabilities are explicitly selected rather than becoming an automatic all-features campaign.

At actual fixture/effect admission, guard the resolved requirement before creation: undeclared database/service/native demand cannot be discovered only after allocation. One requirement owner serves managed affected/native/Python/setup routes and supplies F01/F02 scheduling. Migrate their environment, association, scheduler choice and result composition as consumers of the contract; low-level Cargo remains usable with its narrower automatic association/cleanup scope documented. Delete duplicate route parsers/tables and dead recipe references after each consumer has moved.

Acceptance includes a renamed/new effectful test, unmatched exception, undeclared actual fixture, a lost required role hidden in an otherwise nonempty selection, equivalent route selection and the two B05 effect-free Python cases. The guard must reject before effects; collection itself must not allocate scientific resources. C01's pure interpretation slice is sufficient to start C02; unrelated outcome or scientific work is not a prerequisite.

<a id="c03"></a>
### C03 — Actual Arrow closure supplies assurance

Retire stale workspace `validate-features` metadata and its governance presence test. Use `arrow_validation.resolve` as the selected normal/build/dev, target/host Arrow-leaf activation owner for managed routes, installer and codegen build selection. Preserve Maturin's package/feature/profile/locked selection and installed-extension import/admission checks; those select/build behavior but do not prove Arrow leaf activation.

Migrate declarations, invocation adapters, installer/codegen consumers and governance tests together. Test pure closure adds no unrelated relation/Arrow target; selected Arrow consumers enable actual leaf validation; host/build/dev cases remain covered; unknown/uncovered consumer refuses. Do not replace actual graph proof with a feature-name check or introduce a new blanket provenance campaign. D02/D03 use this delivered contract before sharing correctness artifacts.

<a id="c04"></a>
### C04 — Reconciled public outcome and typed setup observations

After P00 publishes accepted RC01, managed correctness commands return zero only for complete reconciled Pass. Preserve raw `workload_exit`, cancellation and independently reported cleanup. A zero-exit child plus missing selected report must produce nonzero managed exit; failed/missing/uncertain selected evidence cannot pass. Migrate exit-only callers and intentional `test_actual_child_status_survives_missing_report_reconciliation` semantics together. Preserve actual failed reports and terminal freeze: later cleanup cannot rewrite a frozen assurance verdict.

Fixture/admission owners emit authenticated typed infrastructure failure into the current invocation observation before an unwrap/panic projection. The composer reports assertion-not-reached separately from product mismatch; missing setup observation remains unknown/fail. Never parse panic/log prose, treat an outage as the expected model refusal, skip it or pass it. A same-generation/config preflight may bound repeated outage setup while that exact premise holds; no forever/global negative cache.

Use C01 selected identity and C02 association for the selected managed route, but do not wait for every optional role to migrate. Targeted controls cover zero child/missing report, typed outage, invalid/missing observation, cancellation, failing child, terminal immutability and cleanup failure. E02 owns independent cleanup continuation; this package consumes its facts without conflating cleanup with scientific assurance.

<a id="c05"></a>
### C05 — Explicit active boundary and optional evidence scope

Resolve I07 for inactive/optional bodies: the Rust `pse-py` harness is intentionally disabled for the cdylib extension link boundary. Inventory its inactive assertions, test actual public Python-boundary producers in selected scope and explicitly dispose synthetic combinations without a public producer, then delete the dead Rust bodies. Enabling a dormant harness alone is not the target. Preserve lower schema/runtime/codec coverage and test categories; do not manufacture production APIs or claim equivalent public coverage for unreachable combinations.

Extract `qualify_operator` CLI evidence into a separately executed opt-in case/driver result. The substantive API-only path remains meaningful; if CLI is selected and its prerequisite is absent, report failure for that explicit scope. Add explicit component peer routing for pytest context overlap, rather than a broad unittest discovery rewrite. Required external-restart peer scope gets a real selected route, or retire the obsolete body/claim with a reason if the capability is unsupported. No automatic optional-feature sweep. C02 owns role correspondence so a required selected peer cannot quietly disappear.

Acceptance: actual Python assertions execute, selected optional CLI missing prerequisite fails while API-only scope remains independently substantive, the intended peer executes at component scope and selection loses no required role. Remove dormant assertions, mixed API/optional result coupling and obsolete peer references only with the corresponding migrated evidence or explicit retired unsupported claim.

## Integration and limits

P00's accepted rule publication precedes C02 scheduler policy changes and C04 exit changes. C01 can be designed/implemented first; D01 supplies exact nextest compatibility before depending on its metadata behavior. Shared editing surfaces include `test_run`, `validation`, `select`, nextest config, conftest, recipes and resource admission; coordinate one writer per shared surface. E01/E05 change stored layouts and consumers, not invocation semantics. 53g exercises one assembled selection→requirements→prepare→assert→drain→report journey with honest narrower low-level support. No new framework, generic scheduler or source-freeze requirement is introduced.

## Authorized integration boundary — 2026-10-24

Plan52i–52k own the canonical FastMCP registration, typed exposure adaptation,
reveal, attachment and derived interfaces. C01–C04 provide coherent callable
operation inputs, native admission and authentic structured outcomes for those
consumers; native Cargo/nextest/pytest syntax is an executor boundary. No separate
agent CLI, capability catalogue, scientific schema mirror or result store is built.
Protocol success never promotes incomplete test assurance. Broad parser extraction
stays with ADP03; shared-owner changes are coordinated at the operation seam.

For C05 the operator selected activation of the external-restart peer through an
owned real service lifecycle driver. For dormant diagnostics, test actual public
paths and retain lower-level schema/codec coverage; explicitly dispose synthetic
combinations with no public producer without claiming equivalent public coverage.
Optional operator evidence is independently selected and reported; its existing
native executor is implementation evidence, not FastMCP host qualification.

## Verification and Outcome — 2026-10-24

### What was built

**Implemented:** C01–C04 share a pure Cargo/nextest/pytest interpretation, selected-profile
TOMLKit report copy, authentic catalogues and one requirement owner. Managed fixture/effect
admission checks demand before effects; required campaign roles cannot disappear behind a
nonempty selection. Setup executes the actual selected unittest names in its child. The
installer, strict producer and codegen adapters prove actual Arrow target/host leaves and
retain the selected Maturin/Cargo build. Stale validation metadata, its presence test and
reciprocal duplicate parsers are removed.

**Implemented:** managed zero requires complete reconciled Pass. Raw child exit, known-empty
not-run refusal, preparation failure, authenticated typed setup facts, assertion reach,
cancellation, frozen assurance, authentic peer identity and independent cleanup are retained
through the established invocation/result owners. Rust setup errors report reach unknown;
Python pre-yield failures report not-reached. Passed intentional negative controls remain Pass.
No panic-text parsing, production IPC guard around direct solver calls, global governor,
second agent CLI or parallel result store is introduced. Plan53f owns scheduler refinement;
Plan52i–52k own the single FastMCP surface and its assembled qualification.

**Implemented and Tested:** C05 removes the two inactive `pse-py` diagnostic bodies without
enabling the cdylib harness. Actual Python settings/configuration and native route-refusal
projections execute. No public custom registry produces the synthetic SchemaUnknownReference
case; the synthetic NativeReuse report combining source/revision, physical/missing observations
and WorkflowPanic likewise has no public producer in this scope. Existing lower schema,
runtime taxonomy and codec/source/physical coverage is retained; equivalent public coverage
for those combinations is not claimed. A KINSOL layout-change sequence legitimately succeeds;
POUNCE-convex RequireReuse instead reports a real route-admission refusal. The final test
asserts that actual public contract. No test-only production constructor was added.

**Implemented and Tested:** operator graph/query evidence has two independently selected
opt-in roles. The existing database-fixture consumer runs substantive API checks first and
then the separate operator driver; missing selected prerequisites fail. Capturing the selected
executor through the existing initializer owner prevents Cargo alias replacement. The
context-overlap pytest peer is component scoped, with explicit native-context and role demand.
The restart driver selects native compiler inputs locally while retaining the parent runtime
library/source-installer context; its fresh nonresident service never grants authority over
the selected main service. Actual overlap and owned restart acceptance pass below.

### Verification

All passing checks below target zero failures; earlier failed/refused attempts remain failed
records and do not establish acceptance. Local development mode is used, with explicit actual
Arrow force-validation for correctness build/test closures and native libraries where needed.

- **Tested:** `scripts/pse-env -- .venv/bin/python -m unittest` with the 18 scoped modules
  `test_test_invocation`, `test_native_tests`, `test_test_run`, `test_test_resources`,
  `test_test_requirements`, `test_python_tests`, `test_scientific_inputs`, `test_setup_report`,
  `test_arrow_validation`, `test_python_extension`, `test_codegen_builds`,
  `test_producer_deployment`, `test_recipe_arguments`, `test_validation`, `test_run_inspection`,
  `test_regression_experiments`, `test_select` and `test_canonical_boundary_run`, each under
  `scripts.tests`: **318 passed**. Controls cover aliases/profiles/configuration/opaque arguments,
  unsupported/empty selection, actual declarations/roles, admission before allocation,
  validation leaves, typed setup, incomplete reports, cancellation, terminal immutability,
  cleanup/projection failure and authentic peers. Archive-consumer controls separately passed 13.
- **Tested:** `just py-sync-correctness auto` installed the actual force-validated native extension
  and regenerated matching stubs. Actual graph-only installer/codegen selection controls prove
  the selected Maturin normal/build/dev and host/target closures without executing a broad campaign.
- **Tested:** `just unit-package pse-operations setup_failure_payload --execution-effects
  native-local --features pse-operations/canonical-tests`: **1 passed**, 358 filtered out.
  A real selected setup subprocess through `setup_report.execute` passed two invocation controls.
- **Tested:** `scripts/pse-env --resource-class build --native -- .venv/bin/python -m
  scripts.python_tests` selected the settings diagnostic, native route-refusal diagnostic,
  capability-discovery hard-cut and canonical-constructor timeout cases, with `-m 'unit or
  component' -n 0` and a fresh `--basetemp`: **4 passed**. Invocation
  `68e16caff66940b4ad35a35d42708538` is complete Pass with four reported subjects; the two B05
  cases retain local/effect-free requirements. No-allocation guards remain enabled.
- **Tested:** `scripts/pse-env --resource-class build --native -- just unit-package pse-operations
  'test(canonical_candidates_inspection) | test(canonical_graph_explanation)' --features
  pse-operations/canonical-tests`: **5 passed**, 354 filtered out. Authentic invocation
  `07c21df0b71948e688834f692805a623` supplies the independent API-only result.
- **Tested:** the explicit `canonical_boundary_run operator` route with the current
  canonical-tools executor passed **2 tests**, 436 filtered out, in invocation
  `e4f184b396f144d795b643b8a4f58b47`. Both operator roles were actually selected.
- **Tested:** the overlap driver prepared and ran in one admitted `exclusive-observer` lifetime
  with a fresh temporary root. Its actual Rust and pytest component peers each passed one
  case, with distinct databases, matching imported Python artifact, complete Pass and removed
  resources. Authentic invocations `7521e4e56a894f8a98f8685a7cbac06e` and
  `36f2215a45b64aa99d94a7728bcbc429` carry both context-overlap roles. The composed result is
  `build/p53c-context-overlap-20261024-accepted/result.json`. Thirteen driver controls pass
  after fixing the disposed-resource reader; the earlier 318-test matrix remains its own scope.
- **Tested:** `scripts/pse-env --resource-class build --native=solver -- .venv/bin/python -m
  scripts.tests.canonical_boundary_run restart build/p53c-restart-20261024-v7 --executor
  build/p53c-operator-20261024-v4/tools.initializer` passed its one pytest component and
  distinct selected Rust peer. Parent `e162ee76bb9b4ec292f5d690a7c541c9` and child
  `2881dc93115e4cd893f157df7f67cc3d` are both complete Pass, with no unreported subjects or
  report errors. The parent retains the authenticated external-restart role/child terminal,
  complete owned drain and empty independent cleanup failures. The fresh service is stopped.
  Earlier source/environment drift, missing runtime/temp-directory prerequisites and occupied
  observer attempts remain refused/failed records; the stale waiting preparation was cancelled
  before service creation rather than promoted to Pass.
- **Observed limitation:** this acceptance exposed external server cache fragmentation, not
  additional test-binary scope. The main build took 13m30s; a concurrent worktree build took
  21m16s. Existing main receipt `8ef24b4c2e8c778c3b86` and selected build `81ef0cf3d75210ec44c3`
  have identical source/patch/features/profile, compiler tools and configuration; the context
  differs only in `LD_LIBRARY_PATH` and its native-directory snapshot. Worktree paths introduce
  further distinct keys. At that acceptance the server installer keyed complete caller contexts into
  separate source/target directories. Its release profile uses fat LTO and one codegen unit,
  making fallback compilation expensive. The separately authorized followup below corrects
  package reuse; the external release profile remains unchanged.
- **Implemented and Tested, followup:** installed external packages are selected independently
  of caller compiler/worktree/loader/cache context. The original complete receipt is retained;
  ordinary setup refuses an absent package rather than compiling. The actual
  `scripts.tests.surreal_install_equivalence_check` control reused existing build
  `3549c2a97bd565fd2497` across six caller-context variants, with all original receipt bytes
  unchanged, zero compiler captures/builds and zero service starts. The
  [substrate guide](../dev/surreal-substrate.md#installed-external-server-reuse) owns the
  enduring contract. `scripts/pse-env -- .venv/bin/python -m unittest scripts.tests.test_surreal_server
  scripts.tests.test_surreal_source scripts.tests.test_installation_context` passed 295 tests
  against zero failures after replacing an accidental compiler-path refusal with an explicit
  immutable-supervisor binding check. This is functional reuse evidence, not measured speed
  benefit.
- **Tested:** `just turn-end` completed ADR indexing and formatting with exit zero. The 20
  driver/requirement controls passed after formatting; unrelated dirty Python content was
  byte-for-byte preserved. Existing retained legacy-service cleanup warnings were not repaired
  or treated as product qualification.
- **Not run:** whole-repository hygiene, full suites/powersets, benchmark/reference campaigns,
  Plan52 FastMCP host qualification and 53g assembled-series acceptance, per the operator's
  functional-scope boundary. No measured speed improvement is claimed.

### A mistake made and corrected

The first new diagnostic case omitted authored starts and expected a KINSOL structural
reuse failure that the real public sequence does not emit. Actual execution exposed both
premises; it was replaced by the real route-refusal producer. Live operator execution also
exposed loss of required roles at the observer handoff and replacement of the canonical-tools
alias by another Cargo feature build. The composer now carries roles, and the existing
initializer capture preserves the selected executable before preparation. The overlap driver
also read a removed live projection after both actual peers passed; it now composes retained
resource entries and frozen results, including each peer's selected role and complete Pass.

### Deviations from the plan, deliberate

Public diagnostics cover real producers rather than manufacturing equivalents for synthetic
inactive combinations. Restart is activated through a real owned disposable service; it is
not retired and it does not reset the selected service. Rust helper reach stays unknown where
an error return alone cannot prove assertion-not-reached. Existing scheduling barriers remain
conservative for F01/F02 to refine; this scope does not become a global effect interception or
provenance/benchmark campaign. The companion remains retained for the active coordinator and
D/F/G consumers; enduring operation is in the validation guide and blueprint §24.1.
