---
title: "Plan 49g: Shared contracts, maintenance and tooling"
status: done
date: 2026-10-21
adrs: []
review_sources: [docs/design_review/reviews/design_review_in-use-library-remediation_2026-10-21.md, docs/design_review/reviews/audits/design_review_in-use-library-remediation_2026-10-21_audit.md]
doc_role: plan
doc_owner: docs/plans/49-library-native-remediation.md
doc_topics: [design, documentation, libraries, storage, validation]
---

# Plan 49g: Shared contracts, maintenance and tooling

Execution state and pause handoff: [the coordinator](49-library-native-remediation.md#pause-handoff--requested-closeout-2026-10-21).

Delivered scope and evidence: [the Outcome](#outcome--2026-10-23). The
[post-Plan51 reconciliation](#remaining-scope-after-plan51--2026-10-23) retains the selected
pre-execution specification; combined acceptance and publication remain with
[the coordinator](49-library-native-remediation.md#outcome--remaining-49f49h-2026-10-23).
The maintainer subsequently authorized [remaining 49f–49h execution](49-library-native-remediation.md#execution-resumption--remaining-49f49h-2026-10-23)
on 2026-10-23. The earlier reassessment, documentation-only reconciliation, pause handoff and minimal
TOOL08 delivery retain their original dates and evidence boundaries.

## Purpose, basis and responsibility

Make the published boundary catalogue, conformance command, launch command and operator
tools consume the same meanings as the product. Separate diagnostic/export tooling from
runtime execution without making tooling a product dependency. This develops R19, R21,
R22 and the relevant DA-011/019/020/023/024/025 portions of the
[integrated review](../design_review/reviews/design_review_in-use-library-remediation_2026-10-21.md).
The [coordinator](49-library-native-remediation.md) owns package state, source dispositions,
rule decisions and shared completion. This document owns the following local design.

**Interface-checked, original authoring baseline:** source at
`e08b1f1dd9b89484ab7471f32b66f8e9c2a59a05` plus the explicitly retained dirty review/audit
inputs. The original target was **Proposed**; the current foundation and remaining scope below
distinguish subsequent implementation and recorded acceptance from work still required.
Authoring or revising this document establishes no new product qualification.
Current contracts remain at blueprint §§4–6, §20, §21 and §23; proposed changes follow
the coordinator's decision route before dependent implementation.

## Current tooling foundation and proposed graph explanations

**Implemented/source-inspected, 2026-10-22:** the compiled direction-aware catalogue, Rust-owned
conformance/launch commands, product selection/detail contracts, internal tooling split and
generated maintenance exist. TOOL06 retains the current observation hub and selects no new crate.
The coordinator owns evidence and still-open consumer/qualification obligations; earlier source
baseline prose below records original defects rather than current absence.

The [remaining-scope review](../functional_reviews/plan49-remaining-scope-and-library-integration-2026-10-22.md#O07)
adds TOOL08 bounded graph explanations and concrete STO09/RES07 maintenance/query consumers.
Extend existing generated operations and native graph identities. Do not create a second graph,
query service, catalogue or observation substrate. New explanation meaning is a product contract
consumed by tooling; place it below tooling in the existing ownership direction.

For STO09 preserve per-item source/cursor/revision checks and original-operation receipts. For
RES07 keep placement/protection premises across joined metadata and file opening; authenticated
physical descriptor checks stay until an admitted descriptor contract replaces them. Query-shape
acceptance examines actual indexes/expansion, not LIMIT alone. `EXPLAIN ANALYZE FORMAT JSON`
is a version-scoped diagnostic, not a mandatory stable parser or a new monitoring system.

## Foundation assessment and selected target

This section preserves the original target rationale. Implemented foundations and resolved
inquiries are reconciled in the current package scope below; they are not reopened here.

At the original authoring baseline, `xtask/src/codegen/schemas.rs` listed document roots; the Python boundary
separately implements `DocumentType` in `crates/pse-py/src/documents.rs`. The pure renderer
in `pse-codegen` already consumes schemars-derived documents. Reuse that renderer rather
than infer a second schema from Python or introduce an ORM. Rust serde owners retain
each document's meaning. One catalogue declares exported name, Rust type, interpretation
version and input/output direction. Both publication and extension registration consume
that catalogue. A type used in both directions has two direction-qualified entries.

Schemars 1.2.2 selects `SchemaSettings::for_serialize()` or `for_deserialize()` before
`into_generator()`. Use distinct generator contexts; do not mutate a constructed generator
between roots. Canonical catalogue identity is framed in `pse-ids`, with ordered exported
names, directions, versions and canonical schema contents. It is not a hash of debug type
names. The extension derives its identity from its actual compiled catalogue. Generated
Python carries the same identity and import refuses a mismatching extension before a
document is interpreted. No direct blake3 dependency outside its existing owner is needed.

The source review's feature-gated Shooting example is not a JsonSchema root and does not
prove catalogue variation. The selected policy is nevertheless precise: compare the actual
direction-qualified compiled catalogue, whatever feature configuration produced it. TOOL01
records whether supported configurations produce the same or different catalogue and emits
the corresponding expected identity with that build's Python products. Do not create a
universal feature fingerprint unrelated to exported contracts.

At that baseline, `python/pse/conformance.py` declared `RunSettings`, `ConformanceRun` and
`ConformanceManifest` directly in msgspec, including defaults that affect execution.
Move their meaning to versioned Rust document owners near the runtime conformance API;
generate Python types through the catalogue. Rust uses the existing `toml` dependency,
Python uses `msgspec.toml` with the generated type. Package-relative paths, selection
coverage, fixture-owned policy precedence and pure/native execution remain explicit.
This is document loading, not a new conformance executor.

The launch protocol already has Rust `SourceRecordingLaunchRequest` and
`SourceRecordingLaunchDocument` owners and an actual launch/drain implementation.
The Python command should invoke that owner. It must not independently implement ACK,
timeout, process-group drain or source coverage. Literal argv stays literal; success of
the child command does not imply scientific source coverage.

### Canonical maintenance and schema tooling

The [storage companion](49a-surrealmx-canonical-store.md) owns SurrealKit beta.6 artifact
embedding, frozen rollout application and sync/rollout parity. Registry-generated SQL
remains schema authority. This document owns migration of operator *domain mutations* to
generated guarded maintenance functions: authority rotation, placement rehoming,
admission freeze and protection checks. SurrealKit handles schema state; it does not
authorize lifecycle transitions, replay arbitrary effects or own result retention.

Beta.6 is already selected in `.config/surrealkit.toml`. Reuse/adapt the existing
`scripts/surreal_rollout_producer.py`, `scripts/surrealkit_maintenance.py` and
`pse-operations::canonical_maintenance` rather than introduce a parallel rollout runner.
Their exact pending nonce/input/phase settlement, authentic child drain and activation
controls are preservation requirements. Include the toolkit's actual client in STO04's
transport inventory; endpoint changes must preserve those controls even when execution
uses a separately built tool. No stock-tool incompatibility is asserted from version alone.

The supervisor retains privileged authentication and deployment orchestration. Maintenance
functions accept the current maintenance generation, operation identity, expected phase
and bounded page cursor, and return typed progress/refusal. Page replay is idempotent for
the same operation. Freeze prevents new placement-sensitive effects; already admitted
work drains or is reconciled by its existing owner. Schema bootstrap, historical restore
and activation are a separately authorized maintenance phase: current ordinary-writer
guards cannot be required before the schema defining them exists. The lifecycle checks
apply once installed, with an explicit activation point. No claim of one ordinary writer
is made for privileged historical installation.

Choose bounded result reclamation on existing `forget_*`/`reclaim_*` and primary-activation
triggers, with a callable on-demand continuation. The cursor and outstanding protected
payload references stay canonical. Do not add a collector daemon. Protection/lease checks
must precede deletion, and interruption resumes the same bounded operation. This settles
the DA-025 trigger choice while preserving its original participants and owner-identity
binding; it does not revive the challenged diagnosis that those participants are unnecessary.

### Product contracts before tooling placement

Product code uses `capture::DetailSession` and canonical APIs use
`review_pack::PackResultSelection`. Moving those modules wholesale into a crate that
depends on runtime creates a cycle. First put live observation/detail session contracts
with runtime/diagnostics owners and canonical result-selection meaning with the result
owner. Tooling consumes these products to record, export, compare and inspect.

Select an internal tooling module boundary initially. This gives the dependency correction
without inventing a crate solely to move files. A new crate is an explicitly bounded
optional follow-up only if that boundary has independent consumers or build/dependency
benefit; RC09 includes that route but does not require a crate. The eleven product users
of the observation hub continue to depend on `pse-diagnostics`.

Retain the existing observation hub while a bounded tracing-Layer inquiry compares actual
propagation, attribution, cancellation, current state, final flush and cost. A subscriber
layer is not selected merely because tracing is linked. If selected, the layer stores
per-span aggregation in `extensions_mut()` and there is one observation substrate after
consumer cutover. If not selected, remove redundant propagation where the remaining owner
permits it; do not retain two competing current-state authorities.

## Packages and implementation visions

Package progress is recorded only in the [coordinator's package table](49-library-native-remediation.md#package-state).
The following table and package descriptions preserve the selected pre-execution residual
obligations, dispositioned in the Outcome. Earlier foundation discussion preserves the original
design rationale, not instructions to recreate it.

| Package | Prerequisite capability | Delivered behavior and acceptance |
| --- | --- | --- |
| TOOL01 | Implemented catalogue, generated registrations and standalone envelopes | Complete focused direction/version/import acceptance, including the boundary-schema mismatch branch; retain the single declaration and actual-build identity |
| TOOL02 | Implemented Rust conformance documents and generated Python loader | Exercise existing Rust/Python defaults, strict decoding, paths and selected-versus-package coverage controls; no new owner migration |
| TOOL03 | Implemented Rust launch owner and thin Python adapter | Exercise actual command/ACK, cancellation, timeout and bounded-prefix log-drain outcomes; no second protocol or adapter rewrite |
| TOOL04 | Implemented generated maintenance, supervisor adapters and selected recovery evidence | Complete remaining live generation/replay/refusal/protection controls against the current catalogue and payload owners |
| TOOL05 | Implemented product contracts and internal tooling split | Reconcile capture/detail/review/CLI consumer behavior not covered by selected Plan51 journeys; no repeated module move |
| TOOL06 | Completed I21 no-adoption outcome | Retain current observation hub and internal module; no additional Layer/crate inquiry |
| TOOL07 | Implemented pin explanation and I24 purpose/retirement decisions | Reconcile current documentation and finding/package dispositions; remaining cross-area publication stays with QUAL05 |
| TOOL08 | Implemented typed graph API/catalogue and operator command | Establish a native examined-edge/work bound with an actual seek/range/cursor path; preserve current semantics, continuation and protection |

## Remaining scope after Plan51 — 2026-10-23

This reconciliation preceded resumed execution. The [Outcome](#outcome--2026-10-23) records
implementation and actual acceptance; the clauses below retain their original evidence limits.

**Interface-checked:** source `d16a253ac` and the recorded Plan49/50/51 Outcomes. The
maintainer selected the recommended scope reconciliation on 2026-10-23. No new rule change,
library selection or production repair is authorized by this document revision.

**Implemented:** TOOL01–TOOL05 foundations already exist. The sole exported catalogue is
`for_each_boundary_document!` in `pse-runtime::boundary_documents`; schema publication and
native/Python registration consume it. Standalone envelopes have explicit version headers.
Conformance meaning lives in `workflow::conformance_documents`; Python source recording calls
the Rust `tooling::review_launch` owner. Product selection lives in `workflow::result_selection`,
detail contracts in `observation_detail`, and export/review composition in `tooling`.
Generated maintenance and its supervisor callers are present. These are not new implementation
tasks merely because an older checkpoint said publication or compilation was pending.

**Tested, reused at original conditions:** [Plan51g's Outcome](51g-qualification-and-publication.md#verification-and-outcome--assembled-execution-2026-10-23)
records generator equivalence/governance, default/no-default/native all-target Clippy, actual
installed Python consumers, selected project/research/evidence/protected-copy journeys, host
supervisor controls and nonempty disposable recovery. Those establish their named compilation,
generation and behavioral scope. They do not run every existing conformance, launch, live
maintenance or review-tool consumer test. The original mixed-concurrency functional failures
and repaired feature checks retain their recorded conditions; do not recast them as a clean
whole-product run.

The residual packages below are behavioral acceptance and any concrete repair it exposes,
plus TOOL08's native work bound. QUAL01 first matches existing assertions and prior Outcomes
to these obligations, then selects uncovered or invalidated controls. Reuse actual evidence;
do not create another test catalogue or infer a current code gap from historical pending prose.

Plan51 broadens the affected consumers to project/research/lifecycle/import documents, copied
protected results and current backup-v4 application inventories. TOOL01/04/05 must include those
consumers where their changes reach them. Reuse original source/project/artifact activation,
pending-import visibility fences and exact generation/receipt settlement. Descriptive/offline
availability must not become executable availability or deletion authority. Existing lifecycle
and project-closure explainers remain their owners: TOOL08 need not replace them or grow into
a universal project graph. [49h](49h-qualification-and-design-publication.md#post-plan51-acceptance-reconciliation--2026-10-23)
owns the cross-area acceptance selection. Completed Plan51 packages remain complete at their
published boundaries.

### TOOL01: Catalogue, identity and generated registration

The catalogue, Completion Version1, standalone wrappers and generated native/Python registrations
are implemented. Preserve one declaration for membership, name, direction, version and wire
projection. Change declarations/renderer and regenerate only when a concrete missing or incorrect
contract is found; do not recreate wrappers or a registration inventory.

Cover input-only defaults, output-only fields, a type exported both ways, a changed version,
unknown fields and a genuine mismatched extension/Python pair. Compare generated schemas
with independently specified direction expectations, not only generator round trips.
The existing `test_native_compatibility_refuses_version_or_fingerprint_mismatch` covers package
version and registry identity, but not the boundary-schema fingerprint refusal in
`pse._build._check_native_compatibility`; add that negative control with the other identities
valid. Existing `boundary_documents` direction/header and `test_standalone_document_wire.py`
controls supply independent expectations. I19 already records equal actual default/native
catalogues; broad feature compilation does not itself compare every exported schema identity.
Reopen identity variation only for an actual changed exported shape. BND side-route adoption
retains its own owner and is not a prerequisite to reauthor catalogue foundations.

### TOOL02–TOOL03: Commands become consumers

The Rust owners and thin command adapters are implemented. Preserve argument parsing and
formatting. For conformance, retain the distinction between package coverage and
selected fixture coverage; an unknown fixture fails before any fixture runs. The runtime
continues to apply fixture-authored execution policy. A manifest with omitted settings uses
the Rust owner's documented defaults, rather than Python restating them.

For source recording, call the Rust launch operation with immutable owned request data
before scientific construction. Report command completion, selection ACK and drain separately.
Exercise a command producing logs beyond the retained prefix: pipes must continue draining.
The Python protocol/state machine has already been removed. Exercise the existing
`workflow::conformance_documents` and Python modeling/conformance controls, Rust
`tooling::review_launch` execution controls and public review command journeys. Plan51's
project/evidence commands and frozen-review readback cover adjacent consumers, not this complete
ACK/timeout/drain acceptance. Keep outcome tests without restoring a deleted mechanism.

### TOOL04: Operator mutation and reclaim cutover

Generated functions and supervisor adapters are implemented. Reconcile remaining direct canonical
writes to their current-writer, maintenance or privileged installation phase; repair only concrete
unmigrated domain effects. Reuse the existing registry/generator and typed refusal decoder. Plan51's
host controls cover supervisor routing, much of it mocked; its actual recovery covers selected
restore/activation. Neither alone substitutes for live function behavior.

Use the current supported schema and backup-v4 closure, a resumed half-complete page,
an obsolete generation and a protected payload. Start with the existing live
`generated_maintenance_pages_fence_generations_and_replay_exact_effects` control and reconcile
its assertions against remaining protection/refusal obligations. Assert that freezing/rehoming cannot make
a successful acknowledgement refer to unprotected bytes. Retain necessary paged readers
until STO record-id access has replaced their key-index assumptions. Delete direct domain
mutation SQL if any remains after equivalent generated functions are exercised. Include new
project/research/draft/pending-import records in current inventory and activation coverage where
those owners participate; no historical-schema conversion is required.

The storage engine migration is a coordinated maintenance operation. Never infer that
SurrealKit definition rollback restores changed data or that a logical export contains
engine-specific history. TOOL04 and STO cutover share an activation contract, not two
independent migration ledgers.

### TOOL05–TOOL06: Dependency split and bounded observation inquiry

Product consumers and export/comparison implementations have moved to the selected owners.
Plan51 exercises selected project, research, evidence, protected pack and frozen-review consumers;
full Clippy compiles the broader target graph. Reconcile remaining capture/detail, review-pack,
review-analysis and public review-workflow behavior against actual selected tests. Compile success
does not execute those consumers. Preserve explicit projections where a product contract differs
from an export envelope; no repeated module move or new crate is selected.

TOOL06/I21 is complete: actual no-layer/current-hub/candidate-Layer steady/fit and bounded-lifetime
controls support retaining the current hub. The candidate lacked corresponding native-worker
phase coverage, and no independent crate benefit was established. Preserve that no-adoption
decision, cancellation/final closure and bounded retained state. No fresh tracing comparison is
required by Plan51's additional consumers.

The optional crate outcome additionally needs a dependency graph with no runtime→tooling
edge and an ADR before crate addition. It does not block the module correction.

### TOOL07: Purpose decisions and publication

The statrs manifest explanation already names feature unification through egobox-moe and
explicitly rejects a crossing-type defect. I24's selected purpose is implemented: retain the
native-child lane for bounded provider-free process-isolation experiments; the unused
`pse.canon.v2` relation engine/API/tests are removed. Shared identity/allocation contracts remain.
Do not schedule another pin-note fix, purpose inquiry, canonicalizer removal or family upgrade.
Reconcile finding/inquiry/package text with these facts and current enduring owners.

Plan49's published outcomes and Plan51's blueprint revision255 already document their delivered
scope. Only remaining contract changes need the decision/revision route; QUAL05 owns final
cross-area publication and completion. Leave accepted ADRs immutable and current Plan28/32/40
statuses at their existing owners. No new code change is established by this reconciliation.

### TOOL08 — bounded graph explanations

Provide typed immediate-blocker, recorded-source and retained-consumer queries over
`canonical_study_dependencies`, `canonical_run_sources`, `canonical_analysis_edges` and
`canonical_analysis_inputs`. Return node/edge identity and role, selected revision/evidence,
requested neighborhood scope and explicit complete/partial continuation. An observed blocker
is not proof of a complete explanation. Bound examined edges, returned extent and retained
metadata; do not silently call a truncated traversal complete. Source/protected artifact access
continues through the current receiving authority.

Begin with immediate blockers and bounded source/consumer neighborhoods; do not enumerate all
paths or persist an all-pairs closure. Preserve isolates and parallel edge distinctions where
they belong to the question. Graph connectivity is neither numerical dependence nor scientific
usability. Existing petgraph/rustworkx preparation algorithms remain separate from canonical
retrieval; GQL is an optional expression over existing relations, not a second graph/API service.

Publish the product response through TOOL01/BND05 and migrate one actual operator inspection
consumer. Targeted tests cover a known blocker, independent point, distinct edge kinds,
page-plus-one continuation, changed selected revision, protected reference and incomplete scope.
Independent expected graphs/facts specify results. Keep frontend formatting outside the semantic
query, and remove any replaced flat-page client stitching once its consumers move.

### TOOL08 selected Plan 50 dependency checkpoint — 2026-10-22

**Implemented:** the selected dependency supplies typed immediate-blocker, recorded-source and
retained-consumer responses in `pse-operations::canonical_explanations`, published through the
existing boundary catalogue. `canonical-graph-inspect` is the first operator consumer; its
explicit database selector uses the existing canonical receiving authority. The response keeps
original directed identities and parallel edge roles, protected evidence locators, isolates,
revision/generation-bound continuation and explicit local completeness. Shared study policy
selects the observed blocker. No payload acquisition, second graph or client stitching path
was introduced.

**Tested:** the two `canonical_graph_explanation` controls and the adjacent readiness-query
control passed (3 selected tests, zero failures; 14.725 seconds) through the existing C04
reference lifetime with explicit correctness validation. Both real operator commands also ran
while their registered fixtures remained owned. Conditions included three study points with
an independent point and two prerequisite kinds, a three-node analysis with two parallel edge
roles and an isolate, one protected artifact reference, a one-edge page with its sentinel,
changed selection refusal and an 80-point readiness fixture. The reference owner drained,
quiesced and stopped; broader Plan 49 qualification was not run. Plan 50 D01 owns the adjacent
query-inspection acceptance and native physical-plan evidence.

**Unresolved at the 2026-10-22 checkpoint, since resolved below:** the selected response
bounds decoded rows to the requested
page plus one, retained output to 256 KiB and the request to its original clock. It does not
establish a strict bound on native edges examined. Actual native plans retained a downstream
Filter and limit; positive cache observations do not establish folded-block use. The exact
[3.3.4 planner source](https://github.com/surrealdb/surrealdb/blob/1ac2b1dba7cd177621a4cf5d507caebeb2414395/surrealdb/core/src/exec/planner/source.rs)
excludes predicates in the full projection pipeline from graph pushdown, leaves its direct
key-residency recognizer disabled and counts pushed limits after visible filtering. Merely
changing to native IDs and literal limits therefore cannot establish a bounded physical scan
for continuation pages. Choose and test an actual composite-index seek or native range/cursor
mechanism when completing this remaining Plan 49g obligation. This checkpoint accepts only the
minimal semantic/API/decoded-output dependency for Plan 50; it neither closes TOOL08 nor resumes
other Plan 49 packages.

The remaining implementation must distinguish edges examined from returned rows and retained
bytes. Exercise a selective neighborhood with unrelated edges and later continuation pages;
inspect the actual access path and use an independent expected edge set. A downstream filter,
page-plus-one decode, cached adjacency or elapsed-time cutoff alone cannot establish the native
work bound. Preserve exact cursor/revision refusal, parallel roles and the existing operator
consumer while replacing any unbounded query shape. No all-path closure, new graph service or
new lifecycle-release authority is selected.

PRE09's optional configuration documents use the same catalogue only after explicit selection.
No new Python policy engine, raw SMT endpoint or hidden default becomes a tooling convenience.

## Verification and limits

**Tested, existing evidence:** the coordinator's Outcomes, TOOL08 dependency checkpoint and
linked Plan51g Outcome own the original commands, conditions and exclusions. Source inspection
for this revision is **Interface-checked**, not a new behavioral pass.

**Original remaining acceptance, now dispositioned in the Outcome:** targeted catalogue/import refusal, conformance/launch,
live interrupted maintenance/protection, uncovered review-consumer behavior and TOOL08 native
work-bound controls. Compile affected code with explicit correctness validation where required;
regenerate any changed declaration as part of that change. Do not repeat satisfied generation
or compile obligations solely because older notes called them pending.

**Assembled acceptance, owned by the coordinator:** QUAL packages exercise schema install/rollout parity,
current managed server/restore activation and affected Rust/Python command journeys. No new
observation substitution or historical data migration is selected. QUAL01 reconciles existing
evidence before further execution. Documentation validation cannot qualify those behaviors;
the Outcome distinguishes actual subsequent checks from this earlier specification.

## Source routes

- [Integrated review §4.7–§4.8](../design_review/reviews/design_review_in-use-library-remediation_2026-10-21.md#47-c7-boundary-vocabularies-identities-and-documents-are-hand-paired) and [audit companion](../design_review/reviews/audits/design_review_in-use-library-remediation_2026-10-21_audit.md).
- [SurrealDB overview](../libraries/surrealdb.md), especially SurrealKit and operational boundaries; tagged [beta.6 migration notes](https://raw.githubusercontent.com/surrealdb/surrealkit/v1.0.0-beta.6/MIGRATING.md).
- [Schemars contract documentation](https://github.com/GREsau/schemars/blob/master/docs/3-generating.md), reconciled with installed 1.2.2 `SchemaSettings` in the audit evidence.
- Source entry points: `pse-runtime::boundary_documents`, `xtask/src/codegen/schemas.rs`, `pse-codegen::codegen::documents`, `pse-py::documents`, `python/pse/conformance.py`, `pse-runtime::tooling::review_launch`, `scripts/surreal_server.py`, `pse-operations::canonical_maintenance` and product consumers named above.

## Fresh-store integration boundary

The maintainer's 2026-10-21 fresh-data hard pivot removes historical backfill and
retained-data cutover. STO07 consumes the actual privileged installation/activation
interface and affected existing callers; full TOOL04 is not a prerequisite by label.
Broader generated domain maintenance remains owned here and consumes STO lifecycle/schema
interfaces. Existing-data conversion is not required.

## Outcome — 2026-10-23

### Delivered contracts

**Implemented:** TOOL01–TOOL08 are complete at their selected implementation and acceptance
boundary, reconciled against integrated `285b7a8b1` and the adjoining boundary consumers through
`42db023d7`. TOOL01 adds the boundary-schema fingerprint mismatch refusal with otherwise valid
identities. Graph request/response documents now own their required version at the operations
product owner; the existing catalogue generates native registrations, schemas and Python
documents from those actual types. No parallel document inventory was introduced.

TOOL02–TOOL05 retain the implemented Rust conformance/launch owners, generated maintenance and
product/detail/tooling split. Focused acceptance exercises actual defaults, selected coverage,
ACK/timeout/cancellation/log drain, capture/review consumers and live paged maintenance. The live
maintenance control includes exact interrupted-page replay, generation refusal and protected
payload behavior. No second launch protocol, diagnostics substrate or maintenance SQL path was
added. TOOL06's current-hub/no-new-crate decision and TOOL07's existing purpose/pin dispositions
remain complete; their previously satisfied inquiries were not reopened.

TOOL08 replaces projection-filter pagination with existing named scalar compound-index seeks
for study predecessors/consumers, run sources, artifact consumers and analysis-node neighborhoods.
Requests bind identities and the exclusive continuation cursor, force the relevant existing index,
order by its matching suffix and render only the validated numeric page-plus-one limit as a
literal. Every returned edge, including the sentinel, is checked against the selected owner,
revision, role, endpoints and ordering before a continuation is published. The typed local
explanation, isolates, parallel roles, protected evidence and `canonical-graph-inspect` consumer
retain their meanings. Connectivity grants neither scientific usability nor payload acquisition.

Native plan observations expose actual JSON scalar access/limit/predicate/counter facts and keep
missing counters distinct from zero. Real plans use the named `IndexScan` range without a
downstream Filter/Sort/TopK for these requests. The separate exact-source scanner probe checks
raw pair consumption rather than inferring it from returned rows or elapsed time. No production
engine instrumentation, new index/schema, graph service or all-path closure was introduced.

### Verification and limits

**Tested, 2026-10-23; zero-failure target:** the combined explicitly validated Rust selection
(`pse-runtime` and `pse-operations`, nextest, four test threads, current supervised native service)
ran 73 tests: 70 passed and three graph controls failed. Invocation
`0aacb7791611466087bdb977a7cdc3e2` and `/tmp/pse49fgh-rust-acceptance-open-service.log`
retain that result. The passed scope includes catalogue/wire/conformance, launch/capture/review,
live maintenance and root-role refusal controls; the aggregate was not a clean pass.

**Tested:** after correcting native plan scalar observations and the literal scan cap, the exact
three failed graph controls plus six pure graph controls passed together (nine tests, three
threads, 51.724 seconds; invocation `ad7c252823764b5282ceda2afd35e00f`,
`/tmp/pse49fgh-graph-acceptance-rerun.log`). Known blocker, isolates, distinct/parallel roles,
protected inputs, dense seek continuation, changed selection and corrupt sentinel refusals are
covered. An exact rerun of repaired tests does not turn the earlier failed run into a clean one.

**Tested:** the installed force-validated native Python boundary/conformance selection passed
32 tests in 4.54 seconds (invocation `d2205e99ec254c4a9607c08deab011e0`). The focused review
unit command `scripts/pse-env --resource-class build -- .venv/bin/python -m scripts.python_tests
python/pse/tests/test_review_workflows.py --unit-only -n4 -q` passed 12 tests (invocation
`d5716de7776c41cf9dda444705210163`). The host command
`scripts/pse-env --resource-class light -- .venv/bin/python -m unittest
scripts.tests.test_surrealkit_maintenance scripts.tests.test_surreal_scan_budget_probe` passed
43 tests. Host protocol/probe tests do not replace the separate live maintenance control.

**Tested, exact-source probe only:**
`exec::index::iterator::btree::tool08_probe::tool08_compound_range_scan_stops_at_entry_budget`
passed once in the isolated `kv-mem` scanner build, covering root OWNER and VIEWER, 2,400 eligible
post-cursor entries, 300 unrelated-prefix entries and caps of 65 and 1,001, including the native
batch boundary. The receipt is
`/home/paul/.cache/pse-arrow/tool08-scan-probe-20261023-b/probe.json`. It records pinned revision
`1ac2b1dba7cd177621a4cf5d507caebeb2414395`, the exact copied source/patch/lock hashes and
native-path equality. The bound is on raw KV pairs consumed for the scalar compound-index,
exclusive-cursor, literal-limit path under the tested root roles without conditional row
permissions. This is not whole deployed-binary equivalence, an I/O bound or a latency measurement.

Exact command selections, the two separately passed canonical review components, final covariance
reopening, generated-output and static results, and the final composed qualification account
belong to [the coordinator's final Outcome](49-library-native-remediation.md#outcome--remaining-49f49h-2026-10-23).
Existing Plan51 evidence retains its original conditions. No whole-product, every-library or
performance qualification is inferred from these selected checks.

### Mistakes corrected and deliberate deviations

The first graph implementation improved semantic pagination but still left filtering above the
native scan. Existing compound indexes now provide owner-prefix/exclusive-cursor seeks. Plan
inspection initially serialized Surreal values as tagged objects and used a bound limit that
hid the concrete cap from the independently specified assertion. Observations now expose JSON
scalars and the validated numeric cap is literal; user identities/cursors remain bound values.
An initial root-role assertion checked the wrong diagnostic meaning; the repaired test checks
the actual root OWNER/VIEWER contract without admitting conditional record permissions.

The graph documents initially lacked their standalone version at their product owner. The
version moved to the actual request/response types and all derived schemas/registrations/codecs
were regenerated; a catalogue-only wrapper was not treated as an equivalent wire owner. Runtime
acceptance also required consuming lazy Arrow products and using one actual downstream parent
revision, as recorded in [49f](49f-python-and-scientific-boundaries.md#outcome--2026-10-23).

Deliberately, the scan inquiry instruments an isolated exact-source copy rather than the
production engine. It combines native scanner evidence with actual receiving-service access
plans and semantic controls, retaining the narrower entry-consumption claim. Existing indexes,
TOOL06's observation owner and completed Plan51 qualification were reused. Historical-schema
conversion, a new crate/layer, optional configuration exploration and unrequested benchmark
campaigns were not selected. Cross-area architecture publication and retirement remain with 49h
and the coordinator.
