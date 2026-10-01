---
title: Physical semantics, materials and properties
status: current
---

# Physical semantics, materials and properties

`pse-quantity` owns physical types, exact unit algebra and operation admission.
The runtime admits and retains the actual physical inventory. `pse-modeling` checks
scientific declarations as generic entities, functions, interfaces, definitions and tables;
`pse-compiler` and `pse-math` lower them to library-owned mathematics. Scientific knowledge
lives in authored bundles, not a material crate or thermodynamic provider factory.

## 8. Physical typing

A dimension vector is not a physical type. Torque and energy have the same dimension.
Molar and mass-specific enthalpy become confusable once the basis is divided out. An
absolute pressure and a gauge pressure differ only in their datum. A dimension checker
accepts each of those modelling errors, so the typing layer rejects them by comparing the
complete declared contract. Registry relations are described in
[§6.2](schema-and-relations.md#section-6-2); column detail is in the generated
[reference relations](../../generated/relations/reference.md).

Physical checking comes before Symbolica normalization ([ADR-0082](../../adr/0082-library-owned-process-mathematics.md)).
Algebraic simplification must never be the step that finds or hides a physical error.
No hash is part of validity, type selection or the proof that a conversion applies. The
framed identity of the admitted physical inventory
(`pse-compiler::physical_identity`, frame `pse.math.physical-inventory.v6`) enters
preparation keys, so editing a unit or type invalidates dependent artifacts
([§5](identity-and-publication.md#section-5)). Besides every declared contract, it frames
unit compositions, derived-kind definitions, declared kind equivalences, operation contracts,
and the names and typed conditions by which packages address quantity types and reference
states. *Tested* by
`physical_inventory_identity_frames_derived_definitions` (compiler units, with a frozen
empty-inventory vector).

### 8.1 QuantityType

> Decision: [ADR-0135](../../adr/0135-physical-expression-contracts.md) (proposed) — internal anonymous contracts retain declared public quantity boundaries.

> Decision: [ADR-0124](../../adr/0124-unit-algebra-and-derived-kinds.md) — a quantity kind
> may be declared as a monomial of other kinds with its canonical unit and declared result
> (Plan 23 KR2, implemented);
> [ADR-0127](../../adr/0127-chemical-core-in-physical.md) — the species, element and
> reaction kinds that quantity types name as subjects are declared in `pse.physical` itself
> (Plan 23 SM0, implemented; [§9.1](#section-9-1)).

A `QuantityTypeId` resolves to a complete key:

- quantity kind: its dimension, extensivity, whether it is additive or
  origin-sensitive, and an optional discrete category
- optional basis
- optional reference state (the datum)
- point or difference scale
- ordered index shape (package-declared entity-kind IDs)
- optional subject kind

The registered type also names its canonical unit and an optional nominal magnitude. An
absent subject differs from every declared subject-kind identity. Equality compares every IEEE bit,
including signed zero.

| Distinction | Encoded by |
|---|---|
| temperature vs temperature difference | `scale_kind` point vs difference on the same origin-sensitive kind |
| absolute vs gauge pressure | both are points; the gauge type carries its datum as `reference_state` |
| molar vs mass-specific enthalpy | `basis` |
| enthalpies with different datums | `reference_state`; mixing them is a datum mismatch |
| component vs element flow | `subject_kind` (species or element) |
| vector over species vs over cells | the modeling type's index axes (package-declared entity kinds), even when lengths match |

`admission::require_same_contract` compares every component and the canonical unit. It
names the first component that differs. Ports, connections, instance slots and provider
bindings all use this comparison; none of them uses dimensional compatibility.

A sum over a package-declared axis needs no registered shaped type: the modeling path
expands the reduction over the axis's admitted membership. The physical document therefore
registers shapes only over its own axes (`port_set`) and none over a chemistry kind
(*Tested* by `package_declared_axis_indexes_a_sum_without_a_registered_shaped_type`,
compiler units).

A dimensionless kind may carry a `QuantityKindCategory`: `count`, a number of discrete
things (units in operation, trays, modules), or `indicator`, a zero-or-one state. The
category is registry data (`reference.quantity_kinds`, since version 2), and registry admission
refuses it on a kind with a dimension. The category, not the dimension, admits an integer
or binary variable domain ([§6.8](schema-and-relations.md#section-6-8)): the neutral
scalar, mole fractions and other measured dimensionless kinds carry none, and an integer or
binary variable of such a type is refused at preparation. A semicontinuous or semiinteger
variable keeps its physical quantity. The physical bundle declares `count` and `indicator`
kinds and types, named `Count` and `Indicator`. The category enters the physical
inventory identity (§8).

**Derived kinds.** A quantity kind is base or derived (`reference.quantity_kinds` version
3). A base kind authors its dimension. A derived kind authors a monomial over declared kinds
with reduced rational exponents, its canonical unit, and the complete result admitted
at its named boundary: scale, optional datum and subject, and a basis
when its factors' bases may differ ([§8.3](#section-8-3)). Admission expands the monomial
to canonical base-kind factors and derives the dimension. It refuses a cycle, a monomial
that is empty or a single kind, two kinds with one monomial, a pure-number factor (the
neutral scalar, a count or an indicator), a canonical unit that does not store the derived
dimension, a type of the kind stored in another unit, and a kind with no registered type
carrying its declared result; a difference result is reserved for origin-sensitive kinds.
The physical document declares the DIPPR 100 and RPP4 coefficient kinds (molar heat
capacity per temperature to the first through fourth power), the Shomate E kind (molar heat
capacity times squared temperature), `molar_enthalpy` (molar heat capacity times
temperature) and `molar_entropy` ([§8.3](#section-8-3)), each with a defined canonical
unit. Public kinds and quantity identities are declared. Internal semantic kind expressions
may remain anonymous until a named boundary (§8.3); they do not create registry rows.
*Tested* by
`derived_kinds_are_admitted_acyclic_unique_and_derived` (`pse-quantity` units).

**Names.** A quantity type may carry the name packages address it by, and every reference
state carries one ([§8.4](#section-8-4)). Both are declared once, in the physical document
(`reference.quantity_types` and `reference.reference_states`, version 2 each), and share
one namespace of unique identifiers. A package sees them unqualified and as
`<package>.<name>` exactly when its manifest depends on the declaring package. A
package-level declaration or import alias equal to one of them is refused as ambiguous,
while a definition's own members shadow it lexically. A manifest declares no physical
names. The physical inventory names public quantity types, including `Scalar`, `Count` and
`Indicator`, and the named species response contracts in §9.8. *Tested* by
`quantity_names_declared_once_in_physical_document` and
`quantity_name_requires_manifest_dependency` (runtime units),
`ambiguous_quantity_name_refused` and `physical_names_are_scoped_by_document`
(`pse-modeling` units) and `physical_names_are_unique_identifiers` (`pse-quantity` tests).

### 8.2 Units and unit sets

> Decision: [ADR-0135](../../adr/0135-physical-expression-contracts.md) and
> [ADR-0136](../../adr/0136-declared-field-facets.md) (proposed) — checked canonical values and admission from selected physical definitions.

> Decision: [ADR-0124](../../adr/0124-unit-algebra-and-derived-kinds.md) — unit literals
> are canonical products of atomic units with rational exponents, and a product's identity
> is independent of its spelling (Plan 23 KR1, implemented).

A `Unit` is anchored to SI: a symbol, a dimension vector, a positive finite scale and an
offset. A unit is atomic or defined (`reference.units` version 2). An atomic unit authors
its dimension, scale, offset, affinity and optional datum restriction. Only an affine unit
(°C, °F) may carry a nonzero offset. A unit may also restrict its datum. For example,
`psig` has the psi scale, a zero representation offset and the declared gauge reference. A
defined unit authors only its composition: factors over declared units with reduced
rational exponents. Admission expands it to canonical atomic factors and derives its
dimension and scale. It refuses a cycle, a composition that aliases another unit, an
affine or datum-restricted factor, and an identity other than the product identity below.
The physical document defines its composite units this way (`J/(K*mol)` and `m^3` among
them), and the dimensionless `1` is the empty product.

- **Representation conversion.** `CanonicalConversionPlan` binds a selected complete
  quantity contract and its source unit. It checks finite source and result values around
  the canonical multiply-then-add operation. A point applies the representation offset;
  a difference applies the scale only. Dimensions and any unit datum restriction must
  agree with that quantity. Missing optional cells remain missing. Inline literals,
  document columns and mathematical scalar boundaries use this quantity-owned operation.
- **Datum and basis changes are physical conversions.** Representation conversion cannot
  change basis, reference or subject. `DatumConversionPlan` consumes the selected reference
  condition after representation conversion, so a gauge origin is applied exactly once;
  a pressure difference receives no origin shift. Composition-dependent enthalpy and
  entropy translations use paired authored scientific anchors (§8.4). Other admitted
  basis conversions retain their declared parameter dependencies. No expected dimension,
  unit spelling or numerical constant authorizes a physical conversion.
- **Where conversions occur.** Instance slots (`pse-math::binding::SlotBinding`), typed
  connections (`pse-structural::flowsheet`) and provider ports (`pse-math::typed`) each
  first require the same complete contract. They then apply a visible scale/offset
  conversion, which admits affine unit connections such as a °C port feeding a K port.
  Literals are converted to canonical coordinates when admitted. Mathematical bodies
  therefore see only canonical coordinates.
- **Canonical storage.** Each quantity type declares its canonical unit, and body formals
  use it. A unit set (`UnitSet`, the IDAES `UnitSet` equivalent) selects non-affine base
  units for length, mass, time, temperature and amount. Current, luminous intensity and
  currency are optional. Unit sets are validated package declarations; they do not
  override a type's canonical unit.
- **Unit literals.** The expression parser reads a unit literal as a canonical
  `UnitProduct`: each symbol once, with a nonzero rational exponent, in symbol order.
  `{J/(K*mol)}`, `{J/(mol*K)}` and `{J*mol^-1*K^-1}` are one value, `{m^(1/2)*m^(1/2)}` is
  `{m}`, and the renderer writes the canonical spelling back.
  `QuantityRegistry::compose` resolves each factor symbol to an atomic unit and derives the
  product's dimension and scale. A composite literal therefore needs no registered whole
  unit, and a defined unit is spelled by its composition, never looked up by its row
  symbol. A product's identity is its sole atomic unit when it is one unit with exponent
  one, and otherwise the `pse.quantity.unit-product.v1` derivation over its canonical
  atomic factors. A defined unit takes that identity, so a report in a composite unit
  carries it however the unit was written. An affine or datum-restricted unit may appear
  only as the sole factor with exponent one. An unknown symbol (`UnknownUnitSymbol`) and an
  affine factor (`AffineUnitFactor`) are `compile.math.unit_inconsistent`. There is no
  per-definition spelling map. The inventory reconciles `reference.units` with the derived
  `normalized.units`, and duplicate identities are refused. *Tested* by
  `unit_product_is_order_independent` and `rational_unit_exponents_canonicalize`
  (`pse-authoring` units), `composite_literal_needs_no_registered_whole_unit`,
  `affine_unit_only_as_sole_factor` and `defined_unit_dimension_and_scale_are_derived`
  (`pse-quantity` units), `report_in_a_composite_unit_carries_its_identity` (runtime units)
  and `the_unit_product_identity_is_frozen` (`pse-ids` golden vectors).
- **Currency.** Currency is an optional eighth base axis. A conversion between years is
  an ordinary registered scale. The synthetic `fixture-currency` package remains separate from the seed
  costing package's sourced CEPCI conversions
  ([§19.5](workflows-and-results.md#section-19-5)).

The selected relational physical closure uses the same quantity admission. The pure
`pse-relations::physical` builder consumes checked relation rows, reconciles reference
and normalized units, admits defined-unit factors and typed reference conditions, and
returns immutable definitions. Catalog selection supplies the exact revisions; runtime
owns retention and execution. Compatibility projects only distinct encountered
quantity/unit pairs. Missing definitions refuse admission and cannot be supplied by a
process-global inventory. Inputs without quantity occurrences need no physical closure.
This replaces relational dimension/scale approximations with the actual unit algebra
(proposed [ADR-0136](../../adr/0136-declared-field-facets.md)).

Smoothing and safe-domain functions are authored prelude functions with polymorphic
physical signatures. Their widths are positive differences in the operand's quantity
context; an affine representation offset is never a width. Function requirements and
validity predicates are checked through the ordinary typed path. No smoothing formula
is selected by a scientific name in production Rust.

### 8.3 Quantity inference

> Decision: [ADR-0135](../../adr/0135-physical-expression-contracts.md)
> (proposed) — one resolved physical contract admits anonymous scientific intermediates
> and explicit named boundaries. The implementation replaces ADR-0124's named-only
> intermediate inference; ADR-0124's exact unit algebra remains the unit authority (§8.2).

`pse_quantity::resolved` is the operation authority. A `ResolvedPhysicalContract` contains
canonical semantic kind factors with exact rational exponents, complete basis/reference/
scale/subject obligations, actual free binders, ordered index axes and numerical unit
representation. An internal result need not have a registered quantity identity. A public
quantity boundary requires its declared contract; a matching dimension alone is insufficient.
Declared kind equivalences admit scientific identities without manufacturing public types.

`pse-modeling` checks physical operations before numerical construction and retains their
admissions by structural expression occurrence, including lexical scope. A source span or
rendered expression is not a unique operation address. Concrete admissions retain their
actual result and selected rule; generic admissions retain owned requests and operand
schemes, instantiated once against the caller's admitted substitutions. Specialized
functions keep these products. Compiler lowering requires the checked product at each
function operation and passes it to `BodyBuilder`; a missing admission refuses. Finite
mathematical requests create an admission at their resolved occurrence before passing it
into the same builder. The builder consumes admitted operands/results and explicit boundary
scale normalization, rather than selecting a competing physical route (§14.3).

| Operation | Rule |
|---|---|
| Literal | Bare numbers use the designated neutral `Scalar`. A unit-bearing literal uses the actual expected complete contract or requires a unique admitted candidate. `1{1}` can be a mole fraction, count or species response in its declared context; bare `1` cannot acquire that meaning. |
| Add / Sub | Complete kind and contextual obligations must agree, including actual binders and ordered axes. Origin-sensitive point ± difference gives a point; point − point gives a difference only at the same datum. Point + point and difference − point refuse. |
| Discrete / neutral scaling | Count and indicator scaling precedes neutral scaling and keeps the other operand's complete context. An unindexed neutral scalar preserves the physical operand; a composition fraction is a distinct kind. |
| Registered operations | Matching and prerequisites use actual operands. A selected rule fixes result and operand order. Ambiguity or a selected rule's refusal is final and cannot be rescued by algebraic normalization. |
| Anonymous products / quotients / exact powers | Canonical semantic factors and numerical unit representation compose exactly. Qualified-factor cancellation retains contextual obligations and free binders. Named boundaries require admitted kind equivalence and compatible contextual meaning, not dimensions. |
| WeightedMean | Values have one complete type; dimensionless weights require the actual certified unit-sum invariant. Point averaging uses a point and weighted differences, preserving the selected datum. |
| Reduction | Removes exactly its bound coordinate, with its admitted empty-set prototype. A sum of points refuses. Species amount reduction explicitly produces `TotalAmount`; a different subject cannot silently become a species total. |
| Calls / conditionals / physical partials | Arguments and results cross checked complete boundaries. Conditional alternatives agree. A physical partial uses its actual independently bound argument and difference contract, even if numerical argument values coincide. |

The physical package owns the operation declarations required by knowledge bundles.
Preconditions are checked against the actual operands; naming an invariant does not prove
it. A scoped response formula authority can assemble dimension-compatible potential terms,
but preserves basis, datum, subject, axes and binders. It cannot override a selected rule's
refusal or authorize an unrelated ordinary expression. Reconstruction and response
boundaries explicitly authorize the named result (§9.8).

A valid scientific product can therefore pass through anonymous intermediates without
unit stripping and re-dressing. Unsupported physical compositions require an admitted
operation or declared scientific contract. They never acquire meaning from a target type,
a scalar cast or a same-dimension substitute.

**Entropy increments.** A molar heat capacity and a molar entropy have one dimension, so the
physical document separates them by kind, never by the type a context expects. It declares
two dimensionless base kinds, `temperature_ratio` and `logarithmic_temperature_increment`,
and three registered rules: two absolute temperatures divide to a temperature ratio; a
temperature difference over an absolute temperature is a logarithmic temperature
increment, with preconditions disjoint from the difference-over-difference rule; and the
logarithm of a temperature ratio is a logarithmic temperature increment. `molar_entropy` is
the derived kind molar heat capacity times logarithmic temperature increment, whose admitted
result is the stock-datum difference `DeltaS`, as `molar_enthalpy` (molar heat capacity
times temperature) resolves to `DeltaH`. An entropy increment is therefore written with
ordinary products, as `c1*log(T/T0) + (…)*r` with `r = (T - T0)/T` and heat-capacity terms
in the parentheses, and a heat-capacity expression returned where an entropy is expected is
refused. *Tested* by `ds_increment_types_as_entropy_difference` and
`returning_heat_capacity_where_entropy_is_expected_is_refused` (`pse-modeling` units) and
`entropy_increment_lowers_to_its_closed_form` (compiler units).

Failures use the `compile.math` family ([§23.2](operations-and-validation.md#section-23-2)):

- `unit_inconsistent`: incompatible contracts, an ambiguous literal or a mismatched edge.
  Named reasons include `point_plus_point`, `datum_mismatch` and `sum_of_points`.
- `quantity_operation_unsupported`: a missing rule or an unestablished precondition.
- `domain_violation_static`: a statically invalid argument.

Malformed registry declarations are validation invariants.

### 8.4 Bases and reference states

> Decision: [ADR-0135](../../adr/0135-physical-expression-contracts.md) (proposed) — contextual reference translation and owner-relative transfers.

A `Basis` states what a quantity is per: molar, mass, volume, energy, standard volume or
dimensionless. It may add a composition or rate convention, and a standard-volume basis
names its reference conditions. A `ReferenceState` records its name, its kind, an optional
typed temperature and pressure, whether formation enthalpy is included, and an optional
datum subject. Each condition is a value in a declared unit of a declared quantity type
(`ReferenceCondition`). Admission requires that type to be an absolute point of the
temperature or pressure dimension, and the value to be finite and positive in the type's
canonical unit. A package addresses a reference state by its name, as a `ReferenceState`
value, and reads `.temperature` and `.pressure` as quantities of their declared types; a
quantity type is not a reference state. The source boundary requires the datum subject to
be an authored entity; its meaning belongs to the declaring package rather than a closed
phase vocabulary. Enthalpy-like and entropy-like types carry their reference, and the
additive rule refuses to combine different datums. *Tested* by
`reference_state_conditions_are_typed` (`pse-quantity` tests) and
`reference_state_attributes_are_typed` (`pse-modeling` units).

The reference packages make these datums explicit choices, not universal standard
conditions. The `physical` package's `package_datum` (298.15 K, 101325 Pa) excludes
formation enthalpy and is also the gauge-pressure datum: the gauge-pressure type and `psig`
carry it, which distinguishes them from absolute pressure. The stock ideal-gas
thermochemistry datum `stock` is 298.15 K / 100000 Pa. The IDAES oracle conventions are
named datums too, each including formation enthalpy: `bt_ideal_oracle` (300 K, 100000 Pa)
and `bt_pr_oracle` (298.15 K, 101325 Pa). A method that needs any other datum declares
it. A change of basis or reference goes through a registered conversion
together with its parameter dependencies, applied by an explicit model operation (§8.2).
Reaction definitions state their actual basis and heat convention (§9.7).

Reference translation is an explicit operation with source and target contracts,
composition, actual component membership, paired anchor functions and provenance. Both
anchors are evaluated at the same explicit temperature and pressure. Finite weighted
means preserve point meaning; the authorized source-difference to target-difference step
and target-point addition leave composition dependence visible to evaluation and
physical differentiation. A numerical offset cannot erase or mint a reference.

Energy stock and energy transfer are distinct. `EnergyTransferRate` and `MechanicalEnergy`
are datum-free; referenced material energy flow and stored energy retain their selected
stock datum through registered mixed operations. `Transfer<quantity,boundary,Into|OutOf>`
adds an actual owner contract: instance, boundary declaration and ordered coordinates.
Negation changes the numeric value, never the owner or convention. Reorientation within
one owner and reflection across an admitted boundary pair are explicit operations.
Accumulator contributions consume canonical `Into` exactly once; an additional sign
modifier, wrong owner, wrong coordinate or unconverted `OutOf` contribution refuses.
Reports retain the actual quantity/unit and optional bound transfer context; presentation
labels do not establish physical meaning (§4.4).

### 8.5 Nominal magnitudes

`QuantityType.nominal_magnitude` is an optional positive finite default scale in canonical
units. Numerical policy resolution (`pse-math::numerics::resolve`) uses it as the
second-lowest source of a target's nominal. The sources rank:

1. analysis
2. case
3. model
4. bound property default (§9.10)
5. quantity nominal
6. canonical-unit nominal of one, recorded as `canonical_fallback`

Equal-priority conflicts fail. A strict-completeness policy refuses the fallback. The
nominal defines normalization; it is not a bound, a start value or native solver scaling.
[§16](numerical-execution.md#section-16) owns the policy.

## 9. Materials, properties and reactions

> Decision: [ADR-0098](../../adr/0098-modeling-knowledge-ownership.md),
> [ADR-0099](../../adr/0099-modeling-language-and-identities.md),
> [ADR-0100](../../adr/0100-modeling-functions-and-accounting.md) (proposed; implementation authorized).

Packages own chemistry, thermodynamic identities and correlations. Symbolica/Numerica owns
arithmetic and derivatives; native libraries own nonlinear iteration and linear algebra.
FeOS and num-dual remain independent reference-test dependencies only. They do not supply
production properties. The compatibility enum vocabulary does not imply a scientific
implementation or general IDAES equivalence (§6.14).

### 9.1 Material declarations

> Implemented amendment: [ADR-0140](../../adr/0140-scientific-knowledge-admission.md),
> maintainer-authorized Plan 25b; execution status belongs to that plan.

Conservation consumes an explicit complete composition claim, including complete-empty,
not formula-row presence or molar-mass availability. Missing sparse coefficients are zero
only within that admitted complete claim. Unknown species remain legal outside claims
requiring their missing evidence; charge is assessed independently. Atomic weights belong
to mass derivation rather than formula admission.

> Decision: [ADR-0127](../../adr/0127-chemical-core-in-physical.md) — the chemical core is
> declared once, in `pse.physical`, beside the physical quantity types that name its kinds
> as subjects (Plan 23 SM0, implemented).

The physical bundle declares the generic kinds. Its `chemistry` modeling package
(`packages/reference/physical/models/chemistry.pse`) declares the species, element,
reaction and phase kinds and the canonical `liquid` and `vapor` phases, and is the single
phase authority; references name them `chemistry.*`. The kinds live in `pse.physical`
because the physical inventory, admitted from that package alone (§8), names them: 34
quantity types take species, element or reaction as their subject, six operations name
species or element as their result subject, and both finite reductions range over species.
Its `kinds` package keeps the mesh and structural kinds (time, length, port set, stage,
cell, face, node and custom). Seed packages supply chemical entities, attributes,
composition and coefficient tables. IDs distinguish members independently of
names or formula strings. The species catalog supplies CAS/InChIKey identifiers and
formula composition; the separate CIAAW bank supplies all 21 element masses. Molecular
weights derive from those admitted rows, while unformulated pseudo-components may omit
them. Molecular-weight and element-closure checks consume the actual records.
Package requirements express valid memberships and stoichiometric constraints. There is
no separate `pse-material` authority or scientific foreign-key schema.

### 9.2 State coordinates

State coordinates are definition contracts. FTPx and FPhx bindings compose physical
pressure, temperature or enthalpy, composition and flow with a selected property package.
Homogeneous potential definitions use explicit density and composition coordinates;
equilibrium definitions introduce their own phase fractions and phase compositions.
Bounds, starts, normalization, reference conversions and operating intervals are declared
rather than supplied by a provider factory. Changing coordinates changes a binding or
authored equation; every analysis still consumes the same checked definitions.

### 9.3 Property method data

> Implemented amendment: [ADR-0140](../../adr/0140-scientific-knowledge-admission.md) and
> [ADR-0141](../../adr/0141-applicability-evidence-and-permissions.md).

Scientific records identify parameterization, declared family/contract, subject tuple and
variant. Provenance is attached evidence, and phase is a key only for a phase-specific
record. Required pairs select existing ordered/symmetric records or an explicit predictive
rule; fitted zero, missing pair and predicted zero remain distinct. Dependencies and
atomic-fit groups close over the selected model and subjects, with only declared subsystem
projections. No publication-wide source choice or coefficient copy substitutes for selection.

The `methods` bundle owns typed Shomate, DIPPR100/105, RPP4/Wagner, Antoine and
constant-property forms. Source-attributed data distributions own coefficients and named
caloric reference conditions; `seed-data` selects sets and property packages. Caloric
increments evaluate the declared forms between the selected reference and state, with
explicit enthalpy/entropy datum conditions. Derivative fixtures check those increments
against heat capacity; this does not presume arbitrary symbolic antiderivatives.

Published banks and oracle-input banks have distinct provenance and eligibility. Cubic
families, their kappa forms and missing-pair policies are selected package records.
The published Gross–Sadowski bank carries 20 CAS-keyed species with dimensioned PC-SAFT
parameters. The generic kernel contains no species parameters or scientific constants.
The authored SRK and NRTL examples demonstrate package extension; NRTL activity
coefficients derive from the extensive excess-Gibbs potential.

Published values, IDAES comparison inputs and derived interpolation examples remain
separate. Authored fixture tolerances reflect their actual source and precision. The
common conformance runner executes the formulas through the production mathematical path.

### 9.4 External-function contract

> Decision: [ADR-0120](../../adr/0120-provider-envelope-contract.md) — a provider factory
> may declare one closed output interval per output that every successful evaluation lies
> in; the host checks the declaration against the contract (Plan 22 G4, implemented),
> enforces it at every evaluation and frames it into the provider's configuration identity
> (Plan 22 ENV, implemented).

`pse-kernels` is the generic external-function host. `ProviderSpec` binds implementation
identity and revision, parameter-data identity, ordered typed ports, logical array shapes,
derivative source, available order and smoothness order. The host does not carry a species,
phase or equation-of-state interpretation. Registration compares the worker's actual
contract with the declaration; unsupported or inconsistent capabilities refuse.

Requests select exact outputs and derivative order. Physical representation conversions
remain explicit at the boundary. Returned values and derivatives must have the declared
shape and be finite within the admitted domain. Typed trial, contract, resource,
cancellation and terminal failures survive into native callbacks and diagnostic results.
Derivative availability and smoothness are separate contracts.

**Output envelopes** (`pse-kernels`). A provider factory may declare an output envelope
(`ProviderFactory::envelope`): one closed interval per output that every successful
evaluation lies in. Registration (`Registration::new` and `Registration::bind`) checks it:
one interval per output, neither end NaN, the lower end at most the upper, and each
interval containing a real number, so `(+∞, +∞)` and `(−∞, −∞)` are refused as a contract
error. Every evaluation is then checked against it (`Enveloped`): a finite output outside its
interval is a typed contract error naming the provider, the output and the interval, while a
nonfinite output stays a recoverable trial rejection. The envelope enters the provider's
configuration key (`Registration::configuration_key`, frame
`pse.provider.configuration.v1`, over the factory key and the exact bits of each bound); a
provider without one keeps its factory key. The factorable export bounds a provider output by
this envelope ([§7.5](mathematics-and-compilation.md#section-7-5)). *Tested* by
`provider_envelope_is_checked_against_the_contract`, `envelope_rejects_empty_interval`,
`envelope_violation_is_typed_contract_error` and `envelope_in_provider_key` (`pse-kernels`
units).

### 9.5 Phase equilibrium

> Decision: [ADR-0102](../../adr/0102-discrete-and-global-design-target.md) — a certified
> tangent-plane-distance stability check over the global certification route enters the
> design target (Plan 22 G6, partial: the authored tangent-plane model certifies an ideal
> feed stable on the `certify` route; a known Peng–Robinson instability is detected on a
> local route, whose global certification SCIP does not finish in bounded time, and the
> PC-SAFT distance is prepared and solved locally, not certified;
> [§18.10.1](numerical-execution.md#section-18-10-1)).

Authored equilibrium knowledge includes ideal bubble/dew equations, Rachford–Rice starts,
SmoothVLE and log-fugacity equality. The seed binds BTIdeal, FPhx and BT_PR comparisons.
Implicit closures can be inline, nested through the registered KINSOL capability or
accelerated through the registered cubic-root capability where the declared contract fits.

Phase disappearance can also be authored as complementarity: the reference
`ComplementarityVLE` states temperature slacks complementary to the liquid and vapor
fractions, realized smoothly, disjunctively or by an exact penalty
([§19.7](workflows-and-results.md#section-19-7)).

Nested-flash selection compares explicitly eligible regimes under an authored criterion.
Ties, singular derivatives, failed alternatives and unproved branch crossings refuse.
Implicit derivatives describe the selected smooth neighborhood. Neither a converged local
closure nor these derivative checks establish global phase stability or selector smoothness.

### 9.6 Property demand

Lazy demand closure instantiates requested definition members and their defining equations.
Interface defaults, explicit overrides and dispatch tables compose the selected physics.
Function references resolve in their definition scope; imports delimit visibility.
Effective-member lineage remains inspectable. The compiler groups shared implementations
without collapsing independent instance coordinates. External functions use explicit
registered capability references; scientific names never choose a Rust factory.

### 9.7 Reaction binding

> Implemented amendment: [ADR-0140](../../adr/0140-scientific-knowledge-admission.md).

A concrete checked reaction/material projection derives all source coefficients from the
authoritative reaction, checks every nonzero participant, and admits inert extra species.
Selected kinetic and heat records carry the same explicit extent normalization. Reactors
consume that projection, not an overrideable coefficient callback or inherited output.
Conserved apparent/lumped transformations carry their own admitted mapping evidence.

Reaction forms, rate functions, stoichiometry and thermal conventions are authored data.
The saponification seed uses neutral formula units, a sourced Arrhenius form and an explicit
reaction heat. Generic accumulators consume the selected component and energy contributions;
requirements and conformance fixtures check elemental closure. Its solvent-only caloric
approximation is explicit in the package and distinct from total component flow.
This seed does not establish ionic speciation, equilibrium-reaction or multiphase-reaction
qualification. Extending those sciences means new package definitions and fixtures.

### 9.8 Authored thermodynamic implementations

> Decision: [ADR-0135](../../adr/0135-physical-expression-contracts.md) (proposed) — authored
> maps, physical reconstructions, responses and reference translations preserve scientific
> meaning through ordinary checked mathematical composition.

A coordinate map declares physical arguments, validity and identified scalar or indexed
slots. `Coordinate<map.slot>` is a nominal refinement; another slot or map, or a general
`Scalar`, cannot substitute for it. `Reduced<reconstruction>` carries the selected reduced
law contract. Reconstruction consumes the explicit normalization and scientific reference
convention and produces a declared physical potential or increment. These operations lower
to ordinary checked functions and the existing library-owned mathematics; there is no
second differentiation or reduced-law engine.

The current authored seed contains ideal gas/liquid forms, Peng–Robinson residual
potentials and the IDAES delta-convention override, and nonassociating PC-SAFT hard-chain/
dispersion potentials. PR and PC-SAFT reconstruct `ResidualHelmholtzEnergy`, retaining the
ideal pressure and standard-state fugacity terms. NRTL reconstructs `ExcessGibbsEnergy`
under its declared degree-one reduced-amount convention; reconstruction does not multiply
by total amount a second time. Physical derivatives bind independent `T,V,n` or `T,p,n`
arguments before substituting density/composition coordinates.

Species response quantities include `LogFugacityCoefficient` / `FugacityCoefficient` and
`LogActivityCoefficient` / `ActivityCoefficient`. The logarithmic response and its
exponential are named subject-bearing contracts, even though their unit is `1`. They do
not return a neutral scalar. Datum-free `ResidualMolarEnthalpy` and `ResidualMolarEntropy`
remain distinct from the referenced material differences `DeltaH` and `DeltaS`. Explicit
mixed operations combine residual responses with material points while retaining the
material reference.

A response must consume its actual selected potential or physical partials. Generic
lowering first checks the selected potential's dependence on its admitted reconstruction,
and the response's dependence on that potential, using abstract scientific calls and
library normalization before substituting the reduced-law implementation. An unused call,
a zero coefficient or cancelling copies cannot supply a witness. A legitimately zero
reduced law can still produce zero value and derivatives because its scientific dependence
is checked before the implementation becomes zero. Syntactic call reachability alone is
insufficient.

Source consumers retain their original scientific origins explicitly. Vessel energy is
amount times the enthalpy difference from `constants.standard_enthalpy_datum`, minus
volume times absolute pressure. The reactor inventory uses that enthalpy difference and
the pressure difference from `constants.standard_pressure`. BTIdeal and BT_PR oracle
translations use their actual paired caloric anchors and composition. BT_PR's ideal-gas
pressure entropy increment maps physical pressure and reference pressure to their ratio,
then reconstructs `DeltaS` from the logarithmic reduced law and gas constant with named
scientific provenance. Reaction extent is a reaction-subject amount rate; explicit
stoichiometric projection yields component rates, and report units remain `mol/s`.

Methane/ethane/propane data feed homogeneous process models and the vessel. A declared C²
directional-valve function replaces the Rust provider. Independent frozen teqp, FeOS and
caloric oracle inputs remain with source and convention notes. Independent cubic helper
science remains authored alongside the physical potential path. The source packages
contain no general multiparameter Helmholtz or CoolProp implementation. Focused source
controls select current scientific declarations and provide small synthetic parameter
rows for isolated numerical behavior; they exclude external dataset transport. The owning
execution packet records actual evidence. Full package/data transport and assembled
qualification remain Plan 25k work.

### 9.9 Electrolytes and inherent reactions

> Implemented amendment: [ADR-0140](../../adr/0140-scientific-knowledge-admission.md).

An apparent/true-species transformation claiming conservation requires complete elemental
composition and independent charge evidence for every nonzero participant. Charge balance
alone does not establish element balance. This adds no universal isotope/site ontology or
numerical electrolyte qualification.

Electrolyte, true/apparent-species, eNRTL and inherent-reaction packages are outside the
current seed. Generic entities and tables can describe their data, but declarations alone
provide no numerical qualification. A future port supplies equations, physical contracts,
requirements, source provenance and authored fixtures; a missing generic mechanism is a
kernel gap, not permission for a scientific special case in Rust.

### 9.10 Scaling defaults and validity

> Implemented amendment: [ADR-0141](../../adr/0141-applicability-evidence-and-permissions.md).

Applicability claims are known regions, explicitly unrestricted evidence or unknown evidence.
Region observations preserve fitted/recommended/validated/reported meaning, selected record and
responsible form/model layer. Named record/family permissions for unknown evidence and
extrapolation default false, are independent, and retain binding authorization; they cannot
upgrade evidence. A named nominal family covers only its checked subtypes; the claim retains
its actual owner and checked owner ancestry, while the permission retains its authored targets.
B4 enforces the use gate; final result qualification remains a distinct
consumer. Mathematical and hard model domains always refuse. Required dependencies retain
all observations; a winning declared alternative retains its own required dependencies.
Numerical record reads attach evidence at the actual demanded expression, including direct
member reads; inactive branches acquire no evidence obligation. Arbitrary predicate regions
do not gain whole-interval coverage from endpoint tests. Generated observations retain actual
physical inputs, consuming instance, required/alternative status, unknown reason and complete
matched permission scope, targets and independent flags, including the checked owner ancestry. Their framed identity distinguishes
applications of the same source call at different instances or physical inputs.

Starts, nominals, validity intervals and scientific checks are annotations on the same
model that is solved. Numerical policy resolution retains source priority and unit
conversion (§16); block and recycle projections consume that resolved policy.

Hard validity annotations and mathematical domains are unwaivable. Scientific applicability
records membership independently from permission to extrapolate.
An out-of-domain trial is typed and attributed to its source/member/value/bounds. A final
candidate must pass original equations, declared checks and fixture expectations. A policy
allowing unclosed conservation does not waive arbitrary failed checks. Solver success,
physical validity, numerical acceptance and empirical adequacy remain distinct observations.

