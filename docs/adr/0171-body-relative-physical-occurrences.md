---
id: ADR-0171
title: Own physical occurrences by immutable expression body inventories
status: proposed
date: 2026-10-09
deciders: [paul-heyse]
level: decision
principles: [AP-01, AP-03, AP-06, DP-09, DP-19, DP-22, PS-01, PS-09]
blueprint: [§5.3, §14.2, §14.3]
review: docs/design_review/reviews/design_review_body-relative-physical-occurrences-target_2026-10-09.md
evidence: Proposed
supersedes: []
superseded-by: null
revisit: A new expression-bearing grammar cannot resolve checked and lowered operations through the same complete body inventory.
verification: Fresh binder and synthetic-span controls; forced prehash collisions; specialization and strict portable reconstruction; selected ownership and linear preparation extent controls.
standard: {core: '3.4', process-simulator: '1.5'}
scenarios: [docs/design_review/reviews/design_review_efficiency-principles-codebase_2026-10-09.md#s02]
---

# ADR-0171: Own physical occurrences by immutable expression body inventories

## Context

Occurrence construction currently renders every subtree twice, computes a subtree hash
that is then replaced by the root hash, and retains every rendered subtree. Field binding
independently discovers subtree membership through more rendering and hashing. Removing
only those strings would leave repeated preparation and make hash equality an authority.
The maintainer selected compact body-relative ownership and direct derived-state rebuilding.

## Scope

Change checked physical occurrence ownership and its portable interpretation, within the
existing checked-AST and physical-admission authorities. Preserve authored declaration
identity, source attribution, checked dependency discovery and the compiler's distinct
diagnostic occurrence sequence. Plan 33 EFF03 owns implementation and acceptance.

## Drivers

Preparation and retained occurrence metadata should follow the size of the actual syntax.
Repeated syntax in different lexical/binder contexts must remain distinct even with empty
spans. Exact physical admission and portable replay must never be inferred from a prehash.

## Options

Retaining subtree text keeps amplified work/storage. Dropping text while retaining hash-only
membership is unsound. Global AST interning would introduce unnecessary lifetime and sharing
authority. Select immutable owner-local body inventories and compact body/node coordinates;
existing owning ASTs retain syntax once. Selected and portable products own their necessary
syntax, without a global interning obligation.

## Outcome

Each checked field/function owns a deterministic inventory of its expression roots. An
expression occurrence is a body slot and node preorder position; finite-reduction synthesis
has a distinct tagged occurrence under its function owner. The slot resolves to retained
source-bearing syntax. Positions use the existing `Expr::walk` order, including let body
before binding values and reduction domain indices/filter before body. Source ranges are
read from those nodes; empty ranges never merge occurrences.

One enumeration helper supplies checking, specialization, field binding, lowering and portable
restoration. A declaration's outer `(declaration, role, repeated-field position)` key is
unchanged. Within each field, expression roots are enumerated by the actual grammar: direct
expression; predicate comparison lhs/rhs, atom/membership expression, membership-domain
path indices and recursive Boolean
branches; equation sides and conditional branches; logic/cardinality expressions; static
collection/application/comprehension domains, filter and body in existing grammar order.
Function roots comprise body, external output, validity, envelopes, authored applicability,
specialized applicability-use predicates and inputs, and all other checked expression fields
in their existing vector/field order. The helper must
cover every consumed grammar variant and refuse invalid coordinates; it is not a second
scientific declaration registry. Specialization rebuilds the inventory after rewriting.

Temporary address maps may connect a traversal to node coordinates, but addresses never
escape into identities or wires. Where structurally equal bodies must be associated across
snapshots, prepare structural indexes once bottom-up with a dedicated frame. A digest is
only a prefilter: full span-independent structural equality and exact owner paths resolve
candidate matches. Preserve variants, ordered edges, binder names/domains, path segments
and indices, predicates, exact numeric meaning and unit products. A forced digest collision
cannot merge bodies or authorize membership. No rendering or mathematics normalization is
used as structural membership authority. Render only a selected subtree for diagnostics.

Portable records carry compact coordinates against the complete retained owner inventory,
not repeated subtree text or standalone hash membership. Restore checks duplicate, absent,
out-of-range and conflicting coordinates and preserves complete physical receipts without
fresh readmission. Selected packages copy the reached owner's inventory and admissions
together; a slot cannot refer to syntax discarded by selection. The checked AST remains
the source for positive, negative, membership and provenance dependencies.

### Consequences

FunctionRecord becomes version 2; admitted recipes use `pse.admitted-body.v2`. Runtime
admitted-body product envelopes and request identity move to v4 so incompatible candidates
are separated before replay. Add a dedicated structural-body-index frame; preserve historical
frames and `MathLocalOccurrenceV3`, which also governs a distinct compiler diagnostic identity.
Rebuild affected disposable internal products directly, with no old decoder or migration.
Authored/external inputs and unrelated live state remain protected.

### Compensating controls

Fresh repeated `(x/x)` under different physical binders with cleared spans, nested lets,
reductions/filters/indexed paths/guards, identical separate fields, forced digest collisions,
rewritten specialization and empty reductions establish distinctions independently. Strict
portable controls exercise complete receipts and malformed coordinates; whitespace edits
preserve mathematical meaning and update attribution. Selected closure retains reached
syntax/dependencies only. Preparation/retained-extent controls follow actual AST size;
they do not claim an unmeasured latency improvement.

### Confirmation

The scoped target review accepts this design at Proposed strength. No implementation, product acceptance
or performance measurement follows from this decision text. Plan 33 owns those states.

## Pros and cons

Compact ownership removes repeated subtree rendering/storage while preserving exact physical
meaning. Shared inventory enumeration and explicit wire versioning require coordinated changes
across checking, specialization, lowering and reconstruction.

## More information

- [Plan 33 EFF03](../plans/33-efficiency-principles-remediation.md#eff03).
- [Review F05](../design_review/reviews/design_review_efficiency-principles-codebase_2026-10-09.md#f05).

## Status history

- 2026-10-09: Proposed under maintainer-confirmed RC04; scoped target review Accept at Proposed strength; implementation remains outstanding.
