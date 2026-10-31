---
title: "Plan59m: Composed Rust investigations"
status: active
date: 2026-10-31
doc_role: plan
doc_owner: docs/plans/59-agent-cli.md
doc_topics: [current-work, design, libraries, diagnostics, validation]
review_sources: [docs/functional_reviews/rust-centered-search-and-investigation-cli-2026-10-31.md]
---

# Composed Rust investigations

## Purpose and foundation assessment

**Proposed:** agents start with a name, partial shape, known location or original diagnostic,
then refine and inspect the useful result through `search`, `inspect` and `propose`. This owns
composed public consumers of review O03/O04/O07/O10/O11/O12; G/H/I/J retain their backend and
proposal contracts. Baseline `9dd0f472a`; existing tested source/semantic/API/compiler features
are reusable foundations, not proof of these new interactions. Core3.4 and efficiency
heuristics1.0 apply through the binding; no new scientific authority is introduced.

Current position-based LSP, API artifacts, original compiler evidence and native syntax queries
have different context/fidelity. Keep those distinctions through joins. A call spelling is not
a resolved target; API input/output string containment is not structural type equivalence;
macro invocation fallback is not exact token origin; an affected or related test is not coverage.
These are reasons for typed composition under K01/K02, not for flattening every provider to text.

## Target interactions

Name-first discovery returns qualified candidates before it requires a known source position.
Support simple/qualified name, package/module/kind, visibility and signature filters where the
selected provider supplies them. Exact/prefix/fuzzy candidates retain ambiguity rather than
silently selecting one same-spelled entity.
Use source declarations without a session, workspace symbols through G10 when selected, and
acquired API graphs through H07. Inspect ambiguity, context and one useful neighbor before
expanding further. Candidate identity includes actual source/provider context, not display name.

Compose library-native predicates and bounded typed relations over candidates. Text and syntax
can narrow semantic work only when the reduction preserves the question; a global reference
search cannot silently become references only in lexical candidate files. Bind capture environments
deliberately and preserve many-to-many source/expansion mappings. File/node/entity conjunctions
consume K01 meaning without a separate M-specific query language.

`inspect` accepts direct source/index/revision positions/ranges and typed evidence locators:
original diagnostics, logs/artifacts, retained test populations and owner/generated-origin facts.
It can request selected richer facts, but it does not implicitly prepare macros, acquire docs,
compile, run examples or execute analyzer-supplied tests. Test enumeration requiring compilation
remains the explicit native `tests` workflow. Existing effectful/control/schema/help commands
remain adjacent, rather than being hidden inside search modes.

Selected results feed J07 checked proposals. Syntax rewrites preserve unchanged trivia; native
semantic operations retain imports, snippets, file operations and overlap policies. New generated
fragments may use syn/quote/prettyplease; maintained files are not regenerated wholesale.
Application remains separately assessed and unselected; useful preview completion does not
require a Git transaction/commit feature or an apply implementation.

## Package state

| Package | Status | Owner | Completion |
| --- | --- | --- | --- |
| [M01](#m01) | in-progress | 59m | Source discovery, workspace symbols, API and direct inspection consumers implemented; complete provider/native public journeys pending |
| [M02](#m02) | in-progress | 59m | Native structural composition and guarded semantic/API consumers underway; positive G11 feasibility selects G12 immutable native snapshot implementation |
| [M03](#m03) | in-progress | 59m | Proven source/evidence routes migrating and obsolete public routes retiring; eight assembled journeys, adoption and final qualification remain |

<a id="m01"></a>
### M01 — Find an entity without knowing its file

**Consumes:** K02, L01, G10 and H07's base artifact discovery slice. Reuse existing outlines,
working/index/revision bytes, native workspace-symbol responses and acquired APIs. A missing
provider is not a complete empty answer or an automatic text fallback. Requests can explicitly
select source-only discovery. **Acceptance:** same-spelled unrelated trait methods, a reexport,
dependency item, inactive cfg declaration and ambiguous qualified name retain distinct identities;
inspect a known file directly and follow one selected symbol without reconstructing coordinates.
Basic name-first discovery does not wait for G11/G12 native HIR migration.

<a id="m02"></a>
### M02 — Ask a composed Rust question

**Consumes:** M01, K03, L02 and J06. H07/G10 supply base facts; additional typed HIR facts consume
G12 only if selected after G11/A11. Preserve supported base composition when a conditional stage
is unavailable. Native compiler facts consume I07 only when I06/A12 select that branch; opaque
existing artifacts cannot be relabelled as typed borrow/dataflow results.
Deliver file-level and structural grouping, resolved-target refinement, useful API/use/importability
joins, generated-origin drill-through and selected diagnostic/hint/candidate expansion.
New L03 history operations join these results in M03; base M02 does not wait for history delivery.
**Acceptance:** two clause witnesses in one file differ from one-node matching; one same-spelled
call resolves differently; original compiler failure and current semantic observation remain
separate. An API candidate retains package/reexport and destination-context limits. Unknown
generic predicates, cancellation and stale candidates cannot become contradictory/no-match facts.
G11/I06/H08 explicitly decide native snapshot, SSR-accessor, graph and driver branches; branch
non-adoption must identify lost capability and a useful revisit trigger, not claim its delivery.

<a id="m03"></a>
### M03 — Complete the public investigation and retire replaced paths

**Consumes:** M02, L03, J07, C10 and D10; conditional selected backend branches retain their own
acceptance. E06 follows each usable public slice; its final adoption and P02 qualification
follow this assembled producer. Do not make an E06/M03 dependency cycle.
Move selected frontends and ordinary agent callers to the consolidated hierarchy. Maintain a
capability mapping for passive dependency/features/units, owner/generated origin, diagnostics,
retained logs/artifacts and test intent. Keep distinct preparation/execution/control routes.
Delete replaced query/read branches, callers, obsolete fixtures and tests in the same proven
slice; keep no compatibility aliases or duplicate production interpreters merely as evidence.

**Acceptance:** unfamiliar subsystem→name→declaration→reference; mixed Rust/TOML/Markdown search;
combined syntax/identity refinement; trait-method→impl/callers→preview→actual test selection;
original compiler error→API constraint→explicit probe; generated/historical hit→authored owner;
source-bound continuation; and adverse filenames, parsing, permissions, ambiguity, inactive cfg
and missing providers. Each actual journey has independent/native expectations and a revealing
failure case. Unit or snapshot passes alone do not establish native lifetime or useful adoption.

## Conditional design and verification

G11 compares LSP extension, native AnalysisHost/HIR and a narrowly extended analyzer server,
accounting for Cargo/VFS/macro loading and one coherent world. Public SSR range/capture access
and exact family compatibility remain unresolved. Investigate a named useful question rather
than presume all capabilities fit or require prior usage. I06 separately checks pinned rustc
consumer APIs, phase/query timing and structured loans/liveness; neither gate blocks lexical work.
No prototype becomes a second permanent live semantic database by default.

**Proposed verification:** packages run focused controls and public consumers; P02 performs
scope-end assembled qualification with E06 actual fresh-client use. Compare representative
query latency, preparation and answer bytes only for quantitative claims, reporting conditions
and uncertainty. Full scientific/reference/performance campaigns stay at their existing owners.
New plans and examples remain Proposed until code, consumed contracts and named evidence exist.
