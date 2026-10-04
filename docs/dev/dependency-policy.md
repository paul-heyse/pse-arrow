# Dependency and licence policy

> **Decision: [ADR-0066](../adr/0066-dependency-admission-and-licence-policy-are-advisory.md)**
> (admission) · [ADR-0159](../adr/0159-dependencies-float-exact-pins-need-a-reason.md)
> (versions float; pins need a reason) · Blueprint §3.1, §3.3.2 · Register row R-31

## The short version

**Use the library.** During phases 0–1 no third-party dependency is refused, and no
licence is grounds to refuse one. Adding a crate or a Python package needs no ADR, no
design review, and no revision row in the architecture. Nothing in CI blocks a merge because of what
you depended on or what licence it carries.

**Versions float; the lockfiles pin.** Use the tool's default specifier — the `cargo add`
caret, declared in `[workspace.dependencies]`, or `uv add` (a `>=` floor). `Cargo.lock` and `uv.lock`
record what resolved, and `just upgrade` (or `just upgrade <package> …`) moves them to the
latest compatible versions, at your discretion; it then regenerates the cargo-hakari
workspace-hack, runs `just family-check` and prints what moved. Run the tests the move
affects. A caret never crosses a major: crossing one is an explicit requirement bump in the
manifest. Dependabot (ADR-0035) proposes the same lock moves as grouped pull requests with
`versioning-strategy: lockfile-only`; either route is fine.

### Pin reasons

This is the one list of acceptable reasons ([ADR-0159](../adr/0159-dependencies-float-exact-pins-need-a-reason.md)).
An exact version, upper cap (`<`, `<=`, `~`, a wildcard), git revision or hold-back needs an
overt reason specific to that dependency, from this list:

- a named breakage or incompatibility;
- a type-sharing family that must resolve to one version;
- a fork, git revision or vendored source;
- a content-addressed acquisition;
- committed or byte-stable generated output, or a private or unstable API use;
- a parity or reference oracle;
- an experimental control;
- platform or wheel availability.

Not reasons: reproducibility (the lockfiles give it), "already in the lock", "latest on date
X", and a version entering a key or digest (a bump re-keys by design).

Record the reason beside the pin. In Rust, family members are covered by
`[workspace.metadata.pse.families]` and must be exact at the family's declared version;
every other pin has a one-line entry in `[workspace.metadata.pse.pins]`, and
`dependency_pins` checks both. In Python, the reason is a comment beside the `==` pin in
`pyproject.toml`; it is instruction-only, with no test (ADR-0159). Remove a pin, and its
reason, when the reason lapses.

**This is not a constraint to design or execute around.** If a library gets the platform
working, take it. The goal right now is a functional codebase — an ambitious one — and a
dependency you can characterise later is cheaper than an implementation you write today
to avoid it. Substitution is a decision to make *after* the role is understood and the
behaviour is testable, backed by evidence, not a constraint to anticipate now.

## What this replaced

The repository used to treat its dependency set as a closed list: every
`[workspace.dependencies]` entry had to appear in blueprint §3.1 or in
`tests/governance/tooling_deps.toml`, `deny.toml` banned ten named crates and allowed
fourteen licences ("copyleft is absent by omission: anything not listed fails"), git
dependencies were forbidden outright, and six documents repeated that adding a dependency
needed an ADR plus a design review. That posture is right for a hardened v1. It was wrong
for a phase-0 platform still establishing what it is.

## What is still enforced, and why

Relaxing admission and floating versions did not relax reproducibility. These remain
project invariants, checked when the corresponding command is run manually:

| Gate | What it protects |
|---|---|
| `just family-check` (`rust / family-check` when manually dispatched) | **One type universe.** Exactly one resolved `arrow`, `parquet`, `object_store`, `datafusion` and `pyo3`. Two majors make `downcast_ref` return `None` with no compile error — a silent failure that reads like a logic bug. This is the invariant `deny.toml`'s `multiple-versions` used to stand in for, and it states it far more precisely. |
| `tests/governance/tests/dependency_pins.rs` | **Reasoned pins.** A requirement in `[workspace.dependencies]` that bounds a version from above (`=`, `<`, `<=`, `~`, a wildcard) or a git revision is a declared family member or has a reason in `[workspace.metadata.pse.pins]`; family members are exact at the family version; a git source names a full commit; a reason for a floating dependency is stale. Blueprint tables are not a second pin authority. |
| Committed `Cargo.lock` and `uv.lock`, every gate `--locked` | Reproducibility. Versions move only through the lockfiles, with `just upgrade`; a run never resolves anew. |
| ADR for **majoring** one of the four pinned families | A family major changes the API surface the capability maps were extracted against. |
| ADR for adding or removing a **workspace** (`pse-*`) crate | A crate boundary is architecture, not a dependency. |
| `reuse lint` | SPDX headers on **our own** files. Unrelated to third-party licences. |
| import-linter contracts in `pyproject.toml` | Architecture, not hygiene: numpy/scipy at the array boundary, pyomo/pint in the adapter, `idaes` in the parity environment only (uv workspaces enforce a single `requires-python`, so an `idaes` import inside `pse` would drag the platform package back to the parity interpreter range). |
| The clean-room rule (CONTRIBUTING.md §8) | *Copying* code from a reference implementation. Unchanged, and a different question from *depending* on a library. |

## Running the check manually

The machinery is all still installed. It reports instead of gating.

```bash
just deps-report   # advisory: licences, advisories, bans, unused deps. Always exits 0.
just policy        # strict: the same checks, exiting non-zero on a finding.
```

`deny.toml` is that report's configuration. Its `[licenses] allow` list is no longer an
allowlist — it records the licences a future review has already looked at, so the report
can say *"here is one nobody has considered yet"* instead of drowning in the ordinary.
`rust / deny` is available through manually dispatched `rust.yml`, with
`continue-on-error: true`.

## Known hazards that survived as prose

Three of the ten crates `deny.toml` used to ban encoded a semantic trap rather than
hygiene. None is banned now — but read the reason before reaching for one:

- **`datafusion-spark`** — arithmetic that differs silently from ours (blueprint §3.3).
- **`parquet-variant`** — a typed hole in decision D1 (blueprint §3.3).
- **`parquet_derive`** — inverts the registry → Rust generation direction (blueprint §3.3).

The other seven (`serde_yaml`, `uom`, `arrow-flight`, `arrow-avro`, `arrow-pyarrow`,
`salsa`, `openssl-sys`) were scope or hygiene calls that no longer bind. Their reasoning is
kept in the comment block above `deny.toml`'s now-empty `deny = []`.

## What this costs, and when it gets paid

A copyleft dependency linked into a distributed artifact can affect what
`MIT OR Apache-2.0` means for what we ship, and nothing now catches that automatically.
That is a real cost, deliberately accepted while nothing is published: every crate is
`publish = false` and there is no PyPI release.

Register row **R-31** carries the obligation. Its trigger is *the first crate that flips
to `publish = true`, or the first PyPI release*; its check is `just policy`. The point of
keeping every mechanism installed is that answering it then costs a command, not a
rebuild.
