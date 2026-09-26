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

Everything in the Supported column is implemented and exercised on the local Linux
qualification profile. The refused column is enforced: an unsupported request fails with
a source-attributed diagnostic ([§23](operations-and-validation.md#section-23)). There is
no silent fallback, approximate substitute or compatibility route.

| Area | Supported | Refused or outside the profile | Contract |
|---|---|---|---|
| Definitions and composition | Reusable definitions with typed ports, parameters, finite and ragged domains, guards, equations, conservation laws, child instances and typed connections; exact integer and rational powers; declared splitting for extensive fanout | Template parent-domain expansion, inherited features, continuous domains and derivative symbols in templates, implicit property or reaction packages, unsupported scaling or tear policies | [§10](models-and-composition.md#section-10)–[§12](models-and-composition.md#section-12), [§22](models-and-composition.md#section-22) |
| Physical properties | FeOS PC-SAFT with DIPPR ideal-gas terms, homogeneous explicit-density execution for any admitted component count bound to checked records; declared operating envelope; separate enthalpy and entropy references; authored correlation methods; a declared C2 directional valve law | Phase-equilibrium and flash execution; extrapolation outside the envelope; absolute third-law entropy | [§9](physical-semantics.md#section-9) |
| Reactions | Homogeneous molar stoichiometry with element closure and an explicit heat term; one authored rate shared across balances | Equilibrium, multiphase and implicit-concentration reactions | [§9](physical-semantics.md#section-9) |
| Numerical policy | One resolved policy with recorded precedence; reversible normalization; frozen absolute and relative budgets; original-space acceptance; exact Gram convexity evidence, numerical PSD evidence only on opt-in | Conflicting equal-priority sources; relaxation of hard guards | [§16](numerical-execution.md#section-16) |
| Problem classes | NLP (Ipopt, POUNCE), square roots and declared fixed-point maps (KINSOL), LP, MILP and certified convex QP (HiGHS), explicit cones including SDP (Clarabel) | General MINLP; nonconvex QP to HiGHS; arbitrary boxes on KINSOL and constrained fixed-point iteration; JIT or SIMD evaluation | [§18](numerical-execution.md#section-18) |
| Initialization and recycles | Transactional staged initialization; finite supplied continuation; authored tears selected by HiGHS MILP with an independent acyclicity witness; causal fixed-point maps; explicit starts independent of allocation reuse | Any convergence guarantee for a strategy | [§17](numerical-execution.md#section-17) |
| Dynamics | ODE and index-1 DAE with a fixed diag(I,0) mass matrix; consistent initialization; finite events and resets; nonzero time origins; smooth forward sensitivities (Diffsol); recoverable residual trials (IDAS) | Higher-index or general implicit DAE; variable-layout dynamics; hybrid IDAS sensitivities; spatial discretization | [§13](workflows-and-results.md#section-13) |
| Fitting | Steady, transient and mixed fitting over declared sparse or dense support; candidate response derivatives; a qualified estimate requires convergence, original feasibility and response rank | Covariance, global identifiability and uncertainty claims | [§19](workflows-and-results.md#section-19) |
| Results and publication | Typed completion through Rust, Arrow and Python; exact publication, settlement and read-only reopening; typed migration-required refusal for unsupported historical formats | Automatic migration; multi-writer or remote object-store deployment | [§20](identity-and-publication.md#section-20), [§21](workflows-and-results.md#section-21) |
| Python | Registry-generated declarations, blocking and async jobs, Arrow result streams, publication and settlement | Mathematics in Python; production Pyomo or NL routes | [§21](workflows-and-results.md#section-21) |

**Recorded limits.** The following bound every claim made from the current qualification:

- Qualification is local Linux with the pinned default and native feature profiles. Remote
  CI, release, wheel or distribution, and other-platform qualification are not claimed.
- GPU support, distributed execution, global MINLP and general higher-index DAEs are not
  part of the design target, not merely unimplemented.
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

Deferred choices with an observable trigger and review date are owned by the
[deferred-decision register](../../adr/register.md); rows are cited here, not copied.

**Current risks.**

| Risk | Why it matters | Current control |
|---|---|---|
| Scientific adequacy outside exercised cases | A model that compiles and converges can still be physically wrong outside its declared formulation | Declared envelopes and formulations, refusal of unsupported physics, independent physical closure and reference comparisons |
| Incomplete reuse dependencies | An undeclared read makes a reused product silently stale | Complete versioned identity projections, separate validity and retention controls, incremental-versus-clean checks; conservative recomputation is always valid |
| Local convergence only | NLP, root and recycle strategies find local solutions; multiple roots depend on starts | Explicit start policy and seed provenance, original-space qualification, tear acyclicity witnesses; no convergence guarantee is claimed (register R-32) |
| Coupled library pins | FeOS fixes the num-dual derivative family; the vendored Delta crate must track the DataFusion and Arrow family | One resolved version per family, `just family-check`, provider contract review before moving pins (register R-05) |
| Resource estimates | A bad estimate either refuses valid work or admits more than the host can hold | Finite configurable policy, ownership-specific reservations, separate RSS measurement |
| Symbolica licensing | Use beyond Symbolica's unlicensed mode relies on a locally provisioned personal licence | The optional key is read from `SYMBOLICA_LICENSE` at initialization and never enters artifacts, identities or diagnostics; distribution terms are a release question (register R-31) |

**Unresolved design choices.** Each has no selected position; the current system refuses
or does not offer the capability until an owner decides.

| Choice | Current position | What would settle it |
|---|---|---|
| Multi-writer or remote object-store publication | Publication is qualified for a single local writer; Delta conflict detection relies on the store's conditional-create semantics | Qualify the target backend's conditional-write and durability contract before widening deployment (register R-10) |
| Distribution of native solvers | Solvers are linked from the pinned local build; no wheel carries them | Decide bundling versus runtime loading and per-platform build recipes at the first distributed artifact (register R-08, R-09) |
| Licence admission at release | Dependency and licence admission is advisory ([ADR-0066](../../adr/0066-dependency-admission-and-licence-policy-are-advisory.md)) | The first published crate or wheel makes every linked licence a release question (register R-31) |
| Compiled third-party providers | Providers are registered in-tree in `pse-kernels` | An actual external provider package; its loading boundary and failure containment must preserve [D9](architecture-overview.md#section-d9) (register R-04) |
| Phase-equilibrium execution | Refused; mechanical and global stability observations are already distinct | A selected formulation with explicit phase, stability and derivative-regularity contracts under [§9](physical-semantics.md#section-9) |

## Capability coverage against IDAES

Current status per capability area, established at the level of crate and module
existence plus the supported boundary in [§25](#section-25). **Implemented** means the
capability is executable within §25; **Partial** means a named subset is; **Not
implemented** means no executable path exists. Scope is defined in
[§0.2](architecture-overview.md#section-0-2).

| Capability area | IDAES mechanism | Status | Realization |
|---|---|---|---|
| Definitions, parameters, finite domains | `process_block.py`, `ConfigBlock` | Implemented | Reusable definitions and instance bindings; `pse-compiler`, `pse-runtime::workflow::composition` |
| Flowsheets, ports, connections | `flowsheet_model.py`, `unit_model.py`, Pyomo `Port`/`Arc` | Implemented | Typed ports and connections; `pse-structural::flowsheet` |
| Lumped control volume and balances | `control_volume0d.py` | Implemented | Generated balances from contributions; `pse-runtime::workflow::balances`, `packages/reference/units` |
| Distributed control volume | `control_volume1d.py`, Pyomo `dae` | Not implemented | Continuous domains are refused |
| State definitions | `state_definitions/*` | Partial | Single-phase FTPx and FcTP declarations; `packages/reference/states` |
| Pure-component methods | `pure/*` | Partial | NIST Shomate, RPP4, Perry and ideal expressions; `packages/reference/methods` |
| Equations of state | `eos/*` | Partial | PC-SAFT/DIPPR through FeOS; `pse-kernels::feos`. Cubic and eNRTL not implemented |
| Phase equilibrium, bubble/dew, flash | `phase_equil/*` | Not implemented | Refused at admission |
| Transport properties | `transport_properties/*` | Not implemented | — |
| Reactions | `reaction_base.py`, `reactions/*` | Partial | Homogeneous molar kinetics with explicit energy; `pse-runtime::workflow::reactions` |
| Electrolytes | `eos/enrtl*.py`, electrolyte property sets | Not implemented | — |
| Helmholtz and CoolProp | `helmholtz/`, `general_helmholtz/`, `coolprop/` | Not implemented | — |
| Unit model library | `models/unit_models/*` | Partial | Feed, product, mixer, heater, state junction and lumped volume templates; vessel recipe (`pse-runtime::workflow::vessel`); directional valve (`pse-kernels::valve`) |
| Control | `models/control/controller.py` | Not implemented | — |
| Costing | `costing_base.py`, `SSLW.py` | Not implemented | — |
| Initialization framework | `core/initialization/*` | Implemented | Staged strategies and conditional blocks; `pse-runtime::math::initialization`, `pse-structural::initialization` |
| Homotopy | `core/solvers/homotopy.py` | Partial | Finite supplied continuation over fixed and parameter inputs |
| Sequential modular, tears | Pyomo `SequentialDecomposition` | Implemented | `pse-backend-native::tears`, `pse-backend-native::recycle` |
| Scaling | `core/scaling/*` | Partial | Resolved policy and reversible normalization; `pse-math::numerics`, `pse-math::normalization`. No autoscaler or profiler |
| Model statistics, structural diagnostics | `model_statistics.py`, incidence analysis | Implemented | Matching, DM and BTF; `pse-structural::incidence` |
| Numerical diagnostics | `diagnostics_tools/*` | Partial | Original-space quality and local fitting rank; `pse-backend-native::quality`. No degeneracy hunter |
| Solver configuration | `core/solvers/*` | Implemented | Class-specific adapters; `pse-backend-native` |
| Dynamics | `dyn_utils.py`, PETSc DAE | Partial | ODE/index-1 profile; `pse-backend-native::dynamics`, `pse-runtime::workflow::dynamics` |
| Parameter estimation | Pyomo `parmest` usage | Implemented | `pse-runtime::workflow::fitting`; no covariance |
| Parameter sweeps | `parameter_sweep.py` | Implemented | Finite case batches; `pse-runtime::math::solves` |
| Convergence evaluation | `convergence/*` | Not implemented | — |
| Utility minimization | `utility_minimization.py` | Not implemented | — |
| Serialization, tables, tags, units | `model_serializer.py`, `tables.py`, `tags.py`, `units_of_measurement.py` | Implemented | Registry relations, Arrow results, Delta publication; `pse-quantity`, `pse-catalog` |
| Surrogates, DMF, UI, apps, `models_extra` | various | Out of scope | — |

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
| Definition | A reusable typed template: parameters, domains, guards, equations, ports, contributions and child instances |
| Eligibility | Contextual admissibility of a native route for a prepared problem, distinct from static adapter inventory and the selected route |
| Identity projection | A versioned, named selection of what a given scope's identity depends on |
| Instance binding | Assignment of a definition's formals to a specific instance, its topology and its source slots |
| Law template | A conservation or accounting rule expanded over declared contributions |
| `ModelRevision` | An immutable, admitted model with its bindings and topology; the input to every preparation |
| Native termination | The stop reason reported by a native library; not a claim of feasibility or optimality |
| Nonexecuting data | Stored declarations retained with a selected model but not executed; never execution evidence |
| Numerical policy | Resolved tolerances, nominals and scaling with recorded precedence; a separate input, never an adapter default |
| Operating envelope | A provider's declared valid input window; not a statement of empirical accuracy |
| Ordinal, coordinate | A position within one prepared layout; never identity |
| Overlay | An immutable case binding layered on another, such as an initialization stage |
| Physical closure | Independent conservation checks recomputed from source contributions |
| Provider | A registered physical computation with ports, data identity, phase, envelope, derivative order and typed failures |
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
