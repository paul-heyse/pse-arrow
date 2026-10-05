---
title: "26: Testing architecture"
status: done
date: 2026-10-05
adrs: [ADR-0160, ADR-0092, ADR-0119, ADR-0122, ADR-0143, ADR-0145]
review_sources: ["../design_review/reviews/design_review_testing-architecture_2026-10-05.md", "../design_review/reviews/design_review_testing-architecture-conformance_2026-10-05.md"]
scenario_sources: ["../design_review/reviews/design_review_testing-architecture_2026-10-05.md#revealing-changes"]
---

# 26: Testing architecture

## Purpose, baseline and ownership

Make testing a small composition of production contracts, independent expectations and
library-owned execution. Remove repeated policy, calculation, validation and setup wherever
they establish no distinct claim. Correct production foundations that obstruct that target.
This is a **hard design pivot**, including breaking internal APIs and changing policies;
historical formats, suite names, enumeration compatibility and case counts do not constrain it.

The basis is the [testing-architecture review](../design_review/reviews/design_review_testing-architecture_2026-10-05.md),
F01–F09 and S01–S08, under Core 3.3 / ProcessSimulator 1.4. The inspected production baseline
is `9c953ba490731232bd9eca8b979f8cbee5092989`. The review is an uncommitted source artifact at
authoring time. Its source observations and the focused foundation assessment below are
**Implemented/source-inspected** or **Interface-checked**; the target and all work packages
are **Proposed**. No product tests, builds or measurements were run while writing these plans.

This coordinator owns the combined design, cross-plan dependencies, F01–F09 dispositions and
the series completion decision. Companions own their designs, package progress and execution
evidence. Architecture sections and ADRs retain their existing authority; a plan records the
proposed change, not a second production specification.

| Companion | Consumes and delivers | Progress owner |
|---|---|---|
| [26a: Completed results and transport](https://github.com/paul-heyse/pse-arrow/blob/ad665a0222551196b1160e426f5242361215a6a0/docs/plans/26a-completed-results-and-transport.md) | Consumes numerical completion and joined reports; delivers immutable coherent trajectory results and shared bounded export | Its package table |
| [26b: Test composition and independent evidence](https://github.com/paul-heyse/pse-arrow/blob/ad665a0222551196b1160e426f5242361215a6a0/docs/plans/26b-test-composition-and-independent-evidence.md) | Consumes intact model selection and declaration-owned invariant derivation; delivers focused independent mechanism evidence and retired duplicate machinery | Its package table |
| [26c: Test execution and qualification evidence](https://github.com/paul-heyse/pse-arrow/blob/ad665a0222551196b1160e426f5242361215a6a0/docs/plans/26c-test-execution-and-qualification-evidence.md) | Consumes categorized tests and production result boundaries; delivers actual-effect fixtures, one report composition and scoped evidence reuse | Its package table |

Plan 26 does not reopen Plan 25n or adopt AF findings. Plan 25k remains the owner of Plan 25's
paused full product campaign. Authoring this series neither resumes that campaign nor authorizes
production implementation. When both series' qualification is authorized on the same final
tree, execute overlapping scopes once and link the same scoped evidence from their respective
owners. Plan 26's own completion checks can run without claiming Plan 25 qualification.

## Shared target and assessed foundations

| Responsibility | Owner and target use |
|---|---|
| Model meaning and preparation | Authored declarations and production admission/preparation; tests select actual definitions rather than rewriting packages |
| Local layout/value validity | Generated construction and raw admission mint checked batches; consumers borrow certificates without redundant rescans |
| Bundle validity | Production invariant derivation and actual candidate admission; a field certificate does not establish keys, references or domain completeness |
| Scientific permission | Original-model assessment and numerical completion; consumer projections derive from one retained decision |
| Independent correctness | Focused tests use analytic values, distinct numerical/reference calculations or deliberately specified bad inputs capable of rejecting a wrong production result |
| Execution | nextest/pytest own collection, selection and scheduling; recipes own environments/features/profiles |
| Evidence | One thin owner composes actual selection, terminal results, relevant inputs and native identity, without redefining scientific validity |

The existing local certificates, original assessment, completion composition, native fault
controls and `RunResult` table retention are suitable foundations. Reuse them. The trajectory's
publicly mutable completed projections are unsuitable: seal them before retaining exports.
Its whole-collection encoder is reusable after completion/header coherence is corrected; lazy
successful retention avoids repeated work without adding a per-relation encoder hierarchy.

The registry's generic integrity producer is also suitable, but its current invariant catalog
does not expose generated versus authored origin. Add that distinction at the production
producer; do not infer it from test-owned name prefixes. Catalog-wide binding remains valuable
and different from independent family value semantics. Historical semantic fixture branches
do not establish a current production rule; preserve the actual loading/admission predicates.

The runner already deduplicates recipe leaves and retains failures, interruption and provenance.
Preserve those mechanisms while removing overlapping selections, native enumeration and report
parsing. Its global source snapshot is useful context, but is unsuitable as every claim's
reuse key. The selected replacement is explicit recipe-level input scopes, conservatively
broad within a subsystem, without automatic test-impact analysis.

### Intentional changes and preservation

Completed trajectory Rust fields become read-only accessors and clone-shared immutable state.
Test fixture discovery stops requiring a valid/violating YAML pair for each generated instance.
The ordinary comprehensive configuration runs common Rust obligations once in the linked graph;
default-feature absence behavior receives a focused, explicitly different selection. Python
markers describe actual responsibilities/effects. Per-test Git attribution is deleted.
Evidence formats may change without old-format readers or transfer shims.

Preserve explicit force-validation in every correctness recipe, typed failure and incomplete
evidence, native memory caps, actual resource leases, physical units/conventions, original-model
checks, callback fault controls, lifetime tests and independent scientific references. Preserve
runtime relational enforcement. A test is removable because its distinct failure is covered or
impossible under the consumed construction contract, not merely because a validator exists.

Keep tests of constructor/admission owners and bypass paths. Do not repeatedly test their
intrinsic guarantees at every consumer. Distinct mode behavior and independently authored
semantic rules still need distinct evidence. Upstream-only library probes belong in capability
research only when a current question warrants them.

## Decisions, authority routes and implementation order

### P0 — Record the policy and boundary decisions

Before governed implementation, create a decision record for testing responsibility and scoped
qualification, including owned effects, one freshness owner, crate retirement and the removal
of compatibility-only obligations. Supersede ADR-0051 through the established route: its body
explicitly requires the overlapping governance check that this design removes. Carry forward
its complete regeneration-equivalence guarantee and current generated-path ownership. Preserve
ADR-0092's useful evidence distinctions; do not supersede it merely to change a helper.

The decision route requires a bounded review of the selected governance/crate changes. The
principal review's **Revise** judgment is diagnosis, not acceptance of a newly selected remedy.
Schedule that review against the concrete target, without a second whole-codebase audit or
another scientific campaign. Record any separately required hashing/registry/Python boundary
decision where the chosen production change alters such a contract.

Amend blueprint §14.2, §19.2, §21 and §24.1/§24.3 as applicable through the design route and
revision row. Update the qualification guide, AGENTS cleanup/selection wording, recipe docs and
reference relationship documentation together with the governed changes. Accepted ADR bodies
remain immutable. Adding a third-party library does not require an ADR or exact pin.

| Step | Required capability and reason | Delivered work and owner |
|---|---|---|
| P0 | Concrete target in this series | Required decisions/review and authority amendments; coordinator owns the decision route |
| Result authority | Existing joined reports and completion; P0 where a public contract changes | [26a A1](https://github.com/paul-heyse/pse-arrow/blob/ad665a0222551196b1160e426f5242361215a6a0/docs/plans/26a-completed-results-and-transport.md#a1): immutable projections, required before reuse |
| Result transport | A1 working result and actual leases | [26a A2–A3](https://github.com/paul-heyse/pse-arrow/blob/ad665a0222551196b1160e426f5242361215a6a0/docs/plans/26a-completed-results-and-transport.md#a2): shared lazy export with migrated consumers, then migration audit |
| Selection and family provenance | Existing intact selection; production obligation producer | [26b B1–B2](https://github.com/paul-heyse/pse-arrow/blob/ad665a0222551196b1160e426f5242361215a6a0/docs/plans/26b-test-composition-and-independent-evidence.md#b1): workload selection and typed invariant derivation identity |
| Independent mechanism evidence | B2's actual provenance and admitted rule interfaces | [26b B3](https://github.com/paul-heyse/pse-arrow/blob/ad665a0222551196b1160e426f5242361215a6a0/docs/plans/26b-test-composition-and-independent-evidence.md#b3): family semantics, catalog binding and real enforcement before witness retirement |
| Test support and retirement | Distinct retained claims established; P0 for governance/crate changes | [26b B4–B5](https://github.com/paul-heyse/pse-arrow/blob/ad665a0222551196b1160e426f5242361215a6a0/docs/plans/26b-test-composition-and-independent-evidence.md#b4): strongest freshness owner and obsolete machinery deletion |
| Effects and execution | Stable responsibility categories; no dependency on every result package | [26c C1–C3](https://github.com/paul-heyse/pse-arrow/blob/ad665a0222551196b1160e426f5242361215a6a0/docs/plans/26c-test-execution-and-qualification-evidence.md#c1): explicit fixtures, owned effects and one selection/report composition |
| Evidence and measurement readiness | C3 truthful invocation records and workload prerequisites | [26c C4–C5](https://github.com/paul-heyse/pse-arrow/blob/ad665a0222551196b1160e426f5242361215a6a0/docs/plans/26c-test-execution-and-qualification-evidence.md#c4): relevant-input reuse and selected measurement prerequisites |
| Q1 | All functional packages integrated and replacements deleted | Coordinator's single assembled acceptance stage below |

This is a dependency route, not a barrier after every row. B1, C1/C2 and proven-unused support
retirement can proceed independently of trajectory export. B3 precedes generated witness
deletion; A1 precedes A2; C3 precedes evidence reuse. Production consumer migration belongs
with each replacement, not a later cleanup phase. Shared `justfile`, rule declarations,
test configuration and generated surfaces each have one assigned writer during execution.

## Finding dispositions

The implementation is authorized. Dispositions below close with the combined source changes
and scoped Q1 evidence; design acceptance alone does not establish their correction.

| Finding | Review scenarios | Disposition | Work owner | Closure condition |
|---|---|---|---|---|
| [F01](../design_review/reviews/design_review_testing-architecture_2026-10-05.md#f01) | S01/S04 | resolved | 26b B1 | Intact production selection replaces strip/restore and all affected callers |
| [F02](../design_review/reviews/design_review_testing-architecture_2026-10-05.md#f02) | S05 | resolved | 26b B4; 26c C3 | Strongest freshness owner retained; weaker check and duplicate execution removed |
| [F03](../design_review/reviews/design_review_testing-architecture_2026-10-05.md#f03) | S02/S03/S05 | resolved | 26c C3 | Common obligations run once; mode-specific coverage and complete terminal reconciliation preserved |
| [F04](../design_review/reviews/design_review_testing-architecture_2026-10-05.md#f04) | S01/S03 | resolved | 26c C1; 26b B1/B5 | No unrelated autouse/resource setup; support boundaries and actual effect classification agree |
| [F05](../design_review/reviews/design_review_testing-architecture_2026-10-05.md#f05) | S07 | resolved | 26c C2 | Git attribution removed; owned effects and isolated negative write controls exercise the intended guarantee |
| [F06](../design_review/reviews/design_review_testing-architecture_2026-10-05.md#f06) | S06 | resolved | 26c C4/C5 | Relevant inputs/prerequisites control reuse/measurement; irrelevant prose does not |
| [F07](../design_review/reviews/design_review_testing-architecture_2026-10-05.md#f07) | S01/S02 | resolved | 26b B5 | Placeholders, empty structural crate, unused support and unneeded probes/compatibility cases retired |
| [F08](../design_review/reviews/design_review_testing-architecture_2026-10-05.md#f08) | S04/S08 | resolved | 26a A1–A3 | Immutable coherent completion and kind-correct bounded export; lifetime/resource and refusal controls pass |
| [F09](../design_review/reviews/design_review_testing-architecture_2026-10-05.md#f09) | S01/S03/S05 | resolved | 26b B2/B3/B4 | Independent family variation evidence plus every actual catalog binding/enforcement replaces instance duplication |

## Verification and Q1 assembled acceptance

Acceptance below was **Proposed** at authoring; the Outcome records executed evidence and
the conformance review records implementation acceptance. During implementation,
compile touched packages and run the named targeted controls in companions. Regenerate through
`just codegen` when declarations/generators change. Delete replaced mechanisms after the
replacement controls pass and callers move. Do not run integrated/static campaigns per packet.

After all functional scope, Q1 owns one relevant assembled selection: trajectory/direct-run/
shooting Python transport and lifetime journeys, intact reference workload selection, real
invariant candidate admission, and execution/evidence continuation controls. Include applicable
existing science oracles without weakening their assumptions/tolerances. Run `just hygiene`
once, fixing and rerunning failed recipes, plus relevant governance, `just docs`, native
and powerset manual obligations. Apply 26c's leaf composition so governance tests/freshness
already covered in that campaign are not rerun through another aggregate. The new design determines the non-overlapping
default/native/Python commands; selected reference parity runs only where its retained current
scientific claim is affected. Missing selected prerequisites fail; they never become skips.

Q1 must establish that common tests have one intended execution configuration; mode-absence
checks remain distinct; one explicit inventory/report composition exists; result projections
agree; lease accounting survives dropped parents; family oracles catch predicate/null/path
faults; and unrelated concurrent edits cannot be blamed on a test. Review the landed target
against the bounded governance/result changes without treating command success as architecture
acceptance. Broader Plan 25 physical/durable qualification remains with 25k.

Required completion is behavioral alignment, coherent authority, deletion and truthful scoped
evidence. Timing/memory comparisons are optional observations unless a quantitative benefit is
claimed. For such a claim, use representative one-table/multi-table trajectory access and
functional selection/preparation workloads with named conditions, recording retained memory
as well as duration. No throughput percentage or whole-product qualification is promised here.

## Checkpoint and closure

Closed, 2026-10-05: all functional packages, deletions and scoped Q1 acceptance are complete
on the selected uncommitted tree based on `67069879de0da7d65fa6dc505a8ff2ceea594b4c`.
The [implementation review follow-up](../design_review/reviews/design_review_testing-architecture-conformance_2026-10-05.md#remedy-acceptance)
accepts architectural fitness and bounded behavioral adequacy after correcting database
scheduling and actual Python import identity. Existing composite passes are retained;
affected controls and missing manual checks are complete. Enduring contracts remain in
their architecture owners. ADR-0160 remains proposed; decision acceptance is separate.
Plan 25k's broader campaign remains paused.

Plan 26 is retained as the highest-numbered plan. Its companions and cited reviews remain
live dependencies of this record and the pending decision; newly written contents are not
yet recoverable from Git history. Retirement follows ADR-0096 after those dependencies and
history conditions permit it. Local closure does not commit, push or publish this tree.

### Implementation-review finding dispositions

The conformance review retains its initial Revise judgment and independent Accept follow-up.
This table is the current disposition owner for its findings, distinct from the original
testing-architecture diagnosis F01–F09 above.

| Finding | Scenario | Disposition | Owner and correction | Closure evidence |
|---|---|---|---|---|
| [Conformance F01](../design_review/reviews/design_review_testing-architecture-conformance_2026-10-05.md#f01) | Conformance S04 | resolved | 26c C1: database support control moves to the existing `store_tests` namespace | Existing nextest store selector matches; focused linked control passes 1/1; independent follow-up Accept |
| [Conformance F02](../design_review/reviews/design_review_testing-architecture-conformance_2026-10-05.md#f02) | Conformance S05 | resolved | 26c C3/C4: resolve the actual imported checkout extension, hash it alone, and verify the same origin in the running pytest session | Hostile/matching/import-divergence controls; 52/52 focused tools and fresh 33/33 linked Python; independent follow-up Accept |

## Outcome

### What was built

**Implemented**, 2026-10-05: P0, A1–A3, B1–B5 and C1–C5 are integrated. Immutable completed
results own scientific permission, header and assessment; lazy checked transport shares successful
storage and retains escaped Arrow ownership. Intact authored workload selection, private typed
integrity origin, independent mechanism witnesses and complete catalog/admission controls replace
test-owned package rewriting and per-instance YAML. Three placeholders, the empty structural
crate, unused support, upstream-only probes and duplicate regeneration tests are deleted.
Explicit fixtures own mutable effects. Common Rust qualification uses one linked graph plus
separate absence controls; current evidence has one terminal composer, scoped inputs and mandatory
native identity. Measurement consumes named functional prerequisites without repeating validation.

Enduring contracts are in blueprint §14.2, §19.2, §21 and §24.1/§24.3, with the selected decision
in ADR-0160 and the bounded target review. ADR-0160 remains proposed: decision acceptance through
the PR route is distinct from implementation and testing. The companions describe actual delivery
and corrections. F01–F09 are resolved by these replacements and the scoped evidence below.

The independent conformance review initially required two bounded corrections. Database
support tests now inherit the existing store schedule; Python provenance resolves the
imported checkout extension rather than on-disk candidates and refuses a different import
inside pytest before collection. Its independent follow-up accepts the corrected scope.
The architecture test-layer map was corrected through ADR-0160 with blueprint revision 123;
this changes documentation location claims, not production semantics.

**Tested**, on this local Linux checkout, pinned nightly and linked native providers, with a
zero failure baseline and explicit force-validation in correctness recipes:

| Scope and actual command | Result and interpretation |
|---|---|
| `just setup-test-report build/plan26-q1-tools-final` | 129/129 passed, zero failures/skips. Seven affected controls passed after lint repairs; four affected environment/provenance/prerequisite controls passed after type repairs. |
| `just feature-absence --profile ci -p pse-backend-native -p pse-relations -E 'test(feature_absence::) \| test(clarabel_tests::clarabel_mkl_pardiso_refused_without_profile)'` | 2/2 passed; distinct default-feature refusal scope. |
| Linked owner selection below | Initially 106/107 passed, one multi-version registry failure. It is repaired; the affected selection below passes 29/29, and seven focused version/origin controls pass. This is composite evidence, not an initially clean run. |
| `just py-sync-native`, `just inspection-fixture build/plan26-q1-inspection`, then linked Python selection below | Fresh extension/stubs and publication; 33/33 selected tests passed, 177 deselected, zero failures/skips. Completion, Simulation/direct-run export, final array ownership, cancellation, read-only publication, explicit IPC and isolated database teardown are exercised. Shooting projection is exercised in Rust; no separate Python Shooting API is claimed. |
| `just codegen`; generated freshness leaves | All generated target comparisons pass. Each partition is run once in assembled acceptance; no governance aggregate repeats these or the linked governance tests. |
| `just hygiene` with failed/unexecuted leaves resumed individually | Composite pass through default/no-default Clippy and Rust docs: initial Python lint/type and local Rust lint findings repaired; final leaves report zero findings. The passing prefix is not repeated. |
| `just docs` | Publisher passes, 234 chapters indexed. |

Closure continuation retains the preceding observations under their original conditions,
without a fresh comprehensive run or reconstructed reuse receipts. Additional **Tested**
evidence on the same local Linux/pinned environment, against a zero failure baseline:

| Scope and actual command | Result and interpretation |
|---|---|
| `just unit-consolidation-tools` | 52/52 passed after one existing report-control mock initially imported the real extension instead of isolating the new preflight. Includes real subprocess foreign/matching/import-divergence controls and retained terminal/reuse/prerequisite controls. Composite evidence. |
| `just native-test --profile ci -p pse-operations -E 'test(testing::store_tests::failed_isolated_database_setup_removes_owned_database)'` | 1/1 passed, zero failures; explicit force-validation and linked graph. The renamed identity matches the existing store-group selector. |
| Fresh linked Python command below | 33/33 passed, 177 deselected, zero failures/skips; terminal receipt identifies the imported `_native.abi3.so` and 31 binary/library files. Existing registry-unchanged inspection publication is consumed. |
| `just lint-solver-contracts` | Zero findings; rerun after the test-namespace correction. Linked native workspace, conformance and benchmark lint; not numerical execution. |
| Completed `just features-powerset` group in `build/assessment/20261005T070845.708398Z` | Feature-combinations passed all 290 invocations; no-default workspace check passed. The already-running group completed during closure and was not repeated. It captures no source inventory, so it is retained as a scoped compile observation, not automatic unchanged-input reuse. The later `cfg(test)` namespace correction does not change this check scope. |
| `just typecheck`; `just lint-py` | Zero type diagnostics; zero lint findings after one exception-message lint correction. |
| Closure `just docs`; `just turn-end` | Publisher passes with 236 chapters indexed; ADR index and formatting pass. Formatting changes no file bytes. These establish publication/formatting, not product behavior. |

The fresh linked Python command after the provenance correction was:

```bash
PSE_INSPECTION_PUBLICATION="$PWD/build/plan26-q1-inspection" \
PSE_NATIVE_PROVENANCE="$PWD/build/plan26-q1-python-import/native.json" \
just native-python build/plan26-q1-python-import -k 'modeling_simulation_events_checks_and_terminal_reports or modeling_authored_fixture_shared_checks_and_owned_tables or blocking_async_share_terminal_report_and_last_array_owner or completion_projection_and_pre_effect_publication_ticket or conformance_runs_only_selected_fixtures or operational_store_isolation or publication_streams or extension_round_trip'
```

The linked native owner selection was:

```bash
just native-test --profile ci -E '(package(pse-schema) and binary(registry_admission)) or (package(pse-rules) and test(invariants::program::)) or (package(pse-engine) and test(session::contract::tests::)) or (package(pse-columnar) and test(payload_export_refuses_bufferless_storage)) or (package(pse-relations) and (test(validate::obligations::consolidation_unit::) or test(concatenating_checked_chunks_does_not_certify_duplicate_keys) or test(checked_arrow_export_retains_storage_container_and_wrapper_lease))) or (package(pse-operations) and test(failed_isolated_database_setup_removes_owned_database)) or (package(pse-runtime) and (test(trajectory_transport_shares_completion) or test(declared_terminal_endpoint_qualifies) or test(kernel_integrated_dae_checks_partition) or test(kernel_integrated_definite_integrals) or test(shooting_trajectory_projection_retains) or test(shooting_matches_simultaneous_optimum) or test(authored_shooting_fixture_solves) or test(validation_failure_keeps_solve_projection) or test(kernel_conformance_runs_only_selected_fixtures) or test(source_and_physical_boundaries_refuse_the_same_package_closure_and_cycle))) or (package(pse-tests-conformance) and (test(acceptance::dynamic_optimization::) or test(acceptance::complementarity_flash::))) or (package(xtask) and (test(codegen::) or test(inspection_fixture::tests::))) or package(pse-tests-governance)'
```

After the producer repair, the affected native selection was:

```bash
just native-test --profile ci -E '(package(pse-schema) and (binary(registry_admission) or test(builder::integrity::tests::))) or (package(pse-rules) and test(invariants::program::))'
```

Linked Python acceptance used the fresh inspection publication and native provenance path:

```bash
PSE_INSPECTION_PUBLICATION="$PWD/build/plan26-q1-inspection" \
PSE_NATIVE_PROVENANCE="$PWD/build/plan26-q1-python/native.json" \
just native-python build/plan26-q1-python -k 'modeling_simulation_events_checks_and_terminal_reports or modeling_authored_fixture_shared_checks_and_owned_tables or blocking_async_share_terminal_report_and_last_array_owner or completion_projection_and_pre_effect_publication_ticket or conformance_runs_only_selected_fixtures or operational_store_isolation or publication_streams or extension_round_trip'
```

Freshness leaves are `codegen-hakari-check`, `codegen-relations-check`,
`codegen-rust-contracts-check`, `codegen-docs-check`, `codegen-postgres-check`,
`codegen-queries-check`, `codegen-python-check`, `codegen-bindgen-check` and
`codegen-schemas-check`. The completed hygiene leaves include agent/ADR/register, spelling,
license, action/shell/AST/Python lint, typecheck, import/engine boundaries, solver pins,
family consistency, generated freshness, default/no-default Clippy and Rust documentation.
Their scope is static/mechanical; they do not qualify unexecuted scientific workloads.

### A mistake made and corrected

Native provenance initially distinguished historical relation versions without reconciling
unversioned SQL binding. That exposed duplicate generated declarations in a migration test.
The producer now derives only the highest relation version and rebuilds its own outputs,
preserving authored predicates and historical migration declarations. Independent mechanism
witnesses also exposed dictionary/run-end lowering defects, which were repaired in production.
Tool source review caught incomplete input capture, optional standalone provenance and lost
transfer rationale; the owning controls now reject those failures. A generic stub extractor
was corrected to honor actual renamed PyO3 classes, rather than adding a test-helper exception.

The final conformance review caught a database test outside the actual-effect schedule and
Python provenance that hashed filesystem candidates rather than the imported extension.
Those were corrected at the existing namespace and execution owners, with discriminating
controls and affected journeys rerun. Its initial Revise judgment remains in the review;
the independent remedy follow-up supplies the bounded implementation acceptance.

### Deviations from the plan, deliberate

The process deployment resource owner remains strong, with immutable session settings and
individually owned test facades/effects. A weak registry would permit old escaped-buffer charges
to overlap a new pool and mismatch the executor lifetime. Whole-map trajectory export retains
its selected first-access cost; bufferless payload export is explicitly refused. No quantitative
speed, memory or whole-product qualification is claimed. Plan 25k remains paused; other hosts,
CI, distributions, full reference parity and broad performance campaigns are outside this receipt.
