# Full-capability pinned library characterization

**Tested, 2026-09-14.** This is library behavior evidence for the
[full-capability design review](../../reviews/design_review_full-arrow-datafusion-capabilities_2026-09-14.md).
It does not execute PSE's RulePlan compiler, sealed admission, optimizer profile,
strata, provenance or publication. It is not a Wave 1 or Wave 2 acceptance run.

## Reproduce

From the repository root, with the pinned toolchain and `.venv` available:

```bash
bash docs/design_review/evidence/full-arrow-datafusion-2026-09-14/run.sh
```

No existing recipe runs an isolated library characterization. This retained script
creates an isolated temporary package, copies the workspace lock, and selects the
workspace's Arrow/DataFusion dependency specifications. Adding the scratch root
requires regenerating only the scratch lock. Before execution, it compares every
resolved third-party name/version against the actual workspace lock and refuses any
new version. Cargo then executes offline and locked. The workspace manifest/lock
are not changed. The scratch package is removed; normal Cargo build caching remains.

**Conditions:** Rust 1.98.1; Arrow 59.3.0; DataFusion 55.1.0; object_store 0.13.2;
dev profile (probe unoptimized, dependencies opt-level 2); Arrow force_validate
explicitly enabled; SessionContext upstream default features/rules, target partitions
1 and 4, batch size 2; finite in-memory inputs. No timing claim. The probe is subject
to its finite fixtures, not a production memory/row-limit prescription.

**Result:** [probe-output.txt](probe-output.txt) records **37 checks, 33 passed,
4 failed semantic expectations, baseline 0; command exit 1**. Only ephemeral scratch
paths were replaced by `<scratch>` in the log. Source and lock checksums identify
the tested bytes; they are not a validation oracle. The probe compares actual row
values/multiplicities and Arrow field/value behavior.

The 34 relational checks use typed ScalarValue extraction with deterministic textual
rendering for these small integer/list examples and an explicit null marker. This
is a test oracle for these fixture types, not a general canonical/equality codec.
The two UDF checks compare actual strings/nulls and fields; the Arrow group compares
actual integer values, fields and encoded signed-zero distinction.

## Counterexamples and positive controls

| Case | Expected | Actual at the pin |
|---|---|---|
| INTERSECT ALL; A=[1,1,2,NULL], B=[1,NULL] | [1,NULL] | [1,1,NULL], in both partition settings |
| EXCEPT ALL; same inputs | [1,2] | [2], in both partition settings |
| Occurrence-window plus null-safe join | [1,NULL] | Expected result in both settings |
| Occurrence-window plus null-safe anti-join | [1,2] | Expected result in both settings |

Raw ALL set builders use ordinary semi/anti joins without matching occurrence
counts: `external/datafusion/datafusion/expr/src/logical_plan/builder.rs:1390`.
The failures remain visible. A baseline of four is **not** accepted. Their purpose
is to refute a naive new lowering and to require a correct one. No upstream issue
or message was published as part of this work.

Passing cases also cover union all/distinct, distinct intersection/difference,
full outer/non-equi joins, correlated EXISTS, tie-preserving dense_rank, grouping
sets, aggregate FILTER/ordering, nested list functions, finite cyclic distinct
recursion, bounded all recursion, field-aware ScalarUDFImpl and Arrow filter/take/
concat. Each case runs under the conditions stated above; these are examples,
not exhaustive conformance of those families. The occurrence lowering's unordered
row_number is used only to pair indistinguishable values for bag counts. It is not
a proposal for stable IDs or source-lineage ordering.

## Pin and feature evidence

`just metadata` (workspace/no-deps recipe) and
`cargo metadata --locked --offline --format-version 1` (full resolved graph) exited
0. The latter has no equivalent full-graph recipe. [resolved-features.json](resolved-features.json)
retains names, versions and actual features for all 49 Arrow/Parquet/DataFusion
packages and object_store. It omits local absolute manifest paths. This is default
workspace resolution; the probe separately enables force_validate.

The live reading copies were checked directly:

- DataFusion commit `7d3835c71f30cbd3c3ae4041732267f1f453097a`, workspace version 55.1.0.
- Arrow commit `f90e061326bd821a7af09281d9e92de6f3b603d9`, workspace version 59.3.0.

Cargo resolution, not these checkouts alone, selects the probe dependencies.
Transitive features matter: Arrow IPC lz4/zstd and datafusion-sql unparser are
resolved even when umbrella manifest comments suggest a smaller feature set.
Arrow tracking pool, DataFusion FFI/Substrait/Spark and Arrow Flight/Avro/Parquet
Variant companions were not selected by this workspace graph. Their eligibility
does not mean they were compiled or tested here.

Context7 was used for discovery, including `/apache/datafusion` planning,
extension and recursion guides and `/apache/arrow-rs` compute documentation.
Versioned [ScalarUDFImpl rustdoc](https://docs.rs/datafusion-expr/55.1.0/datafusion_expr/trait.ScalarUDFImpl.html),
[AggregateUDFImpl rustdoc](https://docs.rs/datafusion-expr/55.1.0/datafusion_expr/trait.AggregateUDFImpl.html),
[WindowUDFImpl rustdoc](https://docs.rs/datafusion-expr/55.1.0/datafusion_expr/trait.WindowUDFImpl.html)
and [LogicalPlanBuilder rustdoc](https://docs.rs/datafusion-expr/55.1.0/datafusion_expr/logical_plan/builder/struct.LogicalPlanBuilder.html)
were checked alongside pinned source. Historical rustdoc JSON was not regenerated;
no claim in this receipt depends on an absent generated extraction.
