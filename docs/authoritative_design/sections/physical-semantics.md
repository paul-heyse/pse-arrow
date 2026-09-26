---
title: Physical semantics, materials and properties
status: current
---

# Physical semantics, materials and properties

This area gives every value a complete physical meaning. It also binds authored chemistry
to the library that computes properties. `pse-quantity` owns quantity kinds, units, bases,
reference states, conversions and the inference that checks each operation.
`pse-material` owns elements, species, phases and stoichiometry. `pse-kernels` owns the
physical provider contract and its FeOS PC-SAFT/DIPPR and valve-law providers. The runtime
loads the registry once per physical inventory (`pse-runtime::physical`). It admits
selected materials, providers and reactions in `crates/pse-runtime/src/workflow/`
(`sources.rs`, `reactions.rs`). Typed preparation applies the checks before any library
algebra (`pse-compiler::typed_math`, `pse-math::typed`).

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
(`pse-compiler::physical_identity`) enters preparation keys, so editing a unit or type
invalidates dependent artifacts ([§5](identity-and-publication.md#section-5)).

### 8.1 QuantityType

A `QuantityTypeId` resolves to a complete key:

- quantity kind: its dimension, extensivity, and whether it is additive or
  origin-sensitive
- optional basis
- optional reference state (the datum)
- point or difference scale
- ordered index shape (domain kinds)
- optional subject kind

The registered type also names its canonical unit and an optional nominal magnitude. An
absent subject differs from an explicit `none`. Equality compares every IEEE bit,
including signed zero.

| Distinction | Encoded by |
|---|---|
| temperature vs temperature difference | `scale_kind` point vs difference on the same origin-sensitive kind |
| absolute vs gauge pressure | both are points; the gauge type carries its datum as `reference_state` |
| molar vs mass-specific enthalpy | `basis` |
| enthalpies with different datums | `reference_state`; mixing them is a datum mismatch |
| species vs phase composition | `shape` and `subject_kind` |
| vector over species vs over cells | `shape` domain kinds, even when lengths match |

`admission::require_same_contract` compares every component and the canonical unit. It
names the first component that differs. Ports, connections, instance slots and provider
bindings all use this comparison; none of them uses dimensional compatibility.

### 8.2 Units and unit sets

A `Unit` is anchored to SI: a symbol, a dimension vector, a positive finite scale and an
offset. Only an affine unit (°C, °F) may carry a nonzero offset. A unit may also restrict
its datum. For example, `psig` has the psi scale, a zero representation offset and the
declared gauge reference.

- **Representation conversion.** `convert_spec_for_type` changes representation units
  inside a complete quantity context. A point applies the affine offset; a difference
  applies the scale only. Conversion is refused when the dimensions differ or a unit's
  datum restriction differs from the quantity's reference. The context-free
  `convert_spec` refuses datum-restricted units altogether.
- **Datum and basis changes are physical conversions.** Gauge-to-absolute, molar↔mass
  and enthalpy-reference changes are registered `ConversionRule`s. Each is of kind scale,
  affine, or kernel with named parameters, and each is separate from representation. A
  datum is applied exactly once and never by a unit edge. Conversion requires matching
  kind, basis, reference and scale. A composition-dependent conversion needs an explicit
  physical operation: `pse-math::typed` refuses an inferred input conversion and does not
  apply one implicitly.
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
- **Spellings.** An authored definition maps each unit spelling to an admitted unit ID.
  The inventory reconciles `reference.units` with the derived `normalized.units`, and
  duplicate identities are refused. No compound-unit parser runs at execution time: a
  spelling must name an admitted unit.
- **Currency.** Currency is an optional eighth base axis. A conversion between years is
  an ordinary registered scale. The only bundled currency data is the explicitly
  synthetic `fixture-currency` package; production cost data must be declared separately
  ([§19.5](workflows-and-results.md#section-19-5)).

A smoothing tolerance (`SmoothMax`, `SmoothMin`, `SmoothAbs`, `SafeSqrt`, `SafeLog`) is a
positive finite scalar in the first operand's canonical coordinate. A tolerance that
carries a unit is converted as a *difference* of that operand's kind, basis and reference.
Celsius offsets and gauge datums therefore never enter a tolerance. `SafeLog` requires a
dimensionless input (`pse-quantity::smoothing`).

### 8.3 Quantity inference

`pse_quantity::infer::infer_with_evidence` runs once for every operation while typed
preparation builds a body. `pse-compiler::typed_math` lowers authored syntax, and
`pse-math::typed::BodyBuilder` calls inference before it constructs any Symbolica atom.
Dynamics applies the same inference to time derivatives
(`crates/pse-runtime/src/workflow/dynamics.rs`). The result is the complete result type
together with the selected rule and its operand order, never a dimension alone.

| Operation | Rule |
|---|---|
| Literal | A literal whose source span has a declared type in the definition takes that type. Otherwise a bare number takes the registry's explicitly designated neutral dimensionless type, and a number with a unit needs a unique scalar candidate; an ambiguous literal fails. Context belongs to the occurrence, so one source literal can resolve to a point in one place and a difference in another. |
| Add / Sub | Kind, basis, reference and subject must be equal. The index sets must also agree. |
| Origin-sensitive points | point ± difference → point. difference ± difference → difference. point − point → difference, only when the datums are the same. point + point and difference − point fail. |
| Neutral scaling | Multiplying or dividing by the neutral scalar with no indices keeps the other operand's full type, including difference. A composition fraction is not neutral. |
| Other Mul/Div, Pow, roots, functions | Exactly one registered `quantity_operations` rule must match the operand kinds, including a swapped order. It fixes the result kind and the basis, reference, scale, shape and subject policies (preserve, require-equal, registered-conversion, cancel, declared-result). No match or several matches fails; no rule is ever derived from dimensions. A variable exponent needs a dimensionless base. |
| WeightedMean | Values must share one complete type, and weights must be dimensionless. This is the only operation that averages origin-sensitive points. Certified unit-sum normalization needs its invariant proved against the actual operands. |
| Reduction | Removes exactly its bound index. A sum of points fails; a product needs a dimensionless body. |
| UnitConvert, KernelCall/provider, Gather, Broadcast, Conditional, Derivative, Integral | Declared input and output contracts are checked exactly. Conditional branches must have identical types. Derivative and integral combine with the domain unit through a registered rule. |

A registered rule can name preconditions (`reference.quantity_preconditions`: equal
operand bases, or a required operand contract). `PhysicalPreconditions` checks each one
against the actual operands at the point of application. A declared invariant ID is not
evidence that the invariant holds.

The reference `physical` package declares six compositions:

- flow × specific enthalpy → energy flow
- flow × composition → component flow
- gas constant × temperature → molar energy
- pressure ÷ molar energy → molar density
- temperature ÷ temperature scale → dimensionless
- activation energy ÷ molar energy → dimensionless

`crates/pse-quantity/tests/standard_package.rs` exercises these compositions, the gauge
datum and the smoothing tolerances. When a package needs a genuinely new composition, it
adds a rule. Adapters do not invent rules.

Failures use the `compile.math` family ([§23.2](operations-and-validation.md#section-23-2)):

- `unit_inconsistent`: incompatible contracts, an ambiguous literal or a mismatched edge.
  Named reasons include `point_plus_point`, `datum_mismatch` and `sum_of_points`.
- `quantity_operation_unsupported`: a missing rule or an unestablished precondition.
- `domain_violation_static`: a statically invalid argument.

Malformed registry declarations are validation invariants.

### 8.4 Bases and reference states

A `Basis` states what a quantity is per: molar, mass, volume, energy, standard volume or
dimensionless. It may add a composition or rate convention, and a standard-volume basis
names its reference conditions. A `ReferenceState` records its kind, optional temperature
and pressure, whether formation enthalpy is included, and an optional phase.
Enthalpy-like and entropy-like types carry their reference, and the additive rule refuses
to combine different datums.

The reference packages make these datums explicit choices, not universal standard
conditions. The `physical` package's 298.15 K / 101325 Pa reference excludes formation
enthalpy, and its gauge-pressure datum is a distinct reference. The stock ideal-gas
thermochemistry datum is 298.15 K / 100000 Pa. A method that needs any other
datum declares it. A change of basis or reference goes through a registered conversion
together with its parameter dependencies, applied by an explicit model operation (§8.2).
Selected reactions currently need a molar basis (§9.7). Mass-to-molar conversion inside
chemistry is not implemented.

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

> Decision: [ADR-0084](../../adr/0084-physical-provider-and-dynamic-contracts.md),
> [ADR-0088](../../adr/0088-selected-model-and-physical-contracts.md)

Libraries own the physics. FeOS computes thermodynamic properties and num-dual computes
their derivatives. The project owns the binding contract: which authored species maps to
which parameter record, in which coordinate order, with which physical port types,
datums, operating window and phase policy. Selected admission consumes, retains or refuses
every declaration ([ADR-0088](../../adr/0088-selected-model-and-physical-contracts.md)).
Selected property, reaction or phase-equilibrium meaning is never silently ignored. IDAES
names are preserved in enumerations and property names so that parity tests can map both
ways ([§6.14](schema-and-relations.md#section-6-14),
[relationship to IDAES](../../relationship-to-idaes.md)). Numerical IDAES equivalence is
not claimed.

### 9.1 Material declarations

Material facts are authored relations ([§6.4](schema-and-relations.md#section-6-4)):
species (component type, charge, formula, molecular weight, valid phase types), species
elements, phases, phase-species membership, reactions, stoichiometry, and material
systems. Element masses are reference data (the CIAAW 2024 abridged subset in
`packages/reference/elements`). A species is identified by its ID, not its formula.
Element IDs are named identities of their symbols.

`reactions::validate_materials` admits a selected material system only when all of the
following hold:

- the species are distinct and the phases are distinct
- every species/phase pair has an explicit phase-species row
- each species' declared valid phase types include the phase type
- every species is neutral and non-dissociating

A provider that names a material system must cover exactly that system's species and a
single phase. Nothing is derived from an implicit "every component is valid somewhere"
closure. Membership is declared, and a missing row is an error.

IDAES `GenericParameterBlock` configuration survives only as authored storage:
`property_packages`, `method_selections`, `phase_equilibrium_pairs` and package-owned
reactions. An instance that binds a property or reaction package is refused with "package
binding requires an admitted native material/provider declaration". Executed properties
come only from native provider declarations (§9.4).

### 9.2 State coordinates

The provider declaration states the thermodynamic state coordinates. The FeOS provider
uses temperature, molar density, and N−1 mole fractions in the declared component order.
One explicitly named dependent species takes the complement. Pressure is an output, not a
state variable. The provider refuses a noninterior multicomponent trial state instead of
clipping or renormalizing it.

The reference `states` package declares FTPx and FcTP single-phase state coordinates with
their port interfaces and IDAES-compatible bounds (mole fraction in [1e-20, 1.001]). These
are declarations, and they execute only through the refused package binding (§9.1). No
state-definition template, `defined_state`/`always_flash` inference or flash-required
projection exists in the current execution path.

### 9.3 Property method data

The `methods` and `thermo-examples` reference packages declare correlation forms with
sourced coefficients in explicit natural units:

- NIST Shomate
- RPP4 polynomial
- Perry liquid heat capacity and density
- ideal mixing

Enthalpy and entropy forms are definite integrals from an explicit reference. The data
also includes precedence rows and per-dataset validity intervals. An xtask test evaluates
these formulas against the published values and integral identities
(`xtask/src/codegen/physical/tests/formulas.rs`).

This is declared, tested data, not an executed method registry. No workflow resolves
`method_selections` through `method_precedence`. To execute a correlation today, author it
as an ordinary typed computation definition
([§10](models-and-composition.md#section-10)). Physical typing and the provider contract
check it like any other expression.

### 9.4 Physical provider contract

`pse_kernels::ProviderSpec` is the complete, immutable meaning of a provider:

- implementation ID and algorithm revision
- exact parameter-data hash
- selected phase and its revision
- ordered components
- ordered scalar input and output ports, each a quantity type with its representation unit
- optional operating envelope
- implemented derivative order and proven smoothness order, kept separate

`ProviderSpec::validate` checks the following against the registry:

- every port's type, unit dimension and scalar shape
- unique identities
- capacity limits

`Registration::new` checks that the factory's worker reports the same spec. A declaration
alone is never executable.

A `ProviderRequest` names its outputs and the raw derivative order it needs: value, first
or second. Asking for more than the implemented order, or more than the smoothness order,
is refused. Results must be finite and have the declared shape before they are returned.
Failures stay typed:

| Class | Meaning |
|---|---|
| `Trial`, `OutsideEnvelope`, `Singular` | recoverable rejection of one trial point |
| `Contract` | invalid registration or demand |
| `Limit`, `Cancelled` | resource or cooperative stop |
| `Terminal` | unrecoverable library failure |

Native adapters (`pse-backend-native::callback`) and residual dynamics treat trial classes
as recoverable ([ADR-0093](../../adr/0093-qualified-native-strategies.md)). Diagnostics
attribute them to their provider and port ([§23.2](operations-and-validation.md#section-23-2)).
`ProviderSpec::identity` frames the complete meaning, and only consumed provider contracts
enter the prepared semantic key ([§14](mathematics-and-compilation.md#section-14)). Having
derivatives does not imply phase regularity.

Typed preparation converts each input from its canonical coordinate to the port's unit,
and each output back again, with visible scale/offset atoms. Provider ports therefore
declare natural units without any implicit reinterpretation.

### 9.5 Phase equilibrium

Phase-equilibrium execution is not implemented and is refused explicitly. The
`native_providers` declaration carries a `formulation`, and only `homogeneous_density` is
admitted. `workflow::sources::factory` and `FeosPackage::new` both refuse
`phase_equilibrium`. Nothing executes flash, bubble/dew, SmoothVLE or complementarity
formulations, or equilibrium-temperature variables.

What exists is limited to diagnostics:

- `FeosPackage::initialize_npt` runs FeOS NPT density initialization with an explicit
  vapor-like or liquid-like density guess. This does not select a phase branch.
- Mechanical stability (the sign of dP/dρ) and global phase stability are separate
  observations, reported as `not_requested`, `stable`, `unstable` or `failed`.
- `StabilityPolicy` (`unchecked`, `mechanical`, `global`) decides which checks reject a
  trial. A failed check is never reported as stable.

### 9.6 Property demand

Property demand is explicit at each provider call. A model binds a provider through
`authored.native_providers` and names the output it consumes. Its bodies call that
provider through typed preparation, and each evaluation asks for exactly the outputs and
derivative order it needs. The FeOS worker computes one coherent state per derivative
seed block for all requested outputs. Its one-trial cache includes the complete inputs and
demand, and a failure clears the cache.

The global demand fixed point is not part of the current design. That covers property
requirement seeds, method-resolution closure and the `inferred.property_requirements`,
`method_resolutions` and `state_flash_required` relations. A missing property is a missing
provider output or an unresolved path in the definition, and it is refused where it
occurs.

### 9.7 Reaction binding

A selected reaction runs through an `authored.reaction_applications` row. The row names
the following:

- material system, reaction and phase
- the authored rate instance and output (an extent per time)
- one material balance per reactive species
- an energy balance, with an explicit heat-rate instance and output
- the element-closure tolerance
- provenance

`workflow::reactions::project` requires all of the following:

- a single homogeneous phase that matches the system
- a molar `rate` reaction with an explicit reaction phase
- species inside the material system
- element closure within the declared tolerance, checked by
  `pse_material::stoichiometry::admit_homogeneous` against actual element compositions
- molar-amount-per-time species balances and an energy-rate heat term

It then projects one authored rate into every species balance as signed generation or
consumption, using |ν| as the multiplier. It creates no second rate evaluator. The heat
enters as an explicit `heat_in` term, so formation energy is never guessed or counted
twice.

Refused cases include:

- concentration-form laws, because an implicit concentration conversion would be needed
- equilibrium or multiphase reactions
- nonmolar bases
- package-owned reactions

The declared convention also allows a formation-enthalpy route, but only the
explicit-heat route is implemented. The fixture methane + propane → 2 ethane is a
constructed conservation example, not a kinetic claim.

### 9.8 Provider implementations

**FeOS PC-SAFT with DIPPR ideal gas** (`pse_kernels::feos`) is the thermodynamic provider,
with explicit homogeneous density:

- **Record binding.** Each authored species maps to a PC-SAFT record and a DIPPR record,
  both identified by CAS number. The records are checked before `Parameters::from_records`
  runs, not selected with FeOS `subset`. Duplicate or missing records, and conflicting
  binary data, are refused. A missing binary interaction is refused under
  `require_explicit` and treated as zero only under the explicit `zero` policy.
- **Chemistry checks.** When a material system is named, the species molecular weight and
  formula must agree with the bound record. Component counts up to 128 are admitted, and
  derivative width follows the component count.
- **Outputs.** Pressure, total molar enthalpy, total molar entropy and ln φ for each
  component, all computed from one FeOS state.
- **References.** Enthalpy and entropy have separate references. Enthalpy is the DIPPR
  integral from 298.15 K, without formation enthalpy. Entropy is zero for the pure ideal
  gas at 298.15 K and 1 bar, and it includes mixing and residual terms. The FeOS pressure
  datum is corrected inside dual arithmetic. No absolute third-law entropy is claimed.
  The two references must be distinct registry states with those exact values.
- **Operating envelope.** The declared envelope covers temperature, density, pressure and
  every composition, including NPT initialization. It is enforced on every trial. Its
  bounds and provenance are part of provider identity. The envelope is not empirical
  certification.
- **Selected data.** Parameter data are authored in the declaration, so the provider is
  not tied to a fixed component set. The bundled dataset
  (`FeosData::light_hydrocarbons`, `crates/pse-kernels/data/`) covers methane, ethane
  and propane, with identified sources and explicit zero binary interactions.
- **Pins.** FeOS/feos-core, num-dual, `quantity` and nalgebra use the workspace pins
  (`Cargo.toml`). num-dual stays at the version compatible with FeOS. FeOS's `quantity`
  crate is used only at the boundary and is never a units authority.

**Directional valve law** (`pse_kernels::valve`) is a declared C² non-reversing
pressure-difference law. num-dual supplies its exact local derivatives.

**Helmholtz and CoolProp** providers are not implemented. No IDAES Helmholtz parameter
files, `general_helmholtz` equivalent or CoolProp coefficient source is present. A new
provider implements `ProviderFactory` and `Provider` under §9.4, and a `kind` in
`workflow::sources::factory` selects it. Any other `kind` is refused as an unknown native
provider factory. A general cubic equation of state is also absent.

### 9.9 Electrolytes and inherent reactions

These are not implemented. The authored schema can store charge, dissociation species,
component types such as `Ion` and `Apparent`, and `inherent` reactions. Selected admission
refuses them: any species with nonzero charge or a dissociation relation is refused as
"ionic/dissociating selected chemistry". Only `rate` reactions are admitted, and package
bindings are refused. True/apparent species bases, eNRTL and inherent-reaction extents do
not execute.

### 9.10 Scaling defaults and validity

`authored.default_scaling` rows (the equivalent of IDAES `default_scaling_factors`) are
keyed by property package, property kind and index. They take effect only through an
explicit `authored.provider_scaling_bindings` row that names three things:

- the provider and its selected output
- the target variable
- the provenance

The workflow checks that the output and target share one complete contract. It then
contributes a `property_default` numerical source in the output's unit
(`workflow::numerics`), ranked as in §8.5. A default that is not bound through such a row
has no effect.

Validity has two separate meanings:

- **Provider envelope.** This is executed. A trial outside it becomes a recoverable
  `OutsideEnvelope` failure. Nothing is extrapolated silently.
- **Correlation validity intervals.** These are data in the reference method packages
  (§9.3) and are not checked at execution time.

Neither kind of validity establishes empirical accuracy, which rests on independent
reference comparisons within the scoped qualification
([§24.2](operations-and-validation.md#section-24-2)). The recorded capability limits are
in [§25](scope-and-open-design.md#section-25).
