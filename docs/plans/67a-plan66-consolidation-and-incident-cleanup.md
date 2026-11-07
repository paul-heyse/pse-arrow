---
title: "Plan67a: Plan66 consolidation and incident cleanup"
status: in-progress
date: 2026-11-06
doc_role: plan
doc_owner: docs/plans/67-storage-lifecycle-and-build-reuse.md
doc_topics: [current-work, storage, environment, validation]
review_sources: [docs/functional_reviews/storage-lifecycle-and-build-reuse-2026-11-06.md]
---

# Plan66 consolidation and incident cleanup

## Scope and baseline

Integrate the relevant work in the three frozen Plan66 worktrees and retire each checkout
with its owned storage. Trim the incident population to named evidence. This is a bounded
operator action under current ownership rules. [67e](67e-resource-retirement-and-storage-verification.md)
owns repeatable retirement, and [67b](67b-invocation-custody-and-developer-history.md) owns
the durable history design. Neither is a prerequisite for the incident cutover.

**Interface-checked:** on main `c2126df45`, all three worktrees had authored edits. The two
science and assessment branches were four commits behind main and had no branch-only commits.
The contracts branch was 41 commits behind and 11 ahead, with one non-merge change remaining
in the read-only cherry-equivalence view. Branch history, patch equivalence and dirty files
provide different evidence. Re-read them immediately before integration.

| Checkout | Initial receiving scope |
|---|---|
| `/home/paul/pse-arrow-p64-science`, `p66-wet-water` | Staged CoolProp vendor files, unstaged provider, build, configuration and tool edits, and untracked supplier and generator inputs. Reconcile with Plan66g and its preparation owner. Review any change to proposed ADR-0238 independently from accepting the code. |
| `/home/paul/pse-arrow-p65-contracts`, `p66-c03-method-departure` | Compare `d0bd02aca` and its generated outputs with the fitting-assessment foundation already received on main. Receive remaining meaningful differences and the dirty scientific-worker handler change through their Plan66 result, fitting and worker owners. |
| `/home/paul/pse-arrow-p66-assessment-partials`, `p66-assessment-partials` | Review dirty compiler conformance, modeling specialization, and runtime assessment, dynamics and fitting edits against their current Plan66 equivalents. |

Use the fixed incident window **2026-11-05 23:27:37 EST through 2026-11-06 23:27:37 EST**.
The planning census used the mtime of invocation metadata as a discovery proxy and found
283 recent records, 277 with a result file, and 12 IDs directly mentioned by Plan66. Confirm
native creation identity and selected coverage from original records. This window defines the
incident scope for surviving and unresolved checkout roots. Whole-checkout retirement can
also release older holdings when their own scope and ownership settle. Use the earlier
655 GiB census as a size baseline and the refreshed exact preview as the release set.
[Measured incident baseline](../functional_reviews/storage-lifecycle-and-build-reuse-2026-11-06.md#what-survived-the-clean)

## A01 — Reconcile and receive Plan66 changes

For each checkout record the canonical root, Git common directory and checkout ID before
changing registration. Inspect staged, unstaged and untracked paths, branch-only history,
and any user-selected environment or fixture state with a current consumer. Use Git
`status --porcelain`, branch comparison and path-specific diffs. Compare committed patches
and uncommitted changes with current main and the responsible Plan66 contracts. Give each
meaningful item one disposition: already received with a source pointer, integrated and
checked, superseded with an explanation, or preserved separately for a named owner. Keep
authored work recoverable in a named ref or a verified bounded export when it belongs
outside the Plan66 integration. Keep private configuration out of commits and receipts.

Receive coherent changes on current main with explicit path or hunk selection. Cherry-pick
a branch-only commit only if it is independently applicable and represents the remaining
work. Reconcile conflicting implementations at their current semantic owners. Carry actual
dependency pins and necessary lockfile changes together. Regenerate changed derived output
from its source owner. Preserve unrelated main working-tree changes and the three branch
refs. Check current CoolProp preparation and Water properties, fitting assessment across
failure and native worker settlement, and affected modeling and runtime assessment cases
as applicable to the received changes. Use the owning Plan66 package's supported
`scripts/pse check` and selected test targets with force validation where required. Record
the receiving commit or scope and observed outcome at the Plan66 owner. No additional
plan-wide qualification is implied by receipt.

Record representative available build and state observations D can use before losing the
selected checkout targets. Preserve compact timing and compiled-unit evidence, not the build
trees themselves. Establish the main native host explicitly if a receiving check requires
it. Use the repository's normal admission, toolchain and provider selection.
[Plan66 coordinator](66-integrated-simulation-target.md) ·
[Worktree and tool preparation](../dev/agent-environment.md#worktrees)

## A02 — Preview release and preserve named evidence

Produce one bounded preview with the fixed time window, checkout and native invocation
identities, allocated bytes, source ownership, observed status, known references or holds,
disposition, and unresolved conditions. Inventory invocation ledgers, retired fixture
runtime generations, scratch, and owner-released named stores in the incident population.
Check both linked worktree targets and their separately stored state. Recognize paths by
the inspected root and native identity. An unmatched checkout ID remains a separate
reconciliation case until its former ownership and current absence are established.

Match exact invocation IDs and evidence references in current Plan66 and other active
consumers, including specific retained Plan64/65 reconciliation obligations and explicit
operator holds. Ask the relevant owner to identify any additional live dependency from
running work or current acceptance. Keep only the selected result slice needed for each
named use. Preserve original outcome and coverage, relevant provenance and selection, and
the original failed identities and inventory commitments where rerun requires them. Where
the current consumer requires the original native report or a selected raw member, use the
existing verified export and preserve that actual input. Record its exact use, custodian,
bound and release condition. An absent or changed member stays absent or changed in the
export. A projection cannot stand in for an original-format consumer.

If B is available, publish selected structured facts through it. Otherwise use a compact
verified native export to an explicitly selected destination outside the roots being
removed. Use the existing original-file acquisition for a selected raw member. Read back
with the actual intended consumer and verify original identity, outcome, selected bytes
and omissions. For a rerun candidate, confirm the original inventory and result identities
remain usable after the payload is removed. Publish the selected evidence before release.
Unselected, settled invocations produce no historical row or archive. Record their aggregate
disposition in the incident receipt.
[Native report and rerun contract](66f-developer-operations-clients-and-evidence.md#original-evidence-and-exact-rerun)

## A03 — Release exact objects and verify the result

Revalidate the preview immediately before each release. Require the original producer's
terminal and settlement evidence plus current confirmation of process, unit, borrower,
session and fixture state. Read actual service and cgroup membership where relevant. A
`result.json` file or an absent worktree is only a discovery fact. Stop an identified
test-owned service through its existing owner and verify drain and exact identity before
its state is released. Preserve any active, held, unclassified, changed or unresolved
candidate and record the missing premise. Keep selection roots canonical, reject path
substitution and symlinks, and apply exact scoped effects without globs or age-only deletion.

For each integrated worktree, finish evidence transfer and settle all checkout users.
Use ordinary `git worktree remove <path>` once its source is clean and ownership checks
are satisfied. Verify Git deregistration and target removal. Release the matching external
checkout state through its actual owners, using the identity recorded in A01. Handle
interruption between deregistration and state release by reconciling the remaining
identities against the preview. Retry only still-eligible members. Named stores and
selected generations have separate owners and release checks. Leave the configured
canonical data, managed and global compiler caches, and useful main Cargo state out of
the incident release.

For ordinary settled unselected invocations, remove the complete invocation tree after
the owner checks. For selected invocations, release the original tree only after verified
readback and any live original consumer migration. Otherwise retain the minimal original
members required by the unresolved consumer. An interrupted cleanup records the actual
removed and remaining objects and can resume from that evidence.

Confirm the three registered worktrees and their retired roots are absent. Confirm the
remaining named evidence is readable outside those roots, and exercise one selected
failure rerun against the current admitted inventory where a runnable example exists.
Check one changed or missing selection preserves the truthful unavailable outcome.
Re-measure allocated bytes per class and available filesystem bytes using the same
scope and report separately. Write one small receipt with integration references, confirmed
released bytes, unresolved exceptions, and the outstanding owner action for each exception.
The receipt carries the result of this operation and does not grant future cleanup authority.

## Packages

| ID | Status | Prerequisites | Completion |
|---|---|---|---|
| A01 | complete | Refreshed frozen checkout and Plan66 ownership evidence | Recovered source integrated and targeted receiving checked. Checkout identities and source recovery refs retained. |
| A02 | complete | Fixed incident interval and identified native producers and consumers | Exact preview with held, releasable and unresolved scope. Named exports independently read back. The unmatched checkout stays protected. |
| A03 | in-progress | A01 for each checkout and A02 for the selected release population | Three worktrees and owner-released state retired. Before/after measurements and one receipt record actual yield and precise exceptions. Named evidence and rerun checked. |

## Execution checkpoint — November 6

**Interface-checked.** The native creation field yielded 283 incident invocations across six
roots. Of these, 267 reported complete settlement, ten incomplete settlement, and six lacked
terminal and settlement records. The broader cross-check found 27 distinct incident IDs in the
current and reconciliation plan documents across four checkouts. The earlier 12 counted direct
Plan66 mentions in the main checkout only. Two state roots have no current checkout match.

The three dirty source sets have verified recovery refs. The science candidate is
`recovery/plan67-p66-wet-water` at `fe669d8d0`, the contracts edit is at
`refs/plan67/recovery/p65-contracts` at `9529508cb`, and the assessment set is at
`refs/plan67/recovery/p66-assessment-partials` at `8706166ea`.
The 62 changed file blobs of fitting commit `d0bd02aca` match main's `583fc73cb`.
The worker diagnostic bound and partial-assessment receiver passed targeted verification.
The pinned CoolProp candidate passed its native endpoint and bounded-refusal controls on main.
The original wet equipment and actual-child integration scope remains in Plan66.

**Measured.** Two release batches removed 127 unreferenced, independently rechecked invocation
trees. The first removed 24 main-checkout trees with 35.91 GiB of allocated members. The second
removed 103 linked-checkout trees with 166.65 GiB of allocated members. Available bytes on the
backing filesystem measured 362,030,686,208 before the first batch and 572,617,322,496 after the
second. Concurrent operations also used that filesystem. The incident evidence directory under
the project state root contains the exact previews and release results. Both batches retained
named, live and unresolved objects. A third batch released 18 main-checkout borrowed test and
check trees with 13.15 GiB of allocated members. It also checked the bound original report,
absence of the enclosing cgroup, and separate fixture settlement where required. No remaining
metadata referenced the previously released invocation IDs. The three batches released 145
trees and 215.71 GiB in total.

Of 27 named incident invocations, 26 were exported and independently decoded. Twenty carry
selection evidence and six carry report evidence. The exports total 24,607,930 bytes.
One invocation is on an unmatched checkout and was preserved. Three named consumers still
need original members, including the Plan64 JUnit comparison and two Plan65 compiler diagnostic
readers. The seven required raw members have separately verified portable files. The incident
receipt records the original consumers, and the selected raw directory records transfer.

The assessment source is on main in `4f264eab0` and `3a9f93c9a`. The targeted compiler,
modeling, runtime and native fitting checks passed. Main received the prepared science artifact
in `9e26b3215` and the bounded worker diagnostic receiver in `9d80d9836`.

Ordinary Git worktree removal deleted all three checkouts and targets, releasing 300,007,862,272
allocated bytes. External state with missing, unsettled or independently held evidence remains
with its original owner. An independent incident fixture batch released 19 superseded runtime
generations with 29,886,689,280 allocated bytes. Its selected capsule and lane passed native
readback. The combined release recorded 522.95 GiB of allocated data.

An additional bounded release removed 49 executable images and their 49 associations from 14
named test invocations. It bound each image to the original inventory, rehashed it, and rechecked
the original scope, fixture settlement and live file references before release. Six otherwise
eligible invocations lack independent fixture-settlement confirmation and retain their images.
All 26 previously decoded exports still match their verified bytes. Original reports and
inventories for the 14 trimmed invocations still match their commitments. The incident total is
586,092,539,904 allocated bytes (545.84 GiB). Filesystem availability measured separately was
836,589,912,064 bytes at readback. Exact receipts are in the private incident state directory.
