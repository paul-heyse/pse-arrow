---
title: "Plan64/65: Data, results and evidence integration"
status: proposed
date: 2026-11-06
doc_role: review
doc_owner: docs/plans/65-proto-integrated-target.md
doc_topics: [design, libraries, storage, diagnostics, validation]
---

# Data, results and evidence integration

## Judgment and inspected boundary

**Proposed:** organize the remaining data work around protected scientific queries,
cohort analysis and original-evidence delivery. Reuse the existing native store, DataFusion
providers and original-file owners more fully; do not organize delivery around completing
every generated representation first. The strongest amendments connect selection to actual
hydration, make retained interpretation a reusable family responsibility, and provide useful
bulk evidence access without repeating whole-member verification for every small transfer.
An ephemeral analytical view over existing evidence is a worthwhile additional capability.

This supports the combined [Plan64](../plans/64-process-simulator-target.md) and
[Plan65](../plans/65-proto-integrated-target.md) improvement review. Its principal footprint
is [64g](../plans/64g-analyses-studies-and-applications.md) G08 and analytical consumers,
[64h](../plans/64h-reference-coverage-and-qualification.md),
[65c](../plans/65c-storage-projections-and-history.md),
[65f](../plans/65f-diagnostics-and-result-delivery.md) and
[65j](../plans/65j-workflow-lifecycle-and-developer-evidence.md), with A/B generation,
scientific product semantics, project portability and native settlement as adjacent contracts.
Only Plans64/65 supply remaining work. These recommendations do not amend their status,
adopt dependencies or authorize implementation.

The [principal review](plan64-65-integrated-target-and-library-leverage-2026-11-06.md)
owns the combined judgment and recommendation relationships.

**Interface-checked:** the principal source baseline is main
`6045168110f5f5948df43c4f642ea1737751675b`. Source inspection used the persistence checkout
at `57bb5ec6c` and Git reads of main. The selected-result declarations, aggregate fixture
readiness and duplicate IPC metadata admission are integrated on main; the metadata consumer
and default-feature module wiring remain uncommitted proposals in the persistence checkout.
That candidate references an un-emitted root-scoped adapter. The contracts worker's later
scientific inspection regeneration/split is unfinished. Neither candidate compilation nor
whole-operation acceptance is inferred. Root-reported earlier functional results retain their
own commands and conditions; this review ran no builds, tests, benchmarks, database queries,
preparation or other workloads.

The standard is core **3.4**, efficient-architecture heuristics **1.0**, process-simulator
profile **1.5**, test-assessment companion **1.2**, and the current repository binding selected
by [standard.toml](../design_review/design_principles/standard.toml). The full applicable
standard informs the judgment; AP-04/AP-07, DP-01–DP-10, DP-13–DP-24 and PS-11–PS-13 are
especially consequential here. The enduring owners include blueprint §19.2/§19.3,
§20.5, §21.1 and §23/§24. This is an improvement review, not a refreshed diagnostic audit
or a plan-completion audit.

## Existing strengths and exact library boundary

The code already contains substantial useful mechanisms:

- [`result_snapshot.rs`](../../crates/pse-runtime/src/workflow/result_snapshot.rs) and
  [`cohort_provider.rs`](../../crates/pse-runtime/src/workflow/cohort_provider.rs) expose
  protected native results through DataFusion. They check the deployment memory-pool identity,
  retain residual filtering and avoid an early limit when filters remain inexact. A second
  generic cohort provider would duplicate an existing integration.
- [`canonical_capture_results.rs`](../../crates/pse-operations/src/canonical_capture_results.rs)
  batches bounded locators across exact captured members and shares the original capture
  protection. Missing relation membership and incomplete traversal have explicit meaning.
- [`canonical_results.rs`](../../crates/pse-operations/src/canonical_results.rs) and
  [`canonical_files.rs`](../../crates/pse-operations/src/canonical_files.rs) preserve original
  attempt/manifest membership, exact physical files and release obligations. A codec does not
  replace their performing authority.
- [`developer_evidence/directory.rs`](../../crates/pse-host/src/developer_evidence/directory.rs)
  supplies staged original-file delivery over the same report commitments as compact export.
  Open descriptors preserve inode access, with no atomic-snapshot or general pruning-lease claim.
- [`canonical_explanations.rs`](../../crates/pse-operations/src/canonical_explanations.rs)
  already exposes bound graph-query plans and optional executing analysis. Its scope explicitly
  limits conclusions to available operator evidence.

The overview corpus is a capability-discovery input, not the installed API contract.
`analytical_infrastructure/datafusion.md` covers 55.2, `arrow.md` covers 60 and
`object_store.md` covers 0.14.2. Cargo currently pins **DataFusion 55.1.0, Arrow 59.3.0,
object_store 0.13.2, Schemars 1.2.2 and petgraph 0.8.3**. The server owner selects patched
**SurrealDB 3.3.4 / SurrealMX 1.0**, including connection/session admission, typed failure,
rollback and module-controller patches. Current v4 query operators, materialization or wire
features are not proposed as if available on that baseline.

Context7 was consulted first for DataFusion, object_store and SurrealDB. Its responses include
current-main examples, so consequential contracts were checked against pinned installed source
and versioned primary documentation. SurrealMX resolution returned an unrelated MCP library;
the maintained 1.0 source and repository briefing were used instead. Missing Context7 coverage
is not evidence of missing capability.

<a id="DRE01"></a>
## DRE01 — Carry useful selection into the existing physical readers

**Proposed amendment; C02/F04 and 64g analytical consumers.** Complete an operation-shaped
selection contract across the existing snapshot and cohort providers, native canonical query
planning and Arrow decoding. Preserve source identity, requested coordinate window, scientific
permission inputs, returned extent and scan completeness as separate facts. The metadata
cutover under proposed ADR-0250 is useful but does not itself reduce underlying hydration.

**Interface-checked basis:** snapshot `scan` receives projection/filter/limit, but
`SnapshotPartition` obtains the checked batch through the snapshot before exporting it to the
stream. Readable inspection has a separate projection-aware decoding route. Cohort scans
already retain an immutable scientific output selection and original availability classification.
The useful intervention is to connect these existing capabilities where their assurances agree,
not to replace them with a fetch-one abstraction or a new relational engine.

DataFusion 55.1's `TableProvider` contract distinguishes exact, inexact and unsupported
pushdown; residual filters can require fields absent from the final output. Its pinned
implementation is re-exported from `datafusion-session`, rather than implemented in the short
`datafusion-catalog::table` shim. Arrow 59.3 `FileDecoder::with_projection` supplies physical
root projection. These APIs enable delegation; they do not prove that dropping a column preserves
relation-wide keys, predicates or scientific assessment.

Compare three complete realizations: retain full checked native scans for consumers requiring
full admission; use the existing narrower inspection contract for readable subsets; or derive
an explicit required-root closure for a protected analytical scan and push it into decoding.
Recommend the third only for consumers whose required guarantees can be retained without
presenting a projected batch as a fully admitted original relation. Keep full scans as the
deliberate route for full validation, not as an accidental prerequisite of every small answer.

Migration includes both providers, selected/full/empty outputs, predicate dependencies and
consumers such as review packs and project bundles. Delete duplicate selection/hydration passes
when the replacement carries equivalent scope and custody; retain no second active provider
with the same responsibility. Acceptance should challenge an inexact filter plus limit,
predicate-only columns, composite roots, corrupt omitted roots, sparse ranges, withheld
covariance/dual permission, cancellation and escaped buffers. Measure complete query-to-release
work before claiming savings. This improves AP-07, DP-08/DP-10/DP-13 and PS-12 without treating
a small returned page as proof of small examined work.

Primary contracts: [DataFusion 55.1 TableProvider source](https://docs.rs/crate/datafusion-session/55.1.0/source/src/table.rs),
[Arrow 59.3 FileDecoder](https://docs.rs/arrow-ipc/59.3.0/arrow_ipc/reader/struct.FileDecoder.html).

<a id="DRE02"></a>
## DRE02 — Offer verified bulk member delivery beside strict bounded inspection

**Proposed amendment; F03/J04/J05.** Add an explicit sequential original-member transfer
operation using the existing opened-file commitment owner. It should stream bounded chunks,
retain the original descriptor and verify the complete promised member before returning a
completed-member receipt or publishing a destination. Partial transfer stays partial. Keep
strict independently verified range inspection for callers needing that stronger per-call result.

**Interface-checked basis:** `OpenedFile::read_range` supplies the original whole-member digest
to `read_descriptor_borrowed`; that owner scans the whole member before reading the selected
range. Directory acquisition also verifies original members. Repeated small range requests can
therefore repeat substantial verification work. This is a structural observation, not a measured
latency defect. The existing `OpenedFile::with_reader`/`copy_verified` path already computes
the digest over the transferred original stream, validates growth and before/after identity,
and is a better foundation than another checksum or chunk-authority implementation.

Compare retaining current strict range access, a verified streaming transfer, and an explicitly
sealed immutable copy for repeated random access. Prefer the streaming operation for whole-member
delivery: it avoids a new full-closure materialization or retention manager. A sealed copy earns
its additional storage and lifecycle only for a real repeated-random-access requirement. An open
file descriptor, read-only mode, ETag or one successful digest check does not establish immutable
future bytes; do not cache the verification conclusion under those premises alone.

The directory publisher already copies and validates members efficiently. This recommendation
extends acquisition/readback for large consumers; it is not a claim that directory export itself
currently uses the inefficient range loop. No service chunk transport is required for an offline
sink API. If asynchronous I/O is needed, Tokio-util/Bytes may carry chunks and their backing,
while the original owner continues to establish meaning, attribution and completion.

Accept through a member above compact limits, slow/escaped readers, interrupted prefix,
same-inode mutation, missing later member, original omissions, destination collision and final
sync failure. Verify that no prefix receipt means whole-member success. Remove any replaced
full-member copies or alternate hash/adoption code together with their callers. DP-19/DP-21,
H16–H20 and the test-assessment companion support this scoped intervention.

<a id="DRE03"></a>
## DRE03 — Make retained interpretation a family-owned acquisition, not repeated metadata lookup

**Proposed realization of the existing C03/C04 obligation; adjacent A/B/I/K.** Resolve the
descriptor world, original interpretation, validation policy and mapping dependency actually
promised to a selected reader once under its native acquisition. Carry that resolved context
through the read. Do not attach every possible dependency to every row or keep a live fixture,
worker or solver ready merely because its historical output is readable.

Current native file validation binds `FileDescriptor.interpretation` to the current native
interpretation. The selected-result context now records original problem/revision/manifest
and interpretation, but those strings alone cannot supply an unavailable historical reader.
Likewise, result-manifest `descriptors` identify result-set membership; that name must not be
mistaken for a complete retained Protobuf descriptor world. The unfinished ADR-0250 consumer
is a local exact-serialization comparison, not historical scientific admission.

Recommend a family-specific acquired read context inside existing `ResultRead`, artifact or
developer-report custody, with generated structural metadata where useful. Reuse Buffa 0.9.2
descriptor loading/reflection for passive acquired messages and A/B's interpretation checker.
Keep contextual scientific re-admission and original protected-read permission native. A missing,
expired or corrupt dependency is an explicit incompatible/incomplete read, never a silent
substitution of today's policy or an empty successful result.

Compare immutable original-world acquisition with an explicit migrate-and-reactivate operation.
Retain old interpretation only for a promised reader; a regenerable unretained projection may
instead be rebuilt. Stage mapping changes privately and publish one coherent activation. Separate
mapping-only change from physical-datum/provider/derivative changes that invalidate scientific
products even when wire shape is unchanged. Delete duplicate interpretation headers/lookups and
old production writers after complete consumer migration; historical readers retain their named
promise. Acceptance includes old policy unavailable, malformed descriptor closure, foreign
experiment/attempt, interruption around activation and uncertain acknowledgement. This is
DP-01/DP-04/DP-09/DP-24 and PS-11/PS-12, not a new generic evidence database.

<a id="DRE04"></a>
## DRE04 — Reuse native database planning and investigate narrower capture conflicts

**Proposed amendment; G08/C02/C04/C07/F05.** Make current query-plan inspection and the
original operation's selection/capture facts useful together before adding a maintained view,
adjacency cache or external changefeed. For exact study/project capture, investigate whether a
complete dependency domain can narrow the conservative global publication witness. Keep broad
selectors global where that is their actual dependency.

**Interface-checked basis:** canonical graph inspection already explains the same bound query,
optionally executes its inner plan, checks the original anchor, and labels operator-evidence
limits. Current capture uses one authored transaction and exact-request receipts. Every relevant
writer must continue to cover insertions, removals and absent/future membership. The earlier
guest capture hit its execution ceiling; the authored transaction is the implemented foundation.
The bounded legacy settlement code is required by predecessor migration and is not an obsolete
ordinary write path to delete simply to simplify a source search.

Compare current guarded queries/global witness, scoped witness domains generated from complete
selector dependencies, and native engine dependency tracking. Recommend the scoped-domain
investigation for concurrent independent studies, not immediate removal of the current witness.
SurrealMX 1.0 offers transactional and snapshot mechanisms; its engine API does not establish
which predicate dependencies the patched SurrealDB query path records. `FOR UPDATE` on an
existing record alone does not demonstrate phantom/future-member protection. The exact patched
bridge must establish that boundary before any guard is removed.

The tooling corpus also makes SurrealKit rollout/schema comparison, SurrealQL language
intelligence and engine diagnostics worth considering at the existing generator/migration
boundary. Their role would be library-owned parsing, comparison or inspection beneath the
current source declarations and guarded receipts. A tool's second editable schema, migration
history or permissive query endpoint would add an authority instead of removing machinery.
Before selecting a tool, compare its exact 3.3-compatible artifact and rollback/read-only
semantics with `canonical_migration` and the original module receipt. No SurrealKit release
or API is qualified for adoption by this bounded review.

Use existing source-owned plan inspection rather than install an unrestricted database MCP or
create another verdict ledger. Current documentation can describe v4 operator shapes; use the
actual patched 3.3 plan output and retain raw evidence/coverage limits. New execution evidence
must correlate the original bound selection, native operation and actual database statement,
not infer cache misses or inner-function costs from absent outer-plan counters.

Acceptance contrasts concurrent unrelated genuine study updates with same-domain future inserts,
selector metadata changes, lost ACK/replay, expired/stale owner and paged abandoned cleanup.
Compare contention only under matched complete operations; no throughput claim follows from
this review. C05/C06's no-extra-graph/no-durable-continuity decisions remain valid for their
examined consumers. A named continuity or path-analysis promise can reopen them without making
broker/changefeed infrastructure a prerequisite for ordinary reads. AP-07, DP-07/DP-09/DP-19
and H13/H22/H24 govern the comparison.

Primary source: [SurrealMX v1.0](https://github.com/surrealdb/surrealmx/tree/v1.0.0).
The repository's [3.3 storage guide](../libraries/proto/Protobuf_SurrealDB_v3.3.md) and
[`server.rs`](../../crates/pse-host/src/server.rs) define the relevant source/patch boundary.

<a id="DRE05"></a>
## DRE05 — Add analytical evidence views without moving verdict authority

**Proposed enhancement within F01/F05/F06/J02/J05 and 64g/H consumers.** Expose typed,
source-linked observations from one acquired immutable evidence set as an ephemeral Arrow /
DataFusion relation for useful joins, grouping, ordering and comparisons. A concrete consumer
is explaining repeated preparation refusals and failure/capture/settlement patterns across an
explicitly selected set of original invocations, correlated with scientific run/attempt/product
facts where those original associations exist.

The current [`review_analysis`](../../crates/pse-runtime/src/tooling/review_analysis.rs)
already answers ten domain questions over `FrozenEvidenceSet`, retaining exact event/source
references and explicit windows/comparability. Keep numerical-history interpretation and
scientific permission in those native owners. Do not translate every small owned semantic fold
into SQL. The opportunity is to let the existing engine own generic cross-record analytics for
new questions and share the factual projection with F's sink migration, rather than add another
hard-coded classifier or persistent report store for each question.

Compare the current bounded Rust folds, a typed ephemeral relation and an indexed persistent
analytical history. Recommend the ephemeral relation first: generated field correspondence can
serve Proto, Arrow and sink outputs under one source declaration. A persistent history needs a
separate repeated-acquisition/retention benefit. Exact UInt64, timestamps/clock domains,
producer generations, original omissions and unknown coverage must survive. A missing source
does not become zero, a passing expected-refusal test does not become an infrastructure failure,
and readable recorded settlement does not confer present control authority.

Migration absorbs generic repeated fact joins/maps only where the relation replaces them;
fixed domain policy may remain a small native operation. Derive schema with explicit owners,
not sample inference. Schemars 1.2.2's serialize-versus-deserialize contract distinction is useful
for discoverable output-only observations; Serde/JSON map normalization must not be used to
erase duplicate or missing/null distinctions before admission. A Z3 policy model is an eligible
alternative only for a concrete constraint problem that justifies its extraction, solver context
and unknown-result obligations; this review selects no solver-based readiness proof layer.

Accept a fresh independent view of failed tests, expected refusals, zero selection, incomplete
capture, changed source/features, unavailable assertions and unknown cleanup. Include mixed
clock domains and partial evidence windows. Prove no preparation starts on inspection and no
query output grants rerun/scientific permission. Compare complete acquire/query/render/release
work before claiming benefit. This improves AP-04/AP-07, DP-06/DP-13/DP-21 and PS-12.

<a id="DRE06"></a>
## DRE06 — Preserve a coherent extension route for shared archives and richer graph analysis

**Proposed conditional extensions, not local-delivery prerequisites.** The changed artifact and
project-export owners are a suitable place to keep a storage-location-independent immutable
archive boundary. `canonical_files::FileDescriptor` already separates physical instance,
digest, extent, encoding, schema and interpretation from the root path. A shared archive or
independent large-result reader is credible even without a current remote caller.

The SurrealMX/SurrealKV overviews broaden the placement comparison: hot transactional metadata,
durable versioned state and large immutable payloads need not have the same physical layout.
However, the selected server's declared build features are `storage-mem,cli,mcp,surrealism`;
the overview's standalone SurrealKV 1.0 API is not evidence that this server exposes that
backend or its temporal/history contract. Retain the current placement as the local baseline.
An engine or large-value-placement change needs exact supported-backend investigation and a
complete durability/recovery comparison, including active readers and reclamation; introducing
direct KV access must not bypass SurrealDB's record, relationship, authorization and transaction
meaning.

For that consumer, compare the current local descriptor/directory profile with object_store
0.13.2 `get_opts` carrying `range`, `if_match` and `version`, `GetResult::into_stream`, conditional
`put_opts`, and explicit multipart completion/abort. `get_ranges` coalesces default range reads
but has no per-call version parameter; it is not automatically a coherent historical read.
ETags are not the original inode, digest or domain read grant. Multipart completion and catalog
reference publication are different effects; an ambiguous completion must settle the original
operation before a duplicate effect. Abort/drop does not universally clean remote parts.

Prefer one adapter under the selected archive responsibility, not filesystem emulation or a
parallel canonical store. Preserve native manifest/protection authority and publish exact object
references only after the selected durability contract is met. Cloud credentials, backend
semantics, orphan recovery, retention and provider qualification are material continuing costs.
Their principal implementation is separable from ordinary local Plan64/65 completion; record
that boundary instead of quietly making local readers depend on a service.

Petgraph 0.8.3 and compatible rustworkx-core capabilities also merit a bounded richer provenance
or dependency-analysis extension: compute SCCs, reachability or dependency impact over a compact
immutable selected graph while preserving relation occurrence identity and kind. Keep native
process topology, scientific dependency and persisted explanation edges distinct. Do not add an
all-path export or cache merely because a graph library supports it. A named question requiring
more than the current immediate-neighborhood view supplies the expansion trigger and output
scope; cyclic scientific structure still needs its domain resolution semantics.

Primary contracts: [object_store 0.13.2 GetOptions](https://docs.rs/object_store/0.13.2/object_store/struct.GetOptions.html),
[ObjectStore source](https://docs.rs/crate/object_store/0.13.2/source/src/lib.rs),
[MultipartUpload source](https://docs.rs/crate/object_store/0.13.2/source/src/upload.rs),
[petgraph 0.8.3](https://docs.rs/petgraph/0.8.3/petgraph/).

## Joint integration and acceptance

DRE01 and DRE03 form the near-term scientific read migration: actual selection must reach a
physical reader under resolved original interpretation. DRE02 supplies a distinct useful large
evidence operation; DRE05 reuses factual projections while preserving native analytical policy.
DRE04 should inform capture/query placement before new persistent projections or narrower
guards are selected. DRE06 is an explicit extension option whose operational costs must be
judged as a complete intervention, not accumulated automatically with every other proposal.

Carry these through five discriminating whole-operation scenarios:

1. Scientific execution → exact terminal attempt → full/selected/empty/partial read → original
   export → independent interpretation, including late terminal failure and failed release.
2. Multi-experiment study → future member/selector change concurrent with capture → bounded
   cohort query → unavailable-member classification → retained analysis and retirement.
3. Mapping-only revision versus physical-datum/derivative revision → private reconstruction →
   activation interruption/lost ACK → old original read and explicit current re-admission.
4. Conditional/exact readiness → explicit preparation → list/check/rustdoc/test → original
   compact or large member delivery → independent read → changed-source exact rerun.
5. Slow/escaped reader during cancellation/publication failure, with independent operation,
   reap, capture, release and durability uncertainty retained together.

Use existing fixtures and actual native ownership for the effect boundaries. Independent
literal malformed bytes, foreign identities and raw FlatBuffer/Proto cases should challenge
the shared helpers; helper-generated expected values alone are not independent evidence.
Library component tests, focused native controls and full product/scientific qualification
remain different claims. No new measured performance, capacity, durability or scientific
qualification is established here. Adoption, plan revision and any ADR impacts remain with
the existing decision routes and the combined review's coordinator.

## Coverage appendix — fourteen overview files, bounded API depth

This accounts for all seven analytical-infrastructure and seven SurrealDB-family overview
files in the assigned corpus. **Screen** means capability/section and relevant limitation
inspection; **bounded deep dive** adds selected current-code and pinned-source/API checks.
Neither label claims exhaustive API review. **Excluded** records a screened scope boundary,
not a judgment that the library lacks useful capabilities. Sources beneath the Proto corpus
and the maintained Buffa/Arrow sources additionally informed their existing selected boundaries.

| Overview | Depth and use | Concrete unresolved lead |
| --- | --- | --- |
| `analytical_infrastructure/arrow.md` | Bounded deep dive: current IPC readers, projection/hydration and native admission; overview covers 60, consumed API is 59.3. | Compare projected analytical decoding with required scientific/key closure; separately assess Parquet selective decoding only for a named persisted analytical product. |
| `analytical_infrastructure/datafusion.md` | Bounded deep dive: actual snapshot/cohort providers and pinned 55.1 `TableProvider` contract after Context7; overview covers 55.2. | Share selection with physical decoding and evaluate typed ephemeral evidence joins/aggregates without moving numerical policy. |
| `analytical_infrastructure/object_store.md` | Bounded deep dive: pinned 0.13.2 `GetOptions`, range, stream and multipart source after Context7; overview covers 0.14.2. | Qualify backend-specific coherent historical ranges and uncertain multipart completion for a selected shared archive. |
| `analytical_infrastructure/petgraph_and_rustworkx_core.md` | Screen: representation, algorithm and integration families; actual petgraph pin checked. | Select occurrence-preserving immutable dependency/provenance analysis; qualify a particular rustworkx routine only when its output semantics are needed. |
| `analytical_infrastructure/schemars.md` | Screen: directional generation, transformations, identity and validation boundaries against the existing source-generation role. | Resolve output-only observations and schema identity through the same source declarations; exact transformation API comparison remains open. |
| `analytical_infrastructure/serde_serde-json.md` | Screen: typed versus dynamic decoding, raw/borrowed values and absent/null/duplicate distinctions against retained report and metadata boundaries. | Determine which remaining consumers need original-byte retention versus typed adoption before using `Value` or defaults that normalize distinctions. |
| `analytical_infrastructure/z3.md` | Screen: finite constraints, incrementality, explanations and native execution limitations; no solver adoption or API qualification. | Identify a specific policy/consistency problem and compare a source-derived checker with solver extraction, unknown outcomes and lifecycle burden. |
| `surrealDB/surrealdb.md` | Bounded deep dive: current guarded queries/capture/migration and native plan inspection; Context7 current examples separated from patched 3.3 behavior. | Qualify selected query/index and absent-member dependency behavior on the exact source/patch tuple before changing guards or projection placement. |
| `surrealDB/SurrealMX.md` | Bounded deep dive: 1.0 overview and maintained installed/tagged-source documentation, server selection and actual capture boundaries; no runtime probe. | Establish which native transaction/range dependencies the patched SurrealDB bridge actually carries; standalone engine capabilities do not settle that question. |
| `surrealDB/SurrealKV.md` | Screen: persistent/versioned storage and large-value placement, explicitly distinct from the selected server build. | Compare exact compatible backend integration, active readers, compaction/reclamation and crash recovery if storage placement is reopened. |
| `surrealDB/surrealism.md` | Screen: typed guest/database composition, host resources and authority; existing capture execution decision retained as historical evidence. | Reopen a guest placement only for a complete useful operation whose transaction, deadline, module-pool and rollback obligations improve on the authored query. |
| `surrealDB/surrealdb_tooling.md` | Screen: migration/schema tools, language intelligence, query diagnostics and data movement. | Compare a selected parser/schema-comparison or diagnostic integration with existing generation/receipts; no SurrealKit beta or unrestricted operations endpoint is qualified. |
| `surrealDB/surrealdb_tooling_idea.md` | Screen: agent/tooling composition ideas treated as proposals rather than repository policy or installed capability. | Find a concrete developer diagnostic missing from existing native query/evidence owners before adding another tool/control plane. |
| `surrealDB/SurrealDB_v4.md` | Excluded after version/capability-boundary screen: current engagement uses patched 3.3. | A separately authorized version comparison would need replacements for consumed patches, storage/protocol semantics and retained interpretation; no v4 mechanism enters these recommendations as current capability. |
