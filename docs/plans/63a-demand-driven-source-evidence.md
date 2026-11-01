---
title: "Plan63a: Demand-driven source evidence"
status: proposed
date: 2026-11-01
doc_role: plan
doc_owner: docs/plans/63-code-intelligence-cli-robustness.md
doc_topics: [current-work, design, libraries, validation]
review_sources: [docs/functional_reviews/code-intelligence-cli-robustness-2026-11-01.md]
adrs: [ADR-0235]
---

# Demand-driven source evidence

## Purpose and baseline

**Proposed:** implement O01 and source-local O08 obligations. The
[coordinator](63-code-intelligence-cli-robustness.md) owns baseline, shared decisions and disposition.
This companion owns A01–A03 and their local state.

`source_read::rust_parsed` pages entries but then collects all file facts.
`source_command::show_addressed` collects facts before range filtering.
`source_read/facts.rs` copies written/decoded text and constructs per-character mappings.
Existing exact acquisition, parsing, coordinates and guarded pages remain useful; the defect
is when and how much interpretation happens, not a missing parser or evidence service.

## Target and library-backed realization

Resolve source-detail demand before extraction. Navigation, fact interpretation and decoded
mapping are progressively richer products of the same exact source owner. Adopt a typed
detail selection shared by outline/declaration dispatch and their existing internal callers:
navigation, facts, mapped. Public naming may follow parser conventions; these three meanings
must be explicit in the schema/help. Ordinary outlines default to navigation; declaration
inspection defaults to selected facts without character mappings. Ordinary exact-range text
inspection remains original text rather than an automatic request for decoded facts.

Navigation returns identity, edition/backend, declaration hierarchy, selection ranges and parse
issues with truthful coverage. It does not copy every initializer, documentation body or literal
mapping into a declaration signature. Use syntactic header/navigation ranges for declarations;
large selected source remains expandable through the existing guarded source locator.
Fact demand selects declaration/range ownership before traversal and decoding. Mapped demand
adds escape-aware correspondence only for selected interpreted values.

Return the actual selected detail and its omission/coverage meaning through the existing
response owner. Not requested/not constructed is distinct from examined and absent. Keep
detail coverage independent of declaration paging and parse-error coverage so an empty facts
or mapping collection cannot silently imply a complete examination.

Root docs have explicit file ownership: file-fact inspection includes them; selected declaration
facts do not silently attach unrelated root docs. Include owned attributes/docs with a selected
declaration even when their syntax lies before its name. A raw range selects facts intersecting
that range and returns their complete original ranges, not clipped invented facts. Nested
declaration facts retain their actual owner; do not duplicate them once per ancestor.
Malformed syntax/literals retain written evidence and interpretation errors.

Library realization uses existing `ra_ap_syntax =0.0.360`:

| Mechanism | Design role and conditions |
|---|---|
| Typed AST/navigation ranges and `Parse::errors` | Determine selected syntax and original errors; no semantic or compiler-validity claim |
| `SyntaxText::slice` and chunk traversal | Inspect/hash selected text without requiring whole-subtree strings; chunks are not token boundaries |
| `ast::IsString::escaped_char_ranges` | Requested decoded-character mappings, including escapes and multibyte characters |
| Existing source acquisition/digest owner | Exact original bytes and guards, rather than reconstructing authority from pretty-printed tokens |

See [syntax reference](../libraries/rust_analyzers/ra_ap_syntax.md), original §§3–4 and addendum
§§1–3. Qualify exact signatures with pinned source before coding. Quote-relative `map_range_up`
does not replace decoded escape mapping. `ra_ap_ide::file_structure` is an alternative only for
already prepared worlds; it would introduce unnecessary preparation for this source-only path.
Syn/quote/pretty-printing must not become original-coordinate authority.

Projection-only deletion reduces wire bytes after doing the work and is insufficient. A new
persistent lazy-result service or compressed mapping format is unnecessary for selected extraction.
Do not add a generic encoded-byte cap as a substitute for reducing unwanted detail. If a
remaining large selected record requires a cap, first design its explicit omission/expansion
contract with the coordinator; never truncate source witnesses silently.

## Packages

### A01 — Detail demand and migrated first consumers

Inventory all `Outline.facts` consumers, including associations/documentation extraction,
source queries, declaration/literal inspection, batch outlines and tests. Introduce the
selected demand at the existing source reader; dispatch each caller according to the facts
it actually consumes. Add parser-owned detail selection and generated public schema/help.
Preserve existing `--fields` as payload projection and `--budget` as record allowance.

Continuations bind source/backend/edition and detail selection. A continuation cannot silently
switch interpretation; changed source or incompatible detail refuses. Coordinate root-doc and
attribute behavior with callers. Remove the unconditional fact-extraction path and migrate its
callers/tests in the same coherent change; no eager compatibility branch.

Acceptance: navigation outline works without semantic preparation; explicit facts/mapped
selection works; association doc extraction still works; changing detail on a guarded page
cannot return mismatched evidence.

### A02 — Selective extraction and bounded navigation content

Select node/range and requested detail before copying/decoding/mapping. Traverse only relevant
subtrees where possible while preserving correct ownership and parse-error scope.
Do not build all mappings and filter afterward. Ensure batch invocation carries demand into
each source task; cancellation retains the existing actual task owner.

Acceptance: one selected declaration beside a huge unrelated string/doc does not construct
those facts or mappings. A one-entry outline does not copy unrelated initializers into its
navigation payload. Explicit mapped inspection retains exact escaped ASCII, Unicode, newline
and malformed-literal evidence. Root docs and nested owners have independent expectations.

### A03 — Source contract and regression assurance

Extend `source_read` unit tests and host `source_addressing`/`investigation_evidence` journeys
with independently specified source ranges and decoded expectations. Check omitted detail and
avoided mapping construction, not only a smaller serialized result. Use test-local observation
or a controllable extraction seam rather than a production instrumentation subsystem.

Compare original-source/result forms, detail projections, current/stale cursors and actual JSON
schemas. Keep source-only ordinary JSON distinct from proto-native representations. Add
source-local O08 cases for explicit defaults, unknown detail values, invalid combinations and
large selected values. Document the changed default and explicit expansion in B04 examples.

## Package state

| Package | Status | Owner | Completion |
|---|---|---|---|
| A01 | open | Source evidence | Shared demand, parser/schema and migrated first consumers |
| A02 | open | Source evidence | Selective construction and correct ownership/mappings |
| A03 | open | Source evidence | Revealing unit/CLI/schema controls and default guidance |

## Verification and handoff

**Proposed:** compile checks and targeted source-reader unit tests accompany functional changes,
with actual correctness selection. Author host source-addressing/association controls alongside
their mechanisms; execute the assembled journeys once in Q01 at functional scope end.
Compare outline bytes for the
review's file plus self-contained doc/string-heavy fixtures. No exact old byte count is a
portable assertion; fixture output must be independent of unrelated source detail.
Latency/allocation benefits remain unmeasured unless a selected experiment establishes them.
Integrated/static checks belong to Q01, not every package.

**Interface-checked:** relevant owners and critical library mechanisms have been inspected.
No implementation or new test execution is claimed. B01/B03 supply shared failure projection
if new detail refusals need it; A01 does not wait for the entire recovery companion.
