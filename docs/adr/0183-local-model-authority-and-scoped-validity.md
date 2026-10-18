---
id: ADR-0183
title: Own local model revisions and scope validation to actual admissions
status: proposed
date: 2026-10-13
deciders: [paul-heyse]
level: decision
principles: [AP-01, AP-02, AP-04, AP-07, DP-09, DP-19, PS-10]
blueprint: [§4.6, §14.2, §14.3, §14.4, §19, §21, §22.2]
review: git:6af92d1dee1ffa0b80dca71e03a63b489f2d940c:docs/design_review/reviews/design_review_plan38a-local-authority-and-scoped-validity_2026-10-13.md
evidence: Interface-checked
supersedes: []
superseded-by: null
revisit: A consumed cross-process pure memo, remote worker, or changed publication topology needs additional lifetime or trust guarantees beyond owned local computation and the current recording adapter.
verification: Plan38a local construction and disconnected computation, exact delta and branch controls, fresh-versus-rebound original evaluation, demand-upgrade ownership, scope-sensitive SQL validation and durable original-intent settlement controls; Plan38g owns assembled qualification.
---

# ADR-0183: Own local model revisions and scope validation to actual admissions

## Context

The operator accepted Plan38 RC01/02/08/09/11 and authorized execution of its local authoring/preparation/validity companion. Current immutable compiler products and actual validation owners are useful foundations, but canonical selection still gates pure computation, compiler lineage includes store placement, and stable SQL checks may retain a timestamp folded under an older validation owner.

## Scope

Amend the local-authority, preparation and validation clauses of ADR-0164/0178 and the Python composition contract. Their unrelated publication, imported artifact, native ABI, reader, retention and recovery guarantees remain. ADR statuses do not change outside the decision-PR route. Final storage topology and numerical method policy are not selected here.

## Drivers

An edit/re-solve should require only the model's actual dependencies. An owned immutable model remains usable after a publication disconnect; required durable effects still acknowledge exact committed outcomes. Independent heads and source receipts must not become interchangeable because mathematical content matches. Stable validation belongs to the actual admission invocation, while immutable checks can retain sound evidence.

## Options

Keeping canonical selection as local compute authority preserves unnecessary service and permission crossings. Removing only the constructor's connection call leaves the same dependency hidden in preparation. Select complete local admitted revision ownership plus an optional origin/recording adapter over existing persistence. Retain independently owned bounded Salsa lanes and detached products rather than treating cloned databases as snapshots. Extend the existing native validation owner with admission scopes rather than rebuilding sessions, reparsing unchanged SQL or weakening predicates.

## Outcome

The scientific composition root owns authored declarations/documents and their consumed data, exact physical/provider/interpretation/validation context, and allocation leases. A local receiver token, semantic content hash, compiler lineage and optional durable origin have distinct roles. The receiver token remains stable when acknowledged origin is attached. SourceDelta.parent names that receiver token; recording separately uses the actual canonical parent CAS. Current local-head edits advance a bounded lane; older-parent edits fork without rewinding it. Staged refusal/cancellation leaves the prior head usable.

Rust/Python use the same lower admission and preparation path. Runtime(settings, ephemeral=True) may omit substrate and never connects during construction. A supplied substrate permits explicit store access when consumed. Default durable construction requires a configured sink; source/attempt success retains required acknowledgement. Definitely refused and dispatched-but-unsettled effects stay distinct, with original identity, deadlines and ownership through settlement/drain. No offline-durable success protocol or fabricated canonical row is introduced. Caller close and store disconnect remain distinct. Explicit recording reads have an independent configured owner and do not silently upgrade ephemeral execution. Original source settlement is read-only; missing acknowledgement remains unproved. Python interruption preserves an actual source acknowledgement through drain, and transaction-phase refusal evidence cannot be inferred from a later receipt-read error.

One complete physical specialization produces demand-shaped preparation. Conservative incidence, validity/execution dependencies, objective coupling, form facts and operator programs have their own consumed identities. Compatible scalar optimization products may be retained under existing pressure/flight/allocation owners; only demanded derivative products and exclusive mutable workers grow. Fixed parameters, guards, original coordinates and independent original checkers remain authoritative.

An admission scope owns one opaque identity and captured query timestamp. Child admissions and premise-preserving transforms share it. SQL binds once under the actual engine/UDF/configuration owner; immutable predicates and evidence can reuse complete unchanged premises. Stable/ambient checks prepare and evaluate under the actual scope; independent scopes never share permission solely because timestamps match. Volatile CHECK rejection and unsupported-expression refusals remain. Checked batches carry scope-sensitive evidence and byte owners; demanded in-memory batches are populated lazily. New numeric states retain fresh scientific checks.

Local immutable data, imported pure memo and native executable artifacts have separate trust contracts. Content equality cannot transfer source/acquisition permission. Actual native imports retain ABI/environment/build-generation qualification; duplicate observation within one unchanged operation may be removed. Optional disk memo persistence is deferred until a named cold-start or independent-worker consumer requires it; raw Salsa ids never become durable keys.

### Plan45 consumed physical premises (accepted RC03 target)

The physical resolver issues a sealed, versioned per-product dependency projection from actual checking and lookup. It includes positive/transitive quantity, unit, conversion and reference-state meaning; exact lookup misses; complete candidate populations and ambiguity; package visibility; precondition/checker interpretation; and consumed provider specification, output and configuration. Complete inventory and canonical source identities retain their admission/interchange meanings. Current consumed-body/artifact/view frames change version when their preimages change; historical digests remain historical.

A same-lineage compiler lane may refresh physical context only under its existing exclusive writer. Stage the Salsa inputs and current context together, re-admit the current source/scope, and publish coherently or restore/destroy the candidate. Tracked pure products read resolver-issued semantic projections; equal results may backdate. New prepared wrappers retain current registry/checker/provider owners and receiving evidence while compatible immutable mathematics is shared. Source/invocation/schema/session/registration guards remain authoritative.

Unknown dependency classes use the complete immutable behavior context where one exists. A physical-inventory hash or a checker's lifetime immutability alone does not identify arbitrary behavior: where complete interpretation cannot be represented, recompute without retained reuse. No generic certificate system, untracked query input or previous receiver permission is introduced. This target is authorized, not newly runtime-qualified; Plan45a/A06 and its bounded prerequisite review own implementation and evidence.

### Consequences

Migrate all affected producers, Rust/Python consumers and generated declarations together, then remove replaced paths. Store imports retain protection until their complete consumed closure is locally owned. Bound live lanes, metadata, retained products and foreign heaps through existing policy and allocation owners; eviction cannot release borrowed products. Keep actual durable readers, independent writers and recovery protocols intact.

### Compensating controls

Exact parent/context, positive and absence dependency checks prevent stale reuse. Scope-sensitive validation is not a timeless registry fingerprint. Owned source and prepared product leases survive head changes/eviction; uncertain effects survive cancellation until authentic settlement. No new service, crate, topology, compatibility backend or whole-process Python loader observer follows from this decision.

### Confirmation

Interface-checked against current source and pinned library contracts during Plan38a execution planning. The existing audit supplies architectural rationale; the independent bounded target review accepted this amendment before dependent product edits. Targeted implementation evidence and remaining limitations belong to Plan38a; assembled qualification belongs to Plan38g. This record does not claim product acceptance or an ADR status transition.

## Pros and cons

Local ownership removes repeated canonical crossings and makes scientific responsibilities independently usable. The migration must explicitly separate semantic, receipt, invocation and effect identities; merely making storage fields optional would be insufficient.

## More information

[Plan38a](https://github.com/paul-heyse/pse-arrow/blob/6af92d1dee1ffa0b80dca71e03a63b489f2d940c/docs/plans/38a-authoring-preparation-and-validity.md) owns this execution scope; [Plan38](https://github.com/paul-heyse/pse-arrow/blob/6af92d1dee1ffa0b80dca71e03a63b489f2d940c/docs/plans/38-efficiency-and-solver-target.md) owns accepted decisions and finding disposition. ADR-0181 retains caller/activity/physical shutdown meaning. ADR-0182 retains independent scientific artifact/history obligations.

The [Plan45 prerequisite review](../design_review/reviews/design_review_plan45-physical-premises-and-fixed-pattern-klu_2026-10-18.md) assesses the consumed-identity amendment; [Plan45](../plans/45-pipeline-execution-optimization.md#reopened-follow-up-checkpoint-2026-10-18) owns current follow-up disposition.

## Status history

- 2026-10-13 — proposed before dependent implementation under the operator-confirmed Plan38 target; ADR status remains proposed pending its decision PR.
- 2026-10-18 — maintainer authorized Plan45 F05 follow-up under accepted RC03; install complete consumed physical premises and current-owner lane refresh before dependent implementation. Formal status remains proposed.
