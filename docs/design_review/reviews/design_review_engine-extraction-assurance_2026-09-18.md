---
title: Engine extraction and execution assurance
date: 2026-09-18
status: proposed
scope: Plan 10 N06
---

## 1. Decision and scope

**Proposed:** separate generic native execution from durable catalog composition and
provide contract-oriented execution assurance. N07 operation redesign, N13 allocator
replacement and N18 integration remain outside this implementation. Review uses current
factory/session/cache callers, the three library skills and seven isolated library
characterization checks; it does not certify end-to-end simulator or storage behavior.

## 2. Authority and lifecycle

Engine owns actual native assembly, bound inputs, policies and execution. Catalog owns
selected publications, durable identities, snapshot freshness and leases. Runtime composes
deployment resources through the shared engine constructor. Testkit owns local evidence capture and reusable checks, with no
production dependency back into testkit. Evidence never establishes publication authority.

## 3. Contracts

Preserve actual caller implementations, bounded resources and source guards. Keep
semantic/operational/inspection settings as projections of one effective resolution.
Only successful stream exhaustion establishes completion. Error, cancellation, early
drop and incomplete capture remain distinct. Validation and freshness are independent
of observational settings. Native metrics retain their meaning and partition labels.

## 4. Derivation and boundaries

Finalize native assembly before observational wrapping; install execution instrumentation
last. Catalog injects Delta planners and actual retained owners without a reverse engine
dependency. Native object-store registration supplies the same instrumented store to
DataFusion and Delta. A task tracer captures span and dispatch before scheduling.

## 5. Representative changes and failures

A custom UDF/planner remains the actual implementation after instrumentation. A changed
provider policy resolves once and is read back. Dropping a stream closes a span but
records abandonment. An undeclared trace metric remains available through native metrics.
A full collector marks evidence incomplete instead of claiming the operation was checked.

## 6. Gates

These are design verdicts, not implementation acceptance.

| Gate | Verdict | Evidence / required implementation |
|---|---|---|
| G1 authority | Pass | Native state and registry authoritative; evidence is observational |
| G2 fidelity | Pass | Actual implementations and source owners retained; negative unit checks |
| G3 validity | Pass | Existing admission remains independent of capture |
| G4 hidden behavior | Pass | Typed terminal status and native plan/metric inspection |
| G5 consistency | Pass, scoped | Durable reconciliation unchanged; no durability claim from spans |
| G6 reuse | Pass | Observation excluded from semantic identity; actual generations retained |
| G7 capability | Pass, scoped | Pin-specific probes; incomplete evidence cannot pass assurance |

## 7. Findings

| Finding | Principles | Evidence | Consequence | Correction | Verification |
|---|---|---|---|---|---|
| Span close does not mean success | DM-08, DM-28, DM-47 | Probe complete/drop/error | False assurance | Owned-stream terminal status | Three outcomes plus cancellation |
| Custom metrics need declared trace fields | DM-42, DM-50 | Probe complete/declared | Silent missing evidence | Native MetricsSet authority | Missing field counterexample |
| Blocking context propagation loses parent | DM-30, DM-47 | Probe tasks/tasks-fixed | Detached evidence | Native JoinSetTracer capturing dispatch/span | Async/blocking ancestry |
| Generic catalog services couple algorithms to storage | DM-02, DM-43, DM-56 | SnapshotSession and AlgorithmContext fields | Independent unit assembly impossible | Engine/catalog composition | Cargo graph and in-memory units |

Applicability: authority, lifecycle, provider boundaries, reuse and truthful evidence
apply. Scientific method accuracy and distributed publication qualification are not
claims of this slice.

## 8. Alternatives

| Alternative | Tradeoff | Decision |
|---|---|---|
| Keep catalog ownership and per-test plumbing | Duplicate assembly and storage coupling | Reject |
| Engine plus native tracing and thin contract checks | Focused shared boundaries, retains semantic checks | Select |
| Only move files and print native plans | Simpler but leaves ownership and completion ambiguity | Reject |

## 9. Verification

**Tested (library characterization only):** DataFusion 55.1.0, Arrow 59.3.0,
datafusion-tracing 55.0.0, rustc 1.98.1, release/offline; seven checks, zero failures
against baseline zero. The [probe capsule](../evidence/native-engine-assurance-probe-2026-09-18.json) retains exact source/lock bytes, hashes and scenario captures.
Implementation uses selected force-validate units, compilation, family and static DAG
checks. No integration or benchmark runs before Plan 10 N17 closes.

## 10. Limitations

The tracing crate attempts to install its own task tracer once; the already-installed
correct tracer produces one expected AlreadySet warning. An unknown existing hook cannot
qualify as complete assurance. No OTLP service, universal assertion DSL or percentage
claim of functional correctness is introduced.

## 11. Decision

| Decision | Condition | Completion evidence |
|---|---|---|
| Accept scoped design | N06 plus assurance; later plan items stay open | Direct caller migration, old-path deletion, dependency checks and isolated units |

## Updated library guidance applied

The expanded tracing skill was reviewed during implementation: execution, lifecycle,
metrics, context, filtering and storage briefs. It confirms that EOF behavior depends
on the native adapter, close events alone cannot classify termination, metrics may be
cumulative across reused plans, and store method completion does not prove payload
consumption. The [assurance guide](../../dev/native-execution-assurance.md) states these
limits and distinguishes local capture from export and functional correctness.
