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

**Implemented:** agents start with a name, partial shape, known location or original diagnostic,
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
| [M01](#m01) | complete | 59m | Source/native/API discovery and guarded inspection have targeted public acceptance; assembled P02 remains coordinator-owned |
| [M02](#m02) | complete | 59m | Scoped composition, resolved identities and guarded API/use/importability joins have targeted public acceptance; selected G12 is implemented |
| [M03](#m03) | complete | 59m | Consolidated public intents, replaced-route deletion and actual investigation/adoption journeys have targeted acceptance; assembled P02 remains coordinator-owned |

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

## Outcome and verification — 2026-10-31

**Implemented:** source declarations, explicitly prepared workspace/library symbols and acquired
API candidates share the validated public request without sharing an invented authority. Name
policies retain exact/prefix/contains/fuzzy availability, written qualification and provider
limits. Scoped Boolean evaluation keeps file/node/entity witnesses distinct; bounded directed
relations retain direction, multiplicity, cycle controls and unknown facts. Resolved identities
bind the actual declaration to the original session/configuration/generation. Refined rows retain
their source/API candidate reference separately from native identity and shared world provenance.

API joins preserve original public aliases, defining-artifact evidence and package/context
guards. A request-owned bridge compares the acquired documentation's actual library-unit and
transitive feature closure with the same loaded native world, then verifies original source
bytes before lifting a span. Native visibility and `find_use_path` prove selected destination
importability; spelling, signature display or a same-feature root package cannot replace that
proof. Unavailable definitions/context facts retain structural candidates as unknown, without
counting them as proven positives or upgrading unknown through later conjunctions.

Passive source/dependency/owner/compiler/log/test reads moved to `inspect`; discovery uses
`search`, and immutable syntax/native edit previews use `propose`. Replaced public parser
branches, callers, old snapshots and duplicate read interpretations were deleted. Preparation,
acquisition, compiler probes, test execution and session control remain explicit effects. E06
guidance/advice and generated references follow that parser; no compatibility aliases or edit
application path were retained.

**Tested:** the four actual `composed_investigations` consumers each have a targeted passing
run: namesake target refinement→guarded inspection→relationship→preview→test selection;
acquired API constraints with native use/importability; aliases/unused imports/privacy across
two native contexts; and an unchanged root package whose transitive dependency features differ.
These invoke real explicit artifact/helper/session preparation, not fabricated sidecars. Selected
`investigations` public controls additionally cover mixed content, continuation, adverse paths,
partial errors and missing providers. Native component, API, source/template/DTO and layout
controls establish their bounded producer contracts. Exact commands, counts and actual run
identities are centralized in [P02](59-agent-cli.md#p02); targeted passes are not an assertion
that its final assembled campaign has passed.

Actual independent adoption also followed an original compiler failure through API evidence to
an explicit probe, and a generated/historical hit through its declared authored emitter. Those
original managed invocations remain addressed evidence under P02 rather than being reproduced
as a second ledger here. No current semantic result is assigned retroactively to old bytes.

**Mistakes corrected:** an initial native helper launch selected the ordinary host, omitted
compact section fields failed deserialization, and a root-only API context comparison missed
transitive feature changes. Actual public journeys exposed those defects; explicit helper
dispatch, compact DTO defaults and captured transitive profiles corrected the production
connections. A temporary lower-level semantic realization also lost native IDE behavior and
missed descendant cancellation; it was removed in favor of original native IDE operations with
a narrow opt-in request cancellation group. Physical settlement, sibling independence and the
original session lifetime remain separate from merely dropping a client future.

### Conditional decisions and deliberate limits

[G11/G12](59g-rust-semantic-intelligence.md#g11) selected an explicitly prepared immutable
Cargo/VFS/HIR snapshot using the exact compatible RA0.0.358 family; source-only syntax0.0.360
does not cross its AST boundary. The existing custody/preparation owner hosts one selected world,
loads installed standard-library sources and captures actual effective package/target/source
facts. Queries run no Cargo and stale inputs require explicit reprepare. The live LSP provider
remains separately selectable with its actual capabilities. Snapshot cancellation preserves
original IDE semantics through the documented Salsa/IDE-db patch; component and authenticated
service tests establish the bounded selected lifecycle, not a universal cancellation latency.

Native SSR search is not adopted because the examined release does not expose addressed match
ranges/placeholders; syntax captures plus HIR refinement remain available, and semantic SSR
preview retains its distinct editing purpose. Revisit on usable native addressed-capture access
or an explicitly selected narrow extension. [H08](59h-api-and-dependency-intelligence.md#h08)
retains actual metadata/unit owners: counterfactual feature graphs, hosted artifact acquisition,
scraped examples and archive conversion remain unselected with their named revisit triggers.
[I06](59i-compiler-investigations.md#i06) retains original diagnostics and explicitly generated
MIR/probes for the demonstrated conflict/retention question. It does not adopt I07 or fabricate
typed loans/move-path/liveness facts from text; selecting an exact machine-readable
location/phase/domain question reopens that branch.

M01–M03 targeted implementation acceptance is complete. P02 owns assembled qualification,
scope-end hygiene and final adoption accounting; E01's separate safety choice and Plan58 H
scientific/deployment obligations remain with their owners. No performance benefit is Measured
by these functional passes. This plan remains retained as an active coordinator dependency.
