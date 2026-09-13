---
description: Invariants for the Rust workspace
paths:
  - "crates/**"
  - "xtask/**"
  - "benches/**"
  - "tests/**"
---

# Working in the Rust workspace

## One type universe

Exactly one resolved version of `arrow`, `parquet`, `object_store` and `datafusion` may
exist in the graph. Two majors make `downcast_ref` return `None` with no compile error —
the failure is silent and reads like a logic bug (blueprint §3.1).

- `=` pins bind direct dependencies only, and `cargo tree -d` cannot flag a mixed family
  because each package name still appears once. **`just family-check`** asserts the
  resolved graph against `[workspace.metadata.pse.families]`. Run it after touching any
  manifest.
- Import Arrow through `datafusion::arrow::…` inside engine crates; the PyO3 boundary in
  `crates/pse-py` speaks `pyo3-arrow`'s types instead.
- `object_store` is `=0.13.2` deliberately: DataFusion 55 requires `^0.13.2`, so a newer
  0.14.x would split the graph.

## The bans are type-aware, not stylistic

`clippy.toml` denies these because each one panics or fabricates authority:

- `Field::extension_type()` — panics on a missing or invalid extension; use
  `try_extension_type` (§4.4).
- `SessionConfig::set_str` — panics on an invalid key; go through typed `ConfigOptions`
  (§23.2, error code `config.invalid`).
- `SchemaLike::from_type` / `from_samples` — schemas come from the registry, never
  inferred (§5.3).
- `IpcWriteOptions::try_with_compression` — canonical IPC is uncompressed (§5.3,
  `pse.canon.v1`).
- `anyhow::Error` — every `pse-*` crate returns a concrete `thiserror` enum that also
  derives `miette::Diagnostic` with a §23.2 code. `xtask` opts out at crate level.

`unwrap`/`expect`/`panic`/`todo`/`print*`/`dbg!` are denied outside tests for the same
reason: a panic crossing the PyO3 or Ipopt boundary is an abort risk.

## Lint escapes carry a reason

`allow_attributes_without_reason` is denied, so every escape is
`#![allow(clippy::some_lint, reason = "why this case is different")]`. An `#[allow]`
without a reason will not compile in CI.

## Profiles and floating point

- **`panic = "unwind"`** in every profile. PyO3 converts unwinds into Python exceptions
  and the Ipopt callbacks `catch_unwind`; `abort` takes the interpreter down.
- **Never `target-cpu=native`**, and never add `[build] rustflags` to `.cargo/config.toml` —
  a contracted FMA changes float results between your machine and CI. A governance test
  forbids both.

## Before you claim it works

`just ci-fast` covers fmt, `cargo check`, clippy `-D warnings`, nextest and doctests.
**nextest does not run doctests** — that is why `just doctest` is a separate step, not
redundancy. Every test invocation passes `--features pse-relations/force-validate`;
`just test` does it for you, a bare `cargo nextest run` does not.

`just governance` runs the workspace-level assertions (pins match the blueprint, every
crate registered, MSRV equals the toolchain, dependency floors, unsafe allowlist, error
taxonomy). It is cheap. Run it after any manifest change.
