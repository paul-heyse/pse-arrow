# ra_ap_hir — Comprehensive Rust Library Reference

Target version: 0.0.358 (published October 12, 2026) | Research date: October 29, 2026 | Ecosystem: rust-analyzer

Package: `ra_ap_hir`\
Upstream crate: `hir` in `rust-lang/rust-analyzer`\
Purpose: High-level, crate-aware semantic analysis of Rust source code, exposing resolved definitions, inferred types, scopes, traits, implementations, macro expansions, and compiler-derived semantic information.

Primary documentation:

- [https://docs.rs/ra_ap_hir/0.0.358/ra_ap_hir/](https://docs.rs/ra_ap_hir/0.0.358/ra_ap_hir/?utm_source=chatgpt.com)
- [https://github.com/rust-lang/rust-analyzer/tree/master/crates/hir](https://github.com/rust-lang/rust-analyzer/tree/master/crates/hir?utm_source=chatgpt.com)
- [https://rust-analyzer.github.io/book/contributing/architecture.html](https://rust-analyzer.github.io/book/contributing/architecture.html?utm_source=chatgpt.com)

Version 0.0.358 is the latest published release identified in the crates.io documentation index. It depends on correspondingly versioned `ra_ap_hir_def`, `ra_ap_hir_expand`, `ra_ap_hir_ty`, `ra_ap_base_db`, and other rust-analyzer components.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Docs.rs+1



## 1. Architectural orientation

### 1.1 What ra_ap_hir provides

`ra_ap_hir` is rust-analyzer's principal semantic abstraction layer between Rust syntax and higher-level language intelligence.

Where `ra_ap_syntax` represents the grammatical structure of source files, `ra_ap_hir` represents the Rust entities and relationships that exist after contextual interpretation: name resolution, conditional compilation, macro expansion, type inference, and trait resolution.

The distinction is fundamental:

- Syntax identity: A syntax tree describes source text independently of its complete compilation environment.
- Semantic identity: HIR represents Rust entities interpreted within a particular crate instance, including its edition, configuration flags, dependencies, and macro environment.
- Context-dependent interpretation: The same source may participate in multiple crate instances or compilation configurations, producing different semantic interpretations.
- Incremental analysis: Semantic results originate in rust-analyzer's query infrastructure; clients access those results through an object-oriented façade instead of directly manipulating the lower-level inference and resolution representations.

The result is a way to interrogate Rust code in terms of its actual language semantics rather than reconstructing those semantics from tokens, syntax nodes, names, and heuristics.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://rust-lang.github.io\&sz=32)

Rust+1



### 1.2 Capability map

| Capability family          | Principal interfaces                                       | Semantic information available                                                                     |
| -------------------------- | ---------------------------------------------------------- | -------------------------------------------------------------------------------------------------- |
| Syntax-to-semantics        | `Semantics`, `SemanticsImpl`, `HasSource`                  | Source nodes to definitions, references, inferred types, expansion-aware source locations          |
| Crate and module modeling  | `Crate`, `Module`, `ModuleDef`, `DefMap`                   | Crate graphs, modules, declarations, namespaces, visibility, imports, conditional compilation      |
| Symbol resolution          | `PathResolution`, `SemanticsScope`, `ScopeDef`             | Resolved references, lexical scopes, accessible names, namespace-specific resolution               |
| Type analysis              | `Type`, `TypeInfo`, `Callable`, `GenericSubstitution`      | Inferred types, generics, callable signatures, autoderef, coercion compatibility, associated types |
| Traits and implementations | `Trait`, `Impl`, `TraitRef`, `TraitImpls`                  | Trait relationships, method candidates, associated items, implementation applicability             |
| Macros and hygiene         | `Macro`, `MacroCallId`, `HirFileId`, expansion methods     | Expanded syntax, macro-definition resolution, virtual files, source provenance                     |
| Definitions and structure  | `Function`, `Struct`, `Enum`, `Field`, `TypeAlias`, `Body` | Semantic declarations, parameters, fields, variants, signatures, enclosing relationships           |
| Semantic diagnostics       | `diagnostics`, `Semantics`                                 | Compiler-derived analysis findings, failed obligations, unsafe operations                          |
| MIR and evaluation         | `mir`, `EvaluatedConst`, `Layout`                          | Intermediate representation, limited evaluation, layout and drop-related information               |
| Discovery and generation   | `import_map`, `symbols`, `term_search`, `HirDisplay`       | Importable definitions, symbol discovery, term synthesis, contextual type rendering                |

These families support much more than code navigation. Their combination exposes substantial building blocks for semantic code indexing, structural queries, dependency analysis, refactoring intelligence, type-aware transformations, and advanced language tooling.

### 1.3 Position within the rust-analyzer architecture

Consumer-facing analysis

IDE features, semantic queries, indexing, analysis tools

ra_ap_hir — semantic façade

Semantics · Type · Module · Trait · Function · Impl

hir-def

Definitions, scopes, name resolution

hir-ty

Inference, traits, type operations

hir-expand

Expansion, hygiene, macro files

Supporting infrastructure

Salsa queries · base-db / crate graph · syntax trees · source inputs

The distinction between the façade and its underlying engines matters architecturally. The `hir-*` implementation crates use IDs, arenas, and query-oriented representations intended for efficient incremental computation. `ra_ap_hir` provides more cohesive semantic objects and methods, insulating consumers from much of this internal structure.

This is a soft internal API boundary, not a separately stabilized compiler interface. The published `ra_ap_*` crates follow rust-analyzer's rapidly evolving development rather than conventional stable-library API guarantees.

Source: [https://rust-analyzer.github.io/book/contributing/architecture.html](https://rust-analyzer.github.io/book/contributing/architecture.html?utm_source=chatgpt.com)

## 2. Database, identity, and incremental computation

### 2.1 The database is the semantic universe

Nearly every HIR operation takes `&dyn HirDatabase`. The database provides the configuration and derived state against which a semantic question is evaluated.

`ra_ap_hir` does not independently load Cargo projects or discover source files. Its inputs ultimately come from the surrounding rust-analyzer infrastructure:

| Input or mechanism | Responsibility                                                                    |
| ------------------ | --------------------------------------------------------------------------------- |
| `ra_ap_base_db`    | Source-file identities, crate graph, crate configuration, incremental input state |
| `ra_ap_cfg`        | Conditional-compilation expressions and configuration                             |
| `ra_ap_syntax`     | Lossless syntax trees, typed AST nodes, source ranges                             |
| `ra_ap_hir_expand` | Macro expansion, hygiene, expansion-file identities                               |
| `ra_ap_hir_def`    | Definition lowering, module scopes, name resolution, item bodies                  |
| `ra_ap_hir_ty`     | Type inference, trait solving, semantic analysis, MIR-related machinery           |
| `ra_ap_hir`        | Public-facing semantic objects and high-level queries                             |

A crate is a configured compilation instance, not merely a package name. A single source file can occur in more than one crate instance, and the same syntax can acquire different resolutions depending on the active configuration. Consequently, `FileId`, `Crate`, `Module`, and definition identity are not interchangeable notions.

Cargo-specific translation into this database belongs to project-model/loading components. Once loaded, HIR operates on an abstract crate graph and source database; its semantic model is not coupled to a particular build-system implementation.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

source+1



### 2.2 Incremental query boundaries

The underlying engine uses Salsa to compute and cache derived facts on demand. Two internal representations explain much of its incremental behavior:

- `ItemTree`: A condensed representation of a file's declarations, designed to remain unchanged when only function-body contents change.
- `DefMap`: Crate/module name-resolution state, separated from expression-level analysis in individual bodies.

The intended invariant is that editing one function body should not invalidate unrelated global semantic information. This is materially different from building a fresh, whole-program symbol table and type model after each edit.

The benefit is strongest when repeated queries reuse a database across successive revisions. It does not imply that every HIR operation is inexpensive: trait solving, macro expansion, or an uncached body inference can still involve substantial computation.

Cancellation and concurrency: Rust-analyzer's Salsa integration uses cancellation through unwinding when revisions change. Higher-level `ide` APIs catch cancellation and translate it into an explicit result. Direct HIR consumers operate beneath that boundary and must account for the database's cancellation behavior rather than assuming ordinary `Result`-based propagation.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://rust-analyzer.github.io\&sz=32)

rust-analyzer+1



### 2.3 Object lifetimes and locality

`Semantics<'db, DB>` is the central syntax-facing façade. It can be constructed with a concrete database type using `Semantics::new` or with a trait-object database using `Semantics::new_dyn`.

It maintains internal caches for syntax-to-definition and macro-related lookups. The 0.0.358 API marks `Semantics` as neither `Send` nor `Sync`. This restriction applies to the analysis façade, not necessarily to every semantic handle; for example, `Type<'db>` implements both traits.

Many definition handles (`Function`, `Struct`, `Module`, `Impl`, etc.) are inexpensive, copyable identities. By contrast, `Type<'db>` carries context and a database-linked lifetime, and its generic origin is semantically significant. Persisting raw HIR identities as durable cross-run identifiers would require additional assumptions about database and revision stability that the library does not guarantee.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+2



Sources:

[https://docs.rs/ra_ap_hir/latest/ra_ap_hir/struct.Semantics.html](https://docs.rs/ra_ap_hir/latest/ra_ap_hir/struct.Semantics.html?utm_source=chatgpt.com)

[https://rust-analyzer.github.io/book/contributing/architecture.html](https://rust-analyzer.github.io/book/contributing/architecture.html?utm_source=chatgpt.com)

## 3. Definition modeling, module graphs, and name resolution

### 3.1 Semantic entities rather than syntax declarations

The principal definition representations include `ModuleDef`, `Adt` (`Struct`, `Enum`, `Union`), `Function`, `Const`, `Static`, `TypeAlias`, `Trait`, `Impl`, `Macro`, `Field`, and `EnumVariant`.

Common traits provide uniform access to information such as `HasName`, `HasCrate`, `HasContainer`, `HasVisibility`, `HasAttrs`, and `HasSource`, where applicable.

The significant distinction is that these objects represent resolved definitions, not just AST nodes that resemble declarations. They can identify their enclosing module, semantic type, associated items, visibility, attributes, and relationships to other definitions without independently rebuilding those structures.

`Semantics::to_def` and typed conversions such as `to_fn_def`, `to_trait_def`, `to_impl_def`, and `to_enum_variant_def` bridge source AST nodes to these semantic identities. `HasSource` and source-location operations support the opposite direction.

Because syntax can be expanded, conditionally excluded, or included in multiple crate contexts, these bridges are not a universal one-to-one mapping.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+1



### 3.2 Crate and module traversal

`Crate` exposes the configured graph at a higher level through `all`, `root_module`, `modules`, `dependencies`, `reverse_dependencies`, `transitive_reverse_dependencies`, `edition`, `cfg`, `potential_cfg`, `origin`, `version`, and `root_file`.

`Module` then exposes the resolved module tree and contents through `children`, `parent`, `declarations`, `impl_defs`, `scope`, `modules_in_scope`, and `resolve_mod_path`.

A consequential distinction:

- `declarations` enumerates module definitions; it should not be confused with all lexically reachable names.
- `scope` describes items visible from a module, including resolution effects from imports and visibility.
- `resolve_mod_path` resolves names within semantic namespaces instead of simply concatenating module names.
- `find_path` and `find_use_path` work in the reverse direction: given an item, determine a usable path from a particular module, subject to `FindPathConfig`.

Together, these capabilities support semantic API inventories, accessible-symbol discovery, import analysis, and path generation without separately implementing Rust's module and visibility rules.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+1



Module source identity is unusual: `Module` deliberately does not implement `HasSource` because a module may have both a declaration (`mod foo;`) and a definition in another file. `declaration_source` and `definition_source` distinguish the two; crate-root modules have no ordinary `mod` declaration. Treating every module as possessing a single definition span loses important source structure.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust



### 3.3 Lexical scopes and namespace-sensitive resolution

`Semantics::scope` and `scope_at_offset` provide `SemanticsScope` objects for inspecting the environment at a source position. The scope can expose:

- Locally available names through `process_all_names`.
- Visible traits and trait-method availability.
- The containing module, crate, function, and generic-definition context.
- Associated-type shorthand candidates.
- Speculative resolution of hypothetical names or paths.

The complementary `resolve_path`, `resolve_path_per_ns`, and `resolve_path_with_subst` methods answer more specific semantic questions. `PathResolution` distinguishes definitions from locals, type parameters, const parameters, `Self`, and other resolution kinds. `PathResolutionPerNs` preserves the type, value, and macro namespace distinction.

This is more precise than a flat symbol dictionary: the same name can have different meanings across namespaces, and method lookup depends on traits and the receiver's inferred type.

Limit: Scope reconstruction and speculative resolution are useful for discovery and code completion, but some speculative paths use approximations and do not fully reproduce macro hygiene. A candidate visible in a reconstructed scope should not be equated automatically with a proven resolution at a particular expression.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

source+1



### 3.4 Importable symbol discovery

The `import_map` facility, re-exported through HIR, maintains searchable indexes of publicly exported crate items. The important handles are `ImportMap`, `ImportInfo`, `Query`, `SearchMode`, `AssocSearchMode`, and `search_dependencies`. At the crate level, `Crate::query_external_importables` exposes dependency import discovery.

This is distinct from traversing a crate's declaration tree: import maps account for public exports and names under which items can be imported, including re-export paths.

The `symbols` module provides another discovery layer oriented toward symbols and their declaration locations (`FileSymbol`, `SymbolCollector`, `DeclarationLocation`). These facilities have different roles: import maps answer importability questions; symbol collection supports name-based discovery; semantic resolution establishes actual identity at a use site.

Sources:&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+2



[https://docs.rs/ra_ap_hir/latest/ra_ap_hir/import_map/index.html](https://docs.rs/ra_ap_hir/latest/ra_ap_hir/import_map/index.html?utm_source=chatgpt.com)

[https://docs.rs/ra_ap_hir/latest/ra_ap_hir/symbols/index.html](https://docs.rs/ra_ap_hir/latest/ra_ap_hir/symbols/index.html?utm_source=chatgpt.com)

## 4. Macro expansion, hygiene, and source provenance

### 4.1 Expanded code is part of the semantic model

Macro handling is not an optional postprocessing step. Rust's module structure, definitions, expressions, and references can be produced by macros; resolving them requires tracking expanded syntax and its context.

`Semantics` exposes operations including `parse_or_expand`, `expand_macro_call`, `expand_attr_macro`, `expand_derive_macro`, `resolve_macro_call`, and `resolve_derive_macro`.

The surrounding `ra_ap_hir_expand` infrastructure represents expansion trees using `HirFileId` and `MacroCallId`, while source-related types such as `InFile` retain file context. An expanded syntax node need not correspond to a normal on-disk file or a contiguous user-written source range.

Macro definition and expansion semantics are supported by companion infrastructure:

- Declarative macro token-tree transformation is implemented in the macro-expansion/token-tree crates.
- Macro hygiene and syntax context participate in name interpretation.
- Procedural macro execution is handled through separate rust-analyzer components, including a process-based macro server, rather than being implemented by `ra_ap_hir` itself.&#x20;

  [image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

  Rust+2



### 4.2 Expansion-aware navigation

`Semantics` supplies several different ways to move between user-written code and generated syntax:

| Mechanism                              | Distinction                                                               |
| -------------------------------------- | ------------------------------------------------------------------------- |
| `descend_into_macros`                  | Follow syntax through macro expansions                                    |
| `descend_into_macros_exact`            | Constrain descent to corresponding syntax                                 |
| `ancestors_with_macros`                | Traverse enclosing syntax across expansion boundaries                     |
| `find_node_at_offset_with_descend`     | Locate a relevant AST node at a source offset, descending into expansions |
| `original_range`                       | Map generated syntax toward original source, with fallback behavior       |
| `original_range_opt`                   | Seek source mapping without the same fallback                             |
| `original_ast_node`                    | Recover certain original nodes, with documented restrictions              |
| `descend_token_into_include_expansion` | Handle the source/expanded representation of included content             |

These distinctions matter for generated references, semantic indexing, rename analysis, and source diagnostics. A mapping that returns the macro invocation's span is not equivalent to a precise token-level provenance mapping.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+1



### 4.3 Speculative expansion

`speculative_expand_macro_call`, `speculative_expand_attr_macro`, and related derive-expansion operations allow expansion of hypothetical input while attempting to map a selected token into the resulting syntax.

This is an advanced foundation for completion inside macro invocations: a tool can reason about how edited or incomplete input would expand without committing the edit to the database first.

However, the result is a speculative expansion, not a proof that the modified program would successfully compile. Macro behavior can depend on inputs, environment, procedural-macro availability, and implementation constraints.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust



Source:

[https://docs.rs/ra_ap_hir/latest/ra_ap_hir/struct.Semantics.html](https://docs.rs/ra_ap_hir/latest/ra_ap_hir/struct.Semantics.html?utm_source=chatgpt.com)

## 5. Type inference, generic substitution, and type operations

This is arguably the most consequential capability family in `ra_ap_hir`. It offers substantially more than obtaining the type name of a declaration.

### 5.1 Expression-level inference and adjustments

`Semantics::type_of_expr` and `type_of_pat` return `TypeInfo`, which distinguishes the original inferred type from the adjusted type after language-level adjustments.

Related methods include `expr_adjustments`, `pattern_adjustments`, `binding_mode_of_pat`, `type_of_binding_in_pat`, `type_of_self`, `expr_is_diverging`, and `try_expr_returned_type`.

The distinctions matter for correctly interpreting:

- Autoderef, autoref, unsizing, and coercions.
- Pattern matching and match ergonomics.
- Binding types versus the type of a larger pattern.
- Diverging expressions and inferred return behavior.
- Receiver adjustments during method resolution.

A syntax-only extractor may observe that an expression contains no explicit borrow yet miss that Rust effectively borrows its value when calling a method. HIR can expose the inferred and adjusted representations.

Inference is local to a configured semantic environment. Unknown or erroneous types can remain when source is incomplete, dependencies are unavailable, or inference cannot establish a result. HIR represents such uncertainty rather than guaranteeing a concrete type for every expression.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+1



### 5.2 Structural type inspection

`Type<'db>` provides structured inspection rather than requiring a string parser for Rust type syntax.

Notable families include:

- Type kinds: `as_adt`, `as_adt_with_args`, `as_reference`, `as_raw_ptr`, `as_array`, `as_slice`, `as_builtin`, `as_dyn_trait`, `as_impl_traits`, `as_type_param`, `as_callable`, `as_closure`, `as_coroutine`.
- Composite structure: `fields`, `tuple_fields`, `walk`, `type_arguments`, `type_and_const_arguments`, `generic_parameters`, `generic_params`.
- Reference operations: `strip_reference`, `strip_references`, `add_reference`, `autoderef`, `contains_reference`.
- Semantic properties: `is_copy`, `contains_unknown`, `is_never`, `is_packed`, and type-specific predicates.

There are important information-preservation distinctions. `type_arguments` excludes const generic arguments; `type_and_const_arguments` includes type and const arguments in a display-oriented form. `as_adt_with_args` may contain gaps for lifetime and const positions. None should be mistaken for a complete, lossless generic substitution representation.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust



### 5.3 Generic context and substitutions

A `Type<'db>` carries information about its generic origin. This matters when comparing types declared under different generic binders.

For example, a `T` belonging to an `impl<T>` and a `T` belonging to an unrelated `fn f<T>` are not interchangeable solely because their parameter names agree.

The important operations are:

| Mechanism                 | Semantic purpose                                                        |
| ------------------------- | ----------------------------------------------------------------------- |
| `instantiate`             | Replace generic parameters with supplied type arguments                 |
| `instantiate_with_errors` | Replace unresolved generic parameters with error types                  |
| `try_rebase_into`         | Transfer a type into another compatible type's generic context          |
| `try_rebase_into_owner`   | Rebase into a particular generic definition                             |
| `rebase_into_or_error`    | Obtain a representation even when rebasing cannot be completed cleanly  |
| `GenericSubstitution`     | Represent substitutions associated with a resolved path, field, or call |

Rebasing is particularly valuable when reasoning across trait declarations, implementation blocks, associated functions, and instantiated uses. It replaces error-prone comparisons based on type-display strings.

Crucial restriction: Most operations involving multiple `Type` values expect compatible origins. Combining incompatible generic contexts may fail or panic; `Type` equality and explicitly documented exceptions do not establish that arbitrary cross-context operations are safe.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust



### 5.4 Type compatibility is not a single predicate

HIR exposes several different notions of type compatibility:

| Operation                 | What it asks                                                                               |
| ------------------------- | ------------------------------------------------------------------------------------------ |
| `could_unify_with`        | Could these types unify under permissive treatment of unresolved placeholders?             |
| `could_unify_with_deeply` | Can compatibility be established without unresolved generic goals being silently accepted? |
| `could_coerce_to`         | Could the source type coerce to the target type in this direction?                         |
| `impls_trait`             | Does the type satisfy the requested trait obligation under its generic context?            |
| `has_any_impl`            | Is there some substitution of generic arguments for which the trait can be implemented?    |

These are not interchangeable.

For instance, `Option<T>` and `Option<U>` can pass the permissive unification check even when no relationship between `T` and `U` has been proven. Similarly, `has_any_impl` is an existential query: a concrete `impl Trait for Foo<i32>` can satisfy the existence question for generic `Foo<T>` without proving that every `Foo<T>` implements that trait.

`could_coerce_to` is directional, reflecting that Rust coercion is not simply symmetric unification.

This distinction makes the library useful for candidate generation and filtering, while clarifying where more stringent validation is needed.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust



### 5.5 Semantic rendering and source-code generation

`HirDisplay` is an unusually important capability for tools that need to represent types accurately.

- `display` produces human-readable output appropriate for diagnostics, hover text, and inspection.
- `display_truncated` and `display_limited` constrain output for interface contexts.
- `display_source_code` considers a destination module and produces a `Result`, allowing type names and paths to be rendered as source code in a specific context.
- `DisplayTarget` controls the intended display environment.

Human-readable type text is not necessarily valid insertable Rust syntax. An inferred type may be anonymous, opaque, inaccessible by a particular path, or otherwise unsuitable for direct insertion. Source-code rendering is a separate semantic operation, not a cosmetic formatting choice.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust



### 5.6 Layout, drop behavior, and associated types

HIR exposes less obvious type operations with consequences beyond traditional IDE navigation:

- `Type::layout` produces target-dependent layout information as `Result<Layout, LayoutError>`.
- `Type::drop_glue` characterizes relevant drop behavior.
- `Type::normalize_trait_assoc_type` resolves associated-type projections where possible.
- `Type::iterator_item` and `into_iterator_iter` distinguish `Iterator` from `IntoIterator`.
- `Type::future_output` and `into_future_output` distinguish `Future` from `IntoFuture`/`Future`.
- `Type::impls_fnonce` supports determining whether a type satisfies the call-trait hierarchy.

These facilities can support structural characterization of data layouts, trait-oriented interfaces, async expressions, and iterator pipelines. Layout and normalization are fallible or context-dependent; they are not substitutes for complete code generation or compiler-validated ABI analysis.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+1



Primary source:

[https://docs.rs/ra_ap_hir/latest/ra_ap_hir/struct.Type.html](https://docs.rs/ra_ap_hir/latest/ra_ap_hir/struct.Type.html?utm_source=chatgpt.com)

## 6. Trait resolution, implementation discovery, and method dispatch

### 6.1 Trait structure and supertrait relationships

`Trait` provides `direct_supertraits`, `all_supertraits`, `items`, `items_with_supertraits`, and methods for inspecting individual associated items.

It also exposes:

- `is_auto` and `is_unsafe`.
- `dyn_compatibility` and `dyn_compatibility_all_violations`, which assess restrictions on trait-object use.
- `must_implement_one_of`, representing applicable trait-implementation metadata.
- Completion-related metadata such as `complete` and `prefer_underscore_import`.

These facilities support trait hierarchy exploration, associated API discovery, and analysis of the requirements imposed by trait-object usage.

A trait's own items and its inherited associated items form different sets. This distinction is significant for documenting APIs, determining implementation obligations, and constructing member inventories.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust



### 6.2 Enumerating implementation declarations versus proving applicability

`Impl` exposes `all_in_crate`, `all_in_module`, `all_for_trait`, `all_for_type`, `self_ty`, `trait_`, `trait_ref`, `items`, `is_negative`, `is_unsafe`, and `check_orphan_rules`.

This supports both directions of structural exploration: implementations of a given trait and implementations associated with a type.

However, implementation discovery is not equivalent to trait solving.

`Impl::all_for_type` uses a shallow type-oriented search and is not guaranteed to enumerate all applicable blanket implementations. A generic blanket implementation may apply to a concrete type without being returned by a query that merely gathers implementations indexed by that type's constructor.

For stronger applicability questions, use type/trait-obligation reasoning (`impls_trait`, `has_any_impl`, and associated solver-backed operations), while recognizing the different semantics of each.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust



### 6.3 Method candidate search versus actual method resolution

Method dispatch is substantially more complex than locating a function with a matching name. It depends on receiver type, dereference steps, autoref, generic substitutions, available traits, trait bounds, and inherent implementations.

`Type` exposes:

- `iterate_method_candidates`
- `iterate_method_candidates_with_traits`
- `iterate_method_candidates_split_inherent`
- `iterate_path_candidates`
- `iterate_path_candidates_split_inherent`
- `iterate_assoc_items`
- `autoderef`

`Semantics` supplies the complementary use-site resolution methods `resolve_method_call`, `resolve_method_call_as_callable`, `resolve_method_call_fallback`, `resolve_expr_as_callable`, and `resolve_trait_impl_method`.

The architectural distinction is important:

- Candidate enumeration asks what may be callable from a type and scope.
- Use-site resolution asks what a particular expression actually denotes after inference and dispatch rules.
- Callability describes the resulting callable signature, not merely the existence of a named function.

`iterate_method_candidates_split_inherent` also documents a subtlety: a method surfaced as an inherent candidate may originate from a trait when the receiver is `dyn Trait`. Therefore, candidate classification should not be treated as a simplistic declaration-origin classification.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+2



### 6.4 Higher-level operator resolution

A particularly useful advanced family resolves Rust's special syntax back to the language operations it represents:

`resolve_bin_expr`, `resolve_index_expr`, `resolve_prefix_expr`, `resolve_try_expr`, `resolve_await_to_poll`, `resolve_range_expr`, and `resolve_range_pat`.

These enable analysis of operations that cannot be captured by scanning ordinary function-call syntax.

For example, an indexing expression, a `?` expression, or an `.await` expression may depend on trait machinery or special lowering rules. The associated semantic operations expose information that is otherwise hidden in the syntactic shape.

This is valuable when characterizing dependencies or constructing a semantic operation graph, although the exact relationship between an operator and its implementation may involve compiler desugaring rather than a literal source-level call.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust



### 6.5 Solver architecture

The 0.0.358 source imports substantial machinery from `hir_ty::next_solver`, including rustc-derived type IR and inference infrastructure. This is a material version-sensitive distinction: older rust-analyzer descriptions that characterize the current solver solely in terms of Chalk are not sufficiently precise for this release.

`ra_ap_hir` exposes selected high-level queries while the full trait-solving infrastructure remains in `ra_ap_hir_ty` and its implementation dependencies. The HIR façade is thus the more cohesive interface for most semantic interrogation, but it does not promise a fully stable public trait-solver API.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

source



Sources:

[https://docs.rs/ra_ap_hir/latest/ra_ap_hir/struct.Trait.html](https://docs.rs/ra_ap_hir/latest/ra_ap_hir/struct.Trait.html?utm_source=chatgpt.com)

[https://docs.rs/ra_ap_hir/latest/ra_ap_hir/struct.Impl.html](https://docs.rs/ra_ap_hir/latest/ra_ap_hir/struct.Impl.html?utm_source=chatgpt.com)

[https://docs.rs/ra_ap_hir/latest/ra_ap_hir/struct.Semantics.html](https://docs.rs/ra_ap_hir/latest/ra_ap_hir/struct.Semantics.html?utm_source=chatgpt.com)

## 7. Callables, bodies, closures, attributes, and documentation semantics

### 7.1 Function and callable modeling

`Function` offers more than a function's name and signature. It distinguishes declaration properties and context:

- `ty`, `fn_ptr_type`, `ret_type`, and `async_ret_type`.
- `self_param`, `assoc_fn_params`, `method_params`, `params_without_self`.
- `is_const`, `is_async`, `is_unsafe`, `is_varargs`, `extern_block`.
- `has_body`, `returns_impl_future`, and `is_unsafe_to_call`.
- Metadata relevant to entry points and tests: `is_main`, `exported_main`, `is_test`, `is_bench`, `is_ignore`.

`Callable` supplies a normalized representation of callable entities with their parameter and return types. This accommodates function items, callable expressions, closures, and method-call resolution without requiring each category to be handled through unrelated AST rules.

In particular, a function item's type and a function-pointer type are not the same Rust type. The separation of `ty` and `fn_ptr_type` preserves that distinction.

Likewise, an async function's declared return type and the future-producing callable behavior need to be interpreted separately.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+1



### 7.2 Expression bodies and locals

`Body` and `ExpressionStore` expose the lower-level lowered representation of function-like bodies, patterns, expressions, parameters, and source maps.

The syntax-facing façade exposes corresponding resolved behavior through methods such as `type_of_expr`, `resolve_path`, `resolve_field`, `locals_used`, `fn_return_points`, `resolve_label`, and pattern-related queries.

This produces two complementary approaches:

- AST-guided investigation: Traverse source nodes and request semantics for encountered expressions.
- Lowered-body investigation: Inspect expression/pattern identities and the associated body/source maps, accepting tighter coupling to rust-analyzer's internal representations.

The second approach can support operations over whole bodies without continually reparsing source syntax, but lower-level expression IDs remain scoped to their owners and should not be confused with globally meaningful symbol identities.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+2



### 7.3 Closures and capture behavior

`Closure` exposes `captured_items`, `capture_types`, and `fn_trait`. These reveal which environment values are captured, their relevant types, and the inferred `Fn`/`FnMut`/`FnOnce` behavior.

The distinction matters for reasoning about move semantics, borrowing, callable bounds, and closure-generated state.

Closures are not merely anonymous functions: their type and call-trait properties depend on their captured environment. HIR makes this relationship available without reconstructing capture analysis from AST identifiers and lexical scope alone.

The library also exposes coroutine-related type inspection (`Type::as_coroutine`), reflecting its support for Rust's generator/async-related semantic representations.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+1



### 7.4 Attributes and Rustdoc information

`HasAttrs` and `AttrsWithOwner` expose meaningful interpreted attribute information rather than only raw attribute tokens.

Important examples include:

- Conditional compilation and feature configuration.
- `deprecated`, `doc(hidden)`, `non_exhaustive`, unstable-feature markers.
- Macro-export and macro-related metadata.
- Documentation aliases and trait-documentation indicators.
- `hir_docs` for assembled documentation content.
- `resolve_doc_path_on` for resolving intra-documentation references to semantic entities.

This supports API documentation modeling, source annotation, deprecation inventories, and Rustdoc-aware navigation. The key advantage is the ability to combine documentation text with resolved symbol identity.

The API does not itself constitute a full Rustdoc renderer or complete substitute for Rustdoc's final documentation model. Attribute interpretation must still be evaluated within its crate configuration and visibility context.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

source+1



Sources:

[https://docs.rs/ra_ap_hir/latest/ra_ap_hir/struct.Function.html](https://docs.rs/ra_ap_hir/latest/ra_ap_hir/struct.Function.html?utm_source=chatgpt.com)

[https://docs.rs/ra_ap_hir/latest/ra_ap_hir/struct.Closure.html](https://docs.rs/ra_ap_hir/latest/ra_ap_hir/struct.Closure.html?utm_source=chatgpt.com)

[https://docs.rs/ra_ap_hir/latest/src/ra_ap_hir/attrs.rs.html](https://docs.rs/ra_ap_hir/latest/src/ra_ap_hir/attrs.rs.html?utm_source=chatgpt.com)

## 8. Diagnostics, proof inspection, MIR, and semantic synthesis

These capabilities are particularly easy to overlook because they extend beyond the conventional responsibilities of a symbol-resolution library.

### 8.1 Structured semantic diagnostics

`Module::diagnostics(db, acc, style_lints)` collects diagnostic objects into a supplied vector. The `diagnostics` module exposes `AnyDiagnostic` and numerous individual diagnostic types.

The 0.0.358 interface includes diagnostic families covering:

| Family                               | Representative diagnostic types                                                                                             |
| ------------------------------------ | --------------------------------------------------------------------------------------------------------------------------- |
| Name and member resolution           | `UnresolvedIdent`, `UnresolvedImport`, `UnresolvedMethodCall`, `UnresolvedField`, `UnresolvedModule`, `UnresolvedAssocItem` |
| Types and inference                  | `TypeMismatch`, `TypeMustBeKnown`, `TypedHole`, `CannotIndexInto`, `InvalidCast`                                            |
| Trait and implementation validity    | `UnimplementedTrait`, `TraitImplMissingAssocItems`, `TraitImplOrphan`, `TraitImplIncorrectSafety`, `IncoherentImpl`         |
| Pattern and constructor completeness | `MissingFields`, `MissingMatchArms`, `NonExhaustiveLet`, `NonExhaustiveRecordExpr`                                          |
| Functions and control flow           | `MismatchedArgCount`, `AwaitOutsideOfAsync`, `BreakOutsideOfLoop`, `ReturnOutsideFunction`                                  |
| Macro processing                     | `MacroError`, `MacroDefError`, `MacroExpansionParseError`, `UnresolvedMacroCall`                                            |
| Safety and visibility                | `MissingUnsafe`, `PrivateField`, `PrivateAssocItem`                                                                         |

The value is that many findings are represented as typed semantic conditions, rather than requiring a consumer to parse textual compiler messages.

The `style_lints` argument distinguishes additional stylistic analysis from the core diagnostic collection, but should not be interpreted as a general switch for all Clippy or rustc warnings. Rust-analyzer's IDE diagnostics layer performs additional presentation, range mapping, severity handling, and fix generation. Compiler diagnostics from `cargo check` and Clippy are separate external information sources.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

docs.rs+2



### 8.2 Trait-obligation diagnostics and predicate evaluation

`Semantics` also exposes lower-level reasoning aids:

- `evaluate_where_clause_at` evaluates a where-clause condition within source context.
- `get_failed_obligations` retrieves diagnostic information about failed type-system obligations at a source location.
- `PredicateEvaluationResult` pairs explanatory text with `PredicateEvaluationStatus`.

The statuses in 0.0.358 are:

| Status        | Interpretation                                           |
| ------------- | -------------------------------------------------------- |
| `Holds`       | The queried predicate was established                    |
| `NotProven`   | The analysis did not establish that it holds             |
| `Invalid`     | The requested evaluation was invalid                     |
| `Unsupported` | The evaluation is unsupported by the available machinery |

The distinction between `NotProven` and a proven contradiction is fundamental. In a generic context, insufficient information is not the same as a violated constraint.

The current source also integrates proof-tree formatting machinery from `hir_ty::next_solver`. However, `get_failed_obligations` returns textual information, not a guaranteed stable, complete, machine-readable derivation tree through the HIR façade.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

source+2



### 8.3 Safety, fields, patterns, and local semantic checks

Several smaller methods provide useful higher-order semantic information:

- `get_unsafe_ops`, `get_unsafe_ops_for_unsafe_block`, and `is_inside_unsafe` identify unsafe-related operations and contexts.
- `Function::is_unsafe_to_call` takes both a caller context and a call-site edition, rather than relying solely on an `unsafe fn` declaration flag.
- `record_literal_missing_fields`, `record_literal_matched_fields`, `record_pattern_missing_fields`, and `record_pattern_matched_fields` provide type-aware field analysis.
- `resolve_record_field_with_substitution` and related operations preserve the instantiated field type and generic substitution.
- `rename_conflicts` detects potential local-name collisions.
- `locals_used` and `fn_return_points` expose selected dataflow- and control-flow-relevant facts.

These operations offer reusable primitives for refactoring analysis, source validation, and code assistance.

They do not amount to a complete borrow checker, control-flow-analysis framework, or safe refactoring engine. Their coverage is specific to their documented semantic questions. In particular, a local rename-conflict check does not replace a full cross-crate rename operation.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+2



### 8.4 MIR: control-flow representation and interpretation

One of the most advanced exposed capabilities is the `mir` module, re-exported from `ra_ap_hir_ty`.

Its principal representations include:

- `MirBody`, `BasicBlock`, `Statement`, and `Terminator`.
- `Local`, `Place`, `Projection`, `PlaceTy`, and `Operand`.
- `Rvalue`, `AggregateKind`, `BorrowKind`, and `CastKind`.
- `MirLowerError` and `MirEvalError`.
- Query/lowering entry points such as `mir_body_query`, `monomorphized_mir_body_query`, `lower_body_to_mir`, and `interpret_mir`.

This allows certain analyses to operate on lowered control flow, values, and storage locations instead of directly navigating Rust syntax.

The important interaction is between three representations:

| Representation                | Main strength                                               | Main limitation                                                                           |
| ----------------------------- | ----------------------------------------------------------- | ----------------------------------------------------------------------------------------- |
| Source AST                    | Exact syntax, tokens, source editing                        | Implicit operations and semantic identities are not directly represented                  |
| HIR expressions and inference | Resolved entities, types, generic context                   | Does not by itself expose every low-level control-flow operation                          |
| MIR                           | Explicit lowered operations, control flow, places, operands | More internal, more expensive to interpret, and less directly tied to user-written syntax |

The upstream `hir_ty` documentation identifies separate stages for polymorphic MIR lowering, monomorphization, interpretation, and pretty-printing. These mechanisms are implemented by rust-analyzer and should not be assumed to be interchangeable with rustc's internal MIR data structures, compiler passes, or borrow-checker results.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://rust-lang.github.io\&sz=32)

Rust+1



### 8.5 Constant evaluation and binary layout

`Const::eval`, `Static::eval`, and enum-variant discriminant evaluation expose interpreter-backed results. `EvaluatedConst` represents evaluated constant information, while `ConstEvalError` accounts for evaluation failures.

Additional entry points cover anonymous constants and expression-related constant data.

Together with `Type::layout`, `EnumVariant::layout`, and representation information such as `repr`, these operations support selected inquiries into:

- Enum discriminants and constant values.
- Compile-time initializers.
- Array-related constant parameters.
- Target-specific size, alignment, and layout.
- The relationship between declared representation attributes and computed layout.

The distinction between evaluation and source inspection matters: a constant initializer may compute its value through expressions rather than containing an immediately visible literal.

Evaluation and layout results depend on the analysis environment and supported interpreter capabilities. They are not a general-purpose Rust execution engine or a substitute for runtime evaluation.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

source+1



### 8.6 Term search: type-directed expression construction

The `term_search` module is a substantial capability that is rarely mentioned in introductory descriptions of rust-analyzer.

Its principal interfaces are `term_search`, `TermSearchCtx`, `TermSearchConfig`, and the result representation `Expr`.

Given a desired type and semantic scope, the algorithm searches for expressions that can produce that type.

Its implementation uses a breadth-first, type-directed search:

1. Populate reachable values from scope—locals, constants, statics, and simple known expressions.
2. Apply operations that produce new types, such as function calls, method calls, constructors, borrows, and field projections.
3. Match reachable expression types against the target.
4. Stop or compress alternatives according to configurable search limits.

`TermSearchConfig` provides a work budget (`fuel`) and an alternative-expression threshold; the underlying search uses a `Many` representation when the number of candidates becomes excessive.

This is a meaningful architectural departure from conventional text completion. It searches over semantic constructions rather than simply ranking identifiers by prefix.

It is nevertheless a heuristic, bounded synthesis mechanism. Results are not exhaustive, and successful type matching is not equivalent to full compiler verification of generated source. The exact configuration fields have changed between source snapshots, so version-specific confirmation is important before depending on additional options.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://rust-lang.github.io\&sz=32)

source+1



Source:

[https://rust-lang.github.io/rust-analyzer/src/hir/term_search.rs.html](https://rust-lang.github.io/rust-analyzer/src/hir/term_search.rs.html?utm_source=chatgpt.com)

## 9. Integration boundaries and capability combinations

### 9.1 Adjacent components are not interchangeable

The practical choice of abstraction is determined by the semantic question being asked.

| Component                            | Primary responsibility                                  | Important distinction                                |
| ------------------------------------ | ------------------------------------------------------- | ---------------------------------------------------- |
| `ra_ap_syntax`                       | Parsing, tokens, concrete source structure              | No complete crate-aware semantic model               |
| `ra_ap_hir`                          | Resolved semantic objects, types, scopes, relationships | Requires configured analysis database                |
| `ra_ap_hir_def`                      | Definitions, lowering, scopes, name resolution          | Lower-level IDs and compiler internals               |
| `ra_ap_hir_expand`                   | Macro expansion and hygiene                             | Does not independently provide full type inference   |
| `ra_ap_hir_ty`                       | Types, inference, solver, MIR, evaluation               | More tightly coupled to implementation details       |
| `ra_ap_base_db`                      | Abstract crate graph and incremental inputs             | No Cargo-specific filesystem discovery               |
| `ra_ap_project_model`                | Cargo/project discovery and translation                 | Project loading rather than semantic interrogation   |
| `ra_ap_ide_db`                       | Reusable higher-level IDE services                      | Includes reference-search infrastructure             |
| `ra_ap_ide` and companion IDE crates | Navigation, completion, assists, diagnostics            | Editor-oriented outputs and operations               |
| rustc / rustdoc                      | Compiler validation and documentation generation        | Different scope, lifecycle, and interface guarantees |

The rust-analyzer architecture explicitly assigns reference searching to `ide-db`, and full editor-facing operations to the `ide` layer. `ra_ap_hir` supplies resolved identity and types needed by such features, but is not itself a complete find-all-references, rename, or LSP implementation.

`ra_ap_project_model` separately translates real-world Cargo or manually specified project layouts into the abstract crate model consumed downstream. That distinction preserves the ability to analyze non-Cargo configurations while requiring callers to supply accurate project context.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://rust-analyzer.github.io\&sz=32)

rust-analyzer+1



### 9.2 Combining HIR capabilities

Several especially valuable combinations follow directly from the library's architecture.

| General operation           | Native capabilities that compose                                                  | Important remaining responsibility                                               |
| --------------------------- | --------------------------------------------------------------------------------- | -------------------------------------------------------------------------------- |
| Resolved code graph         | `Crate`, `Module`, `ModuleDef`, `Semantics`, `PathResolution`, `HasSource`        | Reference traversal, graph persistence, cross-revision identity                  |
| Type-aware call analysis    | `resolve_method_call`, `Callable`, `TypeInfo`, substitutions, trait queries       | Representing dynamic dispatch, incomplete inference, expansion provenance        |
| Public API characterization | Module scopes, visibility, import maps, ADT/trait structure, `HasAttrs`, docs     | Decide which compiled configurations and exported paths are in scope             |
| Type-aware code generation  | `HirDisplay::display_source_code`, `find_use_path`, candidate search, term search | Generating edits and validating the modified program                             |
| Semantic diagnostics        | HIR diagnostic types, source mapping, failed obligations                          | User-facing diagnostics, suppression, quick fixes, compiler reconciliation       |
| Macro-aware indexing        | `HirFileId`, `MacroCallId`, expansion descent, original ranges                    | Provenance, ambiguity, generated-code identity                                   |
| Lowered operation analysis  | HIR types and definitions, MIR, const evaluation, layout                          | Compatibility of internal representations, error handling, interpretation limits |

The common theme is that `ra_ap_hir` supplies semantic truth within a configured analysis snapshot. It does not prescribe a persistent graph schema, serialization protocol, editor interaction, or durable source-edit workflow.

### 9.3 Choosing between HIR and rustdoc JSON

These are complementary representations rather than general substitutes.

Rustdoc JSON describes compiler-produced documented items and API structure. It is especially useful when the desired artifact is a documentation-oriented inventory tied to a compiler invocation.

HIR provides live, crate-aware semantic interpretation of source, including local variables, inferred expression types, references, trait candidates, scopes, and macro-aware navigation—information well beyond a public documentation inventory.

Rustdoc JSON is not a replacement for position-specific semantic resolution, and HIR is not automatically a replacement for rustdoc's authoritative documentation-generation output.

## 10. Consequential limitations and engineering tradeoffs

### 10.1 Correctness and completeness

`ra_ap_hir` is designed for rust-analyzer's responsive, often incomplete-source analysis environment.

Its answers depend on the available source, crate graph, macro processing, and compiler-like reasoning implemented by rust-analyzer. Its strong semantic facilities should not be interpreted as a guarantee of equivalence with a successful `rustc` compilation.

Particularly important distinctions:

- An unresolved or unknown type is not necessarily a source-language error.
- A permissive unification result is not proof of universal compatibility.
- Implementation enumeration may omit applicable blanket implementations.
- Speculative macro expansion does not prove final compilation behavior.
- Source mapping across macros can be lossy or ambiguous.
- MIR and const evaluation may fail or support only particular operations.
- HIR diagnostic coverage differs from rustc and Clippy coverage.&#x20;

  [image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

  Rust+2



### 10.2 Snapshot and identity boundaries

Semantic identity is associated with a particular database configuration and revision.

A persistent representation of code relationships therefore needs to distinguish HIR-internal identity from any external, stable identifier. Names alone are insufficient because of shadowing, overload-like namespace distinctions, generic contexts, macro-generated definitions, and repeated crate instances.

This also affects incremental workflows: updating source inputs may invalidate previously computed information, even when a particular query returns the same apparent result.

### 10.3 Performance model

Incremental computation is a major advantage, but not a blanket performance guarantee.

The relevant factors are:

- Database reuse: Repeated queries against the same incremental database can reuse cached derived state.
- Query scope: Module-level queries and body-level inference have different invalidation boundaries.
- Macro complexity: Expansion and procedural-macro availability can introduce substantial work.
- Trait solving: Deep generic relationships can be expensive or unresolved.
- Enumeration breadth: Collecting every declaration or querying implementations across dependencies differs from resolving one source position.
- Search bounds: Term search explicitly limits work and collapses alternatives to control memory growth.

`Semantics` itself uses interior-mutable source-to-definition caches and is not `Send`/`Sync`. The underlying database's threading and snapshot model must be considered separately.

The upstream architecture also documents tracing/profiling facilities. Performance conclusions should be based on the actual query mix rather than assuming linear scaling or full-cache reuse.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

source+2



### 10.4 Macro execution and environment fidelity

Procedural macros and build scripts belong to the broader project-loading and macro-execution environment.

A particularly relevant API detail is that `ProcMacros` documentation directs callers toward `get_for_crate` rather than a less granular accessor, preserving crate-scoped query dependencies.

If procedural macros are unavailable or disabled, semantic interpretation of macro-generated items can be incomplete. Conversely, allowing project build steps and procedural macros introduces execution and trust considerations outside HIR itself.

Target configuration, edition, feature flags, sysroot, and dependency completeness similarly influence what the database can resolve.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+2



### 10.5 API stability and documentation quality

Version 0.0.358 is a published release, but the API remains tied to rust-analyzer's internal development. The crate's explicit boundary is intended to reduce coupling within rust-analyzer, not to guarantee conventional SemVer stability for independent consumers.

The 0.0.358 documentation reports approximately 13% rustdoc item coverage (144 of 1,108 items). Consequently, some advanced behavior is best established through source, implementation-crate documentation, or tests rather than method descriptions alone.

During this research, several indexed `docs.rs/.../latest/...` subpages resolved to older snapshots even though the crate index and principal type pages reflected 0.0.358. For consequential API details, the 0.0.358 crate index and corresponding source take precedence over stale indexed subpages.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Docs.rs+1



## 11. Primary-source reference map

These URLs are grouped by the semantic capabilities they document. All are supplied as full URLs for independent access.

| Topic                                    | Direct source                                                                                                                                                         |
| ---------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Release metadata and dependency versions | [https://docs.rs/crate/ra_ap_hir/latest](https://docs.rs/crate/ra_ap_hir/latest?utm_source=chatgpt.com)                                                               |
| Complete current API index               | [https://docs.rs/ra_ap_hir/latest/ra_ap_hir/all.html](https://docs.rs/ra_ap_hir/latest/ra_ap_hir/all.html?utm_source=chatgpt.com)                                     |
| Crate implementation                     | [https://docs.rs/ra_ap_hir/latest/src/ra_ap_hir/lib.rs.html](https://docs.rs/ra_ap_hir/latest/src/ra_ap_hir/lib.rs.html?utm_source=chatgpt.com)                       |
| Syntax-to-semantics implementation       | [https://docs.rs/ra_ap_hir/latest/src/ra_ap_hir/semantics.rs.html](https://docs.rs/ra_ap_hir/latest/src/ra_ap_hir/semantics.rs.html?utm_source=chatgpt.com)           |
| `Semantics` methods                      | [https://docs.rs/ra_ap_hir/latest/ra_ap_hir/struct.Semantics.html](https://docs.rs/ra_ap_hir/latest/ra_ap_hir/struct.Semantics.html?utm_source=chatgpt.com)           |
| Type operations                          | [https://docs.rs/ra_ap_hir/latest/ra_ap_hir/struct.Type.html](https://docs.rs/ra_ap_hir/latest/ra_ap_hir/struct.Type.html?utm_source=chatgpt.com)                     |
| Module and source representation         | [https://docs.rs/ra_ap_hir/latest/ra_ap_hir/struct.Module.html](https://docs.rs/ra_ap_hir/latest/ra_ap_hir/struct.Module.html?utm_source=chatgpt.com)                 |
| Traits and compatibility                 | [https://docs.rs/ra_ap_hir/latest/ra_ap_hir/struct.Trait.html](https://docs.rs/ra_ap_hir/latest/ra_ap_hir/struct.Trait.html?utm_source=chatgpt.com)                   |
| Implementation discovery                 | [https://docs.rs/ra_ap_hir/latest/ra_ap_hir/struct.Impl.html](https://docs.rs/ra_ap_hir/latest/ra_ap_hir/struct.Impl.html?utm_source=chatgpt.com)                     |
| Function semantics                       | [https://docs.rs/ra_ap_hir/latest/ra_ap_hir/struct.Function.html](https://docs.rs/ra_ap_hir/latest/ra_ap_hir/struct.Function.html?utm_source=chatgpt.com)             |
| Attributes and documentation             | [https://docs.rs/ra_ap_hir/latest/src/ra_ap_hir/attrs.rs.html](https://docs.rs/ra_ap_hir/latest/src/ra_ap_hir/attrs.rs.html?utm_source=chatgpt.com)                   |
| Term-search algorithm                    | [https://rust-lang.github.io/rust-analyzer/src/hir/term_search.rs.html](https://rust-lang.github.io/rust-analyzer/src/hir/term_search.rs.html?utm_source=chatgpt.com) |
| MIR internals                            | [https://rust-lang.github.io/rust-analyzer/hir_ty/mir/index.html](https://rust-lang.github.io/rust-analyzer/hir_ty/mir/index.html?utm_source=chatgpt.com)             |
| Type-system implementation               | [https://rust-lang.github.io/rust-analyzer/hir_ty/index.html](https://rust-lang.github.io/rust-analyzer/hir_ty/index.html?utm_source=chatgpt.com)                     |
| Project loading model                    | [https://docs.rs/ra_ap_project_model/latest/ra_ap_project_model/](https://docs.rs/ra_ap_project_model/latest/ra_ap_project_model/?utm_source=chatgpt.com)             |
| Architecture and semantic boundaries     | [https://rust-analyzer.github.io/book/contributing/architecture.html](https://rust-analyzer.github.io/book/contributing/architecture.html?utm_source=chatgpt.com)     |
| Upstream HIR source directory            | [https://github.com/rust-lang/rust-analyzer/tree/master/crates/hir](https://github.com/rust-lang/rust-analyzer/tree/master/crates/hir?utm_source=chatgpt.com)         |

## 12. Overall capability assessment

`ra_ap_hir` is best understood as a crate-aware semantic query interface over an incremental, compiler-like model of Rust code.

Its distinguishing capabilities are not limited to resolved symbols. They include expression-level type inference and adjustments, contextual generic substitution, solver-backed trait reasoning, method-candidate exploration, macro-aware source provenance, structured diagnostics, MIR and constant evaluation, semantic documentation links, and bounded type-directed expression synthesis.

The most important architectural advantages are the ability to reuse rust-analyzer's existing Rust-language semantics rather than implementing custom name resolution, type modeling, trait matching, or macro tracking—and the ability to combine these facilities within a single coherent database.

Its central constraints are equally important: semantic answers are snapshot- and configuration-dependent; some operations are approximate, incomplete, or fallible; lower-level implementations are not stable public abstractions; and higher-level workflows such as reference searching, whole-project editing, serialization, and compiler validation remain outside HIR's core responsibility.

For sophisticated Rust language analysis, the library occupies a particularly valuable middle layer: substantially richer and more accurate than source-syntax analysis alone, while more focused and reusable than a full editor or compiler integration.