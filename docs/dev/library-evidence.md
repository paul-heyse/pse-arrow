# Library evidence workflow

Use `just library-evidence --output NEW_DIR --family tokio` to bundle existing acquired
API facts and observed dependency references. Repeat `--family` for several families;
omission selects every family in `.config/library-leverage.toml`. The output directory
must be new and its parent must exist. The family configuration selects package names
and reference documents; locked Cargo metadata owns versions and sources.

Preparation is explicit. Add `--prepare` to prepare the native analysis tool. Prepare
the reusable semantic session separately with `scripts/pse code session prepare
--backend native-snapshot` and the desired context flags, before running this recipe.
The native service owner prohibits independent detachment from an enclosing workload;
the bundle must reuse that separately prepared session. Add `--acquire` to acquire selected packages' rustdoc
artifacts. These operations may compile dependencies; neither runs scientific workflows.
Reads never silently prepare missing inputs. Native build-script/proc-macro execution
requires `--build-macros`. `--features`, `--no-default-features`, `--target`, and
`--test-context` select the semantic context; they do not promise compiler-unit equivalence.
API exports retain their separately acquired documentation context and feature selection.

For example, after reviewing the requested effects:

```bash
scripts/pse tools prepare native-analysis
scripts/pse code session prepare --backend native-snapshot
just library-evidence --family symbolica --output /tmp/symbolica-evidence --acquire
```

The Rust `xtask library-evidence` owner composes the native commands; the recipe is a
thin launcher. `metadata.json` captures the selected package population. `references.json`
retains the original batch semantic response. Numbered package/export captures and `api-N/`
tables retain exact package IDs, documentation context and source evidence. `manifest.json`
maps those files and records every step's original status and streams. `README.md` gives
bounded navigation and an alphabetical sample of declaration labels. Labels and counts
are observations, not importance rankings or capability recommendations.

Independent package operations continue after individual failures. A partial bundle remains
available and the command returns nonzero. `capture_complete` means commands succeeded;
read each response's coverage before interpreting absence, ambiguous profiles, documentation
availability, or runtime use. Keep original failures rather than replacing them with empty
success. Re-run into another output directory after correcting prerequisites.

The [discovery workflow](../library_leverage/README.md) interprets these facts alongside
the comprehensive overviews, primary documentation and representative local source. It
owns the concise briefing and potential-value hypotheses. Existing library assessments
own selective technical contracts; comparative reviews own adoption recommendations.
Generated evidence is never edited into a recommendation and never overwrites authored
briefings. Retain bundles while research consumes them, or reproduce the exact source and
acquisition context before refreshing claims.
