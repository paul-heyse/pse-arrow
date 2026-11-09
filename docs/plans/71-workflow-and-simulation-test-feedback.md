---
title: "Plan71: Workflow and simulation test feedback"
status: in-progress
date: 2026-11-09
adrs: []
doc_role: plan
doc_owner: docs/plans/71-workflow-and-simulation-test-feedback.md
doc_topics: [current-work, validation, diagnostics, preparation, performance, storage]
review_sources: [docs/functional_reviews/workflow-simulation-test-execution-and-diagnostics-2026-11-09.md]
scenario_sources: [docs/plans/71-workflow-and-simulation-test-feedback.md]
---

# Workflow and simulation test feedback

## Target and authority

**Proposed.** A developer can select a case, learn whether it is ready, follow the work
through the owners performing it, and retrieve enough original evidence to decide what
to fix or rerun. Model-only edits can reuse a valid executable; current-source checks
still establish freshness. Test and simulation results remain exact about selection,
independence, failure, coverage and physical settlement. Routine evidence is bounded,
readable through `pse`, and automatically eligible for release under an explicit policy.

This is a separate developer and test workflow plan. It supersedes no plan and adopts
no unfinished work merely by citing another owner. [Plan 66](66-integrated-simulation-target.md)
retains its scientific target, implementation and assembled qualification. Plan 71
uses selected scientific journeys to validate its own tooling; its results cannot close
Plan 66 acceptance. Plan 66 documentation is outside this series.

**Interface-checked baseline.** The
[functional extension review](../functional_reviews/workflow-simulation-test-execution-and-diagnostics-2026-11-09.md)
was published at `b9eb9f37d` against the inspected Plan 66 working tree. Plan authoring
used that publication and the current checkout's owner contracts. The historical native
workflow had setup failures, an unresolved staged edit, and an interruption; a smaller
follow-up reached a distinct fixture drain failure. The standalone seed campaign aborted
before its first report. Those observations motivate different acceptance scenarios.
They do not identify a solver, scheduling, or compiler-cache cause and are not a
performance comparison. Core 3.4, efficient architecture heuristics, test assessment
and process-simulator 1.5 apply through the [binding](../design_review/design_principles/standard.toml).

Cargo owns current build freshness; Nextest owns native selection and scheduling;
fixture and runtime owners supply physical and scientific facts. Reuse the delivered
test selection and fixture preparation in [Plan 70](70-test-assurance-architecture.md),
original result and CLI contracts in [Plan 69c](69c-ordinary-testing-and-managed-results.md),
and capture, cache and retirement mechanisms in [Plan 67](67-storage-lifecycle-and-build-reuse.md).
The current implementation already captures build outputs, coalesces helper builds,
reuses Nextest binary metadata, publishes bounded runtime observations and expires
ordinary compact test summaries. The changes here join or extend those owners;
they do not reimplement a runner, scientific engine, cache, or history service.
No governance change or architecture exception is selected.

## Series and delivery order

| Owner | Contract |
| --- | --- |
| [71a — CLI inspection and evidence lifecycle](71a-cli-inspection-and-evidence-lifecycle.md) | Shared references, readable observation, data budgets and owner release. |
| [71b — Nextest execution and original attempt evidence](71b-nextest-execution-and-original-attempt-evidence.md) | Runner pin, exact attempts and comparison, current-source build explanation. |
| [71c — Conformance readiness, execution, and recovery](71c-conformance-readiness-execution-and-recovery.md) | Selection/readiness and prepared campaign execution, partial result and settled interruption. |
| [71d — Fixture efficiency and selected diagnostics](71d-fixture-efficiency-and-selected-diagnostics.md) | Narrow effect exclusion, semantic preparation reuse, exact cases and targeted diagnostic escalation. |

Start A01 (identity and lifecycle foundation) and B01 (Nextest cutover) independently.
C01 (readiness) can start from the existing campaign contract. A02's host facts and C02's
scientific projection integrate as each producer is available; neither requires the
other's complete companion. B02 uses A01 and the qualified B01 reader. B03 can investigate
with existing build evidence; its new view consumes A02. A03 integrates each producer's
release in the change that introduces its stored data. D01 first uses the phase and
owner evidence to narrow actual exclusion, and D02/D03 keep scientific and diagnostic
changes local to a named consumer. Allocate shared CLI schemas and invocation ownership
to 71a and coordinate edits to the same surfaces before integration.

## Opportunity dispositions

This table owns adoption; the companions hold package progress. `Scheduled` means
proposed for execution when this draft is approved. A conditional route is investigated
only under its named trigger; rejection of an alternative does not leave the opportunity open.

| Opportunity | Disposition | Work owner | Evidence or revisit trigger |
| --- | --- | --- | --- |
| [O01](../functional_reviews/workflow-simulation-test-execution-and-diagnostics-2026-11-09.md#O01) | Scheduled | [A01–A03](71a-cli-inspection-and-evidence-lifecycle.md#packages), [D03](71d-fixture-efficiency-and-selected-diagnostics.md#d03) | Owner-derived progress and bounded retrieval; opt-in future metrics only for a remaining attribution gap. |
| [O02](../functional_reviews/workflow-simulation-test-execution-and-diagnostics-2026-11-09.md#O02) | Scheduled | [C02–C03](71c-conformance-readiness-execution-and-recovery.md#packages) | Partial evidence survives published boundaries, separates export and drain, and does not reissue effects. |
| [O03](../functional_reviews/workflow-simulation-test-execution-and-diagnostics-2026-11-09.md#O03) | Scheduled | [C01](71c-conformance-readiness-execution-and-recovery.md#c01), [B03](71b-nextest-execution-and-original-attempt-evidence.md#b03) | Passive readiness and valid image reuse; no speculative crate split or second build cache. |
| [O04](../functional_reviews/workflow-simulation-test-execution-and-diagnostics-2026-11-09.md#O04) | Scheduled | [B01](71b-nextest-execution-and-original-attempt-evidence.md#b01), [D01](71d-fixture-efficiency-and-selected-diagnostics.md#d01) | Native round trip on 0.9.151, actual group/fixture ownership, guarded narrowing where independence is established. |
| [O05](../functional_reviews/workflow-simulation-test-execution-and-diagnostics-2026-11-09.md#O05) | Scheduled | [B02–B03](71b-nextest-execution-and-original-attempt-evidence.md#packages) | Exact attempt detail and comparison; recordings only for a specific scheduling question that existing facts cannot answer. |
| [O06](../functional_reviews/workflow-simulation-test-execution-and-diagnostics-2026-11-09.md#O06) | Scheduled | [D02](71d-fixture-efficiency-and-selected-diagnostics.md#d02) | One bounded identity pilot and review of actual reuse seams; no speculative cross-run runtime pool. |
| [O07](../functional_reviews/workflow-simulation-test-execution-and-diagnostics-2026-11-09.md#O07) | Scheduled | [D03](71d-fixture-efficiency-and-selected-diagnostics.md#d03) | Extend existing diagnosis and assertions first; retain conditional escalation and rejected alternatives with rationale. |

## Acceptance and closeout

| Scenario | What must be demonstrated |
| --- | --- |
| Queued, running, refused and draining | From one original invocation a developer can distinguish checkout queue, group predecessor or observable native attempt, fixture setup, scientific work, publication and drain. Missing or stale detail stays visible. Passive CLI commands do not prepare anything. |
| Changed inputs | Unchanged and model-only requests reuse what remains eligible; the latter binds new input and runs fresh science. Rust, build-script, features or provider changes re-enter the correct admission. A historical image cannot qualify the current checkout. |
| Exact original runner evidence | Selected, filtered, not started, interrupted, attempted, retry/flaky, failed and passed stay distinct under the supported profiles. Compare explicit original runs without mistaking absence for success. Preserve force-validation and the exact failure rerun's independent current admission. |
| Conformance and recovery | Missing readiness fails before effects. A controlled cancellation stops new work and drains issued work; an abrupt pre-export abort leaves only actually published facts. A cleanup failure, export collision and unresolved staged edit retain their own outcomes. A selected exchanger journey still proves the qualified transfer and independently checked result. |
| Evidence release | Use a small budget and controlled clock. Eligible bytes are actually released; live, held, foreign, replaced or unsettled subjects cannot be deleted. Expiry is visible in CLI views, and intentional exports and shared caches remain outside automatic result expiry. |

Use selected `pse check` and `pse test` controls during implementation. Integrate the relevant
host, native fixture and selected conformance journeys once at scope end; perform applicable
scope-end checks using repository policy. No full simulation campaign is required by this
series. Record commands, scope, original outcomes and evidence labels when work occurs.

Adopt changes that clearly remove unnecessary work under the same correctness contract
without a speedup benchmark. Explain the removed build, scan, copy, allocation, repeated
preparation or exclusive occupancy and its conditions. Use one matched end-to-end comparison
only if a credible substantial degradation risk remains after inspection and correctness
controls (for example, new work on every test or increased contention for one server).
Match inputs, effective selection, profile, cache and service state; include settlement
when it is part of the operation. Quantified performance claims require measurement;
absence of such a claim is not an incomplete package.

Retire replaced paths when their consumers move. At closure, move enduring contract and
operator guidance to the architecture and developer documentation owners. This draft
records no implementation or qualification, and no other plan status changes on publication.

## Execution checkpoint — November 9

The implementation was integrated into `main` in `b247fc69e`. The initial
qualification below used the isolated worktree. Normal PATH now resolves the
same pinned Nextest 0.9.151 after an explicit managed installation. Existing
Plan 66 evidence retains its original scope.

The host now has original phase-file, phase, and test-attempt views. Attempt comparison
uses original context and completed JUnit without replacing the performing verdict.
Campaign changes add explicit preparation/readiness, bounded checkpoints, scoped
cooperative interruption, and deliberate export. The selected DSL and fixture changes
split useful test identities and remove repeated immutable input work while preserving
independent science and fixture ownership.

**Tested:** The focused `pse test pse-host --lib --timings -E …` run selected 16
tests on the local managed context (`01a1c23e-7f49-7662-889c-2cafde9389f1`,
zero failures against a zero baseline). It exercised the original native selection
and exact rerun, host capture and release controls, and the opt-in timing path.
Passive reads showed live build activity, the committed Cargo timing file, and a
settled ordinary result with enrolled bytes. An exact case comparison kept the
two invocations' verdicts independent of the selected case's outcomes.
`pse test xtask:conformance_cli --features package-fixtures -E 'test(conformance_)'`
then passed five tests against a zero baseline (`01a1c248-8a6a-7733-a32a-4da41937420a`),
covering pure conformance, missing-root refusal, failed science, and partial export.
The host follow-up passed eight focused checks for original file custody, release,
cooperative child stop and CLI help (`01a1c24a-a10a-728b-b1d9-83e1d256d074`);
`pse test pse-authoring:dsl_examples -E …` passed the four renamed power grammar
cases (`01a1c24c-13f5-74df-90ef-a3bb7c976297`). Both had zero failures against a
zero baseline in the local managed context.
The original worktree controls did not qualify weighted runner admission or native
scientific journeys. On `main`, a four-test managed host selection
(`01a1c26f-9109-72df-afe6-38c6db389b7a`) passed and an exact CLI export of its
recorded Nextest 0.9.151 version file matched the committed digest. A second
managed host selection (`01a1c275-1e3e-7457-886c-fc26a6d29cdf`) passed five
tests covering original fixture effect attribution, JUnit overflow, the real
weighted Nextest wakeup, native selection, and original failure rerun across a
listing change. Both settled with zero failures against the zero baseline.
The JUnit overflow control covered an interim relay that was later removed at
the maintainer's direction; it is not part of the target acceptance.

On `main`, `pse test xtask --lib --features package-fixtures -E …` passed four
campaign controls (`01a1c279-3970-7458-b55c-b1f85affd5d9`), including a small
native caller and abrupt exit before/after checkpoint publication. The
`xtask:conformance_cli` selection passed six cases
(`01a1c27d-ae87-7766-a398-09cac33fabf0`), and
`pse test pse-authoring:dsl_examples -E 'test(/^power_/)'` passed all five
named cases (`01a1c27e-9e62-74d9-a47f-c99a61de7dc7`); zero failures against
the zero baseline in each case.

Remaining qualification includes the managed campaign signal/export journey, the selected fixture and exchanger journey, and scope-end
checks. The ordinary capture path has bounded stream prefixes and
release of newly enrolled, authenticated workflow members. Aggregate pre-admission
accounting is deferred for ordinary uncapped writers pending a review of observed
sizes. The temporary JUnit byte cap and related admission proposal were withdrawn.
The campaign checkpoint reservation is a separate existing bound. Ordinary raw
stream retirement has been integrated; campaign raw streams retain separate custody. The live owner
projection is not yet complete across all test, fixture and scientific phases.
No plan closure or measured speedup is claimed.
