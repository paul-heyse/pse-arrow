---
title: Connection and transport lifetimes
status: draft
date: 2026-10-12
adrs: []
review_sources: [docs/design_review/reviews/design_review_surrealdb-lifecycle-and-integration_2026-10-12.md]
doc_owner: docs/plans/37-surrealdb-lifecycle-and-integration.md
doc_topics: [current-work, design, storage]
doc_retention: while-dependent
---

# Plan 37b — Connection and transport lifetimes

## Contract, baseline and retained integration

This companion develops F04–F06 and confirmed RC04/RC05. [Plan 37](37-surrealdb-lifecycle-and-integration.md) owns status, finding disposition, rule decisions and assembled acceptance. The [connection inquiry](../design_review/evidence/surrealdb-lifecycle-and-integration-2026-10-12/connection-and-transport-lifetimes.md) supplies the reviewed source routes and counterexamples.

**Implemented, source-inspected:** Python NativeRuntime.close clears caches and calls CanonicalStore.disconnect. The latter drains ResultReadDrain then closes ordinary and activation clients. Runtime::from_shared, Operations and Heartbeat retain shared store capabilities. Some active runs keep counted protections and already cause safe close refusal; that does not constitute a complete borrower contract for every dependent/alias.

The production bounded native router already owns finite queues, original request clocks, ordered setup, dispatch uncertainty, byte charges and actual physical closure. Its ReplayCursor tracks awaiting/index/refreshed but no acknowledged-operation deadline. The existing five-second replay wrapper only covers first-send completion. Normal replies/reset explicitly release AdmissionPermit; malformed non-payload terminal removals do not. Exact locked Papaya removal clones retired values sharing permits, so ordinary local drop is insufficient.

**Proposed:** corrections below. No transport tests, probes, services or timing measurements were run during authoring. Retain SDK/server 3.3.0 and the narrowly patched WS path; no codec rewrite or transport replacement is selected. Fresh target data cutover removes legacy compatibility, not current request/drain guarantees.

## B1. Logical release and authoritative shutdown

There are three distinct owners: a logical runtime caller, an admitted activity that still needs transport, and the physical deployment connection. Represent these distinctions at the existing shared canonical owner; do not infer them from Arc strong counts, cached aliases or a source protection that happens to remain live.

**Caller close:** NativeRuntime.close becomes idempotent logical release. It prevents new work through that logical caller and releases its optional references. Already admitted work keeps independent activity ownership through renewal, publication and exact terminal/unknown-effect handoff. It is not cancelled because another caller closed. Closing does not clear a shared global cache as a proxy for ownership or close physical drivers. Explicit cache-clear remains a separate cache operation.

An independently created runtime over the same deployment has independent caller ownership. Aliases of one logical caller share that caller's closed state; cloning a reference does not manufacture another live admission authority. Closed caller methods return the repository's typed closed/unavailable diagnostic rather than reconnecting silently. Existing result/handle operations that already own activities can finish/release their owned work.

**Activity ownership:** bind real activities before admitting work, atomically with the physical owner's open state. Long-lived admitted work keeps a guard through the last required canonical operation, including heartbeat shutdown and publication/settlement; repeated renewals do not register a new long-lived borrower each time. Short operation guards cover other ordinary queries, initialization/activation work and inspection as needed. Protected source/result reads keep their current permission and drain constraints. Cancellation moves an activity toward terminal closure; it does not release the guard before native/transport/settlement responsibilities finish or transfer to an explicit recovery owner.

Map all actual entrypoints, not only the representative NativeRuntime: runtime construction/source admission, durable run/study/fit/horizon operations, Heartbeat, result/progress readers, analyses, direct Operations/CanonicalStore aliases, activation and fixture/admin operations. A plain internal alias is not an activity, but it cannot bypass activity admission to issue work. No background task is allowed to retain an unaccounted capability after its owner releases it.

**Authoritative shutdown:** the deployment/fixture/admin owner invokes an explicit physical-close operation. It atomically refuses with actual live dependents while leaving ordinary admission usable, or, with no dependents, transitions open → closing and excludes new ones before driver teardown. Preserve protected-reader refusal and final physical drain. Do not block indefinitely waiting for work whose cancellation authority belongs elsewhere. The caller can separately cancel/join work it owns, then retry shutdown. Service maintenance must obtain this quiescence contract before closing its listener.

A refusal is not shutdown. A completed physical close makes the owner terminal; reopening constructs/revalidates a selected connection under explicit owner action. Low-level SDK disconnect remains available to authentic transport fixtures/operators but cannot be called by ordinary runtime release. Native worker drain, local driver drain and remote DB effect settlement remain separate outcomes.

D0 must record the Rust/Python boundary decision and required design review before this change. Keep the existing Python close spelling where appropriate but change and regenerate its declared contract/stubs; remove physical-close wiring and any replaced lifecycle mechanism rather than add a compatibility method that retains the old unsafe interpretation. Registry/codegen own generated declarations.

**Proposed revealing controls:** one caller releases while another performs renewal/publication; admitted work finishes after its original caller releases; closed caller refuses new work; atomic last-release/new-admission race; physical close refuses protected readers and durable dependents; refusal leaves their required RPCs working; final activity completion permits real driver close. Exercise failed/cancelled publication and uncertainty transfer, not just successful queries. A local policy/mock peer can establish most cases; Q1 adds actual durable/native/Python wiring.

## B2. Finite acknowledged restoration

Model each restoration under its physical socket/session incarnation as one finite operation: initial ordered setup → awaiting ACKs → ready, or terminal failed/disconnected. The existing finite replay allowance becomes an absolute deadline spanning the entire acknowledged chain, not a timeout around first transmission or a new budget for each ACK. Keep its current five-second allowance initially, declared through one bounded-profile owner; this repairs coverage rather than extending caller clocks or claiming a latency SLA.

Stamp restoration once when it begins; replacement sockets create a new incarnation. Maintenance examines restoration age as well as ordinary requests. Progress through ACKs, token refresh and setup sends never resets the same operation's deadline. Preserve ordered set/unset/auth/selection behavior and the existing bounded refresh retry. Late ACKs from a replaced/failed incarnation cannot mark the new session ready.

Replace the ambiguous send boolean with explicit outcomes: a setup command was sent and awaits its ACK, all required commands were acknowledged, or setup failed. Only acknowledged exhaustion can release deferred application routes. Queue expiry/cancellation retains each caller's original admission/deadline and before-dispatch classification.

For the bounded profile, restoration timeout or setup-send failure becomes terminal for that client/session: fail deferred work truthfully, close new admission and physically drain through the existing lexical router ownership. Require explicit owner reopen rather than silently starting an unbounded sequence of restoration attempts. Ordinary transient transport reconnection can remain where its restoration succeeds; this does not permit application-write replay after uncertain dispatch.

A malformed setup response already poisons replay; preserve that path. One caller's cancellation is not authority to cancel restoration still serving others. A successful ping is not a setup ACK. Never flush application routes because the cursor merely disappeared after failure.

**Proposed revealing controls:** local peer keeps socket/ping traffic alive but withholds a setup ACK; it either acknowledges before the bound or remains stalled. Check ready versus terminal failure, driver drain and refusal of later calls. Force a single setup send failure and assert zero deferred application dispatch. Exercise multi-command ordering, one refresh attempt, late/duplicate ACKs across incarnations, already-expired queued callers and unknown submitted application writes. Use controllable time/peer barriers rather than test-only longer product budgets.

Existing native_sdk_stalled_replay_refuses_cancelled_and_expired_routes establishes caller refusal and explicitly disconnects; extend coverage to the new restoration terminal behavior rather than treat that existing test body as proof already obtained.

## B3. Explicit exceptional admission completion

Every terminal removal owns an explicit idempotent admission-completion action. Normal replies, named malformed non-payload replies, wholly undecodable non-payload replies, local failure and reset must agree on this contract. Correlation removal, caller error delivery, capacity release, payload drain and remote effect outcome are distinct facts.

Complete non-payload admission when the terminal handler relinquishes its relevant local ownership, independently of retired map values or snapshots. Cover cancellation at handler awaits: taking a pending entry then being cancelled must not leave an unreachable permit. A small terminal-completion helper/guard may centralize this action; do not depend on final Drop of shared epoch-retired metadata or replace the map merely to repair this branch.

Preserve the existing stronger payload rule: unreadable responses with a payload owner invalidate the physical socket while correlation/charges remain reachable until both socket halves are destroyed. Do not generalize non-payload release into premature release of request/response bytes. A malformed response cannot become successful DB/scientific completion; uncertain mutations retain exact operation identity and settlement responsibility.

**Proposed revealing controls:** hold a removed/snapshot pending value; inject named malformed and undecodable errors for application and reserved-control requests; assert exact capacity is reusable without forcing map reclamation. Repeat enough times to expose a charged finite slot, not merely delivery of one error. Exercise terminal-handler cancellation and idempotent reset/completion. Re-run owned-payload malformed-frame and physical-disconnect tests to retain their stronger boundary.

## Package sequence, scope and completion

B2/B3 are independently repairable inside the retained integration. B1 needs the public close decision and complete activity mapping first. They may share test peers or terminal helpers, but one passing branch does not settle another. Coordinate shared vendor files instead of letting independent writers overwrite them.

Use compile checks and targeted transport/lifetime unit tests during implementation. Existing local canonical_transport_tests and vendored WS tests are mechanism seams; they need no scientific solver to exercise restoration or admission. Scope-aware routes must retain force-validation wherever their selected closure consumes Arrow. New local tests are planned here, not claimed existing passes.

Q1 owns the actual rebuilt runtime/Python/native publication/cancellation journeys after all functional work. Report command, mode, zero failure baseline, result and source/artifact scope. No performance improvement, universal shutdown safety or numerical qualification follows from a mock-peer pass alone. Keep current finite buffer/admission accounting and service caps; do not independently retune one knob and invalidate the composed profile.

C2 consumes B1's quiescence for exclusive maintenance. A1/A2 activities retain scientific ownership through publication. Neither scientific artifact retirement nor physical connection close can stand in for the other's permission/drain. Existing plans are reconciled by H1 only after Plan 37 acceptance.
