# syntax=docker/dockerfile:1
# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
#
# pse-solvers: the native solver prefix /opt/pse-solvers built from pinned,
# checksummed sources by build.sh (ADR-0108, ADR-0105): oneMKL 2026.1.0,
# METIS 5.1.0, MUMPS 5.9.1, SPRAL 2025.09.18, ASL 20241108, Ipopt 3.14.20
# (mumps, spral, pardisomkl) and SCIP Optimization Suite 10.0.2 (SCIP, SoPlex,
# PaPILO).  (The `# syntax` parser directive has to be line 1 for BuildKit to
# honour it, so the SPDX header follows it.)
#
#   builder   compiles the solvers and runs the acceptance tests, so a broken
#             image can never be tagged
#   solvers   runtime libraries, headers and pkg-config files only
#   ci        solvers + C toolchain, libclang for bindgen, uv-managed CPython
#   dev       ci + the pinned Rust toolchain; used by .devcontainer
#
# Build context is docker/solvers/:
#   docker build --target ci -t pse-solvers:ci-local -f docker/solvers/Dockerfile docker/solvers

# ---------------------------------------------------------------------------
# Common Ubuntu base.  Pinned by the multi-arch index digest of ubuntu:24.04;
# Dependabot's docker ecosystem updates this line.
# ---------------------------------------------------------------------------
FROM ubuntu:24.04@sha256:224a1869083a311ef3f13648a154ba79832fbef6364d31493642ca03082da254 AS base
SHELL ["/bin/bash", "-euo", "pipefail", "-c"]
ENV DEBIAN_FRONTEND=noninteractive \
    LANG=C.UTF-8 \
    LC_ALL=C.UTF-8
# Keep downloaded .debs so the BuildKit apt cache mounts below actually help.
RUN rm -f /etc/apt/apt.conf.d/docker-clean \
 && printf 'Binary::apt::APT::Keep-Downloaded-Packages "true";\n' > /etc/apt/apt.conf.d/keep-cache

# ---------------------------------------------------------------------------
# builder: compile the prefix, then prove it links and solves.
# ---------------------------------------------------------------------------
FROM base AS builder
RUN --mount=type=cache,target=/var/cache/apt,sharing=locked \
    --mount=type=cache,target=/var/lib/apt,sharing=locked \
    apt-get update && apt-get install -y --no-install-recommends \
      build-essential \
      gfortran \
      git \
      patch \
      wget \
      ca-certificates \
      pkg-config \
      cmake \
      meson \
      ninja-build \
      libhwloc-dev \
      libgmp-dev \
      libmpfr-dev \
      libboost-dev \
      zlib1g-dev

COPY build.sh checksums.sha256 /src/
# The download cache is a BuildKit cache mount, so re-runs of a failed build do
# not re-fetch the ~270 MB of archives (oneMKL is most of it); the checksum
# gate in build.sh still applies to whatever comes out of it.  The scratch
# directory is fixed rather than mktemp'd: CMake and meson compile by absolute
# path, so __FILE__ strings in libscip/libsoplex/libspral embed it, and the
# weekly rebuild check compares the libraries byte for byte.
RUN --mount=type=cache,target=/var/cache/pse-solvers,sharing=locked \
    PSE_SOLVERS_PREFIX=/opt/pse-solvers \
    PSE_SOLVERS_CACHE=/var/cache/pse-solvers \
    PSE_SOLVERS_WORK=/tmp/pse-solvers-build \
    /src/build.sh \
 && rm -rf /tmp/pse-solvers-build

COPY test /src/test
RUN /src/test/run.sh

# ---------------------------------------------------------------------------
# solvers: runtime only.  Everything a consumer needs to link against Ipopt,
# SCIP, SPRAL and oneMKL (libraries, headers, *.pc) and nothing that built it.
# ---------------------------------------------------------------------------
FROM base AS solvers
RUN --mount=type=cache,target=/var/cache/apt,sharing=locked \
    --mount=type=cache,target=/var/lib/apt,sharing=locked \
    apt-get update && apt-get install -y --no-install-recommends \
      libgfortran5 \
      libgomp1 \
      libhwloc15 \
      libgmp10 \
      libmpfr6

COPY --from=builder /opt/pse-solvers /opt/pse-solvers
# The loader cache also serves processes that start with a cleared
# environment, and oneMKL's dlopen of its CPU-dispatch kernels.
RUN printf '/opt/pse-solvers/lib\n' > /etc/ld.so.conf.d/pse-solvers.conf && ldconfig

# Process-level solver environment (ADR-0108 items 12-14):
#   MKL_CBWR=COMPATIBLE   the pinned oneMKL CNR branch; build-info.txt records
#                         it and the check below keeps the two in step.  It is
#                         the only branch oneMKL honours on non-Intel CPUs.
#   MKL_DYNAMIC=FALSE     oneMKL never changes the requested thread count.
#   OMP_NUM_THREADS=1, MKL_NUM_THREADS=1
#                         serial unless a solve is admitted threads; the
#                         adapter raises both per solve and restores them.
#   OMP_CANCELLATION=TRUE required by SPRAL SSIDS (error -53 otherwise).
#   OMP_PROC_BIND=TRUE, OMP_PLACES=sockets
#                         SPRAL wants bound threads.  With OMP_PLACES unset,
#                         libgomp binds every process's initial thread - and
#                         every thread it later creates - to the first CPU, so
#                         concurrent processes would all share one CPU.  Socket
#                         places keep binding enabled without that collapse on
#                         a single-socket host (a multi-socket host confines
#                         each process to its first socket); a worker that owns
#                         specific cores sets its own places.
#   HWLOC_COMPONENTS      SPRAL asks hwloc for PCI and OS devices (to find
#                         GPUs, even in this CPU-only build); that discovery
#                         costs seconds per Ipopt solve (two HS071 solves: 8 s,
#                         0.1 s with the filter).  CPU and NUMA discovery are
#                         unaffected by excluding the I/O components.
ENV IPOPT_DIR=/opt/pse-solvers \
    SCIPOPTDIR=/opt/pse-solvers \
    MKLROOT=/opt/pse-solvers \
    PKG_CONFIG_PATH=/opt/pse-solvers/lib/pkgconfig \
    LD_LIBRARY_PATH=/opt/pse-solvers/lib \
    PATH=/opt/pse-solvers/bin:$PATH \
    MKL_CBWR=COMPATIBLE \
    MKL_DYNAMIC=FALSE \
    MKL_NUM_THREADS=1 \
    OMP_NUM_THREADS=1 \
    OMP_CANCELLATION=TRUE \
    OMP_PROC_BIND=TRUE \
    OMP_PLACES=sockets \
    HWLOC_COMPONENTS=-linuxio,-pci,-opencl,-cuda,-nvml,-rsmi,-levelzero,-gl
RUN grep -qx "mkl_cbwr $MKL_CBWR" /opt/pse-solvers/share/pse-solvers/build-info.txt \
 && ipopt -v >/dev/null && scip -v >/dev/null

LABEL org.opencontainers.image.title="pse-solvers" \
      org.opencontainers.image.description="Ipopt 3.14.20 (MUMPS 5.9.1+METIS 5.1.0, SPRAL 2025.09.18, oneMKL Pardiso) + SCIP Optimization Suite 10.0.2, oneMKL 2026.1.0 BLAS/LAPACK, no HSL" \
      org.opencontainers.image.licenses="MIT OR Apache-2.0" \
      org.opencontainers.image.source="https://github.com/paul-heyse/pse-arrow" \
      io.pse-arrow.ipopt.version="3.14.20" \
      io.pse-arrow.mumps.version="5.9.1" \
      io.pse-arrow.metis.version="5.1.0" \
      io.pse-arrow.spral.version="2025.09.18" \
      io.pse-arrow.onemkl.version="2026.1.0" \
      io.pse-arrow.onemkl.cbwr="COMPATIBLE" \
      io.pse-arrow.asl.version="20241108" \
      io.pse-arrow.scipoptsuite.version="10.0.2"

# ---------------------------------------------------------------------------
# uv, pinned by digest and copied in rather than curl|sh'd.
# ---------------------------------------------------------------------------
FROM ghcr.io/astral-sh/uv:0.12.13@sha256:b485bd65cc2cf1c9a93b3554012c9c3778cf7b1b5fd3d3096ce9e1226c97e1e6 AS uvbin

# ---------------------------------------------------------------------------
# ci: what the Rust and parity jobs run in.
# ---------------------------------------------------------------------------
FROM solvers AS ci
RUN --mount=type=cache,target=/var/cache/apt,sharing=locked \
    --mount=type=cache,target=/var/lib/apt,sharing=locked \
    apt-get update && apt-get install -y --no-install-recommends \
      build-essential \
      git \
      curl \
      ca-certificates \
      pkg-config \
      libclang-dev \
      python3

COPY --from=uvbin /uv /uvx /usr/local/bin/

ENV UV_PYTHON_INSTALL_DIR=/opt/uv/python \
    UV_TOOL_DIR=/opt/uv/tools \
    UV_PYTHON_PREFERENCE=only-managed \
    UV_LINK_MODE=copy
# Managed CPython for the parity matrix (3.11-3.13; idaes-pse 2.12.0 has no
# 3.14 wheels).  The download is allowed exactly here and nowhere afterwards.
RUN --mount=type=cache,target=/root/.cache/uv,sharing=locked \
    UV_PYTHON_DOWNLOADS=automatic uv python install 3.11 3.12 3.13 \
 && chmod -R a+rX /opt/uv
ENV UV_PYTHON_DOWNLOADS=never

# ---------------------------------------------------------------------------
# dev: ci + the toolchain from rust-toolchain.toml.  Used by .devcontainer and
# by `docker run -it` shells; CI jobs use the ci stage.
# ---------------------------------------------------------------------------
FROM ci AS dev
# Keep in step with /rust-toolchain.toml (governance test toolchain_matches_msrv).
ARG RUST_TOOLCHAIN=1.98.1
ARG CARGO_BINSTALL_VERSION=1.23.0
ARG JUST_VERSION=1.58.0
ARG CARGO_NEXTEST_VERSION=0.9.143

ENV RUSTUP_HOME=/usr/local/rustup \
    CARGO_HOME=/usr/local/cargo \
    PATH=/usr/local/cargo/bin:$PATH \
    # The one place `dev` differs from `ci` other than Rust: CI must be hermetic,
    # but a developer's `just bootstrap` needs the interpreter named in
    # /.python-version (3.14.7), which is deliberately not baked in - the three
    # pre-installed CPythons are the parity matrix (3.11-3.13), not the platform
    # interpreter.  `never` here would make the devcontainer unable to create
    # its own venv.
    UV_PYTHON_DOWNLOADS=automatic

# One RUN on purpose: rustup's toolchain is ~1 GB, so a later `chmod -R` (or any
# other whole-tree touch) in a separate layer would copy all of it a second time.
# `--disable-strategies compile` makes a missing prebuilt binary a loud failure
# instead of a 10-minute source build.  The trailing chmod lets a non-root
# devcontainer user share the toolchain.
RUN curl --proto '=https' --tlsv1.2 -fsSL https://sh.rustup.rs \
      | sh -s -- -y --no-modify-path --profile minimal \
          --default-toolchain "$RUST_TOOLCHAIN" \
          -c rustfmt -c clippy -c rust-src -c rust-analyzer -c llvm-tools \
 && arch="$(uname -m)" \
 && url="https://github.com/cargo-bins/cargo-binstall/releases/download/v${CARGO_BINSTALL_VERSION}/cargo-binstall-${arch}-unknown-linux-musl.tgz" \
 && curl --proto '=https' --tlsv1.2 -fsSL "$url" | tar -xz -C "$CARGO_HOME/bin" cargo-binstall \
 && cargo binstall --no-confirm --disable-strategies compile \
      "just@${JUST_VERSION}" \
      "cargo-nextest@${CARGO_NEXTEST_VERSION}" \
 && rm -rf "$CARGO_HOME/registry" "$CARGO_HOME/git" \
      "$RUSTUP_HOME/downloads" "$RUSTUP_HOME/tmp" /root/.cache/cargo-binstall \
 && chmod -R a+rX "$RUSTUP_HOME" "$CARGO_HOME" \
 && rustc --version && cargo --version && just --version && cargo nextest --version

WORKDIR /work
