---
title: "Plan60d: Change and document evidence"
status: active
date: 2026-11-01
doc_role: plan
doc_owner: docs/plans/60-engineering-code-intelligence.md
doc_topics: [current-work, design, documentation, diagnostics]
review_sources: [docs/functional_reviews/engineering-workflow-code-intelligence-2026-10-31.md, docs/plan_improvement_reviews/plan60-engineering-intelligence-2026-11-01.md, docs/functional_reviews/plan60-agent-engineering-opportunities-2026-11-01.md]
adrs: []
---

# Change and document evidence

## Purpose and foundation assessment

**Proposed:** evidence for coherent change planning and code/document association, contributing
to original [O04/O06](60-engineering-code-intelligence.md#opportunity-dispositions), incorporating
[improvement I03](../plan_improvement_reviews/plan60-engineering-intelligence-2026-11-01.md#I03)
and functional [O03/O04](../functional_reviews/plan60-agent-engineering-opportunities-2026-11-01.md#O03).
The coordinator owns dispositions; this document owns D01–D04 execution state.

Starting baseline is `42bede27a` and Plan62's completed
[Outcome](62-code-intelligence-capability-expansion.md#outcome). Existing Git/history,
lossless syntax, API comparison, generated-origin and document/lifecycle readers remain the
producers. Plan61 supplies typed contracts and original evidence guards. Plan62 supplies rich
API/native correspondence, explicit native worlds and package/compiler-unit explanations.
`cli::dependency::{UnitContext,join_units}`, `correctness::{Resolution,CompilerUnit}` and
`semantic::NativePackageContext` are starting points, not missing resolvers to recreate. Their
completed implementation and qualification retain their original owners. D04 compares two
selected retained contexts; it does not rebuild Plan62's one-context package/unit/world join.
Exact proto origins now use authenticated native byte ranges; reuse those original associations
for D02 rather than revive removed descriptor-column conversion. Source
`ownership::PlanOwnership` currently has `NotDeclared`; package
membership and `xtask::documentation::ownership` do not infer code-path-to-plan authority.

The source review O04/O06 and blueprint §21.7 preserve origin and interpretation. Document
metadata/sections/lifecycle bindings remain canonical through
[the documentation guide](../dev/documentation.md#metadata-and-aggregate-scope). This plan adds
read-only views, not a new authority mapping database, editable status ledger or new publishing
system. Current plan/review/ADR status retains its original owner and meaning.

## Target design

A proposed change has several views: written source declarations/body differences, acquired
API differences, selected package/unit context deltas and resolved relationships in explicitly
prepared worlds. D01/D04 remain independently useful without a native world; D03 adds semantic
comparison only where both selected worlds and correspondence are available. Align
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

### Authorized execution refinement

Execute D01–D04 from `cea33a310`, consuming delivered A–C. A focused `inspect comparison`
accepts guarded before/after references and source/API/context/semantic views; JSON and flags
share validation. Explicit checkout owners bind semantic/context operands. Context bundles
retain each original metadata/unit component and digest; missing components remain partial.
Two independently owned current native sessions supply D03; no session registry, automatic
checkout or historical-world substitution is selected. Association views use canonical
captured document/policy interpreters rather than a second status parser. Guppy/modeling and
other dormant branches retain their existing triggers. Target publication precedes source work.

<a id="d01"></a>
### D01 — Original source and API change views

Consumes delivered shared evidence and existing source/API comparison producers. Expose declaration,
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

Consumes the delivered typed evidence and document/lifecycle/generated-origin readers. New A01
edge projection, B01 native relations and B05 specialized origins enrich only the connections
that actually consume them; explicit citation and ownership paths do not wait for those producers.
Deliver a change-planning question: show a selected source declaration,
authored generator/schema origin where declared, relevant contract sections, plan packages
and original qualification records, each with its actual connection basis.

Use native metadata/defaults/exceptions, scope-table identities and exact section resolvers;
retain the original record address instead of copying mutable state. RA syntax
`HasAttrs::{doc_comments,attrs_with_doc}` and Markdown parsing can nominate authored citations;
HIR `resolve_doc_path_on` can establish applicable intra-doc symbol links under a selected world.
Those are distinct from architecture-section citations and declared document ownership. No
guessed path-to-plan ownership is written back. Distinguish
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

Consumes D01 and the existing Plan62 qualified world/context and observation producers. New
A01/B01 edges are prerequisites only when the named comparison consumes those particular new
relations; comparing already supplied observations does not wait for the whole structural graph.
Use D04 to expose original package/unit premises alongside a semantic answer where relevant,
without making D04's entire delivery a prerequisite for every native comparison. Require
separately and explicitly prepared before/after worlds, including Cargo/feature/target premises.
Do not prepare every workspace configuration or assign today's world to a historical blob.
Compare selected observations only after qualifying source/context correspondence; ambiguous
and missing observations remain unmatched/unknown. Reuse original native owners and A03
aligned output rather than a second semantic diff engine with invented stable identities.

Acceptance includes a cfg-only change that changes a resolved dependency with unchanged local
spelling, a rename with ambiguous correspondence, a missing world and incomplete macro input.
The public answer must explain which comparison was made and which contexts were not examined.
A single-world “no difference” cannot satisfy a two-world request. Include same-package host,
target and test interpretations, changed source membership and macro-generated locations without
unique correspondence. Migrate duplicate current-world
assumptions in affected comparison callers/tests as the replacement becomes usable. Explicit
preparation and source-only fallback remain separate supported choices.

<a id="d04"></a>
### D04 — Selected package and compiler-unit context deltas

Consumes retained Cargo metadata, original admitted `CompilerUnit` records and existing
dependency/context explanation owners. Deliver an aligned comparison of two explicitly selected
feature/root/target contexts: changed inclusion, dependency alias/kind, unit role/features and
the reason a native profile is matched, ambiguous or absent. This is useful without preparing
an analyzer, compiling or acquiring fresh metadata. Native cfg/resolution differences are an
optional D03 enrichment when both original worlds exist.

Keep package identity, unit correspondence and declaration correspondence as separate joins.
Pair same-package units by qualified target name/kinds/source, host/target and mode; feature
changes remain differences rather than preventing all pairing. Preserve unmatched units and
package upgrades/moved roots as candidate correspondence where necessary. Existing `join_units`
deliberately reports compiler equivalence not established; comparison must not promote its
package/library/features/target/role association into compiler equivalence.

Use `cargo_metadata::NodeDep::{name,pkg,dep_kinds}` and target records for authored/resolved
evidence, actual selected units for compilation-selection evidence, and retained original
`Message` observations only for their recorded build facts. Typed defaults can hide field
presence or experimental artifact dependencies; retain raw JSON when the question needs that
fidelity. `Message::parse_stream` can turn malformed/unknown JSON into `TextLine`, not a validated
typed event. Cached build-script output is not proof of fresh execution. Load each selected
snapshot once per inquiry and compare demanded projections rather than repeat scalar extraction.

Acceptance covers renamed dependencies, target-specific features, same-package host/target/test
units, a metadata package absent from selected units, missing/ambiguous native profiles, changed
source membership, incompatible source premises and an absent before-snapshot. Original records
remain inspectable even when comparison cannot establish correspondence. Delete duplicated
manual context joins only after their consumers migrate; preserve the original single-context
inspection and actual admission owners.

#### Conditional modeled comparisons

The operator retained explicit triggers on 2026-11-01. Guppy `PackageGraph`/`PackageSet` and
`FeatureGraph` are candidates when an identified repeated set/conditional-edge question cannot
be served adequately by current owners. `CargoSet::to_summary` and `guppy-summaries::Summary::diff`
are candidates for a selected modeled host/target counterfactual, not actual selected-unit
evidence. Verify experimental workspace-unification compatibility, graph-local set identity and
exact release behavior before adoption. Determinator may nominate changed-path impact only
when a named consumer warrants its modeled uncertainty; it cannot establish coverage or complete
affected selection. Existing `petgraph::astar` already supplies inclusion paths.

Record the concrete question, source/pin qualification, retained-owner alternative, selected
outcome and affected consumer before dependent work. A dormant trigger needs no feasibility
prototype and does not block D04 or Q01. See dependency-tooling addenda A–E/G and the Cargo
metadata main/addendum sections for source leads. Their simulated graph and observed build
meanings remain separate under AP-04/DP-04/DP-09.

## Package state

| Package | Status | Owner | Completion |
| --- | --- | --- | --- |
| D01 | in progress | Source/API comparison owner | Qualified original differences and guarded before/after inspection |
| D02 | in progress | Investigation/document bridge owner | Read-only associations preserving native authority, basis and status |
| D03 | in progress | Native context comparison owner | Public selected-world comparison with cfg and ambiguous mapping acceptance |
| D04 | in progress | Cargo/unit context comparison owner | Aligned original package/unit deltas, qualified correspondence and independently useful passive inspection |

## Verification and handoff

**Proposed:** targeted comparison and association fixtures with positive and misleading joins,
plus actual public change-planning use in E03. Documentation interpreter tests should assert
original declared meaning; a duplicate heuristic expected answer is insufficient. Q01 owns
assembled/static qualification and architecture publication if new enduring contracts require
it. No document immutability exception, new lifecycle vocabulary or live plan-status copy is
authorized by this plan. Targeted comparison/association tests and their actual public
consumers accompany D01–D04 implementation. E03 assembles the change/evidence-path journeys;
Q01 owns the later scope-end checks. Source correspondence and output projections are
Interface-checked leads from the reviews, not newly Tested behavior from this authoring.
