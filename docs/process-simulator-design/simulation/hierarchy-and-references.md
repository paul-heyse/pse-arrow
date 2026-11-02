---
title: "Hierarchy and references"
notion_source: "https://app.notion.com/p/40a141b583698122a7dbd24b1b2f7357"
notion_last_edited: "2026-11-01T20:36:10.274Z"
---

# Hierarchy and references

Source: [Notion](https://app.notion.com/p/40a141b583698122a7dbd24b1b2f7357). Imported 2026-11-02.

**Version 3** · 2026-11-01

Ownership, semantic reference, and execution provenance are different structures. Ownership identifies a definition's or instance's lifecycle; references express relationships; provenance records how one representation was produced from another. A flowsheet can have a tree of owned components and a cyclic material-connection graph at the same time.

## Ownership

The following is a containment outline of the existing conceptual catalog, not a prescription for an object-oriented in-memory representation. A relational implementation may store the same relationships as typed foreign keys.

The Concepts database rows describe **concept types**. A type may list several permitted owner types, while each concrete occurrence has one containment owner. Shared reusable definitions are referenced by identity/version rather than treated as multiply owned instances.

| Owner | Owned content | Main cardinality or rule |
| --- | --- | --- |
| SimulationProblem | Header, Vocabulary, Model, Specification, Query, optional MethodRequest | One of each required section; immutable submission snapshot |
| Vocabulary | Units, quantity kinds, elements, chemical systems, port types, constitutive models, component templates | Definitions may be shared by versioned reference and included in a snapshot manifest |
| Chemical system | Species/phase membership and reactions in scope | Membership and chemical identity are distinguished; shared identities need not be duplicated |
| Component template | Configuration parameters and reusable structure | A definition may instantiate many components |
| Model | Root Component | Exactly one root scope |
| Component | Local index sets/domains, quantities, relations, constitutive uses, modes, slots, ports, connections, child components | Recursive ownership; process connectivity is a separate graph |
| Continuous domain | Boundary regions and optional geometry | Physical extent is separate from numerical mesh |
| Relation / Slot | Alternatives and selected or selectable realizations | Alternative contracts declare which observables and capabilities are shared |
| Connection | Port mapping where needed and generated relations | The mapping implements the declared connection semantics |
| Specification | Scenarios, scenario structure, datasets, observation models, realization selections | Case knowledge and source information |
| Scenario | Bindings, guesses, distributions, case-specific requirements | Overlap and completeness checked against active roles |
| Query | Roles, regime requests, answer profile, expressions, criteria, constraints, outputs, accuracy requests | Active task intent |
| Study | Problem nodes, data edges, generators, selectors, external sources/sinks, nested Studies | Explicit iteration and termination semantics |
| Solution | Status, members, estimates/statistics, diagnostics, accuracy report, provenance | Failed or infeasible runs may have zero solution members |
| Solution member | Values, derivatives, duals, analyses | Available outputs carry qualification and origin |
| Execution environment | Registry and available execution capabilities | Deployment context |
| Execution plan | Plan steps and reconstruction/provenance mappings | A plan definition is distinct from each Run |

A ProblemTemplate is a self-contained problem definition with declared open inputs and exposed outputs. It is referenced by a Study node or a sub-problem realization. A component's public interface must not depend on callers knowing its internal ownership layout.

## Stable identity and human paths

The catalog's Identity path concept supplies readable addresses such as `plant.R101.energy_balance`. Such paths are useful for navigation and diagnostics, but renaming or moving an unchanged component changes its path.

Every persisted entity therefore needs a stable identifier in addition to its current path. Instantiation records both the template identity/version and the instance identity. Compilation records origin identifiers and transformation history. A generated slack, discretization point, or reduced equation may originate from several model entities plus a transformation rather than from exactly one source variable.

Identity does not imply mathematical equivalence. Replacing a physical realization creates a different semantic problem even when its external interface retains stable endpoint identities.

## Reference rules

Cross-layer dependency follows the direction from reusable definitions toward instantiated models, cases, and questions:

| Referencing part | Permitted semantic references |
| --- | --- |
| Vocabulary | Other reusable definitions; pinned external reference data and package definitions |
| Model | Vocabulary, other entities within its model scope, and explicit public interfaces of nested components/sub-problems |
| Specification | Model/Vocabulary identities; its own scenarios, datasets, observation models, and pinned upstream values |
| Query | Model, Specification, Vocabulary, and other query expressions |
| Study | Problem templates, exposed outputs, input slices, and explicitly versioned external inputs |
| Solution | The submitted problem, available model-level outputs, and execution provenance |
| Execution | Problem entities, intermediate representations, runtime capabilities, and reconstruction maps |

The rule prevents a reusable physical model from secretly depending on a caller's active optimization objective or case bindings. It does **not** prohibit same-layer references, flowsheet recycles, reaction cycles, or equations that couple variables bidirectionally.

Expression-definition graphs must be acyclic where evaluation requires it. Physical dependency graphs may be cyclic. Ownership graphs must be acyclic. These are different validations.

A nested sub-problem has its own internal layers and exposes an explicit boundary; callers do not reach through that boundary into its internal Query. External evaluators are referenced by identity and version through realization contracts. MethodRequest remains the explicit place for requested execution settings.

## Slices and cross-instance alignment

A Slice names a quantity and a selection of its indices. Bindings, roles, observations, and outputs target slices. Validation expands or otherwise reasons about their scalar cardinality and overlap; a vector slice is not automatically one degree of freedom.

Connections and Study edges must declare alignment of component/species labels, phase membership, time or spatial coordinates, units, and bases. Similar-looking names do not establish identity or compatibility.

## Lifecycle versus sharing

Deleting an instance removes its owned instantiation, not the versioned reusable definition used by other instances. Immutable problem snapshots retain the exact definitions and data needed to interpret their results. A shared library definition may be referenced by many snapshots without forcing one live mutable owner to control them all.
