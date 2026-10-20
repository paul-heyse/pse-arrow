# Pipeline diagnostics and offline review-artifact evidence

Supporting investigation for the [principal review](../../reviews/design_review_pipeline-diagnostics-and-review-artifacts_2026-10-18.md), dated 2026-10-18. This folder supplies bounded source/library analysis and a passive-reader control; it is not an implementation plan or a second disposition ledger.

The source baseline was `e9d24396c`. During the inquiry, existing documentation and the two library notes were committed externally as `6524e8591`; a source diff over `crates`, `python`, `scripts`, `Cargo.toml` and `pyproject.toml` showed no intervening changes. The coordinator made no commits and preserved the existing Plan46 documents. A fresh principal reviewer independently assessed the evidence and target. All proposed corrections remain **Proposed**.

## Source and library inquiries

- [Native and mathematics capabilities](native-capabilities.md): actual consumer/fork versions, existing native work and evaluator signals, coordinate limitations, safe getter candidates and genuine missing producer detail. **Interface-checked**, not new runtime qualification.
- [Analysis and publication capabilities](analysis-and-publication-capabilities.md): shared Rust interpretation, typed event projection, DataFusion/Perfetto contracts, CPU-profile limitations and portable publication. **Interface-checked**, with current documentation treated as discovery where it differs from the consumed release.

The coordinator inspected decisive runtime clock/window reading, CLI ingress, invocation comparison, source-map and replay/result-export contracts, native/evaluator observations, trace queries and detail-publication metadata. Library researchers used the relevant pinned capability skills and Context7, checking consequential claims against actual consumed source. Neither stale `docs/library-utilization.jsonl` nor `docs/capability-maps/` was used.

## Passive-reader control

**Tested:** current `just diagnostics --help` exited zero and exposed `list`, `inspect`, `capture`, `capture-launch` and `read-detail`. The light route loaded the current editable extension without a new simulation or store.

**Tested:** [reader-probe.py](reader-probe.py) inspected an existing 868,800-byte version-4 IDAS checkpoint through the current Python CLI entry point, with a 60-second window and related-checkpoint traversal disabled. [reader-probe.json](reader-probe.json) retains the compact result. Both CLI calls exited zero against the zero-error target. No new numerical run occurred.

The original checkpoint contains 256 detail records and 34 recent producer events across six producers for the selected operation. Default inspection advances `read_at` using the reader's live monotonic clock and returns zero detail/recent records. A temporary copy with only `clock` set to null exercises the existing unknown-clock fallback: it anchors at the recorded read time and retains all 256/34 records. This synthetic control is not a proposed fix or an original capture. The source bytes were checked unchanged. The temporary control was removed automatically.

Both views retain the original sampled/curtailed coverage, 102 dropped detail items and 372 refused items. The fallback explicitly reports partial transport. The probe establishes a live-versus-frozen reading distinction; it establishes neither full capture coverage nor a numerical performance improvement. Terminal-summary expiry is a separate stale flag and does not delete cumulative facts. The current raw reader itself preserves the original records.

To repeat with the retained local fixture:

```bash
BASH_ENV='' PSE_TEST_RESOURCE_STATE=/tmp/pse-diagnostic-review-resources-20261018 \
  scripts/pse-env --native=solver,klu,isolation,uno,petsc --resource-class light -- \
  .venv/bin/python \
  docs/design_review/evidence/pipeline-diagnostics-and-review-artifacts-2026-10-18/reader-probe.py \
  build/p44-density-22s_40tp/cases/compact-idas-ode-5/diagnostics/checkpoints/000000006ad40b87-000c546e08367563.json
```

This local fixture is not embedded in the review evidence folder. The compact observations and source diagnosis remain readable without it; rerunning the control requires an available compatible checkpoint. Timing values are observations of passive inspection, not a solver benchmark.

## Limits and exclusions

No new simulation, native solver campaign, profiler capture, service, database, dependency change, product edit or environment reconstruction was needed. No observer-overhead, symbol-completeness, external-recipient usability or large-artifact feasibility claim was measured. Existing Plan44 files were used only as bounded reader fixtures and examples of capture metadata; their old numerical timings are not current performance results. Scientific model conformance and whole-product qualification were not rerun. This review does not authorize uploading artifacts or implementing its recommendations.
