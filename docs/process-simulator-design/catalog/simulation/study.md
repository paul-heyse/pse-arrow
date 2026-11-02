---
title: "Simulation concepts — Study"
---

# Simulation concepts — Study

<a id="source-40a141b58369813c837ee8deb81d46b0"></a>

## Data edge

Source: [Notion](https://app.notion.com/p/40a141b58369813c837ee8deb81d46b0).

**Refers to:** Source output request, target slice

**Rules:** Target meaning, source revision/coverage and required qualification are explicit. Compatible units alone are insufficient: basis, datum, material identity, uncertainty semantics and task suitability are checked. Any lossy mapping or translation is declared.

**Multiplicity:** 0..n

**Definition:** An explicit transfer of a qualified upstream product into another task as a binding, numerical start, probability law, admissible set or other declared input, with index and physical-convention mapping.

**Layer:** Study

**Owned by:** [Study](study.md#source-40a141b5836981e78d53d04006644582)

**Concept:** Data edge

<a id="source-40a141b583698181a088ec443620bd67"></a>

## External sink

Source: [Notion](https://app.notion.com/p/40a141b583698181a088ec443620bd67).

**Refers to:** Node outputs

**Rules:** Every value sent is timestamped, unit-tagged and traceable to the Solution it came from.

**Multiplicity:** 0..n

**Definition:** A destination outside the Study that receives values: control moves sent to a plant, results published to another system.

**Layer:** Study

**Owned by:** [Study](study.md#source-40a141b5836981e78d53d04006644582)

**Concept:** External sink

<a id="source-40a141b583698125a09bfe024ab0017c"></a>

## External source

Source: [Notion](https://app.notion.com/p/40a141b583698125a09bfe024ab0017c).

**Refers to:** Datasets, slices

**Rules:** Every value is timestamped and unit-tagged.

**Multiplicity:** 0..n

**Definition:** A stream of values from outside the Study, such as a plant historian or live sensors, feeding edges.

**Layer:** Study

**Owned by:** [Study](study.md#source-40a141b5836981e78d53d04006644582)

**Concept:** External source

<a id="source-40a141b5836981aebb3fddc5e4ada788"></a>

## Generator

Source: [Notion](https://app.notion.com/p/40a141b5836981aebb3fddc5e4ada788).

**Refers to:** Problem template, edges

**Rules:** Loops declare state/termination and result-acceptance policies. Continuation states its branch/path and continuity evidence; a nearby starting guess alone does not guarantee branch tracking. Sampling for statistical aggregation declares its measure and estimator contract.

**Multiplicity:** 0..n

**Definition:** An explicit producer of Study nodes and data edges, such as sweeps, branch continuation, receding horizons, sequential experimental design or multi-start search.

**Layer:** Study

**Owned by:** [Study](study.md#source-40a141b5836981e78d53d04006644582)

**Concept:** Generator

<a id="source-40a141b5836981d3939dc18df0f6dcfe"></a>

## Problem node

Source: [Notion](https://app.notion.com/p/40a141b5836981d3939dc18df0f6dcfe).

**Refers to:** Problem template

**Rules:** Every required open input has one effective value/selection under an explicit overlay policy, with unexplained conflicts rejected. The node retains its model/case/task revision and independent outcome even when the Study is admitted as a whole.

**Multiplicity:** 1..n

**Definition:** One selected task occurrence in a Study: a concrete SimulationProblem or a template instantiated from declared data edges and literals.

**Layer:** Study

**Owned by:** [Study](study.md#source-40a141b5836981e78d53d04006644582)

**Concept:** Problem node

<a id="source-40a141b5836981d6ab02cabbb04b5b63"></a>

## Selector

Source: [Notion](https://app.notion.com/p/40a141b5836981d6ab02cabbb04b5b63).

**Refers to:** Node outputs

**Rules:** Selection states qualification/coverage requirements, tie handling and ordering. A finite best-of selection is not a global optimum certificate, and failed or incomplete results are never silently treated as valid observations.

**Multiplicity:** 0..n

**Definition:** A declared choice or aggregation over eligible task results, such as the best qualified multi-start candidate, a finite Pareto approximation or a source-filtered training dataset.

**Layer:** Study

**Owned by:** [Study](study.md#source-40a141b5836981e78d53d04006644582)

**Concept:** Selector

<a id="source-40a141b5836981e78d53d04006644582"></a>

## Study

Source: [Notion](https://app.notion.com/p/40a141b5836981e78d53d04006644582).

**Refers to:** Nothing

**Rules:** Ordinary data dependencies are acyclic; loops have explicit state, termination and data-transfer contracts. A study is neither one monolithic physical equation model nor barred from being a supported execution/workflow API.

**Multiplicity:** Root, or 0..n in a parent Study

**Owns:** [ProblemTemplate](problem-root.md#source-40a141b5836981688530f7b5250f2695); [Problem node](study.md#source-40a141b5836981d3939dc18df0f6dcfe); [Data edge](study.md#source-40a141b58369813c837ee8deb81d46b0); [Generator](study.md#source-40a141b5836981aebb3fddc5e4ada788); [Selector](study.md#source-40a141b5836981d6ab02cabbb04b5b63); [External source](study.md#source-40a141b583698125a09bfe024ab0017c); [External sink](study.md#source-40a141b583698181a088ec443620bd67)

**Definition:** A declared composition of semantic task nodes, data edges and explicit generators, such as operating-case sweeps, fitting experiments, continuation or receding horizons. Native orchestration may admit and schedule the study as one owned workflow while preserving each task's identity.

**Layer:** Study

**Concept:** Study
