---
title: Boundaries, build, test and development
status: in-progress
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
| <a id="f1"></a>F1 | Existing bounded stream/RPC owners; A5/E exact trust/read interfaces | Installed Python pull/cancel/drain, prepaid responses, weighted physical/file cache, gather/export consumers; remove redundant copies/unsafe release paths | In progress |
| <a id="f2"></a>F2 | Actual lower admission/generator manifests; RC08; selected public-setting decision | Remaining exact consumer/dependency/generator migration, truthful extension modes and float contract; retire replaced roots/projections after checks | In progress |
| <a id="f3"></a>F3 | Current invocation/admission/native-build owners; C actual resources | Product dispatch/width/build-input/inventory/provenance/setup corrections and explicit residual host-policy disposition | In progress |

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

Next: integrate ready implementation, exercise focused owners, regenerate from the actual
extension, then complete the installed mode matrix and owned delayed canonical journey. Preserve
all prior recovery evidence and unknown allocations. The initial installed native extension is
stale against generated contracts and is not acceptance evidence. Other active-plan reconciliation
and the complete assembled campaign remain with the series closeout, not this checkpoint.
