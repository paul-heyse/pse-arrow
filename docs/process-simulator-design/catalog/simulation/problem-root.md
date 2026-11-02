---
title: "Simulation concepts — Problem root"
---

# Simulation concepts — Problem root

<a id="source-40a141b5836981068430f88b37741790"></a>

## Header

Source: [Notion](https://app.notion.com/p/40a141b5836981068430f88b37741790).

**Refers to:** Nothing

**Rules:** Content keys are computed from their declared dependencies, never authored. Stable object IDs survive renames and are not content hashes. Required physical assumptions, branch selections and behavior-changing annotations are semantic; numerical starts and method settings still enter the relevant submission, preparation and provenance keys.

**Multiplicity:** 1

**Definition:** Schema and task-profile versions, title and stable semantic/source identities, with versioned content-key projections for the semantic question and the exact submission. Model revision, prepared product, attempt and result identities remain distinct.

**Layer:** Problem root

**Owned by:** [SimulationProblem](problem-root.md#source-40a141b58369813f9bc6dcfae91d96c7)

**Concept:** Header

<a id="source-40a141b58369813aaf53ff79a7e8521d"></a>

## MethodRequest

Source: [Notion](https://app.notion.com/p/40a141b58369813aaf53ff79a7e8521d).

**Refers to:** Kernel and form identities on the execution side

**Rules:** Required entries must be supported in the selected context or be explicitly refused; advisory entries do not guarantee selection. Scientific model choices and semantic branch/validity requirements are not hidden method preferences. All consumed settings participate in the relevant preparation/submission identities.

**Multiplicity:** 0..1

**Definition:** Execution preferences and requirements for numerical realizations, solver/provider choices, discretization, derivative products and resource policy. Required selections may fix a method for a controlled comparison.

**Layer:** Problem root

**Owned by:** [SimulationProblem](problem-root.md#source-40a141b58369813f9bc6dcfae91d96c7)

**Concept:** MethodRequest

<a id="source-40a141b5836981688530f7b5250f2695"></a>

## ProblemTemplate

Source: [Notion](https://app.notion.com/p/40a141b5836981688530f7b5250f2695).

**Refers to:** Its own slices

**Rules:** Filling its open inputs produces a concrete candidate request, which still requires contextual model/task/kernel admission. Every exposed product declares its type, coverage and possible unavailable outcome; binding values alone does not prove solvability.

**Multiplicity:** 0..n in a Study; 1 in a sub-problem realization

**Definition:** A reusable task envelope with declared open inputs, allowed model/configuration selections and exposed outputs. It can be instantiated by a Study node or an explicitly contracted nested realization.

**Layer:** Problem root

**Owned by:** [Study](study.md#source-40a141b5836981e78d53d04006644582); [Realization](model.md#source-40a141b5836981b2b65ef168ca05ff15)

**Concept:** ProblemTemplate

<a id="source-40a141b58369813f9bc6dcfae91d96c7"></a>

## SimulationProblem

Source: [Notion](https://app.notion.com/p/40a141b58369813f9bc6dcfae91d96c7).

**Refers to:** Admitted model revision and package closure; specification or operating case; registered task profile; optional execution preferences

**Rules:** Owns the request metadata and explicit references to one Vocabulary closure, Model, Specification and Query, plus an optional MethodRequest. Existing authored declarations remain model authority; embedding or projecting them creates no second authoritative object graph. Changes create a new immutable revision or request with the applicable versioned identity projection.

**Multiplicity:** Root, or owned by a Study problem node

**Owns:** [Header](problem-root.md#source-40a141b5836981068430f88b37741790); [MethodRequest](problem-root.md#source-40a141b58369813aaf53ff79a7e8521d); [Vocabulary](vocabulary.md#source-40a141b583698143b08bf5061e91c451); [Model](model.md#source-40a141b58369817a8e18f1c03bc9bcc7); [Specification](specification.md#source-40a141b583698104b5e7f14100c5b90d); [Query](query.md#source-40a141b5836981c4a79edbc9b84b41d1)

**Definition:** An immutable task envelope that references a versioned domain model, its selected specification or operating cases, a registered task profile and requested assessments and outputs. The first-party domain is flowsheet process modeling; this envelope is not a universal physics ontology.

**Layer:** Problem root

**Concept:** SimulationProblem
