---
title: "Query: what is asked"
notion_source: "https://app.notion.com/p/40a141b583698107be4ccf187c25887f"
notion_last_edited: "2026-11-01T20:36:18.899Z"
---

# Query: what is asked

Source: [Notion](https://app.notion.com/p/40a141b583698107be4ccf187c25887f). Imported 2026-11-02.

**Version 3** · 2026-11-01

Query states the active computational intent: what may vary, what must be satisfied or assessed, what should be optimized or inferred, what regime is requested, and what outputs and evidence are required. Specification will often change alongside Query when the same physical model is reused.

## Active roles

| Role | Meaning in the active task |
| --- | --- |
| Specified | Supplied by an active binding |
| Unknown | To be determined by physical relations and active requirements |
| Decision | A free design/operating choice with a declared admissible domain and selection or optimization intent |
| Uncertain | Participates in an active probability or uncertainty treatment |
| Estimand | To be estimated from observations under a declared statistical/loss model |

Bound elements normally default to Specified and other elements to Unknown. Explicit assignments identify additional intent. Active interpretations must not contradict each other at the scalar-element level, while inactive knowledge about the same parameter may remain in Specification.

## Profiles, requirements, and evidence

The current profiles are Solve, Extremize, Expect, Infer, and Analyze. They are extensible contracts, not a ban on all combinations absent from a fixed matrix. See [Answer-kind contracts](../answer-kind-contracts.md).

Operating and design constraints are available in feasibility solves as well as optimization. Each condition declares enforcement, post-solve assessment, or advisory use. Data may supply inputs or validation comparisons without becoming an estimation task.

Regime is an intention checked against model capabilities. Required root/branch/stability selection is semantic. Accuracy requests identify whether they concern numerical convergence, estimated/certified numerical error, propagated uncertainty, or another defined evidence class.

Output requests are independent of role. Derivatives, statistics, or duals are returned only when their definitions and the selected transformations support them; unavailable information is reported explicitly.

## Concepts

[Catalog](../../catalog/simulation-concepts.md)
