---
title: Rust-centered search and investigation through the agent CLI
status: proposed
date: 2026-10-31
doc_role: review
doc_owner: docs/plans/59-agent-cli.md
doc_topics: [design, libraries, environment, diagnostics]
---

# Rust-centered search and investigation through the agent CLI

## Judgment and scope

**Proposed:** make `pse` the preferred search and investigation interface for a whole
checkout, centered on Rust development. Compose fast file/text discovery, structural
queries, semantic relationships, dependency APIs and historical inspection through one
selection and result model. The objective is to remove caller-side discovery, pipelines,
coordinate conversion and repeated reading, while retaining the underlying libraries'
different strengths. A single front door need not mean one search engine.

The highest-value foundation is repository selection plus library-owned text search,
name-first symbol discovery and compact, navigable answers. Richer semantic composition
and source-bound change proposals should build on that foundation. Structured HIR queries
deserve a deliberate feasibility investigation; more compiler dumps alone will not make
ordinary discovery easier.

The maintainer selected **whole checkout, Rust-centered**: Rust, manifests, configuration,
documentation, tests, generated-source ownership and useful Git history. This is not a
proposal for a general document-extraction platform, universal shell replacement or
mandatory cross-repository search service. Further capabilities below are opportunities,
not automatically accepted scope.

Source baseline: `81c2759d8461bd680f7c6fd36aa3516b22bf4bb6`, inspected 2026-10-31.
The user-supplied `docs/libraries/ripgrep.md` and
`docs/libraries/search_tooling_other_than_rg.md` were untracked inputs and remain untouched.
The review applies core 3.4, efficient-architecture heuristics 1.0 and the pse-arrow binding
selected by `standard.toml`. The simulator profile's physical/numerical requirements do
not govern this tooling-only proposal. This is a functional review, not a broad diagnostic
audit, completion audit or amendment of Plan59. Current behavior and retained guarantees
are described by blueprint §23.2 and the native operating guide; changes to those contracts
remain proposals requiring their normal decision route.

## Current capability and evidence

`pse-host` already contains server-free source reads, AST matching, immutable proposals,
owned analyzer sessions, API acquisition, compiler investigations and Git-aware source
acquisition. These are substantial working components, rather than backend placeholders.

| Existing owner | Current capability and limitation |
| --- | --- |
| `source_read`, `source_command` | Exact working/index/revision reads, nested outlines, selected declarations, source identities and guarded outline continuation. Source parsing is UTF-8; textual search needs its own byte/encoding policy. |
| `source_query`, `cli/code` | Multiple exact file operands, ast-grep patterns, captures, strictness and ancestor/descendant/exclusion constraints. No repository selector or structural-query continuation; `--mode semantic` explicitly refuses. |
| `source_proposal`, semantic edit integration | Checked syntax and native semantic previews, including rename, assists, SSR and file operations. No source-application command. |
| `semantic`, `cli/code/semantic` | Position-based explanations, symbols, references, calls, implementations, macros, hints, completion, predicates and test candidates. Working-source sessions require selected Cargo context; no exposed name-first workspace-symbol entry point. |
| `api`, dependency/compiler owners | Locked API search, signatures, declarations, examples, structural comparison, selected units, original diagnostics and explicit compiler artifacts/probes. These answer different questions from source-name discovery. |
| `envelope` and its presentation module | Typed schemas, mandatory identity/status/coverage, top-level payload projection and record allowances. Normal JSON often repeats source/path detail. Human output also exposes structured detail rather than a purpose-built search listing. |

Source inspection shows that structural execution acquires every selected file, analyzes
once to count matches and again to collect deterministic quotas. Its allowance limits
returned records, not examined inputs or parsing work. This is reasonable for exact totals
but should not be inherited by existence or first-results requests. `emit_page` already
supports producer-selected answers without retaining a reconstructed full response;
legacy `encode_frame` limited views retain the full response. Extend the former pattern
where appropriate rather than making every search a durable full-result artifact.

**Tested in this review:** four read-only public invocations on the local Linux checkout,
with the prepared host and pinned toolchain. `pse outline ...environment.rs --budget 5`
returned five of 66 declarations with a continuation, exit0; a two-file syntax query for
`prepend($$$ARGS)` with budget3 returned three of six matches, exit0. The same query with
semantic mode refused, exit125; session status reported unprepared, exit0. No analyzer,
compiler investigation or provider installation was started. Earlier Plan59 semantic,
API and compiler test evidence retains its original conditions; it was not rerun here.

**Measured, single local observations:** compact JSON stdout/stderr lengths were 4,611
bytes for the five-entry outline, 12,909 for the three-match structural query, 1,470 for
semantic refusal and 730 for unprepared status. Python `subprocess.run` wall observations
were approximately 0.174, 0.189, 0.171 and 0.172 seconds respectively. These establish
neither a latency distribution nor a speed comparison with ripgrep. The result sizes
provide a concrete reason to improve default representations.

## Recommended interaction structure

Select three user intents as the target, with examples below **illustrative new syntax**:

| Intent | Proposed entry | Responsibility |
| --- | --- | --- |
| Find candidates, answers or relationships | `pse search` | File, text, syntax, symbol, API, semantic and historical queries using a common source selection and answer model. |
| Read or explain selected or known content | `pse inspect` | Accept a result reference or an explicit source/position/range/declaration locator; return exact context, body/docs and selected related evidence. |
| Construct an inspectable change | `pse propose RESULT` | Syntax replacement or native semantic transformation into the existing immutable proposal owner. |

Retain explicit effectful preparation and execution through their existing owners:
semantic sessions, API acquisition, compiler generation/probes, tests and managed runs.
They are operations with different effects, not extra search modes to execute implicitly.
Ordinary source search must work without analyzer preparation. An explicitly requested
semantic stage can report preparation required; it must not silently become text search.

This is preferable to adding a separate verb for every backend feature or putting every
parameter into one flat command. Clap subcommands and argument groups can organize simple
query modes under `search`; a typed request handles combinations too awkward for flags.
Keeping today's families with shared flags would improve consistency but retain the
discovery burden. A universal free-form query language would add grammar and debugging
work before proving value. Three intents provide a small vocabulary without hiding
meaningful capabilities behind backend-neutral strings.

Migration should move each useful consumer and remove its replaced public route, parser
branch, guidance and tests together. No compatibility alias is presumed necessary in this
development phase. Preparation/control routes remain because their effects remain useful.

Known-file reading must not require a preliminary search: `inspect` accepts explicit path and
line/byte ranges or positions, selected declaration locators, index/revision sources and result
references. Typed locator forms prevent guessing whether a name is a file, invocation or symbol.
Read-only dependency identities, feature/inclusion explanations and selected unit facts belong
to the common search/inspect flow through their existing owners. Generated ownership is source
inspection enrichment. Original compiler diagnostics, retained logs/artifacts and retained test
populations are explicit evidence locators for inspection. Affected-test/command-intent explanation
can compose with search results without executing tests. Test listing that actually needs compilation
or fixture setup retains the explicit `tests` workflow. Discovery/schema/help and execution/control
commands remain adjacent interfaces. No passive route is deleted until its useful behavior has a
clear public replacement; reducing names is not permission to lose a distinct investigation.

For example, the following is a proposed interaction vocabulary, not existing syntax:

```text
pse search text 'PreToolUse' --package pse-host --context 2 --limit 8
pse search symbol EnvironmentSettings --workspace --kind struct
pse search syntax 'tokio::spawn($F)' --changed --capture F
pse search api spawn --dependency tokio --async-only
pse search --request storage-cancellation.json
pse inspect RESULT --view declaration
pse inspect --source crates/pse-host/src/environment.rs --lines 10:40
pse propose RESULT --rename NEW_NAME
```

A grouped request could express file-level conjunction without shell filename joins:

```json
{
  "sources": {"kind": "working", "packages": ["pse-host"]},
  "select": {"within": "file", "all": [
    {"text": {"literal": "PreToolUse"}},
    {"syntax": {"pattern": "Command::new($BIN)"}}
  ]},
  "answer": {"kind": "files", "limit": 10}
}
```

This selects files containing both witnesses; it does not claim that the text and call
belong to one syntax node. Selected answers can request their witness spans or enclosing
declarations rather than rerunning the query with a different interpretation.

## Domain model and query composition

Define query meaning in ordinary Rust types before deriving CLI, configuration and schemas.
The important concepts are source universe, selection, predicate, relation, requested answer
and explicit effects. This is not a proposal for a universal ontology or separate query service.

* A source selection identifies working, index or resolved revision content; package/module,
  path/type/glob and changed-source restrictions narrow that universe. Dependency source,
  ignored files and generated outputs are explicit inclusions. Filesystem walking and Git
  tree/index enumeration are different acquisition mechanisms behind that meaning.
* Predicates distinguish byte/text conditions, syntax patterns and resolved semantic facts.
  Textual equality of names does not establish one symbol. Syntax visibility or cfg spelling
  does not establish active compiler membership.
* Boolean groups have an explicit unit: `all` at a node means the same candidate satisfies
  every predicate; `within-file/all` means the file contains witnesses for each predicate.
  Those witnesses may be different nodes. Semantic conjunction binds the same resolved
  entity in the same context. Negation over incompletely examined input is unknown, not true.
* Relationships retain direction and multiplicity: enclosing declaration, contains, calls,
  called-by, implements, references, generated-from and changed-between are not generic edges.
  Bounded expansion is selected, with cycles/duplicates handled for the requested question;
  do not enumerate every possible path by default.
* The requested answer is matches, files, existence, count, grouped summary, definitions or
  selected relationships. Exact count, first useful results and ranked candidates have different
  completion/ordering obligations. A result limit is not a claim of exhaustive examination.

Examples worth supporting include: files mentioning both an endpoint and a timeout; public
functions with an async modifier and a selected return-type shape; calls matching a syntax
pattern whose target resolves to one definition; implementations in selected packages; and
references to a symbol whose declaration changed between revisions. Mixed-stage requests
must expose syntax-only, semantically confirmed and unavailable portions independently.

Use Clap for concise requests and a typed JSON request for grouping, nesting and batches.
Compile both into the same values and validation functions. Do not implement separate flag,
JSON and saved-query meanings. Prefer native ast-grep rule composition/capture constraints
to an application-built structural interpreter. Keep native rule subtrees typed or validated
at their provider boundary. General SQL, shell evaluation and an open-ended expression VM
are not selected for initial composition.

## Opportunities

<a id="O01"></a>
### O01 — Repository selection and useful file discovery

**Recommend first.** Give `search` filesystem, Git and Cargo-aware selection with filenames,
tree summaries and metadata predicates where useful. This removes `rg --files`, `find`,
`git ls-files` and shell argument construction from ordinary Rust investigation.
Use `ignore::WalkBuilder`/parallel traversal and existing `globset`, `gix` and Cargo metadata
owners. Defaults respect ignore rules and avoid dependency/build output; explicit requests
can widen them. Preserve tracked deletion, untracked working content, index blobs and revision
blobs as distinct facts. Never traverse `target/` merely because it contains Rust files.
Accept with a dirty checkout containing ignored, staged, deleted and generated files, where
the selected universe and omission reasons can be inspected. Traversal does not require
registering every file in a new persistent store.

Broad lexical selection should not force every scanned file through the current whole-file
UTF-8 source owner. Stream bytes and report their searched representation. Hydrate a selected
declaration or guarded edit into the existing exact source identity when needed. A lightweight
lexical observation must not claim a complete content snapshot; subsequent inspection is either
verified against retained premises or explicitly a fresh observation. Whole-file hashing,
transcoding and exact replay have real costs and should follow the requested guarantee.

<a id="O02"></a>
### O02 — Complete fast text and exact-content inspection

**Recommend first.** Support literals, regular expressions, multiple patterns, smart/explicit
case, word/line matching, multiline, inversion, context, file-only/existence/count answers
and explicit encoding/binary behavior. Adjacent TOML, Markdown, YAML and JSON participate.
Provide exact lines/ranges/head/tail and bounded context through `inspect`, including files
that are not valid Rust syntax. Search replacement rendering is not source application.

Prefer embedded `grep-regex`, `grep-searcher` and `grep-matcher` with `ignore`, using native
`Searcher`/`Sink` callbacks over files, readers or Git blob slices. This fits shared selection,
structured extraction and cancellation without a JSON subprocess round trip. A managed
versioned `rg --json` subprocess is a credible alternative for the complete mature pipeline,
especially optional PCRE2 behavior. Compare exact features and total glue before selecting
pins; do not reconstruct PCRE2 semantics with the default regex engine. Native no-match exit1
must map to a successful empty search rather than a false tooling failure. Keep partial I/O
failures separate from no matches. Compression/preprocessing are explicit optional extensions,
not prerequisites for ordinary text. Do not write a replacement regex engine or ignore walker.

<a id="O03"></a>
### O03 — Name-first discovery and contextual API search

**Recommend first.** Find an entity from a simple/qualified name, kind, package, module,
visibility or signature, then inspect it or expand references/calls. Use source outlines for
written declarations without a session; use analyzer workspace-symbol discovery for semantic
candidates and acquired rustdoc graphs for dependency APIs. Expose which universe was searched.
Search source symbols and documented APIs in one interaction without pretending that their
identities or completeness are interchangeable. Fuzzy ranking must expose ambiguity and exact
matches, not silently choose one same-spelled method. Test a name shared by unrelated traits,
an inactive cfg declaration, a reexport and a dependency item.

<a id="O04"></a>
### O04 — Rich structural predicates and reusable query recipes

**Recommend with the common query model.** Expose native pattern alternatives, negation,
node-kind/field and capture constraints, descendant/ancestor/sibling relations, enclosing
declaration filters, and capture extraction. Add file-level combinations and useful grouping
without converting native matches into text and reparsing them. Select strictness deliberately:
signature-style matching and relaxed matching are different searches, especially before a rewrite.
Saved typed requests should support repeatable investigations; no new handwritten general DSL
is justified merely to express existing library rules. Demonstrate same-node versus same-file
conjunction, repeated metavariable equality, nested matches and failed alternatives that do not
leak captures. Distinguish `ast-grep-core` from capabilities supplied by its config/rule family.

Offer native Tree-sitter `Query`/`QueryCursor` as an advanced extraction option for field,
negated-field, capture, quantifier, anchor and text-predicate queries when code-shaped patterns
are insufficient. Reuse the current grammar/runtime and `NODE_TYPES` for discoverability.
Match-limit, recovery and timeout outcomes remain visible. This is a native query capability,
not a second application-written syntax language.

Concrete integration lead: `SerializableRuleCore::get_matcher(DeserializeEnv)` and `RuleCore`
in the tagged 0.51 config family offer a smaller boundary than adopting lint/fix `RuleConfig`.
`CombinedScan` can group rules by candidate node kind, but its examined implementation honors
suppression comments and collects matches before returning. It is not an unsuppressed streaming
ad hoc search engine; select it only for compatible lint-like batches. Prefer native rule
evaluation over shared parsed trees for ordinary search, with exact version alignment verified.

<a id="O05"></a>
### O05 — Compact answers, addressed results and progressive expansion

**Recommend first; shared dependency.** Default search records should contain a readable path,
location, qualified name/kind when known, short context and a result reference. Put source
identity/provider context once per referenced source/context, not repeatedly inside every capture.
Represent normal UTF-8 paths as strings while preserving exact fallback bytes for unusual names.
Use `inspect` for bodies, documentation, captures, original records and provenance detail.

Preserve mandatory identity, outcome and coverage, but redesign their physical representation
where repetition adds no meaning. Offer independent controls for result count, per-result context,
fields, grouping and output allowance; do not truncate JSON bytes or split indivisible records.
Continuation needs query/source premises, stable ordering and an explicit change policy. Avoid
persisting complete results just to display a compact page. Small source-bound references or
an explicitly retained result set can serve their actual lifetime; do not invent a universal
result database. Streaming NDJSON must say whether results are final, partial or pending reorder.
Accept by completing an investigation from a small first page through two useful expansions.

<a id="O06"></a>
### O06 — Demand-driven search and composed execution

**Recommend alongside O01–O05.** Compile selectors/matchers once, traverse the smallest relevant
universe and apply cheap filters before parser/semantic work where the predicates permit.
Use library short-circuiting for existence and first-results queries; exact totals can scan
the complete universe. Share parsing/native preparation across compatible batched questions
without automatically introducing a long-lived service. Reuse the existing Rayon/Tokio ownership
and cancellation paths. Directory cardinality, source bytes, total matches, semantic expansion
and serialized answer size are different work dimensions. Deterministic global sorting or exact
counts may require a full pass; expose the selected tradeoff instead of claiming that a small
page guarantees low cost. Indexed search becomes a candidate only when measured repeated-work
benefit exceeds invalidation and operation burden.

`grep_searcher::Sink::matched` can request successful early termination; `ignore::WalkState::Quit`
can stop new visits while admitted workers settle. A huge no-match file does not call `matched`,
so prompt cancellation also needs a cancellation-aware reader or equivalent checkpoints. File-level
negative predicates need EOF before a result becomes final. Counts should distinguish exact,
at-least and unknown rather than inventing omitted totals for first-N results. Native matcher
`build_many` is OR, not conjunction. Current envelope NDJSON encodes collected response framing;
genuine per-hit search streaming needs its own typed record writer and terminal coverage summary.
Do not build a full `Value`/result vector solely to serialize a stream. Buffer complete records,
settle work on a closed consumer, and keep independently requested exhaustive retention explicit.

<a id="O07"></a>
### O07 — Semantic refinement and typed Rust relationships

**High value; bounded feasibility work before backend commitment.** Resolve candidate targets,
types, implementations and references in selected Cargo context; support meaningful filters
such as callable signature, trait implementation, visibility, async/unsafe/const properties,
reference roles and expansion-aware origin where the provider actually supplies them.
Current position-based LSP queries are useful consumers, but hover/debug text must not become
a fabricated typed model. Investigate native `ra_ap_ide`/`ra_ap_hir` for structured operations
not faithfully exposed by the installed protocol. Compare a single embedded analysis owner
with extending the existing owned analyzer interface. Do not keep two independently maintained
live semantic databases for one project merely to expose a few extra fields.

The feasibility case must answer actual questions—resolved method target versus same spelling,
inferred versus adjusted type, applicable generic trait constraint, or reads versus writes—and
account for Cargo/VFS/macro loading, source changes and upgrade cost. Exact compiler-unit identity
is still a distinct obligation. Native HIR availability alone does not establish whole-workspace
completeness or rustc equivalence. Choose a provider per supported capability/context and report
not-proven, unsupported and incomplete distinctly. Existing LSP functionality is not frozen as
the target ceiling, and no native embedding is approved by this review.

Concrete HIR leads include `Semantics::type_of_expr`/`type_of_pat` and `TypeInfo` original versus
adjusted types, method/field resolution, `could_coerce_to`, generic-owner-aware substitutions,
and implementation/visibility/import maps. `resolve_bin_expr`, `resolve_index_expr`,
`resolve_try_expr` and `resolve_await_to_poll` make implicit trait-mediated dependencies searchable,
including operations missed by a call-expression-only search. Native IDE `find_all_refs` with
`SearchScope` and `ReferenceCategory` can expose scoped read/write uses. Type display strings
do not establish semantic equality; shallow implementation enumeration may miss blanket impls.
Expansion mapping may yield several origins or only an invocation fallback, which must remain
different from a precisely editable authored range.

Compare complete mechanisms: LSP extensions reuse current loading/watchers for workspace symbols
and existing requests; an embedded `AnalysisHost` plus HIR supports richer typed operations but
needs Cargo/VFS/macro world-state integration; a narrowly extended rust-analyzer server may reuse
upstream orchestration more effectively than recreating it. `ra_ap_load_cargo` batch loading alone
is not a live update service. `Semantics` is not a cross-thread shared database handle; extract
plain results within the owning worker and use native analysis snapshots/change application.
Select compatible RA family releases together; overview HIR/IDE 0.0.358 and current syntax
0.0.360 cannot be mixed by assuming their AST types agree.

Native SSR search deserves a specific feasibility check. `MatchFinder::in_context`,
`add_search_pattern(SsrPattern)` and `matches()` exist, while the examined upstream `Match`
range, node and placeholder fields are crate-private; public `matched_text()` alone is not
enough for source-addressed capture results. Start with public ast-grep candidates plus HIR
enrichment, or assess selected-release accessors/a small upstream or source-integrated extension.
Do not derive genuine search matches from identity replacements or parse edit output. Preserve
ordered rewrite precedence separately from Boolean alternatives. Exact-pin access remains
unverified, rather than declared impossible.

<a id="O08"></a>
### O08 — Historical and change-centered investigation

**Recommend after source/result foundations.** Extend existing staged/revision reads into
history questions: which change introduced a string or structural change, previous declaration
content, selected blame ranges and changed-symbol neighborhoods. Use native Git/gix capabilities;
check exact support for pickaxe and line-history before choosing an adapter. A managed Git
operation is preferable to reimplementing missing history semantics. Connect changed symbols to
candidate tests and original compiler diagnostics, retaining candidate-versus-executed distinctions.
Historical text does not acquire today's semantic meaning automatically. Rename heuristics and
symbol correspondence across revisions carry their uncertainty explicitly.

<a id="O09"></a>
### O09 — Structured adjacent-content queries

**Recommend a useful subset.** Inspect JSON via typed values/JSON Pointer and TOML/YAML through
their existing parsers, with field selection and typed predicates. This replaces repetitive
`jq`/`awk` pipelines for manifest/settings questions without promising every jq program or
SQL operation. Prefer existing domain owners for dependency features, source ownership and
test inventory over scraping their configuration representation. A structured value match,
raw source span and rendered configuration value must remain distinguishable. Add another
format only for a credible repository workflow. PDF/OCR, arbitrary archives and binary analysis
are not initial whole-checkout search requirements; existing artifact/log readers remain useful.

<a id="O10"></a>
### O10 — Contextual explanations, diagnostics and API construction

**Extend existing strengths.** From a source or diagnostic result, inspect inferred annotations,
candidate APIs, imports, constraints and examples without asking the agent to rebuild coordinates.
Keep lazy hint/completion/assist resolution. Signature/type/trait filters and doc links should
help find a usable operation rather than only a matching name. Evaluate bounded native term
synthesis for concrete expected-type questions; it is not an automatic source-writing engine.
Join original compiler diagnostics to explicitly current semantics without relabelling current
source as original evidence. Unsupported typed fields remain unavailable. This opportunity
primarily composes existing consumers with O03/O05/O07 rather than creating another briefing verb.

Native `hir::term_search`, `TermSearchConfig` or completion term-search/fuel configuration can
support a deliberate expected-type construction mode; current session configuration does not
explicitly enable it. Report non-exhaustiveness and fuel exhaustion, and preview imports/effects.
Native diagnostics can be a selected query overlay, separate from original rustc/Clippy results.
Current API input/output filters use spelling containment; structural `rustdoc_types::Type`,
generic bounds and reexport graphs can support richer capability queries joined to actual uses
and destination-module importability. Source-qualified package and target/feature context must
survive that join. Hosted rustdoc JSON is an explicit alternative acquisition with its own build
premises, not a substitute for exact locked local context. Guppy is a candidate for repeated
feature-set/counterfactual graph questions; Hakari adds no needed search capability and is not
selected. Existing metadata/shortest-path owners remain sufficient for their current questions.

<a id="O11"></a>
### O11 — Search-to-change proposals with appropriate syntax tools

**Recommend selected workflows.** Carry selected result references directly into existing
checked proposals: exact-byte syntax replacements, semantic rename/assists/SSR and file operations.
Use `ra_ap_syntax` edit facilities for lossless authored regions, ast-grep replacements for
capture-preserving patterns, and `syn`/`quote`/`prettyplease` for newly constructed Rust fragments
where token/AST generation is appropriate. Whole-file regeneration loses trivia and is not the
default refactoring strategy. Show selected overlaps, source changes, snippets and unsupported
effects explicitly. Keep apply as a separately assessed capability; current previews can remain
valuable without it. Acceptance should include a comment-preserving multi-file preview and a
stale selected-result refusal, followed by ordinary authorized edits and actual checks.

<a id="O12"></a>
### O12 — Purposeful compiler explanations

**Selective extension, not an ordinary-search dependency.** Compiler dataflow, borrow checking,
MIR phase information and structured layouts can answer why a borrow/drop remains, why an async
state contains a value, or which monomorphized instance was generated. Existing dumps/probes
already cover several investigations; identify a concrete structured question they cannot
answer before adding a pinned compiler-private driver. Preserve original context and native
results; dataflow facts are not a general proof of program safety, performance or runtime reach.
Native rust-analyzer MIR/evaluation and rustc MIR are different models. Compare their precise
question fit rather than flattening them into one generic compiler record.

For a credible ownership/async workflow, evaluate
`rustc_borrowck::consumers::get_bodies_with_borrowck_facts`, `ConsumerOptions`, and native dataflow
`iterate_to_fixpoint` with `ResultsCursor`/`ResultsVisitor`. Value liveness, storage liveness,
initialization and active loans are different facts. Retain MIR phase, body and location;
borrow facts cannot be reconstructed from source spans or a different Polonius mode. Driver
query timing, rustc-dev availability and exact pinned compiler APIs are unresolved integration
work, not a reason to exclude a useful new question until historical recurrence exists.

<a id="O13"></a>
### O13 — Discoverable typed configuration, schemas and query diagnostics

**Recommend with the query model.** Derive typed parsing, request schema and result views from
their owners. Use Clap argument groups/value parsers and `ArgMatches::value_source` so implicit
defaults do not accidentally override a selected request or configuration. Schemars input and
output contracts have different direction; Serde representation customization must match actual
runtime construction. Schema presence does not enforce semantic preconditions or source identity.
Use `serde_json::RawValue` for opaque original provider records where faithful retention is needed,
not as an excuse for untyped application decisions. Provider-native syntax errors should preserve
their location and a useful correction without introducing an automatic retry loop.

Figment earns adoption for reusable scopes/named query profiles once multiple configuration
sources actually participate: `Serialized<T>`, `merge`/`join`, typed extraction, `Tagged<T>` and
`RelativePathBuf` supply composition/provenance. Define precedence once and keep the native host
environment owner. Do not add a second shell/environment evaluator or silently merge queries
whose arrays/predicates have different meanings. A simple flags/request interface does not need
Figment merely because it was documented.

<a id="O14"></a>
### O14 — Useful progress and operational explanation

**Recommend proportionately.** Show current stage, examined files/bytes, useful hits and
preparation state when a search is long enough to warrant feedback. Use indicatif
`ProgressBar`/`MultiProgress`, draw targets and native iterator/reader integrations for terminal
rendering on stderr; machine results remain clean stdout. Unknown total or incomplete semantic
work must not gain a fake percentage or ETA. Progress completion is not calculation success,
source consistency or native resource settlement. Use the current ownership/cancellation runtime;
progress handles do not replace it. Expose effective selection/provider choice and why a stage
requires preparation through shared query explanation, not a second diagnostics framework.

## Integration, priority and acceptance

First deliver O01/O02/O03/O05/O13 as one useful discovery-to-inspection slice, with O06's
demand-driven execution. This is the strongest route to replacing everyday manual search.
Then add O04/O08/O09/O10 and selected O11 compositions. O07 deserves an early capability
prototype in parallel because its provider choice constrains deeper semantic composition;
it is not a prerequisite for useful text/syntax discovery. O12 remains tied to unanswered
compiler questions; O14 ships with stages that actually need progress.

| Journey | Existing foundation | Proposed acceptance |
| --- | --- | --- |
| Unfamiliar subsystem | Exact-file outlines and declarations | Select a package/tree, find a name, inspect its declaration and follow one relevant reference without shell filename discovery. |
| Text across source and configuration | Source reads and Git acquisition | Literal/regex queries over Rust/TOML/Markdown respect scope, context, exclusions and explicit no-match semantics. |
| Combined structural/semantic question | Native matching and position-based analyzer | Prove file/node conjunction differences, then refine selected candidates by actual identity; unavailable refinement stays visible. |
| Trait-method change | References, calls, proposals, candidate tests | Navigate exact entity→impl/callers→selected preview→real test selection, without treating candidates as coverage. |
| Compiler error | Original diagnostics, APIs, probes | Read original failure, inspect current semantics separately, find an API constraint and run only an explicitly selected experiment. |
| Generated/historical hit | Origin routes and revision reads | Find the authored owner and a relevant historical change without editing generated files or giving old code current semantics. |
| Bounded continuation | Source-bound outline cursors | Continue useful search pages with source/query guards; document working-tree changes and partial examination. |
| Adverse input | Recovery parsing and native diagnostics | Handle unusual filenames, incomplete syntax, unreadable content, ignored input, ambiguity and inactive cfg without silently changing the question. |

A capability-based displacement claim requires these journeys through public `pse`, with
native or independent expectations appropriate to each mode. Unit tests and help snapshots
alone do not establish usable search. Compare latency, preparation and result bytes on a small
representative workload if making quantitative benefit claims. Do not make a full benchmark
campaign or a new database prerequisite for this developer workflow.

## Decisions and boundaries for subsequent planning

The proposal changes current search/output contracts and command grouping. If selected,
update the owning architecture/decision routes for mandatory envelope shape, provenance
placement and command replacement; preserve blueprint §23.2 meaning while reconsidering
physical repetition. Derive schemas and reference documentation through existing generators.
Move the `pse-cli` skill, relevant role/agent guidance and advisory route suggestions with each
delivered public slice. Reminders should recommend useful capabilities; they should not blanket
warn on every `rg`/file read or enforce adoption before the selected capability exists.

No new crate, dependency, daemon, database, automatic preparation, global Codex configuration,
source application or production repair is authorized by this review. A single-process query
should remain possible; native analyzer ownership is reused only for work that needs semantics.
Persistent SCIP/LSIF or text indexes, cross-repository services and embedding search have
credible later uses, but require a concrete repeated-work or discovery benefit and freshness
design. They are not the selected first realization. Retain external tools as deliberate
escape hatches until actual public journeys establish displacement; the goal is to make the
CLI the most useful route, not to remove working capabilities by instruction.

For an explicit offline/historical semantic workflow, assess native `ra_ap_ide::StaticIndex`
or `rust-analyzer scip` before building a custom index. Occurrences, definitions/body ranges,
monikers and documentation can support repeated bounded relationship navigation. The index
is a source/configuration-bound snapshot, not an automatically maintained database or a
rename-proof identity authority. Keep export preparation explicit and compare it with querying
the existing warm semantic owner; live and snapshot results must not silently share freshness.

## Reading coverage and concrete source leads

Two bounded library-research tracks read all nineteen supplied overviews end to end, with
overlapping rereads after truncated output. The twelve analyzer documents contain eleven
unique texts: the two compiler-internals files have identical SHA-256
`2ec763b394e8a76c83cba8488197e7836b2024c3aed5c1930625d54c8014de98`.
One physical reading accounts for both. Coverage is reading, not independent verification of
every overview assertion. Context7 was the first external documentation route, followed by
primary API/source checks for consequential mechanisms. No library or license was excluded.

| Read inputs | Capability carried into the recommendations |
| --- | --- |
| [ripgrep](../libraries/ripgrep.md) §§2–8; [other search tools](../libraries/search_tooling_other_than_rg.md) §§2–14 | Ignore-aware discovery, bytes/encoding, matching engines, aggregation, Git history, structured content and composition; O01/O02/O06/O08/O09. |
| [ast-grep-core](../libraries/rust_analyzers/ast-grep-core.md) §§3–8; [tree-sitter-rust](../libraries/rust_analyzers/tree-sitter-rust.md) §§3–7 | Native matching, rules/captures, query traversal, rewriting, recovery and incremental syntax; O04/O06/O11. |
| [ra_ap_syntax](../libraries/rust_analyzers/ra_ap_syntax.md) §§2–7; [syn/quote/prettyplease](../libraries/rust_analyzers/syn_quote_prettyplease.md) §§2–8 | Lossless syntax/editing versus generated fragments, edition/recovery and source fidelity; O05/O11. |
| [ra_ap_ide](../libraries/rust_analyzers/ra_ap_ide.md); [ra_ap_hir](../libraries/rust_analyzers/ra_ap_hir.md); [rust-analyzer](../libraries/rust_analyzers/rust-analyzer.md) | Native semantic queries, incremental worlds, search scopes/categories, implicit resolution, synthesis and indexes; O03/O07/O10 and conditional snapshots. |
| [rustdoc-types](../libraries/rust_analyzers/rustdoc-types.md) §§2–11; [rustdoc](../libraries/rust_analyzers/rustdoc.md) §§3–9; [dependency tooling](../libraries/rust_analyzers/rust_dependency_tooling.md) | Structural API/types/bounds, reexports, examples, exact metadata and feature graph alternatives; O03/O08/O10. |
| [compiler internals](../libraries/rust_analyzers/rust-compiler-internals.md) and [continued input](../libraries/rust_analyzers/rust-compiler-internals-continued.md) §§2–6 | Phase-aware dataflow, loans, regions, ownership and driver limits; O12. |
| [Clap](../libraries/clap.md) §§2–11; [Figment](../libraries/figment.md) §§2–11 | Typed command hierarchy, parse provenance, discovery/testing, profiles, relative paths and configuration sources; O13. |
| [Serde/serde_json](../libraries/serde_serde-json.md) §§2–10; [Schemars](../libraries/schemars.md) §§2–12 | Shared typed requests/events, borrowing/opaque records, directional schemas, streaming and semantic validation boundaries; O05/O06/O13. |
| [indicatif](../libraries/indicateif.md) §§2–11 | Terminal drawing, counters, concurrent layout, iterator/I/O integration and hidden-target limitations; O14. |

Primary integration pointers, retained for subsequent planning:

* [Searcher and Sink](https://docs.rs/grep-searcher/0.1.16/grep_searcher/trait.Sink.html),
  [ignore WalkState](https://docs.rs/ignore/0.4.33/ignore/enum.WalkState.html), and
  [ripgrep JSON printer](https://docs.rs/grep-printer/latest/grep_printer/struct.JSON.html):
  callbacks, early stop, byte representation and subprocess framing. An optional rg adapter
  should isolate ambient rg config, drain both streams and distinguish exits 0/1/2; it should
  not coexist as a second ordinary lexical path without a distinct benefit.
* [Tagged ast-grep RuleCore source](https://github.com/ast-grep/ast-grep/blob/0.51.0/crates/config/src/rule_core.rs),
  [CombinedScan](https://github.com/ast-grep/ast-grep/blob/0.51.0/crates/config/src/combined.rs),
  [core matcher operations](https://github.com/ast-grep/ast-grep/blob/0.51.0/crates/core/src/ops.rs),
  and [Tree-sitter queries](https://tree-sitter.github.io/tree-sitter/using-parsers/queries/1-syntax.html):
  preserve native binding, field, relation and suppression semantics.
* [Workspace-symbol extension](https://github.com/rust-lang/rust-analyzer/blob/master/docs/book/src/contributing/lsp-extensions.md),
  [HIR Semantics](https://rust-lang.github.io/rust-analyzer/hir/struct.Semantics.html),
  [SearchScope](https://rust-lang.github.io/rust-analyzer/ide/struct.SearchScope.html),
  [SSR Match](https://rust-lang.github.io/rust-analyzer/ide_ssr/struct.Match.html), and
  [StaticIndex](https://rust-lang.github.io/rust-analyzer/ide/struct.StaticIndex.html):
  concrete discovery, typed refinement, public-access limits and snapshot alternatives.
* [Rustdoc search semantics](https://doc.rust-lang.org/stable/rustdoc/read-documentation/search.html),
  [pinned rustdoc-types](https://docs.rs/rustdoc-types/0.61.0/rustdoc_types/), and
  [docs.rs JSON](https://docs.rs/about/rustdoc-json): approximate signature search differs from
  structural type relationships and context-qualified artifact acquisition.
* [Rustc dataflow](https://rustc-dev-guide.rust-lang.org/mir/dataflow.html),
  [borrowck consumers](https://doc.rust-lang.org/nightly/nightly-rustc/rustc_borrowck/consumers/index.html),
  and [external drivers](https://rustc-dev-guide.rust-lang.org/rustc-driver/external-rustc-drivers.html):
  capability evidence and exact-phase extraction requirements, not proof of compatibility
  with this checkout's older pinned nightly.
* [Clap value sources](https://docs.rs/clap/latest/clap/struct.ArgMatches.html),
  [Figment provenance/composition](https://docs.rs/figment/0.10.19/figment/struct.Figment.html),
  [Schemars 1.2.2 settings](https://docs.rs/schemars/1.2.2/schemars/generate/struct.SchemaSettings.html),
  [JSON streams](https://docs.rs/serde_json/latest/serde_json/de/struct.StreamDeserializer.html),
  and [indicatif draw targets](https://docs.rs/indicatif/0.18.6/indicatif/struct.ProgressDrawTarget.html):
  native composition and views rather than custom parser/config/progress frameworks.

Some versioned HIR/SSR, Clap, Figment and ast-grep rustdoc requests were unavailable. Current
upstream/tagged sources and older installed HIR corroborate capability families, but do not
establish compatibility with a future selected native RA family. Overview Schemars/serde_json,
syn and Tree-sitter versions also differ from checkout pins. No dependency update follows from
an overview's version. Exact RA family selection, public SSR range/capture access, compiler-driver
feasibility and representative whole-checkout performance remain explicit investigation items.

Figment follow-up must settle array/profile semantics: `merge` replaces lists, while concatenation
can change ordered glob or predicate meaning; a Global profile can defeat an incorrectly merged
CLI override. Configuration provenance records winning sources, not a complete edit history.
Indicatif hidden targets can suppress `println` output, so durable diagnostics use their actual
diagnostic owner. These are integration conditions, not reasons to reject the libraries.

## Review publication outcome

**Proposed:** fourteen stable opportunities, one interaction/domain model, native integration
alternatives, staged priorities and public consumer acceptance scenarios. Independent research
changed the native SSR and CombinedScan recommendations where primary sources exposed limitations.
The initial broad temptation to reuse every exact UTF-8 snapshot/total-count path for text search
was rejected because it would preserve unnecessary work for file discovery and first-witness queries.

**Tested:** the four read-only CLI observations above establish only their named current behavior.
The review changes documentation only; no production tests, builds, provider preparation, library
installation or benchmark campaign qualifies any proposed search feature. Publication/link and
turn-end checks are reported separately from functional evidence. Plan59 dispositions, existing
architecture, agent configuration and user-supplied library inputs are unchanged.

**Tested — publication only:** `just turn-end`, `just docs` and
`pse docs check docs/functional_reviews/rust-centered-search-and-investigation-cli-2026-10-31.md
docs/functional_reviews/README.md` passed against baseline zero. The local book published 954
chapters, including the two user-supplied untracked inputs. The first native document check
refused a stale xtask source association (exit125); explicit `pse tools prepare xtask` selected
the current native helper and the scoped check passed with complete capture/settlement. That
preparation and documentation build establish no proposed search functionality. Independent
read-only synthesis review identified the direct-locator and passive-consumer hierarchy gaps;
both were incorporated before publication. No broad static or product qualification was needed
for this documentation-only change.
