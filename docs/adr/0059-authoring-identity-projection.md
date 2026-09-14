---
id: ADR-0059
title: Declare authoring identity and naming projections explicitly
status: proposed
date: 2026-09-14
deciders: [paul-heyse]
level: decision
principles: [DM-02, DM-06, DM-07, DM-09, DM-24, DM-60]
blueprint: [§4.2, §6.1, §22.1]
review: docs/design_review/reviews/design_review_authoring-identity-projection_2026-09-14.md#6-acceptance-gates
evidence: Proposed
supersedes: []
superseded-by: null
revisit: An authoring section requires identity or naming behavior beyond a declared scalar identity and owning entity reference
verification: `just test-package pse-schema -p pse-authoring -p pse-relations`; authoring document identity and registration fixtures
---

# ADR-0059: Declare authoring identity and naming projections explicitly

## Context

The blanket requirement that every authored primary key appear in authored.entities cannot represent composite relationship keys such as (template_id, name). Several semantic-ID records are unnamed, while nested named declarations require their actual owner's qualified name. Inferring identity or containment from the first key or foreign key silently changes meaning.

## Scope

Complete the existing DocumentSpec authoring projection within Wave 1. Amend blueprint §4.2, §6.1 and §22.1 before changing those fields. The maintainer authorized this bounded correction; formal acceptance remains the decision-PR lifecycle.

## Drivers

One declaration for each identity and naming rule; exact reference and registration validation; generated row contracts; stable explicit identities during rename.

## Options

Infer from key order or column names: rejected because relationship keys and multiple references are ambiguous. Handwrite loader rules per relation: rejected because generators and admission could disagree. Extend the existing document section declaration with explicit projection metadata: selected.

## Outcome

Each DocumentSection declares optional identity_column, entity_kind, name_column and naming_scope_column, plus independent expression_owner_column for DSL-bearing rows. An identity column must be the relation's sole nonnullable semantic-ID primary key. The authoring alias id maps only to that declared column; supplying both spellings is rejected. An entity kind requires an identity and textual name column. A naming scope is an explicitly declared semantic-ID foreign key to an entity identity; absent scope means the declaring package. Its actual resolved qualified name prefixes the local name. Missing parents, cycles, duplicate identities or qualified names, and named-policy identity mismatches are errors. No digest establishes any of these properties.

Only explicit entity mappings create authored.entities rows and incur closure:entity_registered. Composite relationship records retain their complete declared key and are validated through key and reference contracts. Unnamed semantic-ID records require a persisted explicit identity in every package policy; a named policy does not invent a name for them. Package identity is explicitly stored and is not derived from itself. The EntityKind vocabulary is declared once; document mappings reference its members and register no new vocabulary implicitly.

Expression owners are explicit semantic-ID foreign keys to template identities, independently of entity naming or registration. Every current template DSL surface, including nested binding values and nameless guards/display rows, declares its template_id owner. One shared source binder validates every parsed path against that exact template/instance/entity/domain inventory or an explicit lexical binder. Unknown or ambiguous paths are failures. P3 and full rename consume this binding result; neither guesses an owning FK.

The loader records exact original parser byte spans, injects only declared package/source context and identity projections into transient syntax values, and decodes the generated DTO. This transient value tree is not a second row contract. Document and section declarations, including all identity fields, are projected to reference.schema_documents and reference.schema_document_sections and enter the existing registry fingerprint without changing its framing. Registry admission checks mapping types and references before generation or loading.

### Consequences

The authoring projection has a small explicit inventory. Nested identities depend on resolved owning entities, so declaration order cannot determine identity. Generated schema/docs and row decoding share the same mapping.

### Compensating controls

Negative fixtures cover composite-key alias refusal, duplicate aliases, missing IDs, unknown entity kinds, wrong owner types, cyclic owners, wrong named IDs and unresolved parents. Positive fixtures cover reference roots and nested template declarations in reversed document order.

### Confirmation

Run the named focused test recipe with force-validate enabled and failure baseline zero. Compare exact row identities, names, parent references and original source slices, and mutate projection metadata to prove the registry fingerprint covers it.

## Pros and cons

The correction removes ambiguous implicit behavior without introducing a new authoring platform. Explicit metadata requires review when a new entity family is added.

## More information

ADR-0052, ADR-0053, ADR-0051; plan 03 C-1/C-2. The authorized blueprint amendment uses PSE_DESIGN_EDIT=1 and must carry that explanation in the decision PR.


### Revision 20: complete normalized sources and unresolved bindings

**Proposed before implementation.** Blueprint §7.7 now gives every source its complete typed row key, exact field path and explicit graph root. Predicate and equation relations retain Boolean/comparison/membership structure, equation senses and conditional branches independently of the numerical Opcode dictionary. Normalized tagged references retain actual composite template parameter/feature/port keys, declared domains and lexical indices; normalized guards reference the predicate graph explicitly. P10 refuses these unresolved alternatives. No fabricated identity or placeholder arithmetic node stands in for missing meaning.

Blueprint §8.2 adds the exact package-to-unit-set binding and a derived normalized unit relation. Selection is explicit, dimension and ordered scale computation are checked, and conversion uses the existing quantity implementation. A derived unit ID identifies its package/unit-set/dimension derivation; exact row values and dependency admission establish correctness. The approved source-root/key and derived-unit identity framing is documented in the blueprint before code.

Required negative controls: stale source bytes/rows with unchanged identifiers; duplicate/ambiguous bindings; invalid typed key alternatives; dangling/cyclic roots and predicate/equation branches; unresolved references offered to P10; missing/ambiguous unit context and altered unit definitions with unchanged identity. Positive fixtures retain lexical shadowing, mixed predicates, conditional senses, nested DSL field roots and dimensioned literal conversions. Formal decision acceptance and terminal Wave 1 gates remain pending.

## Status history

- 2026-09-14 — proposed before identity projection implementation under the maintainer's authorized Wave 1 design completion.

### Revision 26: exact field grammar

**Proposed before implementation.** DocumentSection expression_fields declares every DSL leaf by exact typed path and syntax (expression, predicate or equation), including list/struct nesting. Registry admission rejects missing, duplicate, extraneous or conflicting mappings. The mapping is persisted in schema_document_sections and drives loader, P1, shared binding, P3 and JSON Schema. The generic ExprDsl extension remains unchanged. Parse-any fallback would interpret Boolean guard atoms as arithmetic paths and is therefore removed at declared field boundaries. Positive controls include Boolean/atomic guards and nested expression values; negative controls include equations in arithmetic fields and malformed/incomplete field mappings.


### Revision 30: enum literal operands in normalized predicates

**Proposed before implementation.** The concrete blueprint guard `energy_balance_type != none` requires the literal member to be bound to the enum of the actual compared template feature or parameter declaration. The shared binder admits bare single-segment enum members only in equality/inequality context supplied by that operand's nonnull `enum_id`. It checks the exact registered member set; opposite enum declarations must agree. A name that also resolves as a lexical value or declaration is ambiguous and refused. Unknown enum identities/members and ordered enum comparisons are refused. No global member-name search supplies context.

The existing normalized predicate relation appends `left_kind`, `left_enum_id`, `left_enum_member`, `right_kind`, `right_enum_id`, and `right_enum_member`. The kind vocabulary is `expression|enum_literal`. An expression alternative retains the corresponding existing expression node; an enum literal carries the exact enum identity and member with no mathematical node. Each present operand selects exactly one complete alternative. All fields are projected through the registry and compared during actual-source P3 stage validation. Ordinary domain membership remains unchanged; an enum scalar is not a set and no enum-as-domain mapping is inferred.

The binder is shared by P3 and rename, so literal membership and declaration references remain distinct under edits. Positive controls cover left/right literal positions and actual feature/parameter context. Negative controls cover wrong/missing enum identities, unknown members, same spelling from a different enum, ambiguous lexical/entity bindings, ordered comparisons, arithmetic leakage and guessed membership context. No hash, fingerprint or numeric placeholder establishes any of these facts.
