---
title: "Plan 54c: Cross-location reuse and investigations"
status: complete
date: 2026-10-24
adrs: []
review_sources: [docs/design_review/reviews/design_review_rust-build-reuse_2026-10-24.md, docs/functional_reviews/rust-build-reuse-tooling-2026-10-24.md]
doc_role: plan
doc_owner: docs/plans/54-rust-build-reuse.md
doc_topics: [current-work, design, environment, libraries, performance]
---

# Plan 54c: Cross-location reuse and investigations

## Target and foundation

**Proposed:** decide how much safe additional reuse the current host can obtain after
removing accidental differences, without replacing correct misses with unsound hits or
adding machinery unsupported by useful workflows. The [coordinator](54-rust-build-reuse.md)
owns I01–I04, source dispositions and scenarios S07/S08. This companion owns the bounded
investigation packages and conditional adoption. No optimization is mandatory merely
because a library exposes it.

The [library reference](../libraries/rust_caching.md) supplies capability awareness across
hakari, sccache and shear. Context7 followed by exact source settles consequential details.
Do not interpret the reference's general basedirs claims as demonstrated Rust relocation
support, or its upstream performance observations as repository measurements.

## C01 — Establish the supported boundary of Rust relocation

Consume A01/A02's executable/mode and D01's environment-neutral snapshot targets. Reproduce
the original issue with the actual selected build and account separately for generated
OUT_DIR roots, downstream --extern changes, vendored/path dependencies and other relevant
key differences. The recording shim is an attribution aid, not an exact independent Rust
cache-key implementation; its omission of source/extern content and dep-info env inputs
must remain explicit. Preserve actual compiler outputs and behavioral controls.

**Interface-checked:** upstream PR2794 is open at
91dd36202fdb60f1feac5e550500f4abb76a8e09. Its
[Rust environment hashing](https://github.com/pamarcos/sccache/blob/91dd36202fdb60f1feac5e550500f4abb76a8e09/src/compiler/rust.rs#L1907-L1969)
deliberately keeps env!/option_env! values location-sensitive. An OUT_DIR value can appear
as a literal string in output; compiler path remapping is not arbitrary string rewriting.
Therefore this PR plus remapping is not a demonstrated remedy for F01. PR2678's broader
normalization is a candidate requiring independent safety examination, not a preferred fix.

Evaluate actual released or pinned candidate behavior, including source identity, cwd,
generated filenames, selected CARGO_* paths, extern/dynamic dependency constraints and
dep-info restoration. A root remap covering a checkout need not cover the cwd of a
registry dependency. Identical relative targets in two worktrees and differently named
targets inside one checkout are distinct cases. Consider a stable virtual target mapping
only if the candidate preserves all observable source/environment semantics.

Basedirs policy is also a service-lifetime decision. Client-side v0.19 gets it from
the daemon's
[StorageHandshake](https://github.com/mozilla/sccache/blob/v0.19.0/src/server.rs)
through [IpcStorage](https://github.com/mozilla/sccache/blob/v0.19.0/src/cache/ipc_storage.rs).
A new per-worktree environment value does not necessarily change a running daemon's
normalization policy. An adoption proposal must specify the root policy, newly created
worktree behavior, config generation and caller/daemon agreement. Prefer declared supported
roots or an owned configuration generation over restarting shared services for each worktree.
Account for installation_context and Python producer admission of any newly consumed controls.

Run only probe-owned foreground endpoints and caches for candidate experiments. Populate
the cache under the candidate flags/policy in location A; then read it from B. Reusing the
old pre-remapping cache is not a valid expected-hit test after flags/key versions change.
Keep the managed developer service untouched. Source-only comparison suffices to reject
an unsafe candidate; compiling every fork is not an investigation requirement.

**Acceptance, proposed:** compare the same relative target layout across two snapshots,
different target layouts, and same-path artifact loss. Include an include!-generated fixture,
a crate embedding OUT_DIR or CARGO_MANIFEST_DIR as a literal, file!/Location-sensitive
behavior and a generated-content change. Correct restored behavior and required misses
matter alongside attributed hits. Identify which actual heavy dependencies benefit, which
still miss and why. Check changed extern inputs and restored dep-info freshness. Native and
Python consumers remain separate until actually exercised. Unsafe reuse fails regardless
of hit rate.

Close I01 with a precise supported proposal, a limited opportunity or a no-adoption decision.
If no safe available mechanism covers F01, retain correct misses and document the external
limit. A deferred trigger must name the capability required and rerun controls, not merely
"a release containing PR2794". Repeated campaigns may reuse their own deliberate stable
source/target location only where source isolation and lifecycle remain valid; never share
one mutable Cargo target across independent checkouts.

## C02 — Conditional adoption of a qualified relocation mechanism

C02 starts only after C01 identifies a safe candidate and the coordinator records its exact
supported scope, selected implementation and any necessary maintainer rule decision. Do not
put an unreviewed fork or a key-normalization shim on the ordinary compiler path by default.
No adoption is preferable to unsound or unmaintainable reuse.

An eligible implementation belongs in the existing environment/cache/config owners, using
library-owned Rust normalization plus effective compiler remapping where required. Preserve
all remaining meaningful inputs. Migrate every affected caller, including direct recipes,
worktrees, measurement snapshots, rust-analyzer and maturin; a recipe-only setting cannot
establish a global contract. Retain mold/explicit flag precedence, validation and profile
separation. Avoid hard-coded home paths. Root discovery belongs at the existing environment
boundary, with explicit treatment of direct Cargo consumers that do not use it.

Account for changed panic/debug/source paths, file!/Location tests, snapshots, debugger and
rust-analyzer source mapping. New rustflags/key versions can require one-time fresh entries;
do not delete the old shared cache or claim compatible-key reuse without evidence. Extend
controlled installer/producer input handling only for the selected mechanism and keep
unsupported config/environment controls fail-closed.

**Acceptance, proposed:** rerun C01's discriminating controls and the affected actual Rust/
Python journeys using the selected production route; demonstrate invalidation and useful
reuse with compatible cache population. Record current support limits in D03. If C01 cannot
select a mechanism, mark C02 explicitly not adopted or deferred with the owning trigger;
this closes the conditional package but does not falsely resolve F01.

## C03 — Cache capacity, lookup cost and failure attribution

After A03/D01, distinguish eligible hits/misses from non-cacheable calls and failures. Use
the existing build-storage, cache-stat and measurement owners for a representative selected
worktree/campaign sequence. Preserve before/after statistics or use a probe-owned endpoint;
do not zero the shared service's statistics. Unknown-window lifetime percentages and similar
fresh-target percentages do not prove historical miss causes.

Inspect actual cache growth/eviction, frequently reused artifacts, lookup/restoration cost,
storage traffic and expensive misses. If the reported cache errors recur, use the isolated
diagnostic server route to attribute them without silently changing ordinary execution mode.
Do not require a permanent telemetry pipeline or a new budget controller merely to investigate.

Compare the current 100 GiB budget and default compression against a bounded alternative
only where observations identify material pressure or CPU/I/O cost. sccache exposes
SCCACHE_CACHE_ZSTD_LEVEL and disk capacity/read/write controls; integrate a selected setting
through the existing declared cache configuration and service-generation owner. Preserve
private storage, free-space policy and valid artifact semantics. A larger cache can reduce
eviction while consuming storage; a compressed hit can still cost more than compiling a
small unit. Compare complete workflows rather than maximizing size or compression.

**Acceptance, proposed:** close I02 with attributed observations and a selected configuration
or justified unchanged default. Any selected ordinary local setting must be implemented and
compared through D02 before being called beneficial; unexecuted alternatives stay Proposed.
Keep cold/warm, cache storage versus Cargo target storage, native preparation and competing
host load distinguishable. No automated eviction of unrelated caches is authorized.

## C04 — Find expensive work outside ordinary eligible hits

Use actual unit roles, features, profiles and complete timing/critical-path evidence to
investigate I03. The supplied census compares sets and counts; equal common feature sets
alone do not prove equivalent unit identity. Inspect concrete differing units before
describing a duplicate. Check build/host and target roles, check-versus-build mode,
validation/native opt-ins, producer/release profiles, proc macros, build-script execution,
linking, and registry versus vendored incremental treatment.

Keep current workspace incremental compilation and native workspace feature unification
unless a selected edit/build sequence supports a better realization. Hakari can expose
host/target strategies, platforms and exclusions (reference §2.4), but its added feature
activation, generated manifest, wider small closures and publication lifecycle are real
costs. Treat the reported 45/5,822 duration fraction as a lead; it is neither measured
hakari performance nor a wall-time savings bound. If reconsidered, compare actual scoped
graph/critical-path outcomes and all preserved opt-in semantics before proposing an ADR.

Profile expensive non-cacheable binary-owned work, especially xtask's actual tool features
and worker modules. Prefer smaller appropriate selected closures first. Where substantial
reusable implementation is truly binary-owned, compare a thin executable over an existing
library target against current incremental/local artifact reuse. A source file's size alone
does not establish a worthwhile extraction. Do not add a crate, expose test-only APIs,
collapse meaningful profiles or move dependencies merely to improve a cache statistic.

**Acceptance, proposed:** close I03 with concrete expensive causes, affected consumers and
comparisons that can change the decision. Select bounded within-contract changes only when
their mechanism and acceptance are settled; record them here before execution and include
them in D02's combined footprint. A rule/architecture-changing candidate remains a proposal
until explicit decision and the existing ADR/review route. A no-change result can complete
the inquiry, while the existing workflow remains supported.

## C05 — Evaluate wider capabilities for the selected host

Close I04 by assessing the library reference's multilevel reads/promotion/write-error policy,
remote storage, reusable guppy/hakari graph analysis, shear structured/library interfaces,
and distributed cache-miss execution where they could improve this host's actual workflows.
Existing CLI/JSON integration remains sufficient unless an actual consumer benefits from
embedding library APIs. Lack of a current caller does not prove a capability useless.

For storage tiers, compare local latency/capacity/recovery with extra services, transfer,
authentication, executable backend features, write-error policies and operating ownership.
For distributed execution, compare miss critical path with scheduling/toolchain packaging,
trust, native environment and its incompatibility with the selected client-side supervision.
Remote caching and remote execution are distinct mechanisms. No service deployment or new CI
platform is included by evaluating them. A useful deployment proposal needs a concrete
target/ownership decision before it becomes implementation scope; otherwise retain the
single managed disk cache with a reasoned revisit condition.

**Acceptance, proposed:** a concise technical disposition naming an actual potential benefit,
material limits and total machinery for credible alternatives. Source reasoning can settle
an unsuitable option without a benchmark. Avoid turning the library overview into a feature
checklist or inventing mandatory infrastructure to complete this package.

## Package state

| Package | Status | Owner | Completion |
|---|---|---|---|
| C01 | complete | 54c | Supported relocation scope and discriminating safety/reuse decision |
| C02 | complete (not adopted) | 54c | Qualified adoption, or explicit no-adoption/deferral preserving F01's limit |
| C03 | complete | 54c | Large-entry publication repaired and behaviorally compared; retain capacity/compression; end-to-end timing skipped by maintainer |
| C04 | complete | 54c | Compile placement corrected; current xtask/features/incremental design retained with a concrete profiling trigger |
| C05 | complete | 54c | Wider-capability disposition for the selected host without assumed deployment |

Acceptance clauses above describe the planned controls; the checkpoint below records current execution. Inquiry outcomes update their coordinator owner;
decisions requiring changed rules are not inferred from investigation authorization.

## Execution checkpoint

C01's 16 real behavioral controls passed; the [0.19 evidence](../design_review/evidence/rust-build-reuse-019/README.md)
records supported same-path/target reuse and required misses. C02 deliberately adopts no
remapping fork: released 0.19 retains literal environment/cwd inputs; PR2794 cannot remove
F01's OUT_DIR distinction and PR2678 normalizes observably meaningful strings unsafely.
F01 remains externally limited. Revisit only a released or deliberately maintained mechanism
that preserves literals, supports the actual heavy extern cases and passes these controls
under its own populated configuration, including local dep-info freshness.

C05 retains the managed disk cache and local caller-owned compilation. Remote/multilevel
storage has potential recovery/history/second-producer value, with service, latency and
credential costs; reconsider for demonstrated repeated eviction or a concrete recovery/producer
requirement. Distributed execution requires a named operator and a separate supervision/trust/
native-environment decision. Current CLI/JSON and exact Cargo units suffice for graph/shear
consumers; reconsider guppy/hakari analysis or embedding for recurring queries or an actual
long-lived Rust consumer. These are source-supported choices, not measured deployment claims.

C03 found a concrete additional defect: the default 8 MiB client-side IPC frame rejects
large cache publication and can lose its own statistics. A private large-entry control
confirmed failure and successful publication/hits with a larger bound. The maintainer
authorized raising it; selected implementation is a declared 256 MiB frame, caller override,
daemon generation identity and an 8 GiB resident-cache budget. This ceiling is per message,
not a standing allocation. Integrated managed acceptance passed: actual daemon frame/budget, miss/publication/hit,
linked full-payload correctness and a distinct override generation. The maintainer explicitly
skipped the final performance campaign because no remaining adoption decision depended on
quantifying its speedup. The decision rests on supported codec behavior, observed rejection
at the old bound and correct publication/restoration at the new bound. This establishes the
functional benefit, without an end-to-end timing claim.
Capacity stays 100 GiB and compression stays default level 3 unless subsequent evidence
shows eviction or material encoding/extraction cost; private campaign capacity was not pressured.
C04 identified a second concrete avoidable cost: direct doctest, family-check and Python
extension/stub producer entries lacked the existing compile placement hint. The live
doctest entered a four-CPU functional allocation before its inner compiler launch;
producer hints now apply before allocation, preserving caller overrides and scientific
execution ownership. Forty-one environment precedence controls passed. The corrected
expensive-work attribution does not treat lost statistics as proof that Symbolica was a hit.
Existing xtask targets, tool features and incremental policy remain selected. Its default
feature set is empty; native ABI, physical admission, product schemas, canonical/evidence/
project tools, scientific oracles and worker binaries are explicitly gated. Substantial
generation implementation already lives in libraries. Completed warm publication used
0.51/0.26/0.60-second Cargo invocation intervals; the governance full-codegen recheck took
11.3 seconds overall with a 0.31-second Cargo invocation. These show warm artifact reuse,
not extraction savings. The earlier 687-second check included artifact-lock contention and
broad dependency rebuilding; the compiler reference contains no xtask unit. The interrupted
profiling attempt and rejected timing-flag invocation establish no cost observation.

No evidence identifies binary-owned xtask work as a material remaining critical path, so
another extraction is not justified. Revisit if repeated normal tooling edits or artifact
recovery show its compilation/linking dominating a relevant workflow after dependencies
are fresh and the narrowest valid features are selected. Capture isolated complete wall/CPU
and unit timings, identify the actual hotspot and compare a focused extraction with existing
incremental reuse. Preserve full-codegen's required schema/physical/native execution scope.

## Verification and Outcome — 2026-10-24

**Interface-checked:** exact released 0.19 source and the pinned normalization candidates
support the decisions in the [source and runtime evidence](../design_review/evidence/rust-build-reuse-019/README.md).
Released Rust hashing preserves meaningful literal environment/cwd/extern inputs. PR2794
does not resolve the actual OUT_DIR/extern case; PR2678 equates observably different literal
values. No remapping fork is adopted. F01 remains externally limited under the coordinator's
capability-and-behavior trigger, rather than a promised PR-based cure.

**Tested:** 16 real relocation, generated-input, literal-path, changed-extern and local
dep-info controls passed with the selected 0.19 executable and pinned compiler. The private
large-entry control reproduced three correct compiles with lost publication/statistics at
the old frame bound; a raised bound produced one publication and subsequent hits. All 12
linked full-payload oracles passed. Actual managed-wrapper acceptance verified the declared
256 MiB frame, 8 GiB service allowance, a 37,764,932-byte publication and restored behavior,
plus a distinct override generation. Its four linked payload controls passed. The permanent
`just build-cache-probe` also passed actual artifact removal/restoration and payload checks
with an entry larger than 8 MiB. Probe-owned services and allocations were released.

**Implemented:** the frame control lives in existing configuration/environment/service
generation owners and reaches nested, analyzer, installer and measurement consumers. Direct
compiler-producing recipes select the existing compile placement before admission, retaining
explicit caller choices. Focused environment and cache controls passed; D02 records actual
installed-extension and analyzer observations. Cache keys, supported validation/profile
distinctions and caller-owned local compilation remain intact.

The managed disk cache stays at 100 GiB with default compression. Current xtask targets,
feature selection and ordinary workspace incremental policy remain; isolated profiling is
triggered by an actual recurring binary-owned hotspot. Remote/multilevel storage, distributed
execution and embedded graph/shear APIs remain conditional opportunities with concrete
consumer/operation triggers. No new service, crate or compatibility path was introduced.

The maintainer skipped final speed quantification because these adoption decisions were
already settled. The published observations establish mechanisms and bounded behavior,
without a measured end-to-end speedup or full-editor, whole-Python or whole-platform claim.
