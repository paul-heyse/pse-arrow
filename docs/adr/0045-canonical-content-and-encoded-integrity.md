---
id: ADR-0045
title: Separate canonical logical content from encoded artifact integrity
status: accepted
date: 2026-09-13
deciders: [paul-heyse]
level: decision
principles: [DM-11, DM-12, DM-14, DM-15, DM-42, DM-48]
blueprint: [§5.1, §5.3, §20.1, §20.2, §20.4, §20.5]
review: docs/design_review/reviews/design_review_blueprint-rev5-contracts_2026-09-13.md#6-acceptance-gates
evidence: Proposed
supersedes: [ADR-0023]
superseded-by: null
revisit: Any canonical constant or normalization rule changes, or a measured workload exceeds the supported relation-size envelope.
verification: just adr-lint; just docs; docs/plans/02-blueprint-revision-5-contracts.md Verification; bash docs/design_review/evidence/blueprint-rev4-2026-09-13/run.sh
---

# ADR-0045: Separate canonical logical content from encoded artifact integrity

## Context

R4-03 and R4-04 show that stream content identity conflicts with IPC-file/Parquet byte checks and that ignored values under nulls affect the proposed canonical bytes.

## Scope

Amends hashing and artifact publication as pse.canon.v2 and a versioned manifest. Semantic-ID derivation contexts and ADR-0030 valid-float rules remain unchanged.

## Drivers

Make equal declared relations retain one identity despite ignored storage bytes or alternate physical encodings. Publication must distinguish a complete verified object from a corrupt object with a plausible name.

## Options

Use one digest for logical and encoded bytes: rejected by E4. Canonicalize Parquet writer bytes as model identity: rejected. Separate logical identity from physical integrity: selected.

## Outcome

Compute logical relation identity from a framed, versioned canonical stream after recursive normalization of hidden physical content. Store each physical encoding under its own byte checksum and bind logical identity, encoding, size and checksum in the manifest. Snapshot membership is explicit and excludes self-referential revision/ref/pass bookkeeping. Validate existing objects before reuse and publish the ref only after complete artifacts and manifest exist.

### Consequences

The canonicalization change requires v2 even though the model implementation is skeletal. Unknown/v1 artifacts are refused until an explicit importer decodes, validates and rehashes them; byte copying is not migration. Phase 1 has an explicit bounded canonical relation size.

### Compensating controls

The implementation gates are explicit in [plan 02](../plans/02-blueprint-revision-5-contracts.md). Document acceptance does not establish runtime behavior. Deferred optimizations remain disabled until their register trigger and conformance evidence are satisfied.

### Confirmation

E4 is the counterexample. Plan 02 names canonical_null_equivalence, encoding_roundtrip_identity and publication fault fixtures. Both checksum and logical verification are required on untrusted artifact admission. `just adr-lint` and `just docs` check documentation structure only; the plan distinguishes these checks from future behavioral acceptance.

## Pros and cons

Separate logical and encoded identities support IPC/Parquet interchange and explicit integrity checks. Recursive normalization and restore verification have real memory/CPU cost; the bounded envelope makes that cost a stated implementation obligation.

## More information

See the governed blueprint sections, the [revision-4 review](../design_review/reviews/design_review_blueprint-rev4-library-contracts_2026-09-13.md), its [reproducible library evidence](../design_review/evidence/blueprint-rev4-2026-09-13/README.md), the [revision-5 contract review](../design_review/reviews/design_review_blueprint-rev5-contracts_2026-09-13.md), and [plan 02](../plans/02-blueprint-revision-5-contracts.md).

## Status history

- 2026-09-13 — proposed before the revision-5 blueprint amendment; user requested reconciliation of all review findings and library opportunities.
- 2026-09-13 — proposes superseding ADR-0023 for the corrected contract above; accepted arguments remain immutable.
- 2026-09-13 — accepted (revision-5 review, Accept for bounded proposed scope; evidence remains Proposed until the wave-1 implementation review).
