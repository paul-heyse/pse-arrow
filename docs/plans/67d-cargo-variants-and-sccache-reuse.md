---
title: "Plan67d: Cargo variants and sccache reuse"
status: complete
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

Inspect the compiler graphs for the changed test routes and record the consumer of each
materially distinct configuration. Group scenarios with the same compiler inputs. Compare
before and after within the same Cargo operation and distinguish a change of target from a
change to the dependency graph. Remove variants that arose solely from different actors, seeds,
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

Use the existing managed correctness entry point for verification. Cargo freshness reuses
available unchanged outputs directly. Incremental compilation can reuse work after an edit
to a workspace crate. Explicit non-incremental verification enables sccache recovery for
eligible requests with matching inputs when outputs are absent. Imported libraries already
compile non-incrementally under the ordinary profile.

Use the existing no-default check/test comparison, source edit, and controlled absent-output
restores to decide the default. For a named route with recurring cold recovery cost, first
check that eligible compilation is on its critical path. Compare the two modes on that route
with matched inputs and output availability. Repeat enough to distinguish a worthwhile
difference from run-to-run variation before adopting the route. Coverage instrumentation has
its own compiler inputs and needs a comparison when a named coverage workflow warrants one.
Prefer Cargo's `--config build.incremental=false` when it supplies the intended effective mode
without adding a separately hashed `CARGO_INCREMENTAL` value. The ordinary edit loop keeps
its warm per-checkout target.
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
failures warrant it. Start comparisons at the 100 GiB managed baseline and keep ordinary
eviction. The maintainer permits a larger allocation when it improves reuse. Compare
representative compatible repeat consumers at the same keys and record occupancy, surviving
entries, and rebuild cost. Increase the allocation if useful entries disappear at capacity
and a larger isolated comparison recovers that work. Old Rust key
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
| D01 | complete | Current Cargo selection for baseline. C02 migration results for comparison. | Changed-route compatibility matrix reconciled by Cargo operation. Removed scenario-feature callers are absent. Selected native test closures retain force validation. |
| D02 | complete | D01 census and current correctness routes | Existing edit and restore controls support ordinary incremental builds, explicit non-incremental recovery, and checkout-owned campaign layout. Adoption of another recurring lane requires a named workload benefit. |
| D03 | complete | Managed 0.19.0 and stable representative selections. | Metadata recovery, changed-input and fallback controls, and actual large-entry restoration on the selected managed daemon. D02 owns verification-lane adoption. |
| D04 | complete | Native owner and representative command context | Verified existing provider preparation and native object restoration. Keep direct preprocessing disabled after the negative-header controls. |
| D05 | complete | D03 confirms compatible requests and an observable daemon. A trial can use private infrastructure. | Verified private disk/WebDAV promotion and failure behavior. Defer deployment until a named compatible consumer shows representative transfer and recovery savings. |

## Execution checkpoint — November 7

The initial native unit census examined 498 units for the `pse-operations` library with
`canonical-tests` and 810 for the `native_service` target with `native-service`. Each
selection had one matching `arrow-array` dependency unit and native correctness selected
explicit force validation. This establishes the two source contexts for the comparison.
The migrated xtask `evidence-tools` and `host-contracts` selection examined 804 units
and reached zero `pse-runtime` compiler units after removing its test-only feature.
The pre-migration recapture reproduced those three contexts and examined 1,082 units with
one force-validated Arrow leaf for the `pse-runtime` library selection. The bounded
post-migration matrix and its current consumers are reconciled below.

Two force-validated pinned Cargo check selections exercised `pse-runtime` and `pse-admission`
as separate roots. Each selected one `arrow-array` and one DataFusion library unit, with
matching features, effective profiles and recursively matched dependency units. Symbolica
appeared only in the runtime selection. This verifies reuse-compatible planning for those
shared libraries and preserves the narrower admission build. Build time is measured separately.

The managed route now verifies the selected 0.19.0 binary and reports actual daemon
readiness, storage location, mode, and listener multiplicity. It validates a selected frame
size and keeps alternate endpoints paired with alternate storage roots. Cold host preparation
explicitly selects direct Rust compilation even when a user Cargo config supplies a wrapper.
The isolated pinned-nightly metadata probe restored both outputs byte-for-byte and missed after
an included source changed. The client-side handshake refusal was identical with
`SCCACHE_IGNORE_SERVER_IO_ERROR` set and unset. A separate six-client control exposed six
listeners on one stock 0.19 filesystem socket. The later single-unit Symbolica restore
below completes the stock large-output control.

The explicit diagnostic-tool installer built the managed Linux 0.19.0 patch from upstream
revision `965c01943a40c9df96264474bbe28ba98f2e1487` with its locked dependencies and
published the verified executable and source-build receipt. The lifetime directory and
endpoint locks precede storage initialization. Native environment resolution reserves the
physical default pair for the declared executable and permits explicit isolated pairs.
`bash scripts/sccache-source/probe.sh <managed-executable>` observed one listener while six
GCC processes reached code generation together. The probe checked exact C object and Rust
metadata restores, local dep-info, included-source misses, alias handling, conflicts, and
verified drain and restart. The default cutover preserved the existing repository cache
directory identity and completed a GCC compile. Scoped `scripts/pse-host doctor --format json`
observed the patched binary and a single read-write disk daemon at 100 GiB, with a 256 MiB
frame and direct preprocessing disabled. The system cache continued to serve its separate
directory. The large real-unit restore on the patched daemon and end-to-end timings remain
open. [Managed cache guide](../dev/build-performance.md)

After cutover the cache held 45.77 GiB indexed and 45.95 GiB allocated. The earlier
43 and 46 GiB snapshots and aggregate hit and miss counters give no evidence of
capacity-driven rebuilds. The allocation remains 100 GiB pending a matched retention
and restore comparison at capacity.

An isolated pinned-nightly Cargo package with the managed binary showed a warm incremental
freshness path without cache work and a non-incremental check miss and write followed by a
Rust hit after its selected outputs were removed. Under the 256 MiB frame, a separate
non-incremental Cargo build restored a 41,947,450-byte library and a 41,945,356-byte metadata
file byte-for-byte with one write, one hit and zero write errors. The controls establish
eligibility and output recovery. Repository workload timing still governs lane adoption.
The raw correctness adapter can carry Cargo's explicit `--config build.incremental=false`.
The native selection now carries `--non-incremental` through admission, source build
and the original policy for fresh rerun. Older records keep Cargo's original defaults.
Cargo's explicit environment override can otherwise make a nominally non-incremental
selection compile incrementally, so the lane refuses a conflicting `CARGO_INCREMENTAL`.
Native admission verified the actual Cargo configuration argument. The pinned Cargo
timings stream starts with a distinct JSON marker, which the compiler observer now
accepts while still requiring a valid terminal message. The timed non-incremental
`pse-diagnostics` library check and selected process-identity test both passed with
complete compiler capture and settlement.
The final D02 disposition below retains the existing default and checkout-owned campaign layout.

A pinned Symbolica 3.0.1 library unit used the managed 0.19.0 binary, client-side disk
cache and 256 MiB frame. The timed unit used the ordinary MPFR and GMP features and an
isolated trial marker. The cold compile took 87.22 s and wrote one cache entry. Replaying
the same Rust request into an empty output directory took 0.58 s and reported one Rust hit
with zero write errors. The 56,889,532-byte library and 27,864,632-byte metadata matched
exactly. The recovery produced local dep-info in the alternate directory. These timings cover
the individual compiler request and leave end-to-end verification latency open.

The native CMake bridge already covers CoolProp and KLU. The pinned `cc` crate uses the
selected wrapper for its compatible local shims. PETSc
`make all` now supplies a compiler launcher only for C/C++ compilation. The pinned
PETSc translation-unit probe restored both object and dependency output byte-for-byte,
as did the CoolProp CMake bridge controls. Direct mode restored stale output after a new
optional include and a newly introduced shadow header in the private GCC probe. Ordinary
preprocessing correctly missed in both cases. Keep direct mode disabled.

The installed patched daemon passed an isolated build of the actual pinned SuiteSparse
`KLU_static` CMake target and its prerequisites using the native owner options, repository
toolchain bridge, CMake 3.28.3, and GCC 13.3. The cold build wrote 130 entries. After deleting
one selected object's outputs, the repeated CMake target reported one hit, no additional
miss, and byte-identical `klu_version.c.o`, `klu_version.c.o.d`, and `libklu.a`. Replaying
the CMake compile with a private forced header produced a hit for identical inputs and a
miss with changed object bytes after a header edit. Direct preprocessing stayed disabled
and write errors stayed at zero. The private build, cache, and daemon were released. This
exercises the CMake compiler route and leaves the enclosing native owner's publication
and readmission to an explicit `native prepare --capabilities compiler,klu` run.

**Tested.** Explicit native preparation published KLU, CoolProp, and PETSc to one
isolated provider root. A second KLU preparation returned the same immutable path.
A consumer entered the native boundary with `compiler,klu,petsc,coolprop` and used
the selected KLU and PETSc headers and libraries and the selected CoolProp dynamic
library. The three executable controls passed. Cold GCC/G++ requests wrote three
entries to a private 0.19.0 disk daemon. After removing the outputs, the same
commands produced three hits and restored all six object and dependency files
byte-for-byte. Both passes recorded zero cache write errors. The CoolProp control
used absolute-path dynamic loading, as the Rust binding does.

**Measured.** A separate disposable checkout ran the actual workspace Symbolica
3.0.1 library compiler command with `integer-gmp` and `float-mpfr`. The trial
recorded argv and the digests of its 23 extern inputs. Its isolated managed 0.19.0
daemon used client-side disk cache and a 256 MiB frame. The selected unit took
108.99 seconds on its miss. Two independent requests with equivalent keys and
empty output directories completed in 1.15 and 1.23 seconds, each adding one
Rust hit and zero cache write errors. They restored the 56,889,532-byte rlib and
27,866,942-byte metadata byte-for-byte and wrote local dep-info matching after
normalizing the output directory. These timings isolate the selected compiler
command. These measurements establish that eligible missing library outputs benefit from recovery.

**Measured.** The disposable checkout ran three pairs per mode of
`pse check pse-diagnostics --lib --no-default-features --timings` and the matching
`pse test` selecting `process_facts_live_identity_and_missing_are_explicit`.
All twelve invocations passed and settled. After the first build in each mode,
the default Cargo check phases took 0.98–2.79 seconds and the non-incremental phases
took 0.91–0.94 seconds. The corresponding test build phases took 1.36–1.64 and
1.34–1.37 seconds. On the same authored source edit, the check phases took 1.25
and 2.23 seconds and the test build phases took 1.91 and 4.96 seconds. Concurrent
work varied during this control. These observations support retaining the ordinary default.

The no-default split-directory control used the same checkout and scoped correctness
adapter with explicit `build.build-dir`. Its check passed, writing 188,219,392
allocated bytes of separate intermediates and taking 7.52 seconds in Cargo.
The checkout-local default retains its existing freshness. The trial uses the
checkout retirement boundary for the separate intermediates.

The live managed daemon's later readback reported 49,389,964,299 indexed bytes under
its 107,374,182,400-byte limit, `ReadWrite`, and zero reported write errors. This sample
does not establish capacity-driven eviction. The separate stock listener remains active
outside the managed socket. Its local cache has independent ownership.

**Tested.** The runtime all-target check with `--no-default-features` passed with zero
errors in the 1,099-unit route. It exposed four warning records, and the source moved
the fixture helpers behind their actual test consumers and removed redundant qualifiers.
The scoped default `pse check pse-operations --lib --features native-service,canonical-maintenance --timings`
passed with zero compiler errors or warnings and a 72.74-second Cargo phase after rebuilding
metadata. It supplies one default-mode operations sample.

An isolated PETSc 3.24.0 configuration used the authenticated archive and repository patch,
the selected GCC and MKL file digests, and the native owner's configure flags. The actual
GNU Make rule for `src/sys/utils/str.c` invoked the installed managed launcher through
the same `CC` and `CXX` overrides as `make all`. Removing the selected outputs and
repeating that target yielded one cold write and one restore hit overall, with identical
`str.o` and `str.d` and zero write errors. The trial drained its private cache daemon and
released the scratch source. Full PETSc installation and provider readmission remain separate.

The route inventory found the same CMake launcher bridge in HiGHS, SUNDIALS, KLU, CoolProp,
UNO, and root isolation. The pinned IBEX source attaches bundled FILIB and SoPlex with
`add_subdirectory`, so they inherit the parent CMake launcher. The pinned `cc` crate
recognizes the selected wrapper for the Ipopt, UNO, PETSc, and native backend shims.
GMP/MPFR construction keeps the existing upstream compiled-library cache. Native owner
qualification can verify actual reuse without adding a second launcher to these routes.

The private WebDAV control recorded an initial absent key, read-only L1 restore and L0
backfill, credential refusal, remote write refusal, and mid-run and startup unavailability.
The `l0` policy continued using local storage. The `all` startup policy refused the
unavailable tier. The consumer matched the producer's object and dependency bytes, then
served the next compile without a remote request. Defer a deployed tier until a named
compatible consumer establishes representative savings.

## Acceptance

Preserve the actual selected test assertions, provider integrity, and force-validation
closures. Verify removal of redundant compiler configurations for migrated scenarios,
useful same-checkout recovery on absent outputs, accurate invalidation for changed inputs,
and explicit cache error and bypass cases. Establish end-to-end speedup only with
comparable timed executions.
Keep command, mode, scope, and observed failures against the zero target with each result.
An isolated cache probe qualifies its own route. Product acceptance remains with Plan66
and the migrated test owners.

## Outcome

**Interface-checked.** The pinned nightly and lockfile were used for
`pse inspect dependency-units` and `pse inspect test-selection --admitted` on the changed
Linux host selections. The following counts describe selected compiler units in their
indicated operation. The WASI row uses the producer's separate Cargo build operation.
Every listed native test graph selected one `arrow-array` and one `arrow-data` unit
with `force_validate`.

| Actual consumer and Cargo selection | Check units | Admitted test units |
|---|---:|---:|
| Operations library and canonical scenarios | 498 → 498 | 580 |
| Operations native service and maintenance | 819 → 819, all targets | 799, `native_service` |
| Runtime ordinary science, in-process authored input | 1,099, all targets with no defaults | 1,062, library with no defaults |
| Runtime native Study, service and KINSOL/Ipopt | 1,129 integration → 1,120 library | 1,093 integration → 1,094 library with product operators |
| Runtime engineering routes, including Diffsol | — | 1,101, `engineering_service` |
| Independent PID comparison, with Diffsol | — | 1,084, `reference_pid_exchange` |
| Standalone portable native lane, with KINSOL | — | 1,091, runtime consumer library |
| xtask ordinary and developer evidence | 551 ordinary and 804 with `evidence-tools,host-contracts` | — |
| Production Study module, `wasm32-wasip2` release | 300 units from the producer's Cargo build graph | — |

The Operations check counts match within each workload. The scenario flags changed the
compiler configuration even when the unit count held steady. Live manifests, Rust sources,
recipes and scripts have zero references to the retired `canonical-tests`, `native-study-tests`
and `pse-operations/test-support` selections. The Study comparison changes the root and
records the test selection's product-operator coverage separately from the check count.
The xtask evidence route reaches zero Runtime units. Native solvers, the executable lane
producer, and the WASI module remain distinct where their owners need them. The WASI
production graph contains no Arrow. The module owner's qualification variants preserve
separate artifact identities and production-admission rules.

The pinned cargo-shear scan reported one error and one warning against a zero target.
Source inspection confirmed explicit Rust links for generated modules and service files.
Generated policy files are generator-owned. The preexisting `buffa-types` declaration
candidate is outside the scenario migration and the dependency remains reachable through
`pse-host`. The lockfile and workspace feature-unification policy were kept.
[Study module owner](../../crates/pse-host/src/study_module.rs)

**Measured.** The existing no-default Diagnostics comparison exercised warm reuse and a
real source edit. The Symbolica and metadata probes exercised recovery with outputs absent.
The split-directory control exercised a separate intermediate root. The ordinary
Operations check after rebuilding metadata was a single sample. The initial execution
proposal would have repeated both modes across the Operations check and test without
identifying a workflow whose compiler critical path warranted a different policy.
The final decision uses the existing controls. The Operations sample establishes baseline
timing for that command.

**Implemented.** Ordinary editing uses Cargo freshness and incremental workspace builds.
Imported dependencies already use the managed cache when eligible. Explicit
`--non-incremental` remains available for a named recovery workload. Campaigns use their
checkout-owned target and the existing settlement and retirement route. The managed
sccache 0.19.0 budget stays at 100 GiB. A future route change needs a named workflow with
recurring absent-output cost or an independently measured storage or latency benefit.
[Build guide](../dev/build-performance.md)
