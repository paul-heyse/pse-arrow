---
title: "Plan 49g: Shared contracts, maintenance and tooling"
status: in-progress
date: 2026-10-21
adrs: []
review_sources: [docs/design_review/reviews/design_review_in-use-library-remediation_2026-10-21.md, docs/design_review/reviews/audits/design_review_in-use-library-remediation_2026-10-21_audit.md]
doc_role: plan
doc_owner: docs/plans/49-library-native-remediation.md
doc_topics: [design, documentation, libraries, storage, validation]
---

# Plan 49g: Shared contracts, maintenance and tooling

Execution state and pause handoff: [the coordinator](49-library-native-remediation.md#pause-handoff--requested-closeout-2026-10-21).

## Purpose, basis and responsibility

Make the published boundary catalogue, conformance command, launch command and operator
tools consume the same meanings as the product. Separate diagnostic/export tooling from
runtime execution without making tooling a product dependency. This develops R19, R21,
R22 and the relevant DA-011/019/020/023/024/025 portions of the
[integrated review](../design_review/reviews/design_review_in-use-library-remediation_2026-10-21.md).
The [coordinator](49-library-native-remediation.md) owns package state, source dispositions,
rule decisions and shared completion. This document owns the following local design.

**Interface-checked:** authoring source at `e08b1f1dd9b89484ab7471f32b66f8e9c2a59a05`
plus the explicitly retained dirty review/audit inputs. **Proposed:** target and packages.
No product implementation or qualification is established by this document.
Current contracts remain at blueprint §§4–6, §20, §21 and §23; proposed changes follow
the coordinator's decision route before dependent implementation.

## Foundation assessment and selected target

The current `xtask/src/codegen/schemas.rs` lists document roots; the Python boundary
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

`python/pse/conformance.py` currently declares `RunSettings`, `ConformanceRun` and
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

| Package | Prerequisite capability | Delivered behavior and acceptance |
| --- | --- | --- |
| TOOL01 | Existing document owners and generated document renderer | One direction-aware catalogue, actual compiled fingerprint, strict import comparison; catalogue drift and serialize/deserialize defaults tested independently |
| TOOL02 | TOOL01 catalogue registration and generated vocabulary inputs | Rust-owned versioned conformance manifest and defaults; both loaders agree on explicit/omitted controls, unknown fields, relative paths and selected coverage |
| TOOL03 | Existing Rust launch owner; TOOL01 for typed projection | Python diagnostics invokes the Rust owner; ACK failure, cancellation, timeout and bounded log drain have the same result meanings |
| TOOL04 | STO lifecycle/refusal contracts, schema installation/freeze and maintenance pages | Supervisor mutations use generated functions; bootstrap/restore/activation remain privileged phases; interrupted rehoming/reclamation resumes without losing protection |
| TOOL05 | RES result-selection contract and existing diagnostics/detail contracts | Product contracts separated before capture/review tooling modules move; no runtime-to-tooling dependency; existing product and command consumers migrated |
| TOOL06 | TOOL05 semantic boundaries and comparable steady/fit paths | Bounded tracing-Layer and optional crate inquiry, with adopt/no-adopt outcomes; no unqualified substrate replacement |
| TOOL07 | Selected package outcomes and coordinator rule decisions | Correct stale text/pin explanations, purpose decisions for unused machinery and enduring documentation handoff without importing unrelated plans |

### TOOL01: Catalogue, identity and generated registration

Inspect all current schema roots, `DocumentType` implementations, bytes-returning boundary
methods and generated Python decode registrations. One declaration supplies membership,
name, direction and version; it does not duplicate a struct's fields. Add `Version<N>` to
an exported unversioned root such as `Completion` through the appropriate boundary change.
Generate the catalogue's Rust/Python bindings and schema products together; edit declarations
and renderer, then `just codegen`, never hand-edit generated paths.

Cover input-only defaults, output-only fields, a type exported both ways, a changed version,
unknown fields and a genuine mismatched extension/Python pair. Compare generated schemas
with independently specified direction expectations, not only generator round trips.
Inspect all supported export feature configurations before recording invariance or variation.
Delete the old xtask/boundary membership lists once every consumer uses the catalogue.
This package consumes BND vocabulary/identity contracts; it need not wait for every Arrow
side-route migration to begin catalogue design.

### TOOL02–TOOL03: Commands become consumers

Preserve the command's argument parsing and formatting while removing interpretation from
the command adapter. For conformance, retain the distinction between package coverage and
selected fixture coverage; an unknown fixture fails before any fixture runs. The runtime
continues to apply fixture-authored execution policy. A manifest with omitted settings uses
the Rust owner's documented defaults, rather than Python restating them.

For source recording, call the Rust launch operation with immutable owned request data
before scientific construction. Report command completion, selection ACK and drain separately.
Exercise a command producing logs beyond the retained prefix: pipes must continue draining.
Delete the Python protocol/state machine after the owner route passes, keeping CLI behavior
tests that specify user outcomes rather than tests of the deleted mechanism.

### TOOL04: Operator mutation and reclaim cutover

Inventory the supervisor's direct canonical writes and assign each to a current-writer,
maintenance or installation phase. Extend the existing schema registry/generator rather
than publish another handwritten function schema. Migrate runtime and script callers in
the same package; no script path may silently bypass the typed refusal decoder.

Use a restore fixture from the supported historical schema, a resumed half-complete page,
an obsolete generation and a protected payload. Assert that freezing/rehoming cannot make
a successful acknowledgement refer to unprotected bytes. Retain necessary paged readers
until STO record-id access has replaced their key-index assumptions. Delete direct domain
mutation SQL from the script after equivalent generated functions are exercised.

The storage engine migration is a coordinated maintenance operation. Never infer that
SurrealKit definition rollback restores changed data or that a logical export contains
engine-specific history. TOOL04 and STO cutover share an activation contract, not two
independent migration ledgers.

### TOOL05–TOOL06: Dependency split and bounded observation inquiry

Migrate product consumers first, including durable workers, study diagnostics, connected
result selection/restore and PyO3 detail sessions. Then move export packing and comparison
implementation behind the internal tooling module. Public document meaning is preserved
by explicit projections where a product contract differs from an export envelope.

For tracing, compare no layer, current hub and candidate Layer on the same steady solve
and fit. Success requires the same admitted current phases, native/DataFusion context
propagation, final closure and bounded retained state; cancellation/panic cannot strand a
phase. Record CPU/allocation impact if cost is the selection reason. Reject the candidate
if those semantics or burden worsen. A missing benefit is a valid no-adoption outcome.

The optional crate outcome additionally needs a dependency graph with no runtime→tooling
edge and an ADR before crate addition. It does not block the module correction.

### TOOL07: Purpose decisions and publication

Correct the statrs hold explanation to match actual feature unification through egobox-moe;
the two nalgebra versions alone are not a crossing-type defect. Do not upgrade a family
merely to make a note true. Trace the native child lane and `pse.canon.v2` consumers and
present a concrete retain-for-named-exploration or remove decision. Without an explicit
purpose decision they remain unscheduled for deletion; they are not a reason to broaden
this series into process isolation or a second canonical protocol.

After implementing selected work, reconcile blueprint §§0.5, 7, 13–14, 18–21 and 23 with
the actual owners. Do so through confirmed rule changes and revision rows, leaving accepted
ADRs immutable. Current Plan 28/32/40 statuses remain at their existing owners. The final
qualification companion owns cross-area completion and publication checks.

## Verification and limits

**Proposed:** targeted catalogue/generator tests; command policy tests; interrupted
maintenance/reclamation controls; product/tooling dependency assertions; source-recording
drain controls. Compile affected packages with explicit correctness validation where required.
Generation is part of the functional package, not later polish.

**Proposed assembled acceptance:** QUAL packages exercise schema install/rollout parity,
actual managed server migration, Rust/Python command journeys and selected observation
substitution. No test, benchmark, source regeneration or command journey has run as a
result of authoring this plan. Documentation validation of the series cannot qualify them.

## Source routes

- [Integrated review §4.7–§4.8](../design_review/reviews/design_review_in-use-library-remediation_2026-10-21.md#47-c7-boundary-vocabularies-identities-and-documents-are-hand-paired) and [audit companion](../design_review/reviews/audits/design_review_in-use-library-remediation_2026-10-21_audit.md).
- [SurrealDB overview](../libraries/surrealdb.md), especially SurrealKit and operational boundaries; tagged [beta.6 migration notes](https://raw.githubusercontent.com/surrealdb/surrealkit/v1.0.0-beta.6/MIGRATING.md).
- [Schemars contract documentation](https://github.com/GREsau/schemars/blob/master/docs/3-generating.md), reconciled with installed 1.2.2 `SchemaSettings` in the audit evidence.
- Source entry points: `xtask/src/codegen/schemas.rs`, `pse-codegen::codegen::documents`, `pse-py::documents`, `python/pse/conformance.py`, `pse-runtime::capture::launch`, `scripts/surreal_server.py`, `pse-operations::canonical_maintenance` and product consumers named above.

## Fresh-store integration boundary

The maintainer's 2026-10-21 fresh-data hard pivot removes historical backfill and
retained-data cutover. STO07 consumes the actual privileged installation/activation
interface and affected existing callers; full TOOL04 is not a prerequisite by label.
Broader generated domain maintenance remains owned here and consumes STO lifecycle/schema
interfaces. Existing-data conversion is not required.
