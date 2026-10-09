# Hashing inquiry results

These bounded inquiries distinguish a process-local full-key interner, preparation
key traversal and canonical durable framing. Current scope and dispositions remain
with [Plan 28k](../../../plans/28k-graph-kernels-and-hashing-investigations.md).
They do not select RC02 durable hash replacement or introduce hash-only equality.

The maintainer's execution criterion selects targeted local improvements unless
they are incorrect or clearly substantially regressive. Marginal gains are
sufficient; there is no minimum-benefit gate. Timing differences below are bounded
diagnostics on a host also used for repository testing, with possible concurrent
activity and noise. They do not replace integration or representative regression
checks.

## Actual preparation-key traversal before cached prehash

[basis-key-cost-probe.rs](basis-key-cost-probe.rs) was temporarily included inside
`workflow::modeling::reuse_tests`. It prepares an actual authored scalar case,
obtains the checked selection and constructs its real `BasisKey` through the
runtime service. The input has a selected nonlinear equation and an unrelated
definition. [Raw samples](basis-key-results.json) preserve the observed costs.

**Tested — 2026-10-09:** one selected ignored inquiry passed, zero failures against
the zero target, using the pinned native environment, locked dependencies,
explicit force-validation and the optimized dev/test build. The root ran the
following command from a fresh actual native exclusive allocation; the managed
test runner used its exclusive observer profile and Nextest's `local` profile.
The terminal identity was `4658d2faab1f492fbbdb1bbba4c5aa9c`.

```bash
just unit-native-package pse-runtime pse-runtime/native-solvers,pse-runtime/canonical-tests graph_hash_basis_key_cost_inquiry --run-ignored only --success-output immediate --profile local --status-level pass --final-status-level pass
```

The final-status option is part of the executed command; it prints unselected
test statuses and is unnecessary for replay. This targeted pass does not qualify
other runtime or scientific behavior.

**Measured:** seven same-process diagnostic samples. Each sample hashes the
already-constructed key 20,000 times using a fresh `DefaultHasher` per operation,
constructs the real key 20,000 times under the retained selection, then invokes
complete warm preparation 20 times. The order is fixed within each sample.

| Operation | Median | Observed sample range |
| --- | ---: | ---: |
| Full `BasisKey::hash` traversal and `DefaultHasher::finish` | 0.535 µs | 0.482–0.789 µs |
| Runtime `basis_key` construction under the retained selection | 0.270 µs | 0.265–0.282 µs |
| Complete warm preparation call | 150.857 ms | 146.786–155.527 ms |

The real key issued 103 Rust `Hasher` writes totaling 1,430 bytes and reported
48,016 retained bytes using the existing estimator. The write count describes the
process-local `Hash` traversal, not canonical serialized bytes or a durable hash
preimage. Retained-byte accounting is not process RSS.

The warm operation includes the actual protected read and settlement. The hash
loop is neither an instrumented internal cache lookup nor a measurement of
equality, collision handling or a candidate cached prehash. Its boundary therefore
cannot be subtracted from the warm operation to claim an exact latency share.
These sample ranges are descriptive; they are not confidence intervals.

**Implemented decision:** `BasisKey` now computes one process-local Fx prehash at
construction and passes that cached value to subsequent table hash callbacks.
Its explicit equality still compares the complete selected request, dependencies,
root, instance, bindings and limits. The prehash is a bucket hint, never durable
identity or semantic equality. Existing immutable context and allocation owners
remain retained. This removes repeated full-field traversal without imposing a
minimum-benefit threshold on the observed submicrosecond cost. The measurements
above describe the previous implementation, not an observed speedup of the new
one or an exact end-to-end saving.

The new targeted control
`ordinary_preparation_basis_prehash_preserves_exact_identity_under_forced_collisions`
checks equal keys, changed authored dependencies, a single cached-u64 hash write,
and distinct prepared cache products under deliberately identical prehashes.
It passed in the final 24-test runtime selection below, after temporary probe
hooks were removed.
The [baseline replay procedure](basis-key-replay.md) restores the original hash
callback and construction boundary; including the probe alone in the final code
would measure the new cached callback instead.

## Actual factorable graph interning and durable framing

The temporary [probe source](factorable-inquiry.rs) and
[integration patch](factorable-integration.patch) replace the actual builder's
process-local interner for the inquiry. Both maps retain complete `Node` keys and
use full equality. An untimed capture records the actual admitted
`CasePlan::factorable_program` lookup sequence; the whole-projection timing disables
capture. Replay measures that sequence separately, with no timer per lookup.

**Measured — scalar case:** [raw results](factorable-results.json) retain eight
fresh-process executions of `k4-scalar-preparation`, ordered
Std/Fx/Fx/Std/Std/Fx/Fx/Std. Each process warms the projection three times and records
12 batches of 1,000 complete projections. Replay and framed-key batches contain
10,000 operations each. The actual scalar input consumes one value and has two
columns, two rows and nine retained nodes: nine interner lookups, one hit and eight
misses. Its lookup keys include two binary64 constants, three small rational
constants, two two-child sums and two variables. This is a small workload, not a
thermodynamic graph characterization.

| Operation | Range of the eight process medians |
| --- | ---: |
| Complete projection, four Std processes | 8.786–8.940 µs |
| Complete projection, four Fx processes | 8.297–8.833 µs |
| Captured interner replay, Std | 0.880–0.928 µs |
| Captured interner replay, Fx | 0.382–0.395 µs |
| Exact terminal `MathFactorableV2` framed key replay | 1.172–1.280 µs |
| `MathCaseStructureV6` key traversal | 1.798–2.309 µs |
| Two structure keys and the terminal framed key | 4.810–6.674 µs |

These are descriptive process medians, not confidence intervals. The faster
isolated replay does not establish a representative whole-operation benefit: the
complete projection ranges overlap. Small or noisy differences do not exclude
the selected local optimization under the maintainer's criterion. Both
whole-projection policies carry the same
temporary enum dispatch and capture-disabled branch; replay also allocates its
ordinal result vector. These are inquiry costs rather than production adoption
measurements.

**Tested controls within the measured executions:** all eight runs compare the
complete Std and Fx program structures, not just their keys. All compare the
complete scalar program under a constant hasher and replay all nine captured
lookups under forced collisions. Additional controls preserve normalized rational
equality, distinct signed zeros, unequal variables and order-sensitive child
vectors. All recorded controls passed; their scope is the captured input and the
explicit controls, not every possible graph.

The exact terminal replay agrees with the real program key. Its cost includes
field traversal, framing, rational decimal conversion and BLAKE3; the structure
replay likewise includes the complete existing canonical traversal. Neither
isolates the digest algorithm. Separate replay costs cannot be subtracted from
the projection to claim an exact internal profile. RC02 remains unselected; no
isolated digest comparison or complete durable-replacement contract was assessed.

The recorded scalar artifact SHA-256 is
`f64649a2c972f732b8a66987027467ed004b299c7f7301299e3456058854e432`.
The raw file records its path, source and lockfile hashes, baseline commit,
`dev` profile, `native-process,pse-relations/force-validate` features, dedicated
task store and `timing` execution profile. Build and run use the existing benchmark
route:

```bash
scripts/pse-env --native -- cargo bench -p pse-benches --bench modeling_preparation --locked --profile dev --features native-process,pse-relations/force-validate --no-run --message-format=json
PSE_SURREAL_STATE=<dedicated-task-store> PSE_TEST_EXECUTION_PROFILE=timing PSE_WORKER_BINARY=<worker-artifact> PSE_PREPARATION_CASE=k4-scalar-preparation PSE_HASH_INQUIRY=std PSE_PREPARATION_OUTPUT=<fresh-output-directory> OMP_NUM_THREADS=1 OPENBLAS_NUM_THREADS=1 MKL_NUM_THREADS=1 scripts/pse-env --resource-class timing --native -- <recorded-benchmark-artifact>
```

Run the artifact without `--bench`; the inquiry records its own batches. Repeat in
the recorded policy order with fresh output directories and the actual timing
allocation and matching task-owned service. The existing runtime's local 64-GiB
logical pool is retained; it is not a claim that its pool was replaced by the
managed timing receiver's limit.

The first PC-SAFT-02 attempt refused an archival `data/gross2001.json` file before
factorable projection. The benchmark's legacy directory loader was migrated to
the canonical fixtures' declared-format inventory and accounted owned-document
loader, retaining the same Parquet parameters and authored normal-alkane set.
The next attempt exposed another stale caller assumption: modeling declarations
now remain in parser-owned `Document::modeling_rows()` rather than an eager Arrow
batch. Bank membership now reads those admitted immutable rows directly, as the
current workflow does. Strict import admission then exposed the benchmark
manifest's missing direct dependency on `pse.data.species`; its existing `chem`
import now has the exact declared 1.0.0 owner dependency. The entire canonical
source closure's literal import/manifest mappings were inspected before the next
execution. These are benchmark caller migrations; declared document admission,
scientific model and accuracy are preserved.

The corrected document/import route reached actual cold preparation, which
refused `enthalpy_binding` before factorable projection because its physical
types differed. The [raw refusal](pcsaft-02-inquiry-refusal.json) retains the
declaration/document IDs, source range and `invalid_model` diagnostic. Source
inspection identified stale benchmark variables: `DeltaH` and `Scalar` were
bound to canonical Helmholtz exports `ResidualMolarEnthalpy` and
`LogFugacityCoefficient`. The permanent benchmark now uses those exact exported
types without changing equations, parameter data, selected species, state or
accuracy. The [production smoke](pcsaft-02-production-smoke.json) completed its harness,
but all four preparation stages returned `operation.unestablished_invariant` at case
resolution: no applicable fact checker was supplied. It did not obtain a prepared PC-SAFT
product. The [numerical-fact investigation owner](../../../plans/28f-shared-numerical-preparation.md#pc-saft-numerical-fact-investigation-boundary)
records the current checker-wiring lead and required next evidence. No PC-SAFT interner
comparison or performance result was obtained, and the scalar comparison must not
be generalized to this larger thermodynamic workload.

**Implemented local reuse:** `CaseStructure` exposes shared accessors over private
fields. Its constructors share `build`; `with_native` and `with_requirements` are
the only consuming modifiers, and `CasePlan` retains it in an `Arc`. The current
factorable projection previously computed the same structure key twice. Computing it once
locally and reusing the result is the selected smallest contract-preserving
change. **Implemented:** the process-local interner now uses Fx with complete
`Node` equality, and projection computes one structure key for its existing
terminal frame and retained program field. Temporary policy/capture controls and
both benchmark hooks were removed. Durable key bytes and full equality remain
unchanged. The new production-builder control
`process_local_interner_preserves_exact_node_identity` covers repeated nodes,
ordered children, signed zeros, normalized rational equality and unequal large
adjacent rationals. It passed in the final factorable selection below.
A retained structure key could also be safe with
complete constructor/modifier coverage and unchanged full structural equality,
but it has more invalidation and accounting obligations and is not selected. No
global structure-key cache or durable hash replacement is introduced.

## Permanent implementation acceptance

**Tested — 2026-10-09:** after temporary inquiry hooks were removed and the final
source was formatted, the root ran the following checks in the pinned native
checkout environment with locked dependencies, explicit force-validation and
Nextest's `local` profile. All selected tests passed against the zero failure
target. These are functional checks of the permanent implementation, not a
before/after performance campaign.

The factorable selection passed **47/47**: 46 mathematical projection tests and
the runtime consumer
`compiled_factorable_pricing_retains_separate_demanded_callbacks`. It includes the
new full-node identity control, original-evaluator agreement, arbitrary-precision
rational transport, shared graphs, guards and resource/cancellation refusals.

```bash
scripts/pse-env --store -- just unit-native-package pse-math pse-runtime/native-solvers,pse-runtime/canonical-tests factorable -p pse-runtime --profile local --status-level pass
```

The runtime selection passed **24/24**, including the cached-prehash forced
collision control, exact preparation reuse and many-body inventory settlement,
portable reconstruction, nested kernels, flow tear deadline and authored causal
recycle controls. The runner selected this exact command within the pinned native
environment:

```bash
cargo nextest run --no-fail-fast -p pse-runtime -p pse-relations --lib --locked --features pse-relations/force-validate,pse-runtime/native-solvers,pse-runtime/canonical-tests -E '(test(ordinary_preparation_exact_basis_hit) | test(ordinary_preparation_many_body_inventory) | test(ordinary_preparation_basis_prehash) | test(canonical_portable_body_) | test(kernel_nested_) | test(math::flows::tests) | test(authored_causal_recycle)) & (package(pse-runtime))' --profile local --status-level pass
```

The runtime run identity was `bc33b135-775e-4ede-b2e6-4116895ed34c`. Complete key
equality, collision behavior and receiving/current-context checks remain
authoritative. No end-to-end speedup is claimed for the Fx interner, local
structure-key reuse or cached preparation prehash. Four affected native Python journeys
also passed; their exact selection and conditions are at
[28k](../../../plans/28k-graph-kernels-and-hashing-investigations.md#followup-verification-and-outcome-2026-10-09).
The PC-SAFT smoke returned the scientific refusals described above. The final production
restart control passed with actual portable reuse, exact original history and independent
owner/pool release checks. Scope-end check results and their repaired composite boundaries
are owned by [28k's followup outcome](../../../plans/28k-graph-kernels-and-hashing-investigations.md#followup-verification-and-outcome-2026-10-09).

## Factorable baseline replay

The [integration patch](factorable-integration.patch) applies to the final
production source and restores the measured inquiry implementation: the Std/Fx/
constant-collision interner, capture hooks and benchmark entry points. It also
restores the original two structure-key computations, removes the later
production-builder control and reinstates the stale physical types of the refused
PC-SAFT fixture. That last reversal is necessary to reproduce the recorded
refusal; it is not a production alternative or a recommended scientific model.
The accounted document/import caller corrections remain in this snapshot.

**Interface-checked:** after scope-end formatting, `git apply --check` passed
against the final cleanup source. No production files were patched by that check.
The patch restores the preserved actual inquiry source, including its earlier
formatting. Further source changes may require regeneration against the integrated
revision. These hashes
identify the checked input, not a claim that an independently rebuilt executable
matches the historical artifact:

| Patch input | SHA-256 |
| --- | --- |
| `crates/pse-math/src/factorable.rs` | `8a72da06eff84ce08c154f8e845fb16eefb9c5bdd6ac6c78560b2aaad66c47ac` |
| `benches/benches/modeling_preparation.rs` | `896476160dd711e96433b7aa212ff4958a0584cfd131701502d82e5afb6ab003` |
| `benches/benches/k4/preparation.rs` | `bb94cf55d436ac82f7c42eb9a6441bc0f9e25109d2da3463bb6e39127ae0b8be` |

Use a disposable checkout of the integrated followup revision, preserving the
main checkout. The final exact `rustc-hash = 2.1.3` dependency suffices; this replay
requires no additional production dependency or feature:

```bash
git apply --check docs/design_review/evidence/graph-hash-followups-2026-10-09/factorable-integration.patch
git apply docs/design_review/evidence/graph-hash-followups-2026-10-09/factorable-integration.patch
```

Build and run the artifact through the preceding timing route, record fresh
source/toolchain/lock/artifact identities and raw samples, then reverse the patch.
The original dirty checkout and its entire executable input closure were not
frozen. This patch restores the known interner, framing and caller behavior;
other integrated source or tool changes and host activity can change new timings.
Treat a replay as a new measurement, not an exact historical artifact or a matched
before/after production speedup. The retained probe's presence alone is not
executed evidence or a permanent second hash path.
