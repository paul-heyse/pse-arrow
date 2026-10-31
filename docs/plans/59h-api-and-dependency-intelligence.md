---
title: "Plan59h: API and dependency intelligence"
status: active
date: 2026-10-29
doc_role: plan
doc_owner: docs/plans/59-agent-cli.md
doc_topics: [current-work, design, libraries, environment, validation]
review_sources: [docs/functional_reviews/rust-code-intelligence-cli-2026-10-29.md, docs/plan_improvement_reviews/plan59-library-integration-and-target-design-2026-10-30.md, docs/functional_reviews/cli-test-and-runtime-workflows-2026-10-30.md, docs/functional_reviews/rust-centered-search-and-investigation-cli-2026-10-31.md]
---

# API and dependency intelligence

## Current extension — 2026-10-31

**Proposed:** H07/H08 extend the completed H01–H06 baseline with structural capability
queries, source-use/importability composition and bounded graph/acquisition decisions from
[O03/O10](../functional_reviews/rust-centered-search-and-investigation-cli-2026-10-31.md#O10).
The maintainer accepted `search`/`inspect`/`propose` hard cutover and compact shared
provenance/streaming on 2026-10-31 while preserving identity, outcome, coverage and useful
passive reads. Earlier `api`/`dep` read-surface target wording records the completed baseline;
the new public consumer migrates through M03 only when its replacement works. Explicit API
acquisition, documentation checks and probes retain their effect owners. K owns shared query/
result meaning and A10 its governing publication. Historical completed rows and evidence do
not qualify these extensions; this revision starts no implementation or acquisition.

H07 can deliver structural rustdoc/dependency and existing-LSP composition before G12. Only
typed facts selected by G11 and implemented by conditional G12 may extend that answer later.
H08 makes a supported adoption or no-adoption decision; library availability alone selects none.

## Purpose and assessed foundation

**Proposed:** help an agent discover a usable library operation, understand its constraints,
obtain an actionable example and assess a dependency or public-interface change. Exact item
lookup is the foundation; this companion adds the questions an agent asks before knowing an
item's name and after deciding to change it.

This companion develops the code-intelligence review's [O05](../functional_reviews/rust-code-intelligence-cli-2026-10-29.md#O05),
[O06](../functional_reviews/rust-code-intelligence-cli-2026-10-29.md#O06),
[O11](../functional_reviews/rust-code-intelligence-cli-2026-10-29.md#O11),
[O12](../functional_reviews/rust-code-intelligence-cli-2026-10-29.md#O12) and documentation portions
of [O10](../functional_reviews/rust-code-intelligence-cli-2026-10-29.md#O10). The coordinator owns
their dispositions; this document owns H01–H06 package state. Baseline is review publication
`a569b52e4`, inspecting production `93558a8b82a729fbdb44b86b44ee8923fe78dba3`. Core3.4,
efficiency heuristics1.0, test assessment1.2 and process-simulator1.5 apply through the binding.

The operator accepted RC01–RC05 during preparation, including explicit effects and retained
test/build/scientific authority. Plan59 production execution remains paused at the coordinator's
2026-10-30 checkpoint. This authoring
does not install tools, change dependencies, generate JSON, run probes or revive rustdoc-mcp.

The current host already depends on `cargo_metadata`; the actual proposed `pse dep`, `api` and
`probe` remain [D02](59d-knowledge-and-workflow-verbs.md#d02). Its recorded trial establishes
only that trial's findings. D02 owns exact locked-source acquisition, multi-version addressing,
format preflight, context-keyed API artifacts and isolated probe construction. H consumes those
products instead of introducing another registry loader, cache or dependency resolver.

The focused foundation assessment is that typed rustdoc structure and Cargo metadata can
support the selected questions without a new persistent source graph or a live analyzer.
Rustdoc is a documentation projection: it neither supplies body references nor establishes
arbitrary trait applicability. Cargo package reach is conservative; it does not establish
actual compiler units or tests. Those boundaries shape the target rather than being repaired
through custom inference in CLI handlers.

The 2026-10-30 library-integration review supplies
[I01](../plan_improvement_reviews/plan59-library-integration-and-target-design-2026-10-30.md#I01),
[I07](../plan_improvement_reviews/plan59-library-integration-and-target-design-2026-10-30.md#I07),
[I10](../plan_improvement_reviews/plan59-library-integration-and-target-design-2026-10-30.md#I10),
[I12](../plan_improvement_reviews/plan59-library-integration-and-target-design-2026-10-30.md#I12),
[I13](../plan_improvement_reviews/plan59-library-integration-and-target-design-2026-10-30.md#I13),
[I14](../plan_improvement_reviews/plan59-library-integration-and-target-design-2026-10-30.md#I14) and
[I15](../plan_improvement_reviews/plan59-library-integration-and-target-design-2026-10-30.md#I15)
for these remaining targets. The coordinator owns the operator's 2026-10-30 RC01–RC04
confirmations/decision routes; the original O identities and all package states remain intact.

## Target and shared contracts

Keep `pse api` and `pse dep` as the existing task surfaces, extended with selected query modes.
The command grammar is owned by A02/A06 and must describe the actual parser; illustrative
flags in the source review are not a second grammar. Provide name/docs search plus structural
filters for item kind, input/output type references, trait and safety/async/const qualification.
Dependency inspection supports selected why/features/users expansions. API comparison and
example expansion are explicit operations; neither performs an update or executes code.

Return one compact answer: useful signatures or dependency paths, relevant constraints,
source/public-path pointers, acquisition context and a few typed next actions declaring their
arguments/effects. The API/dependency producer selects useful detail modes and page units:
candidates/signatures first, then requested impl groups, docs, examples or paths. Push selection
into addressed acquisition where sound; batch related items against one artifact rather than
regenerate per item. Preserve mandatory identity, status, bounds and coverage in every page.
A03's envelope/fields/budgets provide mechanical projection, not recursive cuts inside arbitrary
declarations/tuples. Report examined artifact/graph scope separately from returned size, with
known omissions, unknown totals, absent matches and unavailable acquisition distinct.

Continuations bind artifact/source digest or revision and extraction/backend context. Refuse a
stale cursor or serve the deliberately retained immutable source; silently switching to current
source would change the question. Do not serialize/retain an entire rebuildable result merely
to display a page. A result is regenerable only while all inputs remain available; dirty source
bytes and uniquely authored probe inputs cannot be treated as disposable preparation. Keep
original compiler/probe diagnostics separately from their regenerable summaries. A04 addresses
artifacts without imposing a common deletion rule; canonical scientific data and Git intent
retain their existing owners.

The owning adapter decodes version-checked inputs into typed checked values before decisions;
owned result/page views serialize directly through directional Serde/Schemars declarations.
Schema validity, decode success and domain validity are separate. Unknown format/protocol versions,
invalid item/context combinations and malformed inputs refuse at the relevant constructor.
Opaque upstream evidence can retain `RawValue` lexical fidelity; it is not the generic domain
model. Do not turn unavailable/nonfinite numerical facts into a null/zero success through JSON.

| Meaning | Owning contract and consumed behavior |
| --- | --- |
| Acquisition and reuse identity | D02 supplies exact package/version/source, features, target/cfg, toolchain, extraction settings and rustdoc schema version through A04/A05. Check `format_version` before typed decode. |
| Documentation context | Preserve extraction flags and documentation-only conditions. Known gates are reported; unknown ordinary-build availability remains unknown. `cfg(doc)` presence is not proof that the API compiles in the selected application. |
| Item identity and public path | Rustdoc IDs are local to an artifact. Reexports and defining locations are separate; cross-artifact matching is qualified rather than numeric-ID equality. |
| Query result and evidence | A03 presents declared API structure, candidate ranking, compatibility diagnostics, planned units and observed compiler results as distinct facts. No rendered docs string becomes an inferred semantic authority. |
| Compiler/test selection | C07 owns the selected command context; existing correctness admission governs actual probes/checks. Unit graphs are plans; captured compiler artifacts and test listing are observations. |
| Source-use enrichment | G's references may be requested when available. Basic API/dependency work has no G session prerequisite; missing semantic enrichment stays explicit. |
| Source navigation and composed answer | J01 addresses exact source/revision; H links its page to original diagnostics/source and optional G/I facts without a generic composer. Missing enrichment never blocks the base answer or runs a probe/check implicitly. |

Use `rustdoc_types::Crate.index`, `paths`, `external_crates`, `Use`, `Type`, `Generics`,
`WherePredicate`, `Trait`, `Impl`, structured attributes and documentation links for the
declared API view. A public-path and trait-aware response should retain associated types,
supertrait constraints, required/provided methods and relevant imports rather than list only
methods declared next to a struct. Safety, ABI, deprecation, `must_use`, non-exhaustiveness and
stripped members appear where decision-relevant; absent metadata is not a universal guarantee.

These mechanisms are preserved from [rustdoc-types §§2–10, 12–13](../libraries/rust_analyzers/rustdoc-types.md),
[rustdoc §§4–6, 8–10](../libraries/rust_analyzers/rustdoc.md), and
[dependency tooling §§2–3, 5–7](../libraries/rust_analyzers/rust_dependency_tooling.md).
Exact release APIs are qualified during implementation through Context7 then primary sources;
the review's source inspection is not an executed pinned-interface probe.

## Package state

| Package | Status | Owner | Completion |
| --- | --- | --- | --- |
| [H01](#h01) | complete | 59h | Search, facade/trait provenance and optional original declaration delivered; actual selected API reads and focused source/context controls pass |
| [H02](#h02) | complete | 59h | Authored examples retain hidden setup and distinct expectations; actual API reads expand examples without executing them |
| [H03](#h03) | complete | 59h | Exact metadata, inclusion paths, authored inverse feature links and shared C07 selected units delivered; actual negative-feature consumer passes without compilation |
| [H04](#h04) | complete | 59h | Native structural differences with selected-input losslessness checks and separate explicit compatibility execution delivered; actual native trials retain missing-reference and rule-coverage limits |
| [H05](#h05) | complete | 59h | Native rustdoc uses the original compiler workflow and diagnostics; actual public check and corrected native broken-link/codeblock controls pass |
| [H06](#h06) | complete | 59h | Original failure → API constraint → explicit corrected probe demonstrated; conditional non-adoptions below retained. Series-end qualification remains with the coordinator |
| [H07](#h07) | in-progress | 59h | Structural signature/bound/associated-type filters and artifact-hash-bound inspection implemented; common public join and native-context qualification remains open |
| [H08](#h08) | complete | 59h | Existing retained metadata/selected-unit owners selected for actual-context queries; explicit graph, hosted, example and archive non-adoptions with concrete revisit triggers below |

<a id="h01"></a>
### H01 — Discover a capability and a usable public path

**Consumes:** D02's working API artifact/lookup slice; A03 payload/projection and A06 parser
discovery. A04/A05 identity and acquisition requirements are consumed through D02, not copied.

**Delivers:** search by name/docs and the structural filters above; bounded candidate
signatures with public import paths, trait provenance, bounds and a source pointer. Resolve
one candidate to its fuller declaration, related docs or selected impl group. Preserve
blanket/negative/synthetic impl metadata as declaration facts, not a trait-solver verdict.
Following an external reference addresses the corresponding package/context only when needed.

Use native rustdoc HTML signature search as an optional browsing action; do not parse its
generated JavaScript or implement full Rust type normalization. Straightforward structured
filters are the CLI's initial capability-search contract. A richer signature query language is
conditional below. Useful unknowns survive rather than becoming rejected or fabricated matches.

**Acceptance:** a reexported trait-provided method is found through its usable path and
required import; two versions/sources remain distinct; a stripped declaration is incomplete
rather than empty; an item present only in documentation does not acquire a compile-success
claim. A selected probe can resolve consequential applicability uncertainty through D02.
Exercise a large nested impl/declaration population and a batch: detail/page limits apply to
the producer's declared units, with omitted detail and examined scope visible. Unknown totals,
changed source and a failed continuation cannot become a complete empty answer.

<a id="h02"></a>
### H02 — Retrieve an actionable example

**Consumes:** H01 selected item/docs and D02 addressed acquisition. Executable handoff consumes
D02's probe operation; displaying examples does not require probe execution or test listing.

**Delivers:** a small relevant authored example with source, hidden setup availability and
expectation: compile-and-run, compile-only (`no_run`), expected compile failure/panic, ignored
or conditioned. Expand setup and transformed code explicitly. API JSON and nightly rustdoc
`--output-format doctest` are separate formats with separately checked version/shape handling.
Request extraction only where authored docs cannot answer the selected setup question.

Finding an example establishes no current execution result. An expected failure may fail for
the wrong reason; preserve diagnostic/expectation distinctions if the user explicitly runs it.
Do not execute arbitrary documentation blocks during API reading or silently turn a probe into
a workspace dependency edit. Preserve uniquely authored probe inputs under D02's retention rules.

**Acceptance:** ordinary, `no_run`, `compile_fail` and ignored examples retain their different
expectations; hidden setup can be revealed; extraction failure preserves the available authored
example with explicit missing detail. Explicit probe execution reports its actual context and
result separately, leaving the workspace lock and API artifact unchanged.

<a id="h03"></a>
### H03 — Explain dependency and feature inclusion

**Consumes:** D02's working exact dependency-metadata slice and A03 presentation. The selected
unit-context view additionally consumes C07's resolved selection; it is not a prerequisite for
ordinary package facts. Reuse captured artifacts when B02/C02 already provides them.

**Delivers:** package/version/source, direct users and a short inclusion path; selected
expansions for optionality, defaults, target conditions, features and inverse activation paths.
Use `cargo_metadata` package/dependency/resolve facts as structure, with native
`cargo tree -e features -i` available as an explanatory expansion. Do not parse pretty tree
output into the authoritative dependency graph.

For selected unit-context queries, consume C07's compatible multi-root admission and shared
metadata rather than resolving each package separately. Preserve root→unit identity, host/
target context and negative/default-feature policies; incompatible roots remain separate.
Cargo metadata is a package view, C07 a selected/admitted unit view, and actual nextest listing
the test-population view. Neither a feature union nor a dependency path replaces those owners.

Keep declaration, lock resolution, selected features, planned compiler units and captured
artifacts separate. C07's nightly `--unit-graph` can explain host/target/mode/profile/unit
features without compilation; `CompilerArtifact`, `CompilerMessage` and `BuildScriptExecuted`
provide observed build facts when actual work occurred. Neither package nor unit reach proves
behavioral impact or test sufficiency. License metadata is descriptive, never rejection policy.

**Acceptance:** explain an optional target-conditioned dependency without activating it; show
a feature inclusion path and the selected unit context separately; distinguish duplicate
versions from repeated compilation; an unavailable graph is not an empty successful result.
No dependency/feature changes, provider preparation or Cargo check are implicit in the read.
Add mixed compatible/incompatible roots and a feature-negative control; a compact inclusion
page still identifies the actual graph/context examined and never claims listed test coverage.

Sources: [Cargo metadata](https://doc.rust-lang.org/cargo/commands/cargo-metadata.html),
[unit graphs](https://doc.rust-lang.org/cargo/reference/unstable.html#unit-graph),
[Cargo tree](https://doc.rust-lang.org/cargo/commands/cargo-tree.html).

<a id="h04"></a>
### H04 — Review a public interface or dependency migration

**Consumes:** H01 addressed declaration views, H03 consumer/reach explanations, and D02
baseline/current acquisition. Semantic references from G are optional enrichment; the core
comparison can work without G or any live analyzer session.

**Delivers:** a selected before/after comparison of additions/removals, public paths, signatures,
bounds, associated items, trait defaults and deprecations, plus likely workspace consumers.
Use `public_api::Builder::from_rustdoc_json`, `PublicApiDiff` and `cargo-semver-checks` where
their inspected pinned interfaces fit; keep the selected tool/version and comparison scope.
The provider adapter absorbs compatibility-tool output changes. Do not build bespoke SemVer
rules merely to fit a generic result shape.

Acquire the two sides with explicit source/version/toolchain/target/features/schema identity.
If contexts differ, show the difference before interpreting the result. Numeric rustdoc IDs
cannot match items across builds. Distinguish a structural difference, a compatibility finding,
a newly useful capability and a question needing a concrete probe. Compatibility tools have
incomplete coverage and do not establish all behavioral compatibility. This operation never
updates the workspace manifest/lock or installs a newer dependency as a side effect.

**Acceptance:** a deliberate added API, removed reexport and changed bound yield useful
before/after views; unmatched items and extraction-context differences remain explicit; likely
consumers can be followed to package/source leads; unsupported comparison input refuses with
a remedy rather than producing a clean compatibility verdict.

Sources: [public-api](https://docs.rs/public-api/latest/public_api/),
[cargo-semver-checks](https://github.com/obi1kenobi/cargo-semver-checks).

<a id="h05"></a>
### H05 — Diagnose documentation around a changed Rust API

**Consumes:** D04's working documentation-check dispatch and A03 diagnostics; C07's selected
compiler context for actual rustdoc work; D02 addressed source/context where API lookup is used.
Markdown checks retain their original owner and independent availability.

**Delivers:** explicit scoped rustdoc checking using native `rustdoc::broken_intra_doc_links`,
private-link, code-block and HTML diagnostics. Preserve source locations, severity, actual
invocation identity and suggested remediation. Optional missing-documentation/example and
coverage information is triage, not a quality score, automatic gate or new entire-workspace
campaign. API reads do not run these checks automatically.

Use original tool diagnostics and B02 extraction when it is available; do not rerun a completed
check merely to explain its message. Package/item scope is honest: selecting a package can
require broader rustdoc preparation than the displayed changed-file findings. Distinguish
selected-source findings from findings outside the requested presentation, without silently
discarding actual compiler outcome information.

**Acceptance:** a broken intra-doc link and invalid code block name the authored location and
native rule; missing-example/coverage output carries its scope and is not correctness evidence;
API lookup remains usable without executing documentation checks. No unrelated historical
documentation cleanup becomes part of the command or this package.

Source: [rustdoc lints](https://doc.rust-lang.org/rustdoc/lints.html).

<a id="h06"></a>
### H06 — Integrate the agent journey and settle conditional mechanisms

**Consumes:** H01–H05 working selected behaviors for this companion's integrated acceptance;
D07/A06 discovery/reference and E03 continuing adoption. Optional G enrichment is tested only
if selected and available, never substituted for the independent API path.

Each working H01/H03 page can serve a direct consumer earlier, without waiting for H06, full G/H/I completion,
or compiler-private preparation. Availability is explicit at each section; a failed enrichment
leaves original diagnostic/source sections useful. H retains query meaning, detail/paging decisions and typed next actions; generic packet
composition is removed from current scope.

**Delivers:** an agent discovers an unfamiliar operation, selects its public path and example,
inspects a dependency/migration question and follows a consequential uncertainty to an explicit
probe or original diagnostic. Addressed H views require no second knowledge/status ledger or
generic briefing product. Update guidance alongside each delivered verb and remove any
actually replaced helper/caller when its replacement is proven; retain independent source and
browser tools with distinct useful roles.

Record the outcomes of the conditional choices below with their inputs, reasons and dependent
scope. A reasoned no-adoption outcome can finish an investigation, not claim an unavailable
capability was implemented. Future triggers remain with the coordinator's disposition owner.

**Acceptance:** the agent reaches a useful API/probe decision without loading G; comparison
and example expansions retain context/evidence boundaries; budgets preserve necessary bounds
and status while paging optional detail. The assembled journey uses actual CLI artifacts and
tools, not only response snapshots. Report performance benefit only if explicitly measured.
Include an original-failure→API-constraint→selected next action without recompiling merely
to explain it; edited source is an explicit mismatch. No view rendering silently runs a probe.

## Conditional mechanisms and non-adoption

| Mechanism | Default and observable reconsideration condition | Decision route |
| --- | --- | --- |
| Guppy feature/reverse queries | Use metadata traversal first. Consider when repeated multi-configuration feature simulation requires relationships that the simple selected graph does not supply. An all-features simulation graph is never the selected-build graph. | H03 assesses a concrete query against `PackageGraph`/feature-graph support; select only if it removes rather than duplicates graph interpretation. |
| `rkyv_0_8` archives and fast hashing | No eager archive corpus or extra cache by default. Consider when repeated selected large-artifact reads make decode/access cost material. JSON must first be converted; archives are not zero-copy JSON. | H06 compares conversion, invalidation, retention and repeat-read value; any quantitative claim needs measurement. |
| Hosted docs.rs JSON | Optional explicit external/version research, never silent fallback for selected local context. Use only when the addressed artifact exists and provenance/format can be reported. | D02 acquisition policy supplies the distinct hosted context; H04 consumes it as exploratory evidence. |
| Scraped examples | Authored examples first. Consider when a selected API lacks a useful authored example and analyzed example targets can supply real calls. | H02 assesses native rustdoc scraping for that scope, with explicit generation; no general example-index platform. |
| Rich signature-query matching | Initial structural/name/docs filters and native browser search. Consider when actual agent queries cannot adequately express a recurring capability search. | H01/H06 assess existing search mechanisms before adding a query parser; no custom Rust trait solver. |
| Hakari | Not adopted for this capability: it changes feature declarations rather than answering read/query questions. | Reopen only under separately selected concrete build-variation work, not H completion. |

## Dependencies, verification and handoff

The independent route is D02's minimal acquisition → H01/H03 → H02/H04, with H05 connecting
to the separate D04 check route. H06 joins the selected behaviors. D02's API acquisition and
dependency metadata slices can become ready separately; neither requires completion of every
probe mode. H01/H03 design can proceed with agreed contracts, but actual CLI consumers need
working artifacts/metadata and the relevant A03/A06 dispatch. H05's native check work consumes
actual C07/correctness selection, not merely an API extraction label.

During authorized implementation, compile touched packages and run focused unit/parser tests
for artifact addressing, projection, candidate filtering, examples, identity and refused input.
Use independent expected declarations and deliberately incompatible contexts where those
challenge plausible adapter mistakes. Once selected functional scope is implemented, H06's
assembled agent journey and the coordinator's scope-end static/documentation qualification
establish their named claims. Do not add a feature-matrix, provider build or scientific campaign
solely for this developer-documentation scope.

**Current execution checkpoint — 2026-10-30:** H is being implemented under the coordinator's
remaining-series authorization. D02's addressed API acquisitions and isolated probes, name/docs
and structural search, facade/trait provenance, authored examples, source-qualified metadata,
and native documentation forwarding have working consumers. `--declaration` explicitly expands
the original schema-qualified item when a compact signature cannot represent all constraints.
`dep --why` uses Petgraph for one shortest package path per reachable workspace member;
`--feature-member` reports exact authored inverse links. Optional/default/target requests remain
separate from resolved metadata features; neither view simulates activation or compiler units.
`dep units DEPENDENCY PACKAGE[:TARGET] ...` explicitly consumes the same selection, compatible
multi-root grouping, feature-absence controls and correctness admission as checks. It returns
the selected dependency's actual planned host/target instances beside original roots, arguments
and examined-unit counts. It neither compiles, enumerates tests nor grants provider readiness.

The API comparison adapter selects two acquired artifacts and checks original context/digests.
The default read uses `public-api` 0.52.2's native public-path matching and declaration
normalization to page additions, removals and changed signatures/bounds. Its rustdoc-types0.59
decoder is qualified separately for each selected format61 document: decoding through it and
back into the pinned0.61 type must preserve the complete typed document. Unrepresentable or
lost fields refuse with a remedy. This is not blanket cross-version compatibility. Native missing
item references remain incomplete; native omission of blanket/auto-trait/auto-derived impls
is explicit. Source/context/backend-bound cursors page whole differences without retaining a
rebuildable full response. Scanning/normalization still examines both complete artifacts.

`--compatibility` instead copies the original bytes into a private comparison capture and
runs explicitly installed `cargo-semver-checks` 0.51.0 under the existing managed owner. Native
findings, exit and complete stdout/stderr stay original evidence. There is no bespoke SemVer
substitute. Actual format61 trials returned100 for a removed public reexport and0 for identical
inputs. That tool did not separately report the same fixture's added function or strengthened
generic bound; the independent structural library did report both. Structural differences,
native compatibility rules and behavioral compatibility remain different claims.
Sources: [native exact release](https://github.com/obi1kenobi/cargo-semver-checks/tree/v0.51.0),
[public-api input dependency](https://github.com/cargo-public-api/cargo-public-api/blob/main/public-api/Cargo.toml).

`api --check-docs` extends the existing selected/admitted compiler workflow with a Rustdoc action,
sharing its compiler observer, original diagnostic addresses, raw capture and physical owner.
It does not introduce another report engine. Broken/private links, invalid block attributes,
invalid Rust blocks and invalid HTML use native rustdoc rules; API reads remain independent.
Cargo controls are inserted before the literal compiler tail. Actual refreshed-control failure
and public-command acceptance remain required, along with actual selected-unit dependency composition,
full H06 journeys, generated discovery and scope-end qualification.

**Tested:** the final focused host command selected dependency, proposal, envelope, API/cursor,
graph-adapter and check-intent controls:47passed, zero failures and zero compiler warnings.
The separate compiler-workflow regression batch passed57 with zero failures; its warnings were
subsequently corrected before the final47-test batch. This is targeted acceptance, not P01 or
all-H completion. Enduring operating guidance is published by E03 during actual delivery; source-review recommendations
and this plan do not themselves establish Tested or Measured capability.

**Native consumer acceptance and correction — 2026-10-30:** actual public serde_json acquisition
finished with child0, complete original capture and settlement; a two-candidate declaration/example
page returned27omissions and a bound continuation. Same-artifact structural comparison preserved
362unresolved native item references on each side as incomplete; native compatibility rules
separately returned0 with complete capture/settlement. `pse dep units tokio pse-host --lib
--no-default-features --without-feature diagnostic-taskdump` returned the original selected root
and436examined units without compilation. `pse api pse-ids --check-docs` completed through the
original compiler workflow with child0 and complete capture/settlement. These are their stated
consumer scenarios, not whole-series qualification.

The first deliberate bad-document test incorrectly expected a token-valid `let = ;` example to
trigger rustdoc's invalid-codeblock lint. Native rule inspection showed that this lint is not a
complete Rust grammar/type check. The fixture now uses an unterminated character literal; both
the original broken-link and invalid-codeblock rule/source addresses pass through the existing
compiler observer, without a test-success verdict. The failed original run remains retained.
Incoming metadata declaration joins also now require alias, matching version requirement,
native kind/target and explicit source/path agreement; conditional same-alias requests are not
misattributed. Unknown joins remain unknown. The six focused correction controls passed;
the remaining unused foreground-read prerequisite warning belongs to the immediately dependent
probe-custody consumer and remains a zero-warning scope-end obligation, not a baseline.

### Outcome — selected API and dependency workflows

**Implemented:** the selected H01–H06 surfaces and their original-artifact, explicit-effect and
shared-selection consumers are integrated. Conditional candidates retain the defaults and
observable triggers above; no archive corpus, second feature solver, hosted fallback or signature
query language was added without a demonstrated consumer. Original raw declarations supply
constraints that the compact renderer cannot faithfully summarize.

**Tested, 2026-10-30:** the actual combined journey used `pse probe new plan59-h06 --deps
serde_json`, an authored `Rc<str>` decoding attempt and `pse probe run plan59-h06`. Original
invocation `01a18f30-d533-7032-bfe4-b99c5daae594` returned101 with complete capture and settlement.
`pse log ID --stderr` retained the actual E0277 and the `Deserialize` constraint. A passive API
read initially returned unprepared for the changed locked context; it did not generate an artifact.
After explicit `pse api serde_json --prepare`, `pse api serde_json serde_json::from_str
--declaration --examples --budget 1` returned the original `T: Deserialize<'a>` predicate,
authored examples and exact artifact context. The compact signature correctly remained incomplete.
Changing the explicitly authored probe to `String` and running it explicitly produced invocation
`01a18f31-e0a2-75cc-9d31-bf3172e4faed`, child0, complete capture and settlement. Reads neither
reran the failure nor executed examples. These expected failed/successful probes are consumer
controls, not unresolved test failures. The native structural/compatibility and selected-unit
journeys above supply the other H consumers; whole-series static qualification is separate.

**Mistakes corrected:** the invalid-codeblock fixture used token-valid text and initially tested
more grammar checking than rustdoc promises; the corrected lexical failure exercises its actual
rule. Initial cross-version library concerns were replaced by a per-input losslessness guard,
allowing native structural comparison without claiming general rustdoc-format compatibility.

**Deliberate limits:** native compatibility rules are neither a complete structural diff nor a
behavioral compatibility proof; source/context differences and missing native item references
remain visible. No performance benefit was measured. Probe resources with legacy missing custody
references remain protected rather than being retroactively attributed to a newer owner.

## Planned search extension packages

H07/H08 consume the [query/result foundations](59k-query-and-result-foundations.md) and
feed [composed investigations](59m-composed-rust-investigations.md). The preceding outcome
and completed rows concern H01–H06, not this new scope.

<a id="h07"></a>
### H07 — Structural capability, use and importability composition

**Consumes:** K01's agreed query meaning for design; working K02 requests/results/locators,
H01/H03's exact acquired API/dependency facts and G10's name-first semantic candidates for
the early public composition. Existing G03/G05 may supply actual uses. Only selected native
typed enrichment later consumes G12; basic structural API/dependency delivery does not wait for
G11/G12. A10 publishes changed governing contracts before dependent implementation.

Replace input/output spelling-containment as the sole capability filter with selected structural
`rustdoc_types::Type`, generic bounds/`WherePredicate`, trait/impl and reexport-graph predicates.
Keep name/docs search useful and explicit; a named type occurrence, a structural signature shape
and a proven applicable trait obligation are different answers. Rustdoc extraction cannot prove
arbitrary trait applicability or selected-build importability. Retain artifact-local item IDs,
source-qualified package/version, defining versus public facade path, target/features/cfg and
documentation-only context. Joining artifacts by numeric ID or rendered type text is invalid.

Join selected API candidates to actual source uses and destination-module importability using
their existing semantic/context owners. Preserve directed/multiple reexport paths and ambiguity;
one public documentation path does not establish accessibility from every destination. If the
available LSP cannot answer the typed join, return the structural candidate plus unavailable
enrichment, not an invented positive. G12 may later supply selected visibility/import maps,
resolved types and generic constraints through the same contract. Batch compatible lookups over
one acquired artifact/model and narrow candidate sets before semantic expansion where sound;
do not build a second graph, trait solver or per-hit acquisition pipeline.

**Public consumer and acceptance:** `search` finds an operation by a selected signature/bound,
`inspect` expands its constraints and reexport path, then follows a source use or asks whether
it is importable into a selected module. Use independent fixtures with identical spellings but
different type identities, associated types/bounds, a reexported dependency operation, private
destination visibility, inactive cfg, renamed dependency and two target/feature contexts.
Malformed/schema-mismatched or missing rustdoc references stay explicit; `cfg(doc)` success
does not imply ordinary-build availability. A missing session/artifact remains unprepared and
does not acquire or compile implicitly. Stale result/artifact/context joins refuse. Actual G12
facts receive their own focused native tests; their absence cannot fail a supported base query.

**Migration/deletion:** move passive API/dependency reads and relevant consumers into M03's
common search/inspect surface, including inclusion paths, authored feature facts and selected
units. Preserve direct known dependency/item inspection and distinct structural comparison,
examples and original diagnostics. Delete replaced parser branches, duplicate spelling-only
structural filtering, formatters, guidance and tests once the replacement passes. Explicit
acquisition, compatibility/check execution and probes keep their original owners/effects.

**Implemented:** common API discovery composes the retained artifact/query owner rather than
acquiring per hit. `syn` parses selected Rust type syntax; predicates compare native rustdoc
type trees for paths, type arguments, references/lifetimes, pointers, tuples, slices and explicit
`<Self as Trait>::Associated` shapes. Parameter/where-clause bounds inspect actual declarations;
unsupported shapes refuse without a spelling fallback. Original declarations, constraints and
directed reexport paths remain inspectable. Portable local item references carry original
artifact SHA, so replacing an artifact directory cannot reinterpret an old item ID. These are
declaration/context facts, not a trait solver or destination visibility proof. Source-use and
importability enrichment stays an explicit semantic operation with unsupported states unknown;
G12 facts keep their own selected context. Public artifact/source-join acceptance remains open.

The request-owned source bridge now reuses each acquired artifact and its original public
reexport route across candidates. It compares the retained Cargo documentation-unit identity
with actual selected native-library package/source-root/features/target facts and transitive
dependency profiles from the same loaded world, then checks span bytes against both original acquisition and native source
digests. Session invocation participates in cache identity; a new preparation cannot inherit an
old bridge merely because its requested configuration has the same spelling. Real native
declaration identity drives use joins, while the native destination module supplies visibility
and import-path facts for unused items. Unsupported associations, ambiguous contexts and missing
native facts remain unknown structural candidates. Dependency/reexport/private-destination and
two-context public qualification is still required before H07 completion.

<a id="h08"></a>
### H08 — Conditional graph and acquisition extensions

**Consumes:** K01's supported question model and current H03/D02 graph/acquisition evidence.
Use working K02 for any adopted public query integration. This decision does not depend on
native HIR adoption, full H07 delivery or a new source graph. Earlier conditional-mechanism
rows remain baseline decisions; this package records their disposition for the selected scope.

Compare current `cargo_metadata`/selected-unit and shortest-path owners with Guppy
`PackageGraph`/feature-graph support for a named repeated multi-configuration, reverse-feature
or counterfactual question. Adopt only when it removes a concrete interpretation burden;
keep simulated feature sets distinct from selected/admitted compiler units. Hakari is not
selected: feature-declaration unification supplies no required search capability here.

Assess hosted docs.rs JSON only for an explicit external/version-research acquisition that
local exact locked context cannot supply usefully. Retain addressed version/target/build
premises, availability and `format_version` checks; hosted artifacts never silently replace
local selected-build evidence. Reuse D02's explicit acquisition/storage boundary. Native rustdoc
scraped examples remain conditional on a selected API lacking a useful authored example and
explicitly generated analyzed targets. Archives/hashing are candidates only for material
repeated large-artifact access; conversion/invalidation/retention costs count in that decision.

**Decision and acceptance:** state adoption or no-adoption per named question, the consumed
exact-release mechanism, affected consumer and observable revisit trigger. Current metadata
remains selected where sufficient; library availability, an overview version or an empty docs
search is not an adoption criterion. A Guppy branch challenges optional/default/target and
multi-root host/target contexts against independent expected feature relationships. A hosted
branch challenges absent artifact, mismatched format/context, differing local versus hosted
builds and explicit no-fallback behavior. Reading an unavailable artifact never starts generation.
No performance benefit is claimed without a named comparison under representative conditions.

**Conditional implementation/deletion:** after a positive decision, use the normal dependency
and applicable decision routes and implement only that bounded producer through K02/D02/H03.
Remove graph/acquisition interpretation that it actually replaces, including old consumers and
tests; retain distinct local/hosted provenance and effects. A no-adoption outcome completes the
inquiry, leaves unsupported queries unavailable and introduces no dependency, daemon or index.
The coordinator owns scope-end qualification of adopted branches and M03 public retirement.

#### H08 outcome — preserve actual-context owners

**Interface-checked:** the concrete dependency question is “which retained workspace package
paths include this exact source-qualified package, which authored feature members mention
it, and which selected host/target compiler unit is being inspected?” Existing
`dependency::inspect_details` answers package reach with native petgraph traversal and exact
authored feature-member inverse links over one retained `MetadataSnapshot`; selected compiler
units retain their distinct admission/context owner. Existing independent tests
`package_paths_and_authored_feature_links_do_not_simulate_activation` and
`omitted_graph_is_unknown_not_empty_features_or_no_consumers` exercise the distinction; their
current integrated test result is owned by the scope-end run, not this interface assessment.

**Decision:** no Guppy producer is adopted for that selected question. Guppy's
[`PackageGraph`/derived feature graph](https://docs.rs/guppy/0.17.23/guppy/graph/struct.PackageGraph.html)
can model forward/reverse dependency and feature relationships; it is a genuine extension
for counterfactual resolver/target/feature comparisons, not a missing Cargo fact parser.
The directly examined primary API is 0.17.23; the overview's later version leads are not
claimed tested or selected pins. The current answer does not simulate alternate resolutions,
and adding a second graph would not improve its selected-unit authority. Revisit for a named
comparison of candidate feature/target/root selections that requires those simulated
relationships. Keep the simulation context explicit and qualify optional/default/host/target
relationships independently if adopted. Hakari remains unselected because manifest feature
unification is not this consumer's query.

**Decision:** no hosted JSON acquisition branch is adopted. The selected API query has an
explicit local, format-checked artifact and retained source-qualified context; no missing
external/version-research artifact has been selected. Hosted
[`docs.rs` JSON](https://docs.rs/about/rustdoc-json) is available by addressed version, target
and format, can be absent, and can use another rustdoc format. Revisit for an explicitly
selected external/version research question unavailable usefully from local exact context;
adopt a distinct D02 acquisition with provenance and no selected-build fallback. Likewise,
native scraped examples remain unselected until a named API lacks a useful authored example
and analyzed targets are explicitly generated. Archive/fast-hash conversion remains
unselected until repeated large-artifact reads present a measured decode/access problem;
no zero-copy-JSON or performance benefit is asserted. These are completed bounded decisions,
not claims that the libraries lack the capabilities. They add no dependency, acquisition,
daemon, index or automatic compiler work; H07/M03 integration and qualification remain their
own obligations.
