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
| `crates/pse-quantity/src/generated/`, `crates/pse-material/src/generated/` | Arrow-free physical fixtures projected from admitted reference packages ([§6.15.7](#section-6-15-7)) | tests and fixture registries |
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
| `pse.expr_dsl` | Authored expression text, the only authored form of an expression ([§7](mathematics-and-compilation.md#section-7)) |
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

The [generated relation reference](../../generated/relations/authored.md) owns columns.
Five namespaces carry relations today:

| Namespace | Holds | Reference |
|---|---|---|
| `reference` | Shipped contracts and reference data: registry self-description, physical types, method/law/connection bindings, algorithm signatures, artifact profiles | [reference](../../generated/relations/reference.md) |
| `authored` | Facts written by people, agents or typed builders: the only writable model namespace | [authored](../../generated/relations/authored.md) |
| `normalized` | Deterministic projections of authored sources: package graph, source occurrences, instance bindings, units | [normalized](../../generated/relations/normalized.md) |
| `runtime` | Results, publication control, retention and observations | [runtime](../../generated/relations/runtime.md) |
| `provenance` | Derivation evidence and authored expected evidence | [provenance](../../generated/relations/provenance.md) |

The model vocabulary still names `inferred` and `compiled` namespaces, but no relation
is declared in either. Compiled mathematics, symbol tables, structure and prepared
artifacts are non-durable products of the compiler and the runtime
([§7](mathematics-and-compilation.md#section-7),
[§14](mathematics-and-compilation.md#section-14)). Stored results record their
dependencies instead of the compiler's internal state.

**Selected admission.** A family's existence does not make it executable
([ADR-0088](../../adr/0088-selected-model-and-physical-contracts.md)).
`pse_runtime::workflow::composition` works out the selected model's dependency closure,
then classifies every stored declaration in one of four ways:

1. It belongs to the executable composition vocabulary (`CompositionDeclarations`).
2. It belongs to a family that the workflow executes.
3. It is descriptive, such as packages, entities, documents, display rows and
   assertions.
4. It is some other declaration. If it references a selected identity, admission refuses
   it with those identities. If it does not, admission reports it as nonexecuting.

The public admission report accounts for every relation. The limits stated under each
family below follow from this rule.

### 6.1 Identity, packages and source edits

`authored.packages` records the version, identity policy and exact dependencies of each
package. `authored.documents` lists every source document, and every source span points
into one of them. `authored.entities` holds entities that are declared explicitly;
registration checks their exact identity, package, kind, name and parent. Names are
attributes, never identity. `reference.aliases` keeps deprecated qualified names for
named-policy entities, because under that policy a rename creates a new entity
([§5.1](identity-and-publication.md#section-5-1)). `normalized.package_graph` holds the
exact package resolution (`pse_authoring::p0`) in dependency order, including isolates.
A missing dependency is refused. `authored.document_edits` and
`authored.rename_requests` are typed requests for native source edits. They bind exact
before-images and expected names, and the authoring driver owns the resulting writes.
Composite relationship keys are not entities by implication.

### 6.2 Physical types

The `reference` relations for dimensions, units, unit sets, quantity kinds, bases,
reference states, quantity types, conversion rules, quantity operations and constants
are the persisted physical vocabulary. `reference.quantity_preconditions` and
`reference.quantity_operation_reductions` declare operation prerequisites. These are
proved from exact operand contracts at the point of use, never from an invariant ID, and
a failure never falls back to a shape-only reduction. `reference.math_context` names
the package's neutral scalar quantity and Boolean kind; neither is guessed from a
dimension. `authored.package_unit_sets` selects a package's representation units
explicitly, and absence is not a default.

The quantity registry requires an explicit neutral dimensionless type, and equal
dimensions never select a physical kind. Conversion rules are typed:

- a scale rule has a finite scale and no offset;
- an affine rule has a finite scale and a finite offset;
- a kernel rule names a kernel and carries neither coefficient.

`pse-quantity` owns the semantics ([§8](physical-semantics.md#section-8)).

### 6.3 Domains and index sets

`authored.domains` declares finite domains and `authored.domain_members` lists their
members. Member ordinals are unique within a domain, and member identities are
semantic IDs rather than labels. A domain marked continuous must have exactly one
`authored.continuous_domains` row, and its unit must equal that row's unit. A discrete
domain has no continuous detail. The native composition route specializes finite
domains only. It refuses continuous domains, and it refuses parent-dependent domains
that lack explicit ragged tuple bindings. Time in dynamic cases is declared separately
([§6.10](#section-6-10), [§13](workflows-and-results.md#section-13)).

### 6.4 Material systems and reactions

`reference.elements` and the authored relations for species, species elements, phases,
phase species, material systems, reactions and stoichiometry are the material
vocabulary. `pse-material` holds their in-memory shape and predicates. For the selected
case, admission loads the material systems named by reaction applications and native
providers, together with their species, phases and reactions. Species and phase types
use the preserved IDAES dictionaries ([§6.14](#section-6-14)).

`authored.reaction_applications` selects a homogeneous molar reaction for a case. The
authored rate is an extent per unit time. Species source terms come from stoichiometry,
and an explicit signed heat-rate output supplies the energy term, so nothing is counted
twice. `authored.native_providers` binds explicit PC-SAFT/DIPPR records to ordered
species, physical port roles and a selected formulation
([§9](physical-semantics.md#section-9)).

`henry_declarations`, `reaction_methods`, `reaction_packages` and `parameter_values`
remain authoring vocabulary with no current execution interpretation. A selected row in
one of these relations is refused unless the physical inventory itself admits that
relation as source data.

### 6.5 Property and method declarations

`reference.property_kinds` and `reference.method_specs` form the property capability
vocabulary, together with the method relations of [§6.15.3](#section-6-15-3).
`authored.property_packages` and `authored.method_selections` declare package and method
choices. `authored.state_bounds` and `authored.phase_equilibrium_pairs` record state
limits and equilibrium pairings. None of these is an execution route.

The executable property path is an explicit native provider binding
([§6.4](#section-6-4)). An instance that names a property or reaction package is
refused. `authored.default_scaling` is consumed, as the property-default source of
numerical policy ([§6.11](#section-6-11)).

### 6.6 Templates

> Decision: [ADR-0010](../../adr/0010-laws-are-templates-over-contributions.md)

A template is a declarative, reusable model: unit, control volume, state, connection
rule, law, scaler or initializer. The `authored.templates` family and its companions
declare typed parameters, features and feature rules, guards, finite domains and
domain bindings, symbols, equations, ports and port members, contributions, submodels,
law instances and law contracts. Templates stay compact until a selected instance
specializes them; each specialization keeps its template, instance and source mapping.

Laws are templates over contributions. A law contract selects exactly one declared
binding in `reference.law_bindings`; a balance choice without a binding is unsupported.
A contribution declares its subject as fixed or as a declared axis, never inferred from
a name. `pse_runtime::workflow::composition::lower` performs the specialization
([§6.15](#section-6-15), [§11](models-and-composition.md#section-11)).

Scaler, initializer, connection-rule and law templates are not process instances.
Default scaler and initializer references are refused until a numerical policy admits
them. The following companions have no current execution interpretation, so a selected
row is refused:

- requirements and property requirements;
- symbol contracts, symbol properties and symbol expressions;
- derivatives;
- material constraints;
- contribution contracts.

Display rows are descriptive.

### 6.7 Instances, compositions and connectivity

`authored.instances` records template instances. Containment is expressed through
`parent_instance_id`. `authored.model_compositions` names the root instance of a
selected composition. Admission selects the root and all of its descendants, and
refuses a root that has a parent. `authored.instance_domain_bindings` maps an instance's
template-local domain name to an actual domain. `authored.connections` joins two ports
under a connection-rule template, with tear cost and policy
([§6.15.5](#section-6-15-5)). `authored.flowsheets`, `authored.scopes` and
`authored.selector_terms` are declared vocabulary without a current execution
interpretation. Specialized convenience constructors, such as the vessel in
`pse_runtime::workflow::vessel`, produce ordinary inspectable declarations.

### 6.8 Symbol declarations and roles

Symbols are authored: `authored.template_symbols` declares each symbol's role, quantity
type, index axes and guard. The `SolverVariableType` and `VariableSemanticRole`
vocabularies classify solver and semantic roles. Symbol tables are not stored. A scalar
occurrence's identity is framed from the instance, the declaration and the ordered index
members (`pse_runtime::workflow::composition::symbol_id`), so labels never enter it. The
native route specializes variable and parameter symbols. Reference, derivative and
expression roles are refused until a declared expression or dynamic binding supplies
them. Model-level dynamics use `authored.computation_models` and
`authored.physical_balances` ([§13](workflows-and-results.md#section-13)).

### 6.10 Cases, observations, dynamics and fitting

> Decision: [ADR-0016](../../adr/0016-cases-and-results-never-mutate-the-model.md)

Cases specify and observe a model; they never change it. The native workflow executes
these declarations:

- the cases carried by `authored.computation_models`;
- template `case_specs` for the selected cases;
- `authored.datasets` and `authored.observations`;
- `authored.dynamic_cases`: semi-explicit dynamics in canonical seconds with an explicit
  time origin;
- `authored.fit_cases`: simultaneous fitting with explicit observation time semantics
  ([§19](workflows-and-results.md#section-19)).

Case specifications, activations and observations target instances through
`pse.target_path`. The authoring driver (`pse_runtime::authoring_driver::targets`)
resolves each target, through the explicit instance domain binding, to a complete target
row: one instance plus exactly one member alternative. The target must name a member ID,
an unambiguous label or an exact finite coordinate. A missing binding, wrong arity,
duplicate dimension or unknown or ambiguous member is a typed failure. Labels and
coordinates are never hashed into an identity. A missing measurement stays a null row
and never becomes zero.

The rest of the case family is declared vocabulary with no current execution
interpretation:

- `cases`;
- `case_activations`, `case_objectives` and `case_policies`;
- `scenarios`;
- `case_sets` and `case_set_samples`.

No tabular observation importer exists. Observations enter as authored rows through
documents or builders.

### 6.11 Numerical requirements and native algorithm signatures

`authored.numerical_requirements` declares numerical meaning: selected targets,
magnitude units and frozen relative budgets. `authored.provider_scaling_bindings` binds
a provider output explicitly to a numerical target and a property default. The runtime
resolves one numerical policy from analysis, case, model, property defaults, quantity
nominals and a recorded canonical fallback. It records the result in
`runtime.resolved_numerics` ([§16](numerical-execution.md#section-16)).

Three `reference` relations describe native algorithms:

- `algorithm_specs`, `algorithm_arguments` and `algorithm_results` declare typed ports,
  determinism, effects and diagnostics. They are listed in the
  [algorithm reference](../../generated/algorithms.md).
- `engine_profiles` records engine settings.
- `function_capabilities` projects the actual typed handler for each authored function
  name and marks unsupported syntax as unavailable.

The registry holds no rule declarations and no executable strata.

### 6.13 Execution results, publication and evidence

The `runtime` namespace holds immutable result and control facts. Each group has its own
owning section:

| Group | Relations | Owner |
|---|---|---|
| Runs and results | `computation_runs`, `solve_runs`, `solve_variables`, `solve_constraints`, `solve_metrics`, `candidate_assessments`, `physical_checks`, `simulation_samples`, `simulation_events`, `fit_*`, `response_sensitivities`, `run_lineage`, `resolved_numerics` | [§19](workflows-and-results.md#section-19) |
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

These closed dictionaries keep IDAES names so that parity can map in both directions.
They are declared in `pse_schema::catalog::s6_14_idaes_enums`, and each member carries
its upstream spelling as `idaes_name`. The [enumeration reference](../../generated/enums.md)
lists the members.

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

`HenryType` preserves the four physical Henry forms and excludes IDAES's test-only
`Dummy` member. Where IDAES declares a member with no implementation, the platform
declares it too. Using such a member is unsupported unless a law binding exists. For
balance types, the shipped stock bindings in `packages/reference/units/laws/balances.yaml`
define which members execute. `python/pse/parity/tests/test_enum_contracts.py` checks the
generated names against the pinned parity environment. That check establishes naming
parity only, not numerical equivalence
([relationship to IDAES](../../relationship-to-idaes.md)).

### 6.15 Semantic specialization contracts

> Decision: [ADR-0088](../../adr/0088-selected-model-and-physical-contracts.md)

These contracts govern how a selected reusable composition becomes compiler input. The
work is done by typed Rust over admitted rows, in
`pse_runtime::workflow::composition::{select, lower}`. It is neither a pass pipeline nor
a rule engine. Lowering checks every declaration and then hands it on. It produces three
things for `pse-compiler`:

- inspectable source-to-scalar bindings;
- balance declarations;
- a flowsheet projection.

Mathematical typing stays with the compiler.

#### 6.15.1 Typed configuration and finite scopes

Configuration values share one tagged physical shape,
`catalog::s6_15_semantic::config_value_type`. Its arms are Boolean, signed, unsigned,
real, text, semantic ID, enumeration, index tuple and quantity, and exactly the selected
arm is present. Specialization then applies these rules:

- **Parameters.** Each value is checked against its declared logical type, enumeration
  and domain constraint. Integers must be exact and reals finite. A missing required
  parameter, an unknown name or a duplicate binding is invalid. A declared default is
  used only where it is declared.
- **Features.** A feature is a Boolean or an enumeration. Inheritance requires an
  explicit resolved binding; a missing value is never read as false. A feature rule must
  hold, and a feature and a parameter cannot share a name.
- **Guards.** A guard is evaluated only over resolved compile-time values. An unresolved
  or null guard is invalid.
- **Domains.** Each template domain is bound through exactly one arm of
  `authored.template_domain_bindings`: an explicit domain, a typed domain parameter or a
  material-derived source. Material-derived domains need an admitted material-system
  binding.
- **Size.** Finite specialization refuses expansions beyond its declared member
  allowance instead of truncating them.

#### 6.15.3 Property demand and method selection

The registry declares a complete method vocabulary in `reference`:

- parameters, parameter axes and state parameters;
- dependencies and precedence;
- provisions and kernel inputs;
- element projection contracts.

`authored.method_selections` and `authored.property_packages` declare the choices. No
general property-demand closure or method-resolution engine runs today. A selected model
that depends on these declarations is refused, as is an instance bound to a property or
reaction package ([§6.5](#section-6-5)). Properties execute only through an explicit
native provider binding with a typed physical contract
([§9](physical-semantics.md#section-9)). Automatic method selection is not implemented
and needs its own owner and decision before it is claimed.

#### 6.15.4 Indexed realization of template declarations

An active symbol or equation is realized over the ordered product of its declared finite
domains, so each scalar has an identity framed from its instance, declaration and member
tuple ([§6.8](#section-6-8)). Four checks apply:

- The symbol's axes must match the shape of its physical quantity, including the domain
  kind.
- Equation text must agree with the declared sense.
- A case target must resolve to an active scalar in the selected composition.
- An enabled submodel's children must be authored explicitly. A disabled submodel must
  have no active children, and submodel multiplicity domains are unsupported.

Law instances expand over their active contributions into balance declarations. A law
with no active contribution is invalid. `authored.instance_equations` has no current
execution interpretation. `normalized.instance_bindings` is the document-binding
projection of prospective roots and children used by source binding
(`pse_runtime::authoring_driver::document::binding`).

#### 6.15.5 Connections and flowsheet topology

A connection binds two ports of instances in the selected composition. Its rule template
must map, through `reference.connection_bindings`, to an admitted expansion; `equality`
is currently the only one. Port members come from the bound state template's complete
interface (`authored.template_port_members`). Port identity is framed from the instance
and the port name. Admission refuses:

- a direction, kind or member mismatch;
- more than one assignment to an inlet;
- extensive-flow fan-out without an explicit splitter conservation formulation.

Information (signal) connections stay distinct from physical ones. Tear cost and policy
are declared per connection, and grouped tear members must agree. Lowering produces a
`pse_structural::flowsheet` projection that keeps isolates and connection occurrences.
Tear selection happens later, at preparation time, rather than as a stored fact
([§12](models-and-composition.md#section-12),
[§17](numerical-execution.md#section-17)).

#### 6.15.7 Shipped reference packages and generated physical fixtures

The packages under `packages/reference` are ordinary authored and reference inputs,
admitted by the same loader as user packages:

- elements;
- physical units, quantities and operations;
- method, state and unit templates;
- thermodynamic examples;
- a synthetic-currency fixture.

Each package's `sources.md` records its references and qualification limits.
`packages/reference/fixture-projection.toml` selects the packages that code generation
admits and projects into the Arrow-free `pse-quantity` and `pse-material` fixtures. No
second hand-written table of the same physical facts exists. The stock templates and
method declarations serve as source fixtures for loading, binding, inspection and
conservation tests. Registering them does not make them an executable property route,
and it does not certify numerics or derivatives.

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
