---
title: Provider contract characterization evidence
date: 2026-09-15
evidence: Tested
---

# Provider contract characterization evidence

These probes support the [provider review](../reviews/design_review_provider-contracts_2026-09-15.md).
They characterize the inspected implementation and upstream mechanisms. Passing a
probe that asserts a current defect does **not** establish target conformance.

## Result

**Tested:**

```text
just test-package pse-catalog --test provider_contracts_review_probe \
  --no-fail-fast --status-level fail --final-status-level fail
```

Rust 1.98.1; DataFusion 55.1.0; default nextest/test profile; explicit
`pse-relations/force-validate`; failure/warning baseline **0**.

Final run `ee6f2cfe-7d58-4058-8051-75d69a68916a`: **5 passed, 0 failed,
0 skipped; 0 compile warnings/errors**. Test duration 0.023 s; incremental test build
1.19 s. These durations describe the probe run, not product workload performance.
The full resolution and selected file hashes are in the
[context record](provider-contracts-context-2026-09-15.json).
The PSE session probes use a 64 MiB `FixedBudget`, one pool thread and one target
partition. They are small semantic characterizations, not resource-limit tests.

| Probe | Observed behavior |
|---|---|
| `direct_role_scan_works_but_sql_hierarchy_cannot_resolve_it` | A bound `before` role returns one row through direct native preparation; `SELECT id FROM roles.inputs.before` fails with not-found |
| `ordinary_information_schema_is_outside_the_admitted_inventory` | The ordinary table metadata query fails source-inventory admission |
| `sealed_catalog_list_registration_returns_none_but_does_not_register` | Root registration returns `None`, then lookup still returns absent |
| `view_resolution_can_inline_without_a_table_scan` | Native scan construction for a `ViewTable` returns an alias over the inner plan, without a retained table scan |
| `native_sql_ddl_changes_the_namespace_before_dataframe_collection` | `SessionContext::sql(CREATE TABLE ...)` registers the table before its returned DataFrame is collected |

The DDL probe uses an isolated in-memory native session. It does not write a product
store. No remote snapshot, concurrency, generic provider admission, publication,
scientific or Python conformance claim follows from these five observations.

## Reproduction

The retained [Rust source](provider-contracts-probe.rs) is outside production modules.
For this run it was included through a temporary integration-test harness at
`crates/pse-catalog/tests/provider_contracts_review_probe.rs`:

```rust
//! Temporary harness for the provider-contract design review evidence.
#[path = "../../../docs/design_review/evidence/provider-contracts-probe.rs"]
mod review;
```

Create that harness only if the path is absent, run the recipe above, and remove
only that temporary harness afterward. The review run removed it. The retained
source intentionally asserts current gaps; when implementing the target, create
positive conformance tests instead of treating these characterizations as invariants
to preserve.

Final command output:

```text
Compiling pse-catalog v0.0.1
Finished `test` profile [unoptimized + debuginfo] target(s) in 1.19s
Nextest run ID ee6f2cfe-7d58-4058-8051-75d69a68916a with nextest profile: default
Starting 5 tests across 1 binary
Summary [0.023s] 5 tests run: 5 passed, 0 skipped
```

Probe setup initially needed corrections for an `OwnedRecordBatch` method coercion,
the pinned `datafusion_catalog::view::ViewTable` import and the temporary harness's
crate documentation. The final clean run includes all corrections. No production
code was changed to make these probes pass.

## Review artifact checks

- `just lint-typos`: 0 findings, baseline 0.
- `git diff --check`: 0 findings, baseline 0.
- A focused structural check verified sections 1–11, balanced code fences, all local
  Markdown links in the review/receipt, and unchanged hashes for 11 inspected PSE
  source files: 0 failures, baseline 0.
- `just docs`: book build completed. It emitted **1 warning**, baseline 0:
  `search index is very large (12898459 bytes)`. No global book configuration was
  changed for this review. The book build alone does not validate an unindexed
  review document; the focused checks above cover its local structure and links.
