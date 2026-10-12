---
title: Plan 36 canonical conflict and retained preparation contracts
review_date: 2026-10-11
tier: change
purpose: target
standard: core-3.4 + process-simulator-1.5
baseline: 2d6bf6ae7d03a8a9f6f109696bfe3d9cc8455466
disposition_owner: docs/plans/36b-studies-coordination-and-results.md
---

# Plan 36 canonical conflict and retained preparation contracts

## 1. Scope and decision

**Accept both revised Proposed contracts at design level.** An independent design-reviewer assessed ADR-0177/0178, Plan 36a and Plan 36b P-B1/P-B7 against Core 3.4, Heuristics for Efficient Architecture 1.0, process-simulator 1.5 and the pse-arrow binding. Behavioral adequacy and architectural fitness are satisfied for the bounded target after the study-association correction below. Implementation races, migrated consumers and assembled qualification remain open.

The target covers interactive binding without structural rebuilding, complete delta admission, retained preparation and exact canonical permission/retirement. Variation includes source changes, new readers/borrowers, cancellation, uncertain effects, changed receiving premises and compiler invalidation. Existing decisions describe current constraints and rule impacts; the standard and functional target determine judgment. Production activation, history deletion, optional publication, public hashing changes and sparse-rank replacement are outside scope. The relevant Plan 36 owners retain implementation and acceptance.

## 2–3. Responsibilities and contracts

Immutable source, selection material, receipts and body content own meaning; engagements own current permission, charges, settlement and drain. Numerical overlays own point values and attribution without mutating their parent. Pure compiler queries own dependency-tracked semantics, while native construction and canonical effects remain outside memoization. Receiving qualification owns its actual trust transition; historical receipt meaning is unchanged.

Canonical source and result creation must participate in the corresponding retirement domain. Exact registration, including absence, is distinct from predicate scanning. Creator/reclaimer paired fences protect both serial orders. Existing live-pin renewal and continued reads check exact authority without an incidental common lifecycle write. Cancellation is its own written authority; effects register it before deciding. Local turns cover actual writes and release before hydration, native work or retry backoff. An uncertain dispatched effect settles its original identity and clock.

Typed source intents preserve V4 framing, explicit deletion domains, complete reference/namespace invalidation and physical admission. Complete selection predicates remain available for independent receiving reconstruction. One declared completion and bounded candidate windows may share observations only inside the same protected read. A retained engagement settles promised publication synchronously before success and retains uncertain effects through settlement/drain.

## 4–5. Scenarios and execution fit

**S01: creator versus collection.** Exact absent-key registration alone does not protect a scanned predicate. Source roots/pins, result readers, analysis inputs and study associations all participate in paired creation/retirement fences. Either commit order invalidates the stale opposing decision; rebuilding after retirement refuses newly ineligible work.

**S02: cancellation and uncertainty.** Cancellation-first refuses late effects; effect-first retains its valid prefix. Native join is required separately from remote settlement. Complete cancelled export retains its original assessment under a cancelled terminal outcome; incomplete export supplies neither usable science nor seed. Definite conflicts rebuild after releasing the local turn under the original clock; lost responses do not authorize blind mutation replay.

**S03: value/source changes.** A typed binding retains exact parent/context and numerical attribution, reevaluates value-dependent facts and keeps independent original assessment. A delta is explicit intent, with overlap refusal and atomic CAS publication. Reference changes reextract dependencies; positive and negative namespace observations invalidate completely. Whole-inventory rules retain conservative dependencies where partitioning is unproved.

**S04: retained preparation and receiving.** Stable consumers share complete immutable products and a live engagement, with exact renewal and once-per-engagement synchronous settlement. Expired or released permission cannot revive. Changed or unknown receiving premises trigger complete qualification or refusal. Shared observation cannot turn a candidate into permission. Dirty-checkout capture has its actual outer time and cancellation owner.

**S05: compiler mutation.** A single-writer lineage/context lane stages charged candidate inputs, drains readers before mutation and publishes one admitted product or restores/destroys the candidate. Historical detached products survive. Salsa equality and dependency tracking can stop pure work; they cannot cache native construction or permission.

## 6. Foundations and gates

| Foundation | Verdict and grounding |
|---|---|
| AP-01 | satisfied: semantic content, current engagement, canonical effects and native execution have coherent owners |
| AP-02 | satisfied: exact identity, absence, clock, failure and lifecycle contracts cross boundaries |
| AP-03 | satisfied: engagements compose existing admission, compiler and store mechanisms |
| AP-04 | satisfied: immutable meaning, current permission, uncertainty and scientific assessment remain distinct |
| AP-05 | satisfied after F01 correction: every creator, continued read, renewal and retirement participates |
| AP-06 | satisfied: pure compiler semantics remain separate from native/canonical effects |
| AP-07 | satisfied qualitatively: repeated work is amortized under complete premises; product gains remain unmeasured |

| Gate | Verdict and scope |
|---|---|
| G1/G2 | pass: scoped authority and scientific/source meaning are retained |
| G3/G5 | pass at design level after F01 correction: paired fences, atomic association creation and original uncertainty ownership |
| G4 | pass: explicit effects and pure Salsa boundary |
| G6 | pass: complete premises and conservative fallback remain mandatory |
| G7 | pass: target is Proposed; inspected source and future race/qualification evidence remain distinct |
| G8 | pass: SurrealDB exact transactions and Salsa dependency/equality mechanisms fit; no extra engine |
| G9 | pass: applicable foundations satisfied for this bounded target |
| PS-G1 | pass at preservation boundary: physical conversion/context remains mandatory; numerical controls not run |
| PS-G2 | pass at preservation boundary: roles and structural meaning retained; sparse-rank change excluded |
| PS-G3 | pass: independent original assessment retained; incomplete exports cannot seed |

## 7. Findings

### F01: study-association installation must fence result retirement

**Diagnosis and evidence.** The initial ADR-0177 named result-pin and analysis creators but omitted study-header and point/run membership installation. `CanonicalStore::create_study` can idempotently call `begin_run` and install associations afterwards. Generated study creation/append checks source identity without establishing a permanently fresh, nonterminal run. `forget_run` scans study/point membership. Absence at run registration therefore does not prove permanent absence: retirement may miss a late association.

**Impact.** A narrower protocol omitting these creators can admit an association to retired result state in a concurrent serial order, violating retained study obligations. This is a correctness prerequisite for removing broad conflict writes.

**Corrective contract.** Every summary-study header and point/run membership installation registers/checks the affected run retirement authority and writes its creator fence atomically with the association. Rebuilt decisions refuse retired runs. Immutable-at-registration is an alternative only if the complete association is installed there and every later creator verifies it. Continued result reads also check their exact live pin, selected manifest and retirement authority.

**Preservation and acceptance.** Preserve idempotent identity, lineage, summary/point roles and original operation clocks. Exercise both commit orders for both creator kinds, including existing terminal runs and uncertain retries. The coordinator corrected the Proposed contract before implementation. Plan 36b owns the still-open implementation disposition; this correction does not close F08/F15.

## 8–9. Alternatives and library fit

Removing shared touches without creator participation loses correctness. Predicate-only scanning cannot protect phantoms; longer leases do not close that gap. Paired fences fit the selected backend's exact registration semantics. Full rebuild/requalification remains a conservative fallback where dependency or receiving ownership is incomplete. Permission flags, stat-only identity and optional publication do not satisfy the selected contract. SurrealDB/Salsa remain their existing mechanism owners; no new library API was proposed or exercised by this review.

## 10. Verification and limits

**Interface-checked:** revised ADRs, approved plan contracts, source preparation/receiving seams, canonical registration/read/renewal/retirement and study-association creators. **Proposed:** migrated implementations and their removals. Reviewer tests, benchmarks and mutations: **not_run**; the assessment was read-only. No whole-product, scientific, backend-race or performance qualification follows from this review.

Acceptance requires actual emitted-query races in both orders, complete writer/reader inventories, exact uncertain receipt recovery, typed bound/fresh and delta/full equivalence, physical/reference invalidation, charged engagement/drain, receiving mutation controls and compiler cutoff/fresh-oracle controls. Original reviews and their evidence remain unchanged.

## 11. Rule impacts and disposition

ADR-0177/0178 implement the accepted RC01–RC06/RC12/RC20 choices at blueprint §14.3/§14.4/§20.4/§20.6/§21. No additional public effects or rule choice was introduced. V4 identity, synchronous settlement, complete receiving qualification and independent scientific assessment remain. Plan 36a owns preparation implementation; Plan 36b owns F01 and canonical acceptance. No SHOULD exception is requested.

## 12. Decision and handoff

Behavioral adequacy, architectural fitness and overall revised target: **Accept at design level**. The independent reviewer returned this judgment after examining the study-creator correction; the coordinator published this synthesis. Real consumer migration, races and P-G1 qualification remain required.
