# Dependency and licence policy

> **Decision: [ADR-0066](../adr/0066-dependency-admission-and-licence-policy-are-advisory.md)**
> (admission) · [ADR-0165](../adr/0165-declared-dependencies-pinned-exactly.md)
> (declared dependencies are pinned exactly) · Blueprint §3.1, §3.3.2 · Register row R-31

## The short version

**Use the library.** During phases 0–1 no third-party dependency is refused, and no
licence is grounds to refuse one. Adding a crate or a Python package needs no ADR, no
design review, and no revision row in the architecture. Nothing in CI blocks a merge because of what
you depended on or what licence it carries.

**Declared dependencies are pinned exactly.** Every registry dependency is `=x.y.z` in
`[workspace.dependencies]` (`cargo add name@=x.y.z`) and `==x.y.z` in `pyproject.toml`
(`[tool.uv] add-bounds = "exact"` makes `uv add` write it); a git dependency is pinned by its
full `rev`. `Cargo.lock` and `uv.lock` hold everything beneath, so an environment changes only
when someone changes it on purpose.

**Change versions deliberately, on judgment.** Add or bump a dependency when the work calls
for it, with no ADR or approval: edit that pin (or its family), run
`just upgrade <package> …` (it moves the named packages, regenerates the cargo-hakari
workspace-hack, runs `just family-check` and prints what moved), check that the lock diff
moved only what you meant, run the tests it affects and name the move in the commit. No
wholesale re-resolve (`uv lock --upgrade`, bare `cargo update`) unless the operator asks.
Dependabot (ADR-0035) runs with `versioning-strategy: lockfile-only`, so it proposes moves
of undeclared packages only.

<a id="pin-reasons"></a>

### Holds

A hold is a pinned version that must not be bumped casually. It carries its reason: in Rust a
one-line entry in `[workspace.metadata.pse.pins]`, in Python a comment beside the pin in
`pyproject.toml` (instruction-only). Typical reasons are a named breakage, committed or
byte-stable generated output, a private or unstable API use, a parity or reference oracle,
and a fork or git revision. Family members are covered by `[workspace.metadata.pse.families]`
and move as a unit. Remove a hold when its reason lapses. A pinned dependency with no hold
can be bumped whenever the work calls for it.

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

Relaxing admission did not relax reproducibility. These remain
project invariants, checked when the corresponding command is run manually:

| Gate | What it protects |
|---|---|
| `just family-check` (`rust / family-check` when manually dispatched) | **One type universe.** Exactly one resolved `arrow`, `parquet`, `object_store`, `datafusion` and `pyo3`. Two majors make `downcast_ref` return `None` with no compile error — a silent failure that reads like a logic bug. This is the invariant `deny.toml`'s `multiple-versions` used to stand in for, and it states it far more precisely. |
| `tests/governance/tests/dependency_pins.rs` | **Exact pins.** Every registry requirement in `[workspace.dependencies]` is a single exact `=x.y.z` (path dependencies exempt); a git source names a full commit; family members are exact at the family version; a hold in `[workspace.metadata.pse.pins]` has a reason and names a declared, pinned dependency. Blueprint tables are not a second pin authority. |
| Committed `Cargo.lock` and `uv.lock`, every gate `--locked` | Reproducibility. Versions move only by a deliberate pin or lock edit; a run never resolves anew. |
| ADR for **majoring** one of the four pinned families | A family major changes the API surface the capability maps were extracted against. |
| ADR for adding or removing a **workspace** (`pse-*`) crate | A crate boundary is architecture, not a dependency. |
| `reuse lint` | SPDX headers on **our own** files. Unrelated to third-party licences. |
| import-linter contracts in `pyproject.toml` | Architecture, not hygiene: numpy/scipy at the array boundary, pyomo/pint in the adapter, `idaes` in the parity environment only (uv workspaces enforce a single `requires-python`, so an `idaes` import inside `pse` would drag the platform package back to the parity interpreter range). |
| The clean-room rule (CONTRIBUTING.md §8) | *Copying* code from a reference implementation. Unchanged, and a different question from *depending* on a library. |

## Running the check manually

The machinery is all still installed. It reports instead of gating.

```bash
just deps-report   # advisory: licences, advisories, bans, unused deps. Findings never fail it.
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
