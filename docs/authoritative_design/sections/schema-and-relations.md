---
title: Schema registry and relation families
status: current
---

# Schema registry and relation families

This page explains how durable meaning is declared, how the declarations become generated
Rust, Python and documentation, and what each relation family is for. `pse-schema` owns
the registry and its admitted contracts, and `pse-codegen` renders it. `pse-relations`
and `pse-model` hold the generated Arrow and plain-value projections. `pse-engine`,
`pse-rules` and `pse-catalog` enforce the relational obligations. Column-level detail
lives only in the [generated registry reference](../../generated/README.md); this page
explains ownership and purpose, not column lists.

## 4. The semantic schema registry

The registry is the one declaration of every durable relation, field, enumeration,
extension type, invariant, authoring document shape and native algorithm signature. The
catalog is assembled in a fixed order by `pse_schema::catalog`, with one module per
family. `RegistryBuilder::build` resolves cross-references and returns an immutable
`Registry`. There is no other description to keep in step. Everything downstream is
either generated from the registry or checked against it.

### 4.1 What is declared once

A relation is declared exactly once as a `RelationSpec` over native Arrow fields
(`pse_schema::model`). A `FieldContract` is a real Arrow `Field` that carries its domain
facets in field metadata, so nested children carry the same contract as top-level
columns. The facets are quantity, enumeration, foreign key, reference, collection,
tagged alternative and integer range. Each relation declares:

- its namespace;
- its authority: `authored`, `reference` or `derived`;
- its snapshot class: model, case, derived or sidecar;
- its primary key and stability.

Assembly rejects duplicate declarations, dangling references and key columns whose
equality is not an admitted exact scalar contract. Floating-point and quantity payloads
never acquire key semantics
([ADR-0030](../../adr/0030-canonical-float-hashing-and-no-float-keys.md)).

The registry also describes itself. The `reference.schema_*` relations are generated
from the same declarations, so the platform can query and diff its own schema:

- relations, columns and logical types;
- enumeration types and members;
- invariants and migrations;
- documents and document sections.

Other declarations have the same single owner:

- **Enumerations.** Closed dictionaries. The member name is stored, and a member ordinal
  is presentation only. IDAES-derived members carry their upstream name
  ([§6.14](#section-6-14)).
- **Invariants.** Unique, foreign-key, check, cardinality, domain, closure and acyclic
  obligations with a severity. They form the relational commit contract
  ([§4.6](#section-4-6)).
- **Documents.** A `DocumentSpec` maps each section of a package document to exactly one
  authored relation ([§22](models-and-composition.md#section-22)).
- **Algorithm signatures.** Typed arguments, results, effects and diagnostics of finite
  native domain algorithms. The registry holds no producer graph, stage scheduler or
  stored stage identity.
- **Migrations.** An explicit transformation between two named versions. Opening a store
  never discovers or applies a conversion path.

**Admitted contracts.** `resolved_contract::RelationContractHandle` binds a declaration
to the actual immutable registry that admitted it. A handle from an independent owner
is accepted only after `require_equivalent` compares every reachable semantic
declaration. Generated consumers check a compact, independently compiled expectation
once per owner (`require_generated`). Copied identifiers, names or digests never admit
foreign meaning. Registry and relation fingerprints identify content, not admission.

### 4.2 What is generated from it

> Decision: [ADR-0031](../../adr/0031-generated-sources-committed-and-diff-checked.md),
> [ADR-0051](../../adr/0051-generated-trees-and-regeneration-check.md)
>
> Decision: [ADR-0106](../../adr/0106-execution-vocabulary-discrete-and-global.md) — the
> shared execution vocabulary is generated from the registry.

`pse-codegen` returns a deterministic path-to-bytes tree and never writes files.
`cargo xtask codegen` writes the tree and removes stale files; `--check` compares a real
regeneration byte for byte. Generated sources are committed, never produced in
`build.rs`, and never edited by hand. Run `just codegen` when a declaration or generator
changes. `just codegen-check` proves equivalence, and `just codegen-bootstrap` orders the
bootstrap when the package loader itself depends on regenerated contracts.

| Generated tree | Content | Main consumers |
|---|---|---|
| `crates/pse-model/src/generated/` | Plain semantic rows, enums and extension values, without Arrow | compiler, workflow, native adapters |
| `crates/pse-relations/src/generated/` | Relation identities, Arrow views/builders/codecs, enum and algorithm-argument adapters | engine, catalog, runtime, `pse-py` |
| `crates/pse-authoring/src/generated/`, `crates/pse-runtime/src/generated/` | Strict document structs and document-to-row adapters | document loading ([§22](models-and-composition.md#section-22)) |
| `crates/pse-quantity/src/generated/` | Arrow-free physical fixtures projected from admitted reference packages ([§6.15.7](#section-6-15-7)) | tests and fixture registries |
| `python/pse/contracts/` | Contract classes, enums with IDAES bindings, `pyarrow` extension types, content digest | Python package ([§21](workflows-and-results.md#section-21)) |
| `docs/generated/` | Relation, enumeration, extension-type, algorithm and invariant reference; authoring JSON Schema | readers and editors |

`crates/pse-ipopt-sys/src/bindings.rs` is also a generated tree, but bindgen produces
it from the Ipopt headers, not from the registry.

Nothing hand-written may restate a generated row shape. The governance test
`no_shadow_structs` checks this. Where runtime reflection cannot give an equivalent
compile-time contract, typed generated accessors are kept, and the generator states that
reason.

### 4.3 Metadata conventions

Metadata is attached once, when a schema is constructed (`pse_schema::arrow`). It is
never patched afterwards, and nested children carry their own metadata.

| Level | Keys | Meaning |
|---|---|---|
| Schema | `pse.contract.id`, `pse.contract.version`, `pse.contract.fingerprint`, `pse.namespace` | Relation identity, version, declaration digest and namespace |
| Schema | `pse.contract.checks`, `pse.contract.delta_properties` | Named native SQL predicates; canonical Delta table properties |
| Schema (durable) | `pse.contract.semantic`, `pse.contract.semantic_format`, `pse.contract.execution_encoding` | Complete semantic witness, its interpretation format and the native layout identity ([§20](identity-and-publication.md#section-20)) |
| Field | `pse.semantic.logical_type`, `.quantity_type`, `.role`, `.fk`, `.enum`, `.reference`, `.collection`, `.tagged_alternative`, `.integer_range`, `.key_encoding` | Field facets of [§4.1](#section-4-1) |
| Field | `ARROW:extension:name`, `ARROW:extension:metadata` | Extension type and its mandatory metadata ([§4.4](#section-4-4)) |

Generated constructors produce the same key set for the same contract. They reject
unknown metadata; an unregistered key is an error, not something silently dropped.
Arrow field maps carry no ordering claim. Canonical hashing therefore serializes an
ordered metadata representation ([§5.3](identity-and-publication.md#section-5-3)).
Metadata and DataFusion `Constraints` describe facts; they do not enforce them. A hash
identifies content or detects corruption, and never discharges schema, key, reference,
physical-type or domain validity. Durable compatibility compares the recorded semantic
witness with an independently compiled expectation. Outcomes are typed: an unsupported
historical format or encoding is reported as requiring migration or unsupported, and is
never opened by a legacy reader (`pse_schema::compatibility`).

### 4.4 Extension types

`pse_schema::model::EXTENSION_TYPES` declares eleven `pse.*` extension types. Each has a
standard storage type, so a consumer that does not know the extension still reads the
storage safely. The [extension-type reference](../../generated/extension_types.md) gives
their storage layouts.

| Extension | Meaning |
|---|---|
| `pse.semantic_id` | 128-bit semantic identity ([§5.1](identity-and-publication.md#section-5-1)) |
| `pse.content_hash` | BLAKE3 digest of an immutable version |
| `pse.dimension_vector` | Rational exponents over eight base dimensions, including currency ([§8](physical-semantics.md#section-8)) |
| `pse.quantity_value` | Value with an explicit quantity type and unit, for heterogeneous columns |
| `pse.bound` | Explicit `finite` or `unbounded` bound; never an infinity or null sentinel |
| `pse.index_tuple` | Ordered domain-member identities |
| `pse.ordinal_ref` | Artifact-local reference; metadata names the target relation |
| `pse.source_span` | Document identity and checked byte range |
| `pse.enum` | Closed enumeration member; metadata names the enumeration |
| `pse.expr_dsl` | Explicit expression-text boundary; modeling expressions are fields of the generic declaration IR ([§7](mathematics-and-compilation.md#section-7)) |
| `pse.target_path` | Authored selector path, resolved to identities before use ([§6.10](#section-6-10)) |

Metadata has exactly three shapes: `{"v":1}`, `{"v":1,"enum_id":…}` and
`{"v":1,"target_relation_id":…}`. They are written by hand in fixed key order because
they enter logical hashes (`pse_schema::ext_metadata`). Factories reject any other
shape, and a new metadata version is how an extension evolves.

Enforcement happens in three places:

- **Rust.** `pse_relations::ext` implements Arrow's `ExtensionType` for every declared
  type, and reads use `try_extension_type`. The panicking accessor is banned in
  `clippy.toml`.
- **DataFusion.** `pse_engine::session::registry` registers every declared type beside
  Arrow's canonical types and refuses any replacement. Registration alone is passive, so
  `pse_engine::session::admission` actively checks fields at every platform logical-plan
  boundary, including subqueries and nested children. `pse_relations::ext::formatter`
  renders values meaningfully in plans and diagnostics.
- **Python.** `python/pse/contracts/extension_types.py` registers the generated
  `pyarrow` classes once. An unregistered `pse.*` type degrades to its storage type in
  Python, while Rust admission rejects an unregistered name. The contract layer records
  that difference ([§21](workflows-and-results.md#section-21)).

Metadata parsing can check the syntax of an ordinal target. Only bundle admission can
show that the target exists and that every ordinal is in range.

### 4.5 Logical scalar types

Fields use native Arrow types chosen for the meaning they carry:

| Meaning | Storage |
|---|---|
| Numerical values | `Float64` |
| Truth values | `Boolean` |
| Names, documentation, paths, enumeration members | `Utf8` |
| Collections and records | `List` and `Struct` |
| Identities, digests and the other domain values | The extensions of [§4.4](#section-4-4) |

Canonical PSE ordinals, counts and versions are stored as checked `Int64`, which Delta
also stores portably. `pse.semantic.integer_range` records the actual domain, for
example nonnegative ordinals or 32-bit source offsets, so storage width and meaning stay
distinct. Timestamps, where a relation records one, are UTC. A tagged alternative
(`model::TaggedAlternative`) declares a struct whose single active arm is selected by a
discriminator column. Inactive arms and children masked by an absent parent are not
values. Dictionary encoding is presentation only and never carries identity. The
`reference.schema_logical_types` relation is generated from these field contracts.

### 4.6 Construction properties and residual obligations

Admission happens in stages, and each stage establishes only its own facts:

1. **Physical layout safety.** Arrow structural validity, exact recursive field contracts
   and extension metadata are checked first (`pse_schema::field_contract`,
   `pse_relations::validate`). Storage-type compatibility or Arrow's `equals_datatype`
   never establishes a declared field, because both ignore names, metadata or nullability.
2. **Value and domain conditions.** Declared native predicates are bound once against
   the actual invocation (`pse_engine::session::checks`, `pse_engine::validation`).
3. **Relational obligations.** Keys, foreign keys, closure and coverage run as one native
   diagnostic query compiled from the registry (`pse_rules::invariants`). The same
   declaration-owned interpretation of keys and visible references
   (`pse_schema::obligations`) serves publication admission in `pse-catalog`.

A property holds only when one of three things establishes it:

- an exact admitted input owner;
- a sound construction rule over admitted inputs;
- successful completion of its residual obligation.

Otherwise it remains unknown. A projection keeps uniqueness only if a complete key
survives. A filter keeps row-local facts but drops completeness claims. A total lookup
needs non-null covered keys and exact unique-key inclusion on the other side. Negative
results from anti-joins depend on the complete right-hand scope. Computed fields declare
their meaning explicitly. Only complete obligations establish properties of the whole
result. Once an unchanged admitted owner has been validated, passing it through another
internal wrapper does not trigger a rescan.

Several kinds of equality stay distinct: SQL equality and grouping, exact typed value
equality, canonical mathematical equivalence and encoded byte identity. Diagnostics
distinguish false, NULL, absent and conflicting values, and bag multiplicity is kept
where the contract requires it. An optimizer is never given an unproved constraint while
that constraint's own validation query is running. Native checked builders and analyzers
establish their stated facts, not application correctness. The failure codes belong to
[§23.2](operations-and-validation.md#section-23-2).

## 6. Relation families

> Decision: [ADR-0098](../../adr/0098-modeling-knowledge-ownership.md),
> [ADR-0099](../../adr/0099-modeling-language-and-identities.md) (proposed; implementation authorized).

The registry declares generic modeling IR, physical quantities, observation inputs and
execution/publication contracts. Scientific concepts are package declarations in
`authored.modeling_declarations`. They do not acquire separate registry families.
The generated reference owns exact fields and versions. Incompatible historical contracts
require an explicit migration; the runtime retains no legacy scientific reader.

The section identities below remain stable while the mechanisms they describe change.

### 6.1 Identity, packages and source edits

`authored.packages` records manifest identity, version, policy and exact dependencies.
`authored.documents` retains exact source bytes. `authored.entities` registers identities
exposed by the remaining YAML physical and observation documents. Generic modeling
entities and their kinds are declarations in the modeling IR, with explicit IDs and
parent identities; they do not extend the platform's closed `EntityKind` dictionary.

`normalized.package_graph` resolves the complete manifest closure in dependency order,
including isolates. Missing dependencies, duplicate meanings and incompatible versions
refuse admission. Source editing uses exact document before-images in the authoring driver;
the retired template rename/binding request relations are gone. Explicit modeling IDs
survive name edits; names and source byte ranges are separate from identity.

### 6.2 Physical types

The reference relations declare dimensions, units, unit sets, quantity kinds, bases,
reference states, quantity types, conversions, quantity operations and constants.
`quantity_kinds` (version 2) carries the optional count or indicator category of a
dimensionless kind ([§8.1](physical-semantics.md#section-8-1)).
`quantity_preconditions` and `quantity_operation_reductions` carry the prerequisites
checked against actual operands. `math_context` explicitly names the neutral scalar
and Boolean kind.

Quantity axes and subjects name package-declared entity kinds. A reference state's
optional datum subject points to an actual authored entity; a kind or arbitrary missing
identity is refused by physical admission. `authored.package_quantity_aliases` scopes
physical type spellings to manifest-owned modeling packages. These declarations enter
one immutable physical inventory, retained by checked modeling products (§8).

### 6.3 Domains and index sets

Finite, ordered, derived and ragged sets and continuous axes are generic declarations.
Checking resolves their entity kinds, membership, bounds and physical coordinates.
Specialization expands only demanded finite members within explicit limits. Discretization
consumes authored stencil or collocation data. The old `domains`, `domain_members` and
continuous-domain relation families are removed.

### 6.4 Material systems and reactions

Chemistry packages declare entity kinds, entities, attributes, sets, tables and datasets.
Composition, molecular weights, allowed phases, stoichiometry and elemental closure are
knowledge expressed through those contracts. The kernel contains no scientific species,
phase, element or reaction type and no material-specific relational validator.
The seed's chemistry and reaction fixtures exercise those declarations (§9).

### 6.5 Property and method declarations

Functions declare correlation equations and domains; interfaces declare property contracts
and defaults; definitions bind methods, parameters and datasets. Dispatch tables select
implementations for members of declared sets. These are ordinary modeling declarations.
The old property/method-selection and provider-data relation families are removed.
Execution follows checked specialization and the shared mathematical path (§9.3, §9.6).

### 6.6 Definitions and interfaces

Reusable definitions and interfaces replace the template relation families. One tagged
modeling IR carries parameters, variables, functions, equations, children, ports, accumulators,
contributions, guards, annotations, requirements and fixtures. Payload tags are interpreted
by the generic modeling kernel; scientific names remain authored data. Parent identities
retain lexical ownership, and source ranges retain attribution.

### 6.7 Instances, compositions and connectivity

Child declarations instantiate checked definitions with explicit arguments. Indexed children
use admitted set membership. Cases select roots and overlays without mutating definitions.
Connection occurrences retain their source identities and port ownership after equality
expansion. Selected graph projections include explicit isolates and tear policies (§12).
The old instance/composition/connection relation families have no remaining production path.

### 6.8 Symbol declarations and roles

> Decision: [ADR-0103](../../adr/0103-variable-domain-facet.md) — variables gain a
> declared domain facet (`continuous`, `integer`, `binary`, `semicontinuous`,
> `semiinteger`) with per-mode semantics (Plan 22 M1 and the M2 refusals, implemented;
> the M2 fixed-assignment stage is not yet implemented).

Variables, parameters and value members are tagged modeling declarations. Specialized
scalar identities derive from declaration, instance and admitted membership, never display
labels or native slots. The compiler resolves source paths to these identities and builds
library-owned mathematical bodies. Symbol tables, derivative layouts and native coordinates
are preparation products, not independently authored relations.

**Domain facet.** A variable declares its domain after its type, as in
`var on[t in periods]: Indicator in binary;`, with `integer`, `binary`, `semicontinuous`
or `semiinteger`. `continuous` is the default: omitting the facet and writing
`in continuous` are the same declaration, and rendering prints no facet for it. Only `var`
takes a facet; one on a parameter or value member is a parse error. The registry enum
`ModelingVariableDomain` is the single authority from source to the native boundary. The
binding arm of `authored.modeling_declarations` (version 2) stores it, present exactly on a
variable; the compiler, `ProblemFacts` and HiGHS consume the generated type directly; and
`runtime.solve_variables` (version 3) states it in its `domain` column on every variable
row, while parameter rows carry none. An interface member's override or implementation
keeps the member's domain; a different one is refused at checking.

Integer and binary variables are dimensionless counts or indicators, while semi domains
keep their physical quantity ([§8.1](physical-semantics.md#section-8-1)). The language's
former built-in `Count` synonym of `Integer` is removed, so `Count` names the physical
count type (the physical bundle's alias); `Integer` remains the built-in integer type.

**Admission and analysis modes.** After case binding, every free discrete variable needs
finite bounds that admit a value of its domain
([§7.5](mathematics-and-compilation.md#section-7-5)). A free discrete variable is an
ordinary free column in structure and degrees of freedom. Each analysis states what it does
with one:

| Analysis | Free discrete variable |
|---|---|
| Steady optimization | A decision: an authored MILP routes to HiGHS ([§18.1](numerical-execution.md#section-18-1)); MIQP and MINLP routes are not yet implemented |
| Root solve | Refused; when the case fixes every discrete variable, the solve is continuous |
| Initialization | Refused; the stage that fixes discrete variables as a scoped overlay (Plan 22 M2) is not yet implemented |
| Fitting | Refused unless the experiment's case fixes it |
| Integrated dynamics | Refused unless the case fixes it; a fixed one is a parameter of integration, piecewise constant through profile changes ([§13.5](workflows-and-results.md#section-13-5)) |
| Nested implicit root | A discrete unknown is refused (analysis `root`) |

Duals and sensitivities stated conditional on a fixed assignment (Plan 22 M2) are not yet
implemented. Each refusal is a typed `pse_modeling::ModelingError::Domain` naming the
variable (identity, declaration and instance path), its domain, the analysis
(`preparation`, `root`, `initialization`, `fitting` or `integrated_dynamics`) and the
violated rule; its diagnostic rule is `modeling.domain`
([§23.2](operations-and-validation.md#section-23-2)).

### 6.10 Cases, observations, dynamics and fitting

Case and test scopes in the modeling IR carry root bindings, values, fixed/free state,
bounds and analysis choices. Initialized, steady, integrated and simultaneous fixture routes
share the same definitions. A fixture retains expected outcomes, oracle provenance and
physical/relative tolerances. Conformance records derivative sampling as not applicable
while free discrete variables remain: sampling perturbs a continuous oracle, and
integrality is never relaxed implicitly. The seed price-taker fixture
(`packages/reference/seed-data/models/price-taker.pse`) is an authored MILP whose linear
relaxation exceeds its optimum, so it shows that integrality is enforced.

`authored.datasets` and `authored.observations` supply measurements. `authored.fit_cases`
binds authored modeling experiments and source paths to the existing sparse fitting engine.
Null measurements remain absent. Targets resolve through the admitted model, including
indexed membership; the legacy template target parser and `dynamic_cases` are removed.
Studies and sequences keep each requested solve and its observations distinct (§19).

### 6.11 Numerical requirements and native algorithm signatures

`authored.numerical_requirements` declares selected targets, magnitude units and frozen
relative budgets. Authored annotations and numerical profiles contribute through the same
policy resolver. `runtime.resolved_numerics` records actual sources and conversions;
projection into blocks or recycle coordinates adds no second precedence rule (§16).

`reference.algorithm_specs`, `algorithm_arguments` and `algorithm_results` declare native
ports, determinism, effects and diagnostics. `engine_profiles` records engine settings;
`function_capabilities` projects the actual built-in handler inventory. External functions
bind explicit registered capabilities and derivative contracts. Neither registry declarations
nor capability names alone make an implementation executable.

### 6.13 Execution results, publication and evidence

> Decision: [ADR-0108](../../adr/0108-ipopt-linear-solvers-and-solver-image.md) — the
> solver image's linear-solver inventory and determinism settings are recorded with
> results (Plan 22 N1; not yet implemented).

The `runtime` namespace holds immutable result and control facts. Each group has its own
owning section:

| Group | Relations | Owner |
|---|---|---|
| Runs and results | `computation_runs`, `solve_runs`, `solve_variables`, `solve_constraints`, `solve_metrics`, `candidate_assessments`, `modeling_checks`, `modeling_reports`, `modeling_conformance`, `modeling_fixture_status`, `modeling_findings`, `simulation_samples`, `simulation_events`, `fit_*`, `response_sensitivities`, `run_lineage`, `resolved_numerics` | [§19](workflows-and-results.md#section-19) |
| Capability inventory | `solver_capabilities` | [§18](numerical-execution.md#section-18) |
| Publication and retention | `publications`, `artifact_descriptors`, `release_checkpoints`, `native_dependencies`, `change_events`, `maintenance_outcomes`, `retained_versions`; `reference.artifact_profiles` | [§20](identity-and-publication.md#section-20) |
| Observation | `diagnostics_findings`, `validation_findings`, `cache_statistics`, `cache_entry_statistics`, `execution_statistics` | [§23](operations-and-validation.md#section-23) |

Several facts that could be conflated are kept apart:

- native termination, numerical acceptance, physical closure and final usability;
- a missing observation and a zero;
- a missing multiplier and a zero multiplier;
- an absent later state (for example after a failed transition) and a recorded one.

Observation relations are volatile evidence, never model or dependency authority.
`provenance.derivations` records derivation evidence. `provenance.assertions` holds
authored expected evidence and is excluded from semantic membership.

### 6.14 Enumerations preserved from IDAES

> Decision: [ADR-0003](../../adr/0003-clean-room-relationship-and-parity-pin.md)

The compatibility package (`packages/reference/physical/models/compatibility.pse`)
owns scientific enum names as ordinary modeling data. Only `ConstraintScalingScheme`,
which selects a numerical algorithm, remains in `s6_14_idaes_enums`. The parity harness's
`enum-bindings.toml` maps declarations to upstream modules and necessary spelling aliases;
it does not repeat their member inventories. The sanctioned names are:

| Group | Enumerations (IDAES module) |
|---|---|
| Balances and flow | `MaterialBalanceType`, `EnergyBalanceType`, `MomentumBalanceType`, `FlowDirection` (`core.base.control_volume_base`); `MaterialFlowBasis` (`core.base.process_base`); `DistributedVars` (`core.base.control_volume1d`) |
| Components and phases | `ComponentType` (upstream class names), `PhaseType` |
| Modular properties | `StateIndex`, `ConcentrationForm`, `HenryType`, `CubicType` |
| Unit models | `FlashType`, `MixingType`, `MomentumMixingType`, `SplittingType`, `EnergySplittingType`, `ThermodynamicAssumption`, `ValveFunctionType`, `HeatExchangerFlowPattern` |
| Control | `ControllerType`, `ControllerMVBoundType`, `ControllerAntiwindupType` |
| Scaling and initialization | `ConstraintScalingScheme`, `DefaultScalingRecommendation`, `InitializationStatus` |
| Dynamics | `DaeVarTypes`; `DiscretizationScheme` (Pyomo DAE spellings, such as `LAGRANGE-RADAU`) |
| Costing | `HXType`, `HXMaterial`, `HXTubeLength`, `VesselMaterial`, `TrayType`, `TrayMaterial`, `HeaterMaterial`, `HeaterSource`, `CompressorType`, `CompressorDriveType`, `CompressorMaterial`, `PumpType`, `PumpMaterial`, `PumpMotorType`, `FanType`, `FanMaterial`, `BlowerType`, `BlowerMaterial` |

`HenryType` preserves the four physical forms and excludes upstream's test-only `Dummy`.
An enum's existence makes no execution claim. Package bindings and their conformance
fixtures establish which formulations have implementations. The parity test admits the
actual modeling package and compares its names with IDAES 2.13.0; this establishes naming
parity only, not numerical equivalence.

### 6.15 Semantic specialization contracts

> Decision: [ADR-0098](../../adr/0098-modeling-knowledge-ownership.md),
> [ADR-0099](../../adr/0099-modeling-language-and-identities.md),
> [ADR-0100](../../adr/0100-modeling-functions-and-accounting.md) (proposed).

`pse-modeling` owns pure checking and specialization over generated declaration values.
`pse-compiler` owns the single Salsa workspace and mathematical lowering. Runtime package
admission supplies the immutable dependency closure and physical context. Checking,
specialization and numerical execution have separate owners and lifetimes.

#### 6.15.1 Typed configuration and finite scopes

Checking admits physical and polymorphic signatures, entity/set/table contracts, defaults,
interface conformance and requirement predicates before instantiation. Visibility follows
explicit imports, including aliases and generated function spellings. Runtime values cannot
choose structural guards. Unresolved configuration refuses rather than selecting a default.
Finite expansion and body construction have explicit resource policies; exhaustion never
silently truncates a model.

#### 6.15.3 Property demand and method selection

Demand closure pulls a requested member together with its defining equations and dependencies.
Effective members follow explicit inheritance, implementation and override rules. Shared
descendant refinements remain unambiguous; competing sibling defaults require an override.
Dispatch groups use actual implementation bindings and definition-scoped functions. No
scientific name selects a special runtime implementation.

#### 6.15.4 Indexed realization of declarations

Specialization binds each indexed occurrence to the admitted ordered membership. Physical
reductions preserve the contracted kind, including empty-set prototypes. Child instances,
function slots, continuous coordinates and structural projections use the same checked
contracts. Source lineage remains attached through lowering and diagnostics. Accumulator
terms enter the existing sparse assembler as signed contributions, preserving independent
closure checks (§10).

#### 6.15.5 Connections and flowsheet topology

Typed ports and connection declarations are checked and expanded in the modeling kernel.
Their mathematical equalities and selected flow graph derive from the same occurrences.
Flow analysis retains explicit node/edge selection and tear policies. Native graph and
fixed-point libraries own tear selection and recycle iteration; the runtime does not
reconstruct topology from variable labels (§12, §17).

#### 6.15.7 Shipped reference packages and generated physical fixtures

The [reference package guide](../../../packages/reference/README.md) identifies current
physical, method, thermodynamic, seed, process and diagnostic bundles. Their `sources.md`
files distinguish published data, upstream comparison inputs and derived demonstrations.
Authored fixtures live beside the knowledge and use the common conformance harness.

`fixture-projection.toml` selects the physical and synthetic-currency bundles for generated
Arrow-free quantity fixtures. Code generation loads and admits those actual documents.
Elemental knowledge is authored modeling data; `pse-material` and its generated fixture
copy have been removed. Independent oracle inputs remain test references.

## Retired section identities

#### 6.9 Compiled mathematics relations — retired

Mathematics is no longer stored as relations: authored expression text lowers directly
to library-owned Symbolica bodies ([§7](mathematics-and-compilation.md#section-7),
[ADR-0082](../../adr/0082-library-owned-process-mathematics.md)).

#### 6.12 Derived structure relations — retired

Incidence, matching and block structure are computed per preparation from complete
projections and are not persisted ([§15](numerical-execution.md#section-15)).

#### 6.15.2 Rule workspace, truth and support — retired

The generic rule fixed-point workspace was deleted; declared invariants run as native
queries ([§4.6](#section-4-6)), and selected-model admission replaces rule-derived
realization ([§6.15](#section-6-15)).

#### 6.15.6 Pass ownership and publication — retired

Stage-by-stage pass ports were replaced by selected-model admission and pure compiler
preparation ([§14](mathematics-and-compilation.md#section-14)).
