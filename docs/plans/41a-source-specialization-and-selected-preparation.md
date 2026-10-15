---
title: Source specialization and selected preparation
status: done
date: 2026-10-15
adrs: []
review_sources:
  - docs/design_review/reviews/design_review_simulation-pipeline-efficiency-and-effectiveness_2026-10-15.md
  - docs/design_review/evidence/simulation-pipeline-efficiency-and-effectiveness-2026-10-15/preparation-and-lifetimes.md
scenario_sources:
  - docs/design_review/reviews/design_review_simulation-pipeline-efficiency-and-effectiveness_2026-10-15.md
doc_owner: docs/plans/41-simulation-pipeline-efficiency-and-effectiveness.md
doc_topics: [design, preparation, validation]
---

# Plan 41a: Source specialization and selected preparation

## Purpose and boundary

Remove demonstrated repeated source, specialization and selected-plan work without changing the mathematics or lexical authority. This companion of [Plan 41](41-simulation-pipeline-efficiency-and-effectiveness.md) owns the packages contributing to review [F01](../design_review/reviews/design_review_simulation-pipeline-efficiency-and-effectiveness_2026-10-15.md#f01)–[F05](../design_review/reviews/design_review_simulation-pipeline-efficiency-and-effectiveness_2026-10-15.md#f05). The coordinator owns their dispositions. Supporting [preparation and lifetime evidence](../design_review/evidence/simulation-pipeline-efficiency-and-effectiveness-2026-10-15/preparation-and-lifetimes.md) supplies the detailed causal premises and adversarial cases.

The target supports interactive value edits, studies, dynamics and fitting through the same admitted preparation owners. Adding a unit or property must not require another specialization or selected-assembly algorithm. This is a standalone new scope: it neither changes existing plan progress nor declares earlier incomplete qualification complete. Blueprint §7/§14 own mathematical and preparation contracts; §19/§21 own workflow and Python consumption.

**Interface-checked baseline:** focused reads on 2026-10-15 of current source over `a08f5fa091db1840fc95b46c4d4f849ce4c5de62` plus the dirty working tree corroborate the source mechanisms below. **Proposed:** all target changes and future acceptance. No product tests, builds or benefit measurements were performed for authoring this companion. The selected review standard is Core 3.4, Efficient Architecture 1.0 and Process Simulator 1.5 through the pse-arrow binding.

## Foundations and target contracts

### Indexed import lookup — F01

`workflow/modeling/canonical.rs::SourceIndex::insert` already maintains `named_roots`; `LexicalLookup::imported` still scans all selected rows after checking namespace, import membership and root-name completeness. Use the existing root bucket at that point. Preserve every candidate, import-group order and multiplicity, and the declaration-ID ordering previously supplied by the ordered row map. Acquisition order is not that order. Maintain sorted buckets or order the requested bucket; do not globally sort/deduplicate atomic import answers.

Pending is different from proved absence. Missing nearer import membership or an incomplete root bucket must request the same missing premise and prevent outer fallback. The shared resolver continues to govern formals, imports, ordinary members, guarded/inherited members and later-segment traversal. All its consumers—local source selection, ingress/delta closure and replay validation—move together; no fast alternate resolver is introduced.

Inherited/guarded member expansion is a further question raised by the supporting inquiry, not evidence that an owner/name cache is sound. A0 inspects whether repeated expansion remains material after indexed roots. A finalized immutable selection or a complete dependency premise may justify reuse; growing admission, recursive probing and temporary absence prohibit premature memoization. If no suitable premise or actual repeated demand is established, retain current member expansion and record that disposition here, without claiming this optional cache is required for F01 closure.

### Immutable lexical base with current overlays — F02/F03

`CheckedPackage::named_types_with` already uses owning/import prefixes and `names_below`; the correction must not relabel it a whole-repository scan. `Evaluator::typed_result`, constructor argument evaluation and `Engine::source_types` repeatedly rebuild visible context during one immutable specialization. Retain the lexical-owner base for the lifetime of the borrowed immutable checked package, then resolve through current reader/admission extent, type/scope parameters, actual function arguments, current values, optional/missing/present refinements and evolving finite function types.

The base does not belong in a permanent owner-only cache on mutable `CheckedPackage`. Admission's preceding type population remains live. An immutable specialization may reuse a base only while its package/context premises are fixed; overlays are occurrence-scoped and refreshed when their actual meanings change. Hoist constructor target/base lookup before the argument loop, while keeping each argument's evaluation and checking.

Expression admission consumes that original package and its current function-contract overlay through one visibility/lookup authority. `SpecializedModel::function_contracts` currently clones the whole checked package to insert finite functions and lowered-function membership. Replace that materialization with a borrowed admission view or equivalently narrow overlay; unchanged declarations, names, children, types and provenance stay borrowed. Functions generated after the first admission must be visible to subsequent requests. Reconstruction-specific result-contract transformations stay local to that reconstructed body and cannot leak into other calls.

A0 settles the common admission-view boundary before A2/A3 migrate their consumers. A locally retained base is viable; a borrowed lookup view avoids repeated broad map copies where inference APIs can consume it. If the first coherent delivery still copies a full stable map per inference, name that residual and its required removal before F02 closure. Renaming package clones or keeping a stale augmented package does not close F03. No second synthetic-name registry, general-purpose cache service or new crate is selected.

### Selected assembly and projection lookup — F04/F05

`CompilerWorkspace::prepare_modeling_functions` builds a full Value `CasePlan` before filtering selected rows and coordinates. Assemble selected functions directly from admitted case/body inventory through the existing assembly authority. Reuse a full plan only when another actual consumer already needs it; do not construct or retain broad work merely to conceal selection loss. Carry the immutable selected observation structure from program-key lookup into construction on a miss, rather than rebuilding it. Its identity includes the complete admitted/physical context, actual row and coordinate request/order, supplied variable view and profile; row IDs alone are insufficient.

Preserve complete selected contribution sums, requested coordinate order, existing row order, duplicate/unknown refusal, bounds, native forms, guards, provider demand and original-space checks across all supported derivative orders. An opaque coupled provider may still need broader evaluation under its supplier contract. Selection is not authority to drop that dependency. Migrate shared fitting experiment/original-check, dynamic function, conditional strategy, case and structural partition consumers together with the relevant replacement slice.

`grouped::plan` already trims transitive local-member demand. Preserve that strength. Build its projection-level runtime/formal map once and borrow it across normalized outputs. Per-expression Let state, fresh renaming, dependency-order numbering, cycle checks, quantity hints, validity-only dependencies and checked-member attribution remain consumer-specific. A shared global renamer would merge identities and provenance unsoundly.

## Packages and dependencies

| Packet | Prerequisite | Delivery and acceptance | State |
| --- | --- | --- | --- |
| A0 | Review/source obligations; coordinator shared-decision route | Settle immutable lexical-base lifetime, evolving function admission view and selected-assembly ownership from actual callers. Decide the bounded inherited-member reuse question above. Name affected APIs, invalidation premises and revealing controls; route any newly discovered rule impact before reliance. | Completed — immutable base/view and direct selection choices settled; inherited-member reuse rejected |
| A1 | Existing SourceIndex completeness/ordering contract; no dependency on all A0 | Replace imported-root row scans with indexed lookup. Move all shared resolver callers; remove the scan implementation and obsolete scan-work tests. Compare ordered answers under reverse acquisition, duplicate/guarded roots, absent/Pending and pending-to-found transitions; show unrelated rows are not examined. | Completed — ordered indexed lookup and actual candidate-count controls passed |
| A2 | A0 immutable base/current-overlay contract | Retain stable lexical context within specialization, hoist constructor setup and migrate arithmetic/source-type inference. Preserve live mutable admission. Exercise same owner with different arguments/refinements, optional-presence arithmetic and multiargument constructors; count base construction versus overlay use. | Completed — immutable bases/current borrowed scopes; 238 modeling controls passed |
| A3 | A0 admission-view contract; A2 view slice if shared | Replace complete CheckedPackage function augmentation with current narrow overlays across rewrite, contextual/contribution/function admission and reconstruction. Test generated-after-first-use lookup and reconstruction override isolation; remove whole-package cloning for this purpose and its old callers/fixtures. | Completed — current borrowed function authority and local reconstruction; all consumers migrated |
| A4 | A0 selected-assembly contract; existing assembly equivalence controls | Deliver direct selected functions and carry selected observation structure across cache lookup/miss. Migrate actual fitting, dynamics, conditional/case and partition consumers in coherent slices. Compare selected results to independent aggregate/coordinate expectations and current assembly; remove broad-first/reconstruction paths when each slice passes. | Completed — direct selected assembly/carried observation structure and native consumers passed |
| A5 | Existing grouped-plan demand contract; A0 only if interface is shared | Build one projection lookup and borrow it per consumer. Preserve consumer-local normalization and transitive trimming. Test overlapping distinct closures, validity-only inputs, local ordering and equal normalized bodies with distinct instances; remove per-output projection-map construction. | Completed — shared projection lookup; independent locals and validity-only controls passed |
| A6 | A1–A5 implemented for all actual consumers | Finish targeted source/compiler/runtime consumer acceptance, confirm replacement deletion and independent test dependencies, and hand evidence/limits to 41f. Findings remain scheduled until their coordinator-owned completion obligations are met. | Completed — all actual consumers migrated; targeted native/Python acceptance and handoff complete |

A1 and A5 can progress independently of the lexical/function seam. A4 can progress once its own assembly contract is settled; it does not wait for all type-overlay work. A2/A3 share editing surfaces in checker/specializer APIs and need one integrated admission-view decision even when execution is delegated. A6 is a migration audit, not a place to defer required consumers.

[41f](41f-causal-diagnostics-qualification-and-closeout.md) F0 supplies only the needed diagnostic meaning slice; its F1 producer integration participates in A1–A5. Emit bounded actual lookup/candidate, base/overlay, package-copy, selected/prepared-row and projection-index facts at the performing owner. Request counts do not substitute for construction counts. Instrumentation must neither re-run inference nor affect scientific/cache identity, and is delivered with the change it explains. Existing diagnostic meanings can be consumed immediately where sufficient.

[41b](41b-cancellation-shared-work-and-admission.md) B1 is required before long reruns of cold preparation. Local deterministic A tests need not wait for cancellation work that they do not consume. B2/B3 must be delivered before A6 claims concurrent sharing/admission behavior; those behaviors are not closed by a faster single request.

## Verification and handoff

**Proposed targeted verification:** use `just check-package` for touched Rust owners and `just unit-package` for meaningful owner controls, retaining explicit force-validation for actual Arrow consumers. Pure checker/specializer tests use source rows, physical/type context and cancellation callbacks without starting a canonical store or solver. Preserve genuine acquisition integration coverage at scope end rather than turning local inference tests into storage journeys.

Retain `named_types_prefix_enumeration_reads_current_types_without_a_retained_cache`, prefix collision/import/physical shadowing and lowered-function resolution controls. Use selected-plan equivalence controls plus independently authored row sums and coordinate permutations to avoid reproducing a common faulty transformation. Retain grouped demand controls including `projected_outputs_bind_only_the_members_they_read`. Add negatives for excluded unrelated guards, missing required guards, unsupported/unknown rows and stale current overlays.

Review S01/S07/S10 are the principal scenario routes. Exercise value edits against fresh preparation and genuine structure/import/physical changes against correct invalidation; value changes must not rebuild structure merely because a context object was replaced. Construction/work observations establish removed repetition under stated controls; they do not establish an elapsed speedup or full-product scientific adequacy.

41f owns assembled scope-end journeys and static checks after functional implementation. No statistical benchmark, hard latency gate or environment/provenance proof is required. Completion here requires actual consumer migration and deletion, recorded targeted results with commands and zero-failure baseline, and explicit investigation dispositions. The Outcome below owns delivered scope and evidence. Enduring preparation contracts have moved to blueprint §14.3; this companion remains consumed by the active series. Existing older plan documents retain their own scope and status.

## Execution checkpoint — 2026-10-15

The operator authorized A0–A6, the B0/B1–B3 preparation-lifetime enablers and the
A/B-specific F0/F1 observations. A0 selects operation-owned immutable lexical bases
with narrow type scopes, a borrowed current-function admission view, and direct
selected assembly with on-demand complete body lookup. Inherited-member caching
is not selected: mutable acquisition and recursive temporary absence lack a sound
complete reusable premise. B0 selects the existing 600-second preparation default
at MathPolicy, distinct original caller/producer clock origins, and exclusive
compiler authority transferred from bounded population admission through actual join.

A0–A6 and the B0/B1–B3 enablers are complete at their authorized scope. Source,
compiler, modeling, default/native runtime and benchmark consumers compile. Targeted
ownership/count controls and selected fitting, dynamics, conditional-solve and local-edit
journeys and the compiled Python setting acceptance have passed.
The inherited-member inquiry selected no cache because mutable acquisition and recursive
absence lack a complete reusable premise. No worktree or service was created. Full 41f
qualification and older-plan reconciliation remain separate.

## Outcome — 2026-10-15

**Implemented:** ordered indexed imports; operation-owned immutable evaluation/source
lexical bases with occurrence-local borrowed type frames; one borrowed current finite-function
admission view with local reconstruction interpretation; direct selected function assembly;
one carried observation structure per key request; one projection lookup with independent
consumer normalization. The complete-package augmentation clone, stable type-map copies
for scoped inference, broad-first selected function plans and miss-time observation
reconstruction are removed. Ordinary broad plans remain only for consumers that require them.
The K4 demand/worker-lifetime benchmark callers now await actual admitted source preparation
and pass the carried observation structure. No compatibility path or new crate was introduced.

The base's immutable package lifetime is explicit; mutable admission still reads fresh
current types. Evaluation/source bases remain distinct where parameter/formal precedence
differs. Selected assembly still examines admitted contribution inventory to identify its
required bodies. The delivery removes broad product construction without claiming constant-time
selection. Inherited-member expansion remains uncached by A0's reasoned no-adoption decision.

### Verification and limits

**Tested — zero-failure target met for each accepted selection:** commands below used the
pinned development profile and `PSE_TEST_RESOURCE_STATE=/tmp/pse-friction-review-resources-uMrtOxGw`,
`CARGO_BUILD_JOBS=2`, through `scripts/pse-env --`. Scope-aware Rust routes selected explicit
Arrow force-validation where consumed. Overlapping runs are not summed into a qualification total.

| Command/selection | Accepted scope/result |
| --- | --- |
| `just check-package pse-modeling`, `just check-package pse-math`, `just check-package pse-compiler` | Source/math/compiler compile; final compiler check warning-free. |
| `just unit-package pse-modeling 'all()'` | Final 238 library unit tests passed, zero failures/skips, including final temporal/definition-environment consumers. |
| `just unit-package pse-math selected_functions` | Three controls passed: independent contribution sums, complete universe/requested coordinates, guards, fixed variables and parameters. |
| `just unit-package pse-math value_function_incidence` | One existing incidence control passed. |
| Compiler selection below | Seven distinct controls accepted; affected fixture corrections and stronger validity-only input rerun passed. |
| `just check-package pse-runtime -p pse-py -p pse-benches` | Final default-feature all-target check passed without warnings. |
| `scripts/pse-env --native=solver,klu,isolation,uno,petsc -- just check-package pse-benches --features native-process` | Native benchmark callers compile, zero warnings; benchmarks were not executed. |
| Native runtime selection below | Eight of nine passed initially; the observation fixture's explicit equation correction then passed its single affected rerun. All nine selected controls accepted. |
| `just codegen` | Full declared generation passed, including contracts, canonical schema, full five-target generation and Ipopt bindings. The native correctness extension and its actual compiled API stub regeneration also passed; four Python setting/forwarding controls passed at 41b. |

The compiler command was `just unit-package pse-compiler` with this exact filter:

```text
test(selected_preparation_uses) | test(selected_shared_observation_body) | test(overlapping_consumer_locals) | test(demanded_member_ranges) | test(projected_outputs_bind_only) | test(actual_numerical_facts_reach_conditional_boundaries) | test(declared_hybrid_event_guards)
```

The native command used `scripts/pse-env --native=solver,klu,isolation,uno,petsc -- just
unit-package pse-runtime` with `--features native-solvers --execution-effects native-local`
and this filter:

```text
test(dynamic_snapshot_pressure_evicts_idle_lane) | test(selected_observation_cache_carries) | test(canonical_source_checkpoint_preserves) | test(producer_preparation_expiry_is_not) | test(local_edits_preserve_unchanged_products_parent_and_true_noops) | test(ordinary_preparation_basis_separates_actual_instance_and_complete_bindings_and_limits) | test(gauss_newton_fit_admits_transient) | test(kernel_integrated_sample_checks_share_model_semantics_and_owned_rows) | test(conditional_unit_solves_original_rows_and_restores_each_boundary_overlay)
```

Its Nextest run was `e7208e0b-eedb-48fb-a18f-807b0b6a05e1`; the final affected
`just unit-package pse-runtime selected_observation_cache_carries --features native-solvers
--execution-effects native-local` rerun passed one test, Nextest
`d5c8ae05-2b7b-4d71-96c8-78c600542b93`. The original 30-control composed A/B run and
its corrected pressure fixture are accounted for in [41b's Outcome](41b-cancellation-shared-work-and-admission.md#outcome--2026-10-15).
The selected numerical journeys retained their existing independent/original assertions.
No canonical server or managed worker was started by these native-local selections.

**Interface-checked:** independent readback challenged lexical grouping/precedence,
current synthetic resolution, reconstruction isolation, sums/order/provider demand and
compiler authority. It exposed omitted downstream clocks; the integrated correction and
targeted actual-dispatch/join tests are recorded at 41b. No material finding remained in
its examined A/B scope after readback. Root separately reviewed the final typed expiry
and benchmark caller migrations.

**Measured — construction counts only, not elapsed benefit:** actual indexed candidate,
lexical base/overlay, selected-plan/body/row, observation-structure and projection-lookup
counters exercise the removed repetition under their controlled inputs. These observations
do not establish an end-to-end speedup or explain numerical convergence. Full-series
assembled/managed/store qualification and hygiene remain with 41f; no cold/statistical
benchmark or environment/provenance proof campaign was run or scheduled here.

### A mistake made and corrected

Test fixtures initially assumed synthetic map keys equal source `Function.id`, that every
member has an explicit demanded path, and that the first admitted row is an equation.
The corrected controls use the actual authoritative identities and explicit equation
lineage, retaining their independent expected mathematics. The runtime observation test
also entered a separate diagnostic root before runtime activity, so it read no matching
metrics; it now establishes the runtime continuation before the test's nested diagnostic
scope. No production authority was weakened to make those tests pass.

### Deviations made deliberately

No required target was dropped. The implementation advanced the needed §14.3 lifetime
clarification and operating metric guide before full-series F6, to keep their explanation
consistent with the consumed implementation. Optional inherited-member reuse was refused
on its actual premises. Acceptance uses work counts and selected complete consumers; the
rest of the series and historical campaigns retain their existing boundaries.
