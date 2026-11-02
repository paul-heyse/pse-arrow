---
title: "pse-arrow Design"
notion_source: "https://app.notion.com/p/40a141b5836981c9b83eeff8209ba9c0"
notion_last_edited: "2026-11-01T23:33:56.466Z"
---

# pse-arrow Design

Source: [Notion](https://app.notion.com/p/40a141b5836981c9b83eeff8209ba9c0). Imported 2026-11-02.

> Design planning for the flowsheet process-modeling front end of [pse-arrow](https://github.com/paul-heyse/pse-arrow), supported by its general formulation, numerical execution and results infrastructure. This space holds the design contracts, rationale and decisions; the repository's authored definitions remain the executable authority.

## Site map

| Area | What it holds | Start at |
| --- | --- | --- |
| Flowsheet Process Domain Model · v1.1 | Equipment composition, materials and states, physical coupling, operating cases, engineering analyses and IDAES acceptance workflows | [Flowsheet Process Domain Model](process/index.md) |
| Implementation and IDAES equivalence | Adopted P0–P7 program, coding-agent packets, scientific qualification and industry expansion direction | [Implementation design and IDAES equivalence plan](delivery/index.md) |
| Unified Simulation Domain Model · v3 | Reusable Model / Specification / Query framework, atomic tasks, Study orchestration, execution and qualified results | [Unified Simulation Domain Model](simulation/index.md) |
| Decisions | Design questions, adopted outcomes, rationale and version links | [Decisions](rationale/design-decisions.md) |
| Version records and historical snapshots | Revision history, previous source snapshots and links to the live structured designs | [Design documents](rationale/version-records.md) |

Original adoption records, 2026-11-01: [Unified Simulation Domain Model — v3](rationale/version-records.md#source-40b141b583698133b0b5f31fc2d3a728) and [Flowsheet Process Domain Model — v1](rationale/version-records.md#source-40b141b583698185aee7fac52f7213f2).

Current revision records, 2026-11-01: [Unified Simulation Domain Model — v3](rationale/version-records.md#source-40b141b583698133b0b5f31fc2d3a728), [Flowsheet Process Domain Model — v1.1](rationale/version-records.md#source-40b141b5836981958b9ad5a14519520f) and [Flowsheet implementation and IDAES equivalence plan — v1](rationale/version-records.md#source-40b141b5836981ae9187c0bc93c10aeb). The process v1 record remains available as history. Design adoption and completed implementation are tracked separately.

## How this space works

- **Structured pages are the live design authority.** Each area combines narrative pages explaining relationships with queryable concept and reference databases. Maintain both when a contract changes.
- **Executable ownership stays in the repository.** Process-facing views, cases and packages reuse the authored definitions and semantic identities. A Notion contract does not by itself establish implementation or scientific parity.
- **Decisions outlive versions.** Record consequential questions and their status in the Decisions log. Preserve earlier rationale when a later decision refines it. Open items remain proposals; adopted directions link to their version records.
- **Evidence is explicit.** Separate proposed behavior, inspected source, executed acceptance checks and reference comparisons. Retain exact versions, assumptions and limitations with a claim.
- **Drafts become structured design.** Drafting may happen elsewhere. Fold adopted changes into the relevant pages and catalogs, retain the history, and identify the current version.

New design areas belong under their own hub with clear ownership and links to existing contracts.

## Decisions

The three open v2 questions about bounds, definition ownership and regime were resolved with qualifications in v3. The v1.1 process refinement adopts explicit cross-industry material/operating contracts and bounded implementation guidance through three additional decisions. Earlier lifecycle and execution decisions remain preserved.

[Decisions](rationale/design-decisions.md)

## Version records and historical snapshots

The v1 and v2 overall source documents are preserved as historical snapshots. New version records describe the adopted changes and point to the live structured pages; they are explicitly identified as revision records rather than complete frozen snapshots. The [Claude v2 source document](https://claude.ai/artifact/J5oKwfVpUTfcyEFYAnEEZJ) remains a historical drafting source.

[Design documents](rationale/version-records.md)

## For agents

> Decisions: `collection://9f465a6b-9cb5-4152-83be-bd906b8e0e42`. Design documents: `collection://496a819c-3260-47ce-a473-5389bb577242`. Each domain hub lists its own data sources. Query structured properties directly; use narrative pages for context. Preserve native child pages, databases, semantic identities and historical records when editing.

## Design hubs

[Flowsheet Process Domain Model](process/index.md)

[Unified Simulation Domain Model](simulation/index.md)
