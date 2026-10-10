# Native inverse supplier semijoin inquiry

This bounded followup supports Plan 34 SI04. Current status and disposition belong to
[Plan 34](../../../plans/34-unified-state-management-and-persistence.md).

**Measured (2026-10-10):** released SurrealDB 3.3.0 on the existing managed current-v3
functional service, loopback administrative HTTP access, isolated registered disposable
databases, current generated table/index declarations, and concurrent repository builds.
The [48-query comparison](native-inverse-plan-comparison.json) captures exact statements,
individual server statement durations, full structured membership plans and returned keys.
The [reproducer](native-inverse-plan-probe.py) asserts equality of complete returned row
objects between the original correlated existence query, an inline IN subquery, and one
LET supplier set. This is a query-shape inquiry, not a scientific execution or complete
preparation benchmark. Timing noise and native intermediate memory remain material.

The 256- and 4,096-membership fixtures include current, inactive and future memberships,
rare/common source scopes, source kinds, target names and unrelated-problem suppliers.
Every query retains the protected transaction retention lock, exact pin problem/revision/
sequence checks, original 30-second RPC deadline, temporal membership predicates, selected
scope and kind predicates, ordered key cursor, 64-row response limit and final pin expiry
check. The original comparison used a 600-second disposable fixture pin to permit the
whole inquiry; it did not change production lease durations. The separate controls refresh
a 60-second fixture pin before each comparison. No production clocks were relaxed.

| 4,096-member query | Correlated membership statement | One supplier LET + membership statement |
|---|---:|---:|
| Rare target, unscoped | 10.422299 s | 2.024187 ms + 9.338240 ms |
| Rare target, rare source scope | 4.255964 ms | 2.006927 ms + 0.481483 ms |
| Common target, tail cursor, dataset kind | 549.798064 ms | 9.206634 ms + 2.324272 ms |
| Rare target, absent source kind | 16.955777 ms | 2.877529 ms + 27.969439 ms |

The rare unscoped correlated plan repeated target-index work over 2,809 eligible current
membership candidates, emitting 2,449,448 inner index rows. The inline IN subquery retained
that repeated work. A single LET evaluates the target supplier query once. Its rare-target
set contains 872 source-version values across both problems; the common-target set contains
4,352. Membership still uses the ordered membership access path: this change does not prove
promotion of the membership-version index. Explicit scope equality retains the previously
selected `(problem, scope, key)` access path.

**Tested:** `PSE_SURREAL_STATE=<serving-state> PSE_PROBE_OUTPUT=/tmp/inverse-controls.json
scripts/pse-env --resource-class light -- python3
docs/design_review/evidence/unified-state-management-2026-10-10/native-inverse-plan-probe.py
--controls-only`, native SurrealDB 3.3.0, zero failure baseline. The
[controls](native-inverse-plan-controls.json) compare complete row objects after adding
repeated target edges. Complete pages contain 64, 54 and 0 unique memberships; rare-scope,
absent-kind and absent-target results stay exact. Mismatched revision, expired pin and
released pin all refuse. Standalone supplier EXPLAIN plans show a single target-index scan;
512 duplicate supplier values still produce only 118 current dataset memberships. Both
comparison and control fixture databases were drained and removed through the registry.

The selected implementation materializes one native supplier set per targeted inventory
query, inside the existing protected transaction, then tests membership version IN that
set. Untargeted queries emit neither set nor filter. Duplicate source versions retain
existence semantics. Client page sizes, decoded ownership, complete-inventory premises and
lease checks remain unchanged. The managed native server memory allocation owns the
transient set; no arbitrary supplier cap, persisted view or fallback path is added.

Eager materialization can do extra work for an empty outer membership range or an absent
source kind, as the table demonstrates. It also includes unrelated-problem and historical
source versions before exact current membership qualification. Follow up at high target
degree and skew when that intermediate becomes a measured service-memory or latency cost;
compare complete paged operation equality and managed server memory, preserving all guards.
These observations support removing repeated inverse-edge work and do not establish a
universal speedup or a high-degree memory bound beyond the managed server allocation.

Reproduce the 48-query comparison with the same command, omit `--controls-only`, and select
a separate `PSE_PROBE_OUTPUT`. The script creates its own registered fixture and reclaims it
on successful completion; failed fixtures remain registered for diagnosis and guarded cleanup.
