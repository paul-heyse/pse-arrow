---
title: "Plan68d: Developer and test receiving on SurrealDB v4"
status: in-progress
date: 2026-11-07
doc_role: plan
doc_owner: docs/plans/68-surrealdb-v4-cutover.md
doc_topics: [current-work, deployment, storage, performance, validation]
review_sources: [docs/functional_reviews/surrealdb-v4-cutover-2026-11-07.md]
---

# Developer and test receiving on SurrealDB v4

## Target and baseline

Receive the completed Plan67 native fixture, working evidence, compiler and
retirement routes on the single v4 generation. Start from the actual completed
Plan67 commands, manifests and effective compiler graph. The authoring baseline
already includes a native SurrealKit fixture, and the working-tree candidate
extends it with rollout and drift cases. Reconcile these with the final Plan67
checkpoint before choosing the regression population. Preserve each named
assertion and its proof boundary. The review's persistent history premise has
been superseded by Plan67's regeneration decision. [Coordinator](68-surrealdb-v4-cutover.md) ·
[O03](../functional_reviews/surrealdb-v4-cutover-2026-11-07.md#O03) ·
[O04](../functional_reviews/surrealdb-v4-cutover-2026-11-07.md#O04) ·
[Plan67 decisions](67-storage-lifecycle-and-build-reuse.md#decisions-and-boundaries)

### D01 — Prepare one immutable v4 suite tool

Use SurrealKit `v1.0.0-rc.1` at
`625e264c4e134def842bfe8ac9d2f918a42bb6c3` as the exact candidate. Its upstream
lock resolves SDK and types 4.0.0. The final Plan67 tool owner must still
select PSE's explicit remote gRPC SDK feature closure and immutable source and
lock receipt. Update the exact source integration and its lock patch from the
new tag. Test patch preconditions, record the original and selected lock
identities, and verify the executable and its receipt before fixture launch.
Reuse the tool owner's locked preparation and fail closed on a stale or
mismatched recipe. Keep the suite SDK in the tool's build closure, separate
from canonical runtime transport. Complete the replacement after the selected
v4 tool passes D02 and release the superseded preparation through 67e's
generation owner.
[RC.1 source](https://github.com/surrealdb/surrealkit/tree/v1.0.0-rc.1) ·
[RC.1 locked SDK](https://github.com/surrealdb/surrealkit/blob/v1.0.0-rc.1/Cargo.lock) ·
[SDK 4.0.0 feature closure](https://docs.rs/crate/surrealdb/4.0.0/features) ·
[Current tool owner](../../crates/pse-host/src/surrealkit.rs)

### D02 — Receive database contracts at the owned endpoint

Reuse the native service's exact endpoint identity and temporary schema
staging. Run `surrealkit test --schema-from sync` for the canonical
generated definition set and `--schema-from both` for the selected
authored rollout chain. Verify the exact RC CLI flags and treat a tooling
difference as a receiving requirement. Compare normalized installed definitions
with the generator's intended result. Keep the original `--json-out`
report inside the invocation custody and verify the named case population
alongside the real exit outcome. Authored schema, suite and seeds are the
regeneration inputs. Consume them through the existing fixture rather than
adding a second server or a report database.

Separate expected database refusal, assertion failure, generated artifact
drift, seed/setup failure, missing or malformed report, interrupted run and
failed physical disposal. Exercise a foreign endpoint, actor separation,
setup and seed interruption, and exact service settlement. Verify suite
database absence when identities are available, or disposal of the whole
owned endpoint after drain. Database tests cover semantics and authorized
non-effects. Rust and independent scientific tests retain their respective
physical transport, native process, numerical and reference assertions.
[Native fixture and report consumer](../../crates/pse-operations/tests/native_service/surrealkit.rs) ·
[SurrealKit test semantics](https://surrealdb.com/docs/manage/schema-migration/testing) ·
[Rollout semantics](https://surrealdb.com/docs/manage/schema-migration/rollouts)

RC.1 adds support for v4 named field migrations and explicit rollout
readiness. Treat a named transform, its await step and rebuilding a derived
index as separate states if the selected rollout actually consumes them.
Fresh generated bootstrap is the receiving target. Persistent row conversion
and migration rollback need a separate maintenance consumer and a selected
design before they can be inferred from suite parity.
[RC.1 release](https://github.com/surrealdb/surrealkit/releases/tag/v1.0.0-rc.1) ·
[SurrealDB field migration rules](https://surrealdb.com/docs/reference/query-language/statements/define/field)

### D03 — Verify current evidence and build reuse

Use the completed Plan67 owner for fresh evidence. Start one selected
test against the current admitted inventory. Exercise a failed original
while its working report is still available, then resolve and rerun exactly
that failed selection under the current listing. Verify the new invocation
identity, changed/missing current test refusal and the distinction between
expired working material and a new regenerated outcome. Receive the actual
finite evidence/knowledge query on the scoped v4 store, using explicitly
selected original observations and their existing interpreter. Readback
must preserve omitted and unresolved coverage. The native owner retains
verdict, fixture admission and physical release. Use [68c](68c-surrealdb-v4-query-improvements.md)
for query work on this finite population.
[Plan66f receiving contract](66f-developer-operations-clients-and-evidence.md#f03--finish-exact-original-failed-reruns) ·
[Plan67b evidence and retirement](67b-invocation-custody-and-developer-history.md#fresh-evidence-and-retirement) ·
[ADR-0252](../adr/0252-ephemeral-developer-evidence-relations.md)

Before moving the SDK and fixtures, retain the matched Plan67 v3 effective
unit, compile/cache and footprint baseline. Compare the same selected
targets, native features, profiles, test populations and warm/cold conditions
after cutover. Count required units and distinguish eligible cache
restoration from incremental reuse. Repeat suite and evidence regeneration
through actual closeout. Check for added retained executable and physical
database copies after settlement and for explicitly named owners of any
remaining allocation. A supplier-version build miss is distinct from
ongoing avoidable unit fragmentation. Credit a benefit only where comparable
measurements establish it.
[Build comparison owner](67d-cargo-variants-and-sccache-reuse.md) ·
[Physical release owner](67e-resource-retirement-and-storage-verification.md)

### D04 — Preserve the history disposition

The review's compound `(hold, expires_unix_ms)` B-tree probe is useful
conditional evidence. The persistent pilot, expiry jobs, maintained history
summary, durable suite ingestion and separate history engine remain deferred.
Reopen through a new decision only for a concrete recurring consumer that
cannot use regenerated or explicitly selected evidence. A future owner must
prove real-role held-prefix selection, exact retirement and receipt settlement,
durability, and physical reuse. O03's current receiving work is D03.
[O03 probe and conditions](../functional_reviews/surrealdb-v4-cutover-2026-11-07.md#O03) ·
[Index contracts](https://surrealdb.com/docs/reference/query-language/statements/define/indexes)

## Packages

| ID | Status | Prerequisites | Completion |
| --- | --- | --- | --- |
| D01 | In progress | Completed 67c/d preparation owner, A exact release tuple | Exact source-built tool and locked gRPC SDK receipt select on v4, with stale and foreign selections refused. |
| D02 | In progress | D01 and B's usable generated declarations and activation witness | Native fixture exercises sync and rollout parity, named actor and database outcomes, report/refusal distinctions, and actual disposal. |
| D03 | In progress | Completed Plan67, v3 baseline before cutover, D02 and the scoped v4 evidence route | Current working-report rerun, fresh evidence regeneration, exact native settlement and matched build/retention observations reach E. |
| D04 | Deferred | Named persistent consumer and separate retention decision | Reassess the recorded v4 held-prefix evidence only when the owner and lifetime justify persistent history. |

## Private candidate checkpoint — November 7

**Tested, standalone tool.** The native `pse diagnostic-tools install
surrealkit` built the exact RC.1 source with the locked remote gRPC closure
in a private tool root. The build reported zero errors and warnings against
the zero target. `verify` accepted its upstream and patched lock identities,
repeat installation reused the same executable, and stale and foreign
selections were refused. The durable tool owner selects the same RC.1 binary
for the live rehearsal.

**Implemented, D02 fixture.** The selected suite has separate authored setup
and seed failures and interruption controls. The force-validated native
`pse check pse-operations:native_service --target-kind test -F
pse-operations/native-service` completed with zero failures. A live B
definition witness requires the canonical maintenance owner and Study
extension. The suite's logical test database supplies its own sync and
rollout parity observations. The assembled receiving run still needs the
B witness on its canonical catalog.

The standalone RC.1 sync on the exact persistent v4 server passed all 19
production-schema cases. The first managed full suite timed out in a later
sync case. The retry observed it specifically in seed refusal after the
full generated schema was installed again. Setup and seed probes now use
a small authored schema, and the RC.1 `rollout plan` owner froze its v4
rollout input. The two full generated-schema cases still exercise sync,
and the separate `both` cases compare the pinned rollout and intentional
drift. The full managed suite is being rerun with those inputs.

**Tested, v3 baseline.** The scoped native `scripts/pse test pse-host --lib`
selection `rerun_controlled_failure_probe` produced one intentional failure.
The original report was inspected and exported using `pse inspect
developer-evidence --scope report`. The exact `pse test --rerun-failed`
selection ran under a new invocation and finished with zero failures against
the zero target. Both invocations settled completely.

**Interface-checked.** Matched `pse inspect dependency-units arrow-array xtask
--all-targets -F xtask/evidence-tools,xtask/host-contracts` runs examined 810
units on main and 844 on the v4 candidate. Each had one force-validated
`arrow-array` unit. Matched cache and allocation observations remain open.

**Implemented.** The scoped evidence fixture can consume an explicitly
exported original report selected with `PSE_PLAN68_ORIGINAL_REPORT` and the
existing `host-contracts` feature. The projection includes the original
invocation outcome, exact digest and provenance. Coverage stays partial
because individual test rows and dependency observations are omitted. The
live `pse test xtask --lib -F xtask/evidence-tools,xtask/host-contracts`
scoped-role and cursor-fence control passed on the private v4 store with
zero failures and complete settlement using the current exported original.

**Tested, offline projection.** With the actual v3 original report selected,
`pse test xtask --lib -F xtask/evidence-tools,xtask/host-contracts --ignored
only -E 'test(=development::evidence::tests::evidence_project_original_failed_report)'`
passed one test with zero failures against the zero target and complete
settlement.
