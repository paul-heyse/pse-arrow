---
title: Current architecture documentation and legacy retirement
status: draft
date: 2026-09-25
adrs: [ADR-0094, ADR-0095]
phase: 1
review_sources: []
scenario_sources: []
---

# Current architecture documentation and legacy retirement

## Context

**Proposed execution plan.** Fully migrate the remaining documentation onto the system
delivered by [Plan 18](18-architectural-documentation.md). Current architecture and its
rationale become the normal reading surface. This is a handoff for another agent, not
a claim that the migration or product implementation has already happened.

The maintainer explicitly prefers useful current explanation over a maintained history
of early design changes. Git history is the default archive. Do not replace the old book
with an equally large published archive, reconstruct every supersession chain, or make
agents read old decisions to understand the current contract.

### Current execution baseline

Rebased on the completed Plan 18 outcome and the current Plan 16 closure records on
2026-09-25. The earlier P10–P13-active/P14–P18-unstarted snapshot is superseded by those
records. Completion here is the recorded scoped outcome, not a new qualification run.

| Work | Treatment for this migration |
|---|---|
| [Plan 16](16-data-model-architecture.md) | Complete for its documented local Linux scope, including maintainer-confirmed independent review; supplies the current implemented architecture |
| [P10–P13 execution](16-p10-p13-execution.md) | Complete; historical implementation context, not a resume instruction |
| [P14–P18 execution](16-p14-p18-execution.md) | Complete; owns final implementation, qualification, measurements and [independent-review closure](16-p14-p18-execution.md#independent-review-and-closure) |
| Plan 16 P00–P09 packets | Completed scoped context; retain useful facts until extracted, then retire by the same rule as the later packets |
| [Plan 17](17-architecture-first-design-review.md) | Complete and inactive. The desired review system is in place. No pilot, process-effectiveness campaign or other rollout work remains |
| [Plan 18](18-architectural-documentation.md) | Complete and inactive; its publisher and documentation rules are the baseline for this migration |
| Plans 01–15 and their packets | Complete and/or superseded as workstreams, even when their metadata says otherwise. They supply no implicit backlog or prerequisite to this migration |
| This plan | Documentation migration and its necessary document-consumer cleanup; becomes active when execution is requested. No older workstream is an implicit prerequisite |

Retired means no longer an execution obligation. It does **not** mean every historical
test passed or every former proposal was implemented. Preserve that distinction in any
retained qualification statement. A file under an old plan can still be a live tool input;
its directory and age do not authorize breaking the consumer.

### Completed Plan 18 capabilities to reuse

Plan 18 is fully complete. Its recorded verification and actual implementation are the
starting point, not tasks to repeat or capabilities to rebuild:

| Existing owner | Completed capability | Plan 19's remaining change |
|---|---|---|
| `docs/site.toml`, `scripts/docs.py` | Collection discovery, explicit scope overrides, tool pins, staged source and generated navigation | Curate current work and retained content through the existing configuration |
| `docs/book.toml`, `docs/theme/` | mdBook rendering and scoped, on-demand Pagefind UI; built-in search already removed | Keep the renderer/search system; adjust only scope presentation if retention makes it necessary |
| Shared `scripts.adr.section_owners` and the publisher's section directory | Unique numbered owners, staged anchors and generated cross-document navigation | Move remaining contracts using this resolver, not a second citation map |
| Existing reading guide and design-change workflow | §§0.1/0.3/0.4/24.4 already extracted with old-anchor links | Update only lifecycle/current-baseline wording; do not perform this extraction again |
| `just docs`, `docs-test`, `docs-serve`, `bootstrap-docs`; docs CI and doctor | One installed-tool/build path, optional rustdoc, successful-artifact replacement and failed-build recovery | Reuse recipes, fixtures and CI; no tool migration, installer or new publishing service |
| AGENTS.md, shared rules, process skills, canonical/generated roles and entry READMEs | Documentation ownership and bounded-context guidance already deployed | Repair stale routes and align the new retirement policy; audit before editing |

[Plan 18's verification](18-architectural-documentation.md#verification) records the
publisher, browser and link checks. Do not repeat its qualification merely to accept
that baseline, or inherit its historical out-of-scope spelling findings as new work.
ADR-0095's proposed status/formal supersession route is separate from the completed
implementation; no pilot, deployment or retrospective approval campaign blocks Plan 19.

Plan 19 deliberately extends Plan 18's incremental extraction policy into a full content
consolidation. MD01 records that new scope; it does not reopen the publishing-tool choice.

### Repository facts that affect sequencing

Inspection for this plan found:

- `docs/site.toml` selects every Plan 15/16/17/18 page as Current. The plans README still
  lists completed workstreams and mentions a Plan 17 pilot follow-up.
- Plan 16 and its last two packets now say done and record local qualification and
  independent review. AGENTS.md still foregrounds P00–P04 and describes later packets
  as future; the publication/current-work routes therefore need reconciliation.
- The roughly 4,700-line blueprint mixes native contracts with obsolete MathIR, NL/Pyomo,
  pass-pipeline and delivery-phase descriptions. Precedence paragraphs do not make that
  obsolete text useful current context.
- `scripts/adr.py` requires contiguous numbering from 0001, local review paths and
  symmetric local supersession links. Accepted bodies are immutable. Simply deleting
  early records would conflict with that model.
- P17 removed `scripts/implementation_phase.py`, the old phase/acceptance adapters and
  blueprint pin parsing. `tests/governance/tests/dependency_pins.rs` now reads Cargo
  declarations. The previous Plan 14 manifest/pin-table waiting conditions are obsolete.
- `tests/governance/tests/every_crate_registered.rs` still parses the blueprint layout
  and `tests/governance/layout_additions.toml`. This concrete residual document coupling
  belongs to MD03; do not defer it to an already completed P17 or drop useful membership
  checks when retiring the table.
- The register contains old-phase triggers and closed history. Its linter rejects an
  empty register and checks dates against today. Stale rows are not a new backlog.
- Ordinary `test-package`, `assessment`, `native-test`, `native-python` and `case-measure`
  commands exist. Do not reintroduce phase guards, architecture-manifest adapters or
  a second execution inventory during migration.

These are migration inputs, not a permanent inventory to keep synchronized.

## Decisions

### Target information model

1. **Current contracts and their basis:** focused pages in
   `docs/authoritative_design/sections/`, reached through the architecture README.
   Each explains responsibility, consumed contracts, important invariants, why the
   current choice fits, limits and relevant extension points. Link a current ADR when
   it owns substantial rationale; do not repeat that rationale in two places.
2. **Current work:** this migration while it runs and any separately authorized new task.
   Plans 16–18 are completed baseline records, not active implementation. Do not invent
   a successor product backlog. No status database or per-component proof matrix.
3. **Current reference:** useful pinned library maps, generated schema reference,
   enduring development instructions and any still-needed qualification baseline.
   A reference survives because a current question or tool needs it, not because it
   once appeared in a plan.
4. **Reviews and plans:** temporary work records. Once findings and enduring rationale
   have reached their owners, remove completed records from the working tree unless
   a concrete active dependency still needs them.
5. **History:** Git by default, outside site navigation and normal search scopes.
   Keep no full `docs/archive/` mirror, second book, external archive service or
   permanent per-document retirement ledger.

Retain mdBook/Pagefind. Current/Reference remain useful; History and Everything operate
only over deliberately retained published pages. Everything must not silently reintroduce
retired Git history. Remove an empty History option or explain its limited retained
contents; do not fabricate historical pages to populate the UI.

The selected design standard remains Core 3.0 / process-simulator profile 1.1. This
migration does not reopen its adoption, add pilots or redesign its principles. Obsolete
charter/version crosswalks may leave the active reading path once live citations move;
AP/DP/PS identifiers and substantive obligations remain.

Current documentation can name an unresolved design choice without pretending it is
settled. A proposal stays a proposal until its owner decides or implements it. Retain
concise rationale for consequential choices, not an ADR for every local edit or a diary
of alternatives already discarded. Existing document title/status conventions suffice;
add no mandatory owner/symbol/test inventory to every page.

### Retention rule

Ask whether a page is needed to understand, change, operate or qualify the current system,
or to complete explicitly active work. If not, retire it. Do not require an individual
historical investigation or a new ADR for every discarded file.

| Material | Default action | Exception or prerequisite |
|---|---|---|
| Plans 01–15, Plan 17 and Plan 18 narratives | Extract current explanation, then remove | Keep a narrowly identified live consumer's input until that consumer moves |
| Plan 16 and all execution packets | Extract current contracts and a compact scoped qualification basis, then retire narratives | Preserve evidence needed for current claims without reopening completed qualification |
| Plan 16's originating reviews and coverage rows | Retire after extracting any enduring rationale or actual surviving limitation | Completed F/N findings are not a new backlog; no second ledger or wholesale copy of old coverage rows |
| Early ADRs, including records still labelled accepted | Retire if their rationale is obsolete or fully replaced | Adoption status alone does not make a decision current; use MD01's policy |
| Still-valid ADRs | Retain unchanged when compact and self-contained | If obsolete dependencies dominate, replace with a current-basis record under MD01 rather than rewriting the original |
| Historical reviews and old standard files | Remove after active citations/findings have an owner | Do not relabel an old verdict as a new review |
| Plan 14 foundation/M22 material | Extract the surviving contract and an honestly scoped qualification summary | Keep original results actually needed for present claims; no rerun, erasure of a still-used report or reopening of M22 |
| Capability maps and executable evidence | Keep useful current library reference | Inspect skill/build consumers before removing maps, committed locks, probes or evidence assets; no wholesale age-based deletion |
| Generated schema/API reference | Preserve generation ownership | Change through the generator only; no hand edits or product codegen for a prose-only cleanup |
| Acceptance TOML, fixtures or data under old plans | Treat as executable inputs until consumer inspection says otherwise | Exclude from reading routes now; deletion follows consumer replacement |
| Register rows | Keep current deliberate deferrals with an owner and observable trigger | Remove closed/stale history and deferrals satisfied by completed work; retain only genuinely unresolved choices |

### Decision consolidation without a historical migration project

MD01 creates one concise governance ADR through `just adr-new`, using the next available
number. It records the **current retention and rationale policy** and broad retired
collections. It does not need a chain of pairwise supersessions for every old decision.
Allocate this record before retiring numbered records. Retained IDs are never renumbered.
Permit gaps; keep the highest issued record until a newer decision exists, so the current
maximum-plus-one allocator cannot reuse an old number. No ID tombstone catalog is needed.

Keep the immutability check for accepted records that remain. Replace obsolete records
with current-basis records and remove old files under the consolidation policy. Retained
ADRs' review/section references must resolve in the retained corpus or use an explicit
immutable Git source where historical evidence is genuinely needed. Do not retain whole
obsolete chains to satisfy lints. A Git citation is not an automatic design verdict.

Update the ADR tool, template, skills and governance together. Formal ADR status remains
truthful; implementation authorization is not fabricated PR acceptance. The policy and
scoped review precede retirement of protected material. Use the authorized design-edit
workflow and record its purpose. Do not globally disable hooks or immutability protection.

### Relationship to completed implementation

Do not reopen Plan 16, change product contracts, refactor runtime code or rerun scientific
qualification while executing this plan. Preserve its recorded limits and independent
review. Before editing shared files, reread current content and coordinate with any active
owner; preserve unrelated dirty work. No separate worktree or standing reviewer is needed.

There is no remaining planned dependency on P17/P18 completion. MD03 owns the small
surviving blueprint-layout test coupling; MD02 checks whether other old inputs still have
real consumers. Deal with actual document consumers in this migration, not with a stale
whole-product dependency. If an unexpected product change really is required, identify
that specific boundary and disposition without reviving an entire completed packet.

## Architectural drivers and scenarios

| ID | Reader/change scenario | Acceptance |
|---|---|---|
| S1 | A new agent starts work | Root/agent entrypoints reach the current architecture and authorized task; Plans 16–18 are complete and no old packet or pilot is presented as work |
| S2 | An agent asks why a current choice exists | A focused contract and, where useful, one current rationale record answer it without traversing early ADR history |
| S3 | A subsystem changes during implementation | Update enduring semantics at their owner; packet checkpoint owns progress; no source-proof inventory is updated |
| S4 | Search for mathematics, Python boundaries or publication | Current results describe the library-owned/native approach and actual limits; retired MathIR/NL/Pyomo contracts are not presented as supported |
| S5 | Remove an old plan/review/ADR | No live link, register entry, recipe, test or skill loses required input; obsolete content disappears from the working corpus and site |
| S6 | Move a still-valid numbered section | One owner remains, citations resolve and useful old URLs reach the new owner without duplicated normative prose |
| S7 | Close a future workstream | Enduring meaning moves to the contract/rationale owner; completed plans and resolved reviews retire without another cleanup campaign |

## Plan

Execute in dependency order. This table is the sole migration progress owner. Record
short state/next-step notes rather than per-command receipts. Each packet is a handoff
boundary with ownership, actions, acceptance and deletion obligations.

| Packet | Responsibility / dependencies | Scenarios | Replaced mechanism | Status |
|---|---|---|---|---|
| MD00 | Reconcile completed baselines and current routes; none | S1/S3 | Stale work selection, pilot wording and future-packet claims | pending |
| MD01 | Current-rationale/retirement policy and ADR tooling; MD00 | S2/S5/S7 | Mandatory contiguous history and full-chain retention | pending |
| MD02 | Extract enduring knowledge and identify live consumers; MD01 | S2/S3/S5 | Plan-owned enduring explanation and unnecessary history dependencies | pending |
| MD03 | Modular architecture and residual layout-test coupling; MD02 | S2/S3/S4/S6 | Mixed legacy blueprint and duplicated layout authority | pending |
| MD04 | Retire obsolete narratives, decisions, reviews and register history; MD02/MD03 | S2/S4/S5 | Retired working corpus and site output | pending |
| MD05 | Publication, agent and workflow alignment; MD03/MD04 | S1–S7 | Legacy entry routes and duplicated lifecycle instructions | pending |
| MD06 | Qualify the migrated reading surface once; MD05 | S1–S6 | Stale artifacts and broken references | pending |
| MD07 | Verify retirement completion and close; MD06 | S5/S7 | Remaining migration-only retention and obsolete routes | pending; no P17/P18 dependency |

### MD00 — Reconcile completed baselines and current routes

**Own:** plans/architecture/root README routes, `docs/site.toml` and AGENTS.md's direction
paragraph. Historical packet checkpoints are not edited into new progress reports.

1. Read this plan, Plan 18's Outcome and Plan 16's P14–P18 closure/qualification boundary.
   Recheck only if newer work changes that boundary; do not repeat completed campaigns.
2. Replace broad Plan 15/16/17/18 Current selections with actual current work. Route agents
   to the current architecture, and to this plan once execution is requested. Completed
   packet evidence may remain Reference briefly while its useful facts are extracted.
3. Remove old P00–P04/P10–P13 resume directions, future P14–P18 claims and pilot language.
   Plans through 18 are completed or superseded as workstreams. No historical outcome
   is rewritten to claim a pilot or broader qualification.
4. Link the completed Plan 16 qualification basis while it remains needed. Do not duplicate
   its result counts, recreate an active checkpoint or invent a new product workstream.

**Acceptance/deletion:** S1 is clear from the entrypoints. Replaced current-work lists and
pilot obligations are gone; no production obligation or acceptance claim was added.

### MD01 — Support selective retirement

**Own:** a new consolidation ADR/scoped governance review; GOVERNANCE.md, CONTRIBUTING.md,
`.claude/rules/decisions.md`, `.codex/skills/adr/`, `docs/adr/template.md`, `scripts/adr.py`,
relevant setup fixtures and the ADR index through its generator. Touch agent hooks only if
the authorized retirement workflow needs it; the existing design-edit path may suffice.

1. Record Decisions above: bulk retirement, current rationale, immutable-record replacement,
   numbering gaps and no inferred backlog. Review that policy against the selected standard;
   this is not a process pilot or renewed approval of Plan 17's method.
2. Replace contiguous-number validation with unique well-formed IDs. Keep maximum-plus-one
   allocation and highest-issued-record retention. Test sparse IDs and allocation after
   retirement; add no parallel numbering registry.
3. Require symmetric supersession for records retained as an actual local pair. For wholesale
   retired history use the consolidation policy, not dangling supersession fields. Do not
   weaken consistency checking for retained pairs.
4. Add a small explicit review-reference form for immutable historical Git citations only
   if a retained record needs it. Otherwise keep local references. Check shape/existence,
   not remote availability or historical semantics.
5. Keep accepted retained bodies immutable. Replace records whose obsolete dependencies
   prevent useful current explanation. Update checklist/index wording to the current set.

**Acceptance/deletion:** focused fixtures cover gaps, no ID reuse, retained-reference
failures, retained-pair consistency and rejected accepted-body rewrites. The ADR index
lists retained decisions. No mandatory per-file retirement matrix or individual old-ADR
review is introduced.

### MD02 — Extract only knowledge with a current consumer

**Own:** selected enduring docs and this plan's short temporary-exception table. Read
source/tool consumers only where needed to settle retention.

1. Use headings, inbound links and consumer searches, not an end-to-end reading of every
   old file. Start with Plan 16's completed contracts/dispositions and final qualification,
   Plan 18's enduring publishing guide, and relevant ADRs 0082–0095. Consult Plan 14/M22,
   Plan 15 and earlier reviews only for a still-needed fact absent from current owners.
   These are leads, not a mandatory keep list.
2. Move architecture into MD03's owners and operating instructions into `docs/dev/`. Keep
   a compact qualification note only if present claims need it; name original conditions
   and exclusions, never infer fresh tests or transfer them to changed paths.
3. Use Plan 16's final dispositions to extract important rationale and genuine surviving
   limits. Do not copy all F/N rows into the new corpus or recreate resolved findings.
   Current contracts and scoped qualification references should stand without reading
   the originating reviews or all completed packets.
4. Check executable consumers before deleting old TOML/data/fixtures. P17's phase and
   acceptance machinery is already gone; an old manifest is no longer presumed live.
   Search current recipe/config/test readers, including constructed paths. A generic
   provenance collector observing a file is not a behavioral dependency on its content.
   Do not copy old manifests into a new current-proof inventory.
5. Preserve unique dirty/untracked material before deletion. Tracked originals can be
   recovered from a named reachable Git commit. Keep a task-local recovery copy/patch for
   working-tree-only history under ignored `build/docs/retirement/` until normal version
   control records the migration. This is not a published or permanent archive.

**Acceptance/deletion:** surviving meaning has one owner; each temporary retention has
an actual consumer/removal event. Older proposals do not become work merely by being read.

### MD03 — Replace the mixed blueprint with focused contracts

**Own:** `docs/authoritative_design/`, the residual layout tests in
`tests/governance/tests/every_crate_registered.rs`, and their now-redundant
`tests/governance/layout_additions.toml`. Use `PSE_DESIGN_EDIT=1` under MD01's recorded
purpose. Preserve concurrent amendments; never overwrite another owner's changes.

Use this responsibility map as the starting structure. Merge adjacent pages if that
improves coherent reading. Source sections are migration leads, not instructions to retain
all their content. Do not create a file per paragraph, symbol or minor implementation detail.

| Owner under `sections/` | Source responsibilities | Current explanation |
|---|---|---|
| Existing reading guide and design-change workflow | §§0.1/0.3/0.4/24.4 | Routes, authority and content lifecycle; remove completed-plan dependencies |
| `architecture-overview.md` | §§0.2/0.5/0.6/1/2 | Responsibilities, dependency direction, boundaries and design basis |
| `workspace-and-dependencies.md` | §3 | Crate roles/library choices; manifests own exact versions, not duplicate pin tables |
| `schema-and-relations.md` | §§4/6 | Declaration ownership, admission, generated projections and family purpose; link generated detail |
| `identity-and-publication.md` | §§5/20 | Identity, compatibility, exact publication, settlement and distinct retention lifetimes |
| `mathematics-and-compilation.md` | §§7/14 | Library-owned mathematics, checked transformations, structure and incremental preparation |
| `physical-semantics.md` | §§8/9 | Quantities, material/property/reaction bindings, providers and admitted limits |
| `models-and-composition.md` | §§10/11/12/22 | Templates, balances, ports, connection/admission rules and extension points |
| `numerical-execution.md` | §§15/16/17/18 | Structure, resolved numerical policy, initialization, capabilities, ownership and truthful outcomes |
| `workflows-and-results.md` | §§13/19/21 | Rust/Python workflows, dynamics/fitting, result meaning, diagnostics and public boundaries |
| `operations-and-validation.md` | §§23/24 except 24.4 | Errors, observation and testing/qualification concepts; link recipes and dev guides |
| `scope-and-open-design.md` | §§25/26 and useful appendices | Present capability limits and genuine unresolved choices; completed Plan 16 scope is not future work |

For each owner:

- Write the current contract and why it is useful. Consult the implementation owner and
  relevant source/tests where uncertainty matters; do not generate structural proof from
  the whole codebase. Module/entrypoint pointers suffice for discovery.
- Describe the implemented Plan 16 baseline and its recorded limits. Label genuinely new
  proposals explicitly; do not downgrade completed packets to future work. Keep a short
  qualification reference where needed, not a status row per symbol.
- Remove displaced MathIR/evaluator, production Pyomo/NL and obsolete pass/campaign
  instructions. Keep surviving semantic requirements when their old mechanism has gone.
  Readers must not apply a stack of later overrides to discover the current meaning.
- Preserve useful section identities and update retained citations. A retired mechanism
  gets a concise retirement pointer where a useful old URL needs one, not an active
  normative section or an invented current equivalent.
- Keep `blueprint.md` as a compact overview/revision/compatibility entry. Old-anchor stubs
  may remain outside Current search; a competing legacy specification may not. Old
  revision narrative is recoverable in Git, not required reading.

Before removing the old layout table, replace its two document-parsing assertions with
workspace-membership checks against Cargo's actual workspace membership. Reuse the
existing `common::metadata()`/manifest helpers and crate-directory discovery. Preserve
the checks for manifest inheritance and validation-feature declarations. Retain a control
that an unregistered crate is detected; do not just delete coverage with the old table.
Then remove `blueprint_crates()`, the layout amendment reader and its TOML after their
last consumer moves. No new workspace registry, generated manifest or document parser.

The pin-table parser and its exemption inventory were already removed in P17. Preserve
`dependency_pins.rs` and `family-check`; remove redundant prose pin tables without adding
replacement checks. Use the existing shared section resolver for all moved identities.
Keep published owners for surviving numbered sections: excluding an owner while leaving
it in the derived directory would create a broken link. Publish necessary compatibility
stubs as Reference rather than making a second owner or requiring a redirect service.

**Acceptance/deletion:** S2/S4/S6 hold; surviving sections resolve once. No legacy normative
body remains just because an old ADR cited it. Temporary tool inputs are explicit and
outside default reading paths.

### MD04 — Remove retired content and obligations

**Own:** retired plans/reviews/ADRs/old-standard files, stale links, register content and
narrow register validation changes required for legitimate retirement.

1. Remove retired narratives after useful knowledge and references move. Remove local
   assets only when no current tool/skill/doc consumes them. Repair inbound links in
   retained Markdown, instructions, templates and source comments to a current owner or,
   when necessary, a specific immutable historical source.
2. Delete the old charter/directive/template and obsolete crosswalks once live citations
   move. Keep the selected standard directly readable, with no dead lineage references
   in its core, binding or skills and no change to substantive review obligations.
3. Curate ADRs by applicability. Obsolete accepted records do not belong in Current search.
   Keep a useful early decision if its rationale survives; age is a triage cue, not a verdict.
4. Remove closed register history and stale deferrals, including work satisfied by Plan 16.
   A real unresolved choice retains one owner/trigger. Permit an intentionally empty
   register. Preserve genuine external/release constraints without manufacturing work
   from historical review dates.
5. Change only retirement, reference and empty-register validation needed here, with
   focused fixtures. Do not assign new work to completed P17 or expand this migration
   into redesigning the general check runner or qualification system.

**Acceptance/deletion:** retired content leaves the working corpus and publication; it is
not simply relabelled History. Links resolve. No active finding or useful product capability
was discarded to pass a documentation check.

### MD05 — Align publication and active instructions

**Own:** `docs/site.toml`, necessary publisher/search UI changes, active README routes,
AGENTS.md, CONTRIBUTING.md, GOVERNANCE.md, PR/issue templates, shared documentation/decision
rules, process skills and canonical agent roles.

Plan 18 already aligned these surfaces with modular publication. Audit their existing
content first and edit only changed routes, current-baseline claims and retirement rules.
Do not rewrite each file, reinstall tools or regenerate unchanged role configuration
merely because it appears in the ownership list.

1. Publish current contracts as Current and library/generated detail as Reference. Keep
   temporary execution inputs and compatibility material out of default routes/search.
2. Retain collection discovery/generated navigation. Configure meaningful current-work
   and scope exceptions without a new hand-maintained global page catalog.
3. Reconcile `docs/dev/architecture-acceptance.md`, `validation-assessment.md`,
   `simulator-acceptance.md`, `native-workflow.md` and related guides by actual consumers.
   Remove old phase/receipt/architecture-manifest instructions and document the existing
   ordinary command surface where relevant. Preserve exact command scope and limits;
   do not introduce aliases for commands P17 already removed.
4. Update ADR/design-review skills and relevant canonical `.claude/agents/` roles; generate
   Codex roles with `just agent-config-sync`. The process-simulator skill inherits core
   context rules. Repair local library-skill pointers only when they name retired files;
   do not rewrite unrelated research or make local-only skills tracked by accident.
5. `CLAUDE.md` keeps importing `@AGENTS.md`; verify the import/aliases rather than duplicate
   policy. Root/architecture/plans/dev/ADR/review READMEs route to owners. `STATUS.md` must
   not resurrect an old resume path; retire it if it contains only historical status and
   update its inbound links instead of creating a second current-status owner.
6. Update plan scaffold and closure guidance: move enduring meaning to its owner, remove
   completed work from Current, and retire resolved reviews/plans when their dependencies
   are discharged. No mandatory monthly archival job or minimum-retention bureaucracy.

**Acceptance/deletion:** S1–S7 have actual routes/instructions. No active instruction requires
a Plan 17 pilot, superseded phase, proof manifest or historical chain before ordinary work.

### MD06 — Qualify the reading surface

**Own:** one scoped migration qualification and repairs at their actual owners. Use
Verification below; rerun affected checks after failures, not unrelated campaigns.

Walk these journeys: find the current implemented architecture and scoped qualification
basis; distinguish completed Plans 16–18 from newly authorized work; find current
model/publication/Python rationale; follow a moved section; confirm removed material
appears in neither Current nor Everything. Existing Plan 18 fixtures cover page discovery
and failed-build recovery; extend them only for changed publication behavior. Inspect a
representative small and large migrated page in the browser.

MD02–MD04 should have discharged temporary retention before this check. Resolve an
observed residual consumer at its named owner; do not call this partial pending P17,
which is already complete.

### MD07 — Verify retirement completion and close

**Dependency:** MD06 passes. No external P17/P18 workstream remains to await or rerun.

1. Confirm MD02–MD04 removed unconsumed acceptance inputs and the residual layout-table
   coupling, using their focused results and caller searches. Do not use completed-plan
   status alone as proof that a particular reader was deleted.
2. Confirm only deliberately retained current reference/qualification material remains.
   Preserve raw reports supporting selected Plan 16/M22 claims; ignored product evidence
   and build caches are outside this cleanup. Do not retain whole narrative collections
   merely because one result still matters.
3. If a final correction changes docs or tooling, rerun affected checks only. Otherwise
   use MD06's result; there is no second qualification campaign at closure.
4. Mark this plan done when temporary exceptions are discharged and reader journeys pass.
   Remove it from Current. Its enduring policy already belongs in the workflow/contract
   owners; after recording Outcome, its narrative follows the same retirement rule.

### Temporary exceptions — initial handoff

These are dependencies, not a file-by-file archive inventory. Narrow or remove them;
never use one input to justify keeping an entire historical plan family.

| Material | Current consumer / reason | Removal event and owner |
|---|---|---|
| Blueprint layout table and `layout_additions.toml` | `every_crate_registered.rs` still reads them | MD03 replaces only that document coupling with Cargo membership checks, then removes the duplicated declarations |
| Plan 16/18 narratives and earlier records used during extraction | Current rationale or qualification facts not yet transferred to their enduring owner | MD02 transfers selected content; MD04 retires narratives after retained links are repaired |

The old phase/acceptance-manifest reader, blueprint pin-table reader and pin exemption
inventory are already gone. They are not retention exceptions or pending P17 work. If
MD02 finds a different live consumer, identify its exact input and bounded migration
owner here rather than restoring the superseded broad exceptions.

## Finding dispositions

No new product review is commissioned or inherited here. Plan 16's recorded dispositions
and independent closure remain historical facts. Record a migration-specific finding only
if actually raised; do not recreate its completed F/N ledger or require a finding per file.

| Finding reference | Scenario reference | Disposition | Decision / work owner | Evidence or revisit trigger |
|---|---|---|---|---|

## Verification

**Proposed:** the executor's checks below are not implementation receipts. Baseline is
zero errors/warnings within the migration scope.

- While changing tooling, run focused stdlib fixtures for altered ADR/register/publisher
  behavior. Reuse `scripts/tests/test_docs.py` and `scripts/tests/docs_integration.py`
  through `just docs-test`; add cases only for genuinely changed behavior. Do not test
  exact prose or keep a hard-coded retired-file list as an architectural acceptance test.
- Run `just adr-index` and `just agent-config-sync` when their inputs change. Regenerate
  product reference only if its generator actually changes.
- For MD03's residual layout-test change, use `just check-package pse-tests-governance`
  and `just test-package pse-tests-governance --test every_crate_registered` with the
  recipe's validation features, adapting the binary name if deliberately renamed. Cover
  missing membership and valid registration while preserving metadata/feature tests.
  These are integration-test binaries, not `--lib` units. The old phase guard is gone.
- At MD06 run `just docs`, `just adr-lint`, `just lint-agents`, `just setup-test`, the full
  offline link/fragment command below, and applicable formatting/lint for changed tooling.
  Spelling, license and whitespace checks cover this migration. Preserve unrelated dirty
  Rust/Python work; report its failures separately.

```bash
lychee --offline --no-progress --include-fragments --exclude-path docs/book/404.html docs/book
```

Use the existing 404 exception only. Check changed source-only root/skill local links too.
ADR IDs and section ownership use existing mechanical validation, not a new architecture-
truth checker. When no recipe covers a focused fixture, add/use a short documented command
rather than a private verification framework.

**Proposed manual acceptance:** inspect S1–S7 and scoped Pagefind results on the built
artifact, including the deployment prefix. Confirm a fresh build removes stale pages.
This is migration usability checking, not a design-review process pilot. Counts can
summarize reduction; no percentage, page quota or context benchmark is a completion gate.
Reuse Plan 18's recorded browser/tool qualification; repeat detailed keyboard/theme/loading
checks only if those components change or a regression is observed. Its measured search
latency and test counts are neither fresh evidence nor targets for this migration.

No full workspace build, solver suite, Python extension refresh, scientific/performance
campaign, remote CI, release or deployment is required merely to migrate documentation.
Plan 16's completed local qualification and independent review remain at their recorded
scope. Documentation retirement neither reruns them nor broadens their claims.

## Open items

The target and retirement defaults are selected. MD02 settles only current applicability
and live-consumer questions using relevant source and any currently active implementation owner.
The executor need not ask whether every old document deserves preservation.

There is no planned wait for Plan 16 or Plan 18. Their implementations are complete.
The remaining layout-test coupling and register/ADR retention mechanics have explicit
owners in this plan. If an unexpected product dependency is discovered, record that
specific issue and continue independent migration work; do not manufacture whole-plan
closure or revive a completed workstream as a placeholder blocker.

This planning task changes only this document. Implementation, retirement, policy adoption
and current-route edits begin when execution is requested. Commit, push, history rewriting
and deployment are not implied.

## Outcome (recorded after implementation)

### What was built

Pending execution. Record the current corpus, retired mechanisms and scoped results here.

### A mistake made and corrected

Pending execution; record an actual correction, not an invented lesson.

### Deviations from the plan, deliberate

Pending execution. Explain any retained exception and its concrete consumer; do not silently
convert temporary legacy retention into the permanent target.
