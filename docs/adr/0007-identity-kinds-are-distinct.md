---
id: ADR-0007
title: Adopt D4: semantic IDs, artifact ordinals and content hashes are three different things
status: accepted
date: 2026-09-13
deciders: [paul-heyse]
level: decision
principles: [DM-11, DM-12, DM-15]
blueprint: [§D4, §5.1, §5.2]
review: not-required: revision 2 resolved the identity finding (F1); revision 3 records no further finding against D4
evidence: Proposed
supersedes: []
superseded-by: null
revisit: A relation needs an identity that is neither a semantic ID, an ordinal nor a content hash
verification: `tests/governance/blake3_owner.rs`; the canonicalization property tests in `tests/conformance`

---

# ADR-0007: Adopt D4: semantic IDs, artifact ordinals and content hashes are three different things

## Context

Blueprint D4 separates stable 128-bit semantic IDs, artifact-local ordinals and blake3 content hashes, and forbids names, row positions, dictionary codes and solver positions from ever being identity.

## Scope

Binds the three identity kinds and their uses. The derived-ID formulas and `derive_key` contexts are ADR-0023's subject.

## Drivers

A rename must not change an identity; a re-batched relation must hash identically; a solver position is an artifact of ordering, not a fact.

## Options

Name-based identity — rejected by F1: a rename would then be a delete plus an insert across every referencing row. Row position as identity — rejected: it is not stable under re-batching.

## Outcome

Authored identity is assigned at creation and stored in the document; `qualified_name` is an attribute a `rename` change op may alter. Ordinals index compiled artifacts. Content hashes identify immutable artifact versions.

### Consequences

Every reference is by identity; retained target texts (`pse.target_path`, `pse.expr_dsl`) are serializations that P2 re-checks against the identity rows (`case.target_text_mismatch`).

### Compensating controls

`rename` is a first-class change op (§22.2); the §22.4 diff report shows one rename row rather than a scatter of deletes and inserts.

### Confirmation

Canonicalization property tests cover re-batching, metadata re-insertion order and dictionary re-encoding (§24.1).

## Pros and cons

Three identity kinds is more machinery than one; the review's F1 shows what one kind costs.

## More information

Blueprint §D4, §5.1 (three forms of identity), §5.2 (revisions and runs); ADR-0023.

## Status history

- 2026-09-13 — accepted with the repository-seeding pull request (backfilled from blueprint revision 3).
