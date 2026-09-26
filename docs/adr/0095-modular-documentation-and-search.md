---
id: ADR-0095
title: Publish focused architectural documents with generated navigation and static search
status: accepted
date: 2026-09-25
deciders: [paul-heyse]
level: decision
principles: [AP-01, AP-02, AP-04, AP-06, DP-23]
blueprint: [§0.1, §0.3, §24.4]
review: git:8950dd3d6ddb:docs/design_review/reviews/design_review_documentation-system_2026-09-25.md
evidence: Proposed
supersedes: [ADR-0033, ADR-0036]
superseded-by: null
revisit: A documented search or publishing limitation cannot be resolved through collection configuration or the existing tools without substantial bespoke machinery
verification: Plan 18 publisher fixtures, real mdBook/Pagefind build, offline fragment checks and browser search scenarios
---

# ADR-0095: Publish focused architectural documents with generated navigation and static search

## Context

The book mixes current guidance with extensive execution history and reference material.
Its built-in search produces a roughly 26 MB index, while manual navigation and the single
blueprint file increase reading and maintenance costs during frequent design changes.

## Scope

Documentation authority, section identity, publication, search and contributor guidance.
Supersedes ADR-0033 (one blueprint file) and ADR-0036 (a committed `SUMMARY.md` with a
generated ADR block); both are retired to Git history under ADR-0096. Existing section
identifiers remain intact. Implementation was authorized and then accepted by the maintainer.

## Drivers

Localize document changes, provide bounded context, preserve one owner per contract and
avoid a second implementation maintained solely to prove architectural conformance.

## Options

Retain the existing search and increase its warning threshold: rejected because the loading
and retrieval problem remains. Migrate the entire site to another generator: unnecessary
migration cost for the present requirements. Keep mdBook and compose collection discovery,
Pagefind and focused Markdown: selected.

## Outcome

Plain Markdown remains canonical. A small standard-library Python adapter stages declared
collections, derives book navigation, invokes mdBook, annotates canonical content and invokes
Pagefind. `docs/site.toml` owns publication choices and tool versions. Current, Reference and
History search scopes distinguish reading purposes; Everything searches all indexed chapters.

The blueprint and extracted pages under `authoritative_design/sections/` form the authoritative
collection. Numbered headings own stable section identities. The existing ADR resolver checks
unique ownership across that collection; old locations retain anchors and links, not duplicate
normative text. Initial extraction is limited to reading/governance sections.

### Consequences

Adding a document within a declared collection needs no manual book entry. The committed ADR
index remains generated from records. Current work is explicitly selected rather than inferred
from stale plan lifecycle fields. Optional rustdoc remains independent of product compilation.

### Compensating controls

Check discovery, title availability, section identity, links and successful rendering/indexing.
Architecture remains an agent/reviewer judgment using relevant source and product tests.
No proof manifest, symbol inventory, source seal or automatic architectural approval is added.

### Confirmation

[Plan 18](https://github.com/paul-heyse/pse-arrow/blob/8950dd3d6ddb3aa7c78acc0db7d0601497b302a8/docs/plans/18-architectural-documentation.md) owns implementation and verification.
Tests establish publication behavior, not design-review effectiveness or product qualification.

## Pros and cons

Retains readable source and existing URLs with a small tooling surface. Search and rendering
remain replaceable. The bounded adapter and two tool pins need maintenance; mixed legacy
blueprint content remains Reference until its sections move with substantive subsystem work.

## More information

[Architecture entry](../authoritative_design/README.md),
[Pagefind](https://pagefind.app/docs/), blueprint §§0.1, 0.3 and 24.4. ADR-0096 extends this
decision from incremental extraction to full consolidation of the current architecture.

## Status history

- 2026-09-25 — proposed; maintainer authorized implementation, formal decision PR pending.
- 2026-09-26 — accepted by the maintainer as implemented through Plans 18 and 19 (review
  verdict Accept); supersedes the retired ADR-0033 and ADR-0036.
