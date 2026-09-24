---
id: ADR-0078
title: Compile P7 through P10 as one typed semantic region
status: proposed
date: 2026-09-23
deciders: [paul-heyse]
level: decision
principles: [DM-01, DM-02, DM-13, DM-28, DM-31, DM-43, DM-59, DM-60]
blueprint: [§7.4, §14.3]
review: docs/design_review/reviews/design_review_rust-computation-target-design_2026-09-23.md
evidence: Proposed
supersedes: []
superseded-by: null
revisit: A supported relational query needs a full intermediate mathematical graph or a stage cannot retain its actual source evidence without serialization.
verification: Plan 13 W09 typed stage and projection units; just unit-rust-computation; just codegen-contracts-check; just adr-lint.
---

# ADR-0078: Compile P7 through P10 as one typed semantic region

## Context

Plan 13 W09 replaces mandatory whole-graph Arrow serialization between realization,
law expansion, selected-method binding and physical canonicalization. Individual native
algorithm ports currently force that representation even when the next consumer needs
the same owned typed graph.

## Scope

Within ADR-0076, declare one mathematical compilation region with four registered
endpoints: MathRealization, MathExpansion, MathBinding and MathCompilation. Each has
the union of its external inputs and final outputs. P7–P10 retain their semantic responsibilities and
local refusals. Typed graph stages carry source correspondence, equations, bindings,
selections and roots; relational segments continue to use DataFusion. The final
projection establishes the registered output obligations before atomic completion.

## Drivers

Avoid repeated decoding and graph reconstruction while retaining exact input ownership,
complete source evidence, deterministic publication and explicit resource admission.

## Options

Independent Arrow-only pass operators retain the unwanted round trips. A global graph
sidecar would introduce hidden state and attempt aliasing. Use one explicitly declared
region whose immutable typed stage values are local to the admitted invocation.

## Outcome

Compile semantic stages through owned typed values. Emit the complete graph family at
its requested publication or inspection boundary. Carry original supported source
occurrences through transformations; never invent row witnesses for unmaterialized
intermediate relations. Shared formal bodies bind to distinct actual occurrences.

W15 deletes the unused unexpanded `AdmittedModel::specialization` hash API and its
identity wrappers. The complete `SpecializationRequest` consumed by the production
compiler is the sole specialization equality contract; its hash selects a bucket
and never establishes reuse. Actual instance bindings remain separate values.

Specialization inputs project only the reachable source graph, including ordered
payload and kernel-binding dependencies. Dense ordinals belong to this temporary
projection. Each occurrence retains its own original-to-projected correspondence;
binding translates the shared result back before emitting provenance. Unrelated
family nodes and storage ordinals cannot invalidate the reusable body. Complete
consumed environment values remain in equality; this projection is not permission
to discard negative or conditional binding dependencies.

Source lowering likewise uses local expression ordinals. The native export adapter
translates only mathematical references into the current relation-family numbering;
source-local predicate/equation IDs and lexical identities are preserved. Row offsets
are encoding decisions and never Salsa inputs.

The reusable definition body is the complete keyed set of lowered source expressions,
including their grammar roots, predicates, equations, lexical bindings, paths and unit
correspondence. A bare expression DAG cannot represent that contract. Normalization
installs these derived bodies together with the generated interfaces and instances
only after the native output obligations succeed. Original family offsets and source
locations remain separately tracked occurrence data. Installation uses the existing
atomic coordinator and exact source identity, preserves that input identity, and is
discarded on source replacement. Realization reads the installed facets; no parallel
compiler or mutable source-graph cache becomes authoritative.

Source-derived case preparation uses the same application compiler coordinator.
A derived case carries the exact admitted source identity; installation refuses a
stale source or an attempt to replace a direct case input. These completed derivations,
like relational phase completions, do not change the authoritative input identity.
Installing a new source removes derived cases. Their actual mathematical values still
update the existing case inputs and numerical/structural queries, and escaped results
retain their original generation. No second compiler database or mutable graph sidecar
is introduced for source-to-solver preparation.

### Consequences

The runtime composes the region directly. Registry declarations and generated callers
change together. The final output set retains all required mathematical and evidence
relations. Intermediate representations do not become independent persistent artifacts.

### Compensating controls

Exact typed equality, complete graph traversal including payload and kernel edges,
guarded evaluation, bounded inputs, retained source owners, canonical byte comparison,
and final registered relation/invariant checks. Failures expose no partial completion.

### Confirmation

Isolated transformation and boundary controls establish only their named claims.
W19 owns full compiler/publication equivalence and W20 owns performance qualification.

## Pros and cons

The region removes mandatory graph transport and lets physical admission consume typed
mathematics. It requires explicit preservation of provenance and replaces the former
per-stage native port plumbing.

## More information

[Execution packet](../plans/13-w07-w11-execution.md), Plan 13 W09 and ADR-0076.
The decision/design PR must carry the corresponding blueprint revision; this proposed
record does not claim that formal acceptance or implementation has completed.

## Status history

- 2026-09-23 — proposed before the region implementation; maintainer-authorized W09 scope.
