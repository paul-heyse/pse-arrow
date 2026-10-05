<!-- SPDX-License-Identifier: MIT OR Apache-2.0 -->
<!-- Copyright (c) 2026 Paul Heyse -->

# Domain schema, version 1.0.0

This package declares the domain schema above `pse.physical`, whose `chemistry` module
holds the chemical core (ADR-0127). It holds no data rows: packages that cite a reference
declare its source entity, method libraries declare forms and data banks declare the rows.

`models/provenance.pse` declares the provenance kinds and the provenance roles. An entity
whose kind refines `source` may be named as the source of a dataset, a constant or a test.
`publication`, `software_release`, `release_artifact` (a located file of a release),
`oracle_test` and `computation` (a reference computed with named software releases) refine
it. Identifier values (`Id<doi>`, `Id<isbn>`) and locators are data; nothing parses them.

A role is a member of `Role`. The kernel acts only on the facets a member declares:
`synthetic` and `oracle_input` data are test-only, so a root outside a test fixture that
reads them is refused; `derived` and `fitted` data name their lineage. `published` and
`measured` declare no facet. A role describes how the data were obtained; it does not
certify them.

`models/properties.pse` declares what a property is (its quantity type, index shape and the
phase types it applies to) and the pure-component properties the generic operations read:
heat capacity, vapor pressure and liquid molar density. Parameter records identify their
parameterization, family, subjects and variant independently of source provenance.
Phase-specific correlations use an explicit phase key; critical and segment parameters
use phase-independent records. A property package names its admitted selection context
and selects existing individual records. Dependencies and atomic fits close under the
consuming subject/model scope; coefficients remain in their original records.

`caloric_set`, `vapor_pressure_set` and `liquid_density_set` bind their forms and evidence
claims. Their generic operations evaluate declared applicability without turning an
unknown range into unrestricted use. Caloric increments request whole-interval coverage.
Named analysis permissions allow unknown evidence or extrapolation independently, and
observations retain the source claim and actual outcome.

`models/interactions.pse` declares canonical symmetric records and per-pair choices of a
stored record or an explicit predictive rule with inputs. Required missing pairs refuse;
stored zero and predicted zero preserve different identities. Directional method records
retain both orientations. Atomic fit membership has one authored direction, with derived
backlinks. Group-contribution structure declares main groups, subgroups with volume and
surface parameters, and species' subgroup counts.

`models/constants.pse` declares the molar gas constant, typed by its own quantity kind and
exact since the 2019 SI redefinition, citing the CODATA 2018 recommended values.

The schema's refusal corpus is `tests/fixtures/domain-refusals`; its pure fixtures are the
`pse.domain-fixtures` distribution.

`models/numerical-policy.pse` owns the shared engineering-design physical allowances
used by the process, thermodynamic and campaign models. The typed constants retain
separate molar bases, physical dimensions and point/difference meanings; they are
maintainer-selected analysis resolution, not empirical constants or uncertainty bounds.
Temperature resolves to 0.1 K, concentration to 1 mol/m³ (0.001 mol/L), power to 1 W,
pressure to 100 Pa, molar flow and inventory to 0.001 mol/s and 0.001 mol, inventory
energy to 100 J, and composition to 0.001. The energy allowance is approximately
0.016 K for the small 1.5 L reference water CSTR; it does not imply the same temperature
error for every inventory. No selected model has a mass quantity contract, so the
process-scale 1 kg guideline introduces no unused quantity declaration.

These physical original-space allowances differ from the runtime's global normalized
numerical policy. Authored integration requests `relative(global)` and
`normalized_absolute(global)` to consume that runtime policy. Explicit tighter literals
remain available for trace concentrations, small inventories, decisions near thresholds
and numerical verification. Smoothing, finite-difference perturbations, work limits,
physical domains and exact mathematical identities keep their separate meanings.
