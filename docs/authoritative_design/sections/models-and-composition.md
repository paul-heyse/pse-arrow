---
title: Models, composition and extension
status: current
---

# Models, composition and extension

This page owns how reusable process models are declared, specialized, connected and
extended. Authored templates, instances and connections remain the model authority; the
workflow's selected admission lowers them into ordinary `authored.computation_models`
definitions and `authored.physical_balances` rows, which the compiler then admits like any
other declaration. Owners in source: `pse-runtime::workflow::composition` (selection and
lowering), `workflow::balances`, `workflow::model` (builders and immutable revisions),
`pse-structural::flowsheet` (flow projection), `pse-runtime::authoring_driver` and
`pse-authoring` (documents, identities, edits), `pse-schema::catalog::documents` (package
surface) and the declarations under `packages/reference`.

## 10. Conservation laws and control volumes

> Decision: [ADR-0010](../../adr/0010-laws-are-templates-over-contributions.md),
> [ADR-0088](../../adr/0088-selected-model-and-physical-contracts.md)

Balances are declared once as a law over signed contributions
([D7](architecture-overview.md#section-d7)). A template states which contributions exist
and which law owns them; lowering assembles the balance, and an independent closure check
sums the same declared sources. No unit contains a handwritten copy of its balance.

### 10.1 Law instances over contributions

A template's `template_law_instances` row names a law template, a scope label, an optional
guard and its options. Its `template_law_contracts` row fixes the balance quantity type and
the typed balance enumeration; the reference `law_bindings` row selects the expansion,
family and subject kind. For each selected case, lowering emits one
`authored.physical_balances` row whose terms are the active contributions of that instance
with the same scope and source family. `workflow::balances` then derives the compiler row
from those terms only.

Invariants enforced at freeze:

- every active contribution has exactly one owning law; an active contribution without a
  selected law, or a law without an active contribution, is refused;
- each contribution's admitted quantity equals the law's quantity; the compiler never sums
  incompatible quantities;
- the law carries exactly one option, an explicit finite positive `tolerance`; there is no
  implicit default;
- a case row may not reuse a balance identity: balance equations derive solely from terms;
- balance identities include the case, so multi-case models do not collide.

The executable law boundary is a **scalar conservation** law over `total`, `energy` or
`momentum` subjects with an identity projection. Indexed laws, `species`, `element` and
`phase_species` subjects, the `isothermal` and `pressure_total` expansions and
default-balance resolution are declared vocabulary that selected admission refuses with a
source-attributed diagnostic. A balance defines a zero-equality row in steady cases; with a
declared accumulation state it becomes that state's rate ([§10.3](#section-10-3)).

Balance rows remain physical declarations. Their tolerance and provenance support an
independent closure observation (`BalanceCheck`), distinct from solver residuals
([§15](numerical-execution.md#section-15)). They are not empirical certification.

### 10.2 Contribution roles and signs

Template contributions declare an orientation: `into_scope`, `out_of_scope` or
`generation`. Lowering maps them to the balance roles `inlet`, `outlet` and `generation`.
Balance rows authored directly, or produced by recipes and reaction projection, may also
use `consumption`, `heat_in`/`heat_out`, `work_in`/`work_out` and
`internal_in`/`internal_out`. Inflow, generation, heat-in, work-in and internal-in carry
`+1`; the opposite roles carry `−1`, times the term's finite multiplier.

An internal transfer names one `transfer_id` and appears exactly twice: with opposite signs,
over the identical typed source output, in the same case and quantity. A source output
appears once per balance mode. Template `accumulation` contributions are refused; an
accumulation is bound through the balance's conserved state instead.

Reaction contributions are projected from selected reaction applications. They reuse the
single authored rate output across species balances with stoichiometric multipliers.
Energy use requires explicit heat or declared formation data without double counting
([§9.7](physical-semantics.md#section-9-7)). The IDAES contribution catalogue remains a
reference for behaviour; only contributions admitted by these contracts execute.

### 10.3 Lumped control volumes, steady and dynamic

A steady lumped control volume is an ordinary template: local inlet and outlet symbols,
explicit equations and scalar laws over its contributions. Pressure equality, pressure
change and isothermal conditions are explicit equations, not implied options.

Dynamic lumped balances bind `physical_balances.accumulation` to a differential state of
the case's single `authored.dynamic_cases` declaration. The balance must supply that state's
rate row in every mode. Event impulses may target only actual non-terminal events. Each
conserved state has one balance owner, and `integral_tolerance` governs integrated closure.
The typed vessel recipe (`workflow::vessel`, `ModelBuilder::vessel`) produces these
declarations for fixed-composition amount and internal energy. It exposes algebraic
temperature, density and pressure closures from the FeOS provider, and optional valve
outflow. Time, events and integration belong to [§13](workflows-and-results.md#section-13)
and [ADR-0084](../../adr/0084-physical-provider-and-dynamic-contracts.md).

Templates cannot yet declare dynamic behaviour. Derivative symbols, `wrt_domain` and
continuous integrals are refused. Dynamic models use the balance and dynamic-case
declarations directly or through a recipe.

### 10.5 Formulation selection without code

Template configuration is typed data evaluated once at specialization:

| Mechanism | Current contract |
|---|---|
| `template_params` | Declared logical type (Boolean, finite real, integer, text/identity) and optional enum; required parameters need a binding; unknown or duplicate bindings are refused; structured parameter types and domain predicates are refused |
| `template_features` | Boolean or enum value from the instance binding or the declared default; inheritance is refused until explicitly resolved |
| `template_feature_rules` | `requires` predicates checked against resolved values; an unsatisfied rule is invalid |
| `template_guards` | Predicates over resolved compile-time values; symbols, equations, ports, contributions, laws and submodels are active only when their guard holds |
| Compile-time conditionals | `if … then … else` in an expression selects one branch from resolved values; no conditional reaches the compiler |
| `template_submodels` | A named child with one template selector (fixed ID or parameter); the child instance must exist, with exactly the declared parameter bindings; disabled submodels must have no child |

An unresolved guard, an unknown enum member or a predicate over runtime values is refused.
The compiler sees only the selected formulation; unused alternatives leave no
residual structure.

## 11. Unit models

### 11.1 What ships and what executes

The executable boundary for reusable composition is selected template lowering. It
supports:

- explicit finite scalar or indexed variables and parameters;
- equations with `eq`, `le` or `ge` sense and optional index filters;
- finite and ragged domain bindings;
- guards, child instances and scalar conservation laws;
- typed connections and case specifications that target template symbols.

Each active template symbol becomes one scalar per index tuple, with identity
`symbol_id(instance, declaration, members)`. The lowered projection and the
`ScalarBinding` inventory are inspectable on the revision. Finite specialization is
bounded; an instance whose index product exceeds 4,096 members is refused.

Refused at selected admission, with source identities:

- instance property- or reaction-package bindings;
- submodel multiplicity;
- material-derived or continuous domains;
- parent-dependent domains without explicit ragged tuples;
- reference, expression and derivative symbol roles;
- default initializer or scaler templates;
- selected rows in families with no execution interpretation, such as symbol, contribution
  and property requirement contracts.

Unrelated stored rows are counted as nonexecuting in `ModelRevision::admission`, not refused.

| Delivered model content | Role |
|---|---|
| `packages/reference/units` (`pse.units`) | Steady single-phase `Flowsheet`, `LumpedControlVolume`, `Heater`, `Feed`, `Product`, `Mixer`, `StateJunction` and `EqualityConnection` templates plus the reference law bindings; clean-room declarations with sources in `sources.md` |
| `packages/reference/states` | FTPx and FcTP state templates and their ordered port members |
| `workflow::vessel` | Typed recipe emitting ordinary computation, balance and dynamic declarations |
| Directional valve law | Rust provider (`pse-kernels::valve`) behind a declared law row and pressure-width contract |
| FeOS provider | Homogeneous explicit-density PC-SAFT/DIPPR properties ([§9.8](physical-semantics.md#section-9-8)) |

The `pse.units` and `states` templates use indexed laws, default-balance resolution,
property-package child selection, expression symbols and delegated child ports. They
therefore load and pass registry validation, but they fall outside the executable boundary
above. They are **not** an executable unit library today. The laws they reference remain
the vocabulary for balance selection ([§6.14](schema-and-relations.md#section-6-14)).
Executable unit models are currently authored as scalar-law templates, computation
definitions or recipes.

### 11.2 Worked variant: adding heat input to a unit

A heater differs from an adiabatic unit only in one additional energy contribution. The
excerpt below uses the actual document field names. IDs are abbreviated, and the
`enthalpyTotal` law binding comes from the reference `pse.units` laws document:

```yaml
template_law_instances:
  - {template_id: '<unit>', law_instance_decl_id: '<energy>', law_template_id: '<enthalpyTotal_law>',
     scope: self, options: [{key: tolerance, value: '1e-8'}], guard_id: null}
template_law_contracts:
  - {law_instance_decl_id: '<energy>', indexed_by: [], quantity_type_id: '<power>',
     balance_enum_id: '<EnergyBalanceType>', default_balance: null,
     coordinates: {member: null, phase: null}}
template_contributions:
  - {template_id: '<unit>', contribution_decl_id: '<in>', name: enthalpy_in, law_family: energy,
     expression: h_in, orientation: into_scope, scope: self, guard_id: null, doc: ''}
  - {template_id: '<unit>', contribution_decl_id: '<out>', name: enthalpy_out, law_family: energy,
     expression: h_out, orientation: out_of_scope, scope: self, guard_id: null, doc: ''}
  - {template_id: '<unit>', contribution_decl_id: '<q>', name: heat_duty, law_family: energy,
     expression: heat, orientation: generation, scope: self, guard_id: '<has_heat>', doc: ''}
```

`h_in`, `h_out` and `heat` are local template symbols of power quantity. The guard makes
the heat term a formulation choice. Adding the row changes the revision identity and the
balance's term list; it adds no capability, equation vocabulary or Rust code.
`composition_unit_scalar_law_owns_contributions_and_heater_variant` in
`workflow/composition/tests.rs` exercises this route.

### 11.4 Variants and narrowing

There is no template inheritance or `derives_from` mechanism. A variant is one of:

- an instance binding of parameters and features on an existing template;
- a template narrowed by `requires` feature rules; for example, the reference templates
  require `not dynamic`;
- a new template, when the equations differ.

A narrowed variant cannot add code, and a new physical operation belongs behind a provider
contract ([§22.3](#section-22-3)).

## 12. Connectivity

Connections couple instances only through typed ports. Selected lowering admits and
expands every connection. Physical connections also form the flow graph used for recycle
analysis.

### 12.1 Typed ports

A `template_ports` row declares `kind` (`material`, `heat`, `work`, `signal`), `direction`
(`inlet`, `outlet`, `bidirectional`), an optional guard and `bound_to`. `bound_to` names a
local symbol group. Port members come from `template_port_members` in declared ordinal
order, or else from the local symbol with that name, over every index tuple. Port order is
authoritative; members are never matched by name at connection time. Only local symbols can
be bound; a port that delegates to a child instance's state is unsupported.

Port identity is `port_id(instance, name)` in `workflow::composition`, shared by
connection producers and admission. Each member is a complete scalar physical port
(`pse_kernels::Port`). Its quantity type carries kind, basis, reference state, scale and
shape, together with a representation unit ([§8](physical-semantics.md#section-8)).
Provider ports extend this with ordered components, exact parameter-data identity and the
selected phase branch. The provider specification validates those extensions
([§9.8](physical-semantics.md#section-9-8)). An active port without active typed members is
invalid.

### 12.2 Connection admission

`authored.connections` records an occurrence from one port to another under a connection
rule. The rule template's `reference.connection_bindings` row selects the expansion. Only
`equality` exists; its behaviour comes from that binding, not from the template name.
A connection is admitted only when:

- both ports belong to the selected composition;
- the source is an `outlet`, the target is an `inlet`, and both have the same kind and
  member count (bidirectional ports are declared vocabulary but cannot be connected);
- each member pair passes the physical slot binding: representation conversion is allowed;
  a change of kind, basis, reference datum or scale, or a composition-dependent conversion,
  is refused;
- each inlet has at most one incoming connection;
- an extensive physical outlet has at most one outgoing connection. Fan-out needs an
  explicit splitter with its own conservation formulation. Signal ports are exempt;
- tear metadata is valid ([§12.5](#section-12-5)).

Implicit mixing or splitting at a port is never provided; mixers and splitters are units
with explicit balances.

### 12.3 Connection expansion during selected lowering

Expansion occurs in `ModelBuilder::freeze`. It is not a separate transformation or an
optional call. Each admitted member pair becomes an ordinary typed definition `s0 - s1 = 0`.
Its identity derives from the connection and both member identities, and the same compiler
admits it like every other definition. Unit conversion is applied only at the binding.
A revision with connections therefore always contains their equations. Removing a coupling
for a solve stage uses a case overlay or strategy ([§12.4](#section-12-4)), never an edit
of the revision.

### 12.4 Values across connections

No state-propagation operation exists. Values cross connections through the equality
equations inside a solve, and through structural initialization. Library DM/BTF blocks are
solved predecessor-first, and each block's committed coordinates become inputs to later
blocks ([§17](numerical-execution.md#section-17)). Explicit starts and stage overlays are
immutable replacements of case bindings. A failed stage leaves the original specification
intact ([ADR-0016](../../adr/0016-cases-and-results-never-mutate-the-model.md),
[§19.1](workflows-and-results.md#section-19-1)). Recycle maps carry each connection's
physical port conversions.

### 12.5 Flow projection and recycle structure

Lowering also emits a `pse_structural::flowsheet::Declaration` for the selected root. It
contains every unit node, including isolated units, and every physical connection
occurrence, including parallel ones. Signal connections remain mathematical assignments
and do not become flow edges, keeping information and physical flow distinct.
`FlowGraph::admit` canonicalizes the inventory, checks the physical bindings, applies
graph limits and hashes the projection into the revision identity.

Tear decisions are authored on connections:

- `tear_cost` is finite, nonnegative and defaults to 1;
- `tear_policy` is `free`, `mandatory` or `forbidden`;
- `tear_group` makes several occurrences one decision, and members of a group must agree
  on cost and policy.

`ModelRevision::prepare_flow` exposes this projection for the selected revision. There
are two tear selectors: an exact weighted feedback-edge MILP through HiGHS with native
bound and gap reporting, and an explicit unweighted petgraph heuristic. Each result is
checked independently against the graph. A recycle strategy needs explicit causal
directions and selected tear groups; KINSOL owns the fixed-point iteration
([§17](numerical-execution.md#section-17)). Tear selection is a prepared analysis, not
a compiled model fact.

### 12.6 Translation between conventions

A connection never reconciles different physical conventions. Differing reference states,
bases or composition coordinates fail slot binding. Translation is an ordinary unit: its
inlet and outlet symbols carry their own quantity types, and authored equations state the
mapping explicitly. A state junction is a template exposing one state on an inlet and an
outlet port. Flow scaling is a unit whose outlet equations multiply extensive members;
extensiveness comes from the quantity kind, never from a name pattern. The reference
package declares a `StateJunction` template, subject to the limits in
[§11.1](#section-11-1). No translator or stream-scaler template ships.

## 22. Authoring and the extension model

> Decision: [ADR-0088](../../adr/0088-selected-model-and-physical-contracts.md),
> [ADR-0089](../../adr/0089-semantic-identity-projections.md)

Documents and typed builders are two surfaces over the same generated declarations.
Both enter one selected-admission boundary, and both yield immutable revisions.

### 22.1 Package layout

A package is a directory with `package.toml` and YAML documents in declared locations. The
registry declares each document kind, path glob and section-to-relation mapping once
(`pse-schema::catalog::documents`, exposed as `reference.schema_documents`). The generated
DTOs and the loader share that declaration.

| Location | Contents |
|---|---|
| `package.toml` | `[package]` identity, name, semantic version, kind (`reference`, `library`, `model`, `case`), `id_policy` (`explicit`, `named`), exact `=` dependencies; optional `[[unit_sets]]` |
| `materials/*.yaml` | Units, quantities, operations, elements, domains, species, phases, material systems, reactions, stoichiometry, parameter values |
| `methods/*.yaml`, `properties/*.yaml` | Property method declarations, property packages, method selections, state bounds, scaling defaults |
| `templates/*.yaml`, `laws/*.yaml` | Template rows ([§6.6](schema-and-relations.md#section-6-6)), connection and law bindings |
| `computation_models/*.yaml` | Computation models, compositions, dynamics, providers, valve laws, balances, numerical requirements, reaction applications, fits |
| `instances/*.yaml`, `cases/*.yaml` | Instances, domain bindings, connections, flowsheets, scopes; cases, specifications, datasets, observations |
| `costing/*.yaml`, `assertions/*.yaml` | Stored costing templates; expected facts (tests, not truth) |

YAML is parsed by `serde-saphyr` under a parse budget, and TOML by `toml` with spans. Every
row keeps a source span into its document. Under `id_policy = explicit`, every entity needs
a persisted ID. `assign_ids` inserts missing ones as a reviewable edit and never writes
files. Identity, naming scope and expression owner come from the declared section mapping,
never from column order. Package resolution in `pse-authoring::p0` is exact: it rejects
duplicate or missing packages and dependencies, non-exact version requirements and cycles.
Schema detail is in the generated [relation reference](../../generated/README.md).

The reference packages are ordinary inputs, listed in `packages/reference/README.md`.
Only packages named in `fixture-projection.toml` feed generated Rust fixtures.

### 22.2 Revisions and edits

There is no change-set relation or operation log. Authored meaning changes through three
routes:

1. **Typed drafts.** `ModelBuilder` (Rust and Python) edits generated declaration values
   and source families. `ModelRevision::edit` starts a new draft; the earlier revision
   never changes.
2. **Documents.** `Runtime::models_from_documents` projects parsed documents into the same
   builders without a text round trip. Source edits are exact byte replacements checked
   against their before-images (`apply_edits`, `authored.document_edits`); a stale or
   duplicated preimage changes nothing.
3. **Identity-bound rename.** `rename_documents` (`authored.rename_requests`) checks the
   expected name. It rebinds every expression and target occurrence of the entity and
   reparses the changed bytes. Named-policy packages cannot rename in place; a rename there
   is removal plus addition.

`ModelBuilder::freeze` is the commit point. A frozen `ModelRevision` guarantees:

- every generated field and source family validates against the registry;
- selected rows are consumed or retained as nonexecuting; unsupported selected meaning is
  refused with its source identity;
- template specialization, reaction and balance projection succeed, and every case passes
  the compiler's selected-case admission (physical typing and structure, without building
  evaluators) ([§14](mathematics-and-compilation.md#section-14));
- the flow projection admits;
- the revision identity covers selected semantics, physical context, provider
  registrations and the semantic schema, but not documentation prose
  ([§5](identity-and-publication.md#section-5)).

It does not guarantee solver eligibility, convergence, physical adequacy or accuracy.
Preparation, execution and qualification own those. Failed admission leaves the previous
revision intact. Durable storage is explicit publication of immutable results with their
declarations ([§20](identity-and-publication.md#section-20),
[ADR-0091](../../adr/0091-immutable-publication-contract.md)). Results never write back
into authored meaning ([D13](architecture-overview.md#section-d13)).

### 22.3 What is data and what is code

| Extension | Realized as | Code change |
|---|---|---|
| Unit variant, formulation option, extra contribution | Template rows or instance bindings within [§11.1](#section-11-1) | No |
| Flowsheet, connection, tear policy, case, dataset | Documents or builder declarations | No |
| Algebraic correlation over admitted operations | Computation definition or template equation in the DSL ([§7](mathematics-and-compilation.md#section-7)) | No |
| Unit, quantity kind or quantity operation | Rows in a physical package ([§8](physical-semantics.md#section-8)) | No |
| Species data binding, material system, homogeneous reaction | Material rows and reaction applications within [§9](physical-semantics.md#section-9) | No |
| Law expansion, subject kind or template feature outside the executable boundary | Workflow lowering with tests | Yes |
| Physical operation not expressible as an admitted expression (EOS, nonsmooth law, root closure) | Provider in `pse-kernels` with a validated `ProviderSpec` and a declaration family | Yes |
| New mathematical primitive or solver class | `pse-math` binding or native adapter ([§18](numerical-execution.md#section-18)) | Yes |
| New declaration family | Registry declaration plus `just codegen` | Yes, generated |

A model is extended through declarations and bindings. New implementation is added only
for a genuinely new physical or numerical operation behind a checked contract. The
directional valve law and the FeOS provider show the code path. The heater variant in
[§11.2](#section-11-2) shows the data path. Scalar-only laws are the current limit on the
data route: ADR-0010 expects most new law families to need no code, but indexed and
non-conservation expansions still need lowering work.

### 22.4 Agent changes

Agents use the same routes as people: typed drafts, exact-preimage document edits,
identity-bound rename and `freeze`. There is no privileged write path, and agents do not
edit generated code or bypass admission. A proposed change is judged through the frozen
revision:

- `identity()` and `case_identity()` show whether semantics changed;
- `admission()` accounts for selected and nonexecuting rows;
- `projection()`, `scalar_bindings()` and `resolved_balances()` show the specialized
  model;
- preparation reports eligibility and routes.

No structural diff report (added/removed equations, sparsity or degrees of freedom) is
produced between revisions. Comparisons use these inspection surfaces.

## Retired section identities

#### 10.4 Distributed control volume — retired

No distributed or discretized control volume exists; continuous template domains and
derivative symbols are refused at selected admission ([§11.1](#section-11-1)) and the
capability limit is owned by [§25](scope-and-open-design.md#section-25).

#### 11.3 Isentropic pressure-change fragment — retired

No pressure-changer template ships; its compile-time conditional technique is current
under [§10.5](#section-10-5) and missing unit coverage is a limit under
[§25](scope-and-open-design.md#section-25).
