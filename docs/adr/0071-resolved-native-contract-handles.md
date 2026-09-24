---
id: ADR-0071
title: Admit resolved native contracts through owner-bound handles
status: proposed
date: 2026-09-18
deciders: [paul-heyse]
level: decision
principles: [DM-01, DM-02, DM-07, DM-24, DM-56, DM-59]
blueprint: [§4.1, §4.2, §4.3, §4.6]
review: docs/design_review/reviews/design_review_contract-foundations_2026-09-18.md
evidence: Tested
supersedes: []
superseded-by: null
revisit: A new semantic metadata role or generated expectation family is introduced.
verification: just unit-contract-foundations (16 tests, force-validate); just py-unit python/pse/tests/test_transfer_contracts.py (31 tests); zero-failure baseline.
---

# ADR-0071: Admit resolved native contracts through owner-bound handles

## Context

Plan 10 R1 replaces repeated serialized declaration comparison. Matching native storage
or copied fingerprints cannot establish transitive enum, extension and reference meaning.

## Scope

Plan 10 N01/N02 and necessary N00 preparation. This refines blueprint §4.1–§4.3 and
§4.6; canonical hash formats, native validation compilation and publication stay in
later packages. The named follow-up design change is `design: resolved native contract
admission`, with a blueprint revision row and markers at these sections; this proposed
record does not silently amend the protected blueprint or accept itself.

## Drivers

Preserve complete admission while sharing native fields and resolved definitions once
per immutable registry. Separate semantic equivalence from observed metadata retention.

## Options

- Repeated declaration strings: duplicates complete values and retains per-borrow text work.
- Fingerprints or Arrow compatibility alone: cannot prove complete semantic equivalence.
- Native descriptor graph and sealed owner-bound handles: selected; exact comparison
  precedes reuse, with finite cyclic traversal and registry-lifetime proof storage.

## Outcome

Generate a frozen native expected graph independently of the runtime registry. Compare
all reachable contracts exactly before binding. Keep native fields, domain definitions,
checks and policies; exclude only explicit prose documentation and derived fingerprint
witnesses from execution equivalence. Unknown metadata remains semantic. Retain full
physical metadata on native batches and report transfer loss independently.

### Consequences

Generated adapters and checked batches retain handles rather than declaration text.
Raw buffers still need physical and local-value admission. A handle establishes neither
relational obligations nor publication validity. Foreign handles require exact rebinding.

### Compensating controls

Private handle construction, complete reference resolution, adversarial copied-digest
units, cyclic comparison units and a generated-adapter test. No global foreign-owner cache.

### Confirmation

The execution inventory records exact unit commands and zero-baseline results. Integration
and performance acceptance remain behind Plan 10 N17/N18.

## Pros and cons

Native graph sharing removes repeated semantic serialization. It introduces bounded
resolution and proof bookkeeping whose ownership must be tested independently of timing.

## More information

[Plan 10](../plans/10-native-contract-consolidation.md), D01 and N01/N02.
The scoped review evaluates the corrected target rather than changing the earlier Revise verdict.

## Status history

- 2026-09-18 — proposed before implementation; maintainer acceptance remains pending.
- 2026-09-18 — implemented; 16 isolated Rust units with force-validate and 31 Python
  transfer units passed. No integrated or performance acceptance claimed.
