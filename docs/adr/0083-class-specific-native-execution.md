---
id: ADR-0083
title: Replace legacy math and Pyomo routes with native class-specific execution
status: accepted
date: 2026-09-24
deciders: [paul-heyse]
level: decision
principles: [DM-38, DM-43, DM-58]
blueprint: [§D12, §18, §21]
review: git:8950dd3d6ddb:docs/design_review/reviews/design_review_unified-native-solvers_2026-09-24.md
evidence: Tested
supersedes: [ADR-0015, ADR-0075]
superseded-by: null
revisit: A concrete problem class needs an additional native interface.
verification: Plan 14 M00–M21 targeted units and M22 qualification
---

# ADR-0083: Replace legacy math and Pyomo routes with native class-specific execution

## Context

The maintainer approved Plan 14 and its hard M00–M05 cutover. The two math reviews
identify competing mathematical ownership and incomplete library composition contracts.

## Scope

The approved M21 extension requires mode-aware structural admission before native
execution, including original residual support for root/fixed-point profiles. NLP
optimization degrees of freedom remain valid. faer verifies implicit response solves;
physical closure observations remain separate from native termination and feasibility.


The concrete replacement text is prepared in
[Plan 14 foundation contract](https://github.com/paul-heyse/pse-arrow/blob/8950dd3d6ddb3aa7c78acc0db7d0601497b302a8/docs/plans/14-math-foundation-contract.md#concrete-decision-amendment-prepared-for-the-design-pr).
Blueprint revision 51 applies it under the local authorization in ADR-0087.
ADR-0087 authorizes this milestone's local reconciliation in blueprint §0.5.
ADR-0015 and the unimplemented Pyomo tear decision ADR-0075 are superseded;
reference parity remains isolated and native class-specific execution is authoritative.
This record describes the implemented M00–M18 foundation, native Rust
solver/integrator lifecycle and public model/result/fitting workflow. Local qualification is recorded in the M22 packet under ADR-0087.
Displaced accepted decisions are reconciled through explicit supersession.

## Drivers

The approved M15–M16 extension is specified in
[the execution packet](https://github.com/paul-heyse/pse-arrow/blob/8950dd3d6ddb3aa7c78acc0db7d0601497b302a8/docs/plans/14-m15-m16-execution.md). It adds library-owned
NLP preprocessing/recovery, independently evaluated physical results, registry-owned
typed public authoring and blocking/async native jobs. Existing exact Delta
publication remains the durable commit boundary. These additions are **Implemented**;
the packet records targeted native/Python contract evidence and excluded terminal
qualification. This implementation evidence does not change the ADR status.

Library-owned algorithms, physical meaning, explicit failure and bounded repeated compilation.

## Options

Retaining the old engine would preserve conflicting authority and is rejected.
The selected native library composition is qualified by narrow executable controls.

## Outcome

Remove old public mathematical and solve routes at the M03–M05 cut. Keep the Ipopt sys boundary. Native NLE/NLP/linear/conic interfaces remain distinct under one solver lifecycle; the two NLP backends share an oracle. Diffsol owns the admitted fixed-mass ODE/index-1 profile. No production Pyomo, compatibility compiler or placeholder solve route.

M08 implements physically checked row/gather maps, persistent faer sparse patterns,
weighted Hessians, separate objective/constraint/gradient/Jacobian requests, and NLE
Jacobian/JVP products. Coefficients retain objective constants/sense, integrality and
parameter assumptions. Exact rational Gram checks establish the initial convexity
witness; explicit Clarabel cones receive format/dimension/parameter admission. Native
solver invocation is implemented in M11–M14, public orchestration/publication in
M16, and the selected Diffsol dynamic profile in M17.

M10 adds one shared runtime math artifact component and a completion-owned worker
boundary. Pure Salsa preparation produces immutable case plans and artifact requests;
DataFusion supplies retention, existing native cache reporting/invalidation is reused,
and shared CPU/pool admission survives cancellation until actual native thread exit.
The M11–M14 adapters consume this boundary.

The approved [M10–M14 execution packet](https://github.com/paul-heyse/pse-arrow/blob/8950dd3d6ddb3aa7c78acc0db7d0601497b302a8/docs/plans/14-m10-m14-execution.md) extends
this boundary into one class-routed solver lifecycle with Ipopt, POUNCE, KINSOL,
HiGHS and Clarabel. It includes semi-variable domains, SDP, finite native solve
sequences, warm-start compatibility, bounded progress and original-model validation.
Native execution remains worker-local; Salsa tracks pure preparation only. The
packet pulls basic warm/reuse contracts forward from M15 while retaining general
presolve/postsolve qualification there. These extensions are implemented and targeted-tested,
including protected callback output, source-space quality and native upload/ABI contracts.
POUNCE uses explicit FERAL and admitted Rayon ownership; KINSOL uses the pinned
serial KLU profile; HiGHS scheduler teardown is exclusive and blocking; Clarabel SDP
uses serial LP64 netlib. Salsa never owns mutable native solver state.

### Consequences

Native solving is implemented through Rust APIs under explicit library features.
M15 adds shared qualified library transformations and independent original-space
observations. M16 adds registry-authoritative model revisions, blocking/async Python
jobs, physical Arrow results and explicit exact publication. Advanced cone and flow/map
construction uses the existing typed Rust services. M17–M18 add generated dynamic,
fitting and selected FeOS provider-factory declarations; the same public joined jobs,
retained Arrow ownership and publication protocol serve these operations. Local scientific, installed workflow and governance evidence is recorded in M22. No old code is retained
as a fallback or as historical evidence.

M18 shares `NlpOracle`, native adapters and library presolve with ordinary solving.
Steady physical constraints use simultaneous coordinates; transient experiments run
Diffsol inline under the outer fit allowance. faer supplies fixed least-squares
products, regular implicit response solves and local rank diagnostics. All-fixed
evaluation remains distinct from native termination. No covariance or global
identifiability claim follows from a solve or local rank.

### Compensating controls

Pinned profiles, pre-normalization physical/domain checks, explicit capability admission,
source-attributed failures and targeted positive/negative unit controls.

### Confirmation

The package-specific units and deleted caller inventory establish implementation;
Plan 14 M22 establishes the scoped local qualification recorded in its execution packet.

## Pros and cons

The change removes duplicate engines and exposes library capabilities. It requires
an atomic caller/schema cut and explicit native/profile qualification.

## More information

The [M21 implementation review](https://github.com/paul-heyse/pse-arrow/blob/8950dd3d6ddb3aa7c78acc0db7d0601497b302a8/docs/design_review/reviews/design_review_m21-design-closure_2026-09-24.md)
assesses the final contract/deletion changes at mechanism scope. It does not supply
independent M22 acceptance.

[Execution packet](https://github.com/paul-heyse/pse-arrow/blob/8950dd3d6ddb3aa7c78acc0db7d0601497b302a8/docs/plans/14-m00-m05-execution.md) and
[main plan](https://github.com/paul-heyse/pse-arrow/blob/8950dd3d6ddb3aa7c78acc0db7d0601497b302a8/docs/plans/14-library-owned-process-simulator.md).

## M22 local qualification

**Tested and Measured:** the [M22 packet](https://github.com/paul-heyse/pse-arrow/blob/8950dd3d6ddb3aa7c78acc0db7d0601497b302a8/docs/plans/14-m22-execution.md#verification)
records local Linux functional Q01–Q17, the 23 cached-development case-cost workloads,
zero required failures and retained-origin conditions. It distinguishes admitted memory
allowances from measured pool/RSS observations and excludes Rust build time.

The [independent final review](https://github.com/paul-heyse/pse-arrow/blob/8950dd3d6ddb3aa7c78acc0db7d0601497b302a8/docs/design_review/reviews/design_review_m22-runtime_2026-09-24.md)
accepts the relevant scoped contracts with no open MUST finding. Companion runtime,
scientific and claims reviews cover G1–G8 and PS-G1–PS-G3. Blueprint revision 51 and
ADR-0087 govern local acceptance. Strict Clippy cleanup and release/remote/platform
qualification remain separate; no broader clean or empirical claim follows.

## Status history

- 2026-09-24: Proposed before the approved implementation; scoped decision review and
  blueprint amendment are part of this work, not an implied whole-plan acceptance.

- 2026-09-24: M00–M05 implementation and targeted controls complete; scoped review
  accepts the foundation. Formal ADR acceptance and blueprint supersession remain pending.

- 2026-09-24: M06–M08 and foundation enhancements implemented; 45 combined targeted
  units pass with explicit force-validation and zero failures. The enhancement review
  accepts this scope. Formal decision acceptance and full M22 qualification remain open.

- 2026-09-24: M09–M10 implemented and scoped-reviewed; 63 targeted force-validation
  units pass with zero failures. Shared workspace compilation passes. Formal decision
  acceptance, blueprint reconciliation and full M22 qualification remain open.

- 2026-09-24: M10–M14 native solver lifecycle implemented. Targeted native/profile
  contracts pass 95 units with force-validation, zero failures; default and full native
  compilation pass. The scoped review accepts this contract boundary. Formal decision
  acceptance, blueprint reconciliation and M22 full qualification remain open.

- 2026-09-24: M15–M16 implementation and targeted native/public-boundary controls
  complete. Publication settlement and whole-plan qualification remain M22; formal
  ADR acceptance and blueprint reconciliation remain pending.

- 2026-09-24: M17–M18 native dynamics and fitting implemented under the approved
  [packet](https://github.com/paul-heyse/pse-arrow/blob/8950dd3d6ddb3aa7c78acc0db7d0601497b302a8/docs/plans/14-m17-m18-execution.md), which records targeted native/Python
  evidence. Scientific qualification, formal decision acceptance and blueprint
  reconciliation remain open.

- 2026-09-24 — accepted for local Linux M22 scope under ADR-0087 after independent final review; blueprint revision 51 reconciles the contracts. No remote or release qualification is claimed.
