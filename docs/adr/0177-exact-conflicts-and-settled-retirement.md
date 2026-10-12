---
id: ADR-0177
title: Exact conflict domains and settled retirement
status: proposed
date: 2026-10-11
deciders: [paul-heyse]
level: decision
principles: [AP-01, AP-03, AP-07, DP-08, DP-18]
blueprint: [§14.3, §20.4, §20.6]
review: docs/design_review/reviews/design_review_plan36-canonical-preparation_2026-10-11.md
evidence: Proposed
supersedes: []
superseded-by: null
revisit: The selected backend no longer registers exact absent-key conflicts, or an unregistered creator can change a scanned retention/cancellation predicate.
verification: Emitted-query races in both commit orders for absent/root/pin creation, active-root release versus unrelated creation and source collection, renewal/reclaim, result/analysis creation/retirement, cancellation/effect and grant settlement; uncertain receipt recovery under original clocks.
---

# ADR-0177: Exact conflict domains and settled retirement

## Context

Plan 36 RC06/RC12 replace broad problem/study writes with exact effect domains and permit residue collection only after settlement. SurrealDB 3.3 RocksDB snapshot `FOR UPDATE` registers exact keys, including absence, for commit validation; it supplies neither blocking exclusion nor predicate-phantom protection. The complete protocol must cover creation, not merely renewal.

## Scope

P-B1/P-B7 refine canonical conflict and retention contracts within the existing store. P-B2 adds derived readiness discovery under the same scientific transition owners. No production history deletion or cutover is authorized. Original scientific authority, source identity, original clocks, uncertain effects, lifecycle lineage and backend support limits remain.

## Drivers

Independent result effects need independent write domains. Retention must observe late roots, readers and analysis borrowers in either commit order. Local failure or a deadline is not remote settlement.

## Options

Dropping shared touches alone loses creation and cancellation safety. Predicate reads alone cannot establish absence. Longer leases cannot close creation phantoms. Select paired creator/reclaimer fences plus exact registered authorities.

## Outcome

Source creators register/check reclaim authority and reclaimed intervals plus their exact root/pin key including absence; they create the root/pin and write a creator fence. Collection registers that creator fence before scanning roots/pins, registers affected exact authorities, and atomically publishes reclaimed intervals/removes eligible payload and writes reclaim authority. Creator-first invalidates the stale collector; collector-first invalidates the stale creator, whose rebuilt decision refuses reclaimed history. Stage/product activation, source revision creation and analysis source roots participate. Pure removal of an owned active-attempt root registers reclaim authority and the exact root, but does not write the creator fence: it adds no retention dependency, and the run root continues protecting the same source. Collection observing the removed root conservatively retains; collection-first invalidates stale release through reclaim authority. Existing pin renewal registers the exact pin and reclaim authority, verifies original identity/live expiry, then writes only that pin. Released/expired pins cannot revive.

Result pin creation registers exact retirement authority including absence, installs its exact pin and writes a result-creator fence. Retirement registers the creator fence before scanning readers and study/analysis obligations, then writes monotone retirement authority. Analysis creation and input-edge installation participate. Every summary-study header and study-point/run membership installation also registers/checks the affected run's retirement authority and writes its result-creator fence atomically with association creation. Rebuilding after retirement refuses the new association. Current idempotent `begin_run` does not establish immutable association at run registration. An alternative immutable-at-registration implementation is valid only if the complete summary/point association is installed there and every later creator verifies it; missing association cannot be assumed permanent. Renewal checks exact live pin, manifest and retirement authority. Source roots held by runs retain their separate lifetime.

Continued source reads register/check their exact live pin plus reclaim authority; continued result reads register/check their exact live pin, selected manifest and retirement authority. They do not bump a common lifecycle counter. Revocation, renewal and collection preserve their registered predicates through completion; old admitted bytes alone do not authorize a later independent read.

Cancellation alone writes exact cancellation authority. Claim/start/append/heartbeat/observe/close/seal/finalize register it before their decision. Checked immutable problem and point-to-study association travel with capabilities; server effects still verify identity/generation. An absent association is registered at its exact key or made immutable by registration. Summary runs participate. Valid effects committed before cancellation remain in the retained prefix; cancellation-first invalidates/refuses late effects. Cancelled complete export preserves original assessment under its cancelled terminal outcome; incomplete export grants no usable science or seed.

Local turns follow actual written domains with stable ordering across owners; they never span hydration, native work or retry backoff. Definite typed conflicts rebuild the whole decision after releasing the turn, under the original clock. Lost response, timeout or cancellation after dispatch settles the original operation receipt/fence; it never blindly replays the application mutation or refreshes its identity/expiry.

Study readiness is derived discovery, never admission. Exact per-point hints retain pending-predecessor counts, complete observed transition revisions and eligibility derived by the existing pure policy. Eligibility includes an early unusable-predecessor refusal even when another dependency is still pending, and unresolved seed resolution remains discoverable. Scientific append/claim/start/settle/observe transitions atomically invalidate or queue their exact hint; bounded outgoing dependency pages propagate changes without an unbounded fan-out transaction. A hint repair registers the complete immediate premise set before saving the derived decision. Cancellation bypasses hints; claim always reruns policy against current authoritative premises. Bounded persisted discovery/rebuild cursors are derived coordination only and are not touched by ordinary science transitions. Restart/recovery reconstructs hints from actual points and immutable edges without rerunning science. Missing or stale hints cannot permanently hide work; cursor settlement and concurrent invalidation preserve a reachable repair obligation.

Protection grants retain immutable original deadline, nonce and authority incarnation. Acquisition registers the exact grant/tombstone key including absence. Settlement registers that key and stable authority, proves the original window closed and advances a monotone closed-through floor atomically before residue collection. Earlier acquisition either committed and is observed/drained or fails validation; later acquisition refuses the closed grant. Never renew the same closed grant, clear its floor under a restored incarnation, or infer eligibility from age. Run retirement similarly fences every late writer/replay before bounded resumable append-receipt cleanup. Keep run lineage and unresolved uncertainty until their borrowers settle.

### Consequences

Migrate declarations, generated functions and every writer/reader together. Replace broad touches only after emitted-query races establish the registered premises. Shared source creation remains a real effect domain, amortized by preparation engagements. No new general scheduler or store is introduced.

### Compensating controls

Snapshot registration semantics are backend/release-specific. Exact fixture races test both orders; range/predicate observations require creator participation. Cancellation joins native drain, and transport drain never proves remote settlement. Cleanup remains owner-driven and restricted to authorized fixtures until a separate production operation is approved.

### Confirmation

Proposed target grounded in the completed coordination investigation and accepted independent bounded contract review. Actual emitted-query controls and P-G1 assembled qualification remain open. ADR status follows the decision-PR route.

## Pros and cons

Exact domains permit independent effects while retaining serial cancellation and reclamation safety. Every creator must participate; missing a writer is a correctness defect, not an optimization fallback.

## More information

Plan 36b P-B1/P-B7 own the complete consumer inventory, deletion and race controls. ADR-0172's settled analysis authority is a pattern, not evidence that source/result grant settlement already exists.

## Status history

- 2026-10-11 — proposed during maintainer-authorized execution of accepted Plan 36 RC06/RC12.
