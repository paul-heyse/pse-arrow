---
title: Execution resources and platform profiles
status: planned
date: 2026-10-17
adrs: []
review_sources: [docs/design_review/reviews/design_review_solver-diagnostics-and-performance-target_2026-10-17.md]
scenario_sources: [docs/design_review/reviews/design_review_solver-diagnostics-and-performance-target_2026-10-17.md]
doc_owner: docs/plans/43-solver-diagnostics-and-performance.md
doc_topics: [current-work, design, environment, libraries, validation]
---

# Plan 43f: Execution resources and platform profiles

## Purpose and foundations

This companion develops O03/O05/O07 and Q06/platform contributors to Q07 in [Plan43](43-solver-diagnostics-and-performance.md). It owns F-package progress. The operator accepted RC08/RC09's bounded routes on 2026-10-17, retaining conservative defaults until a concrete optional profile/provider is selected. No unknown hardware, allocator or numerical winner is preapproved. All implementation/equivalence/measurement remains **Proposed**.

**Interface-checked:** current CaseWorker serial shared scratch, provider/session construction, runtime jobs/study/admission, POUNCE factor pools, Cargo profiling/dist profiles, explicit evaluator compiler flags and native/provider settings. Exclusive mutable workspaces, immutable preparation, issued-work ownership and existing admission are suitable foundations. Fine-grained parallelism must not multiply nested teams/scratch or replace existing execution policy with another scheduler.

## Target packages

<a id="f01"></a>

### F01 — Exclusive admitted evaluator lanes and pool composition

An evaluator lane owns mutable evaluators/provider workers/point caches/nested-session state, sharing only immutable programs/maps. Execute independent case, instance or compatible region work concurrently and reduce contributions in the declared order. Do not put a Rayon loop around shared CaseWorker scratch or serialize hidden global provider locks and call it useful parallelism.

Coordinate lane construction/reset/eviction and retained bytes with runtime jobs/study admission, provider factories, nested native sessions and factor/BLAS/OpenMP pools. Reuse a POUNCE pool across an appropriate compatible admitted lifetime where fitting; it already serves a finite sequence rather than one factor. Issued work stays owned until native join/reset/drain, even if a waiting outer operation cancels.

Compare coarse independent cases, compatible region batches and repeated-instance chunks. Fitting Symbolica batch/SIMD capabilities are eligible; branches/fallible providers, distinct differentiation coordinates, selected demand and numerical reduction remain host obligations. No custom arithmetic lane engine is the default. Sequential/small work retains a low-overhead route; size alone does not establish a universal parallel threshold.

Deterministic scatter does not guarantee deterministic provider effects/error choice or whole native trajectory. Preserve the explicitly promised reduction/error-selection class and test it. FERAL's documented per-supernode parallel/sequential parity remains a scoped library contract, not byte determinism across ordering/ISA/FMA/thread/provider changes.

Acceptance includes concurrent stateful providers, repeated instances, nested implicit evaluation, cancellation/failure, reset/eviction and retained-memory accounting. Show no escaped mutable scratch, silent nested pool expansion or early resource release. Compare single-solve latency and study throughput only when a concrete decision needs measurement; no fixed new runtime/concurrency caps.

<a id="f02"></a>

### F02 — Notification-based waiting under current owners

Replace avoidable periodic admission/resume/native-child/Uno try-lock retry with appropriate existing notifications/wakes. Preserve the queue/admission authority, deadline/cancellation checks, fairness where promised and actual native completion/drain. No second scheduler, background reaper or hidden stop route.

Uno's process exclusion has a native safety purpose; retain it while improving waiting. Model wake-before-wait, release concurrent with registration, spurious notification, cancelled waiter, timeout/release race and late child completion. Terminal observation does not imply unobserved foreign work has drained. Callers retain typed limit/failure reasons.

Acceptance proves no lost wake or busy periodic retry under the exercised path, correct cancellation/deadline and one eventual release after owned work ends. Observe timer/wake work if useful; source reasoning can establish avoidable polling without an exact latency benchmark. Independent admission tests should not require unrelated solver/store startup.

<a id="f03"></a>

### F03 — Required equivalence and actual execution context

Settle Q06 before selecting a changed platform profile. State required behavior for result/derivative/termination/branch/original acceptance and the scope of any deterministic promise. Keep the conservative current profile until a concrete alternative is qualified/selected. There is no blanket tolerance-only relaxation or inference that O3/LTO is bitwise neutral.

Inspect Cargo/native compilation, explicit FP contraction/FMA, ISA selection, runtime SIMD/BLAS dispatch, solver/order/scaling and team counts. Rust transcendental precision and foreign/reduction choices have their actual contracts. Baseline x86-64 versus explicit v3 is a profile choice; host-native target is not silently adopted. Changes affecting enduring precision/boundary decisions follow RC08's concrete route before adoption.

Record actual affinity, cache/SMT group and relevant governor/boost/thread context when interpreting a selected observation. Current reviewed topology supports its stated CPU groups on this machine, not portable numbering or a proven universally optimal timing CCD. Compare placements explicitly if that is the question. Use actual native getters/selected profiles where safe; unknown kernel dispatch remains unknown rather than AVX2/AVX-512 inferred from AUTO/CPU/SDK presence.

This is numerical/diagnostic interpretation, not environment/provenance attestation. Do not build fingerprints, proof chains, capture-to-install transition assurance or sensitivity to incidental environment drift. E05 consumes only the host/build capability needed by its selected capture.

Acceptance independently checks representative values/derivatives and affected scientific trajectories/branches/termination under the selected profile. Determinism claims name their conditions; no CPU ISA or thread change is accepted solely because one scalar output is close.

<a id="f04"></a>

### F04 — BLAS, factor providers and allocator alternatives

Keep technical eligibility open for MKL AUTO/AOCL/HSL and fitting factor providers, including product use when justified. Licence alone is not a rejection. RC09 accepts an investigation/adoption decision route, not a chosen provider. Compare actual use, ABI/index widths, linked dependencies, threading, numerical equivalence, lifecycle and maintenance cost. Do not assume AUTO's AMD dispatch/gain or select a provider from an arbitrary10% BLAS-share threshold.

Source-supported COMPATIBLE/SSE2 explains a possible mechanism; it does not measure the workload's dense fraction or total effect. Native MUMPS/SPRAL/Pardiso/FERAL choices remain class/policy contracts under C04. Provider image/ABI changes migrate their native/Rust/Python consumers and affected qualification through the normal decision route.

Allocator exploration is narrower: first remove demonstrated churn under B/C. Then, if still decision-relevant, compare Rust allocator choice/mimalloc, native/Python allocation and glibc MALLOC_ARENA_MAX at their actual scopes. A Rust global allocator does not automatically cover every Python/native library. An experiment owns its configuration and restores it; no workstation-wide default is installed by a benchmark. Omit unsupported0–10% gains.

Completion selects a concrete qualified optional provider/allocator route or retains current settings with supported reasons. Tests cover allocation/release/failure, FFI ownership/ABI, scientific equivalence, nested teams and process-specific effects. Optional installation/build capability needs are explicit, not a compulsory provider matrix.

<a id="f05"></a>

### F05 — Build realization and trained optimization choices

Use existing release-derived profiling/full debug and dist fat-LTO/one-codegen-unit profiles where fitting; avoid redundant profiles. Current dev/force-validated observations remain valid for their disclosed configuration and do not imply shipped performance. Compare O3/package optimization, LTO/codegen, frame-pointer builds and compiled-evaluator flags only for a selected question, with F03 equivalence and actual binary/native scope.

Correctness closures retain force-validation. A selected timing-only route can separate validation cost only if it does not claim to be the correctness acceptance run; retain an independent force-validated reference and actual features/mode. Do not silently change the assertion-carrying case harness into unvalidated correctness evidence. No new profile is required to recognize source-proven repeated work.

PGO/cargo-pgo and LLVM BOLT are conditional later realizations, not foundational requirements. A representative training/held-out corpus, native/generated-code applicability, reproducible recipe inputs needed to repeat the experiment and independent numerical checks are required if selected. They introduce no environment attestation or mandatory statistical campaign. Compiler remarks/llvm-mca from E05 may inform a confirmed hot region but not substitute for whole-path evidence.

B06 derivative compilation consumes the conservative/selected build contract it actually needs. Higher-frequency frame-pointer versus DWARF sampling requires E05's unwind/overhead capability decision; report native symbols and unresolved frames. Keep the pinned toolchain and finite owned compiler-child cancellation/drain.

## Work packages

| Packet | Prerequisite | Delivery and acceptance | State |
| --- | --- | --- | --- |
| F01 | Exclusive worker/provider construction and enclosing resource contract; B product slice where needed | Admitted lanes/pools with preserved scientific behavior, selected reduction/error class and actual drain; optional library batching route | open |
| F02 | Existing wait/admission/native exclusion contracts | Notification-based waiting with race/cancellation/deadline/fairness tests; remove replaced polling without another scheduler | open |
| F03 | Q06 requirement/actual-context inspection; accepted RC08 route | Explicit selected equivalence profile or conservative retained branch, actual/unknown dispatch facts and qualified boundaries | open |
| F04 | F03 equivalence; accepted RC09 route; particular provider/allocator requirement | Qualified concrete alternative or supported current-provider/allocator branch, coherent ABI/thread/ownership consumers | open |
| F05 | F03 where numerical realization changes; actual build/compiler lifecycle | Fitting existing/selected profile and any justified trained optimization with independent correctness; no redundant blanket campaign | open |

## Verification and checkpoint

**Proposed:** pure race/policy/lane tests and actual touched native/provider fixtures. Use existing workload placement/environment routes; never share target directories across checkouts. G02 assembles concurrency/native outcomes, G03 handles decision-relevant benefit evidence, and G04 scope-end feature/static checks. Do not run all platform/tool alternatives as a mandatory Cartesian qualification.

Current state: document scope only. No lane/wait/provider/build/host changes, installation, scientific test or timing measurement. Conservative current settings remain until a concrete selected branch is implemented/qualified. Completed inquiries cannot erase open structural findings in other companions.
