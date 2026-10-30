# rust-analyzer — Comprehensive Rust Library and Code Intelligence Reference

Research date: October 29, 2026 | Stable language-server release: 2026-10-26 (v0.3.3097) | Verified reusable crate series: ra_ap\_\* 0.0.358

## 1. Executive orientation

rust-analyzer is an incremental, IDE-oriented Rust compiler frontend that provides reusable syntax analysis, semantic resolution, type inference, macro expansion, cross-reference analysis, source transformations, and code intelligence.

Although best known as the Rust Language Server Protocol (LSP) implementation, its internal architecture is a collection of Rust libraries capable of powering standalone source-analysis tools, static-analysis systems, semantic indexes, refactoring engines, and alternative IDE frontends.

Its most consequential capability is not any individual editor feature. It is the ability to maintain a coherent semantic model of a Rust program while recomputing only the information invalidated by source changes.

### 1.1 Core architectural distinctions

| Layer              | Primary responsibility                                                     | Principal interfaces                                             |
| ------------------ | -------------------------------------------------------------------------- | ---------------------------------------------------------------- |
| Syntax             | Lossless parsing, syntax traversal, structural rewriting                   | `ra_ap_syntax`, `ra_ap_parser`                                   |
| Workspace model    | Files, source roots, dependencies, target configuration                    | `ra_ap_project_model`, `ra_ap_vfs`, `ra_ap_base_db`              |
| Incremental engine | Demand-driven computation, dependency tracking, invalidation, cancellation | Salsa, `RootDatabase`                                            |
| Semantic analysis  | Name resolution, HIR, types, traits, macro expansion                       | `ra_ap_hir`, `ra_ap_hir_def`, `ra_ap_hir_ty`, `ra_ap_hir_expand` |
| IDE intelligence   | References, navigation, completions, diagnostics, refactoring              | `ra_ap_ide`, `ra_ap_ide_db`, feature-specific crates             |
| Integration        | Cargo loading, LSP, external compiler diagnostics, editor coordination     | `ra_ap_load-cargo`, `rust-analyzer` binary                       |

The three most important integration boundaries are:

1. Syntax boundary: Analyze syntactic structure without building or resolving a project. This is often sufficient for structural queries, extraction, transformations, and tolerant analysis of incomplete files.
2. Semantic boundary: Analyze resolved Rust entities, types, expressions, trait relationships, and macro-expanded code within a configured crate graph.
3. IDE boundary: Consume semantic information through higher-level operations expressed in file positions, text ranges, navigation targets, diagnostics, and source edits.

The official architecture documentation deliberately treats `syntax`, `hir`, and `ide` as separate API boundaries. The lower-level semantic implementation crates are internal mechanisms rather than stable abstraction boundaries.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://rust-analyzer.github.io\&sz=32)

rust-analyzer+2



### 1.2 Version and distribution model

rust-analyzer has two distinct version identities.

- Language server: Published through weekly dated releases. The latest verified stable release is October 26, 2026, `v0.3.3097`, corresponding to commit `0c4d746`.
- Reusable Rust crates: Published separately with names such as `ra_ap_syntax`, `ra_ap_hir`, and `ra_ap_ide`. The current documentation verified for several core crates is version `0.0.358`.

These version numbers are not interchangeable. The reusable crates are pre-1.0, move quickly, and explicitly do not promise a stable API. The LSP interface offers a more durable integration contract than the internal Rust APIs.

Official sources:

- Release: `https://github.com/rust-lang/rust-analyzer/releases/tag/2026-10-26`
- Release changelog: `https://rust-analyzer.github.io/thisweek/2026/10/26/changelog-351.html`
- Architecture: `https://rust-analyzer.github.io/book/contributing/architecture.html`
- Repository: `https://github.com/rust-lang/rust-analyzer`

[image](https://www.google.com/s2/favicons?domain=https://github.com\&sz=32)

GitHub+2

## 2. Syntax analysis, parsing, and source transformations

### 2.1 `ra_ap_syntax`: lossless Rust syntax trees

`ra_ap_syntax` is the most self-contained component of rust-analyzer. It parses source independently of Cargo, type inference, dependency resolution, and the language server.

Its key properties are:

- Full-fidelity concrete syntax trees (CST): Whitespace, comments, tokens, and malformed constructs are retained. Source text can be reconstructed without losing formatting.
- Error-tolerant parsing: `Parse<T>` contains both a tree and parse errors. Invalid or unfinished files still yield navigable structure.
- Incremental reparsing: `Parse<SourceFile>::reparse` accepts a replaced range, inserted text, and edition. Eligible edits can reparse a substructure rather than rebuilding the entire tree.
- Typed AST views: `ast::Fn`, `ast::Struct`, `ast::Impl`, `ast::Expr`, `ast::Type`, `ast::Pat`, and related types provide typed access to otherwise untyped syntax nodes.
- Structural navigation: Parent, child, sibling, ancestor, descendant, and token traversal; text ranges; typed casting and node selection.
- Edition-aware parsing: Rust grammar is interpreted according to the specified language edition, independently of semantic project configuration.

The key distinction is that a syntax tree describes what the source says, not what its identifiers mean.

A method call can be recognized syntactically without knowing the receiver type, the actual function selected through method resolution, or the trait that provides the method.

Version detail: The independently published syntax crate has also reached `0.0.360` in the currently available docs.rs documentation. The `0.0.358` API references used for the rest of this document form a verified matching dependency set; higher-numbered packages should not automatically be mixed with it.

Source: `https://docs.rs/ra_ap_syntax/latest/ra_ap_syntax/`&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+1



### 2.2 Rowan's green/red tree architecture

rust-analyzer builds its syntax representation on `rowan`, a reusable lossless syntax-tree library.

| Representation                | Mechanism                                  | Architectural significance                                        |
| ----------------------------- | ------------------------------------------ | ----------------------------------------------------------------- |
| `GreenNode`                   | Immutable tree with structural sharing     | Efficient representation and replacement of unchanged subtrees    |
| `SyntaxNode`                  | Navigable contextual view over green nodes | Parent/child traversal and source-offset queries                  |
| Typed AST                     | Rust-specific wrappers over syntax nodes   | Convenient structural analysis without separate parsed data       |
| `SyntaxNodePtr` / `AstPtr<T>` | Syntax-location-based handles              | Reacquiring nodes without permanently retaining full tree handles |
| `TextSize` / `TextRange`      | Source positions and spans                 | Consistent location model for extraction and edits                |

The architecture deliberately keeps semantic data out of syntax nodes. A syntax tree represents one physical source file, while its semantic interpretation can depend on multiple crate contexts.

This separation allows syntax-based tooling to remain operational even when dependencies are unresolved, project metadata is unavailable, or source text is incomplete.

Important limitation: Syntax pointers are not universal persistent identities. Source changes can invalidate their locations or meaning. Cross-revision or cross-workspace identities require additional handling.

Source: `https://rust-analyzer.github.io/book/contributing/syntax.html` (historical architectural explanation, last explicitly scoped to 2020; the current APIs are independently verified).&#x20;

[image](https://www.google.com/s2/favicons?domain=https://rust-analyzer.github.io\&sz=32)

rust-analyzer+1



### 2.3 Parser and grammar mechanisms

`ra_ap_parser` is a separate parser implementation that works with abstract token input/output representations.

Its architecture includes:

- Handwritten recursive-descent grammar with event-based tree construction.
- Parsing designed to recover structure from malformed input rather than aborting.
- `Input`, `Output`, and parser entry points decoupled from concrete syntax-tree storage.
- Specialized reparsing support for eligible syntax constructs.
- Token-tree parsing that can support macro-related structures without requiring a normal physical source file.

The grammar is maintained separately from much of the typed AST machinery. Rust-analyzer also uses ungrammar, a grammar-description approach that supports generating syntax API structures.

This separation permits parser evolution without forcing the semantic engine or every syntax-tree consumer to change in the same way.

Source: `https://docs.rs/ra_ap_parser/latest/ra_ap_parser/`&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust



### 2.4 Syntax editing and AST construction

`ra_ap_syntax` includes more than parsing and traversal. It has a structural editing subsystem:

- `syntax_editor::SyntaxEditor`: Applies structural operations to syntax nodes.
- `SyntaxEdit`: Represents a completed editing operation.
- `SyntaxMapping`: Relates nodes in original and transformed structures.
- `ast::edit`: Editing operations that produce replacement trees rather than mutating immutable trees in place.
- `ast::make`: Constructors for Rust syntax fragments.
- `ast::syntax_factory`: Constructs fragments with additional syntax mappings.
- `ast::prec`: Expression-precedence representation.

These mechanisms support source transformations that require knowledge of Rust syntax: adjusting expressions, generating items, manipulating attributes, maintaining appropriate delimiters, or handling precedence.

The distinction between constructing a correct syntax fragment, building a complete source edit, and ensuring that a semantic transformation preserves program behavior is important. The syntax layer provides the first two kinds of machinery, not an automatic correctness guarantee for the third.

Sources:

- `https://docs.rs/ra_ap_syntax/latest/ra_ap_syntax/syntax_editor/index.html`
- `https://docs.rs/ra_ap_syntax/latest/ra_ap_syntax/ast/index.html`

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+1

## 3. Incremental semantic computation

### 3.1 Salsa: demand-driven analysis

rust-analyzer uses Salsa to maintain derived program information.

Instead of eagerly constructing and rebuilding a monolithic compiler model, analysis is organized around inputs and cached computations.

The broad flow is:

Input state

Source text · crate graph · cfg · target/environment data

Syntax

Parse & item summaries

Definitions

Scopes & name resolution

Types

Inference & trait solving

Semantic queries and IDE services

Definitions · references · completions · diagnostics · refactoring

Source changes invalidate affected computations; unaffected results remain reusable.

Consequential mechanisms:

- Demand-driven execution: Expensive derived facts can be calculated when a consumer asks for them.
- Dependency tracking: Previously computed results are reused when their relevant inputs have not changed.
- Fine-grained invalidation: An edit to a function body need not invalidate unrelated module-level information.
- Incremental data representations: Lowered source representations make it possible to separate changes to function implementations from changes to their surrounding declarations.
- Cancellation: Expensive queries can be abandoned when the underlying source revision changes.
- Concurrency: Independent queries and immutable analysis snapshots permit concurrent work, subject to the database's revision and mutation semantics.

This enables responsive analysis of changing source without maintaining a separate ad hoc dependency tracker for every code-intelligence operation.

### 3.2 `ItemTree`, `DefMap`, and body-level decomposition

Three intermediate representations are particularly important:

| Representation | Function                                                  | Incremental significance                              |
| -------------- | --------------------------------------------------------- | ----------------------------------------------------- |
| `ItemTree`     | Compact summary of declarations in a source file          | A body-only edit need not change item-level structure |
| `DefMap`       | Modules, item scopes, and name-resolution information     | Supports reusable crate/module-level resolution       |
| `Body`         | Lowered expressions, patterns, and body-local information | Supports more localized semantic recomputation        |

The design target explicitly described by rust-analyzer is that typing within one function body should not invalidate global derived facts for other functions.

This is a design invariant, not a guarantee that every real-world edit has negligible cost. Public signatures, imports, macros, crate configuration, and build metadata can affect much larger regions of the dependency graph.

### 3.3 `AnalysisHost` and immutable `Analysis` snapshots

The high-level analysis API separates the mutable state owner from query snapshots.

`AnalysisHost` owns the current analysis database and provides:

- `analysis()` to obtain a queryable snapshot.
- `apply_change(...)` to advance the source/project state.
- `trigger_cancellation()` to invalidate outstanding work.
- `update_lru_capacity(...)` and `update_lru_capacities(...)` for cache management.
- `trigger_garbage_collection()` and memory-inspection functionality.
- Access to `RootDatabase` when lower-level integration is necessary.

`Analysis` represents a view of a specific revision and exposes IDE queries.

When the host applies a new change, existing snapshots are canceled. Consumers must handle cancellation rather than assuming old snapshots can continue indefinitely.

This provides a useful separation between the responsibility for advancing program state and the responsibility for answering semantic questions.

Sources:

- `https://docs.rs/ra_ap_ide/latest/ra_ap_ide/struct.AnalysisHost.html`
- `https://docs.rs/ra_ap_ide/latest/ra_ap_ide/struct.Analysis.html`
- `https://rust-analyzer.github.io/book/contributing/architecture.html`

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+2

## 4. Workspace, build-system, and file-model integration

### 4.1 The abstract `CrateGraph`

rust-analyzer deliberately separates the semantic structure of Rust crates from the build system used to produce it.

The semantic input graph describes:

- Crate root files.
- Dependencies and names under which crates are imported.
- Language editions.
- Configuration flags and key-value `cfg` settings.
- Compilation-target information.
- Environment and build-derived information needed for interpretation.

Consequently, Cargo is a supported source of project information, not an intrinsic dependency of semantic analysis.

A non-Cargo build system can supply an equivalent crate graph without implementing a Rust compiler frontend.

A Rust source file may participate in more than one crate instance, with different dependencies, configurations, and semantics. A physical file's identity is therefore not sufficient to identify every resolved entity within it.

### 4.2 `ra_ap_project_model`

This crate converts concrete project descriptions into the abstract model.

Its components include `CargoWorkspace`, `ProjectWorkspace`, `ProjectJson`, `CargoConfig`, `CargoFeatures`, `Sysroot`, and build-script metadata.

Capabilities include:

- Workspace discovery and Cargo metadata interpretation.
- Feature and target selection.
- Discovery of binaries, libraries, tests, examples, and related compilation targets.
- Build-script output interpretation and generated-source integration.
- Procedural macro artifact discovery.
- Sysroot and standard-library source discovery.
- Support for externally supplied crate layouts.

Importantly, Cargo features become `cfg` inputs at the semantic boundary; the low-level analysis database does not itself implement Cargo feature resolution.

Source: `https://docs.rs/ra_ap_project_model/latest/ra_ap_project_model/`&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust



### 4.3 `rust-project.json`: integration without Cargo

The officially documented non-Cargo project representation can specify:

| Property                                    | Significance                                               |
| ------------------------------------------- | ---------------------------------------------------------- |
| `root_module`, `edition`, `deps`            | Root and dependency topology for each crate                |
| `cfg`, `cfg_groups`                         | Active conditional-compilation settings                    |
| `target`                                    | Target-specific compilation context                        |
| `env`                                       | Environment values used by macros                          |
| `source.include_dirs`, `exclude_dirs`       | Mapping files into source sets                             |
| `is_proc_macro`, `proc_macro_dylib_path`    | Procedural macro support                                   |
| `sysroot`, `sysroot_src`, `sysroot_project` | Standard-library or custom sysroot integration             |
| `crate_attrs`                               | Additional crate-level attributes                          |
| `runnables`                                 | Commands exposed for external build and test systems       |
| `build`                                     | Build-system-specific labels, files, and target categories |

This makes rust-analyzer capable of analyzing projects managed by other build systems, as long as equivalent information is supplied.

Boundary condition: A crate graph must accurately represent the relevant compilation context. Missing dependencies, incorrect feature flags, absent generated files, or incompatible target configuration can produce plausible but incorrect semantic results.

Source: `https://rust-analyzer.github.io/book/non_cargo_based_projects.html`&#x20;

[image](https://www.google.com/s2/favicons?domain=https://rust-analyzer.github.io\&sz=32)

rust-analyzer



### 4.4 VFS, file identities, and source roots

The virtual file system separates physical file access from analysis.

`ra_ap_vfs` provides `Vfs`, `VfsPath`, `FileId`, file sets, change collection, and file-loader abstractions.

Its design is more sophisticated than a normal path-to-text cache:

- File paths are interned into opaque identities.
- File additions, removals, and modifications are batched into changes.
- `FileSet` groups files into disjoint sets and supports relative-path resolution.
- File-loading and watching mechanisms are separate from the core VFS.
- The analysis database receives changes rather than reading the filesystem itself.
- Source roots distinguish local, frequently changing files from comparatively stable dependency sources.

A source root defines a file-resolution boundary. Path-based relationships such as module declarations and included files depend on how files have been grouped.

Architectural implication: Analysis can operate on virtual, generated, overlaid, or otherwise externally managed source content. The semantic core does not require source text to originate from a directly readable filesystem path.

Source: `https://docs.rs/ra_ap_vfs/latest/ra_ap_vfs/`&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust



### 4.5 `ra_ap_load-cargo`

`ra_ap_load-cargo` supplies utilities for loading a Cargo workspace into an analysis database.

Its documented primary operation is a static workspace load, without ongoing source-change incorporation. Interfaces include `load_workspace`, `load_workspace_at`, and `load_workspace_into_db`.

This is materially different from running the complete language server, which maintains filesystem watchers, editor overlays, workspace-reload behavior, and asynchronous request processing.

For batch analysis, the static loader can eliminate substantial custom project-discovery and graph-construction code. A continuously updated analyzer requires additional lifecycle machinery.

Source: `https://docs.rs/ra_ap_load-cargo/latest/ra_ap_load_cargo/`&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust



## 5. Semantic analysis: HIR, resolution, types, and macros

### 5.1 `ra_ap_hir`: the semantic abstraction boundary

The `hir` crate presents resolved Rust program entities as objects rather than exposing the lower-level database machinery.

Its model includes:

- Program structure: `Crate`, `Module`, `ModuleDef`, `Function`, `Struct`, `Enum`, `Union`, `Trait`, `Impl`, `TypeAlias`, `Const`, and `Static`.
- Members and bindings: `Field`, `EnumVariant`, `Local`, `Param`, `SelfParam`, and associated items.
- Type-system objects: `Type`, `TraitRef`, `GenericSubstitution`, callable signatures, generic parameters, and predicates.
- Source and scope: `Semantics`, `SemanticsScope`, `HasSource`, `PathResolution`, and `DefMap`.
- Macros and derived structure: `Macro`, `MacroCallId`, `ProcMacros`, and source-mapping interfaces.

These entities are tied to a compilation context. The same source file can contribute to several semantic crate instances, and a named Rust item can have distinct meaning under different `cfg` settings or dependency graphs.

Unlike the syntax tree, HIR is not merely a representation of the text. It is the result of interpreting Rust language rules within a particular program configuration.

Source: `https://docs.rs/ra_ap_hir/latest/ra_ap_hir/`&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust



### 5.2 `Semantics`: translating syntax locations into meaning

`Semantics<'db>` is one of the most important low-level APIs for building semantic source-analysis tools.

Its capabilities include:

| Operation                  | API handles                                                        | Information exposed                                                     |
| -------------------------- | ------------------------------------------------------------------ | ----------------------------------------------------------------------- |
| Resolve names              | `resolve_path`, `resolve_path_per_ns`, `resolve_mod_path`          | The declaration or declarations denoted by a syntactic path             |
| Resolve calls and members  | `resolve_method_call`, `resolve_field`, `resolve_expr_as_callable` | The semantic target selected by member lookup or call resolution        |
| Infer types                | `type_of_expr`, `type_of_pat`, `type_of_binding_in_pat`            | Inferred expression, pattern, and binding types                         |
| Inspect scopes             | `scope`, `scope_at_offset`                                         | Names and entities visible at a specific source position                |
| Map source to definitions  | `to_def`, `to_node`, `original_ast_node`                           | Relationships between syntax and HIR entities                           |
| Follow macro expansion     | `descend_into_macros`, `expand_macro_call`, `original_range`       | Navigation between macro invocation, expansion, and origin              |
| Inspect implicit semantics | `expr_adjustments`, `pattern_adjustments`, `get_unsafe_ops`        | Compiler-inserted coercions, binding adjustments, and unsafe operations |
| Evaluate conditions        | `evaluate_where_clause_at`, `get_failed_obligations`               | Predicate and trait-bound reasoning within the analyzed context         |

The significant capability here is contextual identity resolution. Two paths with different spelling can resolve to the same definition, while two identically spelled names in different lexical scopes can resolve to different definitions.

This enables semantic cross-referencing and transformations that ordinary text searches or standalone AST matching cannot reliably reproduce.

The API also exposes `original_range` and macro-aware traversal because resolution frequently reaches structures whose semantic location differs from their physical source location.

Source: `https://docs.rs/ra_ap_hir/latest/ra_ap_hir/struct.Semantics.html`&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust



### 5.3 Modules, scopes, and import resolution

Rust name resolution involves substantially more than traversing a module tree.

rust-analyzer handles:

- Multiple namespaces for types, values, and macros.
- `use` imports, glob imports, re-exports, aliases, and visibility.
- Nested lexical scopes and associated-item lookup.
- Module declarations spread across physical source files.
- Conditional compilation and crate-specific dependency names.
- Traits introduced through imports and their impact on method resolution.
- Macro-generated declarations and macro hygiene.

`DefMap` captures early resolution and module scopes; `Module`, `ModuleDef`, and `SemanticsScope` expose resolved structure at a higher level.

`hir::import_map` also provides an index of publicly exported items in a crate, supporting import discovery and candidate lookup.

A useful distinction is lexical visibility versus public reachability: a symbol may be accessible in a particular scope without being publicly exported, while a re-export can make a definition reachable under additional paths.

Source: `https://docs.rs/ra_ap_hir/latest/ra_ap_hir/`

### 5.4 Type inference and trait solving

`ra_ap_hir_ty` supplies Rust type inference, autoderef, coercion, trait solving, and type-related analyses used by completion, hovers, diagnostics, and assists.

Major capabilities include:

- Inference of expression and pattern types.
- Generic substitutions, type and const arguments, and associated types.
- Trait implementation and obligation evaluation.
- Method candidates, including inherent and trait-provided methods.
- Reference and pointer adjustments, autoderef, and coercions.
- Closures and captured variables.
- Async/coroutine types and future outputs.
- Iterator-related type queries.
- Trait-object and `impl Trait` representations.
- Type and layout introspection.

The high-level `Type` API exposes operations such as `iterate_method_candidates`, `impls_trait`, `normalize_trait_assoc_type`, `could_coerce_to`, `autoderef`, `future_output`, `iterator_item`, and `layout`.

Type identity is context-dependent: `Type` values remember their originating semantic context. The API warns that operating across incompatible origins can fail or panic; explicit instantiation or rebasing facilities are provided for supported cases.

Sources:

- `https://docs.rs/ra_ap_hir/latest/ra_ap_hir/struct.Type.html`
- `https://docs.rs/ra_ap_hir_ty/latest/ra_ap_hir_ty/`

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+1

### 5.5 Important current architecture: sharing rustc's next trait solver

A significant distinction from older rust-analyzer documentation is that the modern type engine uses components of rustc's next trait solver, rather than treating Chalk as its current standalone solver.

The verified `ra_ap_hir_ty` 0.0.358 dependency set includes `ra-ap-rustc_next_trait_solver`, `ra-ap-rustc_type_ir`, `ra-ap-rustc_abi`, `ra-ap-rustc_ast_ir`, and `ra-ap-rustc_pattern_analysis`, alongside Salsa 0.28.5.

The Rust Compiler Development Guide explains the division:

- `rustc_type_ir` provides common type-system abstractions, including the `Interner` interface and generic representations of type-system entities.
- `rustc_next_trait_solver` implements solver logic over those abstractions.
- rustc and rust-analyzer each supply their own concrete type representations, interners, and query-backed environments.
- rust-analyzer uses `DbInterner` and Salsa-backed queries rather than rustc's `TyCtxt`.
- Some other analysis machinery, including parts of inference and coercion handling, remains separate.

This sharing reduces duplication of sophisticated trait-solving logic without making rust-analyzer a thin wrapper around rustc. It remains a distinct compiler frontend with different incremental and incomplete-code requirements.

Consequential limitation: A shared solver does not imply identical end-to-end type-checking behavior. Inputs, inference contexts, compiler frontend completeness, unstable language features, and diagnostics can still differ.

Sources:

- `https://rustc-dev-guide.rust-lang.org/solve/sharing-crates-with-rust-analyzer.html`
- `https://docs.rs/crate/ra_ap_hir_ty/0.0.358`

[image](https://www.google.com/s2/favicons?domain=https://rustc-dev-guide.rust-lang.org\&sz=32)

Rust Compiler Development Guide+1

### 5.6 Macro expansion and source provenance

Macro support spans several independently useful components.

| Component                                             | Responsibility                                                |
| ----------------------------------------------------- | ------------------------------------------------------------- |
| `ra_ap_tt`                                            | Token-tree representation                                     |
| `ra_ap_mbe`                                           | Declarative macro (`macro_rules!`) parsing and expansion      |
| `ra_ap_hir_expand`                                    | Integration of expansions with the incremental semantic model |
| `ra_ap_span`                                          | Spans, syntax contexts, and macro hygiene infrastructure      |
| `ra_ap_proc_macro_api`                                | Client-side procedural macro execution interface              |
| `ra_ap_proc_macro_srv` and `ra_ap_proc_macro_srv_cli` | Separate process for loading and executing procedural macros  |

The language frontend can interpret declarations and expressions originating from function-like macros, derive macros, attribute macros, and built-in macro mechanisms, subject to the support and configuration of the particular macro.

Procedural macros are executed in a separate process to isolate accidental panics and crashes from the main analysis process. This is a resilience boundary, not a security sandbox.

Macro-generated code creates important source-identity complications:

- An expansion node may have no direct one-to-one physical source counterpart.
- Multiple generated constructs may correspond to one macro invocation.
- Hygiene changes how identifiers are resolved.
- A reference inside an expansion may require mapping through several layers to produce a useful source location.
- A macro can depend on environment variables, generated files, or nondeterministic external behavior.

`Semantics` offers expansion-aware traversal, original-source ranges, and speculative expansion methods. These support navigation, completion, and source edits around macro-generated code without requiring each consumer to invent a separate origin-tracking system.

Limitation: Macro expansion coverage depends on valid build metadata, proc-macro availability, expansion limits, and supported compiler behavior. A fully resolved view of arbitrary generated Rust cannot be assumed.

Sources:

- `https://rust-analyzer.github.io/book/contributing/architecture.html`
- `https://docs.rs/ra_ap_hir/latest/ra_ap_hir/struct.Semantics.html`

[image](https://www.google.com/s2/favicons?domain=https://rust-analyzer.github.io\&sz=32)

rust-analyzer+1

## 6. IDE intelligence and program-aware transformations

The `ra_ap_ide` crate consolidates compiler-derived information into operations expressed in terms of source files, offsets, ranges, and edits. This is the principal API for consumers that need code intelligence without directly manipulating HIR entities.

### 6.1 Navigation and semantic cross-references

| Capability                      | Main API                                             | Distinctive behavior                                               |
| ------------------------------- | ---------------------------------------------------- | ------------------------------------------------------------------ |
| Definition navigation           | `goto_definition`, `goto_declaration`                | Resolves identifiers to their semantic targets                     |
| Type navigation                 | `goto_type_definition`                               | Navigates from a value or expression to its type definition        |
| Trait implementation navigation | `goto_implementation`                                | Connects traits, implementations, and implementation targets       |
| Reference search                | `find_all_refs`                                      | Searches uses of resolved definitions rather than matching text    |
| Call hierarchy                  | `call_hierarchy`, `incoming_calls`, `outgoing_calls` | Builds caller/callee relationships for supported call sites        |
| Symbol discovery                | `file_structure`, `symbol_search`                    | Supports document outlines and fuzzy workspace symbol search       |
| Semantic identity               | `moniker`                                            | Exposes symbol-identity information useful for cross-file indexing |
| Module structure                | `parent_module`, `child_modules`                     | Navigates the logical Rust module structure                        |

Reference search covers more than plain identifier occurrences, including references in patterns and type positions, constructors, and supported macro expansions.

A useful architectural distinction is on-demand semantic resolution versus precomputed cross-reference indexing.

The native IDE engine is optimized for answering interactive questions about a specific source position. Large, persistent code-indexing systems often require an additional batch-indexing or materialization step; rust-analyzer does not make every semantic relationship available as a permanently maintained external graph.

Source: `https://docs.rs/ra_ap_ide/latest/ra_ap_ide/struct.Analysis.html`&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+1



### 6.2 Contextual completion and term synthesis

Completion is substantially more than symbol-name matching. `ra_ap_ide_completion` combines name resolution, expected types, scope analysis, import discovery, and expression transformations.

Its documented capabilities include:

- Member and method candidates constrained by receiver types and applicable traits.
- Unimported-item and trait-method discovery with import insertion.
- Contextual import paths, attributes, derive macros, and keyword completion.
- Completion of function-call syntax and generic arguments.
- Type-directed suggestions, including conversions and dereferencing.
- Postfix transformations that wrap an expression in a new construct.
- Custom prefix/postfix snippets with required-symbol constraints.
- Configurable import grouping, path prefixing, and exclusion behavior.

Some less obvious options are `completion.autoAwait.enable`, which enables completion forms for methods on awaited future values, and `completion.termSearch.enable`.

Term search is a particularly interesting advanced mechanism. Rather than merely listing already-visible names, it searches for expressions that satisfy an expected type, potentially combining available expressions and calls. It is bounded by configurable work budgets (`completion.termSearch.fuel` and `assist.termSearch.fuel`).

Completion term search is disabled by default in the verified configuration. It should be understood as bounded, heuristic expression synthesis—not an exhaustive proof that no suitable expression exists.

Source: `https://rust-analyzer.github.io/book/configuration.html`&#x20;

[image](https://www.google.com/s2/favicons?domain=https://rust-analyzer.github.io\&sz=32)

rust-analyzer+1



### 6.3 Semantic highlighting, hovers, and inlay hints

These features expose structured program information without requiring a full navigation operation.

Semantic highlighting assigns token categories and modifiers according to resolved meaning. For example, a name is classified differently depending on whether it resolves to a trait, module, method, enum, or variable. The client owns the presentation colors.

Hovers and signature help provide inferred types, declarations, documentation, parameter information, and related actions.

Inlay hints can expose:

- Local-variable, chained-expression, and closure-return types.
- Function-argument and const-generic parameter names.
- Generic type and lifetime parameters.
- Elided lifetimes and compiler-inserted reborrows.

Related-code highlighting is more specialized than ordinary references. It can identify closure captures, async yield points, return and exit points, relevant loop breaks, and other syntax/semantic relationships.

These features are useful as structured semantic annotations for downstream consumers. Unlike source text, their interpretation depends on the configured semantic model and can change when compilation context changes.

Sources:

- `https://rust-analyzer.github.io/book/features.html`
- `https://docs.rs/ra_ap_ide/latest/ra_ap_ide/struct.Analysis.html`

[image](https://www.google.com/s2/favicons?domain=https://rust-analyzer.github.io\&sz=32)

rust-analyzer+2

### 6.4 Assists, refactoring, and source edits

`ra_ap_ide_assists` implements contextual transformations that combine AST construction with semantic information.

Important functional families include:

| Family                      | Representative transformations                                      |
| --------------------------- | ------------------------------------------------------------------- |
| Import and visibility       | Add missing imports, reorganize use trees, adjust visibility        |
| Implementation generation   | Generate trait members, implementations, constructors, conversions  |
| Expression and control flow | Fill match arms, convert conditional expressions, simplify patterns |
| Extraction                  | Extract variables, constants, functions, or modules                 |
| Inlining                    | Inline variables and supported function calls                       |
| Type elaboration            | Add explicit types, convert syntax to more explicit forms           |
| Signature changes           | Adjust parameters and update supported call sites                   |
| Structural editing          | Reorder items, adjust attributes, transform syntax constructs       |
| Semantic rename             | Rename a definition and its resolved references                     |

`Analysis::assists_with_fixes`, `Analysis::rename`, `Analysis::prepare_rename`, and `Analysis::will_rename_file` expose these operations through the IDE interface.

The internal editing machinery can represent changes across multiple files, including source edits and filesystem operations. This is materially different from a text-replacement helper: the transformation may need to update imports, references, module declarations, and surrounding syntax coherently.

Nevertheless, a refactoring is not a formal semantic-equivalence proof. Complex macro expansions, resolution ambiguity, build configurations, or unsupported cases can limit a transformation. Verification with the compiler remains necessary for consequential changes.

A particularly advanced case is rust-analyzer's special rename handling for `self` parameters, where changing method-style syntax can require rewriting calling expressions.

Sources:

- `https://rust-analyzer.github.io/book/assists.html`
- `https://rust-analyzer.github.io/book/features.html`

[image](https://www.google.com/s2/favicons?domain=https://rust-analyzer.github.io\&sz=32)

rust-analyzer+1

### 6.5 Structural Search and Replace (SSR)

`ra_ap_ide_ssr` provides a native structural transformation engine, exposed through `Analysis::structural_search_replace`, the editor command, and CLI facilities.

Its rule language uses `search_pattern ==>> replacement_pattern`, with named placeholders such as `$receiver` and `$argument`.

Important properties:

- Matches Rust syntax structure rather than raw character sequences.
- Placeholders can capture expressions, types, paths, patterns, or items.
- Repeated placeholders can participate in structural matching.
- Placeholder constraints include literal matching and negation.
- Can restrict the operation to a selection rather than an entire workspace.
- Uses semantic name resolution when matching paths.
- Can recognize equivalent spelling through qualification or imports.
- Supports some equivalence between associated-function syntax and method-call syntax.
- Can account for implicit autoderef/autoref when transforming certain method-call patterns.

For example, a rule involving a qualified method path can match a method-call expression that resolves to the same function, even where the source spelling differs.

That combination of structural placeholders and semantic path resolution makes SSR more powerful than regex-based replacement, and in some cases more precise than syntax-only pattern engines.

Its limits are important: SSR does not automatically prove behavioral equivalence, and macro-generated or syntactically unusual source can complicate matching and replacement.

Source: `https://rust-analyzer.github.io/book/features.html` — Structural Search and Replace section.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://rust-analyzer.github.io\&sz=32)

rust-analyzer



### 6.6 Test discovery and executable targets

rust-analyzer also derives operational information from the semantic model and project metadata.

`Analysis` exposes `runnables`, `discover_test_roots`, `discover_tests_in_crate`, `discover_tests_in_file`, and `related_tests`.

These support:

- Discovery of test and benchmark functions.
- Run/debug targets for relevant source positions.
- Hierarchical test discovery.
- Navigation from production code to tests that reference it.
- Integration of discovered targets with editor commands.

This is not a test runner in the same sense as Cargo. rust-analyzer identifies execution targets and constructs relevant runnable metadata; external tools perform the actual compilation, execution, or debugging.

Source: `https://docs.rs/ra_ap_ide/latest/ra_ap_ide/struct.Analysis.html`&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust



## 7. Diagnostics, MIR, interpretation, and compiler boundaries

### 7.1 Two distinct diagnostic engines

rust-analyzer integrates two materially different sources of diagnostics.

| Native rust-analyzer analysis                                                     | External compiler/check integration                          |
| --------------------------------------------------------------------------------- | ------------------------------------------------------------ |
| Uses rust-analyzer's parser, name resolution, type inference, and semantic checks | Uses `cargo check`, rustc, or Clippy                         |
| Supports responsive feedback on incomplete code                                   | Reflects actual compiler and lint behavior                   |
| Can run without fully compiling the project                                       | Requires the relevant build/check command                    |
| May be incomplete or differ in lint handling                                      | Better basis for compiler acceptance and lint compliance     |
| Exposed through `syntax_diagnostics`, `semantic_diagnostics`, `full_diagnostics`  | Coordinated by the language server's flycheck infrastructure |

The manual explicitly notes that some native diagnostics do not yet honor all `#[allow]` and `#[deny]` attributes. Experimental diagnostics can also have more false positives and are separately configurable.

The distinction matters for code-quality systems: native diagnostics are valuable for responsive analysis, but the absence of native errors should not be interpreted as proof of successful compilation.

Source: `https://rust-analyzer.github.io/book/diagnostics.html`&#x20;

[image](https://www.google.com/s2/favicons?domain=https://rust-analyzer.github.io\&sz=32)

rust-analyzer



### 7.2 MIR and interpretation

Although rust-analyzer is primarily an IDE frontend, it contains limited lower-level semantic machinery.

Documented surfaces include:

- `hir::mir` and related MIR-building components.
- `Analysis::view_mir` for rendering an intermediate representation of supported functions.
- `Analysis::interpret_function` for evaluating supported code without compiling it.
- `Analysis::get_recursive_memory_layout` and `Type::layout` for inspecting type layouts.
- Constant-value and failed-obligation inspection.
- Inference-derived adjustments and related expression information.

These capabilities are useful for investigating how rust-analyzer interprets a program, but should not be equated with rustc's complete MIR pipeline, runtime execution semantics, or comprehensive const evaluator.

The LSP documentation explicitly labels internal function interpretation highly experimental.

Sources:

- `https://rust-analyzer.github.io/book/contributing/lsp-extensions.html`
- `https://docs.rs/ra_ap_hir/latest/ra_ap_hir/`
- `https://rust-analyzer.github.io/book/features.html`

[image](https://www.google.com/s2/favicons?domain=https://rust-analyzer.github.io\&sz=32)

rust-analyzer+2

### 7.3 Borrow checking: an important 2026 change

rust-analyzer does not currently provide a complete independent borrow checker.

The August 10, 2026 release explicitly removed its previously broken and incomplete borrow checker.

This prevents a common but misleading conclusion that rust-analyzer's HIR, MIR, and type inference form a complete replacement for rustc's semantic validation pipeline.

For ownership, borrow-lifetime, and compiler-correctness decisions, rustc remains the authoritative checker.

Source: `https://rust-analyzer.github.io/thisweek/2026/08/10/changelog-340.html`&#x20;

[image](https://www.google.com/s2/favicons?domain=https://rust-analyzer.github.io\&sz=32)

rust-analyzer.github.io



## 8. Integration surfaces: native crates, LSP, CLI, and semantic indexes

### 8.1 Choosing the appropriate integration boundary

rust-analyzer exposes several ways to reuse its capabilities, with different coupling and lifecycle costs.

| Integration                      | Access provided                                                   | Main tradeoff                                                          |
| -------------------------------- | ----------------------------------------------------------------- | ---------------------------------------------------------------------- |
| `ra_ap_syntax`                   | Lossless syntax trees, parsing, AST editing                       | No resolved types, imports, or references                              |
| `ra_ap_hir`                      | Resolved entities, types, methods, scopes, macros                 | Requires initialized semantic database and crate graph                 |
| `ra_ap_ide`                      | Navigation, references, diagnostics, assists, completions, search | Unstable native APIs; editor-oriented data structures                  |
| `ra_ap_load-cargo`               | Static Cargo workspace loading and analysis initialization        | No automatic ongoing synchronization                                   |
| `rust-analyzer` LSP server       | Full IDE service with ongoing workspace management                | Protocol translation, process lifecycle, client capability negotiation |
| `rust-analyzer scip` / `lsif`    | Materialized semantic index for offline consumption               | Batch-oriented output, potentially stale after edits                   |
| Lower-level `ra_ap_hir_*` crates | Internal inference, definitions, macro machinery                  | Strongest coupling to implementation details                           |

The distinction is not simply library versus executable. It is whether a consumer needs native Rust semantics, an editor-oriented query API, a live managed service, or a reusable snapshot index.

Official API documentation: `https://rust-lang.github.io/rust-analyzer/ide/`&#x20;

[image](https://www.google.com/s2/favicons?domain=https://github.com\&sz=32)

GitHub



### 8.2 LSP as a managed semantic service

The language server provides facilities beyond the `ide` façade:

- Project discovery, reload, and configuration updates.
- File watchers and synchronized editor overlays.
- Background analysis and cache priming.
- Conversion between internal file/range types and LSP messages.
- Cancellation and asynchronous request orchestration.
- Flycheck integration and compiler diagnostics.
- External formatting through rustfmt.
- Editor notifications and progress/status reporting.

The native IDE layer is deliberately independent of JSON serialization and LSP-specific types. The server converts internal results into protocol representations.

This lets other clients implement custom presentations or protocols, but those clients must take responsibility for the corresponding lifecycle if they bypass the server.

### 8.3 Rust-specific LSP extensions

The upstream LSP extension documentation exposes numerous operations beyond standard navigation and completion.

| Extension family      | Representative operations                                               |
| --------------------- | ----------------------------------------------------------------------- |
| Inspection            | `rust-analyzer/viewSyntaxTree`, `/viewHir`, `/viewMir`, `/viewItemTree` |
| Project inspection    | `rust-analyzer/viewCrateGraph`, `/viewFileText`                         |
| Macro analysis        | `rust-analyzer/expandMacro`, `/rebuildProcMacros`                       |
| Semantic debugging    | `rust-analyzer/getFailedObligations`, `/interpretFunction`              |
| Execution and testing | `experimental/runnables`, test-discovery and related-test requests      |
| Editing               | Structural search/replace, join-lines, move-item, snippet-aware edits   |
| Server coordination   | Workspace reload, status notifications, flycheck control                |
| Presentation          | Enhanced hover actions, annotations, inlay-hint resolution              |

A client needs to distinguish standard LSP methods from rust-analyzer-specific extensions. Some additional capabilities are negotiated through the protocol's `experimental` fields and may not be supported by every editor or LSP client.

The extension documentation itself is described as best-effort, so the server source remains the authority when exact request/response compatibility matters.

Source: `https://rust-analyzer.github.io/book/contributing/lsp-extensions.html`&#x20;

[image](https://www.google.com/s2/favicons?domain=https://rust-analyzer.github.io\&sz=32)

rust-analyzer+1



### 8.4 Native SCIP and LSIF code indexing

A particularly important capability outside ordinary editor usage is native semantic index generation.

rust-analyzer's CLI includes:

- `rust-analyzer scip <path>`
- `rust-analyzer lsif <path>`

The SCIP command accepts an output path (default `index.scip`), a Cargo configuration path, exclusion of vendored libraries, and a cache-priming worker-thread count.

SCIP provides a materialized representation of semantic symbol occurrences and relationships for use by indexing and code-navigation systems. This offers a way to reuse rust-analyzer's semantic work without maintaining a permanent language-server session for every lookup.

The important architectural distinctions are:

- Live `Analysis` queries: Reflect a maintained current analysis revision, subject to cancellation and configuration.
- SCIP/LSIF indexes: Capture the analyzed state at indexing time and require regeneration or additional change tracking to remain current.
- Direct HIR extraction: Exposes a richer semantic vocabulary than is necessarily represented in a standard index.

A semantic index can eliminate the need to reimplement name resolution and cross-reference discovery. It does not automatically provide arbitrary HIR queries, full type inference results, or all relationships needed for a custom graph representation.

The October 19, 2026 changelog includes a fix for SCIP monikers in crates with multiple targets—evidence that crate-instance identity remains a meaningful concern even at the indexing boundary.

Stability warning: The rust-analyzer CLI source explicitly states that subcommands and their flags provide no stability guarantees. Thus, a published SCIP format and the stability of the command used to generate it are separate considerations.

Sources:

- `https://rust-lang.github.io/rust-analyzer/src/rust_analyzer/cli/flags.rs.html`
- `https://rust-analyzer.github.io/thisweek/2026/10/19/changelog-350.html`

[image](https://www.google.com/s2/favicons?domain=https://rust-lang.github.io\&sz=32)

source+1

### 8.5 Batch analysis and diagnostic CLI utilities

The binary includes additional commands useful for programmatic analysis:

| Command                    | Purpose                                                          |
| -------------------------- | ---------------------------------------------------------------- |
| `parse`                    | Parse Rust source supplied on standard input                     |
| `symbols`                  | Extract source symbols                                           |
| `highlight`                | Produce highlighted HTML from source                             |
| `analysis-stats`           | Perform batch semantic analysis and report statistics            |
| `diagnostics`              | Run diagnostic analysis                                          |
| `unresolved-references`    | Inspect unresolved references                                    |
| `prime-caches`             | Populate analysis caches                                         |
| `ssr`                      | Apply structural search-and-replace rules                        |
| `search`                   | Match structural patterns                                        |
| `run-tests`, `rustc-tests` | Exercise the internal MIR interpreter in supported test contexts |
| `scip`, `lsif`             | Generate semantic indexes                                        |

`analysis-stats` is especially useful because its flags allow different stages to be enabled or skipped, including lowering, inference, MIR-related work, data-layout computation, and constant evaluation. It also supports parallel analysis and dependency inclusion.

This creates a native way to characterize different analysis workloads without building a custom measurement harness around each internal query.

Source: `https://rust-lang.github.io/rust-analyzer/src/rust_analyzer/cli/flags.rs.html`&#x20;

[image](https://www.google.com/s2/favicons?domain=https://rust-lang.github.io\&sz=32)

source



## 9. Performance architecture, configuration, and resource management

### 9.1 Main performance mechanisms

rust-analyzer's incremental architecture creates several different opportunities for reducing work:

| Mechanism                       | What it avoids or reduces                           | Qualification                                               |
| ------------------------------- | --------------------------------------------------- | ----------------------------------------------------------- |
| Incremental reparsing           | Rebuilding unchanged syntax subtrees                | Depends on edit and grammar context                         |
| `ItemTree` separation           | Recomputing item-level facts after body-only edits  | Signature and import changes propagate further              |
| Salsa memoization               | Repeating unchanged semantic computations           | Cache effectiveness depends on dependency structure         |
| Demand-driven queries           | Analyzing irrelevant program regions                | Broad reference searches still require substantial work     |
| Source-root durability          | Frequent invalidation of mostly stable dependencies | Incorrect durability assumptions are unsafe                 |
| Cache priming                   | First-query latency after loading                   | Moves work to startup; does not make it free                |
| LRU and query-capacity controls | Unbounded retention of selected cached results      | Eviction can increase future recomputation                  |
| Separate proc-macro processes   | Main-process crashes from accidental macro failures | Adds process and communication overhead                     |
| Parallel analysis               | Serial execution of independent computations        | Benefits depend on query contention and dependency topology |

The benefit of an incremental semantic engine is workload-specific. It tends to be most useful when many queries reuse the same project model across small changes. One-time, broad semantic extraction may behave quite differently from interactive editing.

### 9.2 Cache and concurrency interfaces

Relevant controls include:

- `rust-analyzer.cachePriming.enable`
- `rust-analyzer.cachePriming.numThreads`
- `rust-analyzer.lru.capacity`
- `rust-analyzer.lru.query.capacities`
- `rust-analyzer.numThreads`
- `rust-analyzer.procMacro.processes`
- `AnalysisHost::update_lru_capacity`
- `AnalysisHost::update_lru_capacities`
- `AnalysisHost::trigger_garbage_collection`

The documented default syntax-tree LRU capacity is 128. Query-specific capacities can be configured independently.

`Analysis` snapshots are `Send` but not `Sync` in the verified crate documentation. Concurrent analysis therefore involves separate snapshots or suitably orchestrated ownership, not unrestricted concurrent access through a shared `Analysis` reference.

Source: `https://rust-analyzer.github.io/book/configuration.html`&#x20;

[image](https://www.google.com/s2/favicons?domain=https://rust-analyzer.github.io\&sz=32)

rust-analyzer+1



### 9.3 Profiling and introspection

rust-analyzer includes native observation facilities:

- `RA_PROFILE`: Hierarchical profiling with span filtering and depth/latency controls.
- `RA_PROFILE_JSON`: Machine-readable profiling output for selected spans.
- `RA_COUNT=1`: Counts instances of selected internal structures.
- `RA_LOG`: Structured logging and configuration/protocol diagnostics.
- `analysis-stats`: Repeatable analysis-stage measurements and memory-related inspection.
- Server status and memory-inspection operations.
- Debug views of syntax trees, item trees, HIR, MIR, and the crate graph.

These mechanisms can distinguish several sources of delay: initial workspace loading, build-script execution, procedural macro expansion, cache priming, type inference, and particular IDE queries.

Performance caveat: The official sources establish the mechanisms, but do not establish a universal throughput or latency advantage over alternative Rust analysis tools. Such comparisons require workload-specific measurement.

Source: `https://rust-analyzer.github.io/book/contributing/`&#x20;

[image](https://www.google.com/s2/favicons?domain=https://rust-analyzer.github.io\&sz=32)

rust-analyzer



## 10. Configuration fidelity, security, and important limits

### 10.1 The analyzed program is a configuration-specific projection

A semantic result is meaningful only relative to the configuration under which it was derived.

Important dimensions include:

- Selected compilation targets.
- Cargo features and conditional compilation.
- Workspace members and dependency versions.
- Test versus production configurations.
- Environment variables and build-script outputs.
- Generated code availability.
- Procedural macro expansion.
- Compiler and standard-library source compatibility.

The language server's `cargo.features`, `cargo.target`, `cargo.allTargets`, `cargo.buildScripts.enable`, `procMacro.enable`, and related settings directly influence semantic completeness.

The default configuration enables build-script execution and procedural macros, while other settings permit restriction or substitution.

A useful consequence is that one semantic index should not automatically be interpreted as an exhaustive representation of all possible Rust configurations. Multiple target and feature contexts may yield distinct definitions and references.

Source: `https://rust-analyzer.github.io/book/configuration.html`&#x20;

[image](https://www.google.com/s2/favicons?domain=https://rust-analyzer.github.io\&sz=32)

rust-analyzer



### 10.2 Security model

The official security documentation explicitly assumes that project code is trusted.

Potential code-execution paths include:

- Build scripts and procedural macros, executed by default.
- Cargo configuration that overrides tool executables.
- Toolchain configuration that changes the selected compiler.
- Editor/project configuration that supplies executable paths.

The upstream documentation also states that the syntax-tree library uses substantial `unsafe` code and has not been fully audited for memory safety.

Running procedural macros in a separate process improves crash isolation, but does not make loading an untrusted repository safe.

For analysis of untrusted source, avoiding build-script and procedural-macro execution can reduce risk, but project loading and other tool invocations must also be considered. The syntax-only boundary has materially fewer execution dependencies than full Cargo-backed semantic analysis.

Source: `https://rust-analyzer.github.io/book/security.html`&#x20;

[image](https://www.google.com/s2/favicons?domain=https://rust-analyzer.github.io\&sz=32)

rust-analyzer



### 10.3 API and version stability

rust-analyzer explicitly documents the internal `ide` API as unstable, while relying on the standard LSP interface as a more durable protocol boundary.

Other consequences:

- `ra_ap_*` packages are published for programmatic use, not as a general promise of stable embedded APIs.
- Internal database, HIR, and type-system APIs can change substantially between releases.
- Matching `ra_ap_*` versions matter because dependencies commonly require exact package versions.
- Historical architecture documentation may mention obsolete mechanisms. For example, descriptions centered on Chalk must be reconciled with the current shared next-trait-solver implementation.
- The language server and its reusable crates need not carry the same version number.
- rust-analyzer's command-line subcommands are explicitly unstable.
- The `rust-project.json` format is considered a de facto compatibility boundary, but is not equivalent to a rigorously versioned standard.

For published binaries, the official installation documentation also notes that rust-analyzer expects compatible standard-library source; the latest stable standard-library source is the officially supported baseline.

Sources:

- `https://rust-analyzer.github.io/book/contributing/architecture.html`
- `https://rust-analyzer.github.io/book/installation.html`

[image](https://www.google.com/s2/favicons?domain=https://rust-analyzer.github.io\&sz=32)

rust-analyzer+1

## 11. Architectural synthesis: where the library changes what is practical

The most consequential capabilities arise from combinations across layers.

| Capability combination                        | General architectural possibility                                          | Boundary to preserve                                   |
| --------------------------------------------- | -------------------------------------------------------------------------- | ------------------------------------------------------ |
| Lossless AST + syntax editor                  | Structural extraction and rewriting without compiler setup                 | Syntax equivalence is not semantic equivalence         |
| HIR + `Semantics` + source provenance         | Resolved symbol/relationship extraction with macro-aware locations         | Context-specific identities and incomplete expansions  |
| Incremental database + analysis snapshots     | Continuously refreshed semantic services                                   | Cancellation, synchronization, and memory management   |
| `project_model` + `load-cargo` + HIR          | Batch semantic analysis with minimal custom workspace loading              | Static snapshots and build execution concerns          |
| `ide` + reference search + rename             | Program-aware navigation and source transformations                        | Unstable APIs and imperfect refactoring coverage       |
| SSR + semantic resolution                     | Cross-file structural transformation with name-aware matching              | Matching and edits require validation                  |
| SCIP + live semantic queries                  | Persistent cross-reference index supplemented by richer on-demand analysis | Separate index freshness and semantic query lifecycles |
| Shared rustc solver + rust-analyzer inference | Compiler-aligned trait reasoning in an incremental frontend                | Still not full rustc validation                        |
| Native diagnostics + Cargo/Clippy             | Responsive feedback combined with authoritative compiler checks            | Different completeness and lint behavior               |

Three conclusions are especially important.

First, rust-analyzer is more than a parser or symbol indexer. Its resolved HIR, source provenance, and type-system interfaces can provide semantic relationships that would otherwise require substantial custom compiler-front-end functionality.

Second, the boundary chosen matters more than the number of exposed APIs. A consumer that only needs code navigation may gain little from directly embedding the internal type engine. Conversely, a consumer that needs arbitrary resolved trait, expression, or macro relationships may find LSP or SCIP too narrow.

Third, incrementality and semantic correctness are separate concerns. rust-analyzer's architecture is designed to preserve responsive semantic analysis as source changes, but it does not eliminate the need to represent configuration context, account for incomplete analysis, or verify program behavior with rustc.

## 12. Primary-source reference map

All URLs below are direct, standalone source addresses.

| Area                      | Primary source                                                                       |
| ------------------------- | ------------------------------------------------------------------------------------ |
| Current release           | `https://rust-analyzer.github.io/thisweek/2026/10/26/changelog-351.html`             |
| Overall architecture      | `https://rust-analyzer.github.io/book/contributing/architecture.html`                |
| Native IDE API            | `https://docs.rs/ra_ap_ide/latest/ra_ap_ide/`                                        |
| Syntax API                | `https://docs.rs/ra_ap_syntax/latest/ra_ap_syntax/`                                  |
| Parser                    | `https://docs.rs/ra_ap_parser/latest/ra_ap_parser/`                                  |
| HIR                       | `https://docs.rs/ra_ap_hir/latest/ra_ap_hir/`                                        |
| Semantic mapping          | `https://docs.rs/ra_ap_hir/latest/ra_ap_hir/struct.Semantics.html`                   |
| Type introspection        | `https://docs.rs/ra_ap_hir/latest/ra_ap_hir/struct.Type.html`                        |
| Shared rustc trait solver | `https://rustc-dev-guide.rust-lang.org/solve/sharing-crates-with-rust-analyzer.html` |
| Workspace loading         | `https://docs.rs/ra_ap_load-cargo/latest/ra_ap_load_cargo/`                          |
| Non-Cargo project model   | `https://rust-analyzer.github.io/book/non_cargo_based_projects.html`                 |
| IDE features              | `https://rust-analyzer.github.io/book/features.html`                                 |
| Assists and refactorings  | `https://rust-analyzer.github.io/book/assists.html`                                  |
| Diagnostics               | `https://rust-analyzer.github.io/book/diagnostics.html`                              |
| Configuration             | `https://rust-analyzer.github.io/book/configuration.html`                            |
| LSP extensions            | `https://rust-analyzer.github.io/book/contributing/lsp-extensions.html`              |
| CLI and SCIP flags        | `https://rust-lang.github.io/rust-analyzer/src/rust_analyzer/cli/flags.rs.html`      |
| Profiling                 | `https://rust-analyzer.github.io/book/contributing/`                                 |
| Security                  | `https://rust-analyzer.github.io/book/security.html`                                 |
| Source repository         | `https://github.com/rust-lang/rust-analyzer`                                         |

Overall assessment: rust-analyzer's distinctive strength is its combination of lossless syntax, crate-aware semantics, a modern shared trait solver, macro-aware source mapping, and demand-driven incremental computation. For sophisticated Rust code intelligence, these capabilities offer substantially more than conventional AST tooling, while its multiple integration boundaries allow consumers to select only as much compiler machinery and lifecycle responsibility as their requirements justify.