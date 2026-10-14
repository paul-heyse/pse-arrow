---
title: Boundaries, build, test and development
status: done
date: 2026-10-14
adrs: [ADR-0188]
review_sources: [docs/design_review/reviews/design_review_efficiency-and-solver-target-audit_2026-10-12.md]
doc_owner: docs/plans/38-efficiency-and-solver-target.md
doc_topics: [architecture, environment, validation]
---

# Plan 38f: Boundaries, build, test and development

## Scope and current foundations

This companion implements [Plan 38](38-efficiency-and-solver-target.md)'s product boundaries and actual-dependency development loop. It carries P-D1–D7 and product-relevant P-E1–E6, retaining all remaining source questions explicitly. It does not infer general host-policy harm or schedule a new host-ledger/scheduler redesign. [Audit coverage](../design_review/evidence/efficiency-and-solver-target-audit-2026-10-12/parallelism-and-coverage.md) rejects excluding Python/file work wholesale; Plan 36d/e provide historical selections/checkpoints, not current packet status.

Current source already contains `pse-admission`, Runtime-owned surrogate execution, xtask physical-admission/product-schemas/native-abi capabilities, explicit JSON float_roundtrip and many buffer/stream/accounting mechanisms. Admission's manifest avoids Runtime/Math/Compiler/Operations/backend but legitimately consumes engine/DataFusion; testkit adds its actual fixture closure. Runtime still has broad scientific/Operations dependencies, backend always has POUNCE/FERAL contracts, and pse-py is a cdylib without a Rust unit harness. Source/manifests are **Interface-checked**, not actual selected Cargo graph or installed-binary qualification. Target/checks are **Proposed**.

Current blueprint §3/§14/§18.8/§21 and environment/validation guides own boundaries. RC08/09/11 and R38-02 accepted fences precede relevant G0 amendments; ordinary feature/dependency adjustments do not automatically require ADRs. Changes to public Python/crate contracts use their actual route. Generated outputs are never hand-edited.

## Packets

| Packet | Prerequisites | Delivery/retirement | State |
|---|---|---|---|
| <a id="f1"></a>F1 | Existing bounded stream/RPC owners; A5/E exact trust/read interfaces | Installed Python pull/cancel/drain, prepaid responses, weighted physical/file cache, gather/export consumers; remove redundant copies/unsafe release paths | Done |
| <a id="f2"></a>F2 | Actual lower admission/generator manifests; RC08; selected public-setting decision | Remaining exact consumer/dependency/generator migration, truthful extension modes and float contract; retire replaced roots/projections after checks | Done |
| <a id="f3"></a>F3 | Current invocation/admission/native-build owners; C actual resources | Product dispatch/width/build-input/inventory/provenance/setup corrections and explicit residual host-policy disposition | Done |

F1 subcontracts can improve independently; actual installed-extension/drain integration remains required. F2 verifies existing extraction rather than creates another crate. F3 host decisions are bounded by I-F3; explicit per-operation classes/widths must not silently change scientific native teams. Root manifests/recipes/registry/generated roots are shared edit surfaces owned by execution coordination.

## F1 — Python, Arrow and transport ownership

Complete the actual delayed TableStream/native pull path: observe SIGINT without acquiring a producer-held lock, request cancellation, retain the original exception and already yielded arrays, then settle/join the exact original pull before releasing matching protection/charge. Caller return or decoded/discarded response is not native drain. Request/response nested Operations decoding moves the same prepaid allowance and authentic native/correlation/owner witness, not a fresh unaccounted allocation. Pipes/new file tier retain equivalent bounded decode/backpressure/malformed-input and late-response rules.

Retain actual source/physical/validation identity and fresh caller protections when alternating physical packages. A weighted bounded multi-entry cache may replace single-slot repeated work where already selected/source-supported, with exact owner/equivalence/eviction accounting. Registry name/fingerprint alone is not context equivalence. I-F1 distinguishes implemented mechanism from remaining installed/canonical acceptance and unresolved mutable SQL/UDF/context questions. File capacity activates only for a real qualified file source, with raced first-use/refusal rollback and truthful read-only inactive inspection. Shared pool reservation is accounting, not RSS.

Use library Arrow take for fitting primitive/string gather and selected-range copy for nested lists where required, preserving null/duplicate/range/metadata/reservation semantics. Buffered IPC file export retains finish/flush/error and destination visibility. Existing C-stream shares buffers; optimize conversion/retention instead of inventing another transport. Structural validity, integrity and semantic admission remain distinct.

**Targeted controls:** confirmed entered delayed pull, signal-before/ordinary bridge, cancellation while native producer holds its lock, exact response drain and exception/array preservation; original protection release only after native transport; nested allowance transfer/error/discard; alternating source cache and owner eviction; list/string/null/duplicate gather; cache activation/refusal/inactive; buffered export failure/finish. Actual imported extension plus canonical/IPC journey belongs G3; a mocked Python event alone cannot qualify it.

## F2 — Actual dependency roots, generators and extension modes

Compile/test each responsibility at its actual source owner. Local policy tests inject capability facts; admission tests use true physical/engine context; genuine canonical/native workflows remain integration tests. Direct setup must not pull Runtime merely because Runtime calls a lower helper. Removing service startup does not prove the manifest/dev/build closure is narrow.

Verify existing `pse-admission` consumer/generated adapter migration and xtask capability roots. The physical generator consumes PhysicalInventory/actual engine/registry/resources without Runtime/native product schemas; full product schemas legitimately consume workflow/Operations/native DTOs. A focused subcommand in a broadly linked binary is insufficient. Full aggregate codegen still covers all outputs. Surrogate source is Runtime-owned; scientific state/RNG behavior remains D5's contract. Remove stale old imports/modules/hack/fallback machinery only where actual selected target/host/dev/native graphs show replacement and exact family/feature invariants hold. Do not add a new crate to make counts smaller.

Retain exact dependency pins, locked resolution, workspace feature-unification/toolchain, one Arrow/DataFusion universe, unwind and floating-point constraints. Explicit `force-validate` covers every actual Arrow consuming correctness closure, including selected root dev/host edges; pure closures need no unrelated relation root. Preserve DataFusion SQL/default functions/Parquet/compressed-file and actual document/physical features unless the supported input/function contract explicitly changes. Confirm current removal of unused BLAKE3 rayon and production pse-py test-support separately from public setting R38-02.

The installed Python extension is the correctness boundary: interactive editable mode and explicitly validated correctness mode retain actual binary path/build/native/features provenance and correct dispatch. pse-py has no substitute Rust unit harness. Preserve delivered serde_json float_roundtrip and challenging exact bits in a solver-independent closure. No reimplementation of completed features is scheduled.

Accepted R38-02 removes `ResourceBudget::hashing_may_use_pool` and settings/Python projections/mechanism tests only after I-F2 confirms no real behavior consumer. A public Python contract change takes its decision route; registry/codegen owns generated stubs. Hash bytes/framing/identity do not change, and removing a rayon dependency feature is not the same change. A real consumer reopens the explicit choice rather than silently deleting supported behavior.

**Targeted controls:** exact affected package/capability compile checks, direct owner tests, actual target/host/dev/native selected-unit inspection where needed, explicit Arrow leaf activation, no reverse dependencies, all current generated consumers/aggregate comparison, real imported extension cfg/path/mode, exact float bits and public-setting absence/retained hash identity. Unit-count changes are graph evidence, not build-time gains. No wholesale lock update or alternate toolchain/cache clearing.

## F3 — Product invocation and bounded development work

Product Python/test/solver launch and cancellation use the existing direct owner instead of an unnecessary Rust dispatch/build hop. Preserve selection/exit/outcome, signal ownership and child native drain; observation/report/cleanup failures remain separately honest. Generic unrelated exit sweeps and host ledger redesign are not new product functionality. I-F3 accounts for any residual P-E1/P-E2 acceptance: keep authentic exact resource/retention/unknown records and existing owner, transfer product-required controls to G3, and document why a generic mechanism is retained rather than silently closing it.

Prepare one actual invocation inventory with selected Cargo unit closure, binary/package metadata, filter/profile/platform and executable associations, consumed by enumeration and run. Changed/unknown source/config/environment invalidates it. This is within-invocation reuse, not a cross-run cache or skipped force-validation resolution. Correct exact test names/effect-group classification from actual inventory; store exclusivity is independent of CPU width. Preserve fresh fixture identities and necessary canonical integration; used-fixture pooling needs a real reset/isolation contract and remains unselected absent need.

Native build-script environment/header observation follows actual selected feature consumers. Preserve POUNCE source identity independent of linking, actual receipt/link/transitive-header/bindgen/clang/target/options and byte-preserving generated writes. E-C01 acceptance compares the same target/host/feature/profile unit through plain→unused-native-env→plain and relevant consumed inputs, distinguishing script execution, output bytes/mtime, backend and downstream rustc. Different units or script rerun alone do not establish rebuild benefit. Existing conditional implementation is baseline; fix only remaining actual defects and verify its real consumer.

Keep conservative complete provenance/RUST_INPUTS capture until a responsibility supplies complete present/absent inputs. Retain locked isolated uv/parity sync and incremental maturin with actual interpreter/distributions/editable/native/import identity; only immutable admitted documents may share an existing owner without caching scientific reference results. IDAES/Pyomo references remain independent and fail-never-skip. Header/receipt/native proof caches require actual mutation/borrower lifetime, never stat tuples alone.

Derive actual product/default Cargo/nextest widths from admitted physical lanes, logical affinity and finite quota, preserving explicit overrides within enclosing authority and store/effect groups separately. C4 owns native teams/resources. Retain current disjoint functional/timing partitions/quotas/finite population as accepted default; broader borrowing, fairness, cross-project weights or scheduler redesign is not selected. F28's historical arithmetic is not measured harm. I-F3 can reopen only for a named constrained product operation plus enforceable interference/revocation/drain/owner contract and unchanged-artifact admitted evidence. Otherwise its explicit outcome is retained policy and truthful support limits, not an endless benchmark.

**Targeted controls:** direct selection/exit and actual signal/drain; exact prepared inventory association/unknown refusal; intended relevant versus irrelevant native inputs and stable output; installed reference environment identity; class inheritance/explicit override/CPU versus effect capacity; actual recipe-level placement. Unknown/pinned external work is preserved. No infrastructure guard is bypassed to make a test run.

## Investigations, optional opportunities and verification

| Investigation | Decision criterion and explicit outcome route |
|---|---|
| I-F1 — Installed boundary and actual context | Map every delayed-stream/RPC/cache consumer and exact native drain/validation owner; complete remaining installed/canonical controls. Select remaining correction or preserve demonstrated implementation with its scope. |
| I-F2 — Boundary/feature/build opportunities | Confirm public dead knob consumer; actual multi-entry cache demand, Arrow gather, buffered export, UDF/reflection/metadata contexts and selected feature roots. Select only premise-complete changes; retain current behavior or observable request trigger otherwise. |
| I-F3 — Remaining host/development questions | Inventory residual P-E1–E6/F25/F28 source obligations; assign product-required acceptance, retain existing general host owner/policy with explicit reasons, and identify any concrete changed premise needing operator decision. Nothing is declared passed by supersession. |

Optional sorted-source claims require actual cross-batch/partition ordering. Keep scope-sensitive UDF snapshots, bound-owner reflection and isolated mutable cattrs hooks unless complete equivalence/workload evidence supports sharing. Preserve explicit 4096-row complete-campaign refusal/support limit where still consumed. No compression/cross-output-query/inbound-Arrow/hash-algorithm change is inferred from library availability. Library questions use skills/Context7/exact source; no stale maps/utilization journal.

Compile/targeted tests accompany implementation; regenerate actual declarations when changed. At functional scope end G1–G3 select real installed Python/native/canonical, feature/closure/codegen and applicable hygiene/manual checks. Prior 940-control setup, 398 feature configurations and interrupted managed Python evidence retain their original source/artifact conditions; they are not current assembled acceptance.

## Execution checkpoint — 2026-10-14

The maintainer authorized the detailed execution approach and selected focused 38f acceptance;
full 38g scientific/integration/performance/hygiene qualification remains separate. The public
setting choice explicitly includes its accidental equality distinction: otherwise identical
remaining settings may reuse the process Runtime. The old flag caused refusal, never isolation.
[The independent bounded target review](../design_review/reviews/design_review_consumed-runtime-settings_2026-10-14.md)
accepts that target; proposed ADR-0188 and blueprint revision 167 precede dependent code removal.
Its RC01/RC02 are adopted by the accepted R38-02 and this explicit execution choice; F2 owns
implementation and focused acceptance. No new review finding is open.

The actual delayed-read gap is late metadata transport retaining connection activity without
necessarily retaining the original strong ResultRead and its prepaid decode allowance. F1
corrects composite ownership at real result/file/artifact/progress consumers and replaces the
stale installed delayed-SIGINT fixture with known consecutive blocks in distinct sealed files.
File decode/footer estimates remain distinct from RPC metadata-response grants.

F3 retains direct dispatch and fixes the remaining prepared-inventory handoff/association and
default-width gaps. Physical lanes, usable logical affinity and enforced finite quota determine
default widths; explicit overrides, effect slots and scientific native teams retain their own
contracts. Root serializes compilation, generation and installed builds on main; two isolated
editing worktrees own F1 and F3. No worktree-local builds or service startup are selected.

At this checkpoint the next work was integration, focused owner checks, generation from the
actual extension and installed/canonical acceptance; the Outcome below records their completion. Preserve
all prior recovery evidence and unknown allocations. The initial installed native extension is
stale against generated contracts and is not acceptance evidence. Other active-plan reconciliation
and the complete assembled campaign remain with the series closeout, not this checkpoint.

## Investigation outcomes — 2026-10-14

**I-F1 — Implemented / Tested within focused acceptance.** Result metadata/file-open,
artifact, progress, completion and standalone Operations readers now prepay their
own response envelope before dispatch. A dispatched file-open response retains the
original strong ResultRead and its separate 96 MiB metadata grant across both
sequential descriptor requests; decode/footer accounting remains separate. Nested
SDK response owners compose rather than replace one another. Dropped waiters do
not terminate SDK transport. The installed control deliberately seeds two consecutive
blocks in distinct sealed files and distinguishes ordinary bridge settlement,
signal-before-entry and entered pull cancellation. The protected fixture's authentic
release must finish before disconnect; helper retries only ConnectionBusy within
one finite deadline, and every seeding request key belongs to its fresh run.

The existing weighted multi-entry physical cache remains selected: exact source,
registry, session factory, worker and pool identities plus fresh caller protections
are necessary. Canonical interleaving, eviction/fencing and final-owner controls
exercise that contract. The guarded IPC StreamingTable path still does not activate
DataFusion's general file metadata/stat/list cache; its separately charged footer
cache is a different mechanism. Optional file capacity stays inactive until a real
qualified file source exists. No mutable SQL/UDF context is declared equivalent by
registry name or fingerprint. A demonstrated repeated workload with a complete
context-equivalence contract is the trigger for further sharing.

**I-F2 — Implemented / Interface-checked.** The removed hashing permission had no
hashing consumer. Its ResourceBudget equality participation was an accidental
Runtime-acquisition refusal, not a separate pool. Actual consumed settings still
control compatibility. BLAKE3 rayon remains absent; framing/hash vectors and JSON
float_roundtrip remain unchanged. The existing lower admission and generator roots
are retained: pure physical/native-ABI work does not need Runtime, while product
schema generation legitimately consumes scientific/workflow DTOs. No new crate or
removed scientific capability is warranted. The lockfile source change selects five
exact local supplier packages without a version or transitive-edge change.

Retain Arrow take for primitive/string gather, bounded selected-range copying for
nested lists and buffered no-replace IPC export with finish/flush/sync obligations.
Retain DataFusion's SQL/functions/Parquet/compressed-file features, scope-sensitive
validation snapshots, reflection under its actual bound owner and isolated mutable
cattrs converters. No demonstrated bottleneck plus complete equivalence premise
supports another cache or transport. Sorted-source claims still require actual
cross-batch/partition ordering; compression, cross-output queries, inbound Arrow
and a hash-algorithm change remain unselected. The former 4096-point campaign limit
is not a current study contract: 38d supplies explicit bounded controls and a
100,000-point ceiling in study.rs/study_execution.rs. This inquiry does not qualify
a 100,000-point scientific campaign or revive the obsolete cap.

**I-F3 — Implemented / retained policy.** Direct Python/pytest/Nextest launch stays
with the existing owner. One invocation retains its validated graph, Cargo and
binary metadata, selected test catalog and executable association; source/config,
explicit environment, selection and artifact changes refuse reuse. Automatic widths
follow admitted physical lanes, usable affinity and finite ancestor quota. The
observer's exact admission/setup fields and JUnit report composition are execution
context; arbitrary new build inputs remain significant. Managed observers stay
single-process, and effect/store/native-team capacities remain separate.

Keep conservative complete RUST_INPUTS, immutable managed native generations,
external-prefix admission, locked isolated environments and incremental maturin.
Private relevant-header/selector/receipt probes supply the native-input correction's
functional controls. SUNDIALS bundled CMake regenerates SUN_JOB_ID and
SUN_JOB_START_TIME on reconfiguration: those changed input bytes correctly change
its Rust bindings; no identical-output or downstream-freshness claim is made for
that case. The comparison writer preserves outputs when bytes actually match.
The current build_libraries unit does not consume external SUNDIALS prefix selectors;
its consumed KLU inputs are tested, while separate optional provider branches retain
source-reviewed support pending 38g qualification.

The supplier import also exposed an actual integration limit: complete source
observation refused the larger checkout's metadata inventory. Source capture now
allows 16 MiB per inventory and prepays 36 MiB for both simultaneous inventories
plus existing scratch. Full-content hashing, ordering, reconfirmation, path/depth
bounds and cancellation remain intact; no input family is elided. The existing
startup lease retains its whole admitted allowance, so this is an additional
32 MiB of pool charge for that observation's lifetime, not an RSS claim. Smaller
pools may honestly refuse. The 8,192-file regression control exercises complete
capture, final-file mutation, framing equivalence and cancellation; existing
small-budget controls retain refusal before insertion.

General authenticated host accounting, retention, unknown-record safeguards and
disjoint functional/timing policy remain with their existing owner. No unchanged-
artifact evidence identifies a constrained product operation requiring borrowing,
fairness weights, a new scheduler or another ledger. Their retained outcome is
explicit, not acceptance by supersession. Reopen only for a named operation with
an enforceable interference/revocation/drain contract. Independent IDAES/Pyomo
reference identity is checked separately from scientific parity; no reference result
cache or scientific parity qualification is inferred.

## Verification — focused execution, 2026-10-14

Failure baseline is **zero**. Commands below ran through `scripts/pse-env` and the
private registry `/tmp/pse38e-resources-5aLWFeeA`; canonical commands selected the
owned v6 fixture `/tmp/pse38f-boundary-mwr7yh1f/live`. Arrow-consuming correctness
closures explicitly enabled validation. Installed interactive builds were inspected
only; correctness ran on explicitly validated binaries. No cache clean, alternate
target directory/toolchain, shared-ledger migration or unknown-resource disposal
was used. Targeted passes do not qualify the enclosing Plan 38g campaign.

| Evidence | Command / conditions | Result against zero baseline |
|---|---|---|
| **Tested** — exact hash/float contract | `python3 -m scripts.arrow_validation nextest run --no-fail-fast -p pse-ids --lib --test golden_vectors --locked -E 'test(json_finite_float_roundtrip_is_bit_exact) \| test(frozen_canonical_float_document_vectors) \| test(the_framed_hasher_reproduces_the_frozen_vectors)'` | 3 passed; 64 unselected |
| **Tested** — actual lower owners | `just unit-package pse-admission 'all()' --execution-effects native-local`; `just unit-package pse-relations checked_take --execution-effects native-local`; `just unit-package pse-engine 'test(cache_service::tests)' --execution-effects native-local` | 19, 9 and 14 passed respectively; 0/59/170 unselected |
| **Tested** — nested original response ownership | `python3 -m scripts.arrow_validation nextest run --no-fail-fast -p pse-operations --lib --locked -E 'test(canonical_response_owner)'` | 3 passed; 81 unselected; corrected ordinary-test cfg leaves no warnings |
| **Tested** — native canonical readers/cache/export | `just unit-package pse-runtime` with `--features pse-runtime/canonical-tests`, selecting standalone/public artifact prepay refusal, physical admission/final owners, connected exact-range/cancellation/export, slim completion and integral progress controls | 12 passed; 559 unselected, repeated after the source-observation correction; actual prepared inventory reused through the authenticated exclusive observer |
| **Tested** — complete bounded source capture | `python3 -m scripts.arrow_validation nextest run --no-fail-fast -p pse-buildinfo --lib --locked -E 'test(foundation_unit)'` | 24 passed; includes 8,192-file complete inventory and small-budget refusal |
| **Tested** — invocation, direct dispatch and allocation | `.venv/bin/python -m unittest scripts.tests.test_arrow_validation scripts.tests.test_native_tests scripts.tests.test_pse_env scripts.tests.test_python_tests scripts.tests.test_test_run scripts.tests.test_plan30_routing scripts.tests.test_execution_contracts` | 158 passed; after the final exact observer setup-field correction, affected `ManagedRustSelectionTests` independently reran: 21 passed |
| **Tested** — actual native-local prepared execution | `just unit-package pse-ids 'test(json_finite_float_roundtrip_is_bit_exact)' --execution-effects native-local` | 1 passed; 51 unselected |
| **Interface-checked / Tested** — selected compilation | `just check-package pse-admission`; `just check-package pse-runtime -p pse-py`; selected xtask physical-admission/native-abi checks; backend KINSOL check with `--native=klu` | Passed under locked selected-root validation; no warning claim is transferred to unselected native features |
| **Tested** — ordinary installed boundary | `python3 -m scripts.python_tests -m 'unit or component'` selecting native registry/cache/stub/extension/settings projection files plus the equal-consumed-settings Runtime control | 23 passed with 4 admitted default pytest workers; arrays survive stream close and cancellation is observable |
| **Tested** — linked installed normal journey | `python3 -m scripts.python_tests -n 0 -m 'unit or integration'` selecting stub surface, typed route/eligibility, cache release, live-reader/independent caller and result/progress reopen | Initial 6 passed / 2 failed solely at stale close-message assertions; corrected close controls, including already-admitted solve preservation, reran: 3 passed. The passed reopen journey exercises artifact promotion/read/retention, analysis, progress and buffered no-replace export |
| **Tested / Measured** — entered installed SIGINT | `python3 -m pse.tests.canonical_signal_control --state <owned-state> --fixture-receipt <two-file-receipt> --exclusive-owned-instance 9b964d94-fb4b-4cdd-a23c-862459582feb --spill-directory <owned-spill>` on the linked validated extension | Completed: original exception/cause, stopped-server native pull join, discard-before-resume, retained late-response grant, exact ticket release/collection and escaped Arrow row preservation. Controlled join 0.056 s; ordinary bridge settlement 0.279 s in this deliberately stopped local-server fixture, not a general performance benchmark |
| **Tested** — environment/reference identity | `just ready`; fresh parity interpreter distribution inspection | Ready passed after dependency-source changes and the environment-shaped diagnosis. Independent Python 3.13.15 / idaes-pse 2.13.0 / Pyomo 6.10.1 identities observed; no scientific parity or current parity-extension qualification claimed |

**Measured — E-C01 same-unit freshness:** the ordinary `cargo check -p pse-runtime
--lib --locked --features force-validate --message-format=json -vv` sequence
plain → unused solver environment → plain used identical target/profile/features
and output directories. Backend and Runtime stayed fresh, the backend script did
not execute, and output bytes/mtimes stayed identical. Unused selectors were
IPOPT_DIR, PSE_ROOT_ISOLATION_DIR, SUITESPARSE_INCLUDE_DIR,
SUITESPARSE_LIBRARY_DIR and PSE_KLU_PROVIDER_RECEIPT. An initial mixed trial also
changed PKG_CONFIG_PATH, which aws-lc-sys actually consumes: its TLS dependency
chain correctly rebuilt, while the backend remained fresh. That trial is not
claimed as an unused-input success or native backend defect.

**Tested / Measured — relevant native inputs:** private coherent copies of the
selected KLU and Uno prefixes were admitted as unmanaged external inputs, never as
new managed qualification. The same backend KINSOL unit rebuilt SuiteSparse and
bundled SUNDIALS after an included KLU header edit, exported macro change and
provider-selector change; unchanged repeats were fresh. SuiteSparse's comment-only
case preserved binding bytes and mtime. SUNDIALS's only comment-case binding changes
were its regenerated SUN_JOB_ID/SUN_JOB_START_TIME metadata. Both bindings included
the new probe macro. Uno's same link unit rebuilt its bridge after a private header
edit, retained its existing build identity, then changed that identity after a
JSON-equivalent private receipt-byte edit; its final unchanged repeat was fresh.
The probes distinguish script execution, binding output and Rust consumer rebuild;
they establish no elapsed-build-time or scientific speed claim. Raw temporary
controls remain under `/tmp/pse38f-e-c01/` and
`/tmp/pse38f-native-inputs-jnjn0afs/`; reproducible source/input routes are described
in [supplier provenance](../../vendor/native-supplier-notes.md).

**Interface-checked:** selected Cargo target/host/dev roots retain the lower
admission, physical generator and native-ABI boundaries. Full product schemas
legitimately retain scientific/workflow consumers. The local exact-source wiring
keeps 33 workspace members and changes only five lockfile source associations,
without a dependency-version or transitive-edge change. Unit counts are graph
facts, not build-time measurements. Independent source review found and corrected
missing standalone metadata owners and JUnit-profile handoff; the final bounded
review found no material blocker, including the source-capture envelope correction.

**Not run / unexamined:** full scientific, parity, performance, feature-powerset,
manual governance/docs/native lint and hygiene acceptance remains with 38g, as
explicitly selected by the maintainer. Direct delayed cancellation of each
standalone/artifact entrypoint is not separately exercised: actual pre-dispatch
refusal, normal artifact journeys, nested SDK owner controls and entered installed
file cancellation supply compositional evidence. Arbitrary embedding/interposition,
mutable SQL/UDF equivalence and separate optional external-SUNDIALS/Diffsol provider
branches are not newly qualified. No general host-policy harm or benchmark gain
is inferred. Prior interrupted/recovery evidence and unrelated retained allocations
keep their original outcomes.

### Installed mode matrix and aggregate generation

**Tested:** all four modes were rebuilt after the source-capture correction and
observed by fresh processes importing the actual checkout extension. Every mode
used `python/pse/_native.abi3.so`; environment text did not certify Rust cfg flags.
The final installed selection is linked correctness.

| Mode / build command | Actual compiled features | Actual binary SHA256 / acceptance |
|---|---|---|
| `just py-sync` | none | `6a29c4b32760128a98a895fa647011af379c5260983efbb56ffb91865db00b20`; inspection only |
| `just py-sync-correctness dev` | force-validate | `d9b458d59049a449c29c9a8eb2d8ffd69801f1f37c212ef8e88490cf2cd43d9f`; 23 unit/component checks and fresh two-file SIGINT control passed |
| `just py-sync-native` | native-solvers, solver-highs | `42dd20675d0d9e3f3ab8463cb9af93216c914fdaba0593e18ba8727c87efcbc2`; inspection only |
| `just py-sync-native-correctness` | force-validate, native-solvers, solver-highs | `c77d9b01c7bd69c01269915e0b6146b0eed66d2a07a7cd84364b9fb13ddd56cd`; final 6 stub/route/canonical artifact-progress-export checks and a fresh two-file SIGINT control passed |

Earlier linked correctness artifact `5118f2776600989bae67b56de5de71852d80ee04de2b373ce7aed93a5805ce97`
exercised the corrected three close controls. Their guard and reader/solve ownership
source did not subsequently change; the final binary separately re-exercised the
normal composed journey and entered cancellation. Final ordinary/linked controlled
pull joins were 0.073/0.073 seconds, with ordinary bridge settlement 0.292/0.289
seconds under the same deliberate stopped-server conditions. These timings are
fixture observations, not scientific throughput or general latency claims.

**Tested:** `just codegen` passed aggregate generation (Rust contracts and canonical
schema first, then all five full-generator targets including Ipopt bindings and
actual compiled Python stubs). No generated diff remained. Capability-specific
checks and final aggregate generation preserve one declaration/consumer route.

## Outcome — 2026-10-14

**Implemented / Tested:** F1–F3 and I-F1–I-F3 are complete within the maintainer's
selected focused acceptance. Enduring public-setting/lifetime contracts live in
blueprint §14.3/§18.8/§21 with revision 167 and proposed ADR-0188. The development
guide and supplier provenance own operating/source details. The dead setting,
projections and obsolete test are removed; no shim or parallel production path
remains. Existing qualified cache, Arrow gather and buffered export mechanisms
are retained with their preservation constraints.

The original target review accepted the bounded public contract change. Independent
implementation review exposed standalone response owners and report-profile
association gaps, which are corrected; its final source review found no material
blocker. The additional complete-source allowance correction is covered by its
independent envelope/caller assessment and actual installed controls. Proposed ADR
status is unchanged; accepted target choices do not bypass the decision-PR route.

The revealing failures were repaired rather than hidden: stale cache activation
expectations, an integration test mislabeled unit, foreign pytest temporary-root
ownership, asynchronous fixture-release timing and reused seed request keys,
observer admission/setup/report transformations, the enlarged source inventory and
stale close-message assertions. Incorrect narrow-lane/no-store invocations failed
before tests and were retried through the proper exclusive/store route. The
ordinary constructor's initial math.library refusal was the source metadata limit;
configured license activation and mathematical initialization independently passed.
All corrected focused controls now pass within their stated conditions.

Owned changes are integrated on main and all three editing worktrees are removed.
The exact private service's context/launch owners were checked drained before the
normal quiesce → stop-with-drain route; it is inactive, listener absent, admission
quiesced. Data and failed-attempt receipts remain available; prior 38e recovery
evidence, unrelated allocations and the warm compiler cache are preserved. Rust,
Python and TOML format scope excludes imported supplier trees. No cache clean,
unknown-record deletion, shared migration override, push or decision-PR acceptance
was performed.

The companion remains retained for active 38g readers. Full assembled/scientific,
parity, performance, hygiene/manual acceptance and combined finding closure remain
with 38g; later other-plan reconciliation remains separately scoped. This outcome
claims focused implementation/acceptance, not whole-product qualification.

**Tested — closeout tooling:** root `just turn-end` passed ADR indexing and formatting;
imported vendor sources have no formatter diff. Its safe-fix Ruff phase still reports
51 repository findings against the zero target; its `--exit-zero` result is not clean
lint evidence. Scope-owned findings were corrected, their four-file focused Ruff
check passed and `ManagedRustSelectionTests` reran with 21 passes. The entered
SIGINT control passed again after those helper corrections; the private service was
then authenticated drained/quiesced/stopped again. Remaining full hygiene is 38g.
Binary mode observations precede final formatting; current-source assembled
qualification must rebuild its artifacts rather than promote these focused receipts.

A bounded operational lead for 38g's manual checks remains: the pre-existing CLI
`start` result can print its pre-start admission snapshot even when fresh config
already records open admission. Fresh configuration and authenticated owner checks,
not that console snapshot, established this fixture's admission and final drain.
No lifecycle guard was bypassed or weakened during the controls.
