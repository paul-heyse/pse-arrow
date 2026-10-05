---
id: ADR-0135
title: Admit physical intermediates, mapped laws and directed transfers
status: proposed
date: 2026-09-30
deciders: [paul-heyse]
level: decision
principles: [AP-02, AP-04, AP-06, DP-01, DP-04, PS-01, PS-07]
blueprint: [§8.1, §8.2, §8.3, §8.4, §9.8, §14.3]
review: docs/design_review/reviews/design_review_plan25a-contracts_2026-09-30.md
evidence: Implemented
supersedes: []
superseded-by: null
revisit: A supported scientific law requires erasing physical meaning, or an admitted coordinate/reference change leaves preparation identity unchanged.
verification: Plan 25a focused contract, indexed algebra, map/derivative, residual-potential, transfer orientation and datum-translation controls; integrated qualification in Plan 25k.
standard: core-3.3/process-simulator-1.3
scenarios: [docs/design_review/reviews/design_review_codebase-domain-alignment_2026-09-30.md#s01, docs/design_review/reviews/design_review_codebase-domain-alignment_2026-09-30.md#s09]
---

# ADR-0135: Admit physical intermediates, mapped laws and directed transfers

## Context

Plan 25a addresses F01/F03 and the physical portions of F08/F35. Named boundaries are
typed, but production bodies remove and restore units; reduced laws lose coordinate
meaning and energy transfers borrow a material enthalpy datum. ADR-0124's requirement
that every intermediate resolve to a registered quantity contributes to this pressure.

## Scope

This proposed successor changes intermediate inference, mapped-function admission,
physical reconstruction, transfer/reference operations and their preparation identities.
The intended successor scope for ADR-0124 is its named-only intermediate inference and
competing stepwise/chain admission, together with the replaced lowering contract. Its
canonical unit products, defined-unit admission, rational exponents and public declaration
requirements remain. Acceptance and symmetric supersession links require the decision PR;
this proposed record leaves both records' lifecycle fields unchanged. Public named
quantities remain declared; no new quantity IDs are synthesized.

## Drivers

Ordinary scientific composition must preserve basis, datum, kind, subjects, coordinate
roles and indices. The scientific declarations own formulas; generic admission checks
their contracts and existing library mathematics differentiates/evaluates them.

## Options

- Dimension-only anonymous types would allow same-dimension wrong-kind substitutions.
- More public intermediate kinds preserve the authoring burden and do not solve map roles.
- A separate reduced-law runtime or differentiation engine duplicates existing mathematics.
- Selected: one resolved physical contract and admitted operations, with ordinary checked
  function composition and explicit scientific reconstruction.

## Outcome

### Resolved contracts and numerical values

A quantity-owned checked conversion plan binds the selected complete quantity and source
unit. It checks finite input and output around the existing multiply-then-add operation.
Representation conversion cannot change basis, datum or subject. Missing optional cells
remain missing. Inline, column and mathematical scalar consumers share this operation.

Internal physical contracts carry semantic kind factors, exact exponents, unresolved
basis/datum/scale/subject obligations, free binders, ordered axes and numerical unit
representation. A registered quantity identity is optional internally and required at
public named boundaries. The checker retains the admitted operation by structural body
occurrence and lexical
position, not merely rendered syntax or source span. Concrete products retain the selected
rule and operand/result contracts; generic products retain requests and schemes, instantiated
once against the actual caller substitutions. Specialized functions preserve these products.
Lowering refuses a missing product and consumes admitted operations before numerical
construction. Finite mathematical requests create admission at their resolved occurrence;
BodyBuilder never rebuilds a competing inference decision.

Normalization cannot rescue a refused registered operation. Valid routes must agree.
Cancellation cannot discard qualified-factor obligations or free binders. Declared
operations retain their lawful transport, affine, weighted-mean, derivative and reference
semantics. Boundary admission normalizes composed numerical unit scales explicitly.

### Mapped scientific laws

A coordinate map declares typed physical arguments, guards and identified scalar/indexed
slot expressions. Coordinate<map.slot> is a nominal modeling refinement: no general Scalar
cast or accidental substitution of a different slot/map exists. Slots lower to ordinary
checked pure functions. A wrapper composes them through the existing mathematical path.

A reduced-law application retains its law/map identity, reconstruction contract,
normalization and ideal-reference convention. Explicit authored reconstruction operations,
not expected dimensions, turn that product into ResidualHelmholtzEnergy or
ExcessGibbsEnergy. These differ from stored internal energy. Current PR/PC-SAFT potentials
are residual: the ideal pressure and standard-state fugacity terms remain. NRTL's current
degree-one reduced-amount convention is reconstructed once, without a second total-amount
factor. Physical partials use independent T,V,n or T,p,n arguments before substituting
density coordinates; composition-dependent references remain in the expression graph.

Within an admitted response declaration, addition and subtraction may assemble anonymous
potential-derived terms with other physical terms when dimensions agree. Registered
operation matches, ambiguity and prerequisite refusals remain final. The scoped formula
authority preserves compatible basis, datum, subject, ordered axes and actual binders,
and survives specialization and mathematical lowering. The selected physical function
must retain an admitted reconstruction after library
normalization, and a response must retain its selected potential or physical partials.
Abstract scientific calls establish this dependence before substituting reduced-law
implementations. Syntactic reachability, unused calls, zero coefficients and cancellation
do not provide a witness; a legitimate zero reduced law can still return zero value and
partials. A common basis may be consumed at a reconstruction or response boundary only when the result has no basis, its amount exponent is zero,
and the carried nonempty bases agree. Ordinary expressions retain full semantic contracts.

LogFugacityCoefficient/FugacityCoefficient and LogActivityCoefficient/ActivityCoefficient
are named species-subject contracts. Exponentiation preserves that scientific subject;
unit `1` does not authorize interchange with neutral Scalar.

ResidualMolarEnthalpy and ResidualMolarEntropy are datum-free physical responses, distinct
from the referenced DeltaH/DeltaS differences between material points. Explicit mixed
operations combine these residual responses with stock material points while preserving
the latter's datum.

### Directed transfers and reference changes

EnergyTransferRate and MechanicalEnergy are datum-free.
Transfer<quantity,boundary,Into|OutOf> is a modeling
contract whose owner resolves to instance, boundary declaration and coordinates. Numeric
negation does not change owner/convention. Same-owner reorientation and paired-boundary
reflection are explicit admitted operations. Contributions consume canonical Into once;
an additional sign modifier refuses. Referenced material energy flow remains referenced.

Registered mixed energy operations preserve the ledger datum. Heat cannot translate an
enthalpy reference. Reference translations consume source/target contracts, composition,
paired scientific anchors and provenance. Gauge translations consume the authored pressure
origin after unit conversion; differences receive no origin shift. Reaction extent is a
reaction-subject amount rate and reports carry its actual quantity/unit.

Translation specialization retains paired anchor function expressions at the same
explicit physical temperature and pressure for exactly the actual component members.
Finite weighted means use a point plus weighted differences, followed by an authorized
source-difference to target-difference translation and ordinary target-point addition.
Library evaluation and differentiation see the composition dependence and scientific
anchor calls. Heterogeneous report rows carry an optional actual instance/boundary,
ordered coordinate identities and direction; these are semantic context, not labels.

### Identity and cutover

Version changed physical and typed-body preimages. Include coordinate/slot definitions,
references, reconstruction normalization and ordering; do not reinterpret old recorded
digests. The minimal I1 foundation supports this without importing its cache/resource work.
Delete replaced callers/mechanisms/tests with their replacements, retaining independent
scientific oracles. No compatibility API or second production path is introduced.

### Consequences

Quantity, modeling, compiler/math and authored scientific interfaces change together.
Anonymous intermediate contracts remain internal. Map slot identities follow the existing
identity policy; name-derived renames are not promised identity stability. R-51 retains
physical-closure composition as deferred work while widening its adequacy trigger.

### Compensating controls

Focused controls cover qualified cancellation, invariant refusal, non-unit canonical
scales, indexed/empty reductions, wrong coordinate roles, independent physical derivatives,
zero residual potential, excess homogeneity, all orientation combinations and contextual
datum translation. Scientific oracles detect type-correct but wrong formulas.

### Confirmation

Implemented contract scope is recorded in blueprint §8, §9.8 and §14.3. The owning plan
records the [actual focused commands and outcomes](https://github.com/paul-heyse/pse-arrow/blob/ad665a0222551196b1160e426f5242361215a6a0/docs/plans/25a-physical-values-and-contextual-contracts.md#outcome-recorded-after-implementation). Selected scientific-declaration controls
retain current scientific bodies and use small synthetic parameter rows; external dataset
transport and full package journeys remain Plan 25k scope. This proposed decision does not
claim integrated product qualification or decision acceptance.

## Pros and cons

Physical meaning governs the actual path and scientific formulas stay authored. The
one-time migration crosses core typing and every affected consumer; it cannot be reduced
to introducing helper types.

## More information

[Plan 25a](https://github.com/paul-heyse/pse-arrow/blob/ad665a0222551196b1160e426f5242361215a6a0/docs/plans/25a-physical-values-and-contextual-contracts.md) owns packets;
[Plan 25](../plans/25-design-remediation.md) owns finding dispositions. ADR-0124,
ADR-0127 and blueprint §8/§9.8 provide current context.

## Status history

- 2026-09-30 — proposed before the maintainer-authorized Plan 25a contract implementation.
