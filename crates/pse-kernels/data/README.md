# Light-hydrocarbon thermodynamic data

These ordered methane/ethane/propane subsets come from FeOS commit
`c658aeab484f7a7096bfbf5425e40effd60da167` (the 0.10.1 release source):

- [`parameters/pcsaft/gross2001.json`](https://github.com/feos-org/feos/blob/c658aeab484f7a7096bfbf5425e40effd60da167/parameters/pcsaft/gross2001.json), upstream SHA-256
  `f4b4018c7f02341b937c086cbb38626b72c1f3f3d9677a26b463112e16dfd404`.
- [`parameters/ideal_gas/poling2000.json`](https://github.com/feos-org/feos/blob/c658aeab484f7a7096bfbf5425e40effd60da167/parameters/ideal_gas/poling2000.json), upstream SHA-256
  `8cd2761201f0c1942679a58d9266df39829e84e79744e08bd46700cc67ed08f8`.

The PC-SAFT records cite Gross and Sadowski (2001); the ideal-gas heat-capacity
records cite Poling et al. (2000). No proprietary DIPPR database is distributed.
FeOS implements the DIPPR equation forms used by these published coefficients.
The package explicitly selects zero binary interactions, a predictive assumption,
not fitted mixture parameters. Content identity covers the actual embedded records.
Caloric values use FeOS's 298.15 K integration convention without formation enthalpy.
No independently certified empirical operating envelope is claimed before M22.

Upstream FeOS is MIT OR Apache-2.0; imported records retain FeOS contributor
attribution in `REUSE.toml`.
