---
title: "Plan 53f: Effect-scoped scheduling"
status: draft
date: 2026-10-23
adrs: []
review_sources: [docs/design_review/reviews/design_review_test-architecture-target_2026-10-23.md]
scenario_sources: [docs/design_review/reviews/design_review_test-architecture-target_2026-10-23.md]
doc_role: plan
doc_owner: docs/plans/53-test-architecture-target.md
doc_topics: [current-work, design, validation, environment]
---

# Plan 53f: Effect-scoped scheduling

## Target and limits

**Proposed:** scheduling reflects actual effects and supported protection scope under [R08](../design_review/reviews/design_review_test-architecture-target_2026-10-23.md#r08). C02 owns requirement meaning, nextest/pytest and host/service owners perform scheduling, and the [coordinator](53-test-architecture-target.md#package-state-and-execution-route) owns progress. Accepted RC02 removes demonstrated accidental barriers while retaining necessary native/timing/canonical protection. No global capacity planner, default serial queue, automatic concurrency increase or change to ordinary unbounded build mode is selected.

DTA-044 does not establish that the remaining canonical barrier is unnecessary. Remove B05's two accidental fixture consumers, then retain canonical-owner exclusion for actual canonical work. Configured ceilings, resident demand, retained diagnostic state, live borrowers and timing isolation are different quantities. A per-run exclusive group cannot promise quiet across separate runs.

## Work packages

<a id="f01"></a>
### F01 — Remove proven pure-control barriers through shared requirements

After P00 publishes RC02 and C02 supplies actual catalog correspondence, remove broad scheduler requirements from pure KINSOL profile-identity and Ipopt status/bounds controls. Retain native thread pools, MKL/heavy scientific and timing requirements for cases that actually need them. Use existing [nextest groups and thread requirements](https://nexte.st/docs/configuration/test-groups/); human group displays are not a stable machine schema.

Migrate `.config/nextest.toml`, root/Python conftest grouping and managed pytest scheduler selection as consumers of the same effect facts. B05's capability and constructor/module-evidence Python cases lose their accidental canonical fixture and group together. A renamed/split/new test with unchanged effects retains the requirement or produces an exact stale/missing mapping refusal, rather than silently running unprotected. Keep C02's actual effect guard before creation.

Acceptance compares actual selected requirements across managed affected/native/Python routes, shows known pure cases require no solver/canonical reservation, and confirms genuine native/timing/canonical cases retain protection. Delete broad regex interpretations and route-specific duplicated tables only where replaced by the selected declaration contract. No benchmark is required to establish removal of the accidental barrier.

<a id="f02"></a>
### F02 — Honest service-quiet scope

Resolve I08 for tests that assert service-wide quiet: choose an invocation-owned supervised service once per relevant invocation, or narrow the assertion to the selected database when that is the true supported claim. A separate service per test is not the default. An invocation-owned service includes creation borrower, generation identity, actual member association, shutdown/drain and retained-reader protection; E03 supplies conservative active-access checks, not an absence-based safety shortcut.

A service-wide claim must survive two separately started runs, including one registering/retaining access while another asks for quiet/maintenance. A merely nextest-exclusive or pytest-local group does not satisfy it. If process/service isolation cannot supply the claimed supported scope, preserve conservative protection and report the unsupported claim until an explicit narrower assertion is selected. Do not quietly weaken a real scientific/lifecycle requirement to make parallelism possible.

Migrate service fixture selection, assertion owner and cleanup consumers together. Preserve ordinary unbounded builds and actual timing isolation. Remove replaced shared-service assumptions only after the chosen scope and cross-run control pass. This package does not widen remaining canonical concurrency.

<a id="f03"></a>
### F03 — Explicit bounded local settings without capacity fiction

Use explicit smaller bounded settings for local fixtures whose responsibility permits them, retaining real test inputs and receiving-budget refusals. A configured 64 GiB ceiling is not reservation or measured demand. Keep the functional-store 8 GiB role distinct from the exclusive observer's 144 GiB envelope (72 GiB receiver + 64 GiB observer + 8 GiB control), and keep host versus store demand distinct. These are selected role premises, not a machine-wide scheduling model.

Targeted tests verify the intended role settings are passed and sufficient for the same functional case, with the meaningful limit/refusal controls preserved. No OOM cause is inferred from historical width-16 startup failures versus width-4 success. Widen concurrency only if a concrete supported claim is later selected and actual simultaneous demand/lifetime premises are established; retain the barrier meanwhile. A future quantitative throughput/capacity claim would require its own declared measured baseline and conditions, not a probe campaign to close this plan.

## Integration and completion

C02 is the semantic prerequisite for F01; B05 migration is necessary before removing the two Python groups. F02's scope decision and lifecycle contract precede relaxing service-quiet exclusion. F03 does not depend on blanket barrier removal. Coordinate conftest/nextest/service fixtures with C/B/E writers. 53g qualifies the chosen cross-run scope and preserved native/timing/canonical cases; no unsupported wider concurrency is required for completion. Retain canonical protection and document its support boundary rather than leave an invented optimization backlog.
