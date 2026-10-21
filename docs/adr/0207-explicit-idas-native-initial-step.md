---
id: ADR-0207
title: Expose explicit IDAS native initial-step selection
status: proposed
date: 2026-10-20
deciders: [paul-heyse]
level: decision
principles: [AP-03, AP-07, PS-07, PS-12]
blueprint: [§5.3, §18.10]
review: docs/design_review/reviews/design_review_execution-performance-recommendations-audit_2026-10-20.md
evidence: Tested
supersedes: []
superseded-by: null
revisit: Native automatic selection loses current scope/reset semantics, violates original numerical checks, or exposes a significant complete-operation regression.
verification: Plan48e E06 exercises the production typed setting against configured h0 on smooth/stiff forward-sensitivity ODEs with independent original output/derivative-effect checks and native step/work observations; E01 verifies complete fixed-profile reset.
---

# ADR-0207: Expose explicit IDAS native initial-step selection

## Context

`Profile.initial_step` is positive and supplies both the native attempted first step and
the orientation/scale for consistent initialization. IDAS independently supports automatic
native first-step selection through `IDASetInitStep(0)`. Zero is not a valid replacement
for the positive consistent-initialization orientation scale.

## Scope

Add an explicit IDAS-only typed choice at blueprint §18.10. Native automatic h0 is the
new default; explicit Configured retains the positive attempted native step. Preserve positive
`Profile.initial_step` for IC in both choices. Advance the current IdasSettings document to
version3, the enclosing dynamic Profile document to version3 and its current frame to
`DynamicProfileV12`, so older omission/default meaning is not silently reinterpreted. Neither physical
accuracy nor supplied local integration tolerances changes.

## Drivers

Use the native integrator's current weighted-derivative/sample-horizon estimate instead of
an arbitrary fixed attempted step when no explicit policy is requested. Follow the maintainer's
first-principles adoption preference; retain explicit configured control and distinct IC meaning.

## Options

Retain configured h0 only; expose automatic as an opt-in; or use native automatic selection
by default with an explicit configured alternative. Select the last option under the maintainer's
work-reduction-first steering. Native selection consumes current consistent derivatives and
requested first-output distance, with native hmin/hmax/tstop controls. The smooth fixture has
identical work; the stiff fixture uses fewer residual evaluations and improves original output
classification. This is no universal speed or global-error guarantee.

## Outcome

`IdasSettings.native_initial_step` selects `IdasInitialStep::Configured` or `Automatic`.
The default Automatic leaves fresh native h0 at its library default and explicitly restores
zero on a compatible reused native owner. Explicit Configured uses `Profile.initial_step`
as its attempted physical step. Positive
`Profile.initial_step` continues to orient each IC call. Full profile identity consumes
the choice, so distinct requests cannot share an incompatible retained owner. Diffsol and
all other native methods retain their existing controls. Only the IDAS native initial-step
default changes; practical physical accuracy is unchanged.

### Consequences

The existing profile remains finite and valid. The choice is serialized and schema-visible
through its version3 typed owner and enclosing Profile version3; unknown values and unsupported
old versions refuse. Dynamic profile identities use `DynamicProfileV12`; frozen old frame spellings
retain their historical meaning. New
omission/default chooses automatic; explicit Configured retains its meaning. Current Rust,
Python, replay and schema consumers migrate together without a compatibility shim.

An unresolved study profile (`None`) or omitted fit experiment-profile map requests current
authored defaults on fresh admission; it does not assert a historical concrete native step.
Those generic outer requests keep that meaning. Admission constructs and binds the actual
current Profile/identity before prepared or result reuse; durable replay retains its captured
complete concrete profile. This is not a requirement to version every enclosing request or
prove an old runtime environment. Explicit old concrete profile documents remain refused.

### Compensating controls

Use current actual native step statistics and independently assess physical output and
sensitivity effects. Native success is not a global error certificate. Preserve observed
original-invalid configured trajectories rather than changing their tolerance to make a
comparison pass. Complete fixed-profile reset must restore the actual selected native h0.

### Confirmation

**Tested:** the current native eight-control selection passes automatic/configured initial
steps, current default/version3 refusal, coalesced native statistics across initialization/
reset and retained scheduled DAE/forward-quadrature consumption. The earlier smooth/stiff
comparison keeps rtol/atol0.001: the stiff automatic choice uses41 versus38 accepted steps,
61 versus65 residual evaluations and13 setups for both, while improving original output
classification. No engineering accuracy changes. Plan48e E06 owns exact commands and
conditions; Python receiving and assembled qualification remain with Plan48h.

## Pros and cons

The native capability becomes explicitly usable without a new solver or heuristic. It adds
one local setting, not a universal performance or accuracy guarantee.

## More information

- [Plan48e E06](../plans/48e-dynamics-fitting-and-preconditioning.md#e06--corrector-initial-step-statistics-and-backward-reset)
- Vendored `idas_io.c::IDASetInitStep` and IDAS initial-step selection in `idas.c`.

## Status history

- 2026-10-20 — Proposed before dependent typed-setting implementation under authorized Plan48 execution.
- 2026-10-20 — Amended before automatic-default/version3 migration under explicit maintainer first-principles adoption steering; retain explicit Configured and unchanged IC/accuracy.
