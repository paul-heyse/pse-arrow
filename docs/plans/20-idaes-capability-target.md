---
title: Target architecture for full idaes-pse modeling capability
status: draft
date: 2026-09-26
adrs: []
review_sources: [docs/design_review/reviews/design_review_idaes-capability-target_2026-09-26.md]
scenario_sources: [docs/design_review/design_principles/binding/pse-arrow.md#pse-s01]
---

# Target architecture for full idaes-pse modeling capability

**Status: draft.** This is a target design and a proposed sequencing. It is not authorized work
and schedules nothing: new work starts only when the maintainer authorizes it (AGENTS.md). Every
architectural claim here is *Proposed*. The assessment is in the
[design review](../design_review/reviews/design_review_idaes-capability-target_2026-09-26.md).

## Context

The functional target of `pse-arrow` is the full modeling capability of idaes-pse, realized in
Rust with a design that satisfies the
[selected design standard](../design_review/design_principles/standard.toml): Core 3.0 plus
the process-simulator profile 1.1. The target is characterized statically in the
`pyomo-and-solvers` skill. It holds the full idaes-pse 2.13.0, idaes-ext 3.4.2, idaes-examples
2.10.0 and idaes-ui 0.25.10 trees, plus records of:

- 156 model blocks and 70 method classes;
- 1,989 function semantics;
- 23,371 upstream test oracles;
- 2,559 documentation equations;
- the idaes-ext native inventory.

The current system delivers a strong execution foundation:

- physical typing;
- library-owned mathematics;
- structural admission;
- class-specific native solvers with truthful outcomes;
- immutable revisions;
- Salsa preparation;
- publication.

Its *modeling* layer covers only a narrow slice. Only scalar conservation laws execute. Only a
single-phase FeOS PC-SAFT provider exists, and it is scalar in and out. Phase equilibrium,
package-bound templates, indexed children, delegated ports, distributed domains, costing,
initialization knowledge and derived scaling are all refused or absent (the capability
appendix of
[§25–§26](../authoritative_design/sections/scope-and-open-design.md#capability-coverage-against-idaes)).

The maintainer has stated that FeOS need not remain in the solution. The design is to be
grounded in the best target architecture, and only loosely in the current implementation.

## Decisions

The target needs the following decisions. Each enters through the ADR route (AGENTS.md *When an
ADR is required*).

**Status (2026-09-26): all ten decisions were agreed by the maintainer.**
[Plan 21](21-modeling-kernel.md#decisions) refines A3–A9 into a knowledge-agnostic modeling
kernel. It removes the scientific code concepts (`pse-properties`, `pse-thermo`, registered
identity rules and closure algorithms), which become package data, and it drafts the ADRs in
refined form (packet K0).

| Proposed ADR | Decision | Governs | Route |
|---|---|---|---|
| A1 | **Scope.** The full idaes-pse capability, including `models_extra`, surrogate embedding and training studies, and the applications, is in the functional target. UI, DMF and CLI remain out | blueprint §0.2, relationship-to-IDAES scope, §25 | ADR + `design:` PR |
| A2 | **Parity reference moves to idaes-pse 2.13.0**, matching the pinned characterization | ADR-0003 parity pin | Short ADR (parity-pin rule) |
| A3 | **Property system**: potential-based model forms, registered thermodynamic identities, property packages, Salsa-tracked property resolution with lineage (amends D8 and §9.6), closure formulations. **FeOS leaves production** and becomes a test oracle | D8, D9, §9, ADR-0084/0088 (supersede the relevant parts) | ADR + design review |
| A4 | **Crates**: add `pse-properties`, `pse-modeling` and `pse-thermo` (the last absorbs the thermodynamic role of `pse-kernels`); move composition semantics out of `pse-runtime` | §3.2 | ADR + design review |
| A5 | **Template mechanisms**: package-typed parameters and facts, material domains, state-block children, multiplicity, delegated ports, expression and reference symbols, interface slots, presets, continuous domains; template syntax as an authoring surface | D2, §10.5, §11, §12, §22 | ADR + design review |
| A6 | **Indexed law engine** with contribution index maps, default balances, and cost and accounting subjects (extends D7/ADR-0010) | D7, §10 | ADR (within D7) |
| A7 | **Formulation primitives**: smooth max/min/abs, complementarity, `cbrt`, `tanh`/`sigmoid`/`softplus`, `asinh`, each with declared ε or domain obligations | §7.2 | Short ADR |
| A8 | **Analysis modes**: dynamics generated from the same templates; discretization transformation for space and for simultaneous time; the separate dynamic-state authoring and vessel recipe retire | §13, retired §13.4, D13 kept | ADR + design review |
| A9 | **Initialization knowledge and derived scaling**: start estimators, relaxation stages, adaptive homotopy; package and template nominal hints; derived term-magnitude nominals | §16.2, retired §16.3/§16.4/§17.2/§17.3 | ADR |
| A10 | **Diagnostics catalogue and studies**: named analyses with threshold profiles; study runner for case sets; covariance, sensitivity, rolling horizon, multiperiod | §15.5, §19.3–§19.8 | ADR |

## Target architecture in brief

The companion documents own the detail:

| Area | Document |
|---|---|
| Thermodynamics and properties | [20-target-thermodynamics.md](20-target-thermodynamics.md) |
| Models, control volumes, flowsheets, costing, surrogates | [20-target-modeling-and-flowsheets.md](20-target-modeling-and-flowsheets.md) |
| Initialization, scaling, diagnostics, solving, studies, applications | [20-target-numerical-strategies.md](20-target-numerical-strategies.md) |
| Completeness against idaes-pse | [20-idaes-coverage-map.md](20-idaes-coverage-map.md) |

```text
            authored packages (data): property packages, model forms, parameter sets,
            state definitions, unit / CV / costing / controller / surrogate templates
                                          │
   pse-authoring ── DSL + template syntax ┤ (one authority: registry declarations)
                                          ▼
   pse-modeling (pure) ── specialization, presets and slots, law engine, reactions,
         │               discretization, analysis-mode generation, report symbols
         │ demands (kind, index)
         ▼
   pse-properties (pure) ── property kinds, packages, model forms, identities,
         │                 resolution with lineage, closure formulations
         ▼
   pse-compiler (Salsa) ── tracked specialization + resolution + body admission + plans
         ▼
   pse-math (Symbolica/Numerica, faer) ── bodies, derivatives, evaluators, term magnitudes
         ▼                                   ▲ compiled model programs
   pse-backend-native (Ipopt, POUNCE,        pse-thermo ── closure algorithms (cubic roots,
   KINSOL, HiGHS, Clarabel, Diffsol, IDAS)   density, flash, pure-fluid inversion)
         ▼
   pse-runtime ── admission boundary, strategies (initialization, recycle, homotopy),
                  studies, jobs, results, publication  ──►  pse-py
```

Foundations are `pse-quantity`, `pse-material` (extended), `pse-ids`, `pse-diagnostics`,
`pse-structural`, and the registry (`pse-schema`) with its generated contracts. Dependencies
point downward in the diagram. The semantic crates never reach Arrow, DataFusion, Delta or
Tokio (the ceiling in §3.1).

**Properties the design claims** (review slot 6 tests each):

- **Extension locality.** A new cp correlation, EoS family, unit variant, ΔT formulation,
  costing correlation or controller is one declaration plus focused tests (§E). A new closure
  algorithm or DSL primitive is a registered contract.
- **One authority per meaning.**
  - A potential defines every property derived from it.
  - A template defines a unit, and a preset is binding data.
  - A law defines a balance, and a cost law defines an aggregate.
- **Physical integrity.** Complete quantity types, reference-state and basis rules, envelopes
  and closure checks apply to every generated equation (PS-01–PS-03).
- **Well-posedness before solving.** This exists already and now names units, ports and law
  members (PS-04).
- **Declared formulation, derivative source and scaling** (PS-06, PS-07).
- **Transactional initialization with declared knowledge** (PS-08).
- **Truthful outcomes** (PS-10, kept).
- **One model for every analysis mode** (PS-11).
- **Local testability.** The property, law, discretization and closure mechanisms are testable
  without runtime, storage or native solver startup (PSE-S05).

## Architectural drivers and scenarios

The drivers are breadth (about 90 unit models, 20 packages and 70 method classes to author),
extension locality for that breadth, derivative integrity through thermodynamics, and PS-11
reuse across modes and studies. Scenarios are defined in the review (S01–S14) and refine
binding seeds PSE-S01–S06.

## Plan

The waves follow the [coverage map](20-idaes-coverage-map.md#waves). Each packet names its
acceptance tests and what it deletes. Packets are proposed, not scheduled.

| Packet | Responsibility / dependencies | Scenarios / acceptance | Replaced code / deletion | Status or status-owner link |
|---|---|---|---|---|
| P0.1 ADRs A1–A10 | Decisions; review re-run on the ADR set | Review Accept-scoped | — | superseded by [Plan 21 K0](21-modeling-kernel.md#plan) |
| P0.2 `pse-modeling` extraction | Move specialization, law lowering, reactions and connections from `pse-runtime::workflow::composition`/`balances`/`reactions` into a pure crate consumed by compiler queries | S11: law and lowering tests run without the runtime; revision identities unchanged for existing fixtures | Runtime copies deleted | superseded by [Plan 21 K1–K5](21-modeling-kernel.md#plan) |
| P0.3 Template mechanisms | Package parameters and facts, material domains, state-block children, multiplicity, delegated ports, expression and reference symbols, slots, presets | S02, S06: `pse.units` Heater and Mixer and the `states` FTPx templates admit | Refusal sites at `lower.rs:124, :340, :507, :1438, :1564` retire with their tests | superseded by [Plan 21 K1–K5](21-modeling-kernel.md#plan) |
| P0.4 Indexed law engine | Subjects, index maps, default balances, element projection, phase-pair transfers | S02, S03: CV0D laws expand for componentPhase, componentTotal and elementTotal; closure per member | Scalar-only guard `lower.rs:1267` | superseded by [Plan 21 K1–K5](21-modeling-kernel.md#plan) |
| P0.5 `pse-properties` core | Property-kind registry, model forms, identities, packages, resolver, lineage | S01, S11: ideal-gas and Peng–Robinson properties resolved and compiled; identity consistency checks | Unconsumed method, package and selection relations become consumed | superseded by [Plan 21 K1–K5](21-modeling-kernel.md#plan) |
| P0.6 Formulation primitives | Smoothing, complementarity, `cbrt` and the others in the function enum | Positive and negative controls per function | The parse refusal test in `pse-math/src/functions.rs` changes | superseded by [Plan 21 K1–K5](21-modeling-kernel.md#plan) |
| P1.x Core thermodynamics and units | Wave W1 rows of the coverage map | Conformance per model against `idaes-oracle:` values with the same parameters and references (PS-13) | `vessel` recipe (after P3) | proposed |
| P2.x Reactions, activity, transport, initialization knowledge, derived scaling, diagnostics | Wave W2 | S07, S13 journeys; diagnostics named in model terms | Retired-section text replaced by the new owners | proposed |
| P3.x Discretization and dynamics as a mode | Wave W3 | S05, S06: PFR and HX1D collocation; steady-to-dynamic switch without re-authoring | `authored.dynamic_cases` state authoring for templates; `workflow::vessel` | proposed |
| P4.x Advanced thermodynamics | Wave W4: native PC-SAFT, Helmholtz, nested flash, electrolytes | S03, S04: FeOS-oracle agreement for PC-SAFT; IAPWS-95 against reference data | Production FeOS provider (`pse-kernels::feos`) and its FeOS/num-dual production dependencies | proposed |
| P5.x Costing, surrogates, studies, `models_extra`, applications | Wave W5 | S12, study journeys | — | proposed |

After Plan 23 lands, P1.x–P5.x are executed as **knowledge-port packets**. Each one is package
files plus conformance tests only, conforming to the typed domain schema, with any Rust
change recorded as a kernel gap.

## Finding dispositions

The disposition owner for F01–F13 is now
[Plan 23 *Finding dispositions*](23-thermodynamic-domain-and-campaign.md#finding-dispositions)
(transferred from Plan 21 on 2026-09-29). The table below records the original proposed
owners and is not maintained.

| Finding reference | Scenario reference | Disposition | Decision / work owner | Evidence or revisit trigger |
|---|---|---|---|---|
| [F01](../design_review/reviews/design_review_idaes-capability-target_2026-09-26.md#f01) | S01, S03, S04 | open | A3; P0.5, P4.x | — |
| [F02](../design_review/reviews/design_review_idaes-capability-target_2026-09-26.md#f02) | S01, S02 | open | A3; P0.5 | — |
| [F03](../design_review/reviews/design_review_idaes-capability-target_2026-09-26.md#f03) | S01 | open | A3; P0.5 | — |
| [F04](../design_review/reviews/design_review_idaes-capability-target_2026-09-26.md#f04) | S02, S12 | open | A6; P0.4 | — |
| [F05](../design_review/reviews/design_review_idaes-capability-target_2026-09-26.md#f05) | S07 | open | A3, A7; P0.6, P1.x | — |
| [F06](../design_review/reviews/design_review_idaes-capability-target_2026-09-26.md#f06) | S02, S06 | open | A5; P0.3 | — |
| [F07](../design_review/reviews/design_review_idaes-capability-target_2026-09-26.md#f07) | S08, S11 | open | A4; P0.2 | — |
| [F08](../design_review/reviews/design_review_idaes-capability-target_2026-09-26.md#f08) | S05 | open | A8; P3.x | — |
| [F09](../design_review/reviews/design_review_idaes-capability-target_2026-09-26.md#f09) | S06 | open | A8; P3.x | — |
| [F10](../design_review/reviews/design_review_idaes-capability-target_2026-09-26.md#f10) | S09 | open | A9; P2.x | — |
| [F11](../design_review/reviews/design_review_idaes-capability-target_2026-09-26.md#f11) | S08 | open | A9; P2.x | — |
| [F12](../design_review/reviews/design_review_idaes-capability-target_2026-09-26.md#f12) | S13 | open | A10; P2.x | — |
| [F13](../design_review/reviews/design_review_idaes-capability-target_2026-09-26.md#f13) | S04 | open | A3; P4.x | — |

## Verification

Targeted checks accompany each packet (AGENTS.md *Execution rhythm*). Model conformance uses:

- shared checks: degrees of freedom, closure per law member, derivative consistency where doubt
  remains, envelope rejection, and initialization from default estimators;
- reference validation against the skill's `idaes-oracle:` records, with the same parameters,
  reference states and tolerances as the IDAES test;
- independent thermodynamic references: NIST, teqp, CoolProp, and FeOS where its algorithms were
  not adapted (PS-13).

Comprehensive qualification is separately requested by the maintainer.

## Open items

- Measure the expression-size risk for multicomponent SAFT and IAPWS-95 (thermodynamics
  document R1) before fixing the default closure route per model family.
- Choose the first reference fluid set for Helmholtz forms (water and CO2 proposed).
- Confirm the template syntax grammar in a focused authoring design.
- Decide whether `pse-thermo` is a new crate or `pse-kernels` renamed (A4).

## Outcome (recorded after implementation)

### What was built

### A mistake made and corrected

### Deviations from the plan, deliberate
