# Rust Dependency Graph Tooling: cargo_metadata, cargo tree, and cargo-hakari

Technical capability reference | Research date: October 29, 2026

## 1. Architectural overview

These three tools operate at different levels of Cargo's dependency system: structured metadata extraction, dependency-graph explanation, and build-feature optimization. They are complementary rather than competing implementations.

### Versions and identities

| Component        | Verified version  | Primary role                                                       |
| ---------------- | ----------------- | ------------------------------------------------------------------ |
| `cargo_metadata` | 0.23.1            | Rust API for Cargo metadata and build-event JSON                   |
| `cargo tree`     | Cargo 1.99        | Built-in Cargo command for dependency and feature graph inspection |
| `cargo-hakari`   | 0.9.39            | CLI for workspace-wide feature unification                         |
| `hakari`         | 0.18.0            | Reusable Rust library underlying cargo-hakari                      |
| `guppy`          | Companion library | Graph querying, traversal, dependency resolution simulation        |

Cargo 1.99 is the Cargo release accompanying Rust 1.99, with Rust 1.99.1 released October 15, 2026. The other versions are independently published crates.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://blog.rust-lang.org\&sz=32)

blog.rust-lang.org+5



### Capability boundaries

| Concern                                        | Native capability                | What it makes possible                                                                            |
| ---------------------------------------------- | -------------------------------- | ------------------------------------------------------------------------------------------------- |
| Obtain workspace and package metadata          | `cargo_metadata`                 | Structured discovery of packages, targets, manifests, features, and dependency relationships      |
| Trace dependency inclusion                     | `cargo tree`                     | Explain why a package is present, which packages require it, and where duplicate versions arise   |
| Explain feature activation                     | `cargo tree -e features`         | Identify feature activation chains, including transitive dependencies                             |
| Analyze graph properties programmatically      | `cargo_metadata` + graph tooling | Dependency indexing, reachability, reverse traversal, package classification, and change analysis |
| Simulate different feature/target combinations | `guppy`                          | Examine dependency resolution across configurations rather than inspecting one metadata result    |
| Reduce unnecessary build variation             | `cargo-hakari` / `hakari`        | Consolidate dependency feature sets across workspace build configurations                         |
| Maintain feature unification automatically     | `cargo-hakari`                   | Generate and validate a managed workspace-hack package                                            |

Central distinction: Cargo resolves dependencies; `cargo_metadata` exposes structured results, `cargo tree` explains them interactively, and Hakari deliberately changes workspace dependency declarations to encourage more consistent feature unification.

Hakari does not replace Cargo's resolver or compiler. Its optimization is to make different workspace builds more likely to reuse compatible compiled dependency artifacts.

The important underlying mechanism is that package identity, dependency-edge identity, enabled features, and actual compilation-unit identity are not interchangeable. A package may have multiple dependency paths, different feature activations under different build selections, or multiple host/target compilation contexts.

The remainder examines these distinctions and the native capabilities that expose or manipulate them.

## 2. `cargo_metadata`: structured Cargo introspection

Crate: `cargo_metadata` 0.23.1\
Interface: Rust library wrapping Cargo's `cargo metadata` command and parsing Cargo's machine-readable build messages.

Primary documentation: `https://docs.rs/cargo_metadata/latest/cargo_metadata/`

### 2.1. Execution model and command construction

`cargo_metadata` delegates dependency resolution to the installed Cargo executable. It does not implement an independent resolver, parse an entire workspace solely from manifests, or perform compilation.

Its central entry point, `MetadataCommand`, supports:

| API                                 | Capability and important behavior                                                                            |
| ----------------------------------- | ------------------------------------------------------------------------------------------------------------ |
| `new()`                             | Discover the current Cargo project, looking for `Cargo.toml` in the current directory or ancestors           |
| `cargo_path()`                      | Select the Cargo executable; otherwise uses `$CARGO`, then `cargo`                                           |
| `manifest_path()` / `current_dir()` | Explicitly select the manifest and execution context                                                         |
| `features(CargoOpt)`                | Specify enabled features, `NoDefaultFeatures`, or `AllFeatures`; multiple feature selections can be combined |
| `no_deps()`                         | Retrieve workspace-member metadata without resolving or fetching third-party dependencies                    |
| `other_options()`                   | Supply Cargo options not explicitly modeled by the wrapper                                                   |
| `env()` / `env_remove()`            | Override or remove inherited environment variables for the child Cargo process                               |
| `cargo_command()`                   | Obtain the constructed process command for further control                                                   |
| `exec()` / `parse()`                | Execute Cargo and deserialize its output, or deserialize JSON separately                                     |

The environment controls matter for nested tooling: inherited `RUSTFLAGS`, Cargo configuration variables, target selections, or wrapper-specific environment settings can alter dependency resolution or cause unexpected execution behavior.

`features()` is additive for feature-name selections, but repeating `CargoOpt::NoDefaultFeatures` or `CargoOpt::AllFeatures` triggers a documented panic. Version 0.23 added the typed `PackageName` and `FeatureName` representations, while 0.23.1 includes the `build_directory` field introduced in Cargo's metadata schema.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+1



Execution semantics: A full metadata request can involve registry access and lockfile resolution. `--locked`, `--offline`, `--frozen`, and `--filter-platform` are Cargo-level controls, not independent `cargo_metadata` algorithms. They can be passed through `other_options()`.

The library therefore inherits the selected Cargo version's resolver behavior, manifest interpretation, network requirements, and configuration. Its own crate version governs the Rust API and JSON compatibility layer, not Cargo's dependency-resolution rules.

### 2.2. Data model: declarations versus resolution

The most important distinction is between the manifest-derived package model and the resolved dependency graph.

| Structure    | Meaning                                    | Consequential fields                                                                                                        |
| ------------ | ------------------------------------------ | --------------------------------------------------------------------------------------------------------------------------- |
| `Metadata`   | Workspace-wide result                      | `packages`, `workspace_members`, `workspace_default_members`, `resolve`, root/build/target directories, workspace metadata  |
| `Package`    | Package identity and declared capabilities | `id`, `name`, `version`, `source`, `manifest_path`, `dependencies`, `targets`, `features`, `rust_version`, package metadata |
| `Dependency` | A declared dependency relationship         | Requirement, kind, rename, source, optionality, requested features, default-feature behavior, target condition              |
| `Target`     | A Cargo compilation target declaration     | Target name, kind, crate types, source path, edition, required features, test/documentation settings                        |
| `Resolve`    | Result of dependency resolution            | Resolved nodes and an optional root package                                                                                 |
| `Node`       | Resolved package graph vertex              | Package ID, resolved dependencies, detailed dependency edges, enabled features                                              |
| `NodeDep`    | Resolved dependency edge                   | Dependency alias, destination package ID, dependency kinds and target conditions                                            |

Two distinctions are especially important.

Declared features are not enabled features. `Package.features` describes a package's feature definitions: which other features or optional dependencies each feature activates. `Node.features` describes the features enabled in a particular resolved graph. Similarly, `Package.dependencies` may describe optional, target-specific or development relationships that are not active in the resolved graph.

Dependency names are not package identities. A dependency can be renamed locally, and multiple versions or sources of a crate can coexist. `NodeDep.name` preserves the dependency name used by the dependent crate; `NodeDep.pkg` identifies the actual package. Graph joins should use `PackageId`, not package name alone.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+3



Direct references:

- `https://docs.rs/cargo_metadata/latest/cargo_metadata/struct.Metadata.html`
- `https://docs.rs/cargo_metadata/latest/cargo_metadata/struct.Package.html`
- `https://docs.rs/cargo_metadata/latest/cargo_metadata/struct.Node.html`
- `https://docs.rs/cargo_metadata/latest/cargo_metadata/struct.NodeDep.html`

### 2.3. Workspace and package discovery

The result provides enough structure to discover an entire Cargo workspace without bespoke `Cargo.toml` parsing:

- Membership and default selection: `workspace_members` and `workspace_default_members` distinguish all members from members selected by default. This matters for virtual workspaces and root-package workspaces.
- Package inventory: `packages` contains relevant resolved packages, including non-workspace dependencies unless `--no-deps` is used. The package records carry registry, Git, or path source information.
- Build-target inventory: `Target` records identify binaries, libraries, examples, tests, benchmarks and build scripts, together with associated source files and feature gates.
- Manifest extensions: `Package.metadata` and `Metadata.workspace_metadata` expose user-defined TOML metadata as JSON values. These can carry tool configuration without a separate parsing layer.
- Toolchain compatibility information: The package's `rust_version` records its declared minimum supported Rust version; this is metadata, not proof that all resolved dependencies are compatible with an arbitrary compiler.

Helper methods `root_package()`, `workspace_packages()`, and `workspace_default_packages()` simplify common lookups. `root_package()` is optional because a virtual workspace has no root package. The default-member field can be unavailable with older Cargo versions; the crate provides `WorkspaceDefaultMembers::is_available()` and `is_missing()` to distinguish this case.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust



This makes `cargo_metadata` useful as a canonical discovery and indexing layer for build tooling, source-code analysis, code intelligence, dependency policy engines, artifact registries, and workspace-aware orchestration.

### 2.4. Graph extraction and feature-state analysis

`resolve.nodes` supports construction of a directed package graph and preservation of edge semantics. Its natural uses include forward and reverse adjacency, reachability, dependency-impact analysis, cycle checks for derived dependency subsets, dependency-kind classification, and feature-state inventories.

There are several non-obvious limits:

1. `--no-deps` removes the graph, not merely its transitively referenced package details. `resolve` is `None`; this mode is for workspace inventory.
2. Platform filtering affects resolved edges, not manifest declarations. `--filter-platform <triple>` narrows the resolution output, but the `packages` array still retains dependencies declared for other targets.
3. The default full resolution spans target platforms. A dependency appearing in the unfiltered graph is not evidence that a selected target will compile it.
4. Resolved nodes are packages, not compiler invocations. Host/target compilation, separate feature contexts, build-profile settings, and build-script behavior require additional information to model actual compiler work.
5. One invocation represents one feature-selection context. It is not an inventory of all possible workspace build configurations.

These limitations matter when using the graph to decide whether a package is truly built, whether one compilation artifact can be reused, or whether a dependency is removable.

Cargo's JSON metadata format remains version `1`. The stable field structure does not imply that every package identifier string or future enum value is immutable. The crate exposes non-exhaustive data types; downstream code should preserve forward-compatible handling and avoid depending on incidental formatting of `PackageId`.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://doc.rust-lang.org\&sz=32)

The Cargo Book+2



Cargo schema: `https://doc.rust-lang.org/cargo/commands/cargo-metadata.html`

### 2.5. A second, easily overlooked capability: build-event parsing

`cargo_metadata` is not limited to static dependency metadata. `Message::parse_stream()` parses the line-delimited JSON emitted by Cargo commands using `--message-format=json` or an appropriate JSON diagnostics variant.

| Message variant       | Information exposed                                                        | Useful operation                                                                       |
| --------------------- | -------------------------------------------------------------------------- | -------------------------------------------------------------------------------------- |
| `CompilerArtifact`    | Package, target, enabled features, output filenames, executable, freshness | Artifact discovery, build-output indexing, identifying whether an artifact was rebuilt |
| `CompilerMessage`     | Structured compiler diagnostics, message codes, spans and rendered text    | Diagnostics collection and cross-file navigation                                       |
| `BuildScriptExecuted` | Build-script execution results and emitted configuration                   | Capture native-linking and build-script-derived metadata                               |
| `BuildFinished`       | Cargo's reported completion status                                         | Recognize the final build-result message                                               |
| `TextLine`            | Non-JSON output appearing in the stream                                    | Preserve unexpected text without assuming every line is JSON                           |

The parser is deliberately separate from process execution. A caller can launch Cargo, stream events through `Message::parse_stream()`, correlate them with the package graph, and check the child process's final exit status.

This combination can replace substantial bespoke work for package-to-artifact mapping, diagnostic ingestion, build-result indexing, and structured build observability. It does not itself expose accurate per-rustc execution timing or a complete compilation dependency graph.

The crate also exposes an optional `unstable` feature for parsing nightly JSON test output and a `builder` feature for constructing selected non-exhaustive message structures through builders. These are distinct from the ordinary, stable metadata and build-message parsing capabilities.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+1



Direct reference: `https://docs.rs/cargo_metadata/latest/cargo_metadata/enum.Message.html`

## 3. `cargo tree`: explainable dependency and feature graphs

Component: Built-in Cargo command, not a separately installed Rust library.\
Documentation: `https://doc.rust-lang.org/cargo/commands/cargo-tree.html`

`cargo tree` specializes in answering why a package or feature is part of a dependency selection. It turns Cargo's resolved relationships into human-readable evidence, including inverse and feature-activation views that would otherwise require custom graph traversal and explanation logic.

### 3.1. Core analysis modes

| Operation                       | Command                           | Interpretation                                                              |
| ------------------------------- | --------------------------------- | --------------------------------------------------------------------------- |
| Dependency graph                | `cargo tree`                      | Forward dependency relationships                                            |
| Reverse dependencies            | `cargo tree -i serde`             | Which selected packages depend on `serde`                                   |
| Workspace-wide reverse analysis | `cargo tree -i serde --workspace` | Reverse paths throughout the workspace                                      |
| Feature activation              | `cargo tree -e features`          | Features activated along dependency relationships                           |
| Feature provenance              | `cargo tree -e features -i serde` | Why particular features of `serde` are enabled                              |
| Duplicate versions              | `cargo tree -d`                   | Packages appearing in multiple resolved versions, displayed in inverse form |
| Enabled feature summary         | `cargo tree --format "{p} {f}"`   | Per-package feature inventory                                               |
| Build-oriented graph            | `cargo tree -e normal,build`      | Approximate non-test dependency selection                                   |
| Target selection                | `cargo tree --target <triple>`    | Graph relevant to a selected target                                         |
| All-target analysis             | `cargo tree --target all`         | Dependencies across supported target conditions                             |

The inverse feature graph is the most distinctive diagnostic capability. `cargo tree -e features -i <crate>` connects an enabled feature to the feature or dependency activation chain responsible for it, often all the way back to a selected workspace package.

This is useful for explaining unexpected optional dependencies, large feature sets, unwanted runtime backends, or features introduced through transitive dependencies. It is more informative than merely listing all active features.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://doc.rust-lang.org\&sz=32)

The Cargo Book



### 3.2. Graph slicing, pruning, and rendering

`cargo tree` provides a substantial set of controls that can eliminate the need for an intermediate graph visualization implementation.

Dependency-edge selection. `--edges` (`-e`) accepts `normal`, `build`, `dev`, `features`, and negated filters such as `no-dev` and `no-proc-macro`. The default includes normal, build, and development dependencies. Positive and negative dependency-kind selectors have documented restrictions on mixing.

Package and scope selection. `--package` selects specified packages; `--workspace` broadens the selected workspace membership; `--exclude` removes selected workspace members when used with `--workspace`. `--invert` restricts explanations to dependency paths leading back to the selected scope.

Depth and pruning. `--depth <n>` limits traversal depth; the special `--depth workspace` displays workspace members. `--prune <spec>` removes specified branches from the rendered view.

Repeated-path presentation. By default, repeated dependency subtrees are collapsed with `(*)`. `--no-dedupe` expands them. This is a presentation choice, not the elimination or restoration of underlying dependency edges.

Output formatting. `--format` exposes substitutions for package identity (`{p}`), license (`{l}`), repository (`{r}`), enabled features (`{f}`), and library target name (`{lib}`). `--prefix indent|depth|none` and `--charset utf8|ascii` allow alternative layouts, including flat or depth-prefixed output.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://doc.rust-lang.org\&sz=32)

The Cargo Book+1



The important boundary is that these are display and explanation controls, not a stable, comprehensive machine-readable graph protocol. Applications requiring exact package identities, renamed dependency edges, or reproducible graph transformations are better served by `cargo metadata`.

### 3.3. Understanding duplicate versions

`cargo tree -d` detects packages appearing in multiple versions and displays reverse paths that explain why those versions coexist.

This can expose several different underlying situations:

- A crate has incompatible version requirements across dependency branches.
- A transitive dependency has not migrated to the newer major version used elsewhere.
- Different dependency sources or constraints prevent convergence on one release.

However, duplicate package versions are different from duplicate compilation work.

Cargo may compile the same package version more than once under different compilation contexts, including separate host and target dependency contexts. Conversely, a package listed along many tree paths is not necessarily recompiled for each path.

`-d` therefore diagnoses version duplication; it is not an exhaustive inventory of compiler invocations, incremental-cache misses, or build-unit variation. Those require build-event evidence and additional compilation-context analysis.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://doc.rust-lang.org\&sz=32)

The Cargo Book+1



### 3.4. Feature resolver behavior changes the meaning of the tree

Cargo's feature resolver affects which dependency features are combined.

| Context                    | Relevant behavior                                                                                                                                                                                                        |
| -------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| Resolver 1                 | Broad feature unification across dependency contexts                                                                                                                                                                     |
| Resolver 2                 | Separates features for several contexts that should not be unified, including inactive target dependencies, build/proc-macro dependencies versus normal dependencies, and dev dependencies when they are not being built |
| Resolver 3                 | Retains resolver-2-style feature behavior while adding Rust-version-aware dependency selection behavior                                                                                                                  |
| Multiple Cargo invocations | Each selected workspace build may activate a different feature set, even when the lockfile resolves the same package versions                                                                                            |

Consequently, `cargo tree` is close to a feature-unified build graph, not an exact build plan. Cargo's own documentation notes that `cargo tree -e normal,build` approximates `cargo build`, while the default graph approximates `cargo test`; neither is a strict equivalence.

Also, target defaults differ between tools: `cargo tree` defaults to the host platform, whereas the unfiltered `cargo metadata` resolution includes dependencies for all target platforms. Comparing their outputs without aligning those settings can produce apparent inconsistencies that are entirely expected.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://doc.rust-lang.org\&sz=32)

The Cargo Book+2



Cargo resolver reference: `https://doc.rust-lang.org/cargo/reference/resolver.html`

## 4. `cargo-hakari`: workspace-wide feature-set unification

CLI package: `cargo-hakari` 0.9.39\
Reusable library: `hakari` 0.18.0\
Underlying graph engine: `guppy`

Primary references:

`https://docs.rs/cargo-hakari/latest/cargo_hakari/`

`https://docs.rs/cargo-hakari/latest/cargo_hakari/about/`

`https://docs.rs/hakari/latest/hakari/`

### 4.1. The problem: feature unification is scoped to build selections

Cargo normally unifies features requested for a dependency within a compatible compilation context. However, different build commands may select different workspace members or features, producing different effective feature sets for the same dependency.

Consider three separate invocations that each require the same resolved version of `syn`:

| Workspace selection       | Required `syn` features |
| ------------------------- | ----------------------- |
| Package A                 | `parsing`, `derive`     |
| Package B                 | `full`, `printing`      |
| Packages A and B together | Union of those features |

These are potentially distinct compilation configurations. Switching among them can invalidate cached dependency artifacts and require recompilation of dependent crates.

Hakari introduces a workspace-hack package depended upon by participating workspace members. The generated package requests the feature union required across simulated build selections, helping stabilize effective dependency features even when different workspace packages are built independently.

The mechanism is declarative: ordinary Cargo dependency and feature activation perform the actual unification. Hakari computes and maintains the declarations.

It is therefore useful to distinguish:

- Cargo's native feature unification: combines features within applicable dependency-resolution and compilation contexts.
- Hakari's workspace-wide stabilization: introduces dependencies that make multiple separately invoked builds converge toward common feature sets.
- Compiler artifact reuse: the potential benefit of stable feature sets, subject to other compilation inputs remaining compatible.

Hakari does not unify incompatible dependency versions, replace Cargo's resolver, or guarantee one compilation artifact per package across all host, target, profile, toolchain, and compiler-flag combinations.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+1



### 4.2. The Hakari computation algorithm

The reusable `hakari` crate implements a more substantial algorithm than simply enabling every feature of every dependency.

Its documented process is:

1. Simulate candidate builds. Through `guppy`, examine workspace packages across selected platforms and different feature selections, including no features, default features, and all features. Build and development dependency contexts are part of the simulation.
2. Collect compilation feature sets. Record the distinct feature sets observed for each dependency.
3. Select packages needing unification. By default, include dependencies with multiple simulated feature configurations, rather than indiscriminately adding every dependency to the workspace-hack.
4. Compute a fixpoint. Adding dependencies to the workspace-hack can itself change transitive feature activation. Hakari repeatedly checks for newly divergent dependencies until the computed output stabilizes.
5. Generate Cargo declarations. Produce the necessary dependency and feature declarations, distinguishing normal and build dependencies and incorporating relevant target conditions.

The computation is parallelized with Rayon. The resulting `Hakari` instance exposes a `computed_map` of simulated results and an `output_map` used to generate the workspace-hack declarations.

This is a meaningful advantage over a handcrafted union of `Cargo.toml` feature declarations: the algorithm accounts for dependency activation, selected configurations, transitive interactions, and the secondary effects of introducing the workspace-hack itself.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust



### 4.3. CLI capabilities

| Command                              | Capability                                                                                                                 |
| ------------------------------------ | -------------------------------------------------------------------------------------------------------------------------- |
| `cargo hakari init <path>`           | Initialize a workspace-hack package and configuration                                                                      |
| `cargo hakari generate`              | Compute and update the generated workspace-hack manifest section                                                           |
| `cargo hakari manage-deps`           | Add or reconcile workspace-hack dependencies in participating workspace members                                            |
| `cargo hakari explain <name>`        | Explain why a particular dependency is included in the generated workspace-hack                                            |
| `cargo hakari verify`                | Check whether simulated build configurations retain divergent feature sets                                                 |
| `cargo hakari generate --diff`       | Detect differences between generated and checked-in workspace-hack content without treating the output as silently current |
| `cargo hakari manage-deps --dry-run` | Check whether workspace-member dependency declarations are synchronized                                                    |
| `cargo hakari disable`               | Remove generated workspace-hack content temporarily                                                                        |
| `cargo hakari remove-deps`           | Remove workspace-hack dependency declarations from managed members                                                         |
| `cargo hakari publish -p <crate>`    | Publish a crate with its workspace-hack dependency temporarily removed                                                     |

`explain` and `verify` address different questions from `cargo tree`.

`cargo tree -e features -i <crate>` explains why Cargo activated a feature in a selected graph.

`cargo hakari explain <crate>` explains why Hakari put a dependency into the workspace-hack.

`cargo hakari verify` checks the unification property across Hakari's simulated configuration space. It should not be interpreted as an independently observed guarantee about every conceivable Cargo command or cross-compilation environment.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust



### 4.4. Configuration: consequential options

The default configuration lives at `.config/hakari.toml` in the workspace root. Older `.guppy/hakari.toml` configurations remain supported as a fallback.

| Configuration               | Meaning                                                             | Significant implication                                                                          |
| --------------------------- | ------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------ |
| `hakari-package`            | Identifies the managed workspace-hack package                       | Establishes the target for generated dependencies                                                |
| `resolver`                  | Cargo feature resolver version                                      | Must reflect intended Cargo feature semantics; resolver 2 is strongly recommended by the project |
| `platforms`                 | Explicit target triples to simulate                                 | Can improve feature stabilization for commonly used platforms                                    |
| `traversal-excludes`        | Omit packages during graph traversal                                | Suppresses their dependency subgraphs unless reachable through other paths                       |
| `final-excludes`            | Remove packages from generated output after computation             | Preserves their transitive dependencies as potential unification candidates                      |
| `unify-target-host`         | Controls cross-context feature consolidation                        | Determines how normal and host-side build dependencies participate                               |
| `output-single-feature`     | Include packages that only have one simulated feature configuration | Broader generated output; ordinarily unnecessary                                                 |
| `exact-versions`            | Emit exact dependency versions rather than compatible ranges        | Couples generated requirements more tightly to the lockfile                                      |
| `workspace-hack-line-style` | Controls how workspace members refer to the hack package            | Affects local paths, workspace inheritance, and external publishing behavior                     |
| `dep-format-version`        | Generated dependency declaration format                             | Governs generated TOML formatting and certain publication-compatible declarations                |
| `registries`                | Alternate registry definitions                                      | Supports dependency matching in configurations using additional registries                       |

New configurations generated by `cargo hakari init` use dependency format 4 and resolver 2. Existing configurations may retain older defaults. Version 0.9.34 added support for resolver 3, whose consequential difference from resolver 2 is MSRV-aware version resolution rather than a new feature-unification model.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+3



Documentation: `https://docs.rs/cargo-hakari/latest/cargo_hakari/config/`

### 4.5. Exclusion semantics: three distinct mechanisms

The exclusion mechanisms are especially important because unconditional feature unification can be incorrect for libraries with intentionally incompatible feature combinations.

Traversal excludes remove a package while searching its dependency relationships. Its transitive dependencies are also excluded from that path, except where they remain reachable through other packages. This limits the configurations contributing to the computed feature union.

Final excludes remove selected packages from generated output but still account for their transitive dependencies. This allows an incompatible package to remain outside the workspace-hack without necessarily sacrificing unification of safe dependencies beneath it.

Structural excludes are detected automatically to prevent dependency cycles. As of `cargo-hakari` 0.9.39 and `hakari` 0.18.0, the algorithm excludes third-party packages that depend on the workspace-hack or on managed workspace members through relationships that would create cycles. Their own dependencies can still be considered for unification.

Typical reasons to investigate exclusion behavior include mutually exclusive crypto backends, features that select incompatible implementations, published workspace members reintroduced through `[patch]`, or other feature combinations that cannot safely be forced on simultaneously.

This is not merely an optimization setting: incorrectly including mutually incompatible feature combinations may cause build failures or alter intended dependency behavior.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+2



### 4.6. Target/host feature unification

Cargo distinguishes normal target dependencies from dependencies compiled for the build host, including build scripts and procedural macros. Resolver 2 and 3 prevent some feature sets from being merged across those contexts.

Hakari exposes `UnifyTargetHost` to control a separate feature-stabilization decision:

| Strategy                | Behavior                                                                  | Tradeoff                                                                                  |
| ----------------------- | ------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------- |
| `None`                  | Do not unify features across target and host contexts                     | Most conservative; fewer artificial feature activations, potentially more build variants  |
| `UnifyIfBoth`           | Unify only packages already needed in both contexts                       | Stabilizes shared host/target dependencies without introducing extra host-side instances  |
| `ReplicateTargetOnHost` | Also replicate target-only unification dependencies into the host context | More aggressive consolidation; may increase host-side compilation work                    |
| `Auto`                  | Select between the latter two strategies based on workspace structure     | Default; accounts for procedural macros and workspace packages used as build dependencies |

This is a potentially significant optimization mechanism in workspaces containing procedural macros, code generators, and build-time tooling.

However, feature unification does not erase genuine host/target compilation differences. A cross-compiled target artifact and a host-executed build dependency may still require separate compiled outputs even when their feature sets match.

Hakari's current documentation explicitly identifies full cross-compilation simulation as unfinished work. The `unify-target-host` strategies should therefore be understood as feature-set consolidation mechanisms, not complete cross-compilation planning.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+1



Reference: `https://docs.rs/hakari/latest/hakari/enum.UnifyTargetHost.html`

### 4.7. Programmatic API: the `hakari` crate

The Rust library exposes the computation and manifest-management functionality independently of the CLI.

| API family                                                | Functionality                                                                                                   |
| --------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------- |
| `HakariBuilder::new()`                                    | Construct a computation over a `guppy::PackageGraph`, optionally referencing an existing workspace-hack package |
| `set_platforms()` / `set_resolver()`                      | Configure simulated platforms and Cargo feature-resolution rules                                                |
| `add_traversal_excludes()` / `add_final_excludes()`       | Define package exclusions at different stages                                                                   |
| `set_unify_target_host()`                                 | Configure host/target feature-set consolidation                                                                 |
| `set_output_single_feature()`                             | Control inclusion of single-feature-set packages                                                                |
| `compute()`                                               | Run the unification analysis and produce a `Hakari` result                                                      |
| `Hakari::computed_map` / `output_map`                     | Inspect all simulated feature sets versus dependencies selected for output                                      |
| `Hakari::to_toml_string()`                                | Serialize generated dependency declarations                                                                     |
| `Hakari::read_toml()` / `HakariCargoToml`                 | Read and update the managed section of an existing manifest                                                     |
| `manage_dep_ops()` / `add_dep_ops()` / `remove_dep_ops()` | Derive workspace-member manifest edit operations, with `cli-support` enabled                                    |
| Explanation and verification modules                      | Inspect why unification occurred and evaluate the simulated unification property                                |

The separation between computation, serialization, and filesystem mutation is particularly valuable. The computed graph and prospective manifest edits can be examined or diffed independently of applying them.

Generated manifest content is delimited by Hakari section markers, with unrelated manifest content preserved. This reduces the amount of bespoke TOML transformation machinery required to maintain a workspace-hack.

The API also exposes summaries and serialization facilities for configuration reuse, as well as a dedicated explanation module.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+1



Direct reference: `https://docs.rs/hakari/latest/hakari/struct.HakariBuilder.html`

### 4.8. Publishing and external dependency boundaries

Workspace-hack dependencies are straightforward inside a workspace but can become problematic when individual packages are published or referenced externally by Git or path.

Hakari documents two publication patterns:

Temporary removal. `cargo hakari publish -p <crate>` temporarily removes the workspace-hack dependency during publication, then restores it afterward. An interrupted command may require restoring declarations manually through `manage-deps`.

Published stub package. A uniquely named, essentially empty workspace-hack package is published to a registry. Workspace members reference the registry package by version, while the workspace root applies a local `[patch]` directing Cargo to the real generated workspace-hack.

The second arrangement relies on an important Cargo boundary: the root workspace's `[patch]` applies within that workspace, while an external consumer of a Git/path dependency normally resolves the published stub rather than inheriting the source workspace's large generated feature-unification dependency set.

The line styles `version-only` and `workspace-dotted` support this model. By contrast, unconditional local path dependencies can cause external consumers to import the full workspace-hack dependency graph.

These publication constraints make Hakari partly a dependency-scope management tool, not solely a build-speed tool.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+1



References:

`https://docs.rs/cargo-hakari/latest/cargo_hakari/publishing/`

`https://docs.rs/cargo-hakari/latest/cargo_hakari/patch_directive/index.html`

### 4.9. Performance characteristics and limitations

Hakari's potential benefit comes primarily from reducing feature-set-induced cache fragmentation across repeated builds.

That distinction matters for interpreting performance claims.

The project reports, for a moderately large workspace with several hundred third-party dependencies, cumulative build-time improvements of approximately 1.7× in its example workload, with individual commands ranging from around 1.1× to 100×. These are upstream observations of particular repeated-build sequences, not a general speedup guarantee.

The tradeoff is that the first build—or a build following `cargo clean` or a compiler upgrade—may compile more features and dependencies than would otherwise be necessary. `cargo check` can benefit disproportionately from subsequent reuse because it avoids some linking costs present in full builds.

Additional limits include:

- Feature masking: a workspace-hack can activate features that a participating crate forgot to declare. That crate might compile within the workspace but fail when published or built without the hack.
- Mutually exclusive features: some libraries cannot safely enable every possible feature together; exclusions may be necessary.
- Changing dependency resolution: lockfile changes can change which packages and feature sets must be generated.
- External dependency pollution: unmanaged Git/path publication arrangements may propagate the workspace-hack's transitive graph.
- Incomplete configuration coverage: simulation covers defined build and platform cases, not every actual compiler configuration.
- No general elimination of rebuilds: source changes, build-script output changes, rustc flags, profiles, target triples, and toolchain changes remain relevant.

The project recommends checking `Cargo.lock` into version control even for library-only workspaces using Hakari, because dependency patch updates can change the generated workspace-hack contents.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+1



### 4.10. Current release details

`cargo-hakari` 0.9.39, released September 19, 2026, includes several consequential changes:

- Improved output from `cargo hakari explain`.
- Strict TOML 1.1 configuration parsing.
- Minimum supported Rust version raised to 1.91.
- Updated built-in target-platform definitions.
- Structural-exclusion fixes preventing dependency cycles.
- Improved `manage-deps` behavior for workspace members whose existing hack dependency was build-only, dev-only, optional, or target-specific.
- Upgrade of its bundled `guppy` dependency to 0.19.0, including Cargo simulation fixes.

These are released capabilities, not planned work. The separately published `guppy` crate has subsequently reached version 0.19.1; that should not be confused with the version bundled into `cargo-hakari` 0.9.39.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Docs.rs+2



Changelog: `https://docs.rs/crate/cargo-hakari/latest/source/CHANGELOG.md`

## 5. Ecosystem interactions and architectural distinctions

### 5.1. Where `guppy` fits

`guppy` deserves explicit attention because it supplies much of the graph functionality beyond basic metadata extraction and underpins Hakari's simulation algorithm.

Its core structures include `PackageGraph`, `PackageMetadata`, `PackageLink`, `PackageQuery`, `PackageSet`, `FeatureGraph`, `FeatureQuery`, `FeatureSet`, and `CargoSet`.

| Layer            | Native responsibility                                                                              |
| ---------------- | -------------------------------------------------------------------------------------------------- |
| Cargo            | Authoritative dependency resolution and build execution                                            |
| `cargo_metadata` | Invoke Cargo and expose metadata and build-event records as Rust types                             |
| `guppy`          | Immutable package/feature graph models, traversal, reverse queries, and build-selection simulation |
| `hakari`         | Analyze simulated feature-set variation and compute a workspace-hack dependency union              |
| `cargo-hakari`   | Provide the end-user CLI, configuration, manifest updates, verification, and publishing support    |

`guppy` parses Cargo metadata itself; its `MetadataCommand` is not the same type as `cargo_metadata::MetadataCommand`.

The library supports graph traversal in either direction, edge-level dependency inspection, feature graphs in which vertices represent package-feature combinations, and Cargo build simulation through `FeatureSet::into_cargo_set()`.

Its graph is not necessarily acyclic, which matters for general-purpose graph algorithms and for Hakari's structural-exclusion logic. The `FeatureGraph` is constructed on demand and cached.

Additional guppy features include custom target specifications, custom `cfg` platform information, Rayon integration, and serialized build summaries.

This means that `cargo_metadata` is primarily an extraction and typed-data interface, while `guppy` is a higher-level dependency-graph analysis engine. Choosing the latter for substantive graph transformations can eliminate the need to reproduce traversal and feature-resolution machinery on top of metadata records.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust



Documentation: `https://docs.rs/guppy/latest/guppy/`

### 5.2. Distinguish four representations of a Cargo workspace

A useful architectural distinction is that Cargo tooling exposes several related but non-equivalent representations.

| Representation           | Fundamental unit                         | Strongest use                                                   | What it cannot establish alone                                     |
| ------------------------ | ---------------------------------------- | --------------------------------------------------------------- | ------------------------------------------------------------------ |
| Manifest declarations    | Package and declared dependency          | Discover intended dependencies, constraints, features, targets  | Exactly which dependency features are active                       |
| Resolved package graph   | `PackageId` and resolved edges           | Dependency identity, reachability, reverse traversal            | Exact rustc build units                                            |
| Feature-activation graph | Package-feature relationships            | Explain why features are enabled                                | Whether another command will reuse the same compiled artifact      |
| Build-event stream       | Cargo artifacts and compiler diagnostics | Observe generated outputs, build freshness, compilation results | Complete counterfactual behavior across other build configurations |

`cargo_metadata` covers the first two and parses the fourth. `cargo tree` makes the second and third readily explainable. `guppy` models package and feature graphs and can simulate build selections. Hakari changes manifests to influence which feature sets future Cargo invocations resolve.

Conflating these representations is a common source of errors in custom dependency-analysis systems.

### 5.3. Optimization targets are distinct

| Objective                                               | Relevant mechanism                                    | Important boundary                                                               |
| ------------------------------------------------------- | ----------------------------------------------------- | -------------------------------------------------------------------------------- |
| Identify multiple versions of one dependency            | `cargo tree -d`                                       | Version convergence may require dependency updates; Hakari does not perform them |
| Explain why a feature is active                         | `cargo tree -e features -i`                           | Explains the selected graph, not all possible builds                             |
| Programmatically inventory packages and edges           | `cargo_metadata`                                      | Graph traversal and transformation remain caller responsibilities                |
| Analyze many feature or target configurations           | `guppy`                                               | Simulation must still respect selected Cargo/resolver/platform assumptions       |
| Stabilize dependency features across workspace commands | Hakari                                                | May deliberately enable more features than each isolated build needs             |
| Confirm actual built outputs                            | Cargo JSON messages through `cargo_metadata::Message` | Artifact observations still depend on the command, profile, and toolchain        |
| Diagnose slow compiler work                             | Cargo timings and other build instrumentation         | Package graphs alone do not measure compilation cost                             |

This separation helps avoid using Hakari for a package-version duplication problem, or treating a `cargo tree` listing as direct evidence of runtime build performance.

### 5.4. Capability composition

Several higher-level systems follow naturally from combining native capabilities.

Structured dependency intelligence. Cargo metadata supplies package IDs, sources, targets, and edges; guppy supplies traversal and feature reasoning; Cargo tree supplies explainable feature provenance. Together, these provide more reliable dependency analysis than ad hoc manifest parsing or string-based tree interpretation.

Build-artifact attribution. A package graph can be joined to Cargo JSON compilation events using package and target identifiers. This distinguishes declared or resolved dependencies from artifacts actually produced in a particular run. It supports artifact discovery, diagnostics association, and observation of compilation freshness without requiring a bespoke output parser.

Feature-configuration analysis. A selected Cargo metadata snapshot and `cargo tree` feature explanation can describe one configuration. Guppy's simulations can characterize multiple selections; Hakari's computed map then identifies where feature-set variation might be reduced. This is a progression from description to counterfactual analysis to declarative optimization.

Generated-manifest governance. Hakari's computation, TOML generation, diffing, explanation, and dry-run validation provide a coherent mechanism for keeping workspace-hack declarations synchronized with the dependency graph. They make manual feature-union tracking and custom manifest-editing logic largely unnecessary.

These are general architectural possibilities, not guarantees that a particular application benefits from all of the components. Their relative value depends on whether the task is primarily extraction, explanation, simulation, or build optimization.

## 6. Key technical constraints

| Issue                              | Consequence                                                                                               |
| ---------------------------------- | --------------------------------------------------------------------------------------------------------- |
| Package IDs versus names           | Multiple versions and sources can share a name; graph operations should preserve actual package identity  |
| Declared versus activated features | Manifest feature definitions are not a valid substitute for an active resolution                          |
| Cargo version dependence           | Metadata and resolver results reflect the Cargo executable and its configuration                          |
| `--no-deps`                        | Useful for workspace discovery but does not contain a resolved dependency graph                           |
| Target-platform selection          | Unfiltered metadata and default `cargo tree` use different target scopes                                  |
| Build-unit identity                | Package/feature identity does not fully encode target/host, profile, compiler settings, or artifact reuse |
| Hakari exclusions                  | Necessary to avoid invalid feature combinations and graph cycles                                          |
| Hakari generation consistency      | Depends on the dependency graph and lockfile remaining synchronized                                       |
| Cross-compilation                  | Hakari does not yet provide full cross-compilation simulation                                             |
| Publication                        | Workspace-hack dependency declarations must be handled deliberately at external boundaries                |
| Rust toolchain requirements        | `cargo_metadata` 0.23.1 requires Rust 1.86; `cargo-hakari` 0.9.39 requires Rust 1.91                      |

In addition to these constraints, `cargo_metadata` and the Hakari libraries expose non-exhaustive Rust structures. Client code should account for schema evolution and should not assume that every field or enum is permanently closed to future additions.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Docs.rs+3



## 7. Primary source map

The following primary references cover the technical surface in greater detail.

| Topic                                           | Direct URL                                                                         |
| ----------------------------------------------- | ---------------------------------------------------------------------------------- |
| Cargo metadata JSON format and command behavior | `https://doc.rust-lang.org/cargo/commands/cargo-metadata.html`                     |
| Cargo tree options and feature explanations     | `https://doc.rust-lang.org/cargo/commands/cargo-tree.html`                         |
| Cargo feature resolution                        | `https://doc.rust-lang.org/cargo/reference/resolver.html`                          |
| `cargo_metadata` crate                          | `https://docs.rs/cargo_metadata/latest/cargo_metadata/`                            |
| `MetadataCommand` API                           | `https://docs.rs/cargo_metadata/latest/cargo_metadata/struct.MetadataCommand.html` |
| Cargo build-message parser                      | `https://docs.rs/cargo_metadata/latest/cargo_metadata/enum.Message.html`           |
| Cargo-hakari CLI                                | `https://docs.rs/cargo-hakari/latest/cargo_hakari/`                                |
| Hakari algorithm and rationale                  | `https://docs.rs/cargo-hakari/latest/cargo_hakari/about/`                          |
| Hakari configuration                            | `https://docs.rs/cargo-hakari/latest/cargo_hakari/config/`                         |
| Hakari Rust library                             | `https://docs.rs/hakari/latest/hakari/`                                            |
| `HakariBuilder` API                             | `https://docs.rs/hakari/latest/hakari/struct.HakariBuilder.html`                   |
| Hakari host/target strategies                   | `https://docs.rs/hakari/latest/hakari/enum.UnifyTargetHost.html`                   |
| Guppy graph API                                 | `https://docs.rs/guppy/latest/guppy/`                                              |
| Cargo-hakari changelog                          | `https://docs.rs/crate/cargo-hakari/latest/source/CHANGELOG.md`                    |

Overall: `cargo_metadata` is the foundational typed interface to Cargo's workspace and build information; `cargo tree` is the native dependency and feature-provenance explainer; `guppy` supplies higher-order graph analysis and simulation; and `cargo-hakari`/`hakari` turn that simulated feature information into persistent workspace-wide build optimization. Their strongest combined value is the ability to distinguish what is declared, what resolves, why it resolves, what actually builds, and which avoidable feature variations can be removed—without reimplementing Cargo's native machinery.

# Addendum: Advanced Rust Dependency Graph Analysis, Cargo Simulation, and Build Intelligence

**Companion to:** *Rust Dependency Graph Tooling: cargo_metadata, cargo tree, and cargo-hakari*  
**Research date:** November 1, 2026

**Version scope:** `cargo_metadata` 0.23.1; `guppy` 0.19.1; `guppy-summaries` 0.7.1; `hakari` 0.18.0; `cargo-hakari` 0.9.39; `determinator` 0.13.0; stable Cargo 1.99 and separately identified nightly Cargo capabilities.

This addendum extends the original guide's sections 2, 4, and 5. It concentrates on functionality not substantially developed there: programmatic graph computation, conditional feature semantics, build-selection simulation, dependency snapshots, change-impact analysis, Cargo's compilation-unit introspection, and additional Hakari correctness boundaries.

## A. `guppy`: advanced package-graph computation

### A.1. Graph completeness and query scope

`guppy::graph::PackageGraph` constructs an immutable dependency graph from Cargo metadata. Unlike the typed extraction exposed by `cargo_metadata`, it provides a native analysis model with query resolution, dependency-aware iteration, set algebra, feature relationships, and simulation of selected Cargo behavior.

The fidelity of these operations depends on the metadata supplied.

- **`--all-features`:** Upstream recommends generating metadata with all features enabled to obtain a broad dependency graph. Optional dependencies absent from the metadata resolution cannot be recovered by later graph traversal.
- **`--no-deps`:** `guppy` supports this in a restricted mode, but dependency queries do not work and third-party package information is absent.
- **Platform filtering:** Filtering Cargo metadata can reduce the relationships available to subsequent analysis. A platform-restricted graph should not be mistaken for an unrestricted inventory from which arbitrary alternative-target closures can be reconstructed.
- **Version resolution:** `guppy` analyzes versions already selected by Cargo. It does not independently solve the original manifest's semver constraints to create a new lockfile.

The principal construction paths are `PackageGraph::from_command`, `from_metadata`, and `from_json`. The graph retains package identities, resolved relationships, and metadata needed for subsequent queries.

Workspace selection also has several materially different forms.

| API | Result |
|---|---|
| `resolve_workspace()` | Workspace members only; no transitive closure |
| `query_workspace().resolve()` | Workspace members and reachable dependencies |
| `resolve_all()` | Every package represented in the metadata graph, including potentially unreachable packages |
| `resolve_ids(...)` | Exactly the requested package identities |
| `query_workspace_names(...)` / `query_workspace_paths(...)` | Dependency queries rooted in explicitly selected workspace packages |

`resolve_all()` can differ from the workspace closure when Cargo metadata retains packages introduced through `[patch]` or `[replace]` relationships that are not actually reachable from workspace members. This distinction matters when classifying unused or superseded resolved packages.

**Architectural implication:** Workspace inventory, resolved package inventory, and transitive dependency closure should be treated as separate queries rather than interchangeable views.

Primary source: `https://guppy-rs.github.io/guppy/rustdoc/guppy/graph/struct.PackageGraph.html`

### A.2. Native traversal, reachability, and set operations

`guppy` provides a complete package-query abstraction rather than requiring callers to construct adjacency maps and implement graph algorithms themselves.

`PackageGraph::query_forward`, `query_reverse`, and `query_directed` construct queries rooted at selected package identities. `PackageQuery::resolve()` follows all represented links in the requested direction and returns a `PackageSet`.

`PackageQuery::resolve_with()` and `resolve_with_fn()` permit custom traversal decisions through `PackageLinkVisitor` or a callback receiving dependency-link context. This supports analysis restricted by dependency kind, selected packages, source characteristics, or other edge predicates.

The distinction between filtering a resolved set and filtering traversal edges is important. A post-traversal filter merely removes members from a result; a traversal visitor can prevent an entire dependency branch from being reached.

`PackageGraph::depends_on(a, b)` and `directly_depends_on(a, b)` provide reachability predicates. The former treats a package as transitively dependent on itself, while the latter requires a real self-loop for a package to depend directly on itself. `new_depends_cache()` offers an optional cache for repeated dependency tests.

Once resolved, `PackageSet` supports:

| Functionality | API family |
|---|---|
| Union, intersection, difference, symmetric difference | `union`, `intersection`, `difference`, `symmetric_difference` |
| Predicate selection | `filter`, `partition`, `filter_partition` |
| Dependency-aware iteration | `packages`, `package_ids`, `links` |
| Roots of selected subgraphs | `root_ids` |
| Reusing a result as another query's starting set | `to_package_query` |
| Converting packages into feature selections | `to_feature_set` |
| Graphviz-compatible presentation | `display_dot` |

These operations enable graph slicing, dependency impact calculation, package classification, and comparison of alternative closures without bespoke traversal infrastructure.

**Important constraint:** `PackageSet` operations require sets backed by the same underlying `PackageGraph`; mixing sets from different graphs can panic. Their set algebra is therefore suitable for comparisons within one resolution snapshot, not directly for dependency differences across commits.

`PackageLink` also preserves edge-specific information that package-level nodes cannot express. Its `normal()`, `build()`, `dev()`, and `req_for_kind()` methods expose distinct dependency-kind requirements, while `dep_name()`, `resolved_name()`, and related methods distinguish manifest aliases from resolved crate names.

A single dependency name is not necessarily a unique edge identifier. For example, different target-specific declarations may use the same local name while resolving to different package versions. Likewise, multiple declarations may resolve to one package. Edge-level reasoning should therefore preserve both endpoint identity and declaration context.

Primary sources:

`https://guppy-rs.github.io/guppy/rustdoc/guppy/graph/struct.PackageQuery.html`

`https://guppy-rs.github.io/guppy/rustdoc/guppy/graph/struct.PackageSet.html`

`https://guppy-rs.github.io/guppy/rustdoc/guppy/graph/struct.PackageLink.html`

### A.3. Dependency cycles and topological processing

A Cargo package graph is not guaranteed to be acyclic, even though a valid compilation-unit graph must respect executable build-order dependencies.

The discrepancy arises because a package-level node combines library, binary, test, benchmark, and other target contexts. Development dependencies can produce apparent package cycles without creating an impossible compiler dependency cycle.

For example, package A may depend normally on package B, while B's tests depend on A. A package graph can then contain A ↔ B, although the actual build has separate units for B's tests and B's library.

`PackageGraph::cycles()` exposes strongly connected components through the `Cycles` structure. Its significant operations include:

- `all_cycles()`: identify cyclic strongly connected components, including nontrivial multi-node components and self-loops.
- `is_cyclic(a, b)`: determine whether two package IDs participate in a common nontrivial cycle.
- Dependency-aware ordering: `PackageSet::packages()` and related operations handle these relationships using non-development dependency ordering.

For a package graph, removing dev-only dependency edges restores the acyclic dependency ordering relevant to normal compilation. `guppy` exploits this property rather than requiring consumers to collapse all cyclic components manually.

The resulting order must not be interpreted as an ordering of tests, build scripts, and compiler invocations. If an analysis needs development targets as independent executable units, it must represent that distinction explicitly or obtain Cargo's compilation-unit graph.

There is an additional subtlety in the reachability API: `depends_on(a, a)` returns true by definition, whereas `Cycles::is_cyclic(a, a)` reports true only when A actually participates in a graph cycle.

**Architectural implication:** Native cycle handling and dependency-aware iteration make package-level impact propagation practical without a general-purpose graph-processing layer, provided the consumer understands that development-target semantics remain compressed into package nodes.

Primary source: `https://guppy-rs.github.io/guppy/rustdoc/guppy/graph/struct.Cycles.html`

## B. Feature graphs and build-selection simulation

### B.1. Feature relationships are conditional

`PackageGraph::feature_graph()` constructs and caches a `FeatureGraph`. Its vertices represent package-feature combinations, including an implicit base feature for each package; its edges represent relationships among features and dependency activation.

The feature graph exposes direct and transitive queries, feature metadata, default-feature membership, dependency relationships, cycle inspection, and construction warnings through `build_warnings()`.

The most consequential distinction is between **feature-graph reachability and actual feature activation**.

`FeatureQuery::resolve()` traverses all reachable feature links without evaluating activation conditions. Its result may therefore contain features associated with development dependencies, inactive target conditions, or weak optional-dependency relationships that would not activate in a specific Cargo invocation.

A weak feature reference such as `dep?/feature` is conditional on the dependency being otherwise activated. `FeatureQuery::resolve_with()` and `resolve_with_fn()` permit more selective traversal, but correctly implementing conditional activation is more complicated than performing ordinary graph reachability. The visitor can be invoked twice for a weak relationship as its conditions are processed.

`FeatureGraph::depends_on()` also reports conditional reachability. It should not be interpreted as proof that the relationship is active in a given build.

`FeatureSet` provides set operations, package-level feature inspection through `features_for()` and `packages_with_features()`, and conversion back to a `PackageSet`. This makes it useful for modeling feature relationships independently of any one build command.

For build-specific reasoning, however, the preferred next operation is `FeatureSet::into_cargo_set()` or an explicit `CargoSet` construction.

Primary sources:

`https://guppy-rs.github.io/guppy/rustdoc/guppy/graph/feature/struct.FeatureGraph.html`

`https://guppy-rs.github.io/guppy/rustdoc/guppy/graph/feature/struct.FeatureQuery.html`

`https://guppy-rs.github.io/guppy/rustdoc/guppy/graph/feature/struct.FeatureSet.html`

### B.2. `CargoSet`: simulated Cargo build activation

`guppy::graph::cargo::CargoSet` evaluates a selected feature set under explicit Cargo-simulation options.

Its core constructor is `CargoSet::new(initials, features_only, options)`.

The two feature inputs have different meanings:

- **`initials`:** The selected workspace packages and features whose dependency closures are traversed.
- **`features_only`:** Additional workspace feature inputs that influence feature unification without independently requiring their full dependency closures to be included.

This distinction can model cases in which multiple workspace packages participate in one Cargo invocation, but the analysis is interested primarily in the resulting build closure of a subset.

When no extra unification inputs are needed, `FeatureSet::into_cargo_set()` provides the simpler interface.

Both input sets are expected to refer to workspace packages. Upstream explicitly warns that the behavior of `CargoSet::new()` with initial feature selections outside the workspace is not defined in the intended model and can be surprising.

A `CargoSet` exposes distinct outputs for compilation contexts:

| API | Interpretation |
|---|---|
| `target_features()` | Packages and features associated with target-side compilation |
| `host_features()` | Packages and features associated with build-host compilation |
| `all_features()` | Both sets, retaining platform distinction |
| `target_direct_deps()` / `host_direct_deps()` | Direct-dependency package selections for the corresponding contexts |
| `proc_macro_links()` | Procedural-macro dependency relationships crossing into host compilation |
| `build_dep_links()` | Build-dependency links from target-side packages to host-side packages |
| `target_links()` / `host_links()` | Dependency relationships internal to the corresponding compilation context |

These outputs explain why the same package version may participate in both host and target compilation, potentially with distinct feature sets.

The `CargoSet::with_cargo_link_visitor()` constructor permits additional edge-level filtering or collection during simulation. Its visitor runs after package omissions specified through `CargoOptions` but before subsequent build-selection decisions.

`CargoSetInputs` also captures reusable simulation settings, including the feature-only inputs and Cargo options. Its `to_cargo_set()` method allows repeated simulations of different initial selections under common assumptions.

This offers a useful separation between immutable dependency knowledge, reusable simulation policy, and individual build-selection queries.

**Important boundary:** `CargoSet` is a modeled approximation of Cargo activation, not a record of compiler invocations. It does not by itself establish compiler fingerprints, build-script outputs, optimization profiles, incremental-cache state, or whether an artifact was reused.

Primary sources:

`https://guppy-rs.github.io/guppy/rustdoc/guppy/graph/cargo/struct.CargoSet.html`

`https://docs.rs/guppy/0.19.1/guppy/graph/cargo/struct.CargoSetInputs.html`

### B.3. Simulation controls and resolver compatibility

`CargoOptions` changes the semantics of a simulation through `set_resolver`, `set_include_dev`, `set_initials_platform`, `set_target_platform`, `set_host_platform`, `set_platform`, and `add_omitted_packages`.

The following distinctions affect correctness:

**Resolver version.** In `guppy` 0.19.1, `CargoOptions::new()` defaults to `CargoResolverVersion::V3`. This is a library default, not an inference about the workspace being analyzed. A workspace using resolver 1 or 2 requires the corresponding simulation setting for matching feature semantics.

Resolver 3 has the same feature-unification behavior as resolver 2; its MSRV-aware dependency-version selection is ordinarily handled earlier by Cargo when constructing the metadata graph.

**Development dependencies.** `set_include_dev(true)` follows development dependencies of the initial packages, not development dependencies recursively throughout the entire transitive closure. This matches Cargo's distinction between targets selected for development builds and ordinary transitive dependencies.

**Platform selection.** Target and host platforms can be configured separately. A simulation that uses unrestricted platform specifications can include relationships not active for a concrete target. Separate host and target specifications are important for cross-compilation reasoning.

**Initial compilation context.** `set_initials_platform()` controls how selected initial packages are treated for host or target compilation. This matters in less common build arrangements involving procedural macros or packages intentionally built for the host.

**Omitted packages.** `add_omitted_packages()` excludes incoming edges to specified packages. This is useful for counterfactual dependency analysis, but the resulting `CargoSet` no longer represents an unmodified Cargo invocation.

**Custom platforms.** Optional `custom-cfg-platforms` and `custom-platforms` crate features provide support for `rustc --print=cfg` configurations and JSON target specifications, respectively.

These options make build simulation an explicit, configurable analytical operation rather than a single universal interpretation of a workspace.

`CargoSet` should not be assumed to implement Cargo's separately experimental workspace feature-unification modes unless the particular `guppy` release documents support for them.

Primary sources:

`https://docs.rs/guppy/0.19.1/src/guppy/graph/cargo/cargo_api.rs.html`

`https://guppy-rs.github.io/guppy/rustdoc/guppy/graph/cargo/enum.CargoResolverVersion.html`

## C. Durable dependency snapshots and change analysis

### C.1. `guppy-summaries`: serialization and comparison

`guppy-summaries` 0.7.1 is a separate, lightweight crate for serializing and comparing modeled Cargo build selections.

When `guppy` is compiled with its optional `summaries` feature, `CargoSet::to_summary()` produces a serializable `Summary`. It records packages and feature selections separately for host and target, along with package identity, source information, and selection status.

`Summary::diff()` compares two such records and can identify:

- Added and removed packages.
- Package-version changes.
- Changes in source locations or identities.
- Added and removed features.
- Differences between host and target package inventories.
- Changes in how packages participate in the modeled selection.

The result can be rendered through `SummaryDiff::report()` for human-readable change explanations.

A significant architectural distinction is that `PackageSet` and `FeatureSet` operations generally compare entities within the same immutable graph, whereas serialized summaries can be compared across different graph snapshots or source revisions.

This permits durable dependency-state comparison without retaining the entire original in-memory graph.

However, the summaries do not record execution timings, compiler cache hits, linker activity, or the full set of actual build units. They are **modeled dependency-selection snapshots**, not build-execution traces.

The `guppy` summaries module also exposes serialized representations of package sets and Cargo simulation inputs, supporting reconstruction or comparison of selections subject to matching package identities and feature definitions.

Primary sources:

`https://docs.rs/guppy-summaries/0.7.1/guppy_summaries/`

`https://guppy-rs.github.io/guppy/rustdoc/guppy/graph/summaries/index.html`

### C.2. `determinator`: repository-change impact analysis

`determinator` 0.13.0 is a separate companion crate built on `guppy`. Its purpose is to determine which workspace packages and corresponding builds may be affected by changes between two source revisions.

A `Determinator` instance receives the old and new `PackageGraph` representations, a list of changed file paths, and optional rules governing how changes propagate.

Its analysis combines:

1. **File ownership:** Determine the workspace package associated with each changed file.
2. **Dependency changes:** Compare package dependencies, resolved versions, and feature activation across graph snapshots.
3. **Build-selection simulation:** Evaluate package dependency sets under multiple feature selections, including no features, default features, and all features.
4. **Configured impact rules:** Apply rules for files outside packages, ignored paths, virtual dependencies, or changes that require a broader invalidation.

The computed `DeterminatorSet` exposes an `affected_set` of packages considered affected directly or transitively.

This can eliminate substantial bespoke logic for change-to-package mapping and affected-test selection.

Its limitations are consequential:

- It does not execute build scripts or automatically interpret their dynamic `rerun-if-changed` and `rerun-if-env-changed` declarations.
- It does not independently detect all changes to external environment variables, system libraries, or services.
- Path dependencies outside the workspace may change without corresponding version or source-identity changes.
- File ownership and manifest `include`/`exclude` semantics do not always align.
- Some changes outside known packages require explicit rules or conservative full-workspace invalidation.

The upstream project explicitly acknowledges possible false negatives and false positives. Its output is a change-impact estimate, not a proof that an unaffected package cannot fail under any environment or feature combination.

**Architectural distinction:** `determinator` predicts which builds or tests warrant execution; Cargo performs compilation and cache invalidation; compiler caches such as `sccache` address artifact reuse. These mechanisms operate at different levels and can be combined without being interchangeable.

Primary source: `https://guppy-rs.github.io/guppy/rustdoc/determinator/`

## D. Native Cargo feature unification and compilation-unit introspection

### D.1. Experimental native workspace feature unification

Cargo's nightly-only `-Zfeature-unification` capability introduces a native alternative to some of the feature-stabilization behavior targeted by Hakari.

It enables the `resolver.feature-unification` configuration setting with three documented modes:

| Mode | Semantics |
|---|---|
| `selected` | Unify dependency features across packages selected for the current invocation; default behavior |
| `workspace` | Unify dependency features across all workspace members, even when only a subset is selected |
| `package` | Resolve feature activation more independently per selected package, allowing additional dependency compilation variants |

For example, nightly Cargo can activate a configuration containing `resolver.feature-unification = "workspace"` when invoked with `-Zfeature-unification`. Without that unstable feature enabled, Cargo ignores this configuration setting.

**Relationship to Hakari.** Native workspace unification acts directly within Cargo's resolver and build-selection machinery. Hakari instead generates dependency declarations that request common feature sets using ordinary Cargo semantics.

Both can reduce feature-set variation across separately selected workspace builds, but their behavior and constraints are not identical.

Native workspace unification avoids generated workspace-hack dependencies, manifest rewriting, and workspace-hack publishing concerns. Conversely, Hakari provides explicit simulation, generated declarations, exclusions, explanation, verification, and host/target unification controls.

The experimental Cargo mode may also activate features a package failed to declare itself. Cargo's build-performance guidance warns that this can mask missing feature declarations, a problem also relevant to Hakari-generated unification.

It is not established that Cargo's experimental mode is compatible with every workspace currently using Hakari or that it provides equivalent handling of mutually exclusive features and target-specific contexts.

As of this addendum's research date, **`-Zfeature-unification` is experimental and should not be described as a stable Cargo replacement for Hakari**.

Primary sources:

`https://doc.rust-lang.org/nightly/cargo/reference/unstable.html#feature-unification`

`https://doc.rust-lang.org/nightly/cargo/guide/build-performance.html`

### D.2. `--unit-graph`: Cargo's actual planned compilation units

Nightly Cargo provides an experimental `--unit-graph` option:

`cargo +nightly build --unit-graph -Z unstable-options`

This emits JSON describing Cargo's internal compilation-unit dependency graph. The command constructs the graph without performing compilation.

Unlike `cargo metadata`, which represents packages and a resolution-level feature union, the unit graph identifies the distinct compiler units required by the selected command.

It can represent:

- Multiple compilation units associated with the same package.
- Different feature sets used for different compilation contexts.
- Dependencies between compiler units.
- Build-script and test-related intra-package dependencies.
- Relevant target and compilation-profile distinctions.

This resolves an important information gap in package-level analysis: a package node cannot independently represent every compiler-unit context associated with the package.

The conceptual distinction is:

| Representation | Models |
|---|---|
| `PackageGraph` | Resolved package identity and dependency relationships |
| `FeatureGraph` | Possible feature activation relationships |
| `CargoSet` | Simulated host/target feature selections |
| `--unit-graph` | Cargo's planned compiler-unit dependency graph for one command |
| Build-event messages | Artifacts and diagnostics reported during execution |

The unit graph is therefore particularly relevant when investigating duplicate compiler work that cannot be explained by duplicate package versions alone.

**Limitations:** The JSON interface is experimental, tied to Cargo's internal build model, and not guaranteed to remain schema-stable. It describes what Cargo plans to build, not which units actually ran, were fresh, or consumed the most resources.

It also inherits the selected command's features, targets, profile, and other compilation settings. A unit graph for `cargo build` does not automatically characterize `cargo test`, `cargo check`, or another workspace selection.

Primary source: `https://doc.rust-lang.org/nightly/cargo/reference/unstable.html#unit-graph`

## E. Build execution diagnostics beyond metadata messages

### E.1. Stable Cargo timing reports

`cargo build --timings` is available on stable Cargo and generates an HTML report under `target/cargo-timings/`.

Its principal value is visibility into compiler-unit execution rather than dependency existence.

The report includes unit durations, compilation concurrency, units blocked by dependency completion, build-script execution, and feature information associated with individual compiled units. It can reveal critical-path bottlenecks where a slow dependency delays numerous downstream compilations.

Its concurrency graph distinguishes actively running units, units waiting for available execution capacity, and units unable to proceed until dependencies complete.

The report does not expose the full internal parallelism of `rustc`. Cargo's jobserver coordination and pipelined compilation also affect interpretation of concurrency and execution overlap.

The ordinary `--timings` report is intended for human inspection, not a stable machine-readable timing protocol. Earlier experimental timing-format options, including JSON output, were removed from Cargo's unstable interface in the 1.94 nightly development cycle.

Primary source: `https://doc.rust-lang.org/cargo/reference/timings.html`

### E.2. Experimental persistent build analysis

Nightly Cargo's `-Zbuild-analysis` provides a more extensive observability mechanism.

When enabled, it records build-session data in JSONL logs under `$CARGO_HOME/log/`, with a unique session identifier for each invocation. These logs capture information including timing measurements and rebuild reasons.

The associated experimental commands include:

| Command | Purpose |
|---|---|
| `cargo report sessions` | Discover recorded build sessions |
| `cargo report timings` | Generate timing reports from earlier sessions |
| `cargo report rebuilds` | Inspect explanations for recompilation and cache invalidation |

This addresses a different problem from `cargo_metadata::Message::CompilerArtifact::fresh`.

The `fresh` flag states whether an artifact was already generated. It does not explain the complete reason a prior artifact became invalid.

Build-analysis records can expose rebuild causality, making them more relevant for diagnosing feature variation, compiler-setting changes, and other sources of repeated work.

Additional nightly instrumentation includes:

- `-Zsection-timings`: compiler-section timing information in Cargo timing output.
- `-Zmem-stats`: peak resident memory for individual build units, including compiler, rustdoc, and build-script processes on supported platforms.

These are experimental capabilities. Their log formats, configuration, and reporting interfaces should not be assumed to be stable integrations.

Primary source: `https://doc.rust-lang.org/nightly/cargo/reference/unstable.html#build-analysis`

### E.3. Additional structured build-event information

The original guide describes `cargo_metadata::Message::parse_stream()`. Two event structures warrant additional attention.

**`Artifact` / `CompilerArtifact`.** Beyond package ID, target, features, filenames, executable, and freshness, `Artifact` exposes `manifest_path` and `ArtifactProfile`. These permit more precise association of artifacts with selected package manifests and compilation-profile characteristics.

**`BuildScript` / `BuildScriptExecuted`.** The structured event exposes:

| Field | Information |
|---|---|
| `package_id` | Package whose build script produced the result |
| `linked_libs` | Libraries requested for native linking |
| `linked_paths` | Library search paths |
| `cfgs` | Configuration flags passed into compilation |
| `env` | Environment variables supplied to compilation |
| `out_dir` | Build-script output directory |

These records can support native-link dependency inspection, generated-configuration indexing, and diagnosis of build-script-influenced compilation.

The event stream remains observational. It does not provide a complete record of build-script inputs, command fingerprints, or all reasons for artifact reuse. Consumers must also check Cargo's process exit status rather than treating receipt of a final JSON event as the sole indication of successful execution.

Primary sources:

`https://docs.rs/cargo_metadata/0.23.1/cargo_metadata/struct.Artifact.html`

`https://docs.rs/cargo_metadata/0.23.1/cargo_metadata/struct.BuildScript.html`

### E.4. Metadata `build_directory` compatibility

`cargo_metadata` 0.23.1 stabilized the `build_directory` field in its Rust data model.

This should not be interpreted as a guarantee that every Cargo executable supplies the field under all conditions. The Cargo metadata JSON reference still describes the corresponding field as unstable.

The distinction is between stability of the wrapper's typed API and availability or stability of a field emitted by the selected Cargo executable.

Code consuming metadata from heterogeneous toolchains should handle the field's optionality and should not infer build-artifact paths solely from its presence or absence.

Primary sources:

`https://docs.rs/crate/cargo_metadata/0.23.1/source/CHANGELOG.md`

`https://doc.rust-lang.org/cargo/commands/cargo-metadata.html`

## F. Additional Hakari inspection and correctness boundaries

### F.1. Programmatic structural-exclusion inspection

`hakari` 0.18.0 added `Hakari::structural_excludes()` and `Hakari::is_structural_excluded()`.

These expose packages automatically excluded to avoid cycles involving the workspace-hack package or managed workspace members.

Unlike user-configured traversal and final exclusions, structural exclusions are derived from graph topology and cannot be disabled through configuration.

Several implementation details matter:

- Structural exclusion follows normal and build dependency relationships, including target-specific relationships even if their target is not part of the configured simulation platforms.
- Dev-only relationships are ignored for structural cycle prevention because Cargo permits cycles involving development dependencies.
- The excluded packages themselves are absent from generated workspace-hack dependencies.
- Their transitive dependencies may still contribute to feature unification.
- `computed_map` omits structural-excluded packages as well as workspace packages.
- `is_structural_excluded()` returns false for workspace members, despite the fact that workspace packages are also not eligible as generated third-party hack dependencies.

The distinction prevents a misleading interpretation of the computed map: absence may reflect graph-structural exclusion, not a lack of dependency variation.

`output_map` and `computed_map` serve different roles. The former contains the declarations selected for generated TOML; the latter represents the wider simulated dependency results available for analysis. Exclusion inspection provides the additional information required to interpret those outputs.

Primary source: `https://guppy-rs.github.io/guppy/rustdoc/hakari/struct.Hakari.html`

### F.2. Excluded workspace members and managed declarations

`cargo hakari manage-deps` reconciles workspace-hack dependency declarations according to participation in Hakari computation.

Workspace members excluded through `traversal-excludes` or `final-excludes` are not merely ignored as computation inputs. The command removes their workspace-hack dependency edges, preventing excluded packages from inheriting feature activation through the generated hack package.

This is significant for packages with mutually exclusive feature requirements: excluding them from the computation while leaving their workspace-hack dependencies intact could undermine the intended exclusion.

The CLI's declaration-management semantics should therefore be understood as a coordinated change to which packages participate in the feature-unification mechanism, not only as generation of a central manifest.

Primary source: `https://docs.rs/cargo-hakari/0.9.39/cargo_hakari/config/index.html`

### F.3. Explicitly unsupported Hakari functionality

The upstream `hakari` documentation identifies three incomplete capabilities:

1. Full simulation of cross-compilation configurations.
2. Platform-specific package exclusions.
3. Arbitrary restriction of generated unification output to a chosen subset of packages.

The third limitation is distinct from Hakari's existing traversal and final exclusion mechanisms. Those can suppress packages, but they are not a general-purpose output-selection language guaranteeing arbitrary partial unification without changing other computed relationships.

Likewise, platform-specific exclusions would express different exclusion policies for different platforms; ordinary package-level exclusions do not provide that granularity.

These are documented future-work areas, not released functionality.

Primary source: `https://guppy-rs.github.io/guppy/rustdoc/hakari/`

## G. Additional extension and testing facilities

`guppy` exposes several optional capabilities that expand its usefulness as an embedded dependency-analysis engine.

**Custom platform evaluation.** The `custom-cfg-platforms` feature enables custom platform definitions using `rustc --print=cfg` output; `custom-platforms` adds JSON target-specification support and implies the former. This permits analysis of platform conditions beyond the library's built-in target inventory.

**Property-based testing.** With `proptest1`, `PackageGraph` can generate valid package IDs, dependency links, visitor configurations, and Cargo simulation options drawn from an existing graph. `FeatureGraph` similarly exposes strategies for feature IDs and feature sets. These strategies support testing graph algorithms against valid graph entities and varied selections without inventing synthetic IDs.

**Parallel iteration.** The `rayon1` feature integrates Rayon with selected graph operations. Upstream characterizes this as preliminary support rather than complete parallel-iterator coverage.

**Graph representation.** `PackageSet::display_dot()` supports direct Graphviz DOT serialization. This is useful when visual explanation is needed, but the representation is still a projection of the resolved package graph rather than Cargo's full compilation-unit graph.

These optional capabilities do not change the fundamental requirement that Cargo's metadata be sufficiently complete for the intended analysis.

Primary source: `https://guppy-rs.github.io/guppy/rustdoc/guppy/`

## H. Integrated capability distinctions

The additional functionality supports a more precise division of responsibilities across dependency intelligence, build prediction, and execution observability.

| Question | Native mechanism | Information boundary |
|---|---|---|
| Which packages are transitively reachable? | `guppy::PackageQuery` | One extracted resolution graph |
| Which relationships or features could activate? | `FeatureGraph` and conditional queries | Reachability is not build activation |
| Which features would a selected configuration activate? | `CargoSet` and `CargoOptions` | Simulated Cargo behavior |
| How does a modeled build selection differ from another? | `guppy-summaries` | Build-selection changes, not execution changes |
| Which workspace packages may be affected by source changes? | `determinator` | Rule-based impact approximation |
| How can feature variation across workspace builds be reduced? | Hakari or experimental Cargo workspace unification | Different mechanisms and support constraints |
| Which compiler units does Cargo plan to use? | Nightly `--unit-graph` | Planned execution structure |
| Which compilation outputs and diagnostics were reported? | `cargo_metadata::Message` | Observed command events |
| Which compiler units consumed time or caused rebuilds? | Cargo timings and experimental build analysis | Execution-specific measurements |

The important architectural progression is from **resolved graph → conditional feature model → simulated build selection → planned compiler units → observed execution**.

Each layer answers questions that the preceding layer cannot establish by itself. Combining the layers permits dependency reasoning, counterfactual build analysis, feature optimization, and compilation diagnostics without reimplementing Cargo's central resolver or compiler machinery.

The most consequential constraint is preserving those boundaries: a dependency path does not prove active feature selection; an active feature does not prove a distinct compilation unit; a planned unit does not prove execution; and an observed artifact does not independently explain its entire cache-invalidation history.