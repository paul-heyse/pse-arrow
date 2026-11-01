---
title: Code intelligence for engineering investigation and decision making
status: proposed
date: 2026-10-31
doc_role: review
doc_topics: [design, libraries, environment, diagnostics]
---

# Code intelligence for engineering investigation and decision making

## Judgment, scope and evidence

**Proposed:** extend the CLI around questions about responsibilities, contracts, changes and
library fit. The valuable addition is a compact, source-qualified account of relationships that
an agent can challenge and expand. More isolated navigation commands, automatic architectural
verdicts and generic review briefs would add little value.

The audience is agents conducting design-principle audits, design reviews, functional-extension
reviews, the three library-leverage stages, execution planning, plan preparation/creation and
plan-improvement reviews. The operator selected code/document evidence connections and targeted
skill-guidance recommendations alongside potential CLI extensions. These are recommendations,
not authorization to change skills, production code, Plan59 or architectural decisions.

Inspected source baseline: `9d5dcf27c`, with no included production changes. The two untracked
library overviews present at entry were preserved. Plan59's [P02](../plans/59-agent-cli.md#p02)
owns prior assembled qualification; this review neither repeats it nor reopens completed scope.
Criteria: Core3.4, Efficient Architecture1.0 and the pse-arrow binding. Test Assessment1.2 informs
test-evidence proposals; Process Simulator1.5 limits scientific interpretations. No source graph
can establish physical validity, conservation, convergence or solver correctness by itself.

**Interface-checked:** current typed query/composition, native semantic and API owners, document
ownership, process-skill contracts and the library mechanisms cited below. **Tested during this
review:** actual source-only `search symbol compiler_environment --path
crates/pse-host/src/correctness.rs --limit 2` followed by `inspect --result` using the returned JSON
reference succeeded. Changing only that reference's source digest to an incorrect value refused
with exit125 and a stale-source diagnostic. Mixed Rust-manifest/Markdown `search text native-hir
--literal` returned guarded witnesses from both sources. These are bounded current-CLI observations,
not implementation tests of the recommendations. No new analyzer preparation, compiler campaign,
dependency installation, scientific test or performance benchmark was performed.

## What the delivered CLI already supplies

The existing `investigation::model::Request` separates selection, predicates, demand and
presentation. `Predicate` already has scoped All/Any/Not, guarded refinement and bounded Related
traversal. Related currently names Definition, References, Calls and Implementations. Configured
queries, directional schemas, fields/grouping, streamed records and final coverage already exist.
The source and native references preserve different freshness and context premises.

The current semantic owners provide name discovery, native type/target/read-write observations,
implementations and destination importability; the live provider additionally has its supported
call, hint, completion, macro, predicate and edit operations. Capability availability differs by
provider. API queries already have structured type patterns, original declarations and qualified
facade/definition evidence; comparison and explicit compatibility execution already exist. Syntax
rules, repeated captures, preview validation, history, generated origin, retained diagnostic reads
and result-to-current-test-selection are also delivered. They should not be proposed again.

The important current seams are `investigation/{model,composition,rust}.rs`,
`semantic/{native,engine,edit}.rs`, `api/{query,bridge,comparison}.rs`, the CLI inspection adapters
and their original source/test/document owners. In particular,
`xtask/src/documentation/ownership.rs` exposes native document role, owner, status and section
meaning and explicitly disclaims inferring a code-path-to-plan binding. That distinction should
survive richer connections.

## Workflow value and coverage

| Process | Decision that richer evidence could improve | Principal opportunities |
| --- | --- | --- |
| Design-principle audit | Follow a candidate authority into enforcement and consumers; distinguish a common cause from similar spelling | O01–O03, O06 |
| Design review | Compare boundaries and lifecycle responsibilities, retaining contrary evidence and configuration limits | O01–O04, O09 |
| Functional-extension review | Discover combinations and missing capabilities without requiring an existing caller | O05–O08 |
| Leverage-codebase-audit | Locate generic responsibilities, wrappers and repeated coordination in their full operation | O01–O03, O05 |
| Leverage-library-assessment | Connect an identified need to exact APIs, constraints, documented conditions and current use | O05, O06, O11 |
| Leverage-opportunity-review | Compare complete options and residual responsibilities using the same scoped evidence | O02–O06, O11 |
| Plan-creation / create-plan | Turn a recommendation into coherent producer/consumer/contract/generator/test changes and prerequisites | O02, O04, O06, O09 |
| Plan-execution | Resolve readiness, inspect original failures, select a local change and revealing validation | O03–O05, O07–O09 |
| Plan-improvement-review | Reconcile remaining work with current foundations and compare total change/migration burden | O02, O04, O06, O09 |

The skills remain the reasoning and publication owners. The CLI supplies evidence and useful
operations; it does not decide compliance, recommendations, accepted scope or completion.

<a id="O01"></a>
## O01 — Typed structural relationships beyond navigation

**Recommend, high foundational value.** Add queryable declaration relationships such as field
membership/access, parameter and return-type dependencies, associated items, trait/supertrait
declarations, visibility and enclosing definitions. Separate a declaration's written type,
inferred expression type, adjusted receiver and substituted use. This helps an audit follow a
policy from representation to use, and helps a planner distinguish changing a field from merely
renaming an unrelated symbol.

Illustrative question: “Which public operations consume this policy type, which fields do they
read or write, and which boundary types expose it?” Return typed edges, exact witnesses, selected
crate context and unknown portions, then allow expansion of one operation. This is proposed
interaction, not current command syntax.

Native READ/WRITE categories are static observations, not complete mutation analysis through
aliases, Deref, overloaded operations or interior mutability. Show the relevant role and source
context instead of declaring that all mutations or owners have been enumerated.

Use HIR `Module`, `ModuleDef`, `Adt`, `Field`, `Function`, `Trait`, `Impl`, `Type` and `Semantics`
operations, original IDE reference categories and rustdoc `Type`/generic representations.
The concrete API families and limits are covered in [HIR §§3–7](../libraries/rust_analyzers/ra_ap_hir.md)
and [rustdoc-types §§2–6](../libraries/rust_analyzers/rustdoc-types.md). Extend the existing
relationship/result owner rather than create a second entity model. Syntax-only candidates
remain useful when no semantic world exists, explicitly without resolved-identity claims.

**Critical qualification:** the pinned IDE incoming-call implementation derives enclosing
functions from reference usages; it is not an independent proof that every usage invokes the
target. Outgoing analysis has its own syntactic/callable restrictions. Preserve “referenced by,”
“selected explicit call” and implicit-operation resolution as distinct evidence kinds. Inspect
the exact [0.0.358 call hierarchy source](https://docs.rs/crate/ra_ap_ide/0.0.358/source/src/call_hierarchy.rs)
before extending or relabelling these observations. A callback value, function pointer or
same-spelled method is a required counterexample. Neither edge kind proves runtime reachability.

<a id="O02"></a>
## O02 — Multi-seed responsibility and boundary investigations

**Recommend after O01; first try composition of existing relations.** Agents currently assemble
several discoveries and neighbors mentally. Support a bounded question over selected seeds:
show shared consumers, incoming versus outgoing boundary edges, relevant paths and omitted
frontiers, with grouping by containing operation/module/package. Keep edges and witnesses reusable
between views instead of serializing every path repeatedly.

For `Request`, an agent should be able to examine construction, validation, provider dispatch
and presentation consumers together. That can reveal where a proposed request change propagates,
without claiming that every reference is an enforcement point. For cancellation, start with the
CPU completion owner and native signal, inspect the relationship between them, then expand the
specific cleanup or error branch.

Existing bounded Related traversal, source/entity conjunction and package membership should
absorb the base capability. Native multi-location/reference queries or one compact derived
adjacency view may serve repeated questions. Compare those with repeated scalar requests before
selecting a bulk mechanism. A graph should be a question-shaped projection with edge kinds,
context, cycles and frontier meaning, not a universal repository ontology. AP-01/AP-03/AP-07 and
DP-07/DP-10 shape this recommendation. A path leaving the output package must remain traversable
when required to answer the question.

<a id="O03"></a>
## O03 — Reviewable effects, captures and lifetime slices

**Recommend a bounded native extension; defer general program analysis.** A lifecycle reviewer
needs more than callers: observed capture kinds/types, adjustments and written control structure
help reason about movement, borrowing and retention; they do not establish those properties
transitively. Error propagation and cancellation-versus-completion remain separate questions.

Expose supported HIR closure captures, local bindings, native read/write roles, explicit await,
return/error and drop-related observations with source witnesses. Use lossless AST structure for
written control structure and HIR for resolved meaning. HIR §§5.6/7.3/8 and IDE semantic inspection
describe the enabling capabilities. The useful answer is a selected function/body slice with
unresolved boundaries, not a synthesized claim that all spawned work drains.

The native cancellation work supplies a discriminating case: a signal can propagate while the
CPU task still owns unfinished work, and a local suppression/reset need not affect a sibling
request. A retained future or guard is another useful case. Show these distinctions to the agent;
do not infer task settlement from a call to `cancel` or physical memory release from source scope.
`drop_glue` is a type property, not observed destructor execution; `&T` does not exclude mutation.

Exact loans, move-path initialization and liveness at a MIR location require a separate compiler
consumer. `get_bodies_with_borrowck_facts`, native dataflow cursors and query timing are leads,
not a selected implementation: [exact pinned compiler consumer source](https://github.com/rust-lang/rust/blob/c1070d69382b8d2f2eb65119c738a77d9e324c9e/compiler/rustc_borrowck/src/consumers.rs).
The compiler overview is not at the checkout's compiler revision. Retain original diagnostic/MIR
and explicit probe routes until a concrete machine-readable location-specific question earns
the extraction/component lifecycle. No new driver or compiler component is proposed as a prerequisite.

<a id="O04"></a>
## O04 — Source and context comparisons for a proposed change

**Recommend; extend existing change/API comparison owners.** Separate written changes, changes
to resolved declaration relationships, API differences and changes caused solely by features or
target selection. An agent planning a contract edit needs “what changed and what now consumes
it?” rather than an undifferentiated changed-file list.

Compose Git source origins, lossless declaration/capture comparisons, existing original API
comparison and selected semantic observations. Before/after semantic worlds require explicit,
separately prepared contexts; a revision blob cannot inherit today's world. Names and byte ranges
are candidate correspondences, not enduring semantic identity across revisions. Renames, moves,
cfg changes and incomplete macro expansion must retain ambiguous/unmatched cases.

Begin with source declaration and acquired-API differences that need no new service. Add selected
semantic comparisons where they change a decision. Do not rerun a compiler or reconstruct every
workspace configuration for a passive diff. A feature-only change that alters a dependency while
the edited crate's spelling is unchanged is the important negative case.

<a id="O05"></a>
## O05 — Library-fit questions that connect requirements, APIs and actual use

**Recommend, with guidance-first delivery for the existing base.** The current signature patterns,
original declaration/docs, native importability and use joins already answer substantial parts
of library assessment. Teach their composition before adding another “library assessment” verb.

The useful extension is navigable API relationships and conditions: associated types and methods,
supertraits, generic bounds, reexports, documentation links and relevant impl records, joined to
actual local usage and wrappers. An agent could start with a required input/output capability,
inspect a candidate's documented constraints, identify existing integrations and compare the
responsibilities the wrapper would retain. An operation's async/unsafe/const classification or
signature shape does not establish operational fit.

Use rustdoc item IDs, `index`, `paths`, `external_crates`, structured `Type`/`Generics`/`WherePredicate`
and documentation-link records, with HIR destination checks where supported. Follow compiler-resolved
links as exact links and plain prose links as document references. Original facade/defining artifact
qualification must survive navigation. Generic applicability, missing defining artifacts and omitted
blanket impls remain unknown, even when native usage or an import path is established.

Sources: [rustdoc semantics and linking](../libraries/rust_analyzers/rustdoc.md),
[typed API model](../libraries/rust_analyzers/rustdoc-types.md),
[dependency tooling](../libraries/rust_analyzers/rust_dependency_tooling.md), current
`api/{query,bridge,comparison}`. Counterfactual Guppy feature resolution is an optional future
question, not a substitute for actual admitted Cargo units or a reason to replace current owners.

<a id="O06"></a>
## O06 — Code, contract, plan and library evidence connections

**Recommend a read-only association view.** Bring declared generated-source origin, document
ownership, section references, plan scope bindings and literal source citations into the same
investigation. Preserve why each connection exists: declared ownership, explicit citation,
resolved Rust relationship or heuristic nomination. Do not turn a search hit into an authority link.

For a proposed request change, show the relevant source declaration, authored schema/generator,
contract sections, applicable plan packages and qualification records with their own status and
revision. A review statement remains a proposal; an accepted ADR remains rationale; test evidence
retains its original scope. The view should link to original records instead of copying mutable
completion state into a new ledger.

Reuse the native document metadata/lifecycle/section owners and existing generated-origin and
history readers. Markdown parsing, heading addresses and explicit references can nominate links;
agents can accept a connection in their review prose without maintaining a new mapping database.
A document mentioning a type in a rejected alternative is the revealing false positive. The
current document-owner disclaimer is useful evidence of the remaining join responsibility,
not a defect to remove by guessing. This directly serves plan creation and improvement while
preserving DP-01/DP-05/DP-21.

<a id="O07"></a>
## O07 — Parameterized investigations and focused skill guidance

**Recommend first; primarily adoption and composition.** Give each process skill a small set
of decision-oriented examples: authority→consumers, wrapper→library API, proposed change→affected
contracts, original failure→source/API→test selection. Document when to stop expanding and when
syntax, HIR, rustdoc or compiler evidence is appropriate. These are optional inquiry techniques,
not mandatory stages or canned conclusions.

Saved queries/scopes and typed JSON already exist. Extend only a demonstrated gap, such as
binding returned seed references to several related clauses without hand-copying coordinates.
Retain one validator; do not introduce a process-specific DSL, separate query registry or
automatic workflow engine. Figment's source provenance and Clap's actual argument value sources
already support explicit/default distinction. Report effective query and capability requirements
before expensive execution. A useful macro-free source question must remain usable without
preparing an analyzer.

Targeted changes would belong to `pse-cli` and relevant process-skill references after selection.
No edits are made by this review. An example that merely restates a skill's prose or produces a
generic brief does not earn CLI code. Skill authors retain discretion over investigation depth.

<a id="O08"></a>
## O08 — Evidence-sized answers with meaningful progressive expansion

**Recommend incremental interface work, building on existing projection.** The output should
answer the selected question with grouped witnesses and useful next actions, then expose an
individual relationship, definition, diagnostic or context on demand. Existing limits, fields,
grouping and references are foundations, not missing features.

Improve places where clients still need ad hoc JSON extraction or a large legacy inspection
response to obtain a few relevant facts. Distinguish answer selection, witness context and
rendered-output allowance; do not erase coverage to meet a size preference. Return original
facts separately from an agent's proposed interpretation. For comparative questions show a
small aligned view of alternatives and differences, not concatenated whole artifacts.

Serde typed views/borrowed parsing and `RawValue` can retain opaque original evidence;
Schemars input/output-direction schemas and typed constructors retain validation ownership.
Clap parser-derived help/examples should expose useful query forms and capability limits.
Thiserror/Miette supply related causes and labelled source diagnostics, not a second error
taxonomy or an automatically serialized domain protocol. Indicatif can present actual
stage/activity/bytes on stderr, never invent completeness or success. Preserve indivisible
stream records and actual settlement. Rendering fewer fields must reduce upstream work only
where native semantics permit it. No numeric token SLA or measured speedup is established here.

<a id="O09"></a>
## O09 — Explain validation relevance and setup responsibility

**Recommend evidence enrichment, not test execution redesign.** Current containing-package and
conservative affected selection are valuable. Add navigable distinctions between direct references
from tests, shared fixture/setup dependencies, a contract assertion and an agent's proposed
revealing check. This helps plan writers select local qualification and recognize shared-helper
fan-out without treating filename matches or coverage as independent assurance.

Reuse native test inventories, Cargo units, fixture declarations, source relations and retained
original outcomes. The recent physical-Cargo failure is a useful example: a production compiler
selection correction and a stripped-PATH fake's missing compiler inputs are different causes.
An investigation should lead to the original phase/result and helper definition rather than
silently rerun tests. Exact current selection still belongs to the existing owner.

A test can reference an operation without asserting its important invariant; a test and its
expected value can share the same faulty production calculation. Report links and observable
assertion structure, leaving oracle independence and sufficiency to the reviewer. No new runner,
event reconciliation, automatic coverage gate or test-control framework is recommended.

<a id="O10"></a>
## O10 — Native bulk views and shared delivery only when the workload earns them

**Conditional investigation, not a foundation prerequisite.** Compare request-local multi-seed
queries and resident Analysis reuse with a compact native static projection for repeated global
cross-reference questions. [IDE §6](../libraries/rust_analyzers/ra_ap_ide.md) describes
`StaticIndex::compute`, token definitions/body ranges/references and monikers. These can remove
custom extraction, but an index is not an automatic persistent store or multi-context truth.

The pinned [static-index source](https://docs.rs/crate/ra_ap_ide/0.0.358/source/src/static_index.rs)
visits each file once and makes first/selected-crate choices. Do not assume it preserves every
crate-instance interpretation of a multiply included source file. Exports must state selected
configuration and omitted contexts; a moniker is not identity across arbitrary edits or builds.
Keep non-Send/non-Sync native handles within their owner and export only suitable owned views.

Do not add a persistent graph, index synchronization protocol or Tonic service merely to share
a query result. The existing local helper and complete record protocol are the simpler baseline.
Prost descriptors and Tonic typed streaming/reflection are credible candidates if an actual
multi-client/remote consumer requires them. Compare lifetime, admission, authentication,
cancellation/drain and schema-evolution burden first. Current overview versions are not assumed
to match pins. Repeated extraction or one genuinely global investigation whose native batch
route removes scalar coordination can justify a bulk-view prototype. Qualitative work structure
can select it; measurements are needed for speed claims, not as an adoption gate. A real
remote/multi-client contract is the separate transport trigger.

<a id="O11"></a>
## O11 — Rich API shapes and explicit contextual compatibility

**Recommend targeted extensions to the existing matcher and native comparison owner.** Callback
signatures, higher-ranked bounds, arrays/const arguments and associated-type equality versus
bounds are useful library-fit questions beyond the examined TypePattern forms. Its existing
paths, type arguments, references, tuples, slices, pointers, qualified associated types and `_`
must remain the foundation; a new spelling matcher would lose that structure.

Use rustdoc `GenericArgs`, `AssocItemConstraint`, function-pointer/array types and scoped generic
bounds, with syn parsing the requested shape. Preserve binder scopes, negative/synthetic impl
meaning and document-local item IDs. For example, `Iterator<Item = T>` and
`Iterator<Item: Clone>` ask different questions. A repeated wildcard is not automatically a
unification variable. The manual alternative is full declaration inspection plus an explicit
small compiler probe when actual applicability matters. That remains appropriate for rare forms.

For contextual comparisons, exact HIR0.0.358 offers `could_coerce_to`,
`could_unify_with_deeply`, `impls_trait`, `has_any_impl` and rebasing/substitution operations.
These are not equivalent predicates: existential `has_any_impl` is weaker than a generic
guarantee; permissive unification can admit unresolved types; blanket impl enumeration and
GAT normalization have limits. Compare types only in compatible native origins or through an
explicitly supported rebase. Return the operation asked, actual premises and inconclusive cases,
not a general “compatible” boolean. Existing LSP predicate queries answer part of this already.

Sources: [HIR §§5–6](../libraries/rust_analyzers/ra_ap_hir.md),
[rustdoc Item/links](https://docs.rs/rustdoc-types/0.61.0/rustdoc_types/struct.Item.html),
[document-local Id](https://docs.rs/rustdoc-types/0.61.0/rustdoc_types/struct.Id.html), current
`api/query.rs::TypePattern` and installed native HIR `Type` methods. Before adding source-link
navigation, verify actual producer coordinate conventions: the examined local API Location
comment and upstream Span documentation describe different column bases. This is an integration
question, not a producer defect established by this review.

Explain unknown answers as part of O08: missing macro inputs, inactive contexts, inference gaps
and unsupported predicates should lead to useful next actions through existing diagnostic owners.
`evaluate_where_clause_at` preserves Holds/NotProven/Invalid/Unsupported; textual
`get_failed_obligations` explanations must not become structured proof. Existing ErrorRecord and
TypedDiagnostic already carry rich facts, causes and labels. Reuse them where a search-local
diagnostic needs richer correction context instead of replacing them with Miette's JSON renderer.

For structural query authoring, O07 can expose pinned grammar `NODE_TYPES` and native query-error
locations rather than maintain a second Rust node taxonomy. Explicit ast-grep `stopBy` and
strictness examples can prevent wrong-scope questions. Public insertion helpers with `todo!()`
in ast-grep0.51.0 are not qualified editing mechanisms; syn/quote/prettyplease remain suitable
for generated fragments, not wholesale reprinting of maintained source with comments.

## Recommended sequence, acceptance and rule impacts

Start with O07 and selective O08 improvements to make delivered capabilities easy to use. Develop
O01's honest typed evidence, then O02/O05/O06; together they provide the most useful shared
foundation for the listed skills. O03, O04, O09 and O11 can grow through named consumer questions
using that foundation. O10 remains conditional. Priority expresses expected functional value,
not a measured benefit or mandatory serial package order.

Acceptance should include an unfamiliar subsystem investigation as well as the recent Request,
compiler selection, cancellation and API-qualification examples. Use a same-spelled unrelated
entity, callback value, changed cfg world, missing dependency artifact, historical blob and
misleading document mention to challenge each relevant join. A positive path plus an explicit
unexamined frontier is preferable to a false exhaustive answer. Proposed extensions require
actual consumer use before claiming delivery; existing source-level observations do not supply it.

Rule impacts are proposals for subsequent planning. Extending shared query/relationship/result
contracts may require the existing ADR/schema/publication route if their meaning changes;
targeted examples generally do not. Cross-document associations must preserve native status
ownership and document immutability. Automatic principle verdicts or mandatory audit instrumentation
would conflict with the binding's agent-judgment model and are not recommended. A compiler-private
component, persisted index, protocol cutover or new service would need an explicit selected
decision and its own lifecycle qualification; none is authorized here.

The review performs no broad production or test audit and assigns no new implementation
requirements to completed Plan59 packages. Its hypotheses can feed later plan creation without
reusing historical test success as acceptance of new behavior.

## Research coverage, alternatives and remaining questions

Two bounded library inquiries and one interface inquiry supplied independent source evidence;
the coordinator inspected current sources and ran the bounded public source journeys above.
A focused independent challenge of O01/O03/O10 sharpened read/write, capture and bulk-index
claims in this final account. This was part of functional assessment, not a second diagnostic
review or recommendation-audit publication.

All unique supplied overview content was read end to end, with initially truncated output
recovered through segmented reads:

| Research group | Complete overview coverage |
| --- | --- |
| Native semantics/compiler | ra_ap_hir, ra_ap_ide, rust-analyzer, rust-compiler-internals. The continued compiler document has identical SHA-256 `2ec763b394e8a76c83cba8488197e7836b2024c3aed5c1930625d54c8014de98` and contributes no additional content |
| Structure/API/dependency | ra_ap_syntax, ast-grep-core, tree-sitter-rust, syn_quote_prettyplease, rust_dependency_tooling, rustdoc-types, rustdoc |
| Interface/support | clap, figment, indicateif (Indicatif), prost, schemars, serde_serde-json, thiserror_and_miette, tonic |

Context7 supplied discovery and current upstream leads; exact installed HIR/IDE0.0.358,
syntax0.0.360, ast-grep0.51.0, Tree-sitter0.27.0 and rustdoc-types0.61.0 sources settled
consequential mechanism details. The compiler source check uses exact commit
`c1070d69382b8d2f2eb65119c738a77d9e324c9e`. Overview versions differ for syn, Tree-sitter,
Schemars, serde_json and Tonic; advertised current releases do not authorize pin changes or
establish availability in this checkout. In particular, Tonic0.14.8 cancellation additions
are not assumed present in pinned0.14.6. No additional dependency is selected.

Representative exact interface references for follow-up planning:

- [HIR0.0.358](https://docs.rs/ra_ap_hir/0.0.358/ra_ap_hir/): native relationships and typed operations, with compatible origin/substitution limits.
- [Tree-sitter Rust grammar metadata](https://docs.rs/tree-sitter-rust/0.24.2/tree_sitter_rust/): native node kinds/fields rather than a maintained duplicate taxonomy.
- [ast-grep relational scope](https://ast-grep.github.io/guide/rule-config/relational-rule.html): explicit `stopBy` controls, not an assumption that any ancestor satisfies the question.
- [Figment provenance](https://docs.rs/figment/0.10.19/figment/struct.Figment.html#method.find_metadata) and [Schemars settings](https://docs.rs/schemars/1.2.2/schemars/generate/struct.SchemaSettings.html): reuse delivered configuration and directional schemas.
- [Miette diagnostic conversion](https://docs.rs/miette/7.6.0/miette/trait.IntoDiagnostic.html): preserve existing diagnostic metadata rather than erase it during conversion.
- [Prost Message](https://docs.rs/prost/0.14.4/prost/trait.Message.html) and [Tonic request deadlines](https://docs.rs/tonic/0.14.6/tonic/struct.Request.html#method.set_timeout): transport/encoding mechanisms do not define application settlement or canonical identity.

O07 can also compare a passive catalog of shared query names/scopes with simply documenting
them. Selection and effective-request explanation already exist; a catalog earns code only
when discovering shared questions becomes difficult. O08's richer local query corrections
similarly require an actual consumer; the existing envelope diagnostics are already rich.

No new universal graph store, search DSL, briefing engine, alignment score, test runner,
automatic preparation or remote service is recommended. Existing source/semantic/API owners
plus precise skill examples are viable alternatives for every initial workflow. Optional
native bulk extraction can be justified even for one large question when it removes scalar
coordination; persistent indexing and multi-client transport have different additional obligations.
These alternatives are substantive no-adoption decisions, not claims that the corresponding
libraries lack capability.

The root probes established source discovery, exact inspection, mixed-content witnesses and
stale-reference refusal only. Other workflow examples are source-grounded interaction designs:
they did not execute a fresh native semantic comparison, API acquisition or compiler run in this
review. Prior P02 native/fixture evidence retains its recorded scope. Latency, token savings,
memory use, completeness of architectural diagnosis and new feature behavior remain unmeasured
or untested. Subsequent planning should select concrete consumer questions and qualify those
interfaces without turning this review's examples into a compulsory audit procedure.
