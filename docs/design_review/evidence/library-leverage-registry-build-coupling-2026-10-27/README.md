# Registry build coupling and compile cost probe

This is evidence for the library-leverage comparative review of CBO-040 (G8 probe P1), dated
2026-10-27. CBO-040 is "registry products baked as about 87k lines of Rust plus JSONL plus IPC".
The source commit is `9cb8e4f4a`. All work ran in a detached scratch worktree of that commit
with its own `target/` directory. Nothing in the main checkout was built or edited.

## Question

1. **Bootstrap coupling.** `pse-codegen` re-exports `pse-schema` (`pub use pse_schema::*`).
   `pse-schema` compiles `mod generated;` unconditionally. Suppose a model type gains a field and
   its emitter is updated in the same change. Does `just codegen` still build the generator and
   regenerate? Or does the stale baked literal in
   `crates/pse-schema/src/generated/standard_registry.rs` break the generator's own build? If it
   breaks, what is the routine workaround?
2. **Compile cost.** What wall time and peak RSS does compiling `pse-schema` take? What share of
   an incremental rebuild that reaches registry consumers does it account for?

## Environment

- **Host.** AMD Ryzen 9 9950X3D (16 cores, 32 threads), 188 GiB RAM, Linux 7.0.0-38-generic.
- **Machine load.** The machine was shared with concurrent agent builds: the load average was
  about 42–47 during the measurements. At one point the root volume filled up completely, which
  is transient and recorded below.
- **Toolchain.** The pinned `nightly-2026-09-29` from `rust-toolchain.toml` (cargo 1.101.0-nightly
  3d7cf6e93, rustc c1070d693). `.cargo/config.toml` applies its settings: `fingerprint = "content"`,
  `checksum-freshness`, workspace feature unification, and mold through clang.
- **Profile.** `dev`: workspace members use `opt-level = 2`, `incremental = true` and
  `debug = "line-tables-only"`. Dependencies use `opt-level = 3`.
- **Features.** Default features for `pse-schema` and `pse-runtime`, under workspace feature
  unification. The `just codegen` runs used the recipe's own features: `xtask/force-validate`,
  then `full-codegen,force-validate`.
- **Commands.** All commands ran through `scripts/pse-env --` from the worktree root, with no
  `--resource-class`.
- **Compiler cache.**
  - Measured compiles ran with the compiler cache disabled (`RUSTC_WRAPPER=`; see
    `raw/measure.sh`). This puts every `rustc` under cargo, so `/usr/bin/time -v` reports the
    largest `rustc` child.
  - Warm-ups used the worktree's sccache route (`raw/wtenv.sh`).
  - Disabling the wrapper did not invalidate already-built units: no-op rebuilds took 0.26 s and
    0.40 s.
- **Change markers.** Content fingerprinting ignores `touch`, so each measured change edits the
  file content. A comment line inserted after the header shifts every line number. A same-length
  edit or an appended line shifts none.

## Part 1: bootstrap coupling

**Setup.** `raw/probe-field.diff` makes three changes:
- adds `pub probe_marker: u8` to `pse_schema::model::InvariantSpec`;
- sets it in the hand-written constructor `resolve_invariants` in `crates/pse-schema/src/builder.rs`;
- emits it from the `invariant()` emitter in `crates/pse-codegen/src/codegen/rust/registry.rs`.

**Run.** `just codegen` (`raw/p1-just-codegen.log`).

**Outcome.** The coupling breaks.
- The recipe's first step (`xtask codegen --only rust-contracts`) cannot build `xtask`.
  `pse-schema` fails with 258 × `error[E0063]: missing field 'probe_marker' in initializer of
  'invariant::InvariantSpec'`, all in `generated/standard_registry.rs`.
- The recipe exits with 101, and nothing is regenerated.
- *Tested*, by that command at `9cb8e4f4a`.

**No routine path exists.**
- The `just codegen` comment "contracts and the canonical schema first … so a registry change
  never needs a separate bootstrap" refers to the physical package loader. That loader compiles
  against the contracts.
- The rust-contracts step itself still compiles `pse-schema`, so the comment does not cover this
  case.
- `codegen-rust-contracts` and the other partial recipes build the same `xtask` → `pse-codegen` →
  `pse-schema` chain.
- Search scope: the `justfile` codegen recipes, `xtask/src/codegen.rs`,
  `crates/pse-schema/src/lib.rs`, `crates/pse-schema/src/generated/mod.rs` and a repository grep
  for "bootstrap". Only unrelated bootstraps turned up.

**The workaround found works, but it is manual.**
1. Overwrite `generated/standard_registry.rs` with a stub `hydrate()` that returns an error
   (`raw/bootstrap-stub.rs`, seven lines).
2. Rerun `just codegen`.

Outcome of the rerun (`raw/p1-just-codegen-stub.log`):
- `pse-schema` builds with seven dead-code warnings.
- The rust-contracts step regenerates the literal: 258 lines are added, one
  `probe_marker: 7u8,` per invariant (`raw/p1-regenerated-excerpt.diff`).
- The next step recompiles `pse-schema` against the regenerated file.
- The full `just codegen` finishes with exit 0 in 8 min 30 s. The time is dominated by the
  `full-codegen` native build (7 min 02 s).
- *Tested*.

The first attempt at this rerun (`raw/p1-just-codegen-stub-diskfull.log`) compiled
`pse-schema`, `pse-codegen` and `xtask`. Its link then failed with "Disk full?" while the
volume was transiently full. The rerun above succeeded unchanged once space returned.

**Why the stub works.** The generator assembles its registry from source through
`pse_schema::catalog::assemble()` (`xtask/src/codegen.rs`, `run`). It never calls the hydrating
`registry()`, so the stub's error path is never reached during generation.

**Why the stub is not routine.**
- It is a hand edit of a generated path, which prime directive 2 forbids outside a scratch
  bootstrap.
- It is not wired into any recipe.
- A `cfg`/RUSTFLAGS switch would rebuild the whole graph.
- A Cargo feature cannot gate the module, because ADR-0122 unifies features.

**History.** Over 2026-10-21 → 2026-10-26, 22 commits regenerated
`standard_registry.rs`. Five of them also changed `crates/pse-schema/src/model/` (these are the
changes that can hit the coupling). Five commits changed the registry emitter
(`git log -- crates/pse-schema/src/generated/standard_registry.rs`, cross-checked against
`model/`). How those authors got past the coupling is not recorded in Git.

## Part 2: compile cost

All runs build only `pse-schema` (`cargo build|check -p pse-schema --locked`), with
dependencies already built and the cache disabled. Times are wall clock from GNU time. RSS is
the largest child, which here is the `pse-schema` `rustc`.

| Case | Log | Wall | Peak RSS |
| --- | --- | --- | --- |
| Full (non-incremental, `CARGO_INCREMENTAL=0`) build, HEAD module | `mC2` | 25.4 s | 1.88 GB |
| Same, generated module replaced by the stub | `mE1`, `mE2` | 7.3–7.4 s | 1.43–1.45 GB |
| Incremental build, generated file line numbers shift (comment line inserted at top) | `mB1`, `mB4` | 21.3 s, 21.1 s | 1.18–1.22 GB |
| Incremental build, generated content changes, no line shift (appended / same-length edit) | `mA1`, `mB2`, `mB3` | 1.3–1.6 s | 0.76–0.77 GB |
| Incremental build after a hand edit that shifts lines in `model/invariant.rs`, HEAD module | `mH1`, `mH2` | 17.3 s, 17.3 s | 1.34–1.36 GB |
| Same hand edit, generated module stubbed | `mH4`, `mH5` | 4.9 s, 4.3 s | 0.91–0.94 GB |
| Full `cargo check`, HEAD module | `mD3` | 3.8 s | 0.86 GB |
| Full `cargo check`, stubbed module | `mE3` | 2.0 s | 0.44 GB |
| Incremental `cargo check`, line shift in generated file | `mD1` | 2.9 s | 0.77 GB |

*Measured*, with one or two samples per case under the load stated above. Treat differences
below a few seconds as noise.

**Share of a downstream rebuild.** These runs build `cargo build -p pse-runtime --lib --locked
--timings` after the dependency graph was warmed (`mF0` and the no-op `mF1`). The unit timings
are in `raw/mF2-cargo-timing.html` and `raw/mG1-cargo-timing.html`.

- **Line shift in the generated file only (`mF2`).**
  - Total wall time was 21.6 s.
  - `pse-schema` took 21.25 s from t = 0.32 s, so it is the critical path.
  - All six workspace dependents rebuilt incrementally in parallel behind its metadata and
    finished by about 17.7 s: `pse-relations` 3.4 s, `pse-engine` 1.3 s, `pse-operations` 1.2 s,
    `pse-rules` 0.4 s, `pse-admission` 0.5 s, `pse-runtime` 9.5 s.
  - In this case `pse-schema` codegen determines about 98% of the wall time.
- **Hand edit with a line shift in `model/invariant.rs` (`mG1`).**
  - Total wall time was 39.0 s.
  - `pse-schema` took 18.6 s. It overlaps with `pse-runtime`, which took 31.0 s and is on the
    critical path.
  - The `pse-schema` share of the wall time is lower here, but it is still about 13 s more than
    the stubbed crate needs (from `mH*`).

**Interpretation.**
- The type-check front end of the 87k-line module is cheap: about 2 s of `check`.
- The cost is LLVM codegen of the 2,399 outlined functions at `opt-level = 2`, plus the line
  tables:
  - It accounts for about 18 s, or about 72%, of a full `pse-schema` build.
  - It adds about 0.4 GB of peak RSS.
  - It is redone whenever line numbers in the generated file shift, which any real
    regeneration that adds or removes lines does.
- It is also mostly redone (about 13 s extra) after ordinary hand edits that shift lines in
  `pse-schema` model sources. The likely cause is that the outlined constructors inline model
  code into the same codegen units. That cause is inferred, not traced.
- Peak RSS stays under 2 GB, so no memory cap is at risk at the current size. The module
  header's "array-backed vec literals exhausted the compilation memory cap" concerned an earlier
  emission form that was not measured here.
- For scale, the warm-up `just codegen` steps rebuilt `pse-schema` → `pse-codegen` → `xtask`
  in 53 s (cached dependencies).

## Limits

- Only one model type (`InvariantSpec`) and one kind of change (an added field) were tried.
  Renaming a field or changing its type breaks the build the same way by construction, but
  that is not tested.
- The timings come from one to two samples on a heavily loaded shared machine, using the `dev`
  profile only. Neither the `release`/`dist` profiles nor the force-validate variants of
  `pse-schema` were timed separately.
- The downstream share was measured only for `pse-runtime --lib`. `pse-py`, the tests and the
  `xtask` full-codegen chain also depend on `pse-schema` and were not measured.
- The "real regeneration" cost is approximated by a line-shifting comment, not by a real
  declaration change. A real regeneration that also changes downstream generated code
  (relations, Python contracts) costs more downstream. The `pse-schema` share is then a lower
  bound on wall time, not an upper bound.
- `raw/p1-just-codegen-stub.log` includes a `docs/generated/capabilities/reference.md` change.
  It is a scratch-worktree artifact, because the worktree had no linked native skill, and is
  unrelated to the probe.
- The scratch paths in the raw logs are rewritten to `<scratch>`/`<worktree>`, and the home
  directory to `~`.

## Files

- `raw/probe-field.diff` — the probe change (model, constructor, emitter).
- `raw/p1-just-codegen.log` — the failing `just codegen` with the stale literal.
- `raw/bootstrap-stub.rs` — the workaround stub.
- `raw/p1-just-codegen-stub-diskfull.log` — the first stub attempt. It compiled, then the link
  failed on a transiently full volume.
- `raw/p1-just-codegen-stub.log` — the successful stub-bootstrapped regeneration.
- `raw/p1-regenerated-excerpt.diff` — the regenerated literal: stat and first hunks.
- `raw/m*.log` — the GNU `time -v` output of each measurement, named as in the table.
- `raw/mF2-cargo-timing.html`, `raw/mG1-cargo-timing.html` — the cargo `--timings` reports.
- `raw/measure.sh`, `raw/wtenv.sh` — the measurement and environment helpers, as run.
