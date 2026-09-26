# Thermodynamics actions, information flows and lifecycle blueprint

**Document:** THERMO-ACTIONS-007  
**Version:** 0.1  
**Date:** 25 September 2026  
**Project:** Process simulator  
**Stage:** Step 7, define actions, information flows and lifecycle behavior  
**Status:** Proposed language-independent behavioral blueprint. Catalog checks and a small authored sequential reference model executed; no simulator/provider conformance or independent scenario R/E/V pass.

## 0. Decision and use

The central behavioral boundary is **capture → prepare → execute privately → qualify → publish under a live guard**. Preparation produces proposals, not hidden live edits. Calculations produce candidates, not automatically accepted process states. Accepted local results may support a coupled run without becoming accepted whole-flowsheet results. A current-result update is a separate, revision/run/scope-checked group decision.

This blueprint defines **56 action contracts**, **12 operation specializations**, **18 complete workflow routes**, **8 orthogonal lifecycle views**, **18 handoff facts** and **18 common action rules**. All 94 requirements have exactly one primary action owner consistent with Step 5. All 72 information concepts, 18 information products and 34 scenarios retain their meaning and trace. The 112 action witnesses are future acceptance descriptions, not claims of completed simulator tests.

**Evidence precedence:** B1 controls scope, B4 required behavior, B5 responsibility ownership and B6 information semantics. B2/B3 retain the original pinned research evidence and limitations. This is a transformation of supplied baselines, not refreshed library research or a backend recommendation. The existing 65 H,25 C,four R3 obligations and 14 P1,16 P2,four P3 scenarios are unchanged.

The JSON companion manages these contracts; it is not a runtime JSON schema. Action IDs are not mandatory RPC endpoints or services. The lifecycle labels are observable conceptual views, not a required enumeration. No database, message bus, numerical solver, storage schema or Rust framework is selected.

Read §§1–4 for common behavior and operation promises; §5 for detailed action cards; §§6–8 for lifecycles, handoffs and end-to-end routes; §§9–12 for race/replay decisions, traceability and completion. Relative source links work when the bundle is extracted intact.

**Sources:** [Functional requirements](thermodynamics_functional_requirements_v0_1.md), [responsibility architecture](thermodynamics_conceptual_packaging_v0_1.md), [semantic dictionary](thermodynamics_semantic_information_dictionary_v0_1.md). Primary bases: B4 common unsuccessful outcomes, FR-STA-03, FR-RUN-01..08, FR-RES-01..08 and FR-LIF-01..08; B5 §§5–9; B6 IC-34..42,IC-52..72 and SI-17..28.

## 1. Common action contract and effect boundaries

### 1.1 What an invocation must establish

| Interaction information | Meaning |
| --- | --- |
| Intent and identity | Action type/version plus invocation or command identity and initiating authority. Exact command replay is different from an intentional repeated calculation. |
| Captured subject/context | Material/account/location, immutable definition/input revisions and a relevant dependency set or safe conservative snapshot. Native session identity is not enough. |
| Operation or proposal | Original physical problem and allowed-change contract for numerical work; expected prior model and full proposed changes for edits. |
| Run and scope | Run/ancestor permission, branch/location, completion/publication kind and coherent group where applicable. A fit trial need not have production permission. |
| Input/output roles | Read-only source assertions, private working quantities, newly produced candidate/proposal and required/optional outputs. Every output uses B6 meaning. |
| Completion promise | Action-specific success boundary, required checks, failure categories and postconditions; producing values is not automatic acceptance. |
| Repeat/recovery policy | Whether equivalent evidence may be reused; whether a fresh attempt is required; how duplicates, unknown outcomes, conflicts and cancellation are reconciled. |
| Decision evidence | Retained outcome/attempt/change/publication evidence in IC-54/63/65/66/70 as appropriate. No transport-layer implementation is prescribed. |

A control envelope carrying these references is an interaction convention over the existing dictionary, not a new competing material or result model. An action may return a draft, blocked readiness or scoped diagnostic successfully as a *response* without having successfully resolved the underlying physical problem. All scientific claims use their actual outcome and evidence, not transport success.

### 1.2 Five effect classes

| Effect | Allowed mutation | Adoption boundary |
| --- | --- | --- |
| Author/prepare | Private proposal or new attributable definition/evidence record | Only coordinated domain-checked change AC-48 becomes the current model. |
| Derive/evaluate | New candidate values or private numerical buffers from fixed inputs | No direct current writes; mandatory checks and scoped qualification first. |
| Coordinate/iterate | Run-local trial views and child requests | Values may feed iteration but do not certify a parent unit/flowsheet. |
| Assess | New evidence/assessment record, with original target/candidate retained | P09 qualification does not override P10 currentness. |
| Commit/restore | Only named revision, run-authority or current-binding transitions | Expected context, live permission, scope and group coherence checked at the logical write point. |

An explicitly incomplete authored model may be saved or committed for further editing while its calculations remain blocked. A successfully committed feed edit is not automatically rolled back because a subsequent solve fails. Its old outputs become stale; silently restoring the prior input would erase author intent.

### 1.3 Shared numerical spine

```text
AC-19/20: bind location and expose unit demands
AC-13 + AC-21..26: capture state and original physical/contribution problem
AC-45: issue captured run authority (before numerical work)
AC-09 + AC-27/28: assess exact readiness, realization and coordinate meaning
AC-29/30/31: plan, acquire safe workspace, initialize and retain restoration duties
AC-32/33: execute and detach actual candidate evidence
AC-36 <-> AC-55: check original obligations, acquire bounded missing evidence
AC-37: qualify the attained local/unit/flowsheet scope
AC-44: optionally use compatible local results in a continuing coupled run
AC-49: publish only a qualified coherent group under current revision/run/scope guards
```

The ordering is a dependency outline rather than a mandatory universal call chain. Run capture and request construction may be coordinated in one command. Readiness has separate preparation and dispatch stages so the system does not need an initialized state merely to ask an initializer for one. A cohesive backend may combine many numerical steps; an equation-oriented realization supplies contributions instead of independent flashes. None may omit the observable authority, postcondition or publication boundaries.

## 2. Cross-action behavioral rules

<a id="ar-01"></a>
### AR-01. One captured context per action

All scientifically relevant input references and value roles are captured together. Read a fixed snapshot, validate a dependency capture, or fail/retry capture; never interleave current aliases from different revisions. An action may use a deliberately provisional context with no production authority.

**Primary actions:** [AC-13](#ac-13), [AC-21](#ac-21), [AC-29](#ac-29), [AC-45](#ac-45), [AC-50](#ac-50).

<a id="ar-02"></a>
### AR-02. Proposal, calculation, assessment and commit are different effects

Preparation returns proposals; execution changes only private workspace and records candidates; assessment adds evidence; P10 alone changes current definition/result bindings. Producing a P04-shaped payload does not transfer ownership of P04 semantics to P08.

**Primary actions:** [AC-06](#ac-06), [AC-32](#ac-32), [AC-33](#ac-33), [AC-36](#ac-36), [AC-37](#ac-37), [AC-48](#ac-48), [AC-49](#ac-49).

<a id="ar-03"></a>
### AR-03. Authority is narrower than capability

An engine able to equilibrate must not do so in a property-only request that preserves allocation. Permitted density/root resolution, phase redistribution, chemical transformation, representation mapping and current publication are separate permissions.

**Primary actions:** [AC-21](#ac-21), [AC-22](#ac-22), [AC-23](#ac-23), [AC-24](#ac-24), [AC-32](#ac-32), [AC-36](#ac-36).

<a id="ar-04"></a>
### AR-04. Original targets and conditional demands survive execution

Keep original IC-35/36/37/41. Resolve each conditional check from explicit evidence after branch/phase discovery; an unknown mandatory condition is not silently false. Do not relax a tolerance or redefine a target retrospectively to claim a pass.

**Primary actions:** [AC-21](#ac-21), [AC-23](#ac-23), [AC-31](#ac-31), [AC-36](#ac-36), [AC-37](#ac-37).

<a id="ar-05"></a>
### AR-05. Retry is not replay of a physical effect

A new numerical attempt is evaluated from fixed inputs and has a new attempt identity. Re-delivery of an identified command is not a new command. Incremental material edits and modeled transfers must be applied once per explicit intent, not once per solver iteration, retry or callback.

**Primary actions:** [AC-11](#ac-11), [AC-15](#ac-15), [AC-18](#ac-18), [AC-32](#ac-32), [AC-34](#ac-34), [AC-39](#ac-39), [AC-48](#ac-48), [AC-49](#ac-49).

<a id="ar-06"></a>
### AR-06. Duplicate command and lost-response treatment

For a mutating command, identify intent, expected context and exact payload. Same identity+same payload returns the recorded pending/completed decision; changed payload is a conflict. An unknown outcome must be reconciled before reapplying effects. A prior committed publication replay never rewrites the old result over a newer one. This is duplicate-safe effect semantics, not a guarantee of exactly-once message delivery.

**Primary actions:** [AC-45](#ac-45), [AC-46](#ac-46), [AC-48](#ac-48), [AC-49](#ac-49), [AC-52](#ac-52).

<a id="ar-07"></a>
### AR-07. Numerical failure does not establish infeasibility

Distinguish missing data/dependency, unsupported operation, incomplete specification, initializer failure, native failure, nonconvergence, demonstrated infeasibility and failed postconditions. Preserve scope and known cause, including unknown native cause.

**Primary actions:** [AC-09](#ac-09), [AC-21](#ac-21), [AC-31](#ac-31), [AC-33](#ac-33), [AC-35](#ac-35), [AC-36](#ac-36), [AC-37](#ac-37).

<a id="ar-08"></a>
### AR-08. Private mutation has a bounded lifetime

Lease the full activation/input/evaluation/readback sequence under an appropriate policy. Native cancellation need not be instantaneous. A still-running or uncertain context cannot be freed, pooled or shared merely because publication permission was revoked.

**Primary actions:** [AC-30](#ac-30), [AC-31](#ac-31), [AC-32](#ac-32), [AC-33](#ac-33), [AC-35](#ac-35), [AC-46](#ac-46).

<a id="ar-09"></a>
### AR-09. Accepted scope and current publication remain orthogonal

A candidate may feed a coupled run before global convergence. P09 qualifies local/unit/flowsheet scope; P10 separately checks live run/ancestor authority, dependency context, target slot and coherent group at publication.

**Primary actions:** [AC-37](#ac-37), [AC-44](#ac-44), [AC-45](#ac-45), [AC-46](#ac-46), [AC-49](#ac-49), [AC-56](#ac-56).

<a id="ar-10"></a>
### AR-10. Coherence includes trial dependencies, not only base revision

A unit group may contain many distinct local problem IDs. Their material input lineage and equations must correspond to the declared run-local view and final process criteria. Equal model revision does not justify combining a stage evaluated from an old iterate with a different current inlet. A conservative identical-view policy is allowed but not mandated.

**Primary actions:** [AC-13](#ac-13), [AC-21](#ac-21), [AC-33](#ac-33), [AC-43](#ac-43), [AC-44](#ac-44), [AC-49](#ac-49).

<a id="ar-11"></a>
### AR-11. Data and physics alternatives require a new original problem

Estimated data are fixed and attributable for each trial. Any phase suppression, changed species set, ideal substitution or frozen chemistry during recovery must be an explicitly temporary initialization intervention or authorized alternative problem. Only restored original conditions may pass original-problem checks.

**Primary actions:** [AC-05](#ac-05), [AC-06](#ac-06), [AC-07](#ac-07), [AC-31](#ac-31), [AC-34](#ac-34), [AC-54](#ac-54).

<a id="ar-12"></a>
### AR-12. Checks are evidence, not automatic independent validation

Same-model reevaluation can test consistency; an independent dataset or independent model assumption is a different evidence type. All required checks use predeclared scales and justified conserved bases. Missing evidence blocks only the acceptance scope that requires it.

**Primary actions:** [AC-36](#ac-36), [AC-37](#ac-37), [AC-38](#ac-38), [AC-55](#ac-55).

<a id="ar-13"></a>
### AR-13. Optional work cannot corrupt primary work

When optional properties are separable, preserve detached primary candidate/assessment and run the extension in safe private state. If primary and optional work share a native call that fails without trustworthy primary output, do not invent success; rerun the primary request separately if eligible and authorized. Optional does not mean dispensable when another required check actually depends on it.

**Primary actions:** [AC-22](#ac-22), [AC-32](#ac-32), [AC-33](#ac-33), [AC-35](#ac-35), [AC-36](#ac-36), [AC-41](#ac-41), [AC-55](#ac-55).

<a id="ar-14"></a>
### AR-14. One material account and explicit transfer intent

Add only disjoint portions; apparent/true/aggregate views are not additive. A representation conversion does not by itself transfer matter. Actual exchange amounts/rates retain source, destination, temporal basis and energy convention, and are reconciled once in the process candidate.

**Primary actions:** [AC-12](#ac-12), [AC-16](#ac-16), [AC-17](#ac-17), [AC-18](#ac-18), [AC-39](#ac-39), [AC-40](#ac-40), [AC-42](#ac-42).

<a id="ar-15"></a>
### AR-15. Caloric compatibility precedes energy acceptance

Energy reference functions, reaction/formation corrections, state properties and real exchanges have distinct roles. The unit owns its balance; P03 supplies coherent conventions. Neither target translation nor acceptance may hide physical-model disagreement as heat.

**Primary actions:** [AC-08](#ac-08), [AC-11](#ac-11), [AC-24](#ac-24), [AC-36](#ac-36), [AC-41](#ac-41), [AC-42](#ac-42).

<a id="ar-16"></a>
### AR-16. Save is a snapshot, not execution

Archive captured semantic definitions, original evidence, bindings, quantities and result provenance without fitting or context-triggered model changes. Restoring arrays does not restore defining equations, current bindings or run permissions; an archive is not a command replay log.

**Primary actions:** [AC-50](#ac-50), [AC-51](#ac-51), [AC-52](#ac-52), [AC-53](#ac-53).

<a id="ar-17"></a>
### AR-17. All front ends use the same intent

Interactive, automation, external-unit and solver-facing operations retain identical replace/patch, binding, authority, diagnostics and mutation semantics. No object-construction history or GUI path silently changes physical intent.

**Primary actions:** [AC-14](#ac-14), [AC-15](#ac-15), [AC-19](#ac-19), [AC-47](#ac-47), [AC-48](#ac-48), [AC-56](#ac-56).

<a id="ar-18"></a>
### AR-18. No new numerical scope follows from this blueprint

The original H/C/R3 obligations and 14/16/4 P1/P2/P3 profiles remain unchanged. Whole coherent provider integration is allowed, and no single provider need supply all methods. Action/specification coverage and authored guard tests are not provider execution, independent representability certification or physical validation.

**Primary actions:** [AC-03](#ac-03), [AC-09](#ac-09), [AC-18](#ac-18), [AC-27](#ac-27), [AC-38](#ac-38), [AC-53](#ac-53).

## 3. Failure, interruption and recoverability

| Observed condition | Detection | Preserved boundary | Permitted next step |
| --- | --- | --- | --- |
| Incomplete identity, basis or specification | AC-01/02/13/21 | Retain draft and exact missing meaning; no complete solve. | Return to the relevant authoring action with new evidence; capture a new problem if inputs change. |
| Unavailable dependency or unsupported exact operation | AC-09/27 | Block that production profile; inspectability and unrelated operations may remain available. | Choose an explicitly qualified compatible realization or record a scope gap; do not inherit upstream catalog coverage. |
| Initialization failure | AC-31 | Record unavailable/failed start, not physical infeasibility. | Try a bounded eligible same-problem initializer or declare blocked; keep old accepted result. |
| Native error, crash or uncertain cleanup | AC-32/35 | Detached accepted state retained; session quarantined/isolated. | Reconstruct/reset only under demonstrated policy. An in-process fatal crash requires actual containment design. |
| Wrong target despite numerical success | AC-36/37 | CHECK_FAILED at the original request; candidate remains diagnostic. | A new bounded same-problem attempt is allowed, not target redefinition. |
| Unknown mandatory check or conditional applicability | AC-36/55 | Not assessed, so requested acceptance scope remains blocked. | Acquire bounded evidence or return insufficient evidence; optional checks stay separately qualified. |
| Optional property failure | AC-22/33/41 | Preserve trustworthy independent primary data; expose per-property absence. | Run optional work safely or omit only when truly optional. A failed shared call cannot fabricate detached primary success. |
| Physical alteration requested by fallback | AC-34/54 | Original failure retained; alternative not yet accepted. | Specific authorization, distinct problem, renewed qualification and its own evidence. |
| Unit/recycle iteration exhaustion | AC-44/37 | Local checked candidates remain available in run-local/diagnostic view; parent unaccepted. | Adjust explicit numerical policy or new physical intent; no parent pass from child success. |
| Cancellation or ancestor supersession | AC-46/49 | Future current publication forbidden, regardless of native completion. | Retain evidence, perform safe cleanup, start a separately authorized run if requested. |
| Concurrent edit or target-slot race | AC-48/49 | No partial commit; stale/conflict recorded. | Reconcile/rebase or create a new proposal; do not replay the old update onto new context. |
| Lost response or duplicate completion | AC-32/48/49 | Outcome may be pending or already decided. | Look up/reconcile the identified attempt or command decision before initiating new effects. |
| Archive missing engine or changed semantic recipe | AC-50/51/53 | Inspection possible, affected execution or original-result authority blocked. | Resolve exact artifacts or label an approved changed-model comparison. |

All actions use B4 outcome meanings via B6 IC-63. A failed child is attributed to its actual request/property/location and linked to parent scope. A successful later attempt does not erase earlier evidence. Input defects, unassessed support, demonstrated infeasibility and generic nonconvergence remain distinct. An optional property can coexist with a completed primary operation, but only when primary values and their required checks are trustworthy independently of the failed work.

## 4. Operation-specific physical promises

These are the concrete specializations of AC-22..26. They define original authoritative input and promised outcomes; they do not choose algorithms or assert a provider already supplies them. Stability and chemistry permissions are not inferred from the convenient name of a flash routine.

| Variant | Preserved/authoritative input | Promised outputs | Required checks and limitations |
| --- | --- | --- | --- |
| OV-01 Fixed supplied-phase property | T,P,composition and supplied phase quantity/allocation as specified; allowed root/density resolution explicitly identified | Requested phase observables, supported derivatives and unavailable fields | No protected material/species/phase reallocation; demanded values correctly scoped |
| OV-02 Nonreactive TP phase equilibrium | T,P,overall component totals and declared physical candidates | Candidate phase amounts/compositions and promised properties only | Component conservation, admissible phase scope and requested stability evidence; saturation underdetermination explicit |
| OV-03 PH state resolution | P,original mass/molar/total H target and reference, overall material and chemical/phase permissions | T,phase allocation and declared caloric outputs | Original H residual plus applicable material/authority checks; a converged bound is insufficient |
| OV-04 PS reference/state resolution | P,original S target and convention, overall material and permitted transformations | T,H and phase information as demanded | Original S residual; ideal equipment reference remains distinct from real unit outlet/efficiency balance |
| OV-05 Saturation endpoint or phase-fraction state | Endpoint request or explicit quality with amount basis/denominator; pressure or temperature as appropriate | Incipient endpoint information or completely allocated state only when sufficiently specified | Dependent pure-fluid T,P cannot determine quality; true mixture and pseudo-pure models retain different meanings |
| OV-06 Stability or constrained branch | Original composition/state plus admitted competitors and explicit restrictions/branch policy | Assessed phase-search/stability scope, candidate competitors and limitations | No global certificate beyond actual evidence; suppressed phases and unassessed competitors retained |
| OV-07 Reactive equilibrium with thermal constraint | Justified conserved totals, coherent chemical system, thermal conditions, allowed external exchanges and participation | Species/phase amounts, thermal state and signed actual exchanges | Combined chemical/phase/thermal and exchange-inclusive conservation checks; no fixed-species-only substitute |
| OV-08 Finite-rate/frozen-chemistry properties | Declared local state, kinetic/frozen/equilibrated partition and rate basis; process time/geometry remains host-owned | Qualified rate/property or kinetic-subproblem outputs as actually requested | No unintended equilibrium of kinetic/frozen subsets; law units and extent normalization preserved |
| OV-09 Phase-pair or effective transport | Named phase/domain pair, directional frame where needed and declared effective closure | Only the intrinsic or closure-defined observables requested | Transport property is not equipment transfer coefficient; no weighting shortcut for a different response derivative |
| OV-10 Derivative or solved response | Base state, independent chart, fixed quantities, response/order/branch specification | Tensor/directional result plus method, mapping and conditioning evidence | Same derivative experiment in verification; transitions and unavailable orders qualified |
| OV-11 Equation/residual/value contribution | Local/host variable mapping and captured original constraints | Actually supported contribution and derivative dependency information | No duplicate constraints or residual initialization fixings; structural dependence not inferred from sampled zeros |
| OV-12 Inventory U,V request, P3 | Actual material amount/composition and total U,V under justified caloric convention; no dummy flow | Unresolved T,P in semantic walkthrough, or supported candidate if separately qualified | Amount and empty-inventory semantics; numerical UV not required by R3 alone |

**Phase/species candidate refinement:** numerical discovery of an instance within a declared admissible domain can be part of solving the same problem. Omitting a physically required candidate or adding an excluded chemical species changes scope. The adapter must state which interpretation applies rather than treating every array-length change as either always harmless or always a new physical model.

**Domain boundaries:** numerical search bounds, data validity, physical admissibility and validation envelopes are different predicates. A recovered root outside the approved domain requires the declared qualified outcome, not an unqualified success. No universal tolerance or phase-matching threshold is chosen here.

## 5. Action catalog

| Action | Accountable semantic owner | Kind | Primary requirements |
| --- | --- | --- | --- |
| [AC-01](#ac-01) Resolve constituent identity without changing material | P01 | prepare | [FR-MAT-01](thermodynamics_functional_requirements_v0_1.md#fr-mat-01) |
| [AC-02](#ac-02) Prepare a material representation and conservation basis | P01 | prepare | [FR-MAT-02](thermodynamics_functional_requirements_v0_1.md#fr-mat-02), [FR-MAT-07](thermodynamics_functional_requirements_v0_1.md#fr-mat-07), [FR-MAT-08](thermodynamics_functional_requirements_v0_1.md#fr-mat-08) |
| [AC-03](#ac-03) Define representation maps and extension operation semantics | P01 | prepare | [FR-EXT-01](thermodynamics_functional_requirements_v0_1.md#fr-ext-01), [FR-EXT-02](thermodynamics_functional_requirements_v0_1.md#fr-ext-02) |
| [AC-04](#ac-04) Resolve evidence and parameter values for a fixed use | P02 | prepare | [FR-DAT-01](thermodynamics_functional_requirements_v0_1.md#fr-dat-01), [FR-DAT-02](thermodynamics_functional_requirements_v0_1.md#fr-dat-02), [FR-DAT-07](thermodynamics_functional_requirements_v0_1.md#fr-dat-07) |
| [AC-05](#ac-05) Propose and authorize an estimate or physical-data substitution | P02 | prepare | [FR-DAT-03](thermodynamics_functional_requirements_v0_1.md#fr-dat-03) |
| [AC-06](#ac-06) Coordinate fixed fitting trials and characterization proposals | P02 | prepare | [FR-DAT-04](thermodynamics_functional_requirements_v0_1.md#fr-dat-04), [FR-DAT-05](thermodynamics_functional_requirements_v0_1.md#fr-dat-05), [FR-DAT-06](thermodynamics_functional_requirements_v0_1.md#fr-dat-06) |
| [AC-07](#ac-07) Assemble a reusable recipe and resolved package proposal | P03 | prepare | [FR-GOV-04](thermodynamics_functional_requirements_v0_1.md#fr-gov-04), [FR-CFG-01](thermodynamics_functional_requirements_v0_1.md#fr-cfg-01), [FR-CFG-05](thermodynamics_functional_requirements_v0_1.md#fr-cfg-05), [FR-CFG-07](thermodynamics_functional_requirements_v0_1.md#fr-cfg-07) |
| [AC-08](#ac-08) Assess composed methods and reconcile energy conventions | P03 | assess | [FR-CFG-02](thermodynamics_functional_requirements_v0_1.md#fr-cfg-02), [FR-CFG-04](thermodynamics_functional_requirements_v0_1.md#fr-cfg-04), [FR-PRP-07](thermodynamics_functional_requirements_v0_1.md#fr-prp-07), [FR-CHM-05](thermodynamics_functional_requirements_v0_1.md#fr-chm-05) |
| [AC-09](#ac-09) Assess request-specific readiness and expose unresolved work | P03 | assess | [FR-CFG-03](thermodynamics_functional_requirements_v0_1.md#fr-cfg-03) |
| [AC-10](#ac-10) Define chemistry, participation and allowed reservoir requirements | P06 | prepare | [FR-CHM-01](thermodynamics_functional_requirements_v0_1.md#fr-chm-01), [FR-CHM-04](thermodynamics_functional_requirements_v0_1.md#fr-chm-04), [FR-CHM-07](thermodynamics_functional_requirements_v0_1.md#fr-chm-07) |
| [AC-11](#ac-11) Apply a strict extent or conversion to a material candidate | P06 | derive | [FR-CHM-02](thermodynamics_functional_requirements_v0_1.md#fr-chm-02) |
| [AC-12](#ac-12) Reconcile apparent and true descriptions of one material | P06 | derive | [FR-CHM-06](thermodynamics_functional_requirements_v0_1.md#fr-chm-06) |
| [AC-13](#ac-13) Capture a local material account and role-qualified state | P04 | prepare | [FR-MAT-03](thermodynamics_functional_requirements_v0_1.md#fr-mat-03), [FR-MAT-04](thermodynamics_functional_requirements_v0_1.md#fr-mat-04), [FR-STA-01](thermodynamics_functional_requirements_v0_1.md#fr-sta-01), [FR-STA-04](thermodynamics_functional_requirements_v0_1.md#fr-sta-04), [FR-EXT-03](thermodynamics_functional_requirements_v0_1.md#fr-ext-03) |
| [AC-14](#ac-14) Propose complete composition replacement | P04 | derive | [FR-MAT-05](thermodynamics_functional_requirements_v0_1.md#fr-mat-05) |
| [AC-15](#ac-15) Propose quantity patches, rescaling and direction changes | P04 | derive | [FR-MAT-06](thermodynamics_functional_requirements_v0_1.md#fr-mat-06), [FR-STA-07](thermodynamics_functional_requirements_v0_1.md#fr-sta-07) |
| [AC-16](#ac-16) Describe phases and assess cross-snapshot correspondence | P04 | assess | [FR-STA-05](thermodynamics_functional_requirements_v0_1.md#fr-sta-05), [FR-STA-06](thermodynamics_functional_requirements_v0_1.md#fr-sta-06) |
| [AC-17](#ac-17) Derive a reporting view without changing physical material | P04 | derive | [FR-STA-08](thermodynamics_functional_requirements_v0_1.md#fr-sta-08) |
| [AC-18](#ac-18) Apply declared distribution, loading and transfer accounting | P04 | derive | None |
| [AC-19](#ac-19) Resolve and propose an effective location binding | P07 | prepare | [FR-CFG-06](thermodynamics_functional_requirements_v0_1.md#fr-cfg-06) |
| [AC-20](#ac-20) Declare unit demands, constraints and completion scope | P07 | prepare | [FR-FLW-01](thermodynamics_functional_requirements_v0_1.md#fr-flw-01) |
| [AC-21](#ac-21) Capture and assess the original physical calculation problem | P05 | prepare | [FR-STA-02](thermodynamics_functional_requirements_v0_1.md#fr-sta-02), [FR-STA-03](thermodynamics_functional_requirements_v0_1.md#fr-sta-03) |
| [AC-22](#ac-22) Prepare property-only, transport and effective-property operations | P05 | specialize | [FR-EQL-05](thermodynamics_functional_requirements_v0_1.md#fr-eql-05), [FR-PRP-01](thermodynamics_functional_requirements_v0_1.md#fr-prp-01), [FR-PRP-02](thermodynamics_functional_requirements_v0_1.md#fr-prp-02), [FR-PRP-03](thermodynamics_functional_requirements_v0_1.md#fr-prp-03), [FR-PRP-04](thermodynamics_functional_requirements_v0_1.md#fr-prp-04) |
| [AC-23](#ac-23) Prepare TP, PH, PS, saturation, stability and bounded phase resolution | P05 | specialize | [FR-EQL-01](thermodynamics_functional_requirements_v0_1.md#fr-eql-01), [FR-EQL-02](thermodynamics_functional_requirements_v0_1.md#fr-eql-02), [FR-EQL-03](thermodynamics_functional_requirements_v0_1.md#fr-eql-03), [FR-EQL-04](thermodynamics_functional_requirements_v0_1.md#fr-eql-04), [FR-EQL-06](thermodynamics_functional_requirements_v0_1.md#fr-eql-06), [FR-EQL-07](thermodynamics_functional_requirements_v0_1.md#fr-eql-07), [FR-EQL-08](thermodynamics_functional_requirements_v0_1.md#fr-eql-08), [FR-EXT-04](thermodynamics_functional_requirements_v0_1.md#fr-ext-04) |
| [AC-24](#ac-24) Prepare a coupled chemistry, phase and thermal problem | P05 | specialize | [FR-CHM-03](thermodynamics_functional_requirements_v0_1.md#fr-chm-03) |
| [AC-25](#ac-25) Prepare and assess the meaning of a derivative experiment | P05 | specialize | [FR-PRP-05](thermodynamics_functional_requirements_v0_1.md#fr-prp-05), [FR-PRP-06](thermodynamics_functional_requirements_v0_1.md#fr-prp-06) |
| [AC-26](#ac-26) Define thermodynamic contributions and restoration obligations | P05 | specialize | None |
| [AC-27](#ac-27) Register a provider realization and verify dependency availability | P08 | prepare | [FR-GOV-02](thermodynamics_functional_requirements_v0_1.md#fr-gov-02), [FR-GOV-03](thermodynamics_functional_requirements_v0_1.md#fr-gov-03) |
| [AC-28](#ac-28) Bind native coordinates and observable conventions | P08 | prepare | None |
| [AC-29](#ac-29) Plan a bounded execution attempt | P08 | execute | None |
| [AC-30](#ac-30) Acquire a compatible, isolated numerical session | P08 | execute | [FR-RUN-03](thermodynamics_functional_requirements_v0_1.md#fr-run-03), [FR-RUN-04](thermodynamics_functional_requirements_v0_1.md#fr-run-04) |
| [AC-31](#ac-31) Initialize and restore the original calculation formulation | P08 | execute | [FR-RUN-01](thermodynamics_functional_requirements_v0_1.md#fr-run-01), [FR-RUN-02](thermodynamics_functional_requirements_v0_1.md#fr-run-02) |
| [AC-32](#ac-32) Execute an operation on private working state | P08 | execute | None |
| [AC-33](#ac-33) Normalize and detach returned candidate evidence | P08 | derive | None |
| [AC-34](#ac-34) Retry the same physical problem under a new attempt | P08 | execute | [FR-RUN-05](thermodynamics_functional_requirements_v0_1.md#fr-run-05) |
| [AC-35](#ac-35) Quarantine, restore or retire a provider session | P08 | execute | [FR-RUN-06](thermodynamics_functional_requirements_v0_1.md#fr-run-06), [FR-RUN-07](thermodynamics_functional_requirements_v0_1.md#fr-run-07) |
| [AC-36](#ac-36) Evaluate original postconditions and identify missing evidence | P09 | assess | [FR-RES-02](thermodynamics_functional_requirements_v0_1.md#fr-res-02), [FR-RES-03](thermodynamics_functional_requirements_v0_1.md#fr-res-03) |
| [AC-37](#ac-37) Qualify local, unit or flowsheet results for a named scope | P09 | assess | [FR-CHM-08](thermodynamics_functional_requirements_v0_1.md#fr-chm-08), [FR-RES-01](thermodynamics_functional_requirements_v0_1.md#fr-res-01), [FR-RES-04](thermodynamics_functional_requirements_v0_1.md#fr-res-04), [FR-RES-07](thermodynamics_functional_requirements_v0_1.md#fr-res-07) |
| [AC-38](#ac-38) Register bounded conformance and validation evidence | P09 | assess | [FR-GOV-01](thermodynamics_functional_requirements_v0_1.md#fr-gov-01), [FR-GOV-05](thermodynamics_functional_requirements_v0_1.md#fr-gov-05), [FR-RES-08](thermodynamics_functional_requirements_v0_1.md#fr-res-08) |
| [AC-39](#ac-39) Mix compatible feeds or apply a mechanical split | P07 | derive | [FR-FLW-02](thermodynamics_functional_requirements_v0_1.md#fr-flw-02) |
| [AC-40](#ac-40) Route internal phases and carried solids to product ports | P07 | derive | [FR-FLW-03](thermodynamics_functional_requirements_v0_1.md#fr-flw-03) |
| [AC-41](#ac-41) Coordinate heat-only coupling between distinct material regions | P07 | derive | [FR-FLW-04](thermodynamics_functional_requirements_v0_1.md#fr-flw-04) |
| [AC-42](#ac-42) Translate material descriptions under explicit preserved constraints | P07 | derive | [FR-FLW-05](thermodynamics_functional_requirements_v0_1.md#fr-flw-05), [FR-FLW-06](thermodynamics_functional_requirements_v0_1.md#fr-flw-06) |
| [AC-43](#ac-43) Coordinate a coupled unit or nonequilibrium mathematical formulation | P07 | coordinate | [FR-FLW-07](thermodynamics_functional_requirements_v0_1.md#fr-flw-07), [FR-FLW-08](thermodynamics_functional_requirements_v0_1.md#fr-flw-08) |
| [AC-44](#ac-44) Advance run-local candidates and evaluate parent convergence | P07 | coordinate | [FR-RUN-08](thermodynamics_functional_requirements_v0_1.md#fr-run-08) |
| [AC-45](#ac-45) Issue scoped run authority and capture dependencies | P10 | commit | None |
| [AC-46](#ac-46) Cancel or supersede a run and revoke future publication | P10 | commit | None |
| [AC-47](#ac-47) Prepare a coordinated change with domain validation and impact | P10 | prepare | [FR-LIF-02](thermodynamics_functional_requirements_v0_1.md#fr-lif-02), [FR-LIF-03](thermodynamics_functional_requirements_v0_1.md#fr-lif-03) |
| [AC-48](#ac-48) Commit a definition change and invalidate dependent authority | P10 | commit | [FR-LIF-01](thermodynamics_functional_requirements_v0_1.md#fr-lif-01), [FR-LIF-08](thermodynamics_functional_requirements_v0_1.md#fr-lif-08) |
| [AC-49](#ac-49) Publish a qualified coherent result set with live guards | P10 | commit | [FR-RES-05](thermodynamics_functional_requirements_v0_1.md#fr-res-05) |
| [AC-50](#ac-50) Export a coherent semantic archive without running thermodynamics | P10 | export | [FR-LIF-04](thermodynamics_functional_requirements_v0_1.md#fr-lif-04) |
| [AC-51](#ac-51) Inspect, rebuild and explicitly migrate a saved model | P10 | restore | [FR-LIF-05](thermodynamics_functional_requirements_v0_1.md#fr-lif-05) |
| [AC-52](#ac-52) Restore or undo a snapshot as a guarded semantic change | P10 | restore | [FR-LIF-06](thermodynamics_functional_requirements_v0_1.md#fr-lif-06) |
| [AC-53](#ac-53) Reproduce a recorded case or classify a changed-model comparison | P10 | restore | [FR-LIF-07](thermodynamics_functional_requirements_v0_1.md#fr-lif-07) |
| [AC-54](#ac-54) Authorize a distinct changed-physics alternative | P05 | prepare | None |
| [AC-55](#ac-55) Obtain missing check evidence without changing the checked candidate | P07 | coordinate | None |
| [AC-56](#ac-56) Query result status and lineage without promoting its authority | P09 | query | [FR-RES-06](thermodynamics_functional_requirements_v0_1.md#fr-res-06) |

Every action inherits §1 and applicable AR/SI rules. Supporting actions without primary requirement ownership exist because end-to-end behavior sometimes needs more than one operation. “Inputs” and “outputs” below refer to exact B6 concepts, not physical payload layouts; diagnostic outcomes use IC-63 even where omitted from a success-output list. The accountable package owns the action meaning, not every output concept or numerical implementation.

<a id="ac-01"></a>
### AC-01. Resolve constituent identity without changing material

**Accountable owner:** P01, Material semantics and representations. **Action kind:** prepare.

**Initiator/trigger:** An author, import adapter or provider mapper supplies a name, source identifier or constituent assertion.

**Inputs:** [IC-01](thermodynamics_semantic_information_dictionary_v0_1.md#ic-01), [IC-02](thermodynamics_semantic_information_dictionary_v0_1.md#ic-02), [IC-10](thermodynamics_semantic_information_dictionary_v0_1.md#ic-10).

**Preconditions:** The identifier namespace and source are stated; original assertions are retained. A name alone does not prove identity.

**Procedure and information flow:**

1. P01 searches only the applicable identity assertions.
2. P01 records a unique match, competing matches or unresolved identity with provenance.
3. P01 proposes a new constituent only when its meaning is explicitly supplied; an assay cut does not gain a fabricated registry identity.

**Successful outputs:** [IC-01](thermodynamics_semantic_information_dictionary_v0_1.md#ic-01), [IC-02](thermodynamics_semantic_information_dictionary_v0_1.md#ic-02). A resolved identity assertion or inspectable unresolved proposal; no quantities, selected slate or current package changed.

**Permitted effects:** Only private proposals, snapshots and assessments may be built; no approved model/result mutation.

**Adoption/completion boundary:** A proposal becomes a current model definition only through AC-47/48; it conveys no numerical acceptance.

**Failure and recovery:** UNKNOWN_IDENTITY or AMBIGUOUS_IDENTITY blocks dependent complete operations. Resolution requires additional evidence or an explicit material-definition proposal, not an arbitrary first match.

**Repeated delivery/re-execution:** Repeating an identical resolution against the same evidence may reuse its assessment; changed evidence produces a new assessment, not a rewritten past match.

**AW-01-P:** Equivalent scoped aliases resolve to the same constituent without changing component amounts.
**AW-01-N:** Two conflicting registry assertions cannot be normalized to a silently selected identity.

**Primary B4 requirements:** [FR-MAT-01](thermodynamics_functional_requirements_v0_1.md#fr-mat-01). **Supporting requirements:** None.

**Common action rules:** None. **Handoff facts:** EV-01

**Evidence status:** Action specified. Its positive/negative witnesses have not been executed against the simulator or a thermodynamic provider. Primary ownership is inherited from B5; information and invariants are inherited from the linked B6 concepts.

<a id="ac-02"></a>
### AC-02. Prepare a material representation and conservation basis

**Accountable owner:** P01, Material semantics and representations. **Action kind:** prepare.

**Initiator/trigger:** An author adds a material slate or changes its coordinate and conservation meaning.

**Inputs:** [IC-01](thermodynamics_semantic_information_dictionary_v0_1.md#ic-01), [IC-03](thermodynamics_semantic_information_dictionary_v0_1.md#ic-03), [IC-04](thermodynamics_semantic_information_dictionary_v0_1.md#ic-04), [IC-05](thermodynamics_semantic_information_dictionary_v0_1.md#ic-05), [IC-06](thermodynamics_semantic_information_dictionary_v0_1.md#ic-06).

**Preconditions:** Constituent meanings and representation purpose are available; mass-only and molecular descriptions are explicitly distinguished.

**Procedure and information flow:**

1. P01 specifies constituents, allowed domains and coordinate roles.
2. P01 defines permissible basis conversions and declares only justified conserved rows.
3. P01 assesses completeness and ordering maps, retaining absent molecular/elemental information as unavailable.
4. Return a revision proposal for coordinated adoption by AC-47/48.

**Successful outputs:** [IC-03](thermodynamics_semantic_information_dictionary_v0_1.md#ic-03), [IC-04](thermodynamics_semantic_information_dictionary_v0_1.md#ic-04), [IC-05](thermodynamics_semantic_information_dictionary_v0_1.md#ic-05), [IC-06](thermodynamics_semantic_information_dictionary_v0_1.md#ic-06). A representation proposal with reproducible coordinates and conservation claims, independent of local amounts and provider ordering.

**Permitted effects:** Only private proposals, snapshots and assessments may be built; no approved model/result mutation.

**Adoption/completion boundary:** A proposal becomes a current model definition only through AC-47/48; it conveys no numerical acceptance.

**Failure and recovery:** Reject claimed conversions needing missing molecular data; preserve a valid limited mass-based description. Unresolved active reaction or coordinate references require repair before affected readiness.

**Repeated delivery/re-execution:** Same proposal may be reused; a reordered native vector is a mapping change, not a new chemical identity. Committing the proposal is a separate command.

**AW-02-P:** A mass-based empirical material supports a justified mass balance without an elemental or mole-balance claim.
**AW-02-N:** Changing constituent order without mapping parameter and derivative axes fails qualification.

**Primary B4 requirements:** [FR-MAT-02](thermodynamics_functional_requirements_v0_1.md#fr-mat-02), [FR-MAT-07](thermodynamics_functional_requirements_v0_1.md#fr-mat-07), [FR-MAT-08](thermodynamics_functional_requirements_v0_1.md#fr-mat-08). **Supporting requirements:** None.

**Common action rules:** None. **Handoff facts:** EV-01

**Evidence status:** Action specified. Its positive/negative witnesses have not been executed against the simulator or a thermodynamic provider. Primary ownership is inherited from B5; information and invariants are inherited from the linked B6 concepts.

<a id="ac-03"></a>
### AC-03. Define representation maps and extension operation semantics

**Accountable owner:** P01, Material semantics and representations. **Action kind:** prepare.

**Initiator/trigger:** An author defines lumping, reporting/species coordinates, a distribution reduction, site loading or selective-transfer domain.

**Inputs:** [IC-03](thermodynamics_semantic_information_dictionary_v0_1.md#ic-03), [IC-04](thermodynamics_semantic_information_dictionary_v0_1.md#ic-04), [IC-05](thermodynamics_semantic_information_dictionary_v0_1.md#ic-05), [IC-06](thermodynamics_semantic_information_dictionary_v0_1.md#ic-06), [IC-08](thermodynamics_semantic_information_dictionary_v0_1.md#ic-08), [IC-09](thermodynamics_semantic_information_dictionary_v0_1.md#ic-09).

**Preconditions:** Source and target meaning, support coordinates, denominator and justified conservation are supplied; P3 numerical support is not presumed.

**Procedure and information flow:**

1. P01 defines the forward relation, including state-dependent constraints where a constant matrix is inappropriate.
2. Record conservation, reversibility and information-loss conditions.
3. For distributions specify weighting/support/reduction; for sites specify loading convention and receiving material account.
4. Submit definitions for revision-checked adoption; apply to quantities only through an explicit action.

**Successful outputs:** [IC-07](thermodynamics_semantic_information_dictionary_v0_1.md#ic-07), [IC-08](thermodynamics_semantic_information_dictionary_v0_1.md#ic-08), [IC-09](thermodynamics_semantic_information_dictionary_v0_1.md#ic-09). A map or extension definition supporting an actual state-and-transfer walkthrough, not an opaque metadata attachment.

**Permitted effects:** Only private proposals, snapshots and assessments may be built; no approved model/result mutation.

**Adoption/completion boundary:** A proposal becomes a current model definition only through AC-47/48; it conveys no numerical acceptance.

**Failure and recovery:** A missing denominator or unsupported inverse leaves the map incomplete. No unique detailed composition or distribution is manufactured from a lump or mean.

**Repeated delivery/re-execution:** A map definition is repeatable by revision; applying it is not an inventory mutation. A physical transfer is distinguished from a new view.

**AW-03-P:** A two-component-to-one-lump map preserves stated mass and records the lost split.
**AW-03-N:** A mean molecular weight alone cannot be declared a lossless polymer distribution.

**Primary B4 requirements:** [FR-EXT-01](thermodynamics_functional_requirements_v0_1.md#fr-ext-01), [FR-EXT-02](thermodynamics_functional_requirements_v0_1.md#fr-ext-02). **Supporting requirements:** None.

**Common action rules:** [AR-18](#ar-18). **Handoff facts:** EV-01

**Evidence status:** Action specified. Its positive/negative witnesses have not been executed against the simulator or a thermodynamic provider. Primary ownership is inherited from B5; information and invariants are inherited from the linked B6 concepts.

<a id="ac-04"></a>
### AC-04. Resolve evidence and parameter values for a fixed use

**Accountable owner:** P02, Property evidence, parameters and characterization. **Action kind:** prepare.

**Initiator/trigger:** A preparation coordinator requests the data needed by a declared method and material representation.

**Inputs:** [IC-01](thermodynamics_semantic_information_dictionary_v0_1.md#ic-01), [IC-10](thermodynamics_semantic_information_dictionary_v0_1.md#ic-10), [IC-11](thermodynamics_semantic_information_dictionary_v0_1.md#ic-11), [IC-12](thermodynamics_semantic_information_dictionary_v0_1.md#ic-12), [IC-13](thermodynamics_semantic_information_dictionary_v0_1.md#ic-13), [IC-15](thermodynamics_semantic_information_dictionary_v0_1.md#ic-15), [IC-18](thermodynamics_semantic_information_dictionary_v0_1.md#ic-18).

**Preconditions:** Formula-qualified parameter descriptors, source evidence and override precedence are identified.

**Procedure and information flow:**

1. P02 distinguishes supplied zero, missing, intentionally omitted and rule-derived values.
2. Resolve precedence with ordered subject roles, units, validity and uncertainty retained.
3. Build a fixed snapshot or an explicit unresolved-needs report.
4. Record model/data domains separately from numerical limits; do not silently fit missing data.

**Successful outputs:** [IC-10](thermodynamics_semantic_information_dictionary_v0_1.md#ic-10), [IC-11](thermodynamics_semantic_information_dictionary_v0_1.md#ic-11), [IC-12](thermodynamics_semantic_information_dictionary_v0_1.md#ic-12), [IC-13](thermodynamics_semantic_information_dictionary_v0_1.md#ic-13), [IC-14](thermodynamics_semantic_information_dictionary_v0_1.md#ic-14), [IC-15](thermodynamics_semantic_information_dictionary_v0_1.md#ic-15). Selected data are attributable and fixed for subsequent attempts; unresolved prerequisites name the affected operations.

**Permitted effects:** Only private proposals, snapshots and assessments may be built; no approved model/result mutation.

**Adoption/completion boundary:** A proposal becomes a current model definition only through AC-47/48; it conveys no numerical acceptance.

**Failure and recovery:** MISSING_DATA, conflicting precedence or incompatible formula/units block the affected use. A permitted limited operation may remain ready; missing values never become zero.

**Repeated delivery/re-execution:** The same source/selection revisions yield a reusable snapshot. New observations or overrides create a new candidate snapshot.

**AW-04-P:** A directed pair coefficient retains its constituent roles and source equation.
**AW-04-N:** A missing interaction is not silently set to zero or forced symmetric.

**Primary B4 requirements:** [FR-DAT-01](thermodynamics_functional_requirements_v0_1.md#fr-dat-01), [FR-DAT-02](thermodynamics_functional_requirements_v0_1.md#fr-dat-02), [FR-DAT-07](thermodynamics_functional_requirements_v0_1.md#fr-dat-07). **Supporting requirements:** None.

**Common action rules:** None. **Handoff facts:** EV-01

**Evidence status:** Action specified. Its positive/negative witnesses have not been executed against the simulator or a thermodynamic provider. Primary ownership is inherited from B5; information and invariants are inherited from the linked B6 concepts.

<a id="ac-05"></a>
### AC-05. Propose and authorize an estimate or physical-data substitution

**Accountable owner:** P02, Property evidence, parameters and characterization. **Action kind:** prepare.

**Initiator/trigger:** Missing data motivate an explicit estimate or substitute under an author-approved preparation policy.

**Inputs:** [IC-10](thermodynamics_semantic_information_dictionary_v0_1.md#ic-10), [IC-12](thermodynamics_semantic_information_dictionary_v0_1.md#ic-12), [IC-14](thermodynamics_semantic_information_dictionary_v0_1.md#ic-14), [IC-15](thermodynamics_semantic_information_dictionary_v0_1.md#ic-15), [IC-16](thermodynamics_semantic_information_dictionary_v0_1.md#ic-16), [IC-17](thermodynamics_semantic_information_dictionary_v0_1.md#ic-17).

**Preconditions:** The original missing need, permitted derivation/substitution policy and limits are known; authorization can be predeclared, not necessarily a new interactive prompt.

**Procedure and information flow:**

1. P02 states the proposed estimator or substituted physical description and its expected limits.
2. P07 obtains any numerical evidence through fixed trial problems and AC-29/32.
3. P02 separates estimates, fitted values and substitutes and assesses their evidence.
4. Propose adoption through AC-47/48 without modifying approved data during evaluation.

**Successful outputs:** [IC-13](thermodynamics_semantic_information_dictionary_v0_1.md#ic-13), [IC-14](thermodynamics_semantic_information_dictionary_v0_1.md#ic-14), [IC-17](thermodynamics_semantic_information_dictionary_v0_1.md#ic-17). An attributable candidate data revision with approval basis and explicit physical consequences; original missing-data outcome remains in history.

**Permitted effects:** Only private proposals, snapshots and assessments may be built; no approved model/result mutation.

**Adoption/completion boundary:** A proposal becomes a current model definition only through AC-47/48; it conveys no numerical acceptance.

**Failure and recovery:** Failed estimation returns no approved replacement. A near-ideal substitute cannot be reported as original-data evaluation; an unauthorized substitute remains only a proposal.

**Repeated delivery/re-execution:** Repeated delivery of the same proposal cannot adopt it twice. Re-estimation with new evidence is a new trial/proposal.

**AW-05-P:** An approved estimated pair remains visible in downstream result lineage.
**AW-05-N:** Attaching a material context cannot trigger an unrecorded ideal-data substitution.

**Primary B4 requirements:** [FR-DAT-03](thermodynamics_functional_requirements_v0_1.md#fr-dat-03). **Supporting requirements:** None.

**Common action rules:** [AR-11](#ar-11). **Handoff facts:** EV-01

**Evidence status:** Action specified. Its positive/negative witnesses have not been executed against the simulator or a thermodynamic provider. Primary ownership is inherited from B5; information and invariants are inherited from the linked B6 concepts.

<a id="ac-06"></a>
### AC-06. Coordinate fixed fitting trials and characterization proposals

**Accountable owner:** P02, Property evidence, parameters and characterization. **Action kind:** prepare.

**Initiator/trigger:** An author requests parameter fitting, petroleum characterization or assessment of a prepared trial.

**Inputs:** [IC-03](thermodynamics_semantic_information_dictionary_v0_1.md#ic-03), [IC-10](thermodynamics_semantic_information_dictionary_v0_1.md#ic-10), [IC-11](thermodynamics_semantic_information_dictionary_v0_1.md#ic-11), [IC-12](thermodynamics_semantic_information_dictionary_v0_1.md#ic-12), [IC-14](thermodynamics_semantic_information_dictionary_v0_1.md#ic-14), [IC-16](thermodynamics_semantic_information_dictionary_v0_1.md#ic-16), [IC-20](thermodynamics_semantic_information_dictionary_v0_1.md#ic-20), [IC-35](thermodynamics_semantic_information_dictionary_v0_1.md#ic-35), [IC-56](thermodynamics_semantic_information_dictionary_v0_1.md#ic-56), [IC-64](thermodynamics_semantic_information_dictionary_v0_1.md#ic-64).

**Preconditions:** Original measurements/assay and conventions, recipe, trial bounds, objectives and selection policy are captured. Conditional numerical functionality is checked separately.

**Procedure and information flow:**

1. P02 defines the recipe, required evaluations and trial interpretation.
2. P07 creates fixed trial representations/data/packages and trial-only run authority, invoking P08 numerics rather than making the catalog call back into itself.
3. P02 interprets returned trials and records failed trials, fit evidence and proposed generated constituents.
4. P01 assesses proposed identities; P03 assesses compatibility/caloric completion.
5. P10 may adopt the coordinated proposal only via AC-47/48.

**Successful outputs:** [IC-16](thermodynamics_semantic_information_dictionary_v0_1.md#ic-16), [IC-17](thermodynamics_semantic_information_dictionary_v0_1.md#ic-17), [IC-14](thermodynamics_semantic_information_dictionary_v0_1.md#ic-14). Raw assay, trial snapshots, derived cuts/data and chosen proposal retain lineage. Serialization exports this content without running hidden thermodynamics.

**Permitted effects:** Only private proposals, snapshots and assessments may be built; no approved model/result mutation.

**Adoption/completion boundary:** A proposal becomes a current model definition only through AC-47/48; it conveys no numerical acceptance.

**Failure and recovery:** A failed trial does not invalidate approved production data. A PVT-only cut set cannot qualify thermal calculations. Cancelled or stale campaigns cannot commit a winner automatically.

**Repeated delivery/re-execution:** Every genuinely repeated numerical trial has an attempt identity; redelivery of its evidence does not create a new material definition. Campaign replay is not an implicit production edit.

**AW-06-P:** Changing a characterization recipe yields a distinguishable cut/data proposal while retaining the original assay.
**AW-06-N:** Saving an unresolved package must not launch fitting or mutate production interaction parameters.

**Primary B4 requirements:** [FR-DAT-04](thermodynamics_functional_requirements_v0_1.md#fr-dat-04), [FR-DAT-05](thermodynamics_functional_requirements_v0_1.md#fr-dat-05), [FR-DAT-06](thermodynamics_functional_requirements_v0_1.md#fr-dat-06). **Supporting requirements:** None.

**Common action rules:** [AR-02](#ar-02), [AR-11](#ar-11). **Handoff facts:** EV-01

**Evidence status:** Action specified. Its positive/negative witnesses have not been executed against the simulator or a thermodynamic provider. Primary ownership is inherited from B5; information and invariants are inherited from the linked B6 concepts.

<a id="ac-07"></a>
### AC-07. Assemble a reusable recipe and resolved package proposal

**Accountable owner:** P03, Thermodynamic methods and configured packages. **Action kind:** prepare.

**Initiator/trigger:** An author chooses a whole configured engine or compatible constituent methods.

**Inputs:** [IC-03](thermodynamics_semantic_information_dictionary_v0_1.md#ic-03), [IC-14](thermodynamics_semantic_information_dictionary_v0_1.md#ic-14), [IC-18](thermodynamics_semantic_information_dictionary_v0_1.md#ic-18), [IC-19](thermodynamics_semantic_information_dictionary_v0_1.md#ic-19), [IC-21](thermodynamics_semantic_information_dictionary_v0_1.md#ic-21), [IC-22](thermodynamics_semantic_information_dictionary_v0_1.md#ic-22), [IC-45](thermodynamics_semantic_information_dictionary_v0_1.md#ic-45), [IC-53](thermodynamics_semantic_information_dictionary_v0_1.md#ic-53).

**Preconditions:** Material, method, data and optional chemistry references are explicit; unresolved choices are allowed in a draft only.

**Procedure and information flow:**

1. P03 accepts a new or existing method-definition descriptor, including required observables/data/conventions; declaring a method does not implement its numerical kernel.
2. P03 records the recipe and all consequential method/data/reference choices.
3. Distinguish physical eligibility/defaults from numerical policy and provider realization.
4. Permit an attributable inseparable provider bundle with its opacity limits.
5. Record a fixed package proposal and request AC-08 compatibility treatment before qualifying use.

**Successful outputs:** [IC-18](thermodynamics_semantic_information_dictionary_v0_1.md#ic-18), [IC-19](thermodynamics_semantic_information_dictionary_v0_1.md#ic-19), [IC-20](thermodynamics_semantic_information_dictionary_v0_1.md#ic-20), [IC-22](thermodynamics_semantic_information_dictionary_v0_1.md#ic-22). A reusable package description independent of local state and native session. Structural phase slots, physical candidates and actually searched phases have distinct roles.

**Permitted effects:** Only private proposals, snapshots and assessments may be built; no approved model/result mutation.

**Adoption/completion boundary:** A proposal becomes a current model definition only through AC-47/48; it conveys no numerical acceptance.

**Failure and recovery:** An EOS name alone or missing mandatory data cannot become a fully resolved ready package. Unsupported phase multiplicity remains a capability gap.

**Repeated delivery/re-execution:** Repeated assembly of fixed inputs can reuse a definition; physical changes create another revision. Numerical-policy changes retain separate execution identity.

**AW-07-P:** Two locations share the package but retain different material states and local restrictions.
**AW-07-N:** Dropping an allowed liquid phase is not recorded solely as a numerical-tolerance adjustment.

**Primary B4 requirements:** [FR-GOV-04](thermodynamics_functional_requirements_v0_1.md#fr-gov-04), [FR-CFG-01](thermodynamics_functional_requirements_v0_1.md#fr-cfg-01), [FR-CFG-05](thermodynamics_functional_requirements_v0_1.md#fr-cfg-05), [FR-CFG-07](thermodynamics_functional_requirements_v0_1.md#fr-cfg-07). **Supporting requirements:** None.

**Common action rules:** [AR-11](#ar-11). **Handoff facts:** EV-01

**Evidence status:** Action specified. Its positive/negative witnesses have not been executed against the simulator or a thermodynamic provider. Primary ownership is inherited from B5; information and invariants are inherited from the linked B6 concepts.

<a id="ac-08"></a>
### AC-08. Assess composed methods and reconcile energy conventions

**Accountable owner:** P03, Thermodynamic methods and configured packages. **Action kind:** assess.

**Initiator/trigger:** A package is composed or caloric/chemical values will cross an integration boundary.

**Inputs:** [IC-03](thermodynamics_semantic_information_dictionary_v0_1.md#ic-03), [IC-12](thermodynamics_semantic_information_dictionary_v0_1.md#ic-12), [IC-14](thermodynamics_semantic_information_dictionary_v0_1.md#ic-14), [IC-15](thermodynamics_semantic_information_dictionary_v0_1.md#ic-15), [IC-18](thermodynamics_semantic_information_dictionary_v0_1.md#ic-18), [IC-20](thermodynamics_semantic_information_dictionary_v0_1.md#ic-20), [IC-21](thermodynamics_semantic_information_dictionary_v0_1.md#ic-21), [IC-43](thermodynamics_semantic_information_dictionary_v0_1.md#ic-43), [IC-44](thermodynamics_semantic_information_dictionary_v0_1.md#ic-44), [IC-45](thermodynamics_semantic_information_dictionary_v0_1.md#ic-45), [IC-47](thermodynamics_semantic_information_dictionary_v0_1.md#ic-47).

**Preconditions:** Relevant standard states, caloric contributions, formation treatment and mapping roles are known or explicitly missing.

**Procedure and information flow:**

1. P03 tests compatibility conditions appropriate to the actual formulation, not matching function names.
2. Account for ideal/residual/excess/correlation contributions only where that decomposition applies.
3. State any supported reference transformation as a scoped function, including composition dependence.
4. Reconcile formation-inclusive material energy with any reaction correction; do not alter chemical standards independently.
5. Return assessed, incompatible or unassessed conclusions.

**Successful outputs:** [IC-21](thermodynamics_semantic_information_dictionary_v0_1.md#ic-21), [IC-23](thermodynamics_semantic_information_dictionary_v0_1.md#ic-23). A convention agreement and caloric-completeness/compatibility assessment applicable to named operations.

**Permitted effects:** Append attributable assessment evidence; original input, candidate and criterion remain fixed.

**Adoption/completion boundary:** Assessment can qualify a named scope but only AC-49 makes a current-result association.

**Failure and recovery:** REFERENCE_INCOMPATIBLE or METHOD_INCOMPATIBLE blocks the claimed coupled operation. Missing evidence is unassessed. Model disagreement cannot be hidden in a fabricated reference offset.

**Repeated delivery/re-execution:** Reassess only against identified definition revisions; additional evidence creates another assessment without rewriting historical values.

**AW-08-P:** Equivalent formation-inclusive and correction-based synthetic energy formulations agree.
**AW-08-N:** Adding full reaction heat again to formation-inclusive stream energy fails the accounting check.

**Primary B4 requirements:** [FR-CFG-02](thermodynamics_functional_requirements_v0_1.md#fr-cfg-02), [FR-CFG-04](thermodynamics_functional_requirements_v0_1.md#fr-cfg-04), [FR-PRP-07](thermodynamics_functional_requirements_v0_1.md#fr-prp-07), [FR-CHM-05](thermodynamics_functional_requirements_v0_1.md#fr-chm-05). **Supporting requirements:** None.

**Common action rules:** [AR-15](#ar-15). **Handoff facts:** EV-02

**Evidence status:** Action specified. Its positive/negative witnesses have not been executed against the simulator or a thermodynamic provider. Primary ownership is inherited from B5; information and invariants are inherited from the linked B6 concepts.

<a id="ac-09"></a>
### AC-09. Assess request-specific readiness and expose unresolved work

**Accountable owner:** P03, Thermodynamic methods and configured packages. **Action kind:** assess.

**Initiator/trigger:** A unit, author or coordinator asks whether a particular operation can be attempted.

**Inputs:** [IC-20](thermodynamics_semantic_information_dictionary_v0_1.md#ic-20), [IC-23](thermodynamics_semantic_information_dictionary_v0_1.md#ic-23), [IC-35](thermodynamics_semantic_information_dictionary_v0_1.md#ic-35), [IC-50](thermodynamics_semantic_information_dictionary_v0_1.md#ic-50), [IC-53](thermodynamics_semantic_information_dictionary_v0_1.md#ic-53), [IC-57](thermodynamics_semantic_information_dictionary_v0_1.md#ic-57), [IC-61](thermodynamics_semantic_information_dictionary_v0_1.md#ic-61), [IC-64](thermodynamics_semantic_information_dictionary_v0_1.md#ic-64).

**Preconditions:** Selected unit mode and mandatory/conditional/optional demands are explicit; provider and initialization attestations are distinguished from scientific validity.

**Procedure and information flow:**

1. P07 gathers domain, provider, initializer and evidence assessments.
2. P03 evaluates applicable representation, data, caloric, method, transport/derivative, algorithm, initializer and boundary prerequisites.
3. Mark conditional demands pending if the condition cannot yet be resolved.
4. Return readiness for preparation, readiness for execution, or specific blocked/unassessed needs; recheck changed prerequisites before dispatch.

**Successful outputs:** [IC-24](thermodynamics_semantic_information_dictionary_v0_1.md#ic-24). A request-scoped assessment; a density-only use may be ready while PH or rating remains blocked.

**Permitted effects:** Append attributable assessment evidence; original input, candidate and criterion remain fixed.

**Adoption/completion boundary:** Assessment can qualify a named scope but only AC-49 makes a current-result association.

**Failure and recovery:** Unknown mandatory readiness cannot be treated as permission for a production solve. Evidence-gathering trials may be explicitly authorized without claiming capability; scope gaps remain open.

**Repeated delivery/re-execution:** A readiness result is reusable only for its captured demand and dependencies. It cannot be cached merely by package name.

**AW-09-P:** Missing diffusivity blocks the selected rate-based mode without erasing a valid enthalpy operation.
**AW-09-N:** Successful object construction cannot authorize a compressor operation lacking entropy or an admissible initializer.

**Primary B4 requirements:** [FR-CFG-03](thermodynamics_functional_requirements_v0_1.md#fr-cfg-03). **Supporting requirements:** None.

**Common action rules:** [AR-07](#ar-07), [AR-18](#ar-18). **Handoff facts:** EV-06

**Evidence status:** Action specified. Its positive/negative witnesses have not been executed against the simulator or a thermodynamic provider. Primary ownership is inherited from B5; information and invariants are inherited from the linked B6 concepts.

<a id="ac-10"></a>
### AC-10. Define chemistry, participation and allowed reservoir requirements

**Accountable owner:** P06, Chemistry definitions and participation. **Action kind:** prepare.

**Initiator/trigger:** A reactor/contacting author defines conversion, equilibrium, finite-rate or mixed chemistry.

**Inputs:** [IC-01](thermodynamics_semantic_information_dictionary_v0_1.md#ic-01), [IC-03](thermodynamics_semantic_information_dictionary_v0_1.md#ic-03), [IC-05](thermodynamics_semantic_information_dictionary_v0_1.md#ic-05), [IC-06](thermodynamics_semantic_information_dictionary_v0_1.md#ic-06), [IC-09](thermodynamics_semantic_information_dictionary_v0_1.md#ic-09), [IC-14](thermodynamics_semantic_information_dictionary_v0_1.md#ic-14), [IC-21](thermodynamics_semantic_information_dictionary_v0_1.md#ic-21).

**Preconditions:** Species/domain identities, extent/rate bases and known data are supplied; the unit retains residence time, geometry and actual exchange authorization.

**Procedure and information flow:**

1. P06 qualifies stoichiometry or database chemical-system meaning against justified conserved rows.
2. Define rate/equilibrium law inputs and normalization independently of reactor geometry.
3. Distinguish equilibrated, kinetic, frozen, inert and excluded participation.
4. Declare reservoir needs; actual permission and transfer quantities are captured in a physical problem and result.

**Successful outputs:** [IC-43](thermodynamics_semantic_information_dictionary_v0_1.md#ic-43), [IC-44](thermodynamics_semantic_information_dictionary_v0_1.md#ic-44), [IC-45](thermodynamics_semantic_information_dictionary_v0_1.md#ic-45), [IC-46](thermodynamics_semantic_information_dictionary_v0_1.md#ic-46). A reusable chemical-system proposal whose rate, phase and conservation semantics remain stable across unit uses.

**Permitted effects:** Only private proposals, snapshots and assessments may be built; no approved model/result mutation.

**Adoption/completion boundary:** A proposal becomes a current model definition only through AC-47/48; it conveys no numerical acceptance.

**Failure and recovery:** Unbalanced claimed chemistry, incompatible rate units or undeclared source/sink blocks the affected calculation. Unsupported kinetics is not replaced by equilibrium.

**Repeated delivery/re-execution:** Definition replay does not perform chemistry. Changing residence time alone need not revise the chemical definition.

**AW-10-P:** A kinetic mineral coexists with an equilibrated aqueous subset while its rate constraint remains explicit.
**AW-10-N:** A closed request cannot acquire gas or titrant simply to satisfy a reservoir condition.

**Primary B4 requirements:** [FR-CHM-01](thermodynamics_functional_requirements_v0_1.md#fr-chm-01), [FR-CHM-04](thermodynamics_functional_requirements_v0_1.md#fr-chm-04), [FR-CHM-07](thermodynamics_functional_requirements_v0_1.md#fr-chm-07). **Supporting requirements:** None.

**Common action rules:** None. **Handoff facts:** EV-01

**Evidence status:** Action specified. Its positive/negative witnesses have not been executed against the simulator or a thermodynamic provider. Primary ownership is inherited from B5; information and invariants are inherited from the linked B6 concepts.

<a id="ac-11"></a>
### AC-11. Apply a strict extent or conversion to a material candidate

**Accountable owner:** P06, Chemistry definitions and participation. **Action kind:** derive.

**Initiator/trigger:** A conversion unit imposes a strict reaction extent or conversion for a captured material.

**Inputs:** [IC-25](thermodynamics_semantic_information_dictionary_v0_1.md#ic-25), [IC-27](thermodynamics_semantic_information_dictionary_v0_1.md#ic-27), [IC-28](thermodynamics_semantic_information_dictionary_v0_1.md#ic-28), [IC-35](thermodynamics_semantic_information_dictionary_v0_1.md#ic-35), [IC-43](thermodynamics_semantic_information_dictionary_v0_1.md#ic-43), [IC-44](thermodynamics_semantic_information_dictionary_v0_1.md#ic-44), [IC-45](thermodynamics_semantic_information_dictionary_v0_1.md#ic-45).

**Preconditions:** Stoichiometric normalization, base-reactant amount, sequential/simultaneous policy and input amounts are fixed; unknown elemental data are not certified.

**Procedure and information flow:**

1. P06 converts specified conversion to the declared extent coordinates.
2. Check availability across the complete reaction group under the declared sequence.
3. Form candidate species amounts without clipping or optimizing a strict target.
4. Return transformation evidence; P07 requests compatible thermal/phase resolution and P09 checks full unit results.

**Successful outputs:** [IC-27](thermodynamics_semantic_information_dictionary_v0_1.md#ic-27), [IC-28](thermodynamics_semantic_information_dictionary_v0_1.md#ic-28), [IC-63](thermodynamics_semantic_information_dictionary_v0_1.md#ic-63). A candidate satisfying the actual strict extent or a demonstrated infeasibility diagnosis, never an implicitly limited success.

**Permitted effects:** Compute from captured inputs into new candidate/proposed values; do not cumulatively modify the source.

**Adoption/completion boundary:** Authored input changes use AC-47/48; calculated result groups use AC-36/37/49. Physical transfers are recorded once per declared intent.

**Failure and recovery:** Negative required final material causes INFEASIBLE with conflicting quantities. A limiting optimization requires AC-54 as a different authorized problem.

**Repeated delivery/re-execution:** Compute from the captured original amounts. Redelivering the same transform must not subtract reactants again; a second physical transformation needs new intent/context.

**AW-11-P:** One mole A with A→B at extent 0.4 gives 0.6 A and 0.4 B.
**AW-11-N:** Extent 1.2 from one mole A is rejected, not clipped to extent 1.0.

**Primary B4 requirements:** [FR-CHM-02](thermodynamics_functional_requirements_v0_1.md#fr-chm-02). **Supporting requirements:** None.

**Common action rules:** [AR-05](#ar-05), [AR-15](#ar-15). **Handoff facts:** Local proposal/candidate handoff under the common contract.

**Evidence status:** Action specified. Its positive/negative witnesses have not been executed against the simulator or a thermodynamic provider. Primary ownership is inherited from B5; information and invariants are inherited from the linked B6 concepts.

<a id="ac-12"></a>
### AC-12. Reconcile apparent and true descriptions of one material

**Accountable owner:** P06, Chemistry definitions and participation. **Action kind:** derive.

**Initiator/trigger:** An electrolyte feed, chemical-system adapter or report changes its composition description.

**Inputs:** [IC-03](thermodynamics_semantic_information_dictionary_v0_1.md#ic-03), [IC-05](thermodynamics_semantic_information_dictionary_v0_1.md#ic-05), [IC-07](thermodynamics_semantic_information_dictionary_v0_1.md#ic-07), [IC-25](thermodynamics_semantic_information_dictionary_v0_1.md#ic-25), [IC-27](thermodynamics_semantic_information_dictionary_v0_1.md#ic-27), [IC-28](thermodynamics_semantic_information_dictionary_v0_1.md#ic-28), [IC-45](thermodynamics_semantic_information_dictionary_v0_1.md#ic-45), [IC-47](thermodynamics_semantic_information_dictionary_v0_1.md#ic-47).

**Preconditions:** Forward/reverse constraints and conserved quantities are stated; one physical account identifies overlapping representations.

**Procedure and information flow:**

1. P06 interprets the chemical map and determines whether it is algebraic or requires AC-24 chemistry resolution.
2. Produce a related description of the same account on the declared basis.
3. Verify conserved rows and imposed charge conditions.
4. Mark loss/nonuniqueness of a reverse map and preserve reporting conventions.

**Successful outputs:** [IC-27](thermodynamics_semantic_information_dictionary_v0_1.md#ic-27), [IC-47](thermodynamics_semantic_information_dictionary_v0_1.md#ic-47), [IC-63](thermodynamics_semantic_information_dictionary_v0_1.md#ic-63). Apparent and true descriptions reconcile without becoming additive inventories.

**Permitted effects:** Compute from captured inputs into new candidate/proposed values; do not cumulatively modify the source.

**Adoption/completion boundary:** Authored input changes use AC-47/48; calculated result groups use AC-36/37/49. Physical transfers are recorded once per declared intent.

**Failure and recovery:** Charge/element mismatch or missing reverse information remains a specific failure. Do not normalize charge away or invent unprovided species detail.

**Repeated delivery/re-execution:** Creating a report/map view cannot transfer mass. Repeating it returns equivalent descriptions under the same context.

**AW-12-P:** A synthetic apparent salt and its ions have one conserved material account.
**AW-12-N:** Adding the apparent salt amount again to its already represented ionic inventory fails duplicate-account checks.

**Primary B4 requirements:** [FR-CHM-06](thermodynamics_functional_requirements_v0_1.md#fr-chm-06). **Supporting requirements:** None.

**Common action rules:** [AR-14](#ar-14). **Handoff facts:** Local proposal/candidate handoff under the common contract.

**Evidence status:** Action specified. Its positive/negative witnesses have not been executed against the simulator or a thermodynamic provider. Primary ownership is inherited from B5; information and invariants are inherited from the linked B6 concepts.

<a id="ac-13"></a>
### AC-13. Capture a local material account and role-qualified state

**Accountable owner:** P04, Material states, quantities and local descriptions. **Action kind:** prepare.

**Initiator/trigger:** A feed, internal stage, exchanger segment, interface, inventory or reference-basis calculation needs a local description.

**Inputs:** [IC-03](thermodynamics_semantic_information_dictionary_v0_1.md#ic-03), [IC-04](thermodynamics_semantic_information_dictionary_v0_1.md#ic-04), [IC-05](thermodynamics_semantic_information_dictionary_v0_1.md#ic-05), [IC-06](thermodynamics_semantic_information_dictionary_v0_1.md#ic-06), [IC-25](thermodynamics_semantic_information_dictionary_v0_1.md#ic-25), [IC-26](thermodynamics_semantic_information_dictionary_v0_1.md#ic-26), [IC-27](thermodynamics_semantic_information_dictionary_v0_1.md#ic-27), [IC-48](thermodynamics_semantic_information_dictionary_v0_1.md#ic-48), [IC-49](thermodynamics_semantic_information_dictionary_v0_1.md#ic-49), [IC-64](thermodynamics_semantic_information_dictionary_v0_1.md#ic-64).

**Preconditions:** Account purpose, local role, known quantities and composition representation are explicit; no graphical connector or flow is required.

**Procedure and information flow:**

1. P04 separates intensive coordinates, physical amount, rate and reference calculation amount.
2. Preserve specified, observed, guessed and calculated roles and all availability statuses.
3. Capture one coherent snapshot with its exact dependencies.
4. At zero flow retain an independently specified composition; for empty inventory retain boundary information without inventing a unique fluid state.

**Successful outputs:** [IC-25](thermodynamics_semantic_information_dictionary_v0_1.md#ic-25), [IC-26](thermodynamics_semantic_information_dictionary_v0_1.md#ic-26), [IC-27](thermodynamics_semantic_information_dictionary_v0_1.md#ic-27), [IC-28](thermodynamics_semantic_information_dictionary_v0_1.md#ic-28). A local snapshot suitable for complete resolution or intentionally partial equation participation.

**Permitted effects:** Only private proposals, snapshots and assessments may be built; no approved model/result mutation.

**Adoption/completion boundary:** A proposal becomes a current model definition only through AC-47/48; it conveys no numerical acceptance.

**Failure and recovery:** Missing amount prevents unjustified extensive outputs, not necessarily intensive evaluation. A zero denominator cannot produce a fabricated normalized composition.

**Repeated delivery/re-execution:** A capture is immutable evidence; later input changes create another snapshot. Capturing a reference mole does not create physical inventory.

**AW-13-P:** A closed inventory carries amounts,U,V with T and P unresolved and no dummy flow.
**AW-13-N:** A stale outlet temperature remains a guess, never an inferred new user specification.

**Primary B4 requirements:** [FR-MAT-03](thermodynamics_functional_requirements_v0_1.md#fr-mat-03), [FR-MAT-04](thermodynamics_functional_requirements_v0_1.md#fr-mat-04), [FR-STA-01](thermodynamics_functional_requirements_v0_1.md#fr-sta-01), [FR-STA-04](thermodynamics_functional_requirements_v0_1.md#fr-sta-04), [FR-EXT-03](thermodynamics_functional_requirements_v0_1.md#fr-ext-03). **Supporting requirements:** None.

**Common action rules:** [AR-01](#ar-01), [AR-10](#ar-10). **Handoff facts:** Local proposal/candidate handoff under the common contract.

**Evidence status:** Action specified. Its positive/negative witnesses have not been executed against the simulator or a thermodynamic provider. Primary ownership is inherited from B5; information and invariants are inherited from the linked B6 concepts.

<a id="ac-14"></a>
### AC-14. Propose complete composition replacement

**Accountable owner:** P04, Material states, quantities and local descriptions. **Action kind:** derive.

**Initiator/trigger:** An author supplies an entire new composition for an existing or new material description.

**Inputs:** [IC-03](thermodynamics_semantic_information_dictionary_v0_1.md#ic-03), [IC-04](thermodynamics_semantic_information_dictionary_v0_1.md#ic-04), [IC-25](thermodynamics_semantic_information_dictionary_v0_1.md#ic-25), [IC-27](thermodynamics_semantic_information_dictionary_v0_1.md#ic-27), [IC-28](thermodynamics_semantic_information_dictionary_v0_1.md#ic-28), [IC-64](thermodynamics_semantic_information_dictionary_v0_1.md#ic-64).

**Preconditions:** Replacement intent, complete constituent/basis coverage, total-quantity preservation and normalization policy are specified.

**Procedure and information flow:**

1. P04 interprets unspecified constituents according to the explicitly complete replacement rule, not object creation history.
2. Validate nonnegativity and the declared normalization policy; retain raw input and any authorized normalization.
3. Recalculate dependent component quantities on justified bases.
4. Return a proposed snapshot for AC-47/48, never a direct live edit.

**Successful outputs:** [IC-27](thermodynamics_semantic_information_dictionary_v0_1.md#ic-27), [IC-28](thermodynamics_semantic_information_dictionary_v0_1.md#ic-28). A deterministic replacement proposal with known preserved totals and derived quantities.

**Permitted effects:** Compute from captured inputs into new candidate/proposed values; do not cumulatively modify the source.

**Adoption/completion boundary:** Authored input changes use AC-47/48; calculated result groups use AC-36/37/49. Physical transfers are recorded once per declared intent.

**Failure and recovery:** Conflicting bases, unknown molecular conversions or invalid fractions reject the proposal; no partial live rewrite.

**Repeated delivery/re-execution:** Repeat the same replacement command once against its expected prior context. Repeated delivery cannot act as a patch to a later state.

**AW-14-P:** Replacing a binary composition with pure A clears B under the stated complete replacement rule.
**AW-14-N:** A loaded stream cannot retain unmentioned B because it was not constructed by the same builder.

**Primary B4 requirements:** [FR-MAT-05](thermodynamics_functional_requirements_v0_1.md#fr-mat-05). **Supporting requirements:** None.

**Common action rules:** [AR-17](#ar-17). **Handoff facts:** Local proposal/candidate handoff under the common contract.

**Evidence status:** Action specified. Its positive/negative witnesses have not been executed against the simulator or a thermodynamic provider. Primary ownership is inherited from B5; information and invariants are inherited from the linked B6 concepts.

<a id="ac-15"></a>
### AC-15. Propose quantity patches, rescaling and direction changes

**Accountable owner:** P04, Material states, quantities and local descriptions. **Action kind:** derive.

**Initiator/trigger:** An author edits selected constituent rates, total amount, or a connection orientation.

**Inputs:** [IC-03](thermodynamics_semantic_information_dictionary_v0_1.md#ic-03), [IC-04](thermodynamics_semantic_information_dictionary_v0_1.md#ic-04), [IC-25](thermodynamics_semantic_information_dictionary_v0_1.md#ic-25), [IC-26](thermodynamics_semantic_information_dictionary_v0_1.md#ic-26), [IC-27](thermodynamics_semantic_information_dictionary_v0_1.md#ic-27), [IC-28](thermodynamics_semantic_information_dictionary_v0_1.md#ic-28), [IC-71](thermodynamics_semantic_information_dictionary_v0_1.md#ic-71), [IC-64](thermodynamics_semantic_information_dictionary_v0_1.md#ic-64).

**Preconditions:** Patch versus rescale intent, held-fixed quantities and rate/inventory purpose are declared. Physical transfer is not inferred from an editing action.

**Procedure and information flow:**

1. P04 applies the chosen preservation rule to captured original values.
2. Recompute only dependent totals/fractions whose required data are available.
3. Associate sign with transfer orientation rather than negative physical fractions or inventory.
4. Return a coordinated edit proposal and identify affected balance assumptions.

**Successful outputs:** [IC-26](thermodynamics_semantic_information_dictionary_v0_1.md#ic-26), [IC-27](thermodynamics_semantic_information_dictionary_v0_1.md#ic-27), [IC-28](thermodynamics_semantic_information_dictionary_v0_1.md#ic-28), [IC-71](thermodynamics_semantic_information_dictionary_v0_1.md#ic-71). A proposed material state with explicit patch/rescale and direction semantics; unchanged constituents remain unchanged when that is the rule.

**Permitted effects:** Compute from captured inputs into new candidate/proposed values; do not cumulatively modify the source.

**Adoption/completion boundary:** Authored input changes use AC-47/48; calculated result groups use AC-36/37/49. Physical transfers are recorded once per declared intent.

**Failure and recovery:** An edit leading to invalid physical amounts or incompatible fixed totals is rejected. Numerical trial coordinates may be nonphysical only in private work.

**Repeated delivery/re-execution:** Commands are scoped to expected prior context; identical redelivery returns the prior decision, not another increment. A genuine second increment requires a new command.

**AW-15-P:** Patching A from 2 to 4 kg/s while preserving B=3 gives total 7, not a rescaled B.
**AW-15-N:** Reversing a stream does not make a constituent mole fraction negative.

**Primary B4 requirements:** [FR-MAT-06](thermodynamics_functional_requirements_v0_1.md#fr-mat-06), [FR-STA-07](thermodynamics_functional_requirements_v0_1.md#fr-sta-07). **Supporting requirements:** None.

**Common action rules:** [AR-05](#ar-05), [AR-17](#ar-17). **Handoff facts:** Local proposal/candidate handoff under the common contract.

**Evidence status:** Action specified. Its positive/negative witnesses have not been executed against the simulator or a thermodynamic provider. Primary ownership is inherited from B5; information and invariants are inherited from the linked B6 concepts.

<a id="ac-16"></a>
### AC-16. Describe phases and assess cross-snapshot correspondence

**Accountable owner:** P04, Material states, quantities and local descriptions. **Action kind:** assess.

**Initiator/trigger:** Normalized provider results or authored snapshots need physical phase descriptions or continuity assessment.

**Inputs:** [IC-06](thermodynamics_semantic_information_dictionary_v0_1.md#ic-06), [IC-22](thermodynamics_semantic_information_dictionary_v0_1.md#ic-22), [IC-28](thermodynamics_semantic_information_dictionary_v0_1.md#ic-28), [IC-29](thermodynamics_semantic_information_dictionary_v0_1.md#ic-29), [IC-42](thermodynamics_semantic_information_dictionary_v0_1.md#ic-42), [IC-56](thermodynamics_semantic_information_dictionary_v0_1.md#ic-56), [IC-58](thermodynamics_semantic_information_dictionary_v0_1.md#ic-58).

**Preconditions:** Presence criteria, amount basis, native placeholder meaning and previous snapshot context are stated.

**Procedure and information flow:**

1. P04 separates disjoint present portions, incipient information, numerical placeholders and aggregate views.
2. Establish within-snapshot identity independent of output position.
3. Apply a declared correspondence policy to previous/current snapshots, allowing split, merge, appearance, disappearance and ambiguity.
4. Pass phase descriptions to routing without making a port role a phase identity.

**Successful outputs:** [IC-29](thermodynamics_semantic_information_dictionary_v0_1.md#ic-29), [IC-30](thermodynamics_semantic_information_dictionary_v0_1.md#ic-30), [IC-31](thermodynamics_semantic_information_dictionary_v0_1.md#ic-31). Phase count and accounting follow physical portions; useful zero-amount continuation data remain qualified.

**Permitted effects:** Append attributable assessment evidence; original input, candidate and criterion remain fixed.

**Adoption/completion boundary:** Assessment can qualify a named scope but only AC-49 makes a current-result association.

**Failure and recovery:** Unresolved correspondence remains ambiguous. If a required routing/continuation decision depends on it, block that decision or request an explicit policy; do not invent continuity.

**Repeated delivery/re-execution:** Repeated correspondence uses the same pair of snapshots and policy. Changing only provider order does not create new material.

**AW-16-P:** Reversing two liquid entries preserves constituent totals; a two-to-one merge has no invented unique survivor.
**AW-16-N:** An aggregate-liquid view is not counted as a third physical liquid.

**Primary B4 requirements:** [FR-STA-05](thermodynamics_functional_requirements_v0_1.md#fr-sta-05), [FR-STA-06](thermodynamics_functional_requirements_v0_1.md#fr-sta-06). **Supporting requirements:** None.

**Common action rules:** [AR-14](#ar-14). **Handoff facts:** Local proposal/candidate handoff under the common contract.

**Evidence status:** Action specified. Its positive/negative witnesses have not been executed against the simulator or a thermodynamic provider. Primary ownership is inherited from B5; information and invariants are inherited from the linked B6 concepts.

<a id="ac-17"></a>
### AC-17. Derive a reporting view without changing physical material

**Accountable owner:** P04, Material states, quantities and local descriptions. **Action kind:** derive.

**Initiator/trigger:** A report asks for wet/dry gas, standard volume, phase fraction or another justified basis.

**Inputs:** [IC-04](thermodynamics_semantic_information_dictionary_v0_1.md#ic-04), [IC-07](thermodynamics_semantic_information_dictionary_v0_1.md#ic-07), [IC-21](thermodynamics_semantic_information_dictionary_v0_1.md#ic-21), [IC-25](thermodynamics_semantic_information_dictionary_v0_1.md#ic-25), [IC-26](thermodynamics_semantic_information_dictionary_v0_1.md#ic-26), [IC-27](thermodynamics_semantic_information_dictionary_v0_1.md#ic-27), [IC-28](thermodynamics_semantic_information_dictionary_v0_1.md#ic-28), [IC-31](thermodynamics_semantic_information_dictionary_v0_1.md#ic-31), [IC-38](thermodynamics_semantic_information_dictionary_v0_1.md#ic-38).

**Preconditions:** Observable, denominator, reference conditions and necessary conversion information are supplied; source snapshot is fixed.

**Procedure and information flow:**

1. P04 distinguishes display conversion, basis conversion and representation view.
2. If reference-condition properties are required, P07 requests a separate AC-22/23 evaluation rather than assuming density.
3. Produce the view with source/denominator/convention lineage.
4. Do not edit the underlying material quantities or accepted equilibrium allocation.

**Successful outputs:** [IC-26](thermodynamics_semantic_information_dictionary_v0_1.md#ic-26), [IC-31](thermodynamics_semantic_information_dictionary_v0_1.md#ic-31). An attributable reporting quantity or a scoped unavailable/incomplete result.

**Permitted effects:** Compute from captured inputs into new candidate/proposed values; do not cumulatively modify the source.

**Adoption/completion boundary:** Authored input changes use AC-47/48; calculated result groups use AC-36/37/49. Physical transfers are recorded once per declared intent.

**Failure and recovery:** Missing standard conditions, denominator or molecular data blocks that conversion, not unrelated properties. Optional reporting failure does not invalidate primary accepted physics.

**Repeated delivery/re-execution:** Repeated view construction is observational; no drying, material removal or new inventory occurs.

**AW-17-P:** Dry-gas composition changes its reporting ratio while actual water flow remains unchanged.
**AW-17-N:** An unlabeled vapor-fraction scalar cannot silently set a two-phase physical state.

**Primary B4 requirements:** [FR-STA-08](thermodynamics_functional_requirements_v0_1.md#fr-sta-08). **Supporting requirements:** None.

**Common action rules:** [AR-14](#ar-14). **Handoff facts:** Local proposal/candidate handoff under the common contract.

**Evidence status:** Action specified. Its positive/negative witnesses have not been executed against the simulator or a thermodynamic provider. Primary ownership is inherited from B5; information and invariants are inherited from the linked B6 concepts.

<a id="ac-18"></a>
### AC-18. Apply declared distribution, loading and transfer accounting

**Accountable owner:** P04, Material states, quantities and local descriptions. **Action kind:** derive.

**Initiator/trigger:** A supported material use case or P3 walkthrough supplies a distribution mix/reduction or explicit surface/material transfer.

**Inputs:** [IC-03](thermodynamics_semantic_information_dictionary_v0_1.md#ic-03), [IC-05](thermodynamics_semantic_information_dictionary_v0_1.md#ic-05), [IC-07](thermodynamics_semantic_information_dictionary_v0_1.md#ic-07), [IC-08](thermodynamics_semantic_information_dictionary_v0_1.md#ic-08), [IC-09](thermodynamics_semantic_information_dictionary_v0_1.md#ic-09), [IC-25](thermodynamics_semantic_information_dictionary_v0_1.md#ic-25), [IC-28](thermodynamics_semantic_information_dictionary_v0_1.md#ic-28), [IC-32](thermodynamics_semantic_information_dictionary_v0_1.md#ic-32), [IC-33](thermodynamics_semantic_information_dictionary_v0_1.md#ic-33), [IC-50](thermodynamics_semantic_information_dictionary_v0_1.md#ic-50), [IC-71](thermodynamics_semantic_information_dictionary_v0_1.md#ic-71).

**Preconditions:** P01 definitions fix support and weighting or loading convention; P07 supplies physical transfer intent, limits and time/amount basis. No kinetic prediction is invented.

**Procedure and information flow:**

1. P04 applies the approved map to the captured amounts and attributes.
2. Weight compatible distributions on the declared material basis; retain reduction loss.
3. For a prescribed transfer derive both donor and receiver candidates and exactly one exchange realization.
4. Check donor availability, receiving domain constraints and justified conservation before submitting a coherent change/publication group.

**Successful outputs:** [IC-28](thermodynamics_semantic_information_dictionary_v0_1.md#ic-28), [IC-32](thermodynamics_semantic_information_dictionary_v0_1.md#ic-32), [IC-33](thermodynamics_semantic_information_dictionary_v0_1.md#ic-33), [IC-71](thermodynamics_semantic_information_dictionary_v0_1.md#ic-71). Candidate material/attribute descriptions reconcile the stated transfer without fictitious fluid phases or an unprovided time step.

**Permitted effects:** Compute from captured inputs into new candidate/proposed values; do not cumulatively modify the source.

**Adoption/completion boundary:** Authored input changes use AC-47/48; calculated result groups use AC-36/37/49. Physical transfers are recorded once per declared intent.

**Failure and recovery:** Unsupported numerical adsorption/polymer/UV methods remain unsupported. Undefined loading conventions, overdrawn donor or unavailable inverse distribution reject the action.

**Repeated delivery/re-execution:** Use a transfer-intent identity tied to source snapshots; repeated delivery cannot apply the same delta twice. A fresh physical interval or transfer requires new intent.

**AW-18-P:** Adding 0.1 mol to one mole on 5 kg sorbent gives 0.22 mol/kg under the stated total-loading convention.
**AW-18-N:** Repeated delivery of that transfer cannot give 1.2 mol on the surface or lose donor material twice.

**Primary B4 requirements:** None. **Supporting requirements:** [FR-EXT-01](thermodynamics_functional_requirements_v0_1.md#fr-ext-01), [FR-EXT-02](thermodynamics_functional_requirements_v0_1.md#fr-ext-02), [FR-FLW-06](thermodynamics_functional_requirements_v0_1.md#fr-flw-06), [FR-MAT-03](thermodynamics_functional_requirements_v0_1.md#fr-mat-03), [FR-RES-03](thermodynamics_functional_requirements_v0_1.md#fr-res-03).

**Common action rules:** [AR-05](#ar-05), [AR-14](#ar-14), [AR-18](#ar-18). **Handoff facts:** Local proposal/candidate handoff under the common contract.

**Evidence status:** Action specified. Its positive/negative witnesses have not been executed against the simulator or a thermodynamic provider. Primary ownership is inherited from B5; information and invariants are inherited from the linked B6 concepts.

<a id="ac-19"></a>
### AC-19. Resolve and propose an effective location binding

**Accountable owner:** P07, Flowsheet integration and use-case coordination. **Action kind:** prepare.

**Initiator/trigger:** A package is assigned directly or inherited by a region, stage, side or inventory.

**Inputs:** [IC-20](thermodynamics_semantic_information_dictionary_v0_1.md#ic-20), [IC-22](thermodynamics_semantic_information_dictionary_v0_1.md#ic-22), [IC-42](thermodynamics_semantic_information_dictionary_v0_1.md#ic-42), [IC-48](thermodynamics_semantic_information_dictionary_v0_1.md#ic-48), [IC-49](thermodynamics_semantic_information_dictionary_v0_1.md#ic-49), [IC-64](thermodynamics_semantic_information_dictionary_v0_1.md#ic-64).

**Preconditions:** Location hierarchy, precedence and allowed overlays are explicit; shared package revisions are captured.

**Procedure and information flow:**

1. P07 resolves the direct/default/inherited assignment and records its provenance.
2. Detect hierarchy cycles, conflicting assignments and disallowed specializations.
3. Assess adjacent connections as heat-only, same-material or representation boundaries.
4. Propose a binding revision for AC-47/48; later calls read this effective binding, not the last active provider.

**Successful outputs:** [IC-48](thermodynamics_semantic_information_dictionary_v0_1.md#ic-48), [IC-49](thermodynamics_semantic_information_dictionary_v0_1.md#ic-49). Discoverable location-specific package and physical-overlay semantics independent of calculation order.

**Permitted effects:** Only private proposals, snapshots and assessments may be built; no approved model/result mutation.

**Adoption/completion boundary:** A proposal becomes a current model definition only through AC-47/48; it conveys no numerical acceptance.

**Failure and recovery:** Ambiguous inheritance or incompatible neighboring material descriptions blocks affected execution. A drawing-only move does not alter this binding.

**Repeated delivery/re-execution:** Resolve repeatedly against the same hierarchy revision; edits require recapture and current-binding adoption.

**AW-19-P:** Each exchanger side retains its package through interleaved requests.
**AW-19-N:** A stage never uses the package merely left active by another stage.

**Primary B4 requirements:** [FR-CFG-06](thermodynamics_functional_requirements_v0_1.md#fr-cfg-06). **Supporting requirements:** None.

**Common action rules:** [AR-17](#ar-17). **Handoff facts:** EV-04

**Evidence status:** Action specified. Its positive/negative witnesses have not been executed against the simulator or a thermodynamic provider. Primary ownership is inherited from B5; information and invariants are inherited from the linked B6 concepts.

<a id="ac-20"></a>
### AC-20. Declare unit demands, constraints and completion scope

**Accountable owner:** P07, Flowsheet integration and use-case coordination. **Action kind:** prepare.

**Initiator/trigger:** A unit-model author or coordinator selects an operating mode or prepares a process run.

**Inputs:** [IC-34](thermodynamics_semantic_information_dictionary_v0_1.md#ic-34), [IC-38](thermodynamics_semantic_information_dictionary_v0_1.md#ic-38), [IC-39](thermodynamics_semantic_information_dictionary_v0_1.md#ic-39), [IC-40](thermodynamics_semantic_information_dictionary_v0_1.md#ic-40), [IC-41](thermodynamics_semantic_information_dictionary_v0_1.md#ic-41), [IC-48](thermodynamics_semantic_information_dictionary_v0_1.md#ic-48), [IC-49](thermodynamics_semantic_information_dictionary_v0_1.md#ic-49), [IC-50](thermodynamics_semantic_information_dictionary_v0_1.md#ic-50), [IC-52](thermodynamics_semantic_information_dictionary_v0_1.md#ic-52).

**Preconditions:** Unit mode, input/internal/output locations and process equations are supplied by the host model.

**Procedure and information flow:**

1. P07 enumerates mandatory, conditional and optional property/contribution demands by location.
2. Export pressure, heat, work, efficiency, transfer and routing assumptions without moving equipment geometry into a property package.
3. Define parent/child completion scopes and coherent publication group.
4. Establish which residuals the host must return for unit or flowsheet acceptance.

**Successful outputs:** [IC-50](thermodynamics_semantic_information_dictionary_v0_1.md#ic-50), [IC-52](thermodynamics_semantic_information_dictionary_v0_1.md#ic-52). A demand and completion contract usable for readiness, local problems and later group checks.

**Permitted effects:** Only private proposals, snapshots and assessments may be built; no approved model/result mutation.

**Adoption/completion boundary:** A proposal becomes a current model definition only through AC-47/48; it conveys no numerical acceptance.

**Failure and recovery:** Missing mandatory data identifies the exact unit/location. Downgrading a rating mode to a duty-only mode is an explicitly different unit selection.

**Repeated delivery/re-execution:** Changing only guesses does not change physical mode. Changing unit laws or demands creates a new declaration/context.

**AW-20-P:** Duty-only heating can require enthalpy while rating adds transport and geometry-dependent demands.
**AW-20-N:** A successful enthalpy call cannot certify exchanger rating when viscosity is missing.

**Primary B4 requirements:** [FR-FLW-01](thermodynamics_functional_requirements_v0_1.md#fr-flw-01). **Supporting requirements:** None.

**Common action rules:** None. **Handoff facts:** Local proposal/candidate handoff under the common contract.

**Evidence status:** Action specified. Its positive/negative witnesses have not been executed against the simulator or a thermodynamic provider. Primary ownership is inherited from B5; information and invariants are inherited from the linked B6 concepts.

<a id="ac-21"></a>
### AC-21. Capture and assess the original physical calculation problem

**Accountable owner:** P05, Thermodynamic problems and operation contracts. **Action kind:** prepare.

**Initiator/trigger:** A caller requests complete state resolution or an intentionally partial coupled contribution.

**Inputs:** [IC-20](thermodynamics_semantic_information_dictionary_v0_1.md#ic-20), [IC-24](thermodynamics_semantic_information_dictionary_v0_1.md#ic-24), [IC-28](thermodynamics_semantic_information_dictionary_v0_1.md#ic-28), [IC-34](thermodynamics_semantic_information_dictionary_v0_1.md#ic-34), [IC-36](thermodynamics_semantic_information_dictionary_v0_1.md#ic-36), [IC-37](thermodynamics_semantic_information_dictionary_v0_1.md#ic-37), [IC-38](thermodynamics_semantic_information_dictionary_v0_1.md#ic-38), [IC-41](thermodynamics_semantic_information_dictionary_v0_1.md#ic-41), [IC-42](thermodynamics_semantic_information_dictionary_v0_1.md#ic-42), [IC-45](thermodynamics_semantic_information_dictionary_v0_1.md#ic-45), [IC-46](thermodynamics_semantic_information_dictionary_v0_1.md#ic-46), [IC-49](thermodynamics_semantic_information_dictionary_v0_1.md#ic-49), [IC-50](thermodynamics_semantic_information_dictionary_v0_1.md#ic-50), [IC-64](thermodynamics_semantic_information_dictionary_v0_1.md#ic-64).

**Preconditions:** Input roles, effective binding, required output scope and physical authority are supplied. Completeness or nonlinear uniqueness may be explicitly unassessed.

**Procedure and information flow:**

1. P05 binds quantity/constraint identities to one coherent captured context.
2. Separate observations and guesses from fixed equations.
3. Check authority conflicts and assessed independence; a partial coupled problem is not automatically invalid.
4. Freeze original targets, allowed changes, conditional demands and predeclared acceptance criteria.
5. Route to the appropriate specialization AC-22..26 or return missing/conflicting information.

**Successful outputs:** [IC-34](thermodynamics_semantic_information_dictionary_v0_1.md#ic-34), [IC-35](thermodynamics_semantic_information_dictionary_v0_1.md#ic-35), [IC-36](thermodynamics_semantic_information_dictionary_v0_1.md#ic-36), [IC-37](thermodynamics_semantic_information_dictionary_v0_1.md#ic-37), [IC-41](thermodynamics_semantic_information_dictionary_v0_1.md#ic-41), [IC-42](thermodynamics_semantic_information_dictionary_v0_1.md#ic-42). One immutable original problem or a precise incomplete/conflict assessment. Request preparation does not create execution or publication authority.

**Permitted effects:** Only private proposals, snapshots and assessments may be built; no approved model/result mutation.

**Adoption/completion boundary:** A proposal becomes a current model definition only through AC-47/48; it conveys no numerical acceptance.

**Failure and recovery:** Conflicting fixed split and unrestricted flash, incompatible TP/H, or unresolved required meaning blocks complete resolution. Do not silently discard an observation or constraint.

**Repeated delivery/re-execution:** Same physical request can have multiple attempts, but redelivery is not permission to alter targets. Changed physical constraints create a new IC-35.

**AW-21-P:** A partial column state remains legitimate for coupled solving and cannot be presented as fully resolved.
**AW-21-N:** A saturated pure-fluid TP request cannot invent an intermediate vapor fraction.

**Primary B4 requirements:** [FR-STA-02](thermodynamics_functional_requirements_v0_1.md#fr-sta-02), [FR-STA-03](thermodynamics_functional_requirements_v0_1.md#fr-sta-03). **Supporting requirements:** None.

**Common action rules:** [AR-01](#ar-01), [AR-03](#ar-03), [AR-04](#ar-04), [AR-07](#ar-07), [AR-10](#ar-10). **Handoff facts:** EV-05

**Evidence status:** Action specified. Its positive/negative witnesses have not been executed against the simulator or a thermodynamic provider. Primary ownership is inherited from B5; information and invariants are inherited from the linked B6 concepts.

<a id="ac-22"></a>
### AC-22. Prepare property-only, transport and effective-property operations

**Accountable owner:** P05, Thermodynamic problems and operation contracts. **Action kind:** specialize.

**Initiator/trigger:** A unit requests properties of supplied bulk phases, interfaces or an effective mixture.

**Inputs:** [IC-20](thermodynamics_semantic_information_dictionary_v0_1.md#ic-20), [IC-28](thermodynamics_semantic_information_dictionary_v0_1.md#ic-28), [IC-29](thermodynamics_semantic_information_dictionary_v0_1.md#ic-29), [IC-34](thermodynamics_semantic_information_dictionary_v0_1.md#ic-34), [IC-35](thermodynamics_semantic_information_dictionary_v0_1.md#ic-35), [IC-37](thermodynamics_semantic_information_dictionary_v0_1.md#ic-37), [IC-38](thermodynamics_semantic_information_dictionary_v0_1.md#ic-38), [IC-41](thermodynamics_semantic_information_dictionary_v0_1.md#ic-41), [IC-50](thermodynamics_semantic_information_dictionary_v0_1.md#ic-50).

**Preconditions:** Subject, state coordinates, pair ordering, diffusion frame where relevant, closure and mandatory/optional demand roles are stated.

**Procedure and information flow:**

1. P05 permits only specified dependent-coordinate resolution, such as density/root, while preserving protected quantities.
2. Name effective-property closures separately from additive quantities and equipment transport laws.
3. Specify per-property postconditions and conditional dependencies.
4. P07 invokes the shared execution/qualification path; any missing required property blocks only its actual dependent acceptance scope.

**Successful outputs:** [IC-35](thermodynamics_semantic_information_dictionary_v0_1.md#ic-35), [IC-38](thermodynamics_semantic_information_dictionary_v0_1.md#ic-38), [IC-41](thermodynamics_semantic_information_dictionary_v0_1.md#ic-41). An operation whose returned fields and authority are predictable; equilibrium success carries no all-properties-populated promise.

**Permitted effects:** Construct or interpret operation-specific contracts using the shared original problem; no provider or live material mutation.

**Adoption/completion boundary:** Use AC-29..33 for numerics and AC-36/37/49 for checked publication; unsupported modes return precise outcomes.

**Failure and recovery:** Unrequested phase/species redistribution fails authority. Missing closure, diffusivity or required phase-pair meaning is not replaced with a convenient proxy.

**Repeated delivery/re-execution:** Repeated evaluations can create independent candidates or qualified reuse links; they never accumulate material or overwrite external allocations.

**AW-22-P:** An external two-liquid split receives enthalpy/viscosity values with unchanged protected allocation.
**AW-22-N:** An optional-property omission cannot be used to excuse a missing mandatory energy quantity.

**Primary B4 requirements:** [FR-EQL-05](thermodynamics_functional_requirements_v0_1.md#fr-eql-05), [FR-PRP-01](thermodynamics_functional_requirements_v0_1.md#fr-prp-01), [FR-PRP-02](thermodynamics_functional_requirements_v0_1.md#fr-prp-02), [FR-PRP-03](thermodynamics_functional_requirements_v0_1.md#fr-prp-03), [FR-PRP-04](thermodynamics_functional_requirements_v0_1.md#fr-prp-04). **Supporting requirements:** None.

**Common action rules:** [AR-03](#ar-03), [AR-13](#ar-13). **Handoff facts:** EV-05

**Evidence status:** Action specified. Its positive/negative witnesses have not been executed against the simulator or a thermodynamic provider. Primary ownership is inherited from B5; information and invariants are inherited from the linked B6 concepts.

<a id="ac-23"></a>
### AC-23. Prepare TP, PH, PS, saturation, stability and bounded phase resolution

**Accountable owner:** P05, Thermodynamic problems and operation contracts. **Action kind:** specialize.

**Initiator/trigger:** A stream/unit requests a supported state or branch operation, including a deferred inventory/restricted-state walkthrough.

**Inputs:** [IC-20](thermodynamics_semantic_information_dictionary_v0_1.md#ic-20), [IC-21](thermodynamics_semantic_information_dictionary_v0_1.md#ic-21), [IC-22](thermodynamics_semantic_information_dictionary_v0_1.md#ic-22), [IC-28](thermodynamics_semantic_information_dictionary_v0_1.md#ic-28), [IC-34](thermodynamics_semantic_information_dictionary_v0_1.md#ic-34), [IC-35](thermodynamics_semantic_information_dictionary_v0_1.md#ic-35), [IC-36](thermodynamics_semantic_information_dictionary_v0_1.md#ic-36), [IC-37](thermodynamics_semantic_information_dictionary_v0_1.md#ic-37), [IC-38](thermodynamics_semantic_information_dictionary_v0_1.md#ic-38), [IC-41](thermodynamics_semantic_information_dictionary_v0_1.md#ic-41), [IC-42](thermodynamics_semantic_information_dictionary_v0_1.md#ic-42), [IC-57](thermodynamics_semantic_information_dictionary_v0_1.md#ic-57).

**Preconditions:** Specification pair and amount/reference basis, fixed or reacting chemistry, permitted phase scope and branch/initializer prerequisites are explicit.

**Procedure and information flow:**

1. P05 uses the variant contracts in Section 4 to preserve original physical constraints.
2. Distinguish saturation endpoints, specified quality and underdetermined allocation.
3. Preserve physical, structural and searched phase sets separately.
4. Require target-property residuals for inverse calculations and qualify requested stability evidence.
5. P07/P08 realize only an eligible route; P09 checks its promised scope.

**Successful outputs:** [IC-35](thermodynamics_semantic_information_dictionary_v0_1.md#ic-35), [IC-36](thermodynamics_semantic_information_dictionary_v0_1.md#ic-36), [IC-41](thermodynamics_semantic_information_dictionary_v0_1.md#ic-41), [IC-42](thermodynamics_semantic_information_dictionary_v0_1.md#ic-42). A supported state problem with explicit target, phase/stability and property-completion obligations; restricted results remain restricted.

**Permitted effects:** Construct or interpret operation-specific contracts using the shared original problem; no provider or live material mutation.

**Adoption/completion boundary:** Use AC-29..33 for numerics and AC-36/37/49 for checked publication; unsupported modes return precise outcomes.

**Failure and recovery:** An unavailable initializer is not proof of infeasibility. VLE-only output cannot satisfy required VLLE. A bound-stopped inverse solve with wrong H/S fails target acceptance.

**Repeated delivery/re-execution:** Algorithm retries retain the original target; an authorized phase restriction branches via AC-54. Warm starts are numerical evidence, not authoritative specifications.

**AW-23-P:** PH/PS targets are rechecked at the returned state and a compressor reference PS state remains distinct from its actual outlet.
**AW-23-N:** No-flash or ideal fallback cannot pass an unrestricted original multiphase request under its old identity.

**Primary B4 requirements:** [FR-EQL-01](thermodynamics_functional_requirements_v0_1.md#fr-eql-01), [FR-EQL-02](thermodynamics_functional_requirements_v0_1.md#fr-eql-02), [FR-EQL-03](thermodynamics_functional_requirements_v0_1.md#fr-eql-03), [FR-EQL-04](thermodynamics_functional_requirements_v0_1.md#fr-eql-04), [FR-EQL-06](thermodynamics_functional_requirements_v0_1.md#fr-eql-06), [FR-EQL-07](thermodynamics_functional_requirements_v0_1.md#fr-eql-07), [FR-EQL-08](thermodynamics_functional_requirements_v0_1.md#fr-eql-08), [FR-EXT-04](thermodynamics_functional_requirements_v0_1.md#fr-ext-04). **Supporting requirements:** None.

**Common action rules:** [AR-03](#ar-03), [AR-04](#ar-04). **Handoff facts:** EV-05

**Evidence status:** Action specified. Its positive/negative witnesses have not been executed against the simulator or a thermodynamic provider. Primary ownership is inherited from B5; information and invariants are inherited from the linked B6 concepts.

<a id="ac-24"></a>
### AC-24. Prepare a coupled chemistry, phase and thermal problem

**Accountable owner:** P05, Thermodynamic problems and operation contracts. **Action kind:** specialize.

**Initiator/trigger:** An equilibrium reactor, speciation or reactive-separation model requests chemical transformation.

**Inputs:** [IC-20](thermodynamics_semantic_information_dictionary_v0_1.md#ic-20), [IC-21](thermodynamics_semantic_information_dictionary_v0_1.md#ic-21), [IC-28](thermodynamics_semantic_information_dictionary_v0_1.md#ic-28), [IC-35](thermodynamics_semantic_information_dictionary_v0_1.md#ic-35), [IC-36](thermodynamics_semantic_information_dictionary_v0_1.md#ic-36), [IC-37](thermodynamics_semantic_information_dictionary_v0_1.md#ic-37), [IC-41](thermodynamics_semantic_information_dictionary_v0_1.md#ic-41), [IC-42](thermodynamics_semantic_information_dictionary_v0_1.md#ic-42), [IC-43](thermodynamics_semantic_information_dictionary_v0_1.md#ic-43), [IC-44](thermodynamics_semantic_information_dictionary_v0_1.md#ic-44), [IC-45](thermodynamics_semantic_information_dictionary_v0_1.md#ic-45), [IC-46](thermodynamics_semantic_information_dictionary_v0_1.md#ic-46), [IC-47](thermodynamics_semantic_information_dictionary_v0_1.md#ic-47), [IC-50](thermodynamics_semantic_information_dictionary_v0_1.md#ic-50).

**Preconditions:** Coherent chemical system, justified conserved totals, phase participation, thermal constraints and actual reservoir permissions are defined.

**Procedure and information flow:**

1. P05 combines required chemical, phase and thermal conditions in one original problem.
2. Distinguish equilibrium, finite-rate and frozen subsets and chosen coupling approximation.
3. Include unknown reservoir amounts only where an explicit exchange is authorized.
4. P07 can use one coherent engine or an explicitly converged/approximate split route.
5. Require combined residual checks after all substeps, not only independent substep success.

**Successful outputs:** [IC-35](thermodynamics_semantic_information_dictionary_v0_1.md#ic-35), [IC-36](thermodynamics_semantic_information_dictionary_v0_1.md#ic-36), [IC-37](thermodynamics_semantic_information_dictionary_v0_1.md#ic-37), [IC-41](thermodynamics_semantic_information_dictionary_v0_1.md#ic-41). One identifiable coupled physical question; output exchanges and changed species/phase quantities have explicit authority.

**Permitted effects:** Construct or interpret operation-specific contracts using the shared original problem; no provider or live material mutation.

**Adoption/completion boundary:** Use AC-29..33 for numerics and AC-36/37/49 for checked publication; unsupported modes return precise outcomes.

**Failure and recovery:** Speciation without necessary caloric data cannot pass an energy-balanced reactive operation. Sequential steps invalidating each other remain unconverged or explicitly approximate.

**Repeated delivery/re-execution:** A chemistry solve is evaluated from captured inputs; repeated substeps are iterates, not repeated physical reactions or reservoir additions.

**AW-24-P:** A chemically reactive mineral/aqueous fixture exposes all changed species and authorized exchanges for combined checks.
**AW-24-N:** A fixed-species VLE call cannot register as a chemically equilibrated reactor result.

**Primary B4 requirements:** [FR-CHM-03](thermodynamics_functional_requirements_v0_1.md#fr-chm-03). **Supporting requirements:** None.

**Common action rules:** [AR-03](#ar-03), [AR-15](#ar-15). **Handoff facts:** EV-05

**Evidence status:** Action specified. Its positive/negative witnesses have not been executed against the simulator or a thermodynamic provider. Primary ownership is inherited from B5; information and invariants are inherited from the linked B6 concepts.

<a id="ac-25"></a>
### AC-25. Prepare and assess the meaning of a derivative experiment

**Accountable owner:** P05, Thermodynamic problems and operation contracts. **Action kind:** specialize.

**Initiator/trigger:** A process solver, sensitivity study or fitting use case requests derivatives.

**Inputs:** [IC-04](thermodynamics_semantic_information_dictionary_v0_1.md#ic-04), [IC-07](thermodynamics_semantic_information_dictionary_v0_1.md#ic-07), [IC-20](thermodynamics_semantic_information_dictionary_v0_1.md#ic-20), [IC-28](thermodynamics_semantic_information_dictionary_v0_1.md#ic-28), [IC-35](thermodynamics_semantic_information_dictionary_v0_1.md#ic-35), [IC-36](thermodynamics_semantic_information_dictionary_v0_1.md#ic-36), [IC-38](thermodynamics_semantic_information_dictionary_v0_1.md#ic-38), [IC-39](thermodynamics_semantic_information_dictionary_v0_1.md#ic-39), [IC-41](thermodynamics_semantic_information_dictionary_v0_1.md#ic-41), [IC-42](thermodynamics_semantic_information_dictionary_v0_1.md#ic-42), [IC-72](thermodynamics_semantic_information_dictionary_v0_1.md#ic-72).

**Preconditions:** Output/input observables, independent charts, held-fixed variables, phase/chemistry response, order and branch demands are stated.

**Procedure and information flow:**

1. P05 checks that input directions respect coordinate constraints and reference transformations.
2. Declare local, path or re-solved sensitivity meaning separately from derivative implementation.
3. Specify required branch/conditioning and verification evidence.
4. P08 selects an actually eligible analytic, automatic, implicit or finite-difference realization.
5. P09 rejects incompatible coordinate or transition evidence rather than asserting smoothness.

**Successful outputs:** [IC-35](thermodynamics_semantic_information_dictionary_v0_1.md#ic-35), [IC-39](thermodynamics_semantic_information_dictionary_v0_1.md#ic-39), [IC-41](thermodynamics_semantic_information_dictionary_v0_1.md#ic-41). A fully interpreted derivative request and qualified tensor/unsupported outcome through shared execution.

**Permitted effects:** Construct or interpret operation-specific contracts using the shared original problem; no provider or live material mutation.

**Adoption/completion boundary:** Use AC-29..33 for numerics and AC-36/37/49 for checked publication; unsupported modes return precise outcomes.

**Failure and recovery:** Missing derivative meaning blocks computation. An unsupported higher order is not inferred from first-order support; transition-crossing perturbations cannot validate an unqualified homogeneous derivative.

**Repeated delivery/re-execution:** Perturbed evaluations are separate trial problems/contexts linked to the derivative experiment; they do not edit the base physical state.

**AW-25-P:** A constrained composition-chart derivative maps all dependent coordinates.
**AW-25-N:** A frozen-phase derivative cannot silently satisfy a re-equilibrated total-response demand.

**Primary B4 requirements:** [FR-PRP-05](thermodynamics_functional_requirements_v0_1.md#fr-prp-05), [FR-PRP-06](thermodynamics_functional_requirements_v0_1.md#fr-prp-06). **Supporting requirements:** None.

**Common action rules:** None. **Handoff facts:** EV-05

**Evidence status:** Action specified. Its positive/negative witnesses have not been executed against the simulator or a thermodynamic provider. Primary ownership is inherited from B5; information and invariants are inherited from the linked B6 concepts.

<a id="ac-26"></a>
### AC-26. Define thermodynamic contributions and restoration obligations

**Accountable owner:** P05, Thermodynamic problems and operation contracts. **Action kind:** specialize.

**Initiator/trigger:** A coupled unit or equation compiler requests thermodynamic relationships rather than a completed standalone flash.

**Inputs:** [IC-04](thermodynamics_semantic_information_dictionary_v0_1.md#ic-04), [IC-20](thermodynamics_semantic_information_dictionary_v0_1.md#ic-20), [IC-28](thermodynamics_semantic_information_dictionary_v0_1.md#ic-28), [IC-35](thermodynamics_semantic_information_dictionary_v0_1.md#ic-35), [IC-36](thermodynamics_semantic_information_dictionary_v0_1.md#ic-36), [IC-37](thermodynamics_semantic_information_dictionary_v0_1.md#ic-37), [IC-39](thermodynamics_semantic_information_dictionary_v0_1.md#ic-39), [IC-40](thermodynamics_semantic_information_dictionary_v0_1.md#ic-40), [IC-41](thermodynamics_semantic_information_dictionary_v0_1.md#ic-41), [IC-50](thermodynamics_semantic_information_dictionary_v0_1.md#ic-50), [IC-57](thermodynamics_semantic_information_dictionary_v0_1.md#ic-57).

**Preconditions:** Host/local variable bindings, property demands, units, physical responsibility and supported contribution kinds are supplied.

**Procedure and information flow:**

1. P05 identifies quantities consumed and relationships supplied as values, residuals, equations or qualified callbacks.
2. Distinguish structural dependency from numerically observed zeros and label derivative/scaling semantics.
3. Record temporary fixing/relaxation and the original model restoration obligations.
4. P07 connects the contribution to the host formulation without adding conflicting duplicate constraints.

**Successful outputs:** [IC-35](thermodynamics_semantic_information_dictionary_v0_1.md#ic-35), [IC-40](thermodynamics_semantic_information_dictionary_v0_1.md#ic-40), [IC-41](thermodynamics_semantic_information_dictionary_v0_1.md#ic-41). A bounded mathematical contribution description and required evidence extraction/restoration checks; no universal symbolic API is assumed.

**Permitted effects:** Construct or interpret operation-specific contracts using the shared original problem; no provider or live material mutation.

**Adoption/completion boundary:** Use AC-29..33 for numerics and AC-36/37/49 for checked publication; unsupported modes return precise outcomes.

**Failure and recovery:** A value-only backend cannot satisfy an unqualified symbolic/Hessian demand. Remaining temporary fixing prevents acceptance of the original formulation.

**Repeated delivery/re-execution:** Construction reuse requires identical variable bindings and definition revisions; evaluation uses current trial coordinates only within its captured run.

**AW-26-P:** Stage enthalpy and equilibrium relationships participate while the column retains interstage balances.
**AW-26-N:** A temporary temperature fixation cannot remain unnoticed in the accepted original column problem.

**Primary B4 requirements:** None. **Supporting requirements:** [FR-FLW-07](thermodynamics_functional_requirements_v0_1.md#fr-flw-07), [FR-RUN-02](thermodynamics_functional_requirements_v0_1.md#fr-run-02), [FR-PRP-05](thermodynamics_functional_requirements_v0_1.md#fr-prp-05).

**Common action rules:** None. **Handoff facts:** EV-05

**Evidence status:** Action specified. Its positive/negative witnesses have not been executed against the simulator or a thermodynamic provider. Primary ownership is inherited from B5; information and invariants are inherited from the linked B6 concepts.

<a id="ac-27"></a>
### AC-27. Register a provider realization and verify dependency availability

**Accountable owner:** P08, Provider realization and numerical execution. **Action kind:** prepare.

**Initiator/trigger:** An integrator registers an exact engine/binding or a loader checks a saved dependency manifest.

**Inputs:** [IC-10](thermodynamics_semantic_information_dictionary_v0_1.md#ic-10), [IC-14](thermodynamics_semantic_information_dictionary_v0_1.md#ic-14), [IC-18](thermodynamics_semantic_information_dictionary_v0_1.md#ic-18), [IC-20](thermodynamics_semantic_information_dictionary_v0_1.md#ic-20), [IC-34](thermodynamics_semantic_information_dictionary_v0_1.md#ic-34), [IC-53](thermodynamics_semantic_information_dictionary_v0_1.md#ic-53), [IC-58](thermodynamics_semantic_information_dictionary_v0_1.md#ic-58), [IC-68](thermodynamics_semantic_information_dictionary_v0_1.md#ic-68).

**Preconditions:** Actual artifact/build/binding/data identities, exposed entry points and known lifecycle restrictions are supplied.

**Procedure and information flow:**

1. P08 distinguishes available wrapper, available engine and required data/module resources.
2. Describe actual operation predicates and mapping boundaries, not the upstream feature catalog.
3. Record unsupported and unassessed combinations and opaque internal choices.
4. Return availability/capability attestations to P03 through P07; preserve inspectability when execution is blocked.

**Successful outputs:** [IC-53](thermodynamics_semantic_information_dictionary_v0_1.md#ic-53), [IC-58](thermodynamics_semantic_information_dictionary_v0_1.md#ic-58). A bounded realization record that supports whole-package or qualified submodel use without inherited blanket claims.

**Permitted effects:** Only private proposals, snapshots and assessments may be built; no approved model/result mutation.

**Adoption/completion boundary:** A proposal becomes a current model definition only through AC-47/48; it conveys no numerical acceptance.

**Failure and recovery:** A missing required engine or data resource prevents affected execution. Presence of a solids parameter in a rejecting method is not capability evidence.

**Repeated delivery/re-execution:** Refreshing an availability assessment does not change historic realization identity. A build/data change creates a new realization/reassessment.

**AW-27-P:** A TP-only adapter exposes its restriction even if the upstream library supports PH elsewhere.
**AW-27-N:** An installed wrapper without its required engine cannot mark a saved case executable.

**Primary B4 requirements:** [FR-GOV-02](thermodynamics_functional_requirements_v0_1.md#fr-gov-02), [FR-GOV-03](thermodynamics_functional_requirements_v0_1.md#fr-gov-03). **Supporting requirements:** None.

**Common action rules:** [AR-18](#ar-18). **Handoff facts:** Local proposal/candidate handoff under the common contract.

**Evidence status:** Action specified. Its positive/negative witnesses have not been executed against the simulator or a thermodynamic provider. Primary ownership is inherited from B5; information and invariants are inherited from the linked B6 concepts.

<a id="ac-28"></a>
### AC-28. Bind native coordinates and observable conventions

**Accountable owner:** P08, Provider realization and numerical execution. **Action kind:** prepare.

**Initiator/trigger:** An adapter prepares data exchange with an exact provider realization.

**Inputs:** [IC-03](thermodynamics_semantic_information_dictionary_v0_1.md#ic-03), [IC-04](thermodynamics_semantic_information_dictionary_v0_1.md#ic-04), [IC-07](thermodynamics_semantic_information_dictionary_v0_1.md#ic-07), [IC-20](thermodynamics_semantic_information_dictionary_v0_1.md#ic-20), [IC-21](thermodynamics_semantic_information_dictionary_v0_1.md#ic-21), [IC-35](thermodynamics_semantic_information_dictionary_v0_1.md#ic-35), [IC-38](thermodynamics_semantic_information_dictionary_v0_1.md#ic-38), [IC-39](thermodynamics_semantic_information_dictionary_v0_1.md#ic-39), [IC-53](thermodynamics_semantic_information_dictionary_v0_1.md#ic-53).

**Preconditions:** Host constituent/phase meanings and native order, units, references, tensor axes and placeholder semantics are identified.

**Procedure and information flow:**

1. P08 constructs direction-specific coordinate and convention maps.
2. Validate all affected composition, parameter, property and derivative axes.
3. Identify lossy/unsupported directions; do not demand a fabricated inverse.
4. Record native sentinel and phase-slot meaning without promoting placeholders to material.
5. Bind the mapping to exact source/target definitions and provider version.

**Successful outputs:** [IC-58](thermodynamics_semantic_information_dictionary_v0_1.md#ic-58). An explicit provider binding usable by execution and candidate normalization.

**Permitted effects:** Only private proposals, snapshots and assessments may be built; no approved model/result mutation.

**Adoption/completion boundary:** A proposal becomes a current model definition only through AC-47/48; it conveys no numerical acceptance.

**Failure and recovery:** Unmapped constituent, incompatible reference or unknown axis blocks the affected operation; partial maps are not applied to complete-state requests.

**Repeated delivery/re-execution:** Reuse only with compatible host/native revisions. A changed native ordering invalidates the old map even if component names appear unchanged.

**AW-28-P:** Permuting composition also permutes the corresponding parameter and derivative coordinates.
**AW-28-N:** A remapped composition with an old-order Jacobian is rejected before process acceptance.

**Primary B4 requirements:** None. **Supporting requirements:** [FR-MAT-07](thermodynamics_functional_requirements_v0_1.md#fr-mat-07), [FR-RUN-04](thermodynamics_functional_requirements_v0_1.md#fr-run-04).

**Common action rules:** None. **Handoff facts:** Local proposal/candidate handoff under the common contract.

**Evidence status:** Action specified. Its positive/negative witnesses have not been executed against the simulator or a thermodynamic provider. Primary ownership is inherited from B5; information and invariants are inherited from the linked B6 concepts.

<a id="ac-29"></a>
### AC-29. Plan a bounded execution attempt

**Accountable owner:** P08, Provider realization and numerical execution. **Action kind:** execute.

**Initiator/trigger:** P07 dispatches an assessed original problem for a production attempt or explicitly scoped evidence-gathering trial.

**Inputs:** [IC-24](thermodynamics_semantic_information_dictionary_v0_1.md#ic-24), [IC-35](thermodynamics_semantic_information_dictionary_v0_1.md#ic-35), [IC-41](thermodynamics_semantic_information_dictionary_v0_1.md#ic-41), [IC-53](thermodynamics_semantic_information_dictionary_v0_1.md#ic-53), [IC-57](thermodynamics_semantic_information_dictionary_v0_1.md#ic-57), [IC-58](thermodynamics_semantic_information_dictionary_v0_1.md#ic-58), [IC-64](thermodynamics_semantic_information_dictionary_v0_1.md#ic-64), [IC-70](thermodynamics_semantic_information_dictionary_v0_1.md#ic-70), [IC-72](thermodynamics_semantic_information_dictionary_v0_1.md#ic-72).

**Preconditions:** Captured run/dependency context, realization and numerical policy are available; required execution prerequisites must be met before entering provider execution.

**Procedure and information flow:**

1. P08 selects an eligible algorithm/contribution route and records its termination, precision, resource and retry policy.
2. Preserve original physical problem identity and any trial-only limitation.
3. Identify initializer/session prerequisites and required output/check evidence.
4. Allocate a distinct attempt identity before numerical work; duplicate delivery is reconciled by identity.

**Successful outputs:** [IC-54](thermodynamics_semantic_information_dictionary_v0_1.md#ic-54), [IC-72](thermodynamics_semantic_information_dictionary_v0_1.md#ic-72). A planned attempt linked to one original or explicitly alternative problem, not a changed request.

**Permitted effects:** Private provider/session work and execution evidence only; no canonical definition or current-result writes.

**Adoption/completion boundary:** A candidate remains unaccepted until AC-36/37; uncertain context is handled by AC-35.

**Failure and recovery:** Blocked readiness yields a pre-execution outcome. An unassessed combination can be probed only under explicit trial policy, without a production capability claim.

**Repeated delivery/re-execution:** Resubmitting the same attempt command cannot unknowingly launch another realization. An intentional repeat/retry gets a distinct linked attempt.

**AW-29-P:** Two algorithms can attempt one fixed PH problem with separate evidence.
**AW-29-N:** A retry plan cannot replace the target with the previous candidate value.

**Primary B4 requirements:** None. **Supporting requirements:** [FR-CFG-03](thermodynamics_functional_requirements_v0_1.md#fr-cfg-03), [FR-RUN-05](thermodynamics_functional_requirements_v0_1.md#fr-run-05), [FR-RES-07](thermodynamics_functional_requirements_v0_1.md#fr-res-07).

**Common action rules:** [AR-01](#ar-01). **Handoff facts:** Local proposal/candidate handoff under the common contract.

**Evidence status:** Action specified. Its positive/negative witnesses have not been executed against the simulator or a thermodynamic provider. Primary ownership is inherited from B5; information and invariants are inherited from the linked B6 concepts.

<a id="ac-30"></a>
### AC-30. Acquire a compatible, isolated numerical session

**Accountable owner:** P08, Provider realization and numerical execution. **Action kind:** execute.

**Initiator/trigger:** An attempt needs a fresh or reusable provider context.

**Inputs:** [IC-20](thermodynamics_semantic_information_dictionary_v0_1.md#ic-20), [IC-53](thermodynamics_semantic_information_dictionary_v0_1.md#ic-53), [IC-54](thermodynamics_semantic_information_dictionary_v0_1.md#ic-54), [IC-55](thermodynamics_semantic_information_dictionary_v0_1.md#ic-55), [IC-58](thermodynamics_semantic_information_dictionary_v0_1.md#ic-58), [IC-64](thermodynamics_semantic_information_dictionary_v0_1.md#ic-64), [IC-70](thermodynamics_semantic_information_dictionary_v0_1.md#ic-70).

**Preconditions:** Exact configuration, coordinates, references and realization compatibility are known; sharing and containment policy are declared.

**Procedure and information flow:**

1. P08 checks health and compatibility before leasing or constructing a context.
2. Isolate the entire activation, input installation, calculation and readback sequence, not only the final function call.
3. Reserve the context under the declared exclusive/serialized/reentrant policy.
4. On changed configuration rebuild or prove compatibility; no state from a previous material may become current implicitly.

**Successful outputs:** [IC-55](thermodynamics_semantic_information_dictionary_v0_1.md#ic-55). A private or properly isolated workspace associated with the attempt; accepted host state remains detached.

**Permitted effects:** Private provider/session work and execution evidence only; no canonical definition or current-result writes.

**Adoption/completion boundary:** A candidate remains unaccepted until AC-36/37; uncertain context is handled by AC-35.

**Failure and recovery:** Unknown safe sharing blocks concurrent reuse and requires conservative serialization/isolation. Construction failure returns dependency/provider failure without touching accepted results.

**Repeated delivery/re-execution:** A repeated lease request is reconciled with its existing attempt. Reusing a context never reuses old run-publication authority.

**AW-30-P:** Interleaved materials retain separate configuration and output interpretation under the declared policy.
**AW-30-N:** A changed caloric reference cannot reuse an old native session without reconstruction or compatibility evidence.

**Primary B4 requirements:** [FR-RUN-03](thermodynamics_functional_requirements_v0_1.md#fr-run-03), [FR-RUN-04](thermodynamics_functional_requirements_v0_1.md#fr-run-04). **Supporting requirements:** None.

**Common action rules:** [AR-08](#ar-08). **Handoff facts:** Local proposal/candidate handoff under the common contract.

**Evidence status:** Action specified. Its positive/negative witnesses have not been executed against the simulator or a thermodynamic provider. Primary ownership is inherited from B5; information and invariants are inherited from the linked B6 concepts.

<a id="ac-31"></a>
### AC-31. Initialize and restore the original calculation formulation

**Accountable owner:** P08, Provider realization and numerical execution. **Action kind:** execute.

**Initiator/trigger:** An eligible operation requires a seed, continuation state or staged equation initialization.

**Inputs:** [IC-24](thermodynamics_semantic_information_dictionary_v0_1.md#ic-24), [IC-28](thermodynamics_semantic_information_dictionary_v0_1.md#ic-28), [IC-35](thermodynamics_semantic_information_dictionary_v0_1.md#ic-35), [IC-36](thermodynamics_semantic_information_dictionary_v0_1.md#ic-36), [IC-40](thermodynamics_semantic_information_dictionary_v0_1.md#ic-40), [IC-41](thermodynamics_semantic_information_dictionary_v0_1.md#ic-41), [IC-54](thermodynamics_semantic_information_dictionary_v0_1.md#ic-54), [IC-55](thermodynamics_semantic_information_dictionary_v0_1.md#ic-55), [IC-57](thermodynamics_semantic_information_dictionary_v0_1.md#ic-57).

**Preconditions:** Original constraints and variable roles are captured; all permitted temporary interventions and restoration criteria are identified.

**Procedure and information flow:**

1. P08 checks seed phase region, coordinates and definition compatibility.
2. Generate or adapt numerical guesses without promoting them to specifications.
3. Log every temporary fixed variable, relaxation or altered trial subproblem.
4. Execute preparation; before original-problem acceptance restore or explicitly reconcile every intervention and supply restoration evidence.
5. If restoration fails quarantine affected work and withhold original-problem success.

**Successful outputs:** [IC-54](thermodynamics_semantic_information_dictionary_v0_1.md#ic-54), [IC-57](thermodynamics_semantic_information_dictionary_v0_1.md#ic-57), [IC-56](thermodynamics_semantic_information_dictionary_v0_1.md#ic-56). An admissible starting state and documented restoration status, or a precise initialization-required/failed outcome.

**Permitted effects:** Private provider/session work and execution evidence only; no canonical definition or current-result writes.

**Adoption/completion boundary:** A candidate remains unaccepted until AC-36/37; uncertain context is handled by AC-35.

**Failure and recovery:** Failure to locate a start is not demonstrated infeasibility. An intentionally simplified seed is not a solution of the original equations.

**Repeated delivery/re-execution:** Initialization can be repeated on a fresh private workspace; restoration must be idempotent or reconstruct the original formulation. A cancelled run cannot leave shared model variables fixed.

**AW-31-P:** A temporary stage-temperature fixing is removed before final original-constraint checks.
**AW-31-N:** An unremoved initialization constraint blocks acceptance even when the numerical solver converges.

**Primary B4 requirements:** [FR-RUN-01](thermodynamics_functional_requirements_v0_1.md#fr-run-01), [FR-RUN-02](thermodynamics_functional_requirements_v0_1.md#fr-run-02). **Supporting requirements:** None.

**Common action rules:** [AR-04](#ar-04), [AR-07](#ar-07), [AR-08](#ar-08), [AR-11](#ar-11). **Handoff facts:** EV-09

**Evidence status:** Action specified. Its positive/negative witnesses have not been executed against the simulator or a thermodynamic provider. Primary ownership is inherited from B5; information and invariants are inherited from the linked B6 concepts.

<a id="ac-32"></a>
### AC-32. Execute an operation on private working state

**Accountable owner:** P08, Provider realization and numerical execution. **Action kind:** execute.

**Initiator/trigger:** An initialized attempt is authorized to perform a state/property/chemical/derivative or contribution evaluation.

**Inputs:** [IC-35](thermodynamics_semantic_information_dictionary_v0_1.md#ic-35), [IC-37](thermodynamics_semantic_information_dictionary_v0_1.md#ic-37), [IC-40](thermodynamics_semantic_information_dictionary_v0_1.md#ic-40), [IC-41](thermodynamics_semantic_information_dictionary_v0_1.md#ic-41), [IC-53](thermodynamics_semantic_information_dictionary_v0_1.md#ic-53), [IC-54](thermodynamics_semantic_information_dictionary_v0_1.md#ic-54), [IC-55](thermodynamics_semantic_information_dictionary_v0_1.md#ic-55), [IC-57](thermodynamics_semantic_information_dictionary_v0_1.md#ic-57), [IC-58](thermodynamics_semantic_information_dictionary_v0_1.md#ic-58), [IC-70](thermodynamics_semantic_information_dictionary_v0_1.md#ic-70), [IC-72](thermodynamics_semantic_information_dictionary_v0_1.md#ic-72).

**Preconditions:** Execution readiness and safe session policy hold; any fresh dispatch after cancellation is prohibited unless separately authorized diagnostic work.

**Procedure and information flow:**

1. P08 installs only captured inputs through the recorded mapping.
2. Invoke the whole-package or qualified submodel route, retaining physical restrictions and required output scope.
3. Track actual algorithms, searched phases, chemical changes, native outcomes and modeled exchanges where available.
4. Return terminated or partial evidence; do not publish into the process model.
5. On native error contain or retire the context under AC-35.

**Successful outputs:** [IC-54](thermodynamics_semantic_information_dictionary_v0_1.md#ic-54), [IC-56](thermodynamics_semantic_information_dictionary_v0_1.md#ic-56), [IC-63](thermodynamics_semantic_information_dictionary_v0_1.md#ic-63), [IC-71](thermodynamics_semantic_information_dictionary_v0_1.md#ic-71). Execution evidence from the exact attempt, possibly with incomplete fields, ready for normalization rather than automatic acceptance.

**Permitted effects:** Private provider/session work and execution evidence only; no canonical definition or current-result writes.

**Adoption/completion boundary:** A candidate remains unaccepted until AC-36/37; uncertain context is handled by AC-35.

**Failure and recovery:** A provider success that changed physics is a mismatch candidate, not same-problem success. A crash may require isolation/restart; no rollback of native memory is promised. Accepted host records stay unchanged.

**Repeated delivery/re-execution:** Native calls are not assumed idempotent. Lost responses are reconciled by attempt identity and session health; a fresh re-execution is a new attempt on known inputs.

**AW-32-P:** A whole coherent engine may calculate both chemistry and phases behind one declared operation.
**AW-32-N:** Returning old work-buffer enthalpy after a failed native update cannot be accepted as fresh output.

**Primary B4 requirements:** None. **Supporting requirements:** [FR-STA-03](thermodynamics_functional_requirements_v0_1.md#fr-sta-03), [FR-EQL-01](thermodynamics_functional_requirements_v0_1.md#fr-eql-01), [FR-CHM-03](thermodynamics_functional_requirements_v0_1.md#fr-chm-03), [FR-RUN-07](thermodynamics_functional_requirements_v0_1.md#fr-run-07).

**Common action rules:** [AR-02](#ar-02), [AR-03](#ar-03), [AR-05](#ar-05), [AR-08](#ar-08), [AR-13](#ar-13). **Handoff facts:** EV-10

**Evidence status:** Action specified. Its positive/negative witnesses have not been executed against the simulator or a thermodynamic provider. Primary ownership is inherited from B5; information and invariants are inherited from the linked B6 concepts.

<a id="ac-33"></a>
### AC-33. Normalize and detach returned candidate evidence

**Accountable owner:** P08, Provider realization and numerical execution. **Action kind:** derive.

**Initiator/trigger:** A provider returns values, a contribution, partial results or a failure.

**Inputs:** [IC-26](thermodynamics_semantic_information_dictionary_v0_1.md#ic-26), [IC-28](thermodynamics_semantic_information_dictionary_v0_1.md#ic-28), [IC-29](thermodynamics_semantic_information_dictionary_v0_1.md#ic-29), [IC-35](thermodynamics_semantic_information_dictionary_v0_1.md#ic-35), [IC-38](thermodynamics_semantic_information_dictionary_v0_1.md#ic-38), [IC-39](thermodynamics_semantic_information_dictionary_v0_1.md#ic-39), [IC-40](thermodynamics_semantic_information_dictionary_v0_1.md#ic-40), [IC-53](thermodynamics_semantic_information_dictionary_v0_1.md#ic-53), [IC-54](thermodynamics_semantic_information_dictionary_v0_1.md#ic-54), [IC-55](thermodynamics_semantic_information_dictionary_v0_1.md#ic-55), [IC-58](thermodynamics_semantic_information_dictionary_v0_1.md#ic-58).

**Preconditions:** Attempt identity and native mapping are known; unknown native facts remain unknown instead of guessed.

**Procedure and information flow:**

1. P08 translates coordinates, units, references and axes using AC-28.
2. Detach or safely retain payloads so later workspace mutation cannot change candidate evidence.
3. Give every promised output a computed/unavailable/not-requested/failed status.
4. Record actual used assumptions, examined phase scope, diagnostics and actual modeled exchanges.
5. P04 phase interpretation can use AC-16; P09 qualification remains separate.

**Successful outputs:** [IC-26](thermodynamics_semantic_information_dictionary_v0_1.md#ic-26), [IC-28](thermodynamics_semantic_information_dictionary_v0_1.md#ic-28), [IC-56](thermodynamics_semantic_information_dictionary_v0_1.md#ic-56), [IC-63](thermodynamics_semantic_information_dictionary_v0_1.md#ic-63), [IC-71](thermodynamics_semantic_information_dictionary_v0_1.md#ic-71). An immutable attributable candidate or failure statement with no fabricated property values.

**Permitted effects:** Compute from captured inputs into new candidate/proposed values; do not cumulatively modify the source.

**Adoption/completion boundary:** Authored input changes use AC-47/48; calculated result groups use AC-36/37/49. Physical transfers are recorded once per declared intent.

**Failure and recovery:** Uninterpretable native ordering or conventions reject the affected payload. Optional-property failure is separated from primary failure; uncertain session health is sent to AC-35.

**Repeated delivery/re-execution:** Duplicate delivery of the same candidate is recognized; changed payload under the same candidate/attempt completion identity is a conflict, not an overwrite.

**AW-33-P:** A zero-amount incipient phase remains useful numerical evidence but creates no positive material flow.
**AW-33-N:** A missing viscosity does not become zero and a mutable native buffer cannot alter a prior candidate.

**Primary B4 requirements:** None. **Supporting requirements:** [FR-EQL-05](thermodynamics_functional_requirements_v0_1.md#fr-eql-05), [FR-STA-05](thermodynamics_functional_requirements_v0_1.md#fr-sta-05), [FR-RES-01](thermodynamics_functional_requirements_v0_1.md#fr-res-01).

**Common action rules:** [AR-02](#ar-02), [AR-07](#ar-07), [AR-08](#ar-08), [AR-10](#ar-10), [AR-13](#ar-13). **Handoff facts:** EV-10

**Evidence status:** Action specified. Its positive/negative witnesses have not been executed against the simulator or a thermodynamic provider. Primary ownership is inherited from B5; information and invariants are inherited from the linked B6 concepts.

<a id="ac-34"></a>
### AC-34. Retry the same physical problem under a new attempt

**Accountable owner:** P08, Provider realization and numerical execution. **Action kind:** execute.

**Initiator/trigger:** A permitted numerical failure or insufficient evidence motivates another attempt.

**Inputs:** [IC-35](thermodynamics_semantic_information_dictionary_v0_1.md#ic-35), [IC-41](thermodynamics_semantic_information_dictionary_v0_1.md#ic-41), [IC-53](thermodynamics_semantic_information_dictionary_v0_1.md#ic-53), [IC-54](thermodynamics_semantic_information_dictionary_v0_1.md#ic-54), [IC-56](thermodynamics_semantic_information_dictionary_v0_1.md#ic-56), [IC-57](thermodynamics_semantic_information_dictionary_v0_1.md#ic-57), [IC-63](thermodynamics_semantic_information_dictionary_v0_1.md#ic-63), [IC-70](thermodynamics_semantic_information_dictionary_v0_1.md#ic-70), [IC-72](thermodynamics_semantic_information_dictionary_v0_1.md#ic-72).

**Preconditions:** Original problem and failed attempt are retained; retry policy, budget and run authority permit further work.

**Procedure and information flow:**

1. P08 classifies the proposed change: numerical procedure only or physical alteration.
2. For a same-problem retry create a new attempt, preserving targets, candidates, species restrictions and convention.
3. Requalify seed/session and execute through AC-30..33.
4. If changing physics is necessary, return an alternative proposal to AC-54 instead of editing the original.

**Successful outputs:** [IC-54](thermodynamics_semantic_information_dictionary_v0_1.md#ic-54), [IC-57](thermodynamics_semantic_information_dictionary_v0_1.md#ic-57), [IC-72](thermodynamics_semantic_information_dictionary_v0_1.md#ic-72). A bounded retry chain with independent attempt evidence and one unchanged physical question.

**Permitted effects:** Private provider/session work and execution evidence only; no canonical definition or current-result writes.

**Adoption/completion boundary:** A candidate remains unaccepted until AC-36/37; uncertain context is handled by AC-35.

**Failure and recovery:** Exhausted budget returns NONCONVERGENCE/BUDGET_EXHAUSTED with known causes. Phase suppression or ideal substitution cannot enter this same-problem path.

**Repeated delivery/re-execution:** Repeated delivery of the retry decision cannot spawn unbounded attempts. Each intentional retry gets one new identifier and ancestry.

**AW-34-P:** A different root-finding method solves the same original PH target.
**AW-34-N:** Dropping the second allowed liquid cannot be recorded merely as a faster retry.

**Primary B4 requirements:** [FR-RUN-05](thermodynamics_functional_requirements_v0_1.md#fr-run-05). **Supporting requirements:** None.

**Common action rules:** [AR-05](#ar-05), [AR-11](#ar-11). **Handoff facts:** Local proposal/candidate handoff under the common contract.

**Evidence status:** Action specified. Its positive/negative witnesses have not been executed against the simulator or a thermodynamic provider. Primary ownership is inherited from B5; information and invariants are inherited from the linked B6 concepts.

<a id="ac-35"></a>
### AC-35. Quarantine, restore or retire a provider session

**Accountable owner:** P08, Provider realization and numerical execution. **Action kind:** execute.

**Initiator/trigger:** Provider failure, uncertain cleanup, cancellation or configuration invalidation affects session trust.

**Inputs:** [IC-53](thermodynamics_semantic_information_dictionary_v0_1.md#ic-53), [IC-54](thermodynamics_semantic_information_dictionary_v0_1.md#ic-54), [IC-55](thermodynamics_semantic_information_dictionary_v0_1.md#ic-55), [IC-56](thermodynamics_semantic_information_dictionary_v0_1.md#ic-56), [IC-63](thermodynamics_semantic_information_dictionary_v0_1.md#ic-63), [IC-70](thermodynamics_semantic_information_dictionary_v0_1.md#ic-70).

**Preconditions:** Session identity, active attempt and available failure/isolation evidence are retained.

**Procedure and information flow:**

1. P08 prevents new use of an uncertain or busy context.
2. Request interruption only where supported; separate logical cancellation from actual native termination.
3. Keep resources leased until termination is acknowledged or isolate/retire the worker.
4. Reset only under a verified recovery contract; otherwise reconstruct a new session.
5. Normalize the known failure and preserve accepted host records.

**Successful outputs:** [IC-55](thermodynamics_semantic_information_dictionary_v0_1.md#ic-55), [IC-63](thermodynamics_semantic_information_dictionary_v0_1.md#ic-63). An explicit reusable, quarantined, reconstructed or retired state and recovery evidence.

**Permitted effects:** Private provider/session work and execution evidence only; no canonical definition or current-result writes.

**Adoption/completion boundary:** A candidate remains unaccepted until AC-36/37; uncertain context is handled by AC-35.

**Failure and recovery:** Unknown recovery remains quarantined/retired. A hung in-process native call cannot be forcibly declared safe or returned to a pool. Process containment is an implementation qualification, not a universal promise.

**Repeated delivery/re-execution:** Repeated cleanup is safe only under its declared lifecycle protocol; cannot free or concurrently reuse a still-running context. Reconstruction yields a new session identity.

**AW-35-P:** After a failed native update a fresh session serves another valid request with no retained-buffer contamination.
**AW-35-N:** Cancelling logically cannot free a buffer still being used by an uninterruptible native call.

**Primary B4 requirements:** [FR-RUN-06](thermodynamics_functional_requirements_v0_1.md#fr-run-06), [FR-RUN-07](thermodynamics_functional_requirements_v0_1.md#fr-run-07). **Supporting requirements:** None.

**Common action rules:** [AR-07](#ar-07), [AR-08](#ar-08), [AR-13](#ar-13). **Handoff facts:** EV-11

**Evidence status:** Action specified. Its positive/negative witnesses have not been executed against the simulator or a thermodynamic provider. Primary ownership is inherited from B5; information and invariants are inherited from the linked B6 concepts.

<a id="ac-36"></a>
### AC-36. Evaluate original postconditions and identify missing evidence

**Accountable owner:** P09, Result qualification and coverage evidence. **Action kind:** assess.

**Initiator/trigger:** A candidate or coherent process result set is offered for checking.

**Inputs:** [IC-05](thermodynamics_semantic_information_dictionary_v0_1.md#ic-05), [IC-21](thermodynamics_semantic_information_dictionary_v0_1.md#ic-21), [IC-35](thermodynamics_semantic_information_dictionary_v0_1.md#ic-35), [IC-37](thermodynamics_semantic_information_dictionary_v0_1.md#ic-37), [IC-41](thermodynamics_semantic_information_dictionary_v0_1.md#ic-41), [IC-50](thermodynamics_semantic_information_dictionary_v0_1.md#ic-50), [IC-52](thermodynamics_semantic_information_dictionary_v0_1.md#ic-52), [IC-56](thermodynamics_semantic_information_dictionary_v0_1.md#ic-56), [IC-71](thermodynamics_semantic_information_dictionary_v0_1.md#ic-71).

**Preconditions:** Original obligations, conditional applicability, scales and tolerances were declared before the candidate; evidence source/independence is identified.

**Procedure and information flow:**

1. P09 evaluates authority, target, conserved-basis and applicable energy/phase/chemistry checks.
2. Use disjoint material portions and all authorized exchange terms; preserve dimensional residuals and cancellation-safe scales.
3. Record pass/fail/not-assessed/not-applicable individually.
4. Emit bounded missing-evidence demands to AC-55 under original conventions; never adjust the target to fit the candidate.

**Successful outputs:** [IC-59](thermodynamics_semantic_information_dictionary_v0_1.md#ic-59), [IC-63](thermodynamics_semantic_information_dictionary_v0_1.md#ic-63). A per-obligation evidence set suitable for scope qualification, with no numerical success shortcut.

**Permitted effects:** Append attributable assessment evidence; original input, candidate and criterion remain fixed.

**Adoption/completion boundary:** Assessment can qualify a named scope but only AC-49 makes a current-result association.

**Failure and recovery:** Failed mandatory checks or unavailable mandatory evidence block that scope. Unknown elemental data cannot certify elemental conservation; optional failures remain scoped.

**Repeated delivery/re-execution:** Additional evidence creates another assessment linked to the unchanged candidate and original checks. No check execution applies a physical transfer again.

**AW-36-P:** A candidate h=20 against target 50 yields the original 30-unit discrepancy and rejection at that target check.
**AW-36-N:** A small solver step cannot replace a failed physical energy or material residual.

**Primary B4 requirements:** [FR-RES-02](thermodynamics_functional_requirements_v0_1.md#fr-res-02), [FR-RES-03](thermodynamics_functional_requirements_v0_1.md#fr-res-03). **Supporting requirements:** None.

**Common action rules:** [AR-02](#ar-02), [AR-03](#ar-03), [AR-04](#ar-04), [AR-07](#ar-07), [AR-12](#ar-12), [AR-13](#ar-13), [AR-15](#ar-15). **Handoff facts:** EV-12

**Evidence status:** Action specified. Its positive/negative witnesses have not been executed against the simulator or a thermodynamic provider. Primary ownership is inherited from B5; information and invariants are inherited from the linked B6 concepts.

<a id="ac-37"></a>
### AC-37. Qualify local, unit or flowsheet results for a named scope

**Accountable owner:** P09, Result qualification and coverage evidence. **Action kind:** assess.

**Initiator/trigger:** Required evidence has been gathered, failed, or exhausted for a proposed acceptance scope.

**Inputs:** [IC-35](thermodynamics_semantic_information_dictionary_v0_1.md#ic-35), [IC-41](thermodynamics_semantic_information_dictionary_v0_1.md#ic-41), [IC-52](thermodynamics_semantic_information_dictionary_v0_1.md#ic-52), [IC-56](thermodynamics_semantic_information_dictionary_v0_1.md#ic-56), [IC-59](thermodynamics_semantic_information_dictionary_v0_1.md#ic-59), [IC-63](thermodynamics_semantic_information_dictionary_v0_1.md#ic-63), [IC-64](thermodynamics_semantic_information_dictionary_v0_1.md#ic-64).

**Preconditions:** Candidate identities, original problems, used assumptions, completion kind and lineage are consistent.

**Procedure and information flow:**

1. P09 classifies termination separately from target satisfaction, authority and scientific applicability.
2. Resolve all applicable mandatory checks for the declared scope.
3. For coupled reactive/process results require combined conditions after all substeps and required parent residuals.
4. Issue accepted-for-scope, rejected or unassessed result with requested-versus-used differences and attribution.
5. Return publication eligibility only; P10 owns current binding.

**Successful outputs:** [IC-60](thermodynamics_semantic_information_dictionary_v0_1.md#ic-60), [IC-62](thermodynamics_semantic_information_dictionary_v0_1.md#ic-62), [IC-63](thermodynamics_semantic_information_dictionary_v0_1.md#ic-63). A precise accepted local/constraints/allocation/unit/flowsheet assessment or a qualified failure. Optional-property incompleteness remains visible.

**Permitted effects:** Append attributable assessment evidence; original input, candidate and criterion remain fixed.

**Adoption/completion boundary:** Assessment can qualify a named scope but only AC-49 makes a current-result association.

**Failure and recovery:** Local success does not certify unit or recycle convergence. A changed-physics alternative retains its own scope and cannot pass the original problem. Missing lineage lowers the reproducibility claim.

**Repeated delivery/re-execution:** Qualification replay is observational; additional evidence adds a versioned assessment. It cannot relabel old candidate producer or configuration.

**AW-37-P:** A checked PH local result is usable in a run while the enclosing recycle remains unaccepted.
**AW-37-N:** Two independent chemistry/flash successes cannot pass if their final combined state violates either condition.

**Primary B4 requirements:** [FR-CHM-08](thermodynamics_functional_requirements_v0_1.md#fr-chm-08), [FR-RES-01](thermodynamics_functional_requirements_v0_1.md#fr-res-01), [FR-RES-04](thermodynamics_functional_requirements_v0_1.md#fr-res-04), [FR-RES-07](thermodynamics_functional_requirements_v0_1.md#fr-res-07). **Supporting requirements:** None.

**Common action rules:** [AR-02](#ar-02), [AR-04](#ar-04), [AR-07](#ar-07), [AR-09](#ar-09), [AR-12](#ar-12). **Handoff facts:** EV-12, EV-14

**Evidence status:** Action specified. Its positive/negative witnesses have not been executed against the simulator or a thermodynamic provider. Primary ownership is inherited from B5; information and invariants are inherited from the linked B6 concepts.

<a id="ac-38"></a>
### AC-38. Register bounded conformance and validation evidence

**Accountable owner:** P09, Result qualification and coverage evidence. **Action kind:** assess.

**Initiator/trigger:** A reviewer adds a conceptual review, executed fixture, reproduction or independent reference comparison.

**Inputs:** [IC-10](thermodynamics_semantic_information_dictionary_v0_1.md#ic-10), [IC-20](thermodynamics_semantic_information_dictionary_v0_1.md#ic-20), [IC-41](thermodynamics_semantic_information_dictionary_v0_1.md#ic-41), [IC-53](thermodynamics_semantic_information_dictionary_v0_1.md#ic-53), [IC-59](thermodynamics_semantic_information_dictionary_v0_1.md#ic-59), [IC-60](thermodynamics_semantic_information_dictionary_v0_1.md#ic-60), [IC-61](thermodynamics_semantic_information_dictionary_v0_1.md#ic-61), [IC-69](thermodynamics_semantic_information_dictionary_v0_1.md#ic-69).

**Preconditions:** Claim subject, exact configuration/envelope, witness kind, tolerances and actual evidence status are supplied.

**Procedure and information flow:**

1. P09 distinguishes catalog trace, representability review, implementation conformance, executed capability, physical consistency and independent validation.
2. Bind each claim to its actual material/data/operation/phase and provider context.
3. Preserve unmet P1/P2 coverage and explicit limitations.
4. Withdraw or supersede claims only with new evidence without erasing historic conditions.

**Successful outputs:** [IC-61](thermodynamics_semantic_information_dictionary_v0_1.md#ic-61). A bounded claim that does not expand by library name or by linking requirements to actions.

**Permitted effects:** Append attributable assessment evidence; original input, candidate and criterion remain fixed.

**Adoption/completion boundary:** Assessment can qualify a named scope but only AC-49 makes a current-result association.

**Failure and recovery:** Source inspection and synthetic reference-model checks cannot promote a real-fluid scenario to E/V. An unsupported provider does not close a program-wide scope gap.

**Repeated delivery/re-execution:** Duplicate witness ingestion is deduplicated by evidence identity. A repeated claim cannot amplify confidence or broaden the envelope.

**AW-38-P:** A TP fixture adds evidence only for that pinned TP configuration.
**AW-38-N:** Having 34 mapped scenarios cannot be exported as 34 validated flowsheet capabilities.

**Primary B4 requirements:** [FR-GOV-01](thermodynamics_functional_requirements_v0_1.md#fr-gov-01), [FR-GOV-05](thermodynamics_functional_requirements_v0_1.md#fr-gov-05), [FR-RES-08](thermodynamics_functional_requirements_v0_1.md#fr-res-08). **Supporting requirements:** None.

**Common action rules:** [AR-12](#ar-12), [AR-18](#ar-18). **Handoff facts:** EV-18

**Evidence status:** Action specified. Its positive/negative witnesses have not been executed against the simulator or a thermodynamic provider. Primary ownership is inherited from B5; information and invariants are inherited from the linked B6 concepts.

<a id="ac-39"></a>
### AC-39. Mix compatible feeds or apply a mechanical split

**Accountable owner:** P07, Flowsheet integration and use-case coordination. **Action kind:** derive.

**Initiator/trigger:** A mixer or composition-preserving splitter evaluates captured input material.

**Inputs:** [IC-03](thermodynamics_semantic_information_dictionary_v0_1.md#ic-03), [IC-05](thermodynamics_semantic_information_dictionary_v0_1.md#ic-05), [IC-21](thermodynamics_semantic_information_dictionary_v0_1.md#ic-21), [IC-25](thermodynamics_semantic_information_dictionary_v0_1.md#ic-25), [IC-28](thermodynamics_semantic_information_dictionary_v0_1.md#ic-28), [IC-35](thermodynamics_semantic_information_dictionary_v0_1.md#ic-35), [IC-50](thermodynamics_semantic_information_dictionary_v0_1.md#ic-50), [IC-52](thermodynamics_semantic_information_dictionary_v0_1.md#ic-52), [IC-71](thermodynamics_semantic_information_dictionary_v0_1.md#ic-71).

**Preconditions:** Compatible material/conventions, explicit pressure/heat/work rules and split intent are available; missing molecular bases are not fabricated.

**Procedure and information flow:**

1. P07 sums justified constituent or conserved quantities and compatible energy terms for mixing.
2. Build the mixed-state target rather than averaging temperatures without a physical model.
3. For a mechanical split apply declared fractions to quantities while preserving appropriate state/composition.
4. Request any needed state completion through AC-21/23.
5. Form a coherent product candidate group for AC-36/37/49.

**Successful outputs:** [IC-28](thermodynamics_semantic_information_dictionary_v0_1.md#ic-28), [IC-35](thermodynamics_semantic_information_dictionary_v0_1.md#ic-35), [IC-52](thermodynamics_semantic_information_dictionary_v0_1.md#ic-52), [IC-56](thermodynamics_semantic_information_dictionary_v0_1.md#ic-56), [IC-71](thermodynamics_semantic_information_dictionary_v0_1.md#ic-71). Balanced candidate products and explicit local state-completion needs.

**Permitted effects:** Compute from captured inputs into new candidate/proposed values; do not cumulatively modify the source.

**Adoption/completion boundary:** Authored input changes use AC-47/48; calculated result groups use AC-36/37/49. Physical transfers are recorded once per declared intent.

**Failure and recovery:** Incompatible references/representations require AC-42 first. Invalid split fractions or insufficient caloric data block the affected product set.

**Repeated delivery/re-execution:** Evaluate from original input snapshots; repeating a numerical evaluation is not another physical batch mixing. Publication deduplicates the proposed product set.

**AW-39-P:** Recombining a mechanical split recovers the original quantities.
**AW-39-N:** A mixing rule cannot replace energy conservation with an unqualified arithmetic temperature average.

**Primary B4 requirements:** [FR-FLW-02](thermodynamics_functional_requirements_v0_1.md#fr-flw-02). **Supporting requirements:** None.

**Common action rules:** [AR-05](#ar-05), [AR-14](#ar-14). **Handoff facts:** Local proposal/candidate handoff under the common contract.

**Evidence status:** Action specified. Its positive/negative witnesses have not been executed against the simulator or a thermodynamic provider. Primary ownership is inherited from B5; information and invariants are inherited from the linked B6 concepts.

<a id="ac-40"></a>
### AC-40. Route internal phases and carried solids to product ports

**Accountable owner:** P07, Flowsheet integration and use-case coordination. **Action kind:** derive.

**Initiator/trigger:** A separator or solids-handling model has an internal material description for product allocation.

**Inputs:** [IC-25](thermodynamics_semantic_information_dictionary_v0_1.md#ic-25), [IC-28](thermodynamics_semantic_information_dictionary_v0_1.md#ic-28), [IC-29](thermodynamics_semantic_information_dictionary_v0_1.md#ic-29), [IC-30](thermodynamics_semantic_information_dictionary_v0_1.md#ic-30), [IC-50](thermodynamics_semantic_information_dictionary_v0_1.md#ic-50), [IC-52](thermodynamics_semantic_information_dictionary_v0_1.md#ic-52), [IC-60](thermodynamics_semantic_information_dictionary_v0_1.md#ic-60), [IC-71](thermodynamics_semantic_information_dictionary_v0_1.md#ic-71).

**Preconditions:** Internal-state authority/achieved scope, port roles, carryover/entrainment rules and empty-product handling are known.

**Procedure and information flow:**

1. P07 maps physical portions to product roles using named rules, not native slot positions.
2. Apply declared solids carryover or entrainment only within available material.
3. Construct all related outlet candidates including explicitly empty products.
4. Request compatible post-routing property/state completion when required and check the group against the original unit balances.

**Successful outputs:** [IC-28](thermodynamics_semantic_information_dictionary_v0_1.md#ic-28), [IC-52](thermodynamics_semantic_information_dictionary_v0_1.md#ic-52), [IC-56](thermodynamics_semantic_information_dictionary_v0_1.md#ic-56), [IC-71](thermodynamics_semantic_information_dictionary_v0_1.md#ic-71). A product set with routing lineage independent of internal phase identity; changing routing need not change the internal equilibrium record.

**Permitted effects:** Compute from captured inputs into new candidate/proposed values; do not cumulatively modify the source.

**Adoption/completion boundary:** Authored input changes use AC-47/48; calculated result groups use AC-36/37/49. Physical transfers are recorded once per declared intent.

**Failure and recovery:** Ambiguous required phase correspondence, over-allocation or unspecified required solids disposition blocks product acceptance. No fictitious flow is used for absent phases.

**Repeated delivery/re-execution:** Routing is a derivation from a fixed internal state; repetition cannot remove material a second time. Distinct physical transfers get explicit new intent.

**AW-40-P:** Changing a solids-carryover rule changes outlet allocation while keeping the internal state identifiable.
**AW-40-N:** Swapping provider liquid order cannot silently swap product intent or duplicate total-liquid material.

**Primary B4 requirements:** [FR-FLW-03](thermodynamics_functional_requirements_v0_1.md#fr-flw-03). **Supporting requirements:** None.

**Common action rules:** [AR-14](#ar-14). **Handoff facts:** Local proposal/candidate handoff under the common contract.

**Evidence status:** Action specified. Its positive/negative witnesses have not been executed against the simulator or a thermodynamic provider. Primary ownership is inherited from B5; information and invariants are inherited from the linked B6 concepts.

<a id="ac-41"></a>
### AC-41. Coordinate heat-only coupling between distinct material regions

**Accountable owner:** P07, Flowsheet integration and use-case coordination. **Action kind:** derive.

**Initiator/trigger:** A heat exchanger or utility relation couples two independently modeled materials.

**Inputs:** [IC-21](thermodynamics_semantic_information_dictionary_v0_1.md#ic-21), [IC-28](thermodynamics_semantic_information_dictionary_v0_1.md#ic-28), [IC-35](thermodynamics_semantic_information_dictionary_v0_1.md#ic-35), [IC-41](thermodynamics_semantic_information_dictionary_v0_1.md#ic-41), [IC-49](thermodynamics_semantic_information_dictionary_v0_1.md#ic-49), [IC-50](thermodynamics_semantic_information_dictionary_v0_1.md#ic-50), [IC-52](thermodynamics_semantic_information_dictionary_v0_1.md#ic-52), [IC-71](thermodynamics_semantic_information_dictionary_v0_1.md#ic-71).

**Preconditions:** Each side has its own configuration and consistent within-side caloric convention; unit-owned heat/work/loss signs and constraints are stated.

**Procedure and information flow:**

1. P07 prepares independent side-specific state/property problems.
2. Evaluate only requested required and optional bounding/rating properties.
3. Assemble the shared heat balance without translating material across the wall.
4. Iterate through the host unit model if needed and gather side/coupling residuals.
5. Propose both sides as a coherent qualified publication set.

**Successful outputs:** [IC-35](thermodynamics_semantic_information_dictionary_v0_1.md#ic-35), [IC-52](thermodynamics_semantic_information_dictionary_v0_1.md#ic-52), [IC-56](thermodynamics_semantic_information_dictionary_v0_1.md#ic-56), [IC-71](thermodynamics_semantic_information_dictionary_v0_1.md#ic-71). Side states and shared heat accounting without a common species slate or common absolute enthalpy zero.

**Permitted effects:** Compute from captured inputs into new candidate/proposed values; do not cumulatively modify the source.

**Adoption/completion boundary:** Authored input changes use AC-47/48; calculated result groups use AC-36/37/49. Physical transfers are recorded once per declared intent.

**Failure and recovery:** Required caloric failure blocks the coupling; an optional bound may be absent with a qualified report unless that bound is necessary for the selected mode. No silent reduction from rating to duty-only.

**Repeated delivery/re-execution:** Repeated heat-balance iterations recompute from captured side states, not cumulatively applying the same duty. Group publication is duplicate-safe.

**AW-41-P:** A consistent reference shift on one nonreacting side leaves its enthalpy difference unchanged.
**AW-41-N:** The heat-only route cannot acquire a species map or transfer matter to reconcile energy.

**Primary B4 requirements:** [FR-FLW-04](thermodynamics_functional_requirements_v0_1.md#fr-flw-04). **Supporting requirements:** None.

**Common action rules:** [AR-13](#ar-13), [AR-15](#ar-15). **Handoff facts:** Local proposal/candidate handoff under the common contract.

**Evidence status:** Action specified. Its positive/negative witnesses have not been executed against the simulator or a thermodynamic provider. Primary ownership is inherited from B5; information and invariants are inherited from the linked B6 concepts.

<a id="ac-42"></a>
### AC-42. Translate material descriptions under explicit preserved constraints

**Accountable owner:** P07, Flowsheet integration and use-case coordination. **Action kind:** derive.

**Initiator/trigger:** Material crosses between package regions or a representation conversion is requested.

**Inputs:** [IC-03](thermodynamics_semantic_information_dictionary_v0_1.md#ic-03), [IC-05](thermodynamics_semantic_information_dictionary_v0_1.md#ic-05), [IC-07](thermodynamics_semantic_information_dictionary_v0_1.md#ic-07), [IC-21](thermodynamics_semantic_information_dictionary_v0_1.md#ic-21), [IC-28](thermodynamics_semantic_information_dictionary_v0_1.md#ic-28), [IC-35](thermodynamics_semantic_information_dictionary_v0_1.md#ic-35), [IC-41](thermodynamics_semantic_information_dictionary_v0_1.md#ic-41), [IC-47](thermodynamics_semantic_information_dictionary_v0_1.md#ic-47), [IC-49](thermodynamics_semantic_information_dictionary_v0_1.md#ic-49), [IC-50](thermodynamics_semantic_information_dictionary_v0_1.md#ic-50), [IC-51](thermodynamics_semantic_information_dictionary_v0_1.md#ic-51), [IC-71](thermodynamics_semantic_information_dictionary_v0_1.md#ic-71).

**Preconditions:** Source/target snapshots/bindings, forward map, justified conservation, reference relation where needed and preserved-quantity policy are explicit.

**Procedure and information flow:**

1. P07 applies the mapping as a new description, not a second inventory.
2. Apply only justified convention transformations, retaining model discrepancy separately.
3. Build the target problem with jointly consistent constraints and solve through the shared path where needed.
4. Record mapping loss, reference adjustment, remaining model disagreement and any genuinely authorized physical exchange separately.
5. Check and qualify the complete boundary result.

**Successful outputs:** [IC-27](thermodynamics_semantic_information_dictionary_v0_1.md#ic-27), [IC-28](thermodynamics_semantic_information_dictionary_v0_1.md#ic-28), [IC-35](thermodynamics_semantic_information_dictionary_v0_1.md#ic-35), [IC-51](thermodynamics_semantic_information_dictionary_v0_1.md#ic-51), [IC-56](thermodynamics_semantic_information_dictionary_v0_1.md#ic-56), [IC-71](thermodynamics_semantic_information_dictionary_v0_1.md#ic-71). A target description satisfying its chosen boundary policy with visible loss and discrepancies.

**Permitted effects:** Compute from captured inputs into new candidate/proposed values; do not cumulatively modify the source.

**Adoption/completion boundary:** Authored input changes use AC-47/48; calculated result groups use AC-36/37/49. Physical transfers are recorded once per declared intent.

**Failure and recovery:** Missing inverse information, unknown required reference transform or incompatible simultaneous TP/composition/H preservation blocks translation. Do not insert hidden heat.

**Repeated delivery/re-execution:** Redelivery of the same translation does not transfer or create material again. Changing policy creates a new case/problem; reverse reconstruction needs new information.

**AW-42-P:** At fixed TP and composition a known reference correction is separated from remaining enthalpy-model disagreement.
**AW-42-N:** An information-losing lump cannot be uniquely un-lumped without extra assumptions.

**Primary B4 requirements:** [FR-FLW-05](thermodynamics_functional_requirements_v0_1.md#fr-flw-05), [FR-FLW-06](thermodynamics_functional_requirements_v0_1.md#fr-flw-06). **Supporting requirements:** None.

**Common action rules:** [AR-14](#ar-14), [AR-15](#ar-15). **Handoff facts:** Local proposal/candidate handoff under the common contract.

**Evidence status:** Action specified. Its positive/negative witnesses have not been executed against the simulator or a thermodynamic provider. Primary ownership is inherited from B5; information and invariants are inherited from the linked B6 concepts.

<a id="ac-43"></a>
### AC-43. Coordinate a coupled unit or nonequilibrium mathematical formulation

**Accountable owner:** P07, Flowsheet integration and use-case coordination. **Action kind:** coordinate.

**Initiator/trigger:** A column, reactor or contactor uses internal states and mathematical contributions.

**Inputs:** [IC-28](thermodynamics_semantic_information_dictionary_v0_1.md#ic-28), [IC-35](thermodynamics_semantic_information_dictionary_v0_1.md#ic-35), [IC-36](thermodynamics_semantic_information_dictionary_v0_1.md#ic-36), [IC-37](thermodynamics_semantic_information_dictionary_v0_1.md#ic-37), [IC-39](thermodynamics_semantic_information_dictionary_v0_1.md#ic-39), [IC-40](thermodynamics_semantic_information_dictionary_v0_1.md#ic-40), [IC-41](thermodynamics_semantic_information_dictionary_v0_1.md#ic-41), [IC-45](thermodynamics_semantic_information_dictionary_v0_1.md#ic-45), [IC-48](thermodynamics_semantic_information_dictionary_v0_1.md#ic-48), [IC-49](thermodynamics_semantic_information_dictionary_v0_1.md#ic-49), [IC-50](thermodynamics_semantic_information_dictionary_v0_1.md#ic-50), [IC-52](thermodynamics_semantic_information_dictionary_v0_1.md#ic-52), [IC-57](thermodynamics_semantic_information_dictionary_v0_1.md#ic-57).

**Preconditions:** Host-owned equations, local unknowns, bulk/interface roles, transfer conventions and initializer/restoration obligations are stated.

**Procedure and information flow:**

1. P07 binds local contribution contracts to host variables with no duplicate physical constraints.
2. P08 supplies only qualified values/equations/residuals/derivatives under trial coordinates.
3. Keep separate bulk temperatures/compositions and interface assumptions where the selected formulation requires them.
4. The host solver coordinates iterations, geometry, residence time and transfer laws.
5. Restore original formulation and pass combined unit residual evidence to P09 before publication.

**Successful outputs:** [IC-35](thermodynamics_semantic_information_dictionary_v0_1.md#ic-35), [IC-40](thermodynamics_semantic_information_dictionary_v0_1.md#ic-40), [IC-52](thermodynamics_semantic_information_dictionary_v0_1.md#ic-52), [IC-56](thermodynamics_semantic_information_dictionary_v0_1.md#ic-56). A coupled candidate set with exact formulation and actual contribution capabilities; partial states remain usable within the run.

**Permitted effects:** Create captured run-local views, demands or proposals; the host process solver retains equipment/time integration.

**Adoption/completion boundary:** Compatible local candidates may drive iteration without global publication; AC-49 alone adopts a qualified current group.

**Failure and recovery:** A value-only provider cannot satisfy missing derivatives silently. Efficiency-corrected equilibrium stages cannot pass the full nonequilibrium formulation by label.

**Repeated delivery/re-execution:** Repeated callbacks read a captured trial view and do not overwrite source specifications. A new trial is not a new physical time step unless the host says so.

**AW-43-P:** Distinct bulk/interface descriptions remain distinct while interfacial transfer balances are checked.
**AW-43-N:** A property refresh cannot homogenize both phases into one bulk equilibrium as a convenience.

**Primary B4 requirements:** [FR-FLW-07](thermodynamics_functional_requirements_v0_1.md#fr-flw-07), [FR-FLW-08](thermodynamics_functional_requirements_v0_1.md#fr-flw-08). **Supporting requirements:** None.

**Common action rules:** [AR-10](#ar-10). **Handoff facts:** Local proposal/candidate handoff under the common contract.

**Evidence status:** Action specified. Its positive/negative witnesses have not been executed against the simulator or a thermodynamic provider. Primary ownership is inherited from B5; information and invariants are inherited from the linked B6 concepts.

<a id="ac-44"></a>
### AC-44. Advance run-local candidates and evaluate parent convergence

**Accountable owner:** P07, Flowsheet integration and use-case coordination. **Action kind:** coordinate.

**Initiator/trigger:** A unit or flowsheet solver completes one local iteration or recycle pass.

**Inputs:** [IC-35](thermodynamics_semantic_information_dictionary_v0_1.md#ic-35), [IC-41](thermodynamics_semantic_information_dictionary_v0_1.md#ic-41), [IC-52](thermodynamics_semantic_information_dictionary_v0_1.md#ic-52), [IC-56](thermodynamics_semantic_information_dictionary_v0_1.md#ic-56), [IC-59](thermodynamics_semantic_information_dictionary_v0_1.md#ic-59), [IC-60](thermodynamics_semantic_information_dictionary_v0_1.md#ic-60), [IC-64](thermodynamics_semantic_information_dictionary_v0_1.md#ic-64), [IC-70](thermodynamics_semantic_information_dictionary_v0_1.md#ic-70).

**Preconditions:** Parent scope, run authority, captured physical configuration, trial-input provenance and convergence criteria are explicit.

**Procedure and information flow:**

1. P07 incorporates only compatible run-local candidates, keeping guesses and checked states distinct.
2. A new local evaluation captures the actual trial inputs even when the base model revision is unchanged.
3. The host computes stage/unit/recycle residuals and required convergence.
4. Continue under bounded policy or request group qualification.
5. Do not publish the last local values as a converged parent after pass exhaustion.

**Successful outputs:** [IC-52](thermodynamics_semantic_information_dictionary_v0_1.md#ic-52). A coherent next iteration view or a parent candidate with explicit convergence evidence.

**Permitted effects:** Create captured run-local views, demands or proposals; the host process solver retains equipment/time integration.

**Adoption/completion boundary:** Compatible local candidates may drive iteration without global publication; AC-49 alone adopts a qualified current group.

**Failure and recovery:** Mixing candidates from incompatible definition contexts or superseded iteration dependencies is prohibited. Parent nonconvergence leaves diagnostic child states available only at their achieved scope.

**Repeated delivery/re-execution:** Iteration identity prevents repeated incorporation of one completion as multiple advances. Branch-dependent warm starts do not grant parent acceptance.

**AW-44-P:** A converged flash remains locally useful during an unconverged recycle.
**AW-44-N:** Two outputs from incompatible runs cannot be called one accepted exchanger/unit result.

**Primary B4 requirements:** [FR-RUN-08](thermodynamics_functional_requirements_v0_1.md#fr-run-08). **Supporting requirements:** None.

**Common action rules:** [AR-09](#ar-09), [AR-10](#ar-10). **Handoff facts:** Local proposal/candidate handoff under the common contract.

**Evidence status:** Action specified. Its positive/negative witnesses have not been executed against the simulator or a thermodynamic provider. Primary ownership is inherited from B5; information and invariants are inherited from the linked B6 concepts.

<a id="ac-45"></a>
### AC-45. Issue scoped run authority and capture dependencies

**Accountable owner:** P10, Model revisions, publication and reconstruction. **Action kind:** commit.

**Initiator/trigger:** A coordinator starts a local solve, whole-flowsheet run, fitting campaign or reproduction.

**Inputs:** [IC-35](thermodynamics_semantic_information_dictionary_v0_1.md#ic-35), [IC-48](thermodynamics_semantic_information_dictionary_v0_1.md#ic-48), [IC-52](thermodynamics_semantic_information_dictionary_v0_1.md#ic-52), [IC-64](thermodynamics_semantic_information_dictionary_v0_1.md#ic-64), [IC-70](thermodynamics_semantic_information_dictionary_v0_1.md#ic-70).

**Preconditions:** Target branch/location/scope and approved or explicit trial context are supplied. Child scope is bounded by the parent run.

**Procedure and information flow:**

1. P10 captures exact relevant dependencies or a conservative context.
2. Issue separate run identity and publication permissions, with parent inheritance where used.
3. Give fitting/evidence trials no implicit production-publication authority.
4. P07 propagates the captured run to child attempts and completion groups.

**Successful outputs:** [IC-64](thermodynamics_semantic_information_dictionary_v0_1.md#ic-64), [IC-70](thermodynamics_semantic_information_dictionary_v0_1.md#ic-70). A live run permission and immutable dependency capture, distinct from native session and attempt identity.

**Permitted effects:** Only the named P10 authority transition may change its current references or permission facts under its own guards.

**Adoption/completion boundary:** No scientifically consequential current write occurs before the documented logical commit point; replay returns a prior decision, not a second effect.

**Failure and recovery:** Unknown relevant dependencies require conservative capture. A cancelled parent cannot authorize an active child publication; a new run needs fresh authority.

**Repeated delivery/re-execution:** Duplicate start-command delivery reuses its issued run decision. A deliberately new solve has a new run/generation even under identical physical inputs.

**AW-45-P:** A same-revision new run supersedes a prior run under an explicit replacement policy.
**AW-45-N:** A trial parameter-fit run cannot publish its candidate package to production.

**Primary B4 requirements:** None. **Supporting requirements:** [FR-RES-05](thermodynamics_functional_requirements_v0_1.md#fr-res-05), [FR-RUN-06](thermodynamics_functional_requirements_v0_1.md#fr-run-06), [FR-RUN-08](thermodynamics_functional_requirements_v0_1.md#fr-run-08).

**Common action rules:** [AR-01](#ar-01), [AR-06](#ar-06), [AR-09](#ar-09). **Handoff facts:** EV-07

**Evidence status:** Action specified. Its positive/negative witnesses have not been executed against the simulator or a thermodynamic provider. Primary ownership is inherited from B5; information and invariants are inherited from the linked B6 concepts.

<a id="ac-46"></a>
### AC-46. Cancel or supersede a run and revoke future publication

**Accountable owner:** P10, Model revisions, publication and reconstruction. **Action kind:** commit.

**Initiator/trigger:** An authorized caller cancels a run, replaces it with newer work, or closes its permitted scope.

**Inputs:** [IC-52](thermodynamics_semantic_information_dictionary_v0_1.md#ic-52), [IC-54](thermodynamics_semantic_information_dictionary_v0_1.md#ic-54), [IC-64](thermodynamics_semantic_information_dictionary_v0_1.md#ic-64), [IC-66](thermodynamics_semantic_information_dictionary_v0_1.md#ic-66), [IC-67](thermodynamics_semantic_information_dictionary_v0_1.md#ic-67), [IC-70](thermodynamics_semantic_information_dictionary_v0_1.md#ic-70).

**Preconditions:** Target run, parent/child relation and cancellation/supersession intent are explicit.

**Procedure and information flow:**

1. P10 records revocation at a defined logical point; effective child permission includes every relevant ancestor.
2. P07 stops scheduling new useful work and requests P08 cancellation/cleanup.
3. Retain late candidate evidence only under the revoked run.
4. AC-49 checks live permission again at commit regardless of actual native termination.

**Successful outputs:** [IC-70](thermodynamics_semantic_information_dictionary_v0_1.md#ic-70), [IC-66](thermodynamics_semantic_information_dictionary_v0_1.md#ic-66). Future publication is disallowed for the revoked scope; an already completed publication is not retroactively numerically invalidated by a later cancel.

**Permitted effects:** Only the named P10 authority transition may change its current references or permission facts under its own guards.

**Adoption/completion boundary:** No scientifically consequential current write occurs before the documented logical commit point; replay returns a prior decision, not a second effect.

**Failure and recovery:** An uninterruptible native call may continue physically but cannot publish afterward. Explicit withdrawal of an already published binding is a separate guarded change, not assumed cancellation semantics.

**Repeated delivery/re-execution:** Repeating cancellation is idempotent. A duplicate late result cannot reactivate the run. Cancellation never silently transfers permission to a new run.

**AW-46-P:** Same-revision late completion is rejected after cancellation, including a child whose parent was cancelled.
**AW-46-N:** Revision equality alone cannot allow publication from a superseded run.

**Primary B4 requirements:** None. **Supporting requirements:** [FR-RUN-06](thermodynamics_functional_requirements_v0_1.md#fr-run-06), [FR-RES-05](thermodynamics_functional_requirements_v0_1.md#fr-res-05).

**Common action rules:** [AR-06](#ar-06), [AR-08](#ar-08), [AR-09](#ar-09). **Handoff facts:** EV-08

**Evidence status:** Action specified. Its positive/negative witnesses have not been executed against the simulator or a thermodynamic provider. Primary ownership is inherited from B5; information and invariants are inherited from the linked B6 concepts.

<a id="ac-47"></a>
### AC-47. Prepare a coordinated change with domain validation and impact

**Accountable owner:** P10, Model revisions, publication and reconstruction. **Action kind:** prepare.

**Initiator/trigger:** Any interface proposes composition, slate, parameters, chemistry, reference, binding, provider or presentation changes.

**Inputs:** [IC-03](thermodynamics_semantic_information_dictionary_v0_1.md#ic-03), [IC-14](thermodynamics_semantic_information_dictionary_v0_1.md#ic-14), [IC-17](thermodynamics_semantic_information_dictionary_v0_1.md#ic-17), [IC-20](thermodynamics_semantic_information_dictionary_v0_1.md#ic-20), [IC-27](thermodynamics_semantic_information_dictionary_v0_1.md#ic-27), [IC-28](thermodynamics_semantic_information_dictionary_v0_1.md#ic-28), [IC-45](thermodynamics_semantic_information_dictionary_v0_1.md#ic-45), [IC-49](thermodynamics_semantic_information_dictionary_v0_1.md#ic-49), [IC-53](thermodynamics_semantic_information_dictionary_v0_1.md#ic-53), [IC-64](thermodynamics_semantic_information_dictionary_v0_1.md#ic-64), [IC-65](thermodynamics_semantic_information_dictionary_v0_1.md#ic-65), [IC-72](thermodynamics_semantic_information_dictionary_v0_1.md#ic-72).

**Preconditions:** Expected prior context, full proposed change set, intent and material-preservation policy where applicable are supplied.

**Procedure and information flow:**

1. P10 obtains assessments from each semantic owner and identifies dependent references.
2. Distinguish physical, numerical, realization and presentation consequences.
3. For slate changes repair or explicitly unresolve reactions, phase eligibility, coordinate maps and related data.
4. Prepare precise or conservative invalidation plus required session reconstruction.
5. Return one change proposal; retain inspectable drafts but do not label unresolved dependents ready.

**Successful outputs:** [IC-65](thermodynamics_semantic_information_dictionary_v0_1.md#ic-65). An assessed coherent proposal and impact preview before live adoption.

**Permitted effects:** Only private proposals, snapshots and assessments may be built; no approved model/result mutation.

**Adoption/completion boundary:** A proposal becomes a current model definition only through AC-47/48; it conveys no numerical acceptance.

**Failure and recovery:** A stale proposal requires explicit rebase/revalidation. Conflicting material preservation or unresolved mandatory cross-references cannot be hidden by partial adoption.

**Repeated delivery/re-execution:** Same command/payload and expected context identify one proposal. A different payload under the same command identity is a conflict.

**AW-47-P:** Removing A while preserving B flow revises totals and exposes every affected reaction/map.
**AW-47-N:** A GUI removal and API removal cannot silently use different preservation policies.

**Primary B4 requirements:** [FR-LIF-02](thermodynamics_functional_requirements_v0_1.md#fr-lif-02), [FR-LIF-03](thermodynamics_functional_requirements_v0_1.md#fr-lif-03). **Supporting requirements:** None.

**Common action rules:** [AR-17](#ar-17). **Handoff facts:** Local proposal/candidate handoff under the common contract.

**Evidence status:** Action specified. Its positive/negative witnesses have not been executed against the simulator or a thermodynamic provider. Primary ownership is inherited from B5; information and invariants are inherited from the linked B6 concepts.

<a id="ac-48"></a>
### AC-48. Commit a definition change and invalidate dependent authority

**Accountable owner:** P10, Model revisions, publication and reconstruction. **Action kind:** commit.

**Initiator/trigger:** An authorized assessed change is ready for live adoption.

**Inputs:** [IC-64](thermodynamics_semantic_information_dictionary_v0_1.md#ic-64), [IC-65](thermodynamics_semantic_information_dictionary_v0_1.md#ic-65), [IC-67](thermodynamics_semantic_information_dictionary_v0_1.md#ic-67), [IC-70](thermodynamics_semantic_information_dictionary_v0_1.md#ic-70).

**Preconditions:** Expected prior context still matches; domain validation and impact policy are complete for the change being committed. An explicitly incomplete model can be committed only as not ready.

**Procedure and information flow:**

1. P10 compares expected/current context at the logical commit boundary.
2. Adopt the coherent definition/binding/input change set and new revision together.
3. In the same authority transition, invalidate or require reassessment of affected current results and prevent old captured contexts from publishing as new.
4. Mark sessions incompatible for future use; actual native cleanup may follow asynchronously inside an implementation, without a safety gap at acquisition.
5. Emit one attributable decision through every interface.

**Successful outputs:** [IC-64](thermodynamics_semantic_information_dictionary_v0_1.md#ic-64), [IC-65](thermodynamics_semantic_information_dictionary_v0_1.md#ic-65), [IC-67](thermodynamics_semantic_information_dictionary_v0_1.md#ic-67), [IC-70](thermodynamics_semantic_information_dictionary_v0_1.md#ic-70). A coherent new revision and stale/needs-reassessment state, or no live change if rejected.

**Permitted effects:** Only the named P10 authority transition may change its current references or permission facts under its own guards.

**Adoption/completion boundary:** No scientifically consequential current write occurs before the documented logical commit point; replay returns a prior decision, not a second effect.

**Failure and recovery:** Conflict produces no partial live writes; reevaluate/rebase explicitly. If precise dependency impact is unavailable, invalidate conservatively. Changed definitions cannot retain falsely current results while notifications lag.

**Repeated delivery/re-execution:** Replay of a committed command returns its decision before checking the now-new revision. Same identifier with changed payload is rejected; no automatic last-writer-wins.

**AW-48-P:** A binary-parameter edit prevents old results and sessions from being returned as current without reassessment.
**AW-48-N:** Losing the commit response then repeating the command cannot apply a component increment twice.

**Primary B4 requirements:** [FR-LIF-01](thermodynamics_functional_requirements_v0_1.md#fr-lif-01), [FR-LIF-08](thermodynamics_functional_requirements_v0_1.md#fr-lif-08). **Supporting requirements:** None.

**Common action rules:** [AR-02](#ar-02), [AR-05](#ar-05), [AR-06](#ar-06), [AR-17](#ar-17). **Handoff facts:** EV-03

**Evidence status:** Action specified. Its positive/negative witnesses have not been executed against the simulator or a thermodynamic provider. Primary ownership is inherited from B5; information and invariants are inherited from the linked B6 concepts.

<a id="ac-49"></a>
### AC-49. Publish a qualified coherent result set with live guards

**Accountable owner:** P10, Model revisions, publication and reconstruction. **Action kind:** commit.

**Initiator/trigger:** P07 proposes a P09-qualified result or result group as current at named locations/scopes.

**Inputs:** [IC-48](thermodynamics_semantic_information_dictionary_v0_1.md#ic-48), [IC-52](thermodynamics_semantic_information_dictionary_v0_1.md#ic-52), [IC-60](thermodynamics_semantic_information_dictionary_v0_1.md#ic-60), [IC-64](thermodynamics_semantic_information_dictionary_v0_1.md#ic-64), [IC-66](thermodynamics_semantic_information_dictionary_v0_1.md#ic-66), [IC-67](thermodynamics_semantic_information_dictionary_v0_1.md#ic-67), [IC-70](thermodynamics_semantic_information_dictionary_v0_1.md#ic-70).

**Preconditions:** Original problem(s), immutable candidate set, achieved scope, target slots, expected binding generation and dependency/run authority are supplied.

**Procedure and information flow:**

1. P10 checks required qualification and exact publication kind, candidate-group coherence, captured relevant dependencies and current target context.
2. Check live run and ancestor permission, supersession and target-slot generation at the same logical point as updates.
3. Commit the whole declared group and decision together, or change no current binding.
4. Keep rejected candidates as attributable history; do not change their scientific verdict to conceal obsolete context.

**Successful outputs:** [IC-66](thermodynamics_semantic_information_dictionary_v0_1.md#ic-66), [IC-67](thermodynamics_semantic_information_dictionary_v0_1.md#ic-67). One committed current association per target slot at its declared scope, or a stale/cancelled/superseded/conflict/scope rejection.

**Permitted effects:** Only the named P10 authority transition may change its current references or permission facts under its own guards.

**Adoption/completion boundary:** No scientifically consequential current write occurs before the documented logical commit point; replay returns a prior decision, not a second effect.

**Failure and recovery:** A failed member blocks a required coherent group. Local acceptance is insufficient for a requested flowsheet slot. Unknown commit delivery is resolved by querying the original decision, not by constructing a new destructive command.

**Repeated delivery/re-execution:** Same publication command and identical payload returns its recorded decision without rewriting current state. Replaying an old successful decision after a newer result must not restore the old binding.

**AW-49-P:** Both exchanger outlets publish together only when the group and revision/run guards pass.
**AW-49-N:** A late A result, cancelled child or stale slot generation cannot overwrite a newer current result even with small residuals.

**Primary B4 requirements:** [FR-RES-05](thermodynamics_functional_requirements_v0_1.md#fr-res-05). **Supporting requirements:** None.

**Common action rules:** [AR-02](#ar-02), [AR-05](#ar-05), [AR-06](#ar-06), [AR-09](#ar-09), [AR-10](#ar-10). **Handoff facts:** EV-15

**Evidence status:** Action specified. Its positive/negative witnesses have not been executed against the simulator or a thermodynamic provider. Primary ownership is inherited from B5; information and invariants are inherited from the linked B6 concepts.

<a id="ac-50"></a>
### AC-50. Export a coherent semantic archive without running thermodynamics

**Accountable owner:** P10, Model revisions, publication and reconstruction. **Action kind:** export.

**Initiator/trigger:** A caller saves a case, package or historical result context.

**Inputs:** [IC-03](thermodynamics_semantic_information_dictionary_v0_1.md#ic-03), [IC-10](thermodynamics_semantic_information_dictionary_v0_1.md#ic-10), [IC-14](thermodynamics_semantic_information_dictionary_v0_1.md#ic-14), [IC-16](thermodynamics_semantic_information_dictionary_v0_1.md#ic-16), [IC-17](thermodynamics_semantic_information_dictionary_v0_1.md#ic-17), [IC-20](thermodynamics_semantic_information_dictionary_v0_1.md#ic-20), [IC-28](thermodynamics_semantic_information_dictionary_v0_1.md#ic-28), [IC-35](thermodynamics_semantic_information_dictionary_v0_1.md#ic-35), [IC-45](thermodynamics_semantic_information_dictionary_v0_1.md#ic-45), [IC-49](thermodynamics_semantic_information_dictionary_v0_1.md#ic-49), [IC-53](thermodynamics_semantic_information_dictionary_v0_1.md#ic-53), [IC-60](thermodynamics_semantic_information_dictionary_v0_1.md#ic-60), [IC-62](thermodynamics_semantic_information_dictionary_v0_1.md#ic-62), [IC-64](thermodynamics_semantic_information_dictionary_v0_1.md#ic-64), [IC-67](thermodynamics_semantic_information_dictionary_v0_1.md#ic-67), [IC-70](thermodynamics_semantic_information_dictionary_v0_1.md#ic-70).

**Preconditions:** A fixed snapshot or a consistency-verifiable capture boundary is available; reconstructable semantics and unresolved dependencies are identifiable.

**Procedure and information flow:**

1. P10 captures the selected definition revision and exact domain snapshots/manifests.
2. Export original data/assays, recipes, mappings, conventions, bindings and input roles; record opaque or external dependencies.
3. Store results with their exact provenance separately from definitions and discardable acceleration data.
4. Verify capture consistency and serialization completeness before exposing a complete archive.

**Successful outputs:** [IC-68](thermodynamics_semantic_information_dictionary_v0_1.md#ic-68). A coherent inspectable archive whose saved results do not automatically carry live run authority.

**Permitted effects:** Read a coherent captured semantic snapshot; serialization cannot prepare new physical data.

**Adoption/completion boundary:** Expose only a consistent archive, or an explicitly incomplete export with missing dependencies stated.

**Failure and recovery:** Concurrent changes cannot produce a mixed-revision file. Use a retained snapshot, restart capture or fail explicitly. Missing external artifacts limit reconstruction, not inspection; saving never launches fitting.

**Repeated delivery/re-execution:** Saving is observational; identical captures may yield different container timestamps but identical declared semantic content. No model mutation on repeated save.

**AW-50-P:** An assay archive retains original measurements, generated cuts and complete package references.
**AW-50-N:** Final arrays and a native handle alone cannot count as a reconstructable semantic model.

**Primary B4 requirements:** [FR-LIF-04](thermodynamics_functional_requirements_v0_1.md#fr-lif-04). **Supporting requirements:** None.

**Common action rules:** [AR-01](#ar-01), [AR-16](#ar-16). **Handoff facts:** EV-16

**Evidence status:** Action specified. Its positive/negative witnesses have not been executed against the simulator or a thermodynamic provider. Primary ownership is inherited from B5; information and invariants are inherited from the linked B6 concepts.

<a id="ac-51"></a>
### AC-51. Inspect, rebuild and explicitly migrate a saved model

**Accountable owner:** P10, Model revisions, publication and reconstruction. **Action kind:** restore.

**Initiator/trigger:** A loader opens a saved case or rebuilds a historical model.

**Inputs:** [IC-68](thermodynamics_semantic_information_dictionary_v0_1.md#ic-68), [IC-03](thermodynamics_semantic_information_dictionary_v0_1.md#ic-03), [IC-14](thermodynamics_semantic_information_dictionary_v0_1.md#ic-14), [IC-20](thermodynamics_semantic_information_dictionary_v0_1.md#ic-20), [IC-28](thermodynamics_semantic_information_dictionary_v0_1.md#ic-28), [IC-45](thermodynamics_semantic_information_dictionary_v0_1.md#ic-45), [IC-49](thermodynamics_semantic_information_dictionary_v0_1.md#ic-49), [IC-53](thermodynamics_semantic_information_dictionary_v0_1.md#ic-53), [IC-62](thermodynamics_semantic_information_dictionary_v0_1.md#ic-62), [IC-64](thermodynamics_semantic_information_dictionary_v0_1.md#ic-64).

**Preconditions:** Archive identity/semantic version and available domain interpreters are known; original evidence is retained.

**Procedure and information flow:**

1. P10 opens for inspection independently of dependency availability.
2. Domain owners reconstruct identities, coordinates, chemistry, parameters and package meaning before applying saved numeric values.
3. Record every format/semantic migration and obtain authorization for consequential changes.
4. P08 resolves actual dependency availability and fresh sessions only when qualified.
5. Reassess readiness and restored-result compatibility; publish only via a fresh permitted AC-49 path.

**Successful outputs:** [IC-69](thermodynamics_semantic_information_dictionary_v0_1.md#ic-69), [IC-64](thermodynamics_semantic_information_dictionary_v0_1.md#ic-64). Separate inspection, semantic reconstruction, dependency, readiness and historical-result status; original run permission is not resurrected.

**Permitted effects:** Reconstruct and assess in an isolated target/proposal before guarded live adoption.

**Adoption/completion boundary:** Any model/current-result adoption still uses AC-48/49; saved run permissions and native handles cannot be reactivated.

**Failure and recovery:** Unknown extension or absent engine remains explicit and can block affected execution. Migration must not silently substitute physics; value-only restore into changed equations cannot confer authority.

**Repeated delivery/re-execution:** Repeated loading can create a new workspace/branch but cannot replay physical transfers or reactivate old cancelled runs. Same archive is not a live command log.

**AW-51-P:** A missing-provider case is inspectable while its affected calculations remain blocked.
**AW-51-N:** Restoring numeric values into a different equation recipe cannot be declared original-case reproduction.

**Primary B4 requirements:** [FR-LIF-05](thermodynamics_functional_requirements_v0_1.md#fr-lif-05). **Supporting requirements:** None.

**Common action rules:** [AR-16](#ar-16). **Handoff facts:** EV-17

**Evidence status:** Action specified. Its positive/negative witnesses have not been executed against the simulator or a thermodynamic provider. Primary ownership is inherited from B5; information and invariants are inherited from the linked B6 concepts.

<a id="ac-52"></a>
### AC-52. Restore or undo a snapshot as a guarded semantic change

**Accountable owner:** P10, Model revisions, publication and reconstruction. **Action kind:** restore.

**Initiator/trigger:** An author requests undo, redo or recovery to a retained semantic snapshot.

**Inputs:** [IC-64](thermodynamics_semantic_information_dictionary_v0_1.md#ic-64), [IC-65](thermodynamics_semantic_information_dictionary_v0_1.md#ic-65), [IC-67](thermodynamics_semantic_information_dictionary_v0_1.md#ic-67), [IC-68](thermodynamics_semantic_information_dictionary_v0_1.md#ic-68), [IC-69](thermodynamics_semantic_information_dictionary_v0_1.md#ic-69).

**Preconditions:** Source snapshot, current target branch/context, shared identity links and intended quantity treatment are explicit.

**Procedure and information flow:**

1. P10 constructs a change proposal restoring the relevant domain content and coordinate relationships.
2. Reassess dependent packages/reactions and session/current-result compatibility.
3. Commit through AC-48 against the current expected context.
4. Retain history and use fresh run/publication authority if saved results are proposed as current.

**Successful outputs:** [IC-65](thermodynamics_semantic_information_dictionary_v0_1.md#ic-65), [IC-69](thermodynamics_semantic_information_dictionary_v0_1.md#ic-69), [IC-64](thermodynamics_semantic_information_dictionary_v0_1.md#ic-64). A controlled restoration with original scientific meaning retained and currentness reassessed.

**Permitted effects:** Reconstruct and assess in an isolated target/proposal before guarded live adoption.

**Adoption/completion boundary:** Any model/current-result adoption still uses AC-48/49; saved run permissions and native handles cannot be reactivated.

**Failure and recovery:** Repairing only a component list while leaving changed-order buffers/maps is insufficient. Undo cannot silently erase lineage or reactivate an obsolete run.

**Repeated delivery/re-execution:** An undo command restores its identified snapshot once; repeated delivery cannot walk back another history entry. A subsequent undo is a different command.

**AW-52-P:** Undoing a slate change restores material definitions and verifies all related coordinate and reaction links.
**AW-52-N:** A stale snapshot result cannot become current under a different unresolved parameter revision.

**Primary B4 requirements:** [FR-LIF-06](thermodynamics_functional_requirements_v0_1.md#fr-lif-06). **Supporting requirements:** None.

**Common action rules:** [AR-06](#ar-06), [AR-16](#ar-16). **Handoff facts:** EV-17

**Evidence status:** Action specified. Its positive/negative witnesses have not been executed against the simulator or a thermodynamic provider. Primary ownership is inherited from B5; information and invariants are inherited from the linked B6 concepts.

<a id="ac-53"></a>
### AC-53. Reproduce a recorded case or classify a changed-model comparison

**Accountable owner:** P10, Model revisions, publication and reconstruction. **Action kind:** restore.

**Initiator/trigger:** A reviewer requests a rerun of a saved result or a comparison after changes.

**Inputs:** [IC-20](thermodynamics_semantic_information_dictionary_v0_1.md#ic-20), [IC-35](thermodynamics_semantic_information_dictionary_v0_1.md#ic-35), [IC-53](thermodynamics_semantic_information_dictionary_v0_1.md#ic-53), [IC-60](thermodynamics_semantic_information_dictionary_v0_1.md#ic-60), [IC-61](thermodynamics_semantic_information_dictionary_v0_1.md#ic-61), [IC-62](thermodynamics_semantic_information_dictionary_v0_1.md#ic-62), [IC-64](thermodynamics_semantic_information_dictionary_v0_1.md#ic-64), [IC-68](thermodynamics_semantic_information_dictionary_v0_1.md#ic-68), [IC-69](thermodynamics_semantic_information_dictionary_v0_1.md#ic-69), [IC-72](thermodynamics_semantic_information_dictionary_v0_1.md#ic-72).

**Preconditions:** Recorded model/data/provider context, numerical/branch policy and predeclared comparison tolerances are available or explicitly missing.

**Procedure and information flow:**

1. P10 identifies matching and changed dependencies and the intended claim.
2. P07 issues a new run and P08 rebuilds compatible sessions.
3. Execute using the ordinary preparation/calculation/check path, then P09 assesses differences and evidence independence.
4. Record exact-context reproduction, changed-model comparison or blocked/unassessed status.

**Successful outputs:** [IC-69](thermodynamics_semantic_information_dictionary_v0_1.md#ic-69). An attributable comparison with declared tolerances, not an assumed bitwise identity guarantee.

**Permitted effects:** Reconstruct and assess in an isolated target/proposal before guarded live adoption.

**Adoption/completion boundary:** Any model/current-result adoption still uses AC-48/49; saved run permissions and native handles cannot be reactivated.

**Failure and recovery:** Unavailable original artifacts prevent an unqualified exact-context reproduction claim. Improved numerical results from a new database do not retroactively change old provenance.

**Repeated delivery/re-execution:** Each intentional rerun has a new run/attempt; repeated delivery of one comparison request reconciles that identity rather than manufacturing independent evidence.

**AW-53-P:** The same captured semantic inputs can be compared within stated reproduction tolerance.
**AW-53-N:** A changed parameter database cannot be called an unchanged-model reproduction.

**Primary B4 requirements:** [FR-LIF-07](thermodynamics_functional_requirements_v0_1.md#fr-lif-07). **Supporting requirements:** None.

**Common action rules:** [AR-16](#ar-16), [AR-18](#ar-18). **Handoff facts:** EV-17

**Evidence status:** Action specified. Its positive/negative witnesses have not been executed against the simulator or a thermodynamic provider. Primary ownership is inherited from B5; information and invariants are inherited from the linked B6 concepts.

<a id="ac-54"></a>
### AC-54. Authorize a distinct changed-physics alternative

**Accountable owner:** P05, Thermodynamic problems and operation contracts. **Action kind:** prepare.

**Initiator/trigger:** An author or explicit policy permits an alternate model/phase/chemistry treatment after a limitation or failure.

**Inputs:** [IC-20](thermodynamics_semantic_information_dictionary_v0_1.md#ic-20), [IC-35](thermodynamics_semantic_information_dictionary_v0_1.md#ic-35), [IC-36](thermodynamics_semantic_information_dictionary_v0_1.md#ic-36), [IC-37](thermodynamics_semantic_information_dictionary_v0_1.md#ic-37), [IC-41](thermodynamics_semantic_information_dictionary_v0_1.md#ic-41), [IC-42](thermodynamics_semantic_information_dictionary_v0_1.md#ic-42), [IC-45](thermodynamics_semantic_information_dictionary_v0_1.md#ic-45), [IC-56](thermodynamics_semantic_information_dictionary_v0_1.md#ic-56), [IC-63](thermodynamics_semantic_information_dictionary_v0_1.md#ic-63), [IC-72](thermodynamics_semantic_information_dictionary_v0_1.md#ic-72).

**Preconditions:** Original problem/outcome and precise proposed departures are retained; authorization is specific to changed physics and allowed result use.

**Procedure and information flow:**

1. P05 constructs a new physical problem linked to the original.
2. Record different method/data/phase/species restrictions and revised required checks.
3. P03 requalifies compatibility; P07 obtains appropriate run scope and P08 executes a fresh attempt.
4. P09 labels acceptance for the alternate problem only; original failure/coverage gap remains visible.

**Successful outputs:** [IC-35](thermodynamics_semantic_information_dictionary_v0_1.md#ic-35), [IC-36](thermodynamics_semantic_information_dictionary_v0_1.md#ic-36), [IC-37](thermodynamics_semantic_information_dictionary_v0_1.md#ic-37), [IC-41](thermodynamics_semantic_information_dictionary_v0_1.md#ic-41), [IC-42](thermodynamics_semantic_information_dictionary_v0_1.md#ic-42). A usable explicitly alternative calculation where authorized, never a rewrite of the original success conditions.

**Permitted effects:** Only private proposals, snapshots and assessments may be built; no approved model/result mutation.

**Adoption/completion boundary:** A proposal becomes a current model definition only through AC-47/48; it conveys no numerical acceptance.

**Failure and recovery:** No authorization means no alternate execution or only an unaccepted proposal. A numerical policy entry cannot by itself justify claiming original-problem success.

**Repeated delivery/re-execution:** Replay of the same authorization yields the same alternate proposal; changing the proposed physics creates another distinct problem.

**AW-54-P:** A solid-suppressed alternative remains labeled restricted and linked to the original unrestricted request.
**AW-54-N:** A default ideal fallback cannot erase the original nonideal multiphase failure.

**Primary B4 requirements:** None. **Supporting requirements:** [FR-CFG-05](thermodynamics_functional_requirements_v0_1.md#fr-cfg-05), [FR-RUN-05](thermodynamics_functional_requirements_v0_1.md#fr-run-05), [FR-EXT-04](thermodynamics_functional_requirements_v0_1.md#fr-ext-04).

**Common action rules:** [AR-11](#ar-11). **Handoff facts:** EV-05

**Evidence status:** Action specified. Its positive/negative witnesses have not been executed against the simulator or a thermodynamic provider. Primary ownership is inherited from B5; information and invariants are inherited from the linked B6 concepts.

<a id="ac-55"></a>
### AC-55. Obtain missing check evidence without changing the checked candidate

**Accountable owner:** P07, Flowsheet integration and use-case coordination. **Action kind:** coordinate.

**Initiator/trigger:** P09 identifies missing mandatory or requested optional evaluation evidence.

**Inputs:** [IC-35](thermodynamics_semantic_information_dictionary_v0_1.md#ic-35), [IC-38](thermodynamics_semantic_information_dictionary_v0_1.md#ic-38), [IC-39](thermodynamics_semantic_information_dictionary_v0_1.md#ic-39), [IC-41](thermodynamics_semantic_information_dictionary_v0_1.md#ic-41), [IC-52](thermodynamics_semantic_information_dictionary_v0_1.md#ic-52), [IC-56](thermodynamics_semantic_information_dictionary_v0_1.md#ic-56), [IC-59](thermodynamics_semantic_information_dictionary_v0_1.md#ic-59), [IC-60](thermodynamics_semantic_information_dictionary_v0_1.md#ic-60), [IC-64](thermodynamics_semantic_information_dictionary_v0_1.md#ic-64), [IC-70](thermodynamics_semantic_information_dictionary_v0_1.md#ic-70).

**Preconditions:** The checked candidate, original criterion, evidence dependency and remaining budget are fixed; the demand cannot depend circularly on its own acceptance verdict.

**Procedure and information flow:**

1. P07 creates a bounded verification subrequest under the original material and convention.
2. Use property-only authority where reevaluation must preserve a supplied state; require the actual branch semantics.
3. Obtain P08 evidence through a distinct attempt and return it to P09.
4. Preserve same-model versus independent evidence identity and prevent unbounded evidence-request loops.

**Successful outputs:** [IC-35](thermodynamics_semantic_information_dictionary_v0_1.md#ic-35), [IC-52](thermodynamics_semantic_information_dictionary_v0_1.md#ic-52), [IC-56](thermodynamics_semantic_information_dictionary_v0_1.md#ic-56). Additional evidence linked to the original check, or an explicit unavailable-evidence outcome.

**Permitted effects:** Create captured run-local views, demands or proposals; the host process solver retains equipment/time integration.

**Adoption/completion boundary:** Compatible local candidates may drive iteration without global publication; AC-49 alone adopts a qualified current group.

**Failure and recovery:** A verifier that reflashes away the supplied phase split is not checking that same state. Budget exhaustion leaves mandatory evidence unassessed and acceptance blocked.

**Repeated delivery/re-execution:** Duplicate evidence requests may reuse identical qualified evidence by source identity. Reuse is not independent confirmation and never applies exchanges again.

**AW-55-P:** A PH check obtains enthalpy at the candidate state without replacing the original H target.
**AW-55-N:** A verification subcall cannot change the candidate phase allocation merely to reduce a residual.

**Primary B4 requirements:** None. **Supporting requirements:** [FR-RES-02](thermodynamics_functional_requirements_v0_1.md#fr-res-02), [FR-PRP-02](thermodynamics_functional_requirements_v0_1.md#fr-prp-02), [FR-RUN-08](thermodynamics_functional_requirements_v0_1.md#fr-run-08).

**Common action rules:** [AR-12](#ar-12), [AR-13](#ar-13). **Handoff facts:** EV-13

**Evidence status:** Action specified. Its positive/negative witnesses have not been executed against the simulator or a thermodynamic provider. Primary ownership is inherited from B5; information and invariants are inherited from the linked B6 concepts.

<a id="ac-56"></a>
### AC-56. Query result status and lineage without promoting its authority

**Accountable owner:** P09, Result qualification and coverage evidence. **Action kind:** query.

**Initiator/trigger:** An interface, report or downstream consumer inspects a result or asks for current state.

**Inputs:** [IC-24](thermodynamics_semantic_information_dictionary_v0_1.md#ic-24), [IC-28](thermodynamics_semantic_information_dictionary_v0_1.md#ic-28), [IC-52](thermodynamics_semantic_information_dictionary_v0_1.md#ic-52), [IC-56](thermodynamics_semantic_information_dictionary_v0_1.md#ic-56), [IC-59](thermodynamics_semantic_information_dictionary_v0_1.md#ic-59), [IC-60](thermodynamics_semantic_information_dictionary_v0_1.md#ic-60), [IC-61](thermodynamics_semantic_information_dictionary_v0_1.md#ic-61), [IC-62](thermodynamics_semantic_information_dictionary_v0_1.md#ic-62), [IC-63](thermodynamics_semantic_information_dictionary_v0_1.md#ic-63), [IC-66](thermodynamics_semantic_information_dictionary_v0_1.md#ic-66), [IC-67](thermodynamics_semantic_information_dictionary_v0_1.md#ic-67), [IC-69](thermodynamics_semantic_information_dictionary_v0_1.md#ic-69), [IC-70](thermodynamics_semantic_information_dictionary_v0_1.md#ic-70).

**Preconditions:** The requested scope and historical-versus-current intent are explicit; P10 supplies authoritative currentness/publication facts.

**Procedure and information flow:**

1. P09 composes numerical outcome, check completion, phase stability, applicability, validation, lineage and supplied currentness as separate dimensions.
2. Return unaccepted or historical values only under a clearly requested/qualified view.
3. A strict current query follows the P10 binding and rejects stale/unavailable scope; no silent fallback to last visible numbers.
4. Optional-property additions create linked reporting/evidence, not retrospective mutation of accepted snapshots.

**Successful outputs:** [IC-63](thermodynamics_semantic_information_dictionary_v0_1.md#ic-63). An interpretable status view; merely reading a result changes no definition, calculation or publication.

**Permitted effects:** No definition, calculation, quantity or current-binding mutation.

**Adoption/completion boundary:** Status view is observational; currentness facts come from P10 and can change without changing historical scientific evidence.

**Failure and recovery:** Unknown cause or validity remains unknown. A report cannot call a local candidate a completed plant result or hide alternate-model use.

**Repeated delivery/re-execution:** Queries are observational; time-varying currentness is supplied by P10 and is not cached as permanent scientific truth.

**AW-56-P:** A historically validated but stale state exposes both its historic validity and noncurrent status.
**AW-56-N:** An unqualified green/success indicator for an obsolete or alternate-problem result fails the status contract.

**Primary B4 requirements:** [FR-RES-06](thermodynamics_functional_requirements_v0_1.md#fr-res-06). **Supporting requirements:** None.

**Common action rules:** [AR-09](#ar-09), [AR-17](#ar-17). **Handoff facts:** Local proposal/candidate handoff under the common contract.

**Evidence status:** Action specified. Its positive/negative witnesses have not been executed against the simulator or a thermodynamic provider. Primary ownership is inherited from B5; information and invariants are inherited from the linked B6 concepts.

## 6. Lifecycle views and guarded transitions

These views deliberately separate mutable runtime progression from immutable scientific evidence and currentness. “Transition” can mean appending a new disposition/assessment record rather than mutating an old one. The complete simulator is not a single state machine; these views cooperate through IC-35/54/60/64/66/70. Identity/version fields and independent status dimensions stay intact.

<a id="lc-01"></a>
### LC-01. Definition proposal and revision adoption

**Owner:** P10. **Information:** [IC-64](thermodynamics_semantic_information_dictionary_v0_1.md#ic-64), [IC-65](thermodynamics_semantic_information_dictionary_v0_1.md#ic-65). **Initial view(s):** draft.

| From | To | Trigger | Guard | Action | Effect |
| --- | --- | --- | --- | --- | --- |
| draft | assessed | domain assessment completes | all relevant content owners supplied required semantic assessments | [AC-47](#ac-47) | Freeze the proposal and its expected prior context. |
| draft | blocked | required meaning is absent | specific unresolved domain input recorded | [AC-47](#ac-47) | Retain draft; no live partial change. |
| blocked | draft | new evidence or explicit repair arrives | new proposal revision records added information | [AC-47](#ac-47) | Reassess all affected domain and quantity consequences. |
| assessed | committed | adoption command | authorization and expected/current context agree at commit | [AC-48](#ac-48) | Adopt complete change and invalidate affected current authority together. |
| assessed | conflicted | expected context no longer matches | no exact approved rebase exists | [AC-48](#ac-48) | No current definition write. |
| conflicted | draft | explicit rebase requested | new expected context and reassessment recorded | [AC-47](#ac-47) | Create a new proposal version; retain conflicted predecessor. |
| assessed | abandoned | author declines or campaign stops | proposal has not committed | [AC-46](#ac-46) | Retain diagnostic preparation history. |
| committed | committed | duplicate command delivery | same command and payload, prior recorded decision | [AC-48](#ac-48) | Return prior decision; no second change or invalidation. |

These are dispositions of proposal versions. Committed definition revisions remain immutable; a later edit creates another revision. An incomplete authored model may be deliberately committed with readiness blocked, but never passed off as ready.

<a id="lc-02"></a>
### LC-02. Request preparation and readiness

**Owner:** P05. **Information:** [IC-24](thermodynamics_semantic_information_dictionary_v0_1.md#ic-24), [IC-35](thermodynamics_semantic_information_dictionary_v0_1.md#ic-35), [IC-36](thermodynamics_semantic_information_dictionary_v0_1.md#ic-36), [IC-37](thermodynamics_semantic_information_dictionary_v0_1.md#ic-37). **Initial view(s):** draft.

| From | To | Trigger | Guard | Action | Effect |
| --- | --- | --- | --- | --- | --- |
| draft | partial_coupled | caller requests contribution | unknowns belong to a declared host formulation | [AC-21](#ac-21), [AC-26](#ac-26) | Retain legitimate unknowns and required contribution semantics. |
| draft | blocked | meaning/authority assessment fails | incomplete or conflicting mandatory inputs identified | [AC-21](#ac-21) | No attempt at complete-state acceptance. |
| draft | ready_for_preparation | problem captured and preparation eligible | method/data semantics adequate; initializer may still be needed | [AC-21](#ac-21), [AC-09](#ac-09) | Freeze original physical problem. |
| partial_coupled | ready_for_preparation | host binds unresolved variables | no duplicate/conflicting ownership; actual contribution supported | [AC-26](#ac-26), [AC-43](#ac-43), [AC-09](#ac-09) | Prepare captured contribution context. |
| ready_for_preparation | ready_for_execution | initializer and session prerequisites established | all applicable mandatory dispatch predicates satisfied | [AC-31](#ac-31), [AC-09](#ac-09) | Readiness assessment records exact context. |
| ready_for_preparation | blocked | preparation unavailable or failed | reason distinguished from feasibility | [AC-31](#ac-31), [AC-09](#ac-09) | Do not infer physical infeasibility. |
| ready_for_execution | captured | attempt planned | run is active and captured dependencies match demand | [AC-29](#ac-29) | Create one identified attempt at the frozen problem. |
| captured | blocked | new evidence disproves readiness | failed assumption recorded; original problem retained | [AC-09](#ac-09) | Reject further production dispatch; existing attempts handled by lifecycle guards. |

Readiness is an assessment view, not a single permanent property of a package. The original request is immutable; repaired constraints produce a new request. Initializer/session planning can precede final numerical-dispatch readiness.

<a id="lc-03"></a>
### LC-03. Numerical attempt and completion

**Owner:** P08. **Information:** [IC-54](thermodynamics_semantic_information_dictionary_v0_1.md#ic-54), [IC-56](thermodynamics_semantic_information_dictionary_v0_1.md#ic-56), [IC-57](thermodynamics_semantic_information_dictionary_v0_1.md#ic-57). **Initial view(s):** planned.

| From | To | Trigger | Guard | Action | Effect |
| --- | --- | --- | --- | --- | --- |
| planned | acquired | session granted | active run and compatible safe session | [AC-30](#ac-30) | Associate workspace with this attempt. |
| planned | blocked | provider/session unavailable | specific dependency or readiness cause | [AC-27](#ac-27), [AC-30](#ac-30) | No native evaluation. |
| planned | aborted_before_start | run revoked | no native call entered | [AC-46](#ac-46) | Record cancelled/superseded cause. |
| acquired | initializing | preparation begins | required seed/intervention plan exists | [AC-31](#ac-31) | Log private initialization work. |
| acquired | running | no initializer required | dispatch readiness established | [AC-32](#ac-32) | Enter isolated calculation sequence. |
| initializing | running | initializer succeeds | original formulation restored or restoration obligations retained for final checks | [AC-31](#ac-31), [AC-32](#ac-32) | Proceed without promoting guesses to constraints. |
| initializing | failed | initializer/restoration fails | failure evidenced | [AC-31](#ac-31), [AC-35](#ac-35) | Withhold original-problem acceptance. |
| running | running | logical run revoked during native call | native termination not yet confirmed | [AC-46](#ac-46), [AC-35](#ac-35) | No publication; hold busy resources; do not pretend native thread stopped. |
| running | candidate_ready | provider returns interpretable data | payload detached and attempt identity matches | [AC-33](#ac-33) | Return candidate independent of live run permission. |
| running | failed | native or mapping failure | known/unknown cause retained | [AC-33](#ac-33), [AC-35](#ac-35) | Preserve prior host result and quarantine uncertain session. |
| running | stopped | native interruption confirmed | safe termination/cleanup established | [AC-35](#ac-35) | Record actual stop, not merely a logical request. |
| candidate_ready | candidate_ready | duplicate completion | same completion identity and payload | [AC-33](#ac-33) | No duplicate candidate mutation or process effect. |

Cancellation is a separate run-authority fact. A cancelled run can still have a candidate_ready numerical attempt; P10 rejects its future publication. A retry creates another attempt, not a backward transition that erases this outcome.

<a id="lc-04"></a>
### LC-04. Provider session health and leasing

**Owner:** P08. **Information:** [IC-55](thermodynamics_semantic_information_dictionary_v0_1.md#ic-55). **Initial view(s):** unconstructed.

| From | To | Trigger | Guard | Action | Effect |
| --- | --- | --- | --- | --- | --- |
| unconstructed | ready | construction succeeds | realization/dependency and isolation policy qualified | [AC-30](#ac-30) | Record exact compatible definition/convention/map revisions. |
| ready | leased | attempt acquires | compatible health and sharing contract | [AC-30](#ac-30) | Exclusive or qualified shared ownership of full sequence. |
| leased | busy | native execution starts | attempt matches lease and run permits fresh dispatch | [AC-32](#ac-32) | Retain lease through readback/normalization. |
| busy | ready | calculation and cleanup complete | native work ended and context healthy/compatible | [AC-33](#ac-33), [AC-35](#ac-35) | Return to reuse only after payload safety established. |
| busy | busy | cancellation or invalidation arrives | native work still active | [AC-35](#ac-35), [AC-46](#ac-46), [AC-48](#ac-48) | Mark no-future-reuse; lease cannot be freed prematurely. |
| busy | quarantined | native failure or uncertain cleanup | active execution ended or isolated; safety not established | [AC-35](#ac-35) | Block new use. |
| leased | ready | abort before native entry | no active native work and reset safe | [AC-35](#ac-35) | Release safely. |
| ready | retired | configuration becomes incompatible | reuse not qualified | [AC-35](#ac-35), [AC-48](#ac-48) | Retire without pretending a new reference/model is already installed. |
| quarantined | resetting | approved recovery begins | actual build-specific reset contract available | [AC-35](#ac-35) | No concurrent lease during recovery. |
| resetting | ready | recovery verified | testable health/compatibility postconditions hold | [AC-35](#ac-35) | Record evidence; otherwise retire. |
| quarantined | retired | recovery unknown or failed | reconstruction/containment policy selected | [AC-35](#ac-35) | Create a different session for future work. |
| busy | retired | isolated worker abandoned/restarted | resources no longer accessible to future host work | [AC-35](#ac-35) | No in-process memory-free guarantee inferred. |

Compatibility and health can change independently. A globally active-model provider may require serialization beyond one handle. In-process fatal failures cannot be contained by a semantic record alone; the selected implementation must meet a demonstrated containment policy.

<a id="lc-05"></a>
### LC-05. Candidate evidence and scope qualification

**Owner:** P09. **Information:** [IC-56](thermodynamics_semantic_information_dictionary_v0_1.md#ic-56), [IC-59](thermodynamics_semantic_information_dictionary_v0_1.md#ic-59), [IC-60](thermodynamics_semantic_information_dictionary_v0_1.md#ic-60). **Initial view(s):** unassessed.

| From | To | Trigger | Guard | Action | Effect |
| --- | --- | --- | --- | --- | --- |
| unassessed | awaiting_evidence | mandatory check lacks data | original evidence demand and budget fixed | [AC-36](#ac-36), [AC-55](#ac-55) | Ask only for bounded evidence under original conventions. |
| unassessed | accepted_for_scope | all required checks pass | candidate set, scope and lineage coherent | [AC-36](#ac-36), [AC-37](#ac-37) | Issue assessment, not current result. |
| unassessed | rejected | mandatory condition fails | failed criterion attributed | [AC-36](#ac-36), [AC-37](#ac-37) | Keep useful candidate diagnostics. |
| awaiting_evidence | accepted_for_scope | additional evidence satisfies checks | all applicable mandatory obligations now pass | [AC-55](#ac-55), [AC-36](#ac-36), [AC-37](#ac-37) | New assessment revision retains earlier unassessed one. |
| awaiting_evidence | rejected | evidence demonstrates failure | original target unchanged | [AC-36](#ac-36), [AC-37](#ac-37) | No post hoc tolerance relaxation. |
| awaiting_evidence | diagnostic_only | evidence unavailable or budget ends | mandatory check remains unassessed | [AC-37](#ac-37) | No acceptance at blocked scope. |
| accepted_for_scope | accepted_for_scope | current model changes | historical assessment still refers to captured original | [AC-56](#ac-56) | Scientific assessment preserved; publication currency handled elsewhere. |

Transitions mean creation of evidence/assessment revisions, not mutation of immutable candidate values. An accepted local result can be diagnostic relative to a parent scope. An optional-property failure is carried as a separate per-output fact.

<a id="lc-06"></a>
### LC-06. Run authority and ancestor revocation

**Owner:** P10. **Information:** [IC-70](thermodynamics_semantic_information_dictionary_v0_1.md#ic-70). **Initial view(s):** issued.

| From | To | Trigger | Guard | Action | Effect |
| --- | --- | --- | --- | --- | --- |
| issued | active | run grant committed | captured context and permitted scope valid | [AC-45](#ac-45) | Permit bounded child attempts/publication proposals. |
| active | cancelled | authorized cancellation | target run or ancestor scope identified | [AC-46](#ac-46) | Forbid future publication; request but do not assume native interruption. |
| active | superseded | newer run replaces scope | replacement policy and generation explicit | [AC-45](#ac-45), [AC-46](#ac-46) | Older results cannot win a later race. |
| active | completed | declared run finishes | required publication resolved or run ends without publication | [AC-49](#ac-49), [AC-46](#ac-46) | No fresh writes under completed run. |
| cancelled | cancelled | duplicate cancellation or late result | same run identity | [AC-46](#ac-46), [AC-49](#ac-49) | Return cancellation/rejection; no reactivation. |
| superseded | superseded | older result completes | effective authority revoked | [AC-49](#ac-49) | Keep historical candidate only. |
| completed | completed | saved result loaded | new live run not issued | [AC-51](#ac-51) | Loading cannot resurrect permission. |

The effective permission of a child is the intersection of its own scope and all relevant ancestor permissions. A later cancellation does not retroactively invalidate a publication committed before revocation; explicit withdrawal or a consequential edit is separately recorded.

<a id="lc-07"></a>
### LC-07. Publication decision and current-result association

**Owner:** P10. **Information:** [IC-66](thermodynamics_semantic_information_dictionary_v0_1.md#ic-66), [IC-67](thermodynamics_semantic_information_dictionary_v0_1.md#ic-67). **Initial view(s):** proposed.

| From | To | Trigger | Guard | Action | Effect |
| --- | --- | --- | --- | --- | --- |
| proposed | committed_current | commit request | qualified scope, coherent group, matching dependencies/run/target generation | [AC-49](#ac-49) | Update all target bindings and decision as one logical operation. |
| proposed | rejected | guard fails | one or more required predicates false/unknown | [AC-49](#ac-49) | No member of the group changes. |
| committed_current | stale_binding | dependent model change commits | impact exact or conservative | [AC-48](#ac-48) | Retain historical result; strict current query cannot use it. |
| committed_current | withdrawn | authorized withdrawal | expected binding and intent verified | [AC-47](#ac-47), [AC-48](#ac-48) | Remove current eligibility without altering original scientific evidence. |
| committed_current | committed_current | duplicate same command delivery | exact prior decision and payload match | [AC-49](#ac-49) | Return receipt only; do not reapply an old binding over newer work. |
| rejected | rejected | duplicate rejected command | same command/payload | [AC-49](#ac-49) | Return same attributable decision; new intent requires new command. |

This combines two named views for readability: decision disposition and binding currency. A stale/withdrawn binding is never an accepted-current state. Rejected candidates can be proposed later only under a new authorized context with appropriate reassessment; original receipts do not change.

<a id="lc-08"></a>
### LC-08. Archive inspection and reconstruction

**Owner:** P10. **Information:** [IC-68](thermodynamics_semantic_information_dictionary_v0_1.md#ic-68), [IC-69](thermodynamics_semantic_information_dictionary_v0_1.md#ic-69). **Initial view(s):** capturing.

| From | To | Trigger | Guard | Action | Effect |
| --- | --- | --- | --- | --- | --- |
| capturing | archived | coherent export verified | all required semantic snapshots from one captured context | [AC-50](#ac-50) | Publish archive manifest; no hidden evaluation or fitting. |
| archived | inspection_loaded | open archive | format/identity can be inspected | [AC-51](#ac-51) | No restored currentness or old run permission. |
| inspection_loaded | reconstructed | domain rebuilding succeeds | identities/coordinates/conventions/equations consistent | [AC-51](#ac-51) | Expose exact semantics and any approved migrations. |
| reconstructed | dependency_blocked | required engine/data absent | dependency assessed | [AC-27](#ac-27), [AC-51](#ac-51) | Inspection remains available. |
| reconstructed | ready_for_attempt | readiness checks succeed | exact or explicitly changed realization available | [AC-08](#ac-08), [AC-09](#ac-09), [AC-51](#ac-51) | Fresh run/session required for execution. |
| dependency_blocked | ready_for_attempt | dependencies supplied | new assessment confirms compatibility | [AC-27](#ac-27), [AC-09](#ac-09), [AC-51](#ac-51) | No silent substitute. |
| ready_for_attempt | compared | requested rerun completes | new run and qualified comparison evidence | [AC-53](#ac-53) | Distinguish exact-context reproduction from changed-model comparison. |

Failed capture exposes no complete mixed-revision archive. A partial archive can be explicitly exported as such. Saved numeric state is not a portable command log: restoration must not replay reservoir transfers, cancel/restart native calls or restore previous publication permissions.

## 7. Handoff facts and information delivery

A fact records what an action has already established; it is not a grant of additional physical or publication authority. Every fact carries references to its producing action, exact subject/revision/run where applicable, outcome and evidence. These can be ordinary in-process return values or persistent records; no event-driven architecture is imposed. Reordered or duplicate delivery must not weaken the receiver’s own current-context checks.

| Fact | Producer | Consumer | Information | Meaning |
| --- | --- | --- | --- | --- |
| EV-01 DefinitionProposalPrepared | [AC-01](#ac-01), [AC-02](#ac-02), [AC-03](#ac-03), [AC-04](#ac-04), [AC-05](#ac-05), [AC-06](#ac-06), [AC-07](#ac-07), [AC-10](#ac-10) | [AC-08](#ac-08), [AC-47](#ac-47) | [IC-03](thermodynamics_semantic_information_dictionary_v0_1.md#ic-03), [IC-14](thermodynamics_semantic_information_dictionary_v0_1.md#ic-14), [IC-17](thermodynamics_semantic_information_dictionary_v0_1.md#ic-17), [IC-20](thermodynamics_semantic_information_dictionary_v0_1.md#ic-20), [IC-45](thermodynamics_semantic_information_dictionary_v0_1.md#ic-45), [IC-65](thermodynamics_semantic_information_dictionary_v0_1.md#ic-65) | Domain proposal exists; no live-model mutation implied. |
| EV-02 CompatibilityAssessed | [AC-08](#ac-08) | [AC-09](#ac-09), [AC-47](#ac-47) | [IC-21](thermodynamics_semantic_information_dictionary_v0_1.md#ic-21), [IC-23](thermodynamics_semantic_information_dictionary_v0_1.md#ic-23) | Named conventions/method combination qualified, incompatible or unassessed. |
| EV-03 ChangeCommitted | [AC-48](#ac-48) | [AC-09](#ac-09), [AC-19](#ac-19), [AC-29](#ac-29), [AC-30](#ac-30), [AC-49](#ac-49), [AC-56](#ac-56) | [IC-64](thermodynamics_semantic_information_dictionary_v0_1.md#ic-64), [IC-65](thermodynamics_semantic_information_dictionary_v0_1.md#ic-65), [IC-67](thermodynamics_semantic_information_dictionary_v0_1.md#ic-67) | New context and impact already authoritative; notifications are not the sole stale-result guard. |
| EV-04 BindingResolved | [AC-19](#ac-19) | [AC-20](#ac-20), [AC-21](#ac-21) | [IC-48](thermodynamics_semantic_information_dictionary_v0_1.md#ic-48), [IC-49](thermodynamics_semantic_information_dictionary_v0_1.md#ic-49) | Effective location assignment and overlay trace, not current native stream. |
| EV-05 ProblemCaptured | [AC-21](#ac-21), [AC-22](#ac-22), [AC-23](#ac-23), [AC-24](#ac-24), [AC-25](#ac-25), [AC-26](#ac-26), [AC-54](#ac-54) | [AC-09](#ac-09), [AC-29](#ac-29) | [IC-35](thermodynamics_semantic_information_dictionary_v0_1.md#ic-35), [IC-36](thermodynamics_semantic_information_dictionary_v0_1.md#ic-36), [IC-37](thermodynamics_semantic_information_dictionary_v0_1.md#ic-37), [IC-41](thermodynamics_semantic_information_dictionary_v0_1.md#ic-41) | Original physical targets and authority fixed. |
| EV-06 ReadinessAssessed | [AC-09](#ac-09) | [AC-29](#ac-29), [AC-31](#ac-31) | [IC-24](thermodynamics_semantic_information_dictionary_v0_1.md#ic-24) | One request and context; readiness is not execution or validation. |
| EV-07 RunAuthorized | [AC-45](#ac-45) | [AC-29](#ac-29), [AC-44](#ac-44) | [IC-64](thermodynamics_semantic_information_dictionary_v0_1.md#ic-64), [IC-70](thermodynamics_semantic_information_dictionary_v0_1.md#ic-70) | Bounded active run permission, distinct from session lease. |
| EV-08 RunRevoked | [AC-46](#ac-46) | [AC-32](#ac-32), [AC-35](#ac-35), [AC-44](#ac-44), [AC-49](#ac-49) | [IC-70](thermodynamics_semantic_information_dictionary_v0_1.md#ic-70) | Cancellation/supersession already blocks future publication; native stop may be pending. |
| EV-09 InitializationAssessed | [AC-31](#ac-31) | [AC-09](#ac-09), [AC-32](#ac-32), [AC-36](#ac-36) | [IC-54](thermodynamics_semantic_information_dictionary_v0_1.md#ic-54), [IC-57](thermodynamics_semantic_information_dictionary_v0_1.md#ic-57) | Seed eligibility and intervention/restoration evidence, not a new user constraint. |
| EV-10 CandidateProduced | [AC-32](#ac-32), [AC-33](#ac-33) | [AC-16](#ac-16), [AC-36](#ac-36), [AC-44](#ac-44) | [IC-56](thermodynamics_semantic_information_dictionary_v0_1.md#ic-56), [IC-63](thermodynamics_semantic_information_dictionary_v0_1.md#ic-63), [IC-71](thermodynamics_semantic_information_dictionary_v0_1.md#ic-71) | Detached values and actual used assumptions; no acceptance guarantee. |
| EV-11 SessionQuarantined | [AC-35](#ac-35) | [AC-30](#ac-30) | [IC-55](thermodynamics_semantic_information_dictionary_v0_1.md#ic-55), [IC-63](thermodynamics_semantic_information_dictionary_v0_1.md#ic-63) | Unsafe/unknown context cannot be reused until qualified recovery. |
| EV-12 EvidenceNeeded | [AC-36](#ac-36), [AC-37](#ac-37) | [AC-55](#ac-55) | [IC-41](thermodynamics_semantic_information_dictionary_v0_1.md#ic-41), [IC-59](thermodynamics_semantic_information_dictionary_v0_1.md#ic-59), [IC-60](thermodynamics_semantic_information_dictionary_v0_1.md#ic-60) | Bounded missing evidence under original candidate meaning; no permission to fix the target. |
| EV-13 EvidenceProduced | [AC-55](#ac-55) | [AC-36](#ac-36) | [IC-35](thermodynamics_semantic_information_dictionary_v0_1.md#ic-35), [IC-56](thermodynamics_semantic_information_dictionary_v0_1.md#ic-56) | A verification result with explicit dependency/independence relation. |
| EV-14 ScopeQualified | [AC-37](#ac-37) | [AC-44](#ac-44), [AC-49](#ac-49), [AC-56](#ac-56) | [IC-60](thermodynamics_semantic_information_dictionary_v0_1.md#ic-60), [IC-62](thermodynamics_semantic_information_dictionary_v0_1.md#ic-62) | Accepted at named scope, not automatically current or parent-converged. |
| EV-15 PublicationDecided | [AC-49](#ac-49) | [AC-44](#ac-44), [AC-56](#ac-56) | [IC-66](thermodynamics_semantic_information_dictionary_v0_1.md#ic-66), [IC-67](thermodynamics_semantic_information_dictionary_v0_1.md#ic-67) | Committed or rejected group decision; duplicate delivery does not reapply effects. |
| EV-16 ArchiveExported | [AC-50](#ac-50) | [AC-51](#ac-51) | [IC-68](thermodynamics_semantic_information_dictionary_v0_1.md#ic-68) | Coherent captured semantics plus dependency/reconstruction limits. |
| EV-17 ReconstructionAssessed | [AC-51](#ac-51), [AC-52](#ac-52), [AC-53](#ac-53) | [AC-09](#ac-09), [AC-38](#ac-38), [AC-56](#ac-56) | [IC-69](thermodynamics_semantic_information_dictionary_v0_1.md#ic-69) | Inspection/readiness/reproduction distinctions retained. |
| EV-18 CoverageAssessed | [AC-38](#ac-38) | [AC-09](#ac-09), [AC-56](#ac-56) | [IC-61](thermodynamics_semantic_information_dictionary_v0_1.md#ic-61) | Bounded evidence and gaps; no promotion from action-map completeness. |

A delayed ChangeCommitted or RunRevoked notification is not permission to publish: AC-49 reads authoritative commit-time context. Similarly, a received CandidateProduced fact cannot be used as a ScopeQualified fact. Event timestamps are not physical process time or a substitute for revision/run identity.

## 8. Complete workflows and branch behavior

<a id="wf7-01"></a>
### WF7-01. Prepare an ordinary package and adopt estimated data

**Scope:** SC-01, SC-08, SC-09. **Action route:** [AC-01](#ac-01), [AC-02](#ac-02), [AC-04](#ac-04), [AC-05](#ac-05), [AC-06](#ac-06), [AC-07](#ac-07), [AC-08](#ac-08), [AC-09](#ac-09), [AC-19](#ac-19), [AC-47](#ac-47), [AC-48](#ac-48).

1. AC-01/02 resolve identities and create a semantically qualified slate.
2. AC-04 resolves explicit method-qualified evidence; if missing, AC-05/06 prepares estimates/trials under trial-only authority.
3. AC-07/08 creates a coherent package proposal and convention assessment; P07 gathers provider evidence for AC-09.
4. AC-19 identifies intended locations and overlays.
5. AC-47/48 adopts the material/data/package/binding set against one expected prior context.

**Successful boundary:** The approved definition exists with attributable preparation and exact readiness limits; this is not a solved stream.

**Failure/partial branches:**

- Missing coefficients: preserve a draft or approve a named estimate, never bind-and-auto-fit.
- Failed trial: retain trial evidence without changing production data.
- Intervening edit: reject the stale coordinated proposal or explicitly rebase and revalidate it.

**Publication scope:** Definition revision only; no current thermodynamic result published.

**Acceptance witnesses to execute:** Replacing a context must not change the chosen coefficient set. Save and replay of the adoption command must not re-run estimation.

**Status:** Complete specified action route, not an executed provider/unit fixture or independent R/E/V review.

<a id="wf7-02"></a>
### WF7-02. Edit a feed and recalculate from new input intent

**Scope:** SC-01. **Action route:** [AC-13](#ac-13), [AC-14](#ac-14), [AC-15](#ac-15), [AC-47](#ac-47), [AC-48](#ac-48), [AC-45](#ac-45), [AC-19](#ac-19), [AC-20](#ac-20), [AC-21](#ac-21), [AC-23](#ac-23), [AC-09](#ac-09), [AC-27](#ac-27), [AC-28](#ac-28), [AC-29](#ac-29), [AC-30](#ac-30), [AC-31](#ac-31), [AC-32](#ac-32), [AC-33](#ac-33), [AC-36](#ac-36), [AC-37](#ac-37), [AC-49](#ac-49).

1. Capture the original feed with AC-13.
2. Choose complete replacement AC-14 or patch/rescale AC-15 with its preservation rule; AC-47/48 commits the exact input change.
3. AC-45 starts a fresh captured run, AC-19/20 resolves binding/demand and AC-21/23 freezes the requested state.
4. Qualify and execute via AC-09 and AC-27..33.
5. AC-36/37 checks the original state specification; AC-49 publishes only the allowed current location/scope.

**Successful boundary:** New input and result are individually attributable; unmentioned components follow the chosen edit policy, not construction history.

**Failure/partial branches:**

- Invalid replacement: no live edit.
- Zero-flow composition: retain specified fractions but do not divide zero rates.
- Solve failure after a valid edit: the edit remains current while old results are stale and available as history; do not silently roll the author back.

**Publication scope:** Local result plus explicitly invalidated dependent unit/flowsheet scopes.

**Acceptance witnesses to execute:** Fresh and loaded cases respond identically to the same explicit edit. Failure cannot relabel the previous result as valid for the new input.

**Status:** Complete specified action route, not an executed provider/unit fixture or independent R/E/V review.

<a id="wf7-03"></a>
### WF7-03. Resolve a heater, pump, compressor or valve outlet

**Scope:** SC-02, SC-03, SC-04, SC-05, SC-06, SC-12, SC-13, SC-14. **Action route:** [AC-19](#ac-19), [AC-20](#ac-20), [AC-13](#ac-13), [AC-45](#ac-45), [AC-21](#ac-21), [AC-23](#ac-23), [AC-09](#ac-09), [AC-29](#ac-29), [AC-30](#ac-30), [AC-31](#ac-31), [AC-32](#ac-32), [AC-33](#ac-33), [AC-36](#ac-36), [AC-55](#ac-55), [AC-37](#ac-37), [AC-49](#ac-49).

1. P07 obtains the selected equipment constraints and actual property demands through AC-19/20.
2. AC-13 captures feed, targets and guesses; AC-45/21 freezes context and allowed changes.
3. AC-23 instantiates TP/PH/PS or saturation semantics; the compressor PS reference is a separate local problem from its actual outlet relation.
4. AC-09/29..33 produces candidates with actual searched phases and native status.
5. AC-36 obtains missing enthalpy/entropy evidence through AC-55 without changing allocation or original target.
6. P07 checks the equipment relation, AC-37 qualifies the requested scope and AC-49 publishes constraints, allocation or completed state as actually achieved.

**Successful boundary:** Local target and equipment conditions pass independently; required downstream state completion is visible.

**Failure/partial branches:**

- No entropy/caloric/initializer: precise blocked prerequisite.
- Saturated pure TP underdetermined: endpoint or missing independent constraint, no arbitrary quality.
- h=20 versus target50: failed check despite provider success.
- Optional rating/report property absent: cannot claim rating if it is actually required by the selected mode.

**Publication scope:** Named outlet publication kind and, only with unit checks, coherent unit completion.

**Acceptance witnesses to execute:** Forward H/S then inverse request agrees within predeclared fixture tolerance. No new unit completion follows merely from a successful reference-state calculation.

**Status:** Complete specified action route, not an executed provider/unit fixture or independent R/E/V review.

<a id="wf7-04"></a>
### WF7-04. Mix, flash and route a multiphase separator

**Scope:** SC-01, SC-05, SC-10, SC-11, SC-23. **Action route:** [AC-39](#ac-39), [AC-21](#ac-21), [AC-23](#ac-23), [AC-09](#ac-09), [AC-29](#ac-29), [AC-30](#ac-30), [AC-31](#ac-31), [AC-32](#ac-32), [AC-33](#ac-33), [AC-16](#ac-16), [AC-36](#ac-36), [AC-37](#ac-37), [AC-40](#ac-40), [AC-22](#ac-22), [AC-49](#ac-49).

1. AC-39 constructs the mixed material and energy target from compatible feeds.
2. AC-21/23 defines the shared phase problem and inert/reactive-solid policy.
3. The numerical spine produces detached candidate evidence; AC-16 identifies physical, absent and incipient portions and any required correspondence.
4. AC-36/37 qualifies the internal state.
5. AC-40 applies named product routing and solids carryover; any necessary outlet property completion uses AC-22.
6. Recheck material/energy across every product, qualify the unit group, then AC-49 adopts the complete group.

**Successful boundary:** The products reconcile the internal material under explicit routing rules; physical phase identity is not a port index.

**Failure/partial branches:**

- A required third phase omitted: reject original-problem coverage.
- Ambiguous phase matching: retain ambiguous correspondence and block only dependent routing/continuation, not all valid internal properties.
- Empty phase: no invented positive outlet.
- One outlet fails: do not replace only the other half as a completed separator.

**Publication scope:** Internal local assessment may exist before all-or-none accepted separator product group.

**Acceptance witnesses to execute:** Reverse provider liquid order with unchanged physical material. Vary carryover without changing internal equilibrium identity.

**Status:** Complete specified action route, not an executed provider/unit fixture or independent R/E/V review.

<a id="wf7-05"></a>
### WF7-05. Couple disjoint material packages through heat

**Scope:** SC-02, SC-12, SC-28. **Action route:** [AC-19](#ac-19), [AC-20](#ac-20), [AC-21](#ac-21), [AC-22](#ac-22), [AC-23](#ac-23), [AC-41](#ac-41), [AC-29](#ac-29), [AC-30](#ac-30), [AC-31](#ac-31), [AC-32](#ac-32), [AC-33](#ac-33), [AC-36](#ac-36), [AC-37](#ac-37), [AC-44](#ac-44), [AC-49](#ac-49).

1. Bind each side independently and state unit-owned heat/work/loss rules.
2. AC-41 creates separate side problems and optional bounding/rating requests.
3. P08 executes eligible side operations under a common captured unit context but separate compatible sessions.
4. AC-36 qualifies local targets; the host unit iteration applies its heat-transfer relationship.
5. AC-44 records shared residuals; AC-37 checks the coupled scope.
6. AC-49 commits the coherent side result set.

**Successful boundary:** Within-side caloric differences and shared heat balance agree without material crossing the wall.

**Failure/partial branches:**

- Optional hypothetical bound fails: report absence if not essential to this mode.
- Required side caloric property fails: unit acceptance blocked.
- One side uses incompatible revision/run: no group publication.
- Repeated iterations do not apply the same duty as repeated physical heating.

**Publication scope:** Coherent exchanger group, with diagnostics allowed separately.

**Acceptance witnesses to execute:** Shift one side reference consistently without changing exchanged heat. Reject a cross-wall constituent translation inserted only to match absolute enthalpy.

**Status:** Complete specified action route, not an executed provider/unit fixture or independent R/E/V review.

<a id="wf7-06"></a>
### WF7-06. Execute strict conversion or finite-rate reaction with thermal closure

**Scope:** SC-17, SC-19, SC-25. **Action route:** [AC-10](#ac-10), [AC-11](#ac-11), [AC-08](#ac-08), [AC-20](#ac-20), [AC-21](#ac-21), [AC-24](#ac-24), [AC-22](#ac-22), [AC-23](#ac-23), [AC-29](#ac-29), [AC-30](#ac-30), [AC-31](#ac-31), [AC-32](#ac-32), [AC-33](#ac-33), [AC-36](#ac-36), [AC-37](#ac-37), [AC-49](#ac-49).

1. AC-10 defines stoichiometry, kinetics/participation and property inputs; AC-08 reconciles energy conventions.
2. For strict conversion AC-11 checks the complete requested extent group before species candidate generation.
3. For kinetics the host reactor supplies residence-time/geometry context and requests the qualified local rates, not a full equilibrium substitute.
4. AC-21/24 specifies phase/thermal/chemistry coupling and authorized transformations.
5. P08 evaluates supported operations; P07 assembles the reactor energy and process conditions.
6. AC-36/37 checks conservation, achieved conversion/rate formulation and exactly-once reaction energy before publication.

**Successful boundary:** The actual requested reaction assumption and thermal/process conditions are simultaneously satisfied.

**Failure/partial branches:**

- Infeasible strict extent: reject, optionally AC-54 for separately authorized limiting optimization.
- Frozen species consumed: authority failure.
- Missing energy convention: composition may remain diagnostic, but no energy-balanced reactor claim.
- Retried calculation starts from captured material, not already transformed candidate.

**Publication scope:** Reactive unit candidate set, with local chemical/property evidence retained independently.

**Acceptance witnesses to execute:** A→B extent0.4 succeeds and1.2 from one mole fails. Equivalent energy conventions yield the same authored duty without double addition.

**Status:** Complete specified action route, not an executed provider/unit fixture or independent R/E/V review.

<a id="wf7-07"></a>
### WF7-07. Resolve aqueous chemistry, open exchanges and precipitation

**Scope:** SC-18, SC-20, SC-21, SC-22, SC-24, SC-25. **Action route:** [AC-02](#ac-02), [AC-10](#ac-10), [AC-12](#ac-12), [AC-08](#ac-08), [AC-20](#ac-20), [AC-21](#ac-21), [AC-24](#ac-24), [AC-09](#ac-09), [AC-29](#ac-29), [AC-30](#ac-30), [AC-31](#ac-31), [AC-32](#ac-32), [AC-33](#ac-33), [AC-16](#ac-16), [AC-36](#ac-36), [AC-37](#ac-37), [AC-40](#ac-40), [AC-49](#ac-49).

1. Define one material account with apparent/true description and justified conserved bases.
2. AC-10/12 states species/phase eligibility and mapping; P07 authorizes each reservoir or titrant in the original problem.
3. AC-08 qualifies chemistry/caloric standards and required transport; AC-24 creates the combined problem.
4. P08 returns species/phases, actual exchange amounts and used restrictions.
5. AC-16 separates precipitated solids from inert carried material; AC-36 checks final chemical, phase, thermal and exchange-inclusive balances.
6. P07 handles equipment transfer/routing and P09 qualifies the full scope before group publication.

**Successful boundary:** A coupled result with one physical account and every required external exchange reported once.

**Failure/partial branches:**

- Closed problem requests hidden titrant: reject.
- Speciation-only engine cannot support missing energy demand.
- A later flash invalidates chemistry: iterate under declared strategy or fail/qualify approximation, never claim full equilibrium.
- Saturation index alone is not crystal-size/kinetic-yield prediction.

**Publication scope:** Local chemical scope versus full absorber/crystallizer unit scope explicitly separated.

**Acceptance witnesses to execute:** Apparent and ionic descriptions are nonadditive. Omission or duplicate application of a titrant fails combined conservation.

**Status:** Complete specified action route, not an executed provider/unit fixture or independent R/E/V review.

<a id="wf7-08"></a>
### WF7-08. Solve stages or a nonequilibrium contacting formulation

**Scope:** SC-07, SC-08, SC-09, SC-10, SC-20, SC-26. **Action route:** [AC-19](#ac-19), [AC-20](#ac-20), [AC-13](#ac-13), [AC-21](#ac-21), [AC-22](#ac-22), [AC-24](#ac-24), [AC-25](#ac-25), [AC-26](#ac-26), [AC-09](#ac-09), [AC-29](#ac-29), [AC-30](#ac-30), [AC-31](#ac-31), [AC-32](#ac-32), [AC-33](#ac-33), [AC-43](#ac-43), [AC-44](#ac-44), [AC-36](#ac-36), [AC-37](#ac-37), [AC-49](#ac-49).

1. P07 establishes stage/segment/bulk/interface locations, host equations and required thermodynamic contributions.
2. AC-13 retains partial states; AC-21/26 binds contributions and original constraint ownership.
3. AC-31 records temporary fixing/relaxation; P08 evaluates only actually supported values/residuals/equations/derivatives.
4. AC-43 preserves distinct bulk/interface descriptions and lets the host coordinate material/energy transfer and iterations.
5. Restore original constraints; AC-44 assembles consistent candidate views and residuals.
6. P09 qualifies coupled conditions and P10 publishes only the requested completed group.

**Successful boundary:** A mathematically coherent unit formulation with explicit local authority and actual contribution capability.

**Failure/partial branches:**

- Missing derivative: use an explicitly qualified numerical route or reject.
- Temporary fixing left active: fail original-problem acceptance.
- Bulk property call re-equilibrates both phases: authority failure.
- Efficiency-corrected equilibrium stages are a different declared model, not a SC-26 pass.

**Publication scope:** Local property results, stage/unit candidates and accepted whole unit have distinct scopes.

**Acceptance witnesses to execute:** Nonzero transfer can coexist with distinct bulk compositions/temperatures. A numerically zero Jacobian entry cannot remove a true structural dependency.

**Status:** Complete specified action route, not an executed provider/unit fixture or independent R/E/V review.

<a id="wf7-09"></a>
### WF7-09. Close recycles without prematurely publishing trial outputs

**Scope:** SC-01, SC-05, SC-06, SC-07, SC-13, SC-17, SC-18. **Action route:** [AC-45](#ac-45), [AC-20](#ac-20), [AC-21](#ac-21), [AC-29](#ac-29), [AC-32](#ac-32), [AC-33](#ac-33), [AC-36](#ac-36), [AC-37](#ac-37), [AC-44](#ac-44), [AC-46](#ac-46), [AC-49](#ac-49), [AC-56](#ac-56).

1. AC-45 captures base definitions and run scope; P07 creates a run-local completion view.
2. Each local trial problem freezes its actual input values, even within the same base revision.
3. AC-32/33 returns candidates; AC-36/37 qualifies local operations.
4. AC-44 assembles the next coherent iteration and evaluates parent residuals.
5. Continue within bounds or qualify the full result set.
6. AC-49 publishes only under active matching run authority and complete group criteria.

**Successful boundary:** Local values support numerical iteration without being labeled an accepted plant solution.

**Failure/partial branches:**

- Recycle exhaustion: leave parent unaccepted and previous accepted plant state intact.
- New input revision or superseding run: revoke affected future publication.
- Mixed stale iterates with good individual residuals: reject group coherence.

**Publication scope:** Run-local diagnostic view versus current unit/flowsheet bindings.

**Acceptance witnesses to execute:** Local flash success cannot override a failed recycle residual. Cancelled-parent child completion cannot publish.

**Status:** Complete specified action route, not an executed provider/unit fixture or independent R/E/V review.

<a id="wf7-10"></a>
### WF7-10. Cross a material/package boundary with explicit information loss

**Scope:** SC-16, SC-21, SC-29, SC-30. **Action route:** [AC-02](#ac-02), [AC-03](#ac-03), [AC-08](#ac-08), [AC-12](#ac-12), [AC-19](#ac-19), [AC-20](#ac-20), [AC-42](#ac-42), [AC-21](#ac-21), [AC-23](#ac-23), [AC-24](#ac-24), [AC-29](#ac-29), [AC-32](#ac-32), [AC-33](#ac-33), [AC-36](#ac-36), [AC-37](#ac-37), [AC-49](#ac-49).

1. P07 captures source/target bindings and selects preserved quantities.
2. P01/P06 qualifies the forward representation relation and reverse limitations.
3. P03 supplies known reference transformations; unknown required transformations remain blockers.
4. AC-42 maps material and forms the target physical problem without adding another inventory.
5. Solve as required; report reference adjustment, model discrepancy, mapping residual and genuine physical exchanges separately.
6. P09 checks boundary and process scope; P10 adopts only a compatible target result.

**Successful boundary:** The target satisfies its chosen constraints and honestly states lost information and model disagreement.

**Failure/partial branches:**

- No justified inverse: require new information or reject reconstruction.
- Preserving incompatible corrected H and TP: reject, never hide a heat duty.
- Known source result plus missing target engine: inspection/partial boundary only.

**Publication scope:** A named translation case and target publication scope, not an implicit global package change.

**Acceptance witnesses to execute:** Synthetic hA/hB example exposes a10kJ/kg model discrepancy at fixed350K. An irreversible lump cannot return a unique original composition.

**Status:** Complete specified action route, not an executed provider/unit fixture or independent R/E/V review.

<a id="wf7-11"></a>
### WF7-11. Characterize petroleum or use a limited empirical material

**Scope:** SC-15, SC-16, SC-23, SC-31. **Action route:** [AC-02](#ac-02), [AC-03](#ac-03), [AC-04](#ac-04), [AC-06](#ac-06), [AC-07](#ac-07), [AC-08](#ac-08), [AC-09](#ac-09), [AC-47](#ac-47), [AC-48](#ac-48), [AC-13](#ac-13), [AC-20](#ac-20), [AC-21](#ac-21), [AC-22](#ac-22), [AC-23](#ac-23), [AC-39](#ac-39), [AC-40](#ac-40), [AC-42](#ac-42), [AC-36](#ac-36), [AC-37](#ac-37), [AC-49](#ac-49).

1. P02 retains assay or empirical evidence and prepares cut/correlation proposals.
2. P01 assesses constituent/representation meaning without fictitious registry or molecular data.
3. P03 qualifies PVT, caloric and required inverse/reporting operations separately.
4. P10 adopts coherent proposed definitions and P07 creates actual process requests.
5. Evaluate supported mass/energy operations and qualified separations/translations.
6. Check only justified conserved quantities and report remaining delivery gaps.

**Successful boundary:** A material is useful for the operations its data justify; broad-scope gaps stay explicit until real fixtures are run.

**Failure/partial branches:**

- PVT-only pseudos cannot pass thermal use.
- Mass-only h(T) remains usable while fugacity/molar conversion is unsupported.
- Black-oil bulk description cannot silently turn into invented detailed molecules.

**Publication scope:** Definition adoption and supported local/unit results, not package-wide numerical coverage.

**Acceptance witnesses to execute:** The authored2kg/s,2kJ/kg/K,25K heating example gives100kW without molecular weight. Saving retains both assay and generated cut meaning.

**Status:** Complete specified action route, not an executed provider/unit fixture or independent R/E/V review.

<a id="wf7-12"></a>
### WF7-12. Evaluate derivative responses and independent coordinate transforms

**Scope:** SC-04, SC-07, SC-26. **Action route:** [AC-21](#ac-21), [AC-25](#ac-25), [AC-26](#ac-26), [AC-28](#ac-28), [AC-29](#ac-29), [AC-30](#ac-30), [AC-31](#ac-31), [AC-32](#ac-32), [AC-33](#ac-33), [AC-36](#ac-36), [AC-55](#ac-55), [AC-37](#ac-37).

1. AC-25 fixes observable, chart, held-fixed quantities, response and order.
2. P08 records an eligible derivative realization and controlled perturbation policy where needed.
3. Each perturbation preserves the experiment constraints and uses private states; remap all derivative axes through AC-28.
4. Normalize the tensor and branch/conditioning evidence.
5. P09 compares like-for-like evidence and qualifies only the declared derivative meaning.

**Successful boundary:** A qualified derivative or precise unsupported/unassessed result with source chart and response intact.

**Failure/partial branches:**

- Phase transition crossed: apply declared one-sided/path policy or withhold smoothness claim.
- No higher-order support: do not infer it.
- Perturbation mutates the base feed: reject authority violation.

**Publication scope:** Derivative evidence tied to the base state/problem; no automatic physical-state publication.

**Acceptance witnesses to execute:** For the authored composition example the chart derivatives are-3,-2. Frozen allocation and re-equilibrated response remain different observables.

**Status:** Complete specified action route, not an executed provider/unit fixture or independent R/E/V review.

<a id="wf7-13"></a>
### WF7-13. Recover from failure, retry or request an alternate model

**Scope:** SC-05, SC-06, SC-11, SC-24, SC-34. **Action route:** [AC-29](#ac-29), [AC-30](#ac-30), [AC-31](#ac-31), [AC-32](#ac-32), [AC-33](#ac-33), [AC-35](#ac-35), [AC-34](#ac-34), [AC-54](#ac-54), [AC-09](#ac-09), [AC-36](#ac-36), [AC-37](#ac-37), [AC-46](#ac-46), [AC-49](#ac-49).

1. An attempt returns a numerical/provider/check failure with original targets preserved.
2. AC-35 quarantines uncertain state before any reuse.
3. AC-34 creates a bounded same-problem retry with compatible seed/session.
4. If altered physics is needed, AC-54 obtains specific authorization and creates a new linked physical problem, then requalifies it.
5. P09 retains both original outcome and alternate assessment.
6. P10 honors current context and run permission for any eventual publication.

**Successful boundary:** Useful recovery without silently changing the requested science or corrupting accepted results.

**Failure/partial branches:**

- Native call never stops: do not recycle its memory/context; use qualified containment and revoked publication.
- Retry budget exhausted: preserve original failure.
- Unauthorized ideal fallback: no accepted original result.
- Response lost: reconcile the identified attempt/decision, not a blind new mutation.

**Publication scope:** Each problem and attempt keeps its own outcome and publication permission.

**Acceptance witnesses to execute:** Failed workspace writes cannot affect accepted arrays. Retry/alternative ancestry must be acyclic and distinct.

**Status:** Complete specified action route, not an executed provider/unit fixture or independent R/E/V review.

<a id="wf7-14"></a>
### WF7-14. Edit during execution, cancel and resolve late or duplicate completions

**Scope:** SC-01, SC-06, SC-28. **Action route:** [AC-45](#ac-45), [AC-29](#ac-29), [AC-32](#ac-32), [AC-47](#ac-47), [AC-48](#ac-48), [AC-46](#ac-46), [AC-33](#ac-33), [AC-36](#ac-36), [AC-37](#ac-37), [AC-49](#ac-49), [AC-56](#ac-56).

1. A run starts under context A and executes privately.
2. A successful AC-48 edit establishes B and invalidates affected current bindings; or AC-46 revokes a same-revision run.
3. A late candidate can still be normalized and historically assessed against A.
4. AC-49 checks current dependencies, live run/ancestors, expected target generation and group scope at one commit point.
5. Rejected late work remains attributable history, while a duplicate committed command returns its stored decision without rewriting a newer current result.

**Successful boundary:** No stale, cancelled, superseded or conflicting result overwrites current state.

**Failure/partial branches:**

- Edit wins before publish: A publication rejected.
- Publish wins before edit: A was valid at publication and becomes stale when B commits.
- Cancel after publication: no retroactive numerical invalidity; future writes forbidden.
- Lost publish response: same command replay returns decision, never reapplies old binding over newer work.

**Publication scope:** Authoritative current bindings, with separate immutable scientific assessments.

**Acceptance witnesses to execute:** Exercise both event orderings. Two-member group failure leaves both previous bindings unchanged.

**Status:** Complete specified action route, not an executed provider/unit fixture or independent R/E/V review.

<a id="wf7-15"></a>
### WF7-15. Save, inspect, migrate, undo and reproduce a case

**Scope:** SC-15, SC-29, SC-30, SC-31. **Action route:** [AC-50](#ac-50), [AC-51](#ac-51), [AC-27](#ac-27), [AC-08](#ac-08), [AC-09](#ac-09), [AC-52](#ac-52), [AC-47](#ac-47), [AC-48](#ac-48), [AC-45](#ac-45), [AC-53](#ac-53), [AC-29](#ac-29), [AC-32](#ac-32), [AC-36](#ac-36), [AC-37](#ac-37), [AC-38](#ac-38), [AC-49](#ac-49).

1. AC-50 exports one coherent captured semantic context and result lineage, without provider preparation side effects.
2. AC-51 rebuilds meaning before quantities and resolves missing dependencies separately.
3. Explicit migrations/undo AC-52 go through domain validation and guarded change adoption.
4. AC-53 starts a fresh run to reproduce or compare; P09 assesses bounded evidence.
5. Restored results get current authority only through an explicit compatible publication decision.

**Successful boundary:** Inspectable and attributable saved model, with independent reconstruction/readiness/result-authority states.

**Failure/partial branches:**

- Concurrent save edit: capture fixed revision or restart/fail, never mixed data.
- Missing engine: inspectable but affected numerics blocked.
- Changed data/build: changed-model comparison, not unqualified reproduction.
- Saved cancelled run: never revived by loading.

**Publication scope:** Archive and restored-model scope; no replay of physical effects or prior run grants.

**Acceptance witnesses to execute:** Source/derived assay content survive roundtrip. Snapshot values cannot confer authority on different equations.

**Status:** Complete specified action route, not an executed provider/unit fixture or independent R/E/V review.

<a id="wf7-16"></a>
### WF7-16. P3 surface/selective transfer and distribution actions

**Scope:** SC-27, SC-32. **Action route:** [AC-02](#ac-02), [AC-03](#ac-03), [AC-13](#ac-13), [AC-18](#ac-18), [AC-20](#ac-20), [AC-21](#ac-21), [AC-22](#ac-22), [AC-36](#ac-36), [AC-37](#ac-37), [AC-47](#ac-47), [AC-48](#ac-48), [AC-49](#ac-49).

1. P01 defines site/loading or distribution support and weighting with actual material meaning.
2. P04 captures bulk/surface or distribution-bearing accounts without a fictitious liquid flow.
3. P07 supplies prescribed transfer/mixing intent and units; numerical transport/adsorption may remain unsupported.
4. AC-18 derives donor/receiver or weighted/reduced attribute candidates and one physical exchange where applicable.
5. Check conservation, nonnegative availability, receiving limits and recorded loss.
6. Adopt only under the chosen change/result scope; P3 numeric capability is not created by arithmetic.

**Successful boundary:** A concrete action and information route with no double application or untyped extension semantics.

**Failure/partial branches:**

- Missing loading denominator or support frame: incomplete.
- Incompatible weighting: map explicitly or reject.
- Reverse from mean: nonunique.
- Duplicate transfer intent: return original decision, do not apply delta again.

**Publication scope:** Representability walkthrough and synthetic accounting only until separately qualified numerics.

**Acceptance witnesses to execute:** Mass-weighted histograms give0.35/0.65 for the authored1kg/3kg example. ||0.1mol surface transfer is balanced once, not once per retry.

**Status:** Complete specified action route, not an executed provider/unit fixture or independent R/E/V review.

<a id="wf7-17"></a>
### WF7-17. P3 inventory and constrained-branch action lifecycle

**Scope:** SC-33, SC-34. **Action route:** [AC-13](#ac-13), [AC-07](#ac-07), [AC-08](#ac-08), [AC-21](#ac-21), [AC-23](#ac-23), [AC-54](#ac-54), [AC-09](#ac-09), [AC-29](#ac-29), [AC-32](#ac-32), [AC-36](#ac-36), [AC-37](#ac-37), [AC-47](#ac-47), [AC-48](#ac-48), [AC-49](#ac-49).

1. P04 represents physical material amount, composition, total U and V with unresolved T,P and no flow workaround.
2. P05 forms an inventory or explicitly restricted-branch problem under captured eligibility.
3. P08 reports actual available realization; absent numerical coverage remains explicit.
4. For a purely conceptual review, show the resulting checks and state dependencies rather than inventing a solved state.
5. Lifting a restriction creates a new problem/context and invalidates applicability of the earlier constrained result to it.

**Successful boundary:** Concrete semantics for nonempty/empty inventory and branch restrictions; no implied time integrator or P3 E/V pass.

**Failure/partial branches:**

- U,V without material amount/composition: underdetermined.
- Empty inventory: no fabricated intensive fluid state.
- Suppressed solid: no unrestricted stability certificate.
- Restricted state cannot be relabeled after the restriction is removed.

**Publication scope:** Representability and explicitly bounded constrained-result scope only.

**Acceptance witnesses to execute:** No dummy kg/s or implicit one-second duration appears. A branch seed alone cannot become phase-suppression authority.

**Status:** Complete specified action route, not an executed provider/unit fixture or independent R/E/V review.

<a id="wf7-18"></a>
### WF7-18. Preserve an external phase allocation and add reporting properties

**Scope:** SC-09, SC-10, SC-12, SC-14, SC-26. **Action route:** [AC-13](#ac-13), [AC-16](#ac-16), [AC-17](#ac-17), [AC-19](#ac-19), [AC-21](#ac-21), [AC-22](#ac-22), [AC-28](#ac-28), [AC-29](#ac-29), [AC-30](#ac-30), [AC-32](#ac-32), [AC-33](#ac-33), [AC-36](#ac-36), [AC-37](#ac-37), [AC-49](#ac-49), [AC-55](#ac-55), [AC-56](#ac-56).

1. Capture the external allocation with its actual conventions, source and authority through AC-13/16.
2. AC-21/22 preserves phase/species allocation while permitting only requested dependent property coordinates.
3. Map and execute an eligible phase-property request, not a default bulk reflash.
4. AC-36/37 qualifies promised primary outputs and separately records optional failures.
5. AC-17 produces basis-qualified reporting views; reference-condition calculations use a separate bounded request.
6. AC-49 adopts only authorized completed scope; AC-56 reports currentness, partiality and provenance without material mutation.

**Successful boundary:** Externally owned allocation is preserved and supplementary properties/report views are attributable and nonadditive.

**Failure/partial branches:**

- Provider can only reflash: reject property-only capability or choose an eligible route.
- Missing standard-volume reference or carrier denominator: incomplete report only.
- Optional property native crash: preserve detached prior primary assessment and quarantine the new working context; do not salvage unknown shared-buffer fields.

**Publication scope:** External supplied allocation versus refreshed property scope and non-additive reporting view.

**Acceptance witnesses to execute:** Wet/dry reporting leaves actual water material unchanged. A missing optional property cannot reset phase fractions or erase primary result lineage.

**Status:** Complete specified action route, not an executed provider/unit fixture or independent R/E/V review.

## 9. Consequential ordering, replay and resource decisions

**AD-01 | Action versus endpoint.** Actions define intent, authority, outputs and completion boundaries. They need not be one RPC, public method, process or service. Compound commands may coordinate several actions without weakening their boundaries.

**AD-02 | Replay versus recomputation.** A numerical retry is a new attempt. Repeated delivery of an identified mutation returns its recorded decision. Exactly-once external delivery is not assumed; duplicate-safe committed effects and outcome reconciliation are required.

**AD-03 | Check-and-commit boundary.** P10 checks dependency/run/ancestor/scope/target-generation conditions at the logical point that updates current references. Checking once before a long validation phase is insufficient.

**AD-04 | Cancellation ordering.** Revocation blocks future writes. A publication already committed before cancellation remains historically and numerically assessed; withdrawal is explicit. Logical cancellation does not prove native interruption or safe resource reuse.

**AD-05 | Input edit success is not solve success.** A valid author edit can commit even when the next solve fails. The model remains edited; prior results become stale rather than silently undoing the author or pretending the old result matches.

**AD-06 | Candidate group coherence.** Identical base-model revision alone is not enough. Group acceptance also checks input lineage, run-local dependency view, actual original problems and process residuals. Distinct local problem IDs are normal.

**AD-07 | Optional work failure.** A detached successful primary result may survive independent optional failure. If a shared native call fails without trustworthy primary output, rerun/qualify primary work separately or withhold acceptance; never invent a successful primary outcome.

**AD-08 | Verification is bounded work.** Missing evidence may trigger a new request preserving the checked state and convention. Evidence acquisition has a bound and cannot recursively depend on the acceptance verdict it is intended to establish.

**AD-09 | Retry and alternative.** Physical restrictions, species choices, model substitution and approved data changes create an alternative problem. Temporary initialization simplifications are logged and restored before original-problem checks.

**AD-10 | Restoration and archives.** An archive retains captured meaning and history, not live command effects or old execution permissions. Inspectability, semantic reconstruction, dependency availability and numerical readiness remain separate.

**AD-11 | Transfer identity.** Actual modeled exchange and prescribed accounting actions retain an explicit intent and source snapshot. Iterations, retries, map views and duplicate callbacks do not each apply another physical transfer.

**AD-12 | Implementation freedom.** One coherent external engine may realize multiple actions. Contract IDs do not mandate a universal host flash, symbolic representation, database, event bus, distributed service, thread model or language-specific library.

**AD-13 | P3 action depth.** Surface/distribution/inventory/restricted examples now have explicit transformation and lifecycle routes. They still require independent representability review and do not receive numerical execution or validation commitments from this document.

**AD-14 | Evidence status.** This stage transforms fixed B1..B6 baselines. Structural crosswalks and the included small sequential reference model test only their declared catalog/guard/arithmetic claims, not real provider behavior or the original acceptance suite.

### 9.1 Two legitimate race orderings

| Ordering | Required result |
| --- | --- |
| Publish A commits, then edit B commits | A was a legitimate result of A at publication; B immediately marks its affected binding stale. The immutable A values/assessment survive. |
| Edit B commits, then A requests publication | A fails the dependency/currentness guard and becomes history only. |
| Cancel commits before native completion | Native work may finish; candidate can remain useful evidence but publication is rejected. |
| Publication commits before cancellation | The existing publication is not retroactively invalid solely because of later cancellation. Future writes are forbidden; explicit withdrawal remains separate. |
| Two qualified proposals compete for the same slot generation | At most the proposal satisfying the expected slot/run policy can update it. A stale expected slot cannot override a newer result merely because base physics is unchanged. |
| A publication response is lost, newer result commits, old command is redelivered | Return the old command decision without reapplying the old update. Current state remains the newer result. |

### 9.2 Consistent continuation versus original-problem identity

A continuation or parameter-fit campaign can intentionally solve a sequence of altered trial conditions. Each actual trial retains its physical input identity and numerical ancestry. Success at a homotopy endpoint or trial coefficient set is not the original target unless the original conditions have been restored and checked. Parent campaign membership does not make different trial problems scientifically identical. The same restriction applies when a solver inserts or removes structural phase slots: the record must identify whether this implements the unchanged admitted-domain problem or changes that problem.

### 9.3 Query and display behavior

A strict current query follows the current binding and must expose unavailable/stale scope rather than quietly return the last visible number. A historical/diagnostic query can return that number with its original status and lineage. Updating a display or requesting an optional report cannot change material state or result authority. Independent live context facts may change while historical numerical evidence remains intact.

## 10. Traceability and inheritance

### 10.1 Functional requirements to action owners

| Requirement | Unchanged owner/class | Primary action | Original witness IDs |
| --- | --- | --- | --- |
| [FR-GOV-01](thermodynamics_functional_requirements_v0_1.md#fr-gov-01) | P09 / H | [AC-38](#ac-38) | AT-GOV-01-P, AT-GOV-01-N |
| [FR-GOV-02](thermodynamics_functional_requirements_v0_1.md#fr-gov-02) | P08 / H | [AC-27](#ac-27) | AT-GOV-02-P, AT-GOV-02-N |
| [FR-GOV-03](thermodynamics_functional_requirements_v0_1.md#fr-gov-03) | P08 / H | [AC-27](#ac-27) | AT-GOV-03-P, AT-GOV-03-N |
| [FR-GOV-04](thermodynamics_functional_requirements_v0_1.md#fr-gov-04) | P03 / H | [AC-07](#ac-07) | AT-GOV-04-P, AT-GOV-04-N |
| [FR-GOV-05](thermodynamics_functional_requirements_v0_1.md#fr-gov-05) | P09 / H | [AC-38](#ac-38) | AT-GOV-05-P, AT-GOV-05-N |
| [FR-MAT-01](thermodynamics_functional_requirements_v0_1.md#fr-mat-01) | P01 / H | [AC-01](#ac-01) | AT-MAT-01-P, AT-MAT-01-N |
| [FR-MAT-02](thermodynamics_functional_requirements_v0_1.md#fr-mat-02) | P01 / H | [AC-02](#ac-02) | AT-MAT-02-P, AT-MAT-02-N |
| [FR-MAT-03](thermodynamics_functional_requirements_v0_1.md#fr-mat-03) | P04 / H | [AC-13](#ac-13) | AT-MAT-03-P, AT-MAT-03-N |
| [FR-MAT-04](thermodynamics_functional_requirements_v0_1.md#fr-mat-04) | P04 / H | [AC-13](#ac-13) | AT-MAT-04-P, AT-MAT-04-N |
| [FR-MAT-05](thermodynamics_functional_requirements_v0_1.md#fr-mat-05) | P04 / H | [AC-14](#ac-14) | AT-MAT-05-P, AT-MAT-05-N |
| [FR-MAT-06](thermodynamics_functional_requirements_v0_1.md#fr-mat-06) | P04 / H | [AC-15](#ac-15) | AT-MAT-06-P, AT-MAT-06-N |
| [FR-MAT-07](thermodynamics_functional_requirements_v0_1.md#fr-mat-07) | P01 / H | [AC-02](#ac-02) | AT-MAT-07-P, AT-MAT-07-N |
| [FR-MAT-08](thermodynamics_functional_requirements_v0_1.md#fr-mat-08) | P01 / H | [AC-02](#ac-02) | AT-MAT-08-P, AT-MAT-08-N |
| [FR-DAT-01](thermodynamics_functional_requirements_v0_1.md#fr-dat-01) | P02 / H | [AC-04](#ac-04) | AT-DAT-01-P, AT-DAT-01-N |
| [FR-DAT-02](thermodynamics_functional_requirements_v0_1.md#fr-dat-02) | P02 / H | [AC-04](#ac-04) | AT-DAT-02-P, AT-DAT-02-N |
| [FR-DAT-03](thermodynamics_functional_requirements_v0_1.md#fr-dat-03) | P02 / H | [AC-05](#ac-05) | AT-DAT-03-P, AT-DAT-03-N |
| [FR-DAT-04](thermodynamics_functional_requirements_v0_1.md#fr-dat-04) | P02 / C | [AC-06](#ac-06) | AT-DAT-04-P, AT-DAT-04-N |
| [FR-DAT-05](thermodynamics_functional_requirements_v0_1.md#fr-dat-05) | P02 / C | [AC-06](#ac-06) | AT-DAT-05-P, AT-DAT-05-N |
| [FR-DAT-06](thermodynamics_functional_requirements_v0_1.md#fr-dat-06) | P02 / H | [AC-06](#ac-06) | AT-DAT-06-P, AT-DAT-06-N |
| [FR-DAT-07](thermodynamics_functional_requirements_v0_1.md#fr-dat-07) | P02 / H | [AC-04](#ac-04) | AT-DAT-07-P, AT-DAT-07-N |
| [FR-CFG-01](thermodynamics_functional_requirements_v0_1.md#fr-cfg-01) | P03 / H | [AC-07](#ac-07) | AT-CFG-01-P, AT-CFG-01-N |
| [FR-CFG-02](thermodynamics_functional_requirements_v0_1.md#fr-cfg-02) | P03 / H | [AC-08](#ac-08) | AT-CFG-02-P, AT-CFG-02-N |
| [FR-CFG-03](thermodynamics_functional_requirements_v0_1.md#fr-cfg-03) | P03 / H | [AC-09](#ac-09) | AT-CFG-03-P, AT-CFG-03-N |
| [FR-CFG-04](thermodynamics_functional_requirements_v0_1.md#fr-cfg-04) | P03 / H | [AC-08](#ac-08) | AT-CFG-04-P, AT-CFG-04-N |
| [FR-CFG-05](thermodynamics_functional_requirements_v0_1.md#fr-cfg-05) | P03 / H | [AC-07](#ac-07) | AT-CFG-05-P, AT-CFG-05-N |
| [FR-CFG-06](thermodynamics_functional_requirements_v0_1.md#fr-cfg-06) | P07 / H | [AC-19](#ac-19) | AT-CFG-06-P, AT-CFG-06-N |
| [FR-CFG-07](thermodynamics_functional_requirements_v0_1.md#fr-cfg-07) | P03 / H | [AC-07](#ac-07) | AT-CFG-07-P, AT-CFG-07-N |
| [FR-STA-01](thermodynamics_functional_requirements_v0_1.md#fr-sta-01) | P04 / H | [AC-13](#ac-13) | AT-STA-01-P, AT-STA-01-N |
| [FR-STA-02](thermodynamics_functional_requirements_v0_1.md#fr-sta-02) | P05 / H | [AC-21](#ac-21) | AT-STA-02-P, AT-STA-02-N |
| [FR-STA-03](thermodynamics_functional_requirements_v0_1.md#fr-sta-03) | P05 / H | [AC-21](#ac-21) | AT-STA-03-P, AT-STA-03-N |
| [FR-STA-04](thermodynamics_functional_requirements_v0_1.md#fr-sta-04) | P04 / H | [AC-13](#ac-13) | AT-STA-04-P, AT-STA-04-N |
| [FR-STA-05](thermodynamics_functional_requirements_v0_1.md#fr-sta-05) | P04 / H | [AC-16](#ac-16) | AT-STA-05-P, AT-STA-05-N |
| [FR-STA-06](thermodynamics_functional_requirements_v0_1.md#fr-sta-06) | P04 / H | [AC-16](#ac-16) | AT-STA-06-P, AT-STA-06-N |
| [FR-STA-07](thermodynamics_functional_requirements_v0_1.md#fr-sta-07) | P04 / H | [AC-15](#ac-15) | AT-STA-07-P, AT-STA-07-N |
| [FR-STA-08](thermodynamics_functional_requirements_v0_1.md#fr-sta-08) | P04 / H | [AC-17](#ac-17) | AT-STA-08-P, AT-STA-08-N |
| [FR-EQL-01](thermodynamics_functional_requirements_v0_1.md#fr-eql-01) | P05 / C | [AC-23](#ac-23) | AT-EQL-01-P, AT-EQL-01-N |
| [FR-EQL-02](thermodynamics_functional_requirements_v0_1.md#fr-eql-02) | P05 / C | [AC-23](#ac-23) | AT-EQL-02-P, AT-EQL-02-N |
| [FR-EQL-03](thermodynamics_functional_requirements_v0_1.md#fr-eql-03) | P05 / C | [AC-23](#ac-23) | AT-EQL-03-P, AT-EQL-03-N |
| [FR-EQL-04](thermodynamics_functional_requirements_v0_1.md#fr-eql-04) | P05 / C | [AC-23](#ac-23) | AT-EQL-04-P, AT-EQL-04-N |
| [FR-EQL-05](thermodynamics_functional_requirements_v0_1.md#fr-eql-05) | P05 / H | [AC-22](#ac-22) | AT-EQL-05-P, AT-EQL-05-N |
| [FR-EQL-06](thermodynamics_functional_requirements_v0_1.md#fr-eql-06) | P05 / H | [AC-23](#ac-23) | AT-EQL-06-P, AT-EQL-06-N |
| [FR-EQL-07](thermodynamics_functional_requirements_v0_1.md#fr-eql-07) | P05 / H | [AC-23](#ac-23) | AT-EQL-07-P, AT-EQL-07-N |
| [FR-EQL-08](thermodynamics_functional_requirements_v0_1.md#fr-eql-08) | P05 / C | [AC-23](#ac-23) | AT-EQL-08-P, AT-EQL-08-N |
| [FR-PRP-01](thermodynamics_functional_requirements_v0_1.md#fr-prp-01) | P05 / C | [AC-22](#ac-22) | AT-PRP-01-P, AT-PRP-01-N |
| [FR-PRP-02](thermodynamics_functional_requirements_v0_1.md#fr-prp-02) | P05 / H | [AC-22](#ac-22) | AT-PRP-02-P, AT-PRP-02-N |
| [FR-PRP-03](thermodynamics_functional_requirements_v0_1.md#fr-prp-03) | P05 / C | [AC-22](#ac-22) | AT-PRP-03-P, AT-PRP-03-N |
| [FR-PRP-04](thermodynamics_functional_requirements_v0_1.md#fr-prp-04) | P05 / C | [AC-22](#ac-22) | AT-PRP-04-P, AT-PRP-04-N |
| [FR-PRP-05](thermodynamics_functional_requirements_v0_1.md#fr-prp-05) | P05 / H | [AC-25](#ac-25) | AT-PRP-05-P, AT-PRP-05-N |
| [FR-PRP-06](thermodynamics_functional_requirements_v0_1.md#fr-prp-06) | P05 / C | [AC-25](#ac-25) | AT-PRP-06-P, AT-PRP-06-N |
| [FR-PRP-07](thermodynamics_functional_requirements_v0_1.md#fr-prp-07) | P03 / H | [AC-08](#ac-08) | AT-PRP-07-P, AT-PRP-07-N |
| [FR-CHM-01](thermodynamics_functional_requirements_v0_1.md#fr-chm-01) | P06 / C | [AC-10](#ac-10) | AT-CHM-01-P, AT-CHM-01-N |
| [FR-CHM-02](thermodynamics_functional_requirements_v0_1.md#fr-chm-02) | P06 / C | [AC-11](#ac-11) | AT-CHM-02-P, AT-CHM-02-N |
| [FR-CHM-03](thermodynamics_functional_requirements_v0_1.md#fr-chm-03) | P05 / C | [AC-24](#ac-24) | AT-CHM-03-P, AT-CHM-03-N |
| [FR-CHM-04](thermodynamics_functional_requirements_v0_1.md#fr-chm-04) | P06 / C | [AC-10](#ac-10) | AT-CHM-04-P, AT-CHM-04-N |
| [FR-CHM-05](thermodynamics_functional_requirements_v0_1.md#fr-chm-05) | P03 / H | [AC-08](#ac-08) | AT-CHM-05-P, AT-CHM-05-N |
| [FR-CHM-06](thermodynamics_functional_requirements_v0_1.md#fr-chm-06) | P06 / C | [AC-12](#ac-12) | AT-CHM-06-P, AT-CHM-06-N |
| [FR-CHM-07](thermodynamics_functional_requirements_v0_1.md#fr-chm-07) | P06 / C | [AC-10](#ac-10) | AT-CHM-07-P, AT-CHM-07-N |
| [FR-CHM-08](thermodynamics_functional_requirements_v0_1.md#fr-chm-08) | P09 / C | [AC-37](#ac-37) | AT-CHM-08-P, AT-CHM-08-N |
| [FR-FLW-01](thermodynamics_functional_requirements_v0_1.md#fr-flw-01) | P07 / H | [AC-20](#ac-20) | AT-FLW-01-P, AT-FLW-01-N |
| [FR-FLW-02](thermodynamics_functional_requirements_v0_1.md#fr-flw-02) | P07 / C | [AC-39](#ac-39) | AT-FLW-02-P, AT-FLW-02-N |
| [FR-FLW-03](thermodynamics_functional_requirements_v0_1.md#fr-flw-03) | P07 / C | [AC-40](#ac-40) | AT-FLW-03-P, AT-FLW-03-N |
| [FR-FLW-04](thermodynamics_functional_requirements_v0_1.md#fr-flw-04) | P07 / C | [AC-41](#ac-41) | AT-FLW-04-P, AT-FLW-04-N |
| [FR-FLW-05](thermodynamics_functional_requirements_v0_1.md#fr-flw-05) | P07 / C | [AC-42](#ac-42) | AT-FLW-05-P, AT-FLW-05-N |
| [FR-FLW-06](thermodynamics_functional_requirements_v0_1.md#fr-flw-06) | P07 / C | [AC-42](#ac-42) | AT-FLW-06-P, AT-FLW-06-N |
| [FR-FLW-07](thermodynamics_functional_requirements_v0_1.md#fr-flw-07) | P07 / C | [AC-43](#ac-43) | AT-FLW-07-P, AT-FLW-07-N |
| [FR-FLW-08](thermodynamics_functional_requirements_v0_1.md#fr-flw-08) | P07 / C | [AC-43](#ac-43) | AT-FLW-08-P, AT-FLW-08-N |
| [FR-RUN-01](thermodynamics_functional_requirements_v0_1.md#fr-run-01) | P08 / H | [AC-31](#ac-31) | AT-RUN-01-P, AT-RUN-01-N |
| [FR-RUN-02](thermodynamics_functional_requirements_v0_1.md#fr-run-02) | P08 / H | [AC-31](#ac-31) | AT-RUN-02-P, AT-RUN-02-N |
| [FR-RUN-03](thermodynamics_functional_requirements_v0_1.md#fr-run-03) | P08 / H | [AC-30](#ac-30) | AT-RUN-03-P, AT-RUN-03-N |
| [FR-RUN-04](thermodynamics_functional_requirements_v0_1.md#fr-run-04) | P08 / H | [AC-30](#ac-30) | AT-RUN-04-P, AT-RUN-04-N |
| [FR-RUN-05](thermodynamics_functional_requirements_v0_1.md#fr-run-05) | P08 / H | [AC-34](#ac-34) | AT-RUN-05-P, AT-RUN-05-N |
| [FR-RUN-06](thermodynamics_functional_requirements_v0_1.md#fr-run-06) | P08 / H | [AC-35](#ac-35) | AT-RUN-06-P, AT-RUN-06-N |
| [FR-RUN-07](thermodynamics_functional_requirements_v0_1.md#fr-run-07) | P08 / H | [AC-35](#ac-35) | AT-RUN-07-P, AT-RUN-07-N |
| [FR-RUN-08](thermodynamics_functional_requirements_v0_1.md#fr-run-08) | P07 / H | [AC-44](#ac-44) | AT-RUN-08-P, AT-RUN-08-N |
| [FR-RES-01](thermodynamics_functional_requirements_v0_1.md#fr-res-01) | P09 / H | [AC-37](#ac-37) | AT-RES-01-P, AT-RES-01-N |
| [FR-RES-02](thermodynamics_functional_requirements_v0_1.md#fr-res-02) | P09 / H | [AC-36](#ac-36) | AT-RES-02-P, AT-RES-02-N |
| [FR-RES-03](thermodynamics_functional_requirements_v0_1.md#fr-res-03) | P09 / H | [AC-36](#ac-36) | AT-RES-03-P, AT-RES-03-N |
| [FR-RES-04](thermodynamics_functional_requirements_v0_1.md#fr-res-04) | P09 / H | [AC-37](#ac-37) | AT-RES-04-P, AT-RES-04-N |
| [FR-RES-05](thermodynamics_functional_requirements_v0_1.md#fr-res-05) | P10 / H | [AC-49](#ac-49) | AT-RES-05-P, AT-RES-05-N |
| [FR-RES-06](thermodynamics_functional_requirements_v0_1.md#fr-res-06) | P09 / H | [AC-56](#ac-56) | AT-RES-06-P, AT-RES-06-N |
| [FR-RES-07](thermodynamics_functional_requirements_v0_1.md#fr-res-07) | P09 / H | [AC-37](#ac-37) | AT-RES-07-P, AT-RES-07-N |
| [FR-RES-08](thermodynamics_functional_requirements_v0_1.md#fr-res-08) | P09 / H | [AC-38](#ac-38) | AT-RES-08-P, AT-RES-08-N |
| [FR-LIF-01](thermodynamics_functional_requirements_v0_1.md#fr-lif-01) | P10 / H | [AC-48](#ac-48) | AT-LIF-01-P, AT-LIF-01-N |
| [FR-LIF-02](thermodynamics_functional_requirements_v0_1.md#fr-lif-02) | P10 / H | [AC-47](#ac-47) | AT-LIF-02-P, AT-LIF-02-N |
| [FR-LIF-03](thermodynamics_functional_requirements_v0_1.md#fr-lif-03) | P10 / H | [AC-47](#ac-47) | AT-LIF-03-P, AT-LIF-03-N |
| [FR-LIF-04](thermodynamics_functional_requirements_v0_1.md#fr-lif-04) | P10 / H | [AC-50](#ac-50) | AT-LIF-04-P, AT-LIF-04-N |
| [FR-LIF-05](thermodynamics_functional_requirements_v0_1.md#fr-lif-05) | P10 / H | [AC-51](#ac-51) | AT-LIF-05-P, AT-LIF-05-N |
| [FR-LIF-06](thermodynamics_functional_requirements_v0_1.md#fr-lif-06) | P10 / H | [AC-52](#ac-52) | AT-LIF-06-P, AT-LIF-06-N |
| [FR-LIF-07](thermodynamics_functional_requirements_v0_1.md#fr-lif-07) | P10 / H | [AC-53](#ac-53) | AT-LIF-07-P, AT-LIF-07-N |
| [FR-LIF-08](thermodynamics_functional_requirements_v0_1.md#fr-lif-08) | P10 / H | [AC-48](#ac-48) | AT-LIF-08-P, AT-LIF-08-N |
| [FR-EXT-01](thermodynamics_functional_requirements_v0_1.md#fr-ext-01) | P01 / R3 | [AC-03](#ac-03) | AT-EXT-01-P, AT-EXT-01-N |
| [FR-EXT-02](thermodynamics_functional_requirements_v0_1.md#fr-ext-02) | P01 / R3 | [AC-03](#ac-03) | AT-EXT-02-P, AT-EXT-02-N |
| [FR-EXT-03](thermodynamics_functional_requirements_v0_1.md#fr-ext-03) | P04 / R3 | [AC-13](#ac-13) | AT-EXT-03-P, AT-EXT-03-N |
| [FR-EXT-04](thermodynamics_functional_requirements_v0_1.md#fr-ext-04) | P05 / R3 | [AC-23](#ac-23) | AT-EXT-04-P, AT-EXT-04-N |

### 10.2 Scenario coverage routes

| Scenario | Title | Profile | Step-7 workflow | Primary requirement actions |
| --- | --- | --- | --- | --- |
| SC-01 | Blending and splitting without reaction | P1 | [WF7-01](#wf7-01), [WF7-02](#wf7-02), [WF7-04](#wf7-04), [WF7-09](#wf7-09), [WF7-14](#wf7-14) | [AC-02](#ac-02), [AC-14](#ac-14), [AC-15](#ac-15), [AC-36](#ac-36), [AC-39](#ac-39) |
| SC-02 | Sensible heating and heat exchange | P1 | [WF7-03](#wf7-03), [WF7-05](#wf7-05) | [AC-08](#ac-08), [AC-20](#ac-20), [AC-22](#ac-22), [AC-23](#ac-23), [AC-41](#ac-41) |
| SC-03 | Liquid pumping and pressure-loss calculations | P1 | [WF7-03](#wf7-03) | [AC-09](#ac-09), [AC-20](#ac-20), [AC-22](#ac-22), [AC-23](#ac-23) |
| SC-04 | Gas compression, expansion and intercooling | P1 | [WF7-03](#wf7-03), [WF7-12](#wf7-12) | [AC-08](#ac-08), [AC-13](#ac-13), [AC-20](#ac-20), [AC-23](#ac-23), [AC-25](#ac-25) |
| SC-05 | Cooling and vapor–liquid separation | P1 | [WF7-03](#wf7-03), [WF7-04](#wf7-04), [WF7-09](#wf7-09), [WF7-13](#wf7-13) | [AC-16](#ac-16), [AC-23](#ac-23), [AC-36](#ac-36), [AC-40](#ac-40) |
| SC-06 | Pressure reduction with flashing | P1 | [WF7-03](#wf7-03), [WF7-09](#wf7-09), [WF7-13](#wf7-13), [WF7-14](#wf7-14) | [AC-21](#ac-21), [AC-23](#ac-23), [AC-31](#ac-31), [AC-36](#ac-36), [AC-37](#ac-37), [AC-49](#ac-49) |
| SC-07 | Conventional equilibrium-stage distillation | P1 | [WF7-08](#wf7-08), [WF7-09](#wf7-09), [WF7-12](#wf7-12) | [AC-13](#ac-13), [AC-22](#ac-22), [AC-31](#ac-31), [AC-37](#ac-37), [AC-43](#ac-43), [AC-44](#ac-44) |
| SC-08 | Nonideal-liquid separation and azeotropic behavior | P1 | [WF7-01](#wf7-01), [WF7-08](#wf7-08) | [AC-04](#ac-04), [AC-05](#ac-05), [AC-08](#ac-08), [AC-23](#ac-23), [AC-34](#ac-34) |
| SC-09 | Physical absorption, humidification and gas dissolution | P1 | [WF7-01](#wf7-01), [WF7-08](#wf7-08), [WF7-18](#wf7-18) | [AC-07](#ac-07), [AC-08](#ac-08), [AC-17](#ac-17), [AC-20](#ac-20), [AC-22](#ac-22) |
| SC-10 | Liquid–liquid extraction and decanting | P2 | [WF7-04](#wf7-04), [WF7-08](#wf7-08), [WF7-18](#wf7-18) | [AC-16](#ac-16), [AC-22](#ac-22), [AC-23](#ac-23), [AC-40](#ac-40) |
| SC-11 | Vapor–liquid–liquid separation | P2 | [WF7-04](#wf7-04), [WF7-13](#wf7-13) | [AC-07](#ac-07), [AC-23](#ac-23), [AC-36](#ac-36) |
| SC-12 | Water and steam through saturation | P1 | [WF7-03](#wf7-03), [WF7-05](#wf7-05), [WF7-18](#wf7-18) | [AC-17](#ac-17), [AC-21](#ac-21), [AC-23](#ac-23) |
| SC-13 | Pure-fluid refrigeration loop | P1 | [WF7-03](#wf7-03), [WF7-09](#wf7-09) | [AC-01](#ac-01), [AC-23](#ac-23), [AC-44](#ac-44), [AC-53](#ac-53) |
| SC-14 | Mixed-refrigerant phase change | P2 | [WF7-03](#wf7-03), [WF7-18](#wf7-18) | [AC-02](#ac-02), [AC-08](#ac-08), [AC-17](#ac-17), [AC-23](#ac-23) |
| SC-15 | Assay-derived petroleum pseudocomponents | P2 | [WF7-11](#wf7-11), [WF7-15](#wf7-15) | [AC-01](#ac-01), [AC-04](#ac-04), [AC-06](#ac-06), [AC-08](#ac-08), [AC-50](#ac-50), [AC-53](#ac-53) |
| SC-16 | Black-oil or other reduced petroleum representation | P2 | [WF7-10](#wf7-10), [WF7-11](#wf7-11) | [AC-02](#ac-02), [AC-04](#ac-04), [AC-09](#ac-09), [AC-17](#ac-17), [AC-42](#ac-42) |
| SC-17 | Specified-conversion reaction | P1 | [WF7-06](#wf7-06), [WF7-09](#wf7-09) | [AC-02](#ac-02), [AC-08](#ac-08), [AC-10](#ac-10), [AC-11](#ac-11), [AC-23](#ac-23), [AC-36](#ac-36) |
| SC-18 | Chemical-equilibrium reaction | P1 | [WF7-07](#wf7-07), [WF7-09](#wf7-09) | [AC-02](#ac-02), [AC-07](#ac-07), [AC-08](#ac-08), [AC-10](#ac-10), [AC-24](#ac-24) |
| SC-19 | Kinetically controlled reaction | P2 | [WF7-06](#wf7-06) | [AC-10](#ac-10), [AC-13](#ac-13), [AC-22](#ac-22), [AC-25](#ac-25) |
| SC-20 | Reactive separation | P2 | [WF7-07](#wf7-07), [WF7-08](#wf7-08) | [AC-08](#ac-08), [AC-23](#ac-23), [AC-37](#ac-37), [AC-43](#ac-43), [AC-44](#ac-44) |
| SC-21 | Electrolyte mixing and neutralization | P2 | [WF7-07](#wf7-07), [WF7-10](#wf7-10) | [AC-02](#ac-02), [AC-08](#ac-08), [AC-12](#ac-12), [AC-24](#ac-24), [AC-42](#ac-42) |
| SC-22 | Reactive gas absorption into aqueous liquid | P2 | [WF7-07](#wf7-07) | [AC-08](#ac-08), [AC-10](#ac-10), [AC-37](#ac-37), [AC-43](#ac-43) |
| SC-23 | Inert solids carried with fluid | P2 | [WF7-04](#wf7-04), [WF7-11](#wf7-11) | [AC-02](#ac-02), [AC-07](#ac-07), [AC-22](#ac-22), [AC-40](#ac-40) |
| SC-24 | Crystallization and precipitation | P2 | [WF7-07](#wf7-07), [WF7-13](#wf7-13) | [AC-08](#ac-08), [AC-16](#ac-16), [AC-23](#ac-23), [AC-37](#ac-37) |
| SC-25 | Gas–solid chemical transformation | P2 | [WF7-06](#wf7-06), [WF7-07](#wf7-07) | [AC-07](#ac-07), [AC-08](#ac-08), [AC-10](#ac-10), [AC-37](#ac-37) |
| SC-26 | Rate-based nonequilibrium contacting | P2 | [WF7-08](#wf7-08), [WF7-12](#wf7-12), [WF7-18](#wf7-18) | [AC-21](#ac-21), [AC-22](#ac-22), [AC-43](#ac-43), [AC-44](#ac-44) |
| SC-27 | Adsorption and membrane state extensions | P3 | [WF7-16](#wf7-16) | [AC-02](#ac-02), [AC-03](#ac-03), [AC-08](#ac-08), [AC-13](#ac-13), [AC-21](#ac-21) |
| SC-28 | Heat exchange between different property packages | P1 | [WF7-05](#wf7-05), [WF7-14](#wf7-14) | [AC-08](#ac-08), [AC-19](#ac-19), [AC-22](#ac-22), [AC-36](#ac-36), [AC-41](#ac-41) |
| SC-29 | Material transfer across a property-package boundary | P2 | [WF7-10](#wf7-10), [WF7-15](#wf7-15) | [AC-08](#ac-08), [AC-21](#ac-21), [AC-37](#ac-37), [AC-42](#ac-42) |
| SC-30 | Translation between material representations | P2 | [WF7-10](#wf7-10), [WF7-15](#wf7-15) | [AC-02](#ac-02), [AC-12](#ac-12), [AC-42](#ac-42), [AC-52](#ac-52) |
| SC-31 | Mass-based empirical and nonconventional materials | P2 | [WF7-11](#wf7-11), [WF7-15](#wf7-15) | [AC-02](#ac-02), [AC-09](#ac-09), [AC-13](#ac-13), [AC-22](#ac-22) |
| SC-32 | Polymer distributions and material attributes | P3 | [WF7-16](#wf7-16) | [AC-02](#ac-02), [AC-03](#ac-03), [AC-13](#ac-13), [AC-42](#ac-42), [AC-50](#ac-50) |
| SC-33 | Inventory-based state and dynamic compatibility | P3 | [WF7-17](#wf7-17) | [AC-13](#ac-13), [AC-15](#ac-15) |
| SC-34 | Restricted-equilibrium and metastable-state requests | P3 | [WF7-13](#wf7-13), [WF7-17](#wf7-17) | [AC-07](#ac-07), [AC-23](#ac-23), [AC-34](#ac-34) |

### 10.3 Information concepts and action handoffs

| Concept | Semantic owner | Produces/proposes | Consumes |
| --- | --- | --- | --- |
| [IC-01](thermodynamics_semantic_information_dictionary_v0_1.md#ic-01) Constituent identity | P01 | [AC-01](#ac-01) | [AC-01](#ac-01), [AC-02](#ac-02), [AC-04](#ac-04), [AC-10](#ac-10) |
| [IC-02](thermodynamics_semantic_information_dictionary_v0_1.md#ic-02) Identity assertion and alias resolution | P01 | [AC-01](#ac-01) | [AC-01](#ac-01) |
| [IC-03](thermodynamics_semantic_information_dictionary_v0_1.md#ic-03) Material representation revision | P01 | [AC-02](#ac-02) | [AC-02](#ac-02), [AC-03](#ac-03), [AC-06](#ac-06), [AC-07](#ac-07), [AC-08](#ac-08), [AC-10](#ac-10), [AC-12](#ac-12), [AC-13](#ac-13), [AC-14](#ac-14), [AC-15](#ac-15), [AC-18](#ac-18), [AC-28](#ac-28), [AC-39](#ac-39), [AC-42](#ac-42), [AC-47](#ac-47), [AC-50](#ac-50), [AC-51](#ac-51) |
| [IC-04](thermodynamics_semantic_information_dictionary_v0_1.md#ic-04) Composition coordinate system | P01 | [AC-02](#ac-02) | [AC-02](#ac-02), [AC-03](#ac-03), [AC-13](#ac-13), [AC-14](#ac-14), [AC-15](#ac-15), [AC-17](#ac-17), [AC-25](#ac-25), [AC-26](#ac-26), [AC-28](#ac-28) |
| [IC-05](thermodynamics_semantic_information_dictionary_v0_1.md#ic-05) Conserved-quantity basis | P01 | [AC-02](#ac-02) | [AC-02](#ac-02), [AC-03](#ac-03), [AC-10](#ac-10), [AC-12](#ac-12), [AC-13](#ac-13), [AC-18](#ac-18), [AC-36](#ac-36), [AC-39](#ac-39), [AC-42](#ac-42) |
| [IC-06](thermodynamics_semantic_information_dictionary_v0_1.md#ic-06) Material-domain definition | P01 | [AC-02](#ac-02) | [AC-02](#ac-02), [AC-03](#ac-03), [AC-10](#ac-10), [AC-13](#ac-13), [AC-16](#ac-16) |
| [IC-07](thermodynamics_semantic_information_dictionary_v0_1.md#ic-07) Representation map and reduction | P01 | [AC-03](#ac-03) | [AC-12](#ac-12), [AC-17](#ac-17), [AC-18](#ac-18), [AC-25](#ac-25), [AC-28](#ac-28), [AC-42](#ac-42) |
| [IC-08](thermodynamics_semantic_information_dictionary_v0_1.md#ic-08) Distributed attribute definition | P01 | [AC-03](#ac-03) | [AC-03](#ac-03), [AC-18](#ac-18) |
| [IC-09](thermodynamics_semantic_information_dictionary_v0_1.md#ic-09) Surface/site and loading basis | P01 | [AC-03](#ac-03) | [AC-03](#ac-03), [AC-10](#ac-10), [AC-18](#ac-18) |
| [IC-10](thermodynamics_semantic_information_dictionary_v0_1.md#ic-10) Evidence-source record | P02 | [AC-04](#ac-04) | [AC-01](#ac-01), [AC-04](#ac-04), [AC-05](#ac-05), [AC-06](#ac-06), [AC-27](#ac-27), [AC-38](#ac-38), [AC-50](#ac-50) |
| [IC-11](thermodynamics_semantic_information_dictionary_v0_1.md#ic-11) Property datum and observation | P02 | [AC-04](#ac-04) | [AC-04](#ac-04), [AC-06](#ac-06) |
| [IC-12](thermodynamics_semantic_information_dictionary_v0_1.md#ic-12) Parameter interpretation descriptor | P02 | [AC-04](#ac-04) | [AC-04](#ac-04), [AC-05](#ac-05), [AC-06](#ac-06), [AC-08](#ac-08) |
| [IC-13](thermodynamics_semantic_information_dictionary_v0_1.md#ic-13) Parameter value assertion | P02 | [AC-04](#ac-04), [AC-05](#ac-05) | [AC-04](#ac-04) |
| [IC-14](thermodynamics_semantic_information_dictionary_v0_1.md#ic-14) Resolved parameter snapshot | P02 | [AC-04](#ac-04), [AC-05](#ac-05), [AC-06](#ac-06) | [AC-05](#ac-05), [AC-06](#ac-06), [AC-07](#ac-07), [AC-08](#ac-08), [AC-10](#ac-10), [AC-27](#ac-27), [AC-47](#ac-47), [AC-50](#ac-50), [AC-51](#ac-51) |
| [IC-15](thermodynamics_semantic_information_dictionary_v0_1.md#ic-15) Validity and applicability description | P02 | [AC-04](#ac-04) | [AC-04](#ac-04), [AC-05](#ac-05), [AC-08](#ac-08) |
| [IC-16](thermodynamics_semantic_information_dictionary_v0_1.md#ic-16) Characterization and fitting recipe | P02 | [AC-06](#ac-06) | [AC-05](#ac-05), [AC-06](#ac-06), [AC-50](#ac-50) |
| [IC-17](thermodynamics_semantic_information_dictionary_v0_1.md#ic-17) Characterization or fit proposal and trials | P02 | [AC-05](#ac-05), [AC-06](#ac-06) | [AC-05](#ac-05), [AC-47](#ac-47), [AC-50](#ac-50) |
| [IC-18](thermodynamics_semantic_information_dictionary_v0_1.md#ic-18) Thermodynamic method definition | P03 | [AC-07](#ac-07) | [AC-04](#ac-04), [AC-07](#ac-07), [AC-08](#ac-08), [AC-27](#ac-27) |
| [IC-19](thermodynamics_semantic_information_dictionary_v0_1.md#ic-19) Reusable package recipe | P03 | [AC-07](#ac-07) | [AC-07](#ac-07) |
| [IC-20](thermodynamics_semantic_information_dictionary_v0_1.md#ic-20) Resolved thermodynamic package revision | P03 | [AC-07](#ac-07) | [AC-06](#ac-06), [AC-08](#ac-08), [AC-09](#ac-09), [AC-19](#ac-19), [AC-21](#ac-21), [AC-22](#ac-22), [AC-23](#ac-23), [AC-24](#ac-24), [AC-25](#ac-25), [AC-26](#ac-26), [AC-27](#ac-27), [AC-28](#ac-28), [AC-30](#ac-30), [AC-38](#ac-38), [AC-47](#ac-47), [AC-50](#ac-50), [AC-51](#ac-51), [AC-53](#ac-53), [AC-54](#ac-54) |
| [IC-21](thermodynamics_semantic_information_dictionary_v0_1.md#ic-21) Energy, standard-state and reference convention set | P03 | [AC-08](#ac-08) | [AC-07](#ac-07), [AC-08](#ac-08), [AC-10](#ac-10), [AC-17](#ac-17), [AC-23](#ac-23), [AC-24](#ac-24), [AC-28](#ac-28), [AC-36](#ac-36), [AC-39](#ac-39), [AC-41](#ac-41), [AC-42](#ac-42) |
| [IC-22](thermodynamics_semantic_information_dictionary_v0_1.md#ic-22) Physical eligibility and default policy | P03 | [AC-07](#ac-07) | [AC-07](#ac-07), [AC-16](#ac-16), [AC-19](#ac-19), [AC-23](#ac-23) |
| [IC-23](thermodynamics_semantic_information_dictionary_v0_1.md#ic-23) Method and convention compatibility assessment | P03 | [AC-08](#ac-08) | [AC-09](#ac-09) |
| [IC-24](thermodynamics_semantic_information_dictionary_v0_1.md#ic-24) Operation-specific readiness assessment | P03 | [AC-09](#ac-09) | [AC-21](#ac-21), [AC-29](#ac-29), [AC-31](#ac-31), [AC-56](#ac-56) |
| [IC-25](thermodynamics_semantic_information_dictionary_v0_1.md#ic-25) Local material account | P04 | [AC-13](#ac-13) | [AC-11](#ac-11), [AC-12](#ac-12), [AC-13](#ac-13), [AC-14](#ac-14), [AC-15](#ac-15), [AC-17](#ac-17), [AC-18](#ac-18), [AC-39](#ac-39), [AC-40](#ac-40) |
| [IC-26](thermodynamics_semantic_information_dictionary_v0_1.md#ic-26) Quantity/value assertion | P04 | [AC-13](#ac-13), [AC-15](#ac-15), [AC-17](#ac-17), [AC-33](#ac-33) | [AC-13](#ac-13), [AC-15](#ac-15), [AC-17](#ac-17), [AC-33](#ac-33) |
| [IC-27](thermodynamics_semantic_information_dictionary_v0_1.md#ic-27) Composition description | P04 | [AC-11](#ac-11), [AC-12](#ac-12), [AC-13](#ac-13), [AC-14](#ac-14), [AC-15](#ac-15), [AC-42](#ac-42) | [AC-11](#ac-11), [AC-12](#ac-12), [AC-13](#ac-13), [AC-14](#ac-14), [AC-15](#ac-15), [AC-17](#ac-17), [AC-47](#ac-47) |
| [IC-28](thermodynamics_semantic_information_dictionary_v0_1.md#ic-28) Local state snapshot | P04 | [AC-11](#ac-11), [AC-13](#ac-13), [AC-14](#ac-14), [AC-15](#ac-15), [AC-18](#ac-18), [AC-33](#ac-33), [AC-39](#ac-39), [AC-40](#ac-40), [AC-42](#ac-42) | [AC-11](#ac-11), [AC-12](#ac-12), [AC-14](#ac-14), [AC-15](#ac-15), [AC-16](#ac-16), [AC-17](#ac-17), [AC-18](#ac-18), [AC-21](#ac-21), [AC-22](#ac-22), [AC-23](#ac-23), [AC-24](#ac-24), [AC-25](#ac-25), [AC-26](#ac-26), [AC-31](#ac-31), [AC-33](#ac-33), [AC-39](#ac-39), [AC-40](#ac-40), [AC-41](#ac-41), [AC-42](#ac-42), [AC-43](#ac-43), [AC-47](#ac-47), [AC-50](#ac-50), [AC-51](#ac-51), [AC-56](#ac-56) |
| [IC-29](thermodynamics_semantic_information_dictionary_v0_1.md#ic-29) Domain portion and phase instance | P04 | [AC-16](#ac-16) | [AC-16](#ac-16), [AC-22](#ac-22), [AC-33](#ac-33), [AC-40](#ac-40) |
| [IC-30](thermodynamics_semantic_information_dictionary_v0_1.md#ic-30) Cross-snapshot phase correspondence | P04 | [AC-16](#ac-16) | [AC-40](#ac-40) |
| [IC-31](thermodynamics_semantic_information_dictionary_v0_1.md#ic-31) Aggregate and reporting view | P04 | [AC-16](#ac-16), [AC-17](#ac-17) | [AC-17](#ac-17) |
| [IC-32](thermodynamics_semantic_information_dictionary_v0_1.md#ic-32) Local distributed-attribute realization | P04 | [AC-18](#ac-18) | [AC-18](#ac-18) |
| [IC-33](thermodynamics_semantic_information_dictionary_v0_1.md#ic-33) Local surface/loading realization | P04 | [AC-18](#ac-18) | [AC-18](#ac-18) |
| [IC-34](thermodynamics_semantic_information_dictionary_v0_1.md#ic-34) Operation contract | P05 | [AC-21](#ac-21) | [AC-20](#ac-20), [AC-21](#ac-21), [AC-22](#ac-22), [AC-23](#ac-23), [AC-27](#ac-27) |
| [IC-35](thermodynamics_semantic_information_dictionary_v0_1.md#ic-35) Physical calculation problem | P05 | [AC-21](#ac-21), [AC-22](#ac-22), [AC-23](#ac-23), [AC-24](#ac-24), [AC-25](#ac-25), [AC-26](#ac-26), [AC-39](#ac-39), [AC-41](#ac-41), [AC-42](#ac-42), [AC-43](#ac-43), [AC-54](#ac-54), [AC-55](#ac-55) | [AC-06](#ac-06), [AC-09](#ac-09), [AC-11](#ac-11), [AC-22](#ac-22), [AC-23](#ac-23), [AC-24](#ac-24), [AC-25](#ac-25), [AC-26](#ac-26), [AC-28](#ac-28), [AC-29](#ac-29), [AC-31](#ac-31), [AC-32](#ac-32), [AC-33](#ac-33), [AC-34](#ac-34), [AC-36](#ac-36), [AC-37](#ac-37), [AC-39](#ac-39), [AC-41](#ac-41), [AC-42](#ac-42), [AC-43](#ac-43), [AC-44](#ac-44), [AC-45](#ac-45), [AC-50](#ac-50), [AC-53](#ac-53), [AC-54](#ac-54), [AC-55](#ac-55) |
| [IC-36](thermodynamics_semantic_information_dictionary_v0_1.md#ic-36) Specification, constraint and unknown set | P05 | [AC-21](#ac-21), [AC-23](#ac-23), [AC-24](#ac-24), [AC-54](#ac-54) | [AC-21](#ac-21), [AC-23](#ac-23), [AC-24](#ac-24), [AC-25](#ac-25), [AC-26](#ac-26), [AC-31](#ac-31), [AC-43](#ac-43), [AC-54](#ac-54) |
| [IC-37](thermodynamics_semantic_information_dictionary_v0_1.md#ic-37) Calculation-authority allocation | P05 | [AC-21](#ac-21), [AC-24](#ac-24), [AC-54](#ac-54) | [AC-21](#ac-21), [AC-22](#ac-22), [AC-23](#ac-23), [AC-24](#ac-24), [AC-26](#ac-26), [AC-32](#ac-32), [AC-36](#ac-36), [AC-43](#ac-43), [AC-54](#ac-54) |
| [IC-38](thermodynamics_semantic_information_dictionary_v0_1.md#ic-38) Property observable definition | P05 | [AC-22](#ac-22) | [AC-17](#ac-17), [AC-20](#ac-20), [AC-21](#ac-21), [AC-22](#ac-22), [AC-23](#ac-23), [AC-25](#ac-25), [AC-28](#ac-28), [AC-33](#ac-33), [AC-55](#ac-55) |
| [IC-39](thermodynamics_semantic_information_dictionary_v0_1.md#ic-39) Derivative and solved-response specification | P05 | [AC-25](#ac-25) | [AC-20](#ac-20), [AC-25](#ac-25), [AC-26](#ac-26), [AC-28](#ac-28), [AC-33](#ac-33), [AC-43](#ac-43), [AC-55](#ac-55) |
| [IC-40](thermodynamics_semantic_information_dictionary_v0_1.md#ic-40) Mathematical contribution description | P05 | [AC-26](#ac-26), [AC-43](#ac-43) | [AC-20](#ac-20), [AC-26](#ac-26), [AC-31](#ac-31), [AC-32](#ac-32), [AC-33](#ac-33), [AC-43](#ac-43) |
| [IC-41](thermodynamics_semantic_information_dictionary_v0_1.md#ic-41) Postcondition and check obligation | P05 | [AC-21](#ac-21), [AC-22](#ac-22), [AC-23](#ac-23), [AC-24](#ac-24), [AC-25](#ac-25), [AC-26](#ac-26), [AC-54](#ac-54) | [AC-20](#ac-20), [AC-21](#ac-21), [AC-22](#ac-22), [AC-23](#ac-23), [AC-24](#ac-24), [AC-25](#ac-25), [AC-26](#ac-26), [AC-29](#ac-29), [AC-31](#ac-31), [AC-32](#ac-32), [AC-34](#ac-34), [AC-36](#ac-36), [AC-37](#ac-37), [AC-38](#ac-38), [AC-41](#ac-41), [AC-42](#ac-42), [AC-43](#ac-43), [AC-44](#ac-44), [AC-54](#ac-54), [AC-55](#ac-55) |
| [IC-42](thermodynamics_semantic_information_dictionary_v0_1.md#ic-42) Effective physical restriction and branch request | P05 | [AC-21](#ac-21), [AC-23](#ac-23), [AC-54](#ac-54) | [AC-16](#ac-16), [AC-19](#ac-19), [AC-21](#ac-21), [AC-23](#ac-23), [AC-24](#ac-24), [AC-25](#ac-25), [AC-54](#ac-54) |
| [IC-43](thermodynamics_semantic_information_dictionary_v0_1.md#ic-43) Reaction identity and stoichiometry | P06 | [AC-10](#ac-10) | [AC-08](#ac-08), [AC-11](#ac-11), [AC-24](#ac-24) |
| [IC-44](thermodynamics_semantic_information_dictionary_v0_1.md#ic-44) Reaction-law and thermochemical requirement | P06 | [AC-10](#ac-10) | [AC-08](#ac-08), [AC-11](#ac-11), [AC-24](#ac-24) |
| [IC-45](thermodynamics_semantic_information_dictionary_v0_1.md#ic-45) Chemical system and participation description | P06 | [AC-10](#ac-10) | [AC-07](#ac-07), [AC-08](#ac-08), [AC-11](#ac-11), [AC-12](#ac-12), [AC-21](#ac-21), [AC-24](#ac-24), [AC-43](#ac-43), [AC-47](#ac-47), [AC-50](#ac-50), [AC-51](#ac-51), [AC-54](#ac-54) |
| [IC-46](thermodynamics_semantic_information_dictionary_v0_1.md#ic-46) Reservoir and chemical-exchange requirement | P06 | [AC-10](#ac-10) | [AC-21](#ac-21), [AC-24](#ac-24) |
| [IC-47](thermodynamics_semantic_information_dictionary_v0_1.md#ic-47) Apparent/true-species reconciliation | P06 | [AC-12](#ac-12) | [AC-08](#ac-08), [AC-12](#ac-12), [AC-24](#ac-24), [AC-42](#ac-42) |
| [IC-48](thermodynamics_semantic_information_dictionary_v0_1.md#ic-48) Process location and region description | P07 | [AC-19](#ac-19) | [AC-13](#ac-13), [AC-19](#ac-19), [AC-20](#ac-20), [AC-43](#ac-43), [AC-45](#ac-45), [AC-49](#ac-49) |
| [IC-49](thermodynamics_semantic_information_dictionary_v0_1.md#ic-49) Effective thermodynamic location binding | P07 | [AC-19](#ac-19) | [AC-13](#ac-13), [AC-19](#ac-19), [AC-20](#ac-20), [AC-21](#ac-21), [AC-41](#ac-41), [AC-42](#ac-42), [AC-43](#ac-43), [AC-47](#ac-47), [AC-50](#ac-50), [AC-51](#ac-51) |
| [IC-50](thermodynamics_semantic_information_dictionary_v0_1.md#ic-50) Unit demands, coupling and routing declaration | P07 | [AC-20](#ac-20) | [AC-09](#ac-09), [AC-18](#ac-18), [AC-20](#ac-20), [AC-21](#ac-21), [AC-22](#ac-22), [AC-24](#ac-24), [AC-26](#ac-26), [AC-36](#ac-36), [AC-39](#ac-39), [AC-40](#ac-40), [AC-41](#ac-41), [AC-42](#ac-42), [AC-43](#ac-43) |
| [IC-51](thermodynamics_semantic_information_dictionary_v0_1.md#ic-51) Material-boundary translation case | P07 | [AC-42](#ac-42) | [AC-42](#ac-42) |
| [IC-52](thermodynamics_semantic_information_dictionary_v0_1.md#ic-52) Process completion scope and run-local view | P07 | [AC-20](#ac-20), [AC-39](#ac-39), [AC-40](#ac-40), [AC-41](#ac-41), [AC-43](#ac-43), [AC-44](#ac-44), [AC-55](#ac-55) | [AC-20](#ac-20), [AC-36](#ac-36), [AC-37](#ac-37), [AC-39](#ac-39), [AC-40](#ac-40), [AC-41](#ac-41), [AC-43](#ac-43), [AC-44](#ac-44), [AC-45](#ac-45), [AC-46](#ac-46), [AC-49](#ac-49), [AC-55](#ac-55), [AC-56](#ac-56) |
| [IC-53](thermodynamics_semantic_information_dictionary_v0_1.md#ic-53) Provider realization and capability attestation | P08 | [AC-27](#ac-27) | [AC-07](#ac-07), [AC-09](#ac-09), [AC-27](#ac-27), [AC-28](#ac-28), [AC-29](#ac-29), [AC-30](#ac-30), [AC-32](#ac-32), [AC-33](#ac-33), [AC-34](#ac-34), [AC-35](#ac-35), [AC-38](#ac-38), [AC-47](#ac-47), [AC-50](#ac-50), [AC-51](#ac-51), [AC-53](#ac-53) |
| [IC-54](thermodynamics_semantic_information_dictionary_v0_1.md#ic-54) Execution plan and attempt record | P08 | [AC-29](#ac-29), [AC-31](#ac-31), [AC-32](#ac-32), [AC-34](#ac-34) | [AC-30](#ac-30), [AC-31](#ac-31), [AC-32](#ac-32), [AC-33](#ac-33), [AC-34](#ac-34), [AC-35](#ac-35), [AC-46](#ac-46) |
| [IC-55](thermodynamics_semantic_information_dictionary_v0_1.md#ic-55) Session and workspace lifecycle record | P08 | [AC-30](#ac-30), [AC-35](#ac-35) | [AC-30](#ac-30), [AC-31](#ac-31), [AC-32](#ac-32), [AC-33](#ac-33), [AC-35](#ac-35) |
| [IC-56](thermodynamics_semantic_information_dictionary_v0_1.md#ic-56) Provider candidate and execution evidence | P08 | [AC-31](#ac-31), [AC-32](#ac-32), [AC-33](#ac-33), [AC-39](#ac-39), [AC-40](#ac-40), [AC-41](#ac-41), [AC-42](#ac-42), [AC-43](#ac-43), [AC-55](#ac-55) | [AC-06](#ac-06), [AC-16](#ac-16), [AC-34](#ac-34), [AC-35](#ac-35), [AC-36](#ac-36), [AC-37](#ac-37), [AC-44](#ac-44), [AC-54](#ac-54), [AC-55](#ac-55), [AC-56](#ac-56) |
| [IC-57](thermodynamics_semantic_information_dictionary_v0_1.md#ic-57) Initialization and continuation description | P08 | [AC-31](#ac-31), [AC-34](#ac-34) | [AC-09](#ac-09), [AC-23](#ac-23), [AC-26](#ac-26), [AC-29](#ac-29), [AC-31](#ac-31), [AC-32](#ac-32), [AC-34](#ac-34), [AC-43](#ac-43) |
| [IC-58](thermodynamics_semantic_information_dictionary_v0_1.md#ic-58) Provider coordinate and convention binding | P08 | [AC-27](#ac-27), [AC-28](#ac-28) | [AC-16](#ac-16), [AC-27](#ac-27), [AC-29](#ac-29), [AC-30](#ac-30), [AC-32](#ac-32), [AC-33](#ac-33) |
| [IC-59](thermodynamics_semantic_information_dictionary_v0_1.md#ic-59) Check result and evidence evaluation | P09 | [AC-36](#ac-36) | [AC-37](#ac-37), [AC-38](#ac-38), [AC-44](#ac-44), [AC-55](#ac-55), [AC-56](#ac-56) |
| [IC-60](thermodynamics_semantic_information_dictionary_v0_1.md#ic-60) Qualified result and acceptance assessment | P09 | [AC-37](#ac-37) | [AC-38](#ac-38), [AC-40](#ac-40), [AC-44](#ac-44), [AC-49](#ac-49), [AC-50](#ac-50), [AC-53](#ac-53), [AC-55](#ac-55), [AC-56](#ac-56) |
| [IC-61](thermodynamics_semantic_information_dictionary_v0_1.md#ic-61) Coverage and validation claim | P09 | [AC-38](#ac-38) | [AC-09](#ac-09), [AC-38](#ac-38), [AC-53](#ac-53), [AC-56](#ac-56) |
| [IC-62](thermodynamics_semantic_information_dictionary_v0_1.md#ic-62) Result lineage and attribution | P09 | [AC-37](#ac-37) | [AC-50](#ac-50), [AC-51](#ac-51), [AC-53](#ac-53), [AC-56](#ac-56) |
| [IC-63](thermodynamics_semantic_information_dictionary_v0_1.md#ic-63) Operation outcome and diagnostic statement | P09 | [AC-11](#ac-11), [AC-12](#ac-12), [AC-32](#ac-32), [AC-33](#ac-33), [AC-35](#ac-35), [AC-36](#ac-36), [AC-37](#ac-37), [AC-56](#ac-56) | [AC-34](#ac-34), [AC-35](#ac-35), [AC-37](#ac-37), [AC-54](#ac-54), [AC-56](#ac-56) |
| [IC-64](thermodynamics_semantic_information_dictionary_v0_1.md#ic-64) Revision and dependency context | P10 | [AC-45](#ac-45), [AC-48](#ac-48), [AC-51](#ac-51), [AC-52](#ac-52) | [AC-06](#ac-06), [AC-09](#ac-09), [AC-13](#ac-13), [AC-14](#ac-14), [AC-15](#ac-15), [AC-19](#ac-19), [AC-21](#ac-21), [AC-29](#ac-29), [AC-30](#ac-30), [AC-37](#ac-37), [AC-44](#ac-44), [AC-45](#ac-45), [AC-46](#ac-46), [AC-47](#ac-47), [AC-48](#ac-48), [AC-49](#ac-49), [AC-50](#ac-50), [AC-51](#ac-51), [AC-52](#ac-52), [AC-53](#ac-53), [AC-55](#ac-55) |
| [IC-65](thermodynamics_semantic_information_dictionary_v0_1.md#ic-65) Coordinated change and impact proposal | P10 | [AC-47](#ac-47), [AC-48](#ac-48), [AC-52](#ac-52) | [AC-47](#ac-47), [AC-48](#ac-48), [AC-52](#ac-52) |
| [IC-66](thermodynamics_semantic_information_dictionary_v0_1.md#ic-66) Publication proposal and decision | P10 | [AC-46](#ac-46), [AC-49](#ac-49) | [AC-46](#ac-46), [AC-49](#ac-49), [AC-56](#ac-56) |
| [IC-67](thermodynamics_semantic_information_dictionary_v0_1.md#ic-67) Current-result binding | P10 | [AC-48](#ac-48), [AC-49](#ac-49) | [AC-46](#ac-46), [AC-48](#ac-48), [AC-49](#ac-49), [AC-50](#ac-50), [AC-52](#ac-52), [AC-56](#ac-56) |
| [IC-68](thermodynamics_semantic_information_dictionary_v0_1.md#ic-68) Semantic archive and dependency manifest | P10 | [AC-50](#ac-50) | [AC-27](#ac-27), [AC-51](#ac-51), [AC-52](#ac-52), [AC-53](#ac-53) |
| [IC-69](thermodynamics_semantic_information_dictionary_v0_1.md#ic-69) Reconstruction, migration and reproduction record | P10 | [AC-51](#ac-51), [AC-52](#ac-52), [AC-53](#ac-53) | [AC-38](#ac-38), [AC-52](#ac-52), [AC-53](#ac-53), [AC-56](#ac-56) |
| [IC-70](thermodynamics_semantic_information_dictionary_v0_1.md#ic-70) Run authority and cancellation context | P10 | [AC-45](#ac-45), [AC-46](#ac-46), [AC-48](#ac-48) | [AC-29](#ac-29), [AC-30](#ac-30), [AC-32](#ac-32), [AC-34](#ac-34), [AC-35](#ac-35), [AC-44](#ac-44), [AC-45](#ac-45), [AC-46](#ac-46), [AC-48](#ac-48), [AC-49](#ac-49), [AC-50](#ac-50), [AC-55](#ac-55), [AC-56](#ac-56) |
| [IC-71](thermodynamics_semantic_information_dictionary_v0_1.md#ic-71) Physical transfer and exchange realization | P04 | [AC-15](#ac-15), [AC-18](#ac-18), [AC-32](#ac-32), [AC-33](#ac-33), [AC-39](#ac-39), [AC-40](#ac-40), [AC-41](#ac-41), [AC-42](#ac-42) | [AC-15](#ac-15), [AC-18](#ac-18), [AC-36](#ac-36), [AC-39](#ac-39), [AC-40](#ac-40), [AC-41](#ac-41), [AC-42](#ac-42) |
| [IC-72](thermodynamics_semantic_information_dictionary_v0_1.md#ic-72) Numerical policy | P08 | [AC-29](#ac-29), [AC-34](#ac-34) | [AC-25](#ac-25), [AC-29](#ac-29), [AC-32](#ac-32), [AC-34](#ac-34), [AC-47](#ac-47), [AC-53](#ac-53), [AC-54](#ac-54) |

### 10.4 Earlier walkthroughs and test obligations

| Step-5 handoff | Step-7 action workflow |
| --- | --- |
| PW-01 | [WF7-01](#wf7-01) |
| PW-02 | [WF7-03](#wf7-03) |
| PW-03 | [WF7-04](#wf7-04) |
| PW-04 | [WF7-08](#wf7-08) |
| PW-05 | [WF7-06](#wf7-06) |
| PW-06 | [WF7-07](#wf7-07) |
| PW-07 | [WF7-05](#wf7-05) |
| PW-08 | [WF7-10](#wf7-10) |
| PW-09 | [WF7-11](#wf7-11) |
| PW-10 | [WF7-14](#wf7-14) |
| PW-11 | [WF7-15](#wf7-15) |
| PW-12 | [WF7-16](#wf7-16) |
| PW-13 | [WF7-16](#wf7-16) |
| PW-14 | [WF7-17](#wf7-17) |

All 18 A-products remain decomposed into the same B6 concepts and owners. The 108 B6 relationship contracts and 28 SI rules are included unchanged in the register; the action map adds producers/consumers and does not revise cardinality or meaning. The eight original integrated journeys, eight synthetic fixture specifications, 188 requirement witnesses and 68 scenario witnesses retain their original statuses. The B2/B3 probe mappings remain through their unchanged B4 requirement links. None is treated as executed by linking it to an action.

## 11. Open-item disposition and the next evidence gate

| Inherited item / owner | Step-7 contribution | Remaining boundary |
| --- | --- | --- |
| PO-01 / P04 | Correspondence action and ambiguity-dependent routing branches specified (AC-16/40); matching algorithm/thresholds require qualified profiles. | Within-result IDs are local; correspondence supports continuation, appearance/disappearance, split/merge and ambiguity. No universal matching score or empirical threshold is selected. |
| PO-02 / P03 | Convention qualification and material-boundary action sequence specified (AC-08/42); actual transformations/data and numerical fixtures remain open. | A reference transform may be component/state dependent. Reaction energy and chemical standards must be reconciled. Real provider/data pairs still need evidence and numerical tests. |
| PO-03 / P05 | Contribution binding, private callbacks and initializer restoration specified (AC-26/31/43); concrete equations/solvers and derivatives still require qualification. | Local variables, authority, residual units, scaling, derivative axes and initialization restoration are defined without a chosen equation IR or solver. |
| PO-04 / P08 | Session/attempt/run transition rules and cancellation versus native-stop boundary specified (AC-30/35/46); real containment and concurrency remain untested. | Health/isolation/recovery and cancellation authority are explicit. A tested reentrant/serialized/process-isolated strategy must still be selected per build. |
| PO-05 / P10 | Capture, change impact and atomic logical publication guards specified (AC-45/47/48/49); exact dependency algorithm/storage mechanism remains open. | Capture exact relevant dependencies or a conservative superset. Guarded publication and conservative invalidation are specified; no incremental engine chosen. |
| PO-06 / P02 | Assay/empirical preparation and operation-scoped blocking specified (AC-06/09; WF7-11); complete numerical petroleum/black-oil/empirical profiles remain open. | Raw assay, cut definitions, method/data/caloric completion and limited mass-only descriptions are explicit. Actual complete routes are not inferred. |
| PO-07 / P03 | Coupled chemical/phase/transport operations and contacting workflow specified (AC-22/24/43); real model/data compatibility and numerical coverage remain open. | Phase/domain/pair, diffusion frames, caloric/chemical standards and bulk/interface context are identified; real SC-20–26 profile qualification remains required. |
| PO-08 / P09 | Mandatory/conditional/optional checks, additional evidence and scope qualification specified (AC-36/37/55); real-fixture tolerances and independence still require evidence. | Original targets, scales, required/optional status, unknown evidence and independent references have fields. No universal physical accuracy threshold is introduced. |
| PO-09 / P08 | Exact manifest registration, reconstruction and opaque bundle limits specified (AC-27/50/51/53); exact build/data distributions and access facts not refreshed. | Exact bundle/build/binding/data attribution with explicit undisclosed choices is admitted. Public-availability/license judgments are inherited research limits, not refreshed claims. |
| PO-10 / P01 | Concrete site/distribution/UV/restricted action routes supplied (AC-03/18/23; WF7-16/17); independent R review and any E/V remain pending. | Concrete support/weighting, denominator, inventory and restriction meanings now exist. Step 7 must complete actions; no numerical P3 commitment is added. |

The next step is the planned cross-library and solver integration assessment: choose actual supported integration boundaries and compatible profiles, establish concrete initializers, data/reference transformations and derivative contracts, and test real session/failure behavior. The subsequent validation step executes the existing witnesses. No action count or conceptual workflow establishes those facts.

## 12. Audit and completion

The audit checks catalog integrity and inherited ownership/IDs/hashes. A separate standard-library script executes selected authored sequential guard/replay/arithmetic sketches. That script is neither the simulator nor an adapter, and it does not exercise native interruption, concurrent transactions, actual chemical equilibrium or physical property data. It is intentionally conservative about exact context and same-view grouping.

| Artifact | Count |
| --- | --- |
| Action contracts | 56 |
| Positive/negative action witness descriptions | 112 |
| Operation variants | 12 |
| Common action rules | 18 |
| Workflow routes | 18 |
| Lifecycle views | 8 |
| Guarded lifecycle transitions | 67 |
| Handoff facts | 18 |
| Functional requirements preserved | 94 |
| Information concepts preserved | 72 |
| Scenarios/profile assignments preserved | 34 |
| Authored reference checks executed | 26 |
| Simulator/provider acceptance tests executed | 0 |
| Scenario R/E/V promotions | 0 |

**Reference-check result:** PASS. See `thermodynamics_action_workflow_lifecycle_blueprint_v0_1_reference_checks.json` and `reference_action_tests.py`. These selected checks cover target residuals, explicit accounting, publication/edit ordering, same-revision cancellation, ancestor cancellation, group coherence, stale target generation, duplicate decisions, transfer-intent replay, private state and a few initializer/session/archive distinctions.

**Structural completion:** every functional requirement has one primary action with the unchanged Step-5 owner; every concept is consumed and produced/proposed by a defined action; every scenario has a complete specified route; every lifecycle transition names valid actions and endpoints. The audit report supplies actual results rather than a numerical or architectural correctness proof.

**Important limit:** a well-formed transition table is not a verified concurrent implementation. A sequential reference guard cannot prove that a production storage/locking system implements an atomic group update. Native fault containment, numerical conditioning, runtime compatibility, real phase correspondence, source data and actual acceptance tolerances remain profile-specific qualification tasks.

## 13. Source manifest and artifact navigation

| Key | Unchanged source | SHA-256 |
| --- | --- | --- |
| B1 | [thermodynamics_simulation_behavior_scope_v0_1.md](thermodynamics_simulation_behavior_scope_v0_1.md) | `1d12284bbca6569f198caaae86327788bac8fbaf3946625d99d971ef3602d028` |
| B2 | [dwsim_workflow_reverse_engineering_v0_1.md](dwsim_workflow_reverse_engineering_v0_1.md) | `55e77b6fddab4547280e4e40ca5c742fac3fd322e45534767ddae7f5f6ba94ab` |
| B3 | [thermodynamic_package_comparative_research_v0_1.md](thermodynamic_package_comparative_research_v0_1.md) | `c0f9216c450405360b8532771e6c4beca6536154345ea5adf00130a9c4e73875` |
| B4 | [thermodynamics_functional_requirements_v0_1.md](thermodynamics_functional_requirements_v0_1.md) | `7b9c3c4e2110531cb5767fb00f9ba11dec160294b8cafd909c5b04984a87b5c2` |
| B4-register | [thermodynamics_functional_requirements_v0_1_register.json](thermodynamics_functional_requirements_v0_1_register.json) | `1777f2c60962526252aa863dc2cb673f218b4ed58fe9c108f9721264f9fde825` |
| B5 | [thermodynamics_conceptual_packaging_v0_1.md](thermodynamics_conceptual_packaging_v0_1.md) | `85602e0a750add6a1168cb4f4294998feb5e90e35b37cef5f0b4e63c5f1076f1` |
| B5-register | [thermodynamics_conceptual_packaging_v0_1_register.json](thermodynamics_conceptual_packaging_v0_1_register.json) | `d46ce687dee20cb7e677082039f349a38d8fd71101a659079d47bb8d57d80612` |
| B6 | [thermodynamics_semantic_information_dictionary_v0_1.md](thermodynamics_semantic_information_dictionary_v0_1.md) | `58c33ae37009c2c07485a46ae95fdd51f0475504aaa75cb85ef7a23a8484dee9` |
| B6-register | [thermodynamics_semantic_information_dictionary_v0_1_register.json](thermodynamics_semantic_information_dictionary_v0_1_register.json) | `ab459185e0536ce5ef721c572010ed8d1279f89f8d6ad632b32f927b03f21f8d` |

B4 supplies normative requirements and original witnesses; B5 supplies owners and handoff intent; B6 supplies exact information meaning and invariants. B1 remains scope authority. B2/B3 library findings retain their original evidence limitations. The bundle contains unchanged predecessor files so their links remain inspectable; it redistributes no upstream source libraries or binaries.

**Companions:** [thermodynamics_action_workflow_lifecycle_blueprint_v0_1_register.json](thermodynamics_action_workflow_lifecycle_blueprint_v0_1_register.json), [thermodynamics_action_workflow_lifecycle_blueprint_v0_1_audit.json](thermodynamics_action_workflow_lifecycle_blueprint_v0_1_audit.json), [thermodynamics_action_workflow_lifecycle_blueprint_v0_1_reference_checks.json](thermodynamics_action_workflow_lifecycle_blueprint_v0_1_reference_checks.json), [reference_action_tests.py](reference_action_tests.py), [validate_action_blueprint.py](validate_action_blueprint.py).

**Design thesis:** keep the original physical question and its material meaning intact while allowing pragmatic numerical work. Make preparation, calculation, physical transformation, assessment and current publication explicit actions, so breadth does not depend on hidden state changes or one backend’s internal object graph.
