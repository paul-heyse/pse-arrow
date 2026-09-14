---
id: ADR-0054
title: Complete indexed physical typing and operator inputs
status: proposed
date: 2026-09-14
deciders: [paul-heyse]
level: decision
principles: [DM-02, DM-06, DM-07, DM-08, DM-15, DM-24]
blueprint: [§6.2, §6.4, §6.9, §7.2, §7.3, §7.4, §7.7, §8.2, §8.3]
review: docs/design_review/reviews/design_review_wave-1-contract-corrections_2026-09-14.md#6-acceptance-gates
evidence: Proposed
supersedes: []
superseded-by: null
revisit: An indexed consumer needs an additional physical distinction or an operator input cannot be represented by these relations
verification: `just test-package pse-quantity -p pse-relations`; `just test-package pse-mathir -p pse-relations`; quantity and ordered-IR conformance fixtures

---

# ADR-0054: Complete indexed physical typing and operator inputs

## Context

PiecewiseLinear has breakpoint/type payload but no input expression. Indexed equations omit the residual type P10 must preserve. Phase validity prose distinguishes absent restrictions but the relation list did not allow absence.

## Scope

Amends the cited blueprint sections within the approved Wave 1 boundary. It supplements existing accepted decisions; their arguments remain immutable. Implementation is authorized by the maintainer's approved execution plan; formal ADR acceptance remains the decision-PR lifecycle.

## Drivers

Explicit semantics, enforceable validity, complete dependencies and honest capability claims before reuse or performance work.

## Options

Retain the inconsistent scaffold: rejected because its consumers cannot preserve the declared meaning. Build a general replacement platform: rejected without a demonstrated need. Complete the existing typed contracts: selected.

## Outcome

Conversion rules persist nullable `scale` and `offset` coefficients: `scale` requires a finite scale and no offset; `affine` requires finite scale and offset; `kernel` has neither coefficient and requires a kernel identity. This makes datum conversion reproducible from the reference relations. Unit-definition normalization remains separate. The quantity registry takes an explicit neutral-dimensionless type binding and validates its complete scalar contract; dimension equality alone never selects a Boolean, fraction, or other physical kind.

PiecewiseLinear has one expression child in math_expr_args and retains breakpoint/type payload. Gather has only its group/coordinate payload; no dummy expression child is invented. Kernel binding input-node references participate in graph traversal, validation, hashing and remapping. Add nullable residual_quantity_type_id to math_indexed_equations; P10 requires a resolved non-null value. valid_phase_types is nullable: null selects the declared default, empty is an explicit empty restriction. Literal typing/normalization is per occurrence and expected complete quantity contract before typed consing; shared source literals may specialize into distinct typed nodes. Scope reconciliation is order-independent: disagreeing scopes become None and remain None. The single operator table owns arity and operator semantics; registry projections derive from it. Persist fixed arity count, dependent/payload domain-rule text, traversal family and foldability alongside the existing operator fields; an enum tag alone cannot preserve the fixed child count. Preserve every ordered payload reference and conditional evaluation region. Normalized expression node tables add the derivation identity and exact authored source span to the common mathematical node fields; payload shapes project the common declarations.

A physical `UnitConvert` edge persists its named rule as a `math_quantity_selections` row with `builtin_rule = unit_convert` and one operand-zero conversion. P10 validates predecessor selections as claims against actual child and result complete types, coefficients and required parameter facts. An edge with no named rule is a representation conversion preserving its complete physical type. Inserted physical edges retain their selections; kernel conversions use resolved KernelCall bindings. This distinction prevents unit normalization from silently applying a physical datum.

P3 preserves a template-local domain by its declared composite key `(template_id, domain_name)`. An untyped expression domain reference is a tagged alternative: an actual domain identity, or that composite template key. Normalized domain-bearing payloads persist a nullable actual domain field and paired nullable template owner/name fields, with exactly one alternative present. P10 and compiled payloads require an actual domain resolved through `instance_domain_bindings`; unresolved template references are rejected before physical inference. Neither named hashes nor arbitrary identifiers manufacture a domain declaration. This completes the existing template parse/instantiation boundary without implementing P4–P9.

## Consequences

An explicit Affine constant carries nullable `constant_quantity_type_id` and `constant_unit_id` together. Every nonzero constant requires this complete contract; an untyped zero remains neutral. A supplied unit must be canonical for that complete quantity type, including its reference and scale kind. Noncanonical constants use explicit ordered Add/UnitConvert nodes. The constant participates as operand zero in physical inference, preserving signed zero and term order; its type is never inferred from an asserted parent result.

Scale and affine conversion rules reject nonempty `required_parameters`: their persisted finite coefficients completely specify the transform. Parameterized kernel conversions resolve an explicit existing kernel binding, validate its actual input node, named kernel, complete source and destination types, and every parameter value and unit. The resulting KernelCall persists its binding and conversion selection, whose semantic dependencies include those facts. A callback may locate the binding but cannot certify its validity.

Consumers must use checked contracts and explicit unsupported outcomes. Fixtures and generated output change with their authoritative declarations; this record does not claim runtime acceptance.

### Compensating controls

The scoped design review, negative fixtures and plan 03 terminal gates guard the correction. Existing canonical framing and dependency-family pins remain unchanged except the explicitly named direct promotion.

### Confirmation

`just test-package pse-quantity -p pse-relations`; `just test-package pse-mathir -p pse-relations`; quantity and ordered-IR conformance fixtures. Each result records mode and failure count against baseline zero. The execution ledger distinguishes source inspection, focused tests and complete-wave acceptance.

## Pros and cons

The correction removes an ambiguity or false guarantee with bounded implementation work. Conservative rejection/copying/recomputation may cost more until a separately measured refinement is justified.

## More information

[Wave 1 execution plan](../plans/03-wave-1-foundations.md); blueprint §6.4, §6.9, §7.2, §7.3, §7.4, §7.7, §8.3; charter DM-59 and G1-G7. The authorized blueprint amendment uses PSE_DESIGN_EDIT=1 and must carry a revision row in the decision PR.


### Revision 20: complete normalized sources and unresolved bindings

**Proposed before implementation.** Blueprint §7.7 now gives every source its complete typed row key, exact field path and explicit graph root. Predicate and equation relations retain Boolean/comparison/membership structure, equation senses and conditional branches independently of the numerical Opcode dictionary. Normalized tagged references retain actual composite template parameter/feature/port keys, declared domains and lexical indices; normalized guards reference the predicate graph explicitly. P10 refuses these unresolved alternatives. No fabricated identity or placeholder arithmetic node stands in for missing meaning.

Blueprint §8.2 adds the exact package-to-unit-set binding and a derived normalized unit relation. Selection is explicit, dimension and ordered scale computation are checked, and conversion uses the existing quantity implementation. P3 converts only unconstrained zero-offset units; affine and reference-bearing literals retain authored units until P10 has the expected complete quantity type, without guessing point or difference from the spelling. A derived unit ID identifies its package/unit-set/dimension derivation; exact row values and dependency admission establish correctness. The approved source-root/key and derived-unit identity framing is documented in the blueprint before code.

Expression property demand uses an explicit template_symbol_properties mapping; opaque requirements and this mapping reference the existing authored.scopes selector graph. Seed scope_id denotes the declared logical scope until P4/P5 resolves members. No name-based property inference or invented scope identity is permitted.

Required negative controls: stale source bytes/rows with unchanged identifiers; duplicate/ambiguous bindings; invalid typed key alternatives; dangling/cyclic roots and predicate/equation branches; unresolved references offered to P10; missing/ambiguous unit context and altered unit definitions with unchanged identity. Positive fixtures retain lexical shadowing, mixed predicates, conditional senses, nested DSL field roots and dimensioned literal conversions. Formal decision acceptance and terminal Wave 1 gates remain pending.

### Revision 23: compound literal-unit derivations

**Proposed before implementation.** A compound unit expression resolves declared symbols, ordered multiplication/division and rational powers into an actual derived unit definition. Dimension ratios and scales are checked through the quantity types, and affine/reference-bearing factors are rejected. Persist the complete normalized unit row before conversion; identity is `named_id(package_id, "pse:p3:unit-expression:v1:" + exact_unit_expression)` with the selected package and unit-set context. The identity cannot certify a component definition or admit an altered scale. Negative controls mutate a component unit under unchanged identity, use a wrong dimension, non-finite or nonpositive scale, and introduce an affine factor. Positive controls normalize compound flow units and compare the exact converted value and dimension.

## Status history

- 2026-09-14 — proposed before implementation; maintainer approved the correction plan and its validation-before-hashing clarification.

### Normalized pending indexed selection

Proposed before implementation (blueprint revision 28): numeric and compound index
expressions must survive template normalization without fictitious bound-index IDs.
`PendingGather { group, indices }` retains the exact template symbol declaration and
ordered index-expression node references under the existing Gather opcode. Declared
axis arity, every graph reference and lexical bindings are validated directly. Normalized
gather rows append the exclusive resolved/pending state and optional index-node list;
the existing coordinate map is present only for resolved lexical selections. P4/P5 must
resolve actual domain members and instantiation; P10 explicitly refuses pending selection.
Compiled numerical Gather storage is unchanged. Tests must preserve `x[1]` and `x[i+1]`,
ordered multidimensional indices and all referenced nodes; reject wrong arity, dangling
references and unresolved pending payloads at the physical compiler boundary.

### Revision 24: unresolved conversions and scoped guards

**Proposed before implementation.** Normalized UnitConvert admits an exclusive pending request carrying only the target unit, while resolved and compiled conversions retain their complete source, target and coefficients. P10 uses the actual child quantity type and unit definitions to resolve a pending request through the existing checked conversion implementation, or refuses unresolved template context. No source unit, scale kind or affine coefficient is guessed. The numerical opcode dictionary remains unchanged.

Integral filters retain an optional explicit GuardRef, including the full normalized source/predicate key; physical admission requires supported actual Boolean guard semantics or a typed refusal. Property-demand guards also persist that complete composite key through paired guard_source_id and guard_node_id. Smoothing epsilon omission requires an actual selected declared numerical-policy default; absent such a binding, epsilon is required. Tests must distinguish absent, true and false filters, reject incomplete guard pairs and unresolved compiled conversion requests, and prove point/difference conversions use the full child type.

Integral also persists its lexical bound-index identity; otherwise the body would contain an unowned coordinate. Expression demand seeds carry the exact source symbol declaration and a nullable index: null means deferred template axes, empty means an actual scalar, and populated means concrete actual domain/member identities. Opaque actual domain bindings expand only over checked finite inventories. P4/P5 resolves every deferred index before P6. These distinctions are declared before implementation and never encoded through invented domain IDs.

A demand seed's explicit predicate guard describes its inline evaluation region. P4 also resolves every enclosing declaration guard and equation filter by actual joins through expression_sources, complete typed source row keys/fields, normalized declaration rows and their own expression roots. It conjoins all these conditions before activating demand. This uses already retained authoritative facts rather than duplicating condition meaning into a second field. A fixture must prove both an inline branch guard and a false enclosing declaration guard remain reachable from the same seed.


### Revision 29: explicit smoothing tolerance coordinates

**Proposed before implementation.** Blueprint §8.2 now defines the complete tolerance contract for the existing smooth/safe operators. Bare and resolved epsilon values have an explicit first-operand canonical-coordinate meaning. A normalized PendingSmoothOp retains a declared epsilon unit and value until P10 can validate the actual complete scalar tolerance type. Origin-sensitive operands require the actual registered difference type; matching dimensions or a different kind, basis or reference cannot establish that type. SafeSqrt epsilon belongs to its input coordinate, and SafeLog requires dimensionless input.

The normalized smooth projection carries exclusive coordinate/pending_unit state and a nullable declared epsilon-unit reference; compiled f64 storage stays resolved. Existing checked difference-aware unit conversion handles affine spellings without adding an origin. No numeric expression expansion, argument reordering or implicit policy default is introduced. Full actual unit and quantity definitions remain dependencies; hashes only index the resulting artifacts. Required controls cover Pa/kPa and Fahrenheit/Kelvin tolerances, missing/mismatched complete tolerance types, incompatible dimensions, absent/nonpositive/nonfinite values, round-trip exclusivity and replay after changed unit definitions.

- 2026-09-14 — revision 29 proposed before implementation: preserve declared smoothing units and validate exact complete tolerance context before coordinate conversion.
