# Graph and hashing followup probes

These disposable Rust test modules investigate GH3/GH4 using the actual authored
compiler and admitted portable `Recipe`. They introduce no production runtime,
checker cache or persistent query database. The owning scope and disposition remain
in [Plan 28k](../../../plans/28k-graph-kernels-and-hashing-investigations.md).

The selected followup direction is to adopt local optimizations unless
they are incorrect or clearly substantially regressive. This includes the
process-local Fx interner, reuse of an already-computed structure key and a cached
prehash for `BasisKey`, while retaining complete key equality and the existing
durable identity contracts. Their implementation and evidence belong to the
[hashing inquiry](hashing-results.md) and owning plan. The diagnostics below do
not impose a minimum speedup gate on those changes. Small regressions or timing
noise from concurrent repository testing are not exclusions; future integration
and representative regression checks remain expected.

## Execution boundary

The probes are included temporarily inside
`crates/pse-compiler/src/typed_math_portable.rs`'s existing test module. The compiler
temporarily declares `persistence-probe = ["salsa/persistence"]`. Both integrations
are removed after execution. The Salsa family stays at runtime/macros 0.28.4 and
macro-rules 0.28.5 from the repository lockfile; no production feature is enabled.
Use the pinned checkout environment and explicit
`pse-relations/force-validate` when reproducing these correctness tests.

The probe files' `use super::*` refers to that existing test module, which supplies
its `authored` helper and actual private `Recipe` type. They are not independent
standalone crates and do not make that private implementation a public API.

The [temporary integration patch](temporary-integration.patch) preserves the two
source inclusions and optional feature for replay. Apply it only to a checkout
without that temporary integration, and remove both parts after the inquiry.
Adding the feature expands Salsa's lockfile dependency edges to the already-locked
`serde` and `erased-serde`, and enables `thin-vec/serde`; no package version changes
are required. Resolve that feature-aware lock graph before the locked test command,
inspect the diff, then regenerate the ordinary lock graph after feature removal.
Do not retain the probe feature or persistence-only dependency edges as a production
adoption. No runtime persistence was adopted by this experiment.

Temporary test-module integration:

```rust
#[cfg(feature = "persistence-probe")]
mod recipe_persistence_probe {
    include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../docs/design_review/evidence/graph-hash-followups-2026-10-09/recipe-persistence-probe.rs"));
}
#[cfg(feature = "persistence-probe")]
mod authored_checker_probe {
    include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../docs/design_review/evidence/graph-hash-followups-2026-10-09/authored-checker-probe.rs"));
}
```

Targeted command after adding the temporary feature:

```bash
just unit-package pse-compiler 'test(real_recipe_persistence_feasibility) | test(authored_checker_publication_cost_and_boundaries)' --features pse-compiler/persistence-probe --success-output immediate --status-level pass --profile local
```

## Persistent Recipe feasibility

[recipe-persistence-probe.rs](recipe-persistence-probe.rs) obtains a payload from
an actual admitted authored body with physical receipts and mathematical proof
receipts. A persisted input owns the payload and an interpretation leaf. A
persisted tracked query returns the decoded `Recipe`; a nonpersisted tracked query
provides the contrasting control. The checks cover fresh-database restoration,
query execution counts, changed payload and interpretation leaves, truncated
snapshot refusal in a disposable database, and lawful LRU recomputation of the
same real Recipe value.

The interpretation mutation establishes dependency invalidation even when the
decoded Recipe remains equal. It is a probe input, not a completed production
build/physical-context compatibility or qualification design.

The timing boundary compares ordinary whole Recipe decoding with whole database
snapshot restoration plus a query fetch, including result cloning. It reports raw
samples, payload and snapshot extents, and the Recipe's existing retained-byte
estimate. Separate samples measure existing `reconstruct_fixture` with actual
portable identity/specification, receiving physical registry and proof replay,
evaluator compilation from reconstructed and original admitted bodies, and worker
creation/evaluation. The outputs and Jacobians must agree. These are fixture
receiving controls; they do not measure a real protected read, runtime
qualification, durable storage I/O or a complete preparation journey. Restoring
the persisted memo avoids executing the query's payload decode, but snapshot
restoration still deserializes the stored input and Recipe result. It leaves all
ordinary mathematical reconstruction and receiving qualification necessary.

These seven-sample diagnostics use the local correctness-test build, with fixed
within-sample phase order and shared process/library state. They establish the
measured fixture costs under those conditions, not a general production speedup
or a confidence interval. Decode, complete database restore/fetch and qualified
mathematical reconstruction are separate operation boundaries. In particular,
the reconstruction time cannot be claimed as work eliminated by restoring a
persisted Recipe: the restored Recipe still requires that reconstruction.
Future integration should compare matched complete preparation operations and
account for the work actually reused.

The corruption control only establishes refusal of truncated serialization before
publishing a restored database. It does not establish an authenticated persistence
format, semantic rejection of arbitrary valid-JSON tampering, or general build and
ingredient compatibility.

## Authored checker publication cost

[authored-checker-probe.rs](authored-checker-probe.rs) uses an actual authored scalar
package with 32 unrelated functions. It reports separately:

- The full no-document `pse_modeling::check` comparator.
- The real `CompilerWorkspace::publish_modeling` call.
- Subsequent `prepare_modeling_cancellable` work in the same workspace.

Parsing, initial workspace construction and initial preparation are excluded.
Unchanged, unrelated, selected, source-span, revert, membership insertion/deletion,
missing-reference, absence-to-present and denied physical-scope cases exercise
publication and failed-publication atomicity. The direct checker is a separate
comparator; it is not instrumentation inside `check_modeling`, and these scalar
fixtures do not exercise document admission.

Source inspection establishes the boundary being investigated:
`workspace/modeling.rs::publish_modeling_with` skips checking only when complete
rows, physical scope and documents equal the current revision. Other publications
call `check_modeling`, which invokes `pse_modeling::check_with` before publishing a
new Salsa catalog. Downstream query reuse therefore does not establish that the
upstream full checker was skipped.

## Complete checker integration avenue

A complete typed incremental checker proposal must replace the full checker with
one authoritative mechanism and preserve its observable refusals, source witnesses
and failed-publication atomicity. Its tracked input domains must cover:

- Declaration identity, kind, parent/child membership and insertion/deletion;
  lexical names, visibility, duplicate names and positive and absent lookups.
- Physical quantities, units, preconditions, physical document visibility and
  interpretation; relevant scientific provider, policy and structural-demand
  inputs at their owning boundaries.
- Authored expressions and full declaration contracts, including function calls,
  inheritance/cycle restrictions, validity, contextual/temporal knowledge,
  provenance and source attribution.
- Data-document membership, interpreted plans, rows, absent documents and
  admission allocation limits; failure and cancellation must preserve the old
  published selection.

The full current canonical dependency recheck under a fresh protected read remains
authoritative. A positive-only dependency graph, body-hash equality or a manually
maintained second graph cannot replace the negative lookup and membership premises.
An incremental result must be checked against the full existing checker using
independent mutation cases, not merely its own graph's answers.

## Persistent pure reuse integration avenue

A selected production product must first establish serializable keys, complete
transitive tracked inputs, results and failure behavior, interpretation/build/
ingredient compatibility, receiving qualification, roots and generations, garbage
collection/eviction, exact allocation ownership and stale-fill behavior. Restore
must occur into disposable state and publish only after qualification. Salsa
interned IDs are not durable semantic identities, and restored memoization does
not recover accumulators. Until that complete design is selected, the current
qualified portable-body path remains authoritative.

A later persistence candidate should identify a complete pure result whose
restoration actually replaces repeated work, then integrate its receiving and
lifetime contracts into the existing preparation path. That inquiry can include
serializable mathematical products if the library and qualification contracts
permit them; the present Recipe experiment establishes neither their capability
nor their correctness. Local hashing and graph-key reuse can be integrated
independently of selecting such a persisted product.

## Results

**Tested — 2026-10-09:** the preceding command selected exactly two compiler unit
tests with explicit force-validation: 2 passed, 0 failed against the zero failure
target; 232 tests were outside this selection. It ran in the root's existing
native-admitted shell, using the pinned toolchain and locked dependency family,
Nextest's `local` profile and the optimized Cargo test build. Temporary compiler
includes were removed after the run. The root owns temporary feature/lock cleanup.
The first attempt used `--nocapture`, which the recipe's selection/list phase
rejected; the recorded run uses supported success-output arguments.

The [raw log](recipe-and-checker-probes.log) includes compiler warnings from a
separate temporary factorable probe. Passing these tests is not a static-clean
claim. Extracted raw samples are in
[Recipe results](recipe-persistence-results.json) and
[checker results](authored-checker-results.json).

**Measured:** medians of seven within-process diagnostic samples, milliseconds
per operation, under the preceding conditions:

| Real Recipe operation | Median ms |
| --- | ---: |
| Ordinary payload decode, 100 operations per sample | 0.0569 |
| Complete snapshot restore plus persisted Recipe fetch, 10 per sample | 0.9655 |
| Qualified fixture mathematical reconstruction, one per sample | 2.5599 |
| Evaluator compile from reconstructed body, one per sample | 0.0608 |
| Evaluator compile from original admitted body, one per sample | 0.0104 |
| Reconstructed-body worker creation and evaluation, one per sample | 0.0067 |

The payload was 24,652 bytes; the persisted database snapshot was 109,261 bytes.
The decoded Recipe reported 58,883 retained bytes, while the original admitted
body reported 37,535 descriptor bytes and 871 mathematical retained bytes using
their existing estimators. Different decoded container capacities affect these
estimates; they are not peak-process allocation measurements.

**Retained-design decision:** actual Recipe persistence works with this locked
Salsa family: fresh restore executes the persisted query zero times, the
nonpersisted control once, and payload/interpretation changes each cause one
reexecution. Truncated snapshots are refused in disposable state and LRU eviction
recomputes lawfully. However, complete snapshot restore/fetch costs more than
ordinary payload decode for this product, still deserializes the stored Recipe,
and avoids no mathematical reconstruction. The measured 0.9655 ms complete
restore/fetch exceeds the 0.0569 ms decode it replaces, while the separate
2.5599 ms mathematical reconstruction remains necessary. Keep the current
qualified portable path and leave RC03 unselected for this product. This is a
specific result about Recipe restoration, not a marginal-benefit gate for local
hashing or a rejection of future persistent pure products. Reopen RC03 around a
complete reusable product and its matched preparation, compatibility and
receiving contracts.

| Authored edit | Full no-document checker ms | Actual publication ms | Subsequent preparation ms |
| --- | ---: | ---: | ---: |
| Unchanged | 5.0736 | 0.0063 | 1.0219 |
| Unrelated function | 5.1130 | 5.1721 | 1.4131 |
| Selected function | 5.0722 | 5.3294 | 1.6858 |
| Source span | 4.9549 | 5.0678 | 1.3413 |
| Revert | 5.0653 | 5.1550 | 1.4102 |
| Membership insert | 5.2844 | 5.4967 | 1.4281 |
| Membership delete | 5.1305 | 5.2279 | 1.4465 |
| Missing reference, refused | 3.4046 | 3.2098 | 1.0164 |
| Missing reference becomes present | 5.3688 | 5.4518 | 1.5606 |
| Physical scope denied, refused | 0.2608 | 0.2252 | 0.9573 |

**Proposed:** this authored editing fixture supplies a concrete reason to reopen
N0's checker-domain design investigation. Changed publication takes about
5.1–5.5 ms, including unrelated and span changes, while subsequent preparation
takes about 1.3–1.7 ms. Exact unchanged publication uses the fast path. These
results establish material upstream work for this fixture, not product-wide
latency or savings from an unimplemented incremental checker. Both refused
publications preserved the previous portable semantic identities and usable
selection. The [typed checker proposal](typed-checker-proposal.md) defines the
required replacement domains and decision questions. Keep the full existing
checker and fresh complete canonical recheck until that design is selected and
independently qualified.

## Production smoke and restart diagnostic

The [PC-SAFT production smoke](pcsaft-02-production-smoke.json) records four typed
case-resolution refusals under the unchanged scientific workload. The harness completed
without obtaining a prepared product. [28f](../../../plans/28f-shared-numerical-preparation.md#pc-saft-numerical-fact-investigation-boundary)
owns the numerical-fact investigation; this is no hash comparison or accepted PC-SAFT preparation.

The temporary [restart capture helper](restart-context-probe.rs) ran after each actual
admission and confirmed its independently captured observation reconstructed that admission
identity. It required a temporary existing workspace pse-buildinfo dev-dependency and the
seed/receiving callers recorded the returned values before their unchanged identity assertion.
The [captured observations](restart-context-diagnostic.json) were equal. These observations
do not reconstruct the field that differed in the earlier refused execution. Temporary
capture integration and its dependency are removed. The final production restart control
passed with actual reuse, original history and independent weak-owner/zero-pool teardown
checks. Its only permanent fixture correction waits for both release conditions under a
deadlock watchdog; it never retries admission to force a context match.
