---
title: "Plan 53f: Effect-scoped scheduling"
status: draft
date: 2026-10-23
adrs: []
review_sources: [docs/design_review/reviews/design_review_test-architecture-target_2026-10-23.md, docs/plan_improvement_reviews/test-architecture-remaining-2026-10-24.md]
scenario_sources: [docs/design_review/reviews/design_review_test-architecture-target_2026-10-23.md]
doc_role: plan
doc_owner: docs/plans/53-test-architecture-target.md
doc_topics: [current-work, design, validation, environment]
---

# Plan 53f: Effect-scoped scheduling

## Target and limits

**Proposed:** scheduling reflects actual effects and supported protection scope under
[R08](../design_review/reviews/design_review_test-architecture-target_2026-10-23.md#r08) and
[PI07](../plan_improvement_reviews/test-architecture-remaining-2026-10-24.md#PI07). C02's shared
requirement owner remains the semantic foundation; nextest/pytest and host/service owners
execute scheduling. The [coordinator](53-test-architecture-target.md#package-state-and-execution-route)
owns progress. Accepted RC02 removes demonstrated accidental barriers while retaining necessary
native/timing/canonical protection. No global capacity planner, default serial queue, automatic
concurrency increase or change to ordinary unbounded build mode is selected.

**Interface-checked:** `Requirement` currently owns effects, fixtures, capacity and roles;
exclusion/quiet meanings remain separately expressed in scheduler configuration. Canonical effects
or scientific capacity do not imply service-wide quiet. B05's two local Python cases are already
fixture-free; remove their residual scheduling coupling without redoing their delivered migration.
DTA-044 still does not establish that the remaining canonical barrier is unnecessary.

## Work packages

<a id="f01"></a>
### F01 — One requirement interpretation, including actual exclusion meanings

Extend the existing requirement owner only with the exclusion/quiet distinctions actual cases
need. Keep semantic demand separate from placement widths and actual service/generation acquisition.
Derive per-run nextest/pytest scheduling from those meanings rather than a second effect-policy
table. Agree the descriptor with E01 before its immutable requirement migration; no universal
capacity taxonomy or exact-name acceptance manifest is needed. Preserve C02's actual catalogue
correspondence and before-creation guard.

Remove broad requirements from demonstrably pure KINSOL profile-identity and Ipopt status/bounds
controls. Preserve native pools, MKL/heavy scientific and timing needs where real. Existing
[nextest groups/thread requirements](https://nexte.st/docs/configuration/test-groups/) are execution
mechanisms; human group displays are not stable machine metadata. Migrate nextest overrides,
root/Python conftest grouping and managed pytest selection together. B05's capability and
constructor/module-evidence cases shed the residual canonical group while remaining store-free.

**Acceptance:** actual selected requirements agree across affected/native/Python routes; renamed,
split or new cases preserve meanings or refuse exact stale/missing correspondence before effects.
Known pure cases lose unrelated reservations; genuine timing/native/canonical cases retain them.
Delete broad regex interpretations/route-specific policy tables only where the shared owner
replaces them. No benchmark is needed to establish removal of an accidental barrier.

<a id="f02"></a>
### F02 — Resolve the assertion before selecting service-quiet scope

Resolve inquiry I08 by inventorying actual claims, not just current group names. For example,
`fixture_connections_preserve_deployment_database` asserts database existence; that alone does
not require a private service per test. Choose database-local assertion/protection where that is
the intended claim. For genuine service-wide quiet, select one supervised invocation-owned
service per relevant invocation, including creation borrower, generation identity, actual member
association, shutdown/drain and retained-reader protection. Actual bindings remain with the
invocation/fixture owner, not a sealed semantic requirement or host placement declaration.

E03 supplies selective access discovery with known writer coverage. Missing index entries or a
per-run exclusive group cannot establish quiet across independent invocations. Preserve barriers
until the selected claim and lifecycle protection are implemented. A service per test is not the
default; narrowing a real scientific/lifecycle requirement merely to permit parallelism is invalid.

**Acceptance:** two separately started invocations exercise creation, retained access and
maintenance under the chosen service/database claim. Include one registering/retaining access
while another requests quiet/maintenance. Migrate service selection, assertion and cleanup
consumers together; delete replaced shared-service assumptions only after cross-run controls pass.
If neither isolation nor the justified narrower assertion supplies the supported claim, retain
conservative protection and an explicit unsupported outcome. This does not widen canonical
concurrency or reopen completed 52k delivery.

<a id="f03"></a>
### F03 — Explicit sufficient local settings from current role owners

Use smaller explicit bounded settings where the local fixture responsibility permits them,
retaining real inputs and receiving-budget refusals. Consume current role declarations in
`.config/agent-capacity.toml` and environment owners; do not copy historical numeric envelopes
into another scheduling policy. Functional slots, store slots, compiler limits, OS ceilings,
retained diagnostics and actual resident demand are distinct. A ceiling is not reservation or
measured consumption, and a compiler live-root admission limit is not an OS-memory problem.

**Acceptance:** targeted controls show the chosen role settings reach the receiver and suffice
for the same functional case, including meaningful budget/limit refusals. Preserve the current
ordinary unbounded build selection and timing isolation. Infer no OOM cause from historical
width-16 versus width-4 starts. Retain remaining canonical protection; any later throughput or
concurrency proposal requires its own supported demand/lifetime premises and, for numeric claims,
a declared measured baseline. Such a campaign is not required to close this package.

## Integration and completion

Delivered C02/B05 are foundations. F01's exclusion meaning precedes E01's immutable requirement
cutover; scheduler consumers can migrate when their actual prerequisite slice exists. F02's I08
outcome and implemented E03 coverage/protection precede any quiet-scope relaxation. F03 is
independent of blanket barrier removal. Coordinate nextest/conftest/service declarations with
the B/C/E consumers under one writer per shared surface.

53g qualifies the chosen two-invocation scope and retained native/timing/canonical cases. Less
exclusion establishes no measured speed/capacity claim. Preserve actual canonical protection and
its support boundary without adding an invented optimization backlog.
