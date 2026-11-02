---
title: "Cross-cutting concepts and the problem root"
notion_source: "https://app.notion.com/p/40a141b5836981bc9d8dde87f00b4cb2"
notion_last_edited: "2026-11-01T20:36:16.246Z"
---

# Cross-cutting concepts and the problem root

Source: [Notion](https://app.notion.com/p/40a141b5836981bc9d8dde87f00b4cb2). Imported 2026-11-02.

**Version 3** · 2026-11-01

The shared concepts are identity paths, references, index selections, slices, expressions, values, and annotations. A Slice identifies a quantity and selected indices; it is the target of bindings, roles, constraints, and output requests.

## Identity and reference

A readable path such as `plant.R101.energy_balance` is distinct from a stable semantic identifier. Renaming or reparenting changes the path while stable identity and template/instance lineage preserve traceability. A typed Reference resolves identity and target kind; public component interfaces prevent callers from depending on private ownership details.

Selections carry their scalar cardinality, coordinate meaning, and scope. A vector slice is not one degree of freedom merely because it is addressed by one reference.

## Values and annotations

Physical values carry kind, unit, basis, and index meaning as appropriate. Labels, identifiers, categories, and other non-physical metadata use their own types and do not require invented physical units. Trajectories and fields declare coverage, interpolation, discontinuities, and undefined regions.

Only documentary metadata—such as prose, authorship, and display tags without behavioral effect—is non-semantic. A validity requirement, bound, semantic assertion, or required analysis remains part of the semantic request even if represented as metadata.

## Problem roots and hashes

SimulationProblem contains one Header, Vocabulary, Model, Specification, and Query, plus an optional MethodRequest. The submitted snapshot is immutable; versioned definitions and external data dependencies are pinned or marked unresolved.

The semantic hash identifies the physical/case/question meaning and excludes clearly non-semantic guesses and documentary annotations. A Guess never carries branch selection. The submission hash adds guesses and execution requests. Each Run is separately identified; hashes alone do not promise bitwise reproduction.

ProblemTemplate exposes open inputs and outputs for Study nodes and sub-problem realizations. Study remains outside the atomic problem, and Solution records an actual outcome rather than becoming part of the reusable physical model.

## Concepts

[Catalog](../../catalog/simulation-concepts.md)

[Catalog](../../catalog/simulation-concepts.md)
