---
title: Workspace content lifecycle — current assessment and proposed correction
date: 2026-10-09
tier: design
purpose: target
standard: Core 3.4
decision: revise
---

# Workspace content lifecycle

The current workspace preserves important scientific evidence, authored inputs and
concurrent work, but does not consistently distinguish current guidance, historical
reasoning, generated delivery, disposable scratch and protected state. The verdict is
**Revise** for the workspace content-management design. The correction below is
**Proposed**, without implementation or qualification claims.

Keep the existing architecture/ADR/plan/review responsibilities and resource owners.
Add a small, shared document interpretation contract, explicit publication selection,
and producer-owned disposal. This makes lifecycle actions programmatic without turning
document status, age, size or an agent's search results into deletion authority.

## 1. Scope, drivers and coverage

The functional target is a workspace that helps people and agents understand, change,
operate and qualify the system while limiting unnecessary reading, copies and retained
outputs. It must preserve current functionality, reproducibility, scientific evidence,
historical decision meaning and concurrent uncommitted work. Relevant variation includes
new document families, evolving qualification handoffs, library upgrades, large evidence
bundles, publication-tool replacement and interrupted or concurrent runs.

This is a design-tier, target-purpose review against Core 3.4 and its efficiency
heuristics, selected by [standard.toml](../design_principles/standard.toml), with the
[repository binding](../design_principles/binding/pse-arrow.md). Scientific solver and
physical-model gates are not applicable: their source and behavior were excluded, and
no changes to them are proposed. Root integration combines bounded document mapping,
storage-ownership mapping and an independent design review. The principal reviewer
assessed the current design independently; the root owns the combined evidence and
recommendations.

The checkout baseline was `46545b2ad3e2999b4335692ac49af6091438c3fc`, with substantial
concurrent tracked and untracked changes. The main metadata snapshot is
`2026-10-09T22:02:52.485901+00:00`; external observations span approximately
17:59–18:07:53 America/New_York. Measurements describe a changing checkout, not a
quiescent storage experiment. No cleanup, regeneration, service change, configuration
change or product implementation was performed for this review.

The agreed exclusions are production/test source directories and all `.venv*`
environments. Their aggregate filesystem metadata was measured without assessing their
contents. Third-party source bodies, persistent database contents, private local overrides,
raw personal transcripts and archive payloads were not semantically inspected. Relevant
support tooling and configuration were read to establish generation, loading and ownership.
A broad metadata lookup incidentally returned the first 21 header/comment lines of one
external IDAES file; the search was stopped and those lines were not used or copied.

The bounded audit package is in ignored `build/workspace-review-2026-10-09/`:

| Output | Purpose and limits |
|---|---|
| `inventory.json` | 847 file/link records and 211 grouped trees; paths, tracking, sizes, modification metadata and actual publication projection; no symlink traversal |
| `dispositions.json` | One proposed disposition and reason for every one of the 1,058 records; all have `deletion_authorized: false` |
| `observations.json` | Rounded external-store observations, uncertainty, scope incident and documentation sources |
| `build-storage.txt` | Existing storage observer output; corroboration for stores it already owns |

There were zero inventory scan errors. Coverage is comprehensive at the file/cohort
inventory level, with focused semantic inspection of documents and decisive tooling.
It is not a claim that every support script was audited or every possible source-code
consumer was searched. Excluded production/test consumers remain a retirement blocker
where relevant. The audit package itself is below 1 MiB and has no standing policy role.

No implementation plan owns follow-up yet. The proposed owner is a future workspace
content-lifecycle plan under `docs/plans/`, after the maintainer selects the correction
and any rule impacts. This review supplies observations and proposals, not a second
live disposition ledger or authorization to resume existing plans.

## 2. Current content and storage

### Document coverage and proposed dispositions

The authored document census found 286 Markdown files under `docs/`, excluding generated
reference and the built site, totaling about 7.28 MB of apparent file bytes. These small
files matter primarily for relevance and discovery. The large storage consumers are
build outputs, caches, shared skill evidence and persistent state.

| Cohort | Observed role and proposed treatment |
|---|---|
| Architecture: 15 Markdown files including the index | Retain current contracts and stable section identities; amend through their existing design route |
| ADRs: 96 numbered records plus two indexes/register files | Retain rationale; 50 accepted, 34 proposed, 12 superseded does not imply 12 disposable records; accepted records remain immutable |
| Plans: 38 bodies plus index | 20 in progress, 11 done, 7 draft; reconcile retained obligations by owner rather than infer relevance from status |
| Design-review collection: 99 Markdown files | 50 principal reviews, 42 evidence pages, seven standards/index/principle pages; retain while findings, decisions or evidence dependencies consume them |
| Capability maps: 16 Markdown files | Retain the four canonical pinned evidence maps and reproducibility inputs; investigate additional essays individually |
| Developer guides: 12; root documentation: five | Retain useful instructions, correct stale public summaries and narrow entrypoint reading routes |
| Generated reference | Retain existing generator ownership and consumers; do not hand-edit or remove because it is generated |
| `docs/graph_hashing_reference/` | Current graph/hash review and Plan 28k inputs; retain the two Markdown files and archive while consumed; archive payload uninspected |
| Runtime instructions, process skills, roles and configuration | Retain their functions; reduce repeated detailed guidance and make lifecycle duties explicit at the appropriate owner |
| Support tooling, locks, authored patches and optional manual configuration | Preserve reproducibility and active wiring; an old filename or optional hook configuration is not obsolescence evidence |
| Ignored output/log cohorts | Ownership, consumers and regeneration eligibility mostly unresolved; do not infer disposal from being ignored |

The machine inventory proposes 705 retain, 10 consolidate, 12 retire, two relocate,
one regenerate-on-demand, 316 unresolved and 12 excluded records. These are classifications,
not approved actions. Broad retain classifications for support tooling preserve bytes
pending a specific replacement; they do not certify every implementation as necessary.

The following 12 documents are concrete retirement candidates. All still require a
reviewed reference migration, preserved enduring meaning and resolution of any excluded
consumer before removal. Uncommitted material must first have a durable archive if its
historical meaning is required; Git cannot recover bytes it never recorded.

| Candidates | Basis and remaining obligation |
|---|---|
| [Plan 19](../../plans/19-current-documentation-consolidation.md) | Completed consolidation; reconcile ADR-0096 and review references before retiring the migration narrative |
| [20 capability target](../../plans/20-idaes-capability-target.md), [coverage map](../../plans/20-idaes-coverage-map.md), [modeling](../../plans/20-target-modeling-and-flowsheets.md), [numerical strategies](../../plans/20-target-numerical-strategies.md), [thermodynamics](../../plans/20-target-thermodynamics.md) | Draft target background superseded by later decomposition; preserve still-consumed historical rationale and repair references |
| [21 knowledge placement](../../plans/21-knowledge-placement.md), [kernel architecture](../../plans/21-modeling-kernel-architecture.md) | Their stated retention condition ends when Plan 23 supersedes them; Plan 23 is done, but three/five authored-document references respectively still require reconciliation |
| [Plan 23](../../plans/23-thermodynamic-domain-and-campaign.md) | Completed qualification summarized at blueprint §24.2; 18 authored-document inbound links and scenario/decision dependencies prevent immediate removal |
| [25b applicability review](design_review_plan25b-applicability-implementation_2026-10-01.md), [contracts review](design_review_plan25b-contracts_2026-09-30.md), [selection review](design_review_plan25b-selection-implementation_2026-10-01.md) | Completed observations with no authored-document inbound references found; absence of excluded tooling/code consumers is not established |

Done plans are not uniformly candidates. Retain Plan 22's decision/scenario material,
Plan 26's consumed testing rationale, Plans 30/30a–30d's qualification handoffs, and
Plan 31's uncommitted handoff and highest-numbered identity. Plan 29 requires a separate
30/31 handoff check. Preserve all active Plan 25/27/28 obligations, including transferred
work whose source status may lag its current owner. Status reconciliation is separate
from authorization to execute that work.

Two old PostgreSQL conversational notes in `docs/` are candidates for relocation to
Reference/History discovery before any retirement. Accepted ADR-0112/0114 and Plan 22
still cite them. `datafusion_and_deltalake_tracing.md` and `struct_schema_review.md`
need consumer/replacement investigation; lack of authored-doc inbound links is insufficient.
`math_libraries.md` has a live architecture consumer and should remain available.

Preserve `tooling/dfarrow-apiex`: it remains part of evidence regeneration. Preserve
the root and extraction-tool toolchain pins. Preserve `vendor/patches.md`, the SurrealDB
patch description and authored patch inputs: the selected vendored tree contains local
corrections and cannot be reconstructed from an upstream release alone. Runtime skill
aliases are symlinks, not independent duplicate evidence stores.

The loose `thermodynamics_complete_conversation_handoff/.../expanded/09_validation/results/`
tree contains seven ignored logs, 9,224 apparent bytes/about 48 KiB allocated. A default
search hid them; it is not empty. Their contents and provenance remain unresolved.

### Storage observations

**Measured:** the snapshot reported 129,266,429,952 available bytes, about 120.4 GiB.
Earlier planning observed roughly 27 GiB available; other work changed the filesystem
during the review. Neither change is attributed to this review. Active compiler/cache
and SurrealDB processes were present. No reclaimable-byte total was established.

| Store or cohort | Observed allocation | Ownership and action boundary |
|---|---:|---|
| Checkout `target/` | 26.9 GiB | Active checkout output; preserve during concurrent work; aggregate only |
| Checkout `build/` cohorts | 18.7 GiB | Producer/evidence ownership must determine eligibility; not a general cache |
| `build/plan11`, `build/assessment`, `build/plan10` | 6.4, 4.8, 2.0 GiB | Largest investigation priorities, not deletion grants |
| `docs/book/` | 53.2 MiB | Rebuildable site; preserve last successful publication until replacement |
| `tools/lu-resolve/target/` | 329.8 MiB | Derived tool output; active-use/regeneration conditions unresolved |
| Repo-namespaced sccache | About 66 GiB | Tool-managed cache, potentially used by several checkouts; configured `100G` budget |
| Repo native cache | About 3.2 GiB | Existing generation/identity/borrower-scoped owner; includes pinned source downloads |
| `~/.local/state/pse-arrow` | About 25 GiB | Persistent state, including functional/reference generations; semantic owner only, live mapping unresolved |
| Selected SurrealDB binary store | About 333 MiB | Required pinned 3.3.0 tool; reproducible with network/checksum, not independently disposable while consumed |
| Eleven selected shared skill targets | About 22 GiB | Shared across repositories; selection does not establish exclusive ownership or last use |
| Shared uv cache | About 31 GiB | Per-user cache; attribution to this repository unresolved |
| Shared Cargo registry / Git caches | About 5.2 / 3.2 GiB | Per-user stores; not wholly attributable to this checkout |
| Required root/extraction toolchains | About 969 / 599 MiB | Both have live pin consumers; preserve |
| Checkout `.git/` | About 499 MiB | No Git garbage established; no history rewrite or broad garbage collection recommendation |

Values are allocated bytes unless stated otherwise; external values are rounded.
Rows overlap, including build subcohorts and possible hardlinks across stores. They must
not be added as a savings estimate. Two registered worktrees had no target/build trees;
their source ownership and dirtiness were not inspected, so removal is not proposed.
Container backing storage, undeclared upstream GMP/MPFR caches and current personal
agent-log storage remain unmeasured/unattributed. Historical transcript-size observations
from another review are not current measurements.

## 3. Consequential distinctions and responsibility boundaries

A useful document model separates five questions:

1. **Role:** Is this a contract, instruction, decision, plan, review, reference, evidence
   or index? These roles carry different authority and update rules.
2. **Family state:** What does the existing plan status, ADR status, review decision or
   blueprint state mean? There is no universal `done` state with universal consequences.
3. **Provenance:** Was this authored, generated from declarations, or observed during a
   run? What source/producer and evidence conditions make it reproducible or meaningful?
4. **Discovery:** Should this appear as Current, Reference or History, and for which task
   routes/topics? Being retained does not mean being a default reading input.
5. **Retention:** Who can release it, which dependencies protect it, and which owning
   mechanism may dispose of it? Being reproducible does not make disposal timely or safe.

Existing owners already solve important parts of this problem. Architecture sections own
contracts; ADRs rationale; plans current dispositions; reviews observations. The fixture
resource owner models references, borrows, release and drain. Native cache collection
protects selected generations and borrowers. Validation receipts preserve origin reports
and artifact references. The correction must compose with these owners.

| Responsibility | Proposed owner and consumed contract | Local verification boundary |
|---|---|---|
| Interpret document metadata | One shared typed loader over existing family metadata, collection defaults and sparse exceptions | Small document fixtures; no solver/database startup |
| Select reading/publication | Existing `docs/site.toml` and publisher consume the interpreted view; task routes remain explicit | Discovery/navigation/search and selected-asset fixtures |
| Inventory storage/content | Read-only observer reports identity, provenance, reference gaps and ownership without deciding disposal | Temporary trees; unknowns remain explicit |
| Plan document retirement | Owning plan proposes exact files, enduring-meaning transfers and reference changes | Reviewable Git diff and consumer/reference checks |
| Dispose of derived output | Actual producer/resource owner revalidates release, identity, active use and references | Producer lifecycle/borrow/drain tests |
| Retain scientific/service state | Existing semantic state owner | Existing state lifecycle qualification, outside this review's product scope |

The shared read model is not a new lifecycle service or universal artifact registry.
Publication, retention and scientific state remain separate responsibilities. A document
loader may report an evidence bundle's owner; it cannot independently declare its state
released. Conversely, a resource collector cannot infer document relevance from mtime.

## 4. Findings

### <a id="f01"></a>WCL-F01 — Lifecycle meaning is not consistently executable

**Interface-checked:** `scripts/docs.py::metadata` extracts title/status, while
`scripts/adr.py::split_front_matter` provides a limited flat parser. Document families
contain useful existing metadata, but review ownership/provenance/retirement conditions
are uneven: only 30 of 99 design-review Markdown files have any front matter, and only
four of the 50 principal reviews have a status field. Review decisions are often prose;
`disposition_owner` and `disposition-owner` also vary. Lack of a review status is not
itself a defect; absence of a consistent interpretation contract is.

For S02/S03/S07, automation must reconstruct family meaning and owner dependencies in
multiple consumers or fall back to filenames/status/age. That prevents reliable local
retirement planning and encourages policy drift. This violates AP-04/AP-05/AP-06 and
leaves G1/G3 unresolved for generalized lifecycle action.

**Proposed correction:** one typed interpretation of existing states plus role,
provenance, canonical owner, topics and retention conditions. Collection/bundle defaults
avoid repetitive headers. Normalize legacy aliases on read and reject conflicting owners;
do not introduce two independently writable definitions of one responsibility.
Verification must demonstrate that a done plan with a live handoff remains protected and
that immutable ADRs/native skill formats need no casual rewrite.

### <a id="f02"></a>WCL-F02 — Publication scope misrepresents current relevance

**Interface-checked:** `docs/site.toml` has an empty `current_work` selection. All 38
plan bodies are in History, including active work; only the plan index is Current.
The two historical PostgreSQL conversational notes are Current through the root collection.
The actual discovered projection contains 295 pages: 89 Current, 104 Reference, 102 History.

For S01/S02, Current-only discovery can omit the owning implementation plan while
presenting superseded advice as current guidance. Links from the index partially mitigate
this, but do not make the search classification faithful. This violates AP-04/AP-05 and
G2. Correcting classification does not authorize work on every in-progress plan.

**Proposed correction:** use the existing explicit current-work binding and scoped
overrides, with an owning-plan reconciliation when obligations move. Keep History
searchable on demand. Test the real projection, including done-but-retained handoff pages;
do not infer Current from status alone or create another manually maintained page list.

### <a id="f03"></a>WCL-F03 — Evidence retention implicitly expands site delivery

**Interface-checked:** `scripts/docs.py::stage` copies non-Markdown assets beneath declared
collections unless excluded. This includes two native-producer JSON files of about
8.11 MB and 6.21 MB. Retaining these Plan 28 evidence inputs therefore also places their
bytes into normal site staging, without a separate asset-delivery decision.

For S04/S06, adding a large retained investigation bundle silently increases copies and
publication work. Retention location governs a different responsibility's physical
delivery. This violates AP-01/AP-07 and G6; no measured latency claim is needed to
establish the unnecessary coupling. The files themselves are not established deletion
candidates.

**Proposed correction:** explicit asset selection, with bundle-level download exceptions
and preservation of referenced images/styles/attachments. Retain original evidence under
its owner while independently choosing delivery. Failed publication must preserve the
last successful site; successful replacement must remove obsolete staged delivery.
Tests must cover both selected assets and retained-but-unpublished assets.

### <a id="f04"></a>WCL-F04 — Living entrypoints repeat stale system claims

**Interface-checked:** root README material still describes parity against 2.12, routine
PR gates, confirm-guarded mutations and old name-preservation assumptions. Current
instructions describe 2.13, manual CI, no end-of-turn hooks and local scientific vocabulary.
`CONTRIBUTING.md` says agents run formatting/generators automatically when they stop.
`CITATION.cff` repeats old parity/DataFusion/Pyomo-backend descriptions; the bug template
still offers a Pyomo-adapter category. These are concrete stale summaries, not an argument
to remove contributor information.

For S07, a maintainer changing parity, workflow or architecture must remember several
secondary prose copies. Readers can act on unsupported capabilities and obsolete
workflow instructions. This violates AP-04/AP-06 and G7.

**Proposed correction:** correct these specific facts, retain concise entrypoint purposes,
and route detailed claims to their owners. Generate or mechanically check a small set of
facts only where they already have a canonical declaration; narrative architectural claims
still require judgment. Document review triggers should name changes in the owning
contract, not add generic periodic expiry. Verification compares affected entrypoints
with current owners and exercises only genuine mechanical checks introduced later.

### <a id="f05"></a>WCL-F05 — Legacy cohorts lack enough ownership evidence for safe retirement

**Interface-checked:** the existing storage observer covers target/build/native/sccache,
but not the complete repo-linked/shared/state landscape. Many historical build cohorts
have no established owner, release outcome or complete artifact-reference inventory in
the evidence inspected here. Loose handoff logs and auxiliary research have similar gaps.
This is not a finding against the exact fixture/native owners that already model release.

For S03/S05/S06, a broad age/size cleanup can destroy origin reports, active state or
another checkout's cache. Conversely, permanently retaining unknowns accumulates storage.
The generalized operation remains unresolved under AP-02/G3/G5; the current manual
cross-owner reconstruction violates AP-05/AP-06 for recurring lifecycle management.

**Proposed correction:** extend observation and assign producer/bundle ownership before
enabling disposal. Existing owners provide the deletion decision wherever possible;
legacy unknowns require explicit adoption/release or remain protected. A complete named
cohort removal must revalidate live identity, references, borrowers and outcome at action
time. An inventory snapshot or Git ignore rule is never sufficient authority.

F01 supplies common interpretation for F02 and document retirement, but F02's incorrect
selection can be corrected immediately through existing configuration. F03 is an
independent publication boundary defect; metadata alone will not stop implicit copying.
F04 needs owner-linking and factual repair even if lifecycle automation is deferred.
F05 requires actual producer/resource evidence beyond any document schema.

## 5. Proposed document structure and storage

Preserve the existing physical separation of current architecture, rationale, active
plans, reviews, evidence, reference and generated reference. Do not move every file or
create a permanent `archive/` tree. Git history remains the archive for retired authored
material. Move only where location currently causes wrong discovery or obscures ownership.

**Proposed:** a small `docs/lifecycle.toml` owns the lifecycle vocabulary, schema version,
collection/bundle defaults and sparse exceptions. `docs/site.toml` continues to own
publication collections, scope and explicit current-work selection. The shared loader
combines them with native document metadata; consumers do not reimplement precedence.
Keep discovery facts out of retention policy, and existing family statuses out of a new
parallel status registry.

The following is illustrative new review metadata, not an edit to an existing review:

```yaml
title: Preparation acceptance review
decision: revise
doc_role: review
doc_owner: docs/plans/28-surrealdb-unified-substrate.md
doc_provenance: authored
doc_topics: [preparation, validation]
doc_retention: while-dependent
doc_retirement_trigger: owner-release
doc_retention_reason: Open dispositions and the qualification handoff consume this review.
```

| Field | Intended interpretation |
|---|---|
| Existing `status`, `decision`, `pins`, `regenerated`, evidence fields | Preserve family-specific meaning and original evidence conditions; never coerce into one universal state |
| `doc_role` | Controlled role: contract, instruction, decision, plan, review, reference, evidence, index |
| `doc_owner` | Canonical document/disposition owner, resolved from existing owner fields where present |
| `doc_provenance` | Authored, generated or observed; generated/observed bundles also resolve their source/producer through existing provenance or a bundle declaration |
| `doc_topics` | Small controlled task vocabulary used for discovery; not free-form labels for every sentence |
| `doc_retention` | While-current, while-dependent, owner-managed or protected-unknown; classification is not action permission |
| `doc_retirement_trigger` | Controlled event such as owner-release, replacement-adopted or producer-policy; not an executable predicate embedded in prose |
| `doc_retention_reason` | Human explanation for an exception or dependency; does not independently release anything |

Retention ownership defaults to the canonical document/disposition owner. An override
is justified only for a distinct resource manager and should name that manager, not copy
the same path. Normalize existing `disposition_owner` variants into one read model;
do not separately author `doc_owner` and `disposition_owner` for the same responsibility.
Changing ownership is an explicit reconciliation, not a last-key-wins parser accident.

Use collection defaults for routine architecture/ADR/reference roles, and one bundle
declaration for observed evidence with common producer/conditions. Add per-file overrides
only when semantics differ. Accepted ADRs remain immutable: external defaults/exceptions
classify them without rewriting their historical front matter. Generated headers come
from generators. Shared skills keep their runtime-native metadata; repo selection and
repo-specific wiring stay local, while shared library content remains repo-agnostic.

The loader must reject duplicate keys, unknown controlled values and conflicting owners;
distinguish absent, inherited and explicit values; normalize legacy keys deliberately;
and identify the schema version. The current flat parser must not silently pretend to
validate a richer YAML contract. Reuse an appropriate maintained parser and a small typed
validation layer rather than invent a YAML grammar. Parser choice, exact pin and API
qualification remain implementation work; no new dependency was selected in this review.

For new producer outputs, prefer one attempt/cohort boundary such as
`build/<producer>/<attempt>/` with the existing run identity and evidence/resource owner.
Compact manifests and retained reports may have a longer lifetime than scratch inputs.
Do not rename legacy paths merely to fit this layout, break receipt origin references,
or duplicate authoritative run state into a documentation registry. Existing producer
layouts can be adopted with bounded declarations until a concrete migration is warranted.

Private overrides remain untracked and unpublished. Downloaded source, compiled scratch,
generated reference, pinned reproduction inputs and authored patches remain distinguishable.
A shared skill symlink does not transfer the shared target's lifecycle to this repository.

## 6. Proposed lifecycle operations and retention defaults

There are four useful operations, which may extend existing commands rather than become
four new tools:

1. **Inventory:** read-only, bounded metadata and ownership observation, with explicit
   exclusions, unknowns and snapshot conditions. No broad transcript/body ingestion.
2. **Check:** validate document interpretation, references and publication selection;
   report unsupported or ambiguous lifecycle states without deleting anything.
3. **Retirement plan:** produce a concrete reviewable diff for authored material and a
   named resource-release proposal, including surviving meaning and references.
4. **Prune:** invoke the actual producer/tool owner for a bounded eligible set, with
   revalidation at action time. Do not create a universal filesystem sweeper.

| Content | Proposed default boundary | Automatic action boundary |
|---|---|---|
| Standing current contracts, instructions and retained decisions | No TTL; owner changes/replacement trigger review | None from age/status |
| Plans, reviews and supporting evidence | Owner release after enduring meaning and references are reconciled; done alone is insufficient | Retirement diff may be generated; authored removal remains explicit |
| Proven disposable successful-run scratch | Producer finalization after required outputs/provenance are committed and no borrower/reference needs scratch | Bounded automatic producer cleanup |
| Failed/interrupted runs or unknown legacy outputs | Protected until owner disposition and reference/borrower reconciliation; no generic expiry | None while unknown or required for diagnosis |
| Temporary compiler campaign output | Existing build-performance policy permits at most one temporary campaign target per checkout; this concerns campaign targets, not agent concurrency | Explicit owner reclamation after the campaign; do not alter active checkout targets |
| Compiler/download caches | Preserve existing tool-managed `100G` sccache setting initially; resize only with measured regeneration/use needs | Tool-owned eviction within its existing contract |
| Free-space operation | Preserve current 50 GiB assessment floor initially; observe before large campaigns | Existing refusal/maintenance behavior, no guessed optimal threshold |
| Native generations and scientific/fixture state | Existing exact identity, selected generation, pins, references, borrowers and drain owners | Only their existing qualified release/reclaim paths |
| Published site | Replacement succeeds before old delivery retires | Existing atomic/recoverable publication behavior |
| This review's raw audit package | Owner confirms preserved measurement/classification meaning and necessary provenance, durable handoff/archive and no continuing snapshot consumer | Disposable only after that release; publication alone is insufficient |

The temporary-campaign and floor defaults are grounded in
[build-performance storage policy](../../dev/build-performance.md) and
`.config/build.toml`; they do not introduce a new concurrency restriction or claim an
optimal budget. Failed runs can later receive an explicit bounded diagnostic-retention
policy, but unknown provenance cannot be repaired by assigning a short TTL.

A deletion plan must identify exact artifacts and the owner that can release them.
The action rechecks live identity, selection, borrowers, references and changed metadata;
new symlinks or replacement paths cannot redirect deletion. Compact receipts do not
automatically replace the original artifacts their digests/reference fields validate.
If the owner cannot prove eligibility, report protected/unknown and stop that action.

Run bounded cleanup at producer finalization or explicitly invoked maintenance. Do not
restore end-of-turn hooks, run whole-repo inventory every turn, broadly clean Cargo,
remove environments, prune shared skill targets, stop databases or sweep state by mtime.

## 7. Instructions, skills and agent discovery

The root instruction layer should retain the decisions needed before any task:
authority routes, preservation of dirty work, generated/off-limits boundaries, command
entrypoints and scope-end rhythm. Detailed environment troubleshooting belongs in the
environment guide; lifecycle mechanics in a focused workspace guide; role procedures
in their existing owners. Root instructions link those owners. `CLAUDE.md` remains a
runtime adapter/import, not another shared policy copy.

This is runtime-sensitive. Official Codex documentation describes instruction discovery
along the startup root-to-working-directory chain, with a configurable combined byte
budget. A nested instruction file is not guaranteed to load merely because an agent
later reads a descendant source file. Linked Markdown is not an automatic import.
Keep explicit Codex routes for conditional rules and critical preservation obligations.
See [Codex AGENTS.md guidance](https://learn.chatgpt.com/docs/agent-configuration/agents-md).

Claude loads its memory/imports and unconditional rules differently, while scoped rules
and skills provide conditional guidance. Skill descriptions can be discovered without
loading every complete skill body. Validate each adapter's actual behavior instead of
assuming that deleting a root paragraph makes a nested file available in both runtimes.
See [Claude feature/loading overview](https://code.claude.com/docs/en/features-overview)
and [Codex skills documentation](https://learn.chatgpt.com/docs/build-skills).

The observed repo `AGENTS.md` is 22,333 bytes, with a 3,574-byte global instruction file;
the local Codex configuration allows 64 KiB. These observations do not establish current
truncation. The roughly 19.9 KB plan index is an on-demand route, not inherently startup
context. Most retained docs and skill evidence are not automatically loaded. Storage
size, published page count, loaded bytes and actual context/token effects are distinct
measurements; no latency, quality or token-saving result was established.

**Proposed skill changes:** plan creation declares the canonical disposition owner and
required retention/review triggers; execution closure transfers enduring meaning,
reconciles references and requests release from evidence/resource owners; design review
declares its scope, decision and dependent evidence bundle. These extend existing skills
rather than add a compulsory housekeeping skill to every task. Shared library skills
retain reproducibility inputs and repo-agnostic bodies; local capability maps keep their
distinct repository wiring and evidence role until a specific replacement is demonstrated.

Use Current/Reference/History and topic/role filters through the existing discovery
surface. Give agents a small task route and the owning document, then supporting evidence
on demand. Avoid broad ignore rules that make required evidence disappear from deliberate
searches. A default search selection may be bounded, but an explicit all-retained query
must remain available and must not be mistaken for deletion eligibility.

## 8. Revealing scenarios and alternatives

| Scenario | Required behavior and boundary | Evidence or acceptance gap |
|---|---|---|
| S01 — Start work on an active plan | Explicit selection exposes the current owner; History remains available; selection does not authorize execution | Current empty binding is a demonstrated gap; corrected real projection required |
| S02 — Complete a plan with a live qualification handoff | Preserve review/evidence while the consuming owner still needs it; transfer current meaning before retirement | Done-plan exceptions demonstrate why status cannot drive deletion |
| S03 — Retire a completed review/evidence bundle | Resolve findings, historical citations, excluded consumers and origin-report references before exact removal | Full candidate consumer closure not established |
| S04 — Replace site tooling or rebuild publication | Document meaning/retention unchanged; explicit delivery selection; failed replacement keeps previous successful site | Existing recovery is a preservation constraint; new asset tests required |
| S05 — A borrower starts or a path changes after inventory | Actual owner revalidates identity/borrowers/references; stale snapshot cannot delete the replacement | Existing fixture/native lifecycle is the composition to preserve |
| S06 — Add a large investigation or interrupt a campaign | One bounded producer cohort; durable result/provenance separated from scratch; failure remains protected; site delivery stays explicit | Current implicit asset copy is a demonstrated boundary defect |
| S07 — Change parity, workflow or document family | Canonical owner changes; concise entrypoints reconcile; new family maps into typed interpretation without rewriting every consumer | Stale public summaries are demonstrated; parser/model extension remains Proposed |

The simplest viable first step is to correct stale entrypoints and explicit publication
scope using current mechanisms. It improves immediate relevance but cannot safely automate
cross-family retirement. The recommended incremental target adds minimal typed interpretation
and producer-owned lifecycle operations; it carries a small schema/migration cost to remove
repeated policy reconstruction.

A universal content database/service would add synchronization, availability and ownership
problems without a demonstrated need. An age-based sweeper would erase meaningful distinctions.
A permanent archive tree would retain duplicate search/storage burden and compete with Git
history. Blanket removal of generated docs or shared skills would discard current functionality
and reproducibility without a proven replacement.

Use maintained YAML/TOML/filesystem facilities behind the document interpretation owner.
Existing publication tooling and lifecycle owners are the library/mechanism alternatives
already composed here. No new generic parser, runtime service or dependency family is justified
by this review alone. The exact parser selection must be qualified before implementing a
richer contract; permissive parsing without duplicate-key rejection is insufficient.

## 9. Architectural assessment and gates

These verdicts assess the **current scoped design**, not the proposed correction.

| Foundation | Verdict | Scenario, evidence and obligation |
|---|---|---|
| AP-01 Separation of concerns | Violated | S04/S06: retained evidence location implicitly controls delivery; separate asset selection (F03) |
| AP-02 Stable contracts | Unresolved | S03/S05: generalized retirement/release contract is not established for legacy cohorts (F05) |
| AP-03 Composition | Satisfied within inspected owners | Existing publisher, fixture/native owners and receipt origins have reusable responsibilities; preserve them rather than introduce a competing registry |
| AP-04 Domain model and semantic authority | Violated | S01/S02/S07: missing common interpretation, wrong discovery scope and stale secondary claims (F01/F02/F04) |
| AP-05 Explicit structure | Violated | S02/S03: ownership/retention conditions require manual reconstruction (F01/F05) |
| AP-06 Local reasoning/testability | Violated | S03/S07: repeated cross-document policy/fact reconstruction rather than one consumed interpretation/owner (F01/F04/F05) |
| AP-07 Execution fits workload | Violated | S06: normal retention expands copies/publication regardless of delivery demand (F03); no quantified runtime benefit claimed |

| Gate | Verdict | Evidence or settling obligation |
|---|---|---|
| G1 Authority | Unresolved | A generalized reconciled owner model is missing; unknowns must remain protected, not silently pick a conflicting field |
| G2 Semantic fidelity | Fail | Current/History selection misrepresents active plans and historical notes (F02) |
| G3 Validity | Unresolved | No generalized action eligibility for legacy cohorts was established (F05) |
| G4 Hidden behavior | Pass for inspected observation paths | Inventory/discovery observe without disposal; no claim about all uninspected tooling |
| G5 Consistency and recovery | Unresolved | Existing site recovery is useful; complete cross-owner legacy release/drain evidence is missing |
| G6 Transformation and reuse | Fail | Retention-to-publication projection implicitly expands asset delivery (F03) |
| G7 Truthful capability claims | Fail | Living entrypoints contradict current system/workflow owners (F04) |
| G8 Library leverage | Pass within inspected scope | Existing owners are reusable; no unexploited generic capability was established; exact richer-parser choice remains open, not a qualified new implementation |
| G9 Architectural fitness | Fail | S01/S03/S06/S07 violate the foundations above; proposed remedies do not make current design pass |
| Scientific/profile gates | Not applicable | Production/test scientific behavior was excluded and no scientific contract changes are proposed |

## 10. Verification and proposed implementation sequence

**Measured:** the bounded inventory and storage observations above were collected against
a live checkout. **Interface-checked:** the decisive publisher, metadata parser, storage
observer, cache/resource lifecycle and receipt contracts were inspected. **Proposed:**
metadata vocabulary, loader, selection corrections, skill changes and lifecycle actions.
No cleanup savings, reduced context tokens, product behavior or corrected implementation
was Tested or Measured by this review.

Executed audit commands included
`scripts/pse-env --resource-class light -- python3 /tmp/pse-workspace-inventory.py`,
the corresponding disposition producer,
`scripts/pse-env --resource-class light -- just build-storage` and targeted
metadata/size/reference lookups. The initial
default resource classification was refused with exit 125 because another exclusive heavy
work owner was present; rerunning the bounded inventory in the explicit light class succeeded.
The ephemeral producers are audit aids, not installed tools or policy implementations.

Product tests, site regeneration, code generation, global formatting and integrated
qualification were not run. They would not validate this read-only diagnosis, and global
mutating bundles would cross the agreed scope and concurrent-work boundary. Artifact-only
checks use
`scripts/pse-env --resource-class light -- python3 /tmp/pse-workspace-review-check.py`
for inventory completeness, classification validity and the new review's local links
and whitespace, against a zero-error baseline: 1,058 unique records, 15 local links,
12 review slots and zero errors. `git diff --check -- docs/design_review/README.md`
also passed. These checks do not certify the proposed system.
The global `just turn-end` bundle was not executed because its ADR-index regeneration
and global formatting would cross the agreed review-only boundary.

| Proposed work package | Priority and prerequisites | Concrete acceptance; all future work |
|---|---|---|
| A — Repair public entrypoints and explicit Current selection | Immediate relevance; no new metadata required; owner reconciles active/handoff selection | Correct the named stale claims; assert real discovery scopes and preserved History access; review focused diff |
| B — Introduce minimal shared interpretation | Foundation for programmatic lifecycle; select RC01 and exact parser first | Fixtures for inherited/default/override fields, native family states, legacy owner aliases, duplicate keys, unknown values, conflicts and immutable ADR exceptions |
| C — Separate asset delivery from retention | Independent high-value correction; select RC02; may use B for bundle interpretation | Referenced images/downloads remain delivered; unselected large evidence stays retained but unstaged; failed build preserves prior site; successful build removes obsolete delivery |
| D — Add producer lifecycle observation and bounded cleanup | Requires actual resource owners and B's document interpretation where consumed | Unknown/failed/live-borrowed/replaced-path artifacts protected; eligible successful scratch removed by exact owner; provenance/origin references remain valid |
| E — Reconcile and retire legacy candidates | After owner/reference investigations; cannot infer eligibility from A–D passing | Review exact candidate diffs, current meaning transfers, immutable historical targets and excluded-consumer evidence; release named build cohorts independently |
| F — Update instructions and process skills | Select RC03; implement alongside B–E rather than duplicate their contracts | Each runtime receives required startup obligations and can reach conditional owners; closure records release conditions; shared library skill bodies remain repo-agnostic |

Consequence priority differs from dependency order: immediate discovery/factual repair need
not wait for the schema, while safe automatic disposal cannot precede producer ownership.
Measure staged bytes before/after C and actual runtime-loaded instruction bytes before/after
F under stated startup conditions. A smaller artifact or shorter file alone does not prove
better agent performance. Use targeted support-tool checks for these changes; product/scientific
qualification becomes necessary only if later implementation reaches that behavior.

## 11. Rule impacts and follow-up ownership

These are proposed changes to current rules, not changes made by this review. The future
owning plan should hold each finding reference, scenario, disposition, work owner and
evidence/revisit trigger. Current observations here remain historical; no parallel status
table is introduced. No exceptions to a MUST are requested.

| Rule impact | Current owner and proposed change | Dependency and result if retained |
|---|---|---|
| <a id="rc01"></a>RC01 — Extend document metadata interpretation | `.claude/rules/docs.md`, process skills and document-family conventions: add controlled role/provenance/owner/retention interpretation with defaults/exceptions; preserve existing native fields and accepted-record immutability | F01/F05; if not selected, retain manual retirement interpretation and limit automation to existing qualified producer owners |
| <a id="rc02"></a>RC02 — Make asset publication explicit | `docs/dev/documentation.md`, `docs/site.toml` and publisher behavior: replace implicit collection-wide non-Markdown copying with explicit selected assets/bundles | F03; if not selected, retained evidence continues to expand delivery and the finding remains open |
| <a id="rc03"></a>RC03 — Narrow root guidance through runtime-aware routes | `AGENTS.md`, `CLAUDE.md`, shared rules and process skills: retain startup obligations, move detailed procedures to focused owners, validate both runtime routes and add lifecycle duties at the owning skills | F04 and context-management recommendation; if not selected, repair facts in place and accept limited context reduction without asserting nested instructions will load |

Using the existing `current_work` binding and correcting false summaries does not require
a new authority model. Git-history retirement, accepted ADR immutability, original evidence
provenance, generated-output guards, semantic state retention and the no-end-of-turn-hook
policy remain preservation constraints. A later governance change follows the existing
decision route where required; publishing this review applies none of these rule impacts.

## 12. Decision

Current behavioral/semantic adequacy is **insufficient** for truthful relevance selection
and generalized lifecycle action. Architectural fitness is **insufficient** because
interpretation, discovery and delivery cross responsibilities unnecessarily. Overall:
**Revise**, within the scope and evidence limits above.

The recommended correction is **Proposed** and unqualified: repair known misleading
entrypoints and selection first; introduce minimal shared interpretation; separate delivery
from retained evidence; then enable only owner-proven disposal. Investigate legacy cohorts
before releasing them. The 12 document candidates and large storage cohorts are useful next
investigation targets, not an approved deletion list. No storage was reclaimed by this review.
