# Wave 1 logical-plan construction characterization

**Tested, library behavior only.** Run:

```bash
bash docs/design_review/evidence/wave1-logical-plans-2026-09-14/run.sh
```

The recipe surface has no isolated design-review library-probe recipe. This retained
runner creates a temporary package, takes dependency declarations and toolchain from
the workspace, copies its lockfile, and checks that every resolved third-party
name/version occurs in that lockfile. The temporary package addition may change its
own copied lock; execution is then offline and locked. Scratch cleanup is automatic.
It shares the normal target cache and does not change production manifests or code.

Conditions: DataFusion 55.1.0, Arrow 59.3.0, Rust 1.98.1, dev with dependency opt-level
2, Arrow `force_validate`. The first six checks use schema-only plans with no data
batches. The last two use the default native SessionContext on tiny finite VALUES
inputs. This is not a PSE runtime or performance test.

Result: **8 passed, 0 failed, baseline 0; exit 0.**

1. Unknown column rejected at construction.
2. Direct projection/alias retains semantic metadata and nullability.
3. Field-aware function derives semantic output before execution.
4. Same physical type with wrong semantic domain rejected at construction.
5. All those construction checks invoke the function body zero times.
6. Native expected-schema comparison accepts a metadata/nullability mismatch,
   identifying the application contract dimensions it does not check.
7. `IS NOT TRUE` violation selection retains false and unknown cases.
8. Native anti join expresses missing-reference detection.

The application-defined function intentionally demonstrates a small domain contract;
it is not production quantity typing. The schema-only `LogicalTableSource` is not
used as a stand-in for production provider optimization/pushdown behavior.
The two diagnostic execution checks count tiny witness sets; they do not establish
complete generated diagnostics or qualify every join/predicate case.

See [source](probe.rs), [runner](run.sh), [raw receipt](probe-output.txt), and the
[review](../../reviews/design_review_wave1-logical-plan-foundations_2026-09-14.md).
SHA-256 values in the raw receipt identify source/lock bytes only; none is used as
semantic validation or a proof of an execution result.

Documentation validation on 2026-09-14:

- `just lint-typos`: exit 0, no findings, baseline 0.
- `just docs`: exit 0; one nonblocking large-search-index warning
  (over 11 MB). This builds the book; it does not certify architectural claims.
- Scoped `git diff --check`: exit 0 for the plan checkpoint and book-index edits.
- Scoped local-link/whitespace check: two new documents, zero missing targets or
  trailing-whitespace findings, baseline 0.

The probe was formatted with the pinned Rust 1.98.1 formatter, then rerun so the
source identifier in the receipt matches the retained file. No main-workspace
Rust, Python or solver acceptance run is claimed.
