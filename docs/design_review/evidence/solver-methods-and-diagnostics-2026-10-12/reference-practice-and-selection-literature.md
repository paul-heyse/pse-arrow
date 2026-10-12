> Supporting evidence for [the solver methods, selection and diagnostics review](../../reviews/design_review_solver-methods-selection-and-diagnostics_2026-10-12.md). Wave-1 worker return, B5b (library-research): IDAES 2.13 diagnostics and initialization behaviour, simulator reference practice and algorithm-selection literature, published unchanged by the coordinator. IDAES was read for behaviour only, under the clean-room rule. Baseline: `ff5a4aa2f` plus the Plan 36 working tree (diff fingerprint `45ceea51b0698679`). Citations marked † are from background knowledge. Its judgments are the worker's; the principal review owns the verdicts.

# B5b: How reference simulators and the algorithm-selection literature handle diagnostics, initialization and method selection

## Summary

- **The §15 statement is stale.** `numerical-execution.md` §15 says IDAES `DiagnosticsToolbox` parity "is not a current capability" and that there is "no named … check catalogue". The working tree does have one: a 25-rule `FindingKind` catalogue with explicit dispositions, a threshold policy, and a shipped `idaes-2.13` threshold profile. It also has an MILP degeneracy hunter, LP conditioning certificates and an ℓ1 deletion-filter explanation. *Interface-checked.*
- **Remaining gaps against IDAES 2.13 (Part 1 table):**
  - There is only one severity tier for Jacobian, variable and residual thresholds. IDAES has two: caution at 1e4/1e-4 and warning at 1e8/1e-8.
  - There is no "constraint with no free variables" check.
  - There is no solve-convergence regression analysis. IDAES's `IpoptConvergenceAnalysis` counts iterations, restoration and regularization against a stored baseline.
  - There is no scaling-profiler equivalent.
  - Ipopt cannot use HSL. `ma57` is refused by the settings decoder, citing ADR-0108. The maintainer has now ruled HSL admissible.
  - Homotopy steps grow geometrically on acceptance. IDAES and OpenModelica adapt the step to the observed solver effort.
- **Recycle convergence (Part 2).** Commercial practice is a priori defaults keyed to problem class, then staged switching driven by observation, then a rule table that maps the shape of the convergence trajectory to a remedy. Aspen's default table is: tears → Wegstein, one spec → Secant, several specs or tears plus specs → Broyden, optimization → SQP. pse-arrow has no Wegstein, Broyden or Newton tear method; tears go only through KINSOL fixed point with Anderson. It also has no "nested versus simultaneous" decision.
- **Algorithm selection (Part 3).** The advisory history layer should:
  - use cheap structural *features* as well as exact keys;
  - treat failures and timeouts as right-censored data;
  - score options lexicographically: verified acceptance first, then a PAR-style cost;
  - make every decision a pure function of a frozen, identified history snapshot;
  - partition or invalidate history when a solver build changes;
  - fall back to the a priori order until there is enough evidence.

---

## Part 1. IDAES-PSE 2.13: diagnostics, initialization and solver defaults

Paths below are relative to `.claude/skills/pyomo-and-solvers/content/sources/idaes-pse-2.13.0/idaes/` unless marked `pyomo:`. All of Part 1 is *Interface-checked* against the pinned local source, read for behaviour only. Nothing was run.

### 1.1 What `DiagnosticsToolbox` checks

**Configuration and thresholds** (`core/util/diagnostics_tools/diagnostics_toolbox.py:95-250`):

| Setting | Default |
|---|---|
| Near-bound distance | abs 1e-4, rel 1e-4 |
| Bound violation | 0 |
| Constraint residual | 1e-5, absolute |
| Term mismatch | 1e6 |
| Term cancellation | 1e-4, up to 5 terms |
| Term zero | 1e-10 |
| Variable large / small / zero | 1e4 / 1e-4 / 1e-8 |
| Jacobian caution | > 1e4 or < 1e-4 |
| Jacobian warning | > 1e8 or < 1e-8 |
| Parallel components | 1e-8 |
| Infeasibility-explanation feasibility | 1e-6 |
| `warn_for_evaluation_error_at_bounds` | True |

**Structural warnings** (`_collect_structural_warnings`, `:1224-1286`):
- degrees of freedom ≠ 0;
- inconsistent units (`pyomo.util.check_units`);
- structural singularity from the Dulmage–Mendelsohn partition (`IncidenceGraphInterface`, equalities only), with under- and over-constrained variable and constraint counts per independent block;
- potential evaluation errors (`evaluation_error.py`: log or log10 of a possibly non-positive argument, sqrt, pow, division, tan, asin and acos checked against variable bounds).

**Structural cautions** (`:1288-1318`): variables fixed to 0, and unused variables (with how many of them are fixed).

**Numerical warnings** (`:1320-1438`):
- large residuals (with suggested next steps `display_constraints_with_large_residuals` and `compute_infeasibility_explanation`);
- variables at or outside bounds;
- Jacobian column and row norms outside the warning thresholds;
- parallel constraints and variables.

**Numerical cautions** (`:1440-1580`):
- near bounds, near zero, extreme values, `None` values;
- constraints with mismatched terms, cancelling terms, or no free variables;
- Jacobian rows, columns and entries outside the caution thresholds.

`report_numerical_issues` also prints the **Jacobian condition number**. It uses `jacobian_cond(jac, scaled=True)` (`core/scaling/util.py:920-950`), which is the *Frobenius* condition number computed by explicit sparse inversion, or a dense pseudoinverse when the matrix is not square.

**Parallelism test** (`diagnostics_tools/utils.py:35-136`, after Klotz 2014): vectors are first bucketed by their non-negligible sparsity pattern. A pair is parallel when `||u||·||v|| − |u·v|` ≤ tol, either absolutely or relative to the larger of the two norms.

**Jacobian used.** `get_jacobian` (`core/scaling/util.py:829-918`) comes from PyNumero. By default it is scaled by the user scaling suffixes. It can optionally include a replica of Ipopt's gradient-based autoscaling (`max_grad` 100, `min_scale` 1e-8).

**Naming.** Findings name Pyomo component names (`v.name`, `c.name`) and pair a summary line with a suggested next-step method (for example `display_underconstrained_set()`). The API also has `assert_no_structural_warnings` and `assert_no_numerical_warnings`.

### 1.2 Advanced analyses in IDAES

- **SVDToolbox** (`svd_toolbox.py:118-260`): computes up to 10 of the smallest singular values by default. `svd_dense` (scipy) is the default, `svd_sparse` (svds) the alternative. "Small" means below 1e-6. A variable or constraint is attributed to a mode when its singular-vector component is at least 0.1. Views: rank of the equality constraints, under-determined variables and constraints per mode, and constraint ↔ variable lookups.
- **Degeneracy hunter** (`degeneracy_hunter.py:66-485`). The default MILP solver is **SCIP**, with M = 1e5, m_small = 1e-5 and a Jacobian-row tolerance of 1e-6. It works in two steps:
  1. A candidate MILP finds weights ν, with binary sign indicators, such that Jᵀν = 0.
  2. For each candidate it solves a minimum-support MILP, `min Σ y_c`, with that candidate's weight fixed. This yields the irreducible degenerate sets.
- **`compute_ill_conditioning_certificate`** (`ill_conditioning.py:28`, after Klotz): an LP or NLP that finds the rows or columns contributing to ill-conditioning (`ratio_cutoff` 1e-4).
- **`compute_infeasibility_explanation`** (`diagnostics_toolbox.py:644-686`) delegates to `pyomo.contrib.iis.mis`. That routine first relaxes the problem elastically to find sets whose relaxation restores feasibility, then runs a deletion filter to an approximate minimal infeasible system. The default solver is `ipopt_v2`.
- **`IpoptConvergenceAnalysis`** (`convergence_analysis.py:65-505`) runs a parameter sweep, rebuilding the model for each point. For each point it records:
  - Ipopt iterations;
  - iterations in restoration;
  - iterations with regularization;
  - time;
  - the numerical warnings from `assert_no_numerical_warnings`.

  These are parsed from the Ipopt log (`:380-425`), with per-run limits of 500 iterations and 120 s. `compare_convergence_to_baseline` and `assert_baseline_comparison` diff the results against a JSON baseline (iteration rel_tol 0.1, abs_tol 1). It is a *convergence-regression* tool.
- **`ipopt_solve_halt_on_error`** and the `bounds.py` valid-range helpers.

**Scaling tools:**
- `report_scaling_issues` (`core/util/scaling.py:1678`; large 1e4, small 1e-3, zero 1e-10) lists unscaled variables, badly scaled variables and unscaled constraints.
- `constraint_scaling_transform` (`:449`).
- `AutoScaler.scale_variables_by_magnitude` and `scale_constraints_by_jacobian_norm` (`core/scaling/autoscaling.py:49, 94`).
- Nominal-value scalers (`custom_scaler_base.py:762, 810`).
- `ScalerProfiler` (`core/scaling/scaler_profiling.py`) compares scaling methods by condition number and by whether a perturbed state re-solves.

### 1.3 IDAES initialization practice

- **`InitializerBase.initialize`** (`core/initialization/initializer_base.py:149-205`) always runs the same lifecycle: save state → load guesses → fix initialization states → `precheck` → routine → restore state → `postcheck`. The postcheck uses a residual tolerance of 1e-5 and checks for uninitialized variables. The modular default solver is `ipopt_v2` (`:552-580`).
- **`BlockTriangularizationInitializer`** (`block_triangularization.py:49-230`):
  - `precheck` requires a perfect matching (`:148-161`).
  - It then calls Pyomo's `solve_strongly_connected_components`. 1×1 blocks use `calculate_variable_from_constraint`, a Newton solve. N×N blocks use `ipopt_v2` with tol 1e-8, max_iter 200, linear presolve and model scaling on.
  - A **final full-block solve** follows, because the SCC solves ignore scaling (comment citing Pyomo #3785).
- **Hierarchical unit initializers** (`general_hierarchical.py`: `SingleControlVolumeUnitInitializer`) initialize properties, then the control volume, then the unit. Older helpers include `propagate_state` and `initialize_by_time_element` (`core/util/initialization.py`).
- **Sequential decomposition** (`pyomo:network/decomposition.py:40-150`):

  | Option | Default |
  |---|---|
  | `select_tear_method` | `"mip"` (alternative `"heuristic"`) |
  | `tear_method` | `"Direct"` (alternative `"Wegstein"`) |
  | Wegstein acceleration bounds | [-5, 0] |
  | `iterLim` | 40 |
  | `tol` | 1e-5, absolute |
  | `tear_solver` | `"cplex"` |

  The tear MIP (`:752-800`) enumerates cycles and minimizes `1000·(max times any cycle is torn) + number of tears`. The heuristic is a branch-and-bound over the same criteria (`pyomo:network/foqus_graph.py:525`).
- **Homotopy meta-solver** (`core/solvers/homotopy.py:21-140`). It moves fixed variables toward targets with an adaptive step:

  | Parameter | Default |
  |---|---|
  | `step_init` | 0.1 |
  | `step_cut` | 0.5 |
  | `iter_target` | 4 |
  | `step_accel` | 0.5 |
  | step bounds | [0.05, 1] |
  | per-step limits | 50 iterations, 10 s |
  | `max_eval` | 200 |

  The step adapts to the **solver iteration count of the last step**.

### 1.4 IDAES default solvers and options

`config.py:278-400`. `ipopt` and `ipopt_v2` share these defaults:

| Option | Default |
|---|---|
| `nlp_scaling_method` | `gradient-based` |
| `tol` | 1e-6 |
| `max_iter` | 200 |
| `linear_solver` (`ipopt_v2` only) | **`ma57`** |
| `writer_config.scale_model` (`ipopt_v2`) | True (user scaling suffixes applied in the NL writer) |
| `writer_config.linear_presolve` (`ipopt_v2`) | True |

`ipopt_l1` (`core/solvers/ipopt_l1.py`) is an alternative executable. A PETSc time-stepper wrapper exists (`core/solvers/petsc.py`).

`ipopt-watertap` is a **WaterTAP** wrapper, not part of IDAES. Per its public source it sets `tol` and `constr_viol_tol` to 1e-8, and with user scaling it applies bound relaxation itself and then sets Ipopt's `bound_relax_factor` to 0 ([watertap solvers source](https://watertap.readthedocs.io/en/latest/_modules/watertap/core/solvers.html)). That claim comes from web snippets only.

### 1.5 Capability comparison: IDAES 2.13 against the pse-arrow working tree

pse-arrow pointers:
- `RTD` = `crates/pse-runtime/src/workflow/modeling/diagnostics.rs`
- `MatrixPolicy` and `TermPolicy` = `crates/pse-math/src/diagnostics.rs:11-26, 307-320`
- `FindingKind` = `RTD:180-206`, dispositions `RTD:260-289`
- profile = `packages/reference/diagnostics/idaes-2.13.json`

| IDAES check or tool | pse-arrow equivalent | Notes |
|---|---|---|
| DOF ≠ 0, DM under- and over-constrained sets | `StructuralUnderdetermined` / `StructuralOverdetermined` (InvalidModel, Error); §15.2–15.3 pounce-presolve Hopcroft–Karp, DM and BTF; refusals name semantic IDs | Present, and stricter: it is an admission refusal, not a warning |
| Inconsistent units | Physical admission (§8), before structure | Present, in a different place (§15.2 says so) |
| Potential evaluation errors | `DomainPotentialEvaluationError` (InvalidModel, **Info**) | Present. IDAES severity is *warning*, pse is *info* |
| Variables fixed to 0; unused variables | `VariableFixedZero` (Info), `VariableUnused` (Info), plus `VariableOnlyInInequalities` (no IDAES equivalent) | Present |
| Large residuals | `EquationLargeResidual` (Warning) | Present. IDAES threshold is absolute 1e-5; pse threshold is relative to the row nominal (`RTD:88`) |
| Variables at or outside bounds; near bounds | `VariableOutside{Lower,Upper}Bound` (Warning), `VariableNear{Lower,Upper}Bound` (Info) | Present |
| Near-zero, extreme, `None` values | `VariableSmallValue`, `VariableLargeValue` (Info), `VariableMissingValue` (Error), `VariableNonfinite` | Present |
| Mismatched or cancelling terms | `EquationMismatchedTerm`, `EquationCancelingTerms` (Info) via `TermPolicy` (bounded subset enumeration, `complete` flag) | Present |
| Constraints with no free variables | **No equivalent found** | Absent. Search: `FindingKind` list and `rg` for no-free or external-variable names in pse-runtime and pse-diagnostics |
| Extreme Jacobian rows, columns, entries | `JacobianExtreme{Row,Column,Entry}` (Info) | Present, but **one threshold pair**. The profile uses the caution tier (1e4/1e-4); IDAES's 1e8/1e-8 *warning* tier has no counterpart, and severity is fixed per kind |
| Parallel constraints and variables | `JacobianParallelRows/Columns` (Warning) via `MatrixPolicy.parallel_tolerance` (1e-8) | Present. The metric differs: pse uses `1 − |cos|`, IDAES uses an absolute or max-norm-relative `|u||v| − |u·v|`. Same number, different test (profile README admits non-parity) |
| Jacobian condition number | `JacobianConditionEstimate` (Info): FERAL LU plus Hager–Higham 1-norm estimate (§15.5) | Present. pse gives an estimate and lower bound; IDAES gives an exact Frobenius norm by explicit inverse. pse's is more scalable; the numbers are not comparable |
| SVD toolbox | `MatrixPolicy`: dense-bounded full SVD, rank cutoff (abs 1e-6), singular-vector attribution 0.1, `JacobianNumericalRankDeficiency` (Warning); `ModelingDiagnostics.singular_modes()` (`python/pse/_modeling.py:439-480`) | Present, dense only, refused above the `dense_entries` budget (65,536 in the profile). There is no sparse smallest-k path like IDAES `svd_sparse` |
| Degeneracy hunter (IDS by MILP) | `BN/jacobian_diagnostics.rs` (`Policy`, `DegenerateSet{irreducible_at_tolerance}`), HiGHS session reuse (§18.10, `degeneracy_hunter_reuses_session`) | Present. Uses HiGHS rather than SCIP, with node and attempt budgets |
| Ill-conditioning certificate (Klotz) | Same module, "anchored conditioning certificates (LP family)" (`jacobian_diagnostics.rs:56-60`) | Present |
| `compute_infeasibility_explanation` (elastic, then MIS) | `ModelingPackage::explain_nonlinear` (deletion filter on the ℓ1 route); HiGHS IIS / rays / feasibility relaxation; SCIP IIS; `certify_infeasibility` (§15.5, §18.10.1) | Present and broader. The local result is labelled as not a proof |
| `IpoptConvergenceAnalysis` (sweep, iterations, restoration, regularization, baseline diff) | `diagnose_samples` checks diagnostics at points, not solve effort (`diagnostic_samples.rs`). Per-iteration Ipopt data is captured (`BN/tnlp.rs:560`, including `regularization_size`) | **No equivalent of the convergence-regression baseline found.** Search: `rg` for convergence-summary, restoration-count and compare-convergence names in crates and python/pse |
| `report_scaling_issues` (unscaled or badly scaled) | Not applicable as such: scaling comes from resolved physical nominals (§16.1); the diagnostic Jacobian is reported with `row_nominals` and `variable_nominals` | The "unscaled" concept is absent by design |
| `ScalerProfiler`, `AutoScaler` by Jacobian norm | Ipopt `gradient-based` toggled by `accuracy.native_scaling` (`BN/ipopt/sequence.rs:~308`) | **No scaling-method profiling found** |
| `report_*` / `assert_*` views | Python `ModelingDiagnostics.findings()`, `.table()`, `.complete`; no `assert_*` family | Partial |
| BT initializer (matching precheck, SCC solves, final solve) | §17.1 `initialization::Plan` → conditional blocks via KINSOL or the NLP runner; transactional commit with an original-quality check; final original solve (§17.5) | Present and stronger (transactional) |
| SequentialDecomposition (MIP or heuristic tears; Direct or Wegstein) | §17.4: exact HiGHS MILP with SCC-local order rows (no cycle enumeration) and authored group costs; greedy feedback-arc heuristic; KINSOL fixed point plus Anderson | Tear selection present, with a different objective (cost, not "max times a loop is torn"). **No Wegstein, Direct, Broyden or Newton tear method.** `rg -il "wegstein\|broyden"` over `crates/*/src` and the design sections finds nothing |
| Homotopy meta-solver | §17.5 staged homotopy over declared continuation endpoints; `StepControl` grows by a fixed factor (default 1.5) on acceptance and halves on retryable failure (`RT/math/strategy/path_control.rs:48-70`; `engines.rs:62-73`) | Present. **The step does not adapt to solver iterations**, unlike IDAES `iter_target` |
| Default Ipopt with `ma57` | Build manifest lists `mumps spral pardisomkl` (`BN/ipopt/runtime.rs:218`). The decoder rejects `{"kind":"ma57"}`: "HSL … not representable (ADR-0108 items 9 and 10)" (`BN/execution/tests.rs:669-678`) | **Absent, and now contradicted by the maintainer's HSL-admissible ruling** |

**Observed fact against interpretation.** The §15 sentence (`numerical-execution.md`, §15 intro) contradicts `FindingKind`, `ModelingDiagnosticPolicy` and the shipped profile. That is fact. My interpretation is that the sentence predates this machinery and is now owner drift. The real remaining differences from IDAES are the rows marked absent or partial above.

---

## Part 2. Commercial and open simulator practice

### Aspen Plus, sequential modular

Source: User Guide V10.1, chapter 17 ([PDF](https://user.eng.umd.edu/~nsw/chbe446/AspenPlusV10UserGuide2.pdf)), read in text form. *Interface-checked against public documentation.*

**A priori method table by block class:**

| Convergence block | Default method | Alternatives |
|---|---|---|
| Tears | Wegstein | Direct, Broyden, Newton |
| Single design spec | Secant | Broyden, Newton |
| Multiple design specs | Broyden | Newton |
| Tears and specs together | Broyden | Newton |
| Optimization | SQP | Complex for derivative-free bounded problems |

**Tear convergence test:**
- per variable, `|(X_calc − X_assumed)/X_assumed| ≤ tol`, with tol 1e-3;
- tested variables are total and component mole flows, pressure and enthalpy;
- trace components below tol/100 are either cut off or relaxed gradually;
- component groups and state choices reduce the matrix size for Broyden, Newton and SQP.

**Wegstein:**
- bounded q in [-5, 0], computed per variable;
- `Wait` 1 direct-substitution iteration before the first acceleration, then a configurable alternation of direct steps and acceleration steps;
- 30 flowsheet evaluations.

**Broyden and Newton staging:**
- `Wait` 2 direct-substitution iterations first;
- an option to converge tears to a tear tolerance first, then add the design specs;
- Newton re-forms its Jacobian only when the error reduction per iteration falls below the **reduction factor 0.2**, otherwise it reuses it;
- with active bounds on manipulated variables, Newton and Broyden minimize the tolerance-weighted sum of squares.

**SQP (Biegler).** Infeasible-path by default. A hybrid mode takes 3 tear iterations per optimization iteration.

**Sequencing:**
- `Design Spec Nesting` is **Inside (default), Outside or Simultaneous**;
- tear selection weights torn variables against torn loops (`Variable Weight`, `Loop Weight`, both 1);
- partial user tear sets are completed automatically.

**Diagnostics:**
- per iteration: block, method, iteration, what is being converged, number of unconverged variables, and **max Err/Tol** (nesting depth shown with `>`);
- level 5 adds a per-variable `X`, `G(X)`, new `X` and `Err/Tol` table;
- Tear History and Spec History sheets.

**Rule table from the trajectory of Err/Tol** (`Tear Convergence Suggestions`):

| Observed shape | Suggested remedy |
|---|---|
| Steady but slow | Component build-up (check exits) or a wider Wegstein lower bound (-20, then -50) |
| Oscillating | Wegstein upper bound 0.5 |
| Stalls at a threshold | Tighten inner-loop tolerances, or converge simultaneously |
| Broyden or Newton failing | `Wait` 4, converge tears first, or switch to Wegstein |

**Equation-oriented mode.** Public sources found this session confirm only the general pattern: run the sequential-modular solve, then initialize the EO model from it ([US patent 5,666,297](https://image-ppubs.uspto.gov/dirsearch-public/print/downloadPdf/5666297); [Ualberta EO getting-started](https://sites.ualberta.ca/CMENG/che312/F06ChE416/HysysDocs/AspenPlus20041GettingStartedEOModeling)). The DMO and LSSQP solver internals were **not verified**.

### Aspen HYSYS and gPROMS

No authoritative public documentation was found this session. Searches for HYSYS dominant-eigenvalue acceleration and gPROMS BDNLSOL or initialisation procedures returned nothing usable.

Background knowledge, *unverified*:
- HYSYS Recycle offers Wegstein and dominant-eigenvalue acceleration, with nested or simultaneous recycles.
- gPROMS uses a block-decomposition nonlinear solver with per-block Newton, and initialisation procedures that move from simplified to rigorous models.

Treat both as leads, not evidence.

### OpenModelica

Source: [simulation flags](https://openmodelica.org/doc/OpenModelicaUsersGuide/latest/simulationflags.html).

**Nonlinear solver** (`-nls`): `hybrid`, `kinsol`, `newton`, `homotopy` and **`mixed`**, which runs homotopy first and falls back to hybrid. The current default is not stated on the page. A 2017 commit log records a move to KINSOL as the standard ([history](https://libraries.openmodelica.org/branches/history/master/2016-2021/2017-07-17%2022:47:47..2017-07-18%2011:47:56.html)).

**A priori sparse/dense switch by mathematical facts:**

| Flag | Default |
|---|---|
| `-nlssMaxDensity` | 0.1 |
| `-nlssMinSize` | 1000 |
| `-lssMaxDensity` | 0.2 |
| `-lssMinSize` | 1000 |

**Linear solver.** LAPACK with total pivoting as the fallback (`-ls default`).

**Newton variants.** `damped`, `damped2`, `damped_ls`, `damped_bt`, `pure`.

**Initialization homotopy:**
- applied on the first try when the model contains `homotopy()`;
- adaptive τ: start 0.2, minimum 1e-4, maximum 10, increase factor 2, decrease factor 10;
- `homMaxTries` 10, `homMaxNewtonSteps` 20, with predictor and backtrace strategies;
- *local* homotopy runs per strongly connected component, *global* homotopy over the whole DAE ([trac #2266](https://trac.openmodelica.org/OpenModelica/ticket/2266)).

**Diagnostics:** `LOG_INIT`, `LOG_NLS(_V)`, `LOG_NLS_HOMOTOPY`, `LOG_RES_INIT`, and `-nlsInfo` CSV output per system.

### Modelica `homotopy()` operator (Dymola and OpenModelica)

Introduced in Modelica 3.2. The **model author** supplies an `actual` and a `simplified` expression, and the tool continues λ from 0 to 1 ([Sielemann et al. 2011](https://elib.dlr.de/73049/); [Casella slides](https://lccc.control.lth.se/media/LCCC2012/WorkshopSeptember/slides/Casella.pdf); probability-one variant: [Sielemann 2012](https://2012.international.conference.modelica.org/proceedings/html/pdf/ecp12076223_Sielemann.pdf)). Dymola's tearing and mixed-system internals were not verified.

### DWSIM and BioSTEAM

- **DWSIM** Recycle uses Wegstein with `AccelDelay`, `AccelFreq`, `Qmin` and `Qmax`, plus Broyden or dominant-eigenvalue options. Adjust blocks use secant or bisection ([API](https://dwsim.org/api_help/html/T_DWSIM_UnitOperations_SpecialOps_Helpers_Recycle_WegsteinParameters.htm), [tutorial](https://dwsim.org/tutorials/en/intermediate/04-recycle-loops.html)).
- **BioSTEAM** defaults to **Aitken–Steffensen**. Wegstein and fixed point are in-house; SciPy supplies Anderson and Broyden. A recycle converges when *either* the absolute or the relative molar tolerance is met ([docs](https://biosteam.readthedocs.io/en/latest/tutorial/Convergence.html)).

### Practice that bears on pse-arrow's decision logic

These are *Proposed* readings of established practice.

1. **Layer 1 is a table keyed to problem class**, not a single default:
   - tear-only loops → componentwise secant extrapolation (Wegstein or Aitken);
   - one scalar spec → secant with bracketing;
   - coupled multi-variable systems → quasi-Newton (Broyden), escalating to Newton with Jacobian reuse;
   - optimization → SQP or interior point.

   Size and density thresholds choose sparse against dense (OpenModelica). pse-arrow's fixed-point route has KINSOL plus Anderson only. Wegstein and Aitken are cheap, robust and bounded-q methods that the reference tools use as the default tear method. Broyden on tears is the standard escalation.
2. **Layer 2 is staged and keyed to observations:**
   - "wait N direct steps before acceleration";
   - "converge tears first to a looser tolerance, then add specs";
   - "re-form the Jacobian when the reduction ratio is above 0.2";
   - "mixed: homotopy, then hybrid";
   - homotopy step adapted to the Newton or Ipopt iteration count.

   These are the industrial versions of in-solve switching. pse-arrow's homotopy grows by a fixed factor.
3. **Nested against simultaneous is a first-class decision** (Aspen `Design Spec Nesting`). Simultaneous is the documented remedy for a stall caused by loose inner-loop tolerances. pse-arrow §17.4 offers simultaneous initialization "as an explicit alternative, never a failed-unit fallback". The reference practice treats choosing between them as a decision to make by rule or by observation.
4. **Tear objective.** Aspen weights variables against loops; Pyomo uses minimax tears per cycle (non-redundant tear sets); a minimum tear count is not always fastest ([USP lecture](https://edisciplinas.usp.br/pluginfile.php/5808176/mod_resource/content/1/Aula%20-%20Reciclo.pdf)). pse-arrow uses only authored costs. A world-class default should state its objective, such as a stable tear variable or few torn loops, rather than leave it implicit.
5. **Diagnostics are per iteration and comparable across methods.** The common form is max Err/Tol, the arg-max variable, nesting depth, and a per-variable `X`, `G(X)`, new `X` view, where Err/Tol < 1 means converged. This unit-free normalized signal is what the remedy rules key on. pse-arrow's "violation divided by tolerance" (§15.4) is the same quantity and could drive comparable layer-2 rules.
6. **Authored simplification as initialization** (the Modelica `homotopy()` operator) parallels pse-arrow's declared continuation endpoints. Local per-SCC homotopy, as distinct from global homotopy, is a further option worth noting.

---

## Part 3. Algorithm-selection literature for the advisory history layer

Citations marked † come from background knowledge and were not fetched this session. Those marked ✓ were verified by search this session.

### Framework and key work

- **Rice (1976)**, "The algorithm selection problem", *Advances in Computers* 15 †. Rice separates problem space P, feature space F, algorithm space A and performance space Y, with a selection mapping S: F → A.
  - The maintainer's layer 1 *restricts A per instance*: it decides what is admissible.
  - Layer 3 learns S over the admitted subset only.
  - Layer 2 is outside Rice's static model. It corresponds to *dynamic algorithm configuration* (Biedenkapp et al., ECAI 2020 †) and dynamic portfolios (Gagliolo & Schmidhuber 2006 †).
- **Per-instance selection:**
  - SATzilla (Xu, Hutter, Hoos & Leyton-Brown, JAIR 2008 †; the 2012 version uses cost-sensitive pairwise random forests †);
  - AutoFolio (Lindauer et al., JAIR 2015 †), which configures the selector itself;
  - ASlib (Bischl et al., AIJ 2016 †), which standardizes scenarios as features, feature costs, run status and runtimes, and scores with **PAR10** and the closed gap between the single best solver (SBS) and the virtual best solver (VBS).
- **Survey.** Kerschke, Hoos, Neumann & Trautmann, *Evolutionary Computation* 27(1):3–45, 2019 ✓ ([arXiv 1811.11597](https://arxiv.org/abs/1811.11597v1)). It covers discrete and continuous problems, features including exploratory landscape analysis, and the links between selection, configuration, scheduling and portfolios.
- **Configuration:** ParamILS (JAIR 2009 †), SMAC (LION 2011 †) and irace (ORP 2016 †). All three tune offline over an instance distribution and are prone to overfitting that distribution.
- **Portfolios and schedules:** 3S (Kadioglu et al., CP 2011 †), a static time-sliced pre-schedule followed by per-instance selection. Parallel portfolios are noted only: parallelism that comes purely from architecture is out of scope.
- **Online selection with censoring:** Tornede, Bengs & Hüllermeier, AAAI 2022 ✓ ([paper](https://ojs.aaai.org/index.php/AAAI/article/view/21279)). They adapt bandits to right-censored runtimes; Thompson-sampling variants did best on an ASlib adaptation.
- **Runtime prediction under censoring:** Hutter et al., AIJ 2014 †.
- **Solver-internal decisions learned and shipped:**
  - Berthold & Hendel, "Learning to scale MIPs", AAAI 2021 ✓ ([paper](https://ojs.aaai.org/index.php/AAAI/article/view/16482)). It chooses between Standard and Curtis–Reid scaling, has been the **FICO Xpress default since 8.9**, and is trained on a *numerical-robustness label* (the attention level), not solve time.
  - Bonami, Lodi & Zarpellon, CPAIOR 2018 †: whether to linearize an MIQP.
  - Kruber, Lübbecke & Parmentier, CPAIOR 2017 †: whether to decompose.
- **Simulation and PSE:**
  - Zabegaev, Berre & Keilegavlen 2025 ✓ ([arXiv 2510.04920](https://arxiv.org/abs/2510.04920)): online, in-run learned *linear-solver* selection for multiphysics simulation, comparable to an oracle policy trained on prior data.
  - Bugosen, Laird & Parker 2024 ✓ ([arXiv 2310.09307](https://arxiv.org/pdf/2310.09307)): success rates of Ipopt on 64 instances vary strongly by formulation (full-space 33, implicit 52, ALAMO 64, neural network 48). Formulation is a selection dimension.
  - ML-assisted flowsheet initialization ([arXiv 2307.13621](https://arxiv.org/html/2307.13621v2); Höller et al., AIChE J 2024, DOI 10.1002/aic.18404 ✓).
  - I found no published work on learned Ipopt *option* selection for process models. That is an empty search, not proof of absence.

### Lessons for a history layer that may only reorder or tune admitted options (*Proposed*)

1. **Features versus exact keys.**
   - An exact key such as case identity, numerical identity or a flow fingerprint is right for *replay*: same problem, so reuse the start or order. It generalizes to nothing.
   - The literature's power comes from cheap features that place unseen instances near seen ones. pse-arrow already computes candidates without extra cost: size, nnz and density; DM and BTF block counts and the largest block; tear count and SCC sizes; nonlinearity and convexity class; bound counts; nominal-scale spread; the condition estimate when available.
   - ASlib makes *feature cost* part of the scenario. Features must cost far less than a solve.
   - Recommendation: two tiers. Use the exact key when present; otherwise rank by features, for example k-nearest neighbours or a pairwise cost-sensitive model. Never use features to make an option admissible.
2. **Censored data.**
   - A timeout, iteration limit or failure is a lower bound on cost, not a sample. Dropping it biases towards optimism, and imputing the cutoff biases low.
   - Use PAR-k penalization for evaluation, and censoring-aware estimators (survival or Schmee–Hahn-style imputation †, or censored bandits ✓) for ranking.
   - Because pse-arrow records typed failures and budgets per attempt, the censoring point is known exactly.
3. **Exploration.**
   - A selector that always puts the historical best first never learns about the others. This is selection bias, and is why online selection uses bandits.
   - Under "reorder only", exploration is cheap and safe, because the layer-1 fallback chain still runs the rest. Exploration can be a bounded, *deterministic* permutation, for example on a seeded schedule, recorded in the run.
   - It must never skip an option layer 1 would try.
4. **Cost-sensitive objective.**
   - Score options lexicographically: (a) verified original-space acceptance (§15.4) against failure, then (b) the cost to acceptance, as PAR-k over wall time or a deterministic work unit such as function evaluations or factorizations.
   - Berthold–Hendel show that a robustness label can beat raw time as a training target.
   - Measure value as the closed SBS→VBS gap on a held-out set. Do not report raw speedups.
5. **Determinism and reproducibility.**
   - A decision should be a pure function of (admitted option set, features, history *snapshot identity*), with the snapshot hash in the run's identity, and stochastic selectors seeded from content hashes.
   - Offline-trained selectors, in the AutoFolio and ASlib style, are frozen artefacts, which fits this. Online updating changes decisions between runs and must be versioned explicitly.
6. **Concept drift.**
   - Algorithm-selection results are version-specific: ASlib scenarios fix solver versions. A change in Ipopt, HSL, KINSOL or HiGHS invalidates performance data.
   - Partition history by solver-build identity (pse-arrow already pins builds, for example the `ipopt_linear_solvers` manifest). Treat cross-version data as a weak prior with decay, and re-qualify before trusting it.
7. **Cold start.**
   - With no or thin history, use the layer-1 order, which plays the role of the single best solver.
   - Require a minimum effective sample count and a confidence margin before reordering. Feature-based transfer softens cold start for new cases; exact keys cannot.
8. **Configuration within admitted options.**
   - Run SMAC or irace *offline* on a qualified instance set to produce per-class option profiles, then freeze them as authored profiles.
   - Use irace-style racing with statistical tests to retire dominated options.
   - Watch for overfitting to the benchmark set and validate on held-out flowsheets.
9. **Time-sliced schedules.** pse-arrow's ordered fallback with per-attempt budgets *is* a time-sliced schedule. History may legitimately tune slice budgets as well as the order, as in 3S. Keep a floor so no admitted option is starved.

---

## Probes I would run (none were run)

1. **Do pse findings match IDAES cautions on a shared fixture?** Run the parity harness on one flash-and-recycle fixture: IDAES `report_numerical_issues` against pse `diagnose` with the `idaes-2.13` profile, comparing counts per rule. Expect matches for bounds, values and term rules; differences in parallelism (different metric), condition number (Frobenius against a 1-norm estimate), residuals (absolute against nominal-relative), and the 1e8 warning tier.
2. **Is HSL ma57 loadable in the pinned Ipopt build?** Read the Ipopt build manifest and check `ipopt_linear_solvers` for `ma57`. Expect the current `mumps spral pardisomkl`, so admitting HSL needs a build change and a settings-vocabulary change.

## Search coverage and limits

- **IDAES and Pyomo:** read directly from the skill's pinned 2.13.0 and 6.10.1 trees.
- **pse-arrow:**
  - read: `numerical-execution.md` §15–§17, the `RTD` policy, `FindingKind` and dispositions, `MatrixPolicy` and `TermPolicy`, the `jacobian_diagnostics` header, the profile JSON and README, `StepControl`, and the Ipopt option code;
  - searched with `rg`: Wegstein and Broyden, HSL, convergence-analysis names, no-free-variable names, and history or advisory terms (none in `numerical-execution.md`).
- **Public documentation:** the Aspen Plus chapter 17 text was read in full. HYSYS, gPROMS, Aspen EO internals and Dymola were not found in public sources; the statements about them are marked unverified background. Citations marked † are from memory.

No files were written beyond scratch text in `/tmp/claude-1000/-home-paul-pse-arrow/5335360e-031d-4d0d-ac32-d1df2e1106de/scratchpad/B5b/aspen.txt` (the extracted Aspen manual).