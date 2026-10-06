# Selected native producer contexts

## Evidence boundary

The successful diagnostic captures select `pse-runtime --lib` (885 compiler
units), `xtask --bin pse-worker` (910), and `pse-py --lib` (909), all using the
fresh `producer` profile with `native-solvers,pse-relations/force-validate`.
They remain ineligible for persistent reuse. Selection and successful compilation
do not approve arbitrary build-script or proc-macro I/O.

[runtime-build-review-bases.json](runtime-build-review-bases.json) records the
26 currently missing runtime build-owner raw bases and their selected direct
build helpers. Those are observed candidate bases for a reasoned source review,
not approved hashes. [source-contracts.json](source-contracts.json) records the
individual source files inspected; its SHA256 values are independently useful
file identities, not the producer's framed owner/closure identities.

[runtime-selected-native-inputs.json](runtime-selected-native-inputs.json)
reconciles the selected native OUT_DIRs with **post-capture** run logs,
dependency files, object files, CMake selection files and current consumed bytes.
It finds 407 dependency files and 2,387 distinct consumed files or explicit
header hints. All observed prerequisites exist. 404 dependency targets resolve
to retained object/probe files; three CMake compiler-ID dependency targets have
no retained object. They are compiler discovery probes, not missing material
objects in the linked native libraries. Their source and dependency files are
retained. Header hints are advisory observations, not an assertion that the
callback reported every read.

The mutable run logs and dependency files were not snapshotted in the original
diagnostic build-evidence JSON. A subsequent matching Maturin build can overwrite
them. Their identities and observations here are therefore deliberately
post-capture evidence, not immutable proof of every historical execution branch.
A final guarded capture must associate and snapshot these files and relevant
child environment, and reject changes during reconciliation. This also applies
to generated headers, native object inputs and prefix namespace selection.

[linked-selected-artifacts.json](linked-selected-artifacts.json) associates the
actual Cargo messages with the current worker executable and Python shared
object. Both files match the independently reported capture SHA256 values. The
two `readelf -d` observations exit successfully against a zero failure baseline;
they establish ELF dynamic metadata, not successful scientific execution or
loader namespace completeness. Runtime's root is an rlib: its Cargo native link
directives do not prove the final worker/shared-object link resolution.

## Source-backed dispositions

These are proposed narrow review dispositions for the tooling owner. Each needs
its raw owner and selected executable-helper closure bound to the relevant
supported context. Native library, tool, environment and search declarations
still need the independently reviewed native basis.

| Selected owner | Reached effects and applicable narrow boundary |
|---|---|
| `bindgen` 0.68.1, 0.69.5, 0.72.1 own build scripts | Write `OUT_DIR/host-target.txt` from `TARGET` and emit environment hints. Their selected build-script helper dependencies are empty. These own scripts need no libclang grant; native effects occur when another build owner invokes the bindgen library. |
| `clang-sys` 1.9.1 | Selected `runtime` build-script branch performs no native discovery/linking. Runtime loading is owned by reached bindgen callers. |
| POUNCE algorithm/restoration/rs 0.12.0 | Read `DEP_COINHSL_RPATH`, return if absent/empty, otherwise emit link arguments; selected features exclude MA57. Bind the actual metadata premise and target OS. No selected helper dependencies. |
| POUNCE solve-report 0.12.0 | Export Cargo `TARGET` as report metadata. No filesystem read or subprocess; no selected helpers. |
| `prettyplease` 0.2.37 own build script | Read Cargo package version and emit check-cfg/version metadata. No filesystem read, subprocess or selected helpers. |
| `pardiso-wrapper` 0.1.3 | Selected MKL branch checks target architecture and `DOCS_RS`, emits libm link metadata on Linux. No file discovery or helper dependencies. |
| `private-gemm-x86` 0.1.20 | Generate assembly and Rust text into its own `OUT_DIR`; external I/O search finds only `OUT_DIR` and those writes. The selected `defer` 0.2.1 and `interpol` 0.2.1 executable helper/macro closure remains an independent premise. |
| `rustix` 0.38.44 | Native Rust feature probes invoke `RUSTC`, with `RUSTC_WRAPPER` only if nonempty, and use target plus encoded Rust flags. Metadata-only stdin probes still require the actual compiler/loader closure. |
| `symbolica` 3.0.1 | Execute `git describe` and three `git rev-parse --git-path` queries. Packaged source still executes Git. Current log reports fallback package version; no pure-I/O grant follows from that fallback. Bind Git executable/loader, environment and repository/config discovery premises. |
| `gmp-mpfr-sys` 1.7.1 | Current selected OUT_DIR log shows a cache-hit copy of six archive/header files from the compatible 1.7.1 cache candidate followed by Rust header parsing. That branch needs candidate selection/content and cache environment closure, not GCC/configure/make. A fresh-build branch has separate subprocess obligations. |
| `blake3` 1.8.7, `psm` 0.1.32, `liblzma-sys` 0.4.9, `zstd-sys` 2.1.0 | Actual native compilation through GNU cc plus archiver. Per-object dependency files exist; include-search premises and actual tool closure remain required. The liblzma `bindgen` feature name does not itself imply a bindgen executor. |
| `suitesparse_sys` 0.1.4 | Selected external-prefix branch; `build_vendor` absent. Explicit SuiteSparse include/library directories bypass global fallback discovery. Bindgen 0.69.5 parses headers; macro callbacks do not enumerate include reads. Track exact selected header/include namespace and five static libraries. |
| `sundials-sys` 0.6.2 | Selected vendor CMake build, static IDAS/KINSOL/KLU modules, followed by bindgen 0.68.1. Track CMake/GNU compiler/build/archive helpers, selected KLU prefix and generated/consumed headers. |
| `highs-sys` 1.15.0 | Selected bundled CMake Release/static build plus bindgen 0.72.1. `discover`/`ninja`/`libz` absent; packaged `HiGHS/highs` presence avoids Git submodule invocation. Guard that presence and the reached compiler/CMake/header effects. |
| `scip-sys` 0.1.28 | Selected external `SCIPOPTDIR` bindgen branch; `bundled` and `from-source` absent. Current log selects the solver prefix before Conda/system fallback. Bind prefix glob/library/header contents and actual libclang/clang/Rustfmt effects. |
| Four repository native bridge build owners | Selected linked/native feature branches compile transport C++ and consume exact protected native prefix headers/libraries/receipts. Per-object `.d` records exist. Prefix receipt bytes alone do not identify linked library contents. |

The reached bindgen callers use `Builder::default()` and no formatter override.
The exact 0.68/0.69/0.72 sources use the default Rustfmt subprocess during
`write_to_file`. In 0.68/0.69 the selected `which-rustfmt` feature resolves the
executable; 0.72 delegates `rustfmt` lookup to process spawning. `RUSTFMT` overrides
both. Track the effective executable, possible rustup proxy/toolchain selection,
loader files and configuration search. The official
[bindgen formatting contract](https://github.com/rust-lang/rust-bindgen/blob/main/book/src/code-formatting.md)
supports this general behavior; exact release sources establish the versioned
lookup differences.

## Python selection needs immutable confirmation

Original capture Cargo messages establish `Py_3_11` and `Py_LIMITED_API`. Those
cfgs do not distinguish interpreter discovery from an explicit config file under
abi3. After the matching Maturin build began, the mutable pyo3-ffi log selected
`target/maturin/pyo3-config-x86_64-unknown-linux-gnu-3.14-abi3.txt`. That current
observer path contains CPython 3.11/abi3 metadata and bypasses interpreter
discovery. It must not be asserted as the historical diagnostic capture branch.

If the final supported capture actually selects `PYO3_CONFIG_FILE`, the exact
0.29.2 `resolve_build_config` returns from that file branch before host/cross
discovery. Bind its file bytes and applied build features/environment plus
compiler-version probes; Python startup is then not a reached build input.
If it actually selects `PYO3_PYTHON`, the existing exact interpreter/startup
probe supplies a candidate closure to reconcile against final child environment.
Neither route requires changing the public Python ABI.

The tooling owner now selects the explicit config-file branch for the final
guarded capture, together with an absolute pinned `RUSTFMT` executable. This is
a proposed supported execution context until the actual immutable capture
confirms it. The config file is a pre-existing caller input outside this owner's
OUT_DIR, even when its path is under `target/maturin`; its bytes must be bound.
Generated CMake include roots have the different output boundary described below.
The Python startup observations remain evidence for the unselected discovery
alternative, rather than unnecessary requirements for the config-file branch.

## Remaining qualification actions

[source-grant-recommendations.json](source-grant-recommendations.json) supplies
each of the 26 selected missing build owners with its reviewed branch, raw source
basis, selected helper bases and concrete premises for a narrow grant. It is a
manual review recommendation, not a production approval. The tooling owner owns
the declarations and final grants.

[link-support-inputs.json](link-support-inputs.json) extends the final native
template with CRT objects, small support archives, linker scripts and their
transitive files. Three read-only `clang -### -fuse-ld=mold` observers (default,
shared and PIE) exit successfully against the zero failure baseline. They perform
no link and do not establish historical Rust linker arguments. Ordered candidate
paths bind the narrow driver route without hashing all of `/usr/lib`.

[tool-environment-contracts.json](tool-environment-contracts.json) adds exact
Clang, GCC and mold source selectors beyond Cargo's environment hints. In
particular, `CCC_OVERRIDE_OPTIONS` can inject command options independently of
`CLANG_NO_DEFAULT_CONFIG`; the supported branch must bind its absence. The
bounded observed native input scan finds no literal `__DATE__`, `__TIME__` or
`__TIMESTAMP__` tokens. This applies only to the currently observed consumed
source/header files and does not grant future untracked content.

[cmake-generation-contracts.json](cmake-generation-contracts.json) closes a
different clock path: SUNDIALS generates `SUN_JOB_*` strings with CMake
`TIMESTAMP`, and bindgen exports them. The supported branch uses the coordinator's
explicit `SOURCE_DATE_EPOCH=0,TZ=UTC`. **Tested:** four standalone CMake 3.28
script probes exit zero against the zero failure baseline; repeats agree, and
changing TZ changes the local timestamp. No Cargo or native library build runs
in this probe. The four CMake include roots under OUT_DIR are generated outputs,
excluded from immutable external namespace inputs; a fresh guarded executor
build has no prior CMake cache authority.

[external-selector-contracts.json](external-selector-contracts.json) records
the exact reached pkgconf selectors, Clang distribution detection files, bash
startup/function premises for HiGHS' `ldd --version` helper and the bounded
header alias observation. These contracts supply specific input choices to the
tooling owner; they do not enlarge the audit into unrelated executor branches.

[helper-lookup-candidates.json](helper-lookup-candidates.json) supplies concrete
Symbolica Git ancestor and configuration paths, excluding unrelated workspace
HEAD state. Both current primary Git configs have no include directives. It also
records four successful pinned Rustfmt probes on scratch stdin from the reached
bindgen callers' package directories, with 24 configuration candidates absent.
No repository formatting occurred; the baseline is zero failures. These current
lookup observations must agree with the final supported native environment.

1. Snapshot selected native run logs, object `.d` files, generated headers,
   CMake selections and reached child environment in the final guarded capture;
   keep later Maturin build mutation distinct.
2. Bind actual GNU driver/cc1/cc1plus/assembler/archiver tools and their loader
   inputs, selected CMake/make tools, Rust compiler and Rustfmt/rustup selection.
3. Bind consumed header bytes **and** earlier candidate/search namespace
   premises. The header-shadow probe demonstrates why fixed `.d` files alone
   are incomplete. Directory symlink and membership changes must be represented.
4. Bind selected native prefix library/header bytes and ABI, including static
   archives and final ELF dependencies. Hash exact files, not only receipts.
5. Reconcile libclang's exact-file selector, Clang configuration-disabling
   premise, actual dynamic loader search/cache inputs, and actual PyO3 branch.
6. Record explicit grants from the reviewed source and native bases, then run
   the production qualification controls. No grant is made by this evidence.
