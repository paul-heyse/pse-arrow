<!--
SPDX-License-Identifier: MIT OR Apache-2.0
Copyright (c) 2026 Paul Heyse
-->

# Licences of the software in the `pse-solvers` image

The recipe in this directory (`Dockerfile`, `build.sh`, `test/run.sh`, the
`test/*.c` acceptance programs other than `hs071_c.c`, `conda/env.yml`, this
file and the README) is `MIT OR Apache-2.0` like the rest of `pse-arrow`.
Everything it *builds or installs* is third-party software under its own
terms, listed here. Nothing in this directory vendors third-party source: the
archives are downloaded at build time and verified against
[`checksums.sha256`](./checksums.sha256). The single exception is
[`test/hs071_c.c`](./test/hs071_c.c), a verbatim copy of Ipopt's own example
(see below).

| Component | Version | Licence | Source |
|---|---|---|---|
| Ipopt (with sIpopt) | 3.14.20 | EPL-2.0 | <https://github.com/coin-or/Ipopt> |
| ThirdParty-Mumps (build glue) | 3.0.14 | EPL-2.0 | <https://github.com/coin-or-tools/ThirdParty-Mumps> |
| MUMPS (sequential) | 5.9.1 | CeCILL-C, with BSD-3-Clause parts | <https://mumps-solver.org/> |
| METIS | 5.1.0 | Apache-2.0 | <https://github.com/KarypisLab/METIS> |
| SPRAL (SSIDS) | 2025.09.18 | BSD-3-Clause | <https://github.com/ralna/spral> |
| Intel oneAPI Math Kernel Library (oneMKL) | 2026.1.0 | Intel Simplified Software License (October 2022); binary only | <https://www.intel.com/content/www/us/en/developer/tools/oneapi/onemkl.html> |
| ThirdParty-ASL (build glue) | 2.1.0 | EPL-2.0 | <https://github.com/coin-or-tools/ThirdParty-ASL> |
| AMPL Solver Library (ASL) | `solvers-20241108` | AMPL/Lucent permissive notice | <https://netlib.org/ampl/solvers/> |
| SCIP | 10.0.2 | Apache-2.0 | <https://scipopt.org>, <https://github.com/scipopt/scip> |
| SoPlex | from SCIP Optimization Suite 10.0.2 | Apache-2.0 | <https://soplex.zib.de> |
| PaPILO | from SCIP Optimization Suite 10.0.2 | Apache-2.0 | <https://github.com/scipopt/papilo> |
| Code bundled inside SCIP 10.0.2 | — | nauty/Traces: Apache-2.0; dejavu (with sassy): MIT; tclique: Apache-2.0; CppAD: EPL-1.0; tinycthread: zlib; AMPL mp (`reader_nl`): Lucent/AMPL permissive notice; LUSOL (in PaPILO): BSD or MIT | each under `scip/src/*` and `papilo/src/papilo/external` in the suite archive |
| GNU MP (GMP) | Ubuntu 24.04 `libgmp10` | LGPL-3.0-or-later or GPL-2.0-or-later | <https://gmplib.org> |
| GNU MPFR | Ubuntu 24.04 `libmpfr6` | LGPL-3.0-or-later | <https://www.mpfr.org> |
| Boost (headers only, multiprecision) | Ubuntu 24.04 `libboost-dev` 1.83 | BSL-1.0 | <https://www.boost.org> |
| hwloc | Ubuntu 24.04 `libhwloc15` | BSD-3-Clause | <https://www.open-mpi.org/projects/hwloc/> |
| Ubuntu base image and runtime libraries | 24.04 LTS | per-package (GPL-3.0+ with the GCC Runtime Library Exception for libgfortran5, libgomp1 and libquadmath0) | <https://hub.docker.com/_/ubuntu> |

## Ipopt — Eclipse Public License 2.0

Ipopt is distributed under the [EPL-2.0](https://www.eclipse.org/legal/epl-2.0/).
The full text ships as `LICENSE` in the Ipopt tarball. EPL-2.0 is a weak
(file-level) copyleft: linking `pse-arrow` against an unmodified `libipopt.so`
does not place `pse-arrow` under the EPL, but any modification of an Ipopt
source file must be published under the EPL. This recipe modifies nothing — it
only configures and compiles the released sources.

`test/hs071_c.c` is a verbatim copy of `examples/hs071_c/hs071_c.c` from the
Ipopt 3.14.20 tarball (`Copyright (C) 2005, 2011 International Business Machines
and others`), used unchanged as the image's post-build linkage test. Two SPDX
tags were prepended so `reuse lint` can classify it; no code was changed. It is
covered by the EPL-2.0, not by this repository's licence. `REUSE.toml` at the
repository root must carry an entry for `docker/solvers/test/hs071_c.c`.

## MUMPS — CeCILL-C

MUMPS 5.9.1 is `Copyright 1991-2026 CERFACS, CNRS, ENS Lyon, INP Toulouse,
Inria, Mumps Technologies, University of Bordeaux` and is released under the
[CeCILL-C v1 licence](https://cecill.info/licences/Licence_CeCILL-C_V1-en.html),
except for the AMD ordering variants and `[sdcz]MUMPS_TRUNCATED_RRQR` derived
from LAPACK (BSD-3-Clause) and the optional PORD ordering (separate notice in
`PORD/README`). CeCILL-C is the French analogue of the LGPL: it is compatible
with the LGPL and imposes copyleft only on modified MUMPS files, which this
recipe does not produce. The image builds the *sequential* MUMPS (no MPI, using
MUMPS's own `libseq` stubs) with METIS ordering linked.

MUMPS asks that publications relying on it cite Amestoy, Duff, Koster &
L'Excellent, *SIAM J. Matrix Anal. Appl.* **23**(1), 15-41 (2001), and Amestoy,
Buttari, L'Excellent & Mary, *ACM Trans. Math. Software* **45**(1) (2019).
`CITATION.cff` at the repository root should reflect this when Ipopt/MUMPS
results are published.

## METIS — Apache-2.0

METIS 5.1.0 is `Copyright 1995-2013, Regents of the University of Minnesota`,
licensed under the [Apache License 2.0](https://www.apache.org/licenses/LICENSE-2.0)
(`LICENSE.txt` in the tarball). It is built once as `libmetis.so` and linked by
MUMPS and SPRAL. (METIS 4.x, which COIN-OR's ThirdParty-Metis builds, carries a
non-commercial licence; it is not used.)

## SPRAL — BSD-3-Clause

SPRAL 2025.09.18 is `Copyright (c) 2014-2025, The Science and Technology
Facilities Council (STFC)` under the 3-clause BSD licence (`LICENCE` in the
tarball). The image builds the CPU-only library (no CUDA) with OpenMP, METIS
and hwloc. SSIDS asks to be cited as Hogg, Ovtchinnikov & Scott, *ACM Trans.
Math. Software* **42**(1), 1-25 (2016).

## oneMKL — Intel Simplified Software License

oneMKL is distributed by Intel in binary form under the
[Intel Simplified Software License](https://www.intel.com/content/www/us/en/developer/articles/license/end-user-license-agreement.html)
(version October 2022), which permits use and redistribution of the unmodified
binaries provided the copyright notice and terms are reproduced, forbids
reverse engineering and modification, and grants no patent licence beyond its
terms. The image installs the unmodified LP64/GNU-OpenMP shared libraries, the C
headers and Intel's `mkl-dynamic-lp64-gomp.pc` from Intel's apt packages
`intel-oneapi-mkl-core-2026.1`, `intel-oneapi-mkl-core-devel-2026.1` and
`intel-oneapi-mkl-classic-include-2026.1` (2026.1.0-236). Intel's licence text
and third-party notices are installed with them in
`/opt/pse-solvers/share/licenses/onemkl/`, which satisfies the notice
condition for anyone who receives the image. Personal use was confirmed by the
maintainer on 2026-09-27 (ADR-0108); distribution terms for a wheel remain
register rows R-09 and R-31.

## SCIP Optimization Suite — Apache-2.0

SCIP, SoPlex and PaPILO (`Copyright (c) 2002-2026 Zuse Institute Berlin (ZIB)`)
are licensed under the [Apache License 2.0](https://www.apache.org/licenses/LICENSE-2.0)
since SCIP 9 (`LICENSE` in each component). SCIP ships further third-party code
compiled into `libscip.so`, each with its own permissive notice in the suite
archive: nauty/Traces (Apache-2.0) and dejavu with its sassy preprocessor (MIT)
for symmetry, tclique (Apache-2.0), the CppAD automatic-differentiation headers (EPL-1.0), tinycthread (zlib licence)
for the `tny` thread-pool interface, and AMPL's `mp` library (Lucent/AMPL
permissive notice) for the `.nl` reader. PaPILO's LUSOL component is under the
BSD or MIT licence, at the user's choice. GCG, UG and ZIMPL from the same archive are not built.

SCIP links GMP and MPFR dynamically from Ubuntu's `libgmp10` and `libmpfr6`
(LGPL-3.0-or-later; GMP is dual LGPL-3.0/GPL-2.0), which a user may replace;
Boost is used through its headers only (Boost Software License 1.0).

## AMPL Solver Library — AMPL Optimization / Lucent permissive notice

The ASL sources (`solvers-20241108.tgz`) carry, per file:

> Copyright (C) 2016, 2020 AMPL Optimization, Inc.; written by David M. Gay.
>
> Permission to use, copy, modify, and distribute this software and its
> documentation for any purpose and without fee is hereby granted, provided that
> the above copyright notice appear in all copies and that both that the
> copyright notice and this permission notice and warranty disclaimer appear in
> supporting documentation.
>
> The author and AMPL Optimization, Inc. disclaim all warranties with regard to
> this software, including all implied warranties of merchantability and
> fitness. In no event shall the author be liable for any special, indirect or
> consequential damages or any damages whatsoever resulting from loss of use,
> data or profits, whether in an action of contract, negligence or other
> tortious action, arising out of or in connection with the use or performance
> of this software.

This is a permissive, BSD/MIT-class notice. The ASL is linked statically into
Ipopt's `ipopt` executable and `libipoptamplinterface`; it is the code that
reads `.nl` files and writes `.sol` files.

## HSL — excluded

HSL (MA27/MA57/MA77/MA86/MA97) has no licence route for this project and is
excluded (ADR-0108, register R-34). Ipopt is configured `--without-hsl` and
`--disable-linear-solver-loader`, so no HSL or pardiso-project library can be
linked or loaded at run time; the replacement symmetric-indefinite solvers are
SPRAL SSIDS and oneMKL Pardiso.

## Redistribution note

Pushing `ghcr.io/paul-heyse/pse-solvers` distributes binaries of all of the
above. The image therefore carries the EPL-2.0 (Ipopt, COIN-OR glue),
CeCILL-C (MUMPS) and LGPL-3.0 (GMP, MPFR) obligations to make the corresponding
source available: the exact sources are the archives pinned in
`checksums.sha256`, reachable at the URLs in `build.sh`, the Ubuntu packages of
the digest-pinned base, and the build configuration is this directory. oneMKL
is redistributed unmodified with Intel's licence text in the image.
