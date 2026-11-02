---
title: "Specification: what is known"
notion_source: "https://app.notion.com/p/40a141b58369819695e4f85322f6a5eb"
notion_last_edited: "2026-11-01T20:36:18.091Z"
---

# Specification: what is known

Source: [Notion](https://app.notion.com/p/40a141b58369819695e4f85322f6a5eb). Imported 2026-11-02.

**Version 3** · 2026-11-01

Specification records what is known or assumed about the operating case: parameter values, feed conditions, initial and boundary data, selected physical realizations, observations, uncertainty information, and the source of operating requirements.

It does not erase bounds or limits merely because they are knowledge. Physical support remains in Model, property validity in the realization contract, and equipment ratings retain their engineering source. Query declares which case requirements are enforced, assessed after solving, or advisory, and supplies task-specific decision domains.

## Scenarios and values

A Scenario is one declared case or replication. Completeness is checked against the active Query and supported regime rather than against every conceivable use of the model. Scenario structure identifies shared quantities, local quantities, probabilities/weights where relevant, and temporal or experimental alignment.

A Value source may be a literal, dataset, databank, or upstream Solution. Values and required source versions are frozen for submission. Dataset identifiers, experiment labels, and timestamps are typed metadata; only physical columns require physical kind/unit conventions.

## Guesses and uncertainty

Guesses guide numerical initialization and are non-semantic. A desired stable phase, continuation branch, or other solution selection belongs in Query and cannot be hidden in a guess.

A distribution may be retained as case knowledge even when the current task uses a fixed nominal value. The active Query distinguishes a prior used for inference, an input measure propagated through the model, a posterior passed onward, or inactive uncertainty information. Simultaneously active interpretations must be consistent.

Observation models align predictions with data and specify measurement/error assumptions. They can support estimation or validation-only comparison; merely using a dataset does not imply an Infer task.

## Concepts

[Catalog](../../catalog/simulation-concepts.md)
