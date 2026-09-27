---
title: Modeling kernel before knowledge — implementation plan
status: in-progress
date: 2026-09-26
adrs: [ADR-0097, ADR-0098, ADR-0099, ADR-0100, ADR-0101]
review_sources: [docs/design_review/reviews/design_review_idaes-capability-target_2026-09-26.md]
scenario_sources: [docs/design_review/reviews/design_review_idaes-capability-target_2026-09-26.md#s01]
---

# Modeling kernel before knowledge — implementation plan

**Status: authorized K0–K8 implementation complete, 2026-09-27.** The
[K0–K3 execution packet](21-modeling-kernel-k0-k3-execution.md) owns packet progress.
The [K4–K7 execution packet](21-modeling-kernel-k4-k7-execution.md) owns its implementation
and original evidence. The [K8 execution packet](21-modeling-kernel-k8-execution.md)
owns completed seed/retirement progress, corrective K1–K7 evidence and the assessment
results. Its selected checks retain the earlier passing fixtures as the maintainer requested.
K9 remains proposed and excluded from this execution. Architectural decisions remain
proposed pending their decision PRs.

**Companion documents:**

- [Kernel architecture](21-modeling-kernel-architecture.md): concepts, semantics, pipeline,
  crates, registry, alignment.
- [Knowledge placement and porting guide](21-knowledge-placement.md): where every kind of
  idaes-pse knowledge sits, and how it is ported.

## Context

On 2026-09-26 the maintainer agreed to the ten decisions of
[Plan 20](20-idaes-capability-target.md#decisions). They cover:

- full idaes-pse scope;
- the parity reference moving to 2.13.0;
- a potential-based property system with FeOS leaving production;
- the modeling crate;
- template mechanisms;
- the indexed law engine;
- formulation primitives;
- analysis modes and discretization;
- initialization knowledge and derived scaling;
- the diagnostics catalogue and studies.

The maintainer then asked for a stronger form of that target. Unit operations, property sets and
the like must not each be their own concept and contract. The system must be composable and
extensible enough that carrying idaes-pse's scientific knowledge across is *placement of data
in the structure where that knowledge belongs*, not further logic or contract coding. Build
the architecture first, integrate it with the symbolic and solver infrastructure, and port only
the minimum knowledge needed for integrated testing.

This plan builds that **modeling kernel**. It then proves the kernel with a small, deliberately
chosen **seed** of scientific knowledge validated against IDAES oracles. After this plan, Plan
20's waves W1–W5 become data-only porting packets.

## Decisions

| ID | Decision | Status | Route |
|---|---|---|---|
| A1–A10 | Plan 20 decisions (scope, parity pin 2.13.0, property system, crates, template mechanisms, law engine, formulation primitives, analysis modes, numerical knowledge, diagnostics and studies) | **Agreed by the maintainer, 2026-09-26.** ADRs to be drafted in K0, in the refined form below | ADR route per Plan 20 |
| R1 (refines A3) | The property system is **knowledge on the kernel, not a code subsystem**. Thermodynamic identities are interface default members; equations of state and correlations are functions; property packages are binding definitions; closures are implicit blocks with realization policies (inline, nested, accelerated). No `pse-properties` or `pse-thermo` crate | Proposed | Folded into the A3 ADR |
| R2 (refines A4) | One new crate, `pse-modeling`, holds all kernel semantics. `pse-kernels` becomes the external-function host (generic contract with index shapes); FeOS and valve-law providers are deleted once their knowledge is data | Proposed | Folded into the A4 ADR |
| R3 (refines A5) | A **modeling language** is the primary authoring surface. Its parsed form is a small generic IR declared once in the registry (D1 kept). Science-specific registry families are deleted ([architecture §6](21-modeling-kernel-architecture.md#6-registry-consolidation)) | Proposed | Folded into the A5 ADR |
| R4 (refines A6) | Accumulators are generic (conservation or accounting). Material, energy, element, momentum, charge and cost subjects are package data | Proposed | Folded into the A6 ADR |
| R5 (refines A7) | Only Symbolica built-in functions are code primitives. Smoothing, complementarity, `cbrt`, hyperbolic and activation functions are package functions; `piecewise` functions have kernel-verified continuity | Proposed | Folded into the A7 ADR |
| R6 (refines A8, A9) | Analysis mode and initialization stage are compile-time facts read by `when` variants in package data. Starts, nominals, scale schemes, validity, checks and reports are annotations consumed by generic engines | Proposed | Folded into the A8/A9 ADRs |
| N1 | **Knowledge boundary test** is the acceptance criterion of the architecture ([architecture §1](21-modeling-kernel-architecture.md#1-the-principle-mechanisms-in-code-knowledge-in-data)) | Proposed | Plan acceptance |
| N2 | **Conformance tests are data** next to the knowledge they validate, run by one kernel harness | Proposed | Plan acceptance |

## Architecture in brief

Twelve kernel concepts carry all scientific knowledge
([architecture §2](21-modeling-kernel-architecture.md#2-kernel-concepts)):

| Concept | Concept | Concept |
|---|---|---|
| K1 quantity types | K5 functions | K9 accumulators |
| K2 entity kinds | K6 definitions | K10 operators (∂, d/ds) |
| K3 sets | K7 interfaces with default members | K11 transformations and realization policies |
| K4 tables and datasets | K8 lazy instantiation and dispatch | K12 annotations, requirements and tests |

The kernel defines no scientific concept. Packages give names such as "unit", "phase" or
"equation of state" to ordinary definitions, interfaces, sets and tables.

## The knowledge boundary test

> A port of any idaes-pse unit model, property method or package, reaction form, costing method,
> controller or surrogate consists only of package files and passes its conformance tests.
> Any Rust change it needs is a kernel gap: it is recorded in *Kernel gaps* below, generalized,
> and closed in the kernel with a synthetic test, never special-cased.

The seed (K8) is the first application of this test. Its success criterion is that seed
authoring needed **zero** model-specific Rust.

## Plan

Packets are dependency-ordered. Each names targeted tests over **synthetic knowledge** (kernel
mechanisms are tested without real science) and what it deletes (AGENTS.md *Execution rhythm*).

| Packet | Responsibility / dependencies | Scenarios / acceptance | Replaced code / deletion | Status or status-owner link |
|---|---|---|---|---|
| K0 Decisions and review | Draft ADRs for A1–A10 in the refined form (R1–R6); design review of the kernel architecture with the `design-review` and `design-review-process-simulator` skills; move the parity pin to 2.13.0 (A2) | Review Accept or Accept-scoped; ADRs accepted | Parity pin 2.12.0 | [execution packet](21-modeling-kernel-k0-k3-execution.md#sequence-and-progress) |
| K1 Language and IR | Grammar and parser for packages, entity kinds, sets, tables and datasets, functions, interfaces, definitions, presets, annotations, requirements, tests and cases. The existing DSL becomes the expression layer. Printer round trip; spans; identities; generic registry IR families + `just codegen` | Parse → render → parse identity over a synthetic corpus; span-accurate diagnostics; identity stable under rename | Science-specific registry families and their reference YAML documents (list in [architecture §6](21-modeling-kernel-architecture.md#6-registry-consolidation)); template relation families replaced by the definition IR | [execution packet](21-modeling-kernel-k0-k3-execution.md#sequence-and-progress) |
| K2 Typing | Quantity type variables and polymorphic signatures (`pse-quantity`); entity, set and table typing; interface conformance (members, defaults, overrides); definition checking before instantiation; requirement typing | Positive and negative controls: polymorphic smoothing function; interface with a default; wrong-basis contribution refused | — | [execution packet](21-modeling-kernel-k0-k3-execution.md#sequence-and-progress) |
| K3 Specialization engine (`pse-modeling` + Salsa queries in `pse-compiler`) | Parameter binding; scope values; `when` resolution; requirement evaluation over tables; children and indexed children; **lazy demand closure**; **dispatch grouping**; interface member resolution with defaults and overrides; presets; function inlining and **∂** (Symbolica); **accumulators** with roles, transfers and closure; ports and connections (existing admission rules generalized); annotation collection; lineage; inspection of instantiated members and demand chains | Synthetic suites: lazy member pulls only its defining equations; dispatch of two implementations over one set yields two bodies; interface default versus override lineage; accumulator closure; value edit reuses structure (incremental equals clean) | `pse-runtime::workflow::{composition, balances, reactions, vessel}` and their tests; `model.rs::freeze` lowering responsibilities | [execution packet](21-modeling-kernel-k0-k3-execution.md#sequence-and-progress) |
| K4 Transformations | Continuous sets, `d(x)/d(s)`, integrals; **discretization** with schemes as data and `jacobi_roots` (Symbolica); **implicit blocks**: `inline` realization; `nested` evaluation stage with an `InnerSolver` capability (KINSOL implementation in `pse-backend-native`) and implicit-function derivatives (faer); regime selection; accelerator registry; elastic relaxation; continuation parameters | Synthetic: FD and collocation of a 1D ODE against an analytic solution at declared order; nested versus inline of the same implicit block agree in value and derivatives; regime selection picks the declared criterion; non-converged inner solve → typed trial failure | — | [execution packet](21-modeling-kernel-k4-k7-execution.md#sequence-and-progress) |
| K5 Lowering and evaluation | Specialized model → `pse-math` bodies per (definition specialization, dispatch group) through the existing typed path; external-function stage (generalized provider contract with index shapes and a declared derivative source); verified `piecewise`; term magnitudes for derived nominals | Existing math tests stay green; piecewise continuity proof rejects a C¹ claim on a C⁰ function; external synthetic vector function with declared derivatives | Nonprimitive `Function` handlers; live FeOS columns/factory and science consumers retire together in K8, as confirmed by the maintainer | [execution packet](21-modeling-kernel-k4-k7-execution.md#sequence-and-progress) |
| K6 Analyses and engines | Analysis modes as facts (steady; dynamic integrated through the existing Diffsol/IDAS route consuming generated `d/dt`; dynamic simultaneous); case bindings by symbol path; numerical sources (hints, derived nominals with IDAES scheme names); initialization engine consuming `@start`, `stage` and adaptive homotopy; post-solve `@check`s; generic diagnostics catalogue; study runner over case sets | Synthetic: same definition solves steady and dynamic; staged initialization restores the specification on failure; derived nominal matches hand computation; study isolates a failing point | Live `dynamic_cases` and transient-fitting consumers retire together in K8; replaced initializer/scaler refusals | [execution packet](21-modeling-kernel-k4-k7-execution.md#sequence-and-progress) |
| K7 Conformance harness | Test declarations (function-level and model-level); runner; **shared checks for every definition** (DoF, closure per accumulator, derivative consistency sample, envelope rejection, start-to-solve); `idaes-oracle:` linkage through the skill records; reports | Synthetic definitions gain the shared checks with no test code | Ad hoc fixture scaffolding superseded by test data | [execution packet](21-modeling-kernel-k4-k7-execution.md#sequence-and-progress) |
| K8 Seed knowledge | The science below plus existing knowledge required for retirement, authored per the [placement guide](21-knowledge-placement.md) | All seed conformance tests pass; **no model-specific production Rust or Python** (N1) | Production FeOS provider and FeOS/num-dual production dependencies (FeOS becomes reference-only) once seed PC-SAFT agrees with independent oracles; legacy scientific construction and dynamics | [execution packet](21-modeling-kernel-k8-execution.md) |
| K9 Integrated campaign and measurements | End-to-end scenarios on the seed (below); measurements R1–R3; knowledge boundary audit | Scenario acceptances; measurement report | — | proposed |

After K9, Plan 20's W1–W5 are executed as **knowledge-port packets**: package files plus
conformance tests, with any Rust change routed to *Kernel gaps*.

## Seed knowledge (K8)

The seed is the smallest set of real science that exercises **every kernel mechanism** through
the whole pipeline (symbolic lowering, structural analysis, every realization, every solver
class the seed needs, initialization, scaling, diagnostics, results), anchored to independent
reference values.

| Seed item | Content (knowledge only) | Mechanisms exercised | Reference values |
|---|---|---|---|
| Prelude math | `smooth_max`, `smooth_min`, `smooth_abs`, `smooth_min_over`, complementarity form, `cbrt`, `log10`, `tanh`, `sigmoid`, `safe_log`, `safe_sqrt`; verified `piecewise` valve characteristic | K5 polymorphic functions, verified piecewise, domain obligations | Analytic identities |
| Prelude interfaces | `MaterialState`, `PhaseThermo`, `IdealGasPhase`, `HelmholtzPhase` (identity defaults), `LiquidCp`/`IdealGasCp`/`VaporPressure`, `PhaseEquilibrium`, `ThermoPackage`, `RateLaw`, `DeltaT`, `PressureAssumption`, `UnitCosting`, `Controller` | K7 interfaces, defaults, slots | — |
| Chemistry | Kinds `element`, `species`, `phase`, `reaction`; benzene, toluene, the four saponification species, water; element composition | K2, K4, requirements (element closure) | — |
| Thermodynamic methods | RPP4 ideal-gas cp and vapour pressure; Perry liquid cp and density; constant-property forms | K5, K4 datasets, `@valid` | Source-publication values; IDAES property tests |
| Equations of state | Ideal (gas and Raoult liquid); **Peng–Robinson** α_r with density implicit block (`inline`, `nested`, `accelerated(cubic_roots)`) and the IDAES-δ override variant; **PC-SAFT** hard-chain + dispersion α_r (non-associating) as the scale probe | ∂ identities, implicit realization (all three), override lineage, expression-size measurement (R1) | IDAES `test_ceos_PR`, `test_BT_PR` oracles; FeOS (PC-SAFT); teqp |
| States and equilibrium | FTPx (and FPhx), log-fugacity equality, **SmoothVLE**, ideal bubble/dew, Rachford–Rice `@start`, `stage "ideal-K"`; a nested-flash realization with regime selection as the alternative phase-split realization | Lazy members, `when` over package facts, annotations, stages, regime selection (R2) | IDAES `test_BTIdeal`, `test_BTIdeal_FPhx`, `test_BT_PR` |
| Property packages | `BTIdeal`, `BT_PR`, `Saponification` (thermo + reactions) | Dispatch tables per component, binding definitions, scope defaults | As above; saponification tests |
| Control volumes | `ControlVolume0D` (componentPhase, componentTotal, elementTotal, total; enthalpyTotal; pressureTotal; holdup and accumulation under `analysis.dynamic`), `ControlVolume1D` | Accumulators (conservation, transfers), requirements, modes | Via the unit tests below |
| Units | Feed, Product, Heater, Flash, Mixer (indexed inlets, smooth-min pressure), Separator (indexed outlets, split-basis enum), HeatExchanger (ΔT slot: LMTD, AMTD, Underwood), PressureChanger (assumption slot; Pump, Compressor, Turbine presets), CSTR, PFR (backward FD and Lagrange–Radau) | Children, indexed children, slots, presets, discretization | IDAES `test_heater` (TestBT_Generic, TestSaponification), `test_flash` (TestBTIdealModular), `test_heat_exchanger` (TestBT_Generic_cocurrent), `test_mixer`/`test_separator` cases on seed packages, `test_pressure_changer` (TestSaponification), `test_cstr` (TestSaponification), `test_pfr` |
| Control | PID with signal ports, integral state, anti-windup via smooth function; dynamic CSTR temperature control | Modes (integrated and simultaneous), `require analysis.dynamic` | IDAES `models/control/tests/test_controller` |
| Costing | SSLW heat-exchanger correlation with type and material tables; CEPCI conversion; flowsheet capital accounting accumulator | Accounting accumulator, currency conversion, `@valid` | IDAES `test_SSLW` cases for the ported correlation |
| Numerical profiles | `idaes-2.13` diagnostics thresholds and solver defaults as data | Profiles | — |

**Out of the seed**, deliberately. They add no new mechanism beyond those above:

- activity-coefficient and eNRTL models (same ∂-default mechanism as Helmholtz, over gᴱ);
- multiparameter Helmholtz (same as PC-SAFT, plus nested pure-fluid inversion, which the
  PR-nested and regime tests already cover);
- electrolytes (sets, dispatch, requirements, inherent reactions: covered mechanisms);
- surrogates (functions + data);
- `models_extra`;
- applications.

If porting any of them later needs Rust, that is a kernel gap under N1.

## Integrated campaign (K9)

| Scenario (review ID) | Seed realization | Acceptance |
|---|---|---|
| S01 add a correlation | Add RPP5 cp as a function + dataset | Zero Rust; conformance passes |
| S02 add a unit | Flash from CV0D + package VLE | Zero Rust; IDAES flash oracles |
| S03 add an EoS | PC-SAFT α_r only | All properties from defaults; FeOS agreement |
| S05 steady → dynamic | CSTR + PID switched by analysis mode | Same definitions; dynamic closure |
| S06 distributed | PFR, FD versus collocation | Order behaviour; IDAES `test_pfr` values |
| S07 VLE formulation switch | BT_PR SmoothVLE ↔ complementarity ↔ nested flash | Same flowsheet solves; stability `@check` passes |
| S08 edit → re-solve, sweep | Flash feed-temperature sweep | Value-only reuse (measured) |
| S09 hard start | Flash–mixer recycle from default guesses | `@start` and `stage` converge; specification intact |
| S10 envelope | Perry cp outside its range | Typed envelope failure with lineage |
| S11 local tests | Any definition via K7 | No runtime or native startup for pure checks |
| S12 costing | HX costing on the heat exchanger | Accounting closure; SSLW oracle |
| S13 diagnostics | Over-specified flash; near-singular column case | Named structural refusal; SVD/near-parallel findings in model terms |
| S14 parity | All seed oracles | Agreement within IDAES test tolerances, same parameters and references |

**Measurements.**

- **R1.** Preparation time and memory of PC-SAFT with 2, 10 and 20 components, and of PR,
  under `inline` and `nested`.
- **R2.** Convergence rate of EO SmoothVLE versus nested flash over sampled BT_PR feeds.
- **R3.** Regime crossings inside NLP iterations with nested realizations.

## Kernel gaps

This table records any Rust change that knowledge porting required, with its generalization.
Empty at plan start.

| Gap | Discovered by (port) | Generalized mechanism | Synthetic test | Status |
|---|---|---|---|---|
| Specialized connections lost topology after equation expansion | Migrating tear selection and causal recycle | Retain declared port ownership and directed connection occurrences independently of equality rows; project explicitly selected nodes and tear policies through the existing structural graph | `typed_ports_connect_existing_coordinates_across_children` and `kernel_flow_projection_preserves_ports_isolates_and_explicit_tear_policies` and `authored_causal_recycle_retains_topology_and_refuses_hidden_inputs` | implemented; topology and native causal-map controls pass |
| Authored cases had elastic penalties but no general objective | Migrating the heater optimization consumer | A typed objective annotation selects one existing scalar member and native objective sense; explicit normalization precedes combination with elastic penalties | `kernel_objective_selection_preserves_values_derivatives_and_penalty_direction` and `kernel_dimensional_objectives_require_normalization_before_elastic_combination` | implemented; targeted controls pass |
| Joined authored execution has only a single-case result and native sequence acceptance omits package checks | Retiring public multi-case execution | Keep native allocation reuse and start selection in the existing sequence owner; evaluate prepared original-model checks before admitting a predecessor; carry explicit step identity independently of trajectory sample indices | Positive/negative roots, original-check rejection, independent continuation, cancellation/join and repeated-root publication | implemented; synthetic reuse/seed/original-check controls and migrated public sequence tests pass |
| Function-valued interface parameters were treated as numerical coordinates | Explicit caloric primitive defaults | Resolve function bindings through effective instance members, including overrides and indexed selection; retain cycle and signature checks | `kernel_explicit_primitive_functions_preserve_references_and_derivative_checks` and function-slot controls | implemented; targeted controls pass |
| Power inference dropped the exponent quantity | Prelude cube-root and polynomial functions | Concrete expression and polymorphic scheme inference pass both physical operands to the existing quantity operation, alongside the exact exponent fact | `powers_keep_the_exponent_quantity_in_the_authoritative_operation` | implemented; targeted type controls pass |
| Modeling context omitted operation prerequisites | Caloric primitive derivative identity | Retain admitted physical prerequisites through checking, specialization and typed differentiation; recheck on prerequisite edits and refuse foreign revisions | `kernel_physical_prerequisites_survive_admission_and_context_republication` and nitrogen Shomate derivative fixture | implemented; targeted controls pass |
| Imported tables failed indexed expression lookup | Separate property methods and sourced datasets | Resolve the visible qualified declaration before traversing indices and row fields; preserve lexical precedence and import admission | `kernel_imported_table_paths_preserve_rows_columns_and_visibility` | implemented; targeted control passes |
| Predicate-only function calls were omitted from dependency closure | Caloric validity limits | Traverse predicate expressions, including calls in validity, static filters and conditional equations, through the ordinary syntax walker | `kernel_function_validity_guards_survive_simplification_and_partial_derivatives` | implemented; targeted control passes |
| Pure functions could not consume admitted immutable package data | EOS parameter tables | Admit visible immutable package entities/sets/enumerations/tables while retaining explicit runtime arguments and refusing local captures; retain data in dependency closure | `kernel_immutable_function_data_is_visible_differentiable_and_invalidated` and capture refusal controls | implemented; targeted controls pass |
| Structural interface parameters entered the numerical coordinate path | Helmholtz component sets and strategy bindings | Resolve effective structural parameter values before guards and memberships with finite dependency and type checks | `kernel_structural_interface_parameters_bind_effective_members_before_guards` | implemented; targeted control passes |
| Indexed defaults could not resolve inherited sibling domains | Helmholtz fugacity defaults | Validate against the complete effective interface and rebind inherited domain slots to their explicit overrides; refuse unrelated membership substitutions | Extended structural-interface control | implemented; targeted control passes |
| Child index expressions changed lexical owner during traversal | Species-indexed property fixtures | Preserve the author's scope for actual coordinates while typing declared memberships in the target member's scope, including structural header parameters | `kernel_child_indices_resolve_in_the_authors_import_scope` and integrated-axis control | implemented; targeted controls pass |
| Implicit scopes lost enclosing indexed and structural arguments | Shared PR density residual | Resolve whole indexed arguments and immutable structural context through enclosing implicit scopes, preserving local shadowing | `kernel_implicit_functions_read_enclosing_indexed_members` | implemented; targeted control passes |
| Symbolic partials flattened shared expression bindings | PC-SAFT potential | Reify shared bindings as Symbolica function calls, differentiate with Symbolica and bind their partials to bounded shared outputs; retain original guards | `kernel_shared_partials_preserve_lets_and_mixed_derivatives` and existing partial/guard controls | implemented; targeted controls and all five frozen teqp state comparisons pass |
| Negation dropped the expected physical literal type | Negative residual enthalpy oracle | Carry the admitted expected physical contract through unary negation into literal lowering | `kernel_negative_literals_keep_the_expected_physical_contract` | implemented; targeted control passes |
| Conformance ignored the requested Hessian preparation order | Bounded inline density residual | Prepare exact-Hessian steady/initialized attempts with second derivatives, retaining the declared native policy | `kernel_conformance_prepares_the_requested_exact_hessian` | implemented; seven targeted native conformance controls pass |
| Generic normalization could not express a neutral quotient | Affine-temperature smoothing | Constrain a generic equal-type quotient to the neutral type while retaining concrete physical-operation admission on instantiation | `kernel_generic_normalization_requires_the_concrete_physical_operation` | implemented; supported and absent-operation controls pass |
| Anonymous inherited contracts collided by local ordinal | Caloric state composition starts | Name anonymous declarations by their existing semantic identity; preserve annotations, expectations and requirements across inherited and guarded scopes | `kernel_anonymous_contracts_compose_across_inherited_and_guarded_scopes` and language round trips | implemented; 42 modeling controls and six parser controls pass |
| Difference normalization was tied to one reference datum | Explicit BT oracle-to-stock enthalpy conversion | Declare same-reference difference families; check common actual reference, difference role and all other physical axes against an admitted prototype; no rebasing is inferred | `same_reference_differences_normalize_without_rebasing` and caloric quantity controls | implemented; same-datum alternatives pass; mixed datums, points and component subjects refuse |
| Whole indexed child members could not receive parent annotations | Nested flash input binding | Resolve the declared child interface for typing, expand actual child membership, and keep annotation arguments in the author's lexical scope | `kernel_child_family_annotations_use_target_membership_and_author_scope` | implemented; 43 modeling controls pass |
| Initialization searched only directly declared stages and applied final fixture oracles to intermediate specifications | Inherited ideal-K stage | Retain effective stage names on specialized instances; intermediate solves keep model obligations, while a separate original-specification solve applies final expectations | `kernel_conformance_initializes_inherited_child_stages` and initialization controls | implemented; targeted controls pass |
| Detailed report caps could erase discovered fixture identities; diagnostic and oracle sizes exceeded fixed row allowances | Expanded seed conformance | Separate complete fixture dispositions from capped detail rows; admit actual variable payloads before retention and refuse incomplete diagnostics explicitly | `kernel_conformance_retains_inventory_and_accounts_variable_payloads`, `kernel_conformance_memory_refusal_cannot_erase_or_pass_a_fixture` | implemented; Rust controls and native Python projection tests pass |
| Finite sums lost their physical contraction when expanded into additions | Component flow and concentration totals | Retain the consumed entity kind and independently checked prototype through scalar enumeration, including empty/filtered sets and static values | `kernel_finite_reductions_retain_domains_prototypes_and_derivatives` | implemented; targeted controls pass |
| Kind-only operation dispatch could not distinguish admitted pressure or concentration contracts | Physical smoothing and dilute-liquid coordinates | Select exactly one declaration whose complete prerequisites hold; refuse overlaps, mismatched operands and unproved contracts | `actual_contracts_select_disjoint_rules_and_reject_overlap_or_cross_normalization` | implemented; targeted controls pass |
| Indexed smoothing lacked a generic finite composition operator | `smooth_min_over` and Mixer inlets | Add a bounded, nonempty, type-preserving lexical fold; authored step functions retain library differentiation and explicit order | `kernel_finite_folds_compose_source_functions_and_library_partials` and parser scope/round-trip control | implemented; targeted and authored smoothing checks pass |
| Selected definitions could not project structural members | Unit component membership and state configuration | Share constructor/default/effective-member binding between instantiation and structural projection; preserve lexical isolation and refuse cycles | `kernel_definition_projection_shares_constructor_defaults_and_effective_members` | implemented; targeted checks pass |
| Fixture paths rechecked an abstract child interface after resolving its concrete implementation | Saponification inlet specifications through a generic unit | Use the resolved member's physical contract for concrete case bindings; retain interface checking for reusable source equations | `kernel_fixture_paths_bind_the_selected_implementation_physical_contract` | implemented; targeted checks pass |
| Presolve discarded a typed callback trial failure when a wrapper aborted | PR liquid-branch start | Retain the latest failed callback cause and return it through presolve; clear it after recovery | `kernel_presolve_retains_the_typed_failed_trial_witness` and `recovered_trials_do_not_poison_success_but_panics_do` | implemented; two targeted controls pass |
| Conservation residual lowering joined independent property instances into one derivative body | PR Heater | Assemble signed original contributions with the established sparse case assembler; incidence takes the union of actual term support | `kernel_conservation_assembles_local_derivatives`, `kernel_compiled_original_terms_feed_independent_closure` | implemented; targeted controls pass; selected PR Heater fixture passes |
| A constructed child could not expose a stronger inherited interface | Flash outlet and reactive state contracts | Permit nominal child-interface refinement; preserve inherited members, indexed domains and invariant input/physical contracts | `kernel_child_contract_refinement_preserves_inherited_members` | implemented; targeted positive and refusal controls pass |
| Derivative-work refusals hid the requested extent | PR Heater | Retain typed source, required operations, remaining allowance and Taylor width at the existing admission boundary | `derivative_work_refusal_retains_required_and_available_operations` | implemented; targeted math and boundary projection controls pass |
| Accounting totals could not be referenced as indexed values or annotated families | Separator partition checks and forthcoming capital accounting | Resolve accounting coordinates through the common expression and annotation paths; materialize the existing signed sum after contribution collection | `kernel_indexed_accounting_values_are_readable_in_checks_and_ports` | implemented; targeted control passes |
| Arithmetic blocks expanded derivatives over unrelated body coordinates | Coupled PR unit preparation exceeded the declared work allowance | Select reachable Taylor coordinates per block; keep Symbolica differentiation and full assembly coordinates | `local_taylor_coordinates_preserve_transitive_and_permuted_derivatives`, derivative/guard suite | implemented; targeted controls pass |
| Package-qualified enum values were treated as runtime package traversal | Unit strategy constructor arguments failed specialization | Resolve the complete enumeration prefix through admitted visibility | `kernel_qualified_enumeration_arguments_select_structure` | implemented; targeted control passes |
| Pure function free names captured caller members and function-slot bindings | CSTR reaction functions | Resolve function bodies and nested bindings in the defining lexical scope; retain explicit arguments | `kernel_pure_function_names_do_not_capture_caller_members` and related function controls | implemented; targeted checks pass |
| A diamond exposed an ancestor member beside its checked descendant refinement | Entropy-capable PR state composition | Select the more specific declaration owner independently of base order; retain sibling-conflict refusal | `kernel_diamond_keeps_the_most_specific_checked_member` | implemented; targeted checks pass |
| Automatic presolve could retain constant nonlinear rows after propagation fixed their variables | PR Heater | Validate reduced structure; decline invalid optional reductions while preserving original coordinates and warm seeds; required reductions still refuse | `automatic_presolve_retains_original_when_propagation_leaves_constant_nonlinear_rows`; presolve controls and selected PR Heater fixture | implemented; targeted checks pass |
| Python fixed specialization limits at defaults despite Rust's explicit policy | PR heat exchanger expansion | Immutable package views carry the selected limits through analysis, conformance and CLI; preserve independent memory budgets | `test_modeling_expansion_limits_are_explicit_and_isolated` | implemented; targeted boundary check passes |
| Preset values lost their bound definition contract in nested child arguments | Pump/Compressor/Turbine presets | Use one structural conformance predicate for direct definitions and preset chains | `presets_preserve_the_bound_definition_contract` | implemented; targeted and pressure-preset fixtures pass |
| Numeric mesh coordinates did not resolve indexed children consistently | PFR boundaries and dynamic child inventories | Normalize all child traversals against declared membership; classify initial conditions across the whole path | `continuous_child_literals_resolve_the_admitted_coordinate`, `kernel_integrated_axis_retains_symbolic_time_and_original_derivative_lineage` | implemented; targeted controls pass |
| Library exp/log normalization discarded nested exponential arguments | SSLW correlation | Materialize the exponent through the existing typed evaluator schedule before applying Symbolica exp; retain library-owned values and jets | `exponential_of_nested_logarithms_preserves_values_and_derivatives` | implemented; targeted math and both costing fixtures pass |
| Inherited conditional fields lacked source contracts outside their declaring guard | Optional CV holdup and dynamic indexed CSTR cells | Resolve unique guarded declarations for typing without activating them; selected specialization still refuses inactive demanded members | `kernel_inherited_guarded_members_have_contracts_without_early_activation` | implemented; positive, inactive-demand and ambiguous-contract controls pass |
| Multiplicative operands inherited the result's physical type | Dynamic inventory initial conditions | Match typed lowering: additive and exact-integer contexts propagate expectations; multiplication, division and powers resolve their own operands in checking and static evaluation | `multiplicative_literals_keep_operand_units_in_typed_and_static_expressions` | implemented; product, quotient, power, constructor and incompatible-literal controls pass |
| Dynamic range rejection discarded the offending value and member | Controlled CSTR initialization | Carry the annotation, specialized member, value and bounds in a typed recoverable trial error through callback and diagnostic projections | `kernel_integrated_dae_checks_partition_and_retains_completed_samples_on_failure` | implemented; dynamic guard and retained diagnostic controls pass |
| Terminal quadratures unnecessarily prohibited state/output sensitivities | Vessel fitting and conservation checks | Admit native state/output sensitivities alongside quadratures; keep terminal integral expressions outside the sensitivity output contract | `kernel_integrated_definite_integrals_are_terminal_native_quadratures` | implemented; value, derivative and noncausal-refusal controls pass with Diffsol and IDAS |
| Value aliases consumed fresh slots at every function and local boundary | Vessel property composition | Reuse constant and existing-slot representations; explicit partials and continuity checks still bind independent arguments | `shared_aliases_reuse_slots_but_partial_arguments_remain_independent`, `kernel_partial_distinguishes_equal_argument_values_through_aliases` | implemented; alias, partial, guarded and piecewise controls pass |
| Optional native presolve tapes could exhaust their budget during ordinary preparation | Simultaneous CSTR with presolve disabled | Retain opaque tapes and unestablished obligations when optional projection work is exhausted; keep original equations, independent affine facts and typed fatal/cancellation errors | `exhausted_optional_presolve_tapes_preserve_original_evaluation_and_independent_facts`, `coefficient_projection_preserves_erased_domain_obligations` | implemented; positive, bounded, guard, missing-value and cancellation controls pass; simultaneous CSTR passes |
| Shared blocks acquired dense Hessian support when optional flattening stopped | Integrated vessel rate elimination | Compose library-derived local support through shared blocks; retain provider/branch conservatism and original guards | `shared_nonlinear_coefficients_preserve_affine_rate_support` | implemented; affine/nonlinear-rate, mixed-support and guard controls pass; integrated vessel passes |
| Derivative diagnostic work limits were classified as malformed models | Simultaneous CSTR | Preserve typed resource refusal independently of invalid point/policy contracts | `derivative_sample_budget_refusal_is_not_an_invalid_model` | implemented; static and dynamic derivative controls pass; CSTR passes with an explicit eight-million-cell budget |
| Mathematical construction allowance was inaccessible to authored analyses | Simultaneous vessel quadrature | Expose an optional per-body occurrence policy through immutable modeling limits; retain the backend slot bound and policy-sensitive cache admission | `body_construction_allowance_can_be_explicitly_larger_than_default`, `kernel_body_construction_limits_are_tracked_without_changing_mathematics`, `test_modeling_expansion_limits_are_explicit_and_isolated` | implemented; math/compiler/Python controls pass; larger meshes can still exceed the explicit slot limit |
| Fitting interpreted legacy model/output identities and lost authored case obligations | Vessel and public fitting migration | Bind shared parameters and observations by source path, reuse sparse fitting and native trajectories, retain original physical checks and fixed/bound metadata; scope fixture controls by experiment instance | `authored_fit_retains_fixed_case_values_and_physical_bounds`, `authored_fit_checks_can_reject_a_numerically_feasible_candidate`, `authored_integration_controls_bind_to_the_experiment_instance`, mixed/event fitting controls | implemented; focused Rust and public Python controls pass; transient vessel acceptance in progress |
| A single conformance solver/derivative policy could not execute the complete mixed seed | Root/optimization fixtures and near-back-pressure vessel derivatives | Explicit fixture-ID policy map over the same harness; validate all IDs/settings, retain sampled derivative conditions | `kernel_conformance_mixes_explicit_fixture_solver_and_derivative_policies`, `kernel_conformance_refuses_unused_or_invalid_fixture_policy` | implemented; Rust and native Python controls pass |
| Port equality expansion omitted declared direction and incidence constraints | Physical connections versus signal fanout | Generic authored incoming/outgoing maxima, checked against original port identities and connection occurrences before lowering; no science enum | `kernel_connectivity_limits_keep_direction_multiplicity_and_indexed_port_identity`, typed-port and causal recycle controls | implemented; 63 selected modeling/compiler/native controls pass |
| Coordinate-aware child traversal rejected implicit block instances | Complete BT ideal/PR seed run | Resolve both child declarations and implicit scopes through their existing instance membership; retain index-arity and membership refusal | `implicit_members_remain_addressable_through_child_paths_and_indexed_arguments` | implemented; synthetic scalar/indexed/nested paths, annotations, whole-family arguments and invalid-path controls pass |

## Finding dispositions

### Through-K8 corrective review

The [through-K8 review](../design_review/reviews/design_review_modeling-kernel-through-k8_2026-09-26.md)
records the source observations. The [execution sequence](21-modeling-kernel-k8-execution.md)
links reopened kernel work and seed/retirement dependencies.

| Finding reference | Scenario reference | Disposition | Decision / work owner | Evidence or revisit trigger |
|---|---|---|---|---|
| Through-K8 F01 package closure/visibility | S01 | resolved | K1/K3; E1 | Tested: explicit closure admission, `package_visibility_requires_an_import_for_functions_and_types` and scoped physical aliases in the full native assessment |
| Through-K8 F02 mutable checked products | S11 | resolved | K2; E1 | Tested: admitted-context lifetime and compiler context substitution refusal controls, with force validation |
| Through-K8 F03 repeated admission | S08 | resolved | K3; E1 | Tested: immutable revision reuse and incremental/clean kernel controls, with force validation |
| Through-K8 F04 fixture execution | S05, S11 | resolved | K6/K7; E2 | Tested: `kernel_conformance_mixes_explicit_fixture_solver_and_derivative_policies`, inherited initialization and dynamic conformance controls in the full native assessment; Python CLI policy controls |
| Through-K8 F05 diagnostics/pure dispatch | PSE-S04, S11 | resolved | K6/K7; E2 | Tested: pure authored fixtures without runtime/solver, typed expected failures, complete fixture inventory and native Python ownership controls |
| Through-K8 F06 schemes/capabilities | S06, S04 | resolved | K4/K5; E3 | Tested: `continuous_schemes_are_visible_data_and_lattice_offsets_cross_elements` and `accelerators_use_only_registered_capabilities_and_recognized_forms` in the full native assessment |
| Through-K8 F07 derivative/integral claims | S03, S04 | resolved | K4/K5; E3/E5 | Tested: `kernel_regime_derivatives_are_regular_local_branch_jets` and `kernel_explicit_primitive_functions_preserve_references_and_derivative_checks`; branch crossings and unsupported integral sensitivities remain explicit refusals |
| Through-K8 F08 retirement dependencies | S04, S05 | resolved | K8; E4/E8/E9 | Tested: authored vessel/fitting replacements in the native assessment and selected vessel conformance; production FeOS, legacy scientific relations/builders and their callers removed |
| Through-K8 F09 reference provenance | S14 | resolved | K8; E4/E5 | Tested: all 77 seed fixtures have passing evidence in the K8 packet; sourced data, demonstration fits and IDAES/teqp oracle inputs retain distinct provenance, references and tolerances |

This plan becomes the disposition owner of the Plan 20 review findings that concern the
kernel. [Plan 20's table](20-idaes-capability-target.md#finding-dispositions) links here, so
the status has one owner.

| Finding reference | Scenario reference | Disposition | Decision / work owner | Evidence or revisit trigger |
|---|---|---|---|---|
| [F01](../design_review/reviews/design_review_idaes-capability-target_2026-09-26.md#f01) provider fuses model and algorithm | S03, S04 | open | R1, R2; K4, K5, K8 | — |
| [F02](../design_review/reviews/design_review_idaes-capability-target_2026-09-26.md#f02) no demand derivation | S01, S02 | open | R1; K3 (lazy demand, interfaces) | — |
| [F03](../design_review/reviews/design_review_idaes-capability-target_2026-09-26.md#f03) method data never executes | S01 | open | R1, R3; K1 (families deleted), K8 | — |
| [F04](../design_review/reviews/design_review_idaes-capability-target_2026-09-26.md#f04) scalar-only laws | S02, S12 | open | R4; K3 | — |
| [F05](../design_review/reviews/design_review_idaes-capability-target_2026-09-26.md#f05) no formulation primitives | S07 | open | R5; K4, K5, K8 | — |
| [F06](../design_review/reviews/design_review_idaes-capability-target_2026-09-26.md#f06) composition cannot express the library | S02, S06 | open | R3; K3 | — |
| [F07](../design_review/reviews/design_review_idaes-capability-target_2026-09-26.md#f07) semantics in the runtime | S08, S11 | open | R2; K3 | — |
| [F08](../design_review/reviews/design_review_idaes-capability-target_2026-09-26.md#f08) steady and dynamic authored apart | S05 | open | R6; K6 | — |
| [F09](../design_review/reviews/design_review_idaes-capability-target_2026-09-26.md#f09) no continuous domains | S06 | open | K4 | — |
| [F10](../design_review/reviews/design_review_idaes-capability-target_2026-09-26.md#f10) no initialization knowledge | S09 | open | R6; K6 | — |
| [F11](../design_review/reviews/design_review_idaes-capability-target_2026-09-26.md#f11) no derived nominals | S08 | open | R6; K5, K6 | — |
| [F12](../design_review/reviews/design_review_idaes-capability-target_2026-09-26.md#f12) diagnostics incomplete | S13 | open | K6 | — |
| [F13](../design_review/reviews/design_review_idaes-capability-target_2026-09-26.md#f13) FeOS cannot own the scope | S04 | open | R1; K8 | — |

### K0–K3 target review

The [execution packet](21-modeling-kernel-k0-k3-execution.md#verification) owns the targeted
commands, conditions and packet progress; this table owns adopted finding dispositions.

| Finding reference | Scenario reference | Disposition | Decision / work owner | Evidence or revisit trigger |
|---|---|---|---|---|
| [K0 F01](../design_review/reviews/design_review_modeling-kernel-k0-k3_2026-09-26.md#f01) premature deletion | S03 | resolved | K0; staged K5/K6/K8 retirement | Retirement boundary adopted; authoring graph dependency removed; later consumers retain explicit owners |
| [K0 F02](../design_review/reviews/design_review_modeling-kernel-k0-k3_2026-09-26.md#f02) defaults and unsupported DoF claim | S02 | resolved | K2/K3 | Effective-default demand tests and post-specialization library matching; no numerical-rank claim |
| [K0 F03](../design_review/reviews/design_review_modeling-kernel-k0-k3_2026-09-26.md#f03) closed physical kinds | S01 | resolved | K2 | Generic `EntityKindId` admission and axis/subject controls in packet Verification |
| [K0 F04](../design_review/reviews/design_review_modeling-kernel-k0-k3_2026-09-26.md#f04) unqualified value reuse | S04 | resolved | K3 | Separate initial values, dispatch bodies and structural bindings; Salsa events, shared admission and incremental/clean controls |
| [K0 F05](../design_review/reviews/design_review_modeling-kernel-k0-k3_2026-09-26.md#f05) frontend physical dependencies | S05 | resolved | K1 | Authoring uses petgraph directly; its targeted source tests build and pass without pse-structural |

### K4–K7 target review

The [execution packet](21-modeling-kernel-k4-k7-execution.md#verification) owns executed
commands and their conditions. These dispositions address the six scoped kernel corrections;
they do not close the scientific IDAES scenarios above.

| Finding reference | Scenario reference | Disposition | Decision / work owner | Evidence or revisit trigger |
|---|---|---|---|---|
| [K4 F01](../design_review/reviews/design_review_modeling-kernel-k4-k7_2026-09-26.md#f01) continuous intervals cannot be enumerated | S05, S06 | resolved | K4 | Finite/collocation transformation tests and generated integrated-axis lineage controls |
| [K4 F02](../design_review/reviews/design_review_modeling-kernel-k4-k7_2026-09-26.md#f02) KINSOL signs are not boxes | S03, S04 | resolved | K4 | Native nested-stage interval, original-residual, hint and derivative controls |
| [K4 F03](../design_review/reviews/design_review_modeling-kernel-k4-k7_2026-09-26.md#f03) branch smoothness does not prove selector smoothness | S04 | resolved | K4/K5 | Authored regime execution/tie tests; unproved selector derivatives refuse; piecewise boundary-jet controls |
| [K4 F04](../design_review/reviews/design_review_modeling-kernel-k4-k7_2026-09-26.md#f04) normalization erases original terms | S08, S13 | resolved | K5/K6 | Original cancellation, derived-nominal, independent closure and term diagnostic controls |
| [K4 F05](../design_review/reviews/design_review_modeling-kernel-k4-k7_2026-09-26.md#f05) shared checks need concrete fixtures | S11 | resolved | K7 | Authored physical fixtures, uncovered-definition reports, pure fixture values and owned Python conformance tables |
| [K4 F06](../design_review/reviews/design_review_modeling-kernel-k4-k7_2026-09-26.md#f06) local failure is not infeasibility proof | S13 | resolved | K6 | Bounded local elastic explanation, inconclusive stop controls, separate native outcomes and continuous-relaxation IIS scope |

The broader Plan 20 findings retain their original scenario acceptance: the excluded K9
convergence, formulation-switch, scaling and diagnostics campaigns are not certified by
K8 seed checks. K0–K7 controls establish the exercised kernel mechanisms; K8 establishes
the named seed and replacement behavior, not the full IDAES capability target. The live
FeOS/vessel and legacy transient-fitting consumers have been replaced and deleted; the
K8 packet owns that retirement evidence.

## Verification

Targeted checks accompany each packet (AGENTS.md *Execution rhythm*):

- `just check-package`;
- `just unit-package` over the synthetic suites;
- `just codegen` when the registry IR changes.

Seed conformance runs through the K7 harness. Reference comparisons use the same parameters,
reference states and tolerances as the IDAES tests they cite. Independent thermodynamic
references (FeOS, teqp, NIST, IAPWS release values) are used where IDAES has none (PS-13).
Comprehensive qualification is separately requested by the maintainer.

## Open items

- The grammar and single compiler-owned Salsa workspace are implemented; the execution
  packets own their evidence and approved refinements. Companion examples remain sketches
  until replaced by executable package examples.
- The first accelerator set beyond `cubic_roots`: decided by the R2/R3 measurements, not
  up front.
- The storage format for large datasets (inline, CSV or Arrow/Parquet through the data layer),
  decided with the first dataset larger than the seed.

## Outcome (recorded after implementation)

### What was built

### A mistake made and corrected

### Deviations from the plan, deliberate
