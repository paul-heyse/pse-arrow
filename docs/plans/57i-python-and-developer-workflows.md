---
title: "Plan 57i: Python and developer workflows"
status: draft
date: 2026-10-27
adrs: [ADR-0116, ADR-0178, ADR-0219, ADR-0222, ADR-0224]
review_sources: [docs/library_leverage/reviews/declarations-registry-and-python-boundary.md, docs/library_leverage/reviews/developer-tooling.md, docs/library_leverage/codebase/registry-generation-and-python-boundary.md, docs/library_leverage/codebase/developer-tooling.md, docs/design_review/reviews/design_review_study-execution-and-database-boundary_2026-10-28.md]
doc_role: plan
doc_owner: docs/plans/57-library-leverage-and-correctness.md
doc_topics: [current-work, design, diagnostics, libraries, environment, validation, storage]
---

# Plan 57i: Python and developer workflows

## Database-boundary review amendment

K owns the new durable public client/import boundary; I retains scientific public-family
feasibility, trusted codec preparation, generated developer adapters and capture. I02/I03's
conditional native-public-type direction must not force connection-only records or imports to
load PyO3. Coordinate common generated documents/import leaves with K01/K03, preserving Rust-owned
meaning, strict finite validation, errors and stubs. Never create a second editable schema.

I04 evidence/knowledge and I06 CLI/MCP/selection consumers join K04's operation inventory.
Only consumers whose requirements are connection/data-only move to that path; actual scientific
work retains its native producer and admission. Generated adapter availability alone proves
neither publication validation nor real canonical behavior. SurrealDB remains canonical (U3).
I05 finite capture continues to distinguish clean completion and incomplete/reset streams.
F07 supplies any changed attribution contract without replacing capture's completeness policy.

Before public-family migration, reconcile the selected package/import architecture with I02's
constructor/equality/indexed-view/mutation/helper/stub assessment and its ADR/formal review.
K's boundary correction does not settle those separate choices. Actual native-independent
imports and developer publication routes are assembled J evidence, distinct from code generation.


## Purpose and baseline

This companion develops CBO-041, CBO-043, CBO-044 and CBO-049–CBO-052, including their retained
judgments, minor improvements and unresolved premises. The [coordinator](57-library-leverage-and-correctness.md)
owns the one CBO disposition ledger, operator rule decisions and combined completion. The package
table here owns local execution state. This is comprehensive plan authoring; no runtime, native
provider, database, compiler campaign or implementation is authorized merely by this document.

The inspected baseline is `93946f572` (product source `4f52fd239`), consuming the complete revised
G8/G9 review and supporting codebase/library inputs, additional-pattern revision 4 and retained
evidence integrated at `1540fd164`. The focused foundation assessment uses Core 3.4, efficient-
architecture heuristics, test-assessment 1.2, the pse-arrow binding and applicable process-simulator
1.5 guarantees. Python must preserve physical meaning, units, outcome partiality and exact replay
interpretation; these workflow changes do not formulate or qualify new science.

Blueprint §21/§21.5 owns the Python boundary, §23.2 diagnostic privacy/classification and §24.1
the tooling evidence/knowledge authority. Proposed ADRs are not accepted merely because their
current implementation has tests. [57h](57h-registry-identity-and-diagnostics.md) owns typed codes,
source presentation and generator syntax; [57a](57a-result-data-and-prepaid-accounting.md) owns
the data/buffer accounting boundary. Coordinate the signal-aware wait package in the execution/
observation companion with public native methods so the same wait is migrated once.

## Focused foundation assessment and target

### Trust-aware document crossings, without changing public admission

`pse-py/src/documents.rs` currently imports document classes and codec functions on each crossing.
Rust→Python `DocumentValue` serializes the actual Rust owner and then calls public `decode_json`;
that decoder walks raw nested JSON and calls native header admission again. Import already checks
package, registry and boundary-schema fingerprints in `_build`/`__init__`. The repeated walk is
not a second rule authority: Rust owns `check_required_header`. It is repeated assurance on an
immediate producer output after catalogue agreement is established.

I01 adds a private trusted decode used only by `DocumentValue::into_pyobject` for bytes it just
serialized from a current Rust owner. It reuses `_json_decoder`'s **existing bounded per-thread
cache**, preparation lock and `_decode_identity` hook, then calls its typed decoder without the
untrusted raw-header traversal. It does not install one process-wide Decoder per type. Generated
per-document associated functions provide `PyOnceLock` caches for Python classes and codec
callables, as the existing enum/identity route demonstrates. A `static` inside a generic function
is shared across monomorphizations and is invalid for this purpose. Projection owners, `Arc<T>`,
borrowed owners and distinct input/output class names must resolve to the actual catalogue type.

Public `decode_json`, `decode_toml`, supplied/recorded bytes, imported files, `RecordedInput` and
other untrusted ingress keep required, unique, header-first admission, unknown-field refusal,
finite-value/identity rules, byte allowances and typed causes. The private route is never selected
because an input claims a trusted origin or happens to be a generated Struct. A previously emitted
document that is later supplied back across a boundary is still untrusted ingress. A serializer
that omitted a required header would be a producer defect: typed msgspec decoding can fill its
Literal default, so explicit serializer-output tests retain that obligation. Import mismatch must
refuse before any trusted call becomes reachable.

On Python→Rust, keep JSON as the encoded-document contract and Rust final contextual admission.
Remove only `decode_document`'s redundant header pre-scan for FlatV1/FlatV2/ValueV1 after confirming
`Wire::decode_slice` owns it; Native retains its one required pre-check. Preserve actual order of
size and interpretation admission, including malformed body plus missing/duplicate/wrong header
cases. The source-level route must demonstrate exactly one authoritative check, not simply a
lower callback count on one happy path. Future wrappers migrated under I03 accept typed
`DocumentInput`/`RecordedInput` directly at native signatures instead of Python-side encoding;
the common bridge still performs its encoded-document boundary work.

msgspec tags remain unsuitable as the admission authority: the retained 0.21.1 probe shows
missing direct tags can default, equal duplicate tags pass and first error follows document
order. `pythonize` would remove the JSON hop but change byte allowance, finite-value and encoded-
document guarantees; no measured need or decision selects it. Cattrs msgspec preconfiguration
does not erase the two distinct construction/decode contracts below.

### Public native types: a conditional target with a complete contract decision

CBO-044 favors making forwarding-only native classes the public types where they have a complete
native realization. On **2026-10-27** the operator explicitly selected **“Include conditional
native-public-type target”** (coordinator RC01). This confirms its inclusion in the plan, not
acceptance of an unexamined public contract or authorization of product migration. I02 inventories
and resolves the complete target under an ADR plus formal design review; I03 waits for those
prerequisites and the family-specific preservation decisions. A family without a coherent complete
native realization retains a reasoned bounded facade rather than acquiring a speculative generated
forwarding substitute. I01/I04/I05/I06 remain independently executable when authorized.

The current wrappers are frozen attrs classes around Python native-handle objects; some construct
their handle and some add genuine Python behavior. Review counts of 405 forwards, 41 classes and
22 unwraps are discovery hints, not a demonstrated deletion scope. The initial family map below
identifies concrete owners to complete under I02; it is not a second ongoing symbol registry.

| Public family / source owner | Boundary facts requiring an explicit migration decision |
| --- | --- |
| `_modeling.py`: `ModelingResult`, `ModelingNativeAnalysis`, `ModelingElasticAttempt`, `ModelingNonlinearExplanation`, `ModelingConformance`, `ModelingInitializationAttempt`, `ModelingInitialization`, `ModelingDiagnostics`, `ModelingTrajectory`, `ModelingDiagnosticSamples`, `ModelingKnowledge`, `ModelingPackage` | Properties versus methods, wrapping of nested results, collection loops, typed controls, genuine Python fit/row adapters, and original native report identities |
| `_modeling.py`: `LocalStudy`, `StudyReport`, `CompactStudyResult`, `SingularMode` | `LocalStudy` is not the same attrs facade pattern; `SingularMode` is a Python value. `CompactStudyResult` retains the report **and index**, while `StudyReport` wraps the report alone. One renamed report cannot replace both. |
| `_runs.py`: `StoredResult`, `PreparedOperation`, `LiveSimulationEvent`, `LiveSimulationCursor`, `RunResult`, `ScientificReplay`, `RunHandle`, `ProgressStream`, `StudyHandle` | Event/result ownership, repeat reads, iteration, context/close/drain, completion/refusal and signal-aware waits, plus record-preserving replay |
| `_workflow.py`: `Runtime`; `_project_bundle.py`: `ProjectBundle`; `_physical.py`: `PhysicalContext` and Arrow protocols | Python constructors, deployment/durability selection, immutability, Runtime's generated project/research/workbench mixins, and protocol-based inputs |
| `_inspection.py`: `TableStream`; `_analyses.py`: `Analysis`; `_artifacts.py`: `ScientificArtifact`, `ArtifactSettlement` | Stream iteration/context manager, independent buffer ownership, Python `extension_report`/pyarrow schema comparison, retained artifact capabilities and settlement semantics |
| `_strategies.py`: `PreparedFlow`, `PreparedStrategy`, `StrategyResult`; `_study_workbench.py`: `StudyKpiAnalysis`, `FrozenStudyCohort`, `StudyObserver` | Nested native returns, preprocessing convenience and selection semantics; study indices, frozen membership and observer lifetime |
| `diagnostics.py`: `OperationObservations`, `Diagnostics`, `CaptureSession`, `LaunchedCommand`, `DetailedCollection` | Host command construction, collection/close/cancellation and source-safe exception/report behavior; do not erase real Python lifecycle composition |
| `_transfer.py`, `_array.py`, `_scientific_inputs.py`, `_telemetry.py`, `_research.py`, `_host_operations.py`, `_project_operations_glue.py`, generated public operation modules | Python value/helper/protocol owners and mixins, private native telemetry lifetime, document/row adapters and generated operation lowering; these cannot all be counted as forwarding-only handle classes |
| `__init__.py`, `modeling.py`, authoring/adapters and all public submodule exports | Public aliases, `__all__`, public module/name identity, annotations, documentation and user-visible constructors, plus any exported native types already public |

I02 follows every public export and its actual returned/accepted type through `_build` to the
native owner, recording the final decision in the ADR/review, not a new mechanically maintained
catalogue. Include already-direct types, wrapper fields beyond `_handle`, generated host operation
glue and constructor aliases. Sampled forwarding counts cannot establish completeness.

The preferred realization, conditional on that boundary decision, is PyO3 **0.29.2** public naming via
`#[pyclass(name = "…", module = "pse.…")]`, frozen where appropriate, with re-exports only through
`pse._build`; preserve the one-native-import gate. Rust `#[pymethods]` owns documentation and
accessors for migrated families. Return tuple collections in one crossing where the existing
contract is an eager tuple, retaining order, multiplicity and failure behavior; do not eagerly
materialize a lazy stream merely to use a bulk return. Native method signatures consume typed
documents through the common bridge. Native result returns are base instances, so a Python
subclass cannot supply methods to every Rust-returned object.

Several contract choices cannot be inferred from `frozen` or `subclass`:

- **Equality and hashing.** attrs currently compares/hashes the wrapped **Python handle object**
  and additional fields such as an index. Native Rust report/result types generally lack
  `PartialEq`/`Hash`. Do not derive Rust value equality as a convenience. Preserve handle-object
  identity semantics for directly exposed handles, and report-handle identity plus index for
  compact views, unless an explicitly accepted ADR chooses a different public contract. Repeated
  getters may mint distinct Python handles to equal internal Rust data; those remain distinct
  wherever today's handles are distinct. Tests include same handle/two facades, independent
  handles/same data, views at same/different indices and dictionary/set behavior.
- **Indexed view.** Prefer a dedicated native `CompactStudyResult` view retaining the exact
  Python report handle plus index, with explicit `completion`, `table_names` and `table(name)`
  methods/properties and lifetime. A small Python view is a coherent retained alternative if the
  complete identity/mutation contract would otherwise cost disproportionate machinery. Choose
  one before deleting its facade; no claim of whole-layer deletion is made.
- **Mutation and construction.** attrs raises `FrozenInstanceError`; PyO3 `frozen` alone does
  not reproduce that exception or Python constructor signatures. Decide whether to preserve those
  exceptions with explicit Python-facing behavior or accept their documented change. Include
  assignment/deletion, unknown attributes, subclass mutability, keyword-only/default behavior,
  callable factories and `repr`. Constructor names that currently accept an implementation
  handle must receive an explicit preservation/removal decision, not disappear by renaming.
- **Runtime and mixins.** `Runtime(ProjectRuntimeOperations, ResearchRuntime, StudyWorkbenchMixin)`
  uses `object.__setattr__` to install its native owner; merely enabling native subclassing does
  not settle constructor/MRO or frozen behavior. Prefer moving the generated forwarding operations
  into the native public Runtime when that preserves the generated catalogue. Retain a small
  Python composition facade for genuinely Python-owned operations if that is the coherent contract.
  Compare that route with a Python subclass only after validating allocation/initialization and
  returned-instance behavior. Do not leave duplicate `_host_*` paths as a migration shim.
- **Python helpers.** `TableStream.extension_report` calls pyarrow schema comparison. Keep its
  behavior as a real method through an explicit native-to-Python helper call, move it to a clearly
  named module function only under the accepted API decision, or retain that bounded facade.
  Python row/fit conversion and host lifecycle helpers receive the same explicit disposition.

The stub generator currently strips docstrings in `xtask/src/codegen/python_stubs/validate.rs`.
I02/I03 change the owning emitter/validator to retain introspection documentation and produce
correct public re-exports, modules, return types and keyword defaults, then regenerate stubs.
Inspect actual IDE-facing docstrings and signatures; native `__doc__` alone does not establish
stub quality. Do not hand-edit `_native.pyi` or create a second hand signature surface. Stay at
0.29.2 unless exact-release investigation shows a necessary gap; a PyO3 0.30 move is a pinned-
family major requiring its own ADR route, not an incidental stub fix.

### Keep row construction and document decoding distinct

CBO-043 is retained deliberately. Generated attrs rows validate exact bool type, finite floats,
integer ranges and identity type at construction, and preserve the `_pse_equality_key` contract
(including equal signed zeros). `Declaration`, `FitDeclaration` and public `structure_rows`
make this requirement observable. Frozen msgspec document Structs validate at decode; their
constructors do not enforce annotations, and conversion of an existing target Struct does not
revalidate it. The retained probes at 0.21.1 also show `convert` accepting NaN/infinity.
Replacing rows with Structs would move failures away from construction and can admit `is_override=1`.
No such migration is selected.

Keep cattrs' public fresh converter, strict extra-key hooks, exact scalar/identity adapters and
thread-owned private row transport reuse under ADR-0178. Private transport cannot bypass required
raw/typed version or finite-value checks. External schema-to-Python generators, cattrs msgspec
preconfiguration and a decode-only row fork do not remove the two contracts. The closed schema
emitter retains refusal of unsupported schema constructs, registry enums, frozen types and no
`Any`. Reconsider it only if a candidate actually preserves the full contract and the Python-
boundary decision is explicitly reopened. I06 carries focused regression coverage where I01/I03
touch these seams rather than launching an unrelated model-system rewrite.

### Generated evidence types at the real publication boundary

U3 explicitly keeps SurrealDB canonical for tooling evidence and knowledge. Retain the private
tooling database, immutable evidence generations and source cuts, knowledge reconciliation/CAS,
roles, graph/full-text/cursor behavior, and the separate offline evidence and source-only knowledge
routes. No SQLite/DuckDB/Parquet authority, second service or all-one-language port is selected.
Python adapters read Python-owned records; Rust owns registry declarations, the workspace
DataFusion universe and canonical execution. The seam itself is sound.

I04 removes actual declaration restatement: `evidence_projection.py`'s `EvidenceFact`/
`EvidenceSnapshot` TypedDicts and the hand knowledge-mode choices. Import the existing generated
types from `scripts/generated/evidence_snapshot_input.py` and `knowledge_query_input.py`, and use
generated `StrEnum`s for CLI choices. Keep argparse's actual flag declaration; it is not a
duplicate schema merely because it names a field. The evidence question document has no generated
Python binding today; its operation choices are a retained bounded gap, not something I04 can
pretend already exists. Generating that binding is optional future registry work if a real
consumer justifies it, not required for this consolidation.

Admit **raw nested mappings**, before constructing Structs, with strict typed msgspec conversion
or typed decode of the exact publication document. Choose strict conversion of raw mappings as
the initial route, with a small adaptation boundary shared by evidence source producers; raw facts
must remain raw until the conversion traverses them. Refuse preconstructed Struct members on this
raw-input route, rather than allowing a mixture that conversion can leave unchecked.
`EvidenceSnapshot(... facts=(EvidenceFact(...),))`
followed by `convert` is not validation. A wrong nested string/bool/float and an already-created
invalid Struct must not bypass publication admission. Check finite numbers before any encoder
can map NaN/infinity to JSON null. Preserve original owner completeness, unreported/failed members,
redaction and source digest meaning; the digest consolidation itself belongs to the host-file
owner (CBO-047).

Generated Structs expose tuple collections and explicit optional `None`; current producer mappings
use lists and omit optional keys. `resource_store.publish` accepts a mapping and uses `json.dumps`.
I04 therefore selects a bounded Struct-to-JSON-compatible-mapping adaptation before the **existing**
atomic publisher, preserving absent optional fields and emitted arrays. Do not add a competing
byte publisher or restate fields manually in another class. Use library builtins conversion plus
the declared omission convention, with exact before/after wire fixtures for absent/present/null
and nested cases. Migrate readers from dict subscripting/mutation to the admitted representation
or explicit mapping boundary together; all source adapters in `evidence_sources.py`, invocation
projection, store/offline/analyze paths and `knowledge_store.py` participate.

Rust decode remains final structural/domain enforcement. Generated `KnowledgeQuery.limit` says
`ge=0`, not the full **1–64** rule; generated annotations and import tests are not a claim of full
runtime domain validation. Preserve Rust refusal of zero/65, wrong versions, stale cursors, wrong
cut/store and unauthorized relations. `check_private_contracts.py` establishes candidate imports/
annotations and currently includes both msgspec and pydantic; it neither validates instances nor
proves a msgspec-only closure. Source-only pydantic models represent a separate surface and stay.
Coordinate Rust tooling SDK/transport consumers with the operations companion if its owner moves
the vendored SDK; U3 forbids shrinking that migration by deleting canonical tooling consumers.

### Trace capture: supported HTTP/2 transport and explicit incompleteness

CBO-051's reopened sink choice is settled here by current consumers. `mcp_native_acceptance.py`
and `plan52_f02_acceptance.py` reject `finite_capture_incomplete` using empty `capture_errors` and
zero omitted spans. Yet `surreal_telemetry.Receiver` silently returns on EOF/socket errors and
drops pending streams on RST_STREAM/GOAWAY, ignores request headers and can therefore report no
error after losing input. `telemetry_database_acceptance.py` checks only workload success and
positive bytes. Merely describing this sink as best effort leaves those consumers' stronger
inference unsound.

I05 selects pure-Python **h2 4.4.1** for HTTP/2 state/HPACK/flow-control ownership, retaining
socketserver, selected-service lifecycle, absolute deadlines, capture nonce, finite input/span/
stream bounds and identity-only observation locally. Use server-side `H2Connection` with
`H2Configuration(client_side=False)`, receive events through `receive_data`, acknowledge flow-
controlled data and drain `data_to_send`; send legal response headers and gRPC trailers through
the connection. This choice follows exact-tag [connection](https://github.com/python-hyper/h2/blob/v4.4.1/src/h2/connection.py)
and [event](https://github.com/python-hyper/h2/blob/v4.4.1/src/h2/events.py) interfaces after Context7
returned the unrelated Rust h2 project. No live peer experiment has run for this plan.

Local gRPC framing, bounded OTLP identity-field decoding, data/attribute redaction and service
restore/drain stay with `surreal_telemetry`. Validate the actual Export method/path, content type,
compression refusal, message lengths and successful completion before acknowledging an export.
Keep the existing field walk rather than adding protobuf imports merely to remove about 25 lines;
the deliberately raw test client still consumes `grpc_wire` independently. Unknown protobuf
fields remain tolerated according to the existing bounded identity extraction contract; HTTP/2
validity does not establish complete distributed causality.

Add explicit accepted/completed versus incomplete/rejected transport accounting, preserving
successfully decoded identities alongside failure. EOF, timeout, reset, GOAWAY and protocol/header
errors with pending header/data/message state must yield incomplete capture; a clean connection
close after all accepted exports completed is different. Snapshot the result only after owned
handlers drain; no reader may infer completeness while a handler can still append spans/errors.
Retain bounded diagnostics and an aggregate incomplete flag/count even when the detailed error
list is capped. Bound wire input and parser buffering/header state before expensive retained
allocation, not only inside `Capture.observe` after a whole stream arrives. Preserve or tighten
existing fixture bounds (16 MiB total, 4,096 spans, 1 MiB frame ceiling, 8 MiB stream, 32 streams)
under the actual h2 state machine; these are diagnostic fixture bounds, not workstation resource
partitions. Explicitly handle simultaneous streams and flow-control padding accounting.

Migrate all three named acceptance consumers and capability/diagnostic response consumers to the
new explicit completeness meaning. Empty error arrays or a nonempty trace no longer stand in
for it. A completed local capture means accepted exports completed under the finite capture
protocol; it does not prove the server exported every event, end-to-end trace propagation,
absence of sampling, complete causality or total distributed visibility. Preserve restoration
and workload outcomes separately when capture itself fails.

The raw HTTP/2 client in `scripts/tests/surreal_mx_check.py` remains intentionally raw: it must
abruptly drop a connection without an SDK destructor masking cleanup defects. h2 selection for
the sink supplies no reason to replace that test peer. grpcio's compiled runtime/threads and an
optional collector's process readiness/flush/port lifecycle cost more than this finite sink;
requiring the optional collector would reduce availability. Switching SurrealDB 3.3.4 to OTLP/HTTP
is unavailable: its pinned server compiles tonic without `http-proto`. Add the brief §F reason
and alternatives to `surreal_telemetry`'s module docstring where the next reader will look.

U1 includes a conditional upstream protocol-selection contribution: a released SurrealDB exporter
supporting `OTEL_EXPORTER_OTLP_TRACES_PROTOCOL`/HTTP protobuf could later replace the sink's HTTP/2
layer with stdlib HTTP. No contact, issue, patch submission, future release assumption or present
wait is authorized. The current h2 target must stand without it. A msgspec upstream option for
required unique header-first direct tags is another conditional lead; it must demonstrate the
full admission contract before replacing traversal, and Rust remains the authority.

### Retained tooling choices and small route inquiry

CBO-049 already has one maintained owner declaration in `test_requirements.py`; a per-run selected
nextest catalogue derives scheduling, guards, fixtures and binaries. The scheduling and JUnit TOML
copies are derived views. Keep exact selected-test/group closure, caller config specialization,
longest-prefix precedence and fixture admission. A tool-config overlay cannot reference later
repository-defined groups under the inspected nextest source ordering, and would still leave the
JUnit copy; static prefix overrides would split authority. Nextest setup/wrapper scripts lack the
required reconciled teardown lifecycle. `cargo_feature_check.py` is the cargo-hack adapter and
composes explicit Arrow force-validation per closure; replacing it with cargo-hack deletes a
required boundary rather than removing duplication. No replacement is scheduled.

I06 resolves the source review's residual route question: `just test`, `test-package` and preset
recipes call `scripts.arrow_validation nextest` directly, while `unit-package` and `test_run` use
the selected catalogue. Read their actual callers, advertised modes and canonical fixture refusal
to decide whether that intentional thin route promises canonical tests or deliberately requires
the prepared route. If it promises preparation, adapt the real call path/consumers under the
existing catalogue; if it intentionally refuses, document the supported command distinction at
the existing owner and test the refusal. Do not add a second selection declaration or convert
every recipe to a framework. The review read nextest group behavior from upstream main, not
exact 0.9.146; exact-pin confirmation is required only if this inquiry changes scheduling or
reopens overlays. No nextest upgrade or alignment lint follows automatically.

CBO-052's recipe/script/MCP/clap surfaces have distinct responsibilities. Keep thin opaque recipe
argument forwarding, explicit maintenance `--state` requirements, stdlib-only cold hooks/bootstrap,
generated capability CLI from canonical schemas and discovery derived from `just --dump`.
`--state` and `--native` homonyms are not a shared type. Cyclopts across dozens of argparse scripts,
just `[arg]`/modules, nextest profiles, Typer and FastMCP `generate-cli` do not remove the current
meaning owners. The real repeated generated request choices are handled by I04. A just variable
for a repeated forwarding prefix is optional cosmetic work only when naturally touched; its
absence is not incomplete scope and it must preserve positional data and admission visibility.

Retain CBO-051 front-matter owners: strict `document_metadata.read` with line-exact delimiters,
duplicate/merge-key refusal, ADR delegation, the fail-closed stdlib edit guard's status-only read
and the reference checker's body stripper. Their different cold-path duties do not justify
python-frontmatter or a shared heavy import. Revisit only an actual delimiter/status divergence,
not superficially similar text handling. Preserve the review's current-ADR static comparison as
bounded historical evidence rather than a new recurring alignment check.

## Decision prerequisites and dependencies

| Item | Planning state | Dependent work and route |
| --- | --- | --- |
| CBO-044 public type ownership / RC01 | Direction confirmed 2026-10-27: include conditional native-public-type target | I02 inventories before an ADR plus formal review; I03 waits for the accepted target and authority amendments to blueprint §21/§21.5. No existing ADR is changed here. |
| Mutation/constructor/helper changes | Unresolved explicit public-contract choices | Settle each in I02 and include consequences in the same boundary ADR/review before its family migrates. Retaining a bounded facade is allowed where its semantic work remains. |
| Trusted ingress interpretation | Preserve untrusted-only admission distinction | I01 is within the existing contract; if authoritative wording is judged to require the extra check on every decode, obtain the coordinator's short clarification ADR before removal. |
| U3 canonical tooling store | User confirmed retain SurrealDB | I04 has no store replacement branch; canonical SDK/transport consumers remain in the operations migration footprint. |
| U2 recorded replay points | User confirmed full recorded-point replay | All migrated run/study/replay classes retain actual recorded points, original order/coordinates, evidence and scope; seed-only replay cannot satisfy public migration. Science owners supply the exact replay contract. |
| U1 upstream work | Include conditional opportunities; no contact | Retain msgspec and SurrealDB protocol leads with released-capability triggers; neither blocks selected local work. |

I01's per-type cached bridge is a prerequisite slice for I03's typed-native signatures, not a
dependency on the entire Python migration. I04 and I05 are independent of the final public-class
contract decision and of 57h's registry blob. H01/H02 diagnostic source privacy must remain intact when I03
moves exceptions/types; H06 and I01 share document generator files and require one coordinated
edit. Data/stream buffers remain under 57a; changing class names must not weaken receiving,
allocation or independent chunk ownership. Signal-aware waits share native methods with the
execution/observation owner; do not rewrite its chosen helper in a parallel facade.

## Package state

| Package | Status | Owner | Completion |
| --- | --- | --- | --- |
| [I01](#i01) | pending | Common Rust/Python document bridge and generator | Trusted immediate producer route, per-type handle caches, existing thread-owned decoder reuse, single untrusted Rust header check |
| [I02](#i02) | pending | Public Python contract owner with native/stub consumers | Complete family inventory and conditional ADR/formal review resolving public types, indexed views, identity, mutation, construction/MRO and helpers |
| [I03](#i03) | pending | Accepted public-family producers and all Python consumers | Conditional native public cutover with documented residual facades, bulk returns, typed native inputs, regenerated documented stubs and immediate deletion |
| [I04](#i04) | pending | Evidence/knowledge Python adapters and Rust admission | Generated type adoption with real raw-input validation, finite and optional/list adaptation, existing atomic publication and unchanged canonical semantics |
| [I05](#i05) | pending | Finite diagnostic capture and all completeness consumers | h2 sink, explicit incomplete accounting, bounded protocol/lifecycle controls and independently retained raw abrupt-drop peer |
| [I06](#i06) | pending | Selected-test routes and boundary regression owners | Resolve direct-nextest recipe promise, preserve row/document contracts and retained CLI/front-matter judgments with concrete triggers |

### I01

Change `pse-py/src/documents.rs`, the actual generated catalogue producer, Python codec private
helper and Rust `Wire` consumer together. Targeted tests distinguish trusted and public paths:
missing/equal-duplicate/wrong/boolean version, malformed nested body with wrong header, Native
versus flat envelopes, allowances, unknown fields, identities and finite-number refusal. Verify
input/output direction cache separation and two distinct document types in one process; exercise
multiple threads/cache eviction without sharing mutable decoders or introducing per-call schema
preparation. Use producer-output assertions that require headers rather than decoding through
the trusted path as the only oracle. Keep import fingerprint mismatch refusal and public JSON/TOML
round trips. Delete redundant flat pre-scan and producer-side header callbacks, not untrusted
traversal or the native header authority. Regenerate catalogue outputs through the generator.

### I02

Complete the map above against public exports, native implementations and generator/stub sources.
Classify forwarding-only, indexed view, Python-constructed owner, Python helper/value/protocol and
generated mixin roles; do not use class counts as evidence of semantic parity. Resolve the explicit
alternatives in the boundary ADR/review, with actual constructor/MRO, repr, equality/hash, mutation,
iteration/context management, method/property and public import/name decisions. Identify every
residual facade/helper and what it still owns. Validate proposed public stubs at the exact pinned
PyO3/introspection API; a claim that 0.30 is required needs evidence and a family decision first.
Deliver an actionable family cutover sequence, explicit retained-facade decisions and qualification
cases to I03. If the accepted review cannot support a family migration, record the limitation and
its dependent scope at the coordinator. This is a bounded design package, not a new broad audit.

### I03

After I02's accepted contract and I01's actual bridge, migrate coherent complete families with
their native return/input producers, `_build`, public exports, annotations, help text and tests.
Provide the decided compact indexed view before changing `StudyReport.result`, preserving original
occurrence completion and bounded detail refusal. Migrate signal waits and stream/buffer behavior
with their semantic owners. Remove each replaced facade, unwrap, repeated docstring and obsolete
native implementation name only when every actual caller has moved and targeted behavior passes;
do not keep compatibility aliases/forwarding paths. Remaining genuine Python helpers are explicit
target owners, not legacy retained as evidence.

Test class/module/import identity, constructors/defaults, attribute/method shapes, exact decided
mutation exceptions, handle identity equality/hash and views, `repr`, Python-only helpers,
iteration/context exit/close, nested tuple order and live/cancel/refusal outcomes. Include public
recorded-point replay and `RunResult`/`StoredResult`/`CompactStudyResult` selection, including
unattempted/partial states. Regenerate stubs and inspect signatures/docstrings/re-export usability
against actual runtime classes. No broad deletion percentage or performance benefit is promised;
crossing reduction applies only to operations actually migrated to bulk eager results.

### I04

Migrate `evidence_projection`, `evidence_sources`, `evidence_store` and `knowledge_store` together
with actual mapping publisher and Rust request readers; generated declarations already exist and
must not be hand-edited. Preserve producer source-reading/redaction and publication ownership.
Tests supply raw malformed nested input (including an invalid already-constructed Struct bypass
attempt), NaN/infinity, bool/integer confusion, unknown/missing fields, absent versus null optional
facts and tuple/list wire adaptation. Compare actual emitted JSON and canonical Rust identity,
not merely Python object equality. Cover evidence complete/partial/unreported populations,
offline versus indexed question agreement and knowledge modes/limit 0,1,64,65, stale cursor and
cut/store mismatch. The tests for canonical behavior join final integration timing; package work
uses targeted adapter/domain unit controls. Delete TypedDict twins and generated-enum choice
restatements immediately. Keep question flags with no generated binding and source-only models.

### I05

Add the exact h2 pin to the environment group that actually executes capture, update only its
required lock closure, and confirm capture's declared capability/environment includes it; no cold
hook acquires the import. Run `just ready` after the dependency/environment change under the
repository timing rule. Implement h2 event-driven transport and receipt accounting, migrate all
completeness consumers and add the brief bespoke-code reason. Delete the sink's manual HTTP/2
frame writer/parser and static HPACK header bytes. Keep raw-client `grpc_wire` ownership and its
purpose comments; OTLP identity decoding remains bounded local work.

Targeted tests use legal fragmented HEADERS/CONTINUATION, DATA split across reads/messages,
multiple concurrent streams, SETTINGS/PING, flow-control accounting, invalid path/content type,
compression, oversized headers/frames/messages, span overflow and total byte limits. Exercise EOF
mid-preface/header/data, partial gRPC message, RST_STREAM, GOAWAY and deadline while pending, plus
clean idle close after completed exports. Failure receipts retain decoded spans, incomplete status,
workload and restore results; capped errors cannot erase incomplete status. Verify handler drain
before publication and restoration on operation failure/cancellation. The real pinned tonic export
and both stricter F02 consumers are assembled acceptance under 57j, not something a byte fixture
alone establishes. The deliberately raw abrupt-drop cleanup journey remains independently required.

### I06

Resolve the direct-nextest recipe question from actual advertised/caller scope and canonical
fixture contract, recording either supported prepared routing or intentional refusal at the
existing test-route owner. Preserve caller config capacities, selected-catalogue closure, fixture
guard refusal and Arrow force-validation; if a route changes, targeted tests must detect missing
owner/group declarations and unchanged argument passing. Confirm exact nextest pin semantics if
relevant to a proposed change rather than inheriting main-source behavior as release proof.
Exercise preserved public row construction versus document decode under affected Python changes,
including exact bool, signed zero, nonfinite values and required versions. Carry CLI/front-matter
no-change judgments into closeout with the observable triggers below; no alignment lint or broad
framework conversion is introduced. Cosmetic recipe factoring remains optional and cannot defer
the actual selected-route decision.

## Retained inquiries and reopening conditions

| Inquiry or retained alternative | Evidence needed / trigger | Work constrained |
| --- | --- | --- |
| Full CBO-044 family realization | I02 inventory, exact-pin stub feasibility, accepted public behavior choices | Only the corresponding I03 family waits; no blanket wrapper deletion claim |
| `CompactStudyResult` native view versus small facade | Preserve report Python-handle identity, index, lifetime and frozen public behavior with discriminating cases | Study public cutover |
| Native Runtime versus composition facade | Actual constructor/MRO/frozen/returned-instance and generated host-operation lowering | Runtime/mixin cutover |
| Source-local render method on retained packages | H02's authenticated id-to-text source chain and source privacy check | Optional public method joins the boundary decision |
| Tool-config overlay | nextest gains cross-source groups or JUnit override that genuinely removes a copy, or measured large-selected catalogue cost justifies change | Retain current until then; no automatic benchmark |
| Alternative tooling store | U3 retains canonical SurrealDB; no present branch | A later explicit operator scope change and concrete consumer would be needed to reopen deployment |
| OTLP/HTTP or msgspec header upstream capability | Released exact capability demonstrated against the full local contract | Future local simplification only; no upstream contact now |
| Front-matter/CLI consolidation | Actual owner divergence or a coherent repeated meaning, not shared spelling/count | Reopen only the affected owner; preserve fail-closed/cold-path and explicit maintenance selection |

## Verification and handoff

**Proposed:** the targets and package tests above are future work. This authoring used source,
review/probe evidence, Context7 and exact-release primary source inspection; it ran no product
tests, build, dependency resolution, provider/peer workload or live capture. The retained msgspec
probes are **Tested** only at their documented 0.21.1 conditions, not proof of new bridge/adapters.

During implementation use pinned supervised package compilation and targeted unit/Python tests,
explicit Arrow force-validation where consumed, and full generation as part of generator edits.
Report command/mode/scope and actual failures against zero. [57j](57j-qualification-and-publication.md)
owns integrated native/Python/host/canonical journeys and scope-end static/manual qualification
after all functional work; package-local success does not qualify public or assembled behavior
that has not been exercised. The coordinator owns turn-end formatting, commits and shared index.

At close, reconcile actual residual facades, decision outcomes, generated consumers and deletion
with the coordinator's ledger. Move enduring public-boundary/tooling/diagnostic explanations to
their current authoritative owners through the authorized route, then retire completed planning
material under ADR-0096. Current outcome is **Proposed**; no implementation acceptance, decision
acceptance or measured benefit follows from this draft.
