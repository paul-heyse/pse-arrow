# Simulator acceptance contract

Plan 07 UD00 / ADR-0068. **Proposed oracles; runtime qualification remains open.**
`just simulator-acceptance <new-output-directory>` runs the ordinary Rust simulator
integration target and cold Python integration target. Missing targets, missing
solvers, unavailable backends and incomplete outcomes fail; P10 inspection is not a
substitute. All tests use a zero-failure baseline and Rust force validation.

## Declaration coverage

These are references to the shipped source declarations, not another editable
registry. Every listed method is exercised with its own parameters, quantity/basis/
reference contract and invalid-domain behavior. The native/Pyomo/NL adapter must
report unsupported bindings explicitly until delivered.

| Source | Name | Declaration ID |
|---|---|---|
| `methods/methods/correlations.yaml` | NIST.Shomate.cp | `e91e27d03e454351b534e2fbe01df028` |
| `methods/methods/correlations.yaml` | NIST.Shomate.h | `9a8cfddf1cfe4174b16b9cae00886a5e` |
| `methods/methods/correlations.yaml` | NIST.Shomate.s | `9f779daa99854ab1b707977e7d054c04` |
| `methods/methods/correlations.yaml` | RPP4.cp | `f3b769aa4e1f46ac802051d1123bb522` |
| `methods/methods/correlations.yaml` | RPP4.h | `896f59a263f54103ac8d8cb0dd9ca06b` |
| `methods/methods/correlations.yaml` | RPP4.s | `981027ad2c8e4bdb9e5083b96af34e41` |
| `methods/methods/correlations.yaml` | Perry.liquid.cp | `06535666496e4acc804aae4b25859046` |
| `methods/methods/correlations.yaml` | Perry.liquid.h | `f47bfef6acf44b29978030a37b558676` |
| `methods/methods/correlations.yaml` | Perry.liquid.s | `6c92c46efd71453180ae182240b4ed96` |
| `methods/methods/correlations.yaml` | Perry.liquid.density | `28313358ec4642eab1766802eec46169` |
| `methods/methods/correlations.yaml` | Ideal.gas.density | `221c683b475043a7bbe45f2fc3911698` |
| `methods/methods/correlations.yaml` | Ideal.gas.enthalpy | `e7994cef1d6043c0a3de1706aa33c10c` |
| `methods/methods/correlations.yaml` | Ideal.liquid.enthalpy | `cba7f69ae7d945f6a7e0206279979cbb` |
| `methods/methods/correlations.yaml` | Ideal.liquid.density | `59a34558695646668d3979346891bec7` |
| `states/methods/states.yaml` | FTPx | `bb3ea14061b24335a0c78a25c67e67b6` |
| `states/methods/states.yaml` | FcTP | `c207b61b96124774b194cd76024f772f` |
| `units/templates/units.yaml` | Flowsheet | `e3f9d5f552ee4e4ca0fb00408a597cef` |
| `units/templates/units.yaml` | LumpedControlVolume | `944c2261a26243a3ba12cbabfa5c81ec` |
| `units/templates/units.yaml` | Heater | `242af871dd464fcb82edcad1991b0f13` |
| `units/templates/units.yaml` | Feed | `47b9a6169f1842d8809568cb2cf4d1f7` |
| `units/templates/units.yaml` | Product | `3414cba5df7f494aa697637e0e620a84` |
| `units/templates/units.yaml` | StateJunction | `ae816e50c8794aea9d656b78b229fb70` |
| `units/templates/units.yaml` | EqualityConnection | `b85e948565bb46db98cddceae395febc` |
| `units/templates/units.yaml` | Mixer | `26a0c885415b4cda8f3628f5fc75030b` |

Units, elements, quantity/basis/reference data and material packages resolve through
`packages/reference/{physical,elements,thermo-examples}`. Their actual selected IDs
and source spans must appear in each fresh run's evidence.

## Source cases and independent assertions

| Case | Scope | Oracle |
|---|---|---|
| heater-ftpx, heater-fctp | Nitrogen ideal vapor, each state basis; fixed duty and fixed outlet temperature cases | Component flow and pressure conservation; `Q = F(h_out - h_in)`; specification changes alter DOF correctly |
| mixer-ftpx, mixer-fctp | Two unequal nitrogen feeds, each state basis | Species/total flow sums; `F_out h_out = sum(F_in h_in)`; nonconstant Cp gives an actual nonlinear temperature solve |
| connected-bt-vapor | Two feeds → mixer → heater → product, benzene/toluene vapor | Independent component/energy balances, pressure equalities, published RPP Cp points and enthalpy integral; rename preserves IDs and bindings |
| connected-bt-liquid | Benzene/toluene liquid, Perry methods and ideal liquid mixing | Component/energy balances, temperature-domain checks, independent density/Cp anchors and integral identities |
| analytic-discretization | `dy/dt = 2t`, `y(0)=0`, integral over `[0,1]` | `y(t)=t²`, integral `1/3`; declared discretization order and grid-convergence error, derivative units and initial condition |
| invalid/failure cases | Bad units, missing/ambiguous method, dangling target, bounds, under/overdetermination, infeasible solve, cancellation, resource limit | Distinct structured failure, no successful publication/run, no silent fallback |
| edit/reuse/lifecycle | Parameter edit, topology edit, rename, source replacement, stale parent, concurrent publication, uncertain commit, cold reopen, reader retention | Exact selected versions/absence/implementation/settings; conservative recomputation valid; unchanged identity does not imply unchanged meaning |
| extensions | New method, new invariant and external provider | One declaration and required implementation/tests; normal planning, admission, storage and inspection need no new dispatch branch |

The connected cases execute native Ipopt, generated Pyomo and NL/SOL from the same
problem relation. Every route must actually solve and ingest results. Initializer
steps and scaling are evaluated by their effect on the same case-bound problem.
Comparisons establish semantic outcomes, never equivalence to predecessor graphs.

## Numerical oracles and tolerances

Reuse the independent anchors already recorded in
`xtask/src/codegen/physical/tests/formulas.rs` and the cited primary sources in
`packages/reference/methods/sources.md` and
`packages/reference/thermo-examples/sources.md`. Qualification must evaluate the
compiled native/backend expressions, not only the authored template interpreter.

- Nitrogen NIST JANAF at 300/400/500 K: Cp 29.12/29.25/29.58 J/(mol K)
  (absolute tolerance 0.005), entropy 191.8/200.2/206.7 J/(mol K) (0.05),
  reference-relative enthalpy 50/2970/5910 J/mol (5.1).
- RPP fitted benzene Cp at 298.15/400/500/600 K:
  82.44/113.52/139.35/160.09 J/(mol K); toluene:
  103.7/139.9/170.8/196.2. Check declared-fit evaluation at 1e-8 absolute;
  the fit's agreement with a physical source uses its source uncertainty separately.
- Perry liquid molar densities: benzene 11,421 mol/m³ at 278.68 K and toluene
  10,495 mol/m³ at 178.18 K, absolute tolerance 5 mol/m³. Benzene liquid Cp at
  298.15 K: 135.69 J/(mol K), tolerance 0.2.
- All three correlation families: independent numerical quadrature/differentiation
  checks `dh/dT = Cp` and `ds/dT = Cp/T` inside each validity interval, including
  nonzero basis/reference offsets. Use central-difference step refinement and report
  the step/error; backend-to-backend agreement alone is insufficient.
- Ideal gas density at 300 K and 100,000 Pa: 40.090785014242016 mol/m³,
  absolute tolerance 1e-10 for the declared gas constant.
- Simulator values: default absolute tolerance 1e-8 and relative tolerance 1e-6,
  with dimensional/scaled residual checks reported separately. Jacobian checks use
  independent finite differences at multiple interior points and steps; inactive
  guarded branches must not be evaluated. Nonfinite/failure outcomes are compared
  categorically, not hidden by numerical tolerances.

## Receipt requirements

Evidence selects exact sources, declaration IDs, model/case/publication versions,
backend/options/derivative profile, state basis, objective/DOF, convergence status,
values/residuals/Jacobian checks, independent oracle and tolerances. Cold Rust/SQL/
Python queries select the same publication. Edits, supported reconstruction,
retention and extension/failure results accompany cost measurements at small and
scaled workloads. A receipt is complete only when every Plan 07 S01–S11 and V01–V13
obligation is established; each G1–G7 decision still requires independent review.
