# Diagnostic threshold profiles

`idaes-2.13.json` is explicit settings data for `ModelingDiagnosticSettings.from_json`.
It is portable JSON, independently editable, and is not selected automatically or imported
as a modeling declaration package. Pass its contents to the settings constructor and pass
the resulting settings to the modeling package's diagnostic operation.

The threshold values come from the public configuration behavior of IDAES-PSE 2.13.0's
`DiagnosticsToolbox` and `SVDToolbox`. The Jacobian thresholds select its caution levels;
the rank cutoff selects its absolute small-singular-value threshold, and the singular-vector
threshold its size cutoff for the equations and variables a small singular value names. The source references
are [diagnostics toolbox](https://github.com/IDAES/idaes-pse/blob/2.13.0/idaes/core/util/diagnostics_tools/diagnostics_toolbox.py)
and [SVD toolbox](https://github.com/IDAES/idaes-pse/blob/2.13.0/idaes/core/util/diagnostics_tools/svd_toolbox.py).
The values were read from the pinned local source; no upstream implementation was copied
or executed for this profile.

The finding, dense-entry and combination limits are local execution choices. Change them
explicitly when a different work allowance is needed. Results retain incomplete evidence
when an allowance is exhausted.

PSE evaluates numerical findings in its resolved physical coordinates and nominal scaling.
Matching threshold values do not claim identical IDAES warning sets or numerical parity.
This file does not provide solve acceptance criteria or change a solve's outcome.
