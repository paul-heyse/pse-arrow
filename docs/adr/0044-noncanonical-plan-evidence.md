---
id: ADR-0044
title: Record plan encodings as noncanonical diagnostic evidence
status: accepted
date: 2026-09-13
deciders: [paul-heyse]
level: decision
principles: [DM-15, DM-31, DM-48, DM-59]
blueprint: [§4.3, §6.13, §14.2, §20.2]
review: docs/design_review/reviews/design_review_blueprint-rev5-contracts_2026-09-13.md#6-acceptance-gates
evidence: Proposed
supersedes: [ADR-0019]
superseded-by: null
revisit: A consumer demonstrates a need for canonical plan bytes, or upstream supplies a versioned byte-stability contract worth adopting.
verification: just adr-lint; just docs; docs/plans/02-blueprint-revision-5-contracts.md Verification; bash docs/design_review/evidence/blueprint-rev4-2026-09-13/run.sh
---

# ADR-0044: Record plan encodings as noncanonical diagnostic evidence

## Context

E5 produces different protobuf encodings after sorted metadata insertion. The existing semantic cache key already makes plan byte identity unnecessary for correctness.

## Scope

Amends plan evidence and metadata determinism claims. Semantic stage keys remain ADR-0041 authority.

## Drivers

Retain useful plan attribution without turning diagnostic serialization order into semantic identity. Do not maintain a canonical protobuf contract without a consumer that requires it.

## Options

Recursively canonicalize all protobuf maps: possible but adds an encoding contract without a current consumer. Keep the false equality test: rejected. Preserve diagnostic bytes and explicitly bound their meaning: selected.

## Outcome

Retain protobuf and EXPLAIN as noncanonical diagnostic artifacts with physical checksums, engine/codec identifiers and ordered rule observations. Do not require byte equality for equivalent plans, use their checksums as relation identity, or claim sorted HashMap insertion canonicalizes encoding.

### Consequences

Equivalent runs can carry different plan-evidence checksums without a cache miss or a reproduction failure. Reproduction compares declared semantic outputs and recorded configuration.

### Compensating controls

The implementation gates are explicit in [plan 02](../plans/02-blueprint-revision-5-contracts.md). Document acceptance does not establish runtime behavior. Deferred optimizations remain disabled until their register trigger and conformance evidence are satisfied.

### Confirmation

E5 in the retained review runner establishes the distinction. Plan 02 requires plan-evidence round-trip/attribution checks and verifies that plan bytes never enter semantic memo inputs. `just adr-lint` and `just docs` check documentation structure only; the plan distinguishes these checks from future behavioral acceptance.

## Pros and cons

Existing codecs and EXPLAIN remain useful evidence with checksum integrity. Equivalent plans can have different bytes; consumers compare decoded meaning/attribution and cannot use the diagnostic checksum as a semantic key.

## More information

See the governed blueprint sections, the [revision-4 review](../design_review/reviews/design_review_blueprint-rev4-library-contracts_2026-09-13.md), its [reproducible library evidence](../design_review/evidence/blueprint-rev4-2026-09-13/README.md), the [revision-5 contract review](../design_review/reviews/design_review_blueprint-rev5-contracts_2026-09-13.md), and [plan 02](../plans/02-blueprint-revision-5-contracts.md).

## Status history

- 2026-09-13 — proposed before the revision-5 blueprint amendment; user requested reconciliation of all review findings and library opportunities.
- 2026-09-13 — proposes superseding ADR-0019 for the corrected contract above; accepted arguments remain immutable.
- 2026-09-13 — accepted (revision-5 review, Accept for bounded proposed scope; evidence remains Proposed until the wave-1 implementation review).
