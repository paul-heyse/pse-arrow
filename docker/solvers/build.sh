#!/usr/bin/env bash
# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
#
# Build the native solver prefix $PSE_SOLVERS_PREFIX from pinned, checksummed
# sources (ADR-0108, ADR-0105):
#
#   oneMKL 2026.1.0   the single BLAS/LAPACK provider and the source of Pardiso
#   METIS 5.1.0       one ordering library, shared by MUMPS and SPRAL
#   MUMPS 5.9.1       sequential, rebuilt with METIS
#   SPRAL 2025.09.18  SSIDS with OpenMP, METIS and hwloc; CPU only
#   ASL 20241108      AMPL solver library, only for Ipopt's own `ipopt` driver
#   Ipopt 3.14.20     linear solvers mumps, spral and pardisomkl; no HSL
#   SCIP Optimization Suite 10.0.2
#                     SCIP + SoPlex + PaPILO, IPOPT=ON against the Ipopt above
#
# The script needs no Docker, but oneMKL comes from Intel's Linux x86_64
# packages, so the MKL-based recipe is Linux x86_64 only; other platforms stay
# deferred (register R-08).  It needs: bash, curl or wget, tar, dpkg-deb, make,
# cmake, meson + ninja, a C/C++/Fortran toolchain, pkg-config, and the
# development files of hwloc, GMP, MPFR, Boost and zlib.
#
# Recipe choices, and why:
#
#   * One BLAS/LAPACK and one OpenMP runtime per process (ADR-0108 item 5).
#     oneMKL's LP64 interface with the GNU OpenMP threading layer
#     (`mkl_intel_lp64 mkl_gnu_thread mkl_core`, libgomp) is what Ipopt, MUMPS,
#     SPRAL and SCIP link, through Intel's own `mkl-dynamic-lp64-gomp.pc`.
#     Only that interface and threading layer are installed, so no other
#     layer (ILP64, Intel/TBB threading, sequential, mkl_rt) can be picked up.
#     Pardiso comes from the same MKL (`linear_solver=pardisomkl`).
#     libtool moves the `-Wl,--no-as-needed` of that link line behind the
#     libraries, so libipopt itself records only mkl_intel_lp64; the threading
#     layer, core and libgomp enter the process through libcoinmumps and
#     libspral, whose links record the full line (test/run.sh checks both).
#   * MKL determinism: the image pins `MKL_CBWR` to MKL_CBWR_BRANCH below and
#     records it in build-info.txt.  COMPATIBLE is the only CNR branch oneMKL
#     supports on non-Intel CPUs as well as Intel ones; any other value is
#     ignored on AMD (oneMKL falls back to AUTO without a warning).
#   * METIS 5.1.0 (Apache-2.0), the release bundling its own GKlib.  5.2.1
#     needs an untagged GKlib commit plus downstream patches (among them one
#     removing -march=native), so 5.1.0 is the stable pin.
#   * MUMPS `--with-metis`: Ipopt's `mumps_pivot_order` default (7, automatic)
#     then selects METIS; the typed setting states the ordering explicitly.
#   * No HSL, and no run-time linear-solver loader (ADR-0108 item 9):
#     `--without-hsl --disable-linear-solver-loader` removes ma27/ma57/ma77/
#     ma86/ma97, `pardiso` and the `hsllib` option entirely, so an absent
#     solver is an invalid option value, never a dlopen attempt.  (Ipopt 3.14
#     still registers `pardisolib`, but no `linear_solver` value reaches it.)
#   * SCIP: THREADSAFE, TPI=tny (tinycthread, not OpenMP), shared libraries,
#     exact mode (GMP, MPFR, Boost).  PaPILO is built without TBB so SCIP's
#     presolver cannot start an unadmitted thread pool (DP-20); its LUSOL
#     component links the same MKL.
#   * `-O2 -fPIC`, never `-march=native`: host-specific instruction selection
#     (FMA contraction in particular) would change floating-point results
#     between the build host and CI.  CMake and meson builds use the same
#     optimisation level; METIS's GKlib forces its own -O3 (ISO C99, no
#     -march, no fast-math).
#
# Environment:
#   PSE_SOLVERS_PREFIX  install prefix              (default /opt/pse-solvers)
#   PSE_SOLVERS_CACHE   download cache              (default <work>/cache)
#   PSE_SOLVERS_WORK    scratch build directory     (default a fresh mktemp -d)
#   PSE_SOLVERS_JOBS    parallel build jobs         (default nproc)
#   PSE_SOLVERS_KEEP_BUILD=1   do not delete the scratch directory
#   PSE_SOLVERS_UPDATE_CHECKSUMS=1
#       (re)write checksums.sha256 from what was actually downloaded.  This is
#       the only way the file is ever produced; without it a missing entry is a
#       hard error, so an unpinned download can never be built by accident.

set -euo pipefail

# --------------------------------------------------------------------------
# Pinned versions.  Bumping any of these requires new checksums.sha256 entries
# and, for a pinned family or a changed composition, an ADR (ADR-0108).
# --------------------------------------------------------------------------
IPOPT_VERSION=3.14.20
MUMPS_WRAPPER_VERSION=3.0.14   # coin-or-tools/ThirdParty-Mumps
MUMPS_VERSION=5.9.1            # the MUMPS release its get.Mumps fetches
ASL_WRAPPER_VERSION=2.1.0      # coin-or-tools/ThirdParty-ASL
ASL_RELEASE=solvers-20241108   # the ASL release its get.ASL fetches
METIS_VERSION=5.1.0
SPRAL_VERSION=2025.09.18
MKL_VERSION=2026.1.0           # oneMKL, Intel oneAPI apt pool packages
MKL_SERIES=2026.1
MKL_PACKAGE_BUILD=236
SCIPOPTSUITE_VERSION=10.0.2    # SCIP 10.0.2 + SoPlex + PaPILO
MKL_CBWR_BRANCH=COMPATIBLE     # the image's pinned oneMKL CNR branch

PSE_SOLVERS_PREFIX=${PSE_SOLVERS_PREFIX:-/opt/pse-solvers}

SCRIPT_DIR=$(CDPATH='' cd -- "$(dirname -- "$0")" && pwd)
CHECKSUMS=${PSE_SOLVERS_CHECKSUMS:-$SCRIPT_DIR/checksums.sha256}

log()  { printf '\n==> %s\n' "$*" >&2; }
die()  { printf 'build.sh: error: %s\n' "$*" >&2; exit 1; }

case "$(uname -s)-$(uname -m)" in
  Linux-x86_64) ;;
  *) die "oneMKL is pinned as Intel's Linux x86_64 packages; $(uname -s)-$(uname -m) is not supported (register R-08)" ;;
esac

default_jobs() {
  if command -v nproc >/dev/null 2>&1; then nproc
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
# Downloads.  Every archive this recipe consumes - including the two that the
# upstream get.Mumps / get.ASL scripts would fetch themselves - is listed here
# so that all of them go through the same checksum gate.
# --------------------------------------------------------------------------
MKL_POOL=https://apt.repos.intel.com/oneapi/pool/main
MKL_DEBS="intel-oneapi-mkl-core-${MKL_SERIES}-${MKL_VERSION}-${MKL_PACKAGE_BUILD}_amd64.deb
intel-oneapi-mkl-core-devel-${MKL_SERIES}-${MKL_VERSION}-${MKL_PACKAGE_BUILD}_amd64.deb
intel-oneapi-mkl-classic-include-${MKL_SERIES}-${MKL_VERSION}-${MKL_PACKAGE_BUILD}_amd64.deb"

downloads() {
  cat <<EOF
https://github.com/coin-or/Ipopt/archive/refs/tags/releases/${IPOPT_VERSION}.tar.gz Ipopt-${IPOPT_VERSION}.tar.gz
https://github.com/coin-or-tools/ThirdParty-Mumps/archive/refs/tags/releases/${MUMPS_WRAPPER_VERSION}.tar.gz ThirdParty-Mumps-${MUMPS_WRAPPER_VERSION}.tar.gz
https://github.com/coin-or-tools/ThirdParty-ASL/archive/refs/tags/releases/${ASL_WRAPPER_VERSION}.tar.gz ThirdParty-ASL-${ASL_WRAPPER_VERSION}.tar.gz
https://coin-or-tools.github.io/ThirdParty-Mumps/MUMPS_${MUMPS_VERSION}.tar.gz MUMPS_${MUMPS_VERSION}.tar.gz
https://coin-or-tools.github.io/ThirdParty-ASL/${ASL_RELEASE}.tgz ${ASL_RELEASE}.tgz
https://papers.karypis.org/glaros/files/sw/metis/metis-${METIS_VERSION}.tar.gz metis-${METIS_VERSION}.tar.gz
https://github.com/ralna/spral/archive/refs/tags/v${SPRAL_VERSION}.tar.gz spral-${SPRAL_VERSION}.tar.gz
https://github.com/scipopt/scip/releases/download/v${SCIPOPTSUITE_VERSION}/scipoptsuite-${SCIPOPTSUITE_VERSION}.tgz scipoptsuite-${SCIPOPTSUITE_VERSION}.tgz
EOF
  for deb in $MKL_DEBS; do
    printf '%s/%s %s\n' "$MKL_POOL" "$deb" "$deb"
  done
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
  log "fetching source archives into $CACHE"
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
      printf '# sha256 of every archive docker/solvers/build.sh downloads. Written by\n'
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
unpack() {  # archive destdir-name -> echoes the extracted top-level directory
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
# The linker resolves the DT_NEEDED entries of libspral/libcoinmumps (METIS,
# MKL) through the run-time search path, and configure runs its test programs.
export LD_LIBRARY_PATH="$PSE_SOLVERS_PREFIX/lib${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}"

# The one declared BLAS/LAPACK link line; set by install_mkl.
MKL_PC='mkl-dynamic-lp64-gomp'
MKL_LFLAGS=

# oneMKL libraries installed into the prefix.  Every libmkl_* in the packages
# must be named in exactly one of these lists, so a new CPU kernel in a future
# release is a build failure rather than a silently missing dispatch target.
MKL_LINK_LIBS='libmkl_intel_lp64 libmkl_gnu_thread libmkl_core'
MKL_KERNEL_LIBS='libmkl_def libmkl_mc3 libmkl_avx2 libmkl_avx512 libmkl_avx10
libmkl_vml_def libmkl_vml_mc3 libmkl_vml_avx2 libmkl_vml_avx512 libmkl_vml_avx10
libmkl_vml_cmpt'
MKL_EXCLUDED_LIBS='libmkl_intel_ilp64 libmkl_gf_lp64 libmkl_gf_ilp64
libmkl_intel_thread libmkl_tbb_thread libmkl_sequential libmkl_rt
libmkl_blas95_lp64 libmkl_blas95_ilp64 libmkl_lapack95_lp64 libmkl_lapack95_ilp64'

install_mkl() {
  log "oneMKL $MKL_VERSION (LP64, GNU OpenMP threading)"
  root=$WORK/src/onemkl
  mkdir -p "$root"
  for deb in $MKL_DEBS; do
    dpkg-deb -x "$CACHE/$deb" "$root"
  done
  mkl=$root/opt/intel/oneapi/mkl/$MKL_SERIES
  [ -f "$mkl/include/mkl.h" ] || die "oneMKL headers missing under $mkl"

  known=" $(printf '%s\n' "$MKL_LINK_LIBS" "$MKL_KERNEL_LIBS" "$MKL_EXCLUDED_LIBS" | tr '\n' ' ') "
  for path in "$mkl"/lib/libmkl_*; do
    base=${path##*/}
    base=${base%%.*}
    case "$known" in
      *" $base "*) ;;
      *) die "unclassified oneMKL library $base in $MKL_VERSION: add it to a list in install_mkl" ;;
    esac
  done

  mkdir -p "$PSE_SOLVERS_PREFIX/lib/pkgconfig" "$PSE_SOLVERS_PREFIX/include" \
           "$PSE_SOLVERS_PREFIX/share/licenses/onemkl"
  # shellcheck disable=SC2086  # the lists are whitespace-separated names
  for lib in $MKL_LINK_LIBS $MKL_KERNEL_LIBS; do
    set -- "$mkl"/lib/"$lib".so.[0-9]*
    [ -f "$1" ] || die "oneMKL $MKL_VERSION has no $lib shared library"
    cp -P "$@" "$PSE_SOLVERS_PREFIX/lib/"
  done
  # Development symlinks only for the three link-line libraries.
  for lib in $MKL_LINK_LIBS; do
    cp -P "$mkl/lib/$lib.so" "$PSE_SOLVERS_PREFIX/lib/"
  done
  # C headers only: the Fortran interface files, FFTW wrappers and the SYCL
  # headers are not part of this contract.
  cp "$mkl"/include/*.h "$PSE_SOLVERS_PREFIX/include/"
  # Intel's own relocatable pkg-config file (prefix=${pcfiledir}/../..).
  cp "$mkl/lib/pkgconfig/$MKL_PC.pc" "$PSE_SOLVERS_PREFIX/lib/pkgconfig/"
  cp "$mkl"/share/doc/mkl/licensing/* "$PSE_SOLVERS_PREFIX/share/licenses/onemkl/"

  MKL_LFLAGS=$(pkg-config --libs "$MKL_PC")
  printf '    link line: %s\n' "$MKL_LFLAGS" >&2
  rm -rf "$root"
}

build_metis() {
  log "METIS $METIS_VERSION"
  dir=$(unpack "metis-${METIS_VERSION}.tar.gz" "metis-${METIS_VERSION}")
  # Upstream's `make config` is a thin wrapper around this CMake call.
  # IDXTYPEWIDTH and REALTYPEWIDTH stay at their shipped 32, which is what
  # MUMPS (32-bit ints) and SPRAL (-Dmetis64=false) require.
  cmake -S "$dir" -B "$dir/build" \
    -DCMAKE_BUILD_TYPE=Release \
    -DCMAKE_C_FLAGS="$COMMON_CFLAGS" \
    -DCMAKE_INSTALL_PREFIX="$PSE_SOLVERS_PREFIX" \
    -DGKLIB_PATH="$dir/GKlib" \
    -DSHARED=1
  cmake --build "$dir/build" --parallel "$JOBS"
  cmake --install "$dir/build"
  [ -f "$PSE_SOLVERS_PREFIX/lib/libmetis.so" ] || die 'METIS did not install libmetis.so'
}

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
    --enable-relocatable \
    --disable-shared --enable-static \
    CFLAGS="$COMMON_CFLAGS"
  make_install_in "$dir"
}

build_mumps() {
  log "ThirdParty-Mumps $MUMPS_WRAPPER_VERSION (MUMPS $MUMPS_VERSION, METIS $METIS_VERSION)"
  dir=$(unpack "ThirdParty-Mumps-${MUMPS_WRAPPER_VERSION}.tar.gz" "ThirdParty-Mumps-releases-${MUMPS_WRAPPER_VERSION}")
  ( cd "$dir" && PATH="$SHIM:$PATH" PSE_SOLVERS_CACHE="$CACHE" ./get.Mumps )
  configure_in "$dir" \
    --prefix="$PSE_SOLVERS_PREFIX" \
    --enable-relocatable \
    --enable-shared --disable-static \
    --with-metis \
    --with-metis-lflags="-L$PSE_SOLVERS_PREFIX/lib -lmetis" \
    --with-metis-cflags="-I$PSE_SOLVERS_PREFIX/include" \
    --with-lapack-lflags="$MKL_LFLAGS" \
    CFLAGS="$COMMON_CFLAGS" \
    CXXFLAGS="$COMMON_CXXFLAGS" \
    FCFLAGS="$MUMPS_FCFLAGS" \
    ADD_FCFLAGS="-fallow-argument-mismatch"
  # configure only warns when a METIS it was given turns out unusable.
  grep -q -- '-Dmetis' "$dir/Makefile" || die 'MUMPS was configured without METIS'
  make_install_in "$dir"
}

build_spral() {
  log "SPRAL $SPRAL_VERSION"
  dir=$(unpack "spral-${SPRAL_VERSION}.tar.gz" "spral-${SPRAL_VERSION}")
  # buildtype=release with optimization=2 keeps the recipe's -O2; meson adds
  # -fPIC for the shared library.  BLAS and LAPACK resolve through the same
  # pkg-config file as every other consumer of MKL.
  CFLAGS="$COMMON_CFLAGS" CXXFLAGS="$COMMON_CXXFLAGS" FFLAGS="$COMMON_FCFLAGS" \
  meson setup "$dir/build" "$dir" \
    --prefix="$PSE_SOLVERS_PREFIX" \
    --libdir=lib \
    --buildtype=release \
    -Doptimization=2 \
    -Dgpu=false \
    -Dopenmp=true \
    -Dtests=false \
    -Dexamples=false \
    -Dmodules=false \
    -Dlibblas="$MKL_PC" \
    -Dliblapack="$MKL_PC" \
    -Dlibmetis=metis \
    -Dlibmetis_path="$PSE_SOLVERS_PREFIX/lib" \
    -Dlibmetis_version=5 \
    -Dmetis64=false \
    -Dlibhwloc=hwloc
  grep -q 'HAVE_HWLOC' "$dir/build/build.ninja" || die 'SPRAL was configured without hwloc'
  meson compile -C "$dir/build" -j "$JOBS"
  meson install -C "$dir/build"
}

build_ipopt() {
  log "Ipopt $IPOPT_VERSION"
  dir=$(unpack "Ipopt-${IPOPT_VERSION}.tar.gz" "Ipopt-releases-${IPOPT_VERSION}")
  configure_in "$dir" \
    --prefix="$PSE_SOLVERS_PREFIX" \
    --enable-relocatable \
    --enable-shared --disable-static \
    --with-lapack-lflags="$MKL_LFLAGS" \
    --with-mumps \
    --with-spral \
    --with-spral-cflags="-I$PSE_SOLVERS_PREFIX/include" \
    --with-spral-lflags="-L$PSE_SOLVERS_PREFIX/lib -lspral -lmetis -lhwloc $MKL_LFLAGS -lgfortran -lstdc++" \
    --enable-pardisomkl \
    --with-asl \
    --without-hsl \
    --disable-linear-solver-loader \
    --disable-java \
    CFLAGS="$COMMON_CFLAGS" \
    CXXFLAGS="$COMMON_CXXFLAGS" \
    FCFLAGS="$COMMON_FCFLAGS"
  # configure silently drops an optional solver whose check fails; the image
  # contract is all three, so a missing one stops the build here.
  config=$(grep -rl 'IPOPT_HAS_MUMPS' "$dir/src" --include='config.h' | head -n 1)
  [ -n "$config" ] || die 'cannot find the Ipopt config.h written by configure'
  for have in IPOPT_HAS_MUMPS IPOPT_HAS_SPRAL IPOPT_HAS_PARDISO_MKL; do
    grep -Eq "^#define $have 1" "$config" || die "Ipopt configured without $have"
  done
  ! grep -Eq '^#define IPOPT_HAS_(HSL|LINEARSOLVERLOADER) 1' "$config" \
    || die 'Ipopt configured with HSL or the linear-solver loader'
  make_install_in "$dir"
}

build_scip() {
  log "SCIP Optimization Suite $SCIPOPTSUITE_VERSION"
  dir=$(unpack "scipoptsuite-${SCIPOPTSUITE_VERSION}.tgz" "scipoptsuite-${SCIPOPTSUITE_VERSION}")
  # Release with the recipe's -O2 (CMake's own Release default is -O3).
  # AUTOBUILD=OFF: every dependency is requested explicitly and a missing one
  # is a configure error, never a silently smaller SCIP.
  cmake -S "$dir" -B "$dir/build" \
    -DCMAKE_BUILD_TYPE=Release \
    -DCMAKE_C_FLAGS="-fPIC" \
    -DCMAKE_CXX_FLAGS="-fPIC" \
    -DCMAKE_C_FLAGS_RELEASE="-O2 -DNDEBUG" \
    -DCMAKE_CXX_FLAGS_RELEASE="-O2 -DNDEBUG" \
    -DCMAKE_Fortran_FLAGS_RELEASE="-O2" \
    -DCMAKE_POSITION_INDEPENDENT_CODE=ON \
    -DCMAKE_INSTALL_PREFIX="$PSE_SOLVERS_PREFIX" \
    -DCMAKE_INSTALL_LIBDIR=lib \
    -DAUTOBUILD=OFF \
    -DSHARED=ON \
    -DTHREADSAFE=ON \
    -DTPI=tny \
    -DSYM=snauty \
    -DSOPLEX=ON \
    -DLPS=spx \
    -DPAPILO=ON \
    -DTBB=OFF \
    -DLUSOL=ON \
    -DBLA_PREFER_PKGCONFIG=ON \
    -DBLA_PKGCONFIG_BLAS="$MKL_PC" \
    -DIPOPT=ON \
    -DIPOPT_DIR="$PSE_SOLVERS_PREFIX" \
    -DEXACTSOLVE=ON \
    -DGMP=ON \
    -DMPFR=ON \
    -DBOOST=ON \
    -DZLIB=ON \
    -DREADLINE=OFF \
    -DAMPL=ON \
    -DLAPACK=OFF \
    -DZIMPL=OFF \
    -DGCG=OFF \
    -DUG=OFF \
    -DMPI=OFF \
    -DCLIQUER=OFF \
    -DGSL=OFF \
    -DJANSSON=OFF \
    -DWORHP=OFF \
    -DCONOPT=OFF \
    -DBUILD_TESTING=OFF
  cmake --build "$dir/build" --parallel "$JOBS"
  cmake --install "$dir/build"
}

write_build_info() {
  info=$PSE_SOLVERS_PREFIX/share/pse-solvers/build-info.txt
  mkdir -p "$(dirname "$info")"
  {
    printf 'ipopt %s\n' "$IPOPT_VERSION"
    printf 'ipopt_linear_solvers mumps spral pardisomkl\n'
    printf 'mumps %s (ThirdParty-Mumps %s, sequential)\n' "$MUMPS_VERSION" "$MUMPS_WRAPPER_VERSION"
    printf 'metis %s\n' "$METIS_VERSION"
    printf 'spral %s\n' "$SPRAL_VERSION"
    printf 'onemkl %s\n' "$MKL_VERSION"
    printf 'blas_lapack onemkl %s\n' "$MKL_PC"
    printf 'mkl_link %s\n' "$MKL_LFLAGS"
    printf 'mkl_cbwr %s\n' "$MKL_CBWR_BRANCH"
    printf 'openmp libgomp\n'
    printf 'asl %s (ThirdParty-ASL %s)\n' "$ASL_RELEASE" "$ASL_WRAPPER_VERSION"
    printf 'scipoptsuite %s (scip soplex papilo; tpi tny; threadsafe; exact)\n' "$SCIPOPTSUITE_VERSION"
    printf 'hsl no (linear-solver loader disabled)\n'
    printf 'cflags %s\n' "$COMMON_CFLAGS"
    printf '\n# source archives\n'
    sort -k2,2 "$RECORDED"
  } > "$info"
  log "wrote $info"
}

main() {
  mkdir -p "$PSE_SOLVERS_PREFIX"
  write_shim
  fetch_all
  # Order is the dependency order: MKL and METIS first, then MUMPS and SPRAL
  # which link both, then Ipopt (coinasl.pc, coinmumps.pc, SPRAL), then SCIP
  # (ipopt.pc).
  install_mkl
  build_metis
  build_asl
  build_mumps
  build_spral
  build_ipopt
  build_scip
  write_build_info
  log "installed into $PSE_SOLVERS_PREFIX"
}

main "$@"
