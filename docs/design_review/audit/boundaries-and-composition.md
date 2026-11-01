# Boundaries and composition

**Interface-checked, 2026-11-01, source `19e7b49a67df24c1bf73005a6186e145e930d53a`.**
The [collection baseline and limits](README.md) apply throughout.

## <a id="da-020"></a>DA-020 — Product process ownership imports independent engineering-tool responsibilities

**AP-01, AP-02, AP-06, AP-07, DP-17. Revalidated, narrowed to a current dependency.**

`pse-runtime/Cargo.toml` unconditionally depends on `pse-host`.
`workflow/native_worker.rs` consumes the host's manager and service ownership to supervise
scientific workers. The same host library compilation unit unconditionally includes CLI,
source and API investigation, schema inspection, correctness orchestration and tool preparation
(`pse-host/src/lib.rs` and its manifest).

Product process supervision consequently depends on independently changing engineering
capabilities and their dependencies, even when its consumed ownership contract does not change.
This is a source/build dependency consequence; no rebuild-time magnitude was measured.
The host's optional `native-hir` feature limits part of the reach. A common cold bootstrap and
shared process owner are legitimate requirements; the finding does not prescribe a crate split
or deny sharing that owner.

The former Python-boundary footprint is gone. Historical foundation-placement examples not
reexamined here are not promoted into current evidence.

## <a id="da-053"></a>DA-053 — Analysis modes repeatedly assemble native routing and custody

**AP-01, AP-03, AP-06, DP-01, DP-08, DP-16. Revalidated.**

`workflow/shooting.rs` and `workflow/fitting/modeling.rs` independently compose structural
reservation, structural/oracle facts, `routing::Requirements` and context, route selection,
witness ownership and `math::solves::admit_profile`. Their scientific facts differ
legitimately, but the composition of the execution contract is substantially repeated.
For example, witness retention is reconciled against a reservation separately in each mode,
with different retained-byte arithmetic. A new common routing or custody input requires
coordinated changes in these composers.

Worker lanes also independently assemble channels, an actual owner thread, runtime entry,
constructor authority, original execution scope and panic settlement
(`workflow/shooting/window_team.rs` and `workflow/fitting/experiments.rs`).
The finding concerns repeated ordinary composition, not the count of `Requirements` literals.

Eligibility and route decisions themselves remain centralized in the native backend.
Ambient fallback in `pse-backend-native/src/execution.rs` and
`pse-math/src/construction.rs` deliberately serves direct library callers. No production route
omitting a required owner was established. Mode-specific scientific differences must not be
discarded merely to make the composers look alike.

## <a id="da-054"></a>DA-054 — Study placements repeat continuation composition

**AP-01, AP-03, AP-06, DP-01, DP-08; PS-08, PS-11. Revalidated, narrowed.**

Local `workflow/study_execution.rs::continuation_start` and durable
`workflow/study.rs::apply_study_prediction` separately compose task admission, deadline/scope,
execution and work-admission attachment, proposal, original screening and proposal-failure
fallback. A change to their common continuation semantics must be maintained in both.

Both placements already share `PreparedStudyOperation`, operation admission, scientific owners
and `StudyPreparationLane`. The older description that they share only policy predicates is
incorrect. Their schedulers also serve different failure domains: a local frontier with
in-flight dependents differs legitimately from distributed claims and leases. Two schedulers
alone do not establish this diagnosis.

Local execution retains a staged native owner across suitable successors; durable
`StudyCandidateLane` retains sources and preparation. This is an execution-fit question,
not proof that cross-process native reuse is required or feasible. No slowdown or broken
continuation behavior was measured.

## <a id="da-052"></a>DA-052 — Durable format and configuration projections need a concrete duplication case

**AP-04, AP-05, DP-01, DP-02, DP-03, DP-24. Prior diagnosis downgraded to an inquiry.**

`workflow/durable.rs` owns public `StoredCompletion` version 3, private `CompletionReceipt`
version 1, their constructors and `current_stored_completion`. These are distinct nested
format meanings with owner-local checks, not an established repeated consumer rule.
Canonical configuration readers in `pse-operations/src/canonical.rs`,
`canonical_study_module.rs` and `canonical_files.rs` separately parse portions of `config.json`.

A partial reader is not inherently a second authority: each owner may legitimately need only
its own fields, and numeric version tags are not themselves defects. This refresh did not
establish a cross-owner shared rule being independently interpreted. That specific case is
needed before retaining the older diagnosis. Earlier Python/native
producer and digest examples were not revalidated; they keep their 2026-10-29 evidence date
in Git rather than inheriting this revision's source claim.

## <a id="da-059"></a>DA-059 — Relation generation unconditionally emits the same projection families

**AP-01, AP-07, DP-15, DP-17; H3, H17. Revalidated structure, consequence bounded.**

`pse-codegen/src/codegen/rust/mod.rs` emits both relation and typed-Arrow projections for every
relation. `pse-schema/src/model/relation.rs::RelationSpec` has no per-family projection
selection. Adding a declaration therefore expands both surfaces regardless of its actual
consumer demand.

Typed Arrow reconstruction still has internal/test consumers, including
`workflow/connected_results.rs`, and `pse-engine/src/session/factory.rs` retains
`registry_relation`. No complete reachability proof establishes that a particular projection
can be removed. Uniform generation can also be a useful stable contract. The concern is a
declaration-wide generation policy without expressed demand, not generated code volume alone.

The previous explanation based on a live `pse-py` delivery path is obsolete. That crate is absent.
Historical module/line counts and cold-build cost claims are withdrawn.

## <a id="da-060"></a>DA-060 — Data-layer lifecycle remains an inquiry, not a proved invariant-enforcement gap

**AP-01, AP-07, DP-15, DP-24. Prior diagnosis downgraded.**

Source tracing located `run_invariants` in fixture generation
(`xtask/src/codegen/physical.rs`) and sampled other references in tests. It did not establish
that a supported canonical invariant is unenforced. Primary-key checks consume the registry
(`pse-relations/src/result_batch.rs`); multiple enforcement mechanisms are not automatically
multiple semantic authorities. Blueprint §4.6 deliberately assigns relational obligations to
their admission owners rather than an imagined global SQL stage.

The older broad conclusion that the catalogue falsely governs behavior is withdrawn.
Some engine mechanisms appear internal or test-only in the inspected source population, but
this is not a whole-program absence proof. Whether those mechanisms retain a current supported
consumer remains a bounded lifecycle question. It cannot justify deleting them without further
consumer and contract investigation.

## Earlier identities

Historical explanations are recoverable at source `19e7b49a6` and its Git history.
Older resolved/decision-closed entries were not comprehensively requalified.

| Identity | Disposition and lineage |
|---|---|
| <a id="da-015"></a>DA-015 | Historical Python projection premise retired; `pse-py` is now absent |
| <a id="da-019"></a>DA-019 | Retiring Python supervisor premise retired; native ownership is current |
| <a id="da-017"></a>DA-017 | Previously resolved: capability refinement and owner-decided selection |
| <a id="da-033"></a>DA-033 | Previously resolved: shared exact settlement |
| <a id="da-012"></a>DA-012 | Resolved 2026-10-25: one run supervisor |
| <a id="da-013"></a>DA-013 | Resolved 2026-10-25 for prediction policy; continuation composition remains DA-054 |
| <a id="da-014"></a>DA-014 | Closed by decision I22/RC12; not reopened |
| <a id="da-016"></a>DA-016 | Merged into DA-017 |
| <a id="da-018"></a>DA-018 | Closed by decision I23; not reopened |
