---
title: "Plan60 engineering intelligence: remaining design and integration"
status: proposed
date: 2026-11-01
doc_role: review
doc_owner: docs/plans/60-engineering-code-intelligence.md
doc_topics: [design, libraries, diagnostics, validation]
---

# Plan60 engineering intelligence: remaining design and integration

## Combined judgment and evidence boundary

**Proposed:** retain Plan60's engineering-question objective, but replace its old foundation
assumptions with the delivered Plan61 contracts and the implemented, still qualifying Plan62
producers. The most valuable remaining intervention is typed relationship composition across
those producers, followed by usable change, library-fit, lifetime and validation investigations.
Building another intelligence foundation would duplicate work; declaring the remaining target
already delivered would miss important composition and consumer gaps.

This is the plan-improvement half of one combined assessment. The
[functional companion](../functional_reviews/plan60-agent-engineering-opportunities-2026-11-01.md)
owns additional capabilities and their value arguments. References here identify proposed
amendments, not selected scope. Neither document revises Plan60, its O01–O11 dispositions,
Plans61/62, accepted architecture, or implementation status. Planning the plan revision is a
separate operator step. No broad diagnostic audit or completion audit was performed.

Inspected baseline: `7cc70e2013fe3811188a842b5c300dc0af3a628d`, with the concurrent working-tree
changes present during this engagement. Included evidence comprises the expanded analyzer
overviews, the new untracked Cargo metadata overview, relocated CLI guides, and relevant dirty
host/native/proto/generator sources. These are moving source observations, not a reproducible
committed implementation snapshot. No other agent's work was staged, reset or repaired.
The later Plan62 remaining-scope checkpoint takes precedence over its earlier investigation-only
checkpoint; older open package tables do not establish absence of implementation.

Criteria: Core3.4, Efficient Architecture1.0, Test Assessment1.2 and the pse-arrow binding,
selected through `standard.toml`. Process Simulator1.5 constrains interpretation: source,
schema and relationship evidence cannot establish scientific validity or runtime resource release.
The governing contract remains blueprint §21.7. Reviews supply evidence and proposals.

**Interface-checked:** plan/source reconciliation and the concrete mechanisms cited below.
**Tested, bounded:** the public source/discovery probes in the evidence section. No native
semantic, API-acquisition, two-world or compiler journey was newly qualified in this review.
No numerical efficiency benefit is claimed.

## What remains after Plans61 and62

The [Plan61 Outcome](../plans/61-code-intelligence-foundations.md#outcome) records assembled
typed-contract, generated RPC, offline descriptor/payload and origin-navigation acceptance.
The [Plan62 checkpoint](../plans/62-code-intelligence-capability-expansion.md#remaining-scope-implementation-checkpoint--2026-11-01)
records later source/API/Cargo implementation and ongoing native/compiler/composed journeys.
Its Q01 remains independent; this review grants no completion credit to it.

| Plan60 responsibility | Current foundation and residual obligation |
| --- | --- |
| A01 shared evidence | Plan61 already supplies typed wire facts, validation and generated transport. New structural edge meanings and their consumed projections still need design and implementation; reuse the current declaration/generation owner. |
| A02 multi-seed composition | All/Any/Not/Related and request-local native reuse exist. Current related traversal is one seed and prunes destinations outside source selection before expansion; separate traversal/output selection and preserve typed edges/frontiers. |
| A03 focused answers | Compact sections, original-artifact addresses, fields, budgets and finality exist. Improve question-shaped summaries, discovery and refinement ergonomics rather than replace the result system. |
| B01 structural relations | Plan62 role/operation facts are a substantial foundation. Containment, parameter/return/field dependencies, associated relationships and reusable edge witnesses remain a distinct responsibility-graph slice. |
| B02/B03 lifetime and predicates | Captures, contextual candidates, layout and selected compiler observations exist in source. Compose their qualified meanings into selected investigations; preserve actual provider and acceptance limits. |
| B04 conditional bulk | Plan62 A05 already assessed scoped retrieval and kept direct queries. A repeated/global workload or external export consumer can change that decision; do not make an index a prerequisite. |
| C01/C02 API evidence | Rich schema61 contracts, native example transformation and guarded API/native correspondence are implemented foundations. Adjacency, effective surface explanations and any unsupported query-pattern cases require specific consumer checks. |
| D01/D03 comparisons | Git/API comparisons and qualified native worlds exist. A useful aligned change answer still needs explicit correspondence, before/after context and ambiguity treatment. |
| D02 document associations | Canonical lifecycle/section and generated-origin readers exist. A read-only association view must preserve connection basis and original status without inventing path ownership. |
| E01–E03/Q01 adoption and acceptance | Existing guidance, runnables, test selection and original outcomes are foundations. Actual engineering use, setup/assertion relevance, assembled acceptance and accurate publication remain obligations at their original owners. |

This is reconciliation for review, not a replacement mutable progress ledger. The five
[Plan60 companions](../plans/60-engineering-code-intelligence.md#combined-target-and-responsibility-boundaries)
continue to own their work. New documentation capability is neither automatically integrated
nor automatically unassessed: compare the particular mechanism with the prior review and source.

<a id="I01"></a>
## I01 — Amend the remaining target around consumed typed facts

Plan60a still describes the pre-Plan61 shared foundation. Reuse the authored proto and generated
constructors, validators, descriptors and RPC; retain Serde/Schemars for deliberate CLI views.
`NativeRelationshipFact` and `semantic::RelationshipFact` currently describe a relation/direction
and destination location/roles. They do not by themselves encode a responsibility graph with
both qualified endpoints, structural roles and witnesses. In `investigation::rust`, relationship
detail is rendered into `SearchResult.context`; rendered context should not become a join API.

Extend the existing typed producer slice with the first structural consumer, then expose a
shared edge projection preserving native/artifact identity. Keep declaration, reference,
explicit invocation and implicit operation distinct. Avoid one new service or representation
per relationship kind. Buffa direct `ReflectMessage::get`/`ValueRef` can project known fields;
complete external descriptor pools remain a separate interpretation boundary.

This improves AP-04/AP-05, DP-06/DP-07 and H4/H15: meaning survives composition without
category-sensitive text decoding. Compared with retaining destination-only rows, it adds a
bounded contract migration but removes repeated reconstruction in A02/C01/D02/E02. A universal
graph store adds invalidation and authority obligations without a selected consumer benefit.
Closure should include an ordinary caller decoding a qualified edge without HIR linkage and
a namesake/callback counterexample; schema round trips alone cannot establish the relation's truth.
See [functional O01](../functional_reviews/plan60-agent-engineering-opportunities-2026-11-01.md#O01).

<a id="I02"></a>
## I02 — Separate traversal from result selection and share graph work

`investigation::composition::related` builds an allowed-source set and skips destinations
outside it before enqueuing them. This cannot satisfy Plan60 A02's required path through an
intermediate outside the output scope. `Traversal` also starts from one seed and selects one
relation/direction; combining rendered result lists is not a multi-seed edge model.

Give the existing operation an explicit traversal universe and independent output selection.
Share native preparation and demand-selected adjacency across seeds within the request. Return
qualified nodes/edges, shared neighbors, cycles and an honest unexplored frontier. Bound work
at the graph operation, retaining relation kind, direction, multiplicity and witness provenance
when deduplicating. A node visited for one edge kind must not erase a different relationship.

Prefer current native queries and request-local structures before `StaticIndex::compute`.
The pinned index's analysis lifetime and selected-context/file choices do not provide an
all-world graph. AP-03/AP-07, DP-07/DP-12 and H5/H8/H10 support this amendment: output narrowing
must not invalidate the evidence required to answer. Challenge it with overlapping seeds,
cycles, a necessary excluded intermediate, an absent path and a frontier limit. A bounded
positive witness and a complete negative answer have different examination obligations.

<a id="I03"></a>
## I03 — Compose change evidence through explicit Cargo and source correspondence

`dependency.rs`, actual compiler-unit owners, `api::bridge::SourceBridge::prepare` and Plan62
native worlds already distinguish several necessary contexts. Preserve their checks instead
of introducing a generic configuration hash that treats package resolution as execution.
Use `cargo_metadata::NodeDep::{name,pkg,dep_kinds}` and target records for authored/resolved
explanations; use actual admitted units and retained build observations for compiler claims.
Raw retained JSON is needed where normalized defaults erase field presence or experimental
artifact-dependency detail. `Message::parse_stream` is useful for original event ingestion but
can turn unknown/malformed JSON into `TextLine`; that is not successful typed validation.

For D01/D03, align source/API observations first, then qualify the two selected semantic worlds.
Names and ranges nominate correspondence; they cannot establish identity across revisions.
Preserve cfg-only changes, missing worlds and ambiguous matches in the answer. Guppy package/
feature set operations or `guppy-summaries::Summary::diff` become useful if repeated selection
or modeled host/target comparison earns their adaptation. They do not replace actual unit
evidence, and ordinary set algebra does not join unrelated graph snapshots. Determinator's
modeled impact is not a coverage proof. Existing `petgraph::astar` already serves inclusion
paths; do not propose replacing a nonexistent handwritten shortest-path algorithm.

This improves AP-04/AP-07, DP-04/DP-09 and H12–H14. Start with existing owners; add a graph
library only for a concrete missing query. Test an alias, target-specific feature difference,
unchanged spelling with changed resolution, and a historical blob without a prepared world.
The functional consumer and alternatives are in
[O03](../functional_reviews/plan60-agent-engineering-opportunities-2026-11-01.md#O03).

<a id="I04"></a>
## I04 — Make lifetime investigations evidence slices over existing producers

Plan62's projected captures, stored/source types, suspension fields, initialization, cleanup
and ABI observations substantially change B02's starting point. Reuse `semantic::native`
facts and `compiler_investigation::facts` rather than build another loan/capture model.
Join a selected body, captures, adjustments, awaits and original diagnostics with the actual
cancellation/join owner; expose which portion each provider establishes.

HIR `ClosureCapture`, `Semantics::expr_adjustments` and typed predicates explain source-level
meaning; pinned compiler phase-qualified observations explain selected lowering/cleanup facts.
Saved storage, may-initialization, active loans, use liveness and runtime destruction remain
different properties. Current compiler facts do not justify an observed-resource-release claim.
Extend compiler extraction only when an unanswered question needs a fact the existing artifacts
and diagnostics cannot supply. Preserve Plan62 D04's unmet loan-extraction trigger.

This improves AP-01/AP-04 and DP-05/DP-15. A separate lifecycle inference engine would purchase
broader obligations without establishing settlement. Acceptance should follow a retained guard,
interior mutation, cancellation signal and actual CPU join, including indirect work that remains
unknown. Useful bounded explanation is preferable to an unsupported complete-effect claim.

<a id="I05"></a>
## I05 — Separate API discovery, effective surface and contextual availability

Plan62's `api::query::contracts` already projects recursive generics, typed attributes, stability
and capture classification, and `api::doctests` retains original/transformed/wrapper evidence.
Plan60 C01/C02 should use these foundations and identify particular missing pattern/adjacency
cases rather than reconstruct rich contracts.

One concrete residual is `api::query::TypePattern::{parse,from_syn,matches}`: current matching
supports path/type arguments, references, qualified types, tuples, slices, pointers and wildcard,
but rejects lifetime/const/binding arguments and callable-trait forms. Rich serialized contracts
do not supply those query capabilities. Extend this same matcher using syn typed syntax and
schema61 structures, preserving explicit refusal for unsupported shapes.

Use rustdoc `Item.links`, associated items, supertraits, explicit/negative/synthetic/blanket
impl records, default methods and dereference-mediated APIs to explain an effective candidate
surface. Keep facade exposure, definition, documentation link, shape match, bound/environment
and native call-site result separate. HRTB binder scopes, associated equality versus bounds,
const arguments and GAT constraints must survive structural patterns. HIR environment/bound
queries can refine a compatible selected context; they are not a second rustdoc identity system.

AP-04/AP-05, DP-07/DP-15 and H6 support selected adjacency and original-artifact expansion.
A full eagerly extracted API graph adds work and omission interpretation; original declaration
inspection remains a good alternative for rare cases. Challenge this with reused item IDs,
missing defining artifacts, stripped fields, a blanket/default method and a matching shape that
does not discharge a generic obligation. See
[O02](../functional_reviews/plan60-agent-engineering-opportunities-2026-11-01.md#O02).

<a id="I06"></a>
## I06 — Complete the agent-facing projection and recovery contract

The discovery parser already owns rich metadata and lazy candidate associations; configuration
already uses Figment provenance from the actual composed request. Improve these existing owners.
The public probes below expose specific remaining friction:

- General result objects contain numeric nanosecond guards that ordinary JavaScript JSON
  round trips cannot preserve. `ResultReference::encode` already supplies an opaque URL-safe
  address and some next actions use it. Publish an existing native-encoded reference or exact
  ready invocation consistently with addressed general results; retain structured detail for
  inspection. Do not weaken freshness checks to accommodate lossy callers.
- `commands search` can return a large rich catalog. `--limit 1` limits records, not the width
  of each record. Offer a concise command-description view with guarded access to the full
  parser description. Derive it from Clap metadata; do not maintain another catalog.
- `--fields` currently accepts literal top-level payload names. `records.path` is rejected
  as `host.io`/`runtime.infrastructure`, without valid-field guidance. Explain the supported
  selection grammar and return a typed request/presentation diagnostic with useful alternatives.
  A new nested projection language is unnecessary merely to fix this journey.

Clap `CommandFactory`, public getters and `ValueSource` support discovery and explicit-input
semantics, but private conditional requirements must remain explicitly unavailable. Deferred
construction requires deliberate building before introspection. Figment `find_metadata` already
explains effective values; `Tagged<T>` and `RelativePathBuf` are selective options, not reasons
to duplicate the existing provenance map. Its tags are not overwritten-value history.

Preserve `ErrorRecord` and the existing loss-aware Miette projection. `diagnostic_source`,
`related`, labels and contextual help should survive wrappers; adding `JSONReportHandler` as a
second wire taxonomy would lose the current versioned envelope meaning. Indicatif remains a
stderr presentation consumer of observed activity, separate from durable stream finality and
CPU settlement. `ProgressDrawTarget`, optional `MultiProgress` and `InMemoryTerm` can replace
bespoke terminal coordination if a selected concurrent progress consumer requires it.

The selected validator also provides structured violations that current payload diagnostics
reduce to strings and RPC diagnostics retain opaquely. A focused projection of its original
field/rule paths into these same owners can improve correction without another validator;
[functional O06](../functional_reviews/plan60-agent-engineering-opportunities-2026-11-01.md#O06)
preserves its redaction, truncation and validator-defect limits.

AP-05/AP-06, DP-21/DP-24 and H6/H16 support these changes. Acceptance must include a real agent
copying an opaque address, integer-preserving structured round trip, invalid field recovery,
short discovery followed by detail, and non-TTY/partial-stream behavior. Neither new commands
nor a general interactive shell are required. Functional behavior is developed in
[O05](../functional_reviews/plan60-agent-engineering-opportunities-2026-11-01.md#O05).

<a id="I07"></a>
## I07 — Preserve assurance distinctions when composing validation relevance

Plan62 runnables, original inventory/fixture/report owners and current test selection are the
starting point for E02. Relate the production definition to direct tests and shared setup,
then expose observable assertion structure separately. A call in a test does not identify the
asserted contract; the same helper used to compute expected and actual values can hide a shared
error. Relatedness, target eligibility, execution outcome and independent assurance need their
own evidence basis.

Use native runnable/cfg evidence and syntax structure to nominate revealing checks. Keep
current selection and original outcomes under their existing owners; never regenerate inventory
or rerun tests during inspection. This improves AP-06, DP-05/DP-23 and Test Assessment1.2.
Compare the existing conservative affected selection with a more precise source-backed
explanation without promising complete impact analysis. Challenge the view with a setup failure,
shared fixture fan-out, an unasserted call and a self-derived expected value. The functional
journey belongs to [O04](../functional_reviews/plan60-agent-engineering-opportunities-2026-11-01.md#O04).

<a id="I08"></a>
## I08 — Integrate by engineering question and retire only replaced assembly

Recommend an early coherent slice combining I01/I02 with a small responsibility question;
improve discovery/reference transport in parallel. Then compose API/change/validation/lifetime
questions using their working producers. This is an integration recommendation, not a new
implementation schedule or permission to start work. Contract agreement enables design;
working producer and consumer evidence enable acceptance.

Keep library/API question work independent of unrelated compiler depth or every possible native
relationship. Validate original witnesses and negative examples through real public consumers.
Delete superseded string assembly and duplicated conversions when their consumers migrate;
retain complementary source-only, historical-artifact and explicit compiler routes. Do not make
indexes, remote delivery, runtime-schema validation or full static campaigns per-slice gates.

AP-03/AP-06/AP-07 and H1/H4/H18 support reducing transition work while preserving independent
assurance. Plan60 E03/Q01 should ultimately exercise the assembled selected scope once; Plan62
Q01 remains its own prerequisite evidence where the new plan relies on delivered behavior.
Any newly selected governing rule change or compiler/service/crate commitment needs the existing
decision route before dependent implementation. No such change is approved here.

## Public observations and material limits

**Tested — 2026-11-01, local Linux, current prepared host, source-only commands; zero failures
is the target.** These are review probes, not a product test suite:

| Command/scenario | Actual result |
| --- | --- |
| `scripts/pse search symbol Request --path crates/pse-host/src/investigation/model.rs --limit 2 --format json` | Exit0; two records, `complete:false`, `at_least:2`, limit stop. Correctly did not claim exhaustive absence/count. |
| Returned reference through JavaScript `JSON.parse`/`JSON.stringify`, then `scripts/pse inspect --result … --format json` | Two attempts exited125 with stale observation. The integer guard was rounded; these attempts do not establish concurrent source changes or a successful expansion. |
| Same search with `--limit 1`, Python `json.loads`/`json.dumps` preserving integers, then returned-reference inspection | Both exited0; original source inspection succeeded. Guard value `1793520353462910813` illustrates the precision boundary. |
| Reference with deliberately incorrect source digest | Exit125, original stale-content diagnostic preserved; expected negative control, no acceptance credit as a positive journey. |
| `scripts/pse commands search --limit 1 --fields records.path,records.about --format json` | Exit2; unsupported nested field classified as infrastructure failure. Observed recovery/diagnostic gap, not a passing valid-query test. |
| `scripts/pse commands search --limit 1 --fields records --format json` | Exit0; one full record, omitted top-level fields declared. Record count restriction does not provide compact per-command metadata. |
| `scripts/pse search text Request --path crates/pse-host/src/investigation/model.rs --limit 2 --explain --format json` | Exit0; effective request and field origins, no providers invoked. Existing Figment explanation is useful. |
| `scripts/pse search text native-hir --literal --path crates/pse-host/Cargo.toml --path docs/plans/60b-native-relationships-and-lifetime-evidence.md --limit 3 --format ndjson` | Exit0; two witnesses and a complete terminal record across two examined files. No semantic claim follows from text matches. |
| `scripts/pse code session status --format json` | Exit0; `prepared:false`, subject unprepared. No analyzer was started or another session stopped. |

Fresh native multi-seed, library-fit, before/after-world, lifetime and failure-to-test journeys
were **not_run**: this checkout had no prepared semantic session and concurrent implementation
was still moving. Their current designs were assessed through sources, retained qualification
and revealing scenarios; no passing claim is made for proposed behavior. No new acquisition,
compiler execution, scientific campaign or benchmark ran. The failed field-selection and lossy
reference attempts remain in this account rather than disappearing behind the successful route.

## Sources and disposition

Preparation read all thirteen `docs/libraries/rust_analyzers` documents and four named CLI
guides, including all unique addenda; the compiler documents overlap and are one source family
for shared claims. Protocol reading was selective as requested: deep current-stack reading plus
screening of unused compilation, generation, reflection, validation and alternate stacks.
The [functional companion's source map](../functional_reviews/plan60-agent-engineering-opportunities-2026-11-01.md#sources-and-library-boundaries)
locates concrete library mechanisms and exact-version caveats. Source inspection covered the
existing composition/model/rust adapters, semantic/native facts, API contracts/bridge/doctests,
dependency/compiler-fact owners, generated-contract declarations, catalog and envelope adapters,
and document ownership interpretation. This is focused footprint coverage, not a whole-repository audit.

The combined recommendation is I01/I02/I06 first, with I03–I05/I07 composed through their
existing producers and I08 guiding integration. Functional opportunities can add value, but
must be selected in the later plan-update conversation. No recommendation changes a package
state, weakens acceptance or introduces a new authority over plan progress.
