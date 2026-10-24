# Rust Build and Dependency Optimization: cargo-hakari, sccache, and cargo-shear

Technical capability reference | Research date: October 24, 2026

## 1. Executive orientation

`cargo-hakari`, `sccache`, and `cargo-shear` address three complementary sources of inefficiency in Rust development: redundant feature-specific compilation, repeated compilation of identical inputs, and unnecessary dependency declarations.

They operate at different layers of Cargo's build process and can be combined without fundamentally overlapping responsibilities.

### Verified release scope

| Tool         | Latest stable release verified | Primary function                                                                  |
| ------------ | ------------------------------ | --------------------------------------------------------------------------------- |
| cargo-hakari | 0.9.39 — September 19, 2026    | Dependency feature unification across workspace builds                            |
| sccache      | 0.18.0 — September 14, 2026    | Compiler output caching, with optional remote storage and distributed compilation |
| cargo-shear  | 1.14.0 — September 22, 2026    | Detection and removal of unused or misplaced dependency declarations              |

Sources: cargo-hakari's&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Docs.rs

, sccache's&#x20;

[image](https://www.google.com/s2/favicons?domain=https://github.com\&sz=32)

GitHub

, and cargo-shear's&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Docs.rs

&#x20;release records.



### Capability boundaries

| Dimension                              | cargo-hakari                                      | sccache                                             | cargo-shear                                                       |
| -------------------------------------- | ------------------------------------------------- | --------------------------------------------------- | ----------------------------------------------------------------- |
| Primary optimization                   | Reduce distinct dependency compilation variants   | Reuse previously produced compilation artifacts     | Reduce unnecessary declared dependencies                          |
| Operational layer                      | Cargo dependency graph and feature resolution     | Compiler invocation and artifact storage            | Cargo manifests and source analysis                               |
| Persistent changes                     | Generated manifest and workspace dependency edges | Cache entries and optional configuration            | Manifest edits when fixing                                        |
| Requires full compilation for analysis | No                                                | Compilation occurs on cache misses                  | No by default                                                     |
| Directly accelerates compilation       | Indirectly                                        | Yes, for cache hits                                 | Indirectly                                                        |
| Cross-project usefulness               | Within configured dependency graphs               | Yes, when cache keys are compatible                 | Analyzes each project independently                               |
| Principal tradeoff                     | Broader feature activation                        | Cache compatibility, storage, and resource overhead | Static analysis cannot prove all possible dependencies are unused |

The central distinction is:

- cargo-hakari reduces how many different versions of a dependency's feature configuration Cargo needs to compile.
- sccache reduces how often an equivalent compiler invocation must actually execute.
- cargo-shear reduces which dependencies are declared and potentially participate in resolution and builds.

These effects are complementary, but none subsumes the others. In particular, neither `cargo-hakari` nor `sccache` is a general mechanism for compiling third-party dependencies once and guaranteeing they never recompile.

A comprehensive build design also needs to account for Cargo's fingerprinting, build profiles, incremental compilation, target-directory reuse, feature resolution, build scripts, and toolchain identity. Those underlying mechanisms determine how much benefit these three tools can produce.

## 2. cargo-hakari — Workspace feature unification

Version: 0.9.39\
Project: `guppy-rs/guppy`\
Core engine: `hakari`, using `guppy` for Cargo dependency-graph analysis and build simulation.

Primary documentation:

`https://docs.rs/cargo-hakari/latest/cargo_hakari/`

`https://docs.rs/cargo-hakari/latest/cargo_hakari/about/`

### 2.1 The underlying problem: feature-configuration fragmentation

Cargo unifies compatible dependency features within a particular resolved build graph, but that unification does not necessarily carry across separate Cargo invocations.

Consider a workspace with three crates:

| Workspace crate | Features requested from dependency `baz` |
| --------------- | ---------------------------------------- |
| `core`          | `serde`, `std`                           |
| `cli`           | `serde`, `logging`                       |
| `server`        | `std`, `async`                           |

Building each package separately may cause Cargo to compile `baz` using three distinct feature configurations.

Building all three together may produce another configuration comprising the union of those features.

Cargo normally handles this correctly. The inefficiency arises because different builds require different artifacts. Moreover, feature changes in a dependency can invalidate the artifacts of downstream crates that use it.

The problem becomes especially pronounced with heavily reused dependencies such as `syn`, `serde`, and other packages exposing substantial feature graphs.

`cargo-hakari` standardizes feature selection across workspace members, making more build invocations converge on common dependency configurations.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust



### 2.2 How workspace-hack crates work

The tool generates and maintains a special workspace member, conventionally named `workspace-hack`.

Conceptual feature-unification relationship. Existing direct and transitive dependencies remain; the additional workspace-hack dependency ensures common feature activation.

The generated crate declares dependencies with the accumulated feature sets required across the workspace. Other workspace members then depend on that crate, ensuring Cargo includes those feature requests even when building an individual member.

This is an intentional dependency edge for Cargo resolution rather than a conventional code-level import.

The implementation uses `guppy` to simulate Cargo builds, identify dependencies built with different feature sets, and construct the unification manifest automatically. Unlike manually maintained workspace-hack crates, the generated declarations can be refreshed as the graph evolves.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust



### 2.3 Core commands and lifecycle

| Command                              | Function                                                            |
| ------------------------------------ | ------------------------------------------------------------------- |
| `cargo hakari init workspace-hack`   | Initialize the generated workspace member and configuration         |
| `cargo hakari generate`              | Recompute and write the unification manifest                        |
| `cargo hakari manage-deps`           | Add or reconcile workspace-hack dependencies in workspace members   |
| `cargo hakari generate --diff`       | Detect differences between committed and expected generated content |
| `cargo hakari manage-deps --dry-run` | Check whether workspace dependency edges are current                |
| `cargo hakari explain <crate>`       | Explain why a dependency appears in the generated manifest          |
| `cargo hakari verify`                | Check for dependencies still built with multiple feature sets       |
| `cargo hakari disable`               | Temporarily remove generated feature-unification declarations       |
| `cargo hakari remove-deps`           | Remove the managed workspace dependency edges                       |
| `cargo hakari publish -p <crate>`    | Publish while temporarily removing the workspace-hack dependency    |

The `generate --diff` and `manage-deps --dry-run` commands are especially useful because the generated manifests become reproducible artifacts that can be checked independently of normal compilation.

`explain` provides inspectability into graph-resolution decisions, while `verify` investigates whether the feature sets are actually unified. In the intended supported configuration, surviving duplicate feature sets reported by `verify` indicate a problem worth investigating.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust



### 2.4 Advanced configuration

Hakari uses `.config/hakari.toml`, with a fallback to the older `.guppy/hakari.toml` location.

Important controls include:

| Setting                     | Purpose and significance                                                                       |
| --------------------------- | ---------------------------------------------------------------------------------------------- |
| `hakari-package`            | Identifies the generated workspace package                                                     |
| `resolver`                  | Selects the Cargo resolver semantics to simulate                                               |
| `platforms`                 | Narrows feature analysis to specified target triples                                           |
| `unify-target-host`         | Controls unification between host and target dependency feature sets                           |
| `traversal-excludes`        | Excludes packages and their exclusively reachable transitive dependencies from graph traversal |
| `final-excludes`            | Omits selected packages from output while retaining their reachable transitive dependencies    |
| `output-single-feature`     | Includes dependencies that otherwise have only one feature configuration                       |
| `exact-versions`            | Emits exact resolved dependency versions rather than version ranges                            |
| `workspace-hack-line-style` | Selects how dependencies on the generated crate appear in manifests                            |
| `dep-format-version`        | Controls generated manifest syntax and normalization                                           |
| `registries`                | Supports alternative dependency registries                                                     |

Source:&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust



#### Cargo resolver compatibility

Hakari supports Cargo resolvers 1, 2, and 3.

Resolver 2 introduced more selective feature unification across development dependencies, build dependencies, and target-specific dependencies. Resolver 3 adds MSRV-aware dependency version resolution.

Hakari's support for resolver 3 arrived in version 0.9.34. Its maintainers explain that resolver 3 behaves like resolver 2 for Hakari's feature-analysis stage because Cargo has already resolved package versions when Hakari performs its computation.

The configured resolver should represent the semantics of the workspace being analyzed. The documentation's recommendation of resolver 2 should not be interpreted as an incompatibility with resolver 3.

Source:&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Docs.rs



#### Target-specific graph analysis

By default, Hakari produces a minimal unification set suitable across all possible platforms.

The `platforms` option allows explicitly targeting common development platforms, such as:

```
platforms = [
    "x86_64-unknown-linux-gnu",
    "aarch64-apple-darwin",
    "x86_64-pc-windows-msvc",
]
```

This can improve unification for the selected targets by taking advantage of platform-specific dependency and feature relationships that cannot be assumed universally.

It also creates a distinction between optimizing common development targets and preserving identical unification behavior across every supported target. Target coverage is therefore a consequential configuration decision, not merely a performance hint.

#### Host-target unification

One easily overlooked mechanism is `unify-target-host`.

Cargo can compile the same package separately for its host role (build scripts, procedural-macro dependencies) and target role (normal application dependencies).

Hakari exposes four strategies:

| Strategy                   | Behavior                                                         |
| -------------------------- | ---------------------------------------------------------------- |
| `none`                     | Preserve separate host and target feature sets                   |
| `unify-if-both`            | Unify when the package appears in both roles                     |
| `replicate-target-on-host` | Also reproduce target dependency requirements on the host side   |
| `auto`                     | Select a strategy according to the dependency graph; the default |

The `auto` strategy selects the more extensive replication behavior when the workspace contains procedural macros or crates used as build dependencies, according to the underlying Hakari documentation.

This is useful because host-side artifacts can themselves be frequently reused, but more aggressive unification can increase feature activation and cold-build work.

Importantly, unifying feature sets does not eliminate the architectural distinction between host and target artifacts. Differences in compilation target, outputs, or compiler parameters can still require separate compilation.

Source:

`https://docs.rs/hakari/latest/hakari/enum.UnifyTargetHost.html`

#### Excluding problematic dependencies

Feature unification is not universally semantics-preserving.

Some features are mutually exclusive, select incompatible implementations, change compile-time behavior, or introduce optional capabilities that are inappropriate for another workspace component.

Hakari addresses this with `traversal-excludes` and `final-excludes`. The distinction matters:

- A traversal exclusion removes a package from the graph walk, potentially excluding dependencies reachable only through it.
- A final exclusion removes the selected package from the resulting unification set but preserves its transitive dependencies where applicable.

Hakari also automatically excludes third-party packages that would create dependency cycles in the generated graph. The 0.9.39 release includes fixes to structural cycle avoidance.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Docs.rs



### 2.5 Publishing and external dependency behavior

Workspace-hack crates are useful internally but can create distribution problems when published crates or Git dependencies refer to the generated package.

Hakari supports two main approaches.

Temporary removal at publishing time: `cargo hakari publish -p ...` removes the workspace-hack dependency during publishing and restores it afterward. This avoids exporting the dependency, but introduces a publishing lifecycle requirement.

Registry stub with a root `[patch]`: A minimal workspace-hack package is published to a registry, while the workspace root redirects it to the generated local package. Consumers outside the workspace resolve the lightweight published stub instead of inheriting the full internal feature-unification graph.

The second mechanism separates internal build optimization from the dependency contract exposed to downstream consumers.

Sources:

`https://docs.rs/cargo-hakari/latest/cargo_hakari/publishing/`

`https://docs.rs/cargo-hakari/latest/cargo_hakari/patch_directive/`

### 2.6 Performance characteristics and limitations

Hakari's principal performance advantage is reducing build-configuration variation over repeated development commands, not necessarily making a single clean build faster.

The project's published observations include cumulative speedups around 1.7× and individual commands with substantially larger benefits in particular large-workspace scenarios. These are upstream observations, not general expectations for arbitrary Rust projects. The benefit depends on dependency overlap, feature diversity, build-command patterns, and the downstream rebuilds that would otherwise occur.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust



Key limitations are architectural.

Cold builds can become more expensive. Unifying broader feature sets may compile more code than a single workspace member would otherwise need.

Feature unification can conceal undeclared requirements. A crate may compile because the workspace-hack activates features it failed to declare itself. Builds without Hakari can reveal these omissions.

Compilation artifacts are still tied to more than dependency identity. Different package versions, Rust toolchains, target triples, compilation profiles, and other relevant compiler settings can continue to generate distinct artifacts.

The generated dependency declarations require maintenance. Dependency changes, feature changes, platform changes, or updates to `Cargo.lock` may require regeneration.

The tool does not provide a persistent compiler cache. Its output influences Cargo's dependency resolution and artifact reuse, while the actual artifact storage remains Cargo's and any external compiler cache's responsibility.

### 2.7 Relationship to guppy and the Hakari library

`cargo-hakari` is a CLI, but its functionality is backed by reusable Rust libraries.

| Component      | Responsibility                                                           |
| -------------- | ------------------------------------------------------------------------ |
| `guppy`        | Cargo dependency graph, feature graph, and build-set analysis            |
| `hakari`       | Feature-unification computation and managed manifest representation      |
| `cargo-hakari` | CLI workflow around generation, management, verification, and publishing |

The `hakari` crate exposes functionality such as `HakariBuilder`, `Hakari`, `HakariCargoToml`, and `UnifyTargetHost`.

That separation matters for tools that already manipulate Cargo dependency graphs programmatically: feature-unification analysis is available independently of invoking the CLI and parsing its output.

Primary library API:

`https://docs.rs/hakari/latest/hakari/`

`https://docs.rs/guppy/latest/guppy/`

### 2.8 Version-specific developments

Version 0.9.39 introduced or incorporated several consequential changes:

- Improved explanations from `cargo hakari explain`.
- Stronger protection against dependency cycles when generating the workspace-hack.
- Updated `guppy` integration to 0.19.0, incorporating Cargo simulation fixes.
- TOML 1.1-compliant strict configuration parsing.
- Updated built-in targets and an MSRV of Rust 1.91.

The strict parsing change means previously tolerated but invalid configuration syntax can fail after upgrading.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Docs.rs



## 3. sccache — Compiler artifact caching and distributed compilation

Version: 0.18.0\
Project: Mozilla `sccache`\
Primary role: Reuse equivalent compiler outputs across builds, directories, and potentially machines without re-executing the compiler.

Primary documentation:

`https://github.com/mozilla/sccache/blob/v0.18.0/README.md`

`https://github.com/mozilla/sccache/blob/v0.18.0/docs/Caching.md`

`https://github.com/mozilla/sccache/blob/v0.18.0/docs/Rust.md`

### 3.1 Conceptual architecture

`sccache` operates as a compiler wrapper. Cargo continues to perform dependency resolution, determine which compiler commands are necessary, schedule compilation, and manage its target directory.

When Cargo launches `rustc`, sccache intercepts the invocation and determines whether an equivalent result is already available.

The important distinction is between Cargo avoiding a compilation entirely and sccache satisfying a compilation Cargo decided to execute.

Cargo's fingerprints and target-directory artifacts may make a compiler invocation unnecessary. If Cargo does invoke the compiler, sccache can potentially replace that work with a cache lookup and artifact restoration.

This also explains why sccache remains useful after `cargo clean`, when switching to a separate target directory, or in ephemeral CI environments with compatible shared cache storage. Those situations may defeat Cargo's local artifact reuse without necessarily defeating sccache's reuse.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://github.com\&sz=32)

GitHub



### 3.2 Rust integration and supported compilation

The normal Cargo integration is:

```
# .cargo/config.toml
[build]
rustc-wrapper = "sccache"
```

Equivalent environment-based configuration:

```
RUSTC_WRAPPER=sccache cargo build
```

The actual wrapper executable must be discoverable through `PATH` or configured with an explicit path.

This arrangement requires no modifications to application Rust source, dependency declarations, or Cargo's build graph.

However, not every `rustc` invocation is cacheable.

The Rust-specific implementation requires Cargo-style compiler arguments, including `--crate-name`, `--out-dir`, and supported `--emit` values. It does not support compilation from stdin or an arbitrary `-o` output file. Its cacheability conditions are more restrictive than merely observing a successful `rustc` invocation.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://github.com\&sz=32)

GitHub



#### Cacheable versus non-cacheable crate outputs

| Compilation characteristic                                                                                | sccache behavior                                        |
| --------------------------------------------------------------------------------------------------------- | ------------------------------------------------------- |
| Eligible Rust library compilation                                                                         | Can cache compiler outputs                              |
| Eligible dependency `rlib` compilation                                                                    | Can be reused across compatible invocations             |
| Rust incremental compilation enabled                                                                      | Not cacheable                                           |
| Crates requiring a system-linker invocation, including `bin`, `dylib`, `cdylib`, and `proc-macro` outputs | Not cacheable through the normal Rust object-cache path |
| Unsupported compiler output options                                                                       | Invocation bypasses caching                             |
| Procedural-macro expansion involving untracked filesystem reads                                           | Potential cache-correctness limitation                  |

An important distinction is that a `proc-macro` crate's linked output is not normally cacheable, but other Rust libraries in the procedural macro's dependency graph may still be eligible.

For binaries with substantial compilation inside the executable crate, separating large reusable library components from a thin binary entry point can increase the work eligible for caching. That is an architectural possibility identified in sccache's documentation, not a universal optimization.

Source:

`https://github.com/mozilla/sccache/blob/v0.18.0/docs/Rust.md`

### 3.3 Compiler identity and cache-key construction

sccache's cache is based on equivalence of compiler inputs, not on a declaration that two crates have the same name.

For Rust, documented cache-key inputs include:

- Source-file content digests, using BLAKE3.
- The compiler executable's identity and path.
- Compiler host triple and sysroot.
- Relevant compiler/sysroot library content.
- Parsed compiler arguments.
- Relevant dependencies and other compilation inputs.

This has several consequences.

Toolchain changes can invalidate cached results. Two different `rustc` versions, sysroots, or compiler configurations should not be assumed interchangeable.

Build profiles matter. Debug information, optimization level, code generation settings, target-specific flags, and other effective compiler arguments can produce different cache entries.

Dependency graph changes can propagate. An altered dependency artifact can change the inputs or arguments of downstream compilation.

Cross-directory hits need additional attention. Absolute source paths or compiler arguments can distinguish otherwise similar checkouts.

sccache 0.18.0 improves `SCCACHE_BASEDIRS`, which normalizes specified root paths when constructing cache keys, including relevant paths embedded in compiler arguments. This can help reuse across multiple checkouts, Git worktrees, and related build locations. It is not a guarantee that arbitrary Rust outputs are relocatable across machines.

Primary source:

`https://github.com/mozilla/sccache/blob/v0.18.0/docs/Caching.md`&#x20;

[image](https://www.google.com/s2/favicons?domain=https://github.com\&sz=32)

GitHub



### 3.4 The incremental-compilation tradeoff

One of the most consequential sccache decisions is its interaction with Rust incremental compilation.

Cargo commonly enables incremental compilation in development profiles, particularly for workspace members and path dependencies. Incremental compilation stores intermediate compilation state so rustc can avoid redoing work within an evolving compilation unit.

sccache instead reuses complete eligible compiler outputs across equivalent invocations.

These approaches serve different repetition patterns and cannot simply be stacked for the same incrementally compiled invocation. sccache does not cache Rust incremental compilations.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://github.com\&sz=32)

GitHub



An illustrative cache-oriented configuration is:

```
[profile.dev]
incremental = false

[profile.test]
incremental = false
```

Or for an individual Cargo execution:

```
CARGO_INCREMENTAL=0 cargo build
```

The tradeoff can be expressed as follows:

| Workload                                                                     | Potential advantage                                                       |
| ---------------------------------------------------------------------------- | ------------------------------------------------------------------------- |
| Rapid edits to the same application crate                                    | Cargo/rustc incremental compilation                                       |
| Repeated clean builds of unchanged dependencies                              | sccache                                                                   |
| Multiple CI jobs compiling nearly identical graphs                           | Shared sccache storage                                                    |
| Frequently changing compiler flags or toolchains                             | Less reliable sccache reuse                                               |
| Workspace dependencies mostly unchanged, application code changes frequently | A mixed strategy may be appropriate across build profiles and invocations |

Disabling incremental compilation globally is therefore not intrinsically optimal. It can improve sccache effectiveness while worsening local rebuild latency for actively edited crates.

Cargo profile documentation:

`https://doc.rust-lang.org/cargo/reference/profiles.html#incremental`

### 3.5 Client-server architecture and the newer client-side mode

sccache uses a short-lived wrapper process for each compiler invocation and a persistent local service to coordinate storage and statistics.

In its default server-side mode, the wrapper sends the compilation request to the daemon, which computes the key, accesses storage, and executes the compiler on a miss.

Version 0.17 introduced a significant alternative: client-side mode, available in 0.18.0.

```
SCCACHE_CLIENT_SIDE=1 cargo build
```

Or through the configuration key `client_side_mode`.

In client-side mode, the wrapper process itself performs compilation analysis, cache-key calculation, and compiler execution. The daemon remains responsible for shared cache access and statistics aggregation.

Its IPC implementation uses `IpcStorage` and an initial `StorageHandshake`. Disk-backed entries can sometimes be obtained through a path-based operation, whereas remote backends generally transfer cached bytes.

This shifts substantial compiler-facing work out of the central daemon. The intended benefit is reduced coordination pressure and improved behavior under highly concurrent compilation workloads, although actual performance depends on the workload and storage backend.

Important 0.18.0 limitation: client-side mode is still opt-in. It is ignored when a distributed compilation scheduler is configured or when `SCCACHE_ERROR_LOG` is enabled. The upstream architecture document describes a future intention to retire server-side mode, but that is not the released default at this version.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://github.com\&sz=32)

GitHub



Primary documentation:

`https://github.com/mozilla/sccache/blob/v0.18.0/docs/Architecture.md`

### 3.6 Storage backends and multi-level caching

sccache's storage abstraction is substantially more capable than a local directory cache.

Its supported storage ecosystem includes local disk, S3-compatible object storage, Redis, Memcached, Google Cloud Storage, Azure Blob Storage, GitHub Actions cache storage, and other documented backends such as WebDAV and cloud-provider-specific object stores. Availability can depend on the executable's compiled features and backend-specific authentication.

Backend configuration covers storage endpoints, credentials, cache prefixes, capacity, expiration, and read/write restrictions.

#### Multi-level caching

sccache supports ordered cache tiers:

```
[cache.multilevel]
chain = ["disk", "redis", "s3"]
write_error_policy = "l0"

[cache.disk]
dir = "/var/cache/sccache"
size = 10737418240

[cache.redis]
endpoint = "redis://127.0.0.1:6379"

[cache.s3]
bucket = "shared-compiler-cache"
```

This is illustrative configuration; the Redis service, bucket, credentials, and any necessary backend features must exist separately.

The multi-level engine implements several useful operations:

1. Hierarchical reads. Lookup proceeds from the first tier toward progressively deeper tiers.
2. Promotion/backfill. A hit in a deeper tier can populate faster tiers for subsequent requests.
3. Writes across eligible levels. New artifacts can be written to multiple configured backends.
4. Read-only tiers. Individual levels can participate in lookups while being excluded from writes.
5. Configurable write-failure handling. The `ignore`, `l0`, and `all` policies determine which storage failures are considered significant.

These capabilities support different cache-sharing boundaries: local per-developer reuse, machine-local shared caches, low-latency team caches, and durable remote storage.

The `l0` write-error policy is the default, making the first cache level significant in determining reported write success. Cache availability and build correctness are separate concepts: storage failures should not be mistaken for compiler failures without inspecting how a specific failure was handled.

In version 0.18.0, multi-level fixes also address behavior involving read-only tiers.

Primary documentation:

`https://github.com/mozilla/sccache/blob/v0.18.0/docs/MultiLevel.md`

`https://github.com/mozilla/sccache/blob/v0.18.0/docs/Configuration.md`&#x20;

[image](https://www.google.com/s2/favicons?domain=https://github.com\&sz=32)

GitHub



#### Cache locality versus cache sharing

A multi-level cache introduces an important performance balance.

A fast local cache has low latency and little network overhead but limited cross-machine reuse. A remote cache can reuse artifacts across CI jobs and developers but adds network requests, transfers, authentication, and storage coordination.

A cache hit is only beneficial when the lookup and restoration cost is less than the compilation work it replaces. Large reusable dependencies are more attractive candidates than trivial compilations with expensive remote lookups.

Compression settings, including `SCCACHE_CACHE_ZSTD_LEVEL`, introduce another CPU-versus-transfer tradeoff.

### 3.7 Distributed compilation — different from remote caching

sccache also provides distributed compilation through a separate set of components.

The key distinction is:

- Remote caching: retrieve the outputs of a compilation performed previously.
- Distributed compilation: execute a cache-miss compilation on a remote build server.

Distributed mode adds a scheduler, build-server management, authentication, toolchain packaging, and sandboxed execution. It is therefore substantially more operationally complex than configuring S3 or Redis as a cache backend.

The documented implementation supports Linux build servers, including a separately documented FreeBSD variant. Linux clients receive the greatest documented coverage; macOS and Windows client support is less extensively tested.

The build server's Linux sandboxing uses Bubblewrap, with the upstream quickstart specifying version 0.3.0 or newer, together with its documented root-execution requirements for that server arrangement.

The mechanism also supports custom toolchain archives, toolchain cache limits, token/JWT-based authentication, and remote scheduler status via `sccache --dist-status`. The upstream guide strongly recommends HTTPS in front of the scheduler.

For remote environments, toolchain equivalence, architecture compatibility, sandbox security, and trust in compilation results become material parts of the system design.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://github.com\&sz=32)

GitHub



Primary guide:

`https://github.com/mozilla/sccache/blob/v0.18.0/docs/DistributedQuickstart.md`

### 3.8 Diagnostics and operational controls

| Capability         | Command or setting                          | Purpose                                                       |
| ------------------ | ------------------------------------------- | ------------------------------------------------------------- |
| Cache statistics   | `sccache --show-stats`                      | Inspect requests, hits, misses, and non-cacheable activity    |
| Reset statistics   | `sccache --zero-stats`                      | Establish a new measurement baseline                          |
| Server lifecycle   | `sccache --start-server`, `--stop-server`   | Control daemon lifetime                                       |
| Distributed status | `sccache --dist-status`                     | Inspect the remote compilation scheduler                      |
| Debug logging      | `SCCACHE_LOG`                               | Enable diagnostic logging                                     |
| Error log          | `SCCACHE_ERROR_LOG`                         | Collect error details; affects client-side mode compatibility |
| Server transport   | `SCCACHE_SERVER_PORT`, `SCCACHE_SERVER_UDS` | Configure local server communication                          |
| Recaching          | `SCCACHE_RECACHE`                           | Recompile and replace an existing cached result               |
| Compression        | `SCCACHE_CACHE_ZSTD_LEVEL`                  | Adjust compression cost                                       |
| Root normalization | `SCCACHE_BASEDIRS`                          | Improve cache reuse across compatible checkout locations      |

sccache integrates with compiler jobserver coordination rather than replacing Cargo's overall build scheduling.

The diagnostic distinction to preserve is cache misses versus non-cacheable compilations. A miss means an invocation was eligible for caching but the desired result was unavailable; a non-cacheable invocation cannot benefit from a cache hit under the current compiler mode and inputs.

For performance investigation, aggregate hit rate alone is insufficient. The useful quantities are eligible compilation time avoided, miss compilation cost, cache lookup and transfer cost, and time attributable to uncacheable compiler invocations.

### 3.9 Developments in 0.18.0

The 0.18.0 release includes changes relevant to caching correctness and architecture:

- Improved `SCCACHE_BASEDIRS` treatment of compiler-argument paths.
- Multi-level caching fixes, including read-only configurations.
- Microsoft Entra ID authentication support for Azure storage.
- Improvements around assembler and compiler argument handling.
- Expanded distributed compilation handling for certain Rust library artifacts.
- Minimum supported Rust version of 1.91 for building sccache.

Client-side mode originated in the preceding 0.17 release and remains opt-in in 0.18.0.

Release record:

`https://github.com/mozilla/sccache/releases`&#x20;

[image](https://www.google.com/s2/favicons?domain=https://github.com\&sz=32)

GitHub



## 4. cargo-shear — Dependency and manifest hygiene

Version: 1.14.0\
Project: `Boshen/cargo-shear`\
Primary role: Identify dependency declarations that are unused, incorrectly classified, or unnecessarily retained in Cargo manifests.

Primary documentation:

`https://github.com/Boshen/cargo-shear`

`https://docs.rs/crate/cargo-shear/latest/source/README.md`

`https://docs.rs/crate/cargo-shear/latest/source/CHANGELOG.md`

### 4.1 Conceptual model

`cargo-shear` approaches dependency management as a relationship between two structures:

1. Declared dependencies: The package and workspace dependency declarations in Cargo manifests.
2. Observed dependency usage: Crate references discovered through analysis of the Rust source tree.

Its central operation compares these structures to identify declarations that appear unnecessary.

Unlike compiler caching, this is fundamentally a static program-analysis and manifest-maintenance capability, not a build execution mechanism.

That difference has meaningful consequences. A conventional shear analysis can inspect a large Rust workspace without performing the full compiler workload that would be required for an equivalent build-based dependency analysis. It can also produce actionable manifest edits directly.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Docs.rs



### 4.2 Analysis architecture

cargo-shear combines several established Rust ecosystem components:

| Component                          | Role                                                          |
| ---------------------------------- | ------------------------------------------------------------- |
| `cargo_metadata`                   | Enumerate packages, dependencies, and Cargo targets           |
| `ra_ap_syntax`                     | Parse Rust source using rust-analyzer's syntax infrastructure |
| `toml_edit`                        | Perform manifest modifications                                |
| `globset` and filesystem discovery | Locate and filter source files                                |
| `miette`                           | Render diagnostics                                            |
| `serde_json`                       | Produce structured machine-readable results                   |

For each package, cargo-shear discovers library, binary, integration-test, example, and benchmark targets; analyzes reachable source files; extracts relevant dependency usage; and reconciles the results with manifest declarations.

This architecture is more structured than simply searching files for dependency names with regular expressions.

However, parsing Rust source is not identical to executing the compiler's name resolution and macro-expansion pipeline. That difference is central to understanding its limits.

Source:&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Docs.rs



### 4.3 Principal diagnostic capabilities

| Diagnostic family             | What it detects                                                           | Why it matters                                           |
| ----------------------------- | ------------------------------------------------------------------------- | -------------------------------------------------------- |
| Unused dependencies           | Dependency declarations without discovered usage                          | May eliminate unnecessary graph edges                    |
| Unused workspace dependencies | Entries under `[workspace.dependencies]` that no members require          | Prevents stale centralized declarations                  |
| Misplaced dependencies        | Certain dependencies declared in the wrong normal, dev, or build category | Improves manifest accuracy and build separation          |
| Unlinked Rust files           | Source files not reachable through any discovered module tree             | Identifies potentially orphaned source                   |
| Empty source files            | Empty files that may represent stale project structure                    | Supports structural cleanup                              |
| Unused optional dependencies  | Optional declarations lacking detected usage                              | Identifies obsolete or questionable feature declarations |
| Redundant exclusions          | Ignore entries that no longer suppress findings                           | Prevents suppression configuration from accumulating     |
| Test target mismatches        | Inconsistent `[lib]` test/doctest configuration                           | Aligns target settings with source content               |

The last group, test target mismatch detection, requires the opt-in `--check-test-targets` flag.

Misplaced-dependency detection has an important documented limit: it can identify cases involving integration tests, benchmarks, and examples, but does not fully classify dependencies used exclusively inside unit tests guarded by `#[cfg(test)]`.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Docs.rs



### 4.4 Commands and remediation

| Command                            | Capability                                    |
| ---------------------------------- | --------------------------------------------- |
| `cargo shear`                      | Analyze and report without editing            |
| `cargo shear --fix`                | Apply supported manifest corrections          |
| `cargo shear --deny-warnings`      | Treat warning diagnostics as failures         |
| `cargo shear --format=json`        | Produce structured findings                   |
| `cargo shear --expand`             | Include macro-expanded source analysis        |
| `cargo shear --check-test-targets` | Analyze test and doctest target configuration |

Its automated remediation is useful because findings can be converted into concrete changes rather than remaining advisory.

Nevertheless, removing a dependency declaration is a semantic code-maintenance action, not a purely mechanical storage optimization. Optional features, build-time generation, conditional compilation, and intentionally future-facing dependencies can justify declarations that static analysis otherwise considers unused.

#### Exit-code semantics

One operational detail is easy to overlook:

| Exit code | Normal mode                  | `--fix` mode                      |
| --------- | ---------------------------- | --------------------------------- |
| `0`       | No failing issues            | No issues and no changes required |
| `1`       | Findings requiring attention | Findings detected and fixed       |
| `2`       | Processing error             | Processing error                  |

An exit code of `1` after `cargo shear --fix` does not necessarily mean remediation failed. It can indicate that changes were successfully applied.

Similarly, warnings are permitted by default, whereas `--deny-warnings` elevates them to failing conditions.

These semantics matter for automation: a CI workflow should distinguish analysis failures, detected changes, and actual execution errors rather than interpreting every nonzero exit as an equivalent tool failure.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Docs.rs



### 4.5 Structured diagnostics and embedding

`cargo-shear` exposes machine-readable JSON findings.

Its output includes summary counts and individual diagnostics containing fields such as:

- `code`: Stable-style diagnostic identifier, such as `shear/unused_dependency`.
- `severity`: Error or warning classification.
- `message`: Diagnostic description.
- `file` and `location`: Source location, using byte offsets and lengths.
- `help`: Suggested correction.
- `fixable`: Whether supported automatic remediation is available.

This enables direct integration with tools that aggregate findings, construct editor diagnostics, produce pull requests, or apply targeted manifest corrections.

The package additionally publishes a Rust library interface centered on `CargoShear` and `CargoShearOptions`.

An illustrative integration is:

```
use cargo_shear::{CargoShear, CargoShearOptions};

let options = CargoShearOptions::new(".".into());
let exit_code = CargoShear::new(std::io::stdout(), options).run();
```

The public Rust interface allows other Rust tools to invoke the analysis without relying exclusively on command execution and text parsing. The documented module structure separates source parsing, package processing, manifest editing, and diagnostic rendering.

Primary API documentation:

`https://docs.rs/cargo-shear/latest/cargo_shear/index.html`&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust



### 4.6 Macro expansion: speed versus completeness

The default analysis uses parsed source rather than fully expanded compiler input.

Consequently, dependencies referenced exclusively through macro expansions may not be visible. Removing such a dependency can be incorrect even though the default analysis considers it unused.

cargo-shear provides:

```
cargo shear --expand
```

This delegates expansion to `cargo expand` and then analyzes the expanded source.

The documented tradeoffs are substantial:

- Requires the relevant nightly-based macro-expansion tooling.
- Is significantly slower than the normal analysis.
- Can invoke build operations.
- Reflects the expansion configuration and platform being analyzed.

The expansion path is therefore closer to compiler-assisted analysis, but it does not automatically prove correctness for every feature combination, target platform, or conditional compilation configuration.

This is a meaningful difference from the default parser-based mechanism, which can statically inspect platform-specific source branches without performing a separate build for each target.

Source:&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Docs.rs



### 4.7 Configuration and intentional exceptions

False positives can be suppressed at package scope:

```
[package.metadata.cargo-shear]
ignored = ["generated-runtime"]
ignored-paths = ["src/generated/*.rs"]
```

Or at workspace scope:

```
[workspace.metadata.cargo-shear]
ignored = ["special-build-dependency"]
ignored-paths = ["*/proto/*.rs"]
```

The semantics differ:

Package-level configuration only applies to the containing package.

Workspace-level configuration applies across workspace members and also affects workspace dependency analysis. Workspace path globs are interpreted relative to the workspace root.

A notable maintenance feature is detection of redundant ignore declarations. If a suppression is no longer needed, the tool can report it.

This prevents the exception list itself from becoming a collection of stale, unexamined assumptions.

For workspace-wide ignores, redundancy is evaluated across members rather than treating an ignore as unnecessary merely because one member does not need it.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Docs.rs



### 4.8 New in 1.14.0: preserving predeclared path dependencies

A workspace may deliberately declare a local dependency under `[workspace.dependencies]` before any member inherits that declaration.

Ordinary unused-dependency analysis would classify the declaration as unnecessary.

Version 1.14.0 adds:

```
[workspace.metadata.cargo-shear]
keep-path-dependencies = true
```

When enabled, a workspace dependency with a `path` value is retained if an actual crate exists at that path, even when no current workspace member references it.

If the path becomes stale because the crate is deleted or moved, the declaration can still be reported as unused.

This is a useful distinction between:

- Unused because no consumer currently exists, and
- Invalid because the declared local package no longer exists.

The setting makes it possible to preserve intentional workspace-level dependency declarations without individually suppressing each one.

Source:&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Docs.rs



### 4.9 Native compatibility with cargo-hakari

This is the most directly consequential interaction between two of the requested tools.

A workspace-hack crate intentionally declares dependencies without importing them in Rust source. Its workspace consumers likewise depend on it without necessarily referencing it in code.

To a generic unused-dependency checker, both sides appear suspicious.

cargo-shear explicitly recognizes cargo-hakari's generated workspace-hack structure.

It detects the marker:

```
### BEGIN HAKARI SECTION
```

inside the generated manifest, then excludes the generated crate and dependency edges pointing to it from the ordinary unused-dependency analysis.

This means the two tools can coexist without requiring a large collection of custom suppressions.

The integration is particularly valuable because it preserves two otherwise conflicting invariants:

- cargo-hakari intentionally creates dependencies solely to influence Cargo feature resolution.
- cargo-shear removes or reports dependencies that have no legitimate purpose.

The key is that cargo-shear recognizes the generated dependency graph as legitimate metadata rather than treating every dependency as requiring an explicit source import.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Docs.rs



### 4.10 Limitations and consequential distinctions

#### Manifest reduction is not equivalent to compiled-graph reduction

Removing an unused direct dependency does not guarantee that the crate disappears from the resolved dependency graph.

Another dependency may still require it transitively. Cargo can also continue to resolve a package through another workspace member.

Likewise, a dependency present in a manifest may not contribute to a particular build if it is optional and disabled, or if it only applies to another target or profile.

Therefore, the performance effect depends on whether removal changes the effective resolved build graph, feature activation, build-script execution, and downstream compilation.

#### Static detection is not proof of non-use

Dependencies may have less obvious roles:

- Proc-macro-generated code can reference crates not visibly imported before expansion.
- Feature-gated source can introduce dependencies only for particular configurations.
- Build scripts and generated source can create relationships that ordinary source parsing does not fully reveal.
- Dependency declarations can activate Cargo features even when their crates are not directly referenced in source.

The last point is especially relevant when working with dependency-feature management tools.

These are reasons to retain explicit exceptions where justified and validate meaningful manifest changes with appropriate Cargo builds and tests.

#### Unlinked files are not necessarily obsolete files

An unlinked source file may represent abandoned code, but it could also be intentionally included by another generation or tooling mechanism that the static analysis does not understand.

The finding is therefore evidence of disconnected source structure, not proof that deletion is safe.

#### cargo-shear is not a dependency policy or security analyzer

It does not replace tools that inspect vulnerabilities, licenses, duplicate dependency versions, or dependency supply-chain policies.

Its main contribution is the relationship between manifest declarations and discoverable source usage.

### 4.11 Project maturity

The maintainers describe cargo-shear as feature-complete and primarily focused on bug fixes, dependency upgrades, and user-experience improvements.

This is a project-maintenance designation, not evidence that the tool is deprecated or that further releases cannot occur.

The 1.14.0 release remains an active example of targeted capability additions, particularly its new path-dependency retention setting.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Docs.rs



## 5. How the three tools compose

The three tools act on different stages of the same general Rust build lifecycle.

### 5.1 Shared effects on compilation

The interactions are principally indirect.

| Combination                | Potential synergy                                                                      | Important qualification                                            |
| -------------------------- | -------------------------------------------------------------------------------------- | ------------------------------------------------------------------ |
| cargo-shear + cargo-hakari | Smaller, cleaner declarations can simplify the graph Hakari analyzes                   | Regenerate Hakari output after relevant graph changes              |
| cargo-hakari + sccache     | Stable unified feature configurations may increase reuse of identical compiler outputs | Wider features can increase cold-build cost and change cache keys  |
| cargo-shear + sccache      | Removing genuinely unnecessary dependencies can eliminate compiler work altogether     | No benefit where an unused declaration did not affect a build      |
| All three                  | Reduce unnecessary graph inputs, feature variance, and repeated eligible compilation   | Improvements are workload-dependent, not multiplicative guarantees |

A key architectural observation is that preventing compilation entirely is different from making compilation cheaper.

For example, if a dependency declaration can be removed such that Cargo no longer needs the package, neither Hakari nor sccache needs to optimize its compilation.

Conversely, a necessary dependency with many feature configurations remains an important Hakari candidate, and a consistently configured but repeatedly rebuilt dependency remains an important sccache candidate.

### 5.2 Dependency cleanup and generated-manifest consistency

A typical relationship between these tools is:

```
# Analyze manifest hygiene
cargo shear

# Regenerate feature-unification metadata after graph changes
cargo hakari generate
cargo hakari manage-deps

# Check generated metadata consistency
cargo hakari generate --diff
cargo hakari manage-deps --dry-run
cargo hakari verify
```

These commands illustrate their respective responsibilities, rather than prescribing a single CI or developer workflow.

The need for regeneration arises from a straightforward dependency: Hakari's generated graph is derived from Cargo's manifests and resolved dependencies.

Meaningful manifest edits can invalidate the assumptions represented by the existing workspace-hack crate. Regeneration restores consistency.

By contrast, sccache does not need manifest-specific coordination. It observes compiler commands and inputs that Cargo ultimately produces.

### 5.3 Feature unification versus feature isolation

One important design tension is that cargo-shear favors removal of unnecessary dependency edges, whereas cargo-hakari deliberately introduces dependencies to activate feature unions.

Neither philosophy is universally preferable.

Cargo feature unification is generally additive. That means activating extra features should be compatible with the dependency author's intended feature model, but not every third-party crate follows an architecture in which all features can safely coexist.

For crates with mutually exclusive features, dramatically different feature-induced compilation costs, or configuration-specific semantics, broader unification can be counterproductive.

Hakari's exclusions and target restrictions are therefore not merely escape hatches. They provide a way to constrain optimization to the parts of a graph where it is semantically appropriate.

### 5.4 Cargo's own build system remains essential

These tools complement rather than replace several native mechanisms.

| Native mechanism         | Distinction from the three tools                                                                 |
| ------------------------ | ------------------------------------------------------------------------------------------------ |
| Cargo fingerprints       | Determine whether locally tracked compilation units are dirty                                    |
| `target/` artifact reuse | Reuses outputs already present under Cargo-managed build directories                             |
| Cargo feature resolver   | Determines legal feature unification and dependency resolution for each build                    |
| Build profiles           | Control optimization, debug info, incremental compilation, and other artifact-affecting settings |
| `Cargo.lock`             | Fixes resolved package versions for reproducible dependency selection                            |
| `cargo metadata`         | Provides structured package/graph information without performing compilation                     |
| `CARGO_TARGET_DIR`       | Controls output-directory identity and artifact sharing                                          |
| Cargo build scripts      | Introduce generated inputs and rerun conditions that can influence rebuilds                      |

Two adjacent tools deserve mention solely to clarify boundaries:

`cargo-chef` is primarily designed to structure dependency compilation into reusable build layers, particularly container builds. It is not a direct alternative to sccache's compiler output cache for ordinary local compilation.

`cargo-deny` addresses dependency security, license, and dependency-policy checks. It complements cargo-shear's source-usage analysis rather than replacing it.

Cargo's foundational documentation:

`https://doc.rust-lang.org/cargo/reference/features.html`

`https://doc.rust-lang.org/cargo/reference/profiles.html`

`https://doc.rust-lang.org/cargo/reference/config.html`

### 5.5 Build-performance interpretation

These tools are best evaluated through distinct outcome measures rather than one combined notion of build speed.

| Tool         | Most revealing measurements                                                                                      |
| ------------ | ---------------------------------------------------------------------------------------------------------------- |
| cargo-hakari | Distinct feature configurations per dependency, repeated dependency compilation, affected downstream rebuilds    |
| sccache      | Cache hit/miss rates, compilation time avoided, storage and transfer overhead, non-cacheable invocations         |
| cargo-shear  | Actual resolved dependency graph reductions, removed build-script activity, changes in required compilation work |

A small number of expensive dependencies built repeatedly with different feature sets can make Hakari valuable even when the total number of dependencies remains unchanged.

Likewise, sccache can substantially benefit expensive repeat builds without changing any dependency graph.

And cargo-shear can improve manifest correctness and maintainability even when its impact on wall-clock compilation time is negligible.

Those distinctions help avoid attributing benefits to the wrong mechanism.

## 6. Primary-source reference map

The following are direct URLs suitable for reuse in technical documentation or agent research instructions.

### cargo-hakari and its underlying libraries

| Resource                                 | URL                                                                 |
| ---------------------------------------- | ------------------------------------------------------------------- |
| cargo-hakari overview                    | `https://docs.rs/cargo-hakari/latest/cargo_hakari/`                 |
| Conceptual background                    | `https://docs.rs/cargo-hakari/latest/cargo_hakari/about/`           |
| Publishing behavior                      | `https://docs.rs/cargo-hakari/latest/cargo_hakari/publishing/`      |
| Published stub and Cargo patch mechanism | `https://docs.rs/cargo-hakari/latest/cargo_hakari/patch_directive/` |
| Hakari library API                       | `https://docs.rs/hakari/latest/hakari/`                             |
| Host/target unification strategies       | `https://docs.rs/hakari/latest/hakari/enum.UnifyTargetHost.html`    |
| guppy dependency graph engine            | `https://docs.rs/guppy/latest/guppy/`                               |
| cargo-hakari changelog                   | `https://docs.rs/crate/cargo-hakari/latest/source/CHANGELOG.md`     |

### sccache

| Resource                             | URL                                                                             |
| ------------------------------------ | ------------------------------------------------------------------------------- |
| Main documentation and installation  | `https://github.com/mozilla/sccache/blob/v0.18.0/README.md`                     |
| Rust-specific cacheability rules     | `https://github.com/mozilla/sccache/blob/v0.18.0/docs/Rust.md`                  |
| Cache hashing and identity           | `https://github.com/mozilla/sccache/blob/v0.18.0/docs/Caching.md`               |
| Client/server execution architecture | `https://github.com/mozilla/sccache/blob/v0.18.0/docs/Architecture.md`          |
| Configuration and storage backends   | `https://github.com/mozilla/sccache/blob/v0.18.0/docs/Configuration.md`         |
| Multi-level cache hierarchy          | `https://github.com/mozilla/sccache/blob/v0.18.0/docs/MultiLevel.md`            |
| Distributed compilation              | `https://github.com/mozilla/sccache/blob/v0.18.0/docs/DistributedQuickstart.md` |
| Releases                             | `https://github.com/mozilla/sccache/releases`                                   |

### cargo-shear

| Resource                              | URL                                                            |
| ------------------------------------- | -------------------------------------------------------------- |
| Usage, configuration, limitations, CI | `https://docs.rs/crate/cargo-shear/latest/source/README.md`    |
| Public Rust API                       | `https://docs.rs/cargo-shear/latest/cargo_shear/index.html`    |
| Version history                       | `https://docs.rs/crate/cargo-shear/latest/source/CHANGELOG.md` |
| Upstream repository                   | `https://github.com/Boshen/cargo-shear`                        |

## 7. Overall architectural significance

The most valuable way to understand this toolset is as three independent forms of build-system optimization:

cargo-shear reduces unnecessary dependency declarations. Its lasting value includes dependency hygiene, clearer manifests, and potential removal of unnecessary compilation work.

cargo-hakari reduces dependency feature-configuration fragmentation. Its principal advantage arises across repeated workspace builds where different commands would otherwise produce different feature-specific dependency artifacts.

sccache reduces the cost of repeated eligible compilation. Its architecture extends from local caching to shared multi-level stores and, with additional infrastructure, distributed compilation.

Together, these capabilities cover a substantial portion of avoidable Rust build work without requiring a bespoke build engine. Their effectiveness still depends on Cargo's underlying semantics, and the greatest benefits arise when the actual source of repeated work matches the mechanism each tool is designed to eliminate.