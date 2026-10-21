---
id: ADR-0210
title: Hydrate the standard registry from declaration-derived resolved products
status: proposed
date: 2026-10-20
deciders: [paul-heyse]
level: decision
principles: [AP-03, AP-07, DP-10, DP-19]
blueprint: [§4.2, §4.6]
review: docs/design_review/reviews/design_review_execution-performance-recommendations-audit_2026-10-20.md
evidence: Tested
supersedes: []
superseded-by: null
revisit: A standard catalog meaning cannot be reproduced exactly by the generated product, or complete hydration introduces material startup/retention regression.
verification: Compare independently source-assembled and generated-hydrated standard registries across declarations, obligations, resolved graphs, exact schemas, canonical descriptions and fingerprints, self-description batches and generated expectations; retain changed/custom builder and receiving-validation controls.
---

# ADR-0210: Hydrate the standard registry from declaration-derived resolved products

## Context

The standard catalog deterministically resolves declarations, canonicalizes SQL, derives
contracts and indexes, materializes self-description and hashes their complete meaning at
cold process initialization. Runtime memoization removes repetition within one owner but
cannot remove that cold derivation. A prior projection-only serialization probe did not
reconstruct a complete Registry and therefore did not qualify either hydration or a speedup.

## Scope

Narrow blueprint §4.2 and §4.6 within Plan48 B04/I04. Generate complete resolved products
for the standard catalog only, from the same source declarations and existing generator.
Custom/extended registries retain their actual declaration-owned builder. No fingerprint
framing, semantic-ID contract, array admission or receiving eligibility changes.

## Drivers

Move deterministic cold derivation to generation while preserving one declaration authority,
exact complete meaning and independent mutable runtime owners. The removal of repeated work
supplies the selection rationale; a marginal timing result is not a prerequisite to adoption.

## Options

Runtime memoization alone preserves repeated cold construction. A zero-copy/archive label
alone supplies neither the complete native owner nor its lifetimes. A declaration-derived
resolved product avoids derivation while reconstructing the actual runtime shells needed by
current consumers; this is the selected bounded route.

## Outcome

The generator independently assembles the source catalog and emits standard resolved
relations, obligations, indexes, contract graph, exact native schemas, semantic descriptions
and closures, self-description batches and unchanged identities. Hydration allocates current
maps, Arc ownership, synchronization and empty runtime implementation/proof caches. It does
not deserialize prior sessions, implementation closures or receiving proofs. Only the existing
standard process-registry initializer consumes this product; source generation and every
custom builder continue to use declarations directly, preventing self-referential generation.

Generated Rust literals retain declaration-owned static strings. Embedded native batch data
uses the existing Arrow representation with explicit receiving/array validation. Unknown
model variants or unsupported image construction fail generation or hydration explicitly;
they cannot silently omit a declaration or substitute an empty product. Generated outputs
are published only by the full generator, never hand-edited or seeded by hand.

### Consequences

The standard registry gets fresh runtime owner identity and caches while avoiding derivation
already performed at generation. Regeneration is required when its declarations change.
General builder support is the authority for changed/custom meanings, not a compatibility shim.

### Compensating controls

Independently compare complete source and hydrated registries, including nested metadata,
dictionary ordering, declaration origins/keys/references, expected closures, exact fingerprints,
materialized rows, graph/schema lookup and current receiving behavior. Check independent
hydrated owners and failure of corrupt/incompatible embedded payloads. Correctness closures
retain explicit force-validation. No generated product admits arbitrary newly supplied arrays.

### Confirmation

**Tested:** the explicitly force-validated default-profile native-local
`builder::products::tests` selection passed all five controls. They compare the actual
generated standard registry with independent source assembly across complete meaning and
native encoding, require fresh runtime owners/current receiving behavior, preserve intrinsic
scalars in empty custom registries, and reject corrupt, missing or extra native batches.
Two generator controls and normal Rust-contract publication passed; subsequent full normal
generation completed all five schema targets and binding stages. Standard initialization
now consumes the generated products; source generation and custom builders retain their
independent declaration route. Exact conditions and results live in the
[B04 package Outcome](../plans/48b-registry-and-receiving-validation.md#outcome-recorded-after-implementation).

**Measured, fixture-bounded:** five fresh source/hydrated owners in one current development
test process had median construction-plus-drop times of 304.272 ms and 30.146 ms. Every
owner weak reference expired on drop. This complete-registry comparison is not whole-process
startup, RSS, allocator-only cost or scientific workload speed evidence; the earlier partial
projection timing supplies none of those claims either. Assembled receiving and final
qualification remain with the plan owners, and this record's formal status remains proposed.

## Pros and cons

The route removes cold derivation without duplicating schema authority or inventing a new
cache/proof platform. It increases generated output and requires complete hydration coverage
and runtime native-data decoding; full generation and equivalence checks own those obligations.

## More information

- [Plan48 B04/I04](../plans/48b-registry-and-receiving-validation.md#b04--decide-whether-runtime-reuse-needs-a-static-or-archived-product)
- [Plan48b Outcome](../plans/48b-registry-and-receiving-validation.md#outcome-recorded-after-implementation)
- [Plan48 coordinator](../plans/48-execution-performance-and-numerical-efficiency.md)

## Status history

- 2026-10-20 — Proposed before dependent production changes under authorized Plan48 execution and the operator's deployment-first direction.
