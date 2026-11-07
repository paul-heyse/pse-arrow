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

# Addendum: Advanced Rust Build Optimization Capabilities

Supplement to Rust Build and Dependency Optimization: cargo-hakari, sccache, and cargo-shear

Research date: November 6, 2026

Verified versions: cargo-hakari 0.9.39, hakari 0.18.0, sccache 0.19.0, cargo-shear 1.14.0.

This addendum extends the original reference with additional capabilities, operational mechanisms, and consequential limitations. It concentrates on functionality not previously covered or requiring more detailed explanation, rather than restating the established conceptual models, command lifecycles, or integration patterns.

The principal version change is sccache 0.19.0, released October 24, 2026, which supersedes the original guide's 0.18.0 reference. The release introduces improvements to Rust artifact correctness and build pipelining, multi-level cache resilience, native compiler caching, diagnostics, and distributed compilation.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://github.com\&sz=32)

GitHub+1



## 1. sccache 0.19.0 — Rust compilation correctness and pipelining

### 1.1 Checkout-local dependency information

Rust compiler invocations can produce `.d` dependency-information files, which record source dependencies for build invalidation. These files may contain absolute paths associated with a particular checkout, build directory, or machine.

Earlier sccache versions could restore dependency information originally generated in another environment. Although the compiled artifact might remain reusable, the restored dependency file could reference paths that do not exist in the current environment.

Version 0.19.0 changes this behavior: Rust dependency information is written locally rather than restored from the compiler cache.

This separates two aspects of compilation output that have different portability requirements:

| Output                                      | Relevant identity                                                            |
| ------------------------------------------- | ---------------------------------------------------------------------------- |
| Compiled library or other eligible artifact | Equivalent compiler inputs, dependencies, and output-affecting configuration |
| Dependency-information file                 | Dependencies as represented in the current compilation environment           |

The consequence is that cross-checkout and cross-machine artifact reuse no longer depends on restoring potentially stale dependency-file paths.

This improvement complements `SCCACHE_BASEDIRS`, but addresses a different problem. Base-directory normalization improves cache-key equivalence across directories; locally generated dependency information preserves the current environment's dependency tracking after a cache hit.

It does not make every Rust artifact relocatable. Compiler arguments, embedded paths, build-script results, environment variables, and other compilation inputs can still prevent equivalent caching across different locations.

Sources: sccache 0.19.0 release notes; Cargo build-cache documentation.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://github.com\&sz=32)

GitHub+1



`https://github.com/mozilla/sccache/releases/tag/v0.19.0`

`https://doc.rust-lang.org/cargo/reference/build-cache.html`

### 1.2 Streaming Rust metadata notifications to preserve build pipelining

Rust compilation is not necessarily a completely sequential process at the crate level. Cargo can begin certain dependent compilations once their upstream dependencies have produced sufficient metadata, without requiring the upstream compilation unit to finish all code generation.

This is commonly referred to as build pipelining.

The relevant distinction is between a crate's metadata artifact, often `.rmeta`, and its eventual compiled code artifact.

sccache 0.19.0 introduces the ability to stream `.rmeta` artifact notifications to Cargo before the compilation finishes, including during cache misses and client-side execution.

The mechanism affects compilation scheduling rather than the semantic identity of cached outputs.

Conceptual dependency pipeline

Conceptual illustration, not a measured execution timeline. Downstream compilation still depends on Cargo's dependency constraints and availability of required metadata.

There are several consequential interactions.

Cache misses can still benefit. Even when compilation must actually run, forwarding metadata notifications prevents the wrapper from unnecessarily delaying a downstream compilation until the complete upstream result is available.

Client-side execution retains pipelining. When `SCCACHE_CLIENT_SIDE=1`, the compiler-facing work runs in the wrapper process, with the daemon primarily providing storage coordination. Version 0.19.0 preserves the metadata-notification behavior in this mode as well.

Cache hit rate is not a complete performance measure. Two build executions with identical cache hit rates can have different wall-clock durations if dependent compilation starts at different points.

This capability does not guarantee that all dependent crates can begin immediately after metadata emission. Macro expansion, build scripts, compiler dependencies, and other Cargo scheduling requirements can still impose boundaries.

Sources: sccache 0.19.0 release notes; Cargo build timings documentation.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://github.com\&sz=32)

GitHub+1



`https://github.com/mozilla/sccache/releases/tag/v0.19.0`

`https://doc.rust-lang.org/cargo/reference/timings.html`

### 1.3 Concurrent compiler-output discovery and hashing

sccache 0.19.0 also changes how Rust compilation requests are prepared.

Previously, identifying the outputs expected from a rustc invocation could contribute serial overhead before the remaining cache-key work proceeded. The 0.19.0 implementation computes Rust output information concurrently with other hashing operations.

This introduces another performance dimension distinct from compilation and artifact storage: request preparation latency.

The total duration of an eligible compilation can be decomposed conceptually as:

\\[ T\_{\mathrm{request}} = T\_{\mathrm{preparation}} + T\_{\mathrm{lookup}} + T\_{\mathrm{restore\ or\ compile}} + T\_{\mathrm{finalization}} \\]

Actual phases can overlap, so this is a conceptual decomposition rather than an additive timing guarantee.

The 0.19.0 change targets overlap within preparation work. Its advantage is most relevant when output determination and input hashing are individually expensive enough that performing them concurrently shortens the request's critical path.

It should not be confused with distributed compilation, parallel rustc code generation, or Cargo's parallel scheduling of independent compilation units.

Source: sccache 0.19.0 release notes, PR #2465.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://github.com\&sz=32)

GitHub



`https://github.com/mozilla/sccache/releases/tag/v0.19.0`

## 2. sccache — Native compiler capabilities and mixed-language builds

### 2.1 Compiler support beyond rustc

sccache's compiler abstraction supports several compiler families, including GCC, Clang, MSVC, Rust, CUDA through NVIDIA's `nvcc`, AMD ROCm HIP, and supported assembler workflows.

This means a Rust project containing native components can potentially use the same caching infrastructure for both Rust and non-Rust compilation.

However, integration is compiler-specific. Cargo's `RUSTC_WRAPPER` only governs the Rust compiler invocations that Cargo wraps. Native compilation initiated through `build.rs`, CMake, or other build systems must independently invoke sccache.

For CMake-based compilation, for example, the supported compiler-launcher interface is:

```
set(CMAKE_C_COMPILER_LAUNCHER sccache)
set(CMAKE_CXX_COMPILER_LAUNCHER sccache)
```

These settings are illustrative; they do not replace Cargo's Rust wrapper configuration or automatically configure every native compiler launched by a Rust build script.

The distinction has architectural consequences for mixed Rust/C++ or Rust/CUDA systems. Compiler artifacts may share storage infrastructure while retaining independent compiler identities, preprocessing behavior, and cacheability conditions.

Cache-key compatibility should not be assumed between different compiler families or compilation pathways merely because they write to the same backend.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Docs.rs+1



Primary source:

`https://github.com/mozilla/sccache/blob/v0.19.0/README.md`

### 2.2 C/C++ preprocessor caching

sccache provides a preprocessor-cache mechanism inspired by ccache's direct mode.

For eligible GCC/Clang C/C++ compilations, the mechanism can determine cache validity without repeating the entire preprocessing stage. This is significant because preprocessing may involve opening and processing large header graphs before compilation can begin.

The cache tracks included files and relevant external inputs. When its validity conditions are satisfied, a cached result can be retrieved without invoking the compiler's preprocessor again.

This is a different mechanism from Rust incremental compilation and has substantially different correctness requirements.

Availability constraints. The upstream documentation specifies that this optimization requires GCC or Clang, C/C++ compilation, and local storage. It is disabled for certain flags, including `-MP`, `-Xpreprocessor`, and `-Wp,`, as well as for recently modified headers that could introduce race conditions.

The documented controls provide several performance/correctness tradeoffs:

| Setting                       | Behavior and implication                                                                     |
| ----------------------------- | -------------------------------------------------------------------------------------------- |
| `use_preprocessor_cache_mode` | Enables the mechanism; documented default is `true`                                          |
| `SCCACHE_DIRECT`              | Overrides preprocessor-cache mode for an invocation                                          |
| `file_stat_matches`           | Permits file metadata checks to substitute for content hashing under its validation rules    |
| `use_ctime_for_stat`          | Includes filesystem change time in metadata validation                                       |
| `ignore_time_macros`          | Permits caching despite date/time macros, at the risk of stale results                       |
| `skip_system_headers`         | Omits system-header contents from validation; requires confidence in system-header stability |
| `hash_working_directory`      | Distinguishes compilation requests according to their working directory                      |

The default behavior favors correctness over maximum reuse: file contents are hashed unless metadata-based validation is explicitly enabled, and time-sensitive macros are treated conservatively.

Two notable correctness limitations remain.

First, if a header would have been included had it existed, a previous compilation may not record that nonexistent file. Creating it later can alter preprocessing behavior without invalidating the stored entry.

Second, time-sensitive macros and other external conditions may change preprocessing results independently of ordinary source-file changes.

The library documentation explicitly acknowledges that these situations can produce stale cached outputs. Preprocessor caching is therefore a conditional optimization, not a proof that a source file's preprocessing result is invariant.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://github.com\&sz=32)

GitHub



Primary source:

`https://github.com/mozilla/sccache/blob/v0.19.0/docs/Local.md`

### 2.3 Architecture-aware preprocessing in 0.19.0

The `SCCACHE_CACHE_MULTIARCH` option supports caching certain C/C++ compilations that pass multiple `-arch` values, particularly relevant to universal binaries on Apple platforms.

Such compilations may define different preprocessor macros for different target architectures. A source file guarded by architecture-sensitive conditions can produce different preprocessed code on each target.

Version 0.19.0 corrects a cache-key problem by preprocessing once per architecture rather than relying on an insufficient single-pass representation.

The result is that architecture-dependent conditions contribute to cache identity. Older cache entries produced using the previous multi-architecture method are not reused.

Important limitations: This mode remains opt-in, and the 0.19.0 implementation does not distribute these multi-architecture compilations. It should not be conflated with arbitrary cross-compilation support or with Rust target-triple handling.

The same release fixes an include-validation issue in the C/C++ preprocessor cache. Both changes concern cache correctness rather than the introduction of a general compiler optimization pass.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://github.com\&sz=32)

GitHub



`https://github.com/mozilla/sccache/releases/tag/v0.19.0`

### 2.4 MSVC and C++20 module support

Version 0.19.0 introduces caching for supported C++20 named-module compilations performed through `clang-cl`.

The release also improves MSVC-compatible argument and environment handling: it recognizes `CL` and `_CL_` environment-variable inputs, handles `/Tc` and `/Tp` input-file arguments, and fixes a program-database handling failure involving `-Fd` paths that name directories.

These changes expand the set of compiler invocations that sccache can process correctly.

Named modules are consequential because their generated artifacts can participate in dependency relationships distinct from conventional object files and textual headers. However, this release-specific functionality should not be interpreted as universal caching support for all C++20 module systems, module dependency scanners, or compiler implementations.

Source: sccache 0.19.0 release notes.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://github.com\&sz=32)

GitHub



`https://github.com/mozilla/sccache/releases/tag/v0.19.0`

## 3. sccache — Cache resilience, observability, and configuration

### 3.1 Independent cache-failure domains

Compiler caching introduces multiple operational failure domains. Treating them all as simple cache misses conceals important differences.

| Failure or condition                          | Relevant capability                | Consequence                                                                           |
| --------------------------------------------- | ---------------------------------- | ------------------------------------------------------------------------------------- |
| Local wrapper-to-server communication failure | `SCCACHE_IGNORE_SERVER_IO_ERROR=1` | Allows fallback to ordinary local compilation for supported communication-error paths |
| Cache storage read failure                    | Improved 0.19.0 error reporting    | Network and authorization errors are no longer silently represented as cache misses   |
| Cache-level write failure                     | Multi-level `write_error_policy`   | Determines which failures are tolerated                                               |
| Cache-level startup failure                   | 0.19.0 startup resilience          | Optional unavailable levels need not prevent cache initialization                     |
| Deliberate cache recaching                    | `SCCACHE_RECACHE`                  | Forces recompilation and replacement of an existing cached result                     |

The server-I/O fallback is especially important. By default, sccache can fail a build if it cannot communicate successfully with its associated server. With `SCCACHE_IGNORE_SERVER_IO_ERROR=1`, supported communication failures cause it to invoke the compiler locally instead.

This does not mean the setting suppresses every storage, compiler, or distributed-execution error. Its documented purpose is wrapper-to-server communication recovery.

Likewise, 0.19.0's improved reporting of remote storage read errors is an observability improvement. It does not imply that every remote cache failure must terminate compilation; the actual outcome depends on the surrounding caching and fallback path.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://github.com\&sz=32)

GitHub+1



Source:

`https://github.com/mozilla/sccache/blob/v0.19.0/README.md`

### 3.2 Multi-level cache startup and durability policies

The original guide describes hierarchical reads, writes, promotion, and the available write-error policies. Version 0.19.0 adds an important operational distinction: cache-level availability during startup now respects the durability intent of the configured policy.

| Policy   | Startup requirement                                                       |
| -------- | ------------------------------------------------------------------------- |
| `ignore` | Startup can continue despite unavailable levels                           |
| `l0`     | L0 must pass its startup check; failures in later levels can be tolerated |
| `all`    | All configured levels must pass their startup checks                      |

A temporarily unavailable level whose startup failure is tolerated is not permanently disabled as writable. It can become useful again if the remote service recovers.

This allows the cache hierarchy to distinguish hard requirements from best-effort storage rather than requiring every configured level to be available at daemon startup.

There is a related read-path improvement. In client-side execution, 0.19.0 corrects backfilling for raw cache hits obtained from deeper tiers and adds per-level statistics for those reads.

Together, these changes improve the behavior of cache hierarchies during partial outages and make their effectiveness more observable.

They do not guarantee durability or eventual replication when a remote storage service remains unavailable.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://github.com\&sz=32)

GitHub+1



Primary source:

`https://github.com/mozilla/sccache/blob/v0.19.0/docs/MultiLevel.md`

### 3.3 Environment-based multi-level configuration

Multi-level storage can be configured without relying exclusively on a TOML file.

The principal environment controls are:

```
SCCACHE_MULTILEVEL_CHAIN="disk,redis,s3"
SCCACHE_MULTILEVEL_WRITE_ERROR_POLICY="l0"
```

The configured chain specifies cache levels in fast-to-slow order, and each backend must also have its own connection and storage configuration.

This is useful in environments where configuration is provided through CI variables, container environments, or deployment systems rather than a persistent local configuration file.

An important operational detail is that environment variables override file-based settings, and some changes require restarting the sccache server to take effect.

`SCCACHE_CONF` permits an explicit configuration-file path, while `SCCACHE_SKIP_CACHE_CHECK` suppresses the normal remote cache capability probe. The latter uses the configured access mode without testing it, shifting responsibility for correct permissions and availability to the operator.

A skipped capability check is not proof that the backend is accessible.

Sources: tagged configuration and multi-level documentation.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://github.com\&sz=32)

GitHub+1



`https://github.com/mozilla/sccache/blob/v0.19.0/docs/Configuration.md`

### 3.4 Backend-specific capabilities

Some storage backends expose additional controls beyond the common cache interface.

| Backend               | Additional functionality                                                                                   |
| --------------------- | ---------------------------------------------------------------------------------------------------------- |
| Local disk            | `SCCACHE_LOCAL_RW_MODE` supports read-only operation over previously populated entries                     |
| S3-compatible storage | Read-only access, key prefixes, custom endpoints, and server-side encryption                               |
| Redis                 | Single-node or clustered endpoints, authentication, database selection, entry expiration, and key prefixes |
| Memcached             | Authentication, expiration controls, key prefixes, and read-only operation                                 |
| Azure Blob Storage    | Shared-key connection strings or Microsoft Entra ID-based authentication                                   |
| WebDAV                | Basic or bearer-token authentication and controls for unsupported directory-management requests            |

The S3 configuration supports SSE-S3, AWS-managed KMS keys, and explicitly identified customer-managed KMS keys. The customer-managed key setting takes precedence when provided. This allows the cache's storage-encryption arrangement to align with the storage service's supported security model.

For WebDAV, version 0.19.0 adds `SCCACHE_WEBDAV_DISABLE_CREATE_DIR`, corresponding to the `disable_create_dir` configuration field. It bypasses directory-existence and directory-creation operations that some WebDAV services do not support.

Local disk storage has a separate concurrency limitation: the documented implementation supports a single sccache server accessing a given local cache. Concurrent servers using the same disk cache can race and produce failures. This is different from concurrent compiler requests coordinated through one server.

Read-only mode also has a practical boundary: an empty read-only cache cannot satisfy requests and merely adds lookup overhead.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://github.com\&sz=32)

GitHub+3



Primary references:

`https://github.com/mozilla/sccache/blob/v0.19.0/docs/Configuration.md`

`https://github.com/mozilla/sccache/blob/v0.19.0/docs/Local.md`

### 3.5 Advanced diagnostics and statistical visibility

Beyond the basic statistics described in the original guide, sccache provides interfaces for structured and more detailed diagnostics:

```
# Detailed cache statistics
sccache --show-adv-stats

# Machine-readable statistics
sccache --show-stats --stats-format=json

# Inspect preprocessor-cache entries
sccache --debug-preprocessor-cache
```

The advanced output helps distinguish cache-request counts, caching failures, non-cacheable compilations, compiler execution costs, and cache access costs.

Version 0.19.0 adds two useful distinctions: statistics expose the effective cache access mode, and Rust compilations that cannot be cached are categorized by crate type.

It also records per-level statistics for client-side raw cache reads and corrects `--debug-preprocessor-cache` to use the configured cache directory.

The additional visibility can help distinguish four fundamentally different phenomena:

1. A request is eligible but its artifact is not cached.
2. A request is inherently ineligible under the compiler's current mode.
3. A reusable artifact exists but storage access is unsuccessful or expensive.
4. Cargo performs little or no work because its own dependency fingerprints already consider the relevant compilation up to date.

The fourth case is especially important: a low number of sccache requests can indicate that Cargo's local build cache is effective, rather than that the external compiler cache is performing poorly.

Measurement boundary: sccache statistics describe wrapper-observed activity, not the complete elapsed-time critical path of a Cargo build. They are complementary to Cargo's compiler-unit timing report.

Sources: upstream CLI interface and 0.19.0 release notes.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://github.com\&sz=32)

GitHub+1



`https://github.com/mozilla/sccache/releases/tag/v0.19.0`

### 3.6 Distributed compilation reliability improvements

Version 0.19.0 contains several improvements to the existing distributed compilation machinery.

| Area                     | Improvement and consequence                                                                                          |
| ------------------------ | -------------------------------------------------------------------------------------------------------------------- |
| Toolchain packaging      | More selective packaging of Rust toolchain contents; better handling of distribution toolchains using `lib64`        |
| Toolchain identity       | Rust sysroot incorporated into the weak toolchain key, reducing incorrect equivalence                                |
| Cache lifecycle          | Evicted toolchain archives can be repackaged instead of leaving later requests unable to reuse that toolchain        |
| Scheduler lifecycle      | Job IDs are namespaced across restarts, avoiding collisions between old and new scheduler states                     |
| Package completeness     | Correct handling of certain Rust dependencies without metadata hashes                                                |
| Sandboxed execution      | Improved path handling for overlay builders operating inside chroot environments                                     |
| Error reporting          | Remote compilation failures preserve actual exit-code information rather than always appearing as signal termination |
| Client toolchain storage | `SCCACHE_DIST_CLIENT_CACHE_DIR` is honored as an override for the distributed client's toolchain-cache directory     |

These changes affect correctness, recovery, and interoperability under distributed execution.

One consequential distinction is that a distributed compile requires not only a compiler executable but also a correctly identified and packaged compilation environment. Toolchain identity, sysroot contents, sandbox path interpretation, and archive lifecycle can all affect whether the remote execution is equivalent to local compilation.

The improvements do not eliminate the general security and operational obligations associated with remote build servers, nor do they turn remote caching into distributed compilation.

Source: sccache 0.19.0 release notes.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://github.com\&sz=32)

GitHub



`https://github.com/mozilla/sccache/releases/tag/v0.19.0`

## 4. cargo-hakari — Advanced graph analysis and configuration semantics

### 4.1 Cargo.lock as part of the feature-unification contract

cargo-hakari explicitly requires committing `Cargo.lock` to version control, including in workspaces containing only library crates.

This is a stronger constraint than merely regenerating the workspace-hack package after a known dependency change.

Cargo's version resolution permits compatible releases to introduce additional features, alter transitive dependencies, or change how existing dependencies are activated.

Hakari's generated dependency declarations are computed from a particular resolved dependency graph. If that graph can change independently of the checked-in workspace-hack manifest, the intended feature-unification properties may no longer hold.

Consequently, `Cargo.lock` acts as part of the reproducible input to Hakari's computation. Lockfile stability is important even when Cargo's conventional recommendations for distributing a library would not otherwise require an accompanying lockfile.

This is a documented requirement of cargo-hakari, rather than a general claim that every Rust library must commit its lockfile.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust



Primary source:

`https://docs.rs/cargo-hakari/latest/cargo_hakari/`

### 4.2 Iterative feature-unification computation

The underlying `hakari` library performs a more sophisticated computation than simply gathering every feature named in workspace manifests.

Its algorithm has three conceptual stages.

Build simulation. Using `guppy`, Hakari simulates Cargo builds for workspace packages across the configured platforms and feature configurations. These include builds with no features, default features, and all features. It records the sets of features used for each dependency.

Candidate identification. Dependencies that appear with multiple distinct feature sets are candidates for inclusion in the workspace-hack's generated declarations.

Fixpoint computation. Introducing those candidates into the workspace-hack can itself alter dependency feature activation. Hakari therefore simulates the consequences of the proposed output, identifies additional dependencies that have become fragmented, and repeats until no new candidates emerge.

Conceptually, if \\(H_k\\) denotes the generated unification set at iteration \\(k\\), the computation can be represented as:

\\[ H\_{k+1} = H_k \cup D(H_k) \\]

where \\(D(H_k)\\) represents additional dependencies requiring unification after accounting for the current proposed workspace-hack declarations.

The process terminates when:

\\[ H\_{k+1} = H_k \\]

This is a conceptual description of the documented fixpoint algorithm, not a reproduction of its internal data structures.

The implementation parallelizes parts of the computation using Rayon.

The key architectural implication is that feature unification is an iterative graph operation. A dependency that initially appears to have only one feature configuration can become relevant after other dependencies have been unified.

The generated manifest therefore represents a computed property of the dependency graph, not a direct transcription of its original dependency declarations.

Source: hakari 0.18.0 library documentation.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust



`https://docs.rs/hakari/latest/hakari/`

### 4.3 Exclusion behavior affects workspace dependency edges

The original guide distinguishes `traversal-excludes` from `final-excludes`. An additional important distinction is that exclusions affecting workspace members also influence Hakari's management of dependencies on the generated crate.

For a workspace member included in either exclusion category, `cargo hakari manage-deps` can remove its workspace-hack dependency rather than adding or preserving it.

This means exclusions affect two structures: the dependency sets participating in feature analysis, and the workspace members that acquire dependency edges to the generated crate.

The configuration supports more precise package identification than a plain crate-name exclusion.

Third-party exclusions can specify version ranges, Git dependency identities, or path dependencies.

For example:

```
[traversal-excludes]
third-party = [
    { name = "example-backend", version = "1.0" },
    { name = "custom-parser", git = "https://example.com/parser.git" }
]
```

This is illustrative configuration, not a claim that the named packages exist.

Version-scoped exclusions are useful when one dependency version has feature interactions that should not be unified, while other versions do not have the same constraints.

Hakari separately maintains structural excludes: packages whose inclusion would introduce dependency cycles. These are excluded automatically, while their own dependencies may remain eligible for unification.

The distinction matters because explicit exclusions express configuration policy, whereas structural exclusions arise from graph constraints identified by the computation.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust



Primary source:

`https://docs.rs/cargo-hakari/latest/cargo_hakari/config/`

### 4.4 Dependency output formats affect more than presentation

The `dep-format-version` setting is not merely a formatting preference.

Its versions represent successive changes to the structure and normalization of generated Cargo declarations:

| Format version | Documented behavior                                                                                              |
| -------------- | ---------------------------------------------------------------------------------------------------------------- |
| `1`            | Path-based dependency declarations                                                                               |
| `2`            | Adds version information needed for the documented publishing configuration                                      |
| `3`            | Removes private and other non-root features from generated output and elides build metadata from version strings |
| `4`            | Corrects generated dependency ordering to consistently alphabetical sorting                                      |

Format version `2` is particularly significant because the publishing strategy using a registry stub and root-level patch relies on a versioned dependency declaration.

Later formats improve the generated manifest's normalization and determinism, reducing unnecessary or misleading output.

The distinct `workspace-hack-line-style` setting controls the form of dependency lines managed across workspace manifests.

When changing this setting, upstream documents removing existing managed dependency lines and then regenerating them through `cargo hakari manage-deps`. The reason is that changing the expected representation does not necessarily imply that previously written forms are automatically reconciled in every case.

This is an example of a generated-manifest interface whose textual representation has maintenance implications independent of the underlying feature graph.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust



`https://docs.rs/cargo-hakari/latest/cargo_hakari/config/`

### 4.5 Computational capabilities not implemented in hakari 0.18.0

The upstream `hakari` documentation explicitly identifies several missing capabilities.

| Capability                                           | Documented status | Consequence                                                                                     |
| ---------------------------------------------------- | ----------------- | ----------------------------------------------------------------------------------------------- |
| Explicit simulation of cross-compilation scenarios   | Not implemented   | Platform simulation does not cover every distinct host/target compilation arrangement           |
| Platform-specific exclusions                         | Not implemented   | Exclusion policy cannot generally be specialized independently for each target                  |
| Inclusion-only selection of arbitrary final packages | Not implemented   | There is no general facility to select only a particular subset of dependencies for unification |

The third limitation should be distinguished from `final-excludes`. Excluding unwanted packages from output is not the same API capability as explicitly selecting a closed inclusion set.

Likewise, the `unify-target-host` strategies described in the original guide do not constitute general cross-compilation simulation. They determine how feature-unification computation treats host-side and target-side dependency roles within the supported model.

These are documented limitations of the released library. Upstream identifies them as possible future work, but that does not establish a release commitment or implementation schedule.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust



`https://docs.rs/hakari/latest/hakari/`

### 4.6 Generated manifest and platform compatibility details

Two smaller behaviors are worth recording.

Generated-section boundaries. The `hakari` library recognizes marked generated sections inside the workspace-hack manifest. Content outside those sections can be maintained independently, while the generated section is controlled through Hakari's serialization machinery.

The public `HakariCargoToml` representation supports writing generated content to files, constructing in-memory representations, and producing diffs.

Windows line endings. cargo-hakari's generated workspace-hack manifest does not support CRLF line endings. Its initialization command handles the relevant Git configuration in supported setups.

These constraints are useful when embedding Hakari's generation machinery into larger manifest-management tools or working with cross-platform repositories.

Sources: cargo-hakari and hakari documentation.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+1



`https://docs.rs/cargo-hakari/latest/cargo_hakari/`

## 5. cargo-shear — Test-target configuration reconciliation

### 5.1 Bidirectional test and doctest configuration analysis

cargo-shear's `--check-test-targets` option provides more than detection of disabled tests.

For library targets, it compares source content against the relevant test and doctest settings in `Cargo.toml`.

The analysis detects two different categories of inconsistency:

| Manifest condition                         | Source condition                | Diagnostic meaning                                                          |
| ------------------------------------------ | ------------------------------- | --------------------------------------------------------------------------- |
| `test = false`                             | Test content is present         | Tests appear to have been explicitly disabled despite existing source tests |
| `doctest = false`                          | Documentation tests are present | Doctests appear to have been disabled despite existing doctest content      |
| Test setting remains enabled by default    | No corresponding tests found    | Target may have unnecessary test execution enabled                          |
| Doctest setting remains enabled by default | No corresponding doctests found | Target may have unnecessary doctest execution enabled                       |

The checks for default-enabled targets without relevant tests apply within workspaces, as documented by the project.

The capability is opt-in and can be combined with automated remediation:

```
cargo shear --check-test-targets --fix
```

This converts source/manifest inconsistencies into directly maintainable Cargo target configuration.

The benefit is not limited to eliminating unnecessary test targets. It can also expose situations in which available tests are unexpectedly excluded from normal Cargo execution.

The analysis is still source-based. It should not be treated as proof that every configuration-specific, macro-generated, or externally generated test has been discovered.

Source: cargo-shear 1.14.0 documentation.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Docs.rs



`https://docs.rs/crate/cargo-shear/latest/source/README.md`

## 6. Cargo mechanisms that affect the three tools

The following native Cargo features materially influence the performance and correctness of the surrounding toolchain. They are Cargo capabilities, not functionality implemented by cargo-hakari, sccache, or cargo-shear.

### 6.1 General and workspace-specific Rust compiler wrappers

Cargo exposes two different wrapper configuration mechanisms:

| Configuration                                               | Scope                                 |
| ----------------------------------------------------------- | ------------------------------------- |
| `build.rustc-wrapper` / `RUSTC_WRAPPER`                     | General Cargo rustc invocations       |
| `build.rustc-workspace-wrapper` / `RUSTC_WORKSPACE_WRAPPER` | Rust compilation of workspace members |

When both are configured, Cargo nests the wrappers:

```
RUSTC_WRAPPER RUSTC_WORKSPACE_WRAPPER RUSTC
```

The workspace wrapper also affects the artifact filename hash, causing the corresponding compiler outputs to be distinguished from outputs built without that wrapper.

This is important for compiler-cache systems because a wrapper is not necessarily just a transparent execution step. Cargo can make wrapper identity part of its own local artifact separation.

The two mechanisms enable different integration arrangements. A general wrapper can intercept both workspace and external dependency compilation, while the workspace-specific wrapper can apply additional processing selectively to workspace members.

Using both introduces an ordered wrapper composition whose argument forwarding and effects must remain compatible.

Source: Cargo configuration reference.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://doc.rust-lang.org\&sz=32)

The Cargo Book



`https://doc.rust-lang.org/cargo/reference/config.html`

### 6.2 Separation of final artifacts and intermediate build storage

Cargo distinguishes its final output directory from the directory containing intermediate compiler artifacts.

The relevant configuration keys are:

- `build.target-dir`, also configurable through `CARGO_TARGET_DIR`, for final build artifacts.
- `build.build-dir`, also configurable through `CARGO_BUILD_BUILD_DIR`, for intermediate build artifacts.

By default, both refer to the same directory.

The separation is meaningful because intermediate artifacts include compiler-generated dependencies, build-script outputs, and incremental compilation state, whereas final artifacts are the outputs intended for direct consumption.

Cargo's `build.build-dir` configuration supports path templates including `{workspace-root}`, `{cargo-cache-home}`, and `{workspace-path-hash}`.

This allows more explicit management of where build state accumulates, independently of where final products are written.

However, relocating Cargo's intermediate build directory does not automatically establish a content-addressed compiler cache comparable to sccache. Cargo still uses its own artifact validity and dependency-tracking mechanisms.

This is therefore a storage-layout capability, not an implicit cross-project compilation-sharing mechanism.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://doc.rust-lang.org\&sz=32)

The Cargo Book+1



`https://doc.rust-lang.org/cargo/reference/build-cache.html`

### 6.3 Target selection changes artifact-sharing behavior

Cargo's behavior differs depending on whether compilation explicitly specifies a target.

Without `--target`, Cargo builds for the host and can share certain compiled dependency artifacts with build scripts and procedural macros. Compiler flags may also apply across both ordinary target dependencies and host-side compilation.

With an explicit target, Cargo separates host-side compilation from target-side outputs and applies certain compiler flags only to the requested target.

Even when the explicit target triple is identical to the host, this distinction can affect compilation partitioning and reuse.

The implications extend across the original guide's three optimization mechanisms.

For cargo-hakari, it changes the context in which host and target dependency feature sets are meaningful.

For sccache, distinct compiler arguments and effective compilation environments can produce different cache identities.

For dependency hygiene, the effective graph used by a particular build can differ from the complete set of dependencies declared across all target conditions.

This is a native Cargo behavior and should not be interpreted as a capability for Hakari to simulate arbitrary cross-compilation scenarios.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://doc.rust-lang.org\&sz=32)

The Cargo Book+1



`https://doc.rust-lang.org/cargo/reference/config.html`

### 6.4 Package-specific profiles and build-dependency overrides

Cargo permits compiler-profile settings to vary by package and build role.

For example:

```
[profile.dev.package."*"]
opt-level = 2

[profile.dev.build-override]
opt-level = 0
```

The first configuration applies to non-workspace dependencies, subject to more specific overrides. The second affects build scripts, procedural macros, and their dependencies.

These mechanisms can change both compiler cost and artifact identity without altering package dependency declarations.

A consequential interaction occurs when the same dependency is used in both normal compilation and build-time compilation.

Cargo may otherwise be able to share certain compiled artifacts between those roles. Introducing different effective compiler-profile settings can require separate compilation units.

Thus, an optimization intended to reduce the cost of one compilation role can increase the total number of artifact variants.

This illustrates why dependency feature unification alone cannot eliminate all duplicate compilation. Different feature sets are only one reason equivalent-looking packages may require different artifacts.

Source: Cargo profiles reference.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://doc.rust-lang.org\&sz=32)

The Cargo Book



`https://doc.rust-lang.org/cargo/reference/profiles.html`

### 6.5 Compiler-unit timing and dependency critical-path analysis

Cargo's `--timings` interface provides a complementary view of build performance.

```
cargo build --timings
```

The generated HTML report includes compilation-unit durations, concurrency, dependency relationships, and code-generation timing where available.

This information can reveal the critical path through a build graph.

For example, reducing the duration of a dependency that blocks many downstream crates can improve overall elapsed time more than reducing equivalent cumulative compiler time in independent compilation units.

The distinction can be expressed using a dependency graph \\(G=(V,E)\\), where each compilation unit \\(v\\) has duration \\(t(v)\\).

Ignoring resource contention, a lower bound on the elapsed build duration is the weighted critical path:

\\[ T\_{\mathrm{critical}} = \max\_{p\in\mathcal{P}(G)} \sum\_{v\in p}t(v) \\]

where \\(\mathcal{P}(G)\\) is the set of dependency paths.

Total compiler work is instead approximately:

\\[ W\_{\mathrm{total}}=\sum\_{v\in V}t(v) \\]

These are different quantities.

sccache primarily reduces execution costs for eligible compiler requests, while Hakari can reduce repeated work caused by feature-configuration fragmentation. Their effects on elapsed time depend on where the avoided work lies in the build's dependency and resource-scheduling structure.

Cargo timing reports therefore complement cache statistics by revealing whether a reduction in compiler work also shortens the build's critical path.

The actual elapsed time also depends on available parallelism, resource contention, linking, and activities outside the compiler units shown in the report.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://doc.rust-lang.org\&sz=32)

The Cargo Book



Primary source:

`https://doc.rust-lang.org/cargo/reference/timings.html`

## 7. Release boundaries and qualifications

The capabilities in this addendum have been described against the tagged released versions or current published Cargo documentation identified at the beginning.

Several distinctions are important when comparing this addendum with later development-branch documentation.

sccache incremental compilation. The released Rust documentation continues to identify incrementally compiled Rust invocations as non-cacheable. Development-branch material discusses additional controls around allowing incremental builds to proceed, but those should not be assumed to be available in version 0.19.0 without verification against that release's implementation.

sccache execution architecture. Client-side mode is supported in 0.19.0, but its eventual replacement of server-side execution remains a stated upstream direction, not an already completed architectural transition.

Hakari future functionality. Cross-compilation simulation, platform-specific excludes, and inclusion-only package selection are documented as missing from hakari 0.18.0. Their appearance in a future-work section is not evidence that they are released.

Performance claims. The described improvements explain mechanisms that can reduce preparation, compilation, storage, or coordination overhead. They do not establish quantitative speedups for an unspecified project or guarantee that every change reduces total build duration.

Primary release and version references:

`https://github.com/mozilla/sccache/releases/tag/v0.19.0`

`https://docs.rs/cargo-hakari/latest/cargo_hakari/`

`https://docs.rs/hakari/latest/hakari/`

`https://docs.rs/crate/cargo-shear/latest/source/CHANGELOG.md`

The additional capabilities reinforce an important architectural distinction: Rust build performance depends not only on which dependency artifacts are reusable, but also on when metadata becomes available, how compiler requests are prepared, how artifacts are identified and validated, how storage failures are handled, and which compilation units determine the build's critical path. These mechanisms can produce meaningful improvements independently of conventional cache hit rates or reductions in dependency count.