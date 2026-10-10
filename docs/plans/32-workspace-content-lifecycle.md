---
title: Workspace content lifecycle
status: in-progress
date: 2026-10-09
adrs: [ADR-0168]
review_sources: [../design_review/reviews/design_review_workspace-content-lifecycle_2026-10-09.md, ../design_review/reviews/design_review_workspace-content-lifecycle-target_2026-10-09.md]
scenario_sources: [../design_review/reviews/design_review_workspace-content-lifecycle_2026-10-09.md#8-revealing-scenarios-and-alternatives]
---

# Plan 32: Workspace content lifecycle

## Context

This plan develops the findings and recommendations of the
[workspace content-lifecycle review](../design_review/reviews/design_review_workspace-content-lifecycle_2026-10-09.md).
It owns WCL-F01–F05 dispositions and the implementation sequence below. The intended
result is a workspace that makes relevant guidance easy to find, distinguishes retained
evidence from published delivery and disposable scratch, and supports safe lifecycle
actions without losing functionality, scientific meaning or concurrent work.

The maintainer authorized plan creation on 2026-10-09, confirmed RC01–RC03, selected
mechanisms plus a qualified initial cleanup, and permitted narrow file/path-reference
searches in production/test trees for retirement safety. This document is **Proposed**;
authoring it implements none of the mechanisms and authorizes no deletion by itself.
Product execution, scientific qualification and the paused Plan 28 campaign keep their
existing owners.

The maintainer subsequently authorized implementation and simplified legacy retirement:
remove all pre-28 plans and companions without content salvage, metadata backfill,
open-item reconciliation or scientific requalification. Repair only references needed
by retained documentation/tooling. No old scope is imported into Plan 28 or this plan.
Plan 28 retains its entire implementation and qualification scope. Plan 32 manages
workspace content and resources; it does not absorb another plan's work.

Multiple plans may remain active and execute concurrently. Each scope item has one
status owner, while content-level discovery reconciles relationships across those
owners and derives an aggregate open-scope view. There is no one-plan-at-a-time rule.

The authoring baseline is `main` at `4c24721e691187e1a5b28398b29722fbde671da8`.
The workspace review is committed; an unrelated untracked efficiency-review evidence
directory is preserved. Recheck the actual tree before each editing or cleanup package.
The review's 1,058-record inventory and storage observations are dated snapshots, not
current eligibility or an additive savings estimate. Its raw package remains local under
`build/workspace-review-2026-10-09/` while this plan consumes it.

Core 3.4, its efficiency heuristics and the
[repository binding](../design_review/design_principles/binding/pse-arrow.md) govern the
focused foundation assessment. Scientific profile gates are outside this tooling/document
scope. No additional product-wide review or qualification is implied.

### Preserve the content agents use

The maintainer explicitly requires preservation of the carefully reviewed substantive
authoring contract: how plans frame a change, explain implementation, assess alternatives
and behavior, select evidence and establish completion. Preserve its adaptable document
organization, prose style, templates, examples and room for implementation judgment.
Do not replace this reasoning with a checklist, mandatory new headings, a packet schema,
an architecture score or automatically generated plan approval.

Metadata, tagging, storage, discovery and lifecycle duties sit alongside that content.
Markdown remains the authored source by default; machine-readable views do not become a
substitute plan for an executing agent. If a later representation change is justified,
preserve the full content and its meaning rather than regenerate a shortened plan from
tags. In this plan's migration, keep bodies unchanged except specific factual corrections,
reference repairs, selected enduring-meaning transfers and reviewed retirement.
Instruction/skill changes must not change the substantive guidance in `create-plan` or
its suggested structure and implementation/assessment examples.

## Decisions

### Operator-confirmed rule changes

The following choices were explicitly confirmed on 2026-10-09. Confirmation selects the
direction; the existing decision route still owns any governance adoption. The source
review's current-design verdict remains Revise and does not constitute acceptance of a
new implementation.

| Review rule impact | Confirmed direction and constraint | Adoption route |
|---|---|---|
| [RC01](../design_review/reviews/design_review_workspace-content-lifecycle_2026-10-09.md#rc01) | Shared lifecycle interpretation with collection defaults, preserving native statuses, ownership semantics and document content | WC00 records the metadata/governance decision through an ADR and scoped target review; WC02 implements the shared reader/defaults and consumer migration |
| [RC02](../design_review/reviews/design_review_workspace-content-lifecycle_2026-10-09.md#rc02) | Explicit asset delivery independent of retained evidence | WC00 includes the publication boundary in the decision; WC03 changes publisher selection and the owning publication guide/configuration |
| [RC03](../design_review/reviews/design_review_workspace-content-lifecycle_2026-10-09.md#rc03) | Focused instruction routes and lifecycle duties, with the substantive content-authoring contract preserved as stated above | WC07 updates instructions, routing and lifecycle metadata/closure guidance; no change to how agents reason about or write implementation plans |

Use the ADR allocation and status workflow already defined in the decision rules; do not
reserve an ID in this plan or mark a record accepted here. WC00's review must assess the
concrete target and preserve the source findings, rather than treat the old Revise review
as target acceptance. Accepted records remain immutable apart from their existing allowed
status/reference-relocation operations. ADR-0096's Git-history archive policy remains.

### Scope and defaults

Include removal of obsolete pre-28 plan documents and a first qualified resource-reclamation
pass. Resource candidates without eligibility remain protected or explicitly deferred
with an owner and observable trigger; there is no storage deletion quota or promised
free-space result. Legacy plan removal does not require content transfer. Preserve the current
`100G` compiler-cache budget and 50 GiB assessment floor until measurement supports a
separate change. The existing one-temporary-campaign policy concerns compiler campaign
targets per checkout, not agent concurrency.

Keep production/test bodies outside the relevance review; exact consumer/reference searches
are permitted. Support tooling and its own focused tests are the implementation surface.
Do not inspect `.venv*` contents, database contents, personal transcripts, archive payloads
or unrelated source to infer relevance. Persistent scientific/service state, shared caches,
shared skill targets and toolchains retain their existing owners. No general age sweep,
automatic authored-document deletion, blanket Cargo clean or new end-of-turn hook is selected.

## Current foundations and design consequences

The existing publisher already owns collection discovery, Current/Reference/History,
stable section anchors, mdBook/Pagefind delivery and recoverable site replacement.
`docs/site.toml` explicitly selects current work; its empty selection currently places all
plan bodies in History. Fix this through the existing binding, not a status-to-authorization
inference. Existing documentation/decision owners retain their roles.

The frontmatter paths in `scripts/docs.py` and `scripts/adr.py` are separate limited
interpretations. The publisher extracts title/status; the ADR parser recognizes a small
flat YAML subset and overwrites repeated keys. A shared library-backed reader is required
before richer lifecycle consumers are introduced. Preserve ADR validation, historical
references and section-owner behavior while removing replaced parsing paths.

The existing assessment resource owner is stronger than a directory-age policy:
`register_report`, `reference_report`, `finish_report`, `eligible` and `reclaim` associate
exact identities, outcomes, drain, references, borrowers and pins. Reclamation reserves an
exact resource and checks directory/file identity without following symlinks. Unknown old
reports remain protected. Native generations have a separate identity/current-pointer/
borrower-scoped collector; compose with it rather than import them into an evidence registry.

One foundation correction precedes broader reclamation. `finish_report` currently classifies
artifacts by a small filename allowlist, `.json` suffix and `selected` substring. The
assessment producer writes `summary.md`, while the allowlist names `summary.txt`; source
diffs/tar snapshots can be classified as removable while large JSON remains. Replace that
heuristic with producer-declared artifact meaning. A compact receipt is not a substitute
for the source snapshots or origin artifacts its claims and references require.

The source review's Plan 23 premise was incorrect: live blueprint §24.2 summarizes
Plan 22 Q1, not Plan 23 qualification. The maintainer has explicitly retired the old
plans; this does not require transferring that historical qualification into current
documents. Required historical citations use immutable originals without presenting
old results or unresolved items as current work.

## Architectural drivers and scenarios

Use [review scenarios S01–S07](../design_review/reviews/design_review_workspace-content-lifecycle_2026-10-09.md#8-revealing-scenarios-and-alternatives)
as the scenario owner. The following consequences shape the target rather than introduce
a second scenario registry:

- An active owner must be discoverable without presenting superseded advice as Current;
  a done plan with a live handoff must remain available without implying new authorization.
- Metadata interpretation must preserve family-specific meaning, body bytes and historical
  evidence when a family or parser changes.
- A large retained investigation must not enlarge normal site delivery unless selected.
- A path replacement, new borrower or retained origin reference must defeat stale cleanup
  eligibility; failure/interruption cannot become inferred success.
- A parity/workflow change must reach concise public summaries without requiring repeated
  manually maintained architectural explanations.

These boundaries follow AP-01/AP-04/AP-06/AP-07: interpret once, select only necessary
delivery, reuse real lifecycle owners, and keep routine reading separate from explicit
whole-workspace observation. A central content service, full-text repository ingestion or
new artifact database would add synchronization and maintenance without a demonstrated need.

## Target design

### Shared document interpretation

Introduce a tooling-owned reader in `scripts/document_metadata.py`. It separates original
frontmatter/body bytes, parses only metadata, and returns a typed interpreted view plus
the untouched body. It imports no product package. Consumers use its interpretation;
they do not write separate title/status/owner parsers or infer deletion from those fields.

Use `ruamel.yaml==0.19.1`, without extras, declared once in a `docs` dependency group in
`pyproject.toml` and resolved by `uv.lock`; include that group in the existing dev group.
Declare `markdown-it-py==4.2.0` in that same group for native content-table interpretation;
it is already resolved transitively. Use the commonmark parser with only its table rule
enabled. Preserve GFM escaped-pipe semantics and source locations; reject malformed or
ambiguous bound tables rather than silently padding/truncating selected scope rows.
Select `YAML(typ="safe", pure=True)` with `allow_duplicate_keys=False`. Reject YAML merge tags
in the frontmatter boundary using the library's composed node representation, with a
visited set for alias cycles; reject cyclic metadata rather than recursively traversing
it without termination. Load one mapping with string keys and no arbitrary object tags.
Preserve existing date semantics in each adapter; normalize interpreted date values where
needed without dumping/reformatting the source. Recreate parser instances after failure.
The [library API](https://yaml.dev/doc/ruamel.yaml/api/) and
[exact release](https://pypi.org/project/ruamel.yaml/0.19.1/) support this choice; corpus
compatibility and the application validation remain to be tested. PyYAML's current
transitive presence does not provide direct ownership or duplicate rejection.

`docs/lifecycle.toml` owns the versioned lifecycle vocabulary, collection/bundle defaults
and sparse current exceptions. It references the publisher's collection roots rather
than declaring a second publication tree. It is not a per-file retirement-history ledger.
Native family fields retain their existing validators and meanings. Recognize these
additional fields when authored or inherited:

| Field | Meaning and owned behavior |
|---|---|
| `doc_role` | Contract, instruction, decision, plan, review, reference, evidence or index |
| `doc_owner` | Canonical document/disposition owner; existing owner aliases normalize into this read value |
| `doc_provenance` | Authored, generated or observed; source/producer bindings reuse existing generator/receipt declarations |
| `doc_topics` | A small declared task vocabulary for discovery, initially aligned with existing documentation task routes |
| `doc_retention` | While-current, while-dependent, owner-managed or protected-unknown; never a deletion grant |
| `doc_retirement_trigger` | Owner-release, replacement-adopted or producer-policy; a controlled event, not executable prose |
| `doc_retention_reason` | Explanation of an exception/dependency, not an independent release operation |
| `doc_retention_owner` | Optional distinct resource manager; otherwise defaults to the canonical owner and is omitted |

Resolve global safe defaults, the existing collection default, nearest declared bundle
default, then explicit document fields or an exact-file exception. Reject equally
applicable or ambiguous bundle defaults. An exception and authored field cannot silently
disagree: reject conflicting explicit definitions. Canonical `doc_*` owner references
and configuration reference fields are repository-relative, with optional section
fragments; legacy `disposition_owner`/`disposition-owner` references retain their
native semantics: explicit `docs/` paths are repository-relative, other relative paths
are source-relative, and immutable historical references remain historical. Define
bundle/exception selectors relative to the repository
root too. Normalize references using that representation's declared base before
precedence, existence and conflict checks; never reinterpret a canonical `docs/...` path
relative to its containing document. Do not author a second owner field for the same
responsibility. Missing optional ownership can produce the valid protected-unknown
classification; it cannot satisfy action eligibility.

Validate controlled field types/values and declared topic/owner references; reject unknown
`doc_*` keys. Preserve recognized family metadata without imposing a global closed schema
on ADRs, plans or skills. Existing status, decision, pins, regenerated and evidence fields
remain native; an ADR status, plan status and review decision are not interchangeable.
Collection defaults and exceptions classify immutable records/generated outputs without
rewriting them. Metadata changes to mutable files touch only their metadata slice.

Provision `.venv-docs` only through explicit documentation bootstrap using the locked
docs group. The verified uv contract is `uv sync --locked --only-group docs
--no-install-project` with `UV_PROJECT_ENVIRONMENT` selecting that environment;
`--no-install-project` alone would still install runtime dependencies. The explicit docs
bootstrap must select an environment distinct from the product environment: an ambient
product `UV_PROJECT_ENVIRONMENT` must never redirect this exact sync into `.venv` and
remove its packages. The maintainer selected explicit `--docs` mode: choose `.venv-docs`,
visibly report replacement of inherited selectors and reject product-environment path
aliases before effects. Keep
environment selection at `scripts/pse_env.py`, preserving its caller/local/default
precedence for ordinary commands, and let thin recipes/direct CI consumers use the
already provisioned docs interpreter.
Do not sync the product, compile the extension or access the network on ordinary reads.
Keep bootstrap/tool-specs importable before the parser exists through lazy metadata-path
imports. Missing tools produce an actionable bootstrap instruction. See
[uv synchronization semantics](https://docs.astral.sh/uv/concepts/projects/sync/).

Migrate the publisher and ADR parser consumers together, including index/lint/supersession
paths, docs/ADR tests, `turn-end`'s ADR-index path, setup checks and direct manual-workflow
callers. Delete the replaced regex/flat parser only once all callers use the shared reader
and targeted tests pass; retain no competing fallback parser.

### Discovery and publication

Keep `docs/site.toml` as the owner of Current/Reference/History and publication selection.
Explicitly select actual active owners from Plan 28 onward. Retained completed plans
from that range remain available only for their real dependencies/retention conditions.
Do not select every
in-progress document by status or copy the plan index into a second page/status list.
Move the two PostgreSQL conversational notes out of Current discovery while preserving
the rationale still cited by ADR-0112/0114 and Plan 22.

Add explicit non-Markdown asset declarations to the publication configuration, with
named evidence-bundle selections where a download is intended. Keep theme/rustdoc handling
under their existing owners. Only selected assets are staged; every declared asset must
exist and remain inside its permitted root. A retained JSON/archive is not automatically
delivered. Preserve referenced images/attachments and verify rendered internal links with
the existing publication checks. Do not narrow retention to make an asset-selection test pass.
Failed staging/render/index/replacement preserves the previous successful site; successful
replacement removes obsolete delivery without deleting original evidence.

Immutable ADR metadata also includes a closed historical encoding gap: 22 retained
records contain 37 scalar spans that the previous flat reader treated literally but
strict YAML rejects. Declare only canonical path/key/raw-scalar digests, normalize matching
spans in memory before the same strict parse and preserve original bytes and literal values.
This is proactive family adaptation, never error-triggered fallback or a source rewrite.
Changed/undeclared spans, duplicates, merges and cycles remain strict failures. Historical
baseline reads use the same document identity. Correct the mutable ADR template directly.
The [scoped target review](../design_review/reviews/design_review_workspace-content-lifecycle-target_2026-10-09.md)
accepts this bounded adapter without a new immutability exception.

### Lifecycle operations and storage observation

Add a small `scripts/document_lifecycle.py` surface for read-only inventory, validation
and retirement-plan output. Path/role/topic/scope selection consumes the shared view.
JSON output is versioned, deterministic for a fixed snapshot and explicit about unknowns,
exclusions, provenance and observation time. Retirement-plan output identifies exact files,
the surviving meaning owner, references to repair and blocking consumers; it does not
delete authored files. Do not add a mandatory new lint or housekeeping step to ordinary
turns. Extend existing docs/support checks for the new contracts.

Provide an aggregate scope query across retained Plan 28+ documents. Bind authoritative
native packet/finding tables through explicit selectors in `docs/lifecycle.toml`; derive
item identity, source links, raw status, owner and cross-plan relationships from the
selected content rather than copying editable statuses into a central backlog. Qualify
item IDs by repository-relative document, binding ID and native row ID. Bind by unique
heading path, header signature and native ID/scope/status columns, so repeated IDs in
different sections do not collide. Missing or ambiguous bindings produce diagnostics.
Only explicit binding-defined status vocabulary supplies a class; retain raw cell text
and source links/locations. A table without status reports unknown instead of inferring
completion from prose. Preserve completion versus implementation/qualification
distinctions and expose ambiguous or missing state as unknown. Sparse relationship
bindings may identify a dependency, supersession or another canonical owner without
duplicating that owner's status. Do not infer semantic overlap from similar titles.
The query supports concurrent plans and reports aggregate open/blocked/deferred/complete/
unknown scope with original evidence wording; it schedules or authorizes nothing. Markdown
table syntax/fixtures exercise escaped pipes, inline code/links and fenced examples using
the pinned parser rather than a competing general Markdown grammar.

Extend `scripts/build_storage.py` with structured observation of configured repo-linked
stores, including ownership class, allocated/apparent bytes, active-use uncertainty and
known producer/resource identity. Shared skill targets/caches are reported as shared;
symlink aliases are not duplicate trees. Metadata-only grouped observation preserves the
source/test/environment exclusions. Persistent state remains a semantic-owner boundary.
Report failed/unavailable observations explicitly; do not convert them into empty success
or additive reclaimable totals. Normal document queries do not trigger host-storage scans.

Mutating disposal remains at the existing producer/tool owner, selecting exact identities.
There is no universal `prune` sweeper. New output uses a producer/attempt cohort boundary
where useful; old paths are adopted only when justified, not renamed for uniformity.

### Producer-declared artifact meaning

The assessment producer supplies an explicit artifact manifest to its existing resource
owner. Distinguish retained receipts, provenance, consumed evidence, disposable scratch
and unknown artifacts. Keep classification and identity/digest together in the versioned
resource record rather than create another registry. The producer declares each owned
artifact; an undeclared file is protected. Seal the manifest with finalized output
identity before eligibility or reference reuse. Reclassifying retained evidence as
disposable scratch requires explicit owner release and reference reconciliation; merely
recomputing a manifest cannot downgrade protection. Retained-reference acquisition before
reading, transfer before release, actual drain and exact cleanup reservation remain
unchanged.

Display-only report reads also acquire temporary borrows over the selected report and
actual referenced origins before payload access, resolving a latest pointer once. They
create no persistent consumer reference, checkpoint or cleanup. New lifecycle activation
waits for affected old producers/reclaimers to drain; active records are not migrated.

Always retain the actual summary/receipt names and required source/environment snapshots
while their claims or downstream references need them. Protect authored uncommitted patch
or tar snapshots until their owner proves durable preservation. XML/log/JSON extension
does not decide relevance. A receipt-referenced artifact cannot be removed merely because
a compact result looks sufficient. If a producer intentionally makes an artifact optional,
its receipt/navigation and reuse contract must express that before publication; never
rewrite a referenced origin's digest to conceal later loss.

Automatic finalization handles only explicitly disposable scratch from a complete,
reconciled successful run, with verified provenance, no pin/reference/borrower and actual
drain. Failure, incomplete/interrupted state, source drift, malformed manifests, unknown
legacy records, identity changes or unsupported paths remain protected. Manual release
of failed/historical resources requires their actual owner and preserved required evidence;
it does not fabricate a successful test outcome. Old manifest-less records stay protected
until exact adoption; no compatibility cleanup heuristic remains for new reports.

### Instructions and skills

Keep essential startup authority, dirty-work preservation, generated/off-limits boundaries
and execution rhythm at the root. Move only detailed mechanics to focused existing owners
and verify both runtimes can reach them. Preserve explicit Codex routing for conditional
rules; do not assume later file access loads nested instructions. `CLAUDE.md` remains a
runtime adapter. Shared library skills retain repo-agnostic bodies and reproducibility
inputs; local selection does not confer authority to remove shared targets.

Add lifecycle metadata/ownership pointers and closure responsibilities alongside existing
process-skill guidance: reconcile references, transfer enduring meaning and request release
from the evidence owner. Do not change how plans explain the change, implementation vision,
alternatives, assessment or completion, or their organization/styling guidelines. Lifecycle
validation checks metadata and resource eligibility, not the quality of plan reasoning.

## Plan

The root owns shared declarations, integration, this status table and acceptance. Bounded
mapping/library inquiries may run independently; execution owners coordinate overlapping
publisher/ADR/configuration edits. Worktrees are needed only for genuinely conflicting
concurrent implementation, not routine plan work.

| Packet | Responsibility / dependencies | Scenarios / acceptance | Replaced mechanism / deletion | Status |
|---|---|---|---|---|
| WC00 | Record governance/metadata target and focused foundation corrections; route required ADR and target assessment | S02/S03/S07; concrete target has required scoped review/decision adoption before dependent governance implementation | No rewrite of accepted ADRs or content-authoring standard | Complete |
| WC01 | Correct demonstrated public facts and explicit discovery using existing owners; independent of parser delivery | S01/S07; real scope projection exposes selected owners and historical advice is not Current | Remove obsolete entrypoint claims and repeated detailed explanations only where replaced by an owner link | In progress |
| WC02 | Implement shared metadata interpretation, minimal defaults/exceptions, aggregate content-level scope, docs bootstrap and all parser consumers; depends on WC00 | S02/S07; native metadata/corpus, concurrent-plan scope/owner reconciliation, byte preservation, strict failure cases and bootstrap isolation pass | Delete both replaced parsing mechanisms and their obsolete tests/callers | In progress |
| WC03 | Separate asset selection and evidence retention; consumes WC02's bundle interpretation | S04/S06; selected assets work, unselected evidence remains, failure recovery and obsolete delivery pass | Delete implicit collection-wide asset copying | In progress |
| WC04 | Replace report compaction heuristics with producer-declared artifact roles and integrate validation/receipt consumers; depends on WC00 | S03/S05/S06; truthful outcomes, provenance, references, drain, identity/race and manifest tests pass | Delete filename/suffix classification for new reports; old unknown records protected | Complete |
| WC05 | Extend observation and qualify/reclaim selected legacy output cohorts through actual owners; depends on WC04 and WC02 where document dependencies are consumed | S03/S05/S06; selected eligible artifacts reclaimed, protected cohorts untouched and observations report actual scope | Remove only named proven disposable outputs; no global cache/state cleanup | Complete |
| WC06 | Remove obsolete pre-28 plans/companions and named completed reviews; independent of new metadata delivery | S02/S03/S07; required retained references/tool consumers work; no legacy salvage or scope import | All retained pre-28 plan documents and companions; no archive tree or content transfer | Complete |
| WC07 | Implement runtime routing and lifecycle duties; consumes settled WC02/WC04 contracts | S01/S02/S07; required guidance reachable and substantive plan-authoring contract preserved | Remove relocated mechanics only after both runtimes have working routes | Complete |
| WC08 | Assembled scoped qualification, measurements and handoff after functional scope | S01–S07; integrated docs/discovery/lifecycle cases and relevant static checks; results against zero baseline | Retire temporary migration machinery/audit output only after explicit release | Planned |

### WC00–WC03: establish interpretation and truthful delivery

WC00 records the confirmed rule choices, content-preservation constraint, chosen parser,
source/retention/publication boundaries and artifact-owner composition. A focused target
review addresses those new contracts and their failure cases; it does not restart the
scientific review or re-audit the entire repository. Apply any accepted governance changes
through the existing architecture/ADR route before dependent implementation.

WC01 corrects the named README, CONTRIBUTING, CITATION and bug-template claims against
their current owners. Use a generated fact only when a concrete canonical declaration
already supplies it; do not create another pin table or prose authority. Reconcile
selected owning plans rather than assume the review's dated status census is current.
This small relevance repair can precede the new schema.

WC02 migrates complete parsing/validation consumers and the current metadata corpus,
using collection/bundle defaults instead of boilerplate on every file. Explicit unknown
classification is a supported protected state, not a hidden schema-error allowance.
Choose targeted tests that distinguish absent/inherited/explicit values, alias conflicts,
native fields and immutable records. Bootstrap must work without the parser preinstalled,
and a normal read must not synchronize the product or download anything.

WC03 makes selected assets an explicit input to staging and exercises a real render/search
journey as well as isolated fixtures. Measure original/staged asset bytes under the same
input selection; a change in staged bytes is a publication result, not measured agent
latency or proof that original evidence is disposable.

### WC04–WC05: lifecycle foundations and the first output pass

WC04 changes the assessment producer and its resource owner together. A declared manifest
must cover the real producer outputs, including `summary.md`, source diffs/tar snapshots,
receipts and report references. Preserve reservation, no-follow descriptor traversal,
digest/identity rechecks and reference/drain lifetimes. Reuse the native collector unchanged
unless an actual integration defect is demonstrated. Do not treat document metadata as
authority to release a resource.

WC05 observes and classifies the following initial cohort set. Revalidate the live tree
and owner metadata; these baseline sizes are investigation priorities, not savings.

| Cohort | Authoring classification and next eligibility decision |
|---|---|
| `build/plan11/` (about 6.4 GiB) | Protected unknown; repeated runs contain very large source diffs. Resolve outcome, exact producer/owner, references and durable authored-input preservation before adopting any disposable subset |
| `build/plan10/` (about 2.0 GiB) | Protected unknown; receipts and cleanup-candidate filenames do not confer release. Apply the same exact scope/provenance/reference qualification |
| `build/assessment/` (about 4.8 GiB) | Protect the aggregate; current qualification readers exist. Qualify individual attempts, not the root. Large JSON/XML/logs require artifact semantics, not extension pruning |
| `build/workspace-review-2026-10-09/` (below 1 MiB) | Retained input to this plan; release only after durable measurement/classification meaning and required provenance are preserved and no snapshot consumer remains |
| Native/cache/campaign stores | Use their actual collector/tool owner only for a demonstrated eligible exact identity; preserve active targets, selected native generations and current configured budgets |

Before action, produce a bounded candidate list with exact owner/identity, outcome,
provenance, references, active-use/drain state and proposed disposition. Adopt legacy
artifacts only after those premises are established; pin unknowns rather than synthesize
pass from a historical filename. Execute the eligible subset through the actual owner,
then record what was removed/protected and actual before/after allocation under stated
concurrent-work conditions. Do not require that all unknowns become removable to complete
the bounded pass. Each remaining deferral names the evidence and owner needed to reopen it.

#### Bounded first resource pass, 2026-10-09

The read-only owner/metadata pass selected the four named legacy cohorts below. No candidate
had a sealed producer artifact manifest or sufficient exact durable-input/provenance proof
for adoption; the eligible subset was empty and no legacy bytes were deleted. This completes
the bounded pass without implying that retained storage is all necessary or all disposable.
The allocation observer reports overlapping, non-atomic rows and does not sum them.

| Cohort | Directory identity (device/inode/uid) | Observed allocation | Owner evidence and disposition |
|---|---|---|---|
| `build/plan10/` | `66306/36708886/1000` | 2,197,307,392 bytes | No registered report owner; protected unknown. Reopen with campaign owner's exact outcome, references and durable authored-input proof |
| `build/plan11/` | `66306/14814739/1000` | 6,857,015,296 bytes | No registered report owner; protected unknown. Same campaign-owner qualification trigger |
| `build/assessment/` | `66306/39483400/1000` | 5,179,871,232 bytes | 30 registered legacy attempts: 25 pass, four failure, one incomplete; five pinned, no recorded references/borrows, zero manifests. Protect all; outcome alone is insufficient. Assessment owner may reopen exact attempts with provenance and durable-input proof |
| `build/workspace-review-2026-10-09/` | `66306/12994084/1000` | 675,840 bytes | Retained plan/review input; no resource owner. Plan 32 handoff owner releases only after durable provenance and dependency reconciliation |

Active-use absence is not inferred from old filenames or missing registry rows. The scoped
activation observation found no affected producer/reclaimer process or live registered report
publisher; unrelated resident stores/cache services were preserved. Already-loaded old
reclaimers must drain before cutover because a new manifest cannot constrain old code.
New real producer outputs are all receipts/provenance/consumed evidence, so they intentionally
supply no scratch savings. Historical reports remain displayable; new retained-reference reuse
requires a sealed origin and unchanged receipt. Existing historical test results keep their
original meaning and conditions. Native/cache/persistent stores retain their actual owners;
no exact eligible identity was selected from them.

### WC06: remove obsolete plan documents

Remove all retained pre-28 plans and companions, plus the three named completed Plan 25b
reviews. Do not salvage content, backfill metadata, reconcile old open items, transfer
qualification or import scope into Plan 28/32. Later contrary decisions or invalidated
assumptions settle affected historical items as superseded/no longer applicable; no
item-by-item exercise is required for removal.

Repair only references needed by retained documentation/tooling, using the same original
path at an immutable Git commit where historical citations are required. Accepted records
retain their original meaning under ADR-0096's reference-relocation exception. Preserve
unrelated uncommitted work. The exact reference search checks live consumers, not whether
the obsolete scientific scope was implemented. Metadata structuring and ongoing plan
lifecycle management apply to retained Plan 28+ content, with multiple active documents
and an aggregate view over their separately owned work.

### WC07–WC08: usable guidance and completed scope

Review instruction/skill diffs specifically for accidental changes to the substantive
plan-authoring contract. Add only metadata/lifecycle/routing duties alongside that
contract. Validate root-launched and relevant nested runtime instruction routes; startup
byte counts and on-demand document bytes are separate observations. Do not remove shared
skills or parallel agent freedom to make an instruction-size number smaller.

WC08 assembles the published-site and lifecycle journeys once functional scope is complete,
including the actual retained corpus and the qualified first cleanup. Record limitations
and remaining protected/deferred items, then transfer enduring mechanics to the existing
documentation/environment/build guides and rationale to the adopted decision. This plan
and its source review retire only under their own resolved-dependency conditions and
number-allocation safeguards.

## Finding dispositions

This table owns the adopted findings; packet execution evidence will be linked here rather
than copied into multiple status records. Planned work is not resolved evidence.

| Finding reference | Scenario reference | Disposition | Decision / work owner | Evidence or revisit trigger |
|---|---|---|---|---|
| [WCL-F01](../design_review/reviews/design_review_workspace-content-lifecycle_2026-10-09.md#f01) | S02/S03/S07 | open | WC00/WC02, with WC07 adoption | Shared interpretation and real consumer migration; body/native-state/ownership preservation |
| [WCL-F02](../design_review/reviews/design_review_workspace-content-lifecycle_2026-10-09.md#f02) | S01/S02 | open | WC01/WC08 | Correct real Current/Reference/History projection and deliberate retained handoffs |
| [WCL-F03](../design_review/reviews/design_review_workspace-content-lifecycle_2026-10-09.md#f03) | S04/S06 | open | WC03/WC08 | Selected asset delivery with retained original evidence and recoverable publication |
| [WCL-F04](../design_review/reviews/design_review_workspace-content-lifecycle_2026-10-09.md#f04) | S07 | open | WC01/WC07/WC08 | Named stale claims repaired and owner routes preserved without changing authoring reasoning |
| [WCL-F05](../design_review/reviews/design_review_workspace-content-lifecycle_2026-10-09.md#f05) | S03/S05/S06 | open | WC04/WC05/WC06/WC08 | Explicit producer artifact meaning, bounded live eligibility and exact qualified initial pass; unresolved cohorts stay owned/protected |

## Verification

**Interface-checked:** authoring inspected the current publication/parser/environment,
assessment artifact/provenance/receipt and resource-owner paths. The library comparison
used Context7 and primary sources; parser/bootstrap contracts have not been executed here.
**Proposed:** the target, migrations, packet acceptance and cleanup actions above.
No product behavior, cleanup savings or agent-performance benefit was Tested or Measured
by plan creation. The source review's measurements retain their original dates/conditions.

During implementation, run targeted support-tool tests with each changed mechanism and
delete replaced paths once all callers migrate. Relevant existing acceptance surfaces
include `just docs-test`, `just setup-test` and the focused report-resource tests; use
isolated temporary trees and controlled process identities for destructive cases. Preserve
the zero-failure baseline. Do not start solvers/databases or compile product code merely
to validate document metadata or filesystem lifecycle.

| Claim / risk | Required acceptance evidence |
|---|---|
| Interpretation without content redesign | Native corpus and fixtures preserve Markdown bodies, family states, dates and owner meaning; normalize each reference representation correctly; reject duplicates, merge/cycle ambiguity, ambiguous bundles, malformed/unknown controlled fields and conflicting explicit defaults/aliases |
| Independent bootstrap | Docs-only sync installs the exact group without runtime/native project installation or mutation of an ambient product environment; bootstrap/tool-specs work before parser availability; normal reads perform no sync/network |
| Discovery and delivery | Actual corpus scope projection, rendered navigation/search, selected images/downloads, retained unselected evidence, nonexistent/escaping assets and failed/successful publication replacement |
| Artifact role and truthful outcome | Real producer manifest retains actual summary/provenance/required artifacts and is sealed to finalized identity; role downgrades require owner release/reference reconciliation; undeclared/failed/incomplete/drifted/manifest-less artifacts protected; valid successful scratch eligible |
| Concurrency, identity and recovery | Active/new borrowers/references, non-drained units, competing cleaners, changed digest/dev/inode/owner, symlinks/traversal and interruption/retry cannot delete unrelated or protected bytes |
| Legacy plan removal | Pre-28 plans/companions absent; required retained references and tool consumers work; no content salvage, old-scope reconciliation or Plan28 scope import |
| Concurrent plan scope | Aggregate native-content view covers retained Plan28+ owners, qualifies item IDs, links canonical status owners/dependencies/supersession and reports ambiguous states honestly; no single-plan execution restriction or duplicated status ledger |
| Resource reclamation | Exact candidate identity, provenance, references, live consumer/drain recheck and owner release; report actual removed/protected subsets without inferred savings |
| Agent guidance | Both runtime routes retain critical startup obligations; substantive create-plan content guidance remains intact; no new routine housekeeping gate |

After all functional scope, WC08 runs the relevant assembled documentation/search and
lifecycle journeys, existing offline link/ADR/agent checks, and the scope-end checks required
by the repository's execution rhythm. Select affected checks deliberately; this tooling
scope does not acquire a native scientific campaign. `just ready` applies after the
dependency/environment wiring change. Regenerate only when an actual generator declaration
changes, and never hand-edit generated reference. Report commands, modes, scope, zero
baseline and actual results; a fixture pass does not certify the full metadata corpus.

Measure staged asset bytes before/after the same publication selection and actual loaded
instruction bytes under stated runtime/startup conditions. Record cleanup allocation
changes with concurrent-work/hardlink limits. These are bounded artifact/loading/storage
results, not causal productivity, scientific performance or optimal-cache-budget claims.

## Open items and current checkpoint

Work is paused at the maintainer's request for later resumption. The
[implementation and design handoff](../design_review/evidence/workspace-content-lifecycle-2026-10-09/handoff.md)
captures implemented files, checks, storage limitations and the subsequent representation
discussion. The metadata, publisher, resource, retirement, observer and instruction changes
are present, but this plan is not complete.

The document target is reopened: the maintainer objected to split authoring between Markdown
and external metadata/bindings and asked about unified structured documents. Design reviews
must be first-class parts of the comprehensive document/workspace lifecycle, and the structured
model must capture the existing skills' content guidance without changing that guidance.
Structured Markdown and canonical YAML were researched; no replacement format was selected
or implemented. Resume with this design question, not final closure of the earlier target.
Plan 28 scope and its campaign remain at their existing owners.

[ADR-0168](../adr/0168-workspace-content-lifecycle.md) is proposed; the maintainer-authorized
target is recorded at blueprint §24.4.1 (revision 140). The independent scoped target review
accepts the earlier design at Proposed strength, including the closed immutable scalar adapter;
it does not assess the reopened representation or accept implementation or change ADR status.
WC05 owns the unresolved producer/outcome/provenance decisions for
legacy resource cohorts. WC06 is direct obsolete-document removal with necessary reference
repairs; no Plan 23 evidence transfer or Plan 27 scope reconciliation is pending.

## Outcome (recorded after implementation)

Implementation and targeted verification are in progress. Final assembled acceptance will
record what was built and retired, a mistake made and corrected, deliberate deviations,
named verification with conditions, bounded measurements and remaining owned deferrals.
Enduring mechanics now belong to the documentation/environment/build/validation guides;
this record remains until its handoff and dependent evidence have a durable archive.
