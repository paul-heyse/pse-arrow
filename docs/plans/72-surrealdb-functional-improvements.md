---
title: "Plan72: SurrealDB functional improvements"
status: active
date: 2026-11-09
adrs: [ADR-0261, ADR-0262, ADR-0267, ADR-0268]
doc_role: plan
doc_owner: docs/plans/72-surrealdb-functional-improvements.md
doc_topics: [current-work, design, deployment, storage, libraries, validation, diagnostics]
review_sources: [docs/functional_reviews/surrealdb-functional-improvements-2026-11-09.md]
scenario_sources: [docs/plans/72-surrealdb-functional-improvements.md]
---

# SurrealDB functional improvements

## Target and authority

**Proposed.** Authors define native contracts once, inspect their installed interpretation,
and evolve them through owned transitions. Canonical callers submit complete typed operations.
Engineers can search the full selected population, rank relevant evidence, explain bounded
dependency paths and follow current study state. Operators obtain coherent diagnostics,
protected result access and truthful maintenance/recovery through existing native owners.

Use native database planning, types, graph relationships, transaction procedures and SurrealKit
where they remove unnecessary interpretation, transfer and coordination. Scientific meaning,
independent assurance, original operation identity and effect custody remain explicit. Breaking
interfaces, representation, execution placement and physical implementation are acceptable.
Keep one selected production realization and remove what it replaces as consumers move.

This series derives from the complete [functional review](../functional_reviews/surrealdb-functional-improvements-2026-11-09.md)
and its seven supporting assessments, published at `21f703887cc307f986975f9a2fba998f8bd2afa7`.
It supersedes Plan68 **without importing any scope, target, open package or acceptance obligation**.
Plan68 is inactive legacy. Historical citations inside the source review are provenance,
not an additional source of work for this series.

[Plan66](66-integrated-simulation-target.md) retains its scientific implementation and assembled
acceptance. [Plan71](71-workflow-and-simulation-test-feedback.md) is complete for its adopted
scope and supplies test infrastructure, original-attempt evidence and diagnostics.
Neither series is superseded or amended here.
Plan72 owns only the SurrealDB improvements and receiving obligations described below.

The 72b/72c implementation is committed. Their companions retain open verification/repair
and closeout obligations at the maintainer's request to move on; implementation is not
completed acceptance. Resume from those owning checkpoints rather than reopening library
selection or inferring completion from the commits.

## Baseline and focused foundation assessment

**Interface-checked.** Authoring uses the review publication and current working tree, including
concurrent Plan71 changes. Source owners are the registry/Surreal lowering and canonical codecs;
`GuardedScript` and native transaction functions; study policy/module; metadata/graph/result
readers; native service and source-build preparation; SurrealKit fixtures and telemetry.
The relevant contract and source evidence is linked in each companion. Source inspection and
the review establish the rationale; they do not establish delivered or measured improvements.

The installed family is already v4: types/parser/expression 4.0.0, protocol 0.14.1, MX 1.1.0,
Surrealism 0.7.0 and source-pinned SurrealKit rc.1. Exact manifests, host source declarations,
features and patch identities remain their authorities. Reconfirm affected inputs when executing
a package; do not restart release cutover or inherit a v3 compatibility campaign.

The existing foundations support a direct evolution rather than a replacement framework:

- Registry lowering already emits native declarations/codecs. Improve its contract vocabulary,
  operation inputs and installed interpretation; do not introduce another editable model.
- Native operations already carry authority and original receipts. Explicit isolation and typed
  procedures can replace incidental framing; generation/lease meaning cannot be removed as if it
  were merely a database conflict key.
- Private knowledge and canonical graph queries already have selected cuts and bounded pages.
  Expose complete selection intent to the executor, and choose physical relationships with their
  access paths rather than assuming an adjacency cache accelerates every indexed relation scan.
- Keep compact canonical metadata in persistent MX and retain the bounded direct client as the
  selected starting realization. Full-range unsigned transport, subscription custody, migration
  effects and bucket capabilities are meaningful correctness-risk decisions, not reasons to defer
  the rest of the target.
- Existing native fixtures, structured reports and Collector supply reusable integration points.
  Consume Plan71 contracts and independent scientific readers; add neither a parallel runner nor
  a second diagnostic outcome authority.

Apply core 3.4, efficient architecture 1.0, process-simulator 1.5, test assessment 1.2 and the
[repository binding](../design_review/design_principles/standard.toml). This focused assessment
does not initiate a broad audit or certify an enclosing subsystem.

## Decisions and confirmed rule directions

The maintainer confirmed RC01 and RC02 explicitly on 2026-11-09 and selected richer current
workflows and breaking designs earlier in this engagement. Approval is of planning direction;
required decision records precede implementation of the affected enduring contract.

| Review impact | Planning decision and route |
| --- | --- |
| RC01 — canonical schema/value/interpretation/generation | Accepted direction, 2026-11-09. A01 names concrete changes to blueprint §4/§5/§20, identifies any superseding ADR/design revision, and schedules it before dependent representation changes. No hash or scientific meaning changes are implicit. |
| RC02 — transaction/policy/guards/defaults | Accepted direction, 2026-11-09. C01 identifies D10/D14, authority/commit and patched-default changes and their ADR/design route before dependent implementation. Keep semantic fences and original acknowledgements. |
| RC03 — richer discovery/observation/operating interfaces | Selected scope: implement richer existing workflows without requiring a pre-existing caller. D/E/G own explicit public semantics and changed feature/authorization contracts; use the normal route for enduring changes. |
| RC04 — persistence/transport/artifact alternatives | MX and direct bounded gRPC remain selected. F owns finite decisions for alternatives with actual isolation/history/capability consequences; a changed architectural selection uses its decision route before integration. |
| RC05 — tooling/fixtures/diagnostics | Integrate through existing owners and Plan71. Ordinary tooling/refactoring needs no invented ADR or approval gate; contract/governance changes still use their actual route. |

Pivot to the strongest design where source and qualitative reasoning provide a clear basis.
No benchmark, pilot or repeated benefit demonstration is required for clearly removed work.
Use a probe/comparison only for an unresolved premise posing serious correctness or performance
degradation risk. Targeted implementation tests still show new behavior. A quantitative claim,
if made, needs measured conditions; making no such claim is not an incomplete package.

## Series and contract ownership

| Companion | Owned output |
| --- | --- |
| [72a — Native contracts and schema interpretation](72a-native-contracts-and-schema-interpretation.md) | Shared declaration/native mapping, exact representation decision and compiler/live interpretation. |
| [72b — Schema evolution and deployment](72b-schema-evolution-and-deployment.md) | Typed effect-bearing transition, reconstruction/native/backfill selection, activation and recovery. |
| [72c — Canonical operations and policy](72c-canonical-operations-and-policy.md) | Named operations/isolation, typed readiness, precise conflicts/retries and shared authority. |
| [72d — Query, graph and ranked discovery](72d-query-graph-and-ranked-discovery.md) | Selected-population search, graph/path explanation, ranked lexical and justified retrieval extensions. |
| [72e — Observation and derived state](72e-observation-and-derived-state.md) | Owned following, summaries, precise invalidation and optional durable incremental projections. |
| [72f — Persistence and protected delivery](72f-persistence-and-protected-delivery.md) | Persistence/history and maintenance, artifact custody and complete transport/client delivery. |
| [72g — Developer and operator workflows](72g-developer-and-operator-workflows.md) | Database-contract composition and structured operating/client access using Plan71 foundations. |
| [72h — Diagnostics and execution fit](72h-diagnostics-and-execution-fit.md) | Selected-operation explanation, native execution/scratch and risk-directed mixed-workload evidence. |
| [72i — Integrated acceptance and closeout](72i-integrated-acceptance-and-closeout.md) | Series-wide receiving, applicable scope-end qualification, retirement and enduring documentation. |

A owns declared field/presence/value policy; F owns actual transport assembly, admission and
connection lifetime. C owns authoritative transaction/capture protection; E consumes that choice
for refresh and summary behavior. D owns query meaning; H observes the same builders and never
silently changes inspection into execution. B owns transitions over adopted A/C/F contracts.
G supplies database-specific contract composition while I selects assembled receiving.

## Dependency route and independent work

Start A01/A02, C01 and D's complete-query design from available code. G can consolidate existing
contract inputs while A's new representations are developed. F can settle precise value/wire and
bucket questions independently; no engine replacement is prerequisite for ordinary improvements.

A's chosen mapping and F's actual wire behavior must agree before native numeric cutover. B can
build typed transition intent before a particular representation migrates, then receive that
actual change. C's pure typed readiness can advance with A's carrier contract; precise witness
deletion depends on its real read/authority footprint, not on all of E. D's lexical and graph
improvements do not wait for subscriptions or buckets. E's snapshot-follow capability depends
on a qualified subscription lifetime; one-shot snapshots and query-first summaries remain useful
independently. H can join existing diagnostics while new query/stream producers arrive.

Each companion names its smaller prerequisite slices. An agreed schema or interface is not a
working implementation. Shared files need coordinated ownership even where packages are logically
independent. Use the existing checkout and preserve concurrent Plan66/71 edits.

## Cross-plan boundaries

| Existing owner | Plan72 integration rule |
| --- | --- |
| Plan66 contracts/results/runtime | Consume actual scientific source, result and protection semantics. Schedule necessary database-side consumer migration in Plan72 with its semantic owner; do not adopt or close Plan66's wider physics, numerical or assembled acceptance. |
| Plan71 CLI/original evidence/fixture efficiency | Reuse delivered references, reports, fixture ownership and selected diagnostic contracts. Record only SurrealDB-specific extensions here; inspect live work before treating a gap as missing. |
| Current architecture and accepted ADRs | Retain required meaning until the adopted decision route changes it. Do not use inactive Plan68 text as implementation authority or a hidden qualification obligation. |

## Opportunity dispositions

This table owns source-opportunity disposition; companion package tables own progress.
Scheduled means designed for implementation on adoption of this draft, not delivered.
Investigations have finite decisions and dependent branches; a justified retain/no-change
outcome can close that investigation without leaving an unbounded backlog.

| Opportunity | Disposition | Work owner | Evidence or revisit trigger |
| --- | --- | --- | --- |
| [O01](../functional_reviews/surrealdb-functional-improvements-2026-11-09.md#O01) | Scheduled | 72a A01–A03; 72f transport | Shared named vocabulary and policy-aware mappings; resolve full-range/native-kind risk before representation cutover. |
| [O02](../functional_reviews/surrealdb-functional-improvements-2026-11-09.md#O02) | Resolved | [72a A02/A04 Outcome](72a-native-contracts-and-schema-interpretation.md#outcome) | Typed operation inputs, strict full-schema compiler/live represented agreement and drift/coverage controls delivered. Existing generator retained; no disjoint external emitter selected. |
| [O03](../functional_reviews/surrealdb-functional-improvements-2026-11-09.md#O03) | Scheduled | 72b B01–B03 | Exact typed transition with data effects and recovery; select reconstruction/native migration/backfill per actual change. |
| [O04](../functional_reviews/surrealdb-functional-improvements-2026-11-09.md#O04) | Scheduled | 72c C01/C03 | Explicit complete transactions and one retry owner; independent concurrency/refusal cases protect guard removal. |
| [O05](../functional_reviews/surrealdb-functional-improvements-2026-11-09.md#O05) | Scheduled | 72c C02; 72a carriers | Typed pure shared readiness with visible acquisition; transaction access/init/bindings selected only for concrete benefit and supported contract. |
| [O06](../functional_reviews/surrealdb-functional-improvements-2026-11-09.md#O06) | Scheduled | 72c C04 | Shared operation meaning and explicit invocation/execution authority; replace an ingress rather than add duplicate policy. |
| [O07](../functional_reviews/surrealdb-functional-improvements-2026-11-09.md#O07) | Resolved | [O07 receiving](72d-query-graph-and-ranked-discovery.md#outcome) | Complete native selected-population filtering, explicit set coverage and one hydration; targeted late-match controls passed. |
| [O08](../functional_reviews/surrealdb-functional-improvements-2026-11-09.md#O08) | Resolved | [O08 receiving](72d-query-graph-and-ranked-discovery.md#outcome) | Frozen bounded occurrence/evidence paths delivered; SurrealQL selected, GQL/materialized inverses closed with triggers. |
| [O09](../functional_reviews/surrealdb-functional-improvements-2026-11-09.md#O09) | Resolved | [O09 receiving](72d-query-graph-and-ranked-discovery.md#outcome) | Native best-excerpt BM25, Unicode offsets and rank-epoch continuation delivered; maintainer selected lexical/graph without embeddings. |
| [O10](../functional_reviews/surrealdb-functional-improvements-2026-11-09.md#O10) | Resolved | [O10 receiving](72e-observation-and-derived-state.md#outcome) | Owned fixed-Viewer Subscribe, as-of refresh, bounded fresh principal registration and explicit loss/cleanup delivered and tested. |
| [O11](../functional_reviews/surrealdb-functional-improvements-2026-11-09.md#O11) | Resolved | [O11 receiving](72e-observation-and-derived-state.md#outcome) | Native grouped summaries and exact indirect-run invalidation delivered; query-first reconstruction selected, capture SSI remains 72c’s owner. |
| [O12](../functional_reviews/surrealdb-functional-improvements-2026-11-09.md#O12) | Resolved | [O12 receiving](72f-persistence-and-protected-delivery.md#outcome) | Original-incarnation passive metrics delivered; unsupported MX VERSION/manual snapshot remain explicit; capacity/history/topology triggers retained. |
| [O13](../functional_reviews/surrealdb-functional-improvements-2026-11-09.md#O13) | Resolved | [O13 receiving](72f-persistence-and-protected-delivery.md#outcome) | Sealed files retained: selected bucket lacks complete durable no-clobber publication/recoverable reservation custody; replacement trigger explicit. |
| [O14](../functional_reviews/surrealdb-functional-improvements-2026-11-09.md#O14) | Resolved | [O14 receiving](72f-persistence-and-protected-delivery.md#outcome) | Exact native/Proto readback plus tentative incremental delivery under original terminal/allocation/executor custody delivered and tested. |
| [O15](../functional_reviews/surrealdb-functional-improvements-2026-11-09.md#O15) | Resolved | [O15 receiving](72g-developer-and-operator-workflows.md#outcome) | Existing SurrealKit suite composes independent external pass/fail/interruption under original physical ownership; full fixture passed. |
| [O16](../functional_reviews/surrealdb-functional-improvements-2026-11-09.md#O16) | Resolved | [O16 receiving](72g-developer-and-operator-workflows.md#outcome) | Typed ordinary operating reads delivered; MCP filesystem effects and extra SQL/GraphQL interpretation boundaries close unselected adapters with participant triggers. |
| [O17](../functional_reviews/surrealdb-functional-improvements-2026-11-09.md#O17) | Resolved | [O17 receiving](72h-diagnostics-and-execution-fit.md#outcome) | Shared same-query/versioned explanations and actual Collector delivery integrated with original outcome/report owners; unavailable counters/progress explicit. |
| [O18](../functional_reviews/surrealdb-functional-improvements-2026-11-09.md#O18) | Resolved | [O18 receiving](72h-diagnostics-and-execution-fit.md#outcome) | Native sort spill, cancellation, failed scratch allocation, readiness refusal and cleanup tested; supplier scheduling retained without a quantitative benefit claim. |

The source review's [complete discovery correspondence](../functional_reviews/surrealdb-functional-improvements-2026-11-09.md#complete-discovery-correspondence)
is included in scope at the level of its individual directions, not just these eighteen titles.
Companion correspondence tables map its detailed branches to packages and decision outcomes.
Earlier features, inline/reference/cache choices, sequences, native views/events, feeds, codec
alternatives, emitters, config/seeds and diagnostics remain eligible regardless of release age.
Do not silently turn all mechanism mentions into required adoption.

The review's deliberately unselected tails remain recorded decisions: geographical retrieval,
Agent Memory, a new Kafka/Node/web stack and unrelated hosted/browser deployment have no selected
functional proposition. D/F/G own observable within-scope triggers that would reopen them. Removed
SurrealML is unavailable. These dispositions capture the findings without manufacturing work.

## Verification and completion

**Proposed verification.** Packages name targeted behavior/refusal tests and their replacement
deletions; use scoped `pse check`/`pse test` and explicit force-validation where applicable.
Run codegen when declarations/generators change. I selects relevant integrated journeys and
scope-end hygiene/governance/docs/native checks once functional scope is implemented, following
repository policy. Do not repeat static campaigns per companion or require generic benefits proof.

Completion requires delivered selected behavior, closed finite investigations with their branch
outcomes integrated, moved consumers, deleted replacements, actual required receiving and settled
effects. Local implementation, assembled qualification and benefit measurements stay separate.
Plan66 scientific acceptance and Plan71 completion remain with their own owners. Move enduring
meaning to its architecture/developer owners through the required routes before retiring this
series and resolved source reviews. No production change is performed by authoring this draft.

<a id="72dh-execution-outcome"></a>
## 72d–72h execution outcome

**Implemented**, 2026-11-10. Authorized D01–H04 functionality is delivered with migrated
consumers and replacement deletions. The companion Outcomes own their behavior, corrected
mistakes and finite branch decisions. Enduring meaning is in blueprint §20.5/§21.7 and the
native substrate guide under proposed ADR-0267; scope-end boundary corrections use proposed
ADR-0268 and blueprint §3.1/§3.2. Formal ADR acceptance is separate from implementation.

The companions remain available as receiving inputs to 72i. This is not completion of the
whole series: 72b/72c retain their parked verification/repair obligations, and 72i retains
series-wide journeys, qualification selection and retirement. No scientific/IDAES acceptance,
durable notification history, filesystem-change observation or quantitative speedup is claimed.

**Tested.** The selections below ran on Linux with the pinned toolchain, source-built v4
server, prepared native module/SurrealKit and owned local fixtures. Native correctness routes
retain explicit force-validation for their actual Arrow closures. Every listed final pass
has zero failures against a zero baseline and complete original invocation settlement.
Selections overlap; counts are not added into a new coverage total.

### Discovery receiving

```bash
scripts/pse test xtask --lib --no-default-features --features evidence-tools --ignored all -E 'test(~knowledge_) or test(~evidence_native_scoped_roles_complete_generation_and_cursor_fences)'
scripts/pse test pse-operations --lib --features native-service,canonical-maintenance -E 'test(~bounded_path_) or test(~canonical_graph_explanation_preserves_isolates_parallel_edges_and_protected_inputs)'
```

**Tested:** 7/7 private discovery/follow cases and 3/3 graph cases passed. These establish
the selected late-match/rank/Unicode/cursor/permission and bounded frozen-path distinctions
described in 72d/72e. They do not establish semantic paraphrase quality or complete closure
outside the declared path scope.

### Observation and custody receiving

```bash
scripts/pse test pse-operations --lib --features native-service,canonical-maintenance -E 'test(~canonical::transport::tests::) or test(~transport::client::results::) or test(~transport::lifetime::tests::) or test(completed_query_drains_on_its_physical_executor_after_the_caller_stops_polling) or test(early_query_ack_keeps_physical_admission_bounded_and_original_deadline_releases_uploads) or test(~bounded_path_) or test(~canonical_graph_explanation_preserves_isolates_parallel_edges_and_protected_inputs) or test(~current_summary_and_fixed_viewer_follow_preserve_scientific_classification) or test(~native_server_metrics_reach_owned_collector) or test(~native_sort_spills_exact_results) or test(~native_explicit_invalid_scratch)'
scripts/pse test pse-operations --lib -E 'test(~transport::lifetime::tests::) | test(grpc_authenticate_preserves_access_for_neutral_responses_and_replays_real_rotation) | test(completed_query_drains_on_its_physical_executor_after_the_caller_stops_polling) | test(early_query_ack_keeps_physical_admission_bounded_and_original_deadline_releases_uploads)'
scripts/pse test pse-runtime --lib --features product-reader,engineering-service -E 'test(=tooling::reader::tests::canonical::project_study_workbench_reads_the_selected_store_and_refuses_foreign_cursor)'
scripts/pse test pse-runtime --lib --features product-reader,engineering-service -E 'test(~workbench_public_cohort_retains_population_compares_value_variants_and_refuses_incompatible_quantity)'
```

**Tested:** 80/80 joined native cases, 19/19 focused lifecycle/auth cases and both selected
runtime journeys (1/1 each) passed. The joined scope receives original-executor drain,
terminal framing/errors, aggregate close, subscription registration/cleanup failures, native
summary/follow/65-member streaming, graph, Collector and spill. The runtime journeys receive
ordinary project access and frozen cohort export with original scientific classifications.

### Operator and diagnostic receiving

```bash
scripts/pse test pse-operations:native_service --features native-service,canonical-maintenance --ignored only -E 'test(=surrealkit::surrealkit_generated_contract_uses_owned_service_and_disposes)'
scripts/pse test pse-operations --lib --features native-service -E 'test(~native_server_metrics_reach_owned_collector) or test(~native_telemetry_is_explicit) or test(~native_persistence_observation_requires) or test(~native_persistence_prometheus_preserves) or test(~native_plan_unknown_operators) or test(~native_explanation_keeps) or test(~query_memory)'
```

**Tested:** the complete authored SurrealKit receiving fixture passed (1/1), including
independent external pass/fail/interruption and owned logical/physical disposal. The focused
telemetry/explanation/memory selection passed 7/7. Actual native sort/spill, cancellation,
scratch refusal/cleanup and delivered Collector metrics also passed in the joined 80-case
selection. These are concrete delivery and lifecycle controls, not measured throughput.

**Tested:** a final complete SurrealKit confirmation also passed 1/1 after publication and
formatting. Its first additional attempt was refused before test discovery completed when
the filesystem filled; no fixture started and that invocation settled. For this named
space-recovery need, `scripts/pse run -- cargo clean -p pse-runtime` released reproducible
package build objects under Cargo's owner. The full confirmation and `just ready` then
passed. Managed result retention and the original kernel-stuck supplier build keep their
existing owners.

### Scope-end corrections

**Implemented.** The initial aggregate hygiene/governance run failed against the zero target:
stale ADR index, Python lint/type findings, outdated Tokio ceiling, unused dependencies,
new-code Clippy findings and two quantity fixture governance failures. Those failures were
repaired rather than accepted as a baseline. Final acceptance uses the original passing
unchanged recipes and the successful affected reruns below; the initially failed aggregate
is not relabeled as a pass.

Unused dependency edges and six unused native-policy publications were removed at their
declarations/generator. Thirteen exact, actually consumed parent-relative generated paths
retain documented exceptions for the pinned cargo-shear path-normalization defect; no
audit output is filtered and no generated tree is exempted wholesale.

Independent Python references now use precise Pyomo/component protocols and descriptor-backed
protobuf access, with exact `types-protobuf==7.35.1.20260906` in the isolated parity group.
The scoped unittest/driver lint policy matches their existing independent process role.
Formulas, cases, tolerances and deliberately infeasible controls retain their original
meaning. **Tested:** `just lint-py` passed, `just typecheck` passed both Python 3.13 and
the separate Python 3.12 selection with zero diagnostics; `just reference-unit` passed
25/25 and `just reference-solver-unit` passed 7/7 with prepared Ipopt, against zero
failures. These receive the changed reference boundaries, not Plan66 scientific acceptance.

Proposed ADR-0268 aligns the ceiling with the maintainer's explicit semantic/runtime
clarification. It excludes product execution owners while permitting concurrency libraries.
The quantity fixture uses its existing nondefault isolated-fixture authority, with a
dev-only feature, reasoned cfg(test) assertion and selected scientific-replay Miri coverage.
The allowlist governs crate membership/reasons; it does not itself prove test-only use.
That confinement must also be established from the source and review. Formal decision
acceptance remains separate.

**Interface-checked:** the [independent focused boundary assessment](../design_review/reviews/design_review_semantic_boundaries_2026-11-10.md)
supports this target without a new public mint or execution framework. Its F01 stale
layering sentence is resolved in blueprint §3.2 under revision 383. The quantity replay
selection `scripts/pse test pse-quantity --lib -E 'test(~resolved::receipts::replay_tests)'`
passed 2/2 against zero failures. The added Miri declarations are inspected coverage;
Miri has not been run here and is not claimed as semantic-admission proof.

**Tested:** the runtime repair selection below passed 7/7 against zero failures, including
actual expectation/replay wire round trips and cross-process numerical restoration. Byte
accounting retains eight words; missing decode leases now return typed refusals. Boxing
the large expectation replay request preserves its serialized representation.

```bash
scripts/pse test pse-runtime --lib -E 'test(expectation_local_selected_population_preserves_original_science_and_reader) | test(adaptive_expectation_affine_point_reference_keeps_finite_population_covariant) | test(fixed_expectation_matches_independent_finite_contributions_and_original_replay) | test(replay_constructor_tag_is_exclusive_and_round_trips) | test(replay_constructor_all_families_preserve_their_owned_payloads) | test(numerical_wire_nesting_refuses_before_unreadable_publication) | test(canonical_numerical_observation_survives_producer_exit_without_source_reconstruction)'
```

Document lifecycle bindings now select the actual checkpoint headings without changing
72b/72c's parked native scope. **Tested:** `target/debug/xtask lifecycle validate` passes
with zero metadata/scope errors; `just docs` builds the complete HTML/scoped-search site.
The prepared binary route performs this metadata read without another implicit build.
The pure generator isolation control also passed 1/1 with
`scripts/pse test xtask:xtask --target-kind bin --no-default-features -E 'test(=codegen::private_store::tests::knowledge_schema_is_tooling_only_with_declared_fulltext_and_native_graph)'`.
It checks the canonical registry boundary directly, without referencing an unselected
optional model dependency from the default generator test closure.

### Scope-end qualification

**Tested**, zero failure baseline. `just codegen` passed all 16 generators, including the
six unused policy removals and their provenance. The final `just ready` passed skill sync
and doctor after the dependency and prepared-source changes. The all-target check
`scripts/pse check pse-runtime pse-host xtask --all-targets --features pse-runtime/product-reader,pse-runtime/engineering-service,xtask/canonical-tools,xtask/evidence-tools`
passed with zero compiler diagnostics. Later focused tests compiled their final changed
closures. Required scope-end results are:

| Command/selection | Final result and scope |
| --- | --- |
| `scripts/pse run-all clippy-default clippy-no-default` | Both workspace/all-target configurations passed with `-D warnings`, under the correctness adapter. |
| `governance-tests` in the affected follow-up bundle | 25/25 passed; original unsafe/reason failures repaired. The earlier accompanying Clippy failures have the separate passing rerun above. |
| `scripts/pse run-all adr-frontmatter-check adr-index-check family-check audit-shear codegen-check lint-actions docs-rust` | All 7 recipes passed after publication and formatting; shear reports 0 errors/0 warnings, generation compares all 16 targets without drift. |
| `just lint-py`, `just typecheck` | Passed; both isolated typing environments report 0 diagnostics. |
| `just turn-end` | ADR index and the pinned source formatting/safe-fix recipe passed. Formatting of existing integrated Plan73 sources is a separate commit with no behavior edits. |
| Original unchanged hygiene recipes | Agent wiring, register, spelling, licensing, shell/AST, engine boundary and solver pins passed in the original selection. |

No required 72d–72h implementation or selected qualification remains. Source-built server,
module, external consumer and runtime fixtures settled through their original owners.
The earlier failed supplier-build kernel residual and retained cache are explicitly recorded
in [72h](72h-diagnostics-and-execution-fit.md#outcome); replacement receiving is not evidence
of that residual's physical drain. No whole-series or scientific qualification is inferred.
