---
title: "Studies, diagnostics and admitted bindings contract review"
date: 2026-10-01
standard: core-3.3/process-simulator-1.3
tier: design
purpose: target
evidence: Implemented
decision: Accept
---

# Studies, diagnostics and admitted bindings

The occurrence-based design is appropriate: one pure policy can govern both executors,
while existing operation owners retain scientific assessment, preparation and execution.
Bindings separate reusable canonical content from experiment occurrences, and failure
envelopes can exist independently of scientific result tables. These distinctions are
substantive improvements over the two former study policies. Bounded reinspection finds the seven
identified defects corrected in source, with the focused execution evidence recorded below.
The target design is **Accept** at the stated **Implemented/Tested** evidence level. This accepts
the reviewed contracts and examined generated Python boundary; it does not certify real database
preservation/recovery journeys or Plan 25k product qualification.

## Scope and baseline

Independent delegated review on 2026-10-01 of the dirty main checkout at HEAD
`6fe8a9df307929f9ae8f4ea3eff0968ae7ccf766`, with concurrent Plan 25f implementation.
The baseline includes the uncommitted ADR-0148 proposal and F1–F5/G3 cutover. This document
records inspected source and subsequent bounded reinspection, rather than a stable commit
or whole-product qualification. Final reinspection used the coordinator-declared frozen dirty
source, including terminal-history enrichment, closed physical observations and completed
generation. Current finding dispositions belong to
[Plan 25f](https://github.com/paul-heyse/pse-arrow/blob/ad665a0222551196b1160e426f5242361215a6a0/docs/plans/25f-studies-diagnostics-and-continuation.md) and the
[series coordinator](https://github.com/paul-heyse/pse-arrow/blob/4c24721e691187e1a5b28398b29722fbde671da8/docs/plans/25-design-remediation.md).

Inspected owners: `workflow/bindings.rs`, compiler member case inputs,
`pse-model::diagnostic` and its source projection interface, `pse-model::study`,
`workflow/study.rs`, `study_operations.rs`, `study_execution.rs`, `study_tables.rs`, durable
worker/terminal integration, `pse-operations::study_policy` and `studies`, schema admission,
V5 transition, frozen Plan 25e declarations, Python workflow/modeling entrypoints and the
boundary-document generator registration. Relevant authorities are blueprint §5.3, §19.2,
§19.3, §20.6, §21.5 and §23.2, proposed ADR-0146/0148 and Plan 25f.

Analysis modes considered are authored solves, simulation, fitting and horizons as study
points. This reviews their composition and boundary contracts, not the numerical algorithms,
unit/property fidelity or reference conformance of those owners. Native execution, Python
round trips, real PostgreSQL/restart/publication journeys, performance, full integration and
static gates were not run by this reviewer. Plan 25k owns the excluded broad qualification.
The follow-up inspected durable pre-effect tickets, dispatch/revision fencing and the receipt
owner's recovery contract, including final horizon destination validation and repeated member
attachment. Final reinspection checked generated documents, JSON schemas, compiled native stub
properties and terminal-history corrections. The coordinator supplied focused receipts read
below; this reviewer ran no tests or database commands.

## Meaning, ownership and composition

| Responsibility | Owner and consumed contract | Consequential boundary |
|---|---|---|
| Physical input admission | Runtime binding admission consumes A's resolved contracts and checked conversion | Submitted path/unit is attribution; admitted member/quantity/canonical coordinate supplies content |
| Occurrence model | `pse-model::study` | Dependencies, seed permission, compatibility, attempt facts and scientific facts are distinct |
| Point decisions | `pse-operations::study_policy` | Plain facts produce actions and a conclusion without database or solver effects |
| Operation execution | Existing solve/simulation/fit/horizon owners, composed by `study_operations` | Source/context and settings reconstruct the existing operation; E supplies aggregate usability |
| Effects and fencing | Durable adapter/store and catalog receipt owner | Reread/recompute under locks, validate revisions and reconcile uncertain effects before retry |
| Detailed diagnostic identity | Source typed error plus model-owned envelope | Detailed code remains authoritative; class, disposition, severity and retry have separate purposes |
| Derived output/boundary | Registry/schema generator and Python adapter | Shared outcome relation and generated documents project facts without reclassification |

The model is adequate for repeated experiments, explicit ordering versus usable-result edges,
one selected seed role, permissive seed-only/fallback choices, cancellation, missing results
and attempt history. In particular, equal `binding_hash` values no longer merge requested
occurrences. `ScientificFacts` takes the run owner's final `usable()` decision; it does not
infer success from result table counts. That preserves partial multi-result distinctions.

The physical contract is complete for admitted overlays: admission resolves the target against
the selected revision, excludes computed members, compares resolved meaning, converts through
`CanonicalConversionPlan`, retains target quantity/context and persists canonical coordinates.
Compiler `case.members` composes those coordinates after authored defaults. Compatible explicit
member renames can preserve binding content while the exact source revision remains a separate
reconstruction/admission requirement. Named identities and incompatible revision reuse remain
distinct. A full physical context identity governs interpretation; no hash proves validity.

| Physical element | Units/basis/reference/convention | Enforcement and remaining limit |
|---|---|---|
| Overlay quantity | Registered complete quantity meaning and representation unit, including affine coordinates | `same_meaning` plus checked conversion; typed mismatch operands retained |
| Admitted coordinate | Target member/quantity in the recorded immutable physical context | Worker validation; no second conversion of overlay attribution |
| Horizon parameter/prior/trajectory input | `BindingQuantity` retains submitted meaning; admitted entries retain member/quantity/canonical coordinates | A's admission is reused; replay resolves the authored destination and checks its full contract |

Well-posedness, formulation guards, derivative source/order, scaling, solver class and independent
post-solve checks remain with the existing prepared operation and E's final assessment. This
cutover introduces no numerical solver. Their complete scientific adequacy was not examined;
the study must preserve those owners' decisions and their diagnostics.

## Revealing scenarios

| Scenario | Required behavior and inspected mechanism | Evidence |
|---|---|---|
| S01: equivalent physical inputs, duplicate aliases and explicit member rename | Normalize units once; conflicting duplicates refuse; semantic member/context content frames identity separately from revision and occurrence | Implemented source; F25F-01 correction inspected |
| S02: replace in-process with durable execution | Identical facts use the same dependency/start/conclusion operation; untrusted descriptors have the same role admission; refusal detail remains typed | Implemented policy and role/refusal corrections; paired execution pending |
| S03: failure before any result and subsequent cancellation/retry | Retain envelope, occurrence and attempts; cancellation remains distinct; unknown effects reconcile before retry | Implemented fenced effect/receipt source; focused native and local policy controls; database recovery not run |
| S04: exact Plan 25e to Plan 25f transition | Preserve jobs/studies/member/history identities and bytes; missing old policy evidence remains unavailable; unknown source/history refuses | Implemented frozen-source and migration code; real database journey not run |
| S05: horizon study input in different units or wrong physical basis | Admit physical assignments at the owner boundary before descriptor persistence and dispatch | Implemented typed admission and destination replay; bare-value codec rejection tested; full horizon journey not run |

The generic dependency graph uses adopted petgraph 0.8.3 cycle detection, with directed
occurrence reachability only. `DiGraphMap` coalesces graph edges for that cycle check while
the authoritative point policy retains ordering/usability/seed edge meanings. This projection
does not reinterpret those relationships for execution. No custom graph algorithm or generic
workflow framework is needed. Pure policy tests can supply semantic facts without native or
database startup.

## Findings and bounded reinspection

### <a id="f25f-01"></a>F25F-01 — Duplicate equality disagreed with binding identity

Initial `bindings.rs` admission compared `FiniteBound` values with IEEE equality, while identity
framed raw bits and blueprint §5.3 preserves signed zero. Two assignments to one target with
`+0.0` and `-0.0` were accepted, retaining whichever came first; reversing the request changed
the admitted content hash. This violates AP-02/AP-05, DP-03/DP-04 and G3/G6 under S01.

Reinspection sees the duplicate comparison use coordinate bits, matching identity for finite
inputs, and the authored test `conflicting_signed_zero_assignments_refuse_in_either_order`.
The correction is **Implemented/Tested** by the native focused receipt below, including that
test and affine/equivalent-unit, basis/datum/subject and compatible-rename controls. A's canonical
unit path preserves signed zero without an unnecessary multiply/add. Equal duplicates remain
permitted; opposite signed zero refuses in either order.

### <a id="f25f-02"></a>F25F-02 — Public durable definitions bypassed seed-role admission

Initial `start_defined_study` checked graph, binding/source identities and prepared seed need
but omitted the producer/consumer role checks used by request admission and the in-process
adapter. Public Python deserialization could therefore supply a `Trajectory` role to an
algebraic point; worker acquisition selected a primal solution and relabeled the seed fact with
the unchecked requested role. This violates AP-02/AP-04/AP-05, DP-03/DP-07 and G2/G3 under S02.

Reinspection sees `StudyDefinition::validate_roles` centralize graph and producer/consumer role
validation and `start_defined_study` call it before scheduling. The correction is **Implemented**.
The durable worker compares its point and source bundles with the immutable stored definition
before reconstruction. Focused operation-role and request/job codec tests passed; real database
prequeue rejection remains unexecuted. Adding a legitimate future seed role changes its operation
owner, not independent role interpretation in adapters.

### <a id="f25f-03"></a>F25F-03 — Durable unstarted refusals lost detailed policy meaning

Initial `pse-operations::studies::apply_unstarted` replaced typed refusals with generic
`StudyPolicyAdmission` and debug text. The in-process projection retained specific rules and
structured predecessor/role facts. The same dependency failure thus changed diagnostic meaning
when the executor changed, violating AP-04, DP-01/DP-21 and G1/G2 under S02/S03.

Reinspection sees one `Refusal::boundary_diagnostic` in `pse-model::study`, consumed by both
adapters through the runtime projection and directly by durable `apply_unstarted`. The correction
is **Implemented**. This removes adapter-specific reclassification: detailed code/rule and
structured predecessor/role observations derive from the same typed refusal. Native occurrence
and pure policy controls passed; an exhaustive paired database projection journey was not run.

### <a id="f25f-04"></a>F25F-04 — Known source revision was absent from retained failure locations

Initial compiler/modeling projection supplied source locations with `revision: None`, and the
source-bound study captured these without enriching them despite knowing the exact selected
operation revision. The unused `with_revision` helper only handled top-level locations. Stored
preparation failures and nested causes therefore could not identify the immutable interpretation
of their authored location. This violates DP-21/G2 under S03.

Follow-up sees recursive enrichment preserving attributed revisions, in-process preparation,
result and error capture, and durable terminal cause capture use the operation revision. This
is **Implemented**. Focused diagnostics and relation-projection controls passed, including full
operands, revision and ordered nested causes; they do not certify unexamined source-error
families. Unknown locations remain absent rather than fabricated.

### <a id="f25f-05"></a>F25F-05 — Horizon descriptors expose untyped physical assignments

Initially, `HorizonInputDocument.initial`, `ArrivalDocument.initial` and the remote
`HorizonSignal::Trajectory(Vec<f64>)` projection accept bare canonical values through the new
public generated study request. `StudyOperation::prepare` projects them directly into the existing
horizon owner. Plant input admission checks parameter membership, duplicate/schedule conflicts
and finiteness; it has no supplied quantity/unit/context to compare or convert. A caller can
submit a temperature number in a pressure input, or Celsius as Kelvin, without an attributable
physical refusal. Pinning the source revision makes the target known but does not establish the
meaning of the supplied value. This violates AP-02/AP-04/AP-05, DP-02/DP-03, PS-01 and G2/G3/PS-G1
under S05 and the explicit F2 requirement for public parameter bindings.

The selected correction is a submitted-to-admitted horizon binding boundary using A's existing
physical contract/conversion owner. It consumes target selection and `BindingQuantity`-like
submitted meaning; it produces checked target member/quantity and canonical values under the
recorded context. Persist that product and validate reconstruction without converting attribution
again. Keep numerical horizons, estimators/controllers and E's usability in their existing owners.
Apply the rule to explicit trajectories and initial priors as well as held plant parameters.
A legitimate canonical-unit input remains accepted; wrong basis/datum/subject refuses, and an
affine or equivalent-unit input produces the same admitted physical content. The alternative is
a genuinely narrower supported public operation contract, which must be declared in the plan and
boundary and cannot coexist with an unchanged claim of complete F5 horizon support.

Follow-up implements typed raw quantities and `AdmittedHorizonValues` through A's shared
admission helper. The seven-test runtime codec receipt rejects bare parameter/prior/trajectory
input. A second inspection found replay checking entries against their own member, which would
permit a forged admitted definition to substitute a different valid physical member. Final
reinspection sees `target_member` resolve the authored target/prior through compiler-demanded
paths, then `validate_entry` compare destination identity and full resolved contract. Plant
inputs compare against their authored parameter. Cardinality and extra-entry checks prevent
silent truncation or unused assignments. This correction is **Implemented**, with shared
physical admission and contextual horizon reconstruction controls **Tested**; complete closed-loop
study execution remains unexecuted.

### <a id="f25f-06"></a>F25F-06 — Repeated recovery cannot attach its exact inventory

Initially, `Runtime::reconcile_study_receipts` persisted a nonempty partial native inventory with effect
`Unknown`. Its next recovery, whether another partial observation or the exact complete inventory,
passes the same already-attached members to `reconcile_receipt` and `insert_members` again.
`insert_point_member` is a plain INSERT under point/name uniqueness. The duplicate failure rolls
back reconciliation, preventing partial recovery from reaching `Idempotent`. This violates
AP-02/AP-05, DP-19 and G5 under S03/S04.

The correction gives member attachment an explicit idempotent contract: repeat the
exact descriptor/version safely, refuse conflicting receipt identity, and define replacement of
an earlier attempt's members when a safe retry produces the same point names. Preserve exact
ticket inventory and the distinction between partial effects and complete idempotent effects.
A partial→same partial→complete sequence and repeated complete inventory must settle; changed
URI/version follows the explicit replacement or conflict rule. Database execution remains in
Plan 25k, while local comparison policy can be tested independently.

Final reinspection sees exact whole-descriptor equality skip an existing attachment and reject
conflicting URI/version/selection/contract identity. Stored `member_attempt` attributes the
inventory; another attempt may supersede it only when the retained original attempt's effect is
`Absent` or `Idempotent`. Recovery updates that original effect and performs member attachment
before storing the revised point in the same locked transaction. Thus partial→same partial→complete
attachment adds missing members without duplicate insertion; safe retries replace prior inventory
as a unit. The correction is **Implemented/Tested** for exact comparison and attempt replacement
policy in the final operations receipt below. Transactional crash/restart execution remains unqualified.

### <a id="f25f-07"></a>F25F-07 — Early and stale terminal failures lost retained cause/history

The coordinator's final pass found failure before `PointContext` creation could leave a typed
termination cause outside the point envelope. Stale/superseded attempts could also omit an
operational cause or discard later diagnostic enrichment for an already retained attempt. This
violates the failure-history contract under S03 and DP-21/G2, without implying usable results.

Final independent inspection sees `point_terminal_diagnostic` retain the exact source envelope
and actual terminal attempt/lifecycle before point context exists, with no start or scientific
permission. `stale_attempt_outcome` attributes the operational event to the actual row and keeps
any prior typed cause primary. `retain_attempt` enriches one row per attempt, preserves its
scientific/start/effect facts and suppresses repeated retained causes; actual row lifecycle remains
authoritative. The correction is **Implemented/Tested** by pre-context and repeated-callback
controls in the 18-test operations receipt. Database callback/restart ordering remains unqualified.

The final public boundary is **Implemented**: `PhysicalObservation` is a named closed payload
with finite magnitude, quantity, canonical unit and immutable context; its schema has
`additionalProperties: false`, and the generated Python projection retains the same fields.
Rust serialization/deserialization refuses ordinary nonfinite observations; explicit nonfinite
tags remain distinct. `StudyRequest`, admitted definitions, raw typed horizon quantities and
closed operation/signal variants now exist in generated schemas and Python documents. The
compiled native diagnostic stub exposes `envelope` as a property; Python consumes that structured
envelope without replacing detailed codes or cause order. No handwritten generated compatibility
surface was introduced.

## Focused execution evidence and receipt contract

The coordinator and assigned executors executed these checks. This reviewer read the named
filesystem logs; the last four test rows are executor-reported tool-transcript receipts, with
their source controls inspected independently, rather than independently read execution logs.
Rust correctness recipes use explicit `pse-relations/force-validate`; the zero failure target
remains the baseline. Filtered tests are excluded scope. These are focused receipts, not product
qualification.

| Command and conditions | Result and log |
|---|---|
| `direnv exec . just unit-native-package pse-runtime native-solvers 'test(occurrence_execution_tests) \| test(binding_admission_unit) \| test(diagnostic_rows) \| test(workflow::diagnostics) \| test(study_operation_unit) \| test(study_request_codec_unit)'`; linked native runner | **Tested**, final 35/35 passed, 333 filtered, zero final failures; `/tmp/25f-final-native-controls.log`. Initial run had 25/33 pass and eight failures subsequently corrected; composite receipt |
| `direnv exec . just unit-package pse-operations 'test(study_policy_unit) \| test(study_codec_unit)'` | **Tested**, final 18/18, 112 filtered, zero failures; `/tmp/25f-terminal-history-tests.log`, including exact attachment, safe supersession and terminal enrichment controls; earlier 15/15 and 16/16 receipts preceded final corrections |
| `direnv exec . just unit-native-package pse-runtime native-solvers 'test(study_operation_admission_tests)'` | **Tested**, 4/4, 364 filtered, zero failures; `/tmp/25f-general-operation-controls.log`; simulation/fit owner reconstruction and horizon unit/context/destination controls |
| `direnv exec . just unit-native-package pse-runtime native-solvers 'test(initialization_restores_original_specification) \| test(kernel_initialization_retries_typed_preparation_rejections) \| test(kernel_initialization_shrinks_failed_native_nonlinear_steps) \| test(kernel_initialization_retries_callback_trials_without_event_history) \| test(derived_big_m_follows_value_only_bindings) \| test(pounce_convex_batched_study)'` | **Tested**, 6/6, 358 filtered, zero failures; `/tmp/25f-native-owner-controls.log`; bounded existing-owner regression controls |
| `just unit-package pse-runtime 'test(study_operation_unit) \| test(study_job_v5_codec_unit) \| test(termination_detail_v2_codec_unit)'` | **Tested**, 7/7, 215 filtered, zero failures; `/tmp/25f-runtime-codec-tests.log` |
| `just unit-package pse-catalog 'test(member_ticket_unit)'` | **Tested**, 1/1, 89 filtered, zero failures; `/tmp/25f-member-ticket-tests.log`; exact-inventory admission before I/O only |
| `just unit-package pse-operations 'test(migration_frozen_source_identity_and_history_ownership) \| test(migration_plan25e_frozen_source_identity_and_follow_on_target) \| test(migration_source_and_history_prefix_are_admitted_together) \| test(migration_follow_on_preserves_enum_order_and_complete_layout_signatures)'` | **Tested**, 4/4, 121 filtered, zero failures; `/tmp/25f-migration-unit.log`; frozen declaration/history controls without a database |
| `direnv exec . just codegen` | **Implemented**, all six schema targets generated successfully and contract annotation checks passed; `/tmp/25f-final-codegen.log`; establishes generated artifacts, not a full static gate |
| `direnv exec . just py-unit-native python/pse/tests/test_studies.py -q`; linked editable native extension | **Tested**, executor reports 6 passed, 3 deselected, zero failures, 2.96 s; tool session 41927. Covers occurrence identity, missing-result refusal, physical envelope/closed codec, typed tuples and raw-policy authority; database/cancellation/publication/sweep journeys not run |
| `direnv exec . just py-unit-native python/pse/tests/test_native_boundary_contracts.py::test_generated_settings_admit_native_defaults_and_expose_named_diagnostics python/pse/tests/test_native_workflow.py::test_compiler_failure_retains_typed_authored_source_span -q` | **Tested**, executor reports 3 passed, zero failures, 1.41 s; tool session 9079; supported native diagnostic adapter path |
| `direnv exec . just unit-package pse-codegen 'test(codegen::documents::tests::)'` | **Tested**, executor reports 6/6, 55 excluded, zero failures; Nextest `3153d1aa-832a-4ded-b123-b2987c6e0466`; closed documents, recursive references and typed heterogeneous tuples |
| `direnv exec . bash scripts/memory-cap.sh just unit-package pse-backend-native 'test(dynamics_profile_duration_schema_matches_closed_serde_representation)'` | **Tested**, executor reports 1/1, 140 excluded, zero failures; Nextest `44493c80-c391-49fb-9e69-9593f9dd28ce`; closed duration serde/schema correspondence and malformed/overflow refusal |

Receipt safety is **Implemented** at inspected source level: ArtifactPlan mints the exact ticket
before effects; `record_receipt` checks Running/current attempt/worker, dispatch provenance and
expected revision under job/study locks, commits ticket plus `Unknown`, then native member I/O
begins. Lease-loss callback preserves `Unknown` for a dispatched writer. Reconciliation's no-ticket
`Absent` path rechecks expected revision, ticket absence and non-Running state under the same
locked ownership. A writer that has not recorded its ticket cannot pass that closed dispatch fence.
The runtime does not grant absence or retry from missing, provisioned or partial native receipts.
Only the whole exact ticket inventory can become `Idempotent`; the native owner compares full
member requests and native application transactions. This addresses the ghost-writer absence
problem in the mechanism. Repeated attachment follows the exact comparison and attributed
supersession rule above. No real database/crash/publication journey establishes the complete
behavioral recovery claim yet; the source contract and local policy evidence support design
fitness without making that stronger claim.

## Foundation and gate judgments

| Foundation | Verdict at inspected checkpoint | Reason |
|---|---|---|
| AP-01 | Satisfied | Pure policy, operation execution, physical admission and storage effects have coherent separate responsibilities |
| AP-02 | Satisfied at inspected contract level | Occurrence, scientific permission, physical assignments and effect receipts retain distinct typed meaning |
| AP-03 | Satisfied for operation composition | Existing owners execute mixed points; no second simulation/fitting/horizon engine |
| AP-04 | Satisfied | Occurrence and admitted physical models govern the supported adapters, including horizon destinations |
| AP-05 | Satisfied at inspected contract level | Admission/replay reject invalid roles, target/context meaning and unsafe effects; local comparison controls passed |
| AP-06 | Satisfied for policy/admission separation | Pure fact transition and local physical admission need no database/native execution; operational integration remains necessary |

| Gate | Judgment | Evidence/limit |
|---|---|---|
| G1 | Pass | One refusal projection, one policy and existing operation/receipt authorities |
| G2 | Pass at inspected source level | Typed horizon admission, destination replay and recursive source attribution preserve meaning |
| G3 | Pass at inspected contract level | Shared role/physical admission and immutable descriptor validation; native and codec controls passed |
| G4 | Pass for pure transition | Plain explicit inputs; effects remain in adapters |
| G5 | Pass at inspected contract/local-policy level | Pre-effect fenced ticket, conservative unknown effects and exact repeated attachment; actual database recovery remains unqualified |
| G6 | Pass | Full physical-context identity and canonical member projection; equivalent-unit, signed-zero and rename controls passed |
| G7 | Pass for stated claims | Focused receipts and generated-boundary checks are distinguished from database journeys and Plan 25k qualification |
| G8 | Pass in bounded scope | Adopted petgraph, serde/schemars, checked quantity conversion and existing operation/storage owners reused |
| G9 | Pass at stated evidence level | Each affected foundation is satisfied; no averaging or MUST waiver |
| PS-G1 | Pass at inspected contract level | Overlay and horizon assignments reuse A's complete quantity/context and checked conversion owner |
| PS-G2 | Not applicable to new numerical mechanisms | Existing well-posedness owners are composed; their whole scientific qualification is excluded |
| PS-G3 | Pass for study composition | E's final usability and source assessment remain authoritative; focused occurrence/cancellation/missing-result controls passed, whole scientific qualification excluded |

## Alternatives, preservation and decision

The simplest viable design coincides with the inspected design: ordinary pure policy functions
over explicit facts, operation-owned
preparations, one shared outcome projection and existing executor effect adapters. Keeping two
policies cannot guarantee substitution; a general workflow engine adds unrelated ownership.
Generic graph/canonicalization/serialization/conversion machinery remains library-owned. Bespoke
code is justified for scientific permission and occurrence semantics, which those libraries do not
define. No library replacement or new abstraction is required to correct the findings.

The G3 migration follows the preserving route in proposed ADR-0146: exact frozen Plan 25e source,
distinct fresh/upgraded provenance and committed checksum/history admission, quiescent namespace
ownership, explicit target readiness and V5. The transition removes binding-content uniqueness
while preserving occurrence keys, jobs/studies, source definitions, payloads and member references.
Legacy predecessor/content attribution is carried as `legacy_unavailable`, and store readers do
not fabricate modern scientific permission. Static inspection supports this design; the authored
real database preservation/resume controls are **not_run** here. It does not prove PostgreSQL
constraint, enum ordering, crash recovery or publication behavior. Those limits do not require
whole-product qualification before assessing the local architecture.

Deletion obligations remain part of functional integration: remove replaced study types, old
path-keyed bare-number overlay shapes, their obsolete callers/tests/fixtures and the duplicate
outcome relation; regenerate boundary documents through their owners. Initial old study/overlay
test consumers were migrated during the review. The scoped final source search found no
`ModelingStudyPoint`, `ModelingStudyReport`, `PointOverlay(values=...)` or `modeling_studies` references
in runtime workflow and Python package/tests. Final generation publishes `StudyRequest` and the
examined typed documents through their owners. No compatibility path is justified by this review.

Behavioral/semantic adequacy: **Accept** for the reviewed study and physical binding contracts.
Architectural fitness: **Accept** at the stated **Implemented/Tested** evidence level. All seven
findings have inspected corrections; no material contract defect remains in this bounded
reinspection. This accepts the target design and examined implementation contracts, while
real preserving migration/recovery and full product qualification retain their stated unqualified
status. Acceptance does not narrow the supported operation claims to conceal a MUST gap, and
does not close Plan 25f or Plan 25k.

ADR-0148 remains proposed. Its hashing/Python-boundary/operational decision requires the decision
PR. Routed amendments to blueprint §5.3, §19.3, §20.6, §21.5 and §23.2 are recorded by revision 93.
This review alone neither accepts an ADR nor closes Plan 25f. No MUST exception is claimed.
No SHOULD exception is needed, so the template's `Accept scoped` verdict is not used.
The plan owns current disposition and execution evidence for all seven stable finding IDs.
