---
id: ADR-0185
title: Own neutral native reuse, admitted resources and qualified method contracts
status: proposed
date: 2026-10-13
deciders: [paul-heyse]
level: decision
principles: [AP-02, AP-04, AP-06, AP-07, DP-09, DP-10, DP-15, DP-20, PS-06, PS-08, PS-09, PS-10]
blueprint: [§15.4, §17.6, §18.6, §18.8, §18.9, §21]
review: git:6af92d1dee1ffa0b80dca71e03a63b489f2d940c:docs/design_review/reviews/design_review_plan38c-native-reuse-and-resources_2026-10-13.md
evidence: Tested
supersedes: []
superseded-by: null
revisit: A selected profile cannot reset independent-request numerical policy while preserving its neutral state, or a new method requires different scientific or native resource authority.
verification: Scoped native, method, mutation, lifetime and memory-refusal controls with exact commands and conditions in https://github.com/paul-heyse/pse-arrow/blob/6af92d1dee1ffa0b80dca71e03a63b489f2d940c/docs/plans/38c-native-methods-reuse-and-parallelism.md#verification. Independent original acceptance remains mandatory; assembled scientific/native/durable qualification remains Plan38g.
---

# ADR-0185: Own neutral native reuse, admitted resources and qualified method contracts

## Context

Plan38c implements accepted RC06/07, S-RC03–07/11/12, ARC01 and the R38-01 investigation route. Plan38b supplies governing policy and bounded evidence. The baseline contained retained native objects, conditional KINSOL setup, FERAL role pools, HiGHS coefficient updates and quiescent admission. Allocation reuse could retain value-sensitive policy, and update failures could leave uncertain mutable state. The implementation preserves neutral allocations while removing implicit permission for predecessor numerical setup and policy.

The maintainer approved the complete execution approach and selected narrow maintained-fork reset/width enhancements and compatible neutral reuse as the ordinary independent-solve default. These selections authorize the target, not unexamined numerical or performance claims.

## Scope

Amend native allocation/history and resource lifetime contracts in blueprint §17.6/§18.8, precise outcome/capability publication in §18.6/§18.9 and their generated Python consumers. Clarify the bounded rank-method and scalar-root binding routes in §15.4. No new crate, storage topology, journal, host partition, HSL profile or universal solver language is selected. Retain existing accepted ADRs and their unrelated rationale unchanged; this proposed record governs the scoped amendment pending its decision PR.

## Drivers

Interactive value edits and independent study points should reuse compatible neutral native work without inheriting another point's numerical branch or heuristic history. Mutable factors and foreign contexts need exclusive owners through actual drain. Native teams must execute within the admitted deployment envelope. Scientific acceptance and explicit start/branch permission remain independent of retention and native status.

## Options

Fresh objects everywhere avoid hidden history but repeat neutral work. Unqualified persistent wrappers retain numerical state and obstruct resource scopes. Select bounded service-owned lanes extending the existing NativeSession request loop, with complete profile affinity, independently charged residency, fresh request controls and profile-specific reset. Prefer library-owned reset and explicit-width seams; fall back to fresh affected objects when neutral reuse cannot be established. Do not recreate numerical iteration or factorization locally.

## Outcome

Ordinary independent controls now default to AllowRebuild under the implemented profile-specific neutral-reset or fresh-object fallback paths and transactional HiGHS failure handling. Explicit Fresh and RequireReuse remain authoritative. Reusing allocations/structure neither imports a seed nor grants numerical continuation. Independent requests clear implicit numeric setup, value-sensitive heuristics, stale controls and request-owned advanced-step state; intentional within-sequence products retain their exact existing validity. Live integrator continuation remains distinct from a new independent initial-value problem. Independent request boundaries park only accepted neutral HiGHS and POUNCE NLP state; the other direct backends use fresh-object fallback. RequireReuse observes that actual fallback and refuses when retention cannot be honored. KINSOL requests fresh initial numerical setup on every call, including calls within a finite sequence; actual initial-root exits may perform no setup. Ipopt retains its TNLP/application and actual ReOptimize operation while setting warm_start_same_structure=no to refresh current spaces, maps and gradient scaling. These facts promise neither zero allocation nor zero analysis.

A service-owned lane serializes requests on one native owner thread. Compatible idle lanes precede new construction. Lane resident memory, stacks, caches and one population ticket stay charged while parked; each request borrows it exclusively and acquires active/CPU/construction admission under its original clock. Idle scopes do not accumulate unrelated pools or hold obstructive exclusions. Pressure retires idle owners before starvation; native destruction and actual join precede charge release. Source/provider oracles and expired task admissions cannot hide in parked state. Request finish waits for request native work and callback
borrowers to drain, unsafe state to reset/drop and active/construction owners to release.
Only accepted neutral state and resident charges transfer to the service lane; cancelled,
fatal or uncertain requests transfer none. Lane retirement/service shutdown additionally
waits full native/TLS destruction and owner-thread join before resident/population release.

FERAL and its wrapper provide explicit owned-pool-width and application-boundary reset interfaces. Reset restores construction-time numeric/quality policy and clears Auto/MC64/ordering/cascade history, preserving healthy neutral work. Ordering escalation that changed symbolics requires baseline reconstruction. Main/restoration and batch pools retain distinct exclusive owners. Fatal, callback-invalid, uncertain and unresettable state is discarded. Per-attempt observations never reinterpret cumulative summaries as reset counters. POUNCE-convex retains no native session or implicit numerical seed: independent solves and batches start cold, supplied warm state and RequireReuse refuse, and factories receive explicit positive pool widths. A multi-instance batch factors each member serially inside its admitted outer team; a single member may use its admitted width.

Native profiles declare actual transitive teams, stacks, shared loaded-interface exclusions and selected native build identity. Placement width is distinct from scientific settings. Ambient Rayon is explicitly installed where consumed; extra independently created teams are charged. FERAL's persistent pool and foreign stacks remain in the finite opaque allowance; scope-stack accounting is not a complete allocator or RSS bound. SUNDIALS contexts remain thread-affine until migration is qualified.

The selected SCIP tinycthread interface owns a process-static TPI pool. Concurrent SCIP instances therefore acquire quiescent process exclusion and retain it through SCIPfree and native joins. A joined fresh-thread probe uses a separately admitted explicit stack and observes OpenMP/MKL defaults and relevant domains; nonserial nested defaults, including unsupported OpenMP lists, refuse before launch. Parent thread-local controls do not stand in for child-worker defaults. Failed native destruction is Infrastructure hard-terminal uncertainty. The concurrent owner quarantines its gate, Watch and resident allocation lease; when computation also lacks a successful return/drain witness, the same quarantine retains the actual compute owner. Neither timeout nor cleanup failure grants replacement numerical work. Quarantined owners keep their charges until process exit; a returned diagnostic is not a successful drain witness.

A store-free child lane accepts the closed provider-free authored scalar/square-root source supported by the existing implicit factory, with bounded authenticated IPC, original parent deadline/accounting, original-space screening and actual reap before release. Full PreparedSolve objects, opaque provider closures and strict whole-child memory/RSS bounds are unsupported. KINSOL's native profile fingerprint binds the admitted managed KLU receipt, declared native sources, generated SUNDIALS configuration and ABI widths. The existing native qualifier exports PSE_KLU_PROVIDER_RECEIPT only for managed generations; an unmanaged prefix's stale receipt supplies no fingerprint authority. Missing evidence refuses child launch while direct native execution remains available. The separately authorized executable FileObservation binds deployed bytes, including static SUNDIALS; the fingerprint is not an independent native archive attestation. Child work remains separate from canonical durable study actions and represents independent serial solves, not distributed decomposition.

Native update outcomes distinguish incompatibility before mutation, successful update and failed/uncertain mutation. Mutation failure destroys retained state before returning its original typed cause regardless of rebuild policy; it cannot become a boolean structural refusal. HiGHS bulk updates and optional bounded successful-upload witnesses preserve complete model verification, explicit basis policy and transactional witness publication. Temporary edits restore completely or invalidate.

Selected methods use existing scientific builders and the common policy/evidence boundary. The thin root1d TOMS748 route consumes an admitted scalar interval/continuity/branch contract, charged fallible evaluations and original residual acceptance; library interval-width termination grants no solution permission. Existing exact polynomial/IFT guards remain. Each optional family has an explicit consumed selection or support/trigger outcome. Available typed Ipopt statistics discharge the old C-only timing trigger; diffsol-nl is reconsidered for an actual consumed capability need, not cache dominance.

Same-point square-response factors/actions and bounded SVD/EVD diagnostics use explicit low-level faer Par::Seq execution. Shared factor and multi-RHS reuse preserves exact point/provider/scale/row validity. Known scratch admission and finite opaque factor allowances do not bound private symbolic or numeric fill. Strict complete-memory requests refuse before affected derivative/factor/action work when faer supplies no complete fill bound.

SPQR is not selected: its binding, selected native image and ABI/memory contracts are unqualified. Both consumers retain bounded SVD and explicit large-size refusal. A future separately named sparse-rank tier must state its method/version, threshold/ordering semantics, original-coordinate evidence and qualified binding/ABI/memory owner. QR cannot replace SVD singular modes or silently reinterpret square-response rank tolerance.

### Consequences

Scoped native entry/terminal paths use explicit reset/update outcomes, and generated consumers project the active default. The replacements remove redundant scalar update loops, boolean mutation suppression and expired per-request retention paths within that scope. The common walker, scientific definitions, original assessment, explicit starts and admitted population/active mechanisms remain authoritative.

### Compensating controls

The active default requires a qualified neutral-reset path or enforced fresh fallback for every reachable profile. Same dimensions/nnz are insufficient compatibility. Unknown reset, counters, snapshots, memory and reconstruction remain explicit. RequireReuse never preserves uncertain mutation or silently builds a new backend. Native cancellation, lifetime and branch distinctions remain independent of telemetry and native success strings.

### Confirmation

**Implemented:** scoped reset/fallback, explicit-width, mutation, selected method, child reconstruction and memory-refusal contracts described above. **Tested:** [Plan38c Verification](https://github.com/paul-heyse/pse-arrow/blob/6af92d1dee1ffa0b80dca71e03a63b489f2d940c/docs/plans/38c-native-methods-reuse-and-parallelism.md#verification) owns the exact selected commands, conditions and results, including the maintained fork's application-reset controls; admitted native-solvers controls for Ipopt sequences, KINSOL build/setup, SCIP resources, cold POUNCE-convex batches, HiGHS mutation and strict/same-point response; and authored scalar method controls. These tests do not establish representative comparative performance, broader scientific equivalence or complete child/runtime/assembled qualification. [Plan38c Outcome](https://github.com/paul-heyse/pse-arrow/blob/6af92d1dee1ffa0b80dca71e03a63b489f2d940c/docs/plans/38c-native-methods-reuse-and-parallelism.md#outcome) owns the remaining scope and support decisions; assembled scientific/native/durable qualification remains Plan38g. ADR status remains proposed.

## Pros and cons

Neutral reuse removes avoidable repeated work while keeping independent scientific requests independent. Persistent residency and transitive native scopes require explicit affinity, pressure eviction and teardown. The selected library seams preserve ownership rather than creating a second scheduler or numerical implementation.

## More information

[Plan38c](https://github.com/paul-heyse/pse-arrow/blob/6af92d1dee1ffa0b80dca71e03a63b489f2d940c/docs/plans/38c-native-methods-reuse-and-parallelism.md) owns implementation and investigation outcomes; [Plan38](https://github.com/paul-heyse/pse-arrow/blob/6af92d1dee1ffa0b80dca71e03a63b489f2d940c/docs/plans/38-efficiency-and-solver-target.md) owns shared finding dispositions. ADR-0184 owns governing policy and attempt/replay meaning. ARC01 and the accepted rule rows remain the decision boundary; historical review observations are leads, not current qualification.

## Status history

- 2026-10-13 — proposed before dependent implementation under maintainer-authorized full Plan38c execution, including explicit fork-control and ordinary-default selections. Status transitions remain on the decision-PR route.
- 2026-10-13 — recorded scoped implementation and targeted verification, activated AllowRebuild with explicit reset/fallback, and retained child, native-memory and rank support limits; status remains proposed.
