---
title: "Plan70b: Fixtures and process ownership"
status: in-progress
date: 2026-11-08
adrs: []
doc_role: plan
doc_owner: docs/plans/70-test-assurance-architecture.md
doc_topics: [design, validation, preparation]
review_sources:
  - docs/design_review/reviews/design_review_test-assurance-target_2026-11-08.md
  - docs/plan_improvement_reviews/plan70-repository-test-efficiency-2026-11-08.md
---

# Fixtures and process ownership

**Proposed.** Realize [R02](../design_review/reviews/design_review_test-assurance-target_2026-11-08.md#r02)
and the process portion of R04. Keep effect-free assertions with small inputs, logical database
borrowers on the real client contract, and private physical services where the claim depends on
process, wire, permissions or persistence. Keep Plan67's demonstrated service evidence and
the selected Plan68 v4 fixture and Plan69 custody evidence at their named scopes.
[Coordinator](70-test-assurance-architecture.md) · [v4 fixture receiving](68d-surrealdb-v4-developer-and-test-receiving.md)

### <a id="b01"></a>B01 — Select bounded capture without changing drain semantics

Keep `SupervisedChild` as the process owner. Its rustix 1.1.5 `process` primitives
(`pidfd_open`, `pidfd_send_signal`, and `waitid` with `NOWAIT`) support identity-bound signalling
and observing exit before reaping. PSE checks the process group and settles its observed members.
Exit of the leader alone cannot establish group drain. Existing `StderrCapture` is prepaid,
stderr-only, and restricted to `DrainFailurePolicy::Park`.
[Pinned PIDFD API](https://docs.rs/rustix/1.1.5/rustix/process/fn.pidfd_open.html) ·
[Pinned wait API](https://docs.rs/rustix/1.1.5/rustix/process/fn.waitid.html) ·
[Supervisor](../../crates/pse-diagnostics/src/supervised_child.rs)

Compare two candidates for finite capture. Use rustix 1.1.5 `event::poll` over owned nonblocking
pipes and a cancellation wakeup, or use two supervised bounded readers with a demonstrated
way to interrupt and join each reader when a writer keeps a pipe open. Choose the smallest
composition that admits bounded resources before spawn, drains both streams without pipe deadlock, preserves
an overflow fact, and supports finite refusal without detaching a reader or releasing a live
group. Prefer the option that adds no separate process owner. Resolve a material uncertainty
with a child that exceeds both bounds then exits zero and a descendant that holds a pipe open.
The deciding record must show bounded retained bytes, explicit overflow, and authentic join or
unresolved custody. If neither realization meets the contract, leave bounded-capture consumers
gated and record the specific constraint. Keep legacy Park behavior for its current callers.
[Pinned poll API](https://docs.rs/rustix/1.1.5/rustix/event/fn.poll.html)

### <a id="b02"></a>B02 — Deliver the common process and settlement contract

Use B01's selected owner for bounded diagnostic capture. Separate process exit, overflow,
assertion result and settlement. Reserve a finite settlement window within the original outer
deadline, derive the work cutoff once and clamp startup, RPC, child waits and cancellation to
their owned budget. Keep physical drain authoritative and preserve the first failure and
settlement refusal separately. An unresolved drain retains original custody. For SurrealKit and
the repeated-analysis target, verify that the effective nextest profile cannot silently cut
off legitimate reserved work and drain. Nextest's `slow-timeout` with `terminate-after` can
terminate the harness before its internal settlement deadline. It supplies the runner cutoff.
The fixture owner still budgets work and retains cleanup authority. Correct the selected lane or
budget relationship. Require owner drain before disposal. [D03](70d-scientific-references.md#d03)
and [C03](70c-test-support-and-contract-evidence.md#c03) consume this contract for reference
diagnostics and committed lost-acknowledgement evidence respectively.
[Service run](../../crates/pse-operations/src/native_service.rs) ·
[Runner profiles](../../.config/nextest.toml) ·
[Nextest timeout configuration](https://nexte.st/docs/features/slow-tests/)

### <a id="b03"></a>B03 — Make logical borrower cleanup representative

Run borrowed protection through the ordinary production execution/connection owner with an
explicit retained lifetime. Give the test fixture ownership of the private database and its
teardown. Use the normal result-release join and owned connection drain before removal. Prove
caller drop, sibling borrowing, and refusal on undrained state before removing the
`FixtureLifetime::cleanup_executor` substitution and migrated fixture-only API consumers.
Share the existing `NativeServiceSpec` construction over authenticated prepared inputs across
target-owned cases. Keep explicit variations for real actor, transport, reopen, and topology
controls. Move pure refusals out of unnecessary service preparation in concert with C01.

Give managed retention the original target-owned settlement witness rather than inferring it
from assertion success. Failed assertions with confirmed drain can release their eligible
captures. Unknown drain retains them. Preserve original paired result, holds and expiry.
The already corrected unpublished-listing release is separate. [Canonical protection](../../crates/pse-operations/src/canonical.rs) ·
[Logical fixture](../../crates/pse-operations/src/native_fixture/logical.rs) ·
[Managed release](../../crates/pse-host/src/managed/workflow.rs)

### <a id="b04"></a>B04 — Observe the phase and reason the control claims

Use SurrealKit `1.0.0-rc.1` at `625e264c4e134def842bfe8ac9d2f918a42bb6c3` on the selected
SurrealDB 4.0.0 fixture. `test --schema-from sync --suite <name>` supplies selected declarative database
assertions. The `both` mode builds sync and rollout schemas separately and supplies schema-parity
evidence. It does not by itself establish behavior on a predecessor row. `--json-out` supplies
named suite results when a report is produced. PSE owns service identity and credentials,
diagnostics before report publication, deadline, physical drain and disposal.
[Pinned tester](https://github.com/surrealdb/surrealkit/blob/625e264c4e134def842bfe8ac9d2f918a42bb6c3/crates/surrealkit/src/tester/runner.rs) ·
[Selected fixture and receipt](68d-surrealdb-v4-developer-and-test-receiving.md#packages)

The current v4 control uses distinct setup and seed `THROW` markers, exit 1 without a report,
and a 64 KiB stderr readback limit. Preserve these discriminators and the separate assertion and
rollout drift controls. Move live diagnostics to B02's bounded process contract because a
readback cap leaves the producer file free to grow. Verify that a startup or endpoint refusal
cannot satisfy the intended-marker check. Replace elapsed-time
inference in lease and park cases with a phase/lease or original process
observation where one exists. Challenge a wrong PID or missing witness. Drive the actual
preparation owner through its lock and a controlled cancellable build stage, checking
unchanged publication pointer and successful retry. Keep the synthetic flock check scoped to
flock. Require positive nextest restoration evidence, receiving A02's helper selection.
Use a private coordinator journal for policy tests and a named real manager/cgroup lane for
actual placement, drain and SIGKILL recovery. Prove another owner's state remains untouched.
[SurrealKit control](../../crates/pse-operations/tests/native_service/surrealkit.rs) ·
[Preparation controls](../../crates/pse-host/tests/native.rs) · [Foreground controls](../../crates/pse-host/tests/foreground.rs)

For [I12](../plan_improvement_reviews/plan70-repository-test-efficiency-2026-11-08.md#i12),
keep the full 19-case positive suite. Qualify a smaller seed-mutation run only if the pinned
SurrealKit CLI can select `authored_seed_is_visible` and a stable positive control and report
exactly those identities and original outcomes. Confirm the selection behavior at the pinned
CLI and runner with the expected assertion failure and a substituted or missing case.
Retain the full negative run if exact selection cannot be shown. Keep the distinct setup
and seed refusals, FIFO interruption and drain, and both rollout comparisons on the same
private-service ownership path. Bound report readback before JSON decode. Distinguish absent,
malformed, oversized and partial reports, and authenticate the selected suite, cases and
original exit. B02 owns bounded live diagnostics. A report readback bound alone leaves
producer output free to grow (H5, H19, H26).
[Pinned CLI source tree](https://github.com/surrealdb/surrealkit/tree/625e264c4e134def842bfe8ac9d2f918a42bb6c3) ·
[Authored suite](../../scripts/tests/fixtures/surrealkit/canonical.toml)

### <a id="b05"></a>B05 — Separate worker ordering controls from real deadlines

For [I09](../plan_improvement_reviews/plan70-repository-test-efficiency-2026-11-08.md#i09),
use the existing controlled scientific child and owner acknowledgements for the ordering
controls. Gate release on the observed dispatch or cleanup phase and preserve the original
cancellation, child reap, I/O drain and admission-release distinctions. Replace polling or
unconditional waits only where that phase is directly observable. A synthetic clock can
exercise pure deadline calculations. The current real-clock deadline control keeps its active
native dispatch and actual reap evidence until a shorter witness demonstrates sufficient
startup headroom under supported ordinary load. Compare wrong phase, missing dispatch and
wrong settlement against the expected refusal and successful drain. Use B02's original
deadline and settlement meanings for the real-clock case (AP-06, DP-23, H21, H24).
[Worker proxy controls](../../crates/pse-runtime/src/scientific_worker/proxy_tests.rs) ·
[Original deadline and drain](#b02)

## Packages

| ID | Status | Prerequisites | Completion |
|---|---|---|---|
| [B01](#b01) | complete | Existing process owner and pinned platform primitives | Selected capture mechanism with bounded-stream, overflow and original drain evidence. |
| [B02](#b02) | in-progress | B01 for stream capture, actual service and runner deadline contracts | Shared work/capture and settlement contract, targeted success, overflow and interrupted-drain controls, and selected profile budget correspondence. |
| [B03](#b03) | complete | Existing client and connection owner, B02 for new settlement receiving | Real protection cleanup and target-owned release evidence. Remove replaced fixture seams and duplicated constructors with their consumers. |
| [B04](#b04) | in-progress | B02 for bounded seed and interruption cases, A02 for marked runner cases | Existing wrong-reason, cancellation, restore, private-policy and real-manager controls. SurrealKit preserves exact case and drain evidence, records the negative-selection decision and bounds report admission. |
| [B05](#b05) | in-progress | Existing scientific worker controls, B02 settlement contract for physical receiving | Phase-controlled ordering and failure witnesses, with a real-clock deadline witness that observes active work, reap and release. |

## Execution checkpoint

**Implemented and tested.** B01 selected nonblocking pipes and `poll` inside `SupervisedChild`
in `e5215a46b`. The owner records retained bytes, overflow, pipe completion, child exit and
group reap separately. The default managed `pse-diagnostics --lib` supervisor selection passed
14 controls with zero failures, including both-stream overflow after exit zero, an inherited
writer, and unresolved pipe completion after group reap.

**Implemented.** B02 and B03 landed in `ca1eaa011`. The native service separates work and
settlement cutoffs. Target-owned capture release consumes the original attempt's drain witness.
Borrowed protection cleanup now follows the production connection executor, replacing the
fixture cleanup executor substitution. Shared service construction serves the migrated cases.
`pse check pse-operations --all-targets -F pse-operations/native-service` passed with zero
failures after the native integration target's borrow-check errors were corrected. The default
native-service selections passed the work-cutoff reserve, real panic and drain, caller-drop and
sibling protection controls. The deliberately failing assertion reached its named failure after
physical drain. Its original report remained available and all five eligible captured executables
were released. The focused host release checks refused incomplete original settlement evidence.
`e2d00ba59` replaced an unopened-schema inventory query with fresh non-creating catalog reads.
The three logical controls passed with zero failures and complete settlement in
`01a1bbb0-da65-70b2-9efe-d9c866f7c387`.
[Exact host settlement rerun](70a-selection-and-execution.md#execution-checkpoint)

**Implemented with bounded evidence.** B04 uses the bounded child owner and validates report
extent before decode. The seed-mutation route selects the `seed-mutation` tag and checks the
two exact identities `authored_seed_is_visible` and `decimal_u64_exact`. The 19-case positive
route remains. Four selected `pse-host:foreground` controls passed under the default Nextest
profile with zero failures. They cover literal argument and environment transfer, ordinary
drain, and controller SIGKILL recovery before and after exec. The resumed default Nextest
`pse-operations:native_service` selection passed four cases with zero failures in
`01a1bbad-a153-765c-a96b-ffb03e7699ac`. It included the live SurrealKit seed selection,
wrong-reason, rollout and interruption checks with complete settlement. Both selected
`pse-host:native` preparation cancellation controls passed in `01a1bbb1-da0a-743d-89f6-ddac69a47ab9`.
Repeated-analysis timeout and drain evidence remains open under B02.

**Implemented.** B05 replaces ordering polls with dispatch, close and reap observations in
`c8efe19ab`. It retains the real-clock deadline control. The CoolProp-enabled build finished,
but the workflow detected a changed `Cargo.lock` and refused execution. Rerun the selected
phase, cancellation and real-deadline controls on stable inputs, including original reap and
admission release.
