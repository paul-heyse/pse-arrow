---
title: "Plan67d: Cargo variants and sccache reuse"
status: in-progress
date: 2026-11-06
doc_role: plan
doc_owner: docs/plans/67-storage-lifecycle-and-build-reuse.md
doc_topics: [current-work, design, validation]
review_sources: [docs/functional_reviews/storage-lifecycle-and-build-reuse-2026-11-06.md]
---

# Cargo variants and sccache reuse

## Scope and baseline

**Proposed.** Reduce avoidable Cargo unit variants, keep useful incremental state for ordinary
edits, and restore eligible Rust and native outputs on verification. This companion receives
[O07](../functional_reviews/storage-lifecycle-and-build-reuse-2026-11-06.md#O07) and
[O08](../functional_reviews/storage-lifecycle-and-build-reuse-2026-11-06.md#O08). The test
companion owns migration from test-driven features to actual scenarios. This companion measures
the compilation effect and owns build and cache routes. The retirement companion owns
release of settled campaign state. Plan66 continues to own scientific acceptance.
[Plan67](67-storage-lifecycle-and-build-reuse.md)

**Interface-checked at `c2126df45`.** The pinned Cargo nightly uses workspace feature
unification. `features-combinations` already requests `--config build.incremental=false`.
The ordinary dev/test context keeps interactive incremental work and imported dependencies use
non-incremental builds. The environment owner selects a checksum-pinned managed sccache 0.19.0
with client-side mode, the selected disk root and Unix socket, a 100 GiB budget, and a 256 MiB IPC
frame. The installed PATH binary also reports 0.19.0 and has independent selection and storage.
Reconcile the documented daemon ownership with the current direct executable route before
depending on the historical service or its memory allowance. [Build guide](../dev/build-performance.md)
· [Environment owner](../../crates/pse-host/src/environment.rs) ·
[Tool pin](../../.config/diagnostic-tools.toml) · [Build settings](../../.config/build.toml)

The review's private probes establish eligible metadata-only Rust and GCC restoration,
same-checkout rlib restoration, changed `env!` input invalidation, and client-side local
backfill after a hit in a read-only WebDAV tier. The loopback outage control exercised the
distinct `l0` and `all` policies. Use these as controls for the current binary. Repository-scale
reuse, current large-entry recovery, and remote transfer savings need their own measurements.
[Historical large-frame controls](../design_review/evidence/rust-build-reuse-019/README.md)

## Target and work boundaries

### D01 — Product compatibility matrix and unit census

Use current selected compiler units and native correctness closures to count work by package,
host/build/dev role, target, feature set, profile, and compiler or wrapper context. Record the
actual consumer of each materially distinct configuration. Keep a named compatibility matrix
for real product differences and remove units that arose solely from different actors, seeds,
faults, or test scenarios when the test companion migrates them. Preserve explicit Arrow
force validation in each actual correctness closure, required WASI and native-provider
distinctions, and observable build-script inputs. Use pinned cargo-shear 1.14.0
`--format=json` and `--check-test-targets` to identify candidate empty or duplicate harnesses,
missing test or doctest selection, and unused declarations. Use `--expand` only for specific
macro-mediated cases. Confirm real callers and feature/target conditions before removing
declarations. Keep the existing lockfile and native feature-unification owner.
[Cargo features](https://doc.rust-lang.org/cargo/reference/features.html) ·
[cargo-shear 1.14.0](https://docs.rs/crate/cargo-shear/1.14.0)

### D02 — Stable verification and campaign builds

Use the existing managed correctness entry point for verification. Compare one explicit
non-incremental configuration with the same route's current default on repeated check and
test selections, including the no-default and coverage paths where they do distinct work.
Prefer Cargo's `--config build.incremental=false` when it supplies the intended effective
mode without adding a separately hashed `CARGO_INCREMENTAL` environment value. Migrate a
named verification lane after equivalent coverage and an observed improvement in cacheable
recovery or useful build time. The ordinary edit loop keeps its warm per-checkout target.
Check that route for accidental explicit host `--target`, profile and build-dependency
override splits, duplicate wrapper composition, and needless target relocations. Preserve
needed target and native-link contracts. [Cargo profiles](https://doc.rust-lang.org/cargo/reference/profiles.html)
· [Cargo configuration](https://doc.rust-lang.org/cargo/reference/config.html)

Keep the ordinary checkout-local target and distinguish the inputs of genuine high-variance
campaigns. Compare a stable campaign target against Cargo's split `build.build-dir` for
intermediate artifacts only when separate ownership can make release or repeat preparation
cheaper. A changed build directory can change `OUT_DIR` and dependent keys. Keep the simpler
existing layout if the comparison fails to show a useful boundary. Release a retired campaign
as an owned unit after settlement. Use Cargo's supported cleanup boundary and keep the
ordinary target's freshness intact. [Cargo build cache](https://doc.rust-lang.org/cargo/reference/build-cache.html)
· [Resource retirement](67e-resource-retirement-and-storage-verification.md#E03)

### D03 — Qualify managed Rust caching and its observations

Trace which wrapper and effective configuration ordinary Cargo, native correctness, direnv,
and prepared tool builds actually reach. Report explicit cache-off, compiler-evidence routes,
ineligible requests, misses, successful publication, restores, and storage errors separately.
Keep one writer daemon per local directory and an independent endpoint for the managed route.
Report the cache mode, backend, version and size for the actual daemon. Test the server-I/O
fallback `SCCACHE_IGNORE_SERVER_IO_ERROR=1` for ordinary recovery before enabling it there.
Explicit cache experiments report bypass or failure clearly. Document surviving current
behavior in the build guide when the historical daemon claims have been reconciled.
[sccache local storage](https://github.com/mozilla/sccache/blob/v0.19.0/docs/Local.md)
· [sccache configuration](https://github.com/mozilla/sccache/blob/v0.19.0/docs/Configuration.md)

The tagged Rust implementation accepts eligible metadata output with
`--emit=dep-info,metadata` as well as eligible library codegen. Keep eligible check requests
on the managed route. Use ordinary Cargo freshness first. For equivalent cacheable requests
with missing outputs, confirm dep-info is written locally and verify an included-source edit
invalidates the restore. In 0.19.0, early `.rmeta` notifications can let Cargo overlap
dependent work on misses and output discovery overlaps hashing. Preserve the notification
path and inspect actual timing. Keys still include literal environment values, working
directory, and extern bytes. Compare
same-checkout recovery and a separate equivalently keyed consumer with their conditions
recorded. Use actual loader and compiled-test controls where links or external runtime inputs
matter. Thin test and command entry points are candidates where an identified expensive
reusable computation currently sits in a linker-bound target.
[0.19.0 release](https://github.com/mozilla/sccache/releases/tag/v0.19.0) ·
[Tagged Rust implementation](https://github.com/mozilla/sccache/blob/v0.19.0/src/compiler/rust.rs)

Run a representative large Rust restore in the current launch route, including Symbolica
or DataFusion and the applicable native closure. Confirm actual complete writes and restored
bytes under the configured 256 MiB frame, independently of the reported request counter.
Reuse an established build identity and compare with an absent output under an equivalent
key. Compare compression or frame settings only if measured restore costs or publication
failures warrant it. Keep the 100 GiB managed baseline and ordinary eviction. Old Rust key
version 6 entries may remain on disk after the version 7 upgrade. Attribute an observed
unreachable generation before considering owner-scoped release and avoid general age purges.
Keep shared scientific/provider artifacts outside compiler-cache eviction.

Combine `--show-adv-stats` and `--show-stats --stats-format=json` deltas with Cargo
`--timings`. Observe eligibility, published and restored bytes, key preparation and retrieval,
per-tier hits and errors, compile and link work, and critical-path wall time. Client-side
`compile_requests` alone is an incomplete denominator. Compare warm no-change Cargo
freshness, an ordinary edit, recovery after outputs are absent, and a required verification
or campaign build. Match source, toolchain, provider, profile, concurrency, and force
validation across meaningful comparisons. [Cargo timings](https://doc.rust-lang.org/cargo/reference/timings.html)

### D04 — Cover costly native compiler paths

Inventory the actual C and C++ commands from CMake, the KLU preparation, `cc`, and external
native builds. Preserve the existing CMake launcher and toolchain-file precedence and explicit
KLU launcher when those paths work. Extend the existing native configuration to a measured
costly unwrapped route using its supported launcher, and verify genuine GCC/Clang object
publication and restore with the same command and toolchain identity. Leave compiler
selection, provider receipts, generated headers and flags in their existing owners.

Keep `SCCACHE_DIRECT=false` on the ordinary route. Consider a direct preprocessor trial only
on a sealed header tree whose actual commands justify it. The 0.19 include-validation fix
makes the trial relevant. Verify changed, newly introduced, and shadowing headers miss as
required and match the uncached build. A failure retains the ordinary preprocessing path.
Use preprocessor diagnostics and isolated `SCCACHE_RECACHE` where useful for controlled
investigation. [Native CMake launcher](../../.config/native-cache.cmake) ·
[CMake 3.28 compiler launchers](https://cmake.org/cmake/help/v3.28/variable/CMAKE_LANG_COMPILER_LAUNCHER.html)
· [sccache local storage](https://github.com/mozilla/sccache/blob/v0.19.0/docs/Local.md)

### D05 — Optional second tier and deferred capabilities

Keep disk L0. Compare an isolated two-tier cache with a compatible clean consumer and one
realistic backend. Prefer S3-compatible storage or existing CI cache only when the
environment and matching keys justify it. A low-latency Redis or Memcached candidate needs
entry expiry and size compatible with the artifacts. Use a private test backend when a
compatible real consumer is unavailable, and defer adoption in that case. Use sccache's
`l0` write policy, startup capability checks, a trusted writable producer and read-only
remote consumers. Record no-entry, auth failure, write failure, startup outage, mid-run
outage, L1 restoration, local backfill, and subsequent L0 hit as separate cases. Qualify
credentials, transfer and decompression costs, and per-tier accounting. Keep an optional
tier only if a representative compatible consumer shows useful wall-time savings and
the local fallback remains truthful. Keep one authoritative backend configuration per
trial. Use backend-specific encryption, identity, prefixes, or WebDAV directory options
when the chosen backend requires them. [Tagged multi-level cache](https://github.com/mozilla/sccache/blob/v0.19.0/docs/MultiLevel.md)

Defer Rust path normalization until a released successor proves literal `env!` and
`file!`, generated inputs, proc macros, remaps, and changed externs against equivalent
and intentionally changed controls. The basedirs change landed after 0.19.0 and has
a separate correctness follow-up. Defer distributed execution until sustained misses
on multiple builders justify scheduler and supervision work. Defer Hakari until measured
avoidable graph fragmentation persists despite the pinned workspace unification. A
generated workspace-hack must show a lower total build and maintenance cost. Defer cache
resizing until representative workloads demonstrate a capacity bottleneck. Apple
multi-arch preprocessing, clang-cl/MSVC additions and CUDA/HIP remain unscheduled
until the relevant toolchain has a real consumer.
[Rust basedirs change](https://github.com/mozilla/sccache/pull/2918) ·
[Correctness follow-up](https://github.com/mozilla/sccache/pull/2794)

## Packages

| ID | Status | Prerequisites | Completion |
|---|---|---|---|
| D01 | in-progress | Current Cargo selection for baseline. C02 migration results for comparison. | Named product compatibility matrix, before/after compilation units, and verified removal of test-driven variants with their replacement. |
| D02 | planned | D01 census and current correctness routes | Verified stable non-incremental lane where justified, explicit campaign layout and release path, and preserved ordinary warm editing. |
| D03 | in-progress | Managed 0.19.0 and stable representative selections. Timing can begin before D02 rollout. | Truthful managed-route configuration and stats, representative metadata and large-entry restoration, changed-input and fallback controls, and recorded timings. |
| D04 | planned | Native owner and representative command context | Demonstrated restore of identified GCC/Clang outputs under existing provider contracts. A direct-mode trial needs separate negative-header evidence. |
| D05 | planned | D03 confirms compatible requests and an observable daemon. A trial can use private infrastructure. | Read-only remote promotion and outage evidence with a proceed or defer outcome for an optional real backend. |

## Execution checkpoint — November 7

The initial native unit census examined 498 units for the `pse-operations` library with
`canonical-tests` and 810 for the `native_service` target with `native-service`. Each
selection had one matching `arrow-array` dependency unit and native correctness selected
explicit force validation. This establishes the two source contexts for the comparison.
Consumer mapping and the post-migration variant census remain open.

The managed route now verifies the selected 0.19.0 binary and reports actual daemon
readiness, storage location, mode, and listener multiplicity. It validates a selected frame
size and keeps alternate endpoints paired with alternate storage roots. Cold host preparation
explicitly selects direct Rust compilation even when a user Cargo config supplies a wrapper.
The isolated pinned-nightly metadata probe restored both outputs byte-for-byte and missed after
an included source changed. The client-side handshake refusal was identical with
`SCCACHE_IGNORE_SERVER_IO_ERROR` set and unset. A separate six-client control exposed six
listeners on one stock 0.19 filesystem socket. The writer lifetime fix, representative large
restore, and critical-path measurements remain open.

## Acceptance

Preserve the actual selected test assertions, provider integrity, and force-validation
closures. Show smaller unit counts for migrated scenario coverage, useful same-checkout
recovery on absent outputs, accurate invalidation for changed inputs, and explicit cache
error and bypass cases. Establish end-to-end speedup only with comparable timed executions.
Keep command, mode, scope, and observed failures against the zero target with each result.
An isolated cache probe qualifies its own route. Product acceptance remains with Plan66
and the migrated test owners.
