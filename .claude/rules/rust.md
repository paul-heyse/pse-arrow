---
description: Invariants for the Rust workspace
paths:
  - "crates/**"
  - "xtask/**"
  - "benches/**"
  - "tests/**"
  - "Cargo.toml"
  - "Cargo.lock"
  - "rust-toolchain.toml"
  - "rust-analyzer.toml"
---

# Working in the Rust workspace

## Reach for the library

No third-party crate is refused and no licence is grounds to refuse one through phases
0–1. `deny.toml` bans nothing, `cargo deny` reports without gating, and blueprint §3.1 is
about version authority, not admission. Adding a dependency needs no ADR and no design
review — declare it in `[workspace.dependencies]` pinned exactly (`cargo add name@=x.y.z`),
inherit it with `.workspace = true`, commit `Cargo.lock`. Bump a pin when the work calls for
it, then `just upgrade <crate>`; never a bare `cargo update` unless asked. A hold (a pin not
to bump casually) has a reason in `[workspace.metadata.pse.pins]` (`dependency_pins`,
ADR-0165). Read
[`docs/dev/dependency-policy.md`](../../docs/dev/dependency-policy.md) before assuming
something is off-limits; it also says what *is* still enforced, starting with the next
section.

## One type universe

Exactly one resolved version of `arrow`, `parquet`, `object_store` and `datafusion` may
exist in the graph. Two majors make `downcast_ref` return `None` with no compile error —
the failure is silent and reads like a logic bug (blueprint §3.1).

- `=` pins bind direct dependencies only, and `cargo tree -d` cannot flag a mixed family
  because each package name still appears once. **`just family-check`** asserts the
  resolved graph against `[workspace.metadata.pse.families]`. Run it when a manifest
  change moves or adds a dependency in one of those families.
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
- `anyhow::Error` — every `pse-*` crate returns a concrete `thiserror` enum that also
  implements `miette::Diagnostic` with a typed §23.2 code. `xtask` opts out at crate level.

`unwrap`/`expect`/`panic`/`todo`/`print*`/`dbg!` are denied outside tests for the same
reason: a panic crossing the PyO3 or Ipopt boundary is an abort risk.

Canonical identity IPC remains uncompressed (§5.3, `pse.canon.v2`).
The governance pattern restricts `try_with_compression` only under
`crates/pse-columnar/src/canon`; the API is available for noncanonical transport
(blueprint §3.3.1). Compression is not a library-wide prohibition.

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

Follow *Execution rhythm* in AGENTS.md. While implementing, `just check-package <pkg>`
(or `just check` across crates) and the targeted `just unit-package <pkg> <filter>` tests
for the behaviour you changed are the whole loop (`just affected` previews what a change can
reach). Correctness routes must activate and verify Arrow validation for the actual
normal/build/selected-root dev closure, including host dependencies (ADR-0170). Pure
closures need no Arrow feature or unrelated relation root. Bare commands carry the same
obligation; selecting a feature name alone is not proof of leaf activation.
Delete a replaced mechanism with its tests as soon as the replacement's tests pass and
its callers have moved.

Do not run `cargo fmt` mid-work (`just turn-end` formats at the end of the turn), or `just clippy` and
integration suites mid-plan: clippy runs in `just hygiene` at scope end, and you fix what it
reports (ADR-0161). Once all functional scope in the plan is
implemented (or when the maintainer requests comprehensive qualification), `just ci-fast` covers fmt,
`cargo check`, clippy `-D warnings`, nextest and doctests — **nextest does not run
doctests**, which is why `just doctest` is a separate step. `just governance` covers
pin reasons, crate registration, the nightly pin against the `rust-version` floor, dependency
floors, unsafe allowlist and error taxonomy.
Select integration and performance checks for the implemented scope. These aggregates
are not commit, push or merge gates.
