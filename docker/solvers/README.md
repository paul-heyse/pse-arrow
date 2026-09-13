<!--
SPDX-License-Identifier: MIT OR Apache-2.0
Copyright (c) 2026 Paul Heyse
-->

# `pse-solvers` — the container `pse-arrow` links Ipopt against

`pse-arrow` needs Ipopt **3.14.x**; the Ipopt that Ubuntu 24.04 ships is 3.11.9,
which predates the `IpStdCInterface` shape that `pse-ipopt-sys` binds and the
`hsllib` run-time loader. This directory is the single recipe that produces a
known-good Ipopt for CI, for the devcontainer, and — run natively — for macOS
and Windows in phase 2.

It builds, from pinned and checksummed sources:

| | version | notes |
|---|---|---|
| Ipopt | 3.14.20 | shared, `--disable-java`, linear-solver loader on |
| MUMPS (sequential) | 5.9.1 (ThirdParty-Mumps 3.0.14) | shared, **`--without-metis`** |
| AMPL Solver Library | `solvers-20241108` (ThirdParty-ASL 2.1.0) | static, `-fPIC` |
| BLAS / LAPACK | Ubuntu netlib `libblas3` / `liblapack3` | reference, single-threaded |
| HSL | — | never in an image; see [Adding HSL locally](#adding-hsl-locally) |

Install prefix is `/opt/pse-solvers` (`PSE_SOLVERS_PREFIX` overrides it), holding
`lib/libipopt.so`, `lib/libcoinmumps.so`, `lib/libcoinasl.a`, `bin/ipopt`,
`include/coin-or/**`, `lib/pkgconfig/{ipopt,ipoptamplinterface,coinmumps,coinasl}.pc`
and a `share/pse-solvers/build-info.txt` manifest recording versions and source
checksums.

Why these choices — netlib BLAS for determinism, `--without-metis` for MUMPS
ordering parity with IDAES's own binaries, no HSL — is argued in the comment
header of [`build.sh`](./build.sh) and recorded as ADR-0028.

## Stages

| Target | Contains | Used by |
|---|---|---|
| `builder` | build-essential, gfortran, the sources; runs `build.sh` **and `test/run.sh`** | nothing ships from here |
| `solvers` | runtime libs + headers + `.pc`; `IPOPT_DIR`, `PKG_CONFIG_PATH`, `LD_LIBRARY_PATH`, `PATH` set | base of `ci` |
| `ci` | `solvers` + build-essential, git, curl, pkg-config, `libclang-dev` (bindgen), python3, `uv` 0.12.13 with managed CPython 3.11/3.12/3.13 | every CI job that compiles or solves |
| `dev` | `ci` + rustup at the pinned toolchain, `cargo-binstall`, `just`, `cargo-nextest`; `UV_PYTHON_DOWNLOADS=automatic` | `.devcontainer`, local shells |

The three CPythons baked into `ci` are the **parity matrix** (3.11-3.13, the
range `idaes-pse==2.12.0` supports), not the platform interpreter: `.python-version`
asks for 3.14.7, which nothing in the image provides. CI is hermetic
(`UV_PYTHON_DOWNLOADS=never`), so a CI job must never need 3.14; the `dev` stage
flips that one variable to `automatic` so `just bootstrap` inside the
devcontainer can fetch 3.14.7 on first use. That single variable is the only
difference between `ci` and `dev` other than Rust.

`test/run.sh` runs inside `builder`, so a broken prefix can never be tagged: it
checks `pkg-config --modversion ipopt`, compiles Ipopt's own `hs071_c.c` through
`pkg-config --cflags --libs ipopt` and asserts the HS071 optimum with MUMPS as
the linear solver, writes a hand-written `tiny.nl` (`min (x-1)^2`) and solves it
with `ipopt tiny.nl -AMPL` to prove the ASL driver, and asserts
`ldd libipopt.so` shows MUMPS and neither HSL, METIS nor a threaded BLAS.

## Tags

`ghcr.io/paul-heyse/pse-solvers`:

| Tag | Meaning |
|---|---|
| `ipopt3.14.20-mumps5.9.1-asl20241108-r1` | the `solvers` stage, named by content; `-rN` bumps when the recipe changes without a version change |
| `ci-<tree-hash>` | the `ci` stage for that recipe |
| `dev-<tree-hash>` | the `dev` stage for that recipe |
| `dev-latest` | moving alias for the devcontainer, updated by `solvers-image.yml` |

`<tree-hash>` is `git rev-parse HEAD:docker/solvers` (the git tree object of this
directory — it changes exactly when a file here changes), truncated to 12 hex
characters. Outside a clean checkout, the equivalent fallback is
`sha256(docker/solvers/**)[:12]` over the sorted file list.

`ipopt -v` inside the image prints `Ipopt 3.14.20 (x86_64-pc-linux-gnu),
ASL(20241111)`: `20241108` in the tag is the ASL *release tarball* name,
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
recipe drift.

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

The build context is `docker/solvers/`, not the repository root. On 32 cores the
solver compile is ~25 s and the whole `ci` target ~1 min from cold; the download
cache is a BuildKit cache mount, so a re-run after a failed step re-fetches
nothing.

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

## Running `build.sh` outside Docker

`build.sh` is plain POSIX-ish bash with no Docker assumptions:

```bash
PSE_SOLVERS_PREFIX="$HOME/.local/pse-solvers" docker/solvers/build.sh
export IPOPT_DIR="$HOME/.local/pse-solvers"
export PKG_CONFIG_PATH="$IPOPT_DIR/lib/pkgconfig:$PKG_CONFIG_PATH"
export LD_LIBRARY_PATH="$IPOPT_DIR/lib:$LD_LIBRARY_PATH"   # DYLD_LIBRARY_PATH on macOS
docker/solvers/test/run.sh                                  # same acceptance test
```

It needs a C/C++/Fortran toolchain, `make`, `patch`, `pkg-config`, `curl` or
`wget`, and a reference BLAS/LAPACK. Bumping a pinned version means updating
`checksums.sha256`, which is only ever written by
`PSE_SOLVERS_UPDATE_CHECKSUMS=1 docker/solvers/build.sh`; a tarball with no entry
is a hard error, and a mismatch deletes the cached copy and refuses to build.

## Adding HSL locally

HSL is not redistributable and is never in an image. With your own licence:

1. Build `libhsl.so` from the HSL archive (or use
   [`ThirdParty-HSL`](https://github.com/coin-or-tools/ThirdParty-HSL) with the
   same prefix).
2. Put it somewhere the container can see it and point Ipopt at it:

   ```bash
   docker run --rm -it \
     -v "$PWD":/work -w /work \
     -v "$HOME/hsl/lib":/opt/hsl:ro \
     -e LD_LIBRARY_PATH=/opt/pse-solvers/lib:/opt/hsl \
     ghcr.io/paul-heyse/pse-solvers:dev-latest bash
   ```

3. Select it per solve with Ipopt's options — `linear_solver=ma57` plus
   `hsllib=libhsl.so` — which the 3.14 linear-solver loader `dlopen`s at run
   time. This build keeps that loader enabled
   (`--enable-linear-solver-loader`), which is the only reason it works without
   relinking.

`probe_host` records which of `ma27/ma57/ma86/ma97` are reachable, and the
golden/trajectory tests are only compared against MUMPS runs — an HSL run is a
local convenience, never a CI baseline.

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

Both are non-gating until a green week (plan section 4); trajectory-parity tests
are filtered off Linux because conda-forge's and Homebrew's MUMPS are *not*
built `--without-metis`, so iteration counts differ. The same
[`conda/env.yml`](./conda/env.yml) also gives Linux users a Docker-free local
setup.
