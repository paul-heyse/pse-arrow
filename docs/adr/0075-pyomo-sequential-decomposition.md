---
id: ADR-0075
title: Select flowsheet tears through Pyomo at run time and withdraw compiler tear selection
status: proposed
date: 2026-09-23
deciders: [paul-heyse]
level: decision
principles: [DM-05, DM-07, DM-13, DM-19, DM-28, DM-40, DM-43, DM-59]
blueprint: [§3.1, §3.2, §3.3, §6.7, §6.11, §6.13, §6.15.5, §12.5, §14.1, §17.1, §17.4, §21.3, §23.2, §25]
review: docs/design_review/reviews/design_review_pyomo-sequential-decomposition_2026-09-23.md
evidence: Interface-checked
supersedes: []
superseded-by: null
revisit: The follow-up ADR that runs plan.sequential_modular@1 through Pyomo SequentialDecomposition and supersedes ADR-0015 is opened (register R-32); Pyomo changes select_tear_heuristic or pyomo.network's graph API; or a `plan.tear_selection` cycle-budget refusal occurs on a supported flowsheet.
verification: Pyomo-adapter unit tests for `select_tears` (added with the implementation) — heuristic tear set equal to Pyomo's `select_tear_heuristic` first set on the Slice B recycle topology; `mip` and `weighted` objectives and acyclicity checked, including a solver returning 0.9999999; `plan.tear_selection` refusals (cycle budget, non-optimal MILP, cyclic result); a plan test that a changed `tear_method` or `tear_cost` takes effect on the next plan run; `authored.connections.tear_cost` row-check fixture; `governance / adr-lint`.
---

# ADR-0075: Select flowsheet tears through Pyomo at run time and withdraw compiler tear selection

> Current direction: the approved Plan 14 hard pivot withdraws this proposed Pyomo
> route. ADR-0083 and M14 own native tear/initialization execution. This record retains
> its proposed status pending the formal decision PR; it is not an execution instruction.

## Context

Blueprint §12.5 had pass P5 compute `inferred.tear_candidates` in Rust. There were two
methods:

- `heuristic`: petgraph's `greedy_feedback_arc_set`.
- `mip`: "the minimum feedback arc set through the NL backend and a MILP solver".

Both claims were wrong, as the pinned Pyomo 6.10.1 source and IDAES 2.12.0 show:

- **The real objective is different.** FOQUS and Pyomo minimise the maximum number of tears
  on any elementary cycle, then the number of tears, unweighted.
  - `select_tear_heuristic` is an exact branch-and-bound for that objective, and it is what
    IDAES uses.
  - `select_tear_mip` states the same objective as a MILP and defaults to CPLEX.
- **petgraph's greedy method** is an Eades–Lin–Smyth approximation. It does not reproduce
  FOQUS's choice, although §12.5 said it did.
- **No Rust library** computes an exact minimum directed feedback arc set (Plan 12 outcome).

The maintainer's direction is that solver approaches are swapped in Pyomo, not in Rust.

## Scope

This record amends the blueprint and is a `decision`. It binds four things:

1. **Withdraw compiler tear selection.** P5 no longer selects tears, and
   `inferred.tear_candidates` is withdrawn.
2. **Correct §12.5.**
3. **Select tears at run time.** A `select_tears` plan stage selects them over
   `inferred.topology_edges` through the Pyomo adapter. The stage's `tear_method`, MILP
   profile and `max_cycles` are columns of `compiled.init_stages`, and the result is
   recorded in keyed runtime relations bound to that stage's run.
4. **Move the cost check.** A row check on `authored.connections.tear_cost` replaces the one
   in `tears.rs`.

`select_tears` uses Pyomo as a Pyomo-ecosystem tool: `pyomo.network`'s FOQUS code and
`SolverFactory` MILPs. D12 already admits that use.

`plan.sequential_modular@1` stays a native plan that consumes a selection run. Executing it
through `SequentialDecomposition` is deferred to a follow-up ADR (register R-32), together
with the unit, port and arc bundle tables and the supersession of ADR-0015 (D12) that it
would need.

## Drivers

- **Parity.** The `heuristic` method *is* Pyomo's FOQUS code, which is what IDAES uses.
- **Swappable solvers.** `mip` and `weighted` go through `SolverFactory` with a named MILP
  solver profile, so changing the solver changes a profile, not Rust.
- **No bespoke algorithm.** No exact Rust method exists, and writing one would duplicate a
  maintained implementation.
- **Remove the false claims** in §12.5.

## Options

1. **Keep P5 selection and add an exact Rust `mip`.** Rejected, for three reasons:
   - no crate provides it (`problemreductions` is an i32-only MTZ model over HiGHS);
   - its objective differs from Pyomo's;
   - it fixes the solver in Rust.
2. **Port FOQUS's branch-and-bound to Rust.** Rejected: it duplicates maintained Pyomo code,
   and the solver would still be fixed in Rust.
3. **Run the whole `plan.sequential_modular@1` through `SequentialDecomposition` now.**
   Deferred (R-32). The review found that it needs:
   - D12 superseded;
   - network tables at the boundary, with P8 staying authoritative for connection equations;
   - an outcome contract for silent non-convergence;
   - `Port.Extensive` reconciled with §12.2;
   - a `networkx` pin;
   - unit-plan lowering.
4. **Select tears in Pyomo over the unit graph P5 already persists.** Chosen: it gives the
   same tear-selection parity with no boundary change.

## Outcome

`select_tears` builds a `networkx.MultiDiGraph` from `inferred.topology_edges`, with nodes
and edges in semantic-ID order, and applies the plan's tear method:

| Method | Selection | Determinism |
|---|---|---|
| `heuristic` (default) | Pyomo's `select_tear_heuristic`; the first of its equally good sets | `edge_order` |
| `mip` | one platform Pyomo MILP over the elementary cycles, objective `(|E|+1)·max_cycle_tears + Σ x_e` (Pyomo's constant 1000 replaced so it stays lexicographic at any size) | `solver_dependent` |
| `weighted` | the same MILP, objective `Σ tear_cost·x_e` (a null cost counts as 1) | `solver_dependent` |

Rules for every method:

- **MILP solver.** The MILP uses a named MILP profile, never Pyomo's `cplex` default. A
  termination other than proven optimal is refused, and binaries are read with a 0.5
  threshold.
- **Acyclicity.** Every selected set is validated to leave the graph acyclic.
- **Cycle budget.** Cycle enumeration is bounded by `max_cycles` (default 10 000).
- **Refusals.** All three refusals are the failure class `plan.tear_selection` (§23.2).
- **Acyclic unit graphs.** The stage records an empty selection natively, without Python.
- **Results.** Each run writes `runtime.tear_selection_runs` (keyed by `run_id`) and
  `runtime.tear_selections` (keyed by `run_id, connection_id`).

`plan.sequential_modular@1` runs `select_tears` as its stage 0 and uses the tears of that
stage's own run, never a selection from another run. A change of method, profile, costs or
topology therefore always takes effect. The stage run is a `runtime.runs` row: backend
`pyomo`, or `native` for an acyclic graph; its solver profile is the MILP profile, or the
built-in `tear.foqus_heuristic` profile for `heuristic`.

### Consequences

- **Deletions.**
  - `crates/pse-compiler/src/passes/p5/tears.rs` and its wiring in `p5.rs`.
  - The `inferred.tear_candidates` declarations in `crates/pse-schema/src/catalog/`, and
    their generated Rust and Python contracts (`just codegen`).
  - The tear assertions in `tests/engine/tests/finite_semantic_inference.rs` and the
    pse-rules test support.
  - The conformance fixture `tests/conformance/fixtures/invariants/`
    `inferred-tear_candidates-unique-pk/`, and its generated invariant case.
- **Enums.** The single-variant `TearMethod` becomes `heuristic|mip|weighted`.
  `TearDeterminism`, the `select_tears` stage kind and the `plan.tear_selection` failure
  class are added.
- **petgraph.** It loses its only current code use. The blueprint keeps it for the topology
  graph and SCC/BTD (§15.3), so its dependency entry must be justified by those use sites or
  removed until they land.
- **Tears are now a per-run result.** Two MILP runs may pick different, equally good sets;
  that is declared in the `determinism` column.
- **`networkx` pin.** It must be pinned in the `pyomo` extra, and currently is not. Until
  then `select_tears` is refused at pre-flight.

### Compensating controls

- **Acyclicity check** before any row is written.
- **Graph binding.** `graph_hash` ties each selection to the topology it was computed for.
- **Typed refusal** (`plan.tear_selection`) for a cycle-budget overrun, a non-optimal MILP
  or a cyclic result.
- **Declared determinism class** per method.
- **Pre-flight** for `networkx`, `numpy` and the MILP profile.
- **R-32** for the deferred execution.

### Confirmation

The unit tests named in `verification:` confirm this once implemented. Until then the
evidence is `Interface-checked`:

- the pinned Pyomo 6.10.1 `pyomo/network/foqus_graph.py` and `decomposition.py`.
  `pyproject.toml` pins `pyomo==6.10.1`, so after `just py-sync` they are in the dev
  environment's site-packages;
- the review's scratch probe, which ran `select_tear_heuristic` on a plain graph built from
  edge rows.

## Pros and cons

| | Pyomo tear selection over `topology_edges` (chosen) | Full `SequentialDecomposition` execution now |
|---|---|---|
| For | parity for tear selection; swappable MILP solvers; no boundary change; D12 unchanged | parity for convergence as well |
| Against | the sequential plan stays native until R-32 | needs D12 superseded, new boundary tables, an outcome contract and unit-plan lowering |

## More information

- Blueprint §12.5, §17.4, §21.3 and §6.13.
- The design review at the `review:` path. Its verdict was Revise, and its narrower
  Accept-scoped package is what this revision adopts. Its findings on full
  `SequentialDecomposition` execution carry to R-32.
- ADR-0063, whose tear sentence this record replaces; ADR-0015 (D12); register R-32.
- Pinned Pyomo 6.10.1 `pyomo/network/foqus_graph.py` and `decomposition.py`.
- IDAES 2.12.0 hydrodealkylation flowsheet example (`select_tear_method = "heuristic"`).

## Status history

- 2026-09-23 — proposed as full `SequentialDecomposition` delegation; the design review
  returned Revise.
- 2026-09-23 — narrowed to the review's Accept-scoped package, with `SequentialDecomposition`
  execution deferred to R-32. `needs-review` until the review records Accept or
  Accept-scoped.
- 2026-09-23 — re-review returned Revise (minor): `select_tears` became a plan stage that the plan binds by its own run (R1, R2); failure class, run binding, lexicographic weight, acyclic short-circuit, evidence paths and deletion list added (R3–R6).
