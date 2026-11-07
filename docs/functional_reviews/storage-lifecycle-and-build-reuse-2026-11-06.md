# Storage lifecycle and build reuse

## Decision in brief

**Proposed.** Prevent structural growth by removing routine executable and database copies and
expressing test scenarios as tests. Use SurrealKit suites for database contracts and ordinary Rust
test targets and runtime inputs for application behavior. Keep historical developer evidence as
targeted SurrealDB records with an explicit use, owner, and release policy. Give checkouts,
campaigns, stores, and generations explicit retirement paths. Preserve warm Cargo state and the
managed 100 GiB sccache budget while increasing eligible Rust and native reuse.

The review covers the local Linux workstation and its related worktrees. Source inspection is based
on `c2126df45` and the Nov 6 working tree. Inputs were the complete
`docs/storage-lifecycle-review-2026-11-05.md` memo, the full Rust caching reference including its
0.19 addendum, and the SurrealDB and tooling references' storage, topology, seeding, testing,
rollout parity, and recovery sections. The maintainer's revision selects structural prevention
and SurrealKit-driven database tests. Design assessment follows core 3.4, its efficiency and test guidance, and the pse-arrow
binding. PS-13 applies where a fixture carries scientific assurance. Plan66 owns current work and
its E/F companions own protected data and developer evidence. The opportunities below are proposed
for selection. [Rust caching reference](../libraries/utilities/rust_caching.md) ·
[SurrealDB reference](../libraries/surrealDB/surrealdb.md) ·
[SurrealDB tooling reference](../libraries/surrealDB/surrealdb_tooling.md) ·
[Selected standard](../design_review/design_principles/standard.toml) ·
[Plan66](../plans/66-integrated-simulation-target.md)

## What survived the clean

The maintainer reports freeing approximately 900 GB the previous night, reaching approximately
1 TB free, and then returning to zero within about 24 hours due to pse-arrow activity.
`cargo clean` in main recovered approximately 350 GB. These are operator observations. The
November 5 memo documents an earlier, separately scoped 462.7 GiB cleanup. This review uses the
operator's larger incident as the operating scenario.

**Measured.** A read-only `uv run --no-cache --no-sync python` filesystem census at 22:59 EST
Nov 6 counted still-present regular-file allocated blocks across registered checkout directories,
project XDG state/cache/share, the separate global sccache, and `/tmp/pse-*` directories. It skipped
symlinks, mount crossings, and Git administration and counted hard-linked inodes once. There were
zero traversal errors. About 345 GiB was available on the root ext4 filesystem after the clean.

| Location | GiB with ctime in preceding 24 hours | What it contains |
|---|---:|---|
| Invocation ledgers outside the checkouts | 328.81 | Main 116.34, `p64-science` 100.91, `p65-contracts` 77.92, `p66-assessment-partials` 4.33, two currently unmatched checkout IDs 29.31 |
| Three other worktrees' Cargo targets | 276.84 | `p65-contracts` 162.66, `p64-science` 72.86, `p66-assessment-partials` 41.32 |
| Main fixture runtime captures | 35.15 | Prepared fixture runtime generations dominated by `xtask` and `pse-native-lane` |
| Other checkout files | 6.10 | Includes a 4.42 GiB diagnostic venv with recent ctime and older mtimes |
| Other project state | 4.90 | Includes 3.10 GiB under main products and smaller named stores |
| Managed and global sccache | 3.15 | 2.56 GiB managed and 0.59 GiB global |
| Other project cache, tools, and scratch | 0.14 | Rounded |
| **Total** | **655.09** | **650.49 GiB by mtime** |

The first three rows explain about 98% of the scanned recent footprint. Main's clean leaves all
three roots outside its Cargo cleanup scope. The census identifies where the surviving bytes are.
Both timestamps can reflect changes to existing files, and this post-clean observation has no
record of removed files, peak concurrent scratch, or net allocation over the day. An unprivileged
`lsof -nP +L1` check found zero visible deleted-open files at least 1 MiB, with partial host
visibility. These observations establish the dominant surviving classes rather than a reconstructed
disk timeline.

**Measured.** A follow-up used `find -size +128M` and GNU `stat %W` to check birth times for
large files in the top three classes. Of the recently changed large files, all 523.89 GiB were
created in the same 24-hour window: 319.73 GiB of ledgers, 169.19 GiB of linked-worktree targets,
and 34.97 GiB of fixture runtime captures. This verifies new file creation for that subset.

**Measured.** Total project state was about 548 GiB by `du -xhd1`; main held 230 GiB of invocations
and 43 GiB of fixtures. Matching adjacent capture receipts to regular files by length and grouping
their recorded SHA-256 values found 545 invocation inodes occupying 227.88 GiB for 253 distinct
receipt digests. One file per digest in that population would occupy about 161.05 GiB. The 58
fixture executable inodes occupied 42.24 GiB for 35 distinct digests, or about 28.52 GiB at one
file per digest. The two sets had zero digest overlap. The resulting roughly 80.6 GiB duplicate
estimate assumes the stored commitments still match all bytes and says nothing about deletion
authority. A SHA-256 spot check of two distinct single-link Nextest executables matched their
common recorded digest. There were 125 present Nextest copies in that group and 268 invocation
capture receipts whose executable was absent. Historical readers need an explicit missing-member
outcome. The separate compiler caches totaled about 46 GiB managed and 11 GiB global.

The database and tooling footprint still needs lifecycle work. In the selected named Plan49 store,
`database/` occupied about 128 MiB while 15 worker executables under `.generations/` occupied
13.95 GiB. The November 5 memo documents substantial earlier retirement of named stores. Inspect
each store by data, worker generations, snapshots, journals, payloads, and actual owners before
choosing a recovery or release action.

## Recommended opportunities

<a id="O01"></a>
### O01 — Measure whether workflows stop accumulating data

Use an on-demand, read-only footprint view to verify structural fixes and reconcile legacy
holdings. Group allocated bytes by checkout, owner, and class. Deduplicate hard-linked inodes
and identify held, releasable, and unknown bytes with the reason and observation coverage.
Use existing worktree and native observations where they identify the owner. Keep a small
reproducible census until a recurring operator workflow warrants a command.

Repeat the same test, worktree, and preparation journeys after settlement. The second run may
add a compact result and update useful build state. It should add zero retained executable
copies and zero physical database copies for ordinary tests. Track how many Cargo units the
selected scenarios actually compile, how much active state remains after the last borrower,
and whether retired campaign and checkout state returns to its declared baseline. Report
long-lived authored and scientific data separately from growth attributable to developer work.

<a id="O02"></a>
### O02 — Separate executable custody from an invocation's original report

Keep invocation-local inventory, configuration, provenance, and result identities. Use the
existing immutable tool installation for tools such as Nextest when its ownership and
authentication cover the actual use. Prefer an authenticated active borrow of the exact built
image where the capture owner can protect it across enumeration, launch, and settlement.
When a private copy is required by the current contract, make one for the active workflow and
release it after its final borrower drains. Preserve the opened-original check, byte commitment,
mode, and source-to-image association. Evaluate immutable digest sharing only for remaining cases
where independent active borrowers require stable copies of the same bytes.

Prefer a protected immutable backing file to hard-linking any mutable Cargo source. Same-device
aliases are viable only if publication, mutation detection, mode, verification, and per-path
execution and loader behavior preserve the original contracts. The local ext4 probe
(`cp --reflink=always` on the measured filesystem) returned unsupported, so reflinks are a
conditional optimization elsewhere. A cross-device export can use the existing verified streaming
transfer. Compare build once and run many within one admitted source, feature, native-provider,
and toolchain context. Consider Nextest archives for selected transfer or replay needs after
verifying pinned support and coverage of runtime resources. Routine local re-execution can use the
existing native listing and result path without writing a second archive.

Keep the original report under its native owner while the operation and its selected acquisitions
settle. Publish useful structured facts through O09 and release the routine local report and
executables under their declared policies. Retain exact raw bytes only for a named requirement
such as reproducing a specific corruption, with an owner, size bound, and release condition.
Plan66f exact failed rerun resolves original identities against the current listing and produces a
new result. This is the default historical rerun path. Show which operations are available after
Cargo cleaning and historical payload retirement.
Test equal-byte concurrent capture, mutation, alias and loader behavior, partial publication,
interrupted or failed export, and both read and rerun after target removal. Keep capture and
evidence authority at the existing owners under DP-04, DP-09, and DP-19.
[Capture owner](../../crates/pse-host/src/discovery/capture.rs) ·
[Plan66e](../plans/66e-protected-data-and-result-delivery.md) ·
[Plan66f](../plans/66f-developer-operations-clients-and-evidence.md) ·
[Nextest archives](https://nexte.st/docs/ci-features/archiving/)

<a id="O03"></a>
### O03 — Retire worktree state and scoped stores through their owners

Record checkout identity, canonical root, Git common directory, and state layout at creation.
List registered, moved or missing, and unresolved roots with their retained holdings and owner
guards. Retain Git's worktree semantics and assess an R-56 operation that coordinates target,
external checkout state, and recovery, against explicit release after ordinary Git removal.
Prefer a single retirement interaction when it can authenticate current sessions, generations,
descendants, fixtures, artifacts, and holds. Use positive ownership evidence for historical
reconciliation. A missing checkout or terminal result by itself supplies too little evidence.
Consistent worktree placement helps discovery, while recorded ownership is what makes retirement
safe. Cover path reuse, an unavailable mount, live borrowers, and interruption between Git and
state retirement.

Give named stores and scratch a producer-declared purpose, owner, scope, and release path. At
plan or campaign close, transfer required evidence into a verified compact or exact retained
artifact, or keep an explicit owner hold with a purpose and release condition. Store evidence by
stable invocation or artifact reference. Use session scratch for recreatable intermediates and
promote selected evidence before scratch settlement. A plan name in a directory or an absolute
path in prose is a discovery clue. Release stays with the native owner. This covers abandoned
test worktrees, diagnostic venvs, probe directories, rustc ICE output, and ad hoc `/tmp` runs.
[Checkout state](../../crates/pse-host/src/paths.rs) · [Worktree guide](../dev/agent-environment.md#worktrees) ·
[ADR register R-56](../adr/register.md)

<a id="O04"></a>
### O04 — Make retained lifetimes effective

Use three distinct outcomes in the owner workflow. Recreate or retire disposable material after
positive settlement. Keep selected durable evidence with an explicit hold and disposition.
Preserve unknown, authored, scientific, and unclassified legacy material pending owner review.
Start with preview and explicit release of producer-declared objects. Use an age/count/byte
policy only for classes with a proven ownership and eviction contract. Apply a budget first to
large recreatable payloads and failed staging so it changes growth, then consider presentation
logs. Report reclaimable bytes, protected bytes, and the reason a budget cannot yet be met.

Use the existing distinction in `RetentionClass::eligible` between terminal outcome and actual
settlement. Host startup can report candidates cheaply. Automatic deletion on host start or
invocation settle would need a narrow, explicit policy decision, concurrency protection, and
bounded work. Owner-native removal of a private finished scratch directory is a different
decision from collection of immutable published generations. Validate retryable interrupted
release, a concurrent hold or reader, missing and partially exported evidence, and an unknown
settlement. Release may only reclaim an object whose original identity and current guards still
match. [ADR-0168](../adr/0168-workspace-content-lifecycle.md) ·
[ADR-0234](../adr/0234-managed-invocation-subtree-ownership.md)

<a id="O05"></a>
### O05 — Use SurrealKit suites and Rust test constructs

Adopt the already pinned standalone SurrealKit 1.0.0-beta.6 for schema, SurrealQL, permissions,
and database HTTP API tests. Extend the existing `production_generated_contract` suite rather
than author a second schema. Feed the canonical generator's definitions into selected schema
modules. Declare small synthetic `.surql` seeds and per-suite fixture statements, including
negative and cross-actor cases. Use `sql_expect`, `permissions_matrix`, `schema_metadata`,
`schema_behavior`, and `api_request` where each directly exercises the contract. Filter by suite,
case, or tag, and project selected facts from `--json-out` into O09. Run ordinary schema suites
from sync. Run selected migration suites with `--schema-from both` and rollout parity once
there is an authored rollout chain. The rollout check starts from an empty logical database and
compares the resulting definitions with sync. Keep setup, seed, rollout, and schema authorities
explicit. [SurrealKit pin](../../.config/surrealkit.toml) ·
[Existing generated-contract suite](../../scripts/tests/fixtures/surrealkit/canonical.toml) ·
[SurrealKit testing](https://surrealdb.com/docs/manage/schema-migration/testing)

**Interface-checked.** Context7's official SurrealKit entry confirmed suite filters, reusable
actors, JSON reports, and the sync/rollout distinction. The installed beta.6 help and pinned
runner source establish the exact logical database scope, setup order, and parity behavior.
Use a test-owned endpoint and a dedicated test namespace with namespace-scoped setup credentials.
Root mode can allocate separate namespaces and belongs only to a disposable test-owned server.
One suite run creates logical
databases from definitions and seed rows on that endpoint. Group cases that share data and
separate cases that need independent state. Use normal cleanup with `--keep-db` reserved for an
explicit investigation with a named release. The pinned runner can warn on cleanup errors,
discards statement-level cleanup checks, and can leave state on setup failure or interruption.
Require owner readback of exact suite database absence or settlement of its disposable endpoint.
Exercise passing, failing, setup-failed, and interrupted suites. Never infer release from the
suite verdict or send test provisioning to the configured durable catalog.
[Pinned suite runner](https://github.com/surrealdb/surrealkit/blob/92ebc3cc89b99f00d21c7d2cbf5136d728367061/crates/surrealkit/src/tester/runner.rs) ·
[Context7 SurrealKit documentation](https://context7.com/surrealdb/surrealkit)

Keep application behavior in ordinary Rust unit tests or consolidated integration targets.
Select cases through Nextest, and pass authored runtime inputs for different actors, seeds,
faults, and fixture roles. Audit `canonical-tests`, `test-support`, and the fixture helper
feature compositions by caller. Move database-only assertions to SurrealKit. Move remaining
test selection out of feature flags where Rust test targets, dev-dependencies, or runtime
parameters express it. Preserve actual product/native linkage differences and explicit Arrow
force validation. Retire superseded tests and capture paths only once their replacement proves
the same behavior. The observed runtime captures came from an `xtask` builder and native lane,
so schema migration alone will leave that cost. Move their scientific checks into normal
test-owned entry points or borrow the existing prepared product only where the test genuinely
exercises that executable or provider boundary. [Current fixture declarations](../../crates/pse-operations/Cargo.toml) ·
[Runtime fixture owner at the review baseline](https://github.com/paul-heyse/pse-arrow/blob/c2126df452ea22f8d51ed32b2429565baaf51017/crates/pse-host/src/fixtures/runtime.rs)

Run restart, durability, migration from an on-disk predecessor, backup/restore, protocol,
and process lifecycle controls in the native owner where they need real service behavior.
Create the smallest synthetic state for each case and settle it with its owner. Ordinary
schema and application tests can use isolated logical databases, including on a test-owned
in-memory service when the required contracts fit. Physical copies belong to selected
backup/restore tests. Verify the same named assertions, one build per compatible test context,
zero persistent scratch database copies, and release after success and failure.
[Canonical source profile](../../scripts/surrealdb-source/source-build.md) ·
[Application recovery](../dev/surreal-substrate.md)

<a id="O06"></a>
### O06 — Release prepared and service generations without discarding warm inputs

Inventory the actual selection and borrow graph for host tools, native providers, Surreal builds
and modules, and deployed worker generations. Classify staging, selected, rollback, imported,
failed, and unreferenced material. Prefer explicit bounded retirement per owner. An optional
previous generation earns its place when a consumer has a concrete rollback path. Keep live
and held generations and reconcile pending or partially published ones.

Current native Surreal publication already emits a self-contained selected generation. A
representative legacy 2.3 GiB `source/` directory was mostly `source/target`; about 70 MiB was
source. Legacy release receipts can point into that target and the import route needs the binary
and source lockfile. Establish independent selection and readiness and resolve direct-path
borrowers before releasing old intermediates. Retain useful warm recipe state under its own
budget. Apply the same distinction to module and native-provider preparation. In the named
Plan49 example, the worker-generation owner has a much larger opportunity than DB compaction.
[Server source and import contract](../../scripts/surrealdb-source/source-build.md) ·
[Server owner](../../crates/pse-host/src/server.rs)

**Measured.** A read-only `find` counted 554 Surreal, eight compiler-cache, and four telemetry
service fragments under the user's systemd directory. Modern native fixtures already use
`StartTransientUnit` with `CollectMode=inactive-or-failed` and dispose owned state after confirmed
drain. Classify old fragments against current unit, process, fixture, and selection records.
Retire proven unused fragments through their owners. Optional telemetry has a persistent-service
contract and its own stop and release path. This improves operator visibility and teardown
reliability. The file count is operational clutter with little direct byte impact.
[Manager owner](../../crates/pse-host/src/manager.rs) ·
[Canonical service owner](../../crates/pse-operations/src/native_service.rs) ·
[Telemetry service owner](../../crates/pse-host/src/telemetry_service.rs)

<a id="O07"></a>
### O07 — Reduce the number of Cargo build variants

Make product capability differences the basis of the feature matrix. Use O05's test constructs
for variations in data, actors, scenarios, and faults. Count effective Cargo compilation units
by package, role, target, feature set, profile, and wrapper context before and after removing
test-driven combinations. Keep a named compatibility matrix for genuine product differences.
This reduces the number of variants Cargo writes and increases the repeatability of requests
that reach sccache.

Keep per-checkout targets and interactive incremental compilation. Qualify non-incremental
verification for repeated check and test work using one stable configuration where eligible
libraries can hit sccache. `features-combinations` already uses
`--config build.incremental=false`. Assess the no-default lane and selected verification and
coverage paths separately. Prefer the Cargo config option when it avoids adding an exported
`CARGO_INCREMENTAL` key. Audit accidental explicit host `--target`, different build-dependency
profile overrides, compiler wrapper composition, and unnecessary alternate build roots.
Those choices can partition otherwise compatible units. Preserve required WASI and other target
boundaries, native providers, build-script inputs, force validation, and coverage.

For the remaining high-variance campaigns, compare stable checkout-local campaign ownership
with Cargo's split `build.build-dir`. Use a separate intermediate directory only when it makes
release or repeated work cheaper. Moving it can change `OUT_DIR` and break reuse. Release retired
campaign state as an owned unit after settlement. Keep ordinary target freshness and avoid
eviction rules based on Cargo's internal directory names.

Use the pinned cargo-shear's JSON findings and `--check-test-targets` to identify empty or
duplicated harnesses, unused declarations, and missing test or doctest selection. Check real
feature, macro, and dev-target callers before applying fixes. Use `--expand` for specific
macro-dependent cases. Compare compiled unit counts to distinguish manifest cleanup from
fewer builds. The current pinned Cargo workspace unification is the baseline. Hakari's
iterative host/target feature unions and exclusions would add a generated dependency graph
and could activate additional work. Use a lockfile-based graph census for any residual
fragmentation before reconsidering Hakari. [Current recipes](../../justfile) ·
[Build performance](../dev/build-performance.md) ·
[Prior build reuse review](rust-build-reuse-tooling-2026-10-24.md) ·
[Cargo build cache](https://doc.rust-lang.org/cargo/reference/build-cache.html)

<a id="O08"></a>
### O08 — Make sccache 0.19.0 coverage observable and improve it

The managed binary verified against the pinned checksum. After the maintainer's installation,
the PATH binary also reported 0.19.0. The two entry points have independent selection and cache
ownership. Local managed routes select the real executable, client-side mode, a dedicated Unix
socket, and the configured cache frame. Inspect effective configuration and bypass reasons for
ordinary Cargo, native correctness, direnv, prepared tool builds, and explicit cache-off or
compiler-evidence routes. Reconcile the build guide's historical service and statistics
description with the current direct executable path. Plan54's Python launcher was retired in
`8af5ea110`; its old daemon supervision should be independently verified before relying on it.
Use one writer daemon per local cache directory. [Managed environment](../../crates/pse-host/src/environment.rs) ·
[Tool declaration](../../.config/diagnostic-tools.toml) ·
[sccache local storage](https://github.com/mozilla/sccache/blob/v0.19.0/docs/Local.md)

**Tested.** A five-request probe ran the checksum-verified managed executable and pinned rustc
with a private 32 MiB cache, Unix socket, and client-side mode. It compiled a tiny non-incremental
rlib with an `env!` input. The cold compile missed. After moving its output away, same-path
restoration hit. A different output directory in the same working directory also hit and
produced the same rlib digest. An identical source at another working directory missed.
Changing the embedded value missed and matched direct rustc output byte for byte. Every depfile
named its current source. There were two hits, three misses, three writes, zero cache errors,
and `compile_requests` stayed zero. The probe covers raw rustc reuse and changed-input safety
for this small case. Full-workspace latency, large-entry restoration on today's route, and
campaign performance remain unmeasured here.

**Tested.** A second private-cache probe used managed client-side 0.19.0, pinned rustc,
and GCC. With `SCCACHE_DIRECT=false`, two equivalent metadata-only Rust library invocations
(`--emit=dep-info,metadata`) and two equivalent C compiles each produced one miss and one hit.
The restored `.rmeta` and `.o` matched their first output byte for byte, both depfiles existed,
and the daemon reported zero cache errors. The tagged Rust implementation accepts eligible
metadata output even though the tagged Rust guide says link output is required. This makes
non-incremental `cargo check` library reuse a direct opportunity. The probe is smaller than
an actual Cargo graph and provides a candidate for measuring real check latency.
[Tagged Rust implementation](https://github.com/mozilla/sccache/blob/v0.19.0/src/compiler/rust.rs)

The material capabilities in the caching reference are distinct. Some increase which work
can be reused, some improve the cost of a hit, and some protect feedback time on a miss.

| Capability | Where it can pay off and what to qualify |
|---|---|
| Local Rust dep-info in 0.19 | Preserve invalidation on an eligible restore with an equivalent relocated output or checkout. Use local source paths in the restored build and test a subsequent included-source edit. Cache identity still includes literal environment, cwd, and extern bytes. |
| Early `.rmeta` notifications and concurrent output discovery in 0.19 | Already available on the managed client-side route. Compare Cargo `--timings` on a representative cold check and linked test build. Look for dependent compilations starting sooner on misses and lower lookup preparation overhead on warm restoration. Preserve metadata notifications through the correctness adapter. |
| Eligible library metadata and non-incremental library codegen | Increase the fraction of repeated check and test work that reaches the cache through O07. Compare warm Cargo freshness, an ordinary edit loop, and recovery with absent outputs. Measure eligible units, restored bytes, and wall time. Use thin test or command entry points only where expensive reusable code currently lives in linker-bound targets. |
| Large client-side entries and reuse after a clean | Verify actual writes and restores for heavy outputs such as Symbolica, DataFusion, and the relevant native closure using the configured 256 MiB IPC frame. Keep the same build identity and use a scratch output comparison with a known key. Compare compression or frame choices only when measured write, restore, or large-entry failures justify them. |
| GCC/Clang launchers and direct preprocessor cache | Extend effective wrapping to costly repeated C/C++ paths that currently miss it. Inspect CMake launchers and KLU first, then actual `cc` and external native build invocations. Ordinary C/C++ output caching works with the current direct mode off. Consider enabling the direct shortcut only for a sealed header tree after changed, newly introduced, and shadowing header controls. The 0.19 include-validation fix makes that experiment worthwhile. |
| Multi-level raw-hit promotion and optional tier startup in 0.19 | Run a focused disk L0 plus one optional remote tier experiment for a warm-cache writer and a compatible clean consumer. Use a representative CI or second build machine with matching key inputs, or test restart recovery with a private tier. Check deeper hits, L0 backfill, the subsequent local hit, backend outage at startup and mid-run, and per-tier read/error timing. Prefer one tier with a measured payoff. |
| Read-only tiers and failure separation | Give trusted producers writes and consumers read-only access where sharing fits. For a disk-first trial, start with `l0` policy, keep capability checks enabled, and compare hit, absence, authentication failure, write failure, and local I/O fallback as separate outcomes. Qualify `SCCACHE_IGNORE_SERVER_IO_ERROR=1` for ordinary local recovery against the managed client-side route. Explicit cache experiments should expose bypass. |
| Structured statistics in 0.19 | Use `--show-adv-stats` and `--show-stats --stats-format=json` deltas from the selected daemon. Combine effective access mode, non-cacheable Rust crate types, C/C++ flags, per-tier raw hits, errors, and actual writes with Cargo timings. Use preprocessor diagnostics for direct-mode trials and `SCCACHE_RECACHE` for isolated corruption controls. |

The tagged release already gives the first two Rust improvements where the compiler actually
passes through the managed wrapper. The immediate expansion opportunities are the stable
non-incremental check/test lane, reliable publication of large entries, and missing native
compiler launchers. The 0.19 raw-hit backfill and outage behavior make tiering worth a concrete
trial for compatible clean consumers. That trial depends on demonstrating compatible keys
first. Rust `SCCACHE_BASEDIRS` landed after 0.19.0. Qualify a released successor on literal
`env!`, `file!`, generated input, proc-macro, remap, and changed-extern controls before using
cross-checkout normalization. Keep independently verified current build and test evidence.
[Release notes](https://github.com/mozilla/sccache/releases/tag/v0.19.0) ·
[Tagged Rust implementation](https://github.com/mozilla/sccache/blob/v0.19.0/src/compiler/rust.rs) ·
[Post-release basedirs work](https://github.com/mozilla/sccache/pull/2918) ·
[Correctness follow-up](https://github.com/mozilla/sccache/pull/2794) ·
[sccache multi-level cache](https://github.com/mozilla/sccache/blob/v0.19.0/docs/MultiLevel.md)

**Tested.** A private loopback disk/WebDAV probe used managed client-side 0.19.0 and GCC.
A writer populated both tiers. A new consumer with an empty disk tier and read-only WebDAV
restored the same bytes from L1, recorded one L0 backfill, then hit L0 on the next compile.
The remote object count stayed at one. A separate loopback connection-refusal control started
successfully under `l0` and still served one local hit. Under `all`, startup refused as expected.
These controls establish raw promotion, read-only reuse, per-tier reporting, and the startup
policy on this binary. Real backend transfer cost and compatible cross-machine reuse remain
to be measured.

Measure representative heavy same-checkout recovery and stable campaign reuse next. Separate
Cargo freshness, sccache eligibility, lookup/restoration, linked or incremental work, and
critical-path wall time. Reuse the earlier 37,764,932-byte publication and linked restoration
controls as historical evidence for the configured 256 MiB frame, then validate representative
large outputs on the current launch path. Rust key versions changed from 6 in 0.18 to 7 in
0.19. Allow native disk eviction of old unreachable entries and assess cold rewarming before
considering a targeted stale-generation release. Avoid blanket age purges of either cache.
Report effective size, occupied bytes, evidence of useful hits, and current owner before any
capacity adjustment. [Historical 0.19 evidence](../design_review/evidence/rust-build-reuse-019/README.md)

Choose the optional backend by the actual environment. S3-compatible storage or the existing CI
cache are candidates for cross-machine reuse. Redis and Memcached need a low-latency deployment
and entry expiry that fits artifact size. Azure identity, S3 encryption and key prefixes, and
WebDAV's new directory-creation control matter when the matching backend is selected. Keep one
writer daemon per local directory and one authoritative configuration per trial. Scope remote
storage to trusted artifact producers. Apple multi-arch preprocessing, clang-cl C++20 modules,
MSVC flag fixes, CUDA/HIP, and 0.19 distributed toolchain packaging have no demonstrated
consumer in the reviewed Linux workflows. Distributed execution can be revisited for sustained
uncacheable misses on multiple builders after resolving its scheduler and supervision
requirements. [sccache configuration](https://github.com/mozilla/sccache/blob/v0.19.0/docs/Configuration.md)

<a id="O09"></a>
### O09 — Make SurrealDB the structured history for development

Start with a dedicated developer-evidence database within the existing local SurrealDB instance,
with scoped credentials, an independent schema, a measurable write and retention budget, and
explicit ownership of startup, maintenance, and release. This offers shared query and retention
semantics across worktrees. Compare a separately managed developer instance if a shared process
cannot isolate credentials, working set, storage lifecycle, or recovery from canonical scientific
work. The native test and investigation owners remain responsible for current execution and its
original verdict. The database is the historical query surface for the facts they deliberately
publish. Scientific results and large protected payloads retain their existing application
owners. Check the existing diagnostic invocation and event projections before declaring a new
schema, so the same fact has one historical home. [Blueprint §24](../authoritative_design/sections/operations-and-validation.md) ·
[Canonical ownership and recovery](../dev/surreal-substrate.md)

| Consumer and use | Minimum retained information | Lifecycle |
|---|---|---|
| Developer finding and rerunning a failure | Invocation ID, revision and checkout identity, selected test IDs and compatible build context, original outcome and coverage, bounded failure details, and a source or diagnostic reference when useful | A rolling window for ordinary results, with a longer finite window for failure details. Rerun against the current admitted inventory using original identities. |
| Maintainer comparing regressions or qualification runs | Named campaign, exact source/toolchain/provider context, selected assertions or timing measures, units and conditions, and original outcomes and omissions | Retain a selected comparison or plan cohort through its documented use. Review the hold at closeout and expire it or deliberately renew it. |
| Operator tracing a fixture, store, or worktree | Owner and object identity, purpose, bounded status and timestamps, current settlement or release reference, and holds | Keep until release is reconciled. Keep a small bounded completion record where it supports later diagnosis. Physical release authority stays with the original owner. |

Default historical retention to bounded summaries and actionable excerpts. Release full log
streams, executable copies, and fixture databases after active use and settlement. Represent
an exceptional raw artifact by a verified reference, digest, size, custodian, and expiry, with
the bytes under the existing file owner. Keep bounded native local spooling only when needed
for current settlement or retry of a selected publication. An unavailable history database
must leave the original operation's result truthful and expose incomplete historical capture.

Use typed `SCHEMAFULL` records and stable IDs derived from the native invocation and selected
result identities. Publish related rows transactionally, idempotently on retry, and settle an
unknown acknowledgment against the original request identity. Index the actual access patterns
first, such as revision and failing test, campaign and time, or owner and expiry. Set bounds
on detail size, query windows, and per-class retained bytes or rows. Start with a small pilot
population and size routine, failure, and held windows from actual query needs and physical
growth. Prefer owner-initiated, bounded batches of expiry-indexed records with guard rechecks,
an explainable preview, and separate records of logical deletion and physical space recovered.
Default to current rows only. Changefeeds, historical versions, and extra indexes earn a place
through a specific consumer and budget. [SurrealQL schema](https://surrealdb.com/docs/reference/query-language/statements/define/table) ·
[Transactions](https://surrealdb.com/docs/reference/query-language/language-primitives/transactions) ·
[SurrealQL deletion](https://surrealdb.com/docs/reference/query-language/statements/delete)

Give the instance an explicit development lifecycle. Provision and seed from declared inputs,
observe per-database growth alongside shared process and filesystem use, expire eligible rows,
and verify physical footprint across checkpoint and journal cycles. A successful row deletion
establishes logical release. Measure actual disk and memory recovery on the selected patched
MX profile, including a repeated ingest/expire cycle and a restart. Export selected durable
cohorts in a compact form when they have an independent recovery need. Test restore of that
selection and keep backup retention separate from live row retention. Use the existing
application recovery path for an exact canonical backup. Verify the physical reclamation path
before relying on row expiry to keep the database footprint stable.

Pilot one test workflow. Query recent failures and compare two revisions after releasing local
executables, rerun a selected failure against the current inventory, and retire a closed
campaign while preserving a deliberate hold. Inject retry, unknown commit, unavailable history,
expired raw reference, and interrupted physical release. Backfill only named historical evidence
from readable original reports with explicit omissions, then preview release of independently
settled legacy payloads. This proves the operator can get useful history without accumulating
another copy of every run.

## November 5 dispositions and order

| Original recommendation | Review disposition |
|---|---|
| 1. Checkout removal and orphan GC | Advance explicit identity, preview, and R-56 retirement. Keep ambiguous legacy roots protected. [O03](#O03) |
| 2. Stop copying binaries | Advance immutable reuse for active work and release historical executable bytes by default. Publish selected structured facts. [O02](#O02), [O09](#O09) |
| 3. Turn retention eligibility into collection | Advance owner-scoped release and budget visibility. Require a decision for automatic collection of published data. [O04](#O04) |
| 4. Pin cited evidence | Advance holds for selected structured evidence and exceptional raw evidence under Plan66e/f. [O04](#O04), [O09](#O09) |
| 5. Release plan stores | Advance declared ownership, selective historical publication, native stop/export/restore where applicable, and closeout release. [O03](#O03), [O05](#O05), [O09](#O09) |
| 6. Route and clean scratch | Advance session ownership and explicit promotion of evidence. Preserve failed or interrupted scratch where its owner cannot settle. [O03](#O03) |
| 7. Bound tool/native generations | Advance owner-specific reference and rollback rules. Release legacy build intermediates after reconciling actual paths and consumers. [O06](#O06) |
| 8. Remove service fragments | Advance reconciliation of legacy fragments and their state. Use the existing transient native fixture path and the persistent telemetry owner's release. [O06](#O06) |
| 9. Bound build directories | Remove test-driven build variants first. Use SurrealKit suites and Rust test targets. Keep scoped build state for real compatibility campaigns and measure non-incremental verification. [O05](#O05), [O07](#O07) |
| 10. Reduce sccache to about 40 GiB | Defer resizing. Keep the 100 GiB managed baseline and improve reuse and visibility. Revisit sizing from workload evidence. [O08](#O08) |
| 11. Report footprint and guard admission | Keep read-only footprint measurement for diagnosis and acceptance. The maintainer declined free-space warnings, thresholds, reservations, and admission gates in favor of reducing writes at their owners. [O01](#O01) |

First migrate database-contract checks to the pinned SurrealKit suite and express application
scenarios as Rust tests and runtime inputs. Remove the superseded fixture feature combinations
and repeated fixture captures after checking coverage. In parallel, end routine per-run
executable retention and pilot targeted SurrealDB history so failure queries and reruns survive
release. Retire campaign, checkout, store, and generation state through the owners. Measure
repeated workflows with O01 to verify that the new paths stop accumulating retained binaries,
database copies, and unnecessary target variants.

Advance the 0.19 non-incremental verification, large-entry, and native launcher measurements
alongside those changes. Follow with a two-tier trial for a compatible clean consumer.
Carry the opportunity IDs and Plan66 integration obligations into implementation planning.
Repository-wide speedup, physical SurrealDB reclamation, and realized savings remain to be
measured.
