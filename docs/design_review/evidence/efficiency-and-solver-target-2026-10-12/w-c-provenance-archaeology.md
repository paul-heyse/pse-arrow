> Supporting evidence for [the efficiency and solver target review](https://github.com/paul-heyse/pse-arrow/blob/6af92d1dee1ffa0b80dca71e03a63b489f2d940c/docs/design_review/reviews/design_review_efficiency-and-solver-target_2026-10-12.md). Inquiry W-C (code-mapper): machinery provenance and qualification archaeology from git history and plan texts. Baseline: `main` at `6c368c88a`. This contribution is preserved as returned; its local labels are not finding identifiers, and the principal review owns the judgment.

# W-C: machinery provenance and qualification archaeology

Baseline: `main` at `6c368c88acda548d421b44e8fc4f457c95bcec7e`, clean tree, 2026-10-12. Read-only inquiry.
Sources: `git log`/`git show`/`git log -S`, ADR front matter, plan Outcome and checkpoint text, blueprint
revision table, HEAD file contents. No builds, tests or services. Facts are separated from labelled
interpretation. Dates are commit dates (git `%ad`, short).

## 0. Limits that bear on every row

- **Commit granularity collapses after 2026-10-05.** Of 497 commits since 2026-09-28, 111 have no
  conventional type; from 10-06 the large ones are "plan 28 done", "plan 30 finished",
  "plan 33 efficiency implemented", "plan 34 implemented", "first wave of plan 36 implementation"
  (see s6). "Introduced in commit X" for 10-06..10-11 therefore means the bulk commit that first
  contained the string, found with `git log -S`; it does not date the idea inside the plan.
- Counts of terms (lease, fence ...) are grep counts of the words in HEAD files, not mechanism counts.
- "57 `canonical_*` relations" is verified: 57 files `crates/pse-relations/src/generated/runtime/canonical_*.rs`,
  57 quoted `canonical_*` names in `crates/pse-schema/src/catalog/substrate.rs` (the declaration), 57 `DEFINE TABLE`
  in `crates/pse-operations/src/generated/surreal.surql`.
- Not searched: the contents of `docs/design_review/evidence/**` beyond plan-cited excerpts, and Plan 28 companions
  28f/28g/28k in detail. A mechanism absent from the tables below is not thereby shown absent.
- Not run: the retired Plan 37 companions (37a/b/c) were read only through the coordinator plan's quoted outcomes.

## 1. Eras of the durable lifecycle substrate (facts)

| Era | Dates | Authority for publication / durable runs | Evidence |
|---|---|---|---|
| Delta-era | before 09-27 | Delta tables + `pse-catalog` publication catalog (`crates/pse-catalog`, `vendor/delta-rs`); retirement/tombstone/reclaim first seen in Plan 9, `a46f358bd` 2026-09-18 | `git log -S` on tombstone, Reclaim, JoinSet all first hit `a46f358bd` |
| PostgreSQL-era | 09-27 to 10-06 | PostgreSQL 18 operational store in new crate `pse-operations` (`9ac684ac2` 09-27, Plan 22 O1/O2); durable attempts+leases `0157e3857` (O3); `e83c73615` pse-worker + cross-process cancellation (O4); listener task `3dfb5764f` (B2.4/X8); durable studies `79661d4dd`/`512f4444c` (O7); durable incumbent resume `b80f8d4c1` (G8); "PostgreSQL catalog is the only publication authority" `4487adfc4` 09-28 (O8); ADR-0114 accepted 09-28 | Plan 22 closed `45dd53bee` 09-29 |
| SurrealDB-era | 10-05 onward | ADR-0164 dated 2026-10-05 (added to git 10-06, status `proposed`); `0b4abf7c6` (10-06, "plan 28a and 28b mostly implemented") and `06302af77` (10-06, "plan 28 done") | `06302af77` deletes PostgreSQL store files (`attempts.rs`, `cancellation.rs`, `jobs.rs`, `lifecycle.rs`, `listener.rs`, `retirement.rs`, `studies.rs`, `store.rs`, `schema.rs`, `streams.rs` ...), the `pse-catalog` crate and `vendor/delta-rs` (-107,255 lines over the period); blueprint rev 130 "retirement of PostgreSQL/Delta composition" |

Observed: the publication/run authority changed twice between 09-27 and 10-06 (Delta+catalog -> PostgreSQL -> SurrealDB).
PostgreSQL-era lease/heartbeat/listener/job mechanisms existed 9 days (09-27..10-06); their obligations are named in Plan 28a
("useful lifecycle obligations ... physical representation and recovery protocols are replaced").

ADR status: ADR-0164, 0166, 0168-0182 are all `status: proposed`; ADR-0165 (pins) is `accepted`. None of the lifecycle ADRs
(0164, 0166, 0169, 0172-0182) has been accepted; the blueprint nevertheless carries their contracts (revisions 128-159).

## 2. Admission mechanisms (the brief asks for at least seven)

Lines = HEAD line counts. "Sense" is the observable role. All are named "admission" in source or docs.

| # | Mechanism (owner) | Sense | First commit / date | Plan / ADR | Later modification |
|---|---|---|---|---|---|
| 1 | `crates/pse-engine/src/session/admission.rs` (991 lines): `admit_plan` at every DataFusion logical-plan boundary | semantic/provider admission of plans | `1407716fc` 2026-09-24 (pre-window checkpoint) | ADR-0039 (09-14) semantic admission | 4 commits since 09-25, last `06302af77` 10-06 |
| 2 | `crates/pse-admission` (6,694 lines src + tests): `authoring_driver`, `physical`, generated `document_adapters.rs` | owned document intake, physical source admission through native bindings; "lower admission" | `ff5a4aa2f` 2026-10-11 (34 files, +8,174) | Plan 36 P-E4; ADR-0175 (10-11); blueprint rev 150 | `677394474` 10-12; 2 commits total |
| 3 | `crates/pse-runtime/src/math/strategy/admission.rs` (853 lines): `TaskAdmission` ledger, `WorkAdmission` trait (declared in `pse-backend-native::solve`) | per-task native work/storage accounting | `WorkAdmission` `1837df6d9` 10-04; `TaskAdmission` `72e589a27` 10-04 | Plan 25n; rev 119/121/104 | 4 commits; last `991fcea19` 10-10 (Plan 34) |
| 4 | `crates/pse-backend-native/src/execution.rs` + `execution/` (1,238 + 7,142 lines): adapter "admission of its settings", static backend table; trait `ComputeAdmission` | backend settings/compute admission | execution seam `181827349` 09-27 (Plan 22 A2); `ComputeAdmission` `2410880ef` 10-08 | Plan 22 A2/A5; ADR-0105; Plan 28 parallel (ADR-0166, 10-08) | 70 commits to `execution/`, 40 to `execution.rs` since 09-25 |
| 5 | `crates/pse-columnar` `MemoryPool`/`AllocationLease`/`CpuAdmission`/`CancellationToken` (own types) | process-local capacity and charge | `AllocationLease`, `CpuAdmission` `1407716fc` 09-24; `MemoryPool`/`CancellationToken` `d375a6a00` 09-14 | ADR-0039; later charge rules: rev 74 (09-29, Plan 22 Q1), rev 80 (Plan 23 H1f/H9), rev 104 (10-04), ADR-0166 rev 136 (10-08), ADR-0176 rev 151 (10-11) | commits `5a2199969`, `85db48f2b` 09-29; `00f477260` 10-12 |
| 6 | `crates/pse-operations/src/canonical_lifetime.rs` (532 lines): caller admission, admitted continuation, client-pair teardown | logical caller admission vs physical connection | `b8a2756ee` 2026-10-12 | Plan 37 B1; ADR-0181; rev 157 | 3 commits, all 10-12 (`20f4d29fa`, `1489351f4`, `26af8757b`) |
| 7 | `vendor/surrealdb` patch (`PATCH.md`): `Config::bounded_requests()`: 32 application + 2 reserved control admissions, 34 queued/deferred routes, 64 replay entries | per-connection request admission and replay budget | `d0f2c4181` 2026-10-09 ("plan 30 finished") | Plan 30 / 30a; ADR-0164/0166; rev 138 | `32d73ed0c` 10-12 (finite restoration, B2/B3, ADR-0181); `da9a2cc8b` 10-12; `33a0c7863` 10-12; vendor tree 32,901 Rust lines |
| 8 | `scripts/host_admission.py` (1,840 lines) + `.config/agent-capacity.toml` (91 lines) | host lanes / memory partitions bound to supervised units | `d0f2c4181` 2026-10-09 | Plan 30 / 30d; Plan 29 B1-B4 (`26da21318`, `348f6b92b` 10-07) provide `scripts/pse-env`/`pse.slice` | 7 commits, last `677394474` 10-12 |
| 9 | `scripts/native_operation.py` (2,068 lines): "native setup admission owned by one process operation and child scope" | per-operation native child scope, cancellation fences | `dacc9c332` 2026-10-07 | Plan 28h; Plan 28e continuation 10-10 | 18 commits to 10-12 |
| 10 | `scripts/test_resources.py` (1,711 lines) + `scripts/resource_store.py` (734 lines) | fixture-lifetime registration, "service context admission" | `d0f2c4181` 10-09; `resource_store.py` `ff5a4aa2f` 10-11 | Plan 30c; ADR-0174 (exact resource metadata, 10-11) | 8 and 2 commits |
| 11 | `scripts/surreal_server.py` (7,513 lines; 234 "admission" word hits): service admission, quiesce/stop --drained | store service/primary/observer admission | `0b4abf7c6` 2026-10-06 | Plan 28a; Plan 30b/d; Plan 35 | 24 commits since 10-06 |
| 12 | `crates/pse-operations` canonical `DeclaredStudyAdmission`, `StudyDispatchAdmission`, `SelectedAdmission`, `PhysicalAdmissionKey`, `ReplayAdmission`, `canonical_admission_unit` | study / selected-source / replay admission in the canonical store | `SelectedAdmission` `dacc9c332` 10-07; `ReplayAdmission` `84a1caf17` 10-08; `StudyDispatchAdmission`+`DeclaredStudyAdmission` `219338742` 10-06; `PhysicalAdmissionKey` `d92fa8ee1` 10-09 | Plan 28b/28c; ADR-0148 (10-01) for study contract | within the canonical_* files (s3) |

Also named "admission" but in the domain-modelling sense (physical quantity/checked occurrences), not lifecycle: ADR-0039 (09-14),
ADR-0150 "checked admission" (10-01), `pse-quantity`, `pse-modeling`. Not counted above.

Terminology count at HEAD (word hits, not mechanisms): `admission` 234 in `surreal_server.py`, 83 in `host_admission.py`, 68 in
`native_operation.py`, 32 in `canonical_studies.rs`, 30 in `canonical.rs`, 63 in blueprint s20 text.

## 3. Lifecycle families: provenance table

Owner files are in `crates/pse-operations/src/` unless prefixed. HEAD lines in brackets.

| Family | Mechanism at HEAD | Introduced (plan / ADR / commit / date) | Later modified (by plan, date) |
|---|---|---|---|
| **Fences** | `canonical_guards`, `canonical_reclaimed_ranges` ("range markers fence reopening"), `AttemptFence` (`canonical_execution.rs` [4,934]; word `fence` 306 hits) | guards, ranges, `0b4abf7c6` 10-06 (Plan 28a); `AttemptFence`, `06302af77` 10-06 (28c) | source-creator/result-creator fences and `fn::pse_execution_v1::fence` ff5a4aa2f 10-11 (Plan 36 P-B1/B8, ADR-0173, 0177); "exact terminal seal ... without writing the shared source-creator fence" blueprint rev 156 (ADR-0177, 10-12); permanent owner/identity fences `aef274db9` 10-12 (Plan 37 A3, ADR-0182); released-owner fences `fa969bffb`, `0d82a2871` 10-12 |
| | Receiver/service generation fences (`scripts/surreal_server.py`, `resource_store.py`) | Plan 30b receiver generations, `d0f2c4181` 10-09 | ADR-0174 resource metadata + terminal freeze (rev 149, 10-11); `resource_store.py` `ff5a4aa2f` |
| | Pre-existing: `RetentionFence` (`crates/pse-columnar/src/retention.rs`, in-process) | `1407716fc` 09-24 | none found in window |
| **Leases** | DB leases: `canonical_stages.expires_at` (leased immutable source staging), attempts, `fn::pse_execution_v1::lease/renew/touch` (`canonical.rs` lease hits 111; `canonical_staging.rs` 43; `canonical_selection.rs` 42) | stages/protections `0b4abf7c6` 10-06; attempts `06302af77` 10-06; `fn::...::lease` `d0f2c4181` 10-09 | `219338742` 10-06; ADR-0166 10-08; "final lease clocks" `15a27c30f` 10-12; artifact leases `9db3fef8c` 10-12 |
| | Predecessor: PostgreSQL job leases `LeasePolicy`, `lease_expires`, heartbeat | `9ac684ac2` 09-27, `0157e3857` 09-27 (Plan 22 O1-O3) | deleted `06302af77` 10-06 |
| | Host leases: `scripts/host_admission.py` ledger "lanes bound to supervised lifetimes" | `d0f2c4181` 10-09 (Plan 30d) | 7 commits to 10-12; ADR-0176 (quiescent admission, 10-11) |
| **Receipts** | `canonical_version_receipts` ; `CompletionReceipt` ; `canonical_selection_receipts` ; `SelectionDependencyReceipt` ; `AppendReceipt` ; `fn::...::seal_ack/close_acknowledged` ; `canonical_maintenance_activations` (activation receipts, `canonical_maintenance.rs` [505]) | version receipts `0b4abf7c6` 10-06; `CompletionReceipt` `06302af77` 10-06; `SelectionDependencyReceipt` `d92fa8ee1` 10-09; selection receipts, `AppendReceipt`, `seal_ack`, `close_acknowledged` `ff5a4aa2f` 10-11 (ADR-0180); activation receipts `9db3fef8c`/`8f5a5a7d4` 10-12 (Plan 37 C2) | Pre-window: `StartReceipt` `1601b757f` 09-25 (Plan 16 checkpoint). Scripts: native_cache/producer receipts (`producer_deployment.py` `5260a3e9a` 10-07; Plan 28h) ; allocation-recovery receipt `build/plan35-mcp-20261010/allocation-recovery.json` (Plan 35) |
| **Retention and reclaim** | `canonical_retention.rs` [868] roots/protections/range reclaim ; `canonical_result_retention.rs` [666] ; `fn::pse_retention_v1::*` (6) ; `canonical_result_retirements`, `canonical_study_retirements` | `canonical_retention.rs`, `0b4abf7c6` 10-06 (11 commits since); `canonical_result_retention.rs`, `06302af77` 10-06 (12 commits); retention fns `06302af77` | ADR-0172 full analysis retirement `991fcea19`/rev 144 10-09 (Plan 33); `settle_protection` ff5a4aa2f 10-11; artifact `collect` `9db3fef8c` 10-12; history `collect` `aef274db9` 10-12 |
| | Predecessor: PostgreSQL `retirement.rs` ("an interrupted retirement resumes") | `93161d643` 09-28 (Plan 22 O8.7) | deleted `06302af77` 10-06 |
| | Pre-window: Delta-era retirement/tombstone | `a46f358bd` 09-18 (Plan 9) | `pse-catalog` deleted `06302af77` |
| **Settlement** | `fn::pse_study_v1::settle`; operation identities (`canonical_execution_operations`); ADR-0177 "settled retirement"; `ArtifactSettlement` ; rollout settlement (`scripts/surrealkit_maintenance.py` [1,263; 50 "settle" hits]) | operations `06302af77` 10-06; ADR-0177 + rev 152 10-11 (Plan 36 P-B1/B7); `ArtifactSettlement` `f50672d70` 10-12; rollout settlement `84757f8b6` 10-12 (Plan 37 C2) | `15a27c30f`, `776aaf4b7` fixes 10-12; Plan 30 "cancellation settlement used stale predecessor premises" fixed in `d0f2c4181` period |
| **Drain and teardown** | Native child-scope drain `scripts/native_operation.py` (53 drain hits); service `quiesce`/`stop --drained` `scripts/surreal_server.py` (92 hits); SDK physical socket drain (`vendor/surrealdb` PATCH.md); `canonical_lifetime.rs` (15 hits); `ProbeDrain`/`ResultReadDrain`/`StudyProbeDrain` structs; `TaskTracker` (only `canonical_results.rs`) | native_operation `dacc9c332` 10-07; SDK drain `d0f2c4181` 10-09; `TaskTracker` first appears `7c269f4a5` 10-10 (Plan 35); caller/physical split `b8a2756ee` 10-12 (ADR-0181) | Plan 28e continuation 10-10/10-11 (foreground observer handoffs, listener descriptor race); `3ea9e6634`, `658711f76`, `20f4d29fa`, `ef1c29b1e` 10-12 |
| **Cancellation** | `fn::pse_execution_v1::cancel`, `study_cancelled`, `summary_cancelled`; `StudyCancel` (`pse-runtime/src/workflow/study.rs`); Python `native_operation.py` (140 "cancel" hits) ; pse_columnar `CancellationToken` (own type) | `cancel_run` `06302af77` 10-06 (28c); `StudyCancel` `79661d4dd` 09-28 (Plan 22 O7.1); PostgreSQL listener cancel `3dfb5764f` 09-28; `CancellationToken` `d375a6a00` 09-14 | Plan 30a submission/cancellation/settlement (`d0f2c4181`); ADR-0166 10-08; Plan 34 SM06/SM09 "typed exact-attempt cancellation refusal" `991fcea19` 10-10; Plan 36 "cancellation alone writes its exact authority" ADR-0177 |
| **History release** | `canonical_history.rs` [722]; `fn::pse_history_v1::*` (20); `canonical_artifacts.rs` [780]; `fn::pse_artifact_v1::*` (13) | `aef274db9` 10-12 (Plan 37 A3; ADR-0182; rev 158/159); artifacts `9db3fef8c` 10-12 (A1/A2) | `fa969bffb`, `0d82a2871`, `1aebcdf58`, `52b0caf1e`, `20fa7d2b2`, `459d8ab77` 10-12; predecessor Plan 36 historical-activation (`canonical_preparation_activation*`, `historical_results.rs`) `ff5a4aa2f` 10-11, **deleted** `25f0e53a6` (-6,883 lines) and `bd0c169b4` (-1,330) on 10-12 under OP01 "fresh-store cutover" |

Counts at HEAD: `fn::pse_*_v1` DEFINE FUNCTION = 94 (execution 32, study 18, retention 6, analysis 5, artifact 13, history 20); DEFINE TABLE 57;
DEFINE FIELD 442; DEFINE INDEX 139; DEFINE EVENT 0 (all in `surreal.surql`, 2,428 lines). First appearance of each function namespace:
execution/study/retention/analysis `06302af77` (10-06); artifact `9db3fef8c` (10-12); history `aef274db9` (10-12).

Observed lifecycle code at HEAD: `crates/pse-operations/src` non-generated 34,776 Rust lines (4,745 in `*test*`/`testing`); seven lifecycle files
(`canonical_{lifetime,transport,retention,result_retention,maintenance,history,artifacts}.rs`) total 4,436; generated `surreal.rs` 4,417;
generated runtime `canonical_*` relation files 32,357; vendored SDK 32,901 Rust lines; lifecycle scripts (`surreal_server`, `host_admission`,
`native_operation`, `test_resources`, `resource_store`, `surrealkit`, `surrealkit_maintenance`, `surreal_rollout_producer`,
`producer_deployment`, `test_run`) 17,491 Python lines; `scripts/tests` 43,503 lines. Whole workspace non-generated Rust: 608,336 lines.

Context fact: these scripts and vendor files are shared between product runtime support and the agent/test environment; the Plan 30 title is
"WebSocket RPC and the persistent agent environment" and Plan 29 is "agent workspace effectiveness". The files themselves do not separate the two.

## 4. The 57 `canonical_*` runtime relations: introduction

Declared in `crates/pse-schema/src/catalog/substrate.rs` (938 lines), generated to
`crates/pse-relations/src/generated/runtime/`. Introduction = first commit containing the quoted name (`git log -S` in `crates/pse-schema/src`).

| Commit | Date | Plan / ADR | n | Relations |
|---|---|---|---|---|
| `0b4abf7c6` "plan 28a and 28b mostly implemented" | 10-06 | 28a/28b, ADR-0164 | 16 | edges, guards, interpretations, memberships, payload_blocks, problems, products, protections, reclaimed_ranges, revisions, roots, staged_edits, stages, versions, version_manifests, version_receipts |
| `06302af77` "plan 28 done" | 10-06 | 28c/28d | 21 | analyses, analysis_edges, analysis_inputs, analysis_nodes, attempts, execution_operations, problem_runs, result_batches, result_block_outputs, result_blocks, result_manifests, result_protections, result_retirements, result_seeds, result_sets, run_sources, runs, studies, study_dependencies, study_points, study_retirements |
| `ff5a4aa2f` "first wave of plan 36 implementation" | 10-11 | Plan 36 (ADR-0173..0180) | 7 (+1 later removed) | body_contents, product_associations, product_descriptors, result_rows, selection_receipts, study_readiness, study_readiness_cursors; also `canonical_historical_results` (removed `bd0c169b4` 10-12) |
| `9db3fef8c` "Own copied scientific artifacts..." | 10-12 | Plan 37 A1/A2, ADR-0182 | 7 | artifact_blocks, artifact_obligations, artifact_reads, artifacts, artifact_staging_inputs, maintenance_activations, study_inputs |
| `aef274db9` "release scientific history with permanent owner fences" | 10-12 | Plan 37 A3, ADR-0182 | 5 | history_identities, history_locators, history_obligation_counts, history_obligations, history_retirements |
| `fa969bffb` "preserve cross-owner history and fence released seed reads" | 10-12 | Plan 37 | 1 | artifact_releases |

Trajectory by count (names in `substrate.rs`): 16 (10-06 a.m.) -> 37 (10-06 p.m.) -> 45 (10-11, including `canonical_historical_results`)
-> 52 (10-12, `9db3fef8c`) -> 58 -> 57 after `bd0c169b4`. So 37 of 57 (65%) date to the two 10-06 commits and 20 to 10-11/10-12.

## 5. Narrative 2026-09-25 to 2026-10-12 (facts; commit counts per day: 09-25:3, 09-26:2, 09-27:68, 09-28:155, 09-29:144, 09-30:47, 10-01:14, 10-02:4, 10-03:1, 10-04:12, 10-05:9, 10-06:5, 10-07:25, 10-08:2, 10-09:5, 10-10:5, 10-11:3, 10-12:66)

- 09-27/28 (Plan 22 solver scope + operational store): PostgreSQL 18 store and `pse-operations` crate created; durable attempts, leases,
  heartbeats, cross-process cancel, durable studies "publish once", durable incumbent resume, publication catalog; registry-generated
  schema (ADR-0114). Same days: nightly pin and feature unification (ADR-0122); 11 `fix(build-env)` commits on target-dir and compiler cache.
- 09-29/30 (Plan 22 Q1/closure, Plan 23 domain model, Plan 24 thermo knowledge): in-process charging rules extended (`5a2199969`, `85db48f2b`);
  `scripts/memory-cap.sh` added then retired (`5ac320ff7`, `28f722192`; replaced by `scripts/pse-env` 10-07).
- 10-01..10-05 (Plans 25a-25n, ADR-0142..0163): numerical strategy and preparation; ADR-0159 float policy (10-04) superseded by ADR-0165
  (10-06). `TaskAdmission` ledger 10-04. Execution-fit principles ADR-0162 (10-05).
- 10-05/06: ADR-0164 unified SurrealDB substrate. `0b4abf7c6` and `06302af77` add the canonical substrate (37 relations, 4 function
  namespaces) and delete the PostgreSQL store, `pse-catalog` and `vendor/delta-rs` (730 files, +270,305/-141,792 in `06302af77`).
- 10-07..10-09: Plan 29 (agent environment, `scripts/pse-env`, `pse.slice`, `just activity`), Plan 28 companions 28f-28k,
  ADR-0166 (10-08), Plan 30 (`d0f2c4181`, 237 files +60,878): bounded WebSocket SDK patch, receiver generations, host admission.
- 10-09..10-10: Plan 33 (ADR-0169..0172; `91dc7fdf1`, 416 files), Plan 34 (`991fcea19`), Plan 35 (`7c269f4a5`): relation-local transport, analysis
  retirement, completion-owned protected reads, typed readiness, exact cancel settlement, `TaskTracker`.
- 10-11 (Plan 36 first wave `ff5a4aa2f`, 643 files +90,771/-12,147): ADR-0173..0180; adds `pse-admission` crate, readiness cursors,
  acknowledged close/seal, resource metadata and terminal freeze, historical-activation machinery.
- 10-12 (66 commits): `677394474` Plan 36 continuation; Plan 37 (ADR-0181/0182, revs 157-159): caller/activity/physical lifetimes,
  finite WS restoration, SurrealKit beta.6 rollouts and activation receipts, copied artifacts, history release; deletion of Plan 36's
  historical-activation chain (OP01 "hard pivot ... existing data and evidence need not survive"); close `6c368c88a`.
- Plan 28 stayed `in-progress` throughout (E3/E4/E5 open); Plan 36 stopped by the maintainer 10-12 (six plans `in-progress`, queue `not_run`);
  Plans 30, 33, 34, 35, 37 `done`; Plan 32 `in-progress`.
- Blueprint revisions in the window: rev 60 (09-27) to rev 159 (10-12) = 100 revisions in 16 days; revisions 128-159 are the substrate/lifecycle ones.

## 6. Classification of fixes (every `fix`-prefixed commit since 2026-09-28 00:00: 65 of 497 commits = 13%)

Classification is by the commit subject and the plan text for that change; where a subject fits two categories the primary is stated.
Static-check repairs (lint, type, clippy, format) are counted under build/codegen/toolchain; they could equally be a separate category.

| Category | n | Representative commits |
|---|---|---|
| Scientific or numerical | 12 | `f4792b126` epigraph reports objective at the function value; `cb96b40fe` Diffsol adjoint start; `f12e18f29` SCIP rows in normalized coordinate; `2706f5df5` projected outputs bind only members read (H8); `94db756a2` declared-deviation admission rule (S3); `0589896b1` seed fixtures pass at root cause (H1f); `52d0cc2de` HiGHS acceptance coordinates; `541511730` typed refused native reuse; `b65711fb8` conditional comparisons compile without symbolic blow-up; `598b82ef1`; `94c1970eb`; `baf7ad941` |
| Lifecycle, coordination or retention | 14 | `85db48f2b` charge foreign allowance only while native runs; `5a2199969` charge compiler jobs/fit layouts/session workers; `8ba49bce5` finalization finding study committed succeeds; `b8a2756ee` separate caller admission from physical drain; `3ea9e6634` join native scope terminal cleanup; `658711f76` first signal bounded by cleanup clock; `15a27c30f` artifact settlement identity and final lease clocks; `fa969bffb` cross-owner history and released seed reads; `927e881ce`, `2eee73fca`, `b8c2dd889`, `776aaf4b7` rollout phase/witness/order; `459d8ab77` typed history documents; `00f477260` charge boxed study/publication futures |
| Transport or connection | 1 | `32d73ed0c` bound websocket restoration and drain local request owners |
| Test fixture or harness | 7 | `f1633496e`, `2bbb97fed` type rollout/maintenance fixtures; `548907301` retry test DB drop refused by background worker; `f9075292f` recycle fixtures declare initialization allowance; `1de7093de` self-referencing FK fixtures; `765e8b6d6` Q1 native suite findings (host env, stale tests, recursion limits); `a2717d701` |
| Build, codegen or toolchain (incl. lint/type repairs) | 29 | 11 `fix(build-env)` (`d159b3ee6` ... `b3fa31414`, 09-28: per-checkout target dir, shared compiler cache); `e7dc1183d`, `42a386fe6` (10-12 recovery consumers/codegen); `26af8757b`, `33800cff7` (10-12 Plan 37 lint repairs); `55bf8f1f7`, `0acf0db73`, `83a1a2553`, `65b649c65`, `bbc2bf059`, `b548e97da`, `6af8ae28f` (lint/clippy/REUSE); `9e73ac8cf` regenerate frame catalog; `0a8152e64` hakari; `b2ec988a8` recursion limit; `9cab2ca21` doctor; `50f5dc29b`; `28665c3e0`, `dd0a9f326` (thermo-knowledge tooling) |
| Documentation | 2 | `6483b4ab5` finalize handoff and pivot plans; `7090881e6` survey statements |

Other typed commits since 09-28 (497 total): feat 138, untyped 111, docs 70, merge 34, test 18, chore 12, style 8, design 7, refactor 6, adr 6,
tooling 4, bench 4, perf 2 (`c839613a9` KKT, `f0c72fc0b` Clarabel; scientific-performance, not counted above).

By date, the 14 lifecycle + 1 transport fixes: 12 on 10-12, 2 on 09-29, 1 on 09-28. The 12 on 10-12 fall inside the Plan 37 day and several follow
same-day features (e.g. `1fcaced0d` feat, then `b8c2dd889`/`2eee73fca`/`776aaf4b7` fixes of rollout phase and order).

Fix text embedded in plan Outcomes (not separate `fix` commits) is large: plan 28e records fixes under "Assembled continuation corrections",
"Post-Plan-35 diagnostic and corrections", "Validation and stopped campaign" (the bulk commits that carry them are untyped).

## 7. Failed or interrupted acceptance attempts (from plan Outcome / checkpoint text)

Sources: 28e (`docs/plans/28e-rebuild-retirement-and-qualification.md` lines ~766-1340), 28 coordinator checkpoint, Plans 30, 33, 34, 35,
36 (+36a-e), 37. No Plan 28/30/33/34/35/36/37 text was found recording a *passing* comprehensive assessment. The "category" is the plan's own
stated defect class where it gives one; "plan-classified" otherwise my reading. Counts below are of enumerated attempts (n=34), not an
exhaustive census (28e alone has 68 uses of "failed").

| # | Attempt (plan, date) | Outcome as recorded | Category |
|---|---|---|---|
| 1 | 28e `plan28-e3-unified-state-qualified-20261010` setup capture | 754 tests, 8 failed, 3 errors; exit 130; ten stale-fixture explanations + one unresolved nested-observer drain | fixture/harness (10) + lifecycle (1) |
| 2 | 28e `...creator-completion-listener-fixed-20261010` native gate | 3,114 run, 3,105 passed, 9 failed, 8 skipped, exit 100; cancellation, conformance, caller-drop, claim-expiry | lifecycle |
| 3 | 28e interrupted comprehensive capture (10-10) | 3,114 selected; 2,608 passed, 495 failed (post-interruption admission failures); result-owner fixture failed first | lifecycle/harness |
| 4 | 28e follow-up capture | 709 setup tests pass; native 2,483 passed, 485 failed, 146 not run; transient lifecycle reservation refused fixture registration | lifecycle |
| 5 | 28e `plan28-e3-final-plan35-20261010` | 39 gates passed; native 3,133/3,160 passed, 10 failed, 1 interrupted, 16 not run; Python 193/210, 4 failed, 13 incomplete; deliberately interrupted | lifecycle + fixture |
| 6 | 28e targeted composite (10-11) | 20/21 passed; fitting-clone-owner test hit masked full-pool assertion | fixture |
| 7 | 28e `plan28-e3-final-corrections-20261011` | 37 gates passed; Python lint 4, typing 2; native compile `No space left on device`; exit 120 | build/environment |
| 8 | 28e `just governance` (10-11) | 26/26 governance passed; generation compile exhausted storage; outer exit 125 | build/environment |
| 9 | 28e 10-11 host-restart capture | 39 gates passed; cancelled during native test compilation; exit 130 | build/environment |
| 10 | 28e `plan28-e3-execution-20261011` | 41 passed, 1 failed, 1 blocked; native gate failed in terminal fixture cleanup "Service context admission is closed"; managed native blocked | lifecycle/harness |
| 11 | 28e seed conformance (same run) | 118 fixtures: 98 passed, 9 failed, 11 inconclusive (derivative suspicious entries 4 fixtures, closure failures 4, preparation deadline 9, resource limit 1, derivative evaluation 1); cancelled while active; powerset, recovery, repaired assessment `not_run` | scientific/numerical |
| 12 | 34 first broad native selection | 20 tests: 17 passed, 2 failed, 1 timed out (356 s) | harness/lifecycle timing |
| 13 | 34 first `Blocks::attempt` fixture | omitted authored start/bounds | fixture |
| 14 | 35 managed tests / lifecycle composition | recovery probe DB left in backup; borrowed affiliation left in host allocation; receiver dirs created with permissive umask | lifecycle/tooling |
| 15 | 30 `just hygiene` | six failed leaves (typos, license, Python lint, type, 2 clippy) | build/static |
| 16 | 30 `just governance` | 101/102; root-override prohibition failed until SDK paths selected directly | build/toolchain |
| 17 | 30 first native lint, `cargo hack` | killed by 2 GiB cap routing; 28 invalid cross-package feature requests (harness selection errors) | harness |
| 18 | 30 timing store | startup deadline: storage readiness treated as science-placement readiness | lifecycle/placement |
| 19 | 30 assembled | libtest stack too small for a large async publication frame; KLU metadata in receiver closure; stale predecessor premises in cancellation settlement | lifecycle/build |
| 20 | 33 broader runtime command with reference cases | failed (single projection control passed separately) | scientific/fixture |
| 21 | 36 first ordinary-native assessment | terminal-resource projection: one 5 s clock covered 788 members, expired after a 208-member prefix | lifecycle |
| 22 | 36 ordinary-native assessment (later) | 3,455 passes, 5 failures: fixtures, SDK response-handler payload-grant race, cold-producer, release-metadata fetch before worker | fixture + transport + environment |
| 23 | 36 selected native Python | 45 passed, 3 failed (binding digest form, reload order witness, sweep accounting control) | boundary/fixture |
| 24 | 36 runtime campaign | double-charged prepaid constructor envelopes; readiness-recovery loop starved study discovery; fixture borrowers outlived test process so allocation release failed | lifecycle |
| 25 | 36 first general-history activation journey | failed at final service startup; fixture preserved | lifecycle |
| 26 | 36 full setup run | cancellation-cleanup race orders (repeated owner signal; reaping before descendant drain); 940 setup controls later pass | lifecycle |
| 27 | 36 managed Rust | 3/4 journeys; fixture bound 8 pages vs 9-page retirement protocol | fixture |
| 28 | 36 managed Python (two attempts) | 6 tests then worker stopped on stale recovery hint (1,000-point sweep); later attempt 6 tests then worker failure and interruption; stop checkpoint 10-12: seal commutation repair, queue `not_run` | lifecycle |
| 29 | 36 `just turn-end` at stop | first invocation refused report registration (default resource registry needs fenced migration recovery); isolated registry rerun passed | lifecycle/tooling |
| 30 | 37 historical foundation `just turn-end` | 24 trailing-whitespace errors; Ruff diagnostics not a clean lint result | build/static |
| 31 | 37 native-observer route | compiled but ran no tests (CPU width did not fit inherited lane) | harness |
| 32 | 37 beta disposal helper | selected the namespace it was removing; SQL shell recreated it | lifecycle/tooling |
| 33 | 37 managed artifact fixture | exposed nested placement widening observer cap; large inline futures exhausted worker stack; fixture used semantic labels as graph endpoints | lifecycle + fixture |
| 34 | 37 C-H whole-site link audit | failed outside the affected inputs | documentation |

Counts by primary category (rows with two classes counted once under the first-named): lifecycle/coordination/retention 16 (rows 2-5, 10, 14, 18, 19,
21, 24-26, 28, 29, 32, 33); test fixture/harness 9 (rows 1, 6, 12, 13, 17, 22, 23, 27, 31); build/toolchain/environment/static 6 (rows 7-9, 15, 16, 30);
scientific/numerical 2 (rows 11, 20); documentation 1 (row 34); transport 0 primary (secondary in row 22). Several rows
carry a second class (1, 5, 22, 23, 33). The scientific/numerical rows (11, 20) are the only attempts whose recorded failure concerns
solver/derivative/closure results; the 9 failed + 11 inconclusive seed fixtures are the sole scientific-class failure set recorded for the
10-11 comprehensive campaign.

Open at HEAD (plan text): Plan 28 E3/E4/E5 (E4: 37 selected cases and build/locality measurements unexecuted); Plan 36 P-G1 and its finding
dispositions; F20 sparse rank/fill limit; F28 host-policy investigation.

## 8. Churn since 2026-09-28 00:00 (`git log --no-renames --numstat`; 497 commits; additions/deletions)

Totals: non-generated +1,415,931/-1,059,865; generated +341,753/-132,576. Tool: `/tmp/.../scratchpad/review/tmp/agg.py` (scratch).
Large non-lifecycle movements: `thermo-knowledge/` +216,330/-216,330 (moved out 10-01, `be469cc97`); `thermodynamics_complete_conversation_handoff/`
+189/-425,408 (deleted); `vendor/delta-rs` +2/-107,255 (deleted 10-06); `docs/design_review/evidence` +305,412/-541.

| Top-level | non-gen +/- | gen +/- | Role (my reading from plans/commits) |
|---|---|---|---|
| `crates/pse-runtime` | +208,708/-64,857 | +1,306/-734 | scientific orchestration; `src/workflow` +139,875/-42,316 (modeling +49,670/-11,102; fitting +9,450/-2,683; durable.rs +7,618/-3,096; study.rs +5,198/-2,991; worker.rs +3,500/-2,180); `src/math` +60,760/-10,619 (solves +21,227) |
| `crates/pse-operations` | +85,165/-57,258 | +10,704/-3,847 | store/lifecycle (PostgreSQL deleted, canonical added): `canonical_execution.rs` +7,006/-2,072; `canonical_selection.rs` +6,238/-765; `canonical.rs` +5,632/-963; `canonical_studies.rs` +5,225/-1,489; `canonical_staging.rs` +4,932/-1,377; `canonical_results.rs` +2,664/-723; deleted-era: `store_tests.rs` +2,710/-4,332, `catalog.rs`, `attempts.rs`, `jobs.rs`, `studies.rs`, `schema.rs`; `migrations` +11,066/-11,477; `test-fixtures` +5,579/-5,579 |
| `crates/pse-operations-queries` | +15,007/-14,999 | 0 | generated-style PostgreSQL statement crate, then removed (not in `crates/` at HEAD) |
| `crates/pse-catalog` | +6,294/-29,578 | 0 | Delta publication catalog; deleted 10-06 |
| `vendor/*` | +45,220/-107,930 | 0 | `delta-rs` -107,255; `surrealdb` +39,880/-530; `surrealdb-engine-api` +5,308/-145 |
| `crates/pse-relations` | +4,888/-1,268 | +134,935/-47,051 | generated relations (all `canonical_*` among them) |
| `crates/pse-model` | +6,263/-1,468 | +69,236/-28,469 | generated runtime facts (+ `study_policy` relocated 10-12 `6255da3a9`) |
| `crates/pse-quantity` | +7,580/-1,825 | +28,822/-31,749 | unit algebra (Plan 23) |
| `crates/pse-backend-native` | +78,153/-12,238 | 0 | scientific/numerical: `src(top)` +37,600/-7,720; `dynamics` +8,859/-1,517; `kkt` +6,769/-313; `execution` +4,908/-780; `petsc` +3,612/-86 |
| `crates/pse-modeling` | +44,984/-9,986 | 0 | scientific/modeling (Plan 23 KR1-KR9); `specialize` +8,845/-1,951 |
| `crates/pse-math` | +43,900/-5,584 | 0 | scientific/numerical |
| `crates/pse-compiler` | +22,054/-4,840 | 0 | `workspace` +14,453/-2,167 (preparation/compilation) |
| `crates/pse-schema` | +11,385/-4,828 | 0 | registry; `substrate.rs` declaration |
| `crates/pse-admission` | +7,758/-4 | +416/0 | new 10-11 (lower admission) |
| `crates/pse-codegen` | +6,462/-1,935 | 0 | generators incl. `surreal_*` |
| `crates/pse-py` | +5,622/-4,058 | 0 | Python boundary |
| `crates/pse-engine` | +3,823/-1,188 | 0 | DataFusion engine |
| `crates/pse-columnar` | +725/-74 | +320/0 | capacity/lease primitives |
| `crates/pse-petsc-sys`/`pse-uno-sys`/`pse-ipopt-sys` | +2,208 / +1,164 / +839 | 0 | native bindings (new: petsc, uno) |
| `scripts` | +85,289/-12,586 | 0 | `tests` +47,479/-5,846; `surreal_server.py` +8,723/-1,210; `native_operation.py` +2,395/-327; `test_resources.py` +2,208/-497; `host_admission.py` +1,972/-132; `surrealkit_maintenance.py` +1,617/-354; `pse_env.py` +1,340/-215; `native_tests.py` +1,262/-308; `library_catalog_hints.py` +757/0; `compiler_cache.py` +704/0 |
| `python` | +14,016/-5,467 | +17,270/-3,797 | `pse/contracts` generated +16,733/-3,123; tests +9,603/-2,958 |
| `docs` | +397,930/-34,907 | +72,174/-10,380 | evidence +305,412; plans +34,359/-24,490; reviews +25,679/-3,993; adr +8,104; authoritative_design +6,711/-2,766 |
| `xtask`, `benches`, `packages`, `.config`, `.codex`, `.claude`, `justfile` | +12,601 ; +6,269 ; +14,626 ; +2,917 ; +1,653 ; +423 ; +1,097 | - | tooling/bench/reference packages |

Grouping by the brief's categories (module role, approximate, from the table): lifecycle/coordination/retention/transport sources =
`pse-operations` (non-generated +85,165/-57,258), `pse-admission` (+7,758), `vendor/surrealdb*` (+45,188/-675), lifecycle scripts
(`surreal_server`, `host_admission`, `native_operation`, `test_resources`, `resource_store`, `surrealkit*`, `producer_deployment`, `test_run`:
about +18,000/-3,400 by the per-file figures above) and their tests (`scripts/tests` +47,479/-5,846); scientific/numerical = `pse-backend-native`,
`pse-math`, `pse-modeling`, `pse-compiler`, `pse-runtime/src/math`; test fixture/harness = `scripts/tests`, `pse-operations/test-fixtures`,
`store_tests.rs`, `python/pse/tests`; build/codegen = `pse-codegen`, generated trees, `xtask`, `justfile`, `.config`.

Selected single-commit magnitudes: `06302af77` 730 files +270,305/-141,792; `ff5a4aa2f` 643 files +90,771/-12,147;
`d0f2c4181` 237 files +60,878/-4,019; `91dc7fdf1` 416 files +35,522/-14,296; `991fcea19` 97 files +25,251/-1,071;
`0b4abf7c6` 304 files +41,408/-2,991; `5b864e1c5` 165 files +21,274/-4,863; `677394474` 204 files +17,470/-2,533;
`9db3fef8c` 52 files +6,778/-415; `aef274db9` 33 files +4,815/-113; `b8a2756ee` 69 files +1,865/-119; `32d73ed0c` 9 files +760/-130;
deletions `25f0e53a6` -6,883, `bd0c169b4` -1,330.

## 9. Material uncertainties

- Dates and "later modified by" for 10-06..10-11 rest on coarse bulk commits; per-mechanism intra-day order is not recoverable from git.
- Fix classification used subjects and plan text; commits whose messages are untyped (111) are not classified individually.
- The attempt list is extracted by reading, not by a mechanical census; the 34 rows are those with explicit counts or named causes.
- `CancellationToken`/`MemoryPool` in `pse-columnar` are the project's own types; tokio-util `TaskTracker` appears in one file (`canonical_results.rs`).
  `JoinSet`, `Semaphore` hits (10 and 17 files) were counted by name only and not confirmed as tokio types.
- Whether any specific lifecycle mechanism is consumed by a product feature (as opposed to the test/agent environment) is W-B's question and is not answered here.
