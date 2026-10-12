---
title: Plan 36 development investigation evidence
doc_owner: docs/plans/36e-build-test-and-development.md
doc_topics: [architecture, preparation, storage, validation, environment]
---

# Development investigation evidence

Inspected 2026-10-11 at `2d6bf6ae7`, initially clean main. [Plan 36e](../36e-build-test-and-development.md#investigation-decisions-2026-10-11) owns decisions and package progress. This report records evidence, not a second execution/status ledger. No production source, older plan/index, authoritative design or accepted ADR was edited. No shared cache was cleared, toolchain overridden, production database removed, or other owner's workload stopped.

## What settled the design

| Question | Current source inspected | Consequence |
|---|---|---|
| Exact metadata policy | `scripts/test_resources.py`: resource_metadata, register, finalize, resource_status, borrow, references, report index/manifest, reclaim | Single-resource policies predominate; whole-invocation finalization can project one frozen outcome. Logical uniqueness and registration/freeze need exact shared authority. Keep one policy owner; per-record files remain viable. |
| Cold history and active work | resource_metadata loads `completed/<id>.json` only on exact request; demotion/index publication under host.metadata | No historical-leak inference. New lookup may preserve exact cold reads while eliminating whole-active-ledger work. |
| Lock/deadline | `scripts/host_admission.py`: lock_until, metadata; test_resources calls metadata without a deadline | Original clocks must reach lock acquisition itself. Retaining a sweep deadline alone does not bound an earlier blocking flock. Stable lock inode lifetime is part of the design. |
| Late registration | finalize reads invocation and scans current resources; register does not consume a terminal/frozen phase | Negative control confirms an incomplete pinned resource can appear after finalization. No false passing disposition observed; missing freeze/projection coverage is real. |
| Command result | `scripts/pse_env.py`: scoped native and ordinary finally paths reclaim_reports after release; outer exception maps to125 | Unrelated registry errors can change boundary result. Inherited/no-scope exec paths do not traverse that sweep. Keep own-operation drain separate. |
| Runner result | `scripts/test_run.py`: run_python/run_rust compose after subprocess.call; finish projects via selected owner; nested observer route already has owner declaration | Add terminal finally/recovery at the selected owner. Do not duplicate enclosing signal forwarding or infer Pass from absent reports. |
| Deferred database cleanup | `test_resources.eligible/reclaim/remove_database`; reclaim_reports selects evidence | Explicit selected cleanup must cover eligible same-generation databases after service reopening; unrelated exit sweep does not supply this. |
| Fixture pool | `crates/pse-operations/src/testing.rs`: canonical_fixture_store, remove_isolated_fixture, fixture explicit drain test; Runtime canonical tests and late operations | Handles/delayed operations have actual lifetime. No all-state/late-effect reset primitive establishes reusable isolation. Retain fresh UUID fixtures; no timing needed to reject an unproved reset. |
| Direct test owner | `crates/pse-runtime/src/workflow/mod.rs`: from_shared, check_deployment; workflow/tests.rs affiliation and durability tests | check_deployment already compares real consumed pool/disk/cache/object-store/CPU/width owners. Direct tests can avoid canonical setup; retain composition/binding integration. |
| Lower admission | Runtime authoring_driver production imports and physical.rs/plans/preconditions; xtask physical::Sources; workflow/modeling/pure.rs | Production physical admission can be constructed from EngineResources/EngineFactory without MathService. Generated document adapters currently belong to Runtime and must migrate from generator declarations. Test-only workflow imports cannot become lower dependencies. |
| Full generators | `xtask/Cargo.toml`, codegen physical/schemas and main dispatch; pse-codegen rust generated roots | --only narrows work but does not remove normal Rust dependencies. Separate capability features/targets; actual workflow/native/operations schema types remain high owners. |
| Surrogate root | pse-math Cargo and surrogate.rs; all pse_math::surrogate references across Runtime | Five normal dependencies plus egobox-doe dev dependency move with the single integration consumer. Target-shaped graph removes 75 packages from math without selecting a new crate. |
| Hack fallback | `.cargo/config.toml`, `.config/hakari.toml`, hack manifest; exact Cargo graphs | Supported pinned nightly keeps selected-root feature agreement without hack in sampled roots; removing hack also eliminates unrelated packages. Retire through accepted RC17's concrete ADR route, then verify all affected roots. |
| Features | root Cargo, pse-py Cargo, actual source call sets, DataFusion55.1.0 manifest | SQL/functions/Parquet and external compressed readers have a supported role; fixture census is not an input contract. No update_rayon or production operations/testing consumer found. Own float_roundtrip before pruning. |
| Nextest preparation | `scripts/native_tests.py`: _rust_capture, managed_rust_run; test_run inventory argument; arrow_validation selected unit graph | Retained binary/package metadata already avoids child rebuild/resolution. Extend one prepared invocation object through ordinary and validation orchestration; do not add cross-run cache. |
| Provenance/parity | validation_scope RUST_INPUTS v3; justfile parity; native_tests provenance/import binding | Conservative capture includes execution/config/generated/dirty/absence-sensitive inputs. uv/maturin are already incremental; a lock hash alone is insufficient installed-state ownership. |
| Launch/host | agent-capacity.toml; host_admission Profile/classify/cpu_set; build_environment jobs; native_tests effect groups | Keep disjoint timing lane and quotas. Explicit classes/default widths come from actual owner; CPU quota, physical lanes, logical affinity and store-effect capacity are different inputs. |

## Executed controls

All results below have a zero-failure target. The commands use the checkout environment. These are **Tested** controls or **Measured graph populations**, not scientific or whole-product qualification.

1. `scripts/pse-env -- .venv/bin/python -m unittest scripts.tests.test_test_resources -q`: **50 tests, 0 failures**, 2026-10-11. They patch temporary registries/service observations and exercise the existing policy, not the proposed v2 backend. Current source has no registration/freeze assertion.
2. `scripts/pse-env -- .venv/bin/python docs/plans/36-investigation-evidence/development-probe.py terminal`: **passed**, post-finalize registration accepted but pinned/incomplete; no false Pass. Uses the existing TestResourceTests setup and restores patches. No serving database involved.
3. `scripts/pse-env -- .venv/bin/python docs/plans/36-investigation-evidence/development-probe.py freeze-model`: **10 abstract crash/order cases, 0 failures**. Membership intent precedes resource publication; freeze/recovery accounts for each accepted or uncertain resource. It is a finite protocol example, not filesystem concurrency, fsync, hostile input or production recovery proof.
4. `scripts/pse-env -- .venv/bin/python -m unittest scripts.tests.test_surreal_server.SurrealSupervisorTests.test_recovery_equivalence_accepts_exact_relocation_and_fresh_incarnation scripts.tests.test_surreal_server.SurrealSupervisorTests.test_recovery_equivalence_refuses_changed_receiver_bytes_association_and_permissions scripts.tests.test_surreal_server.SurrealSupervisorTests.test_recovery_receiver_refuses_intermediate_and_manifest_symlinks scripts.tests.test_surreal_server.SurrealSupervisorTests.test_typed_readiness_separates_absence_unavailable_and_mismatch scripts.tests.test_surreal_server.SurrealSupervisorTests.test_unknown_or_mismatched_readiness_never_reserves_or_starts -q`: **5 tests, 0 failures**, temporary fixture files and controlled observations. Carried state-review F06/F08 leads now have focused current source/controls; broader actual host restart/scientific recovery remains outside this result.
5. `scripts/pse-env -- .venv/bin/python docs/plans/36-investigation-evidence/development-probe.py topology`: **passed**; [structured output](topology-results.json). Owned temporary manifests and source symlinks, actual pinned Cargo `check/test --offline --unit-graph`, no code compilation. External source/version/checksum resolution tuples unchanged. Target-shaped manifest edits move surrogate dependencies and remove hack dependencies; no production manifests changed.
6. `scripts/pse-env --recipe-default-class compile -- .venv/bin/python docs/plans/36-investigation-evidence/build-input-probe.py`: **passed**; [structured output](build-input-results.json). Actual unmodified backend build.rs/build_receipt.rs copied into an owned tiny workspace with exact pinned toml/cc/pkg-config, no native features, one miniature consumer. Same target/profile/features/source throughout; plain warm control then unused native environment and return. Each transition reran build script and both consumer rustc units; hashes stayed equal, mtimes changed. No actual full backend latency, native linking or shared-cache perturbation measured.

The first build-input harness run completed Cargo checks but failed its output-location assertion: pinned Cargo's output layout was not the assumed glob. The corrected harness obtains Cargo's target directory and discovers the two named output files; its build-script execution predicate also uses the actual emitted directive. That harness mistake is not a product/build failure. Final recorded output comes from the corrected harness; no stale receipt is promoted.

### Unit graph observations

| Selected closure | Units | Distinct packages | Relevant feature fact |
|---|---:|---:|---|
| Current math check/lib/no-default | 368 | 250 | Four egobox packages present; no Arrow/native backend |
| Surrogate manifest move, math same selection | 264 | 175 | All four egobox absent |
| Surrogate move + no hack, math | 264 | 175 | Math was already outside hack roots |
| Current engine check/lib/no-default | 423 | 285 | Arrow59.3.0 normal leaves, force_validate absent |
| No hack, engine same selection | 382 | 273 | Same normal Arrow leaf features |
| No hack, engine test/lib/force-validate | 378 | 273 | force_validate present on array/data/cast |

The test and check unit populations differ by mode; their counts are not directly a speed comparison. No lower-admission source move was compiled. The prototype intentionally leaves an empty, unreferenced hack workspace member to compare resolution without introducing unrelated lock updates; actual retirement removes its member/declarations/configuration too. All declared external dependency versions remain exact.

Removing the direct BLAKE3 rayon opt-in left the sampled IDs graph at 78 units: other workspace-unified consumers still keep the dependency. The selected declaration cleanup therefore makes no graph-size or performance claim. It removes an unused direct opt-in; it does not remove Rayon from actual mathematical/library consumers or change hashing bytes.

## Alternatives settled without speculative timing

Per-record files have short identity/invocation/resource lock domains and recoverable pending intent; current transitions do not require an indivisible all-resource commit. SQLite remains a credible transactional alternative, but choosing it solely to gain multi-row transactions would invent a new invariant. Actual-file crash/race controls may still falsify the selected ordering and force reconsideration before migration.

A reused used-database fixture is rejected because missing drain/reset guarantees can change test meaning. Fresh fixtures and direct-owner tests already provide a viable target. Whether optional one-use batched provisioning later saves setup does not affect that contract.

Removing DataFusion default functions based on present fixture spellings would narrow supported SQL/invariant behavior. Current exact manifest defaults include nested/crypto/datetime/encoding/regex/string/unicode, compression, parquet, recursive protection and SQL. Keep them. Current Context7 README mentions unparser as a default, but **the exact 55.1.0 manifest inspected does not**; current-branch documentation is not proof of this pin. Keep consumed Arrow/Parquet external-format features rather than infer their dispensability from canonical result IPC policy.

Timing-lane borrowing is unselected because weights cannot revoke an in-flight foreign workload or establish quiet timing/store conditions. No named blocked required operation supplied a stronger reason to change that isolation contract. This investigation closes with retention; it does not leave a mandatory open-ended host benchmark.

Cross-run nextest/environment/provenance caches are likewise unselected. One prepared invocation is a simpler consumed lifetime and already has real retained metadata machinery. An exact compiler/lock/native/import check remains required before parity reuse; current incremental sync/build is the conservative executable target.

## Source and library provenance

Context7 resolved `/rust-lang/cargo` and queried workspace feature unification/unit graphs/build-script change detection. Official source routes: [unstable Cargo](https://github.com/rust-lang/cargo/blob/master/doc/book/src/reference/unstable.md), [build performance](https://github.com/rust-lang/cargo/blob/master/doc/book/src/guide/build-performance.md), [build scripts](https://github.com/rust-lang/cargo/blob/master/doc/book/src/reference/build-scripts.md). Pinned `nightly-2026-09-29`, local config and executed graphs establish this checkout's actual behavior.

The DataFusion library skill was read for exact-release routing; Context7 resolved `/apache/datafusion` and queried feature contracts. [Current README](https://github.com/apache/datafusion/blob/main/README.md) is a discovery lead; local Cargo.lock and registry `datafusion-55.1.0/Cargo.toml` establish selected defaults. No capability maps, utilization JSONL or library catalog was read.

## Limits

Not run: production metadata cutover/crash injection; signal interruption of actual runners; lower-crate/source/generator compilation; full native target/host/dev powerset; real parity setup/IDAES science; complete native/provider deadline/cache-lifetime controls; actual backend warm rebuild latency; host throughput/RSS/timing campaign. These are implementation acceptance or optional quantitative claims, not missing target choices. Unselected alternatives have their reopening conditions in 36e. Broader rank/publication/hash/product choices stay with the coordinator's separate decision boundary.
