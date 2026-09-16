<!-- SPDX-License-Identifier: MIT OR Apache-2.0 -->
<!-- Copyright (c) 2026 Paul Heyse -->

# Reference packages

These versioned YAML/TOML packages are ordinary authored inputs under blueprint
§22.1 and ADR-0064. Their explicit IDs, complete physical quantities, expressions,
parameter values and exact package dependencies are the source authority.

| Package | Contents |
| --- | --- |
| `elements` | CIAAW element masses in kg/mol |
| `physical` | Units, complete quantities, conversions and physical operations |
| `fixture-currency` | Explicitly synthetic currency conversion test data |
| `methods` | NIST Shomate, RPP4 polynomial, Perry liquid and ideal property expressions |
| `states` | Single-phase FTPx and FcTP state declarations and port interfaces |
| `units` | Steady flowsheet, control volume, heater, feed, product, mixer, junction and connection templates |
| `thermo-examples` | Benzene/toluene and nitrogen species, actual method selections and sourced coefficients |

Each package's `sources.md` records physical references and qualification boundaries.
The benzene/toluene cubic coefficients are explicitly derived interpolation examples,
not published RPP4 coefficients. Synthetic currency is not physical reference data.

`fixture-projection.toml` explicitly selects the packages projected into Arrow-free
Rust fixtures. The generator first loads and admits those actual documents; it does
not maintain a second table of physical constants. `just codegen-bootstrap` first
regenerates Rust contracts without generated-DTO consumers, rebuilds the ordinary
package loader, then emits all requested outputs. `just codegen-check` checks the
complete generated trees once the source and contracts are consistent.
