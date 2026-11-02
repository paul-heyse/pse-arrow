---
title: "Equivalence and qualification"
notion_source: "https://app.notion.com/p/40b141b58369813983bcd00450e3b1c8"
notion_last_edited: "2026-11-01T23:34:07.529Z"
---

# Equivalence and qualification

Source: [Notion](https://app.notion.com/p/40b141b58369813983bcd00450e3b1c8). Imported 2026-11-02.

[Implementation design and IDAES equivalence plan](index.md)

**Adopted equivalence and qualification contract** · 2026-11-01. This page defines how to demonstrate equivalence; it does not report a newly executed qualification campaign.

## 1. Define the comparison before implementing the case

Functional equivalence means that a user can express the reference engineering problem, select the necessary physical models and parameter data, perform the requested analysis, and obtain the required scientifically qualified result or the expected, attributable diagnostic/refusal. Python class names, Pyomo expression trees, solver iteration sequences and serialized object layouts are not the scientific target.

Every reference case must resolve five questions:

| Question | Required meaning |
| --- | --- |
| What problem is being reproduced? | Source repository and revision, logical case identity, configuration, selected model and data, fixed/free roles, conditions, objectives and requested outputs |
| What assumptions must agree? | Component/phase membership, thermodynamic and reaction formulations, parameterization, units/bases/datums, smoothing or discrete formulation, boundary/initial conditions and branch selection |
| What constitutes an answer? | Named physical outputs, trajectory or study coverage, feasibility, statistical interpretation, optimization qualification and required diagnostic products |
| How is it compared? | Explicit observable correspondence, unit/reference conversions, comparison grid, dimensional absolute and relative tolerances, and applicable independent physical checks |
| What establishes the claim? | Actual native and independent-reference outcomes, source and environment identities, original-space assessments, limitations and command/campaign ownership |

These are fields of existing authored fixtures, source notes, selected reference exchanges and retained outcomes where those owners already express them. The plan does not introduce a second scientific registry or a parallel result-status machine. The Notion inventory is a planning index.

## 2. Use two distinct comparison modes

**Reference reproduction** aligns the scientific formulation and input data. It establishes that an IDAES case has a native equivalent. Differences in algorithm, sparse ordering, discretization implementation or state coordinates are acceptable only when the output correspondence and numerical approximation are explicit.

**Scientific or numerical improvement** records an intentional change and evaluates it on an appropriate independent basis. A better property model needs stronger agreement with independent data or a justified reference. A better numerical route needs comparable physical acceptance with measured improvements in accuracy, robustness or cost. A lower objective than one local IDAES solution establishes a better feasible candidate for that case; it does not establish global optimality.

An alternative property model that merely converges faster cannot silently replace the matched-physics reproduction case. Preserve both purposes in existing case/configuration declarations. Similarly, a filtered PID and an unfiltered PID are different control laws, even if both are useful. The current process source explicitly documents its derivative-filter addition and selected comparisons. [Process source notes](https://github.com/paul-heyse/pse-arrow/blob/19e7b49a67df24c1bf73005a6186e145e930d53a/packages/reference/process/sources.md).

Assess substitutability against the requested observations, operating region, material/phase representation, datum, accuracy, derivatives and effects. An interface match establishes possible composition, not equivalence for all uses. Preserve value-only, derivative and stateful/dynamic capability differences in the case contract; an external evaluator cannot earn time-advance equivalence without the required lifecycle and output-coverage evidence. Use the existing demanded-property, native-capability and retained-outcome owners described in [Scientific packages and equipment delivery](scientific-packages-and-equipment.md) and [Architecture and process authoring](architecture-and-process-authoring.md).

## 3. Build on the current evidence owners

The repository already separates native candidates, isolated scientific references and original-space acceptance. Current live IDAES parity has exactly four routes: sensitivity against sIPOPT, covariance against Pyomo parmest, degeneracy diagnostics, and a PI loop against IDAES/PETSc plus an analytic response. `just parity` prepares the locked Python 3.13 reference environment and runs `reference_sensitivity_exchange`, `reference_covariance_exchange`, `reference_degeneracy_exchange` and `reference_pid_exchange` in `pse-runtime`. Their isolated drivers exchange explicit cases/results with native candidates. These selected comparisons do not establish broad equipment-library or example equivalence; authored scientific conformance and the thermodynamic campaign below have separate populations and owners. [Current command](https://github.com/paul-heyse/pse-arrow/blob/19e7b49a67df24c1bf73005a6186e145e930d53a/justfile); [reference drivers](https://github.com/paul-heyse/pse-arrow/tree/19e7b49a67df24c1bf73005a6186e145e930d53a/tests/reference/oracles).

Extend the existing authored conformance fixtures and generated `runtime.modeling_conformance`, `runtime.modeling_fixture_status` and `runtime.modeling_findings` products. Extend native candidate exchange and the independent reference drivers when a case needs an external comparison. The runtime's existing attempts, completion assessments, exact retained inputs and lineage remain the result authority. Keep independent analytic and literature checks where already present; replacing them with golden values generated by the candidate implementation would weaken the evidence.

The current thermodynamic campaign and Plan 58h already own broader frozen flash/sweep and assembled scientific/performance work. Carry their unresolved qualification forward under those owners. Adding process cases does not retrospectively qualify those campaigns. [Qualification contracts](https://github.com/paul-heyse/pse-arrow/blob/19e7b49a67df24c1bf73005a6186e145e930d53a/docs/authoritative_design/sections/operations-and-validation.md#section-24-2); [Plan 58h](https://github.com/paul-heyse/pse-arrow/blob/19e7b49a67df24c1bf73005a6186e145e930d53a/docs/plans/58h-qualification-and-publication.md).

## 4. Qualification grows from properties to complete workflows

| Level | Acceptance emphasis | Examples of defects it should expose |
| --- | --- | --- |
| Property and reaction contracts | Values and declared derivatives; state reconstruction; chemical and caloric conventions; data and validity coverage | Incorrect reference enthalpy, missing binary parameters treated as zero, wrong reaction-rate basis, undefined phase properties |
| Unit configurations | Original balances, supplied/receiving state responsibility, specification closure, initialization and configuration-dependent equations | Duplicate normalization, over-specified duty and outlet temperature, double-counted reaction heat, unsupported holdup allocation |
| Coupled flowsheets | Connection identity, component correspondence, recycle behavior, cross-unit balances and specification recovery | Species-order mismatches, hidden causal inputs, leaked temporary fixes, stream aliases counted as material splitting |
| Engineering analyses | Task-specific outputs and qualification over the same physical identities | Changed likelihood, incomplete time coverage, unqualified covariance, local optimum described as global |
| Complete examples and operational use | Authored case reproducibility, selected reference outputs, usable failure diagnostics, exact retained results and reproducible performance conditions | A subprocess exit masking failed science, partial population presented as complete, stale warm starts or values |

Shared checks should be supplied by existing model and analysis contracts, with case-defining inputs and acceptance meaning visible. Admission/policy tests use their actual inputs without unrelated native or storage startup; native integration tests exercise the real consumed route. Share domain types and valid setup where appropriate, but do not derive expected values through the production calculation being checked. Independent analytic and reference checks may repeat a scientific relation without becoming a second production authority.

For an expected failure, establish that the case reaches the intended boundary and fails for the intended reason; a fixture/setup error does not satisfy it. Use focused wrong-value, missing-output, ordering or convention controls when they discriminate consequential comparator failures, without requiring a mutation campaign or a new test framework. Reuse established immutable structural validity only while its premises hold; every new numerical state still needs applicable original residual, bounds, domain and independent physical closure checks. Structural admission alone establishes neither uniqueness nor global physical validity. [PS-04/10/13](https://github.com/paul-heyse/pse-arrow/blob/19e7b49a67df24c1bf73005a6186e145e930d53a/docs/design_review/design_principles/profiles/process-simulator/principles.md); [Test-assessment companion](https://github.com/paul-heyse/pse-arrow/blob/19e7b49a67df24c1bf73005a6186e145e930d53a/docs/design_review/design_principles/core/test-assessment.md).

## 5. Comparison rules depend on the result

For a scalar observable, use a declared dimensional rule such as `abs(native-reference) <= absolute_allowance + relative_allowance * scale`. Define the scale and near-zero treatment for the observable. Do not select one global epsilon for temperature, trace composition, duty, pressure and fitted parameters. Report physical closure separately from reference agreement and from scaled solver tolerances.

For thermodynamics, compare across a declared operating region and relevant phase boundaries. Include state-coordinate round trips, caloric derivative consistency, equilibrium residuals, selected root/phase behavior and property validity. Use additional physical identities and independent sources where warranted. Re-evaluating the same implementation twice is not an independent scientific check. Stable or globally stable equilibrium is claimed only with the corresponding evidence.

For dynamics, agree on physical input profiles, initial conditions, events/resets and requested interval. Compare trajectories at defined physical times with a justified reconstruction/interpolation contract; compare event times separately. Check inventory changes against integrated original fluxes, including event transfers. Perform spatial/time refinement where approximation error matters. The reference trajectory is not an exact continuum truth merely because IDAES produced it.

Apply those obligations to the selected route's actual capabilities. Current integrated dynamics supports a fixed-layout ODE/index-1 DAE profile; IDAS sensitivities across events, adjoints across events/resets, and authored events on the simultaneous route remain unsupported. A terminal-event prefix cannot supply missing future observations or whole-horizon integrals. Temporal dynamics of the spatial control-volume seed also needs a new accumulation closure. Cases demanding these capabilities require an explicit implementation increment and cannot receive equivalence credit from another dynamic route. [Current dynamic scope](https://github.com/paul-heyse/pse-arrow/blob/19e7b49a67df24c1bf73005a6186e145e930d53a/docs/authoritative_design/sections/scope-and-open-design.md#section-25); [trajectory contracts](https://github.com/paul-heyse/pse-arrow/blob/19e7b49a67df24c1bf73005a6186e145e930d53a/docs/authoritative_design/sections/workflows-and-results.md#section-13).

For optimization, compare original-space feasibility and objective values under matched specifications and economic conventions. Different primal points can be equivalent under degeneracy. Retain stationarity, curvature, bound/gap and local/global status where supported. A global claim requires a valid bound or certificate, rather than a collection of local starts.

For fitting and reconciliation, agree on observation mappings, quantity/basis and measurement or aggregation intervals, error model, known versus estimated variance, weights, priors/regularization and shared versus case-local parameters. Preserve source records, uncertainty assumptions, missing-data treatment and any mapping from acquisition time to process time. An interval average cannot silently become a pointwise state. Fit and independent predictive evidence have distinct roles. Assess parameter uncertainty only under its stated assumptions and local identifiability evidence. A good residual alone does not identify confounded parameters; exchanger `U` and area provide a concrete witness.

For stochastic or sampled analyses, retain distribution/dependence, randomization and failure treatment. Use the same recorded realized inputs when comparing deterministic responses case by case; a shared seed alone does not establish sample identity. Independent sampling algorithms can instead be compared through the requested statistical quantities and their estimation uncertainty. For a learned surrogate, distinguish reproducing predictions of a pinned artifact from reproducing an entire training algorithm.

## 6. Breadth is a finite, versioned obligation

Maintain a reviewed case/configuration inventory tied to pinned IDAES and example revisions. Each mandatory row must have an explicit native realization, matched reference conditions and accepted evidence. A representative first case establishes the mechanism for a family; it does not discharge every configuration in that family.

State coverage as a qualified claim over the actual method/data revision, material and phase representation, operating region, unit configuration, analysis operation and requested output population. Existing fixtures, source notes, reference exchanges and retained outcomes hold the underlying evidence; the planning inventory links those owners. A process or industry label is not evidence for every material, configuration or analysis in it. Additional industry archetypes remain separately selected expansion work and do not silently enlarge or shrink the pinned IDAES acceptance denominator.

Use each upstream case's meaningful configurations and assertions to define required coverage. Cover additional semantic risks with focused cases, including component reordering, incompatible datums, missing composition, phase disappearance, failed initialization and invalid property regions. For a deliberately singular, invalid or infeasible case, the expected diagnostic/refusal may pass its conformance assertion while the native attempt retains its actual non-success outcome. Pairwise configuration sampling can help discovery; it does not replace explicitly promised configurations or mandatory reference cases.

Unavailable optional software, solver binaries, datasets or external executables leave a named case unqualified. For a selected mandatory parity run, missing dependencies or a wrong pin are failures under the existing repository policy, never successful skips. Preserve the zero-failure target for the complete selected acceptance population; retain failed, inconclusive, cancelled and unattempted cases under their existing outcome meanings rather than counting them as passes. Scope changes remain explicit; they cannot be used to make a completion percentage look better. [Repository instructions](https://github.com/paul-heyse/pse-arrow/blob/19e7b49a67df24c1bf73005a6186e145e930d53a/AGENTS.md).

## 7. Demonstrate execution quality without changing the science

Measure first preparation, repeated value-only edits, topology/configuration edits, recycles, fitting/studies and dynamic trajectories separately. Record hardware, thread/placement policy, tolerances, initialization, model size and selected native/reference versions. Use the existing native host and measurement contracts. A value-only edit should reuse admissible structure; numerical factors and warm starts are reused only while their own premises hold.

Report robust completion over a declared case set as well as timing among successes. Preserve full scientific failure outcomes. Use repeated measurements and distributions where timing variability matters; no speedup or memory claim is established by this design review. Performance acceptance supplements physical and numerical acceptance.

## 8. Completion claims

Use the repository's evidence vocabulary: **Proposed**, **Interface-checked**, **Implemented**, **Tested**, **Measured**, and **Formally established** where appropriate. The high-level plan is adopted as design guidance. Unimplemented behavior remains Proposed in this evidence vocabulary; documentation adoption is not a promotion to Implemented, Tested or Measured. Source inspection is not a newly executed test result.

The program can claim **pinned flowsheet-process equivalence** only when all mandatory process capability/configuration and example obligations have accepted evidence, with named exclusions outside that claim. A broader **whole-distribution functional equivalence** claim also requires the separately identified non-process application obligations. Keep improved-physics and beyond-release capabilities visible without using them to conceal an unimplemented reference case.

The selected milestone is complete when a model author can reproduce the declared cases through the normal native authoring and execution path, and the retained results substantiate the claimed scientific and operational behavior.
