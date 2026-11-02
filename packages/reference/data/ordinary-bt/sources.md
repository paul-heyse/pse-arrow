<!-- SPDX-License-Identifier: MIT OR Apache-2.0 -->
<!-- Copyright (c) 2026 Paul Heyse -->

# Ordinary benzene/toluene inputs

This package selects ordinary scientific inputs independently of the IDAES comparison
data. It retains ideal-mixture and Peng–Robinson/relational-density physics but changes
the parameter sets and caloric realization. It is not default-configuration IDAES parity.
`ordinary_bt.Ideal`, `Liquid` and `PR` contain no oracle-coordinate outputs or translated
IDAES formation constants. The original comparison packages remain unchanged.

The exact input artifacts are retained under `source-artifacts/`. The Chemicals revision
and artifact SHA256 values have one authored home in the source entities in
`models/parameters.pse`. That collection reproduces Poling vapor-pressure, Yaws critical
and CRC vaporization data; its software revision and each selected CAS row are part of
the attribution, not independent experimental verification.

The FeOS artifact is the existing declared `references.feos_poling2000_json` at commit
`c658aeab484f7a7096bfbf5425e40effd60da167`. Its acquired full-file SHA256 is
`8cd2761201f0c1942679a58d9266df39829e84e79744e08bd46700cc67ed08f8`.
The selected benzene and toluene DIPPR100 coefficients retain their original kmol units;
ordinary quantity admission performs normalization. The native adapter
`feos_poling2000_bt_caloric_delta` reparses this exact data shape into typed source cells,
preserving source/parameterization bindings and exact-parent delta semantics.

## Mathematical conventions and limits

- Poling's pressure coefficients use Wagner2.5,5. They must never be supplied to the
  existing RPP4 Wagner3,6 evaluator. Its fitted critical coordinates also differ from
  the Yaws EOS critical data; these are separate records with separate source roles.
- The FeOS file does not supply an empirical validity interval. The selected298.15–450K
  interval bounds this consuming realization; it is not a newly verified empirical
  validity envelope. Consumer acceptance and empirical evidence remain separate.
  `feos_dh` and `feos_ds` retain an explicit unknown claim over both physical
  integration endpoints, using the shared DIPPR100 mathematical cores. They do not
  infer whole-path support from two unknown point claims. The generic
  `properties.caloric_dh`/`caloric_ds` interval facade remains applicable to records
  with declared interval regions; this package selects the explicit unknown-path
  wrappers for FeOS data.
- Liquid heat capacity comes from the existing published Perry seventh-edition banks
  for both species. Preserve their original envelopes. An ordinary case exceeding one
  needs an explicit attributed applicability decision, not enlarged source bounds.
  The retained benzene range is278.68–353.24K; toluene is178.18–500K. The ordinary
  368K recycle is therefore explicit benzene liquid-Cp extrapolation,14.76K above
  its published maximum. Its consuming recycle cases target that exact record with
  `allow_extrapolation`, keeping unknown evidence denied and the source range intact.
- Ideal-gas enthalpy and entropy are zero at298.15K under the native stock convention.
  Liquid enthalpy is offset by the sourced CRC vaporization enthalpy at298.15K. This
  defines energy differences without claiming new formation enthalpies. No liquid
  entropy datum is invented; these bindings expose liquid enthalpy only.
- The unlike PR interaction is explicitly missing and predicted zero by an authored
  native policy. It is not an observed zero coefficient. Critical-property and
  prediction applicability remain unknown unless the original record supplies more.

The source acquisition and typed definitions do not establish converged flowsheets,
ordinary case acceptance, empirical certification or matched independent references.
Those obligations remain with Plan64b/64c and the exact Plan64h handoff.

The upstream [Wagner data documentation](https://chemicals.readthedocs.io/chemicals.vapor_pressure.html)
distinguishes the Poling2.5,5 and McGarry3,6 coefficient families. The
[CRC reference description](https://thermo.readthedocs.io/thermo.phase_change.html)
identifies `CRC_HVAP_298` as the298.15K value; this package uses that reference value,
not the library's optional temperature-adjustment algorithm.
