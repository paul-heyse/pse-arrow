---
title: "Plan 53c: Invocation, requirements and outcomes"
status: draft
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

Resolve I07 for inactive/optional bodies: the Rust `pse-py` harness is intentionally disabled for the cdylib extension link boundary. Inventory its inactive assertions, add equivalent public Python-boundary tests in actual selected scope, then delete the dead Rust bodies. Enabling a dormant harness alone is not the target. Preserve each boundary guarantee and test category; do not copy low-level internal assertions into unrelated Python tests.

Extract `qualify_operator` CLI evidence into a separately executed opt-in case/driver result. The substantive API-only path remains meaningful; if CLI is selected and its prerequisite is absent, report failure for that explicit scope. Add explicit component peer routing for pytest context overlap, rather than a broad unittest discovery rewrite. Required external-restart peer scope gets a real selected route, or retire the obsolete body/claim with a reason if the capability is unsupported. No automatic optional-feature sweep. C02 owns role correspondence so a required selected peer cannot quietly disappear.

Acceptance: actual Python assertions execute, selected optional CLI missing prerequisite fails while API-only scope remains independently substantive, the intended peer executes at component scope and selection loses no required role. Remove dormant assertions, mixed API/optional result coupling and obsolete peer references only with the corresponding migrated evidence or explicit retired unsupported claim.

## Integration and limits

P00's accepted rule publication precedes C02 scheduler policy changes and C04 exit changes. C01 can be designed/implemented first; D01 supplies exact nextest compatibility before depending on its metadata behavior. Shared editing surfaces include `test_run`, `validation`, `select`, nextest config, conftest, recipes and resource admission; coordinate one writer per shared surface. E01/E05 change stored layouts and consumers, not invocation semantics. 53g exercises one assembled selection→requirements→prepare→assert→drain→report journey with honest narrower low-level support. No new framework, generic scheduler or source-freeze requirement is introduced.
