---
id: ADR-0226
title: Reuse source-owned physical identity types at generated boundaries
status: proposed
date: 2026-10-25
deciders: [paul-heyse]
level: decision
principles: [AP-01, AP-02, AP-04, DP-01, DP-04, DP-14, PS-01]
blueprint: [§4.1, §5.1, §8.1, §21.1]
review: docs/design_review/reviews/design_review_source-owned-physical-identities_2026-10-25.md
evidence: Interface-checked
supersedes: []
superseded-by: null
revisit: A source-owned identity needs a different storage base or its generated boundary cannot retain the physical domain without a second Rust wrapper.
verification: Plan56a A03 identity alias/schema/codec tests and physical boundary controls; the scoped design review assesses one native type per physical meaning and preserved identity framing.
standard: core-3.4/process-simulator-1.5
scenarios: [docs/plans/56-design-principle-alignment.md#s03]
---

# ADR-0226: Reuse source-owned physical identity types at generated boundaries

## Context

ADR-0115 generates a Rust wrapper for every declared identity, while `pse-quantity`
already owns physical identity wrappers. Declaring quantity, unit, basis and reference domains
without reusing those wrappers would create two Rust types for one physical meaning.
The operator selected reuse of the physical wrappers during Plan56a execution preparation.

## Scope

This supplements ADR-0115's generated-identity mechanism for source-owned identities and
amends blueprint §4.1, §5.1, §8.1 and §21.1. It preserves registry-owned scientific vocabularies,
identity bytes, framing, Arrow transport and Rust-owned/generated Python documents. Plan56
RC01–RC06 remain separate pending proposals; this does not adopt owner-local non-scientific enums.

## Drivers

A scientific input must distinguish a quantity type from its unit, basis and reference datum.
A physical-owner change should update its projection without a second native domain type.
Plan56 scenario S03 requires readable original meaning across Rust, Python and protected results.

## Options

| Option | Assessment |
|---|---|
| Generate independent physical wrappers in `pse-model` | Two types for the same meaning and repeated conversion/admission seams; rejected |
| Move physical ownership into the model projection | Reverses the existing dependency direction; rejected |
| Declare a source-owned identity and generate an alias to its native owner | Reuses established ownership with a small generator extension; selected |

## Outcome

Identity declarations may name an existing source-owned Rust type. Assembly validates declared
source/base consistency, and generation re-exports that type instead of creating another wrapper.
The source-owned type retains the domain-specific JSON Schema identity facet. Generated model
traits, Arrow codecs, Python aliases, PyO3 role adapters and source relationships consume that same
declaration. Unsourced identities retain ordinary generation.

Source paths describe implementation ownership, like sourced enumeration paths; they are not
scientific identity bytes or a new persisted physical interpretation. Physical domain carriers
remain explicit declarations, and correct domain membership is admitted by the physical owner.
Python NewType annotations are not runtime proofs of membership.

### Consequences

Physical binding and reader contracts use distinct quantity/unit/basis/reference identities,
including kind/subject domains where exposed. Shared schema/codegen publication must include
all affected consumers. The existing physical registry and conversion plan own meaning and math.

### Compensating controls

Alias/base validation, schema identity-facet assertions, unchanged byte/frame vectors and actual
Rust/Python/Arrow boundary tests guard the extension. Missing original result closure is explicit;
a source alias never authorizes using the latest context to interpret historical values.

### Confirmation

Interface inspection at `6d83bfc43` establishes the existing physical wrappers, generated identity
emitter and PyO3 role adapter. This record does not claim executed product qualification. Plan56a
owns implementation and test outcomes; the scoped review owns architectural judgment.

## Pros and cons

One native type remains authoritative and existing bytes stay exact. Generation must now handle
source-owned aliases consistently through every projection, rather than only its Rust type output.

## More information

- [ADR-0115](0115-registry-typed-identities-and-vocabularies.md)
- [Plan56a](../plans/56a-model-meaning-and-physical-interpretation.md)
- [Plan56 rule decisions](../plans/56-design-principle-alignment.md#rule-changes)
- [DA-005](../design_review/audit/semantic-model-and-authority.md#da-005)

## Status history

- 2026-10-25: proposed following the operator's explicit source-owned identity selection;
  formal ADR status remains separate from implementation authorization and evidence.
