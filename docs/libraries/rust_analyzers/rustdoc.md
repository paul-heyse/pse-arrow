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

# rustdoc — Advanced Capabilities and Schema Semantics Addendum

Research date: November 1, 2026\
Version scope: Rust 1.99.1; `rustdoc-types` 0.61.0; rustdoc JSON format version 61\
Companion document: rustdoc — Comprehensive Rust Library and Tooling Reference, October 29, 2026

## Scope

This addendum extends the original guide with functionality and semantic distinctions that were missing or insufficiently developed. It concentrates on the deeper behavior of rustdoc's compiler integration, structured API representation, executable documentation, source inspection, and build orchestration.

The most consequential additions are:

- Compiler-analysis boundaries: Rustdoc's deliberate avoidance of ordinary function-body type-checking and the consequences for completeness.
- Structured API intelligence: Typed attributes, representation metadata, incomplete-item indicators, advanced type and trait properties, stability dimensions, and cross-crate identity.
- Executable documentation: Doctest-specific compilation, test-harness selection, external execution, compiler customization, and structured test extraction.
- Documentation infrastructure: Dependency tracking, shared search and implementation metadata, macro inspection, expanded linting, and additional Markdown facilities.

The addendum retains the original distinction between stable rustdoc functionality, nightly-only facilities, and the experimental rustdoc JSON contract. It uses the original document's section numbering as a reference without repeating its existing explanations.

## 1. Compiler-analysis boundaries and semantic completeness

Extends original §§2.1–2.3, 10.2, and 12.5–12.7.

### 1.1 Deliberate avoidance of ordinary function-body type-checking

Rustdoc's use of rustc's name resolution, HIR, and type-query infrastructure does not imply that it performs the same semantic analysis as an ordinary crate compilation.

The Rust compiler development guide explicitly documents that rustdoc does not normally type-check function bodies. Internally, this is accomplished through specialized compiler-query behavior, suppression of certain name-resolution errors, and avoiding ordinary resolution of opaque types.

This has several consequences:

1. Rustdoc can successfully document certain configurations that would fail normal compilation.
2. Compiler queries dependent on fully type-checked bodies may be unavailable or incomplete.
3. Rustdoc cannot ordinarily produce `.rlib` artifacts or execute most compiler lints that require body type-checking.
4. Successful documentation generation cannot establish the correctness of method calls, control flow, or all generic interactions inside function bodies.

This distinction is narrower and more precise than simply saying rustdoc omits function bodies from its JSON representation. Rustdoc intentionally operates under a different compiler-analysis regime.

The tradeoff is architectural: rustdoc can construct useful API documentation from configurations selected for documentation purposes without requiring every function body to pass all the checks performed by an ordinary build.

### 1.2 Source-navigation features can change semantic validation behavior

The nightly `--generate-link-to-definition` option introduces an important exception.

Generating semantic links from references in function bodies to their definitions can require resolving type-dependent expressions, including method calls. This may cause rustdoc to type-check bodies it would otherwise avoid.

Consequently, enabling this feature can introduce name-resolution or type errors in a crate that otherwise generates documentation successfully.

The significant distinction is that a source-navigation enhancement can change the compilation checks performed, rather than merely modifying the HTML output.

The related `--generate-macro-expansion` capability, discussed later, provides a different source-inspection mechanism and should not be assumed to have identical semantic-analysis requirements.

### 1.3 Implications for compiler-derived API representations

Rustdoc's output should be interpreted as the combination of three different properties:

| Property                   | What rustdoc can establish                                                                      | What it does not establish                                               |
| -------------------------- | ----------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------ |
| Declaration semantics      | Resolved declarations, many type relationships, and compiler-derived implementation information | Complete behavior of function bodies                                     |
| Documentation reachability | Items represented under the selected documentation configuration                                | Every item available under every ordinary compilation configuration      |
| Documentation validity     | Documentation parsing, supported link resolution, selected lints, and optionally doctests       | Full correctness of the underlying crate across all build configurations |

For code-intelligence consumers, this means the absence of body-level relationships from rustdoc JSON is an intentional limitation of the projection, not evidence that those relationships do not exist.

Combining rustdoc JSON with rust-analyzer or compiler-level analysis can provide additional information, but the sources may differ in compilation assumptions, configuration selection, and analysis completeness.

Primary sources

- Rustdoc compiler architecture and body-type-checking limitation: `https://rustc-dev-guide.rust-lang.org/rustdoc.html`
- Nightly source-navigation features and validation caveats: `https://doc.rust-lang.org/rustdoc/unstable-features.html`

[image](https://www.google.com/s2/favicons?domain=https://rustc-dev-guide.rust-lang.org\&sz=32)

Rust Compiler Development Guide+1

## 2. Advanced rustdoc JSON semantics

Extends original §§8.2–8.8, 9.1, and 12.

The experimental rustdoc JSON schema contains more compiler-derived information than item names, signatures, generics, and implementation relationships. Several additional fields are particularly relevant to API intelligence because they preserve distinctions that would otherwise require custom Rust parsing or semantic interpretation.

The field names and shapes discussed below are verified against the tagged `rustdoc-types` 0.61.0 source, not inferred from older schema versions.

Primary schema source: `https://raw.githubusercontent.com/rust-lang/rustdoc-types/v0.61.0/src/lib.rs`

### 2.1 Typed attributes and representation metadata

`Item.attrs` is a `Vec<Attribute>`, rather than merely a collection of unstructured attribute strings.

The schema recognizes several attribute families explicitly:

| Structured attribute       | Information exposed                           | Consequential use                                       |
| -------------------------- | --------------------------------------------- | ------------------------------------------------------- |
| `NonExhaustive`            | Presence of `#[non_exhaustive]`               | Detect construction and exhaustive-pattern restrictions |
| `MustUse { reason }`       | Must-use annotation and optional explanation  | Identify compiler-enforced usage diagnostics            |
| `MacroExport`              | Exported declarative macro attribute          | Distinguish macro export behavior                       |
| `ExportName(String)`       | Explicit external symbol name                 | Inspect exported-symbol declarations                    |
| `NoMangle`                 | Name-mangling suppression                     | Identify symbol-name behavior                           |
| `LinkSection(String)`      | Explicit binary section placement             | Inspect linker-sensitive declarations                   |
| `AutomaticallyDerived`     | Compiler or macro-generated derivation marker | Identify generated implementation provenance            |
| `TargetFeature { enable }` | Function-specific target-feature declarations | Understand instruction-set-specific API requirements    |
| `Repr(AttributeRepr)`      | Structured type representation attributes     | Inspect FFI and layout-related declarations             |
| `Other(String)`            | Attributes without a dedicated schema variant | Preserve otherwise unmodeled metadata                   |

`AttributeRepr` further exposes `kind`, `align`, `packed`, and `int`. The `ReprKind` discriminator identifies Rust, C, transparent, and SIMD representation.

This is materially more useful than searching source text for `#[repr(...)]`. A structured API consumer can distinguish explicit representation contracts, integer discriminant types, and alignment or packing annotations without implementing an attribute parser.

However, these fields describe declared representation attributes, not actual compiler-computed sizes, offsets, or alignment for a particular monomorphized type. `#[repr(C)]` also does not by itself establish that every component of a type is safe or appropriate for FFI.

#### Important stability distinction

`Attribute::Other(String)` has an explicit compatibility exception: its contents are not governed by `FORMAT_VERSION` and may change without a format-version increment.

The fallback may contain either a compiler/HIR-derived pretty-printed representation or source-like formatting. It should not be treated as a canonical token sequence or stable attribute grammar.

Similarly, the schema documents that attributes may be normalized or combined during rendering. `Item.attrs` is therefore suitable for inspecting supported semantic properties, but not for exact source reconstruction.

Other attributes have dedicated fields instead of appearing in `attrs`: documentation comments, deprecation, ordinary stability, const stability, and unstable provided defaults.

Source: `https://docs.rs/rustdoc-types/0.61.0/rustdoc_types/enum.Attribute.html`&#x20;

[image](https://www.google.com/s2/favicons?domain=https://raw.githubusercontent.com\&sz=32)

raw\.githubusercontent.com



### 2.2 Explicit indications of omitted API structure

Rustdoc JSON does not merely remove inaccessible or hidden information. In several important cases it records that information has been removed.

| Schema location       | Indicator               | Meaning                                                                      |
| --------------------- | ----------------------- | ---------------------------------------------------------------------------- |
| `Module`              | `is_stripped`           | Module is not part of the public API, but contains publicly reexported items |
| `StructKind::Tuple`   | `Vec<Option<Id>>`       | Hidden/private tuple fields retain their positions as `None`                 |
| `StructKind::Plain`   | `has_stripped_fields`   | One or more named fields were omitted                                        |
| `Union`               | `has_stripped_fields`   | One or more union fields were omitted                                        |
| `Enum`                | `has_stripped_variants` | One or more variants were omitted                                            |
| `VariantKind::Tuple`  | `Vec<Option<Id>>`       | Hidden tuple-variant fields retain positional placeholders                   |
| `VariantKind::Struct` | `has_stripped_fields`   | One or more named variant fields were omitted                                |

The preservation of tuple-field positions is especially significant. Omitting a hidden field without recording its position would shift the apparent indices of the remaining tuple fields, producing a misleading structural representation.

For named fields and enum variants, the schema generally preserves a completeness indicator rather than the missing members' identities.

This enables a useful distinction among three situations:

- Complete represented structure: All relevant members are present under the selected documentation configuration.
- Known incomplete structure: The schema explicitly reports omitted members or positional placeholders.
- Potentially incomplete projection: No indicator proves completeness across alternative build configurations or excluded documentation surfaces.

For structural API comparisons, these states cannot be treated equivalently. A missing field or variant is not necessarily a removed declaration. Similarly, an empty represented field collection is not necessarily evidence that a type has no inaccessible fields.

This is particularly relevant to questions about external construction, exhaustive matching, and whether a representation has changed. `#[non_exhaustive]` is an additional, distinct restriction and must not be inferred solely from stripped-field indicators.

Sources:

`https://docs.rs/rustdoc-types/0.61.0/rustdoc_types/enum.StructKind.html`

`https://docs.rs/rustdoc-types/0.61.0/rustdoc_types/enum.VariantKind.html`

`https://docs.rs/rustdoc-types/0.61.0/rustdoc_types/struct.Module.html`

[image](https://www.google.com/s2/favicons?domain=https://raw.githubusercontent.com\&sz=32)

raw\.githubusercontent.com+2

### 2.3 Richer callable, trait, and macro properties

Rustdoc JSON can answer several semantic questions without reconstructing the information from a formatted declaration.

Functions. `FunctionHeader` provides `is_const`, `is_unsafe`, `is_async`, and `abi`. The ABI representation also distinguishes relevant unwinding-capable ABI variants.

`FunctionSignature` exposes parameter names/patterns and types, optional return type, and `is_c_variadic`. Parameter names are not guaranteed to be valid identifier strings because source parameters may use patterns.

`Function.has_body` distinguishes a declaration with an implementation from one without a body. This does not mean the JSON stores the implementation's statements or expressions.

Traits. `Trait` exposes `is_auto`, `is_unsafe`, and `is_dyn_compatible`, together with its associated items, bounds, generics, and implementations.

The `is_dyn_compatible` field is particularly valuable because dyn compatibility is a language-semantic property, not merely the presence or absence of a particular syntactic construct. It can be inspected as a compiler-derived property rather than reimplemented through a separate collection of object-safety rules.

Procedural macros. `ProcMacro` distinguishes function-like, attribute, and derive macros through `MacroKind`, and exposes helper attributes for applicable derive macros.

This provides a machine-readable inventory of macro interface forms, but not the procedural macro's expansion behavior or its complete accepted token grammar.

Advanced type-level representations. The schema also models higher-ranked trait bounds, associated-item constraints, associated type projections, function-pointer signatures, and precise capturing arguments used in `impl Trait` bounds such as `use<'a, T>`.

These representations help preserve type-level relationships that would be obscured by stringifying signatures. They remain documentation-oriented structural data, not proof that an arbitrary combination of generic bounds is satisfiable.

Source: `https://docs.rs/rustdoc-types/0.61.0/rustdoc_types/`

Relevant types: `FunctionHeader`, `FunctionSignature`, `Function`, `Trait`, `ProcMacro`, `GenericBound`, `PreciseCapturingArg`, and `Type`.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+4



### 2.4 Ordinary stability, const stability, and provided-default stability

Schema version 61 distinguishes three forms of compiler stability information that should not be collapsed into a single status.

| Dimension                  | Schema location                                                          | Meaning                                                                               |
| -------------------------- | ------------------------------------------------------------------------ | ------------------------------------------------------------------------------------- |
| Ordinary item stability    | `Item.stability`                                                         | Stability of the declared item through Rust's compiler stability attributes           |
| Const-context stability    | `Item.const_stability`                                                   | Whether the item is stable for use in const contexts                                  |
| Provided-default stability | `Function.default_unstable`, associated constant/type `default_unstable` | Whether a trait-provided default implementation or value requires an unstable feature |

The distinction is important because an item may be usable in ordinary execution without being stable in const contexts.

Similarly, a trait member can be stable as an API contract while its supplied default implementation remains unstable. This affects whether consumers can rely on the provided behavior without implementing their own member.

`ProvidedDefaultUnstable` records the feature gate required for such a default.

There are additional subtleties:

- A reexport has its own stability metadata, separate from the imported definition.
- A declaration's stability may differ from the stability of a path used to access it.
- Associated items may inherit instability from their containing trait or implementation.
- Missing metadata does not establish stability.

These compiler stability annotations are primarily relevant to standard-library and compiler-maintained APIs. The associated Rust stability attributes are not generally available to ordinary third-party crates, which commonly lack such metadata.

Consequently, none of these fields should be used as a universal SemVer or project-maturity classification.

Source: `https://raw.githubusercontent.com/rust-lang/rustdoc-types/v0.61.0/src/lib.rs`

See `Item`, `Stability`, `StabilityLevel`, and `ProvidedDefaultUnstable`.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://raw.githubusercontent.com\&sz=32)

raw\.githubusercontent.com+2



### 2.5 Reexport records and cross-crate identity reconciliation

`Use` and `ExternalCrate` expose additional identity information relevant to reconstructing public namespaces.

A `Use` record contains:

| Field            | Significance                                                            |
| ---------------- | ----------------------------------------------------------------------- |
| `source`         | Full imported path                                                      |
| `name`           | Name introduced at the import site, including renaming                  |
| `id: Option<Id>` | Resolved target when represented; can be absent for primitive reexports |
| `is_glob`        | Whether the import is a wildcard import                                 |

An explicit reexport is an edge between a public exposure and an underlying declaration. It is not necessarily a new declaration identity.

For example, `pub use internal::Engine as PublicEngine` creates a public name that differs from both the imported path and potentially the definition path.

Glob reexports introduce further complexity because they can expose multiple names, with visibility, shadowing, and inlining behavior influencing the documented result.

An API model that uses only `ItemSummary.path` can therefore miss meaningful public names even when it correctly identifies the underlying item.

#### Dependency artifact information

`ExternalCrate` records `name`, `html_root_url`, and `path`.

The `path` field identifies the artifact from which the crate was loaded, commonly an `.rlib` or `.rmeta`. This can help correlate rustdoc references with the build system that invoked rustc and the dependency artifacts it selected.

There are important limits:

- The crate name need not equal the Cargo package name.
- The local `crate_id` namespace is not a persistent identity across independently generated JSON files.
- Artifact paths are build-context evidence, not durable global identifiers.
- A path to a compiled dependency does not uniquely encode its package version, feature configuration, or source identity.

Thus, `ExternalCrate::path` can complement Cargo metadata and other build provenance, but does not replace dependency resolution or cross-build identity reconciliation.

Sources:

`https://docs.rs/rustdoc-types/0.61.0/rustdoc_types/struct.Use.html`

`https://docs.rs/rustdoc-types/0.61.0/rustdoc_types/struct.ExternalCrate.html`

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+1

### 2.6 Target-feature metadata and compilation context

`Crate.target` contains a `Target` record with the target triple and a collection of `TargetFeature` records.

A `TargetFeature` provides:

- `name`: target-feature identifier.
- `implies_features`: features implied by enabling this feature.
- `unstable_feature_gate`: associated language-feature gate when applicable.
- `globally_enabled`: whether the feature is enabled for the compilation session.

This supports a more precise distinction between features recognized by a target and features globally enabled in a particular compilation session.

The implication relationships are useful when considering instruction-set prerequisites. For example, one architectural extension may imply a lower-level extension.

However, a target-feature listing is not a complete API availability matrix. It does not enumerate every possible `cfg` combination, every function's effective runtime requirements, or every externally supported CPU configuration.

The JSON target metadata should be interpreted together with item-level `TargetFeature` attributes and the actual build configuration. An item-specific target-feature declaration is not equivalent to globally enabling that feature for the entire crate.

Sources:

`https://docs.rs/rustdoc-types/0.61.0/rustdoc_types/struct.Target.html`

`https://docs.rs/rustdoc-types/0.61.0/rustdoc_types/struct.TargetFeature.html`

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+1

### 2.7 Constants, discriminants, and source-location fidelity

Rustdoc JSON preserves several different kinds of value information, but they have different interpretation guarantees.

`Constant` contains `expr`, optional `value`, and `is_literal`.

The `expr` field is a stringified expression, not a guaranteed reproduction of the original source. The optional evaluated `value` is computed only for numeric types. Absence of a value does not establish that an expression is invalid or unevaluable by every compiler mechanism.

`Discriminant` similarly separates the source-oriented expression from its numerical value. Its numeric `value` is serialized as a string to preserve integers across the full relevant signed/unsigned 128-bit ranges without relying on JSON number precision. The expression may preserve source formatting but can also be reduced to a placeholder for sufficiently complex cases.

`Span` uses a filename and one-indexed `(line, column)` coordinates for its beginning and end. The schema describes the end as the position of the final character, rather than a conventional exclusive end offset.

These distinctions matter when connecting structured API records to source files, exact-text analysis, or numeric comparisons.

They also establish a firm boundary: rustdoc JSON can provide structured semantic information without guaranteeing round-trippable source syntax or universal source coordinates.

Sources:

`https://docs.rs/rustdoc-types/0.61.0/rustdoc_types/struct.Constant.html`

`https://docs.rs/rustdoc-types/0.61.0/rustdoc_types/struct.Discriminant.html`

`https://docs.rs/rustdoc-types/0.61.0/rustdoc_types/struct.Span.html`

[image](https://www.google.com/s2/favicons?domain=https://raw.githubusercontent.com\&sz=32)

raw\.githubusercontent.com+2

### 2.8 Archived schema representations with `rkyv`

The optional `rustdoc-types` feature `rkyv_0_8` derives `Archive`, `Serialize`, and `Deserialize` implementations and exposes archived forms such as `ArchivedId`.

This enables a different data-access model from repeatedly deserializing ordinary JSON into owned Rust structures.

An archived representation can support access to data in its serialized form without eagerly reconstructing every owned value. For repeated analysis of large API graphs, this can reduce deserialization work when only selected records are needed.

The boundary is important: rustdoc's own JSON output is not already an `rkyv` archive. An application must first create or obtain an archive using the corresponding serialization machinery. The potential benefit applies to subsequent use of that representation, not to arbitrary JSON text directly.

The implementation also has specialized support for recursive and mutually recursive schema types, using appropriate archive bounds and validation facilities. These details matter if extending the archived model, rather than simply using the feature.

The separately optional `rustc-hash` feature concerns in-memory hash-map behavior and has different costs and benefits. Neither feature changes the upstream rustdoc JSON contract.

Source: `https://docs.rs/rustdoc-types/0.61.0/rustdoc_types/`&#x20;

[image](https://www.google.com/s2/favicons?domain=https://raw.githubusercontent.com\&sz=32)

raw\.githubusercontent.com



## 3. Doctest-specific compilation and execution facilities

Extends original §§5.2–5.4 and 10.5.

### 3.1 `#[cfg(doctest)]`: documentation-test-only declarations

Rustdoc sets `cfg(doctest)` while collecting doctests. This is distinct from `cfg(doc)`, which participates in ordinary documentation generation.

`#[cfg(doctest)]` permits test-bearing declarations that do not belong in the crate's normal documentation or compiled public API. A particularly useful application is testing examples embedded in external Markdown without publishing that Markdown as an item page.

For example:

```
#[cfg(doctest)]
#[doc = include_str!("../README.md")]
pub struct ReadmeDoctests;
```

The declaration exists during doctest collection, allowing rustdoc to extract and execute code blocks from the included README. It does not appear in an ordinary crate build.

This separates three concerns: where examples are authored, which examples are tested, and which documentation appears in the published API.

Important limitation: Doctests still link against the crate's externally accessible items. A declaration appearing only under `cfg(doctest)` does not make private crate members accessible to its extracted examples. Ordinary unit tests remain the appropriate mechanism for testing private implementation details.

### 3.2 `test_harness`: multiple test functions within a documentation block

The stable `test_harness` code-block annotation changes how rustdoc compiles a documentation example.

Ordinarily, an eligible code block becomes an executable example, with an implicit `main` function if necessary. Under `test_harness`, contained `#[test]` functions are instead executed through Rust's test harness.

An illustrative documentation block is:

````
//! ```test_harness
//! #[test]
//! fn accepts_valid_input() {
//!     assert!(true);
//! }
//!
//! #[test]
//! #[should_panic]
//! fn rejects_invalid_state() {
//!     panic!("invalid state");
//! }
//! ```
````

This allows documentation to express several independently named assertions and use normal test attributes inside a single code example.

Its significance is not merely syntactic. It delegates test enumeration and expected-panic handling to Rust's established test harness instead of requiring a custom mechanism to discover test functions embedded in documentation.

`test_harness` should be distinguished from `standalone_crate`, which controls doctest compilation isolation rather than test-function discovery.

Primary source: `https://doc.rust-lang.org/rustdoc/write-documentation/documentation-tests.html`&#x20;

[image](https://www.google.com/s2/favicons?domain=https://doc.rust-lang.org\&sz=32)

The rustdoc book+1



### 3.3 Compile-time configuration versus execution environment

Rustdoc has several independent controls over documentation-test execution.

| Control                  | Stability | Responsibility                                                                        |
| ------------------------ | --------- | ------------------------------------------------------------------------------------- |
| `-C` / `--codegen`       | Stable    | Pass rustc code-generation options, including options relevant to doctest compilation |
| `--test-args`            | Stable    | Pass arguments to the doctest test runner                                             |
| `--test-run-directory`   | Stable    | Set the working directory used when executing doctest binaries                        |
| `--test-runtool`         | Stable    | Execute doctests through a specified external program                                 |
| `--test-runtool-arg`     | Stable    | Pass arguments to that external execution program                                     |
| `--test-builder`         | Nightly   | Substitute the compiler-like program used to build doctests                           |
| `--test-builder-wrapper` | Nightly   | Wrap calls to the doctest builder; multiple wrappers may be nested                    |
| `--doctest-build-arg`    | Nightly   | Pass additional arguments to the compiler compiling each doctest                      |

The compile directory and run directory are separate concepts.

Compilation-directory context influences diagnostics, source-path interpretation, and `file!()` behavior. Execution-directory context influences runtime filesystem operations such as relative-path reads and writes.

`--test-run-directory` changes the latter without requiring the former to change. This can preserve consistent compilation diagnostics while allowing examples to execute relative to the directories containing their runtime resources.

`--test-runtool` substitutes the execution command, not the compiler. Rustdoc invokes the specified program with any `--test-runtool-arg` values followed by the path to the compiled doctest executable.

This can accommodate execution through an emulator, virtual machine, instrumentation tool, or other wrapper. It does not itself establish that the selected target can execute successfully; that depends on the external tool and its environment.

The experimental `--test-builder` and `--test-builder-wrapper` facilities operate earlier, at the compilation boundary. They can delegate compilation to an alternative compiler-like executable or surround it with wrappers, while `--doctest-build-arg` supplies per-test compiler arguments.

These mechanisms expose a useful separation of concerns:

Doctest extraction → test compilation → executable invocation → test-result interpretation.

Different native facilities modify different stages, avoiding the need to replace the whole rustdoc test pipeline when only one stage requires specialization.

Sources:

`https://doc.rust-lang.org/rustdoc/command-line-arguments.html`

`https://doc.rust-lang.org/rustdoc/unstable-features.html`

[image](https://www.google.com/s2/favicons?domain=https://doc.rust-lang.org\&sz=32)

The rustdoc book+1

### 3.4 Structured doctest extraction is distinct from API JSON

The original guide identifies the nightly `--output-format doctest` facility but does not fully characterize its output.

It emits JSON to standard output, representing the documentation tests collected from a crate. The upstream documentation currently illustrates doctest format version 2, which is separate from rustdoc API JSON format version 61.

The representation includes:

| Field                                 | Information preserved                                         |
| ------------------------------------- | ------------------------------------------------------------- |
| `format_version`                      | Doctest extraction schema version                             |
| `doctests`                            | Extracted test records                                        |
| `file`, `line`, `name`                | Source location and generated test identity                   |
| `doctest_attributes`                  | Computed execution and compilation attributes                 |
| `original_code`                       | Code before rustdoc preprocessing                             |
| `doctest_code.crate_level`            | Generated crate-level attributes and imports                  |
| `doctest_code.code`                   | Extracted code without crate-level or wrapper content         |
| `doctest_code.wrapper.before/after`   | Generated source surrounding the example                      |
| `doctest_code.wrapper.returns_result` | Whether the wrapper accommodates a `Result`-returning example |

Attributes include such properties as `should_panic`, `no_run`, `compile_fail`, `ignore`, `edition`, `test_harness`, and `standalone_crate`.

This allows programmatic inspection of both the authored example and the effective source constructed by rustdoc.

Such a representation can help answer questions about generated wrappers, error locations, implicit imports, and why a particular documentation example behaves differently from the source fragment shown to readers.

There are two significant boundaries:

1. The output is not interchangeable with `rustdoc_types::Crate` or its `format_version`.
2. Extraction is not equivalent to compilation or successful execution. The representation can omit generated code when fatal syntax problems prevent its construction.

The capability is valuable independently of an execution harness: consumers can analyze extracted tests and compiler transformations without writing their own Rust Markdown/test-attribute parser.

Primary source: `https://doc.rust-lang.org/rustdoc/unstable-features.html`&#x20;

[image](https://www.google.com/s2/favicons?domain=https://doc.rust-lang.org\&sz=32)

The rustdoc book



## 4. Build orchestration, output control, and multi-crate documentation

Extends original §§7.4–7.5, 10.4, and 10.5.

### 4.1 Selective output generation with `--emit`

Rustdoc's stable `--emit` option separates three output categories:

| Emission kind           | Generated artifacts                                             | Architectural significance                                         |
| ----------------------- | --------------------------------------------------------------- | ------------------------------------------------------------------ |
| `html-static-files`     | Shared static assets with content-hashed filenames              | Supports immutable caching of assets associated with the toolchain |
| `html-non-static-files` | Crate-dependent HTML and related files with deterministic names | Maintains predictable per-crate output paths                       |
| `dep-info`              | Makefile-style dependency information                           | Exposes source dependencies for external build orchestration       |

By default, rustdoc emits both HTML categories. Dependency information is opt-in.

A representative invocation is:

```
rustdoc src/lib.rs \
  --emit=html-static-files,html-non-static-files,dep-info=/path/to/crate.d
```

The `dep-info` output identifies files loaded while documenting the crate. It can also be sent to standard output with `--emit=dep-info=-`.

This is more than an output-format convenience. It exposes a native dependency-discovery mechanism for build systems that need to determine when documentation generation must run again.

The important limitation: Dependency information records source-file dependencies of the documented invocation; it is not a complete semantic dependency graph or substitute for Cargo's package-resolution metadata.

The distinction between content-hashed static assets and deterministic crate-dependent filenames also supports different caching strategies. Shared assets can be cached according to their content identity, while crate-specific artifacts require invalidation based on the documented inputs and build context.

Primary source: `https://doc.rust-lang.org/rustdoc/command-line-arguments.html`&#x20;

[image](https://www.google.com/s2/favicons?domain=https://doc.rust-lang.org\&sz=32)

The rustdoc book



### 4.2 Configuration validation and source-path normalization

Two additional stable interfaces are relevant to reproducible documentation generation.

`--check-cfg` accepts the same configuration-expectation syntax as rustc. It can diagnose unexpected configuration names or values, including misspelled feature-dependent conditions.

It complements `--cfg` rather than replacing it: `--cfg` participates in selecting compilation conditions, while `--check-cfg` establishes expectations against which configuration usage can be checked.

`--remap-path-prefix=FROM=TO` performs best-effort source-path rewriting in generated output, diagnostics, debug information, macro expansions, and other affected locations.

This can reduce dependence on absolute build-directory locations when producing documentation in differing build environments.

The remapping is not a guarantee that all machine-specific information disappears from every artifact. Its scope is determined by rustdoc/rustc behavior and the kinds of paths represented.

Rustdoc additionally accepts `@path` response files containing command-line options, one argument per line, allowing tools to avoid unwieldy command lines when orchestrating complex invocations.

Source: `https://doc.rust-lang.org/rustdoc/command-line-arguments.html`&#x20;

[image](https://www.google.com/s2/favicons?domain=https://doc.rust-lang.org\&sz=32)

The rustdoc book+1



### 4.3 Cross-crate search and trait-implementation metadata

Rustdoc has a shared-output model that is easy to overlook if each crate's documentation is treated as a completely independent static website.

When multiple crates are generated into the same documentation root, rustdoc maintains shared browser-facing information, including:

- `crates<SUFFIX>.js`: the collection of crates represented in that documentation root.
- `search-index<SUFFIX>.js`: searchable items across those crates.
- `implementors/.../trait.TraitName.js`: implementation information for particular traits, potentially accumulated from crates other than the crate that defines the trait.

The third category is especially consequential.

A trait may be defined in one crate and implemented for types in another. As additional crates are documented into the same output directory, rustdoc can update the trait's implementor information so those cross-crate relationships become visible.

This is a native form of cross-crate API-relationship aggregation within the documentation renderer.

The output is not simply a collection of isolated HTML pages. It also includes shared search and relationship metadata that rustdoc updates as the documentation collection grows.

However, these JavaScript artifacts are renderer implementation details, not a stable interchange schema. Their contents and organization may change independently of the `rustdoc-types` schema.

For a programmatic API graph, rustdoc JSON remains the more appropriate structured source. The browser-facing artifacts explain how published navigation is assembled, but should not be mistaken for a durable general-purpose knowledge-graph API.

A further boundary concerns docs.rs. The compiler development guide describes docs.rs as building documentation for individual crates, with some shared static resources, rather than assembling each crate's complete dependency documentation into one searchable bundle.

Primary source: `https://rustc-dev-guide.rust-lang.org/rustdoc.html`&#x20;

[image](https://www.google.com/s2/favicons?domain=https://rustc-dev-guide.rust-lang.org\&sz=32)

Rust Compiler Development Guide



### 4.4 Multi-crate landing-page generation

Two nightly options extend rustdoc's multi-crate publication capabilities:

- `--enable-index-page` generates a default top-level index listing documented crates.
- `--index-page` uses a supplied Markdown document as a customized top-level landing page and implicitly enables index-page generation.

These options concern the navigation entry point for a documentation collection, not the individual crate's root-module documentation.

They are therefore distinct from crate-level `//!` documentation and ordinary HTML content-injection mechanisms.

For documentation collections combining numerous crates, a shared landing page can provide an organizing layer above the crate namespaces without requiring a separate HTML generator.

Primary source: `https://doc.rust-lang.org/rustdoc/unstable-features.html`&#x20;

[image](https://www.google.com/s2/favicons?domain=https://doc.rust-lang.org\&sz=32)

The rustdoc book



## 5. Effective API navigation, source inspection, and validation

Extends original §§3.3, 4.2–4.3, 6.1, 7.1, and 10.2.

### 5.1 Dereference-mediated method availability

The visible methods of a Rust type are not restricted to its own inherent implementations and implemented traits.

The compiler's method lookup can also discover methods through `Deref` and `DerefMut` target types.

For example, the generated standard-library documentation for `String` separately displays its inherent methods and a large collection of methods from `Deref<Target = str>`.

Likewise, `Vec<T>` can expose applicable slice functionality through dereferencing.

This creates an important distinction between three related representations:

| Representation                | What it describes                                                               |
| ----------------------------- | ------------------------------------------------------------------------------- |
| Declared method graph         | Methods defined in a type's inherent implementations                            |
| Trait implementation graph    | Methods provided through implemented traits, including defaults                 |
| Effective method-call surface | Methods potentially callable after Rust's autoderef and method-resolution rules |

The third representation is not simply the union of the first two.

Actual method availability can depend on dereference chains, trait imports, receiver types, generic bounds, visibility, and Rust's method-resolution precedence.

A method appearing in rustdoc's `Deref`-derived presentation is consequently different from a method defined directly on the original type.

This distinction matters for API discovery: a graph containing only directly attached methods may miss much of a type's useful functionality.

It also establishes a boundary for automated analysis. Rustdoc's rendered method groups provide meaningful relationships, but determining whether a particular method call resolves successfully under arbitrary generic substitutions still requires language-level method resolution.

Sources:

`https://doc.rust-lang.org/std/string/struct.String.html`

`https://doc.rust-lang.org/std/ops/trait.Deref.html`

[image](https://www.google.com/s2/favicons?domain=https://doc.rust-lang.org\&sz=32)

Rust

### 5.2 Type-search semantics extend beyond functions

Rustdoc's type-signature search treats some non-callable items as though they were function-like.

The original guide mentions fields behaving like getter methods. An additional capability is that constants and statics behave like nullary functions returning their declared type.

For example, a search for `-> u32` can match the associated constant `u32::MAX`.

This broadens the search model from executable operations to typed API values.

It means that type-based search can discover:

- Functions producing a requested type.
- Fields whose type matches the requested output.
- Constants and statics supplying values of that type.

This functionality is useful when the name of an API value is unknown but its type is known.

Several matching limitations are consequential. Rustdoc's search documentation explicitly states that it ignores supertrait relationships, type aliases, and dereference relationships during type-based matching. It is therefore not a general-purpose compiler type-unification engine.

An effective method visible through `Deref` should not be assumed to be discoverable through type search by substituting the original receiver type.

Type-search matching also handles selected generic wrappers specially, including references, `Box`, `Rc`, `Arc`, `Option`, `Result`, `Future`, `From`, and `Into`.

These matching conveniences improve exploratory discovery but should not be interpreted as proof of type-level substitutability.

Primary source: `https://doc.rust-lang.org/rustdoc/read-documentation/search.html`&#x20;

[image](https://www.google.com/s2/favicons?domain=https://doc.rust-lang.org\&sz=32)

The rustdoc book



### 5.3 Additional documentation-specific lints

Several rustdoc lints omitted from the original guide express useful validation properties distinct from missing documentation or broken semantic links.

| Lint                                 | Status                         | Purpose                                                 |
| ------------------------------------ | ------------------------------ | ------------------------------------------------------- |
| `rustdoc::private_doc_tests`         | Stable; allow by default       | Detect doctests attached to private items               |
| `rustdoc::missing_crate_level_docs`  | Stable; allow by default       | Detect missing root-level crate documentation           |
| `rustdoc::unescaped_backticks`       | Stable; allow by default       | Detect likely malformed Markdown inline-code delimiters |
| `rustdoc::redundant_explicit_links`  | Stable; warn by default        | Detect redundant explicit targets on intra-doc links    |
| `rustdoc::missing_doc_code_examples` | Nightly only; allow by default | Detect eligible documented items without code examples  |

These lints validate different aspects of documentation quality.

`private_doc_tests` highlights a mismatch between the location of executable examples and the external API that doctests can actually exercise. It does not prohibit private-item documentation; it identifies examples whose placement may be surprising.

`missing_crate_level_docs` targets the navigational entry point of a crate, rather than requiring comments on every public item.

`unescaped_backticks` and `redundant_explicit_links` address Markdown integrity and maintainability, complementing semantic link validation.

The nightly `missing_doc_code_examples` lint is particularly useful to distinguish from documentation coverage.

It looks for examples on eligible items, but intentionally excludes categories such as implementation blocks, enum variants, fields, modules, type aliases, constants, statics, and foreign reexports.

A lint-clean result therefore does not mean every documented item has an example.

Nor does it imply those examples successfully compile or execute. That is established by separate doctest behavior.

The additional lint families collectively provide a more granular validation model than a single documented/undocumented property.

Sources:

`https://doc.rust-lang.org/rustdoc/lints.html`

`https://doc.rust-lang.org/rustdoc/unstable-features.html`

[image](https://www.google.com/s2/favicons?domain=https://doc.rust-lang.org\&sz=32)

The rustdoc book+3

### 5.4 Standalone Markdown is also an executable documentation input

The guide describes standalone Markdown rendering but does not fully distinguish its associated testing and presentation controls.

Rustdoc can execute eligible code blocks from Markdown files directly, for example:

```
rustdoc --test README.md
```

This is distinct from including a Markdown file as crate documentation via `include_str!`.

In the direct mode, the Markdown file is the input being examined for tests. Under `#[doc = include_str!(...)]`, Markdown becomes documentation attached to a Rust item and participates in that item's documentation context.

Standalone Markdown rendering also has specific stable options:

| Option                      | Behavior                                                                                      |
| --------------------------- | --------------------------------------------------------------------------------------------- |
| `--markdown-css`            | Adds stylesheet references when rendering standalone Markdown; ignored for Rust source inputs |
| `--markdown-playground-url` | Supplies the playground base URL used for runnable code examples                              |
| `--markdown-no-toc`         | Suppresses the otherwise generated standalone Markdown table of contents                      |

The distinction between the two Markdown integration routes is useful because it separates documentation content ownership from executable-example extraction.

A file can serve as standalone Markdown documentation, be incorporated into a crate's generated documentation, or be included on a doctest-only carrier using `#[cfg(doctest)]`. Those choices affect presentation and test discovery differently.

Primary sources:

`https://doc.rust-lang.org/rustdoc/command-line-arguments.html`

`https://doc.rust-lang.org/rustdoc/write-documentation/documentation-tests.html`

[image](https://www.google.com/s2/favicons?domain=https://doc.rust-lang.org\&sz=32)

The rustdoc book+2

### 5.5 Macro expansion in source browsing

The nightly `--generate-macro-expansion` option adds controls for expanding macros directly in rustdoc's generated HTML source pages.

This is a separate capability from `--generate-link-to-definition`.

The two facilities expose different perspectives:

- Definition links help navigate from a source reference to a related declaration.
- Macro-expansion toggles expose generated syntax associated with macro invocations.

Macro expansion can materially change the declarations and executable code present after preprocessing. Making expansion visible in the source browser therefore improves understanding of generated implementations and macro-heavy code.

However, a rendered macro expansion is not a complete representation of procedural macro execution, its input validation, or every relevant compiler transformation. It should not be treated as a general source-preserving macro-expansion API.

For automated code transformations or exact expansion provenance, direct compiler or syntax-tooling interfaces may still be necessary.

Primary source: `https://doc.rust-lang.org/rustdoc/unstable-features.html`&#x20;

[image](https://www.google.com/s2/favicons?domain=https://doc.rust-lang.org\&sz=32)

The rustdoc book



### 5.6 Exception to `#[doc(inline)]` behavior for hidden targets

The original guide summarizes `#[doc(inline)]` as forcing documentation inlining. This is correct for ordinary cases, but an important exception affects `#[doc(hidden)]` targets.

When the underlying target is itself hidden, applying `#[doc(inline)]` to its public reexport can make the reexport visible without exposing the hidden target's normal documentation.

The official rustdoc documentation illustrates a hidden struct reexported with `#[doc(inline)]`. The resulting documentation contains the reexport declaration, but no link to the hidden item's page.

The distinction is:

- Making a reexport visible is not necessarily equivalent to inlining the complete target documentation.
- A public symbol can remain part of the language-level API even when its definition is suppressed from normal documentation.
- Documentation reachability and target-item documentation must be interpreted separately.

This exception reinforces the need to represent the exposure path and declaration identity independently.

Primary source: `https://doc.rust-lang.org/rustdoc/write-documentation/re-exports.html`&#x20;

[image](https://www.google.com/s2/favicons?domain=https://doc.rust-lang.org\&sz=32)

The rustdoc book



## 6. Combined architectural implications

The additional capabilities reveal several useful interactions that are not obvious when considering each feature separately.

Structured API intelligence without reconstructing Rust semantics. Rustdoc JSON combines typed representation attributes, callable signatures, trait properties, associated-item relationships, conditional target information, and explicit omissions. Taken together, these supply a more expressive public-API model than source-pattern matching. They do not provide a complete compiler semantic model, especially for function bodies or arbitrary generic substitutions.

A more faithful model of API completeness. `Module.is_stripped`, `has_stripped_fields`, `has_stripped_variants`, optional tuple-field identifiers, and documentation-hidden reexports make it possible to distinguish known omission from represented absence. Combined with build-context metadata, these indicators reduce misleading conclusions from apparently incomplete API records.

Executable documentation without bespoke orchestration. `cfg(doctest)`, `test_harness`, standalone Markdown testing, custom test runners, alternative builders, and structured doctest extraction provide multiple extension boundaries. They can support specialized execution and analysis while retaining rustdoc's native test extraction and compiler integration.

Documentation generation as a build-system participant. `--emit=dep-info`, configuration validation, path remapping, separable static assets, and shared cross-crate metadata enable documentation generation to participate in broader build and publication infrastructure. Their benefit depends on how build contexts and generated artifacts are managed; they are not substitutes for a complete incremental compilation engine.

Declared versus effectively callable functionality. Direct inherent methods, trait implementations, provided methods, and dereference-derived methods describe different aspects of a type's capabilities. Rustdoc exposes several of these relationships, but fully determining method-call validity remains a compiler-resolution problem.

The central conclusion of this addendum is that rustdoc offers more structured, inspectable, and extensible machinery than a documentation generator alone would suggest, while its information remains deliberately bounded by documentation semantics, selected compilation configuration, and experimental schema compatibility.

The most important additions to the original reference are therefore the explicit representation of incomplete API structure, richer semantic metadata in JSON, additional doctest/compiler integration points, and the distinction between documented relationships and fully resolved Rust behavior.

These mechanisms can substantially reduce bespoke API extraction, documentation analysis, and test-orchestration code, provided their respective completeness and stability boundaries remain explicit.