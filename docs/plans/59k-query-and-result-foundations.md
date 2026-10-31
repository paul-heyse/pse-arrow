---
title: "Plan59k: Query and result foundations"
status: active
date: 2026-10-31
doc_role: plan
doc_owner: docs/plans/59-agent-cli.md
doc_topics: [current-work, design, libraries, diagnostics]
review_sources: [docs/functional_reviews/rust-centered-search-and-investigation-cli-2026-10-31.md]
---

# Query and result foundations

## Purpose and assessed baseline

**Proposed:** one query meaning and result contract for whole-checkout, Rust-centered
investigation. This owns shared semantics for the [search review](../functional_reviews/rust-centered-search-and-investigation-cli-2026-10-31.md)
O05/O06/O13/O14 and consumes its complete query model. Coordinator dispositions own opportunity
status; this table owns package status. Baseline `9dd0f472a`; previous implementation is
recorded at `81c2759d8`. Core3.4, efficient-architecture heuristics1.0 and the repository binding
apply. This is developer tooling, without new physical/numerical interpretation.

Current `source_read`/`source_query` supply exact source identities and native syntax matching;
`semantic` supplies an explicitly prepared world; `envelope::OutputOptions::emit_page` supplies
producer-owned views. Preserve those useful foundations. Current query execution acquires all
explicit files, counts by analysis and analyzes again for deterministic quotas. Presentation
limits do not reduce that work. `encode_frame` can retain a complete response before projecting;
do not inherit that requirement for every lexical first-page operation. Working-file UTF-8
snapshots are not a universal scanner representation. These focused findings justify new
contracts and producer integration, not a second generic search service or result database.

## Target and decisions

The public intents are `search`, `inspect` and `propose`. `inspect` accepts known file/range,
position, declaration, index/revision and evidence locators without requiring a prior query.
Execution, acquisition and preparation remain explicit operations. Record the maintainer's
2026-10-31 confirmations and publication route in A10 before the public cutover. Existing
mandatory identity, outcome and coverage retain their meaning while representation changes.

Own the smallest sufficient typed Rust model for:

* Sources and selectors: working, index, resolved revision, selected package/module/path/type,
  tracked/untracked/ignored/generated/dependency policy. Selection identifies a universe;
  it does not assert all selected Rust is active in an analyzer configuration.
* Predicates: native text/syntax/structured/semantic terms and explicitly scoped Boolean groups.
  Same-line, same-file, same-node and same-entity conjunctions are different operations.
  Relations such as calls/called-by, implements, references, generated-from and changed-between
  retain direction and multiplicity. Bound requested expansion and settle cycle/duplicate
  handling for that question; do not enumerate every path by default. Unsupported semantic information is unknown,
  not false, and negation over unexamined scope is not established.
* Answer demand: existence, paths, counts, first results, exhaustive matches and selected
  relationships; exact/at-least/unknown counts and deterministic/arrival ordering are explicit.
* Result identity: original provider/context, source origin, match or declaration location,
  relation/capture witnesses, coverage and direct next action. Plain byte search can report a
  lightweight observation; guarded previews hydrate exact original bytes before publication.

Clap flags and a structured JSON request construct the same values and invoke one validation
owner. Reuse native AST rule subtrees rather than implement a custom structural language.
No SQL engine, universal task graph or new service is selected. Normal functions lower the
request into native operations, share useful preparation and preserve early-stop intent.

Default results are compact: readable UTF-8 paths, qualified names where available, short
context and small references. Result count, per-result context, selected fields, grouping and
output allowance are independent controls. Exact fallback bytes preserve unusual paths. Source/provider
facts are shared per context rather than duplicated per capture. Records remain indivisible.
Define result-reference scope/lifetime and query/source continuation guards before consumers
depend on them. Unknown totals stay unknown; stable ordering can require work that arrival-order
first results do not. Do not claim an atomic whole-checkout snapshot from independent file reads.

Use Serde for request/event shapes, `RawValue` for opaque original evidence, and Schemars
serialization/deserialization directions for their distinct schemas. Constructors still enforce
semantic validity. Requested fields/context/grouping should reduce upstream work where valid.
Stream complete typed records with one terminal coverage summary; a broken pipe stops admission
and settles owned work. A no-match file needs cancellation checkpoints even without match callbacks.

Saved typed queries and reusable scopes are selected; introduce Figment only when layered sources
earn it. Carry `ArgMatches::value_source`, `Serialized<T>`, `merge`/`join`, provenance and relative
path mechanisms from review O13. Define CLI/request/default precedence once, never let implicit
Clap defaults defeat explicit configuration, and avoid accidental list concatenation/Global-profile
override. Host environment evaluation remains with its current owner. Indicatif can render actual
stage/file/byte progress on stderr; it does not supply execution ownership, totals or success.

## Package state

| Package | Status | Owner | Completion |
| --- | --- | --- | --- |
| [K01](#k01) | complete | 59k | Shared semantics/defaults and B/C/F impact settled below; A10 publishes proposed ADR-0233/0234/blueprint320 before K02; no runtime completion claim |
| [K02](#k02) | in-progress | 59k | Shared validated requests, compact result/locator/event contracts and directional schemas implemented; integrated consumer acceptance and generated references remain |
| [K03](#k03) | in-progress | 59k | Configuration, whole-record writer and demand-aware providers implemented; budget, presentation and complete public/native acceptance remain in progress |

<a id="k01"></a>
### K01 — Settle the common contract and physical boundaries

**Consumes:** source review and current A03/A04, B01, J01/J02 and semantic context owners.
Resolve source/locator validity, predicate scopes, ranking/ordering, demand versus display,
coverage and retaining versus recomputing results. Compare compact command-specific views with
shared envelope evolution; select the least machinery preserving mandatory meaning. Explain
which B/C/F consumers actually change. Record contract and early-stop examples plus a negative
file-level conjunction case. This design product enables A10 and parallel native feasibility;
it is not a claim that K02 exists. No native-provider decision blocks lexical design.

**Settled execution contract — 2026-10-31:** the maintainer selected fast arrival order for
ordinary bounded search. Default demand is at most 20 useful results, with short per-result
witnesses; `limit`, context, selected fields, grouping and output allowance remain separate.
Stable path/location ordering and continuation are explicit choices. Arrival-order output has
no reproducible cursor. A deterministic cursor binds the versioned request, selected universe,
source/provider premises and next ordered position, refusing changed premises rather than
silently restarting with a different interpretation. Working-file acquisitions remain independent
observations; there is no atomic checkout snapshot claim.

One validated request separates source selection, scoped predicates, typed directed relationships,
answer demand and presentation. Flags, JSON and saved requests construct those same values.
An `all` group within a file can have separate witnesses: one declaration mentioning an endpoint
and another containing a timeout satisfies file conjunction, but neither satisfies a same-node
conjunction. Same-entity conjunction binds one resolved entity in one context. Unknown facts
propagate through composition; absence is established only when the relevant examination permits
it. First witness/first-N fulfillment, exhaustive no-match, exact counting and ranking are
different completion demands. Counts state their unit and exact/at-least/unknown status.

Use compact response records and shared source/provider declarations. Versioned typed locators
carry the minimum source/evidence address and freshness premises for a subsequent CLI invocation;
they are not unqualified display-name/index identities or keys requiring a new result database.
Direct known-source/evidence inputs remain supported. A lexical observation preserves its searched
representation and witnessed location without claiming full-source identity. Before exact
inspection/proposal, hydrate exact bytes and verify selected premises; stale selection refuses,
while an explicitly requested new observation is identified as fresh. Guarded proposals retain
the existing exact source/range/conflict validator.

The stream contract declares invocation/query and source/provider context, complete result and
diagnostic records, then one terminal coverage/settlement summary. It does not build or retain a
complete hidden result merely to show a page. Closed/failed output cancels admission and settles
admitted native work; missing terminal delivery is incomplete. Genuine no-match is successful
empty output. Provider unavailable, source error, cancellation and unexamined input remain
distinct. B06 extends the physical writer/settlement route; K03 supplies demand/configuration and
native providers supply actual stopping facts. Existing non-search one-envelope behavior remains.

Configuration precedence is typed defaults, optional repository configuration, selected request
and explicit CLI overrides. Arrays/predicate groups replace rather than concatenate implicitly;
config-relative paths and provenance remain attached to their source. Host environment selection
is unchanged. Serde and directional Schemars derive the wire views from the declared typed owner;
constructors enforce semantic preconditions. Progress on stderr reports actual work, never a
fabricated total, success verdict or settlement proof.

**Foundation impact:** preserve existing source identity/navigation and proposal validation;
adapt them rather than introduce a second acquisition owner. B06 is the changed physical stream
consumer. C10 adds passive inventory/evidence locator and existing selection adapters, not a new
test runner or inventory. F retains its scientific/product schema and observation owners, so no
F07 is selected unless implementation exposes an actual F-consumed shared-view change. G/H/I/J
keep native interpretation; unavailable native fields do not become display-text inference.
The existing count-then-reparse path is not the mandatory realization for each requested answer.

**Publication boundary:** A10 publishes these decisions through proposed ADR-0233/0234 and
blueprint320. K01/A10 completion is design/publication only; K02/K03, provider and consumer tests
and P02 assembled qualification remain open. No new Tested/Measured runtime claim follows.

<a id="k02"></a>
### K02 — Implement one request and result interpretation

**Consumes:** K01's settled design and A10 governing publication; existing source/evidence owners.
Implement typed construction, scoped query validation, mode-specific locators, source-reference
normalization, schemas and compact public views. Provide stream event types for B06/K03 without
requiring the writer first. Direct locators bypass unnecessary searches. Preserve original
diagnostics and compiler/scientific facts instead of reinterpreting them. Move parser/catalog
and selected consumers with the contract; delete replaced duplicate interpretation paths.
**Acceptance:** flags and JSON express equivalent requests; line/file/node conjunctions differ
as intended; unknown semantic predicates never become false; Unicode/raw paths and current/index/
revision sources retain identity; compact views have valid directional schemas and complete records.

<a id="k03"></a>
### K03 — Publish only the requested useful answer

**Consumes:** K02 event/locator contracts and B06 settled-stream integration. Implement common
writer, cancellation/consumer-close propagation, guarded continuation and effective-query
explanation; integrate selected named-query/configuration and terminal-progress behavior.
Reduce work before materialization when native semantics allow. Exact counting can finish a
scan; first-N need not manufacture omitted totals. Retain expensive evidence only for its
declared purpose. Producers implement their native early-stop checks; this layer is not a
replacement matcher or planner. **Acceptance:** first witness, exhaustive no-match, exact count,
partial failure, canceled no-match input and closed consumer preserve their distinct outcomes;
configuration precedence and file-relative paths work without a second environment evaluator.

## Verification and handoff

### Execution checkpoint — 2026-10-31

K01/A10 governing publication is complete. The shared model/configuration/stream and new
source/evidence adapters have focused passing controls, followed by additional integration
changes that still require revalidation. Public journeys exposed and corrected context rows
being counted as matches and a nonportable `u128` locator timestamp representation. Source
guards now distinguish original roots, lightweight witnessed intervals and exact hydration;
native candidate guards retain their original session as well as generation. Current work
integrates output budgets, result projection/grouping and actual progress. This checkpoint
does not establish full K02/K03, B06, M03 or P02 completion.

The next integrated implementation boundary adds bounded output delivery through the existing
native writer, source-to-semantic guards, typed composition and retirement of replaced passive
routes. Focused public checks include a passing open-but-stalled-reader interruption. Their
first eight-test run had seven passes and one parser failure for `--explain` after the mode;
the parser is corrected, with revalidation pending. Optional native-analysis compilation passes;
its first engine test exposed a build-directory assumption in the shared preparation owner.
Restricted snapshots now seal source/configuration inputs without discovering disabled build
outputs; the correction still needs its functional rerun. API/source joins, assembled native
journeys, generated references and scope-end qualification remain open.

**Proposed verification:** targeted unit/parser/schema controls per package, including independent
expectations and public producer→consumer tests. M03/P02 own assembled coverage and scoped
performance comparisons; current small read-only observations are not an SLA. Scope-end hygiene
and integrated checks follow AGENTS.md. No new implementation is established by this document.
Move enduring contracts through the governing architecture route, regenerate derived outputs
through their generators, and update E06 guidance with delivered behavior.
