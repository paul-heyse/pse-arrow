# cargo-hack

Realizations considered: installed `cargo-hack` 0.6.45, from `cargo hack --version` in the
checkout environment. It is a host tool.
Research date 2026-10-27. Sources: local adapter module docstring and recipe
(`features-powerset`). That docstring cites the upstream `src/main.rs` `try_main` selection
of `CARGO_HACK_CARGO_SRC` before `CARGO`. Upstream docs were not re-read; this record is a
premise correction rather than new capability research.

## Types

### cargo-hack/feature-powerset-enumeration
- **Need it serves:** Check every feature combination, up to a depth, for each workspace
  package, and keep going after failures.
- **Recognizable code shapes:** Scripts that enumerate feature sets and loop `cargo check`.
- **Mechanisms and APIs:**
  - `cargo hack check --feature-powerset --depth N`.
  - `--no-default-features`, `--each-feature` and `--keep-going`.
  - `--exclude-features` and `--group-features`.
  - The cargo executable can be substituted through `CARGO_HACK_CARGO_SRC`.
- **Obligations discharged:** Enumeration, ordering and keep-going failure accounting.
- **Residual caller obligations:** Any rewriting of the per-invocation cargo arguments,
  for example to preserve workspace feature unification or add validation features.
- **Semantic conditions:** cargo-hack's printed command lines are display text, not a
  stable interface.
- **Version / build conditions:** Behaviour as of 0.6.45, per the local docstring.
- **Limits and negative results:** No hook to transform each generated cargo invocation
  other than substituting the cargo executable.
- **Evidence:** Interface-checked: local adapter docstring citing upstream source. Not
  independently re-read.

## Current applicability

### [CBO-049](../../../library_leverage/codebase/developer-tooling.md#cbo-049) — research question "Does cargo-hack cover cargo_feature_check.py?"
- **Types:** `cargo-hack/feature-powerset-enumeration`.
- **Realization:** Installed 0.6.45.
- **Conclusion:** The premise is falsified. The script is not a parallel implementation of
  powerset checking. It is the `CARGO_HACK_CARGO_SRC` adapter that cargo-hack invokes, and
  it rewrites bare `--features` of the selected check package so that workspace feature
  unification is retained. cargo-hack already owns enumeration, order and keep-going
  accounting. No further library route was found.
- **What it would discharge / what remains local:** Nothing further. The adapter is the
  extension point cargo-hack offers.
- **Prerequisites, configuration or extension:** n/a.
- **Semantic gaps against the CBO's preservation constraints:** n/a.
- **Unsettled premises and what would settle them:** Whether the adapter is still needed
  under the current Cargo feature-unification setting. Running `cargo hack` directly on
  one package and comparing resolved features would settle it. This is a probe, and it was
  not run.
- **Evidence:** Interface-checked (local source); application Proposed.
