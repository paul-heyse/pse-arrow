---
title: "Plan66e: Protected data and result delivery"
status: in-progress
date: 2026-11-06
adrs: [ADR-0224, ADR-0230, ADR-0250]
review_sources: [docs/plan_improvement_reviews/plan64-65-integrated-target-and-library-leverage-2026-11-06.md, docs/plan_improvement_reviews/plan64-65-data-results-and-evidence-analysis-2026-11-06.md]
doc_role: plan
doc_owner: docs/plans/66-integrated-simulation-target.md
doc_topics: [design, storage, diagnostics, validation, libraries]
---

# Protected data and result delivery

## Target and remaining baseline

**Proposed:** deliver scientific results and retained evidence through their original native
custody, with exact selection, acquired interpretation, coherent mapping activation and
structured outcomes. A small answer should use the physical read appropriate to its assurance
contract. Historical inspection, local wire validity, original scientific qualification and
current permission remain separate judgments. Shared delivery and analytical projections reuse
original facts; they do not create another permission, persistence or verdict authority.

The [coordinator](66-integrated-simulation-target.md) owns the sole go-forward target. This
companion carries the remaining storage/result/diagnostic obligations represented by former
65c/C01–C07, 65f/F01–F06 and 64g/G08, together with selected amendments
[DRE01–DRE06](../plan_improvement_reviews/plan64-65-data-results-and-evidence-analysis-2026-11-06.md).
Those identities explain provenance; this document's E packages own execution. The baseline is
main `66341514d`. Core 3.4, efficiency heuristics, ProcessSimulator 1.5 and the selected
test-assessment companion inform this design. Blueprint §19.2, §20.5 and §23 retain their
architectural ownership. No product workload was run to author this plan.

Existing foundations are consumed, with their remaining assembled verification scheduled below:

- `pse-operations::canonical_codec` already preserves full-range unsigned values through native
  Decimal, IEEE bits, structured identities and relationship meaning; generated native codecs,
  borrowed wire admission and guarded effects already exist. Complete remaining declared
  producer/consumer coverage rather than replace these foundations with ProtoJSON.
- `canonical_results` already owns protected `ResultRead` acquisition, renewal and release;
  escaped readers and failed release retain obligations. `canonical_capture_results` already
  uses atomic capture, protected membership, an exact-request receipt and publication witness.
  G08's remaining migration/acceptance does not authorize rebuilding that protocol.
- `workflow/result_selection.rs` already declares passive `PackResultSelection`, exact predicate
  bits and original context. `pse-columnar::ipc_stream` already rejects duplicate schema and
  nested-field metadata before Arrow map conversion. The source context is implemented; the
  ordinary generated ADR-0250 export/read consumer and its dependency-complete generation remain
  unfinished. Frozen worktree candidates are proposals, not integrated baseline evidence.
- `workflow/result_snapshot.rs` and `cohort_provider.rs` already implement DataFusion providers,
  frozen selections and availability distinctions. Projection-aware inspection also exists.
  Complete their physical composition; a second provider framework adds no needed capability.
- Native diagnostic, scientific-permission/local-validity, keyed multi-cause outcome and developer
  evidence projections are partly delivered. Original directory staging, native commitments and
  strict member inspection exist. `OpenedFile::with_reader`/`copy_verified` already hashes the
  actual transfer. Existing directory export does not need a replacement checksum pipeline.

The implemented baseline closes no remaining population by itself. Required science families
enter through their Plan66 owners and the [66l population](66l-required-population-and-assembled-acceptance.md),
not an illustrative successful result.

## Responsibility and operation boundaries

[66b](66b-contracts-generation-and-interpretation.md) owns shared declarations, generated
structural codecs/descriptors, interpretation compatibility and directional adapter generation.
This companion owns their storage/result integration, protected read custody, family-specific
historical acquisition and mapping activation. Semantic admission and scientific-use decisions
remain with the native producers. Source owners declare meaning once; generation carries the
mechanical correspondence without importing database or Arrow execution into the pure contract.

[66c](66c-preparation-execution-and-service-lifecycles.md) owns execution, retirement and service
effects. E consumes original action/reap/settlement outcomes and returns independent read,
publication and release failures without flattening them. A released or decoded observation
cannot recreate a live operation handle.

[66f](66f-developer-operations-clients-and-evidence.md) owns developer invocation/report
acquisition, readiness, explicit preparation, client commands and exact rerun authority.
E05 specifies the common sequential-member delivery contract; 66f integrates it at the native
opened-file/report owner and migrates developer clients. E07 owns the ephemeral analytical
composition and shared source-fact projections; 66f supplies invocation facts and consumes
answers. These are one implementation per responsibility, not parallel evidence stores.
Scientific result acquisition and project/review-pack consumers remain here. The coordinator
settles shared editing; package dependencies express working capabilities rather than whole-plan
barriers.

A complete operation acquires original scope and dependencies, chooses the promised read
assurance, executes a bounded native/query/decoder operation, delivers facts and extent, and
settles buffers, publication and protection. Sparse coordinates, pending/captured-unavailable/
empty/included populations, terminal attempt, requested versus observed coverage and usability
are not interchangeable. Partial bytes or locally valid metadata cannot imply complete capture.

## Selected design and mechanisms

### Exact selected intent through real physical reads — DRE01

Complete the actual connected-result export → portable-reader migration under ADR-0250.
Use the source-owned generated `RecordedResultSelection` adapter from the existing engineering
descriptor world. The result-specific adapter must compile for ordinary default-feature export;
keep service-only adapters/services gated and avoid duplicate native conversion implementations.
New v1 observations require actual `Some(original_context)`; an absent context is different from
an empty field and remains available only to deliberately retained predecessor artifacts.

The versioned Arrow key is `pse.canonical.selection.proto.v1`. Maintained Buffa 0.9.2
[`Message::try_encode_bounded`](../../vendor/buffa/src/message.rs) checks size/cache and the
65,536-byte budget before writing. Base64 0.22.1
[`Engine` with `engine::general_purpose::STANDARD`](https://docs.rs/base64/0.22.1/base64/engine/trait.Engine.html) produces
the padded string, bounded to 87,384 bytes. Compare input text against bounded source-derived
expected encoding. This local v1 agreement deliberately rejects alternate encoding, unknown or
trailing bytes and altered predicate/context; it is not general Protobuf canonicalization,
authenticity or scientific admission. No untrusted decode-to-permission path is introduced.
Remove reserved obsolete lineage keys from new exports, retain unrelated metadata, refuse mixed
reserved interpretations, and never fall back when v1 is present-invalid. Only absent-v1
artifacts enter the explicitly promised historical reader. Preserve existing raw IPC duplicate
preflight for both file/stream schemas and nested fields.

Selection then reaches the physical reader under its actual assurance. DataFusion 55.1
[`TableProvider`](https://docs.rs/crate/datafusion-session/55.1.0/source/src/table.rs) receives
projection/filter/limit; filter dependencies may be absent from the final projection and
`Inexact` requires residual filtering and prevents unsafe limit pushdown. Arrow 59.3
[`FileDecoder::with_projection`](https://docs.rs/arrow-ipc/59.3.0/arrow_ipc/reader/struct.FileDecoder.html)
can select physical roots. Derive required roots from output, predicates, keys and native
permission inputs, then reuse the existing inspection/decoder path where its guarantees fit.
Keep full checked scans for full relation admission. A projected batch is a readable selected
result, not evidence that omitted roots passed full validation. Retain deployment/pool identity,
original row coordinates and continuation/extent meaning through `SnapshotProvider`, cohorts,
connected results, review packs and project bundles. Delete redundant full-detail hydration and
selection passes only after their actual consumers have equivalent guarantees.

### Original interpretation and coherent derived mappings — DRE03

Acquire the original family-specific descriptor world, interpretation, validation policy,
mapping revision and source dependency closure once under the existing `ResultRead`, artifact
or report acquisition. Retain only dependencies promised to that reader. Manifest `descriptors`
are result-set membership, not automatically a full Protobuf world. An interpretation string
does not supply a missing historical policy. Buffa 0.9.2
[`DescriptorPool`/`DynamicMessage`](https://github.com/anthropics/buffa/blob/v0.9.2/buffa-descriptor/src/reflect/dynamic.rs)
supports passive independent interpretation; 66b owns compatibility checks and native source
owners own any scientific re-admission. No live fixture, solver or worker is kept prepared for
historical reading. Missing, expired, malformed or foreign dependencies give explicit
incompatible/incomplete results rather than today's interpretation or an empty success.

For a selected changed native mapping, construct the replacement privately, validate membership
and original dependencies, and activate a coherent generation. Existing guarded direct queries
remain usable while a derived generation is incomplete/stale. Separate mapping-only rebuild
from physical datum, phase/provider or derivative changes that invalidate scientific products.
Interruption before activation leaves the previous generation; interruption or lost ACK after
activation resolves the original operation before retry. Restore neither old scientific
permission nor current permission from stored facts. Migrate all participating writers/readers
with the cutover and delete replaced writers/maps; retain only specifically promised historical
readers and the legacy migration/recovery they still require. Do not create a global descriptor
catalog or attach every dependency to every row.

The exact store is patched SurrealDB 3.3.4/SurrealMX 1.0, selected by
[`pse-host::server`](../../crates/pse-host/src/server.rs), with the
[maintained 3.3 guide](../libraries/proto/Protobuf_SurrealDB_v3.3.md).
Use generated `SurrealValue::{kind_of,into_value,from_value}` and native `Kind` where their exact
contracts fit. They do not implement permission predicates, membership transactions or retry
policy. Preserve explicit Decimal integrality/range and IEEE-bit policies, relationship
occurrence identity, borrowed cumulative admission, statement accounting and terminal metadata.
`surrealdb_types::{encode_proto,decode_proto}` is native Value interchange, not an arbitrary
domain-message codec or bounded admission substitute. SurrealDB v4 and standalone SurrealKV
capabilities establish no contract for this server.

### Diagnostic meaning and selected evidence delivery — DRE02/DRE05

Complete one source-owned observation population across `pse-diagnostics::observation`, runtime
diagnostic documents/source maps, scientific completion/product scope, canonical results and
developer action/inventory/fixture/assertion/capture/settlement. Use generated observations for
CLI/local-service/portable metadata; retain independent original action, reap, release, I/O join,
panic-payload destruction and durability uncertainty. Unknown, absent, malformed and oversized
details remain distinct. Preserve recursive public redaction and complete original evidence.
Delete presentation-string classification, repeated unit/enum conversion tables and flattened
error mirrors as each consumer migrates. External OTLP/Perfetto schemas retain their own wire
owners while consuming the same native facts, stable correlation identities, clock domains and
coverage. Disabled, stale, dropped and unavailable coverage are explicit; Debug output does not
silently become the explicit export policy.

Select sequential original-member delivery beside strict range inspection. Existing
[`OpenedFile`](../../crates/pse-host/src/test_workflow/execute/evidence/opened.rs) binds original
descriptor, identity and digest. Strict `read_range` intentionally verifies the whole promised
member on each call. Repeated tiny ranges must not be the bulk-transfer implementation.
Reuse `with_reader`/`copy_verified` to hash actual transferred chunks, check growth and before/
after identity, and issue whole-member completion only after EOF/full commitment agreement.
An emitted prefix is provisional. The native owner keeps acquisition/protection alive until
transfer/release settles; a sink publishes only after its selected complete/sync contract.
Cancellation, escaped/slow consumers and sink failure retain partial extent and independent
cleanup causes. Compact payload limits remain explicit; crossing the 64-MiB compact boundary
does not silently materialize the whole closure or fall back to a compact response. No chunk
service is required for the offline sink operation. An open FD/read-only mode/earlier hash does
not justify caching immutability; a sealed-copy profile requires a separate real random-reuse
need. Existing directory manifest-last, no-replace publication and original commitment checks
are reused, including collision and unknown-durability handling.

The integrated project counterpart is
[`ProjectBundleReader::copy_member_verified`](../../crates/pse-runtime/src/workflow/project_portability/member_copy.rs).
It reuses component-wise Rustix `openat`/`NOFOLLOW` containment and the host's read-only
`descriptor_identity` observation, without adopting developer-private UID/mode or SHA256 policy.
The project manifest retains its plain BLAKE3 `EncodingHasher` commitment and separate scientific/
framed-artifact interpretation. The existing bounded transfer loop supplies 64-KiB scratch and
checkpoints before reads and after the last write. Partial writes update the exact accepted count;
EOF, growth, before/after identity and actual transferred digest are checked before a receipt.
The transfer owns no sink flush/sync/publication. `member_bytes` and the real import archive now
consume this owner; their replaced prehash/raw-file API and duplicate archive hash loop are removed.
Full bundle admission remains its existing distinct assurance contract; this is not E03's selected-
hydration completion or a claim of filesystem snapshot isolation.

Select an ephemeral Arrow/DataFusion evidence view over already acquired `FrozenEvidenceSet`
facts. `tooling/review_analysis.rs` remains owner of its ten domain questions, numerical-history
policy, comparability and exact evidence references. Add useful relational joins/group/order for
cross-invocation preparation refusals, failure/capture/settlement patterns and original scientific
run/attempt association when present. Source-owner projections feed both these relations and
delivery/telemetry sinks. Keep small semantic folds native; do not rewrite every question in SQL.
DataFusion's existing session/provider mechanisms supply query execution; native owners supply
meaning, exact `u64`, clocks, producer generations, omission/coverage and original windows.
Release views with acquired evidence; add no persistent analytical/verdict database. An expected
refusal may be a passing test; recorded cleanup is not current control, and no query grants
preparation, rerun or scientific permission. 66f remains the sole owner of those operations.

### Capture and optional expansion decisions — DRE04/DRE06

Accept the implemented atomic capture/witness/protection/receipt population and finish its actual
legacy migration/retirement evidence. Preserve future matching insert/selector-metadata coverage,
same-request lost-ACK replay, stale-owner refusal, cancellation and unrelated lease liveness.
`canonical_capture_legacy` is required bounded predecessor cleanup/recovery while promised data
still needs it; it is not an ordinary production capture path to reproduce. Delete remaining
ordinary legacy callers only when replaced; do not delete the historical recovery promise as a
cleanup shortcut. Use current `canonical_explanations` bound plan inspection and native
`EXPLAIN FORMAT JSON`/optional `EXPLAIN ANALYZE FORMAT JSON`, with raw output and scope limits.
Absent counters are not inferred cache misses or inner-function costs.

Keep the conservative witness. A narrower conflict domain is a bounded decision only: name a
whole-operation contention consumer, enumerate every writer and future/absent membership
dependency, and compare equivalence before selection. Existing-row `FOR UPDATE` alone does not
prove phantom coverage. [SurrealMX 1.0](https://github.com/surrealdb/surrealmx/tree/v1.0.0)
snapshot/range capabilities do not establish that patched SurrealQL exposes the required tracking.
If equivalence or useful benefit is unsupported, retain the current witness with an observable
revisit trigger; no throughput improvement is claimed.

Likewise, shared remote archive and richer graph analysis receive bounded dispositions, not
automatic implementation. For a named shared-reader/archive requirement compare local native
descriptors against object_store 0.13.2
[`get_opts`](https://docs.rs/object_store/0.13.2/object_store/struct.GetOptions.html)
(`range`, `if_match`, `version`), `GetResult::into_stream`, conditional `put_opts` and explicit
[`MultipartUpload::complete/abort`](https://docs.rs/crate/object_store/0.13.2/source/src/upload.rs).
Default `get_ranges` coalesces but has no per-call version argument; it alone does not provide a
historical snapshot. ETag is not native digest/read permission. Upload completion, catalog
publication, ambiguous acknowledgement and orphan cleanup are distinct obligations. Select a
complete adapter under the archive responsibility only when credentials/backend qualification,
retention, active readers and recovery are included; otherwise record non-selection and reopen
for an actual shared/versioned/large-result consumer. No filesystem emulation or second canonical
store is selected.

For a named question beyond existing immediate-neighborhood queries, compare selected immutable
graph extraction with [petgraph 0.8.3](https://docs.rs/petgraph/0.8.3/petgraph/) SCC/reachability/
impact algorithms, preserving direction, edge kind and parallel occurrence identity. Distinguish
scientific topology from persisted explanation edges. No all-path export, adjacency cache,
maintained view, broker/changefeed/outbox or durable cursor/ack protocol is selected. Reopen
those routes only for a specified output or continuity promise and its full freshness/recovery
cost. A favorable library overview alone does not select either extension.

## Packages

An implemented prerequisite is not a request to rebuild it.
Packages can deliver working vertical slices once their actual prerequisites are available.

### E09 bounded decisions — November6

**Interface-checked:** retain the current publication witness for the selected study capture,
project export and result retirement consumers. The authoritative writer dependency declaration is
[`surreal_study_capture_witness::append`](../../crates/pse-codegen/src/codegen/surreal_study_capture_witness.rs):
study/point identity and membership, run selection and terminal attempt, attempt/manifest generation,
result/history retirement, project-branch selectors and research references/publications all affect
capture. Lease/heartbeat fields deliberately do not. Narrowing only a selected study's existing rows
would miss future matches through those other selectors. No observed contention workload or exact
patched-engine predicate-tracking contract presently supports replacing that complete domain.
Reopen for a measured complete capture workload dominated by independent-study witness contention,
with every writer and absent/future match covered before selection. E08 retains its actual
receiving/retirement controls; this decision supplies no new transaction qualification or speed claim.

**Interface-checked:** retain local immutable contained files for the current independent project
bundle and review-pack readers. Their selected output is a self-contained offline directory with
explicit original membership; neither requires shared remote continuity. The existing
[`ProjectBundleReader`](../../crates/pse-runtime/src/workflow/project_portability.rs) and
[`PackReader`](../../crates/pse-runtime/src/tooling/review_pack.rs) retain those responsibilities.
Object-store versioned conditional reads/streaming and explicit multipart completion/abort, detailed
above, remain a credible alternative for a named shared or independently hosted large-result
consumer. Reopen with that consumer's credentials/backend, exact version/read protection, publication,
ambiguous acknowledgement, orphan recovery and retention contract. This is non-selection for these
consumers, not a claim that remote storage cannot help the broader system.

**Interface-checked:** retain native bounded immediate-neighborhood queries for current blocker,
recorded-source and retained-consumer explanations. Their actual scope and revision-bound
continuations are declared by
[`CanonicalGraphRequest`](../../crates/pse-operations/src/canonical_explanations.rs); they do not
claim transitive impact or SCC completeness. No remaining selected explanation requires those
outputs. Reopen petgraph's immutable occurrence-preserving SCC/reachability route for a concrete
transitive question, with edge-kind/direction and output-completeness contracts. Physical process
topology remains a separate scientific owner. No graph cache or new durable graph authority is
selected. These three decisions discharge E09's bounded choice; E01–E08/E10 and L acceptance remain.

| ID | Status | Prerequisites | Completion |
| --- | --- | --- | --- |
| E01 | planned | Baseline native codecs/admission; 66b source declarations and generated structural products | Reconcile the remaining scientific/runtime/canonical request/result producer-reader population. Complete only missing generated native mappings; preserve exact numeric/identity/relationship policy and owned native predicates. Every selected converter has independent malformed/duplicate/extreme-value controls; remove its replaced field map and callers. |
| E02 | in-progress | Existing selection/context and IPC duplicate preflight; 66b ordinary root-scoped adapter generation | Migrate connected Arrow export and portable read together under ADR-0250, including default-feature availability and service feature composition. Require original context for v1, remove reserved predecessor new-write keys, reject mixed/present-invalid metadata, retain deliberate absent-v1 reading. Independent raw IPC/wire tests and real protected export/read prove exact bits/context without permission reconstruction. |
| E03 | planned | Existing protected snapshot/cohort/inspection readers; E01 needed mappings; E02 metadata consumer | Carry output/range/predicate/key/permission root closure into appropriate physical reads for full/selected/empty cohorts, connected results, review packs and project bundles. Retain residual-filter/limit order, coordinates, pool identity and original custody. Remove redundant hydration/selection passes with all callers; full admission remains available where required. |
| E04 | planned | 66b acquired interpretation/compatibility contract; actual family source references from science owners; E01 | Acquire promised historical dependencies once; implement selected mapping reconstruction and coherent activation with original-operation recovery. Migrate corresponding writers/readers and retire replaced production maps. Missing/corrupt/expired/foreign context and unavailable original policy refuse explicitly; mapping-only change and scientific invalidation remain distinct. |
| E05 | in-progress | Existing original opened-file/directory owners; shared contract agreed with 66f; 66c settlement outcomes where consumed | Developer sequential delivery/offline staged publication and project actual-transfer/archive/materialized consumers are integrated. Main's four project and thirteen selected semantic/developer controls pass with complete capture/settlement and zero failures/pending/build warnings. Remaining scientific/review-pack delivery population stays open; original commitments, provisional prefixes and independent publication/release outcomes remain distinct. |
| E06 | planned | Source diagnostic/outcome owners; 66b directional adapters; working 66c/66f producer outcomes | Complete native diagnostic/result-description and CLI/local-service/sink population. Preserve coverage, qualifiers, original attempt, typed multi-cause and detail dispositions/redaction. Migrate OTLP/Perfetto correlations from common facts, removing string classification and duplicate semantic maps without replacing external protocols. |
| E07 | planned | Acquired 66f original evidence and shared source-fact projections; E06 needed facts | Implement ephemeral typed Arrow/DataFusion analytical relations and selected cross-invocation queries. Retain native review-analysis policy and literal evidence attribution; migrate named analyst/client consumers with 66f. Demonstrate useful joins/grouping plus unknown/partial clock/coverage semantics, with no durable verdict store or action grant. |
| E08 | planned | Implemented atomic capture/witness/receipt and original legacy recovery; actual study consumers from science owners | Complete remaining capture caller/migration/deletion population and required acceptance of existing protocol. Cover future membership/selector updates, lost ACK, stale owner, paged abandoned cleanup, both retirement orderings and unrelated lease/cancel liveness. Required historical cleanup remains. This row is verification/migration of baseline, not a second atomic capture implementation. |
| E09 | complete | Concrete consumer/workload inventory; current plan inspection; DRE04/DRE06 alternatives | Retain current witness, local immutable archive and native immediate-neighborhood queries for the identified current consumers, with source-backed dependency/scope limits and observable reopening triggers above. No narrower-guard equivalence, remote backend or transitive-graph qualification is inferred. |
| E10 | planned | E01–E08 selected migrations; E09 dispositions; relevant 66b/66c/66f/science consumers | Reconcile exact producer→publication→query→protected read→original interpretation→delivery→release population, deletions and enduring recovery guidance. Supply required evidence to 66l, including late failures and assembled cross-domain journeys below. Targeted success closes only its named population; scope-end qualification and claimed-benefit measurements remain separately identified. |

## Migration, recovery and acceptance

Track selected populations at their native owners with 66l's shared denominator. Inventory
includes canonical records and request/result codecs; full/selected snapshot and cohort readers;
connected Arrow export and portable payload validation; scientific result metadata and progress;
review-pack/project-closure publication; CLI/local-service errors; OTLP/Perfetto; compact/directory
developer readback and sequential delivery; retained interpretations/mappings and atomic study
capture/legacy recovery. Additional science families extend the same contracts and acceptance
population rather than invent parallel export/admission paths.

Activation is per coherent declared mapping/consumer slice. Do not require whole-series completion
to use a finished reader, and do not activate a new writer before its actual readers and recovery
policy exist. New writes use only the chosen representation. Historical readers survive only
their named artifact/data promise; remove them when that promise and dependent data migration
are retired. Delete old producers, handwritten correspondence, redundant hydration, tests and
fixtures with proven replacement. Preserve independent counterexamples and required predecessor
recovery. No compatibility shim becomes a second production path.

Targeted correctness checks accompany each implemented slice, with explicit Arrow
`force_validate`, required fixtures, exact features/native profiles and original invocation
ownership. Use `scripts/pse check <pkg> --all-targets` and selected `scripts/pse test` routes;
generator changes include `just codegen`. E02 checks both ordinary default-feature and
`engineering-service` compositions. Expected values must include independent literal malformed
bytes/foreign identities, raw FlatBuffer/Proto controls and original native outcomes; round trips
through the same mapper alone are insufficient. No product checks are prerequisites to authoring
this document.

Required assembled journeys, coordinated with 66l:

1. Exact scientific run/terminal attempt → full/selected/empty/sparse read → connected export →
   independent portable interpretation. Include predicate-only fields, composite roots,
   inexact-filter-plus-limit, corrupt omitted roots, withheld covariance/dual use and late terminal
   failure. Selected readability cannot upgrade whole-product scientific permission.
2. Multi-experiment capture concurrent with future matching insert/selector change → cohort
   query → unavailable-member classification → original analysis → retirement. Exercise lost
   ACK, stale owner, both retirement commit orderings, legacy version migration and paged cleanup.
3. Mapping-only revision versus physical/provider/derivative change → private reconstruction →
   interrupted/uncertain activation → old promised read → explicit current re-admission. Include
   absent/expired/corrupt original descriptor/policy and foreign run/attempt.
4. Conditional/exact developer readiness → explicit preparation → native command/test → original
   compact or member above compact limits → sequential independent read → 66f exact rerun after
   changed source/features. Inspect/query never prepares; analytical answers never grant rerun.
5. Slow/escaped reader during cancellation or publication failure. Preserve original action,
   reap, I/O join, capture, release and durability uncertainty independently. Challenge same-inode
   mutation, prefix interruption, late corruption, missing later member, destination collision,
   final sync failure and retry against the original operation.

**Proposed verification:** zero failures/warnings is the baseline for checks actually selected;
report commands, mode, feature/fixture scope and exclusions. Component controls and selected
capture tests do not establish whole G08/result/developer population qualification. 66l owns
assembled integration, relevant solver/reference journeys and scope-end static/manual checks.
Compare complete acquire/query/transfer/render/release work before claiming lower latency,
memory, I/O or contention. Measurements are required for quantitative benefits, not for deleting
structurally repeated work while preserving guarantees. No measured benefit is asserted here.

## Decisions and handoff

The authoring authorization selects ephemeral analytical evidence and sequential member delivery;
their implementation contracts and ownership are scheduled above. It does not select remote
archive, richer graph, scoped witness or durable external continuity. No confirmed design-rule
change is introduced by this companion. Existing ADR-0250 is the metadata cutover route; new
metadata/backend/relation/kernel or authority changes follow the repository's decision table
before affected implementation. A bounded non-selection records its actual consumer boundary
and observable revisit trigger rather than pretending the library lacks the capability.

At completion, E10 hands 66l the exact implemented population, deletions, historical promises,
activation/recovery behavior and remaining evidence limits. Enduring storage, diagnostic and
delivery contracts move to their architectural owners through the established decision route;
the coordinator owns final status and retirement. Outcome evidence will distinguish Proposed,
Implemented, Tested and Measured claims and record corrections and deliberate deviations.
