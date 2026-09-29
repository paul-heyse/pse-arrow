---
title: Workspace, crates and dependencies
status: current
---

# Workspace, crates and dependencies

This page explains how the Cargo workspace is divided into crates, which third-party
libraries own which operations, and which rules keep the dependency graph reproducible.
Exact versions live in the manifests, not here: `Cargo.toml`, `Cargo.lock`,
`pyproject.toml`, `uv.lock` and `rust-toolchain.toml`. Workspace membership is
declared by Cargo metadata. Dependency ceilings and pinned families live in
`[workspace.metadata.pse]`, and `xtask` enforces them.

## 3. Workspace, crates and library boundaries

The workspace is a Rust core with a thin Python package on top. It uses `pse-*` crate
names under [ADR-0002](../../adr/0002-project-name-license-and-distribution.md). Crate
boundaries are part of the architecture: adding or removing a `pse-*` crate needs an ADR
([§24.4](design-change-workflow.md#section-24-4)). Adding a third-party library does not
([§3.3.2](#section-3-3-2)). Library choices follow the responsibilities in
[§0.5](architecture-overview.md#section-0-5): authored typed relations remain the model
authority. Authored packages own scientific equations and data. Libraries own mathematics,
numerical algorithms, relational execution and storage.

### 3.1 Version authority and pinned families

> Decision: [ADR-0122](../../adr/0122-nightly-toolchain-and-feature-unification.md), superseding
> ADR-0018, amends the toolchain rule below. `rust-toolchain.toml` pins one dated nightly,
> its only declaration, and `rust-version` is the stable language floor (1.98.1) that the
> nightly must be at or above (`toolchain_matches_msrv`). `.cargo/config.toml` turns on
> workspace feature unification. Each checkout keeps its own build directory in
> `target/`; a build directory shared between checkouts is rejected as unsound, and
> sccache serves reuse across checkouts. The `=` pins, the committed lockfile, `--locked`
> and the supply-chain checks stand. The dependency ceilings do not follow the
> feature-only edge to `pse-workspace-hack`. Implemented.

**Authority.** Every third-party Rust version is declared once, in
`[workspace.dependencies]` of the root `Cargo.toml`. Members inherit it with
`.workspace = true`. External dependencies use exact `=` versions or an exact commit.
`Cargo.lock` is committed, and every recipe runs `--locked`. Runtime Python libraries are
pinned with `==` in `pyproject.toml`. Tools that only execute carry floors in dependency
groups, and `uv.lock` records what resolved. `rust-toolchain.toml` pins one dated
nightly, and `rust-version` is the stable language floor at or below it
([ADR-0122](../../adr/0122-nightly-toolchain-and-feature-unification.md); governance test
`toolchain_matches_msrv`). No build runs on stable Rust: the pin moves to a recent nightly
when the project needs one, and a breakage is fixed when it appears (maintainer decision,
2026-09-29; build-review F07). The manifests contain no `[patch]` or `[replace]` tables.
The `dependency_pins` governance test checks that each external declaration is exact.
No prose table in this collection is a second pin authority. Read the manifest for a
version, and read the [capability maps](../../capability-maps/README.md) for what that
version exposes.

**One type universe.** Arrow, Parquet, `object_store`, DataFusion and PyO3 must each
resolve to exactly one version. If two Arrow majors coexist, `downcast_ref` returns `None`
with no compile error. An exact pin constrains only a direct dependency, and
`cargo tree -d` does not report a split family because each package name appears once.
`[workspace.metadata.pse.families]` therefore groups the crates by family: `arrow-*` with
`parquet-*`, and `datafusion-*` without its separately released tracing adapter.
`object_store` forms its own family. PyO3 is matched by minor version. `just family-check`
compares the resolved graph with those declarations. The header comment of `Cargo.toml`
explains why exact pins alone are not enough. Upgrading any family member moves the whole
family, and a major upgrade of one of the four families needs an ADR. Dependabot groups
updates by family ([ADR-0035](../../adr/0035-dependabot-with-family-groups.md)).

**Dependency ceilings.** `[workspace.metadata.pse.dependency-ceilings]` lists roots
whose resolved normal-dependency closure must exclude named crates. `family-check` enforces
this list too, including back-edges through native dependencies. There are three
ceilings:

| Ceiling | Roots | Excluded from the closure |
|---|---|---|
| semantic | `pse-ids`, `pse-diagnostics`, `pse-vocabulary`, `pse-model`, `pse-authoring`, `pse-quantity`, `pse-modeling`, `pse-structural`, `pse-compiler` | Arrow, Parquet, DataFusion, Delta, `object_store`, Tokio |
| columnar | `pse-schema`, `pse-relations`, `pse-columnar` | full DataFusion/SQL, Delta, engine, catalog, runtime and codegen crates |
| generator | `pse-codegen` | DataFusion, Delta and every crate that consumes generated values |

**Versions that are contracts.** A few versions are themselves part of the design. They
are cited by the decision that owns them rather than by this page:

- Parity is pinned to the isolated `idaes-pse==2.13.0` reference
  ([ADR-0097](../../adr/0097-modeling-scope-and-parity.md), proposed; pin implemented).
- Ipopt is built from pinned sources in a digest-pinned solver image, and the committed
  bindgen output follows its C ABI
  ([ADR-0108](../../adr/0108-ipopt-linear-solvers-and-solver-image.md), superseding ADR-0028).
- The vendored Delta source (`vendor/delta-rs`) and its kernel branch are recorded as
  exact revisions in workspace metadata. `just delta-source` verifies or regenerates that
  override, and the governance test `delta_revisions` checks the pair.
- The Python platform targets CPython 3.11 or later. The `parity` dependency group is
  limited to the interpreters that IDAES classifies.

External reading copies under `external/` are fetched rather than vendored
([ADR-0032](../../adr/0032-external-checkouts-are-not-vendored.md)). A reading copy is not
evidence of the resolved API. Tie an API claim to the lockfile and the capability-map
evidence.

### 3.2 Workspace crates and dependency direction

> Decision: [ADR-0122](../../adr/0122-nightly-toolchain-and-feature-unification.md) — the crate
> `pse-workspace-hack` has no code. cargo-hakari generates its dependencies from
> `.config/hakari.toml`, and every member except `pse-ids`, `pse-diagnostics` and the
> generated `pse-operations-queries` depends on it, for feature unification only.
> Implemented.
>
> Decision: [ADR-0082](../../adr/0082-library-owned-process-mathematics.md),
> [ADR-0083](../../adr/0083-class-specific-native-execution.md),
> [ADR-0098](../../adr/0098-modeling-knowledge-ownership.md) (proposed; implementation authorized).
>
> Decision: [ADR-0114](../../adr/0114-typed-operational-store.md) — the crate
> `pse-operations` owns the operational store and publication catalog, and `pse-runtime`
> depends on it and carries the `pse-worker` binary (Plan 22 O2–O4); the generated crate
> `pse-operations-queries` sits beneath it, and optional `postgres` features carry the
> generated value mapping (Plan 22 B1, B2). All implemented.
> [ADR-0117](../../adr/0117-platform-vocabulary-crate.md) — the crate `pse-vocabulary` holds
> the registry's platform vocabularies beneath `pse-schema` and `pse-model` (Plan 22 B4,
> implemented). As built it depends on `pse-diagnostics` for its typed parse error, beyond
> ADR-0117 Outcome 1's "serde and thiserror/miette only".

Cargo metadata owns workspace membership: `members` in the root manifest are `crates/*`,
the five `tests/*` crates, `xtask` and `benches`. The table explains roles; it does not
register crates.

| Crate | Responsibility |
|---|---|
| `pse-diagnostics` | Shared diagnostic vocabulary and lossless native causes; depends only on `thiserror`/`miette`, plus postgres-types behind its optional `postgres` feature |
| `pse-ids` | Semantic IDs, content hashes, typed framing under the `Frame` catalog, the typed-id macro; the sole BLAKE3 owner ([§5](identity-and-publication.md#section-5)) |
| `pse-vocabulary` | The ten platform vocabularies the registry is written in (namespace, authority, snapshot class, column role and the rest), re-exported by the generator; depends on `pse-diagnostics` and serde ([ADR-0117](../../adr/0117-platform-vocabulary-crate.md)) |
| `pse-quantity` | Dimensions, units, quantity kinds/types, bases, reference states, conversions, physical operation and function vocabulary ([§8](physical-semantics.md#section-8)) |
| `pse-modeling` | Pure checking and specialization of generic declarations, interfaces, sets, functions, contributions and annotations |
| `pse-model` | Registry-generated plain semantic values, enums and typed ids, without Arrow; validated scalar settings; the store value mapping behind its `postgres` feature |
| `pse-authoring` | Modeling language and shared expression grammar, spans and parse budgets, identity assignment, exact package resolution |
| `pse-kernels` | Generic external-function contracts, typed shapes, derivative/smoothness declarations and attempt-local workers ([§9](physical-semantics.md#section-9)) |
| `pse-structural` | Complete immutable graph projections; incidence, matching/DM/BTF, flowsheet and initialization projections ([§15](numerical-execution.md#section-15)) |
| `pse-math` | Physically admitted Symbolica bodies, guarded evaluation, coefficients, sparse assembly and library bindings ([§7](mathematics-and-compilation.md#section-7)) |
| `pse-compiler` | The single Salsa workspace, admitted revision reuse, mathematical lowering and bounded preparation ([§14](mathematics-and-compilation.md#section-14)) |
| `pse-backend-native` | Class-specific native adapters: Ipopt, POUNCE, POUNCE-convex, KINSOL, HiGHS, Clarabel, SCIP, Diffsol and IDAS, plus tears, presolve and quality ([§18](numerical-execution.md#section-18)) |
| `pse-ipopt-sys` | Generated raw Ipopt C bindings; the build script emits link directives only |
| `pse-columnar` | Arrow canonicalization, owned buffers, reservations, cancellation and native error adapters |
| `pse-schema` | The semantic registry, native field contracts, resolved contracts, compatibility and Delta layouts ([§4](schema-and-relations.md#section-4)) |
| `pse-codegen` | Pure registry generators; returns the generated tree without writing files |
| `pse-relations` | Generated Arrow views, builders, extension types and validators over checked batches |
| `pse-engine` | DataFusion session assembly, bound preparation and execution, validation, providers and resources |
| `pse-rules` | Registry-declared invariants as one native diagnostic query; its consumers are inspection and fixture validation |
| `pse-catalog` | Catalog providers, Delta member writes and candidate admission, exact reopening, export manifests, inspection and catalog-instructed collection; never sees PostgreSQL ([§20](identity-and-publication.md#section-20)) |
| `pse-operations` | The operational store and publication catalog on PostgreSQL 18: the embedded generated schema, create-or-refuse, the attempt lifecycle tables, typed repositories, the listener and the catalog with its reader leases; the `pse-ops` binary ([§20.6](identity-and-publication.md#section-20-6)) |
| `pse-operations-queries` | Generated by Cornucopia from `crates/pse-operations/queries/*.sql`: typed tokio-postgres statements over the registry rows; never edited by hand |
| `pse-runtime` | Composition root: document ingestion, physical inventory, the public workflow, jobs, budget and cancellation, durable execution, publication and the operational query providers; the `pse-worker` and `pse-publication` binaries ([§19](workflows-and-results.md#section-19), [§20](identity-and-publication.md#section-20)) |
| `pse-buildinfo` | Build provenance: toolchain, profile, Git revision and embedded lockfile identity |
| `pse-py` | The PyO3 extension `pse._native`, which is the whole Rust/Python boundary ([§21](workflows-and-results.md#section-21)) |
| `pse-testkit` | Development-only fixtures that use production engine factories; never a production dependency |
| `pse-workspace-hack` | No code: cargo-hakari generates its dependencies so every package selection resolves one feature set per dependency ([ADR-0122](../../adr/0122-nightly-toolchain-and-feature-unification.md)) |

| Non-product member | Role |
|---|---|
| `pse-tests-governance` | Workspace invariants: pins, MSRV, unsafe allowlist, error taxonomy, BLAKE3 ownership, shadow structs, regeneration |
| `pse-tests-engine` | Engine and catalog inspection, pushdown truthfulness and unified sources |
| `pse-tests-conformance` | Canonical encoding properties, generated invariant fixtures and invariant domains |
| `pse-tests-lifecycle` | Publication object races and memory-budget behavior |
| `pse-tests-structural` | Reserved for structural fixtures; it currently contains only a placeholder, and structural tests live with their crates |
| `xtask` | Code generation and regeneration checks, family and ceiling checks, governance and inspection fixtures; the justfile is its surface |
| `pse-benches` | Criterion groups for canonicalization, native cache, consolidation and process cases |

Dependencies point one way. The semantic foundations (`pse-diagnostics`, `pse-ids`,
`pse-quantity`, `pse-model`) sit below `pse-authoring`, `pse-modeling`, `pse-kernels`,
`pse-structural`, `pse-math`, `pse-compiler` and `pse-backend-native`. None of these
reaches Arrow, DataFusion, Delta or Tokio. The columnar crates (`pse-columnar`,
`pse-schema`, `pse-relations`) sit beside them and use Arrow plus the DataFusion leaf
expression APIs. `pse-engine` adds sessions. `pse-rules` and `pse-catalog` build on the
engine. `pse-vocabulary` sits beneath `pse-schema` and `pse-model`, so the registry and the
generated model share one type per platform vocabulary without either ceiling giving way.
`pse-operations` depends only on `pse-model`, `pse-ids`, `pse-diagnostics` and its
generated query crate; no semantic, native or columnar crate depends on it.
`pse-runtime` is the only crate that joins the semantic/native side with the
columnar/storage side and the operational store, and it owns every effect: document I/O,
compilation, native attempts, durable execution and publication. `pse-py` depends on the
runtime and the inspection crates.
`pse-codegen` runs outside the product's normal dependency graph and never depends on
generated values. Generated code is placed in its consumers: see
[§4.2](schema-and-relations.md#section-4-2).

The unsafe-code allowlist in workspace metadata names `pse-ipopt-sys`,
`pse-backend-native` and `pse-py`, which are FFI boundaries. It also names `pse-math`,
whose only unsafe use reads the GMP/MPFR version strings
([ADR-0088](../../adr/0088-selected-model-and-physical-contracts.md)). Every profile keeps
`panic = "unwind"` because the Python and native callback boundaries contain unwinds.

### 3.3 Supporting libraries and their boundaries

> Decision: [ADR-0082](../../adr/0082-library-owned-process-mathematics.md),
> [ADR-0083](../../adr/0083-class-specific-native-execution.md),
> [ADR-0084](../../adr/0084-physical-provider-and-dynamic-contracts.md)
>
> Decision: [ADR-0102](../../adr/0102-discrete-and-global-design-target.md),
> [ADR-0105](../../adr/0105-scip-factorable-backend.md) — SCIP for MIQP, MINLP and global
> certification (Plan 22 G1–G7, implemented, with G4 and G6 partial);
> [ADR-0108](../../adr/0108-ipopt-linear-solvers-and-solver-image.md) — SPRAL SSIDS, oneMKL
> and METIS for Ipopt (Plan 22 N1, implemented);
> [ADR-0114](../../adr/0114-typed-operational-store.md) — PostgreSQL 18
> through `pse-operations`: tokio-postgres, deadpool-postgres, postgres-types and
> tokio-postgres-rustls, with statements compiled by Cornucopia; sqlx is removed (Plan 22 B2,
> implemented); [ADR-0116](../../adr/0116-typed-boundary-documents.md) — schemars and
> nutype for Rust-owned documents and validated settings (Plan 22 B5, implemented);
> typed-index-collections and enum-map at coordinate boundaries (Plan 22 B6, implemented);
> [ADR-0118](../../adr/0118-one-kkt-point-analysis.md) — FERAL and `pounce-sens-core` for
> the KKT-point analysis (Plan 22 S0 and S1, implemented);
> [ADR-0121](../../adr/0121-convexity-compiler-facts.md) — Clarabel on LP and convex QP with
> QDLDL or oneMKL Pardiso, without `clarabel/faer-sparse` (Plan 22 C4, implemented).

Each library owns the operation it implements. PSE code owns the physical, identity and
admission contracts around the library call. A library never becomes the authority for
units, identity, schemas or canonical encoding merely because it is present.

| Library | Owner crate | Role | Boundary |
|---|---|---|---|
| Arrow, Parquet | columnar crates, engine, catalog | Checked columnar data, extension types, IPC and FFI transport | Physical layout safety is not semantic validity ([§4.6](schema-and-relations.md#section-4-6)) |
| DataFusion | `pse-engine`, `pse-rules`, `pse-catalog`, `pse-runtime` | Relational admission, invariant queries, inspection, DML and storage plans | Not a mathematical evaluator ([§3.3.1](#section-3-3-1)) |
| Delta Lake (`deltalake-core`), `object_store` | `pse-catalog` | Durable authored/result member tables, export manifests, and the checkpoint, vacuum and removal the catalog instructs | Visibility, settlement and retention are the catalog's ([§20](identity-and-publication.md#section-20)); remote object stores unqualified (register R-37) |
| PostgreSQL 18: tokio-postgres, deadpool-postgres, postgres-types/postgres-protocol, tokio-postgres-rustls (ring) | `pse-operations`, `pse-operations-queries`; value mapping in `pse-ids`, `pse-model`, `pse-diagnostics`, `pse-vocabulary` | Operational store and publication catalog: pooled connections, typed `SqlState`, binary `COPY`, one listener connection | The registry owns the schema; no SQL is assembled at run time; semantic crates carry only the value protocol ([§20.6](identity-and-publication.md#section-20-6)) |
| Cornucopia | `xtask` | Compiles the store's SQL statements into the generated query crate against a temporary database | Generation only: no runtime dependency; regeneration needs the local server |
| schemars, nutype | owners of Rust-owned documents; `pse-model` | JSON Schemas of boundary documents; validated single-value settings | The serde type is the document's authority ([§21.5](workflows-and-results.md#section-21-5)) |
| typed-index-collections, enum-map | `pse-math`, `pse-backend-native`, `pse-kernels` | One index type per coordinate space; enum-indexed storage | FFI and faer interiors keep `usize`/`i32`, converted at the adapter |
| Symbolica/Numerica | `pse-math` | Algebra, normalization, differentiation, multi-output evaluators and jets | Starts after physical typing and domain obligations; derived artifacts only |
| faer | `pse-math`, `pse-backend-native`, `pse-runtime` | Sparse structure, refill maps, products; bounded LU/SVD for implicit responses and rank | Native solver factorizations stay with their solver |
| FeOS, `feos-core`, num-dual, `quantity`, nalgebra | optional conformance reference tests | Independent thermodynamic and derivative comparisons | No production property route or physical-type authority |
| Ipopt (C ABI) with MUMPS+METIS, SPRAL SSIDS and oneMKL Pardiso; POUNCE | `pse-backend-native` | Local NLP through one shared oracle; a typed linear-solver selection for Ipopt | Separate native routes; no fallback between them or between linear solvers; one BLAS/LAPACK provider (oneMKL) and one OpenMP runtime (libgomp) per process ([§18.3](numerical-execution.md#section-18-3)) |
| pounce-presolve | `pse-math`, `pse-structural`, `pse-backend-native` | Matching, DM, BTF and qualified presolve/postsolve | Original-coordinate recovery is validated independently |
| FERAL, pounce-sens-core | `pse-backend-native` | FERAL: LDLᵀ with inertia of the normalized KKT matrix and sparse LU for conditioning estimates; pounce-sens-core: parametric steps and reduced Hessians over that factor (`SensBacksolver`) | Post-solve analysis only, never a solver of record; verdicts and quantities are stated in original coordinates with their validity ([§15.5.1](numerical-execution.md#section-15-5-1)) |
| SUNDIALS KINSOL/IDAS (+ KLU) | `pse-backend-native` | Square roots, including one-sided bounds, and declared fixed-point iteration; IDAS for recoverable trials, scheduled inputs, directional events, sign constraints, adjoint gradients and forward-over-adjoint second-order sensitivities | Optional features; two-sided boxes on KINSOL and unsupported profiles are refused; KINSOL's nested sessions are budgeted in bytes |
| HiGHS | `pse-backend-native` | LP, MILP, certified convex QP, lexicographic LP and MILP objectives in one solve, and tear MILPs | Integrality is never relaxed silently |
| POUNCE-convex (`pounce-rs` feature `convex`) | `pse-backend-native` | LP, convex QP and continuous cones on explicit selection, batched parallel QP solves for studies, and sum-of-squares bounds on polynomial programs | Explicit only; its QP sensitivity is not used (ADR-0118); SOS bounds are labelled non-rigorous ([§18.10](numerical-execution.md#section-18-10)) |
| Clarabel | `pse-backend-native` | Explicit cones, with SDP under an optional feature on oneMKL; LP and convex QP on explicit selection, lowered to cone form; KKT by QDLDL or, under `clarabel-pardiso`, oneMKL Pardiso from the linked oneMKL | pse-owned boundary types; cone recognition comes from compiler facts, never from Clarabel; rays are verified certificates; `clarabel/faer-sparse` is not used, since it would bring a second faer ([§18.10](numerical-execution.md#section-18-10)) |
| SCIP 10.0.2 (`scip-sys`) | `pse-backend-native` | MIQP, MINLP and explicit global certification over the factorable projection; native constraint handlers, exact rational MILP, IIS, a ranked solution pool and reoptimization | Raw binding against the solver image, checked at build and run time; deterministic concurrency under admitted threads; every claim re-qualified in original coordinates ([§18.10.1](numerical-execution.md#section-18-10-1)) |
| Diffsol | `pse-backend-native` | BDF, SDIRK and explicit dynamics for the admitted mass-matrix profile, with pse-owned faer LU or KLU linear solvers, events, scheduled inputs, forward sensitivities and checkpointed adjoint gradients | No second model language |
| Salsa | `pse-compiler` | Synchronous semantic reuse over admitted values | No I/O, native state or effects in tracked queries |
| rustworkx-core, petgraph | `pse-structural` | Deterministic ordering, acyclicity and graph projections | Graph indices never cross the projection boundary |
| BLAKE3 | `pse-ids` only | Content hashing and derived identity | The canonicalizer, not the hash, defines coverage |
| `serde-saphyr`, `toml`, `winnow`, `sqlparser` | `pse-runtime`, `pse-authoring`, `pse-schema` | YAML/TOML loading with spans, the expression DSL, declared SQL predicates | Parsing evaluates nothing; budgets bound hostile input ([ADR-0021](../../adr/0021-serde-saphyr-replaces-serde-yaml.md)) |
| `syn`, `quote`, `prettyplease` | `pse-codegen`, `xtask` | Deterministic registry and bindgen rendering | Output is committed and checked by regeneration |
| Tokio, Rayon, tracing | runtime, engine, native | One process executor, admitted native pools and structured observation | Observation never changes results ([§23](operations-and-validation.md#section-23)) |
| PyO3, `pyo3-arrow`, `pyo3-async-runtimes` | `pse-py` | Extension module, PyCapsule streams and async jobs | [ADR-0024](../../adr/0024-pyo3-arrow-over-arrow-pyarrow.md) |
| `pyarrow`, `attrs`/`cattrs`, `msgspec`, optional NumPy | `python/pse` | Arrow boundary, generated contract classes, codecs and array boundary | A contract lives in exactly one class system |
| `idaes-pse`, Pyomo (parity group); `teqp` (thermo-reference group) | Python test groups | Reference behavior and independent thermodynamic references | Never production dependencies |

A library's presence does not grant capability. A feature that is compiled in still
needs an admitted operation, profile and test before a workflow advertises it
([§18](numerical-execution.md#section-18)). JIT and SIMD evaluation and GPU execution are
not admitted. Mixed-integer nonlinear solving and global certification through SCIP
(ADR-0102, ADR-0105) are implemented for factorable problems over finite boxes, with the
Plan 22 G4–G7 extensions ([§18.10.1](numerical-execution.md#section-18-10-1)) and durable
incumbents, G8 ([§20.6](identity-and-publication.md#section-20-6)). A library that has no
current consumer is not admitted either. That is a limit of the present scope, not a
prohibition
([§25](scope-and-open-design.md#section-25)).

#### 3.3.1 Arrow and DataFusion roles and capability eligibility

Arrow and DataFusion own the data, relational, admission/inspection and storage
boundaries:

- checked construction and transport of relation batches;
- set-oriented admission and invariant queries;
- catalog and provider inspection;
- editable-table DML;
- plan transport for diagnostics;
- Delta-backed durable reads and writes.

They do not evaluate process mathematics. No expression, derivative or solver iteration
runs as SQL, as a user-defined function or through a DataFusion plan
([ADR-0082](../../adr/0082-library-owned-process-mathematics.md)).

Every Arrow and DataFusion feature, operator, function, extension point, feature flag and
companion crate remains eligible within these roles. An initial subset, a missing
consumer or an optional dependency does not prohibit a feature. Choose the mechanism
that preserves meaning and removes duplicate work, then qualify it against the actual pin
and consumer. Placement rules still apply:

- Sessions are built through typed `ConfigOptions`. `SessionConfig::set_str` and the
  panicking `Field::extension_type` are banned in `clippy.toml`.
- Registration finishes before a session is used. Planner, analyzer, function and type
  extensions may project the one registry authority; they cannot define a second rule,
  unit or schema system.
- External or asynchronous sources resolve to explicitly versioned inputs before
  reproducible execution. Volatile and effectful functions declare their effects and
  cancellation.
- DML may change private authoring or attempt state. Validated publication remains the
  commit boundary ([§20](identity-and-publication.md#section-20)).
- Transport encodings and compression may differ from the canonical frame. Canonical
  identity encoding stays fixed ([§5.3](identity-and-publication.md#section-5-3)).
- Native hashes, joins and aggregates are available. Hash equality, extension metadata,
  castability or plan bytes never substitute for typed equality and complete dependency
  binding.

Resource budgets for these libraries are deployment configuration sized to the
workstation, not language limits ([§23](operations-and-validation.md#section-23)).

#### 3.3.2 Dependency admission and licence policy

> Decision: [ADR-0066](../../adr/0066-dependency-admission-and-licence-policy-are-advisory.md)

No third-party library is refused, and no licence is grounds to refuse one. Adding a
crate or Python package needs no ADR, no design review and no documentation row. Pin it
exactly in the owning manifest, commit the lockfile, and keep `just family-check` clean.
Substitution is a later, evidence-backed decision. Do not design around it now.

Admission is relaxed. These rules are not:

- **One type universe.** See [§3.1](#section-3-1).
- **Exact pins and locked resolution.** They are required for every dependency.
- **Semantic boundaries.** A library placed under [§3.3](#section-3-3) or
  [§3.3.1](#section-3-3-1) does not define units, identity, schemas or canonical encoding.
- **Declared provenance.** A library that affects meaning or output enters artifact
  identity as a declared input ([§5](identity-and-publication.md#section-5)).

`deny.toml` is configured to report. `just deps-report` is advisory and `just policy` is
the strict audit; both run on demand. The licensing question for distributed artifacts
is deferred, not answered. Its trigger is the first publishable crate or PyPI release. The
[dependency policy](../../dev/dependency-policy.md) owns the procedure and that deferral.

#### 3.3.3 Computation placement

Each operation's contract selects its mechanism. No layer is the default executor for
everything.

| Operation | Mechanism | Owner |
|---|---|---|
| Parsing, identity assignment, target and package resolution | Typed Rust over spans | `pse-authoring`, `pse-runtime::authoring_driver` |
| Relation admission, invariants, inspection, set-oriented model/result work | Arrow and DataFusion | `pse-schema`, `pse-engine`, `pse-rules` |
| Package admission; pure checking and specialization | Explicit immutable closure; generic typed Rust | `pse-runtime::workflow::modeling`; `pse-modeling` |
| Physical typing, finite expansion, body preparation | Typed Rust, then Symbolica | `pse-modeling`, `pse-compiler`, `pse-math` |
| Incremental semantic reuse | Salsa, pure and synchronous | `pse-compiler::workspace` |
| Scientific properties and derivatives | Authored functions and potential identities lowered through Symbolica/Numerica | Packages; `pse-compiler`, `pse-math` |
| Structural analysis | pounce-presolve, rustworkx-core, petgraph | `pse-structural` |
| Numerical solution and integration | The class-specific native library | `pse-backend-native` |
| Sparse numerical linear algebra outside solvers | faer | `pse-math`, `pse-runtime` |
| Operational state, publication visibility and retention decisions | PostgreSQL through generated statements | `pse-operations` |
| Member data, export manifests and maintenance | Delta through DataFusion | `pse-catalog` |
| Durable jobs, workers and studies | The operational store's queue, composed with the workflow | `pse-runtime` |
| Effects: I/O, native compilation, attempts, cancellation, joins | Explicit runtime ownership | `pse-runtime` |

The rule has three consequences:

- No per-tuple SQL expansion of mathematics.
- No project-local arithmetic interpreter, AD tape or rule fixed-point engine.
- No solver state or cache side effect in a tracked query.

A domain algorithm does not need a DataFusion extension to take part in compilation.
Before adding an operation, identify the owner whose contract already matches. Extend
that owner. Where a library is missing, add it under [§3.3.2](#section-3-3-2). Replaced
code and its callers are deleted in the same change. No compatibility path or fallback
engine remains.
