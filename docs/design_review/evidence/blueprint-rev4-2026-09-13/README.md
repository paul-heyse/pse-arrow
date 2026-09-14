# Blueprint revision 4: library characterization evidence

This evidence accompanies the
[design review](../../reviews/design_review_blueprint-rev4-library-contracts_2026-09-13.md).
It characterizes library mechanisms used by the proposed design; it does not
exercise an implemented `pse` modeling pipeline.

| File | Purpose |
|---|---|
| [probe.rs](probe.rs) | Six characterization groups, including positive controls and counterexamples |
| [run.sh](run.sh) | Reproduction from repository pins and lockfile, with an ephemeral Cargo project |
| [output.txt](output.txt) | Final successful execution receipt, input hashes, versions and observations |

From the repository root:

```bash
bash docs/design_review/evidence/blueprint-rev4-2026-09-13/run.sh
```

The runner uses `.venv/bin/python`, the Rust toolchain declared by
`rust-toolchain.toml`, workspace dependency declarations and a copy of `Cargo.lock`.
The scratch root requires a local lockfile adjustment; every resulting third-party
version is checked against the repository lock before offline, locked execution.
The repository manifest and lockfile are not modified. Compilation uses the
repository target cache by default; `CARGO_TARGET_DIR` can select another cache.
The temporary project is removed on exit. A later dependency change constitutes
a different experiment; the receipt records the relevant hashes and versions.

**Tested conditions:** Rust 1.98.1, Arrow 59.3.0, DataFusion 55.1.0, dev profile,
Arrow `force_validate` explicitly enabled. The scratch `force-validate` feature
maps directly to `arrow/force_validate`; it is not a platform test-package feature.
Failure baseline: **zero**. The final run completed **6 characterization groups
with 0 assertion failures**. Some assertions deliberately establish that a claimed
guarantee is false. The counts are not platform acceptance results.

E1 uses default DataFusion features plus a registered extension factory; it calls
the validator directly as a control and then executes an ordinary query. E2 uses
nonconstant input columns to prevent constant folding from disguising eager
evaluation. E3 distinguishes successful casting from exact representation. E4
uses equal nullable integer values with different hidden payloads, ordered primary
keys, `take_record_batch`, concatenation and canonical-option IPC streams, then
compares complete physical file encodings. E5 inserts eight metadata keys in
sorted order into 32 independently built maps; the provider codec contributes
fixed bytes, so random provider identities do not explain the differences. E6
uses ordinary floating-point arithmetic, not a platform optimizer.

The retained E5 run produced 32 distinct protobuf encodings and one IPC encoding;
the exact first count can vary. Its assertion requires more than one, which is
enough to refute reproducibility from sorted insertion. No throughput, latency,
memory benchmark, solver behavior or scientific parity is established here.

The first scratch attempt lacked the custom table-provider protobuf codec and
stopped after E4. That harness error was corrected; the retained source and
receipt are from the completed version. No existing capability-map receipts were
overwritten or reclassified as newly executed evidence.
