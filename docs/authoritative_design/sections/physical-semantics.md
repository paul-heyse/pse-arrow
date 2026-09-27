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
(`pse-compiler::physical_identity`) enters preparation keys, so editing a unit or type
invalidates dependent artifacts ([§5](identity-and-publication.md#section-5)).

### 8.1 QuantityType

A `QuantityTypeId` resolves to a complete key:

- quantity kind: its dimension, extensivity, and whether it is additive or
  origin-sensitive
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
| species vs phase composition | `shape` and `subject_kind` |
| vector over species vs over cells | `shape` entity kinds, even when lengths match |

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
  an ordinary registered scale. The synthetic `fixture-currency` package remains separate from the seed
  costing package's sourced CEPCI conversions
  ([§19.5](workflows-and-results.md#section-19-5)).

Smoothing and safe-domain functions are authored prelude functions with polymorphic
physical signatures. Their widths are positive differences in the operand's quantity
context; an affine representation offset is never a width. Function requirements and
validity predicates are checked through the ordinary typed path. No smoothing formula
is selected by a scientific name in production Rust.

### 8.3 Quantity inference

`pse_quantity::infer::infer_with_evidence` runs once for every operation while typed
preparation builds a body. `pse-compiler::typed_math` lowers authored syntax, and
`pse-math::typed::BodyBuilder` calls inference before it constructs any Symbolica atom.
Dynamics applies the same inference to time derivatives
(`pse-compiler::workspace::modeling`). The result is the complete result type
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

The physical package owns the operation declarations required by its knowledge bundles.
The standard-package tests exercise representative products, gauge datums and quantity
contracts. New physical compositions add explicit rules; adapters never infer them from
dimensions. Finite reductions retain their contracted kind and an empty-set prototype;
disjoint rule contracts are selected by actual operand prerequisites.

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
and pressure, whether formation enthalpy is included, and an optional datum subject.
The source boundary requires that subject to be an authored entity; its meaning belongs
to the declaring package rather than a closed phase vocabulary.
Enthalpy-like and entropy-like types carry their reference, and the additive rule refuses
to combine different datums.

The reference packages make these datums explicit choices, not universal standard
conditions. The `physical` package's 298.15 K / 101325 Pa reference excludes formation
enthalpy, and its gauge-pressure datum is a distinct reference. The stock ideal-gas
thermochemistry datum is 298.15 K / 100000 Pa. A method that needs any other
datum declares it. A change of basis or reference goes through a registered conversion
together with its parameter dependencies, applied by an explicit model operation (§8.2).
Reaction definitions state their actual basis and heat convention (§9.7).

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

The physical bundle declares generic kinds; seed packages supply chemical entities,
attributes, composition and coefficient tables. IDs distinguish members independently of
names or formula strings. All 21 previously shipped element masses are retained in authored
chemistry data. Molecular-weight and element-closure checks consume those actual rows.
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

The `methods` bundle owns Shomate, polynomial, constant-property, Perry density and
RPP4 vapor-pressure equations. The `seed-data` bundle owns their coefficients, physical
scales, reference values and provenance. `CaloricReference` defaults subtract explicit
enthalpy/entropy primitives evaluated at the reference temperature and add the supplied
datum. Derivative fixtures check those primitives against heat capacity; this does not
presume arbitrary symbolic antiderivatives.

Published values, IDAES comparison inputs and derived interpolation examples remain
separate. Authored fixture tolerances reflect their actual source and precision. The
common conformance runner executes the formulas through the production mathematical path.

### 9.4 External-function contract

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

### 9.5 Phase equilibrium

> Decision: [ADR-0102](../../adr/0102-discrete-and-global-design-target.md) — a certified
> tangent-plane-distance stability check over the global certification route enters the
> design target (Plan 22 G6; not yet implemented).

Authored equilibrium knowledge includes ideal bubble/dew equations, Rachford–Rice starts,
SmoothVLE and log-fugacity equality. The seed binds BTIdeal, FPhx and BT_PR comparisons.
Implicit closures can be inline, nested through the registered KINSOL capability or
accelerated through the registered cubic-root capability where the declared contract fits.

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

Reaction forms, rate functions, stoichiometry and thermal conventions are authored data.
The saponification seed uses neutral formula units, a sourced Arrhenius form and an explicit
reaction heat. Generic accumulators consume the selected component and energy contributions;
requirements and conformance fixtures check elemental closure. Its solvent-only caloric
approximation is explicit in the package and distinct from total component flow.
This seed does not establish ionic speciation, equilibrium-reaction or multiphase-reaction
qualification. Extending those sciences means new package definitions and fixtures.

### 9.8 Authored thermodynamic implementations

The current authored thermodynamic seed contains ideal gas/liquid forms, Peng–Robinson
residual potentials and the IDAES delta-convention override, and nonassociating PC-SAFT
hard-chain/dispersion potentials. Interface defaults derive properties through physical
partial derivatives. Density closures exercise inline, nested and cubic-accelerated
realizations. Caloric and residual terms retain separate explicit reference choices.

Methane/ethane/propane data feed both homogeneous process models and the vessel. A declared
C² directional-valve function replaces the Rust provider. Independent frozen teqp, FeOS
and caloric oracle inputs remain in tests, with source and convention notes in each bundle.
The source packages contain no general multiparameter Helmholtz or CoolProp implementation.
Focused executed evidence and its limits are recorded by the owning execution packet;
this inventory does not claim the excluded K9 campaign.

### 9.9 Electrolytes and inherent reactions

Electrolyte, true/apparent-species, eNRTL and inherent-reaction packages are outside the
current seed. Generic entities and tables can describe their data, but declarations alone
provide no numerical qualification. A future port supplies equations, physical contracts,
requirements, source provenance and authored fixtures; a missing generic mechanism is a
kernel gap, not permission for a scientific special case in Rust.

### 9.10 Scaling defaults and validity

Starts, nominals, validity intervals and scientific checks are annotations on the same
model that is solved. Numerical policy resolution retains source priority and unit
conversion (§16); block and recycle projections consume that resolved policy.

A validity annotation records membership independently from permission to extrapolate.
An out-of-domain trial is typed and attributed to its source/member/value/bounds. A final
candidate must pass original equations, declared checks and fixture expectations. A policy
allowing unclosed conservation does not waive arbitrary failed checks. Solver success,
physical validity, numerical acceptance and empirical adequacy remain distinct observations.

