<!-- SPDX-License-Identifier: MIT OR Apache-2.0 -->
<!-- Copyright (c) 2026 Paul Heyse -->

# Domain schema fixtures, version 1.0.0

This package exercises the domain schema of `pse.physical` (the `chemistry` module) and
`pse.domain` with pure fixtures. Its elements, species, groups, parameter sets and pair
values are synthetic: they test the schema's mechanisms and are no scientific source. Its
datasets carry the `synthetic` role, so only test fixtures read them.

Its entities live in this separate distribution so that no production closure admits them:
a closure that depends on the schema does not see these elements, species or groups.

- `derived_molar_mass`: a species' molar mass is derived from its formula and the standard
  atomic weights of its elements, and is absent without a formula.
- `two_group_unifac_admission`: two subgroups of two main groups and a species' subgroup
  counts admit; its volume and surface sums are read back.
- `property_operations_read_the_selected_form`: the generic property operations resolve a
  property package's selection and evaluate the selected form, and a pair parameter is read
  from the source the package selects.
