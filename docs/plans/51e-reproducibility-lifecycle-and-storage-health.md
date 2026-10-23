---
title: "Plan 51e: Reproducibility, lifecycle and storage health"
status: complete
date: 2026-10-22
adrs: [ADR-0220]
review_sources: [docs/functional_reviews/surrealkv-surrealmx-functional-extensions-2026-10-22.md]
scenario_sources: [docs/functional_reviews/surrealkv-surrealmx-functional-extensions-2026-10-22.md]
doc_role: plan
doc_owner: docs/plans/51-engineering-workspaces-and-evidence.md
doc_topics: [current-work, design, storage, diagnostics, validation]
---

# Plan 51e: Reproducibility, lifecycle and storage health

## Target and authority

**Implemented:** explain what an exact selected project/study/evidence closure contains, copy it
safely for reproduction/exchange, explain recovery and retention through current owners, and
make storage observations useful without inventing new authority. Resolve whether a complete-
artifact confidentiality capability is needed. O09/O14/O15/O19 and inquiry I07 remain owned by
the [coordinator](51-engineering-workspaces-and-evidence.md).

This work composes existing review packs, reproducer predicates, backup, result protection,
resource registry and disposal operations. It does not move host resource authority into the
database, replace sealed files, add a universal retention engine or make a native checkpoint a
solver restart. New project/dataset/evidence consumers must participate in existing lifecycle
owners when introduced; LC03 is an explanatory/public-command integration, not permission to
postpone correct lifetimes until the end.

## Closure, reproduction and recovery

An exact selected closure records original identities/codecs/digests, selected member rows/ranges,
source/physical/provider requirements, available bytes and intentional omissions. Distinguish
complete executable support, complete selected offline evidence, descriptive-only references and
missing/unsupported closure. A frozen index snapshot or old path does not grant current access.
The source/result/project owner resolves selection and obtains current protection; copying holds
that protection until destination verification and publication. A copied closure is self-contained
and uses its manifest, never ambient live paths.

Use the current reproducer owner's explicit predicate and supported execution path. Opening or
inspecting a pack does not automatically execute science, install a provider or download data.
A request to reproduce is a new operation under current authority and conditions. Preserve original
qualification and identify unsupported providers, incomplete payloads and different conditions;
a successful metadata import does not newly qualify imported numerical results.

Publish manifests only after required copied members validate; interrupted copies remain partial
and cannot be reported as a complete pack. Use existing owned staging/settlement where applicable.
Avoid duplicating full file trees when only exact selected result ranges are requested. If the
selected copier must materialize a whole sealed file, report that actual copied extent and retained
unselected membership without implying it was analytically selected.

Application recovery covers catalogue identity plus the required external payload and source
closure. Native MX persistence/snapshot state is only one component. Keep current service/authority
fences and application recovery operations; explain missing members and unsettled operations before
an explicit recovery action. Never open a live server engine directory with an independent raw
tool or silently repair missing payloads from newer results. Recovery tests use authenticated
disposable state, not operator data deletion.

Native [KV checkpoints](https://github.com/surrealdb/surrealkv/blob/4338a790c8edc56ef5ea1e46157427af4815be3c/src/checkpoint.rs)
flush/copy a consistent engine closure and use completion/restore markers. Restore is not an
atomic application swap. [MX persistence](https://github.com/surrealdb/surrealmx/blob/f67f00cb94f4fb4582090cd86b3b6993f8e25300/src/persistence.rs)
has its own phase and log/snapshot compatibility. Neither substitutes for this application's
selected-file closure or exact uncertain-effect settlement. The unselected KV topology does not
become a second production recovery route merely to exercise its library checkpoint features.
Native checkpoint copying may use hard links to immutable files; a same-device checkpoint is
therefore not automatically an independent off-host backup or a cheap copy-on-write project fork.

## Retention and useful health

An explanation query joins actual ownership: project/source/study references, selected-result
protection, in-flight operations, active borrowers and resource-owner disposal status. Distinguish
logical bytes, copied bytes, physical allocation and unknown/reclaimable amounts. A list of deleted
rows or an engine counter does not prove disk space was reclaimed. Metadata/index retention and
bulk payload retention are independently explicit; importing evidence must not pin all raw data.

Release is an explicit owned effect against selected identities/generations. Recheck protection,
references, in-flight effects and borrowers at the current owner before disposal. A stale explanation
is not a grant to delete. Preserve original operation allowance and exact settlement. Project soft
deletion, draft removal, result release and irreversible artifact deletion remain distinguishable.

Use the current MX statistics/service identity, persistence health, queues, old transactions and
available host/cgroup/storage observations to explain what is happening. Link only real process/
authority identities. Approximate engine memory is not RSS; queue length and pinned versions do
not alone establish a cause. No always-on telemetry platform, routine whole-store GC, native KV
inspector against the canonical server or changed workstation capacity policy is selected.

## Confidentiality question

The bounded decision considers the actual local-use/exchange threat and all artifacts: catalogue,
payloads, project source, measurement inputs, exported packs, diagnostics, traces, crash dumps,
temporary files, key material and backups. Compare existing volume protection and encrypted logical
exchange with native KV TDE. KV-only encryption would leave selected sealed files and other artifacts
outside its scope and would require the unselected backend port; do not present it as complete
project confidentiality.

Exact [KV key management](https://github.com/surrealdb/surrealkv/blob/4338a790c8edc56ef5ea1e46157427af4815be3c/src/tde/keys.rs)
and checkpoint sources show historical-key/encryption-identity requirements. Restoring into an
independently created different encryption database identity differs from copying/opening a closure
with its historical keys. TDE/storage types include doc-hidden interfaces; engine files are not
a stable encrypted interchange format. Root-key custody/recovery and rotation must be part of any
positive selection; keys cannot be casually copied into logs, manifests or diagnostic exports.

## Work packages

<a id="LC01"></a>
### LC01 — Selected closure and reproducer composition

Consume ST04's selection/lifetime contract and available project/research/evidence references.
Extend current inventory/reproducer/review-pack owners to describe and copy selected closure,
with explicit completeness/omissions and supported reproduction predicates. DL01 reuses this
closure resolution rather than implementing a second traversal. Copying/exporting is distinct
from requesting new execution.

Acceptance: copy a selected exact study/result/evidence closure, inspect it independently of the
source service and request a supported reproduction explicitly. Missing required members,
unsupported providers, protection loss and interrupted copy cannot publish a complete pack.
Retain original result qualifiers and do not infer new scientific qualification. Delete replaced
parallel pack traversal logic in migrated consumers.

<a id="LC02"></a>
### LC02 — Explainable application backup and recovery

Extend the existing application backup/recovery command's inspection to new project/research
references and selected evidence artifacts. Describe the required catalogue/payload/source roots,
identity, unresolved operations and missing members. Preserve current engine/service recovery
fences and manifest publication. Native snapshot completion alone cannot establish a complete
application backup.

Acceptance: inspect a complete disposable closure and deliberately incomplete/interrupted one;
only the complete supported closure restores through the existing owner. A changed authority,
missing payload or uncertain operation is explicit. An independent tooling-index rebuild is
not canonical recovery and cannot settle original terminals. No full upstream corruption/DST
campaign is required for this application's closure extension.

<a id="LC03"></a>
### LC03 — Retention explanation and explicit release

Add public explain/select/release integration for project drafts/history, study/results and
selected evidence through existing source/result/resource owners. Show actual blockers and
unknown physical reclamation; use original exact identities and expected generations for release.
New consumers must register real references/protection as they land, not rely on query index rows.

Acceptance: retain an actively borrowed result and a project-referenced revision, show why each
cannot be released, then release only after the owning protection permits it. A stale explanation
and foreign authority cannot dispose current artifacts. Dropping an evidence index does not
release originals; releasing raw detail leaves an honestly incomplete discoverable summary.

<a id="LC04"></a>
### LC04 — Storage health and admitted operator actions

Compose existing engine/service statistics and host diagnostics into a selected health inspection,
with real identity/configuration context, capture time and unsupported/approximate fields explicit.
Expose actionable owner commands for genuine stale operations, protection or persistence issues;
inspection itself performs no repair. EV04 consumes actual diagnostic links and distinguishes
service/tool observations from scientific outcomes.

Acceptance: explain representative queue/pin/persistence and unavailable-service states using
authentic existing observations; unknown correspondence remains unknown. An action requires its
current owner preconditions and retains exact effect settlement. Use current independent host
observation when the database is unavailable. Do not infer memory leaks or reclaimed bytes from
one approximate engine counter.

<a id="LC05"></a>
### LC05 — Complete-artifact confidentiality decision

Resolve I07 with the intended local/exchange user scenario, artifact/key inventory and alternatives
above. A no-adopt outcome names the supported protection boundary and a concrete revisit trigger.
A positive outcome requires explicit selected scope, key custody/recovery, all required artifact
paths and applicable decision work before dependent implementation.

Acceptance for positive adoption includes independent reopen/recovery, missing/wrong-key refusal,
interrupted export/rotation behavior and assurance that required raw artifacts are covered. Do not
add speculative crypto code, create a partial security promise or require an unrelated KV migration
to finish the base workbench. The inquiry is required; adopting TDE/encrypted packs is conditional.

## Verification and handoff

**Implemented:** LC01 shares one exact closure resolver with DL01; original protected result and
artifact readers supply copied membership, and original review-pack owners validate scientific
IPC lineage/finalization. LC02 extends application backup manifest v4 with bounded original
catalogue/reference inventories and shared inspection/restore verification. LC03 composes original
retention/release owners, retained history-settlement receipts and immutable draft retirement.
LC04 supplies bounded native/service/cgroup/host observations and existing evidence associations.
LC05 closes with the maintainer's no-application-crypto decision and full artifact inventory at
the coordinator; filesystem access and explicit custody are the supported boundary.

**Tested (Linux, 2026-10-23, zero failure baseline):** protected lifecycle/source/artifact controls,
public Python lifecycle and descriptive/portable controls, independent IPC checks and the actual
`canonical_recovery_check` numeric journey passed at their recorded conditions. The recovery
journey includes nonempty project/branch/draft custody, complete/incomplete backup inspection,
SIGKILL/reopen, positive 4096/2048 MiB server reconfiguration, exact values/staged payloads,
fresh restore authority, semantic admission and stale credential/creation refusal. Both profiles
admit the unchanged production module pool. This did not select the optional `--scientific`
reference-profile recovery campaign.

Actual health capture on the disposable restored service observed ten native metric families;
its server was stopped afterwards. Capture on the larger primary retained verified service and
cgroup identity while reporting the unchanged 4 MiB metric-response limit precisely. No truncated
prefix became a complete metric population. Host adapters passed 322 tests; affected health
controls passed after the final observation repair. [51g](51g-qualification-and-publication.md)
owns exact commands, completed assembled/scope-end checks and qualification limits.

## Outcome

LC01–LC05 are complete at their selected functional boundary. Enduring meaning lives in
blueprint §20.6/§24.1 and the substrate/diagnostic guides. Whole-series acceptance is recorded
at 51g; this companion does not qualify an unexercised native/provider/platform combination.
**Measured:** no speedup, physical reclaimed bytes or full-artifact confidentiality is claimed.

**Mistakes corrected:** the first recovery fixture fork omitted the original source closure;
the unchanged guard correctly refused it. Actual recovery then exposed unsupported SurrealQL
projection, a premature maintenance RETURN and NULL-versus-NONE transport errors. Repairs preserve
the original inventory equality, nonce, generation and authority guards. Fixture teardown now
waits for authenticated releases before disconnecting; validation uses the original lifecycle
reservation. Health independently retains service observation when bounded metrics fail.

**Deliberate deviations:** no KV topology/checkpoint port, universal retention service, routine
whole-store GC, encrypted-pack/TDE implementation or legacy-data campaign. Numeric functional
recovery and independently asserted scientific exchange cover distinct obligations; one does
not imply the other. Unknown blockers/reclamation/metric correspondence remain explicit.
