---
title: "Plan71d: Fixture efficiency and selected diagnostics"
status: draft
date: 2026-11-09
adrs: []
doc_role: plan
doc_owner: docs/plans/71-workflow-and-simulation-test-feedback.md
doc_topics: [current-work, validation, preparation, diagnostics, performance]
review_sources: [docs/functional_reviews/workflow-simulation-test-execution-and-diagnostics-2026-11-09.md]
---

# Fixture efficiency and selected diagnostics

The [coordinator](71-workflow-and-simulation-test-feedback.md#opportunity-dispositions)
owns O04, O06 and O07 disposition. Reduce repeated work only where case identity,
scientific independence and settlement remain exact. Use existing fixture, authored
source and runtime owners. No general cache, scheduler or diagnostic platform.

## Packages

| ID | Status | Prerequisites | Completion |
| --- | --- | --- | --- |
| [D01](#d01) | Planned | B01 runner; A02 owner-attributed evidence | Retain necessary exclusion and move demonstrated independent work outside it. |
| [D02](#d02) | Planned | A01 references; current selected science and fixture owners | Account for reusable work in two representative workflows; exact case pilot preserves evidence. |
| [D03](#d03) | Planned | A01/A03; named gap for any optional escalation | Existing diagnostic consumers cover selected cases; deeper tools have explicit triggers, CLI retrieval and release. |

<a id="d01"></a>
## D01 — Limit exclusion to the effect it protects

Nextest 0.9.151 `test-groups`, `threads-required` and priority control
invocation-local eligibility and dispatch. Two scientific tokens consumed by one
case express exclusivity, not a CPU cap. Store cases may have distinct database
IDs on the same server; server-wide or timing controls retain appropriate
exclusion. Priority can surface an informative failure earlier; it cannot order
dependencies. Keep cross-invocation placement and physical drain with the
original host and fixture owner. Read effective assignments and measured phase
observations where available; do not infer queue causality from elapsed time.
[Nextest group contract](https://nexte.st/docs/configuration/test-groups/) ·
[Current profiles](../../.config/nextest.toml) ·
[Runner cutover](71b-nextest-execution-and-original-attempt-evidence.md#b01)

In one selected fixture-backed journey, locate the real exclusive region and
look for source preparation or a standalone independent reference performed
inside it. Move that work only when its result can be joined to the exact input
and the integrated assertion still exercises the same receiving contract. If the
lease or science requires the work inside, retain the existing scope and record
why. Validate two independent legitimate borrowers, a server-wide exclusive
control, and cancellation where the slot or underlying resources remain owned
until their real settlement. Do not increase host concurrency or deadlines to
mask setup, staged-edit or drain errors.

Use existing [Nextest setup scripts](https://nexte.st/docs/configuration/setup-scripts/)
for run-scoped environment, including Cargo restoration; consider
[wrappers](https://nexte.st/docs/configuration/wrapper-scripts/) or target runners
only for a selected instrumentation or execution boundary. They cannot own a
borrowed service. Keep native deadline, process timeout, pipe leak symptom and
physical drain distinct. Do not suspend an owner when an external deadline or
lease continues, or use blanket retries. Keep the separate doctest route; consider
[archives and partitions](https://nexte.st/docs/ci-features/archiving/) only for
an actual second executor and an admitted ABI, loader and transfer contract.

<a id="d02"></a>
## D02 — Reuse immutable preparation and name the meaningful case

Trace the exchanger design-to-rating transfer and the selected fixture-backed
workflow. Separate authored capture, compiled structure and provider preparation
from mutable initialization, native solves, transactions and protected result
custody. Reuse only the existing owner with complete input identity: content and
revision, representation, provider, physical envelope, controls that affect the
artifact, and lease. Keep design artifacts qualified and rating consumption
explicit. A shared-flight waiter does not prove repeated construction. Run the
required current solve and independent reference for changed inputs or where
the claim requires independent execution. Retain the shared physical capture
already delivered by [Plan 70f](70f-conformance-and-benchmark-execution.md);
do not adopt a cross-run runtime pool for the one-native/one-pure seed manifest.
Where inspection finds no safe duplication, close the inquiry with that reason.

Pilot stable hand-named cases in one cheap DSL matrix and check list, exact run
and rerun identity after inserting another case. Batch cheap assertions only if
shared setup is relevant and failures still identify the exact input. For an actual
homogeneous admitted file corpus, compare
[`datatest-stable` 0.3.3 `harness!`](https://docs.rs/datatest-stable/0.3.3/datatest_stable/):
its [tagged runner](https://github.com/nextest-rs/datatest-stable/blob/datatest-stable-0.3.3/src/runner.rs)
recursively discovers files and avoids the scan in exact execution, which does
not reapply the discovery regex. Adopt only if original membership, fixture bytes
and corrected positive controls remain independently established. The current
ten-file modeling refusal corpus has those controls and is not the default pilot.
[`rstest` 0.28.0 named cases](https://docs.rs/rstest/0.28.0/rstest/attr.rstest.html#optional-case-description)
still include an ordinal; consider only for a matrix that need not keep durable
names across insertions. Defer `libtest-mimic` until a concrete non-file dynamic
population justifies owning discovery. No universal test-case registry.

<a id="d03"></a>
## D03 — Escalate a named diagnosis through an existing owner

Use typed errors and bounded `pse-testkit` capture for the selected workflows.
Keep attribution, coverage and closure assertions; use [`assert_cmd`](https://docs.rs/assert_cmd/latest/assert_cmd/cmd/struct.Command.html)
and [Insta](https://docs.rs/insta/latest/insta/struct.Settings.html) for selected
CLI contracts and stable presentation. Assert semantics on typed fields.
Defer Snapbox or trycmd unless a real transcript or filesystem corpus is simpler
with them. Do not snapshot unbounded logs, generated payloads or solver output
as a substitute for assertions.

| Unresolved question | Conditional mechanism and boundary |
| --- | --- |
| Are specific span/event fields contractually required? | Use existing [`tracing-subscriber` 0.3.23 layer filtering](https://docs.rs/tracing-subscriber/0.3.23/tracing_subscriber/layer/index.html); consider `tracing-mock` only for explicit event field/order assertions. Future-local dispatch does not automatically cover independently spawned tasks. |
| Where does one named future wait? | Use existing `tokio-metrics` 0.5.2 [`TaskMonitor`](https://docs.rs/tokio-metrics/0.5.2/tokio_metrics/struct.TaskMonitor.html) class observations first. [`FutureMonitor`](https://docs.rs/tokio-metrics/latest/tokio_metrics/struct.FutureMonitor.html) can add selected completed-future poll/idle detail. Root scheduling measurement requires `publish_scheduling_delay()`, includes possible other branches and costs extra work; a dropped future supplies no final sample. Consider [Tokio Console](https://docs.rs/console-subscriber/latest/console_subscriber/) only for a live stall that still lacks evidence and justifies the Tokio tracing/`tokio_unstable` build. Neither observes native solver CPU or proves drain. |
| Can a parser or receipt failure be reproduced through the public boundary? | Use existing [Proptest 1.11.0 failure persistence](https://docs.rs/proptest/1.11.0/proptest/test_runner/struct.Config.html). Retain a bounded concrete minimized witness only where seed/strategy is insufficient. A sequential property model cannot prove a native commit or drain; paused Tokio time remains a pure timer-policy control. |
| Is the unresolved issue a runner stall, native crash or oracle weakness? | Use targeted Nextest stress or the [B03 exact-run attachment](71b-nextest-execution-and-original-attempt-evidence.md#b03) for runner behavior. Use selected GDB or [`rr` 5.10.0](https://github.com/rr-debugger/rr/releases/tag/5.10.0) for a concrete native fault, not timing. Existing CPU/allocation profiling, Criterion, fuzzing, mutation, coverage or modeled concurrency remain selected tools for a specific question; none substitutes for an independent scientific oracle or becomes a default gate. |

If an existing view answers the question, retain the smaller mechanism. Before any
optional capture ships, give it an exact original CLI retrieval, budget, stop and
release under [71a](71a-cli-inspection-and-evidence-lifecycle.md#a03). A diagnostic
failure cannot change the original verdict. Validate typed output, an unavailable
or omitted attachment, and release; do not run a profiling or benchmark campaign
merely to exercise the tool catalog.
