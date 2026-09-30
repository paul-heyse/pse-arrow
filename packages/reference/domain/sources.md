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
heat capacity, vapor pressure and liquid molar density. A `parameter_set` is one source's
parameterization of one property of one species in one phase type; its keys are its
identity, whatever form refines it, and it has no reference attribute, because the datum of
what it computes is its form's result type. `caloric_set`, `vapor_pressure_set` and
`liquid_density_set` are the families a form refines; each is a `temperature_correlation`
whose envelope guards every read. A `property_package` selects one parameter set per
species, property and phase type (`selection`) and gives species roles (`component_role`,
with the charge a role requires). The generic operations `cp`, `enthalpy_increment`,
`entropy_increment`, `psat` and `liquid_density` resolve the selection, narrow the selected
set to its family by its keys and evaluate the form's bound function inside the set's
envelope; a missing selection is a refusal naming the package, species, property and phase
type.

`models/interactions.pse` declares symmetric pair data from one source (`pair`), the source a
package selects for each pair property (`pair_selection`, read by `pair_parameter`), and
group-contribution structure: main groups, subgroups with volume and surface parameters,
and species' subgroup counts. Sources are selected per package and property, so one
property's pairs come from one source. A method library declares its own typed pair and
group-interaction relations keyed by source.

`models/constants.pse` declares the molar gas constant, typed by its own quantity kind and
exact since the 2019 SI redefinition, citing the CODATA 2018 recommended values.

The schema's refusal corpus is `tests/fixtures/domain-refusals`; its pure fixtures are the
`pse.domain-fixtures` distribution.
