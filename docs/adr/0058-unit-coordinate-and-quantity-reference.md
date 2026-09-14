---
id: ADR-0058
title: Separate unit coordinates from physical reference conversions
status: proposed
date: 2026-09-14
deciders: [paul-heyse]
level: decision
principles: [DM-02, DM-06, DM-07, DM-08, DM-15, DM-24]
blueprint: [§6.2, §7.2, §8.1, §8.2]
review: docs/design_review/reviews/design_review_wave-1-contract-corrections_2026-09-14.md#6-acceptance-gates
evidence: Proposed
supersedes: []
superseded-by: null
revisit: A supported unit spelling carries another physical reference convention or a conversion changes both basis and reference
verification: `just test-package pse-quantity -p pse-mathir -p pse-relations`; psig/Pa/absolute composed conversion and incompatible-reference negative fixtures
---

# ADR-0058: Separate unit coordinates from physical reference conversions

## Context

The blueprint described psig as an SI-anchored affine unit and gauge pressure as a quantity with an explicit reference-state conversion. Applying both offsets adds the pressure datum twice. Unit conversion alone lacks the complete physical context needed to decide whether a datum changes.

## Scope

Correct Wave 1 unit and complete-quantity conversion contracts before implementing the standard fixture and P10 adapter. Ordinary Celsius/Fahrenheit point-versus-difference semantics remain governed by blueprint §8.2.

## Drivers

Apply every semantic conversion exactly once, preserve reference conventions explicitly, and reject operations whose required context is absent.

## Options

Remember that an offset was applied using an execution flag: rejected because it encodes history instead of meaning. Treat psig as a standalone route to an absolute value while retaining a gauge quantity type: rejected as semantic relabeling. Bind a datum-bearing unit spelling to an explicit quantity reference, normalize representation within that reference, and change the reference only through its conversion rule: selected.

## Outcome

Add nullable `reference_state_id` to reference.units. It restricts the complete quantity context in which a datum-bearing unit spelling may be used. Unrestricted units such as Pa or psi can represent coordinates within the quantity's declared reference; their absence of a restriction does not erase that reference.

The standard psig unit has the psi scale, zero unit offset, and the explicitly declared gauge-reference restriction. Converting psig to Pa within the same gauge quantity type scales once. The registered gauge-to-absolute quantity conversion adds the datum once. A bare unit-only conversion involving a restricted unit is refused. A context-aware unit conversion checks both source and destination restrictions against the complete quantity type, then computes the representation transform. A reference mismatch requires a named quantity conversion, not metadata substitution.

### Consequences

Unit rows and generated constructors gain one optional field. Existing unit-only callers must supply a complete quantity context for restricted units. No new hashing or dependency framework is introduced.

### Compensating controls

Composed tests check zero psig -> zero gauge Pa ->101325 absolute Pa, reverse conversion, incompatible datum rejection, difference behavior, Celsius/Fahrenheit offsets and absence of a second datum application. Physical reference dependencies are explicit inputs to any reusable result.

### Confirmation

The named unit/quantity/math gates must compare numeric values and complete resulting types, with zero failure baseline. No checksum comparison discharges this check.

## Pros and cons

An explicit restriction is a small schema addition with a clear validation point. Unit-only convenience is reduced precisely where it could otherwise change physical meaning silently.

## More information

ADR-0054; blueprint §6.2, §8.2; plan 03 Q-4/Q-5. PSE_DESIGN_EDIT=1 is used for the authorized amendment and must be stated in the decision PR.

## Status history

- 2026-09-14 — proposed before implementing the gauge conversion correction discovered in Q-4.
