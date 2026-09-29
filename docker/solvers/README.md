<!--
SPDX-License-Identifier: MIT OR Apache-2.0
Copyright (c) 2026 Paul Heyse
-->

# `pse-solvers` — the native solver prefix `pse-arrow` links against

`pse-arrow` needs Ipopt **3.14.x** with a selectable high-performance linear
solver and SCIP **10.0.2** linked against that same Ipopt; Ubuntu 24.04 ships
Ipopt 3.11.9 and no SCIP. This directory is the single recipe that produces the
native prefix `/opt/pse-solvers` for CI, for the devcontainer, and — through
the native cache — for local builds. Its composition is ADR-0108 (one image,
one BLAS/LAPACK, one OpenMP runtime, no HSL) and ADR-0105 (SCIP's components).

It builds, from pinned and checksummed sources:

| | version | notes |
|---|---|---|
| oneMKL | 2026.1.0 (Intel apt packages `2026.1.0-236`) | the only BLAS/LAPACK; LP64 interface, GNU OpenMP threading (`mkl-dynamic-lp64-gomp`); Pardiso; CNR branch `COMPATIBLE` |
| METIS | 5.1.0 | shared; 32-bit `idx_t`/`real_t`; linked by MUMPS and SPRAL |
| MUMPS (sequential) | 5.9.1 (ThirdParty-Mumps 3.0.14) | shared, **with METIS** |
| SPRAL (SSIDS) | 2025.09.18 | shared; OpenMP, METIS, hwloc; CPU only |
| AMPL Solver Library | `solvers-20241108` (ThirdParty-ASL 2.1.0) | static, `-fPIC`; only Ipopt's `ipopt` driver uses it |
| Ipopt | 3.14.20 | shared; `linear_solver` ∈ {`mumps`, `spral`, `pardisomkl`}; `--without-hsl --disable-linear-solver-loader`; sIpopt; `--disable-java` |
| SCIP Optimization Suite | 10.0.2: SCIP 10.0.2 (API 156), SoPlex 8.0.2, PaPILO 3.0.0 | shared; `IPOPT=ON` against the Ipopt above; `THREADSAFE`, `TPI=tny`, exact mode (GMP, MPFR, Boost); PaPILO without TBB; symmetry `snauty` |
| GMP, MPFR, Boost, hwloc | Ubuntu 24.04 packages | GMP 6.3.0, MPFR 4.2.1, Boost 1.83 headers, hwloc 2.10 |
| HSL | — | excluded (ADR-0108, register R-34); see [HSL](#hsl) |

The prefix holds `lib/libipopt.so`, `lib/libsipopt.so`, `lib/libcoinmumps.so`,
`lib/libspral.so`, `lib/libmetis.so`, `lib/libscip.so`, `lib/libsoplexshared.so`,
the oneMKL libraries `lib/libmkl_{intel_lp64,gnu_thread,core}.so*` plus the
CPU-dispatch kernels `lib/libmkl_{def,mc3,avx2,avx512,avx10,vml_*}.so.3`,
`lib/libcoinasl.a`, `bin/{ipopt,scip,spral_ssids,…}`, headers under
`include/{coin-or,scip,objscip,lpi,lpiexact,soplex,papilo,tpi,…}` and `include/{mkl*.h,
spral*.h,metis.h}`, `lib/pkgconfig/{ipopt,ipoptamplinterface,coinmumps,coinasl,
mkl-dynamic-lp64-gomp}.pc` (all relocatable), SCIP's CMake package under
`lib/cmake/`, Intel's licence texts under `share/licenses/onemkl/`, and a
`share/pse-solvers/build-info.txt` manifest recording versions, the MKL link
line, the pinned CNR branch and every source checksum.

Why these choices is argued in the comment header of [`build.sh`](./build.sh)
and recorded in ADR-0108 and ADR-0105.

### Process environment

The `solvers` stage (and so `ci` and `dev`) sets, beside `IPOPT_DIR`,
`SCIPOPTDIR` and `MKLROOT` (all `/opt/pse-solvers`), `PKG_CONFIG_PATH`,
`LD_LIBRARY_PATH` and `PATH`:

| Variable | Value | Why |
|---|---|---|
| `MKL_CBWR` | `COMPATIBLE` | ADR-0108 item 14. `COMPATIBLE` is the only conditional-numerical-reproducibility branch oneMKL supports on non-Intel CPUs; any other value is silently replaced by `AUTO` there. Admission checks `mkl_cbwr_get(MKL_CBWR_BRANCH)` |
| `MKL_DYNAMIC` | `FALSE` | oneMKL never lowers a requested thread count |
| `OMP_NUM_THREADS`, `MKL_NUM_THREADS` | `1` | ambient serial; a solve that is admitted threads raises them for its duration (ADR-0108 item 12). `OMP_NUM_THREADS` also makes any other OpenMP library in the container (a Python wheel's OpenBLAS, for instance) serial by default |
| `OMP_CANCELLATION` | `TRUE` | SPRAL SSIDS refuses to factor without it (SSIDS error -53) |
| `OMP_PROC_BIND` | `TRUE` | SPRAL warns (SSIDS warning +50) when threads are unbound |
| `OMP_PLACES` | `sockets` | with `OMP_PROC_BIND=TRUE` and no places, libgomp pins every process's initial thread — and every thread it creates afterwards — to the first CPU of its affinity mask, so concurrent processes share one CPU. Socket places keep binding enabled without that collapse on a single-socket host (on a multi-socket host each process is confined to its first socket); a worker that owns specific cores should set its own `OMP_PLACES` |
| `HWLOC_COMPONENTS` | `-linuxio,-pci,-opencl,-cuda,-nvml,-rsmi,-levelzero,-gl` | SPRAL asks hwloc for PCI and OS devices to look for GPUs, even when built without CUDA. That discovery costs seconds per Ipopt solve — two HS071 solves took 8 s with SPRAL and 0.1 s with the filter, both in the image and on an Ubuntu host with `libhwloc-plugins`. CPU, cache and NUMA discovery are unaffected |

These are process-level: libgomp, oneMKL and hwloc read them when they
initialize, so a host process that loads the extracted prefix must have them in
its environment when it starts.
`/etc/ld.so.conf.d/pse-solvers.conf` also puts `/opt/pse-solvers/lib` in the
loader cache.

## Stages

| Target | Contains | Used by |
|---|---|---|
| `builder` | build-essential, gfortran, cmake, meson/ninja, hwloc/GMP/MPFR/Boost/zlib headers, the sources; runs `build.sh` **and `test/run.sh`** | nothing ships from here |
| `solvers` | the prefix + `libgfortran5`, `libgomp1`, `libhwloc15`, `libgmp10`, `libmpfr6`; the [process environment](#process-environment) | base of `ci` |
| `ci` | `solvers` + build-essential, git, curl, pkg-config, `libclang-dev` (bindgen), python3, `uv` 0.12.13 with managed CPython 3.11/3.12/3.13 | every CI job that compiles or solves |
| `dev` | `ci` + rustup at the pinned toolchain, `cargo-binstall`, `just`, `cargo-nextest`; `UV_PYTHON_DOWNLOADS=automatic` | `.devcontainer`, local shells |

The three CPythons baked into `ci` are the **image parity matrix** (3.11-3.13).
The current IDAES 2.13.0 preflight uses Python 3.13; the platform interpreter is separate: `.python-version`
asks for 3.14.7, which nothing in the image provides. CI is hermetic
(`UV_PYTHON_DOWNLOADS=never`), so a CI job must never need 3.14; the `dev` stage
flips that one variable to `automatic` so `just bootstrap` inside the
devcontainer can fetch 3.14.7 on first use. That single variable is the only
difference between `ci` and `dev` other than Rust.

`test/run.sh` runs inside `builder`, so a broken prefix can never be tagged:
1. `pkg-config` sees Ipopt 3.14.20, `coinmumps`, `coinasl` and
   `mkl-dynamic-lp64-gomp`; the Ipopt, SPRAL, METIS, oneMKL and SCIP headers and
   the manifest exist, and the manifest's CNR branch equals `MKL_CBWR`;
2. `test/mkl_cbwr.c` requires the `COMPATIBLE` branch to be in force, once from
   the environment and once through `mkl_cbwr_set`, then runs DGEMM and a
   DSYTRF/DSYTRS solve through the dispatch kernels;
3. `test/ipopt_linear_solvers.c` requires `IpoptGetAvailableLinearSolvers` to
   report `mumps`, `spral` and `pardisomkl`, and no HSL routine, no
   pardiso-project Pardiso, no WSMP and nothing run-time loaded;
4. Ipopt's own `hs071_c.c` solves HS071 (twice, the second warm-started) with
   each of the three linear solvers, selected through an `ipopt.opt`, within
   3 s (a lost `HWLOC_COMPONENTS` makes SPRAL take ~8 s), and Ipopt's banner
   names the solver used; `linear_solver=ma57` is refused, and SPRAL without
   `OMP_CANCELLATION` fails (control);
5. `ipopt tiny.nl -AMPL` solves a hand-written `.nl` file and writes a `.sol`;
6. `test/scip_minlp.c` requires the SCIP library to match its headers (10.0.2,
   API 156), SoPlex, PaPILO, GMP, MPFR and Ipopt 3.14.20 among SCIP's external
   codes, the nested Ipopt's `linear_solver` values to be exactly the image's
   three and `nlpi/ipopt/hsllib` to be absent, and solves a small nonconvex
   MINLP (bilinear constraint, integer variable) to its known global optimum
   `-2·sqrt(2)` with `misc/catchctrlc = FALSE`; `scip -v` reports 10.0.2;
7. `readelf`/`ldd` show libipopt → MUMPS, SPRAL and the oneMKL interface
   layer; MUMPS and SPRAL → METIS and the whole oneMKL link line (interface,
   GNU threading, core, libgomp), which therefore loads with libipopt (libtool
   drops `--no-as-needed` ordering for libipopt itself); libscip → Ipopt, GMP,
   MPFR; and, across every library and
   executable in the prefix, no netlib BLAS/LAPACK, OpenBLAS, second OpenMP
   runtime, TBB or HSL, every oneMKL library resolved from the prefix, and
   nothing unresolved.

## Tags

`ghcr.io/paul-heyse/pse-solvers`:

| Tag | Meaning |
|---|---|
| `ipopt3.14.20-mumps5.9.1-metis5.1.0-spral2025.09.18-onemkl2026.1.0-scip10.0.2-r1` | the `solvers` stage, named by content (`RECIPE_TAG` in `solvers-image.yml`); `-rN` bumps when the recipe changes without a version change. The previous recipe was `ipopt3.14.20-mumps5.9.1-asl20241108-r1` |
| `ci-<tree-hash>` | the `ci` stage for that recipe |
| `dev-<tree-hash>` | the `dev` stage for that recipe |
| `dev-latest` | moving alias for the devcontainer, updated by `solvers-image.yml` |

`<tree-hash>` is `git rev-parse HEAD:docker/solvers` (the git tree object of this
directory — it changes exactly when a file here changes), truncated to 12 hex
characters. Outside a clean checkout, the equivalent fallback is
`sha256(docker/solvers/**)[:12]` over the sorted file list.

`ipopt -v` inside the image prints `Ipopt 3.14.20 (x86_64-pc-linux-gnu),
ASL(20241111)`: `solvers-20241108` is the ASL *release tarball* name,
`20241111` is the ASL's internal `ASLdate`. Both refer to the same source.

## How CI uses it

`solvers-image.yml` builds on every PR touching `docker/solvers/**`; on `main`
it builds and pushes the three tags above and opens a PR updating the
`SOLVER_IMAGE` env in the workflows. Jobs pin the image **by digest**, never by
tag:

```yaml
env:
  SOLVER_IMAGE: ghcr.io/paul-heyse/pse-solvers:ci-<tree-hash>@sha256:<digest>
jobs:
  test:
    container: ${{ env.SOLVER_IMAGE }}
```

`rust / clippy`, `rust / test`, `rust / codegen-diff` (needs the Ipopt headers
and libclang for bindgen) and `python / parity` (needs managed CPython plus a
solver on `PATH`) all run in the `ci` stage. A weekly
`solvers-image-rebuild-check` rebuilds with `--no-cache` and compares the
installed library checksums against `share/pse-solvers/build-info.txt` to catch
recipe drift. The recipe is reproducible to that standard: a `--no-cache`
rebuild of the `solvers` stage gave byte-identical `lib*.so*` (2026-09-27),
because the scratch directory is fixed (CMake and meson embed absolute source
paths) and the oneMKL libraries are copied unmodified.

## Building locally

```bash
just solver-image                 # confirm-gated; --target ci  -> pse-solvers:ci-local
just solver-image dev             #                --target dev -> pse-solvers:dev-local
```

or directly:

```bash
docker build --target ci  -t pse-solvers:ci-local  -f docker/solvers/Dockerfile docker/solvers
docker build --target dev -t pse-solvers:dev-local -f docker/solvers/Dockerfile docker/solvers
```

The build context is `docker/solvers/`, not the repository root. On a 32-thread
host (AMD Ryzen 9 9950X3D) `build.sh` takes ~190 s, most of it the SCIP
Optimization Suite; the acceptance tests take ~3 s. The ~270 MB of source and
oneMKL archives sit in a BuildKit cache mount, so a re-run after a failed step
re-fetches nothing. The `solvers` stage is ~0.7 GB (the prefix is ~0.6 GB, of
which oneMKL's libraries are ~0.5 GB).

## Local images

`scripts/native_cache.py solver` (sourced through `scripts/native-solver-env.sh`
by every native recipe) extracts `/opt/pse-solvers` from the **pinned** dev
image in `.github/setup/solver-images.json` into the native cache, and
`scripts/native-solver-runner.sh` runs native test binaries inside that image.
To use a locally built recipe instead — before it is published, or while
changing it — name the image by content in `PSE_SOLVER_IMAGE`:

```bash
just solver-image dev
export PSE_SOLVER_IMAGE="$(docker image inspect --format '{{.Id}}' pse-solvers:dev-local)"
just check-solver-contracts        # extracts the local prefix and compiles against it
```

The override must be immutable — a local image ID (`sha256:<64 hex>`) or a
digest reference (`name@sha256:<64 hex>`); a tag such as `pse-solvers:dev-local`
is refused, because the native cache is keyed by the image identity and a tag
can move. `scripts/solver-images.py runtime dev` prints the image in effect.
Both the extraction and the runner use it; without the variable, both use the
pin, unchanged. The extracted prefix lands in
`$PSE_NATIVE_CACHE/solver/<sha256 of the identity>/` and
`native-solver-env.sh` exports it as `IPOPT_DIR` and `SCIPOPTDIR`. A process
that loads the extracted libraries directly (outside the image) needs the
[process environment](#process-environment) and `LD_LIBRARY_PATH=<prefix>/lib`
(the libraries' RUNPATH is `/opt/pse-solvers/lib`).
The same variable is read by
`just solver-rebuild-check`, which also accept a tag, and by
`just bootstrap-solvers`, which pulls it and so needs a registry reference.

## Running a shell

```bash
docker run --rm -it -v "$PWD":/work -w /work \
  ghcr.io/paul-heyse/pse-solvers:ci-<tree-hash> bash
```

Use the `dev-` tag when you want cargo inside the container; the `ci-` stage has
no Rust toolchain by design (CI installs its own through
`.github/actions/setup-rust`). With the `dev` image:

```bash
docker run --rm -it -v "$PWD":/work -w /work \
  ghcr.io/paul-heyse/pse-solvers:dev-latest \
  cargo check -p pse-ipopt-sys --locked
```

`IPOPT_DIR=/opt/pse-solvers` is already exported, so `pse-ipopt-sys/build.rs`
finds Ipopt without `pkg-config` fallbacks. Bind-mounting the checkout shares
`target/` with the host — pass `-e CARGO_TARGET_DIR=/work/target-container` if
host and container builds should not fight over it.

The `dev` stage bakes the channel of `rust-toolchain.toml` (a dated nightly,
ADR-0122), passed as `--build-arg RUST_TOOLCHAIN` by `just solver-image` and the
`solvers-image` workflow. Moving the pin requires a rebuilt, re-pinned `dev` image
before the devcontainer can build.

## Running `build.sh` outside Docker

`build.sh` is plain bash with no Docker assumptions:

```bash
PSE_SOLVERS_PREFIX="$HOME/.local/pse-solvers" docker/solvers/build.sh
export IPOPT_DIR="$HOME/.local/pse-solvers"
export PKG_CONFIG_PATH="$IPOPT_DIR/lib/pkgconfig:$PKG_CONFIG_PATH"
export LD_LIBRARY_PATH="$IPOPT_DIR/lib:$LD_LIBRARY_PATH"
docker/solvers/test/run.sh                                  # same acceptance test
```

It needs Linux x86_64 (oneMKL is pinned as Intel's Linux packages), a
C/C++/Fortran toolchain, `make`, `patch`, `cmake`, `meson` and `ninja`,
`pkg-config`, `dpkg-deb`, `curl` or `wget`, and the development files of hwloc,
GMP, MPFR, Boost and zlib. Bumping a pinned version means updating
`checksums.sha256`, which is only ever written by
`PSE_SOLVERS_UPDATE_CHECKSUMS=1 docker/solvers/build.sh`; an archive with no
entry is a hard error, and a mismatch deletes the cached copy and refuses to
build. Record where each new checksum was corroborated in the file's header.

## HSL

HSL (MA27/MA57/MA77/MA86/MA97) is excluded: there is no licence route
(ADR-0108, register R-34). Ipopt is configured `--without-hsl` and
`--disable-linear-solver-loader`, so the `ma*` values of `linear_solver`, the
`pardiso` value and the `hsllib` option do not exist in this build and a
request for them is an invalid option, never a `dlopen` attempt. (Ipopt 3.14
still registers `pardisolib`, but no `linear_solver` value can use it.) The
symmetric-indefinite alternatives to MUMPS are SPRAL SSIDS (`linear_solver=spral`)
and oneMKL Pardiso (`linear_solver=pardisomkl`).

## Phase-1b: macOS and Windows without Docker

Until `build.sh` runs natively in phase 2, the non-Linux jobs use a packaged
Ipopt at the same version:

**macOS** (Homebrew, `ipopt` 3.14.20, built with MUMPS and ASL):

```bash
brew install ipopt pkg-config
export IPOPT_DIR="$(brew --prefix ipopt)"
export PKG_CONFIG_PATH="$IPOPT_DIR/lib/pkgconfig:$PKG_CONFIG_PATH"
export DYLD_LIBRARY_PATH="$IPOPT_DIR/lib:$DYLD_LIBRARY_PATH"
```

**Windows** (conda-forge `ipopt` 3.14.20 via micromamba, `shell: bash`):

```bash
micromamba create -y -n pse-solvers -f docker/solvers/conda/env.yml
micromamba activate pse-solvers
export IPOPT_DIR="$CONDA_PREFIX/Library"        # "$CONDA_PREFIX" on Linux/macOS
export PKG_CONFIG_PATH="$IPOPT_DIR/lib/pkgconfig:$PKG_CONFIG_PATH"
```

Both are non-gating until a green week (plan section 4). Neither package
meets the ADR-0108 contract — no pinned oneMKL, no `MKL_CBWR`, and not
necessarily SPRAL, Pardiso or SCIP — so they serve linkage and API checks only;
other platforms stay deferred (register R-08). The same
[`conda/env.yml`](./conda/env.yml) also gives Linux users a Docker-free Ipopt for
the same limited purpose.
