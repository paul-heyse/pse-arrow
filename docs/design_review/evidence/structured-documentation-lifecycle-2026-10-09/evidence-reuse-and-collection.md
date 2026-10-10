# C. Evidence as reusable answers: validity, lifecycle and collection machinery

**Role and baseline.** Code-mapper (read-only) report for the reference-content design review.
Baseline: dirty `main` at `4c24721e691187e1a5b28398b29722fbde671da8`, observed 2026-10-09/10.
`docs/design_review/evidence/efficiency-principles-codebase-2026-10-09/` and
`.../workspace-content-lifecycle-2026-10-09/` are untracked; `scripts/validation_scope.py`,
`validation.py`, `validation_receipts.py`, `test_resources.py` carry uncommitted Plan 33/32 edits.
These are facts and observations; no architectural verdict. **Scope:** per maintainer direction,
the primary subject is evidence produced for design reviews and plans (docs/design_review/evidence
and review/plan-commissioned campaigns under `build/`). Capability-map evidence and product
qualification receipts/resource ledger are treated as separate categories (§1.B, §1.C).
Library skills (the `~/.local/share/library-skills` store and its probes) are out of scope and
are not analysed here; repository documents that cite them are noted only where that citation is
the repo-side condition of an answer.

---

## 0. Three categories, how they differ

| | A. Review/plan evidence (primary) | B. Capability-map evidence | C. Qualification receipts / resource ledger |
|---|---|---|---|
| Location | `docs/design_review/evidence/<topic>-<date>/` (14 dirs + 2 loose md, ~20 MB, tracked except 2 dirs); plan-commissioned campaign dirs in `build/` (gitignored) | `docs/capability-maps/evidence/{rust,python}/` (440 KB, tracked) | `build/assessment/<UTC>-<label>-<pid>-<hex>/` (gitignored); ledger in `~/.local/state/pse-arrow/test-resources/` |
| Purpose | Answer a bounded question for one principal review or plan packet | Back `[probe]`/`[rustdoc:]`/`[api:]` markers in 4 capability maps (library behaviour at pins) | Record executed gates for the current source/env; enable reuse/transfer |
| Producer | Agents (coordinator/mappers/reviewers), ad-hoc scripts, hand-written markdown | `scripts/evidence-regen.sh` (`just evidence-regen`) + manual steps | `scripts/validation.py` (`just assessment`, turn-end/ready/hygiene bundles), `case_measure.py`, `build_measurements.py` |
| Lifetime rule | ADR-0096 "keep while needed"; lifecycle bundle default `doc_retention = "protected-unknown"` (`docs/lifecycle.toml` `[[bundles]] root = "docs/design_review/evidence"`) | Front matter `reviewed`/`regenerated`/`blueprint_revision`; no expiry | Sealed manifest with artifact roles; reclaim only of declared `scratch` (`test_resources.reclaim_reports`) |
| Consumers | Principal review, plan packets, later reviews (as "leads"), ADRs, capability maps, site publication (`docs/site.toml asset_bundles`) | Capability maps; ADR register R-20 | `reuse_checks` (later runs), `case_measure --functional-from`, evidence docs citing receipts by path |
| Condition capture | Free prose ("Baseline", "Conditions", "Tested conditions"), sometimes JSON fields | Probe output header lines (versions, toolchain, run time), committed lockfiles | Machine: `inputs` (scope+version+file digests+env), `source-files.json`, `host.json`, `cargo-metadata.json`, lock copies |

---

## 1. Evidence catalog by question

### 1.A Review/plan evidence (`docs/design_review/evidence/`)

"Inbound" = Markdown/TOML references outside the package itself (`grep` over docs, scripts, .codex, .claude, justfile). "Consumer state" checks the cited review/plan file exists and its `status:`.

| Question answered | Location | Producer / date | Conditions the answer depends on | Recorded? | Inbound consumers (state) |
|---|---|---|---|---|---|
| Do Arrow/DataFusion mechanisms (validator, eager eval, cast exactness, IPC/protobuf determinism) hold the guarantees blueprint rev 4 assumed? | `blueprint-rev4-2026-09-13/` (probe.rs, run.sh, output.txt) | Agent, 2026-09-13/14 | Arrow 59.3.0, DataFusion 55.1.0, Rust 1.98.1, dev profile, force_validate, repo lock | Yes: README "Tested conditions"; output.txt has input hashes/versions | ADR-0044, capability maps arrow-rust/datafusion-rust, review library-research-writes; **principal review retired** (cited only by GitHub permalink) |
| Do DataFusion bag set ops / joins / recursion / UDFs behave correctly at pin? | `full-arrow-datafusion-2026-09-14/` (probe.rs, run.sh, probe-output.txt, resolved-features.json) | Agent, 2026-09-14 | Same pins + object_store 0.13.2, partitions 1/4, upstream commits, resolved features | Yes (README "Conditions", checksums, resolved-features.json) | Capability maps arrow-rust/datafusion-rust; **principal review retired** (permalink) |
| What pinned solver libraries provide for acceleration/globalization; what unpinned candidates offer; literature mechanisms; code map of the solve spine | `solver-acceleration-2026-10-03/` (12 md, 548 KB) | Coordinator + 2 independent assessors + mappers, 2026-10-03 | Baseline `f0b90258…` + diff SHA; standard core 3.3/profile 1.3; pins POUNCE crates `=0.12.0` (features qp, convex), faer 0.24.4 | Yes in README/files | Review solver-acceleration (exists), integrated-solve review, `integrated-solve-pipeline/library-contract-refresh.md` |
| Refreshed library contracts for integrated solve pipeline (POUNCE path following, SUNDIALS reuse, PETSc) | `integrated-solve-pipeline-2026-10-03/library-contract-refresh.md` | Agent, 2026-10-03 | HEAD `f0b90258…` + Plan 25k tree; POUNCE 0.12.0, FERAL 0.18.0, faer 0.24.4, sundials-sys 0.6.2 | Yes (prose) | Review integrated-solve-pipeline only; **not in lifecycle.toml** |
| Build turnaround, runtime preparation, source lifetimes, SurrealDB capabilities and alternatives | `execution-efficiency-2026-10-05/` (5 md) | Mappers, 2026-10-05 | Start `6498b013…` + concurrent edits; SurrealDB 3.3.0 probe receipt of 2026-10-05 | Yes (prose) | Its review (exists); sibling `surrealdb-unified-substrate` README. One file currently dirty: link to retired Plan 25k being rewritten to a permalink |
| Can a unified SurrealDB substrate hold problems/revisions/runs/results; what machinery is removed | `surrealdb-unified-substrate-2026-10-05/` (3 md) | Agent, 2026-10-05 | SurrealDB 3.3.0 (upstream SDK), Context7 docs of date | Yes (prose) | Its review; Plan 28a |
| Which SurrealDB capabilities Plan 28 can rely on (bounded, per probe ID) | `plan-28-surrealdb-capabilities-2026-10-06/README.md` | Agent, 2026-10-06 | SurrealDB 3.3.0 release commit `238bfeb1…`, docs commit `82b7ac19…`, Rust 1.99.0 probe toolchain | Yes | Plans 28, 28a–28e (in-progress) |
| Do canonical-selection queries use the intended composite indexes? | `canonical-selection-2026-10-06/` (README, query-plans.json 12 KB) | Agent, 2026-10-06 | Installed `pse.substrate.v1` schema, SurrealDB 3.3.0, **empty tables** | Partly (schema version, server in JSON `server` key) | **No inbound Markdown**; only lifecycle.toml + site.toml publication; README names Plan 28e as owner but 28e does not link it |
| What inputs do native build scripts/helpers/loaders actually consume (producer review bases)? | `native-producer-inputs-2026-10-06/` (27 JSON + 2 md, 16 MB; `runtime-selected-native-inputs.json` 8.1 MB, `guarded-runtime-native-inputs.json` 6.2 MB) | Agent audit, 2026-10-06 | Exact crate versions (cc 1.6.0, pkg-config 0.3.34, pyo3 0.29.2, bindgen 0.73.2…), host tools, dirty checkout; JSON `scope` field e.g. "exploratory host only; not current selected compiler-unit qualification" | Yes per JSON (`scope`, sha256 of sources, `script_sha256`) — **generating scripts not retained** (only their hash, e.g. `python-helper-probes.json.source_script`) | **No inbound Markdown**; README says it supports Plan 28B/B3 but 28b does not link it; only site.toml asset bundle (22 of 27 JSON listed) |
| Production execution efficiency: where whole-test time goes; math/native library fit; SurrealDB acquisition and analytical boundary | `production-execution-efficiency-2026-10-07/` (execution-baseline.md, math/, surreal/) | Mappers, 2026-10-07 | HEAD `21933874…` + dirty Plan 28; preserved run `build/assessment/plan28-qualified-resumed-20261007/`; Symbolica 3.0.1, faer 0.24.4, Salsa 0.28.4, POUNCE git rev `ff638694…`; SurrealDB v3.3.0 source byte-compared | Yes (prose) | Two reviews; Plans 28e/28f/28g |
| Why did Python preparation stall (py-spy/perf of a worker)? | `preparation-assurance-and-reuse-2026-10-08.md` (front matter title/date) | Agent, 2026-10-08 | Worker PID, CPython 3.14.7, py-spy 0.4.2, perf 7.0.14, glibc, artifact SHA-256s, `build/plan28-e3-parallel-pool-dev-20261008` | Yes | Its review; Plans 28, 28i |
| WebSocket/persistent agent environment source/host observations | `websocket-and-persistent-agent-environment-2026-10-08.md` (front matter `status: review`) | Agent, 2026-10-08 | HEAD `2410880e…` + dirty tree | Yes | Its review; Plans 30, 30a (**done**) |
| Agent effectiveness: host config exposure, interface faults, activity retrospective, capability comparison | `agent-effectiveness-enhancements-2026-10-09/` (3.0 MB incl. 2.4 MB `event-pointers.jsonl.gz`, 570 KB `source-manifest.json`) | Agent + `retrospective/extract.py`, `probe-interfaces.py`, 2026-10-09 | Baseline `d0f2c418…`; fixed event-time window; personal Codex logs (not copied) | Yes (`source-manifest.json` start/cutoff/hashes; `interface-observations.json` `baseline`, `utc_observed`) | Its review; Plan 31 (**done**) |
| Graph/hash follow-ups: persisted Recipe feasibility, checker publication cost, BasisKey prehash cost, factorable inquiry, PC-SAFT smoke | `graph-hash-followups-2026-10-09/` (22 files: 5 `.rs` probes, 3 patches, 9 JSON, junit xml, log, 4 md) | Agent, 2026-10-09 | Salsa 0.28.4/0.28.5, temporary `persistence-probe` feature, local test build, 7-sample diagnostics; JSON fields `artifact_sha256`, `profile`, `features`, `source_files_sha256`, `baseline_head`; absolute paths into `build/graph-hash-followups-20261009-*` | Yes | ADR-0167, Plans 28f, 28k; efficiency `preparation-and-mathematics.md` |
| Codebase efficiency (build/validation, preparation/mathematics, execution/results); does `input_identity` detect every consumed input? | `efficiency-principles-codebase-2026-10-09/` (untracked; 3 md + `input-scope-probe.{py,json}`) | 3 reviewers + coordinator, 2026-10-09 | Baseline `4c24721e…`; Python 3.14.7; **`input_scope_version: 2`** | Yes | Its review; Plan 33 (in-progress) |
| Resume state for Plan 32 | `workspace-content-lifecycle-2026-10-09/handoff.md` (untracked; only evidence file with full `doc_*` front matter, `doc_retention: while-dependent`) | Agent, 2026-10-09 | Working tree; lists local `build/assessment/2026101…` receipts "if it still exists" | Yes | Plan 32 (in-progress) |

Front matter: only the two loose files and `handoff.md` have YAML front matter; all package READMEs are classified by `docs/lifecycle.toml` exceptions as `doc_role = "index"` owning themselves.

### 1.A' Plan-commissioned campaigns in `build/` (gitignored)

`build/` is 19 GB, 301 top-level entries (195 dirs). Name clusters: `plan09-acceptance*` ×22 (36–178 MB each), `plan09-source-probe*` ×5, `graph-hash-followups-20261009-*` ×10, `plan30-*` ~25, `plan28-*`/`plan28-e3-*` ~25, `25k-*`, `seed-conformance-*` ×13, `plan10/` (2.2 GB, JSON inventories such as `development-checks.json`, `failing-tests.json`, `incremental-cleanup-receipt.json`), `plan11/` (6.9 GB, 608 entries: clippy logs, `development-checks-before-i18-r1N.json`, `acceptance_routes.py`), `workspace-review-2026-10-09/`. Plans 9/10/11 are retired (last plan commit 2026-09-26). 97 distinct `build/<top>` paths are cited from `docs/design_review/evidence` + `docs/plans`.

### 1.B Capability-map evidence (separate category)

Answers "what does library X do at pin Y" for four maps. `scripts/evidence-regen.sh` copies committed `apisurface-/support-Cargo.{toml,lock}` into `build/evidence/`, runs rustdoc-JSON extraction and 11 probe binaries, Python `apidump.py`/probes in `.venv`/`.venv-parity`, and `uv export`s requirement files. Front matter: `arrow-rust.md`/`datafusion-rust.md` `blueprint_revision: 36, pins: Cargo.lock, regenerated: null, reviewed: 2026-09-14`; `supporting-rust-libraries.md`/`python-libraries.md` `blueprint_revision: 5, reviewed: 2026-09-13`; `datafusion_provider_contracts.md` `pins: {datafusion: 55.1.0, arrow: 59.3.0}, regenerated: 2026-09-15`. Overlap with A: the maps cite `blueprint-rev4` and `full-arrow-datafusion` packages as their probe evidence — the same "Arrow/DataFusion at 59.3.0/55.1.0" question answered by two harnesses (§4).

### 1.C Qualification receipts and ledger (separate category)

Answers "did gate G pass for inputs I". `checks.json` v5 keys: `version, mode, baseline_failures, scope, checks, complete, source_unchanged, provenance_errors, evidence, source_files, input_coverage, parent, environment, required_checks_covered`; per check `evidence_kind ∈ {executed, unchanged-input-reuse, reviewed-transfer}`, `inputs`, `artifacts` (digests), `origin`, `applicability_transfers`. `build/assessment/` = 471 entries, 4.9 GB (410 timestamped, 61 hand-named e.g. `plan28-efficiency-qualified-final-20261007`, `p16-p18-*`, `25m-*`), plus `latest-<label>` symlinks.

---

## 2. Repeated questions (review/plan evidence; cross-category only where truly re-answered)

| Question cluster | Answers | Do later answers reference / supersede earlier? |
|---|---|---|
| **What SurrealDB 3.3 can do for the substrate (transactions, graph edges, indexes, query plans, SDK transport)** | execution-efficiency `surrealdb-capabilities.md` (10-05); surrealdb-unified-substrate `integrated-capabilities.md` (10-05); plan-28-surrealdb-capabilities README (10-06); canonical-selection (10-06, EXPLAIN); production-execution-efficiency `surreal/README.md` (10-07); websocket evidence (10-08, SDK transport); agent-effectiveness `capabilities/*` (10-09, mentions 15–16×) | Partial chain: unified-substrate README links the earlier note as "Earlier bounded capability evidence"; production `surreal/` says "Prior dated review/evidence documents were leads only; their probe results are not newly executed evidence". No supersession marker; each re-derives from the same upstream source/probes. Condition since changed: the workspace now uses a **locally patched SDK** (`Cargo.toml`: `surrealdb = { version = "=3.3.0", path = "vendor/surrealdb" }`, `vendor/patches.md`, first committed `d0f2c4181` 2026-10-09). Version number unchanged, so a version-keyed check would not see it. |
| **Where does build/test turnaround time go** | execution-efficiency `build-turnaround.md` (10-05); production `execution-baseline.md` (10-07); efficiency `build-and-validation.md` (10-09); Plan 30d timing; `build_measurements.py` outputs | Cross-links: none between the three markdown files (each links only its principal review). Each states "historical measurements keep their original conditions". |
| **Runtime/mathematical preparation reuse and cost** | solver-acceleration `pipeline-map.md`/`reuse-structure-map.md` (10-03); execution-efficiency `runtime-preparation-and-studies.md` (10-05); preparation-assurance (10-08); graph-hash-followups (10-09); efficiency `preparation-and-mathematics.md` (10-09) | efficiency → graph-hash-followups linked; efficiency README: "Older diagnoses against their replaced implementations must not be transferred to this baseline" (blanket, not per-item). |
| **What the pinned solver libraries provide** | solver-acceleration `pinned-library-capabilities.md`; integrated-solve `library-contract-refresh.md`; production `math/README.md` | Refresh links the earlier study ("provided discovery routes"); production `math/` re-derives versions independently. |
| **Native build inputs / producer identity** | native-producer-inputs (10-06); execution-efficiency `build-turnaround.md` (provenance/native prep); efficiency `build-and-validation.md` (native installation/admission); Plan 28h | No links from later docs to native-producer-inputs (zero inbound Markdown). |
| **Arrow/DataFusion behaviour at 59.3.0/55.1.0** (cross-category) | blueprint-rev4 + full-arrow-datafusion (A) and capability-map probes (B) | A README states it did not overwrite or reclassify capability-map receipts; maps cite A. Two independent harnesses. |
| **Is a gate's prior pass still applicable?** (cross-category) | Category C `reuse_checks`; efficiency `input-scope-probe` (A) tested that very function | A answered a question about C's validity rule; then C's rule changed (see §3). |

---

## 3. Lifecycle state: stale, orphaned, consumer-retired

**Concrete condition changes after capture (nothing in the evidence signals them):**

| Evidence | Recorded condition | Current condition |
|---|---|---|
| efficiency `input-scope-probe.json` | `"input_scope_version": 2`; five paths and six env selectors undetected | Working tree `INPUT_SCOPE_VERSION = 3`; `RUST_INPUTS` now includes `scripts`, `.config`, `benches`, `pyproject.toml`, `uv.lock`; `PRODUCT_ENVIRONMENT` adds `NATIVE_BUILD_ENVIRONMENT` (`git diff scripts/validation_scope.py`). The finding the probe documented is being remedied; the JSON stays as-is. |
| solver-acceleration `pinned-library-capabilities.md`, integrated-solve refresh (10-03) | POUNCE crates.io `=0.12.0` | Since `32a14c8ab` (2026-10-04) POUNCE family is a git fork at rev `ff638694…` (pin reason "Plan 25n"). |
| SurrealDB cluster (10-05…10-08) | Upstream SDK 3.3.0 | Patched vendored SDK (above). |
| blueprint-rev4, full-arrow-datafusion | Rust 1.98.1; `run.sh` reads `rust-toolchain.toml` at rerun time | Toolchain is `nightly-2026-09-29` (since `181d870ba`, 2026-09-28): a rerun silently runs under different conditions. |
| Capability maps (B) | `blueprint_revision: 36` / `5`; probe_output header `rustc 1.100.0-nightly (2026-09-12)`, run 2026-09-13 | Blueprint at `revision: 144`; `evidence-regen.sh` uses `tooling/dfarrow-apiex/rust-toolchain.toml` = `nightly-2026-08-18` (differs from the recorded run); script writes `requirements-platform.txt`/`requirements-parity.txt` but the committed files are `requirements-py313.txt`/`requirements-py314.txt`; ADR register R-20 still waits for the first `evidence-regen` that writes `docs/capability-maps/facts/`. |
| production `math/README.md` | Symbolica 3.0.1 | Matches current (3.0.1 since `b8d9e2773`, 2026-10-06); earlier 10-03 solver docs predate it. |

**Consumer retired / done:**
- blueprint-rev4 and full-arrow-datafusion: principal reviews deleted (cited by GitHub permalink at `8950dd3d…`); retained because ADR-0044 and capability maps still cite them.
- execution-efficiency `surrealdb-capabilities.md` cites two reviews in another repository (library-context permalinks).
- websocket evidence → Plans 30/30a `status: done`; agent-effectiveness → Plan 31 `status: done`; both plans still present.
- Plan 25k retired; `runtime-preparation-and-studies.md` is mid-repair to a permalink (dirty diff) — the documented ADR-0096 link-repair route in action.

**Kept only by publication selection:** `canonical-selection-2026-10-06` and `native-producer-inputs-2026-10-06` (16 MB, tracked since `06302af77`) have no inbound Markdown; they remain referenced only by `docs/lifecycle.toml` (README exceptions) and `docs/site.toml asset_bundles`. Their READMEs name owners (Plan 28e, Plan 28B) that do not link back.

**`build/` legacy:** `build_storage.stores()` names `plan10`, `plan11`, `assessment`, `workspace-review-2026-10-09` as `"legacy-producer-protected-unknown"`. Plan 32 handoff records sizes and "30 observed records had zero sealed manifests". Ledger cross-check (read-only, `~/.local/state/pse-arrow/test-resources/reports/*.json`): 31 report index entries, of which 25 are this checkout's `build/assessment` (all between `20261009T073220` and `20261009T234533`); the other ~446 assessment entries, all hand-named assessment dirs (e.g. `plan28-qualified-resumed-20261007`, cited by production `execution-baseline.md`), the cancelled `20261009T222854…-features-powerset` cited by the efficiency README, and **every** top-level `build/` campaign dir (plan09/10/11/28/30, graph-hash-followups, seed-conformance…) are unregistered. The ledger's `allocations.json` has 174 owners (108 `evidence`, 63 `database`, 3 `controls`, all `cleanup: retained`), most `evidence` owners being `/tmp/tmp*/build` paths (test fixtures registering into the real host ledger). Registered reports also include sibling worktrees `../pse-arrow-wt/eff33-*`.

**Signals that exist today:** (1) prose labels ("historical", "Tested, historical skill scope", "leads only"); (2) `docs/adr/register.md` deferred-trigger rows with `check`, `last-checked`, `next-check` linted by `scripts/check_register.py --lint/--due` — the only date-driven revisit mechanism, applied to decisions, not evidence; (3) `[workspace.metadata.pse.pins]` reasons with "checked YYYY-MM-DD"; (4) `docs/library-utilization.jsonl` per-library `verified: "<date>@<commit>"` and `pin`; (5) validation receipts' input identity (machine, category C only); (6) lifecycle `doc_retirement_trigger = "owner-release"` default (no evaluation of whether the owner released). Nothing compares an evidence item's recorded pins/toolchain/revision to current ones.

---

## 4. Repeated collection machinery

| Function | Category A instances (separate implementations) | Category B | Category C (shared) |
|---|---|---|---|
| Environment/version capture | blueprint-rev4 & full-arrow-datafusion `run.sh` (toolchain from rust-toolchain.toml, versions vs lock); native-producer-inputs `host-probes.json` (`versions`, `tools`, `environment`); `preparation-assurance` (manual py-spy/perf/glibc versions); `input-scope-probe.json` (`python`); agent-effectiveness `probe-interfaces.py` (`command()` wrapper) | probe_output.txt header lines; `rustdoc_gen.log` | `validation.provenance()` → `host.json`, `cargo-metadata.json`, copies of Cargo/uv locks; `relevant_environment()` |
| Source identity | free-text baseline SHA + "dirty" note in every README; `solver-acceleration` diff SHA-256; native-producer `source-contracts.json` (sha256 per file); graph-hash JSON `source_files_sha256`, `baseline_head`, `artifact_sha256`; agent-effectiveness `source-manifest.json`, event hashes | committed lockfiles | `validation.sources()` (`source-files.json`), `source.diff`, `source-status.txt`, `untracked-source.tar.gz`, `source-revision.txt` |
| Execution harness / isolation | Two near-identical scratch-Cargo harnesses (`blueprint-rev4/run.sh` vs `full-arrow-datafusion/run.sh`: 8 differing lines): copy workspace dep specs + lock into a temp crate, refuse new versions, run offline/locked. graph-hash: `include!` into an existing test module + `temporary-integration.patch` + temporary feature. `probe-interfaces.py`: `subprocess.run(timeout=30)`; `input-scope-probe.py` imports production `scripts.validation_scope` | `extract()` in evidence-regen.sh copies manifest/lock to `build/evidence/<profile>` | `validation.execute()` with resource class via `scripts/pse-env`; host admission |
| Control vs run | blueprint-rev4 "positive controls and counterexamples"; full-arrow-datafusion expected/actual table; graph-hash persisted vs non-persisted query control; agent-effectiveness "isolated mocked faults"; efficiency probe positive controls | probes print observations, no control schema | none (pass/fail per gate) |
| Output schema / JSON | Each JSON its own ad-hoc top-level shape; only `input-scope-probe.json` (`"schema": "review-input-projection-probe-v1"`) declares a schema; native-producer files share a `scope` key convention only | text probe outputs | `checks.json` v5, `scope.json`, `test-findings.json`, `failures.json` |
| Summarization to Markdown | Hand-written README/analysis per package | Hand-updated `[probe]` quotes ("update the maps' [probe] quotes if measurements changed") | `checkpoint()` writes `summary.md` |
| Rerun entry point | `bash …/run.sh` (2), `just unit-package … --features pse-compiler/persistence-probe` after applying patch (graph-hash), `scripts/pse-env … input-scope-probe.py`, `extract.py --help`; native-producer-inputs and canonical-selection: **no retained producer script** | `just evidence-regen` | `just assessment`, `turn-end`/`ready`/`hygiene`, `case-measure`, `bench-builds` |
| Reuse by input identity | none | none | `validation_receipts.reuse_checks` |
| Placement into build/ | ad hoc names (`build/graph-hash-followups-20261009-*`, `build/plan28-e3-*`), unregistered | `build/evidence` (`PSE_EVIDENCE_WORK`) | `validation.fresh_output()` (unique, refuses overwrite, must be gitignored) + `test_resources.register_report` |

Other in-repo evidence producers (category C-adjacent, reuse validation helpers): `scripts/case_measure.py` (requires `--functional-from` completed qualification; uses `validation.fresh_output`, `validation.sources`), `scripts/build_measurements.py` (isolated snapshot; `validation.fresh_output`), `scripts/thermodynamic_campaign.py` (Plan 23 feeds; own output dir argument), `scripts/audit_tools.py` (`build/audits`), `scripts/native_tests.py` (`build/native-tests`), `scripts/setup_report.py`, `scripts/producer_deployment.py` (receipts), reference freezers `plan14_reference.py`, `idaes_nrtl_reference.py`, `pr_stability_reference.py`, `feos_entropy_reference.py`, `gross2001_bank.py`, and `scripts/library_utilization.py` (`docs/library-utilization.jsonl`, `build/library-usage.sqlite`). Only `validation.py` calls `test_resources.register_report`; measurement/campaign outputs are not ledger-registered.

---

## 5. Existing reuse/validity concepts

| Concept | Where | What it decides | Scope |
|---|---|---|---|
| Input identity | `validation_scope.input_identity(scope, snapshot, environment)` | Projects whole-checkout file digests to a per-scope path-prefix set (`INPUT_SCOPES`: rust-product, python-product, deployment-capture, tooling, documentation, generation) + allow-listed env (`INPUT_ENVIRONMENT`), tagged `INPUT_SCOPE_VERSION` (now 3). Unknown scope keeps all inputs. | Gates only |
| Unchanged-input reuse | `validation_receipts.reuse_checks` → `evidence_kind: "unchanged-input-reuse"` | Requires parent `checks.json` v5 with `input_coverage`, identical gate declaration and invocation, prior `qualified`, unchanged origin report digest and artifact digests, **exactly equal** `inputs`; native verified | Explicitly selected gates |
| Reviewed transfer | same, `"reviewed-transfer"` | Same integrity checks, inputs may differ; mandatory `transfer_reason`; appends `applicability_transfers {from_inputs, to_inputs, changed_inputs, reason}`; never reported as new test | Explicit, human/agent rationale |
| Origin protection | `test_resources.reference_report` / `retain_reference` | Reuse requires origin with sealed manifest `artifact_policy_version 1`; consumer holds a reference so origin is not reclaimed | Ledger-registered reports |
| Artifact roles | `validation.artifact_roles` + `test_resources.ARTIFACT_ROLES = {receipt, provenance, evidence, scratch, unknown}` | Producer-declared meaning; undeclared → `unknown` (protected); only `scratch` reclaimable (`remove_report`) | Producer outputs |
| Measurement prerequisite | `case_measure --functional-from` | Timing only from a completed qualification for the same snapshot | Criterion cases |
| Capability-map regen | `just evidence-regen` | Regenerates outputs from committed lockfiles; no comparison/staleness verdict | Maps |
| Lifecycle metadata | `docs/lifecycle.toml`, `scripts/document_lifecycle.py` (`inventory`, `validate`, `scope`, `retire-plan`) | Role/owner/retention defaults; retirement preflight; "neither authorize nor perform deletion" (`docs/dev/documentation.md`) | Documents incl. evidence bundles |
| Deferred-trigger register | `docs/adr/register.md`, `check_register.py` | Date-driven recheck with shell `check` commands | Decisions |
| Storage observer | `build_storage.observe` | Sizes/ownership labels, never disposal ("consult the producer owner") | build/, target, caches |

Category A reuse is entirely prose-based ("leads only", "historical", "must not be transferred").

---

## 6. Boundary observations

- Authored evidence in `docs/` cites producer artifacts in `build/` by path (97 distinct top-level `build/` paths from evidence + plans; graph-hash JSON embeds absolute `/home/paul/pse-arrow/build/graph-hash-followups-20261009-producer-rebuilt.json` with its digest; production `execution-baseline.md` cites `build/assessment/plan28-qualified-resumed-20261007/`). Most of those targets are unregistered in the ledger, so the protection that `reference_report` gives to receipts consumed by later receipts does not cover receipts consumed by documents.
- Plan 32 handoff (`handoff.md` §"Producer-owned artifacts") already states producer meaning lives in the ledger and documents carry only references ("Workspace resource reference" row in its proposed model; "Completing a document does not release its evidence resources").
- Some raw producer outputs were copied into `docs/` (native-producer-inputs 16 MB of JSON; agent-effectiveness 2.4 MB gz pointers) — producer output committed as authored evidence, without the producer script in two packages.
- Category A's `input-scope-probe.py` imports production code (`scripts.validation_scope`), so its retained output is tied to a code version that has since moved.
- Turn-end receipts are cited as publication checks inside evidence READMEs (agent-effectiveness cites `build/assessment/20261009T111639…-turn-end…/summary.md`, which is registered).
- `docs/book/` (gitignored) contains published copies of evidence assets selected by `site.toml` — a third copy location.

**Search coverage and limits.** Inbound-link searches covered `docs scripts .codex .claude AGENTS.md justfile mkdocs.yml` for `*.md *.toml *.py *.yml *.json`; constructed paths in code were not exhaustively traced. Ledger counts are a point-in-time read of `~/.local/state/pse-arrow/test-resources`. Large JSON payloads were inspected by top-level keys only. No probes, tests or regeneration were run.
