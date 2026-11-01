---
title: "Plan60d: Change and document evidence"
status: active
date: 2026-10-31
doc_role: plan
doc_owner: docs/plans/60-engineering-code-intelligence.md
doc_topics: [current-work, design, documentation, diagnostics]
review_sources: [docs/functional_reviews/engineering-workflow-code-intelligence-2026-10-31.md]
adrs: []
---

# Change and document evidence

## Purpose and foundation assessment

**Proposed:** evidence for coherent change planning and code/document association, contributing
to [O04/O06](60-engineering-code-intelligence.md#opportunity-dispositions). Existing Git
acquisition/history, lossless syntax, original API comparison, generated-origin and native
document/lifecycle readers remain the producers. Current
`xtask/src/documentation/ownership.rs` explicitly disclaims inferring code-path-to-plan bindings;
new associations must not defeat that distinction by guessing.

The source review O04/O06 and blueprint §21.7 preserve origin and interpretation. Document
metadata/sections/lifecycle bindings remain canonical through
[the documentation guide](../dev/documentation.md#metadata-and-aggregate-scope). This plan adds
read-only views, not a new authority mapping database, editable status ledger or new publishing
system. Current plan/review/ADR status retains its original owner and meaning.

## Target design

A proposed change has several views: written source declarations/body differences, acquired
API differences and selected resolved relationships in explicitly prepared contexts. Align
observations with their sources and unknown/unmatched cases instead of rendering one blended
“semantic change” judgment. Git origins and source digests identify bytes; they do not certify a
semantic world for historical blobs. Names/ranges nominate cross-revision correspondence, not
identity. Renames, moves, macro gaps and cfg-only differences can make alignment ambiguous.

Cross-document associations carry their basis: declared ownership, explicit citation, resolved
Rust relationship or heuristic nomination. A source/generator declaration can lead to the exact
original generated origin; a section reference can lead to its native owner; a plan binding
can lead to a package status. A mere mention can nominate investigation without becoming a
binding. Preserve original date/revision/status and source qualification, not copied current
completion fields. A review recommendation remains a proposal and an accepted ADR remains
rationale rather than proof of implementation.

Reuse existing gix-backed origins and comparison adapters, lossless syntax/semantic owners and
document metadata/section parsing. Query-shaped joins and A03 progressive views are enough;
no whole-repository semantic rebuild, compiler execution or universal document graph is needed
for passive source/API comparison or explicit citation lookup.

## Work packages

<a id="d01"></a>
### D01 — Original source and API change views

Consumes A01 shared evidence and existing source/API comparison producers. Expose declaration,
body/structural and original API changes as distinct useful facts, with guarded before/after
inspection. Begin with data already available from Git and acquired artifacts. Preserve source
revision, actual artifact identity and unmatched/candidate correspondence. Do not require C01's
new graph merely to reuse delivered API comparison. Source-only questions remain useful with
no analyzer or API artifact prepared.

Acceptance covers a body-only change, rename/move, similarly named unrelated declarations,
missing before/after API artifact and a historical blob whose current source differs. A
written diff must not claim an unchanged semantic dependency when cfg selection has not been
compared. Tests use known before/after source/API records, with ambiguous mappings surfaced.
Migrate redundant changed-file assembly where the focused view replaces it, preserving
independent history/original-artifact inspection.

<a id="d02"></a>
### D02 — Qualified code, contract, plan and library associations

Consumes A01 plus working existing document/lifecycle/generated-origin readers; new B01 facts
can enrich resolved connections as available, but are not required for explicit citation and
ownership views. Deliver a change-planning question: show a selected source declaration,
authored generator/schema origin where declared, relevant contract sections, plan packages
and original qualification records, each with its actual connection basis.

Use native metadata/defaults/exceptions, scope-table identities and exact section resolvers;
retain the original record address instead of copying mutable state. Markdown heading/reference
parsing can nominate links. No guessed path-to-plan ownership is written back. Distinguish
generated-source origin from a surrounding file's prose mention. Library documentation remains
reference material with version/condition limits, not evidence that a capability is integrated.

Acceptance includes a declared owner, explicit source/section citation, actual package binding,
historical evidence and a type mentioned only in a rejected alternative. A misleading mention
cannot become ownership or proof. Include a moved section through its native resolver and a
missing/unresolvable reference retained as unknown. Remove duplicated document/status parsing
only when the view has moved consumers to the canonical interpreter. No new mapping registry
or manual source-to-plan inventory is part of completion.

<a id="d03"></a>
### D03 — Selected semantic-context comparisons

Consumes D01 and working B01 relations needed by a named change question. Require separately
and explicitly prepared before/after worlds, including actual Cargo/feature/target premises.
Do not prepare every workspace configuration or assign today's world to a historical blob.
Compare selected observations only after qualifying source/context correspondence; ambiguous
and missing observations remain unmatched/unknown. Reuse original native owners and A03
aligned output rather than a second semantic diff engine with invented stable identities.

Acceptance includes a cfg-only change that changes a resolved dependency with unchanged local
spelling, a rename with ambiguous correspondence, a missing world and incomplete macro input.
The public answer must explain which comparison was made and which contexts were not examined.
A single-world “no difference” cannot satisfy a two-world request. Migrate duplicate current-world
assumptions in affected comparison callers/tests as the replacement becomes usable. Explicit
preparation and source-only fallback remain separate supported choices.

## Package state

| Package | Status | Owner | Completion |
| --- | --- | --- | --- |
| D01 | planned | Source/API comparison owner | Qualified original differences and guarded before/after inspection |
| D02 | planned | Investigation/document bridge owner | Read-only associations preserving native authority, basis and status |
| D03 | planned | Native context comparison owner | Public selected-world comparison with cfg and ambiguous mapping acceptance |

## Verification and handoff

**Proposed:** targeted comparison and association fixtures with positive and misleading joins,
plus actual public change-planning use in E03. Documentation interpreter tests should assert
original declared meaning; a duplicate heuristic expected answer is insufficient. Q01 owns
assembled/static qualification and architecture publication if new enduring contracts require
it. No document immutability exception, new lifecycle vocabulary or live plan-status copy is
authorized by this plan.
