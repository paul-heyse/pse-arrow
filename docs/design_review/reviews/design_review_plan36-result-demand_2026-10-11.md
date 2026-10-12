---
title: Plan 36 result demand, acknowledgments and bounded horizon target
date: 2026-10-11
tier: design
purpose: target
standard: core-3.4
profiles: [process-simulator-1.5]
evidence: Proposed
disposition_owner: docs/plans/36b-studies-coordination-and-results.md
---

# Plan 36 result-demand target review

**Accept the proposed target.** ADR-0180 and Plan 36b P-B4–P-B6 provide an adequate architectural direction for selective scientific results, acknowledged durable completion and bounded horizon execution. No additional material target finding or rule change is established.

This independent review accepts the design at **Proposed**, supported by **Interface-checked** source inspection. It does not accept implementation, establish numerical correctness or close the enclosing efficiency findings. The coordinator published the independent reviewer's judgment with proportional presentation.

## 1. Scope, standard and baseline

The standard is Core 3.4, its review template, Efficient Architecture Heuristics 1.0, Process Simulator 1.5 and the pse-arrow binding. The subject is ADR-0180 and Plan 36b P-B4–P-B6 before schema/consumer migration. Baseline: HEAD `2d6bf6ae7d03a8a9f6f109696bfe3d9cc8455466` plus concurrent working-tree changes inspected 2026-10-11. The narrow completion reader is already present; the broader migration remains proposed.

The functional target is useful, physically meaningful and diagnosable results for interactive edit/re-solve, studies, recycles and dynamics, with local extension and development. This review examines result transport, retention and publication. It does not reopen numerical algorithms, preparation, resource admission or the whole efficiency review.

Inspected owners: blueprint §19.2, §19.3, §20.6 and §21; registry `pse-schema/src/catalog/{native_math,substrate}.rs`; Runtime projection/export/readers, durable publication, study/report consumers, staged execution, horizon state and telemetry; generated-query owner `pse-codegen/src/codegen/surreal_execution.rs`. Stale utilization/capability catalogs were excluded.

## 2. Responsibilities and governing contracts

| Responsibility | Authority and consumed contract |
|---|---|
| Scientific completion and permission | Existing assessment owners determine usability, diagnostics and seed eligibility; demand cannot weaken assessment. |
| Requested product extent | Typed demand identifies compact summaries, selected relations/members or explicit full reports. Missing detail remains unavailable. |
| Durable publication | The publication owner accumulates actual acknowledgments; canonical close/seal admits exact frozen coverage under current authority. |
| Live execution state | Horizon roles retain continuation products and rolling windows. Sampled telemetry has distinct loss/lifetime semantics. |

Registry locators map to original rows rather than a second writable scientific representation. Actual keys include solve variables `(run_id, step, symbol_id)`, constraints `(run_id, step, row_id)` and simulation samples `(run_id, sample, symbol_id)`. Owner/partition access selectors cannot substitute for those keys.

Demand selects extent; append acknowledges immutable effects; close freezes demanded membership and emptiness; seal establishes terminal authority; protected read checks current permission; role advancement consumes assessed, permitted products. These distinctions govern operations rather than merely naming result-shaped objects.

## 3. Physical meaning and validity

| Subject | Authority | Preservation obligation |
|---|---|---|
| Scalar rows | Original columns, quantity/unit identity, model context and qualification | Exact bits, units, basis/reference conventions, quality and missingness; filtering never replaces values. |
| Trajectories | Original sample/time/symbol coordinates and stored ordering | Shared blocks retain exact per-symbol ranges/time bounds; unavailable tails stay unavailable. |
| Derived quantities | Scientific qualification, including dual/uncertainty conditions | Materialization cannot invent eligibility or remove required assessment. |
| Starts/seeds | Original provenance, permission and compatibility | Compact reports cannot erase consumed starts or promote private prefixes to reusable solutions. |

No physical conversion, formulation, derivative or solve stage changes. Guards, variable roles, structural admission, scaling, solver class and independent assessment remain upstream obligations. Whole-model well-posedness and numerical fidelity are outside this review's evidence.

## 4. Revealing scenarios

| Scenario | Required target behavior |
|---|---|
| S01 Compact study with continuation | Retain mandatory occurrence facts and exact continuation product; release unselected reports. Full consumers explicitly request full demand. |
| S02 Empty relation followed by interruption elsewhere | Empty membership requires successful explicit closure; absent append/interrupted cursor cannot imply emptiness. |
| S03 Lost append/close/seal response | Settle the original immutable request and recovered authority under the original clock; accumulation advances once. |
| S04 Long horizon with failed role steps | Preserve older accepted seed per role, current estimator/candidate, advanced background product/factor and rolling window. One attempt closes coherently. |
| S05 Saturated telemetry | Ordinary sampling/coalescing records exact loss without blocking/cancelling science. Strict interruption is an explicit policy. |
| S06 New result field or packing mechanism | Registry/result owners define identity/support; original-column filtering keeps packing changes local to transport. |

Edit/re-solve and recycles preserve start, diagnostic and candidate facts. Bounded horizon state cannot remove general staged historical access. Dynamic sensitivity/checkpoint demand remains with actual numerical consumers.

## 5. Mechanisms, execution fit and alternatives

**Interface-checked:** current projection constructs multiple cells per scalar row; metadata/window sizes depend on field count and emptiness requires schema-bearing appends. `DurableAttempt::store_tables` visits all advertised relations. `Operations::settle` reconciles fully, seals and reopens. Ephemeral studies retain full `RunResult` objects. Horizon `Loop` retains requests/results/reports/measurements/inputs; `Staged::record` also clones values and starts. `Tap::observe` can wait for producer capacity or cancel on overflow. These observations support structural remedies without a measured speed claim.

One locator per row removes field-multiplied construction. Payload-shaped packing and bounded batching retain individual identity/receipt bounds. Explicit empty closure avoids payloads used only to represent membership. The accumulator needs ordered per-set digest/count state, demand membership and bounded outstanding gaps rather than all payloads or historical acknowledgment objects. Server admission must compare every advertised set and demanded coverage to frozen canonical state. Actual recovered close authority and seal acknowledgment replace normal rediscovery; uncertainty still reconstructs and compares.

Required original evidence may grow durably with a horizon; live execution need not retain every historical report/seed. Explicit full materialization remains admitted. Keeping per-field cells repeats work; global sorting changes unrelated order; typed block bounds require a populated ordering/range premise absent from Float64 time bounds; retaining full reports defeats compact demand; lossless ordinary telemetry couples observation to science.

Arrow supplies selection while domain owners retain demand, identity and permission. Pinned Arrow 59.3.0 filter source and the DataFusion skill's selection briefs support original-batch filtering with schema/source order retained. Context7 was a discovery lead, not evidence for the exact pinned contract. No universal scheduler, solver mechanism or general query product is required.

## 6. Foundations and gates

These verdicts concern the proposed target.

| Foundation | Verdict and basis |
|---|---|
| AP-01 | Satisfied: assessment, demand, publication, live state and telemetry have distinct owners. |
| AP-02 | Satisfied: representation preserves original meaning; demand/unsupported access are explicit. |
| AP-03 | Satisfied: studies/horizons compose shared capabilities without duplicating scientific policy. |
| AP-04 | Satisfied: demand, coverage, acknowledgments and role advancement govern operations; indexes remain derived. |
| AP-05 | Satisfied: empty/unrequested/incomplete, authority, permission and loss remain distinct. |
| AP-06 | Satisfied: bounded mechanisms admit direct controls; canonical races retain real integration. |
| AP-07 | Satisfied: removes demonstrated transport/rediscovery/live-history work while preserving necessary recovery. |

| Gate | Verdict and basis |
|---|---|
| G1 | Pass: original rows/completion retain authority. |
| G2 | Pass: keys, columns/order, physical meaning and absence states survive. |
| G3 | Pass: unsupported fields, invalid associations, gaps and expired/retired permission refuse. |
| G4 | Pass: selection does not rerun science; telemetry policy is explicit. |
| G5 | Pass: one attempt, frozen coverage, actual authority and original-request settlement prevent false completion. |
| G6 | Pass: original evidence and eligible continuation products survive transformations. |
| G7 | Pass: unavailable/partial output stays explicit; benefit is unmeasured. |
| G8 | Pass: library selection/transport stays distinct from domain authority. |
| G9 | Pass: applicable foundations satisfied for the bounded target. |
| PS-G1 | Pass, preservation scope: original physical context/columns and assessment remain unchanged. |
| PS-G2 | Not applicable to changed mechanism: no formulation, role or solver-admission change. |
| PS-G3 | Pass, preservation scope: demand/telemetry cannot establish convergence or seed permission. |

## 7. Findings and preservation constraints

**No additional material target finding.** Existing indexes, discarded acknowledgments, retained reports and overflow cancellation are already P-B4–P-B6 migration subjects. Their presence prevents implementation acceptance, not proposed-target acceptance. Preserve complete server coverage, recovered close authority, digest-acknowledged original completion, inspected heartbeat join, older per-role seeds and telemetry/science separation.

## 8. Relationships and prerequisite order

Demand and empty coverage precede the changed acknowledgment completion path; both precede coherent horizon streaming. Ordinary telemetry can migrate independently. Slim completion reads remain a bounded independent slice. Exact conflict/retirement and original-clock contracts are preservation prerequisites rather than replacements for these obligations.

## 9. Priority and follow-up

Proceed with coordinated declarations and consumers. Do not replace successful-path reconstruction before actual acknowledgment accumulation and server admission exist. Remove replaced cells and schema-only empty appends after consumer migration, preserving uncertainty recovery. No production cutover/deletion is selected.

## 10. Evidence limits and verification

Source/documents were inspected. Builds/tests/benchmarks/services/production operations were **not_run**. No formulation or scientific reference comparison was exercised.

Plan 36b controls retain exact selection/IEEE/null edges; compact/full consumers; empty versus interrupted export; missing/extra/gapped coverage; duplicate/lost acknowledgment; recovered authority/heartbeat failure; retired/expired reads; failed-role horizons/private-prefix interruption; and saturated native/async telemetry. They must exercise migrated declarations, emitted transactions and consumers. Compilation or existing fixtures cannot substitute.

## 11. Rule impacts and disposition

**Additional rule impacts: none.** Consumes authorized RC08–RC11 and proposed ADR-0180 amendments. No additional exception, production cutover or deletion authority. Plan 36b owns disposition and acceptance; no new finding ledger is introduced.

## 12. Decision

Behavioral/semantic adequacy: **Accept at proposed-design level**. Architectural fitness: **Accept at proposed-design level**. Overall: **Accept the bounded ADR-0180/P-B4–P-B6 target**. Implementation, scientific and performance acceptance remain open.
