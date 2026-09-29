---
id: ADR-0119
title: Declare analysis selections in kernel fixtures
status: accepted
date: 2026-09-28
deciders: [paul-heyse]
level: decision
principles: [PS-11, AP-04, AP-05, DP-02]
blueprint: [§6.10, §13.5, §13.6, §19.2]
review: docs/design_review/reviews/design_review_solver-scope-decisions_2026-09-28.md#adr-0119
evidence: Proposed
supersedes: []
superseded-by: null
revisit: An authored analysis needs a selection the fixture grammar cannot express, so that a runtime- or Python-only input would be needed again (for example an input that is not piecewise constant, or an event that changes the state layout).
verification: Architecture scenarios S12 and S14 and review scenario S09, settled by the Plan 22 tests. G6r fixture_intent_selects_certify, fixture_policy_intent_conflict_refused, objective_bound_check_uses_certified_bound, tpd_certifies_stable_feed and tpd_detects_known_instability. Y0c scheduled_input_sensitivities_cross_changes (Diffsol and IDAS against finite differences) and kernel_fixture_schedules_inputs. Y0d authored_directional_event_routes_to_idas, idas_sign_constraints_from_authored_bounds, simultaneous_route_refuses_authored_events, and the kernel events test rewritten on the fixture.
standard: core-3.1/process-simulator-1.1
scenarios: [docs/plans/22-solver-capabilities-architecture.md#s12, docs/plans/22-solver-capabilities-architecture.md#s14, docs/design_review/reviews/design_review_solver-capabilities_2026-09-27.md#s09]
---

# ADR-0119: Declare analysis selections in kernel fixtures

## Context

An authored fixture carries case values, bounds, expectations and some analysis choices
(blueprint §6.10). Several selections still reach the kernel only through runtime or Python
inputs:
- **Solve intent.** The `certify` intent (ADR-0106) is selectable only through the runtime policy. The G6 kernel-gap note records that an `annotation check` cannot run a certify solve, and that fixtures cannot select the intent.
- **Scheduled inputs.** Profile `changes` (`InputChange`) replace the parameter vector at fixed times. They switch the changed parameters' sensitivities off (`parameter_active`), which blocks adjoints, shooting and NMPC. Authored schedules have no kernel fixture field.
- **Events and modes.** These are the runtime- and Python-only `ModelingDynamicMode` and `ModelingDynamicEvent` inputs and `FitProfile.modes`. Authored events detect crossings in either direction only.
- **IDAS sign constraints.** These come from a per-state vector in the IDAS profile (`IdasSettings.constraints`, in state order), not from authored bounds.

§13.6 records these as follow-ups. The solver scope packet adopts improvements I6 and I7.

## Scope

A kernel contract within [ADR-0106](0106-execution-vocabulary-discrete-and-global.md) (the
intent vocabulary) and [ADR-0110](0110-dynamics-profile-extensions.md) (schedules, events and
IDAS constraints). Neither is superseded. The record binds which analysis selections are
fixture data, how they conflict with runtime policy, and how an objective-bound check reads
a certified bound. Grammar details and frame versions belong to the implementation (Plan 22
G6r kernel, Y0c and Y0d).

Governs blueprint §6.10 (fixture analysis choices), §13.5 (schedules, events and modes),
§13.6 (IDAS constraints and the recorded follow-ups) and §19.2 (check results).

## Drivers

- **PS-11.** Analysis modes are declared over the same model and case. Dynamic modes declare their event handling and integration policy, and a scheduled input is part of the analysis rather than a side channel.
- **AP-04 and DP-01.** Each selection has one authoritative place: the authored fixture. A runtime policy may repeat it but never overrides it silently.
- **AP-05 and DP-02.** Intent, schedules, events, modes and check basis are typed, declared structure. They are not an untyped runtime struct keyed by paths or state order.
- **PS-12.** A check that claims a global property states the basis it was established on.

## Options

| Option | Assessment | Selection |
|---|---|---|
| Keep runtime- and Python-only inputs | Selections live outside the authored model, so fixtures cannot state a certify check, a directional event or a schedule. Every caller re-supplies them | Rejected |
| A check that starts its own certify solve | Analysis selection moves inside a check, and a check becomes an effect (PS-11, G4) | Rejected |
| Precedence between the fixture's intent and the runtime policy | Two authorities, silently reconciled | Rejected; a conflict is a typed refusal |
| **Fixture clauses for intent, schedules, events and modes; checks read a certified bound the step already has** | One authored declaration per selection, typed, with no hidden solve | **Selected** |

## Outcome

1. **Solve intent.**
   - A fixture may declare `intent certify;`.
   - A fixture without the clause leaves the intent to the runtime policy.
   - When the runtime fixture policy (`ModelingFixturePolicy`) names a different intent, the fixture is refused with a typed reason naming both. There is no precedence rule.
2. **Integration schedules** (I6).
   - A fixture's integration declares `schedule u at(...) values(...)`.
   - Each schedule interval's value is its own parameter, so forward and adjoint sensitivities stay live across every change.
   - One `Profile::parameters_at(t)` maps the interval parameters for both integrators.
   - `InputChange`, `Profile.changes` and `parameter_active` are deleted, and the dynamic profile frame is versioned.
3. **Events and modes** (I7).
   - A fixture declares `mode` blocks (the Boolean facts that select `when` variants) and events: `event guard direction tolerance reset(...) next(...)`.
   - The registry enum `EventDirection` replaces `Crossing`.
   - These declarations replace `ModelingDynamicMode` and `ModelingDynamicEvent` as inputs, `FitProfile.modes`, `declared_simulation_modes`, and the Python mode and event settings (`modes=`). Fitting reads an experiment's modes from its authored case.
   - Routes keep their ADR-0110 limits: a directional event routes to IDAS, and IDAS refuses events with forward sensitivities. The simultaneous route refuses authored events.
4. **IDAS sign constraints.** A constant-zero bound annotation on a state or algebraic variable yields its IDAS sign constraint (`IDASetConstraints`).
   - The authored annotations are the only source. The native per-state vector is derived from them, and is no longer a runtime or Python setting (review F03).
   - The bound's guard remains the validity authority. A sign constraint only keeps the integrator's steps inside the domain.
5. **Objective-bound checks** (G6 kernel gap).
   - The compiler classifies an `annotation check` that bounds the step's objective from the optimized side: from below when minimizing, from above when maximizing.
   - When the step carries `global_bound` or `exact_certificate` (ADR-0106), such a check is evaluated against the step's certified dual bound. Otherwise it is evaluated at the point.
   - `runtime.modeling_checks.basis ∈ {point, global_bound}` records which. A `global_bound` result is scoped to the box, tolerances and export fidelity that the step's gap evidence records; its rigour is the step's assurance.
   - A `point` result never states a global property. A consumer that needs one reads `basis`.
   - A check never starts a solve.

### Consequences

- The fixture grammar, the registry and `just codegen` change together. The Python surface loses its mode, event and sign-constraint inputs, and `just python-stubs` follows.
- The PETSc PID parity case and the p09 fit move to authored schedules.
- A certify-backed stability check needs its fixture to declare `intent certify;`. Without it, the check runs at point basis and says so.

### Compensating controls

- The typed intent-conflict refusal.
- `runtime.modeling_checks.basis`.
- Refusal of authored events on the simultaneous route.
- Finite-difference comparison of sensitivities across scheduled changes.

### Confirmation

The tests named in `verification:`, run in Plan 22 G6r (kernel and tests), Y0c and Y0d.
Acceptance does not certify that work.

## Pros and cons

Fixtures become the one place a reader finds every analysis selection, and a check stays
free of effects. The cost is a wider fixture grammar and the deletion of the programmatic
inputs that scripted callers use today.

## More information

- Solver scope packet [I6 and I7](https://github.com/paul-heyse/pse-arrow/blob/f57b71d56f6eb2c319c4340d6f26abc6a1dc5abc/docs/plans/22-solver-scope-execution.md#improvements-over-the-target-design), the "rejected" list, and packets G6r kernel, G6r tests, Y0c and Y0d.
- [Change-tier review](../design_review/reviews/design_review_solver-scope-decisions_2026-09-28.md#adr-0119), finding F03.
- The G6 kernel-gap note in the [main execution packet](https://github.com/paul-heyse/pse-arrow/blob/f57b71d56f6eb2c319c4340d6f26abc6a1dc5abc/docs/plans/22-solver-capabilities-execution.md).
- Related: ADR-0101 (proposed; facts and annotations), ADR-0102, ADR-0105, ADR-0106, ADR-0110, ADR-0120.

## Status history

- 2026-09-28 — proposed and accepted under the maintainer's authorization of Plan 22's full scope, and of the solver scope packet's improvements (2026-09-28), after the [change-tier review](../design_review/reviews/design_review_solver-scope-decisions_2026-09-28.md#adr-0119) returned Accept (author review, Proposed evidence level). Finding F03 was corrected in this record before acceptance.
