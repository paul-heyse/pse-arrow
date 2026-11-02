---
title: "Study: composing problems"
notion_source: "https://app.notion.com/p/40a141b583698111960cfbb8483dd942"
notion_last_edited: "2026-11-01T20:36:19.761Z"
---

# Study: composing problems

Source: [Notion](https://app.notion.com/p/40a141b583698111960cfbb8483dd942). Imported 2026-11-02.

**Version 3** · 2026-11-01

Study composes atomic SimulationProblems through problem nodes, aligned data edges, generators, selectors, and external sources/sinks. It remains outside the atomic problem boundary. A Study may submit problems sequentially or in batches through the same execution interface.

## Composition contracts

A data edge states what its output becomes downstream: a binding, a guess, uncertainty information, or an admissible domain. It also declares units/basis conversion, index alignment, source version, coverage, and failure behavior. A posterior does not automatically become a set of independent marginal distributions.

A sub-problem realization is appropriate when an inner computation defines an evaluation needed by an outer formulation. Its derivative and solution-selection capabilities are explicit. A Study edge is appropriate for staged data transfer; derivative availability alone is not the sole distinction.

Generators include sweeps, continuation, receding horizon, outer iteration, and multi-start. Each iteration has termination, retry, partial-result, and synchronization semantics. Ordinary data dependencies are acyclic; declared iterative generators own their feedback behavior.

## Reuse and external interaction

A repeated control or estimation problem may reuse a compiled structural plan even when its case data and semantic hash change. Warm starts remain guesses; selected operating branches and acceptance rules remain semantic intent.

External sources are timestamped and aligned with the modeled horizon. External sinks describe intended destinations and contracts. Their presence in a design example is not authorization to actuate equipment; deployment requires the application's explicit operational authorization, validity checks, and fallback behavior.

Sweeps and uncertainty propagation can share execution mechanisms but have different declared meanings. A sweep asks for specified cases; Expect reports statistics under a declared measure.

## Concepts

[Catalog](../../catalog/simulation-concepts.md)
