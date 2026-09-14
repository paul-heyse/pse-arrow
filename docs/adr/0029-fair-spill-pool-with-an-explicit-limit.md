---
id: ADR-0029
title: Give every session a FairSpillPool sized from a declared memory limit
status: superseded
date: 2026-09-13
deciders: [paul-heyse]
level: decision
principles: [DM-30, DM-28, DM-43]
blueprint: [§14.3]
review: not-required: adopted in revision 3 from the DataFusion capability map's measured pool behaviour; no review finding
evidence: Interface-checked
supersedes: []
superseded-by: ADR-0046
revisit: A workload's spill rate makes the declared limit the binding constraint on compile time, or a deployment needs a different pool discipline
verification: `tests/engine` memory-exhaustion test (asserts a typed `ResourcesExhausted` naming the keys to change); the §24.3 compile benchmarks

---

# ADR-0029: Give every session a FairSpillPool sized from a declared memory limit

## Context

Blueprint §14.3 builds one DataFusion `SessionContext` per snapshot with a `FairSpillPool` sized from `datafusion.runtime.memory_limit`, never an `UnboundedMemoryPool`.

## Scope

Binds the memory pool, the disk manager configuration and the failure mode of memory exhaustion. Thread budgets are §18.8.

## Drivers

A bounded pool turns exhaustion into a typed, actionable error naming the keys to change; an unbounded one turns it into process death, which loses the run, the diagnostics and the partial outputs.

## Options

`UnboundedMemoryPool` — rejected: an OOM kill is not a failure mode the failure taxonomy (§23.2) can describe. `GreedyMemoryPool` — rejected: one operator can starve the rest of the plan.

## Outcome

`FairSpillPool` sized from a per-deployment `datafusion.runtime.memory_limit`, with an explicitly configured `DiskManager` directory, declared `spill_compression` and `max_spill_file_size_bytes`, and `target_partitions` validated against the rayon budget at construction.

### Consequences

Deployments must declare a memory limit; there is no "unlimited" setting, which is the point.

### Compensating controls

Exhaustion raises `ResourcesExhausted` naming the configuration keys; the pass record notes the failure class and partial outputs are discarded (§14.3).

### Confirmation

`tests/engine` drives a plan past the limit and asserts the typed error; the §24.3 benchmarks record spill behaviour under recorded conditions.

## Pros and cons

A limit that is too low fails runs that would have succeeded; a limit that does not exist fails them less legibly.

## More information

Blueprint §14.3 (pass engine, session), §18.8 (threading), §23.2 (failure taxonomy).

## Status history

- 2026-09-13 — accepted with the repository-seeding pull request (backfilled from blueprint revision 3).
- 2026-09-13 — superseded by ADR-0046.
