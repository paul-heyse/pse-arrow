---
id: ADR-0102
title: Admit discrete decisions, disjunctive programs and global certification into the design target
status: accepted
date: 2026-09-27
deciders: [paul-heyse]
level: decision
principles: [AP-03, DP-13, DP-22, PS-06, PS-09, PS-12]
blueprint: [§3.3, §9.5, §18.9, §19.7, §25]
review: git:f57b71d56f6eb2c319c4340d6f26abc6a1dc5abc:docs/design_review/reviews/design_review_plan22-target_2026-09-27.md#decision
evidence: Proposed
supersedes: []
superseded-by: null
revisit: The Q1 gap-closure measurement shows SCIP cannot certify any of the named certify fixtures (binary TPD, small regression, heater_optimization) within its recorded budget, or an authored discrete construct cannot be lowered to a class that HiGHS or SCIP supports.
verification: Architecture scenarios S10, S11, S12 and S16 and review scenarios S01 and S06, settled by the Plan 22 tests authored_milp_routes_to_highs, small_synthesis_minlp_optimal, gdp_hull_and_bigm_same_optimum, certify_known_global_optimum, relaxed_export_bound_only and tpd_detects_known_instability; the architectural argument is slots 4 and 6 of the Plan 22 target review.
standard: core-3.0/process-simulator-1.1
scenarios: [docs/plans/22-solver-capabilities-architecture.md#s10, docs/plans/22-solver-capabilities-architecture.md#s11, docs/plans/22-solver-capabilities-architecture.md#s12, docs/plans/22-solver-capabilities-architecture.md#s16, docs/design_review/reviews/design_review_solver-capabilities_2026-09-27.md#s01, docs/design_review/reviews/design_review_solver-capabilities_2026-09-27.md#s06]
---

# ADR-0102: Admit discrete decisions, disjunctive programs and global certification into the design target

## Context

The architecture places global MINLP outside the design target ([§25](../authoritative_design/sections/scope-and-open-design.md#section-25)
recorded limits), lists global MINLP and disjunctive programs as "outside the matrix"
(blueprint §18.9), has no optionality construct (§19.7), does not admit general global MINLP
(§3.3), and refuses any claim of global phase stability (§9.5, §25). The authored compiler
emits only continuous variables, so the MILP class that §18.9 lists is reachable only by
internal tear and diagnostic formulations ([F08](../design_review/reviews/design_review_solver-capabilities_2026-09-27.md#f08)).

The solver capability review found SCIP of no benefit to the classes solved today and the
right library for three missing capabilities: global bounds and gap certification, global
infeasibility proofs with a nonlinear IIS, and MINLP/GDP ([§8.1](../design_review/reviews/design_review_solver-capabilities_2026-09-27.md#81-scip-the-maintainers-hypothesis)).
It named triggers. On 2026-09-27 the maintainer fired them: integer variables will be needed
in the solve, and every functional capability the review found missing is to be added
([Plan 22](https://github.com/paul-heyse/pse-arrow/blob/f57b71d56f6eb2c319c4340d6f26abc6a1dc5abc/docs/plans/22-solver-capabilities.md#context)).

## Scope

This record **amends the target**; its level is `decision`, not a deviation. It binds which
problem classes, intents and claims the design serves. The mechanisms belong to sibling
records: the domain facet (ADR-0103), constraint forms and lowerings (ADR-0104), the SCIP
representation and adapter (ADR-0105), the outcome vocabulary (ADR-0106) and the durable
execution that long and many-case work needs (ADR-0112).

Outside the target, not merely unimplemented: GPU execution; distributing a single solve
across processes or hosts (MPI, domain decomposition); general higher-index or
variable-layout DAEs; interval-rigorous (verified) global optimization beyond the backend's
tolerance-based bounds and exact rational MILP; HSL (ADR-0108).

## Drivers

- **Functional target (profile).** Price-taker and multiperiod unit commitment (S10),
  superstructure synthesis over nonlinear alternatives (S11), certification of a small
  nonconvex design and of phase stability (S12), and the IDAES 2.13 applications that
  author discrete decisions (MatOpt, grid integration; ADR-0097).
- **PS-09.** The class is derived from the model and analysis mode; a class the configured
  solvers cannot handle is refused before solving. Admitting a class means naming its owner.
- **PS-12 and DP-22.** A global claim is a different claim from a local one and must say
  under which box, tolerance and representation fidelity it holds.
- **PS-06.** Discrete modes are one of the three declared formulation policies for phase
  appearance and discontinuities, beside smoothing and complementarity.
- **DP-13 and G8.** Branch-and-bound, spatial branching and cutting belong to an established
  solver; the simulator composes them.
- **AP-03.** The new capability must enter through the backend-execution adapter (Plan 22
  A2) so that S18 stays additive.

## Options

| Option | Assessment | Selection |
|---|---|---|
| Keep discrete and global work outside the target (baseline) | Honest today (§25 states the limits), but fails the maintainer's direction and scenarios S10–S12; IDAES users reach GDP and MINLP through Pyomo | Rejected |
| Admit the classes with HiGHS for MILP and SCIP as the single global and MINLP owner, an explicit `Certify` intent, authored domains and disjunctions, and a relaxation-soundness rule | One owner per class; additive through the adapter table; truthful assurance vocabulary | **Selected** |
| Couenne or Bonmin for MINLP | Duplicates SCIP. Couenne needs NL/ASL, which D12 excludes; Bonmin has no C API and is heuristic on nonconvex problems | Rejected |
| Bespoke outer approximation or enumeration over HiGHS and Ipopt | Reimplements solver machinery (DP-13, G8) and gives no global bound | Rejected |
| POUNCE-convex SOS bounds as the global route | Polynomial problems only, floating-point SDP, no exp/log; kept as a labelled non-rigorous bound (ADR-0109) | Rejected as the global owner |

## Outcome

1. **Classes in the target.** Mixed-integer linear programs authored through the domain
   facet; mixed-integer quadratic programs (convex and nonconvex); mixed-integer nonlinear
   programs; generalized disjunctive programs, which lower by a declared realization to one
   of the mixed classes before routing and are never a routing class themselves; nonconvex
   QP and NLP under global certification.
2. **Derived, never hinted.** The class comes from compiler facts: declared domains, lowered
   disjunctions, degree and convexity evidence. An unsupported combination is refused before
   execution with every reason.
3. **Global claims are explicit.** Global certification is the `Certify` intent. It is never
   an automatic upgrade of a local solve. Automatic routing of MIQP and MINLP to SCIP yields
   the backend's assurance and no more than the export fidelity supports.
4. **Finite boxes.** Every integer, semi-continuous or semi-integer variable, and every
   variable entering a global route, has finite case bounds. Admission refuses otherwise and
   names the variable.
5. **Relaxation soundness.** A relaxed export supports a dual bound and a global
   infeasibility conclusion, never a solution claim. Every candidate is re-qualified in
   original coordinates; for mixed classes with relaxed rows the candidate is produced by a
   fixed-assignment continuous re-solve (ADR-0105).
6. **Global phase stability** enters the target as a tangent-plane-distance check over the
   certify route, for equations of state whose rows project exactly (Plan 22 G6).
7. **Durable multi-process execution** of this work through the operational store enters
   the target (ADR-0112). Distributing one solve does not.
8. **Owners.** HiGHS stays the automatic owner of LP, MILP and convex QP, DegeneracyHunter
   and tear selection. SCIP owns MIQP, MINLP, `Certify`, global infeasibility and the
   nonlinear IIS, and is an explicit alternative for linear and mixed-linear classes.

### Consequences

- §25, §18.9, §19.7, §3.3 and §9.5 change their target statements now. Capability matrices
  and implementation descriptions change only when each Plan 22 packet lands.
- New vocabulary (ADR-0106) and a new library-neutral representation, `FactorableProgram`
  (ADR-0105), are genuine new core concepts, not an ordinary extension.
- Global certification is small-scale by nature. Authors must declare finite boxes; many
  rigorous-thermodynamics rows export only as relaxations until their implicit residuals are
  exported (census P1).
- The solver image grows (ADR-0108).

### Compensating controls

Explicit intents; typed refusal before execution; the relaxation-soundness rule;
original-coordinate qualification of every candidate; assurances that state their
tolerances (ADR-0106); negative controls for relaxed exports.

### Confirmation

The design argument is the Plan 22 target review. Implementation is confirmed per scenario by
the tests named in `verification:`, owned by the Plan 22 packets M1, M4, G4, G6 and G7.
Acceptance of this record certifies none of that work.

## Pros and cons

Admitting the classes makes the simulator competitive with equation-oriented tools that
offer MINLP, and one owner per class avoids duplicate engines. The cost is a large new
surface (authoring, registry, transformation, qualification and a native build), and the
value of global certification on process models remains *Proposed* until Q1 measures it.

## More information

- Architecture companion: [§1–§3, §5](../plans/22-solver-capabilities-architecture.md).
- Capability review: [§8.1](../design_review/reviews/design_review_solver-capabilities_2026-09-27.md#81-scip-the-maintainers-hypothesis), [F08](../design_review/reviews/design_review_solver-capabilities_2026-09-27.md#f08), census P1 (slot 10).
- Target review: [decision](https://github.com/paul-heyse/pse-arrow/blob/f57b71d56f6eb2c319c4340d6f26abc6a1dc5abc/docs/design_review/reviews/design_review_plan22-target_2026-09-27.md#decision), findings [T07](https://github.com/paul-heyse/pse-arrow/blob/f57b71d56f6eb2c319c4340d6f26abc6a1dc5abc/docs/design_review/reviews/design_review_plan22-target_2026-09-27.md#t07) and [T09](https://github.com/paul-heyse/pse-arrow/blob/f57b71d56f6eb2c319c4340d6f26abc6a1dc5abc/docs/design_review/reviews/design_review_plan22-target_2026-09-27.md#t09).
- Disposition and packets: [Plan 22](https://github.com/paul-heyse/pse-arrow/blob/f57b71d56f6eb2c319c4340d6f26abc6a1dc5abc/docs/plans/22-solver-capabilities.md).

## Status history

- 2026-09-27 — proposed (Plan 22 D0).
- 2026-09-27 — accepted under the maintainer's authorization of the full Plan 22 scope (2026-09-27), after the [Plan 22 target review](https://github.com/paul-heyse/pse-arrow/blob/f57b71d56f6eb2c319c4340d6f26abc6a1dc5abc/docs/design_review/reviews/design_review_plan22-target_2026-09-27.md#decision) returned Accept (author review, Proposed evidence level). Findings T07, T09 were corrected in this record before acceptance.
