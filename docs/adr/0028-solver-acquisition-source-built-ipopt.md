---
id: ADR-0028
title: Build Ipopt 3.14.20 with MUMPS and ASL from pinned sources in a container; probe HSL at runtime
status: accepted
date: 2026-09-13
deciders: [paul-heyse]
level: decision
principles: [DM-28, DM-43, DM-48, DM-45]
blueprint: [§18.3, §6.13, §26]
review: docs/design_review/reviews/design_review_arrow-native-idaes-core-blueprint-rev2_2026-09-13.md#7-principle-findings
evidence: Proposed
supersedes: []
superseded-by: null
revisit: A platform's recipe cannot be reproduced (macOS or Windows at phase 1b), or a wheel needs the native backend and the linkage question comes due
verification: `python / parity` `test_00_preflight` (Ipopt 3.14.x, MUMPS, ASL, `SolverFactory("ipopt").available(False)`); `solvers-image-rebuild-check` (weekly, reproducibility of library checksums)

---

# ADR-0028: Build Ipopt 3.14.20 with MUMPS and ASL from pinned sources in a container; probe HSL at runtime

## Context

Blueprint §18.3 requires Ipopt 3.14 or newer for `GetIpoptCurrentIterate`/`GetIpoptCurrentViolations`, and §6.13 requires an explicit `probe_host` operation that records the available linear solvers. The first review's F10 asked when the capability probe runs and where its result is persisted; the host's system Ipopt is 3.11.9.

## Scope

Binds how solvers are acquired for CI and development, and the fallback policy when a profile names an unavailable linear solver. Wheel linkage is deliberately left open (register row R-09, a phase-1 ADR).

## Drivers

Iteration statistics as data require Ipopt 3.14; parity iteration counts require MUMPS ordering comparable to IDAES's binaries; HSL cannot be redistributed; a capability claimed but unprobed is a lie (DM-43).

## Options

`idaes get-extensions` in CI — rejected: it fetches IDAES's binaries, which is neither clean-room nor pinned. conda-forge `ipopt` on Linux — rejected for CI: the recipe would not be ours to reproduce; it stays the documented Windows route. Bundle HSL — impossible: licensing.

## Outcome

`docker/solvers/` builds Ipopt `releases/3.14.20`, ThirdParty-Mumps `releases/3.0.14` and ThirdParty-ASL `releases/2.1.0` from checksum-verified tarballs on a digest-pinned `ubuntu:24.04`, with netlib reference BLAS/LAPACK and `--without-metis` so MUMPS ordering matches IDAES's. HSL is never built; `probe_host` records `ma27/ma57/ma86/ma97` availability at runtime.

### Consequences

A profile naming `ma57` on a host without HSL fails with `capability.backend` unless it declares `fallback_linear_solver = mumps`, in which case the fallback is a selected policy recorded in `runs.resolved_options`.

### Compensating controls

`runtime.host_capabilities` persists the probe; `runs.resolved_options` records the options actually passed to `CreateIpoptProblem`, so a reproduction on a different host is either identical or visibly different.

### Confirmation

`python / parity` runs in the container and fails the pre-flight rather than skipping; the weekly rebuild-without-cache job compares library checksums.

## Pros and cons

Building from source costs a container image and a weekly job; using someone's binaries costs reproducibility and the clean-room story.

## More information

Blueprint §18.3 (in-process Ipopt), §6.13 (execution and evidence), §26 (residual risks); review finding F10; register rows R-08 and R-09.

## Status history

- 2026-09-13 — accepted with the repository-seeding pull request (backfilled from blueprint revision 3).
