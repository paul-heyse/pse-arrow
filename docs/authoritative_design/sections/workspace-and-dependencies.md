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
authority. Libraries own the mathematics, thermodynamics, numerical algorithms, relational
execution and storage they are good at.

### 3.1 Version authority and pinned families

**Authority.** Every third-party Rust version is declared once, in
`[workspace.dependencies]` of the root `Cargo.toml`. Members inherit it with
`.workspace = true`. External dependencies use exact `=` versions or an exact commit.
`Cargo.lock` is committed, and every recipe runs `--locked`. Runtime Python libraries are
pinned with `==` in `pyproject.toml`. Tools that only execute carry floors in dependency
groups, and `uv.lock` records what resolved. `rust-toolchain.toml` pins the stable
toolchain, and `rust-version` stays equal to it
([ADR-0018](../../adr/0018-dependency-pins-lockfile-and-msrv.md); governance test
`toolchain_matches_msrv`). The manifests contain no `[patch]` or `[replace]` tables.
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
| semantic | `pse-ids`, `pse-diagnostics`, `pse-model`, `pse-authoring`, `pse-quantity`, `pse-material`, `pse-structural`, `pse-compiler` | Arrow, Parquet, DataFusion, Delta, `object_store`, Tokio |
| columnar | `pse-schema`, `pse-relations`, `pse-columnar` | full DataFusion/SQL, Delta, engine, catalog, runtime and codegen crates |
| generator | `pse-codegen` | DataFusion, Delta and every crate that consumes generated values |

**Versions that are contracts.** A few versions are themselves part of the design. They
are cited by the decision that owns them rather than by this page:

- Parity is pinned to the isolated `idaes-pse==2.12.0` reference
  ([ADR-0003](../../adr/0003-clean-room-relationship-and-parity-pin.md)).
- Ipopt is built from pinned sources in a digest-pinned solver image, and the committed
  bindgen output follows its C ABI
  ([ADR-0028](../../adr/0028-solver-acquisition-source-built-ipopt.md)).
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

> Decision: [ADR-0082](../../adr/0082-library-owned-process-mathematics.md),
> [ADR-0083](../../adr/0083-class-specific-native-execution.md)

Cargo metadata owns workspace membership: `members` in the root manifest are `crates/*`,
the five `tests/*` crates, `xtask` and `benches`. The table explains roles; it does not
register crates.

| Crate | Responsibility |
|---|---|
| `pse-diagnostics` | Shared diagnostic vocabulary and lossless native causes; depends only on `thiserror`/`miette` |
| `pse-ids` | Semantic IDs, content hashes, typed framing; the sole BLAKE3 owner ([§5](identity-and-publication.md#section-5)) |
| `pse-quantity` | Dimensions, units, quantity kinds/types, bases, reference states, conversions, physical operation and function vocabulary ([§8](physical-semantics.md#section-8)) |
| `pse-material` | Species, elements, phases, reactions, stoichiometry and material-system predicates |
| `pse-model` | Registry-generated plain semantic values and enums, without Arrow |
| `pse-authoring` | Expression DSL, spans and parse budgets, target paths, identity assignment, exact package resolution |
| `pse-kernels` | Physical provider contracts: the FeOS PC-SAFT/DIPPR provider, the directional valve and operating envelopes ([§9](physical-semantics.md#section-9)) |
| `pse-structural` | Complete immutable graph projections; incidence, matching/DM/BTF, flowsheet and initialization projections ([§15](numerical-execution.md#section-15)) |
| `pse-math` | Physically admitted Symbolica bodies, guarded evaluation, coefficients, sparse assembly and library bindings ([§7](mathematics-and-compilation.md#section-7)) |
| `pse-compiler` | Typed finite specialization, source binding and bounded synchronous Salsa preparation ([§14](mathematics-and-compilation.md#section-14)) |
| `pse-backend-native` | Class-specific native adapters: Ipopt, POUNCE, KINSOL, HiGHS, Clarabel, Diffsol and IDAS, plus tears, presolve and quality ([§18](numerical-execution.md#section-18)) |
| `pse-ipopt-sys` | Generated raw Ipopt C bindings; the build script emits link directives only |
| `pse-columnar` | Arrow canonicalization, owned buffers, reservations, cancellation and native error adapters |
| `pse-schema` | The semantic registry, native field contracts, resolved contracts, compatibility and Delta layouts ([§4](schema-and-relations.md#section-4)) |
| `pse-codegen` | Pure registry generators; returns the generated tree without writing files |
| `pse-relations` | Generated Arrow views, builders, extension types and validators over checked batches |
| `pse-engine` | DataFusion session assembly, bound preparation and execution, validation, providers and resources |
| `pse-rules` | Registry-declared invariants as one native diagnostic query; its consumers are inspection and fixture validation |
| `pse-catalog` | Catalog providers, exact Delta publication, reopen, inspection and retention ([§20](identity-and-publication.md#section-20)) |
| `pse-runtime` | Composition root: document ingestion, physical inventory, the public workflow, jobs, budget and cancellation ([§19](workflows-and-results.md#section-19)) |
| `pse-buildinfo` | Build provenance: toolchain, profile, Git revision and embedded lockfile identity |
| `pse-py` | The PyO3 extension `pse._native`, which is the whole Rust/Python boundary ([§21](workflows-and-results.md#section-21)) |
| `pse-testkit` | Development-only fixtures that use production engine factories; never a production dependency |

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
`pse-quantity`, `pse-material`, `pse-model`) sit below `pse-authoring`, `pse-kernels`,
`pse-structural`, `pse-math`, `pse-compiler` and `pse-backend-native`. None of these
reaches Arrow, DataFusion, Delta or Tokio. The columnar crates (`pse-columnar`,
`pse-schema`, `pse-relations`) sit beside them and use Arrow plus the DataFusion leaf
expression APIs. `pse-engine` adds sessions. `pse-rules` and `pse-catalog` build on the
engine. `pse-runtime` is the only crate that joins the semantic/native side with the
columnar/storage side, and it owns every effect: document I/O, compilation, native
attempts and publication. `pse-py` depends on the runtime and the inspection crates.
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

Each library owns the operation it implements. PSE code owns the physical, identity and
admission contracts around the library call. A library never becomes the authority for
units, identity, schemas or canonical encoding merely because it is present.

| Library | Owner crate | Role | Boundary |
|---|---|---|---|
| Arrow, Parquet | columnar crates, engine, catalog | Checked columnar data, extension types, IPC and FFI transport | Physical layout safety is not semantic validity ([§4.6](schema-and-relations.md#section-4-6)) |
| DataFusion | `pse-engine`, `pse-rules`, `pse-catalog`, `pse-runtime` | Relational admission, invariant queries, inspection, DML and storage plans | Not a mathematical evaluator ([§3.3.1](#section-3-3-1)) |
| Delta Lake (`deltalake-core`), `object_store` | `pse-catalog` | Durable authored/result tables, exact control-last publication and retention | Settlement is PSE-owned ([§20](identity-and-publication.md#section-20)) |
| Symbolica/Numerica | `pse-math` | Algebra, normalization, differentiation, multi-output evaluators and jets | Starts after physical typing and domain obligations; derived artifacts only |
| faer | `pse-math`, `pse-backend-native`, `pse-runtime` | Sparse structure, refill maps, products; bounded LU/SVD for implicit responses and rank | Native solver factorizations stay with their solver |
| FeOS, `feos-core`, num-dual, `quantity`, nalgebra | `pse-kernels` | PC-SAFT/DIPPR thermodynamics and provider derivatives | FeOS's `quantity` is never a unit authority; dual types stay private to the provider |
| Ipopt (C ABI), POUNCE | `pse-backend-native` | Local NLP through one shared oracle | Separate native routes; no fallback between them |
| pounce-presolve | `pse-math`, `pse-structural`, `pse-backend-native` | Matching, DM, BTF and qualified presolve/postsolve | Original-coordinate recovery is validated independently |
| SUNDIALS KINSOL/IDAS (+ KLU) | `pse-backend-native` | Square roots, declared fixed-point iteration; IDAS for a narrow smooth recovery profile | Optional features; arbitrary boxes and unsupported profiles are refused |
| HiGHS | `pse-backend-native` | LP, MILP, certified convex QP and tear MILPs | Integrality is never relaxed silently |
| Clarabel | `pse-backend-native` | Explicit cones, with SDP under an optional feature | No implicit cone recognition |
| Diffsol | `pse-backend-native` | BDF dynamics for the admitted mass-matrix profile, events and forward sensitivities | No second model language |
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
([§18](numerical-execution.md#section-18)). JIT and SIMD evaluation, GPU execution and
general global MINLP are not admitted. The same holds for any library that has no current
consumer. That is a limit of the present scope, not a prohibition
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
| Selected-model admission and template specialization | Typed Rust over admitted rows | `pse-runtime::workflow::composition` |
| Physical typing, finite expansion, body preparation | Typed Rust, then Symbolica | `pse-compiler`, `pse-math` |
| Incremental semantic reuse | Salsa, pure and synchronous | `pse-compiler::workspace` |
| Thermodynamic properties and provider derivatives | FeOS and num-dual workers | `pse-kernels` |
| Structural analysis | pounce-presolve, rustworkx-core, petgraph | `pse-structural` |
| Numerical solution and integration | The class-specific native library | `pse-backend-native` |
| Sparse numerical linear algebra outside solvers | faer | `pse-math`, `pse-runtime` |
| Durable publication and retention | Delta through DataFusion | `pse-catalog` |
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
