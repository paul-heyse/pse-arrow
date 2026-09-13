#!/usr/bin/env bash
# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
#
# Build Ipopt + MUMPS + ASL from pinned COIN-OR sources into $PSE_SOLVERS_PREFIX.
#
# The script is deliberately OS-neutral: nothing here is Docker-specific, so the
# same recipe runs natively on macOS (Homebrew toolchain) and under MSYS2 on
# Windows in phase 2 (plan section 4).  It needs: a POSIX shell, curl or wget,
# tar, patch, make, a C/C++ compiler, a Fortran compiler, pkg-config, and a
# reference BLAS/LAPACK.
#
# Recipe choices, and why:
#
#   * netlib reference BLAS/LAPACK (`-llapack -lblas`), never OpenBLAS/MKL.
#     Threaded BLAS reorders reductions, so the same NLP takes a different
#     number of Ipopt iterations from run to run.  Determinism is the whole
#     point of the golden/trajectory tests, so we pay the speed.
#   * `--without-metis`.  IDAES's own `compile_solvers.sh` builds MUMPS without
#     METIS; MUMPS then falls back to AMD/QAMD ordering.  Matching that keeps
#     pivot order - and therefore iteration counts - comparable with the IDAES
#     binaries the parity suite measures against.
#   * No HSL, ever.  HSL is not redistributable, so it is never baked into an
#     image.  Ipopt 3.14 keeps its linear-solver loader enabled, so a local
#     `libhsl.so` is picked up at run time through the `hsllib` option and
#     `probe_host` records which of ma27/ma57/ma86/ma97 are reachable.
#   * `-O2 -fPIC`, never `-march=native`: FMA contraction differs between the
#     build host and CI, which would change float results.
#
# Environment:
#   PSE_SOLVERS_PREFIX  install prefix              (default /opt/pse-solvers)
#   PSE_SOLVERS_CACHE   tarball download cache      (default <work>/cache)
#   PSE_SOLVERS_WORK    scratch build directory     (default a fresh mktemp -d)
#   PSE_SOLVERS_JOBS    make -j value               (default nproc)
#   PSE_SOLVERS_KEEP_BUILD=1   do not delete the scratch directory
#   PSE_SOLVERS_UPDATE_CHECKSUMS=1
#       (re)write checksums.sha256 from what was actually downloaded.  This is
#       the only way the file is ever produced; without it a missing entry is a
#       hard error, so an unpinned tarball can never be built by accident.

set -euo pipefail

# --------------------------------------------------------------------------
# Pinned versions.  Bumping any of these requires new checksums.sha256 entries
# and an ADR (the solver recipe is a register row).
# --------------------------------------------------------------------------
IPOPT_VERSION=3.14.20
MUMPS_WRAPPER_VERSION=3.0.14   # coin-or-tools/ThirdParty-Mumps
MUMPS_VERSION=5.9.1            # the MUMPS release its get.Mumps fetches
ASL_WRAPPER_VERSION=2.1.0      # coin-or-tools/ThirdParty-ASL
ASL_RELEASE=solvers-20241108   # the ASL release its get.ASL fetches

PSE_SOLVERS_PREFIX=${PSE_SOLVERS_PREFIX:-/opt/pse-solvers}

SCRIPT_DIR=$(CDPATH='' cd -- "$(dirname -- "$0")" && pwd)
CHECKSUMS=${PSE_SOLVERS_CHECKSUMS:-$SCRIPT_DIR/checksums.sha256}

log()  { printf '\n==> %s\n' "$*" >&2; }
die()  { printf 'build.sh: error: %s\n' "$*" >&2; exit 1; }

default_jobs() {
  if command -v nproc >/dev/null 2>&1; then nproc
  elif command -v sysctl >/dev/null 2>&1; then sysctl -n hw.ncpu
  else echo 1
  fi
}
JOBS=${PSE_SOLVERS_JOBS:-$(default_jobs)}

sha256_of() {
  if command -v sha256sum >/dev/null 2>&1; then sha256sum "$1" | awk '{print $1}'
  elif command -v shasum >/dev/null 2>&1; then shasum -a 256 "$1" | awk '{print $1}'
  else die 'neither sha256sum nor shasum is available'
  fi
}

# --------------------------------------------------------------------------
# Scratch directories
# --------------------------------------------------------------------------
CREATED_WORK=0
if [ -n "${PSE_SOLVERS_WORK:-}" ]; then
  WORK=$PSE_SOLVERS_WORK
  mkdir -p "$WORK"
else
  WORK=$(mktemp -d "${TMPDIR:-/tmp}/pse-solvers.XXXXXX")
  CREATED_WORK=1
fi
CACHE=${PSE_SOLVERS_CACHE:-$WORK/cache}
SHIM=$WORK/shim
mkdir -p "$CACHE" "$SHIM" "$WORK/src"

cleanup() {
  if [ "$CREATED_WORK" = 1 ] && [ "${PSE_SOLVERS_KEEP_BUILD:-0}" != 1 ]; then
    rm -rf "$WORK"
  fi
}
trap cleanup EXIT

# --------------------------------------------------------------------------
# Downloads.  Every tarball this recipe consumes - including the two that the
# upstream get.Mumps / get.ASL scripts would fetch themselves - is listed here
# so that all of them go through the same checksum gate.
# --------------------------------------------------------------------------
downloads() {
  cat <<EOF
https://github.com/coin-or/Ipopt/archive/refs/tags/releases/${IPOPT_VERSION}.tar.gz Ipopt-${IPOPT_VERSION}.tar.gz
https://github.com/coin-or-tools/ThirdParty-Mumps/archive/refs/tags/releases/${MUMPS_WRAPPER_VERSION}.tar.gz ThirdParty-Mumps-${MUMPS_WRAPPER_VERSION}.tar.gz
https://github.com/coin-or-tools/ThirdParty-ASL/archive/refs/tags/releases/${ASL_WRAPPER_VERSION}.tar.gz ThirdParty-ASL-${ASL_WRAPPER_VERSION}.tar.gz
https://coin-or-tools.github.io/ThirdParty-Mumps/MUMPS_${MUMPS_VERSION}.tar.gz MUMPS_${MUMPS_VERSION}.tar.gz
https://coin-or-tools.github.io/ThirdParty-ASL/${ASL_RELEASE}.tgz ${ASL_RELEASE}.tgz
EOF
}

fetch_one() {  # url dest
  url=$1; dest=$2
  if command -v curl >/dev/null 2>&1; then
    curl --proto '=https' --tlsv1.2 -fsSL --retry 3 --retry-delay 2 -o "$dest.part" "$url"
  elif command -v wget >/dev/null 2>&1; then
    wget -q -O "$dest.part" "$url"
  else
    die 'neither curl nor wget is available'
  fi
  mv -f "$dest.part" "$dest"
}

expected_sha() {  # name -> sha or empty
  [ -f "$CHECKSUMS" ] || return 0
  awk -v n="$1" '$1 !~ /^#/ && $2 == n { print $1; exit }' "$CHECKSUMS"
}

RECORDED=$WORK/checksums.recorded
: > "$RECORDED"

fetch_all() {
  log "fetching source tarballs into $CACHE"
  downloads | while read -r url name; do
    [ -n "$url" ] || continue
    if [ ! -f "$CACHE/$name" ]; then
      printf '    %s\n' "$url" >&2
      fetch_one "$url" "$CACHE/$name"
    fi
    actual=$(sha256_of "$CACHE/$name")
    want=$(expected_sha "$name")
    if [ -z "$want" ]; then
      if [ "${PSE_SOLVERS_UPDATE_CHECKSUMS:-0}" = 1 ]; then
        printf '%s  %s\n' "$actual" "$name" >> "$RECORDED"
        printf '    recorded %s  %s\n' "$actual" "$name" >&2
      else
        die "no sha256 for $name in $CHECKSUMS (re-run with PSE_SOLVERS_UPDATE_CHECKSUMS=1 to record it)"
      fi
    elif [ "$want" != "$actual" ]; then
      rm -f "$CACHE/$name"
      die "sha256 mismatch for $name: expected $want, got $actual (cached copy removed)"
    else
      printf '%s  %s\n' "$actual" "$name" >> "$RECORDED"
      printf '    verified %s  %s\n' "$actual" "$name" >&2
    fi
  done

  if [ "${PSE_SOLVERS_UPDATE_CHECKSUMS:-0}" = 1 ]; then
    {
      # REUSE-IgnoreStart
      printf '# SPDX-License-Identifier: MIT OR Apache-2.0\n'
      # REUSE-IgnoreEnd
      printf '# Copyright (c) 2026 Paul Heyse\n'
      printf '#\n'
      printf '# sha256 of every tarball docker/solvers/build.sh downloads. Written by\n'
      # shellcheck disable=SC2016  # backticks here are Markdown-ish prose, not a subshell
      printf '# `PSE_SOLVERS_UPDATE_CHECKSUMS=1 build.sh`; a missing entry is a hard error.\n'
      sort -k2,2 "$RECORDED"
    } > "$CHECKSUMS"
    log "wrote $CHECKSUMS"
  fi
}

# --------------------------------------------------------------------------
# A `wget` that only ever serves verified bytes.
#
# get.Mumps and get.ASL download MUMPS and the ASL themselves and then apply
# upstream's patches.  Re-implementing the patch steps here would silently rot
# the next time ThirdParty-* changes them, so instead we keep upstream's script
# and put this shim first on PATH: it copies the already-verified tarball out
# of the cache and refuses anything that is not in it.  A miss therefore means
# upstream changed a URL, which must be reviewed rather than downloaded.
# --------------------------------------------------------------------------
write_shim() {
  cat > "$SHIM/wget" <<'EOF'
#!/bin/sh
set -e
url=""
for arg in "$@"; do
  case "$arg" in -*) ;; *) url=$arg ;; esac
done
[ -n "$url" ] || { echo "pse-solvers wget shim: no URL in: $*" >&2; exit 1; }
name=${url##*/}
if [ ! -f "$PSE_SOLVERS_CACHE/$name" ]; then
  echo "pse-solvers wget shim: refusing to fetch unverified $url" >&2
  echo "  ($name is not in $PSE_SOLVERS_CACHE - add it to checksums.sha256 first)" >&2
  exit 1
fi
cp "$PSE_SOLVERS_CACHE/$name" "./$name"
EOF
  chmod +x "$SHIM/wget"
}

# --------------------------------------------------------------------------
# Build helpers
# --------------------------------------------------------------------------
unpack() {  # tarball destdir-name -> echoes the extracted top-level directory
  tar -xzf "$CACHE/$1" -C "$WORK/src"
  printf '%s\n' "$WORK/src/$2"
}

configure_in() {  # dir [args...]
  dir=$1; shift
  if ! ( cd "$dir" && ./configure "$@" ); then
    printf '\n----- %s/config.log (tail) -----\n' "$dir" >&2
    tail -n 200 "$dir/config.log" >&2 || true
    die "configure failed in $dir"
  fi
}

make_install_in() {  # dir
  ( cd "$1" && make "-j$JOBS" && make install )
}

# `-O2 -fPIC` is expressed through ADD_* where a Fortran flag has to survive:
# COIN BuildTools only appends ADD_FCFLAGS when FCFLAGS is unset
# (`: ${FCFLAGS:="-O2 $ADD_FCFLAGS"}` in configure), so anything that must be
# on the Fortran command line goes into FCFLAGS itself.
COMMON_CFLAGS='-O2 -fPIC'
COMMON_CXXFLAGS='-O2 -fPIC'
COMMON_FCFLAGS='-O2 -fPIC'
# gfortran >= 10 rejects MUMPS's libseq MPI stubs without this.
MUMPS_FCFLAGS="$COMMON_FCFLAGS -fallow-argument-mismatch"

export PKG_CONFIG_PATH="$PSE_SOLVERS_PREFIX/lib/pkgconfig${PKG_CONFIG_PATH:+:$PKG_CONFIG_PATH}"

build_asl() {
  log "ThirdParty-ASL $ASL_WRAPPER_VERSION ($ASL_RELEASE)"
  dir=$(unpack "ThirdParty-ASL-${ASL_WRAPPER_VERSION}.tar.gz" "ThirdParty-ASL-releases-${ASL_WRAPPER_VERSION}")
  ( cd "$dir" && PATH="$SHIM:$PATH" PSE_SOLVERS_CACHE="$CACHE" ./get.ASL )
  # Static ASL: the AMPL solver library is only ever linked into Ipopt's own
  # `ipopt` driver and libipoptamplinterface, and a static archive keeps one
  # fewer .so in every consumer's RPATH.  -fPIC is mandatory because it lands
  # inside a shared library.
  configure_in "$dir" \
    --prefix="$PSE_SOLVERS_PREFIX" \
    --disable-shared --enable-static \
    CFLAGS="$COMMON_CFLAGS"
  make_install_in "$dir"
}

build_mumps() {
  log "ThirdParty-Mumps $MUMPS_WRAPPER_VERSION (MUMPS $MUMPS_VERSION)"
  dir=$(unpack "ThirdParty-Mumps-${MUMPS_WRAPPER_VERSION}.tar.gz" "ThirdParty-Mumps-releases-${MUMPS_WRAPPER_VERSION}")
  ( cd "$dir" && PATH="$SHIM:$PATH" PSE_SOLVERS_CACHE="$CACHE" ./get.Mumps )
  configure_in "$dir" \
    --prefix="$PSE_SOLVERS_PREFIX" \
    --enable-shared --disable-static \
    --without-metis \
    --with-lapack="-llapack -lblas" \
    CFLAGS="$COMMON_CFLAGS" \
    CXXFLAGS="$COMMON_CXXFLAGS" \
    FCFLAGS="$MUMPS_FCFLAGS" \
    ADD_FCFLAGS="-fallow-argument-mismatch"
  make_install_in "$dir"
}

build_ipopt() {
  log "Ipopt $IPOPT_VERSION"
  dir=$(unpack "Ipopt-${IPOPT_VERSION}.tar.gz" "Ipopt-releases-${IPOPT_VERSION}")
  # --without-hsl keeps HSL out of the binary; --enable-linear-solver-loader
  # (the default, pinned here so a future default change is loud) keeps the
  # run-time `hsllib` dlopen path that local HSL users rely on.
  configure_in "$dir" \
    --prefix="$PSE_SOLVERS_PREFIX" \
    --enable-shared --disable-static \
    --with-mumps \
    --with-asl \
    --without-hsl \
    --enable-linear-solver-loader \
    --disable-java \
    --with-lapack="-llapack -lblas" \
    CFLAGS="$COMMON_CFLAGS" \
    CXXFLAGS="$COMMON_CXXFLAGS" \
    FCFLAGS="$COMMON_FCFLAGS"
  make_install_in "$dir"
}

write_build_info() {
  info=$PSE_SOLVERS_PREFIX/share/pse-solvers/build-info.txt
  mkdir -p "$(dirname "$info")"
  {
    printf 'ipopt %s\n' "$IPOPT_VERSION"
    printf 'mumps %s (ThirdParty-Mumps %s)\n' "$MUMPS_VERSION" "$MUMPS_WRAPPER_VERSION"
    printf 'asl %s (ThirdParty-ASL %s)\n' "$ASL_RELEASE" "$ASL_WRAPPER_VERSION"
    printf 'blas netlib (liblapack/libblas)\n'
    printf 'metis no\n'
    printf 'hsl no (runtime hsllib loader enabled)\n'
    printf 'cflags %s\n' "$COMMON_CFLAGS"
    printf '\n# source tarballs\n'
    sort -k2,2 "$RECORDED"
  } > "$info"
  log "wrote $info"
}

main() {
  mkdir -p "$PSE_SOLVERS_PREFIX"
  write_shim
  fetch_all
  # ASL and MUMPS first: Ipopt's configure discovers both through the
  # coinasl.pc / coinmumps.pc files they install into the same prefix.
  build_asl
  build_mumps
  build_ipopt
  write_build_info
  log "installed into $PSE_SOLVERS_PREFIX"
}

main "$@"
