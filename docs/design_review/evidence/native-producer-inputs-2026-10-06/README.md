# Native producer inputs

This bounded source audit supports the selected compiler input work in
[Plan 28B](../../../plans/28b-selected-compilation-and-reuse.md), especially B3.
It does not grant production reuse eligibility. The producer tooling owner must
reconcile these contracts with fresh compiler-selected units for `pse-runtime
--lib`, `xtask --bin pse-worker`, and `pse-py --lib` in the actual dev/native
contexts. Cargo metadata and an older runtime capture are useful discovery
inputs, but do not establish those selected closures.

**Interface-checked:** exact local Rust source and the official executable source
versions below were inspected. **Observed:** bounded host version, path and driver
probes are retained in [host-probes.json](host-probes.json). No Cargo build,
scientific test, linked extension qualification, daemon restart or production
review grant was performed by this audit. The shared checkout was dirty and
concurrent throughout; source file identities are recorded in
[source-contracts.json](source-contracts.json). Its SHA256 observations are not the
producer owner's framed `reviewed_source` or executable closure bases.
Additional bounded loader resolution and clang configuration candidate
observations are in [loader-probes.json](loader-probes.json). They describe the
observer's environment; they must not be substituted for a selected build's
effective loader context.

## Selection and approval boundary

`xtask/src/producer_identity.rs` already separates the raw package review from
the selected executable dependency closure. A build script review must match
both, including its selected features and platform. Native approval binds the
package, profile, target, production target, requested features, ABI, concrete
file bytes, declared environment and absence facts in `native_review_basis`.
Configuration executors have a separate closure basis. A freshly observed hash
does not approve itself.

This matters for helpers: reviewing a build script's direct source does not
approve a selected helper's newly changed behavior. Conversely, the presence of
an I/O-capable helper package does not prove that the actual caller executes its
I/O branch. The narrow branches below can be approved when their actual selected
contexts and helper source bases match the review.

The actual selected follow-through is in
[selected-contexts.md](selected-contexts.md). Its
[source-grant recommendations](source-grant-recommendations.json) cover the 26
missing native/build owners using their raw reviewed source/helper bases and
specific branch premises. The final native declaration template now includes
link support files and exact helper environment/search effects. SUNDIALS'
generated public timestamp constants require the coordinator-approved fixed
epoch/timezone branch; the standalone CMake probes establish that narrow
setting, without claiming producer qualification.

## Rust helpers and build scripts

| Owner and inspected version | Source-backed behavior | Narrow grant or required tracking |
| --- | --- | --- |
| `stacker` 0.1.25 | Its Linux build branch constructs `cc::Build` and does not compile the Windows source. `cc` 1.6.0 construction initializes lazy state; `CargoOutput::new` reads `CC_ENABLE_DEBUG_OUTPUT`. | The Linux GNU branch can be reviewed without a native compiler closure. Bind the actual `cc` helper closure and that environment read; keep the Windows branch excluded. |
| `clang-sys` 1.9.1 | With `runtime`, its build-script `main` only checks the incompatible `static` feature. Library discovery is compiled into the runtime helper and happens when the caller loads libclang. | The selected runtime build script is a narrow effect-free grant. Do not transfer that grant to a bindgen caller which loads the library. |
| `cc` 1.6.0 | `env_tool` selects target-specific `CC`/`CXX`, wrappers and arguments. It recognizes several compiler wrappers and a custom wrapper. Archiver and ranlib selection have separate flags. `BuildEnv` snapshots the whole inherited environment and reinstalls that snapshot in child commands. | Bind the effective compiler, wrapper, archiver/ranlib and arguments for the selected caller. Targeted Cargo rerun hints cover neither executable bytes nor all environment consumed by the compiler and dynamic loader. Header dependency files do not close the compiler/tool/configuration input set. |
| `psm` 0.1.32 | The Linux x86_64 branch selects `src/arch/x86_64.s`, probes cc's compiler family and compiles assembler-with-cpp. The archive-writer branch is for a supplied wasm object, not this target. | Bind the selected cc compiler/assembler/archive closure and assembly source. Do not approve the reached branch as pure merely because the independent stacker script does no Linux compilation. |
| `pkg-config` 0.3.34 | Override precedence is target spelling, underscored target, host/target role, then the base variable. The command inherits ambient environment. The default command falls back to `pkgconf` only on executable invocation failure. | Bind the effective executable, requested arguments, override values and selected `.pc` dependency files. Include competing candidate presence where search can select a different file. An explicit selected prefix can narrow this materially. |
| `gmp-mpfr-sys` 1.7.1 | The default bundled branch uses raw `CC` (default `gcc`) and `CFLAGS`. It may copy archives and headers from its external cache before compiling. The cache key includes compiler spelling and a flags hash, but not compiler executable bytes. Cache candidates include compatible higher package patch directories. Fresh compilation runs `sh`, configure, `make`, and possibly native tests. | Bind the actual imported archives/headers and cache selection facts, or qualify an explicitly cache-disabled fresh build under a reviewed compiler/process closure. `GMP_MPFR_SYS_CACHE` empty or `_` disables its cache. Do not treat that setting as having retroactively rebuilt an already fresh Cargo unit. A selected system-library feature needs its own header/library and compile/run probe closure. |
| `pyo3-build-config`, `pyo3-ffi`, `pyo3` 0.29.2 | The build-config package itself has `build = false`; its helpers execute in callers. `make_interpreter_config` still tries interpreter discovery with Linux abi3 unless Python discovery is explicitly disabled. Stable ABI permits a fallback when an interpreter is unavailable; it does not itself prevent the attempt. | The actual interpreter branch needs Python executable, startup/import/sysconfig and shared library inputs. An intentionally supported Linux abi3 `PYO3_NO_PYTHON` context can narrow the branch if the target does not require a library directory. Bind actual config overrides and absence facts rather than assuming abi3 makes discovery inert. |
| `bindgen` 0.73.2 | Generation uses a runtime-loaded libclang and normally probes a clang executable for implicit include paths. Cargo callbacks emit header and environment rerun hints. Rustfmt may execute depending on the caller's formatter option. | Bind actual caller options first. Include libclang, compiler probes, resource/system/selected headers, and any selected formatter. `Formatter::None`/Prettyplease and disabled include detection are narrower branches only when actually selected. |
| `zstd-sys` 2.1.0+zstd.1.5.7 | Its bundled cc branch enumerates selected source directories, compiles C and optional assembly, then copies headers. `ZSTD_SYS_USE_PKG_CONFIG` being present selects pkg-config even if its value is empty. `bindgen` and `cmake` features introduce their corresponding executor branches. | Bind the actual selected features and environment presence. A no-bindgen/no-cmake branch needs cc and bundled native sources, not libclang or CMake merely because those exist in the lockfile. Directory enumeration means the source candidate set matters too. |
| `liblzma-sys` 0.4.9 | The static branch enumerates bundled C directories and compiles through cc. Its feature named `bindgen` selects committed Rust bindings; this build script does not invoke bindgen. The nonstatic branch can probe pkg-config and falls back to compilation. | Bind the selected static/features, `LZMA_API_STATIC` presence, bundled configuration/header inputs and cc closure. Do not infer a libclang executor from this feature's name. |
| `ring` 0.17.14 | On packaged Linux x86_64, `.git` absence and `RING_PREGENERATE_ASM` absence select supplied pregenerated assembly/headers and cc compilation. The `.git` branch instead generates assembly with Perl, and other platforms may use nasm. | Bind package `.git` absence explicitly; the raw package traversal ignores that directory. Review only the reached packaged branch, with native source/header and cc inputs. No Perl/nasm completeness is needed when their branches are excluded. |
| `blake3` 1.8.7 | The x86_64 branch probes compiler AVX512 support and may compile assembly. It disables cc's environment rerun emission, then emits only base `CC`/`CFLAGS` itself. | The actual compiler probes and targeted cc environment still need binding beyond emitted hints. Bind chosen native assembly/C inputs and compiler/archive closure; do not infer pure Rust from a Rust-facing API. |

The exact crate source is the primary basis for these branch distinctions.
Current Context7 queries supplied general PyO3, bindgen, cc and sccache routing;
cc's returned Windows examples do not establish the Linux contract. The
[pkg-config 0.3.34 reference](https://docs.rs/pkg-config/0.3.34/pkg_config/)
documents environment override precedence. The
[PyO3 build configuration guide](https://pyo3.rs/v0.29.2/building-and-distribution.html)
documents interpreter/configuration selection; actual startup effects are clear
from the inspected `Command` invocation rather than an isolation guarantee.

## Executables, configuration and loader inputs

**GNU compiler branch.** Disabling wrappers leaves cc's compiler selection
intact. With `CC`/`CXX` absent, the observed Linux defaults are GNU `cc`/`c++`,
not the Rust linker's configured clang. Read-only `gcc`/`g++ -###` observations
in [gcc-driver-probes.json](gcc-driver-probes.json) selected GNU 13's `cc1` and
`cc1plus` and an assembler command resolved through the effective search path.
GCC 13.3 source `gcc.cc` reads default specs files and constructs executable and
library prefixes; `incpath.cc` reads the language-specific include environment.
Bind the actual frontend/assembler/archive files, driver flags, specs selection,
native header inputs and relevant `GCC_EXEC_PREFIX`, `COMPILER_PATH`,
`LIBRARY_PATH`, `CPATH` and language include settings. The inspected upstream
source is a contract guide; the installed Ubuntu patched executable bytes and
observed built-in specs remain part of the actual identity. No compiler change
was proposed merely to avoid reviewing this reached branch.

**Clang 23.1.2.** The installed driver is dynamically linked: the host probe's
`readelf` output includes `libclang-cpp.so.23.1` and `libLLVM.so.23.1`. Driver bytes
alone do not cover these implementations. `Driver::loadConfigFiles` searches
configured user/system directories and the executable directory for default
configuration files, including target/driver names; explicit configs can include
more files. Nonempty `CLANG_NO_DEFAULT_CONFIG` disables implicit configs, but
does not eliminate explicit argument inputs. Include environment such as
`CPATH`, `C_INCLUDE_PATH` and `CPLUS_INCLUDE_PATH`, resource headers, target
sysroot/system headers and the actual selected tools/libraries still matters.
`COMPILER_PATH` influences executable lookup.

The observed `clang -###` link selected `ld.mold`, GNU CRT objects and GNU system
libraries. This is a host observation, not the final Rust link command. A selected
native closure must cover actual link inputs, including linker scripts and the
libraries they reference. Record resolved loader files and relevant loader
environment; `DT_NEEDED` names alone do not identify the loaded bytes.

**libclang discovery.** `clang-sys`'s runtime helper embeds its `build/common.rs`
and `build/dynamic.rs`. `LIBCLANG_PATH` set to a matching file narrows discovery
to that file. A directory permits versioned candidate changes; selection picks
the highest version and uses discovery order as a tie-breaker. Without the
override, it can execute llvm-config and search other directories, including
`LD_LIBRARY_PATH` and `LIBRARY_PATH`. Independently, valid absolute `CLANG_PATH`
selects the executable directly and bypasses executable search, but still runs
version and include-path probes. Exact file selectors plus actual shared-library
loader closure are preferable to recording one directory string as complete.

**mold 2.42.1.** The executable's installed version and commit match the inspected
upstream tag. Response files can recursively introduce arguments. Linker and
version scripts, archives, shared objects and library search choices are inputs.
`MOLD_REPRO` activates reproduction output; a reproduction archive is useful
diagnostic evidence, not a substitute for the selected producer's input contract.
The official [mold documentation](https://github.com/rui314/mold/blob/v2.42.1/docs/mold.md)
and exact tag sources are the appropriate version basis.

**sccache 0.17.0.** The client and daemon are distinct execution authorities.
`Config::load` reads environment plus `SCCACHE_CONF` or the default project config;
the daemon receives configuration at startup. A live daemon may therefore use
different configuration from today's file bytes. Config can select distributed
execution or other cache/toolchain behavior. Rust cache-key construction and
compiler shared-library hashing in sccache are useful mechanisms, but do not
establish this repository's independent source-input review completeness.
The exact [v0.17.0 source](https://github.com/mozilla/sccache/tree/v0.17.0) was
inspected; do not infer a running daemon's startup state from its current socket
or config path.

A supported wrapper-disabled capture is a concrete alternative. The repository's
`scripts/build_environment.py --cache off` sets both Rust wrappers empty and
clears an already present native compiler cache variable. Verify the effective
native launcher too: arbitrary `CC`, `CXX`, CMake toolchain/launcher settings or
custom wrappers remain separate inputs. This alternative must describe the
artifact actually captured and pass the normal freshness doctor. It neither
approves the existing daemon nor establishes another profile's provenance.

One wiring detail needs attention when selecting exact libclang:
`scripts/build_environment.py` currently sets `LIBCLANG_PATH` to the LLVM prefix
directory whenever LLVM is configured. This overwrites an earlier exact-file
value. Nested native setup also sources the build environment. Apply the exact
file selection after final setup, or make the supported route preserve that
explicit selection; verify the actual final child environment. A shell export
made before setup is insufficient evidence.

The producer tooling owner subsequently implemented preservation of an explicit
libclang override and selected a supported wrapper-disabled fresh profile. The
earlier source hash in this folder records the inspected pre-fix implementation;
the final selected capture must bind the changed setup source and final child
environment. This audit does not relabel earlier artifacts with the new profile.

## Python helper observations

[python-helper-probes.json](python-helper-probes.json) records the exact
interpreter discovery script extracted from PyO3 0.29.2 and its four build-flag
queries, executed with the selected checkout virtualenv interpreter. Both
`pyo3-ffi` and `pyo3` package working directories were observed: the command
inherits its build-script working directory, rather than necessarily starting
at the checkout root. The probe used `-B` to prevent bytecode writes while still
allowing existing bytecode reads; this observer difference is explicit.

All four query processes exited successfully. They observed CPython 3.14.7,
shared linking configuration, 64-bit pointers, no disabled GIL and four zero
build flags. User site was disabled by the virtualenv configuration. The helper
loaded stdlib modules and the `_virtualenv` startup hook; that hook's inspected
distutils/setuptools-specific finder branch was not reached by these queries.
The `.pth` candidates were `_virtualenv.pth`, `pse_arrow.pth` and
`a1_coverage.pth`. The last one imports coverage only when
`COVERAGE_PROCESS_START` or `COVERAGE_PROCESS_CONFIG` is present. Both were absent
in the observer environment; their absence must survive qualification.

The artifact records relevant environment presence and value digests, without
publishing raw secret values. It includes interpreter and loader bytes, module
source and cached bytecode identities, `.pth` and `pyvenv.cfg` bytes, current
search paths and concrete candidate absence observations. The absent stdlib zip
and startup-module alternatives are meaningful search premises. Future `.pth`
membership must be checked too, rather than rehashing only today's fixed file
list. These probes did not source native setup, which can invoke Cargo, and
therefore remain subject to reconciliation with its final child environment.
They do not establish that arbitrary startup plugins or new import branches are
safe. The version-specific [Python site documentation](https://docs.python.org/3.14/library/site.html)
describes startup `.pth` execution and customization; the
[command-line guide](https://docs.python.org/3.14/using/cmdline.html) supplies the
observer flag and environment contract. Context7's CPython main-branch response
also described Python 3.15 startup additions; those were excluded from this
3.14.7 assessment.

## Repository native callers

cc 1.6.0 does not automatically emit a GNU C/C++ header dependency file. A
possible tracking-only knob for a fresh supported context is targeted
`CFLAGS_x86_64_unknown_linux_gnu=-MD` and
`CXXFLAGS_x86_64_unknown_linux_gnu=-MD`, which request per-object dependency
files including system headers. Targeted names avoid changing GMP's raw base
`CFLAGS` cache selector. The tooling owner subsequently selected these knobs
for the fresh producer captures; [selected-contexts.md](selected-contexts.md)
records reached commands, dependency association and the snapshot boundary.
This audit did not invoke Cargo. `CC_ENABLE_DEBUG_OUTPUT=1` can expose program and
arguments: this cc version's `CommandLine` deliberately omits environment values
from that logging. Neither diagnostic command output nor dependency files
substitute for tracking compiler, assembler, archiver and loader inputs.

**Tested:** [header-shadow-probe.json](header-shadow-probe.json) records two
scratch-only GNU 13.3 preprocessing commands (`gcc -E -MD -MF`, two ordered
`-I` directories). Both exited successfully against a zero failure baseline.
Adding a header in the earlier directory changed the preprocessed constant
from 11 to 29 while the source and previously consumed fallback header bytes
remained unchanged. A dependency list therefore needs the search premises too:
guard earlier candidate absence or directory membership, or validate actual
resolution in a fresh matching compiler operation. Tracking only today's
consumed file bytes does not establish completeness for replay. This probe did
not compile or execute scientific code.

The actual selected features determine the branch:

- `pse-ipopt-sys/build.rs` with `link` absent is a compile-only shell. With it
  present, a selected `IPOPT_DIR` reads its exact `ipopt.pc`, compiles the C++
  bridge through cc, and links Ipopt. Bindings are already generated and the
  script does not invoke bindgen. The no-prefix branch additionally uses
  pkg-config discovery.
- `pse-backend-native/build.rs` always reads the workspace manifest for the
  POUNCE source receipt. Selected native features read the solver prefix's MKL
  `.pc`; the optional build-info file has a meaningful presence/absence branch.
  Root isolation additionally reads its completion receipt and `.pc`, invokes
  pkg-config, compiles the C++ transport and links static native inputs.
- `pse-uno-sys` and `pse-petsc-sys` with `link` absent return before prefix I/O.
  With it present, completion receipts, headers, native libraries and cc compiler
  inputs are required. A completion receipt describes preparation; its bytes
  alone do not identify all external library bytes currently loaded or linked.

`scripts/native-execution-env.sh` composes build, solver, math and optional local
environment setup. The audit did not read or publish `.envrc.local` values.
Producer declarations should select relevant secret-free environment facts;
license material should not be dumped into review evidence. Native-math setup
queries clang's resource directory and may introduce a CMake cache launcher.
Use the actual recipe context rather than assuming an interactive shell matches.

## Selected-context reconciliation

The tooling owner has now supplied actual runtime, worker and Python producer
captures. [selected-contexts.md](selected-contexts.md) records their scoped
source dispositions, raw review bases, concrete native inputs and linked
artifact observations. [native-namespace-candidates.json](native-namespace-candidates.json)
supplies bounded include, prefix and cache-candidate routes for native
declarations. [linked-loader-observer.json](linked-loader-observer.json)
records two successful loader-only observations under explicit source-derived
native paths; it does not claim the historical child environment.

The [Symbolica Git observer](symbolica-git-observer.json) also finds a concrete
exception to a simple absent-repository assumption: `$HOME/.cargo/.git` exists
as a directory but has no valid HEAD. Git reads system and user configuration
before rejecting ancestor repository candidates. All four read-only Git
queries return the expected status 128; this is fallback-path evidence,
not a failed product test or proof that Git executes no I/O.

## Remaining qualification work

The tooling owner should take final guarded compiler-selected captures, then bind only
the reached branches above to reviewed raw owner and helper closure bases. In
particular, settle the effective wrapper profile, compiler/archiver selections,
GMP cache versus fresh-build path, PyO3 discovery mode, libclang candidate set and
exact native ABI/library files. Unknown or changed branches should refuse reuse
while ordinary scientific execution and canonical persistence remain available.
This is a bounded qualification boundary, not a recommendation to refuse entire
packages or approve all contexts of a crate.

This audit searched the listed exact source files and relevant repository
callers. It did not inspect every GCC implementation input, all configure/make
scripts, arbitrary wrapper implementations, all Python startup plugins, all
possible platform branches or unselected lockfile packages. Those are not
claimed absent. An effective branch still consuming one of them needs its actual
closure before a complete grant.

## Selected helper and native link closure

The reached helper contracts for the actual diagnostic runtime DAG are in
[helper-source-contracts.json](helper-source-contracts.json). They cover selected
cc 1.6.0, CMake 0.1.58, glob 0.3.4, bindgen 0.68.1/0.69.5/0.72.1, and the
local SHA256, JSON and TOML helper paths. Source inventories identify the reviewed
closure; they do not grant every API or platform in those packages. The
proposed supported context has no identified irreducibly unsupported reached
branch. A selected feature, caller option or source change needs its own
matching review.

[Native prefix link inputs](native-prefix-link-inputs.json) adds exact external
archives, receipt/link-description files and finite native library basename
candidates across the selected external link directories. Generated executor
output archives remain products, not immutable external input namespaces.
The final declaration template incorporates these candidates. Its environment
values must be bound to the owning setup, and immutable fresh build snapshots
must match before the producer owner grants native completeness. No grant or
three-target eligibility result is asserted here.

[Guarded runtime native inputs](guarded-runtime-native-inputs.json) reconciles the
first immutable selected-executor snapshots: all 1,945 prerequisites from 407 depfiles
are associated with reviewed source, guarded external namespaces or generated outputs.
The three CMake compiler-detection depfiles name transient objects; their actual retained
detection executables are recorded separately from the 404 library product objects.

[Supported native grant disposition](supported-native-grant-disposition.json) records
the source-bound recommendation for the concrete one-setup native/configuration bases.
It accepts the supported external input closure, with no identified irreducibly unsupported
reached branch. The earlier duplicated-loader diagnostic remains ineligible. Fresh
source-frozen selected-executor captures and exact artifact association remain necessary
for producer qualification; the graph-only context probe supplies neither.
