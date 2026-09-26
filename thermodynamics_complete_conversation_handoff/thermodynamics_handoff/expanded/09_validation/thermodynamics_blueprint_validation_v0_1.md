# Thermodynamics blueprint validation and conformance readiness

**Document:** THERMO-VALIDATION-009  
**Version:** 0.1  
**Project date:** 25 September 2026  
**Stage:** Step 9, validate the blueprint before translating it into Rust  
**Status:** Conditional conceptual-design acceptance with bounded numerical and behavioral evidence. Complete-provider, real-flowsheet and independent physical-validation gates remain open.

## 0. Decision and scope of acceptance

**Proceed with implementation design of the established semantic and behavioral core, subject to the acceptance refinements below. Do not treat this review as approval of a production thermodynamics backend or a completed P1/P2 simulator.**

The architecture supports the reviewed distinctions without requiring an ownership redesign: material versus representation; shared definition versus local state; phase properties versus equilibrium; chemistry meaning versus numerical coupling; local candidates versus parent convergence; acceptance versus current publication. The review found no blocking ownership or information contradiction in the 34 scoped walkthroughs. That is an authored, bounded judgment, not a proof for every future material or an independent external review.

This stage adds actual execution beyond catalog checks, but at carefully identified boundaries:

| Evidence obtained | Result | Claim limit |
| --- | --- | --- |
| New validation-package structural audit | 49 checks passed | IDs, evidence boundaries, recorded outputs and artifact integrity only. |
| Re-executed Step-7/8 structural audits | 35 and 42 checks passed | The supplied catalogs and traces remain consistent. |
| Re-executed Step-7/8 reference tests | 26 and 43 checks passed | Inherited authored models, not native thermodynamics or production software. |
| New numerical probe records | 30 passed | Two extracted upstream functions plus authored host/formulation tests. |
| Generated allocation subcases | 200 feasible five-component cases | Fixed/generated K-value algebra, not real-mixture equilibrium data. |
| Publication-model schedules | 1,120 legal schedules, 8,960 event steps, zero recorded violations | Bounded sequential event model, not deployed concurrency or storage. |
| Focused publication guard combinations | 128 combinations | Boolean admission semantics only. |
| Deliberately weakened guard/alias controls | Eight detected | Tests distinguish these unsafe variants; no claim about uninspected production code. |
| Full-package worker probes | Six blocked before execution | Candidate packages were absent and installation/download access failed here. |
| Real-fluid independent validation | None | Same-model published examples and invented materials are not experimental evidence. |

The user-facing configured property package remains the coherent unit of engineering choice. No new backend, solver, data fabric, Rust dependency, physical schema or production API is selected here.

### Evidence navigation

The [machine-readable register](thermodynamics_blueprint_validation_v0_1_register.json) contains scenario judgments, requirements, experiment and profile dispositions, and findings. The [numerical results](results/numerical_probes.json), [bounded publication results](results/publication_model_checks.json), [native dispatch results](results/native_probe_dispatch.json), and [environment record](results/environment_preflight.json) are actual outputs of this session. [Fixtures](fixtures.json) declare the selected numerical assumptions and tolerances. [Sources](source_manifest.json) distinguish the extracted code from documentation and blocked native-worker targets.

## 1. Validation method and evidence precedence

B1 remains the scope authority, B4 the normative requirement baseline, B5 the responsibility allocation, B6 the information dictionary, B7 the action/lifecycle contract, and B8 the integration admission rules. The complete bundle retains those files unchanged under `baselines/`. The new register is an evidence overlay; it does not silently edit original test statuses, scenario profiles or requirement obligations.

Four workstreams were completed: reconstruct and audit the supplied baselines; perform adversarial information/action walkthroughs of all 34 scenarios; exercise available bounded numerical components and authored process formulations; and explore publication/revision/cancellation behavior with explicit negative controls.

**Test subject is mandatory evidence.** An extracted function, an installed package, a host solver, an adapter, a unit model, and an accepted flowsheet are not equivalent subjects. Each new test identifies which one it actually exercises. No upstream library inherits the success of the authored scaffolds.

### Environmental limitation and the narrower execution route

The runtime contained NumPy and SciPy, but none of `thermo`, `chemicals`, `CoolProp`, `thermopack`, `feos`, `idaes`, `pyomo` or `reaktoro`. Julia and .NET were absent. Package-index access and a direct metadata request failed, including a recorded temporary name-resolution error. A separate official-wheel download also failed. No remote execution service was used. These are environment observations, not evidence that a library is generally unavailable or cannot be installed normally.

Public source retrieval through the GitHub connector remained available. Two small self-contained numerical bodies were transcribed from pinned ChEDL source: `NRTL_gammas` and `Rachford_Rice_flash_error`. Their computational bodies are retained, with the original package imports/docstrings omitted, `math.exp` supplied for NRTL, and copyright/license notices preserved. The extracted module has its own checksum. Its checksum is not presented as the upstream file checksum.

This provided a useful but narrower route: source-function regressions and host/formulation experiments. It did **not** install either complete package, exercise its database, constructors, flash dispatchers, native dependencies or upstream test suite. The new [optional full-package runner](scripts/run_native_probes.py) records missing prerequisites as BLOCKED, never as a successful skipped test. Its dependency-absent path was executed; its package-dependent workers remain untested in this session.

## 2. Conformance-suite structure

The validation package separates evidence so later runs can add genuine native results without relabeling the present work.

| Suite | Included artifact | Responsibility and acceptance rule |
| --- | --- | --- |
| Baseline integrity | `validate_validation_package.py`; unchanged predecessor validators | Verify exact source hashes, IDs, owners, scenarios, traces and local artifact links. |
| Source-kernel regression | `run_numerical_probes.py`, NP-01..14 | Reproduce published function examples, ordered coordinates and algebraic limits; record source-extraction limitations. |
| Numerical formulation | Same runner, NP-15..30 | Solve the specified synthetic VLE/LLE/thermal/chemical problems; verify original residuals and semantic differences. |
| Behavioral exploration | `check_publication_model.py` | Explore all legal schedules in its explicitly bounded model; require counterexamples for deliberately weakened guard variants. |
| Native package admission | `run_native_probes.py` | Check exact requested version, execute in a subprocess when available, preserve failure output, never substitute a different backend. |
| Architecture review | VR-01..34 in this report/register | Walk through required information, authority, action sequence and adversarial case, and retain any missing numerical witness. |
| Physical validation | Existing QG-09 and real-profile witnesses | Still requires independent physical reference data, source uncertainty, exact operating envelope and declared tolerances. |

The fixtures and numerical tolerances are read before the recorded test run. They are tolerances for these authored/regression cases, not a universal engineering accuracy standard. During harness review, the exposed reaction solve was given an independently specified initial guess instead of the already converged condensed solution; the recorded final fixture revision and outputs identify this stronger test. No tolerance was relaxed to conceal a failed physical result.

## 3. What the extracted numerical functions establish

### 3.1 NRTL example and formula-qualified data

At `xs=[0.252,0.748]`, `tau12=-0.178`, `tau21=1.963`, and off-diagonal `alpha=0.2974`, the extracted function returned:

`gamma = [1.9363183763514304, 1.1537609663170014]`.

This reproduces the upstream documented example within the declared absolute tolerance. The test also checks ideal and pure-component limits, all six permutations of an authored three-component parameter system, and a finite-difference Gibbs–Duhem tangent. [SRC-01, SRC-03; NP-01, NP-04, NP-06, NP-07]

A **separate** temperature-dependent parameter fixture uses the documented energy coefficients and `tau_ij=b_ij/(R T)`. The host evaluates `G^E=R T sum(x_i ln gamma_i)` and obtains `H^E=G^E-T(dG^E/dT)` by centered differentiation. The result, **582.964853882 J/mol**, agrees with the documented rounded 582.96485 J/mol. This exercises the extracted gamma function plus authored differentiation, not the installed upstream `HE()` method. The rounded fixed-tau example and energy-parameter example are not treated as identical data. [SRC-03; NP-08]

Two negative controls matter architecturally. Supplying `xs=[0.6,0.6]` to the raw extracted function returns finite values identical to the normalized `[0.5,0.5]` case. The host validation wrapper rejects that input rather than silently normalizing it. Transposing directed interaction parameters likewise returns plausible but wrong example values; the maximum reference discrepancy is approximately 0.114909. [NP-09, NP-10]

**Conclusion:** the numerical kernel is useful, but it is not the complete material/preparation/adapter contract. This is not an allegation about validation elsewhere in the complete library, which was not executed.

### 3.2 Rachford–Rice residual and allocation

For the upstream example `z=[0.5,0.3,0.2]` and `K=[1.685,0.742,0.532]`, the extracted residual at beta=0.5 is **0.04406445591174976**. A host SciPy Brent solve of that residual gives beta approximately **0.690730262773855**, matching the documented allocation. The host reconstructs each component and checks both phase normalizations. [SRC-02, SRC-04; NP-02, NP-03]

The host also checks component permutation, absent phases without fictitious compositions, a zero-feed constituent, and 200 deterministically generated feasible five-component cases. Generated K-values were deliberately constructed from known positive phase compositions and an interior beta; they are not predictions of real phase equilibrium. [NP-05, NP-11, NP-14, NP-28]

If every active K_i equals one, the Rachford–Rice residual is zero for every beta. The standalone fixed-K allocation problem therefore cannot supply a unique phase fraction. Near that degeneracy, a test residual and its beta derivative are both of order 10^-16: a tiny residual does not certify a well-determined fraction or a numerically regular solved sensitivity. [NP-12, NP-13]

The remedy is request-specific. A pure-fluid PH problem can still determine phase fraction through its additional energy constraint. It must use a qualified full formulation, not reject the whole problem merely because one inner equation is degenerate. Conversely, a sign-changing bracket across a jump is not sufficient evidence of a root. The authored discontinuity control makes a root finder terminate near the jump while retaining an enthalpy residual of magnitude 50; the original-target check rejects it. [NP-26]

## 4. Executed authored thermodynamic and process formulations

These examples use declared invented materials. They demonstrate how the blueprint's semantics and solver boundaries can operate together, not fidelity for named industrial fluids.

### 4.1 Coherent ideal binary VLE, calorics and inverse states

The binary fixture uses ideal mixing, ideal gas pressure dependence, constant species heat capacities equal in liquid and vapor, constant latent contributions, and a zero-liquid-volume approximation. No liquid density or realistic pump behavior is claimed.

For each component:

`h_i^L = Cp_i (T - Tref)`  
`h_i^V = h_i^L + L_i`  
`Delta s_i = L_i/Tb_i`  
`K_i(T,P) = (P0/P) exp[(Delta s_i - L_i/T)/R]`.

The model uses compatible phase entropy, including ideal mixing and vapor pressure terms. The complete phase-weighted H and S enter every inverse solve; no caloric term is added afterward. The fixture fixes `Cp=[80,120] J/mol/K`, `L=[30000,40000] J/mol`, `Tb=[340,380] K`, `Tref=298.15 K`, `P0=100000 Pa`, and overall `z=[0.5,0.5]`.

TP-to-PH and TP-to-PS roundtrips pass across seven selected liquid, two-phase and vapor states from 320 to 420 K at 100 kPa. These checks establish the internal consistency of this specific authored formulation and its original-target handling. [NP-15]

At 360 K, the vapor fraction is approximately **0.3674543931**. The analytical total response, including changing phase amounts/compositions, is:

**dH/dT = 2659.239736845 J/mol/K**.

Centered full re-solves give **2659.239743489 J/mol/K**. The frozen-allocation heat capacity is only **100 J/mol/K**. A second check confirms `dH/dT = T dS/dT` under the fixture's fixed-pressure equilibrium response. [NP-16, NP-17]

This is direct evidence for keeping SM-02 local phase derivatives separate from SM-03 condensed solved responses. It is not evidence that any untested external package already provides the correct reduced Jacobian or Hessian.

### 4.2 Valve, compressor, mixer, separator and heat-coupling examples

The synthetic valve starts at 380 K and 200 kPa, then resolves the same molar enthalpy at 100 kPa. The result is approximately **356.859355945 K**, with vapor fraction **0.1247883933**. The recorded enthalpy residual is approximately **1.12e-9 J/mol**. Component accounting and the original caloric convention are preserved. [NP-18]

The gas-compression example starts at 420 K and 100 kPa and targets 300 kPa. Its ideal PS reference is approximately **460.171119553 K**; applying efficiency 0.8 produces a separate actual PH outlet at **470.213899441 K**. The implied work is approximately **5021.389944 J/mol**. [NP-19]

The separator routes one checked internal equilibrium into physical product quantities and verifies component and enthalpy-flow reconstruction. It does not identify phase meaning with outlet position. [NP-20]

A 5 kW heat exchange couples the synthetic fluid to a mass-only cold material with 2 kg/s and Cp=2000 J/kg/K. The cold side moves from 300 K to **301.25 K**; the hot side reaches approximately **370 K**. Applying a consistent constant energy-reference shift to the nonreacting hot side leaves the energy balance unchanged. No molecular weight is needed for the cold material. [NP-22]

The mixer combines 1 mol/s at 340 K with 2 mol/s at 420 K under its energy rule. Its correct synthetic equilibrium outlet is approximately **364.979839 K**, not the arithmetic flow-weighted temperature **393.333333 K**. A subsequent mechanical split preserves the evaluated composition and energy account. [NP-29]

A simple nonthermodynamic recycle fixed point is included solely to distinguish local calculation from parent convergence. Its 40-unit recycle solution does not appear merely because the first local calculation returned 8. [NP-30]

### 4.3 Two-liquid equilibrium using the extracted NRTL function

An artificial symmetric NRTL binary with off-diagonal tau=3 and alpha=0.2 is solved under a declared two-liquid, no-vapor/no-solid scope. The host solves the equality of activities and uses the material balance for a 10 mol feed at z_A=0.4.

The selected phase compositions are approximately **x_A=0.01088787225** and **0.98911212775**, with phase amounts **6.0222604831 mol** and **3.9777395169 mol**. The largest log-activity mismatch is approximately **5.61e-15**. The split has lower modeled Gibbs energy than the tested homogeneous feed, and a 1,001-point common-tangent comparison finds no lower sampled candidate. [NP-21]

This is a real numerical solve of the authored NRTL model, not a pass for the complete `thermo.FlashVLN` package. Symmetry and the chosen two-phase branch are explicit assumptions. The grid check is **sampled stability evidence**, not a universal continuous/global stability certificate, not real-fluid data, and not a general VLL or precipitation algorithm.

### 4.4 Coupled chemical equilibrium and adiabatic energy

An authored balanced isomerization A <-> B has equal species Cp=100 J/mol/K, standard reaction enthalpy -20000 J/mol, standard entropy difference -40 J/mol/K, and a one-mole A feed at 350 K. The condensed temperature problem and an exposed two-unknown root formulation use the same declared chemical and thermal relationships.

Both reach approximately **T=466.893205960 K** and **extent=0.5844660298 mol**. The exposed solve starts from the separately specified `[440 K,0.45 mol]`, not from the condensed solution. Residuals of both formulations are checked against the original thermal and chemical equations. [NP-23]

A second bookkeeping calculation applies component-dependent enthalpy offsets and includes the necessary reaction-reference correction. Without that correction the apparent energy residual is approximately **17533.980894 J**; with the declared correction the balance closes within the fixture tolerance. This checks a consistent alternate energy formulation, not a change to chemical equilibrium standards. Strict-extent rejection remains a separate test. [NP-23, NP-24]

No Reaktoro, IDAES, kinetic reactor, mineral database or actual gas-solid system was exercised. These examples cannot be added together to claim a reactive-distillation or aqueous-absorption scenario.

## 5. Behavioral validation: publication, cancellation and replay

The new reference model enumerates every legal ordering of two candidate chains `capture -> qualify -> publish`, one edit, and one parent cancellation. There are **1,120 legal eight-event schedules**. At each publication the model is compared with an independently stated admission predicate; current nonstale results must belong to the current revision.

The correct reference model had no recorded violations over **8,960 event steps**. Additional focused controls cover replay, group validity, iterate lineage, shared-buffer aliasing and the distinction between publication cancellation and native cleanup. A separate truth table covers **128 combinations** of revision, run, parent, qualification, view, group and target-slot guards.

Eight deliberately unsafe variants are detected: ignoring revision; ignoring parent cancellation; ignoring target generation; reinstalling an old result on command replay; ignoring required qualification; ignoring iterate lineage; admitting a partial invalid group; and aliasing accepted material to native working memory.

This demonstrates that the chosen witnesses can distinguish these unsafe semantics. It does not demonstrate that a future lock, database, message layer or provider adapter implements them. The event space is bounded and sequential; there was no native process crash, concurrent database write or race detector.

Important preserved outcomes include: an edit can stale a previously legitimate result without erasing its history; a valid late A result cannot overwrite B; same-revision parent cancellation still blocks publication; replaying a committed A decision after B does not reinstall A; and a still-running native call cannot be freed or reused just because its publication permission was cancelled.

## 6. Findings, disposition and required refinements

The findings below distinguish blockers, strengthened acceptance witnesses and confirmed requirements. They are not an upstream bug list or a claim that the eventual implementation already complies.

### VF-01. Complete packages were unavailable to execute here

**Classification:** environment_blocker. **Importance:** blocks full-provider admission. **Owners:** P08, P03, P09.

**Observed:** No complete candidate thermodynamics package was installed; package-index/name-resolution and direct-wheel retrieval failed. Six optional native worker invocations were blocked before execution.

**Disposition:** Keep every native IP/QG claim open. Distinguish unavailable runtime from unsupported model. Use exact build/data manifests for the next actual execution.

**Closure:** Open; environmental, not a library defect.

**Evidence:** environment_preflight.json, native_probe_dispatch.json. **Requirement links:** FR-GOV-02, FR-GOV-03, FR-RES-08. **Integration rules:** IR-01, IR-18, IR-22, IR-24.

### VF-02. Zero RR residual does not establish unique phase allocation

**Classification:** acceptance_test_refinement. **Importance:** blocks unqualified solved-allocation derivative claims. **Owners:** P05, P08, P09.

**Observed:** With all active K values equal to one, every beta gives zero RR residual. Near this case the residual and its beta derivative are both about 1e-16 in the test.

**Disposition:** Assess allocation degeneracy/conditioning in its full request context. Do not invent a unique TP allocation or smooth sensitivity. A separate enthalpy/quality constraint can remove the degeneracy and must be handled by a qualified formulation, not rejected blindly.

**Closure:** Concrete witness added; provider-specific policy still requires testing.

**Evidence:** NP-12, NP-13, NP-26. **Requirement links:** FR-STA-02, FR-EQL-04, FR-PRP-06, FR-RES-02. **Integration rules:** IR-12, IR-14, IR-15, IR-20.

### VF-03. Raw numerical kernels do not enforce the whole material contract

**Classification:** confirmed_required_boundary. **Importance:** blocks raw-kernel-as-adapter substitution. **Owners:** P01, P04, P08.

**Observed:** The extracted NRTL function returns finite activity coefficients for xs=[0.6,0.6], equal to those for [0.5,0.5]; transposing directed taus also returns plausible but different values.

**Disposition:** Validate nonnegative normalized composition and formula/subject ordering before execution. Preserve explicit input intent; the full package may add validation elsewhere, which was not inspected or exercised here.

**Closure:** Host-boundary negative controls pass in authored harness; no upstream bug allegation.

**Evidence:** NP-09, NP-10. **Requirement links:** FR-MAT-07, FR-STA-02, FR-DAT-02. **Integration rules:** IR-03, IR-04, IR-19.

### VF-04. Solved enthalpy response differs materially from frozen allocation Cp

**Classification:** confirmed_required_boundary. **Importance:** blocks incorrect SM-03 derivatives. **Owners:** P05, P08, P09.

**Observed:** In the authored binary two-phase fixture, dH/dT=2659.2397368 J/mol/K versus frozen-allocation Cp=100. Full re-solves agree with the derived total response.

**Disposition:** Demand total solved sensitivities for condensed blocks; preserve local derivatives separately. Native implementations must reproduce the declared coordinates/response and conditioning.

**Closure:** Authored mathematical realization demonstrated; external derivative contract remains untested.

**Evidence:** NP-16, NP-17, NP-23. **Requirement links:** FR-PRP-05, FR-PRP-06, FR-FLW-07. **Integration rules:** IR-11, IR-12, IR-13.

### VF-05. Caloric completion is part of the equation, not postprocessing

**Classification:** confirmed_required_boundary. **Importance:** blocks calorically incomplete inverse packages. **Owners:** P02, P03, P05.

**Observed:** The partial-caloric counterexample returns 325 K but the complete enthalpy is 125 against a target of 50; the complete inverse instead returns 310 K. Temperature-dependent NRTL caloric algebra reproduces the documentation example.

**Disposition:** Use the full actual H/S function inside inverse solves and verify original targets. Preserve the documented derivative meaning of temperature-dependent parameters.

**Closure:** Witnesses confirmed; no real pseudocomponent package qualified.

**Evidence:** NP-08, NP-15, NP-25. **Requirement links:** FR-CFG-04, FR-EQL-02, FR-PRP-07. **Integration rules:** IR-03, IR-05, IR-06, IR-20.

### VF-06. Local and constituent successes are not coupled-scenario completion

**Classification:** confirmed_required_boundary. **Importance:** blocks premature process/state claims. **Owners:** P07, P09.

**Observed:** Shared allocation/routing and condensed/exposed ideal chemistry can be tested separately. Neither combination proves an actual reactive column, VLL flash, solids chemistry, or full nonequilibrium contactor.

**Disposition:** Keep final process residuals, coupled chemistry/phase checks, and parent convergence mandatory; do not combine separate primitive test passes into a broad scenario pass.

**Closure:** Coverage boundary retained; full process fixtures remain open.

**Evidence:** NP-20, NP-21, NP-23, NP-27, NP-30. **Requirement links:** FR-CHM-08, FR-FLW-08, FR-RUN-08, FR-RES-08. **Integration rules:** IR-08, IR-16, IR-20, IR-24.

### VF-07. Every publication guard has a concrete counterexample

**Classification:** reference_model_confirmation. **Importance:** blocks unsafe adoption implementations. **Owners:** P09, P10, P08.

**Observed:** The correct authored model had zero violations across 1120 legal eight-event schedules. Eight deliberately weakened guard/alias variants were detected by schedule or focused controls.

**Disposition:** Preserve dependency, live ancestor/run permission, target generation, qualification, iterate lineage, group coherence, and duplicate-decision semantics at the real commit boundary.

**Closure:** Bounded reference model passes; real concurrency/storage and native cleanup remain untested.

**Evidence:** publication_model_checks.json. **Requirement links:** FR-RES-05, FR-RUN-06, FR-LIF-01, FR-LIF-08. **Integration rules:** IR-18, IR-21, IR-22.

### VF-08. Executed source extracts are a separate evidence class

**Classification:** evidence_granularity_refinement. **Importance:** blocks inflated readiness or validation claims. **Owners:** P08, P09.

**Observed:** A numerical body, a host Brent solver, a documented model example, a full imported package, an adapter, and a complete flowsheet are different test subjects. Only the first group was numerically available.

**Disposition:** Record the execution subject, modifications/import context, actual input hash, solver/runtime, checks, and omitted layers. A same-model source example is regression evidence, not independent real-fluid validation.

**Closure:** Evidence record refined here; no change to predecessor normative scope.

**Evidence:** source_manifest.json, numerical_probes.json, native_probe_dispatch.json. **Requirement links:** FR-GOV-01, FR-GOV-02, FR-RES-07, FR-RES-08. **Integration rules:** IR-01, IR-24.

## 7. Disposition of the ten qualification gates

Gate judgments apply to a particular subject. An authored model meeting a mathematical test does not make an uninstantiated native profile pass the same gate.

| Gate | What this stage establishes | Native/profile disposition |
| --- | --- | --- |
| QG-01 Attributable profile | Exact authored fixtures and extracted-source identities are recorded. | Fourteen native candidate templates are not production instantiations. |
| QG-02 Representation and parameter semantics | Walkthroughs, permutation tests, explicit parameter/formula examples and invalid-input controls. | Actual native data/convention mappings remain to be verified. |
| QG-03 Physical/caloric composition | Selected authored caloric consistency, reference and reaction-accounting tests. | No real petroleum, electrolyte, transport or engineering-data completion established. |
| QG-04 Exact exposed operation | Two isolated source functions executed; six full-package dispatches blocked. | Not passed for complete installed providers/adapters. |
| QG-05 Solver mathematics | Local/total derivative and condensed/exposed formulation examples exercised. | Exact native derivatives, sparse structure, Hessians and branch coverage still open. |
| QG-06 Sessions/failure containment | Bounded logical cleanup/authority controls. | No native interleaving, crash containment or concurrent session qualification. |
| QG-07 Numerical conformance | Thirty bounded probe records with known fixtures and postconditions. | No blanket native or full-scenario conformance pass. |
| QG-08 Process/lifecycle behavior | Synthetic valve/compressor/separation/heat/mix/recycle fragments and reference publication model. | No complete production unit, flowsheet or storage implementation. |
| QG-09 Independent physical validation | No independent physical dataset was tested. | Remains open for every real profile. |
| QG-10 Specific operational approval | Conditional approval to proceed with semantic/core implementation design. | No production thermodynamic operating profile approved. |

### Evidence-preserving adoption rule

New evidence is appended at its real subject: extracted gamma function, host fixed-K allocation, authored complete caloric model, or reference publication model. All original AT/AW/P/PV witnesses retain their recorded status. The 34 scenario review records state both their contributing tests and missing full-scenario evidence. None receives an automatic full R/E/V promotion from a matching ID or a passed helper calculation.

## 8. Complete scenario walkthrough record

Each card below records a specific input context, authority/action route, adversarial condition, contributing executable evidence, and unresolved gate. The review used the supplied B4/B6/B7/B8 semantics. It did not inspect an external codebase or constitute a second independent reviewer.

### VR-01 / SC-01: Blending and splitting without reaction

**Profile:** P1. **Review:** authored semantic/action walkthrough.

**Case:** Two compatible feeds with declared component flows and a chosen pressure/heat rule; a separate composition-preserving splitter.

**Route:** Capture each account; perform material and enthalpy-flow aggregation; resolve the mixed state; route a coherent product set.

**Adversarial boundary:** An arithmetic temperature average or a partial component edit masquerading as complete replacement changes input intent.

**Contributing executed probes:** NP-29, NP-30

**Remaining acceptance gate:** Actual material data, full reference simulator integration, and all GUI/API edit paths remain untested.

**Requirement route:** FR-MAT-05, FR-MAT-06, FR-MAT-08, FR-FLW-02, FR-RES-03. **Actions:** AC-02, AC-14, AC-15, AC-36, AC-39. **Information:** IC-05, IC-07, IC-21, IC-25, IC-26, IC-27, IC-43, IC-50, IC-59, IC-65, IC-71. **Native profiles:** IP-01.

**Conclusion:** No blocking ownership/information contradiction identified within this described route. Full numerical scenario execution, independent representability signoff and independent physical validation are not established by this card.

### VR-02 / SC-02: Sensible heating and heat exchange

**Profile:** P1. **Review:** authored semantic/action walkthrough.

**Case:** A duty heater and a separately identified rating mode; complete calorics versus optional transport.

**Route:** Freeze the actual enthalpy function before PH resolution; attach only the properties required by the selected unit mode.

**Adversarial boundary:** Adding a missing caloric term after a partial PH solve gives the wrong target; missing rating viscosity cannot be hidden.

**Contributing executed probes:** NP-15, NP-22, NP-25

**Remaining acceptance gate:** No real-fluid heat duty or geometry/rating model validated.

**Requirement route:** FR-CFG-04, FR-EQL-02, FR-PRP-02, FR-PRP-03, FR-FLW-01, FR-FLW-04. **Actions:** AC-08, AC-20, AC-22, AC-23, AC-41. **Information:** IC-14, IC-20, IC-21, IC-24, IC-29, IC-34, IC-35, IC-36, IC-38, IC-41, IC-48, IC-50, IC-56, IC-71. **Native profiles:** IP-01, IP-04, IP-09.

**Conclusion:** No blocking ownership/information contradiction identified within this described route. Full numerical scenario execution, independent representability signoff and independent physical validation are not established by this card.

### VR-03 / SC-03: Liquid pumping and pressure-loss calculations

**Profile:** P1. **Review:** authored semantic/action walkthrough.

**Case:** A liquid account with pressure specification and a unit-owned pump or pipe relationship; density and viscosity demands explicit.

**Route:** Bind a liquid-capable package; evaluate the supplied phase; let equipment supply pressure/work/hydraulic constraints.

**Adversarial boundary:** Our zero-liquid-volume synthetic VLE fixture cannot be promoted to a physical pumping or pressure-loss model.

**Contributing executed probes:** No new numerical probe for the complete scenario; inherited reference records remain separately classified.

**Remaining acceptance gate:** A real density/viscosity package and pump/pipe law are mandatory outstanding fixtures.

**Requirement route:** FR-PRP-01, FR-PRP-03, FR-CFG-03, FR-FLW-01, FR-EQL-06. **Actions:** AC-09, AC-20, AC-22, AC-23. **Information:** IC-24, IC-28, IC-29, IC-34, IC-37, IC-38, IC-41, IC-42, IC-48, IC-50, IC-53, IC-56, IC-57. **Native profiles:** IP-01, IP-04.

**Conclusion:** No blocking ownership/information contradiction identified within this described route. Full numerical scenario execution, independent representability signoff and independent physical validation are not established by this card.

### VR-04 / SC-04: Gas compression, expansion and intercooling

**Profile:** P1. **Review:** authored semantic/action walkthrough.

**Case:** Gas inlet, outlet pressure, efficiency law, PS reference and distinct actual PH outlet.

**Route:** Compute the isentropic reference; apply the unit efficiency; resolve actual outlet energy; qualify both local and unit balances.

**Adversarial boundary:** Publishing the ideal reference as the actual machine outlet or differentiating a frozen inner phase misses the intended problem.

**Contributing executed probes:** NP-16, NP-19

**Remaining acceptance gate:** Synthetic ideal material only; real compressor envelope, phase policy and provider behavior remain open.

**Requirement route:** FR-EQL-03, FR-EQL-02, FR-CFG-04, FR-STA-04, FR-PRP-05, FR-FLW-01. **Actions:** AC-08, AC-13, AC-20, AC-23, AC-25. **Information:** IC-04, IC-14, IC-20, IC-21, IC-24, IC-26, IC-28, IC-34, IC-35, IC-36, IC-38, IC-39, IC-41, IC-48, IC-50, IC-57. **Native profiles:** IP-01.

**Conclusion:** No blocking ownership/information contradiction identified within this described route. Full numerical scenario execution, independent representability signoff and independent physical validation are not established by this card.

### VR-05 / SC-05: Cooling and vapor–liquid separation

**Profile:** P1. **Review:** authored semantic/action walkthrough.

**Case:** A cooled feed with a shared permitted vapor/liquid allocation and explicit product quantities.

**Route:** Resolve one material equilibrium; check component/energy totals; then apply product routing.

**Adversarial boundary:** A numerical phase entry or report aggregate cannot create a product, and a converged local allocation cannot claim global stability.

**Contributing executed probes:** NP-03, NP-11, NP-20

**Remaining acceptance gate:** No actual-library flash dispatcher or real-mixture stability validation.

**Requirement route:** FR-EQL-01, FR-EQL-06, FR-STA-05, FR-STA-06, FR-FLW-03, FR-RES-03. **Actions:** AC-16, AC-23, AC-36, AC-40. **Information:** IC-05, IC-21, IC-29, IC-30, IC-31, IC-34, IC-35, IC-36, IC-37, IC-41, IC-42, IC-50, IC-56, IC-59, IC-71. **Native profiles:** IP-01.

**Conclusion:** No blocking ownership/information contradiction identified within this described route. Full numerical scenario execution, independent representability signoff and independent physical validation are not established by this card.

### VR-06 / SC-06: Pressure reduction with flashing

**Profile:** P1. **Review:** authored semantic/action walkthrough.

**Case:** Isenthalpic pressure-reducing unit with fixed original H, pressure, amount and nonreacting composition.

**Route:** Capture original target; initialize and solve complete PH; recheck target; publish only the declared outlet completion kind.

**Adversarial boundary:** A root finder reporting success on a jump, a target rewritten to its returned value, or obsolete completion must be rejected.

**Contributing executed probes:** NP-18, NP-25, NP-26

**Remaining acceptance gate:** Synthetic valve executed; production provider and downstream stream publication remain untested.

**Requirement route:** FR-EQL-02, FR-STA-03, FR-RES-02, FR-RES-04, FR-RES-05, FR-RUN-01. **Actions:** AC-21, AC-23, AC-31, AC-36, AC-37, AC-49. **Information:** IC-21, IC-24, IC-35, IC-36, IC-37, IC-41, IC-52, IC-53, IC-57, IC-59, IC-60, IC-64, IC-66, IC-67, IC-70. **Native profiles:** IP-01.

**Conclusion:** No blocking ownership/information contradiction identified within this described route. Full numerical scenario execution, independent representability signoff and independent physical validation are not established by this card.

### VR-07 / SC-07: Conventional equilibrium-stage distillation

**Profile:** P1. **Review:** authored semantic/action walkthrough.

**Case:** Two or more named internal stages with shared method definitions and distinct local variables and completion scopes.

**Route:** Use phase-only or equation contributions; retain initialization interventions; collect final stage/interstage residuals before parent acceptance.

**Adversarial boundary:** A stage at an old iterate may not join a newer inlet merely because the package revision matches.

**Contributing executed probes:** NP-27

**Remaining acceptance gate:** No rigorous multistage column executed; stage topology and all MESH equations remain a real-fixture gate.

**Requirement route:** FR-STA-01, FR-STA-04, FR-PRP-01, FR-FLW-07, FR-RUN-02, FR-RUN-08, FR-RES-04. **Actions:** AC-13, AC-22, AC-31, AC-37, AC-43, AC-44. **Information:** IC-25, IC-26, IC-28, IC-34, IC-36, IC-37, IC-40, IC-48, IC-50, IC-52, IC-54, IC-57, IC-60, IC-66, IC-67. **Native profiles:** IP-01, IP-13.

**Conclusion:** No blocking ownership/information contradiction identified within this described route. Full numerical scenario execution, independent representability signoff and independent physical validation are not established by this card.

### VR-08 / SC-08: Nonideal-liquid separation and azeotropic behavior

**Profile:** P1. **Review:** authored semantic/action walkthrough.

**Case:** A liquid activity model, directed temperature-dependent interactions, compatible vapor and caloric definitions.

**Route:** Resolve ordered parameters and their formulas; evaluate activities/caloric terms; only then qualify an assembled VLE/separation profile.

**Adversarial boundary:** Transposed parameters can produce plausible gamma values; a matching kernel example does not establish azeotrope or distillation behavior.

**Contributing executed probes:** NP-01, NP-04, NP-07, NP-08, NP-10

**Remaining acceptance gate:** Source numerical example reproduced; no complete ethanol-water VLE/azeotrope package or independent mixture data comparison.

**Requirement route:** FR-DAT-02, FR-DAT-03, FR-CFG-02, FR-CFG-04, FR-EQL-07, FR-RUN-05. **Actions:** AC-04, AC-05, AC-08, AC-23, AC-34. **Information:** IC-12, IC-13, IC-14, IC-17, IC-18, IC-20, IC-21, IC-23, IC-24, IC-30, IC-35, IC-42, IC-54, IC-57, IC-65, IC-72. **Native profiles:** IP-02.

**Conclusion:** No blocking ownership/information contradiction identified within this described route. Full numerical scenario execution, independent representability signoff and independent physical validation are not established by this card.

### VR-09 / SC-09: Physical absorption, humidification and gas dissolution

**Profile:** P1. **Review:** authored semantic/action walkthrough.

**Case:** Gas and solvent with allowed physical partitioning, explicit dilute-solute standard, wet/dry reporting and no declared reaction.

**Route:** Evaluate each phase and its partition relationships; apply unit transfer/energy balances; derive reports without changing physical amounts.

**Adversarial boundary:** Missing Henry-law convention or gas-solvent data cannot be repaired by undeclared ideal partitioning.

**Contributing executed probes:** No new numerical probe for the complete scenario; inherited reference records remain separately classified.

**Remaining acceptance gate:** Actual Henry/saturation/concentration conventions and process fixture still required.

**Requirement route:** FR-CFG-02, FR-CFG-07, FR-PRP-01, FR-PRP-07, FR-STA-08, FR-FLW-01. **Actions:** AC-07, AC-08, AC-17, AC-20, AC-22. **Information:** IC-04, IC-06, IC-18, IC-21, IC-22, IC-23, IC-26, IC-28, IC-29, IC-31, IC-34, IC-37, IC-42, IC-48, IC-50. **Native profiles:** IP-02, IP-13.

**Conclusion:** No blocking ownership/information contradiction identified within this described route. Full numerical scenario execution, independent representability signoff and independent physical validation are not established by this card.

### VR-10 / SC-10: Liquid–liquid extraction and decanting

**Profile:** P2. **Review:** authored semantic/action walkthrough.

**Case:** Two liquid instances with component amounts and a named correspondence/routing policy.

**Route:** Solve a shared two-liquid problem; retain physical versus numerical entries; reconcile amounts and both activity conditions; route afterward.

**Adversarial boundary:** Array reversal cannot swap physical meaning; symmetry and equal density do not supply unique continuous phase correspondence.

**Contributing executed probes:** NP-21, NP-04

**Remaining acceptance gate:** Artificial NRTL mixture; sampled common-tangent evidence is not a general global or empirical validation.

**Requirement route:** FR-EQL-08, FR-STA-05, FR-STA-06, FR-PRP-03, FR-FLW-03. **Actions:** AC-16, AC-22, AC-23, AC-40. **Information:** IC-22, IC-29, IC-30, IC-31, IC-34, IC-35, IC-38, IC-50, IC-53, IC-71. **Native profiles:** IP-03.

**Conclusion:** No blocking ownership/information contradiction identified within this described route. Full numerical scenario execution, independent representability signoff and independent physical validation are not established by this card.

### VR-11 / SC-11: Vapor–liquid–liquid separation

**Profile:** P2. **Review:** authored semantic/action walkthrough.

**Case:** One vapor and two candidate liquids in a single equilibrium description.

**Route:** Declare all permitted domains, require compatible shared allocation and energy, and admit an actual three-phase numerical surface.

**Adversarial boundary:** Two independent binary flashes or a two-phase-only adapter cannot satisfy simultaneous VLL by omission.

**Contributing executed probes:** No new numerical probe for the complete scenario; inherited reference records remain separately classified.

**Remaining acceptance gate:** No three-phase native or authored thermodynamic calculation executed; P2 delivery remains open.

**Requirement route:** FR-CFG-07, FR-EQL-01, FR-EQL-08, FR-EQL-06, FR-RES-03. **Actions:** AC-07, AC-23, AC-36. **Information:** IC-05, IC-06, IC-21, IC-22, IC-29, IC-34, IC-35, IC-36, IC-37, IC-41, IC-42, IC-53, IC-56, IC-59, IC-71. **Native profiles:** IP-03.

**Conclusion:** No blocking ownership/information contradiction identified within this described route. Full numerical scenario execution, independent representability signoff and independent physical validation are not established by this card.

### VR-12 / SC-12: Water and steam through saturation

**Profile:** P1. **Review:** authored semantic/action walkthrough.

**Case:** Pure water with subcooled, saturated and superheated requests, plus explicit phase-fraction basis.

**Route:** Distinguish endpoint property query, dependent saturated TP inputs and fully specified allocation using another valid constraint.

**Adversarial boundary:** RR degeneracy alone must not condemn a PH problem whose energy equation determines quality; naive scalar TP bracketing can fail.

**Contributing executed probes:** NP-12, NP-26

**Remaining acceptance gate:** No water backend imported; full-package water and quality workers blocked.

**Requirement route:** FR-EQL-04, FR-EQL-02, FR-EQL-03, FR-STA-02, FR-STA-08. **Actions:** AC-17, AC-21, AC-23. **Information:** IC-04, IC-21, IC-26, IC-29, IC-31, IC-35, IC-36, IC-38, IC-41, IC-50. **Native profiles:** IP-04.

**Conclusion:** No blocking ownership/information contradiction identified within this described route. Full numerical scenario execution, independent representability signoff and independent physical validation are not established by this card.

### VR-13 / SC-13: Pure-fluid refrigeration loop

**Profile:** P1. **Review:** authored semantic/action walkthrough.

**Case:** Closed pure-fluid refrigeration loop, with PS reference, actual compression energy, condenser, throttle and evaporator.

**Route:** Reuse one coherent fluid convention through all unit states; verify cycle heat/work and loop completion separately.

**Adversarial boundary:** One successful compressor or PH call is not an executed refrigeration loop.

**Contributing executed probes:** NP-19, NP-30

**Remaining acceptance gate:** No actual refrigerant cycle executed; component probes are only contributing evidence.

**Requirement route:** FR-MAT-01, FR-EQL-04, FR-EQL-02, FR-EQL-03, FR-RUN-08, FR-LIF-07. **Actions:** AC-01, AC-23, AC-44, AC-53. **Information:** IC-01, IC-02, IC-21, IC-29, IC-35, IC-36, IC-38, IC-41, IC-50, IC-52, IC-54, IC-60, IC-61, IC-68, IC-69. **Native profiles:** IP-05.

**Conclusion:** No blocking ownership/information contradiction identified within this described route. Full numerical scenario execution, independent representability signoff and independent physical validation are not established by this card.

### VR-14 / SC-14: Mixed-refrigerant phase change

**Profile:** P2. **Review:** authored semantic/action walkthrough.

**Case:** A true refrigerant mixture, or explicitly distinct pseudo-pure approximation, with fraction and endpoint conventions.

**Route:** Keep vapor/liquid compositions distinct; qualify mixture inverse routes and temperature-glide reference evidence.

**Adversarial boundary:** Pure-fluid Q semantics or a pseudo-pure surrogate cannot silently replace a specified mixture.

**Contributing executed probes:** NP-15

**Remaining acceptance gate:** Synthetic binary only; actual blend data, glide and full cycle remain untested.

**Requirement route:** FR-MAT-02, FR-STA-08, FR-EQL-04, FR-EQL-02, FR-CFG-02. **Actions:** AC-02, AC-08, AC-17, AC-23. **Information:** IC-03, IC-04, IC-06, IC-18, IC-21, IC-23, IC-25, IC-26, IC-29, IC-31, IC-35, IC-36, IC-38, IC-41. **Native profiles:** IP-06.

**Conclusion:** No blocking ownership/information contradiction identified within this described route. Full numerical scenario execution, independent representability signoff and independent physical validation are not established by this card.

### VR-15 / SC-15: Assay-derived petroleum pseudocomponents

**Profile:** P2. **Review:** authored semantic/action walkthrough.

**Case:** A retained source assay, explicit cuts, generated constituent identities, selected correlations and a data/package revision proposal.

**Route:** Characterize in a provisional context; validate identities and full calorics; commit a consistent revision; save raw and derived evidence.

**Adversarial boundary:** PVT-only pseudocomponents cannot qualify heating; generated cuts alone are insufficient archive reconstruction.

**Contributing executed probes:** No new numerical probe for the complete scenario; inherited reference records remain separately classified.

**Remaining acceptance gate:** No accessible real assay characterized; native DWSIM path and alternative custom-data paths remain blocked/uninstantiated.

**Requirement route:** FR-DAT-05, FR-DAT-01, FR-CFG-04, FR-MAT-01, FR-LIF-04, FR-LIF-07. **Actions:** AC-01, AC-04, AC-06, AC-08, AC-50, AC-53. **Information:** IC-01, IC-02, IC-03, IC-10, IC-11, IC-13, IC-14, IC-16, IC-17, IC-20, IC-21, IC-24, IC-61, IC-62, IC-68, IC-69. **Native profiles:** IP-07.

**Conclusion:** No blocking ownership/information contradiction identified within this described route. Full numerical scenario execution, independent representability signoff and independent physical validation are not established by this card.

### VR-16 / SC-16: Black-oil or other reduced petroleum representation

**Profile:** P2. **Review:** authored semantic/action walkthrough.

**Case:** A reduced oil/gas/water description with empirical standard conditions, supported correlations and declared absent molecular detail.

**Route:** Preserve bulk representation meaning; use only available empirical state operations; translate across a separate boundary when justified.

**Adversarial boundary:** Inventing a unique detailed composition to call a molecular flash violates material meaning.

**Contributing executed probes:** No new numerical probe for the complete scenario; inherited reference records remain separately classified.

**Remaining acceptance gate:** No black-oil runtime or reference case executed.

**Requirement route:** FR-MAT-02, FR-STA-08, FR-CFG-03, FR-DAT-07, FR-FLW-06. **Actions:** AC-02, AC-04, AC-09, AC-17, AC-42. **Information:** IC-03, IC-04, IC-06, IC-07, IC-11, IC-13, IC-15, IC-24, IC-25, IC-26, IC-31, IC-34, IC-47, IC-51, IC-53, IC-57. **Native profiles:** IP-08.

**Conclusion:** No blocking ownership/information contradiction identified within this described route. Full numerical scenario execution, independent representability signoff and independent physical validation are not established by this card.

### VR-17 / SC-17: Specified-conversion reaction

**Profile:** P1. **Review:** authored semantic/action walkthrough.

**Case:** Balanced synthetic A to B with strict imposed extent and an explicit caloric reference treatment.

**Route:** Check availability before transformation; retain reaction normalization; reconcile reaction/formation energy exactly once.

**Adversarial boundary:** Clipping extent 1.2 to 1.0 with one mole A available answers a different problem.

**Contributing executed probes:** NP-24, NP-23

**Remaining acceptance gate:** Strict synthetic amount contract exercised; an actual reaction parameter set and unit package remain outstanding.

**Requirement route:** FR-CHM-01, FR-CHM-02, FR-CHM-05, FR-MAT-08, FR-EQL-02, FR-RES-03. **Actions:** AC-02, AC-08, AC-10, AC-11, AC-23, AC-36. **Information:** IC-05, IC-07, IC-21, IC-35, IC-36, IC-41, IC-43, IC-44, IC-59, IC-71. **Native profiles:** IP-10.

**Conclusion:** No blocking ownership/information contradiction identified within this described route. Full numerical scenario execution, independent representability signoff and independent physical validation are not established by this card.

### VR-18 / SC-18: Chemical-equilibrium reaction

**Profile:** P1. **Review:** authored semantic/action walkthrough.

**Case:** A coherent chemical system with conserved quantities, species participation and thermal constraints.

**Route:** Use one combined problem; solve species/thermal unknowns through a condensed or exposed formulation; check final residuals together.

**Adversarial boundary:** Fixed-species phase equilibrium is not chemical equilibrium; small inner errors or a later reflash cannot be ignored.

**Contributing executed probes:** NP-23

**Remaining acceptance gate:** Authored ideal two-species reaction only; no Reaktoro/IDAES chemistry runtime or real database validation.

**Requirement route:** FR-CHM-03, FR-CHM-05, FR-CHM-07, FR-MAT-08, FR-CFG-07. **Actions:** AC-02, AC-07, AC-08, AC-10, AC-24. **Information:** IC-05, IC-06, IC-07, IC-21, IC-22, IC-29, IC-35, IC-42, IC-43, IC-44, IC-45, IC-46, IC-71. **Native profiles:** IP-11, IP-12.

**Conclusion:** No blocking ownership/information contradiction identified within this described route. Full numerical scenario execution, independent representability signoff and independent physical validation are not established by this card.

### VR-19 / SC-19: Kinetically controlled reaction

**Profile:** P2. **Review:** authored semantic/action walkthrough.

**Case:** A finite-rate law with explicit volume/catalyst/site basis, frozen/equilibrated subsets and unit-owned residence-time context.

**Route:** Provide local activities/concentrations and calorics without forcing forbidden equilibrium; distinguish rate from integrated extent.

**Adversarial boundary:** Changing rate basis or replacing kinetics with equilibrium changes the declared physical model.

**Contributing executed probes:** No new numerical probe for the complete scenario; inherited reference records remain separately classified.

**Remaining acceptance gate:** No kinetic reactor integration or native kinetic/equilibrium coupling executed.

**Requirement route:** FR-CHM-01, FR-CHM-04, FR-PRP-01, FR-PRP-05, FR-STA-01. **Actions:** AC-10, AC-13, AC-22, AC-25. **Information:** IC-04, IC-25, IC-28, IC-34, IC-37, IC-38, IC-39, IC-43, IC-44, IC-45, IC-48. **Native profiles:** IP-10.

**Conclusion:** No blocking ownership/information contradiction identified within this described route. Full numerical scenario execution, independent representability signoff and independent physical validation are not established by this card.

### VR-20 / SC-20: Reactive separation

**Profile:** P2. **Review:** authored semantic/action walkthrough.

**Case:** Reactive multistage or interphase separation with simultaneous chemistry, phase and thermal requirements.

**Route:** Assemble one final coupled acceptance problem, including any declared sequential approximation and its reconciliation.

**Adversarial boundary:** Adding isolated successes from an LLE test and a chemical-equilibrium test cannot certify a reactive separation.

**Contributing executed probes:** NP-21, NP-23

**Remaining acceptance gate:** Only separate contributing primitives exercised; complete coupled reactive separation remains open.

**Requirement route:** FR-CHM-08, FR-CHM-05, FR-EQL-08, FR-FLW-07, FR-RUN-08. **Actions:** AC-08, AC-23, AC-37, AC-43, AC-44. **Information:** IC-21, IC-22, IC-29, IC-34, IC-35, IC-36, IC-40, IC-41, IC-43, IC-44, IC-45, IC-50, IC-52, IC-53, IC-54, IC-59, IC-60, IC-71. **Native profiles:** IP-02, IP-03, IP-12, IP-13.

**Conclusion:** No blocking ownership/information contradiction identified within this described route. Full numerical scenario execution, independent representability signoff and independent physical validation are not established by this card.

### VR-21 / SC-21: Electrolyte mixing and neutralization

**Profile:** P2. **Review:** authored semantic/action walkthrough.

**Case:** One aqueous material account with apparent inputs, true ionic species, charge convention and reporting map.

**Route:** Map conserved quantities without duplicate inventory; solve allowed chemistry; retain pH/activity and energy conventions.

**Adversarial boundary:** An apparent salt and its corresponding ions cannot both be summed as additional physical mass.

**Contributing executed probes:** No new numerical probe for the complete scenario; inherited reference records remain separately classified.

**Remaining acceptance gate:** Inherited synthetic bookkeeping only; no real ionic system or neutralization energy evaluated.

**Requirement route:** FR-CHM-06, FR-CHM-03, FR-CHM-05, FR-MAT-08, FR-FLW-06. **Actions:** AC-02, AC-08, AC-12, AC-24, AC-42. **Information:** IC-05, IC-07, IC-21, IC-25, IC-35, IC-43, IC-44, IC-45, IC-46, IC-47, IC-51, IC-71. **Native profiles:** IP-11, IP-12.

**Conclusion:** No blocking ownership/information contradiction identified within this described route. Full numerical scenario execution, independent representability signoff and independent physical validation are not established by this card.

### VR-22 / SC-22: Reactive gas absorption into aqueous liquid

**Profile:** P2. **Review:** authored semantic/action walkthrough.

**Case:** Gas/aqueous contacting with reaction, phase transfer, required calorics and any explicitly authorized reservoir.

**Route:** Record every signed material/energy exchange once; combine chemistry and contactor constraints; distinguish equilibrium versus rate-based mode.

**Adversarial boundary:** A pH/fugacity constraint cannot silently add titrant/gas; a speciation-only answer is not an energy-balanced absorber.

**Contributing executed probes:** No new numerical probe for the complete scenario; inherited reference records remain separately classified.

**Remaining acceptance gate:** Actual chemistry/transport/caloric compatibility and full absorber fixture absent.

**Requirement route:** FR-CHM-07, FR-CHM-08, FR-CHM-05, FR-CFG-04, FR-FLW-08. **Actions:** AC-08, AC-10, AC-37, AC-43. **Information:** IC-14, IC-20, IC-21, IC-24, IC-28, IC-29, IC-35, IC-37, IC-41, IC-43, IC-44, IC-45, IC-46, IC-48, IC-50, IC-59, IC-60, IC-71. **Native profiles:** IP-11, IP-12, IP-13.

**Conclusion:** No blocking ownership/information contradiction identified within this described route. Full numerical scenario execution, independent representability signoff and independent physical validation are not established by this card.

### VR-23 / SC-23: Inert solids carried with fluid

**Profile:** P2. **Review:** authored semantic/action walkthrough.

**Case:** Disjoint fluid and inert-solid portions, with carried quantities, thermal data and explicit carryover rules.

**Route:** Keep equilibrium-inactive solids in material balances; use their supported mass calorics; route mechanically at the unit boundary.

**Adversarial boundary:** A carried inert solid must not disappear during flash; an effective viscosity requires its own closure.

**Contributing executed probes:** NP-22

**Remaining acceptance gate:** Mass-only thermal contribution exercised; a real slurry package and solids-routing fixture remain untested.

**Requirement route:** FR-MAT-02, FR-MAT-08, FR-CFG-07, FR-PRP-04, FR-FLW-03. **Actions:** AC-02, AC-07, AC-22, AC-40. **Information:** IC-03, IC-05, IC-06, IC-07, IC-18, IC-22, IC-25, IC-29, IC-30, IC-31, IC-38, IC-42, IC-43, IC-50, IC-71. **Native profiles:** IP-09.

**Conclusion:** No blocking ownership/information contradiction identified within this described route. Full numerical scenario execution, independent representability signoff and independent physical validation are not established by this card.

### VR-24 / SC-24: Crystallization and precipitation

**Profile:** P2. **Review:** authored semantic/action walkthrough.

**Case:** Solution with explicit permissible crystal/mineral identities and thermal conditions.

**Route:** Qualify common solution/solid data and phase-onset meaning; check final conservation, chemical/phase conditions and declared stability scope.

**Adversarial boundary:** A saturation index alone is not crystal yield or size; zero-amount candidates do not become products.

**Contributing executed probes:** No new numerical probe for the complete scenario; inherited reference records remain separately classified.

**Remaining acceptance gate:** No crystallization/precipitation engine or kinetic/size model exercised.

**Requirement route:** FR-EQL-08, FR-EQL-06, FR-CHM-08, FR-CHM-05, FR-STA-06. **Actions:** AC-08, AC-16, AC-23, AC-37. **Information:** IC-21, IC-22, IC-29, IC-30, IC-34, IC-35, IC-41, IC-42, IC-43, IC-44, IC-45, IC-53, IC-56, IC-59, IC-60, IC-71. **Native profiles:** IP-11, IP-12.

**Conclusion:** No blocking ownership/information contradiction identified within this described route. Full numerical scenario execution, independent representability signoff and independent physical validation are not established by this card.

### VR-25 / SC-25: Gas–solid chemical transformation

**Profile:** P2. **Review:** authored semantic/action walkthrough.

**Case:** Gas plus reactive solid and product forms with coherent thermochemistry and selected reaction/transport assumptions.

**Route:** Keep solid identities and participation explicit; solve the selected equilibrium or rate problem and a complete energy account.

**Adversarial boundary:** A gas-phase isomerization fixture cannot validate heterogeneous solid behavior.

**Contributing executed probes:** NP-23

**Remaining acceptance gate:** Only abstract coupled energy reasoning; actual gas-solid database, phases and unit closure remain open.

**Requirement route:** FR-CHM-01, FR-CHM-04, FR-CHM-08, FR-CHM-05, FR-CFG-07. **Actions:** AC-07, AC-08, AC-10, AC-37. **Information:** IC-06, IC-21, IC-22, IC-29, IC-37, IC-41, IC-42, IC-43, IC-44, IC-45, IC-59, IC-60, IC-71. **Native profiles:** IP-10, IP-11, IP-12.

**Conclusion:** No blocking ownership/information contradiction identified within this described route. Full numerical scenario execution, independent representability signoff and independent physical validation are not established by this card.

### VR-26 / SC-26: Rate-based nonequilibrium contacting

**Profile:** P2. **Review:** authored semantic/action walkthrough.

**Case:** Distinct bulk-phase and interface states, allowed temperature differences, flux conventions and mandatory transport properties.

**Route:** Evaluate supplied phases without bulk re-equilibration; let the contactor own area/transfer relations; check coupled flux balances.

**Adversarial boundary:** An efficiency-corrected equilibrium-stage result cannot substitute for the full bulk/interface formulation.

**Contributing executed probes:** NP-27

**Remaining acceptance gate:** Property-authority scaffold exercised; no full transfer or multi-component diffusion formulation executed.

**Requirement route:** FR-FLW-08, FR-STA-03, FR-PRP-01, FR-PRP-03, FR-RUN-08. **Actions:** AC-21, AC-22, AC-43, AC-44. **Information:** IC-28, IC-29, IC-34, IC-35, IC-37, IC-38, IC-48, IC-50, IC-52, IC-54, IC-60. **Native profiles:** IP-13.

**Conclusion:** No blocking ownership/information contradiction identified within this described route. Full numerical scenario execution, independent representability signoff and independent physical validation are not established by this card.

### VR-27 / SC-27: Adsorption and membrane state extensions

**Profile:** P3. **Review:** authored semantic/action walkthrough.

**Case:** Separate bulk and site/surface accounts or membrane sides, loading denominator and explicitly supplied transfer.

**Route:** Convert loading under its stated basis, form paired amount changes, preserve distinct account identity and limited numerical capability.

**Adversarial boundary:** An excess-loading convention cannot be silently treated as total stored amount; a surface is not an ordinary liquid.

**Contributing executed probes:** No new numerical probe for the complete scenario; inherited reference records remain separately classified.

**Remaining acceptance gate:** Conceptual action route and inherited arithmetic reviewed; independent R review and numerical extension deferred.

**Requirement route:** FR-EXT-01, FR-MAT-03, FR-MAT-08, FR-STA-03, FR-PRP-07. **Actions:** AC-02, AC-03, AC-08, AC-13, AC-21. **Information:** IC-05, IC-06, IC-07, IC-09, IC-18, IC-21, IC-25, IC-26, IC-28, IC-33, IC-35, IC-37, IC-43, IC-71. **Native profiles:** IP-14.

**Conclusion:** No blocking ownership/information contradiction identified within this described route. Full numerical scenario execution, independent representability signoff and independent physical validation are not established by this card.

### VR-28 / SC-28: Heat exchange between different property packages

**Profile:** P1. **Review:** authored semantic/action walkthrough.

**Case:** Two material regions with separate package conventions and no cross-wall material transfer.

**Route:** Evaluate side energies, reconcile common heat duty, qualify and publish both sides as one intended group.

**Adversarial boundary:** Independent absolute energy zeroes are allowed; mismatched group members or hidden component conversion are not.

**Contributing executed probes:** NP-22

**Remaining acceptance gate:** Synthetic heat-coupling numerical fixture only; real independent packages and integrated lifecycle still untested.

**Requirement route:** FR-FLW-04, FR-CFG-06, FR-PRP-02, FR-PRP-07, FR-RES-03. **Actions:** AC-08, AC-19, AC-22, AC-36, AC-41. **Information:** IC-05, IC-18, IC-21, IC-34, IC-38, IC-41, IC-48, IC-49, IC-50, IC-56, IC-59, IC-71. **Native profiles:** IP-01, IP-04, IP-05.

**Conclusion:** No blocking ownership/information contradiction identified within this described route. Full numerical scenario execution, independent representability signoff and independent physical validation are not established by this card.

### VR-29 / SC-29: Material transfer across a property-package boundary

**Profile:** P2. **Review:** authored semantic/action walkthrough.

**Case:** Source and target package bindings, identity map, known reference transform and chosen preserved quantities.

**Route:** Apply mapping/reference conversion, resolve target under admissible constraints, report residual model disagreement separately from actual heat.

**Adversarial boundary:** Holding incompatible corrected H and T simultaneously by hidden duty must fail.

**Contributing executed probes:** No new numerical probe for the complete scenario; inherited reference records remain separately classified.

**Remaining acceptance gate:** Inherited reference algebra rerun; no cross-native-package material boundary executed.

**Requirement route:** FR-FLW-05, FR-PRP-07, FR-STA-03, FR-RES-07. **Actions:** AC-08, AC-21, AC-37, AC-42. **Information:** IC-10, IC-14, IC-18, IC-21, IC-35, IC-37, IC-49, IC-51, IC-54, IC-62. **Native profiles:** IP-01, IP-02, IP-04, IP-07, IP-08, IP-11.

**Conclusion:** No blocking ownership/information contradiction identified within this described route. Full numerical scenario execution, independent representability signoff and independent physical validation are not established by this card.

### VR-30 / SC-30: Translation between material representations

**Profile:** P2. **Review:** authored semantic/action walkthrough.

**Case:** Explicit directed representation map with preserved quantities, loss and any additional inverse assumptions.

**Route:** Map one account into a non-additive view or target boundary; record what is lost; reject unsupported unique reconstruction.

**Adversarial boundary:** A lump or apparent view does not contain sufficient information for an automatic unique reverse map.

**Contributing executed probes:** No new numerical probe for the complete scenario; inherited reference records remain separately classified.

**Remaining acceptance gate:** Directed semantic walkthrough and inherited algebra only; profile-specific chemical/assay mappings not exercised.

**Requirement route:** FR-FLW-06, FR-MAT-07, FR-MAT-08, FR-CHM-06, FR-LIF-06. **Actions:** AC-02, AC-12, AC-42, AC-52. **Information:** IC-04, IC-05, IC-07, IC-25, IC-43, IC-47, IC-51, IC-58, IC-64, IC-65, IC-69. **Native profiles:** IP-07, IP-08, IP-11.

**Conclusion:** No blocking ownership/information contradiction identified within this described route. Full numerical scenario execution, independent representability signoff and independent physical validation are not established by this card.

### VR-31 / SC-31: Mass-based empirical and nonconventional materials

**Profile:** P2. **Review:** authored semantic/action walkthrough.

**Case:** Mass-based empirical constituent with actual mass flow, a complete stated h(T) and no molecular-weight assertion.

**Route:** Run the supported mass-energy operation; reject molecular operations needing unavailable data without discarding thermal usefulness.

**Adversarial boundary:** No invented molecular weight or fake critical point is admitted to satisfy a fluid API.

**Contributing executed probes:** NP-22

**Remaining acceptance gate:** Thermal synthetic fixture executed; no field material correlation or empirical transport validation.

**Requirement route:** FR-MAT-02, FR-MAT-03, FR-CFG-03, FR-PRP-02, FR-PRP-04. **Actions:** AC-02, AC-09, AC-13, AC-22. **Information:** IC-03, IC-06, IC-18, IC-24, IC-25, IC-26, IC-28, IC-31, IC-34, IC-38, IC-41, IC-53, IC-56, IC-57, IC-71. **Native profiles:** IP-09.

**Conclusion:** No blocking ownership/information contradiction identified within this described route. Full numerical scenario execution, independent representability signoff and independent physical validation are not established by this card.

### VR-32 / SC-32: Polymer distributions and material attributes

**Profile:** P3. **Review:** authored semantic/action walkthrough.

**Case:** Distribution support, mass/number weighting, retained attributes and explicit reduction lineage.

**Route:** Associate distribution with material quantity; combine compatible weighted distributions; record losses when reducing moments.

**Adversarial boundary:** One mean is not a lossless full distribution and cannot generate a unique detailed inverse.

**Contributing executed probes:** No new numerical probe for the complete scenario; inherited reference records remain separately classified.

**Remaining acceptance gate:** Conceptual route and inherited weighted arithmetic; P3 numerical thermodynamics remains deferred.

**Requirement route:** FR-EXT-02, FR-MAT-02, FR-MAT-03, FR-FLW-06, FR-LIF-04. **Actions:** AC-02, AC-03, AC-13, AC-42, AC-50. **Information:** IC-03, IC-06, IC-07, IC-08, IC-16, IC-20, IC-25, IC-26, IC-28, IC-32, IC-47, IC-51, IC-62, IC-68, IC-71. **Native profiles:** IP-14.

**Conclusion:** No blocking ownership/information contradiction identified within this described route. Full numerical scenario execution, independent representability signoff and independent physical validation are not established by this card.

### VR-33 / SC-33: Inventory-based state and dynamic compatibility

**Profile:** P3. **Review:** authored semantic/action walkthrough.

**Case:** Actual inventory composition/amounts and total U,V with location/time meaning, including a separately empty case.

**Route:** Capture unresolved T,P as outputs without dummy flow; distinguish inventory from a numerical reference amount.

**Adversarial boundary:** Total U,V without justified material amount does not uniquely specify a fluid state.

**Contributing executed probes:** No new numerical probe for the complete scenario; inherited reference records remain separately classified.

**Remaining acceptance gate:** Representability walkthrough only; no native inventory solve or time integration claimed.

**Requirement route:** FR-EXT-03, FR-MAT-03, FR-MAT-04, FR-STA-01, FR-STA-07. **Actions:** AC-13, AC-15. **Information:** IC-25, IC-26, IC-27, IC-28, IC-35, IC-48, IC-71. **Native profiles:** IP-14.

**Conclusion:** No blocking ownership/information contradiction identified within this described route. Full numerical scenario execution, independent representability signoff and independent physical validation are not established by this card.

### VR-34 / SC-34: Restricted-equilibrium and metastable-state requests

**Profile:** P3. **Review:** authored semantic/action walkthrough.

**Case:** Declared excluded phases/reactions or requested metastable branch, rationale, candidate scope and reference to original unrestricted question.

**Route:** Create a distinct restricted problem; qualify the limited claim; lifting restrictions invalidates applicability to the new problem.

**Adversarial boundary:** A branch seed is not a physical restriction, and constrained success is not unrestricted stable equilibrium.

**Contributing executed probes:** NP-12, NP-13

**Remaining acceptance gate:** Degeneracy controls only; metastable/phase-restricted native operations remain deferred.

**Requirement route:** FR-EXT-04, FR-CFG-05, FR-EQL-06, FR-EQL-07, FR-RUN-05. **Actions:** AC-07, AC-23, AC-34. **Information:** IC-22, IC-29, IC-30, IC-35, IC-41, IC-42, IC-54, IC-56, IC-57, IC-72. **Native profiles:** IP-14.

**Conclusion:** No blocking ownership/information contradiction identified within this described route. Full numerical scenario execution, independent representability signoff and independent physical validation are not established by this card.

## 9. Numerical probe results and reproducibility

Every row below is an executed case, including explicitly expected rejections and diagnostics. A PASS for a negative control means the specified unsafe interpretation was rejected or exposed, not that the unsafe input was accepted.

| Probe | Executed subject | Result | Evidence class |
| --- | --- | --- | --- |
| NP-01 | NRTL published numerical example | PASS | Extracted upstream function |
| NP-02 | Rachford-Rice residual example | PASS | Extracted upstream function |
| NP-03 | Host Brent root of extracted residual | PASS | Authored host/formulation with extracted function where applicable |
| NP-04 | NRTL all-axis constituent permutation | PASS | Extracted upstream function |
| NP-05 | Flash component permutation invariance | PASS | Authored host/formulation with extracted function where applicable |
| NP-06 | NRTL ideal and pure-component limits | PASS | Extracted upstream function |
| NP-07 | NRTL Gibbs-Duhem composition tangent | PASS | Extracted upstream function |
| NP-08 | Temperature-dependent NRTL caloric identity | PASS | Authored host/formulation with extracted function where applicable |
| NP-09 | Invalid composition is rejected at host boundary | PASS | Authored host/formulation with extracted function where applicable |
| NP-10 | Directed-parameter transpose negative control | PASS | Extracted upstream function |
| NP-11 | Single-phase states do not fabricate absent composition | PASS | Authored host/formulation with extracted function where applicable |
| NP-12 | Identical K values expose allocation degeneracy | PASS | Authored host/formulation with extracted function where applicable |
| NP-13 | Near-degenerate allocation conditioning diagnostic | PASS | Authored host/formulation with extracted function where applicable |
| NP-14 | Zero-feed component is preserved in fixed-species split | PASS | Authored host/formulation with extracted function where applicable |
| NP-15 | Synthetic coherent caloric TP to PH/PS roundtrips | PASS | Authored host/formulation with extracted function where applicable |
| NP-16 | Condensed VLE derivative includes changing allocation | PASS | Authored host/formulation with extracted function where applicable |
| NP-17 | Coherent energy/entropy temperature response | PASS | Authored host/formulation with extracted function where applicable |
| NP-18 | Synthetic valve process balance and outlet PH | PASS | Authored host/formulation with extracted function where applicable |
| NP-19 | Compressor PS reference is distinct from actual PH outlet | PASS | Authored host/formulation with extracted function where applicable |
| NP-20 | Separator routing after shared equilibrium | PASS | Authored host/formulation with extracted function where applicable |
| NP-21 | Synthetic two-liquid equilibrium using extracted NRTL | PASS | Authored host/formulation with extracted function where applicable |
| NP-22 | Heat-only coupling to a mass-only empirical material | PASS | Authored host/formulation with extracted function where applicable |
| NP-23 | Condensed versus exposed reactive energy solve | PASS | Authored host/formulation with extracted function where applicable |
| NP-24 | Strict extent rejects a hidden limiting alternative | PASS | Authored host/formulation with extracted function where applicable |
| NP-25 | Caloric supplementation must enter inverse equations | PASS | Authored host/formulation with extracted function where applicable |
| NP-26 | Saturation-like discontinuity defeats naive sign-bracket acceptance | PASS | Authored host/formulation with extracted function where applicable |
| NP-27 | Phase-property evaluation preserves separate bulk authority | PASS | Authored host/formulation with extracted function where applicable |
| NP-28 | Generated positive five-component allocation cases | PASS | Authored host/formulation with extracted function where applicable |
| NP-29 | Mixer energy balance and mechanical split | PASS | Authored host/formulation with extracted function where applicable |
| NP-30 | Parent recycle convergence differs from local calculation | PASS | Authored host/formulation with extracted function where applicable |

Results contain actual values, residuals and declared limitations. The 200 generated allocation cases, seven-state inverse sweep and six-way permutations are subcases within their named tests; they are not counted as hundreds of distinct validated process scenarios. Published example agreement is an upstream numerical regression, not independent experimental accuracy evidence.

### Rerun instructions

From an extracted bundle with a compatible Python environment and NumPy/SciPy available:

```bash
python scripts/run_numerical_probes.py
python scripts/check_publication_model.py
python scripts/run_native_probes.py
python scripts/validate_validation_package.py
```

The native runner intentionally exits with a nonzero incomplete-run code when required packages or exact requested versions are missing. It does not install software, bypass an unavailable dependency, or substitute another version. Its package worker code follows the cited public APIs but was not exercised with the packages here. Do not call those workers validated merely because the blocked-dispatch branch executed.

The target versions in this probe file are CoolProp 8.0.0 and thermo 0.6.1. They are separate from the earlier source pins; their existence was checked in publisher metadata, but no release artifact was downloaded or installed. A future actual execution records the observed version/build and data. Any change to fixtures, dependencies or tolerances produces a new evidence record rather than rewriting this session's result history. [SRC-05..10]

No binaries are redistributed. The only redistributed upstream source consists of the two small MIT-noticed extracts; all wrappers, synthetic models, validators and reference-publication code are authored test artifacts. They are not a production thermodynamics implementation recommendation.

## 10. Experiment and profile coverage ledger

The following ledger retains all eighteen Step-8 investigation IDs. Contributing authored tests are useful preparation but do not complete their required native/control/process witnesses.

| Investigation | New contributing probes | Full native investigation |
| --- | --- | --- |
| IE-01 Freeze data and reconstruct a package | NP-04 | Not executed; original native probe obligations remain open. |
| IE-02 Caloric completion and inverse consistency | NP-08, NP-15, NP-25 | Not executed; original native probe obligations remain open. |
| IE-03 One-liquid and multiple-liquid authority | NP-03, NP-05, NP-11, NP-14, NP-20, NP-21 | Not executed; original native probe obligations remain open. |
| IE-04 Inverse initializer and saturation eligibility | NP-12, NP-13, NP-15, NP-26 | Not executed; original native probe obligations remain open. |
| IE-05 Phase-property callback preservation | NP-27 | Not executed; original native probe obligations remain open. |
| IE-06 Total reduced response versus local derivative | NP-16, NP-17, NP-23 | Not executed; original native probe obligations remain open. |
| IE-07 Inner-error and finite-difference budget | NP-13, NP-16 | Not executed; original native probe obligations remain open. |
| IE-08 Hessian, chart and sparse-structure contract | Walkthrough / inherited reference model only | Not executed; original native probe obligations remain open. |
| IE-09 Phase-transition and branch strategy | NP-11, NP-12, NP-15, NP-26 | Not executed; original native probe obligations remain open. |
| IE-10 Coupled chemistry, reaction heat and open exchange | NP-23, NP-24 | Not executed; original native probe obligations remain open. |
| IE-11 Supplemental transport and effective properties | NP-27 | Not executed; original native probe obligations remain open. |
| IE-12 Heat-only and directed material translation | NP-22 | Not executed; original native probe obligations remain open. |
| IE-13 Sessions, interleaving and native failure | Walkthrough / inherited reference model only | Not executed; original native probe obligations remain open. |
| IE-14 Original inputs, alternative recovery and optional failure | Walkthrough / inherited reference model only | Not executed; original native probe obligations remain open. |
| IE-15 Process integration, iteration lineage and publication | NP-30 | Not executed; original native probe obligations remain open. |
| IE-16 Petroleum, reduced and empirical scope | NP-22 | Not executed; original native probe obligations remain open. |
| IE-17 Exact restoration and upgrade comparison | Walkthrough / inherited reference model only | Not executed; original native probe obligations remain open. |
| IE-18 Independent coverage review including P3 | Walkthrough / inherited reference model only | Not executed; original native probe obligations remain open. |

All fourteen profile templates retain their actual scope. In particular, an artificial NRTL LLE example does not qualify general real VLL, precipitation or reactive separation; an ideal two-species chemical example does not qualify Reaktoro or eNRTL data; a mass-only heat balance does not supply petroleum characterization or black-oil correlations. P1/P2 delivery gaps remain mandatory scope items, not optional omissions.

### Next concrete qualification order

First, execute a coherent conventional-fluid assembly and a pure-fluid utility backend with pinned data and the intended adapters. Then qualify one nonideal-liquid and one multiple-liquid configuration independently, including caloric inverse states and phase-only authority. Next, exercise a coherent actual chemical system with closed and explicitly open energy/material constraints. In parallel, qualify real provider sessions and the actual revision/publication implementation. Petroleum, black-oil, true nonequilibrium contacting and empirical-field-material fixtures retain their own explicit completion work.

Each profile needs immutable actual data, declared parameter formulas and references, positive/negative operating cases, initializer preconditions, actual returned postconditions, optional-property behavior, valid solver mode, and build-specific failure/reuse behavior. Only after that evidence can the corresponding production-use decision be made. No model-count ranking or language binding can substitute for it.

## 11. Implementation-design release decision

**Ready to carry forward:** the ten responsibility packages; seventy-two information concepts; original problem versus attempt; explicit phase/species/root authority; operation-specific readiness; conditional/optional outputs; phase identity separate from routing; compatible material/energy translation; qualified local and parent scopes; guarded revision/run/group publication; and reconstructable semantic archives.

**Acceptance refinements to carry with them:** fixed-K/all-K-one degeneracy and near-degenerate derivative conditioning; host rejection of non-normalized composition before raw kernels; explicit evidence class for extracted source versus installed package; and the concrete negative controls for scope/lineage/generation/replay adoption.

**Not released by this evidence:** a universal backend selection, claim of complete native interface compatibility, exact Hessian or broad inverse-flash coverage, arbitrary native concurrency, real-fluid physical accuracy, or complete P1/P2 scenario execution.

The next implementation can use these artifacts to build the domain contracts and test harness without waiting for a perfect universal thermodynamic library. Native integrations must still enter through the stated gates. This is conditional design acceptance, not a declaration that the full simulator has already been built or validated.

## 12. Source register and artifact integrity

The predecessor source pins and limitations remain in the unchanged B2/B3 reports. B4 supplies required behavior, B5 ownership, B6 information meaning, B7 action semantics and B8 integration gates. The new source entries below identify only the evidence actually used in this stage.

### SRC-01

**Kind:** source_extracted_numerical_body. **Source:** [NRTL_gammas](https://github.com/CalebBell/thermo/blob/2bb466e98439c2395a004e7095d07f9b97a5a0f0/thermo/nrtl.py).

**Commit:** `2bb466e98439c2395a004e7095d07f9b97a5a0f0`. **Original-file blob:** `816d8eb7034bfc553b9d651acbb8dfeba9432cbf`. **Retrieved windows:** [[2010, 2200], [1, 35]].

Numerical body manually transcribed; original docstring/package context omitted; math.exp supplied; MIT notice retained. Not full package or exact original-file bytes.

### SRC-02

**Kind:** source_extracted_numerical_body. **Source:** [Rachford_Rice_flash_error](https://github.com/CalebBell/chemicals/blob/e79047588b30cfabc564c79fb26d760c746877d7/chemicals/rachford_rice.py).

**Commit:** `e79047588b30cfabc564c79fb26d760c746877d7`. **Original-file blob:** `ef08efe5244f742525e7b914ac95736219febd9f`. **Retrieved windows:** [[610, 735], [1, 24]].

Numerical body and signature transcribed; upstream docstring omitted; MIT notice retained. Host Brent solver is not the upstream flash dispatcher.

### SRC-03

**Kind:** official_documentation_worked_example. **Source:** [Official documentation / publisher metadata](https://thermo.readthedocs.io/thermo.nrtl.html).

Activity coefficients, parameter conversion and rounded excess enthalpy example. Same-model numerical regression, not independent physical validation.

### SRC-04

**Kind:** official_documentation_worked_example. **Source:** [Official documentation / publisher metadata](https://chemicals.readthedocs.io/chemicals.rachford_rice.html).

Specified K-value residual and allocation result. Inner algebraic split, not a complete equilibrium thermodynamic package.

### SRC-05

**Kind:** official_documentation_unexecuted_native_runner. **Source:** [Official documentation / publisher metadata](https://coolprop.org/coolprop/HighLevelAPI.html).

PropsSI input/output semantics and pure saturation usage; not executed here.

### SRC-06

**Kind:** official_documentation_unexecuted_native_runner. **Source:** [Official documentation / publisher metadata](https://coolprop.org/coolprop/LowLevelAPI.html).

Version/backend/state semantics; not executed here.

### SRC-07

**Kind:** official_documentation_unexecuted_native_runner. **Source:** [Official documentation / publisher metadata](https://thermo.readthedocs.io/thermo.flash.html).

Explicit CEOS/FlashVL assembly used in blocked full-package runner.

### SRC-08

**Kind:** official_documentation_unexecuted_native_runner. **Source:** [Official documentation / publisher metadata](https://thermo.readthedocs.io/thermo.heat_capacity.html).

HeatCapacityGas polynomial representation in blocked worker.

### SRC-09

**Kind:** publisher_release_metadata. **Source:** [Official documentation / publisher metadata](https://pypi.org/project/CoolProp/8.0.0/).

Target version and downloadable wheel exist; retrieval here failed. No installation claim.

### SRC-10

**Kind:** publisher_release_metadata. **Source:** [Official documentation / publisher metadata](https://pypi.org/project/thermo/0.6.1/).

Target version, not an installed build.

### Integrity and limitations

The new package audit passed **49 structural checks**. It verified 11 local Markdown links and 52 primary payload files, with the derived audit and manifest excluded from their own checksum set.

The package audit checks predecessor hashes, preservation of all 94 requirements and 34 profile assignments, valid scenario-to-test references, owner consistency, complete findings, actual output counts, retained native-blocked status and artifact links. It does not prove that a written requirement is sufficient for every future physical theory or that a future implementation obeys it.

The [artifact manifest](validation_artifact_manifest.json) records file hashes and sizes. The [validation audit](thermodynamics_blueprint_validation_v0_1_audit.json) records the checks actually performed. Rerunning numerical scripts creates a new output record; preserve the original bundle before updating evidence.

**Final thesis:** the conceptual blueprint has survived targeted semantic, numerical-boundary and behavioral challenges at the layers exercised. The architecture is ready for conditional implementation design; the actual thermodynamics providers and real-process coverage remain explicitly evidence-gated.
