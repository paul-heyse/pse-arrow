<!--
SPDX-License-Identifier: MIT OR Apache-2.0
Copyright (c) 2026 Paul Heyse
-->

# Licences of the software in the `pse-solvers` image

The recipe in this directory (`Dockerfile`, `build.sh`, `test/run.sh`,
`conda/env.yml`, this file and the README) is `MIT OR Apache-2.0` like the rest
of `pse-arrow`. Everything it *builds* is third-party software under its own
terms, listed here. Nothing in this directory vendors third-party source: the
tarballs are downloaded at build time and verified against
[`checksums.sha256`](./checksums.sha256). The single exception is
[`test/hs071_c.c`](./test/hs071_c.c), a verbatim copy of Ipopt's own example
(see below).

| Component | Version | Licence | Source |
|---|---|---|---|
| Ipopt | 3.14.20 | EPL-2.0 | <https://github.com/coin-or/Ipopt> |
| ThirdParty-Mumps (build glue) | 3.0.14 | EPL-2.0 | <https://github.com/coin-or-tools/ThirdParty-Mumps> |
| MUMPS (sequential) | 5.9.1 | CeCILL-C, with BSD-3-Clause parts | <https://mumps-solver.org/> |
| ThirdParty-ASL (build glue) | 2.1.0 | EPL-2.0 | <https://github.com/coin-or-tools/ThirdParty-ASL> |
| AMPL Solver Library (ASL) | `solvers-20241108` | AMPL/Lucent permissive notice | <https://netlib.org/ampl/solvers/> |
| netlib reference BLAS | Ubuntu `libblas3` | BSD-3-Clause-style netlib notice | <https://netlib.org/blas/> |
| netlib reference LAPACK | Ubuntu `liblapack3` | Modified BSD (BSD-3-Clause) | <https://netlib.org/lapack/LICENSE.txt> |
| Ubuntu base image and runtime libraries | 24.04 LTS | per-package (GPL-3.0+ for libgfortran5, with the GCC Runtime Library Exception) | <https://hub.docker.com/_/ubuntu> |

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
MUMPS's own `libseq` stubs) with `--without-metis`, so neither PORD nor METIS
ordering is compiled in.

MUMPS asks that publications relying on it cite Amestoy, Duff, Koster &
L'Excellent, *SIAM J. Matrix Anal. Appl.* **23**(1), 15-41 (2001), and Amestoy,
Buttari, L'Excellent & Mary, *ACM Trans. Math. Software* **45**(1) (2019).
`CITATION.cff` at the repository root should reflect this when Ipopt/MUMPS
results are published.

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

## netlib BLAS and LAPACK

The image links against Ubuntu's reference implementations (`libblas3`,
`liblapack3`), deliberately rather than OpenBLAS or MKL — see `build.sh` for the
determinism argument. LAPACK is under the
[modified BSD licence](https://netlib.org/lapack/LICENSE.txt) (Univ. of
Tennessee, Univ. of California Berkeley, Univ. of Colorado Denver, NAG Ltd.);
the reference BLAS carries the equivalent netlib notice.

## HSL — deliberately absent

HSL (MA27/MA57/MA86/MA97) is **not** redistributable and is never present in any
image built from this directory. Ipopt is configured `--without-hsl` but keeps
its linear-solver loader enabled, so a user who has obtained their own HSL
licence can drop a `libhsl.so` on `LD_LIBRARY_PATH` and select it with Ipopt's
`hsllib` option at run time; `probe_host` records which of `ma27/ma57/ma86/ma97`
are reachable. See [`README.md`](./README.md#adding-hsl-locally).

## Redistribution note

Pushing `ghcr.io/paul-heyse/pse-solvers` distributes binaries of all of the
above. The image therefore carries the EPL-2.0 (Ipopt, COIN-OR glue) and
CeCILL-C (MUMPS) obligations to make the corresponding source available: the
exact sources are the tarballs pinned in `checksums.sha256`, reachable at the
URLs in `build.sh`, and the build configuration is this directory.
