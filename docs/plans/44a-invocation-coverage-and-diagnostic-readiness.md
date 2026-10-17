---
title: Invocation coverage and diagnostic readiness
status: in-progress
date: 2026-10-17
adrs: [ADR-0192]
review_sources: [docs/design_review/reviews/design_review_pipeline-performance-optimization_2026-10-17.md]
doc_owner: docs/plans/44-pipeline-performance-optimization.md
doc_topics: [current-work, diagnostics, design, validation]
---

# Plan 44a: Invocation coverage and diagnostic readiness

## Purpose and foundations

[Plan44](44-pipeline-performance-optimization.md) owns combined scope, accepted RC01/RC02 and dispositions. This companion owns A-package progress and F07/F08, plus the minimum observation support needed for [44e's campaign](44e-short-run-investigation-and-optimization.md). **At authoring — Proposed:** all changes and checks below; the specified-change checkpoint records subsequent implementation.

**Interface-checked foundation:** `pse-runtime/src/workflow/run.rs`, `math/jobs.rs`, `workflow/fitting/oracle.rs`, `pse-diagnostics/src/observation/{producer,observation}.rs`, invocation documents and the current Python/CLI inspection/collection routes. Typed operation/attempt/producer identities, honest loss, selected raw streams, independently scoped native counters and pure inspection are useful existing capabilities. Their current explicit partiality does not establish false scientific completion. Fix propagation and publication ordering instead of replacing that system or universally increasing retention budgets.

The ephemeral fitting/simulation/shooting paths can dispatch before their invocation recorder exists; the worker captures that missing or outer context. Current first-supervisor-poll clock wording is honest but cannot explain all performing-owner stages. Final fitting also closes its operation before publishing integration completion, so raw records and retained current state disagree. These causes are distinct and require separate controls.

## A01 — One invocation context before dispatch

Implement accepted RC01 through the existing recorder/context owner. Allocate the invocation identity/recorder before constructing/submitting native work, then carry that same context into the actual worker, asynchronous supervisor and returned result. Its declared wall boundary starts at invocation establishment before dispatch, not source admission or earlier preparation; document excluded work. Preserve synchronous admission/refusal behavior and retained failure context when no handle is returned. Do not move dispatch merely to hide work in another interval.

Cover actual ephemeral/durable fit, simulation and shooting start routes; inspect ordinary modeling and nested starts for the same premise without assuming all early staged preparation is dispatched science. A nested invocation gets its own recorder and legitimate parent relationship, not accidental inheritance of the outer recorder. Context must survive thread migration without treating thread CPU as asynchronous exclusive CPU.

Migrate the declared clock scope, tests, Rust/Python result projections, inspection/export and case collectors that consume it. Use the existing registry/generator for any changed external schema and regenerate; remove replaced context setup and outdated comparisons without a compatibility execution path. Route blueprint §23.1/proposed ADR-0192 meaning before dependent implementation. Optional recorder/writer failure cannot change original scientific permission.

Acceptance: short real ephemeral and durable fits/simulations plus a shooting anchor expose actual native/integration and final-assessment stages under the returned invocation. A parent/nested control distinguishes recorder identities. Exercise synchronous refusal, cancelled/failed worker and failed final assessment. Capacity omission zero alone is insufficient: selected owner facts establish coverage independently. Do not fabricate intervals for work that was not instrumented.

## A02 — Publish integration terminal state before closing its operation

In the fitting experiment owner, finish the integration producer before finishing its containing operation. Preserve the terminal reason and all available numerical facts on success, derivative failure, domain refusal and cancellation. Inspect the corresponding shooting/window ordering and any shared helper callers to avoid fixing only the illustrative success path.

Acceptance compares retained current/recent producer records and selected `workflow.integration.v1` raw records for the same operation/generation. Completed operations retain the final available integration phase; interrupted work remains interrupted rather than becoming Completed by writer drain. Explicit record loss remains visible. Do not permit arbitrary updates to closed operations or reinterpret missing suffixes as numerical stagnation. Existing writer-failure behavior stays observational.

## A03 — Supply decisive owner facts only where needed

Inventory diagnostics against E01's concrete questions: source/compile/bind/rebind work; integration purpose and selected directions; native evaluation/setup/symbolic/numeric/solve counts; final assessment and result access; queue/wait/worker/cleanup facts. Existing counters, invocation stages, current/recent records, complete-selected streams and source inspection are the default.

For a material question lacking a decisive fact, add the narrow fact at its performing owner and project it through existing typed documents/inspection. Explicitly state activation, reset generation, population, coordinates, inclusive timing and unavailable states. Distinguish allocation checkout from factor validity, optional cache lookup from live owner reuse and queued work from execution. A new observer does not independently charge work, perform numerical probes on read or introduce a global event vocabulary for unrelated libraries.

Acceptance uses a small discriminating control for each added fact, including absent/disabled and interrupted collection. Selected complete capture must report actual coverage rather than being inferred from a budget. Ordinary hub-off/hub-on/selected-detail controls preserve original answers and relevant work counts; they do not promise zero observer cost or identical deadline-limited trajectories.

## A04 — Reusable short-case observation route

Use the existing `scripts/case_measure.py`, `scripts/case_audit.py`, declared process cases and public Rust/Python diagnostics. The review's two drivers are useful fixture seeds; move reusable case construction/assertions into their actual test/harness owners when repeated use warrants it, rather than importing disposable evidence as a permanent production authority. Preserve one case declaration per meaning and the distinction between admission, physical run and process elapsed.

Expose compact declared selection and optional cooperative observation intervals through the existing route. Keep one result per actual invocation, including failures; count cold/repeat/accepted-start/equal-preparation separately. New case metadata needs only the question, configuration, assertions, selected observations and interpretation limits—no fingerprints or environment proof. Use the public launch profiler separately where its lifecycle does not fit cooperative fixture ownership.

Requests to stop wait for actual handle/worker join and registered fixture cleanup. No kill fallback, resource-limit override, unrelated-service stop or claim that a terminal Python body establishes drain. Synchronous APIs without cancellable handles must state their limitation rather than being wrapped in a fictitious hard timeout. Targeted runner controls cover finite interval rejection, selection/order, preserved failed outcome and awaited cleanup.

## Work packages

| Packet | Prerequisite | Delivery and acceptance | State |
| --- | --- | --- | --- |
| A01 | Accepted RC01; current recorder/native dispatch | One pre-dispatch invocation context and migrated clocks/consumers; actual nested/ephemeral/durable/failure controls; delete replaced setup | done |
| A02 | Current producer/operation terminal contract | Integration terminal publication precedes close across applicable fit paths; retained/raw success/failure/cancel controls | done |
| A03 | E01 questions; existing producer population contracts | Only missing decisive owner facts and their actual consumers; activation/loss tests; no new work authority | open |
| A04 | Existing case declarations; E01 case specifications; A01/A02 where composed coverage is needed | Reusable short-case observation/stop route and targeted runner tests; remove replaced duplicate fixture logic | done |

## Verification and handoff

Use targeted unit/compile checks for affected owners and public Python integration controls. A01/A02 unblock trustworthy composed comparisons; A03/A04 can grow only as a concrete case needs them. Do not hold source-proven independent corrections for all observational tooling. Full quality and assembled checks belong to [44f](44f-integrated-acceptance-and-follow-up.md), after functional scope.

At authoring, all A packages were open. Document authoring itself implemented no recorder, producer or runner behavior; the checkpoint below records subsequent execution. Outcome will identify exact tested scope, remaining collection limitations and deliberate changes to the clock boundary.

## Specified-change checkpoint

**Implemented:** A01 creates a fresh recorder inside the entered activity owner before synchronous dispatch; the supervisor and native worker carry that same recorder. Invocation/comparison version2 excludes earlier preparation and refuses old supervisor-only comparison meaning. The actual native-worker owner supplies one bounded inclusive `native.worker_execution` interval. A02 publishes integration completion before closing its experiment operation. The native final-assessment and cancellation controls compare selected source records with retained producer state. All four actual start boundaries were migrated; assembled durable/live journeys remain with 44f. A03 currently supplies these decisive owner facts, sparse verification support visits, and `workflow.integration.v1.forward_direction_columns` from the fitting/shooting selected request. That field describes the physical response layout, not an executed-vector count, and non-forward work retains explicit NotRequested availability. Standalone trajectory observations project the report's `forward_axes`; its campaign-driven inventory remains open. A04 uses one declared audit capability per case and an existing case-owned finite detail session; runner tests cover stop/order/failure/drain. Four once-only harness readiness controls passed with original assertions, registered cleanup and normal owned-service stop. Actual selected-detail loss remains explicit; no campaign has run.

The coordinator owns command/result totals and the enclosing acceptance boundary. These package results do not close the 44e campaign or 44f qualification.
