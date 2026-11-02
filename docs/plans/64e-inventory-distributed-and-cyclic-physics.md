---
title: "Plan64e: Inventory, distributed and cyclic physics"
status: draft
date: 2026-11-02
adrs: []
review_sources: [docs/design_review/reviews/design_review_process_simulator_target_2026-11-02.md]
doc_role: plan
doc_owner: docs/plans/64-process-simulator-target.md
doc_topics: [current-work, design, validation]
---

# Inventory, distributed and cyclic physics

## Purpose and basis

**Proposed:** deliver P4's missing stored-material, distributed accumulation and operating-cycle physics through authored definitions and the existing native dynamic owners. The [coordinator](64-process-simulator-target.md) owns the combined target, rule decisions, finding dispositions and authorization. This companion owns E01–E06; document creation does not authorize their execution.

**Interface-checked baseline:** `1acd8c83ca8f9e6f6e9b5e9c9cbf98e299d574b8`, consuming the [integrated review](../design_review/reviews/design_review_process_simulator_target_2026-11-02.md), especially S07/S09/S10, F04 and P4/P5. Core 3.4, efficiency heuristics 1.0, ProcessSimulator 1.5 and test-assessment 1.2 apply. The [P4 roadmap](../process-simulator-design/delivery/implementation-sequence.md#p4--add-the-missing-inventory-and-distributed-physics) and [cross-industry contracts](../process-simulator-design/process/cross-industry-contracts.md) supply the scientific target.

## Existing foundations and consumed boundary

`crates/pse-modeling/src/specialize/process.rs` already retains conserved inventory, signed flux and permitted state-dependent event-transfer identities. Its simultaneous event-bearing conservation refusal is an explicit supported-scope limit. `crates/pse-runtime/src/workflow/modeling/dynamics.rs`, `dynamics_checks.rs` and `dynamics_events.rs` prepare native operations and original physical checks; `workflow/dynamics.rs` owns shared immutable layouts and attempt-local work. These are foundations, not proof of the expanded P4 journeys.

Authored `packages/reference/process/models/control-volumes.pse`, `control-volume-1d.pse` and `vessels.pse` are the first extension surfaces. Current distributed dynamic refusal must survive until temporal accumulation and its original conservation assessment are supplied. The [dynamic architecture](../authoritative_design/sections/workflows-and-results.md#section-13-6) owns numerical routes; [physical semantics](../authoritative_design/sections/physical-semantics.md) owns quantity and conservation meaning.

E consumes the ordinary scientific records and property/state contracts delivered by 64b/64d, plus 64a's task/outcome and identity rules. It supplies actual storage/distributed/cycle contracts to 64f specialized consumers and 64g analyses. It does not wait for every P2/P3 scientific family: the selected vessel or distributed case's complete closure is the prerequisite.

## Target and design decisions

Represent inventory as stored extensive state with explicit phase allocation and constitutive relationships. Flow at a boundary is not inventory; a lot identifier is not a species or a thermodynamic state. No vessel content is inferred by multiplying an instantaneous flow by an arbitrary duration. Element/total projections carry only conservation claims justified by admitted composition and source data.

Separate physical coordinates, geometry, boundary/initial conditions and operating programs from temporal/spatial discretization. Mesh refinement changes the numerical approximation, not the physical stage count. Retain mappings back to original balances and evaluate inventory change against original signed flux and declared event transfers. Discrete conservation and continuum/error evidence are separate claims.

A physical operating step declares active laws/connections, duration or transition condition, retained inventory, allowed transfer/reset and observations. Repeated batches or cycles carry physical state; independent Study nodes do not implicitly transfer inventory. Prescribed schedule execution, conflict checking and schedule optimization are different operations. General production scheduling is not a P4 prerequisite.

Route selection consumes exact index, layout, event, derivative and output-coverage capabilities. Existing native ODE/index-1 and simultaneous routes remain eligible where they fit. A higher-index/general implicit DAE, layout change, simultaneous actionable event, algebraic reset or hybrid derivative request needs an explicit supported realization or attributable refusal. Qualification of one route does not establish another's support.

The review's IDA candidate APIs (`IDAInit`, `IDASetId`, `IDACalcIC`, `IDASolve`) illustrate residual-DAE integration, not automatic index reduction or hybrid sensitivity semantics. Preserve the [review's exact capability limits](../design_review/reviews/design_review_process_simulator_target_2026-11-02.md#8-library-fit-and-alternatives); select an additional integration only if E01 establishes a missing consumed contract. No dependency or solver change is selected here.

## Package state

| ID | Status | Prerequisites | Completion |
|---|---|---|---|
| E01 | planned | P0 finite cases; selected 64d state/property closure | Case-to-physics and route gap decisions identify existing support, authored additions, generic gaps and explicit refusals |
| E02 | planned | E01 selected lumped storage case; ordinary records | Phase/component/element/total storage projections and independent inventory closure work for the selected configurations |
| E03 | planned | E01 distributed case; required state/transfer science | Authored spatial/temporal accumulation, boundary meaning, origin mappings and coupled consumer replace only the corresponding dynamic refusal |
| E04 | planned | E02 or E03 working storage contract; selected operating program | Physical carryover, transitions, transfers and periodic closure work through existing temporal owners |
| E05 | planned | E01 derivative/error requirements; affected E02–E04 implementation | Route-specific derivative, trajectory/error and unsupported-product behavior is qualified without implying unsupported hybrid capabilities |
| E06 | planned | Selected E02–E05 producer and consumer changes | Complete native/reference transient and cyclic cases, retained readback and correct published scope |

## Work packages and investigations

### E01 — Select the first physics and resolve route feasibility

Start with a dynamic vessel/CSTR, one distributed reactor or contactor and one batch/cyclic witness from the finite reference population. Record each case's inventory law, phases, conserved quantities, initial/boundary conditions, input profiles, events and requested products. This is a concrete decision package, not selection of every possible dynamic formulation.

Inspect the current authored and specialized forms against that specification. Produce a gap disposition: existing formulation and route, missing authored science, missing generic semantic contract, or missing native numerical capability. For each gap, name the affected consumer and settling evidence. E02–E05 may proceed only on the corresponding settled contract; a documented candidate solver is not a working prerequisite. F04 preserves prepared-obligation consumption and does not require compiler migration absent a demonstrated conflicting interpretation.

### E02 — Deliver lumped storage and phase allocation

Extend existing authored storage definitions with the required allocation laws and component/element/total projections. Bind pressure, energy, volume and phase constraints according to the selected vessel's actual physical model. A claimed conserved quantity has an authoritative projection; unknown composition does not become zero elemental content. Reuse existing quantity, balance and lineage owners.

Exercise accumulation with zero external flow, selected reaction and internal phase transfer. Internal contributions cancel once in the enclosing balance; interface storage or reaction remains explicit. Distinguish initial physical conditions from numerical guesses. A contradictory initial specification must identify the original model elements before it is mistaken for numerical failure.

### E03 — Deliver distributed temporal accumulation

Implement the selected distributed balance's missing temporal accumulation through authored packages first, extending generic forms only for an E01-proven gap. Preserve coordinate/geometry meaning, inlet/outlet boundary conditions and actual reconstruction coverage. Couple the formulation to a real distributed consumer; do not remove `control-volume-1d`'s dynamic refusal based solely on an added declaration.

Retain integrated original source/flux accounting with consistent discretization. Refine spatial and temporal resolution independently at defined physical output locations. Compare the physical problem and selected approximation, not raw solver-vector ordinals. Existing unsupported configurations continue to refuse until their own closure and evidence exist.

### E04 — Carry physical state through operating programs

Implement prescribed transitions through existing temporal and immutable-case owners. Each step resolves active equations/connections and records retained inventory, declared resets/transfers and event-side output conventions. A closed batch stage requires no invented outlet flow. A periodic request specifies cycle closure and any required phase condition; a visually settled final trajectory is insufficient.

Check material/energy carried between steps and collection intervals over the complete cycle. A numerical retry must not perform a physical transfer twice. If a requested transition violates the current common-layout or event contract, retain a typed refusal or implement the E01-selected extension with affected consumers; do not silently project away state.

### E05 — Qualify demanded derivatives and trajectory products

Choose forward, adjoint or other supported products from the selected route's actual contract. Preserve physical parameter identity and declared sampled intervals. Detect any incomplete derivative/coverage result and retain it as unavailable rather than a completed stronger product. Reuse existing response-layout, work-limit and original assessment owners.

Separate consistency of initial conditions, integration termination, original physical acceptance and estimated numerical error. An estimator or refinement comparison supports only its stated quantity, region and assumptions. Event-sensitive derivatives need their own evidence; ordinary smooth-path derivative evidence does not establish them.

### E06 — Integrate configured transient and cyclic journeys

Run matched native/reference cases only after the selected physics and route contracts are implemented. Compare input profiles, initial conditions, event times, pre/post-event values, reconstructed trajectories and integrated inventory closure. Retained readback identifies the exact definitions, data, route, approximation and requested versus achieved coverage. Cancellation returns truthful partial coverage and retains actual native custody until settlement.

## Verification and completion

**Proposed verification:** owner-local tests first establish storage allocation, signed transfer, boundary and reset semantics; negative controls expose wrong sign, missing phase allocation, double-counted internal transfer, incompatible initial conditions and incomplete coverage. Independent hand-derived storage cases and matched reference trajectories challenge production transformations rather than recomputing expectations through them.

Use the repository's targeted compile/test routes during implementation. Selected assembled scientific/reference journeys and scope-end checks occur at the owning integration boundary, coordinated with 64h and existing Plan58h. No broad campaign repeats per package. Measure cold preparation, reused value-only cases or longer trajectories only for a corresponding performance claim under matched scientific quality.

**Outcome: Proposed.** No E package is implemented or numerically qualified by creating this document; no product tests, solver runs or measurements were performed for its authoring. Completion requires the selected finite P4 configurations, original physical checks, route-specific products and explicit retained exclusions to agree. Later P5/P6 consumers may use an accepted subset without claiming all P4 complete.
