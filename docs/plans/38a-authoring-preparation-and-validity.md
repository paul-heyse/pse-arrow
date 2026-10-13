---
title: Local authoring, preparation and validity
status: in-progress
date: 2026-10-13
adrs: [ADR-0183]
review_sources: [docs/design_review/reviews/design_review_efficiency-and-solver-target-audit_2026-10-12.md]
doc_owner: docs/plans/38-efficiency-and-solver-target.md
doc_topics: [architecture, preparation, validation]
---

# Plan 38a: Local authoring, preparation and validity

## Purpose and implemented foundations

This companion owns the scientific input consumed by [Plan 38](38-efficiency-and-solver-target.md): locally admitted immutable models, exact edits/lineages, pure preparation and carried validity. It implements corrected F01/F05/F06/F14 and surviving Plan 36 A/preparation/validation obligations. [Audit state evidence](../design_review/evidence/efficiency-and-solver-target-audit-2026-10-12/state-reuse-and-boundaries.md) supplies the baseline; the coordinator owns accepted rule decisions and finding disposition.

At `6c368c88acda548d421b44e8fc4f457c95bcec7e`, granular compiler queries, owned PreparedCase products, detached historical lanes, bound_case value rebinding and actual LocalPlanner/PreparedLocalContract validation already exist. `math/modeling.rs::CompilerLineage` and LaneStamp nevertheless carry issuing-store/canonical sequencing, while NativeRuntime construction opens canonical state even for Ephemeral. The target needs a real local semantic lineage and composition boundary, not just a removed connect call. `pse-admission` and generator capability splits already exist; 38f owns remaining compile/consumer acceptance rather than another extraction.

Contracts/rationale currently live in blueprint §14.2–§14.4, §19.1, §22.2 and their named ADRs. RC01/02/08/09/11 are accepted target boundaries; their concrete governing amendments precede dependent implementation via G0. Source claims are **Interface-checked**; target and controls are **Proposed**, with no new product execution evidence.

## Packets and dependencies

| Packet | Consumes | Delivered behavior and deletion | State |
|---|---|---|---|
| <a id="a1"></a>A1 | RC01/02, actual admitted documents/context and current lower owner | Local model/runtime composition and typed point ingress; remove canonical rehydration as the scientific compute authority after all callers move | Planned |
| <a id="a2"></a>A2 | A1 local identity; granular compiler/edit owners | Exact structural delta/reload and bounded independent lineages; retire duplicate resolution/reparse paths only when fully replaced | Planned |
| <a id="a3"></a>A3 | Complete admitted input/dependencies; A2 for across-edit reuse | One demand-shaped preparation with consumed pattern/form products, compact callbacks and lazy derivative workers | Planned |
| <a id="a4"></a>A4 | RC09/11; A1–A3 lifetime products, E2 for durable effects | Held immutable bundle/content/receipt/trust boundaries; remove repeated same-operation proof without deleting import/native admission | Planned |
| <a id="a5"></a>A5 | Existing actual bound validation owner; RC08 | Carry premise-preserving admission and reuse compiled contracts/batches; delete weak reconstruction/repeated unchanged checks | Planned |

A1 point binding, A5 retained validation and some A3 demand corrections can proceed independently. A2 supplies across-edit dependencies; it is not prerequisite to every local solve. E2 consumes receipts/recording products but a new catalog is not a prerequisite here. C/D own method policy and numerical-state acceptance; A never interprets solver success.

## A1 — Local authority and exact numerical binding

The composition root accepts owned authored relations/documents, exact physical/provider/interpretation/native-validation owners, limits and cancellation. It produces a locally admitted immutable model whose semantic identity excludes incidental endpoint/database placement. Durable origin remains attribution for actual import/publication effects. A local head and published head are separate: two independent writers can branch without silently overwriting one another at publication. Rust/Python loading, declared execution, case resolution, inspection, fitting/study setup and pure conformance consume the same lower admission/preparation operations. Durable construction adds a recording sink, preserving the coordinator's default automatic acknowledgement/visibility contract.

Keep parameter, fixed-coordinate value and free-coordinate start operations distinct. Bind exact independent coordinates through PointOverlay/AdmittedBinding, with duplicate/type/unit/role checks, parent/context association and result provenance. Value changes refresh value-dependent domain/bounds/class/presolve/provider facts; they do not rebuild unchanged body/incidence programs. Fixed/free membership, shape and structural literals take their actual structural route. A compatible numerical bound may update native values, but C1 owns classification-sensitive layout compatibility.

Pure computation can run from an imported local snapshot after publication disconnects. A durable edit/attempt must still fail or expose an honest recording state when its required commit is unavailable; local authority does not mean falsely acknowledging durability. E2 owns that effect boundary. Refusal/cancellation publishes no partial local head. Existing immutable products remain charged while a solve/study/stream borrows them after the head changes or a cache evicts them.

**Targeted acceptance:** fresh versus rebound original outputs and exact lineage; value versus fixed/free/bound changes; foreign same-named provider/UDF/context; independent local heads with publication conflict; disconnected local compute versus unavailable durable acknowledgement; cancellation leaves previous head usable; actual direct owner test needs no unrelated service/native setup. Delete the old store-required preparation callers only after all affected consumers use this contract.

## A2 — Complete edit invalidation and bounded lineages

Consume exact parent plus ordered add/change/remove and record/data ownership intent; omission is not deletion. Resolve changed expression references even when names/parents/kinds/imports stay unchanged. Cover absent/shadowed/imported lookups, namespace membership, duplicates, cycles and component merge/split. Preserve selective data hydration: complete parent declaration/metadata processing does not justify opening all inherited payloads. Retain selected identity framing and complete positive/absence dependencies; a faster fingerprint cannot replace them. Reuse parsed immutable text/span products independently of contextual binding.

The preferred implementation extends current independently owned mutable compiler lanes per local lineage/context. Freeze detached owned products for studies/historical reads. Salsa cloned handles share mutable storage and are not historical snapshots. Exclusive mutation stages complete inputs, cancels/drains relevant readers as required, then exposes an admitted head; failure restores its logical predecessor or rotates from owned admitted input. Bound live lineages, input/key/dependency metadata and foreign heaps separately from query LRU. Database/generation-local ids never become durable content keys.

**Targeted acceptance:** x→a versus x→b references with unchanged superficial metadata; add/delete/absence/shadow/import/cycle; selective inherited data; exact no-op; A/B/A and historical branch without rewinding current head; cancellation/refusal during staged mutation; pressure rotation while borrowed old products remain valid/charged; independent fresh-checker comparison of actually consumed changed/unchanged queries. Tests demonstrate reuse correctness rather than mirroring the tracked key implementation.

## A3 — One complete, demand-shaped preparation

Resolve declared fixture/physical specialization once into the complete executable preparation consumed by solve, initialization, inspection and conformance. Do not guess raw-document demand and then independently specialize again. Preserve bounded candidate windows/current eligibility where a persisted product is actually imported, with complete receipt/dependency equality; local product reuse does not inherit old durable permission.

Own distinct products for conservative incidence, execution/validity dependencies, objective/inequality coupling, form/body facts and selected derivative/output programs. Pattern identity includes selected rows/free roles and operator class; DAE/Jacobian/KKT are not interchangeable. An incidence-preserving body edit can invalidate affinity/solvability/guards/programs; a coefficient becoming zero changes numerical suitability even if the conservative pattern survives. Existing structural plans and per-block initialization remain producers to extend. Export orderings only to accepting library/index spaces; library-internal BTF/symbolic analysis is not automatically duplicate work.

Retain compact alias/axis/guard/applicability maps and atomic callback output publication. Optimize the scalar program once where the public Symbolica API permits; derive only requested orders/actions and grow charged workers lazily. Value calls must not build Second-order Hessians or every result family. Demand upgrades preserve complete order identity and original guards/Taylor semantics. Scratch transfer requires exclusive drained/reset ownership; independent original checkers remain independent. Indexed immutable request/reachability/policy lookup preserves source/physical/precondition/attribution meaning.

**Targeted acceptance:** empty/already-covered/nonempty/converted fixture demand and unknown-path refusal; actual preparation/query/program construction counts; body edit versus reusable pattern; objective crossing prevents unsound independent blocks; compact callbacks match independent original evaluations under malformed input/domain refusal; Value→First→Second upgrades, pressure and simultaneous worker exclusion. Count removed work when useful; no universal speed claim follows.

## A4 — Content, effects, trust and held product lifetime

Pure semantic recomputation, in-flight coalescing, byte retention, executable compilation and durable trust are different responsibilities. Retain one complete receipt/encoded allocation rather than copies per body. Content deduplication may share payload but never source/acquisition/occurrence permission. A held bundle/engagement borrows exact physical/provider/registry/validation/native-generation owners, original operation identity and clocks through dispatched effects and drain. E2 supplies durable settlement; a recorded exact acknowledgement can settle its old effect without minting a fresh live root.

Scope proof by artifact: local owned immutable data; foreign persisted pure memo; native executable/build generation. Digests establish correspondence to bytes, not semantic producer trust or ABI/environment equivalence. Requalify changed/unknown receiver, executable, native generation, nonce, placement or interpretation premises. Remove duplicate same-operation observation first. Native compilation remains an effect outside pure tracked queries. Optional disk persistence needs a consumed cold-start/worker contract, complete keys/version/load validation and retained-byte justification; raw Salsa ids never serialize as semantic identity.

**Targeted acceptance:** equal body/different receipt cannot inherit selection; acquired/absence/context/collision negatives; held allocations survive cache eviction and terminate after actual borrowers; late native load/configuration or changed executable refuses stale proof; pure memos and executable artifacts take distinct validation; failure/cancellation retains dispatched-effect ownership until settlement. No stat-only mutation proof or process-wide Python loader observation is invented.

## A5 — Actual validation owner and demanded checked batches

Extend existing LocalPlanner/ValidationContext prepared physical contracts and retain the actual BoundModel owner. SQL checks require real parse/coercion/NULL/function/configuration/volatility context; typed local lowering does not parse arbitrary SQL. Avoid weak-map reconstruction, cycles or sharing query/time-dependent settings as a timeless registry fingerprint. Decode validity and physical semantics remain separate from numerical result assessment.

Carry FieldCheckedBatch and allocation ownership through unchanged arrays and supported premise-preserving transforms. Delay diagnostic coordinate/path material until failures while preserving exact masks/counts/errors. Safe Arrow types alone are not every structural premise. Demand-populated result-batch retention reuses current C-stream sharing/cursors instead of eagerly materializing all outputs. E3/E4 own durable snapshot/read semantics and F1 owns Python pull/drain. New numeric states always receive fresh scientific checks.

**Targeted acceptance:** unchanged wrappers do not replan/revalidate; same-name foreign UDF/new schema/NULL/configuration requires actual context; nested/list/tagged/malformed input retains error equivalence; diagnostic paths exact; repeated demanded table reuses construction while untouched families stay absent; allocation survives stream/reader drop according to actual borrower. Decoder/cross-process trust checks cannot be skipped on version+digest alone.

## Bounded investigations and acceptance boundary

| Investigation | Settling evidence/outcomes | Dependent work |
|---|---|---|
| I-A1 — Local lineage versus durable origin | Inspect actual CompilerLineage/LaneStamp/consumer equality and complete provider/absence/head semantics. Select bounded local lanes or immutable revision inputs with ownership; no endpoint-only identity. | A1/A2 |
| I-A2 — Pure memo and native trust | Actual cold-start/worker consumer, exact serialization/build keys and artifact ownership. Select suitable pure memo, retain rebuild, or defer on explicit demand trigger; native proof remains distinct. | A4/imports |
| I-A3 — Remaining SQL binding/context | Actual immutable function/configuration/time/catalog premises at conversions; extend direct lowering where complete or retain one bound engine context. | A5/F2 |
| I-A4 — Evaluator parameterization/optimization | Exact pinned public evaluator API, guard/parameter body classes and deterministic optimization scope. Select supported reusable products or retain current path with named limits; JIT/SIMD belongs I-C2. | A3/D4/D5 |

Source/scenarios can settle ownership; discretionary probes settle decision-relevant library ambiguity. Library questions use capability skills, Context7 and exact pinned sources; stale capability maps/utilization journal are excluded. Investigation completion requires an explicit outcome/dependents, not an optimization. Targeted compile/tests accompany changes; assembled scientific/native/Python and scope-end checks belong to 38g after functional scope. This companion claims no product acceptance from authoring.

## Execution checkpoint — 2026-10-13

The maintainer authorized the detailed execution approach. Explicit local construction uses
`Runtime(settings, ephemeral=True)` without a required substrate. Required durable calls
remain fail-closed; unknown dispatched effects retain original-intent settlement. G0 installs
proposed ADR-0183 and architecture revision160 before dependent product edits. Independent
bounded target review precedes those edits; assembled qualification remains with38g.

I-A1 selects independently owned bounded local lanes with detached historical products.
I-A2 retains charged in-memory products and defers optional disk memo persistence until a
named cold-start/independent-worker consumer needs it. I-A3 retains actual SQL context with
one admission scope per operation, preserves volatile rejection and refreshes Stable/ambient
checks across scopes. I-A4 extends pinned scalar reuse and explicit demand compilation;
existing ceiling-based callers keep their actually consumed orders. No pin or topology change.

Runtime/source migration, evaluator retention and validation scope work have isolated owners.
The coordinator owns shared interfaces, local compiler-lineage integration, preparation and
engagement migration, Python declarations/generation, final consumer reconciliation and status.
