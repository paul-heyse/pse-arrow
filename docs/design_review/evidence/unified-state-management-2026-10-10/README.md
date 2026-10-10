# Unified state management execution evidence

These bounded native access-plan probes support Plan 34 SI04. They do not qualify
scientific execution, protected transaction conflicts, or the complete Plan 28 campaign.

**Measured (2026-10-10):** released SurrealDB 3.3.0, loopback HTTP administrative
query access to isolated registered disposable databases on the existing v2 functional
service. Only the investigated table/field/index declarations were copied from current
generated schema. Source baseline `a7a0d55e28aa` plus the Plan 34 dirty implementation.
The query bodies preserve production key/sequence, negative-name, inverse-edge and rooted
product predicates; fixtures contain 256 and 4,096 memberships with a single rare scope,
kind, inverse target and request. Concurrent Rust builds and repository work were active.
Timings are individual server-reported query durations, not a comparable end-to-end
preparation benchmark. Structured plans report rows at each iterator, including examined
index rows before filtering. No minimum speedup is an acceptance gate.

The [baseline](native-access-plan-baseline.json) and
[followup](native-access-plan-followup.json) retain full emitted plans and metrics.
The [probe](native-access-plan-probe.py) reproduces the populated queries and owns its
registered fixture cleanup. Run from the repository root through the light workload
route with an explicitly selected serving `PSE_SURREAL_STATE`; no credentials are printed.
All probe databases, including early setup failures, were removed through the registry.

| 4,096-member query | Examined index rows | Returned | Decision |
|---|---:|---:|---|
| Unscoped first membership page | 768 | 64 | Preserve the ordered page path. |
| Optional rare-scope predicate | 4,096 | 1 | Replace the optional guard for scoped calls. |
| Explicit rare-scope predicate and `(problem, scope, key)` index | 1 | 1 | Integrate at the existing inventory/query owner. |
| Absent exact name | 0 | 0 | Existing selection index fits. |
| Product request present / absent | 4,096 | 1 / 0 | Extra binary-request index did not improve the native plan. |

The direct scope comparison used the same fresh index: optional predicate 7.256194 ms,
explicit predicate 394.286 microseconds. This is a bounded access-path observation;
transaction/decode/complete-operation benefit remains separate. The candidate product
index `(problem, producer, request, key)` still used only the `(problem, producer)` prefix
under both overwritten and fresh index names. Do not add an unused index or change durable
request framing on this evidence. Revisit binary request pushdown when the pinned native
planner changes, at the canonical product-discovery owner, using the same skew and absent
request controls. Kind and inverse-edge selection still inspect the ordered membership
candidate range. The [inverse supplier inquiry](native-inverse-semijoin.md) eliminates
repeated edge work with a query-local supplier set, preserving exact protected membership
and bounded client pages. No new persisted view or graph traversal contract is selected.
