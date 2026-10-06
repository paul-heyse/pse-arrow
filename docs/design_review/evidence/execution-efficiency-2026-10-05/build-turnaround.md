# Rust and native build turnaround

Supporting evidence for the [principal review](../../reviews/design_review_execution-efficiency-and-surrealdb_2026-10-05.md).
The bounded build investigator inspected source at `6498b013e579ec8039573200c92282eb8715b52e`
with the six concurrent numerical changes described by that review. The coordinator checked the
decisive provenance, manifest, native-setup and measurement paths. No builds, tests or cache
changes were performed. Candidate diagnoses below are **Interface-checked**, not measured latency
attribution. This document does not own finding disposition.

## Dependency closure rather than merely dependency features

[`pse-model`](../../../../crates/pse-model/Cargo.toml) describes an otherwise native-execution-free
semantic boundary, but depends on the generated
[`pse-workspace-hack`](../../../../crates/pse-workspace-hack/Cargo.toml). That package directly
depends on DataFusion execution/physical planning/optimizer components, Arrow, cloud-enabled
object_store, PostgreSQL support and host build dependencies. Its empty `build.rs` activates
the host dependency closure. A cold small semantic build must therefore compile machinery its
semantic contract does not consume.

The reason is explicit in [hakari configuration](../../../../.config/hakari.toml): prevent duplicate
feature variants when package selections change, including Cargo implementations that ignore
workspace feature unification. The configuration already separates host/target features and
excludes `pse-ids`, `pse-diagnostics` and the generated operations-query crate. Those exclusions
protect the buildinfo host dependency closure from a second heavy build. This is a material
strength and proof that membership can be selective.

**Proposed direction:** retain one compatible family/type universe and intentional feature
unification, while narrowing hack membership to roots that actually consume the heavy closure.
Compare focused semantic edits and alternating composite recipes. Removing the hack everywhere
is not established as the best answer. The historical build-infrastructure review's selected
heavy-unit reduction from seven to four was interface-checked, explicitly without build-time
measurement; it does not settle today's small-root versus composite-workload tradeoff.

## Complete provenance is also an executable dependency

[`pse-buildinfo/build.rs`](../../../../crates/pse-buildinfo/build.rs) recursively reads all files
under `crates/` and `vendor/`, emits directory change watches and embeds source/build identities
through [`lib.rs`](../../../../crates/pse-buildinfo/src/lib.rs). Tests and unrelated backend
implementations participate. Compiler, runtime and Python crates depend on this provider.
An unrelated source edit can therefore change the provider and propagate compilation through
consumers even when their relevant implementation did not change. `write_changed` correctly
avoids rewriting identical output bytes; it cannot avoid a real change to this whole-tree digest.

[`artifact_requests`](../../../../crates/pse-compiler/src/workspace.rs) also includes both global
identities in mathematical artifact keys. **Important limit:** the compiled-program cache in
[`math/artifacts.rs`](../../../../crates/pse-runtime/src/math/artifacts.rs) is process-local.
A new executable already starts without the old resident cache. Changed keys do not establish
an additional cross-build cache-eviction delay, and no durable artifact reuse loss was measured.

**Proposed direction:** separate complete build provenance from result-affecting implementation
identity and its compilation placement. Preserve exact dirty-source provenance at the outer
executable/publication boundary. Lower preparation keys consume their complete relevant
implementation, ABI, provider, configuration and toolchain dependencies, conservatively when
their closure cannot be narrowed soundly. Never substitute `git HEAD` for dirty-source identity.
Validate both an unrelated edit that should stay local and a relevant evaluator/native edit
that must invalidate the affected artifact.

## Eager native preparation precedes even a narrow build measurement

The `bench-builds` recipe in the [justfile](../../../../justfile) sources both
[`native-solver-env.sh`](../../../../scripts/native-solver-env.sh) and
[`native-math-env.sh`](../../../../scripts/native-math-env.sh) before calling the measurement
runner. Native math setup prepares KLU and interval/root isolation, then sources
[`native-pipeline-env.sh`](../../../../scripts/native-pipeline-env.sh), which prepares Uno and
PETSc unless their prefixes were already exported. Default build measurement ultimately
selects compiler/relations rather than the full native family.

[`native_pipeline_cache.prepare`](../../../../scripts/native_pipeline_cache.py) calls
`highs_archive` while constructing the Uno installation identity, before checking whether the
native installation is already complete. `highs_archive` invokes a memory-capped Cargo build
of `pse-uno-sys` with `highs-provider` to discover the actual archive. BLAS/HiGHS headers and
archives, source and patch identities are then read for the installation identity. The work
is real even when Cargo returns fresh; no elapsed cost is inferred here.

[`native_cache.valid`](../../../../scripts/native_cache.py) rehashes every recorded installed
header/library on a cache hit. `prepare` holds the per-identity exclusive lock during that
verification. Concurrent identical preparations serialize through it. Native build publication
itself is properly staged and checked; persistent installation identities prevent unnecessary
native recompilation. Explicit prefixes can bypass parts of setup, and dedicated pipeline
recipes demonstrate narrower selection.

**Proposed direction:** prepare the capability closure requested by the operation. Keep complete
setup for native-all/composite qualification; avoid it for nonnative or single-adapter work.
Retain verified installation handles over an appropriate immutable lifetime, with explicit
invalidation/revalidation for changed or uncontrolled installations. Do not replace corruption
detection with existence checks. Measure shell/setup plus Cargo separately as well as together.

## Unchanged code generation still rewrites generated files

[`xtask/src/codegen.rs`](../../../../xtask/src/codegen.rs), `write_tree`, uses `fs::write` for every
generated output without comparing existing bytes. The Ipopt binding and Python-stub generators
also write their outputs unconditionally. A no-op regeneration therefore changes source mtimes
and can cause Cargo to reconsider/rebuild generated consumers. Identical bytes do not change
the source identity, although the provenance script may rescan.

Scoped generation, owned output inventories, stale-file deletion and fresh byte comparison in
check mode are strengths. **Proposed correction:** preserve unchanged file bytes and mtimes,
write changed/new outputs, and continue deleting stale owned outputs. Use the generator as the
owner; do not hand-edit generated files. Verify a no-op run and a registry edit that should touch
only the logically affected generated outputs. The expected removal of writes is source-backed;
its build-time benefit is unmeasured.

## Measurement scope and historical evidence

[`build_measurements.py`](../../../../scripts/build_measurements.py) already supports source
snapshots, Cargo artifact JSON and timings, freshness/validation checks, wall/user/system time,
sampled descendant RSS and sccache deltas. It preserves dirty/untracked source and isolates its
build target. These are suitable foundations; no new measurement framework is needed.

Current scenarios cover fresh target, unchanged build, private compiler-body edit, appended
public compiler constant, same-path artifact recovery and optional second snapshot. `--native`
widens selected packages/features but retains the compiler Rust edit stimuli. Native C/C++ and
header edits, generator edits and production/no-validation builds are not represented by those
stimuli. Initial native setup runs before the Python runner and Cargo timer. Workflow samples
include nested commands but inherit already-prepared prefixes. Thus selected-target timing is
not fresh-shell complete-operation timing.

The second-worktree sample changes the absolute exported `CARGO_TARGET_DIR`; ordinary build
configuration deliberately removes that export. The sample therefore does not isolate the
ordinary cross-checkout cache behavior from this path/environment difference. Removing the
export alone is not proof of cache hits either.

The investigator recovered the historical plan with:

```sh
git show 8e9c3ce8264010e6cc2824161e7abb1b791d98c6:docs/plans/15-rust-build-performance.md
```

**Historical Measured, not reproduced:** the retained plan reports
`just bench-builds build/plan15/stable-uncached-verified --cache off --recovery` and
`just bench-builds build/plan15/stable-cached-system --cache on --cold-cache --recovery --second-worktree --execute`.
Conditions were stable 1.98.1, dev profile, 16 jobs and explicit Arrow force validation.

| Historical operation | Uncached seconds | Cached seconds |
|---|---:|---:|
| Cold selected target/cache | 163.48 | 194.21 |
| Unchanged, median of three | 0.177 | 0.164 |
| Private compiler edit, median of three | 0.608 | 0.659 |
| Public compiler edit, median of three | 0.615 | 0.669 |
| Same-path artifact recovery | 164.21 | 28.48 |

The changed second snapshot reports 196.34 seconds with zero Rust cache hits. Recovery reports
346 Rust and 108 C/assembler hits. Cold/recovery samples have no dispersion estimate. Named raw
Plan 15 reports were not found in the present build inventory; these numbers are attributed to
the historical committed plan. They predate the current nightly, hakari/workspace unification,
O2/O3 policy and product source. They do not establish present build latency or native edit cost.

Current Cargo documentation on [build-script change detection](https://github.com/rust-lang/cargo/blob/master/doc/book/src/reference/build-scripts.md)
and [sccache Rust limitations](https://github.com/mozilla/sccache/blob/main/docs/Rust.md), fetched
through Context7, support the mechanism interpretation. They do not qualify the installed tools
or supply present measurements. Profile, linker, generic-instantiation and generated-source-size
costs remain attribution questions rather than findings inferred from code volume.
