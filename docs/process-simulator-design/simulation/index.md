---
title: "Unified Simulation Domain Model"
notion_source: "https://app.notion.com/p/40a141b5836981c882fbf0449dfbcd7a"
notion_last_edited: "2026-11-01T23:33:57.396Z"
---

# Unified Simulation Domain Model

Source: [Notion](https://app.notion.com/p/40a141b5836981c882fbf0449dfbcd7a). Imported 2026-11-02.

> The **SimulationProblem** (Model + Specification + Query, plus an optional MethodRequest) is an immutable submission envelope over separately owned definitions, case data and analysis intent. The general framework supports the flowsheet process domain through declared contracts and capabilities. It does not require the process front end to implement every simulation discipline. Model assumptions and engineering meaning remain explicit; execution records numerical choices and qualified evidence.

This hub is the authoritative structured overall domain model: **v3, current**, 2026-11-01. The narrative pages and database rows incorporate the adopted process-domain review. Existing page and concept identities are preserved.

The dedicated [Flowsheet Process Domain Model](../process/index.md) owns process-specific composition, material/state interfaces, equipment, inventories, physical transfers and operating requirements. This hub owns the reusable formulation/execution framework. The repository's authored definitions remain the executable model authority; these Notion pages are design guidance and do not claim implementation or scientific parity.

Revision record: [Unified Simulation Domain Model — v3](../rationale/version-records.md#source-40b141b583698133b0b5f31fc2d3a728).

The process companion is now [Flowsheet Process Domain Model — v1.1](../rationale/version-records.md#source-40b141b5836981958b9ad5a14519520f). Its material, population, lot/procedure/campaign, capability and assessment contracts refine process-specific meaning over this same framework. Physical campaigns remain distinct from computational Study orchestration; the [Implementation design and IDAES equivalence plan](../delivery/index.md) owns the delivery and qualification scope.

## Reading order

1. [Overview](overview.md): scope, responsibilities, modeling distinctions, and the eight design principles.
2. [Layers](layers.md): one page per layer (Vocabulary, Model, Specification, Query, Study, Solution), each with its concepts.
3. [Hierarchy and references](hierarchy-and-references.md): the ownership tree and the cross-layer reference rule.
4. [Answer-kind contracts](answer-kind-contracts.md): the five supported task profiles, composable formulation requirements, and capability-based admission.
5. [Formulation and execution boundary](formulation-and-execution-boundary.md): what crosses, the gate checks, the API shape, results and diagnostics.
6. [Execution](execution.md): kernels, canonical forms, and planning without a central router.
7. [Worked examples](examples/index.md): process examples and clearly marked deferred integration illustrations, with qualified model-substitution and equilibrium claims.

For a quick decision on where a new piece of information belongs, use the [Placement guide](placement-guide.md).

## Reference databases

| Database | One row per | Key links |
| --- | --- | --- |
| Concepts | General formulation/execution concept | Layer; allowed owner types and references; semantic obligations and assessment limits |
| Answer kinds | Solve, Extremize, Expect, Infer, Analyze | Contract fields; Task classes |
| Task classes | Task profile or deferred extension illustration | Answer kind; capability conditions and scope |
| Canonical forms | Mathematical problem class exchanged by kernels (16) | Produced by, consumed by |
| Kernel kinds | Kind of execution kernel (5) | Takes, produces |

[Concepts](../catalog/simulation-concepts.md)

[Answer kinds](../reference/answer-kinds.md)

[Task classes](../reference/task-classes.md)

[Canonical forms](../reference/canonical-forms.md)

[Kernel kinds](../reference/kernel-kinds.md)

## For agents

> Query the data sources directly rather than parsing pages. Concepts: `collection://54feff54-c516-4bc0-bd43-e95d7c613f53` (fields Concept, Layer, Definition, Owned by, Owns, Multiplicity, Refers to, Rules). Answer kinds: `collection://36bb7561-98d9-45e9-b715-baf91adac20e`. Task classes: `collection://6e4787a7-1f73-426c-ad03-502c291e02ec`. Canonical forms: `collection://95c3ec74-61d8-46b3-b0de-31d66415a7c0`. Kernel kinds: `collection://866ba763-3a17-4277-8152-f939aca5ceef`. Owned by names allowed owner types for each concept; a concrete occurrence has one containment owner. Refers to is text naming other concepts by their Concept title. Layer values: Cross-cutting, Problem root, Vocabulary, Model, Specification, Query, Study, Solution, Execution.

## Changing the model

Edit the structured pages and database rows here directly; they are the source of truth. Record each consequential choice in the Decisions log on the home page. The v2 working document remains a historical source. Current v3 relationships and contracts are maintained in these structured pages and the linked process-domain design.

## Pages

[Overview](overview.md)

[Formulation and execution boundary](formulation-and-execution-boundary.md)

[Layers](layers.md)

[Hierarchy and references](hierarchy-and-references.md)

[Answer-kind contracts](answer-kind-contracts.md)

[Execution](execution.md)

[Worked examples](examples/index.md)

[Placement guide](placement-guide.md)
