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
provider contracts in `pse-kernels`. The dated scientific qualification basis and its
exclusions are recorded in
[§24.2](operations-and-validation.md#section-24-2). Current canonical storage contracts
are owned by [§20](identity-and-publication.md#section-20); historical storage evidence
does not qualify the replacement deployment.

## 25. Supported scope and recorded limits

> Decision: [ADR-0102](../../adr/0102-discrete-and-global-design-target.md) —
> mixed-integer, disjunctive and globally certified classes enter the design target (Plan
> 22 G1–G7 implemented, with G4 and G6 partial as stated below);
> [ADR-0114](../../adr/0114-typed-operational-store.md) — the historical PostgreSQL
> operational store, publication catalog and durable multi-process execution (Plan 22
> O1–O9, G8). That storage composition is retired under proposed
> [ADR-0164](../../adr/0164-unify-simulation-substrate.md). Plan 22 closed on 2026-09-29; its
> [retired record](https://github.com/paul-heyse/pse-arrow/blob/f57b71d56f6eb2c319c4340d6f26abc6a1dc5abc/docs/plans/22-solver-capabilities.md#outcome-recorded-after-implementation)
> states what was built and what it deferred.
>
> Decision: [ADR-0118](../../adr/0118-one-kkt-point-analysis.md) — sensitivity, covariance
> and uncertainty under PS-12 validity, through one KKT-point analysis (Plan 22 S0–S4,
> implemented for the NLP routes, which automatic routing prefers for a quadratic program's
> sensitivity request, N5; the advanced step, Y5c2, implemented). ADR-0118 supersedes
> ADR-0107.

The Supported column describes implemented contracts; scientific evidence retains the
exercised K8 seed and the dated local Linux Plan 22 Q1 basis in §24.2. The integrated
kernel campaign is owned by [Plan 23](https://github.com/paul-heyse/pse-arrow/blob/4c24721e691187e1a5b28398b29722fbde671da8/docs/plans/23-thermodynamic-domain-and-campaign.md).
Neither historical campaign establishes current canonical storage or assembled-worker
qualification. The refused column is enforced: an unsupported request fails with
a source-attributed diagnostic ([§23](operations-and-validation.md#section-23)). There is
no silent fallback, approximate substitute or compatibility route.

| Area | Supported | Refused or outside the profile | Contract |
|---|---|---|---|
| Definitions and composition | Generic typed definitions, interfaces/defaults, finite/indexed and continuous membership, lazy demand, functions/partials, accumulators, children, cases and typed connections; indicator, SOS, cardinality, piecewise-linear and logic declarations, complementarity and nested disjunctions, lowered by declared realizations; objective members with priority, weight, normalization and level tolerances, and several levels optimized lexicographically, natively on HiGHS or as a staged sequence | Ambiguous interfaces/imports, unavailable demanded contracts, invalid physical operations, exceeded expansion limits; a realization whose preconditions fail (infinite bounds, an incomplete or unbounded FBBT interval, a nonlinear disjunct row under plain `hull`); a zero degradation tolerance on a staged objective level | [§6.8](schema-and-relations.md#section-6-8), [§10](models-and-composition.md#section-10)–[§12](models-and-composition.md#section-12), [§19.7](workflows-and-results.md#section-19-7), [§22](models-and-composition.md#section-22) |
| Physical properties | Authored ideal, PR and PC-SAFT seed potentials, DIPPR/Shomate/RPP4/Perry calorics, FTPx/FPhx, BTIdeal/BT_PR and dilute-liquid state bindings; explicit data/reference/envelope contracts, with provider output envelopes enforced at every evaluation | Unproved global stability or branch smoothness: the authored tangent-plane model certifies an ideal feed stable on the `certify` route (ADR-0102, Plan 22 G6, partial), a known Peng–Robinson instability is detected only locally, and the PC-SAFT distance is solved locally, not certified; undeclared extrapolation; unported property families | [§9](physical-semantics.md#section-9) |
| Reactions | Authored saponification stoichiometry, kinetics and heat conventions composed with generic accumulators | Unported reaction knowledge; missing demanded physical contracts | [§9](physical-semantics.md#section-9) |
| Numerical policy | One resolved policy with recorded precedence; reversible normalization; frozen absolute and relative budgets; original-space acceptance; exact Gram convexity evidence, numerical PSD evidence only on opt-in | Conflicting equal-priority sources; relaxation of hard guards | [§16](numerical-execution.md#section-16) |
| Problem classes | NLP (Ipopt with MUMPS, SPRAL or oneMKL Pardiso; POUNCE, including the explicit ℓ1 route), square roots, including one-sided bounds, and declared fixed-point maps (KINSOL), LP, MILP including authored integer, binary and semi-variable domains, and certified convex QP (HiGHS), explicit cones including SDP, and LP and convex QP on explicit selection (Clarabel, with QDLDL or oneMKL Pardiso), with infeasibility rays verified as certificates in original coordinates; MIQP and MINLP, semi domains included (SCIP, automatic) and, on the explicit `certify` intent, tolerance-qualified global certification of factorable problems over finite boxes, implicit blocks included (SCIP: `global_bound`, `proven_infeasible`, `GapQualified`); on SCIP, exact rational MILP (`exact_certificate`), IIS of linear and nonlinear programs, a ranked solution pool, reoptimization across a MIP sequence and deterministic concurrency with streamed incumbents; disjunctive programs through linear realizations, or native ones on SCIP; complementarity through smooth, disjunctive or ℓ1-penalty realizations; continuous cone programs recognized from compiler facts (Clarabel, automatic); LP, convex QP and cones on explicit selection with batched studies (POUNCE-convex); non-rigorous sum-of-squares bounds on polynomial programs; a KKT-point analysis, parametric sensitivity, the reduced Hessian and advanced-step predictions at optimizing NLP candidates; qualified regular-square Root parameter response under ADR-0144 (authorized implementation, focused evidence in Plan 25d) | Native realizations on any adapter but SCIP; certification or global solving where a provider output without a declared envelope enters a nonlinear term (no production provider declares one yet); exact rational solving beyond linear programs without native forms, and exact solving with IIS, reoptimization or concurrency; free discrete variables in root, fitting, integrated-dynamics and nested implicit solves, and in initialization unless its policy fixes them; nonconvex QP to HiGHS; a KKT-point analysis on coefficient, cone or root routes; Root reduced Hessians or covariance propagation; a certified bound from a sum-of-squares relaxation; a Gauss–Newton Hessian outside least-squares fits; two-sided boxes on KINSOL and constrained fixed-point iteration; JIT or SIMD evaluation | [§15.5](numerical-execution.md#section-15-5), [§18](numerical-execution.md#section-18) |
| Initialization and recycles | Transactional staged initialization as one staged sequence with per-step overlays and value-only rebind; finite supplied continuation and bounded adaptive homotopy; authored tears selected by HiGHS MILP with an independent acyclicity witness; causal fixed-point maps; explicit starts independent of allocation reuse | Any convergence guarantee for a strategy | [§17](numerical-execution.md#section-17) |
| Dynamics | ODE and index-1 DAE with a fixed diag(I,0) mass matrix; Diffsol BDF, SDIRK and (mass-free) explicit schemes with pse-owned faer LU or KLU linear solvers whose failures are typed; IDAS with recoverable trials, directional events without sensitivities, sign constraints, Krylov with a Jacobi preconditioner and a steady start; scheduled inputs whose interval values are sensitivity parameters on both integrators; consistent initialization; finite events/resets; physical time origins; smooth forward sensitivities; checkpointed adjoint gradients on both integrators; exact transient Hessians on IDAS by forward-over-adjoint; authored schedules, directional events and modes in fixtures; single and multiple shooting, authored or in Rust; native quadratures; simultaneous authored FD/Radau schemes and simultaneous dynamic optimization; rolling horizons (NMPC and MHE) with an advanced-step controller | Higher-index or general implicit DAE; variable-layout dynamics; IDAS sensitivities across events; adjoints across events or resets or over declared quadratures; second-order sensitivities on Diffsol; authored events on the simultaneous route; unsupported residual/index structure | [§13](workflows-and-results.md#section-13) |
| Fitting | Steady, transient and mixed fitting over declared sparse or dense support; candidate response derivatives, or adjoint gradients alone for transient experiments; exact, limited-memory or Gauss–Newton Hessians; a qualified estimate requires convergence, original feasibility and response rank; one local covariance per fit, exact or Gauss–Newton, withheld with its reason; Wald and profile-likelihood intervals; first-order propagation of the covariance to outputs, with PS-12 validity (ADR-0118) | Covariance with non-unit importance, a rank-deficient response or a parameter at a bound (withheld, with the identifiable directions published); estimated residual variance; global identifiability | [§19](workflows-and-results.md#section-19) |
| Results and publication | Typed completion through Rust, Arrow and Python; ordinary runs retain actual outcomes in one canonical database; exact closed manifests, original bounded IPC, derived row/field indexes, protected reopening, local IPC export and explicit retirement/reclamation | Durable result retention from an explicit ephemeral run; public SQL convenience over canonical results; automatic import or migration of retired PostgreSQL/Delta storage; remote storage deployment and implicit age-based deletion of scientific history | [§20](identity-and-publication.md#section-20), [§21](workflows-and-results.md#section-21) |
| Durable execution | Durable execution by default; fenced registration/claim before native effects, bounded progress ingestion, exact accepted seeds, current-authority recovery and cancellation; authored study occurrences claimed by finite workers under shared scientific policy, with effect-free summary recovery | Distributing one solve across processes; late ingestion by expired/revoked writers; retry without explicit scientific policy/effect knowledge; unsupported interpretation/schema; remote deployment; a queued-priority or independent global retry-policy API | [§20.6](identity-and-publication.md#section-20-6), [§19.3](workflows-and-results.md#section-19-3) |
| Python | Registry-generated declarations and typed ids, generated boundary documents, blocking/async native operations, exact canonical run/attempt handles, typed Arrow result/output/analysis selectors, local export, studies and retained progress; explicit ephemeral execution still uses canonical scientific sources | Mathematics in Python; production Pyomo or NL routes; retired SQL/publication/catalog APIs | [§21](workflows-and-results.md#section-21) |

**Current storage evidence.** The implemented profile is authenticated loopback gRPC
with a thin native SDK client and one RocksDB graph/relational database
([§20.6](identity-and-publication.md#section-20-6)). *Tested*: the named
`just canonical-recovery-test --profile-state <configured-state>` journey passed
on 2026-10-06 using the selected local deployment profile in an isolated fixture. It
exercises server SIGKILL/reopen, exact historical source/product and bit preservation,
large staged payloads, the admission gate and stopped-database backup/validated restore.
It does not establish physical power-loss survival, assembled numerical-worker recovery,
remote deployment, capacity/performance measurements or comprehensive qualification.
The owning packet retains acceptance status; ADR-0164 remains proposed.

**Recorded limits.** The following retain the scope of the dated scientific qualification;
new storage implementation does not broaden that evidence:

- Qualification is local Linux with the pinned default and native feature profiles. Remote
  CI, release, wheel or distribution, and other-platform qualification are not claimed.
- GPU support, distributing a single solve across processes or hosts, general
  higher-index DAEs and interval-rigorous global optimization are not part of the design
  target, not merely unimplemented. Tolerance-qualified global certification
  ([ADR-0102](../../adr/0102-discrete-and-global-design-target.md)) is implemented for
  factorable problems over finite boxes; implicit blocks enter through their residuals, and
  a provider output inside a nonlinear term needs a declared envelope, which no production
  provider declares yet. Only `exact_certificate`, from SCIP's exact rational MILP, is
  rigorous. The historical Plan 22 multi-process qualification exercised local worker
  and publisher processes against PostgreSQL 18 under
  [ADR-0114](../../adr/0114-typed-operational-store.md). Those process routes and that storage
  composition are retired; the evidence is not a current SurrealDB deployment claim.
  The 10 000-point study scale (scenario S15) remains unmeasured.
- Declared operating envelopes and exercised reference comparisons do not certify
  empirical property accuracy. Passing analytic or reference cases does not establish
  untested formulations.
- The IDAES parity harness covers the pinned environment and explicitly exercised
  scientific reference comparisons. It does not establish numerical equivalence with IDAES.
- Native NLP presolve excludes pinned bound tightening without original multiplier
  provenance; interval source-infeasibility analysis and supported affine recovery retain
  their separate contracts ([§18.5](numerical-execution.md#section-18-5)). HiGHS's QP
  regularization is derived from the requested absolute gap and is reserved
  ([§18.10](numerical-execution.md#section-18-10)).
- SCIP's concurrent mode is exercised by two-thread solves, including its incumbent stream
  and cancellation ([§18.8](numerical-execution.md#section-18-8)).
- The KKT-point analysis serves the NLP routes: a quadratic program's sensitivity request
  routes to one automatically, and an explicit coefficient or cone adapter withholds the
  quantities ([§15.5.1](numerical-execution.md#section-15-5-1)). A candidate lacking correct
  original multipliers qualifies only `Feasible` and its sensitivities are withheld. The
  supported affine reduction preserves the original singleton-row response. The first-order prediction
  agrees with Ipopt's sIPOPT within 1e-6 on a nondegenerate program, compared on this machine
  (`just parity`).
- Covariances and intervals assume the declared standard deviations are exact; no residual
  variance is estimated ([§19.4](workflows-and-results.md#section-19-4)).
- Global certification of the Peng–Robinson instability case does not finish in bounded time
  (SCIP left a gap after 10 minutes), so instability is established locally only. The
  certified heater declares its 2 GiB foreign allowance on its own solve
  ([§18.10.1](numerical-execution.md#section-18-10-1)).
- Fixed Symbolica symbol registration gives semantic and numerical agreement across
  processes, not bitwise reproducibility.
- Resource reservations are finite configurable policy with explicit allowances for
  foreign library memory; they are not a measurement of process RSS.
- Parity, native builds and native tests run on the maintainer's Linux machine against the
  solver prefix extracted from the pinned image; no container, CI or other-host route
  builds the native extension.
- Excluded solver capabilities, each impossible with the pinned libraries or a duplicate of
  a delivered capability (DP-01, DP-16):

  | Capability | Reason | Revisit |
  |---|---|---|
  | HSL MA27/57/77/86/97 | No licence route; SPRAL SSIDS and MKL Pardiso serve large KKT systems | register R-34 |
  | HiGHS lazy constraints (callback kind 8) | Declared but never invoked by HiGHS 1.14.0 or 1.15.0 | register R-44 |
  | HiGHS user-solution callback as a MIP seed | Redundant with setting the solution before the run | — |
  | Automatic Clarabel routing without recognition | Routing derives from compiler facts ([§18.10](numerical-execution.md#section-18-10)) | — |
  | sIPOPT, `pounce-sensitivity` or `pounce_convex::QpSensitivity` as a route | Duplicate the one KKT-point analysis; sIPOPT remains the parity oracle | ADR-0118 |
  | Couenne or Bonmin | Duplicate SCIP's MINLP capability | ADR-0102 |
  | Bespoke outer approximation over HiGHS and Ipopt | Reimplements solver machinery (DP-13) | — |
  | diffsol-nl as the nested `InnerSolver` | Loses KINSOL's recoverable trials, sign constraints and cancellation | register R-47 |
  | CVODES, ARKODE | No capability beyond Diffsol and IDAS | ADR-0110 |
  | `scip-sys` `bundled` or `from-source`; `russcip` | Unverified downloads, a second Ipopt, panicking conversions | — |
  | Parsing Ipopt's timing journal | Status strings are never parsed (PS-10) | register R-48 |
  | SCIP for DegeneracyHunter and tear selection | No capability gain; HiGHS serves them | — |
  | Clarabel `faer-sparse` | A second faer beside the pinned one; duplicates MKL Pardiso | register R-45 |
  | SCIP `bounddisjunction` for semi domains | The `semi(indicator)` lowering serves them (ADR-0103) | register R-46 |

## 26. Risks and unresolved design choices

> Decision: [ADR-0164](../../adr/0164-unify-simulation-substrate.md) (proposed) —
> canonical local graph/relational storage, exact admission and retained outcomes replace
> the PostgreSQL/Delta composition. [ADR-0114](../../adr/0114-typed-operational-store.md)
> retains historical rationale, not current deployment qualification.
> [ADR-0108](../../adr/0108-ipopt-linear-solvers-and-solver-image.md) — solver image and
> Ipopt linear solvers (register R-08, R-09 and R-34).

Deferred choices with an observable trigger and review date are owned by the
[deferred-decision register](../../adr/register.md); rows are cited here, not copied.

**Current risks.**

| Risk | Why it matters | Current control |
|---|---|---|
| Scientific adequacy outside exercised cases | A model that compiles and converges can still be physically wrong outside its declared formulation | Declared envelopes and formulations, refusal of unsupported physics, independent physical closure and reference comparisons |
| Incomplete reuse dependencies | An undeclared semantic or build-helper input can make a reused product stale | Complete consumed positive/absent/inventory premises, versioned projections, actual selected producer/root and source-bound helper/native reviews; unknown completeness refuses persistent replay while normal admission remains available |
| Local convergence only | NLP, root and recycle strategies find local solutions; multiple roots depend on starts | Explicit start policy and seed provenance, original-space qualification, tear acyclicity witnesses; no convergence guarantee is claimed (register R-32) |
| Coupled library pins | Reference-only FeOS fixes its num-dual family; Arrow/DataFusion boundary consumers share one type universe | One resolved version per family, `just family-check`, provider contract review before moving pins (register R-05) |
| Resource estimates | A bad estimate either refuses valid work or admits more than the host can hold | Finite configurable policy, ownership-specific reservations, separate RSS measurement |
| Canonical store availability and interpretation | Source admission, preparation and durable execution depend on the authenticated local store; a stale writer or unsupported schema cannot safely continue | Typed infrastructure/interpretation refusal with no durable-to-ephemeral fallback; ephemeral execution omits result retention but still requires canonical sources; open verifies interpretation/schema and performs no reset; exact acknowledgments and owner-issued protections/roots preserve obligations |
| Symbolica licensing | Use beyond Symbolica's unlicensed mode relies on a locally provisioned personal licence | The optional key is read from `SYMBOLICA_LICENSE` at initialization and never enters artifacts, identities or diagnostics; distribution terms are a release question (register R-31) |

**Unresolved design choices.** Each has no selected position; the current system refuses
or does not offer the capability until an owner decides. Multi-writer admission has an
implemented canonical guarded protocol ([§20.2](identity-and-publication.md#section-20-2));
this does not infer broad deployment qualification. The historical PostgreSQL catalog is
not a second authority. Remote storage remains outside the supported local profile;
a real remote consumer would require its own decision and qualification (register R-37).

| Choice | Current position | What would settle it |
|---|---|---|
| Distribution of native solvers | Solvers are linked from the pinned local build; no wheel carries them | Decide bundling versus runtime loading and per-platform build recipes at the first distributed artifact (register R-08, R-09) |
| Licence admission at release | Dependency and licence admission is advisory ([ADR-0066](../../adr/0066-dependency-admission-and-licence-policy-are-advisory.md)) | The first published crate or wheel makes every linked licence a release question (register R-31) |
| Compiled third-party providers | Providers are registered in-tree in `pse-kernels` | An actual external provider package; its loading boundary and failure containment must preserve [D9](architecture-overview.md#section-d9) (register R-04) |
| Broader phase-equilibrium knowledge | Selected seed formulations only; local branch evidence is not global stability. The certified tangent-plane check covers an ideal feed; a Peng–Robinson instability is detected locally and the PC-SAFT distance solved locally (ADR-0102, Plan 22 G6, partial) | New authored formulations with explicit stability, validity and derivative contracts under [§9](physical-semantics.md#section-9), and a certification that closes in bounded time |

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
| Homotopy | `core/solvers/homotopy.py` | Partial | Supplied continuation and bounded authored adaptive homotopy as value-only staged steps |
| Sequential modular, tears | Pyomo `SequentialDecomposition` | Implemented | `pse-backend-native::tears`, `pse-backend-native::recycle` |
| Scaling | `core/scaling/*` | Partial | Resolved policy and reversible normalization; `pse-math::numerics`, `pse-math::normalization`; authored schemes and diagnostics |
| Model statistics, structural diagnostics | `model_statistics.py`, incidence analysis | Implemented | Matching, DM and BTF; `pse-structural::incidence` |
| Numerical diagnostics | `diagnostics_tools/*` | Partial | Original-space quality and local fitting rank; `pse-backend-native::quality`. Bounded Jacobian SVD/optimization and nonlinear explanations |
| Solver configuration | `core/solvers/*` | Implemented | Class-specific adapters; `pse-backend-native` |
| Dynamics | `dyn_utils.py`, PETSc DAE | Partial | ODE/index-1 profile with forward and adjoint sensitivities, shooting, simultaneous dynamic optimization and rolling horizons (NMPC, MHE, advanced step); `pse-backend-native::dynamics`, `pse-runtime::workflow::{dynamics, shooting, horizon}` |
| Parameter estimation | Pyomo `parmest` usage | Implemented | `pse-runtime::workflow::fitting`; local covariance, Wald and profile-likelihood intervals |
| Parameter sweeps | `parameter_sweep.py` | Implemented | Finite authored studies with shared occurrence policy and retained exact outcomes across canonical workers; explicit ephemeral staged execution and batched points on POUNCE-convex; `pse-runtime::workflow::{staged, study}` |
| Convergence evaluation | `convergence/*` | Partial | Finite authored studies and retained per-case outcomes |
| Utility minimization | `utility_minimization.py` | Not implemented | — |
| Serialization, tables, tags, units | `model_serializer.py`, `tables.py`, `tags.py`, `units_of_measurement.py` | Implemented | Registry relations, checked Arrow results and canonical retention/local IPC export; `pse-quantity`, `pse-operations` |
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
| Artifact request | A compiler-issued request for a compiled program, keyed by consumed demand, profile, semantic context and qualified relevant producer/ABI inputs |
| Attempt | One execution of a prepared case; owns mutable workers, native state and resource admission until joined. A durable attempt has an exact canonical lookup key and current generation/expiry fence; terminal admission records closed result membership and truthful completion |
| Authored | Written by an author or package; a primitive fact |
| `BodySpec` | Semantic key of one specialization-local mathematical body; excludes compilation settings |
| Candidate | Primal and available dual data supplied by a native solver, independent of its qualification |
| `CasePlan` | Immutable affine assembly of a case from local body outputs and explicit row contributions |
| `CaseStructure`, case values | The fixed/free layout of a case, and the values bound to it; a layout change is not a body change |
| Completion | The immutable result projection of an attempt: assessments, diagnostics, outcomes and lineage; reading it performs no computation |
| Contribution | A typed physical term that balance laws collect; one transfer has one identity and opposite signs |
| Definition | A reusable typed definition: parameters, domains, guards, equations, ports, contributions and child instances |
| Derived quantity kind | A quantity kind declared as a monomial of other kinds, with its canonical unit and the result a multiplicative chain resolving to it takes; declared, never synthesized |
| Eligibility | Contextual admissibility of a native route for a prepared problem, distinct from static adapter inventory and the selected route |
| Identity projection | A versioned, named selection of what a given scope's identity depends on |
| Instance binding | Assignment of a definition's formals to a specific instance, its topology and its source slots |
| KKT-point analysis | The one local analysis of an optimizing NLP candidate: activity, LICQ, curvature and inertia of its active-set KKT system in original coordinates, from which sensitivities, reduced Hessians and exact covariances are read |
| Local validity | The PS-12 statement published with every requested derived local quantity: certified, or withheld with its reason, with the verdicts of the KKT point it rests on |
| Accumulator | A generic conservation or accounting subject collecting declared contributions |
| Checked package revision | Immutable admitted declarations, visibility and physical context; specialization input |
| Native termination | The stop reason reported by a native library; not a claim of feasibility or optimality |
| Nonexecuting data | Stored declarations retained with a selected model but not executed; never execution evidence |
| Numerical policy | Resolved tolerances, nominals and scaling with recorded precedence; a separate input, never an adapter default |
| Operating envelope | A provider's declared valid input window; not a statement of empirical accuracy |
| Canonical substrate | One authenticated local SurrealDB graph/relational database over RocksDB for immutable sources/products, guarded execution and exact results/analyses; registry declarations generate its typed contracts |
| Canonical lookup key | The complete opaque native record address for an exact revision, attempt or manifest; distinct from scientific semantic IDs and content/request hashes |
| Ordinal, coordinate | A position within one prepared layout; never identity |
| Overlay | An immutable case binding layered on another, such as an initialization stage |
| Physical closure | Independent conservation checks recomputed from source contributions |
| Provider | A registered external function capability with typed coordinates, shape, validity, derivatives and failures |
| Publication | Coherent visibility of an exact closed canonical descriptor through its owning guarded admission; no separate result-publication call or catalog |
| Operation acknowledgment | Immutable exact-request receipt used to settle an uncertain effect without rerunning science or inferring absence from timeout |
| Result manifest | Exact admitted set/batch frontier of one terminal attempt; observation membership is separate from scientific usability |
| Qualification | Original-space numerical assessment of a candidate: feasibility, stationarity, gap or rank as applicable |
| Realization | The declared transformation that lowers an implicit block, constraint form or disjunction, with its stated equivalence |
| Protected selection | A store-issued finite protection of one exact source revision or result manifest; continued database reads require current protection |
| Retained root | A durable reachability obligation issued by source history or product/execution/analysis admission; generic retention can mint or move deliberate history only |
| Qualified producer | Reviewed actual selected compiler-root/input evidence bound to the loaded executable attestation; matching caller-chosen receipt fields alone confer no authority |
| Quantity type | Kind, dimension, basis, reference state, scale kind, shape and subject |
| Reference | Shipped library data, such as units, elements and methods |
| Retention | Preservation of exact source/result/analysis closure under canonical roots and protections; explicit owner retirement and bounded reclamation are distinct from in-process cache eviction |
| Semantic ID | Stable 128-bit identity of an authored entity, unchanged by rename |
| Specialization | Finite expansion of a definition for a selected instance and case into library mathematics |
| Staged sequence | Finite steps on one native session, each an overlay over the original specification with a typed start; initialization, homotopy, studies and authored runs use it |
| Start | The numerical start actually used by an attempt, with its origin; distinct from reused allocation |
| Tear | A connection occurrence cut to break a recycle, selected by an authored policy |
| Unit product | A unit literal as a canonical product of atomic units with rational exponents; its identity does not depend on its spelling |
| Usability | The final decision whether a candidate may be used, combining qualification, closure and explicit opt-ins |
