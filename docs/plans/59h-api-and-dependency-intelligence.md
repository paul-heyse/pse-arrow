---
title: "Plan59h: API and dependency intelligence"
status: active
date: 2026-10-29
doc_role: plan
doc_owner: docs/plans/59-agent-cli.md
doc_topics: [current-work, design, libraries, environment, validation]
review_sources: [docs/functional_reviews/rust-code-intelligence-cli-2026-10-29.md]
---

# API and dependency intelligence

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
test/build/scientific authority. Plan59 production execution remains stopped. This authoring
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

## Target and shared contracts

Keep `pse api` and `pse dep` as the existing task surfaces, extended with selected query modes.
The command grammar is owned by A02/A06 and must describe the actual parser; illustrative
flags in the source review are not a second grammar. Provide name/docs search plus structural
filters for item kind, input/output type references, trait and safety/async/const qualification.
Dependency inspection supports selected why/features/users expansions. API comparison and
example expansion are explicit operations; neither performs an update or executes code.

Return one compact answer: useful signatures or dependency paths, relevant constraints,
source/public-path pointers, acquisition context and a few next actions. Detailed impl groups,
docs, examples, paths and raw artifacts expand on demand. Reuse A03's envelope, fields and
detail budget; small output does not imply a narrow underlying scan. Report known omissions,
unknown totals, absent matches and incomplete/unavailable acquisition distinctly. Batch
related items against one addressed artifact rather than regenerate per item.

| Meaning | Owning contract and consumed behavior |
| --- | --- |
| Acquisition and reuse identity | D02 supplies exact package/version/source, features, target/cfg, toolchain, extraction settings and rustdoc schema version through A04/A05. Check `format_version` before typed decode. |
| Documentation context | Preserve extraction flags and documentation-only conditions. Known gates are reported; unknown ordinary-build availability remains unknown. `cfg(doc)` presence is not proof that the API compiles in the selected application. |
| Item identity and public path | Rustdoc IDs are local to an artifact. Reexports and defining locations are separate; cross-artifact matching is qualified rather than numeric-ID equality. |
| Query result and evidence | A03 presents declared API structure, candidate ranking, compatibility diagnostics, planned units and observed compiler results as distinct facts. No rendered docs string becomes an inferred semantic authority. |
| Compiler/test selection | C07 owns the selected command context; existing correctness admission governs actual probes/checks. Unit graphs are plans; captured compiler artifacts and test listing are observations. |
| Source-use enrichment | G's references may be requested when available. Basic API/dependency work has no G session prerequisite; missing semantic enrichment stays explicit. |

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
| [H01](#h01) | planned | 59h | Capability search and usable public-path/constraint views over D02 artifacts |
| [H02](#h02) | planned | 59h | Relevant examples with setup, expectation and selective extraction |
| [H03](#h03) | planned | 59h | Dependency/feature inclusion explanations with distinct context precision |
| [H04](#h04) | planned | 59h | Explicit API/dependency migration briefs using existing compatibility tools |
| [H05](#h05) | planned | 59h | Scoped native rustdoc diagnostics integrated with D04 |
| [H06](#h06) | planned | 59h | Agent consumer journeys, conditional decisions and integration into D01/E03 |

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

Keep declaration, lock resolution, selected features, planned compiler units and captured
artifacts separate. C07's nightly `--unit-graph` can explain host/target/mode/profile/unit
features without compilation; `CompilerArtifact`, `CompilerMessage` and `BuildScriptExecuted`
provide observed build facts when actual work occurred. Neither package nor unit reach proves
behavioral impact or test sufficiency. License metadata is descriptive, never rejection policy.

**Acceptance:** explain an optional target-conditioned dependency without activating it; show
a feature inclusion path and the selected unit context separately; distinguish duplicate
versions from repeated compilation; an unavailable graph is not an empty successful result.
No dependency/feature changes, provider preparation or Cargo check are implicit in the read.

Sources: [Cargo metadata](https://doc.rust-lang.org/cargo/commands/cargo-metadata.html),
[unit graphs](https://doc.rust-lang.org/cargo/reference/unstable.html#unit-graph),
[Cargo tree](https://doc.rust-lang.org/cargo/commands/cargo-tree.html).

<a id="h04"></a>
### H04 — Review a public interface or dependency migration

**Consumes:** H01 addressed declaration views, H03 consumer/reach explanations, and D02
baseline/current acquisition. Semantic references from G are optional enrichment; the core
comparison can work without G or any live analyzer session.

**Delivers:** a selected before/after brief of additions/removals, public paths, signatures,
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

**Consumes:** H01–H05 working selected behaviors; D01's brief composition for packet handoff;
D07/A06 discovery/reference and E03 continuing adoption. Optional G enrichment is tested only
if selected and available, never substituted for the independent API path.

**Delivers:** an agent discovers an unfamiliar operation, selects its public path and example,
inspects a dependency/migration question and follows a consequential uncertainty to an explicit
probe or original diagnostic. D01 assembles selected H sections rather than H maintaining a
second knowledge/status ledger. Update guidance alongside each delivered verb and remove any
actually replaced helper/caller when its replacement is proven; retain independent source and
browser tools with distinct useful roles.

Record the outcomes of the conditional choices below with their inputs, reasons and dependent
scope. A reasoned no-adoption outcome can finish an investigation, not claim an unavailable
capability was implemented. Future triggers remain with the coordinator's disposition owner.

**Acceptance:** the agent reaches a useful API/probe decision without loading G; comparison
and example expansions retain context/evidence boundaries; budgets preserve necessary bounds
and status while paging optional detail. The assembled journey uses actual CLI artifacts and
tools, not only response snapshots. Report performance benefit only if explicitly measured.

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

**Current state:** every H package is planned. No code, acquisition, tool installation, probe,
compatibility check or consumer journey is delivered by authoring this document. Enduring
operating guidance is published by E03 during actual delivery; source-review recommendations
and this plan do not themselves establish Tested or Measured capability.
