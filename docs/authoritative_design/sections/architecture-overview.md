---
title: Architecture overview and foundational decisions
status: current
---

# Architecture overview and foundational decisions

This page states what the system is for, the native process-simulator contract it
implements, how responsibility and dependency direction are divided across the workspace,
and the fourteen foundational decisions (D1–D14) whose change requires an ADR. Detailed
contracts live in the numbered pages it links. Source owners are the workflow in
`crates/pse-runtime/src/workflow/`, the compiler in `pse-compiler`, library integration in
`pse-math`, native execution in `pse-backend-native` and the registry in `pse-schema`.

## 0. Purpose and scope

`pse-arrow` is a clean-room process systems engineering core in Rust with a Python
authoring and result boundary. It re-implements the modeling capability of IDAES-PSE,
compared against `idaes-pse==2.12.0` through an isolated parity harness only; see
[Relationship to IDAES](../../relationship-to-idaes.md). Routes, authority and notation
are in the [reading guide](reading-guide.md).

### 0.2 What "core IDAES-PSE capabilities" means here

The scope is the modeling framework, not the IDAES example libraries. The table names
the capability families the architecture is shaped to express and how each is realized.
Scope is not a delivery commitment: implemented, partial and absent coverage is recorded
in the [capability coverage appendix](scope-and-open-design.md#capability-coverage-against-idaes),
and supported limits in [§25](scope-and-open-design.md#section-25). IDAES source paths are
relative to `idaes-pse/idaes/`.

| IDAES subsystem | Source | Realization here | Scope |
|---|---|---|---|
| Process blocks, config, flowsheets | `core/base/process_block.py`, `flowsheet_model.py` | Reusable typed definitions, parameters, finite domains and instance bindings ([§11](models-and-composition.md#section-11), [§22](models-and-composition.md#section-22)) | In scope |
| Unit model base, ports, arcs | `core/base/unit_model.py`, Pyomo `network` | Typed ports and connections admitted by the selected revision ([§12](models-and-composition.md#section-12)) | In scope |
| Control volumes and balances | `core/base/control_volume*.py` | Balances generated from declared contributions ([§10](models-and-composition.md#section-10)) | In scope; distributed volumes not implemented |
| Property and modular property framework | `core/base/property_*.py`, `models/properties/modular_properties/**` | Explicit material bindings and physical providers; authored correlation methods ([§9](physical-semantics.md#section-9)) | In scope |
| Reaction framework | `core/base/reaction_base.py`, modular `reactions/**` | Declared stoichiometry, element closure and energy convention ([§9](physical-semantics.md#section-9)) | In scope |
| Generic unit model library | `models/unit_models/*.py` | Authored templates ([§11](models-and-composition.md#section-11)) | In scope |
| Initialization, homotopy | `core/initialization/**`, `core/solvers/homotopy.py` | Transactional staged strategies, explicit starts and finite continuation ([§17](numerical-execution.md#section-17)) | In scope |
| Scaling toolbox | `core/scaling/**` | One resolved numerical policy and coordinate normalization ([§16](numerical-execution.md#section-16)) | In scope |
| Statistics and diagnostics | `core/util/model_statistics.py`, `diagnostics_tools/**` | Structural analysis and original-space qualification ([§15](numerical-execution.md#section-15)) | In scope |
| Solver configuration | `core/solvers/**` | Class-specific native adapters and profiles ([§18](numerical-execution.md#section-18)) | In scope |
| Dynamics | `core/util/dyn_utils.py`, Pyomo `dae` | Native ODE/index-1 integration; no discretization lowering ([§13](workflows-and-results.md#section-13)) | In scope within the dynamic profile |
| Parameter estimation, sweeps | `core/util/parameter_sweep.py`, `convergence/**` | Native fitting and finite case batches ([§19](workflows-and-results.md#section-19)) | In scope |
| Costing, utility minimization | `core/base/costing_base.py`, `models/costing/SSLW.py`, `core/util/utility_minimization.py` | Authored templates over the same compiler | In scope; not implemented |
| Serialization, tables, tags, units | `core/util/model_serializer.py`, `tables.py`, `tags.py`, `units_of_measurement.py` | Registry relations, Arrow results and exact publication ([§4](schema-and-relations.md#section-4), [§20](identity-and-publication.md#section-20)) | In scope |
| Helmholtz and CoolProp backends | `models/properties/helmholtz/**`, `general_helmholtz/**`, `coolprop/**` | Physical provider contracts only | In scope as providers; not implemented |
| Surrogates, DMF, UI, apps, `models_extra` | various | — | Out of scope |
| Pyomo expression, NL and external-function mechanisms | Pyomo, ASL | — | Out of scope; capability is native, parity is isolated |

IDAES names are preserved only where parity needs a two-way mapping; the enumerations are
listed in [§6.14](schema-and-relations.md#section-6-14).

### 0.5 Current native process-simulator contract

> Decision: [ADR-0082](../../adr/0082-library-owned-process-mathematics.md),
> [ADR-0083](../../adr/0083-class-specific-native-execution.md),
> [ADR-0084](../../adr/0084-physical-provider-and-dynamic-contracts.md),
> [ADR-0093](../../adr/0093-qualified-native-strategies.md)

**Model authority.** Authored typed process definitions are the model. Physical typing
and original-domain obligations are checked before any algebra; admitted real-algebra
simplification never discards a guard. Symbolica and Numerica own values, derivatives and
evaluators; `pse-math` is the narrow integration layer. `pse-compiler` owns finite
specialization and pure Salsa preparation. Immutable prepared artifacts carry complete
semantic, build and profile keys; providers, evaluators and native workspaces belong to
the attempt that admitted them. No native solver state or effect lives in a tracked query.

**Class-specific native solving.** Each problem class has a library owner under one
admitted lifecycle ([§18](numerical-execution.md#section-18)):

| Class | Library |
|---|---|
| Nonlinear programs | Ipopt C API (`pse-ipopt-sys`) and POUNCE, sharing one NLP oracle |
| Square roots and declared fixed-point maps | KINSOL with KLU |
| Matching, Dulmage–Mendelsohn, block triangularization, presolve | pounce-presolve, with independent original-coordinate validation |
| LP, MILP, certified convex QP, tear selection | HiGHS |
| Explicit cones, including SDP | Clarabel |
| ODE/index-1 dynamics | Diffsol BDF; IDAS for smooth recoverable residual trials |

Unsupported integrality, convexity, representation, derivative or root-domain
combinations are refused before execution; there is no solver fallback. Native termination,
candidate availability, original-space numerical qualification, recomputed physical closure
and final usability remain separate facts. Library callbacks contain failures, and
cancellation keeps admission until native threads and thread-local state are destroyed.

**Physical providers.** FeOS supplies explicit-density PC-SAFT with DIPPR ideal-gas
contributions; num-dual supplies derivatives at the workspace pin. Ports carry quantity,
basis and reference, component order, data identity and phase. Declared operating envelopes
are enforced and do not establish empirical accuracy. Derivative availability does not
imply phase regularity. Balances keep independent conservation checks
([§9](physical-semantics.md#section-9)).

**Dynamics and fitting.** The dynamic profile is ODE or index-1 DAE with a fixed
diag(I,0) mass matrix, consistent initialization, finite events and resets, and smooth
forward sensitivities. Steady, transient and mixed fitting reuse the same NLP oracle and
physical results. General implicit or higher-index DAEs, hybrid IDAS sensitivities, global
identifiability and uncertainty claims are outside the profile
([§13](workflows-and-results.md#section-13), [§19](workflows-and-results.md#section-19)).

**Workflows.** Rust and Python callers use immutable model revisions, explicit prepare and
start operations, joined blocking or async jobs, retained Arrow results and explicit exact
publication. Python is an authoring and result boundary; Pyomo and IDAES are reference tools
confined to the parity harness ([§21](workflows-and-results.md#section-21)). Advanced cone
and causal-map construction is typed Rust functionality exposed through owned strategies.
The registry generates every public declaration contract.

### 0.6 Selected data-model foundation

> Decision: [ADR-0088](../../adr/0088-selected-model-and-physical-contracts.md),
> [ADR-0089](../../adr/0089-semantic-identity-projections.md),
> [ADR-0090](../../adr/0090-shared-execution-vocabulary.md),
> [ADR-0091](../../adr/0091-immutable-publication-contract.md),
> [ADR-0092](../../adr/0092-ordinary-execution-evidence.md)

The foundation keeps each meaning in one owner and derives every consumer projection
from it. All rows below are implemented; limits are recorded in
[§25](scope-and-open-design.md#section-25).

| Concern | Contract | Owner | Detail |
|---|---|---|---|
| Selected admission | Typed builders and documents enter one boundary. Every selected declaration is consumed, retained as explicitly nonexecuting data, or refused with its source identity. Unrelated stored cases are not selected. | `pse-runtime::workflow` (`ModelBuilder`, `AdmissionEntry`), compiler `Inputs` | [§22](models-and-composition.md#section-22) |
| Separate lifecycles | Definitions, instance bindings, case/analysis requests, resolved numerical policy, prepared products, used starts, results and publications have distinct identities. | workflow model, `pse-compiler::workspace`, `pse-math::assembly` | [§5](identity-and-publication.md#section-5) |
| Material and provider binding | Declared species map to actual PC-SAFT/DIPPR records in explicit coordinate order; separate enthalpy and entropy references; declared envelope and stability policy. Phase-equilibrium execution is refused. | `pse-kernels::feos`, `pse-material` | [§9](physical-semantics.md#section-9) |
| Reaction binding | Declared phases and species, element closure and an explicit heat term; the formation-enthalpy convention is not implemented. Equilibrium and multiphase reactions are refused. | `pse-runtime::workflow` reactions | [§9](physical-semantics.md#section-9) |
| Shared vocabulary | Registry-owned tags projected into `pse-model`, Arrow codecs and Python contracts. Diagnostics classify the failure kind and keep source identities, stage and observations. | `pse-schema`, `pse-model`, `pse-diagnostics` | [§23](operations-and-validation.md#section-23) |
| Identity projections | Versioned, named projections per scope. Floating framing preserves signed zero. Semantic contract identity excludes documentation and encoding. | `pse-ids`, `pse-compiler::physical_identity` | [§5](identity-and-publication.md#section-5) |
| Publication and retention | Immutable attempt members, an expected-parent precondition, a control manifest committed last, read-only settlement and exact reopening under an explicit compatibility and retention contract. | `pse-catalog::delta` | [§20](identity-and-publication.md#section-20) |
| Ordinary evidence | Ordinary tests, benchmarks and factual execution records. Executed, reused, transferred and not-run evidence are labelled separately. A source digest identifies inputs only. | `xtask`, recipes | [§24.2](operations-and-validation.md#section-24-2) |

## 1. Architectural summary

### 1.1 The design in brief

Authored typed definitions carry engineering intent. The runtime admits a selected,
immutable model, case and analysis into compiler inputs. Checked physical and structural
preparation derives library-owned mathematical programs and native layouts. Libraries own
the algorithms: Symbolica/Numerica for arithmetic and derivatives, FeOS for thermodynamics,
native solvers for iteration and factorization, Salsa for synchronous semantic reuse.
Arrow and DataFusion serve columnar and relational boundaries; Delta serves exact
publication. Definitions, policies, attempts and results have separate owners and
identities.

Dependencies point from foundations towards orchestration. Crate roles are owned by
[§3.2](workspace-and-dependencies.md#section-3-2); this is the direction they respect.

| Layer | Crates | Responsibility | Depends on |
|---|---|---|---|
| Foundations | `pse-diagnostics`, `pse-ids`, `pse-quantity`, `pse-material`, `pse-columnar` | Diagnostic vocabulary, identity and canonical hashing, physical quantities and functions, material facts, owned Arrow buffers | Nothing above this layer |
| Declaration | `pse-schema`, `pse-model`, `pse-relations` | Registry; generated library-neutral values; typed Arrow views and validators | Foundations |
| Data | `pse-engine`, `pse-catalog`, `pse-rules` | DataFusion sessions and caches; Delta publication and retention; registry invariant checks | Declaration |
| Mathematics | `pse-kernels`, `pse-math`, `pse-structural`, `pse-authoring`, `pse-compiler`, `pse-backend-native`, `pse-ipopt-sys` | Providers; library mathematics; graph projections; document parsing; specialization and Salsa preparation; native solver adapters | Foundations, `pse-model`, `pse-buildinfo` |
| Orchestration | `pse-runtime` | Selected admission, workflow jobs, resources, results and publication | All of the above |
| Boundary | `pse-py` | Python extension over the workflow and inspection | `pse-runtime` and the crates whose types it exposes |
| Tooling | `pse-codegen`, `pse-buildinfo`, `pse-testkit`, `xtask` | Generation, build provenance, dev fixtures | Declaration; `pse-testkit` is never a production dependency |

The mathematics layer has no Arrow, DataFusion or Delta dependency. `pse-runtime` is the
only crate that joins data and mathematics, so relational concerns cannot leak into
evaluation and native solver state cannot leak into storage.

### 1.2 The three representations, made concrete

| Representation | Current forms | Owner | Consumers |
|---|---|---|---|
| Authored definitions and the selected revision | Documents and builder declarations, `ModelRevision`, case and analysis declarations; `authored` and `reference` relations | `pse-authoring`, `pse-runtime::workflow::model`, registry | Compiler and inspection |
| Prepared immutable products | Compiler `Inputs`, `CaseStructure`, `PreparedCase`, `BodySpec`, `CasePlan`, compiled bodies, artifact requests, eligibility | `pse-compiler`, `pse-math`, `pse-backend-native::routing` | Attempt workers and native adapters |
| Attempts and results | Run handles, workers and native adapter state; `RunResult`, `Completion`, Arrow result tables, publication attempts and settlements | `pse-runtime`, `pse-catalog` | Rust, Arrow and Python readers; exact publication |

Nothing flows backward: a derived product or result never writes into authored
definitions. Initialization stages are overlays on immutable case bindings
([D13](#section-d13)). Identity rules for each form are in [§5](identity-and-publication.md#section-5).

### 1.3 Where each library sits

Library selection is by fit to an actual operation; eligibility is open under
[ADR-0066](../../adr/0066-dependency-admission-and-licence-policy-are-advisory.md). Exact
pins live in `Cargo.toml`, rationale in [§3.3](workspace-and-dependencies.md#section-3-3).

| Library | Role | Owner crate |
|---|---|---|
| Arrow | Columnar boundary, extension types, result tables, Python transfer through pyo3-arrow | `pse-columnar`, `pse-relations`, `pse-py` |
| DataFusion | Admission and set-oriented queries, invariant checks, inspection, retained artifact cache | `pse-engine`, `pse-rules` |
| Delta (delta-rs) and object_store | Exact publication, settlement, retention | `pse-catalog` |
| Salsa | Pure semantic preparation and reuse | `pse-compiler` |
| Symbolica, Numerica (GMP/MPFR) | Algebra, normalization, derivatives, evaluators, coefficient projection | `pse-math` |
| faer | Sparse structure and products; fitting LU/SVD | `pse-math`, `pse-backend-native`, `pse-runtime` |
| pounce-presolve | Matching, DM/BTF, qualified presolve | `pse-structural`, `pse-math`, `pse-backend-native` |
| petgraph, rustworkx-core | Flowsheet SCCs, deterministic ordering | `pse-structural` |
| FeOS, num-dual | PC-SAFT/DIPPR properties and derivatives; valve law derivatives | `pse-kernels` |
| Ipopt, POUNCE, KINSOL, HiGHS, Clarabel, Diffsol, IDAS | Class-specific native solving ([§0.5](#section-0-5)) | `pse-backend-native` |
| pyo3, pyo3-async-runtimes, tokio | Python extension, async joined jobs | `pse-py`, `pse-runtime` |
| blake3 | Identity hashing, only through `pse-ids` | `pse-ids` |

## 2. Architectural decisions

Each decision is binding. Changing one requires an ADR and a design review (AGENTS.md).
The linked section owns the detailed contract.

### D1. The registry declares each meaning once; authored definitions are the model

> Decision: [ADR-0088](../../adr/0088-selected-model-and-physical-contracts.md),
> [ADR-0090](../../adr/0090-shared-execution-vocabulary.md),
> [ADR-0051](../../adr/0051-generated-trees-and-regeneration-check.md)

Every durable shape, tag and public declaration contract is declared once in the
`pse-schema` registry. Rust values, Arrow schemas, Python contracts and reference docs are
generated from it and checked for regeneration equivalence. Authored definitions, loaded
from documents or built in Rust or Python, are the model authority; no hand-written
structure shadows a generated one. *Because* parallel descriptions drift silently.
See [§4](schema-and-relations.md#section-4).

### D2. Author definitions and bindings; derive specialized consequences

> Decision: [ADR-0088](../../adr/0088-selected-model-and-physical-contracts.md)

Authors write reusable definitions, instance bindings, connections, contributions,
material and provider bindings, and case values. Specialized equations, balances, finite
expansions and structural decisions are derived by the compiler at selected
specialization. Required meaning is explicit: an implicit property or reaction package is
refused, not inferred. *Because* imperative build procedures restate consequences and let
callers disagree. See [§11](models-and-composition.md#section-11) and
[§22](models-and-composition.md#section-22).

### D3. Definitions, prepared products and attempts are distinct artifacts

> Decision: [ADR-0088](../../adr/0088-selected-model-and-physical-contracts.md),
> [ADR-0089](../../adr/0089-semantic-identity-projections.md)

The three representations of [§1.2](#section-1-2) have separate owners, identities and
lifetimes. Numerical policy, used starts and publications are further distinct. Shared
model data is immutable; mutable state belongs to one attempt. *Because* conflating them
couples reuse, cancellation and storage lifetimes. See [§5](identity-and-publication.md#section-5).

### D4. Semantic identities, projections, ordinals and storage versions are distinct

> Decision: [ADR-0089](../../adr/0089-semantic-identity-projections.md),
> [ADR-0030](../../adr/0030-canonical-float-hashing-and-no-float-keys.md)

Stable 128-bit semantic IDs name authored entities across revisions; a rename changes a
name, never an identity. Versioned identity projections define what each scope depends on.
Local ordinals and native coordinates index layouts only. Publication IDs, exact Delta
versions and content hashes are separate again. Names, row positions and solver indices are
never identity. *Because* each kind answers a different equality question. See
[§5](identity-and-publication.md#section-5).

### D5. Physical type is more than a unit string

> Decision: [ADR-0088](../../adr/0088-selected-model-and-physical-contracts.md)

Every quantity-bearing value resolves a complete quantity type: kind, dimension, basis,
reference state, point or difference scale, shape and subject. Ports and provider coordinates
carry the same contract plus component order and phase. Conversion requires matching
meaning, not matching exponents. Absence is explicit; a missing value never means
"unknown, to be solved". *Because* dimension checks accept torque-for-energy and
gauge-for-absolute errors. See [§8](physical-semantics.md#section-8).

### D6. Mathematics is library-owned and derived from typed definitions

> Decision: [ADR-0082](../../adr/0082-library-owned-process-mathematics.md)

Symbolica atoms, evaluators and coefficient projections are derived specialization
artifacts with explicit instance and case bindings. Real-algebra semantics apply on the
admitted domain, after physical and original-domain obligations. There is no custom
expression IR, evaluator or differentiator. *Because* library algorithms are more capable
and better tested than a bespoke engine, and one authority avoids competing mathematics.
See [§7](mathematics-and-compilation.md#section-7) and
[§14](mathematics-and-compilation.md#section-14).

### D7. Balances are generated from declared contributions

> Decision: [ADR-0010](../../adr/0010-laws-are-templates-over-contributions.md),
> [ADR-0084](../../adr/0084-physical-provider-and-dynamic-contracts.md)

Material, element and energy balances are law templates expanded over the contributions a
definition declares. A transfer has one identity and opposite signs; extensive fanout needs
declared splitting. Independent physical closure checks read the same source contributions,
never solver residuals. *Because* a new unit should add contributions, not rewrite
conservation. See [§10](models-and-composition.md#section-10).

### D8. Property and reaction demand is bound explicitly

> Decision: [ADR-0088](../../adr/0088-selected-model-and-physical-contracts.md)

Provider calls declare requested outputs and derivative order against an explicit material
binding. Reactions declare species, phases and energy convention. Missing or unsupported
physics is refused before solving; inspection never constructs physics. *Because* lazy
construction hides what a model depends on. See [§9](physical-semantics.md#section-9).

### D9. Physical providers have one typed contract

> Decision: [ADR-0084](../../adr/0084-physical-provider-and-dynamic-contracts.md),
> [ADR-0088](../../adr/0088-selected-model-and-physical-contracts.md),
> [ADR-0093](../../adr/0093-qualified-native-strategies.md)

Provider registration states physical ports, data and component identity, phase, operating
envelope, implemented derivative order, smoothness and typed recoverable or terminal
failures. Outside-envelope evaluation is a recoverable trial failure, never silent
extrapolation or a fabricated value. Only implemented bindings are exposed. *Because* derivative availability does not establish
validity or regularity. See [§9](physical-semantics.md#section-9).

### D10. Computation placement follows the operation

> Decision: [ADR-0082](../../adr/0082-library-owned-process-mathematics.md),
> [ADR-0066](../../adr/0066-dependency-admission-and-licence-policy-are-advisory.md)

Typed Rust owns physical finite compilation; Salsa owns pure dependency tracking;
mathematical, thermodynamic and numerical libraries own their algorithms. Arrow and
DataFusion own admission, set-oriented model and result work, inspection and storage
boundaries; Delta owns publication. No layer duplicates another's authority. Hashes never
substitute for semantic admission. *Because* each mechanism is strongest at its own
operation. See [§1.3](#section-1-3) and [§3.3](workspace-and-dependencies.md#section-3-3).

### D11. Immutable preparation, attempt-owned native execution

> Decision: [ADR-0082](../../adr/0082-library-owned-process-mathematics.md),
> [ADR-0083](../../adr/0083-class-specific-native-execution.md)

Specialization-local immutable programs, sparse layouts and case plans feed worker-local
evaluators, providers and solvers admitted to one attempt. Native libraries own iteration
and factorization. Resource admission is held until native threads exit. *Because* shared
immutable products make reuse safe while mutable scratch stays private. See
[§14](mathematics-and-compilation.md#section-14) and [§18](numerical-execution.md#section-18).

### D12. Native class-specific execution; Python is the authoring and result boundary

> Decision: [ADR-0083](../../adr/0083-class-specific-native-execution.md)

Production execution uses class-specific native Rust adapters through one admitted
lifecycle. Python builds declarations, starts jobs and reads results; it performs no
mathematics. Pyomo and IDAES are isolated reference tools; there is no production Pyomo or
NL route and no compatibility compiler. *Because* one execution path keeps meaning and
failure semantics singular. See [§18](numerical-execution.md#section-18) and
[§21](workflows-and-results.md#section-21).

### D13. Cases and results never mutate the model

> Decision: [ADR-0016](../../adr/0016-cases-and-results-never-mutate-the-model.md),
> [ADR-0093](../../adr/0093-qualified-native-strategies.md)

A model revision defines structure; case and analysis bindings supply values, bounds, fixed
status and requests; an attempt records what happened. Initialization stages and continuation
steps are overlays on immutable bindings; a failed stage leaves the original specification
intact and is retained as evidence. *Because* mutation erases the history a result depends
on. See [§17](numerical-execution.md#section-17) and [§19](workflows-and-results.md#section-19).

### D14. Reuse follows complete declared dependencies

> Decision: [ADR-0089](../../adr/0089-semantic-identity-projections.md)

Every reusable product binds its complete inputs, absence states, implementation and
profile. Reuse requires equality of those dependencies. Program validity, cache retention,
storage retention and native worker lifetime are separate controls. Conservative
recomputation is valid; undeclared reads are not. *Because* a stale reuse is a silent wrong
answer. See [§14.4](mathematics-and-compilation.md#section-14-4).

## Retired section identities

#### 2.1 Doctrine crosswalk — retired

The principle crosswalk to a superseded charter is replaced by the
[selected design standard](../../design_review/design_principles/standard.toml) under
[ADR-0085](../../adr/0085-layered-design-standard.md).
