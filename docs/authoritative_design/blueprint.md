# Arrow-native IDAES core: detailed design architecture blueprint

**Status:** design blueprint, revision 4 (2026-09-13). One file, revised in git: revisions 2 and 3 are the tags `design-rev2` and `design-rev3` (ADR-0033); the *Revision history* table below lists every revision.
**Reviews:** `docs/design_review/reviews/design_review_arrow-native-idaes-core-blueprint_2026-09-13.md` (revision 1 → findings F1–F17) and `design_review_arrow-native-idaes-core-blueprint-rev2_2026-09-13.md` (revision 2 → findings R2-1–R2-8). Revision 3 resolves every finding of both reviews, including the items revision 2 had parked; the decisions taken are listed in *Revision history* and the residual risks in §26
**Follows:** `docs/authoritative_design/proposal.md` (the proposal this document makes concrete)
**Governing doctrine:** `docs/design_review/design_principles/DATA_MODEL_DESIGN_CHARTER.md` (DM-01–DM-60, gates G1–G7) applied through `AGENT_DESIGN_DIRECTIVE.md`. The P-numbered principles cited in §2.1 and §22.2 come from the doctrine texts this document was drafted against (*semantic_design_principles_holistic*, *Inference-Complete Process Metamodel Design Principles*, *semantic_math_basis*), which are not part of this repository.
**Library authorities:** `datafusion-pyarrow-rust-ref` skill corpus, the capability maps under `docs/capability-maps/` (DataFusion 55.1.0, Arrow 59.3.0, rustdoc extraction under `docs/capability-maps/evidence/`), the `external/idaes-pse` reading copy at tag 2.12.0 (`just fetch-external`; read for behaviour, never copied)

---

## Contents

- [Revision history](#revision-history)
- [0. Purpose, scope, and how to read this document](#0-purpose-scope-and-how-to-read-this-document)
- [1. Architectural summary](#1-architectural-summary)
- [2. Architectural decisions](#2-architectural-decisions)
- [3. Workspace, crates, and dependency pins](#3-workspace-crates-and-dependency-pins)
- [4. The semantic schema registry](#4-the-semantic-schema-registry)
- [5. Identity, versions, snapshots, and the catalog](#5-identity-versions-snapshots-and-the-catalog)
- [6. The canonical relation families](#6-the-canonical-relation-families)
- [7. The mathematical IR](#7-the-mathematical-ir)
- [8. Physical typing](#8-physical-typing)
- [9. Material systems and the property framework](#9-material-systems-and-the-property-framework)
- [10. Balance laws and control volumes](#10-balance-laws-and-control-volumes)
- [11. The unit model library as templates](#11-the-unit-model-library-as-templates)
- [12. Connectivity](#12-connectivity)
- [13. Flowsheets, time, and dynamics](#13-flowsheets-time-and-dynamics)
- [14. The compiler](#14-the-compiler)
- [15. Structural analysis and diagnostics](#15-structural-analysis-and-diagnostics)
- [16. Scaling as an explicit transformation](#16-scaling-as-an-explicit-transformation)
- [17. Initialization plans](#17-initialization-plans)
- [18. Backends](#18-backends)
- [19. Cases, results, and analytics](#19-cases-results-and-analytics)
- [20. Persistence, provenance, and reproducibility](#20-persistence-provenance-and-reproducibility)
- [21. The Python boundary and the Pyomo adapter](#21-the-python-boundary-and-the-pyomo-adapter)
- [22. Authoring and the extension model](#22-authoring-and-the-extension-model)
- [23. Observability and failure semantics](#23-observability-and-failure-semantics)
- [24. Testing and acceptance](#24-testing-and-acceptance)
- [25. Delivery phases](#25-delivery-phases)
- [26. Risks and open decisions](#26-risks-and-open-decisions)
- [Appendix A. IDAES capability coverage matrix](#appendix-a-idaes-capability-coverage-matrix)
- [Appendix B. Relation index](#appendix-b-relation-index)
- [Appendix C. Glossary](#appendix-c-glossary)

---

## Revision history

| Revision | Date | Change | git |
|---|---|---|---|
| 1 | 2026-09-13 | Initial blueprint following the proposal. | — |
| 2 | 2026-09-13 | Resolves the priority-1 findings of the design review of the same date. **F1** authored identity is assigned at creation and stored in the document; `qualified_name` is an attribute; `change_ops.op = rename`; case targets resolve to identities at commit (§5.1, §6.1, §6.10, §22.2). **F2** every `pse.expr_dsl` column is the sole authored form of an expression; parsed graphs live in `normalized.*_expr_*` produced by P3 (§6.6, §7.7). **F3** solver profiles and discretization policies are authored policy relations; `compiled` holds only bound plans (§6.10, §6.11, §14.1). **F4** undecided rule outcomes are written to `inferred.undecided`, never to a head relation (§6.7, §7.6, §14.2, §14.5). **F5** `affine_kind` is replaced by `scale_kind ∈ {point, difference}`; multiplication and division are unrestricted after canonical-unit normalization; the addition algebra is unchanged (§6.2, §7.2, §7.4, §8). **F6** a dedicated pass P12 (index expansion) turns indexed equation instances into scalar rows; P12–P15 of revision 1 are now P13–P16; §7 and §8.2 pass references corrected (§6.9, §7.1, §7.2, §14.1). **F7** P10 never folds parameter symbols; static attributes are computed on a case-bound view whose substitutions are recorded (§7.4, §7.5, §14.1, §14.4). **F9** the DataFusion engine is a declared input: `reference.engine_profiles`, explicit rule lists, semantic-settings hash and a `datafusion-proto` plan fingerprint enter the memo key and the pass record (§6.11, §14.2, §14.3, §20.2). **F15** a lifecycle test layer and the FFI/subprocess interruption rules (§18.3, §18.4, §20.1, §24.1). Also: dependency anchors raised to DataFusion 55.1.0 / Arrow 59.3.0 with family-wide `=` pins (§3.1), per the project owner's direction. | `design-rev2` |
| 3 | 2026-09-13 | Resolves every finding of the second review and every item revision 2 had parked, using the four capability maps' measurements. **R2-1** the plan fingerprint leaves the memo key; schema and field metadata are canonicalized at construction; the codec obligations, the analyzer rule list and a versioned settings allow-list are stated (§4.3, §14.2, §14.3, §6.11). **R2-2** `-0.0` preservation is qualified; `Float64` columns are never distinct or join keys in rule plans (§5.3, §14.2, §19.2). **R2-3** §4.3 corrected; the ten `pse.*` types are registered in the engine's extension-type registry (§4.3, §4.4, §5.4, §14.3). **R2-4** the Python boundary is rewritten as enforced contracts: pint validates and never defines, contract classes are generated, strict structuring, an `Any` lint, the msgspec/attrs division, shipped `pyarrow` extension classes with an idempotent registrar, the numpy null rule, pre-flight checks, exact pins and two interpreter ranges (§3.1, §3.3, §4.4, §21). **R2-5** every crate and Python library is pinned, a committed lockfile and `cargo deny` are required, `arrow-flight` and `uom` are dropped, §3.3 roles match use sites, `serde-saphyr` replaces `serde_yaml`, `num-dual` is 0.15 with FeOs conditional, `blake3` belongs to `pse-ids` with `derive_key` (§3.1, §3.2, §3.3, §5.1, §9.8). **R2-6** `object_store`, petgraph and faer statements corrected (§15.3–§15.5, §20.1). **R2-7** §5.4 states the measured pushdown shapes, purity, the wrapper test, `Precision` and selective statistics. **R2-8** `feedback_arc_set`, the formatter factory, `force_validate`, a `datafusion.pse.*` config extension, `pgjson` plans, the optimizer observer, `toml::Spanned`, `implicit_derivative`, and the Ipopt C names are adopted (§12.5, §14.2, §18.2, §18.3, §22.1, §23, §24.1). Parked items: **F8** rule bodies, invariant specs and selectors are typed relations (§4.1, §6.7, §6.10, §6.11); **F10** capability probes are runtime operations with recorded resolved options (§6.13, §18.3); **F12** the hashing contract names its constants and its scope (§5.3, §18.2); **F13** `salsa` is deferred behind an artifact-hash memo with a stated trigger (§14.3); **F14** derivation granularity is declared per relation (§4.1, §14.2); **F16** the bundle carries quantity types and a loss profile (§21.1); **F17** the SVD and condition-number routes are named and the pushdown precondition is discharged (§15.4, §15.5, §5.4). Also decided: `FairSpillPool` with an explicit limit (§14.3), `pyo3-arrow` retained (§21.1), `LogicalPlan::Extension` rule nodes deferred with a trigger (§14.2), IDAES 2.12.0 as the parity reference (§3.1, §6.14, §25), the commit contract stated (§22.2). | `design-rev3` |
| 4 | 2026-09-13 | Repository conventions: MSRV = pinned toolchain (ADR-0018); layout additions (ADR-0038); document paths under `docs/` (ADR-0033). | (this PR) |

## 0. Purpose, scope, and how to read this document

### 0.1 What this document is

The proposal established the thesis: *make mathematical and physical semantics the data model, then compile every executable representation from it.* This document is the blueprint that turns that thesis into things an implementer can build and test:

1. the exact **relation families** (Arrow schemas) that constitute the authoritative model;
2. the **mathematical IR** and its operator contracts;
3. the **compiler passes**, each with a declared contract;
4. the **kernel contract** for constitutive computation and how it generates DataFusion, native, and Pyomo adapters;
5. the **backends** (native evaluation program, Ipopt FFI, NL file, generated Pyomo);
6. the **coverage map** from every core IDAES-PSE capability to its platform realization;
7. the **workspace layout**, dependency pins, and delivery phases with acceptance tests.

It is written for engineers and programming agents who will implement the platform. Where a design decision could reasonably go two ways, the decision is stated with its rationale rather than left open.

This file is the authoritative design. Decisions that changed it are recorded under `docs/adr/` (ADR-NNNN); design reviews under `docs/design_review/reviews/` are evidence, not authority. Section numbers are stable citation targets: new material is inserted as a sub-section, never by renumbering.

### 0.2 What "core IDAES-PSE capabilities" means here

The scope is the modeling framework, not the example libraries. Concretely, the platform must reproduce the capability of these IDAES subsystems (source paths are relative to `idaes-pse/idaes/`):

| IDAES subsystem | Source | In scope |
|---|---|---|
| Process block lifecycle, config system, flowsheet and time domain | `core/base/process_block.py`, `process_base.py`, `flowsheet_model.py` | Yes |
| Unit model base, ports, arcs, translation | `core/base/unit_model.py`, Pyomo `network` | Yes |
| Control volumes 0D and 1D, all balance variants, geometry, reactions, holdup | `core/base/control_volume_base.py`, `control_volume0d.py`, `control_volume1d.py`, extended variants | Yes |
| Physical property framework: parameter blocks, state blocks, metadata, property sets, phases, components | `core/base/property_base.py`, `property_meta.py`, `property_set.py`, `phases.py`, `components.py` | Yes |
| Modular (generic) property framework: state definitions, EOS, pure-component methods, phase equilibrium, transport, bubble/dew | `models/properties/modular_properties/**` | Yes |
| Reaction framework and generic reaction package | `core/base/reaction_base.py`, `models/properties/modular_properties/reactions/**` | Yes |
| Generic unit model library | `models/unit_models/*.py` | Yes (as templates) |
| Costing framework and SSLW library | `core/base/costing_base.py`, `models/costing/SSLW.py` | Yes |
| Initialization framework and initializers, homotopy | `core/initialization/**`, `core/util/initialization.py`, `core/solvers/homotopy.py` | Yes (as plans) |
| Scaling toolbox (new and classic) | `core/scaling/**`, `core/util/scaling.py` | Yes (as transformations) |
| Model statistics and diagnostics toolbox, SVD, degeneracy hunter, convergence analysis | `core/util/model_statistics.py`, `model_diagnostics.py`, `diagnostics_tools/**` | Yes (as queries and native analyses) |
| Solver configuration, PETSc DAE integration, solver features | `core/solvers/**` | Yes (as solver profiles and backends) |
| Dynamic utilities and DAE discretization | `core/util/dyn_utils.py`, Pyomo `dae` | Yes (as lowering) |
| Serialization, tables, tags, units, constants, smooth math | `core/util/model_serializer.py`, `tables.py`, `tags.py`, `units_of_measurement.py`, `constants.py`, `math.py` | Yes |
| Parameter sweeps, utility minimization, convergence evaluation | `core/util/parameter_sweep.py`, `utility_minimization.py`, `convergence/**` | Yes |
| Helmholtz and CoolProp property backends | `models/properties/helmholtz/**`, `general_helmholtz/**`, `modular_properties/coolprop/**` | Provider adapters only |
| Surrogates (`core/surrogate`), DMF (`core/dmf`), UI (`core/ui`), Pecos, apps (`apps/**`), power-generation and gas-solid libraries (`models_extra/**`) | various | Out of scope; surrogate *embedding* is covered by the kernel contract |

Detailed capability-by-capability coverage appears in Appendix A.

### 0.3 Reading guide

| If you need to | Read |
|---|---|
| Understand the non-negotiable decisions | §2 |
| Set up the workspace and pins | §3 |
| Declare a new relation | §4, §6 |
| Understand identity, versions, snapshots | §5 |
| See every authoritative relation | §6 |
| Write or lower an equation | §7, §8 |
| Model a material system or property package | §9 |
| Build a balance, control volume, or unit template | §10, §11, §12 |
| Understand time, dynamics, discretization | §13 |
| Implement or extend a compiler pass | §14 |
| Implement diagnostics, scaling, initialization | §15, §16, §17 |
| Implement a backend or the Python adapter | §18, §21 |
| Persist, hash, or reproduce anything | §20 |
| Add a unit, property method, kernel, or costing method | §22 |
| Plan delivery and acceptance | §24, §25 |

### 0.4 Conventions used in this document

- Relation schemas are written as `namespace.relation @version` followed by columns as `name : ArrowType  [constraints]  -- meaning`. Arrow types use `arrow-schema` names. Semantic extension types are named `pse.<name>` and defined in §4.4.
- Equations are written in plain text; `d/dt` is the time derivative, `Σ_j` is a reduction over the named domain.
- "Authored" means written by a model author or package; "derived" means produced by a compiler pass and never authored; "reference" means shipped library data.
- IDAES names are quoted where the platform deliberately preserves them (for example the property name `enth_mol_phase`) so that parity tests can map both ways.

---

## 1. Architectural summary

### 1.1 The one-paragraph design

A process model is a **versioned snapshot of typed Arrow relations** organized into seven catalog namespaces (`reference`, `authored`, `normalized`, `inferred`, `compiled`, `runtime`, `provenance`). Authors write **packages** (material systems, property methods, unit templates, laws, costing methods) and **cases** (instances, connections, specifications) in a declarative authoring language that parses into `authored` relations. A **staged compiler** with contracted passes derives everything else: type and topology closure, property-demand closure, law expansion into equations, discretization, structural analysis, scaling and initialization plans, and finally a backend-neutral `CanonicalMathProblem`. **DataFusion** is the relational engine for the compiler's set-oriented passes, the snapshot catalog, kernel batch evaluation, and all analytics. **Native Rust numerics** (an evaluation program with automatic differentiation, sparse linear algebra, and structural graph algorithms) execute the problem in-process with Ipopt, or lower it to the AMPL NL format or to a generated Pyomo model. Every artifact is content-addressed, every derived row carries a derivation, and every run records the exact inputs that produced it.

### 1.2 The three representations, made concrete

| Representation | Namespace(s) | Authority | Produced by | Consumed by |
|---|---|---|---|---|
| `CanonicalModel` (engineering intent) | `authored`, `normalized`, `inferred` | Authored facts plus deterministic inference | Authoring parser, passes P0–P6 | Law expansion, projection, UI |
| `CanonicalMathGraph` (indexed symbolic mathematics) | `compiled.math_*` | Derived | Passes P7–P12 | Structural analysis, scaling, backends |
| `CanonicalMathProblem` (ordered executable problem) | `compiled.problems`, `variable_order`, `equation_order`, derived structure | Derived | Passes P13–P16 | Native backend, NL writer, Pyomo adapter, diagnostics |

The `runtime` namespace holds results and evidence; the `provenance` namespace holds derivations and pass records. No pass writes backward into `authored`.

### 1.3 Where each library sits

```text
authoring documents (YAML/TOML/Python builders)
        │  load (serde-saphyr, toml; both with source spans) + expression DSL parse (winnow) → authored relations
        ▼
 Semantic schema registry ──generates──► Arrow schemas · typed Rust views · serde_arrow adapters
        │                                 validators · docs · migrations · TableProviders
        ▼
 Snapshot catalog (DataFusion CatalogProvider/SchemaProvider/TableProvider over immutable artifacts)
        │
        ├─ Rule compiler: typed RulePlanSpec → LogicalPlanBuilder → RecordBatch   (closure, joins, selectors)
        ├─ Fixed-point executor (recursive closure with provenance)                (Rust over DataFusion plans)
        ├─ Native graph algorithms (petgraph + own matching/DM/BTD)                 (structural analysis)
        ├─ Math IR passes (Rust; memoized on declared-input content hashes)         (canonicalization, lowering)
        ├─ Kernel registry → generated ScalarUDFImpl wrappers                      (batch property evaluation)
        ▼
 CanonicalMathProblem
        ├─ Native evaluation program (instruction tape, per-equation reverse AD, faer sparse LA)
        │      └─ Ipopt C-API FFI in-process
        ├─ AMPL NL writer / SOL reader (ipopt, bonmin, couenne, cbc, scip, petsc_snes, petsc_ts, k_aug)
        └─ Generated Pyomo model via pyo3 + pyo3-arrow bundle (parmest, PyROS, GDPopt, parity)
        ▼
 runtime relations (solutions, residuals, duals, iterations, diagnostics) + provenance
        └─ DataFusion analytics: stream tables, KPIs, diagnostics reports, sweeps
```

---

## 2. Architectural decisions

Each decision below is binding for the implementation. "Because" states the rationale; "Consequence" states what it forces.

### D1. Typed relations are the only authority

> Decision: ADR-0004

Every durable fact about a model lives in a typed Arrow relation declared in the semantic schema registry (§4). There is no entity–attribute–value table, no JSON column that carries required behavior, and no Rust struct that is the "real" model with Arrow as an export.

*Because* the proposal's central claim (§2.1 of the proposal) is that structure declared once removes five parallel maintenance surfaces. *Consequence:* Rust structs that hold model data are generated views over relations, never hand-written mirrors.

### D2. Author causes, derive consequences

> Decision: ADR-0005

The authored model holds only primitive facts (topology, materials, quantities, phenomena, laws, options, specifications). Participation, roles, boundary classification, property requirements, method selection, equations, and orderings are derived by contracted passes and materialized with derivation provenance.

*Because* the metamodel doctrine (Principles 1–3, 10–12, 20) and because IDAES's largest maintenance burden is imperative `build()` methods that restate consequences. *Consequence:* an IDAES `build()` method becomes a template plus inference rules; a derived relation that an author tries to write is rejected or downgraded to an assertion.

### D3. Three artifact levels, one schema system

> Decision: ADR-0006

`CanonicalModel`, `CanonicalMathGraph`, and `CanonicalMathProblem` are stages in one catalog, share identity conventions, extension types, hashing, and provenance. They are not three applications.

### D4. Semantic IDs, artifact-local ordinals, and content hashes are distinct

> Decision: ADR-0007

Stable 128-bit semantic IDs identify authored and derived entities across revisions; `UInt32`/`UInt64` ordinals index compiled artifacts; blake3 content hashes identify immutable artifact versions. Names, row positions, dictionary codes, and solver positions are never identity (§5). An authored entity's identity is assigned when it is created and stored in its document; its `qualified_name` is an attribute that a `rename` change op may alter without touching any identity (§5.1, §22.2).

### D5. Physical type is more than a unit string

> Decision: ADR-0008

Every quantity-bearing column and every symbol carries a `quantity_type_id` resolving dimension vector, quantity kind, basis, reference state, affine semantics, and index shape (§8). Heterogeneous value columns require a per-row `quantity_type_id`. Null never means "unknown to be solved" (§7.6).

### D6. The math IR is richer than DataFusion `Expr`

> Decision: ADR-0009

Expression nodes, arguments, and typed operator payloads are relations; indexed operators (`SumOver`, `Gather`, `Broadcast`, `Derivative`, `Integral`, `ImplicitSystem`, `KernelCall`) survive until a backend requires scalarization. DataFusion `Expr` is used to *compute over* the IR, not to *be* the IR (§7).

### D7. Laws are templates over contributions

> Decision: ADR-0010

Material, energy, momentum, element, and charge balances, and the costing and utility-minimization laws, are instances of law templates expanded over a `contributions` substrate by the generic compiler (§10). Control volumes are template compositions that declare which contributions exist.

### D8. Property demand is resolved explicitly

> Decision: ADR-0011

There is no lazy attribute construction. A pass computes the closure of required properties from equations, selects providers by type and configured preference, persists the selection, and generates the property equations or kernel bindings (§9.6). Inspection never constructs physics.

### D9. Kernels have one contract and generated adapters

> Decision: ADR-0012

Every constitutive computation implemented in code is a `KernelSpec` with identity, signature (physical types), mathematical behavior, derivative availability, execution forms, failure behavior, and backend bindings. Scalar, batched Arrow, derivative, DataFusion UDF, NL external-function, and Pyomo bindings are generated from it (§18.5, §21.4).

### D10. DataFusion has four roles and no more

> Decision: ADR-0013

Relational assembly and inference (rule plans), snapshot catalog (providers), batch kernel evaluation (generated UDFs), and analytics (queries over `runtime`). It does not run inside Newton iterations, and a solve is never a scalar function call.

### D11. Native numerics own execution layouts

> Decision: ADR-0014

The evaluation program, sparse Jacobian structures, and solver workspaces are compiled artifacts derived from relations. They borrow Arrow buffers when layouts permit and copy when they do not. Zero-copy is a preference, not an obligation.

### D12. Pyomo is a generated, generic, coarse-grained backend

> Decision: ADR-0015

One adapter consumes a `CanonicalMathProblem` bundle over the Arrow C stream interface and constructs Pyomo objects. No IDAES class hierarchy is recreated. The adapter exists for parity testing and for Pyomo-ecosystem tools (parmest, PyROS, GDPopt, DAE utilities) until native equivalents exist.

### D13. Cases and results never mutate the model

> Decision: ADR-0016

A model revision defines structure; a case revision defines values, bounds, fixed status, objectives, and overlays; a run records what happened. Initialization stages, homotopy steps, and "fix then release" operations are immutable case overlays, not mutations.

### D14. Incrementality follows declared dependencies

> Decision: ADR-0017

Every pass declares its inputs; memoization is keyed by the content hashes of those inputs, the pass version, and — for passes that execute plans — the engine profile (§14.3). A value change invalidates only artifacts that depended on that value being a constant or satisfying a condition (§14.4, §14.5).

### 2.1 Doctrine crosswalk

| Doctrine principle | Realized by |
|---|---|
| Platform-independent semantics (holistic P9) | `authored`/`normalized`/`inferred` namespaces contain no backend concepts; backends bind in `compiled.backend_bindings` |
| Declarative single-sourcing (P10) | Schema registry generates all derived code and docs (§4) |
| Parse, don't validate; illegal states unrepresentable (P11, P12) | Authoring parser produces typed relations; enums are closed dictionaries; option conflicts are schema invariants |
| Stable semantic identity (P13) | §5 |
| Staged compilation, canonicalization before optimization (P14, P15) | Pass pipeline §14; canonicalization P3 precedes any lowering |
| Design by contract (P16) | `PassSpec`, `KernelSpec`, `RelationSpec` are relations with declared preconditions and postconditions |
| Functional core, imperative shell (P17) | Passes are pure functions of snapshots; the run controller and object store are the shell |
| Durable domain truth vs temporal control truth (P19) | Model/case revisions vs `runtime.run_state` |
| Unified mutation model (P20) | Change sets (§22.2) are the only write path into `authored` |
| Explicit failure semantics (P23) | Failure taxonomy §23 |
| Reproducibility and hermeticity (P25) | Manifests §20 |
| Primitive-fact minimality, roles as views, contributions substrate, laws as templates (metamodel P1, P3, P10, P12) | §6.4–§6.9, §10 |
| Unknown is not false (metamodel P18) | Inference predicates are four-valued; `unknown` becomes a diagnostic, never an exclusion (§14.3) |
| Rules typed, stratified, deterministic (metamodel P19) | `RuleSpec` relation and fixed-point executor (§14.2) |
| Solver strategy from static attributes (math basis §0.2) | Static analysis report and solve-plan selection (§18.7) |

---

## 3. Workspace, crates, and dependency pins

### 3.1 Version anchors

**Engine and runtime (Rust).** Every crate in both families is pinned with `=`; `Cargo.lock` is committed and CI asserts one version per family (`cargo tree` grouped by family) — `=` pins bind direct dependencies only, and `cargo tree -d` does not report a mixed family because it is not a duplicate.

| Component | Pin | Note |
|---|---|---|
| `datafusion` and every `datafusion-*` subcrate | **55.1.0** | Requires `arrow ^59.2.0` and `object_store ^0.13.2` (required by DataFusion, not chosen). `scan_with_args`/`ScanArgs`, `PhysicalPlanningContext`, `EnsureRequirements`, `is_strict`, `convert_to_state` are the 55 forms to code against; `convert_to_state` has no consumer unless a UDAF is written. Default features plus nothing else; `serde`, `avro`, `parquet_encryption`, `backtrace` off. |
| `arrow`, `arrow-*`, `parquet` | **59.3.0** | DataFusion 55.1.0 builds and tests against `arrow 59.2.0` (caret), so the platform runs one minor ahead of the Arrow the engine is exercised against — the first place to look if an Arrow-level behaviour surprises the engine. Features: `ipc`, `ffi`, `canonical_extension_types`; `force_validate` in the test and CI profiles only (§24.1); `ipc_compression` and `prettyprint` never on an identity path. `arrow-flight` and `arrow-avro` are not dependencies (no consumer). |
| `object_store` | **0.13.2** | The 0.13 line exposes `*_opts` methods only (`put_opts`, `get_opts`, `copy_opts`, `rename_opts`, `delete_stream`); §20.1 is written against that surface. |
| `pyo3` / `pyo3-arrow` | **0.29** / **0.19.0** | Retained over `arrow-pyarrow`: the platform needs the PyCapsule stream protocol and the numpy bridge that `pyo3-arrow` provides, and `arrow-pyarrow` requires a Python interpreter at build time, which breaks the offline build. `pyo3-arrow` depends on `thiserror 1.x` beside the platform's 2.x; no type crosses that boundary. `abi3-py311` wheels. |
| `tokio` | **1.53.1** | DataFusion runtime. |
| Rust edition / MSRV | **2024** / pinned stable toolchain (1.98.1 at revision 4; ADR-0018) | DataFusion 55's own floor is 1.94.0; the workspace declares only what CI exercises, and `tests/governance` keeps `rust-version` equal to `rust-toolchain.toml`. Every supporting crate below builds under it (verified by the capability-map extraction on a 1.100 nightly; the declared floors of the pre-1.0 crates are re-checked at each upgrade). |

**Supporting crates (Rust).** All pinned with `=`; the resolved set is the one the supporting-library capability map committed as a lockfile under `docs/capability-maps/evidence/rust/`.

| Crate | Pin | Owning crate | Note |
|---|---|---|---|
| `serde` | 1.0.229 | all | `derive`, `rc` |
| `serde_arrow` | 0.15.0 | `pse-relations` (adapters) | feature `arrow-59`; `SchemaLike::from_type`/`from_samples` banned on platform paths; any `SERDE_ARROW:*` field-metadata key is a contract violation (§5.3) |
| `syn` / `quote` / `proc-macro2` / `prettyplease` | 3.0.5 / 1.0.47 / 1.0.107 / 0.3.0 | `pse-schema` | `syn 3`, not the ecosystem's `syn 2`; `proc-macro2` `span-locations`; a `prettyplease` upgrade is a regeneration commit |
| `serde-saphyr` | 1.2.0 | `pse-authoring` | replaces the unmaintained `serde_yaml` (0.9.34+deprecated, last release 2024-03): typed errors with line/column, hostile input refused without panics, a `budget` bound on nesting, aliases and allocation |
| `toml` | 1.1.6 | `pse-authoring` | `Spanned<T>` supplies `pse.source_span` for `package.toml` |
| `winnow` | 1.0.4 | `pse-authoring` | already in the graph via `toml` |
| `petgraph` | 0.8.3 | `pse-structural`, `pse-compiler` | pre-1.0; SCC intra-component order is arbitrary and is sorted by semantic ID (§15.3); `greedy_feedback_arc_set` for tear search (§12.5) |
| `num-dual` | **0.15.0** | `pse-kernels`, `pse-numerics` | every kernel body is written against the 0.15 trait shape (`DualNum<Primitive = f64>`); `implicit_derivative*` for implicit kernels (§18.2) |
| `faer` | 0.24.4 | `pse-numerics`, `pse-diagnostics` | sparse LU, dense SVD, `pseudoinverse_from_svd_with_tolerance`, `matrix_free` (§15.4, §15.5) |
| `feos-core` / `feos` | 0.10.1 — **conditional** | `pse-kernels-ext` | requires `num-dual ^0.14`, whose trait shape differs from 0.15; the provider is enabled only when its upstream tracks 0.15 (§9.8). Imports the `quantity` crate, which is never an authority for units |
| `diffsol` | 0.16.2 — optional | `pse-backend-native` | carries its own linear algebra (`diffsol-la`/`diffsol-nl`); DiffSL never used (§13.6) |
| `egglog` | 3.0.0 — optional, phase 4 | `pse-mathir` | three majors in ten months; never hash its `Debug` rendering (§5.3) |
| `blake3` | 1.8.7 | **`pse-ids`** (sole hasher) | `derive_key` contexts for every 128-bit ID (§5.1); `rayon` feature under the §18.8 budget |
| `rayon` / `tokio` | 1.12.0 / 1.53.1 | `pse-numerics`, `pse-compiler` | one thread budget (§18.8) |
| `tracing` | 0.1.44 | all | late fields pre-declared as `field::Empty` (§23.1) |
| `thiserror` / `miette` | 2.0.20 / 7.6.0 | all / CLI only | `miette::Result` in the CLI and driver only; every `pse-*` crate returns concrete error enums |
| Ipopt C API | ≥ 3.14 | `pse-backend-native` (own `-sys` crate, bindgen output committed) | `GetIpoptCurrentIterate`/`GetIpoptCurrentViolations` (§18.3) |

**Python.** Every library pinned `==`. Two interpreter ranges: the platform package targets CPython ≥ 3.11 (3.14 verified); the parity environment is 3.10–3.13 (3.13 verified) because `idaes-pse` classifies only those. The two are separate dependency sets, so what IDAES drags in (`pydantic`, `sympy`, `networkx`, `pandas`) cannot become a platform dependency.

| Library | Pin | Role |
|---|---|---|
| `pyarrow` | 25.0.1 | capsule producer/consumer; the ten `pse.*` `ExtensionType` classes (§4.4) |
| `pyomo` | 6.10.1 | generated backend (§21.2); `pyomo.util.check_units`; `pyomo.contrib.*` recorded per run (§21.3) |
| `pint` | 0.26.1 | required by Pyomo's units machinery; validates, never defines (§3.3, §21.2) |
| `attrs` / `cattrs` | 26.1.0 / 26.2.0 | generated contract classes and strict structuring (§21.5) |
| `msgspec` | 0.21.1 | the manifest and every JSON/TOML the Python package reads or writes (§21.5) |
| `numpy` / `scipy` | 2.5.3 / 1.18.1 | array boundary (§21.6); test-time oracles only (§24.1) |
| `idaes-pse` | **2.12.0** (parity environment only) | the parity reference; the vendored `idaes-pse/` tree is a reading copy on the 2.10 line (§6.14) |

**Supply chain.** `cargo deny` (advisories, bans, maintenance) runs in CI in addition to `cargo audit`, because a deprecation encoded only in semver build metadata (`serde_yaml`'s) is invisible to `cargo audit`.

The vendored `arrow-rs/` checkout in this repository is on the 60.0.0 development line and is a reading copy only. The API facts extracted under `build/facts/arrow59-default` and `build/facts/df55-default` are the pinned surfaces to code against.

### 3.2 Cargo workspace layout

Crate names use the placeholder prefix `pse-` (process systems engineering); the prefix is a naming decision to confirm, the boundaries are not.

```text
Cargo.toml                       workspace; [workspace.dependencies] carries every pin of §3.1 once; Cargo.lock committed; CI asserts one version per family
crates/
  pse-schema/                    RelationSpec/LogicalType/ExtensionType registry; codegen (syn/quote/prettyplease)
  pse-relations/                 generated typed relation views, builders, validators, migrations (build output committed)
  pse-ids/                       SemanticId, ContentHash, ordinals, derived-ID rules (blake3 derive_key), canonical IPC serializer and hashing; the only crate that depends on blake3
  pse-quantity/                  dimensions, units, quantity kinds/types, bases, reference states, conversions
  pse-material/                  species, elements, phases, phase-species, reactions, stoichiometry, material systems
  pse-authoring/                 package/case document loader (serde-saphyr, toml), expression DSL parser (winnow), change sets
  pse-mathir/                    expression graph, operator catalog and contracts, canonicalization, unit inference
  pse-templates/                 template instantiation, feature guards, law templates, contributions
  pse-rules/                     RuleSpec → DataFusion LogicalPlan compiler; fixed-point executor; provenance rows
  pse-catalog/                   snapshot catalog: CatalogProviderList/SchemaProvider/TableProvider, artifact store
  pse-compiler/                  pass registry, PassSpec contracts, artifact-hash memo (§14.3), pipeline driver
  pse-structural/                incidence, bipartite matching, Dulmage–Mendelsohn, SCC/BTD, DOF, tear selection
  pse-kernels/                   KernelSpec registry; ideal/cubic EOS, pure-component correlations, Henry, transport, reactions
  pse-kernels-ext/               optional providers: CoolProp FFI, Helmholtz external libraries; FeOs when its upstream tracks num-dual 0.15
  pse-numerics/                  evaluation program, AD (per-equation reverse; num-dual for kernels), faer sparse LA
  pse-plans/                     discretization, scaling transformations, initialization plans, solver profiles
  pse-backend-native/            Ipopt C-API FFI driver over the evaluation program
  pse-backend-nl/                AMPL NL writer, SOL reader, external-function library exporter
  pse-backend-pyomo/             bundle assembly for the Python adapter (schemas, ordering, source maps)
  pse-diagnostics/               structural/numerical diagnostics as queries and native analyses; findings relations
  pse-runtime/                   run controller, case overlays, sweeps, result ingestion, lifecycle/cancellation
  pse-py/                        pyo3 + pyo3-arrow extension module (cdylib)
python/
  pse/                           Python package: authoring builders, Pyomo adapter, parity harness (attrs/cattrs contracts)
packages/
  reference/                     shipped reference packages: units, elements, species, property methods, unit templates
tests/
  golden/                        golden relation snapshots and fingerprints per vertical slice (data, not a crate)
  governance/                    workspace member crate pse-tests-governance (§24.1 governance layer)
  engine/                        workspace member crate pse-tests-engine
  conformance/                   workspace member crate pse-tests-conformance
  lifecycle/                     workspace member crate pse-tests-lifecycle
  structural/                    workspace member crate pse-tests-structural
crates/pse-ipopt-sys/            links = "ipopt"; build.rs emits link directives only; bindgen output committed (§18.3)
crates/pse-buildinfo/            lockfile hashes, rustc version, profile, git sha; feeds the manifest and pse.build_info()
xtask/                           codegen, family-check, governance, doc-lint, probe-host, release — the logic behind the justfile
benches/                         crate pse-benches: criterion, harness = false, one bench per §24.3 group
```

> Decision: ADR-0038 — the five `tests/*` crates, `pse-ipopt-sys`, `pse-buildinfo`, `xtask` and `benches` were added to this layout at revision 4; the phase-0 `ipopt` feature is default-on with `pse-ipopt-sys` optional.

Dependency direction is strictly downward in the list above within each layer; `pse-mathir`, `pse-quantity`, `pse-material`, and `pse-ids` depend on no engine crate; `pse-catalog` and `pse-rules` are the only crates that depend on DataFusion planning types; backends depend on `compiled` relation views only.

### 3.3 Supporting libraries and the boundary each must respect

> Decision: ADR-0021, ADR-0022, ADR-0026, ADR-0037

| Library | Role (matches its use sites) | Boundary |
|---|---|---|
| `serde`, `serde_arrow` | Schema-governed conversion between generated Rust records and Arrow batches at import/export boundaries (§20.5, §21) | Explicit schemas from the registry (`SchemaLike` for `Vec<FieldRef>`); `from_type`/`from_samples` banned by a governance grep; no `SERDE_ARROW:*` key may reach a relation schema |
| `syn`, `quote`, `proc_macro2`, `prettyplease` | Code generation from `RelationSpec` into typed views/builders and the Python contract classes (non-macro pipeline, emitted source committed) | Generated code is derived; the registry is authoritative |
| `serde-saphyr`, `toml`, `winnow` | Document loading (YAML/TOML, both with source spans) and expression DSL parsing | Parser output is `authored` relations; no evaluation during parsing; a parsing budget bounds hostile documents |
| `petgraph` | Two use sites: the unit-level topology multigraph and tear search (§12.5); Tarjan SCC and condensation for block triangularization (§15.3) | Graph facts persist in Arrow; matching and Dulmage–Mendelsohn are platform code; node indices are never identity; intra-component order is sorted by semantic ID |
| `egglog` (optional, phase 4) | Equality-saturation rewrites under explicit type/domain guards | Only rewrites justified by physical type, domain, and numerical policy; hand-written canonicalization (§7.4) stays complete on its own; adoption gated on cross-version extraction determinism |
| `num-dual` | Forward-mode and hyper-dual derivatives for kernels written generically over `DualNum`; `implicit_derivative*` for implicit kernels | Kernel-local; never a dense dual vector over the whole problem; one trait shape (0.15) for every kernel body |
| `faer` | Sparse LU for the structural and numerical diagnostics (§15.4), dense SVD and `pseudoinverse_from_svd_with_tolerance`, `matrix_free` operators with shift-invert for the smallest singular values (§15.5) | Per-diagnostic scratch built from `compiled.incidence`; not model truth; `Par` chosen by the §18.8 budget. The evaluation-program workspace and the KKT solves are not faer's (§18.2, §18.3) |
| `feos-core`/`feos` (conditional) | SAFT and cubic Helmholtz-energy providers; Helmholtz reference fluids are expression templates and a platform kernel, not FeOs (§9.8) | Capability providers under `KernelSpec`; enabled only when `feos-core` tracks `num-dual` 0.15; its `quantity` crate is never a units authority |
| `diffsol` (optional) | Native DAE trajectory backend through residual/Jacobian callbacks | Lowered from the IR; no second model language (DiffSL unused); brings its own linear-algebra stack, accepted for the trajectory backend only |
| `blake3` | Content hashing and derived identity, owned by `pse-ids` | `derive_key` contexts; 16-byte XOF prefix for 128-bit IDs; the canonicalizer, not the hash, is what tests cover (§5.3) |
| `object_store` | Artifact storage with atomic single-object writes and conditional updates (`put_opts` with `PutMode`) | Snapshot protocol is platform-owned (§20); no copy-then-delete sequence is a commit |
| `rayon`, `tokio` | Batch kernel evaluation and pass parallelism; DataFusion runtime | One thread budget covering kernels, the solver, hashing and diagnostics (§18.8) |
| `tracing` | Structured observability aligned to pass and run boundaries | Provenance is separate from observability (§23); DataFusion emits no spans of its own |
| `thiserror`, `miette` | Typed failure taxonomy with source anchors | Failure classes are data (§23); `miette::Result` only in the CLI and driver |
| Ipopt C API (own `-sys` crate) | In-process NLP solve | Called only by `pse-backend-native`; no panic or unwind crosses a callback (§18.3) |
| Python: `pyomo` (+`pint`), `pyarrow`, `attrs`/`cattrs`, `msgspec`, `numpy`/`scipy` (tests) | Generated backend, Arrow boundary, generated typed contracts, manifest codec, array boundary | §21: a given contract lives in exactly one class system; pint validates and never defines; numpy never receives a nullable column |

**Deferred with a stated trigger** (not dependencies today): `salsa` — adopt only if sub-pass memoization inside P7 proves necessary after measurement (§14.3); `LogicalPlan::Extension` rule nodes — adopt when rule attribution inside `EXPLAIN` is required by the diff report or agent tooling (§14.2); `datafusion-tracing`/`instrumented-object-store` — when a release matching the pinned engine exists (§23.1); `datafusion-ffi` — only if §22's extension model admits compiled third-party kernel packages.

Explicitly not added: a second dataframe engine, a graph database, a vector database, a second linear-algebra stack outside the trajectory backend, compile-time unit typing in kernels (`uom` was dropped: no kernel needs it — `KernelSpec` signatures are checked by P10's unit inference and by the per-kernel parity suites — and its stock SI system cannot express the currency dimension), external-source `TableProvider`s (a live database in the catalog is a second mutable authority), `datafusion-spark`/`enable_ansi_mode` (silently different arithmetic), Parquet virtual columns and `input_file_name` (position-based identity), `FunctionFactory` (SQL-defined kernels), `AsyncScalarUDFImpl` (I/O inside expression evaluation), `parquet-variant` (a typed hole in D1), `parquet_derive`/`typed-arrow` (they invert the registry→Rust direction), and any Python-side units library **as an authority**: `pint` is present because Pyomo's units machinery is pint; it validates what the adapter emits, it never defines a quantity, and no relation is derived from a pint object (§21.2).

---

## 4. The semantic schema registry

### 4.1 What is declared once

A relation is declared exactly once as a `RelationSpec`. The registry itself is stored as relations in `reference.schema_*` so that the platform can query its own schema, diff versions, and generate migrations.

```text
reference.schema_relations @1
  relation_id        : pse.semantic_id   PK
  namespace          : pse.enum(Namespace)      -- reference|authored|normalized|inferred|compiled|runtime|provenance
  name               : Utf8                     -- e.g. "stoichiometry"
  version            : UInt32
  authority          : pse.enum(Authority)      -- authored|reference|derived
  primary_key        : List<Utf8>               -- column names
  producing_pass_id  : pse.semantic_id  [nullable; required when authority = derived]
  derivation_granularity : pse.enum(DerivationGranularity)  -- row|rule (§14.2 rule 4); required when authority = derived
  stability          : pse.enum(Stability)      -- stable|evolving|internal
  doc                : Utf8

reference.schema_columns @1
  relation_id        : pse.semantic_id   FK schema_relations
  ordinal            : UInt16
  name               : Utf8
  logical_type_id    : pse.semantic_id   FK schema_logical_types
  nullable           : Boolean
  quantity_type_id   : pse.semantic_id  [nullable]   -- one quantity contract for the whole column
  per_row_quantity   : Boolean                        -- true ⇒ a sibling column named <name>_quantity_type_id exists
  fk_relation_id     : pse.semantic_id  [nullable]
  fk_column          : Utf8             [nullable]
  role               : pse.enum(ColumnRole)           -- key|reference|measure|label|payload|provenance
  doc                : Utf8

reference.schema_logical_types @1
  logical_type_id    : pse.semantic_id   PK
  name               : Utf8                     -- "semantic_id", "content_hash", "dimension_vector", "enum:MaterialBalanceType", ...
  arrow_storage      : Utf8                     -- canonical Arrow DataType rendering
  extension_name     : Utf8            [nullable] -- ARROW:extension:name when an extension type
  metadata_schema    : Utf8            [nullable] -- JSON Schema for ARROW:extension:metadata

reference.schema_enums @1
  enum_id            : pse.semantic_id
  member_ordinal     : UInt16
  member             : Utf8
  idaes_name         : Utf8            [nullable] -- parity mapping (e.g. "componentPhase")
  deprecated         : Boolean
  doc                : Utf8

reference.schema_invariants @1
  invariant_id       : pse.semantic_id   PK
  relation_id        : pse.semantic_id
  kind               : pse.enum(InvariantKind)  -- unique|foreign_key|check|cardinality|domain|closure|acyclic
  rule_id            : pse.semantic_id          -- the typed rule plan (reference.rule_plan_nodes, §6.11) that returns violating keys
  severity           : pse.enum(Severity)       -- error|warning
  doc                : Utf8

reference.schema_migrations @1
  relation_id, from_version, to_version, plan_spec, doc
```

### 4.2 What is generated from it

`pse-schema` runs as a build step (a non-macro `syn`/`quote`/`prettyplease` pipeline; emitted source is committed under `pse-relations/src/generated/` and diffed in CI) and produces, for every relation:

| Generated artifact | Form | Consumer |
|---|---|---|
| Arrow `Schema` with field and schema metadata (§4.3) | `fn schema() -> SchemaRef` | providers, writers, validators |
| Typed column view | `struct StoichiometryView<'a> { reaction_id: &'a FixedSizeBinaryArray, ... }` with `try_from(&RecordBatch)` that checks the schema fingerprint | passes, backends |
| Typed builder | `StoichiometryBuilder` with `push(row: StoichiometryRow)` and `finish() -> RecordBatch` | passes, authoring parser |
| `serde_arrow` schema | `SerdeArrowSchema` derived from the registry (never inferred from samples) | import/export adapters, Python contracts |
| Validators | one DataFusion `LogicalPlan` per invariant, executed by P2 | compiler |
| `TableProvider` | read-only provider over the snapshot artifact with exact key-filter pushdown and statistics | catalog |
| Documentation | markdown table per relation, cross-linked | `docs/generated/` |
| Migrations | `fn migrate_v1_to_v2(batch) -> batch` | artifact loader |
| Authoring JSON Schema | for editor validation of package documents | tooling |

Nothing hand-written may duplicate a column list. A hand-written struct that mirrors a relation is a governance failure (`tests/governance/no_shadow_structs.rs` greps generated symbol names against hand-written code).

### 4.3 Metadata conventions

Schema-level metadata keys (attached by the provider and preserved through IPC/Parquet):

```text
pse.contract.id             relation_id
pse.contract.version        relation version
pse.contract.fingerprint    blake3 of the RelationSpec rows
pse.namespace               namespace
pse.snapshot_id             snapshot the batch belongs to
pse.producer_pass_id        pass that produced a derived relation
```

Field-level metadata keys:

```text
pse.semantic.logical_type   logical type name
pse.semantic.quantity_type  quantity_type_id when the column carries one quantity contract
pse.semantic.role           ColumnRole
pse.semantic.fk             "<relation>.<column>"
pse.semantic.enum           enum_id for dictionary-encoded enums
ARROW:extension:name        for extension types (§4.4)
ARROW:extension:metadata    the extension's serialized metadata, mandatory beside the name when the type has metadata
```

`pse.*` metadata keys are a carrier, not the metamodel: nothing in DataFusion acts on them unless a platform component explicitly reads them (generated validators, providers, UDF return-field inference, the Python contract layer). The `ARROW:extension:*` pair is different: DataFusion 55.1 resolves `ARROW:extension:name` against the session's extension-type registry, rejects a field whose storage type does not match the registered type during planning, errors on an unregistered name, and preserves the key through projection, aliasing and grouping. The platform relies on that (§4.4, §14.3). Metadata is **canonicalized at construction**: every `Schema` and `Field` the platform builds goes through the generated constructor, which inserts keys in sorted order from a deterministic source, because both `Field::metadata` and `Schema::metadata` are hash maps whose iteration order is otherwise arbitrary and `datafusion-proto` serializes them in that order (§14.2 rule 5). A foreign key (`SERDE_ARROW:*` or any unregistered prefix) on a relation schema is a P2 contract violation, never a "volatile" key. Constraints: DataFusion validates no relational constraint except `Field` nullability, and not on ingest — `Constraints` declarations are informational to the planner, and P2 is what makes "primary key on every relation" true (§5.4).

### 4.4 Extension types

All extension types have a standard storage type so that an unaware consumer reads them safely.

| Extension name | Storage | Meaning |
|---|---|---|
| `pse.semantic_id` | `FixedSizeBinary(16)` | 128-bit semantic identity (§5.1) |
| `pse.content_hash` | `FixedSizeBinary(32)` | blake3 digest |
| `pse.dimension_vector` | `FixedSizeList<Struct<num: Int16, den: Int16>, 8>` | rational exponents over (length, mass, time, temperature, amount, current, luminous_intensity, currency) |
| `pse.quantity_value` | `Struct<value: Float64, quantity_type_id: FixedSizeBinary(16), unit_id: FixedSizeBinary(16)>` | a value in a heterogeneous column with explicit contract |
| `pse.bound` | `Struct<kind: Dictionary(Int8, Utf8), value: Float64>` with `kind ∈ {finite, unbounded}` | explicit bound; never NaN or null sentinels |
| `pse.index_tuple` | `List<FixedSizeBinary(16)>` | ordered domain member identities |
| `pse.ordinal_ref` | `UInt64` | artifact-local reference; metadata names the target relation |
| `pse.source_span` | `Struct<document_id: FixedSizeBinary(16), start: UInt32, end: UInt32>` | authoring provenance |
| `pse.enum` | `Dictionary(Int32, Utf8)` | closed enumeration; metadata carries `enum_id`; dictionary codes are presentation only |
| `pse.expr_dsl` | `Utf8` | authored expression text: the only authored form of an expression. Its parsed graph is a `normalized.*_expr_*` relation produced by P3 with a derivation to the source span (§7.7); no authored relation stores a parsed graph |
| `pse.target_path` | `Utf8` | an authored selector path naming instances, ports, symbols, or equations (`fs.H101.inlet.flow_mol`, `fs.H101.inlet.*`). P1 resolves it to entity identities at commit (`case_spec_targets` and kin, §6.10); the text is the serialization of those rows and is re-rendered by `rename` |

Rust implementations use `arrow_schema::extension::ExtensionType` — `NAME`, typed `Metadata`, `metadata`, `serialize_metadata`, `deserialize_metadata` (which must error on unexpected metadata), `supports_data_type`, `try_new`, and the provided `validate` and `try_new_from_field_metadata`, which are the read path. The generated `try_from(&RecordBatch)` calls `try_extension_type::<E>()` on every extension column beside the schema-fingerprint check; `Field::extension_type()` (the panicking variant) is banned by a governance grep. Every type's serialized metadata carries a `v` field, and the registration factory (below) accepts the metadata generations it knows and rejects the rest — that is the extension-type migration mechanism (DM-44, DM-51). One generated conformance test per type covers the metadata round trip, `supports_data_type` accept and reject, and reconstruction from a `Field`.

**Engine registration.** The ten types are registered in the DataFusion session's `ExtensionTypeRegistry` at session construction (`ExtensionTypeRegistration::new_arc(name, factory)` on a `MemoryExtensionTypeRegistry` preloaded with the seven canonical `arrow.*` types), generated from this table; the generator asserts that no registration replaces an earlier one. The engine then validates storage type and metadata for every plan, not only for the batches generated code constructs; paths that never touch a session (native buffer borrowing, §18.2; the Python boundary, §21) keep the generated `try_from` check. The registry cannot validate cross-field facts such as `pse.ordinal_ref`'s target relation, which is therefore part of the type's own `Metadata` (`target_relation_id`) so `deserialize_metadata` can enforce it.

**Rendering.** One `ArrayFormatterFactory` (Arrow) that also serves `DFExtensionType::create_array_formatter` (DataFusion) renders every `pse.*` value meaningfully — a semantic ID as its short form and owner name, a bound as `[lo, hi]`, a quantity value with its unit — in `EXPLAIN`, error messages and diagnostics, instead of raw storage bytes (§23.2).

**Python.** The platform package ships and registers the ten matching `pyarrow.ExtensionType` classes, generated from this table by the same generator, through an idempotent register-once helper (`pyarrow.register_extension_type` is process-global and raises on a second registration). An unregistered `pse.*` type in Python degrades silently to its storage type while `ARROW:extension:name` survives in the field metadata, so the contract layer asserts, per column, that every field claiming a `pse.*` extension resolved to a registered class (§21.1). The engine side errors on an unregistered name; the Python side degrades — §21.1's loss profile records that asymmetry.

### 4.5 Logical type catalog (physical scalars)

| Logical type | Arrow | Used for |
|---|---|---|
| `f64` | `Float64` | all numerical values in `compiled` and `runtime` |
| `i64` | `Int64` | counts, ordinals when signed needed |
| `u16`/`u32`/`u64` | `UInt16/32/64` | opcodes, ordinals, positions |
| `bool` | `Boolean` | flags |
| `text` | `Utf8` | names, docs, paths |
| `ts` | `Timestamp(Nanosecond, "UTC")` | run and revision timestamps |
| `list<T>`, `struct{...}` | `List`, `Struct` | variable-length payloads whose element semantics are uniform |

Dictionary encoding is applied at write time for enums and repeated labels; it never carries identity.

---

## 5. Identity, versions, snapshots, and the catalog

### 5.1 Three forms of identity

| Form | Type | Assigned by | Survives |
|---|---|---|---|
| Semantic ID | `pse.semantic_id` | authoring (package entities), inference/compilation (derived entities) | revisions, reordering, re-batching, projection, publication |
| Artifact-local ordinal | `UInt32`/`UInt64` | a compiled pass, deterministically | only within the artifact that defined it |
| Content hash | `pse.content_hash` | canonical serialization (§5.3) | forever; identifies an immutable version |

**Authored entity IDs.** Identity is assigned when an entity is created and is stored in the document that declares it (`id:` on every entity); it is never computed from the entity's name, so a rename changes an attribute and nothing else. Two policies exist, declared per package (`authored.packages.id_policy`):

- `explicit` (the default for authored packages): the authoring tool assigns a UUIDv7 when the entity is first written. P1 rejects an explicit-policy entity that lacks an `id` (`parse.missing_id`); `pse authoring assign-ids` inserts missing ones so that hand-written and agent-written documents stay valid.
- `named` (reference packages whose qualified names *are* the public contract: units, elements, constants, property kinds, the standard templates and methods): `semantic_id = blake3_128("pse:named:v1" ‖ package_id ‖ qualified_name)`. Under this policy a rename is by definition a new entity; the old one is deprecated with a `reference.aliases` row, and `change_ops.op = rename` is rejected.

Both forms are 128-bit and opaque; the policy is recorded on the package, never in the ID. References between entities — template submodels, connections, case targets — are stored by identity (§6.10), so no authored fact depends on a name.

**Derived entity IDs.** Anything a pass creates has an ID computed from what it was created from, never from row position or numerical values:

```text
symbol instance      = blake3_128("pse:symbol:v1" ‖ instance_id ‖ symbol_decl_id ‖ index_tuple)
equation instance    = blake3_128("pse:equation:v1" ‖ instance_id ‖ equation_decl_id ‖ index_tuple)
law-expanded term    = blake3_128("pse:term:v1" ‖ law_instance_id ‖ contribution_id ‖ index_tuple)
connection equation  = blake3_128("pse:conn:v1" ‖ connection_id ‖ member_ordinal ‖ index_tuple)
mesh node            = blake3_128("pse:node:v1" ‖ domain_id ‖ policy_id ‖ node_ordinal)
discretized symbol   = blake3_128("pse:symbol:v1" ‖ parent_symbol_id ‖ node_id)
```

`blake3_128(context ‖ parts)` denotes `blake3::Hasher::new_derive_key(context)`, updated with each part length-prefixed, and the first 16 bytes of the extendable output — a defined 128-bit digest (BLAKE3 is an XOF), not a truncation of an unrelated hash. `derive_key` and a truncated plain hash produce different bytes, so this choice is frozen with the `v1` context strings. `pse-ids` owns the `blake3` dependency; no other crate hashes.

Consequence: re-running compilation on an unchanged snapshot reproduces identical IDs; changing mesh resolution changes only mesh-dependent IDs.

**Ordinals.** Passes assign ordinals by sorting on semantic ID (or on a declared stable order such as domain member ordinal) so that ordinals are a pure function of the snapshot. Ordinals are stored alongside semantic IDs in `compiled.*` relations; cross-artifact references always use the semantic ID.

### 5.2 Model revision, case revision, run

```text
authored.model_revisions @1
  model_revision_id  : pse.semantic_id  PK
  parent_revision_id : pse.semantic_id  [nullable]
  snapshot_id        : pse.content_hash          -- of the authored+reference relations that define structure
  created_at, author, message

authored.case_revisions @1
  case_revision_id   : pse.semantic_id  PK
  model_revision_id  : pse.semantic_id  FK
  parent_case_id     : pse.semantic_id  [nullable]  -- overlay chain (§6.10)
  snapshot_id        : pse.content_hash
  created_at, author, message

runtime.runs @1  (see §6.13)
```

A change to a case never changes the model revision; a change to a template or connection creates a new model revision and invalidates compiled artifacts by dependency (§14.5), not wholesale.

### 5.3 Canonical serialization and hashing

> Decision: ADR-0023, ADR-0030

Content hashes must reflect semantic content, not batch layout. Content identity is defined over the **canonical IPC encoding only**; Parquet artifacts (§20.1) are never hashed (Parquet is deterministic within a writer version and preserves metadata, but embeds `created_by`, so its bytes change on upgrade while the data does not). The canonical serializer lives in `pse-ids` and is versioned as `pse.canon.v1`:

1. Order rows by primary key. Keys are compared with `arrow_row::RowConverter` (byte-comparable rows), which is the only kernel that orders `pse.index_tuple` (a `List`); `SortOptions` are explicit (`ASC NULLS FIRST`) and never defaulted.
2. Concatenate all batches into one (`concat_batches`); expand dictionaries to their value type; assert `DictionaryArray::is_normalized` before expansion in debug builds. This step is what makes "different batch splits hash identically" true — identical rows written as one batch and as two produce different IPC bytes.
3. Metadata: schema and field metadata are already canonical (sorted keys, §4.3); strip the volatile keys `pse.snapshot_id` and `pse.producer_pass_id`; keep the contract keys; a `SERDE_ARROW:*` or otherwise unregistered key is an error, not stripped.
4. Serialize to an Arrow IPC **stream** with `IpcWriteOptions::try_new(64, false, MetadataVersion::V5)` and `try_with_compression(None)` — alignment 64, no legacy format, metadata version V5, no compression. These four constants are part of `pse.canon.v1`; changing any of them is a new canonicalization version and invalidates every stored hash (alignment alone changes the bytes: 872 versus 1032 for identical rows).
5. Floats hash as IEEE-754 bits after `canonical_f64_bits` maps every NaN to the quiet NaN with zero payload and positive sign; `-0.0` is preserved. The function is applied on the hashing path only, never to data on its way to a solver, where a NaN's provenance matters for diagnostics (§18.2). Preservation of `-0.0` holds for the content hash and for Arrow ordering (IEEE 754 `totalOrder`: `null, -1, -0, 0, NaN` with nulls first); it does **not** hold under DataFusion `GROUP BY`, `DISTINCT` or hash joins, which merge `-0.0` with `+0.0` and treat NaN as self-equal — which is why rule plans never use a `Float64` column as a distinct or join key (§14.2 rule 7) and why §19.2 records the collapse for analytics.
6. `content_hash = blake3(ipc_bytes)`, computed by `pse-ids` over the bytes actually serialized (incrementally via `update`; `update_rayon` only under the §18.8 budget).
7. `snapshot_id = blake3(sorted list of (namespace, relation, version, content_hash))`.
8. Never hash a `Debug` or display rendering of anything: three libraries in the dependency set (Arrow `Schema` metadata, `datafusion-proto` field metadata, egglog's `TermDag`) leak hash-container iteration order into such renderings.

Two logically identical relations split into different batches hash identically; a change of reference-state convention changes the `quantity_types` relation and therefore the snapshot. Canonicalization property tests (§24.1) cover re-batching, metadata re-insertion order, dictionary re-encoding and a Parquet round trip.

### 5.4 The snapshot catalog

The catalog is a DataFusion `CatalogProviderList` whose catalogs are snapshots (named by `snapshot_id`, with mutable aliases such as `head` resolved to a snapshot at session creation), whose schemas are the seven namespaces, and whose tables are relations:

```sql
SELECT r.name, s.coefficient
FROM   "<snapshot_id>".authored.stoichiometry s
JOIN   "<snapshot_id>".authored.reactions r USING (reaction_id)
```

Provider contract (df §18, §51, §40A.5), stated against what DataFusion 55.1 was measured to do:

- `schema()` is the generated schema; cheap and stable; a session pins a snapshot so schemas cannot change mid-query. `SchemaProvider::table()` is async by signature but resolves from the manifest loaded at session creation and performs no I/O; registration methods reject calls after the session is sealed.
- `scan_with_args` is implemented; `scan` delegates. Projection is applied at read with `Schema::project` so metadata travels with the projected fields.
- **Pushdown.** Filters on key columns (`pse.semantic_id`, ordinals) and enum columns are handled `Exact`; everything else `Inexact` or `Unsupported`. `Exact` means the optimizer deletes the filter and nothing re-checks it — a wrongly advertised exact filter is a correctness bug. The shapes that actually reach a provider (measured): key equality arrives as `BinaryExpr(Column, Eq, Literal(FixedSizeBinary(16)))`; `IN (a, b)` arrives already rewritten to an `OR` chain of equalities, so the provider matches equality and `OR` chains and never `Expr::InList`; conjunctions arrive split; `IS NOT NULL` on a non-nullable column is eliminated before the provider sees it. `supports_filters_pushdown` is called more than once per plan and is therefore pure — no logging, counting or state. A test-only wrapper provider re-applies every filter advertised as `Exact` to the batches returned and fails on a survivor; it runs in CI over the golden snapshots (§24.1).
- **Statistics** are returned with explicit `Precision` per value: row counts from the manifest are `Exact`; min/max of key columns from the artifact footer are `Exact` for whole artifacts and `Inexact` for any filtered scan; everything not stored is `Absent`. `ScanArgs::statistics_requests()` is answered selectively from cached metadata; `Sum` and `DistinctCount` are `Absent`. `Statistics::new_unknown` is the starting point.
- **Constraints.** `Constraints::new_unverified([PrimaryKey])` on every relation; `Unique` only where P2 validates a `unique` invariant; `Constraints::project` under projection. DataFusion does not validate these — P2 does.
- **No mutation.** None of `insert_into`, `delete_from`, `update`, `truncate`, `merge_into` (all defaulted on the trait, and the set grows) is implemented; a governance test asserts the *set* of implemented `TableProvider` methods. `get_column_default` is not implemented (defaults are authored facts and overlays). `LogicalPlan::{Dml, Ddl, Copy, Statement}` are never constructed.
- **Extension types.** The session carries the `pse.*` extension-type registry (§4.4); a batch with a mismatched storage type is rejected during planning.
- **Configuration.** Session settings are assembled through `ConfigOptions::set` (typed errors); `SessionConfig::set_str` panics on an invalid value and is banned by a governance grep.
- The provider never prunes rows of a coupled mathematical problem for analytical convenience; analytical queries run over `runtime`/`compiled` relations, not over the problem the solver sees.

Artifacts behind providers are Arrow IPC files for compiled and runtime relations (hot path, zero-copy mmap) and Parquet for durable authored/reference/runtime history (§20).

---

## 6. The canonical relation families

This section is the schema catalog. Every relation is declared in the registry (§4) and appears here with its columns. Column types use the logical names of §4.4–§4.5; `sid` abbreviates `pse.semantic_id`, `hash` abbreviates `pse.content_hash`, `enum(X)` abbreviates `pse.enum` with enum `X`. `[n]` marks nullable. Enumerations are listed once, in §6.14.

### 6.1 Identity and packages (`authored`, `reference`)

```text
authored.packages @1
  package_id : sid PK · name : text · version : text (semver) · kind : enum(PackageKind)
  id_policy : enum(IdPolicy)                      -- explicit|named (§5.1)
  dependencies : list<struct<package_id: sid, version_req: text>> · content_hash : hash · doc : text

authored.documents @1
  document_id : sid PK · package_id : sid · path : text · content_hash : hash

authored.entities @1
  entity_id : sid PK · package_id : sid · kind : enum(EntityKind) · name : text · qualified_name : text   -- attributes, never identity
  parent_entity_id : sid [n]                      -- containment (flowsheet ⊃ unit ⊃ control volume ⊃ state block)
  source_span : pse.source_span [n]

reference.aliases @1             -- deprecated qualified names of named-policy entities (§5.1)
  alias_id : sid PK · entity_id : sid · old_qualified_name : text · deprecated_in : text (package version)

authored.model_revisions @1, authored.case_revisions @1   (§5.2)
```

Every other authored relation's primary key is also present in `authored.entities`; the invariant `closure:entity_registered` enforces it.

### 6.2 Physical types (`reference`, extendable by packages)

```text
reference.dimensions @1
  ordinal : u16 PK · name : text                   -- length, mass, time, temperature, amount, current, luminous_intensity, currency

reference.units @1
  unit_id : sid PK · symbol : text · name : text · dimension : pse.dimension_vector
  scale_to_canonical : f64 · offset_to_canonical : f64 · is_affine : bool · system : text · doc : text
  -- canonical units are SI base units; USD_<year> units have dimension currency and scale by CE index (§19.5)

reference.unit_sets @1
  unit_set_id : sid PK · name : text
  time_unit_id, length_unit_id, mass_unit_id, amount_unit_id, temperature_unit_id,
  current_unit_id [n], luminous_intensity_unit_id [n], currency_unit_id [n] : sid
  -- IDAES UnitSet; derived units (pressure, energy, ...) are computed, not stored

reference.quantity_kinds @1
  quantity_kind_id : sid PK · name : text · dimension : pse.dimension_vector
  extensive : bool · doc : text
  -- e.g. molar_flow, mass_flow, pressure, temperature, temperature_difference, molar_enthalpy, molar_entropy,
  --      molar_density, mass_density, mole_fraction, phase_fraction, area, volume, length, heat_duty, work, ...

reference.bases @1
  basis_id : sid PK · kind : enum(BasisKind)       -- molar|mass|volume|energy|standard_volume|dimensionless
  composition_basis : enum(CompositionBasis) [n]   -- mole_fraction|mass_fraction|volume_fraction|molality|molarity
  rate_basis : enum(RateBasis) [n]
  reference_conditions_id : sid [n]

reference.reference_states @1
  reference_state_id : sid PK · kind : enum(ReferenceStateKind)
  -- elemental_at_conditions | compound_at_conditions | ideal_gas_at_conditions | custom
  temperature : f64 [n] (K) · pressure : f64 [n] (Pa) · include_enthalpy_of_formation : bool · phase_id : sid [n] · doc : text

reference.quantity_types @1
  quantity_type_id : sid PK · quantity_kind_id : sid · basis_id : sid [n]
  reference_state_id : sid [n] · scale_kind : enum(ScaleKind)      -- point|difference (§8.1); gauge pressure is a point quantity whose reference state names the datum
  shape : list<enum(DomainKind)>                                  -- e.g. [phase, species] for x_{p,j}
  subject_kind : enum(SubjectKind) [n]                             -- species|phase|element|reaction|none
  canonical_unit_id : sid · nominal_magnitude : f64 [n]            -- default scaling hint (§16.2)
  doc : text

reference.conversion_rules @1
  conversion_id : sid PK · from_quantity_type_id : sid · to_quantity_type_id : sid
  kind : enum(ConversionKind)                                     -- scale|affine|kernel
  kernel_id : sid [n]                                             -- e.g. molar↔mass via molecular weight
  required_parameters : list<text>                                -- e.g. ["mw"]

reference.constants @1
  constant_id : sid PK · name : text · idaes_name : text [n] · value : f64 · unit_id : sid · quantity_kind_id : sid · doc : text
  -- gas_constant R, boltzmann k_B, avogadro N_A, faraday F, gravity g, planck h, speed_of_light c, stefan_boltzmann σ, ...
```

### 6.3 Domains, index sets, coordinates

```text
authored.domains @1
  domain_id : sid PK · owner_entity_id : sid · kind : enum(DomainKind)
  -- time|length|species|phase|phase_species|element|reaction|port_set|stage|cell|face|node|custom
  continuous : bool · unit_id : sid [n] · parent_domain_id : sid [n] · doc : text

authored.domain_members @1
  domain_id : sid · member_id : sid PK · ordinal : u32 · label : text
  coordinate : f64 [n]                                            -- for discrete points of a continuous domain
  ref_entity_id : sid [n]                                         -- species_id, phase_id, reaction_id, ...

authored.continuous_domains @1
  domain_id : sid PK · lower : f64 · upper : f64 · unit_id : sid
  initial_points : list<f64>                                      -- authored breakpoints (IDAES time_set / length_domain)
  discretization_policy_id : sid [n]                              -- FK authored.discretization_policies; cases may override (case_policies)

normalized.domain_products @1
  product_id : sid PK · domain_ids : list<sid>                    -- ordered factor domains of a symbol shape

inferred.valid_index_tuples @1
  product_id : sid · tuple : pse.index_tuple PK · derivation_id : sid
  -- e.g. valid (phase, species) pairs from phase_species; valid (time, length) grid after discretization

compiled.meshes @1
  mesh_id : sid PK · domain_id : sid · policy_id : sid · node_count : u32 · nodes : list<f64>
compiled.mesh_nodes @1
  node_id : sid PK · mesh_id : sid · ordinal : u32 · coordinate : f64 · kind : enum(NodeKind)   -- element_boundary|collocation|interior
compiled.stencils @1
  stencil_id : sid PK · mesh_id : sid · derivative_order : u8 · scheme : enum(DiscretizationScheme)
  node_id : sid · neighbor_node_id : sid · weight : f64
compiled.quadrature_rules @1
  rule_id : sid PK · mesh_id : sid · node_id : sid · weight : f64
```

### 6.4 Material systems (`authored`; elements are `reference`)

```text
reference.elements @1
  element_id : sid PK · symbol : text · name : text · atomic_mass : f64 (kg/mol)

authored.species @1
  species_id : sid PK · package_id : sid · name : text · formula : text [n]
  mw : f64 [n] (kg/mol) · component_type : enum(ComponentType)
  -- Component|Solute|Solvent|Ion|Anion|Cation|Apparent (a platform enum mirroring the IDAES class hierarchy)
  charge : i64 · dissociation_species : list<struct<species_id: sid, coefficient: f64>> [n]   -- ions / apparent species
  valid_phase_types : list<enum(PhaseType)>                        -- IDAES valid_phase_types
  doc : text

authored.species_elements @1
  species_id : sid · element_id : sid · count : f64                -- elemental_composition

authored.phases @1
  phase_id : sid PK · package_id : sid · name : text · phase_type : enum(PhaseType)   -- liquid|vapor|solid|aqueous|undefined
  is_solvent_phase : bool · doc : text

authored.phase_species @1        -- explicit restriction (IDAES per-phase component_list); optional
  phase_id : sid · species_id : sid

authored.henry_declarations @1
  species_id : sid · phase_id : sid · henry_type : enum(HenryType)   -- Kpx|Kpc|Hxp|Hcp
  method_id : sid                                                     -- reference.method_specs (ConstantH, ...)

authored.material_systems @1
  material_system_id : sid PK · package_id : sid · name : text
  species_ids : list<sid> · phase_ids : list<sid> · doc : text

inferred.phase_species @1        -- closure: species valid in phase (valid_phase_types ∩ phase_type, restricted by explicit lists)
  material_system_id : sid · phase_id : sid · species_id : sid · henry : bool · derivation_id : sid

authored.reactions @1
  reaction_id : sid PK · package_id : sid · name : text
  kind : enum(ReactionKind)                                          -- rate|equilibrium|inherent
  basis : enum(BasisKind)                                            -- molar|mass (reaction_basis)
  concentration_form : enum(ConcentrationForm) [n]                   -- molarity|molality|moleFraction|massFraction|partialPressure|activity
  reaction_phase_id : sid [n] · doc : text

authored.stoichiometry @1        -- one row per (reaction, phase, species) with a non-zero coefficient; zero rows are not stored
  reaction_id : sid · phase_id : sid · species_id : sid · coefficient : f64
  -- invariant closure:stoichiometry_species_in_phase ; element balance check is a diagnostic, not an invariant (IDAES allows unbalanced authored data)

authored.reaction_methods @1
  reaction_id : sid PK · rate_form_method_id : sid [n] · rate_constant_method_id : sid [n]
  equilibrium_form_method_id : sid [n] · equilibrium_constant_method_id : sid [n] · heat_of_reaction_method_id : sid [n]

authored.reaction_packages @1
  reaction_package_id : sid PK · package_id : sid · property_package_id : sid · reaction_ids : list<sid>
  unit_set_id : sid · default_arguments : list<struct<key: text, value: text>> · doc : text

authored.parameter_values @1     -- parameter_data for species, phases, reactions, packages, methods
  owner_entity_id : sid · parameter_kind : text                     -- method parameter name, e.g. "cp_mol_ig_comp_coeff"
  index : pse.index_tuple                                            -- e.g. ["A"] or [phase, species]
  value : f64 · unit_id : sid · source : text [n] · std_dev : f64 [n] · estimable : bool
```

### 6.5 Property capability registry (`reference`, extended by packages)

```text
reference.property_kinds @1
  property_kind_id : sid PK · idaes_name : text                      -- "enth_mol_phase", "dens_mass", "fug_phase_comp", ...
  quantity_kind_id : sid · basis_id : sid [n] · shape : list<enum(DomainKind)>
  category : enum(PropertyCategory)                                   -- state|thermo|transport|reaction|derived
  doc : text

reference.method_specs @1        -- a submodel that a package can select (IDAES "method" objects and EOS/state-definition modules)
  method_id : sid PK · family : enum(MethodFamily) · name : text · version : text
  -- MethodFamily: state_definition|eos|pure_component|phase_equilibrium_form|phase_equilibrium_state|bubble_dew|henry|
  --               transport_mixing|reaction_rate_form|rate_constant|equilibrium_form|equilibrium_constant|heat_of_reaction|
  --               enthalpy_transport|custom
  provides : list<sid>                                                -- property_kind_ids
  requires : list<sid>                                                -- property_kind_ids or state symbols needed
  parameter_kinds : list<struct<name: text, quantity_kind_id: sid, indexed_by: list<enum(DomainKind)>, required: bool>>
  realization : enum(MethodRealization)                               -- equation_template|kernel
  template_id : sid [n] · kernel_id : sid [n]
  validity : list<struct<input: text, lower: pse.bound, upper: pse.bound>>
  doc : text

authored.property_packages @1    -- IDAES GenericParameterBlock configuration, typed
  property_package_id : sid PK · package_id : sid · material_system_id : sid · unit_set_id : sid
  state_definition_method_id : sid                                    -- FTPx|FcTP|FpcTP|FPhx|FcPh|...
  temperature_ref : f64 (K) · pressure_ref : f64 (Pa) · include_enthalpy_of_formation : bool
  bubble_dew_method_id : sid [n] · doc : text

authored.state_bounds @1
  property_package_id : sid · state_symbol : text · lower : pse.bound · initial : f64 · upper : pse.bound · unit_id : sid

authored.phase_equilibrium_pairs @1
  property_package_id : sid · phase_a_id : sid · phase_b_id : sid
  state_method_id : sid                                                -- SmoothVLE|SmoothVLE2 (CubicComplementarityVLE)|...
  form_method_id : sid                                                 -- fugacity|log_fugacity per pair (may be overridden per species)

authored.method_selections @1    -- "which submodel computes what, for which scope"
  property_package_id : sid · scope_kind : enum(ScopeKind)             -- package|phase|species|phase_species|reaction
  scope_ids : pse.index_tuple · property_kind_id : sid [n] · family : enum(MethodFamily)
  method_id : sid · options : list<struct<key: text, value: text>>

authored.default_scaling @1
  property_package_id : sid · property_kind_id : sid · index : pse.index_tuple · scaling_factor : f64

inferred.property_requirements @1   -- explicit demand closure (§9.6)
  requirement_id : sid PK · state_scope_id : sid · property_kind_id : sid · index : pse.index_tuple
  requested_by : sid                                                    -- equation_id or requirement_id (transitive)
  derivation_id : sid

inferred.method_resolutions @1
  requirement_id : sid PK · method_id : sid · realization : enum(MethodRealization)
  template_id : sid [n] · kernel_binding_id : sid [n] · status : enum(ResolutionStatus)   -- resolved|unresolved|ambiguous
  derivation_id : sid
```

---

### 6.6 Templates (`authored`; reference packages ship the standard library)

A template is the declarative replacement for an IDAES `*Data` class with `CONFIG` and `build()`. One template kind covers unit models, control volumes, state blocks, reaction blocks, connection rules, costing methods, laws, initializers, and scalers.

```text
authored.templates @1
  template_id : sid PK · package_id : sid · name : text · version : text · kind : enum(TemplateKind)
  -- unit|control_volume|state_block|reaction_block|connection_rule|costing_method|law|initializer|scaler|flowsheet|helper
  default_initializer_template_id : sid [n] · default_scaler_template_id : sid [n]
  idaes_class : text [n]                                               -- parity mapping, e.g. "idaes.models.unit_models.Heater"
  doc : text

authored.template_params @1      -- CONFIG options
  template_id : sid · name : text · logical_type_id : sid · enum_id : sid [n]
  default : text [n] · required : bool · domain_spec : text [n]        -- validator (e.g. "is_property_package", "positive")
  doc : text

authored.template_features @1    -- typed choice structures (dynamic, has_holdup, has_pressure_change, thermodynamic_assumption, ...)
  template_id : sid · name : text · kind : enum(FeatureKind)           -- bool|enum|choice
  enum_id : sid [n] · default : text [n] · inherit_from : text [n]     -- "parent.dynamic" style inheritance
  doc : text

authored.template_feature_rules @1
  template_id : sid · rule : enum(FeatureRuleKind)                     -- implies|excludes|requires
  antecedent : text · consequent : text                                -- predicates over features/params ("dynamic=true" ⇒ "has_holdup=true")

authored.template_guards @1
  guard_id : sid PK · template_id : sid · predicate : pse.expr_dsl · doc : text

authored.template_domains @1     -- local index sets (length_domain, inlet_list, outlet_list, element lists, ...)
  template_id : sid · name : text · kind : enum(DomainKind) · continuous : bool
  members_from : text [n]                                              -- param that supplies members ("inlet_list") or bound package domain ("phase_list")
  bounds : struct<lower: f64, upper: f64> [n] · unit_id : sid [n]

authored.template_symbols @1     -- Var / Param / Expression / DerivativeVar declarations
  template_id : sid · symbol_decl_id : sid PK · name : text · role : enum(SymbolRole)   -- variable|parameter|expression|derivative|reference
  quantity_type_id : sid · indexed_by : list<text>                     -- template domain names ("time", "phase", "species", "length")
  default_lower : pse.bound [n] · default_upper : pse.bound [n] · default_initial : f64 [n]
  reference_to : text [n]                                              -- for role=reference: path to aliased symbol (IDAES Reference)
  wrt_domain : text [n]                                                -- for role=derivative
  guard_id : sid [n] · idaes_name : text [n] · doc : text

authored.template_equations @1
  template_id : sid · equation_decl_id : sid PK · name : text
  indexed_by : list<text> · filter : pse.expr_dsl [n]                  -- e.g. "j in phase_species[p]"
  expression : pse.expr_dsl · sense : enum(Sense) · family_hint : enum(EquationFamily) [n] · role_hint : enum(EquationRole) [n]
  guard_id : sid [n] · idaes_name : text [n] · doc : text
  -- the text is the authored fact. Its parsed graph is normalized.template_expr_* (P3; same schema as compiled.math_* with
  -- template-scoped references; every node carries a derivation to the source span). No authored relation stores a parsed graph.

authored.template_submodels @1   -- composition (Heater ⊃ control_volume ⊃ properties_in/out ⊃ ...)
  template_id : sid · name : text · child_template_id : sid [n]        -- null ⇒ resolved from a param (property_package → its state_block template)
  child_from_param : text [n] · multiplicity_domain : text [n]          -- indexed submodels (state blocks over time × length)
  bindings : list<struct<child_param: text, value: pse.expr_dsl>>       -- e.g. dynamic ← parent.dynamic, has_phase_equilibrium ← ...
  guard_id : sid [n]

authored.template_ports @1
  template_id : sid · name : text · kind : enum(PortKind)              -- material|heat|work|signal
  direction : enum(Direction) · bound_to : text                        -- submodel path whose state defines members ("control_volume.properties_in")
  guard_id : sid [n] · doc : text

authored.template_contributions @1   -- declared phenomena (heat, work, mass transfer, pressure change, reaction, custom terms)
  template_id : sid · contribution_decl_id : sid PK · name : text
  law_family : enum(LawFamily)                                         -- material|energy|momentum|element|charge|cost|utility
  subject : pse.expr_dsl [n]                                           -- selector: species j, phase p, element e, "total"
  expression : pse.expr_dsl · orientation : enum(Orientation)          -- into_scope|out_of_scope|generation|accumulation
  scope : text                                                          -- control region / submodel path
  guard_id : sid [n] · doc : text

authored.template_law_instances @1
  template_id : sid · law_instance_decl_id : sid PK · law_template_id : sid
  scope : text · subject_selector : pse.expr_dsl · options : list<struct<key: text, value: text>>   -- balance_type, ...
  guard_id : sid [n]

authored.template_requirements @1   -- capabilities the bound packages must provide
  template_id : sid · requirement : enum(CapabilityRequirement) · detail : text [n]
  -- material_flow_terms|enthalpy_flow_terms|material_density_terms|energy_density_terms|diffusion_terms|reaction_rate_basis|phase_equilibrium|...

authored.template_display @1     -- _get_performance_contents, define_display_vars, stream-table membership
  template_id : sid · kind : enum(DisplayKind) · label : text · expression : pse.expr_dsl · display_unit_id : sid [n] · format : text [n]
```

### 6.7 Instances, flowsheets, connectivity

```text
authored.instances @1
  instance_id : sid PK · parent_instance_id : sid [n] · template_id : sid · name : text
  param_values : list<struct<name: text, value: text>> · feature_values : list<struct<name: text, value: text>>
  property_package_id : sid [n] · reaction_package_id : sid [n] · doc : text

authored.flowsheets @1
  instance_id : sid PK · time_domain_id : sid · dynamic : enum(TriState)   -- true|false|inherit
  default_property_package_id : sid [n]

authored.scopes @1                -- intensional selectors (metamodel P4); the selector is a typed tree, not text
  scope_id : sid PK · root_term_id : sid
authored.selector_terms @1
  term_id : sid PK · scope_id : sid · parent_term_id : sid [n] · ordinal : u16
  op : enum(SelectorOp)                              -- descendant_of|kind_is|tagged_with|union|intersection|difference|include|exclude
  entity_id : sid [n] · entity_kind : enum(EntityKind) [n] · tag : text [n]

authored.connections @1           -- Arcs
  connection_id : sid PK · from_port_id : sid · to_port_id : sid · rule_template_id : sid   -- default: equality connection rule
  tear_cost : f64 [n]                                        -- tear policy for sequential initialization (§12.5); null ⇒ tearable at unit cost
  doc : text

inferred.instance_tree @1         -- transitive containment
  ancestor_id : sid · descendant_id : sid · depth : u16 · derivation_id : sid

inferred.instance_features @1     -- resolved features after inheritance and rules (dynamic ⇒ has_holdup, useDefault resolution)
  instance_id : sid · name : text · value : text · derivation_id : sid

inferred.instances @1             -- expanded submodel instances (control volumes, state blocks per (t,x), reaction blocks)
  instance_id : sid PK · parent_instance_id : sid · template_id : sid · path : text · index : pse.index_tuple · derivation_id : sid

inferred.ports @1
  port_id : sid PK · instance_id : sid · name : text · kind : enum(PortKind) · direction : enum(Direction) · state_instance_id : sid

inferred.port_members @1
  port_id : sid · ordinal : u16 · symbol_group : text                 -- "flow_mol", "mole_frac_comp", "temperature", "pressure"
  symbol_decl_id : sid · quantity_type_id : sid · derivation_id : sid

inferred.scope_members @1         -- scope_id, entity_id, derivation_id
inferred.boundary_crossings @1    -- scope_id, connection_id, classification : enum(Crossing) (internal|external|inbound|outbound), derivation_id
inferred.topology_edges @1        -- unit-level directed graph: from_instance_id, to_instance_id, connection_id
inferred.tear_candidates @1       -- cycle-breaking edges for sequential initialization (§17.4)

inferred.undecided @1             -- rule outcomes that are unknown or in conflict (§14.2 rule 3); head relations hold only decided-true rows
  undecided_id : sid PK · rule_id : sid · head_relation_id : sid · key : pse.index_tuple · truth : enum(TruthValue)   -- unknown|conflict
  reason : text · supporting : list<struct<relation_id: sid, row_key: text>> · derivation_id : sid
```

### 6.8 Symbols (`compiled`)

```text
compiled.symbols @1
  symbol_id : sid PK · ordinal : u64 · owner_instance_id : sid · symbol_decl_id : sid
  qualified_name : text                                                 -- fs.H101.control_volume.properties_out[0.0].temperature
  index : pse.index_tuple · quantity_type_id : sid · unit_id : sid       -- canonical unit of the package unit set
  role : enum(SymbolRole) · solver_type : enum(SolverVariableType)        -- continuous|binary|integer
  semantic_role : enum(VariableSemanticRole)                              -- state|design_capacity|allocation|slack|aux_reformulation|reporting_only|...
  lifecycle : enum(VariableLifecycle)                                     -- authored|generated_discretization|generated_reformulation|generated_relaxation|runtime_artifact
  default_lower : pse.bound · default_upper : pse.bound · default_initial : f64 [n]
  derivation_id : sid

compiled.symbol_references @1     -- alias links (IDAES Reference / VarLikeExpression)
  alias_symbol_id : sid · target_symbol_id : sid · kind : enum(AliasKind)   -- reference|display_alias

compiled.symbol_groups @1         -- named indexed families for ports, display, and lowering (flow_mol_phase_comp[p,j] over its product)
  group_id : sid PK · owner_instance_id : sid · name : text · product_id : sid
compiled.symbol_group_members @1  -- group_id, tuple, symbol_id
```

### 6.9 Mathematics (`compiled.math_*`)

```text
compiled.math_expr_nodes @1
  node_id : u64 PK · opcode : enum(Opcode) · quantity_type_id : sid [n]   -- null until unit inference has run; never null after P10
  scope_instance_id : sid [n] · subtree_hash : hash                        -- hash-consing key (§7.4)

compiled.math_expr_args @1
  parent_node_id : u64 · argument_ordinal : u16 · child_node_id : u64

-- typed operator payloads (one relation per operator family that needs data beyond children)
compiled.math_symbol_refs @1        node_id, symbol_id
compiled.math_float_constants @1    node_id, value : f64, unit_id : sid
compiled.math_int_constants @1      node_id, value : i64
compiled.math_affine @1             node_id, constant : f64, terms : list<struct<coefficient: f64, child_node_id: u64>>
compiled.math_reductions @1         node_id, kind : enum(ReductionKind) (sum|prod|min|max), domain_id : sid,
                                    bound_index_id : sid, filter_node_id : u64 [n]
compiled.math_gathers @1            node_id, group_id : sid, coordinate_map : list<struct<bound_index_id: sid, position: u16>>
compiled.math_broadcasts @1         node_id, domain_id : sid
compiled.math_derivatives @1        node_id, wrt_domain_id : sid, order : u8
compiled.math_integrals @1          node_id, domain_id : sid, quadrature_policy_id : sid [n]
compiled.math_smooth_ops @1         node_id, eps : f64                     -- smooth_max, smooth_min, smooth_abs, safe_sqrt, safe_log
compiled.math_conditionals @1       node_id, guard_node_id : u64            -- piecewise with explicit guard; lowered per backend (§7.2)
compiled.math_kernel_calls @1       node_id, kernel_binding_id : sid, output_ordinal : u16
compiled.math_implicit_refs @1      node_id, implicit_system_id : sid, unknown_ordinal : u16
compiled.math_unit_converts @1      node_id, scale : f64, offset : f64, from_unit_id : sid, to_unit_id : sid

compiled.math_indexed_equations @1   -- one row per (instance, equation declaration): the indexed form produced by P7–P9 and consumed by P12
  indexed_equation_id : sid PK · owner_instance_id : sid · equation_decl_id : sid [n] · qualified_name : text
  product_id : sid [n] · filter_node_id : u64 [n] · body_node_id : u64 · sense : enum(Sense) · lower_node_id : u64 [n] · upper_node_id : u64 [n]
  law_instance_id : sid [n] · derivation_id : sid

compiled.math_free_indices @1        -- the free indices of an indexed equation; P12 binds them to domain members
  indexed_equation_id : sid · bound_index_id : sid · domain_id : sid · position : u16

compiled.math_equations @1           -- scalar: one row per valid index tuple, produced by P12
  equation_id : sid PK · ordinal : u64 · owner_instance_id : sid · equation_decl_id : sid [n] · parent_indexed_equation_id : sid [n]
  qualified_name : text · index : pse.index_tuple
  body_node_id : u64 · sense : enum(Sense)                                -- eq|le|ge|range|definition
  lower_node_id : u64 [n] · upper_node_id : u64 [n]
  residual_quantity_type_id : sid
  family : enum(EquationFamily) · role : enum(EquationRole)               -- semantic_math_basis §1.2, §1.3
  differentiability : enum(Differentiability) · convexity : enum(Convexity) · monotonicity : enum(Monotonicity)
  default_active : bool · group_id : sid [n]
  law_instance_id : sid [n] · derivation_id : sid

compiled.math_objectives @1
  objective_id : sid PK · owner_instance_id : sid · body_node_id : u64 · sense : enum(ObjectiveSense) · quantity_type_id : sid · default_active : bool

compiled.math_implicit_systems @1
  implicit_system_id : sid PK · unknown_symbol_ids : list<sid> · equation_ids : list<sid>
  branch_policy : text · kernel_binding_id : sid [n]                        -- when encapsulated as an implicit kernel

compiled.math_complementarity @1
  pair_id : sid PK · expr_a_node_id : u64 · expr_b_node_id : u64 · formulation : enum(ComplementarityForm)   -- smooth_eps|binary|sos1

compiled.math_alternative_sets @1 / math_alternatives @1   -- typed optionality (mode, installed/absent, technology choice) for GDP lowering
compiled.math_dae_links @1          derivative_symbol_id : sid, state_symbol_id : sid, wrt_domain_id : sid

compiled.math_static_analysis @1    -- per-equation and per-variable static attributes (§18.7), computed by P14 on the case-bound view (§7.5)
  ... · parameter_dependence : list<sid>   -- treatment = parameter symbols whose substituted values the classification used
```

### 6.10 Cases, specifications, observations (`authored`)

```text
authored.cases @1
  case_id : sid PK · model_revision_id : sid · name : text · parent_case_id : sid [n]   -- overlay chain; child overrides parent
  kind : enum(CaseKind)                                                                -- base|overlay|initialization_stage|scenario|sweep_sample
  doc : text

authored.case_specs @1
  spec_id : sid PK · case_id : sid · target : pse.target_path   -- serialization of case_spec_targets (identity-based); re-rendered by rename
  treatment : enum(Treatment) [n]                             -- fixed|free|parameter
  value : f64 [n] · unit_id : sid [n]                          -- fixed or parameter value; null ⇒ untouched
  initial : f64 [n]                                            -- initial guess; null ⇒ no guess supplied
  lower : pse.bound [n] · upper : pse.bound [n]
  scaling_factor : f64 [n]
  priority : i32                                               -- overlay precedence within a case

authored.case_spec_targets @1    -- resolved by P1 at commit from case_specs.target; rename-stable because it references identities
  spec_id : sid · ordinal : u16 · instance_id : sid · member_kind : enum(TargetKind)   -- symbol|port|group|equation|instance_wildcard
  symbol_decl_id : sid [n] · port_decl_id : sid [n] · index : pse.index_tuple [n] · wildcard : bool

authored.case_activations @1     activation_id PK, case_id, target : pse.target_path, active : bool   -- resolved rows: case_activation_targets (same shape as case_spec_targets)
authored.case_objectives @1      case_id, objective_id, active : bool, weight : f64
authored.case_policies @1        case_id, discretization_policy_id [n] (FK authored.discretization_policies), scaler_template_id [n], initializer_template_id [n] (FK authored.templates), solver_profile_id [n] (FK authored.solver_profiles)

authored.datasets @1             dataset_id, name, source, content_hash
authored.observations @1
  observation_id : sid PK · dataset_id : sid · target : pse.target_path · value : f64 · unit_id : sid   -- resolved rows: observation_targets
  std_dev : f64 [n] · timestamp : ts [n] · tag : text [n]
authored.measurement_models @1   measurement_id, observation_id, model_expression : pse.expr_dsl, error_model : enum(ErrorModel), parameters
authored.scenarios @1            scenario_id, case_id, weight : f64, doc
authored.case_sets @1            case_set_id, base_case_id, generator_kind : enum(GeneratorKind) (grid|latin_hypercube|uniform_random|list), generator_params : list<struct<key: text, value: text>>, seed : u64 [n], sample_count
authored.case_set_samples @1     case_set_id, sample_ordinal, case_id
```

### 6.11 Numerical infrastructure (`reference`, `authored`, `compiled`)

```text
reference.kernel_specs @1        -- §18.5
  kernel_id : sid PK · name : text · version : text · provider : text · artifact_digest : hash
  behavior : enum(KernelBehavior)                              -- explicit|implicit
  inputs : list<struct<name: text, quantity_type_id: sid, shape: list<enum(DomainKind)>>>
  outputs : list<struct<name: text, quantity_type_id: sid, shape: list<enum(DomainKind)>>>
  parameters : list<struct<name: text, quantity_type_id: sid, indexed_by: list<enum(DomainKind)>>>
  smoothness : enum(Differentiability) · validity : list<struct<input: text, lower: pse.bound, upper: pse.bound>>
  monotonicity : list<struct<input: text, direction: enum(Monotonicity)>> · convexity : enum(Convexity)
  derivatives : list<enum(DerivativeKind)>                     -- jacobian|directional|adjoint|hessian
  execution_forms : list<enum(ExecutionForm)>                  -- scalar|batch_arrow|dual|hyperdual
  thread_safe : bool · failure_classes : list<enum(KernelFailure)>
  bindings : list<enum(BackendBinding)>                        -- native|datafusion_udf|nl_external_function|pyomo_external_function|pyomo_expression
  nl_function_name : text [n] · doc : text · test_suite : text

compiled.kernel_bindings @1
  binding_id : sid PK · kernel_id : sid · scope_instance_id : sid
  parameter_bindings : list<struct<name: text, symbol_id: sid [n], value: f64 [n], unit_id: sid [n]>>
  input_bindings : list<struct<name: text, node_id: u64>>

reference.pass_specs @1          -- §14.1
  pass_id : sid PK · name : text · version : text
  inputs : list<sid> · outputs : list<sid>                     -- relation_ids
  preconditions : list<sid> · postconditions : list<sid>       -- invariant_ids
  determinism : enum(Determinism) · cache_key_inputs : list<sid> · diagnostics : list<enum(FailureClass)>

reference.rule_specs @1          -- §14.2; the body is the typed plan below, never text
  rule_id : sid PK · version : text · stratum : u16 · head_relation_id : sid · root_node_id : sid
  negation : enum(NegationPolicy) · monotonic : bool · conflict_policy : enum(ConflictPolicy)

reference.rule_plan_nodes @1     -- the RulePlanSpec algebra as rows (one relation, typed payload columns)
  node_id : sid PK · rule_id : sid · op : enum(RulePlanOp)
  -- scan|filter|project|equi_join|anti_join|union|distinct|kernel_call|recursive
  relation_id : sid [n]                                        -- scan target
  join_keys : list<struct<left: text, right: text>> [n]         -- equi/anti join; key columns must be key-role (never Float64, rule 7)
  predicate_expr_id : sid [n]                                  -- reference.rule_expr_* graph (same schema as math_* restricted to boolean ops)
  projection : list<struct<name: text, expr_id: sid>> [n] · kernel_binding_id : sid [n] · doc : text
reference.rule_plan_edges @1     parent_node_id : sid · ordinal : u16 · child_node_id : sid
reference.rule_dependencies @1   -- derived at registry build from rule_plan_nodes; the queryable dependency facts
  rule_id : sid · relation_id : sid · mode : enum(DependencyMode)   -- read|negate|write
  stratum : u16 · derivation_id : sid

authored.discretization_policies @1   -- a policy (DM-13): authored or shipped by the reference package; never produced by a pass
  policy_id : sid PK · package_id : sid · name : text · method : enum(DiscretizationMethod)   -- finite_difference|collocation
  scheme : enum(DiscretizationScheme)                                          -- BACKWARD|FORWARD|CENTRAL|LAGRANGE_RADAU|LAGRANGE_LEGENDRE
  finite_elements : u32 · collocation_points : u8 [n]

compiled.scaling_plans @1        plan_id, case_id, template_id (scaler), options
compiled.variable_scales @1      symbol_id, scale : f64, offset : f64, source : enum(ScaleSource), derivation_id
compiled.equation_scales @1      equation_id, scale : f64, scheme : enum(ConstraintScalingScheme), derivation_id

compiled.initialization_plans @1 plan_id, case_id, template_id (initializer), doc
compiled.init_stages @1
  stage_id : sid PK · plan_id : sid · ordinal : u16 · kind : enum(StageKind)
  -- solve_subset|solve_blocks|propagate|apply_overlay|continuation|call_plan|check|restore
  target_kind : enum(StageTargetKind)                          -- instance|equation_set|blocks|connection|all
  target_ids : list<sid>                                       -- identities (instances, equations, blocks, connections); never paths
  overlay_case_id : sid [n] · solver_profile_id : sid [n] · tolerance : f64 [n]
  failure_policy : enum(FailurePolicy)                         -- abort|continue|retry_with_profile
  doc : text

authored.solver_profiles @1      -- a policy: the reference package ships ipopt.default and ipopt.user_scaled (§18.3); users add their own
  profile_id : sid PK · package_id : sid · name : text · backend : enum(Backend)    -- native_ipopt|nl_ipopt|nl_bonmin|nl_couenne|nl_cbc|nl_scip|nl_petsc_snes|nl_petsc_ts|pyomo
  solver : text · options : list<struct<key: text, value: text>>
  scaling_mode : enum(ScalingMode)                             -- user|gradient_based|none
  derivative_mode : enum(DerivativeMode)                       -- exact_hessian|limited_memory
compiled.solve_plans @1          plan_id, problem_id, class : enum(SolvePlanClass), justification : text, modifiers : list<text>

reference.engine_profiles @1     -- the relational engine as a declared input (§14.2 rule 5)
  engine_profile_id : sid PK · datafusion_version : text · arrow_version : text
  analyzer_rules : list<text> · optimizer_rules : list<text> · physical_rules : list<text>   -- ordered; installed explicitly
  semantic_settings : list<struct<key: text, value: text>>                                  -- an explicit, versioned key allow-list (§14.2 rule 5), never a namespace match
  setting_allow_list_version : text                                                         -- bumped when the allow-list changes
  content_hash : hash
```

### 6.12 Derived structure (`compiled`)

```text
compiled.problems @1
  problem_id : sid PK · model_revision_id : sid · case_id : sid · discretization_policy_ids : list<sid>
  variable_count, equation_count, inequality_count, objective_count : u64 · degrees_of_freedom : i64 · content_hash : hash

compiled.variable_order @1
  problem_id : sid · symbol_id : sid · position : u64 [n]      -- null when not in the solver vector (fixed/parameter/eliminated)
  treatment : enum(Treatment) · lower : pse.bound · upper : pse.bound · initial : f64 [n] · scale : f64 · offset : f64

compiled.equation_order @1
  problem_id : sid · equation_id : sid · position : u64 [n] · active : bool · scale : f64 · kind : enum(RowKind)   -- equality|inequality|objective

compiled.case_bound_substitutions @1   -- treatment = parameter symbols substituted on the case-bound view (§7.4 step 3, §14.1 P13/P14)
  problem_id : sid · symbol_id : sid · value : f64 · unit_id : sid · value_hash : hash · source : enum(ValueSource)   -- case_spec|package_default|template_default

compiled.incidence @1
  problem_id : sid · equation_id : sid · symbol_id : sid · linear : bool · coefficient : f64 [n]   -- coefficient when affine in this symbol

compiled.dm_partition @1
  problem_id : sid · kind : enum(DMKind) (equation|variable) · id : sid
  block : enum(DMBlock)                                         -- underconstrained|square|overconstrained
  matched_id : sid [n]                                          -- matching partner; null ⇒ unmatched

compiled.blocks @1               problem_id, block_id, kind : enum(BlockKind) (scc|connected_component|bordered_diagonal), order : u32, size : u32
compiled.block_members @1        block_id, kind (equation|variable), id
compiled.sparsity_patterns @1    problem_id, kind (jacobian|hessian), row_ptr : list<u32>, col_idx : list<u32>, content_hash
compiled.evaluation_programs @1  problem_id, program_id, artifact_hash, instruction_count, workspace_size
compiled.backend_bindings @1     problem_id, backend, status : enum(BindingStatus) (supported|unsupported), unsupported_opcodes : list<text>, artifact_hash [n]
```

### 6.13 Execution and evidence (`runtime`, `provenance`)

```text
runtime.runs @1
  run_id : sid PK · problem_id : sid · case_id : sid · model_revision_id : sid
  backend : enum(Backend) · solver_profile_id : sid · plan_id : sid [n] · stage_id : sid [n] · parent_run_id : sid [n]
  attempt : u16                                     -- 1 for the first attempt of (plan run, stage); retry_with_profile creates a new row with attempt + 1
  resolved_options : list<struct<key: text, value: text>>   -- the options actually passed to the solver after capability resolution (§18.3)
  started_at, finished_at : ts · status : enum(TerminationStatus)
  -- optimal|locally_infeasible|infeasible|unbounded|max_iterations|max_time|restoration_failed|solver_error|cancelled|user_interrupt
  environment : struct<platform_version: text, compiler_version: text, solver_version: text, kernel_digests: list<hash>, host: text>
  wall_seconds : f64 · iterations : u32 [n]

runtime.solutions @1             run_id, symbol_id, value : f64, unit_id, bound_status : enum(BoundStatus) (interior|at_lower|at_upper|violated)
runtime.duals @1                 run_id, equation_id, dual : f64, bound_multiplier_lower : f64 [n], bound_multiplier_upper : f64 [n]
runtime.residuals @1             run_id, equation_id, residual : f64, scaled_residual : f64, relative_residual : f64 [n]
runtime.iterations @1            run_id, iteration : u32, objective, inf_pr, inf_du, mu, step_size, regularization, restoration : bool
runtime.solver_events @1         run_id, ordinal, kind : enum(SolverEvent), message : text
runtime.diagnostics_findings @1  finding_id, subject_snapshot : hash, run_id [n], check_id : sid, severity : enum(FindingSeverity), subjects : list<sid>, values : text (JSON), message, next_steps : list<text>
runtime.kernel_evaluations @1    evaluation_id, kernel_binding_id, input_hash, output_batch_hash  -- batch evaluations outside solves (sweeps, initialization guesses)
runtime.host_capabilities @1     -- written by the explicit `probe_host` operation (§18.3); read when a run resolves its profile
  host : text PK · probed_at : ts · ipopt_version : text · linear_solvers : list<text> · hsl_available : bool · petsc_version : text [n]
  python_env : struct<interpreter: text, pyomo: text, pint: text, pyarrow: text, numpy: text, scipy: text, idaes: text [n], pyomo_contrib: list<struct<name: text, version: text>>> [n]

provenance.derivations @1
  derivation_id : sid PK · relation_id : sid · row_key : text · rule_id : sid [n] · pass_id : sid [n]
  supporting : list<struct<relation_id: sid, row_key: text>> · snapshot_id : hash · fingerprint : hash

provenance.pass_records @1       pass_run_id, pass_id, version, snapshot_in : hash, snapshot_out : hash, engine_profile_hash : hash [n],
                                 plan_fingerprints : list<hash> (canonicalized datafusion-proto bytes, evidence only — §14.2 rule 5),
                                 plan_explain : list<text> (EXPLAIN in pgjson format), rules_fired : list<struct<plan_ordinal: u16, rule_name: text, ordinal: u16>>,
                                 duration_ms, finding_count, status
provenance.assertions @1         assertion_id, package_id, expected : text (typed JSON), status : enum(AssertionStatus) (pass|fail|obsolete), reason
provenance.refs @1               name : text PK, snapshot_id : hash, updated_at : ts
```

### 6.14 Enumerations preserved from IDAES

These enumerations are closed dictionaries whose members carry `idaes_name` for parity. Member lists were taken from the vendored IDAES sources (2.10 line) and are re-verified against the pinned parity package, IDAES 2.12.0, by a governance test that compares `schema_enums` to the package's enum classes before phase 1; where IDAES declares members without an implementation, the platform declares them too and reports a missing law binding rather than silently accepting them.

| Enum | IDAES source | Members |
|---|---|---|
| `MaterialBalanceType` | `core/base/control_volume_base.py` | `useDefault`, `none`, `componentPhase`, `componentTotal`, `elementTotal`, `total` |
| `EnergyBalanceType` | same | `useDefault`, `none`, `enthalpyPhase`, `enthalpyTotal`, `energyPhase`, `energyTotal`, `isothermal` (only `enthalpyTotal`, `none`, `isothermal` have law bindings) |
| `MomentumBalanceType` | same | `none`, `pressureTotal`, `pressurePhase`, `momentumTotal`, `momentumPhase` (only `pressureTotal`, `none` bound; no `useDefault`) |
| `FlowDirection` | same | `notSet`, `forward`, `backward` |
| `MaterialFlowBasis` | `core/base/process_base.py` | `molar`, `mass`, `other` |
| `DistributedVars` | `core/base/control_volume1d.py` | `variant`, `uniform` |
| `PhaseType` | `core/base/phases.py` | `undefined`, `liquidPhase`, `vaporPhase`, `solidPhase`, `aqueousPhase` |
| `ComponentType` (platform enum) | `core/base/components.py` class hierarchy | `Component`, `Solute`, `Solvent`, `Ion`, `Anion`, `Cation`, `Apparent` |
| `StateIndex` | `modular_properties/base/utility.py` | `true`, `apparent` |
| `HenryType` | `modular_properties/phase_equil/henry.py` | `Hcp = 1`, `Hxp = 2`, `Kpc = 51`, `Kpx = 52` (values below 51 are constants `c/P`, above are volatilities `P/c`) |
| `ConcentrationForm` | `modular_properties/base/utility.py` | `molarity`, `activity`, `molality`, `moleFraction`, `massFraction`, `partialPressure` |
| `CubicType` | `modular_properties/eos/ceos_common.py` | `PR`, `SRK` |
| `FlashType` | `models/unit_models/feed_flash.py` | `isothermal`, `isenthalpic` |
| `MixingType`, `MomentumMixingType` | `models/unit_models/mixer.py` | `none`, `extensive`; `none`, `minimize`, `equality`, `minimize_and_equality` |
| `SplittingType`, `EnergySplittingType` | `models/unit_models/separator.py` | `totalFlow`, `phaseFlow`, `componentFlow`, `phaseComponentFlow`; `none`, `equal_temperature`, `equal_molar_enthalpy`, `enthalpy_split` |
| `ThermodynamicAssumption` | `models/unit_models/pressure_changer.py` | `isothermal`, `isentropic`, `pump`, `adiabatic` |
| `ValveFunctionType` | `models/unit_models/valve.py` | `linear`, `quick_opening`, `equal_percentage` (a custom callback becomes an authored helper template) |
| `HeatExchangerFlowPattern` | `models/unit_models/heat_exchanger.py` | `countercurrent`, `cocurrent`, `crossflow` |
| `ControllerType`, `ControllerMVBoundType`, `ControllerAntiwindupType` | `models/control/controller.py` | `P`, `PI`, `PD`, `PID`; `NONE`, `SMOOTH_BOUND`, `LOGISTIC`; `NONE`, `CONDITIONAL_INTEGRATION`, `BACK_CALCULATION` |
| `ConstraintScalingScheme` | `core/scaling/custom_scaler_base.py` | `harmonicMean`, `inverseSum`, `inverseRSS`, `inverseMaximum`, `inverseMinimum` |
| `DefaultScalingRecommendation` | same | `userInputRecommended`, `userInputRequired`, `userSetManually` |
| `InitializationStatus` | `core/initialization/initializer_base.py` | `Ok`, `none`, `Failed`, `DoF`, `PrecheckFailed`, `Error` |
| `DaeVarTypes` | `core/solvers/petsc.py` | `ALGEBRAIC`, `DIFFERENTIAL`, `DERIVATIVE`, `TIME` |
| `DiscretizationScheme` | Pyomo DAE | `BACKWARD`, `FORWARD`, `CENTRAL`, `LAGRANGE_RADAU`, `LAGRANGE_LEGENDRE` |
| SSLW costing enums | `models/costing/SSLW.py` | listed in §19.5 |

---

## 7. The mathematical IR

### 7.1 Design constraints

1. The expression graph is a DAG stored in `compiled.math_expr_nodes` and `math_expr_args`; the authoritative argument order is `argument_ordinal`. Packed adjacency arrays for evaluation are generated (§18.2), never maintained separately.
2. Indexed mathematics is preserved until a backend requires scalarization. A species balance over 1,000 cells is one indexed equation instance (`compiled.math_indexed_equations`) whose free index ranges over the cell domain until P12 (index expansion, §14.1) produces the scalar rows that structural analysis and the backends consume.
3. Every node carries a resolved `quantity_type_id` after P10. Unit inference is a pass, not a runtime check.
4. Every operator carries a contract (§7.3). A backend that cannot lower an operator fails explicitly at P16 with the unsupported opcodes listed in `compiled.backend_bindings`.
5. Free unknowns are symbols. A null in a value column never means "to be solved".

### 7.2 Operator catalog

| Opcode | Arity / payload | Dimensional rule | Derivative rule | Domain restriction | Smoothness | Native | NL | Pyomo | DataFusion |
|---|---|---|---|---|---|---|---|---|---|
| `Const` | payload value, unit | given | 0 | — | C∞ | ✓ | ✓ | ✓ | ✓ |
| `SymbolRef` | payload symbol | symbol's type | 1 w.r.t. itself | — | C∞ | ✓ | ✓ | ✓ | ✓ (column) |
| `Affine` | payload constant + (coef, child) list | all children same dimension; scale-kind algebra of §8.3 | coefficients | — | C∞ | ✓ | ✓ | ✓ | ✓ |
| `Mul`, `Div` | 2 | product/quotient of dimensions | product/quotient rule | `Div`: divisor ≠ 0 | C∞ off restriction | ✓ | ✓ | ✓ | ✓ |
| `Pow` | 2 | base dimension^exponent; exponent must be dimensionless constant unless base is dimensionless | standard | fractional exponent: base > 0; negative exponent: base ≠ 0 | C∞ off restriction | ✓ | ✓ | ✓ | ✓ |
| `Neg`, `Abs` | 1 | same | −1; sign | — ; `Abs` nonsmooth at 0 | C∞ ; C⁰ | ✓ | ✓ | ✓ | ✓ |
| `Exp`, `Log`, `Log10`, `Sqrt` | 1 | dimensionless in, dimensionless out (`Sqrt`: half dimension) | standard | `Log`: > 0; `Sqrt`: ≥ 0 (nonsmooth at 0) | C∞ off restriction | ✓ | ✓ | ✓ | ✓ |
| `Sin`, `Cos`, `Tan`, `Asin`, `Acos`, `Atan`, `Sinh`, `Cosh`, `Tanh`, `Erf` | 1 | dimensionless | standard | `Asin`/`Acos`: [−1, 1] | C∞ | ✓ | ✓ | ✓ | ✓ |
| `SmoothMax`, `SmoothMin`, `SmoothAbs` | 2 (or 1) + eps | same dimension for all | closed form (IDAES `math.py`: `0.5*(a+b+sqrt((a-b)^2+eps^2))`) | eps > 0 | C∞ | ✓ | ✓ (expanded) | ✓ (expanded) | ✓ |
| `SafeSqrt`, `SafeLog` | 1 + eps | as `Sqrt`/`Log` | closed form | eps > 0 | C∞ | ✓ | ✓ (expanded) | ✓ (expanded) | ✓ |
| `Conditional` | guard, then, else | then/else same dimension | branchwise | guard is a boolean node; nonsmooth at switch | C⁰ or discontinuous | ✓ | ✗ (must be reformulated, §7.5) | ✓ (Expr_if) | ✓ (CASE) |
| `SumOver`, `ProdOver`, `MinOver`, `MaxOver` | body + domain + bound index + optional filter | body dimension (`SumOver`), product (`ProdOver`) | termwise | — | as body (`MinOver`/`MaxOver` nonsmooth) | ✓ | ✓ (expanded) | ✓ (expanded) | ✓ (aggregate) |
| `Gather` | group + coordinate map | group's type | 1 | index must be valid tuple | — | ✓ | ✓ | ✓ | ✓ (join) |
| `Broadcast` | body + domain | body's | — | — | — | ✓ | ✓ (expanded) | ✓ (expanded) | ✓ |
| `Derivative` | body + domain + order | body dimension / domain unit^order | linear | continuous domain only | — | via discretization (P11) | via P11 | via P11 or `DerivativeVar` | ✗ |
| `Integral` | body + domain + quadrature | body dimension × domain unit | linear | continuous domain | — | via quadrature (P11) | via P11 | via P11 | ✗ |
| `KernelCall` | binding + output ordinal | kernel's output type | from `KernelSpec.derivatives` | kernel validity | kernel's | ✓ | ✓ only with `nl_external_function` binding | ✓ `ExternalFunction` or expression expansion | ✓ generated UDF |
| `ImplicitRef` | implicit system + unknown ordinal | unknown's type | implicit function theorem `dz/du = −G_z⁻¹ G_u` on the selected branch | G_z nonsingular | branchwise C¹ | ✓ | ✗ (must remain equations) | ✗ (must remain equations) | ✗ |
| `UnitConvert` | 1 + scale, offset | from → to | scale | offset ≠ 0 only when a point quantity is converted from an affine unit (°C, psig) to its canonical unit; inserted by P3/P9, so no node carries an affine unit after P10 | C∞ | ✓ | ✓ | ✓ | ✓ |
| `PiecewiseLinear` | breakpoints payload | given | slopes | — | C⁰ | ✓ | ✓ (SOS2 lowering) | ✓ (`Piecewise`) | ✓ |

"✓ (expanded)" means the backend receives scalar forms after P12 (index expansion) and P16 (backend lowering). `Conditional` is admitted in authored templates only under a guard whose value is decided at compile time (feature flags) or when the case selects a backend that supports it; otherwise it must be reformulated (smooth approximation, complementarity, or disjunction).

### 7.3 Operator contract record

Each opcode has a row in `reference.operator_specs` (generated from the `pse-mathir` operator table so that the code and the relation cannot drift):

```text
reference.operator_specs @1
  opcode : enum(Opcode) PK · arity : enum(Arity) (fixed n|variadic|payload)
  dimensional_rule : text · shape_rule : text · derivative_rule : text
  domain_restrictions : list<struct<argument: u16, relation: enum(RelationOp), bound: f64>>
  smoothness : enum(Differentiability) · convexity_rule : text · monotonicity_rule : text
  sparsity_rule : text · rewrite_conditions : list<text>
  lowering : list<enum(BackendBinding)>
```

Rewrite conditions are explicit: `x/x → 1` requires `x ≠ 0` provable from bounds; `log(exp(x)) → x` is unconditional; floating-point reassociation of `Affine` terms is allowed only under a numerical policy that opts in (default: canonical ordering by child hash, which is deterministic but not value-preserving to the last ulp across policies).

### 7.4 Canonicalization (pass P10)

1. **Hash-consing.** `subtree_hash = blake3(opcode ‖ payload ‖ child hashes)`; identical subtrees share one node. Commutative operators (`Affine` terms, `Mul`, `SmoothMax`, `SmoothMin`, reductions with no order dependence) sort children by hash before hashing.
2. **Affine normalization.** Chains of `Add`/`Sub`/constant-`Mul` collapse into one `Affine` node with merged coefficients. Authored `a - b` becomes `Affine{0, [(1,a), (−1,b)]}`. This exposes `AFFINE_EQUALITY`, `NETWORK_BALANCE`, and `SIMPLEX_ALLOCATION` families to static analysis.
3. **Constant folding.** Only among literal constants, unit-aware, and never through domain-restricted operators unless the argument is provably in-domain. Symbol references are never folded, whatever their role: a `role = parameter` symbol stays symbolic, so the canonical graph is a function of the model revision alone and hashes identically across cases. Case-dependent substitution of `treatment = parameter` values happens on the case-bound view (§14.1 P13/P14), never in the graph.
4. **Unit inference and conversion insertion.** Bottom-up: `Affine` children must share dimension and follow the scale-kind algebra of §8.3 (point ± difference → point; difference ± difference → difference; point − point → difference; point + point is an error unless the node is declared `weighted_mean`, the mixing case); `Mul`/`Div` combine dimension vectors and are unrestricted by scale kind, because by P10 every value is in a canonical ratio-scale unit; transcendental functions require dimensionless arguments; `Pow` requires constant exponent unless base is dimensionless. Where an authored constant carries a unit different from the package canonical unit, an explicit `UnitConvert` node is inserted; conversion is never implicit.
5. **Acyclicity.** Node ordinals are assigned in topological order; a cycle is a P10 failure (`math.cyclic_expression`).
6. **Index binding.** Reduction and gather bound indices are resolved to `bound_index_id`s; free indices of an equation template become the equation's `index` tuple.

### 7.5 Equation records

`compiled.math_equations` follows the `EquationRecord` shape of `semantic_math_basis.md` §1.1. Family and role are derived by P14 on the **case-bound view**: the canonical graph with every `treatment = parameter` symbol replaced by its `case_bound_substitutions` value. A coefficient is *constant* when it is a literal or a substituted parameter; `math_static_analysis.parameter_dependence` records which substitutions the classification used, so the same canonical graph may classify differently in a case that estimates a parameter (§19.4) than in one that fixes it, and the derivation says why:

| Detection | Family |
|---|---|
| single symbol with constant bound | `BOUND` |
| `Affine` body, `eq` | `AFFINE_EQUALITY`; if every coefficient ∈ {−1, +1} and the symbols are flows sharing a quantity type: `NETWORK_BALANCE` |
| `Affine` body, `le`/`ge`/`range` | `AFFINE_INEQUALITY` |
| `Affine` of fractions with constant 1 | `SIMPLEX_ALLOCATION` (sum of mole fractions) |
| product of two symbol-dependent subtrees | `BILINEAR` (flow × composition, flow × enthalpy) |
| transcendental of symbol-dependent subtree | `SMOOTH_TRANSCENDENTAL` (Antoine, Arrhenius, ideal-gas cp) |
| contains `SmoothMax`/`SmoothMin` | `NONSMOOTH_CONVEX` approximated; recorded as `SMOOTH_TRANSCENDENTAL` with a `smooth_approximation` flag |
| contains `KernelCall` without expression expansion | `BLACK_BOX` with derivative contract reference |
| `definition` sense | `REPORTING_DEFINITION` when no active equation depends on it, else `DEFINITION` |

Roles (`HARD_FEASIBILITY`, `DEFINITION`, `LINKING` for connection equations, `DOMAIN_GUARD`, `REPORTING`, `APPROXIMATION`) are assigned by the pass that created the equation and are part of the derivation.

### 7.6 Null, bound, and unknown semantics

| Situation | Representation |
|---|---|
| Variable to be solved | symbol with `treatment = free` in `compiled.variable_order` |
| No initial guess | `initial` null in `case_specs`; P13 substitutes the template default or the quantity type's nominal magnitude and records `ScaleSource`/`InitialSource` |
| Unbounded | `pse.bound{kind: unbounded}`; never `±inf`, NaN, or null |
| Missing measurement | null `value` in `observations` (a row may exist for provenance) |
| Inference predicate could not decide | a row in `inferred.undecided` (`truth = unknown` or `conflict`) plus a diagnostic; the head relation holds only decided-true rows, so an undecided fact can never be consumed as true, and the closure report (§14.5) stays open until the row is resolved |

### 7.7 The expression DSL

Authored equations are written in a small textual language. The text, stored in `pse.expr_dsl` columns, is the authored fact and the only form a change set may edit; its parsed graph is a derived `normalized.*_expr_*` relation produced by P3 with a derivation to the source span (§6.6), and a parse → render → parse round trip is the identity on the graph (a test in §24.1).

```ebnf
expr        := affine ;
affine      := term { ("+" | "-") term } ;
term        := factor { ("*" | "/") factor } ;
factor      := unary { "^" unary } ;
unary       := ["-"] primary ;
primary     := number [unit]
             | path [index]
             | function "(" [expr {"," expr}] ")"
             | "sum" "(" binder "|" expr ")"
             | "prod" "(" binder "|" expr ")"
             | "d(" expr ")/d" ident                    (* derivative w.r.t. a continuous domain *)
             | "integral" "(" binder "|" expr ")"
             | "kernel" "." ident "(" [args] ")"
             | "if" expr "then" expr "else" expr        (* Conditional; guard must be compile-time or backend-supported *)
             | "(" expr ")" ;
binder      := ident "in" path [ "where" expr ] ;
path        := ident { "." ident } ;
index       := "[" expr { "," expr } "]" ;
unit        := "{" unit_expr "}" ;                      (* e.g. 320{K}, 2{bar}, 1{mol/s} *)
function    := "exp" | "log" | "log10" | "sqrt" | "abs" | "smooth_max" | "smooth_min" | "smooth_abs" | "safe_sqrt" | "safe_log"
             | "sin" | "cos" | "tan" | "tanh" | "erf" | "min" | "max" | "convert" ;
```

Examples (heater energy balance and a mixer's smooth minimum pressure):

```text
sum(p in phase | control_volume.properties_out[t].enth_flow_phase[p])
  - sum(p in phase | control_volume.properties_in[t].enth_flow_phase[p]) - heat_duty[t] == 0

mixed_state[t].pressure == smooth_min(inlet_state[t, i].pressure for i in inlet_list, eps=1e-3{Pa})
```

The parser (`winnow`) produces nodes with unresolved paths; P3 (canonicalization) resolves paths against template symbols, submodels, and bound package domains. Any path that does not resolve is an authoring error with a source span.

---

## 8. Physical typing

### 8.1 QuantityType

A `quantity_type_id` resolves to the tuple (quantity kind, dimension vector, basis, reference state, scale kind, shape, subject kind, canonical unit, nominal magnitude). This is what allows the compiler to distinguish:

| Distinction | Encoded by |
|---|---|
| temperature vs temperature difference | `scale_kind = point` vs `difference` on kind `temperature`. Both are ratio-scale in the canonical unit K, so multiplication and division are unrestricted; the distinction governs addition (§8.3) and unit offsets (§8.2) |
| absolute vs gauge pressure | both `point` quantities of kind `pressure`; the gauge type carries a `reference_state_id` naming the datum, and conversion is a `conversion_rules.kind = affine` rule whose offset is the datum |
| molar vs mass-specific enthalpy | `basis_id` molar vs mass; conversion rule `molar↔mass` requires the `mw` kernel with the species or phase molecular weight |
| enthalpies with incompatible reference states | `reference_state_id`; addition of two enthalpies with different reference states is a P10 error |
| species composition vs phase composition | `shape = [phase, species]` with `subject_kind = species` vs `shape = [phase]` with `subject_kind = phase` |
| vector indexed by species vs by cells | `shape` domain kinds differ even when lengths match |

### 8.2 Units and unit sets

- Units are SI-anchored: `scale_to_canonical`, `offset_to_canonical`, and a dimension vector. Affine units (°C, °F, psig) carry `is_affine = true` and are legal only for `point` quantities; a `difference` quantity in an affine unit converts by scale only. P3 converts every authored constant, and P9 every method's natural units, to the canonical unit of the package unit set, so no value reaches P10 in an affine unit.
- A property package selects a `unit_set` (IDAES `UnitSet`: time, length, mass, amount, temperature, plus optional current, luminous intensity, currency). Every symbol declared by templates bound to that package is stored in the canonical unit of the set; the `unit_id` column of `compiled.symbols` records it.
- Method natural units: a correlation kernel (Antoine in bar and °C, a Perry's cp in J/(kmol·K)) declares its own input/output units in `KernelSpec`; P9 inserts `UnitConvert` nodes to and from the package canonical units. This reproduces the IDAES generic framework behavior of mixing methods written in different units, but every conversion is a visible node.
- Currency units (`USD_2018`, `USD_2023`, …) have dimension `currency`; conversion between years is a scale by the ratio of Chemical Engineering cost indices from `reference.cost_indices`, standardized through the CE=500 basis exactly as IDAES `register_idaes_currency_units` does (§19.5).

### 8.3 Unit inference algorithm (P10 step 4)

```text
infer(node):
  Const           → declared unit → quantity type by dimension match against the expected context, else a bare dimensioned literal
  SymbolRef       → symbol.quantity_type
  Affine          → all children same dimension; scale-kind algebra:
                      point ± difference → point ; difference ± difference → difference
                      point − point      → difference ; point + point → error unless the node is declared weighted_mean
                                                                        (coefficients sum to 1: mixing rules and averages)
                    result unit = canonical unit; insert UnitConvert for children in other units (never an affine unit at this stage)
  Mul/Div         → dimension vectors add/subtract; the result is a point quantity; operands are unrestricted, because every value is in a
                    canonical ratio-scale unit after P3/P9 — P/(R·T), T/1000{K} and exp(−E/(R·T)) type without any wrapping
  Pow             → exponent must be a dimensionless constant; base dimension scaled by the rational exponent
  Exp/Log/...     → argument must be dimensionless; result dimensionless
  SumOver/ProdOver→ body type; ProdOver over n members multiplies dimension n times (rejected unless dimensionless)
  Derivative      → body dimension / domain unit^order
  Integral        → body dimension × domain unit
  KernelCall      → KernelSpec output type; inputs converted to KernelSpec input units
  Gather/Broadcast→ group/body type
  Conditional     → then/else identical types
```

An equation's `residual_quantity_type_id` is the type of `body − (lower|upper)`; an equation whose sides differ dimensionally is a P10 error (this is the IDAES `assert_units_consistent` check, made structural).

### 8.4 Bases and reference states

- Material flow basis (`molar`, `mass`, `other`) is a property of the state definition and of each reaction package; conservation-law expansion (§10) converts contributions to the law's basis by inserting the conversion rule's kernel (`mw`) rather than summing incompatible quantities.
- Reference states record `T_ref`, `P_ref`, `include_enthalpy_of_formation`, and phase. Enthalpy and entropy quantity types carry the reference state of the package that computed them; a `Translator` template between packages with different reference states must author the offset explicitly (the compiler refuses to equate them).

### 8.5 Nominal magnitudes

`quantity_types.nominal_magnitude` supplies the default scaling hint (`ScaleSource = units`) used when no better information exists (§16). Packages override per property in `authored.default_scaling` (IDAES `default_scaling_factors`).

---

## 9. Material systems and the property framework

This section maps the IDAES Generic (Modular) Property Framework (`models/properties/modular_properties/**`) and the property base classes onto relations, templates, and kernels. The governing change: IDAES resolves properties lazily by attribute access (`build_on_demand` in `core/base/util.py`); the platform resolves them once, explicitly, in pass P6, and persists the resolution.

### 9.1 From `GenericParameterBlock` configuration to relations

| IDAES configuration | Platform relation / column |
|---|---|
| `base_units` | `property_packages.unit_set_id` → `reference.unit_sets` |
| `components[j].type` (`Component`, `Solute`, `Solvent`, `Ion`, `Anion`, `Cation`, `Apparent`) | `species.component_type` (an enum mirroring the class hierarchy; `charge` for ions; `dissociation_species` rows for `Apparent`) |
| `components[j].valid_phase_types` | `species.valid_phase_types` |
| `components[j].elemental_composition` | `species_elements` |
| `components[j].parameter_data` (`mw`, `pressure_crit`, `temperature_crit`, `omega`, `dens_mol_crit`, correlation coefficients) | `parameter_values` rows keyed by species and parameter kind, with units |
| `components[j].<method>` (`cp_mol_ig_comp: RPP4`, `pressure_sat_comp: NIST`, …) | `method_selections` rows with `scope_kind = species` and `family = pure_component` |
| `components[j].henry_component[p] = {method, type, basis}` | `henry_declarations` |
| `components[j].phase_equilibrium_form[(p1,p2)]` | `method_selections` with `scope_kind = phase_species`, `family = phase_equilibrium_form` |
| `phases[p].type`, `equation_of_state`, `equation_of_state_options` | `phases.phase_type`; `method_selections` with `family = eos` and `options` |
| `phases[p].component_list` | `authored.phase_species` |
| `phases[p].visc_d_phase`, `therm_cond_phase`, `surf_tens_phase`, `transport_property_options` | `method_selections` with `family = transport_mixing` |
| `state_definition`, `state_bounds`, `state_components` | `property_packages.state_definition_method_id`, `state_bounds`, `options.state_components` |
| `pressure_ref`, `temperature_ref`, `include_enthalpy_of_formation` | `property_packages` columns; a `reference_states` row |
| `phases_in_equilibrium`, `phase_equilibrium_state`, `bubble_dew_method` | `phase_equilibrium_pairs`, `property_packages.bubble_dew_method_id` |
| `inherent_reactions`, `reaction_basis` | `reactions` rows with `kind = inherent` owned by the package |
| `default_scaling_factors` | `default_scaling` |

The IDAES rule that every component must be valid in at least one phase, and the derived `phase_component_set`, become the inference rule `inferred.phase_species` (§14.2) plus invariant `closure:species_has_phase`.

### 9.2 State definitions are templates

Each IDAES state-definition module (`FTPx`, `FcTP`, `FpcTP`, `FPhx`, `FcPh`, `FpTPxpc`) becomes a `state_block` template shipped in the reference package. Its declarations are exactly the IDAES ones:

| Template | State symbols (role = variable) | Supporting symbols | Supporting equations | `always_flash` |
|---|---|---|---|---|
| `state.FTPx` | `flow_mol`, `mole_frac_comp[j]`, `temperature`, `pressure` | `flow_mol_phase[p]`, `mole_frac_phase_comp[p,j]`, `phase_frac[p]`; expression `flow_mol_phase_comp[p,j] = flow_mol_phase[p]·mole_frac_phase_comp[p,j]` | 1 phase: `flow_mol_phase[p0] = flow_mol`, `mole_frac_comp[i] = mole_frac_phase_comp[p0,i]`, `phase_frac[p] = 1`. 2 phases: `Σ_p flow_mol_phase[p] = flow_mol`, `flow_mol·mole_frac_comp[i] = Σ_p flow_mol_phase[p]·x[p,i]`, `Σ_i x[p_first,i] − Σ_i x[p_last,i] = 0`, `phase_frac[p]·flow_mol = flow_mol_phase[p]`. >2 phases: per-phase `Σ_i x[p,i] = 1`. Outlet only (`defined_state = false`): `sum_mole_frac_out: 1 = Σ_i mole_frac_comp[i]` | true |
| `state.FcTP` | `flow_mol_comp[j]`, `temperature`, `pressure` | `mole_frac_comp[j]` (variable) with `flow_mol_comp[j] = mole_frac_comp[j]·Σ_k flow_mol_comp[k]`; `flow_mol` expression | as FTPx per phase count | true |
| `state.FpcTP` | `flow_mol_phase_comp[p,j]`, `temperature`, `pressure` | expressions `flow_mol`, `flow_mol_phase[p]`, `flow_mol_comp[j]`, `mole_frac_comp[j]`, `phase_frac[p]`; variable `mole_frac_phase_comp[p,j]` with `x[p,j]·flow_mol_phase[p] = flow_mol_phase_comp[p,j]` | — | **false** |
| `state.FPhx` | `flow_mol`, `mole_frac_comp[j]`, `enth_mol`, `pressure` | `temperature` becomes a supporting variable; `enth_mol_eqn: enth_mol = Σ_p enth_mol_phase[p]·phase_frac[p]` | as FTPx | true |
| `state.FcPh` | `flow_mol_comp[j]`, `enth_mol`, `pressure` | as FcTP plus `enth_mol_eqn` | as FcTP | true |
| `state.FpTPxpc` | `flow_mol_phase[p]`, `mole_frac_phase_comp[p,j]`, `temperature`, `pressure` | `flow_mol` expression; `mole_frac_comp[j]` variable with `mole_frac_comp[j]·flow_mol = Σ_p x[p,j]·flow_mol_phase[p]`; `phase_frac[p]·flow_mol = flow_mol_phase[p]`; outlet-only per-phase `Σ_j x[p,j] = 1` | — | **false** |

Every state template declares the four interface contributions IDAES requires of a state block (`get_material_flow_terms(p,j)`, `get_enthalpy_flow_terms(p)`, `get_material_density_terms(p,j)`, `get_energy_density_terms(p)`) as named expressions in `template_requirements`/`template_display`:

```text
material_flow_term[p,j]     = flow_mol_phase_comp[p,j]
enthalpy_flow_term[p]       = flow_mol_phase[p] · enth_mol_phase[p]
material_density_term[p,j]  = dens_mol_phase[p] · mole_frac_phase_comp[p,j]
energy_density_term[p]      = dens_mol_phase[p] · energy_internal_mol_phase[p]
```

plus `material_flow_basis = molar`, `default_material_balance_type = componentTotal`, `default_energy_balance_type = enthalpyTotal`, `define_port_members = state symbols`, `define_display_vars` labels. Mole and phase fractions carry the IDAES bounds (`mole_frac`: `[1e-20, 1.001]`, `phase_frac`: `[0, ∞)`) as template defaults.

The `defined_state`/`always_flash` rule of IDAES becomes one inference rule producing `inferred.state_flash_required(state_instance)`:

```text
flash_required(S) ← state_instance(S, pkg) ∧ has_phase_equilibrium_pairs(pkg)
                  ∧ ( ¬defined_state(S) ∨ always_flash(template_of(S)) )
```

Only state instances with `flash_required = true` receive `_teq[pair]`, the phase-equilibrium constraints, and the smooth-VLE machinery.

### 9.3 The method registry

Every IDAES "method object" (a class exposing `build_parameters` and `return_expression`) becomes a `reference.method_specs` row. The parameter kinds it needs, in its natural units, are declared once; P9 inserts unit conversions to the package unit set (§8.2).

| Family | Shipped methods (IDAES name) | Realization | Provides | Parameter kinds (natural units) |
|---|---|---|---|---|
| `pure_component` | `NIST` (Shomate cp/h/s, Antoine Psat in bar), `RPP3` (cal-based polynomial cp/h/s, Antoine in mmHg), `RPP4` (J-based polynomial; Wagner Psat), `RPP5` (R-scaled polynomial; Antoine in °C), `Perrys` (liquid cp/h/s polynomial in kmol; density eqn 1 and 2), `ConstantProperties` (constant cp, ρ, μ, λ), `ChapmanEnskogLennardJones` (viscosity), `ChungViscosityPure`, `Eucken` (thermal conductivity), CoolProp exponential/non-exponential/polynomial forms | equation template | `cp_mol_{ig,liq,sol}_comp`, `enth_mol_*_comp`, `entr_mol_*_comp`, `dens_mol_liq_comp`, `vol_mol_liq_comp`, `pressure_sat_comp`, `visc_d_phase_comp`, `therm_cond_phase_comp` | e.g. NIST `A..H` with units J/mol/K, J/mol/K/kK, …, kJ/mol; Antoine `A` (dimensionless), `B`, `C` (K); Wagner `A..D` dimensionless; Perry's `C1..C5` J/kmol/K^n |
| `eos` | `Ideal`, `Cubic` (PR, SRK; van der Waals one-fluid mixing rules), `ENRTL` (Song–Chen symmetric eNRTL; phase 4) | equation template plus one implicit kernel (cubic root) | the full EOS interface (§9.4) | `omega`, `pressure_crit`, `temperature_crit`, `mw` per species; `PR_kappa[i,j]`/`SRK_kappa[i,j]` per package; eNRTL `alpha`, `tau` |
| `phase_equilibrium_form` | `fugacity`, `log_fugacity` | equation template | equilibrium constraint per (pair, species) | — |
| `phase_equilibrium_state` | `SmoothVLE`, `CubicComplementarityVLE` (SmoothVLE2) | equation template | `_teq` constraints, slacks | `eps_1 = 0.01 K`, `eps_2 = 0.0005 K`; `eps_t`, `eps_z` (flow units) |
| `bubble_dew` | `IdealBubbleDew`, `LogBubbleDew` (default) | equation template | `temperature_bubble/dew`, `pressure_bubble/dew` and the transition compositions | — |
| `henry` | `ConstantH` with `HenryType ∈ {Hcp, Hxp, Kpc, Kpx}` | equation template | `henry[p,j]` | `henry_ref[p]` in the type's units |
| `transport_mixing` | `ViscosityWilke` (`wilke` or `herning_zimmer` φ_ij), `ThermalConductivityWMS`, `NoMethod` | equation template | `visc_d_phase`, `therm_cond_phase` | — |
| `reaction_rate_form` | `power_law_rate` | equation template | `reaction_rate[r]` | `reaction_order[p,j]` |
| `rate_constant` | `arrhenius` | equation template | `k_rxn[r]` | `arrhenius_const` (units derived from basis and concentration form), `energy_activation` (J/mol) |
| `equilibrium_form` | `power_law_equil`, `log_power_law_equil`, `solubility_product`, `log_solubility_product` | equation template (solubility forms use `SmoothMax`) | equilibrium constraint | `eps`, `s_norm`, `s_scale` |
| `equilibrium_constant` | `ConstantKeq`, `van_t_hoff`, `gibbs_energy` | equation template | `k_eq[r]`, `log_k_eq[r]` | `k_eq_ref`, `T_eq_ref`, `ds_rxn_ref` |
| `heat_of_reaction` | `constant_dh_rxn` | equation template | `dh_rxn[r]` | `dh_rxn_ref` |
| external providers | Helmholtz (IDAES `general_helmholtz` parameter files), CoolProp coefficient source, FeOs models | kernel (`KernelSpec`) | per provider capability list | provider-specific |

Two representative method templates in the authoring language:

```yaml
method: pure.NIST.cp_mol_ig_comp@1
  family: pure_component
  provides: [cp_mol_ig_comp]
  parameters:
    - {name: A, quantity: heat_capacity_molar, unit: "J/mol/K"}
    - {name: B, unit: "J/mol/K/kK"}   # ... C, D, E, F, G, H as in NIST.py
  inputs: [{name: T, quantity: temperature, unit: K}]
  output_unit: "J/mol/K"
  expression: "A + B*t + C*t^2 + D*t^3 + E/t^2 where t = T/1000{K}"

method: rate_constant.arrhenius@1
  family: rate_constant
  provides: [k_rxn]
  parameters:
    - {name: arrhenius_const, unit: derived}      # unit derived by P9 from basis and concentration form
    - {name: energy_activation, unit: "J/mol"}
  expression: "arrhenius_const * exp(-energy_activation / (R * state.temperature))"
```

Unit derivation for `arrhenius_const` follows the IDAES rule exactly: for `moleFraction`, `massFraction`, `activity` forms the rate unit is `amount·volume⁻¹·time⁻¹` (or mass basis); otherwise the concentration unit is raised to the total reaction order. This rule is a `conversion_rules.kind = kernel` entry, not code inside the template.

### 9.4 The equation-of-state contract

An `eos` template must provide expression declarations for the property kinds IDAES's `EoSBase` enumerates. The reference `eos.ideal` template ships the complete equation set:

| Property kind | Ideal expression |
|---|---|
| `act_phase_comp[p,j]`, `act_coeff_phase_comp[p,j]` | `x[p,j]`, `1` |
| `compress_fact_phase[p]` | vapor `1`, else `0` |
| `cp_mol_phase[p]`, `cp_mol_phase_comp[p,j]` | `Σ_j x[p,j]·cp_comp[p,j]`; comp from the pure method selected for the phase type (ig, liq, sol) |
| `cv_mol_phase_comp[p,j]` | vapor `cp_ig − R`; liquid/solid `cp` |
| `dens_mol_phase[p]`, `vol_mol_phase[p]` | vapor `P/(R·T)`, `R·T/P`; liquid/solid `1/Σ_j x[p,j]·v_j(T)` with `v_j` from `vol_mol_*_comp` or `1/dens_mol_*_comp` |
| `dens_mass_phase[p]` | `dens_mol_phase[p]·mw_phase[p]` |
| `enth_mol_phase[p]`, `enth_mol_phase_comp[p,j]` | vapor `Σ_j x·h_ig,j(T)`; liquid/solid `Σ_j x·h_j(T) + (P − P_ref)/dens_mol_phase[p]` |
| `entr_mol_phase_comp[p,j]` | vapor `s_ig,j(T) − R·ln(x[p,j]·P/P_ref)`; liquid/solid `s_j(T)` |
| `energy_internal_mol_phase_comp[p,j]` | `h_ig(T) − R(T − T_ref) + Δn·R·T_ref` (vapor, with the IDAES Δn rule over noble and diatomic elements); `h(T) + dU_form` (liquid/solid) |
| `gibbs_mol_phase_comp[p,j]` | `h_comp − T·s_comp` |
| `fug_phase_comp[p,j]`, `fug_coeff_phase_comp[p,j]` | vapor `x·P`; liquid Raoult `x·Psat_j(T)`, Henry `henry_pressure(p,j,T)`; coefficient `1` |
| `log_fug_phase_comp_eq[p,j,pair]` | the same evaluated at `_teq[pair]` in log form |
| `pressure_osm_phase[p]` | `R·T·Σ_{j∉solvents} i_j·conc_mol_phase_comp[p,j]` |

The reference `eos.cubic` template carries the Peng–Robinson and Soave–Redlich–Kwong parameterization (`u, w, ΩA, Ωb`), Soave alpha and its two temperature derivatives, the one-fluid mixing rules for `am`, `bm`, `dam/dT`, `d²am/dT²`, `delta[p,i]`, the dimensionless `A[p]`, `B[p]`, and the departure functions for enthalpy, entropy, internal energy, cp, cv, Gibbs energy, partial molar volume, speeds of sound, and the fugacity coefficient kernel `ln φ_j = (b_j/bm)(Z − 1) − ln(Z − B) + (A/(B·p̂))·(b_j/bm − δ_j)·ln((2Z + B(u + p̂))/(2Z + B(u − p̂)))`, all as expressions over the IR. Equilibrium-state duplicates (`_eq` quantities at `_teq`) are the same expressions with `T := _teq[pair]`, generated by template instantiation rather than hand-copied.

The compressibility root is the one non-algebraic step. It is a `KernelSpec`:

```text
kernel cubic.compress_fact@1
  behavior: implicit                       -- z³ + b z² + c z + d = 0 with b, c, d from (A, B, u, w)
  inputs:  A (dimensionless), B (dimensionless), eos_type (enum)
  outputs: z_liq (lowest real root), z_vap (highest real root)
  branch_policy: {liquid: min_real_root, vapor: max_real_root, single_root: that_root}
  derivatives: jacobian (implicit function theorem on the selected root), hessian (available)
  bindings: native, datafusion_udf, nl_external_function ("cubic_root_l", "cubic_root_h"), pyomo_external_function
```

The native evaluation solves the cubic in closed form with Newton polishing and returns the branch and its derivatives; the NL binding ships a compiled external-function library with the same two symbols IDAES uses so that existing solver binaries work unchanged.

### 9.5 Phase equilibrium

Relations and generated mathematics per property package:

1. **Pairs and species.** `phase_equilibrium_pairs` lists `(p1, p2)`; `inferred.phase_equilibrium_species(pair, j)` holds every species present in both phases (this is IDAES's `phase_equilibrium_list` with keys `PE<n>`). The classification `identify_VL_component_list` becomes derived columns `role ∈ {vl, henry, l_only, v_only}`.
2. **Equilibrium temperature.** For each flash-required state instance and pair: symbol `_teq[pair]` (temperature, bounded like `temperature`).
3. **Equilibrium constraint.** For each pair and species with role `vl` or `henry`: `equilibrium_constraint[pair, j]` from the pair's form (`fugacity`: `fug_phase_comp_eq(p1) = fug_phase_comp_eq(p2)`; `log_fugacity`: the log form), family `SMOOTH_TRANSCENDENTAL`, role `HARD_FEASIBILITY`.
4. **SmoothVLE** (Burgard et al.): optional `_t1[pair]` with `_t1 = smooth_max(T, temperature_bubble[pair], eps_1)` when there are no vapor-only species, and `_teq = smooth_min(_t1, temperature_dew[pair], eps_2)` when there are no liquid-only species; otherwise the degenerate forms `_teq = _t1` or `_teq = T`.
5. **CubicComplementarityVLE** (Dabadghao et al.): temperature slacks `s[p] ≥ 0`, `_teq − T − s[vap] + s[liq] = 0`, `smooth_min(s[p]·(mol/s)/K, flow_mol_phase[p], eps_t) = 0`; cubic second-derivative expression `6Z + 2b`, split `gp − gn`, and `smooth_min(gn·f, flow_mol_phase[vap], eps_z) = 0`, `smooth_min(gp·f, flow_mol_phase[liq], eps_z) = 0`. Requires both phases to use the same cubic template (an invariant on `method_selections`).
6. **Bubble and dew points.** Symbols `temperature_bubble[pair]`, `temperature_dew[pair]`, `pressure_bubble[pair]`, `pressure_dew[pair]` and the transition compositions `_mole_frac_{tbub,tdew,pbub,pdew}[pair, j]` (plus log forms) are created only when the property demand closure requests them (initialization plans do). `IdealBubbleDew` generates the Raoult/Henry sums (`Σ_j z_j·Psat_j(Tbub) + Σ_henry z_j·H_j(Tbub) − P = 0`, `P·(Σ z_j/Psat_j(Tdew) + …) − 1 = 0`, and the pressure forms); `LogBubbleDew` generates per-species `log_fug_phase_comp_Tbub(liq, j) = log_fug_phase_comp_Tbub(vap, j)` and `Σ_j _mole_frac_tbub[j] = 1`.
7. **Henry's law.** `HenryType` value encodes direction (`Hcp = 1`, `Hxp = 2` are constants; `Kpc = 51`, `Kpx = 52` are volatilities). The concentration term is selected by type (`conc_mol_phase_comp` for `Hcp`/`Kpc`, `mole_frac_phase_comp` for `Hxp`/`Kpx`, with `_true`/`_apparent` suffix for electrolytes), and `henry_pressure = c/H` or `c·K`. Cubic EOS packages reject Henry species (invariant).

### 9.6 Property demand resolution (pass P6)

Inputs: every equation instantiated by templates (P7), every contribution required by law expansion (P8), every display or port member, every initialization-plan requirement, and the `provides`/`requires` closure of each method. Algorithm:

```text
demand := { (state_scope, property_kind, index) referenced by any equation, port member, display item,
             initialization plan stage, or required by a selected method }
repeat
  for each unresolved (S, k, idx) in demand:
    candidates := method_selections for S's package where family provides k
                  ∪ state template declarations that create k (method: None in IDAES metadata)
    if exactly one candidate → resolve; add its `requires` to demand
    if none                  → status = unresolved (diagnostic prop.unsupported: IDAES PropertyNotSupportedError)
    if several               → apply preference order (package selection > phase-type default > reference default);
                               if still ambiguous → status = ambiguous (diagnostic)
until demand is unchanged
```

Outputs: `inferred.property_requirements` (with `requested_by` chains) and `inferred.method_resolutions`, both with derivations. The IDAES Property Interrogator (`PropertyInterrogatorBlock`) is exactly the query `SELECT property_kind, requesting instance FROM inferred.property_requirements`; no dummy blocks are needed. The IDAES "recursive build" failure (`PropertyPackageError` for a method that fails to create its property) cannot occur because provision is declared, not discovered by attribute access. Inspection never constructs physics (decision D8).

A resolved requirement produces either (a) equations from the method's template, instantiated into the state instance's scope (`enth_mol_phase[p]` as an expression, `log_*` quantities as variable-plus-constraint exactly as IDAES does), or (b) a `kernel_binding` and a `KernelCall` node. The IDAES distinction between properties that are Pyomo `Expression`s and those that are `Var + Constraint` (state variables, `phase_frac`, `flow_mol_phase`, `mole_frac_phase_comp`, `_teq`, bubble/dew symbols, all `log_*` forms, critical properties, `log_k_eq`, VLE slacks) is preserved by the templates because it matters for solver structure; it is recorded in `symbols.role`.

### 9.7 Reaction packages

`GenericReactionParameterBlock` configuration maps to `reactions`, `stoichiometry`, `reaction_methods`, `parameter_values` (including `reaction_order[p,j]` overrides; defaults: rate reactions `order = −ν` for reactants and `0` for products, equilibrium reactions `order = +ν`, zeroed for solid phases), and `reaction_packages` (which also carries `reaction_basis`). The reaction block template declares:

| Symbol / equation | Form |
|---|---|
| `dh_rxn[r]` | expression from `heat_of_reaction` method |
| `k_rxn[r]` | expression from `rate_constant` method |
| `reaction_rate[r]` | expression from `rate_form`: `k_rxn[r]·Π_{(p,j): order≠0} C[p,j]^order[p,j]` with `C` selected by `ConcentrationForm` (`molarity → conc_mol_phase_comp`, `activity → act_phase_comp`, `molality → molality_phase_comp`, `moleFraction → mole_frac_phase_comp`, `massFraction → mass_frac_phase_comp`, `partialPressure → pressure_phase_comp`) |
| `k_eq[r]` / `log_k_eq[r]` | expression / variable+constraint from `equilibrium_constant` (`van_t_hoff`: `log_k_eq − ln k_eq_ref = −(dh_rxn/R)(1/T − 1/T_eq_ref)`; `gibbs_energy`: `log_k_eq = −Δh_ref/(R·T) + Δs_ref/R`) |
| `equilibrium_constraint[r]` | from `equilibrium_form` (`power_law_equil`: `k_eq = Π C^order`; solubility product: `Q − smooth_max(0, Q − s, eps) = 0` with the IDAES `s = s_scale·S/(S + s_norm)` over total solid flow) |
| `reaction_rate_basis` | package `reaction_basis` (must be `molar` or `mass`; `other` blocks conversions) |

The "reaction package must match its property package's unit set and required properties" check (`_validate_property_parameter_units`, `_validate_property_parameter_properties`) becomes two invariants on `reaction_packages`.

### 9.8 Provider kernels: Helmholtz, CoolProp, FeOs

- **Helmholtz.** IDAES computes pure-component Helmholtz properties through a compiled library that evaluates AMPL NL expression files (`phi_ideal`, `phi_residual`, their derivatives, saturation approximations) with a density root solve. The platform authors the same `φ⁰(δ, τ)` and `φʳ(δ, τ)` forms as expression templates (`phi_ideal_type01..04`, `phi_residual_type01..05`, `surface_tension_type01`), loads the per-fluid coefficient JSON into `parameter_values`, and exposes one implicit kernel `helmholtz.state@1` whose unknowns are `(δ, τ)` given the chosen state pair (`PH`, `PS`, `PU`, `TPX`) and whose outputs are the mass-basis properties IDAES exposes (`h`, `s`, `u`, `g`, `f`, `cv`, `cp`, `w`, `v`, plus transport when available). Mole basis is a conversion rule with `mw`. Derivatives come from the implicit function theorem on the templates' own derivatives, so no hand-coded derivative tables are needed. This removes the external-library dependency for the native backend; the NL backend keeps a generated external-function library with the IDAES symbol names.
- **CoolProp.** The `CoolPropWrapper` supplies coefficients for the exponential, non-exponential, and polynomial forms; the platform treats it as a *parameter source* producing `parameter_values` rows (with provenance), never as a runtime dependency of the solve.
- **FeOs.** Conditional provider crate under `KernelSpec` for SAFT and cubic models written over `num-dual`, giving derivatives natively. It is enabled only when `feos-core` tracks `num-dual` 0.15: `feos-core` 0.10.1 requires `num-dual ^0.14`, whose `DualNum` trait has a different shape from 0.15 (`DualNum<f64>` versus `DualNum<Primitive = f64>`), so one kernel source cannot serve both, and the platform writes every kernel against 0.15. FeOs also imports the `quantity` crate, a third units representation, which is never an authority for anything. Coverage (which property kinds, which conventions, which reference states) is declared per kernel and validated by parity tests; nothing is assumed.

### 9.9 Electrolytes and inherent reactions

The electrolyte extensions (true/apparent species bases, `AqueousPhase`, `ENRTL`, ion sets, `apparent_inherent_reaction_extent`, `true_to_appr_species` and `appr_to_true_species` constraints) are supported by the schema (`component_type`, `charge`, `dissociation_species`, `StateIndex`, `_true`/`_apparent` property variants) but are scheduled for phase 4 (§25). Inherent reactions are property-package-owned reactions (`reactions.kind = inherent`); their generation terms are emitted by law expansion exactly like rate and equilibrium reactions, but the extents live in the state scope.

### 9.10 Scaling defaults and validity

`default_scaling` rows (IDAES `default_scaling_factors`) and `method_specs.validity` intervals (IDAES metadata `valid_range`) are consumed by the scaling pass (§16) and by the diagnostics "values outside valid range" check (§15).

---

## 10. Balance laws and control volumes

### 10.1 The conservation law template

IDAES control volumes assemble balances procedurally (`add_material_balances`, `add_energy_balances`, `add_momentum_balances` and their variants in `control_volume0d.py` and `control_volume1d.py`). The platform replaces the procedure with one law template expanded over contributions (decision D7):

```text
law_template conservation@1
  inputs:  scope (control region), subject selector (species j | element e | total | phase p | energy | momentum),
           basis, temporal policy (accumulation | none), balance options
  expansion:
    1. collect contributions whose law_family matches and whose subject matches the selector (semantic match, §8.4)
    2. classify each by orientation: into_scope (+), out_of_scope (−), generation (+), accumulation (−)
    3. convert each to the law's basis (insert conversion kernels, never sum incompatible quantities)
    4. group by the balance index (t; t,x; and p,j | j | e | none depending on balance type)
    5. emit one equation per index: Σ signed contributions = 0, family NETWORK_BALANCE or AFFINE_EQUALITY when affine,
       role HARD_FEASIBILITY, with derivation listing every contribution
    6. emit negative-completeness rows for candidate contributions that were excluded and why
```

Material, element, energy, and momentum balances are bindings of this template with different subjects and contribution kinds. The control-volume templates declare the contributions; the compiler does the bookkeeping.

### 10.2 Contribution kinds

The table lists every contribution IDAES control volumes can produce, the condition under which the template declares it, and its orientation. `F(p,j)` is the state's `material_flow_term`, `H(p)` its `enthalpy_flow_term`.

| Law family | Contribution (IDAES symbol) | Declared when | Orientation | Notes |
|---|---|---|---|---|
| material | inlet flow `F_in(p,j)` | always (0D: from `properties_in`; 1D: `−fdt·d(_flow_terms)/dξ`) | into_scope | 1D uses the derivative form (§10.4) |
| material | outlet flow `F_out(p,j)` | always | out_of_scope | |
| material | `rate_reaction_generation[t,(p,j)] = Σ_r ν[r,p,j]·rate_reaction_extent[t,r]` | `has_rate_reactions` | generation | converted by `_rxn_rate_conv`: mass↔molar via `mw_comp[j]`; `other` basis is an error |
| material | `equilibrium_reaction_generation` | `has_equilibrium_reactions` (requires `reactions.has_equilibrium`) | generation | |
| material | `inherent_reaction_generation` | property package has inherent reactions and state includes them | generation | |
| material | phase-equilibrium transfer `Σ_r sd[r]·phase_equilibrium_generation[t,r]` with `sd = +1` if `PE_r = (j,(p,*))`, `−1` if `(j,(*,p))` | `has_phase_equilibrium` and balance type `componentPhase` | generation (antisymmetric across phases) | |
| material | `mass_transfer_term[t,(p,j)]` | `has_mass_transfer` | generation | |
| material | custom molar / mass terms | authored | generation | basis conversion by `mw_comp[j]` |
| material | accumulation `d(material_holdup)/dt` with `material_holdup = volume·phase_fraction[p]·material_density_term(p,j)` | `dynamic` (holdup symbol when `has_holdup`) | accumulation | |
| element | elemental flows `Σ_j conv(j)·F(p,j)·element_comp[j][e]` in and out | balance type `elementTotal` | into/out | `conv = 1` (molar) or `1/mw_comp[j]` (mass) |
| element | `elemental_mass_transfer_term[t,e]` | `has_mass_transfer` | generation | reactions are forbidden with element balances (invariant) |
| element | accumulation `d(element_holdup)/dt` | dynamic | accumulation | |
| energy | `H_in(p)`, `H_out(p)` | always | into / out | |
| energy | `heat[t]` | `has_heat_transfer` | into_scope (sign is authored: positive into the volume) | |
| energy | `work[t]` | `has_work_transfer` | into_scope | |
| energy | `enthalpy_transfer[t]` | `has_enthalpy_transfer` | into_scope | |
| energy | `heat_of_reaction[t] = −Σ_r extent[t,r]·dh_rxn[r]` (rate and equilibrium) | `has_heat_of_reaction` (requires extents to exist) | generation | |
| energy | custom term | authored | generation | |
| energy | accumulation `Σ_p d(energy_holdup[p])/dt`, `energy_holdup = volume·phase_fraction·energy_density_term(p)` | dynamic | accumulation | |
| momentum | `P_in`, `P_out` | always (`pressureTotal`) | into / out | |
| momentum | `deltaP[t]` | `has_pressure_change` | generation | |
| momentum | custom term | authored | generation | |

Balance type selects the grouping and which contributions participate:

| `MaterialBalanceType` | Balance index | Participation |
|---|---|---|
| `componentPhase` | `(t, p, j)` over the phase-species set | all material contributions including phase-equilibrium transfer |
| `componentTotal` | `(t, j)` summing over phases with `(p, j)` valid | phase-equilibrium transfer cancels and is omitted |
| `elementTotal` | `(t, e)` over elements minus linearly dependent ones | elemental contributions only; `element_balances` index excludes elements made dependent by the IDAES rule (an element unique to one species, when that species already contributed one unique element) |
| `total` | `(t)` | not supported by IDAES control volumes (`BalanceTypeNotSupportedError`); supported by mixers/separators and by `add_state_material_balances` |
| `useDefault` | resolved from the representative state's `default_material_balance_type` | inference rule |

`EnergyBalanceType.enthalpyTotal` and `isothermal` (extended control volumes: `T_in = T_out`, or `T[t, prev(x)] = T[t, x]` in 1D) and `MomentumBalanceType.pressureTotal` are the implemented bindings; `enthalpyPhase`, `energyTotal`, `energyPhase`, `pressurePhase`, `momentumTotal`, `momentumPhase` are declared enum members whose law bindings are absent, so selecting them raises `law.unsupported_binding` at P8, exactly where IDAES raises `BalanceTypeNotSupportedError`.

### 10.3 The lumped control volume template (`cv.lumped@1`, IDAES `ControlVolume0DBlock`)

```yaml
template: cv.lumped@1
  kind: control_volume
  params:
    property_package: {type: property_package, required: true}
    reaction_package: {type: reaction_package, default: null}
    property_package_args, reaction_package_args: {type: options}
  features:
    dynamic: {kind: bool, inherit_from: parent.dynamic}
    has_holdup: {kind: bool, default: "= dynamic"}      # rule: dynamic ⇒ has_holdup
    has_phase_equilibrium: {kind: bool, required_explicit: true}
    information_flow: {kind: enum, enum: FlowDirection, default: forward}
    has_rate_reactions, has_equilibrium_reactions, has_mass_transfer, has_heat_transfer,
    has_work_transfer, has_enthalpy_transfer, has_heat_of_reaction, has_pressure_change: {kind: bool, default: false}
    material_balance_type, energy_balance_type, momentum_balance_type: {kind: enum}
  domains:
    time: {from: flowsheet.time}
    phase, species, phase_species: {from: property_package}
  submodels:
    properties_in:  {template: "= property_package.state_block", multiplicity: time,
                     bindings: {defined_state: "= information_flow == forward", has_phase_equilibrium: "= has_phase_equilibrium"}}
    properties_out: {template: "= property_package.state_block", multiplicity: time,
                     bindings: {defined_state: "= information_flow == backward", has_phase_equilibrium: "= has_phase_equilibrium"}}
    reactions:      {template: "= reaction_package.reaction_block", multiplicity: time, guard: "reaction_package != null",
                     bindings: {state: properties_out, has_equilibrium: "= has_equilibrium_reactions"}}
  symbols:
    volume[t]                       : {quantity: volume, guard: has_holdup, default_initial: 1}
    phase_fraction[t,p]             : {quantity: phase_fraction, role: "= n_phases > 1 ? variable : expression(1)", default_initial: 1/n_phases}
    material_holdup[t,p,j]          : {quantity: amount (or mass by basis), guard: has_holdup}
    material_accumulation[t,p,j]    : {role: derivative, of: material_holdup, wrt: time, guard: dynamic}
    rate_reaction_extent[t,r]       : {quantity: molar_flow (reaction basis), guard: has_rate_reactions}
    rate_reaction_generation[t,p,j] : {guard: has_rate_reactions}
    equilibrium_reaction_extent[t,r], equilibrium_reaction_generation[t,p,j]: {guard: has_equilibrium_reactions}
    inherent_reaction_extent[t,r], inherent_reaction_generation[t,p,j]:       {guard: "properties_out.include_inherent_reactions"}
    phase_equilibrium_generation[t,pe]: {guard: "has_phase_equilibrium and material_balance_type == componentPhase"}
    mass_transfer_term[t,p,j]       : {guard: has_mass_transfer}
    energy_holdup[t,p], energy_accumulation[t,p]: {guard: has_holdup / dynamic}
    heat[t], work[t], enthalpy_transfer[t]: {quantity: power, guards: has_heat_transfer / has_work_transfer / has_enthalpy_transfer}
    heat_of_reaction[t]             : {role: expression, guard: has_heat_of_reaction}
    deltaP[t]                       : {quantity: pressure_difference, guard: has_pressure_change}
    element_holdup[t,e], element_accumulation[t,e], elemental_flow_in[t,p,e], elemental_flow_out[t,p,e], elemental_mass_transfer_term[t,e]:
                                      {guard: material_balance_type == elementTotal (and holdup/dynamic/mass-transfer as applicable)}
  equations:
    sum_of_phase_fractions[t]:            "1 == sum(p in phase | phase_fraction[t,p])"            guard: n_phases > 1 and has_holdup
    material_holdup_calculation[t,p,j]:   "material_holdup[t,p,j] == volume[t]*phase_fraction[t,p]*properties_out[t].material_density_term[p,j]"
    rate_reaction_stoichiometry[t,p,j]:   "rate_reaction_generation[t,p,j] == sum(r in rate_reactions | stoich[r,p,j]*rate_reaction_extent[t,r])"
    equilibrium_reaction_stoichiometry, inherent_reaction_stoichiometry: analogous
    energy_holdup_calculation[t,p]:       "energy_holdup[t,p] == volume[t]*phase_fraction[t,p]*properties_out[t].energy_density_term[p]"
    elemental_holdup_calculation[t,e]:    "element_holdup[t,e] == volume[t]*sum((p,j) in phase_species | conv(j)*phase_fraction[t,p]*material_density_term[p,j]*element_comp[j,e])"
  contributions: (as in §10.2, each guarded by its feature)
  law_instances:
    material: {law: conservation, subject: "= material_balance_type", scope: self, options: {basis: properties_out.material_flow_basis}}
    energy:   {law: conservation, subject: energy,   guard: energy_balance_type != none}
    momentum: {law: conservation, subject: momentum, guard: momentum_balance_type != none}
  requirements: [material_flow_terms, enthalpy_flow_terms, material_density_terms (if has_holdup), energy_density_terms (if has_holdup)]
```

The generated equations reproduce IDAES's forms exactly, for example the `componentPhase` material balance:

```text
accum[t,p,j] == F_in[t,p,j] − F_out[t,p,j] + rate_gen[t,p,j]·rxn_conv(t,j) + equil_gen[t,p,j] + inherent_gen[t,p,j]
                + phase_equilibrium_term(t,p,j) + mass_transfer_term[t,p,j] + user_mol(t,p,j) + user_mass(t,p,j)
```

with `accum = convert(material_accumulation → flow units)` when dynamic and `0` otherwise; the total enthalpy balance `Σ_p accum_E[t,p] == Σ_p H_in[t,p] − Σ_p H_out[t,p] + heat + work + enthalpy_transfer + heat_of_reaction + custom`; and the pressure balance `0 == P_in − P_out + deltaP + custom`.

Units follow the IDAES derivations (`flow_units` from the flow basis, `acc_units = amount/time` or `mass/time`, `holdup_units`, `rxn_flow_units` from the reaction basis); a mismatch between flowsheet time units and package time units is invariant `cv.time_units_consistent`.

### 10.4 The distributed control volume template (`cv.distributed_1d@1`, IDAES `ControlVolume1DBlock`)

Additional parameters and features: `area_definition` (`DistributedVars.uniform` ⇒ scalar `area`; `variant` ⇒ `area[t,x]`), `transformation_method` (`dae.finite_difference` | `dae.collocation`), `transformation_scheme` (`BACKWARD`, `FORWARD` for finite difference; `LAGRANGE_RADAU`, `LAGRANGE_LEGENDRE` for collocation), `finite_elements`, `collocation_points`, `flow_direction`, `length_domain_set`, optional external `length_var`.

Structural facts preserved from IDAES:

- `length_domain` is a **normalized, dimensionless continuous domain on [0, 1]**; the physical length is the scalar symbol `length` (or an external reference). The mathematics is written per unit ξ, so every per-unit-length source term is multiplied by `length`.
- `_flow_direction_term = −1` for `forward`, `+1` for `backward`; ports bind to `properties[:, first]` / `properties[:, last]` accordingly (§12.1).
- One indexed state block `properties[t, x]`; `defined_state = true` only at the inlet end (`first` for forward, `last` for backward), which is an `idx_map` in IDAES and a per-member binding rule here.
- Linking symbols `_flow_terms[t,x,p,j] = properties[t,x].material_flow_term[p,j]` and `_enthalpy_flow[t,x,p] = properties[t,x].enthalpy_flow_term[p]` exist so that the spatial derivatives `material_flow_dx`, `enthalpy_flow_dx`, `elemental_flow_dx`, `pressure_dx` are derivatives of symbols, not of expressions.
- Per-unit-length quantities: `material_holdup` (amount/length), `rate_reaction_generation` (flow/length), `heat`, `work`, `enthalpy_transfer` (power/length), `deltaP` (pressure/length).
- **Boundary skip rule.** Balance and stoichiometry equations are skipped at `x = first` unless the scheme is `FORWARD`, in which case they are skipped at `x = last`. This is a filter on the equation template index (`filter: "x != boundary(scheme)"`), evaluated by P12 (index expansion) against the mesh that P11 produced.

Generated equations (componentPhase form):

```text
material_flow_linking[t,x,p,j]:   _flow_terms[t,x,p,j] == properties[t,x].material_flow_term[p,j]
material_balances[t,x,p,j]:       L·convert(material_accumulation[t,x,p,j]) == fdt·material_flow_dx[t,x,p,j]
                                     + L·rate_gen·rxn_conv + L·equil_gen + L·inherent_gen + L·pe_term + L·mass_transfer + L·custom
material_holdup_calculation:      material_holdup[t,x,p,j] == area(t,x)·phase_fraction[t,x,p]·material_density_term[p,j]
enthalpy_balances[t,x]:           L·convert(Σ_p energy_accumulation[t,x,p]) == fdt·Σ_p enthalpy_flow_dx[t,x,p] + L·(heat + work + enthalpy_transfer + heat_of_reaction + custom)
pressure_balance[t,x]:            0 == fdt·pressure_dx[t,x] + L·deltaP[t,x] + L·custom
element_balances[t,x,e]:          L·convert(element_accumulation) == fdt·elemental_flow_dx[t,x,e] + L·elemental_mass_transfer + L·custom
```

The IDAES `apply_transformation()` step is the discretization policy attached to `length_domain` (§13.4); the transformation must be applied to a domain owned by the control volume (external domains are rejected, invariant `cv.length_domain_owned`).

### 10.5 What the template model changes structurally

The unit-model survey identified five mechanisms IDAES uses that a declarative template must express without code:

| IDAES mechanism | Template mechanism |
|---|---|
| Config fragments re-used textually (`_make_heater_config_block` invoked for `hot_side` and `cold_side`) | Named parameter groups (`param_group: heater_side`) included by reference; `hx_process_config` name aliasing becomes `template_ports.alias` rows |
| Narrowing an inherited option (`Turbine` forces `compressor = false`, `thermodynamic_assumption = isentropic`) | `template_feature_rules` with `rule = requires` on a derived template; the derived template's param domain is the narrowed set |
| Conditional component declaration (`heat_duty` exists only if `has_heat_transfer and energy_balance_type != none`) | `guard_id` on symbols, equations, ports, submodels |
| Callback extension points (`delta_temperature_callback`, `valve_function_callback`, `pressure_flow_callback`, performance-curve `build_callback`) | Helper templates selected by an enum feature (`delta_temperature_method ∈ {lmtd, lmtd2, lmtd3, lmtd_smooth, amtd, underwood}`); a custom callback is an authored helper template |
| Sub-model embedding with re-exported ports (`Flash ⊃ Separator`, `SLSeparator ⊃ Separator`) | `template_submodels` plus `template_ports.bound_to = "split.Vap"` |

The recurring IDAES initialization idiom (fix a transfer term to a guess, deactivate its defining equation, solve, then release) is a two-stage `initialization_plan` (§17).

---

## 11. The unit model library as templates

### 11.1 Catalog

Every IDAES generic unit becomes a reference template. The table records composition, the features that switch structure, extra symbols and equations beyond the control volume, ports, and the default plans. Equations are summarized; full forms are in the survey catalogs and are reproduced in the reference package sources.

| Template (IDAES class) | Composition | Structural features / params | Extra symbols and equations | Ports | Default plans |
|---|---|---|---|---|---|
| `unit.heater@1` (`Heater`) | one `cv.lumped` with `has_heat_transfer = true` | `material/energy/momentum_balance_type`, `has_phase_equilibrium`, `has_pressure_change` | references `heat_duty[t] ≡ cv.heat[t]`, `deltaP[t] ≡ cv.deltaP[t]` (guarded) | `inlet`, `outlet` | init `single_cv`; scaler `cv` |
| `unit.feed@1`, `unit.product@1`, `unit.state_junction@1` | one state block `properties[t]` (`defined_state = true`, no phase equilibrium) | `property_package` | references to every state symbol; StateJunction binds `inlet` and `outlet` to the same state | `outlet` / `inlet` / both | init: the state block's plan |
| `unit.feed_flash@1` (`FeedFlash`) | `cv.lumped` with `has_phase_equilibrium = true`, no energy balance | `flash_type ∈ {isothermal, isenthalpic}` | `isothermal[t]: T_in = T_out` or `isenthalpic[t]: Σ_p H_in = Σ_p H_out`; references to inlet state symbols | `outlet` only | fix inlet state |
| `unit.translator@1` (`Translator`) | `properties_in` (`defined_state = true`), `properties_out` (`defined_state = outlet_state_defined`, `has_phase_equilibrium`) | `inlet_property_package`, `outlet_property_package`, `outlet_state_defined`, `has_phase_equilibrium` (mutually exclusive with defined outlet) | **none**; the author adds mapping equations; the compiler refuses to equate quantities with different reference states without them | `inlet`, `outlet` | init `block_triangularization`; solve only when DOF = 0 |
| `unit.cstr@1` (`CSTR`) | `cv.lumped` with geometry, reactions, `has_rate_reactions = true` | reaction and balance features, `has_heat_of_reaction`, `has_heat_transfer`, `has_pressure_change` | `cstr_performance_eqn[t,r]: cv.rate_reaction_extent[t,r] = volume[t]·cv.reactions[t].reaction_rate[r]` | `inlet`, `outlet` | scaler defaults `deltaP: 1e-5`, `volume: 1e3` |
| `unit.pfr@1` (`PFR`), `unit.pipe@1` (`Pipe`) | `cv.distributed_1d` (PFR with reactions) | discretization params, `length_domain_set` | `performance_eqn[t,x,r]: extent[t,x,r] = reactions[t,x].reaction_rate[r]·area`; `volume` with `geometry: volume = area·length` | `inlet`, `outlet` | scaler requires user input for `length`, `area` |
| `unit.stoichiometric_reactor@1` | `cv.lumped` with reactions, `has_rate_reactions = true` | `has_heat_of_reaction`, `has_heat_transfer`, `has_pressure_change` | reference `rate_reaction_extent[t,r]` (user-fixed; no performance equation) | `inlet`, `outlet` | |
| `unit.equilibrium_reactor@1` | `cv.lumped` with reactions | `has_rate_reactions` (default true), `has_equilibrium_reactions` (default true) | `rate_reaction_constraint[t,r]: reactions[t].reaction_rate[r] = 0` | `inlet`, `outlet` | |
| `unit.gibbs_reactor@1` | `cv.lumped` with element balances (`elementTotal` hard-coded) | `inert_species`, `has_heat_transfer`, `has_pressure_change` | domain `lagrange_set` (elements touched by non-inert species); `lagrange_mult[t,e]` (energy/mol); `gibbs_minimization[t,p,j]: 0 = gibbs_scaling·(gibbs_mol_phase_comp[p,j] + Σ_e lagrange_mult[t,e]·element_comp[j,e])` for non-inert `j`; `inert_species_balance[t,p,j]: F_in = F_out` (skipped when linearly dependent on an element balance) | `inlet`, `outlet` | scaler: `lagrange_mult` by `1/(R·T_nominal)` |
| `unit.flash@1` (`Flash`) | `cv.lumped` (`has_phase_equilibrium = true`, heat and pressure change default true) ⊃ `unit.separator` (`outlet_list = [Vap, Liq]`, `split_basis = phaseFlow`, `ideal_separation`, `ideal_split_map` by phase type, `mixed_state_block = cv.properties_out`) | `ideal_separation`, `energy_split_basis` | `split_fraction_eq[t,o]: split.split_fraction[t,o,o] = 1` when not ideal | `inlet`; `vap_outlet`, `liq_outlet` re-exported from the separator | |
| `unit.mixer@1` (`Mixer`) | no control volume; `inlet_state[i][t]` per inlet (`defined_state = true`), `mixed_state[t]` (or an external state) | `inlet_list`/`num_inlets` (domain `inlet_idx`), `material_balance_type` (`elementTotal` invalid), `has_phase_equilibrium`, `energy_mixing_type ∈ {none, extensive}`, `momentum_mixing_type ∈ {none, minimize, equality, minimize_and_equality}`, `construct_ports` | law instances: material mixing per balance type (`0 = Σ_i F_in,i − F_mixed + PE terms + inherent gen`), `enthalpy_mixing[t]: 0 = Σ_i Σ_p H_i − Σ_p H_mixed`; `minimum_pressure[t,i]` with `minimum_pressure_constraint` chain `mp[t,i] = smooth_min(mp[t,i−1], P_i, eps_pressure)` and `mixture_pressure: P_mixed = mp[t,last]`; `pressure_equality[t,i]: P_mixed = P_i`; `minimize_and_equality` builds both with equality inactive by default | one port per inlet name, `outlet` | init `mixer` (§17.3); scaler `mixer` |
| `unit.separator@1` (`Separator`) | no control volume; `mixed_state[t]` (`defined_state = true`) plus per-outlet states unless ideal | `outlet_list`/`num_outlets`, `split_basis ∈ {totalFlow, phaseFlow, componentFlow, phaseComponentFlow}`, `material_balance_type`, `momentum_balance_type ∈ {none, pressureTotal}`, `energy_split_basis ∈ {none, equal_temperature, equal_molar_enthalpy, enthalpy_split}`, `ideal_separation`, `ideal_split_map`, `has_phase_equilibrium` | `split_fraction[t,o,(index by basis)]` with `sum_split_frac`; `material_splitting_eqn` per balance type (`sf·F_mixed = F_out + PE/inherent terms`); `temperature_equality_eqn`, `molar_enthalpy_equality_eqn`, or `molar_enthalpy_splitting_eqn` (forbidden with component-based bases); `pressure_equality_eqn`. Ideal path: no outlet states; port members are expressions (`mixed value` or `eps_flow`/`eps_frac` by `ideal_split_map`) | `inlet`; one port per outlet | init `separator`; scaler `separator` |
| `unit.pressure_changer@1` and derived `unit.compressor@1`, `unit.pump@1`, `unit.turbine@1` | `cv.lumped` with `has_work_transfer = true`, `has_pressure_change = true` | `compressor`, `thermodynamic_assumption ∈ {isothermal, isentropic, pump, adiabatic}`, `support_isentropic_performance_curves` | `work_mechanical ≡ cv.work`, `deltaP ≡ cv.deltaP`, `ratioP[t]` with `ratioP·P_in = P_out`; isothermal: `T_in = T_out`; adiabatic: `work = 0`; pump: `work_fluid = (P_out − P_in)·flow_vol`, `efficiency_pump`, `actual_work` (`work_fluid = work_mech·η` for compressors, reversed for expanders); isentropic: `properties_isentropic[t]` state, `isentropic_pressure`, `state_material_balances`, `isentropic: s_isen = s_in`, `isentropic_energy_balance: work_isentropic = Σ_p H_isen − Σ_p H_in`, `actual_work` with `efficiency_isentropic`, optional performance-curve helper with `head_isentropic`, `head` expressions | `inlet`, `outlet` | init `isentropic_pressure_changer` for isentropic; scaler defaults `ratioP: 1`, efficiencies `10` |
| `unit.valve@1` (`Valve`) | derives from `pressure_changer` with `compressor = false`, `adiabatic`, `material_balance_type = componentTotal` | `valve_function ∈ {linear, quick_opening, equal_percentage, custom}`, `pressure_flow_method` | `valve_opening[t] ∈ [0,1]` (fixed by default), `valve_function[t]` (`opening`, `sqrt(opening)`, `alpha^(opening − 1)`), `Cv`, `pressure_flow_equation[t]: F² = Cv²·(P_in − P_out)·valve_function²` | `inlet`, `outlet` | init guesses outlet pressure (`0.95·P_in` fallback) |
| `unit.heat_exchanger@1` (`HeatExchanger` 0D) | two `cv.lumped` (`hot_side`, `cold_side`, each with `has_heat_transfer = true`) | side param groups, `hot_side_name`/`cold_side_name` aliases, `flow_pattern ∈ {countercurrent, cocurrent, crossflow}`, `delta_temperature_method` | `overall_heat_transfer_coefficient[t]`, `area`, `delta_temperature_in/out[t]` with the pattern-dependent definitions, `crossflow_factor[t]`, `heat_duty ≡ cold_side.heat`, `unit_heat_balance: 0 = hot.heat + convert(cold.heat)`, `delta_temperature[t]` from the helper (`lmtd`: `(dT1 − dT2)/ln(dT1/dT2)`; `lmtd2`, `lmtd3` variants; `lmtd_smooth` (Kazi et al.) with `eps_lmtd_smoothing`; `amtd`; `underwood` `((cbrt(dT1) + cbrt(dT2))/2)³`), `heat_transfer_equation: convert(heat_duty) = [crossflow_factor·] U·A·ΔT` | `hot_side_inlet`, `hot_side_outlet`, `cold_side_inlet`, `cold_side_outlet` plus aliases | init `hx_0d`; scaler `hx` |
| `unit.heat_exchanger_ntu@1`, `unit.heat_exchanger_lumped_capacitance@1` | two `cv.lumped` | as IDAES (effectiveness–NTU relations; wall temperature and heat holdup with `ua_hot_side`, `ua_cold_side`) | per IDAES source | four ports | |
| `unit.heat_exchanger_1d@1` (`HeatExchanger1D`), `unit.shell_and_tube_1d@1` | two `cv.distributed_1d` with flow directions from `flow_type` (cocurrent: both forward; countercurrent: cold backward) | per-side param groups; common `finite_elements`, `collocation_points`, `flow_type`; shell-and-tube adds `shell_is_hot` and geometry | `area`, `length ≡ hot_side.length`, `length_equality`, `heat_transfer_coefficient[t,x]`, `heat_transfer_eq[t,x]: hot.heat = −U·area/length·(T_hot − T_cold)`, `heat_conservation[t,x]: convert(cold.heat) = −hot.heat`; shell-and-tube: `shell_diameter`, `tube_outer/inner_diameter`, `number_of_tubes`, cross-section constraints, `temperature_wall[t,x]`, hot/cold heat-transfer equations through the wall | four ports plus `Shell_*`/`Tube_*` aliases | init `hx_1d` (duty guess then release) / `shell_and_tube` (wall temperature fixed then released) |
| `unit.ms_contactor@1` (`MSContactor`) | no control volume; per stream: `stream[t,e]` over `elements = 1..n`, optional `inlet_state[t]`, side-stream states, reaction blocks; optional heterogeneous reaction blocks | `streams` map (per stream: property/reaction packages, `flow_direction`, `has_feed`, `has_rate_reactions`, `has_equilibrium_reactions`, `has_energy_balance`, `has_heat_transfer`, `has_heat_of_reaction`, `has_pressure_balance`, `has_pressure_change`, `side_streams`), `number_of_finite_elements`, `interacting_streams`, `heterogeneous_reactions` | `material_transfer_term[t,e,(s1,s2,j)]` (positive into `s1`), `energy_transfer_term[t,e,(s1,s2)]`, per-stream holdups, extents, generation terms, stage-wise material/energy/pressure balances with side-stream feeds and the upstream-element rule by flow direction, `volume[e]`, `volume_frac_stream[t,e,s]` | `<stream>_inlet`, `<stream>_outlet` | init `ms_contactor` (transfer terms fixed, per-stream SCC solve, holdup back-calculation) |
| `unit.skeleton@1` (`SkeletonUnitModel`) | nothing | authored symbols/equations; `add_ports(name, member map)` | user-declared | user-declared | init: solve when DOF = 0 |
| `unit.stream_scaler@1` | one state block; expression port members | — | `multiplier`; outlet members `multiplier·var` for flow-like members, pass-through otherwise (expression members, IDAES `VarLikeExpression`) | `inlet`, `outlet` | |
| `unit.sl_separator@1`, `unit.thickener_0d@1` | separate solid and liquid states; embedded separators | solid and liquid packages, `energy_split_basis` | `liquid_recovery ≡ split.split_fraction[t, recovered]`; thickener adds the Kynch flux, Stokes law, continuity and conservation equations with `Conditional` flux functions | five or six ports | |
| `control.pid@1` (`PIDController`) | references to `process_var[t]` and `manipulated_var[t]` | `controller_type ∈ {P, PI, PD, PID}`, `mv_bound_type ∈ {NONE, SMOOTH_BOUND, LOGISTIC}`, `antiwindup_type`, `derivative_on_error`, `calculate_initial_integral` | `setpoint`, `gain_p/i/d/b`, `mv_ref`, `error` (variable or expression), `negative_pv`, derivative symbols, `mv_integral_component` and its derivative, `mv_unbounded` expression, `mv_eqn` (smooth bound or logistic), `mv_integration_eqn` (with conditional-integration Heaviside or back-calculation), `initial_integral_error_eqn`; equations at `t0` deactivated by default | none | scaler as IDAES |

Costing methods (`FlowsheetCostingBlock`, `UnitModelCostingBlock`, SSLW) are templates of kind `costing_method` (§19.5).

### 11.2 Worked template: the heater

```yaml
template: unit.heater@1
  kind: unit
  idaes_class: idaes.models.unit_models.Heater
  params:
    include: [param_group.unit_base, param_group.heater_side]   # dynamic, has_holdup; balance types, has_phase_equilibrium, has_pressure_change, property_package(+args)
  submodels:
    control_volume:
      template: cv.lumped@1
      bindings:
        dynamic: "= dynamic"; has_holdup: "= has_holdup"
        property_package: "= property_package"; property_package_args: "= property_package_args"
        has_phase_equilibrium: "= has_phase_equilibrium"
        material_balance_type: "= material_balance_type"; energy_balance_type: "= energy_balance_type"; momentum_balance_type: "= momentum_balance_type"
        has_heat_transfer: true; has_pressure_change: "= has_pressure_change"
        geometry: "= has_holdup"
  symbols:
    heat_duty[t]: {role: reference, reference_to: control_volume.heat[t], quantity: power}
    deltaP[t]:    {role: reference, reference_to: control_volume.deltaP[t], guard: "has_pressure_change and momentum_balance_type != none"}
  ports:
    inlet:  {kind: material, direction: inlet,  bound_to: control_volume.properties_in}
    outlet: {kind: material, direction: outlet, bound_to: control_volume.properties_out}
  display:
    performance: [{label: "Heat Duty", expr: heat_duty[t]}, {label: "Pressure Change", expr: deltaP[t], guard: has_pressure_change}]
  plans:
    initializer: init.single_control_volume@1
    scaler: scale.heater@1
```

Nothing in this template computes; the compiler instantiates the control volume, resolves the property package's state template into `properties_in[t]` and `properties_out[t]`, expands the three law instances, derives port members from the state template, and emits the equations of §10.3 with derivations back to this file.

### 11.3 Worked template fragment: isentropic pressure change

```yaml
template: unit.pressure_changer@1
  features:
    compressor: {kind: bool, default: true}
    thermodynamic_assumption: {kind: enum, enum: ThermodynamicAssumption, default: isothermal}
    support_isentropic_performance_curves: {kind: bool, default: false}
  submodels:
    control_volume: {template: cv.lumped@1, bindings: {has_work_transfer: true, has_pressure_change: true, ...}}
    properties_isentropic: {template: "= property_package.state_block", multiplicity: time, guard: "thermodynamic_assumption == isentropic",
                            bindings: {defined_state: false, has_phase_equilibrium: "= has_phase_equilibrium"}}
    performance_curve: {template: helper.isentropic_performance_curve@1, guard: support_isentropic_performance_curves}
  symbols:
    work_mechanical[t]: {role: reference, reference_to: control_volume.work[t]}
    ratioP[t]: {quantity: dimensionless, default_initial: 1}
    efficiency_isentropic[t]: {quantity: dimensionless, default_initial: 0.8, guard: "thermodynamic_assumption == isentropic"}
    work_isentropic[t]: {quantity: power, guard: "thermodynamic_assumption == isentropic"}
  equations:
    ratioP_calculation[t]: "ratioP[t] * control_volume.properties_in[t].pressure == control_volume.properties_out[t].pressure"
    isothermal[t]:        {expr: "properties_in[t].temperature == properties_out[t].temperature", guard: "thermodynamic_assumption == isothermal"}
    zero_work_equation[t]:{expr: "control_volume.work[t] == 0", guard: "thermodynamic_assumption == adiabatic"}
    isentropic_pressure[t]: {expr: "properties_isentropic[t].pressure == control_volume.properties_out[t].pressure", guard: isentropic}
    isentropic[t]:          {expr: "properties_isentropic[t].entr_mol == control_volume.properties_in[t].entr_mol", guard: isentropic}
    isentropic_energy_balance[t]:
      {expr: "work_isentropic[t] == sum(p in phase | properties_isentropic[t].enthalpy_flow_term[p]) - sum(p in phase | control_volume.properties_in[t].enthalpy_flow_term[p])", guard: isentropic}
    actual_work[t]:
      {expr: "if compressor then work_isentropic[t] == work_mechanical[t]*efficiency_isentropic[t] else work_mechanical[t] == work_isentropic[t]*efficiency_isentropic[t]", guard: isentropic}
  law_instances:
    isentropic_state_material: {law: state_material_balance, subject: "= material_balance_type", states: [properties_isentropic, control_volume.properties_out], guard: isentropic}
```

`if compressor then … else …` is a compile-time conditional on a feature, so P7 selects one branch and no `Conditional` node reaches the backend.

### 11.4 Derived templates

```yaml
template: unit.turbine@1
  derives_from: unit.pressure_changer@1
  feature_rules:
    - {rule: requires, antecedent: "true", consequent: "compressor == false"}
    - {rule: requires, antecedent: "true", consequent: "thermodynamic_assumption == isentropic"}
  plans: {initializer: init.isentropic_pressure_changer@1}
```

A derived template narrows features and parameters; it cannot add code. The three IDAES wrappers (`Compressor`, `Pump`, `Turbine`) and the valve are four such rows.

---

## 12. Connectivity

### 12.1 Ports

A port is a typed interface derived from a state instance, never a hand-maintained list of variables.

- **Members** come from the bound state template's `define_port_members` declaration (default: the state symbols). For an IDAES-compatible FTPx state a material port has members `flow_mol`, `mole_frac_comp[j]`, `temperature`, `pressure`; each member row records `quantity_type_id` and the symbol group.
- **Kinds.** `material` (state-defined members), `heat` (a single power quantity), `work`, `signal` (a dimensionless or typed scalar). Heat and signal ports are how heat integration and controllers connect without inventing pseudo-streams.
- **Direction** is declared (`inlet`, `outlet`, `bidirectional`) and used only for topology inference and stream tables; equations never depend on it.
- **Distributed control volumes** bind ports to `properties[:, first]` for the inlet and `properties[:, last]` for the outlet when `flow_direction = forward`, reversed for `backward`. The binding is a `Gather` with a fixed coordinate on the length domain.
- **Expression members** (IDAES `VarLikeExpression`, used by the ideal separator and the stream scaler) are legal: a member may reference an expression symbol; the connection rule then produces an equation between an expression and a variable, which is still an affine equality when the expression is affine.

### 12.2 Connection rules

`authored.connections` records an arc; `rule_template_id` selects the physics of the junction. The reference package ships:

| Rule template | Generated equations | Use |
|---|---|---|
| `connection.equality@1` (default; Pyomo `Port.Equality` / IDAES `expand_arcs`) | for every member `m` and index `i`: `from.m[i] == to.m[i]`; family `AFFINE_EQUALITY`, role `LINKING` | one-to-one connections between identical state templates |
| `connection.translate@1` | none by itself; requires the destination to be a `unit.translator` instance whose authored mapping equations link `properties_in` to `properties_out` | different property packages |
| `connection.heat@1` | `from.heat[t] == to.heat[t]` with sign convention `into the destination is positive` | heat-integration graphs, wall models |
| `connection.signal@1` | `from.value[t] == to.value[t]` | controllers |

A connection whose ports have mismatched member sets or incompatible quantity types (different reference states, bases, or shapes) is invariant `conn.member_mismatch`; IDAES surfaces this only at arc expansion as a Pyomo error. Pyomo's `Port.Extensive` rule (automatic splitting and mixing at multi-arc ports) is intentionally not provided: mixers and separators are templates with explicit physics (proposal §5.3).

### 12.3 Arc expansion is a pass, not a transformation call

P8 generates `inferred.connection_equations` (one row per generated equation with `connection_id`, `member_ordinal`, `index`, `equation_id`) and the corresponding `compiled.math_indexed_equations` rows, which P12 expands to scalar `math_equations`. There is no separate "expand arcs" step to forget: a snapshot with connections always compiles to a problem that contains them. Deactivating a connection for an initialization stage is a `case_activations` overlay (§17).

### 12.4 Value propagation is a case operation

IDAES `propagate_state(arc, direction, overwrite_fixed)` copies port member values from source to destination. The platform defines it as a runtime operation producing an overlay case:

```text
propagate(connection, direction = forward | backward, overwrite_fixed = false):
  for each member m and index i of the connection's ports:
    if destination symbol is free, or overwrite_fixed:
      case_specs.initial[destination symbol] := current value of source symbol
```

The operation writes rows, never mutates a model; initialization plans (§17) sequence such operations explicitly.

### 12.5 Topology closure

Pass P5 materializes:

- `inferred.instance_tree` (transitive containment) and `inferred.instances` (expanded submodels, including per-time and per-node state instances);
- `inferred.topology_edges`: a directed multigraph of unit instances from connections (built in `petgraph`, persisted as rows);
- `inferred.scope_members` and `inferred.boundary_crossings` for every `authored.scopes` row, using the cut-set rule (`internal`, `external`, `inbound`, `outbound`) of the metamodel doctrine; flowsheet-level element and energy audits and the diagnostics "external variables" check are queries over these relations;
- `inferred.tear_candidates`: for each cycle family of the unit graph, the edge set that breaks all cycles, computed as a feedback arc set: `heuristic` uses `petgraph::algo::greedy_feedback_arc_set` over the unit multigraph (it returns exactly the recycle edges on flowsheet-shaped graphs, and reproduces the FOQUS cycle-count choice under the platform's edge weights), and `mip` solves the minimum feedback arc set through the NL backend and a MILP solver. The search is library work; the tear *policy* — which connections may be torn and at what cost — is authored data on `connections.tear_cost`. Tears feed the sequential-modular initialization plan (§17.4).

### 12.6 Translators, state junctions, and scalers

- **Translator.** The template declares two states and no equations; the author supplies mapping equations in the instance's `instance_equations` (an authored relation of the same shape as `template_equations` but instance-scoped). The compiler's unit inference refuses `enth_mol_in == enth_mol_out` across packages with different reference states unless the author converts explicitly, which is the failure mode IDAES leaves to the user.
- **StateJunction.** One state, two ports; connection equations on both sides make it a named stream. Stream tables treat it as a stream anchor.
- **StreamScaler.** Outlet members are expressions `multiplier · inlet member` for flow-like members (members whose quantity kind is extensive) and pass-through otherwise; the "flow-like" test is by quantity kind (`extensive = true`), not by name substring as in IDAES.

---

## 13. Flowsheets, time, and dynamics

### 13.1 The flowsheet template and time domain

`unit.flowsheet@1` (IDAES `FlowsheetBlock`) declares:

| Parameter / feature | Rule |
|---|---|
| `dynamic` (`true`, `false`, `inherit`) | top-level `inherit` resolves to `false` with a warning; nested flowsheets inherit the parent's value; `dynamic = true` inside a steady-state parent is `flowsheet.dynamic_in_steady_parent` (IDAES `DynamicError`) |
| `time_set` (breakpoints) | default `[0]`; a dynamic top-level flowsheet with the default set is promoted to `[0, 1]` |
| `time_units` | required when dynamic; must be a time unit |
| `time` (external domain reference) | if supplied, must be continuous when dynamic |
| `default_property_package` | fallback for children with `property_package = useDefault`; resolved by walking `instance_tree` upward (IDAES `_get_default_prop_pack`) |

The time domain is an `authored.domains` row of kind `time`, `continuous = dynamic`, with its members in `domain_members` (steady state: one member `t = 0`; dynamic: the breakpoints, extended by discretization in P11). Every state instance and time-indexed symbol is indexed by this domain even in steady state, so `state[0.0]` addressing survives unchanged.

### 13.2 Dynamic and holdup inference

IDAES `_setup_dynamics` becomes four rules in `inferred.instance_features`:

```text
dynamic(I)     ← authored dynamic(I) = true
dynamic(I)     ← authored dynamic(I) = inherit ∧ parent(I, P) ∧ dynamic(P)
has_holdup(I)  ← authored has_holdup(I) = inherit ∧ dynamic(I)            -- default follows dynamic
error(I, dynamic_in_steady_parent) ← dynamic(I) ∧ parent(I, P) ∧ ¬dynamic(P)
error(I, holdup_required)          ← dynamic(I) ∧ authored has_holdup(I) = false
```

### 13.3 Accumulation terms and derivative symbols

When an instance is dynamic, the control-volume templates declare `material_accumulation[t,(p,j)]`, `energy_accumulation[t,p]`, `element_accumulation[t,e]` with `role = derivative`, `wrt = time`. P7 creates the symbol and a `math_dae_links` row linking it to its state (`material_holdup`, …). The balance equations reference the derivative symbol; nothing else changes. Steady-state instances substitute `0` for the accumulation contribution (a compile-time guard, not a runtime branch).

The IDAES steady-state initial condition convention (`fix_initial_conditions`: fix every accumulation at `t0` to zero) is the reference case overlay `overlay.steady_initial_conditions@1`.

### 13.4 Discretization is a lowering pass (P11)

An `authored.discretization_policies` row (selected per domain by `continuous_domains.discretization_policy_id` or overridden by `case_policies`) attaches to a continuous domain (time or a control volume's length domain). P11 implements Pyomo DAE's semantics as data:

1. **Mesh.** `finite_elements` (`nfe`) equal-width elements over the domain bounds, retaining authored breakpoints; collocation adds `ncp` interior points per element (Radau roots include the element end, Legendre roots do not). Rows in `compiled.meshes`, `compiled.mesh_nodes`.
2. **Symbol expansion.** Every symbol indexed by the domain is expanded from its authored members to all mesh nodes (`lifecycle = generated_discretization`, IDs from §5.1).
3. **Discretization equations.** For every derivative symbol `d` of state `v` with respect to the domain, one equation per node per the scheme, named `<d>_disc_eq[...]`:

| Scheme | First derivative at node `k` | Skipped at |
|---|---|---|
| `BACKWARD` | `d[k] = (v[k] − v[k−1]) / (t_k − t_{k−1})` | first node |
| `FORWARD` | `d[k] = (v[k+1] − v[k]) / (t_{k+1} − t_k)` | last node |
| `CENTRAL` | `d[k] = (v[k+1] − v[k−1]) / (t_{k+1} − t_{k−1})` | first and last nodes |
| `LAGRANGE_RADAU` | `d[k] = Σ_{j=0..ncp} v[l+j]·adot[j][k−l] / (t_{l+ncp} − t_l)` with `l` the element's lower boundary | first node |
| `LAGRANGE_LEGENDRE` | `d[k] = Σ_{j=0..ncp} v[l+j]·adot[j][k−l] / (t_{l+ncp+1} − t_l)`; element-boundary nodes instead get continuity `v[k] = Σ_j v[l'+j]·afinal[j]` named `<v>_<domain>_cont_eq` | first node and element boundaries (for the derivative form) |

Second derivatives use `adotdot` (collocation) or the standard three-point stencils (finite difference); mixed derivatives are unsupported (diagnostic `disc.mixed_derivative`, as in IDAES). Stencil weights are stored in `compiled.stencils` so that the equations are data-generated `Affine` nodes.

4. **Integrals.** `Integral` nodes lower to trapezoid sums over the mesh (Pyomo `Integral` semantics) unless a quadrature rule is attached.
5. **Boundary filters** on 1D balances (§10.4) are evaluated against the mesh.
6. **Scaling propagation.** The IDAES post-discretization step (`scale_time_discretization_equations`) is the rule `sf(dv/dt) = sf(v) / sf(t)` and `sf(disc_eq) = sf(dv/dt)`, `sf(cont_eq) = sf(v)`, applied by the scaling pass because P11 has already produced the rows; the "propagate after build" failure cannot occur.

### 13.5 Dynamic operations as case operations

| IDAES utility | Platform operation | Result |
|---|---|---|
| `copy_values_at_time(fs_tgt, fs_src, t_target, t_source)` | `copy_at_time` over `runtime.solutions` and `case_specs` | new overlay case with `initial` values |
| `copy_non_time_indexed_values` | `copy_non_time_indexed` | overlay |
| `copy_values_from_point(blk, t0)` | `broadcast_from_point` | overlay |
| `deactivate_model_at(b, time, points)` | `case_activations` rows for equations indexed by those points | overlay |
| `deactivate_constraints_unindexed_by`, `fix_vars_unindexed_by` | selector-based activation/treatment rows | overlay |
| `initialize_by_time_element` | `plan.time_element_march@1` (§17.4) | plan |
| `get_derivatives_at`, `find_comp_in_block_at_time` | queries over `math_dae_links` and `symbols.index` | — |

### 13.6 Trajectory backends

- **Fully discretized NLP (default).** The problem after P11 is an algebraic system; any NLP backend solves it. Consistent initial conditions are a first stage of the dynamic plan (solve at `t0` with everything else inactive).
- **PETSc TS through the NL backend.** The IDAES `petsc_dae_by_time_element` workflow is reproduced as a plan: per time interval, the NL writer emits the subsystem at the target node with the `dae_suffix` (`ALGEBRAIC = 0`, `DIFFERENTIAL = 1`, `DERIVATIVE = 2`, `TIME = 3`) and `dae_link` suffixes generated from `math_dae_links`, runs `petsc_ts` with `--ts_init_time`/`--ts_max_time`, reads the trajectory, and writes `runtime.solutions` for the interval, interpolating skipped nodes. A fixed derivative with a non-zero value and a fixed differential variable paired with a free derivative are the same errors IDAES raises (`dae.fixed_derivative`, `dae.fixed_state_free_derivative`).
- **`diffsol` (optional).** A native DAE integrator fed directly from the evaluation program (residual and Jacobian-vector-product callbacks; the evaluation program's sparse rows supply the product). It is a backend binding; the IR is unchanged; DiffSL, its own model language, is never used. `diffsol` carries its own linear-algebra stack (`diffsol-la`/`diffsol-nl`, not `faer`), accepted for the trajectory backend only; whether every discretized process model of §13 is a semi-explicit DAE that `diffsol` accepts is verified per model by P16 (`backend.unsupported_dae_class`).

---

## 14. The compiler

### 14.1 Pass pipeline and contracts

Every pass is a `reference.pass_specs` row; the driver derives the DAG from declared inputs and outputs. Passes are pure functions of the snapshot they read; outputs are new relation versions; nothing writes into `authored`.

| Pass | Name | Reads (namespaces) | Writes | Preconditions → postconditions | Determinism | Main diagnostics |
|---|---|---|---|---|---|---|
| P0 | Package and schema resolution | `authored.packages`, `reference.schema_*` | `normalized.package_graph` | dependency versions satisfiable → every referenced package pinned by content hash | deterministic | `pkg.unresolved`, `pkg.version_conflict`, `schema.version_mismatch` |
| P1 | Authoring parse | package documents | `authored.*` (via change sets, §22.2), including the identity-based target rows (`case_spec_targets`, `case_activation_targets`, `observation_targets`) | documents parse → every entity registered with an `id` (explicit policy) or a name-derived id (named policy); every `pse.expr_dsl` syntactically valid with a source span; every `pse.target_path` resolved to entity identities | deterministic | `parse.syntax`, `parse.unknown_key`, `parse.missing_id`, `parse.unresolved_target` |
| P2 | Primitive-fact validation | `authored.*`, `reference.*` | `provenance.pass_records`, findings | — → all invariants of `reference.schema_invariants` hold | deterministic | one class per invariant kind |
| P3 | Canonicalization | `authored.*` | `normalized.*`, including the parsed expression graphs `normalized.*_expr_*` for every `pse.expr_dsl` column | validated → aliases resolved, defaults materialized, units normalized to package unit sets (no affine unit survives), expression paths resolved against template scopes, derived IDs assigned | deterministic | `canon.unresolved_path`, `canon.unit_mismatch` |
| P4 | Type and capability closure | `normalized.*` | `inferred.phase_species`, `inferred.instance_features`, method compatibility | — → every feature resolved; every method selection compatible with its scope | deterministic (fixed point) | `feature.rule_violation`, `method.incompatible` |
| P5 | Topology and containment closure | `normalized.*`, `inferred.instance_features` | `inferred.instances`, `instance_tree`, `ports`, `port_members`, `topology_edges`, `scope_members`, `boundary_crossings`, `tear_candidates` | — → every port bound; every connection endpoint exists | deterministic | `conn.member_mismatch`, `topology.dangling_port` |
| P6 | Property demand closure | `inferred.*`, `reference.method_specs` | `inferred.property_requirements`, `method_resolutions` | — → every requirement resolved or explicitly unresolved | deterministic (fixed point) | `prop.unsupported`, `prop.ambiguous`, `prop.unknown_subject` |
| P7 | Template instantiation | `inferred.*` | `compiled.symbols`, `symbol_groups`, template equations as `math_*` (indexed), contributions | — → every guard decided; every symbol has quantity type and canonical unit | deterministic | `template.guard_undecidable` |
| P8 | Law expansion and connection equations | contributions, law instances, connections | `math_equations` (balances, connection equations), negative-completeness rows | — → every contribution included exactly once per applicable law | deterministic | `law.unsupported_binding`, `law.basis_conversion_missing`, `law.excluded_candidate` (info) |
| P9 | Method realization and kernel binding | `method_resolutions` | property `math_*` rows, `kernel_bindings`, `UnitConvert` nodes | — → every `KernelCall` bound; every natural-unit method converted | deterministic | `kernel.unbound_parameter` |
| P10 | Math canonicalization and unit inference | `compiled.math_*` | canonical `math_*` (hash-consed, affine-normalized, typed), `math_equations.residual_quantity_type_id` | — → every node typed; DAG acyclic | deterministic | `math.unit_inconsistent`, `math.cyclic_expression`, `math.domain_violation_static` |
| P11 | Discretization lowering | `math_*`, `authored.discretization_policies` (via `normalized`), `math_dae_links` | meshes, nodes, stencils, expanded symbols, `*_disc_eq`, `*_cont_eq` (as indexed equations) | policies attached → no `Derivative`/`Integral` nodes remain for discretized domains | deterministic | `disc.mixed_derivative`, `disc.missing_policy` |
| P12 | Index expansion | `math_indexed_equations`, `math_free_indices`, `valid_index_tuples`, `symbol_groups`, meshes | scalar `math_equations` (one row per valid index tuple that passes the filter, with `parent_indexed_equation_id`); `Gather` nodes whose coordinates are fully bound rewritten to `SymbolRef`; boundary filters of §10.4 evaluated | every free index has a domain with materialized members → every equation is scalar; no `Broadcast` node and no free index remains; IDs per §5.1 | deterministic (tuples in domain-member order) | `expand.unbound_index`, `expand.empty_domain` (info) |
| P13 | Case binding | `authored.cases`, `case_specs`, `case_spec_targets`, `case_activations`, `case_objectives`, overlays | `compiled.problems`, `variable_order`, `equation_order`, `case_bound_substitutions` | case overlays resolve → every symbol has a treatment, bounds, and an initial value or an explicit "none"; every `treatment = parameter` symbol has a substitution row | deterministic (ordinals by semantic ID) | `case.unknown_target`, `case.conflicting_overlay` |
| P14 | Structural analysis and static attributes | problem, `math_*`, `case_bound_substitutions` | `incidence`, `dm_partition`, `blocks`, `math_static_analysis`, `problems.degrees_of_freedom` | — → DOF, DM partition, SCC order, per-equation families and roles, per-variable roles, all computed on the case-bound view with `parameter_dependence` recorded | deterministic | findings of §15.2 |
| P15 | Plan generation | problem, static analysis, package defaults, `authored.solver_profiles`, scaler and initializer templates | `scaling_plans`, `variable_scales`, `equation_scales`, `initialization_plans`, `init_stages`, `solve_plans` | — → a solve plan selected from static attributes only (§18.7) | deterministic | `plan.no_feasible_solve_plan` |
| P16 | Backend lowering | problem, plans | `evaluation_programs`, `sparsity_patterns`, `backend_bindings`, NL text, Pyomo bundle | backend chosen → every opcode supported or the binding is `unsupported` with the opcode list | deterministic per backend | `backend.unsupported_opcode`, `backend.derivative_unavailable` |

The `useDefault` resolution of balance types (representative state block's defaults), IDAES's `_get_representative_property_block`, is part of P4.

P12 is the only pass that changes the cardinality of the mathematics: P7–P11 keep every equation in its indexed form (one row per instance and declaration), and P12 is where the compact form is expanded, why, and with what IDs (doctrine P18 of the metamodel: expansion rules, domains, and generated identities are inspectable).

### 14.2 The rule compiler

Set-oriented inference (P4–P6, P8's contribution matching, P5's scope closure) is written as `reference.rule_specs`, not as Rust loops over rows. A rule body is a typed `RulePlanSpec` (a small algebra: scan, filter, project, equi-join, anti-join for stratified negation, union, distinct) compiled to a DataFusion `LogicalPlan` through `LogicalPlanBuilder`, optimized, and executed against the snapshot session. Example (rule for valid phase–species pairs):

```text
rule phase_species_valid @1  stratum 1  head inferred.phase_species(material_system_id, phase_id, species_id, henry)
  ← material_system_species(M, J) ⋈ material_system_phases(M, P)
    ⋈ species(J, valid_phase_types) ⋈ phases(P, phase_type)
    where phase_type ∈ valid_phase_types            -- or valid_phase_types is null and phase is not aqueous
    ▷ authored.phase_species restricts (P, J) when the phase declares an explicit list
    left ⋈ henry_declarations(J, P) → henry := present
```

Execution rules:

1. **Strata run in order**; within a stratum rules are evaluated to a least fixed point. Recursive closures (containment, subtype, reachability) use the generic fixed-point executor: seed, evaluate delta plans, deduplicate by semantic key, append, repeat until no new rows. DataFusion's recursive CTE support may be used for closures whose provenance can be reconstructed from the result; where per-row derivations are required the external loop is used.
2. **Negation is stratified**: a rule may negate only relations fully computed in a lower stratum.
3. **Four-valued predicates.** Matching predicates return `true | false | unknown | conflict`. `true` rows are written to the head relation. `unknown` and `conflict` rows are written to `inferred.undecided` with the rule, the key, a reason and the supporting rows, and produce diagnostics; a candidate is never silently dropped (metamodel P18), and no consumer can mistake an undecided fact for a true one because head relations hold only decided-true rows. Inside a plan, `Expr::IsUnknown`/`IsNotUnknown` cover the three SQL truth values; `conflict` (two rules asserting incompatible values for one key under `conflict_policy = reject`) is detected by the executor after the stratum's fixed point.
4. **Every head row carries a derivation** (`provenance.derivations`: rule id, supporting rows, fingerprint) at the granularity the registry declares for the head relation (`schema_relations.derivation_granularity`): `row` for inference, law expansion and method resolution, where negative completeness is the deliverable; `rule` for mechanically expanded relations (index expansion, discretization, connection equations), where the derivation is the rule plus the ID formula of §5.1 and is reconstructed on demand rather than stored per row. A benchmark in §24.3 measures derivation rows and bytes per equation so the granularity assignment is evidence-based.
5. **Determinism, and the engine as a declared input.** Output relations are sorted by primary key before hashing (arrival order across partitions is nondeterministic and never assumed). The DataFusion session is built from a `reference.engine_profiles` row (§6.11): the DataFusion and Arrow crate versions; the ordered analyzer rule list installed with `Analyzer::with_rules` and the ordered optimizer rule list installed with `Optimizer::with_rules` (never the engine defaults, whose pipeline changes between releases; `Analyzer::add_function_rewrite` is never called after construction); the physical-optimizer rule list; and the semantic settings as an **explicit, versioned key allow-list** — every `datafusion.optimizer.*` and `datafusion.sql_parser.*` key, `datafusion.execution.time_zone`, `datafusion.execution.skip_physical_aggregate_schema_check` (asserted `false`) and `datafusion.execution.enable_ansi_mode` (asserted `false`) — read back from `information_schema.df_settings`; a namespace match would miss the two `execution.*` keys. Platform policy that can change a plan (kernel strictness defaults, the null policy of §14.2 rule 3) is registered as a `ConfigExtension` under `datafusion.pse.*`, so it appears in `df_settings` and enters the same hash. The profile's `content_hash`, the function registry hash (kernel digests) and the catalog snapshot are part of every rule-executing pass's memo key (§14.3). **The plan fingerprint is not part of the memo key**: the plan is a pure function of the rule plan, the catalog snapshot and the engine profile, all of which are already in the key, so hashing the plan adds no information to reuse validity. It is recorded in `provenance.pass_records.plan_fingerprints` as evidence, computed as blake3 of `logical_plan_to_bytes_with_extension_codec` under the platform `LogicalExtensionCodec`, which implements the *required* `try_encode_table_provider`/`try_decode_table_provider` (encoding a snapshot table as its `(snapshot_id, relation_id, version, content_hash)`) and overrides the provided `try_encode_udf`/`try_decode_udf` to encode a kernel UDF by its `KernelSpec` digest rather than by name. Those bytes are reproducible only because schema and field metadata are canonicalized at construction (§4.3): `datafusion-proto` serializes metadata in map iteration order, and a CI test encodes the same plan in two fresh processes and asserts byte equality. The fingerprint's definition is a platform contract (`pse.planfp.v1`) versioned like §5.3; cross-release stability of the protobuf representation is not assumed. Beside it the pass record stores the plan's `EXPLAIN` in `pgjson` format (queryable structure, evidence for humans and tools, never an identity) and, from the `observer` closures of `Analyzer::execute_and_check` and `Optimizer::optimize`, the ordered list of rules that actually fired on that plan.
6. **Head-schema typing.** Field metadata does not propagate through computed expressions in a DataFusion plan (only through column references, aliases carrying explicit metadata, UDF return fields, and — for `ARROW:extension:name` — projection, aliasing and grouping). The executor therefore re-types every head relation's output against the generated schema of `head_relation_id` — casting with `CastOptions { safe: false }` after `can_cast_types`, and reattaching the `pse.semantic.*` field metadata — before the rows are written; a plan whose output is not compatible with the head schema is a rule-compiler error (`rule.head_schema_mismatch`), never a runtime coercion.
7. **Key discipline.** Distinct, union-dedup and join keys in a rule plan are key-role columns (semantic IDs, ordinals, enums) — never `Float64`, because the engine's value equality merges `-0.0` with `+0.0` and treats NaN as self-equal (§5.3). Null-bearing key comparisons use `distinct`/`not_distinct` semantics and Kleene booleans (`and_kleene`/`or_kleene`) so that a null-versus-null candidate is neither dropped nor matched by accident. The rule compiler rejects a violating `rule_plan_nodes` row (`rule.float_key`).
8. **Rule nodes in plans (deferred).** Wrapping each rule body in a `LogicalPlan::Extension` node that carries `rule_id` survives the optimizer intact and would make rule attribution visible in `EXPLAIN`; it costs the 14-method `UserDefinedLogicalNode` contract and an `ExtensionPlanner`. It is not adopted now: derivations and `rules_fired` already answer "which rule produced this row". Trigger for adoption: the §22.4 diff report or agent tooling needs per-rule attribution inside a plan rendering.

Generated `ScalarUDFImpl` wrappers for kernels (§18.5) are registered in the session so that rule bodies can evaluate kernels in batch (for example, estimating bubble-point temperatures for every state instance in one plan during initialization-plan generation).

### 14.3 The pass engine

> Decision: ADR-0019, ADR-0020, ADR-0029

- **Memoization.** The pass engine keeps a memo keyed by `(pass_id, pass_version, content hashes of the declared inputs in `pass_specs.cache_key_inputs`, engine_profiles.content_hash for passes that execute plans — P2 validators, P4–P6, P8, and kernel batch evaluation wherever it is used)` → output artifact hashes, backed by the artifact store. Backdating is inherent: a pass that re-runs on changed inputs and produces byte-identical outputs leaves the next pass's key unchanged. Per-instance memoization inside P7 uses the same key shape over `(instance_id, template hash, resolved features)`. `salsa` is **deferred**: its distinctive value — automatic dependency tracking — duplicates the declared inputs of `pass_specs`, and an undeclared read is a contract violation, not something to track; its unwind-based cancellation cannot cross the Ipopt boundary. Trigger for adoption: a measurement showing that sub-pass granularity finer than per-instance is needed.
- **Session.** One DataFusion `SessionContext` per snapshot, built from the engine profile (§14.2 rule 5) with the catalog of §5.4, the `pse.*` extension-type registry (§4.4), the generated UDF registry, the `datafusion.pse.*` config extension, a **`FairSpillPool`** sized from `datafusion.runtime.memory_limit` (declared per deployment; never `UnboundedMemoryPool` — a bounded pool turns exhaustion into a typed, actionable `ResourcesExhausted` error naming the keys to change, an unbounded one into a process death), an explicitly configured `DiskManager` directory with `spill_compression` and `max_spill_file_size_bytes` declared, `target_partitions` validated against the rayon budget at construction (§18.8), and a `TimeProvider` (`SystemTimeProvider` in production, a fixed provider under test and reproduction). Passes obtain the session from the driver; no pass creates its own.
- **Parallelism.** Independent passes (for example scaling-plan generation and NL lowering) run concurrently on `rayon`; DataFusion partitions relational work internally. Thread budgets are coordinated (§18.8).
- **Failure.** A pass either succeeds with findings (severity below `error`) or fails with a typed failure (§23). Partial outputs are discarded; the pass record notes the failure class.

### 14.4 Incrementality

Dependencies are semantic. The table refines the proposal's invalidation matrix with the passes defined here:

| Change | Recomputed | Reused |
|---|---|---|
| A case value for a `treatment = fixed` or `free` symbol (`case_specs.value`, `initial`) | P13 onward; P14 only if treatment or activation changed, otherwise structural rows are reused and only `variable_order.initial` changes; evaluation program reused; scaling plan reused unless it depends on values (`AutoScaler`) | P0–P12 |
| A case value for a `treatment = parameter` symbol | P13 onward; the P14 rows whose `parameter_dependence` names the symbol (families, `incidence.linear`, and the incidence edge itself when the substituted value is or becomes exactly zero); the evaluation program's bound-parameter segment only | P0–P12; every P14 row that did not depend on the parameter |
| The engine profile (a DataFusion upgrade, a rule-list or semantic-setting change) | every rule-executing pass and everything downstream of it | passes that execute no plan, provided their input hashes are unchanged |
| A feature flag on one instance | P4 onward for that instance's subtree; other instances' P7 outputs reused because template instantiation is per instance and memoized per (instance, template, features) | everything for untouched instances |
| Discretization policy | P11 onward | P0–P10 |
| A method selection in a property package | P6 onward for states bound to that package | others |
| A new connection | P5, P8 (only the new connection's equations), P10 for the new nodes, P13 onward | all instance-level outputs |
| Kernel implementation version | P9 onward for bindings of that kernel; parity tests flagged | others |

The canonical graph (P10) never depends on a parameter value (§7.4 step 3). The only artifacts that do are the case-bound view's static attributes (P14) and the evaluation program's bound-parameter values, and each records the substituted values' hashes (`case_bound_substitutions.value_hash`) in its derivation, so a parameter change invalidates exactly those rows and nothing structural above them.

### 14.5 Closure proofs

At the end of P14 the compiler writes `provenance.closure_report` (one row per closure category of the metamodel doctrine §31: type, topology, scope, quantity, method, balance, optionality, equation, runtime) with `status ∈ {closed, open}` and the list of open items. A category is `open` while any `inferred.undecided` row belongs to it. A snapshot is publishable as a compiled problem only when every category is closed or the case explicitly declares the open items as intentional (unconnected optional ports, free decision variables).

---

## 15. Structural analysis and diagnostics

IDAES's diagnostics (`core/util/diagnostics_tools/**`, `core/util/model_statistics.py`) become (a) predicates and aggregates over `compiled` and `runtime` relations, (b) graph algorithms over `compiled.incidence`, (c) linear-algebra analyses over the evaluation program's Jacobian, (d) walks over the expression graph, and (e) auxiliary optimization problems built from the Jacobian or the problem. Every check writes `runtime.diagnostics_findings` rows; reports are views over findings. Thresholds are a relation (`reference.diagnostic_thresholds`) whose defaults are the IDAES `DiagnosticsToolbox.CONFIG` values.

### 15.1 Thresholds (defaults preserved from IDAES)

| Key | Default | Used by |
|---|---|---|
| `variable_bounds_absolute_tolerance` / `_relative_tolerance` | 1e-4 / 1e-4 | near-bounds caution |
| `variable_bounds_violation_tolerance` | 0 | at-or-outside-bounds warning |
| `constraint_residual_tolerance` | 1e-5 | large residuals warning |
| `constraint_term_mismatch_tolerance` | 1e6 (ratio) | mismatched terms |
| `constraint_term_cancellation_tolerance` | 1e-4 (relative) | canceling terms |
| `max_canceling_terms` | 5 | cancellation search |
| `constraint_term_zero_tolerance` | 1e-10 | term-is-zero cutoff |
| `variable_large_value_tolerance` / `_small_` / `_zero_` | 1e4 / 1e-4 / 1e-8 | extreme values |
| `jacobian_large_value_caution` / `_warning` | 1e4 / 1e8 | extreme Jacobian rows, columns, entries |
| `jacobian_small_value_caution` / `_warning` | 1e-4 / 1e-8 | same |
| `warn_for_evaluation_error_at_bounds` | true | closed vs open interval tests |
| `parallel_component_tolerance` | 1e-8 | near-parallel pairs |
| `absolute_feasibility_tolerance` | 1e-6 | infeasibility explanation |

### 15.2 Structural checks (no numerical values required)

| Check id | Definition (relational) | IDAES method |
|---|---|---|
| `struct.degrees_of_freedom` | `DOF = \|{free variables appearing in active equalities}\| − \|{active equalities}\|`; an equality is an equation with `sense = eq`; inequalities and definitions are excluded; fixed variables are excluded from the count but not from incidence | `degrees_of_freedom` |
| `struct.inconsistent_units` | P10 rejects unit-inconsistent authored equations, so on compiled problems this is empty by construction; the check remains for authored assertions | `display_components_with_inconsistent_units` |
| `struct.singularity` | Dulmage–Mendelsohn on the equality incidence graph (§15.3). Under-constrained set = unmatched ∪ underconstrained variables with underconstrained equations; over-constrained set = overconstrained variables with overconstrained ∪ unmatched equations; each reported per weakly connected component | `report_structural_issues`, `display_underconstrained_set`, `display_overconstrained_set` |
| `struct.evaluation_error` | interval propagation (FBBT semantics, using fixed values as bounds) over each active equation body and objective; flag `log`/`log10` when `lb ≤ 0` (or `< 0` when not warning at bounds), `sqrt` when `lb < 0`, `tan` when the node's own bounds are unbounded, `asin`/`acos` when outside `[−1, 1]`, `Div` when the denominator interval contains 0, `Pow` unless the exponent is provably a non-negative integer or the base is provably positive (IDAES integer-exponent rule: integer-domain variable, fixed integer, or integer-coefficient linear or quadratic form over integer variables) | `display_potential_evaluation_errors` |
| `struct.fixed_to_zero` | fixed variables with value exactly 0 | `display_variables_fixed_to_zero` |
| `struct.unused_variables` | variables not appearing in any active equation (count fixed separately) | `display_unused_variables` |
| `struct.external_variables` | variables in a scope's active equations whose owner instance is outside the scope | `display_external_variables` |
| `struct.no_free_variables` | equations whose leaves are all parameters or fixed variables | `display_constraints_with_no_free_variables` |
| `stats.*` | every `model_statistics` count (total, activated, deactivated blocks; total, activated, deactivated equalities and inequalities; variables total, fixed, unfixed, in activated equalities, only in inequalities, unused, fixed unused, derivative variables) as one SQL view with the IDAES definitions: "total" means on active scopes regardless of the item's own activity; a ranged inequality with `lower ≠ upper` is neither equality nor inequality | `report_statistics` |

### 15.3 Native structural algorithms (`pse-structural`)

- **Incidence construction.** From `compiled.incidence` restricted to active equalities and free variables (IDAES `IncidenceGraphInterface(include_inequality=False, include_fixed=False)`). Linear-only participation is known from the canonical form: a symbol occurring only in `Affine` terms with a non-zero coefficient participates linearly; coefficients that fold to exactly zero after canonicalization do not create an edge (the `standard_repn` rule).
- **Maximum matching:** Hopcroft–Karp, own implementation. `petgraph` ships only general-graph matching (`maximum_matching`, Gabow's algorithm, O(|V|³), input treated as undirected, and it panics on a degenerate node bound); the incidence graph is bipartite, so Hopcroft–Karp's O(|E|√|V|) is required. `is_bipartite_undirected` is asserted on the graph in debug builds. In CI the matching cardinality and structural rank are cross-checked against `scipy.sparse.csgraph.maximum_bipartite_matching` / `structural_rank` and Pyomo's `IncidenceGraphInterface` over the golden snapshots (§24.1).
- **Coarse Dulmage–Mendelsohn:** projected digraphs over the matching, alternating-path reachability from unmatched nodes on both sides, giving the row partition (unmatched, overconstrained, underconstrained, square) and column partition (unmatched, underconstrained, overconstrained, square), with the matching order preserved.
- **Block triangularization:** requires a perfect matching; Tarjan SCC on the matching-projected digraph (reversed), condensation DAG, topological order made deterministic by ordering ties on semantic ID. Output `compiled.blocks` with `kind = scc` and `order`.
- **Connected components** for independent subsystems.
- **Tear selection** (§12.5).

### 15.4 Numerical checks (require values)

The Jacobian is produced by the evaluation program (§18.2) at the current point and scaled as IDAES does: `J_scaled = diag(sf_equation) · J · diag(1/sf_variable)`; an "Ipopt gradient-based" emulation (`max_grad = 100`, `min_scale = 1e-8`) is available for comparison.

| Check id | Definition | IDAES |
|---|---|---|
| `num.condition_number` | Default: a Hager–Higham 1-norm condition estimate (a handful of sparse LU solves), reported as an estimate. On request: the IDAES-parity Frobenius form `‖J‖_F·‖J⁻¹‖_F` — for square `J` through `faer` sparse LU solves against the identity (O(n) solves, so it is an opt-in expensive diagnostic with the cost stated in the finding), for non-square `J` through `faer::linalg::svd::pseudoinverse_from_svd_with_tolerance` with the tolerance a declared threshold (`pseudo_inverse_tolerance`, default 1e-12 relative). "Exactly singular" is reported as undefined; a 2-norm estimate via the smallest singular values (§15.5) is also stored | `jacobian_cond` |
| `num.large_residuals` | per active equation: `r = max(lb − body, 0)`, then `max(r, body − ub)`; flag if `r·sf > tol`; unevaluable bodies are flagged with a null residual | `display_constraints_with_large_residuals` |
| `num.bounds_violation` | `sf·v ≤ sf·lb − tol` or `sf·v ≥ sf·ub + tol` | `display_variables_at_or_outside_bounds` |
| `num.near_bounds` | `mag = ub − lb`, or the single bound magnitude, or 0; `tol = max(abs_tol/sf, mag·rel_tol)`; flag if within `tol` of either bound | `display_variables_near_bounds` |
| `num.extreme_values` | `m = sf·\|v\|`; flag if `m > large` or (`m < small` and `m > zero`) | `display_variables_with_extreme_values` |
| `num.near_zero`, `num.none_value` | `sf·\|v\| ≤ zero_tol`; value missing | `display_variables_with_value_near_zero`, `display_variables_with_none_value` |
| `num.extreme_jacobian_rows` / `_columns` | row (column) 2-norms outside `[small, large]`; an empty row has norm 0 and is always flagged small | `display_constraints_with_extreme_jacobians`, `display_variables_with_extreme_jacobians` |
| `num.extreme_jacobian_entries` | `\|J_ij\| ≥ large` or `0 < \|J_ij\| ≤ small` | `display_extreme_jacobian_entries` |
| `num.parallel_rows` / `_columns` | bucket vectors by the sorted support of entries with `\|a\| > tol` and `\|a\|/max > tol`; within a bucket, flag pairs with `\| \|u·v\| − ‖u‖‖v‖ \| ≤ tol` or `≤ tol·max(‖u‖, ‖v‖)` (anti-parallel counts) | `display_near_parallel_constraints`, `_variables` |
| `num.mismatched_terms` | for every sum node (with the equation sides negated into one sum): if `max\|term\| / min\|term\| ≥ 1e6` over terms above the zero cutoff | `display_constraints_with_mismatched_terms` |
| `num.canceling_terms` | combinations of 2..`max_canceling_terms` terms whose sum is `≤ tol·max\|term\|`; products distribute a sum child; divisions divide numerator terms by the denominator value; equality nodes are analysed only when at least one side is a sum | `display_constraints_with_canceling_terms`, `display_problematic_constraint_terms` |
| `num.outside_valid_range` | values outside `method_specs.validity` for the property | `list_components_with_values_outside_valid_range` |

### 15.5 Advanced analyses

- **SVD toolbox.** Smallest `k` singular values of the scaled equality Jacobian: dense `faer` SVD below `svd_dense_limit` (default 2,000 rows); above it, shift-invert Lanczos over `faer::matrix_free` with the operator `(JᵀJ)⁻¹` applied through a sparse LU factorization of `J` (`partial_svd` alone targets the *largest* singular values and cannot be used for this); problems whose LU factorization fails fall back to a declared `svd.unavailable` finding rather than a dense attempt; rank deficiency count below `singular_value_tolerance` (1e-6); for each small singular value, the variables and equations whose singular-vector components exceed `size_cutoff_in_singular_vector` (0.1).
- **Degeneracy Hunter** (Dowling and Biegler). Stage 1 candidate MILP: `Jᵀν = 0`, `ν ∈ [−M − m_small, M + m_small]`, binaries `y_pos`, `y_neg` forcing `|ν| ≥ m_small` when set, `Σ(y_pos + y_neg) ≥ 1`, minimize `Σ|ν|`; columns with `Σ|J_ij| ≤ 1e-6` skipped. Stage 2 irreducible degenerate set MILP per candidate: fix `ν_k = 1`, `Jᵀν = 0`, `−M·y ≤ ν ≤ M·y`, minimize `Σy`; members are equations with `y > 0.9`. Both are problems built as `compiled` artifacts and solved through the NL backend with a MILP solver profile (`scip` default, `cbc`).
- **Ill-conditioning certificate** (Klotz): LP `Jᵀy = res`, `Σy_pos − Σy_neg = 1`, minimize `‖res‖₁`, then minimize `‖y‖₁` at fixed residual norm; report components by `|y|` down to `ratio_cutoff` of the largest.
- **Infeasibility explanation** (minimal intractable system). Elasticize in three rounds (variable bounds; inequalities; everything) with slack variables, recording violated items and re-fixing their slacks; then a deletion filter over the elastic set (deactivate each; if the subproblem becomes feasible the item is essential). Output: the MIS and the guard set. Implemented as a plan over case overlays and slack symbols, not as model cloning.
- **Convergence analysis.** A case set (§19.3) solved with Ipopt; per sample record iterations, iterations in restoration, iterations with regularization, CPU time, and whether the numerical checks raise warnings; compare to a baseline with `rel_tol = 0.1`, `abs_tol = 1`.

### 15.6 Reports

`report_structural_issues` and `report_numerical_issues` are projection views: model statistics, warnings (ordered as IDAES orders them: DOF, units, structural singularity, evaluation errors; residuals, bounds, extreme Jacobian norms at warning thresholds, parallel components), cautions (fixed-to-zero, unused; near bounds, near zero, extreme values, none values, mismatched, canceling, no free variables, extreme Jacobian at caution thresholds), and next-step check ids. `assert_no_structural_warnings` and `assert_no_numerical_warnings` are the same views with an error policy, usable as CI gates.

---

## 16. Scaling as an explicit transformation

### 16.1 Model

Scaling is data in `compiled.variable_scales` (`scale`, `offset`, `source`) and `compiled.equation_scales` (`scale`, `scheme`), both with derivations. The transformation is

```text
x = x_ref + D_x x̂           (D_x = diag(1/scale_v), x_ref = offset)
r̂(x̂) = D_r r(x_ref + D_x x̂)  (D_r = diag(scale_e))
Ĵ = D_r J D_x
```

Backends consume it in one of two ways, recorded in `solver_profiles.scaling_mode`: the native and NL backends pre-scale the problem (equivalent to Pyomo's `scale_model = true` writer option) and unscale results on read-back; the NL backend can alternatively emit the `scaling_factor` suffix and set `nlp_scaling_method = user-scaling` for Ipopt. Expression scaling hints (IDAES `scaling_hint` on named expressions) are `variable_scales` rows for expression symbols and serve only the nominal-value algebra.

Validity: a scale is strictly positive and finite; values are clamped to `[min, max]` per component kind (`1e-10`, `1e10`) with a debug finding when clamping occurred.

### 16.2 Sources of scaling factors, in precedence order

| Source (`ScaleSource`) | Rule | IDAES origin |
|---|---|---|
| `user` | `case_specs.scaling_factor` overlay | `set_scaling_factor`, JSON scaling files |
| `template_default` | `template_scaling_defaults` rows (IDAES `DEFAULT_SCALING_FACTORS`), including the markers `user_input_required` (error unless a user value exists), `user_input_recommended`, `user_set_manually` | `CustomScalerBase.DEFAULT_SCALING_FACTORS` |
| `package_default` | `authored.default_scaling` by property kind and index | `default_scaling_factors` |
| `units` | `UNIT_SCALING_FACTORS`: per quantity kind a reference unit and factor (temperature: `1e-2` per K; pressure: `1e-5` per Pa), converted to the symbol's unit | `scale_variable_by_units` |
| `bounds` | `mag = 0.5(ub + lb)` when both bounds, else the single bound; `sf = 1` if `mag = 0` else `1/\|mag\|` | `scale_variable_by_bounds` |
| `nominal_magnitude` | `quantity_types.nominal_magnitude` (§8.5); IDAES's SI table (temperature 100 K, pressure 1e5 Pa, molar density 100 mol/m³, molar enthalpy 1e4 J/mol, entropy 100 J/mol/K, fugacity 1e4 Pa, mole fractions 1e-3, mw 1e-3 kg/mol), rounded to the nearest power of ten in the package unit | `populate_default_scaling_factors` |
| `definition_constraint` | for `v == f(w…)`: substitute every other symbol by its nominal value `1/sf_w` (or `1/value` clamped, or 1), solve the scalar equation for `v` (Newton; nonlinear-in-`v` or zero-derivative cases are errors), `sf_v = 1/\|v\|` | `scale_variable_by_definition_constraint` |
| `propagated` | state-to-state by symbol name and index (`propagate_state_scaling`); derivative `sf(dx/dt) = sf(x)/sf(t)`; discretization equations inherit the derivative's factor, continuity equations the state's | `propagate_state_scaling`, `scale_time_discretization_equations` |
| `magnitude` | `sf = 1/\|value\|` when `\|value\| > zero_tolerance`, else 1 | `AutoScaler.scale_variables_by_magnitude` |

### 16.3 Nominal value algebra (constraint scaling without a solved point)

`get_sum_terms_nominal_values(expr)` returns one signed nominal value per additive term. It is an operator-indexed evaluation over the IR (a native visitor, generated from the operator table so it cannot drift from the opcode catalog):

| Node | Result |
|---|---|
| symbol (variable) | `sign/sf` where `sign` is +1 if `lb ≥ 0` or the domain is non-negative, −1 if `ub ≤ 0` or the domain is non-positive, else the sign of the current value (+1 if none); if the symbol has no scale but has a value, the value itself |
| symbol (parameter) | its value |
| expression symbol with a hint | `±1/hint` (sign from the current value, + if unevaluable); the subtree is not descended |
| `Affine`, equality/inequality/range | concatenation of children's term lists (coefficients applied) |
| `Mul` | outer product of the two lists |
| `Div` | `[Σ(num)/Σ(den)]`, denominator 0 replaced by 1 with a finding |
| `Pow` | `[\|Σ(base)\|^Σ(exp)]` |
| `Neg`, `Abs` | applied termwise |
| transcendental | function applied to `Σ(child)` |
| `Conditional` | concatenation of both branches |
| `KernelCall` | kernel evaluated at `Σ` of each input |
| constant, unit | the value; 1 |

Constraint schemes over the non-zero nominal terms `n_i` (default `inverse_maximum`):

| Scheme | `sf` |
|---|---|
| `harmonic_mean` | `Σ_i 1/\|n_i\|` |
| `inverse_sum` | `1/Σ_i \|n_i\|` |
| `inverse_root_sum_squared` | `1/√(Σ_i n_i²)` |
| `inverse_maximum` | `1/max_i \|n_i\|` |
| `inverse_minimum` | `1/min_i \|n_i\|` |

Nominal derivative norm (`scale_constraint_by_nominal_derivative_norm(norm)`): for each variable `v` in the equation, set every other variable to its nominal value `1/sf_w`, set `v` to 1 (its perfectly scaled value), evaluate `∂body/∂v · (1/sf_v)` with the evaluation program's derivative; `sf = 1/‖(pjac_v)‖_norm`. The Jacobian-norm autoscaler is the same computation at the current point, composing with existing factors: `sf_new = sf_old/‖row‖`.

### 16.4 Scaler templates

A `scaler` template declares the four-stage pipeline IDAES's `CustomScalerBase.scale_model` runs (variable routine, first fill-in, constraint routine, second fill-in), the defaults, and the submodel dispatch order (user-supplied scaler for the submodel → package default state or reaction scaler → the template's default scaler). Reference scaler templates exist for every unit template of §11 with the IDAES routines (for example `scale.cstr`: control-volume routine, `deltaP` by default, `heat[t]` by `1/(0.1·Σ_p nominal enthalpy inflow)`, `volume` forced to `1e3`, reaction rates `1e-3`, constraints by `inverse_maximum`; `scale.gibbs`: `lagrange_mult[t,e]` by `1/(R·T_nominal)`, constraints by `inverse_sum`).

### 16.5 Profiling and persistence

The scaling profiler is a plan that runs the case under a matrix of scaler methods (variables only; each nominal scheme; nominal L1 and L2 norms; actual L1 and L2 norms), with perfect (magnitude-based) and user variable scaling, and records the condition number and the Ipopt iteration statistics (iterations, restoration, regularization) per cell in `runtime.profile_results`. Scaling factors are relations; the IDAES JSON scaling-file format is an import/export adapter.

---

## 17. Initialization plans

### 17.1 Plan model

An initialization plan is a `compiled.initialization_plans` row with ordered `init_stages`. Stage kinds:

| Kind | Semantics |
|---|---|
| `apply_overlay` | activate an overlay case (treatments, fixes, activations, initial values); overlays stack |
| `propagate` | copy values along connections (§12.4) |
| `estimate` | run an estimation kernel (bubble/dew Newton estimates, Rachford–Rice vapor fraction, outlet-state estimation) writing `initial` values |
| `solve_subset` | solve the equations selected by the target with the stage's solver profile; DOF must be 0 unless the stage says otherwise |
| `solve_blocks` | block-triangular sequence: 1×1 blocks by scalar Newton (`calculate_variable_from_constraint`), larger blocks by the profile's solver; inputs of each block fixed to values from earlier blocks |
| `continuation` | homotopy schedule (§17.5) |
| `call_plan` | run a submodel's plan (state blocks, reaction blocks, plug-ins) |
| `check` | DOF, perfect matching, residual, or missing-value check with a failure policy |
| `restore` | discard overlays down to a named level, keeping solution values |

State snapshot and restore follow IDAES `StoreState` semantics: at the end of a plan the fixed status of every variable and the active status of every equation are restored to the pre-plan case, values of variables that were fixed are restored, and values of free variables keep their solved values. Because overlays are immutable rows, "restore" is discarding overlays, not replaying mutations.

Every stage run is a `runtime.runs` row with `parent_run_id` = the plan run and `stage_id`; failures carry the stage.

### 17.2 Standard plan templates

| Plan template | Stages | IDAES source |
|---|---|---|
| `init.block_triangularization@1` | `check(DOF = 0)`; `check(perfect matching)` (failure ⇒ structural singularity); `solve_blocks` over the SCC order with the block solver profile (`tol = 1e-8`, `max_iter = 200`, scaled); `solve_subset(all)` unless `skip_final_solve`; postcheck | `BlockTriangularizationInitializer` |
| `init.single_control_volume@1` | for 0D: `call_plan(properties_in)`; `estimate(outlet ← inlet, with the IDAES pressure/temperature special cases when `deltaP`/`deltaT` are fixed)` or `copy values` when `copy_inlet_state`; `call_plan(properties_out)`; reactions: `apply_overlay(fix external variables)`, `call_plan(reactions)`, `restore`; `solve_subset(unit)`. For 1D: `estimate(states along x)`, `call_plan(properties)`. Plug-ins: `prepare` (deactivate) before, `initialize` and `finalize` after, then `solve_subset(all)` when plug-ins exist | `SingleControlVolumeUnitInitializer`, `ModularInitializerBase` |
| `init.from_data@1` | `apply_overlay(values from a solution or file, free variables only)`; postcheck | `FromDataInitializer` |
| `init.modular_properties@1` | 0 `apply_overlay(fix state symbols; deactivate sum_mole_frac_out, equilibrium constraints when state is phase-component flows, inherent equilibrium)`; 1 `estimate(critical properties; Tbub, Tdew, Pbub, Pdew by damped Newton with the IDAES `TOL = 0.1`, 30 iterations, ±50 K step clamp; transition compositions)` then `solve_subset(bubble/dew/critical equations only)`; 2 `estimate(_teq by the smooth max/min clipping)`; 3 `estimate(state_initialization: K-values from Psat/Henry, vapor fraction by Tbub/Tdew position or modified Rachford–Rice, phase compositions)`; 4 `apply_overlay(fix temperature)`, activate flow balances, phase fraction and mole-fraction equations, `log_*` equations with back-calculated values, equilibrium constraints (unless state is phase-component flows), the pair's VLE constraints; `solve_subset` when DOF = 0; 5 activate all except `do_not_initialize` equations, unfix temperature, seed `log_*` symbols as `log(max(value, 1e-8))`, `solve_subset`; 6 `restore` | `ModularPropertiesInitializer` |
| `init.mixer@1` | `call_plan(each inlet state)`; `estimate(mixed: pressure ← min inlet, flow-like ← sum, others ← mean)`; `call_plan(mixed)`; if pressure equality active: overlay deactivating it and fixing `P_mixed` to the first inlet, `solve_subset`, `restore`; else `solve_subset` | `MixerInitializer` |
| `init.separator@1` | `call_plan(mixed)`; overlay deactivating everything but `sum_split_frac`, `solve_subset` if DOF ≠ 0, `restore`; ideal ⇒ done; `estimate(outlet states: intensive unchanged, flows × split fraction)`; `call_plan(outlets)`; `solve_subset` | `SeparatorInitializer` |
| `init.hx_0d@1`, `init.hx_1d@1`, `init.shell_and_tube@1` | duty guess `0.25·U·A·(T_hot,in − T_cold,in)` fixed on both sides with the heat-transfer equations inactive, solve, then release and solve; the shell-and-tube variant fixes `temperature_wall` to the mean inlet temperature and solves the hot side alone first | `HX0DInitializer`, `HX1DInitializer`, `ShellAndTubeInitializer` |
| `init.isentropic_pressure_changer@1` | performance curves deactivated with `ratioP` (seeded 1.8 / 0.7) and efficiency (0.8) fixed; `call_plan(control volume)`; copy outlet values to the isentropic state; fix isentropic temperature (or enthalpy) and deactivate `isentropic`, solve; release and solve; reactivate curves, unfix, solve | `IsentropicPressureChangerInitializer` |
| `init.valve@1` | `estimate(outlet pressure from ratioP, deltaP, or 0.95·P_in)`, then the adiabatic pressure-changer plan | `Valve.initialize_build` |
| `init.ms_contactor@1` | snapshot; fix transfer terms and volume fractions, extents to 0; deactivate non-balance equations; `solve_blocks` per stream; `restore`; if holdup: fix holdups with holdup equations inactive, solve, back-calculate holdups, solve | `MSContactorInitializer` |
| `init.feed@1` | `call_plan(properties)` | `FeedInitializer` |

The postcheck (`check(residuals ≤ constraint_tolerance·sf; no missing values)`) uses scaled residuals exactly as IDAES's `large_residuals_set` does. Status values are `Ok`, `Failed`, `DoF`, `PrecheckFailed`, `Error`.

### 17.3 Plug-ins and initialization order

`inferred.initialization_order(instance, ordinal)` lists the instance followed by its plug-ins (costing blocks register themselves). Plans run `prepare` in order and `finalize` in reverse, as `ModularInitializerBase` does.

### 17.4 Flowsheet and dynamic plans

- `plan.sequential_modular@1`: order units topologically with tears from `inferred.tear_candidates`; for each unit in order run its plan then `propagate` along outgoing connections; iterate tear streams by direct substitution or Wegstein (`accel` clamp `[−5, 0]`, `iterLim = 40`, `tol = 1e-5`, absolute or relative); finish with `solve_subset(all)`.
- `plan.time_element_march@1` (IDAES `initialize_by_time_element`): supported schemes `LAGRANGE_RADAU` and `BACKWARD` (others rejected); overlay deactivating all but `t0`, solve consistent initial conditions; deactivate time-unindexed equations and fix time-unindexed variables; for each element, reactivate its nodes, fix the differential variables and derivatives at the element start (or every variable at or before it when `fix_diff_only = false`), copy values forward, solve, then deactivate again; restore.
- PETSc time stepping and consistent-initial-condition solves (§13.6) are plan templates over the NL backend.

### 17.5 Continuation (homotopy)

`stage.kind = continuation` reproduces the IDAES homotopy meta-solver: for fixed symbols `v_i` with targets `t_i`, the continuation parameter `n` runs from 0 to 1 with `v_i = t_i·n + v_i,0·(1 − n)`; parameters `step_init = 0.1`, `step_cut = 0.5`, `iter_target = 4`, `step_accel = 0.5`, `max_step = 1`, `min_step = 0.05`, `max_eval = 200` with the IDAES validity ranges; after a successful step `s ← s·(1 + step_accel·(iter_target/iterations − 1))` clamped to `[min_step, max_step]`; on failure roll back to the last solved overlay and `s ← max(min_step, s·step_cut)`; terminations `optimal`, `minStepLength`, `maxEvaluations`, `infeasible`, `other` (converged only with regularization). Each step is a `runtime.runs` row; the progress fraction is recorded.

---

## 18. Backends

### 18.1 The `CanonicalMathProblem`

A problem (`compiled.problems` plus `variable_order`, `equation_order`, canonical `math_*`, `incidence`, `sparsity_patterns`, `variable_scales`, `equation_scales`, `kernel_bindings`, `solve_plans`) is what every backend consumes. Ordinals are assigned by sorting on semantic ID; positions in the solver vectors are `variable_order.position` and `equation_order.position`. Fixed and parameter symbols are not in the solver vector; their values are bound parameters of the evaluation program.

### 18.2 The native evaluation program

`pse-numerics` compiles the expression DAG into an instruction tape:

1. **Topological order** over the nodes reachable from active equations and objectives, with shared subexpressions computed once per evaluation (hash-consing guarantees sharing).
2. **Instructions** are `(opcode, output slot, input slots, payload)` over a contiguous `f64` workspace; `Affine` nodes become fused multiply-add sequences; reductions and gathers are unrolled because the mesh and domains are known.
3. **Residuals**: `r_e = body − (lower | upper)` for each equation position; inequalities and objectives are separate segments.
4. **Jacobian**: per-equation reverse-mode adjoint sweep over that equation's segment (equations in process models are small, so this is efficient and yields sparse rows directly); the sparsity pattern is exactly `compiled.incidence` and is verified at compile time. Kernel calls supply their Jacobians from `KernelSpec.derivatives`; implicit kernels apply the implicit function theorem on the selected branch.
5. **Hessian of the Lagrangian**: forward-over-reverse per equation with the multipliers, sparse pattern from the union of per-equation patterns; kernels contribute second derivatives when available (`num-dual` hyper-duals) and otherwise force the profile to `limited_memory`.
6. **Domain guards**: evaluation of a restricted operator outside its domain (log of a non-positive value, division by zero) raises a recoverable evaluation error carrying the equation and node; the Ipopt driver reports it as an evaluation failure (Ipopt then restores or fails), and the event is recorded in `runtime.solver_events` with the culprit, which is the information IDAES obtains only through `halt_on_ampl_error` and symbolic labels.
7. **Batch evaluation**: the same tape evaluates over Arrow columns of states (initialization guesses, sweeps, property tables) through the generated DataFusion wrappers.

Arrow buffers of parameter values are borrowed only when the array is contiguous (`ptr_offset() == 0`, one chunk) and null-free (`null_count() == 0`) — a sliced, filtered or nullable array is copied through `as_slice()` with the validity checked, never indexed from `data_ptr()` — and copied otherwise; the workspace is native. Implicit kernels obtain their derivatives from `num-dual`'s `implicit_derivative*` / `ImplicitDerivative` (the implicit function theorem is library capability, verified against a hand derivation in the kernel suites), not from platform code.

### 18.3 In-process Ipopt

> Decision: ADR-0028

`pse-backend-native` binds the Ipopt C API (`CreateIpoptProblem`, `IpoptSolve`, `AddIpoptNumOption`/`StrOption`/`IntOption`) with callbacks for `eval_f`, `eval_grad_f`, `eval_g`, `eval_jac_g`, `eval_h` over the evaluation program. Options come from `solver_profiles`; the IDAES defaults are shipped as profiles `ipopt.default` (`tol = 1e-6`, `max_iter = 200`, `nlp_scaling_method = gradient-based`) and `ipopt.user_scaled` (`nlp_scaling_method = user-scaling`, or pre-scaled problem with `nlp_scaling_method = none`), with `linear_solver` resolved when a run starts from `runtime.host_capabilities` (§6.13), which the explicit runtime operation `probe_host` writes (Ipopt version, available linear solvers, HSL, PETSc, the Python environment); a profile naming `ma57` on a host without HSL fails with `capability.backend` unless the profile declares `fallback_linear_solver = mumps`, in which case the fallback is a selected policy, and the options actually passed to `CreateIpoptProblem` are recorded in `runs.resolved_options` so that a reproduction on a different host is either identical or visibly different. Results: primal values (unscaled), duals, bound multipliers (`zL`, `zU`), termination status mapped to `TerminationStatus`, iteration statistics taken as data inside the intermediate callback through `GetIpoptCurrentIterate` and `GetIpoptCurrentViolations` (Ipopt ≥ 3.14; `probe_host` records the version and an older Ipopt degrades to parsing the solver's text output, recorded as such) into `runtime.iterations` (objective, primal and dual infeasibility, `mu`, step size, regularization, restoration flag). The `-sys` crate is generated with `bindgen` from `IpStdCInterface.h`, with the generated bindings committed and diffed in CI as §4.2 treats generated relation code; `index_style` is C (0) and asserted.

Interruption across the C boundary: the driver checks the cancellation token inside the intermediate callback and returns `false` to stop the solve. No Rust panic and no unwind ever crosses an Ipopt callback; a callback catches the failure, records it in the driver state, and returns `false`, and the driver converts the recorded failure into `solve.evaluation_error` or `runtime.cancelled` after `IpoptSolve` returns. The last iterate is written to `runtime.solutions` under the run's terminal status, so a cancelled or failed run is distinguished from a converged one by `runs.status`, never by the absence of rows.

### 18.4 The NL backend

`pse-backend-nl` writes the AMPL NL format directly from the problem (linear and nonlinear parts, common subexpressions as defined variables, bounds, objective, suffixes) and reads SOL files. It is the solver-neutral route and the only route to `ipopt_sens`, `k_aug`, `dot_sens`, `bonmin`, `couenne`, `cbc`, `clp`, `scip`, `petsc_snes`, and `petsc_ts`. Suffixes emitted: `scaling_factor` (when the profile asks for user scaling), `dae_suffix` and `dae_link` (PETSc), `dual`, `ipopt_zL_out`, `ipopt_zU_out` on read-back. Symbolic labels (`.row`, `.col` files) are always written so that solver messages and PETSc trajectories map back to semantic IDs.

Subprocess lifecycle: a solver process belongs to the run that started it. Cancellation or a wall-time limit kills the process group; a SOL file is ingested only when the process exited normally and the file's trailer is complete, otherwise it is discarded and the run ends `runtime.cancelled` or `solve.solver_error`. A partial SOL file is never parsed.

Kernels that are not expression-expandable reach NL solvers only through an AMPL external-function library. The platform exports one compiled library per kernel package (`funcadd` entry points generated from `KernelSpec`, including the IDAES-compatible symbols `cubic_root_l`, `cubic_root_h`, `cbrt`, and the Helmholtz function names) so that the same binaries IDAES uses work unchanged; `KernelSpec.bindings` must list `nl_external_function` for the binding to be valid, and P16 refuses otherwise.

### 18.5 Kernel adapters generated from `KernelSpec`

| Adapter | Generated form | Notes |
|---|---|---|
| scalar | `fn eval(inputs: &[f64], params: &Params) -> Result<Outputs, KernelFailure>` | used by the tape |
| dual / hyper-dual | the same body monomorphized over `num_dual::DualNum` types | derivative availability declared, not inferred |
| batch Arrow | `fn eval_batch(inputs: &[ArrayRef]) -> Result<Vec<ArrayRef>>` with null propagation per the strictness declaration | vectorized over rows; validity violations produce nulls plus a diagnostic column when requested |
| DataFusion `ScalarUDFImpl` | `name`, `signature` (exact Arrow types, dictionary preservation off), `coerce_types` = accept exactly the declared types, else error (no silent widening), `return_field_from_args` (receives the input `Field`s, so the output quantity type is derived from the inputs' `pse.semantic.quantity_type` metadata and attached with the extension name, which the registry then validates downstream), `is_strict` and `is_nullable` derived from `KernelSpec` strictness and validity — never written independently, because 55.0 uses UDF strictness for outer-join elimination, so a wrong declaration changes results — `volatility = Immutable`, `simplify` for all-constant inputs, `conditional_arguments`/`short_circuits` for conditional kernels (so a guarded branch is never evaluated eagerly and a domain guard never fires on a value the model excluded), `output_ordering`/`preserves_lex_ordering`/`strictly_order_preserving` for kernels declared monotone, `evaluate_bounds`/`propagate_constraints` derived from monotonicity and validity where declared; `config_options` is never read (a governance grep) | one registry per snapshot session; `number_rows` used for all-scalar invocations |
| NL external function | C ABI `funcadd` registration with argument count and derivative flags | shipped library |
| Pyomo binding | `ExternalFunction(library, function)` when the kernel is opaque; expression expansion when `pyomo_expression` is declared | §21 |

### 18.6 Solver-neutral results

Every backend writes the same relations (`runtime.runs`, `solutions`, `duals`, `residuals`, `iterations`, `solver_events`). Residuals are always recomputed natively from the returned primals so that the record does not depend on the solver's reporting.

### 18.7 Solve-plan selection

Static attributes only (semantic math basis §0.2): problem class from `math_static_analysis` and `problems.degrees_of_freedom`.

| Static signature | Solve plan | Backend and modifiers |
|---|---|---|
| `DOF = 0`, no objective, smooth | `SQUARE_NLE` | native Ipopt (or `petsc_snes` through NL); modifiers: continuation on failure |
| `DOF > 0`, objective, smooth, continuous | `NLP_LOCAL` | native Ipopt; multistart optional |
| discretized dynamic problem with DAE links, no objective | `DAE_INTEGRATE` or `SQUARE_NLE` on the fully discretized system | PETSc TS plan, `diffsol`, or Ipopt |
| integer or binary symbols (`number_of_units`) | `MINLP` | `bonmin`/`couenne`/`scip` through NL |
| alternative sets present | `GDP` | Pyomo adapter with Big-M, hull, or GDPopt (phase 3); native reformulation later |
| complementarity pairs with `formulation = smooth_eps` | as NLP | smooth reformulation already in the IR |
| `BLACK_BOX` equations without derivative contracts | none | `plan.no_feasible_solve_plan` |

The selected class, its justification, and the modifiers are rows in `compiled.solve_plans`.

### 18.8 Threading

One configuration owns the thread budget: the DataFusion `tokio` runtime and `target_partitions`, the `rayon` pool for batch kernels and pass parallelism, artifact hashing (`blake3::update_rayon`), diagnostics linear algebra (`faer`'s `Par`), and the solver's linear-algebra threads (HSL, MUMPS, PETSc). Defaults leave the solver single-threaded while a solve is running and give the pool to DataFusion otherwise; hashing and diagnostics never take pool threads during a solve; `target_partitions` is validated against the pool size when the session is built; oversubscription is a configuration error, not a runtime surprise.

### 18.9 Backend capability matrix

| Operator or feature | Native Ipopt | NL | Pyomo | DataFusion batch |
|---|---|---|---|---|
| all scalar operators, `Affine`, reductions, gathers | ✓ | ✓ | ✓ | ✓ |
| smooth approximations | ✓ | ✓ (expanded) | ✓ (expanded) | ✓ |
| `Conditional` with runtime guard | ✓ | ✗ | ✓ (`Expr_if`) | ✓ |
| `KernelCall` (expandable) | ✓ | ✓ | ✓ | ✓ |
| `KernelCall` (opaque) | ✓ | ✓ with external library | ✓ with `ExternalFunction` | ✓ |
| `ImplicitRef` | ✓ | ✗ (kept as equations) | ✗ (kept as equations) | ✓ |
| exact Hessian | ✓ (limited memory when a kernel lacks second derivatives) | solver-dependent | solver-dependent | — |
| integer variables | ✗ | ✓ (MILP/MINLP solvers) | ✓ | — |
| DAE suffixes | — | ✓ | ✓ | — |

---

## 19. Cases, results, and analytics

### 19.1 Cases and overlays

A case is a chain of overlays over a model revision. Precedence: the child's `case_specs` rows override the parent's for the same target; within a case, higher `priority` wins; targets are the identity-based `case_spec_targets` rows resolved at commit (§6.10), so a rename between two case revisions changes nothing; P13 expands each target to symbols, and a wildcard expands over the port's or group's members at that time. `treatment = fixed` with a value, `free`, or `parameter` decides `variable_order.treatment`; `initial` supplies guesses; bounds override template defaults. A specification that names a symbol with `role = expression` is an error (IDAES `VarLikeExpression` semantics: expressions cannot be fixed or bounded; use an equation instead).

### 19.2 Results and reporting

- **Stream tables.** A view joining `inferred.ports` on connections (streams named by connection, plus additional ports for feeds and products), the state's display declaration (or the state symbols when `true_state`), `runtime.solutions`, and the reporting unit set (IDAES `reporting_units` config): output columns are streams, rows are display quantities with a units column, missing rows shown as `-`. The interactive variant adds the variable kind (`fixed`, `unfixed`, `parameter`, `expression`) per cell.
- **Unit reports.** `template_display.performance` rows per instance joined to solutions; the IDAES `report()` layout is a projection.
- **KPI tables.** `generate_table` is a DataFrame query over any attribute paths.
- **Tags.** `authored.tags` (`name`, `expression`, `display_unit_id`, `format`, `doc`, `group`) with group defaults (`str_include_units`, `set_in_display_units`); reading a tag converts to the display unit; setting a tag writes a `case_specs` row (in display units when the group says so). Tag tables (`table_heading`, `table_row`) and SVG label substitution are projections.
- **Model statistics** and diagnostics reports are the views of §15.
- **Float semantics in analytics.** `GROUP BY`, `DISTINCT` and joins on a `Float64` column merge `-0.0` with `+0.0` and treat NaN as self-equal (§5.3); ordering does not. Analytics over `runtime` values that group on a float record this as a selected loss in the report's metadata; the platform's own views never group on floats.

### 19.3 Sweeps and convergence studies

`authored.case_sets` hold a generator (`grid`, `latin_hypercube`, `uniform_random`, explicit `list`; the IDAES PySMO sampling methods map to these) and produce sample cases; a runner (`sequential`, `parallel` over the pool) executes each sample with `rebuild_model` semantics replaced by immutable snapshots, records per-sample outputs and solver statistics, and stores results in `runtime.sweep_results`. Baseline comparison and the convergence-summary report (successes, failures, restoration, regularization, numerical issues) are queries.

### 19.4 Parameter estimation and data reconciliation

`observations` and `measurement_models` bind measured values (with `std_dev`) to model expressions. The reference problem templates:

- `problem.weighted_least_squares@1`: objective `Σ_k ((expr_k − obs_k)/σ_k)²` over the bound observations; estimated parameters are symbols whose treatment is switched from `parameter` to `free` by the case; ports left free where reconciliation requires it.
- `problem.multi_scenario_estimation@1`: stacks one problem instance per scenario (shared θ symbols, scenario-local everything else) into one problem via `scenarios`; this is the parmest experiment model expressed as data.

Covariance and confidence regions use reduced-Hessian sensitivities from `k_aug`/`sIpopt` through the NL backend or the native Hessian. Until native estimation is complete, the Pyomo adapter exposes the stacked problem to `parmest`.

### 19.5 Costing

The costing framework (`core/base/costing_base.py`, `models/costing/SSLW.py`) is templates and reference data:

- `costing.flowsheet@1` (IDAES `FlowsheetCostingBlock`): parameters `base_currency` (a currency unit), `base_period` (default year); flow types registered as `flow_types` rows with a fixed cost symbol `<flow>_cost` per type; aggregate symbols and equations `aggregate_capital_cost = Σ_u convert(u.capital_cost)`, `aggregate_fixed_operating_cost`, `aggregate_variable_operating_cost` (per period), `aggregate_flow_<f> = Σ convert(flows)`, `aggregate_flow_costs[f] = convert(aggregate_flow_f · f_cost)`, only for flow types with at least one registered flow. Plug-in ordering registers each unit costing instance in the unit's initialization order.
- `costing_method` templates (IDAES `UnitModelCostingBlock` methods): applied to a unit instance, creating `capital_cost`, `fixed_operating_cost`, `variable_operating_cost` symbols (each must be non-negative; the compiler warns otherwise) and their constraints; `unit_mapping` is a relation `costing_method_defaults(unit_template_id, method_template_id)` with inheritance-aware lookup.
- **Currency.** `USD_CE500` is the base currency unit (dimension `currency`); `USD_CE394 = (500/394)·USD_CE500`; `USD_<year> = (500/CE(year))·USD_CE500` for 1990 onward from `reference.cost_indices`; location factors are reference data.
- **SSLW** methods (`cost_heat_exchanger`, `cost_vessel` with platforms, ladders, and trays, `cost_fired_heater`, `cost_compressor`, `cost_fan`, `cost_blower`, `cost_turbine`, `cost_pump` with motor, and the `cost_pressure_changer` dispatcher) are templates with their enumerations (`HXType`, `HXMaterial`, `HXTubeLength`, `VesselMaterial`, `TrayType`, `TrayMaterial`, `HeaterMaterial`, `HeaterSource`, `CompressorType`, `CompressorDriveType`, `CompressorMaterial`, `PumpMaterial`, `PumpType`, `PumpMotorType`, `FanType`, `FanMaterial`, `BlowerType`, `BlowerMaterial`) as reference enums, the correlation coefficients as `parameter_values`, and the cost-basis quantities (area in ft², work in hp, pressure in psig, duty in BTU/hr, flow in gpm or ft³/min) as `UnitConvert` nodes. `number_of_units` is an integer symbol (`solver_type = integer` when the `integer` option is set, otherwise continuous and fixed to 1). Two IDAES defects noted by the survey (the vertical and horizontal vessel wrappers discarding their arguments; the blower dispatch routing to the pump method) are corrected in the reference package with parity notes.
- Costing initialization is the plan stage "solve each cost symbol from its defining constraint" (1×1 blocks).

### 19.6 Utility minimization (Duran–Grossmann)

`law.heat_integration@1` takes the heating and cooling unit instances, `DTmin`, and `eps`, and generates: `Tin[i]`, `Tout[i]` expressions from the units' control volumes; `Theta[i]` (the smoothed `Q/ΔT`); pinch candidates `T_[p] = Tin[p] + dT[p]` (`dT = DTmin` for heating units, 0 for cooling); `QAh[p]`, `QAc[p]` (bounded below by `1e-8`) with the smooth-max heat-content equations; `Qs`, `Qw`; `heating_utility[p]: Qs ≥ QAc[p] − QAh[p]`; `cooling_utility: Qw = −Σ_i Q_i + Qs`. Initial values come from the pinch calculation kernel. Composite curves are an analytics query.

### 19.7 Optionality

`math_alternative_sets` (installed or absent, mode selection, technology choice) lower to disjunctions. Phase 3 lowers through the Pyomo adapter (Big-M, hull, GDPopt); native reformulation to indicator constraints or binaries through the NL backend follows.

### 19.8 Uncertainty

First-order propagation (IDAES `uncertainty_propagation`) is sensitivities `ds/dp` from `k_aug`/`sIpopt` (NL) or the native reduced Hessian, combined with the parameter covariance from estimation: `σ²_f = (df/dθ)Σ_θ(df/dθ)ᵀ`. Robust optimization (PyROS) is reached through the Pyomo adapter.

---

## 20. Persistence, provenance, and reproducibility

### 20.1 Artifact store

```text
<store root>/
  refs/<name>.json                              -- {snapshot_id, updated_at}; updated with a conditional put
  snapshots/<snapshot_id>/manifest.json         -- see below
  relations/<namespace>/<relation>@<version>/<content_hash>.arrow    -- compiled/runtime hot artifacts (IPC file, mmap-able)
  relations/<namespace>/<relation>@<version>/<content_hash>.parquet  -- authored/reference/runtime durable artifacts
  kernels/<kernel_id>@<version>/<digest>/       -- compiled kernel libraries (native, NL external function)
  documents/<document_id>/<content_hash>        -- authored source documents
```

`object_store` provides atomic single-object writes and conditional updates (`PutMode::Update` with the ref's version); the multi-relation snapshot protocol is platform code: write every relation artifact (idempotent by hash), write the manifest, conditionally update the ref. Readers pin a manifest. Every artifact write hashes the bytes actually serialized and verifies them against the target name before the put (a `StreamWriter` dropped without `finish()` yields a truncated stream with no error at the call site); a mismatch is `runtime.infrastructure` and nothing is written. The final path is written once — artifacts with `PutMode::Create` (an `AlreadyExists` result is success, because the content hash is the name), the ref with `PutMode::Update` and the previous version — and no copy-then-delete sequence is ever a commit; at `object_store` 0.13.2 the trait exposes `*_opts` methods only, and `rename_opts` is not used. The local single-writer mode is the initial implementation.

### 20.2 Manifest

```json
{
  "snapshot_id": "blake3:…",
  "created_at": "…",
  "schema_registry_fingerprint": "blake3:…",
  "relations": [ {"namespace": "authored", "name": "stoichiometry", "version": 1, "content_hash": "blake3:…", "rows": 12, "path": "…"} ],
  "packages": [ {"package_id": "…", "version": "1.2.0", "content_hash": "…"} ],
  "compiler": {"version": "…", "passes": [ {"pass_id": "…", "version": "…"} ]},
  "engine_profile": {"engine_profile_id": "…", "content_hash": "blake3:…"},
  "toolchain": {"lockfile_hash": "blake3:…", "canonicalization": "pse.canon.v1", "plan_fingerprint_contract": "pse.planfp.v1"},
  "kernels": [ {"kernel_id": "…", "version": "…", "digest": "…"} ],
  "parents": ["blake3:…"],
  "closure_report": "blake3:…"
}
```

### 20.3 What a run references

`runtime.runs.environment`, `runs.resolved_options`, `runtime.host_capabilities` and the run's manifest capture: model and case revisions; property, reaction, and template package versions; compiler and pass versions; kernel digests; discretization, scaling, and initialization plan artifacts; solver profile, the options actually used, and backend versions (Ipopt, HSL, PETSc; Pyomo, pint and every `pyomo.contrib.*` package used, numpy and scipy versions, and the IDAES version for parity runs); host and thread configuration; the lockfile hash; result and diagnostic artifact hashes. The questions in the proposal (§10.2) are joins: "which correlation produced this enthalpy term" walks `math_equations → derivations → method_resolutions → method_specs`.

### 20.4 Reproduction

`pse reproduce <run_id>` loads the manifest, verifies every artifact hash, re-executes the pass DAG (memoized, so only missing artifacts are rebuilt), and re-runs the solve with the recorded profile. Byte-identical relation hashes are asserted for every deterministic pass; solver outputs are compared within the profile's tolerance.

### 20.5 Compatibility and migration

Relation versions are explicit; loaders apply generated migrations forward; a snapshot whose schema registry fingerprint is unknown is rejected with `schema.unknown_registry`. IDAES JSON state files (`to_json` format version 4) and scaling JSON files are importable through adapters that map local names to semantic IDs by qualified path.

---

## 21. The Python boundary and the Pyomo adapter

### 21.1 Extension module

> Decision: ADR-0024

`pse-py` (pyo3 0.29, pyo3-arrow 0.19) exposes a small API; every table crosses as an Arrow C stream (`__arrow_c_stream__`) that the consumer drains as a `RecordBatchReader`, never as row objects and never as a materialized `Table` on both sides at once. The capsule protocol, not `pyarrow`, is the contract: any Arrow implementation (pyarrow, polars, datafusion-python at whatever version) can consume it, and none of the typed classes of §21.5 names a `pyarrow` type.

```python
snap = pse.open(store_path).head()                      # SnapshotHandle
snap.table("compiled", "math_equations")                 # object exposing __arrow_c_stream__ → pyarrow / polars / datafusion-python
problem = pse.compile(snap, case="base")                 # ProblemHandle
bundle  = problem.bundle()                               # ProblemBundle: dict of named Arrow streams + manifest
result  = pse.solve(problem, profile="ipopt.default")    # RunHandle with .tables()
```

Bundle contents: `manifest` (problem id, ordering hashes, unit set, and the **per-column loss profile**: for every table and field, whether a `pse.*` extension type is carried and whether the consumer registered it — the engine errors on an unregistered extension name while Python degrades to the storage type, so the declaration is asymmetric and says which side enforces what), `variables` (ordinal, semantic id, qualified name, `quantity_type_id`, `unit_id`, bounds, initial, treatment, scale), `equations` (ordinal, sense, body node, bounds, scale, active, `residual_quantity_type_id`), `objectives`, `expr_nodes`, `expr_args`, the typed payload tables, `kernel_bindings` with their Pyomo binding descriptors, `source_map` (ordinal → qualified name and source span). Transfer is one stream per table, not per-node calls.

### 21.2 Adapter algorithm (Python)

1. Pre-flight: `SolverFactory(name).available(False)` for the profile's solver and, for every kernel bound as `pyomo_external_function`, that the exported library resolves on this host — both before any model object is built; a failure is `capability.backend` naming the solver or kernel id (Pyomo itself only warns when an `ExternalFunction` library is missing and fails later from the solver). Create `ConcreteModel`; one `Var` per free variable (indexed by ordinal), one `Param(mutable=True)` per parameter (a generated-code invariant, not a convention), bounds and initial values from the bundle; units attached from the bundle's unit set through `pyomo.units`, which is backed by `pint`. `pyomo.util.check_units.identify_inconsistent_units` (not a bare `assert_units_consistent`) runs on the built model and names the offending components in a `compile.math` finding; the registry is the authority for every unit — pint validates what the adapter built and, if the two disagree, the adapter has a bug. No relation is ever derived from a pint object.
2. Lower nodes in topological order with memoization: `Affine` → `LinearExpression(constant, linear_coefs, linear_vars)`, asserting `polynomial_degree() == 1` afterwards (a generated per-node equivalence check between the IR's classification and Pyomo's); `Mul`/`Div`/`Pow`/functions → Pyomo operators; smooth operators → their expanded forms; reductions → `sum(...)`; `Conditional` → `Expr_if`; `KernelCall` → expanded expression when `pyomo_expression` is bound, otherwise `ExternalFunction(library, function)` from the exported library; `ImplicitRef` never appears (implicit systems stay as equations).
3. Constraints and objectives from the equation rows; `scaling_factor` suffix from the bundle when the profile uses user scaling.
4. Solve through `SolverFactory` with the profile's options; results (values, `dual`, `ipopt_zL_out`, `ipopt_zU_out`, termination) are returned as Arrow tables keyed by ordinal; residuals are recomputed natively when ingested. The versions of Pyomo, pint and every `pyomo.contrib.*` package touched are recorded in `runs.environment`.

The adapter builds no IDAES classes. Engineering names live in the source map.

### 21.3 Uses of the adapter

Parity testing (§24), and the Pyomo ecosystem: `parmest`, `PyROS`, `GDPopt`, `pyomo.dae` utilities, `pyomo.contrib.iis` as a cross-check oracle for §15.5's minimal intractable system, and `IncidenceGraphInterface`-based cross-checks of the native structural results, run as standing CI parity tests over the golden snapshots. Five of these six live in `pyomo.contrib`, the least stable surface in the dependency set; §18.9's matrix records that standing, and every result obtained through one records the package version (§20.3).

### 21.4 Opaque kernels and ASL

A kernel bound as `pyomo_external_function` requires the exported AMPL external-function library; the same library serves the NL backend. Python callback functions are not used for solves because ASL-based solvers cannot execute them. Every kernel with this binding has an end-to-end test that solves a tiny problem through Ipopt from Pyomo and compares to the native result.

### 21.5 Python contracts

The Python package follows the contract discipline (formerly `docs/library_ref/contract_substrate_discipline.md`, now stated here so the reference cannot dangle):

- **Contract classes are generated.** The attrs classes for every relation and bundle table are emitted by the §4.2 generator from `RelationSpec` — the same projection as the Rust views, in another language — and shipped in `python/pse/contracts/`. A hand-written class that mirrors a relation's column list is the governance failure §4.2 names; the `no_shadow_structs` check covers Python too.
- **Two class systems, one contract each.** `msgspec.Struct(frozen=True, forbid_unknown_fields=True)` with `Meta` constraints is the codec for the §20.2 manifest and for every JSON or TOML file the Python package reads or writes (strict decode, located errors, JSON Schema emitted only for these wire types — never for authoring documents, whose schema the Rust registry generates). `attrs` + `cattrs` structure Arrow rows (already-decoded material) into the generated classes. No type is defined in both systems.
- **Strict structuring.** Every converter is built with the hook factory `register_structure_hook_factory(has, …make_dict_structure_fn(…, _cattrs_forbid_extra_keys=True))` so an unknown key raises `ForbiddenExtraKeysError` (cattrs ignores extra keys by default, which would turn a version drift between Rust and Python into silent data loss); `detailed_validation` is never disabled; `ClassValidationError` groups are rendered with `cattrs.transform_error` into §23.2 findings rather than reduced to their first message. Classes are `attrs.frozen`.
- **No `Any`.** A governance lint walks `attrs.fields()` of every contract class at import time and fails on `Any`, bare `dict` or bare `list`; nothing at runtime forbids them otherwise.
- **Extension types.** The generated `pyarrow.ExtensionType` classes of §4.4 are registered once, idempotently, when the package is imported; the contract layer asserts per column that every field claiming a `pse.*` extension resolved to a registered class, using the `ARROW:extension:name` key that survives even when the type does not.
- **No Python-side model classes** duplicate relations; engineering names live in the source map.

### 21.6 The array boundary (numpy)

A nullable Arrow column converted to a bare `ndarray` turns null into NaN, collapsing the distinction §4.4 exists to protect. The adapter therefore converts with `zero_copy_only=True` by default (which refuses nulls and chunked columns rather than copying silently), asserts `null_count == 0` before any conversion (true for solver vectors by construction), and carries the validity mask alongside where a nullable column must cross. A copy is requested explicitly (`zero_copy_only=False`) only where the call site records why. `-0.0` survives the conversion. `numpy` and `scipy` are pinned because the parity harness compares numbers through them; neither is used on a production path.

---

## 22. Authoring and the extension model

### 22.1 Package layout

```text
<package>/
  package.toml            -- id, version, kind, dependencies
  materials/*.yaml        -- species, phases, elements, reactions, stoichiometry, parameter values
  methods/*.yaml          -- method_specs for equation-template methods (kernels are Rust and only referenced)
  properties/*.yaml       -- property_packages, method_selections, state bounds, equilibrium pairs
  templates/*.yaml        -- unit, control volume, state, reaction, connection rule, helper, initializer, scaler templates
  laws/*.yaml             -- law templates and law instances
  costing/*.yaml          -- costing methods and reference cost data
  cases/*.yaml            -- instances, connections, cases, observations, case sets
  assertions/*.yaml       -- expected derived facts (tests, not truth)
```

P1 parses every document into `authored` rows via a change set; the document's content hash is recorded on each row's source span. YAML documents are parsed by `serde-saphyr` (typed errors with line and column, hostile input refused without a panic, a parsing `budget` on nesting, aliases and allocation) and TOML by `toml` with `Spanned<T>`, so every row's `pse.source_span` comes from the parser. Every entity declared under `id_policy = explicit` carries an `id:` field; `pse authoring assign-ids` inserts missing ones and P1 rejects a document that still lacks one (§5.1). YAML is one surface; the same change sets can be produced by Python builders (`pse.authoring`) and by agents.

### 22.2 The change-set model

> Decision: ADR-0027

```text
authored.change_sets @1     change_set_id, base_revision_id, author, message, created_at
authored.change_ops @1      change_set_id, ordinal, op : enum (insert|update|delete|rename), relation_id, row_key, row (Struct), precondition : text [n]
                            -- rename: row carries entity_id and the new name/qualified_name; rejected for named-policy entities (§5.1)
```

A change set is validated (P2 on the resulting snapshot), applied atomically, and yields a new model or case revision. **The commit contract is P2-level validity**: a committed revision is structurally and referentially valid (every invariant of `reference.schema_invariants` holds, every target resolves to an identity, every expression parses); it may still fail P3–P10 (an unresolvable template path, a unit inconsistency). Publication as a *compiled problem* is gated separately by the closure report (§14.5), and every revision carries `last_closure_status` so a reader can tell a committed revision from a compilable one. There is no other write path into `authored` (decision D2 and doctrine P20). Attempts to write a derived relation are rejected; an "expected" derived fact goes to `provenance.assertions`.

`rename` changes an entity's `name` and `qualified_name` attributes and re-renders every retained `pse.target_path` and `pse.expr_dsl` text that serialized a reference to the old path from the identity-based rows (`case_spec_targets` and kin, `normalized.*_expr_*`); references by identity are untouched, every derived ID is unchanged, and the §22.4 diff report shows one rename row. A change op that edits a retained target text without the corresponding identity rows is rejected by P2 (`case.target_text_mismatch`): the text is a serialization, not a second authority.

### 22.3 What is data and what is code

| Extension | Realized as | Code change needed |
|---|---|---|
| New unit model | template document | no |
| New pure-component correlation, EOS variant, phase-equilibrium form, reaction form | method document with an expression template | no |
| New costing method or costing library | costing template and reference data | no |
| New initializer or scaler strategy | plan template composed from stage kinds | no, unless a new stage kind is required |
| New diagnostic check expressible over relations | a `RulePlanSpec` and thresholds | no |
| New material system, property package, flowsheet, case | documents | no |
| New constitutive computation that cannot be expressed as an expression (root finders, tabulated data, external libraries) | `KernelSpec` plus Rust implementation and tests | yes, in `pse-kernels` or a provider crate |
| New operator (a new universal computational primitive) | opcode with a full contract in `pse-mathir` and all backend lowerings | yes |
| New law family | law template if expressible over contributions; otherwise a compiler rule | usually no |

This is the litmus test of the doctrine: new scope enters as semantics and plug-ins, not as runtime branches.

### 22.4 Agent change sets

An agent proposes a change set (for example: replace the heat-transfer correlation on one exchanger, regenerate, compare). The platform compiles the resulting snapshot, produces a diff report (added and removed equations, changed sparsity, changed scaling, changed DOF, closure report), and the change is accepted or rejected as a whole. Agents never edit `build()`-style code.

---

## 23. Observability and failure semantics

### 23.1 Observability

`tracing` spans are opened per pass run, per rule evaluation, per plan stage, and per solve; events carry the semantic IDs involved. Fields that are known only at the end of a span (output hashes, row counts, finding counts, cache hit or miss) are declared as `field::Empty` when the span opens, because a later `record` of an undeclared field is silently discarded; `#[instrument]` is used with `skip(...)` and explicit `fields(...)`, never with its default of recording every argument. DataFusion emits no spans of its own: rule-level spans are platform code around plan execution, operator-level visibility comes from the `EXPLAIN` `pgjson` output and `rules_fired` stored in the pass record (§14.2 rule 5), and `JoinSetTracer` may be installed for per-task tracing when needed (`datafusion-tracing` is adopted only when a release matching the pinned engine exists). Solver output is captured as data through the Ipopt iterate API (§18.3) with text capture only for messages that have no structured equivalent. Metrics (pass durations, row counts, cache hits, solver iterations) are emitted with the same identifiers. Provenance (`provenance.*`) explains derivation; observability explains execution; they are linked by pass run and run IDs but never merged.

### 23.2 Failure taxonomy

| Failure class | Examples | IDAES counterpart |
|---|---|---|
| `authoring.parse`, `authoring.reference` | syntax error, unknown path | `ConfigurationError` |
| `validation.invariant` | foreign key missing, cardinality, domain | `ConfigurationError`, `PropertyPackageError` |
| `compile.feature` | `dynamic` in a steady-state parent, holdup without dynamics | `DynamicError`, `ConfigurationError` |
| `compile.property` | unsupported or ambiguous property, unresolved method | `PropertyNotSupportedError`, `PropertyPackageError` |
| `compile.law` | unsupported balance binding, basis conversion missing | `BalanceTypeNotSupportedError` |
| `compile.math` | unit inconsistency, cyclic expression | Pyomo `UnitsError` |
| `compile.discretization` | mixed derivative, missing policy | `NotImplementedError` |
| `capability.backend` | unsupported opcode, missing derivative, missing external library | `PropertyPackageError`, ASL errors |
| `plan.initialization` | DOF ≠ 0, structural singularity, postcheck failure | `InitializationError` with `InitializationStatus` |
| `solve.infeasible`, `solve.locally_infeasible`, `solve.unbounded`, `solve.limit`, `solve.evaluation_error`, `solve.solver_error` | termination statuses | Pyomo `TerminationCondition` |
| `runtime.cancelled`, `runtime.timeout`, `runtime.infrastructure` | cancellation token, wall limit, store I/O | — |
| `internal.invariant` | a pass postcondition failed | `BurntToast` |
| `user.model` | authored assertion failed, user equation error | `UserModelError` |

Failures carry the relation rows and source spans involved; they are never flattened into "run failed". Every `pse-*` crate returns concrete error enums (`thiserror` with `miette::Diagnostic` derived, the class names above as `#[diagnostic(code(...))]` in Rust path form); `miette::Result` and the graphical reporter exist only in the CLI and the driver. Findings are relations; a rendered diagnostic is a projection of a finding, never its storage. `pse.*` values inside a diagnostic are rendered by the formatter factory of §4.4, not as raw storage bytes.

DataFusion errors map into the taxonomy at one place, never per call site:

| `DataFusionError` variant | Platform class |
|---|---|
| `ResourcesExhausted` | `runtime.infrastructure` (recoverable; the message names the configuration keys) |
| `SchemaError`, `Plan` from a rule-compiler input | `internal.invariant` (a rule plan that does not type is a platform bug) |
| `SchemaError`, `Plan` from an analytics query | `user.model` |
| `Execution` inside a generated kernel UDF | `solve.evaluation_error` or `compile.property`, from the kernel's declared failure class |
| `Execution` elsewhere, `ArrowError`, `ParquetError`, `ObjectStore`, `IoError` | `runtime.infrastructure` |
| `Configuration` | `config.invalid` (typed; `SessionConfig::set_str`, which panics instead, is banned) |
| `Diagnostic`, `Collection` | unpacked into one finding per contained error (validators return violating keys plural; the first error is never the only one reported) |
| `NotImplemented`, `Substrait`, `Ffi`, `External`, `Context`, `Shared`, `SQL`, `ExecutionJoin` | `internal.invariant` unless wrapped by a platform class |

---

## 24. Testing and acceptance

### 24.1 Test layers

| Layer | What it proves | Mechanism |
|---|---|---|
| Schema governance | every relation is registered; no hand-written struct shadows a relation; every invariant has a positive and a negative fixture; generated code is current; no `authored` or `reference` column has a foreign key into `compiled` or `runtime`; no `authored` relation stores a parsed expression graph (`producing_pass_id` is null on every authored relation); every crate in `Cargo.toml` and every Python library appears in §3.1's tables; the `TableProvider` implementations expose exactly the read-path method set; governance greps ban `Field::extension_type()`, `SessionConfig::set_str`, `SchemaLike::from_type`/`from_samples`, `config_options` reads in kernels, and `SERDE_ARROW:*` keys; the Python `Any` lint and `no_shadow_structs` cover the generated contract classes; the `schema_enums` members match IDAES 2.12.0's enum classes | `tests/governance/*`, CI diff of generated sources |
| Structural validity | the test and CI cargo profiles enable Arrow's `force_validate`, so every array any builder constructs is fully validated across the whole suite | cargo profile |
| Engine reproducibility | the same rule plan encoded in two fresh processes yields byte-identical `datafusion-proto` bytes; adding a rule to the engine profile or changing an allow-listed setting misses every rule-pass memo; a batch with a `pse.*` extension over the wrong storage type is rejected at planning; a `distinct` or join on a `Float64` column is rejected by the rule compiler | `tests/engine/` |
| Pushdown truthfulness | the wrapper provider re-applies every `Exact` filter over the golden snapshots and finds no survivor; an `IN` over a key column reaches the scan as `Exact` (matched as an `OR` chain) | `tests/engine/` |
| Python boundary | an extra key in a bundle row raises `ForbiddenExtraKeysError`; an `Any` field fails the lint; the ten extension types survive Rust → Python → Rust with registration and are reported degraded without it; a nullable column refuses `ndarray` conversion; a missing external-function library or solver raises `capability.backend` before model construction; the manifest rejects an unknown registry fingerprint | `python/pse/tests` |
| Cross-check oracles | matching cardinality and structural rank agree with `scipy.sparse.csgraph` and `IncidenceGraphInterface`; the native minimal intractable system agrees with `pyomo.contrib.iis`; the smallest singular values agree with `scipy.sparse.linalg.svds` on the dense-tractable fixtures — invariants compared, never object identity | parity harness |
| Supply chain | `cargo deny` (advisories, bans, maintenance) and `cargo audit` are green; `cargo tree` shows one version per family; the committed lockfile matches the manifest | CI |
| Canonicalization | identical semantics under renaming, row reordering, and re-batching hash identically; unit inference rejects every dimensionally invalid form and accepts the reference library unchanged (`P/(R·T)`, `T/1000{K}` and `exp(−E/(R·T))` type without wrapping; `T_out − T_in` is a difference; `T + ΔT` is a point; `T1 + T2` is rejected unless the node is `weighted_mean`; a `25{degC}` literal reaches P10 in K); affine normalization is idempotent; parse → render → parse is the identity on every expression graph | property-based tests (`proptest`) over generated expressions |
| Rule engine | rules are order-independent and reach the same fixed point; stratified negation respected; four-valued outcomes recorded | golden inferred relations per fixture package |
| Kernels | values match IDAES or literature at test points; first and second derivatives match finite differences and dual-number results; validity violations fail as declared; batch and scalar paths agree; NL external functions agree with native | per-kernel test suites referenced from `KernelSpec.test_suite` |
| Structural algorithms | matching, DM partition, SCC order, and connected components agree with reference fixtures (generated once from Pyomo incidence analysis on the IDAES tutorials) | fixture files under `tests/structural/` |
| Backend conformance | native residuals and Jacobians equal Pyomo residuals and Jacobians (PyNumero) at random valid points to `1e-10` relative; NL round trip solves with Ipopt to the same solution as native; SOL parsing | parity harness in `python/pse/parity` |
| Diagnostics parity | the IDAES diagnostics tutorial models produce the same warnings and cautions | fixtures from `idaes-examples` diagnostics notebooks |
| Initialization and scaling | plans reach the same converged points as IDAES initializers on the reference units; scaling factors equal IDAES's for the reference scalers | parity fixtures |
| Incrementality | a value change recomputes only P13 onward; a `treatment = parameter` value change recomputes only the P14 rows naming it; the P10 hash is identical across cases that differ only in parameter treatment; changing the engine profile (rule list, semantic setting or DataFusion version) misses every rule-pass memo; a feature change recomputes only the affected subtree; artifact hashes of untouched outputs are unchanged | pass-record assertions |
| Provenance | every equation, symbol, and inferred row has a derivation chain to authored rows; the negative-completeness report lists excluded candidates | closure tests |
| Doctrine conformance | the metamodel conformance tests (automatic participation, material change, scope change, internal cancellation, unknown composition, rename and reorder, redundant relation rejection, domain replacement) pass on the process packages; rename in particular: after renaming a unit only `name`/`qualified_name` columns differ, every derived ID, case binding and derivation join is unchanged, and the diff report contains exactly one rename row | `tests/conformance/` |
| Lifecycle | a truncated or partially written artifact under a correct-looking name is rejected before the put; a ref CAS conflict retries and never publishes a mixed snapshot; cancellation mid-pass leaves no partial artifact; cancellation mid-solve returns `false` from the Ipopt callback without unwinding; an NL solver process killed mid-run leaves no ingested SOL file; a retried stage is a new `runs` row with `attempt + 1`; two change sets on one base revision conflict explicitly; a snapshot with an unknown engine profile is recompiled, never served from the memo | `tests/lifecycle/` |

### 24.2 Vertical slices and acceptance criteria

**Slice A — steady-state heater and mixer (phase 1).** Benzene–toluene ideal package with `FTPx`, feed, heater, mixer, product; equality connections; case fixing feed state and heater duty, and a second case fixing outlet temperature instead. Accept when: the compiled problem has the same variable and equation counts and DOF as the IDAES model; native and Pyomo residuals and Jacobians agree; Ipopt converges to IDAES values within `1e-6` relative; the stream table matches; reload from the artifact store reproduces identical hashes; the block-triangularization plan initializes both cases; renaming the heater between the two cases changes no derived ID and no case binding.

**Slice B — recycle with flash (phase 2).** Peng–Robinson package, `SmoothVLE`, `LogBubbleDew`, flash, separator, compressor (isentropic), heat exchanger (LMTD), recycle with tear stream; modular-properties initialization plan; full diagnostics including SVD and Degeneracy Hunter on a deliberately degenerate variant. Accept when: cubic root kernel derivatives agree with finite differences and the NL external function; the sequential-modular plan converges the recycle; structural and numerical reports match IDAES on the same flowsheet; homotopy moves the feed specification across a phase boundary with the IDAES step statistics; a value-only case change recompiles only P13 onward.

**Slice C — distributed dynamic reactor with costing and estimation (phase 3).** CSTR and PFR (backward finite difference and Radau collocation) with a generic reaction package (Arrhenius, power law), dynamic flowsheet with time discretization, PID controller, SSLW costing, utility minimization, a parameter-estimation case over observations, and a parameter sweep. Accept when: discretization equations equal Pyomo DAE's; the time-element plan and the PETSc plan reproduce IDAES trajectories; costing aggregates match SSLW values; estimated parameters and covariance match parmest; sweep statistics match the IDAES convergence analysis format.

### 24.3 Benchmarks

Measured separately and reported per snapshot: authoring parse, compilation (per pass, including rule-plan optimization time and the extension-registry overhead during planning), provenance size (derivation rows and bytes per equation, by relation, to validate the `row`/`rule` granularity assignment of §14.2 rule 4), Python materialization (bundle transfer and Pyomo construction), residual evaluation, Jacobian evaluation, Hessian evaluation, solve time, and total wall time, for each slice at increasing mesh and component counts. No claim of solve-time improvement is made before measurement; the expected early gains are less repeated construction, better inspection, and reliable reuse.

---

## 25. Delivery phases

| Phase | Scope | Exit criteria |
|---|---|---|
| 0 — Foundations | `pse-schema` registry and codegen; `pse-ids`; `pse-quantity`; `pse-material`; `pse-mathir` with the operator catalog and canonicalization; `pse-catalog` over a local store; `pse-authoring` parser and change sets; passes P0–P3 and P10 | schema governance tests green; expression DSL round-trips; canonical hashing invariant under reordering; a snapshot can be published, read back, and queried through DataFusion |
| 1 — Slice A | reference packages: units, elements, ideal EOS, NIST/RPP/Perry's methods, `FTPx`/`FcTP`, lumped control volume, heater, feed, product, mixer, state junction, equality connections; passes P4–P9, P12–P16; native evaluation program with Jacobian; Ipopt FFI; NL writer and SOL reader; Pyomo adapter; structural diagnostics; nominal scaling; block-triangularization and single-control-volume plans | slice A acceptance |
| 2 — Slice B | cubic EOS with the root kernel and external library; smooth VLE, bubble and dew; flash, separator, pressure changers, valve, 0D heat exchangers; translator; topology closure with tear selection; modular-properties, mixer, separator, HX, isentropic plans; sequential-modular plan; homotopy; full numerical diagnostics, SVD, Degeneracy Hunter, MIS; autoscaler and scaling profiler; exact Hessian | slice B acceptance |
| 3 — Slice C | distributed control volume, PFR, pipe, 1D exchangers; discretization lowering; dynamic flowsheets, time-element and PETSc plans; PID controller; generic reaction package; costing framework and SSLW; utility minimization; observations, estimation, reconciliation; sweeps and convergence analysis; GDP through the Pyomo adapter; uncertainty propagation | slice C acceptance |
| 4 — Breadth | electrolytes (aqueous phase, eNRTL, true and apparent species), Helmholtz templates and native state kernel, CoolProp parameter import, FeOs provider (conditional on `feos-core` tracking `num-dual` 0.15), MSContactor, solid–liquid units, NTU and lumped-capacitance exchangers, shell-and-tube, `diffsol` backend, optional `egglog` rewrites, native GDP lowering | parity on the corresponding IDAES tests |

Each phase ends with the parity harness green against IDAES 2.12.0 (the pinned parity package, §3.1) for its scope and with the doctrine conformance tests passing.

---

## 26. Risks and open decisions

| Risk or question | Position taken here | Fallback |
|---|---|---|
| Exact Hessian cost for large distributed models | per-equation forward-over-reverse with sparsity from incidence; kernels declare second derivatives | Ipopt limited-memory Hessian profile |
| External-function libraries must be built per platform for NL solvers | one generated library per kernel package with the IDAES symbol names, built in CI for Linux, macOS, Windows | native Ipopt path needs no library |
| Pyomo construction cost dominates parity runs | the adapter is coarse-grained and memoized; used for parity and ecosystem tools only | none needed for production solves |
| Floating-point disagreement with IDAES from summation order | parity tolerances are explicit; canonical term ordering is deterministic | record IDAES's order in a test-only policy |
| Canonical hashing of floats | IEEE bits with NaN canonicalization; `−0.0` preserved | — |
| Electrolyte and eNRTL complexity | schema supports it from phase 0; implementation in phase 4 | — |
| Recursive closures: DataFusion recursive CTE versus the external fixed-point loop | external loop when per-row derivations are required; CTE allowed otherwise | — |
| Expressiveness of the template DSL versus IDAES's escape hatch (`SkeletonUnitModel`) | authored instance equations remain IR, so the escape hatch stays inspectable | — |
| Crate naming (`pse-*`) and repository layout | placeholder, to confirm | rename is mechanical |
| Which HSL linear solvers are available on target machines | `probe_host` records them in `runtime.host_capabilities`; a run resolves its profile against that record and stores `resolved_options` (§18.3) | a profile-declared `fallback_linear_solver = mumps`, otherwise a typed `capability.backend` failure |

Status of the two design reviews' items in this revision:

| Review item | Revision 3 position |
|---|---|
| F1–F7, F9, F15 (first review, priority 1) | Resolved in revision 2 and verified by the second review. |
| F8 — rule bodies, invariant specs and selectors were JSON text | Resolved: `rule_plan_nodes`/`rule_plan_edges`/`rule_dependencies`, `selector_terms`, typed `case_sets` generators and `init_stages` targets (§4.1, §6.7, §6.10, §6.11). `diagnostics_findings.values` and `provenance.assertions.expected` stay JSON: open-ended evidence, never behaviour. |
| F10 — capability probe timing; resolved options | Resolved: `probe_host` operation, `runtime.host_capabilities`, `runs.resolved_options`, declared fallback policy (§6.13, §18.3). |
| F11 / R2-5 — pins, unmaintained parser, `num-dual` split, features | Resolved: §3.1 pins every crate and library, lockfile and `cargo deny` required, `serde-saphyr`, `num-dual` 0.15 with FeOs conditional, `arrow-flight` and `uom` dropped, `blake3` owned by `pse-ids`. |
| F12 — hashing constants and scope; borrow preconditions | Resolved: `pse.canon.v1` names alignment 64, `MetadataVersion::V5`, no legacy, no compression; metadata canonicalized at construction; IPC-only identity; foreign keys rejected; `-0.0` qualified; no `Debug` hashing (§5.3); borrow preconditions (§18.2). |
| F13 — `salsa` duplicated artifact-hash memoization | Resolved: deferred with a trigger (§14.3). |
| F14 — per-row derivation cost | Resolved: `derivation_granularity` per relation (§4.1, §14.2 rule 4) and a benchmark (§24.3). |
| F16 — Python loss profile; quantity types in the bundle | Resolved (§21.1). |
| F17 — SVD route, condition-number budget, `Exact` shapes | Resolved: shift-invert over `matrix_free` with a dense limit; Hager–Higham estimate by default with the Frobenius form opt-in and `pseudoinverse_from_svd_with_tolerance` for non-square (§15.4, §15.5); the pushdown spike was run — `Exact` is reachable for key equality and the shapes are stated (§5.4). |
| R2-1 — plan fingerprint non-reproducible; codec, analyzer rules, settings allow-list | Resolved (§4.3, §14.2 rule 5, §6.11, §24.1). |
| R2-2 — `-0.0` under grouping and joins | Resolved (§5.3, §14.2 rule 7, §19.2). |
| R2-3 — §4.3 erratum; extension-type registry | Resolved (§4.3, §4.4, §5.4, §14.3). |
| R2-4 — Python boundary contradictions and unenforced contracts | Resolved (§3.1, §3.3, §4.4, §21.1–§21.6). |
| R2-6 — `object_store`, petgraph, faer statements | Resolved (§15.3–§15.5, §20.1). |
| R2-7 — pushdown shapes, purity, wrapper test, `Precision` | Resolved (§5.4, §24.1). |
| R2-8 — leverage adoptions | Resolved (§4.4, §12.5, §14.2, §18.2, §18.3, §22.1, §23, §24.1). |

Residual risks that no revision can close by specification (carried as risks, not findings):

| Risk | Position |
|---|---|
| Declared MSRV of the pre-1.0 crates (`num-dual`, `faer`, `petgraph`, `egglog`, `diffsol`) | All build under the pinned nightly; declared floors are re-checked at every upgrade in CI. |
| `egglog` extraction determinism across processes and versions | Phase 4; adoption gated on a cross-process, cross-version test; never hash its renderings. |
| `object_store` S3 multi-writer commit coordination | Deferred while §20.1 is single-writer local; the ref CAS is the only serialization point and must be re-validated per backend before any multi-writer deployment. |
| Ipopt, HSL and MUMPS linking and redistribution | On the critical path for `pse-backend-native`; settled in phase 1 with a documented build recipe per platform. |
| `datafusion-proto` byte stability across DataFusion releases | Not assumed; the fingerprint is evidence, not a key, and its contract is versioned. |
| `feos-core` adopting `num-dual` 0.15 | Checked before phase 4; the provider stays conditional until then. |

## Appendix A. IDAES capability coverage matrix

The left column is the IDAES mechanism (source path relative to `idaes-pse/idaes/`); the middle column names the platform realization; the right column is the delivery phase.

| IDAES mechanism | Platform realization | Phase |
|---|---|---|
| `core/base/process_block.py`: `declare_process_block_class`, `CONFIG`, `build()`, scalar/indexed blocks, `initialize`/`idx_map` per-index config | templates (`authored.templates`, `template_params`), submodel multiplicity with per-member bindings (`idx_map`), instantiation pass P7 | 0–1 |
| `process_base.py`: `flowsheet()`, `_setup_dynamics`, default property package search, `initialization_order`, default scaling, `report`, performance and stream-table contents | `instance_tree`, feature inference rules (§13.2), `inferred.initialization_order`, `default_scaling`, display views | 1 |
| `flowsheet_model.py`: time domain creation, nesting rules, `model_check`, `stream_table` | `unit.flowsheet@1`, `domains` of kind `time`, stream-table view | 1 |
| `unit_model.py`: `add_inlet_port`/`add_outlet_port`/`add_port`, `add_state_material_balances`, `fix_initialization_states`, plug-in initialization | `template_ports`, `law.state_material_balance`, plan stages, plug-in order | 1 |
| `control_volume_base.py`: enums, `CONFIG_Template`, balance dispatch, `_rxn_rate_conv`, `_estimate_next_state` | §6.14 enums, `param_group.cv_options`, law expansion (§10), conversion rules, `estimate` stage | 1 |
| `control_volume0d.py`: all balance variants, holdup, phase fractions, reactions, custom terms, initialization, scaling | `cv.lumped@1` (§10.3), `scale.cv_lumped@1`, plan stages | 1 |
| `control_volume1d.py`: length domain, area definition, flow direction, linking variables, spatial derivatives, transformation | `cv.distributed_1d@1` (§10.4), discretization policy, P11 | 3 |
| `extended_control_volume0d.py`/`1d.py`: isothermal energy balance | `EnergyBalanceType.isothermal` law binding | 1/3 |
| `property_base.py`: parameter block, state block contract, ports from `define_port_members`, `build_on_demand` | `property_packages`, state templates (§9.2), port derivation, demand closure P6 | 1 |
| `property_meta.py`, `property_set.py`: unit sets, derived units, standard and electrolyte property sets, metadata flags | `unit_sets`, `quantity_kinds`, `property_kinds` (with `idaes_name`), `method_specs.provides/requires` | 0–1 (electrolyte set: 4) |
| `reaction_base.py`: reaction parameter and reaction blocks, `get_reaction_rate_basis`, required-property validation | `reaction_packages`, reaction block template, invariants | 3 |
| `phases.py`, `components.py`: phase types, component classes, validity rules, electrolyte sets | `phases`, `species`, `inferred.phase_species` rule | 1 (ions: 4) |
| `var_like_expression.py` | expression symbols; `case_specs` targeting an expression is an error | 1 |
| `costing_base.py`, `models/costing/SSLW.py` | `costing.flowsheet@1`, costing method templates, currency units, cost indices | 3 |
| `models/properties/modular_properties/base/generic_property.py`: config, build order, property builders, log forms, initialization | `property_packages`, `method_selections`, P6 realization, `init.modular_properties@1` | 1–2 |
| `state_definitions/*` | state templates (§9.2) | 1 (`FpcTP`, `FpTPxpc`: 2) |
| `eos/ideal.py`, `eos/ceos.py`, `eos/ceos_common.py`, `eos/enrtl*.py` | `eos.ideal@1`, `eos.cubic@1` with `cubic.compress_fact@1` kernel and external library, `eos.enrtl@1` | 1 / 2 / 4 |
| `pure/*` (NIST, RPP3–5, Perry's, constants, Chapman–Enskog, Chung, Eucken, electrolyte permittivity) | pure-component method templates | 1–2 (transport: 3) |
| `phase_equil/*` (bubble/dew, forms, Henry, SmoothVLE, SmoothVLE2) | phase-equilibrium method templates (§9.5) | 2 |
| `transport_properties/*` (Wilke, WMS, NoMethod) | transport mixing method templates | 3 |
| `reactions/*` (rate forms, rate constants, equilibrium forms and constants, heat of reaction) | reaction method templates (§9.7) | 3 |
| `interrogator/*` | query over `inferred.property_requirements` | 1 |
| `helmholtz/`, `general_helmholtz/`, `coolprop/` | Helmholtz expression templates plus `helmholtz.state@1` kernel; CoolProp parameter import | 4 |
| `models/unit_models/*` (all units in §11.1) | unit templates | 1–4 per §11.1 |
| `models/control/controller.py` | `control.pid@1` | 3 |
| `core/initialization/*` (initializer base, block triangularization, single control volume, from data) | plan model and standard plans (§17) | 1 |
| `core/util/initialization.py` (`fix_state_vars`, `revert_state_vars`, `propagate_state`, `solve_indexed_blocks`, `initialize_by_time_element`) | overlays, `propagate` operation, `solve_subset`, `plan.time_element_march@1` | 1 / 3 |
| `core/solvers/homotopy.py` | `continuation` stage | 2 |
| `core/solvers/get_solver.py`, `config.py`, `solvers/config.py` (option merging), `ipopt_l1`, `features.py` | `solver_profiles` with the IDAES defaults, capability probes | 1 |
| `core/solvers/petsc.py` (`petsc_dae_by_time_element`, DAE suffixes, trajectories) | PETSc plan over the NL backend, `math_dae_links`, trajectory ingestion | 3 |
| `core/scaling/*` (scaler base, custom scaler methods, nominal value tools, autoscaler, arc scaler, profiler, util) and `core/util/scaling.py` | scaling transformation model (§16), scaler templates, profiler plan, JSON adapters | 1–2 |
| `core/util/model_statistics.py` | `stats.*` views (§15.2) | 1 |
| `core/util/diagnostics_tools/*` (toolbox, SVD, Degeneracy Hunter, term analysis, evaluation errors, ill-conditioning, convergence analysis, bounds, halt on error) | checks of §15 | 1 (structural, basic numerical) / 2 (advanced) |
| `core/util/model_serializer.py` (`to_json`, `from_json`, `StoreSpec`) | overlays and result relations; JSON adapter for import | 1 |
| `core/util/dyn_utils.py` | case operations (§13.5) | 3 |
| `core/util/tables.py`, `tags.py` | stream-table and tag views, `authored.tags` | 1 |
| `core/util/units_of_measurement.py`, `constants.py`, `math.py` | reporting unit set, `reference.constants`, smooth operators in the opcode catalog | 0–1 |
| `core/util/misc.py` (`set_param_from_config`, `StrEnum`, compact expression printing) | parameter ingestion with units, enums as dictionaries, expression rendering | 0 |
| `core/util/config.py` validators, `exceptions.py` | parameter domains in `template_params`, failure taxonomy (§23) | 0 |
| `core/util/parameter_sweep.py`, `convergence/*` | case sets, runners, convergence analysis | 3 |
| `core/util/utility_minimization.py` | `law.heat_integration@1` | 3 |
| `core/util/phase_equilibria.py` (Txy diagrams) | a case set over compositions with bubble and dew symbols plus a plotting projection | 3 |
| `core/util/structfs/*` (step runner) | plans and pass records are the structured runner | 1 |
| `core/util/env_info.py` | `runtime.runs.environment` | 1 |
| Pyomo `network` (`Port`, `Arc`, `expand_arcs`, `SequentialDecomposition`) | ports, connections, P8, tear selection and sequential plan | 1–2 |
| Pyomo `dae` (`ContinuousSet`, `DerivativeVar`, `Integral`, finite difference, collocation, flatten utilities) | continuous domains, derivative symbols, `Integral` opcode, P11 | 3 |
| Pyomo `contrib.incidence_analysis` | `pse-structural` | 1 |
| Pyomo `contrib.pynumero` (`PyomoNLP` Jacobian route) | native evaluation program | 1 |
| Pyomo expression visitors, `differentiate`, `replace_expressions`, units container | IR traversal, AD, canonicalization, quantity types | 0–1 |
| Pyomo `common.config` (`ConfigBlock`) | `template_params`, `template_features`, parameter domains | 0 |
| Pyomo suffixes (`scaling_factor`, `dual`, `ipopt_zL_out`, `ipopt_zU_out`, DAE suffixes) | scaling relations, result relations, NL suffix emission | 1–3 |
| `ExternalFunction` and IDAES binary libraries (`functions`, `cubic_roots`, `general_helmholtz_external`) | `KernelSpec` bindings and the generated external-function library | 2–4 |
| `core/surrogate/*` (trainers, `SurrogateBlock`) | a trained surrogate is a `KernelSpec` (PySMO polynomial, RBF, kriging as expression templates; ALAMO expressions parsed into the IR; ONNX/Keras through OMLT via the Pyomo adapter) embedded by a `unit.surrogate@1` template with input-bound intersection; training itself is out of scope | 4 (embedding), out of scope (training) |
| `core/dmf/*`, `core/ui/*`, `apps/*`, `models_extra/*` | out of scope; the artifact store and provenance relations replace DMF's workspace, resource, and relation concepts | — |

## Appendix B. Relation index

| Namespace | Relations |
|---|---|
| `reference` | `schema_relations`, `schema_columns`, `schema_logical_types`, `schema_enums`, `schema_invariants`, `schema_migrations`, `dimensions`, `units`, `unit_sets`, `quantity_kinds`, `bases`, `reference_states`, `quantity_types`, `conversion_rules`, `constants`, `elements`, `property_kinds`, `method_specs`, `kernel_specs`, `pass_specs`, `rule_specs`, `operator_specs`, `rule_plan_nodes`, `rule_plan_edges`, `rule_dependencies`, `rule_expr_*`, `engine_profiles`, `aliases`, `diagnostic_thresholds`, `cost_indices`, `location_factors` |
| `authored` | `packages`, `documents`, `entities`, `model_revisions`, `case_revisions`, `change_sets`, `change_ops`, `domains`, `domain_members`, `continuous_domains`, `species`, `species_elements`, `phases`, `phase_species`, `henry_declarations`, `material_systems`, `reactions`, `stoichiometry`, `reaction_methods`, `reaction_packages`, `parameter_values`, `property_packages`, `state_bounds`, `phase_equilibrium_pairs`, `method_selections`, `default_scaling`, `templates`, `template_params`, `template_features`, `template_feature_rules`, `template_guards`, `template_domains`, `template_symbols`, `template_equations`, `template_submodels`, `template_ports`, `template_contributions`, `template_law_instances`, `template_requirements`, `template_display`, `template_scaling_defaults`, `instances`, `instance_equations`, `flowsheets`, `scopes`, `selector_terms`, `connections`, `cases`, `case_specs`, `case_spec_targets`, `case_activations`, `case_activation_targets`, `case_objectives`, `case_policies`, `discretization_policies`, `solver_profiles`, `datasets`, `observations`, `observation_targets`, `measurement_models`, `scenarios`, `case_sets`, `case_set_samples`, `tags` |
| `normalized` | `package_graph`, `domain_products`, the parsed expression graphs `template_expr_*`, `instance_expr_*`, `display_expr_*`, `contribution_expr_*`, `guard_expr_*` (one family per `pse.expr_dsl` column), canonical copies of authored relations after P3 |
| `inferred` | `valid_index_tuples`, `phase_species`, `phase_equilibrium_species`, `state_flash_required`, `property_requirements`, `method_resolutions`, `instance_tree`, `instance_features`, `instances`, `ports`, `port_members`, `scope_members`, `boundary_crossings`, `topology_edges`, `tear_candidates`, `connection_equations`, `initialization_order`, `undecided`, law negative-completeness rows |
| `compiled` | `meshes`, `mesh_nodes`, `stencils`, `quadrature_rules`, `symbols`, `symbol_references`, `symbol_groups`, `symbol_group_members`, `math_expr_nodes`, `math_expr_args`, `math_symbol_refs`, `math_float_constants`, `math_int_constants`, `math_affine`, `math_reductions`, `math_gathers`, `math_broadcasts`, `math_derivatives`, `math_integrals`, `math_smooth_ops`, `math_conditionals`, `math_kernel_calls`, `math_implicit_refs`, `math_unit_converts`, `math_indexed_equations`, `math_free_indices`, `math_equations`, `math_objectives`, `math_implicit_systems`, `math_complementarity`, `math_alternative_sets`, `math_alternatives`, `math_dae_links`, `math_static_analysis`, `kernel_bindings`, `scaling_plans`, `variable_scales`, `equation_scales`, `initialization_plans`, `init_stages`, `solve_plans`, `problems`, `variable_order`, `equation_order`, `case_bound_substitutions`, `incidence`, `dm_partition`, `blocks`, `block_members`, `sparsity_patterns`, `evaluation_programs`, `backend_bindings` |
| `runtime` | `runs`, `solutions`, `duals`, `residuals`, `iterations`, `solver_events`, `diagnostics_findings`, `kernel_evaluations`, `host_capabilities`, `sweep_results`, `profile_results`, `run_state` |
| `provenance` | `derivations`, `pass_records`, `assertions`, `refs`, `closure_report` |

## Appendix C. Glossary

| Term | Meaning |
|---|---|
| Authored | written by an author or package; a primitive fact |
| Derived | produced by a pass or rule; never authored; always carries a derivation |
| Reference | shipped library data (units, elements, methods, templates) |
| Semantic ID | 128-bit stable identity of an authored or derived entity |
| Ordinal | artifact-local integer position |
| Content hash | blake3 of a canonically serialized artifact |
| Snapshot | the set of relation artifacts named by one manifest |
| Model revision, case revision, run | structure, values, execution (never mixed) |
| Template | declarative replacement for an IDAES `*Data` class: parameters, features, symbols, equations, submodels, ports, contributions, laws |
| Contribution | a typed physical term (flow, generation, transfer, accumulation) that laws collect |
| Law template / instance | a universal balance or accounting rule and its scoped application |
| Kernel | a constitutive computation implemented in code under a `KernelSpec` contract |
| Method | an IDAES-style property or reaction submodel, realized as an equation template or a kernel |
| Quantity type | dimension, kind, basis, reference state, scale kind, shape, subject |
| Scale kind | `point` (a value on a scale: T, P, h) or `difference` (ΔT, ΔP); governs the addition algebra of §8.3, not multiplication |
| Undecided | a rule outcome of `unknown` or `conflict`, stored in `inferred.undecided` rather than in any head relation |
| Case-bound view | the canonical graph with `treatment = parameter` symbols substituted by their case values; the input of structural analysis (P14) |
| Index expansion | pass P12: the indexed equation instances of P7–P11 become one scalar equation row per valid index tuple |
| Engine profile | the declared DataFusion version, analyzer/optimizer/physical rule lists and the versioned semantic-settings allow-list that a rule-executing pass depends on (§14.2 rule 5) |
| Plan fingerprint | blake3 of the canonicalized `datafusion-proto` bytes of a rule plan; recorded as evidence in the pass record, never part of the memo key (`pse.planfp.v1`) |
| Derivation granularity | per-relation declaration of whether derivations are stored per row or reconstructed from the rule and the ID formula |
| Host capabilities | the recorded result of the explicit `probe_host` operation; the input from which a run's solver options are resolved |
| Loss profile | the bundle manifest's per-column statement of which `pse.*` extension types survive the Python boundary and which degrade to storage types |
| Overlay | an immutable case that overrides its parent |
| Plan | an ordered list of stages (initialization, scaling, solve, sweep) executed by the runtime |
| Pass | a contracted compiler transformation between relation versions |
| Rule | a typed, stratified inference producing derived rows |
| Closure report | proof that every closure category (type, topology, scope, quantity, method, balance, optionality, equation, runtime) is satisfied |
