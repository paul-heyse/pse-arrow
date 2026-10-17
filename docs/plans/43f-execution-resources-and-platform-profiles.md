---
title: Execution resources and platform profiles
status: in-progress
date: 2026-10-17
adrs: [ADR-0195]
review_sources: [docs/design_review/reviews/design_review_solver-diagnostics-and-performance-target_2026-10-17.md]
scenario_sources: [docs/design_review/reviews/design_review_solver-diagnostics-and-performance-target_2026-10-17.md]
doc_owner: docs/plans/43-solver-diagnostics-and-performance.md
doc_topics: [current-work, design, environment, libraries, validation]
---

# Plan 43f: Execution resources and platform profiles

## Purpose and foundations

This companion develops O03/O05/O07 and Q06/platform contributors to Q07 in [Plan43](43-solver-diagnostics-and-performance.md). It owns F-package progress. The operator accepted RC08/RC09's bounded routes on 2026-10-17, retaining conservative defaults until a concrete optional profile/provider is selected. No unknown hardware, allocator or numerical winner is preapproved. At document authoring, implementation/equivalence/measurement was **Proposed**; the Outcome records delivered scope and evidence.

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

**Remaining production alignment — 2026-10-17:** notification migration is partial. `root_isolation.rs::acquire_library` still retries IBEX exclusion/compute resume every 1 ms; `scip.rs::ConcurrentAdmission::acquire` retries its concurrent gate/resume every 1 ms; `petsc.rs::Admission::enter` wakes every 10 ms to recheck execution; and `pse-math/src/native_compiled.rs::wait` polls compiler exit with `waitid(NOHANG)` and a 5 ms sleep. These are production admission/resume/child waits, not test watchdogs. Complete event-driven release/exit/cancellation/deadline waiting at the existing owners. Preserve native exclusion, predicate rechecks, original absolute clocks, tentative-permit rollback, process identity and actual issued-work join; conservative uncertain-drain handling must not be weakened merely to remove a timer. Existing job admission/resume, Uno, surrogate and private-readiness notification paths remain implemented.

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
| F01 | Exclusive worker/provider construction and enclosing resource contract; B product slice where needed | Admitted lanes/pools with preserved scientific behavior, selected reduction/error class and actual drain; optional library batching route | done |
| F02 | Existing wait/admission/native exclusion contracts | Notification-based waiting with race/cancellation/deadline/fairness tests; remove replaced polling without another scheduler | in progress |
| F03 | Q06 requirement/actual-context inspection; accepted RC08 route | Explicit selected equivalence profile or conservative retained branch, actual/unknown dispatch facts and qualified boundaries | done |
| F04 | F03 equivalence; accepted RC09 route; particular provider/allocator requirement | Qualified concrete alternative or supported current-provider/allocator branch, coherent ABI/thread/ownership consumers | done |
| F05 | F03 where numerical realization changes; actual build/compiler lifecycle | Fitting existing/selected profile and any justified trained optimization with independent correctness; no redundant blanket campaign | done |

## Verification and checkpoint

**Interface-checked — production completeness, 2026-10-17:** F01 matches the selected coarse admitted/exclusive lane target; F03–F05 retain the supported conservative build/provider/allocator choices. F02 remains incomplete for the production native gate/resume/compiler-child polling listed above. **Tested, earlier 2026-10-17:** the Outcome below records controls for the implemented paths. [43g’s Outcome](43g-qualification-and-design-publication.md#outcome-recorded-after-implementation) owns assembled/scoped acceptance, repaired composite boundaries, scope-end checks and exclusions. The supported inquiry conclusions below are complete retained/selected branches; they are not pending benchmark or environment-proof work.

### Q06 / F03–F05 retained-profile decision — 2026-10-17

**Interface-checked:** current Cargo/native/compiler source and actual native control getters. Required equivalence covers original values/derivatives, scientific branches and termination classes, original acceptance and any explicitly promised reduction/error order. Fixed input/library/algorithm/order/thread context is required for a deterministic claim; neither one close scalar nor CNR establishes whole-pipeline bit determinism.

Retain current conservative realization: devO2/dependenciesO3, releaseO2/thinLTO/16 codegen units, existing full-debug profiling and fat-LTO/one-unit dist profiles. Compiled arithmetic explicitly uses baseline x86-64/generic/O2, no fast math and contraction off; the Ipopt bridge also disables contraction/fast math. Common native C/C++/Fortran O2/PIC flags do not establish explicit contraction-off everywhere. No new host-native/v3/FMA/O3 profile is selected without affected scientific qualification. Existing profiles fit the selected capture and derivative compilation; no redundant profile is required.

Retain coherent oneMKL LP64/GNU-thread ABI, COMPATIBLE selection and current teams. Actual CBWR/dynamic/OpenMP getters and thread-local RAII restoration supply current facts; AUTO or SDK/CPU presence does not establish an AMD kernel or gain. AOCL/HSL and other eligible providers have no qualified ABI/thread/lifecycle/scientific-trajectory advantage for this scope; licence is not a rejection reason. There is no arbitrary BLAS-share threshold.

Retain current allocation after the selected B/C churn corrections. No isolated decision-relevant evidence justifies a global allocator or workstation `MALLOC_ARENA_MAX` change, and Rust global allocation would not automatically cover all Python/native allocations. PGO/BOLT are not selected because no representative training/held-out corpus or actual native/generated-code applicability has been established for a concrete target; existing build profiles meet the need. These are completed supported retained branches, not deferred environment/provenance proof or promises of later benchmark tournaments. Qualification of the changed execution mechanisms remains at G02–G04.

## Outcome (recorded after implementation)

### What was built

**Implemented — delivered scope; F02 notification migration is partial as recorded above:** F01/F02 use existing coarse admitted case/study/session lanes with exclusive mutable workers/provider maps and precharged reset overlap. One notifying cancellation owner and actual permit-release/change notifications replace owned idle polling across job admission, native exclusion/pressure, surrogate serialization and private child readiness. Predicate rechecks, original deadlines, rollback and actual issued-work join remain with their existing owners; no nested scheduler or unadmitted fine-grained teams were added.

**Tested, 2026-10-17:** six engine resource controls, 20 runtime job/race controls and selected actor/surrogate/private-child wake/cancel/deadline controls passed. The integrated runtime selection passed 76/76. Three actual managed sixteen-owner/equal-occurrence/cancellation studies passed, including no tail start and registered cleanup. [43g Outcome](43g-qualification-and-design-publication.md#outcome-recorded-after-implementation) owns combined/native/static acceptance and exact conditions.

**Interface-checked:** F03–F05/Q06 retain the conservative numerical/build/provider/allocator profiles specified in the completed retained-profile decision above. Original values/derivatives, branches/termination and acceptance plus explicitly promised order define equivalence; no blanket bit determinism is promised. Keep oneMKL LP64/GNU COMPATIBLE/current teams with actual getters/RAII, current allocation after churn removal and existing dev/release/profiling/dist profiles. Eligible AOCL/HSL/MKL AUTO have no established fitting ABI/lifecycle/scientific advantage here; license is not a rejection reason. No representative training/held-out target justifies PGO/BOLT. These are completed retained decisions, not unfinished experiments or environment proof deferrals.

### A mistake made and corrected

Independent review exposed retries after paired admission semaphores closed and a surrogate stop path that changed a raw flag without waking its waiter. Closed resources now terminate with a typed refusal; the surrogate uses the same notifying cancellation owner. Native private-readiness tests distinguish data, HUP, stop and the original deadline before actual child join.

### Deviations from the plan, deliberate

Retain the existing coarse execution lanes and default platform profiles rather than create fine-grained nested scheduling, a global allocator switch or host-native/FMA flags without useful scientific evidence. Current source-work controls establish removed unnecessary work without exact comparative timing. Formal ADR-0195 remains proposed.
