#!/usr/bin/env bash
# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
#
# Post-build acceptance test for the pse-solvers prefix.  The Dockerfile runs it
# in the `builder` stage, so an image that cannot compile against Ipopt or
# SCIP, lacks a linear solver, or carries a second BLAS is never tagged.
#
#   1. pkg-config, headers and the build manifest
#   2. oneMKL: the pinned CNR branch is in force (by environment and by
#      mkl_cbwr_set), and DGEMM/DSYTRF run through the dispatch kernels
#   3. Ipopt's linear-solver inventory: mumps, spral, pardisomkl; no HSL,
#      no pardiso-project Pardiso, no run-time loader
#   4. Ipopt's own hs071_c example solves HS071 through the C interface with
#      each of mumps, spral and pardisomkl (selected via ipopt.opt); an HSL
#      solver is refused and SPRAL without OMP_CANCELLATION fails
#   5. `ipopt tiny.nl -AMPL` solves a hand-written NL file (ASL reader/writer)
#   6. SCIP 10.0.2: version and headers agree, Ipopt/SoPlex/PaPILO/GMP/MPFR
#      are linked, and a small nonconvex MINLP reaches its known optimum
#   7. linkage: one BLAS/LAPACK (oneMKL from the prefix), one OpenMP runtime
#      (libgomp), METIS in MUMPS and SPRAL, nothing from HSL

set -euo pipefail

PREFIX=${PSE_SOLVERS_PREFIX:-/opt/pse-solvers}
EXPECTED_IPOPT_VERSION=${EXPECTED_IPOPT_VERSION:-3.14.20}
BUILD_INFO=$PREFIX/share/pse-solvers/build-info.txt

export PKG_CONFIG_PATH="$PREFIX/lib/pkgconfig${PKG_CONFIG_PATH:+:$PKG_CONFIG_PATH}"
export LD_LIBRARY_PATH="$PREFIX/lib${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}"
export PATH="$PREFIX/bin:$PATH"
# The image's process environment (Dockerfile `solvers` stage); set here too
# because this script runs in `builder`, and natively against any prefix.
export MKL_CBWR=${MKL_CBWR:-COMPATIBLE}
export MKL_DYNAMIC=${MKL_DYNAMIC:-FALSE}
export MKL_NUM_THREADS=${MKL_NUM_THREADS:-1}
export OMP_NUM_THREADS=${OMP_NUM_THREADS:-1}
export OMP_CANCELLATION=${OMP_CANCELLATION:-TRUE}
export OMP_PROC_BIND=${OMP_PROC_BIND:-TRUE}
export OMP_PLACES=${OMP_PLACES:-sockets}
export HWLOC_COMPONENTS=${HWLOC_COMPONENTS:--linuxio,-pci,-opencl,-cuda,-nvml,-rsmi,-levelzero,-gl}

SCRIPT_DIR=$(CDPATH='' cd -- "$(dirname -- "$0")" && pwd)
CC_BIN=${CC:-cc}

ok()  { printf '  ok   %s\n' "$*"; }
die() { printf '\nrun.sh: FAILED: %s\n' "$*" >&2; exit 1; }

WORK=$(mktemp -d "${TMPDIR:-/tmp}/pse-solvers-test.XXXXXX")
trap 'rm -rf "$WORK"' EXIT
cd "$WORK"

printf '\n==> 1. pkg-config, headers and manifest\n'
version=$(pkg-config --modversion ipopt) || die 'pkg-config cannot see ipopt'
[ "$version" = "$EXPECTED_IPOPT_VERSION" ] \
  || die "pkg-config --modversion ipopt = $version, expected $EXPECTED_IPOPT_VERSION"
ok "pkg-config --modversion ipopt = $version"
for pc in coinmumps coinasl mkl-dynamic-lp64-gomp; do
  pkg-config --exists "$pc" || die "$pc.pc missing"
done
ok "coinmumps $(pkg-config --modversion coinmumps), coinasl $(pkg-config --modversion coinasl), mkl-dynamic-lp64-gomp $(pkg-config --modversion mkl-dynamic-lp64-gomp)"
for header in coin-or/IpStdCInterface.h coin-or/IpLinearSolvers.h spral_ssids.h metis.h mkl.h \
              scip/scip.h scip/scipdefplugins.h scip/config.h; do
  [ -f "$PREFIX/include/$header" ] || die "missing $PREFIX/include/$header"
done
ok 'Ipopt, SPRAL, METIS, oneMKL and SCIP headers present'
[ -f "$BUILD_INFO" ] || die "missing $BUILD_INFO"
cbwr=$(awk '$1 == "mkl_cbwr" { print $2 }' "$BUILD_INFO")
[ "$cbwr" = "$MKL_CBWR" ] || die "build-info records mkl_cbwr '$cbwr' but MKL_CBWR is '$MKL_CBWR'"
ok "build-info.txt present; pinned CNR branch $cbwr"

MKL_FLAGS=$(pkg-config --cflags --libs mkl-dynamic-lp64-gomp)
IPOPT_FLAGS=$(pkg-config --cflags --libs ipopt)

printf '\n==> 2. oneMKL conditional numerical reproducibility\n'
# shellcheck disable=SC2086
"$CC_BIN" -O2 -o mkl_cbwr "$SCRIPT_DIR/mkl_cbwr.c" $MKL_FLAGS -lm
./mkl_cbwr env > cbwr-env.out 2>&1 || { cat cbwr-env.out; die 'MKL_CBWR=COMPATIBLE is not in force'; }
ok "env:  $(sed -n 2p cbwr-env.out)"
env -u MKL_CBWR ./mkl_cbwr set > cbwr-set.out 2>&1 || { cat cbwr-set.out; die 'mkl_cbwr_set(COMPATIBLE) did not take effect'; }
ok "set:  $(sed -n 2p cbwr-set.out)"
ok "$(sed -n 1p cbwr-env.out)"

printf '\n==> 3. Ipopt linear-solver inventory\n'
# shellcheck disable=SC2086
"$CC_BIN" -O2 -o ipopt_linear_solvers "$SCRIPT_DIR/ipopt_linear_solvers.c" $IPOPT_FLAGS
./ipopt_linear_solvers > inventory.out 2>&1 || { cat inventory.out; die 'linear-solver inventory violates the image contract'; }
sed 's/^/       /' inventory.out
ok 'mumps, spral and pardisomkl linked; no HSL, pardiso-project Pardiso, WSMP or loader'

printf '\n==> 4. hs071_c with each linear solver\n'
# shellcheck disable=SC2086
"$CC_BIN" -O2 -o hs071_c "$SCRIPT_DIR/hs071_c.c" $IPOPT_FLAGS -lm
ok 'compiled hs071_c'
run_hs071() {  # linear_solver -> runs in its own directory with an ipopt.opt
  mkdir -p "hs071-$1" && ( cd "hs071-$1" && printf 'linear_solver %s\n' "$1" > ipopt.opt && ../hs071_c > out 2>&1 )
}
for solver in mumps spral pardisomkl; do
  start=$(date +%s%N)
  run_hs071 "$solver" || { cat "hs071-$solver/out"; die "hs071_c failed with linear_solver=$solver"; }
  elapsed_ms=$(( ($(date +%s%N) - start) / 1000000 ))
  # Two HS071 solves take well under a second; SPRAL with hwloc I/O discovery
  # enabled takes ~8 s, so this bound catches a lost HWLOC_COMPONENTS.
  [ "$elapsed_ms" -lt 3000 ] || die "hs071_c with $solver took ${elapsed_ms} ms (HWLOC_COMPONENTS='$HWLOC_COMPONENTS')"
  out=hs071-$solver/out
  [ "$(grep -c 'EXIT: Optimal Solution Found' "$out")" = 2 ] \
    || { cat "$out"; die "hs071_c with $solver did not report two optimal solves"; }
  grep -Eq 'f\(x\*\) = 1\.70140[0-9]+e\+01' "$out" \
    || { cat "$out"; die "hs071_c with $solver did not reach f(x*) = 17.014017"; }
  banner=$(grep -m1 -o 'running with linear solver [^.]*' "$out") \
    || { cat "$out"; die "hs071_c with $solver printed no linear-solver banner"; }
  case "$banner" in
    *"$solver"*|*MUMPS*) ;;
    *) cat "$out"; die "hs071_c asked for $solver but Ipopt reports '$banner'" ;;
  esac
  ok "$solver: 2 optimal solves in ${elapsed_ms} ms, f(x*) = $(grep -m1 'f(x\*)' "$out" | awk '{print $3}'), '$banner'"
done
if run_hs071 ma57; then
  cat hs071-ma57/out; die 'linear_solver=ma57 was accepted'
fi
grep -q 'ma57' hs071-ma57/out || { cat hs071-ma57/out; die 'ma57 refusal did not name the option value'; }
ok 'linear_solver=ma57 refused (not a valid value in this build)'
mkdir -p hs071-spral-nocancel
( cd hs071-spral-nocancel && printf 'linear_solver spral\n' > ipopt.opt \
  && OMP_CANCELLATION=FALSE ../hs071_c > out 2>&1 ) \
  && { cat hs071-spral-nocancel/out; die 'SPRAL solved without OMP_CANCELLATION=TRUE'; }
ok 'spral without OMP_CANCELLATION fails (control)'

printf '\n==> 5. ASL driver on a tiny NL file\n'
# min (x - 1)^2, unconstrained, x0 = 0.  Written by hand in the ASCII "g" NL
# format so the test needs neither AMPL nor Pyomo; the expression tree is
#   O0: o5 (pow) [ o0 (plus) [ v0, n-1.0 ], n2 ].
cat > tiny.nl <<'NL'
g3 1 1 0
 1 0 1 0 0
 0 1 0 0 0 0
 0 0
 0 1 0
 0 0 0 1
 0 0 0 0 0
 0 1
 0 0
 0 0 0 0 0
O0 0
o5
o0
v0
n-1.0
n2
x1
0 0.0
r
b
3
k0
G0 1
0 0
NL
# -AMPL makes the driver write tiny.sol, so the whole .nl -> .sol round trip is
# exercised and not just the reader.
ipopt tiny.nl -AMPL > tiny.out 2>&1 || { cat tiny.out; die 'ipopt exited non-zero on tiny.nl'; }
grep -q 'EXIT: Optimal Solution Found' tiny.out \
  || { cat tiny.out; die 'ipopt did not solve tiny.nl to optimality'; }
grep -Eq 'Objective\.+: +0\.0+e\+00' tiny.out \
  || { cat tiny.out; die 'ipopt did not reach the known optimum f(1) = 0 of tiny.nl'; }
[ -f tiny.sol ] || { cat tiny.out; die 'ipopt wrote no tiny.sol (ASL .sol writer broken)'; }
grep -q 'Optimal Solution Found' tiny.sol \
  || { cat tiny.sol; die 'tiny.sol does not report an optimal solution'; }
ok 'ipopt tiny.nl -AMPL solved to f(x*) = 0 and wrote tiny.sol'

printf '\n==> 6. SCIP Optimization Suite\n'
"$CC_BIN" -O2 -o scip_minlp "$SCRIPT_DIR/scip_minlp.c" -I"$PREFIX/include" -L"$PREFIX/lib" -lscip -lm
./scip_minlp > scip.out 2>&1 || { cat scip.out; die 'SCIP acceptance test failed'; }
sed 's/^/       /' scip.out
ok 'SCIP 10.0.2 solved the nonconvex MINLP to t* = -2*sqrt(2) with Ipopt, SoPlex and PaPILO linked'
scip -v > scip-v.out 2>&1 || { cat scip-v.out; die 'scip -v failed'; }
grep -q 'SCIP version 10.0.2' scip-v.out || { cat scip-v.out; die 'scip -v does not report 10.0.2'; }
ok "scip -v: $(grep -m1 'SCIP version' scip-v.out)"

printf '\n==> 7. linkage\n'
if command -v ldd >/dev/null 2>&1; then
  needed() { readelf -d "$1" | awk '/NEEDED/ { gsub(/[][]/, "", $5); print $5 }'; }
  lib=$PREFIX/lib
  expect() {  # file pattern...
    file=$1; shift
    for pattern in "$@"; do
      needed "$file" | grep -q "$pattern" || { needed "$file"; die "${file##*/} does not link $pattern"; }
    done
  }
  # libtool places `-Wl,--no-as-needed` after the libraries, so libipopt
  # records only the oneMKL interface layer it calls; the threading layer,
  # oneMKL core and libgomp are recorded by MUMPS and SPRAL, which libipopt
  # loads, and the load closure below must contain them.
  expect "$lib/libipopt.so" libcoinmumps libspral libmkl_intel_lp64
  expect "$lib/libcoinmumps.so" libmetis libmkl_intel_lp64 libmkl_gnu_thread libmkl_core libgomp
  expect "$lib/libspral.so" libmetis libhwloc libmkl_intel_lp64 libmkl_gnu_thread libmkl_core libgomp
  expect "$(readlink -f "$lib/libscip.so")" libipopt libgmp libmpfr
  closure=$(ldd "$lib/libipopt.so")
  for dep in libmkl_intel_lp64 libmkl_gnu_thread libmkl_core libgomp libmetis; do
    printf '%s\n' "$closure" | grep -q "$dep" || { printf '%s\n' "$closure"; die "loading libipopt does not load $dep"; }
  done
  ok 'libipopt -> MUMPS, SPRAL, oneMKL; MUMPS and SPRAL -> METIS, oneMKL (interface, GNU threading, core), libgomp; libscip -> Ipopt, GMP, MPFR'
  # One BLAS/LAPACK provider and one OpenMP runtime across everything the
  # prefix ships, and every oneMKL library resolved from the prefix itself.
  : > ldd.all
  for f in "$lib"/*.so* "$PREFIX"/bin/*; do
    if [ -L "$f" ] || [ ! -f "$f" ] || ! readelf -h "$f" >/dev/null 2>&1; then
      continue
    fi
    ldd "$f" 2>/dev/null | sed "s|^|${f##*/}: |" >> ldd.all || true
  done
  ! grep -Eiq 'libblas\.so|liblapack\.so|openblas|libiomp5|libomp\.so|libtbb|hsl' ldd.all \
    || { grep -Ei 'libblas\.so|liblapack\.so|openblas|libiomp5|libomp\.so|libtbb|hsl' ldd.all; die 'a second BLAS/LAPACK, OpenMP runtime, TBB or HSL is linked'; }
  # Lines are "<file>: <dependency> => <path>"; select oneMKL dependencies.
  grep -E ':[[:space:]]+libmkl_[^ ]+ => ' ldd.all > mkl.deps || true
  [ -s mkl.deps ] || die 'no library in the prefix resolves a oneMKL dependency'
  ! grep -vq "=> $lib/libmkl_" mkl.deps \
    || { grep -v "=> $lib/libmkl_" mkl.deps; die 'a oneMKL library resolves outside the prefix'; }
  ! grep -q 'not found' ldd.all || { grep 'not found' ldd.all; die 'unresolved shared-library dependency'; }
  ok 'one BLAS/LAPACK (oneMKL from the prefix), one OpenMP runtime (libgomp), no TBB, no HSL, nothing unresolved'
else
  printf '  skip  linkage checks (no ldd)\n'
fi

printf '\nrun.sh: all solver checks passed (ipopt %s)\n' "$version"
