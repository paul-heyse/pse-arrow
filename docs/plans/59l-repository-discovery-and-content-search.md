---
title: "Plan59l: Repository discovery and content search"
status: active
date: 2026-10-31
doc_role: plan
doc_owner: docs/plans/59-agent-cli.md
doc_topics: [current-work, design, libraries, diagnostics]
review_sources: [docs/functional_reviews/rust-centered-search-and-investigation-cli-2026-10-31.md]
---

# Repository discovery and content search

## Purpose and assessed foundation

**Proposed:** ordinary discovery, text and adjacent-content investigation through the common
query/result contract in [59k](59k-query-and-result-foundations.md). Own review O01/O02/O08/O09
provider work and supply demand/feedback facts for O06/O14. Baseline `9dd0f472a`; the review's
native observations remain current-capability evidence, not feature delivery. Core3.4 and
efficiency heuristics1.0 apply through the binding. The coordinator owns opportunity disposition.

Reuse `ignore`, `globset`, Cargo metadata, gix and existing Working/Index/Blob source owners.
Current exact-file source analysis does not supply whole-checkout discovery or arbitrary byte
search. Select files before parsing/analyzer work, without excluding valid candidates by an
unsound lexical prefilter. Repository selection is not limited to compiled modules or tracked files.

## Target and mechanisms

One source enumeration supplies native text, structural and inspection consumers. Working
filesystem enumeration respects ignore defaults and explicit overrides; index/revision enumeration
reads their own trees, including paths absent from the current checkout. Generated sources are
searchable under explicit selection while their edit protection remains. Explain exclusions and
source origin without running Cargo for simple path/text queries.

Select embedded `ignore::WalkBuilder`, `grep-regex`/`grep-matcher` and
`grep-searcher::Searcher`/`Sink` as the ordinary lexical realization: shared selectors, direct
structured callbacks and native early stop fit combined predicates and avoid an internal JSON
round trip. Verify exact pins/features before implementation. A managed `rg --json` adapter is
a fallback choice if a material promised capability is better delegated completely; record
the changed choice and use one ordinary lexical path, not two interchangeable implementations.
Optional PCRE2 must retain its actual semantics and build availability; never emulate it with
the default regex engine. No custom regex, ignore walker or universal index is planned.

Support literal/regex patterns, case/word/line/multiline options, context, inversion, counts,
paths and existence, with explicit byte/encoding/binary policy. Native multiple-pattern building
is alternation; file-level conjunction/negation has its own witnesses and EOF obligations.
Direct `inspect` supports exact content/ranges/head/tail and enclosing declaration/section where
appropriate. A plain byte scan need not hash or decode every entire file; exact hydration and
transformed-representation coordinates follow K02's guarantees. No-match is a successful empty
answer, not an execution failure. Permission/reader errors remain visible partial/error scope.

Git historical operations retain revision and heuristic limits. Reuse gix where its exact
operation fits; use managed Git for missing pickaxe/blame/line-history semantics rather than
reimplementing them. Text differences do not prove semantic correspondence across revisions.
For JSON/TOML/YAML, use existing parsers and typed field/predicate requests; JSON Pointer is
not a promise to implement every jq expression. Prefer existing dependency/test/configuration
meaning owners over scraping serialized files. PDF/OCR, arbitrary archives and binary analysis
are outside the initial target; existing artifact/log owners remain usable.

## Package state

| Package | Status | Owner | Completion |
| --- | --- | --- | --- |
| [L01](#l01) | planned | 59l | Shared source enumeration and direct exact-content inspection |
| [L02](#l02) | planned | 59l | Native lexical search with real demand/stop/error semantics |
| [L03](#l03) | planned | 59l | Structured adjacent content and historical investigation providers |

<a id="l01"></a>
### L01 — Select and inspect the actual universe

**Consumes:** K02 contracts and existing J01 source acquisition; A04 evidence locators.
Deliver workspace/package/path/type/glob/changed selectors with explicit tracked/ignored/
untracked/generated/dependency policy. Distinguish filesystem, index and resolved-tree
enumeration. Direct known-file inspection needs no prior result and no analyzer. Integrate
native owner/origin enrichment through D10 rather than create another ownership map.
**Acceptance:** a disposable dirty tree has ignored, staged, deleted, unusual-name and generated
files; a revision includes a path absent from working files. Selection, omission and direct
range inspection preserve their requested universe. Move duplicate public file-reading callers
and retire replaced frontends with D10/M03 once useful behavior is represented.

<a id="l02"></a>
### L02 — Search text through the native library pipeline

**Consumes:** L01 selection/inspection and K03 publication/demand. Compile matchers once, reuse
native scanning/sinks and share work for compatible clauses. Poll cancellation during long
no-match reads as well as callbacks; `WalkState::Quit` stops new visits while active workers
settle. Merge overlapping context windows, preserve witness spans and do not confuse line/file
Boolean scopes. Pattern errors retain native locations and useful correction context.
**Acceptance:** compare literal/regex/multiline/context results with independent native tool
expectations; distinguish engine unsupported, no-match, partial unreadable scope and real errors.
Existence stops with a witness; file-level negation finalizes only after sufficient examination.
Byte paths, zero-width matches, CRLF, binary detection and early consumer closure exercise their
actual contracts. Explicitly selected preprocessing has distinct searched/original coordinates.

<a id="l03"></a>
### L03 — Query structured and historical content

**Consumes:** L01 source selection/inspection, K02 locators and existing parser/Git/metadata
owners. Basic field and history operations do not wait for L02 text/streaming delivery;
selected text-backed composition consumes the working L02 producer.
Deliver typed JSON/TOML/YAML field selection and useful predicates, plus selected history/blame/
pickaxe operations. Settle which Git operations use gix versus native Git from exact capabilities.
Join matches to current owner/declaration inspection without assigning today's semantic context
to old bytes. **Acceptance:** changed string count versus changed matching lines differ, revision
reads include renamed/deleted paths, malformed configuration is not an empty result, structured
value locations retain known/unknown source ranges, and original input bytes are preserved.
Counterfactual dependency graphs and API meaning stay with H07/H08; this is not a second domain
interpreter or generalized shell/SQL engine.

## Verification and handoff

**Proposed verification:** focused unit/adapter tests plus public source→search→inspect journeys
at each package. M03/P02 own assembled discovery and displacement evidence; E06 moves skill and
advice with useful delivery. Do not require native HIR, a compiler-private driver, persistent
search storage or scientific qualification for lexical completion. No performance benefit is
Measured by authoring; use a small representative comparison only for claims made at execution.
