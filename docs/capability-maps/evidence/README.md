# Evidence for the capability maps

Reproduction material for the `[probe]`, `[rustdoc:…]` and `[api:…]` provenance markers in the
four maps in the parent directory (`arrow-rust.md`, `datafusion-rust.md`,
`supporting-rust-libraries.md`, `python-libraries.md`).

| Directory | Covers |
|---|---|
| `rust/` | arrow 59.3.0 / datafusion 55.1.0 and the §3.3 supporting crates: `=`-pinned manifests, **committed lockfiles**, rustdoc-JSON generation scripts and their logs, eleven probe programs and their captured outputs, and `checkmap.py` (the mechanical check the maps claim) |
| `python/` | the Python boundary libraries: pinned requirement sets for the platform (3.14) and parity (3.13) interpreters, `apidump.py`/`apiq.py` (the rustdoc-JSON analogue), three probe programs |
| `arrow_probe.rs` | Arrow probes 1–5 (IPC metadata round-trip, null/NaN/`-0.0` distinctness, `totalOrder` sort, batch-split byte equality, alignment sensitivity), still quoted by `arrow-rust.md`; run it as a `src/bin/` target of the `rust/` scratch project |

Regenerate with `just evidence-regen` (see `rust/README.md` and `python/README.md` for the manual
steps). The extracted fact profiles (`build/facts/*`) are derived output and are not committed;
the lockfiles are what make the extraction reproducible.

Note on pinning: every crate is `=`-pinned individually and deliberately. Pinning only the umbrella
(`arrow = "=59.3.0"`) does **not** pin the family — sub-crates depend on `^`, so they float upward,
and `cargo tree -d` does not flag it because a mixed family is not a duplicate. `cargo xtask
family-check` exists because of this.
