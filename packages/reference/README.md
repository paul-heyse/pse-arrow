<!-- SPDX-License-Identifier: MIT OR Apache-2.0 -->
<!-- Copyright (c) 2026 Paul Heyse -->

# Reference packages

These versioned bundles contain ordinary authored `.pse` declarations, physical
YAML relations and TOML manifests. Their explicit IDs, complete physical quantities,
expressions, datasets and exact package dependencies are the source authority.

| Package | Contents |
| --- | --- |
| `physical` | Units, complete quantities, conversions, physical operations, entity kinds, prelude functions and compatibility enum declarations |
| `fixture-currency` | Explicitly synthetic currency conversion test data |
| `domain` | The thermodynamic domain schema (`pse.domain`): the phase kind and the canonical `liquid` and `vapor` phases |
| `methods` | Caloric interfaces and Shomate, polynomial, constant, density and vapor-pressure functions |
| `thermodynamics` | Potential identities, ideal/PR/PC-SAFT forms and state/equilibrium interfaces |
| `seed-data` | Chemical entities, CIAAW masses, sourced coefficients and named property-package bindings with fixtures |
| `process` | Control volumes, unit definitions, controllers, costing and vessel/fitting fixtures |
| `diagnostics` | Authored diagnostic thresholds and numerical profiles |

Each package's `sources.md` records physical references and qualification boundaries.
Published datasets, upstream oracle inputs and derived demonstrations retain distinct
provenance. Synthetic currency is not physical reference data. Scientific enums are
ordinary modeling declarations; only scaling strategies interpreted by the kernel
remain registry enums.

`fixture-projection.toml` explicitly selects the packages projected into Arrow-free
Rust fixtures. The generator first loads and admits those actual documents; it does
not maintain a second table of physical constants. `just codegen-bootstrap` first
regenerates Rust contracts without generated-DTO consumers, rebuilds the ordinary
package loader, then emits all requested outputs. `just codegen-check` checks the
complete generated trees once the source and contracts are consistent.
