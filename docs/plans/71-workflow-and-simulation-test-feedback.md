---
title: "Plan71: Workflow and simulation test feedback"
status: complete
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

This table owns adoption; the companions hold package progress. A conditional route is investigated
only under its named trigger; rejection of an alternative does not leave the opportunity open.

| Opportunity | Disposition | Work owner | Evidence or revisit trigger |
| --- | --- | --- | --- |
| [O01](../functional_reviews/workflow-simulation-test-execution-and-diagnostics-2026-11-09.md#O01) | Resolved | [A01–A03](71a-cli-inspection-and-evidence-lifecycle.md#packages), [D03](71d-fixture-efficiency-and-selected-diagnostics.md#d03) | Owner-derived progress and bounded retrieval; opt-in future metrics only for a remaining attribution gap. |
| [O02](../functional_reviews/workflow-simulation-test-execution-and-diagnostics-2026-11-09.md#O02) | Resolved | [C02–C03](71c-conformance-readiness-execution-and-recovery.md#packages) | Partial evidence survives published boundaries, separates export and drain, and does not reissue effects. |
| [O03](../functional_reviews/workflow-simulation-test-execution-and-diagnostics-2026-11-09.md#O03) | Resolved | [C01](71c-conformance-readiness-execution-and-recovery.md#c01), [B03](71b-nextest-execution-and-original-attempt-evidence.md#b03) | Passive readiness and valid image reuse; no speculative crate split or second build cache. |
| [O04](../functional_reviews/workflow-simulation-test-execution-and-diagnostics-2026-11-09.md#O04) | Resolved | [B01](71b-nextest-execution-and-original-attempt-evidence.md#b01), [D01](71d-fixture-efficiency-and-selected-diagnostics.md#d01) | Native round trip on 0.9.151, actual group/fixture ownership, guarded narrowing where independence is established. |
| [O05](../functional_reviews/workflow-simulation-test-execution-and-diagnostics-2026-11-09.md#O05) | Resolved | [B02–B03](71b-nextest-execution-and-original-attempt-evidence.md#packages) | Exact attempt detail and comparison; recordings only for a specific scheduling question that existing facts cannot answer. |
| [O06](../functional_reviews/workflow-simulation-test-execution-and-diagnostics-2026-11-09.md#O06) | Resolved | [D02](71d-fixture-efficiency-and-selected-diagnostics.md#d02) | One bounded identity pilot and review of actual reuse seams; no speculative cross-run runtime pool. |
| [O07](../functional_reviews/workflow-simulation-test-execution-and-diagnostics-2026-11-09.md#O07) | Resolved | [D03](71d-fixture-efficiency-and-selected-diagnostics.md#d03) | Extend existing diagnosis and assertions first; retain conditional escalation and rejected alternatives with rationale. |

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
operator guidance to the architecture and developer documentation owners. This series
does not change another plan's status or acceptance obligations.

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
the maintainer's direction; it is not part of the target acceptance. The
uncapped follow-up passed the direct runner, weighted wakeup, and exact rerun
controls. Four retirement controls initially refused an insecure test directory;
after the fixture explicitly created a private directory, `pse test --rerun-failed
01a1c280-af41-7245-b550-3cf0dc69b116` passed all four and settled with zero
failures (`01a1c283-46ba-7157-aab7-508c805ee580`). The explicit retirement preview
reported actual apparent and allocated bytes for a retained DSL run.

On `main`, `pse test xtask --lib --features package-fixtures -E …` passed four
campaign controls (`01a1c279-3970-7458-b55c-b1f85affd5d9`), including a small
native caller and abrupt exit before/after checkpoint publication. The
`xtask:conformance_cli` selection passed six cases
(`01a1c27d-ae87-7766-a398-09cac33fabf0`), and
`pse test pse-authoring:dsl_examples -E 'test(/^power_/)'` passed all five
named cases (`01a1c27e-9e62-74d9-a47f-c99a61de7dc7`); zero failures against
the zero baseline in each case.

**Tested:** On `main`, `pse test pse-runtime --lib -F
solver-ipopt,solver-kinsol,solver-root-isolation -E …` selected the independent
exchanger rating comparison, the costed exchanger protected view, and the
controlled conformance cancellation and fatal-fixture drain cases. With the
locked reference interpreter selected explicitly, all four passed with zero
failures against the zero baseline and complete settlement
(`01a1c291-6979-75ed-821c-512df4a99322`). The runtime controls establish issued
native work drain and returned versus unattempted status at the runtime boundary.

**Tested:** `pse test pse-operations:pse_operations --target-kind lib
-F pse-operations/native-service -E 'test(=native_fixture::logical::canonical_server_unit::fixture_explicit_drain_retains_database_for_runner_disposition)'`
passed the single selected case with zero failures against the zero baseline
and complete settlement (`01a1c297-9fb2-75ab-b50f-b04045fd5815`). It exercised
two independent databases on one runner-owned service and preserved final
database disposition with that owner. The terminal output projection rejected
an invalid field; inspection recovered the original successful test verdict.
The same package/target and native-service feature with `--ignored only`
then passed the three selected rollback/recovery, exclusive expiry and
cancelled-apply controls using the exact prepared module receipts and the
four-controller fixture (`01a1c2a0-8494-72ed-b410-48003161438c`, zero failures
against the zero baseline, complete settlement).

**Tested:** `pse test pse-host --lib -E …` passed all 16 selected controls after
the timer fixtures were corrected to use private unit directories
(`01a1c29f-1d30-7389-9cc4-ba4da5859406`, zero failures against the zero baseline,
complete settlement). The selection covered exact campaign release, held and
unsettled extents, interrupted release replay, replaced members, enclosing drain,
cooperative stop, timer ownership, CLI help, and current native-tool identity.
`pse tools result-expiry-status` reported the exact local timer active without
creating a result artifact.

The first managed campaign preparation compiled successfully but refused its own
binary: the requested feature roots and the compiler's resolved feature set were
compared as if they were the same. The owner now verifies the exact profile
request and independently binds the full resolved feature set to the compiler
artifact and its manifest. `pse test pse-host --lib -E
'test(prepared_xtask::tests::)'` passed all 14 controls with zero failures against
the zero baseline and complete settlement (`01a1c2af-a758-7324-821a-4c22f095f998`).
The live retry also exposed the old 1 MiB tool-manifest ceiling. The complete
campaign source fingerprint fit the existing compiler-document bound; preparation
published and reopened a 1,607,197-byte manifest. The two unpublished capture
directories were removed after the replacement became current. The managed pure
case then passed and, with only an authored expectation changed, returned the
intended scientific failure under the same executable selection
(`01a1c2b6-51e8-7027-a4e7-55ea300d5afe`,
`01a1c2b6-6eb8-7279-a0c4-118afd5998a3`).
The original passing scratch result previewed 12,288 allocated bytes. Its hold
refused release; clearing the hold allowed exact retirement. The CLI then
reported retired checkpoint and raw streams while preserving the original verdict.
The campaign nested under `pse run` became eligible only after its enclosing
workload drained (`01a1c2ba-7f18-7770-9503-e736d54806dd`); the CLI then previewed
the original allocated extent.
The selected native density case passed and drained, and its deliberate export
published six Arrow relations (`01a1c2b7-24b0-703c-9895-013706720af1`).
Two detached selections were stopped through `pse cancel` after a case had
entered `solve`. The mixed run retained its original findings and reported the
native shutdown error separately from the host's completed drain
(`01a1c2b7-4aec-7159-963f-4a4986592840`). The single-case control retained
`Cancelled` and an incomplete scientific result, reported the native runtime
drained without error, and `pse wait` confirmed process settlement
(`01a1c2b9-555c-74a8-80ac-4a3645657946`).
The final focused `pse test pse-host --lib -E …` selection passed nine runner,
attempt, phase and effect controls with zero failures against the zero baseline
and complete settlement (`01a1c2c5-b3ce-7308-9c6c-3155f52413cd`). It covered
zero match, profile and platform/ignored selection, weighted wakeup, retry and
flaky policy, and original fixture and phase-stream custody.

The ordinary capture path has bounded stream prefixes and
release of newly enrolled, authenticated workflow members. Aggregate pre-admission
accounting is deferred for ordinary uncapped writers pending a review of observed
sizes. The temporary JUnit byte cap and related admission proposal were withdrawn.
The ordinary release preview for the four-case native run showed 8,155,530
apparent bytes and 8,200,192 allocated bytes across 21 verified workflow members;
this is one retained selection, not an aggregate usage estimate.
The campaign checkpoint reservation is a separate existing bound. Ordinary and
campaign raw stream retirement are integrated under their respective owners.
The live owner projection reports bounded, owner-observed phases and keeps
unavailable runner and scientific detail explicit.

## Outcome

**Implemented:** The adopted scope is on `main`. Nextest 0.9.151 is the normal
runner. The CLI reads original observations and releases settled ordinary and
campaign evidence through the owners that captured it. Explicit exports and
reusable native images keep their own custody. Independent fixture preparation
and selected case evidence are available without broadening the scientific suite.

**Tested:** The selected runner, custody, campaign and scientific controls are
recorded above. After the final source edits, passive `pse conformance readiness`
refused the stale image without preparing or writing a result artifact
(`01a1c2c8-92e3-77ea-b8df-7052e2ff1f43`). An explicit preparation is needed before
the next campaign on these bytes. `just turn-end` passed both stages with zero
failures against the zero baseline.
`just codegen` published all 16 schema targets; `just codegen-check` passed
with zero differences against the zero baseline
(`01a1c2d0-01d5-72f7-a25b-82e0f2736fe1`).

The original repository-wide `pse run-all hygiene governance-tests` bundle
settled with failures (`01a1c2bb-79e5-71a7-83f8-1f65663f12c4`). Governance ran
25 tests: 23 passed and two failed against the zero baseline on the existing
`pse-quantity` unsafe-code fixture. Unrelated repository hygiene findings in
spelling, licensing, Python lint/type checks and dependency checks are deferred
to the maintainer's next pivot. Plan71 addressed its own strict Clippy findings,
the Rustix flag spelling, and affected generated outputs without changing those
other areas. The maintainer directed that Clippy not be rerun before the next scope.

Ordinary writer caps and aggregate admission remain deferred pending a later
review of actual result sizes. Conditional diagnostic recording was not needed
by the selected cases. No full scientific campaign or measured speedup is claimed.
