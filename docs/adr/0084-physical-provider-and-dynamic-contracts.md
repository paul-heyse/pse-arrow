---
id: ADR-0084
title: Separate physical provider validity from derivative and dynamic eligibility
status: accepted
date: 2026-09-24
deciders: [paul-heyse]
level: decision
principles: [DM-07, DM-22, DM-24, DM-43]
blueprint: [§D9, §9, §13.6]
review: docs/design_review/reviews/design_review_math-foundation-enhancements_2026-09-24.md
evidence: Tested
supersedes: [ADR-0022, ADR-0043]
superseded-by: null
revisit: The selected physical model requires a property or DAE capability outside the admitted profile.
verification: Plan 14 M00–M21 targeted units and M22 qualification
---

# ADR-0084: Separate physical provider validity from derivative and dynamic eligibility

## Context

The maintainer approved Plan 14 and its hard M00–M05 cutover. The two math reviews
identify competing mathematical ownership and incomplete library composition contracts.

## Scope

The approved M21 extension makes property operating envelopes and physical balance
contributions typed source contracts. Outside-envelope evaluation is a recoverable
trial failure, without silent extrapolation. Declared operating bounds do not establish
empirical accuracy. Generated balances and independent physical closure share source
contributions, not solver residuals. Diffsol output integration owns cumulative flux
quadrature, including finite event/restart accounting. Public declaration/result schemas
change without a compatibility route. See [M21 execution](../plans/14-m21-execution.md).


The concrete replacement text is prepared in
[Plan 14 foundation contract](../plans/14-math-foundation-contract.md#concrete-decision-amendment-prepared-for-the-design-pr).
Blueprint revision 51 applies it under the local authorization in ADR-0087.
ADR-0087 authorizes this milestone's local reconciliation in blueprint §0.5.
Superseding ADR-0022 removes the obsolete num-dual 0.15 prerequisite; the compatible
workspace pin is authoritative. ADR-0043's typed failures and implemented-only
bindings are preserved, with the removed Pyomo/NL bindings withdrawn.
This record describes the implemented M00–M18 physical foundation, native
execution, dynamic profile and fitting contracts. Local qualification is recorded in the M22 packet under ADR-0087.
Displaced accepted decisions are reconciled through explicit supersession.

## Drivers

Library-owned algorithms, physical meaning, explicit failure and bounded repeated compilation.

## Options

Retaining the old engine would preserve conflicting authority and is rejected.
The selected native library composition is qualified by narrow executable controls.

## Outcome

Provider contracts include physical units/basis/reference, component/data identity, phase, derivative order and typed failure. FeOS 0.10.1 and num-dual 0.14.2 implement the selected explicit-density property profile. Dynamics is limited initially to native ODE/index-1 mass-matrix equations.

The implemented M06–M08 packet selects an explicit-density methane/ethane/propane
PC-SAFT/DIPPR package. Pressure is a model equation output. NPT density iteration
and stability diagnostics are separate initialization operations; their phase
initializers do not establish globally smooth phase selection. Provider requests
declare outputs and raw derivative order, with complete contract keys and local
cancellation/allocation controls. Symbolica generates composition derivatives;
Numerica jet coefficients are explicitly converted at the raw-partial boundary.

### Consequences

M11–M16 implement native algebraic solving and its public workflow. M17–M18 extend
the same mathematical and lifecycle owners with Diffsol BDF on a fixed diag(I,0)
mass profile and ordinary steady/transient fitting. Library consistent initialization,
root location, interpolation and smooth forward sensitivities are selected; general
implicit DAE, hybrid gradients and uncertainty claims remain outside this profile.
No old code is retained as a fallback or as historical evidence.

Diffsol's infallible Rust operators use a private contained unwind signal for typed
provider failure or cancellation. The native operation catches the signal and discards
its solver before any C/Python boundary. Neither fabricated numerical outputs nor a
global panic-hook change is permitted. Native integration inside fitting runs inline
under the outer job's admission, without nested executor permits.

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

The [M21 implementation review](../design_review/reviews/design_review_m21-design-closure_2026-09-24.md)
assesses the final contract/deletion changes at mechanism scope. It does not supply
independent M22 acceptance.

[Execution packet](../plans/14-m00-m05-execution.md) and
[main plan](../plans/14-library-owned-process-simulator.md).

## M22 local qualification

**Tested and Measured:** the [M22 packet](../plans/14-m22-execution.md#verification)
records local Linux functional Q01–Q17, the 23 cached-development case-cost workloads,
zero required failures and retained-origin conditions. It distinguishes admitted memory
allowances from measured pool/RSS observations and excludes Rust build time.

The [independent final review](../design_review/reviews/design_review_m22-scientific_2026-09-24.md)
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

- 2026-09-24: M17–M18 dynamic and fitting contracts implemented; complete native
  controls, physical derivative coordinates, source/result schemas and the generated
  FeOS factory are described in the [packet](../plans/14-m17-m18-execution.md).
  Analytical targeted controls are distinct from the outstanding physical M22 journeys.

- 2026-09-24 — accepted for local Linux M22 scope under ADR-0087 after independent final review; blueprint revision 51 reconciles the contracts. No remote or release qualification is claimed.
