---
title: "Simulation concepts — Cross-cutting"
---

# Simulation concepts — Cross-cutting

<a id="source-40a141b5836981c6ba96d20249d627bb"></a>

## Annotation

Source: [Notion](https://app.notion.com/p/40a141b5836981c6ba96d20249d627bb).

**Refers to:** Anything

**Rules:** Only genuinely descriptive metadata is excluded from semantic/preparation identity. Bounds, conservation tolerances, phase/branch policies, source selections or behavior-changing annotations are typed contracts with their own authority and identity dependencies.

**Multiplicity:** 0..n on any concept

**Definition:** Descriptive metadata such as documentation, tags, citations and authorship when it has no declared effect on scientific or execution behavior.

**Layer:** Cross-cutting

**Concept:** Annotation

<a id="source-40b141b58369819db846e3d9a5250500"></a>

## Assessment requirement

Source: [Notion](https://app.notion.com/p/40b141b58369819db846e3d9a5250500).

**Refers to:** Relation; Quantity; Query constraint; Accuracy demand; Validation finding

**Rules:** Separate mandatory acceptance from optional diagnostics, and distinguish proved, estimated or assumed evidence. Each requirement has scope, method/threshold or acceptance policy and an explicit outcome when unsupported or unassessed. Solver convergence alone does not discharge it.

**Multiplicity:** 0..n

**Definition:** A typed requirement to assess a scientific or task contract, including conservation, original-equation feasibility, physical applicability, output accuracy, structural conditions or result usability, with its owner and required evidence strength.

**Layer:** Cross-cutting

**Owned by:** [Query](query.md#source-40a141b5836981c4a79edbc9b84b41d1); [Model](model.md#source-40a141b58369817a8e18f1c03bc9bcc7)

**Concept:** Assessment requirement

Added in overall model v3 on 2026-11-01. The catalog properties carry this concept's definition and obligations. See [Unified Simulation Domain Model](../../simulation/index.md) for its formulation and execution context.


<a id="source-40a141b58369815d812ddfb1eae8fa0f"></a>

## Expression

Source: [Notion](https://app.notion.com/p/40a141b58369815d812ddfb1eae8fa0f).

**Refers to:** Slices, units

**Rules:** Dimensionally consistent; expectation and probability operators appear only in Query expressions.

**Multiplicity:** 1 in a relation, derived quantity, observation model, criterion, query constraint or output request

**Definition:** A formula over slices, constants and operators: arithmetic, differential and integral operators, reductions over index sets (sum, integral, max, final value), and in the Query only, expectation and probability operators.

**Layer:** Cross-cutting

**Concept:** Expression

<a id="source-40a141b5836981e98b0af95bbfb5828b"></a>

## Identity path

Source: [Notion](https://app.notion.com/p/40a141b5836981e98b0af95bbfb5828b).

**Refers to:** Its owner

**Rules:** Paths are unique within their declared lookup scope but can change on rename or move. References and source/result mappings use stable semantic IDs plus version/context; a display path is not identity.

**Multiplicity:** 1 per owned concept

**Definition:** A human-readable hierarchical locator derived from owner names and local labels, for example [plant.R101.energy](http://plant.R101.energy)\_balance, used for display, lookup and source navigation.

**Layer:** Cross-cutting

**Concept:** Identity path

<a id="source-40a141b58369815d83f0e267e4d621c9"></a>

## Index selection

Source: [Notion](https://app.notion.com/p/40a141b58369815d83f0e267e4d621c9).

**Refers to:** Index sets

**Rules:** Lies within the target's index domain; continuous points carry units.

**Multiplicity:** 0..1 per reference

**Definition:** A part of an index space: all of it, a subset of labels, a point or interval on a continuous domain (t = 0, t = final, z from 0 to 0.2 m), a boundary region, or a scenario.

**Layer:** Cross-cutting

**Concept:** Index selection

<a id="source-40a141b5836981ab971de59e4bee24d0"></a>

## Reference

Source: [Notion](https://app.notion.com/p/40a141b5836981ab971de59e4bee24d0).

**Refers to:** The target identity

**Rules:** Resolve target identity, expected kind, version and visibility/ownership scope. Dependency rules follow declared authority boundaries rather than a universal vertical ordering of every relationship; provenance and result references do not change source authority.

**Multiplicity:** 0..n in the concept holding it

**Definition:** A typed reference to a stable semantic entity and the required revision/context, such as quantity, relation, component, port, index set, choice, dataset or scenario; a path is a readable locator.

**Layer:** Cross-cutting

**Concept:** Reference

<a id="source-40b141b5836981e3871ee0dea5d905fe"></a>

## Semantic identity

Source: [Notion](https://app.notion.com/p/40b141b5836981e3871ee0dea5d905fe).

**Refers to:** Header; Reference; Identity path; Naming map; source and instance revisions

**Rules:** Retain identity across supported renames and edits; include the applicable revision/context in references. Derived numerical artifacts retain explicit derivation identities and source mappings rather than reusing array positions as semantic IDs.

**Multiplicity:** 1 per authored entity or instantiated occurrence

**Definition:** A stable typed identifier for an authored entity or an instantiated occurrence, distinct from its display name/path, source span, content digest, numerical ordinal and storage address.

**Layer:** Cross-cutting

**Concept:** Semantic identity

Added in overall model v3 on 2026-11-01. The catalog properties carry this concept's definition and obligations. See [Unified Simulation Domain Model](../../simulation/index.md) for its formulation and execution context.


<a id="source-40a141b583698168b8e2c978ee205227"></a>

## Slice

Source: [Notion](https://app.notion.com/p/40a141b583698168b8e2c978ee205227).

**Refers to:** Quantity, index selection

**Rules:** Selections resolve in their declared context. Each effective occurrence has an unambiguous numerical role within a task, while uncertainty, observation and source provenance remain separate annotations/contracts.

**Multiplicity:** 0..n in the concept holding it

**Definition:** A reference to a stable quantity identity plus an explicit index/case selection, used to target bindings, task roles, outputs and constraints.

**Layer:** Cross-cutting

**Concept:** Slice

<a id="source-40b141b5836981a0972ec6905ba54ace"></a>

## Validation finding

Source: [Notion](https://app.notion.com/p/40b141b5836981a0972ec6905ba54ace).

**Refers to:** Assessment requirement; Diagnostic; Accuracy report; Status; source/model/form identities

**Rules:** Static contracts, structural diagnostics, runtime checks and post-run scientific validation retain separate meanings. A numerical rank test is not universal identifiability proof; unavailable evidence remains unavailable. Findings can exist for refused or unsolved models.

**Multiplicity:** 0..n on admission, preparation or retained outcomes

**Definition:** A source-attributed assessment result recording the requirement/claim, method, selected context, evidence and outcome: satisfied, violated, unsupported or not assessed, with proved, estimated or assumed status as applicable.

**Layer:** Cross-cutting

**Owned by:** [Solution](solution.md#source-40a141b5836981029300ed3530505b8c)

**Concept:** Validation finding

Added in overall model v3 on 2026-11-01. The catalog properties carry this concept's definition and obligations. See [Unified Simulation Domain Model](../../simulation/index.md) for its formulation and execution context.


<a id="source-40a141b5836981dbba7fcd513df97fc3"></a>

## Value

Source: [Notion](https://app.notion.com/p/40a141b5836981dbba7fcd513df97fc3).

**Refers to:** Quantity kind, index sets

**Rules:** Unit compatible with its target; trajectories and fields cover the selected interval or region.

**Multiplicity:** 1 in a binding, guess or Solution entry

**Definition:** A unit-carrying datum: scalar, vector or tensor, discrete label, structured object (such as a molecular graph), a trajectory or field with its interpolation meaning, or a distribution.

**Layer:** Cross-cutting

**Concept:** Value
