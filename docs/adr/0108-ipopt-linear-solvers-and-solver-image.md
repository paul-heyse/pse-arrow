---
id: ADR-0108
title: Build one solver image with MUMPS+METIS, SPRAL and oneMKL Pardiso; select Ipopt linear solvers explicitly; exclude HSL
status: accepted
date: 2026-09-27
deciders: [paul-heyse]
level: decision
principles: [AP-05, DP-11, DP-15, DP-20, PS-09, PS-11]
blueprint: [§6.13, §18.3, §18.8, §18.10, §26]
review: docs/design_review/reviews/design_review_plan22-target_2026-09-27.md#t04
evidence: Interface-checked
supersedes: [ADR-0028]
superseded-by: null
revisit: A licence route for HSL is obtained (register R-34), a oneMKL release changes its conditional-numerical-reproducibility contract, a macOS or Windows build is requested (R-08), or the Q1 measurement shows neither SPRAL nor MKL Pardiso improving on MUMPS+METIS for n_KKT ≥ 10⁴.
verification: Plan 22 N1 tests ipopt_spral_selectable_and_recorded, ipopt_pardisomkl_selectable_under_cbwr, ipopt_linear_solver_in_profile_key, ipopt_mumps_metis_ordering_selectable and ipopt_unavailable_linear_solver_refused, plus spral_refused_without_omp_cancellation and single_blas_provider_in_process; python / parity test_00_preflight; the weekly solvers-image rebuild check; the Q1 measurement of MUMPS+METIS, SPRAL, MKL Pardiso and FERAL on a process case with n_KKT ≥ 10⁴.
standard: core-3.0/process-simulator-1.1
scenarios: [docs/plans/22-solver-capabilities-architecture.md#s18]
---

# ADR-0108: Build one solver image with MUMPS+METIS, SPRAL and oneMKL Pardiso; select Ipopt linear solvers explicitly; exclude HSL

## Context

ADR-0028 builds Ipopt 3.14.20 with MUMPS and ASL from pinned sources in a container, with
netlib reference BLAS/LAPACK and `--without-metis` "so MUMPS ordering matches IDAES's", and
probes HSL at run time with an optional `fallback_linear_solver`. The fallback contradicts
§18's no-fallback rule, and `ipopt.rs` forces MUMPS
([F11](../design_review/reviews/design_review_solver-capabilities_2026-09-27.md#f11)). The
IDAES premise does not hold either: idaes-ext 3.4.2 (`scripts/compile_solvers.sh`) builds
ThirdParty/Metis and links HSL when present, so IDAES's default is MA27.

On 2026-09-27 the maintainer excluded HSL (no licence route) and selected SPRAL SSIDS and
oneMKL Pardiso, both licensed for personal use, with MUMPS rebuilt with METIS
([execution packet](../plans/22-solver-capabilities-execution.md), decision 2).

The Ipopt 3.14.20 sources set hard constraints (*Interface-checked* in the
`native-solver-libraries` corpus):
- `pardisomkl` exists only when MKL is Ipopt's BLAS/LAPACK (`doc/install.dox`, "Pardiso from Intel MKL");
- SPRAL requires `OMP_CANCELLATION=TRUE` and `OMP_PROC_BIND=TRUE` in the environment (same file);
- `mumps_pivot_order` defaults to 7, automatic choice, which picks METIS once it is linked (`IpMumpsSolverInterface.cpp`);
- with MUMPS linked and no HSL, Ipopt's default `linear_solver` stays `mumps` (`IpAlgBuilder.cpp` precedence).

oneMKL's conditional numerical reproducibility requires a pinned code branch and a constant
thread count.

## Scope

Supersedes ADR-0028 and restates its retained parts. Binds the solver image's composition and
the typed selection of Ipopt's linear solver, including threads and determinism. SCIP's
components in the same image are ADR-0105. Wheel linkage (R-09), other platforms (R-08) and
the pin-update token (R-21) stay deferred and move to this record.

## Drivers

- **DP-15.** Selection is by declared capability and explicit policy; an absent library is
  refused, never substituted.
- **DP-11.** A multithreaded, CPU-dispatching library needs a stated determinism contract.
- **DP-20 and AP-05.** OpenMP and MKL threads are admitted resources, not ambient
  configuration; one OpenMP runtime and one BLAS/LAPACK provider per process.
- **PS-09.** The linear solver is part of the recorded solver selection.
- **PS-11.** A build change that alters default factorization order is a result-affecting input.

## Options

| Option | Assessment | Selection |
|---|---|---|
| Keep ADR-0028 (MUMPS without METIS, HSL probe with fallback) | The fallback violates §18; HSL has no licence route; no high-performance symmetric-indefinite option for large KKT systems | Rejected |
| HSL MA27/MA57 through the runtime loader | No licence route | Excluded (register R-34) |
| MKL for Pardiso only, netlib BLAS elsewhere | Impossible in one Ipopt library: `pardisomkl` requires MKL as Ipopt's LAPACK; two BLAS providers in one process risk symbol interposition | Rejected |
| Two Ipopt builds (netlib and MKL) | Two libraries and two ABIs for one role; SCIP would link only one | Rejected |
| One image: oneMKL as the process BLAS/LAPACK, METIS shared by MUMPS and SPRAL, SPRAL and MKL Pardiso beside MUMPS, typed selection, pinned CBWR | One provider per role; explicit threads and determinism | **Selected** |

## Outcome

**Retained from ADR-0028.**

1. `docker/solvers/` builds every native solver library from checksum-verified pinned sources
   on a digest-pinned `ubuntu:24.04`: Ipopt `releases/3.14.20`, ThirdParty-Mumps
   `releases/3.0.14` and ThirdParty-ASL `releases/2.1.0` (ASL serves the parity container's
   Ipopt executable only). The image contains exactly one Ipopt, which SCIP also links.
2. No third-party binaries are fetched: `idaes get-extensions` and conda-forge remain rejected
   for CI.
3. The options actually passed to `CreateIpoptProblem` and the image's linear-solver inventory
   are recorded with each result.
4. The parity preflight fails rather than skips; the weekly rebuild compares library checksums.

**Replaced and added.**

5. **One BLAS/LAPACK and one OpenMP runtime.** A pinned oneMKL release (LP64 interface, GNU
   OpenMP threading layer `mkl_gnu_thread`) is Ipopt's BLAS/LAPACK and the single BLAS/LAPACK
   provider of every process that loads the native profile; Clarabel's SDP profile uses it
   too. libgomp is the single OpenMP runtime, shared by MKL and SPRAL. The netlib reference
   BLAS/LAPACK of ADR-0028 is removed.
6. **METIS.** One pinned METIS release is built once and shared by MUMPS and SPRAL; MUMPS is
   rebuilt with METIS, replacing `--without-metis`. MUMPS stays the sequential (non-MPI) build.
7. **SPRAL SSIDS.** A pinned SPRAL release built with METIS, hwloc and OpenMP (CPU only, no
   GPU) against the image's MKL; Ipopt is configured `--with-spral`.
8. **oneMKL Pardiso.** Available as `linear_solver = pardisomkl` through the MKL LAPACK link.
9. **HSL is excluded.** Ipopt is configured without HSL; the HSL probe and
   `fallback_linear_solver` are removed. The typed setting refuses `ma27`, `ma57`, `ma77`,
   `ma86`, `ma97` and the runtime-loaded `pardiso`; `hsllib` and `pardisolib` are reserved.
10. **Typed selection.** `BackendSettings::Ipopt` carries `linear_solver ∈ {mumps, spral,
    pardisomkl}` with per-solver typed parameters, plus `ordering`, `mu_strategy`, `bound_push`
    and the other admitted controls. `linear_solver` is reserved from raw native options. The
    default is `mumps` with `ordering = metis`, stated explicitly rather than inherited from
    `mumps_pivot_order = 7`, so the METIS build cannot silently change a default. The parity
    profile `idaes-2.13` names its own solver and ordering.
11. **Absent means refused.** A linear solver missing from the linked image, or whose
    preconditions are unmet, is a typed `Unsupported` refusal before the solve, naming the
    missing library or setting. There is no fallback.
12. **Threads (§18.8).** SPRAL and MKL threads are admitted: the adapter acquires CPU permits
    for the requested count, calls `omp_set_num_threads` and `mkl_set_num_threads_local` on the
    owning worker thread before the solve and restores them after, with `MKL_DYNAMIC=FALSE`.
    MUMPS stays serial.
13. **SPRAL preconditions.** `OMP_CANCELLATION=TRUE` and `OMP_PROC_BIND=TRUE` are process-level
    and read when the OpenMP runtime starts. The solver-image environment and the `pse-worker`
    binary (ADR-0112) set them. Admission checks `omp_get_cancellation()` and the bind policy on
    the owning worker and refuses SPRAL when they are unmet.
14. **Determinism (DP-11).** `MKL_CBWR` is pinned to one branch recorded in the image manifest
    and checked at admission with `mkl_cbwr_get`; a mismatch is refused. The profile key records
    the MKL, METIS, SPRAL and MUMPS versions, the CBWR branch, the linear solver, the ordering
    and the thread counts. The claim is run-to-run reproducibility within one image, CBWR branch
    and thread count, and numerical — never bitwise — agreement across linear solvers and thread
    counts.

### Consequences

- Every Ipopt route, including MUMPS, now runs on MKL BLAS under the CBWR contract.
- The default MUMPS ordering changes to METIS, explicitly and recorded; results shift within
  tolerance. Parity makes no iteration-count claim.
- The image grows substantially (MKL).
- Licence terms are a distribution question for the first wheel (R-09, R-31); personal use is
  confirmed.

### Compensating controls

Admission checks for CBWR and OpenMP settings; a single-provider check for BLAS/LAPACK; the
profile key; register rows R-08, R-09, R-21 (moved here) and R-34 (HSL).

### Confirmation

The tests named in `verification:` (Plan 22 N1, owned with the T3 image track). The Q1
measurement settles whether SPRAL or Pardiso earns its place on large KKT systems; until then
their performance value is *Proposed*.

## Pros and cons

The image gains two high-performance inertia-reporting factorizations that complement MUMPS,
at the cost of MKL's size, a process-wide BLAS and OpenMP policy, and a determinism contract
weaker than bitwise. Excluding HSL loses IDAES's default solver; parity never claimed iteration
equality.

## More information

- Architecture companion [§5.4](../plans/22-solver-capabilities-architecture.md#54-one-solver-image-ipopt-linear-algebra-mkl-and-openmp).
- Target review [T04](../design_review/reviews/design_review_plan22-target_2026-09-27.md#t04), [T05](../design_review/reviews/design_review_plan22-target_2026-09-27.md#t05) and [T06](../design_review/reviews/design_review_plan22-target_2026-09-27.md#t06).
- Capability review F11 and L-N2.
- Supersedes ADR-0028. Register rows R-08, R-09, R-21, R-34. Plan 22 packets N1, A7.

## Status history

- 2026-09-27 — proposed (Plan 22 D0). Supersedes ADR-0028: the container build from pinned
  sources, one Ipopt, recorded options and the parity and rebuild checks are retained; netlib
  BLAS, `--without-metis`, the HSL probe and `fallback_linear_solver` are replaced.
- 2026-09-27 — accepted under the maintainer's authorization of the full Plan 22 scope (2026-09-27), after the [Plan 22 target review](../design_review/reviews/design_review_plan22-target_2026-09-27.md#decision) returned Accept (author review, Proposed evidence level). Findings T04, T05, T06 were corrected in this record before acceptance.
