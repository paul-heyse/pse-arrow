---
title: "SurrealKV and SurrealMX functional extensions"
date: 2026-10-22
doc_role: review
doc_owner: docs/functional_reviews/surrealkv-surrealmx-functional-extensions-2026-10-22.md
doc_retention: while-dependent
doc_topics: [design, libraries, storage, diagnostics, preparation, performance, validation]
---

# SurrealKV and SurrealMX functional extensions

## Judgment and exploration scope

**Proposed:** develop two related capabilities: an engineering project workbench over exact
model, data and study revisions, and a queryable evidence workbench over test, diagnostic,
research and measurement records. They have useful CLI/Python consumers before a graphical
frontend. Their strongest shared foundation is explicit identity, coherent selected views,
typed queryable metadata and selective access to original evidence.

The libraries also warrant a substantive storage-design investigation. Compare the current
SurrealDB/MX service and sealed files with embedded SurrealDB and direct native KV/MX designs.
In particular, native persistent KV could place typed metadata and bounded payload objects
under one storage publication/backup root. That can remove some cross-store coordination,
not merely accelerate existing lookups. It also assumes responsibilities currently supplied
by SurrealDB and the file tier. No topology is selected solely from engine benchmarks.

This is the authorized **library-targeted functional-extension review**, covering simulator
product, developer, operator and research activities, including credible future desktop,
browser and collaborative workflows. The maintainer explicitly allowed all realizations,
including replacing canonical topology. Structured test-result analysis was an example, not
the inquiry's boundary. Opportunities need neither a diagnosed defect nor an existing caller.

Recommendations remain **Proposed**. This review does not extend an accepted plan, select a
new production engine, authorize a migration or change scientific authority. Current plans
are context and potential delivery owners, not the ceiling of the inquiry.

### Baseline, method and evidence limits

**Interface-checked:** source `d7cbf2667bb5dc8a3c083da395c93d036c1b10e1`, with the supplied
untracked `docs/libraries/SurrealMX.md` as an additional input. Both supplied documents,
[SurrealKV](../libraries/SurrealKV.md) and that local SurrealMX source, were read completely.
Their mechanisms and combinations informed the full inquiry, including features
with no immediate adoption recommendation. Exact upstream source resolves consequential
questions; an overview describes possibilities rather than the installed contract.

Version anchors inspected are SurrealKV 1.0.0 commit
`4338a790c8edc56ef5ea1e46157427af4815be3c` and SurrealMX 1.0.0 commit
`f67f00cb94f4fb4582090cd86b3b6993f8e25300`. Feature followups used the supplied primary
links and exact sources, following the maintainer's instruction. Initial documentation-index
discovery returned mixed-version KV material and no valid MX match; it supplies no decisive
feature evidence here. Missing indexed documentation is not capability absence.

The assessment applies Core 3.4, Efficient Architecture Heuristics 1.0, Process Simulator 1.5
and the pse-arrow binding. Bounded library research, current-consumer mapping and independent
design advice informed the principal review. This is functional exploration, with no broad
diagnostic-audit prerequisite or automatic recommendation-audit stage.

Current implementation descriptions are **Implemented/source-inspected**, not new Tested
claims. No product tests, engine probes, benchmarks, installations, dependency changes, store
operations or service lifecycle changes were performed. Document publication checks establish
only document consistency. There is no measured speed, capacity or developer-effort claim.

## Current strengths and genuine extension space

The architecture already has a database. The question is which additional capabilities should
use it, where native engines offer a better realization, and which responsibilities can disappear.

| Existing foundation | Source-inspected owner / contract | Consequence for this review |
| --- | --- | --- |
| Custom SurrealDB 3.3.4 with persistent MX 1.0, explicit SSI, synchronous AOL/EveryAppend, gRPC and Surrealism | `scripts/surreal_source.py`, `scripts/surrealdb-source/mx-1.0-controls.patch`, blueprint §20.6; proposed ADR-0214 | MX upgrade, SSI, existing controls and metrics are delivered foundations, not new opportunities. |
| Exact immutable model revisions, guarded operations and selected result manifests | `pse-operations::{canonical_metadata,canonical_execution,canonical_studies,canonical_results}`; blueprint §5/§20 | Revision history, durable study execution and consistent result selection already exist. Product-level navigation and cross-run questions can compose them. |
| Bulk scientific data separated from database metadata | `pse-runtime::workflow::{result_files,result_blocks,result_export}`; blueprint §20.1–§20.4 | Sealed IPC/seed objects, exact descriptors, protection and physical custody are existing strengths. Replacing this split is a deliberate design alternative. |
| Local studies, explicit recording and shared occurrence projections | `pse-runtime::workflow::{study,study_execution,study_demand,study_tables,study_operations}` | Effect-free admission and local execution already support experimentation. A workspace should organize these operations rather than reimplement their executors. |
| Structured test and assessment facts | `scripts/{test_run,validation,validation_receipts,validation_scope,validation_result}.py` | Stable selected identities, outcomes, source/configuration evidence and report references already exist. The useful addition is compositional analysis over retained facts. |
| Resource and invocation authorities | `scripts/{resource_store,test_resources,run_inspection,invocation_disposition}.py` | Frozen terminals, membership fences, protected reports and unknown-effect handling cannot be replaced by a search index or successful import. |
| Active diagnostics and offline scientific review | `scripts/active_diagnostics.py`, runtime `tooling/{review_analysis,frozen_inspection}`, `workflow/reproducer.rs` | Standalone diagnostic identities, structured analysis and executable reproduction already exist. Cross-record discovery is new composition, not a new universal debugger. |
| Prepared artifacts and pure incremental preparation | `pse-compiler::workspace`, `pse-math`, runtime modeling preparation | Some immutable artifacts already persist. A database does not make evaluator/provider/native state portable or make all caches safe to share. |
| Canonical query/graph inspection, MX metrics and disposable schema rehearsal | [substrate guide](../dev/surreal-substrate.md), Plan50 Outcome | Extend useful workflows instead of adding duplicate telemetry, another query catalogue or another engine-upgrade package. |

The inspected tooling surfaces have multiple structured histories but no demonstrated single
consumer composing all their questions. That is a scoped observation, not proof that the
repository contains no related capability. Scientific IDs, invocation IDs, test identities,
diagnostic action IDs and benchmark samples are distinct domains; associations require actual
recorded premises.

## Feature understanding and corrected premises

Neither library is a SQL, document, graph or analytical query engine. Native embedding supplies
transactional ordered bytes; SurrealDB adds its record/query/index/permission machinery, while
Arrow/DataFusion serves columnar processing. Direct embedding has to purchase the higher-level
behavior its selected consumers actually need.

| Capability combination considered | Functional value and material boundary | Opportunity routes |
| --- | --- | --- |
| KV MVCC, timestamped versions, `get_at`, history and retention | Historical parameter/context views; later backdated commits can change retrospective answers. Immutable run receipts retain separate meaning. | O01–O03, O05 |
| KV transactions, locked reads, savepoints, range tombstones and group commit | Atomic metadata/index changes, staged editing and bulk logical disposal. SI does not supply general phantom/write-skew protection; range deletion is not immediate physical reclamation. | O03, O13, O16 |
| KV WAL policies, recovery reports, checkpoints and integrity tools | Recoverable local projects and useful recovery inspection. Restore is non-atomic; a reported sync failure does not universally prove no commit. | O09, O14–O16 |
| KV LSM/radix memtable, cache, compression, filters, comparators and value separation | Storage/workload fit and ordered indexes. Comparator/encoding compatibility matters; prefix-only value-log reclamation can retain obsolete space. | O10, O12, O15–O17 |
| KV encryption/key management and inspection | Confidential native projects and archives with explicit key retention. Hidden API/format maturity and coverage of external payloads matter. | O14, O19 |
| KV OPFS primitives, browser execution and migration companions | Browser/offline integration research. The supplied ready-Tree premise is corrected below; compatibility tools are not a replication protocol. | O18, O20 |
| MX SSI, conditional operations, read-for-update and savepoints | Coherent interactive working collections and invariant-preserving command composition. Conflict retry, rollback and external effect settlement remain different operations. | O03, O10, O13 |
| MX ART, borrowed inspection, buffered scans and bidirectional cursors | Low-allocation ordered navigation and immediate reductions. Key encoding remains owned; skipped offsets still cost traversal; native bytes do not imply zero-copy Arrow. | O04, O10, O12 |
| MX framed AOL, group commit, consistent-cut snapshots and compression | Recoverable memory-first working state with explicit durability modes. Snapshots are current-state cuts, not historical archives or solver checkpoints. | O09, O14, O17 |
| MX inline/tracked GC, reader pins, queue controls, age limits and statistics | Diagnose retention, conflict and persistence pressure; choose lifecycle behavior. Approximate accounting is not RSS and age checks are not continuous cancellation. | O07, O13, O15 |
| Engine background work, errors, deterministic/reference-model tests and platform support | Better failure-oriented validation and deployment choices, rather than bespoke WAL/MVCC/concurrency machinery. Upstream testing is not assembled application qualification. | O09, O14–O20 |

### Corrections that change feasibility

**Browser persistence.** SurrealKV overview §9 describes OPFS-backed `Tree` initialization and
operation. The exact 1.0 README instead says the filesystem primitives are not yet connected
to `Tree`; its storage module is doc-hidden and outside the stable API. Browser filesystem
tests do not establish the complete database lifecycle. O18 is therefore a genuine integration
investigation, not a ready deployment recipe. [KV release README][kv-readme], [KV exports][kv-lib].

**SurrealDB/KV integration.** The current custom server selects `storage-mem`. Stock 3.3.4's
KV adapter calls `with_versioned_index`, absent from the inspected 1.0 builder, and layers its
own durability coordination. Native KV 1.0 through this server requires an adapter port and
semantic verification, not merely a feature flag or dependency bump. Direct KV and embedded
SurrealDB remain separate alternatives. [3.3.4 KV adapter][db-kv], [KV builder][kv-tree].

**Encryption and restore.** Native KV has encryption and checkpoint mechanisms, but encryption
types are doc-hidden and the format is not frozen. An encrypted checkpoint cannot be restored
into an independently created encrypted database with a different encryption identity. Copying
a checkpoint and opening it with required keys is a distinct route. Encryption identity,
application authority and scientific content identity must not be conflated. [KV key contract][kv-keys],
[KV checkpoint implementation][kv-checkpoint].

The supplied documents remain unedited operator inputs. These dispositions correct this review's
premises without silently changing the reference documents or claiming broader runtime validation.

## Proposed capabilities

### O01 — An engineering project and revision workbench

<a id="O01"></a>

An engineer should be able to open a project, find its packages and physical inputs, compare two
revisions, create a what-if variant, run selected cases and see results against the exact variant.
The same capability is useful from Python and CLI before a flowsheet editor exists.

Compose existing canonical source versions, local package construction, `LocalStudy`, explicit
recording and selected results into project-level commands. Project labels, branches/bookmarks
and experiment intent add useful organization; they must point to exact immutable closures.
SurrealDB records/indexes/relations fit cross-project discovery. Direct MX working state or KV
project files are credible alternatives for a self-contained deployment (O17).

**Recommend early.** The new capability is the coherent project interaction, not a second model
or executor. Local sibling edits already branch in `modeling/canonical.rs::admitted`; the
extension is persistent user-visible branch/compare/merge policy and recoverable project state.
Presentation annotations and view layout can belong to that project without changing physical
meaning or entering scientific identity merely because they are stored beside the model.
Demonstrate a source edit, semantic comparison, fork and recorded/local study whose
outputs still resolve the exact source, physical inputs and numerical policy. A graph of model
relationships must not be mistaken for equation dependency or solve order (PS-05).

### O02 — Historical engineering context with explicit time meanings

<a id="O02"></a>

Support questions such as “which equipment assumption was valid at the experiment date?” and
“which source version did the accepted run actually consume?” They are different questions.
Versioned KV can support the first with `set_at`, `get_at`, historical iteration and soft deletion;
immutable revisions and recorded input receipts support the second. A later correction can change
an effective-time view without rewriting a past run's recorded evidence.

**Recommend with O01/O05.** Keep effective time, recorded revision and run consumption explicit.
Do not build bitemporal machinery for every record by default: append-only correction records
plus indexes may suffice. Long-lived transaction snapshots are not an archive. Cross-key coherent
historical state needs an explicit application revision/cut, not independent latest `get_at`
calls. Demonstrate a backdated correction and confirm that original run reconstruction is unchanged.
Mechanisms: KV overview §§4.1–4.4 and [transaction source][kv-tx].

### O03 — Transactional drafts, commands and recoverable experimentation

<a id="O03"></a>

Permit a user to stage a coherent parameter/model edit, try a grouped transformation, discard it
or publish it as a new revision. MX SSI plus conditional updates and savepoints can replace
bespoke write-set/rollback logic in an embedded workspace; KV can make native drafts durable.
Through SurrealDB, use its owned transactions rather than opening underlying engine files.

**Recommend the user capability; select placement with O17.** A draft may be an immutable source
overlay, not a long-open database transaction. Savepoints undo pending database writes; they do
not rewind a committed revision, native solver, file write or external provider. Keep editing
transactions short and launch scientific work from an immutable admitted draft. An undo command
can publish a compensating revision. Acceptance should combine partial draft rollback, conflicting
edits, discard and cancellation of a solve without corrupting the draft. [MX transactions][mx-tx].

### O04 — A searchable study and scientific-result workbench

<a id="O04"></a>

Let an engineer ask for studies by model/physical revision, parameter cohort, outcome class,
requested KPI and recorded qualification. Select interesting points, compare candidates and
open their retained diagnostics without loading every trajectory. Research notebooks and a
future frontend can share the selection semantics.

Build on existing `StudyReport::table`, compact/full output demand, outcome/attempt projections,
result manifests and protected selectors. SurrealDB indexes select metadata; Arrow/DataFusion
performs consumed numerical aggregation over selected original batches. Ordered native cursors
fit neighboring-case navigation where predicates truly match key order.

**Recommend early.** Retain unattempted, cancelled, failed, candidate and usable distinctions;
missing detail is not a zero-valued result. Numerical ranking must declare units, eligibility
and completeness. A bounded exploratory sample cannot establish a global optimum. Demonstrate
the same selection in Python/CLI and a lazy original-trajectory read, including failed points.

### O05 — A scientific data and experiment catalogue

<a id="O05"></a>

Organize observations, calibration datasets, physical-property sources, reference cases and
generated research datasets by their declared meaning. A researcher could select observations
for one material system/envelope, create a fitting study and trace fitted parameters and residuals
back to the exact original measurements.

Use typed dataset/experiment metadata, source content identities, provenance edges and explicit
quantity conventions. Existing package data documents, fitting experiment inputs and reference
campaign Parquet data are integration seams, not evidence that a general catalogue is complete.
Keep large data in the present columnar tier initially; compare O16 where one project file would
be materially simpler. KV temporal versions can organize corrections without mutating original
records; SurrealDB supplies discovery/query semantics.

**Recommend early with O04.** Include provenance and uncertainty/weight conventions, raw versus
curated/derived roles and explicit inclusion/exclusion. Database presence grants no physical or
statistical validity. Make the complete raw measurement → calibration revision → admitted fit
observations → fitted parameter revision → downstream design-study interaction navigable.
Acquisition time, calibration validity, model/integration time and commit ordering need their
actual meanings; a single storage timestamp cannot represent them all.
Reusable surrogate-training or validation datasets are a valuable composed
research outcome; no new ML framework, active-learning loop or automatic experiment scheduler
is justified by storage alone.

### O06 — Queryable test and qualification evidence

<a id="O06"></a>

Codex and the operator should be able to ask “which runtime tests failed in comparable native
runs?”, “which crates contain the largest selected failure population?” and “which selected tests
remain unreported?” and retrieve the original protected artifacts. This is useful immediately.

Compose the existing invocation selection, immutable terminal, test identities, assessment scope,
source/build/configuration facts and report references into a rebuildable evidence view. Prefer
an explicit import/query route over inserting a database dependency into every test assertion or
requiring the canonical service for emergency inspection. SurrealDB is a strong persistent query
realization; a one-shot Arrow/DataFusion view over current records is the simpler first consumer.

**Recommend early.** A persistent projection must name source identity/version, import coverage,
staleness and deletion availability. Repeated ingestion must be idempotent. Use Rust binary/test
IDs and Python nodeids rather than classifying log prose. Associate crate/module/source ownership
only from inventories or declared mappings. Selected, executed, failed, skipped, reused,
interrupted and unreported populations remain separate, with explicit denominators and filters.
Failures of a command/report are not automatically failed product assertions. Flakiness needs
comparable repeated execution, not merely a cross-run mixture of passes and failures.

An illustrative interaction is a structured failure query returning invocation/test IDs,
configuration differences and original evidence references, followed by the existing protected
reader. Its exact command grammar is future planning work. The projection cannot finalize a
terminal, acknowledge an effect, grant artifact retention or overwrite the original outcome.

### O07 — Cross-record diagnostic investigation

<a id="O07"></a>

Offer one investigation that can move from a failed test/run to related model diagnostics,
preparation comparison, native attempt, profile and storage-health observation. Also support
standalone actions that have no test invocation. The benefit is following authentic relationships,
not collecting every raw log into another database.

Compose Plan50's action documents, runtime review evidence and invocation references with O06's
discovery view. Store typed descriptors and selected indexed fields, retaining original files
and their digest/owner references. Open raw frames, IPC, cores and traces through their owning
readers only when requested. MX statistics can contextualize a database observation; shared
counters and sampled stacks do not establish operation-private causes.

**Recommend early.** A lost/truncated capture must remain incomplete; a missing association must
remain absent. Keep tool failure, observer interruption, target completion, scientific outcome and
drain separate. Demonstrate both an authentically linked diagnostic and a standalone action, plus
an unavailable/deleted payload. Avoid invented nearest-timestamp causality or automatic source
lookup that changes an offline result.

### O08 — Comparable performance and experiment history

<a id="O08"></a>

Make it possible to compare preparation/solve/storage observations across exact workloads,
features, source revisions, machines and capture modes. A researcher could filter a cohort before
examining duration distributions, allocation evidence or suspected regressions.

Use existing `case_measure.py` manifests, benchmark records, input/environment snapshots and
recorded sample membership. Store queryable descriptors and selected scalar observations;
consume larger tables through Arrow. The current code explicitly does not map every per-invocation
artifact to Criterion samples, so the workbench must not manufacture that join.

**Recommend after O06's identity/cohort seam.** Report incompatible or unknown conditions rather
than aggregate them silently. A failed/timed-out observation is censored or failed evidence,
not an exceptionally fast sample. Profiler-enabled execution remains distinguishable from
ordinary timing. Statistical conclusions require an actual suitable population; upstream KV/MX
benchmarks are not this simulator's performance baseline.

### O09 — Reproducible cases, disposable fixtures and portable evidence

<a id="O09"></a>

Allow the operator to retain or exchange the exact closure for a selected failure/experiment,
open it independently and explicitly rerun supported operations. Provide small disposable
project fixtures for query, storage and recovery behavior without restarting a production-scale
campaign.

Existing frozen review packs, authored/frozen reproducer codecs, recorded study descriptors and
closed backup/restore already cover parts of this workflow. Add closure discovery and a
user-facing package/selection operation rather than another replay engine. KV checkpoints or
MX snapshots can supply a storage cut, but providers, source interpretation, sealed payloads,
keys and deployment premises still need their actual owners. A checkpoint cannot serialize a
live native workspace or prove that an old scientific result reproduces.

**Recommend composition early; engine cloning is conditional.** Reproduction is explicit new
work with its own outcome. Changed builds, missing providers and unsupported schedules refuse
truthfully. Native checkpoints are not guaranteed cheap copy-on-write branches; external
independent backup and application-authority fencing remain separate questions.

### O10 — Native transactional working collections

<a id="O10"></a>

Direct MX can serve a process-local interactive collection of draft metadata, ordered selected
cases, inspection state or transient indexes. A coherent native transaction could replace a
custom mixture of maps, locks, snapshots and rollback journals where such a consumer exists.
Borrowed `with_value`/`scan_with`, reusable buffers and cursors can avoid intermediate metadata
objects and repeated sorting.

**Investigate for one concrete O01/O04 consumer.** Compare a plain immutable map, existing RPDS
source structures, database queries and direct MX. Do not duplicate canonical state merely to
use a fast engine. Independent direct calls do not provide a shared multi-key snapshot. Snapshot
pins must end with the actual consuming operation; a long-lived browser widget should not hold
a native transaction indefinitely. Verify any claimed copy/allocation reduction through the
whole caller path. This is not yet a zero-copy Arrow or cross-process shared-memory design.
[MX access paths][mx-db], [MX cursors][mx-cursor].

### O11 — Discoverable, dependency-correct immutable preparation reuse

<a id="O11"></a>

Expose which compatible immutable preparation artifacts already exist for a selected project,
what consumed premises permit reuse and why an artifact is unavailable after a change. This can
make cross-session work predictable and improve research/development introspection.

Existing persisted compiler/math artifacts, receiving validation and Salsa products are the
foundation. Query/index their actual semantic/build/profile/provider eligibility, rather than
persisting the whole Salsa database or treating an engine snapshot as a valid compiler image.
Direct KV could hold an independent artifact cache; the existing catalogue may already suffice.

**Recommend discovery with O01/O07; no blanket cache expansion.** New persistence needs a
complete consumed artifact and a real restart/reuse consumer. Prepared native state, provider
effects and factors retain their existing lifetimes. A value-only edit and a structural edit
should demonstrate different actual reuse behavior without inventing an invalidation cause.

### O12 — Query and index shapes matched to engineering questions

<a id="O12"></a>

Provide selected-source dependency exploration, nearby study navigation, provenance traversal and
filtered evidence retrieval through access paths suited to their real predicates. Composite
ordered keys and cursors can serve a native project deployment; SurrealDB secondary indexes and
graph relations can serve richer questions; DataFusion can aggregate selected columnar values.

**Recommend per consumer, not a generic new query framework.** Derived indexes must share a
transaction/publication boundary with their authoritative records or expose rebuild/staleness.
Use seek/continuation rather than growing offsets when ordering permits. Query execution plans
are observation, not scientific identity. No returned-row limit proves bounded physical scans;
Plan49g TOOL08's native edge-work obligation remains open. Full-text/vector search would be a
higher-layer optional capability for an actual document-discovery question, not a KV/MX feature
or prerequisite for structured engineering queries.

### O13 — Simpler coherent operational state transitions

<a id="O13"></a>

Evaluate whether one complete native/database operation can replace repeated client-side
read/decide/update coordination for an actual project, study or lifecycle transition. MX SSI,
conditional updates and savepoints are useful mechanisms; SurrealDB/Surrealism can place bounded
existing pure policy beside its data without moving native science into a transaction.

**Recommend targeted composition; do not replace existing authorities wholesale.** Current study
readiness, immutable operation settlement, cancellation and generation fencing already exist.
An embedded KV alternative must preserve cross-key invariants despite SI. Commit failure,
unacknowledged external effects and physical worker drain remain independent of transaction
rollback. Storage savepoints are not a substitute for idempotent operation IDs and settlement.

The file-backed host/test registry should remain usable when compiler or database startup is
broken. Replacing it with direct embedded storage also requires a real multi-process ownership
model; an embedded handle does not supply cross-process coordination. Prefer O06's derived query
view unless a complete authority replacement demonstrably removes machinery while preserving
these failure-path consumers.

### O14 — Recoverable project lifecycle and operator inspection

<a id="O14"></a>

Offer an operator workflow that explains what a selected project checkpoint contains, which
artifacts/keys it requires, whether it is independent of the live store and how to recover it
without confusing a new application authority with the original one.

KV `create_checkpoint`/`restore_from_checkpoint` and recovery reports, or MX consistent-cut
snapshots/AOL, can provide native mechanisms. Reuse the existing closed application backup and
maintenance owner where external files remain. Native checkpoint directory creation can share
immutable files through hard links; this is not an off-host independent backup. Detectable
interruption is not atomic restore. [KV checkpoints][kv-checkpoint], [MX persistence][mx-persist].

**Recommend useful lifecycle UI/CLI composition; select engine mechanism with topology.** A
representative recovery must account for the whole application closure, original uncertain
operations and changed incarnation. Do not make a new recurring backup daemon or broad disaster
campaign a prerequisite for every developer command.

### O15 — Storage-health explanations and lifecycle-aware retention

<a id="O15"></a>

Explain whether growth or slow work relates to pinned readers, queued persistence, compaction,
write stalls, obsolete versions, corruption or unresolved application retention. Show what is
observed, which owner can act and what remains unknown. This is valuable for both simulator
operation and diagnosing its development fixtures.

Compose already exposed MX statistics and conflict tracing with actual protection/operation
lifetimes. Native KV adds `recovery_report`, background/write-stall errors and `skv` inspection,
manifest/SST/scrub tools. These do not check scientific schema validity. Keep expensive scrub,
GC, snapshot and maintenance as explicit actions; observation alone must not reclaim data.

**Recommend after O07's association seam.** Reader age and engine bytes are diagnostic signals,
not proof of an owner-specific leak or process RSS. KV history retention is compaction-governed,
and MX snapshot pins influence memory reclamation. Existing workstation defaults do not need
a new global governor. Native knobs, profiles and autotuning are choices for a selected workload,
not universal recommendations or ways around single-flusher/compaction limits.

### O16 — A unified native project store for metadata and bounded payloads

<a id="O16"></a>

This is a high-value architectural candidate, not merely another cache. A direct persistent KV
project could store typed source/result descriptors, bounded IPC/seed objects and exact publication
membership in one native storage domain. For suitable objects, one transaction can make bytes
and references visible together; one native checkpoint root can capture that closure. Some
file-instance tickets, cross-store intents and separate payload reclamation could disappear.

**Investigate early before fixing the frontend persistence boundary.** Compare three complete
realizations: current SurrealDB catalogue plus sealed files; database metadata with selected
small values inline and large payloads external; native KV with chunked payload objects and
application-owned typed indexes/query integration. The native design has to replace the actual
query, schema, reference and scientific-selection behavior, not only the byte store.

Large multi-transaction products still need closed manifests and activation. Native KV SI does
not inherit current MX SSI guarantees. Owned value retrieval/decoding may lose file-backed Arrow
access and introduce copies or memory pressure. Long-lived seeds/retained runs can interact badly
with prefix-only value-log reclamation; inline storage has its own compaction costs. An uncertain
sync result still needs exact settlement. No recommendation to move all bulk data follows from
the existence of a value log. Source evidence and bounded probes may justify a target-design
selection. Before declaring the replacement delivered or qualified, demonstrate publication,
protected reading, failed-sync recovery, retirement and restore over a representative
mixed-lifetime project, including the original post-solve/domain/closure evidence.

### O17 — A deliberate deployment topology for local and growing projects

<a id="O17"></a>

Choose product deployment by complete capabilities: an authenticated shared local service,
embedded SurrealDB retaining its data model, direct MX for memory-first projects, or direct KV
for disk-oriented projects. Embedded SurrealDB is an important middle option between the current
remote SDK and reimplementing indexes/queries over bytes.

**Recommend an early topology comparison, not an immediate default switch.** Direct embedding
can remove an external server/transport lifecycle for a self-contained project. Embedded
SurrealDB still needs the selected query/schema behavior and installed Surrealism policy unless
a separately established in-process policy/transaction path replaces that invocation. Direct
KV/MX can remove more of that machinery while assuming its domain responsibilities.
One application process owns an embedded project; independent CLI/worker processes need
exclusive ownership or an application broker. Embedding does not automatically preserve the
current concurrent-client behavior. Persistent MX still needs its active dataset in memory.
Native KV offers disk-oriented growth, but the current server
needs a real 1.0 adapter port. A newer upstream integration is a separate pinned candidate,
not a reason to assume stock 3.3.4 is compatible. No paired MX/KV design has an implicit shared
transaction or automatic tiering/replication mechanism.

Keep initial design evidence workload-shaped: metadata-heavy edit/re-solve, large retained
trajectories, concurrent evidence ingestion and recoverable desktop startup. Compare normal,
failure and maintenance interactions. Do not migrate for a vendor throughput ranking; do not
retain a more complicated topology solely because it is already present.

### O18 — Browser and offline project capability

<a id="O18"></a>

A future user could inspect a portable project, edit a local draft and organize selected results
without an active server. Native MX's WASM in-memory core can support transient working state;
the KV OPFS primitives suggest a persistent browser project direction.

**Keep as explicit integration exploration.** Exact KV 1.0 does not yet expose the ready
OPFS-backed `Tree` described by the overview. Evaluate the actual worker/storage seam and a
complete close/reopen lifecycle before selecting it. A logical project export with another
browser persistence mechanism is a meaningful alternative. Browser quotas, eviction, mount
exclusion, blocking worker maintenance and missing encryption/checkpoint parity need platform
evidence; native guarantees cannot be copied into the browser contract.

Offline inspection/editing does not imply the Rust/native solver stack runs in a browser.
Synchronization, conflicts and permissions belong to O20, not engine snapshot files. No browser
implementation or frontend framework is selected by this review.

### O19 — Confidential projects and artifacts with owned keys

<a id="O19"></a>

Commercializable process models and experimental datasets can justify encrypted local project
storage and recoverable archives. Native KV's key-manager seam and encrypted file formats are a
credible enabling mechanism; MX does not itself provide encryption or authorization.

**Recommend as a product capability candidate, not a new current workstation requirement.**
Compare native KV encryption, platform/volume encryption and application-level packaging against
the actual threat and exchange workflow. KV encryption covers its files, not external IPC/core/
trace payloads, filenames or access patterns. Keys must survive every required historical artifact;
rotation does not instantly re-encrypt old files. The exported but doc-hidden API and unsettled
format need explicit compatibility ownership. Database auth, artifact confidentiality and
scientific identity are separate concerns. [KV encryption contract][kv-keys].

### O20 — Logical exchange, collaboration and reproducible research publication

<a id="O20"></a>

Support sharing a selected project or research result with its exact data/model closure,
opening it under a new owner's permissions and comparing or explicitly merging changes.
Even before multiple users, this serves notebooks, external review and moving a case between
machines. Combine O01/O05/O09 with a logical export that is independent of an incidental live
database directory.

**Recommend portable logical selection before distributed synchronization.** Existing recorded
study descriptors and offline review packs supply foundations, not automatic editable project
exchange. Identity conflicts, reference roots, unavailable artifacts and unsupported providers
need explicit treatment. Neither engine supplies a domain merge algorithm, CRDT, replication,
consensus or multi-store atomicity. KV compatibility readers target their stated legacy formats;
they do not make arbitrary engine directories a stable interchange standard. Preserve their
potential value for real import needs without creating a historical migration obligation for
the current regenerable design-phase data.

### O21 — Incremental observation and scientific data acquisition

<a id="O21"></a>

A frontend, notebook or acquisition process could follow admitted study progress, append
measurement batches, refresh a selected result view and reconnect after a gap. This supports
interactive dynamics and research monitoring without repeatedly reloading a whole project.
Existing live trajectory projections and horizon execution are foundations, not a new historian
or a demonstrated external-plant connector.

**Recommend selected observation with O04/O05; external acquisition is a separate extension.**
Native ordered keys and transactional batches can retain declared acquisition sequences and
consistent progress. Use existing selected/live-prefix owners first. If SurrealDB live queries
or changefeeds are proposed, verify their exact selected server/transport, transaction and
reconnection support; neither KV nor MX supplies a general native application changefeed or
durable message broker. A cursor gap requires explicit catch-up or a fresh coherent selection.

Persist bulk observations in batches, retaining source clock, ordering, units, calibration and
coverage. The database should not become a per-scalar native-solver callback. An incoming value
does not silently change an executing case's admitted inputs; observation, a new input revision
and an external control command have distinct authority and effect settlement. An actual
external-control/digital-twin deployment would need its own timing and safety contract, not
merely a fast storage engine. A useful first demonstration is a reconnecting read-only study
observer that preserves original point/sample order and exposes unavailable gaps.

## Coherent target choices, interactions and priority

The recommended early target is a useful workbench boundary, not a mandatory new platform.
Project work (O01/O03/O04/O05), evidence queries (O06/O07) and topology/payload decisions
(O16/O17) should inform each other before a frontend or universal storage interface freezes.
They can share exact references and selective retrieval without sharing one generic domain model.

| Recommended disposition | Opportunities | Reason and next decision |
| --- | --- | --- |
| Develop first | O01, O03–O07 | Immediate CLI/Python consumers and foundations for the simulator product. Choose a concrete end-to-end project and test-investigation interaction; retain distinct scientific and tooling identities. |
| Resolve architecture alongside first consumers | O16/O17 | Potential to remove meaningful service/file-lifecycle machinery. Compare complete deployed systems and their failure paths fairly; do not normalize an abstraction before selecting necessary native capabilities. |
| Compose with first consumers | O02, O09, O11–O15, selected observation in O21 | Historical context, reproducibility, reuse visibility, query fit and operational explanations improve the workbenches. Several mechanisms already exist and need composition rather than reinvention. |
| Add from comparable retained evidence | O08 | Performance history becomes valuable once sample/cohort identity is established; charts alone do not establish regression or causality. |
| Stage for product delivery needs | O18–O20, external acquisition in O21 | Browser integration, confidentiality, exchange and acquisition have credible value, with explicit platform/format/permission questions. Portable logical exchange can precede collaboration. |
| Adopt only at a demonstrated native consumer | O10 | Native MX is a strong transactional working-set candidate but can otherwise become a redundant cache/index. Its broader canonical deployment remains fairly considered in O17. |

For O06, a derived persistent evidence dataset is useful if repeated cross-run queries justify
incremental ingestion and operation. It is not a replacement terminal composer or universal result
ledger. A one-shot native columnar query is a valid simpler realization. Keep the collector able
to explain database unavailability and expose existing raw/structured owner records independently.

For O16/O17, alternatives are not interchangeable plug-ins. Direct KV needs SI-aware invariant
enforcement and owned indexes; direct MX favors memory-resident state and SSI; embedded SurrealDB
retains higher-level behavior with embedded lifecycle; the current service supports independent
clients and centralized ownership. A split MX/KV design needs a declared publication/replication
model and a concrete benefit. Without that, it adds reconciliation rather than removing it.

Rejected as default additions: another universal scheduler/debugger/query catalogue, whole-store
polling as routine observation, automatic long-lived transactions for UI views, blanket raw-payload
ingestion, custom query/index machinery where existing database/columnar facilities suffice,
database-owned solver state, and a full upstream benchmark/DST campaign as an adoption ritual.
An actual requirement can reopen an alternative; no licence is rejected on policy grounds.

## Further investigations and delivery demonstrations

These are bounded questions owned by the opportunities, not a second backlog or required full
implementation plan. Source inspection can settle semantic questions; probes are for the
remaining consequential uncertainty. New performance claims require appropriately named measurements.

| Question / owner | Useful evidence or representative demonstration |
| --- | --- |
| Evidence projection and real associations — O06/O07 | Import an authentic complete-failing invocation and an interrupted/unreported one twice; obtain typed failure/crate filters with honest denominators; open a protected artifact and a standalone diagnostic; report stale/missing source records. |
| Project workflow — O01–O05 | Fork one exact package/physical closure, change values and structure separately, execute a local study, record it deliberately, compare selected outcomes and trace a fit to original observations. |
| Temporal meaning — O02 | Insert a backdated correction and a soft deletion, compare effective-time views with unchanged exact run receipts, and establish which history survives each deletion/retention operation. |
| Native workspace value — O10 | Implement only a representative metadata operation in a disposable experiment; compare actual synchronization/copies/maintenance with an immutable map or existing query. No speculative native hot-path rollout. |
| Unified payload store — O16 | Exercise bounded native payload/metadata admission, closed multi-chunk membership, protected selection, mixed-lifetime updates/deletes, uncertain commit settlement and a complete recoverable root. Include Arrow decode/copy behavior and physical reclamation, not just logical row counts. |
| Canonical topology — O17 | Account for actual query/schema/index/permission/study consumers, SSI/SI differences, process ownership, ordinary emergency tooling and backup closure. For server-backed KV1.0, first establish adapter API/semantic compatibility. |
| Checkpoint/encryption — O14/O19 | Inspect key dependencies and completion markers; restore or reopen an independent artifact using the correct encryption identity while keeping application authority safe. Fail closed on missing keys/required payloads; test deliberate interrupted recovery only in disposable state. |
| Browser persistence — O18 | Establish whether a chosen pinned implementation connects a complete Tree to OPFS, then demonstrate worker close/reopen, exclusion and quota failure. Primitive build coverage alone is insufficient. |
| Exchange — O20 | Export a selected closure, open it with independent authority, retain original identities/qualification and report unsupported provider or absent payload without inferred live repair. |
| Incremental observation — O21 | Follow an admitted study, disconnect/reconnect and reconcile an explicit cursor gap without inventing samples, reordering data or automatically issuing control effects. |

Upstream model checking, simulation/reference tests, corruption tests and inspection tools are
useful mechanisms and source evidence. Application tests should exercise this application's
selected invariants rather than duplicate millions of upstream test operations. No numerical
result is qualified by a successful database transaction or by a passed metadata query.

## Rule impacts and relationship to existing work

Existing choices inform these proposals but do not veto the explicitly authorized design space.
Any selected rule change proceeds through the normal plan/decision route before implementation.

| Reference | Conditional impact | Decision needed if selected |
| --- | --- | --- |
| RC01 | A derived cross-run evidence workbench adds new functional scope beyond completed Plan50's selected commands. | Define its imported facts, freshness and retention boundary. Existing terminal/resource authorities remain authoritative; no rule change is needed merely to build a derived view. Do not reopen or mark Plan50 incomplete. |
| RC02 | O16 replaces some or all sealed-file publication/custody semantics. | Amend blueprint §20.1–§20.4 and proposed ADR-0187 through a reviewed decision; preserve or replace their actual guarantees, including uncertain effects, strong reads and exact physical reclamation. |
| RC03 | O17 changes the canonical engine, service/embedding boundary or deployment authority. | Revisit D10, blueprint §20.6, proposed ADR-0164/ADR-0214 and relevant operational guards; demonstrate the full alternative instead of treating native KV SI as existing SSI. |
| RC04 | O13 replaces file-backed invocation/resource authority rather than adding a query projection. | Revisit blueprint §24.1 and proposed ADR-0174, including crash prefixes, late registration, composer exclusion, protection and database-unavailable inspection. O06 alone does not require this replacement. |
| RC05 | O11 expands preparation persistence or sharing beyond consumed compatible artifacts. | Preserve blueprint §14 ownership/eligibility and prior no-adoption premises. A whole Salsa image or live evaluator/provider transfer is a separate decision, not a consequence of storage availability. |
| RC06 | A selected project/research/evidence relation family or public Rust/Python document adds domain meaning. | Product relations/documents use the existing registry/catalogue, generation and short ADR route where required; host-only evidence stays with its own declarations/readers. Changing a foundational identity or Python-boundary contract needs the fuller decision route. No parallel schema authority is proposed. |
| RC07 | Browser/offline synchronization or encryption changes supported platform, authority or recovery guarantees. | State the supported platform and actual persistence/security contract; do not inherit native or old-profile guarantees without evidence. |

Plan49f retains independent Python/scientific boundary work; Plan49g retains contracts, maintenance
and TOOL08's native graph-work question; Plan49h retains its qualification/publication scope.
Their coordinator owns their progress. Plan32 owns shared document/artifact lifecycle; Plan40e
owns its assembled diagnostic campaign. This review neither closes nor restarts them. Selected
opportunities may enter those owners where appropriate, or a new implementation series, after
maintainer selection. The review is the unscheduled opportunity owner until that decision.

## Source routes

The complete supplied overviews establish capability awareness; references here identify
consequential checks and integration seams, not a substitute for those documents.

- [SurrealKV overview](../libraries/SurrealKV.md): transactions §§3–4; durability/recovery §§5–6;
  encryption/operations §§7–8; browser/integration §§9–10; limitations §12.
- Operator-supplied local source `docs/libraries/SurrealMX.md`: transactions/access §§3–5; persistence §§6–7;
  diagnostics/platform §8; integration/limitations §§10–12. Its stock-3.3.4 discussion does not
  describe this repository's patched engine as unupgraded.
- [Current architecture](../authoritative_design/sections/architecture-overview.md),
  [identity/publication](../authoritative_design/sections/identity-and-publication.md),
  [workflows/results](../authoritative_design/sections/workflows-and-results.md),
  [scope/limits](../authoritative_design/sections/scope-and-open-design.md).
- [Plan50 Outcome](../plans/50-codex-effectiveness-and-diagnostics.md#outcome),
  [current plans](../plans/README.md), [diagnostic guide](../dev/simulation-diagnostics.md),
  [qualification guide](../dev/validation-assessment.md),
  [substrate guide](../dev/surreal-substrate.md).
- Tooling source paths: `scripts/resource_store.py`, `scripts/test_resources.py`,
  `scripts/test_run.py`, `scripts/run_inspection.py`, `scripts/validation.py`,
  `scripts/validation_receipts.py`, `scripts/validation_scope.py`, `scripts/case_measure.py`,
  `scripts/active_diagnostics.py`. Scientific source paths: runtime
  `workflow/{study,study_demand,study_tables,study_operations,reproducer}.rs`, modeling
  `canonical/local.rs`, `pse-operations` canonical source/study/result owners and campaign
  `packages/reference/campaign/{package.toml,sources.md}`. These identify owners rather than
  frozen symbol inventories or new status ledgers.

[kv-readme]: https://github.com/surrealdb/surrealkv/blob/4338a790c8edc56ef5ea1e46157427af4815be3c/README.md
[kv-lib]: https://github.com/surrealdb/surrealkv/blob/4338a790c8edc56ef5ea1e46157427af4815be3c/src/lib.rs
[kv-tree]: https://github.com/surrealdb/surrealkv/blob/4338a790c8edc56ef5ea1e46157427af4815be3c/src/lsm.rs
[kv-tx]: https://github.com/surrealdb/surrealkv/blob/4338a790c8edc56ef5ea1e46157427af4815be3c/src/transaction.rs
[kv-checkpoint]: https://github.com/surrealdb/surrealkv/blob/4338a790c8edc56ef5ea1e46157427af4815be3c/src/checkpoint.rs
[kv-keys]: https://github.com/surrealdb/surrealkv/blob/4338a790c8edc56ef5ea1e46157427af4815be3c/src/tde/keys.rs
[kv-whitepaper]: https://surrealdb.com/static/whitepapers/surrealkv.pdf
[db-kv]: https://github.com/surrealdb/surrealdb/blob/v3.3.4/surrealdb/kvs-surrealkv/src/lib.rs
[mx-tx]: https://github.com/surrealdb/surrealmx/blob/f67f00cb94f4fb4582090cd86b3b6993f8e25300/src/tx.rs
[mx-db]: https://github.com/surrealdb/surrealmx/blob/f67f00cb94f4fb4582090cd86b3b6993f8e25300/src/db.rs
[mx-cursor]: https://github.com/surrealdb/surrealmx/blob/f67f00cb94f4fb4582090cd86b3b6993f8e25300/src/cursor.rs
[mx-persist]: https://github.com/surrealdb/surrealmx/blob/f67f00cb94f4fb4582090cd86b3b6993f8e25300/src/persistence.rs
