# A. Content model and consumer journeys — supporting evidence

**Role.** Bounded code-mapper inquiry supporting the coordinator's design-tier, target-purpose
review (Core 3.4) of the repository's reference-content system. It establishes facts only:
the content model the guidance already implies, how the corpus realizes it, and how agents
and tools consume it. It carries no verdict; "Observation" marks a possible issue for the
principal reviewer to judge. **Baseline:** dirty `main` at
`4c24721e691187e1a5b28398b29722fbde671da8` (256 changed/untracked paths at inspection,
2026-10-09). Untracked inputs relied on: `docs/lifecycle.toml`, `scripts/document_lifecycle.py`,
`scripts/document_metadata.py`, Plans 32/33, ADR-0168–0172, the efficiency review and its
evidence dir, both workspace-content-lifecycle reviews and the handoff, plus four
2026-10-09 target reviews. One read-only projection was run:
`scripts/pse-env --docs --resource-class light -- python3 -m scripts.document_lifecycle scope`
(228 items; the default resource class was refused by admission). No product tests ran.

---

## 1. Element catalog

Obligation strength: **M** mandatory (stated as must/required or enforced by tool),
**S** suggested (explicitly adaptable), **P** practice only (appears in corpus, no guidance).

### 1.1 Review-side elements

| Element | Guidance source(s) / strength | Identity form and stability | States, transitions, who changes | Relations (side that writes) | Consumers |
|---|---|---|---|---|---|
| **Design review (principal)** | design-review SKILL "Deliver the review", "Organize the output"; template Part 1 "Organize the review…"; binding *Reviews in this repository* (location M). Front matter **not required** anywhere (`.claude/rules/docs.md` *Conventions* and AGENTS.md *Doc conventions* list ADRs, plans, capability maps, authoritative design only) | Filename `design_review_{slug}_{YYYY-MM-DD}.md` (binding, M). Slug free-form; same-day multiple reviews of one subject distinguished only by slug suffix (`full-analysis-retirement-target` vs `…-independent-target`; `workspace-content-lifecycle` vs `…-target`) | Verdict fixed at publication (template "Historical reviews retain…"); retires to Git when findings dispositioned (binding *Lifecycle*, M). No state field; 6 reviews carry an undefined `status: review` key | → disposition owner: written **review side** at publication (front matter `disposition_owner`/`disposition-owner`, §1 table row, or bold field) and not updated on adoption. ← plan `review_sources` (plan side). ← ADR `review:` (ADR side). ← indexes (hand prose) | Agents reading; `scripts/docs.py` (footer shows front-matter `status`); Pagefind; lychee; `document_lifecycle` inventory (role `review` from collection default) |
| **Supporting analysis** | design-review SKILL ("supporting documents identify their narrower role, baseline… route back"); template ("optional… does not require a new evidence folder") — S | Evidence-dir file names free; local IDs (`BF/PP/EX` in efficiency evidence) declared "local to these inquiries" in prose | None | → principal: prose link (supporting side). Principal → supporting: prose link | Agents only. **Not machine-distinguishable** from principal: no role field; lifecycle bundle default makes everything under `evidence/` `doc_role=evidence` and READMEs are self-owned `index` exceptions |
| **Finding** | Template *Finding standard* (ID "stable within the review; linkable anchor such as `f01`" — M for ID, S for anchor form); slot 7 table header shows `F01` with anchor `f01` | Review-local `F01`/`f01`. Not globally unique (see §6). 15/55 reviews have no anchors at all | No state in the review ("Status belongs to the disposition owner", template slot 7) | Plan disposition row links `review#fNN` (plan side) | Plan authors; lifecycle `scope_tables` read the plan row, not the review |
| **Scenario** | Template slot 4 (`S01`… stable local IDs, S); binding PSE-S01–S07 seeds | Review-local `Snn` (89 anchors) or binding `PSE-Snn` | None | Plan `scenario_sources` front matter + disposition "Review scenarios" column (plan side) | Plan authors |
| **Rule impact RCnn** | design-review SKILL *Identify rule impacts* ("stable identifier `review#RCnn`", M); binding *Routes for required changes*; template slot 11 does **not** name RC IDs (only REFERENCE §3 table does) | Review-local `rcNN` anchor (57). Same `RC01` recurs across reviews; plans qualify in prose ("Parallel RC01", "Graph/hash RC04", "WCL-RC01", "preparation RC01–RC04") | Proposed → confirmed/rejected by operator in plan creation (create-plan *Confirm rule changes*, M). Confirmation recorded only in prose | Plan rule-change table links `review#rcNN` (plan side); review may append handoff text (efficiency review §10 *Adoption handoff*); blueprint revision rows cite qualified RC labels | Plan authors; no tool |
| **Operator confirmation / authorization** | create-plan: "Record the outcome… item, decision and its date, and the route" (M content, S form) | None; prose sentences ("**Accept, 2026-10-05**") | Implicit | Plan section (plan side); also restated in review appendix, `docs/design_review/README.md`, `docs/plans/README.md`, blueprint revision row, memory files | Agents |

### 1.2 Plan-side elements

| Element | Guidance / strength | Identity | States / who changes | Relations (writer side) | Consumers |
|---|---|---|---|---|---|
| **Plan (coordinator or standalone)** | `docs/plans/README.md` *Norms* (M: naming, front matter `title,status,date,adrs`; optional `review_sources`,`scenario_sources`); `.claude/rules/decisions.md` *Plans and the register*; create-plan + `references/plan-structure.md` (S content); `just plan` skeleton (justfile `plan slug:`) | `NN-kebab.md`, number allocated by `just plan` from retained + Git-added names; never reused | `draft|in-progress|done|abandoned` (plans README only; **no validator** — not in `scripts/document_metadata.py` ENUMS). Edited by executing agent | → ADRs `adrs:`; → reviews `review_sources:` (both plan side). ← `docs/site.toml` `current_work` glob; ← `docs/lifecycle.toml` exceptions/scope_tables/relationships (external) | `scripts/docs.py` (scope, footer); `document_lifecycle`; agents |
| **Companion plan** | plans README (letters, "`just plan` does not allocate"); create-plan *Organize documents* (S) | `NNx-kebab.md`, hand-allocated | Same front matter | → coordinator: prose; dependencies in `lifecycle.toml [[relationships]]` (6 rows, external) | Same |
| **Packet / package** | create-plan uses "work package(s)" (S, "stable identifiers when other documents refer"); decisions.md + plans README use "packet" (M: targeted tests + deletion); `just plan` skeleton column "Status or status-owner link" | Free per plan: `A1`,`C8`,`N0–N13`,`L7`,`GH1`,`R0`,`WC00`,`EFF00`,`EI01`,`P1`. Collisions across plans (§6) | **No defined vocabulary**; free-text cell; 80 distinct raw values in projection | Prerequisites in prose/table cells; cross-plan deps in `lifecycle.toml` (external) | Lifecycle regex `status_profiles.packet` |
| **Disposition row** | Template slot 11 fields (S: "Suggested dispositions…"); decisions.md (M: "Use open, scheduled, resolved, deferred, superseded or disproved"); binding *Follow-up ownership* | Keyed by review finding link; cross-review label prefixes in prose | Written by coordinator only ("single writer", roles README) | → review finding (plan side) → packet owner (cell text) → evidence (link to companion Outcome) | Lifecycle `status_profiles.finding` regex |
| **Checkpoint** | ≥6 phrasings: AGENTS.md *Execution rhythm*; decisions.md *Execution rhythm in plans*; plans README *Execution rhythm*; create-plan *Establish completion*; execute-plan; plan-structure.md *Transitions* (M existence for resume; S content) | Heading variants: "Current checkpoint", "Checkpoint", "Checkpoint and next step", "Open items and current checkpoint", "Implementation checkpoint — 2026-10-09", "Execution checkpoint (2026-10-07)", "Authoring checkpoint" | Overwritten/appended by executor; Plan 28's is ~275 lines incl. "Earlier implementation handoff" | Links handoffs, companions | Resume journey (agents) |
| **Outcome** | plans README (M: `## Outcome (recorded after implementation)` + three sub-headings, evidence labels); decisions.md ("append `## Outcome`"); AGENTS PD4; `just plan` skeleton | Heading: 13× canonical, 3× "Outcome", plus "Investigation Outcome — 2026-10-09", "Followup verification and outcome — 2026-10-09" | Intended at close; realized as interim (Plan 28 Outcome: "This is an implementation checkpoint"), placeholder (Plan 32) | Disposition rows link companion Outcome anchors as evidence | Agents; dispositions |
| **Handoff** | None specific (skills README: "no separate root STATUS file or handoff skill") — P | Free location: plan subsections ("Earlier implementation handoff", "Efficiency review handoff, 2026-10-07"), evidence file (`evidence/workspace-content-lifecycle-2026-10-09/handoff.md`, the only doc with `doc_*` front matter besides one review) | Snapshot | → plan (handoff side) | Resume |

### 1.3 Decision/architecture elements

| Element | Guidance / strength | Identity | States / who changes | Relations | Consumers |
|---|---|---|---|---|---|
| **ADR** | adr SKILL; `docs/adr/template.md`; decisions.md (M, linted) | `ADR-NNNN` = filename number, allocated by `scripts.adr new`, never reused | `proposed→accepted|rejected|deprecated|superseded` via decision PR; immutable except status, `superseded-by`, Status history, reference relocation (`scripts/adr.py` `MUTABLE_KEYS`) | `review:` (ADR side), `blueprint:` (ADR side) ↔ section `> Decision:` marker (section side) ↔ revision row (blueprint side): three authored directions; supersedes pair maintained by `just adr-supersede`; plan link in More information (ADR side) ↔ plan `adrs:` (plan side) | `scripts/adr.py` lint/index; `scripts/docs.py` (status→search scope); agent-hooks (protects non-proposed) |
| **Status-history entry** | adr SKILL *Immutability*; decisions.md ("append-only") — M | `- YYYY-MM-DD — text` | Append only | — | Humans |
| **Register row** | `docs/adr/register.md` header; decisions.md; adr SKILL (M) | `R-NN`, high-water mark R-52 linted | `open|watch`; removed when decided | → ADR (register side); ADR `revisit` (ADR side) | `scripts/check_register.py --lint/--due` |
| **Architecture section §** | docs.md, adr SKILL rule 3 (M: insert, never renumber) | `§N.M` stable; publisher anchor `#section-N-M` | `status: current` front matter | `> Decision:` marker | `scripts.adr` resolver; publisher |
| **Revision row** | adr SKILL rule 2; decisions.md (M) | Monotonic integer (now 144) | Append | Cites ADR + prose RC labels | Humans |
| **Evidence bundle / item** | binding ("supporting evidence files under `docs/design_review/evidence/`"); template ("does not require a new evidence folder"); library-research role (dated topic folder + README for "library-evidence locations this repository's AGENTS.md names" — **AGENTS.md names none**) | Dirs `topic-YYYY-MM-DD/` (14) or single files (2). Slugs differ from consuming review (`execution-efficiency-2026-10-05` vs review `execution-efficiency-and-surrealdb`); several are plan-produced (`graph-hash-followups`, `native-producer-inputs`, `canonical-selection`). 3 dirs lack README | `lifecycle.toml` bundle default `evidence/observed/protected-unknown`; READMEs exceptioned as self-owned `index` | Consumer link prose only; publication via `site.toml [[asset_bundles]]` + `publication.asset_bundles` list (two edits) | Publisher; capability maps |

### 1.4 Guidance elements

| Element | Source | Identity / versioning | Notes |
|---|---|---|---|
| **Skill** | `.codex/skills/<name>/SKILL.md` (aliases `.claude/skills`, `.agents/skills`) | Front matter `name, description` (plan skills) or + `allowed-tools, user-invocable, model-baseline: claude-5 (2026-08)` (adr, design-review, process-simulator). No version; `model-baseline` has no tool consumer (searched `scripts/`, `scripts/tests/`, `xtask/`) | Git history only for revisions |
| **Role** | `.agents/roles/*.md` + native adapters `.claude/agents/*.md` (YAML `model`,`effort`) and `.codex/agents/*.toml` | Model/effort duplicated in roles README table and adapter files | `library-research` forbids editing "STATUS" (no such file) |
| **Rule file** | `.claude/rules/*.md` (`description`, `paths` globs) | Unversioned | Routing duplicated in AGENTS.md scope table (§5) |
| **Standard documents** | core principles/template ("Version 3.4 · 2026-10-05"), heuristics ("Version 1.0"), profile 1.5; `standard.toml` selects core/profile versions but **not** heuristics version | In-document version line | Version numbers also copied into skills README, design-reviewer role, design_review README, §24.4 |

---

## 2. Consumer journeys (what an agent reads, decides, edits today)

**J1 Resume paused work.** Read: `docs/plans/README.md` (prose, ~90 lines of routing for
Plan 28 alone) → plan's checkpoint (variant heading) → linked companion checkpoints →
handoff wherever it lives (Plan 32: evidence-dir `handoff.md`; Plan 28: "Earlier
implementation handoff" subsection; 28e: "Qualification handoff…", "Parallel extension
acceptance…") → actual tree. Private memory (`MEMORY.md` lists "Plan 22 resume state",
"Plan 23 domain model state") is a parallel off-repo resume store. Decide: which statement
is current when coordinator and companion disagree. Example: Plan 28 checkpoint assigns the
PC-SAFT refusal to "Plan 33/EFF06"; 28k checkpoint says "owner at 28f". Plan 33's packet
section says "implementation has not started" while its Progress cells say Integrated.

**J2 Adopt a review into a plan (RC confirmation).** Read review §rule impacts + findings;
present each RC to operator (create-plan). Edit: plan rule-change section (heading varies:
"Rule changes confirmed by the maintainer", "Operator-confirmed rule changes", "Confirmed rule
changes and adoption routes"); disposition table; `review_sources`/`scenario_sources`;
`docs/plans/README.md` prose; `docs/design_review/README.md` prose; `docs/site.toml`
`current_work`; `docs/lifecycle.toml` exception (`doc_owner` = plan itself) + one
`[[scope_tables]]` per table with exact heading and exact header list + optional
`[[relationships]]`; optionally the review itself (efficiency review gained "Adoption handoff —
2026-10-09" restating Plan 33's confirmations); ADRs via `just adr-new`, blueprint revision
row, `> Decision:` marker, register rows. The review's own proposed-owner table (efficiency §10:
Plan 28h/28e/28f…) is left stale and contradicted by its appendix. Supporting evidence
(`build-and-validation.md`) still says "Plan 28 owns existing efficiency finding dispositions".

**J3 Update packet status in a companion while the coordinator owns dispositions.** Edit
companion packet cell (free text), companion checkpoint, possibly companion Outcome; decide
whether the coordinator's disposition cell changes (e.g., Plan 28 "Enhancement F01 —
Implemented; focused controls pass, E3/E4 pending" duplicates package state in a disposition
cell). If a table heading/header is reworded, the `lifecycle.toml` binding must be edited too
(headings and header lists are matched literally; binding names are heading slugs and
`[[relationships]]` reference `doc::binding-slug::ID`).

**J4 Resolve/close a finding.** Edit disposition cell (observed forms: "Resolved",
"resolved", "Resolved, 2026-10-09", "Resolved for V2 sequence framing, 2026-10-09"), link
evidence (companion Outcome anchor). Review text unchanged. Nothing computes "all findings of
review R dispositioned" — review→finding membership is implicit in link anchors in first
cells; one disposition table may hold several reviews' findings. Plan 29 is `status: done`
yet its dispositions still read "scheduled" (20 items project as open).

**J5 Retire a plan/review.** `docs/dev/documentation.md` *Lifecycle and retirement*: move
enduring meaning; remove from `current_work`; search inbound links incl. constructed paths;
repair to owners or immutable permalinks; ADR `review:` relocation to `git:<commit>:<path>`;
keep highest-numbered plan. Also (unstated there) remove `lifecycle.toml` exceptions/bindings
and index prose. Tool: `document_lifecycle retire-plan` emits candidates but always lists
blocker "owner release not established" (hard-coded) and covers plans only; no review
retirement preflight exists.

**J6 What's open across plans.** `document_lifecycle scope --state open` (documented in
documentation.md). Projection at baseline: 228 items — findings 65 open/22 complete/3
deferred/8 unknown; packets 64 open/15 complete/51 unknown. Unknown arises from bound tables
with no status column (all 30a–d, 28i, 28j packet tables; Plan 33 investigation packets whose
status is prose "All are planned") and from plans without bindings. Lexical misreads:
"Investigation complete; … E3/E4 pending" → open; "disproved as thrash; open as cost" → open.
Done Plans 29/30/31 still contribute items. Native IDs are not plan-unique (§6).

**J7 What do we already know about X.** No topic index for reviews: `lifecycle.toml` topic
vocabulary has 10 terms; reviews get only collection default `design`. Routes: Pagefind
(`just docs`, published scope; plans collection defaults to History unless in
`current_work`), grep, `docs/design_review/README.md` (curated prose covering 8 of 55
reviews), `docs/library-utilization.jsonl`/library-catalog MCP (libraries only),
capability maps (`just lib-outline`). 32 of 55 reviews have no inbound link from any plan,
index, skill or rule (only from ADRs and/or other reviews, or none).

**J8 Create a new evidence bundle.** Decide folder vs single file, slug (review slug or
topic), date suffix, README presence/role, whether to add `lifecycle.toml` README exception,
and whether to publish non-Markdown assets (two `site.toml` edits). No recipe.

**J9 Create a new review.** Decide slug/date, front-matter keys (none required), how to
record standard version, verdict casing, disposition owner field name, finding/RC/scenario
ID prefixes and anchors, heading per slot (template slots are "content references", S),
principal vs supporting split, and index prose. No scaffold recipe (`just` has `adr-new`,
`plan`; nothing for reviews or evidence).

---

## 3. Mechanical decisions made ad hoc, with drift

| Form | Decision | Observed drift |
|---|---|---|
| Review | Front matter presence | 28/55 have YAML; others use §1 Markdown table (agent-coordination) or bold-field paragraph (plan-28-completion, full-analysis-retirement-independent-target) |
| Review | Keys | 20 distinct keys; `disposition_owner` (5) vs `disposition-owner` (2); `profile` vs `profiles`; `decision` (21) vs `status: review` (6) vs neither; `architectural-fitness`/`behavioral-adequacy` (2) |
| Review | Values | `standard`: `core-3.3`, `core-3.3/process-simulator-1.3`, `core-3.4`, `Core 3.4`, `Core 3.4; Heuristics 1.0; Process Simulator 1.5`; `decision`: `Accept`/`accept`, `Revise`/`revise`, `Not Accept`; `purpose: conformance` in 4 (binding: "conformance purpose is not used") |
| Review | Headings | 268 distinct H2 titles; 35 variants carrying the decision, 21 carrying disposition/rule impacts ("Authority changes and disposition" vs "Rule impacts and disposition" vs "Rule impacts and follow-up ownership"…), 23 "Findings…" variants; slot numbers shift (efficiency review §10 = template slot 11) and plans link number-bearing anchors (`#11-rule-impacts-and-disposition`) |
| Review | IDs/anchors | Anchor prefixes: `f` (213), `s` (89), `rc` (57) plus ad hoc `c, ip, fu, fs, t, wp, ef, pe, l-n, l-c, l-d, us, pa, af-, ca, ca-f, adr-, decision, resolution-check, remedy-acceptance`; heading-text anchors also used (`#u01--flow-fingerprint-structural-framing-…`); visible ID `WCL-F01` over anchor `f01` |
| Plan | `review_sources` path form | Repo-relative `docs/…` (28*, 29, 30*) vs source-relative `../design_review/…` (31–33); inline vs block YAML list (31) |
| Plan | Skeleton | `just plan` emits Context/Decisions/Drivers/Plan/Finding dispositions/Verification/Open items/Outcome (Plan 32 follows it); plan-structure.md suggests a different outline (Plan 33 follows it) |
| Plan | Packet table | ~25 header variants in 28+ plans; "Package" vs "Packet"; "Status" vs "Progress" vs none |
| Plan | Lifecycle registration | `lifecycle.toml` holds 48 `[[scope_tables]]`, 48 `[[exceptions]]` (each 28+ plan self-owned), 6 `[[relationships]]`, 22 `[[legacy_yaml]]`; all hand-authored, must mirror heading/header text |
| Plan | Current-work registration | `site.toml` `current_work` (`plans/28*.md`, 32, 33) excludes done Plans 29–31 yet `docs/plans/README.md` describes 29/30 inside the active-work prose before "Latest completed plan" |
| ADR | Index | `docs/adr/README.md` (generated, modified in tree) lacks ADR-0170–0172; regenerated only by `just turn-end`/`adr-index` |
| Evidence | Layout | dir vs file; README or not; slug ≠ review slug; plan-produced bundles in the review evidence tree |

---

## 4. Vocabulary owners and divergences

| Vocabulary | Defined / interpreted at | Divergence |
|---|---|---|
| Review decision | Template *Decision rules* (Accept / **Accept scoped** / Revise / Not Accept / Revise or Reject); adr SKILL + decisions.md ("**Accept or Accept-scoped**") | Spelling; casing in front matter; "Accept at Proposed strength" qualifier in prose |
| Foundation / gate verdicts | Template Part 1 (satisfied/violated/unresolved/not applicable; pass/fail/unresolved/not applicable) | Consistent in template; corpus uses table cells |
| Disposition | Template slot 11 ("Suggested dispositions are…"); decisions.md ("Use open, scheduled, resolved, deferred, superseded or disproved"); binding ("Use the template's slot 11 fields"); `lifecycle.toml status_profiles.finding` regex (adds pending, unresolved, planned, blocked, paused, complete, done) | Suggested vs mandated; realized values include "In progress", "Implemented; …", "Settled for…", "Adopted and implemented…", "Investigation complete…" |
| Packet status | No guidance vocabulary; `just plan` header "Status or status-owner link"; `status_profiles.packet` regex | Free text (80 distinct raw values across all bound tables) |
| Plan status | `docs/plans/README.md` (`draft|in-progress|done|abandoned`); `just plan` writes `draft`; publisher shows raw value in footer | Not validated; "currentness" decided elsewhere (see §6 item 7) |
| ADR status / level | `scripts/adr.py` `STATUSES`, `LEVELS` (enforced); adr SKILL; decisions.md; template default; `scripts/docs.py` maps status→search scope | Consistent |
| Evidence labels | design-principles `## §D Evidence vocabulary` (owner); restated AGENTS.md PD4, docs.md, adr SKILL (with semantics), PR template, `scripts/adr.py` `EVIDENCE`, template *Evidence and acceptance*, binding policy table | Principles also has `## D. Execution and failure`; "principles §D" is ambiguous between two headings |
| Execution outcomes | worker.md and execute-plan (`passed/failed/blocked/not_run`); commit subjects "(checks not_run)" | Consistent |
| Register status | `register.md` (`open|watch`); decisions.md says "next-review date", register column is `next-check` | Minor |
| Retention / role / provenance / trigger | `scripts/document_metadata.py` `ENUMS`; `lifecycle.toml` defaults; target review field table | Applied by external defaults; authored `doc_*` only in handoff.md and one review body example |
| Capability-map status | `status: evidence-map` / `proposed`; docs.md says front matter required | 5 of 10 maps (excluding README) have no front matter |

---

## 5. Guidance as content

- **Duplication of rules.** ADR-0096 retirement/Git-archive rule restated in 14 files
  (AGENTS.md, CLAUDE.md, adr SKILL, docs.md, decisions.md, docs/README, plans README,
  design_review README, binding, register.md, adr/README, reading-guide, design-change-workflow,
  documentation.md) plus execute-plan closure paragraph. The "design principles together with
  Heuristics…" paragraph appears in 23 guidance files. Checkpoint definition in ≥6 places;
  Outcome definition in ≥4 (plans README, decisions.md, `just plan`, AGENTS PD4/binding).
- **Undefined cross-references.** "Review cadence" cited by execute-plan, plan-execution,
  create-plan, design-review, skills README, roles README/design-reviewer, decisions.md,
  AGENTS.md ("binding's review periods (ADR-0129)"), §24.4; the linked binding section
  defines location/skills/grounding/purpose/lifecycle but no cadence; ADR-0129 says "Preserve
  the existing cadence" without defining it. library-research cites evidence locations
  "AGENTS.md names" (none named).
- **Lifecycle routing coverage.** create-plan, execute-plan, plan-execution, AGENTS.md,
  docs.md, plans README route to documentation.md *Metadata and aggregate scope*;
  design-review, plan-design-review, the review template and binding do not (reviews have no
  lifecycle duty text).
- **Routing duplication.** AGENTS.md scope table vs `.claude/rules/*.md` `paths:` globs:
  "Recipes, scripts, build and agent configuration → tooling.md" but tooling globs cover
  `.codex/**` and `.claude/settings.json` only — not `.agents/**`, `.claude/agents/**`,
  `.claude/rules/**`, AGENTS.md, CLAUDE.md. `docs/design_review/**` gets docs.md only, not
  decisions.md. `rust-analyzer.toml` is in both rust and tooling globs.
  `scripts/check_agent_config.py` checks path references, not table/glob agreement.
- **Versioning.** Skills: no version, `model-baseline` on 3 of 8 process skills, unconsumed.
  Standard docs carry version lines; `standard.toml` selects core/profile but not heuristics.
  Reviews record standard version inconsistently (24/28 front-matter reviews, prose in
  others); ADRs optional `standard:`; plans have no guidance-version field (Plan 33 states
  "Core 3.4, Efficiency Heuristics 1.0 and Process Simulator 1.5" in prose). The binding says
  it "does not copy the version values", yet skills README, design-reviewer role,
  design_review README and §24.4 copy them.
- **Additive lifecycle paragraphs.** create-plan, execute-plan, plan-execution end with
  appended lifecycle/routing paragraphs (handoff: "additive… original content retained as an
  unchanged prefix"); current `git diff` shows these three skills plus adr changed.

---

## 6. Verification of the Background claims

| # | Claim | Result | Evidence |
|---|---|---|---|
| 1 | 53 reviews, 28 with front matter | **Corrected: 55**, 28 with front matter (27 without) | `docs/design_review/reviews/` listing; 7 reviews untracked |
| 2 | Inconsistent keys incl. `disposition_owner` vs `disposition-owner` | Confirmed (5 vs 2; 20 distinct keys) | §3 |
| 3 | ~40 heading variants | Confirmed in spirit: 35 decision-bearing + 21 disposition-bearing H2 variants; 268 distinct H2 titles overall | §3 |
| 4 | Finding IDs local f01/s01/rc01 plus ~15 ad hoc prefixes | Confirmed: 20 non-standard anchor prefixes; 15 reviews without any anchors | §3 |
| 5 | Plan 28 holds several F01–F05 sets | Confirmed: completion-audit F01–F05, "Enhancement F01–F05", "Parallel F01–F05", "Graph/hash F01–F03"; also four distinct RC01s and two U01s (graph, preparation) disambiguated only by prose prefixes | Plan 28 *Finding dispositions* and subsections |
| 6 | Principal vs supporting not machine-readable | Confirmed | §1.1 |
| 7 | ~30 reviews linked only from ADRs | Refined: 17 linked only from ADRs; 9 from ADRs + other reviews; 5 only from other reviews; 1 none → 32 with no plan/index/skill/rule inbound link (grep of basenames over `docs/` excl. `docs/book`, AGENTS.md, `.codex/skills`, `.claude`, `scripts`, justfile) | — |
| 8 | testing-architecture-conformance has zero inbound links | Confirmed under the same search scope | — |
| 9 | Plans: uniform front matter | Mostly: same keys, but path form and list style differ (§3); no `doc_*` keys in any plan | — |
| 10 | ~10 packet-table header variants | **Understated:** ~25 | §3 |
| 11 | Free-text status cells | Confirmed: 80 distinct raw statuses; case duplicates (`Scheduled`/`scheduled`, `Resolved`/`resolved`) | scope projection |
| 12 | Packet ID letters reused across 28a–d and 30a–d | Confirmed: A1–A3, B1–B3, C1–C3, D1–D3; also P1–P6 (29, 31) and F01–F05 (28, 31, 33); multi-ID rows ("AE-01, AE-24") become one native ID | scope projection `native_id` |
| 13 | Efficiency review appended "Adoption handoff" duplicating Plan 33 | Confirmed; also restated in `docs/design_review/README.md` and `docs/plans/README.md` | review §10 |
| 14 | ADR index generated but stale | Confirmed: missing ADR-0170, 0171, 0172; statuses of listed entries match files | `docs/adr/README.md` |
| 15 | "Is plan X current" decided in ~7 places | Confirmed (8): `site.toml current_work`; plan `status`; `lifecycle.toml` bindings/exceptions; regex `(?:2[89]|[3-9][0-9])[a-z]?-` in `document_lifecycle.py` (unbound-document emission); collection default `doc_topics=["current-work"]` for every plan incl. done; `docs/plans/README.md` prose; table cells; checkpoint prose ("Work is paused…"). §24.4.1 and plans README also fix "28+" as the managed range | — |
| 16 | Disposition vocabulary suggested vs mandated | Confirmed | §4 |
| 17 | "Accept scoped" vs "Accept-scoped" | Confirmed | §4 |
| 18 | package vs packet | Confirmed (create-plan/plan-structure "package"; decisions.md/plans README/just plan "packet"; tables mostly "Package") | — |
| 19 | Three checkpoint definitions | ≥6 phrasings | §5 |
| 20 | Review cadence cited but undefined | Confirmed | §5 |
| 21 | Review front matter not required | Confirmed | §1.1 |
| 22 | Outcome heading drift | Confirmed | §1.2 |

**Additional observations (possible issues, not judged).** (a) The handoff states named bundle
definitions reside in `docs/lifecycle.toml`; publication bundles are actually
`docs/site.toml [[asset_bundles]]`, while `lifecycle.toml [[bundles]]` holds metadata
defaults — two "bundle" concepts. (b) `retire-plan` can never report an unblocked candidate.
(c) Reviews carry no `doc_owner`; the `design_review` collection has no owner default, so
review retention resolves to `protected-unknown`. (d) Evidence READMEs are self-owned, so
evidence→review/plan dependence is invisible to tools. (e) Template slot 1 asks for a
"Disposition owner" at review time, before adoption; binding says "Until then the review names a
proposed work owner" — the field is predictably superseded and not updated.

**Unresolved edges.** Search coverage for inbound links used basename grep; constructed paths
in code beyond `scripts/` and justfile were not searched. Heading-variant counts depend on
the normalization used (leading numbers stripped). The projection reflects tables as bound
at baseline; unbound plan tables are absent from it.
