# Integration boundaries and replacement opportunities

Supporting source investigation for the
[unified-substrate review](../../reviews/design_review_surrealdb-unified-simulation-substrate_2026-10-05.md).
The principal review owns the architectural judgment. These observations do not authorize a
migration or establish measured speedups.

**Interface-checked, 2026-10-05:** checkout HEAD
`6498b013e579ec8039573200c92282eb8715b52e`, alongside concurrent numerical work. This
investigation reads the persistence, authoring, compiler and generation boundaries; no
product commands or storage mutations were performed. Proposed removal depends on moving
the named consumers, not on merely installing a database.

## Durability is currently a separate workflow

Blueprint §20 describes execution in memory until the caller requests publication.
This concerns ordinary result relations, not an absence of all durable execution state:
[`DurableAttempt::finish`](../../../../crates/pse-runtime/src/workflow/durable.rs) already
records attempt completion, progress, incumbents and reusable seeds. Durable studies also
persist sources and write point members automatically before their final publication through
[`workflow/study.rs`](../../../../crates/pse-runtime/src/workflow/study.rs). The target
generalizes automatic durability to ordinary complete problem/run/result operations.
[`workflow/publication.rs`](../../../../crates/pse-runtime/src/workflow/publication.rs)
implements a distinct `Workspace`, `Published`, `PublicationAttempt` and
`PublicationSettlement`. Its composition registers an operational intent, executes immutable
Delta member writes and then conditionally commits the catalog. A lost acknowledgement is
settled through catalog lookup. A publication identifies both a run attempt and a separate
publication history/head.

[`pse-catalog`](../../../../crates/pse-catalog/Cargo.toml) directly depends on DataFusion and
Delta. [`pse-operations`](../../../../crates/pse-operations/Cargo.toml) depends on generated
operations queries, PostgreSQL client, pool and TLS adapters.
[`pse-runtime`](../../../../crates/pse-runtime/Cargo.toml) composes both.
This structure implements a real guarantee: readers see a coherent admitted result despite
cross-resource failure. The guarantee is necessary; the particular two-store protocol is
not intrinsically necessary.

**Proposed integrated operation:** accepting a normal run durably records the selected
problem revision, inputs, configuration and interpretation before starting execution.
Scientific values, diagnostics and result sets are attached to that run inside the same
database. A small result can commit with its terminal state; larger results can be staged in
bounded private chunks and made visible through one completion transition after admission.
No transaction needs to encompass numerical solving. Failed and cancelled runs remain
queryable with honest outcome and partiality information.

This could remove mandatory Delta member publication, member manifests, external-prefix
settlement and PostgreSQL catalog composition for adopted data. It does not remove run
identity, visibility rules, uncertain-acknowledgement lookup, legitimate write conflicts or
cleanup of abandoned private data. Those remaining responsibilities become operations over
one authoritative system rather than coordination between two systems.

Automatic durable results differ materially from retaining an optional `publish` call whose
implementation changes. That user-visible behavior belongs in the target and migration.

## Querying becomes an ordinary operation on retained problems and runs

[`pse-catalog::inspection::TableReader`](../../../../crates/pse-catalog/src/inspection.rs)
currently prepares an exact registered member or SQL query through an `EngineSession`, then
returns leased Arrow batches. It supports cancellation and explicit close. This is a useful
streaming boundary, not evidence that all reads materialize everything.

The integrated alternative receives a problem/revision/run selector, output identity and
optional case/time/component selection. The database joins or traverses retained identities,
filters and projects the requested scientific values, and returns the selected answer.
An application should not first reconstruct a package, reopen all publication members or
interpret a publication manifest to answer a stored result question.

Different runs of one revision must remain distinguishable. A default such as latest
completed run is an explicit selection policy, not an equivalence between all outcomes.
Units, basis, reference state, qualification and diagnostics travel with the selected
answer. Database record typing supports these distinctions but does not define their
scientific interpretation independently.

Arrow is still a useful output representation for Python and analytical consumers. Producing
Arrow from selected records does not require Delta to remain a canonical store. Existing
SQL endpoints need a deliberate retained analytical projection or API replacement; graph
query syntax is not automatic compatibility with arbitrary DataFusion SQL.

The current Python [`Runtime.query`](../../../../python/pse/_workflow.py) binds operational
tables and optionally supplied in-memory results or an opened publication. It is not the
proposed direct stored-results-by-run-ID operation. The result-query improvement therefore
changes the consumer journey as well as its internal driver.

## Source ownership and compilation can change together

[`OwnedDocumentSet`](../../../../crates/pse-runtime/src/authoring_driver/document/owned.rs)
retains complete package inventories. An affected edit retains a predecessor through
`BundleOwner::_parent`; contextual admission loops over all retained batches. These are
physical ownership choices, not obligations of scientific authoring.

[`ModelingPackage`](../../../../crates/pse-runtime/src/workflow/modeling.rs) combines runtime,
compiler revision, physical context, declarations, document ownership and source export.
It already admits authored declarations as data through `modeling_package`; ordinary
parameter/model edits are not inherently Rust recompilations.
[`workspace/modeling.rs`](../../../../crates/pse-compiler/src/workspace/modeling.rs) provides
tracked document inputs, selected declarations and specialization. It is not presently a
database-backed compiler.

**Proposed integration:** persistent addressable objects and explicit semantic dependencies
become the source of selected compiler inputs and retained derived products. The runtime
holds a pinned revision and the needed working set, rather than an owning transitive chain
of prior bundles. Contextual admission can survive process exit when its actual value,
contract and implementation premises have a reproducible persisted meaning.

That target must address membership, absence and interpretation changes as well as positive
references. Deleting a definition, inserting a previously absent name, changing a provider,
or changing a rule can affect consumers whose old positive-edge set contains no newly added
record. Query correctness requires those dependencies to be represented. Ordinary graph
reachability is not itself complete incremental evaluation.

Avoid moving the same defect into storage: a revision design requiring unbounded ancestry
replay for each read or a full graph copy per edit defeats targeted updates. Persistent
history and bounded active working sets are compatible if revision membership and relevant
version selection have direct indexed access. The principal review selects the design and
remaining uncertainty; this note does not invent a second implementation contract.

Native evaluators, sparse layouts and solver state remain operation-shaped derived data.
They can consume a compact closure linked to persistent identities. Saving canonical
compilation inputs or portable descriptions does not make process pointers, library handles
or mutable solver memory restartable. Any retained native artifact requires its actual
format and dependency compatibility contract.

## Build and generation savings need a replacement inventory

The [earlier build investigation](../execution-efficiency-2026-10-05/build-turnaround.md)
establishes broad workspace-hack dependencies, whole-tree build provenance, eager native
setup and no-op generated writes. A unified architecture can make some of their consumers
obsolete rather than merely optimizing them.

| Current machinery | What a unified target can remove | What remains a real responsibility |
|---|---|---|
| Generated PostgreSQL queries, pool, COPY/SQL adapters and installation path | Remove after operational/catalog consumers migrate | Typed operations, concurrency, cancellation and recovery in the selected engine |
| Delta member layout, manifests and publication reconciliation | Remove as mandatory machinery for database-owned results | Exact value representation, coherent completion and bulk I/O |
| Source relation exports reconstructed for persistence | Replace with retained canonical objects and queryable dependencies | Authored source fidelity and selected scientific admission |
| Per-representation generated relation surfaces | Remove surfaces used only to bridge retired representations | Contracts actually consumed by remaining Rust, Python and numerical interfaces |
| Monolithic preparation/provenance keys | Replace with relevant persistent dependency/implementation identities | Complete dependencies and exact provenance; no claim that metadata alone changes build invalidation |
| Native preparation for unrelated operations | Narrow runtime capability closure and build recipes | Native libraries required by the selected scientific operation |

[`pse-codegen::documents`](../../../../crates/pse-codegen/src/codegen/documents.rs) currently
derives JSON Schemas from Rust-owned serde types and generates closed Python msgspec types.
These have a distinct purpose from PostgreSQL query generation or Delta layouts. A generic
database query interface does not automatically replace typed public configuration and
scientific result contracts. Reconsider their physical realization without silently making
Python untyped or duplicating the domain definition in handwritten DDL.

[`xtask::codegen::write_tree`](../../../../xtask/src/codegen.rs) writes each generated output
without comparing bytes. Retiring an output removes that write entirely; remaining generators
still benefit from unchanged-output preservation. Native ABI bindings are not eliminated by
moving models and results into a graph.

[`artifact_requests`](../../../../crates/pse-compiler/src/workspace.rs) includes global source
and build identities. The corresponding cache is process-local. A persistent selected
artifact scheme would introduce a real cross-process reuse opportunity; that opportunity
should be assessed on its own compatible artifact representation, rather than described as
a previously measured cross-build cache regression.

The relevant comparison is the combined closure after deleting migrated consumers. An
embedded database added to an unchanged dependency graph only adds another engine. A local
server can isolate that engine from application compilation and accommodate concurrent
workers, while still requiring installation, lifecycle and durability ownership.

## Transfer from library-context

The supplied graph-native review recommends an admitted graph with projections rather than
many storage-specific representations. Its capability follow-up broadens native execution
through stored functions and qualified shared kernels. These support evaluating the complete
composition rather than restricting SurrealDB to document storage.

Its separate database per rebuilt snapshot, disposable generations and single-operator
cutover are not adopted here: this target requires ordinary incremental revisions, concurrent
runs and preserved scientific outcomes. Likewise, removing a database from compiler scratch
work in that repository does not require keeping all compilation in memory here. Placement
follows the actual operation and contracts.

This investigation supplies current source evidence and candidate replacement boundaries.
The principal review must settle the proposed architecture, scientific constraints, rule
impacts and its recommendation independently.
