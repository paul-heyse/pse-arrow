---
id: ADR-0238
title: Bind native water properties to authored thermodynamic contracts
status: proposed
date: 2026-11-03
deciders: [paul-heyse]
level: decision
principles: [AP-01, AP-04, AP-07, PS-01, PS-02, PS-06, PS-12]
blueprint: [§9.8]
review: "not-required: a backend binding within the existing typed provider and authored physical-expression contracts; Plan64d D02 owns consumer qualification"
evidence: Proposed
supersedes: []
superseded-by: null
revisit: A required wet-state, fluid, phase, derivative or reference convention cannot satisfy the admitted consumer contract.
verification: Independent IAPWS water values and derivative controls; exact loaded-library and datum admission; actual authored pump/turbine preparation, solve, original balances and refusal of unsupported states.
---

# ADR-0238: Bind native water properties to authored thermodynamic contracts

## Context

Plan64d D02 and 64f F02 require steam and supercritical-water consumers. Existing authored
cubic and caloric implementations do not supply that science. The existing physical provider
contract can bind a complete native realization without introducing a second modeling engine.

## Scope

Bind CoolProp 8 HEOS Water through an optional feature of the existing native backend crate,
using the pinned `coolprop-sys` full C ABI. Amend blueprint §9.8 for this selected provider.
Authored definitions retain state coordinates, equations, ports, reference convention and
equipment behavior. No new crate, Python product, global reference-state mutation or arbitrary
provider-name dispatch is selected.

## Drivers

The first consumers need coherent pressure, molar enthalpy and entropy with homogeneous first
and second derivatives. Turbine reference states and a hydraulic liquid pump must consume actual
scientific state contracts. A standalone property calculation cannot qualify those consumers.
Provider preparation must retain exact data, reference and native ownership.

## Options

Select the library-owned multiparameter water realization and its analytic partials over a new
local IAPWS implementation. The safe `rfluids` wrapper does not expose the demanded partials or
its low-level handle; a narrow RAII owner over the existing generated sys binding preserves
them. Repeated string-based high-level flashes duplicate state preparation and obscure shared
outputs. Explicit temperature/density provider inputs let the authored outer problem retain
pressure/enthalpy/entropy specifications and numerical route selection.

## Outcome

Prepare an attempt-owned `AbstractState`, verify the actual loaded release and scientific data
identity, and supply one coherent pressure/enthalpy/entropy result with its demanded partials.
Reuse the state across compatible evaluations and release it on actual owner completion.
Independent attempts do not share mutable states. Library admission and native load failures
remain attributable; runtime lookup never downloads a missing library.

The authored stock convention has ideal-gas enthalpy and entropy zero at its declared reference
temperature and pressure. Evaluate the library's ideal-only anchor at that temperature and
`rho0 = p0 / (R_model T0)` using HEOS Water's fitted gas constant and molar mass. Shift enthalpy
and internal energy by the same enthalpy offset, and entropy by its ideal anchor. Retain the
actual reference conditions, fitted constants and offset values in the immutable interpretation.
Never set equilibrium-liquid enthalpy to zero and label it the stock ideal-gas convention.

### Consequences

Join the native factory through the existing admitted provider descriptor, compiler and runtime
preparation owners. Authored steam states and pump/turbine cases consume it through normal
scientific calls. The initial public outputs are pressure, enthalpy and entropy; native internal
energy is a coherence diagnostic until a consumer demands a distinct typed molar-u output.

### Compensating controls

Homogeneous derivatives do not imply wet-state derivatives, saturation selection or a stable
flash certificate. Preserve route-specific refusal until a demanded configuration is implemented
and tested. Native and reference comparisons retain the same datum and mass/mole mapping;
undoing the recorded offset recovers the independent IAPWS comparison convention.

### Confirmation

Plan64d D02 owns independent values, derivatives, reference/domain controls and actual authored
equipment tests. Plan64f F02 owns assembled power configurations and 64h owns finite reference
correspondence. No implementation or scientific qualification is claimed by this proposed record.

## Pros and cons

The existing provider boundary gains complete water science and analytic derivatives at the cost
of native artifact and reference-translation obligations. It avoids copying scientific equations
into Rust and keeps equipment composition in the authored packages.

## More information

[Plan64d D02](../plans/64d-scientific-assets-and-equipment.md#d02--deliver-state-and-thermodynamic-foundations)
owns execution. Primary interfaces are the [CoolProp low-level API](https://coolprop.org/coolprop/LowLevelAPI.html)
and [8.0.0 C ABI](https://github.com/CoolProp/CoolProp/blob/v8.0.0/include/CoolProp/CoolPropLib.h).
Existing authored reference translations and typed provider contracts remain the integration owners.

## Status history

- 2026-11-03: Proposed before the Plan64d native water backend binding; formal acceptance remains separate.
