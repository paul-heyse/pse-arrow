# Rust build reuse

The toolchain is the dated nightly in `rust-toolchain.toml`, its only declaration
(ADR-0122). The source uses no nightly language feature. The nightly carries Cargo's
**workspace feature unification** (`-Z feature-unification`,
`[resolver] feature-unification = "workspace"`), which `.cargo/config.toml` turns on for
every Cargo run in a checkout, including rust-analyzer and the native correctness adapter.
Dependency features resolve over all workspace members whatever `-p` selects, so
`scripts/pse check`, `scripts/pse test` and direct native correctness commands share
eligible builds of Symbolica, DataFusion and Arrow in the same compiler context.
Opt-in validation and native backend features still select distinct compiled units.

The pinned nightly is the supported Cargo path. ADR-0175 retires the redundant
workspace-hack fallback and its generator routes after actual target/host/dev/native
graph checks. An installed `cargo-hakari` is therefore not an active build requirement;
do not regenerate a competing workspace-hack crate. Explicit correctness routes still verify Arrow leaf validation; ordinary
interactive builds keep that feature opt-in.

Each checkout keeps its intermediates in its own `target/`. Cargo's build directory is
not shared between checkouts: Cargo keys a workspace crate's unit by its
workspace-relative path with checkout-local fingerprint state. Content fingerprints do
not make a shared directory a supported isolation boundary (ADR-0122). Across checkouts,
the shared sccache can restore eligible identical compilations.

That reuse depends on `CARGO_TARGET_DIR` staying unset. sccache keys every rustc call on
its `CARGO_*` environment, so an absolute per-checkout target path would make every Rust
compilation miss the entries another checkout made. The native environment owner, applied by
`scripts/pse-env` and the CLI, therefore removes `CARGO_TARGET_DIR` when it
names the checkout's own `target/`, which is Cargo's default, and when it is empty or
points outside the checkout, as a value inherited from another checkout does: a
long-running agent or editor started under one checkout's direnv carries its absolute path
into every worktree. A directory inside the checkout, such as `target/measure-production`,
stays explicit, and `PSE_CARGO_TARGET_DIR` chooses another directory deliberately.
`scripts/pse-env -- sh -c 'echo ${CARGO_TARGET_DIR-unset}'` prints `unset` in an
ordinary checkout. A bare `cargo` outside a recipe or an activated direnv keeps whatever
its process inherited, possibly another checkout's target directory, so build through the
recipes.

Cargo uses content freshness on the pinned nightly: `.cargo/config.toml` enables both
`unstable.checksum-freshness` and `build.fingerprint = "content"`. Identical-byte Rust
touches and source restorations can remain fresh; changed source and included bytes must
recompile even when their length and mtime are unchanged. Declared build-script file
inputs retain Cargo's mtime-based `rerun-if-changed` behavior, and declared environment
changes still rerun the script. This policy applies to recipes, rust-analyzer and native
correctness commands. It preserves meaningful
compiler, feature, profile or validation differences.

Ordinary local `just` commands and direnv select the versioned managed `sccache`
declared in `.config/diagnostic-tools.toml`; an unrelated PATH binary does not replace it.
The current selection is 0.19.0. Install it with `just diagnostic-tools sccache` or
`just bootstrap-rust-tools`.
`just doctor` verifies the managed executable's bytes and version and reports the effective
selection without installing anything. Service admission checks the selected managed bytes
against the declaration while computing its existing executable-generation identity.
Explicit `RUSTC_WRAPPER` settings take precedence, including the empty string and custom
wrappers. `PSE_SCCACHE_BINARY` selects an alternative real ELF binary on the managed
wrapper route; doctor reports its actual version/path as an override. A missing optional
managed install leaves automatic builds unwrapped; explicit cache-on requests refuse.
CI retains its existing cache policy. Workspace
crates use O2, with incremental compilation in dev/test; release retains non-incremental
ThinLTO. Imported dependencies use O3 without incremental compilation, including vendored
path dependencies. Keep Cargo's artifacts for direct reuse; sccache provides a fallback
for eligible compilations. The Linux linker is mold, as declared in `.cargo/config.toml`.
The existing profiles serve the daily workflows; no separate optimized test profile is
needed.

Agent configurations leave `CARGO_TERM_COLOR` unset by default. Cargo chooses presentation
for the caller's terminal; explicit human settings remain supported. Refresh sessions after
changing defaults: existing processes retain their inherited environment. sccache hashes
almost all `CARGO_*` variables, so equivalent invocations should avoid accidental differences
without removing values that can affect compiled output.

```bash
scripts/pse check pse-compiler --lib
scripts/pse test pse-compiler --lib -E 'test(workspace_tests::)'
scripts/pse-env --cache off -- scripts/pse check pse-compiler --lib
```

`.config/build.toml` owns compiler-cache settings; `.cargo/config.toml` owns the default
32 Cargo jobs. Explicit supported selections retain their precedence. The ordinary local
unbounded mode shares all configured CPUs; native scientific allocations keep their own
ownership. The CLI and direct commands use the same native environment entry.

The environment preserves explicit encoded flags, or `RUSTFLAGS`, according to
Cargo's precedence. Without those overrides it retains the repository target flags,
including mold. Direct doctest, family-check and Python extension/stub producer recipes
carry the compile hint before allocation, so the configured ordinary-build mode applies.
A later nested hint cannot widen an existing functional allocation; explicit resource
selections still take precedence. Scientific execution keeps its own allocation owners.
No `target-cpu=native`, fast-math, panic-abort or global incremental
disable is introduced. Correctness recipes still request Arrow force validation.

The automatic local cache has a 100 GiB budget under
`${XDG_CACHE_HOME:-$HOME/.cache}/pse-arrow/sccache`, with a dedicated endpoint. It
does not stop the user's default cache server. An explicit `SCCACHE_DIR` keeps the
user's server configuration. Ordinary managed-cache statistics observe the admitted service.
Compiler-cache eligibility, successful artifact restoration and end-to-end feedback latency
are separate observations; a hit counter alone establishes no workflow speedup.

The shared environment requires sccache client-side compilation, keeping compiler
children inside the invoking operation's supervision. It disables the C/C++ direct
include shortcut so preprocessing observes newly present headers. The repository's
`.config/sccache.toml` supplies the default configuration; explicit configurations
remain supported. The presence of `SCCACHE_LOG` (even an empty value) or a configured
distributed scheduler disables upstream client-side compilation. The repository also
restricts `SCCACHE_ERROR_LOG` on this route. These settings cause ordinary builds to use
the compiler directly; direct managed compiler calls refuse before service admission. An explicit cache
experiment refuses those settings rather than measuring a different execution mode.
The local cache remains a trusted build input, not an artifact provenance authority.

Client-side cache publication sends a complete compressed entry over daemon IPC.
`.config/build.toml` declares a 256 MiB message ceiling; `SCCACHE_MAX_FRAME_LENGTH`
can select another positive decimal byte limit, up to the protocol's four-byte bound.
The managed service captures this selection in its generation identity and scrubbed
daemon environment, and nested recipes, language servers, installer contexts and
measurements retain it. The resident daemon has an 8 GiB supervised memory allowance.
Neither ceiling is a standing memory allocation; actual large transfers consume buffers.
The 100 GiB disk budget and default compression remain unchanged.
Raising the frame ceiling does not enlarge existing entries. Newly accepted large entries
incur actual IPC and storage work on publication; later restoration can avoid compilation.
Adoption follows the demonstrated publication/restoration repair, without an end-to-end
speedup claim. A performance campaign is needed only to make that quantitative claim or
settle an actual configuration tradeoff.

The upstream default 8 MiB frame can silently reject large client-side publications
and lose the associated statistics. A zero error/miss counter therefore does not prove
complete accounting. Inspect actual publication/recovery for important large artifacts;
`compile_requests` also remains zero in healthy 0.19 client-side mode. The increased
frame admits the tested large artifacts without weakening cache keys. Entries above
the selected frame still require a deliberate larger bound or an upstream transport fix.
Storage lookup durations exclude key preparation and output extraction: Rust hits still
run dep-info/output discovery and hash inputs, and proc macros, linking and incremental
workspace work remain outside ordinary eligible hits.

Released 0.19 hashes literal env-deps and relevant Cargo values, source/extern bytes and
cwd. Different OUT_DIR values and propagated extern changes can correctly miss across
checkouts. Same-path recovery and differently named targets with otherwise equal inputs
are separate cases. No Rust basedirs/remapping fork is selected; an upstream candidate
must preserve embedded literals and qualify actual heavy dependency/extern cases before
adoption. Keep each checkout's artifacts for the fastest no-op and incremental edit path.


## Native preparations

The native operation owner uses one persistent root,
`${PSE_NATIVE_CACHE:-${XDG_CACHE_HOME:-$HOME/.cache}/pse-arrow/native}`. Recipes request
only their required capabilities. Solver extraction is keyed by immutable image identity;
compiled providers include actual source, compiler, target, flags and build-tool inputs.
Private staging publishes immutable generations after complete byte/interface verification.
Construction and publication use separate coordination, so an admitted reader need not wait
for another generation's build. An operation retains its selected generation until actual
descendants drain; a new operation verifies it again. Missing or changed files cause readmission
or rebuilding. Unverifiable surviving owners retain conservative guards. Explicit `IPOPT_DIR`
remains supported with full boundary verification. Manual sourced helpers grant no operation
reuse; the math helper selects KLU/root isolation and the pipeline helper selects UNO/PETSc.

Bindgen selects resource headers from its selected Clang without changing the C/C++
compiler. When no user CMake toolchain file exists, `.config/native-cache.cmake`
supplies the supported C/C++ compiler launchers through `CMAKE_TOOLCHAIN_FILE`, which
the pinned `cmake` build dependency forwards. KLU also supplies launchers explicitly.
Fortran is unchanged. GMP/MPFR retains its upstream persistent cache mechanism.

## Selected timing investigations

Use `scripts/pse timings INVOCATION` to inspect retained native workflow phase durations.
For a compiler critical-path question, select the actual package and request Cargo's own
timing output through the correctness adapter:

```bash
scripts/pse-env -- scripts/cargo-bin/cargo-pse-correctness check \
  -p pse-compiler --lib --locked --timings
```

Inspect Cargo's timing output for overlapping compiler work; summed compilation durations
are not wall time. Record the exact source, feature, provider, profile and validation context,
the cold/warm premise and concurrent work. Repeat meaningful comparisons before claiming
a feedback improvement. Cache hits and unchanged-build freshness are narrower evidence than
an end-to-end edit, compilation and executed-test measurement.

Build measurements remain separate from product qualification. Cache reuse cannot substitute
for current source, native-byte and executed-test evidence. Native fixture and retained
invocation artifacts keep their original ownership; this guide grants no disposal authority.
