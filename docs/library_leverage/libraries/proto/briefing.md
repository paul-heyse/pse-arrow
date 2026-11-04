# Proto ecosystem: codebase-wide discovery and proposed target

Inspected 2026-11-03 at `f55951cfe995a689f9b75185b8e89cb362021616`, including the
eight operator-supplied uncommitted Proto references. This supersedes the Plan64-only
interpretation at `01b62e928`. Mechanisms are **Interface-checked** by representative
source and exact library-source reads; target compositions and benefits are **Proposed**.
No product/scientific execution, conformance campaign or performance measurement ran.

## Proposed direction

Use Proto as shared contract infrastructure across scientific exchange, local operations,
storage projections, diagnostics and engineering tools. Keep executable domain meaning with
its actual Rust/registry owners; derive descriptors, codecs, validation and clients from
those meanings. Independently governed external protocols remain schema-owned imports.
This owner-derived hybrid is a comparison outcome, not a requirement to express every
internal object as a message or expose every operation through a service.

The main opportunity is to stop reinterpreting structural meaning independently in wire
adapters, generation, database projections, inspectors and clients. Additional value comes
from descriptor-aware inspection/editing, typed retained diagnostics, independent scientific
clients and extensible provider contracts. Those are complete-operation hypotheses, not
benefits established merely by changing serialization.

Keep native mathematical/evaluator/solver objects in effective numerical representations.
Exchange reconstructible source, configuration, capabilities, provenance and outcomes where
there is a consumer. Keep Arrow for columnar scientific products when its execution/access
model fits. Distinguish SurrealDB's native value codec and gRPC protocol from application
persistence, query projections and historical interpretation.

Local operation is the target. New process boundaries must purchase isolation or independent
access; direct native execution remains eligible. Any breaking change is acceptable in the
recommendation. Incidental mechanisms and compatibility shims do not constrain the target.
Scientific meaning, original operation custody and truthful outcomes still require owners.

## Responsibility-oriented briefings

| Briefing | Mechanisms and intended use |
| --- | --- |
| [Contract ownership and generation](briefings/contract-ownership-and-generation.md) | Owner-derived descriptors, shared generation/options, source identity and evolution; Proto-first/Rust-first comparisons |
| [Representation, reflection and validation](briefings/representation-reflection-and-validation.md) | Buffa/Prost, dynamic/generated reflection, presence, mutation, ProtoJSON, encoding/views and generated constraints |
| [SurrealDB storage and projections](briefings/surrealdb-storage-and-projections.md) | Native codecs, queryable/opaque/hybrid records, graph freshness, historical interpretation, replay and recovery |
| [Runtime and local services](briefings/runtime-and-local-services.md) | ConnectRPC/Tonic, authentication, preparation, streams, limits, deadlines and settlement |
| [Math, solvers and workers](briefings/math-solvers-and-scientific-workers.md) | Reconstructible artifacts, provider demands, numerical ownership, worker placement and scientific extensions |
| [Diagnostics and results](briefings/diagnostics-and-results.md) | Typed failures, bounded details, progress completeness, retained evidence, Arrow and observability |
| [Clients, intelligence and reference tooling](briefings/clients-code-intelligence-and-reference-tooling.md) | Native facts/CLI, independent clients/calculations, inspection and compiler/build/debug tools |

No length limit applies. Each document supplies precise mechanisms, references, complete local
compositions and counterconditions. The [coverage account](coverage.md) maps the broader
reference and codebase population; [applicability](applicability.md) connects it to current
opportunities. The [comparative review](../../reviews/proto/README.md) owns the integrated
target judgment rather than seven independent adoption decisions.

## Newly important research territory

The supplied additions expand discovery beyond engineering APIs: public database value
codecs; descriptor-driven native mappings; graph/history/replay; generation companions and
structured generation diagnostics; archival fidelity; rich errors and terminal stream
semantics; bounded encoding, size caches and segmented data. Each is assessed at an actual
consumer boundary, including internal runtime and mathematical artifacts.

Engineering projections already derive from native owners, and reference exchange already
retains an original message separately from its checked interpretation. These are extension
and consolidation opportunities. Hand-mapped host facts, private native-child documents and
durable JSON records supply useful comparisons; different formats can remain justified.

Priority follows potential whole-operation value: shared generation/interpretation; typed
diagnostic and result contracts; scientific exchange/worker extensibility; database projection
and retained interpretation; selective representations and independent tooling. The shortlist
does not exclude other supplied capabilities. Views can retain too much memory, scalar RPC can
overwhelm numerical work, and generic database codecs do not provide application transactions.
These counterconditions shape the proposed design.

## Evidence limits and ownership

Current declarations include Buffa 0.9.2, ConnectRPC 0.9.1, Protovalidate-Buffa 0.7.2,
Prost 0.14.4, Tonic 0.14.6, surrealdb-types 3.3.4 and surrealdb-protocol 0.13.1;
manifest and lockfile remain their owners. Newer releases in the references are eligible
routes whose conditions are recorded in the thematic accounts. Context7 supplies orientation;
installed/tagged source supplies decisive version-specific evidence.

No Proto semantic census or API-export bundle was prepared; the existing family-export
configuration does not select Proto. Source samples cannot establish universal absence.
Recorded installed-tool availability and qualification were not rerun. SurrealDB v4 was
excluded. [Plan64](../../../plans/64-process-simulator-target.md) owns implementation status;
this research does not change its status or accept a new architecture.
