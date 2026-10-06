# Source lifetimes, admission and publication boundaries

Supporting source assessment for the [execution-efficiency and SurrealDB review](../../reviews/design_review_execution-efficiency-and-surrealdb_2026-10-05.md).
This is coordinator evidence, not an independent overall verdict or finding-status ledger.

## Baseline and method

Inspected 2026-10-05 at `6498b013e579ec8039573200c92282eb8715b52e` with concurrent
numerical changes in the working tree. The files examined below were unchanged at the
start of this investigation. Source inspection only: no product test, allocation experiment,
database operation or performance measurement was run. Mechanism claims are
**Interface-checked**; proposed corrections have not been implemented.

The assessment applies Core 3.4 AP-04/AP-07, DP-03/09/10/19/20/23 and relevant H12–H20,
without treating every copy or repeated check as a defect. Exact source bytes, source locations,
immutable old revisions still held by real consumers, and checks under changed native bindings
have legitimate purposes.

## A. Editing retains a chain of complete superseded bundles

**Interface-checked.** The private `BundleOwner` in
[`document/owned.rs`](../../../../crates/pse-runtime/src/authoring_driver/document/owned.rs)
contains both its current `DocumentBundle` and `_parent: Option<OwnedDocumentBundle>`.
`OwnedDocumentSet::edit` clones unaffected package owners, which is useful. For an affected
package, however, it:

1. Reserves the current package extent and copies every document's bytes into a new map.
2. Calls `load_reusing` to retain unchanged parser owners while rebuilding hydration.
3. Creates a new bundle owner with `_parent: Some(part.clone())`.

Consequently the latest edited bundle retains its predecessor, which retains its predecessor,
and so on. Dropping every external handle to earlier revisions does not release those bundle
owners. Each contains the complete prior document inventory and its leases. This is a source
lifetime consequence, not a measured leak or an assertion that shared Arrow buffers are copied.

The parent preserves allocation ownership for reused data. That protection is necessary;
retaining all superseded source content is not the smallest ownership unit that can supply it.
The existing `editing_one_document_reuses_other_parser_owners` test checks one-step parser reuse;
it does not establish bounded retained history over many successive edits.

**Proposed correction.** Give immutable document/parser/column allocations directly retained
owners and leases. A revision retains the document versions it actually uses. Old revisions
remain alive only through genuine consumers or an explicit history policy. Share unchanged
documents directly rather than retaining a complete ancestry of bundle owners. The operation
continues to preserve original bytes, source locations and all currently referenced revisions.

**Verification that would settle the correction.** Repeatedly edit one document, dropping each
old revision. Observe retained allocations after warm-up, then retain one intentional old
revision and verify that exactly its needed allocations survive. Check both latest and retained
old-source inspection. Exercise cancellation/refusal before replacing the active revision and
verify that leases remain charged until their last actual consumer drops. No such experiment
was performed in this review.

**Database relevance.** Persistent revision records could support explicit history, but inserting
the same ownership chain into a graph database would preserve its retention problem. Direct
in-memory ownership is a simpler correction for the existing edit API.

## B. Immutable admission evidence loses the native context it established

**Interface-checked.** `OwnedDocumentSet::validate_context` first compares registry declarations,
then calls `FieldCheckedBatch::admit` on every retained bundle batch. Its principal consumers
include [`PhysicalContext::from_documents`](../../../../crates/pse-runtime/src/workflow/physical.rs)
and [`Runtime::modeling_from_documents`](../../../../crates/pse-runtime/src/workflow/modeling.rs).
The ordinary sequence loading a document set, constructing physical context and admitting a
model can therefore revisit the same unchanged batches under the same runtime assembly.

[`FieldCheckedBatch::admit`](../../../../crates/pse-relations/src/columnar.rs) validates schema,
obtains the context's prepared relation predicate and evaluates it. The resulting checked
batch retains a relation contract and immutable storage, but not the actual validation-context
owner used for that admission.
[`PreparedLocalContract::evaluate_selected`](../../../../crates/pse-relations/src/validate/prepared.rs)
performs full Arrow structural checks, traverses local values and evaluates native predicates.
Predicate preparation is cached; successful evaluation of unchanged values is not retained
across these calls.

The distinction matters because
[`EngineFactory::validation_context`](../../../../crates/pse-engine/src/session/factory.rs)
returns a retained context from the actual immutable native assembly. Equality of registry
names or schema alone would be insufficient: another runtime can supply different native
bindings. Within the same actual immutable owner, however, those premises have not changed.
The present API cannot express that narrower already-established guarantee.

**Proposed correction.** Retain successful local admission together with immutable storage,
the resolved contract and the actual native validation owner. Reuse it only for the identical
owner and unchanged values, or a transformation whose contract preserves that property.
Re-admit when data, schema, interpretation or native implementation changes. This is a local
capability/ownership correction; it does not call for persisted proof objects or digest-based
trust. Keep force-validation available to correctness controls and keep cross-relation closure,
new numerical-state checks and storage reconciliation where their premises differ.

**Verification that would settle the correction.** Existing evaluation counters can distinguish
predicate preparation from execution. Exercise two consumers of the same immutable data and
context; then substitute a different native predicate owner with the same schema and demonstrate
the appropriate reevaluation/refusal. Include a changed batch, selected rows, cross-relation
closure and an imported stored representation. Do not optimize away the first semantic check or
independent physical checks.

**Database relevance.** Database schemas do not automatically prove the Rust/native contextual
invariants. This work can be removed at existing ownership boundaries without a store pivot.
A SurrealDB realization would still need to preserve the same distinction between admission,
changed trust/context and storage-codec validation.

## C. Package-level work remains broader than a changed document

**Interface-checked, supporting manifestation rather than a separate verdict.**
[`load_reusing`](../../../../crates/pse-runtime/src/authoring_driver/document/load.rs) compares
original bytes, document declaration and interpretation before reusing a parser. It retains
the parser `Arc`, but clones syntax values/spans into the current representation, rehydrates
the package and projects its documents. `OwnedDocumentSet::edit` copies all source bytes in
each affected package. It retains unaffected package owners rather than rebuilding everything.

Some package-wide work is necessary: an identity/name or binding edit may affect other documents.
The review has not established a sound incremental hydration algorithm, nor measured which
portion dominates latency. A correction should preserve membership, absent lookup, header and
binding dependencies rather than assuming that an unchanged text file has unchanged meaning.
Separate source parsing reuse from semantic dependency reuse in subsequent preparation design.

## D. Publication has real strengths and an unresolved granularity tradeoff

**Interface-checked.** [`RunResult::prepare_publication`](../../../../crates/pse-runtime/src/workflow/publication.rs)
consumes retained result tables and explicitly does not rerun science. It prepares a publication
artifact independently of visible commit. The surrounding protocol separates intent, member
writes and catalog visibility. These are different effects; their coexistence is not itself
duplicate authority.

[`Publication::open_interpreted`](../../../../crates/pse-catalog/src/delta/publication.rs) binds all
member descriptors. `bind_interpreted_members` opens providers with bounded concurrency, retains
their recorded contracts and constructs a catalog. Relation payloads remain lazy. The
[`DecodedView`](../../../../crates/pse-catalog/src/delta/provider.rs) forwards filters, projections
and limits to the native view while restoring declared schema metadata. It is incorrect to
describe this path as eagerly collecting all published rows.

[`admit_output`](../../../../crates/pse-catalog/src/delta/write.rs) keeps pure, single-consumer
inputs streaming. It materializes dependency-bearing writes before taking an output permit so
an orchestrating writer does not hold a slot while waiting for a child writer. Removing that
materialization without preserving the dependency/deadlock contract would be an invalid economy.

Opening every member's metadata for a narrow read is a candidate granularity improvement, not
a demonstrated throughput bottleneck. Assess lazy member binding against the promised immediate
rejection of missing/incompatible selected members, exact reopening and lease lifetime. The
current cache and bounded-open behavior are contrary evidence to a blanket whole-store-load
diagnosis. No Delta-to-SurrealDB speedup is established by this inspection.

## Evidence limits

No quantitative performance, capacity or acceptance claim follows from these source observations.
The source-owner chain and repeated local evaluation are concrete mechanisms suitable for the
independent principal reviewer to assess. Publication opening and package hydration require the
stated semantic tradeoff assessment before being promoted to independent defects. Existing tests
were read as verification leads, not newly executed evidence.
