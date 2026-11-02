---
title: "Layers"
notion_source: "https://app.notion.com/p/40a141b583698177bc1cebdcb8588f2f"
notion_last_edited: "2026-11-01T20:36:15.516Z"
---

# Layers

Source: [Notion](https://app.notion.com/p/40a141b583698177bc1cebdcb8588f2f). Imported 2026-11-02.

**Version 3** · 2026-11-01

The general formulation is organized into reusable definitions, physical model instances, case knowledge, and task intent. Study composes atomic problems; Solution and execution provenance record their outcomes.

| Layer | Main question | Responsibility |
| --- | --- | --- |
| Cross-cutting and problem root | How is a request identified and packaged? | Stable identity and human paths, references, slices, expressions, values, annotations, Header, SimulationProblem, MethodRequest, ProblemTemplate |
| Vocabulary | What definitions does this model use? | Versioned units/kinds, material identities, port types, constitutive definitions, component templates |
| Model | What physical system and assumptions are represented? | Component instances, quantities, relations, domains, ports, capabilities, alternatives |
| Specification | What is known about this case? | Parameters, feeds, initial/boundary data, datasets, observations, distributions, case requirements, realization selections, guesses |
| Query | What should this computation find or establish? | Roles, task profile, constraints and assessment policy, objectives, regime, outputs, evidence requests |
| Study | How do several problems work together? | Problems/templates, data alignment, sweeps, continuation, outer loops, receding horizons |
| Solution | What was found and how strongly is it supported? | Values, members, estimates, statistics, diagnostics, evidence, reconstruction and provenance |
| Execution | How was the request transformed and computed? | Available capabilities, plans, runs, numerical forms, settings, reconstruction maps |

Cross-layer dependencies flow from reusable definitions toward use. Same-layer references are allowed, and physical connectivity may contain recycles; ownership and expression-definition cycles are validated separately. See [Hierarchy and references](hierarchy-and-references.md).

These layers describe the general problem envelope. The flowsheet process domain gives engineering meaning to the equipment, material states, balances, and connection contracts represented within them. Broad molecular and full-field concepts in the catalog remain extension vocabulary.

## Layer pages

[Cross-cutting concepts and the problem root](layers/cross-cutting-and-problem-root.md)

[Vocabulary: what things are](layers/vocabulary.md)

[Model: what is true](layers/model.md)

[Specification: what is known](layers/specification.md)

[Query: what is asked](layers/query.md)

[Study: composing problems](layers/study.md)

[Solution: what comes back](layers/solution.md)
