# Preserved execution and build evidence

Supporting evidence for the [principal review](../../reviews/design_review_production-execution-efficiency_2026-10-07.md).
This document supplies observations and source checks, not a separate verdict or disposition ledger.

## Baseline and limits

Inspection uses `main` at `219338742915fb16c02534c80430bd2ea97bf94a` plus the existing uncommitted
Plan 28 continuation. Those changes were the inspection snapshot. During publication,
commit `5260a3e9a3cecd69b6358ab86d4e917ae2508b29` recorded the continuation and this supporting
evidence. A before/after content comparison confirmed unchanged production, configuration
and test bytes. The review supplies no release qualification.
The maintainer requested stopping qualification to conduct this review. The owned validation
runner was interrupted and its reports preserved; the selected SurrealDB server was retained.
The review has not rebuilt, rerun the native series, changed production code or changed accuracy.

**Measured, historical preserved run:** `build/assessment/plan28-qualified-resumed-20261007/`
contains `checks.json`, `native-test.xml`, `native-test.log` and the native selection inventories.
The native command was `just native-test --profile local --config-file <run>/native-test-nextest.toml`,
in native force-validation mode. These existing observations are not newly executed benchmarks.
The report is incomplete, with unchanged source during its execution: 39 passed gates, one
interrupted gate and one not-run gate. Of the positive gates, 37 were explicitly transferred
from the previous qualified source context; the current producer and imported-Python association
checks are distinct. The full Python series did not run.

The native selection contained 2,821 tests across 86 binaries. Before the requested interruption,
2,425 passed, one was aborted by signal 15, and 395 did not run. The aborted progress-batching
test is not a discovered scientific failure. The native gate elapsed 1,793.689 seconds including
the logged 4 minute 11 second test build. That total includes launch/setup, compilation and
scheduling as well as test execution. The following are completed whole-test times; setup,
body and teardown are not separated.

| Completed test | Seconds | Relevant operation |
|---|---:|---|
| `reference_flash_completed_blocks_discharge_reporting_allowance` | 284.005 | Seventeen-package source admission, preparation, explicit block-preparation control and production case execution |
| `canonical_progress_backpressure_cancels_and_retains_integral_observation` | 35.514 | Durable progress/backpressure journey |
| `canonical_connected_results_reopens_declared_arrow_exact_range_and_cancellation` | 35.210 | Reopened exact Arrow range and cancellation journey |
| `canonical_output_indexes_read_original_rows_and_skip_unrelated_blocks` | 35.125 | Small indexed result projection with full runtime fixture |
| `large_activation_commits_3500_tiny_objects` | 31.605 | Large source staging and activation |
| `fixture_explicit_removal_disarms_last_owner_cleanup` | 24.559 | Multiple database lifecycle transitions |
| `indexed_process_composition_keeps_memoryless_units_algebraic` | 20.179 | Compiler process composition |
| `conditional_unit_recycle_reference_specializes_actual_unit_boundaries` | 20.045 | Conditional/recycle specialization |
| `engineering_accuracy_exact_original_optimum_uses_actual_scip_transport_and_value_arithmetic` | 19.732 | Actual SCIP and original objective-quality control |

For completed library tests, `pse-operations` has 78 observations, median 7.900 seconds and
summed duration 572.796 seconds; `pse-runtime` has 242, median 0.013 seconds and summed duration
927.641 seconds. Sums are neither process CPU time nor the suite critical path. Interrupted
and unexecuted tests are excluded. The censored, heterogeneous sample does not establish
the latency distribution of production cases or a full-run completion estimate.

## Lifecycle and scheduling attribution

**Interface-checked:** `pse_operations::testing::canonical_fixture_store` connects two gRPC
clients, installs the complete schema in a new isolated database, and attaches synchronous
last-borrower cleanup. `CanonicalStore::create` explicitly performs installation; ordinary
`open` only verifies the interpretation marker. The generated schema inspected here contains
39 tables, 292 fields and 100 indexes: 431 definitions, 88,034 bytes. This is repeated test
provisioning, not a production database rebuild on every solve. Removal first drains result
releases. Ownership and cleanup checks are meaningful and must remain.

`workflow::tests::runtime` always constructs that canonical fixture, even when a caller's
assertion concerns mathematics. The 284-second flash control separately calls `prepare_blocks`
to inspect the reporting allowance and then calls `solve_case`; the discarded first prepared
value is test-specific additional work. It is not evidence that ordinary solving duplicates
that exact preparation. The small 80-row result projection test includes source activation,
run creation/claim, result publication, reads and teardown. Its 35-second total is not
an isolated indexed-query measurement.

The current Nextest `store` group has four slots and matching tests require all four.
The broad earlier override covers operations, runtime workflows/mathematics and scientific
acceptances, so these tests serialize against the shared server. This scheduling explains
aggregate waiting structurally; increasing test concurrency is not a production-code remedy.
Production pool/resource choices and the selected 32 GiB server/worker allocation were not changed.

## Current build boundaries

**Interface-checked:** workspace development code is optimized at level 2, third-party code
at level 3. The selected SurrealDB 3.3.0 dependency is remote gRPC with default features disabled;
an embedded database engine is not being compiled into this SDK root. Native linking explicitly
uses the selected solver image's oneMKL LP64/GNU-threading provider and libraries. Build source
does not establish their runtime bottleneck or an advantage from changing optimization levels.

Earlier defects must not be replayed as current findings: semantic/compiler/math manifests no
longer inherit the broad workspace-hack, `bench-builds` no longer eagerly sources the complete
native environment, and generation preserves unchanged outputs. Current compiler artifact keys
use the selected environment rather than the outer whole-tree build identities.

Two concrete remaining mechanisms deserve design assessment:

1. `pse-buildinfo/build.rs` and `identity.rs` watch and read all files beneath `crates`, `vendor`
   and `xtask`, then embed the changing outer identity into `pse-buildinfo`. Its current direct
   consumers are Python and the optional xtask worker/deployment composition. Compiler/runtime
   manifests no longer directly depend on it. An unrelated test/backend source edit changes
   this shared compiled attestation even when a consumer's result-affecting unit closure is
   unchanged. `write_changed` avoids identical writes but cannot prevent a genuinely changed
   global digest. Preserve full dirty-source attestation while considering its placement at
   the actual artifact/deployment boundary. This is build amplification, not evidence of
   scientific cache invalidation in current compiler keys. Cargo documents recursive scanning
   of watched directories in its [build-script change detection](https://doc.rust-lang.org/cargo/reference/build-scripts.html#change-detection).
2. `native-execution-env.sh` sources solver and math preparation. `native_cache.prepare` holds
   an exclusive per-identity lock while `valid` rehashes every recorded installed header/library.
   Math setup also prepares KLU, root isolation and, absent explicit prefixes, Uno/PETSc.
   `native_pipeline_cache.prepare` discovers the HiGHS Cargo archive before the Uno cache-hit
   decision and hashes its archive/headers and BLAS files. Complete native qualification needs
   the complete family; narrow operations need only their consumed capability closure.
   A verified installation lifetime could remove repeated verification, but its immutability,
   corruption detection and changed-provider invalidation must be designed first. Existence
   checks alone do not preserve the guarantee.

No proportion of the observed build or test time is assigned to either mechanism. No cold/warm
build experiment, syscall profile or solver-stage profile was run for this review. Remaining
stage attribution should use a small existing operation with setup/preparation/solve/result
boundaries only when it can choose between materially different remedies; a full campaign or
a permanent instrumentation framework is unnecessary for that decision.
