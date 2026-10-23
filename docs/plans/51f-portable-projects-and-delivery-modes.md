---
title: "Plan 51f: Portable projects and delivery modes"
status: in-progress
date: 2026-10-22
adrs: [ADR-0220]
review_sources: [docs/functional_reviews/surrealkv-surrealmx-functional-extensions-2026-10-22.md]
scenario_sources: [docs/functional_reviews/surrealkv-surrealmx-functional-extensions-2026-10-22.md]
doc_role: plan
doc_owner: docs/plans/51-engineering-workspaces-and-evidence.md
doc_topics: [current-work, design, storage, numerical-execution]
---

# Plan 51f: Portable projects and delivery modes

## Selected delivery target

**Proposed:** export a selected logical project closure, inspect it without its original service
and import it under fresh local authority for ordinary editing/study operations. Resolve the
bounded browser/offline and collaboration questions without assuming a frontend, live replication
or multi-user platform is required. O18/O20, I06/I08 and package state belong to the
[coordinator](51-engineering-workspaces-and-evidence.md).

Logical exchange is the selected portability boundary. It composes LC01's exact copied closure,
existing admitted source/physical identity and PJ01's project ownership. Engine directories,
native transaction handles, server credentials and old protection grants are not a portable
project format. A local/ephemeral modeling or study operation can remain store-independent as
today; durable project import/edit uses the chosen canonical service. Offline inspection is not
an implicit selection of embedded SurrealDB or direct KV.

## Export and import meaning

The versioned manifest names selected source/physical package closure, chosen raw/curated dataset
versions, calibration/context, parameter-publication ancestry, study definitions and requested
result/evidence members. Include original identity, codec/version, digest, qualification,
completeness/omissions and provider requirements where relevant. Provider descriptions are
requirements, not executable handles or permission to install/download them. Intentional omission
and unsupported/missing required closure are distinct. Reuse existing schema/codec authorities;
do not infer source schema from file shape.

Hold current protection during the selected copy, verify destination members and publish the
manifest last. Offline readers validate that copied closure and never auto-follow old absolute
paths, original service addresses or live authority grants. Source/result identities and scientific
qualifiers retain their meaning, while physical placement and imported project ownership are new.
Allow a metadata/descriptive export when explicitly requested, clearly distinct from a complete
editable/executable closure.

Import validates the manifest and selected closure, checks supported codecs/providers and creates
a fresh project/source owner through ordinary admission. Do not overwrite an existing source
problem, replay historical operation IDs as current effects, infer a terminal or silently merge
into the current branch. A user-requested merge is PJ04's exact-parent objectwise operation after
import. Stage large copies outside short catalogue transactions; activate only the reconciled
closure. Interrupted imports and uncertain activation settle their original operation without
duplicating a project. Existing generated data need no legacy migration or preservation campaign.

Historical run/attempt references stay origin-qualified evidence locators. They are not receiving
canonical run/attempt locators, local producer receipts or grants to use a result as an execution
seed. Copying a digest/label cannot mint those authorities. Any supported new execution or seed
use passes the existing receiving admission/eligibility and protection owners explicitly; preserve
an origin-to-import mapping rather than recasting historical terminals as local execution.

Imported historical results retain their original evidence label and conditions. They are not
newly Tested on the receiving host. Unsupported providers or omitted data can permit inspection
while refusing execution; do not create an automatic fallback solver or silent live repair.

## Browser and synchronization alternatives

Browser participation could mean a read-only logical-pack viewer, an offline source editor with
later native admission, or browser-native execution. These are different support claims. Assess
the smallest concrete useful user operation first. The native numerical/provider system does not
become browser-compatible because a storage crate compiles to Wasm.

The supplied [KV overview](../libraries/SurrealKV.md) describes browser primitives, but exact
[KV1.0 Tree](https://github.com/surrealdb/surrealkv/blob/4338a790c8edc56ef5ea1e46157427af4815be3c/src/lsm.rs)
and [library surface](https://github.com/surrealdb/surrealkv/blob/4338a790c8edc56ef5ea1e46157427af4815be3c/src/lib.rs)
do not wire a complete OPFS-backed Tree into the released stable interface. Do not turn primitive
or doc-hidden storage coverage into a released ready-to-reopen browser database claim. MX Wasm
memory support is not by itself durable browser persistence. If later pinned source changes this,
verify actual worker close/reopen, exclusion, quota/error and supported-browser behavior before
promising it. Logical packs can support useful browser inspection without either native engine.

For the present sole operator, explicit export/import and objectwise conflict resolution may
provide all useful exchange. Live collaboration adds identity/permissions, causal delivery,
conflict policy, deletion/retention and offline divergence obligations. MX SSI or KV SI is local
transaction isolation, not distributed replication or CRDT conflict resolution. No last-write-wins,
automatic cross-engine synchronization or "copy engine files" sharing is selected by default.

## Work packages

<a id="DL01"></a>
### DL01 — Logical project export

Consume LC01's closure/protection/copier contract and available PJ01/SC01 research references.
Declare the versioned logical manifest through existing contract owners, expose selected export
via Rust/Python/CLI and include exact membership/completeness and original evidence qualifiers.
Do not duplicate source/result traversal already owned by LC01. Choose only requested payloads,
with actual physical copied extent transparent when it exceeds selected analytic rows.

Acceptance: export one project branch with source/physical context, selected data/calibration,
parameter ancestry, study definition and chosen results; validate independently with the source
service unavailable. Required omissions and unknown codecs cannot pass as complete. An interrupted
copy cannot publish a complete manifest. No credentials, live grants or executable native handles
are carried as authority. Delete replaced ad hoc archive assembly in migrated consumers.

<a id="DL02"></a>
### DL02 — Fresh-authority import and local use

Consume DL01, ST02's exact effects and PJ01/PJ04's source-owner operations. Validate/copy/admit/
activate under a new project/source problem, preserving scientific origin references and current
placement separately. Expose independent inspection and an explicit request to admit/edit/study.
Report unsupported provider or incomplete executable closure without refusing all descriptive
inspection. Integrate ordinary Python modeling/study consumers rather than an import-only engine.

Acceptance: import on fresh disposable authority, edit the new branch and run a supported local
or deliberately recorded study. Original project/run identities are not overwritten and imported
results are not newly qualified. Retry an uncertain activation without duplication. Origin labels
that resemble local run IDs and imported results offered as unauthorized seeds are handled without
creating local execution authority. Wrong-digest, foreign-live-path and unsupported-provider cases
fail at the appropriate boundary. No legacy
data migration or compatibility storage path is introduced.

<a id="DL03"></a>
### DL03 — Browser/offline participation decision

Resolve I06 for a concrete inspection/edit operation, using supplied primary links and exact
selected release sources. Compare logical-pack viewer/editor with browser-native storage. Source
inspection can settle unsupported wiring; use a disposable browser worker probe only for remaining
consequential persistence uncertainty. Do not build an arbitrary frontend before selecting a useful
supported interaction, and do not expand this into numerical browser porting by implication.

Complete with a no-adopt decision naming the native/offline logical alternative and observable
revisit trigger, or an explicitly selected supported slice plus actual integration/acceptance.
Positive persistence claims require close/reopen, exclusion, quota/failure and format coverage on
the selected platform. New platform/rule guarantees require the existing decision route first.

<a id="DL04"></a>
### DL04 — Collaboration and synchronization decision

Resolve I08 against current single-operator exchange and any concrete multi-writer need. Evaluate
explicit logical import plus PJ04 merge before adding a live sync service. Describe identity,
permission, exact-parent conflict, offline divergence, deletion and provenance semantics for any
positive target; do not mistake gRPC live query notifications for a durable synchronization log.

Complete with no-adopt plus supported manual exchange and a concrete multi-writer trigger, or an
explicitly selected implementation with independent conflict/authorization/recovery acceptance.
Neither a speculative CRDT abstraction nor automatic replication is a base-series obligation.
If a new authority rule is required, obtain its explicit selection before dependent implementation.

## Verification and evolution

**Proposed:** targeted manifest/admission/CLI tests use independent source facts and disposable
authorities; 51g owns the assembled export/import journey and scope-end qualification. Native
engine checkpoint portability and new numerical performance are not established by these tests.

A later graphical viewer can consume the logical format; a standalone-editable desktop requirement
can deliberately reopen 51a's embedded topology alternative. Neither needs to alter scientific
identity, infer accepted historical results or retrofit a generic backend facade now. Current
checkpoint: plan authored; no export/import, browser or collaboration implementation claimed.
