# rustdoc — Comprehensive Rust Library and Tooling Reference

Research date: October 29, 2026 | Rust toolchain: 1.99.1 | rustdoc-types: 0.61.0 | Rustdoc JSON schema reference: version 61

## 1. Purpose, scope, and architectural identity

`rustdoc` is Rust's compiler-integrated documentation engine, not an ordinary reusable Rust library. It generates navigable API documentation, resolves semantic links, executes documentation tests, validates documentation, and exposes an experimental structured representation of crate APIs.

Its significance extends beyond publishing documentation: rustdoc provides a compiler-informed representation of Rust's externally visible type system, including declarations, generics, traits, implementations, reexports, conditional visibility, and documentation relationships. This makes its structured output useful for API analysis, compatibility checking, library indexing, dependency exploration, and code-intelligence tooling.

Three distinct components must be distinguished:

| Component                                  | Responsibility                                                                     | Stability and integration boundary                                             |
| ------------------------------------------ | ---------------------------------------------------------------------------------- | ------------------------------------------------------------------------------ |
| `rustdoc` executable                       | Generates HTML documentation, checks docs, runs doctests                           | Distributed with the Rust toolchain; core functionality stable                 |
| Compiler-internal `rustdoc` / `librustdoc` | Uses rustc HIR, type queries, cleaned documentation IR, analysis passes, renderers | Internal compiler implementation; no stable public embedding API               |
| `rustdoc-types`                            | Provides Serde-compatible Rust definitions for rustdoc's structured JSON output    | Published crate, currently 0.61.0; underlying JSON format remains experimental |

Rust 1.99.1, released October 15, 2026, is the current stable toolchain. The current published `rustdoc-types` release is 0.61.0, defining JSON format version 61. These are independent version numbers. In particular, a specific nightly rustdoc build may emit a format different from the latest published `rustdoc-types` schema.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://blog.rust-lang.org\&sz=32)

Rust Blog+2



Primary sources:

`https://doc.rust-lang.org/rustdoc/`

`https://rustc-dev-guide.rust-lang.org/rustdoc.html`

`https://docs.rs/rustdoc-types/0.61.0/rustdoc_types/`

### Capability map

| Functional area                    | Principal mechanisms                                                        | Consequential value                                                                             |
| ---------------------------------- | --------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------- |
| Compiler-informed API discovery    | HIR, type queries, cleaned documentation IR                                 | Understands Rust declarations and resolved relationships rather than relying on syntax matching |
| Documentation surface construction | Reachability, reexports, inline/no-inline controls, conditional compilation | Models the documented API rather than merely the source module tree                             |
| Semantic navigation                | Intra-doc links, type-signature search, trait and impl presentation         | Connects APIs through language semantics                                                        |
| Executable documentation           | Doctests, compile-failure tests, target-specific behavior                   | Makes examples verifiable compatibility artifacts                                               |
| Documentation validation           | Rustdoc lints, link checking, coverage reporting                            | Detects inconsistencies without custom Markdown analysis                                        |
| Structured data extraction         | Experimental rustdoc JSON, `rustdoc-types`                                  | Enables programmatic API graphs and structural comparisons                                      |
| Advanced generation                | Scraped examples, source-definition links, layout inspection                | Enriches documentation with compiler-derived information                                        |
| Publishing and integration         | Cargo, docs.rs, cross-crate metadata, themes                                | Supports multi-crate publication and automated documentation pipelines                          |

## 2. Internal architecture: compiler semantics to documentation representation

### 2.1 Rustdoc is a compiler consumer

Rustdoc does not reconstruct Rust semantics from source text. It uses rustc infrastructure to load crates, perform name resolution, access HIR, and query type information.

The central conceptual pipeline is:

Rust source, attributes, Cargo configuration, dependencies

rustc frontend — parsing, expansion, resolution, HIR, type queries

Rustdoc clean IR — documentation-oriented item graph

Analysis and transformation passes — impls, links, visibility, lints

HTML renderer

Pages, search, source browsing

JSON renderer

Experimental machine-readable graph

This architecture explains rustdoc's most important advantage over parser-only documentation systems: it can distinguish semantically meaningful language structures, including resolved types and implementations, without implementing a separate Rust name resolver or type-system model.

It also explains its primary limitation: rustdoc's representation is intentionally optimized for documentation, not exhaustive compiler analysis. It does not preserve everything available in source syntax, HIR, MIR, or the compiler's query system.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://rustc-dev-guide.rust-lang.org\&sz=32)

Rust Compiler Development Guide+1



Sources:

`https://rustc-dev-guide.rust-lang.org/rustdoc.html`

`https://rustc-dev-guide.rust-lang.org/rustdoc-internals.html`

### 2.2 The cleaned documentation IR

The internal `clean` subsystem transforms Rust compiler representations into a documentation-specific intermediate representation.

Relevant internal components include `DocContext`, `visit_ast::RustdocVisitor`, `clean::utils::krate`, and `clean::Crate`.

The cleaning process does more than convert node types. It normalizes the documentation view of imports, reexports, visibility and module traversal. It can inline items defined in other crates, process public exports of privately nested items, and handle import globs and cycles.

This is a material distinction between a source declaration graph and a documented API graph. A publicly reachable item may appear under a path that is not its definition path. Conversely, source items can be omitted from documentation by visibility or documentation attributes.

The internal IR also incorporates documentation-oriented information that does not correspond one-to-one with source declarations, including synthesized auto-trait implementations and blanket implementations.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://rustc-dev-guide.rust-lang.org\&sz=32)

Rust Compiler Development Guide



### 2.3 Transformation and validation passes

Rustdoc applies passes over the cleaned representation, including `collect-trait-impls`, `collect-intra-doc-links`, `strip-private`, `strip-hidden`, `propagate-doc-cfg`, and documentation lint processing.

These passes establish relationships that syntax-only extraction cannot directly provide. They also transform the contents and shape of the resulting documentation graph.

For programmatic consumers, an important implication follows: the output is not simply an inventory of source-file AST nodes. It is a compiler-processed view shaped by visibility rules, compiler-derived implementations, inlining decisions, and renderer-oriented transformations.

The names and organization of these internal passes are useful investigation handles, but they should not be considered stable extension interfaces.

## 3. Documentation input, reachability, and surface construction

### 3.1 Documentation sources and attributes

Rustdoc recognizes Rust's outer `///` and inner `//!` documentation comments, along with the equivalent `#[doc = "..."]` attributes.

The attribute form enables documentation to originate outside the Rust source file, notably through `include_str!`, such as `#![doc = include_str!("../README.md")]`. It also supports macro-generated documentation.

Documentation can attach to modules, traits, associated items, functions, types, variants, fields, macros, and other language items. Crate-level documentation provides the entry point for the generated documentation hierarchy.

The underlying content uses Markdown with Rustdoc-specific processing for code examples and semantic links. The `#[doc(...)]` attribute family additionally governs presentation, search behavior, external linking, and reexport handling.

Source: `https://doc.rust-lang.org/rustdoc/write-documentation/the-doc-attribute.html`

### 3.2 Public reachability versus source visibility

By default, rustdoc documents publicly reachable items, not simply every declaration marked `pub`.

For example, a public item inside a private module is not directly accessible through the private module path. It becomes externally accessible if the crate exposes it through an appropriate public reexport.

The stable `--document-private-items` option expands documentation to include non-public items. It does not ordinarily override `#[doc(hidden)]`; exposing hidden items is a separate, unstable capability.

`#[doc(hidden)]` and Rust visibility have fundamentally different meanings: the former suppresses documentation, while the latter governs language-level access. A hidden public item can still be callable by downstream code. This distinction is particularly consequential for API inventory and compatibility checking.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://doc.rust-lang.org\&sz=32)

The rustdoc book



### 3.3 Reexports and documentation inlining

Rustdoc's treatment of `pub use` is sophisticated enough to affect API interpretation.

| Mechanism                             | Behavior                                                                    |
| ------------------------------------- | --------------------------------------------------------------------------- |
| Public reexport from a private module | Typically inlines the exported item under its accessible public path        |
| `#[doc(inline)]`                      | Forces documentation inlining of a reexport                                 |
| `#[doc(no_inline)]`                   | Retains the reexport as a reference rather than inlining its target         |
| `#[doc(hidden)]`                      | Suppresses the annotated item or reexport from normal documentation         |
| Reexport chains                       | Can accumulate documentation and applicable attributes from multiple stages |
| Cross-crate reexports                 | Can expose external definitions under a local crate's namespace             |

Inlined items may acquire documentation from both the original definition and reexport sites. Some attributes propagate or merge, while others are intentionally excluded. Intra-doc links in inherited comments are resolved relative to the site where those comments were originally defined.

Architectural implication: a reliable model of a crate's public namespace must retain both item identity and public exposure paths. Selecting a single definition path is not equivalent to reconstructing all legal import paths.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://doc.rust-lang.org\&sz=32)

The rustdoc book



Source: `https://doc.rust-lang.org/rustdoc/write-documentation/re-exports.html`

### 3.4 Conditional compilation is part of API identity

Rustdoc uses the selected target, feature configuration, edition, and `--cfg` inputs to determine which items enter documentation.

The stable `#[cfg(doc)]` mechanism can deliberately broaden documentation, for example through `#[cfg(any(windows, doc))]`, so platform-specific items are visible outside their normal target.

However, rustdoc does not perform a separate semantic compilation for every platform. When `cfg(doc)` causes mutually exclusive items to be present together, rustdoc may successfully render configurations that would not type-check as an ordinary program. The `doc` configuration is also not automatically passed to doctests.

This creates three distinct notions of availability: visibility in generated documentation, availability under a real build configuration, and successful compilation under that configuration. They must not be conflated.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://doc.rust-lang.org\&sz=32)

The rustdoc book



The nightly `#[doc(cfg(...))]` and `#[doc(auto_cfg)]` facilities add documentation-facing availability markers. `doc(auto_cfg)` can derive markers from configuration attributes, with inherited rules and `hide`/`show` filters; explicit `doc(cfg)` can override derived presentation. These markers describe availability but do not cause inactive items to be compiled into the documentation graph.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://doc.rust-lang.org\&sz=32)

The rustdoc book



Sources:

`https://doc.rust-lang.org/rustdoc/advanced-features.html`

`https://doc.rust-lang.org/rustdoc/unstable-features.html`

## 4. Semantic linking, navigation, and API discovery

### 4.1 Compiler-resolved documentation links

Rustdoc supports intra-doc links: Markdown references that resolve against Rust items rather than requiring manually constructed HTML URLs.

Supported forms include item names, module paths, `crate::` and `Self::` references, associated items, and namespace disambiguators such as `struct@Foo`, `fn@Foo`, and `trait@Foo`. Function and macro syntax can also disambiguate references.

Resolution uses Rust language semantics, allowing documentation to remain connected when source structures change, provided the referenced API remains accessible.

Important limits remain. Fully qualified syntax such as `<Vec as IntoIterator>::into_iter()` is not generally supported as an intra-doc link target, and associated items supplied through blanket trait implementations have documented resolution limitations.

The `rustdoc::broken_intra_doc_links` lint can detect unresolved or ambiguous references, making intra-doc links substantially more maintainable than manually authored HTML paths.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://doc.rust-lang.org\&sz=32)

The rustdoc book



Source: `https://doc.rust-lang.org/rustdoc/write-documentation/linking-to-items-by-name.html`

### 4.2 Search by API name and approximate type signature

Rustdoc's generated documentation includes a client-side search engine with two substantially different retrieval mechanisms.

Name and path search supports qualified names, fuzzy matching, crate filtering, and lookup through alternative names declared using `#[doc(alias = "...")]`. The alias mechanism is useful for exposing terminology used by other languages, libraries, standards, or domains without modifying Rust's actual API.

Type-signature search is more distinctive. It can find functions by argument types, return types, and approximate combinations of both. Queries such as `usize -> vec` and `option<T>, fnonce -> option<U>` can locate APIs even when the function name is unknown.

Type-search capabilities also include parameter and return-type discovery, while some non-function items, such as struct fields, can be represented through function-like search semantics.

This constitutes a lightweight, compiler-informed API discovery engine. However, it is not equivalent to trait solving or complete Rust type unification. Its matching is approximate, and semantic distinctions such as some aliasing, trait relationships, lifetimes, and array lengths are not fully modeled in search.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://doc.rust-lang.org\&sz=32)

The rustdoc book



Source: `https://doc.rust-lang.org/rustdoc/read-documentation/search.html`

### 4.3 Trait implementations as navigable API relationships

Rustdoc aggregates and displays inherent methods, trait implementations, associated items, provided methods, blanket implementations, and compiler-derived auto-trait implementations.

This is especially significant in Rust because an object's usable functionality is not restricted to methods declared directly in its defining module.

The resulting documentation can expose:

Inherent functionality: Methods and associated functions defined directly on a type.

Trait-mediated functionality: APIs accessible through implemented traits, including provided trait methods.

Derived applicability: Auto-trait and blanket-implementation relationships that extend functionality beyond explicitly authored impl blocks.

These relationships are useful for discovering extension traits and understanding which behaviors become available through generic implementations. They also explain why a documentation-derived method graph should preserve the provenance and kind of each implementation instead of treating every displayed method as locally declared.

## 5. Executable documentation and doctest infrastructure

Rustdoc treats examples as potentially executable tests. This creates a direct connection between API explanation and compiler verification.

### 5.1 Compilation and execution semantics

`rustdoc --test` and `cargo test --doc` extract eligible Rust code blocks from documentation, compile them, and run them unless the block's attributes specify otherwise.

Rustdoc preprocesses examples by adding selected lint allowances, optionally injecting `extern crate`, and creating a `main` wrapper when required. A documentation block therefore need not look like a complete Rust binary to be testable.

Lines prefixed with `# ` participate in compilation but are omitted from displayed documentation. This allows authors to keep required setup, imports, and scaffolding executable while presenting a smaller focused example. These hidden lines remain real test code, not comments.

Doctest success normally requires both successful compilation and execution without an unexpected panic.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://doc.rust-lang.org\&sz=32)

The rustdoc book



### 5.2 Doctest modes and assertions

| Mechanism                                   | Verification behavior                   | Important distinction                                         |
| ------------------------------------------- | --------------------------------------- | ------------------------------------------------------------- |
| Default Rust block                          | Compile and run                         | Validates executable behavior                                 |
| `no_run`                                    | Compile without executing               | Suitable when execution requires an unavailable environment   |
| `compile_fail`                              | Require compilation failure             | Does not ordinarily establish which particular error occurred |
| `should_panic`                              | Require a runtime panic                 | Tests an expected failure during execution                    |
| `ignore`                                    | Skip the doctest                        | Provides no verification while ignored                        |
| `ignore-...`                                | Ignore for matching target environments | Allows target-sensitive selection                             |
| `edition2018`, `edition2021`, `edition2024` | Select code-block compilation edition   | Can differ from the surrounding crate's edition               |
| `standalone_crate`                          | Prevent doctest merging                 | Preserves isolation-sensitive compilation details             |

A subtle distinction concerns negative examples. `compile_fail` verifies that code is rejected, but an unrelated compiler error can satisfy that requirement. Nightly supports error-number-qualified compile-failure tests, although the documentation warns against treating diagnostic error numbers as a durable, exhaustive contract.

Crate-level controls such as `#![doc(test(attr(...)))]` and `#![doc(test(no_crate_inject))]` further influence doctest compilation.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://doc.rust-lang.org\&sz=32)

The rustdoc book



### 5.3 Rust 2024 doctest merging

Starting with the 2024 edition, rustdoc can merge compatible doctests into a shared compilation unit, reducing repeated compiler initialization and compilation work.

The individual doctests still execute in their own processes. Compilation sharing therefore does not mean they share process state.

The `standalone_crate` annotation prevents merging when an example depends on standalone compilation characteristics, such as precise source-line behavior.

This is a notable native optimization: it reduces doctest compilation overhead without requiring an external test-batching framework. The size of the benefit depends on the crate's doctest workload.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://doc.rust-lang.org\&sz=32)

The rustdoc book



### 5.4 Test extraction and specialized execution

Rustdoc supports test filtering and harness-related options, test arguments, execution-directory controls, and custom runner mechanisms. These are relevant for cross-target testing and environments where compiled tests must execute through an external runtime.

Separately, the nightly `--output-format doctest` mode emits structured information about extracted tests, including original source, transformed code, execution attributes, and locations.

This doctest JSON is a separate schema from rustdoc's API JSON. A consumer of `rustdoc-types::Crate` cannot assume it will deserialize doctest extraction output.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://doc.rust-lang.org\&sz=32)

The rustdoc book



Sources:

`https://doc.rust-lang.org/rustdoc/write-documentation/documentation-tests.html`

`https://doc.rust-lang.org/rustdoc/unstable-features.html`

## 6. Documentation validation, linting, and coverage

### 6.1 Documentation-specific lints

Rustdoc has its own lint infrastructure in addition to lints supplied by rustc.

Important checks include `rustdoc::broken_intra_doc_links`, `rustdoc::private_intra_doc_links`, `rustdoc::invalid_codeblock_attributes`, `rustdoc::invalid_rust_codeblocks`, `rustdoc::invalid_html_tags`, `rustdoc::bare_urls`, and other documentation-format diagnostics.

The distinction between broken and private links matters. A private target may resolve correctly during analysis yet remain inaccessible in the published public documentation.

The `missing_docs` lint is a compiler lint rather than exclusively a rustdoc lint. Accordingly, it can be used outside rustdoc execution, whereas most `rustdoc::` lints operate only when rustdoc runs.

Lint levels support the normal Rust mechanisms—`allow`, `warn`, `deny`, and `forbid`—making documentation correctness enforceable through existing compiler diagnostic workflows.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://doc.rust-lang.org\&sz=32)

The rustdoc book



Source: `https://doc.rust-lang.org/rustdoc/lints.html`

### 6.2 Check-only execution

The unstable `--check` option performs documentation checking without generating documentation or executing doctests.

This separates validation from rendering, potentially avoiding output-file generation and HTML rendering when only diagnostics are required.

It does not replace doctest execution: the check-only path verifies a different set of conditions.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://doc.rust-lang.org\&sz=32)

The rustdoc book



### 6.3 Documentation and example coverage

The unstable `--show-coverage` option reports documentation coverage, including item counts and documentation-example counts.

Coverage calculation has defined inclusion rules. It primarily considers local-crate items; reexports of external items do not count as local definitions, and certain kinds of items have special rules for code-example coverage.

The feature can emit textual reports or coverage-specific JSON.

It is therefore useful for finding documentation gaps, but coverage percentages are not direct measures of documentation accuracy or API comprehensiveness. An item with a nonempty docstring can count as documented regardless of the explanatory quality of that text.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://doc.rust-lang.org\&sz=32)

The rustdoc book



Source: `https://doc.rust-lang.org/rustdoc/unstable-features.html`

## 7. HTML generation, customization, and publishing

### 7.1 Static documentation output

Rustdoc generates static HTML, CSS, JavaScript, and related assets. Output includes module and item pages, linked signatures, impl sections, generated source views, search data, and cross-references.

The output is designed for hosting without a server-side Rust process. Search and interactive navigation operate in the browser using generated assets.

Rustdoc can also render standalone Markdown files, although this is a separate input mode from compiling a crate and does not provide the same compiler-derived item graph.

### 7.2 Presentation and content controls

The stable command-line surface includes `--theme`, `--check-theme`, `--extend-css`, `--html-in-header`, `--html-before-content`, and `--html-after-content`.

Crate-level attributes provide additional capabilities, including `html_logo_url`, `html_favicon_url`, `html_playground_url`, `html_no_source`, and `html_root_url`.

These operate at different layers. Themes and stylesheet extensions modify presentation; HTML injection options add arbitrary page content; source-disabling controls affect generated source browsing; and playground configuration connects runnable examples to an external execution service.

A custom theme is not an independent rustdoc renderer. It modifies rustdoc's established HTML structure, which can evolve between compiler releases.

Sources:

`https://doc.rust-lang.org/rustdoc/command-line-arguments.html`

`https://doc.rust-lang.org/rustdoc/write-documentation/the-doc-attribute.html`

### 7.3 External-crate linking

Rustdoc can link symbols to documentation from other crates, including crates not being regenerated in the current invocation.

The linking decision favors locally generated documentation when available, then an explicit `--extern-html-root-url` override if provided, then the external crate's `html_root_url` metadata.

The command-line `--extern-html-root-url` override remains unstable; the `html_root_url` documentation attribute provides an established way for a crate to declare its preferred external documentation location.

This allows multi-crate documentation to preserve navigability without copying every dependency's generated HTML into one output tree. External link quality nevertheless depends on available crate metadata and valid published documentation locations.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://doc.rust-lang.org\&sz=32)

The rustdoc book



### 7.4 Cargo integration

The distinction between `cargo doc` and `cargo rustdoc` is operationally important.

| Cargo command                         | Principal behavior                                                            | Implication                                                            |
| ------------------------------------- | ----------------------------------------------------------------------------- | ---------------------------------------------------------------------- |
| `cargo doc`                           | Builds documentation for selected package targets and dependencies by default | Suitable for normal workspace and dependency documentation             |
| `cargo doc --no-deps`                 | Documents selected packages without generating dependency docs                | Reduces documentation work when dependency HTML is unnecessary         |
| `cargo doc --all-features`            | Activates all available selected-package features                             | Changes the documented configuration; may expose otherwise absent APIs |
| `cargo rustdoc`                       | Invokes rustdoc on a selected target with additional custom arguments         | Suitable for specialized output modes and experimental flags           |
| `RUSTDOCFLAGS` / `build.rustdocflags` | Configure rustdoc invocations originating from Cargo                          | Applies flags more broadly than a single target's extra arguments      |

`cargo rustdoc` does not generate dependency documentation, and additional arguments require a single selected target. `cargo doc` output is cumulative: existing generated files may remain across invocations, so the contents of an output directory are not necessarily a precise record of only the latest build.

Cargo also controls relevant package, feature, target, manifest, and profile selection. Those build settings directly influence what rustdoc can discover.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://doc.rust-lang.org\&sz=32)

The Cargo Book+1



Sources:

`https://doc.rust-lang.org/cargo/commands/cargo-doc.html`

`https://doc.rust-lang.org/cargo/commands/cargo-rustdoc.html`

### 7.5 docs.rs integration

`docs.rs` is a separate documentation-building and hosting service, not part of rustdoc itself.

It supports build customization through `[package.metadata.docs.rs]`, including feature selection, default targets, additional targets, Cargo arguments, and rustdoc arguments.

This matters for documentation reproducibility: the docs.rs presentation of a crate can differ from documentation generated with Cargo defaults or a different compiler release.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

docs.rs



Source: `https://docs.rs/about/metadata`

## 8. Rustdoc JSON: structured API and relationship extraction

This is rustdoc's most consequential interface for programmatic consumers.

### 8.1 Status and generation

The `--output-format json` backend emits a machine-readable representation of a crate's documented API. As of this research, it remains experimental and nightly-gated.

A representative invocation is `cargo +nightly rustdoc --lib -- -Z unstable-options --output-format json`.

The generated output is ordinarily written under `target/doc` as a crate-named `.json` file. It can be deserialized into `rustdoc_types::Crate` using Serde.

The underlying output schema is not covered by Rust's normal stable-API compatibility guarantees. A program consuming it must explicitly handle schema-version differences.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://doc.rust-lang.org\&sz=32)

The rustdoc book+1



Sources:

`https://doc.rust-lang.org/rustdoc/unstable-features.html`

`https://docs.rs/rustdoc-types/0.61.0/rustdoc_types/`

### 8.2 The top-level `Crate` representation

The `rustdoc_types::Crate` structure is organized around identifiers, item definitions, and cross-reference metadata.

| Field                             | Meaning                                                  | Important semantic detail                                        |
| --------------------------------- | -------------------------------------------------------- | ---------------------------------------------------------------- |
| `root: Id`                        | Root module item                                         | Entry point for traversing the crate hierarchy                   |
| `index: HashMap<Id, Item>`        | Local items and selected referenced external trait items | Not a complete database of every dependency's declarations       |
| `paths: HashMap<Id, ItemSummary>` | Item paths and classification metadata                   | Useful for linking; not a complete public-reexport path registry |
| `external_crates`                 | External crate identifiers, names, root URLs             | Supports attribution and cross-crate references                  |
| `crate_version`                   | Version string supplied to rustdoc                       | Optional; not independently inferred from a package registry     |
| `includes_private`                | Whether private items were included                      | Important extraction-context metadata                            |
| `target`                          | Target information for the generated documentation       | The represented API depends on target configuration              |
| `format_version`                  | Breaking-schema version                                  | Primary compatibility check for consumers                        |

The `index` and `paths` maps serve different purposes. The former contains rich item information; the latter provides link-oriented summaries, especially for external items that lack complete local definitions.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust



Source: `https://docs.rs/rustdoc-types/0.61.0/rustdoc_types/struct.Crate.html`

### 8.3 The common `Item` abstraction

Each `Item` provides shared metadata independently of its Rust item kind.

Important fields include `id`, `crate_id`, `name`, `span`, `visibility`, `docs`, `links`, `attrs`, `deprecation`, `stability`, `const_stability`, and `inner`.

Several distinctions are easily overlooked.

Documentation presence has more than two states. `docs` is optional: `None` means no documentation was supplied, while `Some("")` represents documentation that exists but is empty. This difference can matter for coverage analysis or transformations.

Source locations are optional. Macro-generated items and some other constructs can lack an ordinary span. A program requiring precise file ranges cannot assume that every item has a concrete source location.

Intra-doc links are already resolved. The `links` map connects documentation references to item identifiers, providing semantic relationships without independently resolving all Markdown references.

Visibility is explicit, but reachability is contextual. A declaration's visibility alone does not reconstruct every public exposure path. Reexport and module relationships remain necessary.

Stability metadata is not a universal SemVer classifier. Compiler stability attributes, deprecation metadata, and a library author's compatibility policy are different concepts. Missing `stability` information does not mean that an API is formally guaranteed stable.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust



Source: `https://docs.rs/rustdoc-types/0.61.0/rustdoc_types/struct.Item.html`

### 8.4 Item kinds and the structural API model

The `ItemEnum` discriminator models a broad range of declarations and relationships.

| Family                  | Representative variants                             | Information made available                                   |
| ----------------------- | --------------------------------------------------- | ------------------------------------------------------------ |
| Namespace structure     | `Module`, `Use`, `ExternCrate`                      | Containment, imports, reexports                              |
| Concrete data types     | `Struct`, `Union`, `Enum`, `Variant`, `StructField` | Fields, variants, representation-related structure           |
| Callable APIs           | `Function`, `Macro`, `ProcMacro`                    | Function signatures and macro classifications                |
| Trait system            | `Trait`, `TraitAlias`, `Impl`                       | Trait definitions, bounds, implementations, associated items |
| Type-level expressions  | `TypeAlias`, `Primitive`, `ExternType`              | Type names and representations                               |
| Values and associations | `Constant`, `Static`, `AssocConst`, `AssocType`     | Types, values or defaults where represented                  |

This representation can support much richer analysis than extracting public function signatures alone.

For example, it allows consumers to distinguish concrete type declarations from aliases, inspect associated types, identify trait implementations, and traverse generic constraints.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust



Source: `https://docs.rs/rustdoc-types/0.61.0/rustdoc_types/enum.ItemEnum.html`

### 8.5 Type representation

The `Type` enum includes forms such as `ResolvedPath`, `Generic`, `Primitive`, `DynTrait`, `ImplTrait`, `FunctionPointer`, `Tuple`, `Array`, `Slice`, `BorrowedRef`, `RawPointer`, and `QualifiedPath`.

Additional structures represent generic arguments, parameters, bounds, and where clauses.

This makes it possible to construct a structured model of API type signatures without reparsing formatted Rust source.

The distinction between `ResolvedPath` and an ordinary textual type path is important. A resolved path can retain semantic identity and structured generic arguments, while preserving only a formatted signature would lose much of that structure.

Nevertheless, `rustdoc-types::Type` should not be mistaken for rustc's complete internal type system. It is a serialized, documentation-oriented representation. For example, some compile-time expressions are represented textually rather than as a complete executable semantic model.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust



Source: `https://docs.rs/rustdoc-types/0.61.0/rustdoc_types/enum.Type.html`

### 8.6 Trait and implementation relationships

The `Impl` structure is unusually valuable for capability analysis.

Its fields include `trait_`, `for_`, `generics`, `items`, `provided_trait_methods`, `is_negative`, `is_synthetic`, `blanket_impl`, and `is_unsafe`.

These distinguish several materially different relationships.

| Implementation classification              | How represented             | Why it matters                                                                  |
| ------------------------------------------ | --------------------------- | ------------------------------------------------------------------------------- |
| Inherent implementation                    | `trait_` is absent          | Methods belong directly to the type's inherent API                              |
| Explicit trait implementation              | `trait_` identifies a trait | Connects a type to a trait contract                                             |
| Compiler-implied auto-trait implementation | `is_synthetic`              | Not necessarily an impl block declared in source                                |
| Negative implementation                    | `is_negative`               | Records a prohibited or explicitly negated trait relationship                   |
| Blanket implementation                     | `blanket_impl`              | Describes generic applicability rather than an isolated concrete implementation |
| Provided trait methods                     | `provided_trait_methods`    | Identifies methods available through trait defaults without local overrides     |

A documentation-derived relationship graph can therefore distinguish directly implemented capabilities from functionality obtained through trait machinery.

However, determining whether a particular generic method call is valid under arbitrary substitutions can still require full compiler reasoning. The existence of an `Impl` record is not a replacement for rustc's trait solver.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust



Source: `https://docs.rs/rustdoc-types/0.61.0/rustdoc_types/struct.Impl.html`

### 8.7 Identity, paths, and reexports

Rustdoc JSON supplies opaque item identifiers for relationships within the emitted representation. Those identifiers should not be assumed to constitute persistent, cross-build identities suitable for long-term databases.

`ItemSummary.path` is also not a canonical catalog of every public import path. The official schema documents that an item may be available through several paths and that the selected path is implementation-defined.

Consequently, an API-analysis system generally needs to distinguish:

Definition identity: The represented language item.

Exposure identity: The public path or reexport through which downstream users access that item.

Build context: The crate version, target, features, toolchain, and extraction settings under which the representation was produced.

These are different dimensions of API identity. Collapsing them into one string path can create false additions, removals, or equivalences when comparing releases.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust



Source: `https://docs.rs/rustdoc-types/0.61.0/rustdoc_types/struct.ItemSummary.html`

### 8.8 External-crate information is intentionally incomplete

Rustdoc JSON is generated for a particular crate. It contains enough information about external items to construct relevant references and relationships, but it does not serialize every dependency's entire internal type graph.

The `index` may include selected external traits and associated items that are needed locally, while `paths` contains summaries for other references.

A complete multi-crate documentation graph therefore requires additional crate representations and identity reconciliation. Rustdoc JSON provides the data needed for meaningful cross-crate references, but not a finished, unified dependency-wide knowledge graph.

## 9. Programmatic integration and companion libraries

### 9.1 `rustdoc-types`: the schema boundary

`rustdoc-types` supplies Rust definitions and Serde serialization/deserialization for the experimental JSON schema. Its purpose is to make structured output usable without independently maintaining a large collection of JSON variant definitions.

Version 0.61.0 corresponds to schema version 61, with the schema version recorded in its changelog.

The crate also offers optional performance-oriented integrations, including `rustc-hash` for faster hash-map implementations in some workloads and `rkyv` integration for archived representations.

These are consumer-side data-structure capabilities. They do not change the upstream rustdoc JSON format.

The `rustc-hash` feature is most relevant to workloads that repeatedly deserialize and query large documentation graphs; expected benefit depends on the balance of parsing, hashing, traversal, and other work.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Docs.rs+1



Sources:

`https://docs.rs/rustdoc-types/0.61.0/rustdoc_types/`

`https://docs.rs/crate/rustdoc-types/0.61.0/source/CHANGELOG.md`

### 9.2 `rustdoc-json`: generation orchestration

The separately maintained `rustdoc-json` crate provides `rustdoc_json::Builder`, which wraps the Cargo and rustdoc invocation process for generating structured documentation.

Its responsibilities include choosing the toolchain, selecting a manifest or package target, supplying generation options, and returning the generated JSON path.

This can eliminate custom subprocess orchestration when a Rust application needs to generate documentation data as part of a larger analysis workflow.

It does not itself turn rustdoc's experimental JSON contract into a stable one.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust



Source: `https://docs.rs/rustdoc-json/latest/rustdoc_json/`

### 9.3 `public-api`: public-surface extraction and comparison

The `public-api` crate consumes rustdoc JSON and produces a higher-level public API listing. It also provides API-diff functionality, with `cargo public-api` offering a related CLI interface.

Its distinction from `rustdoc-types` is that it supplies an interpreted public API abstraction, rather than merely exposing the serialized documentation schema.

This is useful when the required result is a list of externally usable symbols or changes between versions, rather than arbitrary queries over the entire rustdoc JSON representation.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust



Source: `https://docs.rs/public-api/latest/public_api/`

### 9.4 `cargo-semver-checks`: compatibility interpretation

`cargo-semver-checks` uses rustdoc JSON for semantic-versioning analysis and compatibility lints.

This illustrates a further layer above raw extraction: determining which changes are potentially breaking requires interpreting Rust-specific relationships, not merely finding additions and deletions in a JSON object.

Changes to trait implementations, generic bounds, associated items, reexport paths, and public type signatures can have different compatibility implications.

Rustdoc JSON supplies much of the necessary structural input, but compatibility analysis requires additional rules and assumptions about supported configurations.

Source: `https://github.com/obi1kenobi/cargo-semver-checks`

### 9.5 docs.rs-hosted JSON as an alternative data source

Since May 23, 2025, docs.rs has built and hosted rustdoc JSON for published crates.

Published JSON can be accessed through routes such as:

`https://docs.rs/crate/clap/latest/json`

`https://docs.rs/crate/clap/latest/json.gz`

`https://docs.rs/crate/clap/latest/json/42`

By default, downloads use Zstandard compression, with gzip alternatives available. Target-specific URLs are also supported.

This materially changes programmatic workflows: a tool exploring published libraries can often retrieve precomputed API representations instead of compiling each crate locally.

However, older releases may not have JSON until rebuilt, and existing hosted artifacts can use earlier schema versions. The `format_version` field must remain the authority for parsing compatibility.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

docs.rs



Source: `https://docs.rs/about/rustdoc-json`

## 10. Advanced and experimental capabilities

Rustdoc has several less obvious features that deserve separate attention because they can replace substantial custom tooling.

### 10.1 Automated extraction of real-world usage examples

The unstable scraped examples feature analyzes Cargo example targets, identifies call sites of documented APIs, and attaches relevant code snippets to their documentation pages.

It can therefore surface actual usages of functions without requiring all examples to be manually copied into doc comments.

Rustdoc limits the number of snippets shown directly, links to additional occurrences, and orders examples to favor smaller excerpts.

This is a different capability from doctests. Doctests validate examples written in documentation; scraped examples discover usages from other code and present them in documentation.

The mechanism can be useful where API comprehension benefits from concrete call-site context. Its coverage is restricted by the sources and targets analyzed and should not be treated as exhaustive reference discovery.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://doc.rust-lang.org\&sz=32)

The rustdoc book



Source: `https://doc.rust-lang.org/rustdoc/scraped-examples.html`

### 10.2 Source-code links to definitions

The unstable `--generate-link-to-definition` option adds semantic links on source-code pages, allowing readers to navigate from referenced types to their definitions.

This improves the generated source browser's usefulness for following relationships across declarations.

There is an important tradeoff: generating those links can require semantic analysis of function bodies that rustdoc would otherwise omit. The official documentation warns that this can cause some conditionally documented code to be rejected when enabling the feature.

It is therefore not merely a harmless presentation toggle.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://doc.rust-lang.org\&sz=32)

The rustdoc book



### 10.3 Compiler-computed type layouts

The unstable `--show-type-layout` option adds memory-layout information to type documentation, derived from rustc.

It can expose information such as computed sizes and layout characteristics for the documented target and compilation context.

This provides a compiler-informed inspection mechanism without a bespoke layout-analysis frontend. However, much of Rust's type layout is not specified as stable and may change between compilations or compiler versions.

The output should not be interpreted as a cross-version ABI contract unless independent language or representation guarantees apply.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://doc.rust-lang.org\&sz=32)

The rustdoc book



### 10.4 Shared documentation metadata and scalable multi-crate generation

Rustdoc generates some shared information across crates, including search metadata.

Historically, merging shared metadata during every crate's documentation generation could require work proportional to the growing documentation collection. The rustdoc documentation explicitly describes this as potentially quadratic in the number of crates.

The unstable `--write-doc-meta-dir` and `--read-doc-meta-dir` mechanisms separate per-crate metadata generation from a later finalization phase.

This changes the workflow from repeated aggregate merging to independent production followed by a final merge. It enables more efficient orchestration of large multi-crate documentation builds, although intermediate output may be incomplete until finalization.

The important capability is separation of per-crate work from shared-index assembly. That can reduce redundant coordination and provide a more natural boundary for parallel build systems.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://doc.rust-lang.org\&sz=32)

The rustdoc book



### 10.5 Additional experimental presentation and inspection features

Other noteworthy nightly facilities include `--document-hidden-items`, which changes whether hidden items appear; `#[doc(notable_trait)]`, which enhances how important trait relationships are presented; `#[doc(masked)]`, which suppresses selected external implementation information; and `--persist-doctests`, which retains compiled doctest executables for further inspection.

Hosting-oriented options include `--resource-suffix` and `--static-root-path`, which control static asset naming and placement. These can matter for cache management and serving large documentation collections.

The `rust-docs-json` rustup component also provides JSON documentation artifacts for toolchain crates such as `std`, `core`, `alloc`, `proc_macro`, and `test` on nightly, allowing analysis tools to inspect part of the standard-library API without regenerating it themselves.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://doc.rust-lang.org\&sz=32)

The rustdoc book+2



Source: `https://doc.rust-lang.org/rustdoc/unstable-features.html`

## 11. Important distinctions from adjacent Rust analysis technologies

Rustdoc's value is easier to understand when contrasted with tools that operate at different semantic layers.

| Technology                         | Primary abstraction                       | What rustdoc supplies that it does not                                                          | What rustdoc does not replace                                             |
| ---------------------------------- | ----------------------------------------- | ----------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------- |
| `syn`, `ra_ap_syntax`, Tree-sitter | Parsed syntax                             | Compiler-informed resolved documentation relationships, reachable API surface, impl aggregation | Complete syntax preservation, inexpensive parsing, syntax transformations |
| `ra_ap_hir` / rust-analyzer        | IDE-oriented semantic model               | A ready-made publication-oriented API representation, documentation formatting and doctests     | Interactive resolution, incremental IDE analysis, rich source navigation  |
| `rustc` internals                  | Compiler HIR, types, MIR, query system    | A higher-level documentation projection and public documentation pipeline                       | Complete compiler semantics, function-body analysis, borrow checking, MIR |
| `cargo metadata`                   | Packages, dependencies, features, targets | Language-item declarations, signatures, impls, documentation                                    | Dependency resolution and package graph metadata                          |
| `rustdoc-types`                    | Serialized rustdoc schema                 | The generation, transformation, and rendering engine itself                                     | A stable compiler API or independent semantic analyzer                    |
| `public-api`                       | Interpreted external API inventory        | Broader raw documentation information and HTML/doctest functionality                            | General-purpose documentation-graph exploration                           |
| `cargo-semver-checks`              | Compatibility analysis                    | Source API data used to inform compatibility rules                                              | A complete compatibility decision based on structural differences alone   |

The distinction is not simply between shallow and deep analysis.

Rustdoc provides a valuable middle-level abstraction: more semantically informative than a syntax tree, substantially more specialized and accessible than compiler internals, but less complete and interactive than a full compiler or language-server analysis environment.

Its greatest utility is often in precisely that middle ground, where the objective is to understand a library's exposed capabilities rather than execute arbitrary compiler queries.

## 12. Cross-cutting limitations and interpretation hazards

### 12.1 The represented API is configuration-dependent

Generated documentation reflects the chosen feature set, target, active `cfg` values, and documentation-specific configuration.

A JSON representation is therefore not automatically the universal API of a crate across all supported targets and optional features.

`#[cfg(doc)]` may broaden visibility, but it is not a complete replacement for evaluating several genuine compilation configurations.

### 12.2 Documentation visibility is not the same as language visibility

`#[doc(hidden)]` can exclude publicly accessible items from normal documentation.

Similarly, an item visible in documentation may be conditionally unavailable in ordinary builds.

An inventory of documented items cannot, without further interpretation, claim to enumerate every symbol that downstream code might use.

### 12.3 Definition paths are not the complete namespace

Rust permits reexports, renaming, and access through multiple paths.

The JSON `paths` map is useful for linking but cannot by itself represent every public exposure relationship.

API diffing and documentation indexes that care about import paths must account for reexport semantics.

### 12.4 Compiler-derived implementations require classification

Blanket and synthetic auto-trait implementations can appear in documentation even without directly corresponding source impl declarations.

Treating all implementations as identical edges loses meaningful distinctions between explicit behavior, generic applicability, and inferred compiler relationships.

### 12.5 Structured documentation is not source reconstruction

Rustdoc JSON is not designed for round-tripping into equivalent Rust source.

Function bodies, precise macro-expansion provenance, all source tokens, compiler control-flow representations, and some semantic details are outside its intended model.

For source-preserving transformations or function-body analysis, a parser, rust-analyzer, or compiler interface may remain necessary.

### 12.6 JSON stability is separate from Rust toolchain stability

Stable HTML documentation support does not imply a stable JSON schema.

The published `rustdoc-types` version identifies a particular schema, while actual emitted output depends on the nightly compiler version.

Schema incompatibility should be treated as an expected integration condition rather than an exceptional accident.

### 12.7 Tool-generated documentation does not guarantee executable completeness

Rustdoc can document items admitted through `cfg(doc)` without performing every compiler check applicable to normal builds.

Doctests likewise use their own compilation and execution settings.

Passing documentation generation, passing doctests, and passing ordinary crate compilation establish different properties.

## 13. Architectural significance

Rustdoc is particularly valuable where the desired representation is a semantically resolved, externally oriented view of a Rust library.

Its strongest native combinations are worth recognizing.

API inventory and relationship modeling. JSON output, item identifiers, reexports, type representations, and implementation records together provide the basis for an API relationship graph. This avoids having to reimplement substantial Rust language-resolution behavior.

Semantic library discovery. Type-signature search, documentation aliases, compiler-resolved links, and trait presentation support discovery based on functionality and relationships rather than just lexical symbol names.

Documentation as verifiable interface. Compiler-resolved references, documentation lints, doctests, negative compilation tests, and coverage analysis establish several complementary ways to detect stale or inconsistent documentation.

Cross-version capability analysis. Structured output, externally hosted JSON, public-surface extraction, and compatibility tooling support release-to-release comparison, provided build configuration and schema compatibility are handled explicitly.

Scalable multi-crate reference generation. Cargo orchestration, external-crate links, optional metadata finalization, and static assets allow documentation for multiple related crates to be generated and served without constructing an entirely separate documentation engine.

The main architectural choice is the required level of semantic fidelity. For documentation and public API understanding, rustdoc offers an unusually capable compiler-derived abstraction. For arbitrary source analysis, incremental IDE functionality, or exact compiler semantics, its documentation-oriented projection imposes real boundaries.

### Key source entry points

| Investigation area                                      | Primary source                                                 |
| ------------------------------------------------------- | -------------------------------------------------------------- |
| Rustdoc's documented features and command-line contract | `https://doc.rust-lang.org/rustdoc/`                           |
| Compiler integration and internal architecture          | `https://rustc-dev-guide.rust-lang.org/rustdoc-internals.html` |
| Structured API JSON and nightly-only capabilities       | `https://doc.rust-lang.org/rustdoc/unstable-features.html`     |
| Versioned JSON data structures                          | `https://docs.rs/rustdoc-types/0.61.0/rustdoc_types/`          |
| Hosted JSON for published crates                        | `https://docs.rs/about/rustdoc-json`                           |
| Programmatic JSON generation                            | `https://docs.rs/rustdoc-json/latest/rustdoc_json/`            |
| Public API inventory and diffing                        | `https://docs.rs/public-api/latest/public_api/`                |
| Semantic-version compatibility analysis                 | `https://github.com/obi1kenobi/cargo-semver-checks`            |

Overall assessment: Rustdoc is best understood as Rust's compiler-backed documentation and API-projection subsystem. Its HTML renderer and doctest engine are mature, stable functionality; its JSON backend is an exceptionally useful but still experimental extraction interface. The distinction between the underlying language model, the cleaned documentation model, and the published API surface is essential to using its capabilities correctly.