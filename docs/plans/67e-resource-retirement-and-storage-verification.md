---
title: "Plan67e: Resource retirement and storage verification"
status: in-progress
date: 2026-11-06
doc_role: plan
doc_owner: docs/plans/67-storage-lifecycle-and-build-reuse.md
doc_topics: [current-work, design, validation]
review_sources: [docs/functional_reviews/storage-lifecycle-and-build-reuse-2026-11-06.md]
---

# Resource retirement and storage verification

## Scope and baseline

**Proposed.** Give each developer resource an end of life through its existing owner, and show that
repeated settled workflows stop accumulating bytes. This companion receives
[O01](../functional_reviews/storage-lifecycle-and-build-reuse-2026-11-06.md#O01),
[O03](../functional_reviews/storage-lifecycle-and-build-reuse-2026-11-06.md#O03),
[O04](../functional_reviews/storage-lifecycle-and-build-reuse-2026-11-06.md#O04), and
[O06](../functional_reviews/storage-lifecycle-and-build-reuse-2026-11-06.md#O06). It supplies the
recurring path. The coordinator's incident companion can consolidate Plan66 and perform the
one-time trim using existing owner operations while these changes are pending. Plan66c owns
actual process and service settlement. Plan66e/f govern protected result and original evidence
contracts. The history companion supplies compact evidence and holds, and the test and build
companions remove producers of redundant state. [Plan67](67-storage-lifecycle-and-build-reuse.md)

**Interface-checked at `c2126df45`.** `CheckoutState` binds the canonical checkout and Git common
directory, then maps their identity to a project state root beneath the selected user state
directory. Git removal handles the worktree and its target, while externally placed state belongs
to its native owners. The worktree listing can observe inventory and invocation users, including
unknown facts. The original owners establish settlement independently. `RetentionClass::eligible` currently
limits eligibility to disposable presentation and checks terminal outcome, actual settlement,
generation use, and an independent owner's retention. Treat that predicate as an input to
classification. Actual release requires an authenticated resource identity and its owner's
decision under ADR-0168 and ADR-0234. [Checkout and retention
owners](../../crates/pse-host/src/paths.rs) · [Worktree operation](../../crates/pse-host/src/worktree.rs)
· [ADR-0168](../adr/0168-workspace-content-lifecycle.md) ·
[ADR-0234](../adr/0234-managed-invocation-subtree-ownership.md)

## Target and work boundaries

### E01 — Read-only storage census

Start with a bounded, reproducible census for a selected repository, checkout or owner. Compose
existing worktree and native observations to identify state, target, preparation, and live
holdings. Report allocated bytes separately from apparent size, de-duplicate observed hard-linked
inodes, and distinguish actual bytes from estimates. Report observed coverage and the reasons for
protected, potentially releasable, and unresolved holdings. Cross-filesystem holdings get their own
totals. Symlinks, inaccessible directories and missing owner records must appear as scope limits.
Collect a current-filesystem comparison independently of logical record deletion.

Use this inquiry on demand and for package acceptance. Retain the small census approach while it
satisfies the operator workflow. Add a native read-only command only if repeated diagnosis proves
an unmet need. Launches continue using existing admission semantics.

### E02 — Worktree identity and retirement

Associate the native worktree creation path with the checkout and repository identities and the
actual target and external state layout. Capture the exact association before removing the worktree.
Use the existing native observations to distinguish registered, moved, missing, and unresolved
roots. A new worktree can reuse a previous path and identity input tuple, so validate the original
registration and state identity at the release boundary. Explicitly refuse unknown ownership or
an unavailable backing mount.

Ship the documented owner workflow first. Reconcile branch and dirty work, preserve required
evidence, close the checkout's admitted sessions and prepare/fixture borrowers, use ordinary
`git worktree remove`, and release the authenticated external resources through their owners.
Each step reports its actual outcome. If Git succeeds and an external release stops partway,
retry with the original bounded selection and recheck ownership. Preserve surviving holds in
their actual owner, and report the exceptional bytes rather than moving an entire state tree.
Legacy and unmatched roots remain protected until positive attribution and settlement establish a
release path.

Assess the R-56 trigger against the actual consolidation and the repeat journey. Recommend a
coordinated native removal operation only if the explicit workflow cannot authenticate and settle
the required owners reliably. A native operation must coordinate actual checkout-wide ownership
and preserve Git's dirty-worktree protection and recovery. Record the decision before work that
depends on a new operation. [Git 2.43 worktree](https://git-scm.com/docs/git-worktree/2.43.0) ·
[Worktree guide](../dev/agent-environment.md#worktrees) · [R-56](../adr/register.md)

<a id="E03"></a>
### E03 — Scratch, stores, holds and recovery

At creation, the producer declares purpose, owner, identity, and closeout for recreatable
session scratch, named developer stores, and campaign state. Use the existing checkout session
scratch owner where it fits. Classify probe inputs, diagnostic environments, compiler crash
output, and ad hoc work directories by their actual producer and use. Authored inputs remain
protected. For a closed campaign or plan, publish selected structured evidence through the
history companion before releasing its source. Keep exceptional raw evidence only when its
independent owner can identify the requirement, exact member, bound and release condition.
Release a test database through its native service or fixture owner with authenticated stop,
reader drain, and actual disposal. The canonical owner handles protected stores and their
recovery contracts. [Canonical service](../../crates/pse-operations/src/native_service.rs) ·
[Canonical storage and recovery](../dev/surreal-substrate.md)

Preview describes the candidate, expected bytes, actual guard facts, and refusal reason. Release
rechecks identity, reference/hold state, and settlement under the resource owner's synchronization
before removing any members. An interrupted export, uncertain acknowledgement, concurrent hold,
or incomplete drain retains the original resources required to resolve it. An interrupted release
has a retry route with already removed members distinguished from new identities. Prefer explicit
age, count, or byte policies for proven recreatable classes, beginning with large completed
scratch and failed staging. Report whether a class meets its policy and what still protects its
remaining bytes. Producer-owned private scratch may end with the producer's actual settlement.
Published generations and original evidence use explicit owner release. A broader automatic
policy requires a separate decision before implementation.

### E04 — Prepared tool and deployment generations

For native host tools, provider preparation, Surreal server and study modules, and deployed
workers, trace actual selected generations, prepared configuration, runtime borrowers,
protected references and explicit rollback requirements. Separate failed staging and disposable
intermediates from sealed inputs, current selection, and any retained rollback. Preview and release
proven unreferenced material within each owner. A previous generation survives only when a
consumer has a documented rollback route. Keep independently justified warm construction state
under its own bound and report unknown historical roots separately.

The Surreal server owner publishes a self-contained selected generation. Legacy releases can
still name a binary in `source/target` and the import path needs that binary and the source
lockfile. Prove independent selection and readiness, then migrate and settle exact direct-path
borrowers before releasing those intermediates. Study module receipts and native provider
generations retain their own compatibility and provenance checks. For a named store, attribute
deployed executable bytes to its worker-generation owner and database bytes to its actual
database owner. This preserves a viable rollback and recovery path across cleanup.
[O06's measured examples](../functional_reviews/storage-lifecycle-and-build-reuse-2026-11-06.md#O06)
include a 2.3 GiB legacy Surreal source directory dominated by build intermediates and a
named store with 13.95 GiB of worker executables alongside a roughly 128 MiB database.
[Surreal source and import](../../scripts/surrealdb-source/source-build.md) ·
[Server selection](../../crates/pse-host/src/server.rs) ·
[Study module selection](../../crates/pse-host/src/study_module.rs) ·
[Native preparation](../dev/build-performance.md#native-preparations)

### E05 — Service retirement

Keep native transient fixture launch through `start_service_bound` and
`CollectMode=inactive-or-failed`. Systemd collection releases the unit object under its
collection policy. The native fixture owner separately authenticates service and descendant
settlement before disposing state. Reconcile legacy Surreal and compiler-cache fragments with
their loaded manager identity, process/cgroup facts and selection records. Remove only proven
released fragments through the actual owner. Optional telemetry has its own persistent
configuration and authenticated stop/release path. The review's census found 554 Surreal,
eight compiler-cache, and four telemetry fragments. The expected payoff is fewer obsolete
service records and clearer teardown. Authenticate attribution beyond a name prefix or
inactive unit state. Preserve refusal and repeatable recovery if a fragment was
replaced between preview and release. [Native manager](../../crates/pse-host/src/manager.rs) ·
[Telemetry owner](../../crates/pse-host/src/telemetry_service.rs) ·
[systemd unit collection](https://www.freedesktop.org/software/systemd/man/latest/systemd.unit.html#CollectMode=)

## Packages

| ID | Status | Prerequisites | Completion |
|---|---|---|---|
| E01 | complete | Read access to selected state and owners | Reproducible read-only census with allocation and ownership coverage. Supplies the baseline for E06. |
| E02 | in-progress | E01 ownership view and existing Git and native settlement | Recorded checkout association, ordinary Git removal plus explicit state release, and the R-56 disposition. |
| E03 | planned | Existing resource ownership and named hold contract from history companion | Purpose-bound scratch, stores and campaigns release through their original owners. Verify selected evidence before release. |
| E04 | in-progress | E03 release and hold semantics, existing selection/readiness receipts | Owner-specific retirement of superseded generations and legacy intermediates, with usable live and rollback selections. |
| E05 | in-progress | E03 release semantics and exact current unit identity | Native fixture teardown remains truthful and owner-proven legacy fragments can retire. Persistent telemetry follows its own owner. |
| E06 | planned | E01–E05, invocation custody and test/campaign producers migrated | Repeated settled workflows and a complete retirement journey establish structural storage behavior. |

## Acceptance

- Observe one mounted and one unavailable state root, shared inodes, and unresolved ownership.
  The read-only view never creates state, reads through a symlink outside scope, or reports an
  incomplete census as complete.
- Exercise a clean consolidated worktree, a dirty worktree, a live borrower, a held artifact,
  a reused path, and an interruption after Git removal. In the completed case, Git no longer
  lists the worktree and its retired external state is gone, with a verified record of any
  separately protected evidence.
- Exercise open readers, new holds between preview and release, interrupted export, failed
  staging, and retry. Verify the selected server/module and its needed recovery material
  after releasing a retired generation. Show that unit disappearance alone cannot release a
  fixture and that a verified obsolete fragment can retire without affecting current services.
- Repeat ordinary tests and fixture preparation, settle a real compatibility campaign, and
  remove a consolidated worktree. Compare allocated bytes by class and the actual produced
  items. Require zero additional retained executable or physical database copies for repeat
  ordinary tests. Attribute selected compact results, useful build state, authored data, and
  protected science separately. Count observed retained bytes and physical reclamation
  independently of logical deletion and record unresolved exceptions by owner.

## Execution checkpoint — November 7

The incident census combines the native worktree observation and one physical allocation scan
of the actual checkout targets and state roots. It observed four registered worktrees on the
backing filesystem. The linked targets held 276.84 GiB, the main invocation ledger held
196.83 GiB, and main fixture captures held 42.46 GiB at this post-release observation. The
native view disclosed unresolved historical identities. Those rows do not establish either
active use or release eligibility. The exact per-root observations and separately measured
filesystem capacity are in the incident evidence directory named by [67a](67a-plan66-consolidation-and-incident-cleanup.md).
The [operator census](../dev/agent-environment.md#inspecting-allocated-storage) uses one physical
scan per filesystem. A temporary two-root control counted one shared inode once and preserved
an unavailable root as a partial observation with its diagnostic.

Ordinary Git removal retired the integrated assessment checkout. The recorded external
association remains usable for its two drained invocations with missing terminal records.
Keep the native removal command deferred under R-56 until an actual case needs coordination
that ordinary Git and the resource owner cannot perform.

The service preview found 516 legacy Surreal and five compiler-cache definition candidates
and separated them from selected, rollback, referenced, and unresolved definitions. One
disabled, unloaded Surreal definition passed the exact pre-release checks and was removed.
The independent post-release observation found the fragment absent and telemetry definitions
and manager identities consistent with the preview. The remaining candidates are pending their
individual owner rechecks.

The incident runtime fixture release verified 19 original completed preparations, each with
two exact captured executables and a capsule matching the current selection's digest.
The release removed 29,886,689,280 allocated bytes. The native reader independently verified
the current capsule and native lane afterward. Five older preparations and four generations
without a complete original preparation were held. The remaining retirement classes still
need their own owner paths.
