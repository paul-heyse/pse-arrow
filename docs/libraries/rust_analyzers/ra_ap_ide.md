# `ra_ap_ide` — Comprehensive Rust Library Overview

Target version: 0.0.358 · Published October 12, 2026 · Research date: October 29, 2026

Upstream: rust-analyzer, maintained by the Rust project\
Language: Rust 2024 edition · Minimum Rust version: 1.98\
License: MIT OR Apache-2.0\
Package: `ra_ap_ide`

Primary references:

- Crate: [https://crates.io/crates/ra_ap_ide](https://crates.io/crates/ra_ap_ide?utm_source=chatgpt.com)
- Published API: [https://docs.rs/ra_ap_ide/latest/ra_ap_ide/](https://docs.rs/ra_ap_ide/latest/ra_ap_ide/?utm_source=chatgpt.com)
- Upstream source: [https://github.com/rust-lang/rust-analyzer/tree/master/crates/ide](https://github.com/rust-lang/rust-analyzer/tree/master/crates/ide?utm_source=chatgpt.com)
- Architecture: [https://rust-analyzer.github.io/book/contributing/guide.html](https://rust-analyzer.github.io/book/contributing/guide.html?utm_source=chatgpt.com)

The latest published release I could verify is 0.0.358, although some search-indexed rustdoc pages still display older versions. The discussion below targets the published crate family, distinguishing version-sensitive details where necessary. The `0.0.x` versioning and frequent releases make exact-version pinning and API verification especially important.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Docs.rs+1



## 1. Purpose and architectural position

`ra_ap_ide` is the high-level, embeddable IDE intelligence interface to rust-analyzer. It exposes source-code understanding and transformations through file-oriented and position-oriented operations, reusing rust-analyzer's incremental parser, name resolution, type inference, macro expansion, semantic model, and cross-crate analysis.

Its significance extends beyond implementing language-server features. It supplies reusable Rust code-intelligence primitives for semantic navigation, reference analysis, automated refactoring, diagnostics, program structure extraction, code transformations, and contextual information retrieval.

The central distinction is between semantic compiler infrastructure and IDE-oriented semantic services:

| Layer                 | Primary responsibility                                                    |
| --------------------- | ------------------------------------------------------------------------- |
| `ra_ap_syntax`        | Lossless parsing, syntax trees, tokens, AST manipulation                  |
| `ra_ap_hir`           | Name resolution, types, traits, definitions, semantic relationships       |
| `ra_ap_ide_db`        | Incremental analysis database, shared IDE infrastructure, symbol indexing |
| `ra_ap_ide`           | Public-facing analysis queries and IDE-compatible results                 |
| `ra_ap_rust_analyzer` | Workspace orchestration, LSP protocol, editor/server integration          |

`ra_ap_ide` sits above several specialized implementation crates:

- `ra_ap_ide_completion`: context-sensitive completion.
- `ra_ap_ide_assists`: automated source transformations and refactorings.
- `ra_ap_ide_diagnostics`: source-oriented diagnostics and quick fixes.
- `ra_ap_ide_ssr`: structural search and replacement.

These are real crate dependencies, rather than functionality entirely implemented inside `ra_ap_ide`. The top-level API integrates their capabilities behind a common analysis state and result model.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+3



### Why this architecture matters

A library consumer can combine operations that ordinarily require considerable custom infrastructure:

- Resolve an identifier, identify its semantic definition, locate references and determine applicable transformations.
- Interpret a source location using complete crate-level context rather than token matching or syntax alone.
- Generate edits from semantic analysis instead of manually reconstructing source text.
- Retain incrementally computed analysis between queries instead of rebuilding an entire compiler representation for each request.
- Retrieve code structure and semantic context without implementing or communicating through an LSP server.

Important boundary: `ra_ap_ide` is not a standalone Cargo workspace loader, repository indexer, compiler replacement, or complete language server. It analyzes a supplied world state. External orchestration remains responsible for constructing that world state, maintaining its files and crate graph, and consuming results.

The upstream architecture explicitly separates analysis from filesystem I/O: an external client provides file contents and project information, then submits queries against the current state.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://rust-analyzer.github.io\&sz=32)

rust-analyzer+1



## 2. Analysis state, incremental computation, and concurrency

### 2.1 `AnalysisHost` and `Analysis`: the core execution model

The API separates mutable inputs from semantic queries.

| Component              | Function                                              | Important properties                        |
| ---------------------- | ----------------------------------------------------- | ------------------------------------------- |
| `AnalysisHost`         | Owns and updates analysis state                       | Mutable, single authoritative state         |
| `Analysis`             | Queries an analysis snapshot                          | Sendable, not `Sync`; cancelable            |
| `RootDatabase`         | Salsa-backed incremental query database               | Caches and invalidates derived computations |
| `FileChange`           | Describes changes to analysis inputs                  | File contents, source roots, crate graph    |
| `ChangeWithProcMacros` | Bundles file changes and procedural macro information | Used by `AnalysisHost::apply_change`        |

Principal entry points:

- `AnalysisHost::new(lru_capacity)` — create the host with configurable cache capacity.
- `AnalysisHost::with_database(db)` — initialize from a supplied database.
- `AnalysisHost::apply_change(change)` — update the analysis world and invalidate affected computations.
- `AnalysisHost::analysis()` — obtain an analysis snapshot.
- `Analysis::from_single_file(...)` — construct an isolated analysis context without loading a complete workspace.

An important nuance: `apply_change` accepts `ChangeWithProcMacros`, from the supporting IDE database infrastructure, rather than accepting a bare `FileChange` directly.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+2



Source: [https://docs.rs/ra_ap_ide/latest/ra_ap_ide/struct.AnalysisHost.html](https://docs.rs/ra_ap_ide/latest/ra_ap_ide/struct.AnalysisHost.html?utm_source=chatgpt.com)

### 2.2 Incremental Salsa query computation

rust-analyzer's architecture is designed to reuse prior computations across source changes.

- Derived-query invalidation: file and configuration changes propagate through query dependencies. Unaffected results can be reused.
- Lazy computation: semantic information is computed when queries demand it; acquiring a snapshot does not imply exhaustively analyzing every source file.
- Cross-query reuse: navigation, diagnostics, completion, and other services share the underlying analysis database.
- Read-mostly optimization: library source roots are treated as relatively immutable, making dependency analysis less sensitive to frequent application-source edits.
- Explicit input versions: source updates are applied to the database rather than independently discovered by query methods.

The architectural opportunity is substantial: a host serving multiple analysis functions can share expensive parsing, name resolution, and type-inference work rather than maintaining separate indexes and semantic engines for each capability.

The performance benefit is conditional. A query may still require large amounts of previously uncomputed semantic work, and changes to high-level dependencies can invalidate more results than localized edits.

Source: [https://rust-analyzer.github.io/book/contributing/guide.html](https://rust-analyzer.github.io/book/contributing/guide.html?utm_source=chatgpt.com)

### 2.3 World-state construction: `FileChange`, source roots, and crate graphs

`FileChange` supports three central operations:

- `change_file(file_id, new_text)` — insert, update, or remove file contents.
- `set_roots(roots)` — establish the source-root partition.
- `set_crate_graph(graph)` — establish compilation-unit relationships.

`SourceRoot` maps virtual filesystem paths to file identifiers and distinguishes local source from dependency/library source. Paths are interpreted relative to their roots.

`CrateGraphBuilder` supplies compilation-unit construction and dependency relationships, including crate roots, editions, configuration options, environment, and dependency edges. A crate is a compilation unit, not necessarily the same entity as a Cargo package: packages may contain library, binary, test, and example targets.

The resulting distinction is important:

- File topology establishes where source lives and how module paths resolve.
- Crate topology determines dependency relationships and compilation context.
- Conditional configuration determines which portions of Rust source are semantically active.
- Procedural macro configuration determines whether generated items and expressions can be analyzed correctly.

A source file can participate in multiple crate contexts. Consequently, a purely file-based representation cannot always preserve Rust's complete compilation semantics.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://rust-lang.github.io\&sz=32)

Rust+3



Sources:

[https://rust-lang.github.io/rust-analyzer/ide/struct.FileChange.html](https://rust-lang.github.io/rust-analyzer/ide/struct.FileChange.html?utm_source=chatgpt.com)

[https://rust-lang.github.io/rust-analyzer/ide/struct.CrateGraphBuilder.html](https://rust-lang.github.io/rust-analyzer/ide/struct.CrateGraphBuilder.html?utm_source=chatgpt.com)

### 2.4 Cancellation and concurrency

`Analysis` queries generally return `Cancellable<T>`. Updating the host cancels outstanding snapshots and ensures queries do not silently continue against obsolete state.

Consequences:

1. Cancellation is part of normal operation, not necessarily a semantic-analysis failure.
2. Long-running queries must be retried against a new snapshot when fresh answers are required.
3. Concurrent analysis requests can use separate snapshots.
4. `Analysis` is `Send` but not `Sync`; the interface is not designed around freely sharing a single snapshot by concurrent reference.
5. State updates and query execution must be coordinated rather than treated as independent transactions.

The upstream implementation uses Salsa's cancellation machinery, including special unwinding handled at the analysis boundary. This is distinct from returning incomplete but successful answers.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://rust-analyzer.github.io\&sz=32)

rust-analyzer+1



### 2.5 Cache management and instrumentation

Less obvious host facilities include:

- `update_lru_capacity` — adjust the general query-cache capacity.
- `update_lru_capacities` — specify capacities for selected query families.
- `trigger_cancellation` — explicitly cancel ongoing analysis.
- `trigger_garbage_collection` — request cleanup of database-held resources.
- `parallel_prime_caches` — warm analysis caches concurrently for a specified crate scope, reporting progress.
- `status` — return analysis-state debugging information.
- `per_query_memory_usage` — inspect query-associated memory usage; importantly, the documented operation clears the database.
- `raw_database` / `raw_database_mut` — access the underlying database rather than relying exclusively on the high-level IDE interface.

`parallel_prime_caches` requires a crate scope closed under transitive dependencies. The operation creates an opportunity to prepare frequently needed analysis before expensive user-facing queries, at the cost of work and memory that might not otherwise have been necessary.

Architectural distinction: explicit cache priming is not always advantageous. Lazy analysis naturally avoids unnecessary computation; priming favors predictable latency for anticipated queries.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+1



## 3. Semantic navigation, symbol relationships, and dependency analysis

This family is especially important because its outputs are grounded in Rust semantics rather than textual similarity.

### 3.1 Definition and implementation navigation

| API                    | Semantic question                                                  |
| ---------------------- | ------------------------------------------------------------------ |
| `goto_definition`      | What definition does this source reference resolve to?             |
| `goto_declaration`     | Where is the corresponding declaration?                            |
| `goto_type_definition` | What type definition is associated with this expression or symbol? |
| `goto_implementation`  | Which implementations relate to this trait or type?                |
| `parent_module`        | Which module declaration introduces the current module?            |
| `child_modules`        | Which modules are introduced by the current module?                |

`NavigationTarget` represents results through file identifiers, source ranges, symbol names, kinds, and optional descriptive context.

Its two principal ranges have different meanings:

- `full_range` encompasses the source element, potentially including its body, attributes, and documentation.
- `focus_range` identifies the useful navigation destination, typically its name or defining token.

This difference supports both precise source jumps and structural containment operations.

Navigation may return multiple targets. A definition can also be represented through generated code, imports, or a declaration separate from the underlying implementation, so callers should not assume that a cursor position maps to exactly one source location.

A further distinction: `goto_implementation` is not equivalent to resolving the concrete runtime target of a dynamically dispatched call. It exposes semantically related implementation definitions, while runtime dispatch remains context-dependent.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://rust-lang.github.io\&sz=32)

Rust+1



Source: [https://docs.rs/ra_ap_ide/latest/ra_ap_ide/struct.NavigationTarget.html](https://docs.rs/ra_ap_ide/latest/ra_ap_ide/struct.NavigationTarget.html?utm_source=chatgpt.com)

### 3.2 Semantic reference search

`Analysis::find_all_refs` resolves the symbol at a position and searches for its references.

`FindAllRefsConfig` supports:

- Explicit `SearchScope` restriction.
- Exclusion of import references.
- Exclusion of test references.
- Fixture-related contextual configuration.

`ReferenceSearchResult` contains declaration information and references grouped by file. Each reference includes a `TextRange` and `ReferenceCategory` metadata that can distinguish aspects of usage, such as reads and writes.

The result is more expressive than a flat occurrence list. It supports relationships among definitions, reference locations, and usage categories while preserving source provenance.

Important semantics:

- Two identical identifier strings may resolve to different definitions.
- Imports and aliases affect identity and visibility.
- Some references arise through macro expansion or generated items.
- Results depend on the available crate graph, source contents, configuration, and supported resolution semantics.
- Restricting the search scope improves efficiency but can deliberately produce a non-exhaustive result.

`SearchScope` supports single-file, file-range, multi-file, and module-subtree construction, plus scope intersection. Automatic search-scope decisions can exploit visibility: local variables may be restricted to a small source range, whereas public definitions may require searching dependent crates.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://rust-lang.github.io\&sz=32)

Rust+2



Sources:

[https://rust-lang.github.io/rust-analyzer/ide/struct.ReferenceSearchResult.html](https://rust-lang.github.io/rust-analyzer/ide/struct.ReferenceSearchResult.html?utm_source=chatgpt.com)

[https://rust-lang.github.io/rust-analyzer/ide/struct.SearchScope.html](https://rust-lang.github.io/rust-analyzer/ide/struct.SearchScope.html?utm_source=chatgpt.com)

### 3.3 Symbol search and retrieval

`Analysis::symbol_search(query, limit)` provides fuzzy symbol discovery.

The `Query` configuration model supports more than simple substring matching:

- Fuzzy, prefix, and exact matching strategies.
- Case sensitivity.
- Type-only filtering.
- Inclusion of dependency/library symbols.
- Associated-item search controls.
- Path-based qualification and filtering.
- Import exclusion.

This is useful when the desired entity is known by name, approximation, or partial path rather than a source cursor position.

Unlike reference search, symbol search is principally an entity-discovery mechanism. It returns candidate definitions; it does not automatically enumerate every occurrence of each candidate.

Combining symbol search with navigation and reference analysis allows name-based discovery to become semantic exploration without implementing a separate symbol resolver.

Source: [https://rust-lang.github.io/rust-analyzer/ide/struct.Query.html](https://rust-lang.github.io/rust-analyzer/ide/struct.Query.html?utm_source=chatgpt.com)&#x20;

[image](https://www.google.com/s2/favicons?domain=https://rust-lang.github.io\&sz=32)

Rust



### 3.4 Call hierarchy

Three operations form the call-hierarchy interface:

- `call_hierarchy` — identify callable candidates at a source position.
- `incoming_calls` — identify callers.
- `outgoing_calls` — identify callees.

`CallHierarchyConfig` includes test exclusion. `CallItem` combines a `NavigationTarget` with source ranges for the corresponding call relationships.

The distinction between a symbol-level call relationship and a call-site occurrence is consequential: one caller may invoke the same callee multiple times, and those occurrences matter for source navigation, transformation, and dependency characterization.

Limits: this is a static call hierarchy, not a runtime trace or complete dynamic call graph. Trait dispatch, function pointers, callbacks, macros, unresolved code, and configuration-dependent compilation all complicate call-target completeness.

It is useful for discovering potential execution relationships, but not sufficient by itself for proving runtime reachability or constructing an exhaustive interprocedural control-flow graph.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://rust-lang.github.io\&sz=32)

Rust+1



Source: [https://rust-lang.github.io/rust-analyzer/ide/struct.CallItem.html](https://rust-lang.github.io/rust-analyzer/ide/struct.CallItem.html?utm_source=chatgpt.com)

### 3.5 Semantic monikers and external identity

`Analysis::moniker` exposes an identity representation for definitions that may be referenced outside their source file.

The associated types include:

- `Moniker`
- `MonikerIdentifier`
- `MonikerKind`
- `MonikerDescriptorKind`
- `PackageInformation`
- `MonikerResult`

A `Moniker` combines a semantic identifier, a kind, and package information. It is intended to identify a definition independently of the immediate cursor location. The upstream description explicitly notes that visibility declarations do not determine whether a moniker is present.

This creates a distinction among three kinds of identity:

| Identity                | Scope and purpose                                                 |
| ----------------------- | ----------------------------------------------------------------- |
| `FileId` + `TextRange`  | Concrete source location in an analysis world                     |
| HIR definition identity | Semantic entity inside rust-analyzer's analysis model             |
| `Moniker`               | Representation of an externally referenceable definition identity |

Monikers are particularly relevant to semantic indexing and connecting definitions to references across independently analyzed contexts.

However, their presence should not be interpreted as a guarantee of globally stable, rename-proof identity across arbitrary source revisions.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://rust-lang.github.io\&sz=32)

Rust



Source: [https://rust-lang.github.io/rust-analyzer/ide/struct.Moniker.html](https://rust-lang.github.io/rust-analyzer/ide/struct.Moniker.html?utm_source=chatgpt.com)

## 4. Contextual completion, hover, and semantic presentation

### 4.1 Completion is semantic synthesis, not symbol lookup

`Analysis::completions` accepts a `CompletionConfig`, source position, and optional trigger character. It uses the current syntactic and semantic context to construct candidate expressions and edits.

The main capability families include:

| Family                            | Important mechanisms                                                                            |
| --------------------------------- | ----------------------------------------------------------------------------------------------- |
| Name and member completion        | Visible definitions, paths, methods, fields, associated items, trait-aware candidates           |
| Auto-import                       | Discover symbols not presently in scope and propose necessary imports                           |
| Expression completion             | Postfix transformations, callable snippets, automatic `await` and iteration-related completions |
| Context-sensitive code generation | Function-call insertions, user snippets, and other context-specific templates                   |
| Term search                       | Search for expressions capable of satisfying an expected type, subject to a work budget         |
| Candidate ranking                 | Contextual relevance, type compatibility, name matching, locality, import status                |

`CompletionConfig` exposes controls including `enable_postfix_completions`, `enable_imports_on_the_fly`, `enable_auto_await`, `enable_auto_iter`, `enable_term_search`, `term_search_fuel`, snippet capabilities, and import-path preferences.

Term search deserves particular attention. Rather than merely identifying a name with a matching prefix, it can search for expressions satisfying the expected type. Its fuel parameter matters because this is a bounded synthesis/search operation, not a guarantee that all valid expressions will be found.

`CompletionItem` distinguishes its visible `label`, lookup text, detail/documentation, semantic kind, relevance, replacement range, and actual `TextEdit`. It can also carry information about an import to add.

`resolve_completion_edits` provides a separate step for computing additional edits, such as imports. This separation avoids eagerly preparing every expensive edit for every displayed candidate.

`CompletionRelevance` records factors affecting ranking, including name/type matches and whether an import is required. The resulting score is a selection heuristic, not a formal assertion of semantic preference or runtime suitability.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+3



Sources:

[https://docs.rs/ra_ap_ide/latest/ra_ap_ide/struct.CompletionConfig.html](https://docs.rs/ra_ap_ide/latest/ra_ap_ide/struct.CompletionConfig.html?utm_source=chatgpt.com)

[https://docs.rs/ra_ap_ide/latest/ra_ap_ide/struct.CompletionItem.html](https://docs.rs/ra_ap_ide/latest/ra_ap_ide/struct.CompletionItem.html?utm_source=chatgpt.com)

[https://docs.rs/ra_ap_ide/latest/ra_ap_ide/struct.CompletionRelevance.html](https://docs.rs/ra_ap_ide/latest/ra_ap_ide/struct.CompletionRelevance.html?utm_source=chatgpt.com)

### 4.2 Hover, signature help, and documentation resolution

These operations expose different views of semantic information.

- `hover` produces a `HoverResult` containing formatted `Markup` and possible `HoverAction` entries. Depending on the entity and configuration, the information can include signatures, inferred types, documentation, and navigation-oriented actions.
- `signature_help` returns `SignatureHelp` associated with the invocation context, including callable parameter information.
- `external_docs` resolves documentation destinations for the source symbol, taking optional build-output and sysroot context.
- `highlight_related` identifies other source ranges related to a selected construct or symbol.

The distinction between hover content and documentation navigation is useful: the former provides an immediate semantic summary; the latter identifies an external documentation destination. Neither requires independently implementing Rust name resolution.

The API's markdown/markup representations are display-oriented products, not a replacement for structured HIR type and definition objects. When exact semantic manipulation is necessary, the lower-level `ra_ap_hir::Semantics` interface may be more appropriate.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+1



Sources:

[https://docs.rs/ra_ap_ide/latest/ra_ap_ide/struct.HoverResult.html](https://docs.rs/ra_ap_ide/latest/ra_ap_ide/struct.HoverResult.html?utm_source=chatgpt.com)

[https://docs.rs/ra_ap_ide/latest/ra_ap_ide/struct.SignatureHelp.html](https://docs.rs/ra_ap_ide/latest/ra_ap_ide/struct.SignatureHelp.html?utm_source=chatgpt.com)

### 4.3 Inlay hints expose otherwise implicit semantics

`inlay_hints` and `inlay_hints_resolve` provide a richer surface than inferred variable types.

`InlayHintsConfig` controls families such as:

| Information exposed                    | Semantic significance                                                     |
| -------------------------------------- | ------------------------------------------------------------------------- |
| Inferred types                         | Makes type inference visible at bindings and expressions                  |
| Parameter names                        | Connects call arguments with their semantic roles                         |
| Generic arguments                      | Makes otherwise omitted type parameters easier to inspect                 |
| Type and method adjustments            | Exposes auto-reference, dereference, and related implicit transformations |
| Chaining types                         | Shows intermediate types in method chains                                 |
| Closure return types and captures      | Exposes inferred closure behavior                                         |
| Binding modes                          | Illuminates pattern-binding semantics                                     |
| Lifetime elision                       | Reveals elided lifetime information                                       |
| Implicit drops                         | Exposes places where values may be dropped                                |
| Enum discriminants and implicit bounds | Makes derived declarations and constraints visible                        |

This is an important example of higher-level IDE APIs providing access to information that would otherwise require traversing type-inference and semantic representations manually.

Hints are configured independently and may be intentionally suppressed to reduce noise. `InlayHint` includes positioning, kind, label, and additional information; lazy resolution permits deferring details until they are requested.

In the published `Analysis` interface, `inlay_hints_resolve` identifies a specific hint using a source range, hash, and caller-supplied hashing function. That makes hint identity and snapshot consistency relevant when resolving previously generated results.

These are explanatory projections of semantic analysis, not a substitute for formal compiler diagnostics or a stable serialization of complete inferred types.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://rust-lang.github.io\&sz=32)

Rust+2



Source: [https://docs.rs/ra_ap_ide/latest/ra_ap_ide/struct.InlayHintsConfig.html](https://docs.rs/ra_ap_ide/latest/ra_ap_ide/struct.InlayHintsConfig.html?utm_source=chatgpt.com)

### 4.4 Semantic highlighting and presentation-oriented structure

`highlight`, `highlight_range`, `highlight_as_html`, and `highlight_as_html_with_config` provide syntax- and semantics-informed token classifications.

`HighlightConfig` controls behavior for categories such as strings, comments, punctuation, operators, documentation injection, and unresolved names. The output uses ranges with `HlTag` and `HlMods`, allowing a renderer to distinguish categories and modifiers without parsing and classifying Rust syntax independently.

`highlight_related` is different: rather than classifying an entire file, it identifies related source ranges for a selected construct. This is useful for understanding contextual relationships and local symbol occurrences.

The HTML functions provide a convenience rendering path; the range/tag representation is the more general integration point because it preserves rendering independence.

Other presentation primitives include `folding_ranges`, `file_structure`, `matching_brace`, `extend_selection`, and `file_line_index`. These derive outline, navigation, selection, and coordinate-mapping information directly from rust-analyzer's source model.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+1



Source: [https://docs.rs/ra_ap_ide/latest/ra_ap_ide/struct.HighlightConfig.html](https://docs.rs/ra_ap_ide/latest/ra_ap_ide/struct.HighlightConfig.html?utm_source=chatgpt.com)

## 5. Diagnostics, refactoring, and structured source transformation

### 5.1 Diagnostics: syntax versus semantic analysis

Three separate entry points are exposed:

| Operation              | Coverage                                      |
| ---------------------- | --------------------------------------------- |
| `syntax_diagnostics`   | Parser-level problems in a file               |
| `semantic_diagnostics` | Problems identified through semantic analysis |
| `full_diagnostics`     | Combined syntax and semantic diagnostics      |

`Diagnostic` includes a diagnostic code, message, text range, severity, and optional fixes. Additional flags distinguish properties such as unused-code and experimental diagnostics.

`DiagnosticsConfig` allows selective diagnostic behavior: enabling or disabling categories, excluding named codes, controlling experimental diagnostics, macro analysis, and configuring certain fix-generation choices.

Do not equate these results with a complete compiler run. rust-analyzer implements IDE-oriented diagnostics, including HIR-derived diagnostics. `rustc` and Clippy also supply distinct diagnostics; in the complete rust-analyzer language server, those tools can be invoked externally and their results combined with IDE analysis. Calling `ra_ap_ide::Analysis::full_diagnostics` alone does not perform a Cargo build or Clippy analysis.

This boundary is especially consequential for borrow checking, detailed lifetime validation, compiler-specific lints, code generation, and target-dependent checks. A reported clean IDE diagnostic set is not equivalent to confirmed compilation.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+2



Sources:

[https://docs.rs/ra_ap_ide/latest/ra_ap_ide/struct.DiagnosticsConfig.html](https://docs.rs/ra_ap_ide/latest/ra_ap_ide/struct.DiagnosticsConfig.html?utm_source=chatgpt.com)

[https://docs.rs/ra_ap_ide/latest/ra_ap_ide/struct.Diagnostic.html](https://docs.rs/ra_ap_ide/latest/ra_ap_ide/struct.Diagnostic.html?utm_source=chatgpt.com)

[https://docs.rs/ra_ap_ide_diagnostics/latest/ra_ap_ide_diagnostics/](https://docs.rs/ra_ap_ide_diagnostics/latest/ra_ap_ide_diagnostics/?utm_source=chatgpt.com)

### 5.2 Assists: contextual transformations and quick fixes

`assists_with_fixes` combines generally available refactorings with applicable diagnostic quick fixes for a selected source range.

The assist infrastructure supports transformations such as extracting or restructuring code, converting constructs, modifying declarations, changing import organization, and generating context-dependent Rust code. The exact inventory evolves with rust-analyzer releases.

The important structure is:

- `AssistId` identifies the transformation.
- `AssistKind` identifies its broad category.
- `Assist` contains the label, target range, optional grouping, and optionally computed `SourceChange`.
- `AssistConfig` controls available categories, import preferences, snippet capability, and behavior of selected transformations.
- `AssistResolveStrategy` controls which edits are actually materialized.

The distinction between discovering an applicable action and computing its edit is important. Some refactorings involve semantic analysis and potentially extensive source changes; an application can enumerate available actions without fully calculating every transformation.

`AssistConfig` also exposes the less obvious term-search controls, expression-default filling, path-selection preferences, and rename-conflict presentation behavior. These settings influence what code certain assists can synthesize, not merely how an editor menu looks.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+2



Sources:

[https://docs.rs/ra_ap_ide/latest/ra_ap_ide/struct.AssistConfig.html](https://docs.rs/ra_ap_ide/latest/ra_ap_ide/struct.AssistConfig.html?utm_source=chatgpt.com)

[https://docs.rs/ra_ap_ide/latest/ra_ap_ide/struct.Assist.html](https://docs.rs/ra_ap_ide/latest/ra_ap_ide/struct.Assist.html?utm_source=chatgpt.com)

### 5.3 Semantic rename and filesystem-aware refactoring

`rename`, `prepare_rename`, and `will_rename_file` form a coherent rename interface.

- `prepare_rename` verifies whether a symbol at a position can be renamed and identifies the relevant range.
- `rename` computes a `SourceChange` reflecting semantic references to the definition and returns a `RenameError` when the operation cannot be performed.
- `will_rename_file` computes changes associated with renaming a module-related source file.

Unlike textual replacement, semantic rename can account for symbol identity, scope, and references across files. Renaming a file can also require updating module declarations or imports.

Rust introduces particularly important complications: import aliases, field shorthand, macros, module declarations, same-spelled definitions in different scopes, and configuration-dependent references. The presence of semantic rename does not remove all ambiguity, nor does it guarantee that every macro-generated or dynamically constructed use can be transformed.

The result is a proposed edit, not a filesystem mutation. The caller decides whether and how to apply it, including conflict checking and handling outstanding source revisions.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+1



Source: [https://docs.rs/ra_ap_ide/latest/ra_ap_ide/struct.RenameConfig.html](https://docs.rs/ra_ap_ide/latest/ra_ap_ide/struct.RenameConfig.html?utm_source=chatgpt.com)

### 5.4 Structural search and replace (`SSR`)

`structural_search_replace` is one of the more distinctive capabilities surfaced through `ra_ap_ide`.

Its implementation comes from `ra_ap_ide_ssr`, and the operation combines syntax-tree pattern matching with semantic knowledge.

Its pattern language uses:

- A search/replacement separator, `==>>`.
- Named placeholders such as `$expr`.
- Source-pattern matching for expressions, types, paths, patterns, and items.
- Matching of paths by their resolved semantic identity in applicable contexts.
- Replacement templates capable of reusing captured syntax.
- Contextual transformation logic for certain equivalent Rust call forms.
- Selection restrictions to constrain the transformation's search space.

For example, the schematic rule `foo($x) ==>> bar($x)` expresses a structured rewrite that retains the matched expression, rather than an unconstrained string substitution.

The documented implementation also includes treatment of method-call and fully qualified call forms, and some automatic reference/dereference adaptation. These are reasons to prefer a semantic structural engine over regular-expression replacement when the intended rule depends on Rust syntax and name resolution.

The top-level method accepts a rule string, a parse-only flag, a position supplying resolution context, and selected file ranges. It returns a `SourceChange` or `SsrError`.

Limits: SSR is not a general proof-preserving Rust program-equivalence engine. Rules depend on syntactic shapes and semantic context; the constructed replacement can still require subsequent validation. Macro expansion, formatting, imports, and rewriting across different compilation configurations introduce additional complexity.

The upstream LSP exposure of SSR has been described as experimental. That is a statement about the editor/protocol integration, distinct from the existence of a published library API.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

source+3



Sources:

[https://docs.rs/ra_ap_ide_ssr/latest/ra_ap_ide_ssr/](https://docs.rs/ra_ap_ide_ssr/latest/ra_ap_ide_ssr/?utm_source=chatgpt.com)

[https://docs.rs/ra_ap_ide_ssr/latest/src/ra_ap_ide_ssr/lib.rs.html](https://docs.rs/ra_ap_ide_ssr/latest/src/ra_ap_ide_ssr/lib.rs.html?utm_source=chatgpt.com)

### 5.5 Source edits are first-class results

`SourceChange` is the shared representation behind semantic rename, assists, and structural rewrites.

It can contain:

| Component                          | Meaning                                                                                     |
| ---------------------------------- | ------------------------------------------------------------------------------------------- |
| `source_file_edits`                | Per-file `TextEdit` and optional `SnippetEdit`                                              |
| `file_system_edits`                | File creation, movement, or other filesystem-level operations represented by the edit model |
| `is_snippet`                       | Indicates snippet-specific editing behavior                                                 |
| `annotations`                      | Metadata associated with changes                                                            |
| `merge` and edit insertion methods | Composition of proposed edits                                                               |

`TextEdit` represents ordered textual changes using source offsets; an `Indel` is an atomic insertion/deletion operation.

The ability to represent edits across multiple files, with filesystem operations and snippets, makes it possible to separate semantic decision-making from change application. This can eliminate much custom edit-generation logic, while preserving external control over validation, preview, atomicity, and persistence.

The critical integration obligation is that offsets refer to a particular source state. Applying an edit after the underlying files change can invalidate ranges or introduce conflicts. The host's snapshot cancellation does not independently make application of a returned `SourceChange` transactional.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust



Source: [https://docs.rs/ra_ap_ide/latest/ra_ap_ide/struct.SourceChange.html](https://docs.rs/ra_ap_ide/latest/ra_ap_ide/struct.SourceChange.html?utm_source=chatgpt.com)

### 5.6 Smaller editing services

Other native operations are worth distinguishing from refactorings:

- `join_lines` computes edits to join lines while handling selected Rust punctuation and spacing concerns.
- `on_enter` generates context-dependent newline behavior, including comment continuation.
- `on_char_typed` generates automatic corrections after selected characters.
- `move_item` moves eligible structural items.
- `extend_selection` expands a selected range through surrounding syntactic structures.
- `matching_brace` locates corresponding delimiters.

These features are primarily syntax-aware editing services. They may use semantic context where relevant, but they are not interchangeable with high-level semantic assists.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust



## 6. Static indexing, structural discovery, and code intelligence

### 6.1 `StaticIndex`: a particularly important advanced capability

`StaticIndex` provides a static representation of analyzed Rust source for applications that need durable code-navigation information rather than interactive cursor-by-cursor queries.

Its explicitly documented intended uses include read-only code browsers and LSIF/SCIP indexing.

In version 0.0.358, the published constructor is:

`StaticIndex::compute(analysis, vendored_libs_config, include_hovers)`

The `include_hovers` argument is significant: collecting full hover material increases the information available in the output and may increase indexing work and output size. Older upstream documentation shows a two-argument signature; the published 0.0.358 interface is authoritative for this version.

The result exposes:

| Type                | Contents                                                                                                     |
| ------------------- | ------------------------------------------------------------------------------------------------------------ |
| `StaticIndex`       | Indexed files and a shared `TokenStore`                                                                      |
| `StaticIndexedFile` | `FileId`, folding ranges, and `(TextRange, TokenId)` mappings                                                |
| `TokenStaticData`   | Definition, definition body, references, moniker, symbol kind, display name, signature, documentation, hover |

`TokenStaticData` is especially consequential because it connects source locations with richer semantic identities and relationships. Its `definition` and `definition_body` are deliberately distinct: one identifies the defining token, while the other identifies the enclosing definition.

That distinction permits source navigation and structural containment queries to share a common representation without conflating a symbol's name with the full item it defines.

The `references` collection and `moniker` field make the representation useful for joining semantic relationships across indexed files. Documentation and optional hover content enrich the same records without requiring a completely separate indexing pass for those presentation features.

The important architectural opportunity: `StaticIndex` can replace much custom machinery otherwise needed to extract definition/reference links, source-range mappings, symbol information, and hover metadata for a read-only code intelligence layer.

It does not independently provide a persistent database, incremental index-update protocol, storage schema, or a complete SCIP/LSIF serialization and publication service. Those responsibilities remain external.

The published type is neither `Send` nor `Sync`; the index construction and ownership model should not be assumed to support unrestricted cross-thread sharing.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+2



Sources:

[https://docs.rs/ra_ap_ide/latest/ra_ap_ide/struct.StaticIndex.html](https://docs.rs/ra_ap_ide/latest/ra_ap_ide/struct.StaticIndex.html?utm_source=chatgpt.com)

[https://docs.rs/ra_ap_ide/latest/ra_ap_ide/struct.StaticIndexedFile.html](https://docs.rs/ra_ap_ide/latest/ra_ap_ide/struct.StaticIndexedFile.html?utm_source=chatgpt.com)

[https://docs.rs/ra_ap_ide/latest/ra_ap_ide/struct.TokenStaticData.html](https://docs.rs/ra_ap_ide/latest/ra_ap_ide/struct.TokenStaticData.html?utm_source=chatgpt.com)

### 6.2 Live analysis versus static semantic indexing

These are complementary modes with different tradeoffs.

| Dimension                | Interactive `Analysis` queries               | `StaticIndex`                                       |
| ------------------------ | -------------------------------------------- | --------------------------------------------------- |
| Primary model            | Query current source state                   | Materialize a semantic representation               |
| Computation              | Demand-driven and incremental                | Index-building pass                                 |
| Typical use              | Editing, navigation, completion, diagnostics | Read-only browsing, offline search/index generation |
| Information availability | Rich operations on demand                    | Information explicitly captured during indexing     |
| Updates                  | Apply changes to `AnalysisHost`              | Recompute or externally maintain serialized index   |
| Persistence              | In-memory analysis database                  | External persistence needed for durable results     |

The combination permits a general architecture in which interactive analysis supplies current information while an exported index supports inexpensive repeated read-only queries.

However, the library does not automatically synchronize these two representations. Staleness detection, version identifiers, invalidation, storage, and consistency guarantees across an exported index remain responsibilities of the surrounding system.

### 6.3 File outlines and document structure

`Analysis::file_structure` returns `StructureNode` entries suitable for constructing a hierarchical outline of symbols. `FileStructureConfig` includes control over whether local structures are included.

Related functionality:

- `folding_ranges` identifies regions that can be collapsed.
- `parent_module` and `child_modules` expose module navigation relationships.
- `file_line_index` maps between absolute source offsets and line/column positions.
- `parse` returns the syntax tree for a file.
- `source_root_id` and `is_local_source_root` classify where a file belongs.

This provides a useful boundary: file structure is not the same as semantic dependency structure. An outline can identify functions and nested items without resolving every cross-crate relationship; call hierarchy and reference search add the separate semantic layer.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+1



### 6.4 Code annotations and references as navigational metadata

`annotations` and `resolve_annotation` expose higher-level decorations associated with source ranges.

`AnnotationConfig` includes controls for categories such as:

- Implementation annotations.
- References to definitions and methods.
- Enum-variant references.
- Runnable annotations.

These capabilities support code-lens-like experiences: contextual information attached to definitions or source locations, potentially with additional details resolved only when requested.

The two-stage `annotations`/`resolve_annotation` interface again reflects a recurring design pattern in `ra_ap_ide`: generate a useful initial representation, then defer potentially costly detail until required.

Annotations should not be confused with complete dependency extraction. They are selected presentation-oriented semantic summaries, governed by configuration.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+1



Source: [https://docs.rs/ra_ap_ide/latest/ra_ap_ide/struct.AnnotationConfig.html](https://docs.rs/ra_ap_ide/latest/ra_ap_ide/struct.AnnotationConfig.html?utm_source=chatgpt.com)

## 7. Test discovery, execution targets, and crate-context introspection

### 7.1 Test discovery is richer than searching for `#[test]`

The published `Analysis` API exposes:

| Operation                            | Function                                                                            |
| ------------------------------------ | ----------------------------------------------------------------------------------- |
| `discover_test_roots`                | Find roots for hierarchical test discovery                                          |
| `discover_tests_in_crate`            | Discover tests within a crate context                                               |
| `discover_tests_in_crate_by_test_id` | Discover tests through a crate identifier suitable for the test-discovery interface |
| `discover_tests_in_file`             | Discover tests associated with one file                                             |
| `runnables`                          | Identify executable/test-oriented targets associated with a file                    |
| `related_tests`                      | Find tests related to a selected source position                                    |

`TestItem` provides a structured representation for a test explorer. `Runnable` exposes a navigation target, runnable kind, conditional-configuration information, and related metadata.

These are higher-level features than simply locating `#[test]` tokens. The analysis may consider module structure, recognized test forms, and crate configuration when discovering relevant items.

The distinction between `runnables` and `related_tests` is useful: the first identifies targets in a file; the second asks which tests relate to a particular source location.

Execution remains separate from discovery. `ra_ap_ide` does not run the discovered tests, orchestrate Cargo commands, manage test subprocesses, or collect execution outcomes. Translating a discovered target into an executable command belongs to the caller or to another rust-analyzer integration layer.

Similarly, discovery results should not be interpreted as a universal guarantee that every custom test framework, generated test, or configuration-specific target can be identified.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+1



Source: [https://docs.rs/ra_ap_ide/latest/ra_ap_ide/struct.Runnable.html](https://docs.rs/ra_ap_ide/latest/ra_ap_ide/struct.Runnable.html?utm_source=chatgpt.com)

### 7.2 Crate graph and configuration queries

`Analysis` exposes several crate-oriented queries that are easily overlooked because they sit alongside editor-focused APIs:

- `crates_for(file_id)` — identify crate contexts containing a file.
- `relevant_crates_for(file_id)` — identify potentially relevant crate contexts.
- `transitive_rev_deps(crate_id)` — navigate transitive reverse dependencies.
- `crate_edition(crate_id)` — inspect Rust edition context.
- `crate_root(crate_id)` — locate a crate's root file.
- `is_proc_macro_crate(crate_id)` — classify a procedural-macro crate.
- `is_crate_no_std(crate_id)` — identify `no_std`/`no_core` configuration.
- `fetch_crates()` — retrieve crate information.
- `view_crate_graph(full)` — render the crate graph as Graphviz DOT.

These queries bridge source-level analysis with the compilation-unit graph.

An important consequence is that the same file cannot always be interpreted correctly without a crate context. Features, editions, macro availability, dependencies, and environment can change which items exist or how code resolves.

`transitive_rev_deps` provides an especially useful traversal primitive when examining which crates depend on a given compilation unit. This is a dependency-graph relationship, not evidence that a specific function is called or executed.

Conversely, `crates_for` should not be interpreted as package ownership metadata in the Cargo sense: the underlying crate graph models rustc compilation units.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust



Source: [https://docs.rs/ra_ap_ide/latest/ra_ap_ide/struct.Analysis.html](https://docs.rs/ra_ap_ide/latest/ra_ap_ide/struct.Analysis.html?utm_source=chatgpt.com)

## 8. Macro analysis and specialized semantic inspection

### 8.1 Macro expansion as an analysis dependency

Rust's macro system is a major reason why code-intelligence implementations based only on original source ASTs are incomplete.

`Analysis::expand_macro(position)` exposes macro expansion information through `ExpandedMacro`, containing the macro name and expansion text.

This lets a consumer inspect generated Rust that may be necessary to understand definitions, implementations, diagnostics, and navigation relationships.

There are three materially different cases:

| Macro mechanism                   | Analysis considerations                                              |
| --------------------------------- | -------------------------------------------------------------------- |
| Declarative macros                | Rust macro expansion and hygiene must be modeled                     |
| Procedural macros                 | Analysis may require an external procedural-macro execution service  |
| Built-in/compiler-specific macros | Semantics may depend on compiler-provided behavior and configuration |

The high-level `expand_macro` result is a human-readable expansion, not a complete public representation of all expansion provenance and hygiene relationships.

Source-to-expansion mapping is a deeper concern. A generated token may have a source origin that differs from its apparent expanded location; some generated elements may not correspond to a single editable original-source range.

For operations that need exact macro-aware semantic mappings, lower-level `ra_ap_hir`, span infrastructure, and semantic source-mapping APIs deserve attention. A textual `ExpandedMacro` result alone is insufficient.

Procedural macro execution also introduces availability and correctness concerns: the relevant macro binaries and environment must be provided, and executing third-party procedural macros is a distinct trust and resource-management consideration.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+2



Source: [https://docs.rs/ra_ap_ide/latest/ra_ap_ide/struct.ExpandedMacro.html](https://docs.rs/ra_ap_ide/latest/ra_ap_ide/struct.ExpandedMacro.html?utm_source=chatgpt.com)

### 8.2 Semantic inspection and debugging operations

Beyond the familiar IDE services, `Analysis` exposes specialized introspection capabilities:

| Operation                     | Exposed information                                      | Important boundary                                                              |
| ----------------------------- | -------------------------------------------------------- | ------------------------------------------------------------------------------- |
| `view_syntax_tree`            | Readable syntax-tree representation                      | Syntax, not resolved semantics                                                  |
| `view_hir`                    | Internal semantic representation at a source position    | Debugging representation, not a stable general-purpose IR export                |
| `view_item_tree`              | Item-tree representation of a file                       | Item-level structure rather than full executable semantics                      |
| `view_mir`                    | MIR-oriented representation at a position                | rust-analyzer's MIR capabilities, not necessarily rustc's complete MIR pipeline |
| `interpret_function`          | Evaluation through rust-analyzer's interpreter machinery | Not an arbitrary Rust execution engine                                          |
| `evaluate_predicate`          | Evaluation of a predicate in semantic context            | Subject to solver and analysis limitations                                      |
| `get_failed_obligations`      | Details about failed semantic obligations                | Diagnostic inspection rather than full proof extraction                         |
| `get_recursive_memory_layout` | Memory-layout-related information for a selected type    | Analysis-dependent, not a general target ABI guarantee                          |

These are significant capabilities because they allow inspection of intermediate semantic decisions without independently building and maintaining a complete Rust compiler frontend.

Two distinctions matter.

First, an IDE-oriented representation is not necessarily equivalent to rustc's final internal representation. rust-analyzer deliberately performs responsive, incremental analysis of incomplete source code, and some analyses have different completeness and precision characteristics.

Second, these interfaces should not automatically be treated as stable structured interchange formats. Several expose human-readable `String` results, useful for diagnostics and exploration but less suitable for durable typed semantic integration.

Where structured type, trait, resolution, or HIR information is required, using `ra_ap_hir` directly may avoid parsing diagnostic text back into objects.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust



Source: [https://docs.rs/ra_ap_ide/latest/ra_ap_ide/struct.Analysis.html](https://docs.rs/ra_ap_ide/latest/ra_ap_ide/struct.Analysis.html?utm_source=chatgpt.com)

## 9. Integration boundaries, guarantees, and architectural tradeoffs

### 9.1 Choosing the appropriate abstraction layer

The `ra_ap_*` libraries are related, but they solve different problems.

| Requirement                                                      | Most direct layer                                   | Why                                                       |
| ---------------------------------------------------------------- | --------------------------------------------------- | --------------------------------------------------------- |
| Parse incomplete Rust and preserve source structure              | `ra_ap_syntax`                                      | Lossless syntax without needing complete crate semantics  |
| Resolve symbols, inspect types and traits, explore HIR           | `ra_ap_hir`                                         | Structured semantic entities and operations               |
| Deliver completion, references, diagnostics, assists, navigation | `ra_ap_ide`                                         | Integrated, file/range-oriented user-facing services      |
| Export definition/reference navigation for static browsing       | `ra_ap_ide::StaticIndex`                            | Precomputed semantic indexing information                 |
| Load and model Cargo workspaces                                  | `ra_ap_project_model` and supporting infrastructure | Project discovery and compilation configuration           |
| Track filesystem changes and identities                          | `ra_ap_vfs` and surrounding orchestration           | File identity, paths, and change propagation              |
| Serve Language Server Protocol requests                          | `ra_ap_rust_analyzer`                               | Protocol, lifecycle, request dispatch, editor integration |
| Validate actual compilation                                      | `rustc` / Cargo                                     | Authoritative compiler execution                          |
| Produce compiler-integrated lint diagnostics                     | Clippy                                              | Compiler-oriented lint passes                             |

This separation matters because the higher-level library may reduce the amount of bespoke implementation for a complete IDE behavior, but it does not remove the need to manage a valid analysis world.

Conversely, direct use of `ra_ap_hir` is not always better simply because it is more expressive. When a task maps naturally to `Analysis::find_all_refs`, `Analysis::rename`, or `StaticIndex`, rebuilding the equivalent service from lower-level definitions may add unnecessary implementation and maintenance costs.

### 9.2 Completeness and correctness considerations

The principal constraints cut across most capability families:

Conditional compilation. Rust source has different meanings under different features, targets, editions, and configurations. A query is meaningful relative to the configured analysis world, not necessarily every possible build configuration.

Incomplete source tolerance. rust-analyzer is designed to provide information while source is being edited. This is an important advantage for interactive tooling but means the analysis model may contain unresolved or partially inferred information.

Procedural macro dependence. Missing or failing macro expansion can affect generated symbols, references, diagnostics, and navigation.

Dynamic behavior. Static references and call relationships do not establish exact runtime execution, dynamic-dispatch targets, or code reachability.

Symbol identity versus text position. `FileId`/`TextRange` locates source in a particular world state, while semantic definitions and monikers represent different identity concepts. None should be treated as an automatic permanent identifier across arbitrary source revisions.

Snapshot lifetime. Analysis snapshots are canceled when their owning host's world state changes. Results and edits must be associated with the source state that produced them.

Version compatibility. The `ra_ap_*` crate family is tightly coupled. The published dependency versions are exact, and the API evolves frequently. Mixing unrelated versions of `ra_ap_ide`, `ra_ap_hir`, and `ra_ap_ide_db` is not an appropriate assumption.

These are not reasons to avoid the higher-level library; they define where its semantics end and where external orchestration, compiler validation, or lower-level analysis must begin.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+2



### 9.3 Performance model

The main performance advantages follow from the design, rather than from any universal throughput claim.

| Mechanism                             | Potential benefit                               | Important cost or limitation                                        |
| ------------------------------------- | ----------------------------------------------- | ------------------------------------------------------------------- |
| Incremental Salsa database            | Reuse computations across queries and edits     | Invalidation depends on changed inputs and dependency relationships |
| Lazy semantic queries                 | Avoid computing unused information              | Initial queries can still be expensive                              |
| Search scopes                         | Reduce candidate sets for reference searches    | Restricted scopes can omit relevant references                      |
| Cached semantic foundations           | Share type/name-resolution work across services | Memory consumption and cache lifecycle                              |
| Deferred assist/completion resolution | Avoid preparing every candidate's edits         | Additional resolution request when selected                         |
| Configurable hints and diagnostics    | Avoid unnecessary detail or computation         | Less complete initial results                                       |
| Parallel cache priming                | Reduce later query latency                      | Proactive compute, CPU, and memory costs                            |
| Static index construction             | Prepare efficient read-only semantic retrieval  | Indexing time, storage, and freshness management                    |

The practical distinction is between interactive incremental workloads and batch whole-workspace extraction. The former benefit strongly from maintaining a warm analysis host; the latter can exploit static indexing but may need substantial upfront work and separate durable storage.

No performance ratio should be inferred without workload-specific measurement.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+2



### 9.4 Public API stability

Although published on crates.io, `ra_ap_ide` is an extraction of rust-analyzer's internal IDE layer rather than a conventional stable-semver application framework.

The `0.0.358` version number, exact companion dependencies, and distinction between upstream development rustdoc and published crate rustdoc are material.

In particular, do not assume that an API shown in rust-analyzer's development documentation exists with the identical signature in a specific published crate. The `StaticIndex::compute` signature difference is a concrete example.

When implementation depends on exact method signatures or configuration fields, the version-specific docs.rs API and source corresponding to the pinned release are the appropriate authority.

## 10. Capability synthesis

The strongest architectural value of `ra_ap_ide` is not an isolated feature such as go-to-definition or completion. It is the composition of semantic analysis, incremental state, source-aware navigation, structured transformations, and static indexing behind a common world model.

Three combinations are particularly consequential:

1\. Semantic exploration: symbol discovery → definition resolution → references → call hierarchy → documentation and type context. This yields an integrated view of how entities relate without separately implementing symbol resolution, reference classification, and cross-file navigation.

2\. Semantics-aware code modification: diagnostics or cursor context → applicable assist/rename/SSR → `SourceChange` → external preview and validation. This provides a transformation pipeline that can be more reliable than token- or string-based rewriting, while retaining control over edit application.

3\. Interactive and durable code intelligence: incremental `Analysis` queries for fresh information, combined with `StaticIndex` for precomputed semantic relationships. These mechanisms can share the same underlying semantic model even though their update, storage, and consistency requirements differ.

The main design decision is therefore whether the desired abstraction is a source-oriented IDE service or a structured semantic/compiler representation.

Use the conceptual capabilities of `ra_ap_ide` when the operation can be expressed in terms of source locations, definitions, references, suggested edits, diagnostics, or static navigation records. Investigate `ra_ap_hir` and its supporting crates when a task requires direct manipulation of Rust semantic entities, inferred types, traits, internal representations, or detailed macro mappings beyond what the high-level API exposes.

### Primary source map

These are the most useful direct entry points for deeper, version-aware research:

| Area                              | Primary URL                                                                                                                                                                           |
| --------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Published crate and API index     | [https://docs.rs/ra_ap_ide/latest/ra_ap_ide/](https://docs.rs/ra_ap_ide/latest/ra_ap_ide/?utm_source=chatgpt.com)                                                                     |
| Core `Analysis` query surface     | [https://docs.rs/ra_ap_ide/latest/ra_ap_ide/struct.Analysis.html](https://docs.rs/ra_ap_ide/latest/ra_ap_ide/struct.Analysis.html?utm_source=chatgpt.com)                             |
| Mutable state and caching         | [https://docs.rs/ra_ap_ide/latest/ra_ap_ide/struct.AnalysisHost.html](https://docs.rs/ra_ap_ide/latest/ra_ap_ide/struct.AnalysisHost.html?utm_source=chatgpt.com)                     |
| Published upstream source         | [https://github.com/rust-lang/rust-analyzer/tree/master/crates/ide](https://github.com/rust-lang/rust-analyzer/tree/master/crates/ide?utm_source=chatgpt.com)                         |
| Interactive completion engine     | [https://github.com/rust-lang/rust-analyzer/tree/master/crates/ide-completion](https://github.com/rust-lang/rust-analyzer/tree/master/crates/ide-completion?utm_source=chatgpt.com)   |
| Refactoring and assists           | [https://github.com/rust-lang/rust-analyzer/tree/master/crates/ide-assists](https://github.com/rust-lang/rust-analyzer/tree/master/crates/ide-assists?utm_source=chatgpt.com)         |
| Diagnostics implementation        | [https://github.com/rust-lang/rust-analyzer/tree/master/crates/ide-diagnostics](https://github.com/rust-lang/rust-analyzer/tree/master/crates/ide-diagnostics?utm_source=chatgpt.com) |
| Structural search and replacement | [https://github.com/rust-lang/rust-analyzer/tree/master/crates/ide-ssr](https://github.com/rust-lang/rust-analyzer/tree/master/crates/ide-ssr?utm_source=chatgpt.com)                 |
| Core semantic layer               | [https://github.com/rust-lang/rust-analyzer/tree/master/crates/hir](https://github.com/rust-lang/rust-analyzer/tree/master/crates/hir?utm_source=chatgpt.com)                         |
| Shared IDE database               | [https://github.com/rust-lang/rust-analyzer/tree/master/crates/ide-db](https://github.com/rust-lang/rust-analyzer/tree/master/crates/ide-db?utm_source=chatgpt.com)                   |
| rust-analyzer architecture        | [https://rust-analyzer.github.io/book/contributing/guide.html](https://rust-analyzer.github.io/book/contributing/guide.html?utm_source=chatgpt.com)                                   |

Bottom line: `ra_ap_ide` is a high-level Rust semantic-services library with substantial reusable capability beyond ordinary language-server functionality. Its combination of incremental analysis, symbol relationships, structured code transformations, test discovery, and SCIP/LSIF-oriented static indexing makes it a potentially valuable foundation for sophisticated Rust source-intelligence systems, provided its tightly coupled versions, compilation-context requirements, and separation from compiler execution are respected.