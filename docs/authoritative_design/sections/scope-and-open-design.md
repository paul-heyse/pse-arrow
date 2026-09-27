---
title: Supported scope and open design
status: current
---

# Supported scope and open design

This page states what the implemented system supports, where it deliberately refuses
or stops, which risks remain, and which design choices are genuinely open. It also
carries the IDAES capability coverage summary and the glossary. Capability boundaries
are enforced in source by refusal at admission or routing: selected admission in
`pse-runtime::workflow`, contextual eligibility in `pse-backend-native::routing` and
provider contracts in `pse-kernels`. The qualification basis for these statements is
[§24.2](operations-and-validation.md#section-24-2).

## 25. Supported scope and recorded limits

> Decision: [ADR-0102](../../adr/0102-discrete-and-global-design-target.md) —
> mixed-integer, disjunctive and globally certified classes enter the design target;
> [ADR-0107](../../adr/0107-sensitivity-covariance-uncertainty.md) — sensitivity,
> covariance and uncertainty under PS-12 validity;
> [ADR-0112](../../adr/0112-postgresql-operational-store-and-catalog.md) — operational
> store, publication catalog and durable multi-process execution. The Supported column
> changes only as the [Plan 22](../../plans/22-solver-capabilities.md) packets land.

The Supported column describes implemented contracts and the exercised K8 seed.
The earlier local Linux qualification in §24.2 predates this replacement; the current
assessment is owned by [Plan 21](../../plans/21-modeling-kernel.md). The refused column is enforced: an unsupported request fails with
a source-attributed diagnostic ([§23](operations-and-validation.md#section-23)). There is
no silent fallback, approximate substitute or compatibility route.

| Area | Supported | Refused or outside the profile | Contract |
|---|---|---|---|
| Definitions and composition | Generic typed definitions, interfaces/defaults, finite/indexed and continuous membership, lazy demand, functions/partials, accumulators, children, cases and typed connections | Ambiguous interfaces/imports, unavailable demanded contracts, invalid physical operations, exceeded expansion limits | [§10](models-and-composition.md#section-10)–[§12](models-and-composition.md#section-12), [§22](models-and-composition.md#section-22) |
| Physical properties | Authored ideal, PR and PC-SAFT seed potentials, DIPPR/Shomate/RPP4/Perry calorics, FTPx/FPhx, BTIdeal/BT_PR and dilute-liquid state bindings; explicit data/reference/envelope contracts | Unproved global stability or branch smoothness (a certified tangent-plane stability check is in the target: ADR-0102, Plan 22 G6, not yet implemented); undeclared extrapolation; unported property families | [§9](physical-semantics.md#section-9) |
| Reactions | Authored saponification stoichiometry, kinetics and heat conventions composed with generic accumulators | Unported reaction knowledge; missing demanded physical contracts | [§9](physical-semantics.md#section-9) |
| Numerical policy | One resolved policy with recorded precedence; reversible normalization; frozen absolute and relative budgets; original-space acceptance; exact Gram convexity evidence, numerical PSD evidence only on opt-in | Conflicting equal-priority sources; relaxation of hard guards | [§16](numerical-execution.md#section-16) |
| Problem classes | NLP (Ipopt, POUNCE), square roots and declared fixed-point maps (KINSOL), LP, MILP and certified convex QP (HiGHS), explicit cones including SDP (Clarabel) | MIQP, MINLP, disjunctive programs and global certification until Plan 22 M and G packets land (in the target: ADR-0102); authored MILP until M1 lands; nonconvex QP to HiGHS; arbitrary boxes on KINSOL and constrained fixed-point iteration; JIT or SIMD evaluation | [§18](numerical-execution.md#section-18) |
| Initialization and recycles | Transactional staged initialization; finite supplied continuation; authored tears selected by HiGHS MILP with an independent acyclicity witness; causal fixed-point maps; explicit starts independent of allocation reuse | Any convergence guarantee for a strategy | [§17](numerical-execution.md#section-17) |
| Dynamics | ODE and index-1 DAE with a fixed diag(I,0) mass matrix; consistent initialization; finite events/resets; physical time origins; smooth forward sensitivities; native quadratures; simultaneous authored FD/Radau schemes | Higher-index or general implicit DAE; variable-layout dynamics; hybrid IDAS sensitivities; unsupported residual/index structure | [§13](workflows-and-results.md#section-13) |
| Fitting | Steady, transient and mixed fitting over declared sparse or dense support; candidate response derivatives; a qualified estimate requires convergence, original feasibility and response rank | Covariance, confidence intervals and uncertainty propagation until Plan 22 S3–S4 land (in the target with PS-12 validity: ADR-0107); global identifiability | [§19](workflows-and-results.md#section-19) |
| Results and publication | Typed completion through Rust, Arrow and Python; exact publication, settlement and read-only reopening; typed migration-required refusal for unsupported historical formats | Automatic migration; multi-writer or remote object-store deployment until the catalog lands (in the target: ADR-0112, Plan 22 O8) | [§20](identity-and-publication.md#section-20), [§21](workflows-and-results.md#section-21) |
| Python | Registry-generated declarations, blocking and async jobs, Arrow result streams, publication and settlement | Mathematics in Python; production Pyomo or NL routes | [§21](workflows-and-results.md#section-21) |

**Recorded limits.** The following bound every claim made from the current qualification:

- Qualification is local Linux with the pinned default and native feature profiles. Remote
  CI, release, wheel or distribution, and other-platform qualification are not claimed.
- GPU support, distributing a single solve across processes or hosts, general
  higher-index DAEs and interval-rigorous global optimization are not part of the design
  target, not merely unimplemented. Mixed-integer and disjunctive programs,
  tolerance-qualified global certification ([ADR-0102](../../adr/0102-discrete-and-global-design-target.md))
  and durable multi-process execution through the operational store
  ([ADR-0112](../../adr/0112-postgresql-operational-store-and-catalog.md)) are in the target
  and not yet implemented.
- Declared operating envelopes and exercised reference comparisons do not certify
  empirical property accuracy. Passing analytic or reference cases does not establish
  untested formulations.
- The IDAES parity harness covers the pinned environment and explicitly exercised
  compatibility names. It does not establish numerical equivalence with IDAES.
- With automatic presolve, pinned bound tightening can return a multiplier that fails
  complementarity against the original bound; HiGHS' default QP regularization can miss
  a stricter requested objective gap. Both candidates remain feasible and are reported as
  not qualified; disabling presolve or setting `qp_regularization_value` qualifies them.
- Fixed Symbolica symbol registration gives semantic and numerical agreement across
  processes, not bitwise reproducibility.
- Resource reservations are finite configurable policy with explicit allowances for
  foreign library memory; they are not a measurement of process RSS.

## 26. Risks and unresolved design choices

> Decision: [ADR-0112](../../adr/0112-postgresql-operational-store-and-catalog.md) —
> multi-writer and remote publication decided through the PostgreSQL catalog (register
> R-10 removed); [ADR-0108](../../adr/0108-ipopt-linear-solvers-and-solver-image.md) —
> solver image and Ipopt linear solvers (register R-08, R-09 and R-34). Plan 22; not yet
> implemented.

Deferred choices with an observable trigger and review date are owned by the
[deferred-decision register](../../adr/register.md); rows are cited here, not copied.

**Current risks.**

| Risk | Why it matters | Current control |
|---|---|---|
| Scientific adequacy outside exercised cases | A model that compiles and converges can still be physically wrong outside its declared formulation | Declared envelopes and formulations, refusal of unsupported physics, independent physical closure and reference comparisons |
| Incomplete reuse dependencies | An undeclared read makes a reused product silently stale | Complete versioned identity projections, separate validity and retention controls, incremental-versus-clean checks; conservative recomputation is always valid |
| Local convergence only | NLP, root and recycle strategies find local solutions; multiple roots depend on starts | Explicit start policy and seed provenance, original-space qualification, tear acyclicity witnesses; no convergence guarantee is claimed (register R-32) |
| Coupled library pins | Reference-only FeOS fixes its num-dual family; the vendored Delta crate tracks DataFusion and Arrow | One resolved version per family, `just family-check`, provider contract review before moving pins (register R-05) |
| Resource estimates | A bad estimate either refuses valid work or admits more than the host can hold | Finite configurable policy, ownership-specific reservations, separate RSS measurement |
| Symbolica licensing | Use beyond Symbolica's unlicensed mode relies on a locally provisioned personal licence | The optional key is read from `SYMBOLICA_LICENSE` at initialization and never enters artifacts, identities or diagnostics; distribution terms are a release question (register R-31) |

**Unresolved design choices.** Each has no selected position; the current system refuses
or does not offer the capability until an owner decides. Multi-writer and remote
object-store publication is no longer open: [ADR-0112](../../adr/0112-postgresql-operational-store-and-catalog.md)
decides it through the PostgreSQL publication catalog, and qualification follows Plan 22 O8.

| Choice | Current position | What would settle it |
|---|---|---|
| Distribution of native solvers | Solvers are linked from the pinned local build; no wheel carries them | Decide bundling versus runtime loading and per-platform build recipes at the first distributed artifact (register R-08, R-09) |
| Licence admission at release | Dependency and licence admission is advisory ([ADR-0066](../../adr/0066-dependency-admission-and-licence-policy-are-advisory.md)) | The first published crate or wheel makes every linked licence a release question (register R-31) |
| Compiled third-party providers | Providers are registered in-tree in `pse-kernels` | An actual external provider package; its loading boundary and failure containment must preserve [D9](architecture-overview.md#section-d9) (register R-04) |
| Broader phase-equilibrium knowledge | Selected seed formulations only; local branch evidence is not global stability (a certified stability check is in the target: ADR-0102) | New authored formulations with explicit stability, validity and derivative contracts under [§9](physical-semantics.md#section-9) |

## Capability coverage against IDAES

Current status per capability area, established at the level of crate and module
existence plus the supported boundary in [§25](#section-25). **Implemented** means the
capability is executable within §25; **Partial** means a named subset is; **Not
implemented** means no executable path exists. Scope is defined in
[§0.2](architecture-overview.md#section-0-2).

| Capability area | IDAES mechanism | Status | Realization |
|---|---|---|---|
| Definitions, parameters, finite domains | `process_block.py`, `ConfigBlock` | Implemented | Reusable definitions and instance bindings; `pse-modeling`, compiler modeling queries |
| Flowsheets, ports, connections | `flowsheet_model.py`, `unit_model.py`, Pyomo `Port`/`Arc` | Implemented | Typed ports and connections; `pse-structural::flowsheet` |
| Lumped control volume and balances | `control_volume0d.py` | Implemented | Generic accumulators; authored process control-volume definitions |
| Distributed control volume | `control_volume1d.py`, Pyomo `dae` | Partial | Authored spatial CV and backward/Radau PFR seed |
| State definitions | `state_definitions/*` | Partial | Authored FTPx/FPhx, BTIdeal/BT_PR and dilute-liquid bindings |
| Pure-component methods | `pure/*` | Partial | NIST Shomate, RPP4, Perry and ideal expressions; `packages/reference/methods` |
| Equations of state | `eos/*` | Partial | Authored ideal, PR and PC-SAFT seed potentials; eNRTL not ported |
| Phase equilibrium, bubble/dew, flash | `phase_equil/*` | Partial | SmoothVLE, selected bubble/dew and nested ideal flash; local branch contracts |
| Transport properties | `transport_properties/*` | Not implemented | — |
| Reactions | `reaction_base.py`, `reactions/*` | Partial | Authored saponification kinetics and shared balances |
| Electrolytes | `eos/enrtl*.py`, electrolyte property sets | Not implemented | — |
| Helmholtz and CoolProp | `helmholtz/`, `general_helmholtz/`, `coolprop/` | Not implemented | — |
| Unit model library | `models/unit_models/*` | Partial | Authored Feed, Product, Mixer, Heater, Flash, Separator, pressure-changer presets, heat exchanger, CSTR, PFR and PC-SAFT vessel seed |
| Control | `models/control/controller.py` | Partial | Authored filtered PID with anti-windup and integrated/simultaneous fixtures |
| Costing | `costing_base.py`, `SSLW.py` | Partial | SSLW heat-exchanger tables, currency conversion and accounting |
| Initialization framework | `core/initialization/*` | Implemented | Staged strategies and conditional blocks; `pse-runtime::math::initialization`, `pse-structural::initialization` |
| Homotopy | `core/solvers/homotopy.py` | Partial | Supplied continuation and bounded authored adaptive homotopy |
| Sequential modular, tears | Pyomo `SequentialDecomposition` | Implemented | `pse-backend-native::tears`, `pse-backend-native::recycle` |
| Scaling | `core/scaling/*` | Partial | Resolved policy and reversible normalization; `pse-math::numerics`, `pse-math::normalization`; authored schemes and diagnostics |
| Model statistics, structural diagnostics | `model_statistics.py`, incidence analysis | Implemented | Matching, DM and BTF; `pse-structural::incidence` |
| Numerical diagnostics | `diagnostics_tools/*` | Partial | Original-space quality and local fitting rank; `pse-backend-native::quality`. Bounded Jacobian SVD/optimization and nonlinear explanations |
| Solver configuration | `core/solvers/*` | Implemented | Class-specific adapters; `pse-backend-native` |
| Dynamics | `dyn_utils.py`, PETSc DAE | Partial | ODE/index-1 profile; `pse-backend-native::dynamics`, `pse-runtime::workflow::dynamics` |
| Parameter estimation | Pyomo `parmest` usage | Implemented | `pse-runtime::workflow::fitting`; no covariance |
| Parameter sweeps | `parameter_sweep.py` | Implemented | Finite case batches; `pse-runtime::math::solves` |
| Convergence evaluation | `convergence/*` | Partial | Finite authored studies and retained per-case outcomes |
| Utility minimization | `utility_minimization.py` | Not implemented | — |
| Serialization, tables, tags, units | `model_serializer.py`, `tables.py`, `tags.py`, `units_of_measurement.py` | Implemented | Registry relations, Arrow results, Delta publication; `pse-quantity`, `pse-catalog` |
| Surrogates, DMF, UI, apps, `models_extra` | various | Not implemented | Later Plan 20 target ports; outside the K8 seed |

## Relation index

Relation, enumeration and extension-type detail is generated from the registry; see the
[generated registry reference](../../generated/README.md). Namespaces are
[authored](../../generated/relations/authored.md),
[reference](../../generated/relations/reference.md),
[normalized](../../generated/relations/normalized.md),
[runtime](../../generated/relations/runtime.md) and
[provenance](../../generated/relations/provenance.md).

## Glossary

| Term | Meaning |
|---|---|
| Admission | Checking a request against its contract before use; admitted input is consumed, retained as nonexecuting data, or refused with its source identity |
| Analysis request | The requested mode, outputs, derivatives and policy for an attempt; part of its identity |
| Artifact request | A compiler-issued request for a compiled program, keyed by demand, profile, source, build and ABI identity |
| Attempt | One execution of a prepared case; owns mutable workers, native state and resource admission until joined |
| Authored | Written by an author or package; a primitive fact |
| `BodySpec` | Semantic key of one specialization-local mathematical body; excludes compilation settings |
| Candidate | Primal and available dual data supplied by a native solver, independent of its qualification |
| `CasePlan` | Immutable affine assembly of a case from local body outputs and explicit row contributions |
| `CaseStructure`, case values | The fixed/free layout of a case, and the values bound to it; a layout change is not a body change |
| Completion | The immutable result projection of an attempt: assessments, diagnostics, outcomes and lineage; reading it performs no computation |
| Contribution | A typed physical term that balance laws collect; one transfer has one identity and opposite signs |
| Definition | A reusable typed definition: parameters, domains, guards, equations, ports, contributions and child instances |
| Eligibility | Contextual admissibility of a native route for a prepared problem, distinct from static adapter inventory and the selected route |
| Identity projection | A versioned, named selection of what a given scope's identity depends on |
| Instance binding | Assignment of a definition's formals to a specific instance, its topology and its source slots |
| Accumulator | A generic conservation or accounting subject collecting declared contributions |
| Checked package revision | Immutable admitted declarations, visibility and physical context; specialization input |
| Native termination | The stop reason reported by a native library; not a claim of feasibility or optimality |
| Nonexecuting data | Stored declarations retained with a selected model but not executed; never execution evidence |
| Numerical policy | Resolved tolerances, nominals and scaling with recorded precedence; a separate input, never an adapter default |
| Operating envelope | A provider's declared valid input window; not a statement of empirical accuracy |
| Ordinal, coordinate | A position within one prepared layout; never identity |
| Overlay | An immutable case binding layered on another, such as an initialization stage |
| Physical closure | Independent conservation checks recomputed from source contributions |
| Provider | A registered external function capability with typed coordinates, shape, validity, derivatives and failures |
| Publication | An exact, immutable Delta commit of selected results under an expected-parent precondition |
| Publication ticket, settlement | The serializable attempt handle issued before effects, and the read-only determination of committed, not committed or unresolved |
| Qualification | Original-space numerical assessment of a candidate: feasibility, stationarity, gap or rank as applicable |
| Quantity type | Kind, dimension, basis, reference state, scale kind, shape and subject |
| Reference | Shipped library data, such as units, elements and methods |
| Retention | Preservation of the closure needed to reopen retained publications; distinct from cache retention |
| Semantic ID | Stable 128-bit identity of an authored entity, unchanged by rename |
| Specialization | Finite expansion of a definition for a selected instance and case into library mathematics |
| Start | The numerical start actually used by an attempt, with its origin; distinct from reused allocation |
| Tear | A connection occurrence cut to break a recycle, selected by an authored policy |
| Usability | The final decision whether a candidate may be used, combining qualification, closure and explicit opt-ins |
