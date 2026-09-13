#!/usr/bin/env bash
# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
#
# Post-build acceptance test for the pse-solvers prefix.  The Dockerfile runs it
# in the `builder` stage, so an image that cannot compile against Ipopt, cannot
# reach MUMPS, or has no working ASL driver is never tagged.  It also runs
# unchanged against a Homebrew / conda-forge / natively built prefix.
#
#   1. pkg-config reports ipopt 3.14.20 and the C header is installed
#   2. Ipopt's own hs071_c example compiles via `pkg-config --cflags --libs
#      ipopt` and solves HS071 to its known optimum (proves the C interface and
#      the MUMPS linear solver)
#   3. `ipopt tiny.nl -AMPL` solves a hand-written NL file and writes a .sol
#      (proves the ASL reader and writer)
#   4. libipopt is linked against MUMPS and against no HSL

set -euo pipefail

PREFIX=${PSE_SOLVERS_PREFIX:-/opt/pse-solvers}
EXPECTED_IPOPT_VERSION=${EXPECTED_IPOPT_VERSION:-3.14.20}

export PKG_CONFIG_PATH="$PREFIX/lib/pkgconfig${PKG_CONFIG_PATH:+:$PKG_CONFIG_PATH}"
export LD_LIBRARY_PATH="$PREFIX/lib${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}"
export PATH="$PREFIX/bin:$PATH"

SCRIPT_DIR=$(CDPATH='' cd -- "$(dirname -- "$0")" && pwd)
CC_BIN=${CC:-cc}

ok()  { printf '  ok   %s\n' "$*"; }
die() { printf '\nrun.sh: FAILED: %s\n' "$*" >&2; exit 1; }

WORK=$(mktemp -d "${TMPDIR:-/tmp}/pse-solvers-test.XXXXXX")
trap 'rm -rf "$WORK"' EXIT
cd "$WORK"

printf '\n==> 1. pkg-config and headers\n'
version=$(pkg-config --modversion ipopt) || die 'pkg-config cannot see ipopt'
[ "$version" = "$EXPECTED_IPOPT_VERSION" ] \
  || die "pkg-config --modversion ipopt = $version, expected $EXPECTED_IPOPT_VERSION"
ok "pkg-config --modversion ipopt = $version"
[ -f "$PREFIX/include/coin-or/IpStdCInterface.h" ] \
  || die "missing $PREFIX/include/coin-or/IpStdCInterface.h"
ok 'IpStdCInterface.h present'
pkg-config --exists coinmumps || die 'coinmumps.pc missing'
pkg-config --exists coinasl   || die 'coinasl.pc missing'
ok "coinmumps $(pkg-config --modversion coinmumps), coinasl $(pkg-config --modversion coinasl)"

printf '\n==> 2. hs071_c through the C interface\n'
# shellcheck disable=SC2046  # pkg-config output is intentionally word-split
"$CC_BIN" -O2 -o hs071_c "$SCRIPT_DIR/hs071_c.c" $(pkg-config --cflags --libs ipopt) -lm
ok 'compiled hs071_c'
./hs071_c > hs071.out 2>&1 || { cat hs071.out; die 'hs071_c exited non-zero'; }
grep -q 'EXIT: Optimal Solution Found' hs071.out \
  || { cat hs071.out; die 'hs071_c did not report an optimal solution'; }
# HS071's optimum is f(x*) = 17.014017; the warm-started re-solve stays in 17.0x.
grep -Eq 'f\(x\*\) = 1\.70[0-9]+e\+01' hs071.out \
  || { cat hs071.out; die 'hs071_c objective is not the known HS071 optimum'; }
grep -q 'MUMPS' hs071.out || { cat hs071.out; die 'hs071_c did not use MUMPS'; }
ok "$(grep -c 'EXIT: Optimal Solution Found' hs071.out) optimal solves, objective $(grep -m1 'f(x\*)' hs071.out | awk '{print $3}')"

printf '\n==> 3. ASL driver on a tiny NL file\n'
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

printf '\n==> 4. linkage of libipopt\n'
if command -v ldd >/dev/null 2>&1 && [ -f "$PREFIX/lib/libipopt.so" ]; then
  ldd "$PREFIX/lib/libipopt.so" > ldd.out
  grep -qi 'coinmumps' ldd.out || { cat ldd.out; die 'libipopt.so is not linked against MUMPS'; }
  ! grep -qi 'hsl' ldd.out     || { cat ldd.out; die 'libipopt.so links HSL; it must not'; }
  ! grep -qi 'metis' ldd.out   || { cat ldd.out; die 'libipopt.so links METIS; it must not'; }
  ! grep -qi 'openblas\|mkl' ldd.out || { cat ldd.out; die 'libipopt.so links a threaded BLAS'; }
  ok 'libipopt.so -> MUMPS, netlib BLAS/LAPACK, no HSL, no METIS'
else
  printf '  skip  ldd check (not a Linux shared-library prefix)\n'
fi

printf '\nrun.sh: all solver checks passed (ipopt %s)\n' "$version"
