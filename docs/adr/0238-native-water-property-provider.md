---
id: ADR-0238
title: Bind native water properties to authored thermodynamic contracts
status: proposed
date: 2026-11-03
deciders: [paul-heyse]
level: decision
principles: [AP-01, AP-04, AP-07, PS-01, PS-02, PS-06, PS-12]
blueprint: [§9.8]
review: "not-required: a backend binding within existing typed providers; Plan66 G02/G04 and I02 own remaining consumer qualification; maintainer reconciled the bounded artifact/binding implementation route on 2026-11-06"
evidence: Proposed
supersedes: []
superseded-by: null
revisit: A required fluid, saturation envelope, derivative, reference convention or native preparation target cannot satisfy this bounded binding.
verification: Independent IAPWS values and analytic first/second derivative controls; exact source/patch/dependency/toolchain and loaded-image admission; actual dry/wet equipment, original balances, protected delivery and unsupported-domain/curvature refusals.
---

# ADR-0238: Bind native water properties to authored thermodynamic contracts

## Context

Plan66 G02/G04 and I02 retain remaining Water, steam-equipment and power consumers. Delivered
homogeneous HEOS states and three pump/dry-turbine consumers are foundations, not work to
recreate. The third released turbine remains required: 400 K and 200 kPa inlet, 140 kPa outlet,
1000 mol/s and isentropic efficiency 0.9. Its reference and actual outlet are wet; current
homogeneous temperature/density providers correctly refuse them.

CoolProp's C++ pressure-based saturation derivatives reach second order, but the unpatched
8.0.0 C ABI/Rust binding lacks `AbstractState_second_saturation_deriv`. The specialized
two-phase density cross derivative does not provide a complete caloric Hessian. This is a
binding/artifact obligation, not permission to reinterpret homogeneous derivatives.

## Scope

Retain CoolProp 8 HEOS Water through the existing native backend feature and pinned
`coolprop-sys`. Add one minimal C export for library-owned
`second_saturation_deriv(Of, P, P)` inside the existing wrapper/handle manager/exception boundary.
The target includes analytic first and second pressure derivatives of saturated endpoint
temperature, density, molar enthalpy, entropy and internal energy. Explicit first-only execution
may be eligible; it is not a permanent cap or a substitute for required second-order consumers.

Authored definitions retain coordinates, equations, phase membership, ports, datum and equipment.
The new wet state uses pressure and quality with native saturation endpoints; homogeneous
contracts remain intact and the pump stays liquid. No new crate, Python product, global datum
mutation, provider-name dispatch or general flash ontology is selected. Acceptance and
architecture publication for blueprint §9.8 remain separate.

## Drivers

Consumers need coherent caloric values, datum and demanded derivatives in their actual
homogeneous/saturated coordinates. Turbine entropy matching and enthalpy work require a valid
wet state; scalar values cannot qualify equipment. Artifact selection must preserve exact
source, patch, data, build inputs and native allocation ownership.

## Options

Select library-owned HEOS saturation endpoints and analytic derivatives over local equations,
finite-difference production derivatives or unstated interpolation. The high-level `rfluids`
route and nested sigma strings do not expose the required second-order endpoint contract.
Pressure/enthalpy flash values alone do not establish derivative eligibility. A separate C++
state owner/helper library duplicates handle lifetime and admission; prefer the minimal export
in the original wrapper unless concrete integration burden forces reconsideration.

Retain `coolprop-sys`'s synchronization and handle API. Its platform dependency currently copies
and links the bundled image; `regen-bindings` changes the table, not that artifact. A maintained
local supplier correction must remove that bundled-copy/link path for this integration, consume
the explicitly prepared image and regenerate the matched binding. One artifact serves dry and
wet calls; no bundled dry path plus separate wet loader is selected.

## Outcome

Prepare an attempt-owned `AbstractState`, verify the actual loaded release and scientific data
identity, and supply one coherent pressure/enthalpy/entropy result with its demanded partials.
Reuse the state across compatible evaluations and release it on actual owner completion.
Independent attempts do not share mutable states. Library admission and native load failures
remain attributable; runtime lookup never downloads a missing library.
Endpoint states and second-order result buffers remain charged to their existing attempt/pool
owners through actual release. Check cancellation around native calls; do not claim an
uninterruptible library flash has stopped before it returns and its handles settle.

### Native construction and binding

Extend existing explicit `pse-host::native::providers` preparation/selection with a CoolProp
capability. Keep source 8.0.0 at `ae81610e7d23efc57f9d051c8e70a4d66e87537f`, maintain the
minimal wrapper/header patch as a checked input, and record the sys correction through existing
`vendor/deltas` provenance. No fork publication is required.

Preparation captures checksum-verified archives for the complete native build closure. This
release uses CPM dependencies, including tag-based inputs, and Python header/data generation;
a top-level archive hash alone is insufficient. Resolve/retain exact dependency and generator
inputs during explicit preparation, then configure against them with implicit downloads disabled.
Use CPM's existing `CPM_<name>_SOURCE` overrides for retained sources and local-only refusal
for a missing dependency; do not rely on an opportunistically populated download cache.
Python is a build tool, not a runtime scientific owner. Preserve selected compiler/cache bridge,
tool hashes, target and options in construction identity. A shared Release CMake build and
installed headers/image become one immutable completed generation. Reconstructibility is
required; bitwise reproducibility is a measured claim until repeated construction demonstrates it.

A coordinated exact-version local correction of existing `coolprop-sys` 8.0.0 consumes the
completed provider and records its immutable image path. Its Cargo build neither downloads nor
constructs CoolProp. Remove the selected platform bundle dependency while retaining the single
global synchronization boundary and native handle lifecycle. Regenerate its owned table through
an explicit generator from the matched maintained C declaration; never edit generated bindings.
The prepared-prefix environment selection is a construction input: its build script records the
image path for the existing `COOLPROP_PATH`/global loader and watches selection changes. Match
the new declaration in both the installed native header and the sys supplier's full ABI header.
The added export takes the existing handle, output and two derivative keys, with neighboring
exports' error code/bounded message-buffer convention. Check native errors and finite results.

Preparation authenticates source/patch/dependency/tool identity and every retained file. The
consumer build embeds selected source/patch/binding identity and actual image digest; runtime
verifies those exact bytes, version, upstream revision, Water data and datum. Replace the
published-image SHA premise with this matched-artifact premise, never a version-only check or
arbitrary caller path. Carry it through existing native build/profile interpretation. Missing
preparation, stale patch/header/binding, changed bytes and unsupported targets refuse explicitly.

### Datum and saturated composition

The authored stock convention has ideal-gas enthalpy and entropy zero at its declared reference
temperature and pressure. Evaluate the library's ideal-only anchor at that temperature and
`rho0 = p0 / (R_model T0)` using HEOS Water's fitted gas constant and molar mass. Shift enthalpy
and internal energy by the same enthalpy offset, and entropy by its ideal anchor. Retain the
actual reference conditions, fitted constants and offset values in the immutable interpretation.
Never set equilibrium-liquid enthalpy to zero and label it the stock ideal-gas convention.

Prepare saturation endpoints using pressure/quality at quality zero and one; pressure/temperature
alone cannot select a wet state. Admit a declared pure-Water subcritical pressure interval away
from triple/critical degeneracies. Verify endpoints, finite values, positive latent heat and
ordered distinct densities. Apply the identical immutable h/u and s offsets as homogeneous
Water. First and second endpoint pressure derivatives are library saturation derivatives.

An authored wet state extends `equilibrium.EntropyThermoPackage` with pressure/molar quality,
actual liquid/vapor members, datum-safe h/s mixing and volume/density closure. Reuse existing
component/phase and origin-sensitive arithmetic meanings. Generalize only the turbine wrapper's
constructor types to its already-consumed entropy interface; retain the hydraulic pump's strict
liquid type/guard. Existing reference entropy/pressure, enthalpy work and efficiency equations
remain authoritative. No additional equipment engine or automatic regime switching is introduced.

### Consequences

Join the native factory through the existing admitted provider descriptor, compiler and runtime
preparation owners. Authored steam states and pump/turbine cases consume it through normal
scientific calls. Homogeneous public outputs remain pressure, enthalpy and entropy. Saturation
adds coordinates and endpoint/phase/caloric outputs needed by the wet consumer; internal energy
remains a coherence diagnostic until a consumer demands a distinct typed molar-u output.

### Compensating controls

Homogeneous derivatives do not imply wet-state derivatives, saturation selection or a stable
flash certificate. Preserve route-specific refusal until a demanded configuration is implemented
and tested. Do not apply the homogeneous positive isothermal mechanical-response guard to the
bulk equilibrium wet mixture. Its bounded saturation/endpoint contract is distinct; phase
fractions/native flash imply no global stability proof. Exact-Hessian consumers require actual
second-order eligibility; explicit first-only execution grants no curvature products. Derivative
availability grants no scientific-use permission. Native/reference comparisons retain datum and mass/mole mapping;
undoing the recorded offset recovers the independent IAPWS comparison convention.

### Confirmation

Plan66 G02/G04 own endpoint/domain/datum/derivative controls and actual wet equipment; I02 owns
integrated power consumers; 66d owns changed math; 66c/66f own relevant native preparation and
delivery; 66l owns independent population/assembled evidence. These are narrow consuming
dependencies, not whole-companion gates.

Required controls cover C ABI success and invalid-handle/key/error-buffer behavior, Water liquid
and vapor endpoint first/second derivatives at 140 kPa, cancellation/settlement, artifact
mutation/stale binding refusal and the three delivered homogeneous consumers. Numerical
derivative comparisons are independent tests, never production fallbacks. Upstream second-
saturation tests enumerate T, density, h, s and u for propane; they do not qualify Water.

Wet equipment must check both reference/outlet states, original entropy/enthalpy and complete
phase/component balances, released temperature/work/quality, independent IAPWS values, and
protected/native-view/portable readback with original permissions/datum. Rounded expectations
are not an executed independent-reference receipt. A dry quality-one substitution must fail.
Unsupported identity/envelope/datum/units/derivatives refuse without granting point/curvature
products. No new artifact, implementation or scientific qualification is claimed by this revision.

## Pros and cons

The existing provider gains bounded wet caloric and second-order endpoint science while equipment
stays authored. The cost is one C export, matched sys integration and reproducible preparation/
provenance, preferable to another EOS, unsupported derivative substitution or second state/loader.

## More information

[Plan66 G02/G04](../plans/66g-properties-equipment-and-steady-workflows.md) and
[I02](../plans/66i-specialized-process-families.md) own remaining scientific execution.
Primary sources are the [low-level API](https://coolprop.org/coolprop/LowLevelAPI.html),
[exact HEOS implementation](https://github.com/CoolProp/CoolProp/blob/ae81610e7d23efc57f9d051c8e70a4d66e87537f/src/Backends/Helmholtz/HelmholtzEOSMixtureBackend.cpp),
[upstream endpoint tests](https://github.com/CoolProp/CoolProp/blob/ae81610e7d23efc57f9d051c8e70a4d66e87537f/src/Tests/CoolProp-Tests.cpp),
[C wrapper](https://github.com/CoolProp/CoolProp/blob/ae81610e7d23efc57f9d051c8e70a4d66e87537f/src/CoolPropLib.cpp),
and [dependency declarations](https://github.com/CoolProp/CoolProp/blob/ae81610e7d23efc57f9d051c8e70a4d66e87537f/cmake/dependencies.cmake).
Existing datum translations, typed providers and native supplier receipts remain integration
owners. Plan64/65 references below are historical rationale, not active prerequisites.

## Status history

- 2026-11-03: Proposed before the Plan64d native water backend binding; formal acceptance remains separate.
- 2026-11-06: Revised this proposal for Plan66's required wet turbine and full library-owned
  second-order pressure endpoints, one reproducibly prepared patched image and matched C/Rust
  binding. Maintainer reconciled the bounded artifact/ABI route before production changes;
  acceptance and architecture publication remain separate.
